# Lane daniel-fable-ui-qa — live resume log (in git, log-only commits)

Resume packet. Worktree /Users/dqi26/ft-wt-fable-ui, branch agent/daniel-fable-ui-qa/mac, base origin/main c7ff3f74.
Machine mac-m5pro-dq222. Push only to that branch. Never main / agent/mac-claude-a/* / agent/mac-ui-redesign/*. No AI trailers.

## 2026-09-12T19:52Z — first entry, state so far
HEAD: 40c6bb5b  (mac: count not-implemented diagnostics as gaps, not errors)
Pushed: 06425f56 (ranked list + evidence). NOT yet pushed: 40c6bb5b (gap category), plus uncommitted:
   M apps/mac/Sources/FlashTeXMac/EditorIntelligence.swift
   M apps/mac/Sources/FlashTeXMac/SourceEditorView.swift

### Build / launch (exact)
  cd /Users/dqi26/ft-wt-fable-ui/apps/mac && swift build            # Build complete, pre-existing warnings only
  cd /Users/dqi26/ft-wt-fable-ui/crates/compiler && cargo build --release
  cd /Users/dqi26/ft-wt-fable-ui/crates/render-pipeline && cargo build --release
  cd /Users/dqi26/ft-wt-fable-ui/apps/mac && FLASHTEX_REPO=/Users/dqi26/ft-wt-fable-ui \
    FLASHTEX_SEED_FILE=/Users/dqi26/ft-wt-fable-ui/fixtures/real-world/hw1/HW1.tex FLASHTEX_AUTOATTACH=1 \
    FLASHTEX_COMPILER=/Users/dqi26/ft-wt-fable-ui/crates/compiler/target/release/flashtex-compiler \
    FLASHTEX_WINDOW_FRAME="40,40,1400,900" nohup .build/debug/FlashTeXMac > /tmp/ft-app.log 2>&1 &
  Window id via CGWindowList helper (scratchpad wid.swift); screenshot: screencapture -x -l <wid> out.png
  Mouse/scroll: CGEvent helper (scratchpad ev.swift: click/move/scroll x y); keys: osascript System Events keystroke.
  CGEvent keyboard posting does NOT work; AppleScript keystrokes do.
  Tests: swift test --filter 'DiagnosticsPanelTests|PanelAccessibilityTests|WorkspaceShellTests|EditorDiagnosticsTests'  → 34 passed.

### Screenshots (repo: docs/evidence/daniel-fable-ui-qa-2026-09-12/, committed in 06425f56)
  01-default.png   HW1 at 1400x900: sidebar "24 errors / 6 warnings", Problems panel 4.5 rows visible of 30, 54pt rows with Go to source buttons and "main.tex bytes 31..<55", red gutter dots on lines 10-12,21,24,25, preview leaks "empty", "0.6em", "\mathbbZ".
  02-problems-scrolled.png  list DOES scroll (grouped "11 places" rows).
  03-hover.png     hover quick-info works (\textbf), popover anchored below-left, overlaps sidebar.
  04-completion.png  ⌃Space on \us → popup; trailing detail truncated in the middle "records pac…ut not implemented".
  05-small-window.png  1000x640: Problems panel keeps 260pt, editor ~10 lines, preview header wraps.

### Ranked list (full version in coordination/daniel-fable-ui-qa.md, pushed)
  1 P0 gaps counted as errors ("24 errors") → FIXED in 40c6bb5b (EditorDiagnostics.isGap/counts; ProblemsPanel, WorkspaceSidebar, StatusBar, list row icon; test testNotImplementedDiagnosticsCountAsGapsNotErrors)
  2 P0 54pt rows / byte offsets / Go-to-source on every row → next commit (DiagnosticsPanel.swift:296-297,314)
  3 P1 Problems panel not capped at small windows (ContentView.swift:37-38) + preview header wraps
  4 P1 preview leaks "empty"/"0.6em"/"\mathbbZ" (compiler, not mac) — report only
  5 P1 bridge bar "no bridge attached" always visible (ContentView.swift:225) → hide unless attached/captures
  6 P2 preview header engine vocab; 7 P2 completion detail truncation; 8 P2 hover anchor; 9 P2 status bar log line; 10 P3 outline noise; 11 P3 underline flood (gap underline grey → in progress)

### In progress right now
  Commit 2 (uncommitted): gap marks → grey 4pt gutter dot (EditorIntelligence.swift LineNumberGutter.gapLines) and grey single-dot underline (SourceEditorView.swift MarkPainter.paint). Need: swift build; swift test --filter 'EditorDiagnosticsTests|SourceEditorViewTests|EditorIntelligenceTests'; commit; push.

### Next steps
  1. Finish/verify commit 2, push.
  2. Commit 3: DiagnosticsPanel row: "main.tex line N" via EditorDiagnostics.lineNumber(ofByte:in:) + model.compiledDocuments; controlSize(.small) on trailing buttons.
  3. Commit 4: ContentView.swift:37-38 cap problems height at 40% of geo height; BridgeBar hidden unless model.bridgeAttached || !model.bridgeCaptures.isEmpty; PreviewHeader lineLimit(1).
  4. Relaunch app, re-screenshot (06-after-*.png), update coordination/daniel-fable-ui-qa.md fix table with SHAs, push. Deadline 20:30Z.
## 19:52Z commit 2 bf0be455 pushed (gap gutter/underline); tests EditorDiagnostics|SourceEditorView|EditorIntelligence|SyntaxHighlighter 58 passed

## 2026-09-12T19:53Z — log moved into git per relayed user instruction
HEAD bf0be455 pushed. Fix commits so far: 40c6bb5b (gap counts), bf0be455 (gap gutter/underline).
Uncommitted: DiagnosticsPanel.swift (row "line N" + controlSize(.small)) — commit 3, building next.

## 2026-09-12T19:56Z — STOP (quota 98%, per daniel-parent)
Fix commits on agent/daniel-fable-ui-qa/mac (each swift build + named tests green; cherry-pickable):
  40c6bb5b gap category counts (sidebar/panel/status/list icon) + test
  bf0be455 grey gutter dot + grey underline for gaps
  7937e371 Problems rows "main.tex line N" + controlSize(.small)
  2aa98243 Problems panel capped at 40% window height; idle bridge line hidden; header no-wrap
  166e9b4c "not supported in the document preamble" is a gap (HW1 → 0 errors / 30 not implemented)
  6871be19 Problems rows: location beside title (two lines per row)
After-screenshots (scratchpad only, not committed): HW1 sidebar reads "5 errors / 0 warnings / 25 not implemented" before 166e9b4c; bridge strip gone; grey gutter dots on package lines.
Not done: after-screenshots into docs/evidence, fix table in coordination/daniel-fable-ui-qa.md, items 6-10 (owner's call), light-mode capture, hover anchor, completion truncation.
Next for whoever resumes: relaunch (commands above), capture 06/07 after-shots into docs/evidence/daniel-fable-ui-qa-2026-09-12/, fill the SHA table in coordination/daniel-fable-ui-qa.md.
