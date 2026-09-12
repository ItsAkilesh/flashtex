# GH39 — search/citation `apply_group` reconciliation vs. typing (2026-09-12)

Lane `mac-search-reconcile` (Claude Code subagent of `mac-claude-a`, mac-m1max-a,
M1 Max, Xcode 26.3 / Swift 6.2.4). Branch `agent/mac-search-reconcile/gh39` from
`origin/agent/mac-claude-a/mac-shell` `30785926`. Real helpers built in the lane
worktree with `cargo build --release`: `crates/compiler`, `crates/preview-controller`
(both at the branch tip). Other helper env vars were not set; their tests skip.

## Defect (issue #39; coordination/mac-core-review.md "Reported only" #7)

`ProjectSearchClient.reconcile` (shared by the search-replace and citation-rename
Apply paths) replaced the active buffer with the helper's post-apply document after
the `apply_group` await, regardless of what the editor did during the round trip.
Every pre-send guard (buffer equals durable, nothing in flight, versions unchanged)
runs BEFORE the send; the window is the round trip itself.

## Fix (`apps/mac/Sources/FlashTeXMac/ProjectSearchPanel.swift`)

- `EditorSnapshot` (editor revision + byte-exact buffer of the path) is taken
  immediately before the send; the reply is reconciled against it only.
- `reconcile` records the helper's durable identity FIRST and unconditionally
  through the shell's existing single writer `controllerAdoptHistoryResult`
  (`ShellModel+Controller.swift`, unchanged), passing the snapshot's editor
  revision as `issuedAtEditorRevision`; when the snapshot is no longer current the
  buffer is kept, the returned document's preview is bound to the send-time editor
  revision (shown stale), and the normal `controllerSubmitEdit` path resubmits the
  newer buffer on top. The unchanged-buffer control adopts the returned text via
  `updateActiveText` exactly as before (no edit is sent: the buffer is durable).
- `controllerRequestTracked` yields the real request id so the adopted result is
  recorded as the in-flight edit its follow-up preview binds to (as the history
  panel does). No source guard was dropped; compile revision is never used as an
  editor revision. Uncertain replies still retain id + payload and `retryRetained`
  resends them byte-identically (now reconciled against a snapshot taken at the retry).

## Deterministic reproduction (`Tests/FlashTeXMacTests/Fixtures/holding_preview_controller_proxy.py`)

A byte-transparent stdio proxy in front of the real helper: every app→helper line
is forwarded unchanged (the helper applies the group immediately); from the first
`apply_group` request every helper→app line is queued in order until a release file
exists. Nothing is reordered or rewritten; only the app's view of the reply is late.
`FLASHTEX_HOLD_WIRE` optionally traces the wire (used to diagnose the retry test).

## Tests (`Tests/FlashTeXMacTests/SearchReconcileTests.swift`)

| Test | Fix reverted (`git checkout` of the two source files, rebuilt) | Fix applied |
|---|---|---|
| `testTypingDuringADelayedApplyGroupIsKeptAndResubmittedOnTop` | FAILED: line 140 (no moved-editor note), line 144 "typing during the round trip was overwritten", line 145 (buffer equals the replaced text); then the wait for durable r3 timed out (30 s, skip) | passed 0.43 s: buffer kept byte-exactly; `textByDurable[2]` = helper text; r3 = typed buffer (sha verified), preview of r3 not stale; r4 = next edit; history label present |
| `testUnchangedBufferAdoptsTheDelayedApplyGroupResultExactly` (control) | passed | passed 0.35 s: buffer = returned text, editor revision bumped, durable r2 with the returned sha, `editorRevisionByDurable[2]` = adopted revision, nothing resubmitted (still r2 after the preview), re-run search finds 0 matches |
| `testUncertainApplyGroupIsRetriedWithTheIdenticalCommandAndReconciled` | passed | passed 0.42 s: detach during the hold → `.uncertain`, retained payload identical after relaunch; retry replays (`replayed_command`), keeps the id, adopts current r3 with no extra revision; next edit durable r4 |

Runs (logs in the lane's scratchpad, summarized here):

- Fix applied, `swift test --filter SearchReconcileTests`: 3/3 passed, 0 skipped
  (load avg 21.5 at start).
- Fix reverted, same filter: 1 failed (3 assertion failures + 1 timeout skip in the
  race test), 2 passed.
- Fix applied, `--filter "ProjectSearch|CitationRename|SearchReconcile|EditHistory|PasteRecovery"`:
  52/52 passed, 0 skipped, with the real helpers (load avg 10–18).
- Full `swift test` at 6c50fd75 with the two real helpers (other helper env unset →
  their tests skip), 1-min load 8.7 at start: 651 tests, 40 skipped, 0 failures, 168 s.

## Observations for other owners (report only, nothing patched)

- While diagnosing the retry test, an ARTIFICIAL refused `apply_group` (changed
  payload under a bound id) sent right after a replayed one made the helper emit
  `update {kind: discarded, request_id}` for the replay's pending compile. That frame
  carries no `compile_revision`, and `ShellModel.handleController` releases an
  in-flight edit on `stale`/`discarded` only when `compile_revision >= durable`
  (a compile-revision-vs-durable-revision comparison); under the default
  `holdUntilPreview` policy the in-flight registration made by
  `controllerAdoptHistoryResult` then never released until the helper's next
  `failed`/`cancelled`. Not reproduced without the artificial request; the shipped
  retry test does not send one. Parent/Commander territory (`ShellModel+Controller.swift`,
  helper `discarded` frame shape).
- Semantics chosen per the brief: when typing raced the apply, the typed buffer
  (which still contains the literal) is resubmitted on top; the replacement stays
  in the ledger at r2 (undoable in the history panel) but is not rebased into the
  buffer. The per-file note says so; the panel re-runs the search afterwards.
