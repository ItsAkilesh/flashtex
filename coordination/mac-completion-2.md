# mac-completion-2 handoff — completion pickup/keyboard latency, stale-context cancellation

- Updated UTC: 2026-09-12T12:10Z
- Agent / parent / machine alias: `mac-completion-2` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: Commander replenishment (issue #2 comment 5646989044) item 7,
  "Completion": current task = pickup/keyboard latency + stale-context
  cancellation measured best-of-N at load < 20, including the real helper
  route; follow-up 1 = keyboard-only popup traversal (Tab/Shift-Tab/Esc/
  Return/arrows) with VoiceOver labels + command table/README parity;
  follow-up 2 = `\cite{` completion from declared bibliography kinds only.
- Owned paths: `apps/mac/Sources/FlashTeXMac/Completion.swift`,
  `apps/mac/Tests/FlashTeXMacTests/CompletionTests.swift`, new
  `apps/mac/Tests/FlashTeXMacTests/CompletionLatencyTests.swift`,
  `docs/evidence/completion-*`, this handoff, `coordination/agents/mac-completion-2.json`.
  Parent-retained (diff requests only): `ShellModel.swift`,
  `ShellModel+Controller.swift`, `ContentView.swift`, `PreviewView.swift`,
  `FlashTeXMacApp.swift`, `SourceEditorView.swift`.
- Branch: `agent/mac-completion-2/latency` from `origin/agent/mac-claude-a/mac-shell`
  `cd58fc2e` (main `c11c005` merged there; `origin/main` at start `4b1850a1`, read, not merged).

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
  newer request discards): `CompletionTests.testFetcherQueriesThreeCategoriesAndRefusesStaleOrFailedReplies`.
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
  mention "completed"); `docs/evidence/` has no `completion-*` entry.

NOT covered (this lane's scope):

1. End-to-end pickup latency: keystroke → list shown (first open) and
   keystroke → list updated (narrowing), i.e. including off-main scan,
   run-loop delivery and popup update. Only the main-thread enqueue cost and
   the off-main compute were measured.
2. Keyboard latency: ↓/↑ → selection updated in the popup; Return → text
   inserted and list closed. Not measured.
3. Document switch: a pending scan for the previous buffer/caret must never
   open or mutate the list once the editor swapped documents (SourceEditorView
   replaces `tv.string` on switch); a helper `complete` reply for a query
   bound to a revision that File > Open replaced (`replaceProject` bumps
   `editorRevision`) must be refused at bind. Not explicitly tested.
4. Popup update cost: `selectCompletion` reloads the whole table on every
   arrow key; `present` re-frames/re-orders the panel on every narrowing.

## Durable checkpoint

- Branch `agent/mac-completion-2/latency`, worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a8e5d330dfc5852c1`.
- Dirty files: none at this checkpoint.
- Next commands: write `CompletionLatencyTests.swift` (best-of-N pickup/
  keyboard harness), measure baseline, optimise popup update paths in
  `Completion.swift`, re-measure, write `docs/evidence/completion-latency-2026-09-12.md`.
- Consumed main SHA: `c11c005` (via mac-shell); `origin/main` `4b1850a1` fetched, not merged.
- Staffing/billing: shared Claude Max 20x on mac-m1max-a via parent; no purchases.
