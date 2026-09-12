# Commander handover: orchestrator-astra -> claude (mac-m5pro-kabir)

Declared: 2026-09-12T16:05Z by the user, who is the authority for this change.

## Why this file exists before any dispatch

`orchestrator-astra` is currently the active Commander and was dispatching five
minutes before this was written. Two commanders issuing assignments at once
produces exactly the duplicate-ownership failure recorded in
`docs/resources/ft005-duplication.md`, where three agents independently built the
same thing because no single owner held the board.

So this handover is announced before I issue a single dispatch, not after.

## What I am taking over

- Assignment of tasks across all agents and machines.
- Keeping agents running: detecting stalled or dead workers and re-tasking them.
- Integration sequencing decisions.

## What I am asking of orchestrator-astra

Stop issuing new dispatches. Finish or hand back anything mid-flight, and record
it here. If you disagree with this handover, say so in this file rather than
dispatching in parallel; the user decides, not either of us.

Until astra acknowledges, I will:

- Not re-dispatch any task that already has a live owner making progress.
- Not cancel any in-flight assignment.
- Restrict myself to restarting agents that are demonstrably stopped, and to
  assigning work that is genuinely unowned.

That ordering is deliberate: an idle agent costs throughput, but a contradicted
assignment costs correctness and trust, and is much harder to unwind.

## Liveness census at takeover

Measured from remote branch tips, because an agent record on `main` is stale by
design: agents report on their own branches and those reports only reach main at
integration. Judging liveness from main would have declared most of the fleet
dead when it is not.

Active within the last 40 minutes, 13 branches:
mac-admission-groups, commander-preview-performance, commander/dispatch-service,
aarush-macbook/companion-capture, commander-render-core,
claude/compiler-foundation, chatgpt-a/companion-reliability,
aarush-macbook/test-coverage-v2, mac-claude-a/mac-shell,
mac-render-pipeline/delta-proposal, mac-search-reconcile/gh39,
commander-runtime-display, mac-admission-correlation.

Quiet for 60 minutes or more, and therefore to be checked rather than assumed
dead: mac-v2-conformance, mac-core-review, mac-realworld-corpus,
mac-helper-display, mac-large-document, mac-citation-rename, mac-nearby-errors,
mac-capture-list-handoff, mac-packaging-tfm, mac-preview-anchoring.

Quiet is not the same as stopped. Some of those branches are finished work
awaiting integration. Each is verified before any conclusion is drawn.

## Standing rule I am adopting

No agent is declared dead on one observation. A single process check returning
zero is not proof a worker exited: I made that mistake earlier today and killed a
cleanup while a worker was still writing. Liveness is judged on branch activity
plus a second observation separated in time.
