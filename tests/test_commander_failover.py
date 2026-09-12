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
