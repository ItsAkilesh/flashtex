"""Synthetic fleet observations only; no models, services or issues are changed."""
import copy
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
from contextlib import redirect_stdout

SPEC = importlib.util.spec_from_file_location('fleet_health', Path(__file__).parents[1] / 'scripts/fleet_health.py')
fleet = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fleet)
NOW = fleet.utc('2026-09-12T05:00:00Z')
STAMP = '2026-09-12T04:59:00Z'
MAIN = 'a' * 40
WORKER = 'b' * 40
PREVIOUS = 'c' * 40
ARCHIVE = 'coordination/completions/worker/FT-001-r1.json'


def sample():
    assignment = {'agent_id': 'worker', 'task_id': 'FT-001', 'revision': 2,
                  'branch': 'agent/worker/task', 'state': 'assigned'}
    snapshot = {'fetch': 'ok', 'main_sha': MAIN, 'authority_ref': 'origin/main', 'assignments': {'FT-001': assignment},
            'reports': {'worker': {
                'agent_id': 'worker', 'branch': assignment['branch'], 'machine': 'remote-mac',
                'state': 'in_progress', 'updated_utc': STAMP,
                'assignment_acknowledgements': {'FT-001': {'revision': 2, 'adaptation': 'Read revision 2.', 'main_sha': MAIN}},
                'process_evidence': {'machine': 'remote-mac', 'pid': 123, 'state': 'running',
                                     'observed_utc': STAMP, 'in_flight': False},
                'resource_evidence': {'status': 'available', 'observed_utc': STAMP}}},
            'next': {'worker': dict(assignment, previous_report_sha=PREVIOUS)},
            'queues': {'worker': {'agent_id': 'worker', 'next_index': 0, 'steps': [{'objective': 'Next task'}, {'objective': 'Second task'}]}},
            'issues': [], 'errors': [], 'dispatcher': {'state': 'published'},
            'services': {s: {'LoadState': 'loaded', 'ActiveState': 'active', 'SubState': 'running',
                             'MainPID': '123'} for s in fleet.SERVICES}}
    old_report = copy.deepcopy(snapshot['reports']['worker'])
    old_report['state'] = 'ready_for_integration'
    old_report['assignment_acknowledgements']['FT-001']['revision'] = 1
    snapshot['completions'] = {ARCHIVE: {
        'agent_id': 'worker', 'task_id': 'FT-001', 'revision': 1, 'state': 'reported_ready',
        'worker_branch': assignment['branch'], 'worker_report_sha': PREVIOUS,
        'assignment': dict(assignment, revision=1), 'report': old_report}}
    snapshot['completion_sources'] = {ARCHIVE: {'sha': PREVIOUS, 'report': copy.deepcopy(old_report)}}
    snapshot['ack_sources'] = {'worker/FT-001': {'main_sha': MAIN, 'ancestor_of_main': True, 'assignment': copy.deepcopy(assignment)}}
    return snapshot


class AnalysisTests(unittest.TestCase):
    def codes(self, snapshot):
        return {f['code'] for f in fleet.analyze(snapshot, now=NOW)['findings']}

    def test_fresh_remote_evidence_is_never_verified(self):
        report = fleet.analyze(sample(), now=NOW)
        self.assertFalse(report['actionable'])
        self.assertEqual(self.codes(sample()), {'remote_activity_reported_only'})
        self.assertFalse(report['workers'][0]['remote_process_verified'])
        self.assertTrue(report['workers'][0]['acknowledged'])

    def test_exact_ack_and_pointer_revision_required(self):
        snapshot = sample()
        snapshot['reports']['worker']['assignment_acknowledgements']['FT-001']['revision'] = 1
        snapshot['next']['worker']['revision'] = 3
        self.assertTrue({'ack_required', 'next_mismatch'} <= self.codes(snapshot))
        snapshot['reports']['worker']['assignment_acknowledgements']['FT-001'] = {'revision': 2, 'adaptation': ' '}
        self.assertIn('ack_required', self.codes(snapshot))

    def test_comments_summary_and_pid_without_time_cannot_prove_activity(self):
        snapshot = sample()
        snapshot['reports']['worker']['process_evidence'] = {'pid': 123}
        snapshot['reports']['worker']['summary'] = 'Running, launched successfully per GitHub comment.'
        self.assertIn('activity_unproven', self.codes(snapshot))

    def test_process_identity_machine_and_freshness_required(self):
        for changes in ({'machine': 'other'}, {'pid': True}, {'pid': -1},
                        {'observed_utc': '2026-09-12T03:00:00Z'}, {'state': 'stopped'}):
            with self.subTest(changes=changes):
                snapshot = sample()
                snapshot['reports']['worker']['process_evidence'].update(changes)
                self.assertIn('activity_unproven', self.codes(snapshot))

    def test_resource_time_is_independent_of_report_heartbeat(self):
        snapshot = sample()
        snapshot['reports']['worker']['resource_evidence'] = {'status': 'exhausted', 'observed_utc': '2026-09-12T01:00:00Z'}
        snapshot['reports']['worker']['process_evidence']['in_flight'] = 'unknown'
        self.assertTrue({'resource_stale', 'resource_exhausted', 'in_flight_unresolved'} <= self.codes(snapshot))
        self.assertNotIn('report_stale', self.codes(snapshot))

    def test_unknown_and_future_evidence_stays_unknown_or_future(self):
        snapshot = sample()
        snapshot['reports']['worker']['updated_utc'] = '2026-09-13T05:00:00Z'
        snapshot['reports']['worker']['resource_evidence'] = {}
        self.assertTrue({'resource_unknown', 'report_future'} <= self.codes(snapshot))
        self.assertEqual(fleet.freshness('2026-09-12T05:00:00', NOW, 600), 'unknown')

    def test_exhaustion_not_completion_and_invalid_queue_index(self):
        snapshot = sample()
        snapshot['queues']['worker']['next_index'] = 2
        self.assertIn('queue_exhausted', self.codes(snapshot))
        for index in (-1, True, 3, '1'):
            snapshot['queues']['worker']['next_index'] = index
            self.assertIn('queue_invalid', self.codes(snapshot))

    def test_missing_reports_duplicate_assignments_and_orphan_pointer(self):
        snapshot = sample()
        snapshot['assignments']['FT-002'] = dict(snapshot['assignments']['FT-001'], task_id='FT-002')
        snapshot['reports'] = {}
        snapshot['next']['orphan'] = {'agent_id': 'orphan'}
        self.assertTrue({'ack_required', 'report_identity_mismatch', 'multiple_active_assignments', 'next_orphaned'} <= self.codes(snapshot))

    def test_recovery_issue_detection_does_not_treat_comment_as_fix(self):
        snapshot = sample()
        snapshot['issues'] = [
            {'number': 1, 'title': '[recovery] worker FT-001-r2', 'state': 'OPEN', 'body': 'Comment says fixed.'},
            {'number': 2, 'title': 'Worker recovery needed: worker', 'state': 'OPEN'},
            {'number': 3, 'title': 'Auth unavailable', 'labels': [{'name': 'recovery'}], 'state': 'OPEN'},
            {'number': 4, 'title': '[recovery] old', 'state': 'CLOSED'},
            {'number': 5, 'title': 'Normal work', 'state': 'OPEN'}]
        findings = [f for f in fleet.analyze(snapshot, now=NOW)['findings'] if f['code'] == 'recovery_open']
        self.assertEqual([f['subject'] for f in findings], ['#1', '#2', '#3'])

    def test_local_service_and_pending_dispatch_require_review(self):
        snapshot = sample()
        snapshot['services'][fleet.SERVICES[0]]['MainPID'] = '0'
        snapshot['dispatcher'] = {'state': 'pending', 'started_utc': STAMP}
        self.assertTrue({'local_service_unverified', 'dispatcher_needs_review'} <= self.codes(snapshot))

    def test_analysis_deterministic_and_does_not_mutate_snapshot(self):
        snapshot = sample()
        original = copy.deepcopy(snapshot)
        first = fleet.analyze(snapshot, now=NOW)
        snapshot['reports'] = dict(reversed(list(snapshot['reports'].items())))
        self.assertEqual(first, fleet.analyze(snapshot, now=NOW))
        self.assertEqual(original, snapshot)

    def test_malformed_records_are_diagnostics(self):
        snapshot = sample()
        snapshot['reports']['worker']['assignment_acknowledgements'] = []
        snapshot['reports']['worker']['resource_evidence'] = []
        snapshot['queues']['bad'] = None
        snapshot['issues'] = [None, {'state': None}]
        self.assertTrue({'invalid_record', 'ack_required', 'resource_unknown'} <= self.codes(snapshot))

    def test_empty_inventory_is_not_healthy(self):
        snapshot = sample()
        snapshot['reports'] = {}
        snapshot['assignments'] = {}
        self.assertTrue({'report_inventory_empty', 'assignment_inventory_empty'} <= self.codes(snapshot))

    def test_task_identity_mismatch_is_rejected(self):
        snapshot = sample()
        snapshot['assignments']['FT-001']['task_id'] = 'FT-WRONG'
        self.assertIn('invalid_assignment', self.codes(snapshot))

    def test_current_plus_two_followup_coverage(self):
        for index, depth in ((0, 2), (1, 1), (2, 0)):
            with self.subTest(depth=depth):
                snapshot = sample()
                snapshot['queues']['worker']['next_index'] = index
                report = fleet.analyze(snapshot, now=NOW)
                queue = report['workers'][0]['queue_depth']
                self.assertEqual(queue['current_assignments'], 1)
                self.assertEqual(queue['remaining_followups'], depth)
                self.assertEqual(queue['coverage_satisfied'], depth >= 2)
                self.assertEqual('queue_low_water' in self.codes(snapshot), depth < 2)

    def test_blank_queue_placeholders_do_not_supply_coverage(self):
        for step in ({}, {'objective': ' '}, None):
            snapshot = sample()
            snapshot['queues']['worker']['steps'][1] = step
            self.assertIn('queue_invalid', self.codes(snapshot))
            depth = fleet.analyze(snapshot, now=NOW)['workers'][0]['queue_depth']
            self.assertIsNone(depth['remaining_followups'])
            self.assertFalse(depth['coverage_satisfied'])

    def test_completed_worker_is_pending_dispatch_not_executing(self):
        for state in ('ready_for_integration', 'completed', 'complete', 'integrated', 'verified'):
            with self.subTest(state=state):
                snapshot = sample()
                snapshot['reports']['worker']['state'] = state
                report = fleet.analyze(snapshot, now=NOW)
                self.assertTrue(report['actionable'])
                self.assertIn('pending_completion_dispatch', self.codes(snapshot))
                self.assertEqual(report['workers'][0]['activity'], 'completion_pending_dispatch')

    def test_completion_publication_next_ack_are_distinct_stages(self):
        snapshot = sample()
        sequence = fleet.analyze(snapshot, now=NOW)['workers'][0]['handoff_sequence']
        self.assertEqual(sequence['previous_completion']['status'], 'recorded_valid')
        self.assertEqual(sequence['publication']['status'], 'linked_on_main')
        self.assertEqual(sequence['next_ack']['status'], 'verified_history')
        snapshot['reports']['worker']['state'] = 'ready_for_integration'
        snapshot['reports']['worker']['assignment_acknowledgements']['FT-001']['revision'] = 1
        report = fleet.analyze(snapshot, now=NOW)
        self.assertEqual(report['workers'][0]['activity'], 'awaiting_next_ack')
        self.assertEqual(report['workers'][0]['handoff_sequence']['publication']['status'], 'linked_on_main')
        self.assertIn('next_ack_pending', self.codes(snapshot))
        self.assertNotIn('pending_completion_dispatch', self.codes(snapshot))

    def test_forged_completion_and_publication_links_are_rejected(self):
        mutations = [
            lambda s: s['completions'][ARCHIVE].update(worker_report_sha='d' * 40),
            lambda s: s['completions'][ARCHIVE]['report'].update(summary='Forged success'),
            lambda s: s['completions'][ARCHIVE].update(agent_id='another-worker'),
            lambda s: s['next']['worker'].update(previous_report_sha='d' * 40),
            lambda s: s['next']['worker'].update(revision=1),
            lambda s: s['completion_sources'].clear(),
            lambda s: s['completions'].clear()]
        for mutate in mutations:
            snapshot = sample()
            mutate(snapshot)
            self.assertIn('completion_publication_link_unverified', self.codes(snapshot))
            sequence = fleet.analyze(snapshot, now=NOW)['workers'][0]['handoff_sequence']
            self.assertNotEqual(sequence['publication']['status'], 'linked_on_main')

    def test_forged_ack_main_history_or_assignment_is_not_verified(self):
        for field, value in (('main_sha', 'd' * 40), ('ancestor_of_main', False), ('assignment', {'revision': 2})):
            snapshot = sample()
            snapshot['ack_sources']['worker/FT-001'][field] = value
            self.assertIn('ack_history_unverified', self.codes(snapshot))
            self.assertFalse(fleet.analyze(snapshot, now=NOW)['workers'][0]['acknowledgement_verified'])

    def test_task_branch_records_do_not_prove_publication_on_main(self):
        snapshot = sample()
        snapshot['authority_ref'] = 'HEAD'
        self.assertTrue({'authority_ref_unverified', 'completion_publication_link_unverified'} <= self.codes(snapshot))

    def test_first_revision_does_not_invent_previous_completion(self):
        snapshot = sample()
        snapshot['assignments']['FT-001']['revision'] = 1
        worker = fleet.analyze(snapshot, now=NOW)['workers'][0]
        self.assertEqual(worker['handoff_sequence']['previous_completion']['status'], 'first_assignment')
        self.assertNotIn('completion_publication_link_unverified', self.codes(snapshot))

    def test_supervisor_cycle_pid_and_usage_do_not_imply_liveness_or_quota(self):
        snapshot = sample()
        report = snapshot['reports']['worker']
        report.pop('process_evidence')
        report.pop('resource_evidence')
        report['supervisor'] = {'pid': 99, 'cycle': 2, 'assignment': 'FT-001:1',
                                'last_completed_utc': STAMP, 'state': 'model_failed'}
        report['claude_usage'] = {'remaining_quota': 'unknown', 'reported_utc': STAMP,
                                  'billing_route': 'verified_prepaid_api'}
        result = fleet.analyze(snapshot, now=NOW)
        self.assertTrue({'supervisor_assignment_mismatch', 'supervisor_needs_review', 'activity_unproven', 'resource_unknown'} <= self.codes(snapshot))
        self.assertFalse(result['workers'][0]['remote_process_verified'])
        self.assertFalse(result['workers'][0]['supervisor_history']['remote_process_verified'])
        self.assertEqual(result['workers'][0]['supervisor_history']['quota_availability'], 'unknown')


class CollectionTests(unittest.TestCase):
    def runner(self, failures=()):
        data = sample()
        records = {
            'coordination/assignments/FT-001.json': data['assignments']['FT-001'],
            'coordination/agents/worker.json': dict(data['reports']['worker'], updated_utc='2000-01-01T00:00:00Z'),
            'coordination/next/worker.json': data['next']['worker'],
            'coordination/queues/worker.json': data['queues']['worker'],
            ARCHIVE: data['completions'][ARCHIVE],
            'coordination/authority.json': {'commander_id': 'existing-leader'},
            'coordination/control.json': {'state': 'running'}}
        calls = []

        def run(argv, *, cwd, timeout):
            calls.append(argv)
            self.assertEqual(timeout, 30)
            if argv[0] in failures:
                return SimpleNamespace(returncode=1, stdout='', stderr='synthetic error')
            if argv[:2] == ['git', 'fetch']:
                output = ''
            elif argv[:2] == ['git', 'rev-parse']:
                output = WORKER if argv[-1].startswith('origin/agent/') else MAIN
            elif argv[:2] == ['git', 'ls-tree']:
                self.assertIn(MAIN, argv)
                output = '\n'.join(records)
            elif argv[:2] == ['git', 'show']:
                sha, path = argv[2].split(':', 1)
                output = json.dumps(data['reports']['worker'] if sha == WORKER else data['completion_sources'][ARCHIVE]['report'] if sha == PREVIOUS else records[path])
            elif argv[:2] == ['git', 'merge-base']:
                output = MAIN
            elif argv[:3] == ['gh', 'issue', 'list']:
                output = '[]'
            elif argv[:3] == ['systemctl', '--user', 'show']:
                output = 'LoadState=loaded\nActiveState=active\nSubState=running\nMainPID=123\n'
            else:
                self.fail('Unapproved command: ' + repr(argv))
            return SimpleNamespace(returncode=0, stdout=output, stderr='')
        return run, calls

    def test_pins_main_and_worker_and_only_read_commands(self):
        runner, calls = self.runner()
        snapshot = fleet.collect(Path('/synthetic'), runner=runner, dispatcher={'state': 'published'})
        self.assertEqual(snapshot['reports']['worker']['updated_utc'], STAMP)
        self.assertEqual(snapshot['report_sources']['worker']['sha'], WORKER)
        self.assertEqual(snapshot['main_sha'], MAIN)
        self.assertEqual(snapshot['authority']['commander_id'], 'existing-leader')
        self.assertEqual(snapshot['errors'], [])
        self.assertFalse(fleet.analyze(snapshot, now=NOW)['actionable'])
        self.assertEqual(calls[0], ['git', 'fetch', 'origin', '--prune'])

    def test_injected_gh_and_no_fetch_do_not_call_either(self):
        runner, calls = self.runner()
        snapshot = fleet.collect(Path('/synthetic'), runner=runner, fetch=False, gh_output='[{"number": 8, "title": "[recovery] auth"}]')
        self.assertFalse(any(c[:2] == ['git', 'fetch'] or c[0] == 'gh' for c in calls))
        self.assertEqual(snapshot['issues'][0]['number'], 8)
        self.assertIn('remote_freshness_unverified', {f['code'] for f in fleet.analyze(snapshot, now=NOW)['findings']})

    def test_command_failures_are_not_empty_healthy_fleet(self):
        runner, _ = self.runner(failures=('git', 'gh', 'systemctl'))
        snapshot = fleet.collect(Path('/synthetic'), runner=runner)
        self.assertEqual(snapshot['fetch'], 'failed')
        self.assertTrue(snapshot['errors'])
        self.assertTrue(fleet.analyze(snapshot, now=NOW)['actionable'])

    def test_timeouts_and_malformed_issue_output_remain_visible(self):
        def timeout(argv, **kwargs):
            raise subprocess.TimeoutExpired(argv, 30)
        snapshot = fleet.collect(Path('/synthetic'), runner=timeout, gh_output='{"not": "an issue list"}')
        self.assertTrue(any(e['reason'] == 'TimeoutExpired' for e in snapshot['errors']))
        self.assertTrue(any(e['operation'] == 'issues' for e in snapshot['errors']))

    def test_saved_snapshot_cli_uses_no_commands(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / 'snapshot.json'
            path.write_text(json.dumps(sample()))
            with patch.object(fleet, 'command', side_effect=AssertionError('command forbidden')), redirect_stdout(io.StringIO()) as output:
                self.assertEqual(fleet.main(['--snapshot', str(path), '--now', '2026-09-12T05:00:00Z']), 0)
            self.assertFalse(json.loads(output.getvalue())['workers'][0]['remote_process_verified'])


if __name__ == '__main__':
    unittest.main()
