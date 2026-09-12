# FlashTeX visual corpus: reference-render and raster-diff evidence

Generated 20260912T072838Z on mac-m1max-a by `tests/visual-corpus/harness/run.sh`.

**Scope statement.** These are narrow-case measurements over a small declared corpus. They never claim general pixel perfection, LaTeX compatibility, or parity outside these fixtures, these engines, this font, this page size, this DPI and these builds. The reference engines are test oracles only; FlashTeX never invokes them and remains an original Rust implementation.

**Acceptance vs diagnostics.** The only acceptance signals in this report are the exact-equality gates below (zero pixel difference between FlashTeX's export raster and its preview rasters, and raw PDF byte identity against the pinned profile). Every tolerance, threshold, SSIM, registration shift or regression comparison further down is a diagnostic to explain *why* something differs; none of them ever counts as acceptance.

## Exact-equality gates (acceptance)

| Fixture | Compiler | export = preview-equivalent | export = native preview capture | PDF bytes = pinned | PDF SHA-256 |
|---|---|---|---|---|---|
| 01-plain-paragraph | de1020c | DIFFERENT: 1240/1938816 px, max |Δ| 255 | unavailable | **EQUAL** | `88deda9f24394700…` |
| 01-plain-paragraph | main | DIFFERENT: 1240/1938816 px, max |Δ| 255 | unavailable | DIFFERENT | `88deda9f24394700…` |
| 01-plain-paragraph | pipeline | DIFFERENT: 1385/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `834462c4a861d0b2…` |
| 02-wrapping-paragraph | de1020c | DIFFERENT: 17142/1938816 px, max |Δ| 255 | unavailable | **EQUAL** | `1bc0994be86d3ece…` |
| 02-wrapping-paragraph | main | DIFFERENT: 17142/1938816 px, max |Δ| 255 | unavailable | DIFFERENT | `1bc0994be86d3ece…` |
| 02-wrapping-paragraph | pipeline | DIFFERENT: 16693/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `0a14a5c29583a221…` |
| 03-section-heading | de1020c | DIFFERENT: 2140/1938816 px, max |Δ| 6 | unavailable | **EQUAL** | `7f0f7247434d3132…` |
| 03-section-heading | main | DIFFERENT: 2117/1938816 px, max |Δ| 6 | unavailable | DIFFERENT | `599a00dbcb62e333…` |
| 03-section-heading | pipeline | DIFFERENT: 1858/1938816 px, max |Δ| 6 | unavailable | unpinned (no reference profile entry) | `653601a673db7c53…` |
| 04-bold-emph | de1020c | DIFFERENT: 1261/1938816 px, max |Δ| 4 | unavailable | **EQUAL** | `7a34b0a51a2992f4…` |
| 04-bold-emph | main | DIFFERENT: 1261/1938816 px, max |Δ| 4 | unavailable | DIFFERENT | `7a34b0a51a2992f4…` |
| 04-bold-emph | pipeline | DIFFERENT: 1622/1938816 px, max |Δ| 4 | unavailable | unpinned (no reference profile entry) | `9e1c832d9e02118c…` |
| 05-unicode | de1020c | DIFFERENT: 587/1938816 px, max |Δ| 3 | unavailable | **EQUAL** | `e721656e20b80d47…` |
| 05-unicode | main | DIFFERENT: 587/1938816 px, max |Δ| 3 | unavailable | DIFFERENT | `e721656e20b80d47…` |
| 05-unicode | pipeline | DIFFERENT: 508/1938816 px, max |Δ| 2 | unavailable | unpinned (no reference profile entry) | `9c47f67aa9f7c91a…` |
| 06-math-inline | de1020c | DIFFERENT: 1041/1938816 px, max |Δ| 254 | unavailable | **EQUAL** | `0a1a2a6eebc4b2b1…` |
| 06-math-inline | main | DIFFERENT: 1041/1938816 px, max |Δ| 254 | unavailable | DIFFERENT | `0a1a2a6eebc4b2b1…` |
| 06-math-inline | pipeline | DIFFERENT: 1502/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `441cf7de98417253…` |
| 07-math-display | de1020c | DIFFERENT: 1021/1938816 px, max |Δ| 255 | unavailable | **EQUAL** | `915882632c74f40b…` |
| 07-math-display | main | DIFFERENT: 1021/1938816 px, max |Δ| 255 | unavailable | DIFFERENT | `0f8277b1aac1eab4…` |
| 07-math-display | pipeline | DIFFERENT: 1281/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `ad85a5704037ce0b…` |
| 08-two-page | de1020c | DIFFERENT: 58237/1938816 px, max |Δ| 255 | unavailable | **EQUAL** | `6e8c29a5f28f5429…` |
| 08-two-page | main | DIFFERENT: 58237/1938816 px, max |Δ| 255 | unavailable | DIFFERENT | `6e8c29a5f28f5429…` |
| 08-two-page | pipeline | DIFFERENT: 58512/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `fab2b9688b9ce8b4…` |
| 09-mixed-document | de1020c | DIFFERENT: 4608/1938816 px, max |Δ| 250 | unavailable | **EQUAL** | `eb9ed31b44ff065e…` |
| 09-mixed-document | main | DIFFERENT: 4600/1938816 px, max |Δ| 250 | unavailable | DIFFERENT | `fc8d547cb07b446a…` |
| 09-mixed-document | pipeline | DIFFERENT: 5019/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `8e173133252cf2f5…` |
| 10-unicode-paragraph | de1020c | DIFFERENT: 9269/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `405e9d22c4c242b1…` |
| 10-unicode-paragraph | main | DIFFERENT: 9269/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `405e9d22c4c242b1…` |
| 10-unicode-paragraph | pipeline | DIFFERENT: 8489/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `3c183aef95c9a183…` |
| 11-nested-lists | de1020c | DIFFERENT: 2764/1938816 px, max |Δ| 6 | unavailable | unpinned (no reference profile entry) | `dddb38b9e9050f15…` |
| 11-nested-lists | main | DIFFERENT: 3060/1938816 px, max |Δ| 7 | unavailable | unpinned (no reference profile entry) | `2be630dc95b8a81e…` |
| 11-nested-lists | pipeline | DIFFERENT: 2899/1938816 px, max |Δ| 6 | unavailable | unpinned (no reference profile entry) | `20c2decbaa702819…` |
| 12-justified-paragraphs | de1020c | DIFFERENT: 30049/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `47e5eefae9a4319a…` |
| 12-justified-paragraphs | main | DIFFERENT: 30049/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `47e5eefae9a4319a…` |
| 12-justified-paragraphs | pipeline | DIFFERENT: 27963/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `188eebf8739994c9…` |
| 13-math-display-rich | de1020c | DIFFERENT: 1699/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `80f03c44b40cd1b1…` |
| 13-math-display-rich | main | DIFFERENT: 1699/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `0170cf520ff62e6c…` |
| 13-math-display-rich | pipeline | DIFFERENT: 3433/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `7f4aa8a9500de9b0…` |
| 14-math-inline-dense | de1020c | DIFFERENT: 3142/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `04244b7cb4eedb5c…` |
| 14-math-inline-dense | main | DIFFERENT: 3142/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `04244b7cb4eedb5c…` |
| 14-math-inline-dense | pipeline | DIFFERENT: 7075/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `6b258f9ad199d8cc…` |
| 15-three-page-sections | de1020c | DIFFERENT: 49094/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `9f2923c8d7e3dba9…` |
| 15-three-page-sections | main | DIFFERENT: 49088/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `9b490607f689e55f…` |
| 15-three-page-sections | pipeline | DIFFERENT: 51958/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `6c9fb561639c5572…` |
| 16-heading-page-break | de1020c | DIFFERENT: 58237/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `a1b742b01edb41a0…` |
| 16-heading-page-break | main | DIFFERENT: 58237/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `53ac9e5351991d14…` |
| 16-heading-page-break | pipeline | DIFFERENT: 58512/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `85f60cde6085faa7…` |
| 17-apostrophes | de1020c | DIFFERENT: 1919/1938816 px, max |Δ| 4 | unavailable | unpinned (no reference profile entry) | `aed18719902a6924…` |
| 17-apostrophes | main | DIFFERENT: 1919/1938816 px, max |Δ| 4 | unavailable | unpinned (no reference profile entry) | `aed18719902a6924…` |
| 17-apostrophes | pipeline | DIFFERENT: 2287/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `8a0833c6633d92e9…` |
| 18-ligatures | de1020c | DIFFERENT: 9644/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `cccda2e7ef9bd851…` |
| 18-ligatures | main | DIFFERENT: 9644/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `cccda2e7ef9bd851…` |
| 18-ligatures | pipeline | DIFFERENT: 9883/1938816 px, max |Δ| 255 | unavailable | unpinned (no reference profile entry) | `0e230eae1171cff7…` |

- Reference profile: `reference-profile.json`.
- Classification of native-preview differences: the native capture comes from a screen capture at the display's backing scale, resampled to the raster size, so a DIFFERENT result there is expected to be dominated by resampling and text rasterization (CoreText on screen vs CoreGraphics PDF rendering); it is reported as-is, without normalisation. Preview-equivalent vs export differences isolate the drawing path (CoreText glyph run vs the PDF writer's text operators) from any capture effects.

## Provenance

- suite_branch: `mvo/rev2`
- suite_sha: `c462e254d5146d9eb4b9165a4226c2cc82d6690f`
- input_main_sha: `53fee3012b2902ca05bd31766defa515b3044cec`
- machine: `mac-m1max-a`
- os: `macOS 26.3.1 arm64`
- swift: `Apple Swift version 6.2.4 (swiftlang-6.2.4.1.4 clang-1700.6.4.2)`
- cargo: `cargo 1.99.0-nightly (3efb1f477 2026-07-17)`
- python: `3.12.0`
- pillow: `12.2.0`
- DPI: 144 (every raster: CoreGraphics bitmap, sRGB IEC61966-2.1, 8-bit RGBA, white opaque background, MediaBox mapped to width_pt*144/72 px; text antialiased, font smoothing off, subpixel positioning on)
- Overlay/heatmap PNGs emitted for engines: pdflatex,pdflatex-lm, sides: export,native (metrics are computed for every engine and side; PNGs are downscaled by 2 until ≤90000 B)
- Every overlay/heatmap PNG carries a burned-in footer (and XMP dc:description) with the fixture SHA-256, oracle engine+version+font, compiler and flashtex-pdf SHAs, side, DPI/colour profile and run stamp; `metrics.json` repeats them per entry under `provenance`.
- Registration: global (dx,dy) between reference and candidate estimated by 1-D ink-projection cross-correlation (±60 pt search; scale assumed 1 because both sides are rasterized from equal MediaBoxes at the same DPI — the native capture's resample factor is recorded separately). Tables show raw error, the registration shift, and the rendering error after undoing the shift. Regions: text area (1in margins), header/footer bands, and display-math boxes derived from the reference word boxes.
- Pixel threshold for `above_threshold_fraction`: |Δluma| ≥ 32/255; SSIM: 8×8 blocks, K1=0.01, K2=0.03
- Arithmetic backend: Pillow 12.2.0 (accelerator; identical integer results to the stdlib path)

### Reference engines (oracle only)

| Oracle | Available | Version | Body font | Preamble |
|---|---|---|---|---|
| pdflatex | yes | pdfTeX 3.141592653-2.6-1.40.29 (TeX Live 2026) | URW Nimbus Roman (`times` package, T1 fontenc) | `\documentclass[12pt]{article} \usepackage[T1]{fontenc} \usepackage{times} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| pdflatex-lm | yes | pdfTeX 3.141592653-2.6-1.40.29 (TeX Live 2026) | Latin Modern Roman Type 1 (`lmodern` package, T1 fontenc) — LaTeX's default Computer Modern look | `\documentclass[12pt]{article} \usepackage[T1]{fontenc} \usepackage{lmodern} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| xelatex | yes | XeTeX 3.141592653-2.6-0.999998 (TeX Live 2026) | Times New Roman (`fontspec`, /System/Library/Fonts/Supplemental/Times New Roman.ttf) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{Times New Roman} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| xelatex-lm | yes | XeTeX 3.141592653-2.6-0.999998 (TeX Live 2026) | Latin Modern Roman OpenType (`fontspec`, lmroman12-*.otf from the TeX Live tree by explicit path; bold-italic uses lmroman10-bolditalic, the only LM bold-italic face) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{lmroman12-regular.otf}[Path=/usr/local/texlive/2026/texmf-dist/fonts/opentype/public/lm/,BoldFont=lmroman12-bold.otf,ItalicFont=lmroman12-italic.otf,BoldItalicFont=lmroman10-bolditalic.otf] \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| lualatex | yes | This is LuaHBTeX, Version 1.24.0 (TeX Live 2026) | Times New Roman (`fontspec`, /System/Library/Fonts/Supplemental/Times New Roman.ttf) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{Times New Roman} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| lualatex-lm | yes | This is LuaHBTeX, Version 1.24.0 (TeX Live 2026) | Latin Modern Roman OpenType (`fontspec`, lmroman12-*.otf from the TeX Live tree by explicit path; bold-italic uses lmroman10-bolditalic, the only LM bold-italic face) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{lmroman12-regular.otf}[Path=/usr/local/texlive/2026/texmf-dist/fonts/opentype/public/lm/,BoldFont=lmroman12-bold.otf,ItalicFont=lmroman12-italic.otf,BoldItalicFont=lmroman10-bolditalic.otf] \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |

### Reference availability this run

- rendered fresh by an installed engine: 108 fixture/oracle pairs
- live renders vs previously recorded references (20260912T064032Z, BasicTeX (TeX Live 2026), same engine versions): 108 pairs, rasters pixel-identical at 144 DPI for 108 of them (max differing px 0); PDF bytes identical for 0 (differences are CreationDate/ModDate, trailer /ID and compressed-stream bytes only); details in `reference-vs-recorded.json`

The `-lm` oracles are the intended primary apples-to-apples target once a Latin-Modern-metrics FlashTeX pipeline exists; the Times oracles match the current compiler's Times metrics. Both are reported for every fixture.

TeX distribution this run: **MacTeX / TeX Live full (/usr/local/texlive/2026)**, texbin → `/usr/local/texlive/2026/bin/universal-darwin`, tlmgr revision 78301 (2026-03-07 18:41:28 +0100).

Engine flags: `-interaction=batchmode -halt-on-error -file-line-error`. Page size: US letter 612×792 pt for every producer (checked per page from the MediaBox). LaTeX package versions: see `provenance.json` → `packages`.

### FlashTeX builds under test

- compiler `main`: `origin/main` @ `5f9f4ecf09f24c7da2dc062af01374b856b208b9` (crates/compiler) — Move existing root product slot to bounded AI diagnostic context and align standby setup paths
- compiler `de1020c`: `de1020c` @ `de1020cd0be7cede11be2691e00e7f5b15cb2224` (crates/compiler) — compiler: add math, real font metrics and PDF output
- compiler `pipeline`: `origin/agent/mac-render-pipeline/unified` @ `c000dadcb637293ac1963decc2387b323b0cefe4` (crates/render-pipeline) — render-pipeline: secnumdepth option and \setcounter{secnumdepth} parsing
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
| 01-plain-paragraph | lualatex | pipeline | 1/1 | ok | 0.3845 | 0.9943 | (0.5,0) weak | 0.3810 | 0.9944 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 12.23 | 0.41 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | de1020c | 1/1 | ok | 0.3662 | 0.9941 | (0,0) | 0.3662 | 0.9941 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.30 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | main | 1/1 | ok | 0.3662 | 0.9941 | (0,0) | 0.3662 | 0.9941 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.30 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | pipeline | 1/1 | ok | 0.2368 | 0.9967 | (0,0) | 0.2368 | 0.9967 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.04 | 0.36 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.1873 | 0.9975 | (0,0) | 0.1873 | 0.9975 | 255 | 0.0024 | 0.0014 | 13/13/13 | yes | 0.43 | 0.46 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.1873 | 0.9975 | (0,0) | 0.1873 | 0.9975 | 255 | 0.0024 | 0.0014 | 13/13/13 | yes | 0.43 | 0.46 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-main-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | pipeline | 1/1 | ok | 0.3946 | 0.9944 | (1,0) weak | 0.3788 | 0.9947 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 12.75 | 0.41 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-pipeline-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 0.3660 | 0.9942 | (0,0) | 0.3660 | 0.9942 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | main | 1/1 | ok | 0.3660 | 0.9942 | (0,0) | 0.3660 | 0.9942 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 0.2380 | 0.9966 | (0,0) | 0.2380 | 0.9966 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.03 | 0.67 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.1607 | 0.9980 | (0,0) | 0.1607 | 0.9980 | 255 | 0.0023 | 0.0014 | 13/13/13 | yes | 0.30 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.1607 | 0.9980 | (0,0) | 0.1607 | 0.9980 | 255 | 0.0023 | 0.0014 | 13/13/13 | yes | 0.30 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex | pipeline | 1/1 | ok | 0.3847 | 0.9943 | (0.5,0) weak | 0.3811 | 0.9944 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 12.22 | 0.41 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | de1020c | 1/1 | ok | 0.3665 | 0.9941 | (0,0) | 0.3665 | 0.9941 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | main | 1/1 | ok | 0.3665 | 0.9941 | (0,0) | 0.3665 | 0.9941 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | pipeline | 1/1 | ok | 0.2367 | 0.9967 | (0,0) | 0.2367 | 0.9967 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.03 | 0.67 | 1.0000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 8.3842 | 0.8656 | (-3,0) weak | 8.3713 | 0.8657 | 255 | 0.0613 | 0.0509 | 210/210/210 | yes | 116.20 | 5.12 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 8.3842 | 0.8656 | (-3,0) weak | 8.3713 | 0.8657 | 255 | 0.0613 | 0.0509 | 210/210/210 | yes | 116.20 | 5.12 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | pipeline | 1/1 | ok | 7.1751 | 0.8901 | (0,0) | 7.1751 | 0.8901 | 255 | 0.0554 | 0.0449 | 210/210/210 | yes | 169.48 | 4.26 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | de1020c | 1/1 | ok | 7.7868 | 0.8620 | (-0.5,-14.5) weak | 7.7691 | 0.8602 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.57 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | main | 1/1 | ok | 7.7868 | 0.8620 | (-0.5,-14.5) weak | 7.7691 | 0.8602 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.57 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | pipeline | 1/1 | ok | 4.5384 | 0.9368 | (0,0) | 4.5384 | 0.9368 | 255 | 0.0447 | 0.0336 | 210/210/210 | yes | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 8.3726 | 0.8663 | (0,0) | 8.3726 | 0.8663 | 255 | 0.0611 | 0.0509 | 210/210/210 | yes | 116.24 | 5.12 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 8.3726 | 0.8663 | (0,0) | 8.3726 | 0.8663 | 255 | 0.0611 | 0.0509 | 210/210/210 | yes | 116.24 | 5.12 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-main-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | pipeline | 1/1 | ok | 7.1603 | 0.8910 | (1,0) weak | 7.0822 | 0.8919 | 255 | 0.0553 | 0.0447 | 210/210/210 | yes | 169.52 | 4.26 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-pipeline-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 7.7862 | 0.8620 | (0,0) | 7.7862 | 0.8620 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | main | 1/1 | ok | 7.7862 | 0.8620 | (0,0) | 7.7862 | 0.8620 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 4.5467 | 0.9367 | (0,0) | 4.5467 | 0.9367 | 255 | 0.0447 | 0.0337 | 210/210/210 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 8.3710 | 0.8656 | (0,0) | 8.3710 | 0.8656 | 255 | 0.0613 | 0.0509 | 210/210/210 | yes | 119.39 | 5.19 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 8.3710 | 0.8656 | (0,0) | 8.3710 | 0.8656 | 255 | 0.0613 | 0.0509 | 210/210/210 | yes | 119.39 | 5.19 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex | pipeline | 1/1 | ok | 7.1985 | 0.8902 | (0.5,0) weak | 7.1662 | 0.8904 | 255 | 0.0556 | 0.0451 | 210/210/210 | yes | 170.98 | 4.33 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | de1020c | 1/1 | ok | 7.7866 | 0.8620 | (-0.5,-14.5) weak | 7.7691 | 0.8602 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | main | 1/1 | ok | 7.7866 | 0.8620 | (-0.5,-14.5) weak | 7.7691 | 0.8602 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | pipeline | 1/1 | ok | 4.5391 | 0.9368 | (0,0) | 4.5391 | 0.9368 | 255 | 0.0446 | 0.0336 | 210/210/210 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.3394 | 0.9793 | (0,52.5) moderate | 1.1861 | 0.9829 | 255 | 0.0085 | 0.0073 | 15/15/15 | yes | 0.25 | 21.02 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.3597 | 0.9789 | (0.5,52.5) moderate | 1.2078 | 0.9824 | 255 | 0.0086 | 0.0074 | 15/17/15 | no | 2.70 | 21.02 | 0.8667 | 0/0 | - |
| 03-section-heading | lualatex | pipeline | 1/1 | ok | 0.9609 | 0.9877 | (0,-0.5) weak | 0.9214 | 0.9881 | 255 | 0.0067 | 0.0057 | 15/17/15 | no | 11.43 | 0.20 | 0.8667 | 0/0 | - |
| 03-section-heading | lualatex-lm | de1020c | 1/1 | ok | 1.2336 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | 15/15/15 | yes | 5.46 | 21.19 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex-lm | main | 1/1 | ok | 1.2425 | 0.9769 | (0,53.5) moderate | 1.1122 | 0.9812 | 255 | 0.0082 | 0.0070 | 15/17/15 | no | 6.72 | 21.19 | 0.8667 | 0/0 | - |
| 03-section-heading | lualatex-lm | pipeline | 1/1 | ok | 0.9262 | 0.9869 | (0,0.5) weak | 0.9033 | 0.9876 | 255 | 0.0066 | 0.0055 | 15/17/15 | no | 5.82 | 0.50 | 0.8667 | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.3496 | 0.9793 | (0.5,52.5) moderate | 1.2021 | 0.9827 | 255 | 0.0085 | 0.0072 | 15/15/15 | yes | 0.33 | 20.87 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-de1020c-export-p1-overlay.png) |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.3719 | 0.9789 | (0.5,52.5) moderate | 1.2209 | 0.9823 | 255 | 0.0086 | 0.0074 | 15/17/15 | no | 2.78 | 20.87 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-main-export-p1-overlay.png) |
| 03-section-heading | pdflatex | pipeline | 1/1 | ok | 0.9952 | 0.9873 | (0.5,-0.5) weak | 0.9729 | 0.9875 | 255 | 0.0068 | 0.0057 | 15/17/15 | no | 11.51 | 0.35 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-pipeline-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | de1020c | 1/1 | ok | 1.2335 | 0.9771 | (0,54) moderate | 1.0954 | 0.9811 | 255 | 0.0082 | 0.0069 | 15/15/15 | yes | 5.46 | 22.37 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | main | 1/1 | ok | 1.2428 | 0.9769 | (0,54) moderate | 1.1201 | 0.9807 | 255 | 0.0082 | 0.0070 | 15/17/15 | no | 6.73 | 22.37 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-main-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | pipeline | 1/1 | ok | 0.9297 | 0.9868 | (0,1) weak | 0.8922 | 0.9875 | 255 | 0.0066 | 0.0055 | 15/17/15 | no | 5.82 | 1.34 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.3370 | 0.9793 | (0,52.5) moderate | 1.1868 | 0.9829 | 255 | 0.0085 | 0.0073 | 15/15/15 | yes | 0.24 | 21.02 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.3600 | 0.9789 | (0.5,52.5) moderate | 1.2088 | 0.9824 | 255 | 0.0086 | 0.0074 | 15/17/15 | no | 2.70 | 21.02 | 0.8667 | 0/0 | - |
| 03-section-heading | xelatex | pipeline | 1/1 | ok | 0.9661 | 0.9877 | (0,-0.5) weak | 0.9258 | 0.9880 | 255 | 0.0067 | 0.0057 | 15/17/15 | no | 11.43 | 0.20 | 0.8667 | 0/0 | - |
| 03-section-heading | xelatex-lm | de1020c | 1/1 | ok | 1.2337 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | 15/15/15 | yes | 5.46 | 22.35 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex-lm | main | 1/1 | ok | 1.2424 | 0.9769 | (0,53.5) moderate | 1.1122 | 0.9812 | 255 | 0.0082 | 0.0069 | 15/17/15 | no | 6.72 | 22.35 | 0.8667 | 0/0 | - |
| 03-section-heading | xelatex-lm | pipeline | 1/1 | ok | 0.9262 | 0.9869 | (0,0.5) weak | 0.9033 | 0.9876 | 255 | 0.0066 | 0.0055 | 15/17/15 | no | 5.82 | 1.31 | 0.8667 | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.3808 | 0.9945 | (0,0) | 0.3808 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 3.77 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.3808 | 0.9945 | (0,0) | 0.3808 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 3.77 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | pipeline | 1/1 | ok | 0.4278 | 0.9938 | (0,0) | 0.4278 | 0.9938 | 255 | 0.0033 | 0.0026 | 10/11/9 | no | 18.67 | 0.41 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | de1020c | 1/1 | ok | 0.4244 | 0.9933 | (-49,0) weak | 0.4148 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.02 | 0.63 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | main | 1/1 | ok | 0.4244 | 0.9933 | (-49,0) weak | 0.4148 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.02 | 0.63 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | pipeline | 1/1 | ok | 0.3472 | 0.9950 | (-0.5,0) weak | 0.3305 | 0.9952 | 255 | 0.0030 | 0.0023 | 10/11/9 | no | 0.53 | 0.68 | 1.0000 | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.3836 | 0.9944 | (-0.5,0) weak | 0.3764 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 4.04 | 0.46 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-de1020c-export-p1-overlay.png) |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.3836 | 0.9944 | (-0.5,0) weak | 0.3764 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 4.04 | 0.46 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-main-export-p1-overlay.png) |
| 04-bold-emph | pdflatex | pipeline | 1/1 | ok | 0.4169 | 0.9939 | (0,0) | 0.4169 | 0.9939 | 255 | 0.0033 | 0.0026 | 10/11/9 | no | 18.91 | 0.41 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-pipeline-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | de1020c | 1/1 | ok | 0.4246 | 0.9933 | (-49,0) weak | 0.4162 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.08 | 0.73 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | main | 1/1 | ok | 0.4246 | 0.9933 | (-49,0) weak | 0.4162 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.08 | 0.73 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-main-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | pipeline | 1/1 | ok | 0.3564 | 0.9948 | (-0.5,0) moderate | 0.3296 | 0.9953 | 255 | 0.0030 | 0.0024 | 10/11/9 | no | 0.58 | 0.67 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.3848 | 0.9944 | (-0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0031 | 0.0025 | 10/12/8 | no | 3.81 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.3848 | 0.9944 | (-0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0031 | 0.0025 | 10/12/8 | no | 3.81 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex | pipeline | 1/1 | ok | 0.4274 | 0.9938 | (0,0) | 0.4274 | 0.9938 | 255 | 0.0033 | 0.0026 | 10/11/9 | no | 18.70 | 0.41 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | de1020c | 1/1 | ok | 0.4245 | 0.9933 | (-49,0) weak | 0.4138 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 15.90 | 0.73 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | main | 1/1 | ok | 0.4245 | 0.9933 | (-49,0) weak | 0.4138 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 15.90 | 0.73 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | pipeline | 1/1 | ok | 0.3307 | 0.9952 | (0,0) | 0.3307 | 0.9952 | 255 | 0.0029 | 0.0023 | 10/11/9 | no | 0.42 | 0.67 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.2971 | 0.9955 | (0,0) | 0.2971 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.2971 | 0.9955 | (0,0) | 0.2971 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex | pipeline | 1/1 | ok | 0.3579 | 0.9948 | (11.5,0) moderate | 0.3266 | 0.9952 | 255 | 0.0029 | 0.0022 | 11/11/11 | yes | 7.39 | 0.41 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex-lm | de1020c | 1/1 | ok | 0.3619 | 0.9938 | (2.5,0) weak | 0.3494 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.30 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex-lm | main | 1/1 | ok | 0.3619 | 0.9938 | (2.5,0) weak | 0.3494 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.30 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex-lm | pipeline | 1/1 | ok | 0.2635 | 0.9960 | (-0.5,0) moderate | 0.2087 | 0.9969 | 255 | 0.0024 | 0.0019 | 11/11/11 | yes | 0.02 | 0.36 | 1.0000 | 0/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.3127 | 0.9955 | (0,0) | 0.3127 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.88 | 0.46 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-de1020c-export-p1-overlay.png) |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.3127 | 0.9955 | (0,0) | 0.3127 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.88 | 0.46 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-main-export-p1-overlay.png) |
| 05-unicode | pdflatex | pipeline | 1/1 | ok | 0.3571 | 0.9950 | (11.5,0) moderate | 0.3059 | 0.9957 | 255 | 0.0029 | 0.0022 | 11/11/11 | yes | 7.59 | 0.41 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-pipeline-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | de1020c | 1/1 | ok | 0.3617 | 0.9938 | (2.5,0) weak | 0.3496 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | main | 1/1 | ok | 0.3617 | 0.9938 | (2.5,0) weak | 0.3496 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-main-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | pipeline | 1/1 | ok | 0.2654 | 0.9960 | (-0.5,0) moderate | 0.2082 | 0.9969 | 255 | 0.0024 | 0.0019 | 11/11/11 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.2963 | 0.9955 | (0,0) | 0.2963 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.2963 | 0.9955 | (0,0) | 0.2963 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex | pipeline | 1/1 | ok | 0.3577 | 0.9948 | (11.5,0) moderate | 0.3296 | 0.9951 | 255 | 0.0029 | 0.0022 | 11/11/11 | yes | 7.36 | 0.41 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex-lm | de1020c | 1/1 | ok | 0.3620 | 0.9938 | (2.5,0) weak | 0.3493 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | - |
| 05-unicode | xelatex-lm | main | 1/1 | ok | 0.3620 | 0.9938 | (2.5,0) weak | 0.3493 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | - |
| 05-unicode | xelatex-lm | pipeline | 1/1 | ok | 0.2637 | 0.9960 | (-0.5,0) moderate | 0.2088 | 0.9969 | 255 | 0.0024 | 0.0019 | 11/11/11 | yes | 0.02 | 0.67 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.3832 | 0.9934 | (0,3.5) | 0.2554 | 0.9959 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | ok | 0.3832 | 0.9934 | (0,3.5) | 0.2554 | 0.9959 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | pipeline | 1/1 | ok | 0.2941 | 0.9948 | (16.5,0) weak | 0.2801 | 0.9952 | 255 | 0.0024 | 0.0019 | 15/17/10 | no | 8.76 | 1.56 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | de1020c | 1/1 | ok | 0.3682 | 0.9931 | (-32.5,3.5) moderate | 0.3134 | 0.9945 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 12.25 | 2.20 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | main | 1/1 | ok | 0.3682 | 0.9931 | (-32.5,3.5) moderate | 0.3134 | 0.9945 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 12.25 | 2.20 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | pipeline | 1/1 | ok | 0.2944 | 0.9949 | (-1,0) moderate | 0.2401 | 0.9960 | 255 | 0.0024 | 0.0019 | 15/17/10 | no | 0.99 | 1.55 | 1.0000 | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.3821 | 0.9936 | (0,3.5) | 0.2595 | 0.9960 | 255 | 0.0029 | 0.0023 | 13/14/10 | no | 3.89 | 2.69 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-de1020c-export-p1-overlay.png) |
| 06-math-inline | pdflatex | main | 1/1 | ok | 0.3821 | 0.9936 | (0,3.5) | 0.2595 | 0.9960 | 255 | 0.0029 | 0.0023 | 13/14/10 | no | 3.89 | 2.69 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-main-export-p1-overlay.png) |
| 06-math-inline | pdflatex | pipeline | 1/1 | ok | 0.2818 | 0.9951 | (16.5,0) weak | 0.2776 | 0.9954 | 255 | 0.0024 | 0.0018 | 13/17/8 | no | 9.13 | 1.56 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-pipeline-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | de1020c | 1/1 | ok | 0.3680 | 0.9931 | (9.5,3.5) moderate | 0.3234 | 0.9943 | 255 | 0.0029 | 0.0023 | 14/14/12 | no | 13.02 | 2.53 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | main | 1/1 | ok | 0.3680 | 0.9931 | (9.5,3.5) moderate | 0.3234 | 0.9943 | 255 | 0.0029 | 0.0023 | 14/14/12 | no | 13.02 | 2.53 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-main-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | pipeline | 1/1 | ok | 0.2947 | 0.9949 | (-1,0) moderate | 0.2416 | 0.9959 | 255 | 0.0024 | 0.0019 | 14/17/9 | no | 0.96 | 1.74 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.3831 | 0.9934 | (0,3.5) | 0.2548 | 0.9959 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | ok | 0.3831 | 0.9934 | (0,3.5) | 0.2548 | 0.9959 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex | pipeline | 1/1 | ok | 0.2947 | 0.9948 | (16.5,0) weak | 0.2809 | 0.9952 | 255 | 0.0024 | 0.0019 | 15/17/10 | no | 8.76 | 1.56 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | de1020c | 1/1 | ok | 0.3682 | 0.9931 | (9.5,3.5) moderate | 0.3233 | 0.9943 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 12.25 | 2.44 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | main | 1/1 | ok | 0.3682 | 0.9931 | (9.5,3.5) moderate | 0.3233 | 0.9943 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 12.25 | 2.44 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | pipeline | 1/1 | ok | 0.2944 | 0.9949 | (-1,0) moderate | 0.2401 | 0.9960 | 255 | 0.0024 | 0.0019 | 15/17/10 | no | 0.99 | 1.61 | 1.0000 | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.2595 | 0.9950 | (0,0) | 0.2595 | 0.9950 | 255 | 0.0026 | 0.0019 | 13/10/7 | no | 0.60 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | ok | 0.2723 | 0.9946 | (0,0) | 0.2723 | 0.9946 | 255 | 0.0027 | 0.0020 | 13/11/7 | no | 0.60 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex | pipeline | 1/1 | ok | 0.3297 | 0.9939 | (0,0) | 0.3297 | 0.9939 | 255 | 0.0028 | 0.0022 | 13/18/9 | no | 3.18 | 2.40 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex-lm | de1020c | 1/1 | ok | 0.3495 | 0.9931 | (0,0) | 0.3495 | 0.9931 | 255 | 0.0029 | 0.0024 | 13/10/7 | no | 2.12 | 0.92 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex-lm | main | 1/1 | ok | 0.3623 | 0.9927 | (0,0) | 0.3623 | 0.9927 | 255 | 0.0030 | 0.0024 | 13/11/7 | no | 2.12 | 0.92 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex-lm | pipeline | 1/1 | ok | 0.3135 | 0.9941 | (-0.5,0) weak | 0.3044 | 0.9942 | 255 | 0.0027 | 0.0022 | 13/18/9 | no | 1.86 | 2.25 | 1.0000 | 3/1 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.2609 | 0.9949 | (0,0) | 0.2609 | 0.9949 | 255 | 0.0026 | 0.0019 | 13/10/7 | no | 0.59 | 1.41 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-de1020c-export-p1-overlay.png) |
| 07-math-display | pdflatex | main | 1/1 | ok | 0.2737 | 0.9945 | (0,0) | 0.2737 | 0.9945 | 255 | 0.0027 | 0.0020 | 13/11/7 | no | 0.59 | 1.41 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-main-export-p1-overlay.png) |
| 07-math-display | pdflatex | pipeline | 1/1 | ok | 0.3245 | 0.9940 | (0,0) | 0.3245 | 0.9940 | 255 | 0.0028 | 0.0021 | 13/18/9 | no | 3.18 | 2.46 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-pipeline-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | de1020c | 1/1 | ok | 0.3528 | 0.9931 | (0,0) | 0.3528 | 0.9931 | 255 | 0.0029 | 0.0024 | 13/10/7 | no | 2.13 | 1.73 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | main | 1/1 | ok | 0.3656 | 0.9927 | (0,0) | 0.3656 | 0.9927 | 255 | 0.0030 | 0.0025 | 13/11/7 | no | 2.13 | 1.73 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-lm-main-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | pipeline | 1/1 | ok | 0.3130 | 0.9941 | (-0.5,0) weak | 0.3030 | 0.9942 | 255 | 0.0028 | 0.0022 | 13/18/9 | no | 1.86 | 2.77 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.2589 | 0.9950 | (0,0) | 0.2589 | 0.9950 | 255 | 0.0026 | 0.0019 | 14/10/7 | no | 0.59 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | ok | 0.2717 | 0.9946 | (0,0) | 0.2717 | 0.9946 | 255 | 0.0027 | 0.0020 | 14/11/7 | no | 0.59 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex | pipeline | 1/1 | ok | 0.3294 | 0.9939 | (0,0) | 0.3294 | 0.9939 | 255 | 0.0028 | 0.0022 | 14/18/9 | no | 3.18 | 2.40 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex-lm | de1020c | 1/1 | ok | 0.3495 | 0.9931 | (0,0) | 0.3495 | 0.9931 | 255 | 0.0029 | 0.0024 | 14/10/7 | no | 2.12 | 1.54 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex-lm | main | 1/1 | ok | 0.3623 | 0.9927 | (0,0) | 0.3623 | 0.9927 | 255 | 0.0030 | 0.0024 | 14/11/7 | no | 2.12 | 1.54 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex-lm | pipeline | 1/1 | ok | 0.3136 | 0.9941 | (-0.5,0) weak | 0.3044 | 0.9942 | 255 | 0.0027 | 0.0022 | 14/18/9 | no | 1.86 | 2.59 | 1.0000 | 3/1 | - |
| 08-two-page | lualatex | de1020c | 3/3 | ok | 26.2176 | 0.5657 | (0,0); (-3,6) moderate; (-3,34.5) moderate | 24.6323 | 0.5959 | 255 | 0.1880 | 0.1574 | 1800/1800/1800 | yes | 164.00 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | lualatex | main | 3/3 | ok | 26.2176 | 0.5657 | (0,0); (-3,6) moderate; (-3,34.5) moderate | 24.6323 | 0.5959 | 255 | 0.1880 | 0.1574 | 1800/1800/1800 | yes | 164.00 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | lualatex | pipeline | 3/3 | ok | 21.1557 | 0.6700 | (0,0); (0,0); (1,0) weak | 21.1019 | 0.6710 | 255 | 0.1619 | 0.1322 | 1800/1800/1800 | yes | 163.81 | 67.95 | 0.8960 | 0/0 | - |
| 08-two-page | lualatex-lm | de1020c | 3/3 | ok | 23.7967 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7513 | 0.5869 | 255 | 0.1801 | 0.1495 | 1800/1800/1800 | yes | 142.51 | 30.71 | 0.8878 | 0/0 | - |
| 08-two-page | lualatex-lm | main | 3/3 | ok | 23.7967 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7513 | 0.5869 | 255 | 0.1801 | 0.1495 | 1800/1800/1800 | yes | 142.51 | 30.71 | 0.8878 | 0/0 | - |
| 08-two-page | lualatex-lm | pipeline | 3/3 | ok | 12.9946 | 0.8187 | (0,0); (0,0); (0,0) | 12.9946 | 0.8187 | 255 | 0.1280 | 0.0963 | 1800/1800/1800 | yes | 0.00 | 0.36 | 1.0000 | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/3 | ok | 26.1974 | 0.5667 | (0,-14.5) weak; (0,6) moderate; (0,34.5) moderate | 24.5504 | 0.5980 | 255 | 0.1875 | 0.1572 | 1800/1800/1800 | yes | 143.13 | 92.17 | 0.8952 | 0/0 | [p1](images/08-two-page/pdflatex-de1020c-export-p1-overlay.png) |
| 08-two-page | pdflatex | main | 3/3 | ok | 26.1974 | 0.5667 | (0,-14.5) weak; (0,6) moderate; (0,34.5) moderate | 24.5504 | 0.5980 | 255 | 0.1875 | 0.1572 | 1800/1800/1800 | yes | 143.13 | 92.17 | 0.8952 | 0/0 | [p1](images/08-two-page/pdflatex-main-export-p1-overlay.png) |
| 08-two-page | pdflatex | pipeline | 3/3 | ok | 21.1720 | 0.6711 | (-1.5,0) weak; (-1.5,0) weak; (-1.5,0) weak | 21.0316 | 0.6731 | 255 | 0.1614 | 0.1321 | 1800/1800/1800 | yes | 179.69 | 67.14 | 0.8961 | 0/0 | [p1](images/08-two-page/pdflatex-pipeline-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | de1020c | 3/3 | ok | 23.7963 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7519 | 0.5868 | 255 | 0.1801 | 0.1496 | 1800/1800/1800 | yes | 142.52 | 31.52 | 0.8878 | 0/0 | [p1](images/08-two-page/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | main | 3/3 | ok | 23.7963 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7519 | 0.5868 | 255 | 0.1801 | 0.1496 | 1800/1800/1800 | yes | 142.52 | 31.52 | 0.8878 | 0/0 | [p1](images/08-two-page/pdflatex-lm-main-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | pipeline | 3/3 | ok | 13.0228 | 0.8182 | (0,0); (0,0); (0,0) | 13.0228 | 0.8182 | 255 | 0.1281 | 0.0965 | 1800/1800/1800 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | [p1](images/08-two-page/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 08-two-page | xelatex | de1020c | 3/3 | ok | 26.2425 | 0.5656 | (-3,-14.5) weak; (-3,6) moderate; (-3,34.5) moderate | 24.6595 | 0.5953 | 255 | 0.1881 | 0.1578 | 1800/1800/1800 | yes | 164.01 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | xelatex | main | 3/3 | ok | 26.2425 | 0.5656 | (-3,-14.5) weak; (-3,6) moderate; (-3,34.5) moderate | 24.6595 | 0.5953 | 255 | 0.1881 | 0.1578 | 1800/1800/1800 | yes | 164.01 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | xelatex | pipeline | 3/3 | ok | 21.1736 | 0.6696 | (1.5,0) weak; (0,0); (1,0) weak | 21.1122 | 0.6705 | 255 | 0.1620 | 0.1325 | 1800/1800/1800 | yes | 163.83 | 67.95 | 0.8960 | 0/0 | - |
| 08-two-page | xelatex-lm | de1020c | 3/3 | ok | 23.7969 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7509 | 0.5869 | 255 | 0.1801 | 0.1495 | 1800/1800/1800 | yes | 142.51 | 31.52 | 0.8878 | 0/0 | - |
| 08-two-page | xelatex-lm | main | 3/3 | ok | 23.7969 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7509 | 0.5869 | 255 | 0.1801 | 0.1495 | 1800/1800/1800 | yes | 142.51 | 31.52 | 0.8878 | 0/0 | - |
| 08-two-page | xelatex-lm | pipeline | 3/3 | ok | 12.9979 | 0.8186 | (0,0); (0,0); (0,0) | 12.9979 | 0.8186 | 255 | 0.1280 | 0.0963 | 1800/1800/1800 | yes | 0.00 | 0.67 | 1.0000 | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.4844 | 0.9504 | (0,38.5) | 1.7912 | 0.9702 | 255 | 0.0174 | 0.0146 | 54/54/45 | no | 2.41 | 31.76 | 0.9778 | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | ok | 2.5141 | 0.9497 | (0,38.5) | 1.8136 | 0.9695 | 255 | 0.0176 | 0.0148 | 54/56/43 | no | 2.90 | 32.26 | 0.9767 | 0/0 | - |
| 09-mixed-document | lualatex | pipeline | 1/1 | ok | 2.1735 | 0.9628 | (1,-17.5) weak | 2.1034 | 0.9644 | 255 | 0.0156 | 0.0131 | 54/59/43 | no | 39.69 | 12.91 | 0.9070 | 0/0 | - |
| 09-mixed-document | lualatex-lm | de1020c | 1/1 | ok | 2.2538 | 0.9494 | (-0.5,39) moderate | 1.9017 | 0.9641 | 255 | 0.0169 | 0.0140 | 54/54/45 | no | 30.16 | 31.13 | 0.9333 | 0/0 | - |
| 09-mixed-document | lualatex-lm | main | 1/1 | ok | 2.2691 | 0.9490 | (-0.5,39) moderate | 1.9241 | 0.9634 | 255 | 0.0170 | 0.0141 | 54/56/43 | no | 31.94 | 31.57 | 0.9302 | 0/0 | - |
| 09-mixed-document | lualatex-lm | pipeline | 1/1 | ok | 1.8730 | 0.9647 | (0,-17) moderate | 1.5892 | 0.9706 | 255 | 0.0147 | 0.0121 | 54/59/43 | no | 9.21 | 13.55 | 0.9535 | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.4853 | 0.9504 | (-0.5,38.5) moderate | 1.8886 | 0.9685 | 255 | 0.0174 | 0.0146 | 54/54/45 | no | 2.48 | 31.80 | 0.9778 | 0/0 | [p1](images/09-mixed-document/pdflatex-de1020c-export-p1-overlay.png) |
| 09-mixed-document | pdflatex | main | 1/1 | ok | 2.5141 | 0.9497 | (-0.5,38.5) moderate | 1.9110 | 0.9679 | 255 | 0.0175 | 0.0148 | 54/56/43 | no | 2.97 | 32.30 | 0.9767 | 0/0 | [p1](images/09-mixed-document/pdflatex-main-export-p1-overlay.png) |
| 09-mixed-document | pdflatex | pipeline | 1/1 | ok | 2.1784 | 0.9626 | (1,-17) weak | 2.0728 | 0.9646 | 255 | 0.0156 | 0.0131 | 54/59/43 | no | 39.81 | 12.87 | 0.9070 | 0/0 | [p1](images/09-mixed-document/pdflatex-pipeline-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | de1020c | 1/1 | ok | 2.2522 | 0.9480 | (-0.5,39.5) moderate | 1.9012 | 0.9623 | 255 | 0.0167 | 0.0139 | 54/54/45 | no | 30.17 | 32.47 | 0.9333 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | main | 1/1 | ok | 2.2675 | 0.9475 | (-0.5,39.5) moderate | 1.9236 | 0.9616 | 255 | 0.0169 | 0.0140 | 54/56/43 | no | 31.95 | 32.96 | 0.9302 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-main-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | pipeline | 1/1 | ok | 1.8593 | 0.9634 | (0,-16.5) moderate | 1.6085 | 0.9701 | 255 | 0.0144 | 0.0119 | 54/59/43 | no | 9.20 | 12.53 | 0.9535 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.4838 | 0.9503 | (-0.5,38.5) | 1.8537 | 0.9690 | 255 | 0.0174 | 0.0146 | 54/54/45 | no | 2.45 | 31.76 | 0.9778 | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | ok | 2.5135 | 0.9496 | (-0.5,38.5) | 1.8761 | 0.9684 | 255 | 0.0176 | 0.0148 | 54/56/43 | no | 2.94 | 32.26 | 0.9767 | 0/0 | - |
| 09-mixed-document | xelatex | pipeline | 1/1 | ok | 2.1695 | 0.9628 | (1,-17.5) weak | 2.0790 | 0.9647 | 255 | 0.0156 | 0.0131 | 54/59/43 | no | 39.82 | 12.91 | 0.9070 | 0/0 | - |
| 09-mixed-document | xelatex-lm | de1020c | 1/1 | ok | 2.2539 | 0.9494 | (-0.5,39) moderate | 1.9016 | 0.9641 | 255 | 0.0169 | 0.0140 | 54/54/45 | no | 30.16 | 32.06 | 0.9333 | 0/0 | - |
| 09-mixed-document | xelatex-lm | main | 1/1 | ok | 2.2692 | 0.9490 | (-0.5,39) moderate | 1.9240 | 0.9634 | 255 | 0.0170 | 0.0141 | 54/56/43 | no | 31.94 | 32.55 | 0.9302 | 0/0 | - |
| 09-mixed-document | xelatex-lm | pipeline | 1/1 | ok | 1.8737 | 0.9647 | (0,-17) moderate | 1.5900 | 0.9706 | 255 | 0.0147 | 0.0121 | 54/59/43 | no | 9.21 | 12.95 | 0.9535 | 0/0 | - |
| 10-unicode-paragraph | lualatex | de1020c | 1/1 | ok | 3.3153 | 0.9490 | (0,0) | 3.3153 | 0.9490 | 255 | 0.0261 | 0.0212 | 8/81/0 | no | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | main | 1/1 | ok | 3.3153 | 0.9490 | (0,0) | 3.3153 | 0.9490 | 255 | 0.0261 | 0.0212 | 8/81/0 | no | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | pipeline | 1/1 | recovered | 3.3993 | 0.9471 | (0,0) | 3.3993 | 0.9471 | 255 | 0.0265 | 0.0215 | 8/81/0 | no | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex-lm | de1020c | 1/1 | ok | 3.2356 | 0.9447 | (0,0) | 3.2356 | 0.9447 | 255 | 0.0259 | 0.0211 | 82/81/79 | no | 64.10 | 1.33 | 0.8987 | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | main | 1/1 | ok | 3.2356 | 0.9447 | (0,0) | 3.2356 | 0.9447 | 255 | 0.0259 | 0.0211 | 82/81/79 | no | 64.10 | 1.33 | 0.8987 | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 2.3198 | 0.9657 | (-0.5,0) moderate | 2.1847 | 0.9681 | 255 | 0.0218 | 0.0166 | 82/81/81 | no | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 10-unicode-paragraph | pdflatex | de1020c | 1/1 | ok | 3.3620 | 0.9486 | (0,0) | 3.3620 | 0.9486 | 255 | 0.0262 | 0.0213 | 85/81/76 | no | 75.04 | 1.87 | 0.9211 | 3/1 | [p1](images/10-unicode-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex | main | 1/1 | ok | 3.3620 | 0.9486 | (0,0) | 3.3620 | 0.9486 | 255 | 0.0262 | 0.0213 | 85/81/76 | no | 75.04 | 1.87 | 0.9211 | 3/1 | [p1](images/10-unicode-paragraph/pdflatex-main-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex | pipeline | 1/1 | recovered | 3.3581 | 0.9479 | (0,0) | 3.3581 | 0.9479 | 255 | 0.0263 | 0.0211 | 85/81/77 | no | 121.15 | 2.66 | 0.9091 | 3/0 | [p1](images/10-unicode-paragraph/pdflatex-pipeline-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 3.2510 | 0.9445 | (0,0) | 3.2510 | 0.9445 | 255 | 0.0260 | 0.0212 | 82/81/80 | no | 65.00 | 1.32 | 0.9000 | 0/1 | [p1](images/10-unicode-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | main | 1/1 | ok | 3.2510 | 0.9445 | (0,0) | 3.2510 | 0.9445 | 255 | 0.0260 | 0.0212 | 82/81/80 | no | 65.00 | 1.32 | 0.9000 | 0/1 | [p1](images/10-unicode-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 2.3867 | 0.9643 | (-0.5,0) moderate | 2.2154 | 0.9673 | 255 | 0.0221 | 0.0169 | 82/81/81 | no | 0.41 | 0.36 | 1.0000 | 0/0 | [p1](images/10-unicode-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 10-unicode-paragraph | xelatex | de1020c | 1/1 | ok | 3.3629 | 0.9483 | (0,0) | 3.3629 | 0.9483 | 255 | 0.0262 | 0.0213 | 84/81/78 | no | 78.17 | 2.01 | 0.9231 | 3/1 | - |
| 10-unicode-paragraph | xelatex | main | 1/1 | ok | 3.3629 | 0.9483 | (0,0) | 3.3629 | 0.9483 | 255 | 0.0262 | 0.0213 | 84/81/78 | no | 78.17 | 2.01 | 0.9231 | 3/1 | - |
| 10-unicode-paragraph | xelatex | pipeline | 1/1 | recovered | 3.4027 | 0.9470 | (0,0) | 3.4027 | 0.9470 | 255 | 0.0265 | 0.0214 | 84/81/79 | no | 128.14 | 2.97 | 0.8987 | 3/0 | - |
| 10-unicode-paragraph | xelatex-lm | de1020c | 1/1 | ok | 3.2358 | 0.9447 | (0,0) | 3.2358 | 0.9447 | 255 | 0.0259 | 0.0211 | 82/81/79 | no | 64.10 | 1.45 | 0.8987 | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | main | 1/1 | ok | 3.2358 | 0.9447 | (0,0) | 3.2358 | 0.9447 | 255 | 0.0259 | 0.0211 | 82/81/79 | no | 64.10 | 1.45 | 0.8987 | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 2.3204 | 0.9657 | (-0.5,0) moderate | 2.1847 | 0.9681 | 255 | 0.0218 | 0.0166 | 82/81/81 | no | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex | de1020c | 1/1 | recovered | 1.4653 | 0.9696 | (18,-49) weak | 1.4605 | 0.9709 | 255 | 0.0104 | 0.0085 | 29/24/24 | no | 123.00 | 61.96 | 0.9167 | 0/0 | - |
| 11-nested-lists | lualatex | main | 1/1 | ok | 1.4651 | 0.9700 | (-21,-8) moderate | 1.2190 | 0.9785 | 255 | 0.0105 | 0.0087 | 29/29/29 | yes | 22.58 | 9.35 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex | pipeline | 1/1 | ok | 1.4784 | 0.9703 | (-16.5,-20) moderate | 1.3615 | 0.9752 | 255 | 0.0104 | 0.0087 | 29/29/29 | yes | 18.47 | 26.65 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (-2,-49) weak | 1.3067 | 0.9711 | 255 | 0.0100 | 0.0083 | 29/24/24 | no | 121.01 | 62.61 | 0.9167 | 0/0 | - |
| 11-nested-lists | lualatex-lm | main | 1/1 | ok | 1.3731 | 0.9689 | (-29,-8) moderate | 1.2485 | 0.9756 | 255 | 0.0101 | 0.0085 | 29/29/29 | yes | 24.90 | 10.02 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex-lm | pipeline | 1/1 | ok | 1.3304 | 0.9702 | (-17,-20) moderate | 1.1995 | 0.9761 | 255 | 0.0100 | 0.0083 | 29/29/29 | yes | 20.01 | 27.33 | 1.0000 | 0/0 | - |
| 11-nested-lists | pdflatex | de1020c | 1/1 | recovered | 1.4657 | 0.9698 | (19,-49) weak | 1.4558 | 0.9711 | 255 | 0.0104 | 0.0085 | 29/24/24 | no | 123.27 | 61.96 | 0.9167 | 0/0 | [p1](images/11-nested-lists/pdflatex-de1020c-export-p1-overlay.png) |
| 11-nested-lists | pdflatex | main | 1/1 | ok | 1.4717 | 0.9702 | (-20,-8) moderate | 1.1988 | 0.9789 | 255 | 0.0105 | 0.0086 | 29/29/29 | yes | 21.91 | 9.35 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-main-export-p1-overlay.png) |
| 11-nested-lists | pdflatex | pipeline | 1/1 | ok | 1.4478 | 0.9709 | (-16,-20) moderate | 1.3661 | 0.9757 | 255 | 0.0103 | 0.0085 | 29/29/29 | yes | 17.80 | 26.65 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-pipeline-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | de1020c | 1/1 | recovered | 1.3674 | 0.9687 | (23.5,-49) weak | 1.3216 | 0.9706 | 255 | 0.0101 | 0.0083 | 29/24/24 | no | 121.22 | 62.61 | 0.9167 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | main | 1/1 | ok | 1.3674 | 0.9690 | (-29,-8) moderate | 1.2361 | 0.9758 | 255 | 0.0102 | 0.0085 | 29/29/29 | yes | 24.28 | 10.02 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-main-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | pipeline | 1/1 | ok | 1.3363 | 0.9701 | (-17,-20) moderate | 1.2000 | 0.9760 | 255 | 0.0100 | 0.0083 | 29/29/29 | yes | 19.39 | 27.33 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 11-nested-lists | xelatex | de1020c | 1/1 | recovered | 1.4648 | 0.9696 | (18,-49) weak | 1.4600 | 0.9709 | 255 | 0.0104 | 0.0085 | 29/24/24 | no | 123.00 | 61.96 | 0.9167 | 0/0 | - |
| 11-nested-lists | xelatex | main | 1/1 | ok | 1.4648 | 0.9699 | (-21,-8) moderate | 1.2177 | 0.9785 | 255 | 0.0105 | 0.0087 | 29/29/29 | yes | 22.57 | 9.35 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex | pipeline | 1/1 | ok | 1.4782 | 0.9703 | (-16.5,-20) moderate | 1.3610 | 0.9752 | 255 | 0.0104 | 0.0087 | 29/29/29 | yes | 18.46 | 26.65 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (-2,-49) weak | 1.3068 | 0.9711 | 255 | 0.0100 | 0.0083 | 29/24/24 | no | 121.01 | 61.76 | 0.9167 | 0/0 | - |
| 11-nested-lists | xelatex-lm | main | 1/1 | ok | 1.3730 | 0.9689 | (-29,-8) moderate | 1.2487 | 0.9756 | 255 | 0.0101 | 0.0085 | 29/29/29 | yes | 24.90 | 9.15 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex-lm | pipeline | 1/1 | ok | 1.3305 | 0.9702 | (-17,-20) moderate | 1.1997 | 0.9761 | 255 | 0.0100 | 0.0083 | 29/29/29 | yes | 20.01 | 26.44 | 1.0000 | 0/0 | - |
| 12-justified-paragraphs | lualatex | de1020c | 1/1 | ok | 15.4038 | 0.7396 | (-3,0) weak | 15.3872 | 0.7397 | 255 | 0.1111 | 0.0927 | 360/360/360 | yes | 99.24 | 32.55 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex | main | 1/1 | ok | 15.4038 | 0.7396 | (-3,0) weak | 15.3872 | 0.7397 | 255 | 0.1111 | 0.0927 | 360/360/360 | yes | 99.24 | 32.55 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex | pipeline | 1/1 | ok | 12.9690 | 0.7908 | (0,14.5) weak | 12.9455 | 0.7922 | 255 | 0.0987 | 0.0805 | 360/360/360 | yes | 155.85 | 25.45 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | de1020c | 1/1 | ok | 13.7995 | 0.7503 | (-8.5,2.5) weak | 13.7712 | 0.7503 | 255 | 0.1052 | 0.0873 | 360/360/360 | yes | 83.76 | 8.29 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | main | 1/1 | ok | 13.7995 | 0.7503 | (-8.5,2.5) weak | 13.7712 | 0.7503 | 255 | 0.1052 | 0.0873 | 360/360/360 | yes | 83.76 | 8.29 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | pipeline | 1/1 | ok | 7.8017 | 0.8918 | (0,0) | 7.8017 | 0.8918 | 255 | 0.0766 | 0.0577 | 360/360/360 | yes | 0.00 | 0.36 | 1.0000 | 0/0 | - |
| 12-justified-paragraphs | pdflatex | de1020c | 1/1 | ok | 15.3708 | 0.7415 | (-3,0) weak | 15.3659 | 0.7414 | 255 | 0.1108 | 0.0925 | 360/360/360 | yes | 99.31 | 32.55 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-de1020c-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex | main | 1/1 | ok | 15.3708 | 0.7415 | (-3,0) weak | 15.3659 | 0.7414 | 255 | 0.1108 | 0.0925 | 360/360/360 | yes | 99.31 | 32.55 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-main-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex | pipeline | 1/1 | ok | 13.0297 | 0.7911 | (1,14.5) weak | 12.9203 | 0.7936 | 255 | 0.0986 | 0.0807 | 360/360/360 | yes | 155.88 | 25.45 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-pipeline-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | de1020c | 1/1 | ok | 13.7992 | 0.7503 | (-8.5,2.5) weak | 13.7701 | 0.7503 | 255 | 0.1051 | 0.0873 | 360/360/360 | yes | 83.76 | 8.99 | 0.8889 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | main | 1/1 | ok | 13.7992 | 0.7503 | (-8.5,2.5) weak | 13.7701 | 0.7503 | 255 | 0.1051 | 0.0873 | 360/360/360 | yes | 83.76 | 8.99 | 0.8889 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-main-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | pipeline | 1/1 | ok | 7.8191 | 0.8916 | (0,0) | 7.8191 | 0.8916 | 255 | 0.0767 | 0.0579 | 360/360/360 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 12-justified-paragraphs | xelatex | de1020c | 1/1 | ok | 15.3809 | 0.7404 | (0,0) | 15.3809 | 0.7404 | 255 | 0.1111 | 0.0928 | 360/360/360 | yes | 106.70 | 32.71 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex | main | 1/1 | ok | 15.3809 | 0.7404 | (0,0) | 15.3809 | 0.7404 | 255 | 0.1111 | 0.0928 | 360/360/360 | yes | 106.70 | 32.71 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex | pipeline | 1/1 | ok | 13.0485 | 0.7899 | (0.5,14.5) weak | 12.9800 | 0.7920 | 255 | 0.0990 | 0.0810 | 360/360/360 | yes | 159.34 | 25.61 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | de1020c | 1/1 | ok | 13.7994 | 0.7503 | (-8.5,2.5) weak | 13.7708 | 0.7503 | 255 | 0.1051 | 0.0873 | 360/360/360 | yes | 83.76 | 8.99 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | main | 1/1 | ok | 13.7994 | 0.7503 | (-8.5,2.5) weak | 13.7708 | 0.7503 | 255 | 0.1051 | 0.0873 | 360/360/360 | yes | 83.76 | 8.99 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | pipeline | 1/1 | ok | 7.8017 | 0.8918 | (0,0) | 7.8017 | 0.8918 | 255 | 0.0766 | 0.0577 | 360/360/360 | yes | 0.00 | 0.67 | 1.0000 | 0/0 | - |
| 13-math-display-rich | lualatex | de1020c | 1/1 | recovered | 0.4724 | 0.9903 | (0,3.5) moderate | 0.4325 | 0.9914 | 255 | 0.0040 | 0.0031 | 19/17/7 | no | 1.15 | 5.89 | 1.0000 | 2/2 | - |
| 13-math-display-rich | lualatex | main | 1/1 | recovered | 0.4851 | 0.9899 | (0,3.5) moderate | 0.4453 | 0.9911 | 255 | 0.0041 | 0.0032 | 19/18/7 | no | 1.15 | 5.89 | 1.0000 | 2/2 | - |
| 13-math-display-rich | lualatex | pipeline | 1/1 | recovered | 0.5820 | 0.9871 | (5.5,1.5) weak | 0.5737 | 0.9877 | 255 | 0.0046 | 0.0037 | 19/34/11 | no | 10.46 | 3.93 | 0.9091 | 2/2 | - |
| 13-math-display-rich | lualatex-lm | de1020c | 1/1 | recovered | 0.5515 | 0.9887 | (-2,4) weak | 0.5477 | 0.9892 | 255 | 0.0043 | 0.0035 | 19/17/7 | no | 3.38 | 5.45 | 1.0000 | 4/2 | - |
| 13-math-display-rich | lualatex-lm | main | 1/1 | recovered | 0.5643 | 0.9884 | (-2,4) weak | 0.5604 | 0.9888 | 255 | 0.0044 | 0.0036 | 19/18/7 | no | 3.38 | 5.45 | 1.0000 | 4/2 | - |
| 13-math-display-rich | lualatex-lm | pipeline | 1/1 | recovered | 0.5591 | 0.9872 | (3.5,1.5) moderate | 0.4986 | 0.9891 | 255 | 0.0045 | 0.0037 | 19/34/11 | no | 8.92 | 3.67 | 0.9091 | 4/2 | - |
| 13-math-display-rich | pdflatex | de1020c | 1/1 | recovered | 0.5010 | 0.9899 | (1,3.5) weak | 0.4808 | 0.9907 | 255 | 0.0041 | 0.0033 | 19/17/7 | no | 1.58 | 6.00 | 1.0000 | 2/2 | [p1](images/13-math-display-rich/pdflatex-de1020c-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex | main | 1/1 | recovered | 0.5138 | 0.9896 | (1,3.5) weak | 0.4936 | 0.9903 | 255 | 0.0042 | 0.0033 | 19/18/7 | no | 1.58 | 6.00 | 1.0000 | 2/2 | [p1](images/13-math-display-rich/pdflatex-main-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex | pipeline | 1/1 | recovered | 0.5855 | 0.9871 | (4,1.5) moderate | 0.5498 | 0.9881 | 255 | 0.0046 | 0.0037 | 19/34/11 | no | 10.73 | 4.00 | 0.9091 | 2/2 | [p1](images/13-math-display-rich/pdflatex-pipeline-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | de1020c | 1/1 | recovered | 0.5530 | 0.9888 | (-2,4) weak | 0.5406 | 0.9893 | 255 | 0.0043 | 0.0035 | 19/17/7 | no | 3.38 | 6.19 | 1.0000 | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | main | 1/1 | recovered | 0.5658 | 0.9884 | (-2,4) weak | 0.5534 | 0.9890 | 255 | 0.0044 | 0.0036 | 19/18/7 | no | 3.38 | 6.19 | 1.0000 | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-main-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | pipeline | 1/1 | recovered | 0.5614 | 0.9872 | (3.5,1.5) moderate | 0.5085 | 0.9889 | 255 | 0.0045 | 0.0037 | 19/34/11 | no | 8.92 | 4.16 | 0.9091 | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 13-math-display-rich | xelatex | de1020c | 1/1 | recovered | 0.4941 | 0.9899 | (0,0) | 0.4941 | 0.9899 | 255 | 0.0041 | 0.0032 | 21/17/8 | no | 1.56 | 2.84 | 1.0000 | 2/2 | - |
| 13-math-display-rich | xelatex | main | 1/1 | recovered | 0.5069 | 0.9896 | (0,0) | 0.5069 | 0.9896 | 255 | 0.0042 | 0.0033 | 21/18/8 | no | 1.56 | 2.84 | 1.0000 | 2/2 | - |
| 13-math-display-rich | xelatex | pipeline | 1/1 | recovered | 0.5861 | 0.9870 | (1.5,1.5) weak | 0.5572 | 0.9879 | 255 | 0.0046 | 0.0037 | 21/34/13 | no | 12.97 | 2.97 | 0.7692 | 2/2 | - |
| 13-math-display-rich | xelatex-lm | de1020c | 1/1 | recovered | 0.5517 | 0.9887 | (-2,4) weak | 0.5476 | 0.9892 | 255 | 0.0043 | 0.0035 | 21/17/8 | no | 3.20 | 3.05 | 1.0000 | 4/2 | - |
| 13-math-display-rich | xelatex-lm | main | 1/1 | recovered | 0.5645 | 0.9884 | (-2,4) weak | 0.5604 | 0.9888 | 255 | 0.0044 | 0.0036 | 21/18/8 | no | 3.20 | 3.05 | 1.0000 | 4/2 | - |
| 13-math-display-rich | xelatex-lm | pipeline | 1/1 | recovered | 0.5593 | 0.9872 | (3.5,1.5) moderate | 0.4987 | 0.9891 | 255 | 0.0045 | 0.0037 | 21/34/13 | no | 11.48 | 3.13 | 0.7692 | 4/2 | - |
| 14-math-inline-dense | lualatex | de1020c | 1/1 | recovered | 1.3179 | 0.9705 | (-34.5,13.5) | 0.9604 | 0.9783 | 255 | 0.0099 | 0.0082 | 62/59/25 | no | 52.74 | 10.98 | 0.9600 | 2/1 | - |
| 14-math-inline-dense | lualatex | main | 1/1 | recovered | 1.3179 | 0.9705 | (-34.5,13.5) | 0.9604 | 0.9783 | 255 | 0.0099 | 0.0082 | 62/59/25 | no | 52.74 | 10.98 | 0.9600 | 2/1 | - |
| 14-math-inline-dense | lualatex | pipeline | 1/1 | recovered | 0.9935 | 0.9797 | (0,0) | 0.9935 | 0.9797 | 255 | 0.0083 | 0.0065 | 62/85/25 | no | 29.50 | 3.07 | 1.0000 | 2/2 | - |
| 14-math-inline-dense | lualatex-lm | de1020c | 1/1 | recovered | 1.2473 | 0.9702 | (5,13.5) moderate | 1.1675 | 0.9735 | 255 | 0.0097 | 0.0079 | 62/59/23 | no | 60.36 | 10.71 | 0.9565 | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | main | 1/1 | recovered | 1.2473 | 0.9702 | (5,13.5) moderate | 1.1675 | 0.9735 | 255 | 0.0097 | 0.0079 | 62/59/23 | no | 60.36 | 10.71 | 0.9565 | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | pipeline | 1/1 | recovered | 0.9471 | 0.9801 | (0,1.5) moderate | 0.8319 | 0.9817 | 255 | 0.0080 | 0.0064 | 62/85/27 | no | 4.37 | 2.30 | 1.0000 | 2/2 | - |
| 14-math-inline-dense | pdflatex | de1020c | 1/1 | recovered | 1.3129 | 0.9708 | (-34,13.5) | 0.9608 | 0.9783 | 255 | 0.0099 | 0.0082 | 53/59/18 | no | 56.56 | 13.98 | 0.8889 | 2/1 | [p1](images/14-math-inline-dense/pdflatex-de1020c-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex | main | 1/1 | recovered | 1.3129 | 0.9708 | (-34,13.5) | 0.9608 | 0.9783 | 255 | 0.0099 | 0.0082 | 53/59/18 | no | 56.56 | 13.98 | 0.8889 | 2/1 | [p1](images/14-math-inline-dense/pdflatex-main-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex | pipeline | 1/1 | recovered | 0.9830 | 0.9799 | (0,0) | 0.9830 | 0.9799 | 255 | 0.0082 | 0.0064 | 53/85/21 | no | 24.56 | 2.58 | 1.0000 | 2/2 | [p1](images/14-math-inline-dense/pdflatex-pipeline-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | de1020c | 1/1 | recovered | 1.2469 | 0.9702 | (5,13.5) moderate | 1.1673 | 0.9735 | 255 | 0.0097 | 0.0079 | 55/59/19 | no | 64.42 | 13.17 | 0.8947 | 2/1 | [p1](images/14-math-inline-dense/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | main | 1/1 | recovered | 1.2469 | 0.9702 | (5,13.5) moderate | 1.1673 | 0.9735 | 255 | 0.0097 | 0.0079 | 55/59/19 | no | 64.42 | 13.17 | 0.8947 | 2/1 | [p1](images/14-math-inline-dense/pdflatex-lm-main-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | pipeline | 1/1 | recovered | 0.9478 | 0.9801 | (0,1.5) moderate | 0.8320 | 0.9817 | 255 | 0.0080 | 0.0064 | 55/85/24 | no | 13.39 | 3.13 | 1.0000 | 2/2 | [p1](images/14-math-inline-dense/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 14-math-inline-dense | xelatex | de1020c | 1/1 | recovered | 1.3179 | 0.9705 | (-34.5,13.5) | 0.9575 | 0.9783 | 255 | 0.0099 | 0.0082 | 63/59/25 | no | 52.75 | 12.18 | 0.9600 | 2/1 | - |
| 14-math-inline-dense | xelatex | main | 1/1 | recovered | 1.3179 | 0.9705 | (-34.5,13.5) | 0.9575 | 0.9783 | 255 | 0.0099 | 0.0082 | 63/59/25 | no | 52.75 | 12.18 | 0.9600 | 2/1 | - |
| 14-math-inline-dense | xelatex | pipeline | 1/1 | recovered | 0.9936 | 0.9797 | (0,0) | 0.9936 | 0.9797 | 255 | 0.0083 | 0.0065 | 63/85/26 | no | 28.67 | 6.45 | 1.0000 | 2/2 | - |
| 14-math-inline-dense | xelatex-lm | de1020c | 1/1 | recovered | 1.2473 | 0.9702 | (5,13.5) moderate | 1.1675 | 0.9735 | 255 | 0.0097 | 0.0079 | 63/59/23 | no | 60.36 | 11.58 | 0.9565 | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | main | 1/1 | recovered | 1.2473 | 0.9702 | (5,13.5) moderate | 1.1675 | 0.9735 | 255 | 0.0097 | 0.0079 | 63/59/23 | no | 60.36 | 11.58 | 0.9565 | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | pipeline | 1/1 | recovered | 0.9471 | 0.9801 | (0,1.5) moderate | 0.8319 | 0.9817 | 255 | 0.0080 | 0.0064 | 63/85/28 | no | 4.40 | 5.65 | 1.0000 | 2/2 | - |
| 15-three-page-sections | lualatex | de1020c | 3/3 | recovered | 26.4239 | 0.5635 | (0.5,20) moderate; (0.5,19) weak; (0.5,-24.5) moderate | 25.2158 | 0.5876 | 255 | 0.1894 | 0.1584 | 1806/1806/1806 | yes | 154.50 | 39.85 | 0.8984 | 0/0 | - |
| 15-three-page-sections | lualatex | main | 3/3 | recovered | 26.4376 | 0.5632 | (0.5,20) moderate; (0.5,19) weak; (0.5,-24.5) moderate | 25.2273 | 0.5874 | 255 | 0.1895 | 0.1585 | 1806/1809/1806 | no | 154.51 | 39.85 | 0.8978 | 0/0 | - |
| 15-three-page-sections | lualatex | pipeline | 3/3 | recovered | 23.9029 | 0.6049 | (0,14) weak; (0.5,-26.5) moderate; (0.5,-55.5) moderate | 21.6032 | 0.6601 | 255 | 0.1767 | 0.1465 | 1806/1809/1806 | no | 181.70 | 54.62 | 0.8989 | 0/0 | - |
| 15-three-page-sections | lualatex-lm | de1020c | 3/3 | recovered | 24.2612 | 0.5493 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0398 | 0.5821 | 255 | 0.1832 | 0.1520 | 1806/1806/1806 | yes | 122.99 | 44.85 | 0.8926 | 3/0 | - |
| 15-three-page-sections | lualatex-lm | main | 3/3 | recovered | 24.2718 | 0.5491 | (0.5,22.5) moderate; (3.5,-24) weak; (0.5,-24) weak | 23.0722 | 0.5811 | 255 | 0.1833 | 0.1520 | 1806/1809/1806 | no | 123.00 | 44.85 | 0.8920 | 3/0 | - |
| 15-three-page-sections | lualatex-lm | pipeline | 3/3 | recovered | 19.9026 | 0.6420 | (0,0); (0,-26.5) moderate; (0,-26.5) moderate | 18.3451 | 0.6885 | 255 | 0.1619 | 0.1304 | 1806/1809/1806 | no | 0.04 | 64.70 | 0.9994 | 3/0 | - |
| 15-three-page-sections | pdflatex | de1020c | 3/3 | recovered | 26.6994 | 0.5541 | (0.5,36.5) moderate; (0,-24.5) moderate; (0,-24.5) weak | 25.0916 | 0.5919 | 255 | 0.1902 | 0.1599 | 1806/1806/1806 | yes | 136.77 | 43.12 | 0.8914 | 0/0 | [p1](images/15-three-page-sections/pdflatex-de1020c-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex | main | 3/3 | recovered | 26.7135 | 0.5538 | (0.5,36.5) moderate; (0,-24.5) moderate; (0,-24.5) weak | 25.1031 | 0.5918 | 255 | 0.1903 | 0.1599 | 1806/1809/1806 | no | 136.79 | 43.12 | 0.8908 | 0/0 | [p1](images/15-three-page-sections/pdflatex-main-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex | pipeline | 3/3 | recovered | 24.0962 | 0.5969 | (1,-0.5) moderate; (0.5,-55.5) moderate; (0.5,-12.5) moderate | 21.8316 | 0.6499 | 255 | 0.1772 | 0.1476 | 1806/1809/1806 | no | 187.15 | 62.95 | 0.8915 | 0/0 | [p1](images/15-three-page-sections/pdflatex-pipeline-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | de1020c | 3/3 | recovered | 24.2615 | 0.5493 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0362 | 0.5822 | 255 | 0.1833 | 0.1521 | 1806/1806/1806 | yes | 123.00 | 44.54 | 0.8926 | 3/0 | [p1](images/15-three-page-sections/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | main | 3/3 | recovered | 24.2722 | 0.5491 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0478 | 0.5820 | 255 | 0.1833 | 0.1521 | 1806/1809/1806 | no | 123.00 | 44.54 | 0.8920 | 3/0 | [p1](images/15-three-page-sections/pdflatex-lm-main-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | pipeline | 3/3 | recovered | 19.9143 | 0.6418 | (0,0); (0,-26.5) moderate; (0,-26.5) moderate | 18.3574 | 0.6881 | 255 | 0.1620 | 0.1306 | 1806/1809/1806 | no | 0.05 | 64.16 | 0.9994 | 3/0 | [p1](images/15-three-page-sections/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 15-three-page-sections | xelatex | de1020c | 3/3 | recovered | 26.4202 | 0.5637 | (-3,20) moderate; (-3,19) weak; (-3,-24.5) moderate | 25.1892 | 0.5879 | 255 | 0.1893 | 0.1587 | 1806/1806/1806 | yes | 154.49 | 39.85 | 0.8984 | 0/0 | - |
| 15-three-page-sections | xelatex | main | 3/3 | recovered | 26.4339 | 0.5634 | (-3,20) moderate; (-3,19) weak; (-3,-24.5) moderate | 25.2007 | 0.5877 | 255 | 0.1894 | 0.1588 | 1806/1809/1806 | no | 154.51 | 39.85 | 0.8978 | 0/0 | - |
| 15-three-page-sections | xelatex | pipeline | 3/3 | recovered | 23.9337 | 0.6046 | (0,14) weak; (1,-26.5) moderate; (1,-55.5) moderate | 21.7068 | 0.6586 | 255 | 0.1768 | 0.1470 | 1806/1809/1806 | no | 181.69 | 54.62 | 0.8989 | 0/0 | - |
| 15-three-page-sections | xelatex-lm | de1020c | 3/3 | recovered | 24.2611 | 0.5493 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0397 | 0.5821 | 255 | 0.1832 | 0.1519 | 1806/1806/1806 | yes | 122.99 | 44.55 | 0.8926 | 3/0 | - |
| 15-three-page-sections | xelatex-lm | main | 3/3 | recovered | 24.2716 | 0.5491 | (0.5,22.5) moderate; (3.5,-24) weak; (0.5,-24) weak | 23.0720 | 0.5811 | 255 | 0.1833 | 0.1520 | 1806/1809/1806 | no | 123.00 | 44.55 | 0.8920 | 3/0 | - |
| 15-three-page-sections | xelatex-lm | pipeline | 3/3 | recovered | 19.9041 | 0.6419 | (0,0); (0,-26.5) moderate; (0,-26.5) moderate | 18.3465 | 0.6884 | 255 | 0.1618 | 0.1304 | 1806/1809/1806 | no | 0.04 | 64.16 | 0.9994 | 3/0 | - |
| 16-heading-page-break | lualatex | de1020c | 2/2 | ok | 20.5835 | 0.6612 | (0,0); (0,20.5) moderate | 20.0992 | 0.6687 | 255 | 0.1484 | 0.1240 | 962/962/962 | yes | 161.32 | 45.97 | 0.8950 | 0/0 | - |
| 16-heading-page-break | lualatex | main | 2/2 | ok | 20.5883 | 0.6611 | (0,0); (0,20.5) moderate | 20.1037 | 0.6686 | 255 | 0.1484 | 0.1240 | 962/963/962 | no | 161.35 | 45.97 | 0.8939 | 0/0 | - |
| 16-heading-page-break | lualatex | pipeline | 2/2 | ok | 17.5089 | 0.7218 | (0,0); (-1,57.5) weak | 17.3220 | 0.7266 | 255 | 0.1328 | 0.1087 | 962/963/962 | no | 162.19 | 36.60 | 0.8949 | 0/0 | - |
| 16-heading-page-break | lualatex-lm | de1020c | 2/2 | ok | 19.0181 | 0.6506 | (-1,0) weak; (-1,20) moderate | 18.0483 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/962/962 | yes | 137.29 | 13.93 | 0.8888 | 1/0 | - |
| 16-heading-page-break | lualatex-lm | main | 2/2 | ok | 19.0252 | 0.6506 | (-1,0) weak; (-1,20) moderate | 18.0517 | 0.6754 | 255 | 0.1440 | 0.1195 | 962/963/962 | no | 137.30 | 13.93 | 0.8877 | 1/0 | - |
| 16-heading-page-break | lualatex-lm | pipeline | 2/2 | ok | 10.7729 | 0.8475 | (0,0); (0,0) | 10.7729 | 0.8475 | 255 | 0.1046 | 0.0792 | 962/963/962 | no | 0.07 | 0.38 | 0.9990 | 1/0 | - |
| 16-heading-page-break | pdflatex | de1020c | 2/2 | ok | 20.5977 | 0.6620 | (0,-14.5) weak; (0,20.5) moderate | 20.0449 | 0.6704 | 255 | 0.1481 | 0.1240 | 962/962/962 | yes | 141.32 | 45.21 | 0.8952 | 0/0 | [p1](images/16-heading-page-break/pdflatex-de1020c-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex | main | 2/2 | ok | 20.6026 | 0.6619 | (0,-14.5) weak; (0,20.5) moderate | 20.0498 | 0.6703 | 255 | 0.1481 | 0.1240 | 962/963/962 | no | 141.35 | 45.21 | 0.8940 | 0/0 | [p1](images/16-heading-page-break/pdflatex-main-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex | pipeline | 2/2 | ok | 17.4975 | 0.7234 | (-1.5,0) weak; (-1.5,57.5) moderate | 17.1872 | 0.7303 | 255 | 0.1324 | 0.1086 | 962/963/962 | no | 178.07 | 35.83 | 0.8950 | 0/0 | [p1](images/16-heading-page-break/pdflatex-pipeline-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | de1020c | 2/2 | ok | 19.0175 | 0.6505 | (-1,0) weak; (-1,20) moderate | 18.0489 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/962/962 | yes | 137.30 | 14.56 | 0.8888 | 1/0 | [p1](images/16-heading-page-break/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | main | 2/2 | ok | 19.0245 | 0.6505 | (-1,0) weak; (-1,20) moderate | 18.0522 | 0.6754 | 255 | 0.1440 | 0.1196 | 962/963/962 | no | 137.31 | 14.56 | 0.8877 | 1/0 | [p1](images/16-heading-page-break/pdflatex-lm-main-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | pipeline | 2/2 | ok | 10.8017 | 0.8468 | (0,0); (0,0) | 10.8017 | 0.8468 | 255 | 0.1047 | 0.0794 | 962/963/962 | no | 0.07 | 0.74 | 0.9990 | 1/0 | [p1](images/16-heading-page-break/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 16-heading-page-break | xelatex | de1020c | 2/2 | ok | 20.6100 | 0.6610 | (-3,-14.5) weak; (0,20.5) moderate | 20.1333 | 0.6680 | 255 | 0.1485 | 0.1243 | 962/962/962 | yes | 161.33 | 45.97 | 0.8950 | 0/0 | - |
| 16-heading-page-break | xelatex | main | 2/2 | ok | 20.6148 | 0.6609 | (-3,-14.5) weak; (0,20.5) moderate | 20.1380 | 0.6679 | 255 | 0.1486 | 0.1243 | 962/963/962 | no | 161.35 | 45.97 | 0.8939 | 0/0 | - |
| 16-heading-page-break | xelatex | pipeline | 2/2 | ok | 17.5395 | 0.7212 | (1.5,0) weak; (-1,57.5) weak | 17.3052 | 0.7263 | 255 | 0.1329 | 0.1090 | 962/963/962 | no | 162.22 | 36.60 | 0.8949 | 0/0 | - |
| 16-heading-page-break | xelatex-lm | de1020c | 2/2 | ok | 19.0182 | 0.6506 | (-1,0) weak; (-1,20) moderate | 18.0480 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/962/962 | yes | 137.29 | 14.56 | 0.8888 | 1/0 | - |
| 16-heading-page-break | xelatex-lm | main | 2/2 | ok | 19.0252 | 0.6506 | (-1,0) weak; (-1,20) moderate | 18.0514 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/963/962 | no | 137.30 | 14.56 | 0.8877 | 1/0 | - |
| 16-heading-page-break | xelatex-lm | pipeline | 2/2 | ok | 10.7756 | 0.8474 | (0,0); (0,0) | 10.7756 | 0.8474 | 255 | 0.1046 | 0.0792 | 962/963/962 | no | 0.07 | 0.73 | 0.9990 | 1/0 | - |
| 17-apostrophes | lualatex | de1020c | 1/1 | ok | 0.8502 | 0.9875 | (-1.5,0) weak | 0.8119 | 0.9882 | 255 | 0.0068 | 0.0053 | 10/25/7 | no | 1.20 | 0.42 | 1.0000 | 0/0 | - |
| 17-apostrophes | lualatex | main | 1/1 | ok | 0.8502 | 0.9875 | (-1.5,0) weak | 0.8119 | 0.9882 | 255 | 0.0068 | 0.0053 | 10/25/7 | no | 1.20 | 0.42 | 1.0000 | 0/0 | - |
| 17-apostrophes | lualatex | pipeline | 1/1 | ok | 0.9397 | 0.9852 | (53.5,0) moderate | 0.8851 | 0.9860 | 255 | 0.0073 | 0.0058 | 10/25/9 | no | 56.38 | 0.41 | 0.8889 | 0/0 | - |
| 17-apostrophes | lualatex-lm | de1020c | 1/1 | ok | 0.8622 | 0.9848 | (0,0) | 0.8622 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.56 | 0.33 | 0.9231 | 0/0 | - |
| 17-apostrophes | lualatex-lm | main | 1/1 | ok | 0.8622 | 0.9848 | (0,0) | 0.8622 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.56 | 0.33 | 0.9231 | 0/0 | - |
| 17-apostrophes | lualatex-lm | pipeline | 1/1 | ok | 0.6750 | 0.9896 | (-0.5,0) moderate | 0.6315 | 0.9900 | 255 | 0.0061 | 0.0047 | 25/25/25 | yes | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 17-apostrophes | pdflatex | de1020c | 1/1 | ok | 0.8063 | 0.9887 | (0,0) | 0.8063 | 0.9887 | 255 | 0.0066 | 0.0052 | 25/25/13 | no | 2.67 | 0.44 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-de1020c-export-p1-overlay.png) |
| 17-apostrophes | pdflatex | main | 1/1 | ok | 0.8063 | 0.9887 | (0,0) | 0.8063 | 0.9887 | 255 | 0.0066 | 0.0052 | 25/25/13 | no | 2.67 | 0.44 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-main-export-p1-overlay.png) |
| 17-apostrophes | pdflatex | pipeline | 1/1 | ok | 0.9356 | 0.9856 | (0,0) | 0.9356 | 0.9856 | 255 | 0.0072 | 0.0058 | 25/25/25 | yes | 51.51 | 0.99 | 0.9200 | 0/0 | [p1](images/17-apostrophes/pdflatex-pipeline-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | de1020c | 1/1 | ok | 0.8624 | 0.9848 | (0,0) | 0.8624 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.55 | 0.70 | 0.9231 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | main | 1/1 | ok | 0.8624 | 0.9848 | (0,0) | 0.8624 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.55 | 0.70 | 0.9231 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-main-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | pipeline | 1/1 | ok | 0.6744 | 0.9896 | (-0.5,0) moderate | 0.6313 | 0.9900 | 255 | 0.0061 | 0.0047 | 25/25/25 | yes | 0.02 | 0.67 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 17-apostrophes | xelatex | de1020c | 1/1 | ok | 0.7981 | 0.9883 | (0,0) | 0.7981 | 0.9883 | 255 | 0.0066 | 0.0051 | 25/25/13 | no | 2.69 | 0.44 | 1.0000 | 0/0 | - |
| 17-apostrophes | xelatex | main | 1/1 | ok | 0.7981 | 0.9883 | (0,0) | 0.7981 | 0.9883 | 255 | 0.0066 | 0.0051 | 25/25/13 | no | 2.69 | 0.44 | 1.0000 | 0/0 | - |
| 17-apostrophes | xelatex | pipeline | 1/1 | ok | 0.9402 | 0.9852 | (0,0) | 0.9402 | 0.9852 | 255 | 0.0073 | 0.0058 | 25/25/25 | yes | 51.51 | 0.99 | 0.9200 | 0/0 | - |
| 17-apostrophes | xelatex-lm | de1020c | 1/1 | ok | 0.8620 | 0.9848 | (0,0) | 0.8620 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.56 | 0.70 | 0.9231 | 0/0 | - |
| 17-apostrophes | xelatex-lm | main | 1/1 | ok | 0.8620 | 0.9848 | (0,0) | 0.8620 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.56 | 0.70 | 0.9231 | 0/0 | - |
| 17-apostrophes | xelatex-lm | pipeline | 1/1 | ok | 0.6748 | 0.9896 | (-0.5,0) moderate | 0.6310 | 0.9900 | 255 | 0.0061 | 0.0047 | 25/25/25 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex | de1020c | 1/1 | ok | 1.3714 | 0.9790 | (0,0) | 1.3714 | 0.9790 | 255 | 0.0109 | 0.0087 | 36/36/36 | yes | 41.08 | 1.23 | 0.8889 | 0/0 | - |
| 18-ligatures | lualatex | main | 1/1 | ok | 1.3714 | 0.9790 | (0,0) | 1.3714 | 0.9790 | 255 | 0.0109 | 0.0087 | 36/36/36 | yes | 41.08 | 1.23 | 0.8889 | 0/0 | - |
| 18-ligatures | lualatex | pipeline | 1/1 | ok | 1.3969 | 0.9783 | (0,0) | 1.3969 | 0.9783 | 255 | 0.0111 | 0.0087 | 36/36/36 | yes | 50.01 | 1.21 | 0.8889 | 0/0 | - |
| 18-ligatures | lualatex-lm | de1020c | 1/1 | ok | 1.3134 | 0.9787 | (0,0) | 1.3134 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.13 | 0.34 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex-lm | main | 1/1 | ok | 1.3134 | 0.9787 | (0,0) | 1.3134 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.13 | 0.34 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex-lm | pipeline | 1/1 | ok | 0.9754 | 0.9860 | (0,0) | 0.9754 | 0.9860 | 255 | 0.0092 | 0.0070 | 36/36/36 | yes | 0.21 | 0.36 | 1.0000 | 0/0 | - |
| 18-ligatures | pdflatex | de1020c | 1/1 | ok | 1.2894 | 0.9808 | (0,0) | 1.2894 | 0.9808 | 255 | 0.0105 | 0.0082 | 36/36/36 | yes | 40.70 | 1.23 | 0.8889 | 0/0 | [p1](images/18-ligatures/pdflatex-de1020c-export-p1-overlay.png) |
| 18-ligatures | pdflatex | main | 1/1 | ok | 1.2894 | 0.9808 | (0,0) | 1.2894 | 0.9808 | 255 | 0.0105 | 0.0082 | 36/36/36 | yes | 40.70 | 1.23 | 0.8889 | 0/0 | [p1](images/18-ligatures/pdflatex-main-export-p1-overlay.png) |
| 18-ligatures | pdflatex | pipeline | 1/1 | ok | 1.3694 | 0.9794 | (0,0) | 1.3694 | 0.9794 | 255 | 0.0108 | 0.0085 | 36/36/36 | yes | 49.41 | 1.21 | 0.8889 | 0/0 | [p1](images/18-ligatures/pdflatex-pipeline-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | de1020c | 1/1 | ok | 1.3148 | 0.9787 | (0,0) | 1.3148 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.14 | 0.69 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | main | 1/1 | ok | 1.3148 | 0.9787 | (0,0) | 1.3148 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.14 | 0.69 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-main-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | pipeline | 1/1 | ok | 0.9763 | 0.9860 | (0,0) | 0.9763 | 0.9860 | 255 | 0.0092 | 0.0070 | 36/36/36 | yes | 0.22 | 0.67 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 18-ligatures | xelatex | de1020c | 1/1 | ok | 1.3498 | 0.9797 | (0,0) | 1.3498 | 0.9797 | 255 | 0.0108 | 0.0087 | 36/36/36 | yes | 40.84 | 1.23 | 0.8889 | 0/0 | - |
| 18-ligatures | xelatex | main | 1/1 | ok | 1.3498 | 0.9797 | (0,0) | 1.3498 | 0.9797 | 255 | 0.0108 | 0.0087 | 36/36/36 | yes | 40.84 | 1.23 | 0.8889 | 0/0 | - |
| 18-ligatures | xelatex | pipeline | 1/1 | ok | 1.3851 | 0.9786 | (0,0) | 1.3851 | 0.9786 | 255 | 0.0111 | 0.0088 | 36/36/36 | yes | 49.75 | 1.21 | 0.8889 | 0/0 | - |
| 18-ligatures | xelatex-lm | de1020c | 1/1 | ok | 1.3133 | 0.9787 | (0,0) | 1.3133 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.13 | 0.69 | 1.0000 | 0/0 | - |
| 18-ligatures | xelatex-lm | main | 1/1 | ok | 1.3133 | 0.9787 | (0,0) | 1.3133 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.13 | 0.69 | 1.0000 | 0/0 | - |
| 18-ligatures | xelatex-lm | pipeline | 1/1 | ok | 0.9752 | 0.9860 | (0,0) | 0.9752 | 0.9860 | 255 | 0.0092 | 0.0070 | 36/36/36 | yes | 0.21 | 0.67 | 1.0000 | 0/0 | - |

## Diagnostic: preview-equivalent comparison (CoreText draw of compile_result vs reference PDF raster)

Weaker than a capture of the real preview: it re-implements the app's draw code path rather than exercising the SwiftUI Canvas. Word-box metrics are not available for this side (no PDF), so they are omitted.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| raw | SSIM₈ raw | registration Δ pt (dx,dy per page; `weak`/`moderate` = shift explains <25% of the error) | mean\|Δ\| after reg | SSIM₈ after reg | max | differing | ≥thr | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-plain-paragraph | lualatex | de1020c | 1/1 | ok | 0.1686 | 0.9978 | (0,0) | 0.1686 | 0.9978 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.1686 | 0.9978 | (0,0) | 0.1686 | 0.9978 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | pipeline | 1/1 | ok | 0.3905 | 0.9942 | (0.5,0) weak | 0.3836 | 0.9943 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | de1020c | 1/1 | ok | 0.3642 | 0.9942 | (0,0) | 0.3642 | 0.9942 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | main | 1/1 | ok | 0.3642 | 0.9942 | (0,0) | 0.3642 | 0.9942 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | pipeline | 1/1 | ok | 0.2320 | 0.9967 | (0,0) | 0.2320 | 0.9967 | 255 | 0.0026 | 0.0018 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.1741 | 0.9977 | (0,0) | 0.1741 | 0.9977 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.1741 | 0.9977 | (0,0) | 0.1741 | 0.9977 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | pipeline | 1/1 | ok | 0.3868 | 0.9945 | (0,0) | 0.3868 | 0.9945 | 255 | 0.0031 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 0.3639 | 0.9942 | (0,0) | 0.3639 | 0.9942 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | main | 1/1 | ok | 0.3639 | 0.9942 | (0,0) | 0.3639 | 0.9942 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 0.2332 | 0.9966 | (0,0) | 0.2332 | 0.9966 | 255 | 0.0026 | 0.0018 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.1716 | 0.9978 | (0,0) | 0.1716 | 0.9978 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.1716 | 0.9978 | (0,0) | 0.1716 | 0.9978 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | pipeline | 1/1 | ok | 0.3905 | 0.9942 | (0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0031 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | de1020c | 1/1 | ok | 0.3644 | 0.9942 | (0,0) | 0.3644 | 0.9942 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | main | 1/1 | ok | 0.3644 | 0.9942 | (0,0) | 0.3644 | 0.9942 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | pipeline | 1/1 | ok | 0.2320 | 0.9967 | (0,0) | 0.2320 | 0.9967 | 255 | 0.0026 | 0.0018 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 8.3831 | 0.8656 | (-3,0) weak | 8.3728 | 0.8658 | 255 | 0.0613 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 8.3831 | 0.8656 | (-3,0) weak | 8.3728 | 0.8658 | 255 | 0.0613 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | pipeline | 1/1 | ok | 7.1754 | 0.8902 | (0,0) | 7.1754 | 0.8902 | 255 | 0.0554 | 0.0449 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | de1020c | 1/1 | ok | 7.7865 | 0.8620 | (-0.5,-14.5) weak | 7.7677 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | main | 1/1 | ok | 7.7865 | 0.8620 | (-0.5,-14.5) weak | 7.7677 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | pipeline | 1/1 | ok | 4.5417 | 0.9367 | (0,0) | 4.5417 | 0.9367 | 255 | 0.0447 | 0.0336 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 8.3720 | 0.8664 | (0,0) | 8.3720 | 0.8664 | 255 | 0.0611 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 8.3720 | 0.8664 | (0,0) | 8.3720 | 0.8664 | 255 | 0.0611 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | pipeline | 1/1 | ok | 7.1612 | 0.8910 | (1,0) weak | 7.0823 | 0.8919 | 255 | 0.0553 | 0.0448 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 7.7859 | 0.8621 | (-0.5,-14.5) weak | 7.7684 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | main | 1/1 | ok | 7.7859 | 0.8621 | (-0.5,-14.5) weak | 7.7684 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 4.5499 | 0.9366 | (0,0) | 4.5499 | 0.9366 | 255 | 0.0447 | 0.0337 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 8.3703 | 0.8657 | (0,0) | 8.3703 | 0.8657 | 255 | 0.0613 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 8.3703 | 0.8657 | (0,0) | 8.3703 | 0.8657 | 255 | 0.0613 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | pipeline | 1/1 | ok | 7.1991 | 0.8902 | (0.5,0) weak | 7.1660 | 0.8904 | 255 | 0.0556 | 0.0451 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | de1020c | 1/1 | ok | 7.7862 | 0.8620 | (-0.5,-14.5) weak | 7.7678 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | main | 1/1 | ok | 7.7862 | 0.8620 | (-0.5,-14.5) weak | 7.7678 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | pipeline | 1/1 | ok | 4.5425 | 0.9367 | (0,0) | 4.5425 | 0.9367 | 255 | 0.0447 | 0.0336 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.3394 | 0.9793 | (0,52.5) moderate | 1.1862 | 0.9829 | 255 | 0.0085 | 0.0073 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.3597 | 0.9789 | (0.5,52.5) moderate | 1.2078 | 0.9824 | 255 | 0.0086 | 0.0074 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | pipeline | 1/1 | ok | 0.9609 | 0.9877 | (0,-0.5) weak | 0.9213 | 0.9881 | 255 | 0.0067 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | de1020c | 1/1 | ok | 1.2336 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | main | 1/1 | ok | 1.2425 | 0.9769 | (0,53.5) moderate | 1.1122 | 0.9812 | 255 | 0.0082 | 0.0070 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | pipeline | 1/1 | ok | 0.9263 | 0.9869 | (0,0.5) weak | 0.9034 | 0.9876 | 255 | 0.0066 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.3496 | 0.9793 | (0.5,52.5) moderate | 1.2021 | 0.9827 | 255 | 0.0085 | 0.0072 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.3719 | 0.9789 | (0.5,52.5) moderate | 1.2209 | 0.9823 | 255 | 0.0086 | 0.0074 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | pipeline | 1/1 | ok | 0.9951 | 0.9873 | (0.5,-0.5) weak | 0.9728 | 0.9875 | 255 | 0.0068 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | de1020c | 1/1 | ok | 1.2335 | 0.9771 | (0,54) moderate | 1.0954 | 0.9811 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | main | 1/1 | ok | 1.2427 | 0.9769 | (0.5,54) moderate | 1.1273 | 0.9806 | 255 | 0.0082 | 0.0070 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | pipeline | 1/1 | ok | 0.9297 | 0.9868 | (0,1) weak | 0.8922 | 0.9875 | 255 | 0.0066 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.3370 | 0.9793 | (0,52.5) moderate | 1.1869 | 0.9829 | 255 | 0.0085 | 0.0073 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.3601 | 0.9789 | (0.5,52.5) moderate | 1.2088 | 0.9824 | 255 | 0.0086 | 0.0074 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | pipeline | 1/1 | ok | 0.9660 | 0.9877 | (0,-0.5) weak | 0.9258 | 0.9880 | 255 | 0.0067 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | de1020c | 1/1 | ok | 1.2337 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | main | 1/1 | ok | 1.2424 | 0.9769 | (0,53.5) moderate | 1.1122 | 0.9812 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | pipeline | 1/1 | ok | 0.9262 | 0.9869 | (0,0.5) weak | 0.9034 | 0.9876 | 255 | 0.0066 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.3808 | 0.9945 | (0,0) | 0.3808 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.3808 | 0.9945 | (0,0) | 0.3808 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | pipeline | 1/1 | ok | 0.4278 | 0.9938 | (0,0) | 0.4278 | 0.9938 | 255 | 0.0033 | 0.0026 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | de1020c | 1/1 | ok | 0.4244 | 0.9933 | (-49,0) weak | 0.4148 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | main | 1/1 | ok | 0.4244 | 0.9933 | (-49,0) weak | 0.4148 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | pipeline | 1/1 | ok | 0.3473 | 0.9949 | (-0.5,0) weak | 0.3304 | 0.9952 | 255 | 0.0030 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.3837 | 0.9944 | (-0.5,0) weak | 0.3764 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.3837 | 0.9944 | (-0.5,0) weak | 0.3764 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | pipeline | 1/1 | ok | 0.4168 | 0.9939 | (0,0) | 0.4168 | 0.9939 | 255 | 0.0033 | 0.0026 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | de1020c | 1/1 | ok | 0.4246 | 0.9933 | (-49,0) weak | 0.4162 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | main | 1/1 | ok | 0.4246 | 0.9933 | (-49,0) weak | 0.4162 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | pipeline | 1/1 | ok | 0.3564 | 0.9948 | (-0.5,0) moderate | 0.3295 | 0.9953 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.3849 | 0.9944 | (-0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0032 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.3849 | 0.9944 | (-0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0032 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | pipeline | 1/1 | ok | 0.4274 | 0.9938 | (31.5,0) weak | 0.4233 | 0.9941 | 255 | 0.0033 | 0.0026 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | de1020c | 1/1 | ok | 0.4245 | 0.9933 | (-49,0) weak | 0.4138 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | main | 1/1 | ok | 0.4245 | 0.9933 | (-49,0) weak | 0.4138 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | pipeline | 1/1 | ok | 0.3308 | 0.9952 | (0,0) | 0.3308 | 0.9952 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.2971 | 0.9955 | (0,0) | 0.2971 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.2971 | 0.9955 | (0,0) | 0.2971 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | pipeline | 1/1 | ok | 0.3579 | 0.9948 | (11.5,0) moderate | 0.3267 | 0.9952 | 255 | 0.0029 | 0.0022 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex-lm | de1020c | 1/1 | ok | 0.3619 | 0.9938 | (2.5,0) weak | 0.3494 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | main | 1/1 | ok | 0.3619 | 0.9938 | (2.5,0) weak | 0.3494 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | pipeline | 1/1 | ok | 0.2636 | 0.9960 | (-0.5,0) moderate | 0.2087 | 0.9969 | 255 | 0.0024 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.3125 | 0.9955 | (0,0) | 0.3125 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.3125 | 0.9955 | (0,0) | 0.3125 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex | pipeline | 1/1 | ok | 0.3571 | 0.9950 | (11.5,0) moderate | 0.3060 | 0.9957 | 255 | 0.0029 | 0.0022 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex-lm | de1020c | 1/1 | ok | 0.3618 | 0.9938 | (2.5,0) weak | 0.3496 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex-lm | main | 1/1 | ok | 0.3618 | 0.9938 | (2.5,0) weak | 0.3496 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex-lm | pipeline | 1/1 | ok | 0.2655 | 0.9960 | (-0.5,0) moderate | 0.2082 | 0.9969 | 255 | 0.0024 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.2963 | 0.9955 | (0,0) | 0.2963 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.2963 | 0.9955 | (0,0) | 0.2963 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | pipeline | 1/1 | ok | 0.3577 | 0.9948 | (11.5,0) moderate | 0.3296 | 0.9951 | 255 | 0.0029 | 0.0022 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex-lm | de1020c | 1/1 | ok | 0.3620 | 0.9938 | (2.5,0) weak | 0.3493 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | main | 1/1 | ok | 0.3620 | 0.9938 | (2.5,0) weak | 0.3493 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | pipeline | 1/1 | ok | 0.2638 | 0.9960 | (-0.5,0) moderate | 0.2088 | 0.9969 | 255 | 0.0024 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.3823 | 0.9934 | (0,3.5) | 0.2551 | 0.9959 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | ok | 0.3823 | 0.9934 | (0,3.5) | 0.2551 | 0.9959 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | pipeline | 1/1 | ok | 0.3048 | 0.9947 | (0,0) | 0.3048 | 0.9947 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | de1020c | 1/1 | ok | 0.3683 | 0.9931 | (-32.5,3.5) moderate | 0.3139 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | main | 1/1 | ok | 0.3683 | 0.9931 | (-32.5,3.5) moderate | 0.3139 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | pipeline | 1/1 | ok | 0.3070 | 0.9947 | (-1,0) moderate | 0.2372 | 0.9962 | 255 | 0.0025 | 0.0020 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.3812 | 0.9936 | (0,3.5) | 0.2591 | 0.9960 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | main | 1/1 | ok | 0.3812 | 0.9936 | (0,3.5) | 0.2591 | 0.9960 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | pipeline | 1/1 | ok | 0.2915 | 0.9950 | (16.5,0) weak | 0.2870 | 0.9953 | 255 | 0.0025 | 0.0018 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | de1020c | 1/1 | ok | 0.3681 | 0.9931 | (-32.5,3.5) moderate | 0.3137 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | main | 1/1 | ok | 0.3681 | 0.9931 | (-32.5,3.5) moderate | 0.3137 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | pipeline | 1/1 | ok | 0.3074 | 0.9947 | (-1,0) moderate | 0.2389 | 0.9961 | 255 | 0.0025 | 0.0020 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.3823 | 0.9934 | (0,3.5) | 0.2545 | 0.9959 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | ok | 0.3823 | 0.9934 | (0,3.5) | 0.2545 | 0.9959 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | pipeline | 1/1 | ok | 0.3054 | 0.9947 | (0,0) | 0.3054 | 0.9947 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | de1020c | 1/1 | ok | 0.3684 | 0.9931 | (-32.5,3.5) moderate | 0.3141 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | main | 1/1 | ok | 0.3684 | 0.9931 | (-32.5,3.5) moderate | 0.3141 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | pipeline | 1/1 | ok | 0.3071 | 0.9947 | (-1,0) moderate | 0.2372 | 0.9962 | 255 | 0.0025 | 0.0020 | -/-/- | - | - | - | - | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.2575 | 0.9950 | (0,0) | 0.2575 | 0.9950 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | ok | 0.2702 | 0.9946 | (0,0) | 0.2702 | 0.9946 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex | pipeline | 1/1 | ok | 0.3301 | 0.9939 | (0,0) | 0.3301 | 0.9939 | 255 | 0.0028 | 0.0022 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | de1020c | 1/1 | ok | 0.3485 | 0.9931 | (0,0) | 0.3485 | 0.9931 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | main | 1/1 | ok | 0.3613 | 0.9927 | (0,0) | 0.3613 | 0.9927 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | pipeline | 1/1 | ok | 0.3145 | 0.9941 | (-0.5,0) weak | 0.3073 | 0.9942 | 255 | 0.0027 | 0.0022 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.2593 | 0.9950 | (0,0) | 0.2593 | 0.9950 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex | main | 1/1 | ok | 0.2720 | 0.9946 | (0,0) | 0.2720 | 0.9946 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex | pipeline | 1/1 | ok | 0.3249 | 0.9941 | (0,0) | 0.3249 | 0.9941 | 255 | 0.0028 | 0.0021 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex-lm | de1020c | 1/1 | ok | 0.3516 | 0.9931 | (0,0) | 0.3516 | 0.9931 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex-lm | main | 1/1 | ok | 0.3644 | 0.9927 | (0,0) | 0.3644 | 0.9927 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex-lm | pipeline | 1/1 | ok | 0.3141 | 0.9941 | (-0.5,0) weak | 0.3060 | 0.9942 | 255 | 0.0028 | 0.0022 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.2573 | 0.9950 | (0,0) | 0.2573 | 0.9950 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | ok | 0.2701 | 0.9946 | (0,0) | 0.2701 | 0.9946 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | pipeline | 1/1 | ok | 0.3298 | 0.9939 | (0,0) | 0.3298 | 0.9939 | 255 | 0.0028 | 0.0022 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | de1020c | 1/1 | ok | 0.3485 | 0.9931 | (0,0) | 0.3485 | 0.9931 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | main | 1/1 | ok | 0.3613 | 0.9927 | (0,0) | 0.3613 | 0.9927 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | pipeline | 1/1 | ok | 0.3146 | 0.9941 | (-0.5,0) weak | 0.3074 | 0.9942 | 255 | 0.0027 | 0.0022 | -/-/- | - | - | - | - | 3/1 | - |
| 08-two-page | lualatex | de1020c | 3/3 | ok | 26.2189 | 0.5659 | (0,0); (-3,6) moderate; (-3,34.5) moderate | 24.6331 | 0.5959 | 255 | 0.1880 | 0.1574 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex | main | 3/3 | ok | 26.2189 | 0.5659 | (0,0); (-3,6) moderate; (-3,34.5) moderate | 24.6331 | 0.5959 | 255 | 0.1880 | 0.1574 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex | pipeline | 3/3 | ok | 21.1582 | 0.6699 | (0,0); (0,0); (1,0) weak | 21.1037 | 0.6709 | 255 | 0.1619 | 0.1323 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | de1020c | 3/3 | ok | 23.7956 | 0.5635 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7517 | 0.5869 | 255 | 0.1801 | 0.1495 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | main | 3/3 | ok | 23.7956 | 0.5635 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7517 | 0.5869 | 255 | 0.1801 | 0.1495 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | pipeline | 3/3 | ok | 13.0043 | 0.8184 | (0,0); (0,0); (0,0) | 13.0043 | 0.8184 | 255 | 0.1280 | 0.0963 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/3 | ok | 26.1972 | 0.5669 | (0,-14.5) weak; (0,6) moderate; (0,34.5) moderate | 24.5512 | 0.5981 | 255 | 0.1875 | 0.1571 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | main | 3/3 | ok | 26.1972 | 0.5669 | (0,-14.5) weak; (0,6) moderate; (0,34.5) moderate | 24.5512 | 0.5981 | 255 | 0.1875 | 0.1571 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | pipeline | 3/3 | ok | 21.1714 | 0.6711 | (-1.5,0) weak; (1.5,0) weak; (-1.5,0) weak | 21.0218 | 0.6737 | 255 | 0.1614 | 0.1321 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | de1020c | 3/3 | ok | 23.7951 | 0.5635 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7523 | 0.5869 | 255 | 0.1800 | 0.1496 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | main | 3/3 | ok | 23.7951 | 0.5635 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7523 | 0.5869 | 255 | 0.1800 | 0.1496 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | pipeline | 3/3 | ok | 13.0322 | 0.8180 | (0,0); (0,0); (0,0) | 13.0322 | 0.8180 | 255 | 0.1281 | 0.0966 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | de1020c | 3/3 | ok | 26.2441 | 0.5658 | (-3,-14.5) weak; (-3,6) moderate; (-3,34.5) moderate | 24.6604 | 0.5954 | 255 | 0.1881 | 0.1577 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | main | 3/3 | ok | 26.2441 | 0.5658 | (-3,-14.5) weak; (-3,6) moderate; (-3,34.5) moderate | 24.6604 | 0.5954 | 255 | 0.1881 | 0.1577 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | pipeline | 3/3 | ok | 21.1760 | 0.6695 | (1.5,0) weak; (0,0); (1,0) weak | 21.1126 | 0.6705 | 255 | 0.1620 | 0.1326 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | de1020c | 3/3 | ok | 23.7957 | 0.5634 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7513 | 0.5869 | 255 | 0.1800 | 0.1495 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | main | 3/3 | ok | 23.7957 | 0.5634 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7513 | 0.5869 | 255 | 0.1800 | 0.1495 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | pipeline | 3/3 | ok | 13.0077 | 0.8183 | (0,0); (0,0); (0,0) | 13.0077 | 0.8183 | 255 | 0.1280 | 0.0964 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.4844 | 0.9504 | (0,38.5) | 1.7906 | 0.9701 | 255 | 0.0174 | 0.0146 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | ok | 2.5141 | 0.9497 | (0,38.5) | 1.8130 | 0.9695 | 255 | 0.0176 | 0.0148 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | pipeline | 1/1 | ok | 2.1774 | 0.9628 | (0.5,-17.5) weak | 2.1060 | 0.9643 | 255 | 0.0156 | 0.0131 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | de1020c | 1/1 | ok | 2.2538 | 0.9494 | (-0.5,39) moderate | 1.9068 | 0.9641 | 255 | 0.0169 | 0.0140 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | main | 1/1 | ok | 2.2692 | 0.9490 | (-0.5,39) moderate | 1.9292 | 0.9634 | 255 | 0.0170 | 0.0141 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | pipeline | 1/1 | ok | 1.8764 | 0.9648 | (0,-17) moderate | 1.5943 | 0.9706 | 255 | 0.0147 | 0.0121 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.4854 | 0.9504 | (-0.5,38.5) moderate | 1.8777 | 0.9688 | 255 | 0.0174 | 0.0146 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | main | 1/1 | ok | 2.5141 | 0.9497 | (-0.5,38.5) moderate | 1.9001 | 0.9681 | 255 | 0.0175 | 0.0148 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | pipeline | 1/1 | ok | 2.1814 | 0.9627 | (1,-17) weak | 2.0798 | 0.9647 | 255 | 0.0156 | 0.0131 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | de1020c | 1/1 | ok | 2.2522 | 0.9480 | (-0.5,39.5) moderate | 1.9062 | 0.9622 | 255 | 0.0167 | 0.0139 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | main | 1/1 | ok | 2.2675 | 0.9475 | (-0.5,39.5) moderate | 1.9286 | 0.9615 | 255 | 0.0169 | 0.0140 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | pipeline | 1/1 | ok | 1.8627 | 0.9635 | (0,-16.5) moderate | 1.6137 | 0.9700 | 255 | 0.0145 | 0.0119 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.4838 | 0.9503 | (-0.5,38.5) | 1.8423 | 0.9692 | 255 | 0.0174 | 0.0146 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | ok | 2.5135 | 0.9496 | (-0.5,38.5) | 1.8647 | 0.9686 | 255 | 0.0176 | 0.0148 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | pipeline | 1/1 | ok | 2.1728 | 0.9628 | (1,-17.5) weak | 2.0852 | 0.9647 | 255 | 0.0156 | 0.0131 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | de1020c | 1/1 | ok | 2.2538 | 0.9494 | (-0.5,39) moderate | 1.9067 | 0.9641 | 255 | 0.0169 | 0.0140 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | main | 1/1 | ok | 2.2692 | 0.9490 | (-0.5,39) moderate | 1.9291 | 0.9634 | 255 | 0.0170 | 0.0141 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | pipeline | 1/1 | ok | 1.8771 | 0.9648 | (0,-17) moderate | 1.5952 | 0.9706 | 255 | 0.0147 | 0.0121 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | lualatex | de1020c | 1/1 | ok | 3.3162 | 0.9491 | (0,0) | 3.3162 | 0.9491 | 255 | 0.0261 | 0.0212 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | main | 1/1 | ok | 3.3162 | 0.9491 | (0,0) | 3.3162 | 0.9491 | 255 | 0.0261 | 0.0212 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | pipeline | 1/1 | recovered | 3.3969 | 0.9471 | (0,0) | 3.3969 | 0.9471 | 255 | 0.0265 | 0.0214 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex-lm | de1020c | 1/1 | ok | 3.2405 | 0.9447 | (0,0) | 3.2405 | 0.9447 | 255 | 0.0259 | 0.0211 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | main | 1/1 | ok | 3.2405 | 0.9447 | (0,0) | 3.2405 | 0.9447 | 255 | 0.0259 | 0.0211 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 2.3128 | 0.9658 | (-0.5,0) moderate | 2.1851 | 0.9680 | 255 | 0.0218 | 0.0166 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | pdflatex | de1020c | 1/1 | ok | 3.3644 | 0.9486 | (0,0) | 3.3644 | 0.9486 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | pdflatex | main | 1/1 | ok | 3.3644 | 0.9486 | (0,0) | 3.3644 | 0.9486 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | pdflatex | pipeline | 1/1 | recovered | 3.3533 | 0.9479 | (0,0) | 3.3533 | 0.9479 | 255 | 0.0263 | 0.0211 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 3.2558 | 0.9444 | (0,0) | 3.2558 | 0.9444 | 255 | 0.0260 | 0.0212 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | pdflatex-lm | main | 1/1 | ok | 3.2558 | 0.9444 | (0,0) | 3.2558 | 0.9444 | 255 | 0.0260 | 0.0212 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 2.3799 | 0.9644 | (-0.5,0) moderate | 2.2159 | 0.9672 | 255 | 0.0221 | 0.0169 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | xelatex | de1020c | 1/1 | ok | 3.3626 | 0.9484 | (0,0) | 3.3626 | 0.9484 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | xelatex | main | 1/1 | ok | 3.3626 | 0.9484 | (0,0) | 3.3626 | 0.9484 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | xelatex | pipeline | 1/1 | recovered | 3.4020 | 0.9470 | (0,0) | 3.4020 | 0.9470 | 255 | 0.0265 | 0.0214 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | xelatex-lm | de1020c | 1/1 | ok | 3.2406 | 0.9446 | (0,0) | 3.2406 | 0.9446 | 255 | 0.0259 | 0.0211 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | main | 1/1 | ok | 3.2406 | 0.9446 | (0,0) | 3.2406 | 0.9446 | 255 | 0.0259 | 0.0211 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 2.3134 | 0.9658 | (-0.5,0) moderate | 2.1851 | 0.9680 | 255 | 0.0218 | 0.0166 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | de1020c | 1/1 | recovered | 1.4652 | 0.9696 | (18,-49) weak | 1.4604 | 0.9709 | 255 | 0.0104 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | main | 1/1 | ok | 1.4651 | 0.9700 | (-21,-8) moderate | 1.2181 | 0.9785 | 255 | 0.0105 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | pipeline | 1/1 | ok | 1.4784 | 0.9703 | (-16.5,-20) moderate | 1.3615 | 0.9753 | 255 | 0.0104 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (-2,-49) weak | 1.3068 | 0.9711 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | main | 1/1 | ok | 1.3732 | 0.9689 | (-29,-8) moderate | 1.2485 | 0.9756 | 255 | 0.0101 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | pipeline | 1/1 | ok | 1.3303 | 0.9702 | (-17,-20) moderate | 1.1997 | 0.9761 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | de1020c | 1/1 | recovered | 1.4655 | 0.9698 | (19,-49) weak | 1.4558 | 0.9711 | 255 | 0.0104 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | main | 1/1 | ok | 1.4715 | 0.9702 | (-20,-8) moderate | 1.1981 | 0.9790 | 255 | 0.0105 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | pipeline | 1/1 | ok | 1.4478 | 0.9709 | (-16.5,-20) moderate | 1.3597 | 0.9754 | 255 | 0.0103 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | de1020c | 1/1 | recovered | 1.3674 | 0.9687 | (23.5,-49) weak | 1.3215 | 0.9706 | 255 | 0.0101 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | main | 1/1 | ok | 1.3674 | 0.9690 | (-29,-8) moderate | 1.2361 | 0.9758 | 255 | 0.0102 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | pipeline | 1/1 | ok | 1.3363 | 0.9701 | (-17,-20) moderate | 1.2000 | 0.9760 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | de1020c | 1/1 | recovered | 1.4648 | 0.9696 | (18,-49) weak | 1.4599 | 0.9709 | 255 | 0.0104 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | main | 1/1 | ok | 1.4647 | 0.9700 | (-21,-8) moderate | 1.2168 | 0.9785 | 255 | 0.0105 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | pipeline | 1/1 | ok | 1.4782 | 0.9703 | (-16.5,-20) moderate | 1.3611 | 0.9753 | 255 | 0.0104 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (1,-49) weak | 1.3138 | 0.9708 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | main | 1/1 | ok | 1.3731 | 0.9689 | (-29,-8) moderate | 1.2487 | 0.9756 | 255 | 0.0101 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | pipeline | 1/1 | ok | 1.3303 | 0.9702 | (-17.5,-20) moderate | 1.1888 | 0.9763 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | de1020c | 1/1 | ok | 15.4036 | 0.7397 | (-3,0) weak | 15.3867 | 0.7398 | 255 | 0.1111 | 0.0927 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | main | 1/1 | ok | 15.4036 | 0.7397 | (-3,0) weak | 15.3867 | 0.7398 | 255 | 0.1111 | 0.0927 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | pipeline | 1/1 | ok | 12.9693 | 0.7908 | (0,14.5) weak | 12.9445 | 0.7924 | 255 | 0.0988 | 0.0805 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | de1020c | 1/1 | ok | 13.7975 | 0.7504 | (-8.5,2.5) weak | 13.7722 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | main | 1/1 | ok | 13.7975 | 0.7504 | (-8.5,2.5) weak | 13.7722 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | pipeline | 1/1 | ok | 7.8075 | 0.8916 | (0,0) | 7.8075 | 0.8916 | 255 | 0.0766 | 0.0577 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | de1020c | 1/1 | ok | 15.3703 | 0.7416 | (-3,0) weak | 15.3659 | 0.7415 | 255 | 0.1107 | 0.0925 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | main | 1/1 | ok | 15.3703 | 0.7416 | (-3,0) weak | 15.3659 | 0.7415 | 255 | 0.1107 | 0.0925 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | pipeline | 1/1 | ok | 13.0307 | 0.7911 | (1,14.5) weak | 12.9210 | 0.7936 | 255 | 0.0986 | 0.0807 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | de1020c | 1/1 | ok | 13.7973 | 0.7504 | (-8.5,2.5) weak | 13.7711 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | main | 1/1 | ok | 13.7973 | 0.7504 | (-8.5,2.5) weak | 13.7711 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | pipeline | 1/1 | ok | 7.8248 | 0.8913 | (0,0) | 7.8248 | 0.8913 | 255 | 0.0767 | 0.0579 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | de1020c | 1/1 | ok | 15.3817 | 0.7405 | (0,0) | 15.3817 | 0.7405 | 255 | 0.1110 | 0.0928 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | main | 1/1 | ok | 15.3817 | 0.7405 | (0,0) | 15.3817 | 0.7405 | 255 | 0.1110 | 0.0928 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | pipeline | 1/1 | ok | 13.0500 | 0.7899 | (0.5,14.5) weak | 12.9803 | 0.7921 | 255 | 0.0990 | 0.0810 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | de1020c | 1/1 | ok | 13.7974 | 0.7504 | (-8.5,2.5) weak | 13.7718 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | main | 1/1 | ok | 13.7974 | 0.7504 | (-8.5,2.5) weak | 13.7718 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | pipeline | 1/1 | ok | 7.8076 | 0.8916 | (0,0) | 7.8076 | 0.8916 | 255 | 0.0766 | 0.0577 | -/-/- | - | - | - | - | 0/0 | - |
| 13-math-display-rich | lualatex | de1020c | 1/1 | recovered | 0.4665 | 0.9904 | (0,3.5) moderate | 0.4400 | 0.9913 | 255 | 0.0040 | 0.0031 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex | main | 1/1 | recovered | 0.4792 | 0.9900 | (0,3.5) moderate | 0.4528 | 0.9910 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex | pipeline | 1/1 | recovered | 0.6067 | 0.9871 | (1.5,1.5) weak | 0.5909 | 0.9877 | 255 | 0.0047 | 0.0038 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex-lm | de1020c | 1/1 | recovered | 0.5457 | 0.9888 | (0,0) | 0.5457 | 0.9888 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | lualatex-lm | main | 1/1 | recovered | 0.5585 | 0.9885 | (0,0) | 0.5585 | 0.9885 | 255 | 0.0044 | 0.0036 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | lualatex-lm | pipeline | 1/1 | recovered | 0.5829 | 0.9872 | (3.5,1.5) moderate | 0.5310 | 0.9888 | 255 | 0.0046 | 0.0038 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | pdflatex | de1020c | 1/1 | recovered | 0.4914 | 0.9901 | (1,3.5) weak | 0.4902 | 0.9905 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | pdflatex | main | 1/1 | recovered | 0.5042 | 0.9897 | (1,3.5) weak | 0.5030 | 0.9902 | 255 | 0.0042 | 0.0033 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | pdflatex | pipeline | 1/1 | recovered | 0.6092 | 0.9871 | (4,1.5) moderate | 0.5551 | 0.9883 | 255 | 0.0047 | 0.0038 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | pdflatex-lm | de1020c | 1/1 | recovered | 0.5476 | 0.9889 | (-2,4) weak | 0.5407 | 0.9892 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | pdflatex-lm | main | 1/1 | recovered | 0.5603 | 0.9885 | (-2,4) weak | 0.5535 | 0.9889 | 255 | 0.0044 | 0.0036 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | pdflatex-lm | pipeline | 1/1 | recovered | 0.5846 | 0.9872 | (3.5,1.5) moderate | 0.5419 | 0.9886 | 255 | 0.0046 | 0.0038 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | xelatex | de1020c | 1/1 | recovered | 0.4850 | 0.9901 | (0,0) | 0.4850 | 0.9901 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex | main | 1/1 | recovered | 0.4978 | 0.9897 | (0,0) | 0.4978 | 0.9897 | 255 | 0.0042 | 0.0033 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex | pipeline | 1/1 | recovered | 0.6102 | 0.9870 | (1.5,1.5) moderate | 0.5735 | 0.9879 | 255 | 0.0047 | 0.0038 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex-lm | de1020c | 1/1 | recovered | 0.5459 | 0.9888 | (0,0) | 0.5459 | 0.9888 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | xelatex-lm | main | 1/1 | recovered | 0.5587 | 0.9885 | (0,0) | 0.5587 | 0.9885 | 255 | 0.0044 | 0.0036 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | xelatex-lm | pipeline | 1/1 | recovered | 0.5831 | 0.9872 | (3.5,1.5) moderate | 0.5311 | 0.9888 | 255 | 0.0046 | 0.0038 | -/-/- | - | - | - | - | 4/2 | - |
| 14-math-inline-dense | lualatex | de1020c | 1/1 | recovered | 1.3153 | 0.9706 | (-34.5,13.5) | 0.9661 | 0.9780 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex | main | 1/1 | recovered | 1.3153 | 0.9706 | (-34.5,13.5) | 0.9661 | 0.9780 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex | pipeline | 1/1 | recovered | 1.0666 | 0.9799 | (0,0) | 1.0666 | 0.9799 | 255 | 0.0087 | 0.0069 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | lualatex-lm | de1020c | 1/1 | recovered | 1.2497 | 0.9701 | (2.5,13.5) moderate | 1.1687 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | main | 1/1 | recovered | 1.2497 | 0.9701 | (2.5,13.5) moderate | 1.1687 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | pipeline | 1/1 | recovered | 1.0041 | 0.9807 | (-0.5,1.5) moderate | 0.8705 | 0.9836 | 255 | 0.0084 | 0.0068 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | pdflatex | de1020c | 1/1 | recovered | 1.3096 | 0.9709 | (-34,13.5) | 0.9598 | 0.9781 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | pdflatex | main | 1/1 | recovered | 1.3096 | 0.9709 | (-34,13.5) | 0.9598 | 0.9781 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | pdflatex | pipeline | 1/1 | recovered | 1.0580 | 0.9799 | (0,0) | 1.0580 | 0.9799 | 255 | 0.0086 | 0.0068 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | pdflatex-lm | de1020c | 1/1 | recovered | 1.2493 | 0.9701 | (2.5,13.5) moderate | 1.1681 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | pdflatex-lm | main | 1/1 | recovered | 1.2493 | 0.9701 | (2.5,13.5) moderate | 1.1681 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | pdflatex-lm | pipeline | 1/1 | recovered | 1.0054 | 0.9807 | (-0.5,1.5) moderate | 0.8682 | 0.9836 | 255 | 0.0084 | 0.0068 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | xelatex | de1020c | 1/1 | recovered | 1.3153 | 0.9706 | (-34.5,13.5) | 0.9632 | 0.9781 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex | main | 1/1 | recovered | 1.3153 | 0.9706 | (-34.5,13.5) | 0.9632 | 0.9781 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex | pipeline | 1/1 | recovered | 1.0667 | 0.9799 | (0,0) | 1.0667 | 0.9799 | 255 | 0.0087 | 0.0069 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | xelatex-lm | de1020c | 1/1 | recovered | 1.2496 | 0.9701 | (2.5,13.5) moderate | 1.1687 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | main | 1/1 | recovered | 1.2496 | 0.9701 | (2.5,13.5) moderate | 1.1687 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | pipeline | 1/1 | recovered | 1.0041 | 0.9807 | (-0.5,1.5) moderate | 0.8704 | 0.9836 | 255 | 0.0084 | 0.0068 | -/-/- | - | - | - | - | 2/2 | - |
| 15-three-page-sections | lualatex | de1020c | 3/3 | recovered | 26.4238 | 0.5637 | (0.5,20) moderate; (0.5,19) weak; (0.5,-24.5) moderate | 25.2152 | 0.5876 | 255 | 0.1893 | 0.1584 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex | main | 3/3 | recovered | 26.4375 | 0.5634 | (0.5,20) moderate; (0.5,19) weak; (0.5,-24.5) moderate | 25.2267 | 0.5874 | 255 | 0.1894 | 0.1585 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex | pipeline | 3/3 | recovered | 23.9031 | 0.6050 | (0,14) weak; (0.5,-26.5) moderate; (0.5,-55.5) moderate | 21.6044 | 0.6601 | 255 | 0.1767 | 0.1466 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex-lm | de1020c | 3/3 | recovered | 24.2613 | 0.5495 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0400 | 0.5821 | 255 | 0.1832 | 0.1519 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | lualatex-lm | main | 3/3 | recovered | 24.2719 | 0.5494 | (0.5,22.5) moderate; (0,-24) weak; (0.5,-24) weak | 23.0396 | 0.5821 | 255 | 0.1832 | 0.1520 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | lualatex-lm | pipeline | 3/3 | recovered | 19.9055 | 0.6420 | (0,0); (0,-26.5) moderate; (0,-26.5) moderate | 18.3487 | 0.6884 | 255 | 0.1619 | 0.1304 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex | de1020c | 3/3 | recovered | 26.6983 | 0.5543 | (0.5,36.5) moderate; (0.5,-24.5) weak; (0,-24.5) weak | 25.1309 | 0.5916 | 255 | 0.1901 | 0.1598 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex | main | 3/3 | recovered | 26.7124 | 0.5541 | (0.5,36.5) moderate; (0.5,-24.5) weak; (0,-24.5) weak | 25.1424 | 0.5914 | 255 | 0.1902 | 0.1599 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex | pipeline | 3/3 | recovered | 24.0963 | 0.5970 | (1,-0.5) moderate; (0.5,-55.5) moderate; (0.5,-12.5) moderate | 21.8322 | 0.6500 | 255 | 0.1773 | 0.1476 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex-lm | de1020c | 3/3 | recovered | 24.2616 | 0.5495 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0365 | 0.5822 | 255 | 0.1832 | 0.1521 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex-lm | main | 3/3 | recovered | 24.2722 | 0.5493 | (0.5,22.5) moderate; (0,-24) weak; (0.5,-24) weak | 23.0367 | 0.5823 | 255 | 0.1833 | 0.1521 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex-lm | pipeline | 3/3 | recovered | 19.9171 | 0.6417 | (0,0); (0,-26.5) moderate; (0,-26.5) moderate | 18.3609 | 0.6880 | 255 | 0.1620 | 0.1306 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex | de1020c | 3/3 | recovered | 26.4204 | 0.5639 | (-3,20) moderate; (-3,19) weak; (-3,-24.5) moderate | 25.1894 | 0.5880 | 255 | 0.1893 | 0.1587 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex | main | 3/3 | recovered | 26.4341 | 0.5636 | (-3,20) moderate; (-3,19) weak; (-3,-24.5) moderate | 25.2009 | 0.5877 | 255 | 0.1894 | 0.1588 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex | pipeline | 3/3 | recovered | 23.9340 | 0.6046 | (0,14) weak; (1,-26.5) moderate; (1,-55.5) moderate | 21.7072 | 0.6586 | 255 | 0.1768 | 0.1470 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex-lm | de1020c | 3/3 | recovered | 24.2611 | 0.5495 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0399 | 0.5821 | 255 | 0.1831 | 0.1519 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex-lm | main | 3/3 | recovered | 24.2717 | 0.5493 | (0.5,22.5) moderate; (3.5,-24) weak; (0.5,-24) weak | 23.0722 | 0.5811 | 255 | 0.1832 | 0.1520 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex-lm | pipeline | 3/3 | recovered | 19.9070 | 0.6419 | (0,0); (0,-26.5) moderate; (0,-26.5) moderate | 18.3502 | 0.6883 | 255 | 0.1619 | 0.1304 | -/-/- | - | - | - | - | 3/0 | - |
| 16-heading-page-break | lualatex | de1020c | 2/2 | ok | 20.5841 | 0.6613 | (0,0); (0,20.5) moderate | 20.0997 | 0.6688 | 255 | 0.1484 | 0.1240 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex | main | 2/2 | ok | 20.5889 | 0.6612 | (0,0); (0,20.5) moderate | 20.1043 | 0.6687 | 255 | 0.1484 | 0.1240 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex | pipeline | 2/2 | ok | 17.5103 | 0.7217 | (0,0); (-1,57.5) weak | 17.3227 | 0.7266 | 255 | 0.1328 | 0.1087 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex-lm | de1020c | 2/2 | ok | 19.0167 | 0.6509 | (-1,0) weak; (-1,20) moderate | 18.0483 | 0.6755 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | lualatex-lm | main | 2/2 | ok | 19.0237 | 0.6509 | (-1,0) weak; (-1,20) moderate | 18.0517 | 0.6755 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | lualatex-lm | pipeline | 2/2 | ok | 10.7803 | 0.8473 | (0,0); (0,0) | 10.7803 | 0.8473 | 255 | 0.1046 | 0.0792 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex | de1020c | 2/2 | ok | 20.5976 | 0.6621 | (0,-14.5) weak; (0,20.5) moderate | 20.0459 | 0.6704 | 255 | 0.1481 | 0.1240 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex | main | 2/2 | ok | 20.6024 | 0.6620 | (0,-14.5) weak; (0,20.5) moderate | 20.0509 | 0.6704 | 255 | 0.1481 | 0.1240 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex | pipeline | 2/2 | ok | 17.4970 | 0.7234 | (-1.5,0) weak; (-1.5,57.5) moderate | 17.1857 | 0.7304 | 255 | 0.1324 | 0.1086 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex-lm | de1020c | 2/2 | ok | 19.0160 | 0.6507 | (-1,0) weak; (-1,20) moderate | 18.0489 | 0.6754 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex-lm | main | 2/2 | ok | 19.0230 | 0.6507 | (-1,0) weak; (-1,20) moderate | 18.0522 | 0.6754 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex-lm | pipeline | 2/2 | ok | 10.8088 | 0.8466 | (0,0); (0,0) | 10.8088 | 0.8466 | 255 | 0.1047 | 0.0794 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex | de1020c | 2/2 | ok | 20.6109 | 0.6611 | (-3,-14.5) weak; (0.5,20.5) moderate | 20.1127 | 0.6683 | 255 | 0.1485 | 0.1243 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex | main | 2/2 | ok | 20.6157 | 0.6611 | (-3,-14.5) weak; (0,20.5) moderate | 20.1384 | 0.6680 | 255 | 0.1485 | 0.1243 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex | pipeline | 2/2 | ok | 17.5412 | 0.7211 | (1.5,0) weak; (-1,57.5) weak | 17.3041 | 0.7263 | 255 | 0.1330 | 0.1091 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex-lm | de1020c | 2/2 | ok | 19.0167 | 0.6509 | (-1,0) weak; (-1,20) moderate | 18.0480 | 0.6754 | 255 | 0.1438 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex-lm | main | 2/2 | ok | 19.0238 | 0.6508 | (-1,0) weak; (-1,20) moderate | 18.0514 | 0.6754 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex-lm | pipeline | 2/2 | ok | 10.7830 | 0.8472 | (0,0); (0,0) | 10.7830 | 0.8472 | 255 | 0.1046 | 0.0793 | -/-/- | - | - | - | - | 1/0 | - |
| 17-apostrophes | lualatex | de1020c | 1/1 | ok | 0.8503 | 0.9875 | (-1.5,0) weak | 0.8116 | 0.9882 | 255 | 0.0068 | 0.0053 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex | main | 1/1 | ok | 0.8503 | 0.9875 | (-1.5,0) weak | 0.8116 | 0.9882 | 255 | 0.0068 | 0.0053 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex | pipeline | 1/1 | ok | 0.9451 | 0.9851 | (53.5,0) moderate | 0.8876 | 0.9859 | 255 | 0.0073 | 0.0058 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | de1020c | 1/1 | ok | 0.8624 | 0.9848 | (0,0) | 0.8624 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | main | 1/1 | ok | 0.8624 | 0.9848 | (0,0) | 0.8624 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | pipeline | 1/1 | ok | 0.6767 | 0.9896 | (-0.5,0) moderate | 0.6335 | 0.9899 | 255 | 0.0061 | 0.0047 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | de1020c | 1/1 | ok | 0.8064 | 0.9887 | (0,0) | 0.8064 | 0.9887 | 255 | 0.0066 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | main | 1/1 | ok | 0.8064 | 0.9887 | (0,0) | 0.8064 | 0.9887 | 255 | 0.0066 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | pipeline | 1/1 | ok | 0.9301 | 0.9857 | (0,0) | 0.9301 | 0.9857 | 255 | 0.0072 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | de1020c | 1/1 | ok | 0.8626 | 0.9848 | (0,0) | 0.8626 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | main | 1/1 | ok | 0.8626 | 0.9848 | (0,0) | 0.8626 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | pipeline | 1/1 | ok | 0.6761 | 0.9896 | (-0.5,0) moderate | 0.6335 | 0.9899 | 255 | 0.0061 | 0.0047 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | de1020c | 1/1 | ok | 0.7982 | 0.9883 | (0,0) | 0.7982 | 0.9883 | 255 | 0.0066 | 0.0051 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | main | 1/1 | ok | 0.7982 | 0.9883 | (0,0) | 0.7982 | 0.9883 | 255 | 0.0066 | 0.0051 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | pipeline | 1/1 | ok | 0.9357 | 0.9853 | (53.5,0) weak | 0.9057 | 0.9856 | 255 | 0.0073 | 0.0058 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | de1020c | 1/1 | ok | 0.8621 | 0.9848 | (0,0) | 0.8621 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | main | 1/1 | ok | 0.8621 | 0.9848 | (0,0) | 0.8621 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | pipeline | 1/1 | ok | 0.6766 | 0.9896 | (-0.5,0) moderate | 0.6330 | 0.9899 | 255 | 0.0061 | 0.0047 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | de1020c | 1/1 | ok | 1.3878 | 0.9788 | (0,0) | 1.3878 | 0.9788 | 255 | 0.0109 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | main | 1/1 | ok | 1.3878 | 0.9788 | (0,0) | 1.3878 | 0.9788 | 255 | 0.0109 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | pipeline | 1/1 | ok | 1.3808 | 0.9786 | (0,0) | 1.3808 | 0.9786 | 255 | 0.0110 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | de1020c | 1/1 | ok | 1.3042 | 0.9789 | (0,0) | 1.3042 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | main | 1/1 | ok | 1.3042 | 0.9789 | (0,0) | 1.3042 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | pipeline | 1/1 | ok | 0.9109 | 0.9874 | (0,0) | 0.9109 | 0.9874 | 255 | 0.0089 | 0.0067 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | de1020c | 1/1 | ok | 1.2717 | 0.9812 | (0,0) | 1.2717 | 0.9812 | 255 | 0.0104 | 0.0080 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | main | 1/1 | ok | 1.2717 | 0.9812 | (0,0) | 1.2717 | 0.9812 | 255 | 0.0104 | 0.0080 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | pipeline | 1/1 | ok | 1.3535 | 0.9796 | (0,0) | 1.3535 | 0.9796 | 255 | 0.0107 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | de1020c | 1/1 | ok | 1.3056 | 0.9789 | (0,0) | 1.3056 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | main | 1/1 | ok | 1.3056 | 0.9789 | (0,0) | 1.3056 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | pipeline | 1/1 | ok | 0.9120 | 0.9874 | (0,0) | 0.9120 | 0.9874 | 255 | 0.0089 | 0.0067 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | de1020c | 1/1 | ok | 1.3346 | 0.9799 | (0,0) | 1.3346 | 0.9799 | 255 | 0.0107 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | main | 1/1 | ok | 1.3346 | 0.9799 | (0,0) | 1.3346 | 0.9799 | 255 | 0.0107 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | pipeline | 1/1 | ok | 1.3721 | 0.9788 | (0,0) | 1.3721 | 0.9788 | 255 | 0.0110 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | de1020c | 1/1 | ok | 1.3041 | 0.9789 | (0,0) | 1.3041 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | main | 1/1 | ok | 1.3041 | 0.9789 | (0,0) | 1.3041 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | pipeline | 1/1 | ok | 0.9112 | 0.9874 | (0,0) | 0.9112 | 0.9874 | 255 | 0.0089 | 0.0067 | -/-/- | - | - | - | - | 0/0 | - |

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933593, 504, 382, 398, 316, 319, 289, 222, 238, 244, 219, 320, 264, 216, 279, 1013]`; ink px ref/ours 2442/2420 (ratio 0.991); SSIM blocks <0.9: 218/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [7.89, 0.02] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.381, differing 0.00311, SSIM₈ 0.9944 (raw 0.3845, 0.003114, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9909→0.9911 / 0.6146→0.6088; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 24.94 dy 0.41; `single` dx 23.9 dy 0.41; `a` dx 22.44 dy 0.41

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934693, 513, 482, 413, 349, 354, 303, 222, 247, 229, 183, 134, 119, 107, 100, 368]`; ink px ref/ours 1890/2420 (ratio 1.2804); SSIM blocks <0.9: 183/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.68, -0.12] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2368, differing 0.002561, SSIM₈ 0.9967 (raw 0.2368, 0.002561, 0.9967)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9947→0.9947 / 0.3784→0.3784; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `a` dx 0.06 dy -0.36; `single` dx 0.06 dy -0.36; `line.` dx 0.06 dy -0.36

### 01-plain-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935388, 716, 424, 359, 290, 196, 196, 126, 136, 124, 122, 86, 78, 86, 81, 408]`; ink px ref/ours 2408/2421 (ratio 1.0054); SSIM blocks <0.9: 119/30294; [overlay](images/01-plain-paragraph/pdflatex-de1020c-export-p1-overlay.png) (74851 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (69274 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.1, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1873, differing 0.002361, SSIM₈ 0.9975 (raw 0.1873, 0.002361, 0.9975)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9961→0.9961 / 0.2993→0.2993; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 1.15 dy 0.46; `single` dx 1.03 dy 0.46; `a` dx 1.0 dy 0.46

### 01-plain-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935388, 716, 424, 359, 290, 196, 196, 126, 136, 124, 122, 86, 78, 86, 81, 408]`; ink px ref/ours 2408/2421 (ratio 1.0054); SSIM blocks <0.9: 119/30294; [overlay](images/01-plain-paragraph/pdflatex-main-export-p1-overlay.png) (74582 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-main-export-p1-heatmap.png) (69010 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.1, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1873, differing 0.002361, SSIM₈ 0.9975 (raw 0.1873, 0.002361, 0.9975)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9961→0.9961 / 0.2993→0.2993; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 1.15 dy 0.46; `single` dx 1.03 dy 0.46; `a` dx 1.0 dy 0.46

### 01-plain-paragraph — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933678, 551, 315, 257, 292, 305, 273, 227, 249, 246, 233, 262, 242, 248, 295, 1143]`; ink px ref/ours 2408/2420 (ratio 1.005); SSIM blocks <0.9: 214/30294; [overlay](images/01-plain-paragraph/pdflatex-pipeline-export-p1-overlay.png) (76472 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-pipeline-export-p1-heatmap.png) (73714 B, ÷1)
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [8.74, 0.02] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3788, differing 0.003059, SSIM₈ 0.9947 (raw 0.3946, 0.0031, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9916 / 0.6306→0.6049; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 25.96 dy 0.41; `single` dx 24.93 dy 0.41; `a` dx 23.46 dy 0.41

### 01-plain-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933593, 464, 437, 341, 371, 326, 338, 319, 328, 279, 309, 227, 246, 191, 211, 836]`; ink px ref/ours 1898/2421 (ratio 1.2756); SSIM blocks <0.9: 235/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (74092 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (70641 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-9.72, -0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.366, differing 0.003024, SSIM₈ 0.9942 (raw 0.366, 0.003024, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9907→0.9907 / 0.5849→0.5849; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -24.75 dy 0.73; `single` dx -23.83 dy 0.73; `a` dx -22.41 dy 0.73

### 01-plain-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933593, 464, 437, 341, 371, 326, 338, 319, 328, 279, 309, 227, 246, 191, 211, 836]`; ink px ref/ours 1898/2421 (ratio 1.2756); SSIM blocks <0.9: 235/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-main-export-p1-overlay.png) (73792 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (70326 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-9.72, -0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.366, differing 0.003024, SSIM₈ 0.9942 (raw 0.366, 0.003024, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9907→0.9907 / 0.5849→0.5849; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -24.75 dy 0.73; `single` dx -23.83 dy 0.73; `a` dx -22.41 dy 0.73

### 01-plain-paragraph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934687, 523, 461, 409, 357, 365, 307, 217, 258, 220, 166, 149, 110, 115, 91, 381]`; ink px ref/ours 1898/2420 (ratio 1.275); SSIM blocks <0.9: 183/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) (75906 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-pipeline-export-p1-heatmap.png) (68702 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.12, -0.13] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.238, differing 0.002569, SSIM₈ 0.9966 (raw 0.238, 0.002569, 0.9966)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9946→0.9946 / 0.3804→0.3804; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `single` dx 0.06 dy 0.67; `line.` dx 0.06 dy 0.67; `fits` dx 0.05 dy 0.67

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933592, 501, 393, 369, 326, 314, 303, 228, 242, 221, 258, 293, 276, 217, 267, 1016]`; ink px ref/ours 2441/2420 (ratio 0.9914); SSIM blocks <0.9: 218/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [7.72, 0.02] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3811, differing 0.003094, SSIM₈ 0.9944 (raw 0.3847, 0.003107, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9909→0.9911 / 0.6148→0.609; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 24.91 dy 0.41; `single` dx 23.87 dy 0.41; `a` dx 22.41 dy 0.41

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934697, 523, 468, 412, 364, 338, 306, 221, 250, 228, 184, 130, 120, 107, 95, 373]`; ink px ref/ours 1891/2420 (ratio 1.2797); SSIM blocks <0.9: 183/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.68, -0.12] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2367, differing 0.002565, SSIM₈ 0.9967 (raw 0.2367, 0.002565, 0.9967)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9947→0.9947 / 0.3784→0.3784; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `single` dx 0.06 dy 0.67; `line.` dx 0.06 dy 0.67; `fits` dx 0.05 dy 0.67

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1843569, 8245, 6975, 6225, 5591, 5440, 5063, 5205, 4905, 4993, 4668, 4605, 4171, 4405, 4590, 20166]`; ink px ref/ours 39630/39220 (ratio 0.9897); SSIM blocks <0.9: 3873/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [3.53, 4.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.1751, differing 0.055426, SSIM₈ 0.8901 (raw 7.1751, 0.055426, 0.8901)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8247→0.8247 / 11.4662→11.4662; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 14.85; `oak` dx -415.78 dy 14.85; `branch` dx -413.6 dy 14.85

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1865202, 8541, 6901, 6635, 6187, 5810, 5205, 4902, 4174, 3820, 3188, 2946, 2731, 2484, 2302, 7788]`; ink px ref/ours 30065/39220 (ratio 1.3045); SSIM blocks <0.9: 3163/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.9, 0.11] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 4.5384, differing 0.044666, SSIM₈ 0.9368 (raw 4.5384, 0.044666, 0.9368)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8992→0.8992 / 7.2527→7.2527; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `river` dx 0.07 dy -0.36; `below.` dx 0.07 dy -0.36; `the` dx 0.06 dy -0.36

### 02-wrapping-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831980, 8181, 7016, 6406, 5963, 5948, 5852, 5289, 5188, 5656, 5267, 5378, 5255, 4876, 5221, 25340]`; ink px ref/ours 39390/39464 (ratio 1.0019); SSIM blocks <0.9: 4347/30294; [overlay](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-overlay.png) (55275 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (42381 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 8.3959, not lower; centroid estimate [-7.52, 4.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.3726, differing 0.06114, SSIM₈ 0.8663 (raw 8.3726, 0.06114, 0.8663)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7864→0.7864 / 13.3813→13.3813; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831980, 8181, 7016, 6406, 5963, 5948, 5852, 5289, 5188, 5656, 5267, 5378, 5255, 4876, 5221, 25340]`; ink px ref/ours 39390/39464 (ratio 1.0019); SSIM blocks <0.9: 4347/30294; [overlay](images/02-wrapping-paragraph/pdflatex-main-export-p1-overlay.png) (55155 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-main-export-p1-heatmap.png) (42281 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 8.3959, not lower; centroid estimate [-7.52, 4.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.3726, differing 0.06114, SSIM₈ 0.8663 (raw 8.3726, 0.06114, 0.8663)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7864→0.7864 / 13.3813→13.3813; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1843949, 8117, 7107, 5998, 5577, 5552, 4958, 5101, 4781, 4976, 4671, 4733, 4299, 4229, 4583, 20185]`; ink px ref/ours 39390/39220 (ratio 0.9957); SSIM blocks <0.9: 3826/30294; [overlay](images/02-wrapping-paragraph/pdflatex-pipeline-export-p1-overlay.png) (55243 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-pipeline-export-p1-heatmap.png) (39400 B, ÷4)
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [3.87, 4.14] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.0822, differing 0.054998, SSIM₈ 0.8919 (raw 7.1603, 0.05526, 0.891)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8261→0.8283 / 11.4425→11.3054; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 14.85; `oak` dx -415.72 dy 14.85; `branch` dx -413.61 dy 14.85

### 02-wrapping-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834636, 8219, 6952, 6893, 6423, 6140, 6491, 6184, 5976, 5757, 5362, 5028, 4837, 4417, 4790, 20711]`; ink px ref/ours 30080/39464 (ratio 1.312); SSIM blocks <0.9: 4465/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (55467 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (42826 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-8.5, -14.5] pt REJECTED: applying it gives mean|Δ| 7.8128, not lower; centroid estimate [-12.18, 0.82] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7862, differing 0.059683, SSIM₈ 0.862 (raw 7.7862, 0.059683, 0.862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7795→0.7795 / 12.4441→12.4441; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.04; `jumps` dx 416.92 dy -13.77

### 02-wrapping-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834636, 8219, 6952, 6893, 6423, 6140, 6491, 6184, 5976, 5757, 5362, 5028, 4837, 4417, 4790, 20711]`; ink px ref/ours 30080/39464 (ratio 1.312); SSIM blocks <0.9: 4465/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-overlay.png) (55356 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (42729 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-8.5, -14.5] pt REJECTED: applying it gives mean|Δ| 7.8128, not lower; centroid estimate [-12.18, 0.82] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7862, differing 0.059683, SSIM₈ 0.862 (raw 7.7862, 0.059683, 0.862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7795→0.7795 / 12.4441→12.4441; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.04; `jumps` dx 416.92 dy -13.77

### 02-wrapping-paragraph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1865112, 8452, 6961, 6696, 6112, 5868, 5236, 4937, 4180, 3697, 3259, 2976, 2729, 2442, 2381, 7778]`; ink px ref/ours 30080/39220 (ratio 1.3039); SSIM blocks <0.9: 3164/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) (54414 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-pipeline-export-p1-heatmap.png) (84958 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.79, 0.2] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 4.5467, differing 0.044684, SSIM₈ 0.9367 (raw 4.5467, 0.044684, 0.9367)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.899→0.899 / 7.2659→7.2659; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `below.` dx 0.09 dy 0.67; `the` dx 0.08 dy 0.67; `quiet` dx 0.08 dy 0.67

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1843237, 8182, 7105, 6371, 5338, 5542, 4997, 5380, 4885, 5018, 4589, 4772, 4341, 4372, 4463, 20224]`; ink px ref/ours 39556/39220 (ratio 0.9915); SSIM blocks <0.9: 3863/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [3.74, 4.14] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.1662, differing 0.055349, SSIM₈ 0.8904 (raw 7.1985, 0.055575, 0.8902)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8248→0.8255 / 11.5035→11.4492; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 14.85; `oak` dx -415.83 dy 14.85; `branch` dx -413.65 dy 14.85

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1865202, 8519, 6926, 6611, 6165, 5820, 5219, 4930, 4154, 3824, 3205, 2948, 2708, 2486, 2302, 7797]`; ink px ref/ours 30075/39220 (ratio 1.3041); SSIM blocks <0.9: 3162/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.06, 0.12] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 4.5391, differing 0.044649, SSIM₈ 0.9368 (raw 4.5391, 0.044649, 0.9368)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8992→0.8992 / 7.2539→7.2539; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `river` dx 0.07 dy 0.67; `below.` dx 0.07 dy 0.67; `the` dx 0.06 dy 0.67

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927034, 823, 788, 676, 574, 663, 677, 532, 542, 509, 470, 495, 548, 559, 536, 3390]`; ink px ref/ours 5807/4956 (ratio 0.8535); SSIM blocks <0.9: 428/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -0.5] pt by ink-projection correlation (centroid estimate [16.95, 3.04] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9214, differing 0.006537, SSIM₈ 0.9881 (raw 0.9609, 0.006696, 0.9877)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9804→0.9809 / 1.5358→1.4727; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Section` dx 38.01 dy 0.44; `Introduction` dx 29.06 dy 0.59; `Second` dx 29.06 dy 0.44
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927269, 841, 701, 639, 640, 666, 648, 652, 591, 568, 483, 440, 491, 498, 500, 3189]`; ink px ref/ours 4775/4956 (ratio 1.0379); SSIM blocks <0.9: 444/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.5] pt by ink-projection correlation (centroid estimate [12.58, 4.72] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9033, differing 0.006438, SSIM₈ 0.9876 (raw 0.9262, 0.006551, 0.9869)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.979→0.9802 / 1.4804→1.4435; header-band 1.0→0.9994 / 0.0→0.002; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Introduction` dx 29.06 dy -0.67; `Second` dx 29.06 dy 0.32; `Section` dx 29.06 dy 0.32
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923855, 915, 733, 647, 632, 615, 615, 519, 606, 614, 675, 690, 668, 592, 707, 5733]`; ink px ref/ours 6093/4705 (ratio 0.7722); SSIM blocks <0.9: 662/30294; [overlay](images/03-section-heading/pdflatex-de1020c-export-p1-overlay.png) (39794 B, ÷2), [heatmap](images/03-section-heading/pdflatex-de1020c-export-p1-heatmap.png) (41655 B, ÷2)
  - registration error (diagnostic): global shift [0.5, 52.5] pt by ink-projection correlation (centroid estimate [1.59, 20.53] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2021, differing 0.007752, SSIM₈ 0.9827 (raw 1.3496, 0.008486, 0.9793)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9669→0.9767 / 2.157→1.6932; header-band 1.0→0.9709 / 0.0→1.5688; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.75 dy 26.08; `second` dx 0.61 dy 26.08; `a` dx 0.58 dy 26.08

### 03-section-heading — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923616, 943, 723, 653, 638, 627, 618, 535, 600, 612, 715, 698, 709, 583, 762, 5784]`; ink px ref/ours 6093/4903 (ratio 0.8047); SSIM blocks <0.9: 673/30294; [overlay](images/03-section-heading/pdflatex-main-export-p1-overlay.png) (39449 B, ÷2), [heatmap](images/03-section-heading/pdflatex-main-export-p1-heatmap.png) (41393 B, ÷2)
  - registration error (diagnostic): global shift [0.5, 52.5] pt by ink-projection correlation (centroid estimate [5.4, 19.4] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2209, differing 0.007844, SSIM₈ 0.9823 (raw 1.3719, 0.008606, 0.9789)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9663→0.9765 / 2.1927→1.7078; header-band 1.0→0.9684 / 0.0→1.6746; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.75 dy 26.08; `second` dx 0.61 dy 26.08; `a` dx 0.58 dy 26.08
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926905, 849, 627, 618, 570, 654, 603, 511, 587, 560, 533, 584, 592, 541, 522, 3560]`; ink px ref/ours 6093/4956 (ratio 0.8134); SSIM blocks <0.9: 433/30294; [overlay](images/03-section-heading/pdflatex-pipeline-export-p1-overlay.png) (39489 B, ÷2), [heatmap](images/03-section-heading/pdflatex-pipeline-export-p1-heatmap.png) (89005 B, ÷1)
  - registration error (diagnostic): global shift [0.5, -0.5] pt by ink-projection correlation (centroid estimate [16.85, 2.59] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9729, differing 0.00669, SSIM₈ 0.9875 (raw 0.9952, 0.00681, 0.9873)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9797→0.9802 / 1.5906→1.5547; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Section` dx 38.01 dy 0.37; `Introduction` dx 29.06 dy 0.71; `Second` dx 29.06 dy 0.37
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924475, 952, 791, 621, 670, 652, 705, 745, 643, 649, 621, 599, 586, 629, 701, 4777]`; ink px ref/ours 4791/4705 (ratio 0.982); SSIM blocks <0.9: 718/30294; [overlay](images/03-section-heading/pdflatex-lm-de1020c-export-p1-overlay.png) (39969 B, ÷2), [heatmap](images/03-section-heading/pdflatex-lm-de1020c-export-p1-heatmap.png) (41991 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 54.0] pt by ink-projection correlation (centroid estimate [-2.62, 22.29] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0954, differing 0.00748, SSIM₈ 0.9811 (raw 1.2335, 0.008161, 0.9771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9634→0.975 / 1.9715→1.4903; header-band 1.0→0.9643 / 0.0→1.793; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.95 dy 28.22; `second` dx -11.5 dy 28.22; `a` dx -10.07 dy 28.22

### 03-section-heading — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924351, 973, 792, 646, 666, 667, 692, 761, 638, 638, 652, 591, 594, 611, 758, 4786]`; ink px ref/ours 4791/4903 (ratio 1.0234); SSIM blocks <0.9: 729/30294; [overlay](images/03-section-heading/pdflatex-lm-main-export-p1-overlay.png) (39649 B, ÷2), [heatmap](images/03-section-heading/pdflatex-lm-main-export-p1-heatmap.png) (41655 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 54.0] pt by ink-projection correlation (centroid estimate [1.18, 21.17] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1201, differing 0.007589, SSIM₈ 0.9807 (raw 1.2428, 0.008232, 0.9769)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9631→0.9747 / 1.9863→1.5143; header-band 1.0→0.9618 / 0.0→1.8988; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.95 dy 28.22; `second` dx -11.5 dy 28.22; `a` dx -10.07 dy 28.22
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927240, 848, 717, 625, 642, 639, 648, 656, 592, 565, 505, 457, 463, 514, 516, 3189]`; ink px ref/ours 4791/4956 (ratio 1.0344); SSIM blocks <0.9: 444/30294; [overlay](images/03-section-heading/pdflatex-lm-pipeline-export-p1-overlay.png) (38528 B, ÷2), [heatmap](images/03-section-heading/pdflatex-lm-pipeline-export-p1-heatmap.png) (88481 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 1.0] pt by ink-projection correlation (centroid estimate [12.63, 4.35] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8922, differing 0.006445, SSIM₈ 0.9875 (raw 0.9297, 0.006561, 0.9868)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9789→0.9803 / 1.4859→1.4222; header-band 1.0→0.998 / 0.0→0.0259; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Second` dx 29.06 dy 1.98; `Section` dx 29.06 dy 1.98; `Introduction` dx 29.06 dy 0.96
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926938, 842, 788, 688, 596, 656, 692, 570, 547, 502, 487, 462, 528, 537, 542, 3441]`; ink px ref/ours 5744/4956 (ratio 0.8628); SSIM blocks <0.9: 428/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -0.5] pt by ink-projection correlation (centroid estimate [17.0, 2.66] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9258, differing 0.006579, SSIM₈ 0.988 (raw 0.9661, 0.00674, 0.9877)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9803→0.9808 / 1.5441→1.4798; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Section` dx 38.01 dy 0.44; `Introduction` dx 29.06 dy 0.59; `Second` dx 29.06 dy 0.44
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927268, 837, 710, 636, 639, 662, 650, 654, 592, 565, 490, 440, 486, 497, 502, 3188]`; ink px ref/ours 4777/4956 (ratio 1.0375); SSIM blocks <0.9: 444/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.5] pt by ink-projection correlation (centroid estimate [12.58, 4.72] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9033, differing 0.006436, SSIM₈ 0.9876 (raw 0.9262, 0.00655, 0.9869)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.979→0.9802 / 1.4803→1.4434; header-band 1.0→0.9994 / 0.0→0.002; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Second` dx 29.06 dy 1.95; `Section` dx 29.06 dy 1.95; `Introduction` dx 29.06 dy 0.96
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933208, 516, 394, 388, 297, 286, 289, 277, 293, 285, 248, 271, 298, 241, 274, 1251]`; ink px ref/ours 2712/2449 (ratio 0.903); SSIM blocks <0.9: 247/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [46.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.4481, not lower; centroid estimate [11.57, 0.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4278, differing 0.003317, SSIM₈ 0.9938 (raw 0.4278, 0.003317, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9901→0.9901 / 0.6838→0.6838; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 31.4 dy 0.41; `one` dx 30.19 dy 0.41; `on` dx 28.87 dy 0.41
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933790, 499, 397, 373, 349, 381, 286, 322, 272, 271, 256, 214, 198, 204, 200, 804]`; ink px ref/ours 2276/2449 (ratio 1.076); SSIM blocks <0.9: 225/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-7.42, 0.04] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3305, differing 0.002918, SSIM₈ 0.9952 (raw 0.3472, 0.002996, 0.995)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9919→0.9924 / 0.555→0.5282; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `bold` dx 1.16 dy -0.68; `and` dx 0.88 dy -0.68; `emphasised,` dx 0.87 dy -0.68
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

### 04-bold-emph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933598, 513, 444, 319, 342, 300, 270, 233, 301, 263, 246, 223, 207, 251, 234, 1072]`; ink px ref/ours 2746/2465 (ratio 0.8977); SSIM blocks <0.9: 225/30294; [overlay](images/04-bold-emph/pdflatex-de1020c-export-p1-overlay.png) (76779 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-de1020c-export-p1-heatmap.png) (70915 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [3.52, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3764, differing 0.003158, SSIM₈ 0.9945 (raw 0.3836, 0.003143, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9912 / 0.6131→0.6017; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.27 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933598, 513, 444, 319, 342, 300, 270, 233, 301, 263, 246, 223, 207, 251, 234, 1072]`; ink px ref/ours 2746/2465 (ratio 0.8977); SSIM blocks <0.9: 225/30294; [overlay](images/04-bold-emph/pdflatex-main-export-p1-overlay.png) (76430 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-main-export-p1-heatmap.png) (70527 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [3.52, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3764, differing 0.003158, SSIM₈ 0.9945 (raw 0.3836, 0.003143, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9912 / 0.6131→0.6017; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.27 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933369, 499, 380, 334, 314, 321, 249, 275, 277, 265, 256, 281, 274, 236, 287, 1199]`; ink px ref/ours 2746/2449 (ratio 0.8918); SSIM blocks <0.9: 242/30294; [overlay](images/04-bold-emph/pdflatex-pipeline-export-p1-overlay.png) (77193 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-pipeline-export-p1-heatmap.png) (74383 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [13.8, 0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4169, differing 0.003295, SSIM₈ 0.9939 (raw 0.4169, 0.003295, 0.9939)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9903→0.9903 / 0.6663→0.6663; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 31.97 dy 0.41; `one` dx 30.75 dy 0.41; `on` dx 29.44 dy 0.41
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

### 04-bold-emph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933105, 492, 380, 370, 376, 348, 330, 343, 330, 319, 248, 266, 243, 259, 233, 1174]`; ink px ref/ours 2268/2465 (ratio 1.0869); SSIM blocks <0.9: 259/30294; [overlay](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-overlay.png) (77588 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-heatmap.png) (72220 B, ÷1)
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-17.16, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4162, differing 0.003327, SSIM₈ 0.9934 (raw 0.4246, 0.003322, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6786→0.6653; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -28.1 dy 0.73; `one` dx -26.97 dy 0.73; `on` dx -25.72 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933105, 492, 380, 370, 376, 348, 330, 343, 330, 319, 248, 266, 243, 259, 233, 1174]`; ink px ref/ours 2268/2465 (ratio 1.0869); SSIM blocks <0.9: 259/30294; [overlay](images/04-bold-emph/pdflatex-lm-main-export-p1-overlay.png) (77191 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-main-export-p1-heatmap.png) (71784 B, ÷1)
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-17.16, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4162, differing 0.003327, SSIM₈ 0.9934 (raw 0.4246, 0.003322, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6786→0.6653; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -28.1 dy 0.73; `one` dx -26.97 dy 0.73; `on` dx -25.72 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933726, 491, 398, 359, 367, 347, 320, 302, 255, 274, 281, 229, 215, 212, 218, 822]`; ink px ref/ours 2268/2449 (ratio 1.0798); SSIM blocks <0.9: 225/30294; [overlay](images/04-bold-emph/pdflatex-lm-pipeline-export-p1-overlay.png) (74913 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-pipeline-export-p1-heatmap.png) (70918 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-6.87, 0.05] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3296, differing 0.00292, SSIM₈ 0.9953 (raw 0.3564, 0.003017, 0.9948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9917→0.9924 / 0.5696→0.5268; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `bold` dx 1.16 dy 0.67; `emphasised,` dx 0.88 dy 0.67; `and` dx 0.88 dy 0.67
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933203, 522, 415, 385, 269, 296, 317, 278, 278, 265, 250, 263, 297, 246, 277, 1255]`; ink px ref/ours 2702/2449 (ratio 0.9064); SSIM blocks <0.9: 246/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [46.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.4462, not lower; centroid estimate [11.06, 0.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4274, differing 0.003317, SSIM₈ 0.9938 (raw 0.4274, 0.003317, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9901→0.9901 / 0.6831→0.6831; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 31.49 dy 0.41; `one` dx 30.27 dy 0.41; `on` dx 28.96 dy 0.41
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933949, 473, 428, 367, 352, 371, 302, 334, 238, 257, 245, 175, 198, 187, 182, 758]`; ink px ref/ours 2260/2449 (ratio 1.0836); SSIM blocks <0.9: 217/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.87, 0.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3307, differing 0.00293, SSIM₈ 0.9952 (raw 0.3307, 0.00293, 0.9952)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9923→0.9923 / 0.5286→0.5286; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `bold` dx 1.16 dy 0.67; `and` dx 0.88 dy 0.67; `emphasised,` dx 0.87 dy 0.67
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933992, 502, 323, 321, 268, 308, 234, 222, 257, 241, 252, 233, 247, 245, 259, 912]`; ink px ref/ours 2129/2085 (ratio 0.9793); SSIM blocks <0.9: 201/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [11.5, 0.0] pt by ink-projection correlation (centroid estimate [9.05, 0.01] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3266, differing 0.002767, SSIM₈ 0.9952 (raw 0.3579, 0.002898, 0.9948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9916→0.9926 / 0.572→0.5016; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 14.27 dy 0.41; `—` dx 13.6 dy 0.41; `Résumé` dx 11.31 dy 0.41

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934712, 472, 304, 331, 283, 384, 333, 300, 200, 240, 174, 123, 169, 155, 138, 498]`; ink px ref/ours 1547/2085 (ratio 1.3478); SSIM blocks <0.9: 186/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [0.03, -0.12] pt); confidence moderate (shift explains 21% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2087, differing 0.002248, SSIM₈ 0.9969 (raw 0.2635, 0.002446, 0.996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9936→0.9951 / 0.4211→0.3336; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 0.05 dy -0.36; `Résumé` dx 0.04 dy -0.36; `—` dx 0.04 dy -0.36

### 05-unicode — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934287, 570, 367, 345, 277, 258, 239, 232, 233, 241, 245, 185, 188, 173, 231, 745]`; ink px ref/ours 2119/2123 (ratio 1.0019); SSIM blocks <0.9: 209/30294; [overlay](images/05-unicode/pdflatex-de1020c-export-p1-overlay.png) (76114 B, ÷1), [heatmap](images/05-unicode/pdflatex-de1020c-export-p1-heatmap.png) (72769 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-5.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.331, not lower; centroid estimate [3.28, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3127, differing 0.002744, SSIM₈ 0.9955 (raw 0.3127, 0.002744, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9928→0.9928 / 0.4998→0.4998; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 30.73 dy 0.46; `also` dx -5.31 dy 0.46; `dash;` dx -3.91 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '—' (U+2014) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934287, 570, 367, 345, 277, 258, 239, 232, 233, 241, 245, 185, 188, 173, 231, 745]`; ink px ref/ours 2119/2123 (ratio 1.0019); SSIM blocks <0.9: 209/30294; [overlay](images/05-unicode/pdflatex-main-export-p1-overlay.png) (75014 B, ÷1), [heatmap](images/05-unicode/pdflatex-main-export-p1-heatmap.png) (71550 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-5.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.331, not lower; centroid estimate [3.28, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3127, differing 0.002744, SSIM₈ 0.9955 (raw 0.3127, 0.002744, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9928→0.9928 / 0.4998→0.4998; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 30.73 dy 0.46; `also` dx -5.31 dy 0.46; `dash;` dx -3.91 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934082, 524, 279, 255, 267, 278, 233, 249, 262, 254, 228, 236, 215, 222, 277, 955]`; ink px ref/ours 2119/2085 (ratio 0.984); SSIM blocks <0.9: 195/30294; [overlay](images/05-unicode/pdflatex-pipeline-export-p1-overlay.png) (74995 B, ÷1), [heatmap](images/05-unicode/pdflatex-pipeline-export-p1-heatmap.png) (72952 B, ÷1)
  - registration error (diagnostic): global shift [11.5, 0.0] pt by ink-projection correlation (centroid estimate [7.18, 0.02] pt); confidence moderate (shift explains 14% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3059, differing 0.002681, SSIM₈ 0.9957 (raw 0.3571, 0.002879, 0.995)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9921→0.9935 / 0.5708→0.4685; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 14.58 dy 0.41; `—` dx 13.92 dy 0.41; `Résumé` dx 11.63 dy 0.41

### 05-unicode — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933782, 470, 394, 304, 327, 351, 291, 322, 255, 269, 237, 246, 249, 203, 219, 897]`; ink px ref/ours 1537/2123 (ratio 1.3813); SSIM blocks <0.9: 237/30294; [overlay](images/05-unicode/pdflatex-lm-de1020c-export-p1-overlay.png) (75747 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-de1020c-export-p1-heatmap.png) (73310 B, ÷1)
  - registration error (diagnostic): global shift [2.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.9, -0.14] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3496, differing 0.00287, SSIM₈ 0.994 (raw 0.3617, 0.002921, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.9905 / 0.5782→0.5533; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 16.16 dy 0.73; `also` dx -13.37 dy 0.73; `dash;` dx -9.57 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '—' (U+2014) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933782, 470, 394, 304, 327, 351, 291, 322, 255, 269, 237, 246, 249, 203, 219, 897]`; ink px ref/ours 1537/2123 (ratio 1.3813); SSIM blocks <0.9: 237/30294; [overlay](images/05-unicode/pdflatex-lm-main-export-p1-overlay.png) (75476 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-main-export-p1-heatmap.png) (73014 B, ÷1)
  - registration error (diagnostic): global shift [2.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.9, -0.14] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3496, differing 0.00287, SSIM₈ 0.994 (raw 0.3617, 0.002921, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.9905 / 0.5782→0.5533; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 16.16 dy 0.73; `also` dx -13.37 dy 0.73; `dash;` dx -9.57 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934693, 462, 325, 315, 299, 379, 324, 309, 193, 237, 188, 127, 158, 184, 117, 506]`; ink px ref/ours 1537/2085 (ratio 1.3565); SSIM blocks <0.9: 188/30294; [overlay](images/05-unicode/pdflatex-lm-pipeline-export-p1-overlay.png) (75095 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-pipeline-export-p1-heatmap.png) (70823 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [0.99, -0.11] pt); confidence moderate (shift explains 22% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2082, differing 0.002245, SSIM₈ 0.9969 (raw 0.2654, 0.002449, 0.996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9936→0.9951 / 0.4242→0.3328; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `café` dx 0.02 dy 0.67; `Résumé` dx 0.02 dy 0.67; `café` dx 0.01 dy 0.67

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934022, 474, 305, 329, 268, 300, 248, 213, 267, 249, 251, 235, 257, 221, 253, 924]`; ink px ref/ours 2139/2085 (ratio 0.9748); SSIM blocks <0.9: 201/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [11.5, 0.0] pt by ink-projection correlation (centroid estimate [7.51, 0.0] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3296, differing 0.002774, SSIM₈ 0.9951 (raw 0.3577, 0.002881, 0.9948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9916→0.9925 / 0.5717→0.5063; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 14.22 dy 0.41; `—` dx 13.56 dy 0.41; `Résumé` dx 11.27 dy 0.41

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934715, 466, 304, 331, 283, 386, 334, 298, 198, 242, 175, 119, 174, 153, 140, 498]`; ink px ref/ours 1542/2085 (ratio 1.3521); SSIM blocks <0.9: 186/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [0.03, -0.1] pt); confidence moderate (shift explains 21% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2088, differing 0.002248, SSIM₈ 0.9969 (raw 0.2637, 0.002448, 0.996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9936→0.9951 / 0.4214→0.3337; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 0.05 dy 0.67; `Résumé` dx 0.04 dy 0.67; `—` dx 0.04 dy 0.67

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

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [139.855, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934757, 425, 319, 252, 228, 219, 241, 265, 190, 183, 193, 214, 213, 191, 231, 695]`; ink px ref/ours 1729/1739 (ratio 1.0058); SSIM blocks <0.9: 189/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [16.5, 0.0] pt by ink-projection correlation (centroid estimate [7.7, -0.38] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2801, differing 0.002417, SSIM₈ 0.9952 (raw 0.2941, 0.002449, 0.9948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9919→0.9931 / 0.4642→0.4076; header-band 0.9986→0.9986 / 0.0409→0.0409; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6529→0.7234 / 17.0219→14.2131 [131.7,62.4–343.8,93.6 pt]
- largest word displacements (pt): `text.` dx 16.46 dy -2.68; `of` dx 16.07 dy -2.68; `sentence` dx 13.66 dy -2.68
- word-sequence differences: replace ref ['a', 'b'] ours ['?', '?']; replace ref ['α'] ours ['?']; replace ref ['β,'] ours ['?', ',']

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

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [139.855, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934728, 387, 269, 272, 275, 242, 241, 259, 235, 243, 216, 218, 179, 174, 194, 684]`; ink px ref/ours 1358/1739 (ratio 1.2806); SSIM blocks <0.9: 186/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [4.57, -0.45] pt); confidence moderate (shift explains 18% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2401, differing 0.002192, SSIM₈ 0.996 (raw 0.2944, 0.002391, 0.9949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.992→0.9937 / 0.4646→0.3778; header-band 0.9986→0.9989 / 0.0409→0.0409; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.666→0.7476 / 15.7606→12.3873 [137.6,62.4–364.2,93.6 pt]
- largest word displacements (pt): `inside` dx -1.12 dy -2.68; `a` dx -1.11 dy -2.68; `sentence` dx -1.11 dy -2.68
- word-sequence differences: replace ref ['a', 'b'] ours ['?', '?']; replace ref ['α'] ours ['?']; replace ref ['β,'] ours ['?', ',']

### 06-math-inline — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933877, 403, 304, 272, 266, 302, 299, 308, 222, 194, 272, 315, 237, 246, 231, 1068]`; ink px ref/ours 1699/1833 (ratio 1.0789); SSIM blocks <0.9: 211/30294; [overlay](images/06-math-inline/pdflatex-de1020c-export-p1-overlay.png) (73649 B, ÷1), [heatmap](images/06-math-inline/pdflatex-de1020c-export-p1-heatmap.png) (73190 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-4.61, 3.51] pt); confidence strong (shift explains 32% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2595, differing 0.002359, SSIM₈ 0.996 (raw 0.3821, 0.002886, 0.9936)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9898→0.9937 / 0.6107→0.4148; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5648→0.6749 / 21.4145→17.7703 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `a` dx -7.78 dy 1.32; `sentence` dx -7.76 dy 1.32; `of` dx -7.59 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']; replace ref ['√xinside'] ours ['√x', 'inside']

### 06-math-inline — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933877, 403, 304, 272, 266, 302, 299, 308, 222, 194, 272, 315, 237, 246, 231, 1068]`; ink px ref/ours 1699/1833 (ratio 1.0789); SSIM blocks <0.9: 211/30294; [overlay](images/06-math-inline/pdflatex-main-export-p1-overlay.png) (73481 B, ÷1), [heatmap](images/06-math-inline/pdflatex-main-export-p1-heatmap.png) (72980 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-4.61, 3.51] pt); confidence strong (shift explains 32% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2595, differing 0.002359, SSIM₈ 0.996 (raw 0.3821, 0.002886, 0.9936)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9898→0.9937 / 0.6107→0.4148; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5648→0.6749 / 21.4145→17.7703 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `a` dx -7.78 dy 1.32; `sentence` dx -7.76 dy 1.32; `of` dx -7.59 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']; replace ref ['√xinside'] ours ['√x', 'inside']

### 06-math-inline — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [139.855, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934994, 362, 239, 227, 246, 248, 242, 237, 191, 176, 203, 218, 167, 179, 191, 696]`; ink px ref/ours 1699/1739 (ratio 1.0235); SSIM blocks <0.9: 181/30294; [overlay](images/06-math-inline/pdflatex-pipeline-export-p1-overlay.png) (70901 B, ÷1), [heatmap](images/06-math-inline/pdflatex-pipeline-export-p1-heatmap.png) (68417 B, ÷1)
  - registration error (diagnostic): global shift [16.5, 0.0] pt by ink-projection correlation (centroid estimate [8.31, -0.38] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2776, differing 0.002395, SSIM₈ 0.9954 (raw 0.2818, 0.002383, 0.9951)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9924→0.9935 / 0.4444→0.4036; header-band 0.9986→0.9986 / 0.0409→0.0409; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6579→0.7347 / 16.2885→14.0222 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `text.` dx 16.71 dy -2.68; `of` dx 16.32 dy -2.68; `sentence` dx 13.91 dy -2.68
- word-sequence differences: replace ref ['a', 'b'] ours ['?', '?']; replace ref ['α+', 'β,'] ours ['?', '+', '?', ',']; replace ref ['√xinside'] ours ['√', '?', 'inside']

### 06-math-inline — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933850, 444, 339, 259, 271, 318, 327, 365, 277, 225, 258, 298, 218, 220, 185, 962]`; ink px ref/ours 1340/1833 (ratio 1.3679); SSIM blocks <0.9: 225/30294; [overlay](images/06-math-inline/pdflatex-lm-de1020c-export-p1-overlay.png) (74236 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-de1020c-export-p1-heatmap.png) (73689 B, ÷1)
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-7.35, 3.41] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3234, differing 0.002634, SSIM₈ 0.9943 (raw 0.368, 0.002877, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9912 / 0.5881→0.4963; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5867→0.6637 / 18.2641→16.2875 [137.6,62.4–364.3,93.6 pt]
- largest word displacements (pt): `text.` dx -25.38 dy 1.32; `of` dx -25.05 dy 1.32; `sentence` dx -22.81 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']

### 06-math-inline — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933850, 444, 339, 259, 271, 318, 327, 365, 277, 225, 258, 298, 218, 220, 185, 962]`; ink px ref/ours 1340/1833 (ratio 1.3679); SSIM blocks <0.9: 225/30294; [overlay](images/06-math-inline/pdflatex-lm-main-export-p1-overlay.png) (74034 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-main-export-p1-heatmap.png) (73452 B, ÷1)
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-7.35, 3.41] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3234, differing 0.002634, SSIM₈ 0.9943 (raw 0.368, 0.002877, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9912 / 0.5881→0.4963; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5867→0.6637 / 18.2641→16.2875 [137.6,62.4–364.3,93.6 pt]
- largest word displacements (pt): `text.` dx -25.38 dy 1.32; `of` dx -25.05 dy 1.32; `sentence` dx -22.81 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']

### 06-math-inline — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [139.855, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934717, 389, 246, 288, 291, 239, 249, 267, 227, 249, 231, 206, 165, 163, 184, 705]`; ink px ref/ours 1340/1739 (ratio 1.2978); SSIM blocks <0.9: 186/30294; [overlay](images/06-math-inline/pdflatex-lm-pipeline-export-p1-overlay.png) (73089 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-pipeline-export-p1-heatmap.png) (70668 B, ÷1)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [5.57, -0.48] pt); confidence moderate (shift explains 18% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2416, differing 0.0022, SSIM₈ 0.9959 (raw 0.2947, 0.002394, 0.9949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.992→0.9937 / 0.4651→0.3803; header-band 0.9986→0.9989 / 0.0409→0.0409; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6653→0.7452 / 15.7708→12.5022 [137.6,62.4–364.3,93.6 pt]
- largest word displacements (pt): `a` dx -1.15 dy -2.68; `inside` dx -1.14 dy -2.68; `sentence` dx -1.14 dy -2.68
- word-sequence differences: replace ref ['a', 'b'] ours ['?', '?']; replace ref ['α+', 'β,'] ours ['?', '+', '?', ',']; replace ref ['√x'] ours ['√', '?']

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

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [139.855, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934755, 432, 302, 267, 219, 227, 247, 241, 202, 176, 205, 202, 210, 208, 214, 709]`; ink px ref/ours 1739/1739 (ratio 1.0); SSIM blocks <0.9: 189/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [16.5, 0.0] pt by ink-projection correlation (centroid estimate [8.44, -0.37] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2809, differing 0.002421, SSIM₈ 0.9952 (raw 0.2947, 0.002446, 0.9948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9919→0.993 / 0.4651→0.4088; header-band 0.9986→0.9986 / 0.0409→0.0409; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6525→0.7224 / 17.047→14.2671 [131.7,62.1–343.9,93.6 pt]
- largest word displacements (pt): `text.` dx 16.45 dy -2.68; `of` dx 16.06 dy -2.68; `sentence` dx 13.65 dy -2.68
- word-sequence differences: replace ref ['a', 'b'] ours ['?', '?']; replace ref ['α'] ours ['?']; replace ref ['β,'] ours ['?', ',']

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

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [139.855, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934729, 379, 274, 273, 274, 248, 238, 257, 238, 242, 215, 220, 179, 173, 193, 684]`; ink px ref/ours 1359/1739 (ratio 1.2796); SSIM blocks <0.9: 186/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [3.96, -0.45] pt); confidence moderate (shift explains 18% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2401, differing 0.002191, SSIM₈ 0.996 (raw 0.2944, 0.002392, 0.9949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.992→0.9937 / 0.4646→0.3778; header-band 0.9986→0.9989 / 0.0409→0.0409; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.666→0.7475 / 15.7609→12.388 [137.6,62.1–364.2,93.6 pt]
- largest word displacements (pt): `inside` dx -1.12 dy -2.68; `a` dx -1.11 dy -2.68; `sentence` dx -1.11 dy -2.68
- word-sequence differences: replace ref ['a', 'b'] ours ['?', '?']; replace ref ['α'] ours ['?']; replace ref ['β,'] ours ['?', ',']

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

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [303.542, 690.0, 44.64, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934084, 509, 410, 387, 263, 264, 324, 288, 259, 210, 210, 187, 197, 179, 171, 874]`; ink px ref/ours 1954/2052 (ratio 1.0502); SSIM blocks <0.9: 228/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3349, not lower; centroid estimate [23.38, -0.46] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3297, differing 0.002837, SSIM₈ 0.9939 (raw 0.3297, 0.002837, 0.9939)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9902→0.9902 / 0.5269→0.5269; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6063→0.6063 / 16.6593→16.6593 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 1.75 Δy 0.0 len 44.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx 8.82 dy 0.34; `display.` dx 3.63 dy 6.37; `the` dx 1.06 dy 6.37
- word-sequence differences: insert ref [] ours ['?', '∑', '?=', '1']; replace ref ['n', 'i=1', 'i=', 'n(n'] ours ['?', '=', '?', '(?']; insert ref [] ours ['(1)']

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

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [303.542, 690.0, 44.64, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934142, 391, 451, 318, 382, 348, 332, 328, 299, 235, 239, 167, 183, 143, 175, 683]`; ink px ref/ours 1608/2052 (ratio 1.2761); SSIM blocks <0.9: 224/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [8.02, -0.07] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3044, differing 0.002702, SSIM₈ 0.9942 (raw 0.3135, 0.002738, 0.9941)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9906→0.9907 / 0.501→0.4865; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6253→0.6163 / 15.617→15.7462 [261.4,84.4–349.7,122.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 1.75 Δy 0.0 len 44.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx 0.01 dy 5.6; `Before` dx 0 dy 5.6; `the` dx 0.0 dy 5.6
- word-sequence differences: insert ref [] ours ['?', '∑', '?=', '1']; replace ref ['n', 'i=1', 'i=', 'n(n'] ours ['?', '=', '?', '(?']; insert ref [] ours ['(1)']

### 07-math-display — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934567, 583, 430, 473, 321, 297, 196, 195, 302, 176, 128, 141, 187, 135, 135, 550]`; ink px ref/ours 1978/1796 (ratio 0.908); SSIM blocks <0.9: 193/30294; [overlay](images/07-math-display/pdflatex-de1020c-export-p1-overlay.png) (75037 B, ÷1), [heatmap](images/07-math-display/pdflatex-de1020c-export-p1-heatmap.png) (71021 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.55, 0.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2609, differing 0.00259, SSIM₈ 0.9949 (raw 0.2609, 0.00259, 0.9949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9919→0.9919 / 0.417→0.417; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4801→0.4801 / 19.0485→19.0485 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.1; `display.` dx 0.17 dy 2.13; `the` dx 0.11 dy 2.13
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934403, 590, 450, 476, 329, 308, 209, 207, 313, 184, 131, 149, 189, 139, 151, 588]`; ink px ref/ours 1978/1886 (ratio 0.9535); SSIM blocks <0.9: 207/30294; [overlay](images/07-math-display/pdflatex-main-export-p1-overlay.png) (75158 B, ÷1), [heatmap](images/07-math-display/pdflatex-main-export-p1-heatmap.png) (71234 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [12.76, 0.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2737, differing 0.002683, SSIM₈ 0.9945 (raw 0.2737, 0.002683, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9913→0.9913 / 0.4375→0.4375; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4801→0.4801 / 19.0485→19.0485 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.1; `display.` dx 0.17 dy 2.13; `the` dx 0.11 dy 2.13
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']; insert ref [] ours ['(1)']

### 07-math-display — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [303.542, 690.0, 44.64, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934281, 510, 337, 254, 334, 239, 287, 232, 281, 219, 220, 178, 228, 201, 194, 821]`; ink px ref/ours 1978/2052 (ratio 1.0374); SSIM blocks <0.9: 220/30294; [overlay](images/07-math-display/pdflatex-pipeline-export-p1-overlay.png) (75654 B, ÷1), [heatmap](images/07-math-display/pdflatex-pipeline-export-p1-heatmap.png) (74399 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.342, not lower; centroid estimate [24.42, -0.71] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3245, differing 0.002786, SSIM₈ 0.994 (raw 0.3245, 0.002786, 0.994)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9904→0.9904 / 0.5187→0.5187; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6049→0.6049 / 16.7328→16.7328 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 1.75 Δy 0.0 len 44.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx 8.81 dy 0.53; `display.` dx 3.62 dy 6.37; `the` dx 1.06 dy 6.37
- word-sequence differences: insert ref [] ours ['?', '∑', '?=', '1']; replace ref ['n', 'i=1', 'i=', 'n(n'] ours ['?', '=', '?', '(?']; insert ref [] ours ['(1)']

### 07-math-display — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933745, 450, 367, 487, 356, 402, 264, 258, 285, 261, 208, 305, 184, 175, 192, 877]`; ink px ref/ours 1630/1796 (ratio 1.1018); SSIM blocks <0.9: 235/30294; [overlay](images/07-math-display/pdflatex-lm-de1020c-export-p1-overlay.png) (75867 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-de1020c-export-p1-heatmap.png) (74457 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-21.21, 0.91] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3528, differing 0.00293, SSIM₈ 0.9931 (raw 0.3528, 0.00293, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9889 / 0.5639→0.5639; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4864→0.4864 / 18.773→18.773 [261.4,84.2–349.7,122.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.75 dy 2.53; `2` dx -3.54 dy 2.36; `display.` dx -3.42 dy 0.73
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933581, 457, 387, 490, 364, 413, 277, 270, 296, 269, 211, 313, 186, 179, 208, 915]`; ink px ref/ours 1630/1886 (ratio 1.1571); SSIM blocks <0.9: 249/30294; [overlay](images/07-math-display/pdflatex-lm-main-export-p1-overlay.png) (76034 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-main-export-p1-heatmap.png) (74701 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.9, 0.9] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3656, differing 0.003023, SSIM₈ 0.9927 (raw 0.3656, 0.003023, 0.9927)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9883→0.9883 / 0.5843→0.5843; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4864→0.4864 / 18.773→18.773 [261.4,84.2–349.7,122.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.75 dy 2.53; `2` dx -3.54 dy 2.36; `display.` dx -3.42 dy 0.73
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']; insert ref [] ours ['(1)']

### 07-math-display — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [303.542, 690.0, 44.64, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934088, 502, 350, 388, 365, 359, 314, 334, 304, 251, 207, 187, 160, 158, 160, 689]`; ink px ref/ours 1630/2052 (ratio 1.2589); SSIM blocks <0.9: 224/30294; [overlay](images/07-math-display/pdflatex-lm-pipeline-export-p1-overlay.png) (76229 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-pipeline-export-p1-heatmap.png) (74225 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [10.76, -0.57] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.303, differing 0.002742, SSIM₈ 0.9942 (raw 0.313, 0.00278, 0.9941)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9905→0.9907 / 0.5003→0.4844; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6199→0.6103 / 15.8368→15.9842 [261.4,84.2–349.7,122.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 1.75 Δy 0.0 len 44.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -0.01 dy 6.63; `Before` dx 0 dy 6.63; `the` dx -0.0 dy 6.63
- word-sequence differences: insert ref [] ours ['?', '∑', '?=', '1']; replace ref ['n', 'i=1', 'i=', 'n(n'] ours ['?', '=', '?', '(?']; insert ref [] ours ['(1)']

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

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [303.542, 690.0, 44.64, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934090, 502, 413, 380, 269, 272, 322, 281, 259, 216, 211, 181, 198, 176, 181, 865]`; ink px ref/ours 1971/2052 (ratio 1.0411); SSIM blocks <0.9: 228/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3344, not lower; centroid estimate [23.26, -0.55] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3294, differing 0.00284, SSIM₈ 0.9939 (raw 0.3294, 0.00284, 0.9939)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9902→0.9902 / 0.5265→0.5265; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6618→0.6618 / 13.5871→13.5871 [261.1,82.4–349.7,128.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 1.75 Δy 0.0 len 44.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx 8.81 dy 0.34; `display.` dx 3.62 dy 6.37; `the` dx 1.06 dy 6.37
- word-sequence differences: insert ref [] ours ['?', '∑', '?=', '1']; replace ref ['n', '∑', 'i=1', 'i=', 'n(n'] ours ['?', '=', '?', '(?']; insert ref [] ours ['(1)']

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

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [303.542, 690.0, 44.64, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934141, 387, 456, 322, 364, 362, 331, 331, 297, 236, 237, 169, 180, 146, 172, 685]`; ink px ref/ours 1616/2052 (ratio 1.2698); SSIM blocks <0.9: 224/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [7.59, -0.24] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3044, differing 0.002702, SSIM₈ 0.9942 (raw 0.3136, 0.002736, 0.9941)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9906→0.9907 / 0.5011→0.4866; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7012→0.7031 / 12.8967→13.0014 [261.1,82.3–349.7,128.3 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 1.75 Δy 0.0 len 44.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx 0.01 dy 6.63; `Before` dx 0 dy 6.63; `the` dx 0.0 dy 6.63
- word-sequence differences: insert ref [] ours ['?', '∑', '?=', '1']; replace ref ['n', '∑', 'i=1', 'i=', 'n(n'] ours ['?', '=', '?', '(?']; insert ref [] ours ['(1)']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1582532, 30565, 26059, 23430, 21516, 20347, 19326, 19431, 18874, 18237, 17636, 17063, 16112, 16206, 16728, 74754]`; ink px ref/ours 152381/137064 (ratio 0.8995); SSIM blocks <0.9: 14333/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 26.7909, not lower; centroid estimate [-1.35, -1.22] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.76, differing 0.206087, SSIM₈ 0.5915 (raw 26.76, 0.206087, 0.5915)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.349→0.349 / 42.7396→42.7396; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9954→0.9954 / 0.1454→0.1454
- page 2: |Δ| histogram (16 bins, pixel counts) `[1583934, 30419, 25979, 23349, 21676, 20267, 19471, 19099, 18714, 18297, 17579, 17067, 16247, 16176, 16123, 74419]`; ink px ref/ours 151462/137140 (ratio 0.9054); SSIM blocks <0.9: 14340/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 26.8271, not lower; centroid estimate [-0.32, -0.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.6272, differing 0.20525, SSIM₈ 0.5933 (raw 26.6272, 0.20525, 0.5933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3517→0.3517 / 42.5358→42.5358; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9974→0.9974 / 0.0759→0.0759
- page 3: |Δ| histogram (16 bins, pixel counts) `[1809804, 10045, 8672, 7742, 7262, 6986, 6697, 7001, 6584, 6575, 6251, 6599, 5891, 5974, 6303, 30430]`; ink px ref/ours 33623/60680 (ratio 1.8047); SSIM blocks <0.9: 5829/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.64, 72.72] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 9.9186, differing 0.073722, SSIM₈ 0.8281 (raw 10.08, 0.074242, 0.8251)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7209→0.7271 / 16.1071→15.8294; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 144.86; `branch` dx -413.6 dy 144.86; `oak` dx -415.78 dy 130.42

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1679221, 30257, 24613, 23116, 21715, 20687, 18617, 17592, 14612, 13077, 11214, 10487, 9393, 9073, 7998, 27144]`; ink px ref/ours 104967/137064 (ratio 1.3058); SSIM blocks <0.9: 11040/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.42, -0.44] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9638, differing 0.157272, SSIM₈ 0.7777 (raw 15.9638, 0.157272, 0.7777)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6455→0.6455 / 25.4994→25.4994; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9982→0.9982 / 0.0806→0.0806
- page 2: |Δ| histogram (16 bins, pixel counts) `[1679037, 30431, 24428, 23048, 21754, 20671, 18848, 17381, 14630, 13286, 11078, 10486, 9549, 8950, 8069, 27170]`; ink px ref/ours 104819/137140 (ratio 1.3084); SSIM blocks <0.9: 11081/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.52, -0.13] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9808, differing 0.157313, SSIM₈ 0.7767 (raw 15.9808, 0.157313, 0.7767)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6438→0.6438 / 25.5337→25.5337; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9993→0.9993 / 0.0301→0.0301
- page 3: |Δ| histogram (16 bins, pixel counts) `[1824206, 13452, 10860, 10211, 9694, 9205, 8046, 7853, 6415, 5662, 4935, 4578, 4200, 3915, 3571, 12013]`; ink px ref/ours 46386/60680 (ratio 1.3082); SSIM blocks <0.9: 4895/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.88, 0.81] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.0392, differing 0.06944, SSIM₈ 0.9017 (raw 7.0392, 0.06944, 0.9017)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8431→0.8431 / 11.2488→11.2488; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `below.` dx 0.04 dy -0.36; `below.` dx 0.04 dy -0.36; `below.` dx 0.04 dy -0.36

### 08-two-page — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p1-overlay.png) (56504 B, ÷8), [heatmap](images/08-two-page/pdflatex-de1020c-export-p1-heatmap.png) (38399 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -14.5] pt by ink-projection correlation (centroid estimate [-6.07, -5.06] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.4928, differing 0.229367, SSIM₈ 0.4891 (raw 31.5764, 0.22951, 0.4881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1821→0.1892 / 50.4615→50.1371; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.96 / 0.0468→1.3594
- page 2: |Δ| histogram (16 bins, pixel counts) `[1518465, 30739, 25693, 24091, 22137, 22533, 22535, 20944, 20824, 22593, 20919, 20497, 20504, 19480, 20990, 105872]`; ink px ref/ours 150460/131743 (ratio 0.8756); SSIM blocks <0.9: 16800/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p2-overlay.png) (54962 B, ÷8), [heatmap](images/08-two-page/pdflatex-de1020c-export-p2-heatmap.png) (38797 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 6.0] pt by ink-projection correlation (centroid estimate [-5.52, 7.86] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.0952, differing 0.225993, SSIM₈ 0.4952 (raw 33.6289, 0.239479, 0.4516)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1237→0.1935 / 53.7443→49.6907; header-band 1.0→0.999 / 0.0→0.0276; footer-band 0.9986→0.9986 / 0.0304→0.0304
- page 3: |Δ| histogram (16 bins, pixel counts) `[1774335, 11537, 9689, 9104, 8316, 8407, 8934, 8213, 7615, 9012, 8045, 7866, 7928, 7590, 8264, 43961]`; ink px ref/ours 33504/71008 (ratio 2.1194); SSIM blocks <0.9: 7408/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p3-overlay.png) (66565 B, ÷4), [heatmap](images/08-two-page/pdflatex-de1020c-export-p3-heatmap.png) (59542 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 34.5] pt by ink-projection correlation (centroid estimate [-6.67, 111.28] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.0633, differing 0.081669, SSIM₈ 0.8097 (raw 13.387, 0.093579, 0.7605)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6172→0.7105 / 21.3963→16.9124; header-band 1.0→0.8991 / 0.0→5.299; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 214.06; `branch` dx -435.47 dy 167.32; `over` dx -407.04 dy 214.02

### 08-two-page — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/08-two-page/pdflatex-main-export-p1-overlay.png) (56322 B, ÷8), [heatmap](images/08-two-page/pdflatex-main-export-p1-heatmap.png) (38218 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -14.5] pt by ink-projection correlation (centroid estimate [-6.07, -5.06] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.4928, differing 0.229367, SSIM₈ 0.4891 (raw 31.5764, 0.22951, 0.4881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1821→0.1892 / 50.4615→50.1371; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.96 / 0.0468→1.3594
- page 2: |Δ| histogram (16 bins, pixel counts) `[1518465, 30739, 25693, 24091, 22137, 22533, 22535, 20944, 20824, 22593, 20919, 20497, 20504, 19480, 20990, 105872]`; ink px ref/ours 150460/131743 (ratio 0.8756); SSIM blocks <0.9: 16800/30294; [overlay](images/08-two-page/pdflatex-main-export-p2-overlay.png) (54787 B, ÷8), [heatmap](images/08-two-page/pdflatex-main-export-p2-heatmap.png) (38614 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 6.0] pt by ink-projection correlation (centroid estimate [-5.52, 7.86] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.0952, differing 0.225993, SSIM₈ 0.4952 (raw 33.6289, 0.239479, 0.4516)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1237→0.1935 / 53.7443→49.6907; header-band 1.0→0.999 / 0.0→0.0276; footer-band 0.9986→0.9986 / 0.0304→0.0304
- page 3: |Δ| histogram (16 bins, pixel counts) `[1774335, 11537, 9689, 9104, 8316, 8407, 8934, 8213, 7615, 9012, 8045, 7866, 7928, 7590, 8264, 43961]`; ink px ref/ours 33504/71008 (ratio 2.1194); SSIM blocks <0.9: 7408/30294; [overlay](images/08-two-page/pdflatex-main-export-p3-overlay.png) (66466 B, ÷4), [heatmap](images/08-two-page/pdflatex-main-export-p3-heatmap.png) (59467 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 34.5] pt by ink-projection correlation (centroid estimate [-6.67, 111.28] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.0633, differing 0.081669, SSIM₈ 0.8097 (raw 13.387, 0.093579, 0.7605)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6172→0.7105 / 21.3963→16.9124; header-band 1.0→0.8991 / 0.0→5.299; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 214.06; `branch` dx -435.47 dy 167.32; `over` dx -407.04 dy 214.02

### 08-two-page — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1583040, 30686, 25741, 23074, 21550, 20486, 19257, 19723, 18532, 18146, 17688, 17169, 15740, 15834, 16380, 75770]`; ink px ref/ours 151753/137064 (ratio 0.9032); SSIM blocks <0.9: 14229/30294; [overlay](images/08-two-page/pdflatex-pipeline-export-p1-overlay.png) (57708 B, ÷8), [heatmap](images/08-two-page/pdflatex-pipeline-export-p1-heatmap.png) (36422 B, ÷8)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.1, -2.23] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.6362, differing 0.205191, SSIM₈ 0.5954 (raw 26.7545, 0.205539, 0.5933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3515→0.3576 / 42.7356→42.4824; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9951→0.9953 / 0.1484→0.1484
- page 2: |Δ| histogram (16 bins, pixel counts) `[1583820, 30040, 25563, 22832, 21473, 20749, 19258, 19461, 18706, 18132, 17581, 17090, 16116, 15669, 16524, 75802]`; ink px ref/ours 150460/137140 (ratio 0.9115); SSIM blocks <0.9: 14259/30294; [overlay](images/08-two-page/pdflatex-pipeline-export-p2-overlay.png) (57773 B, ÷8), [heatmap](images/08-two-page/pdflatex-pipeline-export-p2-heatmap.png) (36490 B, ÷8)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-0.89, -1.32] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.5326, differing 0.204525, SSIM₈ 0.5956 (raw 26.7672, 0.204965, 0.593)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3508→0.3578 / 42.7661→42.3267; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9971→0.9966 / 0.0778→0.0778
- page 3: |Δ| histogram (16 bins, pixel counts) `[1810605, 10099, 8603, 7598, 7344, 6930, 6661, 7094, 6413, 6678, 6224, 6551, 5790, 5958, 6244, 30024]`; ink px ref/ours 33504/60680 (ratio 1.8111); SSIM blocks <0.9: 5804/30294; [overlay](images/08-two-page/pdflatex-pipeline-export-p3-overlay.png) (63208 B, ÷4), [heatmap](images/08-two-page/pdflatex-pipeline-export-p3-heatmap.png) (49032 B, ÷4)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.41, 71.68] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 9.9261, differing 0.073644, SSIM₈ 0.8282 (raw 9.9944, 0.07373, 0.8269)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7236→0.7269 / 15.9719→15.8354; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 144.86; `branch` dx -413.61 dy 144.86; `oak` dx -415.72 dy 130.42

### 08-two-page — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p1-overlay.png) (55057 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p1-heatmap.png) (37650 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.44, -3.32] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4403, differing 0.210395, SSIM₈ 0.5038 (raw 27.5467, 0.210674, 0.5026)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.208 / 44.0147→43.8447; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1556053, 29239, 25104, 24114, 22751, 21524, 23492, 22483, 22053, 21728, 19173, 18342, 18558, 16712, 18303, 79187]`; ink px ref/ours 105011/131743 (ratio 1.2546); SSIM blocks <0.9: 16449/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p2-overlay.png) (54094 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p2-heatmap.png) (38418 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 36.5] pt by ink-projection correlation (centroid estimate [-5.19, 9.09] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.3288, differing 0.209484, SSIM₈ 0.5027 (raw 29.0579, 0.219561, 0.4664)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.148→0.2198 / 46.4361→42.915; header-band 1.0→0.9042 / 0.0→5.2131; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746783, 14069, 12340, 11757, 11133, 10758, 11588, 11172, 10690, 10911, 9823, 9052, 9221, 8387, 9339, 41793]`; ink px ref/ours 46441/71008 (ratio 1.529); SSIM blocks <0.9: 8608/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p3-overlay.png) (81008 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p3-heatmap.png) (66089 B, ÷4)
  - registration error (diagnostic): global shift [-1.0, 52.0] pt by ink-projection correlation (centroid estimate [-5.44, 40.42] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.4866, differing 0.102874, SSIM₈ 0.754 (raw 14.7842, 0.109995, 0.721)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5542→0.6347 / 23.6284→20.0792; header-band 1.0→0.8091 / 0.0→10.1537; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 55.47; `oak` dx 426.94 dy 37.61; `oak` dx 426.94 dy 32.12

### 08-two-page — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p1-overlay.png) (54851 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-main-export-p1-heatmap.png) (37436 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.44, -3.32] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4403, differing 0.210395, SSIM₈ 0.5038 (raw 27.5467, 0.210674, 0.5026)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.208 / 44.0147→43.8447; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1556053, 29239, 25104, 24114, 22751, 21524, 23492, 22483, 22053, 21728, 19173, 18342, 18558, 16712, 18303, 79187]`; ink px ref/ours 105011/131743 (ratio 1.2546); SSIM blocks <0.9: 16449/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p2-overlay.png) (53891 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-main-export-p2-heatmap.png) (38213 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 36.5] pt by ink-projection correlation (centroid estimate [-5.19, 9.09] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.3288, differing 0.209484, SSIM₈ 0.5027 (raw 29.0579, 0.219561, 0.4664)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.148→0.2198 / 46.4361→42.915; header-band 1.0→0.9042 / 0.0→5.2131; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746783, 14069, 12340, 11757, 11133, 10758, 11588, 11172, 10690, 10911, 9823, 9052, 9221, 8387, 9339, 41793]`; ink px ref/ours 46441/71008 (ratio 1.529); SSIM blocks <0.9: 8608/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p3-overlay.png) (80884 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-main-export-p3-heatmap.png) (65970 B, ÷4)
  - registration error (diagnostic): global shift [-1.0, 52.0] pt by ink-projection correlation (centroid estimate [-5.44, 40.42] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.4866, differing 0.102874, SSIM₈ 0.754 (raw 14.7842, 0.109995, 0.721)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5542→0.6347 / 23.6284→20.0792; header-band 1.0→0.8091 / 0.0→10.1537; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 55.47; `oak` dx 426.94 dy 37.61; `oak` dx 426.94 dy 32.12

### 08-two-page — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1678891, 30013, 24898, 23141, 21598, 20781, 18655, 17662, 14743, 12707, 11482, 10476, 9413, 8881, 8415, 27060]`; ink px ref/ours 105111/137064 (ratio 1.304); SSIM blocks <0.9: 11045/30294; [overlay](images/08-two-page/pdflatex-lm-pipeline-export-p1-overlay.png) (54533 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-pipeline-export-p1-heatmap.png) (81932 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.48, -0.49] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9984, differing 0.157345, SSIM₈ 0.7771 (raw 15.9984, 0.157345, 0.7771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6447→0.6447 / 25.5546→25.5546; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9981→0.9981 / 0.0813→0.0813
- page 2: |Δ| histogram (16 bins, pixel counts) `[1678712, 30153, 24763, 23163, 21564, 20782, 18781, 17551, 14720, 12926, 11335, 10477, 9551, 8754, 8450, 27134]`; ink px ref/ours 105011/137140 (ratio 1.306); SSIM blocks <0.9: 11088/30294; [overlay](images/08-two-page/pdflatex-lm-pipeline-export-p2-overlay.png) (54533 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-pipeline-export-p2-heatmap.png) (82283 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.55, -0.09] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 16.0143, differing 0.157415, SSIM₈ 0.7761 (raw 16.0143, 0.157415, 0.7761)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.643→0.643 / 25.5873→25.5873; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9993→0.9993 / 0.0303→0.0303
- page 3: |Δ| histogram (16 bins, pixel counts) `[1824092, 13273, 10951, 10271, 9628, 9232, 8087, 7928, 6429, 5489, 5060, 4590, 4207, 3857, 3709, 12013]`; ink px ref/ours 46441/60680 (ratio 1.3066); SSIM blocks <0.9: 4904/30294; [overlay](images/08-two-page/pdflatex-lm-pipeline-export-p3-overlay.png) (74105 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-pipeline-export-p3-heatmap.png) (45923 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.17, 0.83] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.0558, differing 0.069502, SSIM₈ 0.9014 (raw 7.0558, 0.069502, 0.9014)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8427→0.8427 / 11.2754→11.2754; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `below.` dx 0.04 dy 0.67; `below.` dx 0.04 dy 0.67; `below.` dx 0.04 dy 0.67

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1581634, 30214, 25515, 23782, 21671, 20933, 19589, 19568, 18768, 18249, 17860, 17780, 16063, 16138, 16195, 74857]`; ink px ref/ours 152232/137064 (ratio 0.9004); SSIM blocks <0.9: 14337/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.39, -1.18] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.718, differing 0.206581, SSIM₈ 0.5911 (raw 26.825, 0.206396, 0.5903)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3471→0.3519 / 42.8432→42.5781; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9952→0.9948 / 0.1455→0.1455
- page 2: |Δ| histogram (16 bins, pixel counts) `[1583388, 30403, 25383, 23691, 21653, 20834, 19384, 19158, 18778, 18481, 18220, 17581, 15856, 15697, 16083, 74226]`; ink px ref/ours 151458/137140 (ratio 0.9055); SSIM blocks <0.9: 14359/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 26.8068, not lower; centroid estimate [-0.4, -0.4] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.6433, differing 0.205302, SSIM₈ 0.5929 (raw 26.6433, 0.205302, 0.5929)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3511→0.3511 / 42.5613→42.5613; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9975→0.9975 / 0.0759→0.0759
- page 3: |Δ| histogram (16 bins, pixel counts) `[1809997, 9944, 8508, 7736, 7386, 7133, 6675, 6904, 6706, 6599, 6349, 6695, 5848, 6094, 6209, 30033]`; ink px ref/ours 33659/60680 (ratio 1.8028); SSIM blocks <0.9: 5837/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.63, 72.59] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 9.9754, differing 0.073973, SSIM₈ 0.8275 (raw 10.0526, 0.074211, 0.8255)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7216→0.726 / 16.0632→15.9201; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 144.86; `branch` dx -413.65 dy 144.86; `oak` dx -415.83 dy 130.42

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1679213, 30197, 24622, 23082, 21587, 20834, 18656, 17658, 14519, 13108, 11247, 10477, 9366, 9061, 8023, 27166]`; ink px ref/ours 105020/137064 (ratio 1.3051); SSIM blocks <0.9: 11040/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.5, -0.38] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.968, differing 0.15725, SSIM₈ 0.7776 (raw 15.968, 0.15725, 0.7776)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6454→0.6454 / 25.506→25.506; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9982→0.9982 / 0.0808→0.0808
- page 2: |Δ| histogram (16 bins, pixel counts) `[1679061, 30352, 24469, 22931, 21704, 20798, 18854, 17451, 14555, 13294, 11143, 10455, 9505, 8950, 8099, 27195]`; ink px ref/ours 104857/137140 (ratio 1.3079); SSIM blocks <0.9: 11080/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.61, -0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9849, differing 0.157286, SSIM₈ 0.7766 (raw 15.9849, 0.157286, 0.7766)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6437→0.6437 / 25.5402→25.5402; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9992→0.9992 / 0.0302→0.0302
- page 3: |Δ| histogram (16 bins, pixel counts) `[1824205, 13398, 10906, 10153, 9694, 9232, 8054, 7921, 6338, 5683, 4958, 4590, 4161, 3897, 3596, 12030]`; ink px ref/ours 46395/60680 (ratio 1.3079); SSIM blocks <0.9: 4896/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.95, 0.83] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.0408, differing 0.069434, SSIM₈ 0.9017 (raw 7.0408, 0.069434, 0.9017)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8431→0.8431 / 11.2514→11.2514; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `below.` dx 0.04 dy 0.67; `below.` dx 0.04 dy 0.67; `below.` dx 0.04 dy 0.67

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

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.364, 665.007, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1911501, 1957, 1684, 1512, 1477, 1567, 1674, 1437, 1338, 1266, 1376, 1308, 1202, 1451, 1315, 6751]`; ink px ref/ours 9691/9284 (ratio 0.958); SSIM blocks <0.9: 1181/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, -17.5] pt by ink-projection correlation (centroid estimate [17.16, -9.87] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 2.1034, differing 0.015178, SSIM₈ 0.9644 (raw 2.1735, 0.015569, 0.9628)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9406→0.9433 / 3.4739→3.3604; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6251→0.6314 / 19.6218→16.4627 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -436.13 dy -2.41; `2` dx 181.2 dy -39.31; `+` dx 178.63 dy -39.31
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['1', '1', '=', '2']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']

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

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.364, 665.007, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1913369, 2055, 1864, 1813, 1594, 1648, 1600, 1549, 1331, 1232, 1243, 1187, 1030, 1197, 1067, 5037]`; ink px ref/ours 7447/9284 (ratio 1.2467); SSIM blocks <0.9: 1166/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -17.0] pt by ink-projection correlation (centroid estimate [11.52, -7.17] pt); confidence moderate (shift explains 15% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.5892, differing 0.013263, SSIM₈ 0.9706 (raw 1.873, 0.014683, 0.9647)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9436→0.953 / 2.9936→2.5399; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7168→0.6339 / 16.9577→14.3857 [275.7,96.4–519.8,156 pt]
- largest word displacements (pt): `2` dx 181.2 dy -38.74; `+` dx 178.63 dy -38.74; `Mixed` dx 29.06 dy -0.67
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['1', '1', '=', '2']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']

### 09-mixed-document — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908448, 1975, 1818, 1995, 1553, 1434, 1436, 1374, 1435, 1503, 1793, 1550, 1349, 1400, 1470, 8283]`; ink px ref/ours 9857/9252 (ratio 0.9386); SSIM blocks <0.9: 1551/30294; [overlay](images/09-mixed-document/pdflatex-de1020c-export-p1-overlay.png) (53665 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-de1020c-export-p1-heatmap.png) (55263 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 38.5] pt by ink-projection correlation (centroid estimate [7.57, 32.71] pt); confidence moderate (shift explains 24% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8886, differing 0.014393, SSIM₈ 0.9685 (raw 2.4853, 0.017384, 0.9504)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9207→0.9518 / 3.9723→2.8965; header-band 1.0→0.9857 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6361→0.7546 / 15.3285→9.5698 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `twelve` dx -3.05 dy 39.17; `in` dx -2.78 dy 39.17; `set` dx -2.52 dy 39.17
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908171, 1976, 1849, 2001, 1559, 1421, 1424, 1373, 1456, 1524, 1817, 1563, 1351, 1440, 1492, 8399]`; ink px ref/ours 9857/9465 (ratio 0.9602); SSIM blocks <0.9: 1570/30294; [overlay](images/09-mixed-document/pdflatex-main-export-p1-overlay.png) (53411 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-main-export-p1-heatmap.png) (54956 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 38.5] pt by ink-projection correlation (centroid estimate [10.11, 31.38] pt); confidence moderate (shift explains 24% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.911, differing 0.014521, SSIM₈ 0.9679 (raw 2.5141, 0.017545, 0.9497)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9195→0.9512 / 4.0183→2.9169; header-band 1.0→0.9827 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6361→0.7546 / 15.3285→9.5698 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `twelve` dx -3.05 dy 39.17; `in` dx -2.78 dy 39.17; `set` dx -2.52 dy 39.17
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.364, 665.007, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1911473, 1897, 1726, 1527, 1514, 1464, 1572, 1413, 1455, 1273, 1512, 1386, 1208, 1376, 1322, 6698]`; ink px ref/ours 9857/9284 (ratio 0.9419); SSIM blocks <0.9: 1187/30294; [overlay](images/09-mixed-document/pdflatex-pipeline-export-p1-overlay.png) (54392 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-pipeline-export-p1-heatmap.png) (50226 B, ÷2)
  - registration error (diagnostic): global shift [1.0, -17.0] pt by ink-projection correlation (centroid estimate [18.47, -9.3] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 2.0728, differing 0.014976, SSIM₈ 0.9646 (raw 2.1784, 0.01556, 0.9626)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9403→0.9437 / 3.4817→3.3113; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6222→0.6294 / 19.8655→16.4864 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `twelve` dx -436.31 dy -2.3; `2` dx 181.2 dy -39.4; `+` dx 178.63 dy -39.4
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['1', '1', '=', '2']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']

### 09-mixed-document — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909812, 2060, 1849, 1927, 1609, 1602, 1682, 1704, 1540, 1378, 1642, 1427, 1316, 1278, 1363, 6627]`; ink px ref/ours 7573/9252 (ratio 1.2217); SSIM blocks <0.9: 1632/30294; [overlay](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-overlay.png) (54199 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-heatmap.png) (55530 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 39.5] pt by ink-projection correlation (centroid estimate [-1.26, 34.72] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9012, differing 0.014685, SSIM₈ 0.9623 (raw 2.2522, 0.016745, 0.948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9169→0.9418 / 3.5997→2.9167; header-band 1.0→0.9856 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.381→0.5284 / 23.145→12.7296 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 25.83; `paper.` dx -52.2 dy 40.23; `letter` dx -48.65 dy 40.23
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909619, 2067, 1888, 1937, 1626, 1586, 1662, 1710, 1557, 1411, 1651, 1441, 1322, 1309, 1383, 6647]`; ink px ref/ours 7573/9465 (ratio 1.2498); SSIM blocks <0.9: 1647/30294; [overlay](images/09-mixed-document/pdflatex-lm-main-export-p1-overlay.png) (53914 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-main-export-p1-heatmap.png) (55220 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 39.5] pt by ink-projection correlation (centroid estimate [1.27, 33.39] pt); confidence moderate (shift explains 15% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9236, differing 0.014813, SSIM₈ 0.9616 (raw 2.2675, 0.016855, 0.9475)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9161→0.9411 / 3.6241→2.9371; header-band 1.0→0.9826 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.381→0.5284 / 23.145→12.7296 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 25.83; `paper.` dx -52.2 dy 40.23; `letter` dx -48.65 dy 40.23
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.364, 665.007, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1913900, 1905, 1705, 1566, 1556, 1612, 1716, 1460, 1471, 1139, 1303, 1171, 1041, 1216, 1159, 4896]`; ink px ref/ours 7573/9284 (ratio 1.2259); SSIM blocks <0.9: 1217/30294; [overlay](images/09-mixed-document/pdflatex-lm-pipeline-export-p1-overlay.png) (53732 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-pipeline-export-p1-heatmap.png) (49331 B, ÷2)
  - registration error (diagnostic): global shift [0.0, -16.5] pt by ink-projection correlation (centroid estimate [9.63, -7.3] pt); confidence moderate (shift explains 13% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.6085, differing 0.013296, SSIM₈ 0.9701 (raw 1.8593, 0.01445, 0.9634)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9415→0.9521 / 2.9717→2.5709; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3759→0.3723 / 29.4508→28.0937 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `2` dx 181.2 dy -38.42; `+` dx 178.63 dy -38.42; `Mixed` dx 29.06 dy 0.96
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['1', '1', '=', '2']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']

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

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.364, 665.007, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1911556, 1958, 1631, 1475, 1462, 1642, 1666, 1436, 1348, 1327, 1334, 1353, 1151, 1408, 1344, 6725]`; ink px ref/ours 9680/9284 (ratio 0.9591); SSIM blocks <0.9: 1185/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, -17.5] pt by ink-projection correlation (centroid estimate [18.42, -9.86] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 2.079, differing 0.015084, SSIM₈ 0.9647 (raw 2.1695, 0.015553, 0.9628)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9405→0.9438 / 3.4675→3.3213; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6253→0.6314 / 19.6066→16.4627 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -436.17 dy -2.41; `2` dx 181.2 dy -39.31; `+` dx 178.63 dy -39.31
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['1', '1', '=', '2']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']

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

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.364, 665.007, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1913351, 2071, 1855, 1816, 1600, 1637, 1622, 1515, 1354, 1244, 1221, 1192, 1012, 1205, 1077, 5044]`; ink px ref/ours 7446/9284 (ratio 1.2468); SSIM blocks <0.9: 1166/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -17.0] pt by ink-projection correlation (centroid estimate [11.67, -7.19] pt); confidence moderate (shift explains 15% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.59, differing 0.013264, SSIM₈ 0.9706 (raw 1.8737, 0.01468, 0.9647)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9436→0.953 / 2.9948→2.5413; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3409→0.3858 / 29.4202→28.29 [275.7,121.1–336.2,156 pt]
- largest word displacements (pt): `2` dx 181.2 dy -38.74; `+` dx 178.63 dy -38.74; `Mixed` dx 29.06 dy 0.96
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['1', '1', '=', '2']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']

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

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893214, 4005, 3363, 2992, 2803, 2652, 2391, 2513, 2385, 2412, 2240, 2292, 2168, 2115, 2188, 9083]`; ink px ref/ours 18767/18366 (ratio 0.9786); SSIM blocks <0.9: 1866/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.98, 1.81] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3993, differing 0.026536, SSIM₈ 0.9471 (raw 3.3993, 0.026536, 0.9471)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9156→0.9156 / 5.4325→5.4325; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
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

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1902630, 4015, 3402, 3053, 2744, 2678, 2435, 2370, 1999, 1747, 1649, 1747, 1445, 1228, 1226, 4448]`; ink px ref/ours 14211/18366 (ratio 1.2924); SSIM blocks <0.9: 1551/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [2.35, -0.11] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 2.1847, differing 0.021243, SSIM₈ 0.9681 (raw 2.3198, 0.021814, 0.9657)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9451→0.949 / 3.7077→3.4918; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `the` dx 0.06 dy -0.36; `line.` dx 0.06 dy -0.36; `close` dx 0.05 dy -0.36
- word-sequence differences: delete ref ['�'] ours []

### 10-unicode-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893459, 4089, 3397, 2841, 2581, 2734, 2699, 2550, 2363, 2423, 2285, 2125, 2111, 2043, 2055, 9061]`; ink px ref/ours 18728/18449 (ratio 0.9851); SSIM blocks <0.9: 1812/30294; [overlay](images/10-unicode-paragraph/pdflatex-de1020c-export-p1-overlay.png) (78112 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (60327 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.7, 1.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.362, differing 0.026242, SSIM₈ 0.9486 (raw 3.362, 0.026242, 0.9486)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.918→0.918 / 5.373→5.373; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 12.0 Δy 0.0 len 10.0 vs 12.0, thickness px 1 vs 1
- largest word displacements (pt): `quotes’,` dx -429.81 dy 14.82; `æ,` dx -421.71 dy 14.68; `å,` dx -421.21 dy 14.68
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['C12'] ours ['€12']

### 10-unicode-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (15): warning: 'Š' (U+0160) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893459, 4089, 3397, 2841, 2581, 2734, 2699, 2550, 2363, 2423, 2285, 2125, 2111, 2043, 2055, 9061]`; ink px ref/ours 18728/18449 (ratio 0.9851); SSIM blocks <0.9: 1812/30294; [overlay](images/10-unicode-paragraph/pdflatex-main-export-p1-overlay.png) (77497 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-main-export-p1-heatmap.png) (59679 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.7, 1.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.362, differing 0.026242, SSIM₈ 0.9486 (raw 3.362, 0.026242, 0.9486)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.918→0.918 / 5.373→5.373; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 12.0 Δy 0.0 len 10.0 vs 12.0, thickness px 1 vs 1
- largest word displacements (pt): `quotes’,` dx -429.81 dy 14.82; `æ,` dx -421.71 dy 14.68; `å,` dx -421.21 dy 14.68
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['C12'] ours ['€12']

### 10-unicode-paragraph — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893840, 4023, 3312, 2914, 2657, 2593, 2410, 2548, 2288, 2352, 2210, 2193, 2198, 2090, 2089, 9099]`; ink px ref/ours 18728/18366 (ratio 0.9807); SSIM blocks <0.9: 1843/30294; [overlay](images/10-unicode-paragraph/pdflatex-pipeline-export-p1-overlay.png) (78584 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-pipeline-export-p1-heatmap.png) (61321 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.86, 1.94] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3581, differing 0.026339, SSIM₈ 0.9479 (raw 3.3581, 0.026339, 0.9479)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9168→0.9168 / 5.3666→5.3666; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `single` dx -398.5 dy 14.85; `quotes’,` dx -398.1 dy 14.85; `Kraków,` dx -394.31 dy 14.85
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['C12'] ours ['€12']

### 10-unicode-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893882, 3869, 3242, 2959, 2871, 2862, 2862, 2749, 2745, 2384, 2325, 2135, 2070, 1985, 2056, 7820]`; ink px ref/ours 14393/18449 (ratio 1.2818); SSIM blocks <0.9: 1933/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (79144 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (62677 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.76, -1.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.251, differing 0.025995, SSIM₈ 0.9445 (raw 3.251, 0.025995, 0.9445)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9112→0.9112 / 5.1961→5.1961; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 95.5 pt with NO reference rule on this page
- largest word displacements (pt): `that` dx 447.55 dy -14.89; `Łódź,` dx 428.89 dy -14.75; `single` dx 411.29 dy -14.79
- word-sequence differences: replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']

### 10-unicode-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (15): warning: 'Š' (U+0160) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893882, 3869, 3242, 2959, 2871, 2862, 2862, 2749, 2745, 2384, 2325, 2135, 2070, 1985, 2056, 7820]`; ink px ref/ours 14393/18449 (ratio 1.2818); SSIM blocks <0.9: 1933/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-main-export-p1-overlay.png) (78616 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (62164 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.76, -1.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.251, differing 0.025995, SSIM₈ 0.9445 (raw 3.251, 0.025995, 0.9445)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9112→0.9112 / 5.1961→5.1961; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 95.5 pt with NO reference rule on this page
- largest word displacements (pt): `that` dx 447.55 dy -14.89; `Łódź,` dx 428.89 dy -14.75; `single` dx 411.29 dy -14.79
- word-sequence differences: replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']

### 10-unicode-paragraph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1902002, 3984, 3355, 3028, 2858, 2712, 2438, 2400, 2014, 1784, 1705, 1752, 1517, 1290, 1293, 4684]`; ink px ref/ours 14393/18366 (ratio 1.276); SSIM blocks <0.9: 1581/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) (76391 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-pipeline-export-p1-heatmap.png) (56364 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.8, -0.5] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 2.2154, differing 0.021415, SSIM₈ 0.9673 (raw 2.3867, 0.022086, 0.9643)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9429→0.9477 / 3.8147→3.5409; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `the` dx -10.77 dy -0.36; `close` dx -10.76 dy -0.36; `line.` dx -10.76 dy -0.36
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

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893240, 4080, 3316, 2943, 2806, 2663, 2402, 2600, 2223, 2291, 2270, 2273, 2252, 2083, 2178, 9196]`; ink px ref/ours 18684/18366 (ratio 0.983); SSIM blocks <0.9: 1864/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.5, 0.0] pt REJECTED: applying it gives mean|Δ| 3.4043, not lower; centroid estimate [3.32, 1.83] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.4027, differing 0.026492, SSIM₈ 0.947 (raw 3.4027, 0.026492, 0.947)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9154→0.9154 / 5.4378→5.4378; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Łódź,` dx -425.09 dy 14.85; `single` dx -398.55 dy 14.85; `quotes’,` dx -398.15 dy 14.85
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

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1902611, 4031, 3383, 3076, 2742, 2669, 2443, 2362, 2009, 1743, 1646, 1748, 1450, 1223, 1225, 4455]`; ink px ref/ours 14217/18366 (ratio 1.2918); SSIM blocks <0.9: 1551/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.9, -0.12] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 2.1847, differing 0.021243, SSIM₈ 0.9681 (raw 2.3204, 0.021814, 0.9657)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9451→0.949 / 3.7087→3.4918; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 0.06 dy 0.67; `close` dx 0.05 dy 0.67; `the` dx 0.05 dy 0.67
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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920563, 1417, 1066, 995, 979, 945, 897, 984, 895, 787, 921, 880, 789, 876, 858, 4964]`; ink px ref/ours 6059/6022 (ratio 0.9939); SSIM blocks <0.9: 933/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-16.5, -20.0] pt by ink-projection correlation (centroid estimate [-18.7, -27.57] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3615, differing 0.009827, SSIM₈ 0.9752 (raw 1.4784, 0.01044, 0.9703)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9525→0.9604 / 2.3628→2.1761; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3714→0.2291 / 28.2→39.414 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `list.` dx 4.92 dy -54.39; `the` dx 2.35 dy -54.39; `After` dx 0 dy -54.39

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921391, 1359, 1196, 1070, 1012, 1035, 943, 1037, 1000, 852, 821, 962, 815, 753, 768, 3802]`; ink px ref/ours 4803/6022 (ratio 1.2538); SSIM blocks <0.9: 956/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-17.0, -20.0] pt by ink-projection correlation (centroid estimate [-20.64, -27.5] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1995, differing 0.009225, SSIM₈ 0.9761 (raw 1.3304, 0.009982, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9619 / 2.1264→1.9172; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4897→0.3922 / 22.1151→32.6993 [109.1,139.7–273.7,184 pt]
- largest word displacements (pt): `list.` dx 0.01 dy -55.15; `After` dx 0 dy -55.15; `the` dx 0.0 dy -55.15

### 11-nested-lists — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920969, 1439, 1023, 930, 793, 1108, 857, 760, 794, 725, 868, 911, 886, 817, 931, 5005]`; ink px ref/ours 6062/5958 (ratio 0.9828); SSIM blocks <0.9: 995/30294; [overlay](images/11-nested-lists/pdflatex-de1020c-export-p1-overlay.png) (42629 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-de1020c-export-p1-heatmap.png) (43704 B, ÷2)
  - registration error (diagnostic): global shift [19.0, -49.0] pt by ink-projection correlation (centroid estimate [102.36, -62.51] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.4558, differing 0.010208, SSIM₈ 0.9711 (raw 1.4657, 0.01038, 0.9698)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9518→0.9553 / 2.3427→2.2545; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.506→0.3995 / 22.5175→28.0218 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `numbered` dx 309.05 dy -72.76; `First` dx 308.96 dy -72.76; `list.` dx 272.83 dy -126.61
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '•' (U+2022) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920641, 1429, 1080, 1041, 841, 1072, 967, 816, 809, 843, 926, 899, 807, 772, 916, 4957]`; ink px ref/ours 6062/6125 (ratio 1.0104); SSIM blocks <0.9: 935/30294; [overlay](images/11-nested-lists/pdflatex-main-export-p1-overlay.png) (43102 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-main-export-p1-heatmap.png) (43720 B, ÷2)
  - registration error (diagnostic): global shift [-20.0, -8.0] pt by ink-projection correlation (centroid estimate [-23.64, -10.08] pt); confidence moderate (shift explains 19% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1988, differing 0.009175, SSIM₈ 0.9789 (raw 1.4717, 0.010475, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9523→0.9663 / 2.3521→1.916; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.2523→0.34 / 34.7601→36.8114 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `First` dx -43.02 dy -11.56; `numbered` dx -42.93 dy -11.56; `Second` dx -43.02 dy -10.59

### 11-nested-lists — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920959, 1469, 995, 913, 887, 1011, 883, 901, 877, 847, 867, 895, 780, 812, 911, 4809]`; ink px ref/ours 6062/6022 (ratio 0.9934); SSIM blocks <0.9: 916/30294; [overlay](images/11-nested-lists/pdflatex-pipeline-export-p1-overlay.png) (43435 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-pipeline-export-p1-heatmap.png) (43469 B, ÷2)
  - registration error (diagnostic): global shift [-16.0, -20.0] pt by ink-projection correlation (centroid estimate [-17.39, -27.85] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3661, differing 0.009829, SSIM₈ 0.9757 (raw 1.4478, 0.010309, 0.9709)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9535→0.9611 / 2.3139→2.1834; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4616→0.2937 / 26.395→38.5846 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `list.` dx 4.91 dy -54.39; `the` dx 2.35 dy -54.39; `After` dx 0 dy -54.39

### 11-nested-lists — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921429, 1208, 1106, 1054, 881, 1201, 884, 878, 1059, 686, 919, 983, 802, 823, 761, 4142]`; ink px ref/ours 4790/5958 (ratio 1.2438); SSIM blocks <0.9: 1048/30294; [overlay](images/11-nested-lists/pdflatex-lm-de1020c-export-p1-overlay.png) (43103 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-de1020c-export-p1-heatmap.png) (43247 B, ÷2)
  - registration error (diagnostic): global shift [23.5, -49.0] pt by ink-projection correlation (centroid estimate [101.54, -62.0] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3216, differing 0.009723, SSIM₈ 0.9706 (raw 1.3674, 0.010055, 0.9687)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.95→0.9547 / 2.1855→2.0271; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6045→0.4278 / 14.6672→25.9327 [108.1,139.7–272.5,184 pt]
- largest word displacements (pt): `First` dx 308.96 dy -73.53; `numbered` dx 305.46 dy -73.53; `After` dx 272.65 dy -127.37
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '•' (U+2022) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921134, 1212, 1169, 1161, 937, 1162, 1004, 946, 1071, 802, 959, 953, 721, 766, 730, 4089]`; ink px ref/ours 4790/6125 (ratio 1.2787); SSIM blocks <0.9: 995/30294; [overlay](images/11-nested-lists/pdflatex-lm-main-export-p1-overlay.png) (43645 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-main-export-p1-heatmap.png) (43549 B, ÷2)
  - registration error (diagnostic): global shift [-29.0, -8.0] pt by ink-projection correlation (centroid estimate [-24.46, -9.57] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2361, differing 0.009335, SSIM₈ 0.9758 (raw 1.3674, 0.010152, 0.969)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9505→0.9612 / 2.1855→1.9757; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3409→0.4892 / 30.9535→27.9135 [108.1,139.7–272.5,184 pt]
- largest word displacements (pt): `sub-item.` dx -49.84 dy -12.33; `sub-item.` dx -47.67 dy -11.36; `numbered` dx -46.52 dy -12.33

### 11-nested-lists — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921335, 1309, 1178, 1113, 1018, 1088, 924, 1043, 1099, 793, 881, 924, 683, 756, 744, 3928]`; ink px ref/ours 4790/6022 (ratio 1.2572); SSIM blocks <0.9: 960/30294; [overlay](images/11-nested-lists/pdflatex-lm-pipeline-export-p1-overlay.png) (43740 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-pipeline-export-p1-heatmap.png) (42949 B, ÷2)
  - registration error (diagnostic): global shift [-17.0, -20.0] pt by ink-projection correlation (centroid estimate [-18.21, -27.34] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2, differing 0.009223, SSIM₈ 0.976 (raw 1.3363, 0.010011, 0.9701)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9522→0.9616 / 2.1358→1.9179; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.47→0.3837 / 22.7412→33.2109 [108.1,139.7–272.5,184 pt]
- largest word displacements (pt): `After` dx 0 dy -55.15; `the` dx 0.0 dy -55.15; `list.` dx -0.0 dy -55.15

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920540, 1440, 1081, 991, 989, 931, 882, 1001, 903, 758, 906, 888, 799, 893, 867, 4947]`; ink px ref/ours 6057/6022 (ratio 0.9942); SSIM blocks <0.9: 933/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-16.5, -20.0] pt by ink-projection correlation (centroid estimate [-18.65, -27.54] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.361, differing 0.009828, SSIM₈ 0.9752 (raw 1.4782, 0.010441, 0.9703)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9525→0.9604 / 2.3626→2.1753; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3713→0.2289 / 28.1096→39.2908 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `list.` dx 4.91 dy -54.39; `the` dx 2.35 dy -54.39; `After` dx 0 dy -54.39

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921379, 1371, 1183, 1088, 1014, 1009, 952, 1059, 988, 848, 826, 966, 808, 758, 753, 3814]`; ink px ref/ours 4790/6022 (ratio 1.2572); SSIM blocks <0.9: 956/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-17.0, -20.0] pt by ink-projection correlation (centroid estimate [-20.82, -27.57] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1997, differing 0.009219, SSIM₈ 0.9761 (raw 1.3305, 0.009978, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9619 / 2.1265→1.9174; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4493→0.3419 / 21.5142→33.5953 [109.1,143.5–273.7,182.9 pt]
- largest word displacements (pt): `list.` dx 0.01 dy -54.13; `After` dx 0 dy -54.13; `the` dx 0.0 dy -54.13

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1768408, 14384, 11989, 10890, 10189, 9368, 9105, 9173, 8916, 8785, 8260, 8461, 7663, 8024, 7989, 37212]`; ink px ref/ours 67719/67185 (ratio 0.9921); SSIM blocks <0.9: 7220/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 14.5] pt by ink-projection correlation (centroid estimate [-2.13, 25.25] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 12.9455, differing 0.098728, SSIM₈ 0.7922 (raw 12.969, 0.09874, 0.7908)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.666→0.6808 / 20.7261→20.0198; header-band 1.0→0.914 / 0.0→4.6025; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 58.19; `oak` dx -415.78 dy 43.74; `branch` dx -413.6 dy 58.19

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1812418, 14567, 11640, 11333, 10796, 10018, 8952, 8503, 7141, 6678, 5336, 5188, 4639, 4442, 3784, 13381]`; ink px ref/ours 51487/67185 (ratio 1.3049); SSIM blocks <0.9: 5388/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.05, -0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.8017, differing 0.076628, SSIM₈ 0.8918 (raw 7.8017, 0.076628, 0.8918)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8273→0.8273 / 12.4677→12.4677; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `jumps` dx 0.01 dy -0.36; `over` dx 0.01 dy -0.36; `the` dx 0.01 dy -0.36

### 12-justified-paragraphs — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1744770, 14653, 12288, 11028, 10978, 10548, 10474, 9663, 9503, 10351, 9533, 9493, 9214, 8786, 9639, 47895]`; ink px ref/ours 67213/67475 (ratio 1.0039); SSIM blocks <0.9: 8223/30294; [overlay](images/12-justified-paragraphs/pdflatex-de1020c-export-p1-overlay.png) (82985 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-de1020c-export-p1-heatmap.png) (63660 B, ÷4)
  - registration error (diagnostic): global shift [-3.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.27, 32.27] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.3659, differing 0.110805, SSIM₈ 0.7414 (raw 15.3708, 0.11076, 0.7415)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5868→0.5868 / 24.5671→24.5592; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 75.37; `branch` dx -435.47 dy 55.2; `branch` dx -435.47 dy 35.03

### 12-justified-paragraphs — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1744770, 14653, 12288, 11028, 10978, 10548, 10474, 9663, 9503, 10351, 9533, 9493, 9214, 8786, 9639, 47895]`; ink px ref/ours 67213/67475 (ratio 1.0039); SSIM blocks <0.9: 8223/30294; [overlay](images/12-justified-paragraphs/pdflatex-main-export-p1-overlay.png) (82886 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-main-export-p1-heatmap.png) (63557 B, ÷4)
  - registration error (diagnostic): global shift [-3.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.27, 32.27] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.3659, differing 0.110805, SSIM₈ 0.7414 (raw 15.3708, 0.11076, 0.7415)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5868→0.5868 / 24.5671→24.5592; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 75.37; `branch` dx -435.47 dy 55.2; `branch` dx -435.47 dy 35.03

### 12-justified-paragraphs — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1768237, 14151, 11981, 10636, 10509, 9453, 8881, 9247, 8652, 8589, 8455, 8706, 7785, 7668, 8237, 37629]`; ink px ref/ours 67213/67185 (ratio 0.9996); SSIM blocks <0.9: 7184/30294; [overlay](images/12-justified-paragraphs/pdflatex-pipeline-export-p1-overlay.png) (82082 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-pipeline-export-p1-heatmap.png) (57534 B, ÷4)
  - registration error (diagnostic): global shift [1.0, 14.5] pt by ink-projection correlation (centroid estimate [-1.83, 25.23] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 12.9203, differing 0.098407, SSIM₈ 0.7936 (raw 13.0297, 0.098624, 0.7911)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6665→0.6845 / 20.8232→19.9565; header-band 1.0→0.9141 / 0.0→4.6025; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 58.19; `oak` dx -415.72 dy 43.74; `branch` dx -413.61 dy 58.19

### 12-justified-paragraphs — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755333, 14131, 12262, 11805, 11161, 10699, 11419, 10985, 10442, 10508, 9029, 8874, 8613, 7950, 8550, 37055]`; ink px ref/ours 51535/67475 (ratio 1.3093); SSIM blocks <0.9: 7894/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-de1020c-export-p1-overlay.png) (84590 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-de1020c-export-p1-heatmap.png) (62747 B, ÷4)
  - registration error (diagnostic): global shift [-8.5, 2.5] pt by ink-projection correlation (centroid estimate [-6.35, 7.1] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7701, differing 0.105126, SSIM₈ 0.7503 (raw 13.7992, 0.105148, 0.7503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6012→0.6012 / 22.0542→22.0076; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.0; `oak` dx 426.94 dy 3.45

### 12-justified-paragraphs — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755333, 14131, 12262, 11805, 11161, 10699, 11419, 10985, 10442, 10508, 9029, 8874, 8613, 7950, 8550, 37055]`; ink px ref/ours 51535/67475 (ratio 1.3093); SSIM blocks <0.9: 7894/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-main-export-p1-overlay.png) (84467 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-main-export-p1-heatmap.png) (62623 B, ÷4)
  - registration error (diagnostic): global shift [-8.5, 2.5] pt by ink-projection correlation (centroid estimate [-6.35, 7.1] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7701, differing 0.105126, SSIM₈ 0.7503 (raw 13.7992, 0.105148, 0.7503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6012→0.6012 / 22.0542→22.0076; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.0; `oak` dx 426.94 dy 3.45

### 12-justified-paragraphs — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1812192, 14403, 11858, 11505, 10596, 10109, 8910, 8620, 7186, 6472, 5479, 5180, 4646, 4342, 3941, 13377]`; ink px ref/ours 51535/67185 (ratio 1.3037); SSIM blocks <0.9: 5401/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-pipeline-export-p1-overlay.png) (80119 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-pipeline-export-p1-heatmap.png) (49773 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.91, 0.06] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.8191, differing 0.076681, SSIM₈ 0.8916 (raw 7.8191, 0.076681, 0.8916)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8269→0.8269 / 12.4954→12.4954; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `falls` dx 0.02 dy 0.67; `the` dx 0.02 dy 0.67; `patient` dx -0.02 dy 0.67

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1767742, 14026, 12251, 10906, 9970, 9613, 9110, 9370, 8889, 8825, 8266, 8515, 7821, 7865, 7993, 37654]`; ink px ref/ours 67565/67185 (ratio 0.9944); SSIM blocks <0.9: 7229/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 14.5] pt by ink-projection correlation (centroid estimate [-0.82, 25.48] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 12.98, differing 0.098773, SSIM₈ 0.792 (raw 13.0485, 0.098993, 0.7899)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6645→0.6811 / 20.8532→20.0697; header-band 1.0→0.9144 / 0.0→4.6025; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 58.19; `oak` dx -415.83 dy 43.74; `branch` dx -413.65 dy 58.19

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1812466, 14515, 11670, 11274, 10741, 10083, 8960, 8563, 7077, 6715, 5330, 5172, 4622, 4456, 3792, 13380]`; ink px ref/ours 51522/67185 (ratio 1.304); SSIM blocks <0.9: 5385/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.16, -0.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.8017, differing 0.076582, SSIM₈ 0.8918 (raw 7.8017, 0.076582, 0.8918)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8273→0.8273 / 12.4677→12.4677; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `jumps` dx 0.01 dy 0.67; `over` dx 0.01 dy 0.67; `the` dx 0.01 dy 0.67

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

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [239.44, 701.731, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1930970, 657, 520, 457, 366, 482, 657, 535, 530, 336, 350, 328, 369, 378, 396, 1485]`; ink px ref/ours 2479/2822 (ratio 1.1384); SSIM blocks <0.9: 409/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [5.5, 1.5] pt by ink-projection correlation (centroid estimate [23.46, -0.02] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5737, differing 0.004497, SSIM₈ 0.9877 (raw 0.582, 0.00459, 0.9871)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9793→0.9807 / 0.9302→0.8998; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3719→0.4136 / 23.2555→21.1424 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -82.25 Δy 8.0 len 30.5 vs 12.0, thickness px 1 vs 1; Δx 40.25 Δy -5.25 len 33.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `1` dx -27.25 dy 2.15; `+` dx -27.12 dy 2.15; `∞` dx -19.71 dy -5.98
- word-sequence differences: replace ref ['1', '0'] ours ['∫']; replace ref ['√x'] ours ['0', '1', '√', '?']; replace ref ['x2', 'dx='] ours ['?', '2', ',', '??', '=']

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

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [239.44, 701.731, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1930971, 752, 591, 460, 465, 446, 456, 586, 559, 414, 530, 328, 304, 320, 393, 1241]`; ink px ref/ours 2285/2822 (ratio 1.235); SSIM blocks <0.9: 418/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [3.5, 1.5] pt by ink-projection correlation (centroid estimate [1.9, 0.39] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4986, differing 0.004198, SSIM₈ 0.9891 (raw 0.5591, 0.00452, 0.9872)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9795→0.9828 / 0.8936→0.7868; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3782→0.4413 / 23.043→20.6144 [232.8,84.9–384.7,123.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -27.75 Δy 1.5 len 30.5 vs 31.0, thickness px 1 vs 1; Δx 4.75 Δy 1.5 len 33.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `1` dx -27.25 dy 2.3; `+` dx -27.12 dy 2.3; `∞` dx -19.71 dy -5.84
- word-sequence differences: replace ref ['1', '0'] ours ['∫']; replace ref ['√x'] ours ['0', '1', '√', '?']; replace ref ['x2', 'dx='] ours ['?', '2', ',', '??', '=']

### 13-math-display-rich — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931825, 678, 566, 390, 402, 560, 481, 438, 376, 334, 290, 257, 295, 302, 330, 1292]`; ink px ref/ours 2470/2605 (ratio 1.0547); SSIM blocks <0.9: 320/30294; [overlay](images/13-math-display-rich/pdflatex-de1020c-export-p1-overlay.png) (82961 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-de1020c-export-p1-heatmap.png) (81191 B, ÷1)
  - registration error (diagnostic): global shift [1.0, 3.5] pt by ink-projection correlation (centroid estimate [0.39, 2.75] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4808, differing 0.004039, SSIM₈ 0.9907 (raw 0.501, 0.004125, 0.9899)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9839→0.9853 / 0.8007→0.767; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4017→0.3792 / 25.1214→25.5101 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -66.25 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2; Δx 18.75 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `display:` dx 0.1 dy -10.42; `Rich` dx 0 dy -10.42; `√x` dx -7.41 dy 4.96
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (3): error: \left is not supported in math mode; error: \right is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931658, 686, 587, 398, 417, 568, 488, 447, 386, 340, 298, 260, 301, 307, 342, 1333]`; ink px ref/ours 2470/2696 (ratio 1.0915); SSIM blocks <0.9: 331/30294; [overlay](images/13-math-display-rich/pdflatex-main-export-p1-overlay.png) (83195 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-main-export-p1-heatmap.png) (81483 B, ÷1)
  - registration error (diagnostic): global shift [1.0, 3.5] pt by ink-projection correlation (centroid estimate [11.26, 2.59] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4936, differing 0.004134, SSIM₈ 0.9903 (raw 0.5138, 0.004221, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9833→0.9847 / 0.8212→0.7875; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4017→0.3792 / 25.1214→25.5101 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -66.25 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2; Δx 18.75 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `display:` dx 0.1 dy -10.42; `Rich` dx 0 dy -10.42; `√x` dx -7.41 dy 4.96
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [239.44, 701.731, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1930995, 647, 509, 406, 421, 567, 513, 527, 555, 331, 369, 337, 348, 346, 435, 1510]`; ink px ref/ours 2470/2822 (ratio 1.1425); SSIM blocks <0.9: 405/30294; [overlay](images/13-math-display-rich/pdflatex-pipeline-export-p1-overlay.png) (83481 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-pipeline-export-p1-heatmap.png) (83059 B, ÷1)
  - registration error (diagnostic): global shift [4.0, 1.5] pt by ink-projection correlation (centroid estimate [24.05, 0.09] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5498, differing 0.004432, SSIM₈ 0.9881 (raw 0.5855, 0.004578, 0.9871)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9793→0.9811 / 0.9358→0.8678; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.372→0.4381 / 23.2576→20.2166 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -82.25 Δy 8.0 len 30.5 vs 12.0, thickness px 1 vs 1; Δx 40.25 Δy -5.25 len 33.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `1` dx -27.25 dy 2.15; `+` dx -27.12 dy 2.15; `∞` dx -19.71 dy -5.99
- word-sequence differences: replace ref ['1', '0'] ours ['∫']; replace ref ['√x'] ours ['0', '1', '√', '?']; replace ref ['x2', 'dx='] ours ['?', '2', ',', '??', '=']

### 13-math-display-rich — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931353, 583, 548, 467, 406, 416, 441, 540, 508, 387, 355, 345, 457, 339, 349, 1322]`; ink px ref/ours 2283/2605 (ratio 1.141); SSIM blocks <0.9: 363/30294; [overlay](images/13-math-display-rich/pdflatex-lm-de1020c-export-p1-overlay.png) (83715 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-de1020c-export-p1-heatmap.png) (82698 B, ÷1)
  - registration error (diagnostic): global shift [-2.0, 4.0] pt by ink-projection correlation (centroid estimate [-23.81, 3.31] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5406, differing 0.004269, SSIM₈ 0.9893 (raw 0.553, 0.004302, 0.9888)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.982→0.983 / 0.8838→0.8638; header-band 1.0→0.9992 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4037→0.4126 / 25.0041→24.5413 [232.8,84.8–384.7,123 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -11.5 Δy 4.0 len 21.0 vs 32.0, thickness px 1 vs 1; Δx -16.75 Δy 4.0 len 21.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `display:` dx -1.48 dy -10.16; `Rich` dx 0 dy -10.16; `√x` dx -7.41 dy 5.22
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (3): error: \left is not supported in math mode; error: \right is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931186, 591, 569, 475, 421, 424, 448, 549, 518, 393, 363, 348, 463, 344, 361, 1363]`; ink px ref/ours 2283/2696 (ratio 1.1809); SSIM blocks <0.9: 374/30294; [overlay](images/13-math-display-rich/pdflatex-lm-main-export-p1-overlay.png) (83779 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-main-export-p1-heatmap.png) (82824 B, ÷1)
  - registration error (diagnostic): global shift [-2.0, 4.0] pt by ink-projection correlation (centroid estimate [-12.95, 3.15] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5534, differing 0.004364, SSIM₈ 0.989 (raw 0.5658, 0.004398, 0.9884)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9815→0.9826 / 0.9043→0.8793; header-band 1.0→0.9992 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4037→0.4126 / 25.0041→24.5413 [232.8,84.8–384.7,123 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -11.5 Δy 4.0 len 21.0 vs 32.0, thickness px 1 vs 1; Δx -16.75 Δy 4.0 len 21.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `display:` dx -1.48 dy -10.16; `Rich` dx 0 dy -10.16; `√x` dx -7.41 dy 5.22
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [239.44, 701.731, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931079, 623, 590, 487, 440, 450, 433, 586, 621, 356, 391, 342, 462, 322, 390, 1244]`; ink px ref/ours 2283/2822 (ratio 1.2361); SSIM blocks <0.9: 418/30294; [overlay](images/13-math-display-rich/pdflatex-lm-pipeline-export-p1-overlay.png) (84081 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-pipeline-export-p1-heatmap.png) (83444 B, ÷1)
  - registration error (diagnostic): global shift [3.5, 1.5] pt by ink-projection correlation (centroid estimate [-0.15, 0.65] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5085, differing 0.004217, SSIM₈ 0.9889 (raw 0.5614, 0.004485, 0.9872)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9795→0.9825 / 0.8972→0.8027; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3793→0.4385 / 23.3686→21.0828 [232.8,84.8–384.7,123 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -27.75 Δy 1.5 len 30.5 vs 32.0, thickness px 1 vs 1; Δx 4.75 Δy 1.5 len 33.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `1` dx -27.25 dy 2.41; `+` dx -27.12 dy 2.41; `∞` dx -19.71 dy -5.73
- word-sequence differences: replace ref ['1', '0'] ours ['∫']; replace ref ['√x'] ours ['0', '1', '√', '?']; replace ref ['x2', 'dx='] ours ['?', '2', ',', '??', '=']

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

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [239.44, 701.731, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1930914, 669, 530, 442, 366, 473, 647, 555, 546, 330, 381, 343, 380, 374, 383, 1483]`; ink px ref/ours 2475/2822 (ratio 1.1402); SSIM blocks <0.9: 409/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.5, 1.5] pt by ink-projection correlation (centroid estimate [23.22, -0.06] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5572, differing 0.004462, SSIM₈ 0.9879 (raw 0.5861, 0.004598, 0.987)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9792→0.9808 / 0.9367→0.8876; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5686→0.6017 / 16.3424→14.924 [227.3,75.6–384.7,129 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -82.25 Δy 8.0 len 30.5 vs 12.0, thickness px 1 vs 1; Δx 40.25 Δy -5.25 len 33.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `0` dx -29.64 dy 5.24; `1` dx -27.25 dy 2.15; `+` dx -27.12 dy 2.15
- word-sequence differences: insert ref [] ours ['∫']; delete ref ['∫1'] ours []; replace ref ['√x'] ours ['1', '√', '?']

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

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [239.44, 701.731, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1930967, 755, 590, 462, 467, 445, 453, 587, 558, 409, 536, 330, 303, 322, 390, 1242]`; ink px ref/ours 2286/2822 (ratio 1.2345); SSIM blocks <0.9: 418/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [3.5, 1.5] pt by ink-projection correlation (centroid estimate [1.9, 0.33] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4987, differing 0.0042, SSIM₈ 0.9891 (raw 0.5593, 0.00452, 0.9872)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9795→0.9828 / 0.8939→0.7869; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.559→0.5957 / 16.3873→15.0108 [227.3,75.4–384.7,128.9 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -27.75 Δy 1.5 len 30.5 vs 31.0, thickness px 1 vs 1; Δx 4.75 Δy 1.5 len 33.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `0` dx -29.64 dy 5.38; `1` dx -27.25 dy 2.3; `+` dx -27.12 dy 2.3
- word-sequence differences: insert ref [] ours ['∫']; delete ref ['∫1'] ours []; replace ref ['√x'] ours ['1', '√', '?']

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

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 14.08pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [235.512, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924764, 1392, 1126, 911, 947, 833, 781, 861, 748, 816, 651, 871, 657, 633, 582, 2243]`; ink px ref/ours 4839/4873 (ratio 1.007); SSIM blocks <0.9: 672/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [4.0, 1.5] pt REJECTED: applying it gives mean|Δ| 1.0539, not lower; centroid estimate [2.49, 0.29] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9935, differing 0.008271, SSIM₈ 0.9797 (raw 0.9935, 0.008271, 0.9797)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9682→0.9682 / 1.5713→1.5713; header-band 0.9986→0.9986 / 0.0547→0.0547; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4334→0.4334 / 22.6848→22.6848 [96.4,69.7–279.9,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 22.25 Δy 1.5 len 28.0 vs 27.5, thickness px 1 vs 1; Δx -16.75 Δy 7.0 len 14.0 vs 27.5, thickness px 1 vs 1
- largest word displacements (pt): `,` dx -271.95 dy 18.42; `,` dx -56.0 dy 16.32; `must` dx 39.81 dy 2.06
- word-sequence differences: replace ref ['a2'] ours ['?2']; replace ref ['b2', 'p1,', 'σ2'] ours ['?2', '=', '?2', '?2']; replace ref ['πr2'] ours ['?1,', '?', '2']

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

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 14.08pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [235.512, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1925027, 1323, 1075, 1007, 1013, 869, 808, 866, 905, 827, 632, 825, 578, 560, 561, 1940]`; ink px ref/ours 4245/4873 (ratio 1.1479); SSIM blocks <0.9: 678/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 1.5] pt by ink-projection correlation (centroid estimate [7.25, -0.09] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8319, differing 0.007563, SSIM₈ 0.9817 (raw 0.9471, 0.008035, 0.9801)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9687→0.9715 / 1.4972→1.3106; header-band 0.9986→0.9976 / 0.0547→0.0719; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3339→0.4231 / 23.83→21.2882 [176,69.7–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -0.5 Δy 1.5 len 28.0 vs 27.0, thickness px 1 vs 1; Δx -0.75 Δy 1.5 len 14.0 vs 14.5, thickness px 1 vs 1
- largest word displacements (pt): `,` dx 61.8 dy 0.89; `,` dx -0.09 dy 18.68; `+` dx 13.12 dy 2.28
- word-sequence differences: replace ref ['a2'] ours ['?2']; replace ref ['b2', 'z2'] ours ['?2', '=', '?2', '?2']; replace ref ['p1,', 'σ2'] ours ['?1,', '?', '2']

### 14-math-inline-dense — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921723, 1256, 1319, 999, 1015, 858, 878, 898, 911, 966, 857, 1132, 834, 737, 861, 3572]`; ink px ref/ours 4824/5720 (ratio 1.1857); SSIM blocks <0.9: 948/30294; [overlay](images/14-math-inline-dense/pdflatex-de1020c-export-p1-overlay.png) (45660 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-de1020c-export-p1-heatmap.png) (44432 B, ÷2)
  - registration error (diagnostic): global shift [-34.0, 13.5] pt by ink-projection correlation (centroid estimate [-13.94, 7.12] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9608, differing 0.008388, SSIM₈ 0.9783 (raw 1.3129, 0.009906, 0.9708)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9533→0.9734 / 2.0983→1.1478; header-band 1.0→0.9469 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3428→0.3554 / 26.6779→25.5089 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -407.82 dy 29.13; `,` dx -87.71 dy 29.13; `,` dx 84.05 dy -10.04
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours ['a2+b2', '=c2']; insert ref [] ours ['α', 'βγ,', ',', '√2', ',', 'xj', 'i', ',']

### 14-math-inline-dense — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921723, 1256, 1319, 999, 1015, 858, 878, 898, 911, 966, 857, 1132, 834, 737, 861, 3572]`; ink px ref/ours 4824/5720 (ratio 1.1857); SSIM blocks <0.9: 948/30294; [overlay](images/14-math-inline-dense/pdflatex-main-export-p1-overlay.png) (44933 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-main-export-p1-heatmap.png) (43745 B, ÷2)
  - registration error (diagnostic): global shift [-34.0, 13.5] pt by ink-projection correlation (centroid estimate [-13.94, 7.12] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9608, differing 0.008388, SSIM₈ 0.9783 (raw 1.3129, 0.009906, 0.9708)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9533→0.9734 / 2.0983→1.1478; header-band 1.0→0.9469 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3428→0.3554 / 26.6779→25.5089 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -407.82 dy 29.13; `,` dx -87.71 dy 29.13; `,` dx 84.05 dy -10.04
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours ['a2+b2', '=c2']; insert ref [] ours ['α', 'βγ,', ',', '√2', ',', 'xj', 'i', ',']

### 14-math-inline-dense — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 14.08pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [235.512, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1925067, 1312, 1093, 881, 953, 804, 751, 786, 703, 752, 658, 907, 660, 657, 601, 2231]`; ink px ref/ours 4824/4873 (ratio 1.0102); SSIM blocks <0.9: 674/30294; [overlay](images/14-math-inline-dense/pdflatex-pipeline-export-p1-overlay.png) (44410 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-pipeline-export-p1-heatmap.png) (40045 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [4.0, 1.5] pt REJECTED: applying it gives mean|Δ| 1.0535, not lower; centroid estimate [2.97, 0.25] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.983, differing 0.008186, SSIM₈ 0.9799 (raw 0.983, 0.008186, 0.9799)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9684→0.9684 / 1.5546→1.5546; header-band 0.9986→0.9986 / 0.0547→0.0547; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4346→0.4346 / 22.5256→22.5256 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 22.25 Δy 1.5 len 28.0 vs 27.5, thickness px 1 vs 1; Δx -16.75 Δy 7.0 len 14.0 vs 27.5, thickness px 1 vs 1
- largest word displacements (pt): `,` dx 83.73 dy 0.89; `,` dx -55.72 dy 16.32; `,` dx -51.73 dy 3.99
- word-sequence differences: replace ref ['a2'] ours ['?2']; replace ref ['b2', 'p1,', 'σ2'] ours ['?2', '=', '?2', '?2']; replace ref ['πr2'] ours ['?1,', '?', '2']

### 14-math-inline-dense — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922114, 1292, 1255, 994, 1030, 864, 932, 987, 1070, 1032, 804, 1077, 790, 689, 773, 3113]`; ink px ref/ours 4254/5720 (ratio 1.3446); SSIM blocks <0.9: 957/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-de1020c-export-p1-overlay.png) (45804 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-de1020c-export-p1-heatmap.png) (44320 B, ÷2)
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-7.87, 6.82] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1673, differing 0.009234, SSIM₈ 0.9735 (raw 1.2469, 0.009657, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9655 / 1.9928→1.475; header-band 1.0→0.9479 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3525→0.2973 / 24.7689→27.2474 [183.4,69.8–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -51.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `z2` dx 408.28 dy -8.86; `must` dx -73.95 dy 13.68; `that` dx -69.96 dy 13.68
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2'] ours ['a2+b2', '=c2', ',', 'α', 'βγ,', ',', '√2', ',']; insert ref [] ours ['πr2', 'a+b']

### 14-math-inline-dense — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922114, 1292, 1255, 994, 1030, 864, 932, 987, 1070, 1032, 804, 1077, 790, 689, 773, 3113]`; ink px ref/ours 4254/5720 (ratio 1.3446); SSIM blocks <0.9: 957/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-main-export-p1-overlay.png) (45207 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-main-export-p1-heatmap.png) (43751 B, ÷2)
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-7.87, 6.82] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1673, differing 0.009234, SSIM₈ 0.9735 (raw 1.2469, 0.009657, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9655 / 1.9928→1.475; header-band 1.0→0.9479 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3525→0.2973 / 24.7689→27.2474 [183.4,69.8–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -51.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `z2` dx 408.28 dy -8.86; `must` dx -73.95 dy 13.68; `that` dx -69.96 dy 13.68
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2'] ours ['a2+b2', '=c2', ',', 'α', 'βγ,', ',', '√2', ',']; insert ref [] ours ['πr2', 'a+b']

### 14-math-inline-dense — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 14.08pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [235.512, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1925027, 1308, 1099, 978, 997, 880, 822, 865, 920, 814, 655, 815, 562, 583, 547, 1944]`; ink px ref/ours 4254/4873 (ratio 1.1455); SSIM blocks <0.9: 676/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-pipeline-export-p1-overlay.png) (44737 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-pipeline-export-p1-heatmap.png) (39570 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 1.5] pt by ink-projection correlation (centroid estimate [9.04, -0.05] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.832, differing 0.007562, SSIM₈ 0.9817 (raw 0.9478, 0.008032, 0.9801)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9687→0.9715 / 1.4985→1.3109; header-band 0.9986→0.9976 / 0.0547→0.0719; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.336→0.4079 / 24.4577→21.84 [183.4,69.8–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -0.5 Δy 1.5 len 28.0 vs 27.0, thickness px 1 vs 1; Δx -0.75 Δy 1.5 len 14.0 vs 14.5, thickness px 1 vs 1
- largest word displacements (pt): `,` dx 282.01 dy -13.54; `,` dx -0.09 dy 19.71; `+` dx -6.5 dy 2.28
- word-sequence differences: replace ref ['a2'] ours ['?2']; replace ref ['b2', 'z2'] ours ['?2', '=', '?2', '?2']; replace ref ['p1,', 'σ2'] ours ['?1,', '?', '2']

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

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 14.08pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [235.512, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924778, 1391, 1127, 899, 956, 830, 775, 834, 762, 819, 644, 877, 651, 640, 570, 2263]`; ink px ref/ours 4852/4873 (ratio 1.0043); SSIM blocks <0.9: 672/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [4.0, 1.5] pt REJECTED: applying it gives mean|Δ| 1.054, not lower; centroid estimate [2.25, 0.27] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9936, differing 0.008265, SSIM₈ 0.9797 (raw 0.9936, 0.008265, 0.9797)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9682→0.9682 / 1.5715→1.5715; header-band 0.9986→0.9986 / 0.0547→0.0547; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4336→0.4336 / 22.6714→22.6714 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 22.25 Δy 1.5 len 28.0 vs 27.5, thickness px 1 vs 1; Δx -16.75 Δy 7.0 len 14.0 vs 27.5, thickness px 1 vs 1
- largest word displacements (pt): `,` dx -271.95 dy 18.42; `,` dx -56.0 dy 16.32; `must` dx 39.78 dy 2.06
- word-sequence differences: replace ref ['a2'] ours ['?2']; replace ref ['b2', 'p1,', 'σ2'] ours ['?2', '=', '?2', '?2']; replace ref ['πr2'] ours ['?1,', '?', '2']

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

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 14.08pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [235.512, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1925030, 1314, 1084, 1001, 1018, 869, 803, 869, 916, 817, 635, 822, 580, 559, 560, 1939]`; ink px ref/ours 4244/4873 (ratio 1.1482); SSIM blocks <0.9: 678/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 1.5] pt by ink-projection correlation (centroid estimate [7.32, -0.09] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8319, differing 0.007564, SSIM₈ 0.9817 (raw 0.9471, 0.008035, 0.9801)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9687→0.9715 / 1.4972→1.3106; header-band 0.9986→0.9976 / 0.0547→0.0719; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.338→0.4096 / 24.4817→21.889 [183.4,69.7–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -0.5 Δy 1.5 len 28.0 vs 27.0, thickness px 1 vs 1; Δx -0.75 Δy 1.5 len 14.0 vs 14.5, thickness px 1 vs 1
- largest word displacements (pt): `,` dx 61.8 dy 0.89; `+` dx 13.12 dy -14.94; `,` dx -0.09 dy 19.71
- word-sequence differences: replace ref ['a2'] ours ['?2']; replace ref ['b2', 'z2'] ours ['?2', '=', '?2', '?2']; replace ref ['p1,', 'σ2'] ours ['?1,', '?', '2']

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

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1633257, 24504, 20395, 19324, 17476, 17703, 16927, 16921, 16416, 16378, 15662, 15273, 14347, 14339, 14946, 64948]`; ink px ref/ours 112785/122171 (ratio 1.0832); SSIM blocks <0.9: 12453/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 14.0] pt by ink-projection correlation (centroid estimate [-5.62, 56.55] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 22.4214, differing 0.17125, SSIM₈ 0.6465 (raw 23.3326, 0.175166, 0.6284)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4071→0.4381 / 37.2838→35.7182; header-band 1.0→0.9856 / 0.0→0.7483; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1599409, 26160, 21742, 20282, 19189, 18572, 18002, 17880, 17502, 17667, 17373, 16785, 16562, 16428, 16947, 78316]`; ink px ref/ours 112804/128654 (ratio 1.1405); SSIM blocks <0.9: 14184/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -26.5] pt by ink-projection correlation (centroid estimate [-3.73, 51.98] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.5529, differing 0.178706, SSIM₈ 0.6256 (raw 26.5531, 0.194556, 0.5522)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2853→0.43 / 42.431→36.3; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8214 / 0.0→9.1359
- page 3: |Δ| histogram (16 bins, pixel counts) `[1659366, 21539, 18426, 16729, 15515, 15484, 14774, 14875, 14431, 14528, 13950, 13930, 13465, 13616, 13739, 64449]`; ink px ref/ours 112799/86107 (ratio 0.7634); SSIM blocks <0.9: 11571/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, -55.5] pt by ink-projection correlation (centroid estimate [-6.03, -53.48] pt); confidence moderate (shift explains 14% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 18.8352, differing 0.144767, SSIM₈ 0.7082 (raw 21.8231, 0.160263, 0.6342)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4162→0.5353 / 34.8718→30.0913; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy -113.01; `branch` dx -413.6 dy -113.01; `oak` dx -415.78 dy -98.57
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

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

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1709517, 26006, 20588, 19862, 18255, 18152, 16166, 15519, 12673, 11713, 10043, 9446, 8322, 7929, 7632, 26993]`; ink px ref/ours 85641/122171 (ratio 1.4265); SSIM blocks <0.9: 9954/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.42, 29.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 14.4676, differing 0.138075, SSIM₈ 0.7902 (raw 14.4676, 0.138075, 0.7902)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6651→0.6651 / 23.1211→23.1211; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606496, 26618, 21670, 21258, 20293, 19308, 19258, 20466, 19484, 17477, 16700, 16285, 15072, 14839, 15683, 67909]`; ink px ref/ours 85678/128654 (ratio 1.5016); SSIM blocks <0.9: 14925/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -26.5] pt by ink-projection correlation (centroid estimate [2.38, 24.58] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 22.5689, differing 0.176444, SSIM₈ 0.6019 (raw 25.0566, 0.19117, 0.5284)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2469→0.39 / 40.0443→34.7408; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8243 / 0.0→9.1359
- page 3: |Δ| histogram (16 bins, pixel counts) `[1667318, 22081, 18413, 17199, 16829, 15833, 16496, 17571, 16353, 14515, 13445, 13464, 12409, 12122, 12772, 51996]`; ink px ref/ours 85675/86107 (ratio 1.005); SSIM blocks <0.9: 12370/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -26.5] pt by ink-projection correlation (centroid estimate [0.08, -80.94] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 17.9989, differing 0.143458, SSIM₈ 0.6733 (raw 20.1836, 0.15636, 0.6074)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3731→0.4785 / 32.2564→28.7647; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `river` dx 0.05 dy -142.22; `below.` dx 0.05 dy -142.22; `river` dx 0.05 dy -142.22
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1595899, 24122, 20968, 20023, 17718, 17966, 18322, 17267, 17094, 18406, 16944, 15990, 16755, 16056, 16950, 88336]`; ink px ref/ours 112156/116938 (ratio 1.0426); SSIM blocks <0.9: 14346/30294; [overlay](images/15-three-page-sections/pdflatex-de1020c-export-p1-overlay.png) (48200 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-de1020c-export-p1-heatmap.png) (35459 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 36.5] pt by ink-projection correlation (centroid estimate [-13.64, 42.99] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.376, differing 0.183932, SSIM₈ 0.5937 (raw 27.5937, 0.195208, 0.5337)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2665→0.3542 / 43.4405→40.4605; header-band 1.0→0.9879 / 0.0→0.6238; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1592865, 24793, 21494, 20314, 18311, 18548, 19317, 17570, 17258, 18450, 17153, 16254, 16690, 16013, 17183, 86603]`; ink px ref/ours 112190/122724 (ratio 1.0939); SSIM blocks <0.9: 14238/30294; [overlay](images/15-three-page-sections/pdflatex-de1020c-export-p2-overlay.png) (49115 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-de1020c-export-p2-heatmap.png) (35590 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.72, 17.61] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.0991, differing 0.18994, SSIM₈ 0.5742 (raw 27.5896, 0.197186, 0.5428)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2693→0.3363 / 44.0962→41.0127; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8846 / 0.0→4.8261
- page 3: |Δ| histogram (16 bins, pixel counts) `[1626455, 22468, 19561, 18552, 16338, 16950, 16970, 15864, 15658, 16281, 15491, 14672, 14985, 14631, 15599, 78341]`; ink px ref/ours 112177/99417 (ratio 0.8863); SSIM blocks <0.9: 12897/30294; [overlay](images/15-three-page-sections/pdflatex-de1020c-export-p3-overlay.png) (45393 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-de1020c-export-p3-heatmap.png) (33428 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.66, -40.6] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.7997, differing 0.172056, SSIM₈ 0.6079 (raw 24.915, 0.178224, 0.5858)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.338→0.3733 / 39.8213→38.0388; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy -63.97; `branch` dx -435.47 dy -58.38; `branch` dx -435.47 dy -52.79

### 15-three-page-sections — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1595636, 24136, 20977, 20021, 17699, 17990, 18320, 17252, 17091, 18406, 16976, 15992, 16802, 16058, 16973, 88487]`; ink px ref/ours 112156/117099 (ratio 1.0441); SSIM blocks <0.9: 14364/30294; [overlay](images/15-three-page-sections/pdflatex-main-export-p1-overlay.png) (48049 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-main-export-p1-heatmap.png) (35311 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 36.5] pt by ink-projection correlation (centroid estimate [-13.87, 43.09] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.3987, differing 0.184079, SSIM₈ 0.5934 (raw 27.6238, 0.195371, 0.5331)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2656→0.3539 / 43.4885→40.4816; header-band 1.0→0.9865 / 0.0→0.7294; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1592762, 24798, 21500, 20307, 18308, 18537, 19321, 17580, 17246, 18440, 17181, 16265, 16709, 16018, 17203, 86641]`; ink px ref/ours 112190/122799 (ratio 1.0946); SSIM blocks <0.9: 14247/30294; [overlay](images/15-three-page-sections/pdflatex-main-export-p2-overlay.png) (48928 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-main-export-p2-heatmap.png) (35421 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.86, 17.76] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.1109, differing 0.190018, SSIM₈ 0.574 (raw 27.6017, 0.197255, 0.5426)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2689→0.3359 / 44.1155→41.0316; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8846 / 0.0→4.8261
- page 3: |Δ| histogram (16 bins, pixel counts) `[1626455, 22468, 19561, 18552, 16338, 16950, 16970, 15864, 15658, 16281, 15491, 14672, 14985, 14631, 15599, 78341]`; ink px ref/ours 112177/99417 (ratio 0.8863); SSIM blocks <0.9: 12897/30294; [overlay](images/15-three-page-sections/pdflatex-main-export-p3-overlay.png) (45181 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-main-export-p3-heatmap.png) (33233 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.66, -40.6] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.7997, differing 0.172056, SSIM₈ 0.6079 (raw 24.915, 0.178224, 0.5858)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.338→0.3733 / 39.8213→38.0388; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy -63.97; `branch` dx -435.47 dy -58.38; `branch` dx -435.47 dy -52.79
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1635853, 23852, 20716, 19313, 17186, 17487, 16994, 17251, 16617, 16241, 15726, 14724, 13817, 14122, 14532, 64385]`; ink px ref/ours 112156/122171 (ratio 1.0893); SSIM blocks <0.9: 12105/30294; [overlay](images/15-three-page-sections/pdflatex-pipeline-export-p1-overlay.png) (49849 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-pipeline-export-p1-heatmap.png) (84591 B, ÷4)
  - registration error (diagnostic): global shift [1.0, -0.5] pt by ink-projection correlation (centroid estimate [-5.12, 33.73] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.8975, differing 0.168707, SSIM₈ 0.6599 (raw 23.0879, 0.173852, 0.6368)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.42→0.4598 / 36.8983→34.9579; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1595248, 25436, 22168, 20001, 18813, 18801, 18194, 18772, 18224, 17743, 17444, 16334, 16241, 16406, 17536, 81455]`; ink px ref/ours 112190/128654 (ratio 1.1468); SSIM blocks <0.9: 14662/30294; [overlay](images/15-three-page-sections/pdflatex-pipeline-export-p2-overlay.png) (51423 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-pipeline-export-p2-heatmap.png) (37233 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -55.5] pt by ink-projection correlation (centroid estimate [-3.18, 29.22] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.1051, differing 0.181586, SSIM₈ 0.6082 (raw 27.0655, 0.196425, 0.5329)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2539→0.4317 / 43.2561→35.835; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.6136 / 0.0→18.4532
- page 3: |Δ| histogram (16 bins, pixel counts) `[1656765, 21028, 18469, 16794, 15421, 15593, 15189, 15276, 14926, 14468, 14024, 13522, 13098, 13653, 14050, 66540]`; ink px ref/ours 112177/86107 (ratio 0.7676); SSIM blocks <0.9: 11932/30294; [overlay](images/15-three-page-sections/pdflatex-pipeline-export-p3-overlay.png) (44815 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-pipeline-export-p3-heatmap.png) (83723 B, ÷4)
  - registration error (diagnostic): global shift [0.5, -12.5] pt by ink-projection correlation (centroid estimate [-5.48, -76.3] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.4923, differing 0.147733, SSIM₈ 0.6816 (raw 22.1353, 0.161425, 0.6211)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3947→0.4926 / 35.3769→31.1465; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy -127.53; `oak` dx -415.72 dy -127.53; `oak` dx -415.72 dy -127.53
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610609, 24967, 21053, 20726, 19661, 18340, 19742, 19412, 18598, 18249, 16287, 15994, 15576, 14410, 15675, 69517]`; ink px ref/ours 85692/116938 (ratio 1.3646); SSIM blocks <0.9: 14337/30294; [overlay](images/15-three-page-sections/pdflatex-lm-de1020c-export-p1-overlay.png) (48458 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-de1020c-export-p1-heatmap.png) (35292 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.11, 38.21] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.3365, differing 0.178246, SSIM₈ 0.5819 (raw 25.0354, 0.18832, 0.5339)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2672→0.3354 / 39.3505→37.2; header-band 1.0→0.9885 / 0.0→0.6238; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606782, 25413, 21423, 20910, 20004, 18862, 20556, 19676, 18675, 18670, 16345, 16142, 15704, 14246, 15727, 69681]`; ink px ref/ours 85729/122724 (ratio 1.4315); SSIM blocks <0.9: 14619/30294; [overlay](images/15-three-page-sections/pdflatex-lm-de1020c-export-p2-overlay.png) (49635 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-de1020c-export-p2-heatmap.png) (35905 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.23, 12.88] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.2389, differing 0.184974, SSIM₈ 0.5622 (raw 25.2323, 0.19048, 0.5336)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2548→0.3156 / 40.3276→38.0352; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9087 / 0.0→4.802
- page 3: |Δ| histogram (16 bins, pixel counts) `[1641126, 23072, 19471, 19055, 17908, 16873, 18097, 18161, 17038, 16322, 14757, 14561, 14158, 12872, 14209, 61136]`; ink px ref/ours 85728/99417 (ratio 1.1597); SSIM blocks <0.9: 13069/30294; [overlay](images/15-three-page-sections/pdflatex-lm-de1020c-export-p3-overlay.png) (45694 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-de1020c-export-p3-heatmap.png) (33523 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.13, -45.32] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.5333, differing 0.165392, SSIM₈ 0.6026 (raw 22.5169, 0.171027, 0.5804)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3295→0.3666 / 35.9875→34.4099; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -92.02; `oak` dx 426.94 dy -86.43; `oak` dx 426.94 dy -80.84

### 15-three-page-sections — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610426, 24967, 21071, 20740, 19645, 18366, 19745, 19409, 18564, 18243, 16308, 16000, 15616, 14400, 15699, 69617]`; ink px ref/ours 85692/117099 (ratio 1.3665); SSIM blocks <0.9: 14350/30294; [overlay](images/15-three-page-sections/pdflatex-lm-main-export-p1-overlay.png) (48287 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-main-export-p1-heatmap.png) (35119 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.34, 38.32] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.3593, differing 0.178393, SSIM₈ 0.5814 (raw 25.0554, 0.188444, 0.5336)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2666→0.3349 / 39.3824→37.2211; header-band 1.0→0.9867 / 0.0→0.7294; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606684, 25405, 21442, 20908, 20007, 18865, 20557, 19667, 18668, 18646, 16362, 16148, 15720, 14244, 15762, 69731]`; ink px ref/ours 85729/122799 (ratio 1.4324); SSIM blocks <0.9: 14626/30294; [overlay](images/15-three-page-sections/pdflatex-lm-main-export-p2-overlay.png) (49463 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-main-export-p2-heatmap.png) (35699 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.36, 13.02] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.2507, differing 0.185052, SSIM₈ 0.562 (raw 25.2442, 0.190554, 0.5334)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2544→0.3151 / 40.3466→38.0542; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9087 / 0.0→4.802
- page 3: |Δ| histogram (16 bins, pixel counts) `[1641126, 23072, 19471, 19055, 17908, 16873, 18097, 18161, 17038, 16322, 14757, 14561, 14158, 12872, 14209, 61136]`; ink px ref/ours 85728/99417 (ratio 1.1597); SSIM blocks <0.9: 13069/30294; [overlay](images/15-three-page-sections/pdflatex-lm-main-export-p3-overlay.png) (45473 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-main-export-p3-heatmap.png) (33322 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.13, -45.32] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.5333, differing 0.165392, SSIM₈ 0.6026 (raw 22.5169, 0.171027, 0.5804)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3295→0.3666 / 35.9875→34.4099; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -92.02; `oak` dx 426.94 dy -86.43; `oak` dx 426.94 dy -80.84
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1709369, 25806, 20973, 19785, 18421, 18190, 16126, 15512, 12487, 11595, 10184, 9358, 8312, 7947, 7810, 26941]`; ink px ref/ours 85692/122171 (ratio 1.4257); SSIM blocks <0.9: 9958/30294; [overlay](images/15-three-page-sections/pdflatex-lm-pipeline-export-p1-overlay.png) (49381 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-pipeline-export-p1-heatmap.png) (76142 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.41, 28.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 14.4862, differing 0.138241, SSIM₈ 0.7901 (raw 14.4862, 0.138241, 0.7901)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6649→0.6649 / 23.1507→23.1507; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606512, 26137, 22046, 21196, 20466, 19234, 19353, 20406, 19226, 17596, 16763, 16333, 15102, 14787, 15873, 67786]`; ink px ref/ours 85729/128654 (ratio 1.5007); SSIM blocks <0.9: 14927/30294; [overlay](images/15-three-page-sections/pdflatex-lm-pipeline-export-p2-overlay.png) (52202 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-pipeline-export-p2-heatmap.png) (37272 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -26.5] pt by ink-projection correlation (centroid estimate [2.32, 24.48] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 22.576, differing 0.176473, SSIM₈ 0.6015 (raw 25.0655, 0.191229, 0.5281)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2465→0.3894 / 40.0584→34.7521; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8243 / 0.0→9.1359
- page 3: |Δ| histogram (16 bins, pixel counts) `[1667255, 21865, 18502, 17144, 17039, 15839, 16623, 17231, 16395, 14591, 13569, 13491, 12358, 12210, 12854, 51850]`; ink px ref/ours 85728/86107 (ratio 1.0044); SSIM blocks <0.9: 12373/30294; [overlay](images/15-three-page-sections/pdflatex-lm-pipeline-export-p3-overlay.png) (45205 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-pipeline-export-p3-heatmap.png) (86709 B, ÷4)
  - registration error (diagnostic): global shift [0.0, -26.5] pt by ink-projection correlation (centroid estimate [0.05, -81.02] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 18.01, differing 0.14351, SSIM₈ 0.6728 (raw 20.1912, 0.156408, 0.6071)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3726→0.4777 / 32.2686→28.7825; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `river` dx 0.06 dy -141.18; `river` dx 0.06 dy -141.18; `river` dx 0.06 dy -141.18
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

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

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1632654, 24201, 20075, 19396, 17579, 18208, 17007, 17021, 16412, 16533, 15771, 15718, 14344, 14079, 14737, 65081]`; ink px ref/ours 112652/122171 (ratio 1.0845); SSIM blocks <0.9: 12454/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 14.0] pt by ink-projection correlation (centroid estimate [-5.76, 56.4] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 22.4561, differing 0.171452, SSIM₈ 0.6459 (raw 23.3895, 0.175471, 0.6274)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4055→0.4372 / 37.3746→35.7736; header-band 1.0→0.9856 / 0.0→0.7483; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1599071, 25729, 21560, 20635, 19077, 19157, 17789, 18049, 17637, 17824, 17458, 17239, 16412, 16209, 16577, 78393]`; ink px ref/ours 112671/128654 (ratio 1.1419); SSIM blocks <0.9: 14180/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, -26.5] pt by ink-projection correlation (centroid estimate [-3.89, 51.83] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.669, differing 0.178965, SSIM₈ 0.6241 (raw 26.5725, 0.194576, 0.5521)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2852→0.4293 / 42.4619→36.4563; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8197 / 0.0→9.1359
- page 3: |Δ| histogram (16 bins, pixel counts) `[1658914, 21066, 18184, 17271, 15547, 15844, 14795, 15121, 14448, 14695, 14004, 14330, 13382, 13381, 13489, 64345]`; ink px ref/ours 112666/86107 (ratio 0.7643); SSIM blocks <0.9: 11561/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, -55.5] pt by ink-projection correlation (centroid estimate [-6.22, -53.62] pt); confidence moderate (shift explains 13% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 18.9952, differing 0.145153, SSIM₈ 0.7058 (raw 21.8391, 0.160345, 0.6342)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4162→0.5325 / 34.8971→30.3255; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy -113.01; `branch` dx -413.65 dy -113.01; `oak` dx -415.83 dy -98.57
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

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

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1709540, 25936, 20617, 19771, 18287, 18140, 16142, 15640, 12638, 11659, 10110, 9442, 8379, 7804, 7721, 26990]`; ink px ref/ours 85700/122171 (ratio 1.4256); SSIM blocks <0.9: 9952/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.4, 28.98] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 14.4715, differing 0.138056, SSIM₈ 0.7901 (raw 14.4715, 0.138056, 0.7901)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.665→0.665 / 23.1273→23.1273; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606507, 26656, 21636, 21218, 20250, 19367, 19259, 20427, 19581, 17342, 16748, 16256, 15188, 14813, 15671, 67897]`; ink px ref/ours 85737/128654 (ratio 1.5006); SSIM blocks <0.9: 14929/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -26.5] pt by ink-projection correlation (centroid estimate [2.36, 24.54] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 22.5694, differing 0.17641, SSIM₈ 0.6019 (raw 25.0571, 0.191137, 0.5283)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2468→0.39 / 40.045→34.7416; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8243 / 0.0→9.1359
- page 3: |Δ| histogram (16 bins, pixel counts) `[1667309, 22143, 18386, 17156, 16815, 15835, 16527, 17518, 16424, 14387, 13524, 13446, 12491, 12097, 12767, 51991]`; ink px ref/ours 85734/86107 (ratio 1.0044); SSIM blocks <0.9: 12376/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -26.5] pt by ink-projection correlation (centroid estimate [0.07, -80.98] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 17.9987, differing 0.143433, SSIM₈ 0.6733 (raw 20.1838, 0.156327, 0.6074)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.373→0.4785 / 32.2567→28.7644; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `river` dx 0.05 dy -141.19; `below.` dx 0.05 dy -141.19; `river` dx 0.05 dy -141.19
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1582532, 30565, 26059, 23430, 21516, 20347, 19326, 19431, 18874, 18237, 17636, 17063, 16112, 16206, 16728, 74754]`; ink px ref/ours 152381/137064 (ratio 0.8995); SSIM blocks <0.9: 14333/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 26.7909, not lower; centroid estimate [-1.35, -1.22] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.76, differing 0.206087, SSIM₈ 0.5915 (raw 26.76, 0.206087, 0.5915)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.349→0.349 / 42.7396→42.7396; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9954→0.9954 / 0.1454→0.1454
- page 2: |Δ| histogram (16 bins, pixel counts) `[1835236, 7774, 6569, 6084, 5711, 5552, 5275, 5626, 4905, 5186, 4964, 5029, 4659, 4669, 4779, 26798]`; ink px ref/ours 29740/42749 (ratio 1.4374); SSIM blocks <0.9: 4791/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 57.5] pt by ink-projection correlation (centroid estimate [7.36, 30.38] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.8839, differing 0.057107, SSIM₈ 0.8616 (raw 8.2577, 0.059519, 0.852)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.764→0.8356 / 13.1951→9.8607; header-band 1.0→0.6129 / 0.0→18.8087; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 86.93; `branch` dx -413.6 dy 86.93; `oak` dx -415.78 dy 58.19
- word-sequence differences: insert ref [] ours ['1']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1679221, 30257, 24613, 23116, 21715, 20687, 18617, 17592, 14612, 13077, 11214, 10487, 9393, 9073, 7998, 27144]`; ink px ref/ours 104967/137064 (ratio 1.3058); SSIM blocks <0.9: 11040/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.42, -0.44] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9638, differing 0.157272, SSIM₈ 0.7777 (raw 15.9638, 0.157272, 0.7777)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6455→0.6455 / 25.4994→25.4994; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9982→0.9982 / 0.0806→0.0806
- page 2: |Δ| histogram (16 bins, pixel counts) `[1851855, 9326, 8015, 7170, 7107, 6532, 6256, 5622, 4738, 4333, 3768, 3699, 3405, 3176, 2883, 10931]`; ink px ref/ours 33407/42749 (ratio 1.2796); SSIM blocks <0.9: 3716/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.49, -0.53] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 5.582, differing 0.051951, SSIM₈ 0.9173 (raw 5.582, 0.051951, 0.9173)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8681→0.8681 / 8.92→8.92; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Orphan` dx 29.06 dy 0.32; `Heading` dx 29.06 dy 0.32; `below.` dx 0.04 dy 0.62
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/16-heading-page-break/pdflatex-de1020c-export-p1-overlay.png) (56514 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-de1020c-export-p1-heatmap.png) (38291 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -14.5] pt by ink-projection correlation (centroid estimate [-6.07, -5.06] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.4928, differing 0.229367, SSIM₈ 0.4891 (raw 31.5764, 0.22951, 0.4881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1821→0.1892 / 50.4615→50.1371; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.96 / 0.0468→1.3594
- page 2: |Δ| histogram (16 bins, pixel counts) `[1821555, 8081, 6675, 6513, 5830, 5892, 6026, 5614, 5619, 6564, 5700, 5695, 5919, 5335, 5734, 32064]`; ink px ref/ours 29765/46935 (ratio 1.5769); SSIM blocks <0.9: 5085/30294; [overlay](images/16-heading-page-break/pdflatex-de1020c-export-p2-overlay.png) (53067 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-de1020c-export-p2-heatmap.png) (47489 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 20.5] pt by ink-projection correlation (centroid estimate [1.21, 55.26] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.5969, differing 0.061448, SSIM₈ 0.8517 (raw 9.6191, 0.066682, 0.8358)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7375→0.764 / 15.3741→13.6882; header-band 1.0→0.9933 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 127.11; `branch` dx -435.47 dy 74.69; `branch` dx -435.47 dy 54.75

### 16-heading-page-break — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/16-heading-page-break/pdflatex-main-export-p1-overlay.png) (56323 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-main-export-p1-heatmap.png) (38112 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -14.5] pt by ink-projection correlation (centroid estimate [-6.07, -5.06] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.4928, differing 0.229367, SSIM₈ 0.4891 (raw 31.5764, 0.22951, 0.4881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1821→0.1892 / 50.4615→50.1371; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.96 / 0.0468→1.3594
- page 2: |Δ| histogram (16 bins, pixel counts) `[1821428, 8129, 6714, 6528, 5819, 5874, 6008, 5633, 5620, 6536, 5697, 5676, 5924, 5351, 5752, 32127]`; ink px ref/ours 29765/46988 (ratio 1.5786); SSIM blocks <0.9: 5089/30294; [overlay](images/16-heading-page-break/pdflatex-main-export-p2-overlay.png) (52982 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-main-export-p2-heatmap.png) (47474 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 20.5] pt by ink-projection correlation (centroid estimate [1.33, 55.31] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.6068, differing 0.061521, SSIM₈ 0.8515 (raw 9.6287, 0.066762, 0.8356)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7373→0.7637 / 15.3895→13.7041; header-band 1.0→0.9933 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 127.11; `branch` dx -435.47 dy 74.69; `branch` dx -435.47 dy 54.75
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1583040, 30686, 25741, 23074, 21550, 20486, 19257, 19723, 18532, 18146, 17688, 17169, 15740, 15834, 16380, 75770]`; ink px ref/ours 151753/137064 (ratio 0.9032); SSIM blocks <0.9: 14229/30294; [overlay](images/16-heading-page-break/pdflatex-pipeline-export-p1-overlay.png) (57687 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-pipeline-export-p1-heatmap.png) (36329 B, ÷8)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.1, -2.23] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.6362, differing 0.205191, SSIM₈ 0.5954 (raw 26.7545, 0.205539, 0.5933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3515→0.3576 / 42.7356→42.4824; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9951→0.9953 / 0.1484→0.1484
- page 2: |Δ| histogram (16 bins, pixel counts) `[1835228, 7538, 6710, 5802, 5945, 5720, 5447, 5590, 5188, 5274, 4848, 4841, 4686, 4557, 4782, 26660]`; ink px ref/ours 29765/42749 (ratio 1.4362); SSIM blocks <0.9: 4768/30294; [overlay](images/16-heading-page-break/pdflatex-pipeline-export-p2-overlay.png) (52567 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-pipeline-export-p2-heatmap.png) (44275 B, ÷4)
  - registration error (diagnostic): global shift [-1.5, 57.5] pt by ink-projection correlation (centroid estimate [6.85, 29.6] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7382, differing 0.05662, SSIM₈ 0.8651 (raw 8.2405, 0.059233, 0.8535)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7662→0.841 / 13.1691→9.6194; header-band 1.0→0.6153 / 0.0→18.8087; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 86.74; `branch` dx -413.61 dy 86.74; `oak` dx -415.72 dy 58.19
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/16-heading-page-break/pdflatex-lm-de1020c-export-p1-overlay.png) (55012 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-de1020c-export-p1-heatmap.png) (37570 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.44, -3.32] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4403, differing 0.210395, SSIM₈ 0.5038 (raw 27.5467, 0.210674, 0.5026)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.208 / 44.0147→43.8447; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1804006, 9924, 8335, 8540, 7532, 7526, 7743, 7505, 7333, 7253, 6797, 6490, 6663, 5901, 6567, 30701]`; ink px ref/ours 33505/46935 (ratio 1.4008); SSIM blocks <0.9: 6195/30294; [overlay](images/16-heading-page-break/pdflatex-lm-de1020c-export-p2-overlay.png) (59765 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-de1020c-export-p2-heatmap.png) (52974 B, ÷4)
  - registration error (diagnostic): global shift [-1.0, 20.0] pt by ink-projection correlation (centroid estimate [-2.85, 25.0] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.6575, differing 0.066846, SSIM₈ 0.8469 (raw 10.4882, 0.077169, 0.7985)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6782→0.7566 / 16.7624→13.7844; header-band 1.0→0.9928 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 27.65; `oak` dx 426.94 dy 21.13; `oak` dx 426.94 dy -13.72

### 16-heading-page-break — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/16-heading-page-break/pdflatex-lm-main-export-p1-overlay.png) (54809 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-main-export-p1-heatmap.png) (37351 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.44, -3.32] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4403, differing 0.210395, SSIM₈ 0.5038 (raw 27.5467, 0.210674, 0.5026)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.208 / 44.0147→43.8447; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1803847, 9966, 8356, 8579, 7516, 7490, 7720, 7518, 7334, 7248, 6813, 6489, 6653, 5923, 6576, 30788]`; ink px ref/ours 33505/46988 (ratio 1.4024); SSIM blocks <0.9: 6199/30294; [overlay](images/16-heading-page-break/pdflatex-lm-main-export-p2-overlay.png) (59683 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-main-export-p2-heatmap.png) (52894 B, ÷4)
  - registration error (diagnostic): global shift [-1.0, 20.0] pt by ink-projection correlation (centroid estimate [-2.74, 25.04] pt); confidence moderate (shift explains 18% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.664, differing 0.06691, SSIM₈ 0.8469 (raw 10.5022, 0.077249, 0.7984)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.678→0.7565 / 16.7848→13.7947; header-band 1.0→0.9928 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 27.65; `oak` dx 426.94 dy 21.13; `oak` dx 426.94 dy -13.72
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1678891, 30013, 24898, 23141, 21598, 20781, 18655, 17662, 14743, 12707, 11482, 10476, 9413, 8881, 8415, 27060]`; ink px ref/ours 105111/137064 (ratio 1.304); SSIM blocks <0.9: 11045/30294; [overlay](images/16-heading-page-break/pdflatex-lm-pipeline-export-p1-overlay.png) (54541 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-pipeline-export-p1-heatmap.png) (82242 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.48, -0.49] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9984, differing 0.157345, SSIM₈ 0.7771 (raw 15.9984, 0.157345, 0.7771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6447→0.6447 / 25.5546→25.5546; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9981→0.9981 / 0.0813→0.0813
- page 2: |Δ| histogram (16 bins, pixel counts) `[1851571, 9299, 7990, 7375, 7035, 6666, 6157, 5679, 4790, 4187, 3936, 3626, 3412, 3107, 2988, 10998]`; ink px ref/ours 33505/42749 (ratio 1.2759); SSIM blocks <0.9: 3720/30294; [overlay](images/16-heading-page-break/pdflatex-lm-pipeline-export-p2-overlay.png) (58603 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-pipeline-export-p2-heatmap.png) (39703 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.78, -0.66] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 5.605, differing 0.052086, SSIM₈ 0.9165 (raw 5.605, 0.052086, 0.9165)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8668→0.8668 / 8.9567→8.9567; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Heading` dx 29.07 dy 1.97; `Orphan` dx 29.06 dy 1.97; `below.` dx 0.04 dy 1.68
- word-sequence differences: insert ref [] ours ['1']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1581634, 30214, 25515, 23782, 21671, 20933, 19589, 19568, 18768, 18249, 17860, 17780, 16063, 16138, 16195, 74857]`; ink px ref/ours 152232/137064 (ratio 0.9004); SSIM blocks <0.9: 14337/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.39, -1.18] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.718, differing 0.206581, SSIM₈ 0.5911 (raw 26.825, 0.206396, 0.5903)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3471→0.3519 / 42.8432→42.5781; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9952→0.9948 / 0.1455→0.1455
- page 2: |Δ| histogram (16 bins, pixel counts) `[1835218, 7757, 6533, 6145, 5728, 5647, 5150, 5577, 4831, 5232, 5102, 5209, 4612, 4548, 4879, 26648]`; ink px ref/ours 29777/42749 (ratio 1.4356); SSIM blocks <0.9: 4789/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 57.5] pt by ink-projection correlation (centroid estimate [7.31, 30.45] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.8923, differing 0.0572, SSIM₈ 0.8614 (raw 8.254, 0.059478, 0.8521)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.764→0.8352 / 13.1891→9.874; header-band 1.0→0.6129 / 0.0→18.8087; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 86.93; `branch` dx -413.65 dy 86.93; `oak` dx -415.83 dy 58.19
- word-sequence differences: insert ref [] ours ['1']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1679213, 30197, 24622, 23082, 21587, 20834, 18656, 17658, 14519, 13108, 11247, 10477, 9366, 9061, 8023, 27166]`; ink px ref/ours 105020/137064 (ratio 1.3051); SSIM blocks <0.9: 11040/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.5, -0.38] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.968, differing 0.15725, SSIM₈ 0.7776 (raw 15.968, 0.15725, 0.7776)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6454→0.6454 / 25.506→25.506; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9982→0.9982 / 0.0808→0.0808
- page 2: |Δ| histogram (16 bins, pixel counts) `[1851881, 9289, 8025, 7117, 7103, 6574, 6242, 5665, 4695, 4351, 3778, 3699, 3388, 3180, 2902, 10927]`; ink px ref/ours 33418/42749 (ratio 1.2792); SSIM blocks <0.9: 3717/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.54, -0.52] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 5.5832, differing 0.051941, SSIM₈ 0.9172 (raw 5.5832, 0.051941, 0.9172)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.868→0.868 / 8.9218→8.9218; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Orphan` dx 29.06 dy 1.95; `Heading` dx 29.06 dy 1.95; `below.` dx 0.04 dy 1.64
- word-sequence differences: insert ref [] ours ['1']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926517, 1080, 862, 816, 625, 664, 641, 647, 593, 655, 633, 619, 659, 544, 670, 2591]`; ink px ref/ours 5219/5184 (ratio 0.9933); SSIM blocks <0.9: 540/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [53.5, 0.0] pt by ink-projection correlation (centroid estimate [9.82, 0.96] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8851, differing 0.007144, SSIM₈ 0.986 (raw 0.9397, 0.007278, 0.9852)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9764→0.9805 / 1.5019→1.2697; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx 62.62 dy 0.41; `’` dx 62.44 dy 0.41; `plain` dx 60.06 dy 0.41
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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1928624, 1020, 806, 796, 801, 765, 702, 568, 695, 586, 516, 487, 435, 360, 371, 1284]`; ink px ref/ours 3891/5184 (ratio 1.3323); SSIM blocks <0.9: 460/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.83, -0.2] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.6315, differing 0.005967, SSIM₈ 0.99 (raw 0.675, 0.006094, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9834→0.984 / 1.0788→1.0092; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `’` dx 0.04 dy -0.36; `apostrophe.` dx 0.04 dy -0.36; `a` dx 0.03 dy -0.36

### 17-apostrophes — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927689, 1087, 914, 809, 742, 653, 583, 605, 537, 460, 552, 502, 496, 521, 444, 2222]`; ink px ref/ours 5173/5099 (ratio 0.9857); SSIM blocks <0.9: 463/30294; [overlay](images/17-apostrophes/pdflatex-de1020c-export-p1-overlay.png) (41422 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-de1020c-export-p1-heatmap.png) (86687 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-7.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8459, not lower; centroid estimate [-5.05, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8063, differing 0.006638, SSIM₈ 0.9887 (raw 0.8063, 0.006638, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9819→0.9819 / 1.2887→1.2887; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -7.83 dy 0.46; `roll,` dx -6.97 dy 0.46; `the` dx -6.5 dy 0.46
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927689, 1087, 914, 809, 742, 653, 583, 605, 537, 460, 552, 502, 496, 521, 444, 2222]`; ink px ref/ours 5173/5099 (ratio 0.9857); SSIM blocks <0.9: 463/30294; [overlay](images/17-apostrophes/pdflatex-main-export-p1-overlay.png) (40743 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-main-export-p1-heatmap.png) (86627 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-7.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8459, not lower; centroid estimate [-5.05, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8063, differing 0.006638, SSIM₈ 0.9887 (raw 0.8063, 0.006638, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9819→0.9819 / 1.2887→1.2887; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -7.83 dy 0.46; `roll,` dx -6.97 dy 0.46; `the` dx -6.5 dy 0.46
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926596, 1012, 822, 811, 701, 702, 623, 608, 613, 642, 649, 626, 601, 559, 636, 2615]`; ink px ref/ours 5173/5184 (ratio 1.0021); SSIM blocks <0.9: 540/30294; [overlay](images/17-apostrophes/pdflatex-pipeline-export-p1-overlay.png) (41341 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-pipeline-export-p1-heatmap.png) (89878 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-7.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.976, not lower; centroid estimate [10.35, 0.92] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9356, differing 0.007223, SSIM₈ 0.9856 (raw 0.9356, 0.007223, 0.9856)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.977→0.977 / 1.4954→1.4954; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `“quoted”` dx -424.84 dy 14.85; `apostrophe.` dx 62.72 dy 0.41; `’` dx 62.54 dy 0.41

### 17-apostrophes — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926770, 1005, 918, 847, 826, 791, 712, 833, 730, 582, 566, 566, 556, 521, 500, 2093]`; ink px ref/ours 3904/5099 (ratio 1.3061); SSIM blocks <0.9: 564/30294; [overlay](images/17-apostrophes/pdflatex-lm-de1020c-export-p1-overlay.png) (42120 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-lm-de1020c-export-p1-heatmap.png) (87949 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.84, -1.09] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8624, differing 0.00704, SSIM₈ 0.9848 (raw 0.8624, 0.00704, 0.9848)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9757→0.9757 / 1.3784→1.3784; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.51 dy 0.68; `plain` dx -61.24 dy 0.68; `a` dx -59.82 dy 0.68
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926770, 1005, 918, 847, 826, 791, 712, 833, 730, 582, 566, 566, 556, 521, 500, 2093]`; ink px ref/ours 3904/5099 (ratio 1.3061); SSIM blocks <0.9: 564/30294; [overlay](images/17-apostrophes/pdflatex-lm-main-export-p1-overlay.png) (41448 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-lm-main-export-p1-heatmap.png) (87552 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.84, -1.09] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8624, differing 0.00704, SSIM₈ 0.9848 (raw 0.8624, 0.00704, 0.9848)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9757→0.9757 / 1.3784→1.3784; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.51 dy 0.68; `plain` dx -61.24 dy 0.68; `a` dx -59.82 dy 0.68
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1928632, 1020, 778, 820, 806, 744, 673, 625, 677, 612, 490, 486, 428, 362, 371, 1292]`; ink px ref/ours 3904/5184 (ratio 1.3279); SSIM blocks <0.9: 461/30294; [overlay](images/17-apostrophes/pdflatex-lm-pipeline-export-p1-overlay.png) (40964 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-lm-pipeline-export-p1-heatmap.png) (85650 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.55, -0.15] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.6313, differing 0.00597, SSIM₈ 0.99 (raw 0.6744, 0.006095, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9834→0.984 / 1.078→1.0089; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `’` dx 0.06 dy 0.67; `plain` dx 0.05 dy 0.67; `apostrophe.` dx 0.05 dy 0.67

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926424, 1073, 963, 775, 655, 697, 675, 641, 605, 619, 622, 609, 652, 547, 684, 2575]`; ink px ref/ours 5210/5184 (ratio 0.995); SSIM blocks <0.9: 545/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [20.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.9509, not lower; centroid estimate [11.62, 0.89] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9402, differing 0.007301, SSIM₈ 0.9852 (raw 0.9402, 0.007301, 0.9852)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9764→0.9764 / 1.5027→1.5027; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `“quoted”` dx -424.88 dy 14.85; `apostrophe.` dx 62.6 dy 0.41; `’` dx 62.42 dy 0.41

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1928608, 1036, 801, 794, 811, 761, 704, 566, 692, 590, 512, 485, 440, 360, 380, 1276]`; ink px ref/ours 3889/5184 (ratio 1.333); SSIM blocks <0.9: 460/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [2.36, -0.2] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.631, differing 0.005964, SSIM₈ 0.99 (raw 0.6748, 0.006093, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9834→0.984 / 1.0786→1.0085; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `’` dx 0.04 dy 0.67; `apostrophe.` dx 0.04 dy 0.67; `a` dx 0.03 dy 0.67

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920114, 1744, 1457, 1210, 1071, 1013, 1047, 886, 956, 950, 887, 863, 1046, 856, 945, 3771]`; ink px ref/ours 8390/8210 (ratio 0.9785); SSIM blocks <0.9: 790/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [4.0, 0.0] pt REJECTED: applying it gives mean|Δ| 1.4574, not lower; centroid estimate [-1.39, 0.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3969, differing 0.011054, SSIM₈ 0.9783 (raw 1.3969, 0.011054, 0.9783)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9655→0.9655 / 2.2303→2.2303; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -444.42 dy 14.85; `ruffled,` dx -432.48 dy 14.85; `muffin.` dx 44.85 dy 0.41

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923492, 1777, 1322, 1424, 1289, 1091, 1053, 973, 820, 654, 652, 694, 583, 492, 503, 1997]`; ink px ref/ours 6488/8210 (ratio 1.2654); SSIM blocks <0.9: 682/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [5.75, -0.37] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9754, differing 0.009185, SSIM₈ 0.986 (raw 0.9754, 0.009185, 0.986)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9778→0.9778 / 1.5566→1.5566; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `office,` dx -1.49 dy -0.36; `effect,` dx -1.33 dy -0.36; `affine,` dx -1.16 dy -0.36

### 18-ligatures — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921093, 1851, 1396, 1273, 1121, 942, 936, 833, 874, 895, 836, 811, 990, 726, 718, 3521]`; ink px ref/ours 8210/8297 (ratio 1.0106); SSIM blocks <0.9: 724/30294; [overlay](images/18-ligatures/pdflatex-de1020c-export-p1-overlay.png) (47915 B, ÷2), [heatmap](images/18-ligatures/pdflatex-de1020c-export-p1-heatmap.png) (40339 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.13, 0.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2894, differing 0.010507, SSIM₈ 0.9808 (raw 1.2894, 0.010507, 0.9808)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9693→0.9693 / 2.0608→2.0608; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.86 dy 14.86; `ruffled,` dx -433.44 dy 14.82; `muffin.` dx 40.63 dy 0.37

### 18-ligatures — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921093, 1851, 1396, 1273, 1121, 942, 936, 833, 874, 895, 836, 811, 990, 726, 718, 3521]`; ink px ref/ours 8210/8297 (ratio 1.0106); SSIM blocks <0.9: 724/30294; [overlay](images/18-ligatures/pdflatex-main-export-p1-overlay.png) (47344 B, ÷2), [heatmap](images/18-ligatures/pdflatex-main-export-p1-heatmap.png) (39830 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.13, 0.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2894, differing 0.010507, SSIM₈ 0.9808 (raw 1.2894, 0.010507, 0.9808)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9693→0.9693 / 2.0608→2.0608; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.86 dy 14.86; `ruffled,` dx -433.44 dy 14.82; `muffin.` dx 40.63 dy 0.37

### 18-ligatures — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920475, 1775, 1339, 1190, 1091, 1041, 1012, 935, 815, 878, 851, 913, 1034, 852, 864, 3751]`; ink px ref/ours 8210/8210 (ratio 1.0); SSIM blocks <0.9: 767/30294; [overlay](images/18-ligatures/pdflatex-pipeline-export-p1-overlay.png) (47755 B, ÷2), [heatmap](images/18-ligatures/pdflatex-pipeline-export-p1-heatmap.png) (40267 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.4074, not lower; centroid estimate [-2.0, 0.89] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3694, differing 0.010829, SSIM₈ 0.9794 (raw 1.3694, 0.010829, 0.9794)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9672→0.9672 / 2.1863→2.1863; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.86 dy 14.85; `ruffled,` dx -433.44 dy 14.85; `muffin.` dx 46.44 dy 0.41

### 18-ligatures — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920380, 1602, 1329, 1334, 1326, 1152, 1112, 1125, 1111, 987, 890, 943, 938, 768, 690, 3129]`; ink px ref/ours 6473/8297 (ratio 1.2818); SSIM blocks <0.9: 798/30294; [overlay](images/18-ligatures/pdflatex-lm-de1020c-export-p1-overlay.png) (49357 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-de1020c-export-p1-heatmap.png) (41434 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-18.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3343, not lower; centroid estimate [-5.55, -0.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3148, differing 0.010687, SSIM₈ 0.9787 (raw 1.3148, 0.010687, 0.9787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.1014→2.1014; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `coffin,` dx -21.57 dy 0.73; `official,` dx -20.57 dy 0.73; `offline,` dx -20.28 dy 0.68

### 18-ligatures — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920380, 1602, 1329, 1334, 1326, 1152, 1112, 1125, 1111, 987, 890, 943, 938, 768, 690, 3129]`; ink px ref/ours 6473/8297 (ratio 1.2818); SSIM blocks <0.9: 798/30294; [overlay](images/18-ligatures/pdflatex-lm-main-export-p1-overlay.png) (48755 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-main-export-p1-heatmap.png) (40887 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-18.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3343, not lower; centroid estimate [-5.55, -0.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3148, differing 0.010687, SSIM₈ 0.9787 (raw 1.3148, 0.010687, 0.9787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.1014→2.1014; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `coffin,` dx -21.57 dy 0.73; `official,` dx -20.57 dy 0.73; `offline,` dx -20.28 dy 0.68

### 18-ligatures — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923523, 1748, 1334, 1401, 1274, 1082, 1110, 928, 839, 689, 626, 662, 611, 465, 512, 2012]`; ink px ref/ours 6473/8210 (ratio 1.2683); SSIM blocks <0.9: 679/30294; [overlay](images/18-ligatures/pdflatex-lm-pipeline-export-p1-overlay.png) (47652 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-pipeline-export-p1-heatmap.png) (38564 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [5.58, -0.36] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9763, differing 0.009195, SSIM₈ 0.986 (raw 0.9763, 0.009195, 0.986)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9778→0.9778 / 1.5581→1.5581; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `office,` dx -1.49 dy 0.67; `effect,` dx -1.31 dy 0.67; `affine,` dx -1.15 dy 0.67

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920092, 1745, 1405, 1265, 1164, 1066, 1045, 961, 985, 883, 926, 873, 1006, 792, 889, 3719]`; ink px ref/ours 8366/8210 (ratio 0.9814); SSIM blocks <0.9: 790/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.0, 0.0] pt REJECTED: applying it gives mean|Δ| 1.4685, not lower; centroid estimate [-1.14, 0.89] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3851, differing 0.011051, SSIM₈ 0.9786 (raw 1.3851, 0.011051, 0.9786)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.2115→2.2115; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.26 dy 14.85; `ruffled,` dx -432.74 dy 14.85; `muffin.` dx 45.05 dy 0.41

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923496, 1768, 1335, 1422, 1293, 1080, 1072, 958, 820, 659, 644, 693, 578, 498, 511, 1989]`; ink px ref/ours 6486/8210 (ratio 1.2658); SSIM blocks <0.9: 682/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [5.82, -0.36] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9752, differing 0.009178, SSIM₈ 0.986 (raw 0.9752, 0.009178, 0.986)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9779→0.9779 / 1.5564→1.5564; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oﬀice,` dx -1.49 dy 0.67; `effect,` dx -1.33 dy 0.67; `aﬀine,` dx -1.16 dy 0.67

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

- previous evidence: `tests/visual-corpus/evidence/20260912T065946Z`; comparable entries: 648
- worse: 
  - 02-wrapping-paragraph/lualatex/pipeline/export: ssim_8x8_mean 0.8938 -> 0.8901
  - 02-wrapping-paragraph/lualatex/pipeline/preview: ssim_8x8_mean 0.8938 -> 0.8902
  - 02-wrapping-paragraph/pdflatex/pipeline/export: ssim_8x8_mean 0.8947 -> 0.891
  - 02-wrapping-paragraph/pdflatex/pipeline/preview: ssim_8x8_mean 0.8947 -> 0.891
  - 02-wrapping-paragraph/xelatex/pipeline/export: ssim_8x8_mean 0.894 -> 0.8902
  - 02-wrapping-paragraph/xelatex/pipeline/preview: ssim_8x8_mean 0.894 -> 0.8902
  - 08-two-page/pdflatex/pipeline/export: ssim_8x8_mean 0.6738 -> 0.6711
  - 08-two-page/pdflatex/pipeline/export: diff_mean 20.9655 -> 21.172
  - 08-two-page/pdflatex/pipeline/preview: ssim_8x8_mean 0.6739 -> 0.6711
  - 08-two-page/pdflatex/pipeline/preview: diff_mean 20.9638 -> 21.1714
  - 13-math-display-rich/lualatex/pipeline/export: ssim_8x8_mean 0.9896 -> 0.9871
  - 13-math-display-rich/lualatex/pipeline/preview: ssim_8x8_mean 0.9896 -> 0.9871
  - 13-math-display-rich/pdflatex/pipeline/export: ssim_8x8_mean 0.9896 -> 0.9871
  - 13-math-display-rich/pdflatex/pipeline/preview: ssim_8x8_mean 0.9896 -> 0.9871
  - 13-math-display-rich/xelatex/pipeline/export: ssim_8x8_mean 0.9895 -> 0.987
  - 13-math-display-rich/xelatex/pipeline/preview: ssim_8x8_mean 0.9896 -> 0.987
  - 14-math-inline-dense/lualatex/pipeline/export: ssim_8x8_mean 0.9818 -> 0.9797
  - 14-math-inline-dense/lualatex/pipeline/preview: diff_mean 0.8299 -> 1.0666
  - 14-math-inline-dense/pdflatex/pipeline/export: ssim_8x8_mean 0.9822 -> 0.9799
  - 14-math-inline-dense/pdflatex/pipeline/preview: ssim_8x8_mean 0.9822 -> 0.9799
  - 14-math-inline-dense/pdflatex/pipeline/preview: diff_mean 0.82 -> 1.058
  - 14-math-inline-dense/pdflatex-lm/pipeline/preview: diff_mean 0.805 -> 1.0054
  - 14-math-inline-dense/xelatex/pipeline/export: ssim_8x8_mean 0.9819 -> 0.9797
  - 14-math-inline-dense/xelatex/pipeline/preview: diff_mean 0.8298 -> 1.0667
  - 15-three-page-sections/lualatex/pipeline/export: ssim_8x8_mean 0.6073 -> 0.6049
  - 15-three-page-sections/lualatex/pipeline/preview: ssim_8x8_mean 0.6074 -> 0.605
  - 15-three-page-sections/pdflatex/pipeline/export: ssim_8x8_mean 0.6054 -> 0.5969
  - 15-three-page-sections/pdflatex/pipeline/export: diff_mean 23.7585 -> 24.0962
  - 15-three-page-sections/pdflatex/pipeline/preview: ssim_8x8_mean 0.6054 -> 0.597
  - 15-three-page-sections/pdflatex/pipeline/preview: diff_mean 23.7591 -> 24.0963
  - 15-three-page-sections/xelatex/pipeline/export: ssim_8x8_mean 0.6079 -> 0.6046
  - 15-three-page-sections/xelatex/pipeline/preview: ssim_8x8_mean 0.6079 -> 0.6046
  - 18-ligatures/lualatex/pipeline/export: ssim_8x8_mean 0.984 -> 0.9783
  - 18-ligatures/lualatex/pipeline/export: diff_mean 1.1489 -> 1.3969
  - 18-ligatures/lualatex/pipeline/preview: ssim_8x8_mean 0.9862 -> 0.9786
  - 18-ligatures/lualatex/pipeline/preview: diff_mean 1.0292 -> 1.3808
  - 18-ligatures/pdflatex/pipeline/export: ssim_8x8_mean 0.9907 -> 0.9794
  - 18-ligatures/pdflatex/pipeline/export: diff_mean 0.7396 -> 1.3694
  - 18-ligatures/pdflatex/pipeline/export: above_threshold_fraction 0.005625 -> 0.008544
  - 18-ligatures/pdflatex/pipeline/preview: ssim_8x8_mean 0.9926 -> 0.9796
  - 18-ligatures/pdflatex/pipeline/preview: diff_mean 0.6206 -> 1.3535
  - 18-ligatures/pdflatex/pipeline/preview: above_threshold_fraction 0.004868 -> 0.008519
  - 18-ligatures/xelatex/pipeline/export: ssim_8x8_mean 0.9896 -> 0.9786
  - 18-ligatures/xelatex/pipeline/export: diff_mean 0.8056 -> 1.3851
  - 18-ligatures/xelatex/pipeline/export: above_threshold_fraction 0.006119 -> 0.008757
  - 18-ligatures/xelatex/pipeline/preview: ssim_8x8_mean 0.9915 -> 0.9788
  - 18-ligatures/xelatex/pipeline/preview: diff_mean 0.6841 -> 1.3721
  - 18-ligatures/xelatex/pipeline/preview: above_threshold_fraction 0.00554 -> 0.008748

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
