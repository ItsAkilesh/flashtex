#!/usr/bin/env python3
"""Free-task polling and refill planning for the daniel-* engineering lanes.

The supervisor never exits orchestration just because its initial batch
finished.  It polls the coordination records on a fixed interval, recomputes
which lanes are behind their dispatched revision, and reports how many lanes
should be started to hold a target of `running + refill` in flight.

Two facts drive every design choice here:

1.  A lane publishes its status report on *its own branch*, not on main and not
    in whatever branch happens to be checked out.  Reading
    `coordination/agents/<agent>.json` from the working tree reports every lane
    as unstarted, which is wrong and dangerous -- it invites re-dispatching work
    that is already done.  Every read below goes through `git show <branch>:...`.

2.  Free capacity is *derived*, never stored.  A stored counter goes stale the
    instant a session dies mid-lane, and this fleet has lost sessions twice.
    Recomputing `assigned_revision != acknowledged_revision` from git on every
    poll is crash-proof, because git is the only state that survives a restart.

No third-party dependencies: this runs on a stock Python 3.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from dataclasses import dataclass, asdict
from typing import Iterable

DEFAULT_PREFIX = "daniel-"
# The supervisor lane supervises; it is not one of the engineering lanes it counts.
SUPERVISOR_AGENT = "daniel-parent"
POLL_SECONDS = 30
REFILL = 2


class GitError(RuntimeError):
    pass


def git(*args: str, repo: str) -> str:
    proc = subprocess.run(
        ["git", "-C", repo, *args], capture_output=True, text=True
    )
    if proc.returncode != 0:
        raise GitError(proc.stderr.strip() or f"git {' '.join(args)} failed")
    return proc.stdout


def git_show_json(ref: str, path: str, repo: str) -> dict | None:
    """Read a JSON blob at `ref:path`, or None if absent or unparseable.

    Absent and malformed are deliberately collapsed: for refill purposes a
    report we cannot read is indistinguishable from a report that is not there,
    and both mean the same thing -- we have no evidence this lane is current.
    """
    try:
        raw = git("show", f"{ref}:{path}", repo=repo)
    except GitError:
        return None
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        return None


@dataclass(frozen=True)
class Lane:
    task_id: str
    agent_id: str
    branch: str
    owned_path: str
    assigned_revision: int | None
    acknowledged_revision: int | None
    state: str | None
    report_state: str | None

    @property
    def is_free(self) -> bool:
        """True when this lane has no published ack at the dispatched revision."""
        if self.state == "cancelled":
            return False
        return self.acknowledged_revision != self.assigned_revision

    @property
    def status(self) -> str:
        if self.state == "cancelled":
            return "cancelled"
        if self.acknowledged_revision is None:
            return "never-acked"
        if self.is_free:
            return "behind"
        return "current"


def discover_lanes(repo: str, main_ref: str, prefix: str) -> list[Lane]:
    """Enumerate assignments at `main_ref`, pairing each with its branch ack."""
    listing = git("ls-tree", "--name-only", main_ref, "coordination/assignments/", repo=repo)
    lanes: list[Lane] = []
    for path in listing.split():
        assignment = git_show_json(main_ref, path, repo=repo)
        if not assignment:
            continue
        agent_id = assignment.get("agent_id") or ""
        if not agent_id.startswith(prefix) or agent_id == SUPERVISOR_AGENT:
            continue
        branch = assignment.get("branch") or ""
        report = git_show_json(branch, f"coordination/agents/{agent_id}.json", repo=repo)
        acked = None
        report_state = None
        if report:
            report_state = report.get("state")
            acks = report.get("assignment_acknowledgements") or {}
            entry = acks.get(assignment.get("task_id")) if isinstance(acks, dict) else None
            if isinstance(entry, dict):
                acked = entry.get("revision")
        owned = assignment.get("owned_paths") or []
        lanes.append(
            Lane(
                task_id=assignment.get("task_id") or "?",
                agent_id=agent_id,
                branch=branch,
                owned_path=owned[0] if owned else "?",
                assigned_revision=assignment.get("revision"),
                acknowledged_revision=acked,
                state=assignment.get("state"),
                report_state=report_state,
            )
        )
    return sorted(lanes, key=lambda lane: lane.task_id)


def plan_refill(lanes: Iterable[Lane], running: int, refill: int) -> list[Lane]:
    """Lanes to start now to hold `running + refill` in flight.

    The target moves with `running`, so the number to start is just `refill`,
    capped by how many lanes are actually free.  Written out rather than
    expressed as `(running + refill) - running`, which cancels to the same
    thing and only obscures it.

    `running` is supplied by the caller because only the caller knows how many
    child sessions it actually has alive.  This tool never guesses or fabricates
    a child count -- an invented count is worse than no count, because it reads
    as evidence.
    """
    free = [lane for lane in lanes if lane.is_free]
    return free[: max(0, refill)]


def render_table(lanes: list[Lane]) -> str:
    rows = []
    for lane in lanes:
        mark = "TODO" if lane.is_free else "ok  "
        acked = lane.acknowledged_revision
        rows.append(
            f"{mark} {lane.task_id} {lane.agent_id:<22} "
            f"assigned r{lane.assigned_revision} acked r{acked} "
            f"{lane.owned_path}"
        )
    free = sum(1 for lane in lanes if lane.is_free)
    rows.append("")
    rows.append(f"free: {free} of {len(lanes)}")
    return "\n".join(rows)


def cmd_status(args: argparse.Namespace) -> int:
    lanes = discover_lanes(args.repo, args.main_ref, args.prefix)
    if args.json:
        print(json.dumps([asdict(lane) for lane in lanes], indent=2))
    else:
        print(render_table(lanes))
    return 0


def cmd_free(args: argparse.Namespace) -> int:
    lanes = [lane for lane in discover_lanes(args.repo, args.main_ref, args.prefix) if lane.is_free]
    if args.json:
        print(json.dumps([asdict(lane) for lane in lanes], indent=2))
    else:
        for lane in lanes:
            print(f"{lane.task_id} {lane.agent_id} r{lane.assigned_revision} {lane.owned_path}")
    return 0


def cmd_poll(args: argparse.Namespace) -> int:
    """Poll forever, emitting a refill plan whenever free capacity exists.

    Finishing the initial batch does not end the loop -- that is the whole
    point.  A lane that lands makes its successor revision visible on the next
    tick, so the supervisor always has somewhere to go.
    """
    deadline = time.monotonic() + args.max_seconds if args.max_seconds else None
    while True:
        if args.fetch:
            try:
                git("fetch", "origin", "--quiet", repo=args.repo)
            except GitError as exc:
                # A fetch failure is reported, never silently treated as "no new work".
                print(f"[{time.strftime('%H:%M:%S')}] fetch failed: {exc}", file=sys.stderr)
        lanes = discover_lanes(args.repo, args.main_ref, args.prefix)
        plan = plan_refill(lanes, running=args.running, refill=args.refill)
        stamp = time.strftime("%H:%M:%S")
        free = sum(1 for lane in lanes if lane.is_free)
        print(f"[{stamp}] free={free}/{len(lanes)} running={args.running} start={len(plan)}")
        for lane in plan:
            print(f"    start {lane.task_id} {lane.agent_id} r{lane.assigned_revision} {lane.owned_path}")
        sys.stdout.flush()
        if args.once:
            return 0
        if deadline and time.monotonic() >= deadline:
            return 0
        time.sleep(args.interval)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="daniel-supervisor", description=__doc__)
    parser.add_argument("--repo", default=".", help="repository or worktree to read")
    parser.add_argument("--main-ref", default="origin/main", help="ref carrying authoritative assignments")
    parser.add_argument("--prefix", default=DEFAULT_PREFIX, help="agent_id prefix to supervise")
    sub = parser.add_subparsers(dest="command", required=True)

    status = sub.add_parser("status", help="print every supervised lane and its drift")
    status.add_argument("--json", action="store_true")
    status.set_defaults(func=cmd_status)

    free = sub.add_parser("free", help="print only lanes behind their dispatched revision")
    free.add_argument("--json", action="store_true")
    free.set_defaults(func=cmd_free)

    poll = sub.add_parser("poll", help="poll on an interval and emit a refill plan")
    poll.add_argument("--interval", type=int, default=POLL_SECONDS)
    poll.add_argument("--refill", type=int, default=REFILL)
    poll.add_argument("--running", type=int, default=0, help="child sessions actually alive; never guessed")
    poll.add_argument("--fetch", action="store_true", help="git fetch before each tick")
    poll.add_argument("--once", action="store_true", help="single tick, then exit")
    poll.add_argument("--max-seconds", type=int, default=0, help="stop after N seconds (0 = forever)")
    poll.set_defaults(func=cmd_poll)

    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except GitError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
