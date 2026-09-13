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

## Supported LaTeX

This is an original engine that implements a growing subset of LaTeX. Anything
outside the subset produces an explicit diagnostic — never silent output.
Details and the exact evidence live in `crates/compiler/README.md` (parser and
command set) and `crates/render-pipeline/README.md` (layout, fonts, math);
this is the condensed version, checked against the bundled `flashtex-render`.

**Document structure.** `\documentclass[10pt|11pt|12pt,…]{article}` (US Letter;
`geometry` margins are honoured), `\usepackage{…}` (recorded; packages are not
loaded — one warning lists the unimplemented ones), `\begin{document}` …
`\end{document}`, `%` comments, blank-line paragraphs, `\par`, `\\`, `~`,
`\noindent`, `\newpage`/`\clearpage`/`\pagebreak`, `\section`/`\subsection`
with LaTeX numbering, `\label`/`\ref`/`\pageref` (multi-pass convergence,
`??` for undefined), `\input{file}` across the documents in the request,
`\newcommand`/`\renewcommand` with 0–9 arguments (scoped, 64-deep expansion
limit), `\setlength{\parskip}`/`\parindent` in the preamble.

**Text.** `\textbf`, `\emph`, `\textit`, `\textsl`, `\texttt`, `\textrm`,
`\textsf`, `\textmd`, `\textup`, `\textnormal`, the declarations `\bfseries`,
`\itshape`, `\ttfamily`, `\normalfont`, `\em`, … and `\bf`/`\it`/`\tt`, size
declarations `\tiny` … `\Huge` (real `size1x.clo` tables), `\hfill`/`\hfil`,
`\hspace{…}`, `\quad`/`\qquad`, `\bigskip`/`\medskip`/`\smallskip`, TeX
ligatures (`--`, `---`, quotes), `\'e`-style accents, justified paragraphs
with TeX's line-breaking cost model. Lists: `itemize`, `enumerate` with
LaTeX's `\list` geometry; `\setlist[…]{itemsep=…,topsep=…}` spacing.
`figure` with `\caption` is laid out in source order (no floats, no images).

**Math.** `$…$`, `\[…\]`, `$$…$$`, `equation` (numbered, flush-right `(n)`),
`matrix`/`pmatrix`/`bmatrix`/`vmatrix`/`Vmatrix`, `cases`, `array`,
`split`/`aligned`/`alignedat`/`gathered` grids; `align`/`gather`/`alignat`/
`flalign`/`multline` are accepted but each row is set as its own centred
display and `&` alignment points are ignored (a `math_limitation` warning);
`\frac` (`\dfrac`, `\tfrac`, `\cfrac`), `\sqrt[n]{…}`, `^`/`_` with braces,
`\binom`, `{n \choose k}`, `{a \over b}`, `\left`/`\right` and `\big…\Bigg`
delimiters (sized through the `lmex` chain), `\text{…}`, `\mathrm` and
friends, `\mathbb{A–Z}`, `\boxed`, `\overline`/`\underline`, `\overset`/
`\underset`/`\stackrel`, `\tag`, `\pmod`, accents (`\hat`, `\bar`, `\vec`,
`\tilde`, `\dot`, `\ddot`, …), the Greek alphabet, the usual binary/relation
symbols (`\times`, `\cdot`, `\pm`, `\leq`, `\neq`, `\approx`, `\equiv`, `\in`,
`\notin`, `\subset`, `\cup`, `\cap`, `\setminus`, `\angle`, `\to`, `\implies`,
`\iff`, …), `\sum`/`\prod`/`\int` with display limits, `\lim`, `\sin` …
`\operatorname{…}`, `\ldots`/`\cdots`, math spacing `\,` `\;` `\!` and TeX's
inter-atom spacing. Paragraph environments `center`, `flushleft`,
`flushright`, `quote`, `quotation`.

**Recognised but not implemented (reported, not approximated silently).**
Hyphenation (long words overflow the margin with an `overfull_hbox` warning);
`\def`, `\let`, catcodes, registers and conditionals; package commands in
general — `amsmath`/`amssymb`/`amsthm` (`\newtheorem` and theorem
environments), `microtype`, `hyperref`, `enumitem` keys other than
`itemsep`/`topsep` (e.g. **`\setlist leftmargin`**, `label`); `\includegraphics`
and images; `tabular` and tables (the body is typeset as plain text);
`\footnote`; bibliographies and `\cite`; page headers and page numbers;
two-column layout; `\oint`, `\mapsto`, `\mp`, `\ll`, `\gg`, `\lfloor`,
`\lceil`, `\vdots`, `\ddots`, `\ell`, `\hbar`; non-Latin scripts and RTL;
sans, typewriter and small-caps faces beyond metric-only substitution.
Environments the engine does not know warn and typeset their body as plain
text; unknown commands are reported and, when their argument looks like a
parameter, skipped.

The list above was checked by rendering a probe document with the bundled
`flashtex-render` (`pmatrix`, `cases`, `array`, `center`, `\angle`, `\mathbb`
pass cleanly; `align`, `\cite`, `\footnote`, `\newtheorem`, `tabular`,
`\includegraphics`, `\oint` produce the diagnostics described).

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
