# FlashTeX visual corpus: reference-render and raster-diff evidence

Generated 20260912T064724Z on mac-m1max-a by `tests/visual-corpus/harness/run.sh`.

**Scope statement.** These are narrow-case measurements over a small declared corpus. They never claim general pixel perfection, LaTeX compatibility, or parity outside these fixtures, these engines, this font, this page size, this DPI and these builds. The reference engines are test oracles only; FlashTeX never invokes them and remains an original Rust implementation.

**Acceptance vs diagnostics.** The only acceptance signals in this report are the exact-equality gates below (zero pixel difference between FlashTeX's export raster and its preview rasters, and raw PDF byte identity against the pinned profile). Every tolerance, threshold, SSIM, registration shift or regression comparison further down is a diagnostic to explain *why* something differs; none of them ever counts as acceptance.

## Exact-equality gates (acceptance)

| Fixture | Compiler | export = preview-equivalent | export = native preview capture | PDF bytes = pinned | PDF SHA-256 |
|---|---|---|---|---|---|
| 01-stacked-fraction | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 02-sqrt-left-right | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 03-sum-limits-scripts | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 04-tall-braces | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 05-cube-root | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 06-lim-sin | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 07-tall-sqrt-braces | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 08-nested-scripts | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 09-int-display | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 10-left-bracket-frac-squared | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 11-accents | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 12-sum-limits-inline | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 13-bigop-scripts-inline | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 14-mixed-text-math | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |
| 15-nested-fraction-sum | math | **EQUAL** (1 page) | unavailable | unpinned (no reference profile entry) | `927a55424060dbb0…` |

- Reference profile: `none`.
- Classification of native-preview differences: the native capture comes from a screen capture at the display's backing scale, resampled to the raster size, so a DIFFERENT result there is expected to be dominated by resampling and text rasterization (CoreText on screen vs CoreGraphics PDF rendering); it is reported as-is, without normalisation. Preview-equivalent vs export differences isolate the drawing path (CoreText glyph run vs the PDF writer's text operators) from any capture effects.

## Provenance

- suite_branch: `agent/mac-math-layout/math-boxes`
- suite_sha: `8ac2cb510a1339b8a1c25e3b7a12c6bc6ea58030`
- input_main_sha: `13ee88dff72631bf2bc7a7f9f9d1f3a966a13fc6`
- machine: `mac-m1max-a`
- os: `macOS 26.3.1 arm64`
- swift: `Apple Swift version 6.2.4 (swiftlang-6.2.4.1.4 clang-1700.6.4.2)`
- cargo: `cargo 1.99.0-nightly (3efb1f477 2026-07-17)`
- python: `3.12.0`
- DPI: 144.0 (every raster: CoreGraphics bitmap, sRGB IEC61966-2.1, 8-bit RGBA, white opaque background, MediaBox mapped to width_pt*144.0/72 px; text antialiased, font smoothing off, subpixel positioning on)
- Overlay/heatmap PNGs emitted for engines: pdflatex-lm, sides: export (metrics are computed for every engine and side; PNGs are downscaled by 2 until ≤45000 B)
- Pixel threshold for `above_threshold_fraction`: |Δluma| ≥ 32/255; SSIM: 8×8 blocks, K1=0.01, K2=0.03
- Arithmetic backend: Pillow ? (accelerator; identical integer results to the stdlib path)

### Reference engines (oracle only)

| Oracle | Available | Version | Body font | Preamble |
|---|---|---|---|---|
| pdflatex | NO | - | - | `` |
| pdflatex-lm | NO | - | - | `` |
| xelatex | NO | - | - | `` |
| xelatex-lm | NO | - | - | `` |
| lualatex | NO | - | - | `` |
| lualatex-lm | NO | - | - | `` |

The `-lm` oracles are the intended primary apples-to-apples target once a Latin-Modern-metrics FlashTeX pipeline exists; the Times oracles match the current compiler's Times metrics. Both are reported for every fixture.

Engine flags: ``. Page size: US letter 612×792 pt for every producer (checked per page from the MediaBox). LaTeX package versions: see `provenance.json` → `packages`.

### FlashTeX builds under test

- compiler `math`: `HEAD (this checkout)` @ `8ac2cb510a1339b8a1c25e3b7a12c6bc6ea58030` — declared-corpus lookup with rules-v1 + font-hints-v1 negotiated by a wrapper
- PDF writer: `origin/agent/mac-pdf/pdf-output` @ `4bd8c2e79f66c161b7fb6438f3262f8d980eb8c9` (`flashtex-pdf --verify --embed-font auto`; body font Times-Roman standard-14, Unicode fallback subset of auto (Latin Modern via font hints; see build.json pdf_stderr))
- export raster: the flashtex-pdf PDF rasterized by the same CoreGraphics rasterizer as the references
- preview-equivalent raster: `rasterize preview` re-implements the Mac app's `PDFExport.render` draw (CoreText `Times-Roman` at x_pt/baseline_y_pt/font_size_pt, U+2500 runs as 0.5em×0.0857em rules) straight into the bitmap. It links nothing from apps/mac and is **not** the SwiftUI preview; it is labelled preview-equivalent throughout.
- native preview capture: not part of this run (see limitations).

### Fixtures

| Fixture | SHA-256 | Purpose |
|---|---|---|
| `01-stacked-fraction.tex` | `145d499fbfd06188…` | Stacked fraction (text-style inner fraction inside a display fraction), Inner-Rel-Ord spacing; rev-1 oracle formula A. |
| `02-sqrt-left-right.tex` | `c0a86e0368992f7a…` | Radical overbar geometry and \left\right delimiter sizing around a fraction; rev-1 formula B. |
| `03-sum-limits-scripts.tex` | `97aac8d0f4a5e1fa…` | Display \sum with limits above/below plus sub/superscripts on a character; rev-1 formula C. |
| `04-tall-braces.tex` | `bb6c1d5a07580a90…` | 36pt fraction stack forcing the extensible cmex brace recipe (top/mid/bot pieces); rev-2 formula D. |
| `05-cube-root.tex` | `1d0a8eeb7f76d861…` | Radical with degree (LaTeX \r@@t: 5mu, raise 0.6(h-d), -10mu); rev-2 formula E. |
| `06-lim-sin.tex` | `9e29622097698ede…` | Text operator with a lower limit, \nolimits operator with thin space, script-style Rel spacing; rev-2 formula F. |
| `07-tall-sqrt-braces.tex` | `dc1f28985d98dece…` | Extensible radical (top/rep/bot pieces) over the extensible brace stack; rev-2 formula G. |
| `08-nested-scripts.tex` | `a5ee0d5bdb690ce4…` | Nested superscripts and subscripts on one nucleus: script and scriptscript sizes, sup/sub separation. |
| `09-int-display.tex` | `06a8ce2184fe34cd…` | Display \int (\nolimits, large variant) with italic-correction offset between the scripts; f italic correction before (. |
| `10-left-bracket-frac-squared.tex` | `80eb087ff957b1ca…` | Scripts on a \left\right Inner atom: sup_drop from the box height, bracket sizing. |
| `11-accents.tex` | `5bfdb0214470fbf6…` | Accents with skew: \hat over dotless i and \vec (cmmi accent) over x. |
| `12-sum-limits-inline.tex` | `567572f5327b3f10…` | \sum\limits in text style: limits above/below at text size with the small operator. |
| `13-bigop-scripts-inline.tex` | `c5c70de4653a424d…` | Big operator with both limits as scripts in text style (box nucleus: sup_drop/sub_drop). |
| `14-mixed-text-math.tex` | `c2f47b726e5d73ae…` | Mixed text/math line: roman words with interword space around an inline formula; math spacing of Bin/Rel. |
| `15-nested-fraction-sum.tex` | `cbf509b2e4a806ad…` | Binom-free stacked fractions: a sum in the numerator, a fraction plus a term in the denominator (text-style inner). |

Full SHAs and `.meta.json` contents are in `provenance.json`. Only the body after `\begin{document}` is shared by every producer; the harness substitutes the engine preamble and strips it for FlashTeX.

## Diagnostic: export comparison (flashtex-pdf PDF vs reference PDF, both rasterized identically)

This is the PDF-output comparison against the oracle. Word boxes come from PDFKit on both PDFs; rules are ink rows ≥10pt long. Diagnostic only.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| | max | differing | ≥thr | SSIM₈ | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-stacked-fraction | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 01-stacked-fraction | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 02-sqrt-left-right | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 02-sqrt-left-right | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 03-sum-limits-scripts | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 03-sum-limits-scripts | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 04-tall-braces | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 04-tall-braces | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 05-cube-root | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 05-cube-root | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 06-lim-sin | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 06-lim-sin | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 07-tall-sqrt-braces | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 07-tall-sqrt-braces | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 08-nested-scripts | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 08-nested-scripts | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 09-int-display | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 09-int-display | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 10-left-bracket-frac-squared | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 10-left-bracket-frac-squared | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 11-accents | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 11-accents | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 12-sum-limits-inline | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 12-sum-limits-inline | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 13-bigop-scripts-inline | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 13-bigop-scripts-inline | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 14-mixed-text-math | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 14-mixed-text-math | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 15-nested-fraction-sum | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 15-nested-fraction-sum | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |

## Diagnostic: preview-equivalent comparison (CoreText draw of compile_result vs reference PDF raster)

Weaker than a capture of the real preview: it re-implements the app's draw code path rather than exercising the SwiftUI Canvas. Word-box metrics are not available for this side (no PDF), so they are omitted.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| | max | differing | ≥thr | SSIM₈ | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-stacked-fraction | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 01-stacked-fraction | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 02-sqrt-left-right | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 02-sqrt-left-right | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 03-sum-limits-scripts | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 03-sum-limits-scripts | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 04-tall-braces | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 04-tall-braces | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 05-cube-root | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 05-cube-root | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 06-lim-sin | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 06-lim-sin | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 07-tall-sqrt-braces | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 07-tall-sqrt-braces | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 08-nested-scripts | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 08-nested-scripts | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 09-int-display | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 09-int-display | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 10-left-bracket-frac-squared | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 10-left-bracket-frac-squared | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 11-accents | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 11-accents | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 12-sum-limits-inline | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 12-sum-limits-inline | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 13-bigop-scripts-inline | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 13-bigop-scripts-inline | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 14-mixed-text-math | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 14-mixed-text-math | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 15-nested-fraction-sum | pdflatex | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |
| 15-nested-fraction-sum | pdflatex-lm | math | 0/1 | failed | - | - | - | - | - | -/-/- | - | - | - | - | -/- | - |

## Per-fixture diagnostic details (export side)

### 01-stacked-fraction — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 01-stacked-fraction — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 02-sqrt-left-right — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 02-sqrt-left-right — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 03-sum-limits-scripts — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 03-sum-limits-scripts — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 04-tall-braces — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 04-tall-braces — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 05-cube-root — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 05-cube-root — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 06-lim-sin — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 06-lim-sin — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 07-tall-sqrt-braces — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 07-tall-sqrt-braces — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 08-nested-scripts — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 08-nested-scripts — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 09-int-display — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 09-int-display — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 10-left-bracket-frac-squared — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 10-left-bracket-frac-squared — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 11-accents — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 11-accents — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 12-sum-limits-inline — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 12-sum-limits-inline — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 13-bigop-scripts-inline — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 13-bigop-scripts-inline — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 14-mixed-text-math — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 14-mixed-text-math — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 15-nested-fraction-sum — pdflatex vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

### 15-nested-fraction-sum — pdflatex-lm vs compiler `math` (export)

- FlashTeX diagnostics (1): error: flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF

## Diagnostic thresholds (never acceptance)

Thresholds file: `harness/raster-thresholds.json` (copied here as `thresholds.used.json`). A failure here is an acceptance signal for the narrow case only.


## Limitations and honesty notes

- The reference preambles are the harness's (12pt times / lmodern); the corpus compiler lays out with Computer Modern metrics (CmMathMetrics::latex_12pt). Latin Modern text glyph heights and italic corrections differ from cmr12 in places, and the times variant's text font is not CM at all, so raster metrics mix font differences with layout differences; the structural gate (tools/structural_gate.py, real CM oracle) isolates layout.
- The harness's preview-equivalent raster draws every text item in Times-Roman and skips typed rule items; the export side (flashtex-pdf) draws typed rules and resolves Latin Modern font hints, substituting the document face for the math symbol and extension families with a warning.
- Metrics are for these fixtures, this DPI, these builds and this machine only.
- PDFKit exposes text selections only; it cannot report rule/line geometry from the reference PDFs, so rule presence and position are compared from ink rows of the rasters (≥10pt contiguous dark run), and FlashTeX's own rectangle geometry (`re f`) is listed only for cross-checking.
- The pdflatex reference uses the URW Nimbus Roman clone (`times` package), xelatex/lualatex use the macOS Times New Roman TrueType, and FlashTeX's PDF uses the standard-14 `Times-Roman` name resolved by the rasterizer's CoreGraphics PDF engine (plus an embedded Times New Roman subset for non-WinAnsi glyphs). Glyph outlines therefore differ slightly even where positions agree; the metrics include that font-substitution noise.
- FlashTeX has no justification, hyphenation, kerning or ligatures; line breaks and word x positions diverge progressively along a line. Page-level SSIM over text is dominated by that, not by glyph rendering.
- Missing pages (page-count mismatch) are reported, not silently skipped.
