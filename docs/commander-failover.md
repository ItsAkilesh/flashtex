# Jaysen Opus standby Commander

The user explicitly requested one NEW Opus standby on Jaysen's Mac. This is the
only added staffing exception. Linux stays at four active agents; the three paused
agents stay paused. Astra remains Commander until verified termination and a
serialized handoff. The standby must not run a shadow dispatcher or mutate main.
Use only the already-authorized Jaysen Max20x allowance, without overages/purchases.

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

## Revival prompt for orchestrator-jaysen-opus

> You are the designated standby successor on mac-m1max-a. Remain read-only while
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
> main, naming orchestrator-jaysen-opus and the evidence SHA/identity. Use truthful
> local primary author jay3332 and actual executor trailers. Push non-force. If
> main changes, discard the unpublished claim and rebuild after rereading authority;
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
