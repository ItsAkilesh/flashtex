#!/usr/bin/env python3
"""Tests for the lane refill planner.

The test that matters most is `test_report_is_read_from_the_lane_branch`: the
real supervisor bug this tool exists to prevent is reading a lane's report from
the checked-out tree instead of from the lane's own branch, which makes every
finished lane look unstarted.  That test builds a real git repository with the
report committed on a side branch and nowhere else, so a regression to a
working-tree read fails it.
"""

import json
import os
import subprocess
import tempfile
import unittest

from daniel_supervisor import Lane, discover_lanes, plan_refill


def lane(task_id, assigned, acked, state="assigned"):
    return Lane(
        task_id=task_id,
        agent_id=f"daniel-{task_id.lower()}",
        branch=f"agent/daniel-{task_id.lower()}/x",
        owned_path="crates/x",
        assigned_revision=assigned,
        acknowledged_revision=acked,
        state=state,
        report_state=None,
    )


class FreeCapacityTests(unittest.TestCase):
    def test_matching_revisions_are_not_free(self):
        self.assertFalse(lane("FT-001", 3, 3).is_free)

    def test_behind_revision_is_free(self):
        self.assertTrue(lane("FT-001", 4, 3).is_free)

    def test_never_acked_is_free(self):
        self.assertTrue(lane("FT-001", 1, None).is_free)

    def test_cancelled_is_never_free(self):
        self.assertFalse(lane("FT-001", 4, 1, state="cancelled").is_free)

    def test_ack_ahead_of_assignment_is_treated_as_free(self):
        # An ack newer than the dispatched revision means the two records
        # disagree.  Surfacing it as free forces a human look rather than
        # silently trusting the lane's own claim over the Commander's.
        self.assertTrue(lane("FT-001", 2, 5).is_free)

    def test_status_labels(self):
        self.assertEqual(lane("FT-001", 3, 3).status, "current")
        self.assertEqual(lane("FT-001", 4, 3).status, "behind")
        self.assertEqual(lane("FT-001", 4, None).status, "never-acked")
        self.assertEqual(lane("FT-001", 4, 1, state="cancelled").status, "cancelled")


class RefillPlanTests(unittest.TestCase):
    def test_starts_refill_many_when_capacity_allows(self):
        lanes = [lane(f"FT-00{i}", 2, 1) for i in range(1, 6)]
        self.assertEqual(len(plan_refill(lanes, running=3, refill=2)), 2)

    def test_capped_by_actual_free_lanes(self):
        lanes = [lane("FT-001", 2, 1)]
        self.assertEqual(len(plan_refill(lanes, running=9, refill=2)), 1)

    def test_empty_when_every_lane_is_current(self):
        lanes = [lane(f"FT-00{i}", 2, 2) for i in range(1, 4)]
        self.assertEqual(plan_refill(lanes, running=0, refill=2), [])

    def test_finishing_the_initial_batch_does_not_end_refill(self):
        # The failure this guards: after the first batch lands, a supervisor
        # that keys off "did my children exit" stops. Free capacity is what
        # drives the loop, so a newly dispatched revision is pickup-able with
        # zero children running.
        lanes = [lane("FT-001", 3, 2)]
        self.assertEqual(len(plan_refill(lanes, running=0, refill=2)), 1)

    def test_negative_refill_is_clamped(self):
        lanes = [lane("FT-001", 2, 1)]
        self.assertEqual(plan_refill(lanes, running=0, refill=-5), [])


class GitReadTests(unittest.TestCase):
    """End-to-end against a real repository, not a mock."""

    def setUp(self):
        self.dir = tempfile.mkdtemp()
        self.git_cmd("git", "init", "-q", "-b", "main")
        self.git_cmd("git", "config", "user.email", "t@example.invalid")
        self.git_cmd("git", "config", "user.name", "t")

    def git_cmd(self, *args):
        subprocess.run(args, cwd=self.dir, check=True, capture_output=True)

    def write(self, path, payload):
        full = os.path.join(self.dir, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w") as handle:
            json.dump(payload, handle)

    def commit(self, message):
        self.git_cmd("git", "add", "-A")
        self.git_cmd("git", "commit", "-q", "-m", message)

    def seed_assignment(self, revision=3, branch="agent/daniel-x/x"):
        self.write(
            "coordination/assignments/FT-900.json",
            {
                "task_id": "FT-900",
                "agent_id": "daniel-x",
                "revision": revision,
                "state": "assigned",
                "branch": branch,
                "owned_paths": ["crates/x"],
            },
        )
        self.commit("seed assignment")

    def test_report_is_read_from_the_lane_branch(self):
        self.seed_assignment(revision=3)
        # Publish the ack ONLY on the lane branch; main never sees it.
        self.git_cmd("git", "checkout", "-q", "-b", "agent/daniel-x/x")
        self.write(
            "coordination/agents/daniel-x.json",
            {
                "state": "ready_for_integration",
                "assignment_acknowledgements": {"FT-900": {"revision": 3}},
            },
        )
        self.commit("lane publishes ack on its own branch")
        self.git_cmd("git", "checkout", "-q", "main")

        lanes = discover_lanes(self.dir, "main", "daniel-")
        self.assertEqual(len(lanes), 1)
        # A working-tree read would report None here and call the lane free.
        self.assertEqual(lanes[0].acknowledged_revision, 3)
        self.assertFalse(lanes[0].is_free)

    def test_missing_report_on_branch_counts_as_free(self):
        self.seed_assignment(revision=3)
        self.git_cmd("git", "branch", "agent/daniel-x/x")
        lanes = discover_lanes(self.dir, "main", "daniel-")
        self.assertTrue(lanes[0].is_free)
        self.assertEqual(lanes[0].status, "never-acked")

    def test_missing_branch_does_not_raise(self):
        # A dispatched lane whose branch does not exist yet must degrade to
        # "free", not crash the poll loop and take the whole supervisor down.
        self.seed_assignment(revision=3, branch="agent/daniel-x/never-created")
        lanes = discover_lanes(self.dir, "main", "daniel-")
        self.assertTrue(lanes[0].is_free)

    def test_malformed_report_json_counts_as_free(self):
        self.seed_assignment(revision=3)
        self.git_cmd("git", "checkout", "-q", "-b", "agent/daniel-x/x")
        full = os.path.join(self.dir, "coordination/agents/daniel-x.json")
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w") as handle:
            handle.write("{ this is not json")
        self.commit("malformed report")
        self.git_cmd("git", "checkout", "-q", "main")
        lanes = discover_lanes(self.dir, "main", "daniel-")
        self.assertTrue(lanes[0].is_free)

    def test_supervisor_lane_is_excluded_from_its_own_count(self):
        self.write(
            "coordination/assignments/FT-901.json",
            {
                "task_id": "FT-901",
                "agent_id": "daniel-parent",
                "revision": 5,
                "state": "assigned",
                "branch": "agent/daniel-parent/supervisor",
                "owned_paths": ["tools/daniel-supervisor"],
            },
        )
        self.commit("seed supervisor assignment")
        self.assertEqual(discover_lanes(self.dir, "main", "daniel-"), [])

    def test_other_machines_lanes_are_not_supervised(self):
        self.write(
            "coordination/assignments/FT-902.json",
            {
                "task_id": "FT-902",
                "agent_id": "mac-font-engine",
                "revision": 4,
                "state": "assigned",
                "branch": "agent/mac-font-engine/x",
                "owned_paths": ["crates/font-engine"],
            },
        )
        self.commit("seed peer assignment")
        self.assertEqual(discover_lanes(self.dir, "main", "daniel-"), [])


if __name__ == "__main__":
    unittest.main()
