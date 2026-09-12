# Project control

Status: three workers assigned; autonomous startup acknowledgement pending.
Updated: 2026-09-12T03:17:32Z.
Coordinator: primary Codex agent on `linux-primary`, appointed Commander by user.
Dispatch and reporting: [ORCHESTRATION.md](../ORCHESTRATION.md).

## Completion and duration

The user explicitly superseded the 10am stop on September 12: continue until the
user stops work or the entire project is fully implemented and tested. There is
no current automatic time deadline or demo-only completion rule. The earlier
`2026-09-12T14:00:00Z` is historical planning data, not authorization to stop.
Model calls remain bounded; timeboxes are checkpoints, not silent abandonment.

Authoritative machine-readable control: `coordination/control.json`. Only Commander
updates it, based on the user or verified whole-project acceptance evidence.
Individual worker completion never changes the global control to complete.

## Acceptance gates

1. Real native Mac/companion shells and Rust communication.
2. Original Rust compilation to preview and PDF for the declared demo constructs.
3. Measured incremental reuse, error recovery, and source navigation.
4. Both Pencil and camera capture through Grok into reviewed LaTeX insertion.
5. Verified end-to-end demo, latency evidence, dark preview and correct export.

Implementation status at this document's creation: this repository contains plans
and collaboration instructions. No application implementation has been verified.
Full compatibility remains an outstanding product requirement, not a demo claim.

## Coordinator updates

Every 15–20 minutes of active execution, reconcile per-agent reports: next gate,
dependencies, optimistic/likely/pessimistic remaining time, available funding,
and integration reserve. Estimate the critical path, not the sum of all parallel
task durations. State confidence and evidence for estimates.

If the forecast exceeds the deadline, report the gap immediately and propose
reassignment, a cheaper experiment, or explicit scope options. Continue useful
authorized work. Do not quietly remove requirements or promise guaranteed delivery.
