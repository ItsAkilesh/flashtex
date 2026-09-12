# CJK IME composition against the real preview controller — 2026-09-12

Lane `mac-ime-composition` (Claude Code subagent of `mac-claude-a`, mac-m1max-a,
Xcode 26.3 / Swift 6.2.4). Branch `agent/mac-ime-composition/ime` from
`origin/agent/mac-claude-a/mac-shell` `5bc3fc0f`.

Tests: `apps/mac/Tests/FlashTeXMacTests/IMECompositionTests.swift` driven by
`apps/mac/Tests/FlashTeXMacTests/IMEHarness.swift` (NSTextInputClient calls —
`setMarkedText`, `insertText`, `unmarkText` — on the real `CompletingTextView`
hosted through `SourceEditorView` in an `NSWindow` that is ordered front but never
made key; no key events, no focus, no Accessibility).

Helpers (main-checkout release builds, both zero external deps):

| binary | mtime | sha256 |
| --- | --- | --- |
| `crates/preview-controller/target/release/flashtex-preview-controller` | Sep 12 09:30 | `0fe60d5ff06ea147f803a6270b074b06594c450578ca44463afa9c351601396e` |
| `crates/compiler/target/release/flashtex-compiler` | Sep 12 09:31 | `1d615ef71e59435a45846aaf5b763d4e17589e6ce5ee1b55d4a47859c1c98238` |

Command (from `apps/mac`):

```
FLASHTEX_PREVIEW_CONTROLLER=… FLASHTEX_COMPILER=… FLASHTEX_NO_ACTIVATE=1 \
  swift test --skip-build --filter IMECompositionTests
```

Results (logs in `mac-ime-composition-2026-09-12/swift-test-run{1,2,3}.log`,
all under shared load — parent's heavy-build window, 1-min load 16–27 — not an
isolated timing result; the tests assert behaviour, not latency):

| run | `uptime` at start | result |
| --- | --- | --- |
| 1 | load 27.20 43.30 28.27 | 3/4 passed; `testCancelled…` failed on my initial assumption that a cancelled composition leaves no undo action (measured: AppKit leaves one "Typing" action) |
| 2 | load 18.01 38.17 27.30 | 4/4 passed, 2.90 s |
| 3 | load 16.73 37.57 27.16 | 4/4 passed, 2.86 s |

Between runs 1 and 2 the cancel test was changed to assert the MEASURED
behaviour: the leftover "Typing" undo action after an IME cancel is a no-op for
the buffer, the model (no revision) and the helper (durable stays r1, no edit in
flight, empty `history_status`). Without the two env vars every test XCTSkips
(verified).

## What each test establishes (real helper)

- `testMarkedTextIsNeverDurableAndTheCommitIsOneDurableEditAndOneUndoStep` (a, d):
  five Japanese steps に/にほ/にほん/にほんご/日本語 — after every step the helper's
  `document` reply is still r1 with the original text and `source_sha256`, `file_status`
  is `matches_source`, `history_status` undo stack is empty, no editor revision,
  nothing in flight, `caretUTF16` = composition start. Commit `日本語`: one revision,
  durable r2 whose `text` is byte-equal to the buffer and whose `source_sha256` equals
  `SourceDigest.sha256Hex` of the committed UTF-8, `caretByte` = start + 9,
  `file_status` `differs_from_source`, `history_status` undo = `["Source edit"]` (ONE
  entry for the whole composition). One `NSUndoManager.undo()` restores the
  original text exactly (not a kana at a time), becomes durable r3, and `canUndo`
  is then false (composition steps were never undo steps). Second composition with
  a surrogate pair よし/𠮷/𠮷野家: marked range 4 UTF-16 units, `caretByte` = start + 10,
  durable r4 hash matches.
- `testPreviewArrivingMidCompositionLeavesTheMarkedRangeCaretAndCompositionIntact` (b):
  plain typing `x ` puts an edit in flight (hold-until-preview); Korean jamo
  ㅎ/하/한 are marked while that compile runs in the background; when the preview for
  the typed revision lands (`model.result` updated, SwiftUI re-renders the hosted
  editor) `hasMarkedText` stays true and `markedRange`, `selectedRange` and the view
  string are identical to before; the helper's `document` is r2 = typed text.
  The composition continues 한ㄱ/한구/한국/한국ㅇ/한국어 and commits once: r3,
  `caretByte` = start + 9, two ledger entries (typing, composition).
- `testHelperKilledMidCompositionKeepsTheCompositionAndTheCommitIsDurableOnceAfterReattach` (c):
  pinyin steps n/ni/你/你h/你好 marked, then `kill(pid, SIGKILL)` on the pid the
  harness launched (run 3: pid 86453 → relaunched 86455). The shell observes the
  exit (`controllerStatus` "helper exited…", `controllerAttached == false`); marked
  range, caret, view string, `activeText` and `editorRevision` are unchanged;
  composing on (你好世) is still local. Explicit `attachController` (the shell has
  no automatic controller relaunch — see limitation) restores r1 from the private
  ledger, resubmits nothing (buffer == r1), the composition survives the reattach.
  Commit 你好世界: durable r2 exactly once (held for 0.5 s with nothing in flight
  or queued), `caretByte` = start + 12, `history_status` undo = `["Source edit"]`.
- `testCancelledCompositionLeavesTheHelperAndLedgerUntouched` (a'): か/かん/漢 then
  IME cancel — helper r1, ledger empty, no revision; the leftover AppKit "Typing"
  undo action is a no-op end to end.

## Limitations

- The preview controller is NOT auto-relaunched by the shell after an abnormal
  exit (only the direct worker and the bridge children are). The restart case is
  therefore kill → observed exit → explicit reattach; a user would use File > Attach.
- Compositions are replayed at the `NSTextInputClient` API, not through a real
  input source; the candidate window, key routing and `NSTextInputContext`
  bookkeeping are outside the test process (the same limitation as the existing
  `SourceEditorViewTests` marked-text cases).
- Not run: the full `swift test` (parent's heavy-build window; the parent runs the
  full suite at integration under load < 15).
