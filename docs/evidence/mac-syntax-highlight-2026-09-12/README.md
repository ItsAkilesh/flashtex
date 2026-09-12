# Mac editor: syntax highlighting + editor intelligence (lane mac-syntax-highlight, 2026-09-12)

Branch `agent/mac-syntax-highlight/editor` (from `agent/mac-claude-a/mac-shell` 5918282d).
Feedback addressed: "there is no syntax highlighting or intellisense or lsp-like features at all".

## Screenshots (window-id captures of the packaged app on HW1)

- `hw1-light.png` — `EditorPreferences.appearance = light`
- `hw1-dark.png` — `EditorPreferences.appearance = dark` (same run, dynamic colours, no repaint)

Visible in both: control sequences (purple), `\begin`/`\end` environment names (blue/teal),
`\usepackage`/`\documentclass` file arguments (red), `\newcommand`-defined names (`\N`, `\problem`,
declaration blue), `\mathbb` inside a definition body, braces/brackets dimmed, line-number gutter
with red (error) / orange (warning) dots per line from `editorMarkReport`, diagnostic underlines
(existing marks lane) still painted on top of the colours. The caret is not in the editor in these
captures (`FLASHTEX_NO_ACTIVATE=1`), so the current-line band is not shown.

Capture procedure:

```
apps/mac/scripts/make-app.sh --helper-root /Users/jay3332/Projects/flashtex \
  --render /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a775d6722811ee1dc/crates/render-pipeline/target/release/flashtex-render
FLASHTEX_NO_ACTIVATE=1 FLASHTEX_SEED_FILE=<repo>/fixtures/real-world/hw1/HW1.tex \
  apps/mac/build/FlashTeX.app/Contents/MacOS/FlashTeX -FlashTeX.EditorPreferences.v1.appearance light &
# window id from CGWindowListCopyWindowInfo filtered by the pid (layer 0), then
screencapture -x -o -l <windowID> hw1-light.png
```

The appearance is passed through the argument domain of `UserDefaults`, so the user's stored
preference is untouched.

## Tests

- `SyntaxHighlighterTests` (16): lexer runs on tricky inputs (nested braces in `\ref{}`, escaped
  `\$`, `%` inside math, unbalanced `$` stops at a blank line, verbatim/lstlisting/comment
  environments, `\verb`, CRLF line table, surrogate pairs and combining marks never split);
  incremental == full re-lex invariant after scripted edits and 400 random edits; load-gated
  560 KB bench (`FLASHTEX_BENCH_FORCE` to force).
- `EditorIntelligenceTests` (10): token/quick-info/definition targets, Return-key rules, completion
  row icon + documentation line, and a hosted editor: gutter installed with markers and drawn
  offscreen, ⌘-click routing + caret placement, Return through `doCommand(by:)` with undo, current
  line, hover present/dismiss.
- Existing suites after the change: SourceEditorViewTests 27, IMECompositionTests 4 (+4 skipped
  without helper env), LargeDocumentEditorTests, CompletionTests, CompletionLatencyTests,
  CompletionAccessibilityTests, EditorPreferencesTests, EditorRotorTests — 0 failures.

## Measured (debug build, load average 10–11)

```
syntax-highlight bench: 560 KB reset 105 ms cpu; keystroke edits 0.075, 0.050, 0.039, 0.028 ms cpu
(lines re-lexed [1, 3, 1, 4]); 15:14 up 1 day, 14:06, 3 users, load averages: 9.83 10.87 19.93
```

Budget from the assignment: keystroke re-highlight ≤ 2 ms CPU on the 560 KB document. The re-lex
is 0.03–0.08 ms; painting is bounded to the changed lines inside the visible window (4 000-unit
padding each side, like `MarkPainter`). The one-off `reset` (document open) lexes the whole buffer
for the line-start modes; 105 ms in a debug build — a release build is several times faster.

## Design notes

- Colours are layout-manager temporary attributes (`.foregroundColor`), never storage attributes:
  undo, the `text` binding, `EditorPreferences.apply` (font re-application) and the marks lane are
  unaffected. Nothing is painted while marked (IME) text exists; the dirty range is flushed after the
  commit.
- The line model follows the storage through `NSTextStorage.didProcessEditingNotification`
  (`editedCharacters` only), so undo, snippets and programmatic edits all keep it in sync.
- The gutter reads line numbers from the same line table (binary search per visible fragment).
