# mac-editor-a11y-3 (Claude Code subagent, parent mac-claude-a)

- Updated UTC: 2026-09-12T12:50Z
- Agent / parent / machine: mac-editor-a11y-3 / mac-claude-a / mac-m1max-a
- Task: Commander replenishment item 10 follow-ups (issue #2 comment
  5646989044), not started by mac-editor-a11y-2: (1) VoiceOver rotor landmarks
  for section headings on the editor text view, (2) reduce-motion path for the
  preview anchoring settle window and preview scroll animations, (3) one more
  whole-document selection measurement plus the one legitimate owned lever
  (bounded announcement work on large selections).
- Branch: `agent/mac-editor-a11y-3/rotor-motion` from
  `origin/agent/mac-claude-a/mac-shell` 82749c26. Worktree
  `.claude/worktrees/agent-a7aa80bdc7d51edd9`.
- Owned: new files for this lane (`EditorRotor.swift`, `ReduceMotion.swift`,
  their tests), `Completion.swift` (prior owner finished), `PreviewAnchor.swift`,
  `PreviewV2View.swift`, `SourceEditorView+LargeDocument.swift`,
  `LargeDocumentEditorTests.swift`, this handoff, `coordination/agents/mac-editor-a11y-3.json`.
  Parent-retained (diff requests only): `SourceEditorView.swift`, `ShellModel*.swift`,
  `ContentView.swift`, `PreviewView.swift`, `FlashTeXMacApp.swift`.

## Coverage audit (existing on mac-shell 82749c26, before this lane)

Grep targets: `apps/mac/Tests/FlashTeXMacTests/*`, `apps/mac/Tests/FlashTeXAccessibilityTests/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`,
`coordination/mac-editor-a11y-2.md`.

- Follow-up 1 (rotor): `Tests/FlashTeXAccessibilityTests/EditorModelTests.swift:
  testRotorCategories` covers the pure model (`AccessibleEditorModel.rotorItems(.headings)`
  labels/lines, `nextRotorItem` forward/backward with wrap, empty category → nil);
  `testModelIsDeterministic` covers equality. `grep -rn CustomRotor apps/mac`
  → no source, no test: the real `NSTextView` (`CompletingTextView`, built in
  `Completion.swift:scrollable()` — no `SourceEditorView.swift` hook needed)
  exposes no `accessibilityCustomRotors`. UNCOVERED: the AppKit rotor, its
  item search (forward/backward from a caret, from a current item, filter
  string, empty document, off-screen headings) and cache invalidation on edits.
- Follow-up 2 (reduce motion): `grep -rn "accessibilityDisplayShouldReduceMotion\|reduceMotion" apps/mac`
  → nothing. `PreviewAnchor.swift:143 settleWindow = 0.15` with a deferred
  `asyncAfter` re-application; `PreviewView.swift:52 withAnimation { proxy.scrollTo }`
  unconditional; `PreviewV2View.swift` has NO `withAnimation`/`.animation`
  site (grep), only the `PreviewAnchorKeeper` background at line 761.
  `PreviewAnchoringTests` (hosted resize / page-count tests) cover anchoring
  under the default path only. UNCOVERED: any reduce-motion behaviour and its
  injected-flag test.
- Follow-up 3 (whole-document selection): `coordination/mac-editor-a11y-2.md`
  measured 560 KB select-all 106→272 ms (AppKit `setSelectedRange` 33 ms raw,
  first `selectAll` 250 ms with the coordinator silenced), announcement
  `selectionAnnouncement` of the whole document 2.6 ms (grapheme count of
  548 k UTF-16 units), `lineColumn` at end 0.27 ms. `LargeDocumentEditorTests.
  testFiveHundredSixtyKilobyteOperations` prints select-all and shift-arrow
  figures (no assertion). The prior lane's `refreshBraceHighlight` prefilter and
  `noteTypingStep` hooks ARE applied on mac-shell (`SourceEditorView.swift:781, 911`).
  UNCOVERED: any bound on the announcement work for large selections and a
  before/after measurement of that lever.

## Checkpoint

- Updated UTC: 2026-09-12T18:34Z (resumed after quota cut ~17:25Z).
- Branch/SHA: `agent/mac-editor-a11y-3/rotor-motion` @ 7cdc5d7a (pushed):
  22bbe3c8 registration/audit, 974e765a rotor (follow-up 1, 7 tests),
  b57a0a32 merge of mac-shell 9ba9851c, 7cdc5d7a reduce motion (follow-up 2, 3 tests).
- Dirty: `SourceEditorView+LargeDocument.swift` (bounded selection announcement),
  `LargeDocumentEditorTests.swift` (+2 tests), this handoff.
- Next: run `LargeDocumentEditorTests/testWholeDocumentSelectionCostAndBoundedAnnouncement`
  when 1-min load < 20 (`FLASHTEX_NO_ACTIVATE=1 swift test --filter ...`), commit
  follow-up 3, push; apply the one-line `announceNow` hook on a local
  `…-applied` branch for the select-all after-figure; final report with the
  `PreviewView.swift` + `SourceEditorView.swift` diff requests.
- Decisions: reduce motion keeps synchronous anchor corrections, drops only the
  deferred settle-timer scroll (drift counted + anchor re-captured); the
  selection lever is the announcement grapheme walk only (highlight already
  skips non-empty selections); limit 65 536 UTF-16 units → line-span form.
- Consumed: mac-shell 9ba9851c. Resource: parent's shared Claude Max 20x; no purchases.
