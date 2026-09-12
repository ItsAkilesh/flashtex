# mac-search-reconcile handoff — GH39 apply_group reconciliation vs. typing

- Updated UTC: see `coordination/agents/mac-search-reconcile.json` `updated_utc`
- Agent / parent / machine alias: `mac-search-reconcile` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: GH39 "Native search reconciliation can overwrite typing during the
  apply_group round trip" (`gh issue view 39 -R flash-tex/flashtex`;
  reproduction `coordination/mac-core-review.md` §"Reported only" #7).
- Owned paths: `apps/mac/Sources/FlashTeXMac/ProjectSearchPanel.swift`
  (`applyGroup`/`sendApplyGroup`/`reconcile`), `apps/mac/Sources/FlashTeXMac/CitationRename.swift`
  (doc comment only), `apps/mac/Tests/FlashTeXMacTests/SearchReconcileTests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/Fixtures/holding_preview_controller_proxy.py`,
  this handoff and `coordination/agents/mac-search-reconcile.json`.
  Not touched: `ShellModel.swift`, `ShellModel+Controller.swift`, `ContentView.swift`,
  `PreviewView.swift`, `FlashTeXMacApp.swift`, `SourceEditorView.swift` (parent-retained);
  no crate.
- Branch: `agent/mac-search-reconcile/gh39` from `origin/agent/mac-claude-a/mac-shell`
  `30785926`.

## Coverage audit (mandatory first step; ≤10 min)

Searched `apps/mac/Tests/FlashTeXMacTests/*`, `tools/native-validation/mac-live/reports/20260912T110944Z.md`,
`docs/evidence/*` for the GH39 gap (typing during the search/citation
`apply_group` round trip). Already covered:

| Where | What it proves | Gap left |
|---|---|---|
| `ProjectSearchTests.swift` `ProjectSearchHelperTests.testPlanIsProposalOnlyAndApplyReplacesEveryMatchPerFile` | unchanged-buffer control: returned result adopted, no extra durable revision, history label | typing during the round trip is never exercised |
| `ProjectSearchTests.swift` `ProjectSearchHelperTests.testStaleProposalsAndUnsubmittedBuffersAreRefusedPerFile` | pre-send guards: local non-durable edit refused per file; moved project refused as a whole | guards are checked BEFORE the send only |
| `ProjectSearchTests.swift` `ProjectSearchHelperTests.testRetryingAnUncertainCommandReplaysExactly` | identical id+payload replay, changed payload refused (helper contract) | not through `retryRetained` + `reconcile` |
| `CitationRenameTests.swift` `testTypedRenameCoversAllThreeDocumentsAndApplyAdvancesDurableRevisionsExactly`, `testApplyGroupReplaysTheIdenticalCommandAndRefusesAChangedPayload` | same two properties for the citation lane (shared core) | same gap |
| `PasteRecoveryTests.swift` `testMultiRangeReplacementIsOneGroupWithExactText` | `controllerAdoptHistoryResult(issuedAtEditorRevision:)` adopts a group when the editor did not move | the search lane's `reconcile` does not use it; moved-editor branch untested |
| `EditHistoryTests.swift` `testEditUndoRedoRoundTripAdoptsExactDurableText`, `testIdenticalRetryAfterALostReplyIsIdempotent` | undo/redo adoption + retry through `controllerAdoptHistoryResult` | not the search/citation path |
| `tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*` | no apply_group-during-typing evidence (grep `apply_group|reconcil|GH39`: only the paste-recovery group test above and bridge divergence logs) | — |

Conclusion: the race itself (delayed `apply_group` reply + intervening typing)
has NO test; `ProjectSearchClient.reconcile` unconditionally
`updateActiveText(helperText)` after the await. Implementing only the uncovered
part: snapshot-validated reconciliation + a deterministic delayed-reply test.

## Implemented (uncovered part only)

- `apps/mac/Sources/FlashTeXMac/ProjectSearchPanel.swift`: `EditorSnapshot`
  taken before the `apply_group` send; `controllerRequestTracked` (request id);
  `reconcile` now records durable identity through
  `ShellModel.controllerAdoptHistoryResult` (existing single writer, parent
  file untouched) and keeps the buffer when the snapshot moved, binding the
  returned document's preview to the send-time editor revision and letting
  `controllerSubmitEdit` resubmit; per-file note names the kept text.
- `apps/mac/Sources/FlashTeXMac/CitationRename.swift`: doc comment only
  (shared core).
- `apps/mac/Tests/FlashTeXMacTests/Fixtures/holding_preview_controller_proxy.py`:
  transparent stdio proxy that holds every helper→app line from the first
  `apply_group` until a release file exists (optional wire trace).
- `apps/mac/Tests/FlashTeXMacTests/SearchReconcileTests.swift` (3 tests, real
  helper behind the proxy): race, control, uncertain-retry.
- `docs/evidence/mac-search-reconcile-2026-09-12.md`: before/after table,
  run summaries, observations for other owners.

## Evidence (measured)

- `swift build` and `swift build --build-tests`: OK.
- `SearchReconcileTests` with fix: 3/3 passed (0.43 / 0.35 / 0.42 s), 0 skipped.
- Same suite with the fix reverted: race test FAILED (assertions at lines
  140/144/145: buffer overwritten with the replaced text, then a 30 s timeout
  skip waiting for r3); control and retry passed unchanged.
- Related suites with fix (`ProjectSearch|CitationRename|SearchReconcile|EditHistory|PasteRecovery`):
  52/52 passed, 0 skipped, real compiler + preview-controller, load 10–18.
- Full `swift test` (two real helpers, others skip): see the final report /
  agents JSON `usage.evidence`.
- No parent-retained file changed; no hook needed (the existing
  `controllerAdoptHistoryResult` sufficed).

## Durable checkpoint

- Branch `agent/mac-search-reconcile/gh39`; consumed mac-shell `30785926`.
- Helpers: built in this worktree with `cargo build --release`
  (`crates/compiler`, `crates/preview-controller`).
- Dirty files at this checkpoint: the five paths above plus this handoff and
  `coordination/agents/mac-search-reconcile.json`; next: commit, push, report.
- Rerun: `cd apps/mac && FLASHTEX_NO_ACTIVATE=1 FLASHTEX_COMPILER=<worktree>/crates/compiler/target/release/flashtex-compiler FLASHTEX_PREVIEW_CONTROLLER=<worktree>/crates/preview-controller/target/release/flashtex-preview-controller swift test --filter SearchReconcileTests`.
- Resource: shared Claude Max quota with parent mac-claude-a; no purchases.
