# Command-line tools, building from source, and supported LaTeX

FlashTeX's engine is a set of small Rust programs that the Mac app runs for
you. You can also run them yourself from a terminal. None of them invoke a TeX
distribution: layout, fonts and PDF writing are all original code.

| Tool | What it is | Bundled in the app |
|---|---|---|
| `flashtex-render` | The **producer** the app uses: typesets LaTeX with Latin Modern / TeX metrics, answers runtime-v1 requests, and can write a PDF or a display list | `FlashTeX.app/Contents/MacOS/flashtex-render` |
| `flashtex-compiler` | The older FT-002 worker with Core-14 (Times) metrics and a smaller LaTeX subset; same wire protocol | `…/MacOS/flashtex-compiler` |
| `flashtex-pdf` | Writes a PDF from a runtime-v1 `compile_result` (text items and typed rules) | `…/MacOS/flashtex-pdf` |
| `flashtex-pdf-exact` | Writes a PDF from a rendering-v2 display list with the original glyph IDs and embedded font programs — the highest-fidelity route | `…/MacOS/flashtex-pdf-exact` |

Every example below uses the app-bundled binaries; substitute
`crates/<crate>/target/release/…` if you built from source. If you installed
with the DMG or `install.sh` the app is in `/Applications` (or
`~/Applications`):

```sh
FT=/Applications/FlashTeX.app/Contents/MacOS
```

All tools speak **JSON Lines**: one JSON object per line on stdin, one reply
per line on stdout, human-readable notes on stderr. There is no `--tex FILE`
one-shot flag yet (`flashtex-render --help` prints only the options listed
below), so wrap your file in a request with a few lines of Python.

## `flashtex-render`

```
usage: flashtex-render [--v2 out.json] [--pdf out.pdf] [--font-dir DIR]... [--class-options OPTS] [--secnumdepth N] [--timing]
```

| Flag | Meaning | Default |
|---|---|---|
| `--pdf out.pdf` | Also write a PDF of the last request (text-item route; math symbols outside the Latin Modern text face are written as `?` with a stderr note — use `flashtex-pdf-exact` for exact math) | off |
| `--v2 out.json` | Also write the rendering-v2 display list of the last request (input for `flashtex-pdf-exact from-v2` and for *File › Open Display List (v2)…* in the app) | off |
| `--font-dir DIR` | Extra font directory, repeatable; probed before the bundled and TeX Live defaults | — |
| `--class-options OPTS` | Class options assumed when the input has no `\documentclass` (body-only input) | `12pt` |
| `--secnumdepth N` | Section numbering depth when the source does not set the counter | `2` |
| `--timing` | Print each request's wall time on stderr | off |
| `-h`, `--help` | Print the usage line | |

### Render a `.tex` file to PDF

Save this as `mkreq.py` — it turns a `.tex` file into one `compile` request
line:

```python
import json, pathlib, sys
tex = pathlib.Path(sys.argv[1]).read_text()
print(json.dumps({"protocol_version": 1, "id": "req-1", "type": "compile",
                  "payload": {"project_id": "cli", "revision": 1, "entry_path": "main.tex",
                              "documents": [{"path": "main.tex", "text": tex}]}}))
```

Then:

```sh
python3 mkreq.py fixtures/real-world/hw1/HW1.tex > request.jsonl
"$FT/flashtex-render" --pdf hw1.pdf --v2 hw1-v2.json --timing < request.jsonl > result.jsonl
```

On an M1 Max this prints `flashtex-render: req-1 rendered in 31.41 ms` and
writes a three-page `hw1.pdf` (239 KB) plus `hw1-v2.json`; `result.jsonl`
holds the `compile_result` line (status `recovered`, 8 diagnostics for this
document) followed by nothing else. A request with several files lists each
one in `documents` with its project-relative `path`; `\input{chapter1}`
resolves against those paths (absolute paths and `..` are rejected).

For the exact PDF (embedded Latin Modern subsets, original glyph IDs, typed
rules):

```sh
"$FT/flashtex-pdf-exact" from-v2 hw1-v2.json --out hw1-exact.pdf
```

### The reply

`compile_result` → `payload`:

| Field | Meaning |
|---|---|
| `status` | `ok`, `recovered` (pages were produced despite diagnostics) or `failed` (no pages) |
| `pages[]` | `number` (1-based), `width_pt`, `height_pt`, `items[]` |
| `items[]` | `kind: "text"` with `text`, `x_pt`, `baseline_y_pt`, `font_size_pt`, `source`; with `rules-v1` negotiated also `kind: "rule"` boxes |
| `diagnostics[]` | `severity` (`error`/`warning`), `code`, `message`, `source` (`{path, start_byte, end_byte}` — zero-based UTF-8 byte offsets, or `null`), `recovery` (what was rendered instead, or `null`) |
| `pdf_path` | always `null`; PDFs come from `--pdf` or the pdf tools |

Malformed JSON, an unknown `protocol_version`/`type`, invalid UTF-8, unsafe
paths or a request line over 8 MiB get an `error` envelope
(`{"type":"error","payload":{"code":…,"message":…}}`) instead of a result; the
worker keeps running. Add `"layout_capabilities": ["rules-v1", "font-hints-v1"]`
to the payload to receive typed fraction/radical rules and per-item font hints
(the app requests both), and `"display-list-v2"` to get the display list as a
second stdout line after the result.

### Fonts and metrics

`flashtex-render` lays text out with the same TeX font metrics pdfLaTeX uses
(`ec-lm*.tfm`, `rm-lmr*.tfm`) and paints with the Latin Modern OpenType faces.
It looks for them, in order:

1. `FLASHTEX_FONT_DIRS` / `FLASHTEX_TFM_DIRS` (colon-separated; explicit
   entries always win) and `--font-dir`;
2. `FLASHTEX_LM_DIR`;
3. the bundle next to the executable: `<exe>/../Resources/texmf/fonts/{opentype,tfm}/public/lm`
   and `<exe>/../Resources/Fonts` — the app ships Latin Modern OTFs under
   `apps/mac/Fonts` and the required `.tfm` set under
   `apps/mac/Fonts/texmf`, so **no TeX installation is needed**;
4. MacTeX/BasicTeX 2025–2026 and Debian TeX Live paths.

Without the TFMs the OpenType metrics are used and a `tfm_missing` /
`math_metrics_opentype` warning says so; a missing required metric set is the
blocking `required_metrics_unavailable` error, never a silent fallback. The
bundle covers Latin Modern Roman regular/bold/italic at 5–17 pt and the math
faces; sans, typewriter and small caps are not bundled.

## `flashtex-compiler`

The original runtime-v1 worker (`crates/compiler`). Same request/reply shapes
as above, no command-line flags, metrics from the Adobe Core 14 AFMs (Times,
Helvetica, Courier, Symbol) and a smaller LaTeX subset — on `HW1.tex` it
reports 119 diagnostics where `flashtex-render` reports 8. Use it only when
you need its behaviour specifically; *File › Attach Built Compiler* (⌘⇧K) in
the app attaches it.

```sh
"$FT/flashtex-compiler" < request.jsonl > result.jsonl
```

## `flashtex-pdf` and `flashtex-pdf-exact`

```
usage: flashtex-pdf [INPUT.json] --out OUTPUT.pdf [--verify] [--embed-font PATH|auto] [--default-face embedded|lm|times]
```

Reads a `compile_result` envelope (file or stdin) and writes a PDF from its
text items and rules. `--verify` re-parses the written file; `--embed-font
auto` embeds `$FLASHTEX_UNICODE_FONT`, else Latin Modern, else a system font;
`--default-face` picks whether base-14 Times or the embedded font is used
first. Characters that neither WinAnsi, Symbol nor the embedded face can
represent are written as `?` with a warning (14 on `HW1.tex`). This is the
*File › Export PDF via Rust Writer…* route.

```
usage: flashtex-pdf-exact reemit REF.pdf OUT.pdf | classify A.pdf B.pdf | dump X.pdf | from-v2 LIST.json --out OUT.pdf [--font-dir DIR]...
```

`from-v2` is the one you want: it takes a display list (`flashtex-render
--v2`) and embeds subsetted CFF font programs, keeps the producer's advances
and glyph IDs, and draws typed rules. `dump`, `reemit` and `classify` are
PDF-comparison utilities used by the fidelity tests. This is the *File ›
Export PDF (exact, v2)…* route.

## Building from source

Requirements: macOS 14+ on Apple Silicon, Xcode Command Line Tools (Swift 6),
a stable Rust toolchain from rustup. There is **no Cargo workspace** at the
repository root — build each crate on its own:

```sh
git clone https://github.com/flash-tex/flashtex.git && cd flashtex
for c in compiler render-pipeline pdf bridge preview-controller edit-ledger project-files; do
  cargo build --release --manifest-path crates/$c/Cargo.toml
done
./apps/mac/scripts/make-app.sh --install --open     # packages FlashTeX.app into ~/Applications
```

Binaries land in `crates/<crate>/target/release/` (`flashtex-render`,
`flashtex-compiler`, `flashtex-pdf` + `flashtex-pdf-exact`, `flashtex-bridge`,
`flashtex-preview-controller`, `flashtex-edit-ledger`, `flashtex-project-files`).
`make-app.sh` picks them up from those paths,
verifies the bundled fonts and metrics against a pinned manifest, and ad-hoc
signs the bundle; add `--dmg` for a disk image. `cargo test` in a crate
directory runs that crate's tests; `swift test` in `apps/mac` runs the app's.

The section below is generated from the compiler itself
(`flashtex-compiler --supported markdown`). Do not edit it by hand: change the
compiler, then run `crates/compiler/scripts/render_supported_latex.sh`. The
same data is available as JSON from `flashtex-compiler --supported`.

<!-- BEGIN GENERATED supported-latex: `flashtex-compiler --supported markdown`; do not edit by hand -->
## Supported LaTeX

This compiler implements a finite LaTeX subset: 323 text-mode and 538 math-mode command entries, 55 environments and 17 layout-neutral packages. Every other command produces an explicit "not supported" diagnostic naming it, and every other environment or package a warning; nothing is dropped silently. Descriptions note approximations. Outstanding features with reproductions are in `crates/compiler/UNSUPPORTED.md`.

Regenerate with `crates/compiler/scripts/render_supported_latex.sh`; `cargo test --test supported_latex` fails when this section is stale.

### Coverage

| Canonical set | Commands supported | Environments supported |
| --- | ---: | ---: |
| kernel | 173/410 (42.2%) | 19/30 (63.3%) |
| amsmath | 40/102 (39.2%) | 17/21 (81.0%) |
| amssymb | 225/229 (98.3%) | none defined |
| enumitem | 1/17 (5.9%) | none defined |
| geometry | 0/7 (0.0%) | none defined |
| graphicx | 6/8 (75.0%) | none defined |
| hyperref | 35/132 (26.5%) | 1/2 (50.0%) |
| tikz | 0/43 (0.0%) | 0/2 (0.0%) |
| xcolor | 14/71 (19.7%) | none defined |
| siunitx | 17/240 (7.1%) | none defined |
| **total** | **548/1314 (41.7%)** | |

Generated by `flashtex-compiler --supported coverage` against `crates/compiler/supported/canonical-latex.tsv` (LaTeX2e reference-manual index and package sources, each name confirmed by pdfLaTeX; TeX Live 2026). The total row counts commands and environments together. Supported means handled without an unsupported diagnostic, not typographic parity.

Canonical sources:

- Engine: TeX 3.141592653 (TeX Live 2026)
- Source kernel: doc/latex/latex2e-help-texinfo/latex2e.texi (UPDATED May 2024) @findex commands and @EnvIndex environments; confirmed with LaTeX2e 2025-11-01 patch level 0, article class
- Source amsmath: tokens of amsmath.sty, amstext.sty, amsbsy.sty, amsopn.sty (amsmath.sty 2025/07/09 v2.17z AMS math features; amstext.sty 2024/11/17 v2.01 AMS text; amsbsy.sty 1999/11/29 v1.2d Bold Symbols; amsopn.sty 2022/04/08 v2.04 operator names); dependency baseline: amsgen
- Source amssymb: tokens of amssymb.sty, amsfonts.sty (amssymb.sty 2013/01/14 v3.01 AMS font symbols; amsfonts.sty 2013/01/14 v3.01 Basic AMSFonts support); dependency baseline: none
- Source enumitem: tokens of enumitem.sty (enumitem.sty 2025/02/06 v3.11 Customized lists); dependency baseline: none
- Source geometry: tokens of geometry.sty (geometry.sty 2026/03/07 v6.0 Page Geometry); dependency baseline: keyval, ifvtex, iftex
- Source graphicx: tokens of graphicx.sty, graphics.sty (graphicx.sty 2024/12/31 v1.2e Enhanced LaTeX Graphics (DPC,SPQR); graphics.sty 2024/08/06 v1.4g Standard LaTeX Graphics (DPC,SPQR)); dependency baseline: keyval, trig
- Source hyperref: tokens of hyperref.sty, nameref.sty (hyperref.sty 2026-01-29 v7.01p Hypertext links for LaTeX; nameref.sty 2026-01-29 v2.58 Cross-referencing by name of section); dependency baseline: iftex, keyval, kvsetkeys, kvdefinekeys, pdfescape, ltxcmds, pdftexcmds, infwarerr, hycolor, refcount, gettitlestring, kvoptions, etoolbox, stringenc, intcalc, url, bitset, bigintcalc, rerunfilecheck, uniquecounter
- Source tikz: tokens of tikz.sty, tikz.code.tex (tikz.sty 2025-08-29 v3.1.11a (3.1.11a); pgf 3.1.11a); dependency baseline: pgf, pgfrcs, pgfcore, graphicx, keyval, graphics, trig, pgfsys, xcolor, pgfcomp-version-0-65, pgfcomp-version-1-18, pgffor, pgfkeys, pgfmath
- Source xcolor: tokens of xcolor.sty (xcolor.sty 2024/09/29 v3.02 LaTeX color extensions (UK)); dependency baseline: none
- Source siunitx: tokens of siunitx.sty (siunitx.sty 2025-07-09 v3.4.14 A comprehensive (SI) units package); dependency baseline: translations, etoolbox, pdftexcmds, infwarerr, iftex, ltxcmds, amstext, amsgen, array

### Text commands

| Command | Arguments | Behaviour |
| --- | --- | --- |
| `\lstset` | `{keys}` | listings keys for the listings that follow; keys the render pipeline does not lay out warn |
| `\lstdefinestyle` | `{name}{keys}` | named listings key set for style=; keys the render pipeline does not lay out warn |
| `\lstloadlanguages` | `{languages}` | accepted no-op; the C, C++, Java and Python keyword lists are built in |
| `\lstinputlisting` | `[keys]{file}` | listing of a project document's lines |
| `\num` | `[options]{number}` | siunitx number: digit groups, decimal marker, exponent, uncertainty, as an upright formula |
| `\qty` | `[options]{number}{units}` | siunitx quantity: number, unbreakable thin space, unit |
| `\unit` | `[options]{units}` | siunitx unit: prefixes, powers, \per as a power, fraction or solidus; literal m/s |
| `\si` | `[options]{units}` | siunitx v2 name of \unit |
| `\SI` | `[options]{number}[pre-unit]{units}` | siunitx v2 name of \qty with an optional pre-unit |
| `\numlist` | `[options]{numbers}` | siunitx list of ;-separated numbers joined by list-separator and " and " |
| `\numrange` | `[options]{number}{number}` | siunitx range: two numbers joined by range-phrase " to " |
| `\qtylist` | `[options]{numbers}{units}` | siunitx list of quantities, the unit repeated |
| `\qtyrange` | `[options]{number}{number}{units}` | siunitx range of quantities, the unit repeated |
| `\SIlist` | `[options]{numbers}{units}` | siunitx v2 name of \qtylist |
| `\SIrange` | `[options]{number}{number}{units}` | siunitx v2 name of \qtyrange |
| `\ang` | `[options]{degrees;minutes;seconds}` | siunitx angle with degree, minute and second marks |
| `\sisetup` | `{options}` | siunitx settings for the following commands (document-global in this model) |
| `\DeclareSIUnit` | `[options]{\name}{units}` | defines a siunitx unit macro usable inside \unit and \qty |
| `\section` | `{...}` | numbered section heading; starred form unnumbered |
| `\subsection` | `{...}` | numbered subsection heading; starred form unnumbered |
| `\subsubsection` | `{...}` | numbered subsubsection heading; starred form unnumbered |
| `\tableofcontents` |  | article contents list from the previous layout pass |
| `\textbf` | `{...}` | bold text |
| `\textmd` | `{...}` | medium-weight text |
| `\emph` | `{...}` | emphasis: toggles italic |
| `\textit` | `{...}` | italic text |
| `\textsl` | `{...}` | slanted text (typeset as italic) |
| `\textsc` | `{...}` | small capitals (upright in the Core 14 layout) |
| `\textup` | `{...}` | upright text |
| `\texttt` | `{...}` | typewriter text |
| `\textrm` | `{...}` | roman text |
| `\textsf` | `{...}` | sans-serif text |
| `\textnormal` | `{...}` | normal text face |
| `\begin` | `{env}` | opens a supported environment |
| `\end` | `{env}` | closes the innermost open environment |
| `\par` |  | ends the paragraph |
| `\documentclass` | `[options]{class}` | records the class and its 10pt/11pt/12pt size option; only the document body is typeset |
| `\setlength` | `{\length}{dimension}` | preamble \parskip, and \parindent of 0pt; other lengths warn |
| `\usepackage` | `[options]{a,b,c}` | records packages; layout-neutral ones are silent, every other package warns that it is not implemented |
| `\definecolor` | `[class]{name}{model}{spec}` | colour definition in rgb, cmy, cmyk, gray, RGB, HTML or Gray (model lists pick the target model) |
| `\providecolor` | `[class]{name}{model}{spec}` | \definecolor unless the colour is already defined |
| `\xdefinecolor` | `[class]{name}{model}{spec}` | xcolor synonym of \definecolor |
| `\colorlet` | `[class]{name}[model]{expression}` | names an xcolor expression, optionally converted to a model |
| `\definecolorset` | `[class]{models}{head}{tail}{set}` | defines name,spec;... colours in one go |
| `\DefineNamedColor` | `{named}{name}{model}{spec}` | driver named colour, as dvipsnam.def uses it |
| `\selectcolormodel` | `{model}` | xcolor target model: natural, rgb, cmy, cmyk or gray |
| `\color` | `[model]{expression}` | text colour for the rest of the group; pdfTeX's exact operator values |
| `\textcolor` | `[model]{expression}{text}` | text in a colour |
| `\pagecolor` | `[model]{expression}` | page background colour, document-wide |
| `\nopagecolor` |  | removes the page background colour |
| `\normalcolor` |  | back to the default text colour |
| `\colorbox` | `[model]{expression}{text}` | text on a filled box \fboxsep larger than its content |
| `\fcolorbox` | `[model]{frame}{fill}{text}` | \colorbox inside a \fboxrule frame |
| `\newcolumntype` | `{X}[n]{spec}` | array column type expanded in later tabular specifications |
| `\arraybackslash` |  | array no-op: \\ already ends the row inside p, m and b entries |
| `\setlist` | `[list]{options}` | enumitem keys recorded on every matching list; itemsep and topsep also set the built-in layout, other keys warn |
| `\newcommand` | `{\name}[n]{body}` | defines a macro with 0-9 arguments; rejects an existing name |
| `\renewcommand` | `{\name}[n]{body}` | redefines an existing macro |
| `\DeclareMathOperator` | `*{\name}{text}` | defines \name as \operatorname{text}; the starred form takes limits |
| `\input` | `{path}` | expands a project-relative document in place |
| `\include` | `{path}` | expands a project-relative document in place |
| `\label` | `{key}` | names the current section, equation or figure number |
| `\ref` | `{key}` | number of the labelled item |
| `\pageref` | `{key}` | page number of the labelled item |
| `\eqref` | `{key}` | parenthesised equation number of the labelled item |
| `\numberwithin` | `[\style]{counter}{parent}` | amsmath: counter reset by parent and printed \theparent.\style{counter} (equation, figure, table; theorem counters within section) |
| `\counterwithin` | `{counter}{parent}` | counter reset by parent and printed \theparent.\arabic{counter}; starred form keeps the printed form |
| `\counterwithout` | `{counter}{parent}` | undoes \counterwithin; starred form keeps the printed form |
| `\caption` | `{...}` | numbered "Figure N:" caption inside figure; "Algorithm N" inside algorithm |
| `\item` | `[label]` | entry of an itemize, enumerate or description list |
| `\includegraphics` | `*[keys]{file}` | image box in running text (graphicx keys as written) |
| `\scalebox` | `{x}[y]{...}` | graphics.sty scaled box of the content |
| `\resizebox` | `*{width}{height}{...}` | graphics.sty box scaled to a width and/or height; ! keeps the aspect ratio |
| `\rotatebox` | `[keys]{angle}{...}` | graphicx rotated box; the box is the rotated bounding box |
| `\reflectbox` | `{...}` | graphics.sty box mirrored left to right |
| `\graphicspath` | `{{dir/}...}` | image search directories; no material |
| `\url` | `{url}` | monospaced URL text; with hyperref a URI link is recorded for export (not clickable in the preview) |
| `\href` | `{url}{text}` | link text; with hyperref a URI (or #name) link is recorded for export (not clickable in the preview) |
| `\nolinkurl` | `{url}` | monospaced URL text without a link |
| `\hypersetup` | `{key=value,...}` | hyperref options: colorlinks, link/url/cite colours, hidelinks, pdfborder, bookmarks, PDF info |
| `\pdfstringdefDisableCommands` | `{...}` | accepted; PDF strings drop commands already |
| `\hyperbaseurl` | `{url}` | accepted; relative URIs are written unchanged |
| `\setpdflinkmargin` | `{dimension}` | accepted; link rectangles keep pdfTeX's 1pt margin |
| `\autoref` | `{key}` | hyperref name (\sectionautorefname, ...) and number of the label, linked; starred form unlinked |
| `\autopageref` | `{key}` | \pageautorefname and the label's page, linked |
| `\nameref` | `{key}` | title of the labelled section or caption, linked; starred form unlinked |
| `\hyperlink` | `{name}{text}` | text linked to a named destination |
| `\hypertarget` | `{name}{text}` | text with a named destination |
| `\hyperdef` | `{category}{name}{text}` | text with the destination category.name |
| `\hyperref` | `[key]{text}` | text linked to a label's destination; also {url}{category}{name}{text} |
| `\texorpdfstring` | `{tex}{pdf}` | typesets tex; bookmarks use pdf |
| `\phantomsection` |  | anonymous destination (section*.n) for the next \label |
| `\pdfbookmark` | `[level]{text}{name}` | outline entry and destination name.level |
| `\AMSautorefname` |  | hyperref name, follows \equationautorefname |
| `\FancyVerbLineautorefname` |  | hyperref name: line |
| `\Hfootnoteautorefname` |  | hyperref name, follows \footnoteautorefname |
| `\Itemautorefname` |  | hyperref name, follows \itemautorefname |
| `\appendixautorefname` |  | hyperref name: Appendix |
| `\chapterautorefname` |  | hyperref name: chapter |
| `\equationautorefname` |  | hyperref name: Equation |
| `\figureautorefname` |  | hyperref name: Figure |
| `\footnoteautorefname` |  | hyperref name: footnote |
| `\itemautorefname` |  | hyperref name: item |
| `\pageautorefname` |  | hyperref name: page |
| `\paragraphautorefname` |  | hyperref name: paragraph |
| `\partautorefname` |  | hyperref name: Part |
| `\sectionautorefname` |  | hyperref name: section |
| `\subparagraphautorefname` |  | hyperref name: subparagraph |
| `\subsectionautorefname` |  | hyperref name: subsection |
| `\subsubsectionautorefname` |  | hyperref name: subsubsection |
| `\tableautorefname` |  | hyperref name: Table |
| `\theoremautorefname` |  | hyperref name: Theorem |
| `\hfill` |  | infinite-stretch horizontal glue |
| `\hfil` |  | infinite-stretch horizontal glue (same order as \hfill) |
| `\hss` |  | horizontal glue 0pt plus 1fil minus 1fil (inside boxes) |
| `\hspace` | `{dimension}` | fixed horizontal space; starred form identical |
| `\mbox` | `{...}` | unbreakable horizontal box |
| `\makebox` | `[width][l\|c\|r\|s]{...}` | horizontal box of a given width; \width/\height/\depth/\totalheight in the width |
| `\fbox` | `{...}` | framed box, \fboxsep and \fboxrule from \setlength |
| `\framebox` | `[width][l\|c\|r\|s]{...}` | framed box of a given width |
| `\parbox` | `[t\|c\|b][height][t\|c\|b\|s]{width}{...}` | vertical box of paragraphs at a width, aligned on its first, centre or last line |
| `\raisebox` | `{lift}[height][depth]{...}` | raised or lowered box with optional height/depth overrides |
| `\phantom` | `{...}` | invisible box with the content's width, height and depth |
| `\hphantom` | `{...}` | invisible box with the content's width only |
| `\vphantom` | `{...}` | invisible box with the content's height and depth only |
| `\smash` | `{...}` | box with its height and depth set to zero |
| `\llap` | `{...}` | zero-width box, content overlapping to the left |
| `\rlap` | `{...}` | zero-width box, content overlapping to the right |
| `\strut` |  | invisible rule of .7/.3 \baselineskip height/depth |
| `\newsavebox` | `{\name}` | declares a box register |
| `\sbox` | `{\name}{...}` | stores a horizontal box |
| `\savebox` | `{\name}[width][pos]{...}` | stores a horizontal box of a given width |
| `\usebox` | `{\name}` | typesets a stored box again |
| `\newlength` | `{\name}` | declares a length for box dimensions, \hspace and \setlength |
| `\settowidth` | `{\name}{...}` | sets a length to the natural width of the content |
| `\settoheight` | `{\name}{...}` | sets a length to the height of the content |
| `\settodepth` | `{\name}{...}` | sets a length to the depth of the content |
| `\footnote` | `[n]{...}` | numbered mark and page-bottom footnote text |
| `\footnotemark` | `[n]` | footnote mark only |
| `\footnotetext` | `[n]{...}` | footnote text without a mark |
| `\normalfont` |  | resets the text face |
| `\bfseries` |  | switches to bold |
| `\mdseries` |  | switches to medium weight |
| `\itshape` |  | switches to italic |
| `\slshape` |  | switches to slanted (typeset as italic) |
| `\scshape` |  | switches to small capitals (upright in the Core 14 layout) |
| `\upshape` |  | switches to upright |
| `\ttfamily` |  | switches to typewriter |
| `\rmfamily` |  | switches to roman |
| `\sffamily` |  | switches to sans-serif |
| `\em` |  | toggles emphasis |
| `\bf` |  | LaTeX 2.09 form: bold roman |
| `\it` |  | LaTeX 2.09 form: italic roman |
| `\sl` |  | LaTeX 2.09 form: slanted roman |
| `\sc` |  | LaTeX 2.09 form: small-caps roman (upright in the Core 14 layout) |
| `\tt` |  | LaTeX 2.09 form: upright typewriter |
| `\rm` |  | LaTeX 2.09 form: upright roman |
| `\sf` |  | LaTeX 2.09 form: upright sans-serif |
| `\quad` |  | 1em of horizontal space |
| `\qquad` |  | 2em of horizontal space |
| `\bigskip` |  | ends the paragraph and adds 12pt of vertical space |
| `\medskip` |  | ends the paragraph and adds 6pt of vertical space |
| `\smallskip` |  | ends the paragraph and adds 3pt of vertical space |
| `\vspace` | `{dimension}` | ends the paragraph and adds fixed vertical space |
| `\hrule` |  | full-measure horizontal rule |
| `\newpage` |  | forces a page break |
| `\clearpage` |  | forces a page break |
| `\cleardoublepage` |  | forces a page break (one-sided article) |
| `\pagebreak` | `[n]` | forces a page break |
| `\nopagebreak` | `[n]` | accepted no-op; the layout never breaks there on its own |
| `\linebreak` | `[n]` | line break |
| `\nolinebreak` | `[n]` | accepted no-op |
| `\vfill` |  | vertical glue filling the rest of the page |
| `\columnbreak` | `[n]` | multicol: ends the current column of multicols (priority n, default 4) |
| `\newcolumn` |  | multicol: ends the current column of multicols, filling it |
| `\raggedcolumns` |  | multicol: columns keep their natural height |
| `\flushcolumns` |  | multicol: columns are stretched to one height (the default) |
| `\pagestyle` | `{style}` | accepted; no headers or footers are rendered |
| `\thispagestyle` | `{style}` | accepted; no headers or footers are rendered |
| `\pagenumbering` | `{style}` | accepted; no page numbers are rendered |
| `\listfiles` |  | accepted no-op; there is no log stream |
| `\centering` |  | centres the following paragraphs |
| `\Centering` |  | centres the following paragraphs (ragged2e form) |
| `\raggedright` |  | left-aligned following paragraphs |
| `\RaggedRight` |  | left-aligned following paragraphs (ragged2e form) |
| `\raggedleft` |  | right-aligned following paragraphs |
| `\RaggedLeft` |  | right-aligned following paragraphs (ragged2e form) |
| `\noindent` |  | accepted no-op; paragraphs are never indented |
| `\indent` |  | accepted; the first-line indent is diagnosed, not drawn |
| `\tiny` |  | size declaration from the class size table |
| `\scriptsize` |  | size declaration from the class size table |
| `\footnotesize` |  | size declaration from the class size table |
| `\small` |  | size declaration from the class size table |
| `\normalsize` |  | size declaration: the active body size |
| `\large` |  | size declaration from the class size table |
| `\Large` |  | size declaration from the class size table |
| `\LARGE` |  | size declaration from the class size table |
| `\huge` |  | size declaration from the class size table |
| `\Huge` |  | size declaration from the class size table |
| `\cite` | `[note]{keys}` | citation as latex.ltx \@citex ([1, 2, note]); natbib's \cite under natbib |
| `\nocite` | `{keys}` | no output; warns for undefined keys |
| `\bibitem` | `[label]{key}` | entry of thebibliography; natbib reads Name(Year)Long names labels |
| `\bibliography` | `{files}` | reads the project's <jobname>.bbl in its place; .bib files are not read |
| `\bibliographystyle` | `{style}` | natbib punctuation of the BibTeX style; entry formatting comes from the .bbl |
| `\citet` | `*[pre][post]{keys}` | natbib textual citation: Name (Year) or Name [n] |
| `\citep` | `*[pre][post]{keys}` | natbib parenthetical citation: (Name, Year) or [n] |
| `\citealt` | `*[pre][post]{keys}` | natbib textual citation without brackets |
| `\citealp` | `*[pre][post]{keys}` | natbib parenthetical citation without brackets |
| `\citeauthor` | `*{keys}` | natbib author names |
| `\citeyear` | `{keys}` | natbib year |
| `\citeyearpar` | `{keys}` | natbib year in brackets |
| `\citenum` | `{keys}` | natbib citation number |
| `\Citet` | `*[pre][post]{keys}` | \citet with the first name capitalised |
| `\Citep` | `*[pre][post]{keys}` | \citep with the first name capitalised |
| `\Citealt` | `*[pre][post]{keys}` | \citealt with the first name capitalised |
| `\Citealp` | `*[pre][post]{keys}` | \citealp with the first name capitalised |
| `\Citeauthor` | `*{keys}` | \citeauthor with the first name capitalised |
| `\citestyle` | `{style}` | natbib punctuation of a named BibTeX style |
| `\bibpunct` | `[cmt]{open}{close}{sep}{mode}{aysep}{yrsep}` | natbib citation punctuation and mode |
| `\setcitestyle` | `{options}` | natbib citation punctuation keywords and key=value settings |
| `\newblock` |  | glue between blocks of a bibliography entry |
| `\natexlab` | `{letter}` | natbib extra year label, shown in author-year mode |
| `\penalty` | `number` | a break opportunity (BibTeX's \penalty0); sets no text |
| `\title` | `{...}` | title for \maketitle |
| `\author` | `{...}` | author block for \maketitle; \and and \thanks inside it |
| `\date` | `{...}` | date for \maketitle; \today inside it |
| `\maketitle` |  | article.cls title block |
| `\TeX` |  | latex.ltx logo: T, kern -.1667em, E lowered .5ex, kern -.125em, X |
| `\LaTeX` |  | latex.ltx logo: L, kern -.36em, script-size A raised to the T height, kern -.15em, \TeX |
| `\LaTeXe` |  | \LaTeX, kern .15em, 2 and a text-style subscript varepsilon |
| `\rule` | `[raise]{dimension}{dimension}` | filled rule box; pt/in/cm/mm/bp/dd/cc/pc/sp, em, ex, \textwidth, \linewidth, \columnwidth |
| `\thinspace` |  | text kern .16667em (math: thin muskip) |
| `\negthinspace` |  | text kern -.16667em |
| `\medspace` |  | text kern .2222em |
| `\negmedspace` |  | text kern -.2222em |
| `\thickspace` |  | text kern .2777em |
| `\negthickspace` |  | text kern -.2777em |
| `\enspace` |  | text kern .5em |
| `\enskip` |  | horizontal glue of .5em |
| `\AA` |  | text symbol \AA: OT1 Å, T1 Å (tex-text-encoding; unavailable is a LaTeX error) |
| `\aa` |  | text symbol \aa: OT1 å, T1 å (tex-text-encoding; unavailable is a LaTeX error) |
| `\AE` |  | text symbol \AE: OT1 Æ, T1 Æ (tex-text-encoding; unavailable is a LaTeX error) |
| `\ae` |  | text symbol \ae: OT1 æ, T1 æ (tex-text-encoding; unavailable is a LaTeX error) |
| `\OE` |  | text symbol \OE: OT1 Œ, T1 Œ (tex-text-encoding; unavailable is a LaTeX error) |
| `\oe` |  | text symbol \oe: OT1 œ, T1 œ (tex-text-encoding; unavailable is a LaTeX error) |
| `\O` |  | text symbol \O: OT1 Ø, T1 Ø (tex-text-encoding; unavailable is a LaTeX error) |
| `\o` |  | text symbol \o: OT1 ø, T1 ø (tex-text-encoding; unavailable is a LaTeX error) |
| `\L` |  | text symbol \L: OT1 Ł, T1 Ł (tex-text-encoding; unavailable is a LaTeX error) |
| `\l` |  | text symbol \l: OT1 ł, T1 ł (tex-text-encoding; unavailable is a LaTeX error) |
| `\ss` |  | text symbol \ss: OT1 ß, T1 ß (tex-text-encoding; unavailable is a LaTeX error) |
| `\SS` |  | text symbol \SS: OT1 SS, T1 ẞ (tex-text-encoding; unavailable is a LaTeX error) |
| `\TH` |  | text symbol \TH: OT1 unavailable, T1 Þ (tex-text-encoding; unavailable is a LaTeX error) |
| `\th` |  | text symbol \th: OT1 unavailable, T1 þ (tex-text-encoding; unavailable is a LaTeX error) |
| `\DH` |  | text symbol \DH: OT1 unavailable, T1 Ð (tex-text-encoding; unavailable is a LaTeX error) |
| `\dh` |  | text symbol \dh: OT1 unavailable, T1 ð (tex-text-encoding; unavailable is a LaTeX error) |
| `\DJ` |  | text symbol \DJ: OT1 unavailable, T1 Đ (tex-text-encoding; unavailable is a LaTeX error) |
| `\dj` |  | text symbol \dj: OT1 unavailable, T1 đ (tex-text-encoding; unavailable is a LaTeX error) |
| `\NG` |  | text symbol \NG: OT1 unavailable, T1 Ŋ (tex-text-encoding; unavailable is a LaTeX error) |
| `\ng` |  | text symbol \ng: OT1 unavailable, T1 ŋ (tex-text-encoding; unavailable is a LaTeX error) |
| `\IJ` |  | text symbol \IJ: OT1 Ĳ, T1 Ĳ (tex-text-encoding; unavailable is a LaTeX error) |
| `\ij` |  | text symbol \ij: OT1 ĳ, T1 ĳ (tex-text-encoding; unavailable is a LaTeX error) |
| `\i` |  | text symbol \i: OT1 ı, T1 ı (tex-text-encoding; unavailable is a LaTeX error) |
| `\j` |  | text symbol \j: OT1 ȷ, T1 ȷ (tex-text-encoding; unavailable is a LaTeX error) |
| `\S` |  | text symbol \textsection: OT1 §, T1 § (tex-text-encoding; unavailable is a LaTeX error) |
| `\P` |  | text symbol \textparagraph: OT1 ¶, T1 ¶ (tex-text-encoding; unavailable is a LaTeX error) |
| `\dag` |  | text symbol \textdagger: OT1 †, T1 † (tex-text-encoding; unavailable is a LaTeX error) |
| `\ddag` |  | text symbol \textdaggerdbl: OT1 ‡, T1 ‡ (tex-text-encoding; unavailable is a LaTeX error) |
| `\copyright` |  | text symbol \textcopyright: OT1 ©, T1 © (tex-text-encoding; unavailable is a LaTeX error) |
| `\pounds` |  | text symbol \textsterling: OT1 £, T1 £ (tex-text-encoding; unavailable is a LaTeX error) |
| `\dots` |  | text symbol \textellipsis: OT1 …, T1 … (tex-text-encoding; unavailable is a LaTeX error) |
| `\ldots` |  | text symbol \textellipsis: OT1 …, T1 … (tex-text-encoding; unavailable is a LaTeX error) |
| `\textsection` |  | text symbol \textsection: OT1 §, T1 § (tex-text-encoding; unavailable is a LaTeX error) |
| `\textparagraph` |  | text symbol \textparagraph: OT1 ¶, T1 ¶ (tex-text-encoding; unavailable is a LaTeX error) |
| `\textdagger` |  | text symbol \textdagger: OT1 †, T1 † (tex-text-encoding; unavailable is a LaTeX error) |
| `\textdaggerdbl` |  | text symbol \textdaggerdbl: OT1 ‡, T1 ‡ (tex-text-encoding; unavailable is a LaTeX error) |
| `\textcopyright` |  | text symbol \textcopyright: OT1 ©, T1 © (tex-text-encoding; unavailable is a LaTeX error) |
| `\textsterling` |  | text symbol \textsterling: OT1 £, T1 £ (tex-text-encoding; unavailable is a LaTeX error) |
| `\textellipsis` |  | text symbol \textellipsis: OT1 …, T1 … (tex-text-encoding; unavailable is a LaTeX error) |
| `\textbackslash` |  | text symbol \textbackslash: OT1 \, T1 \ (tex-text-encoding; unavailable is a LaTeX error) |
| `\textasciitilde` |  | text symbol \textasciitilde: OT1 ~, T1 ~ (tex-text-encoding; unavailable is a LaTeX error) |
| `\textasciicircum` |  | text symbol \textasciicircum: OT1 ^, T1 ^ (tex-text-encoding; unavailable is a LaTeX error) |
| `\textunderscore` |  | text symbol \textunderscore: OT1 _, T1 _ (tex-text-encoding; unavailable is a LaTeX error) |
| `\textbar` |  | text symbol \textbar: OT1 \|, T1 \| (tex-text-encoding; unavailable is a LaTeX error) |
| `\textless` |  | text symbol \textless: OT1 <, T1 < (tex-text-encoding; unavailable is a LaTeX error) |
| `\textgreater` |  | text symbol \textgreater: OT1 >, T1 > (tex-text-encoding; unavailable is a LaTeX error) |
| `\textbraceleft` |  | text symbol \textbraceleft: OT1 {, T1 { (tex-text-encoding; unavailable is a LaTeX error) |
| `\textbraceright` |  | text symbol \textbraceright: OT1 }, T1 } (tex-text-encoding; unavailable is a LaTeX error) |
| `\newtheorem` | `{env}[counter]{name}[within]` | defines a theorem-like environment: the kernel head, or with amsthm the current style; starred form unnumbered |
| `\theoremstyle` | `{style}` | selects the amsthm style for following \newtheorem |
| `\newtheoremstyle` | `{name}{above}{below}{body font}{indent}{head font}{punct}{space}{spec}` | defines an amsthm style; a custom head specification is diagnosed |
| `\swapnumbers` |  | amsthm: numbers before names in the heads of later \newtheorem environments |
| `\qed` |  | amsthm end-of-proof box, flush right |
| `\qedhere` |  | amsthm: puts the proof's end-of-proof box here instead of at \end{proof} |
| `\algnewcommand` | `{\name}[n]{body}` | algorithmicx definition: keyword texts (\algorithmicrequire ...), \algorithmicindent, \alglinenumber and \item[...] label commands |
| `\algrenewcommand` | `{\name}[n]{body}` | algorithmicx redefinition of a keyword text, \algorithmicindent or \alglinenumber |
| `\algsetup` | `{key=value}` | algorithmic indent, linenosize and linenodelimiter |
| `\floatname` | `{float}{name}` | the caption name of the algorithm float |
| `\\` |  | line break; an optional [length] is consumed |
| `\,` |  | text kern .16667em (\thinspace) |
| `\!` |  | text kern -.16667em (\negthinspace) |
| `\:` |  | text kern .2222em (\medspace) |
| `\>` |  | text kern .2222em (\medspace) |
| `\;` |  | text kern .2777em (\thickspace) |

### Commands run by the expansion pass

| Command | Arguments | Behaviour |
| --- | --- | --- |
| `\long` |  | prefix: the following definition accepts \par in arguments |
| `\protected` |  | e-TeX prefix: the following macro is not expanded inside \edef-like contexts |
| `\providecommand` | `{\name}[n][default]{body}` | defines the macro only when \name is undefined |
| `\DeclareRobustCommand` | `{\name}[n][default]{body}` | defines or redefines a macro (robustness is not modelled separately) |
| `\newenvironment` | `{env}[n][default]{begin}{end}` | defines an environment run by \begin{env}/\end{env} |
| `\renewenvironment` | `{env}[n][default]{begin}{end}` | redefines an environment |
| `\newcounter` | `{counter}[within]` | allocates a counter (\c@counter, \thecounter) reset by within |
| `\setcounter` | `{counter}{number}` | sets a counter globally |
| `\addtocounter` | `{counter}{number}` | adds to a counter globally |
| `\stepcounter` | `{counter}` | increments a counter and resets its dependants |
| `\refstepcounter` | `{counter}` | increments a counter and makes it the current \label value |
| `\value` | `{counter}` | a counter's value in a number context |
| `\Alph` | `{counter}` | a counter as an upper-case letter |
| `\fnsymbol` | `{counter}` | a counter as a footnote symbol |
| `\AtBeginDocument` | `{code}` | stores code that runs at \begin{document} |
| `\AtEndDocument` | `{code}` | stores code that runs at \end{document} |
| `\makeatother` |  | makes @ an other character again |
| `\space` |  | expands to one space |
| `\ignorespaces` |  | skips the spaces that follow |
| `\jobname` |  | expands to texput |

### Math structures

| Command | Arguments | Behaviour |
| --- | --- | --- |
| `\,` |  | thin space (3mu) |
| `\:` |  | medium space (4mu) |
| `\>` |  | medium space (4mu) |
| `\;` |  | thick space (5mu) |
| `\ ` |  | control space (6mu) |
| `\!` |  | negative thin space (-3mu) |
| `\\|` |  | double vertical bar |
| `\color` | `[model]{expression}` | colours the rest of the math group |
| `\textcolor` | `[model]{expression}{body}` | math body in a colour |
| `\num` | `[options]{number}` | siunitx number or ;-separated list inside a formula |
| `\numlist` | `[options]{number}` | siunitx number or ;-separated list inside a formula |
| `\unit` | `[options]{units}` | siunitx unit inside a formula |
| `\si` | `[options]{units}` | siunitx unit inside a formula |
| `\qty` | `[options]{number}{units}` | siunitx quantity (3mu thin space before the unit) or number range inside a formula |
| `\numrange` | `[options]{number}{units}` | siunitx quantity (3mu thin space before the unit) or number range inside a formula |
| `\SI` | `[options]{number}[pre-unit]{units}` | siunitx v2 quantity inside a formula |
| `\qtylist` | `[options]{numbers}{units}` | siunitx list of quantities inside a formula |
| `\SIlist` | `[options]{numbers}{units}` | siunitx list of quantities inside a formula |
| `\qtyrange` | `[options]{number}{number}{units}` | siunitx range of quantities inside a formula |
| `\SIrange` | `[options]{number}{number}{units}` | siunitx range of quantities inside a formula |
| `\ang` | `[options]{angle}` | siunitx angle inside a formula |
| `\sisetup` | `{options}` | siunitx settings changed inside a formula |
| `\rule` | `[raise]{dimension}{dimension}` | latex.ltx \rule box in a formula, em/ex of the text font |
| `\frac` | `{num}{den}` | fraction; \cfrac lays out as \frac |
| `\cfrac` | `{num}{den}` | fraction; \cfrac lays out as \frac |
| `\dfrac` | `{num}{den}` | amsmath \genfrac fraction in display or text style |
| `\tfrac` | `{num}{den}` | amsmath \genfrac fraction in display or text style |
| `\genfrac` | `{left}{right}{thickness}{style}{num}{den}` | amsmath generalized fraction: delimiters, pt rule thickness and a 0-3 style |
| `\phantom` | `{x}` | empty box with the width and/or height and depth of the argument |
| `\hphantom` | `{x}` | empty box with the width and/or height and depth of the argument |
| `\vphantom` | `{x}` | empty box with the width and/or height and depth of the argument |
| `\xrightarrow` | `[below]{above}` | amsmath/mathtools extensible arrow stretched to its labels (\ext@arrow) |
| `\xleftarrow` | `[below]{above}` | amsmath/mathtools extensible arrow stretched to its labels (\ext@arrow) |
| `\xleftrightarrow` | `[below]{above}` | amsmath/mathtools extensible arrow stretched to its labels (\ext@arrow) |
| `\substack` | `{a \\ b}` | amsmath centred script-style rows for limits |
| `\sqrt` | `[index]{x}` | radical with optional raised index |
| `\binom` | `{n}{k}` | amsmath binomial: zero-thickness \genfrac in parentheses; d/t forms force the style |
| `\dbinom` | `{n}{k}` | amsmath binomial: zero-thickness \genfrac in parentheses; d/t forms force the style |
| `\tbinom` | `{n}{k}` | amsmath binomial: zero-thickness \genfrac in parentheses; d/t forms force the style |
| `\choose` |  | TeX infix binomial and fraction inside a group |
| `\over` |  | TeX infix binomial and fraction inside a group |
| `\overset` | `{script}{base}` | script-size list centred above or below a base |
| `\stackrel` | `{script}{base}` | script-size list centred above or below a base |
| `\underset` | `{script}{base}` | script-size list centred above or below a base |
| `\operatorname` | `{name}` | upright named operator (\mathop); starred and withlimits forms take limits |
| `\operatornamewithlimits` | `{name}` | upright named operator (\mathop); starred and withlimits forms take limits |
| `\bmod` |  | upright mod |
| `\mod` |  | upright mod |
| `\pmod` | `{n}` | parenthesised (mod n) |
| `\mathbb` | `{A-Z}` | double-struck capitals from Latin Modern Math; other arguments are diagnosed |
| `\mathfrak` | `{letters}` | Euler Fraktur letters as Unicode mathematical fraktur; digits and other characters unchanged |
| `\mathcal` | `{A-Z}` | script capitals from New Computer Modern Math at cmsy10 metrics; other arguments are diagnosed |
| `\varnothing` |  | empty set at msbm10's 0.7778em advance (\emptyset's glyph) |
| `\iff` |  | long double arrow between thick (5mu) spaces |
| `\implies` |  | long double arrow between thick (5mu) spaces |
| `\impliedby` |  | long double arrow between thick (5mu) spaces |
| `\bot` |  | shared symbol glyph with its own atom class (Ord / Bin) |
| `\bigtriangleup` |  | shared symbol glyph with its own atom class (Ord / Bin) |
| `\mathbin` | `{math}` | argument boxed as one atom of the forced class |
| `\mathrel` | `{math}` | argument boxed as one atom of the forced class |
| `\mathord` | `{math}` | argument boxed as one atom of the forced class |
| `\mathop` | `{math}` | argument boxed as one atom of the forced class |
| `\mathopen` | `{math}` | argument boxed as one atom of the forced class |
| `\mathclose` | `{math}` | argument boxed as one atom of the forced class |
| `\mathpunct` | `{math}` | argument boxed as one atom of the forced class |
| `\mathbf` | `{text}` | literal text in Times-Bold |
| `\textbf` | `{text}` | literal text in Times-Bold |
| `\mathit` | `{...}` | letters and digits of a plain argument as Unicode mathematical italic, sans-serif or monospace; any other argument stays in the current math face |
| `\mathsf` | `{...}` | letters and digits of a plain argument as Unicode mathematical italic, sans-serif or monospace; any other argument stays in the current math face |
| `\mathtt` | `{...}` | letters and digits of a plain argument as Unicode mathematical italic, sans-serif or monospace; any other argument stays in the current math face |
| `\mathrm` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
| `\mathnormal` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
| `\boldsymbol` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
| `\bm` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
| `\mbox` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
| `\hbox` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
| `\textrm` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
| `\textit` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
| `\textnormal` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
| `\text` | `{text}` | literal text in math |
| `\boxed` | `{...}` | real rule around, over or under the body |
| `\overline` | `{...}` | real rule around, over or under the body |
| `\underline` | `{...}` | real rule around, over or under the body |
| `\overbrace` | `{body}` | cmex brace pieces with rule fills over or under a display-style body; scripts are limits |
| `\underbrace` | `{body}` | cmex brace pieces with rule fills over or under a display-style body; scripts are limits |
| `\overrightarrow` | `{body}` | amsmath \arrowfill@ as wide as the body, over or under it |
| `\overleftarrow` | `{body}` | amsmath \arrowfill@ as wide as the body, over or under it |
| `\overleftrightarrow` | `{body}` | amsmath \arrowfill@ as wide as the body, over or under it |
| `\underrightarrow` | `{body}` | amsmath \arrowfill@ as wide as the body, over or under it |
| `\underleftarrow` | `{body}` | amsmath \arrowfill@ as wide as the body, over or under it |
| `\underleftrightarrow` | `{body}` | amsmath \arrowfill@ as wide as the body, over or under it |
| `\dashrightarrow` |  | amsfonts dashed arrow: two msam \dabar@ pieces and a head in one relation |
| `\dasharrow` |  | amsfonts dashed arrow: two msam \dabar@ pieces and a head in one relation |
| `\dashleftarrow` |  | amsfonts dashed arrow: two msam \dabar@ pieces and a head in one relation |
| `\Bbb` | `{A-Z}` | obsolete amsfonts alias of \mathbb |
| `\bold` | `{text}` | obsolete amsfonts alias of \mathbf |
| `\hat` | `{body}` | base-14 accent glyph centred over the body |
| `\bar` | `{body}` | base-14 accent glyph centred over the body |
| `\vec` | `{body}` | base-14 accent glyph centred over the body |
| `\tilde` | `{body}` | base-14 accent glyph centred over the body |
| `\dot` | `{body}` | base-14 accent glyph centred over the body |
| `\ddot` | `{body}` | base-14 accent glyph centred over the body |
| `\acute` | `{body}` | base-14 accent glyph centred over the body |
| `\grave` | `{body}` | base-14 accent glyph centred over the body |
| `\widehat` | `{body}` | cmex successor-chain accent grown to the body (msbm extra-wide form past 2em with amsfonts) |
| `\widetilde` | `{body}` | cmex successor-chain accent grown to the body (msbm extra-wide form past 2em with amsfonts) |
| `\check` | `{body}` | parsed, but no base-14 glyph exists: diagnosed and typeset without a mark |
| `\breve` | `{body}` | parsed, but no base-14 glyph exists: diagnosed and typeset without a mark |
| `\left` |  | consumes the following delimiter, kept at ordinary size |
| `\right` |  | consumes the following delimiter, kept at ordinary size |
| `\big` |  | consumes the following delimiter, kept at ordinary size |
| `\Big` |  | consumes the following delimiter, kept at ordinary size |
| `\bigg` |  | consumes the following delimiter, kept at ordinary size |
| `\Bigg` |  | consumes the following delimiter, kept at ordinary size |
| `\bigl` |  | consumes the following delimiter, kept at ordinary size |
| `\bigr` |  | consumes the following delimiter, kept at ordinary size |
| `\Bigl` |  | consumes the following delimiter, kept at ordinary size |
| `\Bigr` |  | consumes the following delimiter, kept at ordinary size |
| `\biggl` |  | consumes the following delimiter, kept at ordinary size |
| `\biggr` |  | consumes the following delimiter, kept at ordinary size |
| `\Biggl` |  | consumes the following delimiter, kept at ordinary size |
| `\Biggr` |  | consumes the following delimiter, kept at ordinary size |
| `\bigm` |  | consumes the following delimiter, kept at ordinary size |
| `\Bigm` |  | consumes the following delimiter, kept at ordinary size |
| `\biggm` |  | consumes the following delimiter, kept at ordinary size |
| `\Biggm` |  | consumes the following delimiter, kept at ordinary size |
| `\dots` |  | three periods |
| `\ldots` |  | three periods |
| `\dotsc` |  | three periods |
| `\dotso` |  | three periods |
| `\cdots` |  | three math-axis dots |
| `\dotsb` |  | three math-axis dots |
| `\dotsm` |  | three math-axis dots |
| `\dotsi` |  | three math-axis dots |
| `\iint` |  | repeated integral glyph |
| `\iiint` |  | repeated integral glyph |
| `\lbrace` |  | brace glyph |
| `\rbrace` |  | brace glyph |
| `\quad` |  | 1em/2em math space |
| `\qquad` |  | 1em/2em math space |
| `\displaystyle` |  | accepted without changing size |
| `\textstyle` |  | accepted without changing size |
| `\scriptstyle` |  | accepted without changing size |
| `\scriptscriptstyle` |  | accepted without changing size |
| `\limits` |  | accepted without changing script placement |
| `\nolimits` |  | accepted without changing script placement |
| `\nonumber` |  | accepted without effect |
| `\notag` |  | accepted without effect |
| `\middle` |  | accepted without effect |
| `\qedhere` |  | amsthm: the display holds the proof's end-of-proof box |
| `\tag` | `{label}` | (label) two quads after the display; starred form without parentheses |
| `\begin` | `{env}` | opens a math grid environment |

### Math symbols

`\alpha` α, `\beta` β, `\gamma` γ, `\delta` δ, `\theta` θ, `\lambda` λ, `\mu` μ, `\pi` π, `\sigma` σ, `\phi` φ, `\omega` ω, `\epsilon` ε, `\varepsilon` ε, `\zeta` ζ, `\eta` η, `\vartheta` ϑ, `\iota` ι, `\kappa` κ, `\nu` ν, `\xi` ξ, `\varpi` ϖ, `\rho` ρ, `\varsigma` ς, `\tau` τ, `\upsilon` υ, `\varphi` ϕ, `\chi` χ, `\psi` ψ, `\Gamma` Γ, `\Delta` Δ, `\Theta` Θ, `\Lambda` Λ, `\Xi` Ξ, `\Pi` Π, `\Sigma` Σ, `\Upsilon` Υ, `\Phi` Φ, `\Psi` Ψ, `\Omega` Ω, `\le` ≤, `\ge` ≥, `\ne` ≠, `\equiv` ≡, `\sim` ∼, `\cong` ≅, `\propto` ∝, `\perp` ⊥, `\partial` ∂, `\nabla` ∇, `\prod` ∏, `\ast` ∗, `\prime` ′, `\cup` ∪, `\cap` ∩, `\subset` ⊂, `\subseteq` ⊆, `\supset` ⊃, `\supseteq` ⊇, `\notin` ∉, `\ni` ∋, `\emptyset` ∅, `\oplus` ⊕, `\otimes` ⊗, `\wedge` ∧, `\land` ∧, `\lor` ∨, `\to` →, `\rightarrow` →, `\leftarrow` ←, `\gets` ←, `\uparrow` ↑, `\downarrow` ↓, `\leftrightarrow` ↔, `\Leftarrow` ⇐, `\Leftrightarrow` ⇔, `\Uparrow` ⇑, `\Downarrow` ⇓, `\angle` ∠, `\aleph` ℵ, `\Re` ℜ, `\Im` ℑ, `\wp` ℘, `\langle` 〈, `\rangle` 〉, `\lvert` ∣, `\rvert` ∣, `\lVert` ∣∣, `\rVert` ∣∣, `\times` ×, `\div` ÷, `\pm` ±, `\leq` ≤, `\geq` ≥, `\neq` ≠, `\approx` ≈, `\cdot` ⋅, `\infty` ∞, `\sum` ∑, `\int` ∫, `\in` ∈, `\forall` ∀, `\exists` ∃, `\vee` ∨, `\Rightarrow` ⇒, `\mid` ∣, `\setminus` ∖, `\Longrightarrow` ⟹, `\mp` ∓, `\ll` ≪, `\gg` ≫, `\simeq` ≃, `\vdots` ⋮, `\ddots` ⋱, `\lfloor` ⌊, `\rfloor` ⌋, `\lceil` ⌈, `\rceil` ⌉, `\oint` ∮, `\mapsto` ↦, `\ell` ℓ, `\hbar` ℏ, `\circ` ∘, `\parallel` ∥, `\coloneqq` ≔, `\hookrightarrow` ↪, `\models` ⊨, `\vdash` ⊢, `\dashv` ⊣, `\top` ⊤, `\Longleftrightarrow` ⟺, `\longrightarrow` ⟶, `\longleftarrow` ⟵, `\Longleftarrow` ⟸, `\longleftrightarrow` ⟷, `\triangle` △, `\bigtriangledown` ▽, `\boxdot` ⊡, `\boxplus` ⊞, `\boxtimes` ⊠, `\square` □, `\blacksquare` ■, `\centerdot` ⬝, `\lozenge` ◊, `\blacklozenge` ⧫, `\circlearrowright` ↻, `\circlearrowleft` ↺, `\leftrightharpoons` ⇋, `\boxminus` ⊟, `\Vdash` ⊩, `\Vvdash` ⊪, `\vDash` ⊨, `\twoheadrightarrow` ↠, `\twoheadleftarrow` ↞, `\leftleftarrows` ⇇, `\rightrightarrows` ⇉, `\upuparrows` ⇈, `\downdownarrows` ⇊, `\upharpoonright` ↾, `\downharpoonright` ⇂, `\upharpoonleft` ↿, `\downharpoonleft` ⇃, `\rightarrowtail` ↣, `\leftarrowtail` ↢, `\leftrightarrows` ⇆, `\rightleftarrows` ⇄, `\Lsh` ↰, `\Rsh` ↱, `\rightsquigarrow` ⇝, `\leftrightsquigarrow` ↭, `\looparrowleft` ↫, `\looparrowright` ↬, `\circeq` ≗, `\succsim` ≿, `\gtrsim` ≳, `\gtrapprox` ⪆, `\multimap` ⊸, `\therefore` ∴, `\because` ∵, `\doteqdot` ≑, `\triangleq` ≜, `\precsim` ≾, `\lesssim` ≲, `\lessapprox` ⪅, `\eqslantless` ⪕, `\eqslantgtr` ⪖, `\curlyeqprec` ⋞, `\curlyeqsucc` ⋟, `\preccurlyeq` ≼, `\leqq` ≦, `\leqslant` ⩽, `\lessgtr` ≶, `\backprime` ‵, `\risingdotseq` ≓, `\fallingdotseq` ≒, `\succcurlyeq` ≽, `\geqq` ≧, `\geqslant` ⩾, `\gtrless` ≷, `\vartriangleright` ⊳, `\vartriangleleft` ⊲, `\trianglerighteq` ⊵, `\trianglelefteq` ⊴, `\bigstar` ★, `\between` ≬, `\blacktriangledown` ▾, `\blacktriangleright` ▶, `\blacktriangleleft` ◀, `\vartriangle` ▵, `\blacktriangle` ▴, `\triangledown` ▿, `\eqcirc` ≖, `\lesseqgtr` ⋚, `\gtreqless` ⋛, `\lesseqqgtr` ⪋, `\gtreqqless` ⪌, `\Rrightarrow` ⇛, `\Lleftarrow` ⇚, `\veebar` ⊻, `\barwedge` ⊼, `\doublebarwedge` ⩞, `\measuredangle` ∡, `\sphericalangle` ∢, `\varpropto` ∝, `\smallsmile` ⌣, `\smallfrown` ⌢, `\Subset` ⋐, `\Supset` ⋑, `\Cup` ⋓, `\Cap` ⋒, `\curlywedge` ⋏, `\curlyvee` ⋎, `\leftthreetimes` ⋋, `\rightthreetimes` ⋌, `\subseteqq` ⫅, `\supseteqq` ⫆, `\bumpeq` ≏, `\Bumpeq` ≎, `\lll` ⋘, `\ggg` ⋙, `\circledS` Ⓢ, `\pitchfork` ⋔, `\dotplus` ∔, `\backsim` ∽, `\backsimeq` ⋍, `\complement` ∁, `\intercal` ⊺, `\circledcirc` ⊚, `\circledast` ⊛, `\circleddash` ⊝, `\lvertneqq` ≨, `\gvertneqq` ≩, `\nleq` ≰, `\ngeq` ≱, `\nless` ≮, `\ngtr` ≯, `\nprec` ⊀, `\nsucc` ⊁, `\lneqq` ≨, `\gneqq` ≩, `\nleqslant` ⩽̸, `\ngeqslant` ⩾̸, `\lneq` ⪇, `\gneq` ⪈, `\npreceq` ⋠, `\nsucceq` ⋡, `\precnsim` ⋨, `\succnsim` ⋩, `\lnsim` ⋦, `\gnsim` ⋧, `\nleqq` ≦̸, `\ngeqq` ≧̸, `\precneqq` ⪵, `\succneqq` ⪶, `\precnapprox` ⪹, `\succnapprox` ⪺, `\lnapprox` ⪉, `\gnapprox` ⪊, `\nsim` ≁, `\ncong` ≇, `\diagup` ⟋, `\diagdown` ⟍, `\varsubsetneq` ⊊, `\varsupsetneq` ⊋, `\nsubseteqq` ⫅̸, `\nsupseteqq` ⫆̸, `\subsetneqq` ⫋, `\supsetneqq` ⫌, `\varsubsetneqq` ⫋, `\varsupsetneqq` ⫌, `\subsetneq` ⊊, `\supsetneq` ⊋, `\nsubseteq` ⊈, `\nsupseteq` ⊉, `\nparallel` ∦, `\nmid` ∤, `\nshortmid` ∤, `\nshortparallel` ∦, `\nvdash` ⊬, `\nVdash` ⊮, `\nvDash` ⊭, `\nVDash` ⊯, `\ntrianglerighteq` ⋭, `\ntrianglelefteq` ⋬, `\ntriangleleft` ⋪, `\ntriangleright` ⋫, `\nleftarrow` ↚, `\nrightarrow` ↛, `\nLeftarrow` ⇍, `\nRightarrow` ⇏, `\nLeftrightarrow` ⇎, `\nleftrightarrow` ↮, `\divideontimes` ⋇, `\nexists` ∄, `\Finv` Ⅎ, `\Game` ⅁, `\eth` ð, `\eqsim` ≂, `\beth` ℶ, `\gimel` ℷ, `\daleth` ℸ, `\lessdot` ⋖, `\gtrdot` ⋗, `\ltimes` ⋉, `\rtimes` ⋊, `\shortmid` ∣, `\shortparallel` ∥, `\smallsetminus` ∖, `\thicksim` ∼, `\thickapprox` ≈, `\approxeq` ≊, `\succapprox` ⪸, `\precapprox` ⪷, `\curvearrowleft` ↶, `\curvearrowright` ↷, `\digamma` ϝ, `\varkappa` ϰ, `\Bbbk` 𝕜, `\hslash` ℏ, `\backepsilon` ϶, `\ulcorner` ⌜, `\urcorner` ⌝, `\llcorner` ⌞, `\lrcorner` ⌟, `\yen` ¥, `\checkmark` ✓, `\circledR` ®, `\maltese` ✠, `\restriction` ↾, `\Doteq` ≑, `\doublecup` ⋓, `\doublecap` ⋒, `\llless` ⋘, `\gggtr` ⋙.

### Operator names

Typeset as upright words: `\sin`, `\cos`, `\tan`, `\cot`, `\sec`, `\csc`, `\arcsin`, `\arccos`, `\arctan`, `\sinh`, `\cosh`, `\tanh`, `\coth`, `\log`, `\ln`, `\lg`, `\exp`, `\lim`, `\liminf`, `\limsup`, `\max`, `\min`, `\sup`, `\inf`, `\det`, `\gcd`, `\deg`, `\dim`, `\ker`, `\arg`, `\hom`, `\Pr`, `\sgn`.

### Environments

| Environment | Mode | Behaviour |
| --- | --- | --- |
| `document` | text | the typeset body; preamble content is not typeset |
| `equation` | text | numbered display |
| `equation*` | text | unnumbered display |
| `table` | text | float body in place; \caption numbers Table n |
| `NoHyper` | text | hyperref: references and links inside are typeset without links |
| `displaymath` | text | unnumbered display |
| `gather` | text | centred rows, each numbered |
| `gather*` | text | centred rows |
| `align` | text | rows aligned at &, each numbered |
| `align*` | text | rows aligned at & |
| `alignat` | text | rows aligned at &, each numbered; the column count is consumed |
| `alignat*` | text | rows aligned at &; the column count is consumed |
| `flalign` | text | rows aligned at &, each numbered |
| `flalign*` | text | rows aligned at & |
| `multline` | text | multi-line display; only the last line is numbered |
| `multline*` | text | multi-line display |
| `subequations` | text | amsmath: displays inside number as the parent number plus a, b, ...; a \label right after \begin gets the parent number |
| `figure` | text | numbered captions; no floating |
| `algorithm` | text | algorithm.sty float: "Algorithm N" captions (plain, ruled, boxed); no floating in this layout |
| `algorithm*` | text | two-column algorithm float |
| `algorithmic` | text | algorithmic or algpseudocode statements: nested blocks, bold keywords, line numbers, comments, procedures |
| `center` | text | centred paragraphs |
| `flushleft` | text | left-aligned paragraphs |
| `flushright` | text | right-aligned paragraphs |
| `quote` | text | indented paragraphs |
| `quotation` | text | indented paragraphs |
| `verse` | text | indented lines; each \\ ends a line |
| `itemize` | text | bulleted list; article labels per depth, \item[label] |
| `enumerate` | text | numbered list; article labels per depth, enumitem label/label*/shortlabels, start and resume |
| `description` | text | list of bold \item[term] labels |
| `tabular` | text | table with l/c/r/p columns, rules and multicolumn; with array also >{} <{} !{} m b w and \extrarowheight |
| `tabular*` | text | table of a given width |
| `verbatim` | text | literal typewriter lines; a tab is one space (latex.ltx \@verbatim) |
| `verbatim*` | text | literal typewriter lines with visible spaces and tabs |
| `lstlisting` | text | listings code; columns, numbers, frames and keywords laid out by the render pipeline |
| `proof` | text | amsthm proof: italic head and an end-of-proof box (\qedhere moves it) |
| `thebibliography` | text | References section with numbered \bibitem entries |
| `minipage` | text | [t\|c\|b][height][t\|c\|b\|s]{width}: vertical box of paragraphs in running text; footnotes are set inline |
| `lrbox` | text | {\name}: stores its body as a horizontal box |
| `multicols` | text | multicol {n}[preface][premulticols]: balanced columns, laid out by the render pipeline |
| `multicols*` | text | multicol {n}[preface][premulticols]: unbalanced columns, laid out by the render pipeline |
| `array` | math | math grid, centred cells |
| `matrix` | math | math grid, centred cells |
| `smallmatrix` | math | math grid, centred cells |
| `pmatrix` | math | math grid, centred cells in ( ) |
| `bmatrix` | math | math grid, centred cells in [ ] |
| `Bmatrix` | math | math grid, centred cells in { } |
| `vmatrix` | math | math grid, centred cells in \| \| |
| `Vmatrix` | math | math grid, centred cells in ‖ ‖ |
| `cases` | math | math grid, left-aligned cells with a left { |
| `dcases` | math | math grid, left-aligned cells with a left { |
| `aligned` | math | math grid, centred cells |
| `alignedat` | math | math grid, centred cells |
| `split` | math | math grid, centred cells |
| `gathered` | math | math grid, centred cells |

### Packages loaded without a warning

| Package | Options | Why it is silent |
| --- | --- | --- |
| `inputenc` | `utf8` | source text is already decoded as UTF-8 |
| `fontenc` | `T1` | text glyphs are mapped from Unicode |
| `color` | `dvipsnames, usenames` | color.sty colours with pdfTeX's exact operator values |
| `xcolor` | `natural, rgb, cmy, cmyk, gray, dvipsnames, svgnames, x11names, table` | xcolor 3.02 definitions, expressions and target models with pdfTeX's exact operator values; hsb models, colour series and table colours are diagnosed |
| `natbib` | `numbers, super, authoryear, round, square, angle, curly, comma, semicolon, colon, sort, compress, sort&compress, longnamesfirst, sectionbib, openbib` | natbib citation commands, punctuation and bibliography labels |
| `amsthm` | `` | \newtheorem, \theoremstyle, \newtheoremstyle, \swapnumbers, the proof environment, \qed and \qedhere |
| `array` | `` | tabular >{} <{} !{} m b w columns, \newcolumntype and \extrarowheight |
| `enumitem` | `shortlabels` | list keys (label, start, resume, seps, margins) parsed as options; \setlist |
| `listings` | `` | lstlisting, \lstinline, \lstset and \lstinputlisting |
| `geometry` | `letterpaper, margin=1in` | matches the fixed US Letter page with 1in margins |
| `hyperref` | `colorlinks, bookmarksnumbered` | links, destinations, bookmarks and PDF info recorded for export; \autoref, \nameref, \hypersetup |
| `siunitx` | `any \sisetup keys` | v3 \num, \unit, \qty, lists, ranges, \ang, \sisetup and \DeclareSIUnit; unmodelled keys are diagnosed |
| `multicol` | `` | multicols and multicols* with preface, \columnbreak, \raggedcolumns (columns set by the render pipeline) |
| `algorithm` | `plain, ruled, boxed, section` | the algorithm float: style, counter reset and float name options |
| `algorithmic` | `noend` | \STATE, \IF, \FOR, \WHILE, \REPEAT, \REQUIRE, \COMMENT |
| `algpseudocode` | `noend` | \State, \If, \For, \While, \Procedure, \Function, \Call, \Comment |
| `algorithmicx` | `` | the layout algpseudocode builds on |

Any other package, or these packages with other options, is recorded and reported as recognised but not implemented.
<!-- END GENERATED supported-latex -->

## Troubleshooting

Diagnostics appear in the app's Problems panel and in `diagnostics[]`. The
common codes and what to do about them:

| Code | Severity | Meaning | What you can do |
|---|---|---|---|
| `compiler` | warning or error | A message from the parser, re-wrapped by `flashtex-render`: `\foo is not supported by this compiler version`, `packages X are recognised but not implemented`, `\setlist keys leftmargin … are recognised but not implemented`, `environment 'X' is not implemented; its body is typeset as plain text`, `undefined reference`, unmatched braces, an `\input` file not found. (`flashtex-compiler` itself emits these without a `code` field.) | Remove or replace the construct; the `recovery` text says what was rendered instead |
| `overfull_hbox` | warning | `overfull line: N pt too wide (no hyphenation available)` — a line could not be broken within the text width, so it sticks into the margin like TeX's *Overfull \hbox* | Rephrase or add a break point; hyphenation is not implemented |
| `overfull_vbox` | warning | A line extends past the page's text area | Shorten the page or force a break with `\newpage` |
| `overfull_display` | warning | Display math is wider than the text width by N pt | Split the formula |
| `paragraph_final_linebreak` | warning | A trailing `\\` at the end of a paragraph was ignored (LaTeX would set an empty last line; this paragraph is one line pitch shorter) | Remove the trailing `\\` |
| `math_resource_profile` | warning (no source) | Glyphs of a TeX math family (`lmmi10`, `lmsy10`, `lmex10`) are drawn from the single-design Latin Modern Math font because no optical-size outline resource exists; outlines differ slightly from the pdfLaTeX design | Informational — positions are still from TeX metrics |
| `math_limitation` | warning | A math construct is laid out with a documented simplification: `align`/`gather` rows set as separate centred displays with `&` ignored, a nested `array`/`cases`/`matrix` inside a sub-formula set as one row, `\quad` glue inside `\left…\right` dropped, a delimiter or radical variant that is too small, a missing math glyph or accent | Informational; restructure if the simplification is visible |
| `math_glyph_unmapped` | warning | TeX metrics placed a glyph (an extensible assembly) that has no Latin Modern Math mapping; nothing was drawn | Use a smaller delimiter |
| `math_text_overflow` | error | Too many `\text{}` arguments in one formula; the extra ones were not typeset | Split the formula |
| `unsupported_block` | error | Historical: a block-level construct without a layout. Current `flashtex-render` builds do not emit it (unknown environments come through as `compiler` warnings instead) | — |
| `unsupported_script` | error | Text in a script the shaper does not handle (non-Latin); the segment was not drawn | Not supported yet |
| `missing_glyph` | warning | A character has no glyph in the selected Latin Modern face; nothing was drawn for it | Use a supported character or `\text{}` with Latin text |
| `font_unavailable` / `math_font_unavailable` | error | The Latin Modern text face (Times metrics substituted) or Latin Modern Math (no math typeset) could not be found | Check the font search path above; with the bundled app this indicates a broken install |
| `tfm_missing` / `math_metrics_opentype` / `tfm_run_error` | warning | A `.tfm` metric file was not found (or its lig/kern program failed on a word), so OpenType advances were used for that face or word | Point `FLASHTEX_TFM_DIRS` at a Latin Modern `texmf/fonts/tfm/public/lm` directory, or use the bundled app, which ships the metrics for 5–17 pt roman, 5–12 pt bold and 7–12 pt italic |
| `required_metrics_unavailable` | error | The pinned required metric set (`ec-lmr12`, `rm-lmr12/8/6`, licence) is missing or altered; layout is not reference geometry | Reinstall; the bundle is verified at packaging time |
| `labels_unstable` | warning (no source) | `\ref`/`\pageref` values did not converge within the pass limit; the last pass is shown | Check for labels whose value depends on their own page position |
| `display_list_declined` | warning (no source) | The `display-list-v2` sibling would exceed the 16 MiB reply limit, so the app's v2 preview receives no frame for this revision (the v1 pages still render) | Split the project into `\input` files |
| `payload_too_large`, `malformed_json`, `invalid_utf8`, `unsupported_protocol_version`, `unsupported_type` | `error` envelope | The request line itself was rejected (over 8 MiB, not JSON, …); no `compile_result` is produced and the worker keeps running | Fix the request; split very large documents |

When a `compile_result` is `recovered`, the pages are real output rendered
around the problem; when it is `failed` there are no pages and the app keeps
the last good preview on screen, with the old underlines flagged as kept from
the previous revision.
