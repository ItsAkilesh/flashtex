# Revival packet — TEMPLATE (all values UNVERIFIED)

Standby: `orchestrator-jaysen-opus` (mac-m1max-a). Active Commander at template
time: `orchestrator-astra` (linux-primary, handle `/root/runtime_validator`).

This template is not a claim. It is filled in only when every field below has
a verified value and a named source. Any remaining `UNVERIFIED` blocks takeover.
Silence, heartbeat gaps, quota text, an idle/completed turn, process liveness
alone, a fetch/network failure, or an inaccessible Linux machine never fill a
field. See `tools/commander-standby/README.md` for the procedure this packet gates.

## A. Trigger

| Field | Value | Source / evidence path |
|---|---|---|
| Trigger type (`explicit_quiesced_handoff` / `positive_termination`) | UNVERIFIED | |
| Who named `orchestrator-jaysen-opus` as the single successor (commit SHA / issue comment URL) | UNVERIFIED | |
| Commander's fail-closed terminal-signal gate: exercised? result? | UNVERIFIED | |
| User instruction reference (issue/comment/date) authorizing this takeover | UNVERIFIED | |

## B. Astra session termination evidence (linux-primary)

| Field | Value | Source / evidence path |
|---|---|---|
| Exact session/handle | `/root/runtime_validator` (per authority.json) — termination: UNVERIFIED | |
| Pinned process (failover.json): pid 268514, start_ticks 30612338, boot_id f1d9f6a2-ccc1-4c85-bfcb-e629905e9820 — exact match confirmed terminated? | UNVERIFIED | |
| Terminal witness receipt: branch `agent/orchestrator-witness/astra-30612338`, file `coordination/failover/witness-linux.json` — exists? commit SHA? process pin and observed main match current failover.json/authority.json? | UNVERIFIED | |
| Termination signal / exit status / timestamp (UTC) | UNVERIFIED | |
| Who observed it (person or fail-closed gate) and how | UNVERIFIED | |
| Last Astra-authored commit on main (SHA, time) | UNVERIFIED | |
| Last Astra issue comment (URL, time) | UNVERIFIED | |
| Positive statement that no Astra model call may still be in flight | UNVERIFIED | |

## C. Stopped jobs (each must be positively stopped, not merely silent)

| Job | State | PID / unit status / timestamp | Source |
|---|---|---|---|
| `flashtex-dispatch.service` (dispatcher, `--commander-id orchestrator-astra`) | UNVERIFIED | | |
| `flashtex/dispatcher-publication.json` journal state (`pending` blocks resume) | UNVERIFIED | | |
| Cursor publication sessions (`cursor-agent`) | UNVERIFIED | | |
| Integration worktrees (`/home/natkarri/flashtex-astra-integration`, `flashtex-mac-integration`, `flashtex-compiler-integration`): `MERGE_HEAD`, `integration.json` state | UNVERIFIED | | |
| Any `worker-sync-publication.json` pending fast-forward | UNVERIFIED | | |
| Any promotion (`integrate.py promote`) in progress | UNVERIFIED | | |
| Coordination watcher / awake units (`flashtex-coordination-watch`, control-driven services) | UNVERIFIED | | |
| Hosted product engineers still running (root preview-controller, compiler_corpus fonts, supervisor_api_review rendering) — they continue; list their handles | UNVERIFIED | | |
| Paused agents (bridge-context-jobs, commander-project-index, commander-edit-ledger) — untouched, dirty work preserved | UNVERIFIED | | |

## D. Fresh main

| Field | Value | Source |
|---|---|---|
| `git fetch origin --prune` time (UTC) | UNVERIFIED | |
| `origin/main` SHA at claim (full 40 hex) | UNVERIFIED | |
| `authority.json` on that SHA: `commander_id` / `authority_state` / `claim_base_main` / `updated_utc` | UNVERIFIED | |
| `control.json` on that SHA: `state` | UNVERIFIED | |
| Newest dispatcher commit on that SHA (author, time) | UNVERIFIED | |
| Open `[recovery]` issues and owners | UNVERIFIED | |

## E. Claim JSON diff (prepared, NOT pushed)

```diff
--- coordination/authority.json (origin/main @ UNVERIFIED)
+++ coordination/authority.json (claim branch @ UNVERIFIED)
-  "commander_id": "orchestrator-astra",
+  "commander_id": "orchestrator-jaysen-opus",
-  "machine": "linux-primary",
+  "machine": "mac-m1max-a",
   "claim_mode": UNVERIFIED,
-  "predecessor": "orchestrator-sol",
+  "predecessor": "orchestrator-astra",
   "predecessor_main_writes_quiesced": UNVERIFIED,
-  "claim_base_main": "9da7e48148d91872dc0514119e1a04568ebfcb51",
+  "claim_base_main": UNVERIFIED,
   "dispatch_service": UNVERIFIED,
   "dispatch_service_pid_at_claim": UNVERIFIED,
   "updated_utc": UNVERIFIED,
   "agent_handle": UNVERIFIED,
   "model": UNVERIFIED,
   "handoff_evidence": UNVERIFIED,
   "claim_commit_executor": UNVERIFIED,
   "cursor_quota_state": UNVERIFIED
   (publication_rule and revival_rule unchanged)
```

| Field | Value |
|---|---|
| Claim branch name | UNVERIFIED |
| Claim commit SHA (local) | UNVERIFIED |
| Commit author / executor / trailers (truthful) | UNVERIFIED |
| Refetch result immediately before push: `origin/main` == claim base? | UNVERIFIED |
| Non-force push result | UNVERIFIED (not attempted) |

## F. Resources at claim

| Field | Value | Source |
|---|---|---|
| Pool / allocation used by the successor | `claude-mac20x` / `claude-mac20x-standby` — quota: UNVERIFIED | |
| Linux Claude route | blocked (API-only, unverified funding) — confirm unchanged: UNVERIFIED | |
| Cursor state on mac-m1max-a | not logged in at template time — recheck: UNVERIFIED | |
| Purchases / overages | none authorized; none made: UNVERIFIED | |

## G. Sign-off

| Field | Value |
|---|---|
| Packet completed by | UNVERIFIED |
| Completed at (UTC) | UNVERIFIED |
| Every field above verified (yes/no) | no |
