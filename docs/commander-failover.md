# Resource-selected standby Commander

Latest user instruction supersedes the fixed Jaysen designation below. Keep linux-primary
as Commander while usable. Run `python3 scripts/select_successor.py evidence.json`
on independently verified, at most five-minute-old machine evidence. Each row requires
registered, working_verified, billing_authorized, orchestration_capable and
usable_capacity_verified booleans, an observed_utc timestamp and session_evidence.
Remaining numeric capacity requires a comparison_profile that identifies identical
provider/plan/window units; do not label unlike resources alike. Unknown balances
and incomparable profiles cannot establish a winner. Daniel is eligible only after
actual registration and live session evidence. Equal capacity breaks ties by machine ID.
Selection returns claim_authorized=false: it does not weaken any terminal, journal,
publisher-stop or non-force authority gate. Before claim rerun with fresh evidence;
name the selected machine’s actual unique orchestrator ID. The former Jaysen standby
remains read-only unless selected. No new model calls are needed for polling.

## Standby scope

The existing Jaysen standby is a candidate, not the predetermined successor.
Select the eligible machine using fresh comparable evidence as described above.
Use only that machine's existing authorized billing route; no purchases or
unapproved overages. Follow the latest staffing override in AGENTS.md and
coordination/control.json. A standby must not run a shadow dispatcher or mutate main.

## Current operational boundary

`scripts/commander_failover.py` is a Linux witness that stays read-only while the Commander lives. It pins the actual
hosting Codex PID, Linux boot ID and process start ticks in `coordination/failover.json`.
A recycled PID is not mistaken for the old process. It checks publisher service
state, live publication processes and unresolved journals. Unknown or inaccessible
state blocks. It never kills this session, spends model credits, changes authority, or marks a
takeover authorized. The explicit configuration permits stopping only
flashtex-dispatch.service after the exact pinned hosting process has terminated
and current authority still names Astra. Other jobs and unresolved journals
remain blockers; they are never blindly killed or replayed. The private status file is not
itself an independently authenticated remote proof.

The live process may remain present after quota exhaustion. That case is deliberately
blocked. Official [Codex noninteractive documentation](https://learn.chatgpt.com/docs/non-interactive-mode)
describes turn.failed events, but a failed turn does not establish that this hosting
TUI session has terminated. No quota-to-terminal hook for this actual session has
been verified. Terminal-only Git witness publication is implemented and tested against an
isolated bare remote; actual Mac receipt consumption and its session/PID still
require verification. Therefore unattended quota-triggered failover is NOT
currently proven operational. GH20 tracks those concrete remaining gates.

Local read-only monitor (no model calls):

```sh
python3 scripts/commander_failover.py --repo /home/natkarri/flashtex-orchestrator \
  --config coordination/failover.json --output /tmp/flashtex-commander-witness.json \
  --watch --interval 30
```

Before any automated handoff is enabled, validate the actual terminal signal,
authenticated witness transport, stopping of the dispatcher and every in-flight
publisher, journal reconciliation and serialized authority claim in an isolated
fixture. Do not weaken a gate merely to make the standby report green. Quota text,
heartbeat age, an idle turn and an inaccessible Linux machine are insufficient.

## Copyable revival prompt for the selected successor

> You are a candidate standby; the fresh resource selector chooses the successor. Remain read-only while
> orchestrator-astra is active. Fetch main and read AGENTS.md, ORCHESTRATION.md,
> coordination/authority.json, control.json, RESOURCES.md, COMMANDER-RESUME.md,
> docs/INDEX.md and docs/evidence/astra-integration-2026-09-12.md. Review all exact
> current assignments, completion records, queues, branch tips, open issues and
> resource reports. A ready report is not proof a worker is running or integrated.
>
> Before takeover, independently verify the exact pinned Commander session/process
> has terminated and every dispatch/publication/integration job is stopped. Reconcile
> all uncertain publication journals; never replay an uncertain paid call. If any
> evidence is missing, continue standby and existing authorized product work only.
> Positive evidence must come from the actual host/runtime, not a heartbeat timer.
>
> After verified quiescence, prepare ONE authority-only claim from exact fetched
> main, naming the selected actual orchestrator ID and the evidence SHA/identity. Use the executing
> machine’s truthful primary author, authenticated local coauthor and actual
> executor trailers; preserve Jaysen’s primary-author exception when applicable.
> Push non-force. If main changes or publication is uncertain, stop and reconcile
> the claim journal and current authority; never automatically retry;
> if another successor already claimed, remain quiesced. Reread your successful
> claim before every global mutation. Resume global queues, integration, resource
> accounting and 30-second deterministic polling under your explicit authority.
>
> Keep current plus two meaningful followups for eligible workers, preserve dirty
> worktrees and paused staffing, and verify current ACK/session evidence. Continue
> improvement until explicit user stop. Exact PDF bytes, zero-pixel equality and
> typing-to-visible latency are separate gates. Do not invent completion estimates,
> quota balances, native results or paid-call authorization.

## Terminal-only Git transport

After exact process death, verified stopped service/jobs and resolved journals,
the witness may publish only
`agent/orchestrator-witness/astra-30612338:coordination/failover/witness-linux.json`.
It cannot push main or alter authority. The receipt pins observed main and process
identity. A private pending publication journal prevents duplicate retries after
an uncertain commit/push; the successor must reconcile the exact branch manually.
The test fixture verifies main remains byte-for-byte at its prior Git commit.

The standby must fetch this branch, inspect the actual receipt plus its source,
match process pin and observed main to the current authority/config, and independently
confirm the host witness is the configured monitor. A receipt is not a blanket
permission to take over a different/restarted session. If main/authority changed,
reconcile before any claim. No claim runner has been installed on the Mac yet.

## Autonomous claim executor (explicitly authorized, no sleeping-user prompt)

`scripts/claim_commander.py` now implements the one-shot non-force authority claim.
The parent monitor must independently review the exact witness branch commit and
supply `--reviewed-witness SHA`, plus fresh capacity evidence, actual successor
machine/unique ID, truthful local executor/coauthor, and a private journal path.
It checks the configured witness author/executor/direct parent, exact process pin,
main SHA, <=120-second terminal receipt, every stopped service, no pending jobs,
current project/authority and resource selection before creating any claim. The
claim changes only authority.json; any competing main push rejects without merge
or retry. A pending journal requires reconciliation, including uncertain delivery.
The isolated bare-remote test exercises a real claim and duplicate/competing refusal.

This executor calls no model and requires no extra confirmation beyond the existing
user authorization. It must NOT run while Astra lives. Install/route the existing
remote monitor to this command only after its host can validate the actual witness
and resource evidence. A script being published does not prove that remote hookup
or a quota-to-terminal host adapter works. After successful claim, the selected
parent invokes its already-authorized orchestrator session and resumes queues;
first reread authority. Re-pin the new host witness before future succession.

## Local publication serialization

Commander main/control publishers must hold `dispatch_loop.publication_lock(root)`
from fresh fetch/authority check through the non-force push. It resolves Git common
state across linked worktrees; dispatcher scans hold the same lock. Remote races
still fail closed and require journal reconciliation. The worktree dispatcher lock
continues to reject duplicate dispatchers. This lock is not takeover authority.
