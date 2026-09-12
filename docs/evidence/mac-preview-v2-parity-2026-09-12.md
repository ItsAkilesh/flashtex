# Mac v2 preview: zero-tolerance export/preview parity and off-main preparation

Owner: mac-preview-v2 (Claude Code subagent, parent mac-claude-a). Status: measured on
2026-09-12, branch `agent/mac-preview-v2/exact-glyphs`. Supersedes the single-fixture
"0 differing pixels" claim in bc28c59 (that held only for a synthetic list).
Linked from `coordination/mac-preview-v2.md` and the apps/mac README "v2 preview" section.

## What was compared

`V2Parity.compare` (apps/mac/Sources/FlashTeXMac/GlyphRunRenderer.swift): every page of a
prepared frame is rasterized through `GlyphRunRenderer.rasterize` (the bitmap the v2 pane
blits on screen) and, separately, the frame is exported to PDF through
`GlyphRunRenderer.pdfData` (same draw routine on a `CGContext` PDF page) and each PDF page is
rasterized back by CoreGraphics into an identically configured bitmap (sRGB, premultiplied
RGBA, antialiased, font smoothing off, subpixel positioning on). The raw RGBA bytes are
compared; any differing pixel counts. Tolerance: 0. Both bitmaps are SHA-256'd.

Inputs (SHA-256 in `mac-preview-v2-parity-2026-09-12/inputs.sha256`) were produced by
`flashtex-render --v2` built from a scratch archive of
`origin/agent/mac-render-pipeline/unified` 79ba728 with `--font-dir apps/mac/Fonts
--font-dir /usr/local/texlive/2026/texmf-dist/fonts/opentype/public/lm-math` (font files
only; no TeX engine in the path):

| document | pages | items | glyphs | fonts |
|---|---|---|---|---|
| `math.tex` (section + `\frac{a+b}{c}` + nested fraction) | 1 | 11 runs + 3 typed rules | 44 | LMRoman10-Regular, LatinModernMath-Regular, LMRoman12-Bold |
| `big.tex` (demo body × 10, 58,120 B) | 20 | 9,730 runs | 47,025 | LMRoman12-Regular/Italic/Bold |

## Results (app run, `FLASHTEX_V2_PARITY_OUT`, 2 px/pt)

| report | pages | differing pixels | preview == export SHA-256 |
|---|---|---|---|
| `parity-math-rules.json` | 1 (1224×1584 px) | 0 | yes |
| `parity-big-20pages.json` | 20 (1224×1584 px each) | 0 on every page | yes on every page |

Captured from the running app (`FLASHTEX_NO_ACTIVATE=1`, window captured by id with
`screencapture -l`): `window-big-20pages.jpg` (full window, 20-page list, Latin Modern
runs with ligatures/accents, bold/italic) and `page-math-rules-crop.png` (typed fraction
rules from the math list).

Test-suite measurements (`PreviewV2ParityTests`, plus a diagnostic sweep that was run once
and not kept):

- `display-list-v2-text.json` (pipeline 7094ef7) and `display-list-v2-math-rules.json`:
  0 differing pixels at 0.5, 0.75, 1, 1.25, 1.5, 1.6, 1.7, 1.8, 1.9, 2, 2.2, 2.5, 3, 4 px/pt.
- 20-page document, 1.00…2.00 step 0.05: 0 at 1.00, 1.15, 1.20, 1.30, 1.35, 1.40, 1.65,
  1.70, 1.75, 1.80, 1.90, 1.95, 2.00; nonzero at 1.05 (290), 1.10 (210), 1.25 (480),
  1.45 (420), 1.50 (430), 1.55 (460), 1.60 (670), 1.85 (740).

## Two measured causes of disagreement, both fixed in the shared routine

1. Rules: `CGContext.fill(rect)` (fast rectangle fill) computes edge coverage differently
   from the scan converter that replays the PDF's `re f`, giving one gray level of
   difference along every row of every rule (22 px at 1 px/pt, 45 at 2, 64 at 3 on the
   math list). Rules are now `addRect` + `fillPath`: 0 differing pixels.
2. Coordinates: CoreGraphics' PDF writer serializes numbers at 7 significant digits
   (`637.706` for 637.70595…), and a rule row whose coverage sits at a rounding boundary
   flips a level (32 px at 1.9 px/pt); glyph edges at 3–4 px/pt showed 1–5 px. Prepared
   pages now quantize every coordinate and font size through the same `%.7g` (≤5e-5 pt
   from the tick geometry, tested), so preview and export start from identical numbers.

## Remaining, documented limitation (not geometry)

At some fractional scales CoreGraphics rasterizes thin glyph stems differently through a
`CTFont` than through the PDF-embedded font: e.g. at 1.37 px/pt one 0.7 px-thick en dash
differs by up to 44 levels on 52 pixels of each even page of the 20-page list; rule and
glyph positions are identical. Subpixel positioning/quantization settings and drawing
glyph-by-glyph do not change it. The gate therefore pins 1 and 2 px/pt (the non-Retina and
Retina display scales) and reports other scales. The pane's own bitmap is rendered at
fit-scale × display scale, so the on-screen raster may use a fractional scale; the
comparison of that bitmap with the export is what `FLASHTEX_V2_PARITY_SCALE` measures.

The pipeline's `--pdf` is still the legacy v1 writer (Times-Roman/Symbol, `(?)` for math
glyphs, 3-decimal coordinates), so parity is against the Mac CoreGraphics export; the
product exporter is crates/pdf (mac-pdf lane).

## Off-main preparation, measured (release build, 20 pages)

decode + validate 990 ms (19.5 MB JSON), page preparation 208 ms — both on
`V2Loader.queue`; page raster 3 ms/page at 2 px/pt on `V2PageRasterizer.queue`; CoreGraphics
PDF export 197 ms. Main-thread paint is a bitmap blit plus highlight overlays.
Stale-load results (`V2Loader.staleResultsDropped`) and stale page bitmaps
(`V2PageRasterizer.staleBitmapsDropped`) are counted and covered by
`PreviewV2ShellTests`.
