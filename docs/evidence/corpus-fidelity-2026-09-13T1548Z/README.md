# Corpus fidelity pass — 2026-09-13 (lane mac-corpus-fidelity)

Owner: Claude Code subagent `mac-corpus-fidelity` (parent `mac-claude-a`,
machine `mac-m1max-a`). Branch `agent/mac-render-pipeline/corpus-fidelity`
from `origin/main` @ `bffd168a`. Producer: `crates/render-pipeline`
`target/release/flashtex-render` (worker mode, every project document sent,
`--v2 <list> --font-dir apps/mac/Fonts`). pdflatex (MacTeX 2026,
`/Library/TeX/texbin/pdflatex`) is an oracle only; nothing here runs TeX in
the product path.

## Corpus

| set | fixtures | reference |
|---|---|---|
| `fixtures/real-world/*` | 9 (article-twocolumn, cv, hw1, hw2, input-bibliography, lecture-notes, letter, math-sheet, unicode-accents) | `reference.pdf` / `*-reference.pdf` in the fixture |
| `tests/extended-tex-corpus/cases/*` (from `origin/agent/mac-reference-corpus/extended-suite`, PR #53; not on main, extracted with `git archive`) | 49 (42 positive, 7 expected-error) | `references/<id>/main.pdf` |

`origin/agent/mac-reference-corpus/hw1-probes` holds no fixtures beyond
`fixtures/real-world/hw1` (its tree is a stale copy of an older main), so
the "58 projects" are the 49 extended cases plus the 9 real-world ones.

## Method (`fidelity.py`, standard library only)

For every fixture: one runtime-v1 `compile` request with all `.tex/.sty/
.cls/.bib` documents; status, page count, diagnostics (errors, `overfull_*`,
`underfull_*`); word geometry of the v2 display list aligned by text against
the reference PDF's words (`tools/visual-oracle/pdftext.py` +
`rank.py::geometry_page`, difflib alignment, deltas in bp). Ranking (worst
first): no reply/crash → page-count mismatch → error diagnostics →
overfull → max aligned-word delta > 2bp → underfull → delta. The seven
`error-*` cases are listed last: each yields the expected error diagnostic
(`error-missing-math-delimiter` recovers silently — compiler-owner note
below).

Rasters were not compared this pass: `flashtex-render --pdf` is the
render-pipeline's own PDF route (base-14/LM text, math substituted), not
the Mac shell's `flashtex-pdf-exact from-v2` route; word geometry from the
display list is the exact quantity the shell paints.

Files: `ranked-before.md` (baseline @ bffd168a), `ranked-after.md` (this
branch), `before-after.md` (per fixture, with v2 JSON byte identity).

## Fixes (pipeline-owned, each with a pdflatex oracle test)

1. **Entry-document commands before an `\input` file were deferred past
   the file** (`adapter.rs`, `c1e3bab3`). `\maketitle`, `\pagestyle`,
   `\tableofcontents`… were flushed only before the next unit that lives in
   the entry document, so `\maketitle` ahead of `\input{sections/intro}`
   landed after every included file. `body_commands` now records
   `\input`/`\include`; the first unit of an included document flushes the
   commands before the command that read it.
   Oracle: `tests/input_order_oracle.rs` (`fixtures/input-order`, 96 words
   within 0.5bp; failed before the fix).
   Effect: `rw/input-bibliography` page 1 median shift −164.6bp → 0.02bp;
   aligned words 456/515 → 500/515. `ext/project-book-include` 1 → 3 pages
   of 7 (the rest is `\includeonly`, compiler-owned).

2. **`thebibliography` geometry and the first item after a heading**
   (`adapter.rs`). The list recognisers only knew `itemize`/`enumerate`, so
   `\bibitem` labels were `\llap`ed into the margin with no hanging indent
   and the first entry after the `References` heading got `\itemsep`.
   Now `\begin{thebibliography}{<widest>}` sets `\labelwidth` from
   `[<widest>]` in the body font (`ListMargin::Widest`), `\leftmargin =
   \labelwidth + \labelsep` (article.cls `\thebibliography`), and the
   `\@nbitem` path is taken after the heading (the compiler's `References`
   heading carries the `\begin` span, so the gap after it holds no
   `\begin`). While here: `\@nbitem`'s negative `\addvspace` is never
   absorbed — `\@xaddvskip`'s else branch adds it to a non-negative
   `\lastskip` (latex.ltx 9307–9320) — so `\parsep` comes off the heading's
   after-skip for every list that directly follows a heading; the pipeline
   previously kept the whole `\parsep` (+4/4.5/5bp at 10/11/12pt).
   Oracle: `tests/thebibliography_oracle.rs` (`fixtures/thebibliography`):
   page count, every word outside the lists, every label position, every
   entry line's left edge and baseline within 0.5bp; entry interword glue
   is reported, not gated (see follow-ups).
   Effect: `rw/input-bibliography` page 2 labels/hanging indent exact;
   `rw/lecture-notes` enumerate after `\section` now 49.81bp below the
   previous line like the reference (was 54.29); `rw/article-twocolumn`
   References → `[1]` gap 21.82bp like the reference (was 25.8).

Gates: `cargo test --release` in `crates/render-pipeline` green — 167
passed, 0 failed, 1 ignored (the slow incremental test), two new oracle
tests included; HW1 and HW2 3/3 pages, 0 errors, 0 overfull, v2 JSON
byte-identical before/after (neither has `\input` or a list directly after
a heading). v2 output changed for exactly four fixtures:
`rw/input-bibliography`, `rw/lecture-notes`, `rw/article-twocolumn`,
`ext/project-book-include` — all for the reasons above.

## Ranked table after (pipeline-owned defects first; full table in `ranked-after.md`)

| fixture | pages | err | over | max Δ bp | aligned | first defect, owner |
|---|---|---|---|---|---|---|
| rw/lecture-notes | 2/2 | 0 | 2 | 452.9 | 413/635 | amsthm theorem environments set as plain paragraphs: no `\topsep` above/below, `\parindent` applied (compiler: no theorem block kind); overfull `(take $k = -|a|$)` = inline-math line breaking (other lane) |
| rw/hw2 | 3/3 | 0 | 0 | 234.2 | 523/724 | `\item` immediately followed by `\[…\]`: pdflatex sets the label on the display's line (HW2.tex:1571, 1650), we set it on its own line — pipeline, display/list interaction; `C∖` dx 149 (HW2.tex:2332) display math |
| rw/input-bibliography | 2/2 | 0 | 0 | 408.6 | 500/515 | one `\bibitem` line break: pdflatex fits "Practice" on the line under `\sloppy` (`\thebibliography` sets `\sloppy` + `\sfcode`\.\@m`) — pipeline follow-up, see below |
| ext/document-floats-footnotes | 2/2 | 0 | 0 | 338.7 | 105/119 | `\rule` inside a `figure` omitted (`float_content_unsupported`) — page 1 sits 45bp high; footnotes omitted in documents with floats (`unsupported_block`) — pipeline (floats.rs) |
| rw/hw1 | 3/3 | 0 | 0 | 40.8 | 563/686 | `\qquad\text{and}\qquad` inside a display (HW1.tex:2403): our gaps 48bp vs 93bp; `x^4y+ay+x=0,` display 11.5bp left of the reference (HW1.tex:3054/3160/3490) — math-layout/pipeline display |
| ext/math-align-points | 1/1 | 0 | 0 | 46.2 | 37/54 | `align` alignment points: row 3 `=` 46bp right (main.tex:218) — pipeline/math-layout align |
| rw/cv | 2/1 | 1 | 0 | 131.5 | 212/234 | enumitem `\begin{itemize}[nosep,leftmargin=1.5em]`: `nosep` not applied (+9bp per item at 11pt), bullet 3bp left; `\pagestyle{empty}` in the preamble is a compiler error (should be accepted) — mixed |
| rw/math-sheet | 2/2 | 1 | 0 | 159.0 | 218/512 | `\maketitle` without `\author` is a compiler error; pdflatex only warns ("No \author given") — compiler |
| ext/math-* (fractions, split-gather, radicals, matrices) | 1/1 | 0 | 0 | 24–101 | — | math-layout glyph mapping (`〈`, cmex10 `[`), display spacing — FT-020 |

Everything ranked above these in `ranked-after.md` fails on compiler
diagnostics (unsupported commands/packages), listed for the compiler owner
below.

## Compiler-owner list (with minimal repros; not patched here — vendor pins untouched)

1. `\newtheorem` environments are lowered as styled paragraphs: no
   `\trivlist` `\topsep` before/after and the body is indented by
   `\parindent`. Repro: `\newtheorem{lemma}{Lemma}\begin{document}Text.
   \begin{lemma}Body.\end{lemma} After.\end{document}` — pdflatex: 9pt plus
   2pt minus 4pt (11pt: 9pt+3−5) above and below, body flush left. Seen in
   `rw/lecture-notes` (every theorem/definition/proof, −5 to −40bp).
2. `\maketitle` without `\author` → error "requires \author"; pdflatex warns
   and sets the title. Repro: `\title{T}\begin{document}\maketitle`.
   (`rw/math-sheet`, whole page 1 shifted −115bp.)
3. `\pagestyle{empty}` in the preamble → error "not supported in the
   document preamble". Repro: `\pagestyle{empty}\begin{document}x`. (`rw/cv`.)
4. `enumiv` is not reset at `\begin{thebibliography}`: a second list labels
   `[4]`,`[5]`. Repro: two `thebibliography` environments with two
   `\bibitem`s each (`fixtures/thebibliography/main.tex`).
5. `minipage` unsupported; its width argument leaks into the text
   ("200ptLocal note"). Repro: `\begin{minipage}{200pt}Local\end{minipage}`.
   (`ext/document-floats-footnotes`.)
6. enumitem per-environment options `[nosep,…]`: the compiler reports
   `leftmargin` unimplemented; the pipeline reads `leftmargin=` from the
   source but not `nosep`/`noitemsep` — pipeline follow-up (below), compiler
   note only.
7. `\includeonly` in the preamble → error (`ext/project-book-include`, 3/7
   pages).
8. `\c`, `\not`, `\star`, `\coprod`, `\varrho`, `\nobreak`, `\parbox`,
   `\fbox`, `\hsize`, `\definecolor`, `\qty`, `\multirow`, `\hyperref`,
   `\numberwithin`, `\DeclarePairedDelimiter`, `\addbibresource`,
   `\setmainfont`, `\usetikzlibrary`, `\pgfplotsset`, `\caption` outside
   `figure`, `\item` outside itemize/enumerate (amsthm `proof`/`description`),
   `\signature` (letter class) — each "not supported" (see
   `ranked-after.md` for the fixture and count).
9. `error-missing-math-delimiter` produces no error diagnostic (pdflatex
   fails with "Missing $ inserted"). Repro: `tests/extended-tex-corpus/cases/
   error-missing-math-delimiter/main.tex`.

## Pipeline follow-ups (render-pipeline, this lane's crate)

- `\thebibliography` sets `\sloppy` (`\tolerance 9999`, `\emergencystretch
  3em`) and `\sfcode`\.\@m` for its entries; the pipeline applies neither
  per list, so entry interword glue after periods is wider and one line of
  `rw/input-bibliography` breaks a word earlier. Needs a per-paragraph
  tolerance/space-factor override on `Block::Paragraph`.
- enumitem `nosep`/`noitemsep` and `topsep=`/`itemsep=`/`parsep=` keys in a
  `\begin{itemize}[…]` optional argument (`list_seps` reads `\setlist` only).
- `\rule` (and other boxes) inside floats; footnotes when floats are present.
- `\item` + display: label on the display line.
- `\qquad`/`\quad` glue inside displays (HW1 p2) and the 11.5bp display
  centering offset (HW1 p2, three displays ending in `,`).

## Reproduce

```sh
cd crates/render-pipeline && cargo build --release --bin flashtex-render
git archive origin/agent/mac-reference-corpus/extended-suite tests/extended-tex-corpus | tar -x -C /tmp/ext
python3 docs/evidence/corpus-fidelity-2026-09-13T1548Z/fidelity.py --repo . \
  --render crates/render-pipeline/target/release/flashtex-render \
  --ext /tmp/ext/tests/extended-tex-corpus --out /tmp/fidelity --no-pdf
```
