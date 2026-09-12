# mac-editor-accessibility (Claude Code subagent, parent mac-claude-a)

- Updated UTC: 2026-09-12T06:05Z
- Agent / parent / machine alias: mac-editor-accessibility / mac-claude-a / mac-m1max-a
- Task / acceptance gate / owned paths: lane "Responsive native editor
  accessibility and selection semantics" plus its follow-ups "Keyboard/edit
  undo integration through durable core" and "Large document interaction
  benchmark with correct source ranges". Owned:
  `apps/mac/Sources/FlashTeXMac/SourceEditorView.swift`,
  `apps/mac/Tests/FlashTeXMacTests/SourceEditorViewTests.swift`, this handoff,
  `coordination/agents/mac-editor-accessibility.json`. Parent retains
  ShellModel*/ContentView/PreviewView/FlashTeXMacApp (no diff needed there,
  see below). Transferred crates (font-engine, paragraph-layout, math-layout)
  untouched.
- Branch / code revision / main integrated through:
  refill task on `agent/mac-editor-accessibility/ime`, based on
  `origin/agent/mac-claude-a/mac-shell` 40d53b7 (my first lane,
  `agent/mac-editor-accessibility/responsive` 1643f86, is merged there).
  Worktree: `.claude/worktrees/agent-ac192954cb317fe5e`.
- State: ready for integration (parent review). Lane items and the refill
  task (input-method correctness) done; one diff needed in Completion.swift
  (other lane's file) is listed below, not applied.

## Refill: input-method correctness (marked text)

Observed AppKit behaviour (hosted real `NSTextView`, `NSTextInputClient` calls):
`setMarkedText` posts no `textDidChange`, only selection changes (one or two
per step); `insertText` over marked text and `unmarkText` with marked text
left post one text change; `setMarkedText("")` (IME cancel) removes the
composition silently. A synthesized `NSEvent` cannot drive a real input
source in a test process (an Option-e key event inserts its `characters`
literally: no dead-key state, no candidate window), so the sequences are
replayed at the client API — the path the input context takes.

`SourceEditorView` now:
- pushes nothing to the binding while marked text exists (a text change that
  leaves marked text behind is held back), so no revision and no compile per
  composition step; the commit is one text change, one revision, one compile
  (`testCompositionReachesTheModelOnlyWhenCommitted`, fake worker: recorder
  sees exactly the committed revision compiled);
- reports the caret at the composition start (a position of the model's
  text) during composition, so `caretByte` stays exact; composition steps are
  never announced; composing counts as typing for the navigation guard, and
  the view is not re-synced from the model (text/marks/selection) until the
  composition ends — a navigation issued mid-composition applies after the
  commit and the typing pause (or is dropped by the model when a compile
  result lands, its own rule);
- closes the completion list and cancels its pending scan at the head of the
  next run-loop turn when a composition step is observed (after the keystroke
  that started it has enqueued the list's re-scan), through the public
  `CompletingTextView.scheduler.cancel()` / `close(.textChanged)`; IME cancel
  (`setMarkedText("")` + `unmarkText`) leaves the buffer, model and revision
  untouched and the list closed; a dead-key sequence under the open list
  (mark "´", replace with "é") is not swallowed: the model sees the accent
  once (`testCompositionCancelDropsTheStepsAndClosesTheCompletionList`);
- byte offsets across composed characters: e + U+0301 (2 UTF-16 units,
  3 bytes) vs precomposed é (1 unit, 2 bytes) vs the dead-key composed é all
  give exact `caretByte`, the reverse `nsRange(utf8Bytes:)` mapping, one
  column, one announced character, and a cluster-aligned mark range; the
  decomposed bytes are kept as typed (Swift `==` calls them equal, the model
  compares bytes) (`testCaretBytesAreExactAcrossComposedAndDecomposedCharacters`).

Not drivable here: real IME candidate windows, dead-key state machines and
VoiceOver output need an input source / Accessibility permission this test
process does not have; `interpretKeyEvents` with synthesized events was
tried and inserts the literal characters.

## Ready behavior and evidence

`SourceEditorView` (NSTextView wrapper):
- VoiceOver: label "LaTeX source", help text, native value / selected-text
  range / role `.textArea` asserted through the NSAccessibility protocol in a
  hosted window. Every caret/selection change that is not a typing step
  announces `Line L, column C` or `Selected N characters, line a column b to
  [line c] column d` (1-based; columns count grapheme clusters; `\n`, `\r`,
  `\r\n` breaks; newline counting uses `memchr` over the native UTF-8 storage,
  0.0x ms at the end of 60 KB), coalesced to one low-priority announcement per
  run-loop turn, only while the editor is first responder. Navigation
  selections and applied captures announce immediately ("Inserted capture. …").
- Marks: `MarkPainter` paints diagnostic underlines only over a window around
  the visible text (visible chars ± 4 000), keeps a sorted list of disjoint
  painted ranges, paints only the uncovered gaps of a new window on scroll
  (clip-view bounds notification), grows the ranges across edits
  (`shouldChangeTextIn` delegate, over-approximation), and clears exactly what
  it painted when marks change. TextKit 1 is selected up front in `makeNSView`
  (no mid-session TextKit 2 → 1 switch when the first mark arrives).
  Measured (60 KB, 200 marks, debug build, laid out; CPU/wall): first paint
  0.32–0.35 ms, unchanged 0.0006 ms, all 200 shifted 0.44–0.50 ms, scroll to the
  end 0.30–0.34 ms. The previous whole-document pass was 1.4–2.5 ms.
- Navigation vs typing: a navigation selection is deferred (timer on common
  run-loop modes) while the view has marked (IME) text, and while the user
  typed within the last 350 ms it is deferred instead of moving the caret
  backwards; it applies as soon as typing pauses, only if still the newest
  token; a navigation forward of the caret applies at once. A recreated
  coordinator treats the model's current selection token as already applied
  (a re-created view used to replay the last navigation and jump the caret
  back). `scrollRangeToVisible` (minimal scroll) is kept.
- Capture insertion = one undo step: `breakUndoCoalescing` on both sides,
  `shouldChangeText`/`didChangeText`, action name "Insert Capture". The
  binding is no longer written from inside the SwiftUI view update; the model
  is told exactly once through `onEditApplied` (next main-queue turn) and the
  view refuses to reset from the stale binding until that delivery ran.
- Undo through the durable core (hosted window, fake bridge + fake edit
  ledger, `testCaptureUndoRedoReachesTheDurableDocumentInOrder`): typing in
  the editor reaches the durable document; pin via the editor's own caret
  report (UTF-16 12 → byte 13); approve → ledger commit first
  (`transactionTrace == ["ledger"]`) → the hosted view adopts the edit as one
  undo step → receipt → confirmed with no document_edit; ⌘Z removes exactly
  the capture and the durable text/revision follow through document_edit;
  ⇧⌘Z restores it as an ordinary edit (no second receipt, `editApplied`
  delivered once); two undos peel capture then typed word in order; the
  tombstone survives (re-approval is `.duplicate`). Model-level ordering also
  covered by `testCaptureInsertionIsOneUndoStepDeliveredToTheModelOnce`.
- Native text: `SourceEditorView.nativeText(of:)` converts the storage with
  one `NSString.getBytes` pass (~0.17 ms for 60 KB) into a contiguous UTF-8
  `String` for the binding. Finding: `tv.string` is a lazy NSString bridge and
  every byte-wise use downstream (`ShellModel.updateActiveText` → `sameBytes`,
  `utf8ByteRange`, `==` in `BridgeSession.edited`) re-transcoded the whole
  buffer: measured 2.1–2.7 ms per `sameBytes`, 3.5–13.9 ms per `==`, 4.2 ms per
  keystroke for the textDidChange → binding round trip on a 60 KB non-ASCII
  document; 0.11 ms p50 / 0.20–0.40 ms max after.
- Large-document benchmark (hosted window, real ShellModel, 60 KB, 200
  keystrokes mixing ASCII, 2/3/4-byte scalars, a ZWJ emoji and newlines, one
  warm-up keystroke first): textDidChange → binding round trip wall p50
  0.11 ms, p99 0.18–0.32 ms, max 0.20–0.40 ms; whole keystroke CPU (storage
  edit + layout + delegate + binding + model) p50 0.30 ms, max 1.4 ms (first
  emoji: font fallback). Assertion: each keystroke < 1 ms (wall round trip, or
  whole-keystroke CPU when wall was preempted — this machine runs several
  agents' builds), median < 0.5 ms. After each keystroke
  `model.caretUTF16 == selectedRange.location`, `model.caretByte` equals the
  independently computed UTF-8 offset, and the byte offset maps back to the
  same UTF-16 caret through `nsRange(utf8Bytes:)`.
  One-time cost observed: the first edit of a fresh text view costs ~7 ms CPU
  (AppKit setup + layout), not repeated; the round trip never includes it.

Validation: `swift test` in apps/mac with the four real worker binaries
(FLASHTEX_COMPILER/PDF/BRIDGE/EDIT_LEDGER) on the ime branch: 384 tests,
0 failures, 12 pre-existing env-gated skips. `SourceEditorViewTests` 17/17,
repeated 6× consecutively without failure. Under a heavy concurrent load
burst (full suite at 103 s instead of 33 s) one run of the keystroke bench
exceeded the whole-keystroke CPU budget (TextKit layout inflates under
contention), and another lane's wall-clock bench (`CompletionTests`
1 MB, 42 ms vs 20 ms) failed in the same run; the keystroke budget now
compares the round trip's own CPU time (p50 0.11 ms, max 0.22 ms), which is
what the assertion is about.

## Incomplete behavior / limitations / needs from others

- Announcements are posted with `NSAccessibility.post(.announcementRequested)`;
  VoiceOver itself was not run (no Accessibility/UI-scripting permission on this
  machine), so the wording is verified by tests, not by ear.
- The typing guard is time-based (350 ms since the last user edit); a
  deliberate keyboard "Go to source" issued within 350 ms of a keystroke that
  targets text before the caret waits for the pause.
- `ShellModel.caretByte` uses `utf8ByteRange(of:)`, which snaps a caret inside a
  surrogate pair instead of returning nil (the editor never places one there;
  `SourceEditorView.caretByte` returns nil in that case). No change requested.
- The durable-core undo test uses the Python doubles (fake_bridge /
  fake_edit_ledger) like `ShellModelBridgeTests`; the Rust helper is covered by
  the existing `RealEditLedgerTests` (green), not by a hosted-window test.
- `EditorDiagnostics.Mark` values are compared with `!=` on every update
  (O(marks), string compares); with the new identity field the parent could
  compare ids only if this ever shows up.

## Diffs needed in parent-retained files

Parent-retained (ShellModel/ContentView/PreviewView/App): none.

Completion.swift (mac-completion lane, not edited): `CompletingTextView.keyDown`
never hands Esc to `super` — with no list open it calls `requestCompletion()`
and returns — so while an input method has marked text, Esc cannot cancel the
composition (the input context never sees the event) and a scan is enqueued
mid-composition; the list's key path also re-scans after every keystroke while
marked text exists. Exact minimal diff for that lane:

```
     override func keyDown(with event: NSEvent) {
+        if hasMarkedText() { super.keyDown(with: event); return } // the input method owns every key of a composition
         if event.modifierFlags.contains(.control), event.charactersIgnoringModifiers == " " {
 ...
     func requestCompletion() {
         observeStorageIfNeeded()
         let caret = selectedRange()
-        guard caret.length == 0 else { return }
+        guard caret.length == 0, !hasMarkedText() else { return }
```

With those two lines the editor-side close/cancel becomes belt and braces.

## Resources / rules

- Resource pool: shared Claude Max 20x quota of parent mac-claude-a
  (allocation alias `claude-mac20x-shared`); no purchases, no overages, no
  paid network calls. Usage figure: unknown (not readable from this session).
- Context usage at this checkpoint: about 30% of the 1M window by the
  session's own token accounting; no compaction expected. Checkpoint written
  per docs/context-checkpoints.md anyway.
- Commit identity: jay3332 primary author (Mac rule), trailers name this
  subagent and "git via Claude Code".

## Dirty files / running jobs / next action

- Dirty: none after this commit. Running jobs: none.
- Next: parent reviews/merges `agent/mac-editor-accessibility/responsive`
  into mac-shell; re-run `swift test` there. No further lane items queued.

## Resume reading list

AGENTS.md, CLAUDE.md, docs/INDEX.md, coordination/PROJECT.md,
docs/contracts/runtime-v1.md, docs/contracts/transfer-v1.md,
apps/mac/Sources/FlashTeXMac/SourceEditorView.swift,
apps/mac/Tests/FlashTeXMacTests/SourceEditorViewTests.swift, this file.
