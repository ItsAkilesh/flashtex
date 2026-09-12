# mac-bridge-recovery handoff

- Updated UTC: 2026-09-12T10:02Z
- Agent / parent / machine alias: mac-bridge-recovery (Claude Code subagent) /
  mac-claude-a / mac-m1max-a
- Task / acceptance gate / owned paths: lane "Bounded automatic relaunch of the
  capture bridge and edit-ledger helpers after an abnormal exit, with correct
  reconciliation" (parent dispatch, issue #2 thread; no FT number / assignment
  file, so no ack) / full `swift test` in apps/mac green with the real helpers /
  `apps/mac/Sources/FlashTeXMac/{BridgeSession,BridgeClient,EditLedgerClient}.swift`,
  `apps/mac/Tests/FlashTeXMacTests/{BridgeClientTests,BridgeRecoveryTests,
  ShellModelBridgeTests,RealBridgeTests,RealEditLedgerTests}.swift`,
  `apps/mac/Tests/FlashTeXMacTests/Fixtures/{fake_bridge,fake_edit_ledger}.py`,
  `coordination/mac-bridge-recovery.md`, `coordination/agents/mac-bridge-recovery.json`
- Branch / code revision / main integrated through:
  `agent/mac-bridge-recovery/relaunch` (from origin/agent/mac-claude-a/mac-shell
  92a052c, merged mac-shell 35b4e12 at the final checkpoint) / see JSON / origin/main as contained in mac-shell 35b4e12
  (merge-base 780f145).
- State: ready for integration
- Ready behavior and evidence:
  - `BridgeSession` relaunches either child after an abnormal exit
    (status != 0) with the same executable/arguments/store, using the shell's
    worker policy: 0.2 / 1 / 3 s backoff, at most `maxRelaunches` (3) per
    minute per child; beyond that the exit stays visible with the hint
    "Edit > Attach Capture Bridge to retry". Clean exits (0) and explicit
    `terminate()` never relaunch; a detach cancels a pending relaunch; the
    work item only replaces the process that actually exited (a helper already
    replaced by the poisoned-handle `reopenLedger` path is left alone); events
    from a replaced process are identity-guarded (`client === c`).
  - After a **bridge** relaunch: a receipt in flight or owed at the crash
    leaves live tracking as capture state `uncertain` ("settling from the edit
    ledger"); deferred edits are dropped; a durable edit the editor has not
    adopted yet stays expected (`expectedApplication` is no longer cleared on
    exit) and its receipt follows the adoption on the new process; the
    ledger-guarded `reconcile` runs against the live source (confirmed / replayed
    from the retained pre-edit snapshot / held as evidence), the document is
    reopened at the live revision, and the pinned destination is re-pinned
    identically when the source is still at the pinned revision (binding
    compared) — otherwise dropped and reported ("pin again").
  - After a **ledger** relaunch: `openLedger` on the same store with the live
    text/revision (aligned / buffer replaced the store / durable adoption via the
    existing `onAdoptDocument`), then `reconcile`, then reopen on the bridge.
    A transient `replace_document` failure (helper exited mid-typing) no
    longer poisons the session as "diverged"; the durable store keeps following
    typing while the bridge is down (`edited` no longer gated on `running`).
  - `capture_submit` that the bridge never answered (transient failure) is
    shown as `uncertain` with "resubmit with the same capture ID"; the identical
    resubmission is idempotent on both the fake and the real bridge.
  - New session hooks (defaults are no-ops): `onRelaunched(child, summary)`,
    `onRevisionFloor(revision)`; new read-only state `relaunchCount[child]`,
    `relaunching`, `detached`; status after a relaunch reads
    "attached: … · relaunched N× · main.tex open at revision R".
  - Test doubles: `fake_bridge.py` now persists its journal in `--store`
    (`fake_journal.json`, atomic rewrite; documents/anchors stay in memory like
    the real bridge) and gains `%crash`, `%crash-once`,
    `%crash-once-journaled`, `crash-status-*`, `crash-applied-once-*`,
    `crash-applied-before-once-*`; `fake_edit_ledger.py` gains
    `crash-before-commit-*`, `crash-after-commit-*` (apply capture IDs),
    `%ledger-crash-once` (replace text) and an idle `crash` operation.
  - Tests: BridgeRecoveryTests §6 (10 new): relaunch + reconcile restores
    state and the destination; destination lost when the source moved past the
    pin; receipt in flight settled exactly once (bridge journaled it / did not);
    durable edit awaiting adoption survives; ledger crash during apply inserts
    exactly once (after-commit reply lost, before-commit retry); ledger idle and
    mid-typing crash relaunched and realigned; pending-receipt evidence kept;
    per-minute limit; detach cancels and clean exit never relaunches; crash
    during capture_submit → uncertain, retryable, one journal record.
    RealBridgeTests / RealEditLedgerTests: the real binaries are SIGKILLed
    (`pgrep -f "<helper> --store <unique dir>"`), relaunched, journal/flock
    verified, insertion confirmed afterwards, no process left after detach.
- Incomplete behavior / blockers / needs from others:
  - Parent-retained `ShellModel+Bridge.swift` diffs (not applied; see the
    final report): wire `onRelaunched` → `captureNote`, `onRevisionFloor` →
    `advanceEditorRevision(atLeast:)`, and relax the `bridgeTextChanged` guard
    so the durable ledger keeps following typing while the bridge is down.
    Without them the branch is self-contained (tests pass): the relaunch's
    `openLedger` realigns the store with the buffer, but text typed during a
    bridge outage only reaches the ledger at the relaunch, and the shell's
    capture note does not mention the relaunch (the bridge status/log do).
  - `apps/mac/README.md` paragraph on relaunch (parent-owned; suggested text in
    the report).
  - A pending transaction whose `.tex` export was still owed (export failure)
    is settled by reconciliation after a bridge relaunch without that export;
    the durable store stays authoritative and the user saves manually (the
    export path was already broken in that case).
  - A protocol violation terminates the process with SIGTERM (status 15) and
    therefore also relaunches (bounded); `%trailing` (exit 0) does not.
- Interface changes / consumer actions: `BridgeSession.client` is now
  `private(set) var` (was `let`); `CaptureState.uncertain` added (ContentView
  shows `rawValue`, nothing switches exhaustively); `reconcile`'s
  `currentSource` is `@escaping`; wire-level protocols unchanged.
- Reviewed peer revisions / resulting adaptations: origin/agent/mac-claude-a/
  mac-shell 92a052c (base; ShellModel worker relaunch policy mirrored: same
  delays, limit, status wording, detach cancellation).
- Validation commands / results / artifact paths: `cd apps/mac && swift test`
  with FLASHTEX_COMPILER / FLASHTEX_PDF / FLASHTEX_BRIDGE / FLASHTEX_EDIT_LEDGER /
  FLASHTEX_PREVIEW_CONTROLLER pointing at the release binaries in the main
  checkout: 382 tests, 0 failures, 7 env-gated skips (project-files helper not
  built ×3, screenshot/evidence dirs ×4), run before merging mac-shell 35b4e12.
  After the merge (load average 51 with 16 lanes on this Mac): 393 tests, 9
  skips, 7 failures — all in `SourceEditorViewTests.testLargeDocumentKeystroke…`
  (per-keystroke wall/CPU gate of another lane, load-induced; also fails when
  run alone at that load, passed in both pre-merge full runs); the bridge lane
  filter (BridgeRecovery/BridgeClient/ShellModelBridge/Real*/ShellModel)
  35/35 green post-merge. Focused runs of the new tests passed on every run.
- Exact deadline UTC / remaining time / integration reserve: no fixed deadline
  (continuous authorization); 20% reserve kept for integration.
- ETA remaining: 0 / 0 / 0 (lane done; awaiting parent review/integration).
- Resource pool / allocation ID / maximum: Claude Max 20x on mac-m1max-a
  (shared account quota) / claude-mac20x-bridge-recovery / parent's allowance;
  quota not visible to the subagent.
- Confirmed spend / estimated usage / in-flight reservation / remaining: unknown.
- Billing evidence / freshness / unknowns: none available to a subagent.
- Child tasks and their deducted allocations: none.
- Dirty files / unpushed work / running jobs: none after this checkpoint.
- Decisions / failed approaches / linked findings: the in-flight receipt is
  deliberately routed through `reconcile` rather than re-sent, because the
  relaunched bridge holds the current (post-edit) snapshot and would refuse a
  late `capture_applied` with `revision_conflict`; reconcile reopens the
  retained pre-edit snapshot first. `fail()` keeps the exit/relaunch status
  when a request fails only because the process is gone (the request failure
  completion lands after the exit event). The real edit-ledger's store lock is
  an flock (released on SIGKILL), verified by the real test.
- Exact next action or command: none pending; parent to review/integrate
  `agent/mac-bridge-recovery/relaunch` and apply the ShellModel+Bridge diffs.
- Resume reading list: this file, `apps/mac/Sources/FlashTeXMac/BridgeSession.swift`
  (relaunch section), `apps/mac/Tests/FlashTeXMacTests/BridgeRecoveryTests.swift` §6.
- Context checkpoint (docs/context-checkpoints.md): worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a305a521269404799`,
  branch `agent/mac-bridge-recovery/relaunch`, HEAD = the commit carrying this
  file, base 92a052c. Context usage at this checkpoint is well under a quarter
  of the 1M window (no exact percentage is exposed to the subagent). Rules
  carried: no purchases/overages, transferred crates untouched, parent-retained
  Mac files untouched, user is the primary Git author on this machine, truthful
  trailers on every commit.
