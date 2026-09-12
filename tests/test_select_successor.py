import importlib.util
from pathlib import Path
import unittest
from datetime import datetime, timezone
spec = importlib.util.spec_from_file_location('selector', Path(__file__).parents[1] / 'scripts/select_successor.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
NOW = datetime(2026, 9, 12, 8, tzinfo=timezone.utc)
def row(name, amount=20, **extra):
    r = dict(machine=name, observed_utc=NOW.isoformat(), registered=True, working_verified=True, billing_authorized=True, orchestration_capable=True, usable_capacity_verified=True, session_evidence='verified-session-1', remaining=amount, comparison_profile='same-provider-same-plan-same-window-v1')
    r.update(extra); return r
class SelectionTests(unittest.TestCase):
    def test_preferred(self):
        self.assertEqual(m.select([row('linux-primary', 1), row('daniel', 90)], NOW)['selected'], 'linux-primary')
    def test_largest(self):
        self.assertEqual(m.select([row('jaysen'), row('daniel', 90)], NOW)['selected'], 'daniel')
    def test_unknown_and_unregistered(self):
        self.assertIsNone(m.select([row('a', None), row('b', 99, registered=False)], NOW)['selected'])
    def test_incomparable(self):
        self.assertIsNone(m.select([row('a'), row('b', comparison_profile='dollars')], NOW)['selected'])
    def test_stale_future_and_exhausted(self):
        for changes in [dict(observed_utc='2026-09-12T07:00:00Z'), dict(observed_utc='2026-09-12T09:00:00Z'), dict(usable_capacity_verified=False), dict(remaining=float('nan'))]:
            self.assertIsNone(m.select([row('a', **changes)], NOW)['selected'])
    def test_tie_and_no_authority(self):
        result=m.select([row('b'), row('a')], NOW)
        self.assertEqual(result['selected'], 'a'); self.assertFalse(result['claim_authorized'])
    def test_duplicate(self):
        with self.assertRaises(ValueError): m.select([row('a'), row('a')], NOW)
