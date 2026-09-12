# mac-editor-a11y-2 (Claude Code subagent, parent mac-claude-a)

- Updated UTC: 2026-09-12T12:30Z
- Agent / parent / machine: mac-editor-a11y-2 / mac-claude-a / mac-m1max-a
- Task: Commander replenishment item 10 (issue #2 comment 5646989044):
  large-document editor bottlenecks (60 KB and 560 KB prose), then follow-up 1
  (VoiceOver rotor landmarks for `\section` headings, read-only), then
  follow-up 2 (reduce-motion path for the preview anchoring settle window).
- Branch: `agent/mac-editor-a11y-2/large-doc` from
  `origin/agent/mac-claude-a/mac-shell` cd58fc2 (main c11c005 merged; main tip
  seen 3377748). Worktree `.claude/worktrees/agent-a8d7196881ddd6069`.
- Owned: `apps/mac/Sources/FlashTeXAccessibility/AccessibilityViews.swift`,
  `AccessibilityCommands.swift`, their tests, `Tests/FlashTeXMacTests/IMEHarness.swift`,
  new files for this lane, this handoff, `coordination/agents/mac-editor-a11y-2.json`.
  Parent-retained (diff requests only): `SourceEditorView.swift`, `ShellModel*.swift`,
  `ContentView.swift`, `PreviewView.swift`, `FlashTeXMacApp.swift`.

## Coverage audit (existing, before this lane)

Large-document work already covered on mac-shell cd58fc2:
- `SourceEditorViewTests.testLargeDocumentKeystrokeRoundTripAndCaretBytesStayCorrect`
  — 60 KB, 200 keystrokes typed at the END of the document (before
  `\end{document}`): round trip wall p50 0.11 ms, whole keystroke CPU p50 0.3 ms.
  Not covered: keystrokes near the START of a large buffer (everything after
  the caret is re-laid out), 560 KB.
- `SourceEditorViewTests.testMarksArePaintedOnlyAroundTheVisibleWindowAndFast`,
  `testWholeDocumentApplyMarksStillPaintsEverything`,
  `testHostedEditorRepaintsMarksWhenScrolled`, `testPainterTracksGapsAndEdits`
  — 60 KB, 200 marks: first paint 0.32 ms, scroll repaint 0.3 ms (windowed
  painter). Not covered: 560 KB.
- `SourceEditorViewTests.testLineColumnOnLargeBufferIsSubMillisecond`,
  `testNativeTextIsByteEqualToTheStorageAndCheapToCompare` — 60 KB
  line/column and native text conversion.
- `EditorDiagnosticsTests.testRebaseOfLargeDocumentIsFast` — 60 KB, 200
  diagnostics rebase across edits.
- `EditorDiagnosticsExplanationsTests.testMarksPlusExplanationsStayFastAfterFirstFetch`
  — 62 KB, 200 diagnostics, report+attach per keystroke ~1.1 ms.
- `PasteRecoveryTests.testLargePasteIsExactlyOneDurableEditAndThePreviewBindsToIt`
  (real preview controller + edit ledger): 500 KB paste into a 60 KB base is
  one durable edit. Not covered: selection + typing AFTER the large paste on
  the 560 KB buffer in the same session.
- `IMECompositionTests` (4 tests, real controller/compiler): composition on a
  small document. Not covered: composition inside a long line of a large buffer.
- `CompletionTests` 1 MB scan bench (completion lane).
- Selection changes (select-all, shift-arrow over long lines), keyboard
  page-down through 10k lines, and any 560 KB measurement: NOT covered anywhere
  (grep of Tests/, tools/native-validation/mac-live/reports/20260912T110944Z.md,
  docs/evidence: no select-all / pageDown / 560 KB editor figures).
- Follow-up 1: `AccessibleEditorModel.rotorItems(.headings)` (pure, tested by
  `EditorModelTests.testRotorCategories`) exists, but nothing in
  `Sources/FlashTeXMac` exposes it to AppKit (`grep CustomRotor` → none): the
  real editor has no VoiceOver rotor. Uncovered.
- Follow-up 2: `PreviewAnchor.settleWindow` 0.15 s with no reduce-motion path;
  `PreviewView.swift:52 withAnimation { proxy.scrollTo }` unconditional. Uncovered.

## Current task: large-document measurements (debug build, hosted real NSTextView + ShellModel)

`LargeDocumentEditorTests` (new): `proseDocument(bytes:)` — 56-byte prose
lines in paragraphs, `\section` every 30 paragraphs, one 6 KB line. Figures are
thread-CPU ms (wall is preemption-dominated: the 1-minute load average was
27–45 during every run, above the 20 skip threshold, so the bench was run with
`FLASHTEX_BENCH_FORCE=1`; both runs print `uptime`). Each op includes one drained
run-loop turn (coalesced announcement, SwiftUI update).

| operation | 560 KB / 10 316 lines, before → after hooks | 60 KB, before → after |
|---|---|---|
| full TextKit 1 layout (once) | 388 → 189 ms | 32 → 19 ms |
| select all (first) | 106 → 272 ms (AppKit, see below) | 12 → 9 ms |
| shift-right ×40 on the 6 KB line, p50 | 2.3 → 1.4 ms | 1.6 → 1.5 ms |
| shift-down ×40 over prose, p50 | 4.2 → 2.2 ms | 4.0 → 2.5 ms |
| shift-right extending a whole-document selection | 52 → 24 ms | 8.9 → 4.8 ms |
| page down ×434 (60 KB: ×46), p50 | 2.2 → 1.6 ms | 2.6 → 1.5 ms |
| arrow down ×100, p50 | 0.21 → 0.25 ms | 0.29 → 0.12 ms |
| IME step in the 6 KB line, p50 / max | 4.6 / 18 → 3.6 / 11.6 ms | 6.4 / 10.5 → 2.8 / 4.9 ms |
| IME commit | 13.9 → 4.6 ms | 7.7 → 3.0 ms |
| 200 marks first paint / all shifted / unchanged | 0.86 / 0.83 / 0.001 → 0.53 / 0.44 / 0.000 ms | 1.9 / 2.0 / 0.002 → 0.59 / 0.72 / 0.001 ms |
| keystroke at the START of the buffer, p50 | 6.6 → 3.1 ms | 3.4 → 1.5 ms |
| keystroke at the end of the buffer, p50 | 6.4 → 3.1 ms | 2.9 → 0.8 ms |

Breakdown run (560 KB, scratch test, since removed): `selectionAnnouncement`
of the whole document 2.6 ms (grapheme count of 548 k units); `lineColumn` at
the end 0.27 ms; `utf16.count` on a fresh String 1.28 ms (Swift builds UTF-16
breadcrumbs once per String instance); `nativeText(of:)` 1.2 ms; `caretByte`
0.012 ms; `BraceMatcher.match` 0.006 ms once breadcrumbs exist; text-storage
replace 0.26 ms; `updateActiveText` 0.06 ms; raw `setSelectedRange(whole)`
33 ms and the first `selectAll` 250 ms with the coordinator silenced
(`programmaticChanges += 1`), i.e. AppKit's own whole-selection work
(selection rects over 10 k lines), not ours; shift-down raw 0.9 ms vs 1.8 ms
silent-with-turn vs 2.0 ms announcing (announcement ≈ 0.2 ms); first
`setMarkedText` in the 6 KB line 117 ms then 0.2 ms per step, 0.7 ms max in a
short line (TextKit re-lays out the marked paragraph; AppKit-internal).

**Dominant fixable cost: the delimiter-highlight refresh.** Every keystroke ran
`BraceMatcher.match(in: currentText(of:))`, which costs one native UTF-8
conversion of the storage (1.2 ms at 560 KB) plus the UTF-16 breadcrumbs of the
fresh String (1.3 ms), and it ran twice per keystroke because AppKit posts the
selection change before `textDidChange` (the selection-change refresh saw a
stale `lastKnownText` length and converted again). Prose keystrokes are
practically never adjacent to `{}[]$`.

Fix (owned new file + 2 hunks in parent-retained `SourceEditorView.swift`):
- `apps/mac/Sources/FlashTeXMac/SourceEditorView+LargeDocument.swift`:
  `BraceMatcher.delimiterAdjacent(in: NSTextStorage?, caretUTF16:)` — O(1)
  look at the two UTF-16 units around the caret on the storage's own NSString
  (no bridge, no copy). Exact prefilter of `match` (a match implies adjacency).
- Hooks: `refreshBraceHighlight` calls `match` only when `delimiterAdjacent`;
  `shouldChangeTextIn` marks the turn as a typing step (`noteTypingStep()`) so
  the pre-`textDidChange` selection notification neither refreshes the
  highlight nor announces (announcement was already suppressed at turn end).
  Behaviour kept: highlight/announcement when the caret is next to a
  delimiter (`SourceEditorViewTests` 20/20 with hooks applied; new
  `testHighlightStillFollowsDelimitersOnALargeBuffer`).

Not fixed (not ours / AppKit): whole-document selection cost (~25–50 ms per
shift-arrow at 560 KB, 33 ms raw `setSelectedRange`) is NSTextView/TextKit 1
selection geometry; TextKit 2 would change it but the marks painter is TextKit
1 temporary attributes. The remaining ~3 ms per keystroke at 560 KB is one
`nativeText` pass (1.2 ms, needed for the binding) + storage/layout + the turn.
Through the real preview controller a keystroke on the 560 KB buffer costs
9.5 ms CPU (`testLargePaste…`): the difference (~6 ms) is the controller edit
submission for a 560 KB document (ShellModel+Controller, parent-retained;
not measured further inside the bound).

Durable evidence (real preview controller + compiler,
`testLargePasteSelectionAndTypingEndDurableThroughTheRealHelper`): a 500 KB
paste through the hosted NSTextView (`insertText`, as `paste:` does) into a
60 KB document is one editor revision and one helper revision (r1 → r2,
durable after 991 ms, byte-identical to the editor); select-all and a
shift-arrow produce no revision; three keystrokes at the start of the 560 KB
buffer reach the helper byte-for-byte (`Zé→The quick brown fox`, sha256 equal
to the editor text) in 1–3 grouped helper revisions, durable after 1 276 ms
(load average 76 at that moment).

## Tests

- `apps/mac/Tests/FlashTeXMacTests/LargeDocumentEditorTests.swift` (5 tests):
  `testSixtyKilobyteOperations`, `testFiveHundredSixtyKilobyteOperations`
  (skip when 1-min load > 20 unless `FLASHTEX_BENCH_FORCE`; assert only
  the unchanged-marks budget and correctness — the printed figures are the
  evidence), `testDelimiterAdjacentIsAnExactPrefilterOfTheMatcher`,
  `testHighlightStillFollowsDelimitersOnALargeBuffer`,
  `testLargePasteSelectionAndTypingEndDurableThroughTheRealHelper` (XCTSkip
  without `FLASHTEX_PREVIEW_CONTROLLER`/`FLASHTEX_COMPILER`).
- Runs: with hooks applied — `LargeDocumentEditorTests` 4/4 (forced, load 45)
  + durable test 1/1 (load 76); `SourceEditorViewTests` 20/20. Full
  `swift test` NOT run (1-minute load 22–76 throughout, above the 15 bound).
- `swift build --build-tests` clean (pre-existing DocumentKindsTests warnings only).

## Follow-ups 1 and 2: not started (bound reached)

- Follow-up 1 (rotor landmarks): `AccessibleEditorModel.rotorItems(.headings)`
  exists and is tested; the missing piece is an `NSAccessibilityCustomRotor`
  on the editor text view (`accessibilityCustomRotors`) whose item search
  delegate maps `rotorItems(.headings)` to `NSAccessibilityCustomRotor.ItemResult`
  with the text view as element and `targetRange` = heading UTF-16 range;
  read-only (moving the rotor selects the range through `setSelectedRange`).
  Needs a hook in `makeNSView` (parent-retained) or the `CompletingTextView`
  subclass override; not written.
- Follow-up 2 (reduce motion): `PreviewAnchor.settleWindow` (0.15 s) and
  `PreviewView.swift:52 withAnimation { proxy.scrollTo }` have no
  `NSWorkspace.shared.accessibilityDisplayShouldReduceMotion` path; not written.

## Checkpoint

- Branch/SHA: `agent/mac-editor-a11y-2/large-doc` (lane files, no
  parent-retained edits) and `agent/mac-editor-a11y-2/large-doc-applied`
  (= lane + the SourceEditorView hooks, used for every measurement above).
- Dirty: none after the commits. Consumed mac-shell cd58fc2 / main c11c005.
- Next: parent applies the two hunks (diff below), merges, re-runs
  `swift test` on a calm machine (load < 15) with the four helper env vars.
- Resource: shared Claude Max 20x quota of parent; no purchases. Load 8→76.

## Diff for the parent-retained file (applied on the `-applied` branch only)

```diff
--- a/apps/mac/Sources/FlashTeXMac/SourceEditorView.swift
+++ b/apps/mac/Sources/FlashTeXMac/SourceEditorView.swift
@@ -776,7 +776,10 @@ struct SourceEditorView: NSViewRepresentable {
                 return false // nothing changes: the caret stepped over the closer
             }
             shiftPendingClosers(edit: range, replacementLength: replacementLength)
-            if !pairing, programmaticChanges == 0 { lastEdit = replacementString.map { (range, $0) } }
+            if !pairing, programmaticChanges == 0 {
+                lastEdit = replacementString.map { (range, $0) }
+                noteTypingStep() // the selection change AppKit posts before textDidChange is a typing step: no highlight refresh, no announcement
+            }
             return true
         }
 
@@ -900,7 +903,12 @@ struct SourceEditorView: NSViewRepresentable {
         func refreshBraceHighlight(_ tv: NSTextView) {
             guard let lm = tv.layoutManager else { return }
             let caret = tv.selectedRange()
+            // O(1) look at the storage first: the native-text conversion and the
+            // UTF-16 breadcrumbs behind `match` are O(n) per fresh buffer (1.2 +
+            // 1.3 ms at 560 KB, LargeDocumentEditorTests) and a prose keystroke
+            // is almost never next to a delimiter.
             let new = caret.length == 0 && !tv.hasMarkedText()
+                && BraceMatcher.delimiterAdjacent(in: tv.textStorage, caretUTF16: caret.location)
                 ? BraceMatcher.match(in: currentText(of: tv), caretUTF16: caret.location) : nil
             guard new != braceHighlight else { return }
             let length = tv.textStorage?.length ?? 0
```
