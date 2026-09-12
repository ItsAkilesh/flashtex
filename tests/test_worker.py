"""Worker loop checks use temporary Git repositories and never call a real model."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

SCRIPTS = Path(__file__).resolve().parents[1] / 'scripts'
spec = importlib.util.spec_from_file_location('worker_coord', SCRIPTS / 'coord.py')
coord = importlib.util.module_from_spec(spec)
spec.loader.exec_module(coord)
spec = importlib.util.spec_from_file_location('worker_under_test', SCRIPTS / 'worker.py')
worker = importlib.util.module_from_spec(spec)
with patch.dict(sys.modules, {'coord': coord}):
    spec.loader.exec_module(worker)


class WorkerTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        coord.git(self.root, 'init', '--initial-branch=main')
        coord.git(self.root, 'config', 'user.name', 'Synthetic Test')
        coord.git(self.root, 'config', 'user.email', 'fixture@example.invalid')
        coord.git(self.root, 'config', 'commit.gpgsign', 'false')
        (self.root / 'README').write_text('baseline\n')
        self.assignment = dict(schema_version=1, task_id='FT-TEST', revision=1, agent_id='tester',
                               branch='agent/tester/task', state='assigned',
                               timebox_minutes=10, allocation_id='synthetic',
                               owned_paths=['src'], acceptance=['tests'], objective='fixture')
        self.publish_assignment()
        coord.git(self.root, 'switch', '-c', self.assignment['branch'])
        self.args = SimpleNamespace(id='tester', machine='fixture', capability=['rust'],
                                    max_cycles=8, cycle_seconds=60)
        self.path, self.state = worker.read_state(self.root)
        self.result = dict(state='ready_for_integration', summary='Implemented fixture',
                           next_action='Integrate', eta_minutes=[0, 0, 0], tests=['fixture passed'],
                           reviewed_peers=[], adaptation='No changed peer interfaces')
        self.execute = self.enterContext(patch.object(worker, 'execute', side_effect=self.model))
        self.publish = self.enterContext(patch.object(coord, 'publish', side_effect=self.commit_checkpoint))

    def publish_assignment(self):
        coord.write_json(self.root / 'coordination/assignments/FT-TEST.json', self.assignment)
        coord.git(self.root, 'add', '.')
        coord.git(self.root, 'commit', '-m', 'synthetic assignment')
        coord.git(self.root, 'update-ref', 'refs/remotes/origin/main', 'HEAD')

    def fleet(self):
        return {'assignments': [{'assignment': self.assignment}]}

    def run_cycle(self, fleet=None):
        return worker.cycle(self.root, self.args, self.fleet() if fleet is None else fleet, self.path, self.state)

    def model(self, command, root, prompt, seconds, log):
        saved = json.loads(self.path.read_text())
        self.assertEqual(saved['phase'], 'model_running')
        self.assertEqual(saved['cycles'], 1 if self.execute.call_count == 1 else self.execute.call_count)
        self.assertIn('approval_policy="never"', command)
        self.assertIn('workspace-write', command)
        self.assertLessEqual(seconds, self.args.cycle_seconds)
        path = root / 'src/output.txt'
        path.parent.mkdir(exist_ok=True)
        path.write_text('useful work\n')
        output = Path(command[command.index('--output-last-message') + 1])
        output.write_text(json.dumps(self.result))
        return 0

    def commit_checkpoint(self, root, args):
        self.assertEqual(json.loads(self.path.read_text())['phase'], 'publication_started')
        coord.git(root, 'commit', '-m', 'synthetic checkpoint only; no real Cursor call')

    def test_no_assignment_does_not_spend(self):
        self.assertEqual(self.run_cycle({'assignments': []}), 'waiting_for_assignment')
        self.execute.assert_not_called()
        self.publish.assert_not_called()
        self.assertFalse(self.path.exists())

    def test_duplicate_active_assignments_refuse_before_spending(self):
        fleet = self.fleet()
        fleet['assignments'].append({'assignment': dict(self.assignment, task_id='FT-OTHER')})
        with self.assertRaisesRegex(ValueError, 'multiple active'):
            self.run_cycle(fleet)
        self.execute.assert_not_called()

    def test_dirty_work_refuses_and_preserves_contents(self):
        (self.root / 'README').write_text('unfinished human work\n')
        with self.assertRaisesRegex(ValueError, 'dirty'):
            self.run_cycle()
        self.assertEqual((self.root / 'README').read_text(), 'unfinished human work\n')
        self.execute.assert_not_called()

    def test_success_reports_ack_and_publishes_once(self):
        self.assertEqual(self.run_cycle(), 'ready_for_integration')
        record = json.loads((self.root / 'coordination/agents/tester.json').read_text())
        self.assertEqual(record['assignment_acknowledgements']['FT-TEST']['revision'], 1)
        self.assertEqual(record['supervisor']['tests'], ['fixture passed'])
        self.assertEqual(record['state'], 'ready_for_integration')
        self.assertEqual(coord.git(self.root, 'status', '--porcelain'), '')
        self.assertEqual(self.state['completed_assignment'], 'FT-TEST:1')
        self.assertEqual(self.state['phase'], 'idle')
        self.assertEqual(self.run_cycle(), 'waiting_for_next_assignment')
        self.execute.assert_called_once()
        self.publish.assert_called_once()

    def test_newer_revision_resumes_after_completion(self):
        self.run_cycle()
        self.assignment['revision'] = 2
        self.publish_assignment()
        self.assertEqual(self.run_cycle(), 'ready_for_integration')
        self.assertEqual(self.execute.call_count, 2)
        self.assertEqual(self.state['completed_assignment'], 'FT-TEST:2')
        record = json.loads((self.root / 'coordination/agents/tester.json').read_text())
        self.assertEqual(record['assignment_acknowledgements']['FT-TEST']['revision'], 2)

    def test_out_of_scope_write_is_preserved_and_not_published(self):
        def bad_model(*args):
            self.model(*args)
            (self.root / 'README').write_text('unauthorized change\n')
            return 0
        self.execute.side_effect = bad_model
        with self.assertRaisesRegex(ValueError, 'unexpected changed paths'):
            self.run_cycle()
        self.assertEqual((self.root / 'README').read_text(), 'unauthorized change\n')
        self.publish.assert_not_called()
        with self.assertRaisesRegex(ValueError, 'reconciliation'):
            self.run_cycle()
        self.execute.assert_called_once()

    def test_model_failure_is_durable_and_not_retried(self):
        self.execute.side_effect = None
        self.execute.return_value = 7
        with self.assertRaises(RuntimeError):
            self.run_cycle()
        self.assertEqual(json.loads(self.path.read_text())['phase'], 'model_failed')
        with self.assertRaisesRegex(ValueError, 'reconciliation'):
            self.run_cycle()
        self.execute.assert_called_once()
        self.publish.assert_not_called()

    def test_model_timeout_preserves_running_marker_without_retry(self):
        self.execute.side_effect = subprocess.TimeoutExpired('synthetic-model', 60)
        with self.assertRaises(subprocess.TimeoutExpired):
            self.run_cycle()
        self.assertEqual(json.loads(self.path.read_text())['phase'], 'model_running')
        with self.assertRaisesRegex(ValueError, 'reconciliation'):
            self.run_cycle()
        self.execute.assert_called_once()
        self.publish.assert_not_called()

    def test_publication_failure_preserves_staged_work_without_retry(self):
        self.publish.side_effect = subprocess.TimeoutExpired('synthetic-cursor', 180)
        with self.assertRaises(subprocess.TimeoutExpired):
            self.run_cycle()
        self.assertEqual(json.loads(self.path.read_text())['phase'], 'publication_failed')
        self.assertIn('src/output.txt', coord.git(self.root, 'diff', '--cached', '--name-only'))
        with self.assertRaisesRegex(ValueError, 'reconciliation'):
            self.run_cycle()
        self.execute.assert_called_once()
        self.publish.assert_called_once()

    def test_blocked_result_is_published_then_waits_for_new_assignment(self):
        self.result['state'] = 'blocked'
        self.result['summary'] = 'Tool unavailable'
        self.assertEqual(self.run_cycle(), 'blocked')
        self.assertEqual(self.run_cycle(), 'waiting_for_next_assignment')
        self.execute.assert_called_once()
        self.publish.assert_called_once()

    def test_model_commit_is_detected_even_when_change_is_hidden_from_diff(self):
        def bad_model(*args):
            self.model(*args)
            (self.root / 'README').write_text('hidden unauthorized change\n')
            coord.git(self.root, 'add', '.')
            coord.git(self.root, 'commit', '-m', 'synthetic unauthorized model commit')
            return 0
        self.execute.side_effect = bad_model
        with self.assertRaisesRegex(ValueError, 'HEAD|branch|Git'):
            self.run_cycle()
        self.publish.assert_not_called()
        self.assertEqual((self.root / 'README').read_text(), 'hidden unauthorized change\n')

    def test_model_branch_switch_is_detected(self):
        def bad_model(*args):
            self.model(*args)
            coord.git(self.root, 'switch', '-c', 'agent/tester/unauthorized')
            return 0
        self.execute.side_effect = bad_model
        with self.assertRaisesRegex(ValueError, 'HEAD|branch|Git'):
            self.run_cycle()
        self.publish.assert_not_called()

    def test_exhausted_task_can_resume_with_new_revision(self):
        self.state.update(assignment='FT-TEST:1', cycles=8, task_cycles=8,
                          task_started_utc=coord.stamp())
        self.assertEqual(self.run_cycle(), 'allocation_exhausted')
        self.assignment['revision'] = 2
        self.publish_assignment()
        self.execute.side_effect = lambda *args: 7
        with self.assertRaises(RuntimeError):
            self.run_cycle()
        self.execute.assert_called_once()
        self.assertEqual(self.state['task_cycles'], 1)
        self.assertEqual(self.state['cycles'], 9)

    def test_cycle_cap_prevents_further_spending(self):
        self.state.update(assignment='FT-TEST:1', cycles=self.args.max_cycles, task_cycles=self.args.max_cycles, task_started_utc=coord.stamp())
        self.assertEqual(self.run_cycle(), 'allocation_exhausted')
        self.execute.assert_not_called()


if __name__ == '__main__':
    unittest.main()
