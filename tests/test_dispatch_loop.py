"""Isolated Git integration checks; all commits here are synthetic fixtures."""
from datetime import datetime, timedelta, timezone
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import coord
import dispatch_loop as loop


class FetchRaceTests(unittest.TestCase):
    def test_linked_worktrees_share_publication_lock(self):
        import fcntl
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            common = root / 'common.git'
            common.mkdir()
            with patch.object(coord, 'git', return_value=str(common)):
                with loop.publication_lock(root / 'one'):
                    with (common / 'flashtex' / 'main-publication.lock').open('a') as other:
                        with self.assertRaises(BlockingIOError):
                            fcntl.flock(other, fcntl.LOCK_EX | fcntl.LOCK_NB)
                with loop.publication_lock(root / 'two'):
                    pass

    def test_shared_ref_compare_and_swap_race_retries_read_only_fetch(self):
        race = RuntimeError("cannot lock ref 'refs/remotes/origin/agent/a': is at abc but expected def")
        with patch.object(coord, 'git', side_effect=[race, 'ok']) as git, patch.object(loop.time, 'sleep'):
            self.assertEqual(loop.fetch_origin(Path('/tmp')), 'ok')
            self.assertEqual(git.call_count, 2)
            self.assertTrue(all(call.args[1:] == ('fetch', 'origin', '--prune') for call in git.call_args_list))

    def test_auth_or_arbitrary_lock_failure_is_not_retried(self):
        for text in ['authentication failed', 'cannot lock ref: lock exists', 'publication failed']:
            with patch.object(coord, 'git', side_effect=RuntimeError(text)) as git:
                with self.assertRaises(RuntimeError):
                    loop.fetch_origin(Path('/tmp'))
                self.assertEqual(git.call_count, 1)

    def test_repeated_ref_race_stops_after_three_attempts(self):
        race = RuntimeError("cannot lock ref 'refs/remotes/origin/x': is at abc but expected def")
        with patch.object(coord, 'git', side_effect=race) as git, patch.object(loop.time, 'sleep'):
            with self.assertRaises(RuntimeError):
                loop.fetch_origin(Path('/tmp'))
            self.assertEqual(git.call_count, 3)


class DispatcherTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        base = Path(self.temp.name)
        self.remote = base / 'remote.git'
        self.root = base / 'commander'
        self.worker = base / 'worker'
        self.run_git(base, 'init', '--bare', str(self.remote))
        self.run_git(base, 'clone', str(self.remote), str(self.root))
        self.identity(self.root)
        self.run_git(self.root, 'checkout', '-b', 'agent/commander/test')
        self.assignment = dict(schema_version=1, task_id='TASK', agent_id='worker', branch='agent/worker/task',
                               revision=1, state='assigned', owned_paths=['src/compiler'], objective='first',
                               acceptance=['test'], dependencies=[], timebox_minutes=30, allocation_id='test')
        self.queue = dict(schema_version=1, agent_id='worker', task_id='TASK', steps=[dict(
            objective='Implement second slice', acceptance=['new meaningful tests pass'], owned_paths=['src/compiler'],
            timebox_minutes=30, allocation_id='approved-worker-grant')])
        self.write(self.root, 'coordination/assignments/TASK.json', self.assignment)
        self.write(self.root, 'coordination/queues/worker.json', self.queue)
        self.write(self.root, 'coordination/authority.json', {'schema_version': 1, 'commander_id': 'test-commander', 'authority_state': 'active'})
        self.commit(self.root)
        self.run_git(self.root, 'push', 'origin', 'HEAD:main')
        self.baseline = self.run_git(self.root, 'rev-parse', 'HEAD')
        self.run_git(base, 'clone', '-b', 'main', str(self.remote), str(self.worker))
        self.identity(self.worker)
        self.run_git(self.worker, 'checkout', '-b', 'agent/worker/task')
        self.report = dict(schema_version=1, agent_id='worker', branch='agent/worker/task',
                           state='ready_for_integration', updated_utc=coord.stamp(),
                           summary='first slice implemented', tests=['cargo test passed'],
                           supervisor={'assignment': 'TASK:1'},
                           assignment_acknowledgements={'TASK': {'revision': 1, 'main_sha': self.baseline, 'adaptation': 'Accept task and its tests'}})
        self.publish_report()
        self.args = SimpleNamespace(publish=False, allocation=None, stale_seconds=600, timeout=10, commander_id='test-commander')

    def run_git(self, cwd, *args):
        p = subprocess.run(['git', *args], cwd=cwd, text=True, capture_output=True)
        self.assertEqual(p.returncode, 0, p.stderr)
        return p.stdout.strip()

    def identity(self, root):
        self.run_git(root, 'config', 'user.name', 'Synthetic test')
        self.run_git(root, 'config', 'user.email', 'test@example.invalid')

    def write(self, root, path, obj):
        coord.write_json(root / path, obj)

    def commit(self, root):
        self.run_git(root, 'add', '.')
        self.run_git(root, 'commit', '--allow-empty', '-m', 'synthetic test fixture')

    def publish_report(self):
        self.write(self.worker, 'coordination/agents/worker.json', self.report)
        self.commit(self.worker)
        self.run_git(self.worker, 'push', 'origin', 'HEAD:agent/worker/task')

    def update_main(self):
        self.write(self.root, 'coordination/queues/worker.json', self.queue)
        self.commit(self.root)
        self.run_git(self.root, 'push', 'origin', 'HEAD:main')

    def test_stale_commander_stops_before_assignment_mutation(self):
        self.args.commander_id = 'old-commander'
        with self.assertRaisesRegex(ValueError, 'authority changed'):
            loop.scan_once(self.root, self.args)
        self.assertEqual(self.run_git(self.root, 'status', '--porcelain'), '')
        self.assertEqual(json.loads((self.root / 'coordination/assignments/TASK.json').read_text())['revision'], 1)

    def test_ready_prepares_next_same_task_with_evidence_not_verification(self):
        with patch.object(coord, 'publish') as publish:
            result = loop.scan_once(self.root, self.args)
            publish.assert_not_called()
        self.assertEqual(result['prepared'], ['TASK'])
        assignment = json.loads((self.root / 'coordination/assignments/TASK.json').read_text())
        self.assertEqual(assignment['revision'], 2)
        self.assertEqual(assignment['branch'], self.assignment['branch'])
        self.assertEqual(assignment['state'], 'assigned')
        archive = json.loads((self.root / 'coordination/completions/worker/TASK-r1.json').read_text())
        self.assertEqual(archive['state'], 'reported_ready')
        self.assertTrue(archive['review_required'])
        self.assertEqual(archive['report']['tests'], ['cargo test passed'])
        self.assertEqual(archive['worker_report_sha'], self.run_git(self.worker, 'rev-parse', 'HEAD'))
        self.assertEqual(self.run_git(self.root, 'rev-parse', 'origin/main'), self.baseline)
        with self.assertRaisesRegex(RuntimeError, 'dirty'):
            loop.scan_once(self.root, self.args)

    def test_future_wrong_revision_and_blocked_reports_do_not_advance(self):
        for change in [dict(state='blocked'), dict(supervisor={'assignment': 'TASK:2'}),
                       dict(updated_utc=(datetime.now(timezone.utc) + timedelta(hours=1)).isoformat()),
                       dict(assignment_acknowledgements={'TASK': {'revision': 3, 'main_sha': self.baseline}})]:
            original = dict(self.report)
            self.report.update(change)
            self.publish_report()
            with patch.object(coord, 'publish') as publish:
                result = loop.scan_once(self.root, self.args)
                publish.assert_not_called()
            self.assertFalse(result['prepared'])
            self.assertFalse(self.run_git(self.root, 'status', '--porcelain'))
            self.report = original

    def test_stale_current_ready_report_remains_consumable(self):
        self.report['updated_utc'] = (datetime.now(timezone.utc) - timedelta(hours=1)).isoformat()
        self.publish_report()
        result = loop.scan_once(self.root, self.args)
        self.assertEqual(result['prepared'], ['TASK'])
        archive = json.loads((self.root / 'coordination/completions/worker/TASK-r1.json').read_text())
        self.assertTrue(archive['stale_completion'])

    def test_ack_requires_full_assignment_and_explicit_adaptation(self):
        self.report['assignment_acknowledgements']['TASK']['adaptation'] = ''
        self.publish_report()
        self.assertFalse(loop.scan_once(self.root, self.args)['prepared'])
        self.report['assignment_acknowledgements']['TASK']['adaptation'] = 'Accepted'
        self.publish_report()
        self.assignment['objective'] = 'Changed without revision bump'
        self.write(self.root, 'coordination/assignments/TASK.json', self.assignment)
        self.update_main()
        self.assertFalse(loop.scan_once(self.root, self.args)['prepared'])

    def test_manual_report_exact_ack_accepted(self):
        self.report.pop('supervisor')
        self.publish_report()
        self.assertEqual(loop.scan_once(self.root, self.args)['prepared'], ['TASK'])

    def test_inherited_or_forged_ack_report_rejected(self):
        self.report['branch'] = 'agent/worker/old'
        self.publish_report()
        self.assertFalse(loop.scan_once(self.root, self.args)['prepared'])
        self.report['branch'] = 'agent/worker/task'
        self.report['assignment_acknowledgements']['TASK']['main_sha'] = 'a' * 40
        self.publish_report()
        self.assertFalse(loop.scan_once(self.root, self.args)['prepared'])

    def test_dependency_not_integrated_skips_without_spend(self):
        self.queue['steps'][0]['dependencies'] = ['OTHER']
        self.update_main()
        self.args.publish, self.args.allocation = True, 'cursor-test'
        with patch.object(coord, 'publish') as publish:
            result = loop.scan_once(self.root, self.args)
            publish.assert_not_called()
        self.assertIn('dependency OTHER', result['skipped'][0]['reason'])

    def test_scope_expansion_and_missing_grant_rejected_before_writes(self):
        self.queue['steps'][0]['owned_paths'] = ['src']
        self.update_main()
        with self.assertRaisesRegex(ValueError, 'ownership'):
            loop.scan_once(self.root, self.args)
        self.assertFalse(self.run_git(self.root, 'status', '--porcelain'))
        self.queue['steps'][0]['owned_paths'] = ['src/compiler']
        self.queue['steps'][0].pop('allocation_id')
        self.update_main()
        with self.assertRaisesRegex(ValueError, 'allocation_id'):
            loop.scan_once(self.root, self.args)
        self.assertFalse(self.run_git(self.root, 'status', '--porcelain'))

    def fake_publish(self, root, args):
        # Mock ONLY: production uses coord.publish's actual Cursor and provenance checks.
        self.commit(root)
        self.run_git(root, 'push', 'origin', 'HEAD:agent/commander/test')

    def test_publish_advances_main_once_and_same_report_cannot_replay(self):
        self.queue['steps'].append(dict(self.queue['steps'][0], objective='third slice'))
        self.update_main()
        self.args.publish, self.args.allocation = True, 'cursor-test'
        with patch.object(coord, 'publish', side_effect=self.fake_publish) as publish:
            result = loop.scan_once(self.root, self.args)
            self.assertTrue(result['published'])
            publish.assert_called_once()
            second = loop.scan_once(self.root, self.args)
            self.assertFalse(second['prepared'])
            publish.assert_called_once()
        main = self.run_git(self.remote, 'rev-parse', 'main')
        self.assertEqual(main, result['commit'])
        queue = json.loads((self.root / 'coordination/queues/worker.json').read_text())
        self.assertEqual(queue['next_index'], 1)

    def test_ambiguous_publish_failure_blocks_paid_retry(self):
        self.args.publish, self.args.allocation = True, 'cursor-test'
        with patch.object(coord, 'publish', side_effect=RuntimeError('Cursor timeout')) as publish:
            with self.assertRaisesRegex(RuntimeError, 'timeout'):
                loop.scan_once(self.root, self.args)
            with self.assertRaisesRegex(RuntimeError, 'unresolved'):
                loop.scan_once(self.root, self.args)
            publish.assert_called_once()
        self.assertTrue((self.root / 'coordination/completions/worker/TASK-r1.json').exists())

    def test_main_race_never_overwrites_new_main(self):
        self.args.publish, self.args.allocation = True, 'cursor-test'
        def racing_publish(root, args):
            self.fake_publish(root, args)
            self.run_git(self.worker, 'checkout', '-b', 'main-writer', self.baseline)
            (self.worker / 'peer.txt').write_text('concurrent independent update')
            self.commit(self.worker)
            self.run_git(self.worker, 'push', 'origin', 'HEAD:main')
        with patch.object(coord, 'publish', side_effect=racing_publish):
            with self.assertRaisesRegex(RuntimeError, 'main changed'):
                loop.scan_once(self.root, self.args)
        self.assertEqual(self.run_git(self.remote, 'rev-parse', 'main'), self.run_git(self.worker, 'rev-parse', 'HEAD'))

    def test_dirty_and_noncurrent_commander_stop_without_mutation(self):
        (self.root / 'work.txt').write_text('preserve me')
        with self.assertRaisesRegex(RuntimeError, 'dirty'):
            loop.scan_once(self.root, self.args)
        self.assertEqual((self.root / 'work.txt').read_text(), 'preserve me')
        self.commit(self.root)
        with self.assertRaisesRegex(RuntimeError, 'exactly match'):
            loop.scan_once(self.root, self.args)

    def test_archive_prevents_duplicate_consumption(self):
        self.write(self.root, 'coordination/completions/worker/TASK-r1.json', {'schema_version': 1, 'state': 'reported_ready'})
        self.commit(self.root)
        self.run_git(self.root, 'push', 'origin', 'HEAD:main')
        result = loop.scan_once(self.root, self.args)
        self.assertIn('already consumed', result['skipped'][0]['reason'])

    def test_clean_commander_fast_forwards_new_main_before_dispatch(self):
        self.run_git(self.worker, 'checkout', '-b', 'main-writer', self.baseline)
        (self.worker / 'new-interface.txt').write_text('peer integration')
        self.commit(self.worker)
        self.run_git(self.worker, 'push', 'origin', 'HEAD:main')
        new_main = self.run_git(self.worker, 'rev-parse', 'HEAD')
        result = loop.scan_once(self.root, self.args)
        self.assertEqual(result['baseline_main'], new_main)
        self.assertEqual(self.run_git(self.root, 'rev-parse', 'HEAD'), new_main)
        self.assertEqual((self.root / 'new-interface.txt').read_text(), 'peer integration')
        self.assertEqual(result['prepared'], ['TASK'])

    def test_authoritative_control_stops_without_dispatch_or_spend(self):
        for state in ['user_stopped']:
            self.write(self.root, 'coordination/control.json', {'schema_version': 1, 'state': state})
            self.commit(self.root)
            self.run_git(self.root, 'push', 'origin', 'HEAD:main')
            self.args.publish, self.args.allocation = True, 'cursor-test'
            with patch.object(coord, 'publish') as publish:
                result = loop.scan_once(self.root, self.args)
                publish.assert_not_called()
            self.assertTrue(result['stopped'])
            self.assertEqual(result['reason'], state)
            self.assertFalse(result['prepared'])
            self.assertFalse(self.run_git(self.root, 'status', '--porcelain'))

    def test_published_legacy_owner_branch_remains_dispatchable(self):
        assignment_path = self.root / 'coordination/assignments/TASK.json'
        assignment = json.loads(assignment_path.read_text())
        assignment['agent_id'] = 'legacy-worker'
        assignment_path.write_text(json.dumps(assignment))
        self.commit(self.root)
        self.run_git(self.root, 'push', 'origin', 'HEAD:main')
        queue = dict(self.queue, agent_id='legacy-worker')
        plan, reason = loop.plan_step(self.root, 'coordination/queues/legacy-worker.json', queue, {'TASK': assignment}, datetime.now(timezone.utc), 600)
        self.assertIsNone(plan)
        self.assertEqual(reason, 'worker branch not published')
        self.assertTrue(coord.published_branch_assignment(self.root, 'legacy-worker', assignment['branch']))
        self.assertFalse(coord.published_branch_assignment(self.root, 'other-worker', assignment['branch']))

    def test_malformed_queue_does_not_stop_other_workers(self):
        self.write(self.root, 'coordination/queues/bad-worker.json', {'schema_version': 1, 'agent_id': 'bad-worker', 'steps': None})
        self.commit(self.root)
        self.run_git(self.root, 'push', 'origin', 'HEAD:main')
        result = loop.scan_once(self.root, self.args)
        self.assertEqual(result['prepared'], ['TASK'])
        bad = next(item for item in result['skipped'] if item['queue'].endswith('bad-worker.json'))
        self.assertTrue(bad['needs_commander_review'])
        self.assertIn('invalid queue', bad['reason'])

    def test_paused_worker_cannot_be_redispatched(self):
        self.write(self.root, 'coordination/control.json', {'schema_version': 1, 'state': 'running', 'paused_agents': ['worker']})
        self.commit(self.root)
        self.run_git(self.root, 'push', 'origin', 'HEAD:main')
        result = loop.scan_once(self.root, self.args)
        self.assertFalse(result['prepared'])
        self.assertIn('paused', result['skipped'][0]['reason'])

    def test_verified_milestone_continues_dispatch(self):
        self.write(self.root, 'coordination/control.json', {'schema_version': 1, 'state': 'verified_complete'})
        self.commit(self.root)
        self.run_git(self.root, 'push', 'origin', 'HEAD:main')
        result = loop.scan_once(self.root, self.args)
        self.assertEqual(result['prepared'], ['TASK'])
        self.assertFalse(result.get('stopped', False))

    def test_queue_exhaustion_is_not_project_completion(self):
        self.queue['next_index'] = 1
        self.update_main()
        result = loop.scan_once(self.root, self.args)
        self.assertFalse(result['prepared'])
        self.assertIn('Commander must add', result['skipped'][0]['reason'])
        self.assertNotIn('complete', result)


if __name__ == '__main__':
    unittest.main()
