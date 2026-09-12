#!/usr/bin/env python3
"""Tests for the fail-closed standby revival gate. Pure synthetic records plus one
isolated bare-remote fixture. No network, no model calls, no writes to the real repo.

Run: python3 -m unittest tools/commander-standby/test_standby_gate.py
"""
import copy
from datetime import datetime, timedelta, timezone
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import standby_gate as gate  # noqa: E402

MAIN = 'a' * 40
BOOT = 'f1d9f6a2-ccc1-4c85-bfcb-e629905e9820'
PIN = {'pid': 268514, 'start_ticks': 30612338, 'boot_id': BOOT}
BRANCH = 'agent/orchestrator-witness/astra-30612338'
NOW = datetime(2026, 9, 12, 8, 0, tzinfo=timezone.utc)


def config():
    return {'schema_version': 1, 'predecessor_id': 'orchestrator-astra', 'successor_id': gate.SUCCESSOR,
            'successor_machine': 'mac-m1max-a', 'process': dict(PIN), 'publisher_services': ['flashtex-dispatch.service'],
            'mode': 'read_only_standby', 'witness_branch': BRANCH}


def authority(commander='orchestrator-astra', state='active'):
    return {'commander_id': commander, 'authority_state': state, 'claim_base_main': '9' * 40}


def receipt(state='terminal_quiescent_observed', reasons=(), services=None, processes=(), journals=(),
            observed_main=MAIN, pin=None, claim_authorized=False):
    if services is None:
        services = [{'unit': 'flashtex-dispatch.service', 'active': 'inactive', 'pid': 0}]
    return {'schema_version': 1, 'state': state, 'predecessor_id': 'orchestrator-astra',
            'successor_id': gate.SUCCESSOR, 'process_pin': dict(pin or PIN), 'reasons': list(reasons),
            'publisher_processes': list(processes), 'journal_blockers': list(journals), 'services': services,
            'observed_utc': (NOW - timedelta(minutes=2)).isoformat(), 'claim_authorized': claim_authorized,
            'observed_main_sha': observed_main}


def commit(parent=MAIN):
    return {'sha': 'b' * 40, 'parents': [parent], 'author': gate.WITNESS_AUTHOR,
            'message': 'Observe terminal Commander and stopped publication jobs\n\n'
                       'Implementation-Agent: Codex Astra\n' + gate.WITNESS_EXECUTOR + '\n'}


def run(**overrides):
    args = {'config': config(), 'authority': authority(), 'main_sha': MAIN, 'witness_refs': {BRANCH: 'b' * 40},
            'receipt': receipt(), 'receipt_commit': commit(), 'local_journal': None, 'now': NOW}
    args.update(overrides)
    return gate.evaluate(**args)


class GateDecisions(unittest.TestCase):
    def assertNoAction(self, verdict, reason):
        self.assertEqual(verdict['verdict'], 'no_action')
        self.assertFalse(verdict['revival_permitted'])
        self.assertFalse(verdict['executed'])
        self.assertTrue(any(r.startswith(reason) for r in verdict['reasons']), verdict['reasons'])

    def test_absent_witness_ref_is_not_terminal(self):
        self.assertNoAction(run(witness_refs={}, receipt=None, receipt_commit=None), 'no_terminal_receipt_published')

    def test_alive_commander_blocked_receipt(self):
        alive = receipt(state='blocked', reasons=['exact_commander_process_still_exists'],
                        services=[{'unit': 'flashtex-dispatch.service', 'active': 'active', 'pid': 687692}])
        v = run(receipt=alive)
        self.assertNoAction(v, 'receipt_state_not_terminal')
        self.assertNoAction(v, 'receipt_lists_blockers')

    def test_capacity_error_text_is_not_terminal(self):
        # Issue #22-style quota/capacity wording while the process lives never counts.
        capacity = receipt(state='blocked', reasons=['exact_commander_process_still_exists'])
        capacity['error'] = 'Hosted font/render slots temporarily blocked by model capacity'
        capacity['note'] = 'turn.failed: usage limit reached'
        self.assertNoAction(run(receipt=capacity), 'receipt_state_not_terminal')
        # Even a receipt that claims terminal but carries only quota text and live services is blocked.
        quota_only = receipt(services=[{'unit': 'flashtex-dispatch.service', 'active': 'active', 'pid': 687692}])
        quota_only['error'] = 'ActionRequiredError: usage limit'
        self.assertNoAction(run(receipt=quota_only), 'publisher_services_not_verified_stopped')

    def test_exact_terminal_but_jobs_running_blocked(self):
        v = run(receipt=receipt(processes=[{'pid': 700001, 'start_ticks': 1, 'boot_id': BOOT, 'state': 'S'}]))
        self.assertNoAction(v, 'publication_processes_remain')
        v = run(receipt=receipt(journals=['/x/flashtex/dispatcher-publication.json:unresolved']))
        self.assertNoAction(v, 'unresolved_publication_journals')
        v = run(receipt=receipt(services=[{'unit': 'flashtex-dispatch.service', 'active': 'activating', 'pid': 0}]))
        self.assertNoAction(v, 'publisher_services_not_verified_stopped')
        v = run(receipt=receipt(services=[]))
        self.assertNoAction(v, 'publisher_services_not_verified_stopped')

    def test_exact_terminal_and_jobs_stopped_permitted_flag_only(self):
        v = run()
        self.assertEqual(v['verdict'], 'revival_permitted')
        self.assertTrue(v['revival_permitted'])
        self.assertFalse(v['executed'])
        self.assertEqual(v['reasons'], [])
        self.assertIn('never an executed claim', v['limitations'])

    def test_stale_or_recycled_process_pin(self):
        recycled = receipt(pin={'pid': 268514, 'start_ticks': 30999999, 'boot_id': BOOT})
        self.assertNoAction(run(receipt=recycled), 'receipt_pin_does_not_match_current_config')
        rebooted = receipt(pin={'pid': 268514, 'start_ticks': 30612338, 'boot_id': 'other-boot'})
        self.assertNoAction(run(receipt=rebooted), 'receipt_pin_does_not_match_current_config')
        stale_cfg = config(); stale_cfg['process'] = {'pid': 195402, 'start_ticks': 1, 'boot_id': BOOT}
        self.assertNoAction(run(config=stale_cfg), 'receipt_pin_does_not_match_current_config')

    def test_main_moved_since_observation(self):
        self.assertNoAction(run(main_sha='c' * 40), 'main_moved_since_terminal_observation')
        self.assertNoAction(run(receipt_commit=commit(parent='c' * 40)), 'receipt_commit_not_on_observed_main')

    def test_competing_or_newer_claim(self):
        self.assertNoAction(run(authority=authority('orchestrator-someone-else')), 'competing_or_newer_claim_on_main')
        self.assertNoAction(run(authority=authority(gate.SUCCESSOR)), 'already_claimed_by_successor')
        self.assertNoAction(run(authority=authority(state='quiesced')), 'authority_state_not_active')
        self.assertNoAction(run(authority=None), 'authority_unreadable')

    def test_receipt_that_asserts_authorization_is_rejected(self):
        self.assertNoAction(run(receipt=receipt(claim_authorized=True)), 'receipt_asserts_claim_authorization')

    def test_receipt_commit_provenance(self):
        bad = commit(); bad['author'] = 'Codex Astra <astra@flashtex.invalid>'
        self.assertNoAction(run(receipt_commit=bad), 'receipt_commit_author_not_witness')
        bad = commit(); bad['message'] = 'Observe\n\nCommit-Executor: someone\n'
        self.assertNoAction(run(receipt_commit=bad), 'receipt_commit_missing_witness_executor_trailer')
        self.assertNoAction(run(receipt_commit=None), 'receipt_commit_unreadable')

    def test_future_dated_or_unreadable_receipt(self):
        future = receipt(); future['observed_utc'] = (NOW + timedelta(hours=1)).isoformat()
        self.assertNoAction(run(receipt=future), 'receipt_future_dated')
        self.assertNoAction(run(receipt='unreadable'), 'receipt_unreadable')
        self.assertNoAction(run(config=None), 'failover_config_absent')

    def test_pending_or_lost_push_journal_blocks(self):
        for state in ('pending', 'pushed', 'prepared_local', 'unknown'):
            self.assertNoAction(run(local_journal={'state': state}), 'unreconciled_local_claim_journal')
        self.assertNoAction(run(local_journal='unreadable'), 'local_claim_journal_unreadable')

    def test_config_must_name_this_successor_in_read_only_mode(self):
        c = config(); c['successor_id'] = 'orchestrator-other'
        self.assertNoAction(run(config=c), 'config_names_different_successor')
        c = config(); c['mode'] = 'active'
        self.assertNoAction(run(config=c), 'config_mode_not_read_only_standby')

    def test_prepared_claim_is_data_only(self):
        claim = gate.prepared_claim(config(), authority(), MAIN, 'b' * 40, NOW)
        self.assertEqual(claim['commander_id'], gate.SUCCESSOR)
        self.assertEqual(claim['claim_base_main'], MAIN)
        self.assertEqual(claim['predecessor'], 'orchestrator-astra')
        self.assertIn('b' * 40, claim['handoff_evidence'])


def git(cwd, *args, name='t', email='t@x'):
    return subprocess.run(['git', *args], cwd=cwd, capture_output=True, text=True, check=True,
                          env=dict(os.environ, GIT_AUTHOR_NAME=name, GIT_AUTHOR_EMAIL=email, GIT_COMMITTER_NAME=name,
                                   GIT_COMMITTER_EMAIL=email, GIT_CONFIG_GLOBAL='/dev/null')).stdout.strip()


class BareRemoteFixture(unittest.TestCase):
    """collect() reads the witness branch; prepare never pushes; remote main stays byte-identical."""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix='standby-gate-'))
        self.remote = self.tmp / 'remote.git'
        git(self.tmp, 'init', '--bare', '-q', '-b', 'main', str(self.remote))
        seed = self.tmp / 'seed'
        git(self.tmp, 'clone', '-q', str(self.remote), str(seed))
        (seed / 'coordination').mkdir()
        (seed / 'coordination/authority.json').write_text(json.dumps(authority(), indent=2) + '\n')
        (seed / 'coordination/failover.json').write_text(json.dumps(config(), indent=2) + '\n')
        git(seed, 'add', '.'); git(seed, 'commit', '-q', '-m', 'seed'); git(seed, 'push', '-q', 'origin', 'HEAD:main')
        self.main = git(seed, 'rev-parse', 'HEAD')
        self.seed = seed
        self.clone = self.tmp / 'standby'
        git(self.tmp, 'clone', '-q', str(self.remote), str(self.clone))

    def publish_receipt(self, rec):
        wt = self.tmp / 'witness'
        git(self.seed, 'worktree', 'add', '-q', '--detach', str(wt), self.main)
        (wt / 'coordination/failover').mkdir(parents=True)
        (wt / gate.RECEIPT_PATH).write_text(json.dumps(rec, indent=2) + '\n')
        git(wt, 'add', '--', gate.RECEIPT_PATH)
        git(wt, 'commit', '-q', '-m', 'Observe terminal Commander and stopped publication jobs\n\n'
            'Implementation-Agent: Codex Astra\n' + gate.WITNESS_EXECUTOR + '\n',
            name='FlashTeX Witness', email='witness@flashtex.invalid')
        git(wt, 'push', '-q', 'origin', 'HEAD:refs/heads/' + BRANCH)
        return git(wt, 'rev-parse', 'HEAD')

    def test_absent_ref_then_alive_then_terminal(self):
        ev = gate.collect(self.clone)
        self.assertEqual(ev['witness_refs'], {})
        self.assertNoActionLike(gate.evaluate(ev['config'], ev['authority'], ev['main_sha'], ev['witness_refs'],
                                              ev['receipt'], ev['receipt_commit'], None, NOW), 'no_terminal_receipt_published')
        alive = receipt(state='blocked', reasons=['exact_commander_process_still_exists'], observed_main=self.main)
        self.publish_receipt(alive)
        ev = gate.collect(self.clone)
        self.assertNoActionLike(gate.evaluate(ev['config'], ev['authority'], ev['main_sha'], ev['witness_refs'],
                                              ev['receipt'], ev['receipt_commit'], None, NOW), 'receipt_state_not_terminal')
        # Witness force-updates its own branch only in tests; real witness never retries. Rewrite for the terminal case.
        git(self.seed, 'push', '-q', 'origin', '--delete', BRANCH)
        git(self.seed, 'worktree', 'remove', '--force', str(self.tmp / 'witness'))
        sha = self.publish_receipt(receipt(observed_main=self.main))
        ev = gate.collect(self.clone)
        self.assertEqual(ev['receipt_commit']['sha'], sha)
        v = gate.evaluate(ev['config'], ev['authority'], ev['main_sha'], ev['witness_refs'],
                          ev['receipt'], ev['receipt_commit'], None, NOW)
        self.assertEqual(v['verdict'], 'revival_permitted')
        self.assertFalse(v['executed'])
        # Prepare without the human phrase is refused; with it, only a local commit exists and remote is untouched.
        journal = self.tmp / 'state/claim.json'
        with self.assertRaises(PermissionError):
            gate.prepare_local_claim(self.clone, ev, v, 'yes', journal)
        self.assertFalse(journal.exists())
        prepared = gate.prepare_local_claim(self.clone, ev, v, gate.HUMAN_CONFIRMATION, journal, NOW)
        self.assertFalse(prepared['pushed'])
        self.assertEqual(git(self.seed, 'ls-remote', str(self.remote), 'refs/heads/main').split()[0], self.main)
        self.assertEqual(json.loads(journal.read_text())['state'], 'prepared_local')
        local = json.loads(subprocess.check_output(
            ['git', 'show', prepared['commit'] + ':coordination/authority.json'], cwd=self.clone, text=True))
        self.assertEqual(local['commander_id'], gate.SUCCESSOR)
        self.assertEqual(local['claim_base_main'], self.main)
        # A second evaluation now blocks on the unreconciled local journal.
        v2 = gate.evaluate(ev['config'], ev['authority'], ev['main_sha'], ev['witness_refs'],
                           ev['receipt'], ev['receipt_commit'], gate.read_journal(journal), NOW)
        self.assertNoActionLike(v2, 'unreconciled_local_claim_journal')
        with self.assertRaises(RuntimeError):
            gate.prepare_local_claim(self.clone, ev, v, gate.HUMAN_CONFIRMATION, journal)

    def assertNoActionLike(self, verdict, reason):
        self.assertEqual(verdict['verdict'], 'no_action')
        self.assertFalse(verdict['revival_permitted'])
        self.assertTrue(any(r.startswith(reason) for r in verdict['reasons']), verdict['reasons'])


if __name__ == '__main__':
    unittest.main()
