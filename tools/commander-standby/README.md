# Standby Commander tools (`orchestrator-jaysen-opus`)

These tools belong to the read-only standby successor set up under GitHub issue
#20. They collect evidence only. Nothing in this directory writes to `main`,
`coordination/control.json`, `coordination/authority.json`,
`coordination/queues/**`, `coordination/assignments/**`, `coordination/next/**`,
comments on issues, dispatches work, or promotes anything.

## `monitor.sh` — evidence collection only

```sh
bash tools/commander-standby/monitor.sh            # 60 s interval; Ctrl-C to stop
STANDBY_MONITOR_INTERVAL=30 bash tools/commander-standby/monitor.sh
```

Each round it runs `git fetch --prune origin` (the only local side effect:
remote-tracking refs are updated) and prints, from `origin/main`:

- the main tip (SHA, author, commit time, subject);
- `coordination/authority.json` fields: `commander_id`, `authority_state`,
  `claim_mode`, `claim_base_main`, `dispatch_service`,
  `dispatch_service_pid_at_claim`, `predecessor`,
  `predecessor_main_writes_quiesced`, `updated_utc`;
- `coordination/control.json` fields: `state`, `stop_at_utc`,
  `local_max_active_agents`, `paused_agents`;
- `coordination/failover.json` (published by Astra in main `0512585`/`23f1dd1`):
  `predecessor_id`, `successor_id`, `mode`, the pinned `process`
  (pid/start_ticks/boot_id), `publisher_services`, `quota_terminal_adapter`,
  `mac_standby_ack`, `witness_branch`;
- terminal witness branches `agent/orchestrator-witness/*` on origin, and, when
  the configured `witness_branch` exists, the receipt
  `coordination/failover/witness-linux.json` it carries (see
  `docs/commander-failover.md`, "Terminal-only Git transport"). A receipt is
  evidence to be verified against the pinned process and current authority; it
  is not blanket permission to take over;
- dispatcher evidence: the newest commit (author identity and commit time) that
  touched `coordination/queues`, `coordination/next`, or
  `coordination/assignments` — this shows the last publication, not liveness;
- open recovery issues (`[recovery]` in the title, plus the `recovery` label if
  one ever exists — the repository currently has no such label), and the
  timestamp/author of the latest comment on issue #20;
- lines in `coordination/COMMANDER.md` and `coordination/COMMANDER-RESUME.md`
  matching quiesce/handoff/successor/terminate/standby/takeover markers.

Each round ends by running `standby_gate.py` (below) and printing its verdict.
It exits only on SIGINT/SIGTERM (both trapped; when started as a background job
from a non-interactive shell, bash ignores SIGINT, so use SIGTERM there). A
fetch failure is printed as such and is explicitly **not** takeover evidence.

## `standby_gate.py` — deterministic, fail-closed revival gate

```sh
python3 tools/commander-standby/standby_gate.py            # evaluate; JSON verdict
python3 tools/commander-standby/standby_gate.py --once     # one guarded invocation
python3 -m unittest tools/commander-standby/test_standby_gate.py
```

Inputs (all read from a fresh `origin` fetch, nothing written): `origin/main`
SHA, `coordination/authority.json`, `coordination/failover.json`, the remote
heads `agent/orchestrator-witness/*`, and, if the configured `witness_branch`
exists, its tip commit metadata and `coordination/failover/witness-linux.json`.
`evaluate()` is a pure function over that data so it can be tested with
synthetic records.

Verdict is `no_action` unless **all** of the following hold, in which case it
is `revival_permitted` — a review flag; `executed` is always `false`:

- `failover.json` names `orchestrator-jaysen-opus` as successor in
  `read_only_standby` mode with a valid process pin;
- `authority.json` still names the predecessor (`orchestrator-astra`) as
  `active` — a different id is a competing/newer claim, our own id means
  already claimed, both block;
- the configured witness branch exists on origin and carries a receipt whose
  `state` is `terminal_quiescent_observed`, `reasons` is empty,
  `claim_authorized` is exactly `false` (a receipt asserting authorization is
  rejected), identities match, and `process_pin` equals the current
  `failover.json` pin exactly (stale/recycled pids fail);
- every `publisher_services` entry is `inactive`/`failed` with pid 0,
  `publisher_processes` and `journal_blockers` are empty;
- `observed_main_sha` equals the freshly fetched `origin/main` (main moving
  after the observation means someone wrote main: re-observe), the receipt is
  not future-dated, and the receipt commit is authored by
  `FlashTeX Witness <witness@flashtex.invalid>`, carries
  `Commit-Executor: commander_failover.py deterministic local witness`, and
  sits directly on the observed main;
- no local claim journal exists (a `pending`/`pushed`/`prepared_local` journal
  from an earlier invocation must be reconciled by hand — this is the lost or
  uncertain-push guard).

Absent witness ref, an alive/blocked receipt, quota or capacity error text
(for example issue #22 "Hosted font/render slots temporarily blocked by model
capacity"), idle turns, heartbeat age and an unreachable Linux host all yield
`no_action`.

### One-shot guarded invocation

`--once` evaluates and, when permitted, additionally prints
`would_prepare_claim` (the exact `authority.json` the takeover would publish)
and manual next steps. It writes nothing.

`--once --prepare --journal <path> --human-confirmed "<phrase>"` goes one step
further **only** with the exact phrase
`I verified tools/commander-standby/revival-packet.md and authorize a LOCAL claim commit`:
it creates the claim commit in a detached temporary worktree on the fetched
main and records a local journal. It never pushes; the non-force push to
`main` remains a separate manual command printed in the result, and a rejected
push means another claim won. Without the phrase, or with any `no_action`
verdict, or with an existing journal, `--prepare` is refused and nothing is
written. Tested against an isolated bare remote: remote `main` stays
byte-identical.

Exercised on 2026-09-12 against the real origin: verdict `no_action`, reason
`no_terminal_receipt_published`; `--prepare` refused with no journal written.

## What the monitor output can and cannot establish

- It can show that `main` or the authority record changed, that a dispatcher
  publication happened at some time, and that a recovery issue exists.
- It cannot show that the Astra session terminated, that its dispatcher
  (`flashtex-dispatch.service` on `linux-primary`) is stopped, or that no Cursor
  / integration / promotion job is running. Those facts live on `linux-primary`
  and must come from the Commander's own fail-closed terminal-signal gate or a
  person with access to that machine.

## Takeover preconditions (verbatim, issue #20)

> Takeover requires independently verified terminal evidence for exact Astra
> session AND all Astra dispatch/publication/integration jobs stopped, followed
> by one non-force authority claim from freshly fetched main. Heartbeat gaps,
> quota text, idle/completed turn, process liveness alone, or inaccessible Linux
> do not authorize takeover. Local hosted terminal-signal integration is
> currently UNVERIFIED; I am implementing a fail-closed evidence gate and exact
> revival packet. Do not claim automatic failover operational until both this
> signal and your monitor are exercised.

And `coordination/authority.json` `revival_rule` (verbatim):

> Require explicit quiesced handoff or positive exact process/session
> termination plus stopped publication jobs; silence, timeout, stale heartbeat,
> quota error, and network failure are insufficient.

## How a takeover WOULD be executed (documented, NOT run)

This is the non-force claim procedure from AGENTS.md, `docs/autonomous-workers.md`
("Commander continuity and recovery issues" and the copyable revival prompt) and
`authority.json`'s `publication_rule`. It is recorded here so the procedure is
reviewable in advance. None of it is executed by anything in this directory, and
none of it may be executed while `authority.json` still names an active
`orchestrator-astra` without the evidence above.

1. Fill in `tools/commander-standby/revival-packet.md` completely. Every field must be a
   verified value with its source; any `UNVERIFIED` field means stop.
   Required: positive termination evidence for the exact Astra session
   (`/root/runtime_validator` on `linux-primary`, pinned in
   `coordination/failover.json` as pid 268514 / start_ticks 30612338 /
   boot_id f1d9f6a2-…) **and** evidence that `flashtex-dispatch.service`, any
   Cursor publication, any `scripts/integrate.py` worktree (`MERGE_HEAD`,
   pending `integration.json`), and any promotion are stopped — or an explicit
   quiesced handoff on `main` naming `orchestrator-jaysen-opus` as the single
   successor. If the terminal witness receipt on
   `agent/orchestrator-witness/astra-30612338` exists, fetch it, read
   `coordination/failover/witness-linux.json` and its commit, and check that
   its process pin and observed main match the current `failover.json` and
   `authority.json`; the receipt transport is marked "end-to-end Mac claim
   unverified" by its author and must be treated accordingly.
2. Inspect pending paid-call/publication journals named in the packet
   (`flashtex/dispatcher-publication.json`, `worker-sync-publication.json`,
   `integration.json`) before resuming anything; never replay an uncertain
   paid call.
3. `git fetch origin --prune`; pin `SHA=$(git rev-parse origin/main)`.
   Create a clean branch from exactly that SHA (for example
   `agent/orchestrator-jaysen-opus/authority-claim`).
4. Edit `coordination/authority.json` on that branch only:
   `commander_id: orchestrator-jaysen-opus`, `machine: mac-m1max-a`,
   `authority_state: active`, `claim_mode` = `explicit_quiesced_handoff` (if
   Astra named this successor and quiesced) or the positive-termination mode,
   `predecessor: orchestrator-astra`, `predecessor_main_writes_quiesced: true`
   only if actually verified, `claim_base_main: <SHA>`,
   `dispatch_service` / `dispatch_service_pid_at_claim` reflecting the real
   stopped state, `updated_utc`, `handoff_evidence` (the packet's exact
   evidence, no secrets), `claim_commit_executor` (truthful), and keep the
   `publication_rule` and `revival_rule` unchanged. Add a concise Commander
   packet update in `coordination/COMMANDER.md` and the successor's own
   `coordination/agents/orchestrator-jaysen-opus.json` report.
5. Commit with the currently authorized truthful executor. On `mac-m1max-a`
   the user is primary author (`jay3332`); trailers must name the implementing
   agent and the actual executor; Cursor is not logged in here, so the
   user-authorized direct-git fallback (RESOURCES.md "Current publication
   override") applies and must be stated as such, never as Cursor execution.
6. Refetch. If `origin/main` != `<SHA>`, abandon the claim and rebuild it from
   the new tip (step 3). Otherwise push **non-force**:
   `git push origin HEAD:refs/heads/main`. A rejected push means another claim
   won: stop all global writes.
7. Only after the claim is on `main`: reread `authority.json` from a fresh
   fetch before every later main/control write; restore the fenced dispatcher
   only under `--commander-id orchestrator-jaysen-opus`; reconcile worker ACKs,
   sessions and resource evidence; keep the four-agent Linux cap and all
   billing constraints.

Anything short of step 1 being fully verified is "keep existing non-overlapping
work, report the uncertainty, open/update a `[recovery]` issue, do not claim
leadership".

## `revival-packet.md`

A blank template listing every evidence field a takeover needs. All values are
left `UNVERIFIED`; it is filled only when real evidence exists.
