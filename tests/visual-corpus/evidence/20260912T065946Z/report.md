# FlashTeX visual corpus: reference-render and raster-diff evidence

Generated 20260912T065946Z on mac-m1max-a by `tests/visual-corpus/harness/run.sh`.

**Scope statement.** These are narrow-case measurements over a small declared corpus. They never claim general pixel perfection, LaTeX compatibility, or parity outside these fixtures, these engines, this font, this page size, this DPI and these builds. The reference engines are test oracles only; FlashTeX never invokes them and remains an original Rust implementation.

**Acceptance vs diagnostics.** The only acceptance signals in this report are the exact-equality gates below (zero pixel difference between FlashTeX's export raster and its preview rasters, and raw PDF byte identity against the pinned profile). Every tolerance, threshold, SSIM, registration shift or regression comparison further down is a diagnostic to explain *why* something differs; none of them ever counts as acceptance.

## Exact-equality gates (acceptance)

| Fixture | Compiler | export = preview-equivalent | export = native preview capture | PDF bytes = pinned | PDF SHA-256 |
|---|---|---|---|---|---|
| 01-plain-paragraph | de1020c | DIFFERENT: 1240/1938816 px, max |Δ| 255 | unavailable | **EQUAL** | `88deda9f24394700…` |
| 01-plain-paragraph | main | DIFFERENT: 1240/1938816 px, max |Δ| 255 | unavailable | DIFFERENT | `88deda9f24394700…` |
| 01-plain-paragraph | pipeline | DIFFERENT: 1306/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `fe37a7b6af576002…` |
| 02-wrapping-paragraph | de1020c | DIFFERENT: 17142/1938816 px, max |Δ| 255 | unavailable | **EQUAL** | `1bc0994be86d3ece…` |
| 02-wrapping-paragraph | main | DIFFERENT: 17142/1938816 px, max |Δ| 255 | unavailable | DIFFERENT | `1bc0994be86d3ece…` |
| 02-wrapping-paragraph | pipeline | DIFFERENT: 16621/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `bf3b703d2e9df65a…` |
| 03-section-heading | de1020c | DIFFERENT: 2140/1938816 px, max |Δ| 6 | unavailable | **EQUAL** | `7f0f7247434d3132…` |
| 03-section-heading | main | DIFFERENT: 2117/1938816 px, max |Δ| 6 | unavailable | DIFFERENT | `599a00dbcb62e333…` |
| 03-section-heading | pipeline | DIFFERENT: 1819/1938816 px, max |Δ| 6 | unavailable | unpinned (no reference profile entry) | `fa6a6c20909f1dca…` |
| 04-bold-emph | de1020c | DIFFERENT: 1261/1938816 px, max |Δ| 4 | unavailable | **EQUAL** | `7a34b0a51a2992f4…` |
| 04-bold-emph | main | DIFFERENT: 1261/1938816 px, max |Δ| 4 | unavailable | DIFFERENT | `7a34b0a51a2992f4…` |
| 04-bold-emph | pipeline | DIFFERENT: 1265/1938816 px, max |Δ| 4 | unavailable | unpinned (no reference profile entry) | `1e1c1ac2acf476cb…` |
| 05-unicode | de1020c | DIFFERENT: 587/1938816 px, max |Δ| 3 | unavailable | **EQUAL** | `e721656e20b80d47…` |
| 05-unicode | main | DIFFERENT: 587/1938816 px, max |Δ| 3 | unavailable | DIFFERENT | `e721656e20b80d47…` |
| 05-unicode | pipeline | DIFFERENT: 627/1938816 px, max |Δ| 3 | unavailable | unpinned (no reference profile entry) | `8e366afd620e8e38…` |
| 06-math-inline | de1020c | DIFFERENT: 1041/1938816 px, max |Δ| 254 | unavailable | **EQUAL** | `0a1a2a6eebc4b2b1…` |
| 06-math-inline | main | DIFFERENT: 1041/1938816 px, max |Δ| 254 | unavailable | DIFFERENT | `0a1a2a6eebc4b2b1…` |
| 06-math-inline | pipeline | DIFFERENT: 731/1938816 px, max |Δ| 4 | unavailable | unpinned (no reference profile entry) | `5a024e79dd2f53ff…` |
| 07-math-display | de1020c | DIFFERENT: 1021/1938816 px, max |Δ| 255 | unavailable | **EQUAL** | `915882632c74f40b…` |
| 07-math-display | main | DIFFERENT: 1021/1938816 px, max |Δ| 255 | unavailable | DIFFERENT | `0f8277b1aac1eab4…` |
| 07-math-display | pipeline | DIFFERENT: 503/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `0cdd6abe32609e07…` |
| 08-two-page | de1020c | DIFFERENT: 58237/1938816 px, max |Δ| 255 | unavailable | **EQUAL** | `6e8c29a5f28f5429…` |
| 08-two-page | main | DIFFERENT: 58237/1938816 px, max |Δ| 255 | unavailable | DIFFERENT | `6e8c29a5f28f5429…` |
| 08-two-page | pipeline | DIFFERENT: 56755/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `7623c08367486055…` |
| 09-mixed-document | de1020c | DIFFERENT: 4608/1938816 px, max |Δ| 250 | unavailable | **EQUAL** | `eb9ed31b44ff065e…` |
| 09-mixed-document | main | DIFFERENT: 4600/1938816 px, max |Δ| 250 | unavailable | DIFFERENT | `fc8d547cb07b446a…` |
| 09-mixed-document | pipeline | DIFFERENT: 4373/1938816 px, max |Δ| 243 | unavailable | unpinned (no reference profile entry) | `0488b77bad096b67…` |
| 10-unicode-paragraph | de1020c | DIFFERENT: 9269/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `405e9d22c4c242b1…` |
| 10-unicode-paragraph | main | DIFFERENT: 9269/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `405e9d22c4c242b1…` |
| 10-unicode-paragraph | pipeline | DIFFERENT: 8619/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `8f19f50f143b15ef…` |
| 11-nested-lists | de1020c | DIFFERENT: 2764/1938816 px, max |Δ| 6 | unavailable | unpinned (no reference profile entry) | `dddb38b9e9050f15…` |
| 11-nested-lists | main | DIFFERENT: 3060/1938816 px, max |Δ| 7 | unavailable | unpinned (no reference profile entry) | `2be630dc95b8a81e…` |
| 11-nested-lists | pipeline | DIFFERENT: 2658/1938816 px, max |Δ| 6 | unavailable | unpinned (no reference profile entry) | `e12f001072058712…` |
| 12-justified-paragraphs | de1020c | DIFFERENT: 30049/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `47e5eefae9a4319a…` |
| 12-justified-paragraphs | main | DIFFERENT: 30049/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `47e5eefae9a4319a…` |
| 12-justified-paragraphs | pipeline | DIFFERENT: 28871/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `923c51fc8ce9c2d4…` |
| 13-math-display-rich | de1020c | DIFFERENT: 1699/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `80f03c44b40cd1b1…` |
| 13-math-display-rich | main | DIFFERENT: 1699/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `0170cf520ff62e6c…` |
| 13-math-display-rich | pipeline | DIFFERENT: 797/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `1b0b4b8eac044601…` |
| 14-math-inline-dense | de1020c | DIFFERENT: 3142/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `04244b7cb4eedb5c…` |
| 14-math-inline-dense | main | DIFFERENT: 3142/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `04244b7cb4eedb5c…` |
| 14-math-inline-dense | pipeline | DIFFERENT: 1038/1938816 px, max |Δ| 5 | unavailable | unpinned (no reference profile entry) | `067b8d5c20842992…` |
| 15-three-page-sections | de1020c | DIFFERENT: 49094/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `9f2923c8d7e3dba9…` |
| 15-three-page-sections | main | DIFFERENT: 49088/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `9b490607f689e55f…` |
| 15-three-page-sections | pipeline | DIFFERENT: 51329/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `b4028c7a42d1c78f…` |
| 16-heading-page-break | de1020c | DIFFERENT: 58237/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `a1b742b01edb41a0…` |
| 16-heading-page-break | main | DIFFERENT: 58237/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `53ac9e5351991d14…` |
| 16-heading-page-break | pipeline | DIFFERENT: 56755/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `ec8e6aa2f0d1dc80…` |
| 17-apostrophes | de1020c | DIFFERENT: 1919/1938816 px, max |Δ| 4 | unavailable | unpinned (no reference profile entry) | `aed18719902a6924…` |
| 17-apostrophes | main | DIFFERENT: 1919/1938816 px, max |Δ| 4 | unavailable | unpinned (no reference profile entry) | `aed18719902a6924…` |
| 17-apostrophes | pipeline | DIFFERENT: 2264/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `20088ef3c49a335d…` |
| 18-ligatures | de1020c | DIFFERENT: 9644/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `cccda2e7ef9bd851…` |
| 18-ligatures | main | DIFFERENT: 9644/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `cccda2e7ef9bd851…` |
| 18-ligatures | pipeline | DIFFERENT: 9919/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `d4d04998e9e94f92…` |

- Reference profile: `reference-profile.json`.
- Classification of native-preview differences: the native capture comes from a screen capture at the display's backing scale, resampled to the raster size, so a DIFFERENT result there is expected to be dominated by resampling and text rasterization (CoreText on screen vs CoreGraphics PDF rendering); it is reported as-is, without normalisation. Preview-equivalent vs export differences isolate the drawing path (CoreText glyph run vs the PDF writer's text operators) from any capture effects.

## Provenance

- suite_branch: `mvo/rev2`
- suite_sha: `65f7c44a5e885c8c7a038ea196a32a928b95b0e7`
- input_main_sha: `462fb27d8cf1f8defe9a03a1be89a405619cccd7`
- machine: `mac-m1max-a`
- os: `macOS 26.3.1 arm64`
- swift: `Apple Swift version 6.2.4 (swiftlang-6.2.4.1.4 clang-1700.6.4.2)`
- cargo: `cargo 1.99.0-nightly (3efb1f477 2026-07-17)`
- python: `3.12.0`
- pillow: `12.2.0`
- DPI: 144.0 (every raster: CoreGraphics bitmap, sRGB IEC61966-2.1, 8-bit RGBA, white opaque background, MediaBox mapped to width_pt*144.0/72 px; text antialiased, font smoothing off, subpixel positioning on)
- Overlay/heatmap PNGs emitted for engines: pdflatex,pdflatex-lm, sides: export,native (metrics are computed for every engine and side; PNGs are downscaled by 2 until ≤90000 B)
- Every overlay/heatmap PNG carries a burned-in footer (and XMP dc:description) with the fixture SHA-256, oracle engine+version+font, compiler and flashtex-pdf SHAs, side, DPI/colour profile and run stamp; `metrics.json` repeats them per entry under `provenance`.
- Registration: global (dx,dy) between reference and candidate estimated by 1-D ink-projection cross-correlation (±60 pt search; scale assumed 1 because both sides are rasterized from equal MediaBoxes at the same DPI — the native capture's resample factor is recorded separately). Tables show raw error, the registration shift, and the rendering error after undoing the shift. Regions: text area (1in margins), header/footer bands, and display-math boxes derived from the reference word boxes.
- Pixel threshold for `above_threshold_fraction`: |Δluma| ≥ 32/255; SSIM: 8×8 blocks, K1=0.01, K2=0.03
- Arithmetic backend: Pillow 12.2.0 (accelerator; identical integer results to the stdlib path)

### Reference engines (oracle only)

| Oracle | Available | Version | Body font | Preamble |
|---|---|---|---|---|
| pdflatex | NO — PDFs reused from run 20260912T064032Z | pdfTeX 3.141592653-2.6-1.40.29 (TeX Live 2026) | URW Nimbus Roman (`times` package, T1 fontenc) | `\documentclass[12pt]{article} \usepackage[T1]{fontenc} \usepackage{times} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| pdflatex-lm | NO — PDFs reused from run 20260912T064032Z | pdfTeX 3.141592653-2.6-1.40.29 (TeX Live 2026) | Latin Modern Roman Type 1 (`lmodern` package, T1 fontenc) — LaTeX's default Computer Modern look | `\documentclass[12pt]{article} \usepackage[T1]{fontenc} \usepackage{lmodern} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| xelatex | NO — PDFs reused from run 20260912T064032Z | XeTeX 3.141592653-2.6-0.999998 (TeX Live 2026) | Times New Roman (`fontspec`, /System/Library/Fonts/Supplemental/Times New Roman.ttf) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{Times New Roman} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| xelatex-lm | NO — PDFs reused from run 20260912T064032Z | XeTeX 3.141592653-2.6-0.999998 (TeX Live 2026) | Latin Modern Roman OpenType (`fontspec`, lmroman12-*.otf from the TeX Live tree by explicit path; bold-italic uses lmroman10-bolditalic, the only LM bold-italic face) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{lmroman12-regular.otf}[Path=/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm/,BoldFont=lmroman12-bold.otf,ItalicFont=lmroman12-italic.otf,BoldItalicFont=lmroman10-bolditalic.otf] \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| lualatex | NO — PDFs reused from run 20260912T064032Z | This is LuaHBTeX, Version 1.24.0 (TeX Live 2026) | Times New Roman (`fontspec`, /System/Library/Fonts/Supplemental/Times New Roman.ttf) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{Times New Roman} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| lualatex-lm | NO — PDFs reused from run 20260912T064032Z | This is LuaHBTeX, Version 1.24.0 (TeX Live 2026) | Latin Modern Roman OpenType (`fontspec`, lmroman12-*.otf from the TeX Live tree by explicit path; bold-italic uses lmroman10-bolditalic, the only LM bold-italic face) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{lmroman12-regular.otf}[Path=/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm/,BoldFont=lmroman12-bold.otf,ItalicFont=lmroman12-italic.otf,BoldItalicFont=lmroman10-bolditalic.otf] \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |

### Reference availability this run

- rendered fresh by an installed engine: 0 fixture/oracle pairs
- reused from run `20260912T064032Z` (This is LuaHBTeX, Version 1.24.0 (TeX Live 2026)): 36 pairs — the PDF stored in `evidence/20260912T064032Z/references/` was reused after its recorded fixture SHA-256 and preamble matched; raster and word boxes re-derived by this run's rasterizer; the engine version above is the one that produced that PDF, not a binary present on this machine
- reused from run `20260912T064032Z` (XeTeX 3.141592653-2.6-0.999998 (TeX Live 2026)): 36 pairs — the PDF stored in `evidence/20260912T064032Z/references/` was reused after its recorded fixture SHA-256 and preamble matched; raster and word boxes re-derived by this run's rasterizer; the engine version above is the one that produced that PDF, not a binary present on this machine
- reused from run `20260912T064032Z` (pdfTeX 3.141592653-2.6-1.40.29 (TeX Live 2026)): 36 pairs — the PDF stored in `evidence/20260912T064032Z/references/` was reused after its recorded fixture SHA-256 and preamble matched; raster and word boxes re-derived by this run's rasterizer; the engine version above is the one that produced that PDF, not a binary present on this machine

The `-lm` oracles are the intended primary apples-to-apples target once a Latin-Modern-metrics FlashTeX pipeline exists; the Times oracles match the current compiler's Times metrics. Both are reported for every fixture.

Engine flags: `-interaction=batchmode -halt-on-error -file-line-error`. Page size: US letter 612×792 pt for every producer (checked per page from the MediaBox). LaTeX package versions: see `provenance.json` → `packages`.

### FlashTeX builds under test

- compiler `main`: `origin/main` @ `462fb27d8cf1f8defe9a03a1be89a405619cccd7` (crates/compiler) — Remove obsolete staffing and Cursor-only wording from agent startup instructions
- compiler `de1020c`: `de1020c` @ `de1020cd0be7cede11be2691e00e7f5b15cb2224` (crates/compiler) — compiler: add math, real font metrics and PDF output
- compiler `pipeline`: `origin/agent/mac-render-pipeline/unified` @ `7094ef74c400c42d1c42abcdef539f1fcc04a4b8` (crates/render-pipeline) — render-pipeline: resume checkpoint; drop out-of-ownership harness snapshot
- PDF writer: `5b5f7b5` @ `5b5f7b5bcbc44ba9376b3a237afe93ba1503d13c` (`flashtex-pdf --verify --embed-font auto`; body font Times-Roman standard-14, Unicode fallback subset of embedding subset of TimesNewRomanPSMT from /System/Library/Fonts/Supplemental/Times New Roman.ttf)
- export raster: the flashtex-pdf PDF rasterized by the same CoreGraphics rasterizer as the references
- preview-equivalent raster: `rasterize preview` re-implements the Mac app's `PDFExport.render` draw (CoreText `Times-Roman` at x_pt/baseline_y_pt/font_size_pt, U+2500 runs as 0.5em×0.0857em rules) straight into the bitmap. It links nothing from apps/mac and is **not** the SwiftUI preview; it is labelled preview-equivalent throughout.
- native preview capture: not part of this run (see limitations).

### Fixtures

| Fixture | SHA-256 | Purpose |
|---|---|---|
| `01-plain-paragraph.tex` | `223aac6742bf98c1…` | Single short line of body text: baseline position, left margin, glyph advance widths. |
| `02-wrapping-paragraph.tex` | `e4f5d58b6d478973…` | Two multi-line paragraphs that wrap; exercises line breaking, justification and paragraph skip. |
| `03-section-heading.tex` | `96fa8f8be5ebe02a…` | Unnumbered section headings (secnumdepth 0): heading size, bold face, vertical skips before/after. |
| `04-bold-emph.tex` | `19ba4826ae3f479f…` | Inline font switches: bold, italic, nested. |
| `05-unicode.tex` | `3656f7c08076a0b2…` | UTF-8 literals with diacritics and an em dash, plus the equivalent TeX accent commands and ---. |
| `06-math-inline.tex` | `f21876a4ef3db1c4…` | Inline math: fraction with rule, Greek letters, square root. |
| `07-math-display.tex` | `73833d4b239902af…` | Displayed equation with sum limits, scripts and a fraction; display centring and vertical skips. |
| `08-two-page.tex` | `1c7a5355db97a4ef…` | Enough text to overflow to a second page; page count, page break position, second-page top margin. |
| `09-mixed-document.tex` | `3815168adff5d76c…` | Everything together on one page: heading, font switches, Unicode, inline and display math, wrapping. |
| `10-unicode-paragraph.tex` | `c862912f9c40321e…` | Long UTF-8 paragraph: many accented letters, em/en dashes, curly and angle quotes, currency signs; wraps over several lines. |
| `11-nested-lists.tex` | `56b9511e1048111d…` | Nested itemize/enumerate: list indentation, bullets, numbering, vertical spacing. |
| `12-justified-paragraphs.tex` | `8b566fe1cee1915d…` | Four justified multi-line paragraphs: interword stretch, hyphenation, paragraph skip accumulation. |
| `13-math-display-rich.tex` | `7a0878277cb454bb…` | Display math with \int and \sum limits, \frac, \sqrt, superscripts and a \left( ... \right) pair. |
| `14-math-inline-dense.tex` | `4db4c0efbbe30c80…` | Many short inline math fragments in one wrapping line: inline math widths, script placement, line breaking around math. |
| `15-three-page-sections.tex` | `a40ef2bd5ea86493…` | Three pages, one section per page, forced with \newpage: page count, per-page heading position, top margin on pages 2-3. |
| `16-heading-page-break.tex` | `08d443b98d6f6f2a…` | A section heading that lands at the bottom of page 1: LaTeX's club/widow and heading-keep rules move it to page 2. |
| `17-apostrophes.tex` | `bcfad1ab8a669d3f…` | ASCII apostrophes and TeX quote ligatures: quotesingle vs quoteright glyph and advance width (regression for the metrics bug found in issue #2). |
| `18-ligatures.tex` | `9a9ac05f3eca9436…` | ff/fi/fl/ffi/ffl ligature words: TeX substitutes ligature glyphs, changing advance widths and line breaks. |

Full SHAs and `.meta.json` contents are in `provenance.json`. Only the body after `\begin{document}` is shared by every producer; the harness substitutes the engine preamble and strips it for FlashTeX.

## Diagnostic: export comparison (flashtex-pdf PDF vs reference PDF, both rasterized identically)

This is the PDF-output comparison against the oracle. Word boxes come from PDFKit on both PDFs; rules are ink rows ≥10pt long. Diagnostic only.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| raw | SSIM₈ raw | registration Δ pt (dx,dy per page; `weak`/`moderate` = shift explains <25% of the error) | mean\|Δ\| after reg | SSIM₈ after reg | max | differing | ≥thr | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-plain-paragraph | lualatex | de1020c | 1/1 | ok | 0.1575 | 0.9980 | (0,0) | 0.1575 | 0.9980 | 255 | 0.0023 | 0.0014 | 13/13/13 | yes | 0.29 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.1575 | 0.9980 | (0,0) | 0.1575 | 0.9980 | 255 | 0.0023 | 0.0014 | 13/13/13 | yes | 0.29 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex | pipeline | 1/1 | recovered | 0.3850 | 0.9944 | (0,0) | 0.3850 | 0.9944 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 5.45 | 0.41 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | de1020c | 1/1 | ok | 0.3662 | 0.9941 | (0,0) | 0.3662 | 0.9941 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.30 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | main | 1/1 | ok | 0.3662 | 0.9941 | (0,0) | 0.3662 | 0.9941 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.30 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 0.3718 | 0.9942 | (0,0) | 0.3718 | 0.9942 | 255 | 0.0031 | 0.0025 | 13/13/13 | yes | 6.78 | 0.36 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.1873 | 0.9975 | (0,0) | 0.1873 | 0.9975 | 255 | 0.0024 | 0.0014 | 13/13/13 | yes | 0.43 | 0.46 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.1873 | 0.9975 | (0,0) | 0.1873 | 0.9975 | 255 | 0.0024 | 0.0014 | 13/13/13 | yes | 0.43 | 0.46 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-main-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | pipeline | 1/1 | recovered | 0.3928 | 0.9945 | (0,0) | 0.3928 | 0.9945 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 5.97 | 0.41 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-pipeline-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 0.3660 | 0.9942 | (0,0) | 0.3660 | 0.9942 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | main | 1/1 | ok | 0.3660 | 0.9942 | (0,0) | 0.3660 | 0.9942 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 0.3717 | 0.9942 | (0,0) | 0.3717 | 0.9942 | 255 | 0.0031 | 0.0025 | 13/13/13 | yes | 6.79 | 0.67 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.1607 | 0.9980 | (0,0) | 0.1607 | 0.9980 | 255 | 0.0023 | 0.0014 | 13/13/13 | yes | 0.30 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.1607 | 0.9980 | (0,0) | 0.1607 | 0.9980 | 255 | 0.0023 | 0.0014 | 13/13/13 | yes | 0.30 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex | pipeline | 1/1 | recovered | 0.3843 | 0.9944 | (0,0) | 0.3843 | 0.9944 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 5.44 | 0.41 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | de1020c | 1/1 | ok | 0.3665 | 0.9941 | (0,0) | 0.3665 | 0.9941 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | main | 1/1 | ok | 0.3665 | 0.9941 | (0,0) | 0.3665 | 0.9941 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 0.3715 | 0.9942 | (0,0) | 0.3715 | 0.9942 | 255 | 0.0031 | 0.0025 | 13/13/13 | yes | 6.78 | 0.67 | 1.0000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 8.3842 | 0.8656 | (-3,0) weak | 8.3713 | 0.8657 | 255 | 0.0613 | 0.0509 | 210/210/210 | yes | 116.20 | 5.12 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 8.3842 | 0.8656 | (-3,0) weak | 8.3713 | 0.8657 | 255 | 0.0613 | 0.0509 | 210/210/210 | yes | 116.20 | 5.12 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | pipeline | 1/1 | recovered | 7.1464 | 0.8938 | (4.5,0) weak | 7.1201 | 0.8929 | 255 | 0.0551 | 0.0447 | 210/210/210 | yes | 125.76 | 2.75 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | de1020c | 1/1 | ok | 7.7868 | 0.8620 | (-0.5,-14.5) weak | 7.7691 | 0.8602 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.57 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | main | 1/1 | ok | 7.7868 | 0.8620 | (-0.5,-14.5) weak | 7.7691 | 0.8602 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.57 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 6.7833 | 0.8883 | (0.5,0) weak | 6.7070 | 0.8893 | 255 | 0.0543 | 0.0441 | 210/210/210 | yes | 85.77 | 1.87 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 8.3726 | 0.8663 | (0,0) | 8.3726 | 0.8663 | 255 | 0.0611 | 0.0509 | 210/210/210 | yes | 116.24 | 5.12 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 8.3726 | 0.8663 | (0,0) | 8.3726 | 0.8663 | 255 | 0.0611 | 0.0509 | 210/210/210 | yes | 116.24 | 5.12 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-main-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | pipeline | 1/1 | recovered | 7.1035 | 0.8947 | (0,0) | 7.1035 | 0.8947 | 255 | 0.0549 | 0.0444 | 210/210/210 | yes | 125.79 | 2.75 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-pipeline-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 7.7862 | 0.8620 | (0,0) | 7.7862 | 0.8620 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | main | 1/1 | ok | 7.7862 | 0.8620 | (0,0) | 7.7862 | 0.8620 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 6.7841 | 0.8883 | (0.5,0) weak | 6.7047 | 0.8893 | 255 | 0.0542 | 0.0441 | 210/210/210 | yes | 85.78 | 2.04 | 0.8952 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 8.3710 | 0.8656 | (0,0) | 8.3710 | 0.8656 | 255 | 0.0613 | 0.0509 | 210/210/210 | yes | 119.39 | 5.19 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 8.3710 | 0.8656 | (0,0) | 8.3710 | 0.8656 | 255 | 0.0613 | 0.0509 | 210/210/210 | yes | 119.39 | 5.19 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex | pipeline | 1/1 | recovered | 7.1269 | 0.8940 | (0,0) | 7.1269 | 0.8940 | 255 | 0.0552 | 0.0448 | 210/210/210 | yes | 128.93 | 2.82 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | de1020c | 1/1 | ok | 7.7866 | 0.8620 | (-0.5,-14.5) weak | 7.7691 | 0.8602 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | main | 1/1 | ok | 7.7866 | 0.8620 | (-0.5,-14.5) weak | 7.7691 | 0.8602 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 6.7827 | 0.8883 | (0.5,0) weak | 6.7066 | 0.8893 | 255 | 0.0543 | 0.0441 | 210/210/210 | yes | 85.77 | 2.04 | 0.8952 | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.3394 | 0.9793 | (0,52.5) moderate | 1.1861 | 0.9829 | 255 | 0.0085 | 0.0073 | 15/15/15 | yes | 0.25 | 21.02 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.3597 | 0.9789 | (0.5,52.5) moderate | 1.2078 | 0.9824 | 255 | 0.0086 | 0.0074 | 15/17/15 | no | 2.70 | 21.02 | 0.8667 | 0/0 | - |
| 03-section-heading | lualatex | pipeline | 1/1 | recovered | 0.9265 | 0.9882 | (-0.5,-1.5) weak | 0.8907 | 0.9891 | 255 | 0.0064 | 0.0055 | 15/15/15 | yes | 1.95 | 0.70 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex-lm | de1020c | 1/1 | ok | 1.2336 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | 15/15/15 | yes | 5.46 | 21.19 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex-lm | main | 1/1 | ok | 1.2425 | 0.9769 | (0,53.5) moderate | 1.1122 | 0.9812 | 255 | 0.0082 | 0.0070 | 15/17/15 | no | 6.72 | 21.19 | 0.8667 | 0/0 | - |
| 03-section-heading | lualatex-lm | pipeline | 1/1 | recovered | 0.8557 | 0.9881 | (-1,0) moderate | 0.8053 | 0.9889 | 255 | 0.0062 | 0.0052 | 15/15/15 | yes | 3.66 | 0.45 | 1.0000 | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.3496 | 0.9793 | (0.5,52.5) moderate | 1.2021 | 0.9827 | 255 | 0.0085 | 0.0072 | 15/15/15 | yes | 0.33 | 20.87 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-de1020c-export-p1-overlay.png) |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.3719 | 0.9789 | (0.5,52.5) moderate | 1.2209 | 0.9823 | 255 | 0.0086 | 0.0074 | 15/17/15 | no | 2.78 | 20.87 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-main-export-p1-overlay.png) |
| 03-section-heading | pdflatex | pipeline | 1/1 | recovered | 0.9558 | 0.9878 | (0,-2) moderate | 0.8932 | 0.9877 | 255 | 0.0066 | 0.0055 | 15/15/15 | yes | 2.03 | 0.87 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-pipeline-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | de1020c | 1/1 | ok | 1.2335 | 0.9771 | (0,54) moderate | 1.0954 | 0.9811 | 255 | 0.0082 | 0.0069 | 15/15/15 | yes | 5.46 | 22.37 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | main | 1/1 | ok | 1.2428 | 0.9769 | (0,54) moderate | 1.1201 | 0.9807 | 255 | 0.0082 | 0.0070 | 15/17/15 | no | 6.73 | 22.37 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-main-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | pipeline | 1/1 | recovered | 0.8549 | 0.9881 | (-1,0) moderate | 0.8056 | 0.9889 | 255 | 0.0062 | 0.0052 | 15/15/15 | yes | 3.67 | 0.73 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.3370 | 0.9793 | (0,52.5) moderate | 1.1868 | 0.9829 | 255 | 0.0085 | 0.0073 | 15/15/15 | yes | 0.24 | 21.02 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.3600 | 0.9789 | (0.5,52.5) moderate | 1.2088 | 0.9824 | 255 | 0.0086 | 0.0074 | 15/17/15 | no | 2.70 | 21.02 | 0.8667 | 0/0 | - |
| 03-section-heading | xelatex | pipeline | 1/1 | recovered | 0.9285 | 0.9882 | (-0.5,-1.5) weak | 0.8892 | 0.9891 | 255 | 0.0065 | 0.0055 | 15/15/15 | yes | 1.95 | 0.70 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex-lm | de1020c | 1/1 | ok | 1.2337 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | 15/15/15 | yes | 5.46 | 22.35 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex-lm | main | 1/1 | ok | 1.2424 | 0.9769 | (0,53.5) moderate | 1.1122 | 0.9812 | 255 | 0.0082 | 0.0069 | 15/17/15 | no | 6.72 | 22.35 | 0.8667 | 0/0 | - |
| 03-section-heading | xelatex-lm | pipeline | 1/1 | recovered | 0.8558 | 0.9881 | (-1,0) moderate | 0.8052 | 0.9889 | 255 | 0.0062 | 0.0052 | 15/15/15 | yes | 3.66 | 0.70 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.3808 | 0.9945 | (0,0) | 0.3808 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 3.77 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.3808 | 0.9945 | (0,0) | 0.3808 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 3.77 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | pipeline | 1/1 | recovered | 0.3955 | 0.9943 | (6,0) weak | 0.3930 | 0.9945 | 255 | 0.0032 | 0.0025 | 10/10/10 | yes | 4.88 | 0.41 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | de1020c | 1/1 | ok | 0.4244 | 0.9933 | (-49,0) weak | 0.4148 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.02 | 0.63 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | main | 1/1 | ok | 0.4244 | 0.9933 | (-49,0) weak | 0.4148 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.02 | 0.63 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | pipeline | 1/1 | recovered | 0.4305 | 0.9934 | (-35,0) weak | 0.4210 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/10/10 | yes | 12.08 | 0.68 | 1.0000 | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.3836 | 0.9944 | (-0.5,0) weak | 0.3764 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 4.04 | 0.46 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-de1020c-export-p1-overlay.png) |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.3836 | 0.9944 | (-0.5,0) weak | 0.3764 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 4.04 | 0.46 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-main-export-p1-overlay.png) |
| 04-bold-emph | pdflatex | pipeline | 1/1 | recovered | 0.3885 | 0.9944 | (0,0) | 0.3885 | 0.9944 | 255 | 0.0032 | 0.0024 | 10/10/10 | yes | 5.09 | 0.41 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-pipeline-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | de1020c | 1/1 | ok | 0.4246 | 0.9933 | (-49,0) weak | 0.4162 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.08 | 0.73 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | main | 1/1 | ok | 0.4246 | 0.9933 | (-49,0) weak | 0.4162 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.08 | 0.73 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-main-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | pipeline | 1/1 | recovered | 0.4313 | 0.9934 | (-35,0) weak | 0.4220 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/10/10 | yes | 12.13 | 0.67 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.3848 | 0.9944 | (-0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0031 | 0.0025 | 10/12/8 | no | 3.81 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.3848 | 0.9944 | (-0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0031 | 0.0025 | 10/12/8 | no | 3.81 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex | pipeline | 1/1 | recovered | 0.3962 | 0.9943 | (3,0) weak | 0.3946 | 0.9945 | 255 | 0.0032 | 0.0025 | 10/10/10 | yes | 4.91 | 0.41 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | de1020c | 1/1 | ok | 0.4245 | 0.9933 | (-49,0) weak | 0.4138 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 15.90 | 0.73 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | main | 1/1 | ok | 0.4245 | 0.9933 | (-49,0) weak | 0.4138 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 15.90 | 0.73 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | pipeline | 1/1 | recovered | 0.4257 | 0.9935 | (-35,0) weak | 0.4192 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/10/10 | yes | 11.98 | 0.67 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.2971 | 0.9955 | (0,0) | 0.2971 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.2971 | 0.9955 | (0,0) | 0.2971 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex | pipeline | 1/1 | recovered | 0.3307 | 0.9953 | (0,0) | 0.3307 | 0.9953 | 255 | 0.0028 | 0.0021 | 11/11/11 | yes | 3.98 | 0.41 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex-lm | de1020c | 1/1 | ok | 0.3619 | 0.9938 | (2.5,0) weak | 0.3494 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.30 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex-lm | main | 1/1 | ok | 0.3619 | 0.9938 | (2.5,0) weak | 0.3494 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.30 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex-lm | pipeline | 1/1 | recovered | 0.3558 | 0.9943 | (-5.5,0) moderate | 0.3116 | 0.9949 | 255 | 0.0029 | 0.0023 | 11/11/11 | yes | 3.39 | 0.36 | 1.0000 | 0/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.3127 | 0.9955 | (0,0) | 0.3127 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.88 | 0.46 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-de1020c-export-p1-overlay.png) |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.3127 | 0.9955 | (0,0) | 0.3127 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.88 | 0.46 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-main-export-p1-overlay.png) |
| 05-unicode | pdflatex | pipeline | 1/1 | recovered | 0.3413 | 0.9954 | (3.5,0) weak | 0.3331 | 0.9956 | 255 | 0.0028 | 0.0021 | 11/11/11 | yes | 4.18 | 0.41 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-pipeline-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | de1020c | 1/1 | ok | 0.3617 | 0.9938 | (2.5,0) weak | 0.3496 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | main | 1/1 | ok | 0.3617 | 0.9938 | (2.5,0) weak | 0.3496 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-main-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | pipeline | 1/1 | recovered | 0.3558 | 0.9943 | (-5.5,0) moderate | 0.3117 | 0.9949 | 255 | 0.0029 | 0.0023 | 11/11/11 | yes | 3.40 | 0.67 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.2963 | 0.9955 | (0,0) | 0.2963 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.2963 | 0.9955 | (0,0) | 0.2963 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex | pipeline | 1/1 | recovered | 0.3300 | 0.9953 | (0,0) | 0.3300 | 0.9953 | 255 | 0.0028 | 0.0021 | 11/11/11 | yes | 3.96 | 0.41 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex-lm | de1020c | 1/1 | ok | 0.3620 | 0.9938 | (2.5,0) weak | 0.3493 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | - |
| 05-unicode | xelatex-lm | main | 1/1 | ok | 0.3620 | 0.9938 | (2.5,0) weak | 0.3493 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | - |
| 05-unicode | xelatex-lm | pipeline | 1/1 | recovered | 0.3558 | 0.9943 | (-5.5,0) moderate | 0.3115 | 0.9949 | 255 | 0.0029 | 0.0023 | 11/11/11 | yes | 3.39 | 0.67 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.3832 | 0.9934 | (0,3.5) | 0.2554 | 0.9959 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | ok | 0.3832 | 0.9934 | (0,3.5) | 0.2554 | 0.9959 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | pipeline | 1/1 | recovered | 0.2550 | 0.9956 | (0.5,0) weak | 0.2540 | 0.9957 | 255 | 0.0022 | 0.0017 | 15/10/9 | no | 30.26 | 2.18 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | de1020c | 1/1 | ok | 0.3682 | 0.9931 | (-32.5,3.5) moderate | 0.3134 | 0.9945 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 12.25 | 2.20 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | main | 1/1 | ok | 0.3682 | 0.9931 | (-32.5,3.5) moderate | 0.3134 | 0.9945 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 12.25 | 2.20 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | pipeline | 1/1 | recovered | 0.2818 | 0.9949 | (-4,0) weak | 0.2728 | 0.9951 | 255 | 0.0023 | 0.0018 | 15/10/9 | no | 40.10 | 2.16 | 1.0000 | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.3821 | 0.9936 | (0,3.5) | 0.2595 | 0.9960 | 255 | 0.0029 | 0.0023 | 13/14/10 | no | 3.89 | 2.69 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-de1020c-export-p1-overlay.png) |
| 06-math-inline | pdflatex | main | 1/1 | ok | 0.3821 | 0.9936 | (0,3.5) | 0.2595 | 0.9960 | 255 | 0.0029 | 0.0023 | 13/14/10 | no | 3.89 | 2.69 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-main-export-p1-overlay.png) |
| 06-math-inline | pdflatex | pipeline | 1/1 | recovered | 0.2594 | 0.9957 | (0,0) | 0.2594 | 0.9957 | 255 | 0.0022 | 0.0016 | 13/10/8 | no | 27.82 | 2.11 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-pipeline-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | de1020c | 1/1 | ok | 0.3680 | 0.9931 | (9.5,3.5) moderate | 0.3234 | 0.9943 | 255 | 0.0029 | 0.0023 | 14/14/12 | no | 13.02 | 2.53 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | main | 1/1 | ok | 0.3680 | 0.9931 | (9.5,3.5) moderate | 0.3234 | 0.9943 | 255 | 0.0029 | 0.0023 | 14/14/12 | no | 13.02 | 2.53 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-main-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | pipeline | 1/1 | recovered | 0.2818 | 0.9949 | (-7,0) weak | 0.2780 | 0.9949 | 255 | 0.0023 | 0.0018 | 14/10/9 | no | 40.12 | 2.23 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.3831 | 0.9934 | (0,3.5) | 0.2548 | 0.9959 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | ok | 0.3831 | 0.9934 | (0,3.5) | 0.2548 | 0.9959 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex | pipeline | 1/1 | recovered | 0.2548 | 0.9956 | (0.5,0) weak | 0.2536 | 0.9957 | 255 | 0.0022 | 0.0016 | 15/10/9 | no | 30.26 | 2.18 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | de1020c | 1/1 | ok | 0.3682 | 0.9931 | (9.5,3.5) moderate | 0.3233 | 0.9943 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 12.25 | 2.44 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | main | 1/1 | ok | 0.3682 | 0.9931 | (9.5,3.5) moderate | 0.3233 | 0.9943 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 12.25 | 2.44 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | pipeline | 1/1 | recovered | 0.2818 | 0.9949 | (-4,0) weak | 0.2728 | 0.9951 | 255 | 0.0023 | 0.0018 | 15/10/9 | no | 40.10 | 2.23 | 1.0000 | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.2595 | 0.9950 | (0,0) | 0.2595 | 0.9950 | 255 | 0.0026 | 0.0019 | 13/10/7 | no | 0.60 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | ok | 0.2723 | 0.9946 | (0,0) | 0.2723 | 0.9946 | 255 | 0.0027 | 0.0020 | 13/11/7 | no | 0.60 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex | pipeline | 1/1 | recovered | 0.3472 | 0.9925 | (0,0) | 0.3472 | 0.9925 | 255 | 0.0027 | 0.0022 | 13/6/6 | no | 2.87 | 17.60 | 1.0000 | 3/0 | - |
| 07-math-display | lualatex-lm | de1020c | 1/1 | ok | 0.3495 | 0.9931 | (0,0) | 0.3495 | 0.9931 | 255 | 0.0029 | 0.0024 | 13/10/7 | no | 2.12 | 0.92 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex-lm | main | 1/1 | ok | 0.3623 | 0.9927 | (0,0) | 0.3623 | 0.9927 | 255 | 0.0030 | 0.0024 | 13/11/7 | no | 2.12 | 0.92 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex-lm | pipeline | 1/1 | recovered | 0.3284 | 0.9924 | (0,0) | 0.3284 | 0.9924 | 255 | 0.0026 | 0.0022 | 13/6/6 | no | 1.52 | 18.00 | 1.0000 | 3/0 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.2609 | 0.9949 | (0,0) | 0.2609 | 0.9949 | 255 | 0.0026 | 0.0019 | 13/10/7 | no | 0.59 | 1.41 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-de1020c-export-p1-overlay.png) |
| 07-math-display | pdflatex | main | 1/1 | ok | 0.2737 | 0.9945 | (0,0) | 0.2737 | 0.9945 | 255 | 0.0027 | 0.0020 | 13/11/7 | no | 0.59 | 1.41 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-main-export-p1-overlay.png) |
| 07-math-display | pdflatex | pipeline | 1/1 | recovered | 0.3410 | 0.9926 | (0,0) | 0.3410 | 0.9926 | 255 | 0.0026 | 0.0021 | 13/6/6 | no | 2.86 | 17.50 | 1.0000 | 3/0 | [p1](images/07-math-display/pdflatex-pipeline-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | de1020c | 1/1 | ok | 0.3528 | 0.9931 | (0,0) | 0.3528 | 0.9931 | 255 | 0.0029 | 0.0024 | 13/10/7 | no | 2.13 | 1.73 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | main | 1/1 | ok | 0.3656 | 0.9927 | (0,0) | 0.3656 | 0.9927 | 255 | 0.0030 | 0.0025 | 13/11/7 | no | 2.13 | 1.73 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-lm-main-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | pipeline | 1/1 | recovered | 0.3282 | 0.9923 | (0,0) | 0.3282 | 0.9923 | 255 | 0.0027 | 0.0022 | 13/6/6 | no | 1.52 | 17.44 | 1.0000 | 3/0 | [p1](images/07-math-display/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.2589 | 0.9950 | (0,0) | 0.2589 | 0.9950 | 255 | 0.0026 | 0.0019 | 14/10/7 | no | 0.59 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | ok | 0.2717 | 0.9946 | (0,0) | 0.2717 | 0.9946 | 255 | 0.0027 | 0.0020 | 14/11/7 | no | 0.59 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex | pipeline | 1/1 | recovered | 0.3473 | 0.9925 | (0,0) | 0.3473 | 0.9925 | 255 | 0.0027 | 0.0022 | 14/6/6 | no | 2.86 | 17.60 | 1.0000 | 3/0 | - |
| 07-math-display | xelatex-lm | de1020c | 1/1 | ok | 0.3495 | 0.9931 | (0,0) | 0.3495 | 0.9931 | 255 | 0.0029 | 0.0024 | 14/10/7 | no | 2.12 | 1.54 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex-lm | main | 1/1 | ok | 0.3623 | 0.9927 | (0,0) | 0.3623 | 0.9927 | 255 | 0.0030 | 0.0024 | 14/11/7 | no | 2.12 | 1.54 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex-lm | pipeline | 1/1 | recovered | 0.3285 | 0.9924 | (0,0) | 0.3285 | 0.9924 | 255 | 0.0026 | 0.0022 | 14/6/6 | no | 1.52 | 17.64 | 1.0000 | 3/0 | - |
| 08-two-page | lualatex | de1020c | 3/3 | ok | 26.2176 | 0.5657 | (0,0); (-3,6) moderate; (-3,34.5) moderate | 24.6323 | 0.5959 | 255 | 0.1880 | 0.1574 | 1800/1800/1800 | yes | 164.00 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | lualatex | main | 3/3 | ok | 26.2176 | 0.5657 | (0,0); (-3,6) moderate; (-3,34.5) moderate | 24.6323 | 0.5959 | 255 | 0.1880 | 0.1574 | 1800/1800/1800 | yes | 164.00 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | lualatex | pipeline | 3/3 | recovered | 21.2365 | 0.6686 | (0,0); (0,0); (0,0) | 21.2365 | 0.6686 | 255 | 0.1626 | 0.1328 | 1800/1800/1800 | yes | 176.11 | 73.33 | 0.8938 | 0/0 | - |
| 08-two-page | lualatex-lm | de1020c | 3/3 | ok | 23.7967 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7513 | 0.5869 | 255 | 0.1801 | 0.1495 | 1800/1800/1800 | yes | 142.51 | 30.71 | 0.8878 | 0/0 | - |
| 08-two-page | lualatex-lm | main | 3/3 | ok | 23.7967 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7513 | 0.5869 | 255 | 0.1801 | 0.1495 | 1800/1800/1800 | yes | 142.51 | 30.71 | 0.8878 | 0/0 | - |
| 08-two-page | lualatex-lm | pipeline | 3/3 | recovered | 19.5875 | 0.6686 | (-2.5,0) weak; (0.5,14.5) weak; (0,0) | 19.4468 | 0.6713 | 255 | 0.1568 | 0.1277 | 1800/1800/1800 | yes | 133.80 | 10.60 | 0.8868 | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/3 | ok | 26.1974 | 0.5667 | (0,-14.5) weak; (0,6) moderate; (0,34.5) moderate | 24.5504 | 0.5980 | 255 | 0.1875 | 0.1572 | 1800/1800/1800 | yes | 143.13 | 92.17 | 0.8952 | 0/0 | [p1](images/08-two-page/pdflatex-de1020c-export-p1-overlay.png) |
| 08-two-page | pdflatex | main | 3/3 | ok | 26.1974 | 0.5667 | (0,-14.5) weak; (0,6) moderate; (0,34.5) moderate | 24.5504 | 0.5980 | 255 | 0.1875 | 0.1572 | 1800/1800/1800 | yes | 143.13 | 92.17 | 0.8952 | 0/0 | [p1](images/08-two-page/pdflatex-main-export-p1-overlay.png) |
| 08-two-page | pdflatex | pipeline | 3/3 | recovered | 20.9655 | 0.6738 | (0,0); (0,0); (0,0) | 20.9655 | 0.6738 | 255 | 0.1611 | 0.1315 | 1800/1800/1800 | yes | 159.29 | 72.49 | 0.8939 | 0/0 | [p1](images/08-two-page/pdflatex-pipeline-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | de1020c | 3/3 | ok | 23.7963 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7519 | 0.5868 | 255 | 0.1801 | 0.1496 | 1800/1800/1800 | yes | 142.52 | 31.52 | 0.8878 | 0/0 | [p1](images/08-two-page/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | main | 3/3 | ok | 23.7963 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7519 | 0.5868 | 255 | 0.1801 | 0.1496 | 1800/1800/1800 | yes | 142.52 | 31.52 | 0.8878 | 0/0 | [p1](images/08-two-page/pdflatex-lm-main-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | pipeline | 3/3 | recovered | 19.5857 | 0.6686 | (-2.5,0) weak; (0.5,14.5) weak; (0,0) | 19.4431 | 0.6713 | 255 | 0.1568 | 0.1277 | 1800/1800/1800 | yes | 133.81 | 11.17 | 0.8868 | 0/0 | [p1](images/08-two-page/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 08-two-page | xelatex | de1020c | 3/3 | ok | 26.2425 | 0.5656 | (-3,-14.5) weak; (-3,6) moderate; (-3,34.5) moderate | 24.6595 | 0.5953 | 255 | 0.1881 | 0.1578 | 1800/1800/1800 | yes | 164.01 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | xelatex | main | 3/3 | ok | 26.2425 | 0.5656 | (-3,-14.5) weak; (-3,6) moderate; (-3,34.5) moderate | 24.6595 | 0.5953 | 255 | 0.1881 | 0.1578 | 1800/1800/1800 | yes | 164.01 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | xelatex | pipeline | 3/3 | recovered | 21.2017 | 0.6690 | (0,0); (0,0); (0,0) | 21.2017 | 0.6690 | 255 | 0.1626 | 0.1330 | 1800/1800/1800 | yes | 176.13 | 73.33 | 0.8938 | 0/0 | - |
| 08-two-page | xelatex-lm | de1020c | 3/3 | ok | 23.7969 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7509 | 0.5869 | 255 | 0.1801 | 0.1495 | 1800/1800/1800 | yes | 142.51 | 31.52 | 0.8878 | 0/0 | - |
| 08-two-page | xelatex-lm | main | 3/3 | ok | 23.7969 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7509 | 0.5869 | 255 | 0.1801 | 0.1495 | 1800/1800/1800 | yes | 142.51 | 31.52 | 0.8878 | 0/0 | - |
| 08-two-page | xelatex-lm | pipeline | 3/3 | recovered | 19.5872 | 0.6686 | (-2.5,0) weak; (0.5,14.5) weak; (0,0) | 19.4457 | 0.6713 | 255 | 0.1568 | 0.1277 | 1800/1800/1800 | yes | 133.80 | 11.17 | 0.8868 | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.4844 | 0.9504 | (0,38.5) | 1.7912 | 0.9702 | 255 | 0.0174 | 0.0146 | 54/54/45 | no | 2.41 | 31.76 | 0.9778 | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | ok | 2.5141 | 0.9497 | (0,38.5) | 1.8136 | 0.9695 | 255 | 0.0176 | 0.0148 | 54/56/43 | no | 2.90 | 32.26 | 0.9767 | 0/0 | - |
| 09-mixed-document | lualatex | pipeline | 1/1 | recovered | 2.1112 | 0.9610 | (0,-47) | 1.5254 | 0.9738 | 255 | 0.0152 | 0.0128 | 54/41/41 | no | 3.68 | 30.57 | 1.0000 | 0/0 | - |
| 09-mixed-document | lualatex-lm | de1020c | 1/1 | ok | 2.2538 | 0.9494 | (-0.5,39) moderate | 1.9017 | 0.9641 | 255 | 0.0169 | 0.0140 | 54/54/45 | no | 30.16 | 31.13 | 0.9333 | 0/0 | - |
| 09-mixed-document | lualatex-lm | main | 1/1 | ok | 2.2691 | 0.9490 | (-0.5,39) moderate | 1.9241 | 0.9634 | 255 | 0.0170 | 0.0141 | 54/56/43 | no | 31.94 | 31.57 | 0.9302 | 0/0 | - |
| 09-mixed-document | lualatex-lm | pipeline | 1/1 | recovered | 1.9578 | 0.9595 | (-2.5,-46.5) moderate | 1.8301 | 0.9648 | 255 | 0.0151 | 0.0124 | 54/41/41 | no | 32.18 | 31.34 | 0.9512 | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.4853 | 0.9504 | (-0.5,38.5) moderate | 1.8886 | 0.9685 | 255 | 0.0174 | 0.0146 | 54/54/45 | no | 2.48 | 31.80 | 0.9778 | 0/0 | [p1](images/09-mixed-document/pdflatex-de1020c-export-p1-overlay.png) |
| 09-mixed-document | pdflatex | main | 1/1 | ok | 2.5141 | 0.9497 | (-0.5,38.5) moderate | 1.9110 | 0.9679 | 255 | 0.0175 | 0.0148 | 54/56/43 | no | 2.97 | 32.30 | 0.9767 | 0/0 | [p1](images/09-mixed-document/pdflatex-main-export-p1-overlay.png) |
| 09-mixed-document | pdflatex | pipeline | 1/1 | recovered | 2.1148 | 0.9609 | (0,-47) | 1.5167 | 0.9739 | 255 | 0.0152 | 0.0128 | 54/41/41 | no | 3.71 | 30.53 | 1.0000 | 0/0 | [p1](images/09-mixed-document/pdflatex-pipeline-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | de1020c | 1/1 | ok | 2.2522 | 0.9480 | (-0.5,39.5) moderate | 1.9012 | 0.9623 | 255 | 0.0167 | 0.0139 | 54/54/45 | no | 30.17 | 32.47 | 0.9333 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | main | 1/1 | ok | 2.2675 | 0.9475 | (-0.5,39.5) moderate | 1.9236 | 0.9616 | 255 | 0.0169 | 0.0140 | 54/56/43 | no | 31.95 | 32.96 | 0.9302 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-main-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | pipeline | 1/1 | recovered | 1.9585 | 0.9580 | (-2.5,-46) moderate | 1.8344 | 0.9644 | 255 | 0.0149 | 0.0123 | 54/41/41 | no | 32.19 | 30.33 | 0.9512 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.4838 | 0.9503 | (-0.5,38.5) | 1.8537 | 0.9690 | 255 | 0.0174 | 0.0146 | 54/54/45 | no | 2.45 | 31.76 | 0.9778 | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | ok | 2.5135 | 0.9496 | (-0.5,38.5) | 1.8761 | 0.9684 | 255 | 0.0176 | 0.0148 | 54/56/43 | no | 2.94 | 32.26 | 0.9767 | 0/0 | - |
| 09-mixed-document | xelatex | pipeline | 1/1 | recovered | 2.1114 | 0.9610 | (0,-47) | 1.4772 | 0.9743 | 255 | 0.0152 | 0.0127 | 54/41/41 | no | 3.73 | 30.57 | 1.0000 | 0/0 | - |
| 09-mixed-document | xelatex-lm | de1020c | 1/1 | ok | 2.2539 | 0.9494 | (-0.5,39) moderate | 1.9016 | 0.9641 | 255 | 0.0169 | 0.0140 | 54/54/45 | no | 30.16 | 32.06 | 0.9333 | 0/0 | - |
| 09-mixed-document | xelatex-lm | main | 1/1 | ok | 2.2692 | 0.9490 | (-0.5,39) moderate | 1.9240 | 0.9634 | 255 | 0.0170 | 0.0141 | 54/56/43 | no | 31.94 | 32.55 | 0.9302 | 0/0 | - |
| 09-mixed-document | xelatex-lm | pipeline | 1/1 | recovered | 1.9580 | 0.9595 | (-2.5,-46.5) moderate | 1.8303 | 0.9648 | 255 | 0.0151 | 0.0124 | 54/41/41 | no | 32.18 | 30.73 | 0.9512 | 0/0 | - |
| 10-unicode-paragraph | lualatex | de1020c | 1/1 | ok | 3.3153 | 0.9490 | (0,0) | 3.3153 | 0.9490 | 255 | 0.0261 | 0.0212 | 8/81/0 | no | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | main | 1/1 | ok | 3.3153 | 0.9490 | (0,0) | 3.3153 | 0.9490 | 255 | 0.0261 | 0.0212 | 8/81/0 | no | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | pipeline | 1/1 | recovered | 3.4025 | 0.9475 | (0,0) | 3.4025 | 0.9475 | 255 | 0.0264 | 0.0214 | 8/81/0 | no | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex-lm | de1020c | 1/1 | ok | 3.2356 | 0.9447 | (0,0) | 3.2356 | 0.9447 | 255 | 0.0259 | 0.0211 | 82/81/79 | no | 64.10 | 1.33 | 0.8987 | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | main | 1/1 | ok | 3.2356 | 0.9447 | (0,0) | 3.2356 | 0.9447 | 255 | 0.0259 | 0.0211 | 82/81/79 | no | 64.10 | 1.33 | 0.8987 | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 3.2141 | 0.9449 | (-1.5,0) weak | 3.2048 | 0.9452 | 255 | 0.0259 | 0.0209 | 82/81/81 | no | 84.89 | 1.79 | 0.8765 | 0/0 | - |
| 10-unicode-paragraph | pdflatex | de1020c | 1/1 | ok | 3.3620 | 0.9486 | (0,0) | 3.3620 | 0.9486 | 255 | 0.0262 | 0.0213 | 85/81/76 | no | 75.04 | 1.87 | 0.9211 | 3/1 | [p1](images/10-unicode-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex | main | 1/1 | ok | 3.3620 | 0.9486 | (0,0) | 3.3620 | 0.9486 | 255 | 0.0262 | 0.0213 | 85/81/76 | no | 75.04 | 1.87 | 0.9211 | 3/1 | [p1](images/10-unicode-paragraph/pdflatex-main-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex | pipeline | 1/1 | recovered | 3.3417 | 0.9486 | (0,0) | 3.3417 | 0.9486 | 255 | 0.0261 | 0.0211 | 85/81/77 | no | 56.02 | 1.35 | 0.9351 | 3/0 | [p1](images/10-unicode-paragraph/pdflatex-pipeline-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 3.2510 | 0.9445 | (0,0) | 3.2510 | 0.9445 | 255 | 0.0260 | 0.0212 | 82/81/80 | no | 65.00 | 1.32 | 0.9000 | 0/1 | [p1](images/10-unicode-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | main | 1/1 | ok | 3.2510 | 0.9445 | (0,0) | 3.2510 | 0.9445 | 255 | 0.0260 | 0.0212 | 82/81/80 | no | 65.00 | 1.32 | 0.9000 | 0/1 | [p1](images/10-unicode-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 3.2291 | 0.9446 | (-1.5,0) weak | 3.2194 | 0.9449 | 255 | 0.0260 | 0.0210 | 82/81/81 | no | 85.30 | 1.79 | 0.8765 | 0/0 | [p1](images/10-unicode-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 10-unicode-paragraph | xelatex | de1020c | 1/1 | ok | 3.3629 | 0.9483 | (0,0) | 3.3629 | 0.9483 | 255 | 0.0262 | 0.0213 | 84/81/78 | no | 78.17 | 2.01 | 0.9231 | 3/1 | - |
| 10-unicode-paragraph | xelatex | main | 1/1 | ok | 3.3629 | 0.9483 | (0,0) | 3.3629 | 0.9483 | 255 | 0.0262 | 0.0213 | 84/81/78 | no | 78.17 | 2.01 | 0.9231 | 3/1 | - |
| 10-unicode-paragraph | xelatex | pipeline | 1/1 | recovered | 3.3738 | 0.9479 | (0,0) | 3.3738 | 0.9479 | 255 | 0.0262 | 0.0212 | 84/81/79 | no | 60.00 | 1.51 | 0.9241 | 3/0 | - |
| 10-unicode-paragraph | xelatex-lm | de1020c | 1/1 | ok | 3.2358 | 0.9447 | (0,0) | 3.2358 | 0.9447 | 255 | 0.0259 | 0.0211 | 82/81/79 | no | 64.10 | 1.45 | 0.8987 | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | main | 1/1 | ok | 3.2358 | 0.9447 | (0,0) | 3.2358 | 0.9447 | 255 | 0.0259 | 0.0211 | 82/81/79 | no | 64.10 | 1.45 | 0.8987 | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 3.2139 | 0.9449 | (-1.5,0) weak | 3.2049 | 0.9452 | 255 | 0.0259 | 0.0209 | 82/81/81 | no | 84.89 | 1.96 | 0.8765 | 0/0 | - |
| 11-nested-lists | lualatex | de1020c | 1/1 | recovered | 1.4653 | 0.9696 | (18,-49) weak | 1.4605 | 0.9709 | 255 | 0.0104 | 0.0085 | 29/24/24 | no | 123.00 | 61.96 | 0.9167 | 0/0 | - |
| 11-nested-lists | lualatex | main | 1/1 | ok | 1.4651 | 0.9700 | (-21,-8) moderate | 1.2190 | 0.9785 | 255 | 0.0105 | 0.0087 | 29/29/29 | yes | 22.58 | 9.35 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex | pipeline | 1/1 | recovered | 1.5034 | 0.9691 | (26.5,-49) weak | 1.4442 | 0.9706 | 255 | 0.0105 | 0.0088 | 29/24/24 | no | 134.39 | 61.98 | 0.9167 | 0/0 | - |
| 11-nested-lists | lualatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (-2,-49) weak | 1.3067 | 0.9711 | 255 | 0.0100 | 0.0083 | 29/24/24 | no | 121.01 | 62.61 | 0.9167 | 0/0 | - |
| 11-nested-lists | lualatex-lm | main | 1/1 | ok | 1.3731 | 0.9689 | (-29,-8) moderate | 1.2485 | 0.9756 | 255 | 0.0101 | 0.0085 | 29/29/29 | yes | 24.90 | 10.02 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex-lm | pipeline | 1/1 | recovered | 1.3526 | 0.9690 | (54,-49) weak | 1.3121 | 0.9705 | 255 | 0.0099 | 0.0083 | 29/24/24 | no | 132.04 | 62.65 | 0.9167 | 0/0 | - |
| 11-nested-lists | pdflatex | de1020c | 1/1 | recovered | 1.4657 | 0.9698 | (19,-49) weak | 1.4558 | 0.9711 | 255 | 0.0104 | 0.0085 | 29/24/24 | no | 123.27 | 61.96 | 0.9167 | 0/0 | [p1](images/11-nested-lists/pdflatex-de1020c-export-p1-overlay.png) |
| 11-nested-lists | pdflatex | main | 1/1 | ok | 1.4717 | 0.9702 | (-20,-8) moderate | 1.1988 | 0.9789 | 255 | 0.0105 | 0.0086 | 29/29/29 | yes | 21.91 | 9.35 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-main-export-p1-overlay.png) |
| 11-nested-lists | pdflatex | pipeline | 1/1 | recovered | 1.4978 | 0.9693 | (53,-49) weak | 1.4609 | 0.9710 | 255 | 0.0104 | 0.0086 | 29/24/24 | no | 134.66 | 61.98 | 0.9167 | 0/0 | [p1](images/11-nested-lists/pdflatex-pipeline-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | de1020c | 1/1 | recovered | 1.3674 | 0.9687 | (23.5,-49) weak | 1.3216 | 0.9706 | 255 | 0.0101 | 0.0083 | 29/24/24 | no | 121.22 | 62.61 | 0.9167 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | main | 1/1 | ok | 1.3674 | 0.9690 | (-29,-8) moderate | 1.2361 | 0.9758 | 255 | 0.0102 | 0.0085 | 29/29/29 | yes | 24.28 | 10.02 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-main-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | pipeline | 1/1 | recovered | 1.3521 | 0.9690 | (52,-49) weak | 1.3046 | 0.9706 | 255 | 0.0099 | 0.0083 | 29/24/24 | no | 132.25 | 62.65 | 0.9167 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 11-nested-lists | xelatex | de1020c | 1/1 | recovered | 1.4648 | 0.9696 | (18,-49) weak | 1.4600 | 0.9709 | 255 | 0.0104 | 0.0085 | 29/24/24 | no | 123.00 | 61.96 | 0.9167 | 0/0 | - |
| 11-nested-lists | xelatex | main | 1/1 | ok | 1.4648 | 0.9699 | (-21,-8) moderate | 1.2177 | 0.9785 | 255 | 0.0105 | 0.0087 | 29/29/29 | yes | 22.57 | 9.35 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex | pipeline | 1/1 | recovered | 1.5034 | 0.9690 | (52,-49) weak | 1.4569 | 0.9706 | 255 | 0.0105 | 0.0088 | 29/24/24 | no | 134.39 | 61.98 | 0.9167 | 0/0 | - |
| 11-nested-lists | xelatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (-2,-49) weak | 1.3068 | 0.9711 | 255 | 0.0100 | 0.0083 | 29/24/24 | no | 121.01 | 61.76 | 0.9167 | 0/0 | - |
| 11-nested-lists | xelatex-lm | main | 1/1 | ok | 1.3730 | 0.9689 | (-29,-8) moderate | 1.2487 | 0.9756 | 255 | 0.0101 | 0.0085 | 29/29/29 | yes | 24.90 | 9.15 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex-lm | pipeline | 1/1 | recovered | 1.3526 | 0.9690 | (54,-49) weak | 1.3120 | 0.9705 | 255 | 0.0099 | 0.0083 | 29/24/24 | no | 132.04 | 61.79 | 0.9167 | 0/0 | - |
| 12-justified-paragraphs | lualatex | de1020c | 1/1 | ok | 15.4038 | 0.7396 | (-3,0) weak | 15.3872 | 0.7397 | 255 | 0.1111 | 0.0927 | 360/360/360 | yes | 99.24 | 32.55 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex | main | 1/1 | ok | 15.4038 | 0.7396 | (-3,0) weak | 15.3872 | 0.7397 | 255 | 0.1111 | 0.0927 | 360/360/360 | yes | 99.24 | 32.55 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex | pipeline | 1/1 | recovered | 13.1323 | 0.7893 | (-0.5,14.5) weak | 13.0045 | 0.7920 | 255 | 0.0993 | 0.0813 | 360/360/360 | yes | 109.81 | 24.00 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | de1020c | 1/1 | ok | 13.7995 | 0.7503 | (-8.5,2.5) weak | 13.7712 | 0.7503 | 255 | 0.1052 | 0.0873 | 360/360/360 | yes | 83.76 | 8.29 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | main | 1/1 | ok | 13.7995 | 0.7503 | (-8.5,2.5) weak | 13.7712 | 0.7503 | 255 | 0.1052 | 0.0873 | 360/360/360 | yes | 83.76 | 8.29 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | pipeline | 1/1 | recovered | 11.6358 | 0.8076 | (1,0) weak | 11.5477 | 0.8076 | 255 | 0.0931 | 0.0759 | 360/360/360 | yes | 76.99 | 1.80 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | pdflatex | de1020c | 1/1 | ok | 15.3708 | 0.7415 | (-3,0) weak | 15.3659 | 0.7414 | 255 | 0.1108 | 0.0925 | 360/360/360 | yes | 99.31 | 32.55 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-de1020c-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex | main | 1/1 | ok | 15.3708 | 0.7415 | (-3,0) weak | 15.3659 | 0.7414 | 255 | 0.1108 | 0.0925 | 360/360/360 | yes | 99.31 | 32.55 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-main-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex | pipeline | 1/1 | recovered | 13.0472 | 0.7918 | (-0.5,14.5) weak | 12.8874 | 0.7948 | 255 | 0.0987 | 0.0809 | 360/360/360 | yes | 109.87 | 24.00 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-pipeline-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | de1020c | 1/1 | ok | 13.7992 | 0.7503 | (-8.5,2.5) weak | 13.7701 | 0.7503 | 255 | 0.1051 | 0.0873 | 360/360/360 | yes | 83.76 | 8.99 | 0.8889 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | main | 1/1 | ok | 13.7992 | 0.7503 | (-8.5,2.5) weak | 13.7701 | 0.7503 | 255 | 0.1051 | 0.0873 | 360/360/360 | yes | 83.76 | 8.99 | 0.8889 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-main-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | pipeline | 1/1 | recovered | 11.6351 | 0.8076 | (1,0) weak | 11.5402 | 0.8076 | 255 | 0.0930 | 0.0759 | 360/360/360 | yes | 77.00 | 1.98 | 0.8889 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 12-justified-paragraphs | xelatex | de1020c | 1/1 | ok | 15.3809 | 0.7404 | (0,0) | 15.3809 | 0.7404 | 255 | 0.1111 | 0.0928 | 360/360/360 | yes | 106.70 | 32.71 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex | main | 1/1 | ok | 15.3809 | 0.7404 | (0,0) | 15.3809 | 0.7404 | 255 | 0.1111 | 0.0928 | 360/360/360 | yes | 106.70 | 32.71 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex | pipeline | 1/1 | recovered | 13.1381 | 0.7892 | (-0.5,14.5) weak | 13.0398 | 0.7918 | 255 | 0.0995 | 0.0817 | 360/360/360 | yes | 117.21 | 24.16 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | de1020c | 1/1 | ok | 13.7994 | 0.7503 | (-8.5,2.5) weak | 13.7708 | 0.7503 | 255 | 0.1051 | 0.0873 | 360/360/360 | yes | 83.76 | 8.99 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | main | 1/1 | ok | 13.7994 | 0.7503 | (-8.5,2.5) weak | 13.7708 | 0.7503 | 255 | 0.1051 | 0.0873 | 360/360/360 | yes | 83.76 | 8.99 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | pipeline | 1/1 | recovered | 11.6355 | 0.8076 | (1,0) weak | 11.5475 | 0.8076 | 255 | 0.0930 | 0.0759 | 360/360/360 | yes | 76.99 | 1.98 | 0.8889 | 0/0 | - |
| 13-math-display-rich | lualatex | de1020c | 1/1 | recovered | 0.4724 | 0.9903 | (0,3.5) moderate | 0.4325 | 0.9914 | 255 | 0.0040 | 0.0031 | 19/17/7 | no | 1.15 | 5.89 | 1.0000 | 2/2 | - |
| 13-math-display-rich | lualatex | main | 1/1 | recovered | 0.4851 | 0.9899 | (0,3.5) moderate | 0.4453 | 0.9911 | 255 | 0.0041 | 0.0032 | 19/18/7 | no | 1.15 | 5.89 | 1.0000 | 2/2 | - |
| 13-math-display-rich | lualatex | pipeline | 1/1 | recovered | 0.4416 | 0.9896 | (0,0) | 0.4416 | 0.9896 | 255 | 0.0034 | 0.0027 | 19/6/6 | no | 3.17 | 27.24 | 1.0000 | 2/0 | - |
| 13-math-display-rich | lualatex-lm | de1020c | 1/1 | recovered | 0.5515 | 0.9887 | (-2,4) weak | 0.5477 | 0.9892 | 255 | 0.0043 | 0.0035 | 19/17/7 | no | 3.38 | 5.45 | 1.0000 | 4/2 | - |
| 13-math-display-rich | lualatex-lm | main | 1/1 | recovered | 0.5643 | 0.9884 | (-2,4) weak | 0.5604 | 0.9888 | 255 | 0.0044 | 0.0036 | 19/18/7 | no | 3.38 | 5.45 | 1.0000 | 4/2 | - |
| 13-math-display-rich | lualatex-lm | pipeline | 1/1 | recovered | 0.4463 | 0.9890 | (-0.5,-36) weak | 0.4292 | 0.9899 | 255 | 0.0034 | 0.0028 | 19/6/6 | no | 1.42 | 27.69 | 1.0000 | 4/0 | - |
| 13-math-display-rich | pdflatex | de1020c | 1/1 | recovered | 0.5010 | 0.9899 | (1,3.5) weak | 0.4808 | 0.9907 | 255 | 0.0041 | 0.0033 | 19/17/7 | no | 1.58 | 6.00 | 1.0000 | 2/2 | [p1](images/13-math-display-rich/pdflatex-de1020c-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex | main | 1/1 | recovered | 0.5138 | 0.9896 | (1,3.5) weak | 0.4936 | 0.9903 | 255 | 0.0042 | 0.0033 | 19/18/7 | no | 1.58 | 6.00 | 1.0000 | 2/2 | [p1](images/13-math-display-rich/pdflatex-main-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex | pipeline | 1/1 | recovered | 0.4400 | 0.9896 | (6.5,-36) weak | 0.4188 | 0.9907 | 255 | 0.0034 | 0.0027 | 19/6/6 | no | 3.67 | 27.12 | 1.0000 | 2/0 | [p1](images/13-math-display-rich/pdflatex-pipeline-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | de1020c | 1/1 | recovered | 0.5530 | 0.9888 | (-2,4) weak | 0.5406 | 0.9893 | 255 | 0.0043 | 0.0035 | 19/17/7 | no | 3.38 | 6.19 | 1.0000 | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | main | 1/1 | recovered | 0.5658 | 0.9884 | (-2,4) weak | 0.5534 | 0.9890 | 255 | 0.0044 | 0.0036 | 19/18/7 | no | 3.38 | 6.19 | 1.0000 | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-main-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | pipeline | 1/1 | recovered | 0.4462 | 0.9890 | (-0.5,-35.5) weak | 0.4325 | 0.9897 | 255 | 0.0034 | 0.0028 | 19/6/6 | no | 1.41 | 26.77 | 1.0000 | 4/0 | [p1](images/13-math-display-rich/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 13-math-display-rich | xelatex | de1020c | 1/1 | recovered | 0.4941 | 0.9899 | (0,0) | 0.4941 | 0.9899 | 255 | 0.0041 | 0.0032 | 21/17/8 | no | 1.56 | 2.84 | 1.0000 | 2/2 | - |
| 13-math-display-rich | xelatex | main | 1/1 | recovered | 0.5069 | 0.9896 | (0,0) | 0.5069 | 0.9896 | 255 | 0.0042 | 0.0033 | 21/18/8 | no | 1.56 | 2.84 | 1.0000 | 2/2 | - |
| 13-math-display-rich | xelatex | pipeline | 1/1 | recovered | 0.4415 | 0.9895 | (6.5,-36) moderate | 0.4138 | 0.9907 | 255 | 0.0034 | 0.0028 | 21/6/6 | no | 3.58 | 23.89 | 1.0000 | 2/0 | - |
| 13-math-display-rich | xelatex-lm | de1020c | 1/1 | recovered | 0.5517 | 0.9887 | (-2,4) weak | 0.5476 | 0.9892 | 255 | 0.0043 | 0.0035 | 21/17/8 | no | 3.20 | 3.05 | 1.0000 | 4/2 | - |
| 13-math-display-rich | xelatex-lm | main | 1/1 | recovered | 0.5645 | 0.9884 | (-2,4) weak | 0.5604 | 0.9888 | 255 | 0.0044 | 0.0036 | 21/18/8 | no | 3.20 | 3.05 | 1.0000 | 4/2 | - |
| 13-math-display-rich | xelatex-lm | pipeline | 1/1 | recovered | 0.4465 | 0.9890 | (-0.5,-36) weak | 0.4293 | 0.9899 | 255 | 0.0034 | 0.0028 | 21/6/6 | no | 1.42 | 23.78 | 1.0000 | 4/0 | - |
| 14-math-inline-dense | lualatex | de1020c | 1/1 | recovered | 1.3179 | 0.9705 | (-34.5,13.5) | 0.9604 | 0.9783 | 255 | 0.0099 | 0.0082 | 62/59/25 | no | 52.74 | 10.98 | 0.9600 | 2/1 | - |
| 14-math-inline-dense | lualatex | main | 1/1 | recovered | 1.3179 | 0.9705 | (-34.5,13.5) | 0.9604 | 0.9783 | 255 | 0.0099 | 0.0082 | 62/59/25 | no | 52.74 | 10.98 | 0.9600 | 2/1 | - |
| 14-math-inline-dense | lualatex | pipeline | 1/1 | recovered | 0.8302 | 0.9818 | (0.5,-14.5) weak | 0.7929 | 0.9821 | 255 | 0.0066 | 0.0053 | 62/32/20 | no | 38.35 | 11.32 | 0.9500 | 2/0 | - |
| 14-math-inline-dense | lualatex-lm | de1020c | 1/1 | recovered | 1.2473 | 0.9702 | (5,13.5) moderate | 1.1675 | 0.9735 | 255 | 0.0097 | 0.0079 | 62/59/23 | no | 60.36 | 10.71 | 0.9565 | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | main | 1/1 | recovered | 1.2473 | 0.9702 | (5,13.5) moderate | 1.1675 | 0.9735 | 255 | 0.0097 | 0.0079 | 62/59/23 | no | 60.36 | 10.71 | 0.9565 | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | pipeline | 1/1 | recovered | 0.8055 | 0.9808 | (-1,-14.5) moderate | 0.7447 | 0.9819 | 255 | 0.0065 | 0.0053 | 62/32/19 | no | 30.77 | 11.90 | 0.9474 | 2/0 | - |
| 14-math-inline-dense | pdflatex | de1020c | 1/1 | recovered | 1.3129 | 0.9708 | (-34,13.5) | 0.9608 | 0.9783 | 255 | 0.0099 | 0.0082 | 53/59/18 | no | 56.56 | 13.98 | 0.8889 | 2/1 | [p1](images/14-math-inline-dense/pdflatex-de1020c-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex | main | 1/1 | recovered | 1.3129 | 0.9708 | (-34,13.5) | 0.9608 | 0.9783 | 255 | 0.0099 | 0.0082 | 53/59/18 | no | 56.56 | 13.98 | 0.8889 | 2/1 | [p1](images/14-math-inline-dense/pdflatex-main-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex | pipeline | 1/1 | recovered | 0.8202 | 0.9822 | (1,-14.5) weak | 0.7901 | 0.9825 | 255 | 0.0065 | 0.0052 | 53/32/16 | no | 36.69 | 11.91 | 0.9375 | 2/0 | [p1](images/14-math-inline-dense/pdflatex-pipeline-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | de1020c | 1/1 | recovered | 1.2469 | 0.9702 | (5,13.5) moderate | 1.1673 | 0.9735 | 255 | 0.0097 | 0.0079 | 55/59/19 | no | 64.42 | 13.17 | 0.8947 | 2/1 | [p1](images/14-math-inline-dense/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | main | 1/1 | recovered | 1.2469 | 0.9702 | (5,13.5) moderate | 1.1673 | 0.9735 | 255 | 0.0097 | 0.0079 | 55/59/19 | no | 64.42 | 13.17 | 0.8947 | 2/1 | [p1](images/14-math-inline-dense/pdflatex-lm-main-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | pipeline | 1/1 | recovered | 0.8050 | 0.9808 | (-0.5,-14.5) moderate | 0.7299 | 0.9822 | 255 | 0.0065 | 0.0053 | 55/32/16 | no | 24.10 | 12.82 | 0.9375 | 2/0 | [p1](images/14-math-inline-dense/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 14-math-inline-dense | xelatex | de1020c | 1/1 | recovered | 1.3179 | 0.9705 | (-34.5,13.5) | 0.9575 | 0.9783 | 255 | 0.0099 | 0.0082 | 63/59/25 | no | 52.75 | 12.18 | 0.9600 | 2/1 | - |
| 14-math-inline-dense | xelatex | main | 1/1 | recovered | 1.3179 | 0.9705 | (-34.5,13.5) | 0.9575 | 0.9783 | 255 | 0.0099 | 0.0082 | 63/59/25 | no | 52.75 | 12.18 | 0.9600 | 2/1 | - |
| 14-math-inline-dense | xelatex | pipeline | 1/1 | recovered | 0.8301 | 0.9819 | (0.5,-14.5) weak | 0.7928 | 0.9821 | 255 | 0.0066 | 0.0053 | 63/32/20 | no | 38.34 | 12.18 | 0.9500 | 2/0 | - |
| 14-math-inline-dense | xelatex-lm | de1020c | 1/1 | recovered | 1.2473 | 0.9702 | (5,13.5) moderate | 1.1675 | 0.9735 | 255 | 0.0097 | 0.0079 | 63/59/23 | no | 60.36 | 11.58 | 0.9565 | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | main | 1/1 | recovered | 1.2473 | 0.9702 | (5,13.5) moderate | 1.1675 | 0.9735 | 255 | 0.0097 | 0.0079 | 63/59/23 | no | 60.36 | 11.58 | 0.9565 | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | pipeline | 1/1 | recovered | 0.8056 | 0.9808 | (-1,-14.5) moderate | 0.7447 | 0.9819 | 255 | 0.0065 | 0.0053 | 63/32/19 | no | 30.77 | 12.75 | 0.9474 | 2/0 | - |
| 15-three-page-sections | lualatex | de1020c | 3/3 | recovered | 26.4239 | 0.5635 | (0.5,20) moderate; (0.5,19) weak; (0.5,-24.5) moderate | 25.2158 | 0.5876 | 255 | 0.1894 | 0.1584 | 1806/1806/1806 | yes | 154.50 | 39.85 | 0.8984 | 0/0 | - |
| 15-three-page-sections | lualatex | main | 3/3 | recovered | 26.4376 | 0.5632 | (0.5,20) moderate; (0.5,19) weak; (0.5,-24.5) moderate | 25.2273 | 0.5874 | 255 | 0.1895 | 0.1585 | 1806/1809/1806 | no | 154.51 | 39.85 | 0.8978 | 0/0 | - |
| 15-three-page-sections | lualatex | pipeline | 3/3 | recovered | 23.8429 | 0.6073 | (0.5,14) weak; (0.5,-26.5) moderate; (0.5,-55.5) moderate | 21.5340 | 0.6637 | 255 | 0.1765 | 0.1463 | 1806/1806/1806 | yes | 165.50 | 51.60 | 0.8991 | 0/0 | - |
| 15-three-page-sections | lualatex-lm | de1020c | 3/3 | recovered | 24.2612 | 0.5493 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0398 | 0.5821 | 255 | 0.1832 | 0.1520 | 1806/1806/1806 | yes | 122.99 | 44.85 | 0.8926 | 3/0 | - |
| 15-three-page-sections | lualatex-lm | main | 3/3 | recovered | 24.2718 | 0.5491 | (0.5,22.5) moderate; (3.5,-24) weak; (0.5,-24) weak | 23.0722 | 0.5811 | 255 | 0.1833 | 0.1520 | 1806/1809/1806 | no | 123.00 | 44.85 | 0.8920 | 3/0 | - |
| 15-three-page-sections | lualatex-lm | pipeline | 3/3 | recovered | 21.9333 | 0.6000 | (0.5,0) weak; (0.5,-26.5) moderate; (0.5,-26.5) moderate | 20.2282 | 0.6462 | 255 | 0.1706 | 0.1400 | 1806/1806/1806 | yes | 109.20 | 63.08 | 0.8929 | 3/0 | - |
| 15-three-page-sections | pdflatex | de1020c | 3/3 | recovered | 26.6994 | 0.5541 | (0.5,36.5) moderate; (0,-24.5) moderate; (0,-24.5) weak | 25.0916 | 0.5919 | 255 | 0.1902 | 0.1599 | 1806/1806/1806 | yes | 136.77 | 43.12 | 0.8914 | 0/0 | [p1](images/15-three-page-sections/pdflatex-de1020c-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex | main | 3/3 | recovered | 26.7135 | 0.5538 | (0.5,36.5) moderate; (0,-24.5) moderate; (0,-24.5) weak | 25.1031 | 0.5918 | 255 | 0.1903 | 0.1599 | 1806/1809/1806 | no | 136.79 | 43.12 | 0.8908 | 0/0 | [p1](images/15-three-page-sections/pdflatex-main-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex | pipeline | 3/3 | recovered | 23.7585 | 0.6054 | (0,-0.5) moderate; (0,-55.5) moderate; (0,2) moderate | 21.5955 | 0.6585 | 255 | 0.1758 | 0.1462 | 1806/1806/1806 | yes | 152.25 | 59.90 | 0.8917 | 0/0 | [p1](images/15-three-page-sections/pdflatex-pipeline-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | de1020c | 3/3 | recovered | 24.2615 | 0.5493 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0362 | 0.5822 | 255 | 0.1833 | 0.1521 | 1806/1806/1806 | yes | 123.00 | 44.54 | 0.8926 | 3/0 | [p1](images/15-three-page-sections/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | main | 3/3 | recovered | 24.2722 | 0.5491 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0478 | 0.5820 | 255 | 0.1833 | 0.1521 | 1806/1809/1806 | no | 123.00 | 44.54 | 0.8920 | 3/0 | [p1](images/15-three-page-sections/pdflatex-lm-main-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | pipeline | 3/3 | recovered | 21.9366 | 0.5999 | (0.5,0) weak; (0.5,-26.5) moderate; (0.5,-26.5) moderate | 20.2319 | 0.6460 | 255 | 0.1706 | 0.1401 | 1806/1806/1806 | yes | 109.21 | 62.47 | 0.8929 | 3/0 | [p1](images/15-three-page-sections/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 15-three-page-sections | xelatex | de1020c | 3/3 | recovered | 26.4202 | 0.5637 | (-3,20) moderate; (-3,19) weak; (-3,-24.5) moderate | 25.1892 | 0.5879 | 255 | 0.1893 | 0.1587 | 1806/1806/1806 | yes | 154.49 | 39.85 | 0.8984 | 0/0 | - |
| 15-three-page-sections | xelatex | main | 3/3 | recovered | 26.4339 | 0.5634 | (-3,20) moderate; (-3,19) weak; (-3,-24.5) moderate | 25.2007 | 0.5877 | 255 | 0.1894 | 0.1588 | 1806/1809/1806 | no | 154.51 | 39.85 | 0.8978 | 0/0 | - |
| 15-three-page-sections | xelatex | pipeline | 3/3 | recovered | 23.8245 | 0.6079 | (0,14) weak; (0,-26.5) moderate; (0,-55.5) moderate | 21.5210 | 0.6641 | 255 | 0.1764 | 0.1466 | 1806/1806/1806 | yes | 165.49 | 51.60 | 0.8991 | 0/0 | - |
| 15-three-page-sections | xelatex-lm | de1020c | 3/3 | recovered | 24.2611 | 0.5493 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0397 | 0.5821 | 255 | 0.1832 | 0.1519 | 1806/1806/1806 | yes | 122.99 | 44.55 | 0.8926 | 3/0 | - |
| 15-three-page-sections | xelatex-lm | main | 3/3 | recovered | 24.2716 | 0.5491 | (0.5,22.5) moderate; (3.5,-24) weak; (0.5,-24) weak | 23.0720 | 0.5811 | 255 | 0.1833 | 0.1520 | 1806/1809/1806 | no | 123.00 | 44.55 | 0.8920 | 3/0 | - |
| 15-three-page-sections | xelatex-lm | pipeline | 3/3 | recovered | 21.9328 | 0.5999 | (0.5,0) weak; (0.5,-26.5) moderate; (0.5,-26.5) moderate | 20.2280 | 0.6462 | 255 | 0.1706 | 0.1400 | 1806/1806/1806 | yes | 109.20 | 62.48 | 0.8929 | 3/0 | - |
| 16-heading-page-break | lualatex | de1020c | 2/2 | ok | 20.5835 | 0.6612 | (0,0); (0,20.5) moderate | 20.0992 | 0.6687 | 255 | 0.1484 | 0.1240 | 962/962/962 | yes | 161.32 | 45.97 | 0.8950 | 0/0 | - |
| 16-heading-page-break | lualatex | main | 2/2 | ok | 20.5883 | 0.6611 | (0,0); (0,20.5) moderate | 20.1037 | 0.6686 | 255 | 0.1484 | 0.1240 | 962/963/962 | no | 161.35 | 45.97 | 0.8939 | 0/0 | - |
| 16-heading-page-break | lualatex | pipeline | 2/2 | recovered | 17.5863 | 0.7205 | (0,0); (0,57.5) weak | 17.5200 | 0.7210 | 255 | 0.1335 | 0.1092 | 962/962/962 | yes | 172.54 | 35.66 | 0.8940 | 0/0 | - |
| 16-heading-page-break | lualatex-lm | de1020c | 2/2 | ok | 19.0181 | 0.6506 | (-1,0) weak; (-1,20) moderate | 18.0483 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/962/962 | yes | 137.29 | 13.93 | 0.8888 | 1/0 | - |
| 16-heading-page-break | lualatex-lm | main | 2/2 | ok | 19.0252 | 0.6506 | (-1,0) weak; (-1,20) moderate | 18.0517 | 0.6754 | 255 | 0.1440 | 0.1195 | 962/963/962 | no | 137.30 | 13.93 | 0.8877 | 1/0 | - |
| 16-heading-page-break | lualatex-lm | pipeline | 2/2 | recovered | 15.8892 | 0.7331 | (-2.5,0) weak; (0.5,14.5) weak | 15.6897 | 0.7370 | 255 | 0.1266 | 0.1032 | 962/962/962 | yes | 127.96 | 5.25 | 0.8878 | 1/0 | - |
| 16-heading-page-break | pdflatex | de1020c | 2/2 | ok | 20.5977 | 0.6620 | (0,-14.5) weak; (0,20.5) moderate | 20.0449 | 0.6704 | 255 | 0.1481 | 0.1240 | 962/962/962 | yes | 141.32 | 45.21 | 0.8952 | 0/0 | [p1](images/16-heading-page-break/pdflatex-de1020c-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex | main | 2/2 | ok | 20.6026 | 0.6619 | (0,-14.5) weak; (0,20.5) moderate | 20.0498 | 0.6703 | 255 | 0.1481 | 0.1240 | 962/963/962 | no | 141.35 | 45.21 | 0.8940 | 0/0 | [p1](images/16-heading-page-break/pdflatex-main-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex | pipeline | 2/2 | recovered | 17.3554 | 0.7253 | (0,0); (0,57.5) weak | 17.3276 | 0.7253 | 255 | 0.1322 | 0.1082 | 962/962/962 | yes | 156.49 | 34.89 | 0.8941 | 0/0 | [p1](images/16-heading-page-break/pdflatex-pipeline-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | de1020c | 2/2 | ok | 19.0175 | 0.6505 | (-1,0) weak; (-1,20) moderate | 18.0489 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/962/962 | yes | 137.30 | 14.56 | 0.8888 | 1/0 | [p1](images/16-heading-page-break/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | main | 2/2 | ok | 19.0245 | 0.6505 | (-1,0) weak; (-1,20) moderate | 18.0522 | 0.6754 | 255 | 0.1440 | 0.1196 | 962/963/962 | no | 137.31 | 14.56 | 0.8877 | 1/0 | [p1](images/16-heading-page-break/pdflatex-lm-main-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | pipeline | 2/2 | recovered | 15.8862 | 0.7332 | (-2.5,0) weak; (0.5,14.5) weak | 15.6820 | 0.7371 | 255 | 0.1266 | 0.1032 | 962/962/962 | yes | 127.97 | 5.50 | 0.8878 | 1/0 | [p1](images/16-heading-page-break/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 16-heading-page-break | xelatex | de1020c | 2/2 | ok | 20.6100 | 0.6610 | (-3,-14.5) weak; (0,20.5) moderate | 20.1333 | 0.6680 | 255 | 0.1485 | 0.1243 | 962/962/962 | yes | 161.33 | 45.97 | 0.8950 | 0/0 | - |
| 16-heading-page-break | xelatex | main | 2/2 | ok | 20.6148 | 0.6609 | (-3,-14.5) weak; (0,20.5) moderate | 20.1380 | 0.6679 | 255 | 0.1486 | 0.1243 | 962/963/962 | no | 161.35 | 45.97 | 0.8939 | 0/0 | - |
| 16-heading-page-break | xelatex | pipeline | 2/2 | recovered | 17.5354 | 0.7211 | (0,0); (0,57.5) weak | 17.4542 | 0.7220 | 255 | 0.1334 | 0.1093 | 962/962/962 | yes | 172.55 | 35.66 | 0.8940 | 0/0 | - |
| 16-heading-page-break | xelatex-lm | de1020c | 2/2 | ok | 19.0182 | 0.6506 | (-1,0) weak; (-1,20) moderate | 18.0480 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/962/962 | yes | 137.29 | 14.56 | 0.8888 | 1/0 | - |
| 16-heading-page-break | xelatex-lm | main | 2/2 | ok | 19.0252 | 0.6506 | (-1,0) weak; (-1,20) moderate | 18.0514 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/963/962 | no | 137.30 | 14.56 | 0.8877 | 1/0 | - |
| 16-heading-page-break | xelatex-lm | pipeline | 2/2 | recovered | 15.8885 | 0.7332 | (-2.5,0) weak; (0.5,14.5) weak | 15.6883 | 0.7370 | 255 | 0.1266 | 0.1032 | 962/962/962 | yes | 127.96 | 5.49 | 0.8878 | 1/0 | - |
| 17-apostrophes | lualatex | de1020c | 1/1 | ok | 0.8502 | 0.9875 | (-1.5,0) weak | 0.8119 | 0.9882 | 255 | 0.0068 | 0.0053 | 10/25/7 | no | 1.20 | 0.42 | 1.0000 | 0/0 | - |
| 17-apostrophes | lualatex | main | 1/1 | ok | 0.8502 | 0.9875 | (-1.5,0) weak | 0.8119 | 0.9882 | 255 | 0.0068 | 0.0053 | 10/25/7 | no | 1.20 | 0.42 | 1.0000 | 0/0 | - |
| 17-apostrophes | lualatex | pipeline | 1/1 | recovered | 0.9542 | 0.9850 | (0,0) | 0.9542 | 0.9850 | 255 | 0.0074 | 0.0059 | 10/25/9 | no | 50.61 | 0.41 | 0.8889 | 0/0 | - |
| 17-apostrophes | lualatex-lm | de1020c | 1/1 | ok | 0.8622 | 0.9848 | (0,0) | 0.8622 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.56 | 0.33 | 0.9231 | 0/0 | - |
| 17-apostrophes | lualatex-lm | main | 1/1 | ok | 0.8622 | 0.9848 | (0,0) | 0.8622 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.56 | 0.33 | 0.9231 | 0/0 | - |
| 17-apostrophes | lualatex-lm | pipeline | 1/1 | recovered | 0.8283 | 0.9858 | (-2.5,0) moderate | 0.7676 | 0.9874 | 255 | 0.0068 | 0.0054 | 25/25/25 | yes | 2.85 | 0.36 | 1.0000 | 0/0 | - |
| 17-apostrophes | pdflatex | de1020c | 1/1 | ok | 0.8063 | 0.9887 | (0,0) | 0.8063 | 0.9887 | 255 | 0.0066 | 0.0052 | 25/25/13 | no | 2.67 | 0.44 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-de1020c-export-p1-overlay.png) |
| 17-apostrophes | pdflatex | main | 1/1 | ok | 0.8063 | 0.9887 | (0,0) | 0.8063 | 0.9887 | 255 | 0.0066 | 0.0052 | 25/25/13 | no | 2.67 | 0.44 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-main-export-p1-overlay.png) |
| 17-apostrophes | pdflatex | pipeline | 1/1 | recovered | 0.9556 | 0.9853 | (9,0) weak | 0.9310 | 0.9858 | 255 | 0.0073 | 0.0059 | 25/25/25 | yes | 49.01 | 0.99 | 0.9200 | 0/0 | [p1](images/17-apostrophes/pdflatex-pipeline-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | de1020c | 1/1 | ok | 0.8624 | 0.9848 | (0,0) | 0.8624 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.55 | 0.70 | 0.9231 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | main | 1/1 | ok | 0.8624 | 0.9848 | (0,0) | 0.8624 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.55 | 0.70 | 0.9231 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-main-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | pipeline | 1/1 | recovered | 0.8285 | 0.9858 | (-2.5,0) moderate | 0.7673 | 0.9874 | 255 | 0.0068 | 0.0054 | 25/25/25 | yes | 2.84 | 0.67 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 17-apostrophes | xelatex | de1020c | 1/1 | ok | 0.7981 | 0.9883 | (0,0) | 0.7981 | 0.9883 | 255 | 0.0066 | 0.0051 | 25/25/13 | no | 2.69 | 0.44 | 1.0000 | 0/0 | - |
| 17-apostrophes | xelatex | main | 1/1 | ok | 0.7981 | 0.9883 | (0,0) | 0.7981 | 0.9883 | 255 | 0.0066 | 0.0051 | 25/25/13 | no | 2.69 | 0.44 | 1.0000 | 0/0 | - |
| 17-apostrophes | xelatex | pipeline | 1/1 | recovered | 0.9561 | 0.9849 | (6.5,0) weak | 0.9447 | 0.9853 | 255 | 0.0074 | 0.0059 | 25/25/25 | yes | 49.01 | 0.99 | 0.9200 | 0/0 | - |
| 17-apostrophes | xelatex-lm | de1020c | 1/1 | ok | 0.8620 | 0.9848 | (0,0) | 0.8620 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.56 | 0.70 | 0.9231 | 0/0 | - |
| 17-apostrophes | xelatex-lm | main | 1/1 | ok | 0.8620 | 0.9848 | (0,0) | 0.8620 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.56 | 0.70 | 0.9231 | 0/0 | - |
| 17-apostrophes | xelatex-lm | pipeline | 1/1 | recovered | 0.8283 | 0.9858 | (-2.5,0) moderate | 0.7675 | 0.9874 | 255 | 0.0068 | 0.0054 | 25/25/25 | yes | 2.85 | 0.67 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex | de1020c | 1/1 | ok | 1.3714 | 0.9790 | (0,0) | 1.3714 | 0.9790 | 255 | 0.0109 | 0.0087 | 36/36/36 | yes | 41.08 | 1.23 | 0.8889 | 0/0 | - |
| 18-ligatures | lualatex | main | 1/1 | ok | 1.3714 | 0.9790 | (0,0) | 1.3714 | 0.9790 | 255 | 0.0109 | 0.0087 | 36/36/36 | yes | 41.08 | 1.23 | 0.8889 | 0/0 | - |
| 18-ligatures | lualatex | pipeline | 1/1 | recovered | 1.1489 | 0.9840 | (0.5,0) moderate | 1.0484 | 0.9858 | 255 | 0.0098 | 0.0075 | 36/36/36 | yes | 0.76 | 0.41 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex-lm | de1020c | 1/1 | ok | 1.3134 | 0.9787 | (0,0) | 1.3134 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.13 | 0.34 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex-lm | main | 1/1 | ok | 1.3134 | 0.9787 | (0,0) | 1.3134 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.13 | 0.34 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex-lm | pipeline | 1/1 | recovered | 1.3281 | 0.9782 | (0,0) | 1.3281 | 0.9782 | 255 | 0.0108 | 0.0087 | 36/36/36 | yes | 49.76 | 1.16 | 0.8889 | 0/0 | - |
| 18-ligatures | pdflatex | de1020c | 1/1 | ok | 1.2894 | 0.9808 | (0,0) | 1.2894 | 0.9808 | 255 | 0.0105 | 0.0082 | 36/36/36 | yes | 40.70 | 1.23 | 0.8889 | 0/0 | [p1](images/18-ligatures/pdflatex-de1020c-export-p1-overlay.png) |
| 18-ligatures | pdflatex | main | 1/1 | ok | 1.2894 | 0.9808 | (0,0) | 1.2894 | 0.9808 | 255 | 0.0105 | 0.0082 | 36/36/36 | yes | 40.70 | 1.23 | 0.8889 | 0/0 | [p1](images/18-ligatures/pdflatex-main-export-p1-overlay.png) |
| 18-ligatures | pdflatex | pipeline | 1/1 | recovered | 0.7396 | 0.9907 | (0,0) | 0.7396 | 0.9907 | 255 | 0.0084 | 0.0056 | 36/36/36 | yes | 0.52 | 0.41 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-pipeline-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | de1020c | 1/1 | ok | 1.3148 | 0.9787 | (0,0) | 1.3148 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.14 | 0.69 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | main | 1/1 | ok | 1.3148 | 0.9787 | (0,0) | 1.3148 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.14 | 0.69 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-main-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | pipeline | 1/1 | recovered | 1.3286 | 0.9782 | (0,0) | 1.3286 | 0.9782 | 255 | 0.0108 | 0.0087 | 36/36/36 | yes | 49.77 | 1.40 | 0.8889 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 18-ligatures | xelatex | de1020c | 1/1 | ok | 1.3498 | 0.9797 | (0,0) | 1.3498 | 0.9797 | 255 | 0.0108 | 0.0087 | 36/36/36 | yes | 40.84 | 1.23 | 0.8889 | 0/0 | - |
| 18-ligatures | xelatex | main | 1/1 | ok | 1.3498 | 0.9797 | (0,0) | 1.3498 | 0.9797 | 255 | 0.0108 | 0.0087 | 36/36/36 | yes | 40.84 | 1.23 | 0.8889 | 0/0 | - |
| 18-ligatures | xelatex | pipeline | 1/1 | recovered | 0.8056 | 0.9896 | (0,0) | 0.8056 | 0.9896 | 255 | 0.0087 | 0.0061 | 36/36/36 | yes | 0.43 | 0.41 | 1.0000 | 0/0 | - |
| 18-ligatures | xelatex-lm | de1020c | 1/1 | ok | 1.3133 | 0.9787 | (0,0) | 1.3133 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.13 | 0.69 | 1.0000 | 0/0 | - |
| 18-ligatures | xelatex-lm | main | 1/1 | ok | 1.3133 | 0.9787 | (0,0) | 1.3133 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.13 | 0.69 | 1.0000 | 0/0 | - |
| 18-ligatures | xelatex-lm | pipeline | 1/1 | recovered | 1.3281 | 0.9782 | (0,0) | 1.3281 | 0.9782 | 255 | 0.0108 | 0.0087 | 36/36/36 | yes | 49.76 | 1.40 | 0.8889 | 0/0 | - |

## Diagnostic: preview-equivalent comparison (CoreText draw of compile_result vs reference PDF raster)

Weaker than a capture of the real preview: it re-implements the app's draw code path rather than exercising the SwiftUI Canvas. Word-box metrics are not available for this side (no PDF), so they are omitted.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| raw | SSIM₈ raw | registration Δ pt (dx,dy per page; `weak`/`moderate` = shift explains <25% of the error) | mean\|Δ\| after reg | SSIM₈ after reg | max | differing | ≥thr | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-plain-paragraph | lualatex | de1020c | 1/1 | ok | 0.1686 | 0.9978 | (0,0) | 0.1686 | 0.9978 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.1686 | 0.9978 | (0,0) | 0.1686 | 0.9978 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | pipeline | 1/1 | recovered | 0.3848 | 0.9944 | (0,0) | 0.3848 | 0.9944 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | de1020c | 1/1 | ok | 0.3642 | 0.9942 | (0,0) | 0.3642 | 0.9942 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | main | 1/1 | ok | 0.3642 | 0.9942 | (0,0) | 0.3642 | 0.9942 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 0.3738 | 0.9942 | (0,0) | 0.3738 | 0.9942 | 255 | 0.0031 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.1741 | 0.9977 | (0,0) | 0.1741 | 0.9977 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.1741 | 0.9977 | (0,0) | 0.1741 | 0.9977 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | pipeline | 1/1 | recovered | 0.3931 | 0.9945 | (0,0) | 0.3931 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 0.3639 | 0.9942 | (0,0) | 0.3639 | 0.9942 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | main | 1/1 | ok | 0.3639 | 0.9942 | (0,0) | 0.3639 | 0.9942 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 0.3736 | 0.9942 | (0,0) | 0.3736 | 0.9942 | 255 | 0.0031 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.1716 | 0.9978 | (0,0) | 0.1716 | 0.9978 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.1716 | 0.9978 | (0,0) | 0.1716 | 0.9978 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | pipeline | 1/1 | recovered | 0.3841 | 0.9944 | (0,0) | 0.3841 | 0.9944 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | de1020c | 1/1 | ok | 0.3644 | 0.9942 | (0,0) | 0.3644 | 0.9942 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | main | 1/1 | ok | 0.3644 | 0.9942 | (0,0) | 0.3644 | 0.9942 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 0.3735 | 0.9942 | (0,0) | 0.3735 | 0.9942 | 255 | 0.0031 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 8.3831 | 0.8656 | (-3,0) weak | 8.3728 | 0.8658 | 255 | 0.0613 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 8.3831 | 0.8656 | (-3,0) weak | 8.3728 | 0.8658 | 255 | 0.0613 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | pipeline | 1/1 | recovered | 7.1465 | 0.8938 | (4.5,0) weak | 7.1178 | 0.8930 | 255 | 0.0551 | 0.0447 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | de1020c | 1/1 | ok | 7.7865 | 0.8620 | (-0.5,-14.5) weak | 7.7677 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | main | 1/1 | ok | 7.7865 | 0.8620 | (-0.5,-14.5) weak | 7.7677 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 6.7841 | 0.8883 | (0.5,0) weak | 6.7069 | 0.8893 | 255 | 0.0543 | 0.0441 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 8.3720 | 0.8664 | (0,0) | 8.3720 | 0.8664 | 255 | 0.0611 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 8.3720 | 0.8664 | (0,0) | 8.3720 | 0.8664 | 255 | 0.0611 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | pipeline | 1/1 | recovered | 7.1038 | 0.8947 | (0,0) | 7.1038 | 0.8947 | 255 | 0.0549 | 0.0444 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 7.7859 | 0.8621 | (-0.5,-14.5) weak | 7.7684 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | main | 1/1 | ok | 7.7859 | 0.8621 | (-0.5,-14.5) weak | 7.7684 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 6.7849 | 0.8883 | (0.5,0) weak | 6.7045 | 0.8894 | 255 | 0.0542 | 0.0441 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 8.3703 | 0.8657 | (0,0) | 8.3703 | 0.8657 | 255 | 0.0613 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 8.3703 | 0.8657 | (0,0) | 8.3703 | 0.8657 | 255 | 0.0613 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | pipeline | 1/1 | recovered | 7.1276 | 0.8940 | (0,0) | 7.1276 | 0.8940 | 255 | 0.0552 | 0.0448 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | de1020c | 1/1 | ok | 7.7862 | 0.8620 | (-0.5,-14.5) weak | 7.7678 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | main | 1/1 | ok | 7.7862 | 0.8620 | (-0.5,-14.5) weak | 7.7678 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 6.7835 | 0.8883 | (0.5,0) weak | 6.7065 | 0.8893 | 255 | 0.0542 | 0.0441 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.3394 | 0.9793 | (0,52.5) moderate | 1.1862 | 0.9829 | 255 | 0.0085 | 0.0073 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.3597 | 0.9789 | (0.5,52.5) moderate | 1.2078 | 0.9824 | 255 | 0.0086 | 0.0074 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | pipeline | 1/1 | recovered | 0.9259 | 0.9882 | (-0.5,-1.5) weak | 0.8904 | 0.9891 | 255 | 0.0064 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | de1020c | 1/1 | ok | 1.2336 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | main | 1/1 | ok | 1.2425 | 0.9769 | (0,53.5) moderate | 1.1122 | 0.9812 | 255 | 0.0082 | 0.0070 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | pipeline | 1/1 | recovered | 0.8561 | 0.9880 | (-0.5,0) weak | 0.8189 | 0.9886 | 255 | 0.0062 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.3496 | 0.9793 | (0.5,52.5) moderate | 1.2021 | 0.9827 | 255 | 0.0085 | 0.0072 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.3719 | 0.9789 | (0.5,52.5) moderate | 1.2209 | 0.9823 | 255 | 0.0086 | 0.0074 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | pipeline | 1/1 | recovered | 0.9554 | 0.9878 | (0,-2) moderate | 0.8928 | 0.9877 | 255 | 0.0066 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | de1020c | 1/1 | ok | 1.2335 | 0.9771 | (0,54) moderate | 1.0954 | 0.9811 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | main | 1/1 | ok | 1.2427 | 0.9769 | (0.5,54) moderate | 1.1273 | 0.9806 | 255 | 0.0082 | 0.0070 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | pipeline | 1/1 | recovered | 0.8553 | 0.9881 | (0,0) | 0.8553 | 0.9881 | 255 | 0.0062 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.3370 | 0.9793 | (0,52.5) moderate | 1.1869 | 0.9829 | 255 | 0.0085 | 0.0073 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.3601 | 0.9789 | (0.5,52.5) moderate | 1.2088 | 0.9824 | 255 | 0.0086 | 0.0074 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | pipeline | 1/1 | recovered | 0.9280 | 0.9882 | (-0.5,-1.5) weak | 0.8889 | 0.9891 | 255 | 0.0065 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | de1020c | 1/1 | ok | 1.2337 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | main | 1/1 | ok | 1.2424 | 0.9769 | (0,53.5) moderate | 1.1122 | 0.9812 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | pipeline | 1/1 | recovered | 0.8562 | 0.9880 | (-0.5,0) weak | 0.8189 | 0.9886 | 255 | 0.0062 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.3808 | 0.9945 | (0,0) | 0.3808 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.3808 | 0.9945 | (0,0) | 0.3808 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | pipeline | 1/1 | recovered | 0.3955 | 0.9943 | (6,0) weak | 0.3930 | 0.9945 | 255 | 0.0032 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | de1020c | 1/1 | ok | 0.4244 | 0.9933 | (-49,0) weak | 0.4148 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | main | 1/1 | ok | 0.4244 | 0.9933 | (-49,0) weak | 0.4148 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | pipeline | 1/1 | recovered | 0.4306 | 0.9934 | (-35,0) weak | 0.4210 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.3837 | 0.9944 | (-0.5,0) weak | 0.3764 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.3837 | 0.9944 | (-0.5,0) weak | 0.3764 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | pipeline | 1/1 | recovered | 0.3884 | 0.9944 | (0,0) | 0.3884 | 0.9944 | 255 | 0.0032 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | de1020c | 1/1 | ok | 0.4246 | 0.9933 | (-49,0) weak | 0.4162 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | main | 1/1 | ok | 0.4246 | 0.9933 | (-49,0) weak | 0.4162 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | pipeline | 1/1 | recovered | 0.4313 | 0.9934 | (-35,0) weak | 0.4220 | 0.9934 | 255 | 0.0034 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.3849 | 0.9944 | (-0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0032 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.3849 | 0.9944 | (-0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0032 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | pipeline | 1/1 | recovered | 0.3962 | 0.9943 | (3,0) weak | 0.3944 | 0.9945 | 255 | 0.0032 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | de1020c | 1/1 | ok | 0.4245 | 0.9933 | (-49,0) weak | 0.4138 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | main | 1/1 | ok | 0.4245 | 0.9933 | (-49,0) weak | 0.4138 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | pipeline | 1/1 | recovered | 0.4257 | 0.9935 | (-35,0) weak | 0.4191 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.2971 | 0.9955 | (0,0) | 0.2971 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.2971 | 0.9955 | (0,0) | 0.2971 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | pipeline | 1/1 | recovered | 0.3306 | 0.9953 | (0,0) | 0.3306 | 0.9953 | 255 | 0.0028 | 0.0021 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex-lm | de1020c | 1/1 | ok | 0.3619 | 0.9938 | (2.5,0) weak | 0.3494 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | main | 1/1 | ok | 0.3619 | 0.9938 | (2.5,0) weak | 0.3494 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | pipeline | 1/1 | recovered | 0.3559 | 0.9943 | (-5.5,0) moderate | 0.3115 | 0.9949 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.3125 | 0.9955 | (0,0) | 0.3125 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.3125 | 0.9955 | (0,0) | 0.3125 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex | pipeline | 1/1 | recovered | 0.3411 | 0.9954 | (3.5,0) weak | 0.3330 | 0.9956 | 255 | 0.0028 | 0.0021 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex-lm | de1020c | 1/1 | ok | 0.3618 | 0.9938 | (2.5,0) weak | 0.3496 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex-lm | main | 1/1 | ok | 0.3618 | 0.9938 | (2.5,0) weak | 0.3496 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex-lm | pipeline | 1/1 | recovered | 0.3558 | 0.9943 | (-5.5,0) moderate | 0.3116 | 0.9949 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.2963 | 0.9955 | (0,0) | 0.2963 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.2963 | 0.9955 | (0,0) | 0.2963 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | pipeline | 1/1 | recovered | 0.3300 | 0.9953 | (0,0) | 0.3300 | 0.9953 | 255 | 0.0027 | 0.0021 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex-lm | de1020c | 1/1 | ok | 0.3620 | 0.9938 | (2.5,0) weak | 0.3493 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | main | 1/1 | ok | 0.3620 | 0.9938 | (2.5,0) weak | 0.3493 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | pipeline | 1/1 | recovered | 0.3559 | 0.9943 | (-5.5,0) moderate | 0.3114 | 0.9949 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.3823 | 0.9934 | (0,3.5) | 0.2551 | 0.9959 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | ok | 0.3823 | 0.9934 | (0,3.5) | 0.2551 | 0.9959 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | pipeline | 1/1 | recovered | 0.2549 | 0.9956 | (0.5,0) weak | 0.2538 | 0.9957 | 255 | 0.0022 | 0.0017 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | de1020c | 1/1 | ok | 0.3683 | 0.9931 | (-32.5,3.5) moderate | 0.3139 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | main | 1/1 | ok | 0.3683 | 0.9931 | (-32.5,3.5) moderate | 0.3139 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | pipeline | 1/1 | recovered | 0.2819 | 0.9949 | (-4,0) weak | 0.2728 | 0.9951 | 255 | 0.0023 | 0.0018 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.3812 | 0.9936 | (0,3.5) | 0.2591 | 0.9960 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | main | 1/1 | ok | 0.3812 | 0.9936 | (0,3.5) | 0.2591 | 0.9960 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | pipeline | 1/1 | recovered | 0.2593 | 0.9957 | (0,0) | 0.2593 | 0.9957 | 255 | 0.0022 | 0.0016 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | de1020c | 1/1 | ok | 0.3681 | 0.9931 | (-32.5,3.5) moderate | 0.3137 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | main | 1/1 | ok | 0.3681 | 0.9931 | (-32.5,3.5) moderate | 0.3137 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | pipeline | 1/1 | recovered | 0.2818 | 0.9949 | (-7,0) weak | 0.2780 | 0.9949 | 255 | 0.0023 | 0.0018 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.3823 | 0.9934 | (0,3.5) | 0.2545 | 0.9959 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | ok | 0.3823 | 0.9934 | (0,3.5) | 0.2545 | 0.9959 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | pipeline | 1/1 | recovered | 0.2548 | 0.9956 | (0.5,0) weak | 0.2535 | 0.9957 | 255 | 0.0022 | 0.0016 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | de1020c | 1/1 | ok | 0.3684 | 0.9931 | (-32.5,3.5) moderate | 0.3141 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | main | 1/1 | ok | 0.3684 | 0.9931 | (-32.5,3.5) moderate | 0.3141 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | pipeline | 1/1 | recovered | 0.2819 | 0.9949 | (-4,0) weak | 0.2728 | 0.9951 | 255 | 0.0023 | 0.0018 | -/-/- | - | - | - | - | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.2575 | 0.9950 | (0,0) | 0.2575 | 0.9950 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | ok | 0.2702 | 0.9946 | (0,0) | 0.2702 | 0.9946 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex | pipeline | 1/1 | recovered | 0.3473 | 0.9925 | (0,0) | 0.3473 | 0.9925 | 255 | 0.0027 | 0.0022 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | lualatex-lm | de1020c | 1/1 | ok | 0.3485 | 0.9931 | (0,0) | 0.3485 | 0.9931 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | main | 1/1 | ok | 0.3613 | 0.9927 | (0,0) | 0.3613 | 0.9927 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | pipeline | 1/1 | recovered | 0.3284 | 0.9924 | (0,0) | 0.3284 | 0.9924 | 255 | 0.0026 | 0.0022 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.2593 | 0.9950 | (0,0) | 0.2593 | 0.9950 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex | main | 1/1 | ok | 0.2720 | 0.9946 | (0,0) | 0.2720 | 0.9946 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex | pipeline | 1/1 | recovered | 0.3410 | 0.9927 | (0,0) | 0.3410 | 0.9927 | 255 | 0.0026 | 0.0021 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | pdflatex-lm | de1020c | 1/1 | ok | 0.3516 | 0.9931 | (0,0) | 0.3516 | 0.9931 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex-lm | main | 1/1 | ok | 0.3644 | 0.9927 | (0,0) | 0.3644 | 0.9927 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex-lm | pipeline | 1/1 | recovered | 0.3281 | 0.9924 | (0,0) | 0.3281 | 0.9924 | 255 | 0.0027 | 0.0022 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.2573 | 0.9950 | (0,0) | 0.2573 | 0.9950 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | ok | 0.2701 | 0.9946 | (0,0) | 0.2701 | 0.9946 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | pipeline | 1/1 | recovered | 0.3473 | 0.9925 | (0,0) | 0.3473 | 0.9925 | 255 | 0.0027 | 0.0022 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | xelatex-lm | de1020c | 1/1 | ok | 0.3485 | 0.9931 | (0,0) | 0.3485 | 0.9931 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | main | 1/1 | ok | 0.3613 | 0.9927 | (0,0) | 0.3613 | 0.9927 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | pipeline | 1/1 | recovered | 0.3284 | 0.9924 | (0,0) | 0.3284 | 0.9924 | 255 | 0.0026 | 0.0022 | -/-/- | - | - | - | - | 3/0 | - |
| 08-two-page | lualatex | de1020c | 3/3 | ok | 26.2189 | 0.5659 | (0,0); (-3,6) moderate; (-3,34.5) moderate | 24.6331 | 0.5959 | 255 | 0.1880 | 0.1574 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex | main | 3/3 | ok | 26.2189 | 0.5659 | (0,0); (-3,6) moderate; (-3,34.5) moderate | 24.6331 | 0.5959 | 255 | 0.1880 | 0.1574 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex | pipeline | 3/3 | recovered | 21.2380 | 0.6686 | (0,0); (0,0); (0,0) | 21.2380 | 0.6686 | 255 | 0.1626 | 0.1328 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | de1020c | 3/3 | ok | 23.7956 | 0.5635 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7517 | 0.5869 | 255 | 0.1801 | 0.1495 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | main | 3/3 | ok | 23.7956 | 0.5635 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7517 | 0.5869 | 255 | 0.1801 | 0.1495 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | pipeline | 3/3 | recovered | 19.5865 | 0.6686 | (-2.5,0) weak; (0.5,14.5) weak; (0,0) | 19.4457 | 0.6713 | 255 | 0.1568 | 0.1276 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/3 | ok | 26.1972 | 0.5669 | (0,-14.5) weak; (0,6) moderate; (0,34.5) moderate | 24.5512 | 0.5981 | 255 | 0.1875 | 0.1571 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | main | 3/3 | ok | 26.1972 | 0.5669 | (0,-14.5) weak; (0,6) moderate; (0,34.5) moderate | 24.5512 | 0.5981 | 255 | 0.1875 | 0.1571 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | pipeline | 3/3 | recovered | 20.9638 | 0.6739 | (0,0); (0,0); (0,0) | 20.9638 | 0.6739 | 255 | 0.1611 | 0.1314 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | de1020c | 3/3 | ok | 23.7951 | 0.5635 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7523 | 0.5869 | 255 | 0.1800 | 0.1496 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | main | 3/3 | ok | 23.7951 | 0.5635 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7523 | 0.5869 | 255 | 0.1800 | 0.1496 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | pipeline | 3/3 | recovered | 19.5847 | 0.6686 | (-2.5,0) weak; (0.5,14.5) weak; (0,0) | 19.4420 | 0.6714 | 255 | 0.1568 | 0.1277 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | de1020c | 3/3 | ok | 26.2441 | 0.5658 | (-3,-14.5) weak; (-3,6) moderate; (-3,34.5) moderate | 24.6604 | 0.5954 | 255 | 0.1881 | 0.1577 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | main | 3/3 | ok | 26.2441 | 0.5658 | (-3,-14.5) weak; (-3,6) moderate; (-3,34.5) moderate | 24.6604 | 0.5954 | 255 | 0.1881 | 0.1577 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | pipeline | 3/3 | recovered | 21.2026 | 0.6690 | (0,0); (0,0); (0,0) | 21.2026 | 0.6690 | 255 | 0.1626 | 0.1329 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | de1020c | 3/3 | ok | 23.7957 | 0.5634 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7513 | 0.5869 | 255 | 0.1800 | 0.1495 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | main | 3/3 | ok | 23.7957 | 0.5634 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7513 | 0.5869 | 255 | 0.1800 | 0.1495 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | pipeline | 3/3 | recovered | 19.5861 | 0.6686 | (-2.5,0) weak; (0.5,14.5) weak; (0,0) | 19.4446 | 0.6713 | 255 | 0.1568 | 0.1276 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.4844 | 0.9504 | (0,38.5) | 1.7906 | 0.9701 | 255 | 0.0174 | 0.0146 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | ok | 2.5141 | 0.9497 | (0,38.5) | 1.8130 | 0.9695 | 255 | 0.0176 | 0.0148 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | pipeline | 1/1 | recovered | 2.1111 | 0.9610 | (0,-47) | 1.5106 | 0.9741 | 255 | 0.0152 | 0.0128 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | de1020c | 1/1 | ok | 2.2538 | 0.9494 | (-0.5,39) moderate | 1.9068 | 0.9641 | 255 | 0.0169 | 0.0140 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | main | 1/1 | ok | 2.2692 | 0.9490 | (-0.5,39) moderate | 1.9292 | 0.9634 | 255 | 0.0170 | 0.0141 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | pipeline | 1/1 | recovered | 1.9577 | 0.9595 | (-2.5,-46.5) moderate | 1.8320 | 0.9648 | 255 | 0.0151 | 0.0124 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.4854 | 0.9504 | (-0.5,38.5) moderate | 1.8777 | 0.9688 | 255 | 0.0174 | 0.0146 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | main | 1/1 | ok | 2.5141 | 0.9497 | (-0.5,38.5) moderate | 1.9001 | 0.9681 | 255 | 0.0175 | 0.0148 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | pipeline | 1/1 | recovered | 2.1147 | 0.9609 | (0,-47) | 1.5068 | 0.9740 | 255 | 0.0152 | 0.0128 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | de1020c | 1/1 | ok | 2.2522 | 0.9480 | (-0.5,39.5) moderate | 1.9062 | 0.9622 | 255 | 0.0167 | 0.0139 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | main | 1/1 | ok | 2.2675 | 0.9475 | (-0.5,39.5) moderate | 1.9286 | 0.9615 | 255 | 0.0169 | 0.0140 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | pipeline | 1/1 | recovered | 1.9585 | 0.9580 | (-2.5,-46) moderate | 1.8360 | 0.9644 | 255 | 0.0149 | 0.0123 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.4838 | 0.9503 | (-0.5,38.5) | 1.8423 | 0.9692 | 255 | 0.0174 | 0.0146 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | ok | 2.5135 | 0.9496 | (-0.5,38.5) | 1.8647 | 0.9686 | 255 | 0.0176 | 0.0148 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | pipeline | 1/1 | recovered | 2.1113 | 0.9610 | (0,-47) | 1.4665 | 0.9744 | 255 | 0.0152 | 0.0127 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | de1020c | 1/1 | ok | 2.2538 | 0.9494 | (-0.5,39) moderate | 1.9067 | 0.9641 | 255 | 0.0169 | 0.0140 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | main | 1/1 | ok | 2.2692 | 0.9490 | (-0.5,39) moderate | 1.9291 | 0.9634 | 255 | 0.0170 | 0.0141 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | pipeline | 1/1 | recovered | 1.9580 | 0.9595 | (-2.5,-46.5) moderate | 1.8322 | 0.9648 | 255 | 0.0151 | 0.0124 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | lualatex | de1020c | 1/1 | ok | 3.3162 | 0.9491 | (0,0) | 3.3162 | 0.9491 | 255 | 0.0261 | 0.0212 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | main | 1/1 | ok | 3.3162 | 0.9491 | (0,0) | 3.3162 | 0.9491 | 255 | 0.0261 | 0.0212 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | pipeline | 1/1 | recovered | 3.4068 | 0.9475 | (0,0) | 3.4068 | 0.9475 | 255 | 0.0264 | 0.0214 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex-lm | de1020c | 1/1 | ok | 3.2405 | 0.9447 | (0,0) | 3.2405 | 0.9447 | 255 | 0.0259 | 0.0211 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | main | 1/1 | ok | 3.2405 | 0.9447 | (0,0) | 3.2405 | 0.9447 | 255 | 0.0259 | 0.0211 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 3.2093 | 0.9450 | (-1.5,0) weak | 3.2080 | 0.9451 | 255 | 0.0258 | 0.0209 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | pdflatex | de1020c | 1/1 | ok | 3.3644 | 0.9486 | (0,0) | 3.3644 | 0.9486 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | pdflatex | main | 1/1 | ok | 3.3644 | 0.9486 | (0,0) | 3.3644 | 0.9486 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | pdflatex | pipeline | 1/1 | recovered | 3.3448 | 0.9486 | (0,0) | 3.3448 | 0.9486 | 255 | 0.0261 | 0.0211 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 3.2558 | 0.9444 | (0,0) | 3.2558 | 0.9444 | 255 | 0.0260 | 0.0212 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | pdflatex-lm | main | 1/1 | ok | 3.2558 | 0.9444 | (0,0) | 3.2558 | 0.9444 | 255 | 0.0260 | 0.0212 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 3.2243 | 0.9448 | (-2,0) weak | 3.1793 | 0.9456 | 255 | 0.0259 | 0.0209 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | xelatex | de1020c | 1/1 | ok | 3.3626 | 0.9484 | (0,0) | 3.3626 | 0.9484 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | xelatex | main | 1/1 | ok | 3.3626 | 0.9484 | (0,0) | 3.3626 | 0.9484 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | xelatex | pipeline | 1/1 | recovered | 3.3755 | 0.9479 | (0,0) | 3.3755 | 0.9479 | 255 | 0.0262 | 0.0212 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | xelatex-lm | de1020c | 1/1 | ok | 3.2406 | 0.9446 | (0,0) | 3.2406 | 0.9446 | 255 | 0.0259 | 0.0211 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | main | 1/1 | ok | 3.2406 | 0.9446 | (0,0) | 3.2406 | 0.9446 | 255 | 0.0259 | 0.0211 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 3.2091 | 0.9450 | (-1.5,0) weak | 3.2080 | 0.9451 | 255 | 0.0258 | 0.0209 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | de1020c | 1/1 | recovered | 1.4652 | 0.9696 | (18,-49) weak | 1.4604 | 0.9709 | 255 | 0.0104 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | main | 1/1 | ok | 1.4651 | 0.9700 | (-21,-8) moderate | 1.2181 | 0.9785 | 255 | 0.0105 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | pipeline | 1/1 | recovered | 1.5034 | 0.9691 | (20,-49) weak | 1.4565 | 0.9706 | 255 | 0.0105 | 0.0088 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (-2,-49) weak | 1.3068 | 0.9711 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | main | 1/1 | ok | 1.3732 | 0.9689 | (-29,-8) moderate | 1.2485 | 0.9756 | 255 | 0.0101 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | pipeline | 1/1 | recovered | 1.3525 | 0.9690 | (54,-49) weak | 1.3119 | 0.9705 | 255 | 0.0099 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | de1020c | 1/1 | recovered | 1.4655 | 0.9698 | (19,-49) weak | 1.4558 | 0.9711 | 255 | 0.0104 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | main | 1/1 | ok | 1.4715 | 0.9702 | (-20,-8) moderate | 1.1981 | 0.9790 | 255 | 0.0105 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | pipeline | 1/1 | recovered | 1.4978 | 0.9693 | (53,-49) weak | 1.4608 | 0.9710 | 255 | 0.0104 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | de1020c | 1/1 | recovered | 1.3674 | 0.9687 | (23.5,-49) weak | 1.3215 | 0.9706 | 255 | 0.0101 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | main | 1/1 | ok | 1.3674 | 0.9690 | (-29,-8) moderate | 1.2361 | 0.9758 | 255 | 0.0102 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | pipeline | 1/1 | recovered | 1.3521 | 0.9690 | (52,-49) weak | 1.3047 | 0.9706 | 255 | 0.0099 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | de1020c | 1/1 | recovered | 1.4648 | 0.9696 | (18,-49) weak | 1.4599 | 0.9709 | 255 | 0.0104 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | main | 1/1 | ok | 1.4647 | 0.9700 | (-21,-8) moderate | 1.2168 | 0.9785 | 255 | 0.0105 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | pipeline | 1/1 | recovered | 1.5034 | 0.9690 | (52,-49) weak | 1.4568 | 0.9706 | 255 | 0.0105 | 0.0088 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (1,-49) weak | 1.3138 | 0.9708 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | main | 1/1 | ok | 1.3731 | 0.9689 | (-29,-8) moderate | 1.2487 | 0.9756 | 255 | 0.0101 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | pipeline | 1/1 | recovered | 1.3526 | 0.9690 | (60,-49) weak | 1.3193 | 0.9706 | 255 | 0.0099 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | de1020c | 1/1 | ok | 15.4036 | 0.7397 | (-3,0) weak | 15.3867 | 0.7398 | 255 | 0.1111 | 0.0927 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | main | 1/1 | ok | 15.4036 | 0.7397 | (-3,0) weak | 15.3867 | 0.7398 | 255 | 0.1111 | 0.0927 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | pipeline | 1/1 | recovered | 13.1332 | 0.7892 | (-0.5,14.5) weak | 13.0053 | 0.7920 | 255 | 0.0993 | 0.0813 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | de1020c | 1/1 | ok | 13.7975 | 0.7504 | (-8.5,2.5) weak | 13.7722 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | main | 1/1 | ok | 13.7975 | 0.7504 | (-8.5,2.5) weak | 13.7722 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | pipeline | 1/1 | recovered | 11.6373 | 0.8076 | (1,0) weak | 11.5456 | 0.8076 | 255 | 0.0931 | 0.0759 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | de1020c | 1/1 | ok | 15.3703 | 0.7416 | (-3,0) weak | 15.3659 | 0.7415 | 255 | 0.1107 | 0.0925 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | main | 1/1 | ok | 15.3703 | 0.7416 | (-3,0) weak | 15.3659 | 0.7415 | 255 | 0.1107 | 0.0925 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | pipeline | 1/1 | recovered | 13.0478 | 0.7917 | (-0.5,14.5) weak | 12.8874 | 0.7948 | 255 | 0.0987 | 0.0809 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | de1020c | 1/1 | ok | 13.7973 | 0.7504 | (-8.5,2.5) weak | 13.7711 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | main | 1/1 | ok | 13.7973 | 0.7504 | (-8.5,2.5) weak | 13.7711 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | pipeline | 1/1 | recovered | 11.6366 | 0.8076 | (1,0) weak | 11.5382 | 0.8077 | 255 | 0.0930 | 0.0759 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | de1020c | 1/1 | ok | 15.3817 | 0.7405 | (0,0) | 15.3817 | 0.7405 | 255 | 0.1110 | 0.0928 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | main | 1/1 | ok | 15.3817 | 0.7405 | (0,0) | 15.3817 | 0.7405 | 255 | 0.1110 | 0.0928 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | pipeline | 1/1 | recovered | 13.1399 | 0.7892 | (-0.5,14.5) weak | 13.0409 | 0.7918 | 255 | 0.0995 | 0.0816 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | de1020c | 1/1 | ok | 13.7974 | 0.7504 | (-8.5,2.5) weak | 13.7718 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | main | 1/1 | ok | 13.7974 | 0.7504 | (-8.5,2.5) weak | 13.7718 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | pipeline | 1/1 | recovered | 11.6370 | 0.8076 | (1,0) weak | 11.5454 | 0.8076 | 255 | 0.0930 | 0.0759 | -/-/- | - | - | - | - | 0/0 | - |
| 13-math-display-rich | lualatex | de1020c | 1/1 | recovered | 0.4665 | 0.9904 | (0,3.5) moderate | 0.4400 | 0.9913 | 255 | 0.0040 | 0.0031 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex | main | 1/1 | recovered | 0.4792 | 0.9900 | (0,3.5) moderate | 0.4528 | 0.9910 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex | pipeline | 1/1 | recovered | 0.4416 | 0.9896 | (0,0) | 0.4416 | 0.9896 | 255 | 0.0034 | 0.0027 | -/-/- | - | - | - | - | 2/0 | - |
| 13-math-display-rich | lualatex-lm | de1020c | 1/1 | recovered | 0.5457 | 0.9888 | (0,0) | 0.5457 | 0.9888 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | lualatex-lm | main | 1/1 | recovered | 0.5585 | 0.9885 | (0,0) | 0.5585 | 0.9885 | 255 | 0.0044 | 0.0036 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | lualatex-lm | pipeline | 1/1 | recovered | 0.4464 | 0.9890 | (-0.5,-36) weak | 0.4283 | 0.9899 | 255 | 0.0034 | 0.0028 | -/-/- | - | - | - | - | 4/0 | - |
| 13-math-display-rich | pdflatex | de1020c | 1/1 | recovered | 0.4914 | 0.9901 | (1,3.5) weak | 0.4902 | 0.9905 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | pdflatex | main | 1/1 | recovered | 0.5042 | 0.9897 | (1,3.5) weak | 0.5030 | 0.9902 | 255 | 0.0042 | 0.0033 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | pdflatex | pipeline | 1/1 | recovered | 0.4400 | 0.9896 | (3.5,-36) weak | 0.4356 | 0.9904 | 255 | 0.0034 | 0.0027 | -/-/- | - | - | - | - | 2/0 | - |
| 13-math-display-rich | pdflatex-lm | de1020c | 1/1 | recovered | 0.5476 | 0.9889 | (-2,4) weak | 0.5407 | 0.9892 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | pdflatex-lm | main | 1/1 | recovered | 0.5603 | 0.9885 | (-2,4) weak | 0.5535 | 0.9889 | 255 | 0.0044 | 0.0036 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | pdflatex-lm | pipeline | 1/1 | recovered | 0.4462 | 0.9890 | (-0.5,-35.5) weak | 0.4314 | 0.9897 | 255 | 0.0034 | 0.0028 | -/-/- | - | - | - | - | 4/0 | - |
| 13-math-display-rich | xelatex | de1020c | 1/1 | recovered | 0.4850 | 0.9901 | (0,0) | 0.4850 | 0.9901 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex | main | 1/1 | recovered | 0.4978 | 0.9897 | (0,0) | 0.4978 | 0.9897 | 255 | 0.0042 | 0.0033 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex | pipeline | 1/1 | recovered | 0.4415 | 0.9896 | (0,0) | 0.4415 | 0.9896 | 255 | 0.0034 | 0.0028 | -/-/- | - | - | - | - | 2/0 | - |
| 13-math-display-rich | xelatex-lm | de1020c | 1/1 | recovered | 0.5459 | 0.9888 | (0,0) | 0.5459 | 0.9888 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | xelatex-lm | main | 1/1 | recovered | 0.5587 | 0.9885 | (0,0) | 0.5587 | 0.9885 | 255 | 0.0044 | 0.0036 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | xelatex-lm | pipeline | 1/1 | recovered | 0.4465 | 0.9890 | (-0.5,-36) weak | 0.4283 | 0.9899 | 255 | 0.0034 | 0.0028 | -/-/- | - | - | - | - | 4/0 | - |
| 14-math-inline-dense | lualatex | de1020c | 1/1 | recovered | 1.3153 | 0.9706 | (-34.5,13.5) | 0.9661 | 0.9780 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex | main | 1/1 | recovered | 1.3153 | 0.9706 | (-34.5,13.5) | 0.9661 | 0.9780 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex | pipeline | 1/1 | recovered | 0.8299 | 0.9819 | (0.5,-14.5) weak | 0.7928 | 0.9821 | 255 | 0.0066 | 0.0053 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | lualatex-lm | de1020c | 1/1 | recovered | 1.2497 | 0.9701 | (2.5,13.5) moderate | 1.1687 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | main | 1/1 | recovered | 1.2497 | 0.9701 | (2.5,13.5) moderate | 1.1687 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | pipeline | 1/1 | recovered | 0.8055 | 0.9808 | (-0.5,-14.5) moderate | 0.7305 | 0.9822 | 255 | 0.0065 | 0.0053 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | pdflatex | de1020c | 1/1 | recovered | 1.3096 | 0.9709 | (-34,13.5) | 0.9598 | 0.9781 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | pdflatex | main | 1/1 | recovered | 1.3096 | 0.9709 | (-34,13.5) | 0.9598 | 0.9781 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | pdflatex | pipeline | 1/1 | recovered | 0.8200 | 0.9822 | (1,-14.5) weak | 0.7900 | 0.9825 | 255 | 0.0065 | 0.0052 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | pdflatex-lm | de1020c | 1/1 | recovered | 1.2493 | 0.9701 | (2.5,13.5) moderate | 1.1681 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | pdflatex-lm | main | 1/1 | recovered | 1.2493 | 0.9701 | (2.5,13.5) moderate | 1.1681 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | pdflatex-lm | pipeline | 1/1 | recovered | 0.8050 | 0.9808 | (-0.5,-14.5) moderate | 0.7299 | 0.9822 | 255 | 0.0065 | 0.0053 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | xelatex | de1020c | 1/1 | recovered | 1.3153 | 0.9706 | (-34.5,13.5) | 0.9632 | 0.9781 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex | main | 1/1 | recovered | 1.3153 | 0.9706 | (-34.5,13.5) | 0.9632 | 0.9781 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex | pipeline | 1/1 | recovered | 0.8298 | 0.9819 | (0.5,-14.5) weak | 0.7928 | 0.9821 | 255 | 0.0066 | 0.0053 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | xelatex-lm | de1020c | 1/1 | recovered | 1.2496 | 0.9701 | (2.5,13.5) moderate | 1.1687 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | main | 1/1 | recovered | 1.2496 | 0.9701 | (2.5,13.5) moderate | 1.1687 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | pipeline | 1/1 | recovered | 0.8055 | 0.9808 | (-1,-14.5) moderate | 0.7445 | 0.9819 | 255 | 0.0065 | 0.0053 | -/-/- | - | - | - | - | 2/0 | - |
| 15-three-page-sections | lualatex | de1020c | 3/3 | recovered | 26.4238 | 0.5637 | (0.5,20) moderate; (0.5,19) weak; (0.5,-24.5) moderate | 25.2152 | 0.5876 | 255 | 0.1893 | 0.1584 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex | main | 3/3 | recovered | 26.4375 | 0.5634 | (0.5,20) moderate; (0.5,19) weak; (0.5,-24.5) moderate | 25.2267 | 0.5874 | 255 | 0.1894 | 0.1585 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex | pipeline | 3/3 | recovered | 23.8435 | 0.6074 | (0.5,14) weak; (0.5,-26.5) moderate; (0.5,-55.5) moderate | 21.5336 | 0.6637 | 255 | 0.1765 | 0.1463 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex-lm | de1020c | 3/3 | recovered | 24.2613 | 0.5495 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0400 | 0.5821 | 255 | 0.1832 | 0.1519 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | lualatex-lm | main | 3/3 | recovered | 24.2719 | 0.5494 | (0.5,22.5) moderate; (0,-24) weak; (0.5,-24) weak | 23.0396 | 0.5821 | 255 | 0.1832 | 0.1520 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | lualatex-lm | pipeline | 3/3 | recovered | 21.9324 | 0.6000 | (0.5,0) weak; (0.5,-26.5) moderate; (0.5,-26.5) moderate | 20.2282 | 0.6462 | 255 | 0.1706 | 0.1400 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex | de1020c | 3/3 | recovered | 26.6983 | 0.5543 | (0.5,36.5) moderate; (0.5,-24.5) weak; (0,-24.5) weak | 25.1309 | 0.5916 | 255 | 0.1901 | 0.1598 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex | main | 3/3 | recovered | 26.7124 | 0.5541 | (0.5,36.5) moderate; (0.5,-24.5) weak; (0,-24.5) weak | 25.1424 | 0.5914 | 255 | 0.1902 | 0.1599 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex | pipeline | 3/3 | recovered | 23.7591 | 0.6054 | (0,-0.5) moderate; (0,-55.5) moderate; (0,2) moderate | 21.5951 | 0.6585 | 255 | 0.1758 | 0.1462 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex-lm | de1020c | 3/3 | recovered | 24.2616 | 0.5495 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0365 | 0.5822 | 255 | 0.1832 | 0.1521 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex-lm | main | 3/3 | recovered | 24.2722 | 0.5493 | (0.5,22.5) moderate; (0,-24) weak; (0.5,-24) weak | 23.0367 | 0.5823 | 255 | 0.1833 | 0.1521 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex-lm | pipeline | 3/3 | recovered | 21.9357 | 0.5999 | (0.5,0) weak; (0.5,-26.5) moderate; (0.5,-26.5) moderate | 20.2318 | 0.6460 | 255 | 0.1706 | 0.1401 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex | de1020c | 3/3 | recovered | 26.4204 | 0.5639 | (-3,20) moderate; (-3,19) weak; (-3,-24.5) moderate | 25.1894 | 0.5880 | 255 | 0.1893 | 0.1587 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex | main | 3/3 | recovered | 26.4341 | 0.5636 | (-3,20) moderate; (-3,19) weak; (-3,-24.5) moderate | 25.2009 | 0.5877 | 255 | 0.1894 | 0.1588 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex | pipeline | 3/3 | recovered | 23.8252 | 0.6079 | (0,14) weak; (0,-26.5) moderate; (0,-55.5) moderate | 21.5201 | 0.6641 | 255 | 0.1764 | 0.1466 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex-lm | de1020c | 3/3 | recovered | 24.2611 | 0.5495 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0399 | 0.5821 | 255 | 0.1831 | 0.1519 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex-lm | main | 3/3 | recovered | 24.2717 | 0.5493 | (0.5,22.5) moderate; (3.5,-24) weak; (0.5,-24) weak | 23.0722 | 0.5811 | 255 | 0.1832 | 0.1520 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex-lm | pipeline | 3/3 | recovered | 21.9320 | 0.6000 | (0.5,0) weak; (0.5,-26.5) moderate; (0.5,-26.5) moderate | 20.2280 | 0.6462 | 255 | 0.1706 | 0.1400 | -/-/- | - | - | - | - | 3/0 | - |
| 16-heading-page-break | lualatex | de1020c | 2/2 | ok | 20.5841 | 0.6613 | (0,0); (0,20.5) moderate | 20.0997 | 0.6688 | 255 | 0.1484 | 0.1240 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex | main | 2/2 | ok | 20.5889 | 0.6612 | (0,0); (0,20.5) moderate | 20.1043 | 0.6687 | 255 | 0.1484 | 0.1240 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex | pipeline | 2/2 | recovered | 17.5877 | 0.7205 | (0,0); (0,57.5) weak | 17.5211 | 0.7211 | 255 | 0.1335 | 0.1092 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex-lm | de1020c | 2/2 | ok | 19.0167 | 0.6509 | (-1,0) weak; (-1,20) moderate | 18.0483 | 0.6755 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | lualatex-lm | main | 2/2 | ok | 19.0237 | 0.6509 | (-1,0) weak; (-1,20) moderate | 18.0517 | 0.6755 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | lualatex-lm | pipeline | 2/2 | recovered | 15.8910 | 0.7330 | (-2.5,0) weak; (0.5,14.5) weak | 15.6885 | 0.7370 | 255 | 0.1266 | 0.1032 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex | de1020c | 2/2 | ok | 20.5976 | 0.6621 | (0,-14.5) weak; (0,20.5) moderate | 20.0459 | 0.6704 | 255 | 0.1481 | 0.1240 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex | main | 2/2 | ok | 20.6024 | 0.6620 | (0,-14.5) weak; (0,20.5) moderate | 20.0509 | 0.6704 | 255 | 0.1481 | 0.1240 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex | pipeline | 2/2 | recovered | 17.3544 | 0.7254 | (0,0); (0,57.5) weak | 17.3265 | 0.7254 | 255 | 0.1322 | 0.1081 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex-lm | de1020c | 2/2 | ok | 19.0160 | 0.6507 | (-1,0) weak; (-1,20) moderate | 18.0489 | 0.6754 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex-lm | main | 2/2 | ok | 19.0230 | 0.6507 | (-1,0) weak; (-1,20) moderate | 18.0522 | 0.6754 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex-lm | pipeline | 2/2 | recovered | 15.8880 | 0.7331 | (-2.5,0) weak; (0.5,14.5) weak | 15.6809 | 0.7371 | 255 | 0.1266 | 0.1032 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex | de1020c | 2/2 | ok | 20.6109 | 0.6611 | (-3,-14.5) weak; (0.5,20.5) moderate | 20.1127 | 0.6683 | 255 | 0.1485 | 0.1243 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex | main | 2/2 | ok | 20.6157 | 0.6611 | (-3,-14.5) weak; (0,20.5) moderate | 20.1384 | 0.6680 | 255 | 0.1485 | 0.1243 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex | pipeline | 2/2 | recovered | 17.5365 | 0.7212 | (0,0); (0,57.5) weak | 17.4545 | 0.7220 | 255 | 0.1334 | 0.1093 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex-lm | de1020c | 2/2 | ok | 19.0167 | 0.6509 | (-1,0) weak; (-1,20) moderate | 18.0480 | 0.6754 | 255 | 0.1438 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex-lm | main | 2/2 | ok | 19.0238 | 0.6508 | (-1,0) weak; (-1,20) moderate | 18.0514 | 0.6754 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex-lm | pipeline | 2/2 | recovered | 15.8903 | 0.7330 | (-2.5,0) weak; (0.5,14.5) weak | 15.6873 | 0.7370 | 255 | 0.1266 | 0.1032 | -/-/- | - | - | - | - | 1/0 | - |
| 17-apostrophes | lualatex | de1020c | 1/1 | ok | 0.8503 | 0.9875 | (-1.5,0) weak | 0.8116 | 0.9882 | 255 | 0.0068 | 0.0053 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex | main | 1/1 | ok | 0.8503 | 0.9875 | (-1.5,0) weak | 0.8116 | 0.9882 | 255 | 0.0068 | 0.0053 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex | pipeline | 1/1 | recovered | 0.9575 | 0.9850 | (0,0) | 0.9575 | 0.9850 | 255 | 0.0074 | 0.0059 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | de1020c | 1/1 | ok | 0.8624 | 0.9848 | (0,0) | 0.8624 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | main | 1/1 | ok | 0.8624 | 0.9848 | (0,0) | 0.8624 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | pipeline | 1/1 | recovered | 0.8324 | 0.9857 | (-2.5,0) moderate | 0.7680 | 0.9874 | 255 | 0.0068 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | de1020c | 1/1 | ok | 0.8064 | 0.9887 | (0,0) | 0.8064 | 0.9887 | 255 | 0.0066 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | main | 1/1 | ok | 0.8064 | 0.9887 | (0,0) | 0.8064 | 0.9887 | 255 | 0.0066 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | pipeline | 1/1 | recovered | 0.9532 | 0.9853 | (9,0) weak | 0.9305 | 0.9858 | 255 | 0.0073 | 0.0058 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | de1020c | 1/1 | ok | 0.8626 | 0.9848 | (0,0) | 0.8626 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | main | 1/1 | ok | 0.8626 | 0.9848 | (0,0) | 0.8626 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | pipeline | 1/1 | recovered | 0.8327 | 0.9857 | (-2.5,0) moderate | 0.7676 | 0.9874 | 255 | 0.0068 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | de1020c | 1/1 | ok | 0.7982 | 0.9883 | (0,0) | 0.7982 | 0.9883 | 255 | 0.0066 | 0.0051 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | main | 1/1 | ok | 0.7982 | 0.9883 | (0,0) | 0.7982 | 0.9883 | 255 | 0.0066 | 0.0051 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | pipeline | 1/1 | recovered | 0.9539 | 0.9850 | (6.5,0) weak | 0.9416 | 0.9853 | 255 | 0.0074 | 0.0059 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | de1020c | 1/1 | ok | 0.8621 | 0.9848 | (0,0) | 0.8621 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | main | 1/1 | ok | 0.8621 | 0.9848 | (0,0) | 0.8621 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | pipeline | 1/1 | recovered | 0.8325 | 0.9857 | (-2.5,0) moderate | 0.7678 | 0.9874 | 255 | 0.0068 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | de1020c | 1/1 | ok | 1.3878 | 0.9788 | (0,0) | 1.3878 | 0.9788 | 255 | 0.0109 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | main | 1/1 | ok | 1.3878 | 0.9788 | (0,0) | 1.3878 | 0.9788 | 255 | 0.0109 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | pipeline | 1/1 | recovered | 1.0292 | 0.9862 | (0.5,0) moderate | 0.9126 | 0.9878 | 255 | 0.0093 | 0.0071 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | de1020c | 1/1 | ok | 1.3042 | 0.9789 | (0,0) | 1.3042 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | main | 1/1 | ok | 1.3042 | 0.9789 | (0,0) | 1.3042 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | pipeline | 1/1 | recovered | 1.3044 | 0.9787 | (0,0) | 1.3044 | 0.9787 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | de1020c | 1/1 | ok | 1.2717 | 0.9812 | (0,0) | 1.2717 | 0.9812 | 255 | 0.0104 | 0.0080 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | main | 1/1 | ok | 1.2717 | 0.9812 | (0,0) | 1.2717 | 0.9812 | 255 | 0.0104 | 0.0080 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | pipeline | 1/1 | recovered | 0.6206 | 0.9926 | (0,0) | 0.6206 | 0.9926 | 255 | 0.0079 | 0.0049 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | de1020c | 1/1 | ok | 1.3056 | 0.9789 | (0,0) | 1.3056 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | main | 1/1 | ok | 1.3056 | 0.9789 | (0,0) | 1.3056 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | pipeline | 1/1 | recovered | 1.3049 | 0.9787 | (0,0) | 1.3049 | 0.9787 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | de1020c | 1/1 | ok | 1.3346 | 0.9799 | (0,0) | 1.3346 | 0.9799 | 255 | 0.0107 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | main | 1/1 | ok | 1.3346 | 0.9799 | (0,0) | 1.3346 | 0.9799 | 255 | 0.0107 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | pipeline | 1/1 | recovered | 0.6841 | 0.9915 | (0,0) | 0.6841 | 0.9915 | 255 | 0.0083 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | de1020c | 1/1 | ok | 1.3041 | 0.9789 | (0,0) | 1.3041 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | main | 1/1 | ok | 1.3041 | 0.9789 | (0,0) | 1.3041 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | pipeline | 1/1 | recovered | 1.3043 | 0.9787 | (0,0) | 1.3043 | 0.9787 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |

## Per-fixture diagnostic details (export side)

### 01-plain-paragraph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935537, 648, 477, 427, 319, 237, 160, 146, 149, 113, 138, 79, 113, 76, 54, 143]`; ink px ref/ours 2442/2421 (ratio 0.9914); SSIM blocks <0.9: 131/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.95, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1575, differing 0.002279, SSIM₈ 0.998 (raw 0.1575, 0.002279, 0.998)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9969→0.9969 / 0.2517→0.2517; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `This` dx -0.75 dy 0.46; `is` dx -0.65 dy 0.46; `one` dx -0.62 dy 0.46

### 01-plain-paragraph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935537, 648, 477, 427, 319, 237, 160, 146, 149, 113, 138, 79, 113, 76, 54, 143]`; ink px ref/ours 2442/2421 (ratio 0.9914); SSIM blocks <0.9: 131/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.95, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1575, differing 0.002279, SSIM₈ 0.998 (raw 0.1575, 0.002279, 0.998)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9969→0.9969 / 0.2517→0.2517; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `This` dx -0.75 dy 0.46; `is` dx -0.65 dy 0.46; `one` dx -0.62 dy 0.46

### 01-plain-paragraph — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933605, 526, 348, 365, 361, 297, 268, 224, 251, 233, 277, 303, 287, 193, 250, 1028]`; ink px ref/ours 2442/2417 (ratio 0.9898); SSIM blocks <0.9: 212/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [4.46, 0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.385, differing 0.003121, SSIM₈ 0.9944 (raw 0.385, 0.003121, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.991→0.991 / 0.6154→0.6154; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 10.52 dy 0.41; `single` dx 9.6 dy 0.41; `a` dx 8.69 dy 0.41

### 01-plain-paragraph — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933594, 461, 436, 345, 357, 334, 344, 324, 320, 261, 321, 233, 248, 209, 197, 832]`; ink px ref/ours 1890/2421 (ratio 1.281); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-10.52, -0.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3662, differing 0.003027, SSIM₈ 0.9941 (raw 0.3662, 0.003027, 0.9941)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9906→0.9906 / 0.5854→0.5854; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -24.74 dy -0.3; `single` dx -23.83 dy -0.3; `a` dx -22.41 dy -0.3

### 01-plain-paragraph — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933594, 461, 436, 345, 357, 334, 344, 324, 320, 261, 321, 233, 248, 209, 197, 832]`; ink px ref/ours 1890/2421 (ratio 1.281); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-10.52, -0.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3662, differing 0.003027, SSIM₈ 0.9941 (raw 0.3662, 0.003027, 0.9941)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9906→0.9906 / 0.5854→0.5854; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -24.74 dy -0.3; `single` dx -23.83 dy -0.3; `a` dx -22.41 dy -0.3

### 01-plain-paragraph — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933494, 477, 428, 367, 396, 385, 333, 290, 334, 274, 274, 226, 250, 201, 225, 862]`; ink px ref/ours 1890/2417 (ratio 1.2788); SSIM blocks <0.9: 226/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.11, -0.11] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3718, differing 0.003074, SSIM₈ 0.9942 (raw 0.3718, 0.003074, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9907→0.9907 / 0.5942→0.5942; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -14.35 dy -0.36; `single` dx -14.24 dy -0.36; `a` dx -13.69 dy -0.36

### 01-plain-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935388, 716, 424, 359, 290, 196, 196, 126, 136, 124, 122, 86, 78, 86, 81, 408]`; ink px ref/ours 2408/2421 (ratio 1.0054); SSIM blocks <0.9: 119/30294; [overlay](images/01-plain-paragraph/pdflatex-de1020c-export-p1-overlay.png) (77366 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (71851 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.1, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1873, differing 0.002361, SSIM₈ 0.9975 (raw 0.1873, 0.002361, 0.9975)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9961→0.9961 / 0.2993→0.2993; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 1.15 dy 0.46; `single` dx 1.03 dy 0.46; `a` dx 1.0 dy 0.46

### 01-plain-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935388, 716, 424, 359, 290, 196, 196, 126, 136, 124, 122, 86, 78, 86, 81, 408]`; ink px ref/ours 2408/2421 (ratio 1.0054); SSIM blocks <0.9: 119/30294; [overlay](images/01-plain-paragraph/pdflatex-main-export-p1-overlay.png) (77131 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-main-export-p1-heatmap.png) (71616 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.1, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1873, differing 0.002361, SSIM₈ 0.9975 (raw 0.1873, 0.002361, 0.9975)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9961→0.9961 / 0.2993→0.2993; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 1.15 dy 0.46; `single` dx 1.03 dy 0.46; `a` dx 1.0 dy 0.46

### 01-plain-paragraph — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933616, 523, 344, 294, 314, 310, 265, 278, 231, 246, 275, 274, 258, 239, 261, 1088]`; ink px ref/ours 2408/2417 (ratio 1.0037); SSIM blocks <0.9: 209/30294; [overlay](images/01-plain-paragraph/pdflatex-pipeline-export-p1-overlay.png) (78161 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-pipeline-export-p1-heatmap.png) (75194 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [5.31, 0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3928, differing 0.00311, SSIM₈ 0.9945 (raw 0.3928, 0.00311, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9912→0.9912 / 0.6278→0.6278; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 11.54 dy 0.41; `single` dx 10.63 dy 0.41; `a` dx 9.71 dy 0.41

### 01-plain-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933593, 464, 437, 341, 371, 326, 338, 319, 328, 279, 309, 227, 246, 191, 211, 836]`; ink px ref/ours 1898/2421 (ratio 1.2756); SSIM blocks <0.9: 235/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (76560 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (73024 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-9.72, -0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.366, differing 0.003024, SSIM₈ 0.9942 (raw 0.366, 0.003024, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9907→0.9907 / 0.5849→0.5849; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -24.75 dy 0.73; `single` dx -23.83 dy 0.73; `a` dx -22.41 dy 0.73

### 01-plain-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933593, 464, 437, 341, 371, 326, 338, 319, 328, 279, 309, 227, 246, 191, 211, 836]`; ink px ref/ours 1898/2421 (ratio 1.2756); SSIM blocks <0.9: 235/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-main-export-p1-overlay.png) (76168 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (72726 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-9.72, -0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.366, differing 0.003024, SSIM₈ 0.9942 (raw 0.366, 0.003024, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9907→0.9907 / 0.5849→0.5849; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -24.75 dy 0.73; `single` dx -23.83 dy 0.73; `a` dx -22.41 dy 0.73

### 01-plain-paragraph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933494, 477, 421, 374, 415, 376, 318, 294, 341, 288, 255, 235, 235, 193, 232, 868]`; ink px ref/ours 1898/2417 (ratio 1.2734); SSIM blocks <0.9: 226/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) (76731 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-pipeline-export-p1-heatmap.png) (73060 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-3.31, -0.12] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3717, differing 0.00307, SSIM₈ 0.9942 (raw 0.3717, 0.00307, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9907→0.9907 / 0.594→0.594; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -14.36 dy 0.67; `single` dx -14.23 dy 0.67; `a` dx -13.69 dy 0.67

### 01-plain-paragraph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935492, 657, 509, 378, 314, 240, 190, 128, 161, 125, 139, 77, 101, 98, 61, 146]`; ink px ref/ours 2441/2421 (ratio 0.9918); SSIM blocks <0.9: 134/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-2.11, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1607, differing 0.002287, SSIM₈ 0.998 (raw 0.1607, 0.002287, 0.998)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9968→0.9968 / 0.2569→0.2569; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `This` dx -0.75 dy 0.46; `is` dx -0.66 dy 0.46; `one` dx -0.62 dy 0.46

### 01-plain-paragraph — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935492, 657, 509, 378, 314, 240, 190, 128, 161, 125, 139, 77, 101, 98, 61, 146]`; ink px ref/ours 2441/2421 (ratio 0.9918); SSIM blocks <0.9: 134/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-2.11, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1607, differing 0.002287, SSIM₈ 0.998 (raw 0.1607, 0.002287, 0.998)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9968→0.9968 / 0.2569→0.2569; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `This` dx -0.75 dy 0.46; `is` dx -0.66 dy 0.46; `one` dx -0.62 dy 0.46

### 01-plain-paragraph — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933597, 512, 370, 355, 367, 314, 264, 242, 240, 242, 263, 303, 264, 212, 248, 1023]`; ink px ref/ours 2441/2417 (ratio 0.9902); SSIM blocks <0.9: 212/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [4.29, 0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3843, differing 0.003121, SSIM₈ 0.9944 (raw 0.3843, 0.003121, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9911 / 0.6143→0.6143; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 10.49 dy 0.41; `single` dx 9.58 dy 0.41; `a` dx 8.66 dy 0.41

### 01-plain-paragraph — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933595, 460, 437, 342, 360, 330, 346, 325, 305, 282, 301, 246, 245, 198, 207, 837]`; ink px ref/ours 1891/2421 (ratio 1.2803); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-10.52, -0.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3665, differing 0.003027, SSIM₈ 0.9941 (raw 0.3665, 0.003027, 0.9941)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9906→0.9906 / 0.5857→0.5857; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -24.74 dy 0.73; `single` dx -23.83 dy 0.73; `a` dx -22.41 dy 0.73

### 01-plain-paragraph — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933595, 460, 437, 342, 360, 330, 346, 325, 305, 282, 301, 246, 245, 198, 207, 837]`; ink px ref/ours 1891/2421 (ratio 1.2803); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-10.52, -0.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3665, differing 0.003027, SSIM₈ 0.9941 (raw 0.3665, 0.003027, 0.9941)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9906→0.9906 / 0.5857→0.5857; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -24.74 dy 0.73; `single` dx -23.83 dy 0.73; `a` dx -22.41 dy 0.73

### 01-plain-paragraph — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933496, 480, 426, 377, 400, 368, 336, 289, 332, 279, 270, 226, 251, 190, 230, 866]`; ink px ref/ours 1891/2417 (ratio 1.2782); SSIM blocks <0.9: 225/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.11, -0.11] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3715, differing 0.003077, SSIM₈ 0.9942 (raw 0.3715, 0.003077, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9907→0.9907 / 0.5937→0.5937; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -14.35 dy 0.67; `single` dx -14.24 dy 0.67; `a` dx -13.69 dy 0.67

### 02-wrapping-paragraph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831807, 8259, 7034, 6392, 5883, 5835, 5922, 5218, 5432, 5617, 5346, 5301, 5198, 5202, 5188, 25182]`; ink px ref/ours 39630/39464 (ratio 0.9958); SSIM blocks <0.9: 4366/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.86, 4.65] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.3713, differing 0.061277, SSIM₈ 0.8657 (raw 8.3842, 0.061305, 0.8656)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7852→0.7854 / 13.3998→13.3792; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831807, 8259, 7034, 6392, 5883, 5835, 5922, 5218, 5432, 5617, 5346, 5301, 5198, 5202, 5188, 25182]`; ink px ref/ours 39630/39464 (ratio 0.9958); SSIM blocks <0.9: 4366/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.86, 4.65] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.3713, differing 0.061277, SSIM₈ 0.8657 (raw 8.3842, 0.061305, 0.8656)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7852→0.7854 / 13.3998→13.3792; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1843927, 8260, 7006, 6017, 5607, 5496, 5032, 5042, 4898, 4945, 4602, 4773, 4525, 4322, 4364, 20000]`; ink px ref/ours 39630/39427 (ratio 0.9949); SSIM blocks <0.9: 3775/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [4.5, 0.0] pt by ink-projection correlation (centroid estimate [2.05, 2.55] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.1201, differing 0.055007, SSIM₈ 0.8929 (raw 7.1464, 0.055098, 0.8938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8305→0.8314 / 11.4199→11.2602; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 14.85; `branch` dx -435.47 dy 14.85; `over` dx -407.04 dy 14.85

### 02-wrapping-paragraph — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834635, 8254, 7002, 6804, 6347, 6194, 6497, 6173, 6010, 5798, 5303, 5023, 4839, 4475, 4684, 20778]`; ink px ref/ours 30065/39464 (ratio 1.3126); SSIM blocks <0.9: 4465/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -14.5] pt by ink-projection correlation (centroid estimate [-12.29, 0.73] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7691, differing 0.059721, SSIM₈ 0.8602 (raw 7.7868, 0.059701, 0.862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7795→0.7767 / 12.445→12.4168; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -14.75; `oak` dx 426.94 dy -9.07; `jumps` dx 416.92 dy -14.79

### 02-wrapping-paragraph — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834635, 8254, 7002, 6804, 6347, 6194, 6497, 6173, 6010, 5798, 5303, 5023, 4839, 4475, 4684, 20778]`; ink px ref/ours 30065/39464 (ratio 1.3126); SSIM blocks <0.9: 4465/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -14.5] pt by ink-projection correlation (centroid estimate [-12.29, 0.73] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7691, differing 0.059721, SSIM₈ 0.8602 (raw 7.7868, 0.059701, 0.862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7795→0.7767 / 12.445→12.4168; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -14.75; `oak` dx 426.94 dy -9.07; `jumps` dx 416.92 dy -14.79

### 02-wrapping-paragraph — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1845315, 7968, 6863, 6484, 6058, 5706, 5674, 5722, 5520, 5174, 4573, 4477, 4274, 4221, 4009, 16778]`; ink px ref/ours 30065/39427 (ratio 1.3114); SSIM blocks <0.9: 3895/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.38, -1.37] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 6.707, differing 0.054027, SSIM₈ 0.8893 (raw 6.7833, 0.054262, 0.8883)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8218→0.8238 / 10.8396→10.7167; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -14.8; `oak` dx 450.74 dy -14.8; `jumps` dx 438.77 dy -14.8

### 02-wrapping-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831980, 8181, 7016, 6406, 5963, 5948, 5852, 5289, 5188, 5656, 5267, 5378, 5255, 4876, 5221, 25340]`; ink px ref/ours 39390/39464 (ratio 1.0019); SSIM blocks <0.9: 4347/30294; [overlay](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-overlay.png) (55388 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (42501 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 8.3959, not lower; centroid estimate [-7.52, 4.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.3726, differing 0.06114, SSIM₈ 0.8663 (raw 8.3726, 0.06114, 0.8663)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7864→0.7864 / 13.3813→13.3813; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831980, 8181, 7016, 6406, 5963, 5948, 5852, 5289, 5188, 5656, 5267, 5378, 5255, 4876, 5221, 25340]`; ink px ref/ours 39390/39464 (ratio 1.0019); SSIM blocks <0.9: 4347/30294; [overlay](images/02-wrapping-paragraph/pdflatex-main-export-p1-overlay.png) (55317 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-main-export-p1-heatmap.png) (42429 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 8.3959, not lower; centroid estimate [-7.52, 4.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.3726, differing 0.06114, SSIM₈ 0.8663 (raw 8.3726, 0.06114, 0.8663)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7864→0.7864 / 13.3813→13.3813; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1844442, 8261, 7087, 5877, 5663, 5555, 4907, 5038, 4612, 4997, 4565, 4790, 4466, 4235, 4368, 19953]`; ink px ref/ours 39390/39427 (ratio 1.0009); SSIM blocks <0.9: 3750/30294; [overlay](images/02-wrapping-paragraph/pdflatex-pipeline-export-p1-overlay.png) (55504 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-pipeline-export-p1-heatmap.png) (38956 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 7.1751, not lower; centroid estimate [2.39, 2.66] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.1035, differing 0.05485, SSIM₈ 0.8947 (raw 7.1035, 0.05485, 0.8947)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.832→0.832 / 11.3515→11.3515; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 14.85; `branch` dx -435.47 dy 14.85; `over` dx -407.04 dy 14.85

### 02-wrapping-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834636, 8219, 6952, 6893, 6423, 6140, 6491, 6184, 5976, 5757, 5362, 5028, 4837, 4417, 4790, 20711]`; ink px ref/ours 30080/39464 (ratio 1.312); SSIM blocks <0.9: 4465/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (55526 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (42889 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-8.5, -14.5] pt REJECTED: applying it gives mean|Δ| 7.8128, not lower; centroid estimate [-12.18, 0.82] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7862, differing 0.059683, SSIM₈ 0.862 (raw 7.7862, 0.059683, 0.862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7795→0.7795 / 12.4441→12.4441; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.04; `jumps` dx 416.92 dy -13.77

### 02-wrapping-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834636, 8219, 6952, 6893, 6423, 6140, 6491, 6184, 5976, 5757, 5362, 5028, 4837, 4417, 4790, 20711]`; ink px ref/ours 30080/39464 (ratio 1.312); SSIM blocks <0.9: 4465/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-overlay.png) (55452 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (42819 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-8.5, -14.5] pt REJECTED: applying it gives mean|Δ| 7.8128, not lower; centroid estimate [-12.18, 0.82] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7862, differing 0.059683, SSIM₈ 0.862 (raw 7.7862, 0.059683, 0.862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7795→0.7795 / 12.4441→12.4441; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.04; `jumps` dx 416.92 dy -13.77

### 02-wrapping-paragraph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1845342, 7947, 6783, 6447, 6235, 5633, 5703, 5715, 5516, 5076, 4699, 4410, 4319, 4187, 4107, 16697]`; ink px ref/ours 30080/39427 (ratio 1.3107); SSIM blocks <0.9: 3895/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) (55491 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-pipeline-export-p1-heatmap.png) (39257 B, ÷4)
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.27, -1.27] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 6.7047, differing 0.054004, SSIM₈ 0.8893 (raw 6.7841, 0.054238, 0.8883)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8218→0.8239 / 10.8409→10.7129; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -13.78; `oak` dx 450.74 dy -13.78; `jumps` dx 438.77 dy -13.78

### 02-wrapping-paragraph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831948, 8135, 6967, 6429, 5850, 5891, 5942, 5391, 5434, 5606, 5386, 5299, 5280, 5136, 5055, 25067]`; ink px ref/ours 39556/39464 (ratio 0.9977); SSIM blocks <0.9: 4369/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 8.3798, not lower; centroid estimate [-7.66, 4.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.371, differing 0.06128, SSIM₈ 0.8656 (raw 8.371, 0.06128, 0.8656)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7852→0.7852 / 13.3786→13.3786; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 20.54; `branch` dx -435.52 dy 14.86; `over` dx -406.99 dy 20.5

### 02-wrapping-paragraph — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831948, 8135, 6967, 6429, 5850, 5891, 5942, 5391, 5434, 5606, 5386, 5299, 5280, 5136, 5055, 25067]`; ink px ref/ours 39556/39464 (ratio 0.9977); SSIM blocks <0.9: 4369/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 8.3798, not lower; centroid estimate [-7.66, 4.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.371, differing 0.06128, SSIM₈ 0.8656 (raw 8.371, 0.06128, 0.8656)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7852→0.7852 / 13.3786→13.3786; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 20.54; `branch` dx -435.52 dy 14.86; `over` dx -406.99 dy 20.5

### 02-wrapping-paragraph — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1843827, 8131, 7201, 6186, 5753, 5573, 5096, 4902, 4862, 4976, 4621, 4609, 4534, 4358, 4287, 19900]`; ink px ref/ours 39556/39427 (ratio 0.9967); SSIM blocks <0.9: 3769/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 7.2187, not lower; centroid estimate [2.26, 2.66] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.1269, differing 0.055242, SSIM₈ 0.894 (raw 7.1269, 0.055242, 0.894)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8309→0.8309 / 11.3887→11.3887; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 14.85; `branch` dx -435.52 dy 14.85; `over` dx -406.99 dy 14.85

### 02-wrapping-paragraph — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834641, 8258, 6994, 6804, 6326, 6209, 6507, 6167, 6023, 5775, 5321, 5033, 4798, 4503, 4670, 20787]`; ink px ref/ours 30075/39464 (ratio 1.3122); SSIM blocks <0.9: 4467/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -14.5] pt by ink-projection correlation (centroid estimate [-12.45, 0.74] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7691, differing 0.059703, SSIM₈ 0.8602 (raw 7.7866, 0.059691, 0.862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7795→0.7767 / 12.4447→12.4168; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.04; `jumps` dx 416.92 dy -13.77

### 02-wrapping-paragraph — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834641, 8258, 6994, 6804, 6326, 6209, 6507, 6167, 6023, 5775, 5321, 5033, 4798, 4503, 4670, 20787]`; ink px ref/ours 30075/39464 (ratio 1.3122); SSIM blocks <0.9: 4467/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -14.5] pt by ink-projection correlation (centroid estimate [-12.45, 0.74] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7691, differing 0.059703, SSIM₈ 0.8602 (raw 7.7866, 0.059691, 0.862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7795→0.7767 / 12.4447→12.4168; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.04; `jumps` dx 416.92 dy -13.77

### 02-wrapping-paragraph — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1845331, 7973, 6858, 6481, 6021, 5738, 5687, 5711, 5541, 5152, 4584, 4476, 4258, 4218, 4009, 16778]`; ink px ref/ours 30075/39427 (ratio 1.311); SSIM blocks <0.9: 3896/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.54, -1.35] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 6.7066, differing 0.054013, SSIM₈ 0.8893 (raw 6.7827, 0.054251, 0.8883)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8218→0.8238 / 10.8387→10.716; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -13.78; `oak` dx 450.74 dy -13.78; `jumps` dx 438.77 dy -13.78

### 03-section-heading — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923800, 859, 844, 658, 666, 649, 719, 577, 609, 557, 596, 606, 682, 626, 750, 5618]`; ink px ref/ours 5807/4705 (ratio 0.8102); SSIM blocks <0.9: 660/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 52.5] pt by ink-projection correlation (centroid estimate [1.69, 20.98] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1861, differing 0.007719, SSIM₈ 0.9829 (raw 1.3394, 0.008456, 0.9793)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9669→0.9769 / 2.1407→1.6679; header-band 1.0→0.9712 / 0.0→1.5688; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.58 dy 26.34; `second` dx 0.44 dy 26.34; `a` dx 0.41 dy 26.34

### 03-section-heading — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923583, 891, 850, 673, 658, 644, 695, 610, 589, 547, 648, 627, 719, 617, 803, 5662]`; ink px ref/ours 5807/4903 (ratio 0.8443); SSIM blocks <0.9: 671/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 52.5] pt by ink-projection correlation (centroid estimate [5.5, 19.86] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2078, differing 0.007802, SSIM₈ 0.9824 (raw 1.3597, 0.008569, 0.9789)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9663→0.9766 / 2.1731→1.6869; header-band 1.0→0.9684 / 0.0→1.6746; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.58 dy 26.34; `second` dx 0.44 dy 26.34; `a` dx 0.41 dy 26.34
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927451, 775, 776, 635, 567, 608, 628, 507, 566, 477, 529, 506, 515, 477, 598, 3201]`; ink px ref/ours 5807/4755 (ratio 0.8188); SSIM blocks <0.9: 395/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -1.5] pt by ink-projection correlation (centroid estimate [3.0, 3.83] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8907, differing 0.006319, SSIM₈ 0.9891 (raw 0.9265, 0.006439, 0.9882)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9811→0.9826 / 1.4808→1.4236; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 5.31 dy -1.21; `second` dx 4.4 dy -1.21; `a` dx 3.48 dy -1.21

### 03-section-heading — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924467, 959, 778, 633, 646, 661, 738, 747, 647, 638, 596, 597, 643, 582, 684, 4800]`; ink px ref/ours 4775/4705 (ratio 0.9853); SSIM blocks <0.9: 718/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 53.5] pt by ink-projection correlation (centroid estimate [-2.67, 22.65] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0886, differing 0.007454, SSIM₈ 0.9817 (raw 1.2336, 0.008176, 0.9771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9634→0.9751 / 1.9716→1.5009; header-band 1.0→0.9699 / 0.0→1.6444; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.94 dy 27.15; `second` dx -11.49 dy 27.15; `a` dx -10.06 dy 27.15

### 03-section-heading — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924345, 994, 761, 661, 641, 673, 731, 760, 640, 631, 625, 592, 654, 570, 731, 4807]`; ink px ref/ours 4775/4903 (ratio 1.0268); SSIM blocks <0.9: 729/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 53.5] pt by ink-projection correlation (centroid estimate [1.14, 21.53] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1122, differing 0.00756, SSIM₈ 0.9812 (raw 1.2425, 0.008247, 0.9769)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9631→0.9747 / 1.9859→1.5233; header-band 1.0→0.9672 / 0.0→1.7502; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.94 dy 27.15; `second` dx -11.49 dy 27.15; `a` dx -10.06 dy 27.15
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927973, 841, 643, 692, 588, 637, 618, 647, 534, 517, 427, 451, 472, 443, 518, 2815]`; ink px ref/ours 4775/4755 (ratio 0.9958); SSIM blocks <0.9: 430/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.36, 5.51] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8053, differing 0.00602, SSIM₈ 0.9889 (raw 0.8557, 0.006238, 0.9881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9809→0.9822 / 1.3676→1.2872; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -8.21 dy -0.4; `second` dx -7.53 dy -0.4; `a` dx -6.99 dy -0.4

### 03-section-heading — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923855, 915, 733, 647, 632, 615, 615, 519, 606, 614, 675, 690, 668, 592, 707, 5733]`; ink px ref/ours 6093/4705 (ratio 0.7722); SSIM blocks <0.9: 662/30294; [overlay](images/03-section-heading/pdflatex-de1020c-export-p1-overlay.png) (41934 B, ÷2), [heatmap](images/03-section-heading/pdflatex-de1020c-export-p1-heatmap.png) (43789 B, ÷2)
  - registration error (diagnostic): global shift [0.5, 52.5] pt by ink-projection correlation (centroid estimate [1.59, 20.53] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2021, differing 0.007752, SSIM₈ 0.9827 (raw 1.3496, 0.008486, 0.9793)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9669→0.9767 / 2.157→1.6932; header-band 1.0→0.9709 / 0.0→1.5688; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.75 dy 26.08; `second` dx 0.61 dy 26.08; `a` dx 0.58 dy 26.08

### 03-section-heading — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923616, 943, 723, 653, 638, 627, 618, 535, 600, 612, 715, 698, 709, 583, 762, 5784]`; ink px ref/ours 6093/4903 (ratio 0.8047); SSIM blocks <0.9: 673/30294; [overlay](images/03-section-heading/pdflatex-main-export-p1-overlay.png) (41803 B, ÷2), [heatmap](images/03-section-heading/pdflatex-main-export-p1-heatmap.png) (43754 B, ÷2)
  - registration error (diagnostic): global shift [0.5, 52.5] pt by ink-projection correlation (centroid estimate [5.4, 19.4] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2209, differing 0.007844, SSIM₈ 0.9823 (raw 1.3719, 0.008606, 0.9789)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9663→0.9765 / 2.1927→1.7078; header-band 1.0→0.9684 / 0.0→1.6746; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.75 dy 26.08; `second` dx 0.61 dy 26.08; `a` dx 0.58 dy 26.08
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927351, 776, 613, 641, 575, 628, 555, 508, 557, 494, 569, 558, 537, 506, 578, 3370]`; ink px ref/ours 6093/4755 (ratio 0.7804); SSIM blocks <0.9: 401/30294; [overlay](images/03-section-heading/pdflatex-pipeline-export-p1-overlay.png) (41473 B, ÷2), [heatmap](images/03-section-heading/pdflatex-pipeline-export-p1-heatmap.png) (88679 B, ÷1)
  - registration error (diagnostic): global shift [0.0, -2.0] pt by ink-projection correlation (centroid estimate [2.9, 3.38] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8932, differing 0.006353, SSIM₈ 0.9877 (raw 0.9558, 0.006556, 0.9878)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9805→0.9804 / 1.5277→1.4275; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 5.48 dy -1.48; `second` dx 4.57 dy -1.48; `a` dx 3.65 dy -1.48

### 03-section-heading — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924475, 952, 791, 621, 670, 652, 705, 745, 643, 649, 621, 599, 586, 629, 701, 4777]`; ink px ref/ours 4791/4705 (ratio 0.982); SSIM blocks <0.9: 718/30294; [overlay](images/03-section-heading/pdflatex-lm-de1020c-export-p1-overlay.png) (42178 B, ÷2), [heatmap](images/03-section-heading/pdflatex-lm-de1020c-export-p1-heatmap.png) (44189 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 54.0] pt by ink-projection correlation (centroid estimate [-2.62, 22.29] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0954, differing 0.00748, SSIM₈ 0.9811 (raw 1.2335, 0.008161, 0.9771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9634→0.975 / 1.9715→1.4903; header-band 1.0→0.9643 / 0.0→1.793; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.95 dy 28.22; `second` dx -11.5 dy 28.22; `a` dx -10.07 dy 28.22

### 03-section-heading — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924351, 973, 792, 646, 666, 667, 692, 761, 638, 638, 652, 591, 594, 611, 758, 4786]`; ink px ref/ours 4791/4903 (ratio 1.0234); SSIM blocks <0.9: 729/30294; [overlay](images/03-section-heading/pdflatex-lm-main-export-p1-overlay.png) (42121 B, ÷2), [heatmap](images/03-section-heading/pdflatex-lm-main-export-p1-heatmap.png) (44122 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 54.0] pt by ink-projection correlation (centroid estimate [1.18, 21.17] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1201, differing 0.007589, SSIM₈ 0.9807 (raw 1.2428, 0.008232, 0.9769)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9631→0.9747 / 1.9863→1.5143; header-band 1.0→0.9618 / 0.0→1.8988; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.95 dy 28.22; `second` dx -11.5 dy 28.22; `a` dx -10.07 dy 28.22
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927957, 857, 652, 684, 602, 644, 603, 616, 554, 515, 442, 454, 447, 462, 516, 2811]`; ink px ref/ours 4791/4755 (ratio 0.9925); SSIM blocks <0.9: 430/30294; [overlay](images/03-section-heading/pdflatex-lm-pipeline-export-p1-overlay.png) (41891 B, ÷2), [heatmap](images/03-section-heading/pdflatex-lm-pipeline-export-p1-heatmap.png) (89454 B, ÷1)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.31, 5.14] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8056, differing 0.006022, SSIM₈ 0.9889 (raw 0.8549, 0.006211, 0.9881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9809→0.9822 / 1.3664→1.2875; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -8.22 dy 0.67; `second` dx -7.54 dy 0.67; `a` dx -7.0 dy 0.67

### 03-section-heading — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923747, 880, 847, 676, 683, 663, 736, 610, 596, 553, 616, 586, 654, 628, 733, 5608]`; ink px ref/ours 5744/4705 (ratio 0.8191); SSIM blocks <0.9: 660/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 52.5] pt by ink-projection correlation (centroid estimate [1.75, 20.6] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1868, differing 0.007742, SSIM₈ 0.9829 (raw 1.337, 0.008469, 0.9793)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9669→0.9769 / 2.137→1.669; header-band 1.0→0.9712 / 0.0→1.5688; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.57 dy 26.34; `second` dx 0.43 dy 26.34; `a` dx 0.4 dy 26.34

### 03-section-heading — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923509, 902, 846, 685, 678, 671, 724, 642, 583, 549, 658, 607, 691, 618, 792, 5661]`; ink px ref/ours 5744/4903 (ratio 0.8536); SSIM blocks <0.9: 671/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 52.5] pt by ink-projection correlation (centroid estimate [5.55, 19.47] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2088, differing 0.007836, SSIM₈ 0.9824 (raw 1.36, 0.008591, 0.9789)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9663→0.9766 / 2.1737→1.6885; header-band 1.0→0.9684 / 0.0→1.6746; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.57 dy 26.34; `second` dx 0.43 dy 26.34; `a` dx 0.4 dy 26.34
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927376, 774, 786, 619, 569, 664, 627, 539, 582, 502, 533, 485, 527, 455, 588, 3190]`; ink px ref/ours 5744/4755 (ratio 0.8278); SSIM blocks <0.9: 395/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -1.5] pt by ink-projection correlation (centroid estimate [3.06, 3.45] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8892, differing 0.006328, SSIM₈ 0.9891 (raw 0.9285, 0.006472, 0.9882)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9811→0.9826 / 1.484→1.4211; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 5.3 dy -1.21; `second` dx 4.39 dy -1.21; `a` dx 3.47 dy -1.21

### 03-section-heading — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924466, 954, 785, 634, 640, 664, 732, 752, 647, 641, 594, 601, 641, 581, 686, 4798]`; ink px ref/ours 4777/4705 (ratio 0.9849); SSIM blocks <0.9: 718/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 53.5] pt by ink-projection correlation (centroid estimate [-2.67, 22.65] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0886, differing 0.007455, SSIM₈ 0.9817 (raw 1.2337, 0.008177, 0.9771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9634→0.9751 / 1.9717→1.501; header-band 1.0→0.9699 / 0.0→1.6444; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.94 dy 28.18; `second` dx -11.49 dy 28.18; `a` dx -10.06 dy 28.18

### 03-section-heading — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924345, 998, 760, 662, 635, 674, 730, 762, 639, 633, 622, 599, 649, 577, 726, 4805]`; ink px ref/ours 4777/4903 (ratio 1.0264); SSIM blocks <0.9: 729/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 53.5] pt by ink-projection correlation (centroid estimate [1.14, 21.53] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1122, differing 0.00756, SSIM₈ 0.9812 (raw 1.2424, 0.008248, 0.9769)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9631→0.9747 / 1.9858→1.5233; header-band 1.0→0.9672 / 0.0→1.7502; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.94 dy 28.18; `second` dx -11.49 dy 28.18; `a` dx -10.06 dy 28.18
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927972, 838, 648, 694, 580, 638, 614, 652, 532, 519, 427, 460, 465, 443, 520, 2814]`; ink px ref/ours 4777/4755 (ratio 0.9954); SSIM blocks <0.9: 430/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.36, 5.51] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8052, differing 0.00602, SSIM₈ 0.9889 (raw 0.8558, 0.006238, 0.9881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9809→0.9822 / 1.3677→1.287; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -8.21 dy 0.63; `second` dx -7.53 dy 0.63; `a` dx -6.99 dy 0.63

### 04-bold-emph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933574, 491, 483, 352, 286, 278, 321, 275, 308, 283, 235, 240, 216, 217, 213, 1044]`; ink px ref/ours 2712/2465 (ratio 0.9089); SSIM blocks <0.9: 227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3809, not lower; centroid estimate [1.29, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3808, differing 0.003147, SSIM₈ 0.9945 (raw 0.3808, 0.003147, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9912→0.9912 / 0.6087→0.6087; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.28 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933574, 491, 483, 352, 286, 278, 321, 275, 308, 283, 235, 240, 216, 217, 213, 1044]`; ink px ref/ours 2712/2465 (ratio 0.9089); SSIM blocks <0.9: 227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3809, not lower; centroid estimate [1.29, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3808, differing 0.003147, SSIM₈ 0.9945 (raw 0.3808, 0.003147, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9912→0.9912 / 0.6087→0.6087; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.28 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-italic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman10-bolditalic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933466, 532, 395, 345, 290, 332, 294, 275, 291, 269, 232, 258, 289, 238, 260, 1050]`; ink px ref/ours 2712/2434 (ratio 0.8975); SSIM blocks <0.9: 237/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [6.0, 0.0] pt by ink-projection correlation (centroid estimate [2.4, 0.01] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.393, differing 0.00317, SSIM₈ 0.9945 (raw 0.3955, 0.003173, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9909→0.9915 / 0.6321→0.6138; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 8.87 dy 0.41; `one` dx 7.97 dy 0.41; `on` dx 7.06 dy 0.41

### 04-bold-emph — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933135, 462, 395, 357, 369, 349, 325, 343, 352, 298, 238, 265, 257, 274, 230, 1167]`; ink px ref/ours 2276/2465 (ratio 1.083); SSIM blocks <0.9: 258/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-17.7, 0.03] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4148, differing 0.003328, SSIM₈ 0.9934 (raw 0.4244, 0.00332, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6784→0.663; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -27.99 dy -0.63; `one` dx -26.86 dy -0.63; `on` dx -25.6 dy -0.63
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933135, 462, 395, 357, 369, 349, 325, 343, 352, 298, 238, 265, 257, 274, 230, 1167]`; ink px ref/ours 2276/2465 (ratio 1.083); SSIM blocks <0.9: 258/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-17.7, 0.03] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4148, differing 0.003328, SSIM₈ 0.9934 (raw 0.4244, 0.00332, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6784→0.663; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -27.99 dy -0.63; `one` dx -26.86 dy -0.63; `on` dx -25.6 dy -0.63
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-italic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman10-bolditalic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933046, 459, 409, 410, 329, 363, 328, 348, 326, 288, 283, 269, 274, 251, 310, 1123]`; ink px ref/ours 2276/2434 (ratio 1.0694); SSIM blocks <0.9: 252/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-35.0, 0.0] pt by ink-projection correlation (centroid estimate [-16.59, 0.03] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.421, differing 0.003313, SSIM₈ 0.9934 (raw 0.4305, 0.003346, 0.9934)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9894→0.9895 / 0.6881→0.673; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -22.87 dy -0.68; `one` dx -22.58 dy -0.68; `on` dx -22.17 dy -0.68

### 04-bold-emph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933598, 513, 444, 319, 342, 300, 270, 233, 301, 263, 246, 223, 207, 251, 234, 1072]`; ink px ref/ours 2746/2465 (ratio 0.8977); SSIM blocks <0.9: 225/30294; [overlay](images/04-bold-emph/pdflatex-de1020c-export-p1-overlay.png) (79089 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-de1020c-export-p1-heatmap.png) (73373 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [3.52, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3764, differing 0.003158, SSIM₈ 0.9945 (raw 0.3836, 0.003143, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9912 / 0.6131→0.6017; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.27 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933598, 513, 444, 319, 342, 300, 270, 233, 301, 263, 246, 223, 207, 251, 234, 1072]`; ink px ref/ours 2746/2465 (ratio 0.8977); SSIM blocks <0.9: 225/30294; [overlay](images/04-bold-emph/pdflatex-main-export-p1-overlay.png) (78620 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-main-export-p1-heatmap.png) (72889 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [3.52, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3764, differing 0.003158, SSIM₈ 0.9945 (raw 0.3836, 0.003143, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9912 / 0.6131→0.6017; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.27 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-italic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman10-bolditalic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933598, 475, 372, 347, 306, 341, 264, 268, 264, 262, 282, 259, 277, 227, 257, 1017]`; ink px ref/ours 2746/2434 (ratio 0.8864); SSIM blocks <0.9: 233/30294; [overlay](images/04-bold-emph/pdflatex-pipeline-export-p1-overlay.png) (79105 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-pipeline-export-p1-heatmap.png) (73392 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [4.63, 0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3885, differing 0.003158, SSIM₈ 0.9944 (raw 0.3885, 0.003158, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9911 / 0.6209→0.6209; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 9.44 dy 0.41; `one` dx 8.53 dy 0.41; `on` dx 7.62 dy 0.41

### 04-bold-emph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933105, 492, 380, 370, 376, 348, 330, 343, 330, 319, 248, 266, 243, 259, 233, 1174]`; ink px ref/ours 2268/2465 (ratio 1.0869); SSIM blocks <0.9: 259/30294; [overlay](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-overlay.png) (79764 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-heatmap.png) (74597 B, ÷1)
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-17.16, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4162, differing 0.003327, SSIM₈ 0.9934 (raw 0.4246, 0.003322, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6786→0.6653; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -28.1 dy 0.73; `one` dx -26.97 dy 0.73; `on` dx -25.72 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933105, 492, 380, 370, 376, 348, 330, 343, 330, 319, 248, 266, 243, 259, 233, 1174]`; ink px ref/ours 2268/2465 (ratio 1.0869); SSIM blocks <0.9: 259/30294; [overlay](images/04-bold-emph/pdflatex-lm-main-export-p1-overlay.png) (79524 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-main-export-p1-heatmap.png) (74351 B, ÷1)
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-17.16, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4162, differing 0.003327, SSIM₈ 0.9934 (raw 0.4246, 0.003322, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6786→0.6653; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -28.1 dy 0.73; `one` dx -26.97 dy 0.73; `on` dx -25.72 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-italic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman10-bolditalic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933045, 478, 409, 396, 339, 345, 323, 331, 318, 303, 295, 270, 268, 253, 312, 1131]`; ink px ref/ours 2268/2434 (ratio 1.0732); SSIM blocks <0.9: 253/30294; [overlay](images/04-bold-emph/pdflatex-lm-pipeline-export-p1-overlay.png) (79751 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-pipeline-export-p1-heatmap.png) (76908 B, ÷1)
  - registration error (diagnostic): global shift [-35.0, 0.0] pt by ink-projection correlation (centroid estimate [-16.05, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.422, differing 0.003304, SSIM₈ 0.9934 (raw 0.4313, 0.003348, 0.9934)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9894→0.9895 / 0.6893→0.6745; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -22.98 dy 0.67; `one` dx -22.69 dy 0.67; `on` dx -22.29 dy 0.67

### 04-bold-emph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933519, 519, 462, 366, 293, 287, 321, 265, 298, 295, 232, 216, 218, 229, 229, 1067]`; ink px ref/ours 2702/2465 (ratio 0.9123); SSIM blocks <0.9: 227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [0.77, 0.0] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3834, differing 0.003146, SSIM₈ 0.9943 (raw 0.3848, 0.003146, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.991 / 0.6151→0.6128; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.26 dy 0.46; `bold` dx 5.16 dy 0.46; `and` dx 5.12 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933519, 519, 462, 366, 293, 287, 321, 265, 298, 295, 232, 216, 218, 229, 229, 1067]`; ink px ref/ours 2702/2465 (ratio 0.9123); SSIM blocks <0.9: 227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [0.77, 0.0] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3834, differing 0.003146, SSIM₈ 0.9943 (raw 0.3848, 0.003146, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.991 / 0.6151→0.6128; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.26 dy 0.46; `bold` dx 5.16 dy 0.46; `and` dx 5.12 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-italic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman10-bolditalic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933451, 538, 382, 375, 272, 332, 298, 284, 278, 277, 246, 239, 283, 242, 253, 1066]`; ink px ref/ours 2702/2434 (ratio 0.9008); SSIM blocks <0.9: 237/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [3.0, 0.0] pt by ink-projection correlation (centroid estimate [1.88, 0.0] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3946, differing 0.003155, SSIM₈ 0.9945 (raw 0.3962, 0.003176, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9909→0.9914 / 0.6333→0.6213; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 8.96 dy 0.41; `one` dx 8.05 dy 0.41; `on` dx 7.14 dy 0.41

### 04-bold-emph — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933144, 455, 411, 335, 384, 345, 332, 339, 340, 281, 249, 264, 266, 261, 226, 1184]`; ink px ref/ours 2260/2465 (ratio 1.0907); SSIM blocks <0.9: 255/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-16.16, 0.03] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4138, differing 0.003314, SSIM₈ 0.9934 (raw 0.4245, 0.003312, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6784→0.6613; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -27.75 dy 0.73; `one` dx -26.62 dy 0.73; `on` dx -25.36 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933144, 455, 411, 335, 384, 345, 332, 339, 340, 281, 249, 264, 266, 261, 226, 1184]`; ink px ref/ours 2260/2465 (ratio 1.0907); SSIM blocks <0.9: 255/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-16.16, 0.03] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4138, differing 0.003314, SSIM₈ 0.9934 (raw 0.4245, 0.003312, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6784→0.6613; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -27.75 dy 0.73; `one` dx -26.62 dy 0.73; `on` dx -25.36 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-italic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman10-bolditalic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933067, 459, 424, 395, 350, 355, 352, 352, 316, 270, 286, 277, 281, 231, 292, 1109]`; ink px ref/ours 2260/2434 (ratio 1.077); SSIM blocks <0.9: 250/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-35.0, 0.0] pt by ink-projection correlation (centroid estimate [-15.05, 0.02] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4192, differing 0.003311, SSIM₈ 0.9934 (raw 0.4257, 0.00333, 0.9935)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9895→0.9895 / 0.6804→0.67; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -22.63 dy 0.67; `one` dx -22.34 dy 0.67; `on` dx -21.94 dy 0.67

### 05-unicode — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934471, 531, 434, 351, 255, 281, 235, 186, 150, 219, 214, 199, 193, 149, 197, 751]`; ink px ref/ours 2129/2123 (ratio 0.9972); SSIM blocks <0.9: 193/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [5.16, -0.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2971, differing 0.002695, SSIM₈ 0.9955 (raw 0.2971, 0.002695, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9928→0.9928 / 0.4749→0.4749; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 30.41 dy 0.46; `also` dx -5.46 dy 0.46; `dash;` dx -4.07 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '—' (U+2014) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934471, 531, 434, 351, 255, 281, 235, 186, 150, 219, 214, 199, 193, 149, 197, 751]`; ink px ref/ours 2129/2123 (ratio 0.9972); SSIM blocks <0.9: 193/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [5.16, -0.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2971, differing 0.002695, SSIM₈ 0.9955 (raw 0.2971, 0.002695, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9928→0.9928 / 0.4749→0.4749; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 30.41 dy 0.46; `also` dx -5.46 dy 0.46; `dash;` dx -4.07 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934259, 541, 339, 333, 231, 273, 246, 180, 207, 210, 225, 226, 213, 255, 240, 838]`; ink px ref/ours 2129/2093 (ratio 0.9831); SSIM blocks <0.9: 192/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.34, not lower; centroid estimate [5.01, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3307, differing 0.002774, SSIM₈ 0.9953 (raw 0.3307, 0.002774, 0.9953)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9924→0.9924 / 0.5285→0.5285; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 8.22 dy 0.41; `—` dx 7.3 dy 0.41; `Résumé` dx 6.39 dy 0.41

### 05-unicode — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933784, 466, 389, 318, 316, 350, 300, 311, 265, 275, 232, 237, 245, 200, 242, 886]`; ink px ref/ours 1547/2123 (ratio 1.3723); SSIM blocks <0.9: 237/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [2.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.87, -0.15] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3494, differing 0.002871, SSIM₈ 0.994 (raw 0.3619, 0.002925, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.9905 / 0.5784→0.5529; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 16.19 dy -0.3; `also` dx -13.35 dy -0.3; `dash;` dx -9.55 dy -0.3
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '—' (U+2014) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933784, 466, 389, 318, 316, 350, 300, 311, 265, 275, 232, 237, 245, 200, 242, 886]`; ink px ref/ours 1547/2123 (ratio 1.3723); SSIM blocks <0.9: 237/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [2.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.87, -0.15] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3494, differing 0.002871, SSIM₈ 0.994 (raw 0.3619, 0.002925, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.9905 / 0.5784→0.5529; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 16.19 dy -0.3; `also` dx -13.35 dy -0.3; `dash;` dx -9.55 dy -0.3
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933929, 478, 336, 302, 253, 344, 343, 292, 248, 285, 250, 191, 204, 213, 259, 889]`; ink px ref/ours 1547/2093 (ratio 1.3529); SSIM blocks <0.9: 208/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-5.5, 0.0] pt by ink-projection correlation (centroid estimate [-4.01, -0.11] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3116, differing 0.002677, SSIM₈ 0.9949 (raw 0.3558, 0.002858, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9908→0.9919 / 0.5687→0.498; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `—` dx -6.26 dy -0.36; `dash.` dx -6.0 dy -0.36; `café` dx -4.96 dy -0.36

### 05-unicode — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934287, 570, 367, 345, 277, 258, 239, 232, 233, 241, 245, 185, 188, 173, 231, 745]`; ink px ref/ours 2119/2123 (ratio 1.0019); SSIM blocks <0.9: 209/30294; [overlay](images/05-unicode/pdflatex-de1020c-export-p1-overlay.png) (78341 B, ÷1), [heatmap](images/05-unicode/pdflatex-de1020c-export-p1-heatmap.png) (75180 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-5.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.331, not lower; centroid estimate [3.28, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3127, differing 0.002744, SSIM₈ 0.9955 (raw 0.3127, 0.002744, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9928→0.9928 / 0.4998→0.4998; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 30.73 dy 0.46; `also` dx -5.31 dy 0.46; `dash;` dx -3.91 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '—' (U+2014) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934287, 570, 367, 345, 277, 258, 239, 232, 233, 241, 245, 185, 188, 173, 231, 745]`; ink px ref/ours 2119/2123 (ratio 1.0019); SSIM blocks <0.9: 209/30294; [overlay](images/05-unicode/pdflatex-main-export-p1-overlay.png) (77861 B, ÷1), [heatmap](images/05-unicode/pdflatex-main-export-p1-heatmap.png) (74693 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-5.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.331, not lower; centroid estimate [3.28, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3127, differing 0.002744, SSIM₈ 0.9955 (raw 0.3127, 0.002744, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9928→0.9928 / 0.4998→0.4998; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 30.73 dy 0.46; `also` dx -5.31 dy 0.46; `dash;` dx -3.91 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934238, 493, 320, 279, 237, 273, 249, 215, 234, 245, 221, 220, 196, 220, 243, 933]`; ink px ref/ours 2119/2093 (ratio 0.9877); SSIM blocks <0.9: 186/30294; [overlay](images/05-unicode/pdflatex-pipeline-export-p1-overlay.png) (77737 B, ÷1), [heatmap](images/05-unicode/pdflatex-pipeline-export-p1-heatmap.png) (74838 B, ÷1)
  - registration error (diagnostic): global shift [3.5, 0.0] pt by ink-projection correlation (centroid estimate [3.13, 0.03] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3331, differing 0.002752, SSIM₈ 0.9956 (raw 0.3413, 0.002778, 0.9954)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9926→0.993 / 0.5454→0.5264; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 8.53 dy 0.41; `—` dx 7.62 dy 0.41; `Résumé` dx 6.71 dy 0.41

### 05-unicode — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933782, 470, 394, 304, 327, 351, 291, 322, 255, 269, 237, 246, 249, 203, 219, 897]`; ink px ref/ours 1537/2123 (ratio 1.3813); SSIM blocks <0.9: 237/30294; [overlay](images/05-unicode/pdflatex-lm-de1020c-export-p1-overlay.png) (78100 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-de1020c-export-p1-heatmap.png) (75826 B, ÷1)
  - registration error (diagnostic): global shift [2.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.9, -0.14] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3496, differing 0.00287, SSIM₈ 0.994 (raw 0.3617, 0.002921, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.9905 / 0.5782→0.5533; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 16.16 dy 0.73; `also` dx -13.37 dy 0.73; `dash;` dx -9.57 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '—' (U+2014) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933782, 470, 394, 304, 327, 351, 291, 322, 255, 269, 237, 246, 249, 203, 219, 897]`; ink px ref/ours 1537/2123 (ratio 1.3813); SSIM blocks <0.9: 237/30294; [overlay](images/05-unicode/pdflatex-lm-main-export-p1-overlay.png) (77679 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-main-export-p1-heatmap.png) (75399 B, ÷1)
  - registration error (diagnostic): global shift [2.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.9, -0.14] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3496, differing 0.00287, SSIM₈ 0.994 (raw 0.3617, 0.002921, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.9905 / 0.5782→0.5533; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 16.16 dy 0.73; `also` dx -13.37 dy 0.73; `dash;` dx -9.57 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933917, 479, 352, 304, 246, 357, 324, 295, 259, 279, 241, 197, 195, 236, 223, 912]`; ink px ref/ours 1537/2093 (ratio 1.3617); SSIM blocks <0.9: 208/30294; [overlay](images/05-unicode/pdflatex-lm-pipeline-export-p1-overlay.png) (78407 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-pipeline-export-p1-heatmap.png) (75299 B, ÷1)
  - registration error (diagnostic): global shift [-5.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.05, -0.11] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3117, differing 0.00267, SSIM₈ 0.9949 (raw 0.3558, 0.002855, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9908→0.9919 / 0.5686→0.4982; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `—` dx -6.29 dy 0.67; `dash.` dx -6.03 dy 0.67; `café` dx -4.98 dy 0.67

### 05-unicode — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934491, 509, 440, 374, 248, 251, 251, 167, 176, 216, 207, 211, 173, 149, 193, 760]`; ink px ref/ours 2139/2123 (ratio 0.9925); SSIM blocks <0.9: 190/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [3.62, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2963, differing 0.002685, SSIM₈ 0.9955 (raw 0.2963, 0.002685, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9928→0.9928 / 0.4736→0.4736; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 30.37 dy 0.46; `also` dx -5.49 dy 0.46; `dash;` dx -4.09 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '—' (U+2014) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934491, 509, 440, 374, 248, 251, 251, 167, 176, 216, 207, 211, 173, 149, 193, 760]`; ink px ref/ours 2139/2123 (ratio 0.9925); SSIM blocks <0.9: 190/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [3.62, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2963, differing 0.002685, SSIM₈ 0.9955 (raw 0.2963, 0.002685, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9928→0.9928 / 0.4736→0.4736; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 30.37 dy 0.46; `also` dx -5.49 dy 0.46; `dash;` dx -4.09 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934271, 512, 348, 350, 225, 256, 257, 203, 206, 184, 251, 227, 203, 236, 244, 843]`; ink px ref/ours 2139/2093 (ratio 0.9785); SSIM blocks <0.9: 192/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3389, not lower; centroid estimate [3.47, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.33, differing 0.002752, SSIM₈ 0.9953 (raw 0.33, 0.002752, 0.9953)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9924→0.9924 / 0.5275→0.5275; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 8.18 dy 0.41; `—` dx 7.26 dy 0.41; `Résumé` dx 6.35 dy 0.41

### 05-unicode — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933783, 466, 389, 315, 321, 350, 297, 313, 265, 274, 231, 236, 249, 198, 242, 887]`; ink px ref/ours 1542/2123 (ratio 1.3768); SSIM blocks <0.9: 237/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [2.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.87, -0.13] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3493, differing 0.002871, SSIM₈ 0.994 (raw 0.362, 0.002927, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.9905 / 0.5785→0.5529; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 16.19 dy 0.73; `also` dx -13.35 dy 0.73; `dash;` dx -9.55 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '—' (U+2014) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933783, 466, 389, 315, 321, 350, 297, 313, 265, 274, 231, 236, 249, 198, 242, 887]`; ink px ref/ours 1542/2123 (ratio 1.3768); SSIM blocks <0.9: 237/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [2.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.87, -0.13] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3493, differing 0.002871, SSIM₈ 0.994 (raw 0.362, 0.002927, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.9905 / 0.5785→0.5529; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 16.19 dy 0.73; `also` dx -13.35 dy 0.73; `dash;` dx -9.55 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933930, 474, 339, 300, 253, 347, 340, 292, 249, 287, 248, 190, 208, 211, 259, 889]`; ink px ref/ours 1542/2093 (ratio 1.3573); SSIM blocks <0.9: 208/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-5.5, 0.0] pt by ink-projection correlation (centroid estimate [-4.01, -0.1] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3115, differing 0.002676, SSIM₈ 0.9949 (raw 0.3558, 0.002859, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9908→0.9919 / 0.5687→0.4978; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `—` dx -6.26 dy 0.67; `dash.` dx -6.0 dy 0.67; `café` dx -4.97 dy 0.67

### 06-math-inline — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933844, 435, 352, 317, 197, 257, 297, 287, 248, 200, 246, 306, 299, 245, 218, 1068]`; ink px ref/ours 1729/1833 (ratio 1.0602); SSIM blocks <0.9: 216/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-5.21, 3.5] pt); confidence strong (shift explains 33% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2554, differing 0.002365, SSIM₈ 0.9959 (raw 0.3832, 0.00292, 0.9934)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9895→0.9934 / 0.6124→0.4082; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5653→0.68 / 21.4908→17.5131 [131.7,62.4–343.8,93.6 pt]
- largest word displacements (pt): `inside` dx -8.16 dy 1.32; `a` dx -8.04 dy 1.32; `sentence` dx -8.01 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933844, 435, 352, 317, 197, 257, 297, 287, 248, 200, 246, 306, 299, 245, 218, 1068]`; ink px ref/ours 1729/1833 (ratio 1.0602); SSIM blocks <0.9: 216/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-5.21, 3.5] pt); confidence strong (shift explains 33% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2554, differing 0.002365, SSIM₈ 0.9959 (raw 0.3832, 0.00292, 0.9934)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9895→0.9934 / 0.6124→0.4082; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5653→0.68 / 21.4908→17.5131 [131.7,62.4–343.8,93.6 pt]
- largest word displacements (pt): `inside` dx -8.16 dy 1.32; `a` dx -8.04 dy 1.32; `sentence` dx -8.01 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935216, 397, 327, 263, 210, 203, 217, 213, 134, 167, 145, 147, 190, 161, 175, 651]`; ink px ref/ours 1729/1510 (ratio 0.8733); SSIM blocks <0.9: 168/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [-25.68, 0.01] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.254, differing 0.002253, SSIM₈ 0.9957 (raw 0.255, 0.002229, 0.9956)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.993→0.9932 / 0.4076→0.4058; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6989→0.7035 / 15.295→14.8322 [131.7,62.4–343.8,93.6 pt]
- largest word displacements (pt): `inside` dx -48.22 dy -2.68; `a` dx -47.3 dy -2.68; `sentence` dx -46.38 dy -2.68
- word-sequence differences: delete ref ['a', 'b'] ours []; replace ref ['α', '+', 'β,'] ours [',']; delete ref ['√x'] ours []

### 06-math-inline — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933869, 425, 339, 269, 252, 319, 324, 366, 281, 210, 256, 307, 229, 233, 196, 941]`; ink px ref/ours 1358/1833 (ratio 1.3498); SSIM blocks <0.9: 226/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-32.5, 3.5] pt by ink-projection correlation (centroid estimate [-8.35, 3.43] pt); confidence moderate (shift explains 15% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3134, differing 0.002591, SSIM₈ 0.9945 (raw 0.3682, 0.002867, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9912 / 0.5885→0.5009; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5865→0.6624 / 18.2797→17.0519 [137.6,62.4–364.2,93.6 pt]
- largest word displacements (pt): `text.` dx -25.34 dy 1.32; `of` dx -25.01 dy 1.32; `sentence` dx -22.78 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933869, 425, 339, 269, 252, 319, 324, 366, 281, 210, 256, 307, 229, 233, 196, 941]`; ink px ref/ours 1358/1833 (ratio 1.3498); SSIM blocks <0.9: 226/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-32.5, 3.5] pt by ink-projection correlation (centroid estimate [-8.35, 3.43] pt); confidence moderate (shift explains 15% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3134, differing 0.002591, SSIM₈ 0.9945 (raw 0.3682, 0.002867, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9912 / 0.5885→0.5009; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5865→0.6624 / 18.2797→17.0519 [137.6,62.4–364.2,93.6 pt]
- largest word displacements (pt): `text.` dx -25.34 dy 1.32; `of` dx -25.01 dy 1.32; `sentence` dx -22.78 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934893, 356, 287, 237, 223, 288, 259, 271, 215, 215, 173, 184, 213, 173, 205, 624]`; ink px ref/ours 1358/1510 (ratio 1.1119); SSIM blocks <0.9: 191/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-4.0, 0.0] pt by ink-projection correlation (centroid estimate [-28.82, -0.06] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2728, differing 0.002274, SSIM₈ 0.9951 (raw 0.2818, 0.002306, 0.9949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9919→0.9922 / 0.4504→0.4361; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6877→0.6802 / 13.9194→14.8014 [137.6,62.4–364.2,93.6 pt]
- largest word displacements (pt): `of` dx -62.64 dy -2.68; `text.` dx -62.11 dy -2.68; `sentence` dx -61.15 dy -2.68
- word-sequence differences: delete ref ['a', 'b'] ours []; replace ref ['α', '+', 'β,'] ours [',']; delete ref ['√x'] ours []

### 06-math-inline — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933877, 403, 304, 272, 266, 302, 299, 308, 222, 194, 272, 315, 237, 246, 231, 1068]`; ink px ref/ours 1699/1833 (ratio 1.0789); SSIM blocks <0.9: 211/30294; [overlay](images/06-math-inline/pdflatex-de1020c-export-p1-overlay.png) (76140 B, ÷1), [heatmap](images/06-math-inline/pdflatex-de1020c-export-p1-heatmap.png) (75883 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-4.61, 3.51] pt); confidence strong (shift explains 32% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2595, differing 0.002359, SSIM₈ 0.996 (raw 0.3821, 0.002886, 0.9936)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9898→0.9937 / 0.6107→0.4148; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5648→0.6749 / 21.4145→17.7703 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `a` dx -7.78 dy 1.32; `sentence` dx -7.76 dy 1.32; `of` dx -7.59 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']; replace ref ['√xinside'] ours ['√x', 'inside']

### 06-math-inline — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933877, 403, 304, 272, 266, 302, 299, 308, 222, 194, 272, 315, 237, 246, 231, 1068]`; ink px ref/ours 1699/1833 (ratio 1.0789); SSIM blocks <0.9: 211/30294; [overlay](images/06-math-inline/pdflatex-main-export-p1-overlay.png) (75656 B, ÷1), [heatmap](images/06-math-inline/pdflatex-main-export-p1-heatmap.png) (75364 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-4.61, 3.51] pt); confidence strong (shift explains 32% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2595, differing 0.002359, SSIM₈ 0.996 (raw 0.3821, 0.002886, 0.9936)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9898→0.9937 / 0.6107→0.4148; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5648→0.6749 / 21.4145→17.7703 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `a` dx -7.78 dy 1.32; `sentence` dx -7.76 dy 1.32; `of` dx -7.59 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']; replace ref ['√xinside'] ours ['√x', 'inside']

### 06-math-inline — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935328, 325, 257, 200, 195, 223, 240, 206, 148, 143, 186, 183, 163, 166, 171, 682]`; ink px ref/ours 1699/1510 (ratio 0.8888); SSIM blocks <0.9: 159/30294; [overlay](images/06-math-inline/pdflatex-pipeline-export-p1-overlay.png) (71665 B, ÷1), [heatmap](images/06-math-inline/pdflatex-pipeline-export-p1-heatmap.png) (71993 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [3.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.2702, not lower; centroid estimate [-25.08, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2594, differing 0.002187, SSIM₈ 0.9957 (raw 0.2594, 0.002187, 0.9957)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9931→0.9931 / 0.4146→0.4146; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6926→0.6926 / 15.7779→15.7779 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `a` dx -47.04 dy -2.68; `sentence` dx -46.13 dy -2.68; `of` dx -45.21 dy -2.68
- word-sequence differences: delete ref ['a', 'b'] ours []; replace ref ['α+', 'β,'] ours [',']; replace ref ['√xinside'] ours ['inside']

### 06-math-inline — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933850, 444, 339, 259, 271, 318, 327, 365, 277, 225, 258, 298, 218, 220, 185, 962]`; ink px ref/ours 1340/1833 (ratio 1.3679); SSIM blocks <0.9: 225/30294; [overlay](images/06-math-inline/pdflatex-lm-de1020c-export-p1-overlay.png) (76717 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-de1020c-export-p1-heatmap.png) (76359 B, ÷1)
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-7.35, 3.41] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3234, differing 0.002634, SSIM₈ 0.9943 (raw 0.368, 0.002877, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9912 / 0.5881→0.4963; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5867→0.6637 / 18.2641→16.2875 [137.6,62.4–364.3,93.6 pt]
- largest word displacements (pt): `text.` dx -25.38 dy 1.32; `of` dx -25.05 dy 1.32; `sentence` dx -22.81 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']

### 06-math-inline — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933850, 444, 339, 259, 271, 318, 327, 365, 277, 225, 258, 298, 218, 220, 185, 962]`; ink px ref/ours 1340/1833 (ratio 1.3679); SSIM blocks <0.9: 225/30294; [overlay](images/06-math-inline/pdflatex-lm-main-export-p1-overlay.png) (76459 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-main-export-p1-heatmap.png) (76103 B, ÷1)
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-7.35, 3.41] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3234, differing 0.002634, SSIM₈ 0.9943 (raw 0.368, 0.002877, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9912 / 0.5881→0.4963; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5867→0.6637 / 18.2641→16.2875 [137.6,62.4–364.3,93.6 pt]
- largest word displacements (pt): `text.` dx -25.38 dy 1.32; `of` dx -25.05 dy 1.32; `sentence` dx -22.81 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']

### 06-math-inline — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934876, 375, 274, 237, 244, 281, 265, 271, 217, 211, 184, 175, 207, 165, 195, 639]`; ink px ref/ours 1340/1510 (ratio 1.1269); SSIM blocks <0.9: 190/30294; [overlay](images/06-math-inline/pdflatex-lm-pipeline-export-p1-overlay.png) (75183 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-pipeline-export-p1-heatmap.png) (72552 B, ÷1)
  - registration error (diagnostic): global shift [-7.0, 0.0] pt by ink-projection correlation (centroid estimate [-27.82, -0.09] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.278, differing 0.002307, SSIM₈ 0.9949 (raw 0.2818, 0.002315, 0.9949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9919→0.9919 / 0.4503→0.4443; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6876→0.6803 / 13.917→14.9091 [137.6,62.4–364.3,93.6 pt]
- largest word displacements (pt): `of` dx -62.68 dy -2.68; `text.` dx -62.15 dy -2.68; `sentence` dx -61.18 dy -2.68
- word-sequence differences: delete ref ['a', 'b'] ours []; replace ref ['α+', 'β,'] ours [',']; delete ref ['√x'] ours []

### 06-math-inline — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933853, 435, 339, 322, 197, 256, 302, 277, 256, 193, 249, 311, 284, 255, 208, 1079]`; ink px ref/ours 1739/1833 (ratio 1.0541); SSIM blocks <0.9: 216/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-4.48, 3.52] pt); confidence strong (shift explains 33% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2548, differing 0.002356, SSIM₈ 0.9959 (raw 0.3831, 0.002915, 0.9934)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9895→0.9934 / 0.6123→0.4073; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5653→0.6801 / 21.4919→17.5101 [131.7,62.1–343.9,93.6 pt]
- largest word displacements (pt): `inside` dx -8.16 dy 1.32; `a` dx -8.04 dy 1.32; `sentence` dx -8.02 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933853, 435, 339, 322, 197, 256, 302, 277, 256, 193, 249, 311, 284, 255, 208, 1079]`; ink px ref/ours 1739/1833 (ratio 1.0541); SSIM blocks <0.9: 216/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-4.48, 3.52] pt); confidence strong (shift explains 33% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2548, differing 0.002356, SSIM₈ 0.9959 (raw 0.3831, 0.002915, 0.9934)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9895→0.9934 / 0.6123→0.4073; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5653→0.6801 / 21.4919→17.5101 [131.7,62.1–343.9,93.6 pt]
- largest word displacements (pt): `inside` dx -8.16 dy 1.32; `a` dx -8.04 dy 1.32; `sentence` dx -8.02 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935222, 400, 327, 261, 200, 213, 220, 199, 146, 159, 149, 147, 181, 161, 170, 661]`; ink px ref/ours 1739/1510 (ratio 0.8683); SSIM blocks <0.9: 168/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [-24.95, 0.03] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2536, differing 0.002248, SSIM₈ 0.9957 (raw 0.2548, 0.002221, 0.9956)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.993→0.9933 / 0.4073→0.4053; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6986→0.7032 / 15.2967→14.8396 [131.7,62.1–343.9,93.6 pt]
- largest word displacements (pt): `inside` dx -48.22 dy -2.68; `a` dx -47.3 dy -2.68; `sentence` dx -46.39 dy -2.68
- word-sequence differences: delete ref ['a', 'b'] ours []; replace ref ['α', '+', 'β,'] ours [',']; delete ref ['√x'] ours []

### 06-math-inline — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933869, 423, 342, 268, 250, 323, 325, 362, 281, 211, 255, 310, 230, 231, 194, 942]`; ink px ref/ours 1359/1833 (ratio 1.3488); SSIM blocks <0.9: 226/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-8.96, 3.44] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3233, differing 0.002624, SSIM₈ 0.9943 (raw 0.3682, 0.002868, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9912 / 0.5885→0.4961; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5865→0.6641 / 18.2832→16.2833 [137.6,62.1–364.2,93.6 pt]
- largest word displacements (pt): `text.` dx -25.34 dy 1.32; `of` dx -25.01 dy 1.32; `sentence` dx -22.78 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933869, 423, 342, 268, 250, 323, 325, 362, 281, 211, 255, 310, 230, 231, 194, 942]`; ink px ref/ours 1359/1833 (ratio 1.3488); SSIM blocks <0.9: 226/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-8.96, 3.44] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3233, differing 0.002624, SSIM₈ 0.9943 (raw 0.3682, 0.002868, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9912 / 0.5885→0.4961; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5865→0.6641 / 18.2832→16.2833 [137.6,62.1–364.2,93.6 pt]
- largest word displacements (pt): `text.` dx -25.34 dy 1.32; `of` dx -25.01 dy 1.32; `sentence` dx -22.78 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934893, 353, 288, 241, 221, 291, 257, 270, 217, 214, 172, 186, 214, 171, 203, 625]`; ink px ref/ours 1359/1510 (ratio 1.1111); SSIM blocks <0.9: 191/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-4.0, 0.0] pt by ink-projection correlation (centroid estimate [-29.43, -0.05] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2728, differing 0.002278, SSIM₈ 0.9951 (raw 0.2818, 0.002307, 0.9949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9919→0.9922 / 0.4504→0.4361; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6877→0.6803 / 13.9202→14.8017 [137.6,62.1–364.2,93.6 pt]
- largest word displacements (pt): `of` dx -62.64 dy -2.68; `text.` dx -62.11 dy -2.68; `sentence` dx -61.15 dy -2.68
- word-sequence differences: delete ref ['a', 'b'] ours []; replace ref ['α', '+', 'β,'] ours [',']; delete ref ['√x'] ours []

### 07-math-display — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934500, 594, 464, 552, 281, 307, 196, 220, 289, 154, 134, 152, 181, 132, 125, 535]`; ink px ref/ours 1954/1796 (ratio 0.9191); SSIM blocks <0.9: 191/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-8.59, 1.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2595, differing 0.002628, SSIM₈ 0.995 (raw 0.2595, 0.002628, 0.995)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.992→0.992 / 0.4148→0.4148; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4802→0.4802 / 19.0416→19.0416 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934336, 601, 484, 555, 289, 318, 209, 232, 300, 162, 137, 160, 183, 136, 141, 573]`; ink px ref/ours 1954/1886 (ratio 0.9652); SSIM blocks <0.9: 205/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [11.72, 1.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2723, differing 0.002721, SSIM₈ 0.9946 (raw 0.2723, 0.002721, 0.9946)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9913→0.9913 / 0.4352→0.4352; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4802→0.4802 / 19.0416→19.0416 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']; insert ref [] ours ['(1)']

### 07-math-display — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934197, 429, 351, 349, 223, 227, 238, 247, 325, 173, 164, 233, 249, 211, 201, 999]`; ink px ref/ours 1954/1404 (ratio 0.7185); SSIM blocks <0.9: 245/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [9.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3862, not lower; centroid estimate [-46.49, -15.27] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3472, differing 0.002712, SSIM₈ 0.9925 (raw 0.3472, 0.002712, 0.9925)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.988→0.988 / 0.5549→0.5549; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6137→0.6137 / 10.7475→10.7475 [261.4,84.5–349.7,122.4 pt]
- largest word displacements (pt): `display.` dx 5.74 dy -34.78; `the` dx 4.82 dy -34.78; `After` dx 3.9 dy -34.78
- word-sequence differences: delete ref ['n', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 07-math-display — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933827, 410, 355, 528, 348, 373, 280, 264, 264, 218, 237, 183, 297, 180, 203, 849]`; ink px ref/ours 1608/1796 (ratio 1.1169); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-23.95, 1.41] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3495, differing 0.002895, SSIM₈ 0.9931 (raw 0.3495, 0.002895, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.989→0.989 / 0.5586→0.5586; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4893→0.4893 / 18.7965→18.7965 [261.4,84.4–349.7,122.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.73 dy 1.09; `2` dx -3.54 dy 2.25; `display.` dx -3.41 dy -0.3
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933663, 417, 375, 531, 356, 384, 293, 276, 275, 226, 240, 191, 299, 184, 219, 887]`; ink px ref/ours 1608/1886 (ratio 1.1729); SSIM blocks <0.9: 249/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-3.64, 1.4] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3623, differing 0.002987, SSIM₈ 0.9927 (raw 0.3623, 0.002987, 0.9927)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9884→0.9884 / 0.579→0.579; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4893→0.4893 / 18.7965→18.7965 [261.4,84.4–349.7,122.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.73 dy 1.09; `2` dx -3.54 dy 2.25; `display.` dx -3.41 dy -0.3
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']; insert ref [] ours ['(1)']

### 07-math-display — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934305, 340, 293, 371, 296, 305, 289, 244, 284, 194, 226, 223, 304, 186, 187, 769]`; ink px ref/ours 1608/1404 (ratio 0.8731); SSIM blocks <0.9: 261/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3381, not lower; centroid estimate [-61.85, -14.88] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3284, differing 0.002634, SSIM₈ 0.9924 (raw 0.3284, 0.002634, 0.9924)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9878→0.9878 / 0.5249→0.5249; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6177→0.6177 / 10.6117→10.6117 [261.4,84.4–349.7,122.2 pt]
- largest word displacements (pt): `After` dx 3.9 dy -35.64; `the` dx 2.47 dy -35.64; `display.` dx 0.83 dy -35.64
- word-sequence differences: delete ref ['n', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 07-math-display — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934567, 583, 430, 473, 321, 297, 196, 195, 302, 176, 128, 141, 187, 135, 135, 550]`; ink px ref/ours 1978/1796 (ratio 0.908); SSIM blocks <0.9: 193/30294; [overlay](images/07-math-display/pdflatex-de1020c-export-p1-overlay.png) (77361 B, ÷1), [heatmap](images/07-math-display/pdflatex-de1020c-export-p1-heatmap.png) (73494 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.55, 0.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2609, differing 0.00259, SSIM₈ 0.9949 (raw 0.2609, 0.00259, 0.9949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9919→0.9919 / 0.417→0.417; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4801→0.4801 / 19.0485→19.0485 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.1; `display.` dx 0.17 dy 2.13; `the` dx 0.11 dy 2.13
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934403, 590, 450, 476, 329, 308, 209, 207, 313, 184, 131, 149, 189, 139, 151, 588]`; ink px ref/ours 1978/1886 (ratio 0.9535); SSIM blocks <0.9: 207/30294; [overlay](images/07-math-display/pdflatex-main-export-p1-overlay.png) (77395 B, ÷1), [heatmap](images/07-math-display/pdflatex-main-export-p1-heatmap.png) (73676 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [12.76, 0.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2737, differing 0.002683, SSIM₈ 0.9945 (raw 0.2737, 0.002683, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9913→0.9913 / 0.4375→0.4375; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4801→0.4801 / 19.0485→19.0485 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.1; `display.` dx 0.17 dy 2.13; `the` dx 0.11 dy 2.13
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']; insert ref [] ours ['(1)']

### 07-math-display — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934427, 390, 282, 235, 296, 204, 191, 196, 322, 189, 172, 229, 273, 220, 230, 960]`; ink px ref/ours 1978/1404 (ratio 0.7098); SSIM blocks <0.9: 238/30294; [overlay](images/07-math-display/pdflatex-pipeline-export-p1-overlay.png) (76074 B, ÷1), [heatmap](images/07-math-display/pdflatex-pipeline-export-p1-heatmap.png) (74688 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3789, not lower; centroid estimate [-45.45, -15.53] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.341, differing 0.002633, SSIM₈ 0.9926 (raw 0.341, 0.002633, 0.9926)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9882→0.9882 / 0.545→0.545; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6138→0.6138 / 10.7546→10.7546 [261.4,84.5–349.7,122.4 pt]
- largest word displacements (pt): `display.` dx 5.73 dy -34.6; `the` dx 4.81 dy -34.6; `After` dx 3.9 dy -34.6
- word-sequence differences: delete ref ['n', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 07-math-display — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933745, 450, 367, 487, 356, 402, 264, 258, 285, 261, 208, 305, 184, 175, 192, 877]`; ink px ref/ours 1630/1796 (ratio 1.1018); SSIM blocks <0.9: 235/30294; [overlay](images/07-math-display/pdflatex-lm-de1020c-export-p1-overlay.png) (78133 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-de1020c-export-p1-heatmap.png) (76778 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-21.21, 0.91] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3528, differing 0.00293, SSIM₈ 0.9931 (raw 0.3528, 0.00293, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9889 / 0.5639→0.5639; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4864→0.4864 / 18.773→18.773 [261.4,84.2–349.7,122.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.75 dy 2.53; `2` dx -3.54 dy 2.36; `display.` dx -3.42 dy 0.73
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933581, 457, 387, 490, 364, 413, 277, 270, 296, 269, 211, 313, 186, 179, 208, 915]`; ink px ref/ours 1630/1886 (ratio 1.1571); SSIM blocks <0.9: 249/30294; [overlay](images/07-math-display/pdflatex-lm-main-export-p1-overlay.png) (78567 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-main-export-p1-heatmap.png) (77269 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.9, 0.9] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3656, differing 0.003023, SSIM₈ 0.9927 (raw 0.3656, 0.003023, 0.9927)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9883→0.9883 / 0.5843→0.5843; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4864→0.4864 / 18.773→18.773 [261.4,84.2–349.7,122.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.75 dy 2.53; `2` dx -3.54 dy 2.36; `display.` dx -3.42 dy 0.73
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']; insert ref [] ours ['(1)']

### 07-math-display — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934228, 414, 297, 340, 319, 298, 261, 263, 290, 258, 185, 337, 187, 196, 173, 770]`; ink px ref/ours 1630/1404 (ratio 0.8613); SSIM blocks <0.9: 262/30294; [overlay](images/07-math-display/pdflatex-lm-pipeline-export-p1-overlay.png) (76567 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-pipeline-export-p1-heatmap.png) (76306 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3384, not lower; centroid estimate [-59.11, -15.38] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3282, differing 0.002654, SSIM₈ 0.9923 (raw 0.3282, 0.002654, 0.9923)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9878→0.9878 / 0.5245→0.5245; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6148→0.6148 / 10.6134→10.6134 [261.4,84.2–349.7,122.1 pt]
- largest word displacements (pt): `After` dx 3.9 dy -34.2; `the` dx 2.47 dy -34.2; `display.` dx 0.82 dy -34.2
- word-sequence differences: delete ref ['n', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 07-math-display — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934499, 617, 467, 539, 276, 309, 204, 216, 283, 145, 142, 139, 179, 138, 120, 543]`; ink px ref/ours 1971/1796 (ratio 0.9112); SSIM blocks <0.9: 191/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-8.7, 0.93] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2589, differing 0.00263, SSIM₈ 0.995 (raw 0.2589, 0.00263, 0.995)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.992→0.992 / 0.4138→0.4138; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5723→0.5723 / 15.5285→15.5285 [261.1,82.4–349.7,128.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934335, 624, 487, 542, 284, 320, 217, 228, 294, 153, 145, 147, 181, 142, 136, 581]`; ink px ref/ours 1971/1886 (ratio 0.9569); SSIM blocks <0.9: 205/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [11.61, 0.92] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2717, differing 0.002723, SSIM₈ 0.9946 (raw 0.2717, 0.002723, 0.9946)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9913→0.9913 / 0.4342→0.4342; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5723→0.5723 / 15.5285→15.5285 [261.1,82.4–349.7,128.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']; insert ref [] ours ['(1)']

### 07-math-display — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934200, 434, 345, 346, 227, 237, 228, 222, 353, 167, 169, 222, 250, 213, 207, 996]`; ink px ref/ours 1971/1404 (ratio 0.7123); SSIM blocks <0.9: 245/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [9.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3861, not lower; centroid estimate [-46.61, -15.36] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3473, differing 0.002718, SSIM₈ 0.9925 (raw 0.3473, 0.002718, 0.9925)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.988→0.988 / 0.555→0.555; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6642→0.6642 / 8.7673→8.7673 [261.1,82.4–349.7,128.4 pt]
- largest word displacements (pt): `display.` dx 5.73 dy -34.78; `the` dx 4.81 dy -34.78; `After` dx 3.9 dy -34.78
- word-sequence differences: delete ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 07-math-display — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933826, 409, 357, 531, 338, 382, 277, 256, 275, 218, 233, 187, 294, 181, 204, 848]`; ink px ref/ours 1616/1796 (ratio 1.1114); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-24.38, 1.24] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3495, differing 0.002894, SSIM₈ 0.9931 (raw 0.3495, 0.002894, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.989→0.989 / 0.5587→0.5587; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.583→0.583 / 15.5332→15.5332 [261.1,82.3–349.7,128.3 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.73 dy 2.12; `2` dx -3.54 dy 2.25; `display.` dx -3.41 dy 0.73
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933662, 416, 377, 534, 346, 393, 290, 268, 286, 226, 236, 195, 296, 185, 220, 886]`; ink px ref/ours 1616/1886 (ratio 1.1671); SSIM blocks <0.9: 249/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.07, 1.24] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3623, differing 0.002987, SSIM₈ 0.9927 (raw 0.3623, 0.002987, 0.9927)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9884→0.9884 / 0.5791→0.5791; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.583→0.583 / 15.5332→15.5332 [261.1,82.3–349.7,128.3 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.73 dy 2.12; `2` dx -3.54 dy 2.25; `display.` dx -3.41 dy 0.73
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']; insert ref [] ours ['(1)']

### 07-math-display — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934305, 336, 298, 374, 288, 312, 285, 238, 293, 194, 223, 226, 301, 187, 188, 768]`; ink px ref/ours 1616/1404 (ratio 0.8688); SSIM blocks <0.9: 261/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3382, not lower; centroid estimate [-62.28, -15.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3285, differing 0.002632, SSIM₈ 0.9924 (raw 0.3285, 0.002632, 0.9924)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9878→0.9878 / 0.525→0.525; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6988→0.6988 / 8.7719→8.7719 [261.1,82.3–349.7,128.3 pt]
- largest word displacements (pt): `After` dx 3.9 dy -34.61; `the` dx 2.47 dy -34.61; `display.` dx 0.83 dy -34.61
- word-sequence differences: delete ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 08-two-page — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1535837, 30452, 26147, 24533, 22450, 21872, 22500, 20593, 20315, 21336, 19731, 19135, 19424, 19064, 19858, 95569]`; ink px ref/ours 152381/134585 (ratio 0.8832); SSIM blocks <0.9: 16227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, -14.5] pt REJECTED: applying it gives mean|Δ| 31.6106, not lower; centroid estimate [-5.32, -4.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.604, differing 0.230212, SSIM₈ 0.4867 (raw 31.604, 0.230212, 0.4867)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1802→0.1802 / 50.4999→50.4999; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9989→0.9989 / 0.0437→0.0437
- page 2: |Δ| histogram (16 bins, pixel counts) `[1518144, 30651, 25967, 24422, 22157, 21752, 22680, 20693, 20985, 22451, 20965, 20617, 21112, 20181, 20664, 105375]`; ink px ref/ours 151462/131743 (ratio 0.8698); SSIM blocks <0.9: 16822/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 6.0] pt by ink-projection correlation (centroid estimate [-4.96, 8.87] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.1586, differing 0.226538, SSIM₈ 0.4928 (raw 33.6579, 0.240085, 0.4507)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1228→0.1908 / 53.7834→49.7761; header-band 1.0→0.9986 / 0.0→0.0276; footer-band 0.9989→0.9989 / 0.0284→0.0284
- page 3: |Δ| histogram (16 bins, pixel counts) `[1774169, 11550, 9778, 9157, 8337, 8439, 8946, 8134, 7667, 8840, 8115, 7824, 8036, 7751, 8314, 43759]`; ink px ref/ours 33623/71008 (ratio 2.1119); SSIM blocks <0.9: 7433/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 34.5] pt by ink-projection correlation (centroid estimate [-5.9, 112.32] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.1344, differing 0.082072, SSIM₈ 0.8081 (raw 13.3908, 0.093775, 0.7598)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6161→0.7084 / 21.4009→17.02; header-band 1.0→0.8984 / 0.0→5.299; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 214.06; `branch` dx -435.47 dy 167.32; `over` dx -407.04 dy 214.02

### 08-two-page — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1535837, 30452, 26147, 24533, 22450, 21872, 22500, 20593, 20315, 21336, 19731, 19135, 19424, 19064, 19858, 95569]`; ink px ref/ours 152381/134585 (ratio 0.8832); SSIM blocks <0.9: 16227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, -14.5] pt REJECTED: applying it gives mean|Δ| 31.6106, not lower; centroid estimate [-5.32, -4.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.604, differing 0.230212, SSIM₈ 0.4867 (raw 31.604, 0.230212, 0.4867)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1802→0.1802 / 50.4999→50.4999; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9989→0.9989 / 0.0437→0.0437
- page 2: |Δ| histogram (16 bins, pixel counts) `[1518144, 30651, 25967, 24422, 22157, 21752, 22680, 20693, 20985, 22451, 20965, 20617, 21112, 20181, 20664, 105375]`; ink px ref/ours 151462/131743 (ratio 0.8698); SSIM blocks <0.9: 16822/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 6.0] pt by ink-projection correlation (centroid estimate [-4.96, 8.87] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.1586, differing 0.226538, SSIM₈ 0.4928 (raw 33.6579, 0.240085, 0.4507)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1228→0.1908 / 53.7834→49.7761; header-band 1.0→0.9986 / 0.0→0.0276; footer-band 0.9989→0.9989 / 0.0284→0.0284
- page 3: |Δ| histogram (16 bins, pixel counts) `[1774169, 11550, 9778, 9157, 8337, 8439, 8946, 8134, 7667, 8840, 8115, 7824, 8036, 7751, 8314, 43759]`; ink px ref/ours 33623/71008 (ratio 2.1119); SSIM blocks <0.9: 7433/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 34.5] pt by ink-projection correlation (centroid estimate [-5.9, 112.32] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.1344, differing 0.082072, SSIM₈ 0.8081 (raw 13.3908, 0.093775, 0.7598)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6161→0.7084 / 21.4009→17.02; header-band 1.0→0.8984 / 0.0→5.299; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 214.06; `branch` dx -435.47 dy 167.32; `over` dx -407.04 dy 214.02

### 08-two-page — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1583136, 30910, 26298, 23155, 21464, 20385, 19396, 18988, 18535, 18614, 17496, 16515, 16559, 16202, 16455, 74708]`; ink px ref/ours 152381/134174 (ratio 0.8805); SSIM blocks <0.9: 14354/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 26.6859, not lower; centroid estimate [2.53, -11.22] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.6805, differing 0.205375, SSIM₈ 0.5916 (raw 26.6805, 0.205375, 0.5916)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3488→0.3488 / 42.6251→42.6251; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9989→0.9989 / 0.0437→0.0437
- page 2: |Δ| histogram (16 bins, pixel counts) `[1586005, 30562, 25889, 22904, 21768, 20373, 19583, 18837, 18225, 17903, 17338, 16591, 16323, 16450, 16042, 74023]`; ink px ref/ours 151462/134174 (ratio 0.8859); SSIM blocks <0.9: 14228/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [3.47, -10.88] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.4549, differing 0.203953, SSIM₈ 0.5964 (raw 26.4549, 0.203953, 0.5964)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3565→0.3565 / 42.2655→42.2655; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9989→0.9989 / 0.0284→0.0284
- page 3: |Δ| histogram (16 bins, pixel counts) `[1802555, 10867, 9467, 8075, 7855, 7468, 7117, 7279, 6685, 7302, 6633, 6655, 6385, 6421, 6393, 31659]`; ink px ref/ours 33623/67103 (ratio 1.9957); SSIM blocks <0.9: 6136/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.92, 84.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 10.574, differing 0.078506, SSIM₈ 0.8179 (raw 10.574, 0.078506, 0.8179)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7095→0.7095 / 16.8957→16.8957; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 173.76; `branch` dx -435.47 dy 130.42; `branch` dx -435.47 dy 115.97

### 08-two-page — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571209, 29050, 24879, 23374, 22427, 21235, 22990, 22122, 21052, 20710, 18292, 17708, 17105, 16387, 17019, 73257]`; ink px ref/ours 104967/134585 (ratio 1.2822); SSIM blocks <0.9: 15666/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.39, -3.28] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.439, differing 0.210412, SSIM₈ 0.5039 (raw 27.547, 0.210718, 0.5025)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.2081 / 44.0152→43.8425; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1555826, 29617, 25174, 23833, 22680, 21623, 23309, 22906, 21714, 21966, 18997, 18388, 18461, 17121, 17761, 79440]`; ink px ref/ours 104819/131743 (ratio 1.2569); SSIM blocks <0.9: 16449/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 36.5] pt by ink-projection correlation (centroid estimate [-5.16, 9.05] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.3277, differing 0.209484, SSIM₈ 0.5027 (raw 29.058, 0.219579, 0.4664)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1479→0.2199 / 46.4363→42.9133; header-band 1.0→0.9042 / 0.0→5.2131; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746654, 14263, 12363, 11679, 11023, 10858, 11514, 11288, 10599, 11007, 9764, 9045, 9193, 8553, 9140, 41873]`; ink px ref/ours 46386/71008 (ratio 1.5308); SSIM blocks <0.9: 8607/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 52.0] pt by ink-projection correlation (centroid estimate [-5.15, 40.4] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.4871, differing 0.102858, SSIM₈ 0.754 (raw 14.7852, 0.11002, 0.721)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5542→0.6347 / 23.6301→20.0799; header-band 1.0→0.8091 / 0.0→10.1537; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 54.44; `oak` dx 426.94 dy 36.59; `oak` dx 426.94 dy 31.09

### 08-two-page — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571209, 29050, 24879, 23374, 22427, 21235, 22990, 22122, 21052, 20710, 18292, 17708, 17105, 16387, 17019, 73257]`; ink px ref/ours 104967/134585 (ratio 1.2822); SSIM blocks <0.9: 15666/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.39, -3.28] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.439, differing 0.210412, SSIM₈ 0.5039 (raw 27.547, 0.210718, 0.5025)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.2081 / 44.0152→43.8425; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1555826, 29617, 25174, 23833, 22680, 21623, 23309, 22906, 21714, 21966, 18997, 18388, 18461, 17121, 17761, 79440]`; ink px ref/ours 104819/131743 (ratio 1.2569); SSIM blocks <0.9: 16449/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 36.5] pt by ink-projection correlation (centroid estimate [-5.16, 9.05] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.3277, differing 0.209484, SSIM₈ 0.5027 (raw 29.058, 0.219579, 0.4664)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1479→0.2199 / 46.4363→42.9133; header-band 1.0→0.9042 / 0.0→5.2131; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746654, 14263, 12363, 11679, 11023, 10858, 11514, 11288, 10599, 11007, 9764, 9045, 9193, 8553, 9140, 41873]`; ink px ref/ours 46386/71008 (ratio 1.5308); SSIM blocks <0.9: 8607/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 52.0] pt by ink-projection correlation (centroid estimate [-5.15, 40.4] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.4871, differing 0.102858, SSIM₈ 0.754 (raw 14.7852, 0.11002, 0.721)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5542→0.6347 / 23.6301→20.0799; header-band 1.0→0.8091 / 0.0→10.1537; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 54.44; `oak` dx 426.94 dy 36.59; `oak` dx 426.94 dy 31.09

### 08-two-page — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1612270, 28080, 24275, 22255, 21604, 20289, 20381, 20180, 18813, 18052, 16129, 15291, 14445, 14586, 14330, 57836]`; ink px ref/ours 104967/134174 (ratio 1.2782); SSIM blocks <0.9: 13677/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.5, 0.0] pt by ink-projection correlation (centroid estimate [3.45, -10.45] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.4287, differing 0.188564, SSIM₈ 0.6061 (raw 23.5751, 0.189149, 0.6073)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3745→0.3765 / 37.6606→37.262; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1607617, 28422, 24896, 22322, 21633, 20473, 19866, 20717, 18886, 18267, 16421, 15632, 14707, 14500, 14411, 60046]`; ink px ref/ours 104819/134174 (ratio 1.2801); SSIM blocks <0.9: 14205/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 14.5] pt by ink-projection correlation (centroid estimate [3.26, -10.7] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.7282, differing 0.190321, SSIM₈ 0.5983 (raw 24.0039, 0.191671, 0.589)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3449→0.3741 / 38.352→37.2083; header-band 1.0→0.9079 / 0.0→4.8172; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1784258, 13233, 11910, 10301, 10232, 9586, 9268, 9524, 8814, 8572, 7621, 7224, 6962, 6664, 6624, 28023]`; ink px ref/ours 46386/67103 (ratio 1.4466); SSIM blocks <0.9: 6674/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 11.213, not lower; centroid estimate [3.67, 12.39] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.1835, differing 0.089657, SSIM₈ 0.8094 (raw 11.1835, 0.089657, 0.8094)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.696→0.696 / 17.8711→17.8711; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -14.8; `oak` dx 450.74 dy -14.8; `oak` dx 450.74 dy -14.8

### 08-two-page — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p1-overlay.png) (56586 B, ÷8), [heatmap](images/08-two-page/pdflatex-de1020c-export-p1-heatmap.png) (38478 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -14.5] pt by ink-projection correlation (centroid estimate [-6.07, -5.06] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.4928, differing 0.229367, SSIM₈ 0.4891 (raw 31.5764, 0.22951, 0.4881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1821→0.1892 / 50.4615→50.1371; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.96 / 0.0468→1.3594
- page 2: |Δ| histogram (16 bins, pixel counts) `[1518465, 30739, 25693, 24091, 22137, 22533, 22535, 20944, 20824, 22593, 20919, 20497, 20504, 19480, 20990, 105872]`; ink px ref/ours 150460/131743 (ratio 0.8756); SSIM blocks <0.9: 16800/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p2-overlay.png) (55036 B, ÷8), [heatmap](images/08-two-page/pdflatex-de1020c-export-p2-heatmap.png) (38869 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 6.0] pt by ink-projection correlation (centroid estimate [-5.52, 7.86] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.0952, differing 0.225993, SSIM₈ 0.4952 (raw 33.6289, 0.239479, 0.4516)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1237→0.1935 / 53.7443→49.6907; header-band 1.0→0.999 / 0.0→0.0276; footer-band 0.9986→0.9986 / 0.0304→0.0304
- page 3: |Δ| histogram (16 bins, pixel counts) `[1774335, 11537, 9689, 9104, 8316, 8407, 8934, 8213, 7615, 9012, 8045, 7866, 7928, 7590, 8264, 43961]`; ink px ref/ours 33504/71008 (ratio 2.1194); SSIM blocks <0.9: 7408/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p3-overlay.png) (66684 B, ÷4), [heatmap](images/08-two-page/pdflatex-de1020c-export-p3-heatmap.png) (59661 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 34.5] pt by ink-projection correlation (centroid estimate [-6.67, 111.28] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.0633, differing 0.081669, SSIM₈ 0.8097 (raw 13.387, 0.093579, 0.7605)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6172→0.7105 / 21.3963→16.9124; header-band 1.0→0.8991 / 0.0→5.299; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 214.06; `branch` dx -435.47 dy 167.32; `over` dx -407.04 dy 214.02

### 08-two-page — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/08-two-page/pdflatex-main-export-p1-overlay.png) (56375 B, ÷8), [heatmap](images/08-two-page/pdflatex-main-export-p1-heatmap.png) (38279 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -14.5] pt by ink-projection correlation (centroid estimate [-6.07, -5.06] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.4928, differing 0.229367, SSIM₈ 0.4891 (raw 31.5764, 0.22951, 0.4881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1821→0.1892 / 50.4615→50.1371; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.96 / 0.0468→1.3594
- page 2: |Δ| histogram (16 bins, pixel counts) `[1518465, 30739, 25693, 24091, 22137, 22533, 22535, 20944, 20824, 22593, 20919, 20497, 20504, 19480, 20990, 105872]`; ink px ref/ours 150460/131743 (ratio 0.8756); SSIM blocks <0.9: 16800/30294; [overlay](images/08-two-page/pdflatex-main-export-p2-overlay.png) (54836 B, ÷8), [heatmap](images/08-two-page/pdflatex-main-export-p2-heatmap.png) (38677 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 6.0] pt by ink-projection correlation (centroid estimate [-5.52, 7.86] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.0952, differing 0.225993, SSIM₈ 0.4952 (raw 33.6289, 0.239479, 0.4516)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1237→0.1935 / 53.7443→49.6907; header-band 1.0→0.999 / 0.0→0.0276; footer-band 0.9986→0.9986 / 0.0304→0.0304
- page 3: |Δ| histogram (16 bins, pixel counts) `[1774335, 11537, 9689, 9104, 8316, 8407, 8934, 8213, 7615, 9012, 8045, 7866, 7928, 7590, 8264, 43961]`; ink px ref/ours 33504/71008 (ratio 2.1194); SSIM blocks <0.9: 7408/30294; [overlay](images/08-two-page/pdflatex-main-export-p3-overlay.png) (66599 B, ÷4), [heatmap](images/08-two-page/pdflatex-main-export-p3-heatmap.png) (59570 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 34.5] pt by ink-projection correlation (centroid estimate [-6.67, 111.28] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.0633, differing 0.081669, SSIM₈ 0.8097 (raw 13.387, 0.093579, 0.7605)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6172→0.7105 / 21.3963→16.9124; header-band 1.0→0.8991 / 0.0→5.299; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 214.06; `branch` dx -435.47 dy 167.32; `over` dx -407.04 dy 214.02

### 08-two-page — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1587936, 30510, 26093, 22620, 21328, 20753, 19095, 19085, 18279, 18386, 17021, 16649, 15827, 15615, 16048, 73571]`; ink px ref/ours 151753/134174 (ratio 0.8842); SSIM blocks <0.9: 14216/30294; [overlay](images/08-two-page/pdflatex-pipeline-export-p1-overlay.png) (56629 B, ÷8), [heatmap](images/08-two-page/pdflatex-pipeline-export-p1-heatmap.png) (36434 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [1.77, -12.23] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.254, differing 0.202987, SSIM₈ 0.5996 (raw 26.254, 0.202987, 0.5996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3614→0.3614 / 41.9484→41.9484; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.9984 / 0.0468→0.0468
- page 2: |Δ| histogram (16 bins, pixel counts) `[1588531, 30127, 25795, 22738, 21438, 20590, 19323, 19016, 18278, 18487, 17356, 16794, 15871, 15577, 15909, 72986]`; ink px ref/ours 150460/134174 (ratio 0.8918); SSIM blocks <0.9: 14145/30294; [overlay](images/08-two-page/pdflatex-pipeline-export-p2-overlay.png) (56604 B, ÷8), [heatmap](images/08-two-page/pdflatex-pipeline-export-p2-heatmap.png) (36342 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.9, -11.89] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.2072, differing 0.202468, SSIM₈ 0.6013 (raw 26.2072, 0.202468, 0.6013)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.364→0.364 / 41.876→41.876; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9986→0.9986 / 0.0304→0.0304
- page 3: |Δ| histogram (16 bins, pixel counts) `[1803785, 10879, 9434, 8016, 7946, 7641, 7035, 7217, 6640, 7131, 6477, 6705, 6196, 6245, 6378, 31091]`; ink px ref/ours 33504/67103 (ratio 2.0028); SSIM blocks <0.9: 6094/30294; [overlay](images/08-two-page/pdflatex-pipeline-export-p3-overlay.png) (65891 B, ÷4), [heatmap](images/08-two-page/pdflatex-pipeline-export-p3-heatmap.png) (51004 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.15, 83.26] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 10.4354, differing 0.077918, SSIM₈ 0.8205 (raw 10.4354, 0.077918, 0.8205)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7136→0.7136 / 16.6757→16.6757; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 173.76; `branch` dx -435.47 dy 130.42; `branch` dx -435.47 dy 115.97

### 08-two-page — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p1-overlay.png) (55132 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p1-heatmap.png) (37729 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.44, -3.32] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4403, differing 0.210395, SSIM₈ 0.5038 (raw 27.5467, 0.210674, 0.5026)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.208 / 44.0147→43.8447; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1556053, 29239, 25104, 24114, 22751, 21524, 23492, 22483, 22053, 21728, 19173, 18342, 18558, 16712, 18303, 79187]`; ink px ref/ours 105011/131743 (ratio 1.2546); SSIM blocks <0.9: 16449/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p2-overlay.png) (54166 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p2-heatmap.png) (38489 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 36.5] pt by ink-projection correlation (centroid estimate [-5.19, 9.09] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.3288, differing 0.209484, SSIM₈ 0.5027 (raw 29.0579, 0.219561, 0.4664)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.148→0.2198 / 46.4361→42.915; header-band 1.0→0.9042 / 0.0→5.2131; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746783, 14069, 12340, 11757, 11133, 10758, 11588, 11172, 10690, 10911, 9823, 9052, 9221, 8387, 9339, 41793]`; ink px ref/ours 46441/71008 (ratio 1.529); SSIM blocks <0.9: 8608/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p3-overlay.png) (81074 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p3-heatmap.png) (66153 B, ÷4)
  - registration error (diagnostic): global shift [-1.0, 52.0] pt by ink-projection correlation (centroid estimate [-5.44, 40.42] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.4866, differing 0.102874, SSIM₈ 0.754 (raw 14.7842, 0.109995, 0.721)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5542→0.6347 / 23.6284→20.0792; header-band 1.0→0.8091 / 0.0→10.1537; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 55.47; `oak` dx 426.94 dy 37.61; `oak` dx 426.94 dy 32.12

### 08-two-page — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p1-overlay.png) (54950 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-main-export-p1-heatmap.png) (37540 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.44, -3.32] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4403, differing 0.210395, SSIM₈ 0.5038 (raw 27.5467, 0.210674, 0.5026)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.208 / 44.0147→43.8447; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1556053, 29239, 25104, 24114, 22751, 21524, 23492, 22483, 22053, 21728, 19173, 18342, 18558, 16712, 18303, 79187]`; ink px ref/ours 105011/131743 (ratio 1.2546); SSIM blocks <0.9: 16449/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p2-overlay.png) (53988 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-main-export-p2-heatmap.png) (38312 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 36.5] pt by ink-projection correlation (centroid estimate [-5.19, 9.09] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.3288, differing 0.209484, SSIM₈ 0.5027 (raw 29.0579, 0.219561, 0.4664)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.148→0.2198 / 46.4361→42.915; header-band 1.0→0.9042 / 0.0→5.2131; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746783, 14069, 12340, 11757, 11133, 10758, 11588, 11172, 10690, 10911, 9823, 9052, 9221, 8387, 9339, 41793]`; ink px ref/ours 46441/71008 (ratio 1.529); SSIM blocks <0.9: 8608/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p3-overlay.png) (80976 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-main-export-p3-heatmap.png) (66055 B, ÷4)
  - registration error (diagnostic): global shift [-1.0, 52.0] pt by ink-projection correlation (centroid estimate [-5.44, 40.42] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.4866, differing 0.102874, SSIM₈ 0.754 (raw 14.7842, 0.109995, 0.721)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5542→0.6347 / 23.6284→20.0792; header-band 1.0→0.8091 / 0.0→10.1537; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 55.47; `oak` dx 426.94 dy 37.61; `oak` dx 426.94 dy 32.12

### 08-two-page — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1612433, 27856, 24167, 22468, 21846, 20013, 20505, 19937, 18971, 17855, 16393, 15247, 14544, 14310, 14602, 57669]`; ink px ref/ours 105111/134174 (ratio 1.2765); SSIM blocks <0.9: 13679/30294; [overlay](images/08-two-page/pdflatex-lm-pipeline-export-p1-overlay.png) (55062 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-pipeline-export-p1-heatmap.png) (35533 B, ÷8)
  - registration error (diagnostic): global shift [-2.5, 0.0] pt by ink-projection correlation (centroid estimate [3.4, -10.49] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.4215, differing 0.188519, SSIM₈ 0.6062 (raw 23.5734, 0.189081, 0.6073)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3745→0.3767 / 37.6579→37.2505; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1607690, 28300, 24750, 22592, 21739, 20338, 19968, 20354, 19216, 18173, 16453, 15705, 14682, 14172, 14883, 59801]`; ink px ref/ours 105011/134174 (ratio 1.2777); SSIM blocks <0.9: 14200/30294; [overlay](images/08-two-page/pdflatex-lm-pipeline-export-p2-overlay.png) (55519 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-pipeline-export-p2-heatmap.png) (36339 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 14.5] pt by ink-projection correlation (centroid estimate [3.24, -10.66] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.7249, differing 0.190314, SSIM₈ 0.5983 (raw 24.0008, 0.191652, 0.5891)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3451→0.3742 / 38.3471→37.203; header-band 1.0→0.9079 / 0.0→4.8172; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1784343, 13071, 11844, 10472, 10298, 9502, 9300, 9447, 8913, 8477, 7646, 7275, 6894, 6647, 6768, 27919]`; ink px ref/ours 46441/67103 (ratio 1.4449); SSIM blocks <0.9: 6673/30294; [overlay](images/08-two-page/pdflatex-lm-pipeline-export-p3-overlay.png) (78592 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-pipeline-export-p3-heatmap.png) (53461 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 11.2121, not lower; centroid estimate [3.38, 12.41] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.183, differing 0.089625, SSIM₈ 0.8094 (raw 11.183, 0.089625, 0.8094)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.696→0.696 / 17.8702→17.8702; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy 15.12; `oak` dx 450.74 dy -13.78; `oak` dx 450.74 dy -13.78

### 08-two-page — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1535378, 29989, 25555, 24965, 22556, 22246, 22654, 20669, 20453, 21240, 20149, 19662, 19349, 18997, 19406, 95548]`; ink px ref/ours 152232/134585 (ratio 0.8841); SSIM blocks <0.9: 16215/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, -14.5] pt by ink-projection correlation (centroid estimate [-5.35, -4.01] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.6466, differing 0.23054, SSIM₈ 0.4854 (raw 31.6507, 0.230423, 0.4864)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1798→0.1843 / 50.5745→50.3662; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9987→0.9601 / 0.0438→1.3685
- page 2: |Δ| histogram (16 bins, pixel counts) `[1517468, 30567, 25348, 24959, 22402, 22356, 22688, 20537, 21047, 22558, 21491, 21046, 20679, 20011, 20481, 105178]`; ink px ref/ours 151458/131743 (ratio 0.8698); SSIM blocks <0.9: 16818/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 6.0] pt by ink-projection correlation (centroid estimate [-5.03, 8.77] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.1667, differing 0.226588, SSIM₈ 0.493 (raw 33.675, 0.240149, 0.4508)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1229→0.1911 / 53.8106→49.7887; header-band 1.0→0.9986 / 0.0→0.0276; footer-band 0.9989→0.9989 / 0.0285→0.0285
- page 3: |Δ| histogram (16 bins, pixel counts) `[1773969, 11449, 9657, 9335, 8367, 8575, 8971, 8031, 7696, 8844, 8288, 7974, 7985, 7726, 8258, 43691]`; ink px ref/ours 33659/71008 (ratio 2.1096); SSIM blocks <0.9: 7435/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 34.5] pt by ink-projection correlation (centroid estimate [-5.9, 112.18] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.1653, differing 0.08222, SSIM₈ 0.8076 (raw 13.4019, 0.093836, 0.7597)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.616→0.7076 / 21.4186→17.0693; header-band 1.0→0.8984 / 0.0→5.299; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 214.06; `branch` dx -435.52 dy 167.32; `over` dx -406.99 dy 214.02

### 08-two-page — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1535378, 29989, 25555, 24965, 22556, 22246, 22654, 20669, 20453, 21240, 20149, 19662, 19349, 18997, 19406, 95548]`; ink px ref/ours 152232/134585 (ratio 0.8841); SSIM blocks <0.9: 16215/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, -14.5] pt by ink-projection correlation (centroid estimate [-5.35, -4.01] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.6466, differing 0.23054, SSIM₈ 0.4854 (raw 31.6507, 0.230423, 0.4864)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1798→0.1843 / 50.5745→50.3662; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9987→0.9601 / 0.0438→1.3685
- page 2: |Δ| histogram (16 bins, pixel counts) `[1517468, 30567, 25348, 24959, 22402, 22356, 22688, 20537, 21047, 22558, 21491, 21046, 20679, 20011, 20481, 105178]`; ink px ref/ours 151458/131743 (ratio 0.8698); SSIM blocks <0.9: 16818/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 6.0] pt by ink-projection correlation (centroid estimate [-5.03, 8.77] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.1667, differing 0.226588, SSIM₈ 0.493 (raw 33.675, 0.240149, 0.4508)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1229→0.1911 / 53.8106→49.7887; header-band 1.0→0.9986 / 0.0→0.0276; footer-band 0.9989→0.9989 / 0.0285→0.0285
- page 3: |Δ| histogram (16 bins, pixel counts) `[1773969, 11449, 9657, 9335, 8367, 8575, 8971, 8031, 7696, 8844, 8288, 7974, 7985, 7726, 8258, 43691]`; ink px ref/ours 33659/71008 (ratio 2.1096); SSIM blocks <0.9: 7435/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 34.5] pt by ink-projection correlation (centroid estimate [-5.9, 112.18] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.1653, differing 0.08222, SSIM₈ 0.8076 (raw 13.4019, 0.093836, 0.7597)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.616→0.7076 / 21.4186→17.0693; header-band 1.0→0.8984 / 0.0→5.299; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 214.06; `branch` dx -435.52 dy 167.32; `over` dx -406.99 dy 214.02

### 08-two-page — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1583745, 30278, 25836, 23637, 22043, 20918, 19528, 18857, 18576, 18283, 17742, 17071, 16276, 16308, 15958, 73760]`; ink px ref/ours 152232/134174 (ratio 0.8814); SSIM blocks <0.9: 14338/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.49, -11.18] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.5737, differing 0.205156, SSIM₈ 0.5932 (raw 26.5737, 0.205156, 0.5932)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3513→0.3513 / 42.4542→42.4542; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9987→0.9987 / 0.0438→0.0438
- page 2: |Δ| histogram (16 bins, pixel counts) `[1585747, 30302, 25607, 23538, 21603, 20807, 19783, 18649, 18339, 18004, 17665, 17252, 15884, 16053, 15874, 73709]`; ink px ref/ours 151458/134174 (ratio 0.8859); SSIM blocks <0.9: 14205/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [3.39, -10.98] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.4302, differing 0.203958, SSIM₈ 0.5965 (raw 26.4302, 0.203958, 0.5965)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3567→0.3567 / 42.2258→42.2258; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9989→0.9989 / 0.0285→0.0285
- page 3: |Δ| histogram (16 bins, pixel counts) `[1802295, 10701, 9398, 8110, 8066, 7603, 7180, 7046, 6729, 7236, 6766, 6803, 6350, 6469, 6399, 31665]`; ink px ref/ours 33659/67103 (ratio 1.9936); SSIM blocks <0.9: 6135/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.92, 84.17] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 10.6011, differing 0.078637, SSIM₈ 0.8174 (raw 10.6011, 0.078637, 0.8174)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7087→0.7087 / 16.939→16.939; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 173.76; `branch` dx -435.52 dy 130.42; `branch` dx -435.52 dy 115.97

### 08-two-page — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571173, 29128, 24898, 23367, 22271, 21337, 23030, 22042, 21136, 20589, 18380, 17673, 17118, 16399, 17001, 73274]`; ink px ref/ours 105020/134585 (ratio 1.2815); SSIM blocks <0.9: 15670/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.46, -3.21] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4386, differing 0.210358, SSIM₈ 0.5039 (raw 27.5472, 0.210665, 0.5025)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2059→0.2081 / 44.0154→43.8419; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1555802, 29658, 25241, 23771, 22602, 21613, 23447, 22794, 21821, 21830, 19124, 18335, 18417, 17162, 17754, 79445]`; ink px ref/ours 104857/131743 (ratio 1.2564); SSIM blocks <0.9: 16449/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 36.5] pt by ink-projection correlation (centroid estimate [-5.25, 9.02] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.3274, differing 0.209432, SSIM₈ 0.5027 (raw 29.0582, 0.219508, 0.4664)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1479→0.2198 / 46.4365→42.9128; header-band 1.0→0.9042 / 0.0→5.2131; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746646, 14271, 12388, 11672, 10984, 10870, 11549, 11242, 10632, 10956, 9797, 9063, 9172, 8550, 9128, 41896]`; ink px ref/ours 46395/71008 (ratio 1.5305); SSIM blocks <0.9: 8610/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 52.0] pt by ink-projection correlation (centroid estimate [-5.21, 40.43] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.4868, differing 0.102833, SSIM₈ 0.754 (raw 14.7852, 0.109992, 0.7209)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5542→0.6347 / 23.63→20.0794; header-band 1.0→0.8091 / 0.0→10.1537; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 55.47; `oak` dx 426.94 dy 37.61; `oak` dx 426.94 dy 32.12

### 08-two-page — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571173, 29128, 24898, 23367, 22271, 21337, 23030, 22042, 21136, 20589, 18380, 17673, 17118, 16399, 17001, 73274]`; ink px ref/ours 105020/134585 (ratio 1.2815); SSIM blocks <0.9: 15670/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.46, -3.21] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4386, differing 0.210358, SSIM₈ 0.5039 (raw 27.5472, 0.210665, 0.5025)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2059→0.2081 / 44.0154→43.8419; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1555802, 29658, 25241, 23771, 22602, 21613, 23447, 22794, 21821, 21830, 19124, 18335, 18417, 17162, 17754, 79445]`; ink px ref/ours 104857/131743 (ratio 1.2564); SSIM blocks <0.9: 16449/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 36.5] pt by ink-projection correlation (centroid estimate [-5.25, 9.02] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.3274, differing 0.209432, SSIM₈ 0.5027 (raw 29.0582, 0.219508, 0.4664)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1479→0.2198 / 46.4365→42.9128; header-band 1.0→0.9042 / 0.0→5.2131; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746646, 14271, 12388, 11672, 10984, 10870, 11549, 11242, 10632, 10956, 9797, 9063, 9172, 8550, 9128, 41896]`; ink px ref/ours 46395/71008 (ratio 1.5305); SSIM blocks <0.9: 8610/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 52.0] pt by ink-projection correlation (centroid estimate [-5.21, 40.43] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.4868, differing 0.102833, SSIM₈ 0.754 (raw 14.7852, 0.109992, 0.7209)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5542→0.6347 / 23.63→20.0794; header-band 1.0→0.8091 / 0.0→10.1537; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 55.47; `oak` dx 426.94 dy 37.61; `oak` dx 426.94 dy 32.12

### 08-two-page — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1612227, 28099, 24375, 22218, 21483, 20425, 20378, 20120, 18848, 18013, 16182, 15257, 14451, 14534, 14340, 57866]`; ink px ref/ours 105020/134174 (ratio 1.2776); SSIM blocks <0.9: 13680/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.5, 0.0] pt by ink-projection correlation (centroid estimate [3.38, -10.38] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.4266, differing 0.188488, SSIM₈ 0.6061 (raw 23.5741, 0.189094, 0.6074)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3745→0.3765 / 37.659→37.2586; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1607611, 28493, 24923, 22279, 21517, 20572, 19877, 20651, 18915, 18192, 16509, 15639, 14675, 14531, 14385, 60047]`; ink px ref/ours 104857/134174 (ratio 1.2796); SSIM blocks <0.9: 14205/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 14.5] pt by ink-projection correlation (centroid estimate [3.18, -10.73] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.726, differing 0.190256, SSIM₈ 0.5983 (raw 24.003, 0.19161, 0.5891)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.345→0.3741 / 38.3506→37.2048; header-band 1.0→0.9079 / 0.0→4.8172; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1784247, 13238, 11939, 10290, 10196, 9595, 9265, 9516, 8834, 8547, 7653, 7252, 6925, 6666, 6629, 28024]`; ink px ref/ours 46395/67103 (ratio 1.4463); SSIM blocks <0.9: 6674/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 11.2132, not lower; centroid estimate [3.61, 12.41] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.1845, differing 0.089642, SSIM₈ 0.8094 (raw 11.1845, 0.089642, 0.8094)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.696→0.696 / 17.8726→17.8726; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy 15.12; `oak` dx 450.74 dy -13.78; `oak` dx 450.74 dy -13.78

### 09-mixed-document — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908424, 2029, 1775, 1940, 1563, 1506, 1540, 1401, 1384, 1449, 1711, 1473, 1402, 1450, 1477, 8292]`; ink px ref/ours 9691/9252 (ratio 0.9547); SSIM blocks <0.9: 1549/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 38.5] pt by ink-projection correlation (centroid estimate [6.27, 32.14] pt); confidence strong (shift explains 28% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.7912, differing 0.014073, SSIM₈ 0.9702 (raw 2.4844, 0.017426, 0.9504)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9208→0.9543 / 3.9708→2.7408; header-band 1.0→0.9861 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6356→0.7568 / 15.3342→9.5623 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -2.87 dy 39.06; `in` dx -2.66 dy 39.06; `set` dx -2.46 dy 39.06
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908131, 2035, 1814, 1942, 1572, 1504, 1531, 1399, 1398, 1458, 1718, 1493, 1411, 1490, 1509, 8411]`; ink px ref/ours 9691/9465 (ratio 0.9767); SSIM blocks <0.9: 1568/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 38.5] pt by ink-projection correlation (centroid estimate [8.8, 30.82] pt); confidence strong (shift explains 28% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8136, differing 0.014201, SSIM₈ 0.9695 (raw 2.5141, 0.017591, 0.9497)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9196→0.9537 / 4.0183→2.7612; header-band 1.0→0.9829 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6356→0.7568 / 15.3342→9.5623 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -2.87 dy 39.06; `in` dx -2.66 dy 39.06; `set` dx -2.46 dy 39.06
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-italic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1912194, 1898, 1625, 1558, 1617, 1418, 1544, 1403, 1360, 1249, 1307, 1252, 1122, 1347, 1269, 6653]`; ink px ref/ours 9691/8456 (ratio 0.8726); SSIM blocks <0.9: 1241/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -47.0] pt by ink-projection correlation (centroid estimate [-0.33, -27.05] pt); confidence strong (shift explains 28% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.5254, differing 0.012766, SSIM₈ 0.9738 (raw 2.1112, 0.015239, 0.961)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9377→0.9582 / 3.3743→2.4381; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.589→0.7082 / 20.3431→12.7845 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `.` dx -51.4 dy -0.05; `paper.` dx 6.71 dy -46.38; `letter` dx 5.78 dy -46.38
- word-sequence differences: delete ref ['x2', '+', 'y2', '=', 'z2'] ours []; delete ref ['1', '2', '+', '1', '5', '=', '3', '6'] ours []

### 09-mixed-document — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909444, 2180, 1960, 2265, 1611, 1596, 1560, 1754, 1443, 1429, 1644, 1400, 1251, 1263, 1288, 6728]`; ink px ref/ours 7447/9252 (ratio 1.2424); SSIM blocks <0.9: 1580/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 39.0] pt by ink-projection correlation (centroid estimate [0.63, 34.84] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9017, differing 0.014708, SSIM₈ 0.9641 (raw 2.2538, 0.016899, 0.9494)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9192→0.9448 / 3.6023→2.9175; header-band 1.0→0.9856 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6352→0.7524 / 13.0739→8.01 [275.7,96.4–519.8,156 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 24.19; `paper.` dx -52.18 dy 38.59; `letter` dx -48.63 dy 38.59
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909251, 2188, 1999, 2275, 1629, 1577, 1541, 1758, 1460, 1465, 1654, 1412, 1256, 1295, 1308, 6748]`; ink px ref/ours 7447/9465 (ratio 1.271); SSIM blocks <0.9: 1595/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 39.0] pt by ink-projection correlation (centroid estimate [3.16, 33.52] pt); confidence moderate (shift explains 15% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9241, differing 0.014836, SSIM₈ 0.9634 (raw 2.2691, 0.017009, 0.949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9184→0.9441 / 3.6268→2.9379; header-band 1.0→0.9825 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6352→0.7524 / 13.0739→8.01 [275.7,96.4–519.8,156 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 24.19; `paper.` dx -52.18 dy 38.59; `letter` dx -48.63 dy 38.59
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-italic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1912722, 2067, 1802, 1803, 1656, 1491, 1564, 1681, 1413, 1228, 1280, 1187, 1014, 1173, 1124, 5611]`; ink px ref/ours 7447/8456 (ratio 1.1355); SSIM blocks <0.9: 1290/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.5, -46.5] pt by ink-projection correlation (centroid estimate [-5.97, -24.35] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8301, differing 0.014076, SSIM₈ 0.9648 (raw 1.9578, 0.015056, 0.9595)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9352→0.9439 / 3.1291→2.9214; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5767→0.756 / 18.59→10.5645 [275.7,96.4–519.8,156 pt]
- largest word displacements (pt): `twelve` dx 436.31 dy -61.25; `.` dx -82.97 dy -0.37; `paper.` dx -46.41 dy -46.81
- word-sequence differences: delete ref ['x2', '+', 'y2', '=', 'z2'] ours []; delete ref ['1', '2', '+', '1', '5', '=', '3', '6'] ours []

### 09-mixed-document — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908448, 1975, 1818, 1995, 1553, 1434, 1436, 1374, 1435, 1503, 1793, 1550, 1349, 1400, 1470, 8283]`; ink px ref/ours 9857/9252 (ratio 0.9386); SSIM blocks <0.9: 1551/30294; [overlay](images/09-mixed-document/pdflatex-de1020c-export-p1-overlay.png) (55852 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-de1020c-export-p1-heatmap.png) (57386 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 38.5] pt by ink-projection correlation (centroid estimate [7.57, 32.71] pt); confidence moderate (shift explains 24% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8886, differing 0.014393, SSIM₈ 0.9685 (raw 2.4853, 0.017384, 0.9504)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9207→0.9518 / 3.9723→2.8965; header-band 1.0→0.9857 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6361→0.7546 / 15.3285→9.5698 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `twelve` dx -3.05 dy 39.17; `in` dx -2.78 dy 39.17; `set` dx -2.52 dy 39.17
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908171, 1976, 1849, 2001, 1559, 1421, 1424, 1373, 1456, 1524, 1817, 1563, 1351, 1440, 1492, 8399]`; ink px ref/ours 9857/9465 (ratio 0.9602); SSIM blocks <0.9: 1570/30294; [overlay](images/09-mixed-document/pdflatex-main-export-p1-overlay.png) (55795 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-main-export-p1-heatmap.png) (57340 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 38.5] pt by ink-projection correlation (centroid estimate [10.11, 31.38] pt); confidence moderate (shift explains 24% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.911, differing 0.014521, SSIM₈ 0.9679 (raw 2.5141, 0.017545, 0.9497)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9195→0.9512 / 4.0183→2.9169; header-band 1.0→0.9827 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6361→0.7546 / 15.3285→9.5698 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `twelve` dx -3.05 dy 39.17; `in` dx -2.78 dy 39.17; `set` dx -2.52 dy 39.17
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-italic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1912175, 1877, 1678, 1542, 1579, 1378, 1456, 1424, 1447, 1280, 1377, 1313, 1083, 1318, 1301, 6588]`; ink px ref/ours 9857/8456 (ratio 0.8579); SSIM blocks <0.9: 1243/30294; [overlay](images/09-mixed-document/pdflatex-pipeline-export-p1-overlay.png) (54925 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-pipeline-export-p1-heatmap.png) (52509 B, ÷2)
  - registration error (diagnostic): global shift [0.0, -47.0] pt by ink-projection correlation (centroid estimate [0.98, -26.49] pt); confidence strong (shift explains 28% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.5167, differing 0.012699, SSIM₈ 0.9739 (raw 2.1148, 0.015231, 0.9609)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9375→0.9583 / 3.38→2.4241; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5872→0.7087 / 20.4926→12.7911 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `.` dx -51.3 dy -0.13; `paper.` dx 7.3 dy -46.27; `letter` dx 6.39 dy -46.27
- word-sequence differences: delete ref ['x2', '+', 'y2', '=', 'z2'] ours []; delete ref ['1', '2', '+', '1', '5', '=', '3', '6'] ours []

### 09-mixed-document — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909812, 2060, 1849, 1927, 1609, 1602, 1682, 1704, 1540, 1378, 1642, 1427, 1316, 1278, 1363, 6627]`; ink px ref/ours 7573/9252 (ratio 1.2217); SSIM blocks <0.9: 1632/30294; [overlay](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-overlay.png) (56395 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-heatmap.png) (57756 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 39.5] pt by ink-projection correlation (centroid estimate [-1.26, 34.72] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9012, differing 0.014685, SSIM₈ 0.9623 (raw 2.2522, 0.016745, 0.948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9169→0.9418 / 3.5997→2.9167; header-band 1.0→0.9856 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.381→0.5284 / 23.145→12.7296 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 25.83; `paper.` dx -52.2 dy 40.23; `letter` dx -48.65 dy 40.23
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909619, 2067, 1888, 1937, 1626, 1586, 1662, 1710, 1557, 1411, 1651, 1441, 1322, 1309, 1383, 6647]`; ink px ref/ours 7573/9465 (ratio 1.2498); SSIM blocks <0.9: 1647/30294; [overlay](images/09-mixed-document/pdflatex-lm-main-export-p1-overlay.png) (56357 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-main-export-p1-heatmap.png) (57753 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 39.5] pt by ink-projection correlation (centroid estimate [1.27, 33.39] pt); confidence moderate (shift explains 15% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9236, differing 0.014813, SSIM₈ 0.9616 (raw 2.2675, 0.016855, 0.9475)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9161→0.9411 / 3.6241→2.9371; header-band 1.0→0.9826 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.381→0.5284 / 23.145→12.7296 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 25.83; `paper.` dx -52.2 dy 40.23; `letter` dx -48.65 dy 40.23
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-italic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1913041, 1958, 1661, 1490, 1637, 1524, 1711, 1665, 1514, 1171, 1261, 1204, 1073, 1169, 1204, 5533]`; ink px ref/ours 7573/8456 (ratio 1.1166); SSIM blocks <0.9: 1340/30294; [overlay](images/09-mixed-document/pdflatex-lm-pipeline-export-p1-overlay.png) (54548 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-pipeline-export-p1-heatmap.png) (52223 B, ÷2)
  - registration error (diagnostic): global shift [-2.5, -46.0] pt by ink-projection correlation (centroid estimate [-7.86, -24.48] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8344, differing 0.014047, SSIM₈ 0.9644 (raw 1.9585, 0.014905, 0.958)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9329→0.9432 / 3.1302→2.9282; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.2898→0.5286 / 31.3504→20.6358 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `twelve` dx 436.31 dy -59.61; `.` dx -83.03 dy 0.67; `paper.` dx -46.43 dy -45.16
- word-sequence differences: delete ref ['x2', '+', 'y2', '=', 'z2'] ours []; delete ref ['1', '2', '+', '1', '5', '=', '3', '6'] ours []

### 09-mixed-document — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908485, 1993, 1726, 1891, 1568, 1563, 1547, 1422, 1387, 1527, 1642, 1498, 1332, 1389, 1540, 8306]`; ink px ref/ours 9680/9252 (ratio 0.9558); SSIM blocks <0.9: 1553/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 38.5] pt by ink-projection correlation (centroid estimate [7.53, 32.15] pt); confidence strong (shift explains 25% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8537, differing 0.01428, SSIM₈ 0.969 (raw 2.4838, 0.017418, 0.9503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9206→0.9525 / 3.9698→2.8407; header-band 1.0→0.9857 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6356→0.7542 / 15.3342→9.5697 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -2.91 dy 39.06; `in` dx -2.65 dy 39.06; `set` dx -2.4 dy 39.06
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908190, 1999, 1768, 1889, 1580, 1561, 1537, 1420, 1400, 1532, 1656, 1516, 1342, 1430, 1571, 8425]`; ink px ref/ours 9680/9465 (ratio 0.9778); SSIM blocks <0.9: 1572/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 38.5] pt by ink-projection correlation (centroid estimate [10.06, 30.83] pt); confidence strong (shift explains 25% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8761, differing 0.014408, SSIM₈ 0.9684 (raw 2.5135, 0.017582, 0.9496)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9195→0.9519 / 4.0173→2.8611; header-band 1.0→0.9827 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6356→0.7542 / 15.3342→9.5697 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -2.91 dy 39.06; `in` dx -2.65 dy 39.06; `set` dx -2.4 dy 39.06
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-italic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1912242, 1875, 1576, 1496, 1648, 1474, 1540, 1422, 1359, 1315, 1250, 1258, 1072, 1292, 1329, 6668]`; ink px ref/ours 9680/8456 (ratio 0.8736); SSIM blocks <0.9: 1243/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -47.0] pt by ink-projection correlation (centroid estimate [0.93, -27.04] pt); confidence strong (shift explains 30% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.4772, differing 0.012584, SSIM₈ 0.9743 (raw 2.1114, 0.015235, 0.961)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9376→0.9589 / 3.3746→2.361; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.589→0.7082 / 20.3435→12.7845 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `.` dx -51.4 dy -0.05; `paper.` dx 7.32 dy -46.38; `letter` dx 6.4 dy -46.38
- word-sequence differences: delete ref ['x2', '+', 'y2', '=', 'z2'] ours []; delete ref ['1', '2', '+', '1', '5', '=', '3', '6'] ours []

### 09-mixed-document — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909464, 2171, 1943, 2272, 1611, 1596, 1548, 1766, 1439, 1468, 1597, 1418, 1229, 1257, 1310, 6727]`; ink px ref/ours 7446/9252 (ratio 1.2425); SSIM blocks <0.9: 1581/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 39.0] pt by ink-projection correlation (centroid estimate [0.78, 34.82] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9016, differing 0.014705, SSIM₈ 0.9641 (raw 2.2539, 0.016899, 0.9494)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9191→0.9448 / 3.6023→2.9173; header-band 1.0→0.9856 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3967→0.5342 / 23.5725→12.9044 [275.7,121.1–336.2,156 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 25.22; `paper.` dx -52.18 dy 39.62; `letter` dx -48.63 dy 39.62
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909271, 2179, 1982, 2282, 1629, 1577, 1529, 1770, 1456, 1504, 1607, 1430, 1234, 1289, 1330, 6747]`; ink px ref/ours 7446/9465 (ratio 1.2712); SSIM blocks <0.9: 1596/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 39.0] pt by ink-projection correlation (centroid estimate [3.31, 33.5] pt); confidence moderate (shift explains 15% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.924, differing 0.014833, SSIM₈ 0.9634 (raw 2.2692, 0.017009, 0.949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9184→0.9441 / 3.6268→2.9377; header-band 1.0→0.9825 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3967→0.5342 / 23.5725→12.9044 [275.7,121.1–336.2,156 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 25.22; `paper.` dx -52.18 dy 39.62; `letter` dx -48.63 dy 39.62
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-italic.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1912732, 2070, 1777, 1818, 1661, 1474, 1561, 1695, 1410, 1248, 1248, 1203, 1000, 1159, 1143, 5617]`; ink px ref/ours 7446/8456 (ratio 1.1356); SSIM blocks <0.9: 1290/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.5, -46.5] pt by ink-projection correlation (centroid estimate [-5.82, -24.38] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8303, differing 0.014075, SSIM₈ 0.9648 (raw 1.958, 0.015056, 0.9595)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9352→0.9439 / 3.1295→2.9217; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3571→0.5341 / 30.5311→19.9395 [275.7,121.1–336.2,156 pt]
- largest word displacements (pt): `twelve` dx 436.31 dy -60.22; `.` dx -82.97 dy 0.66; `paper.` dx -46.41 dy -45.78
- word-sequence differences: delete ref ['x2', '+', 'y2', '=', 'z2'] ours []; delete ref ['1', '2', '+', '1', '5', '=', '3', '6'] ours []

### 10-unicode-paragraph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893839, 3905, 3475, 2974, 2739, 2765, 2584, 2505, 2344, 2294, 2314, 2129, 2034, 2031, 2138, 8746]`; ink px ref/ours 18767/18449 (ratio 0.9831); SSIM blocks <0.9: 1824/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.58, 1.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3153, differing 0.026129, SSIM₈ 0.949 (raw 3.3153, 0.026129, 0.949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9186→0.9186 / 5.2982→5.2982; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 11.75 Δy 0.0 len 10.0 vs 11.5, thickness px 1 vs 1
- word-sequence differences: replace ref ['Cremebrulee,jalapeno,facade,naive,cooperate,Angstrom,Ærø,Þorr,Øresund,Skoda,Łodz,Is-', 'tanbul,Zurich—anemdash;1990–1995anendash;“curlydoublequotes”,‘curlysinglequotes’,', '«guillemets»,and‚Germanlowquotes‘.', 'SenorMuller’sresumelistsSaoPaulo,Krakow,Reyk-', 'javik,MalmoandBordeaux;thecafe’smenuofferscrepes,souffleandapatethatcosts€12—', 'or£10,¥1500—perserving,1⁄2portionavailable.', 'Sæglopur,manana,Nandu,cedilla,y,ø,a,æ,', 'œuvre,Œdipe,ß,andDzclosetheline.'] ours ['Creme', 'brulee,', 'jalapeno,', 'facade,', 'naive,', 'cooperate,', 'Angstrom,', 'Ærø,']

### 10-unicode-paragraph — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (15): warning: 'Š' (U+0160) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893839, 3905, 3475, 2974, 2739, 2765, 2584, 2505, 2344, 2294, 2314, 2129, 2034, 2031, 2138, 8746]`; ink px ref/ours 18767/18449 (ratio 0.9831); SSIM blocks <0.9: 1824/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.58, 1.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3153, differing 0.026129, SSIM₈ 0.949 (raw 3.3153, 0.026129, 0.949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9186→0.9186 / 5.2982→5.2982; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 11.75 Δy 0.0 len 10.0 vs 11.5, thickness px 1 vs 1
- word-sequence differences: replace ref ['Cremebrulee,jalapeno,facade,naive,cooperate,Angstrom,Ærø,Þorr,Øresund,Skoda,Łodz,Is-', 'tanbul,Zurich—anemdash;1990–1995anendash;“curlydoublequotes”,‘curlysinglequotes’,', '«guillemets»,and‚Germanlowquotes‘.', 'SenorMuller’sresumelistsSaoPaulo,Krakow,Reyk-', 'javik,MalmoandBordeaux;thecafe’smenuofferscrepes,souffleandapatethatcosts€12—', 'or£10,¥1500—perserving,1⁄2portionavailable.', 'Sæglopur,manana,Nandu,cedilla,y,ø,a,æ,', 'œuvre,Œdipe,ß,andDzclosetheline.'] ours ['Creme', 'brulee,', 'jalapeno,', 'facade,', 'naive,', 'cooperate,', 'Angstrom,', 'Ærø,']

### 10-unicode-paragraph — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; warning: U+01C5 'ǅ' has no glyph in Times-Roman; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893467, 3892, 3278, 2904, 2691, 2665, 2440, 2561, 2424, 2356, 2261, 2257, 2132, 2056, 2127, 9305]`; ink px ref/ours 18767/18400 (ratio 0.9804); SSIM blocks <0.9: 1826/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 3.411, not lower; centroid estimate [4.55, 0.32] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.4025, differing 0.026357, SSIM₈ 0.9475 (raw 3.4025, 0.026357, 0.9475)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9163→0.9163 / 5.4352→5.4352; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- word-sequence differences: replace ref ['Cremebrulee,jalapeno,facade,naive,cooperate,Angstrom,Ærø,Þorr,Øresund,Skoda,Łodz,Is-', 'tanbul,Zurich—anemdash;1990–1995anendash;“curlydoublequotes”,‘curlysinglequotes’,', '«guillemets»,and‚Germanlowquotes‘.', 'SenorMuller’sresumelistsSaoPaulo,Krakow,Reyk-', 'javik,MalmoandBordeaux;thecafe’smenuofferscrepes,souffleandapatethatcosts€12—', 'or£10,¥1500—perserving,1⁄2portionavailable.', 'Sæglopur,manana,Nandu,cedilla,y,ø,a,æ,', 'œuvre,Œdipe,ß,andDzclosetheline.'] ours ['Creme', 'brulee,', 'jalapeno,', 'facade,', 'naive,', 'cooperate,', 'Angstrom,', 'Ærø,']

### 10-unicode-paragraph — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894069, 3840, 3317, 2950, 2789, 2858, 2845, 2787, 2685, 2416, 2321, 2100, 2019, 1981, 2036, 7803]`; ink px ref/ours 14211/18449 (ratio 1.2982); SSIM blocks <0.9: 1927/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.22, -0.91] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2356, differing 0.025902, SSIM₈ 0.9447 (raw 3.2356, 0.025902, 0.9447)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9117→0.9117 / 5.1715→5.1715; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 95.5 pt with NO reference rule on this page
- largest word displacements (pt): `that` dx 447.55 dy -14.89; `Łódź,` dx 428.89 dy -14.75; `single` dx 411.29 dy -14.79
- word-sequence differences: replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']; replace ref ['�'] ours ['Dz']

### 10-unicode-paragraph — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (15): warning: 'Š' (U+0160) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894069, 3840, 3317, 2950, 2789, 2858, 2845, 2787, 2685, 2416, 2321, 2100, 2019, 1981, 2036, 7803]`; ink px ref/ours 14211/18449 (ratio 1.2982); SSIM blocks <0.9: 1927/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.22, -0.91] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2356, differing 0.025902, SSIM₈ 0.9447 (raw 3.2356, 0.025902, 0.9447)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9117→0.9117 / 5.1715→5.1715; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 95.5 pt with NO reference rule on this page
- largest word displacements (pt): `that` dx 447.55 dy -14.89; `Łódź,` dx 428.89 dy -14.75; `single` dx 411.29 dy -14.79
- word-sequence differences: replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']; replace ref ['�'] ours ['Dz']

### 10-unicode-paragraph — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; warning: U+01C5 'ǅ' has no glyph in Times-Roman; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894446, 3863, 3242, 2911, 2719, 2816, 2710, 2923, 2703, 2289, 2329, 2273, 1924, 1868, 1921, 7879]`; ink px ref/ours 14211/18400 (ratio 1.2948); SSIM blocks <0.9: 1907/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [3.92, -1.6] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2048, differing 0.025745, SSIM₈ 0.9452 (raw 3.2141, 0.025868, 0.9449)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.912→0.9129 / 5.1347→5.1129; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Łódź,` dx 440.44 dy -14.8; `single` dx 439.44 dy -14.8; `Kraków,` dx 427.7 dy -14.8
- word-sequence differences: delete ref ['�'] ours []

### 10-unicode-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893459, 4089, 3397, 2841, 2581, 2734, 2699, 2550, 2363, 2423, 2285, 2125, 2111, 2043, 2055, 9061]`; ink px ref/ours 18728/18449 (ratio 0.9851); SSIM blocks <0.9: 1812/30294; [overlay](images/10-unicode-paragraph/pdflatex-de1020c-export-p1-overlay.png) (80373 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (62523 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.7, 1.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.362, differing 0.026242, SSIM₈ 0.9486 (raw 3.362, 0.026242, 0.9486)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.918→0.918 / 5.373→5.373; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 12.0 Δy 0.0 len 10.0 vs 12.0, thickness px 1 vs 1
- largest word displacements (pt): `quotes’,` dx -429.81 dy 14.82; `æ,` dx -421.71 dy 14.68; `å,` dx -421.21 dy 14.68
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['C12'] ours ['€12']

### 10-unicode-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (15): warning: 'Š' (U+0160) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893459, 4089, 3397, 2841, 2581, 2734, 2699, 2550, 2363, 2423, 2285, 2125, 2111, 2043, 2055, 9061]`; ink px ref/ours 18728/18449 (ratio 0.9851); SSIM blocks <0.9: 1812/30294; [overlay](images/10-unicode-paragraph/pdflatex-main-export-p1-overlay.png) (80033 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-main-export-p1-heatmap.png) (62203 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.7, 1.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.362, differing 0.026242, SSIM₈ 0.9486 (raw 3.362, 0.026242, 0.9486)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.918→0.918 / 5.373→5.373; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 12.0 Δy 0.0 len 10.0 vs 12.0, thickness px 1 vs 1
- largest word displacements (pt): `quotes’,` dx -429.81 dy 14.82; `æ,` dx -421.71 dy 14.68; `å,` dx -421.21 dy 14.68
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['C12'] ours ['€12']

### 10-unicode-paragraph — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; warning: U+01C5 'ǅ' has no glyph in Times-Roman; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894075, 3918, 3239, 2925, 2587, 2656, 2422, 2569, 2402, 2319, 2257, 2213, 2069, 2013, 2148, 9004]`; ink px ref/ours 18728/18400 (ratio 0.9825); SSIM blocks <0.9: 1796/30294; [overlay](images/10-unicode-paragraph/pdflatex-pipeline-export-p1-overlay.png) (80903 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-pipeline-export-p1-heatmap.png) (62462 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [4.43, 0.45] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3417, differing 0.026117, SSIM₈ 0.9486 (raw 3.3417, 0.026117, 0.9486)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.918→0.918 / 5.3382→5.3382; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `—` dx -434.48 dy 14.85; `ø,` dx -432.72 dy 14.85; `å,` dx -432.34 dy 14.85
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['C12'] ours ['€12']

### 10-unicode-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893882, 3869, 3242, 2959, 2871, 2862, 2862, 2749, 2745, 2384, 2325, 2135, 2070, 1985, 2056, 7820]`; ink px ref/ours 14393/18449 (ratio 1.2818); SSIM blocks <0.9: 1933/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (81389 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (64932 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.76, -1.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.251, differing 0.025995, SSIM₈ 0.9445 (raw 3.251, 0.025995, 0.9445)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9112→0.9112 / 5.1961→5.1961; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 95.5 pt with NO reference rule on this page
- largest word displacements (pt): `that` dx 447.55 dy -14.89; `Łódź,` dx 428.89 dy -14.75; `single` dx 411.29 dy -14.79
- word-sequence differences: replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']

### 10-unicode-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (15): warning: 'Š' (U+0160) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893882, 3869, 3242, 2959, 2871, 2862, 2862, 2749, 2745, 2384, 2325, 2135, 2070, 1985, 2056, 7820]`; ink px ref/ours 14393/18449 (ratio 1.2818); SSIM blocks <0.9: 1933/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-main-export-p1-overlay.png) (81104 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (64657 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.76, -1.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.251, differing 0.025995, SSIM₈ 0.9445 (raw 3.251, 0.025995, 0.9445)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9112→0.9112 / 5.1961→5.1961; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 95.5 pt with NO reference rule on this page
- largest word displacements (pt): `that` dx 447.55 dy -14.89; `Łódź,` dx 428.89 dy -14.75; `single` dx 411.29 dy -14.79
- word-sequence differences: replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']

### 10-unicode-paragraph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; warning: U+01C5 'ǅ' has no glyph in Times-Roman; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894276, 3899, 3196, 2864, 2853, 2791, 2665, 2933, 2786, 2295, 2307, 2246, 2037, 1821, 1929, 7918]`; ink px ref/ours 14393/18400 (ratio 1.2784); SSIM blocks <0.9: 1913/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) (81003 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-pipeline-export-p1-heatmap.png) (64317 B, ÷2)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [3.37, -1.99] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2194, differing 0.025839, SSIM₈ 0.9449 (raw 3.2291, 0.025954, 0.9446)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9116→0.9125 / 5.1587→5.1362; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Łódź,` dx 440.44 dy -14.8; `single` dx 439.44 dy -14.8; `Kraków,` dx 427.7 dy -14.8
- word-sequence differences: delete ref ['Dz'] ours []

### 10-unicode-paragraph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893605, 3862, 3352, 2965, 2706, 2743, 2608, 2494, 2326, 2303, 2340, 2187, 2140, 2037, 2209, 8939]`; ink px ref/ours 18684/18449 (ratio 0.9874); SSIM blocks <0.9: 1829/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.24, 1.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3629, differing 0.026184, SSIM₈ 0.9483 (raw 3.3629, 0.026184, 0.9483)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9175→0.9175 / 5.3742→5.3742; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 11.75 Δy 0.0 len 10.0 vs 11.5, thickness px 1 vs 1
- largest word displacements (pt): `quotes’,` dx -429.86 dy 14.82; `æ,` dx -421.74 dy 14.68; `å,` dx -421.3 dy 14.68
- word-sequence differences: replace ref ['Is-', 'tanbul,'] ours ['Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']

### 10-unicode-paragraph — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (15): warning: 'Š' (U+0160) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893605, 3862, 3352, 2965, 2706, 2743, 2608, 2494, 2326, 2303, 2340, 2187, 2140, 2037, 2209, 8939]`; ink px ref/ours 18684/18449 (ratio 0.9874); SSIM blocks <0.9: 1829/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.24, 1.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3629, differing 0.026184, SSIM₈ 0.9483 (raw 3.3629, 0.026184, 0.9483)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9175→0.9175 / 5.3742→5.3742; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 11.75 Δy 0.0 len 10.0 vs 11.5, thickness px 1 vs 1
- largest word displacements (pt): `quotes’,` dx -429.86 dy 14.82; `æ,` dx -421.74 dy 14.68; `å,` dx -421.3 dy 14.68
- word-sequence differences: replace ref ['Is-', 'tanbul,'] ours ['Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']

### 10-unicode-paragraph — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; warning: U+01C5 'ǅ' has no glyph in Times-Roman; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893719, 3972, 3215, 3017, 2678, 2650, 2382, 2549, 2395, 2201, 2224, 2255, 2182, 1979, 2216, 9182]`; ink px ref/ours 18684/18400 (ratio 0.9848); SSIM blocks <0.9: 1818/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 3.4197, not lower; centroid estimate [4.89, 0.34] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3738, differing 0.026194, SSIM₈ 0.9479 (raw 3.3738, 0.026194, 0.9479)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9169→0.9169 / 5.3893→5.3893; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `—` dx -434.52 dy 14.85; `€12` dx -434.33 dy 14.85; `ø,` dx -432.85 dy 14.85
- word-sequence differences: replace ref ['Is-', 'tanbul,'] ours ['Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; delete ref ['Dz'] ours []

### 10-unicode-paragraph — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894039, 3880, 3308, 2945, 2794, 2860, 2838, 2793, 2672, 2427, 2320, 2102, 2019, 1974, 2036, 7809]`; ink px ref/ours 14217/18449 (ratio 1.2977); SSIM blocks <0.9: 1927/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.66, -0.92] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2358, differing 0.025899, SSIM₈ 0.9447 (raw 3.2358, 0.025899, 0.9447)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9117→0.9117 / 5.1718→5.1718; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 95.5 pt with NO reference rule on this page
- largest word displacements (pt): `that` dx 447.55 dy -13.86; `Łódź,` dx 428.89 dy -13.72; `single` dx 411.29 dy -13.77
- word-sequence differences: replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']; replace ref ['\uffff'] ours ['Dz']

### 10-unicode-paragraph — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (15): warning: 'Š' (U+0160) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894039, 3880, 3308, 2945, 2794, 2860, 2838, 2793, 2672, 2427, 2320, 2102, 2019, 1974, 2036, 7809]`; ink px ref/ours 14217/18449 (ratio 1.2977); SSIM blocks <0.9: 1927/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.66, -0.92] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2358, differing 0.025899, SSIM₈ 0.9447 (raw 3.2358, 0.025899, 0.9447)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9117→0.9117 / 5.1718→5.1718; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 95.5 pt with NO reference rule on this page
- largest word displacements (pt): `that` dx 447.55 dy -13.86; `Łódź,` dx 428.89 dy -13.72; `single` dx 411.29 dy -13.77
- word-sequence differences: replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']; replace ref ['\uffff'] ours ['Dz']

### 10-unicode-paragraph — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; warning: U+01C5 'ǅ' has no glyph in Times-Roman; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894432, 3892, 3229, 2916, 2719, 2812, 2698, 2925, 2698, 2307, 2323, 2269, 1933, 1862, 1923, 7878]`; ink px ref/ours 14217/18400 (ratio 1.2942); SSIM blocks <0.9: 1907/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [3.47, -1.61] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2049, differing 0.025739, SSIM₈ 0.9452 (raw 3.2139, 0.025865, 0.9449)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.912→0.9129 / 5.1344→5.113; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Łódź,` dx 440.44 dy -13.78; `single` dx 439.44 dy -13.78; `Kraków,` dx 427.7 dy -13.78
- word-sequence differences: delete ref ['\uffff'] ours []

### 11-nested-lists — lualatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920945, 1355, 1097, 945, 846, 1093, 822, 796, 797, 678, 928, 902, 872, 877, 856, 5007]`; ink px ref/ours 6059/5958 (ratio 0.9833); SSIM blocks <0.9: 999/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [18.0, -49.0] pt by ink-projection correlation (centroid estimate [101.05, -62.24] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.4605, differing 0.010223, SSIM₈ 0.9709 (raw 1.4653, 0.010385, 0.9696)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9514→0.9548 / 2.342→2.2656; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4507→0.3438 / 22.2877→28.6457 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `numbered` dx 307.89 dy -72.76; `First` dx 307.79 dy -72.76; `list.` dx 272.83 dy -126.61
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '•' (U+2022) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920669, 1354, 1148, 1052, 886, 1052, 931, 853, 833, 794, 984, 888, 789, 825, 823, 4935]`; ink px ref/ours 6059/6125 (ratio 1.0109); SSIM blocks <0.9: 942/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-21.0, -8.0] pt by ink-projection correlation (centroid estimate [-24.95, -9.81] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.219, differing 0.009203, SSIM₈ 0.9785 (raw 1.4651, 0.010461, 0.97)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.952→0.9656 / 2.3417→1.9483; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.2153→0.2509 / 35.1224→36.416 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `First` dx -44.19 dy -11.56; `numbered` dx -44.09 dy -11.56; `Second` dx -44.19 dy -10.59

### 11-nested-lists — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (8): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920608, 1233, 1057, 1021, 823, 945, 912, 863, 974, 824, 896, 851, 953, 909, 991, 4956]`; ink px ref/ours 6059/6009 (ratio 0.9917); SSIM blocks <0.9: 998/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [26.5, -49.0] pt by ink-projection correlation (centroid estimate [112.74, -62.21] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.4442, differing 0.010139, SSIM₈ 0.9706 (raw 1.5034, 0.010458, 0.9691)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9505→0.9549 / 2.4028→2.2114; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4507→0.3433 / 22.2877→28.312 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `numbered` dx 339.09 dy -72.82; `First` dx 337.11 dy -72.82; `list.` dx 283.52 dy -126.62
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921444, 1232, 1126, 1001, 890, 1148, 910, 879, 962, 748, 869, 1022, 949, 799, 793, 4044]`; ink px ref/ours 4803/5958 (ratio 1.2405); SSIM blocks <0.9: 1044/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.0, -49.0] pt by ink-projection correlation (centroid estimate [99.1, -62.16] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3067, differing 0.00967, SSIM₈ 0.9711 (raw 1.3675, 0.010034, 0.9687)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.95→0.9538 / 2.1857→2.0885; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6066→0.4338 / 14.6679→25.5897 [109.1,139.7–273.7,184 pt]
- largest word displacements (pt): `First` dx 307.79 dy -73.53; `numbered` dx 304.29 dy -73.53; `After` dx 272.65 dy -127.37
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '•' (U+2022) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921121, 1220, 1183, 1117, 943, 1107, 1020, 942, 979, 867, 922, 993, 867, 749, 760, 4026]`; ink px ref/ours 4803/6125 (ratio 1.2752); SSIM blocks <0.9: 994/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-29.0, -8.0] pt by ink-projection correlation (centroid estimate [-26.9, -9.74] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2485, differing 0.009372, SSIM₈ 0.9756 (raw 1.3731, 0.010139, 0.9689)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9503→0.9609 / 2.1947→1.9954; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3446→0.4841 / 30.8097→27.9932 [109.1,139.7–273.7,184 pt]
- largest word displacements (pt): `sub-item.` dx -51.0 dy -12.33; `sub-item.` dx -48.84 dy -11.36; `numbered` dx -47.69 dy -12.33

### 11-nested-lists — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (8): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921541, 1138, 1118, 1082, 876, 1003, 998, 901, 1089, 873, 791, 935, 1016, 812, 896, 3747]`; ink px ref/ours 4803/6009 (ratio 1.2511); SSIM blocks <0.9: 1024/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [54.0, -49.0] pt by ink-projection correlation (centroid estimate [110.8, -62.13] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3121, differing 0.009684, SSIM₈ 0.9705 (raw 1.3526, 0.009914, 0.969)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9504→0.9564 / 2.1618→1.9278; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6066→0.4328 / 14.6679→25.7153 [109.1,139.7–273.7,184 pt]
- largest word displacements (pt): `First` dx 337.11 dy -73.58; `numbered` dx 335.49 dy -73.58; `After` dx 281.69 dy -127.38
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920969, 1439, 1023, 930, 793, 1108, 857, 760, 794, 725, 868, 911, 886, 817, 931, 5005]`; ink px ref/ours 6062/5958 (ratio 0.9828); SSIM blocks <0.9: 995/30294; [overlay](images/11-nested-lists/pdflatex-de1020c-export-p1-overlay.png) (44933 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-de1020c-export-p1-heatmap.png) (46001 B, ÷2)
  - registration error (diagnostic): global shift [19.0, -49.0] pt by ink-projection correlation (centroid estimate [102.36, -62.51] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.4558, differing 0.010208, SSIM₈ 0.9711 (raw 1.4657, 0.01038, 0.9698)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9518→0.9553 / 2.3427→2.2545; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.506→0.3995 / 22.5175→28.0218 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `numbered` dx 309.05 dy -72.76; `First` dx 308.96 dy -72.76; `list.` dx 272.83 dy -126.61
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '•' (U+2022) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920641, 1429, 1080, 1041, 841, 1072, 967, 816, 809, 843, 926, 899, 807, 772, 916, 4957]`; ink px ref/ours 6062/6125 (ratio 1.0104); SSIM blocks <0.9: 935/30294; [overlay](images/11-nested-lists/pdflatex-main-export-p1-overlay.png) (45705 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-main-export-p1-heatmap.png) (46238 B, ÷2)
  - registration error (diagnostic): global shift [-20.0, -8.0] pt by ink-projection correlation (centroid estimate [-23.64, -10.08] pt); confidence moderate (shift explains 19% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1988, differing 0.009175, SSIM₈ 0.9789 (raw 1.4717, 0.010475, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9523→0.9663 / 2.3521→1.916; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.2523→0.34 / 34.7601→36.8114 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `First` dx -43.02 dy -11.56; `numbered` dx -42.93 dy -11.56; `Second` dx -43.02 dy -10.59

### 11-nested-lists — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (8): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920772, 1273, 988, 974, 763, 982, 923, 802, 940, 890, 811, 864, 960, 860, 1064, 4950]`; ink px ref/ours 6062/6009 (ratio 0.9913); SSIM blocks <0.9: 984/30294; [overlay](images/11-nested-lists/pdflatex-pipeline-export-p1-overlay.png) (45263 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-pipeline-export-p1-heatmap.png) (46331 B, ÷2)
  - registration error (diagnostic): global shift [53.0, -49.0] pt by ink-projection correlation (centroid estimate [114.05, -62.48] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.4609, differing 0.010163, SSIM₈ 0.971 (raw 1.4978, 0.01043, 0.9693)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.951→0.957 / 2.3939→2.1677; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.506→0.4024 / 22.5175→27.7862 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `numbered` dx 340.25 dy -72.82; `First` dx 338.27 dy -72.82; `list.` dx 283.51 dy -126.62
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921429, 1208, 1106, 1054, 881, 1201, 884, 878, 1059, 686, 919, 983, 802, 823, 761, 4142]`; ink px ref/ours 4790/5958 (ratio 1.2438); SSIM blocks <0.9: 1048/30294; [overlay](images/11-nested-lists/pdflatex-lm-de1020c-export-p1-overlay.png) (45461 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-de1020c-export-p1-heatmap.png) (45613 B, ÷2)
  - registration error (diagnostic): global shift [23.5, -49.0] pt by ink-projection correlation (centroid estimate [101.54, -62.0] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3216, differing 0.009723, SSIM₈ 0.9706 (raw 1.3674, 0.010055, 0.9687)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.95→0.9547 / 2.1855→2.0271; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6045→0.4278 / 14.6672→25.9327 [108.1,139.7–272.5,184 pt]
- largest word displacements (pt): `First` dx 308.96 dy -73.53; `numbered` dx 305.46 dy -73.53; `After` dx 272.65 dy -127.37
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '•' (U+2022) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921134, 1212, 1169, 1161, 937, 1162, 1004, 946, 1071, 802, 959, 953, 721, 766, 730, 4089]`; ink px ref/ours 4790/6125 (ratio 1.2787); SSIM blocks <0.9: 995/30294; [overlay](images/11-nested-lists/pdflatex-lm-main-export-p1-overlay.png) (46381 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-main-export-p1-heatmap.png) (46203 B, ÷2)
  - registration error (diagnostic): global shift [-29.0, -8.0] pt by ink-projection correlation (centroid estimate [-24.46, -9.57] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2361, differing 0.009335, SSIM₈ 0.9758 (raw 1.3674, 0.010152, 0.969)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9505→0.9612 / 2.1855→1.9757; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3409→0.4892 / 30.9535→27.9135 [108.1,139.7–272.5,184 pt]
- largest word displacements (pt): `sub-item.` dx -49.84 dy -12.33; `sub-item.` dx -47.67 dy -11.36; `numbered` dx -46.52 dy -12.33

### 11-nested-lists — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (8): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921522, 1109, 1110, 1143, 860, 1054, 974, 899, 1188, 806, 841, 899, 862, 839, 863, 3847]`; ink px ref/ours 4790/6009 (ratio 1.2545); SSIM blocks <0.9: 1028/30294; [overlay](images/11-nested-lists/pdflatex-lm-pipeline-export-p1-overlay.png) (45743 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-pipeline-export-p1-heatmap.png) (45797 B, ÷2)
  - registration error (diagnostic): global shift [52.0, -49.0] pt by ink-projection correlation (centroid estimate [113.23, -61.97] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3046, differing 0.009658, SSIM₈ 0.9706 (raw 1.3521, 0.009934, 0.969)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9505→0.9564 / 2.161→1.9202; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6045→0.4314 / 14.6672→25.6563 [108.1,139.7–272.5,184 pt]
- largest word displacements (pt): `First` dx 338.27 dy -73.58; `numbered` dx 336.66 dy -73.58; `After` dx 281.69 dy -127.38
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — xelatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920921, 1399, 1106, 932, 854, 1068, 813, 810, 810, 650, 914, 907, 884, 897, 861, 4990]`; ink px ref/ours 6057/5958 (ratio 0.9837); SSIM blocks <0.9: 999/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [18.0, -49.0] pt by ink-projection correlation (centroid estimate [101.1, -62.21] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.46, differing 0.010222, SSIM₈ 0.9709 (raw 1.4648, 0.010387, 0.9696)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9514→0.9548 / 2.3412→2.2648; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4507→0.3437 / 22.2161→28.5578 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `numbered` dx 307.89 dy -72.76; `First` dx 307.79 dy -72.76; `list.` dx 272.83 dy -126.61
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '•' (U+2022) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920649, 1393, 1157, 1041, 889, 1033, 919, 864, 838, 779, 966, 899, 796, 849, 819, 4925]`; ink px ref/ours 6057/6125 (ratio 1.0112); SSIM blocks <0.9: 942/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-21.0, -8.0] pt by ink-projection correlation (centroid estimate [-24.9, -9.78] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2177, differing 0.0092, SSIM₈ 0.9785 (raw 1.4648, 0.010464, 0.9699)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.952→0.9657 / 2.3411→1.9463; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.2151→0.2507 / 35.01→36.3016 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `First` dx -44.19 dy -11.56; `numbered` dx -44.09 dy -11.56; `Second` dx -44.19 dy -10.59

### 11-nested-lists — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (8): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920583, 1257, 1072, 1016, 837, 924, 900, 873, 977, 811, 880, 856, 966, 927, 999, 4938]`; ink px ref/ours 6057/6009 (ratio 0.9921); SSIM blocks <0.9: 998/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [52.0, -49.0] pt by ink-projection correlation (centroid estimate [112.79, -62.18] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.4569, differing 0.010166, SSIM₈ 0.9706 (raw 1.5034, 0.010459, 0.969)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9505→0.9564 / 2.4028→2.1637; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4507→0.3461 / 22.2161→28.2963 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `numbered` dx 339.09 dy -72.82; `First` dx 337.11 dy -72.82; `list.` dx 283.51 dy -126.62
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921432, 1237, 1125, 1009, 893, 1121, 930, 896, 951, 735, 882, 1028, 939, 802, 786, 4050]`; ink px ref/ours 4790/5958 (ratio 1.2438); SSIM blocks <0.9: 1044/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.0, -49.0] pt by ink-projection correlation (centroid estimate [98.92, -62.23] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3068, differing 0.009665, SSIM₈ 0.9711 (raw 1.3675, 0.010029, 0.9687)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.95→0.9538 / 2.1857→2.0886; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5188→0.4109 / 16.5479→24.3869 [109.1,143.5–273.7,182.9 pt]
- largest word displacements (pt): `First` dx 307.8 dy -72.5; `numbered` dx 304.3 dy -72.5; `After` dx 272.65 dy -126.34
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '•' (U+2022) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921108, 1226, 1183, 1126, 942, 1082, 1040, 958, 970, 856, 933, 999, 857, 750, 750, 4036]`; ink px ref/ours 4790/6125 (ratio 1.2787); SSIM blocks <0.9: 994/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-29.0, -8.0] pt by ink-projection correlation (centroid estimate [-27.08, -9.8] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2487, differing 0.00937, SSIM₈ 0.9756 (raw 1.373, 0.010132, 0.9689)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9503→0.9609 / 2.1945→1.9958; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3001→0.3695 / 29.444→31.5736 [109.1,143.5–273.7,182.9 pt]
- largest word displacements (pt): `sub-item.` dx -50.99 dy -11.3; `sub-item.` dx -48.83 dy -10.33; `numbered` dx -47.68 dy -11.3

### 11-nested-lists — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (8): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921529, 1143, 1117, 1090, 879, 976, 1018, 918, 1078, 860, 804, 941, 1006, 815, 889, 3753]`; ink px ref/ours 4790/6009 (ratio 1.2545); SSIM blocks <0.9: 1024/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [54.0, -49.0] pt by ink-projection correlation (centroid estimate [110.62, -62.2] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.312, differing 0.00968, SSIM₈ 0.9705 (raw 1.3526, 0.009909, 0.969)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9504→0.9564 / 2.1618→1.9277; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5188→0.411 / 16.5479→24.2998 [109.1,143.5–273.7,182.9 pt]
- largest word displacements (pt): `First` dx 337.12 dy -72.56; `numbered` dx 335.5 dy -72.56; `After` dx 281.69 dy -126.35
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 12-justified-paragraphs — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1744402, 14700, 12306, 11286, 10602, 10260, 11020, 9359, 9682, 10331, 9550, 9486, 9124, 9371, 9695, 47642]`; ink px ref/ours 67719/67475 (ratio 0.9964); SSIM blocks <0.9: 8260/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.58, 32.29] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.3872, differing 0.111127, SSIM₈ 0.7397 (raw 15.4038, 0.111111, 0.7396)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5838→0.584 / 24.6198→24.5932; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 75.37; `branch` dx -435.47 dy 55.2; `branch` dx -435.47 dy 35.03

### 12-justified-paragraphs — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1744402, 14700, 12306, 11286, 10602, 10260, 11020, 9359, 9682, 10331, 9550, 9486, 9124, 9371, 9695, 47642]`; ink px ref/ours 67719/67475 (ratio 0.9964); SSIM blocks <0.9: 8260/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.58, 32.29] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.3872, differing 0.111127, SSIM₈ 0.7397 (raw 15.4038, 0.111111, 0.7396)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5838→0.584 / 24.6198→24.5932; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 75.37; `branch` dx -435.47 dy 55.2; `branch` dx -435.47 dy 35.03

### 12-justified-paragraphs — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1766711, 14410, 12535, 10677, 9994, 9417, 9209, 8982, 8994, 9057, 8268, 8364, 8132, 8157, 8058, 37851]`; ink px ref/ours 67719/67524 (ratio 0.9971); SSIM blocks <0.9: 7245/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 14.5] pt by ink-projection correlation (centroid estimate [2.77, 23.92] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.0045, differing 0.098968, SSIM₈ 0.792 (raw 13.1323, 0.099297, 0.7893)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6636→0.6815 / 20.9862→20.0779; header-band 1.0→0.9089 / 0.0→4.8172; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 58.19; `branch` dx -435.47 dy 43.74; `branch` dx -435.47 dy 29.3

### 12-justified-paragraphs — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755357, 14133, 12350, 11646, 11189, 10634, 11430, 11055, 10405, 10568, 8991, 8890, 8563, 8103, 8348, 37154]`; ink px ref/ours 51487/67475 (ratio 1.3105); SSIM blocks <0.9: 7892/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-8.5, 2.5] pt by ink-projection correlation (centroid estimate [-6.5, 6.98] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7712, differing 0.105152, SSIM₈ 0.7503 (raw 13.7995, 0.105157, 0.7503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6011→0.6011 / 22.0545→22.0094; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -14.75; `oak` dx 426.94 dy -9.02; `oak` dx 426.94 dy -3.3

### 12-justified-paragraphs — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755357, 14133, 12350, 11646, 11189, 10634, 11430, 11055, 10405, 10568, 8991, 8890, 8563, 8103, 8348, 37154]`; ink px ref/ours 51487/67475 (ratio 1.3105); SSIM blocks <0.9: 7892/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-8.5, 2.5] pt by ink-projection correlation (centroid estimate [-6.5, 6.98] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7712, differing 0.105152, SSIM₈ 0.7503 (raw 13.7995, 0.105157, 0.7503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6011→0.6011 / 22.0545→22.0094; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -14.75; `oak` dx 426.94 dy -9.02; `oak` dx 426.94 dy -3.3

### 12-justified-paragraphs — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1778197, 13498, 11984, 11086, 10548, 9701, 9884, 10052, 9497, 9002, 7655, 7532, 7317, 7240, 6943, 28680]`; ink px ref/ours 51487/67524 (ratio 1.3115); SSIM blocks <0.9: 6684/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [3.85, -1.39] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.5477, differing 0.092779, SSIM₈ 0.8076 (raw 11.6358, 0.093075, 0.8076)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6932→0.6952 / 18.5934→18.4267; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -14.8; `oak` dx 450.74 dy -14.8; `oak` dx 450.74 dy -14.8

### 12-justified-paragraphs — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1744770, 14653, 12288, 11028, 10978, 10548, 10474, 9663, 9503, 10351, 9533, 9493, 9214, 8786, 9639, 47895]`; ink px ref/ours 67213/67475 (ratio 1.0039); SSIM blocks <0.9: 8223/30294; [overlay](images/12-justified-paragraphs/pdflatex-de1020c-export-p1-overlay.png) (83109 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-de1020c-export-p1-heatmap.png) (63780 B, ÷4)
  - registration error (diagnostic): global shift [-3.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.27, 32.27] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.3659, differing 0.110805, SSIM₈ 0.7414 (raw 15.3708, 0.11076, 0.7415)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5868→0.5868 / 24.5671→24.5592; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 75.37; `branch` dx -435.47 dy 55.2; `branch` dx -435.47 dy 35.03

### 12-justified-paragraphs — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1744770, 14653, 12288, 11028, 10978, 10548, 10474, 9663, 9503, 10351, 9533, 9493, 9214, 8786, 9639, 47895]`; ink px ref/ours 67213/67475 (ratio 1.0039); SSIM blocks <0.9: 8223/30294; [overlay](images/12-justified-paragraphs/pdflatex-main-export-p1-overlay.png) (83048 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-main-export-p1-heatmap.png) (63713 B, ÷4)
  - registration error (diagnostic): global shift [-3.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.27, 32.27] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.3659, differing 0.110805, SSIM₈ 0.7414 (raw 15.3708, 0.11076, 0.7415)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5868→0.5868 / 24.5671→24.5592; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 75.37; `branch` dx -435.47 dy 55.2; `branch` dx -435.47 dy 35.03

### 12-justified-paragraphs — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1767672, 14355, 12471, 10394, 10260, 9793, 8804, 9150, 8657, 8823, 8281, 8431, 8075, 7975, 8108, 37567]`; ink px ref/ours 67213/67524 (ratio 1.0046); SSIM blocks <0.9: 7182/30294; [overlay](images/12-justified-paragraphs/pdflatex-pipeline-export-p1-overlay.png) (82021 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-pipeline-export-p1-heatmap.png) (57249 B, ÷4)
  - registration error (diagnostic): global shift [-0.5, 14.5] pt by ink-projection correlation (centroid estimate [3.07, 23.9] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 12.8874, differing 0.09821, SSIM₈ 0.7948 (raw 13.0472, 0.098742, 0.7918)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6676→0.686 / 20.8502→19.8908; header-band 1.0→0.9089 / 0.0→4.8172; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 58.19; `branch` dx -435.47 dy 43.74; `branch` dx -435.47 dy 29.3

### 12-justified-paragraphs — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755333, 14131, 12262, 11805, 11161, 10699, 11419, 10985, 10442, 10508, 9029, 8874, 8613, 7950, 8550, 37055]`; ink px ref/ours 51535/67475 (ratio 1.3093); SSIM blocks <0.9: 7894/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-de1020c-export-p1-overlay.png) (84654 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-de1020c-export-p1-heatmap.png) (62808 B, ÷4)
  - registration error (diagnostic): global shift [-8.5, 2.5] pt by ink-projection correlation (centroid estimate [-6.35, 7.1] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7701, differing 0.105126, SSIM₈ 0.7503 (raw 13.7992, 0.105148, 0.7503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6012→0.6012 / 22.0542→22.0076; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.0; `oak` dx 426.94 dy 3.45

### 12-justified-paragraphs — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755333, 14131, 12262, 11805, 11161, 10699, 11419, 10985, 10442, 10508, 9029, 8874, 8613, 7950, 8550, 37055]`; ink px ref/ours 51535/67475 (ratio 1.3093); SSIM blocks <0.9: 7894/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-main-export-p1-overlay.png) (84574 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-main-export-p1-heatmap.png) (62721 B, ÷4)
  - registration error (diagnostic): global shift [-8.5, 2.5] pt by ink-projection correlation (centroid estimate [-6.35, 7.1] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7701, differing 0.105126, SSIM₈ 0.7503 (raw 13.7992, 0.105148, 0.7503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6012→0.6012 / 22.0542→22.0076; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.0; `oak` dx 426.94 dy 3.45

### 12-justified-paragraphs — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1778247, 13488, 11854, 11134, 10712, 9576, 9886, 10088, 9514, 8857, 7825, 7451, 7348, 7154, 7091, 28591]`; ink px ref/ours 51535/67524 (ratio 1.3103); SSIM blocks <0.9: 6687/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-pipeline-export-p1-overlay.png) (82086 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-pipeline-export-p1-heatmap.png) (54509 B, ÷4)
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [4.0, -1.27] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.5402, differing 0.09277, SSIM₈ 0.8076 (raw 11.6351, 0.09305, 0.8076)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6932→0.6953 / 18.5922→18.4148; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -13.78; `oak` dx 450.74 dy -13.78; `oak` dx 450.74 dy -13.78

### 12-justified-paragraphs — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1744655, 14295, 12346, 11375, 10598, 10442, 10890, 9620, 9798, 10256, 9531, 9562, 9258, 9139, 9507, 47544]`; ink px ref/ours 67565/67475 (ratio 0.9987); SSIM blocks <0.9: 8255/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 15.3833, not lower; centroid estimate [-6.26, 32.52] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.3809, differing 0.111072, SSIM₈ 0.7404 (raw 15.3809, 0.111072, 0.7404)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5852→0.5852 / 24.5832→24.5832; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 75.37; `branch` dx -435.52 dy 55.2; `branch` dx -435.52 dy 35.03

### 12-justified-paragraphs — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1744655, 14295, 12346, 11375, 10598, 10442, 10890, 9620, 9798, 10256, 9531, 9562, 9258, 9139, 9507, 47544]`; ink px ref/ours 67565/67475 (ratio 0.9987); SSIM blocks <0.9: 8255/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 15.3833, not lower; centroid estimate [-6.26, 32.52] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.3809, differing 0.111072, SSIM₈ 0.7404 (raw 15.3809, 0.111072, 0.7404)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5852→0.5852 / 24.5832→24.5832; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 75.37; `branch` dx -435.52 dy 55.2; `branch` dx -435.52 dy 35.03

### 12-justified-paragraphs — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1766434, 14060, 12509, 10915, 10217, 9704, 9246, 9017, 8991, 8998, 8423, 8381, 8145, 8065, 7920, 37791]`; ink px ref/ours 67565/67524 (ratio 0.9994); SSIM blocks <0.9: 7232/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 14.5] pt by ink-projection correlation (centroid estimate [4.09, 24.15] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.0398, differing 0.098972, SSIM₈ 0.7918 (raw 13.1381, 0.099473, 0.7892)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6636→0.6812 / 20.9955→20.1342; header-band 1.0→0.9089 / 0.0→4.8172; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 58.19; `branch` dx -435.52 dy 43.74; `branch` dx -435.52 dy 29.3

### 12-justified-paragraphs — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755364, 14133, 12378, 11631, 11131, 10674, 11450, 11022, 10428, 10570, 8986, 8886, 8529, 8142, 8340, 37152]`; ink px ref/ours 51522/67475 (ratio 1.3096); SSIM blocks <0.9: 7895/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-8.5, 2.5] pt by ink-projection correlation (centroid estimate [-6.61, 7.02] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7708, differing 0.105112, SSIM₈ 0.7503 (raw 13.7994, 0.105112, 0.7503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6011→0.6011 / 22.0544→22.0087; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.0; `oak` dx 426.94 dy 3.45

### 12-justified-paragraphs — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755364, 14133, 12378, 11631, 11131, 10674, 11450, 11022, 10428, 10570, 8986, 8886, 8529, 8142, 8340, 37152]`; ink px ref/ours 51522/67475 (ratio 1.3096); SSIM blocks <0.9: 7895/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-8.5, 2.5] pt by ink-projection correlation (centroid estimate [-6.61, 7.02] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7708, differing 0.105112, SSIM₈ 0.7503 (raw 13.7994, 0.105112, 0.7503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6011→0.6011 / 22.0544→22.0087; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.0; `oak` dx 426.94 dy 3.45

### 12-justified-paragraphs — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1778194, 13507, 12003, 11089, 10499, 9705, 9930, 10025, 9506, 8991, 7678, 7522, 7286, 7261, 6936, 28684]`; ink px ref/ours 51522/67524 (ratio 1.3106); SSIM blocks <0.9: 6686/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [3.74, -1.35] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.5475, differing 0.092773, SSIM₈ 0.8076 (raw 11.6355, 0.093029, 0.8076)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6932→0.6952 / 18.5929→18.4264; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -13.78; `oak` dx 450.74 dy -13.78; `oak` dx 450.74 dy -13.78

### 13-math-display-rich — lualatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1932109, 636, 516, 481, 385, 417, 581, 452, 336, 325, 292, 257, 284, 309, 277, 1159]`; ink px ref/ours 2479/2605 (ratio 1.0508); SSIM blocks <0.9: 323/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-0.19, 2.65] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4325, differing 0.003871, SSIM₈ 0.9914 (raw 0.4724, 0.004014, 0.9903)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9845→0.9863 / 0.755→0.6913; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.402→0.3954 / 25.1108→24.6697 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -66.25 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2; Δx 18.75 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `display:` dx 0.1 dy -10.41; `Rich` dx 0 dy -10.41; `√x` dx -7.41 dy 4.97
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (3): error: \left is not supported in math mode; error: \right is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931942, 644, 537, 489, 400, 425, 588, 461, 346, 331, 300, 260, 290, 314, 289, 1200]`; ink px ref/ours 2479/2696 (ratio 1.0875); SSIM blocks <0.9: 334/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [10.67, 2.48] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4453, differing 0.003967, SSIM₈ 0.9911 (raw 0.4851, 0.004109, 0.9899)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9839→0.9857 / 0.7754→0.7118; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.402→0.3954 / 25.1108→24.6697 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -66.25 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2; Δx 18.75 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `display:` dx 0.1 dy -10.41; `Rich` dx 0 dy -10.41; `√x` dx -7.41 dy 4.97
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: \left is not supported in math mode; error: \right is not supported in math mode; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933042, 449, 335, 385, 262, 329, 564, 277, 289, 233, 279, 265, 296, 307, 286, 1218]`; ink px ref/ours 2479/1360 (ratio 0.5486); SSIM blocks <0.9: 332/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.0, -36.0] pt REJECTED: applying it gives mean|Δ| 0.4563, not lower; centroid estimate [-82.51, -17.61] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4416, differing 0.003395, SSIM₈ 0.9896 (raw 0.4416, 0.003395, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9834→0.9834 / 0.7058→0.7058; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5241→0.5241 / 13.8217→13.8217 [232.8,85.1–384.7,123.2 pt]
- largest word displacements (pt): `display.` dx 5.64 dy -35.63; `the` dx 4.73 dy -35.63; `Text` dx 3.9 dy -35.63
- word-sequence differences: delete ref ['1', '0'] ours []; delete ref ['√x', '1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k'] ours []

### 13-math-display-rich — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931240, 712, 529, 432, 438, 438, 472, 553, 438, 442, 462, 316, 320, 342, 358, 1324]`; ink px ref/ours 2285/2605 (ratio 1.14); SSIM blocks <0.9: 363/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.0, 4.0] pt by ink-projection correlation (centroid estimate [-21.76, 3.05] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5477, differing 0.004333, SSIM₈ 0.9892 (raw 0.5515, 0.004325, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.982→0.9828 / 0.8815→0.875; header-band 1.0→0.9992 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4003→0.4082 / 24.7221→24.3784 [232.8,84.9–384.7,123.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -11.5 Δy 4.0 len 21.0 vs 31.0, thickness px 1 vs 1; Δx -16.75 Δy 4.0 len 21.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `display:` dx -1.48 dy -10.27; `Rich` dx 0 dy -10.27; `√x` dx -7.41 dy 5.11
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (3): error: \left is not supported in math mode; error: \right is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931073, 720, 550, 440, 453, 446, 479, 562, 448, 448, 470, 319, 326, 347, 370, 1365]`; ink px ref/ours 2285/2696 (ratio 1.1799); SSIM blocks <0.9: 374/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.0, 4.0] pt by ink-projection correlation (centroid estimate [-10.89, 2.89] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5604, differing 0.004428, SSIM₈ 0.9888 (raw 0.5643, 0.00442, 0.9884)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9814→0.9824 / 0.902→0.8905; header-band 1.0→0.9992 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4003→0.4082 / 24.7221→24.3784 [232.8,84.9–384.7,123.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -11.5 Δy 4.0 len 21.0 vs 31.0, thickness px 1 vs 1; Δx -16.75 Δy 4.0 len 21.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `display:` dx -1.48 dy -10.27; `Rich` dx 0 dy -10.27; `√x` dx -7.41 dy 5.11
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: \left is not supported in math mode; error: \right is not supported in math mode; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1932804, 549, 382, 394, 336, 315, 368, 339, 315, 344, 446, 301, 285, 275, 296, 1067]`; ink px ref/ours 2285/1360 (ratio 0.5952); SSIM blocks <0.9: 350/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -36.0] pt by ink-projection correlation (centroid estimate [-104.07, -17.21] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4292, differing 0.003347, SSIM₈ 0.9899 (raw 0.4463, 0.003445, 0.989)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9824→0.9838 / 0.7134→0.6861; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5241→0.5241 / 13.647→13.647 [232.8,84.9–384.7,123.1 pt]
- largest word displacements (pt): `Text` dx 3.9 dy -36.37; `display.` dx -2.09 dy -36.37; `after` dx 1.4 dy -36.37
- word-sequence differences: delete ref ['1', '0'] ours []; delete ref ['√x', '1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k'] ours []

### 13-math-display-rich — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931825, 678, 566, 390, 402, 560, 481, 438, 376, 334, 290, 257, 295, 302, 330, 1292]`; ink px ref/ours 2470/2605 (ratio 1.0547); SSIM blocks <0.9: 320/30294; [overlay](images/13-math-display-rich/pdflatex-de1020c-export-p1-overlay.png) (85256 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-de1020c-export-p1-heatmap.png) (83491 B, ÷1)
  - registration error (diagnostic): global shift [1.0, 3.5] pt by ink-projection correlation (centroid estimate [0.39, 2.75] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4808, differing 0.004039, SSIM₈ 0.9907 (raw 0.501, 0.004125, 0.9899)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9839→0.9853 / 0.8007→0.767; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4017→0.3792 / 25.1214→25.5101 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -66.25 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2; Δx 18.75 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `display:` dx 0.1 dy -10.42; `Rich` dx 0 dy -10.42; `√x` dx -7.41 dy 4.96
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (3): error: \left is not supported in math mode; error: \right is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931658, 686, 587, 398, 417, 568, 488, 447, 386, 340, 298, 260, 301, 307, 342, 1333]`; ink px ref/ours 2470/2696 (ratio 1.0915); SSIM blocks <0.9: 331/30294; [overlay](images/13-math-display-rich/pdflatex-main-export-p1-overlay.png) (85792 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-main-export-p1-heatmap.png) (84157 B, ÷1)
  - registration error (diagnostic): global shift [1.0, 3.5] pt by ink-projection correlation (centroid estimate [11.26, 2.59] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4936, differing 0.004134, SSIM₈ 0.9903 (raw 0.5138, 0.004221, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9833→0.9847 / 0.8212→0.7875; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4017→0.3792 / 25.1214→25.5101 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -66.25 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2; Δx 18.75 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `display:` dx 0.1 dy -10.42; `Rich` dx 0 dy -10.42; `√x` dx -7.41 dy 4.96
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: \left is not supported in math mode; error: \right is not supported in math mode; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933085, 452, 329, 325, 310, 443, 416, 273, 315, 249, 283, 259, 298, 255, 313, 1211]`; ink px ref/ours 2470/1360 (ratio 0.5506); SSIM blocks <0.9: 328/30294; [overlay](images/13-math-display-rich/pdflatex-pipeline-export-p1-overlay.png) (79819 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-pipeline-export-p1-heatmap.png) (80495 B, ÷1)
  - registration error (diagnostic): global shift [6.5, -36.0] pt by ink-projection correlation (centroid estimate [-81.92, -17.51] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4188, differing 0.003267, SSIM₈ 0.9907 (raw 0.44, 0.003374, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9834→0.9856 / 0.7032→0.6484; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.524→0.524 / 13.8276→13.8276 [232.8,85.1–384.7,123.2 pt]
- largest word displacements (pt): `display.` dx 6.64 dy -35.44; `the` dx 5.73 dy -35.44; `after` dx 4.82 dy -35.44
- word-sequence differences: delete ref ['1', '0'] ours []; delete ref ['√x', '1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k'] ours []

### 13-math-display-rich — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931353, 583, 548, 467, 406, 416, 441, 540, 508, 387, 355, 345, 457, 339, 349, 1322]`; ink px ref/ours 2283/2605 (ratio 1.141); SSIM blocks <0.9: 363/30294; [overlay](images/13-math-display-rich/pdflatex-lm-de1020c-export-p1-overlay.png) (86042 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-de1020c-export-p1-heatmap.png) (85050 B, ÷1)
  - registration error (diagnostic): global shift [-2.0, 4.0] pt by ink-projection correlation (centroid estimate [-23.81, 3.31] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5406, differing 0.004269, SSIM₈ 0.9893 (raw 0.553, 0.004302, 0.9888)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.982→0.983 / 0.8838→0.8638; header-band 1.0→0.9992 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4037→0.4126 / 25.0041→24.5413 [232.8,84.8–384.7,123 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -11.5 Δy 4.0 len 21.0 vs 32.0, thickness px 1 vs 1; Δx -16.75 Δy 4.0 len 21.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `display:` dx -1.48 dy -10.16; `Rich` dx 0 dy -10.16; `√x` dx -7.41 dy 5.22
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (3): error: \left is not supported in math mode; error: \right is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931186, 591, 569, 475, 421, 424, 448, 549, 518, 393, 363, 348, 463, 344, 361, 1363]`; ink px ref/ours 2283/2696 (ratio 1.1809); SSIM blocks <0.9: 374/30294; [overlay](images/13-math-display-rich/pdflatex-lm-main-export-p1-overlay.png) (86154 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-main-export-p1-heatmap.png) (85205 B, ÷1)
  - registration error (diagnostic): global shift [-2.0, 4.0] pt by ink-projection correlation (centroid estimate [-12.95, 3.15] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5534, differing 0.004364, SSIM₈ 0.989 (raw 0.5658, 0.004398, 0.9884)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9815→0.9826 / 0.9043→0.8793; header-band 1.0→0.9992 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4037→0.4126 / 25.0041→24.5413 [232.8,84.8–384.7,123 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -11.5 Δy 4.0 len 21.0 vs 32.0, thickness px 1 vs 1; Δx -16.75 Δy 4.0 len 21.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `display:` dx -1.48 dy -10.16; `Rich` dx 0 dy -10.16; `√x` dx -7.41 dy 5.22
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: \left is not supported in math mode; error: \right is not supported in math mode; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1932903, 425, 398, 455, 304, 328, 342, 333, 383, 263, 291, 318, 436, 274, 290, 1073]`; ink px ref/ours 2283/1360 (ratio 0.5957); SSIM blocks <0.9: 350/30294; [overlay](images/13-math-display-rich/pdflatex-lm-pipeline-export-p1-overlay.png) (80000 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-pipeline-export-p1-heatmap.png) (80855 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, -35.5] pt by ink-projection correlation (centroid estimate [-106.12, -16.95] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4325, differing 0.003319, SSIM₈ 0.9897 (raw 0.4462, 0.003413, 0.989)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9825→0.9835 / 0.7131→0.6912; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5246→0.5246 / 13.8189→13.8189 [232.8,84.8–384.7,123 pt]
- largest word displacements (pt): `Text` dx 3.9 dy -35.05; `display.` dx -2.09 dy -35.05; `after` dx 1.38 dy -35.05
- word-sequence differences: delete ref ['1', '0'] ours []; delete ref ['√x', '1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k'] ours []

### 13-math-display-rich — xelatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931959, 611, 525, 464, 361, 421, 572, 475, 341, 330, 316, 270, 301, 321, 289, 1260]`; ink px ref/ours 2475/2605 (ratio 1.0525); SSIM blocks <0.9: 322/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 3.5] pt REJECTED: applying it gives mean|Δ| 0.4946, not lower; centroid estimate [-0.43, 2.61] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4941, differing 0.004089, SSIM₈ 0.9899 (raw 0.4941, 0.004089, 0.9899)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9839→0.9839 / 0.7898→0.7898; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5932→0.5932 / 17.259→17.259 [227.3,75.6–384.7,129 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -66.25 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2; Δx 18.75 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `√x` dx -7.41 dy 4.37; `display.` dx 1.09 dy 3.86; `the` dx 1.02 dy 3.86
- word-sequence differences: replace ref ['∫1'] ours ['∫', '1']; replace ref ['1', '+', 'x2', 'dx=', '∞', '∑', 'k=0', '((−1)k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (3): error: \left is not supported in math mode; error: \right is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931792, 619, 546, 472, 376, 429, 579, 484, 351, 336, 324, 273, 307, 326, 301, 1301]`; ink px ref/ours 2475/2696 (ratio 1.0893); SSIM blocks <0.9: 333/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 3.5] pt REJECTED: applying it gives mean|Δ| 0.5074, not lower; centroid estimate [10.43, 2.44] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5069, differing 0.004184, SSIM₈ 0.9896 (raw 0.5069, 0.004184, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9834→0.9834 / 0.8102→0.8102; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5932→0.5932 / 17.259→17.259 [227.3,75.6–384.7,129 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -66.25 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2; Δx 18.75 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `√x` dx -7.41 dy 4.37; `display.` dx 1.09 dy 3.86; `the` dx 1.02 dy 3.86
- word-sequence differences: replace ref ['∫1'] ours ['∫', '1']; replace ref ['1', '+', 'x2', 'dx=', '∞', '∑', 'k=0', '((−1)k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: \left is not supported in math mode; error: \right is not supported in math mode; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933010, 453, 361, 360, 263, 340, 563, 297, 301, 232, 292, 271, 316, 293, 271, 1193]`; ink px ref/ours 2475/1360 (ratio 0.5495); SSIM blocks <0.9: 332/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [6.5, -36.0] pt by ink-projection correlation (centroid estimate [-82.75, -17.65] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4138, differing 0.003256, SSIM₈ 0.9907 (raw 0.4415, 0.003402, 0.9895)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9833→0.9854 / 0.7056→0.6405; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6827→0.6827 / 9.4959→9.4959 [227.3,75.6–384.7,129 pt]
- largest word displacements (pt): `display.` dx 6.46 dy -35.63; `the` dx 5.55 dy -35.63; `after` dx 4.64 dy -35.63
- word-sequence differences: delete ref ['∫1', '0', '√x', '1', '+', 'x2', 'dx=', '∞'] ours []

### 13-math-display-rich — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931243, 707, 529, 432, 440, 437, 471, 554, 438, 438, 467, 317, 319, 346, 354, 1324]`; ink px ref/ours 2286/2605 (ratio 1.1395); SSIM blocks <0.9: 363/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.0, 4.0] pt by ink-projection correlation (centroid estimate [-21.76, 3.0] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5476, differing 0.004334, SSIM₈ 0.9892 (raw 0.5517, 0.004326, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.982→0.9828 / 0.8818→0.875; header-band 1.0→0.9992 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.591→0.6091 / 17.212→16.9465 [227.3,75.4–384.7,128.9 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -11.5 Δy 4.0 len 21.0 vs 31.0, thickness px 1 vs 1; Δx -16.75 Δy 4.0 len 21.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `√x` dx -7.41 dy 4.51; `display.` dx -7.46 dy 4.15; `the` dx -4.97 dy 4.15
- word-sequence differences: replace ref ['∫1'] ours ['∫', '1']; replace ref ['1', '+', 'x2', 'dx=', '∞', '∑', 'k=0', '((−1)k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (3): error: \left is not supported in math mode; error: \right is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931076, 715, 550, 440, 455, 445, 478, 563, 448, 444, 475, 320, 325, 351, 366, 1365]`; ink px ref/ours 2286/2696 (ratio 1.1794); SSIM blocks <0.9: 374/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.0, 4.0] pt by ink-projection correlation (centroid estimate [-10.89, 2.84] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5604, differing 0.004429, SSIM₈ 0.9888 (raw 0.5645, 0.004421, 0.9884)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9814→0.9824 / 0.9022→0.8905; header-band 1.0→0.9992 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.591→0.6091 / 17.212→16.9465 [227.3,75.4–384.7,128.9 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -11.5 Δy 4.0 len 21.0 vs 31.0, thickness px 1 vs 1; Δx -16.75 Δy 4.0 len 21.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `√x` dx -7.41 dy 4.51; `display.` dx -7.46 dy 4.15; `the` dx -4.97 dy 4.15
- word-sequence differences: replace ref ['∫1'] ours ['∫', '1']; replace ref ['1', '+', 'x2', 'dx=', '∞', '∑', 'k=0', '((−1)k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: \left is not supported in math mode; error: \right is not supported in math mode; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1932800, 552, 382, 393, 340, 313, 366, 340, 313, 339, 452, 305, 284, 277, 293, 1067]`; ink px ref/ours 2286/1360 (ratio 0.5949); SSIM blocks <0.9: 350/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -36.0] pt by ink-projection correlation (centroid estimate [-104.07, -17.26] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4293, differing 0.003347, SSIM₈ 0.9899 (raw 0.4465, 0.003445, 0.989)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9824→0.9838 / 0.7136→0.6861; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6798→0.6798 / 9.4957→9.4957 [227.3,75.4–384.7,128.9 pt]
- largest word displacements (pt): `Text` dx 3.9 dy -35.34; `display.` dx -2.09 dy -35.34; `after` dx 1.4 dy -35.34
- word-sequence differences: delete ref ['∫1', '0', '√x', '1', '+', 'x2', 'dx=', '∞'] ours []

### 14-math-inline-dense — lualatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921604, 1302, 1274, 1027, 985, 855, 941, 950, 907, 1044, 839, 1126, 837, 737, 803, 3585]`; ink px ref/ours 4839/5720 (ratio 1.1821); SSIM blocks <0.9: 954/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-34.5, 13.5] pt by ink-projection correlation (centroid estimate [-14.42, 7.16] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9604, differing 0.008451, SSIM₈ 0.9783 (raw 1.3179, 0.009946, 0.9705)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9529→0.9733 / 2.1063→1.1471; header-band 1.0→0.9477 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3388→0.3463 / 26.8632→26.291 [96.4,69.7–279.9,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -447.81 dy 29.13; `,` dx 105.07 dy -10.04; `,` dx 84.05 dy -10.04
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours ['a2+b2', '=c2']; replace ref ['πr2'] ours ['α', 'βγ,']

### 14-math-inline-dense — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921604, 1302, 1274, 1027, 985, 855, 941, 950, 907, 1044, 839, 1126, 837, 737, 803, 3585]`; ink px ref/ours 4839/5720 (ratio 1.1821); SSIM blocks <0.9: 954/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-34.5, 13.5] pt by ink-projection correlation (centroid estimate [-14.42, 7.16] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9604, differing 0.008451, SSIM₈ 0.9783 (raw 1.3179, 0.009946, 0.9705)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9529→0.9733 / 2.1063→1.1471; header-band 1.0→0.9477 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3388→0.3463 / 26.8632→26.291 [96.4,69.7–279.9,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -447.81 dy 29.13; `,` dx 105.07 dy -10.04; `,` dx 84.05 dy -10.04
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours ['a2+b2', '=c2']; replace ref ['πr2'] ours ['α', 'βγ,']

### 14-math-inline-dense — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (3): error: \nu is not supported in math mode; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927670, 942, 769, 690, 726, 570, 655, 674, 573, 681, 491, 680, 577, 545, 543, 2030]`; ink px ref/ours 4839/2635 (ratio 0.5445); SSIM blocks <0.9: 590/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -14.5] pt by ink-projection correlation (centroid estimate [26.11, -8.25] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7929, differing 0.006416, SSIM₈ 0.9821 (raw 0.8302, 0.006567, 0.9818)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.971→0.9716 / 1.3268→1.267; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4644→0.4385 / 18.1896→19.2519 [96.4,69.7–279.9,108.1 pt]
- largest word displacements (pt): `,` dx -364.11 dy 0.41; `,` dx -110.55 dy -1.3; `,` dx -78.79 dy -2.68
- word-sequence differences: delete ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours []; delete ref ['πr2'] ours []; delete ref ['δx,', 'n!,', '=', 'c2'] ours []

### 14-math-inline-dense — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922082, 1331, 1233, 1019, 1029, 845, 940, 995, 1041, 1032, 818, 1076, 808, 667, 795, 3105]`; ink px ref/ours 4245/5720 (ratio 1.3475); SSIM blocks <0.9: 958/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-9.66, 6.78] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1675, differing 0.009235, SSIM₈ 0.9735 (raw 1.2473, 0.00966, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9655 / 1.9935→1.4755; header-band 1.0→0.9479 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3397→0.2823 / 24.4382→26.6867 [176,69.7–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -51.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -114.06 dy 11.6; `,` dx 110.02 dy -10.8; `,` dx 106.77 dy -10.8
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'z2'] ours ['a2+b2', '=c2']; replace ref ['p1,', 'σ2'] ours ['α', 'βγ,']

### 14-math-inline-dense — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922082, 1331, 1233, 1019, 1029, 845, 940, 995, 1041, 1032, 818, 1076, 808, 667, 795, 3105]`; ink px ref/ours 4245/5720 (ratio 1.3475); SSIM blocks <0.9: 958/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-9.66, 6.78] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1675, differing 0.009235, SSIM₈ 0.9735 (raw 1.2473, 0.00966, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9655 / 1.9935→1.4755; header-band 1.0→0.9479 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3397→0.2823 / 24.4382→26.6867 [176,69.7–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -51.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -114.06 dy 11.6; `,` dx 110.02 dy -10.8; `,` dx 106.77 dy -10.8
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'z2'] ours ['a2+b2', '=c2']; replace ref ['p1,', 'σ2'] ours ['α', 'βγ,']

### 14-math-inline-dense — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (3): error: \nu is not supported in math mode; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927637, 965, 753, 730, 736, 633, 659, 788, 746, 690, 507, 631, 544, 483, 537, 1777]`; ink px ref/ours 4245/2635 (ratio 0.6207); SSIM blocks <0.9: 628/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, -14.5] pt by ink-projection correlation (centroid estimate [30.87, -8.63] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7447, differing 0.006221, SSIM₈ 0.9819 (raw 0.8055, 0.0065, 0.9808)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9692→0.9711 / 1.2874→1.1897; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4387→0.4524 / 16.8115→15.6928 [176,69.7–288.8,108.1 pt]
- largest word displacements (pt): `,` dx -112.86 dy -1.3; `,` dx -78.65 dy -2.68; `,` dx 57.01 dy -14.8
- word-sequence differences: delete ref ['a2', '+', 'b2', 'z2'] ours []; delete ref ['p1,', 'σ2'] ours []; delete ref ['πr2'] ours []

### 14-math-inline-dense — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921723, 1256, 1319, 999, 1015, 858, 878, 898, 911, 966, 857, 1132, 834, 737, 861, 3572]`; ink px ref/ours 4824/5720 (ratio 1.1857); SSIM blocks <0.9: 948/30294; [overlay](images/14-math-inline-dense/pdflatex-de1020c-export-p1-overlay.png) (48021 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-de1020c-export-p1-heatmap.png) (46698 B, ÷2)
  - registration error (diagnostic): global shift [-34.0, 13.5] pt by ink-projection correlation (centroid estimate [-13.94, 7.12] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9608, differing 0.008388, SSIM₈ 0.9783 (raw 1.3129, 0.009906, 0.9708)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9533→0.9734 / 2.0983→1.1478; header-band 1.0→0.9469 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3428→0.3554 / 26.6779→25.5089 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -407.82 dy 29.13; `,` dx -87.71 dy 29.13; `,` dx 84.05 dy -10.04
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours ['a2+b2', '=c2']; insert ref [] ours ['α', 'βγ,', ',', '√2', ',', 'xj', 'i', ',']

### 14-math-inline-dense — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921723, 1256, 1319, 999, 1015, 858, 878, 898, 911, 966, 857, 1132, 834, 737, 861, 3572]`; ink px ref/ours 4824/5720 (ratio 1.1857); SSIM blocks <0.9: 948/30294; [overlay](images/14-math-inline-dense/pdflatex-main-export-p1-overlay.png) (47648 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-main-export-p1-heatmap.png) (46347 B, ÷2)
  - registration error (diagnostic): global shift [-34.0, 13.5] pt by ink-projection correlation (centroid estimate [-13.94, 7.12] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9608, differing 0.008388, SSIM₈ 0.9783 (raw 1.3129, 0.009906, 0.9708)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9533→0.9734 / 2.0983→1.1478; header-band 1.0→0.9469 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3428→0.3554 / 26.6779→25.5089 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -407.82 dy 29.13; `,` dx -87.71 dy 29.13; `,` dx 84.05 dy -10.04
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours ['a2+b2', '=c2']; insert ref [] ours ['α', 'βγ,', ',', '√2', ',', 'xj', 'i', ',']

### 14-math-inline-dense — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (3): error: \nu is not supported in math mode; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927927, 886, 792, 637, 682, 572, 614, 623, 582, 584, 497, 689, 565, 563, 575, 2028]`; ink px ref/ours 4824/2635 (ratio 0.5462); SSIM blocks <0.9: 582/30294; [overlay](images/14-math-inline-dense/pdflatex-pipeline-export-p1-overlay.png) (41571 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-pipeline-export-p1-heatmap.png) (39492 B, ÷2)
  - registration error (diagnostic): global shift [1.0, -14.5] pt by ink-projection correlation (centroid estimate [26.59, -8.29] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7901, differing 0.006385, SSIM₈ 0.9825 (raw 0.8202, 0.006508, 0.9822)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9715→0.9721 / 1.3109→1.2623; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4704→0.4521 / 17.9214→18.9221 [96.4,69.7–279.8,108.1 pt]
- largest word displacements (pt): `,` dx -377.39 dy 0.41; `,` dx -44.93 dy 0.41; `,` dx 39.31 dy -14.04
- word-sequence differences: delete ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours []; delete ref ['πr2'] ours []; delete ref ['δx,', 'n!,', '=', 'c2'] ours []

### 14-math-inline-dense — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922114, 1292, 1255, 994, 1030, 864, 932, 987, 1070, 1032, 804, 1077, 790, 689, 773, 3113]`; ink px ref/ours 4254/5720 (ratio 1.3446); SSIM blocks <0.9: 957/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-de1020c-export-p1-overlay.png) (48294 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-de1020c-export-p1-heatmap.png) (46738 B, ÷2)
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-7.87, 6.82] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1673, differing 0.009234, SSIM₈ 0.9735 (raw 1.2469, 0.009657, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9655 / 1.9928→1.475; header-band 1.0→0.9479 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3525→0.2973 / 24.7689→27.2474 [183.4,69.8–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -51.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `z2` dx 408.28 dy -8.86; `must` dx -73.95 dy 13.68; `that` dx -69.96 dy 13.68
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2'] ours ['a2+b2', '=c2', ',', 'α', 'βγ,', ',', '√2', ',']; insert ref [] ours ['πr2', 'a+b']

### 14-math-inline-dense — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922114, 1292, 1255, 994, 1030, 864, 932, 987, 1070, 1032, 804, 1077, 790, 689, 773, 3113]`; ink px ref/ours 4254/5720 (ratio 1.3446); SSIM blocks <0.9: 957/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-main-export-p1-overlay.png) (47989 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-main-export-p1-heatmap.png) (46419 B, ÷2)
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-7.87, 6.82] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1673, differing 0.009234, SSIM₈ 0.9735 (raw 1.2469, 0.009657, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9655 / 1.9928→1.475; header-band 1.0→0.9479 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3525→0.2973 / 24.7689→27.2474 [183.4,69.8–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -51.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `z2` dx 408.28 dy -8.86; `must` dx -73.95 dy 13.68; `that` dx -69.96 dy 13.68
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2'] ours ['a2+b2', '=c2', ',', 'α', 'βγ,', ',', '√2', ',']; insert ref [] ours ['πr2', 'a+b']

### 14-math-inline-dense — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (3): error: \nu is not supported in math mode; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927669, 928, 777, 697, 745, 648, 649, 780, 783, 682, 498, 630, 527, 505, 515, 1783]`; ink px ref/ours 4254/2635 (ratio 0.6194); SSIM blocks <0.9: 626/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-pipeline-export-p1-overlay.png) (41291 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-pipeline-export-p1-heatmap.png) (40008 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, -14.5] pt by ink-projection correlation (centroid estimate [32.66, -8.59] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7299, differing 0.006158, SSIM₈ 0.9822 (raw 0.805, 0.006497, 0.9808)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9693→0.9715 / 1.2867→1.1665; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4403→0.4439 / 17.1477→16.2232 [183.4,69.8–288.8,108.1 pt]
- largest word displacements (pt): `,` dx 57.01 dy -14.8; `,` dx -42.62 dy 0.67; `,` dx -37.01 dy -17.13
- word-sequence differences: delete ref ['a2', '+', 'b2', 'z2'] ours []; delete ref ['p1,', 'σ2'] ours []; delete ref ['πr2'] ours []

### 14-math-inline-dense — xelatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921627, 1283, 1284, 1007, 1006, 848, 933, 938, 920, 1037, 846, 1107, 844, 738, 798, 3600]`; ink px ref/ours 4852/5720 (ratio 1.1789); SSIM blocks <0.9: 954/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-34.5, 13.5] pt by ink-projection correlation (centroid estimate [-14.66, 7.14] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9575, differing 0.00844, SSIM₈ 0.9783 (raw 1.3179, 0.009938, 0.9705)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9529→0.9733 / 2.1063→1.1425; header-band 1.0→0.9477 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3387→0.3463 / 26.8617→26.2884 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -447.81 dy 29.13; `,` dx 105.07 dy -10.04; `,` dx 84.05 dy -10.04
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours ['a2+b2', '=c2']; replace ref ['πr2'] ours ['α', 'βγ,']

### 14-math-inline-dense — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921627, 1283, 1284, 1007, 1006, 848, 933, 938, 920, 1037, 846, 1107, 844, 738, 798, 3600]`; ink px ref/ours 4852/5720 (ratio 1.1789); SSIM blocks <0.9: 954/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-34.5, 13.5] pt by ink-projection correlation (centroid estimate [-14.66, 7.14] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9575, differing 0.00844, SSIM₈ 0.9783 (raw 1.3179, 0.009938, 0.9705)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9529→0.9733 / 2.1063→1.1425; header-band 1.0→0.9477 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3387→0.3463 / 26.8617→26.2884 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -447.81 dy 29.13; `,` dx 105.07 dy -10.04; `,` dx 84.05 dy -10.04
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours ['a2+b2', '=c2']; replace ref ['πr2'] ours ['α', 'βγ,']

### 14-math-inline-dense — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (3): error: \nu is not supported in math mode; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927692, 925, 778, 672, 744, 565, 649, 661, 581, 681, 494, 661, 585, 548, 537, 2043]`; ink px ref/ours 4852/2635 (ratio 0.5431); SSIM blocks <0.9: 590/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -14.5] pt by ink-projection correlation (centroid estimate [25.87, -8.27] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7928, differing 0.00642, SSIM₈ 0.9821 (raw 0.8301, 0.00656, 0.9819)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.971→0.9716 / 1.3266→1.2669; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4645→0.4384 / 18.1805→19.2545 [96.4,69.7–279.8,108.1 pt]
- largest word displacements (pt): `,` dx -364.11 dy 0.41; `,` dx -110.55 dy -18.53; `,` dx -78.79 dy -2.68
- word-sequence differences: delete ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours []; delete ref ['πr2'] ours []; delete ref ['δx,', 'n!,', '=', 'c2'] ours []

### 14-math-inline-dense — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922076, 1334, 1239, 1015, 1031, 844, 933, 1003, 1046, 1026, 822, 1075, 807, 668, 793, 3104]`; ink px ref/ours 4244/5720 (ratio 1.3478); SSIM blocks <0.9: 958/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-9.59, 6.78] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1675, differing 0.009235, SSIM₈ 0.9735 (raw 1.2473, 0.00966, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9655 / 1.9935→1.4755; header-band 1.0→0.9479 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3525→0.2992 / 24.8173→27.2669 [183.4,69.7–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -51.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -114.06 dy 11.6; `,` dx 110.02 dy -9.77; `,` dx 106.77 dy -9.77
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'z2'] ours ['a2+b2', '=c2']; replace ref ['p1,', 'σ2'] ours ['α', 'βγ,']

### 14-math-inline-dense — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922076, 1334, 1239, 1015, 1031, 844, 933, 1003, 1046, 1026, 822, 1075, 807, 668, 793, 3104]`; ink px ref/ours 4244/5720 (ratio 1.3478); SSIM blocks <0.9: 958/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-9.59, 6.78] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1675, differing 0.009235, SSIM₈ 0.9735 (raw 1.2473, 0.00966, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9655 / 1.9935→1.4755; header-band 1.0→0.9479 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3525→0.2992 / 24.8173→27.2669 [183.4,69.7–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -51.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -114.06 dy 11.6; `,` dx 110.02 dy -9.77; `,` dx 106.77 dy -9.77
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'z2'] ours ['a2+b2', '=c2']; replace ref ['p1,', 'σ2'] ours ['α', 'βγ,']

### 14-math-inline-dense — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (3): error: \nu is not supported in math mode; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern Math unavailable (latinmodern-math.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); math is not typeset
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927631, 967, 761, 725, 739, 633, 654, 790, 759, 677, 508, 629, 548, 482, 537, 1776]`; ink px ref/ours 4244/2635 (ratio 0.6209); SSIM blocks <0.9: 628/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, -14.5] pt by ink-projection correlation (centroid estimate [30.94, -8.63] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7447, differing 0.006221, SSIM₈ 0.9819 (raw 0.8056, 0.0065, 0.9808)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9692→0.9711 / 1.2875→1.1897; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4412→0.4546 / 17.2111→15.9846 [183.4,69.7–288.8,108.1 pt]
- largest word displacements (pt): `,` dx -112.86 dy -18.53; `,` dx -78.65 dy -2.68; `,` dx 57.01 dy -14.8
- word-sequence differences: delete ref ['a2', '+', 'b2', 'z2'] ours []; delete ref ['p1,', 'σ2'] ours []; delete ref ['πr2'] ours []

### 15-three-page-sections — lualatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1598038, 24940, 20790, 19965, 17891, 18166, 18319, 16938, 16645, 17913, 16734, 16380, 17123, 16110, 16892, 85972]`; ink px ref/ours 112785/116938 (ratio 1.0368); SSIM blocks <0.9: 14156/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 20.0] pt by ink-projection correlation (centroid estimate [-14.14, 65.81] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.653, differing 0.186006, SSIM₈ 0.5818 (raw 27.2737, 0.194397, 0.5415)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2795→0.3358 / 42.923→40.8971; header-band 1.0→0.988 / 0.0→0.6238; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1594695, 25695, 21111, 20501, 18440, 18500, 18873, 17381, 16974, 18514, 17079, 16542, 16961, 16194, 16980, 84376]`; ink px ref/ours 112804/122724 (ratio 1.0879); SSIM blocks <0.9: 14115/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 19.0] pt by ink-projection correlation (centroid estimate [-13.27, 40.37] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.5618, differing 0.192924, SSIM₈ 0.5591 (raw 27.3092, 0.196515, 0.5489)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2795→0.3158 / 43.642→41.7156; header-band 1.0→0.8751 / 0.0→4.9901; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1628395, 23133, 19213, 18606, 16386, 16651, 16742, 15568, 15271, 16122, 15480, 14939, 15490, 14823, 15245, 76752]`; ink px ref/ours 112799/99417 (ratio 0.8814); SSIM blocks <0.9: 12452/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -24.5] pt by ink-projection correlation (centroid estimate [-13.21, -17.77] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.4326, differing 0.170498, SSIM₈ 0.6218 (raw 24.6889, 0.177258, 0.6001)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3613→0.3976 / 39.454→37.4405; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 95.02; `branch` dx -435.47 dy 74.98; `branch` dx -435.47 dy 54.95

### 15-three-page-sections — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1597770, 24963, 20791, 19977, 17886, 18196, 18313, 16925, 16624, 17910, 16759, 16378, 17158, 16122, 16915, 86129]`; ink px ref/ours 112785/117099 (ratio 1.0382); SSIM blocks <0.9: 14174/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 20.0] pt by ink-projection correlation (centroid estimate [-14.37, 65.91] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.6757, differing 0.186153, SSIM₈ 0.5814 (raw 27.303, 0.194557, 0.5409)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2785→0.3353 / 42.9697→40.9182; header-band 1.0→0.9865 / 0.0→0.7294; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1594578, 25713, 21123, 20504, 18434, 18504, 18880, 17385, 16956, 18491, 17098, 16543, 16977, 16195, 17002, 84433]`; ink px ref/ours 112804/122799 (ratio 1.0886); SSIM blocks <0.9: 14126/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 19.0] pt by ink-projection correlation (centroid estimate [-13.41, 40.52] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.5736, differing 0.193002, SSIM₈ 0.5589 (raw 27.321, 0.196593, 0.5486)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.279→0.3155 / 43.6608→41.7345; header-band 1.0→0.8751 / 0.0→4.9901; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1628395, 23133, 19213, 18606, 16386, 16651, 16742, 15568, 15271, 16122, 15480, 14939, 15490, 14823, 15245, 76752]`; ink px ref/ours 112799/99417 (ratio 0.8814); SSIM blocks <0.9: 12452/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -24.5] pt by ink-projection correlation (centroid estimate [-13.21, -17.77] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.4326, differing 0.170498, SSIM₈ 0.6218 (raw 24.6889, 0.177258, 0.6001)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3613→0.3976 / 39.454→37.4405; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 95.02; `branch` dx -435.47 dy 74.98; `branch` dx -435.47 dy 54.95
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1633175, 24836, 21009, 19227, 17717, 17744, 16775, 16790, 16201, 16288, 15629, 14982, 14824, 14448, 14648, 64523]`; ink px ref/ours 112785/122582 (ratio 1.0869); SSIM blocks <0.9: 12410/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 14.0] pt by ink-projection correlation (centroid estimate [-7.13, 55.58] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 22.2562, differing 0.170947, SSIM₈ 0.6509 (raw 23.258, 0.175459, 0.6319)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4129→0.446 / 37.1623→35.4631; header-band 1.0→0.9895 / 0.0→0.6401; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1603329, 25744, 21745, 20182, 18696, 18105, 17262, 17694, 17382, 17877, 17128, 16268, 16532, 16657, 16523, 77692]`; ink px ref/ours 112804/126174 (ratio 1.1185); SSIM blocks <0.9: 13978/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -26.5] pt by ink-projection correlation (centroid estimate [-4.97, 43.88] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.2395, differing 0.176077, SSIM₈ 0.634 (raw 26.279, 0.192101, 0.5576)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2943→0.4292 / 41.9902→36.4753; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9195 / 0.0→4.4986
- page 3: |Δ| histogram (16 bins, pixel counts) `[1656239, 22092, 18635, 16896, 16016, 15832, 15066, 15104, 14325, 14745, 14091, 13580, 13928, 13833, 13899, 64535]`; ink px ref/ours 112799/88545 (ratio 0.785); SSIM blocks <0.9: 11639/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -55.5] pt by ink-projection correlation (centroid estimate [-7.74, -46.94] pt); confidence moderate (shift explains 13% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.1063, differing 0.146467, SSIM₈ 0.7063 (raw 21.9917, 0.161844, 0.6325)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4139→0.5324 / 35.139→30.5249; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy -98.57; `branch` dx -435.47 dy -84.12; `branch` dx -435.47 dy -69.67

### 15-three-page-sections — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610512, 25235, 21060, 20651, 19444, 18355, 19690, 19541, 18810, 18112, 16258, 16004, 15486, 14441, 15467, 69750]`; ink px ref/ours 85641/116938 (ratio 1.3654); SSIM blocks <0.9: 14340/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.1, 38.3] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.3382, differing 0.178231, SSIM₈ 0.5819 (raw 25.0358, 0.188298, 0.534)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2673→0.3354 / 39.3511→37.2027; header-band 1.0→0.9885 / 0.0→0.6238; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606570, 25871, 21215, 20968, 19842, 18843, 20472, 19874, 18904, 18530, 16244, 16137, 15497, 14453, 15539, 69857]`; ink px ref/ours 85678/122724 (ratio 1.4324); SSIM blocks <0.9: 14622/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.17, 12.98] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.2452, differing 0.185, SSIM₈ 0.5621 (raw 25.2319, 0.19044, 0.5335)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2546→0.3153 / 40.3269→38.0452; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9087 / 0.0→4.802
- page 3: |Δ| histogram (16 bins, pixel counts) `[1640991, 23406, 19393, 19012, 17789, 16864, 18053, 18241, 17308, 16215, 14610, 14544, 14086, 12947, 14031, 61326]`; ink px ref/ours 85675/99417 (ratio 1.1604); SSIM blocks <0.9: 13072/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.1, -45.23] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.5359, differing 0.16539, SSIM₈ 0.6023 (raw 22.516, 0.171006, 0.5805)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3297→0.3661 / 35.986→34.4142; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -93.06; `oak` dx 426.94 dy -87.47; `oak` dx 426.94 dy -81.88

### 15-three-page-sections — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610331, 25236, 21076, 20663, 19433, 18378, 19689, 19539, 18777, 18107, 16280, 16009, 15525, 14432, 15499, 69842]`; ink px ref/ours 85641/117099 (ratio 1.3673); SSIM blocks <0.9: 14353/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.33, 38.4] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.361, differing 0.178378, SSIM₈ 0.5814 (raw 25.0557, 0.188422, 0.5336)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2667→0.3349 / 39.383→37.2238; header-band 1.0→0.9867 / 0.0→0.7294; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606469, 25870, 21229, 20969, 19844, 18847, 20473, 19865, 18895, 18504, 16264, 16143, 15512, 14453, 15571, 69908]`; ink px ref/ours 85678/122799 (ratio 1.4333); SSIM blocks <0.9: 14629/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [3.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.3, 13.12] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.3196, differing 0.18529, SSIM₈ 0.5596 (raw 25.2438, 0.190513, 0.5332)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2542→0.315 / 40.346→37.9148; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9074 / 0.0→4.802
- page 3: |Δ| histogram (16 bins, pixel counts) `[1640991, 23406, 19393, 19012, 17789, 16864, 18053, 18241, 17308, 16215, 14610, 14544, 14086, 12947, 14031, 61326]`; ink px ref/ours 85675/99417 (ratio 1.1604); SSIM blocks <0.9: 13072/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.1, -45.23] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.5359, differing 0.16539, SSIM₈ 0.6023 (raw 22.516, 0.171006, 0.5805)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3297→0.3661 / 35.986→34.4142; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -93.06; `oak` dx 426.94 dy -87.47; `oak` dx 426.94 dy -81.88
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1651995, 24426, 20673, 19572, 18469, 17498, 17533, 17701, 16853, 15595, 13708, 13244, 12832, 12721, 13003, 52993]`; ink px ref/ours 85641/122582 (ratio 1.4313); SSIM blocks <0.9: 12089/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.1, 28.07] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.8, differing 0.165888, SSIM₈ 0.6509 (raw 20.9245, 0.166145, 0.6503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.442→0.4444 / 33.4371→33.2334; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1611978, 26306, 22072, 20811, 19970, 18619, 18491, 20129, 19509, 17834, 16593, 15935, 15246, 14899, 15211, 65213]`; ink px ref/ours 85678/126174 (ratio 1.4727); SSIM blocks <0.9: 14675/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -26.5] pt by ink-projection correlation (centroid estimate [1.14, 16.49] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 22.0038, differing 0.17339, SSIM₈ 0.6125 (raw 24.5411, 0.188087, 0.5386)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2636→0.3946 / 39.2176→34.5043; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9195 / 0.0→4.4986
- page 3: |Δ| histogram (16 bins, pixel counts) `[1665013, 22240, 18686, 17650, 17293, 15963, 16380, 17353, 16220, 14932, 13658, 13116, 12475, 12580, 12882, 52375]`; ink px ref/ours 85675/88545 (ratio 1.0335); SSIM blocks <0.9: 12273/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -26.5] pt by ink-projection correlation (centroid estimate [-1.62, -74.4] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 17.8808, differing 0.144054, SSIM₈ 0.6753 (raw 20.3342, 0.157653, 0.611)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3791→0.4827 / 32.4948→28.5702; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -142.22; `oak` dx 450.74 dy -142.22; `oak` dx 450.74 dy -142.22

### 15-three-page-sections — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1595899, 24122, 20968, 20023, 17718, 17966, 18322, 17267, 17094, 18406, 16944, 15990, 16755, 16056, 16950, 88336]`; ink px ref/ours 112156/116938 (ratio 1.0426); SSIM blocks <0.9: 14346/30294; [overlay](images/15-three-page-sections/pdflatex-de1020c-export-p1-overlay.png) (48279 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-de1020c-export-p1-heatmap.png) (35533 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 36.5] pt by ink-projection correlation (centroid estimate [-13.64, 42.99] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.376, differing 0.183932, SSIM₈ 0.5937 (raw 27.5937, 0.195208, 0.5337)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2665→0.3542 / 43.4405→40.4605; header-band 1.0→0.9879 / 0.0→0.6238; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1592865, 24793, 21494, 20314, 18311, 18548, 19317, 17570, 17258, 18450, 17153, 16254, 16690, 16013, 17183, 86603]`; ink px ref/ours 112190/122724 (ratio 1.0939); SSIM blocks <0.9: 14238/30294; [overlay](images/15-three-page-sections/pdflatex-de1020c-export-p2-overlay.png) (49194 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-de1020c-export-p2-heatmap.png) (35666 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.72, 17.61] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.0991, differing 0.18994, SSIM₈ 0.5742 (raw 27.5896, 0.197186, 0.5428)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2693→0.3363 / 44.0962→41.0127; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8846 / 0.0→4.8261
- page 3: |Δ| histogram (16 bins, pixel counts) `[1626455, 22468, 19561, 18552, 16338, 16950, 16970, 15864, 15658, 16281, 15491, 14672, 14985, 14631, 15599, 78341]`; ink px ref/ours 112177/99417 (ratio 0.8863); SSIM blocks <0.9: 12897/30294; [overlay](images/15-three-page-sections/pdflatex-de1020c-export-p3-overlay.png) (45472 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-de1020c-export-p3-heatmap.png) (33505 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.66, -40.6] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.7997, differing 0.172056, SSIM₈ 0.6079 (raw 24.915, 0.178224, 0.5858)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.338→0.3733 / 39.8213→38.0388; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy -63.97; `branch` dx -435.47 dy -58.38; `branch` dx -435.47 dy -52.79

### 15-three-page-sections — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1595636, 24136, 20977, 20021, 17699, 17990, 18320, 17252, 17091, 18406, 16976, 15992, 16802, 16058, 16973, 88487]`; ink px ref/ours 112156/117099 (ratio 1.0441); SSIM blocks <0.9: 14364/30294; [overlay](images/15-three-page-sections/pdflatex-main-export-p1-overlay.png) (48113 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-main-export-p1-heatmap.png) (35362 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 36.5] pt by ink-projection correlation (centroid estimate [-13.87, 43.09] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.3987, differing 0.184079, SSIM₈ 0.5934 (raw 27.6238, 0.195371, 0.5331)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2656→0.3539 / 43.4885→40.4816; header-band 1.0→0.9865 / 0.0→0.7294; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1592762, 24798, 21500, 20307, 18308, 18537, 19321, 17580, 17246, 18440, 17181, 16265, 16709, 16018, 17203, 86641]`; ink px ref/ours 112190/122799 (ratio 1.0946); SSIM blocks <0.9: 14247/30294; [overlay](images/15-three-page-sections/pdflatex-main-export-p2-overlay.png) (48993 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-main-export-p2-heatmap.png) (35472 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.86, 17.76] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.1109, differing 0.190018, SSIM₈ 0.574 (raw 27.6017, 0.197255, 0.5426)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2689→0.3359 / 44.1155→41.0316; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8846 / 0.0→4.8261
- page 3: |Δ| histogram (16 bins, pixel counts) `[1626455, 22468, 19561, 18552, 16338, 16950, 16970, 15864, 15658, 16281, 15491, 14672, 14985, 14631, 15599, 78341]`; ink px ref/ours 112177/99417 (ratio 0.8863); SSIM blocks <0.9: 12897/30294; [overlay](images/15-three-page-sections/pdflatex-main-export-p3-overlay.png) (45238 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-main-export-p3-heatmap.png) (33289 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.66, -40.6] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.7997, differing 0.172056, SSIM₈ 0.6079 (raw 24.915, 0.178224, 0.5858)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.338→0.3733 / 39.8213→38.0388; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy -63.97; `branch` dx -435.47 dy -58.38; `branch` dx -435.47 dy -52.79
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1638000, 23983, 21229, 19071, 17692, 17988, 16963, 16781, 15798, 16055, 15336, 14803, 14370, 14059, 14778, 61910]`; ink px ref/ours 112156/122582 (ratio 1.093); SSIM blocks <0.9: 11899/30294; [overlay](images/15-three-page-sections/pdflatex-pipeline-export-p1-overlay.png) (49361 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-pipeline-export-p1-heatmap.png) (83519 B, ÷4)
  - registration error (diagnostic): global shift [0.0, -0.5] pt by ink-projection correlation (centroid estimate [-6.63, 32.76] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.551, differing 0.167245, SSIM₈ 0.6738 (raw 22.759, 0.17255, 0.6478)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4378→0.4796 / 36.3703→34.4395; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1600993, 25061, 22378, 20232, 18809, 18227, 17206, 17697, 18198, 18068, 17205, 15865, 16124, 16462, 16645, 79646]`; ink px ref/ours 112190/126174 (ratio 1.1246); SSIM blocks <0.9: 14488/30294; [overlay](images/15-three-page-sections/pdflatex-pipeline-export-p2-overlay.png) (50700 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-pipeline-export-p2-heatmap.png) (36832 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -55.5] pt by ink-projection correlation (centroid estimate [-4.42, 21.12] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.7324, differing 0.178828, SSIM₈ 0.6149 (raw 26.531, 0.193293, 0.5413)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2677→0.426 / 42.399→35.856; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.7185 / 0.0→14.2492
- page 3: |Δ| histogram (16 bins, pixel counts) `[1656367, 21514, 19112, 16918, 15988, 15992, 15188, 15070, 14605, 14603, 13795, 13687, 13332, 13461, 14163, 65021]`; ink px ref/ours 112177/88545 (ratio 0.7893); SSIM blocks <0.9: 11825/30294; [overlay](images/15-three-page-sections/pdflatex-pipeline-export-p3-overlay.png) (44888 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-pipeline-export-p3-heatmap.png) (82971 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 2.0] pt by ink-projection correlation (centroid estimate [-7.19, -69.76] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.5031, differing 0.14868, SSIM₈ 0.6868 (raw 21.9854, 0.161433, 0.627)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4045→0.5002 / 35.135→31.1675; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy -113.09; `branch` dx -435.47 dy -113.09; `branch` dx -435.47 dy -113.09

### 15-three-page-sections — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610609, 24967, 21053, 20726, 19661, 18340, 19742, 19412, 18598, 18249, 16287, 15994, 15576, 14410, 15675, 69517]`; ink px ref/ours 85692/116938 (ratio 1.3646); SSIM blocks <0.9: 14337/30294; [overlay](images/15-three-page-sections/pdflatex-lm-de1020c-export-p1-overlay.png) (48536 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-de1020c-export-p1-heatmap.png) (35370 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.11, 38.21] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.3365, differing 0.178246, SSIM₈ 0.5819 (raw 25.0354, 0.18832, 0.5339)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2672→0.3354 / 39.3505→37.2; header-band 1.0→0.9885 / 0.0→0.6238; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606782, 25413, 21423, 20910, 20004, 18862, 20556, 19676, 18675, 18670, 16345, 16142, 15704, 14246, 15727, 69681]`; ink px ref/ours 85729/122724 (ratio 1.4315); SSIM blocks <0.9: 14619/30294; [overlay](images/15-three-page-sections/pdflatex-lm-de1020c-export-p2-overlay.png) (49716 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-de1020c-export-p2-heatmap.png) (35982 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.23, 12.88] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.2389, differing 0.184974, SSIM₈ 0.5622 (raw 25.2323, 0.19048, 0.5336)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2548→0.3156 / 40.3276→38.0352; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9087 / 0.0→4.802
- page 3: |Δ| histogram (16 bins, pixel counts) `[1641126, 23072, 19471, 19055, 17908, 16873, 18097, 18161, 17038, 16322, 14757, 14561, 14158, 12872, 14209, 61136]`; ink px ref/ours 85728/99417 (ratio 1.1597); SSIM blocks <0.9: 13069/30294; [overlay](images/15-three-page-sections/pdflatex-lm-de1020c-export-p3-overlay.png) (45775 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-de1020c-export-p3-heatmap.png) (33601 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.13, -45.32] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.5333, differing 0.165392, SSIM₈ 0.6026 (raw 22.5169, 0.171027, 0.5804)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3295→0.3666 / 35.9875→34.4099; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -92.02; `oak` dx 426.94 dy -86.43; `oak` dx 426.94 dy -80.84

### 15-three-page-sections — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610426, 24967, 21071, 20740, 19645, 18366, 19745, 19409, 18564, 18243, 16308, 16000, 15616, 14400, 15699, 69617]`; ink px ref/ours 85692/117099 (ratio 1.3665); SSIM blocks <0.9: 14350/30294; [overlay](images/15-three-page-sections/pdflatex-lm-main-export-p1-overlay.png) (48369 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-main-export-p1-heatmap.png) (35199 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.34, 38.32] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.3593, differing 0.178393, SSIM₈ 0.5814 (raw 25.0554, 0.188444, 0.5336)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2666→0.3349 / 39.3824→37.2211; header-band 1.0→0.9867 / 0.0→0.7294; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606684, 25405, 21442, 20908, 20007, 18865, 20557, 19667, 18668, 18646, 16362, 16148, 15720, 14244, 15762, 69731]`; ink px ref/ours 85729/122799 (ratio 1.4324); SSIM blocks <0.9: 14626/30294; [overlay](images/15-three-page-sections/pdflatex-lm-main-export-p2-overlay.png) (49551 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-main-export-p2-heatmap.png) (35778 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.36, 13.02] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.2507, differing 0.185052, SSIM₈ 0.562 (raw 25.2442, 0.190554, 0.5334)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2544→0.3151 / 40.3466→38.0542; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9087 / 0.0→4.802
- page 3: |Δ| histogram (16 bins, pixel counts) `[1641126, 23072, 19471, 19055, 17908, 16873, 18097, 18161, 17038, 16322, 14757, 14561, 14158, 12872, 14209, 61136]`; ink px ref/ours 85728/99417 (ratio 1.1597); SSIM blocks <0.9: 13069/30294; [overlay](images/15-three-page-sections/pdflatex-lm-main-export-p3-overlay.png) (45559 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-main-export-p3-heatmap.png) (33403 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.13, -45.32] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.5333, differing 0.165392, SSIM₈ 0.6026 (raw 22.5169, 0.171027, 0.5804)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3295→0.3666 / 35.9875→34.4099; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -92.02; `oak` dx 426.94 dy -86.43; `oak` dx 426.94 dy -80.84
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1652285, 24054, 20834, 19601, 18532, 17397, 17727, 17517, 16799, 15438, 13952, 13304, 12847, 12643, 13023, 52863]`; ink px ref/ours 85692/122582 (ratio 1.4305); SSIM blocks <0.9: 12095/30294; [overlay](images/15-three-page-sections/pdflatex-lm-pipeline-export-p1-overlay.png) (49905 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-pipeline-export-p1-heatmap.png) (83217 B, ÷4)
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.11, 27.99] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.7849, differing 0.165885, SSIM₈ 0.6512 (raw 20.914, 0.166103, 0.6506)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4425→0.4449 / 33.4204→33.2093; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1611967, 26063, 22207, 20813, 20100, 18492, 18592, 20026, 19342, 17926, 16677, 15918, 15360, 14933, 15330, 65070]`; ink px ref/ours 85729/126174 (ratio 1.4718); SSIM blocks <0.9: 14673/30294; [overlay](images/15-three-page-sections/pdflatex-lm-pipeline-export-p2-overlay.png) (51540 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-pipeline-export-p2-heatmap.png) (36811 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -26.5] pt by ink-projection correlation (centroid estimate [1.08, 16.39] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 22.0197, differing 0.173473, SSIM₈ 0.6119 (raw 24.5524, 0.188119, 0.5383)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.263→0.3938 / 39.2356→34.5297; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9195 / 0.0→4.4986
- page 3: |Δ| histogram (16 bins, pixel counts) `[1665050, 21890, 18780, 17701, 17483, 15860, 16530, 17219, 16102, 14894, 13815, 13135, 12604, 12476, 13093, 52184]`; ink px ref/ours 85728/88545 (ratio 1.0329); SSIM blocks <0.9: 12275/30294; [overlay](images/15-three-page-sections/pdflatex-lm-pipeline-export-p3-overlay.png) (45356 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-pipeline-export-p3-heatmap.png) (86125 B, ÷4)
  - registration error (diagnostic): global shift [0.5, -26.5] pt by ink-projection correlation (centroid estimate [-1.66, -74.48] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 17.891, differing 0.14414, SSIM₈ 0.6748 (raw 20.3433, 0.157695, 0.6107)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3787→0.4819 / 32.5093→28.5864; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -141.18; `oak` dx 450.74 dy -141.18; `oak` dx 450.74 dy -141.18

### 15-three-page-sections — xelatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1598134, 24346, 20570, 20303, 17834, 18531, 18289, 17058, 16696, 18223, 16864, 16725, 16884, 15809, 16706, 85844]`; ink px ref/ours 112652/116938 (ratio 1.038); SSIM blocks <0.9: 14151/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 20.0] pt by ink-projection correlation (centroid estimate [-14.28, 65.66] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.6776, differing 0.186275, SSIM₈ 0.5808 (raw 27.2666, 0.194317, 0.5416)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2798→0.3327 / 42.9114→40.9342; header-band 1.0→0.9882 / 0.0→0.6238; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1594667, 25211, 20916, 20918, 18357, 18899, 18879, 17365, 17043, 18690, 17168, 16929, 16763, 15978, 16689, 84344]`; ink px ref/ours 112671/122724 (ratio 1.0892); SSIM blocks <0.9: 14116/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 19.0] pt by ink-projection correlation (centroid estimate [-13.44, 40.23] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.5193, differing 0.192816, SSIM₈ 0.56 (raw 27.307, 0.196469, 0.5491)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2798→0.3158 / 43.6382→41.6469; header-band 1.0→0.8753 / 0.0→4.9901; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1628301, 22537, 18948, 19120, 16594, 16998, 16606, 15676, 15330, 16301, 15579, 15269, 15273, 14546, 15139, 76599]`; ink px ref/ours 112666/99417 (ratio 0.8824); SSIM blocks <0.9: 12446/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, -24.5] pt by ink-projection correlation (centroid estimate [-13.4, -17.92] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.3708, differing 0.170354, SSIM₈ 0.623 (raw 24.687, 0.177244, 0.6004)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3618→0.3984 / 39.4508→37.3396; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 95.02; `branch` dx -435.52 dy 74.98; `branch` dx -435.52 dy 54.95

### 15-three-page-sections — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1597865, 24369, 20573, 20314, 17829, 18560, 18284, 17045, 16675, 18219, 16890, 16723, 16919, 15820, 16730, 86001]`; ink px ref/ours 112652/117099 (ratio 1.0395); SSIM blocks <0.9: 14169/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 20.0] pt by ink-projection correlation (centroid estimate [-14.51, 65.76] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.7003, differing 0.186422, SSIM₈ 0.5802 (raw 27.2959, 0.194477, 0.5411)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2788→0.3322 / 42.9582→40.9553; header-band 1.0→0.9859 / 0.0→0.7294; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1594550, 25229, 20928, 20921, 18351, 18903, 18886, 17369, 17025, 18667, 17187, 16930, 16779, 15979, 16711, 84401]`; ink px ref/ours 112671/122799 (ratio 1.0899); SSIM blocks <0.9: 14127/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 19.0] pt by ink-projection correlation (centroid estimate [-13.58, 40.37] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.5311, differing 0.192893, SSIM₈ 0.5598 (raw 27.3187, 0.196547, 0.5488)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2793→0.3155 / 43.657→41.6657; header-band 1.0→0.8753 / 0.0→4.9901; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1628301, 22537, 18948, 19120, 16594, 16998, 16606, 15676, 15330, 16301, 15579, 15269, 15273, 14546, 15139, 76599]`; ink px ref/ours 112666/99417 (ratio 0.8824); SSIM blocks <0.9: 12446/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, -24.5] pt by ink-projection correlation (centroid estimate [-13.4, -17.92] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.3708, differing 0.170354, SSIM₈ 0.623 (raw 24.687, 0.177244, 0.6004)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3618→0.3984 / 39.4508→37.3396; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 95.02; `branch` dx -435.52 dy 74.98; `branch` dx -435.52 dy 54.95
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1633610, 24191, 20849, 19636, 17861, 18157, 17017, 16632, 16140, 16397, 15865, 15364, 14611, 14103, 14556, 63827]`; ink px ref/ours 112652/122582 (ratio 1.0881); SSIM blocks <0.9: 12400/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 14.0] pt by ink-projection correlation (centroid estimate [-7.28, 55.44] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 22.2098, differing 0.17091, SSIM₈ 0.6522 (raw 23.1848, 0.175242, 0.6331)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4148→0.4469 / 37.0451→35.3932; header-band 1.0→0.9899 / 0.0→0.6401; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1602930, 25200, 21637, 20511, 18661, 18507, 17250, 17909, 17395, 18044, 17181, 16821, 16511, 16259, 16269, 77731]`; ink px ref/ours 112671/126174 (ratio 1.1198); SSIM blocks <0.9: 13969/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -26.5] pt by ink-projection correlation (centroid estimate [-5.14, 43.74] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.2273, differing 0.176032, SSIM₈ 0.6345 (raw 26.3082, 0.19222, 0.5575)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.294→0.4288 / 42.0366→36.4595; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9193 / 0.0→4.4986
- page 3: |Δ| histogram (16 bins, pixel counts) `[1656124, 21504, 18474, 17443, 16049, 16234, 15150, 14889, 14483, 15016, 14301, 13925, 13585, 13576, 13734, 64329]`; ink px ref/ours 112666/88545 (ratio 0.7859); SSIM blocks <0.9: 11626/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -55.5] pt by ink-projection correlation (centroid estimate [-7.93, -47.08] pt); confidence moderate (shift explains 13% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.1259, differing 0.146431, SSIM₈ 0.7057 (raw 21.9804, 0.161834, 0.6331)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4148→0.5307 / 35.1207→30.5591; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy -98.57; `branch` dx -435.52 dy -84.12; `branch` dx -435.52 dy -69.67

### 15-three-page-sections — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610506, 25291, 21030, 20657, 19401, 18347, 19737, 19476, 18882, 17976, 16350, 15988, 15566, 14393, 15459, 69757]`; ink px ref/ours 85700/116938 (ratio 1.3645); SSIM blocks <0.9: 14342/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.12, 38.24] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.3381, differing 0.178198, SSIM₈ 0.5818 (raw 25.0355, 0.188267, 0.534)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2673→0.3353 / 39.3506→37.2025; header-band 1.0→0.9885 / 0.0→0.6238; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606582, 25903, 21170, 20968, 19823, 18854, 20527, 19777, 18992, 18394, 16314, 16116, 15583, 14416, 15528, 69869]`; ink px ref/ours 85737/122724 (ratio 1.4314); SSIM blocks <0.9: 14625/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.18, 12.94] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.2451, differing 0.184973, SSIM₈ 0.5621 (raw 25.2319, 0.190403, 0.5335)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2546→0.3153 / 40.3268→38.045; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9087 / 0.0→4.802
- page 3: |Δ| histogram (16 bins, pixel counts) `[1640989, 23461, 19348, 18999, 17773, 16865, 18108, 18155, 17419, 16063, 14692, 14521, 14163, 12910, 14026, 61324]`; ink px ref/ours 85734/99417 (ratio 1.1596); SSIM blocks <0.9: 13075/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.12, -45.27] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.5358, differing 0.165367, SSIM₈ 0.6023 (raw 22.5158, 0.170978, 0.5805)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3297→0.3661 / 35.9857→34.4139; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -92.03; `oak` dx 426.94 dy -86.44; `oak` dx 426.94 dy -80.85

### 15-three-page-sections — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610327, 25293, 21041, 20672, 19387, 18372, 19737, 19474, 18847, 17969, 16375, 15996, 15603, 14382, 15493, 69848]`; ink px ref/ours 85700/117099 (ratio 1.3664); SSIM blocks <0.9: 14355/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.35, 38.34] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.3608, differing 0.178345, SSIM₈ 0.5814 (raw 25.0553, 0.188391, 0.5336)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2667→0.3348 / 39.3823→37.2236; header-band 1.0→0.9867 / 0.0→0.7294; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606481, 25902, 21184, 20969, 19825, 18858, 20529, 19767, 18983, 18368, 16334, 16122, 15598, 14416, 15560, 69920]`; ink px ref/ours 85737/122799 (ratio 1.4323); SSIM blocks <0.9: 14632/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [3.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.32, 13.08] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.3194, differing 0.185255, SSIM₈ 0.5596 (raw 25.2438, 0.190477, 0.5332)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2542→0.315 / 40.3458→37.9144; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9074 / 0.0→4.802
- page 3: |Δ| histogram (16 bins, pixel counts) `[1640989, 23461, 19348, 18999, 17773, 16865, 18108, 18155, 17419, 16063, 14692, 14521, 14163, 12910, 14026, 61324]`; ink px ref/ours 85734/99417 (ratio 1.1596); SSIM blocks <0.9: 13075/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.12, -45.27] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.5358, differing 0.165367, SSIM₈ 0.6023 (raw 22.5158, 0.170978, 0.5805)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3297→0.3661 / 35.9857→34.4139; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -92.03; `oak` dx 426.94 dy -86.44; `oak` dx 426.94 dy -80.85
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (4): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1652039, 24394, 20659, 19597, 18402, 17570, 17504, 17691, 16894, 15536, 13744, 13239, 12855, 12682, 13004, 53006]`; ink px ref/ours 85700/122582 (ratio 1.4304); SSIM blocks <0.9: 12091/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.11, 28.02] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.7986, differing 0.165835, SSIM₈ 0.6509 (raw 20.9242, 0.16611, 0.6503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.442→0.4444 / 33.4367→33.2311; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1611983, 26396, 21982, 20835, 19906, 18643, 18468, 20136, 19560, 17738, 16624, 15943, 15328, 14884, 15157, 65233]`; ink px ref/ours 85737/126174 (ratio 1.4716); SSIM blocks <0.9: 14678/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -26.5] pt by ink-projection correlation (centroid estimate [1.12, 16.45] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 22.0043, differing 0.173361, SSIM₈ 0.6124 (raw 24.5406, 0.18806, 0.5386)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2636→0.3946 / 39.2168→34.5052; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9195 / 0.0→4.4986
- page 3: |Δ| histogram (16 bins, pixel counts) `[1665015, 22301, 18651, 17636, 17236, 16016, 16385, 17314, 16301, 14794, 13703, 13132, 12549, 12540, 12854, 52389]`; ink px ref/ours 85734/88545 (ratio 1.0328); SSIM blocks <0.9: 12278/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -26.5] pt by ink-projection correlation (centroid estimate [-1.64, -74.44] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 17.8811, differing 0.144017, SSIM₈ 0.6753 (raw 20.3337, 0.157614, 0.6109)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3791→0.4827 / 32.494→28.5706; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -141.19; `oak` dx 450.74 dy -141.19; `oak` dx 450.74 dy -141.19

### 16-heading-page-break — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1535837, 30452, 26147, 24533, 22450, 21872, 22500, 20593, 20315, 21336, 19731, 19135, 19424, 19064, 19858, 95569]`; ink px ref/ours 152381/134585 (ratio 0.8832); SSIM blocks <0.9: 16227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, -14.5] pt REJECTED: applying it gives mean|Δ| 31.6106, not lower; centroid estimate [-5.32, -4.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.604, differing 0.230212, SSIM₈ 0.4867 (raw 31.604, 0.230212, 0.4867)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1802→0.1802 / 50.4999→50.4999; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9989→0.9989 / 0.0437→0.0437
- page 2: |Δ| histogram (16 bins, pixel counts) `[1822153, 8300, 6498, 6892, 5653, 5727, 5850, 5558, 5441, 6299, 5756, 5774, 5950, 5464, 5726, 31775]`; ink px ref/ours 29740/46935 (ratio 1.5782); SSIM blocks <0.9: 5105/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 20.5] pt by ink-projection correlation (centroid estimate [1.73, 56.04] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.5944, differing 0.061631, SSIM₈ 0.8508 (raw 9.5631, 0.06661, 0.8356)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7373→0.7627 / 15.2831→13.6827; header-band 1.0→0.9933 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 127.3; `branch` dx -435.47 dy 74.69; `branch` dx -435.47 dy 54.75

### 16-heading-page-break — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1535837, 30452, 26147, 24533, 22450, 21872, 22500, 20593, 20315, 21336, 19731, 19135, 19424, 19064, 19858, 95569]`; ink px ref/ours 152381/134585 (ratio 0.8832); SSIM blocks <0.9: 16227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, -14.5] pt REJECTED: applying it gives mean|Δ| 31.6106, not lower; centroid estimate [-5.32, -4.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.604, differing 0.230212, SSIM₈ 0.4867 (raw 31.604, 0.230212, 0.4867)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1802→0.1802 / 50.4999→50.4999; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9989→0.9989 / 0.0437→0.0437
- page 2: |Δ| histogram (16 bins, pixel counts) `[1822026, 8348, 6537, 6907, 5642, 5709, 5832, 5577, 5442, 6271, 5753, 5755, 5955, 5480, 5744, 31838]`; ink px ref/ours 29740/46988 (ratio 1.58); SSIM blocks <0.9: 5109/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 20.5] pt by ink-projection correlation (centroid estimate [1.84, 56.09] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.6035, differing 0.061697, SSIM₈ 0.8506 (raw 9.5727, 0.066689, 0.8354)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7371→0.7624 / 15.2984→13.6973; header-band 1.0→0.9933 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 127.3; `branch` dx -435.47 dy 74.69; `branch` dx -435.47 dy 54.75
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1583136, 30910, 26298, 23155, 21464, 20385, 19396, 18988, 18535, 18614, 17496, 16515, 16559, 16202, 16455, 74708]`; ink px ref/ours 152381/134174 (ratio 0.8805); SSIM blocks <0.9: 14354/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 26.6859, not lower; centroid estimate [2.53, -11.22] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.6805, differing 0.205375, SSIM₈ 0.5916 (raw 26.6805, 0.205375, 0.5916)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3488→0.3488 / 42.6251→42.6251; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9989→0.9989 / 0.0437→0.0437
- page 2: |Δ| histogram (16 bins, pixel counts) `[1831867, 8230, 7004, 6298, 5907, 5714, 5222, 5685, 4953, 5610, 5125, 5068, 5019, 4786, 5118, 27210]`; ink px ref/ours 29740/46136 (ratio 1.5513); SSIM blocks <0.9: 4930/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 57.5] pt by ink-projection correlation (centroid estimate [10.92, 34.58] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.3596, differing 0.060886, SSIM₈ 0.8505 (raw 8.4921, 0.061562, 0.8493)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7595→0.8182 / 13.5694→10.5633; header-band 1.0→0.6092 / 0.0→19.2344; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 100.37; `branch` dx -435.47 dy 58.19; `branch` dx -435.47 dy 43.74

### 16-heading-page-break — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571209, 29050, 24879, 23374, 22427, 21235, 22990, 22122, 21052, 20710, 18292, 17708, 17105, 16387, 17019, 73257]`; ink px ref/ours 104967/134585 (ratio 1.2822); SSIM blocks <0.9: 15666/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.39, -3.28] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.439, differing 0.210412, SSIM₈ 0.5039 (raw 27.547, 0.210718, 0.5025)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.2081 / 44.0152→43.8425; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1803988, 9978, 8468, 8179, 7716, 7346, 7872, 7659, 7179, 7375, 6674, 6476, 6724, 6021, 6417, 30744]`; ink px ref/ours 33407/46935 (ratio 1.4049); SSIM blocks <0.9: 6193/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 20.0] pt by ink-projection correlation (centroid estimate [-3.15, 25.13] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.6577, differing 0.066786, SSIM₈ 0.847 (raw 10.4893, 0.07714, 0.7988)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6786→0.7566 / 16.7641→13.7846; header-band 1.0→0.9928 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 26.59; `oak` dx 426.94 dy 20.1; `oak` dx 426.94 dy -14.75

### 16-heading-page-break — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571209, 29050, 24879, 23374, 22427, 21235, 22990, 22122, 21052, 20710, 18292, 17708, 17105, 16387, 17019, 73257]`; ink px ref/ours 104967/134585 (ratio 1.2822); SSIM blocks <0.9: 15666/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.39, -3.28] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.439, differing 0.210412, SSIM₈ 0.5039 (raw 27.547, 0.210718, 0.5025)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.2081 / 44.0152→43.8425; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1803821, 10027, 8506, 8198, 7698, 7316, 7848, 7679, 7178, 7366, 6682, 6479, 6723, 6037, 6428, 30830]`; ink px ref/ours 33407/46988 (ratio 1.4065); SSIM blocks <0.9: 6197/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 20.0] pt by ink-projection correlation (centroid estimate [-3.03, 25.18] pt); confidence moderate (shift explains 18% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.6644, differing 0.06685, SSIM₈ 0.8469 (raw 10.5033, 0.077221, 0.7987)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6785→0.7566 / 16.7866→13.7953; header-band 1.0→0.9928 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 26.59; `oak` dx 426.94 dy 20.1; `oak` dx 426.94 dy -14.75
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1612270, 28080, 24275, 22255, 21604, 20289, 20381, 20180, 18813, 18052, 16129, 15291, 14445, 14586, 14330, 57836]`; ink px ref/ours 104967/134174 (ratio 1.2782); SSIM blocks <0.9: 13677/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.5, 0.0] pt by ink-projection correlation (centroid estimate [3.45, -10.45] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.4287, differing 0.188564, SSIM₈ 0.6061 (raw 23.5751, 0.189149, 0.6073)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3745→0.3765 / 37.6606→37.262; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1827928, 9266, 8175, 7155, 7129, 6688, 6428, 6808, 6007, 5866, 5329, 5281, 5031, 4836, 4904, 21985]`; ink px ref/ours 33407/46136 (ratio 1.381); SSIM blocks <0.9: 4814/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 14.5] pt by ink-projection correlation (centroid estimate [6.05, 3.67] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.9506, differing 0.062877, SSIM₈ 0.8679 (raw 8.2032, 0.064102, 0.859)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7751→0.803 / 13.1083→12.0038; header-band 1.0→0.9079 / 0.0→4.8172; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -14.8; `oak` dx 450.74 dy -14.8; `oak` dx 450.74 dy -14.8

### 16-heading-page-break — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/16-heading-page-break/pdflatex-de1020c-export-p1-overlay.png) (56597 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-de1020c-export-p1-heatmap.png) (38374 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -14.5] pt by ink-projection correlation (centroid estimate [-6.07, -5.06] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.4928, differing 0.229367, SSIM₈ 0.4891 (raw 31.5764, 0.22951, 0.4881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1821→0.1892 / 50.4615→50.1371; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.96 / 0.0468→1.3594
- page 2: |Δ| histogram (16 bins, pixel counts) `[1821555, 8081, 6675, 6513, 5830, 5892, 6026, 5614, 5619, 6564, 5700, 5695, 5919, 5335, 5734, 32064]`; ink px ref/ours 29765/46935 (ratio 1.5769); SSIM blocks <0.9: 5085/30294; [overlay](images/16-heading-page-break/pdflatex-de1020c-export-p2-overlay.png) (53188 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-de1020c-export-p2-heatmap.png) (47611 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 20.5] pt by ink-projection correlation (centroid estimate [1.21, 55.26] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.5969, differing 0.061448, SSIM₈ 0.8517 (raw 9.6191, 0.066682, 0.8358)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7375→0.764 / 15.3741→13.6882; header-band 1.0→0.9933 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 127.11; `branch` dx -435.47 dy 74.69; `branch` dx -435.47 dy 54.75

### 16-heading-page-break — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/16-heading-page-break/pdflatex-main-export-p1-overlay.png) (56402 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-main-export-p1-heatmap.png) (38186 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -14.5] pt by ink-projection correlation (centroid estimate [-6.07, -5.06] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.4928, differing 0.229367, SSIM₈ 0.4891 (raw 31.5764, 0.22951, 0.4881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1821→0.1892 / 50.4615→50.1371; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.96 / 0.0468→1.3594
- page 2: |Δ| histogram (16 bins, pixel counts) `[1821428, 8129, 6714, 6528, 5819, 5874, 6008, 5633, 5620, 6536, 5697, 5676, 5924, 5351, 5752, 32127]`; ink px ref/ours 29765/46988 (ratio 1.5786); SSIM blocks <0.9: 5089/30294; [overlay](images/16-heading-page-break/pdflatex-main-export-p2-overlay.png) (53122 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-main-export-p2-heatmap.png) (47589 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 20.5] pt by ink-projection correlation (centroid estimate [1.33, 55.31] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.6068, differing 0.061521, SSIM₈ 0.8515 (raw 9.6287, 0.066762, 0.8356)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7373→0.7637 / 15.3895→13.7041; header-band 1.0→0.9933 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 127.11; `branch` dx -435.47 dy 74.69; `branch` dx -435.47 dy 54.75
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1587936, 30510, 26093, 22620, 21328, 20753, 19095, 19085, 18279, 18386, 17021, 16649, 15827, 15615, 16048, 73571]`; ink px ref/ours 151753/134174 (ratio 0.8842); SSIM blocks <0.9: 14216/30294; [overlay](images/16-heading-page-break/pdflatex-pipeline-export-p1-overlay.png) (56656 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-pipeline-export-p1-heatmap.png) (36363 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [1.77, -12.23] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.254, differing 0.202987, SSIM₈ 0.5996 (raw 26.254, 0.202987, 0.5996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3614→0.3614 / 41.9484→41.9484; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.9984 / 0.0468→0.0468
- page 2: |Δ| histogram (16 bins, pixel counts) `[1831859, 7933, 7188, 6195, 6133, 5872, 5434, 5717, 5240, 5673, 5050, 4915, 4907, 4656, 5069, 26975]`; ink px ref/ours 29765/46136 (ratio 1.55); SSIM blocks <0.9: 4902/30294; [overlay](images/16-heading-page-break/pdflatex-pipeline-export-p2-overlay.png) (53538 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-pipeline-export-p2-heatmap.png) (45148 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 57.5] pt by ink-projection correlation (centroid estimate [10.41, 33.8] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.4011, differing 0.060758, SSIM₈ 0.8511 (raw 8.4568, 0.061423, 0.851)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7622→0.819 / 13.5144→10.6311; header-band 1.0→0.6092 / 0.0→19.2344; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 100.18; `branch` dx -435.47 dy 58.19; `branch` dx -435.47 dy 43.74

### 16-heading-page-break — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/16-heading-page-break/pdflatex-lm-de1020c-export-p1-overlay.png) (55093 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-de1020c-export-p1-heatmap.png) (37651 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.44, -3.32] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4403, differing 0.210395, SSIM₈ 0.5038 (raw 27.5467, 0.210674, 0.5026)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.208 / 44.0147→43.8447; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1804006, 9924, 8335, 8540, 7532, 7526, 7743, 7505, 7333, 7253, 6797, 6490, 6663, 5901, 6567, 30701]`; ink px ref/ours 33505/46935 (ratio 1.4008); SSIM blocks <0.9: 6195/30294; [overlay](images/16-heading-page-break/pdflatex-lm-de1020c-export-p2-overlay.png) (59799 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-de1020c-export-p2-heatmap.png) (53033 B, ÷4)
  - registration error (diagnostic): global shift [-1.0, 20.0] pt by ink-projection correlation (centroid estimate [-2.85, 25.0] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.6575, differing 0.066846, SSIM₈ 0.8469 (raw 10.4882, 0.077169, 0.7985)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6782→0.7566 / 16.7624→13.7844; header-band 1.0→0.9928 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 27.65; `oak` dx 426.94 dy 21.13; `oak` dx 426.94 dy -13.72

### 16-heading-page-break — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/16-heading-page-break/pdflatex-lm-main-export-p1-overlay.png) (54900 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-main-export-p1-heatmap.png) (37449 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.44, -3.32] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4403, differing 0.210395, SSIM₈ 0.5038 (raw 27.5467, 0.210674, 0.5026)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.208 / 44.0147→43.8447; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1803847, 9966, 8356, 8579, 7516, 7490, 7720, 7518, 7334, 7248, 6813, 6489, 6653, 5923, 6576, 30788]`; ink px ref/ours 33505/46988 (ratio 1.4024); SSIM blocks <0.9: 6199/30294; [overlay](images/16-heading-page-break/pdflatex-lm-main-export-p2-overlay.png) (59766 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-main-export-p2-heatmap.png) (52964 B, ÷4)
  - registration error (diagnostic): global shift [-1.0, 20.0] pt by ink-projection correlation (centroid estimate [-2.74, 25.04] pt); confidence moderate (shift explains 18% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.664, differing 0.06691, SSIM₈ 0.8469 (raw 10.5022, 0.077249, 0.7984)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.678→0.7565 / 16.7848→13.7947; header-band 1.0→0.9928 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 27.65; `oak` dx 426.94 dy 21.13; `oak` dx 426.94 dy -13.72
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1612433, 27856, 24167, 22468, 21846, 20013, 20505, 19937, 18971, 17855, 16393, 15247, 14544, 14310, 14602, 57669]`; ink px ref/ours 105111/134174 (ratio 1.2765); SSIM blocks <0.9: 13679/30294; [overlay](images/16-heading-page-break/pdflatex-lm-pipeline-export-p1-overlay.png) (55023 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-pipeline-export-p1-heatmap.png) (35381 B, ÷8)
  - registration error (diagnostic): global shift [-2.5, 0.0] pt by ink-projection correlation (centroid estimate [3.4, -10.49] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.4215, differing 0.188519, SSIM₈ 0.6062 (raw 23.5734, 0.189081, 0.6073)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3745→0.3767 / 37.6579→37.2505; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1827942, 9288, 8013, 7407, 7076, 6805, 6289, 6695, 6121, 5833, 5396, 5272, 4992, 4686, 5089, 21912]`; ink px ref/ours 33505/46136 (ratio 1.377); SSIM blocks <0.9: 4815/30294; [overlay](images/16-heading-page-break/pdflatex-lm-pipeline-export-p2-overlay.png) (61095 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-pipeline-export-p2-heatmap.png) (44026 B, ÷4)
  - registration error (diagnostic): global shift [0.5, 14.5] pt by ink-projection correlation (centroid estimate [6.34, 3.53] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.9425, differing 0.062894, SSIM₈ 0.868 (raw 8.199, 0.064111, 0.8591)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7754→0.8032 / 13.1017→11.9908; header-band 1.0→0.9079 / 0.0→4.8172; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -13.78; `oak` dx 450.74 dy -13.78; `oak` dx 450.74 dy -13.78

### 16-heading-page-break — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1535378, 29989, 25555, 24965, 22556, 22246, 22654, 20669, 20453, 21240, 20149, 19662, 19349, 18997, 19406, 95548]`; ink px ref/ours 152232/134585 (ratio 0.8841); SSIM blocks <0.9: 16215/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, -14.5] pt by ink-projection correlation (centroid estimate [-5.35, -4.01] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.6466, differing 0.23054, SSIM₈ 0.4854 (raw 31.6507, 0.230423, 0.4864)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1798→0.1843 / 50.5745→50.3662; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9987→0.9601 / 0.0438→1.3685
- page 2: |Δ| histogram (16 bins, pixel counts) `[1822074, 8238, 6407, 6960, 5733, 5829, 5829, 5447, 5415, 6355, 5865, 5944, 5803, 5413, 5801, 31703]`; ink px ref/ours 29777/46935 (ratio 1.5762); SSIM blocks <0.9: 5105/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 20.5] pt by ink-projection correlation (centroid estimate [1.68, 56.11] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.62, differing 0.061704, SSIM₈ 0.8505 (raw 9.5693, 0.066627, 0.8355)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7372→0.7622 / 15.293→13.7236; header-band 1.0→0.9933 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 127.3; `branch` dx -435.52 dy 74.69; `branch` dx -435.52 dy 54.75

### 16-heading-page-break — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1535378, 29989, 25555, 24965, 22556, 22246, 22654, 20669, 20453, 21240, 20149, 19662, 19349, 18997, 19406, 95548]`; ink px ref/ours 152232/134585 (ratio 0.8841); SSIM blocks <0.9: 16215/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, -14.5] pt by ink-projection correlation (centroid estimate [-5.35, -4.01] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.6466, differing 0.23054, SSIM₈ 0.4854 (raw 31.6507, 0.230423, 0.4864)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1798→0.1843 / 50.5745→50.3662; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9987→0.9601 / 0.0438→1.3685
- page 2: |Δ| histogram (16 bins, pixel counts) `[1821947, 8286, 6446, 6975, 5722, 5811, 5811, 5466, 5416, 6327, 5862, 5925, 5808, 5429, 5819, 31766]`; ink px ref/ours 29777/46988 (ratio 1.578); SSIM blocks <0.9: 5109/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 20.5] pt by ink-projection correlation (centroid estimate [1.79, 56.15] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.6293, differing 0.061772, SSIM₈ 0.8504 (raw 9.5789, 0.066707, 0.8354)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.737→0.7619 / 15.3083→13.7384; header-band 1.0→0.9933 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 127.3; `branch` dx -435.52 dy 74.69; `branch` dx -435.52 dy 54.75
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1583745, 30278, 25836, 23637, 22043, 20918, 19528, 18857, 18576, 18283, 17742, 17071, 16276, 16308, 15958, 73760]`; ink px ref/ours 152232/134174 (ratio 0.8814); SSIM blocks <0.9: 14338/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.49, -11.18] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.5737, differing 0.205156, SSIM₈ 0.5932 (raw 26.5737, 0.205156, 0.5932)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3513→0.3513 / 42.4542→42.4542; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9987→0.9987 / 0.0438→0.0438
- page 2: |Δ| histogram (16 bins, pixel counts) `[1831785, 8100, 7018, 6389, 5954, 5846, 5116, 5657, 4959, 5572, 5208, 5215, 4937, 4725, 5186, 27149]`; ink px ref/ours 29777/46136 (ratio 1.5494); SSIM blocks <0.9: 4929/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 57.5] pt by ink-projection correlation (centroid estimate [10.87, 34.64] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.3348, differing 0.060806, SSIM₈ 0.8508 (raw 8.4972, 0.061583, 0.8491)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7593→0.8187 / 13.5774→10.5235; header-band 1.0→0.6092 / 0.0→19.2344; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 100.37; `branch` dx -435.52 dy 58.19; `branch` dx -435.52 dy 43.74

### 16-heading-page-break — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571173, 29128, 24898, 23367, 22271, 21337, 23030, 22042, 21136, 20589, 18380, 17673, 17118, 16399, 17001, 73274]`; ink px ref/ours 105020/134585 (ratio 1.2815); SSIM blocks <0.9: 15670/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.46, -3.21] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4386, differing 0.210358, SSIM₈ 0.5039 (raw 27.5472, 0.210665, 0.5025)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2059→0.2081 / 44.0154→43.8419; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1803993, 9979, 8502, 8146, 7678, 7363, 7893, 7643, 7204, 7342, 6717, 6469, 6695, 6027, 6420, 30745]`; ink px ref/ours 33418/46935 (ratio 1.4045); SSIM blocks <0.9: 6193/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 20.0] pt by ink-projection correlation (centroid estimate [-3.1, 25.13] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.6575, differing 0.066761, SSIM₈ 0.847 (raw 10.4892, 0.077113, 0.7988)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6786→0.7566 / 16.7641→13.7843; header-band 1.0→0.9928 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 27.62; `oak` dx 426.94 dy 21.13; `oak` dx 426.94 dy -13.72

### 16-heading-page-break — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571173, 29128, 24898, 23367, 22271, 21337, 23030, 22042, 21136, 20589, 18380, 17673, 17118, 16399, 17001, 73274]`; ink px ref/ours 105020/134585 (ratio 1.2815); SSIM blocks <0.9: 15670/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.46, -3.21] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4386, differing 0.210358, SSIM₈ 0.5039 (raw 27.5472, 0.210665, 0.5025)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2059→0.2081 / 44.0154→43.8419; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1803826, 10028, 8540, 8165, 7660, 7333, 7869, 7663, 7203, 7333, 6725, 6472, 6694, 6043, 6431, 30831]`; ink px ref/ours 33418/46988 (ratio 1.4061); SSIM blocks <0.9: 6197/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 20.0] pt by ink-projection correlation (centroid estimate [-2.98, 25.18] pt); confidence moderate (shift explains 18% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.6642, differing 0.066825, SSIM₈ 0.8469 (raw 10.5033, 0.077194, 0.7987)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6785→0.7566 / 16.7865→13.795; header-band 1.0→0.9928 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 27.62; `oak` dx 426.94 dy 21.13; `oak` dx 426.94 dy -13.72
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document; error: Latin Modern face unavailable (lmroman12-bold.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1612227, 28099, 24375, 22218, 21483, 20425, 20378, 20120, 18848, 18013, 16182, 15257, 14451, 14534, 14340, 57866]`; ink px ref/ours 105020/134174 (ratio 1.2776); SSIM blocks <0.9: 13680/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.5, 0.0] pt by ink-projection correlation (centroid estimate [3.38, -10.38] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.4266, differing 0.188488, SSIM₈ 0.6061 (raw 23.5741, 0.189094, 0.6074)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3745→0.3765 / 37.659→37.2586; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1827914, 9291, 8204, 7130, 7086, 6720, 6435, 6781, 6020, 5842, 5366, 5288, 5018, 4839, 4895, 21987]`; ink px ref/ours 33418/46136 (ratio 1.3806); SSIM blocks <0.9: 4814/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 14.5] pt by ink-projection correlation (centroid estimate [6.09, 3.67] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.9501, differing 0.062856, SSIM₈ 0.8679 (raw 8.2029, 0.064076, 0.859)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7751→0.803 / 13.1079→12.003; header-band 1.0→0.9079 / 0.0→4.8172; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 450.74 dy -13.78; `oak` dx 450.74 dy -13.78; `oak` dx 450.74 dy -13.78

### 17-apostrophes — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927391, 1130, 902, 756, 663, 601, 596, 579, 591, 504, 530, 551, 566, 535, 531, 2390]`; ink px ref/ours 5219/5099 (ratio 0.977); SSIM blocks <0.9: 459/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-5.57, 0.01] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8119, differing 0.00674, SSIM₈ 0.9882 (raw 0.8502, 0.006811, 0.9875)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9801→0.9811 / 1.3589→1.2977; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -2.93 dy 0.42; `quotes,` dx -1.49 dy 0.42; `and` dx -1.35 dy 0.42
- word-sequence differences: replace ref ['It’stheowl’sbranch;don’t,can’t,won’t,o’clock,rock’n’roll,the’90s,Muller’sresume,“quoted”'] ours ["It's", 'the', "owl's", 'branch;', "don't,", "can't,", "won't,", "o'clock,"]; replace ref ['‘single’'] ours ["single'"]; replace ref ['’'] ours ["'"]

### 17-apostrophes — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927391, 1130, 902, 756, 663, 601, 596, 579, 591, 504, 530, 551, 566, 535, 531, 2390]`; ink px ref/ours 5219/5099 (ratio 0.977); SSIM blocks <0.9: 459/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-5.57, 0.01] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8119, differing 0.00674, SSIM₈ 0.9882 (raw 0.8502, 0.006811, 0.9875)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9801→0.9811 / 1.3589→1.2977; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -2.93 dy 0.42; `quotes,` dx -1.49 dy 0.42; `and` dx -1.35 dy 0.42
- word-sequence differences: replace ref ['It’stheowl’sbranch;don’t,can’t,won’t,o’clock,rock’n’roll,the’90s,Muller’sresume,“quoted”'] ours ["It's", 'the', "owl's", 'branch;', "don't,", "can't,", "won't,", "o'clock,"]; replace ref ['‘single’'] ours ["single'"]; replace ref ['’'] ours ["'"]

### 17-apostrophes — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926286, 1103, 974, 775, 719, 626, 613, 675, 606, 641, 631, 563, 668, 634, 616, 2686]`; ink px ref/ours 5219/5148 (ratio 0.9864); SSIM blocks <0.9: 546/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-20.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.9861, not lower; centroid estimate [4.92, 0.92] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9542, differing 0.007376, SSIM₈ 0.985 (raw 0.9542, 0.007376, 0.985)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9761→0.9761 / 1.5251→1.5251; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx 54.26 dy 0.41; `’` dx 53.35 dy 0.41; `plain` dx 52.43 dy 0.41
- word-sequence differences: replace ref ['It’stheowl’sbranch;don’t,can’t,won’t,o’clock,rock’n’roll,the’90s,Muller’sresume,“quoted”'] ours ['It’s', 'the', 'owl’s', 'branch;', 'don’t,', 'can’t,', 'won’t,', 'o’clock,']

### 17-apostrophes — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926769, 1003, 928, 840, 817, 779, 749, 812, 699, 601, 585, 566, 552, 538, 509, 2069]`; ink px ref/ours 3891/5099 (ratio 1.3105); SSIM blocks <0.9: 564/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.57, -1.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8622, differing 0.007037, SSIM₈ 0.9848 (raw 0.8622, 0.007037, 0.9848)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9757→0.9757 / 1.378→1.378; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.51 dy -0.35; `plain` dx -61.26 dy -0.35; `a` dx -59.83 dy -0.35
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926769, 1003, 928, 840, 817, 779, 749, 812, 699, 601, 585, 566, 552, 538, 509, 2069]`; ink px ref/ours 3891/5099 (ratio 1.3105); SSIM blocks <0.9: 564/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.57, -1.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8622, differing 0.007037, SSIM₈ 0.9848 (raw 0.8622, 0.007037, 0.9848)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9757→0.9757 / 1.378→1.378; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.51 dy -0.35; `plain` dx -61.26 dy -0.35; `a` dx -59.83 dy -0.35
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927279, 975, 823, 856, 830, 742, 752, 705, 621, 561, 559, 551, 542, 495, 482, 2043]`; ink px ref/ours 3891/5148 (ratio 1.3231); SSIM blocks <0.9: 509/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.08, -0.23] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7676, differing 0.00656, SSIM₈ 0.9874 (raw 0.8283, 0.006765, 0.9858)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9773→0.98 / 1.3238→1.225; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `’` dx -9.05 dy -0.36; `apostrophe.` dx -8.32 dy -0.36; `plain` dx -7.6 dy -0.36

### 17-apostrophes — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927689, 1087, 914, 809, 742, 653, 583, 605, 537, 460, 552, 502, 496, 521, 444, 2222]`; ink px ref/ours 5173/5099 (ratio 0.9857); SSIM blocks <0.9: 463/30294; [overlay](images/17-apostrophes/pdflatex-de1020c-export-p1-overlay.png) (43546 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-de1020c-export-p1-heatmap.png) (89187 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-7.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8459, not lower; centroid estimate [-5.05, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8063, differing 0.006638, SSIM₈ 0.9887 (raw 0.8063, 0.006638, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9819→0.9819 / 1.2887→1.2887; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -7.83 dy 0.46; `roll,` dx -6.97 dy 0.46; `the` dx -6.5 dy 0.46
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927689, 1087, 914, 809, 742, 653, 583, 605, 537, 460, 552, 502, 496, 521, 444, 2222]`; ink px ref/ours 5173/5099 (ratio 0.9857); SSIM blocks <0.9: 463/30294; [overlay](images/17-apostrophes/pdflatex-main-export-p1-overlay.png) (43196 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-main-export-p1-heatmap.png) (88776 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-7.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8459, not lower; centroid estimate [-5.05, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8063, differing 0.006638, SSIM₈ 0.9887 (raw 0.8063, 0.006638, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9819→0.9819 / 1.2887→1.2887; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -7.83 dy 0.46; `roll,` dx -6.97 dy 0.46; `the` dx -6.5 dy 0.46
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926384, 1063, 857, 755, 765, 660, 590, 637, 619, 598, 669, 602, 660, 676, 581, 2700]`; ink px ref/ours 5173/5148 (ratio 0.9952); SSIM blocks <0.9: 543/30294; [overlay](images/17-apostrophes/pdflatex-pipeline-export-p1-overlay.png) (43742 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-pipeline-export-p1-heatmap.png) (37965 B, ÷2)
  - registration error (diagnostic): global shift [9.0, 0.0] pt by ink-projection correlation (centroid estimate [5.44, 0.88] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.931, differing 0.007228, SSIM₈ 0.9858 (raw 0.9556, 0.007307, 0.9853)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9765→0.9779 / 1.5274→1.4617; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `“quoted”` dx -424.84 dy 14.85; `apostrophe.` dx 54.37 dy 0.41; `’` dx 53.45 dy 0.41

### 17-apostrophes — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926770, 1005, 918, 847, 826, 791, 712, 833, 730, 582, 566, 566, 556, 521, 500, 2093]`; ink px ref/ours 3904/5099 (ratio 1.3061); SSIM blocks <0.9: 564/30294; [overlay](images/17-apostrophes/pdflatex-lm-de1020c-export-p1-overlay.png) (44426 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-lm-de1020c-export-p1-heatmap.png) (38053 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.84, -1.09] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8624, differing 0.00704, SSIM₈ 0.9848 (raw 0.8624, 0.00704, 0.9848)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9757→0.9757 / 1.3784→1.3784; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.51 dy 0.68; `plain` dx -61.24 dy 0.68; `a` dx -59.82 dy 0.68
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926770, 1005, 918, 847, 826, 791, 712, 833, 730, 582, 566, 566, 556, 521, 500, 2093]`; ink px ref/ours 3904/5099 (ratio 1.3061); SSIM blocks <0.9: 564/30294; [overlay](images/17-apostrophes/pdflatex-lm-main-export-p1-overlay.png) (44131 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-lm-main-export-p1-heatmap.png) (37777 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.84, -1.09] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8624, differing 0.00704, SSIM₈ 0.9848 (raw 0.8624, 0.00704, 0.9848)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9757→0.9757 / 1.3784→1.3784; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.51 dy 0.68; `plain` dx -61.24 dy 0.68; `a` dx -59.82 dy 0.68
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927271, 1001, 806, 835, 865, 745, 725, 725, 622, 565, 545, 519, 589, 473, 470, 2060]`; ink px ref/ours 3904/5148 (ratio 1.3186); SSIM blocks <0.9: 509/30294; [overlay](images/17-apostrophes/pdflatex-lm-pipeline-export-p1-overlay.png) (44121 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-lm-pipeline-export-p1-heatmap.png) (88562 B, ÷1)
  - registration error (diagnostic): global shift [-2.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.35, -0.18] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7673, differing 0.006568, SSIM₈ 0.9874 (raw 0.8285, 0.006773, 0.9858)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9773→0.98 / 1.3242→1.2245; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `’` dx -9.03 dy 0.67; `apostrophe.` dx -8.31 dy 0.67; `plain` dx -7.58 dy 0.67

### 17-apostrophes — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927755, 1202, 1003, 779, 683, 594, 527, 566, 500, 452, 495, 487, 558, 563, 513, 2139]`; ink px ref/ours 5210/5099 (ratio 0.9787); SSIM blocks <0.9: 446/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-7.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8494, not lower; centroid estimate [-3.78, -0.06] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7981, differing 0.006649, SSIM₈ 0.9883 (raw 0.7981, 0.006649, 0.9883)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9813→0.9813 / 1.2756→1.2756; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -7.91 dy 0.46; `roll,` dx -6.83 dy 0.46; `the` dx -6.31 dy 0.46
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927755, 1202, 1003, 779, 683, 594, 527, 566, 500, 452, 495, 487, 558, 563, 513, 2139]`; ink px ref/ours 5210/5099 (ratio 0.9787); SSIM blocks <0.9: 446/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-7.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8494, not lower; centroid estimate [-3.78, -0.06] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7981, differing 0.006649, SSIM₈ 0.9883 (raw 0.7981, 0.006649, 0.9883)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9813→0.9813 / 1.2756→1.2756; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -7.91 dy 0.46; `roll,` dx -6.83 dy 0.46; `the` dx -6.31 dy 0.46
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926262, 1068, 1011, 795, 693, 624, 638, 659, 625, 590, 634, 592, 703, 621, 640, 2661]`; ink px ref/ours 5210/5148 (ratio 0.9881); SSIM blocks <0.9: 551/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [6.5, 0.0] pt by ink-projection correlation (centroid estimate [6.71, 0.85] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9447, differing 0.007356, SSIM₈ 0.9853 (raw 0.9561, 0.00737, 0.9849)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9759→0.9769 / 1.5281→1.488; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `“quoted”` dx -424.88 dy 14.85; `apostrophe.` dx 54.25 dy 0.41; `’` dx 53.33 dy 0.41

### 17-apostrophes — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926762, 1016, 928, 836, 824, 766, 758, 808, 697, 605, 580, 562, 563, 533, 517, 2061]`; ink px ref/ours 3889/5099 (ratio 1.3111); SSIM blocks <0.9: 564/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.04, -1.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.862, differing 0.007035, SSIM₈ 0.9848 (raw 0.862, 0.007035, 0.9848)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9757→0.9757 / 1.3777→1.3777; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.51 dy 0.68; `plain` dx -61.26 dy 0.68; `a` dx -59.83 dy 0.68
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926762, 1016, 928, 836, 824, 766, 758, 808, 697, 605, 580, 562, 563, 533, 517, 2061]`; ink px ref/ours 3889/5099 (ratio 1.3111); SSIM blocks <0.9: 564/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.04, -1.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.862, differing 0.007035, SSIM₈ 0.9848 (raw 0.862, 0.007035, 0.9848)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9757→0.9757 / 1.3777→1.3777; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.51 dy 0.68; `plain` dx -61.26 dy 0.68; `a` dx -59.83 dy 0.68
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927276, 984, 832, 841, 837, 734, 756, 696, 626, 558, 558, 552, 546, 506, 481, 2033]`; ink px ref/ours 3889/5148 (ratio 1.3237); SSIM blocks <0.9: 509/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.55, -0.23] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7675, differing 0.00656, SSIM₈ 0.9874 (raw 0.8283, 0.006764, 0.9858)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9773→0.98 / 1.3239→1.2248; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `’` dx -9.05 dy 0.67; `apostrophe.` dx -8.32 dy 0.67; `plain` dx -7.6 dy 0.67

### 18-ligatures — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920329, 1644, 1438, 1260, 1111, 1044, 1010, 908, 954, 966, 919, 985, 1012, 775, 800, 3661]`; ink px ref/ours 8390/8297 (ratio 0.9889); SSIM blocks <0.9: 773/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-12.52, 1.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3714, differing 0.01092, SSIM₈ 0.979 (raw 1.3714, 0.01092, 0.979)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9664→0.9664 / 2.192→2.192; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -444.42 dy 14.86; `ruffled,` dx -432.48 dy 14.82; `muffin.` dx 39.04 dy 0.37

### 18-ligatures — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920329, 1644, 1438, 1260, 1111, 1044, 1010, 908, 954, 966, 919, 985, 1012, 775, 800, 3661]`; ink px ref/ours 8390/8297 (ratio 0.9889); SSIM blocks <0.9: 773/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-12.52, 1.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3714, differing 0.01092, SSIM₈ 0.979 (raw 1.3714, 0.01092, 0.979)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9664→0.9664 / 2.192→2.192; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -444.42 dy 14.86; `ruffled,` dx -432.48 dy 14.82; `muffin.` dx 39.04 dy 0.37

### 18-ligatures — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922422, 1769, 1428, 1228, 1010, 1014, 1026, 843, 785, 762, 799, 735, 698, 661, 722, 2914]`; ink px ref/ours 8390/8202 (ratio 0.9776); SSIM blocks <0.9: 634/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [0.53, -0.01] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0484, differing 0.009456, SSIM₈ 0.9858 (raw 1.1489, 0.009843, 0.984)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9746→0.9775 / 1.8339→1.674; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fluffy,` dx 1.78 dy 0.41; `waffle,` dx 1.71 dy 0.41; `efficient,` dx 1.53 dy 0.41

### 18-ligatures — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920361, 1636, 1302, 1341, 1350, 1189, 1095, 1088, 1119, 970, 874, 974, 952, 771, 683, 3111]`; ink px ref/ours 6488/8297 (ratio 1.2788); SSIM blocks <0.9: 799/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-18.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3339, not lower; centroid estimate [-5.38, -0.31] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3134, differing 0.010683, SSIM₈ 0.9787 (raw 1.3134, 0.010683, 0.9787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.0992→2.0992; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `coffin,` dx -21.56 dy -0.3; `official,` dx -20.56 dy -0.3; `offline,` dx -20.28 dy -0.35

### 18-ligatures — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920361, 1636, 1302, 1341, 1350, 1189, 1095, 1088, 1119, 970, 874, 974, 952, 771, 683, 3111]`; ink px ref/ours 6488/8297 (ratio 1.2788); SSIM blocks <0.9: 799/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-18.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3339, not lower; centroid estimate [-5.38, -0.31] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3134, differing 0.010683, SSIM₈ 0.9787 (raw 1.3134, 0.010683, 0.9787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.0992→2.0992; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `coffin,` dx -21.56 dy -0.3; `official,` dx -20.56 dy -0.3; `offline,` dx -20.28 dy -0.35

### 18-ligatures — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920275, 1602, 1256, 1368, 1285, 1207, 1189, 1145, 1117, 948, 879, 916, 921, 758, 795, 3155]`; ink px ref/ours 6488/8202 (ratio 1.2642); SSIM blocks <0.9: 805/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [11.0, 0.0] pt REJECTED: applying it gives mean|Δ| 1.4136, not lower; centroid estimate [7.67, -1.33] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3281, differing 0.010764, SSIM₈ 0.9782 (raw 1.3281, 0.010764, 0.9782)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9654→0.9654 / 2.1202→2.1202; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx 445.86 dy -14.8; `ruffled,` dx 433.14 dy -14.8; `muffin.` dx -43.38 dy -0.36

### 18-ligatures — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921093, 1851, 1396, 1273, 1121, 942, 936, 833, 874, 895, 836, 811, 990, 726, 718, 3521]`; ink px ref/ours 8210/8297 (ratio 1.0106); SSIM blocks <0.9: 724/30294; [overlay](images/18-ligatures/pdflatex-de1020c-export-p1-overlay.png) (50200 B, ÷2), [heatmap](images/18-ligatures/pdflatex-de1020c-export-p1-heatmap.png) (42457 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.13, 0.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2894, differing 0.010507, SSIM₈ 0.9808 (raw 1.2894, 0.010507, 0.9808)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9693→0.9693 / 2.0608→2.0608; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.86 dy 14.86; `ruffled,` dx -433.44 dy 14.82; `muffin.` dx 40.63 dy 0.37

### 18-ligatures — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921093, 1851, 1396, 1273, 1121, 942, 936, 833, 874, 895, 836, 811, 990, 726, 718, 3521]`; ink px ref/ours 8210/8297 (ratio 1.0106); SSIM blocks <0.9: 724/30294; [overlay](images/18-ligatures/pdflatex-main-export-p1-overlay.png) (49795 B, ÷2), [heatmap](images/18-ligatures/pdflatex-main-export-p1-heatmap.png) (42114 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.13, 0.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2894, differing 0.010507, SSIM₈ 0.9808 (raw 1.2894, 0.010507, 0.9808)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9693→0.9693 / 2.0608→2.0608; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.86 dy 14.86; `ruffled,` dx -433.44 dy 14.82; `muffin.` dx 40.63 dy 0.37

### 18-ligatures — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1925987, 1923, 1403, 1249, 1019, 1004, 912, 833, 632, 491, 439, 496, 477, 362, 448, 1141]`; ink px ref/ours 8210/8202 (ratio 0.999); SSIM blocks <0.9: 509/30294; [overlay](images/18-ligatures/pdflatex-pipeline-export-p1-overlay.png) (49007 B, ÷2), [heatmap](images/18-ligatures/pdflatex-pipeline-export-p1-heatmap.png) (39756 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.08, -0.07] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7396, differing 0.008381, SSIM₈ 0.9907 (raw 0.7396, 0.008381, 0.9907)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9852→0.9852 / 1.1796→1.1796; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `muffin.` dx 3.04 dy 0.41; `finally` dx 2.13 dy 0.41; `office,` dx -1.21 dy 0.41

### 18-ligatures — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920380, 1602, 1329, 1334, 1326, 1152, 1112, 1125, 1111, 987, 890, 943, 938, 768, 690, 3129]`; ink px ref/ours 6473/8297 (ratio 1.2818); SSIM blocks <0.9: 798/30294; [overlay](images/18-ligatures/pdflatex-lm-de1020c-export-p1-overlay.png) (51525 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-de1020c-export-p1-heatmap.png) (43610 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-18.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3343, not lower; centroid estimate [-5.55, -0.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3148, differing 0.010687, SSIM₈ 0.9787 (raw 1.3148, 0.010687, 0.9787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.1014→2.1014; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `coffin,` dx -21.57 dy 0.73; `official,` dx -20.57 dy 0.73; `offline,` dx -20.28 dy 0.68

### 18-ligatures — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920380, 1602, 1329, 1334, 1326, 1152, 1112, 1125, 1111, 987, 890, 943, 938, 768, 690, 3129]`; ink px ref/ours 6473/8297 (ratio 1.2818); SSIM blocks <0.9: 798/30294; [overlay](images/18-ligatures/pdflatex-lm-main-export-p1-overlay.png) (51231 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-main-export-p1-heatmap.png) (43325 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-18.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3343, not lower; centroid estimate [-5.55, -0.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3148, differing 0.010687, SSIM₈ 0.9787 (raw 1.3148, 0.010687, 0.9787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.1014→2.1014; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `coffin,` dx -21.57 dy 0.73; `official,` dx -20.57 dy 0.73; `offline,` dx -20.28 dy 0.68

### 18-ligatures — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920282, 1603, 1251, 1369, 1265, 1192, 1205, 1152, 1128, 981, 853, 904, 919, 744, 818, 3150]`; ink px ref/ours 6473/8202 (ratio 1.2671); SSIM blocks <0.9: 803/30294; [overlay](images/18-ligatures/pdflatex-lm-pipeline-export-p1-overlay.png) (51169 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-pipeline-export-p1-heatmap.png) (43523 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [11.0, 0.0] pt REJECTED: applying it gives mean|Δ| 1.4142, not lower; centroid estimate [7.5, -1.32] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3286, differing 0.010774, SSIM₈ 0.9782 (raw 1.3286, 0.010774, 0.9782)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9654→0.9654 / 2.121→2.121; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx 445.86 dy -13.78; `ruffled,` dx 433.14 dy -13.78; `muffin.` dx -43.39 dy 0.67

### 18-ligatures — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920422, 1579, 1465, 1271, 1273, 1053, 999, 990, 930, 938, 916, 878, 984, 761, 739, 3618]`; ink px ref/ours 8366/8297 (ratio 0.9918); SSIM blocks <0.9: 758/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3891, not lower; centroid estimate [-12.27, 0.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3498, differing 0.010823, SSIM₈ 0.9797 (raw 1.3498, 0.010823, 0.9797)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9675→0.9675 / 2.1573→2.1573; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.26 dy 14.86; `ruffled,` dx -432.74 dy 14.82; `muffin.` dx 39.24 dy 0.37

### 18-ligatures — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920422, 1579, 1465, 1271, 1273, 1053, 999, 990, 930, 938, 916, 878, 984, 761, 739, 3618]`; ink px ref/ours 8366/8297 (ratio 0.9918); SSIM blocks <0.9: 758/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3891, not lower; centroid estimate [-12.27, 0.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3498, differing 0.010823, SSIM₈ 0.9797 (raw 1.3498, 0.010823, 0.9797)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9675→0.9675 / 2.1573→2.1573; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.26 dy 14.86; `ruffled,` dx -432.74 dy 14.82; `muffin.` dx 39.24 dy 0.37

### 18-ligatures — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924922, 2030, 1531, 1311, 1062, 1028, 973, 797, 753, 715, 731, 525, 436, 386, 392, 1224]`; ink px ref/ours 8366/8202 (ratio 0.9804); SSIM blocks <0.9: 583/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.78, -0.07] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8056, differing 0.008737, SSIM₈ 0.9896 (raw 0.8056, 0.008737, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9835→0.9835 / 1.2852→1.2852; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `muffin.` dx 1.65 dy 0.41; `finally` dx 1.4 dy 0.41; `baffling,` dx 0.9 dy 0.41

### 18-ligatures — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920366, 1631, 1312, 1334, 1353, 1175, 1117, 1063, 1128, 976, 869, 980, 948, 757, 705, 3102]`; ink px ref/ours 6486/8297 (ratio 1.2792); SSIM blocks <0.9: 800/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-18.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3341, not lower; centroid estimate [-5.31, -0.31] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3133, differing 0.010682, SSIM₈ 0.9787 (raw 1.3133, 0.010682, 0.9787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.099→2.099; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `coﬀin,` dx -21.56 dy 0.73; `oﬀicial,` dx -20.56 dy 0.73; `offline,` dx -20.28 dy 0.68

### 18-ligatures — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920366, 1631, 1312, 1334, 1353, 1175, 1117, 1063, 1128, 976, 869, 980, 948, 757, 705, 3102]`; ink px ref/ours 6486/8297 (ratio 1.2792); SSIM blocks <0.9: 800/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-18.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3341, not lower; centroid estimate [-5.31, -0.31] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3133, differing 0.010682, SSIM₈ 0.9787 (raw 1.3133, 0.010682, 0.9787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.099→2.099; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `coﬀin,` dx -21.56 dy 0.73; `oﬀicial,` dx -20.56 dy 0.73; `offline,` dx -20.28 dy 0.68

### 18-ligatures — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): error: Latin Modern face unavailable (lmroman12-regular.otf: not found in 2 search directories: /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm, /usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math); Times metrics substituted, output is not the requested document
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920281, 1594, 1270, 1351, 1293, 1189, 1215, 1128, 1121, 952, 879, 911, 924, 752, 810, 3146]`; ink px ref/ours 6486/8202 (ratio 1.2646); SSIM blocks <0.9: 805/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [11.0, 0.0] pt REJECTED: applying it gives mean|Δ| 1.4138, not lower; centroid estimate [7.74, -1.32] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3281, differing 0.010762, SSIM₈ 0.9782 (raw 1.3281, 0.010762, 0.9782)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9654→0.9654 / 2.1203→2.1203; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx 445.86 dy -13.78; `ruffled,` dx 433.14 dy -13.78; `muﬀin.` dx -43.38 dy 0.67

## Diagnostic thresholds (never acceptance)

Thresholds file: `harness/thresholds.json` (copied here as `thresholds.used.json`). A failure here is an acceptance signal for the narrow case only.

- 01-plain-paragraph/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'dx_abs_mean', 'dy_abs_mean', 'line_start_agreement', 'ssim_8x8_mean']; pass
- 02-wrapping-paragraph/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'line_start_agreement', 'ssim_8x8_mean']; pass
- 03-section-heading/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'dx_abs_mean', 'dy_abs_mean', 'ssim_8x8_mean']; pass
- 04-bold-emph/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'dx_abs_mean', 'dy_abs_mean', 'ssim_8x8_mean']; pass
- 05-unicode/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'dx_abs_mean', 'dy_abs_mean', 'ssim_8x8_mean']; pass
- 06-math-inline/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'dx_abs_mean', 'dy_abs_mean', 'ssim_8x8_mean']; pass
- 07-math-display/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'dx_abs_mean', 'dy_abs_mean', 'ssim_8x8_mean']; pass
- 08-two-page/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'line_start_agreement', 'ssim_8x8_mean']; pass
- 09-mixed-document/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'dx_abs_mean', 'dy_abs_mean', 'ssim_8x8_mean']; pass
- 10-unicode-paragraph/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'ssim_8x8_mean']; **FAIL** diff_mean=3.362 > 3.0; above_threshold_fraction=0.021285 > 0.02
- 11-nested-lists/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'ssim_8x8_mean']; pass
- 12-justified-paragraphs/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'ssim_8x8_mean']; **FAIL** ssim_8x8_mean=0.7415 < 0.9; diff_mean=15.3708 > 3.0; above_threshold_fraction=0.092527 > 0.02
- 13-math-display-rich/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'ssim_8x8_mean']; pass
- 14-math-inline-dense/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'ssim_8x8_mean']; pass
- 15-three-page-sections/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'ssim_8x8_mean']; **FAIL** ssim_8x8_mean=0.5541 < 0.9; diff_mean=26.6994 > 3.0; above_threshold_fraction=0.159865 > 0.02
- 16-heading-page-break/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'ssim_8x8_mean']; **FAIL** ssim_8x8_mean=0.662 < 0.9; diff_mean=20.5977 > 3.0; above_threshold_fraction=0.124 > 0.02
- 17-apostrophes/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'ssim_8x8_mean']; pass
- 18-ligatures/pdflatex/de1020c/export: checked ['above_threshold_fraction', 'diff_mean', 'ssim_8x8_mean']; pass

## Diagnostic regression check vs previous evidence (never acceptance)

- previous evidence: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a8603282c8958a68c/tests/visual-corpus/evidence/20260912T055841Z`; comparable entries: 216
- worse: 
  - 06-math-inline/lualatex/main/export: ssim_8x8_mean 0.9958 -> 0.9934
  - 06-math-inline/lualatex/main/preview: ssim_8x8_mean 0.9958 -> 0.9934
  - 06-math-inline/lualatex-lm/main/export: ssim_8x8_mean 0.9953 -> 0.9931
  - 06-math-inline/lualatex-lm/main/preview: ssim_8x8_mean 0.9953 -> 0.9931
  - 06-math-inline/pdflatex-lm/main/export: ssim_8x8_mean 0.9953 -> 0.9931
  - 06-math-inline/pdflatex-lm/main/preview: ssim_8x8_mean 0.9953 -> 0.9931
  - 06-math-inline/xelatex/main/export: ssim_8x8_mean 0.9958 -> 0.9934
  - 06-math-inline/xelatex/main/preview: ssim_8x8_mean 0.9958 -> 0.9934
  - 06-math-inline/xelatex-lm/main/export: ssim_8x8_mean 0.9953 -> 0.9931
  - 06-math-inline/xelatex-lm/main/preview: ssim_8x8_mean 0.9953 -> 0.9931
  - 08-two-page/lualatex/main/export: diff_mean 25.62 -> 26.2176
  - 08-two-page/lualatex/main/export: above_threshold_fraction 0.154641 -> 0.157423
  - 08-two-page/lualatex/main/preview: diff_mean 25.6206 -> 26.2189
  - 08-two-page/lualatex/main/preview: above_threshold_fraction 0.154663 -> 0.157403
  - 08-two-page/lualatex-lm/main/export: diff_mean 23.4169 -> 23.7967
  - 08-two-page/lualatex-lm/main/preview: diff_mean 23.417 -> 23.7956
  - 08-two-page/pdflatex/main/export: diff_mean 25.618 -> 26.1974
  - 08-two-page/pdflatex/main/export: above_threshold_fraction 0.154426 -> 0.157175
  - 08-two-page/pdflatex/main/preview: diff_mean 25.6175 -> 26.1972
  - 08-two-page/pdflatex/main/preview: above_threshold_fraction 0.15445 -> 0.157148
  - 08-two-page/pdflatex-lm/main/export: diff_mean 23.416 -> 23.7963
  - 08-two-page/pdflatex-lm/main/preview: diff_mean 23.4162 -> 23.7951
  - 08-two-page/xelatex/main/export: diff_mean 25.6193 -> 26.2425
  - 08-two-page/xelatex/main/export: above_threshold_fraction 0.154866 -> 0.157764
  - 08-two-page/xelatex/main/preview: diff_mean 25.6199 -> 26.2441
  - 08-two-page/xelatex/main/preview: above_threshold_fraction 0.154882 -> 0.157748
  - 08-two-page/xelatex-lm/main/export: diff_mean 23.4169 -> 23.7969
  - 08-two-page/xelatex-lm/main/preview: diff_mean 23.417 -> 23.7957
  - 09-mixed-document/lualatex/main/export: ssim_8x8_mean 0.9553 -> 0.9497
  - 09-mixed-document/lualatex/main/preview: ssim_8x8_mean 0.9553 -> 0.9497
  - 09-mixed-document/lualatex-lm/main/export: ssim_8x8_mean 0.9546 -> 0.949
  - 09-mixed-document/lualatex-lm/main/preview: ssim_8x8_mean 0.9546 -> 0.949
  - 09-mixed-document/pdflatex/main/export: ssim_8x8_mean 0.9553 -> 0.9497
  - 09-mixed-document/pdflatex/main/preview: ssim_8x8_mean 0.9553 -> 0.9497
  - 09-mixed-document/pdflatex-lm/main/export: ssim_8x8_mean 0.9532 -> 0.9475
  - 09-mixed-document/pdflatex-lm/main/preview: ssim_8x8_mean 0.9532 -> 0.9475
  - 09-mixed-document/xelatex/main/export: ssim_8x8_mean 0.9553 -> 0.9496
  - 09-mixed-document/xelatex/main/preview: ssim_8x8_mean 0.9553 -> 0.9496
  - 09-mixed-document/xelatex-lm/main/export: ssim_8x8_mean 0.9546 -> 0.949
  - 09-mixed-document/xelatex-lm/main/preview: ssim_8x8_mean 0.9546 -> 0.949

## Limitations and honesty notes

- Reference engines and fonts: pdflatex uses the psnfss `times` package (URW Nimbus Roman clone); xelatex and lualatex use the macOS system `Times New Roman` TrueType via fontspec. Neither is byte-identical to the Times-Roman standard-14 face CoreGraphics substitutes when rasterizing the FlashTeX PDF.
- Compiler build `main` (origin/main) has no math support: math fixtures compile with status `recovered` and the math is rendered as plain text; `de1020c` typesets math with Unicode symbols and U+2500 rule runs.
- The preview-equivalent raster re-implements the app's CoreText draw; it is not a capture of the SwiftUI preview.
- Metrics are for these fixtures, this DPI, these builds and this machine only.
- Exact-equality gates are the only acceptance signals; thresholds, SSIM, registration and regression numbers are diagnostics.
- PDFKit exposes text selections only; it cannot report rule/line geometry from the reference PDFs, so rule presence and position are compared from ink rows of the rasters (≥10pt contiguous dark run), and FlashTeX's own rectangle geometry (`re f`) is listed only for cross-checking.
- The pdflatex reference uses the URW Nimbus Roman clone (`times` package), xelatex/lualatex use the macOS Times New Roman TrueType, and FlashTeX's PDF uses the standard-14 `Times-Roman` name resolved by the rasterizer's CoreGraphics PDF engine (plus an embedded Times New Roman subset for non-WinAnsi glyphs). Glyph outlines therefore differ slightly even where positions agree; the metrics include that font-substitution noise.
- FlashTeX has no justification, hyphenation, kerning or ligatures; line breaks and word x positions diverge progressively along a line. Page-level SSIM over text is dominated by that, not by glyph rendering.
- Missing pages (page-count mismatch) are reported, not silently skipped.
