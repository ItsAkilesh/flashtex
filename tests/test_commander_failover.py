import copy
import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location('failover', Path(__file__).resolve().parents[1] / 'scripts/commander_failover.py')
f = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(f)


class FailoverTests(unittest.TestCase):
    def setUp(self):
        self.pin = {'pid': 42, 'start_ticks': 100, 'boot_id': 'boot'}
        self.config = {'process': self.pin, 'predecessor_id': 'astra', 'successor_id': 'jaysen'}
        self.authority = {'commander_id': 'astra', 'authority_state': 'active'}
        self.services = [{'active': 'inactive', 'pid': 0}]

    def test_live_quota_or_idle_process_is_never_terminal(self):
        for state in ('S', 'R', 'D', 'T'):
            observed = dict(self.pin, state=state)
            result = f.evaluate(self.config, self.authority, observed, self.services, [], [])
            self.assertEqual(result['state'], 'blocked')
            self.assertFalse(result['claim_authorized'])

    def test_missing_recycled_or_dead_process_is_observed_only(self):
        for observed in (None, dict(self.pin, start_ticks=101, state='R'), dict(self.pin, boot_id='other', state='R'), dict(self.pin, state='Z')):
            result = f.evaluate(self.config, self.authority, observed, self.services, [], [])
            self.assertEqual(result['state'], 'terminal_quiescent_observed')
            self.assertFalse(result['claim_authorized'])

    def test_each_publication_uncertainty_blocks(self):
        cases = [([], [], []), ([{'active': 'active', 'pid': 0}], [], []),
                 ([{'active': 'inactive', 'pid': 7}], [], []),
                 (self.services, ['pending'], []), (self.services, [], [self.pin])]
        for services, journals, processes in cases:
            self.assertEqual(f.evaluate(self.config, self.authority, None, services, journals, processes)['state'], 'blocked')

    def test_changed_authority_blocks_stale_standby(self):
        for authority in ({'commander_id': 'other', 'authority_state': 'active'}, {'commander_id': 'astra', 'authority_state': 'quiesced'}):
            self.assertEqual(f.evaluate(self.config, authority, None, self.services, [], [])['state'], 'blocked')

    def test_malformed_identity_cannot_be_offline_proof(self):
        for key, value in [('pid', True), ('start_ticks', 0), ('boot_id', '')]:
            config = copy.deepcopy(self.config)
            config['process'][key] = value
            with self.assertRaises(ValueError):
                f.evaluate(config, self.authority, None, self.services, [], [])

    def test_dispatcher_stop_requires_terminal_identity_and_current_authority(self):
        config = dict(self.config, publisher_services=['flashtex-dispatch.service'],
                      allow_dispatcher_stop_after_process_exit=True)
        calls = []
        self.assertFalse(f.quiesce_after_terminal(config, self.authority, dict(self.pin, state='S'), calls.append))
        self.assertEqual(calls, [])
        self.assertFalse(f.quiesce_after_terminal(config, {'commander_id': 'other'}, None, calls.append))
        self.assertEqual(calls, [])
        self.assertTrue(f.quiesce_after_terminal(config, self.authority, None, calls.append))
        self.assertEqual(calls, ['flashtex-dispatch.service'])

    def test_unreviewed_service_cannot_be_stopped(self):
        config = dict(self.config, publisher_services=['unrelated.service'],
                      allow_dispatcher_stop_after_process_exit=True)
        with self.assertRaises(ValueError):
            f.quiesce_after_terminal(config, self.authority, None, lambda _: self.fail('must not stop'))

    def test_active_witness_never_attempts_publication(self):
        self.assertIsNone(f.publish_terminal_receipt({}, {'state': 'blocked'}, Path('/missing'), Path('/missing/journal')))

    def test_pending_receipt_never_replays_an_uncertain_push(self):
        import tempfile
        import json
        with tempfile.TemporaryDirectory() as folder:
            journal = Path(folder) / 'pending.json'
            journal.write_text(json.dumps({'state': 'pending'}))
            with self.assertRaisesRegex(RuntimeError, 'reconcile'):
                f.publish_terminal_receipt({}, {'state': 'terminal_quiescent_observed'}, Path(folder), journal)

    def test_published_receipt_is_idempotent_without_git(self):
        import tempfile
        import json
        with tempfile.TemporaryDirectory() as folder:
            journal = Path(folder) / 'done.json'
            journal.write_text(json.dumps({'state': 'published', 'commit': 'abc'}))
            self.assertEqual(f.publish_terminal_receipt({}, {'state': 'terminal_quiescent_observed'}, Path(folder), journal), 'abc')

    def test_terminal_receipt_pushes_only_its_dedicated_branch(self):
        import tempfile
        import json
        import subprocess
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            remote, repo = root / 'remote.git', root / 'repo'
            def git(*args, cwd=root):
                return subprocess.check_output(['git', *args], cwd=cwd, text=True, stderr=subprocess.PIPE).strip()
            git('init', '--bare', str(remote))
            git('clone', str(remote), str(repo))
            git('config', 'user.name', 'Fixture', cwd=repo)
            git('config', 'user.email', 'fixture@example.invalid', cwd=repo)
            (repo / 'coordination').mkdir()
            (repo / 'coordination/authority.json').write_text(json.dumps(self.authority))
            git('add', '.', cwd=repo)
            git('commit', '-m', 'fixture', cwd=repo)
            git('push', 'origin', 'HEAD:main', cwd=repo)
            git('fetch', 'origin', cwd=repo)
            base = git('rev-parse', 'origin/main', cwd=repo)
            config = dict(self.config, witness_branch='agent/orchestrator-witness/test',
                          witness_coauthor='Fixture <fixture@example.invalid>')
            result = f.evaluate(config, self.authority, None, self.services, [], [])
            result['observed_main_sha'] = base
            journal = root / 'publication.json'
            commit = f.publish_terminal_receipt(config, result, repo, journal)
            self.assertEqual(git('rev-parse', 'refs/heads/main', cwd=remote), base)
            self.assertEqual(git('rev-parse', 'refs/heads/agent/orchestrator-witness/test', cwd=remote), commit)
            self.assertEqual(git('diff-tree', '--no-commit-id', '--name-only', '-r', commit, cwd=repo),
                             'coordination/failover/witness-linux.json')
            self.assertEqual(f.publish_terminal_receipt(config, result, repo, journal), commit)
            worktree = json.loads(journal.read_text())['worktree']
            git('worktree', 'remove', worktree, cwd=repo)

    def test_malformed_pin_cannot_stop_even_an_authorized_service(self):
        config = dict(self.config, process={'pid': True, 'start_ticks': 100, 'boot_id': 'boot'},
                      publisher_services=['flashtex-dispatch.service'],
                      allow_dispatcher_stop_after_process_exit=True)
        with self.assertRaises(ValueError):
            f.quiesce_after_terminal(config, self.authority, None, lambda _: self.fail('must not stop'))

    def test_service_list_is_validated_before_any_stop(self):
        config = dict(self.config, publisher_services=['flashtex-dispatch.service', 'unrelated.service'],
                      allow_dispatcher_stop_after_process_exit=True)
        with self.assertRaises(ValueError):
            f.quiesce_after_terminal(config, self.authority, None, lambda _: self.fail('must not stop'))

    def test_shell_wrapped_future_publication_remains_a_blocker(self):
        for args in [['bash', '-c', 'cargo test; git push origin HEAD:main'],
                     ['python3', 'scripts/coord.py', 'publish'],
                     ['/usr/bin/git', 'merge', 'origin/main']]:
            self.assertTrue(f.looks_like_publisher(args))
        self.assertFalse(f.looks_like_publisher(['python3', 'scripts/commander_failover.py', '--watch']))
