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

This compiler implements a finite LaTeX subset: 106 text-mode and 305 math-mode command entries, 43 environments and 5 layout-neutral packages. Every other command produces an explicit "not supported" diagnostic naming it, and every other environment or package a warning; nothing is dropped silently. Descriptions note approximations. Outstanding features with reproductions are in `crates/compiler/UNSUPPORTED.md`.

Regenerate with `crates/compiler/scripts/render_supported_latex.sh`; `cargo test --test supported_latex` fails when this section is stale.

### Coverage

| Canonical set | Commands supported | Environments supported |
| --- | ---: | ---: |
| kernel | 104/410 (25.4%) | 15/30 (50.0%) |
| amsmath | 33/102 (32.4%) | 17/21 (81.0%) |
| amssymb | 20/229 (8.7%) | none defined |
| enumitem | 1/17 (5.9%) | none defined |
| geometry | 0/7 (0.0%) | none defined |
| graphicx | 0/8 (0.0%) | none defined |
| hyperref | 2/132 (1.5%) | 0/2 (0.0%) |
| tikz | 0/43 (0.0%) | 0/2 (0.0%) |
| **total** | **192/1003 (19.1%)** | |

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

### Text commands

| Command | Arguments | Behaviour |
| --- | --- | --- |
| `\section` | `{...}` | numbered section heading; starred form unnumbered |
| `\subsection` | `{...}` | numbered subsection heading; starred form unnumbered |
| `\subsubsection` | `{...}` | numbered subsubsection heading; starred form unnumbered |
| `\tableofcontents` |  | article contents list from the previous layout pass |
| `\textbf` | `{...}` | bold text |
| `\textmd` | `{...}` | medium-weight text |
| `\emph` | `{...}` | emphasis: toggles italic |
| `\textit` | `{...}` | italic text |
| `\textsl` | `{...}` | slanted text (typeset as italic) |
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
| `\setlist` | `[list]{options}` | enumitem itemsep and topsep; other keys warn |
| `\newcommand` | `{\name}[n]{body}` | defines a macro with 0-9 arguments; rejects an existing name |
| `\renewcommand` | `{\name}[n]{body}` | redefines an existing macro |
| `\DeclareMathOperator` | `*{\name}{text}` | defines \name as \operatorname{text}; the starred form takes limits |
| `\input` | `{path}` | expands a project-relative document in place |
| `\include` | `{path}` | expands a project-relative document in place |
| `\label` | `{key}` | names the current section, equation or figure number |
| `\ref` | `{key}` | number of the labelled item |
| `\pageref` | `{key}` | page number of the labelled item |
| `\eqref` | `{key}` | parenthesised equation number of the labelled item |
| `\caption` | `{...}` | numbered "Figure N:" caption inside figure |
| `\item` |  | entry of an itemize or enumerate list |
| `\url` | `{url}` | monospaced URL text; links are not clickable |
| `\href` | `{url}{text}` | link text; links are not clickable |
| `\nolinkurl` | `{url}` | monospaced URL text without a link |
| `\hfill` |  | infinite-stretch horizontal glue |
| `\hfil` |  | infinite-stretch horizontal glue (same order as \hfill) |
| `\hspace` | `{dimension}` | fixed horizontal space; starred form identical |
| `\footnote` | `[n]{...}` | numbered mark and page-bottom footnote text |
| `\footnotemark` | `[n]` | footnote mark only |
| `\footnotetext` | `[n]{...}` | footnote text without a mark |
| `\normalfont` |  | resets the text face |
| `\bfseries` |  | switches to bold |
| `\mdseries` |  | switches to medium weight |
| `\itshape` |  | switches to italic |
| `\slshape` |  | switches to slanted (typeset as italic) |
| `\upshape` |  | switches to upright |
| `\ttfamily` |  | switches to typewriter |
| `\rmfamily` |  | switches to roman |
| `\sffamily` |  | switches to sans-serif |
| `\em` |  | toggles emphasis |
| `\bf` |  | LaTeX 2.09 form: bold roman |
| `\it` |  | LaTeX 2.09 form: italic roman |
| `\sl` |  | LaTeX 2.09 form: slanted roman |
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
| `\cite` | `[note]{keys}` | numbered citation from thebibliography entries |
| `\nocite` | `{keys}` | accepted no-op; there is no .bib pipeline |
| `\bibitem` | `[label]{key}` | entry of thebibliography |
| `\bibliography` | `{files}` | diagnosed: .bib input is not read |
| `\bibliographystyle` | `{style}` | diagnosed: no effect without .bib support |
| `\title` | `{...}` | title for \maketitle |
| `\author` | `{...}` | author block for \maketitle; \and and \thanks inside it |
| `\date` | `{...}` | date for \maketitle; \today inside it |
| `\maketitle` |  | article.cls title block |
| `\newtheorem` | `{env}[counter]{name}` | defines a numbered theorem-like environment (amsthm) |
| `\theoremstyle` | `{style}` | selects the amsthm style for following \newtheorem |
| `\\` |  | line break; an optional [length] is consumed |

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
| `\frac` | `{num}{den}` | fraction; \cfrac lays out as \frac |
| `\cfrac` | `{num}{den}` | fraction; \cfrac lays out as \frac |
| `\dfrac` | `{num}{den}` | amsmath \genfrac fraction in display or text style |
| `\tfrac` | `{num}{den}` | amsmath \genfrac fraction in display or text style |
| `\genfrac` | `{left}{right}{thickness}{style}{num}{den}` | amsmath generalized fraction: delimiters, pt rule thickness and a 0-3 style |
| `\phantom` | `{x}` | empty box with the width and/or height and depth of the argument |
| `\hphantom` | `{x}` | empty box with the width and/or height and depth of the argument |
| `\vphantom` | `{x}` | empty box with the width and/or height and depth of the argument |
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
| `\mathrm` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
| `\mathit` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
| `\mathsf` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
| `\mathtt` | `{...}` | keeps its argument in the current math face (no distinct face yet) |
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
| `\hat` | `{body}` | base-14 accent glyph centred over the body |
| `\bar` | `{body}` | base-14 accent glyph centred over the body |
| `\vec` | `{body}` | base-14 accent glyph centred over the body |
| `\tilde` | `{body}` | base-14 accent glyph centred over the body |
| `\dot` | `{body}` | base-14 accent glyph centred over the body |
| `\ddot` | `{body}` | base-14 accent glyph centred over the body |
| `\acute` | `{body}` | base-14 accent glyph centred over the body |
| `\grave` | `{body}` | base-14 accent glyph centred over the body |
| `\widehat` | `{body}` | unstretched accent; warns over more than one symbol |
| `\widetilde` | `{body}` | unstretched accent; warns over more than one symbol |
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
| `\tag` | `{label}` | (label) two quads after the display; starred form without parentheses |
| `\begin` | `{env}` | opens a math grid environment |

### Math symbols

`\alpha` α, `\beta` β, `\gamma` γ, `\delta` δ, `\theta` θ, `\lambda` λ, `\mu` μ, `\pi` π, `\sigma` σ, `\phi` φ, `\omega` ω, `\epsilon` ε, `\varepsilon` ε, `\zeta` ζ, `\eta` η, `\vartheta` ϑ, `\iota` ι, `\kappa` κ, `\nu` ν, `\xi` ξ, `\varpi` ϖ, `\rho` ρ, `\varsigma` ς, `\tau` τ, `\upsilon` υ, `\varphi` ϕ, `\chi` χ, `\psi` ψ, `\Gamma` Γ, `\Delta` Δ, `\Theta` Θ, `\Lambda` Λ, `\Xi` Ξ, `\Pi` Π, `\Sigma` Σ, `\Upsilon` Υ, `\Phi` Φ, `\Psi` Ψ, `\Omega` Ω, `\le` ≤, `\ge` ≥, `\ne` ≠, `\equiv` ≡, `\sim` ∼, `\cong` ≅, `\propto` ∝, `\perp` ⊥, `\partial` ∂, `\nabla` ∇, `\prod` ∏, `\ast` ∗, `\prime` ′, `\cup` ∪, `\cap` ∩, `\subset` ⊂, `\subseteq` ⊆, `\supset` ⊃, `\supseteq` ⊇, `\notin` ∉, `\ni` ∋, `\emptyset` ∅, `\oplus` ⊕, `\otimes` ⊗, `\wedge` ∧, `\land` ∧, `\lor` ∨, `\to` →, `\rightarrow` →, `\leftarrow` ←, `\gets` ←, `\uparrow` ↑, `\downarrow` ↓, `\leftrightarrow` ↔, `\Leftarrow` ⇐, `\Leftrightarrow` ⇔, `\Uparrow` ⇑, `\Downarrow` ⇓, `\therefore` ∴, `\angle` ∠, `\aleph` ℵ, `\Re` ℜ, `\Im` ℑ, `\wp` ℘, `\langle` 〈, `\rangle` 〉, `\lvert` ∣, `\rvert` ∣, `\lVert` ∣∣, `\rVert` ∣∣, `\times` ×, `\div` ÷, `\pm` ±, `\leq` ≤, `\geq` ≥, `\neq` ≠, `\approx` ≈, `\cdot` ⋅, `\infty` ∞, `\sum` ∑, `\int` ∫, `\in` ∈, `\forall` ∀, `\exists` ∃, `\vee` ∨, `\Rightarrow` ⇒, `\mid` ∣, `\setminus` ∖, `\Longrightarrow` ⟹, `\mp` ∓, `\ll` ≪, `\gg` ≫, `\simeq` ≃, `\vdots` ⋮, `\ddots` ⋱, `\lfloor` ⌊, `\rfloor` ⌋, `\lceil` ⌈, `\rceil` ⌉, `\oint` ∮, `\mapsto` ↦, `\ell` ℓ, `\hbar` ℏ, `\circ` ∘, `\parallel` ∥, `\nmid` ∤, `\nleq` ≰, `\ngeq` ≱, `\subsetneq` ⊊, `\supsetneq` ⊋, `\lesssim` ≲, `\gtrsim` ≳, `\triangleq` ≜, `\coloneqq` ≔, `\nexists` ∄, `\complement` ∁, `\rightsquigarrow` ⇝, `\hookrightarrow` ↪, `\leftrightarrows` ⇆, `\models` ⊨, `\vdash` ⊢, `\dashv` ⊣, `\top` ⊤, `\measuredangle` ∡, `\square` □, `\blacksquare` ■, `\lozenge` ◊, `\checkmark` ✓, `\Longleftrightarrow` ⟺, `\longrightarrow` ⟶, `\longleftarrow` ⟵, `\Longleftarrow` ⟸, `\longleftrightarrow` ⟷, `\triangle` △, `\bigtriangledown` ▽.

### Operator names

Typeset as upright words: `\sin`, `\cos`, `\tan`, `\cot`, `\sec`, `\csc`, `\arcsin`, `\arccos`, `\arctan`, `\sinh`, `\cosh`, `\tanh`, `\coth`, `\log`, `\ln`, `\lg`, `\exp`, `\lim`, `\liminf`, `\limsup`, `\max`, `\min`, `\sup`, `\inf`, `\det`, `\gcd`, `\deg`, `\dim`, `\ker`, `\arg`, `\hom`, `\Pr`, `\sgn`.

### Environments

| Environment | Mode | Behaviour |
| --- | --- | --- |
| `document` | text | the typeset body; preamble content is not typeset |
| `equation` | text | numbered display |
| `equation*` | text | unnumbered display |
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
| `figure` | text | numbered captions; no floating |
| `center` | text | centred paragraphs |
| `flushleft` | text | left-aligned paragraphs |
| `flushright` | text | right-aligned paragraphs |
| `quote` | text | indented paragraphs |
| `quotation` | text | indented paragraphs |
| `itemize` | text | bulleted list |
| `enumerate` | text | numbered list; enumitem [label] templates a, A, i, I, 1 |
| `tabular` | text | table with l/c/r/p columns, rules and multicolumn |
| `tabular*` | text | table of a given width |
| `verbatim` | text | literal monospaced lines |
| `verbatim*` | text | literal monospaced lines with visible spaces |
| `lstlisting` | text | literal monospaced lines (basic listings) |
| `proof` | text | amsthm proof with a closing square |
| `thebibliography` | text | References section with numbered \bibitem entries |
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
| `amsthm` | `` | \newtheorem, \theoremstyle and the proof environment |
| `enumitem` | `shortlabels` | enumerate label templates; \setlist itemsep/topsep |
| `geometry` | `letterpaper, margin=1in` | matches the fixed US Letter page with 1in margins |

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
