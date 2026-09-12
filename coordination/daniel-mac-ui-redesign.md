# FlashTeX Mac UI — design spec and critique

Lane: `daniel-parent` (mac-m5pro-dq222), branch `agent/daniel-parent/mac-ui-redesign`.
Role after the scope change: design authority / UX spec and review. No Swift in
this lane; the implementation lanes under `mac-claude-a` own `apps/mac`. Everything
below is written so those lanes can build against it and so the Commander can
rule on the few real decisions.

## 0. What this is based on

| Thing | Where | Note |
|---|---|---|
| Live app (what the user runs) | `origin/agent/mac-claude-a/mac-shell` @ `30c40a73` | Single-column `ContentView`: banner / split / footer. |
| Redesign lane (in flight) | `origin/agent/mac-ui-redesign/shell` @ `2bdf7876` (4 commits on `e8bbc740`) | `NavigationSplitView` sidebar, tabs, Problems panel, status bar, palette. Three real screenshots in `docs/evidence/mac-ui-redesign-2026-09-12/`. |
| Syntax-highlighting lane | not pushed at time of writing | Critiqued from the relayed description only; marked as such. |
| This worktree's `apps/mac` | base `abbe88a5` (main) — 501 commits behind mac-shell | I cite the live branch's line numbers, not this worktree's. `swift build` here: `Build complete! (10.27s)`, unchanged by this lane. |

I could not screenshot my own build (`screencapture` refused: no Screen Recording
permission for this shell). The redesign lane's screenshots are the visual evidence
and I read them closely; the file:line citations are from the branches above.

## 1. The user's brief, mapped to causes and owners

> "we need to be able to scroll through the error messages (it overflows), there is
> no syntax highlighting or intellisense or lsp-like features at all, it looks pretty
> bad"

| Named defect | Root cause (evidence) | Owner | Spec section |
|---|---|---|---|
| Error list overflows / can't scroll | **Two real bugs.** (1) v2 preview path: `PreviewV2Pane.diagnostics` is a bare `VStack`+`ForEach`, no `ScrollView`, no height cap — N diagnostics push the pages off the pane (`PreviewV2View.swift:733-750` on mac-shell). (2) v1 path: the `List` *does* scroll but is capped at `maxHeight: 180` (`DiagnosticsPanel.swift:249`) and rows have no `lineLimit`, so two verbose TeX messages fill the whole box and it *reads* as overflow. Also the top banner's `ForEach(capabilityNotes)` in an `HStack` (`ContentView.swift:76`) overflows horizontally with no recourse. The redesign lane's `ProblemsPanel` fixes (2) partially (220 pt, scrolling list) but keeps 54 pt rows and a fixed height — 4 of 119 rows visible in `hw1-workspace-1-FlashTeX.png`. | mac-ui-redesign | §6 |
| No syntax highlighting | `SourceEditorView` sets one font and nothing else (`SourceEditorView.swift:25`). | syntax lane | §8 |
| No IntelliSense / LSP | Completion exists (`Completion.swift`, Esc/⌃Space) but is invisible unless invoked; no hover, no gutter, no go-to-def for macros. | mac-ui-redesign | §8.4 |
| "Looks pretty bad" | See §2. Not one thing; a stack of small defaults. | this spec | §3–§11 |

## 2. Critique of the current direction

### 2.1 What the live app gets wrong (mac-shell `30c40a73`)

Concrete, in order of how much each contributes to "weird":

1. **Five stacked text strips, each styled differently.** Top banner (`ContentView.swift:104-105`: padding 12/6, `.bar`), editor header (`:165`: padding 8, no background), capture bar (`:266-267`: 8/4, `.bar`), bridge bar (`:300-301`: 8/3, `.bar`), footer (`:378`: 12/4, no background). Same grey caption text at five different rhythms. Nothing tells the eye which is chrome and which is content.
2. **The banner is a log line, not a status.** One `HStack` (`:55-105`) with source badge, result id, project id, revision, `status:`, error/warning counts, recovery note, `pdf: none`, `layout: legacy`, capability notes, spinner, `latency 147 ms (median…)`, historical/stale message, worker status. Developer telemetry sits at the same visual weight as "compile failed".
3. **Toolbar is a settings panel.** Three caption-labelled switches (Dark preview / Auto-compile / v2 preview) plus "Reload fixture" plus Compile (`:20-41`). Labelled toggles are not a Mac toolbar idiom, and two of the five items are developer switches.
4. **Editor and preview have no hierarchy.** `HSplitView` with two bare panes (`:13-16`); the preview desk uses `windowBackgroundColor` (`PreviewView.swift:51`), the same grey as the chrome, so the pages float in the same colour as everything else.
5. **Developer vocabulary in user chrome.** `WORKER`, `FIXTURE`, `HISTORICAL`, `bridge:`, `Not a real compile.`, `no worker attached`, `layout: legacy`, `capture_convert`, `5,126 UTF-8 bytes · 5,126 UTF-16 units` (`:162`), `anchor a1 · main.tex byte 1234 @ rev 7` (`:254`). An author does not know what a worker or a bridge is.
6. **Colour means four things.** Orange = stale preview (`:101`), dirty document (`:156`), warning (`:65`), fixture (`:112`), bridge trouble (`:285`). Purple = historical preview *and* the experimental v2 pane (`PreviewV2View.swift:764`). Red = error *and* load error *and* pairing-code expiry.
7. **The review sheet is a narrow modal.** Fixed 520 pt (`:441`), a column of caption paragraphs, `TextEditor` with `.border(.separator)`, and a shadow-page thumbnail at `maxWidth: 120` (`ProposalPreview.swift`, `ProposalPreviewView`). The most consequential decision in the app — insert AI-converted LaTeX or not — gets an inch-wide preview.
8. **Six top-level windows** (Nearby, Durable History, Accessibility Help, Settings, Find in Project, Citation Rename — `FlashTeXMacApp.swift:181-193`). Search and history are tool windows in any IDE; here they float with no relation to the document.
9. **Diagnostics live under the preview but belong to the source**, headed by a policy sentence used as a title: "Diagnostics (n) — the preview above is still shown; errors are not hidden" (`DiagnosticsPanel.swift:239`).

### 2.2 The redesign lane: right skeleton, wrong defaults

The skeleton (`NavigationSplitView` + `HSplitView` + bottom panel + status bar +
palette) is the correct Mac translation of a JetBrains layout — Xcode's shape, not
IDEA's tool-window stripes. Keep it. What follows is where it currently reads as a
generic IDE skin rather than a LaTeX editor. Cited from the screenshots and
`agent/mac-ui-redesign/shell`.

**A. Three columns starve the two things that matter.** In `hw1-workspace-1`
(1200 pt window) the sidebar takes 240, the editor ~480 and the preview ~460. The
editor wraps `\newcommand{\problem}[2]{…}` mid-argument; the preview, fit-to-width,
draws 11 pt body text at roughly 5 pt on screen — unreadable. A JetBrains layout has
one main area; this app has *two*, and the defaults treat the preview as a third
panel. Fix: sidebar collapsed by default below 1440 pt; preview zoom policy that
never drops below a readable floor (§9.2); the `1200` minimum and `1500×950` default
(`d3feb0b7`) are treating the symptom.

**B. 111 errors that are not the author's fault.** The HW1 document produces
111 "errors" and dotted underlines on nearly every line of the preamble — because
`fontenc`, `inputenc`, `geometry`, `amsmath`… are "recognised but not implemented".
The panel, the sidebar badge, the status bar and the gutter all shout 111 in red.
Nothing in the UI distinguishes *your LaTeX is wrong* from *FlashTeX cannot do this
yet*. For a from-scratch engine this is **the** LaTeX-specific design problem, and no
IDE template solves it. Spec in §6.4. Until the contract carries a category, a
message-prefix heuristic gets 90% of the value today.

**C. The outline lists macro definitions as sections.** Outline shows "Problem #1
\hfill…" at line 18 — that is the `\subsection*` *inside*
`\newcommand{\problem}[2]{…}`, not a heading; the real headings produced by
`\problem{1}{4}` are not found. "Environments: enumerate, enumerate, enumerate,
enumerate" is noise: nobody navigates by enumerate. Authors navigate by sections,
by captioned figures/tables, by labelled equations, by theorem-like environments.
Indentation is by absolute level (subsection = 2 tabs even when no section exists),
so titles truncate at 240 pt. Spec in §5.4.

**D. Problems rows are built for a 30-line C++ error list, not TeX.** 54 pt rows,
a "Go to source" button on every one of 119 rows, locations as `main.tex bytes
31..<55`, `↳ recovery:` prefixed lines. TeX errors are verbose and multi-line;
the JetBrains answer is a *list + detail* split, not taller rows. Spec in §6.

**E. The empty state is bigger than the full state.** "No problems — The last
compile reported no diagnostics." occupies 150 pt of window (`ProblemsPanel.swift:59`)
for a non-event. Collapse to one 24 pt line, or hide.

**F. Transient failures are dumped into the status bar and truncated.** `Bridge
relaunched after an abnormal exit: pending receipts not reconciled: edit…` (status
bar, right) and `bridge exited (1); not relaunched: 3 relaunches in the last
minute — Edit > Attach C…` (bridge bar) — both cut off, neither actionable, both
in orange. A status bar reports state; it does not carry sentences. Spec in §4.4.

**G. Capture and Bridge bars survived the redesign unchanged** — still two
stacked strips under the editor (`ContentView.swift:149-206` on the lane). They
are the *reviewed-proposal* surface and deserve a tool-window tab, not two rows of
caption text.

**H. Icon-only toolbar with ambiguous glyphs.** Moon, magnifier, speech bubble,
clock, share, window, triangle, ⌘. The lane's README admits labels live in
tooltips. Speech bubble and window are not guessable. Nine items is too many; §4.3
cuts it to five.

**I. Preview header still speaks engine.** `WORKER · flashtex-compiler ·
recovered · provisional rendering · ⓘ rules-v1, font-hints-v1`. The relocated
banner is still the banner.

**J. Nothing in the layout expresses source ↔ preview.** There is caret-sync
highlighting and click-to-source, but: no scroll lock, no way to see *where on the
page* the engine recovered, no gutter mark that mirrors a preview mark. The split
is two unrelated scroll views with a divider. Spec in §7 — this is where the design
judgment pays off and where the generic-IDE frame gives nothing.

### 2.3 The syntax-highlighting lane (from the description; not yet pushed)

The engineering scope (incremental ≤2 ms, IME-safe, gutter, current line,
auto-indent, `\end{}`) is right. Design risks to head off before it lands:

- **Palette clash.** Diagnostic underlines are red/orange; if any token colour is
  red/orange/yellow the gutter and the underline stop reading as *problems*. §8.1
  reserves those hues.
- **Math as tokens instead of math as a mode.** Colouring `\alpha` blue and `^`
  grey misses the point; an author needs to see *where math mode begins and ends*,
  because that is where 80% of real TeX errors live (unbalanced `$`, `\[` without
  `\]`). Tint the math *region* (§8.1), not only its tokens.
- **Gutter markers must respect §6.4 categories.** 111 red gutter dots for
  unimplemented packages would recreate defect B in the editor.
- **Light/dark parity** is a token table, not two hand-picked sets — §8.1 gives
  both columns with contrast checked against the editor background.

## 3. Principles (what "designed around LaTeX and the split editor" means)

1. **The document is the product; chrome is quiet.** The only pure-white (or, in
   dark preview, pure-dark) surface in the window is the page. Every bar is a
   material, every label is secondary until it has something to say.
2. **Two mains, not one.** Editor and preview share the window as equals by
   default; the sidebar and bottom panel are guests. A guest never makes a main
   unreadable.
3. **Errors belong to the source; evidence belongs to the preview.** A diagnostic
   is *listed* once (Problems), *marked* in the editor (gutter + underline), and
   *shown* in the preview where the engine recovered (margin mark). Same colour,
   same glyph, three places, one selection.
4. **The engine is honest about gaps.** "Not implemented by FlashTeX" is a
   category, not an error. It is grey, collapsed, and never underlined in red.
5. **Speak the author's language.** Compiler, preview, page, section, label,
   capture. Never worker, bridge, fixture, capability, revision, byte.
6. **Native first.** `.bar` materials, system semantic colours, SF Symbols,
   `NavigationSplitView`/`HSplitView`/`VSplitView`, sheets sized to their content,
   the standard sidebar toggle. JetBrains is the inspiration for *structure and
   density*, not for chrome.

## 4. Layout

### 4.1 Main window, default (≥1440 pt)

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│ ● ● ●  ⧉ │ FlashTeX — HW1.tex                    [⌘B Compile ▸] [Preview ▾]  ⚠ 3 │ ⌘ │  ← unified toolbar, 5 items
├───────────┬───────────────────────────────────────┬───────────────────────────────────┤
│ PROJECT   │ HW1.tex •  macros.tex                 │ Preview · Page 1 of 3 · Fit ▾  ☾ ⛓ │  ← 28 pt pane headers
│  HW1.tex •│───────────────────────────────────────┼───────────────────────────────────┤
│  macros   │ 12 │\section{Problem 1}               │                                   │
│           │ 13 │Let $a \mid b$ and $b \mid c$.    │   ┌─────────────────────────┐     │
│ OUTLINE   │ 14 │\begin{align}                     │   │      Problem Sheet 1    │     │
│ ▾ 1 Probl…│ 15 │  a \mid b^2 + c^2 \label{eq:1}   │   │                         │     │
│   1.1 (a) │ 16 │\end{align}                       │  ▌│  Problem 1  [4 points]  │     │  ← ▌ margin mark = recovered here
│ ▸ 2 Bonus │ 17 │Suppose that \emph{$a \mid b$.    │   │  Let a | b and b | c.   │     │
│ ▸ Figures │ 18 │            ~~~~~~~~~~~~~~~~~~~   │   │     a | b² + c²    (1)  │     │
│ ▸ Equatio…│ 19 │Then \ref{eq:1} holds.            │   │  Suppose that a | b.    │     │
│ ▸ Tables  │ 20 │                                  │   │                         │     │
│           │    │                                  │   └─────────────────────────┘     │
│ PROBLEMS  │    │                                  │                                   │
│  ⊗ 1  ⚠ 2 │    │                                  │              page 1               │
│  ▫ 7 gaps │    │                                  │                                   │
├───────────┴────┴──────────────────────────────────┴───────────────────────────────────┤
│ PROBLEMS  All  Errors  Warnings  Gaps          ⌄  │  ⊗ Missing } inserted            │  ← bottom tool area,
│ ⊗ Missing } inserted            HW1.tex  17:23   │  HW1.tex:17:23                    │     list | detail,
│ ⚠ Overfull \hbox (12.3pt)       HW1.tex  13      │  \emph{$a \mid b$.                │     VSplitView-resizable
│ ⚠ Reference `eq:2' undefined    HW1.tex  19:8    │  Recovered: closed the group at   │
│ ▸ Not implemented by FlashTeX (7)                 │  end of paragraph; text after it │
│                                                   │  is set upright.                  │
├───────────────────────────────────────────────────┴───────────────────────────────────┤
│ ✓ Compiled 147 ms   ⊗ 1  ⚠ 2  ▫ 7   Ln 17, Col 23   UTF-8   HW1.tex saved       ⌘⇧P │  ← 22 pt status bar
└───────────────────────────────────────────────────────────────────────────────────────┘
```

Column defaults: sidebar 220 (min 180, max 320), editor:preview 50:50 of the
remainder. Bottom panel default 220 (min 96, max 50% of window), hidden until the
first compile with an error, then remembered per session. Window minimum 1000 pt;
below 1440 the sidebar starts collapsed (user can open it; the choice persists).

### 4.2 Main window, narrow (1000–1200 pt) — sidebar collapsed, preview at 75% floor

```
┌───────────────────────────────────────────────────────────────┐
│ ● ● ●  ⧉ │ HW1.tex           [⌘B Compile ▸] [Preview ▾]  ⚠ 3 │
├───────────────────────────────┬───────────────────────────────┤
│ HW1.tex •                     │ Preview · 1/3 · 75% ▾   ☾ ⛓  │
│───────────────────────────────┼───────────────────────────────┤
│ 12 │\section{Problem 1}       │ ┌────────────────────────┐ ◂▸ │  ← horizontal scroll,
│ 13 │Let $a \mid b$ …          │ │   Problem Sheet 1      │    │     not smaller text
│ …  │                          │ │   Problem 1 [4 points] │    │
├────┴──────────────────────────┴─┴────────────────────────┴────┤
│ ✓ Compiled 147 ms  ⊗ 1 ⚠ 2 ▫ 7   Ln 17, Col 23   HW1.tex saved │
└───────────────────────────────────────────────────────────────┘
```

### 4.3 Toolbar (five items, `toolbarRole(.editor)`)

| Placement | Item | Why |
|---|---|---|
| navigation | system sidebar toggle | free with `NavigationSplitView` |
| principal | **Compile** (`hammer`, title shown) with a ▾ menu: Auto-compile ✓, Compile on save, Attach compiler… | the one primary action; auto-compile is a mode of it, not a separate switch |
| principal | **Preview** menu (`doc.richtext`): Source only / Split / Preview only; Dark page ✓; Zoom ▸ | replaces Dark-preview toggle and the split's implicit state |
| primary | **Problems** badge (`exclamationmark.triangle`, count) — toggles the bottom panel | the count is the affordance |
| primary | **Captures** (`camera.viewfinder`, badge when a proposal waits) — opens the Captures tab | the reviewed-proposal flow is first-class |

Removed from the toolbar: Reload fixture, v2 preview, search, history, share,
palette button. They live in menus and the palette (⌘⇧P). Fixture/v2/latency go
under a **Debug** menu that only appears with `FLASHTEX_DEBUG_MENU=1` or ⌥ held.

### 4.4 Status bar (22 pt) and notice strip

Status bar segments, left to right, each a `Label` in `.caption` with
`monospacedDigit()`, 12 pt gaps, hairline dividers only between groups:

| Segment | States | Tooltip / click |
|---|---|---|
| Compile state | `✓ Compiled 147 ms` · `◌ Compiling…` (accent) · `⊗ Failed` (red) · `✓ Recovered` (yellow dot) · `⏸ Preview stale` (secondary, `clock`) · `— No compiler` | engine name + capabilities; click = Compile |
| Problems | `⊗ 1  ⚠ 2  ▫ 7` | click toggles the panel filtered to that severity |
| Caret | `Ln 17, Col 23` (`Sel 120`) | click = Go to Line |
| Encoding | `UTF-8` | — |
| Document | `HW1.tex saved` · `HW1.tex •` · `HW1.tex conflict` | click = Save / Resolve |
| Right | `⌘⇧P` palette hint; capture state glyph when a companion is paired | — |

Everything that is a *sentence* — "Bridge relaunched after an abnormal exit…",
"Click text in the preview to select its source range", navigation refusals — moves
to a **notice strip**: a single 24 pt row between the toolbar and the panes that
appears only while it has something to say, with a `Dismiss` or an action
(`Attach…`, `Recompile`). One notice at a time, newest wins, auto-dismiss for
informational ones after 6 s, never for failures. Hints ("Click text in the
preview…") are removed; they belong in Help.

## 5. Sidebar

### 5.1 Sections
Project · Outline · Problems, as now (`WorkspaceSidebar.swift`). Problems in the
sidebar shows counts only and opens the panel; it does not list rows.

### 5.2 Rows
24 pt, `.body` 13 pt, icon `.secondary` hierarchical, trailing line number
`.caption2 .tertiary monospacedDigit`. Selection = system sidebar selection (use a
`List(selection:)`; the lane's button rows lose ↑/↓ — its README admits this).

### 5.3 Dirty marker
Title dot `•` after the name (Mac convention), no colour. Never orange.

### 5.4 Outline content — LaTeX, not symbols
Groups, in this order, each collapsible, empty groups hidden:

1. **Sections** — `\part`…`\subparagraph`, indented *relative to the shallowest
   level present*. Skip anything inside `\newcommand`/`\renewcommand`/`\def`
   bodies (brace-balanced skip). Show the number when `\section` is unstarred and
   numbering can be counted lexically ("1.2"); otherwise just the title.
2. **Figures & Tables** — `figure`/`table` environments, titled by their
   `\caption{…}` (first 60 chars), icon `photo` / `tablecells`.
3. **Equations** — `equation`/`align`/`gather`… *with a `\label`*, titled by the
   label key, icon `function`.
4. **Theorems** — `theorem`/`lemma`/`definition`/`proof`… titled by the optional
   argument or the environment name, icon `text.book.closed`.
5. **Labels** — everything else labelled, collapsed by default.

Not listed: `enumerate`, `itemize`, `center`, `document`, `quote`, `array`,
`tabular` inside a `table`. "Environments" as a category goes away.

## 6. Problems panel (the named defect)

### 6.1 Geometry and scrolling
- Bottom of the detail column, full width under editor+preview, in a `VSplitView`
  with a real drag handle. Default 220 pt, min 96, max 50% of window height,
  height remembered.
- Header 28 pt pinned; list scrolls *always* (never a fixed-height `List` inside a
  `VStack`); the v2 pane's diagnostics `VStack` is deleted — v2 diagnostics feed
  the same panel with a `Display list` category.
- **Empty**: the panel collapses to its 28 pt header reading `No problems` with a
  green `checkmark.circle`; or, if the user hid it, stays hidden. Never a 150 pt
  `ContentUnavailableView`.
- Auto-show on the first result with ≥1 error after being hidden; auto-hide never.

### 6.2 List + detail
Below ~160 pt the panel is list-only. Above it, the panel splits 55:45 into a
**list** (left) and a **detail** (right), the JetBrains Problems-view shape and
the right one for TeX's verbosity:

- **List row** (2 lines, 40 pt, no buttons): line 1 = severity glyph + message
  (single line, `.body`, middle-truncated); line 2 = `HW1.tex  17:23` in
  `.caption .secondary monospacedDigit` + a `3 places` pill for grouped rows.
  Double-click / Return = go to source (the existing `goToOccurrence`); ⌘⌥]/[ step
  within a group as today. `Go to source` and `Fix…` buttons move to the detail.
- **Detail**: full message wrapped, selectable; the source line quoted in
  monospace with the span highlighted; `Recovered:` line; the explanation
  (`ExplanationMemo`) if any; buttons `Go to Source`, `Fix…`, `Copy`. This is where
  a 6-line TeX message lives without bending the list.

### 6.3 Ordering and grouping
Order: errors, then warnings, then gaps, then display-list; within a severity,
document order (entry document first, then included files in include order).
Grouping is the existing `EditorDiagnostics.groups` (same message → one row with
`N places`); grouped rows sort by their first occurrence. Filter chips in the
header: `All · Errors · Warnings · Gaps` (segmented, small). Counts in the header
are unfiltered; the sidebar and status bar show the same three numbers.

### 6.4 Categories — the LaTeX-specific part

| Category | Meaning | Glyph | Colour | Editor | Preview |
|---|---|---|---|---|---|
| Error | the source is wrong; the engine could not do what was asked | `xmark.octagon.fill` | `systemRed` | red dotted underline + gutter dot | red margin mark where recovery happened |
| Warning | typeset, but the author should look (overfull box, undefined ref, missing citation) | `exclamationmark.triangle.fill` | `systemOrange` | orange dotted underline + gutter dot | — |
| Gap | **FlashTeX does not implement this yet** (package, command, option) | `puzzlepiece.extension` | `secondary` (grey) | *no underline*; faint grey gutter tick | grey margin mark |
| Display list | v2 pipeline validation | `rectangle.dashed` | `secondary` | — | — |

"Gap" is collapsed by default in the list (one row: `Not implemented by FlashTeX
(7)` expanding to the seven), shown as `▫ 7` in the status bar, and **not**
counted as errors anywhere. This turns HW1 from "111 errors" into "1 error, 2
warnings, 7 gaps", which is what is actually true.

**Decision needed (contract):** runtime-v1 `Diagnostic` carries `severity`,
`message`, `source`, `recovery` and no category. Two routes:

- *Heuristic, no contract change (do now):* the shell classifies by message
  prefix (`packages … recognised but not implemented`, `command … not
  implemented`, `option … ignored`) — a small table in `EditorDiagnostics`, unit
  tested against the compiler's actual strings.
- *Contract, Tier-1 (Commander rules):* add an optional `category:
  "error"|"warning"|"gap"` (or `kind`) to `Diagnostic` in `runtime-v1`, emitted by
  the compiler. Reversible (optional field, old shells ignore it), but it is a
  schema change on a published contract with other consumers. I recommend the
  heuristic today and the field when the contract next revs; I would not block the
  panel on it.

### 6.5 Linking back to the source
Selecting a row (single click) does three things without stealing focus: scrolls
the editor so the span is visible and pulses `showFindIndicator` on it (exists);
scrolls the preview to the page holding any item whose source lies inside the
span and flashes its margin mark; highlights the row. Return/double-click
additionally moves focus and the caret. Rows whose span was edited since the
compile (`staleIdentities`) show the location in strikethrough-grey and the
detail says "recompile to navigate" — never a refusal dialog.

## 7. Source ↔ preview

This is the interaction the layout exists for. Four behaviours, all in the
preview header or implicit:

1. **Caret follow (exists: `CaretSync`).** Keep the 15% accent fill + 1.5 pt
   underline. Add: when the caret item is off-screen in the preview and follow is
   on, scroll it to the vertical centre, no animation under reduce-motion.
2. **Scroll lock ⛓ (new, header toggle, default on).** Editor scroll → preview
   scroll by mapping the first visible editor line to the nearest item with a
   source span at or after it (binary search over items sorted by `startByte`;
   `caretItems` already has the inverse). Preview scroll → editor scroll by the
   symmetric mapping, only while the *preview* has the pointer (avoid feedback: the
   pane under the pointer is the master). Cheap: no new data.
3. **Click-to-source (exists).** Keep hover highlight. Add ⌘-click on a page = select
   the *whole* source span of the enclosing paragraph (the run of items with
   contiguous spans), which is what TeXShop/Skim users expect from SyncTeX.
4. **Recovery marks (new).** For each diagnostic with a `source`, every page item
   whose span lies inside it gets a 3 pt mark in the page's left margin at the
   item's baseline, colour by §6.4. Hovering the mark shows the message; clicking
   selects the Problems row. This is how "recovered: preview shown with
   provisional rendering" stops being a sentence in a banner and becomes a place on
   the page.

Editor gutter marks (syntax lane) use the same colours and the same click →
Problems row selection, so all three surfaces agree.

## 8. Editor (for the syntax-highlighting lane)

### 8.1 Highlighting palette
Chrome uses system semantic colours; the editor palette is explicit so it is
stable across appearances and never borrows a diagnostic hue. Five roles only.
Contrast is against `textBackgroundColor` (light `#FFFFFF`, dark `#1E1E1E`).

| Role | Tokens | Light | Dark | Style |
|---|---|---|---|---|
| Command | `\foo`, `\begin`, `\end` | `#1F4FB5` (6.9:1) | `#7FA7F0` (7.2:1) | regular |
| Math region | `$…$`, `\(…\)`, `\[…\]`, `equation`/`align` bodies | text `#0F6E5A` (6.4:1); **region tint** `#0F6E5A` @ 6% | text `#6FD0B4` (8.0:1); tint @ 8% | tint spans the full line height; nested commands keep Command colour |
| Reference key | `\label{k}`, `\ref{k}`, `\cite{k}`, `\eqref` argument | `#6B3FA0` (7.1:1) | `#C9A6F5` (9.1:1) | underline on hover (go-to) |
| Comment | `% …` | `#8A8A8E` (3.4:1, decorative) | `#7C7C80` | italic |
| Argument braces / plain text | `{`, `}`, `[`, `]`, prose | primary at 100% / braces at 65% | same | regular |

Reserved, never used for tokens: red, orange, yellow, green-as-status, accent
blue (`controlAccentColor` — selection and links only). Command blue is a fixed
hex, not the accent, so changing the system accent does not recolour code.

### 8.2 Gutter and lines
- Gutter: line numbers `.caption` monospaced, `tertiaryLabelColor`, current line
  number `secondaryLabelColor`; 8 pt right padding; width fits the line count.
- Diagnostic markers: 6 pt filled circle in the gutter at the line, §6.4 colours;
  gap = 2 pt tick in `tertiaryLabelColor`. One marker per line, highest severity
  wins; tooltip lists all.
- Current line: `primary` @ 4% full-width band (dark: 6%). No border.
- Underlines: keep `thick|patternDot` red/orange (exists); none for gaps.
- Selection, caret, find indicator: system.

### 8.3 Font and spacing
From `EditorPreferences` (exists): 13 pt monospaced default, line height 1.25,
`textContainerInset` 8×8 (exists), tab 4. Nothing else configurable in v1.

### 8.4 IntelliSense — what earns its place in a LaTeX editor
In priority order; the first three exist and only need to be *visible*:

1. `\ref`/`\eqref`/`\cite` key completion with the label's section title or the
   bib title as the detail line (exists: `Completion.swift`; show the popup
   automatically after `{` inside those commands, not only on ⌃Space).
2. `\begin{x}` → `\end{x}` (exists / syntax lane).
3. Command completion filtered to what this engine supports, with a `not
   implemented` detail for the rest (exists: "supported by this compiler") — the
   popup should render that in the Gap grey, not hide it.
4. **Hover on a math command shows the glyph** (`\alpha` → α, `\mathbb{Z}` → ℤ):
   tiny, LaTeX-specific, delightful. A static table; no engine call.
5. **⌘-click / hover on a user macro shows its `\newcommand` definition** and jumps
   to it. This is the one place go-to-definition pays off in TeX.
6. Hover on a diagnostic span: message + recovery + `Fix…` (exists in tooltip form).

Not worth building now: symbol rename, generic "quick info" on built-ins, a full
LSP. Popup styling: 6 pt radius, `.popover` material, rows 24 pt, kind glyph in
`secondary`, key `.body`, detail `.caption .secondary`, max 8 rows then scroll.

## 9. Preview pane

### 9.1 Header (28 pt)
`Preview · Page 2 of 3 · Fit ▾ · ☾ · ⛓`. Left: title and page; right: zoom
menu (Fit width · 75% · 100% · 125% · 150%), dark-page toggle (`moon`), scroll
lock (`link`). Engine identity, capabilities, latency and route move to the
status-bar compile segment's tooltip and the Debug menu. `WORKER`/`FIXTURE`/
`HISTORICAL` badges go; a fixture-driven preview shows `Sample` in the header
title (`Preview (sample)`); a historical frame shows `Preview · showing r7 while
r8 compiles` in `secondary` with a `clock.arrow.circlepath`.

### 9.2 Zoom policy
Default *Fit width* but with a **floor of 75%**: when fit would go below 75% the
page stays at 75% and scrolls horizontally. Readable beats fully visible. ⌘+/⌘−
step the menu, ⌘0 = Fit.

### 9.3 Desk and page
Desk: `underPageBackgroundColor` (light) / `#1F1F1F` (dark) — distinct from the
chrome so pages read as objects on a surface. Page: white / `#292929` (exists),
shadow `radius 6, y 2, opacity 0.25` (light) / `radius 8, opacity 0.6` (dark), 24 pt
page gap (exists), page number as a `caption2` label centred *below* the page
rather than inside its corner. Margin marks per §7.4.

## 10. Captures and the review sheet

Capture bar + Bridge bar collapse into one **Captures** tab in the bottom tool
area (next to Problems): a list of captures with state (`received · converting ·
ready to review · inserted · rejected`), the pinned insertion point as the first
row (`Insert at HW1.tex 17:23 — Pin here (⌘⌥P)`), and companion/bridge status as
the tab's header line with an `Attach…` action when detached. The status bar
shows one glyph (`iphone.radiowaves.left.and.right`, badge = proposals waiting).

Review sheet, resizable, 840×560 default:

```
┌────────────────────────────────────────────────────────────────────────┐
│ Review capture from iPad · 12:04                    Inserts at HW1.tex 17:23 │
├───────────────────────────────────┬────────────────────────────────────┤
│ \begin{align}                     │  ┌──────────────────────────────┐  │
│   \int_0^1 x^2\,dx = \frac{1}{3}  │  │  Problem 1  [4 points]        │  │
│ \end{align}                       │  │  Let a | b …                  │  │
│                                   │  │      ∫₀¹ x² dx = 1/3   (2)    │  │  ← shadow page, readable
│ (editable · monospaced · §8.1)    │  │  Suppose that a | b.          │  │
│                                   │  └──────────────────────────────┘  │
│ Ambiguities                       │  ✓ No new problems                 │
│  • "x^2" could be "x²" or "x_2"   │  Needs: amsmath (already loaded)   │
├───────────────────────────────────┴────────────────────────────────────┤
│ [Reject]                                        [Cancel]  [Insert  ⏎]  │
└────────────────────────────────────────────────────────────────────────┘
```

Left: the LaTeX (editable unless it is a bridge capture, in which case read-only
with a lock glyph and the reason in a tooltip). Right: the shadow page at
readable zoom with the inserted region tinted 8% accent; below it the verdict
(`✓ No new problems` / `⊗ 1 new error` in §6.4 colours) and dependencies.
"Approve and insert" → **Insert**; the SHA/revision verification sentence becomes
a tooltip on the lock. Keyboard: ⏎ inserts, ⎋ cancels, ⌘⌫ rejects.

## 11. Tokens

### 11.1 Spacing (4 pt base)
`2 · 4 · 8 · 12 · 16 · 24`. Bars: 28 pt (pane header, tab bar, panel header),
22 pt (status bar), 24 pt (notice strip). Rows: 24 (sidebar, popup), 40 (Problems
2-line), 44 (palette). Pane inner padding 8; sheet padding 16; between segments
in a bar 12; icon–label 4.

### 11.2 Type
Three UI sizes only: `.body` 13, `.caption` 11, `.caption2` 10 (line numbers,
tertiary counters). `.headline` only in sheet titles. `monospacedDigit()` on
anything that counts. Editor and code fragments: `EditorPreferences.font`.
No `.bold()` on captions for emphasis — use colour (`primary` vs `secondary`).

### 11.3 Colour semantics (chrome)

| Meaning | Token | Never also used for |
|---|---|---|
| Error | `systemRed` | load errors, pairing expiry (those are notices, `primary` text with a red glyph) |
| Warning | `systemOrange` | stale, dirty, fixture, bridge trouble |
| Gap / informational | `secondaryLabelColor` + glyph | — |
| Recovered / provisional | `systemYellow` dot only | — |
| OK / compiled | `systemGreen` glyph only | connected pills (use the glyph) |
| In progress | accent `ProgressView` | — |
| Stale / historical | `secondary` + `clock` glyph | — |
| Selection, active tab, links, caret sync | `controlAccentColor` | syntax colours |
| Experimental / debug | none — lives behind the Debug menu | purple goes away |

Materials: `.bar` for every bar; `windowBackgroundColor` for panels and sidebar
content; `textBackgroundColor` for the editor; desk per §9.3. Dividers: system
`Divider()` only, never `.border(.separator)` on content.

### 11.4 Iconography
SF Symbols, `.symbolRenderingMode(.hierarchical)`, weight regular, one size per
bar (`.caption` in 28 pt bars, default in the toolbar). Mapping: compile `hammer`,
preview `doc.richtext`, source `doc.text`, problems `exclamationmark.triangle`,
error `xmark.octagon.fill`, warning `exclamationmark.triangle.fill`, gap
`puzzlepiece.extension`, captures `camera.viewfinder`, companion
`iphone.radiowaves.left.and.right`, outline `list.bullet.indent`, section
`number`, figure `photo`, table `tablecells`, equation `function`, theorem
`text.book.closed`, label `tag`, scroll lock `link` / `link.badge.plus` off,
dark page `moon`, palette `command`, history `clock.arrow.circlepath`.

### 11.5 Focus and selection
System focus ring everywhere; no custom rings. List/sidebar selection: system.
Custom rows (tabs): active = accent @ 14% fill + 2 pt accent underline (the lane's
`DocumentTabBar` is right); hover = `primary` @ 5%. Keyboard focus order:
Sidebar → Tabs → Editor → Preview → Problems → Status bar (the capture bars leave
the order because they leave the editor column).

### 11.6 Light / dark
Follow the system; the only independent switch is the preview's dark *page*
(exists). Dark chrome uses the same tokens; the explicit hexes above are the only
places a value differs.

## 12. Vocabulary

| Now | Say |
|---|---|
| worker / WORKER | compiler (in tooltips: "flashtex-compiler 4888a67") |
| bridge | companion link / capture link |
| fixture / FIXTURE / "Not a real compile." | sample (`Preview (sample)`) |
| recovered: preview shown with provisional rendering | Recovered (glyph) — the detail says where |
| editor at revision 8 — press ⌘B to compile | Preview stale (`clock`) — tooltip "Edited since last compile · ⌘B" |
| 5,126 UTF-8 bytes · 5,126 UTF-16 units | gone (Debug menu → Document Info) |
| anchor a1 · main.tex byte 1234 @ rev 7 | Insert at HW1.tex 17:23 |
| main.tex bytes 31..<55 | HW1.tex 2:1–2:24 |
| Approve and insert | Insert |
| Pin insertion point | Pin here |
| capture_convert / Convert | Convert to LaTeX |
| layout: legacy / rules-v1, font-hints-v1 | gone (compile-state tooltip) |
| HISTORICAL | showing r7 while r8 compiles |

## 13. Sequencing for the implementation lanes

Ranked by impact per risk; all in files the redesign lane already owns, so no
coordination hazard beyond their own branch.

| # | Change | Files (on `mac-ui-redesign/shell`) | Size | Impact |
|---|---|---|---|---|
| 1 | Problems: VSplitView + resizable, empty state collapses, 2-line rows, list+detail | `ProblemsPanel.swift`, `DiagnosticsPanel.swift` | M | fixes the named bug properly |
| 2 | Gap category via message heuristic; grey glyph, collapsed group, no red underline, counts split | `EditorDiagnostics.swift` (+tests), `ProblemsPanel`, `WorkspaceSidebar`, status bar, `SourceEditorView.applyMarks` | M | HW1 goes from 111 errors to 1 |
| 3 | Delete v2 pane diagnostics VStack; feed the panel | `PreviewV2View.swift` | S | removes the hard overflow |
| 4 | Preview zoom floor + header vocabulary | `ContentView.swift` `PreviewHeader`, `PreviewView.swift` | S | readable preview at 1200 pt |
| 5 | Notice strip; sentences out of the status bar | `ContentView.swift` `StatusBar` + new `NoticeStrip.swift` | S | no truncated errors |
| 6 | Sidebar collapsed < 1440; outline per §5.4 | `ContentView.swift`, `DocumentOutline.swift`, `WorkspaceSidebar.swift` | M | outline becomes useful |
| 7 | Toolbar diet + Debug menu | `ContentView.swift` `WorkspaceToolbar`, `FlashTeXMacApp.swift` | S | five guessable items |
| 8 | Captures tab replaces capture/bridge bars | `ContentView.swift`, `CaptureList.swift` | M | review flow first-class |
| 9 | Recovery margin marks in preview | `PreviewView.swift`, `PreviewV2View.swift` | M | errors visible on the page |
| 10 | Scroll lock | `PreviewView.swift`, `SourceEditorView.swift`, `CaretSync.swift` | M | the split becomes one document |
| 11 | Review sheet two-column | `ContentView.swift` `ProposalReviewSheet`, `ProposalPreview.swift` | M | decisions made on a readable page |
| 12 | Fold Find in Project / Durable History into tool tabs | `ProjectSearchPanel.swift` (1200 lines), `EditHistoryPanel.swift` (850) | L | later; window IDs are load-bearing for tests and `FLASHTEX_OPEN_WINDOW` |
| 13 | `category` on the runtime-v1 contract | `docs/contracts/runtime-v1.md`, compiler | Tier-1 | Commander decision; #2 does not wait for it |

For the syntax lane: §8.1 palette and §8.2 gutter before landing; §6.4 gap
handling in the gutter (coordinate with #2 — a shared `DiagnosticCategory` in
`EditorDiagnostics` is the join point).

## 14. Judgment calls — overrule freely

Confident (would argue): 1, 2, 3, 4, 5 in §13; the gap category; math *region*
tinting; sentences out of the status bar; no orange for dirty/stale; readable
preview floor.

Judgment calls where I picked and you may prefer otherwise:

- **Sidebar collapsed by default below 1440.** Alternative: always open at 200 and
  accept a narrower preview. I chose the preview.
- **List + detail Problems view.** Alternative: single list with rows that wrap to
  3 lines and a disclosure per row. Simpler; worse for 6-line TeX messages.
- **Zoom floor 75%.** Could be 66% or 100%. 75% keeps 11 pt body ≈ 8 pt on screen.
- **Toolbar cut to five.** Someone may want Search and History back; the palette
  has them. Xcode ships search in the toolbar; I would still leave it out.
- **"Gap" as the word.** Alternatives: "Unsupported", "Not yet". "Gap" is short
  and honest; "Unsupported" sounds final.
- **Dark page toggle stays in the preview header** rather than following editor
  appearance. Authors proofing for print want a white page in a dark room.
- **Xcode-shape rather than JetBrains tool-window stripes.** If the user wants the
  literal IDEA look (icon stripes on the window edges), that is a different, larger
  change and un-Mac; I would push back once and then do it.
- **Scroll lock default on.** Some authors find it jumpy; it is one click off and
  remembered.

## 15. Coordination notes

- This lane changed **no Swift**. `apps/mac` in this worktree builds as it did
  (`Build complete! (10.27s)`, warnings only, pre-existing Swift 6 isolation
  notes in `TypingBench.swift`).
- Files this document asks other lanes to change are listed in §13; nothing here
  is touched by more than one lane except `EditorDiagnostics.swift` (redesign lane
  for #2, syntax lane for gutter markers) — agree the `DiagnosticCategory` shape
  first.
- The only Tier-1 item is #13 (contract field). Everything else is reversible UI.
