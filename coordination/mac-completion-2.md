# mac-completion-2 handoff — completion pickup/keyboard latency, stale-context cancellation

- Updated UTC: 2026-09-12T12:42Z
- Agent / parent / machine alias: `mac-completion-2` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: Commander replenishment (issue #2 comment 5646989044) item 7,
  "Completion": current task = pickup/keyboard latency + stale-context
  cancellation measured best-of-N at load < 20, including the real helper
  route; follow-up 1 = keyboard-only popup traversal (Tab/Shift-Tab/Esc/
  Return/arrows) with VoiceOver labels + command table/README parity;
  follow-up 2 = `\cite{` completion from declared bibliography kinds only.
- State: **all three delivered and pushed** (current task + follow-up 1 +
  follow-up 2), ready for parent integration. No parent-retained file was
  changed; no diff request is needed (the fetcher's extra `snapshot` query
  goes through the parent's existing `completionFetcher.request` call).
- Owned paths changed: `apps/mac/Sources/FlashTeXMac/Completion.swift`,
  `apps/mac/Tests/FlashTeXMacTests/CompletionTests.swift`, new
  `apps/mac/Tests/FlashTeXMacTests/CompletionLatencyTests.swift`,
  `docs/evidence/completion-latency-2026-09-12.md`, this handoff,
  `coordination/agents/mac-completion-2.json`. Finished-lane files edited
  for the follow-ups: `apps/mac/Sources/FlashTeXAccessibility/{AccessibilityCommands,AccessibilityViews,CompletionAccessibility}.swift`,
  `apps/mac/Tests/FlashTeXAccessibilityTests/CommandTableTests.swift`,
  `apps/mac/README.md` (shortcut row + completion section),
  `apps/mac/Sources/FlashTeXMac/DocumentKinds.swift` (one detach-race fix in `refresh`).
- Branch / tip: `agent/mac-completion-2/latency` at `0e44bdbf` + evidence
  commit (see git); base `origin/agent/mac-claude-a/mac-shell` `cd58fc2e`
  (main `c11c005` merged there; `origin/main` `4b1850a1` fetched, not merged).

## Coverage audit (mandatory first step)

Already covered on mac-shell `cd58fc2e` (lane `mac-completion`, merged; its
handoff `coordination/mac-completion.md`):

- Stale-context cancellation at the scheduler:
  `CompletionTests.testSchedulerRefusesCancelledAndSupersededOutcomes`
  (explicit cancel, superseding request, cancel after compute before delivery).
- Stale-context cancellation at the view (caret move, programmatic text
  replacement, stale generation never opens a session, accept refused after
  text change, resign first responder):
  `CompletionTests.testTextViewCancelsOnCaretMoveTextChangeAndResign`.
- Revision binding of Rust metadata (older/newer revision refused, equal
  merges): `CompletionTests.testTextViewRefusesMetadataNotBoundToItsEditorRevision`,
  `testMetadataBindsOnlyToItsRevisionAndMergesSameRevision`,
  `testProjectIndexReplyDecodesIsBoundedAndRefusesStaleVersions`.
- Fetcher refusals (stale source versions, helper error, decode failure,
  newer request discards): `CompletionTests.testFetcherQueriesThreeCategoriesAndRefusesStaleOrFailedReplies`
  (now `…ThreeCategoriesPlusKinds…`).
- Keyboard functional behaviour (⌃Space/Esc open, ↑/↓ wrap, typing narrows,
  Delete widens, Return/Tab insert, Esc closes and a late outcome cannot
  reopen, ←/Home leave, ⌘-shortcuts close, mouse click/double-click):
  `CompletionTests.testKeyboardChoosesInsertsAndClosesThroughTheRealTextView`.
- Keystroke cost with the list open (main-thread enqueue only, best-of-5,
  load-gated `XCTSkip` > 20): `CompletionTests.testKeystrokeThroughOpenListOnDemoTexDoesNotScanOnMain`.
- Synchronous scan cost (demo.tex avg-of-50 < 2 ms; 1 MB best-of-20 < 20 ms):
  `testCandidateComputationOnDemoTexStaysUnderTwoMilliseconds`,
  `testCompletionOnOneMegabyteBufferIsFast`.
- Real helper route (`complete` label/citation/command, request → bound
  metadata latency printed, stale refusal helper side "source versions
  changed", editor side after an edit): `CompletionLiveHelperTests.testHelperVocabularyReachesThePopupBoundToTheEditorRevision`
  (env-gated on `FLASHTEX_PREVIEW_CONTROLLER` + `FLASHTEX_COMPILER`).
- Accessibility text: `FlashTeXAccessibilityTests/CompletionAccessibilityTests`
  (row labels, "n of m" announcements, NSAccessibility read-back);
  `CommandTableTests.testCommandTableMatchesREADMEShortcuts` (README parity of
  the `completion` entry "Esc / ⌃Space").
- `tools/native-validation/mac-live/reports/20260912T110944Z.md`: no
  completion pickup/keyboard measurements (only typing-bench/historical rows
  mention "completed"); `docs/evidence/` had no `completion-*` entry.

Was NOT covered, now added (this lane):

1. End-to-end pickup latency (keystroke → list shown / narrowed, incl. off-main
   scan, run-loop delivery, popup) and keyboard latency (↓ → selection,
   Return → inserted), best-of-15 with per-stage breakdown:
   `CompletionLatencyTests.testPickupNarrowArrowAndReturnLatencyBestOfN`,
   `testArrowSelectionThroughTwelveRowsAvoidsTheTableReload`; real helper
   route `\cite{` pickup best-of-10 inside the live test.
2. Document switch: `testOutcomeForThePreviousDocumentNeverOpensOrMutatesTheList`,
   `testHelperReplyAfterProjectReplacementIsRefusedAtBind`.
3. Popup update cost: selection-only update, no re-frame/re-order while on
   screen, panel prebuilt during the first scan.

## Results (see `docs/evidence/completion-latency-2026-09-12.md`)

demo.tex best of 15 at load 13–17 (bounds enforced, passed): pickup
⌃Space→list 5.5–5.7 ms, narrow 5.1–5.6 ms, ↓→selection 0.16 ms,
Return→inserted 1.2–3.0 ms; off-main scan 0.88 ms, queue wait 0.01 ms,
delivery lag 0.02 ms, present 1.5–1.8 ms, panel display cycle ~1.3–2.7 ms
(estimate). Baseline before the popup changes (load 34–38, under load):
pickup 10.5 ms with a 4.5 ms delivery lag that was the editor's pending text
layout (test artefact, now settled before timing). Real helper `\cite{`
pickup best 0.97 ms (load 17.1); edit → bound index metadata best 26 ms.

## Test evidence

- `swift build --build-tests` clean (warnings only from an unrelated test file).
- Targeted with real compiler + preview-controller built from this branch's
  crates (`cargo build --release`): 176 tests, 0 failures, 6 skips (load-gated
  timing bounds + env-gated others) across FlashTeXAccessibilityTests,
  CompletionTests (27), CompletionLatencyTests (4), CompletionLiveHelperTests
  (2), SourceEditorView, ShellModel, PreviewController, ProjectDocuments,
  DocumentKinds (8), CitationRename, EditorPreferences, IMEComposition,
  PanelAccessibility, TypingBench, EditorDiagnostics, Navigation — at load 20–36.
- Full `swift test` not run: 1-minute load stayed ≥ 19 throughout (brief:
  only below 15).

## Durable checkpoint

- Branch `agent/mac-completion-2/latency`, worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a8e5d330dfc5852c1`;
  commits `35da3aac` (audit), `6dfd0bc0` (latency), `c50f8ac0` (follow-up 1),
  `0e44bdbf` (follow-up 2), + evidence/handoff commit.
- Dirty files: none after the evidence commit.
- Next: parent merges the branch into mac-shell; if a quiet window (load < 15)
  appears, rerun `swift test` with all helpers and append the numbers.
- Consumed main SHA: `c11c005` (via mac-shell); `origin/main` `4b1850a1` fetched, not merged.
- Staffing/billing: shared Claude Max 20x on mac-m1max-a via parent; no purchases.
