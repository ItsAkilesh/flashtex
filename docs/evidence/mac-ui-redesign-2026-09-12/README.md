# mac-ui-redesign — workspace shell evidence (2026-09-12)

Lane `mac-ui-redesign` (Claude Code subagent, parent `mac-claude-a`, mac-m1max-a),
branch `agent/mac-ui-redesign/shell` on top of `agent/mac-claude-a/mac-shell` e8bbc740.
Issue #2 comment 5647841253: a visibly modern native editor UI on top of the
existing features — nothing in the model layer was replaced; every control below
calls the operation its menu item already called.

## What is new vs relocated

New (SwiftUI, macOS 14, no dependencies):

- `ContentView.swift` — `NavigationSplitView` shell: sidebar column, detail =
  editor/preview `HSplitView` + bottom Problems panel + status bar; window toolbar
  (`WorkspaceToolbar`) with labelled items whose tooltips name the menu shortcut;
  the command palette sheet; `PreviewHeader` (the former top banner, one line).
- `WorkspaceSidebar.swift` — Project (open members with kind/dirty/durable
  revision, plus unopened `\input`/`\include` targets openable in place), Outline
  (Sections / Environments / Labels disclosure groups, line numbers, click selects
  the command in the editor), Problems (error/warning counts → filtered panel).
- `DocumentOutline.swift` — bounded lexical outline scan (comments skipped,
  `document` environment excluded, 2 MiB cap) + `ShellModel.reveal(outlineItem:)`.
- `DocumentTabBar.swift` — one tab per open document, active highlight, dirty dot,
  durable revision, detach on non-entry members; switching via
  `ProjectDocuments.switchDocument`. The Project menu moved here from the old header.
- `ProblemsPanel.swift` — header (counts, All/Errors/Warnings filter, Hide) over
  the shared `DiagnosticsListView` (mac-diagnostics-3); the filter hides groups by
  severity but keeps result indices, so explanations, Fix… and occurrences are unchanged.
- `CommandPalette.swift` — View > Command Palette… (⌘⇧P) and the toolbar's
  Commands button: every `AccessibilityCommand` (README-parity tested) with menu
  and shortcut key caps; type to filter, ↑/↓, Return runs, Esc closes; editor keys
  and the preview click are listed as hints. `CommandPaletteModel.perform` is the
  single dispatcher onto existing model operations / `openWindow`.
- Status bar: editor revision, durable revision of the active document, last
  latency, route (fixture/worker/controller), error/warning counts (toggle the
  panel), navigation/stale/explanation note, capture note, exact-export progress.
- View menu items: Command Palette… (⌘⇧P), Toggle Problems (⌘⇧M).
  Pin Insertion Point moved to ⌘⌥P (README, docs, tooltips and notes updated).
  Rename Citation… joined the command table (no shortcut; menu path spelled).
- Automation hooks for evidence: `FLASHTEX_SHOW_PALETTE=1`, `FLASHTEX_WINDOW_FRAME=x,y,w,h`.

Relocated, content unchanged: `SourceEditorView`, `CaptureBar`, `BridgeBar`,
`PreviewView` / `PreviewV2Pane`, the quick-fix and proposal review sheets, the
Project menu, `DocumentKindIndicator`. `DiagnosticsListView` gained three optional
parameters (severity filter, header, max height) with unchanged defaults.

## Screenshots (packaged app, `screencapture -l <window id>`, `FLASHTEX_NO_ACTIVATE=1`)

Built with `apps/mac/scripts/make-app.sh --debug --helper-root <main checkout>`
(bundled `flashtex-compiler` auto-attached; `flashtex-render` bundled from the
parent's package). Seed: `fixtures/real-world/hw1/HW1.tex`.

- `hw1-workspace-1-FlashTeX.png` — sidebar (Project 1 member; Outline: 2 sections,
  7 environments with line numbers; Problems 111 errors / 8 warnings / All 119),
  document tab, toolbar, editor with diagnostic underlines, preview header
  (WORKER · flashtex-compiler · recovered · capabilities), preview pages, Problems
  panel (All/Errors/Warnings, grouped rows with recovery lines and Go to source),
  status bar (r2 · 134 ms · worker · 111/8 · note).
- `hw1-palette-*.png` — the command palette sheet over the same window
  (37 commands; key caps ⌘, ⌘O ⌘S ⌘⇧S ⌘⇧O ⌘R ⌘⇧K ⌘⇧R …; menu badges).
- `fixture-problems-1-FlashTeX.png` — protocol fixture compiled by the bundled
  compiler: empty Outline/Problems states ("No problems — The last compile reported
  no diagnostics.").
- `*-windows.txt` — the CGWindowList rows the captures were taken from (pid-scoped).

The first capture (before d3feb0b7) showed the three columns overflowing a
restored 900 pt frame clamped to the old 1100 pt minimum; the window minimum is now
1200 pt with a 1500×950 default size.

## Tests

- `CommandTableTests` (8): README ⇄ command table ⇄ menu wiring parity now also
  parses `CommandGroup(after: .sidebar)` (View) and `CitationRename.swift`; focus
  order checked per pane source file: Sidebar → Tabs → Editor → Capture bar →
  Bridge bar → Preview → Problems.
- `WorkspaceShellTests` (9): outline scan, reveal/stale refusal, switchOrNote,
  palette rows/filter/runnable set, workspace flags, View-menu entries.
- `DiagnosticsPanelTests`, `PanelAccessibilityTests`, `DiagnosticGroupAccessibilityTests` (20) unchanged and green.
- Full `swift test` with the parent's helper env: see the final report / handoff for the executed count at the final SHA.

## Known rough edges

- Toolbar shows icon-only for most items (Compile keeps its title); labels are in
  tooltips and the palette. `Label` titles appear when the user chooses
  "Icon and Text" in the toolbar customisation.
- The Outline rescans the active buffer 150 ms after edits settle; documents over
  2 MiB show an empty Outline by design.
- The command palette runs commands after the sheet closes (next run-loop turn),
  so panels open over the main window; palette rows for editor keys are hints only.
- The Problems panel keeps `DiagnosticsListView`'s fixed row layout; a resizable
  panel height (drag handle) is not implemented — View > Toggle Problems hides it.
- Sidebar rows are buttons (keyboard/VoiceOver activation) rather than a `List`
  selection; the active document is drawn selected but arrow keys do not move it.
