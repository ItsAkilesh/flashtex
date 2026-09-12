import sys
from pathlib import Path
from types import SimpleNamespace
import unittest
sys.path.insert(0,str(Path(__file__).parents[1]/'scripts'))
import dispatch_loop as m
class ConflictTests(unittest.TestCase):
    def test_conflict_does_not_block_unrelated_lane(self):
        plans=[{'args':SimpleNamespace(task='A',path=['crates/index'])},{'args':SimpleNamespace(task='B',path=['crates/fonts'])}]
        assignments={'old':{'task_id':'OLD','state':'paused_by_user','owned_paths':['crates/index']}}
        accepted,skipped=m.partition_owned_plans(plans,assignments)
        self.assertEqual([p['args'].task for p in accepted],['B'])
        self.assertEqual(skipped[0]['conflicting_tasks'],['OLD'])
        self.assertTrue(skipped[0]['needs_commander_review'])
    def test_cancelled_transfer_releases_ownership(self):
        accepted,skipped=m.partition_owned_plans([{'args':SimpleNamespace(task='NEW',path=['crates/font'])}],{'old':{'task_id':'OLD','state':'cancelled','owned_paths':['crates/font']}})
        self.assertEqual(len(accepted),1);self.assertEqual(skipped,[])
    def test_parent_child_paths_conflict(self):
        accepted,skipped=m.partition_owned_plans([{'args':SimpleNamespace(task='A',path=['apps/mac/F.swift'])}],{'parent':{'task_id':'P','state':'assigned','owned_paths':['apps/mac']}})
        self.assertFalse(accepted);self.assertEqual(skipped[0]['task'],'A')
