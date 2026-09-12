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
    setUp = fixture.CoordinationTests.setUp
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
        self.cmd(cwd, 'commit', '-m', 'synthetic merge\n\nImplementation-Agent: test-double\nCommit-Executor: Cursor CLI')
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
