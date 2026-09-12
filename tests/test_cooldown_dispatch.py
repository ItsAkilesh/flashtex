import importlib.util
from pathlib import Path
import tempfile
import unittest
from datetime import datetime, timezone
from unittest.mock import patch
spec=importlib.util.spec_from_file_location('cooldown',Path(__file__).parents[1]/'scripts/cooldown_dispatch.py'); m=importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
class CooldownTests(unittest.TestCase):
    def test_exact_boundary(self):
        s={'resume_utc':'2026-09-12T08:21:08Z'}
        self.assertFalse(m.due(s,datetime(2026,9,12,8,21,7,tzinfo=timezone.utc)))
        self.assertTrue(m.due(s,datetime(2026,9,12,8,21,8,tzinfo=timezone.utc)))
    def test_no_work_before_deadline(self):
        with tempfile.TemporaryDirectory() as d, patch.object(m.subprocess,'check_output') as call:
            result=m.run(Path(d),{'resume_utc':'2026-09-12T08:21:08Z'},Path(d)/'journal',datetime(2026,9,12,8,tzinfo=timezone.utc))
            self.assertEqual(result['state'],'cooldown');call.assert_not_called()
    def test_pending_is_not_replayed(self):
        with tempfile.TemporaryDirectory() as d, patch.object(m.subprocess,'check_output') as call:
            p=Path(d)/'journal';p.write_text('{"state":"pending"}')
            self.assertEqual(m.run(Path(d),{'resume_utc':'2026-09-12T08:21:08Z'},p,datetime(2026,9,12,9,tzinfo=timezone.utc))['state'],'pending');call.assert_not_called()
