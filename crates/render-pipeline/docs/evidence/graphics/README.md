# Inline graphics and graphics box transforms vs pdfLaTeX

Gate: `tests/graphics_oracle.rs` (cargo never runs TeX). References:
`fixtures/graphics/reference/NN-*.json`, made by `fixtures/graphics/oracle.py`
with MacTeX 2026 `pdflatex` (pdfTeX 3.141592653-2.6-1.40.29) from the
uncompressed content streams (`\pdfcompresslevel=0 \pdfobjcompresslevel=0`):
glyph origins and advance vectors (`q/Q`, `cm`, `Tf`, `Td`, `TJ`), image
unit-square transforms (`Do`, form `/BBox`/`/Matrix`), clipping forms, and
rules (pdfTeX strokes a rule as its centre line; recorded as rectangles).
Fixtures and images: `fixtures/graphics/make_fixtures.py` (deterministic PNGs
and an uncompressed PDF).

Tolerances: words 0.5 bp (first glyph origin), image transforms and clip
quadrilaterals 0.01 bp, rules 0.05 bp; line starts must be the same.

| fixture | pages ours/ref | words | max word diff (bp) | images | max transform/clip diff (bp) | rules | line starts |
|---|---|---|---|---|---|---|---|
| 01-inline-heights | 1/1 | 52/52 | 0.005 | 3/3 | 0.0026 | 0/0 | same |
| 02-inline-breaks | 1/1 | 22/22 | 0.004 | 9/9 | 0.0013 | 0/0 | same |
| 03-inline-natural | 1/1 | 6/6 | 0.003 | 4/4 | 0.0005 | 0/0 | same |
| 04-inline-keys | 1/1 | 6/6 | 0.004 | 5/5 | 0.0041 | 0/0 | same |
| 05-scalebox | 1/1 | 33/33 | 0.110 | 0/0 | 0 | 0/0 | same |
| 06-reflect-negative | 1/1 | 9/9 | 0.001 | 0/0 | 0 | 0/0 | same |
| 07-resizebox | 1/1 | 9/9 | 0.479 | 0/0 | 0 | 0/0 | same |
| 08-resizebox-star | 1/1 | 7/7 | 0.034 | 0/0 | 0 | 0/0 | same |
| 09-rotate-quarter | 1/1 | 9/9 | 0.110 | 0/0 | 0 | 0/0 | same |
| 10-rotate-angles | 1/1 | 9/9 | 0.433 | 0/0 | 0 | 0/0 | same |
| 11-rotate-origin-lr | 1/1 | 7/7 | 0.071 | 0/0 | 0 | 0/0 | same |
| 12-rotate-origin-tb | 1/1 | 11/11 | 0.143 | 0/0 | 0 | 0/0 | same |
| 13-rotate-image | 1/1 | 3/3 | 0.001 | 2/2 | 0.0012 | 0/0 | same |
| 14-scale-image | 1/1 | 4/4 | 0.005 | 3/3 | 0.0050 | 0/0 | same |
| 15-graphicx-angle | 1/1 | 5/5 | 0.001 | 4/4 | 0.0009 | 0/0 | same |
| 16-trim-clip | 1/1 | 4/4 | 0.001 | 3/3 | 0.0065 | 0/0 | same |
| 17-trim-noclip | 1/1 | 3/3 | 0.001 | 2/2 | 0.0006 | 0/0 | same |
| 18-viewport | 1/1 | 4/4 | 0.001 | 3/3 | 0.0006 | 0/0 | same |
| 19-graphicspath | 1/1 | 4/4 | 0.004 | 2/2 | 0.0003 | 0/0 | same |
| 20-draft-key | 1/1 | 5/5 | 0.002 | 0/0 | 0 | 8/8 | same |
| 21-draft-option | 1/1 | 6/6 | 0.003 | 0/0 | 0 | 8/8 | same |
| 22-nested | 1/1 | 5/5 | 0.018 | 1/1 | 0.0017 | 0/0 | same |
| 23-transform-math | 1/1 | 8/8 | 0.009 | 0/0 | 0 | 0/0 | same |
| 24-transform-breaks | 1/1 | 37/37 | 0.154 | 0/0 | 0 | 0/0 | same |

24/24 fixtures pass. The largest word offsets (0.4–0.5 bp in 07 and 10) are
inside transformed boxes: the content box's width comes from Latin Modern
TFM/OpenType metrics and the factors are floats, where TeX's `\Gscale@div`
and trig.sty's series truncate to scaled points and five decimals.

Other gates on this branch (with the compiler crate of
`agent/kabir-claude/inline-graphics-compiler` swapped into `vendor/compiler`,
uncommitted): `tests/floats_oracle.rs` prints the same table as main;
HW1/HW2 `--v2` output byte-identical to main; all 11 `fixtures/real-world`
documents byte-identical to main.
