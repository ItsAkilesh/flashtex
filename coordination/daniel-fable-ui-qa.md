# Fable UI QA — what shipped on main c7ff3f74 (HW1 demo)

Lane: `daniel-fable-ui-qa` (mac-m5pro-dq222), branch `agent/daniel-fable-ui-qa/mac`,
worktree `/Users/dqi26/ft-wt-fable-ui`, base `origin/main` @ `c7ff3f74`.
Scope: demo-focused QA of the merged editor intelligence + demo UI. mac-claude-a
owns `apps/mac`; the fix commits below are separate and cherry-pickable.

## Method

- `cd apps/mac && swift build` → `Build complete!` on `c7ff3f74` (only
  pre-existing `nonisolated(unsafe)` warnings).
- `crates/compiler` and `crates/render-pipeline` built `--release` from the same
  SHA; the app launched with
  `FLASHTEX_REPO=<wt> FLASHTEX_SEED_FILE=fixtures/real-world/hw1/HW1.tex FLASHTEX_AUTOATTACH=1 FLASHTEX_COMPILER=… .build/debug/FlashTeXMac`
  at 1400×900 (`FLASHTEX_WINDOW_FRAME`), then resized to 1000×640.
- Real screenshots via `screencapture -x -l <windowid>` (Screen Recording works
  on this machine today); mouse/scroll via a small CGEvent helper, keys via
  System Events. All under `docs/evidence/daniel-fable-ui-qa-2026-09-12/`.
- The Grok capture/review flow was inspected in code only (no paired iPhone
  here); see item 8.

## Ranked list — most demo-visible first

| # | Severity | What the audience sees | Evidence | Where | Fix |
|---|---|---|---|---|---|
| 1 | **P0** | HW1 opens to **"24 errors · 6 warnings"** in the sidebar, the panel header, the status bar, and red dots on lines 10–12, 21, 24, 25 — yet *none* of the 30 is the author's fault: every one is "not implemented / not supported by this compiler version" (`\setlength`, `\setlist`, `\pagestyle`, `\Large`, `\mathbb`×11, six packages…). Nothing separates *your LaTeX is wrong* from *FlashTeX cannot do this yet*. | `01-default.png` | `ProblemsPanel.swift:22-28`, `WorkspaceSidebar.swift:215-223`, `ContentView.swift:395-401`, `DiagnosticsPanel.swift:276-277` | Message-prefix **Gap** heuristic (spec §6.4): classify `not implemented` / `not supported by this compiler version` / `not supported in math mode` / `is unsupported` as a third category, count it as "N not implemented" in grey with `puzzlepiece.extension`, keep real errors red. Commit below. |
| 2 | **P0** | Problems rows are ~54 pt tall: title + `↳ recovery:` + `main.tex bytes 31..<55` + a **"Go to source" button on every row**. 30 diagnostics → 4½ visible; the list *does* scroll (02) but reads as overflow. Byte offsets mean nothing to an author. | `01-default.png`, `02-problems-scrolled.png` | `DiagnosticsPanel.swift:290-296`, `:313` | Show `main.tex line 3` (helper `EditorDiagnostics.lineNumber(ofByte:in:)` already exists; grouped rows already use it), `controlSize(.small)` on the trailing controls. Commit below. |
| 3 | **P1** | At 1000×640 the Problems panel keeps its 260 pt and the editor is left with ~10 lines; the preview header wraps ("provisional rendering" on two lines, capability text clipped). | `05-small-window.png` | `ContentView.swift:37-38` (cap is `height − 240`), `ContentView.swift:312,335` | Cap the panel at 40 % of the window height as well; `lineLimit(1)` on the header texts. Commit below. |
| 4 | **P1** | The preview typesets `\pagestyle{empty}` as the word **"empty"**, `\parskip{0.65em}` as **"0.6em"**, `\mathbb{Z}` as **"\mathbbZ"**, `[(a)]` literally. First thing on the page. | `01-default.png` (preview, top) | compiler recovery ("typeset the command literally and continued") — not `apps/mac` | Compiler lane: skipped preamble commands must drop their argument, not typeset it; `\mathbb` is the 11× offender. Out of this lane's scope; flagged to Commander. |
| 5 | **P1** | Two developer strips live permanently under the editor: `Pin insertion point · no insertion point pinned` and `bridge: no bridge attached` (orange). | `01-default.png` | `ContentView.swift:185-186, 225-250` | Hide `BridgeBar` unless a bridge is attached or a capture exists. Commit below. Keep `CaptureBar` — it is the demo's pin-then-review entry point. |
| 6 | **P2** | Preview header speaks engine: `WORKER · flashtex-compiler · recovered · provisional rendering · ⓘ rules-v1, font-hints-v1`. | `01-default.png` | `ContentView.swift:300-340, 350` | Tooltip-only for badge/capabilities; header should read "Preview · recovered (provisional)". Spec §4.4. Not changed here (owner's call on wording). |
| 7 | **P2** | Completion popup works (⌃Space on `\us` → `\usepackage[options]{a,b,c}`), but the row's trailing detail truncates in the **middle**: `cmd · records pac…ut not implemented`; the detail pane below also ends in `I…`. | `04-completion.png` | `SourceEditorView.swift` completion row (trailing `detail`) | `truncationMode(.tail)` + `lineLimit(1)` on the trailing text; give the row title `layoutPriority(1)`. Left for the owner: the popup lives in the syntax lane's files. |
| 8 | **P2** | Hover quick-info works (`\textbf — Command — \textbf{text}: bold.`) but the popover anchors below-left of the pointer and overlaps the sidebar when the token is at the left edge of the editor. | `03-hover.png` | `SourceEditorView.swift:611-700` (`HoverController`) | Anchor the popover to the token's glyph rect (`firstRect(forCharacterRange:)`) with `preferredEdge: .maxY`. |
| 9 | **P2** | Status bar is a log line: `r8 · 25 ms · worker · 24 · 6 · Click text in the preview to select its source range.` | `01-default.png` | `ContentView.swift:380-410` | Keep counts + latency; drop `r8`/`worker` to tooltips. Not changed here. |
| 10 | **P3** | Outline lists `Problem #1 \hfill…` (the `\newcommand` body at line 18) as a section and five `enumerate` rows as "Environments". | `01-default.png` (sidebar) | `DocumentOutline.swift` | Skip headings inside `\newcommand{…}{…}`; list environments only for theorem/figure/table. Not changed here. |
| 11 | **P3** | Syntax colours (dark): commands magenta, arguments lavender, `\begin{…}` blue, `$…$` pink — readable, but dotted diagnostic underlines on 20 of the first 25 lines drown the colouring. Fixed by #1 for gaps (no red on gaps). Light mode not captured (system is dark; `-NSRequiresAquaSystemAppearance YES` is the way to check). | `01-default.png` | `SyntaxHighlighter.swift`, `SourceEditorView.swift:375-391` | Gap marks: grey underline colour (commit below covers the list/gutter; underline colour follows the same predicate). |

### Grok capture / review flow (code inspection, item 8 of the brief)

`ContentView.swift:207-218`: the capture bar shows a **"Review N proposals"** button
only after a proposal arrives; `ProposalReviewSheet` (`:450+`) renders the raw
LaTeX in an editable `TextEditor` with a preview thumbnail and requires an
explicit Insert click — nothing is inserted automatically. That matches the
Commander's standing demo rule. Weakness (spec §2.1 item 7): the sheet is a
fixed 520 pt column with a 120 pt-wide preview thumbnail; the most consequential
decision in the demo gets the smallest surface. Not changed here.

### Verified working

- Problems list scrolls (`02-problems-scrolled.png`, 30 rows, `List`).
- `⌃Space` completion popup with kind glyph, signature and doc line.
- Hover quick-info on commands.
- Line-number gutter with per-line severity dots; current-line highlight.
- Sidebar Problems rows filter the panel; segmented All/Errors/Warnings.

## Fix commits (cherry-pick onto any `apps/mac` branch)

Filled in below as each lands; each builds with `swift build` and passes the
named tests.

| SHA | Item | Files | Tests |
|---|---|---|---|
| _pending_ | | | |
