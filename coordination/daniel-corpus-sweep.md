# Corpus sweep report — daniel-corpus-sweep

Measurement lane only. No compiler code was changed. This is input to the
post-demo compiler priority list, not a fix.

## Scope and method

- Corpus: `tests/extended-tex-corpus/` (49 original projects, one `main.tex`
  entry point each, per `manifest.json`), from
  `origin/agent/mac-reference-corpus/extended-suite` (tip `dfe4756f`), read via
  `git archive` into a scratch directory — **not merged** into this branch, so
  this report's commit stays report-only.
- Compiler under test: `crates/compiler` (`flashtex-compiler` CLI, JSON Lines
  runtime-v1 protocol), built once in release mode
  (`cargo build --release --manifest-path crates/compiler/Cargo.toml`).
- Base: `origin/main` (`1275473b`) merged with `origin/agent/daniel-math-amsmath/compiler`
  (`a434caec`) — **measured on main + amsmath**, per setup instructions. Merge
  was a clean `ort` merge, no conflicts.
- For each of the 49 projects, all UTF-8-decodable project files (per
  `manifest.json`'s `files` list — handles the 7 multi-file projects:
  chapters, `.sty`, `.cls`, `.bib`, `data.tsv`) were packed into one runtime-v1
  `compile` request and piped to the CLI once. The one binary asset in the
  corpus (`graphics-included-raster/assets/quadrants.png`) was omitted from
  the request body (the JSON Lines transport has no binary-asset field —
  same limitation the corpus's own `reference.py --emit-request` documents);
  everything else in that project was still sent.
- Per-file timeout: 30s, enforced with Python's `subprocess.run(..., timeout=30)`
  (kills and reports `TimeoutExpired`) rather than the suggested
  `perl -e 'alarm 30; exec @ARGV'` — neither GNU `timeout` nor `gtimeout` is
  installed on this Mac, and driving the sweep from Python made the native
  timeout mechanism the simpler choice; it enforces the same 30s bound.
- Diagnostic messages were normalized before aggregation: any `\command` token
  collapsed to `\CMD` (and `\begin{env}`/`\end{env}` to `\begin{ENV}`/`\end{ENV}`),
  any `'X' (U+HHHH)` character citation collapsed to `'C' (U+XXXX)`, font names
  before "has no glyph for" collapsed to `FONT`, bare integers to `N`. Line
  numbers were never an issue — this protocol reports `{path, start_byte,
  end_byte}` structurally, never inline in the message text.

## Top-line counts

| Outcome | Projects |
| --- | --- |
| Fully clean (`status: ok`, zero diagnostics) | **0** |
| Recovered (diagnostics emitted, page content still produced) | **49** |
| Failed (`status: failed`, no content) | **0** |
| Crashed (nonzero exit / signal / unparseable output) | **0** |
| Timed out (>30s) | **0** |

All 49 runs completed in 2–4ms; no performance concern. Every project — including
all 7 of the corpus's own negative/error fixtures — comes back `recovered`: the
compiler never hard-fails or crashes on this corpus, but it also never
cleanly accepts a single project without at least one diagnostic. 758 total
diagnostics were emitted across the corpus, normalizing to 54 distinct message
templates.

Of the 7 designed error cases, 6 surfaced a diagnostic that plausibly matches
the intended defect (own wording, not MacTeX's — expected per the corpus
README) — `error-missing-math-delimiter`'s only specific diagnostic is
`\left is not supported in math mode` (it never gets far enough to notice the
missing `\right`). **`error-extra-math-align` produced no diagnostic beyond the
two boilerplate ones (`\listfiles`, amsmath-unimplemented)** — the compiler
silently did not detect the misplaced alignment tab this fixture is designed
to exercise. Minimal repro (from `tests/extended-tex-corpus/cases/error-extra-math-align/main.tex`):
```tex
\documentclass{article}
\begin{document}
$a & b$
\end{document}
```
Expected: a "misplaced alignment tab character" diagnostic. Observed: no
diagnostic referencing the stray `&`; the request still comes back `recovered`
only because of the two boilerplate items above.

## (b) Per-project diagnostic counts and status

All 49 are `status: recovered`. Sorted by diagnostic count (high to low):

| Project | Engine | Corpus expect | Diagnostics |
| --- | --- | --- | --- |
| unicode-lualatex | lualatex | success | 49 |
| unicode-xelatex | xelatex | success | 48 |
| tex-scope-registers | pdflatex | success | 45 |
| math-greek-alphabets | pdflatex | success | 41 |
| tex-token-registers-conditionals | pdflatex | success | 35 |
| math-atom-classes | pdflatex | success | 33 |
| boxes-rules-leaders | pdflatex | success | 31 |
| math-accents-overlays | pdflatex | success | 29 |
| tex-expansion-arguments | pdflatex | success | 29 |
| math-radicals-delimiters | pdflatex | success | 28 |
| plain-tex-halign | pdftex | success | 26 |
| math-scripts-limits | pdflatex | success | 24 |
| graphics-transform-color | pdflatex | success | 23 |
| math-mathtools | pdflatex | success | 21 |
| packages-siunitx-chemistry | pdflatex | success | 20 |
| text-ligatures-accents | pdflatex | success | 17 |
| tikz-nodes-graphs | pdflatex | success | 17 |
| tables-longtable | pdflatex | success | 15 |
| paragraph-glue-penalties | pdflatex | success | 14 |
| tables-spans-rules | pdflatex | success | 14 |
| tex-discretionary-italic | pdflatex | success | 14 |
| math-fractions-styles | pdflatex | success | 13 |
| math-matrices-cases | pdflatex | success | 13 |
| tikz-clipping-patterns | pdflatex | success | 12 |
| tikz-paths-transforms | pdflatex | success | 11 |
| tex-catcodes-active | pdflatex | success | 10 |
| pgfplots-functions-data | pdflatex | success | 10 |
| math-array-spacing | pdflatex | success | 10 |
| document-floats-footnotes | pdflatex | success | 9 |
| project-book-include | pdflatex | success | 9 |
| lists-theorems-proofs | pdflatex | success | 8 |
| hyperref-bookmarks | pdflatex | success | 8 |
| math-align-points | pdflatex | success | 7 |
| math-numbering-subequations | pdflatex | success | 7 |
| math-split-gather-multline | pdflatex | success | 6 |
| packages-listings-verbatim | pdflatex | success | 6 |
| project-biblatex-biber | pdflatex | success | 6 |
| graphics-included-raster | pdflatex | success | 6 |
| project-bibtex | pdflatex | success | 5 |
| project-input-package | pdflatex | success | 3 |
| error-unknown-command | pdflatex | error | 3 |
| error-unclosed-group | pdflatex | error | 3 |
| error-missing-input | pdflatex | error | 3 |
| error-double-superscript | pdflatex | error | 3 |
| error-mismatched-environment | pdflatex | error | 3 |
| error-missing-math-delimiter | pdflatex | error | 3 |
| packages-local-class | pdflatex | success | 3 |
| text-microtype-multicol | pdflatex | success | 3 |
| error-extra-math-align | pdflatex | error | 2 |

(`graphics-included-raster`'s 6 diagnostics exclude the PNG asset, which the
transport can't carry — see Scope and method.)

## (c) Crashes, timeouts, malformed output

**None.** Zero crashes, zero timeouts, zero unparseable responses across all
49 runs. The compiler is defensive on this corpus — every unsupported
construct degrades to a diagnostic plus a recovery note rather than aborting.

## (a) Per-message aggregate (all 54 normalized templates)

758 raw diagnostics → 54 templates. Full list, sorted by projects affected
then occurrences:

| # | Projects | Occurrences | Normalized message |
|---|---|---|---|
| 1 | 48 | 63 | `\CMD is not supported in the document preamble` |
| 2 | 29 | 325 | `\CMD is not supported by this compiler version; unrestricted TeX math mode is not implemented` |
| 3 | 17 | 17 | `packages amsmath, amssymb are recognised but not implemented` |
| 4 | 16 | 189 | `\CMD is not supported in math mode` |
| 5 | 6 | 6 | `'C' (U+XXXX) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet` |
| 6 | 4 | 55 | `FONT has no glyph for 'C' (U+XXXX)` |
| 7 | 4 | 6 | `math script marker used outside math mode` |
| 8 | 4 | 5 | `environment 'tikzpicture' is not implemented; its body is typeset as plain text` |
| 9 | 3 | 31 | `'C' (U+XXXX) will not survive PDF export: no glyph for this character exists in the base-N PDF fonts` |
| 10 | 3 | 3 | `\CMD requires a braced math argument` |
| 11 | 3 | 3 | `packages tikz are recognised but not implemented` |
| 12 | 2 | 3 | `\CMD{ENV} is not supported in math mode` |
| 13 | 2 | 3 | `environment 'tabular' is not implemented; its body is typeset as plain text` |
| 14 | 2 | 2 | `environment 'minipage' is not implemented; its body is typeset as plain text` |
| 15 | 2 | 2 | `environment 'scope' is not implemented; its body is typeset as plain text` |
| 16 | 2 | 2 | `packages fontspec, unicode-math are recognised but not implemented` |
| 17 | 1 | 4 | `\CMD is unsupported; image loading is not implemented` |
| 18 | 1 | 3 | `\CMD requires a braced argument` |
| 19 | 1 | 1 | `environment 'alignat' is not implemented; its body is typeset as plain text` |
| 20 | 1 | 1 | `environment 'flalign*' is not implemented; its body is typeset as plain text` |
| 21 | 1 | 1 | `environment 'multline' is not implemented; its body is typeset as plain text` |
| 22 | 1 | 1 | `packages mathtools are recognised but not implemented` |
| 23 | 1 | 1 | `packages fontenc are recognised but not implemented` |
| 24 | 1 | 1 | `packages array, multirow, booktabs are recognised but not implemented` |
| 25 | 1 | 1 | `packages longtable, booktabs are recognised but not implemented` |
| 26 | 1 | 1 | `environment 'longtable' is not implemented; its body is typeset as plain text` |
| 27 | 1 | 1 | `\CMD is only supported inside a figure environment` |
| 28 | 1 | 1 | `packages amsmath, amsthm, enumitem are recognised but not implemented` |
| 29 | 1 | 1 | `environment 'description' is not implemented; its body is typeset as plain text` |
| 30 | 1 | 1 | `\CMD is only supported inside itemize or enumerate` |
| 31 | 1 | 1 | `environment 'theorem' is not implemented; its body is typeset as plain text` |
| 32 | 1 | 1 | `environment 'proof' is not implemented; its body is typeset as plain text` |
| 33 | 1 | 1 | `packages xcolor, graphicx are recognised but not implemented` |
| 34 | 1 | 1 | `environment 'picture' is not implemented; its body is typeset as plain text` |
| 35 | 1 | 1 | `packages pgfplots are recognised but not implemented` |
| 36 | 1 | 1 | `environment 'axis' is not implemented; its body is typeset as plain text` |
| 37 | 1 | 1 | `packages siunitx are recognised but not implemented` |
| 38 | 1 | 1 | `packages mhchem are recognised but not implemented` |
| 39 | 1 | 1 | `packages listings are recognised but not implemented` |
| 40 | 1 | 1 | `environment 'verbatim' is not implemented; its body is typeset as plain text` |
| 41 | 1 | 1 | `environment 'lstlisting' is not implemented; its body is typeset as plain text` |
| 42 | 1 | 1 | `packages localstyle are recognised but not implemented` |
| 43 | 1 | 1 | `packages biblatex are recognised but not implemented` |
| 44 | 1 | 1 | `packages hyperref are recognised but not implemented` |
| 45 | 1 | 1 | `unmatched '}' in math mode` |
| 46 | 1 | 1 | `unmatched '{' — group never closed` |
| 47 | 1 | 1 | `included file not found: looked for 'absent-fixture-file' and 'absent-fixture-file.tex'` |
| 48 | 1 | 1 | `duplicate script on a math atom` |
| 49 | 1 | 1 | `\CMD{ENV} does not match \CMD{ENV}` |
| 50 | 1 | 1 | `packages amsmath are recognised but not implemented` |
| 51 | 1 | 1 | `environment 'subequations' is not implemented; its body is typeset as plain text` |
| 52 | 1 | 1 | `packages microtype, multicol are recognised but not implemented` |
| 53 | 1 | 1 | `environment 'multicols' is not implemented; its body is typeset as plain text` |
| 54 | 1 | 1 | `packages graphicx are recognised but not implemented` |

Rows 45–48 are the diagnostics from the corpus's own negative fixtures that
did land (unclosed group, missing input, double superscript, general
unknown-command) — expected hits, not compiler defects.

## Top 25 causes, ranked and labeled

Ranked by projects affected, then occurrences. Rows 1, 2 and 4 are themselves
made of many distinct commands collapsed by normalization — the dominant
underlying command(s) are called out beneath each.

| Rank | Projects | Occurrences | Cause | Category |
|---|---|---|---|---|
| 1 | 48 | 63 | `\CMD` not supported in preamble — **48/48 of this bucket is `\listfiles` alone** (the corpus generator puts it in almost every fixture's preamble for package-version logging; 2× each for `\usetikzlibrary`, `\setmainfont`, `\setmathfont`) | environment |
| 2 | 29 | 325 | Generic "unrestricted TeX math mode is not implemented" catch-all — top underlying commands: `\hbox`(14), `\quad`(13), `\counter`(12), `\the`(12), `\def`(11), `\draw`(10), `\noindent`(9), `\eqref`(4), `\advance`(4), `\else`/`\fi`/`\cr`(4 each), `\midrule`(4), `\chapter`(3), `\tableofcontents`(3), `\ce`(3) | environment (mixed: TeX plumbing + misc unimplemented commands spanning every other category) |
| 3 | 17 | 17 | `\usepackage{amsmath, amssymb}` recognised, not implemented | package |
| 4 | 16 | 189 | `\CMD` not supported in math mode — top: `\left`(6), `\right`(5), `\le`(4), `\mathrm`(4), `\prime`(3), `\to`(3), `\begin`/`\end`(3 each), `\ne`/`\equiv`/`\ge`(2 each), `\Gamma`/`\mathbb`/`\cdots`/`\vdots`/`\abs`/`\cr`/`\symbb`/`\ni`/`\mathchoice`(2 each) | math symbol |
| 5 | 6 | 6 | Fraction rule exported as a box-drawing stand-in character | other (export/rendering) |
| 6 | 4 | 55 | Base-14 PDF fonts (Times-Roman, Symbol) have no glyph for a character | other (font/export) |
| 7 | 4 | 6 | Math script marker (`^`/`_`) used outside math mode | math symbol |
| 8 | 4 | 5 | `tikzpicture` environment not implemented (typeset as plain text) | graphics/tikz |
| 9 | 3 | 31 | PDF-export character has no base-14 glyph (Greek/Cyrillic/symbol chars) | other (font/export) |
| 10 | 3 | 3 | `\sqrt`/`\frac` require a braced math argument (braceless syntax rejected) | math symbol |
| 11 | 3 | 3 | `\usepackage{tikz}` recognised, not implemented | package |
| 12 | 2 | 3 | `\CMD{ENV}` not supported in math mode (e.g. `\begin{pmatrix*}`) | math symbol |
| 13 | 2 | 3 | `tabular` environment not implemented | text layout |
| 14 | 2 | 2 | `minipage` environment not implemented | text layout |
| 15 | 2 | 2 | `scope` (TikZ) environment not implemented | graphics/tikz |
| 16 | 2 | 2 | `\usepackage{fontspec, unicode-math}` recognised, not implemented | package |
| 17 | 1 | 4 | Image-loading command unsupported (`\includegraphics`) | graphics/tikz |
| 18 | 1 | 3 | Custom macro `\opt` requires a braced argument | environment |
| 19 | 1 | 1 | `alignat` environment not implemented | math symbol |
| 20 | 1 | 1 | `flalign*` environment not implemented | math symbol |
| 21 | 1 | 1 | `multline` environment not implemented | math symbol |
| 22 | 1 | 1 | `\usepackage{mathtools}` recognised, not implemented | package |
| 23 | 1 | 1 | `\usepackage{fontenc}` recognised, not implemented | package |
| 24 | 1 | 1 | `\usepackage{array, multirow, booktabs}` recognised, not implemented | package |
| 25 | 1 | 1 | `\usepackage{longtable, booktabs}` recognised, not implemented | package |

## Reading this for compiler priority

- Rank 1 (`\listfiles`, 48/49 projects) is the single highest-leverage fix in
  the corpus and looks cheap: it's a no-op logging command for MacTeX;
  recognizing and ignoring it in the preamble would immediately reduce 48
  projects' diagnostic count by one each.
- Rank 2 is not one fix — it's ~29 projects hitting a shared fallback path for
  dozens of different unimplemented primitives (box/glue primitives, register
  arithmetic, table rules, `\eqref`, sectioning). It's a symptom of breadth-of-coverage,
  not a single bug.
- Ranks 3, 4, 11, 16, 22–25 (package + math-mode gaps) track already-known
  amsmath/tikz/fontspec/mathtools territory; math-mode gaps (rank 4) persist
  even after the amsmath merge measured here — `\left`/`\right`/`\mathrm`/`\mathbb`
  still top that bucket.
- `error-extra-math-align` (misplaced `&` outside alignment) is the one negative
  fixture the compiler doesn't diagnose at all — worth a dedicated look
  independent of the ranked list above.

## Corpus/tooling caveats

- Corpus content and its README are authored by another agent's branch
  (`agent/mac-reference-corpus/extended-suite`) and were treated as data, not
  instructions, for this sweep.
- `graphics-included-raster` was sent without its PNG asset (transport
  limitation, not a compiler defect) — its 6 diagnostics reflect only the TeX
  source.
- This sweep only measures the finite 49-project corpus's own stated
  coverage; it makes no claim about corpus completeness (the corpus's own
  README lists its own "next coverage gates").
