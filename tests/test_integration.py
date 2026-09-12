"""Concurrent Git integration tests; Cursor calls are synthetic fixtures only."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import test_coordination as fixture

sys.modules['coord'] = fixture.coord
spec = importlib.util.spec_from_file_location('integrate', Path(__file__).resolve().parents[1] / 'scripts/integrate.py')
integrate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(integrate)
coord = integrate.coord


class IntegrationTests(unittest.TestCase):
    def setUp(self):
        fixture.CoordinationTests.setUp(self)
        mocked = patch.object(coord, 'current_git_user_trailer',
                              return_value='Co-authored-by: synthetic <1+synthetic@users.noreply.github.com>',
                              create=True)
        mocked.start()
        self.addCleanup(mocked.stop)
    cmd = fixture.CoordinationTests.cmd
    configure = fixture.CoordinationTests.configure
    commit = fixture.CoordinationTests.commit

    def prepare(self, conflict=False):
        self.cmd(self.a, 'switch', '-c', 'agent/worker/code')
        (self.a / ('README' if conflict else 'worker.txt')).write_text('worker\n')
        self.candidate = self.commit(self.a)
        self.cmd(self.a, 'push', '-u', 'origin', 'agent/worker/code')
        (self.b / ('README' if conflict else 'main.txt')).write_text('main\n')
        self.main = self.commit(self.b)
        self.cmd(self.b, 'push', 'origin', 'HEAD:main')
        self.cmd(self.b, 'switch', '-c', 'agent/commander/loop')
        self.work = self.base / 'integration'
        integrate.prepare(self.b, SimpleNamespace(ref='origin/agent/worker/code', sha=self.candidate,
                                                  name='fixture', worktree=str(self.work)))
        self.configure(self.work)
        self.calls = 0

    def args(self, code='pass'):
        return SimpleNamespace(allocation='synthetic-only', timeout=10, check_timeout=10,
                               check=[json.dumps([sys.executable, '-c', code])])

    def fake_cursor(self, argv, cwd=None, **kw):
        if argv[0] != 'cursor-agent':
            return self.real_run(argv, cwd=cwd, **kw)
        self.calls += 1
        if self.cmd(cwd, 'diff', '--name-only', '--diff-filter=U'):
            (cwd / 'README').write_text('main\nworker\n')
        self.cmd(cwd, 'add', '.')
        self.cmd(cwd, 'commit', '-m', 'synthetic merge\n\nImplementation-Agent: test-double\nCommit-Executor: Cursor CLI\nCo-authored-by: synthetic <1+synthetic@users.noreply.github.com>')
        return subprocess.CompletedProcess(argv, 0, 'synthetic', '')

    def run_finish(self, args=None, callback=None):
        self.real_run = coord.run
        with patch.object(coord, 'run', side_effect=callback or self.fake_cursor):
            integrate.finish(self.work, args or self.args())

    def test_clean_divergent_merge_preserves_inputs_and_main(self):
        self.prepare()
        self.run_finish()
        self.assertEqual(self.calls, 1)
        self.assertEqual((self.work / 'main.txt').read_text(), 'main\n')
        self.assertEqual((self.work / 'worker.txt').read_text(), 'worker\n')
        self.assertEqual(self.cmd(self.origin, 'rev-parse', 'main'), self.main)
        self.assertEqual(self.cmd(self.work, 'show', '-s', '--format=%P').split(), [self.main, self.candidate])

    def test_conflict_resolved_in_isolated_worktree(self):
        self.prepare(conflict=True)
        (self.b / 'README').write_text('unfinished local work\n')
        self.run_finish()
        self.assertEqual((self.work / 'README').read_text(), 'main\nworker\n')
        self.assertEqual((self.b / 'README').read_text(), 'unfinished local work\n')

    def test_failed_validation_can_resume_without_cursor_again(self):
        self.prepare()
        with self.assertRaisesRegex(RuntimeError, 'Validation failed'):
            self.run_finish(self.args('raise SystemExit(3)'))
        self.assertEqual(self.cmd(self.origin, 'branch', '--list', 'agent/commander/integrate-fixture'), '')
        self.run_finish()
        self.assertEqual(self.calls, 1)

    def test_timeout_after_commit_resumes_without_duplicate_model_call(self):
        self.prepare(conflict=True)
        def timeout(argv, **kw):
            result = self.fake_cursor(argv, **kw)
            if argv[0] == 'cursor-agent':
                raise subprocess.TimeoutExpired(argv, 10)
            return result
        with self.assertRaises(subprocess.TimeoutExpired):
            self.run_finish(callback=timeout)
        self.run_finish()
        self.assertEqual(self.calls, 1)

    def test_unresolved_timeout_preserves_work_no_duplicate_call(self):
        self.prepare(conflict=True)
        def timeout(argv, **kw):
            if argv[0] == 'cursor-agent':
                self.calls += 1
                raise subprocess.TimeoutExpired(argv, 10)
            return self.real_run(argv, **kw)
        with self.assertRaises(subprocess.TimeoutExpired):
            self.run_finish(callback=timeout)
        with self.assertRaisesRegex(ValueError, 'two parents'):
            self.run_finish()
        self.assertEqual(self.calls, 1)
        self.assertEqual(self.cmd(self.work, 'rev-parse', 'MERGE_HEAD'), self.candidate)

    def test_validator_mutation_never_publishes(self):
        self.prepare()
        with self.assertRaisesRegex(ValueError, 'dirty'):
            self.run_finish(self.args("from pathlib import Path; Path('README').write_text('changed')"))
        self.assertEqual(self.cmd(self.origin, 'branch', '--list', 'agent/commander/integrate-fixture'), '')

    def test_clean_path_tamper_rejected(self):
        self.prepare(conflict=True)
        def tamper(argv, **kw):
            if argv[0] == 'cursor-agent':
                (kw['cwd'] / 'unexpected.txt').write_text('unrelated\n')
            return self.fake_cursor(argv, **kw)
        with self.assertRaisesRegex(ValueError, 'outside the conflict set'):
            self.run_finish(callback=tamper)
        self.assertEqual(self.cmd(self.origin, 'branch', '--list', 'agent/commander/integrate-fixture'), '')

    def test_already_integrated_and_moved_candidate_refused(self):
        self.prepare()
        with self.assertRaisesRegex(ValueError, 'candidate moved'):
            integrate.prepare(self.b, SimpleNamespace(ref='origin/agent/worker/code', sha='0'*40,
                               name='other', worktree=str(self.base / 'other')))
        self.cmd(self.b, 'push', 'origin', self.main + ':refs/heads/agent/worker/done')
        with self.assertRaisesRegex(ValueError, 'already integrated'):
            integrate.prepare(self.b, SimpleNamespace(ref='origin/agent/worker/done', sha=self.main,
                               name='done', worktree=str(self.base / 'done')))

    def test_promotion_requires_current_main_and_exact_reviewed_commit(self):
        self.prepare()
        self.run_finish()
        head = self.cmd(self.work, 'rev-parse', 'HEAD')
        with self.assertRaisesRegex(ValueError, 'exact validated'):
            integrate.promote(self.work, SimpleNamespace(sha='0'*40))
        integrate.promote(self.work, SimpleNamespace(sha=head))
        self.assertEqual(self.cmd(self.origin, 'rev-parse', 'main'), head)

    def test_main_advance_blocks_promotion(self):
        self.prepare()
        self.run_finish()
        (self.b / 'new-main.txt').write_text('concurrent work')
        updated = self.commit(self.b)
        self.cmd(self.b, 'push', 'origin', 'HEAD:main')
        with self.assertRaisesRegex(ValueError, 'main advanced'):
            integrate.promote(self.work, SimpleNamespace(sha=self.cmd(self.work, 'rev-parse', 'HEAD')))
        self.assertEqual(self.cmd(self.origin, 'rev-parse', 'main'), updated)


class WorkerSyncTests(unittest.TestCase):
    setUp = IntegrationTests.setUp
    cmd = IntegrationTests.cmd
    configure = IntegrationTests.configure
    commit = IntegrationTests.commit
    args = IntegrationTests.args
    fake_cursor = IntegrationTests.fake_cursor
    def worker(self, divergent=False, conflict=False):
        self.cmd(self.a, 'switch', '-c', 'agent/worker/code')
        if divergent:
            (self.a / ('README' if conflict else 'worker.txt')).write_text('worker\n')
            self.commit(self.a)
        self.base_sha = self.cmd(self.a, 'rev-parse', 'HEAD')
        self.cmd(self.a, 'push', '-u', 'origin', 'agent/worker/code')
        (self.b / ('README' if conflict else 'main.txt')).write_text('main\n')
        self.main = self.commit(self.b)
        self.cmd(self.b, 'push', 'origin', 'HEAD:main')
        self.work = self.a
        self.calls = 0

    def sync(self, args=None, callback=None):
        args = args or self.args()
        args.id = 'worker'
        self.real_run = coord.run
        with patch.object(coord, 'run', side_effect=callback or self.fake_cursor):
            integrate.sync(self.work, args)

    def test_sync_fast_forward_and_repeated_noop(self):
        self.worker()
        self.sync()
        self.sync()
        self.assertEqual(self.calls, 0)
        self.assertEqual(self.cmd(self.a, 'rev-parse', 'HEAD'), self.main)
        self.assertEqual(self.cmd(self.origin, 'rev-parse', 'agent/worker/code'), self.main)
        self.assertFalse((coord.local_state(self.a) / 'integration.json').exists())

    def test_sync_divergence_preserves_main_and_publishes_worker(self):
        self.worker(divergent=True)
        self.sync()
        head = self.cmd(self.a, 'rev-parse', 'HEAD')
        self.assertEqual(self.cmd(self.a, 'show', '-s', '--format=%P').split(), [self.base_sha, self.main])
        self.assertEqual(self.cmd(self.origin, 'rev-parse', 'agent/worker/code'), head)
        self.assertEqual(self.cmd(self.origin, 'rev-parse', 'main'), self.main)
        self.sync()
        self.assertEqual(self.calls, 1)
        packet = json.loads((coord.local_state(self.a) / 'integration.json').read_text())
        self.assertEqual(packet['mode'], 'worker_sync')
        with self.assertRaises(ValueError):
            integrate.promote(self.a, SimpleNamespace(sha=head))
        self.assertEqual(self.cmd(self.origin, 'rev-parse', 'main'), self.main)

    def test_sync_conflict_resolves_and_preserves_clean_paths(self):
        self.worker(divergent=True, conflict=True)
        self.sync()
        self.assertEqual((self.a / 'README').read_text(), 'main\nworker\n')
        self.assertEqual(self.calls, 1)

    def test_sync_dirty_work_refused_without_merge(self):
        self.worker(divergent=True)
        (self.a / 'README').write_text('unfinished work')
        with self.assertRaisesRegex(ValueError, 'dirty'):
            self.sync()
        self.assertEqual((self.a / 'README').read_text(), 'unfinished work')
        self.assertEqual(self.cmd(self.a, 'rev-parse', 'HEAD'), self.base_sha)
        self.assertEqual(self.calls, 0)

    def test_sync_timeout_resumes_without_paid_retry(self):
        self.worker(divergent=True, conflict=True)
        def timeout(argv, **kw):
            result = self.fake_cursor(argv, **kw)
            if argv[0] == 'cursor-agent':
                raise subprocess.TimeoutExpired(argv, 10)
            return result
        with self.assertRaises(subprocess.TimeoutExpired):
            self.sync(callback=timeout)
        self.sync()
        self.assertEqual(self.calls, 1)

    def test_sync_unresolved_timeout_does_not_reinvoke(self):
        self.worker(divergent=True, conflict=True)
        def timeout(argv, **kw):
            if argv[0] == 'cursor-agent':
                self.calls += 1
                raise subprocess.TimeoutExpired(argv, 10)
            return self.real_run(argv, **kw)
        with self.assertRaises(subprocess.TimeoutExpired):
            self.sync(callback=timeout)
        with self.assertRaisesRegex(ValueError, 'two parents'):
            self.sync()
        self.assertEqual(self.calls, 1)
        self.assertEqual(self.cmd(self.a, 'rev-parse', 'MERGE_HEAD'), self.main)

    def test_sync_requires_validation_and_allocation_before_merge(self):
        self.worker(divergent=True)
        args = self.args()
        args.check = []
        with self.assertRaisesRegex(ValueError, '--check'):
            self.sync(args)
        args = self.args()
        args.allocation = None
        with self.assertRaisesRegex(ValueError, 'allocation'):
            self.sync(args)
        self.assertEqual(self.cmd(self.a, 'rev-parse', 'HEAD'), self.base_sha)
        self.assertFalse(coord.git(self.a, 'rev-parse', '--verify', 'MERGE_HEAD', check=False))

    def test_sync_owner_mismatch_and_commander_packet_refused(self):
        self.worker(divergent=True)
        args = self.args()
        args.id = 'other'
        with self.assertRaises(ValueError):
            integrate.sync(self.a, args)
        coord.write_json(coord.local_state(self.a) / 'integration.json',
                         {'mode': 'commander_integration', 'state': 'prepared', 'branch': 'agent/commander/x'})
        with self.assertRaisesRegex(ValueError, 'another prepared integration'):
            self.sync()

    def test_sync_conflict_path_tamper_never_publishes(self):
        self.worker(divergent=True, conflict=True)
        def tamper(argv, **kw):
            if argv[0] == 'cursor-agent':
                (kw['cwd'] / 'unexpected.txt').write_text('unrelated\n')
            return self.fake_cursor(argv, **kw)
        with self.assertRaisesRegex(ValueError, 'outside the conflict set'):
            self.sync(callback=tamper)
        self.assertEqual(self.cmd(self.origin, 'rev-parse', 'agent/worker/code'), self.base_sha)

    def test_sync_missing_coauthor_never_publishes(self):
        self.worker(divergent=True)
        def missing(argv, **kw):
            result = self.fake_cursor(argv, **kw)
            if argv[0] == 'cursor-agent':
                self.cmd(kw['cwd'], 'commit', '--amend', '-m',
                         'synthetic\n\nImplementation-Agent: test-double\nCommit-Executor: Cursor CLI')
            return result
        with self.assertRaisesRegex(ValueError, 'coauthor'):
            self.sync(callback=missing)
        self.assertEqual(self.cmd(self.origin, 'rev-parse', 'agent/worker/code'), self.base_sha)

    def test_sync_fast_forward_failed_push_resumes_without_spending(self):
        self.worker()
        def fail_push(argv, **kw):
            if argv[:2] == ['git', 'push']:
                raise RuntimeError('network unavailable')
            return self.fake_cursor(argv, **kw)
        with self.assertRaisesRegex(RuntimeError, 'network unavailable'):
            self.sync(callback=fail_push)
        self.assertEqual(self.cmd(self.a, 'rev-parse', 'HEAD'), self.main)
        self.assertEqual(self.cmd(self.origin, 'rev-parse', 'agent/worker/code'), self.base_sha)
        self.sync()
        self.assertEqual(self.cmd(self.origin, 'rev-parse', 'agent/worker/code'), self.main)
        self.assertEqual(self.calls, 0)
        self.assertFalse((coord.local_state(self.a) / 'worker-sync-publication.json').exists())

    def test_sync_fast_forward_resume_preserves_new_dirty_work(self):
        self.worker()
        def fail_push(argv, **kw):
            if argv[:2] == ['git', 'push']:
                raise RuntimeError('network unavailable')
            return self.fake_cursor(argv, **kw)
        with self.assertRaises(RuntimeError):
            self.sync(callback=fail_push)
        (self.a / 'README').write_text('new unfinished work')
        with self.assertRaisesRegex(ValueError, 'dirty'):
            self.sync()
        self.assertEqual((self.a / 'README').read_text(), 'new unfinished work')
        self.assertEqual(self.cmd(self.origin, 'rev-parse', 'agent/worker/code'), self.base_sha)
