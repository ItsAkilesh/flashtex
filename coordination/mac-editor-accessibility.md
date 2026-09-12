# mac-editor-accessibility (Claude Code subagent, parent mac-claude-a)

- Updated UTC: 2026-09-12T05:20Z
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
  `agent/mac-editor-accessibility/responsive`, based on
  `origin/agent/mac-claude-a/mac-shell` d8baed6 (merged after it advanced past
  6b43a3a; `EditorDiagnostics.Mark` gained identity/resultStatus, tests adapted)
  and merged with `origin/main` 5499e41 (780f145 reviewed: no apps/mac changes).
  Worktree: `.claude/worktrees/agent-ac192954cb317fe5e`.
- State: ready for integration (parent review). All three lane items done.

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
(FLASHTEX_COMPILER/PDF/BRIDGE/EDIT_LEDGER): 320 tests, 0 failures, 5
pre-existing skips (PreviewController, DocumentFiles helper, NearbyView screenshots need env).
`SourceEditorViewTests` 14/14, repeated 3× consecutively without failure
after the CPU-time budgets; before them, wall-clock budgets flaked under
concurrent builds (max 6.9 ms wall for one preempted keystroke).

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

None. ContentView's existing `SourceEditorView(...)` call is unchanged and the
model API is used as is. Optional follow-up for the parent: nothing in
ShellModel needs to change for the native string; it simply stores what the
binding hands it.

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
