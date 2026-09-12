# pdflatex oracle comparison: `wrap-sample.tex`, variant C

The comparison is executable: `cargo test --test oracle_wrap_sample -- --nocapture`
prints the per-word table below and asserts the headline numbers.

## Reference (oracle) declaration

| Item | Value |
|---|---|
| Sample | `tools/native-validation/oracle-samples/wrap-sample.tex` on `origin/agent/mac-validation/native-verification` (`d855ff8894099e92936b5f0b896938073db69993` at the time of writing; the sample is unchanged since the report) |
| Oracle data | `tools/native-validation/reports/oracle-20260912T050958Z.json`, result `wrap-sample` / `C-times12-1in-ragged`, `comparisons[de1020c].deltas[*]` fields `word`, `oracle_x`, `oracle_bottom`, `oracle_line_start` (copied into `tests/oracle_wrap_sample.rs`) |
| Engine | `/Library/TeX/texbin/pdflatex` = pdfTeX 3.141592653-2.6-1.40.29 (TeX Live 2026, BasicTeX); pdflatex is an oracle only and was **not** run by this crate |
| Preamble | `\documentclass[12pt]{article}`, `\usepackage[T1]{fontenc}`, `\usepackage{times}`, `\usepackage[margin=1in]{geometry}`, `\setlength{\parindent}{0pt}`, `\setcounter{secnumdepth}{0}`, `\pagestyle{empty}`, `\raggedright`, `\hyphenpenalty=10000`, `\exhyphenpenalty=10000` |
| Fonts | psnfss `ptmr8t` / `ptmb8t` / `ptmri8t` (fontinst TFMs from the Adobe Times AFMs; URW Nimbus Roman outlines). TFM `\fontdimen2..4,7` = 0.25 / 0.15 / 0.06 / 0.06 em (read from the TeX Live TFM files with a 40-line TFM parser; no TeX program was run). |
| Word boxes | PDFKit via `oracle_extract.swift`: `x` = glyph-box left, `bottom` = glyph-box bottom (baseline + font descent), from the page's top-left, in PDF points (bp). No rasterisation, so **no DPI** is involved; the comparison is on PDF coordinates. |

## Coordinate conversion

pdflatex computes in TeX points (1 pt = 1/72.27 in) and writes the PDF in big
points (1 bp = 1/72 in). Every oracle coordinate therefore is `tex_pt × 72/72.27`
(`BP_PER_TEX_PT = 0.996264…`) plus the 72 bp left margin. The crate runs the
layout in TeX points with the oracle's `\hsize` = 469.75499 pt (6.5 in), 12 pt
body, 17.28 pt `\Large` bold heading, `\topskip` 12 pt, `\baselineskip` 14.5 pt,
`\parskip` 0 pt plus 1 pt, `\section` skips 3.5 ex / 2.3 ex of the 12 pt body
(18.9 / 12.42 pt), and converts with `tex_pt_to_bp` before comparing. That is
why the oracle's line pitch reads 14.446 bp: it is 14.5 TeX pt (issue #10's
"14.45 vs 14.4" is exactly this unit difference, not a spacing bug).

## What the crate reproduces

Layout: `Algorithm::TotalFit`, `BreakMode::RaggedRight`, `NoHyphenation`,
Core 14 Times adapter with the AFM kern pairs and `fi`/`fl` ligatures, TeX
`\spacefactor` spacing.

Headline (ours − oracle, 102 words):

```
line starts matched: 102/102; mean|dx| 0.001 bp; max|dx| 0.002 bp; mean|dy| 0.016 bp; max|dy| 0.132 bp
```

* **Line starts**: all 8 oracle line starts are reproduced (heading; "A"; "emphasised";
  "This"; "have"; "how"; "longer"; "Short") and no extra breaks are made.
  Paragraph line counts 1 / 2 / 4 / 1.
* **Horizontal**: every word is within 0.002 bp; the residual is the PDF's
  3-decimal coordinate rounding. The tight case from issue #10 — "and" at the end
  of "have … starts and" — lands at 522.662 bp versus the oracle's 522.663 with a
  right edge of 539.99 bp (0.01 bp inside the 540 bp measure). Without the AFM
  kerns the same line measures 2.0 pt wider and "and" wraps, which is the de1020c
  compiler's 450 pt outlier.
* **Vertical**: body baselines are within 0.013 bp (`\topskip`, `\section`
  afterskip, TeX interline glue across the heading and paragraph boundaries, and
  `\parskip` 0 all modelled). The heading's −0.069 bp and the bold "bold phrase"
  −0.132 bp are not baseline differences: the oracle's `bottom` is a glyph-box
  bottom using PDFKit's descent for the embedded URW *bold* font (≈0.209 em vs the
  Adobe AFM's 0.205 em); the roman rows show the two agree once the descent is
  the same (0.217 em).

Per-word extract (full table from the test; x/bottom in bp):

| word | oracle x | ours x | dx | oracle bottom | ours bottom | dy | line start o/u |
|---|---|---|---|---|---|---|---|
| Wrapping | 72.000 | 72.000 | +0.000 | 87.553 | 87.484 | -0.069 | yes/yes |
| and | 151.879 | 151.880 | +0.001 | 87.553 | 87.484 | -0.069 | no/no |
| A | 72.000 | 72.000 | +0.000 | 113.357 | 113.369 | +0.012 | yes/yes |
| naïve | 83.620 | 83.620 | +0.000 | 113.357 | 113.369 | +0.012 | no/no |
| and | 472.810 | 472.809 | -0.001 | 113.357 | 113.369 | +0.012 | no/no |
| emphasised | 72.000 | 72.000 | +0.000 | 127.802 | 127.815 | +0.013 | yes/yes |
| This | 72.000 | 72.000 | +0.000 | 142.248 | 142.261 | +0.013 | yes/yes |
| both | 496.314 | 496.313 | -0.001 | 142.248 | 142.261 | +0.013 | no/no |
| have | 72.000 | 72.000 | +0.000 | 156.694 | 156.706 | +0.012 | yes/yes |
| and | 522.663 | 522.662 | -0.001 | 156.694 | 156.706 | +0.012 | no/no |
| how | 72.000 | 72.000 | +0.000 | 171.140 | 171.152 | +0.012 | yes/yes |
| few | 495.907 | 495.906 | -0.001 | 171.140 | 171.152 | +0.012 | no/no |
| longer | 72.000 | 72.000 | +0.000 | 185.586 | 185.598 | +0.012 | yes/yes |
| here. | 407.953 | 407.952 | -0.001 | 185.586 | 185.598 | +0.012 | no/no |
| Short | 72.000 | 72.000 | +0.000 | 200.032 | 200.044 | +0.012 | yes/yes |
| coffee | 216.443 | 216.442 | -0.001 | 200.032 | 200.044 | +0.012 | no/no |
| — | 326.024 | 326.023 | -0.001 | 200.032 | 200.044 | +0.012 | no/no |
| done. | 340.968 | 340.967 | -0.001 | 200.032 | 200.044 | +0.012 | no/no |

## First-fit versus total-fit on this sample

`first_fit_ragged_matches_total_fit_on_this_sample` shows the greedy breaker
gives identical output here. That is expected, not luck: under `\raggedright`
every line has infinite stretch, so its badness is 0 and its demerits are
`(\linepenalty + 0)^2 = 100`; total demerits are then 100 × lines, total-fit
minimises the line count, and TeX's `<=` tie rule makes the latest feasible
predecessor win, which recursively is the greedy break. The two would differ
where shrinking a line (finite `\fontdimen4`, still active under
`\raggedright`) saves a whole line, or in justified mode (see
`tests/golden.rs::first_fit_and_total_fit_differ_on_a_loose_line`).

## Known differences from pdflatex that this sample does not exercise

* Justified mode with automatic hyphenation: no pattern hyphenator is shipped
  (only `\-`). TeX's second pass would find more break points than we do.
* `ff`, `ffi`, `ffl`: T1 Times has fontinst composites; we typeset `f`+`f`
  with the AFM `f f` kern (−25), which happens to give the same 641-unit width
  for "coffee" here, but `ffi`/`ffl` would differ by 25 units.
* `\addvspace` merging of consecutive vertical skips, `\looseness`, per-character
  heights/depths (we use font ascender/descender for interline glue), and
  `\flushbottom`.
* Glyph *positions inside* a word were compared only through the next word's
  start; per-glyph offsets follow the same advances + kerns.
