# Reference comparison: flashtex-math-layout vs pdfTeX

## Declaration

| item | value |
| --- | --- |
| Reference engine | pdfTeX 3.141592653-2.6-1.40.29 (TeX Live 2026), BasicTeX at `/Library/TeX/texbin/pdflatex`, format `pdflatex 2026.3.1`; used **only** as an oracle in a scratch directory, never in the product path |
| Reference document | `docs/oracle/compare.tex` — `article` 10pt, `\displaystyle`, `\pdfcompresslevel=0` so the page stream is readable |
| Reference fonts | Computer Modern Type 1 (`cmr10`, `cmr7`, `cmmi10`, `cmmi7`, `cmsy10`, `cmex10`, subsetted by pdfTeX); metrics from the `cm` TFM files of the same TeX Live |
| Ours | `flashtex-math-layout` at this revision with `CmMathMetrics::latex_10pt()`, `Style::DISPLAY`; runs emitted by `cargo run --example emit_runs` |
| Units / DPI | vector comparison in PDF points (bp, 1/72 in); no rasterisation, so no DPI applies. Ours is converted from TeX points once with 72/72.27 |
| Extraction | (1) page content stream parsed by `tools/oracle_compare.py`: glyph origins from `Tf`/`Td`/`TJ` with the PDF's own `/Widths`, rules from the stroked `w … m … l S` segments pdfTeX draws for thin rules; (2) Apple PDFKit per-character selection boxes from `tools/oracle_glyphs.swift` (macOS 26.3.1) as an independent cross-check of glyph left edges and advances |
| Alignment | per formula, ours is translated so its first glyph's origin coincides with the same glyph's origin in the reference; every other glyph/rule is then compared directly. No scaling or fitting |
| Overlays | not produced (optional in the assignment); the tables below are the quantitative evidence |

Fonts are the same on both sides, so the comparison is absolute, not just
relative: Computer Modern is what the reference uses and what the adapter
embeds. (A Times-based comparison would only be meaningful as ratios; the
Times adapter is an approximation and is not compared here.)

## Headline

Largest absolute deviation over all glyph origins and rule geometry in the
three formulas: **0.003 bp** (pdfTeX writes three decimals, so this is the
reference's own rounding). All 20 glyphs and 4 rules were matched by font,
glyph id, position and, for rules, length and thickness. The comparison
found no missing or extra glyphs and no limitation reports.

| formula | LaTeX | glyphs | rules | max abs Δ (bp) |
| --- | --- | --- | --- | --- |
| A | `\frac{\frac{a}{b}}{c}=1` | 5 | 2 | 0.003 |
| B | `\sqrt{x}+\left(\frac{a}{b}\right)` | 7 | 2 | 0.003 |
| C | `\sum_{i=1}^{n}x_i^2` | 8 | 0 | 0.001 |

What that covers: stacked fraction numerator/denominator shifts and both rule
positions (axis-centred, 0.398 bp thick), Inner–Rel–Ord spacing, radical sign
choice/raise and overbar, the 12pt `cmex10` parenthesis selection and axis
centring, display-size `\sum` with limits above and below, and sub/superscript
shifts on a character nucleus.

## Reproduce

```sh
# in a scratch directory (oracle only)
/Library/TeX/texbin/pdflatex -interaction=nonstopmode -output-directory=. crates/math-layout/docs/oracle/compare.tex
swiftc -O -o oracle_glyphs crates/math-layout/tools/oracle_glyphs.swift && ./oracle_glyphs compare.pdf > pdfkit.json
cargo run -q --manifest-path crates/math-layout/Cargo.toml --example emit_runs > runs.json
python3 crates/math-layout/tools/oracle_compare.py compare.pdf runs.json pdfkit.json
# stage 2: the same with compare2.tex, `emit_runs -- 2`, runs2/pdfkit2.json
```

`docs/oracle/` keeps the inputs and the captured outputs (`runs.json`,
`pdfkit.json`, `report.md`) from the run recorded below.

## PDFKit cross-check notes

PDFKit reports left edges identical to the content-stream origins (Δ = 0.000)
for every glyph it can select. Two caveats are visible in the tables:
`cmex10` glyphs (the delimiters and the display `\sum`) have no Unicode
mapping in the subset font, so PDFKit does not expose them as characters;
and PDFKit merges some adjacent characters into one selection box (`+` and `b` with
the following delimiter, `2` with `i`, `n` with `\sum`), which widens those
boxes beyond the single-glyph advance. Neither affects the origin comparison.


## Stage 2 (FT-020 rev 2): extensible and nested constructs

Same engine, fonts, extraction and alignment as above; document
`docs/oracle/compare2.tex`, runs from `cargo run --example emit_runs -- 2`,
captured outputs `docs/oracle/runs2.json`, `pdfkit2.json`, `report2.md`.

| formula | LaTeX | what it exercises | glyphs | rules | max abs Δ (bp) |
| --- | --- | --- | --- | --- | --- |
| D | `\left\{\frac{\frac{\frac{a}{b}}{c}}{\frac{d}{\frac{e}{f}}}\right\}` | 36pt body: extensible brace recipe (top 0x38, mid 0x3C, bot 0x3A, no repeaters), three nested fraction levels, axis centring | 12 | 5 | 0.001 |
| E | `\sqrt[3]{\frac{a}{b}}` | radical degree (LaTeX `\r@@t`: 5mu, raise 0.6(h−d), −10mu), 24pt sign, overbar | 4 | 2 | 0.001 |
| F | `\lim_{x\to 0}\frac{\sin x}{x}` | text operator with a lower limit (Rule 13a), `\nolimits` operator, Op–Ord thin space, script-style Rel spacing suppression | 11 | 1 | 0.003 |
| G | `\sqrt{\left\{…\right\}}` (D under a radical) | extensible radical recipe (top 0x76, 3 × rep 0x75, bot 0x74) with its overbar over the extensible braces | 17 | 6 | 0.001 |

All 44 glyph rows and 16 rule rows in `report2.md` match within
0.003 bp; the extensible pieces match piece-for-piece (same glyph ids, same
stacking origins, same repeater count). No limitation was reported.

## Full report, stage 1 (captured 2026-09-12)

Δ columns are ours − reference in bp. "ours baseline"/"ref baseline" are
relative to the anchor glyph's baseline, positive up.
Largest absolute deviation over all formulas: 0.003 bp

### Formula A

| glyph | font | ours x | ref x | Δx | ours baseline | ref baseline | Δy |
|---|---|---|---|---|---|---|---|
| a | cmmi7 gid 97 | 0.000 | 0.000 | +0.000 | -0.000 | 0.000 | -0.000 |
| b | cmmi7 gid 98 | 0.409 | 0.409 | -0.000 | -7.358 | -7.358 | -0.000 |
| c | cmmi10 gid 99 | 0.005 | 0.005 | +0.000 | -18.077 | -18.077 | -0.000 |
| = | cmr10 gid 61 | 9.480 | 9.480 | -0.000 | -11.243 | -11.243 | -0.000 |
| 1 | cmr10 gid 49 | 19.996 | 19.999 | -0.003 | -11.243 | -11.243 | -0.000 |

| rule | ours x | ref x | Δx | ours centre y | ref centre y | Δy | ours w | ref w | Δw | ours h | ref h | Δh |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 0.000 | 0.000 | +0.000 | -1.432 | -1.432 | +0.000 | 4.321 | 4.321 | +0.000 | 0.398 | 0.398 | +0.000 |
| 1 | -1.196 | -1.195 | -0.001 | -8.753 | -8.752 | -0.001 | 6.712 | 6.712 | +0.000 | 0.398 | 0.398 | +0.000 |

Largest absolute deviation in formula A: 0.003 bp; limitations: none

PDFKit cross-check (content-stream x vs PDFKit box left edge, same page):
| ref glyph | stream x | PDFKit left | Δ | PDFKit width | stream advance | Δ |
|---|---|---|---|---|---|---|
| cmmi7 97 | 188.906 | 188.906 | +0.000 | 4.322 | 4.322 | +0.000 |
| cmmi7 98 | 189.315 | 189.315 | +0.000 | 3.504 | 3.504 | +0.000 |
| cmmi10 99 | 188.911 | 188.911 | +0.000 | 4.312 | 4.312 | -0.000 |
| cmr10 61 | 198.386 | 198.386 | +0.000 | 7.749 | 7.749 | -0.000 |
| cmr10 49 | 208.905 | 208.905 | +0.000 | 4.981 | 4.981 | +0.000 |

### Formula B

| glyph | font | ours x | ref x | Δx | ours baseline | ref baseline | Δy |
|---|---|---|---|---|---|---|---|
| √ | cmsy10 gid 112 | 0.000 | 0.000 | +0.000 | -0.000 | 0.000 | -0.000 |
| x | cmmi10 gid 120 | 8.302 | 8.302 | +0.000 | -7.662 | -7.662 | -0.000 |
| + | cmr10 gid 43 | 16.210 | 16.207 | +0.003 | -7.662 | -7.662 | -0.000 |
| ( | cmex10 gid 16 | 26.173 | 26.173 | -0.000 | 3.396 | 3.396 | +0.000 |
| a | cmmi10 gid 97 | 33.318 | 33.318 | +0.000 | -0.922 | -0.923 | +0.001 |
| b | cmmi10 gid 98 | 33.813 | 33.813 | +0.000 | -14.496 | -14.496 | -0.000 |
| ) | cmex10 gid 17 | 39.780 | 39.780 | -0.000 | 3.396 | 3.396 | +0.000 |

| rule | ours x | ref x | Δx | ours centre y | ref centre y | Δy | ours w | ref w | Δw | ours h | ref h | Δh |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 8.302 | 8.302 | +0.000 | 0.199 | 0.199 | +0.000 | 5.694 | 5.694 | -0.000 | 0.398 | 0.398 | +0.000 |
| 1 | 33.318 | 33.318 | +0.000 | -5.172 | -5.172 | +0.000 | 5.266 | 5.266 | +0.000 | 0.398 | 0.398 | +0.000 |

Largest absolute deviation in formula B: 0.003 bp; limitations: none

PDFKit cross-check (content-stream x vs PDFKit box left edge, same page):
| ref glyph | stream x | PDFKit left | Δ | PDFKit width | stream advance | Δ |
|---|---|---|---|---|---|---|
| cmsy10 112 | 186.100 | 186.100 | +0.000 | 8.302 | 8.302 | +0.000 |
| cmmi10 120 | 194.402 | 194.402 | -0.000 | 5.694 | 5.694 | +0.000 |
| cmr10 43 | 202.307 | 202.307 | +0.000 | 15.915 | 7.749 | +8.166 |
| cmmi10 97 | 219.418 | 219.418 | +0.000 | 5.266 | 5.266 | +0.000 |
| cmmi10 98 | 219.913 | 219.913 | +0.000 | 11.917 | 4.276 | +7.641 |

### Formula C

| glyph | font | ours x | ref x | Δx | ours baseline | ref baseline | Δy |
|---|---|---|---|---|---|---|---|
| n | cmmi7 gid 110 | 0.000 | 0.000 | +0.000 | -0.000 | 0.000 | -0.000 |
| ∑ | cmex10 gid 88 | -4.733 | -4.733 | +0.000 | -2.989 | -2.989 | +0.000 |
| i | cmmi7 gid 105 | -3.991 | -3.990 | -0.001 | -24.208 | -24.208 | +0.000 |
| = | cmr7 gid 61 | -1.172 | -1.171 | -0.001 | -24.208 | -24.208 | +0.000 |
| 1 | cmr7 gid 49 | 4.944 | 4.945 | -0.001 | -24.208 | -24.208 | +0.000 |
| x | cmmi10 gid 120 | 11.318 | 11.318 | +0.000 | -12.453 | -12.454 | +0.001 |
| 2 | cmr7 gid 50 | 17.012 | 17.012 | +0.000 | -8.340 | -8.340 | +0.000 |
| i | cmmi7 gid 105 | 17.012 | 17.012 | +0.000 | -14.916 | -14.917 | +0.001 |

| rule | ours x | ref x | Δx | ours centre y | ref centre y | Δy | ours w | ref w | Δw | ours h | ref h | Δh |
|---|---|---|---|---|---|---|---|---|---|---|---|---|

Largest absolute deviation in formula C: 0.001 bp; limitations: none

PDFKit cross-check (content-stream x vs PDFKit box left edge, same page):
| ref glyph | stream x | PDFKit left | Δ | PDFKit width | stream advance | Δ |
|---|---|---|---|---|---|---|
| cmmi7 110 | 190.971 | 190.971 | +0.000 | 9.658 | 4.925 | +4.733 |
| cmmi7 105 | 186.981 | 186.981 | +0.000 | 2.819 | 2.819 | +0.000 |
| cmr7 61 | 189.800 | 189.800 | +0.000 | 6.116 | 6.116 | +0.000 |
| cmr7 49 | 195.916 | 195.916 | +0.000 | 3.972 | 3.972 | -0.000 |
| cmmi10 120 | 202.289 | 202.289 | +0.000 | 5.694 | 5.694 | +0.000 |
| cmr7 50 | 207.983 | 207.983 | +0.000 | 2.819 | 3.972 | -1.153 |
| cmmi7 105 | 207.983 | 207.983 | +0.000 | 2.819 | 2.819 | +0.000 |

## Full report, stage 2 (captured 2026-09-12)

Largest absolute deviation over all formulas: 0.003 bp

### Formula D

| glyph | font | ours x | ref x | Δx | ours baseline | ref baseline | Δy |
|---|---|---|---|---|---|---|---|
| { | cmex10 gid 56 | 0.000 | 0.000 | +0.000 | 0.000 | 0.000 | +0.000 |
| { | cmex10 gid 60 | 0.000 | 0.000 | +0.000 | -8.966 | -8.966 | -0.000 |
| { | cmex10 gid 58 | 0.000 | 0.000 | +0.000 | -26.899 | -26.899 | -0.000 |
| a | cmmi5 gid 97 | 12.511 | 12.512 | -0.001 | -4.937 | -4.937 | +0.000 |
| b | cmmi5 gid 98 | 12.863 | 12.863 | -0.000 | -10.014 | -10.014 | -0.000 |
| c | cmmi7 gid 99 | 12.660 | 12.660 | -0.000 | -16.538 | -16.538 | -0.000 |
| d | cmmi7 gid 100 | 12.366 | 12.367 | -0.001 | -24.171 | -24.171 | +0.000 |
| e | cmmi5 gid 101 | 12.754 | 12.754 | -0.000 | -28.851 | -28.851 | -0.000 |
| f | cmmi5 gid 102 | 12.442 | 12.443 | -0.001 | -33.929 | -33.929 | -0.000 |
| } | cmex10 gid 57 | 20.024 | 20.025 | -0.001 | 0.000 | 0.000 | +0.000 |
| } | cmex10 gid 61 | 20.024 | 20.025 | -0.001 | -8.966 | -8.966 | -0.000 |
| } | cmex10 gid 59 | 20.024 | 20.025 | -0.001 | -26.899 | -26.899 | -0.000 |

| rule | ours x | ref x | Δx | ours centre y | ref centre y | Δy | ours w | ref w | Δw | ours h | ref h | Δh |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 12.511 | 12.512 | -0.001 | -5.870 | -5.870 | -0.000 | 3.858 | 3.858 | -0.000 | 0.398 | 0.398 | +0.000 |
| 1 | 11.316 | 11.316 | -0.000 | -10.612 | -10.612 | -0.000 | 6.249 | 6.249 | -0.000 | 0.398 | 0.398 | +0.000 |
| 2 | 10.051 | 10.052 | -0.001 | -17.933 | -17.933 | +0.000 | 8.778 | 8.778 | -0.000 | 0.398 | 0.398 | +0.000 |
| 3 | 11.247 | 11.247 | -0.000 | -25.603 | -25.602 | -0.001 | 6.386 | 6.386 | +0.000 | 0.398 | 0.398 | +0.000 |
| 4 | 12.442 | 12.443 | -0.001 | -29.785 | -29.785 | -0.000 | 3.995 | 3.995 | +0.000 | 0.398 | 0.398 | +0.000 |

Largest absolute deviation in formula D: 0.001 bp; limitations: none

PDFKit cross-check (content-stream x vs PDFKit box left edge, same page):
| ref glyph | stream x | PDFKit left | Δ | PDFKit width | stream advance | Δ |
|---|---|---|---|---|---|---|
| cmex10 58 | 186.653 | 186.653 | +0.000 | 8.856 | 8.856 | +0.000 |
| cmmi5 97 | 199.165 | 199.516 | +0.351 | 3.155 | 3.858 | -0.703 |
| cmmi5 98 | 199.516 | 199.516 | +0.000 | 3.155 | 3.155 | -0.000 |
| cmmi7 99 | 199.313 | 199.313 | +0.000 | 3.560 | 3.560 | +0.000 |
| cmmi7 100 | 199.020 | 199.020 | +0.000 | 4.147 | 4.147 | +0.000 |
| cmmi5 101 | 199.407 | 199.407 | +0.000 | 3.372 | 3.372 | +0.000 |
| cmmi5 102 | 199.096 | 199.096 | +0.000 | 3.407 | 3.407 | +0.000 |
| cmex10 59 | 206.678 | 206.678 | +0.000 | 8.856 | 8.856 | +0.000 |

### Formula E

| glyph | font | ours x | ref x | Δx | ours baseline | ref baseline | Δy |
|---|---|---|---|---|---|---|---|
| 3 | cmr5 gid 51 | 0.000 | 0.000 | +0.000 | -0.000 | 0.000 | -0.000 |
| √ | cmex10 gid 114 | -2.145 | -2.145 | +0.000 | 10.711 | 10.712 | -0.001 |
| a | cmmi10 gid 97 | 9.014 | 9.013 | +0.001 | 2.862 | 2.862 | -0.000 |
| b | cmmi10 gid 98 | 9.509 | 9.508 | +0.001 | -10.712 | -10.711 | -0.001 |

| rule | ours x | ref x | Δx | ours centre y | ref centre y | Δy | ours w | ref w | Δw | ours h | ref h | Δh |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 7.818 | 7.818 | +0.000 | 10.910 | 10.911 | -0.001 | 7.657 | 7.657 | +0.000 | 0.398 | 0.398 | +0.000 |
| 1 | 9.014 | 9.013 | +0.001 | -1.387 | -1.387 | -0.000 | 5.266 | 5.266 | +0.000 | 0.398 | 0.398 | +0.000 |

Largest absolute deviation in formula E: 0.001 bp; limitations: none

PDFKit cross-check (content-stream x vs PDFKit box left edge, same page):
| ref glyph | stream x | PDFKit left | Δ | PDFKit width | stream advance | Δ |
|---|---|---|---|---|---|---|
| cmr5 51 | 188.591 | 188.591 | +0.000 | 7.818 | 3.390 | +4.427 |
| cmmi10 97 | 197.604 | 197.604 | +0.000 | 5.266 | 5.266 | +0.000 |
| cmmi10 98 | 198.099 | 198.099 | +0.000 | 4.276 | 4.276 | -0.000 |

### Formula F

| glyph | font | ours x | ref x | Δx | ours baseline | ref baseline | Δy |
|---|---|---|---|---|---|---|---|
| l | cmr10 gid 108 | 0.000 | 0.000 | +0.000 | -0.000 | 0.000 | -0.000 |
| i | cmr10 gid 105 | 2.767 | 2.768 | -0.000 | -0.000 | 0.000 | -0.000 |
| m | cmr10 gid 109 | 5.535 | 5.535 | -0.000 | -0.000 | 0.000 | -0.000 |
| x | cmmi7 gid 120 | -1.297 | -1.300 | +0.003 | -6.155 | -6.155 | +0.000 |
| → | cmsy7 gid 33 | 3.221 | 3.218 | +0.003 | -6.155 | -6.155 | +0.000 |
| 0 | cmr7 gid 48 | 11.163 | 11.160 | +0.003 | -6.155 | -6.155 | +0.000 |
| s | cmr10 gid 115 | 17.990 | 17.987 | +0.003 | 6.740 | 6.739 | +0.001 |
| i | cmr10 gid 105 | 21.920 | 21.917 | +0.003 | 6.740 | 6.739 | +0.001 |
| n | cmr10 gid 110 | 24.687 | 24.684 | +0.003 | 6.740 | 6.739 | +0.001 |
| x | cmmi10 gid 120 | 31.883 | 31.883 | -0.001 | 6.740 | 6.739 | +0.001 |
| x | cmmi10 gid 120 | 24.936 | 24.933 | +0.003 | -6.834 | -6.834 | +0.000 |

| rule | ours x | ref x | Δx | ours centre y | ref centre y | Δy | ours w | ref w | Δw | ours h | ref h | Δh |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 17.990 | 17.987 | +0.003 | 2.491 | 2.490 | +0.001 | 19.586 | 19.586 | +0.000 | 0.398 | 0.398 | +0.000 |

Largest absolute deviation in formula F: 0.003 bp; limitations: none

PDFKit cross-check (content-stream x vs PDFKit box left edge, same page):
| ref glyph | stream x | PDFKit left | Δ | PDFKit width | stream advance | Δ |
|---|---|---|---|---|---|---|
| cmr10 108 | 186.847 | 186.847 | +0.000 | 2.768 | 2.768 | +0.000 |
| cmr10 105 | 189.614 | 189.614 | +0.000 | 2.768 | 2.768 | -0.000 |
| cmr10 109 | 192.382 | 192.382 | -0.000 | 8.302 | 8.302 | -0.000 |
| cmmi7 120 | 185.547 | 185.547 | +0.000 | 4.518 | 4.518 | +0.000 |
| cmsy7 33 | 190.065 | 190.065 | +0.000 | 7.942 | 7.942 | +0.000 |
| cmr7 48 | 198.007 | 198.007 | +0.000 | 3.972 | 3.972 | +0.000 |
| cmr10 115 | 204.834 | 204.834 | +0.000 | 3.929 | 3.929 | +0.000 |
| cmr10 105 | 208.763 | 208.763 | +0.000 | 2.768 | 2.768 | -0.000 |
| cmr10 110 | 211.531 | 211.531 | +0.000 | 5.535 | 5.535 | -0.000 |
| cmmi10 120 | 218.730 | 218.730 | +0.000 | 5.694 | 5.694 | +0.000 |
| cmmi10 120 | 211.780 | 211.780 | +0.000 | 5.694 | 5.694 | +0.000 |

### Formula G

| glyph | font | ours x | ref x | Δx | ours baseline | ref baseline | Δy |
|---|---|---|---|---|---|---|---|
| √ | cmex10 gid 118 | 0.000 | 0.000 | +0.000 | -0.000 | 0.000 | -0.000 |
| √ | cmex10 gid 117 | 0.000 | 0.000 | +0.000 | -5.579 | -5.579 | -0.000 |
| √ | cmex10 gid 117 | 0.000 | 0.000 | +0.000 | -11.557 | -11.557 | +0.000 |
| √ | cmex10 gid 117 | 0.000 | 0.000 | +0.000 | -17.534 | -17.534 | -0.000 |
| √ | cmex10 gid 116 | 0.000 | 0.000 | +0.000 | -23.512 | -23.512 | -0.000 |
| { | cmex10 gid 56 | 10.516 | 10.516 | +0.000 | -3.525 | -3.525 | +0.000 |
| { | cmex10 gid 60 | 10.516 | 10.516 | +0.000 | -12.491 | -12.491 | -0.000 |
| { | cmex10 gid 58 | 10.516 | 10.516 | +0.000 | -30.424 | -30.424 | -0.000 |
| a | cmmi5 gid 97 | 23.027 | 23.027 | +0.000 | -8.462 | -8.462 | +0.000 |
| b | cmmi5 gid 98 | 23.379 | 23.379 | -0.000 | -13.539 | -13.539 | -0.000 |
| c | cmmi7 gid 99 | 23.176 | 23.176 | -0.000 | -20.063 | -20.063 | -0.000 |
| d | cmmi7 gid 100 | 22.882 | 22.882 | +0.000 | -27.696 | -27.696 | +0.000 |
| e | cmmi5 gid 101 | 23.270 | 23.270 | -0.000 | -32.376 | -32.376 | -0.000 |
| f | cmmi5 gid 102 | 22.958 | 22.958 | +0.000 | -37.454 | -37.454 | -0.000 |
| } | cmex10 gid 57 | 30.540 | 30.540 | +0.000 | -3.525 | -3.525 | +0.000 |
| } | cmex10 gid 61 | 30.540 | 30.540 | +0.000 | -12.491 | -12.491 | -0.000 |
| } | cmex10 gid 59 | 30.540 | 30.540 | +0.000 | -30.424 | -30.424 | -0.000 |

| rule | ours x | ref x | Δx | ours centre y | ref centre y | Δy | ours w | ref w | Δw | ours h | ref h | Δh |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 10.516 | 10.516 | +0.000 | 0.199 | 0.199 | +0.000 | 28.880 | 28.880 | -0.000 | 0.398 | 0.398 | +0.000 |
| 1 | 23.027 | 23.027 | +0.000 | -9.395 | -9.395 | -0.000 | 3.858 | 3.858 | -0.000 | 0.398 | 0.398 | +0.000 |
| 2 | 21.832 | 21.832 | -0.000 | -14.137 | -14.137 | -0.000 | 6.249 | 6.249 | -0.000 | 0.398 | 0.398 | +0.000 |
| 3 | 20.567 | 20.567 | +0.000 | -21.458 | -21.458 | +0.000 | 8.778 | 8.778 | -0.000 | 0.398 | 0.398 | +0.000 |
| 4 | 21.763 | 21.763 | -0.000 | -29.128 | -29.127 | -0.001 | 6.386 | 6.386 | +0.000 | 0.398 | 0.398 | +0.000 |
| 5 | 22.958 | 22.958 | +0.000 | -33.310 | -33.310 | -0.000 | 3.995 | 3.995 | +0.000 | 0.398 | 0.398 | +0.000 |

Largest absolute deviation in formula G: 0.001 bp; limitations: none

PDFKit cross-check (content-stream x vs PDFKit box left edge, same page):
| ref glyph | stream x | PDFKit left | Δ | PDFKit width | stream advance | Δ |
|---|---|---|---|---|---|---|
| cmex10 58 | 197.377 | 197.377 | +0.000 | 8.856 | 8.856 | +0.000 |
| cmmi5 97 | 209.888 | 210.240 | +0.352 | 3.155 | 3.858 | -0.703 |
| cmmi5 98 | 210.240 | 210.240 | +0.000 | 3.155 | 3.155 | -0.000 |
| cmmi7 99 | 210.037 | 210.037 | +0.000 | 3.560 | 3.560 | +0.000 |
| cmmi7 100 | 209.743 | 209.743 | +0.000 | 4.147 | 4.147 | +0.000 |
| cmmi5 101 | 210.131 | 210.131 | +0.000 | 3.372 | 3.372 | +0.000 |
| cmmi5 102 | 209.819 | 209.819 | +0.000 | 3.407 | 3.407 | +0.000 |
| cmex10 59 | 217.401 | 217.401 | +0.000 | 8.856 | 8.856 | +0.000 |

## Stage 3 (FT-020 rev 3): declared visual corpus with structural, raster and parity gates

The rev 1/2 formulas became committed fixtures and eight more cases were
added. Every case is one line of a 12pt `article` (`margin=1in`, real
Computer Modern OT1 fonts, `\pagestyle{empty}`), and three gates run over
the same 15 cases from one command, `tools/run_visual.sh`:

| gate | tool | what is compared | numbers at `29e63ca` |
| --- | --- | --- | --- |
| structural | `tools/structural_gate.py --regress fixtures/visual/structural-baseline.json` | every glyph origin (font, glyph id, x, baseline) and rule (x, centre line, length, thickness) of `flashtex-math-corpus --runs` against the pdfTeX oracle for the fixture as committed; page placement of the first glyph absolutely, everything else relative to it; per-case thresholds in `fixtures/visual/thresholds.json` (0.01 bp, placement 0.05 bp) and a 0.001 bp regression tolerance against the recorded baseline | 15/15 pass; corpus max \|Δ\| **0.0055 bp**; placement \|Δ\| ≤ 0.0005 bp |
| box parity (preview = PDF) | `tools/box_parity.py` | the runtime-v1 `compile_result` the corpus compiler emits (one `text` item per glyph with `font-hints-v1`, one `rule` item per rule with `rules-v1`) against (a) the `Td` origins and `re` rectangles the pinned `crates/pdf` (4bd8c2e) wrote and (b) the boxes a CoreText draw of the same result (`tools/coretext_boxes.swift`: `CTLineDraw` at `x_pt`/`baseline_y_pt`, one fill per rule, into a CoreGraphics PDF) actually used; tolerance 0.05 pt | 15/15 pass; PDF writer max \|Δ\| **0.00096 pt** (its three-decimal rounding), CoreText **0.00000 pt** |
| raster | the visual-oracle harness (`tests/visual-corpus/harness` at `db18236`, exported read-only) with `fixtures/visual/raster-thresholds.json` and `--regress` | FlashTeX export raster (flashtex-pdf output) and preview-equivalent raster against the pdflatex reference rasters, SSIM/diff-mean/above-threshold limits | **not regenerable in this session** — see below |

Evidence: `docs/visual-evidence/20260912T070234Z/` (`structural.md`,
`box-parity.md`/`.json`, the harness's `report.md`, `metrics.json`,
`provenance.json`). The oracle report for the structural gate is also kept
at `docs/oracle/structural-report.md`.

### The corpus (`fixtures/visual/*.tex`, `*.meta.json`)

| case | body | exercises |
| --- | --- | --- |
| 01-stacked-fraction | `\displaystyle\frac{\frac{a}{b}}{c}=1` | formula A |
| 02-sqrt-left-right | `\displaystyle\sqrt{x}+\left(\frac{a}{b}\right)` | formula B |
| 03-sum-limits-scripts | `\displaystyle\sum_{i=1}^{n}x_i^2` | formula C |
| 04-tall-braces | `\displaystyle\left\{\frac{\frac{\frac{a}{b}}{c}}{\frac{d}{\frac{e}{f}}}\right\}` | formula D, extensible brace recipe |
| 05-cube-root | `\displaystyle\sqrt[3]{\frac{a}{b}}` | formula E |
| 06-lim-sin | `\displaystyle\lim_{x\to 0}\frac{\sin x}{x}` | formula F |
| 07-tall-sqrt-braces | `\displaystyle\sqrt{\left\{…\right\}}` | formula G, extensible radical |
| 08-nested-scripts | `\displaystyle x^{y^z}_{i_j}` | script and scriptscript sizes, sup/sub separation |
| 09-int-display | `\displaystyle\int_0^1 f(x)` | `\nolimits` large operator, italic-correction script offset |
| 10-left-bracket-frac-squared | `\displaystyle\left[\frac{a}{b}\right]^2` | scripts on an Inner box (sup_drop), bracket sizing |
| 11-accents | `\displaystyle\hat{\imath}+\vec{x}` | accent skew centring, cmmi accent |
| 12-sum-limits-inline | `\sum\limits_{i=1}^{n}x_i` | limits on the small operator in text style |
| 13-bigop-scripts-inline | `\prod_{k=1}^{m}a_k` | box-nucleus scripts (sup_drop/sub_drop) in text style |
| 14-mixed-text-math | `Let $x^2+y^2=z^2$ hold.` | roman words with interword space around an inline formula |
| 15-nested-fraction-sum | `\displaystyle\frac{a+b}{\frac{c}{d}+e}` | text-style inner fractions inside a display fraction |

### Pinned oracle

`fixtures/visual/oracle-geometry.json` pins the reference geometry of all
15 cases, extracted from the pdfTeX 1.40.29 (TeX Live 2026, format
`pdflatex 2026.3.1`, run of 12 Sep 2026 02:43) PDFs of the fixtures as
committed, keyed by each fixture's SHA-256. `structural_gate.py` compiles
with pdflatex when it is installed and otherwise uses the pin, and refuses a
pinned case whose fixture has changed. A baseline regenerated from the pin
is byte-identical to the committed `structural-baseline.json`, so the pinned
numbers are the numbers the live oracle produced.

### Tuning done from corpus deltas (before → after, structural gate)

- Accent centring (`11-accents`): the accent was centred on the character's
  width without its italic correction; TeX centres on the `char_box` width,
  which includes it (tex.web §738). max \|Δ\| **0.9176 → 0.0055 bp**.
- Text operators (`06-lim-sin`): the last character's italic correction is
  now kept (tex.web §752); no numeric change for cmr12, where it is zero,
  but the box width is now TeX's. The remaining 0.08 bp on this case came
  from an `lmodern` oracle preamble (OT1 `lmr12` heights differ from
  `cmr12`), fixed by declaring the structural preamble as real CM; the
  engine was right.
- No other case exceeded pdfTeX's three-decimal rounding, so nothing else
  was tuned. The residual 0.0055 bp (`ı` after `\hat` in 11-accents) is the
  skew-kern rounding of the reference's own output and stays under the
  0.01 bp threshold; it is recorded, not hidden.

### What could not be regenerated in this session, and why

- **Raster reference overlays**: `pdflatex` is not installed on
  `mac-m1max-a` (BasicTeX was removed; MacTeX is pending), so the harness
  reports every reference engine as unavailable and produces no reference
  rasters, overlays or SSIM numbers. `report.md` in the evidence directory
  therefore contains only the FlashTeX-side rasters and the harness's own
  export-vs-preview-equivalent rows. Those rows read DIFFERENT (a few
  hundred pixels per page) by design at harness `db18236`: its
  preview-equivalent draw uses Times-Roman for every text item and skips
  typed `rule` items, while the export side embeds Latin Modern Roman via
  the font hints and draws the rules. `box_parity.py` is the parity gate
  for this corpus for that reason. The `raster-thresholds.json` values are
  still the declared placeholders; they will be set from measured numbers
  with ~10–20% headroom on the first run with an oracle present.
- **Structural oracle**: not re-run; the pinned geometry above stands in,
  with the pdfTeX banner and PDF SHA-256 of the run it came from.
- **Faces on the parity check**: the PDF side embedded `LMRoman10-Regular/
  Italic` from a local Latin Modern directory (`FLASHTEX_LM_DIR`), and the
  CoreText side had no system-installed Latin Modern, so it drew
  Times-Roman/Times-Italic at the same origins. Origins, sizes and rule
  rectangles are what the gate compares; glyph outlines differ between the
  two faces and are reported per item, not gated.
- The earlier WIP evidence run `20260912T064724Z` was deleted: every case
  had failed the corpus lookup (fixed in `748527c`) and no oracle was
  present, so it measured nothing.
