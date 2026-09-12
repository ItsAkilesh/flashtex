# Fleet health observations

Owner: commander-fleet. Status: ready for review, September 12, 2026.
Authority remains `coordination/authority.json`; this tool cannot transfer command.

Run from a repository checkout:

```sh
python3 scripts/fleet_health.py
python3 -m unittest discover -s tests -p test_fleet_health.py -v
```

The checker fetches origin, pins the main commit, reads its assignments, next-task
pointers, queues, control, authority and reports, then reads each assigned worker's
report from its pinned remote branch tip. It never checks out a branch or modifies
tracked files. A missing branch/report is retained as a collection error; an older
main report cannot silently stand in for a successfully fetched worker report.
`--no-fetch` skips even remote-tracking updates and explicitly marks Git freshness
unverified. The check does not launch inference, change assignments, publish Git,
change issues, start services, retry work, or claim remote processes are running.

JSON goes to stdout. Exit 0 means no actionable finding in the observations supplied;
1 means findings need review; 2 means invalid command input. An exit 0 is never a
fleet liveness certificate. All remote workers have `remote_process_verified: false`,
including those with fresh reported PID/session evidence. Stale reports, exhausted
queues, missed heartbeats, service failures and quota errors never establish that
the Commander terminated or permit a successor to take authority.

The default report/process freshness limit is 600 seconds; resource freshness is
1800 seconds. Override with `--stale-seconds` and `--resource-seconds`. Time must have
an explicit timezone. Future dates more than 60 seconds ahead are flagged.
Fresh report timestamps do not refresh resource evidence. Missing resource status
stays unknown, and no monetary balance is calculated or inferred from subscriptions.

Agent reports may include these optional machine-readable fields:

```json
{
  "machine": "worker-machine",
  "process_evidence": {
    "machine": "worker-machine",
    "pid": 12345,
    "session": "local-worker-session",
    "state": "running",
    "observed_utc": "2026-09-12T05:00:00Z",
    "in_flight": false
  },
  "resource_evidence": {
    "status": "available",
    "observed_utc": "2026-09-12T05:00:00Z"
  }
}
```

A positive integer PID or nonempty session identifies the reported process; its
machine must match the report. These are reporting fields, not remotely verified
attestations. Exact task revision ACK plus nonempty adaptation remains separately
required. Existing prose reports, comments and task rows cannot satisfy these
checks. `in_flight` values other than absent, false or `"none"` require reconciliation.
Resource statuses are `available`, `exhausted`, `blocked`; missing/unrecognized
values are unknown. These observations do not override resource authority or grant
funding, and should contain no secrets or private billing data.

Each active worker's `queue_depth` gives the current assignment count, remaining
follow-ups after `next_index`, target of two follow-ups, and whether that target
is covered. A remaining step needs a nonempty objective; blank placeholders and
malformed queues yield unknown depth rather than fictitious coverage. Fewer than
two follow-ups raises `queue_low_water`; zero also raises `queue_exhausted`.
Coverage records prepared work, not funding eligibility or executable acceptance.

`handoff_sequence` separates four observations for each worker:

1. `previous_completion` reads the previous revision's archive under
   `coordination/completions/<agent>/<task>-rN.json`. Agent, task, revision, branch,
   reported completion state and ACK must agree. The retained report must equal
   the actual report read at `worker_report_sha`; missing or inconsistent source
   records cannot pass the link check.
2. `publication` links that SHA to the matching current next pointer's
   `previous_report_sha`, with both archive and pointer read from pinned main.
   `linked_on_main` proves only the recorded handoff, not product integration,
   execution by a particular commit tool, or validation of implementation quality.
3. `next_ack` distinguishes a reported exact revision from `verified_history`:
   the ACK's `main_sha` must be an ancestor of pinned main and contain the exact
   current assignment. A fabricated revision number alone cannot verify pickup.
4. `current_completion` retains the reported state, report commit/timestamp and
   whether its ACK matches the current revision. A ready/completed report with the
   current ACK raises `pending_completion_dispatch`; an old ready report after
   publication of a new revision instead raises `next_ack_pending`.

First revisions explicitly have no previous completion. Missing archives or
mismatched SHAs raise `completion_publication_link_unverified`. A worker's
`activity` is `awaiting_next_ack`, `completion_pending_dispatch`, `blocked`,
`reported_in_progress`, or `unverified`; none is an assertion of a live remote
process. In particular, a recent PID never makes a completed worker executing.

The current Claude supervisor emits `supervisor` fields (`pid`, `cycle`,
`assignment`, `last_completed_utc`, `state`) and `claude_usage` fields including
`reported_utc` and `remaining_quota`. The checker exposes these in
`supervisor_history` as historical cycle evidence. A wrong assignment is flagged;
blocked, failed, pending or recorded in-flight states require review. Even a fresh
cycle timestamp, PID or `verified_prepaid_api` string does not replace independent
process/resource evidence. Historical `quota_availability` remains `unknown`.

Open GitHub issues are read with `gh issue list`, capped at 1000 (a full result
flags possible truncation). Titles containing “recovery” or labels containing that
word are surfaced, including older “Worker recovery needed” titles. Comments are
not examined as proof of a fix. Authentication failures and malformed issue data
are collection errors, not an empty healthy issue list. `--gh-json PATH` substitutes
saved JSON from the same `gh --json number,title,url,state,updatedAt,labels,body`
shape and avoids a GitHub request.

Local service observations use only `systemctl --user show` for
`flashtex-dispatch.service` and `flashtex-coordination-watch.service`, retaining
load/active/substate and MainPID. They observe the executing host only. An active
service with a PID does not prove that dispatch is useful or that any remote worker
started. On systems without systemd the capability failure remains visible.
`--dispatcher-json PATH` optionally reads a saved dispatcher publication journal:
`state: pending` or unknown state needs review, while `published` is only a recorded
publication result. The tool never acquires dispatcher locks or retries pending work.
An absent journal is explicitly unknown. Publication history is not current liveness.

For reproducible, offline analysis:

```sh
python3 scripts/fleet_health.py --snapshot /tmp/fleet-snapshot.json \
  --now 2026-09-12T05:00:00Z
```

`--snapshot` runs no commands. Its JSON object has `assignments`, `reports`, `next`,
`queues` mappings keyed by task/agent IDs, an `issues` array, a `services` mapping
keyed by unit name, optional `dispatcher` object, `errors` array, `fetch` status
(`ok`, `failed`, `skipped`), and pinned `main_sha`. Optional `report_sources` maps
agents to pinned report branch/SHA. `collect(root, runner=..., gh_output=...)`
returns this shape and supports an injected command runner taking argv, cwd and
timeout and returning returncode/stdout/stderr. `analyze(snapshot, now=...)` is pure
and deterministic. The tests inject every external command and use only temporary
files for CLI input; they never start a real service, model or issue operation.

For handoff replay, `completions` maps full archive paths to JSON records and
`completion_sources` maps those paths to `{sha, report}` read from Git.
`ack_sources` maps `agent/task` to `{main_sha, ancestor_of_main, assignment}` from
Git history checks. Saved snapshots are caller-supplied evidence and can be edited;
offline analysis checks internal consistency, not cryptographic authenticity.
Run collection against trusted repository refs to obtain fresh source observations.
`authority_ref` must be `origin/main` to label publication `linked_on_main`.
Snapshots collected with `--ref HEAD` or another branch explicitly leave publication
on main unverified even if their files are internally consistent.

Every subprocess has a 30-second timeout. Total runtime grows with the number of
assignments; there is no unbounded polling loop. Command failures retain operation
and exception/exit category without copying potentially sensitive stderr into the
report. The checker detects coordination gaps; a human or authorized Commander
must review underlying evidence and decide how to resolve them.
