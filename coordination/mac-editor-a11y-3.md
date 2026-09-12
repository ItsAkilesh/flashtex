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

## Results (final, 2026-09-12T18:40Z)

Branch `agent/mac-editor-a11y-3/rotor-motion` @ c7efc09b (pushed), on mac-shell 9ba9851c.

1. Rotor (974e765a): `EditorRotor.swift` + `Completion.swift` (+10 lines,
   `accessibilityCustomRotors()` on `CompletingTextView`): Headings (built-in
   type) and Environments rotors over `AccessibleEditorModel.rotorItems`, AppKit
   SearchParameters contract (ends when no current item, strictly after/before
   the caret VoiceOver passes, filter string, no wrap), results carry the text
   view + UTF-16 range; items cached per storage edit generation.
   `EditorRotorTests.swift`: 7 tests pass (forward/backward, from caret, empty
   document, off-screen headings, filter, cache invalidation on edit; 560 KB
   rebuild 80–130 ms once per edit, printed). No window activated.
2. Reduce motion (7cdc5d7a): `ReduceMotion.swift` (`isEnabled` reads
   `NSWorkspace.shared.accessibilityDisplayShouldReduceMotion` each use,
   `override` for tests, `animate(_:_:)` = `withAnimation` or immediate);
   `PreviewAnchor.swift` probe keeps synchronous corrections, skips the
   deferred settle-timer scroll under reduce motion (drift counted in
   `driftsLeftUncorrected`, anchor re-captured). `PreviewV2View.swift` has no
   animated site (grep). `ReduceMotionTests.swift`: 3 tests pass (hosted,
   never key). `PreviewAnchoringTests` 8 pass + 1 helper skip unchanged.
3. Selection lever (c7efc09b): `SourceEditorView+LargeDocument.swift`
   `boundedSelectionAnnouncement` (limit 65 536 UTF-16 units → exact line-span
   form). `LargeDocumentEditorTests`: +2 tests pass (unit; load-aware bench).
   Measured 560 KB, debug, FLASHTEX_BENCH_FORCE (1-min load 196–299 from other
   agents' builds — wall figures unreliable, CPU figures quoted): select-all
   through the hosted editor 35.09 ms CPU p50 (wall 245 / 109 ms in two runs);
   announcement alone 2.58 ms CPU unbounded → 0.27 ms bounded. With the
   `announceNow` hook applied locally (labelled `-applied` branch, discarded):
   coordinator announced "Selected 10316 lines, line 1 column 1 to line 10317
   column 1"; select-all CPU p50 35.09 ms — no measurable change at the
   operation level; the remaining cost is TextKit 1 selection layout.
   25/25 tests passed on the applied branch (SourceEditorViewTests 20,
   ReduceMotion 3, LargeDocument 2).

### Diff requests for parent-retained files (both compiled and tested on the applied branch)

```diff
--- a/apps/mac/Sources/FlashTeXMac/PreviewView.swift
+++ b/apps/mac/Sources/FlashTeXMac/PreviewView.swift
@@ -49,7 +49,7 @@ struct PreviewView: View {
             .onChange(of: caretPage) { _, page in
                 // Page-level only: keeps the page under the caret in view when the
                 // editor moves across pages; no scrolling within a page.
-                if let page { withAnimation { proxy.scrollTo(page, anchor: .top) } }
+                if let page { ReduceMotion.animate { proxy.scrollTo(page, anchor: .top) } }
             }
--- a/apps/mac/Sources/FlashTeXMac/SourceEditorView.swift
+++ b/apps/mac/Sources/FlashTeXMac/SourceEditorView.swift
@@ -981,7 +981,7 @@ struct SourceEditorView: NSViewRepresentable {
         func announceNow(text: String, range: NSRange, prefix: String, suffix: String = "") {
-            guard let message = SourceEditorView.selectionAnnouncement(text: text, range: range) else { return }
+            guard let message = SourceEditorView.boundedSelectionAnnouncement(text: text, range: range) else { return }
```

Limitations: no VoiceOver end-to-end run (Accessibility permission not granted);
the system reduce-motion setting itself was never toggled (injected flag only);
full `swift test` not run (1-min load never below 20 during the resumed session).

## Checkpoint

- Branch/SHA: `agent/mac-editor-a11y-3/rotor-motion` @ c7efc09b (pushed). Lane DONE.
- Dirty: this handoff, `coordination/agents/mac-editor-a11y-3.json` (final commit pending).
- Consumed: mac-shell 9ba9851c. Resource: parent's shared Claude Max 20x; no purchases.
