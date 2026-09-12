# mac-editor-accessibility (Claude Code subagent, parent mac-claude-a)

- Updated UTC: 2026-09-12T04:50Z
- Agent / parent / machine alias: mac-editor-accessibility / mac-claude-a / mac-m1max-a
- Task / acceptance gate / owned paths: lane "Responsive native editor
  accessibility and selection semantics" (+ follow-ups "Keyboard/edit undo
  integration through durable core", "Large document interaction benchmark
  with correct source ranges"). Owned: `apps/mac/Sources/FlashTeXMac/SourceEditorView.swift`,
  `apps/mac/Tests/FlashTeXMacTests/SourceEditorViewTests.swift`, this handoff,
  `coordination/agents/mac-editor-accessibility.json`. Parent retains
  ShellModel*/ContentView/PreviewView/FlashTeXMacApp (needed diffs are listed below,
  not applied). Transferred crates (font-engine, paragraph-layout, math-layout) untouched.
- Branch / code revision / main integrated through:
  `agent/mac-editor-accessibility/responsive` from
  `origin/agent/mac-claude-a/mac-shell` 6b43a3a; not merged with main yet
  (main advanced only in coordination/ dispatch commits since).
  Worktree: `.claude/worktrees/agent-ac192954cb317fe5e`.
- State: in progress (first checkpoint ready for parent review).

## Ready behavior and evidence (this checkpoint)

`SourceEditorView` (NSTextView wrapper):
- VoiceOver: label "LaTeX source", help text, native value / selected-text
  range / role `.textArea` (asserted through the NSAccessibility protocol in a
  hosted window). Every caret/selection change that is not a typing step
  announces `Line L, column C` or `Selected N characters, line a column b to
  [line c] column d` (1-based; columns count grapheme clusters; `\n`, `\r`,
  `\r\n` line breaks), coalesced to one low-priority announcement per run-loop
  turn, only while the editor is first responder. Navigation selections and
  applied captures announce immediately ("Inserted capture. …").
- Marks: `MarkPainter` paints diagnostic underlines only over a window around
  the visible text (visible chars ± 4 000), tracks disjoint painted ranges,
  paints newly exposed gaps on scroll (clip-view bounds notification), grows
  the tracked ranges across edits (`shouldChangeTextIn` delegate), and clears
  exactly what it painted when marks change. TextKit 1 is selected up front in
  `makeNSView` (no mid-session TextKit 2 → 1 switch when the first mark arrives).
  Measured (60 KB buffer, 200 marks, debug build, laid out): first paint
  0.32 ms, unchanged marks 0.0002 ms, all 200 marks shifted 0.43 ms,
  scroll to the end 0.29 ms; the previous whole-document pass was 1.4–2.5 ms.
- Navigation vs typing: a navigation selection is deferred (timer, common run
  loop modes) while the view has marked (IME) text, and while the user typed
  within the last 350 ms it is deferred instead of moving the caret backwards;
  it applies as soon as typing pauses, only if still the newest token. A
  navigation forward of the caret applies at once. A recreated coordinator
  treats the model's current selection token as already applied (previously a
  re-created view replayed the last navigation and jumped the caret back).
  `scrollRangeToVisible` (minimal scroll) is kept.
- Capture insertion = one undo step: `breakUndoCoalescing` on both sides,
  `shouldChangeText`/`didChangeText`, action name "Insert Capture". The binding
  is no longer written from inside the SwiftUI view update; the model is told
  exactly once through `onEditApplied` (next main-queue turn), and the view
  refuses to reset from the stale binding until that delivery ran. Hosted
  test: type abc → capture → type de → undo ×3 / redo ×3 peel and restore
  the three steps in order; `model.activeText` tracks every step; the pending
  edit is delivered once and never re-delivered by undo/redo.
- Native text: `SourceEditorView.nativeText(of:)` converts the storage with
  one `NSString.getBytes` pass (~0.17 ms for 60 KB) into a contiguous UTF-8
  `String` for the binding. Finding: `tv.string` is a lazy NSString bridge, and
  every byte-wise use downstream (`ShellModel.updateActiveText` → `sameBytes`,
  `utf8ByteRange`, `==`) re-transcoded the whole buffer: measured 2.1–2.7 ms
  per `sameBytes`, 3.5–13.9 ms per `==`, i.e. 4.2 ms per keystroke for the
  textDidChange → binding round trip on a 60 KB non-ASCII document before
  this change; 0.11 ms p50 / 0.20 ms max after.
- Large-document benchmark (hosted window, real ShellModel, 60 KB, 200
  keystrokes mixing ASCII, 2/3/4-byte scalars, a ZWJ emoji and newlines):
  every keystroke's textDidChange → binding round trip < 1 ms (p50 0.11 ms,
  p99 0.16 ms, max 0.20 ms, debug build); after each keystroke
  `model.caretUTF16 == selectedRange.location`, `model.caretByte` equals the
  independently computed UTF-8 offset, and the byte offset maps back to the
  same UTF-16 caret through `nsRange(utf8Bytes:)`.

Validation: `swift test` in apps/mac with the four real worker binaries:
198 tests, 0 failures (SourceEditorViewTests 13/13; EditorDiagnosticsTests,
TypingBenchTests, CompletionTests unchanged and green).

## Incomplete behavior / limitations / needs from others

- Announcements are posted with `NSAccessibility.post(.announcementRequested)`;
  VoiceOver was not run (no Accessibility/UI-scripting permission on this
  machine), so the wording is verified by tests, not by ear.
- The typing guard is time-based (350 ms since the last user edit); a
  deliberate keyboard "Go to source" issued within 350 ms of a keystroke that
  targets text before the caret waits for the pause.
- `ShellModel.caretByte` uses `utf8ByteRange(of:)`, which snaps a caret inside a
  surrogate pair instead of returning nil (the editor never places one there;
  `SourceEditorView.caretByte` returns nil in that case).
- Undo through the durable core: the ledger learns of an insertion via
  `editApplied` → `updateActiveText` → `bridgeTextChanged` (contract step 4) and
  of an undo via the same `updateActiveText` path as any edit; no
  `RealEditLedgerTests` change was needed. Not yet verified against a live
  bridge + ledger in this lane (RealBridge/RealEditLedger suites are green).

## Diffs needed in parent-retained files

None required for this checkpoint. Optional (parent's call): ShellModel could
store the native string it receives without change; nothing else to apply.

## Resources / rules

- Resource pool: shared Claude Max 20x quota of parent mac-claude-a
  (allocation alias `claude-mac20x-shared`); no purchases, no overages, no
  paid network calls. Usage figure: unknown (not readable from this session).
- Context usage at this checkpoint: well under 20% of the 1M window (the
  session reports ~235k tokens consumed of 15M budget); no compaction expected.
- Commit identity: jay3332 primary author (Mac rule), trailers name this
  subagent and "git via Claude Code".

## Dirty files / running jobs / next action

- Dirty: none after this commit. Running jobs: none.
- Next: (1) push; (2) merge origin/main at the next clean checkpoint;
  (3) evidence file for the benchmark numbers under docs/evidence if the parent
  wants one (currently printed by the tests); (4) report to parent.

## Resume reading list

AGENTS.md, CLAUDE.md, docs/INDEX.md, coordination/PROJECT.md,
docs/contracts/runtime-v1.md, apps/mac/Sources/FlashTeXMac/SourceEditorView.swift,
apps/mac/Tests/FlashTeXMacTests/SourceEditorViewTests.swift, this file.
