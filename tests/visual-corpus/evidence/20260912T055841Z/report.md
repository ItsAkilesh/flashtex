# FlashTeX visual corpus: reference-render and raster-diff evidence

Generated 20260912T055841Z on mac-m1max-a by `tests/visual-corpus/harness/run.sh`.

**Scope statement.** These are narrow-case measurements over a small declared corpus. They never claim general pixel perfection, LaTeX compatibility, or parity outside these fixtures, these engines, this font, this page size, this DPI and these builds. The reference engines are test oracles only; FlashTeX never invokes them and remains an original Rust implementation.

**Acceptance vs diagnostics.** The only acceptance signals in this report are the exact-equality gates below (zero pixel difference between FlashTeX's export raster and its preview rasters, and raw PDF byte identity against the pinned profile). Every tolerance, threshold, SSIM, registration shift or regression comparison further down is a diagnostic to explain *why* something differs; none of them ever counts as acceptance.

## Exact-equality gates (acceptance)

| Fixture | Compiler | export = preview-equivalent | export = native preview capture | PDF bytes = pinned | PDF SHA-256 |
|---|---|---|---|---|---|
| 01-plain-paragraph | de1020c | DIFFERENT: 1240/1938816 px, max |Δ| 255 | DIFFERENT: 37010/1938816 px, max |Δ| 249 | baseline pinned in this run (not a pass) | `88deda9f24394700…` |
| 01-plain-paragraph | main | DIFFERENT: 1315/1938816 px, max |Δ| 255 | DIFFERENT: 41787/1938816 px, max |Δ| 250 | baseline pinned in this run (not a pass) | `982579044f6e075e…` |
| 02-wrapping-paragraph | de1020c | DIFFERENT: 17142/1938816 px, max |Δ| 255 | DIFFERENT: 355351/1938816 px, max |Δ| 252 | baseline pinned in this run (not a pass) | `1bc0994be86d3ece…` |
| 02-wrapping-paragraph | main | DIFFERENT: 17263/1938816 px, max |Δ| 255 | DIFFERENT: 403534/1938816 px, max |Δ| 252 | baseline pinned in this run (not a pass) | `a648c982d1829fad…` |
| 03-section-heading | de1020c | DIFFERENT: 2140/1938816 px, max |Δ| 6 | DIFFERENT: 67245/1938816 px, max |Δ| 252 | baseline pinned in this run (not a pass) | `7f0f7247434d3132…` |
| 03-section-heading | main | DIFFERENT: 2103/1938816 px, max |Δ| 6 | DIFFERENT: 69503/1938816 px, max |Δ| 255 | baseline pinned in this run (not a pass) | `456aedda047b3d9f…` |
| 04-bold-emph | de1020c | DIFFERENT: 1261/1938816 px, max |Δ| 4 | DIFFERENT: 37492/1938816 px, max |Δ| 252 | baseline pinned in this run (not a pass) | `7a34b0a51a2992f4…` |
| 04-bold-emph | main | DIFFERENT: 1233/1938816 px, max |Δ| 4 | DIFFERENT: 40225/1938816 px, max |Δ| 252 | baseline pinned in this run (not a pass) | `483817723c7a07a7…` |
| 05-unicode | de1020c | DIFFERENT: 587/1938816 px, max |Δ| 3 | DIFFERENT: 37446/1938816 px, max |Δ| 251 | baseline pinned in this run (not a pass) | `e721656e20b80d47…` |
| 05-unicode | main | DIFFERENT: 658/1938816 px, max |Δ| 2 | DIFFERENT: 40985/1938816 px, max |Δ| 251 | baseline pinned in this run (not a pass) | `4a9c9cd453fdc68b…` |
| 06-math-inline | de1020c | DIFFERENT: 1041/1938816 px, max |Δ| 254 | DIFFERENT: 31683/1938816 px, max |Δ| 249 | baseline pinned in this run (not a pass) | `0a1a2a6eebc4b2b1…` |
| 06-math-inline | main | DIFFERENT: 734/1938816 px, max |Δ| 4 | DIFFERENT: 31189/1938816 px, max |Δ| 251 | baseline pinned in this run (not a pass) | `055d64e818f4342a…` |
| 07-math-display | de1020c | DIFFERENT: 1021/1938816 px, max |Δ| 255 | DIFFERENT: 30918/1938816 px, max |Δ| 252 | baseline pinned in this run (not a pass) | `915882632c74f40b…` |
| 07-math-display | main | DIFFERENT: 697/1938816 px, max |Δ| 255 | DIFFERENT: 37796/1938816 px, max |Δ| 252 | baseline pinned in this run (not a pass) | `ada42128ad2cb2f6…` |
| 08-two-page | de1020c | DIFFERENT: 58237/1938816 px, max |Δ| 255 | DIFFERENT: 1184411/1938816 px, max |Δ| 252 | baseline pinned in this run (not a pass) | `6e8c29a5f28f5429…` |
| 08-two-page | main | DIFFERENT: 53192/1938816 px, max |Δ| 255 | DIFFERENT: 1201636/1938816 px, max |Δ| 252 | baseline pinned in this run (not a pass) | `44e782b1fe089026…` |
| 09-mixed-document | de1020c | DIFFERENT: 4608/1938816 px, max |Δ| 250 | DIFFERENT: 133326/1938816 px, max |Δ| 252 | baseline pinned in this run (not a pass) | `eb9ed31b44ff065e…` |
| 09-mixed-document | main | DIFFERENT: 4801/1938816 px, max |Δ| 255 | DIFFERENT: 143932/1938816 px, max |Δ| 252 | baseline pinned in this run (not a pass) | `ed1c2d77c3510fb2…` |

- Reference profile: `reference-profile.json` — **pinned/re-baselined in this run** (explicit `--pin-profile`).
- Classification of native-preview differences: the native capture comes from a screen capture at the display's backing scale, resampled to the raster size, so a DIFFERENT result there is expected to be dominated by resampling and text rasterization (CoreText on screen vs CoreGraphics PDF rendering); it is reported as-is, without normalisation. Preview-equivalent vs export differences isolate the drawing path (CoreText glyph run vs the PDF writer's text operators) from any capture effects.

## Provenance

- suite_branch: `agent/mac-visual-oracle/reference-raster`
- suite_sha: `322684073cc1cc86e84dfa1f8c67562df736959f`
- input_main_sha: `984b77db504051dbd20b1bbae9f8a9f14e0cf1c7`
- machine: `mac-m1max-a`
- os: `macOS 26.3.1 arm64`
- swift: `Apple Swift version 6.2.4 (swiftlang-6.2.4.1.4 clang-1700.6.4.2)`
- cargo: `cargo 1.99.0-nightly (3efb1f477 2026-07-17)`
- python: `3.12.0`
- pillow: `12.2.0`
- DPI: 144 (every raster: CoreGraphics bitmap, sRGB IEC61966-2.1, 8-bit RGBA, white opaque background, MediaBox mapped to width_pt*144/72 px; text antialiased, font smoothing off, subpixel positioning on)
- Overlay/heatmap PNGs emitted for engines: pdflatex,pdflatex-lm, sides: export,native (metrics are computed for every engine and side; PNGs are downscaled by 2 until ≤90000 B)
- Pixel threshold for `above_threshold_fraction`: |Δluma| ≥ 32/255; SSIM: 8×8 blocks, K1=0.01, K2=0.03
- Arithmetic backend: Pillow 12.2.0 (accelerator; identical integer results to the stdlib path)

### Reference engines (oracle only)

| Oracle | Available | Version | Body font | Preamble |
|---|---|---|---|---|
| pdflatex | yes | pdfTeX 3.141592653-2.6-1.40.29 (TeX Live 2026) | URW Nimbus Roman (`times` package, T1 fontenc) | `\documentclass[12pt]{article} \usepackage[T1]{fontenc} \usepackage{times} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| pdflatex-lm | yes | pdfTeX 3.141592653-2.6-1.40.29 (TeX Live 2026) | Latin Modern Roman Type 1 (`lmodern` package, T1 fontenc) — LaTeX's default Computer Modern look | `\documentclass[12pt]{article} \usepackage[T1]{fontenc} \usepackage{lmodern} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| xelatex | yes | XeTeX 3.141592653-2.6-0.999998 (TeX Live 2026) | Times New Roman (`fontspec`, /System/Library/Fonts/Supplemental/Times New Roman.ttf) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{Times New Roman} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| xelatex-lm | yes | XeTeX 3.141592653-2.6-0.999998 (TeX Live 2026) | Latin Modern Roman OpenType (`fontspec`, lmroman12-*.otf from the TeX Live tree by explicit path; bold-italic uses lmroman10-bolditalic, the only LM bold-italic face) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{lmroman12-regular.otf}[Path=/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm/,BoldFont=lmroman12-bold.otf,ItalicFont=lmroman12-italic.otf,BoldItalicFont=lmroman10-bolditalic.otf] \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| lualatex | yes | This is LuaHBTeX, Version 1.24.0 (TeX Live 2026) | Times New Roman (`fontspec`, /System/Library/Fonts/Supplemental/Times New Roman.ttf) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{Times New Roman} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| lualatex-lm | yes | This is LuaHBTeX, Version 1.24.0 (TeX Live 2026) | Latin Modern Roman OpenType (`fontspec`, lmroman12-*.otf from the TeX Live tree by explicit path; bold-italic uses lmroman10-bolditalic, the only LM bold-italic face) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{lmroman12-regular.otf}[Path=/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm/,BoldFont=lmroman12-bold.otf,ItalicFont=lmroman12-italic.otf,BoldItalicFont=lmroman10-bolditalic.otf] \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |

The `-lm` oracles are the intended primary apples-to-apples target once a Latin-Modern-metrics FlashTeX pipeline exists; the Times oracles match the current compiler's Times metrics. Both are reported for every fixture.

Engine flags: `-interaction=batchmode -halt-on-error -file-line-error`. Page size: US letter 612×792 pt for every producer (checked per page from the MediaBox). LaTeX package versions: see `provenance.json` → `packages`.

### FlashTeX builds under test

- compiler `main`: `origin/main` @ `18ba1b3d33e1e0bb0eb54aebf381310ca14b5fbd` — fix: honor exact published worker branch assignments
- compiler `de1020c`: `de1020c` @ `de1020cd0be7cede11be2691e00e7f5b15cb2224` — compiler: add math, real font metrics and PDF output
- PDF writer: `5b5f7b5` @ `5b5f7b5bcbc44ba9376b3a237afe93ba1503d13c` (`flashtex-pdf --verify --embed-font auto`; body font Times-Roman standard-14, Unicode fallback subset of embedding subset of TimesNewRomanPSMT from /System/Library/Fonts/Supplemental/Times New Roman.ttf)
- export raster: the flashtex-pdf PDF rasterized by the same CoreGraphics rasterizer as the references
- preview-equivalent raster: `rasterize preview` re-implements the Mac app's `PDFExport.render` draw (CoreText `Times-Roman` at x_pt/baseline_y_pt/font_size_pt, U+2500 runs as 0.5em×0.0857em rules) straight into the bitmap. It links nothing from apps/mac and is **not** the SwiftUI preview; it is labelled preview-equivalent throughout.
- native preview capture: screencapture -x -o -l <CGWindowList id> of the running FlashTeXMac window; page rectangle detected as the largest white region bounded by the pane background; resampled with CoreGraphics high-quality interpolation to the 144-DPI raster size; the 'page N' caption region (bottom-right of the page) is masked white on the capture only; page edges refined inward until each edge line is >=60% paper white (removes the anti-aliased border). App: `/private/tmp/claude-501/-Users-jay3332-Projects-flashtex/e30fd4a4-f46a-4c3f-a28c-cbb8617b4425/scratchpad/app/apps/mac/.build/debug/FlashTeXMac`. Display(s):  Resolution: 3024 x 1964 Retina; Resolution: 1920 x 1080 (1080p FHD - Full High Definition);. Capture backing factor(s): [1.0] px/pt; resample factor to the 144-DPI raster: 2.839907192575406–2.839907192575406 (>1 means the capture was UPSAMPLED, so glyph edges are interpolated and this comparison is coarser than the export one).

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

Full SHAs and `.meta.json` contents are in `provenance.json`. Only the body after `\begin{document}` is shared by every producer; the harness substitutes the engine preamble and strips it for FlashTeX.

## Diagnostic: export comparison (flashtex-pdf PDF vs reference PDF, both rasterized identically)

This is the PDF-output comparison against the oracle. Word boxes come from PDFKit on both PDFs; rules are ink rows ≥10pt long. Diagnostic only.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| | max | differing | ≥thr | SSIM₈ | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-plain-paragraph | lualatex | de1020c | 1/1 | ok | 0.1575 | 255 | 0.0023 | 0.0014 | 0.9980 | 13/13/13 | yes | 0.29 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.4024 | 255 | 0.0032 | 0.0025 | 0.9937 | 13/13/13 | yes | 26.11 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | de1020c | 1/1 | ok | 0.3662 | 255 | 0.0030 | 0.0025 | 0.9941 | 13/13/13 | yes | 12.45 | 0.30 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | main | 1/1 | ok | 0.3931 | 255 | 0.0032 | 0.0026 | 0.9933 | 13/13/13 | yes | 13.91 | 0.30 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.1873 | 255 | 0.0024 | 0.0014 | 0.9975 | 13/13/13 | yes | 0.43 | 0.46 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.3986 | 255 | 0.0032 | 0.0025 | 0.9939 | 13/13/13 | yes | 26.62 | 0.46 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-main-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 0.3660 | 255 | 0.0030 | 0.0025 | 0.9942 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | main | 1/1 | ok | 0.3927 | 255 | 0.0032 | 0.0026 | 0.9933 | 13/13/13 | yes | 13.90 | 0.73 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.1607 | 255 | 0.0023 | 0.0014 | 0.9980 | 13/13/13 | yes | 0.30 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.4030 | 255 | 0.0032 | 0.0025 | 0.9937 | 13/13/13 | yes | 26.09 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | de1020c | 1/1 | ok | 0.3665 | 255 | 0.0030 | 0.0025 | 0.9941 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | main | 1/1 | ok | 0.3932 | 255 | 0.0032 | 0.0026 | 0.9933 | 13/13/13 | yes | 13.91 | 0.73 | 1.0000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 8.3842 | 255 | 0.0613 | 0.0509 | 0.8656 | 210/210/210 | yes | 116.20 | 5.12 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 8.4755 | 255 | 0.0621 | 0.0516 | 0.8581 | 210/210/210 | yes | 146.62 | 16.30 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | de1020c | 1/1 | ok | 7.7868 | 255 | 0.0597 | 0.0495 | 0.8620 | 210/210/210 | yes | 91.94 | 3.57 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | main | 1/1 | ok | 7.8526 | 255 | 0.0600 | 0.0498 | 0.8552 | 210/210/210 | yes | 134.89 | 12.02 | 0.9095 | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 8.3726 | 255 | 0.0611 | 0.0509 | 0.8663 | 210/210/210 | yes | 116.24 | 5.12 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 8.5059 | 255 | 0.0620 | 0.0516 | 0.8582 | 210/210/210 | yes | 146.59 | 16.30 | 0.8952 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-main-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 7.7862 | 255 | 0.0597 | 0.0495 | 0.8620 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | main | 1/1 | ok | 7.8514 | 255 | 0.0600 | 0.0498 | 0.8552 | 210/210/210 | yes | 134.89 | 12.71 | 0.9095 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 8.3710 | 255 | 0.0613 | 0.0509 | 0.8656 | 210/210/210 | yes | 119.39 | 5.19 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 8.4942 | 255 | 0.0621 | 0.0518 | 0.8578 | 210/210/210 | yes | 144.25 | 16.36 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | de1020c | 1/1 | ok | 7.7866 | 255 | 0.0597 | 0.0495 | 0.8620 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | main | 1/1 | ok | 7.8523 | 255 | 0.0600 | 0.0498 | 0.8552 | 210/210/210 | yes | 134.89 | 12.71 | 0.9095 | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.3394 | 255 | 0.0085 | 0.0073 | 0.9793 | 15/15/15 | yes | 0.25 | 21.02 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.2663 | 255 | 0.0081 | 0.0070 | 0.9802 | 15/13/11 | no | 5.51 | 22.60 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex-lm | de1020c | 1/1 | ok | 1.2336 | 255 | 0.0082 | 0.0069 | 0.9771 | 15/15/15 | yes | 5.46 | 21.19 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex-lm | main | 1/1 | ok | 1.1783 | 255 | 0.0079 | 0.0067 | 0.9780 | 15/13/11 | no | 2.23 | 22.87 | 1.0000 | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.3496 | 255 | 0.0085 | 0.0072 | 0.9793 | 15/15/15 | yes | 0.33 | 20.87 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-de1020c-export-p1-overlay.png) |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.2822 | 255 | 0.0082 | 0.0070 | 0.9801 | 15/13/11 | no | 5.62 | 22.46 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-main-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | de1020c | 1/1 | ok | 1.2335 | 255 | 0.0082 | 0.0069 | 0.9771 | 15/15/15 | yes | 5.46 | 22.37 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | main | 1/1 | ok | 1.1779 | 255 | 0.0079 | 0.0067 | 0.9781 | 15/13/11 | no | 2.23 | 23.97 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-main-export-p1-overlay.png) |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.3370 | 255 | 0.0085 | 0.0073 | 0.9793 | 15/15/15 | yes | 0.24 | 21.02 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.2665 | 255 | 0.0081 | 0.0071 | 0.9802 | 15/13/11 | no | 5.50 | 22.60 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex-lm | de1020c | 1/1 | ok | 1.2337 | 255 | 0.0082 | 0.0069 | 0.9771 | 15/15/15 | yes | 5.46 | 22.35 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex-lm | main | 1/1 | ok | 1.1784 | 255 | 0.0079 | 0.0067 | 0.9780 | 15/13/11 | no | 2.23 | 23.94 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.3808 | 255 | 0.0031 | 0.0024 | 0.9945 | 10/12/8 | no | 3.77 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.4195 | 255 | 0.0033 | 0.0026 | 0.9939 | 10/12/8 | no | 29.18 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | de1020c | 1/1 | ok | 0.4244 | 255 | 0.0033 | 0.0027 | 0.9933 | 10/12/8 | no | 16.02 | 0.63 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | main | 1/1 | ok | 0.4313 | 255 | 0.0033 | 0.0027 | 0.9932 | 10/12/8 | no | 9.38 | 0.63 | 1.0000 | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.3836 | 255 | 0.0031 | 0.0024 | 0.9944 | 10/12/8 | no | 4.04 | 0.46 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-de1020c-export-p1-overlay.png) |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.4290 | 255 | 0.0034 | 0.0026 | 0.9937 | 10/12/8 | no | 29.44 | 0.46 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-main-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | de1020c | 1/1 | ok | 0.4246 | 255 | 0.0033 | 0.0027 | 0.9933 | 10/12/8 | no | 16.08 | 0.73 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | main | 1/1 | ok | 0.4325 | 255 | 0.0033 | 0.0027 | 0.9931 | 10/12/8 | no | 9.32 | 0.73 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-main-export-p1-overlay.png) |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.3848 | 255 | 0.0031 | 0.0025 | 0.9944 | 10/12/8 | no | 3.81 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.4208 | 255 | 0.0033 | 0.0026 | 0.9939 | 10/12/8 | no | 29.21 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | de1020c | 1/1 | ok | 0.4245 | 255 | 0.0033 | 0.0027 | 0.9933 | 10/12/8 | no | 15.90 | 0.73 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | main | 1/1 | ok | 0.4288 | 255 | 0.0034 | 0.0027 | 0.9932 | 10/12/8 | no | 9.50 | 0.73 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.2971 | 255 | 0.0027 | 0.0020 | 0.9955 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.3747 | 255 | 0.0030 | 0.0023 | 0.9941 | 11/16/5 | no | 17.27 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex-lm | de1020c | 1/1 | ok | 0.3619 | 255 | 0.0029 | 0.0024 | 0.9938 | 11/19/7 | no | 6.78 | 0.30 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex-lm | main | 1/1 | ok | 0.3858 | 255 | 0.0030 | 0.0024 | 0.9932 | 11/16/5 | no | 13.19 | 0.30 | 1.0000 | 0/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.3127 | 255 | 0.0027 | 0.0020 | 0.9955 | 11/19/7 | no | 5.88 | 0.46 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-de1020c-export-p1-overlay.png) |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.3712 | 255 | 0.0030 | 0.0023 | 0.9945 | 11/16/5 | no | 17.43 | 0.46 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-main-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | de1020c | 1/1 | ok | 0.3617 | 255 | 0.0029 | 0.0024 | 0.9938 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | main | 1/1 | ok | 0.3853 | 255 | 0.0030 | 0.0024 | 0.9932 | 11/16/5 | no | 13.19 | 0.73 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-main-export-p1-overlay.png) |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.2963 | 255 | 0.0027 | 0.0020 | 0.9955 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.3752 | 255 | 0.0030 | 0.0023 | 0.9941 | 11/16/5 | no | 17.25 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex-lm | de1020c | 1/1 | ok | 0.3620 | 255 | 0.0029 | 0.0024 | 0.9938 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | - |
| 05-unicode | xelatex-lm | main | 1/1 | ok | 0.3858 | 255 | 0.0030 | 0.0024 | 0.9932 | 11/16/5 | no | 13.19 | 0.73 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.3832 | 255 | 0.0029 | 0.0023 | 0.9934 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | recovered | 0.2455 | 255 | 0.0023 | 0.0017 | 0.9958 | 15/14/12 | no | 11.36 | 2.08 | 0.9167 | 0/0 | - |
| 06-math-inline | lualatex-lm | de1020c | 1/1 | ok | 0.3682 | 255 | 0.0029 | 0.0023 | 0.9931 | 15/14/13 | no | 12.25 | 2.20 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | main | 1/1 | recovered | 0.2606 | 255 | 0.0022 | 0.0018 | 0.9953 | 15/14/12 | no | 6.11 | 2.04 | 0.9167 | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.3821 | 255 | 0.0029 | 0.0023 | 0.9936 | 13/14/10 | no | 3.89 | 2.69 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-de1020c-export-p1-overlay.png) |
| 06-math-inline | pdflatex | main | 1/1 | recovered | 0.2741 | 255 | 0.0023 | 0.0018 | 0.9955 | 13/14/10 | no | 12.25 | 1.97 | 0.9000 | 0/0 | [p1](images/06-math-inline/pdflatex-main-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | de1020c | 1/1 | ok | 0.3680 | 255 | 0.0029 | 0.0023 | 0.9931 | 14/14/12 | no | 13.02 | 2.53 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | main | 1/1 | recovered | 0.2609 | 255 | 0.0022 | 0.0018 | 0.9953 | 14/14/11 | no | 5.88 | 2.10 | 0.9091 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-main-export-p1-overlay.png) |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.3831 | 255 | 0.0029 | 0.0023 | 0.9934 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | recovered | 0.2451 | 255 | 0.0022 | 0.0017 | 0.9958 | 15/14/12 | no | 11.36 | 2.08 | 0.9167 | 0/0 | - |
| 06-math-inline | xelatex-lm | de1020c | 1/1 | ok | 0.3682 | 255 | 0.0029 | 0.0023 | 0.9931 | 15/14/13 | no | 12.25 | 2.44 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | main | 1/1 | recovered | 0.2606 | 255 | 0.0022 | 0.0018 | 0.9953 | 15/14/12 | no | 6.11 | 2.15 | 0.9167 | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.2595 | 255 | 0.0026 | 0.0019 | 0.9950 | 13/10/7 | no | 0.60 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | recovered | 0.4269 | 255 | 0.0033 | 0.0027 | 0.9904 | 13/16/6 | no | 128.09 | 24.82 | 0.8333 | 3/0 | - |
| 07-math-display | lualatex-lm | de1020c | 1/1 | ok | 0.3495 | 255 | 0.0029 | 0.0024 | 0.9931 | 13/10/7 | no | 2.12 | 0.92 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex-lm | main | 1/1 | recovered | 0.4121 | 255 | 0.0032 | 0.0026 | 0.9903 | 13/16/6 | no | 126.11 | 25.17 | 0.8333 | 3/0 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.2609 | 255 | 0.0026 | 0.0019 | 0.9949 | 13/10/7 | no | 0.59 | 1.41 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-de1020c-export-p1-overlay.png) |
| 07-math-display | pdflatex | main | 1/1 | recovered | 0.4223 | 255 | 0.0032 | 0.0026 | 0.9906 | 13/16/6 | no | 128.09 | 24.72 | 0.8333 | 3/0 | [p1](images/07-math-display/pdflatex-main-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | de1020c | 1/1 | ok | 0.3528 | 255 | 0.0029 | 0.0024 | 0.9931 | 13/10/7 | no | 2.13 | 1.73 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | main | 1/1 | recovered | 0.4118 | 255 | 0.0032 | 0.0026 | 0.9903 | 13/16/6 | no | 126.10 | 24.66 | 0.8333 | 3/0 | [p1](images/07-math-display/pdflatex-lm-main-export-p1-overlay.png) |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.2589 | 255 | 0.0026 | 0.0019 | 0.9950 | 14/10/7 | no | 0.59 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | recovered | 0.4264 | 255 | 0.0033 | 0.0026 | 0.9905 | 14/16/6 | no | 128.09 | 24.82 | 0.8333 | 3/0 | - |
| 07-math-display | xelatex-lm | de1020c | 1/1 | ok | 0.3495 | 255 | 0.0029 | 0.0024 | 0.9931 | 14/10/7 | no | 2.12 | 1.54 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex-lm | main | 1/1 | recovered | 0.4122 | 255 | 0.0032 | 0.0026 | 0.9903 | 14/16/6 | no | 126.11 | 24.86 | 0.8333 | 3/0 | - |
| 08-two-page | lualatex | de1020c | 3/3 | ok | 26.2176 | 255 | 0.1880 | 0.1574 | 0.5657 | 1800/1800/1800 | yes | 164.00 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | lualatex | main | 3/3 | ok | 25.6200 | 255 | 0.1849 | 0.1546 | 0.5672 | 1800/1800/1800 | yes | 156.63 | 152.37 | 0.8900 | 0/0 | - |
| 08-two-page | lualatex-lm | de1020c | 3/3 | ok | 23.7967 | 255 | 0.1801 | 0.1495 | 0.5633 | 1800/1800/1800 | yes | 142.51 | 30.71 | 0.8878 | 0/0 | - |
| 08-two-page | lualatex-lm | main | 3/3 | ok | 23.4169 | 255 | 0.1780 | 0.1476 | 0.5623 | 1800/1800/1800 | yes | 162.82 | 98.75 | 0.8949 | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/3 | ok | 26.1974 | 255 | 0.1875 | 0.1572 | 0.5667 | 1800/1800/1800 | yes | 143.13 | 92.17 | 0.8952 | 0/0 | [p1](images/08-two-page/pdflatex-de1020c-export-p1-overlay.png) |
| 08-two-page | pdflatex | main | 3/3 | ok | 25.6180 | 255 | 0.1843 | 0.1544 | 0.5683 | 1800/1800/1800 | yes | 152.18 | 151.60 | 0.9008 | 0/0 | [p1](images/08-two-page/pdflatex-main-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | de1020c | 3/3 | ok | 23.7963 | 255 | 0.1801 | 0.1496 | 0.5633 | 1800/1800/1800 | yes | 142.52 | 31.52 | 0.8878 | 0/0 | [p1](images/08-two-page/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | main | 3/3 | ok | 23.4160 | 255 | 0.1780 | 0.1477 | 0.5623 | 1800/1800/1800 | yes | 162.82 | 99.71 | 0.8949 | 0/0 | [p1](images/08-two-page/pdflatex-lm-main-export-p1-overlay.png) |
| 08-two-page | xelatex | de1020c | 3/3 | ok | 26.2425 | 255 | 0.1881 | 0.1578 | 0.5656 | 1800/1800/1800 | yes | 164.01 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | xelatex | main | 3/3 | ok | 25.6193 | 255 | 0.1849 | 0.1549 | 0.5676 | 1800/1800/1800 | yes | 156.64 | 152.37 | 0.8900 | 0/0 | - |
| 08-two-page | xelatex-lm | de1020c | 3/3 | ok | 23.7969 | 255 | 0.1801 | 0.1495 | 0.5633 | 1800/1800/1800 | yes | 142.51 | 31.52 | 0.8878 | 0/0 | - |
| 08-two-page | xelatex-lm | main | 3/3 | ok | 23.4169 | 255 | 0.1780 | 0.1476 | 0.5623 | 1800/1800/1800 | yes | 162.82 | 99.71 | 0.8949 | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.4844 | 255 | 0.0174 | 0.0146 | 0.9504 | 54/54/45 | no | 2.41 | 31.76 | 0.9778 | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | recovered | 2.3211 | 255 | 0.0164 | 0.0137 | 0.9553 | 54/54/39 | no | 104.96 | 18.76 | 0.8974 | 0/0 | - |
| 09-mixed-document | lualatex-lm | de1020c | 1/1 | ok | 2.2538 | 255 | 0.0169 | 0.0140 | 0.9494 | 54/54/45 | no | 30.16 | 31.13 | 0.9333 | 0/0 | - |
| 09-mixed-document | lualatex-lm | main | 1/1 | recovered | 2.1482 | 255 | 0.0160 | 0.0133 | 0.9546 | 54/54/39 | no | 79.56 | 18.16 | 0.8718 | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.4853 | 255 | 0.0174 | 0.0146 | 0.9504 | 54/54/45 | no | 2.48 | 31.80 | 0.9778 | 0/0 | [p1](images/09-mixed-document/pdflatex-de1020c-export-p1-overlay.png) |
| 09-mixed-document | pdflatex | main | 1/1 | recovered | 2.3188 | 255 | 0.0164 | 0.0137 | 0.9553 | 54/54/39 | no | 105.07 | 18.80 | 0.8974 | 0/0 | [p1](images/09-mixed-document/pdflatex-main-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | de1020c | 1/1 | ok | 2.2522 | 255 | 0.0167 | 0.0139 | 0.9480 | 54/54/45 | no | 30.17 | 32.47 | 0.9333 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | main | 1/1 | recovered | 2.1698 | 255 | 0.0160 | 0.0133 | 0.9532 | 54/54/39 | no | 79.55 | 19.41 | 0.8974 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-main-export-p1-overlay.png) |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.4838 | 255 | 0.0174 | 0.0146 | 0.9503 | 54/54/45 | no | 2.45 | 31.76 | 0.9778 | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | recovered | 2.3191 | 255 | 0.0164 | 0.0137 | 0.9553 | 54/54/39 | no | 105.07 | 18.76 | 0.8974 | 0/0 | - |
| 09-mixed-document | xelatex-lm | de1020c | 1/1 | ok | 2.2539 | 255 | 0.0169 | 0.0140 | 0.9494 | 54/54/45 | no | 30.16 | 32.06 | 0.9333 | 0/0 | - |
| 09-mixed-document | xelatex-lm | main | 1/1 | recovered | 2.1483 | 255 | 0.0160 | 0.0133 | 0.9546 | 54/54/39 | no | 79.56 | 19.00 | 0.8974 | 0/0 | - |

## Diagnostic: preview-equivalent comparison (CoreText draw of compile_result vs reference PDF raster)

Weaker than a capture of the real preview: it re-implements the app's draw code path rather than exercising the SwiftUI Canvas. Word-box metrics are not available for this side (no PDF), so they are omitted.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| | max | differing | ≥thr | SSIM₈ | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-plain-paragraph | lualatex | de1020c | 1/1 | ok | 0.1686 | 255 | 0.0023 | 0.0014 | 0.9978 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.4049 | 255 | 0.0032 | 0.0025 | 0.9937 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | de1020c | 1/1 | ok | 0.3642 | 255 | 0.0030 | 0.0025 | 0.9942 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | main | 1/1 | ok | 0.3887 | 255 | 0.0032 | 0.0026 | 0.9934 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.1741 | 255 | 0.0023 | 0.0014 | 0.9977 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.3958 | 255 | 0.0032 | 0.0025 | 0.9940 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 0.3639 | 255 | 0.0030 | 0.0024 | 0.9942 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | main | 1/1 | ok | 0.3883 | 255 | 0.0031 | 0.0026 | 0.9934 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.1716 | 255 | 0.0023 | 0.0014 | 0.9978 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.4056 | 255 | 0.0032 | 0.0025 | 0.9937 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | de1020c | 1/1 | ok | 0.3644 | 255 | 0.0030 | 0.0025 | 0.9942 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | main | 1/1 | ok | 0.3888 | 255 | 0.0032 | 0.0026 | 0.9934 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 8.3831 | 255 | 0.0613 | 0.0509 | 0.8656 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 8.4759 | 255 | 0.0621 | 0.0516 | 0.8582 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | de1020c | 1/1 | ok | 7.7865 | 255 | 0.0597 | 0.0495 | 0.8620 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | main | 1/1 | ok | 7.8537 | 255 | 0.0601 | 0.0498 | 0.8552 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 8.3720 | 255 | 0.0611 | 0.0509 | 0.8664 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 8.5059 | 255 | 0.0620 | 0.0516 | 0.8583 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 7.7859 | 255 | 0.0597 | 0.0495 | 0.8621 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | main | 1/1 | ok | 7.8525 | 255 | 0.0600 | 0.0498 | 0.8553 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 8.3703 | 255 | 0.0613 | 0.0509 | 0.8657 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 8.4944 | 255 | 0.0621 | 0.0518 | 0.8579 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | de1020c | 1/1 | ok | 7.7862 | 255 | 0.0597 | 0.0495 | 0.8620 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | main | 1/1 | ok | 7.8534 | 255 | 0.0600 | 0.0498 | 0.8552 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.3394 | 255 | 0.0085 | 0.0073 | 0.9793 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.2662 | 255 | 0.0081 | 0.0070 | 0.9802 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | de1020c | 1/1 | ok | 1.2336 | 255 | 0.0082 | 0.0069 | 0.9771 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | main | 1/1 | ok | 1.1783 | 255 | 0.0079 | 0.0067 | 0.9780 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.3496 | 255 | 0.0085 | 0.0072 | 0.9793 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.2820 | 255 | 0.0082 | 0.0070 | 0.9801 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | de1020c | 1/1 | ok | 1.2335 | 255 | 0.0082 | 0.0069 | 0.9771 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | main | 1/1 | ok | 1.1779 | 255 | 0.0079 | 0.0067 | 0.9781 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.3370 | 255 | 0.0085 | 0.0073 | 0.9793 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.2663 | 255 | 0.0081 | 0.0071 | 0.9802 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | de1020c | 1/1 | ok | 1.2337 | 255 | 0.0082 | 0.0069 | 0.9771 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | main | 1/1 | ok | 1.1784 | 255 | 0.0079 | 0.0067 | 0.9780 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.3808 | 255 | 0.0031 | 0.0024 | 0.9945 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.4195 | 255 | 0.0033 | 0.0026 | 0.9939 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | de1020c | 1/1 | ok | 0.4244 | 255 | 0.0033 | 0.0027 | 0.9933 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | main | 1/1 | ok | 0.4314 | 255 | 0.0033 | 0.0027 | 0.9931 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.3837 | 255 | 0.0031 | 0.0024 | 0.9944 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.4290 | 255 | 0.0034 | 0.0026 | 0.9937 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | de1020c | 1/1 | ok | 0.4246 | 255 | 0.0033 | 0.0027 | 0.9933 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | main | 1/1 | ok | 0.4326 | 255 | 0.0033 | 0.0027 | 0.9931 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.3849 | 255 | 0.0032 | 0.0025 | 0.9944 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.4209 | 255 | 0.0033 | 0.0026 | 0.9939 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | de1020c | 1/1 | ok | 0.4245 | 255 | 0.0033 | 0.0027 | 0.9933 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | main | 1/1 | ok | 0.4289 | 255 | 0.0034 | 0.0027 | 0.9932 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.2971 | 255 | 0.0027 | 0.0020 | 0.9955 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.3747 | 255 | 0.0030 | 0.0023 | 0.9941 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex-lm | de1020c | 1/1 | ok | 0.3619 | 255 | 0.0029 | 0.0024 | 0.9938 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | main | 1/1 | ok | 0.3859 | 255 | 0.0030 | 0.0024 | 0.9932 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.3125 | 255 | 0.0027 | 0.0020 | 0.9955 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.3712 | 255 | 0.0030 | 0.0023 | 0.9945 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex-lm | de1020c | 1/1 | ok | 0.3618 | 255 | 0.0029 | 0.0024 | 0.9938 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex-lm | main | 1/1 | ok | 0.3854 | 255 | 0.0030 | 0.0024 | 0.9932 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.2963 | 255 | 0.0027 | 0.0020 | 0.9955 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.3752 | 255 | 0.0030 | 0.0023 | 0.9941 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex-lm | de1020c | 1/1 | ok | 0.3620 | 255 | 0.0029 | 0.0024 | 0.9938 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | main | 1/1 | ok | 0.3859 | 255 | 0.0030 | 0.0024 | 0.9932 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.3823 | 255 | 0.0029 | 0.0023 | 0.9934 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | recovered | 0.2455 | 255 | 0.0023 | 0.0017 | 0.9958 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | de1020c | 1/1 | ok | 0.3683 | 255 | 0.0029 | 0.0023 | 0.9931 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | main | 1/1 | recovered | 0.2606 | 255 | 0.0022 | 0.0018 | 0.9953 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.3812 | 255 | 0.0029 | 0.0023 | 0.9936 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | main | 1/1 | recovered | 0.2741 | 255 | 0.0023 | 0.0018 | 0.9955 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | de1020c | 1/1 | ok | 0.3681 | 255 | 0.0029 | 0.0023 | 0.9931 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | main | 1/1 | recovered | 0.2610 | 255 | 0.0022 | 0.0018 | 0.9953 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.3823 | 255 | 0.0029 | 0.0023 | 0.9934 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | recovered | 0.2451 | 255 | 0.0023 | 0.0016 | 0.9958 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | de1020c | 1/1 | ok | 0.3684 | 255 | 0.0029 | 0.0023 | 0.9931 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | main | 1/1 | recovered | 0.2606 | 255 | 0.0022 | 0.0018 | 0.9953 | -/-/- | - | - | - | - | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.2575 | 255 | 0.0026 | 0.0019 | 0.9950 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | recovered | 0.4269 | 255 | 0.0033 | 0.0027 | 0.9904 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | lualatex-lm | de1020c | 1/1 | ok | 0.3485 | 255 | 0.0029 | 0.0024 | 0.9931 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | main | 1/1 | recovered | 0.4120 | 255 | 0.0032 | 0.0026 | 0.9903 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.2593 | 255 | 0.0026 | 0.0019 | 0.9950 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex | main | 1/1 | recovered | 0.4222 | 255 | 0.0032 | 0.0026 | 0.9906 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | pdflatex-lm | de1020c | 1/1 | ok | 0.3516 | 255 | 0.0029 | 0.0024 | 0.9931 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex-lm | main | 1/1 | recovered | 0.4117 | 255 | 0.0032 | 0.0026 | 0.9903 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.2573 | 255 | 0.0026 | 0.0019 | 0.9950 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | recovered | 0.4264 | 255 | 0.0033 | 0.0026 | 0.9905 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | xelatex-lm | de1020c | 1/1 | ok | 0.3485 | 255 | 0.0029 | 0.0024 | 0.9931 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | main | 1/1 | recovered | 0.4121 | 255 | 0.0032 | 0.0026 | 0.9903 | -/-/- | - | - | - | - | 3/0 | - |
| 08-two-page | lualatex | de1020c | 3/3 | ok | 26.2189 | 255 | 0.1880 | 0.1574 | 0.5659 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex | main | 3/3 | ok | 25.6206 | 255 | 0.1848 | 0.1547 | 0.5674 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | de1020c | 3/3 | ok | 23.7956 | 255 | 0.1801 | 0.1495 | 0.5635 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | main | 3/3 | ok | 23.4170 | 255 | 0.1780 | 0.1476 | 0.5625 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/3 | ok | 26.1972 | 255 | 0.1875 | 0.1571 | 0.5669 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | main | 3/3 | ok | 25.6175 | 255 | 0.1843 | 0.1545 | 0.5685 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | de1020c | 3/3 | ok | 23.7951 | 255 | 0.1800 | 0.1496 | 0.5635 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | main | 3/3 | ok | 23.4162 | 255 | 0.1780 | 0.1477 | 0.5625 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | de1020c | 3/3 | ok | 26.2441 | 255 | 0.1881 | 0.1577 | 0.5658 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | main | 3/3 | ok | 25.6199 | 255 | 0.1849 | 0.1549 | 0.5678 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | de1020c | 3/3 | ok | 23.7957 | 255 | 0.1800 | 0.1495 | 0.5634 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | main | 3/3 | ok | 23.4170 | 255 | 0.1780 | 0.1476 | 0.5625 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.4844 | 255 | 0.0174 | 0.0146 | 0.9504 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | recovered | 2.3216 | 255 | 0.0164 | 0.0137 | 0.9553 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | de1020c | 1/1 | ok | 2.2538 | 255 | 0.0169 | 0.0140 | 0.9494 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | main | 1/1 | recovered | 2.1482 | 255 | 0.0160 | 0.0133 | 0.9546 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.4854 | 255 | 0.0174 | 0.0146 | 0.9504 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | main | 1/1 | recovered | 2.3187 | 255 | 0.0164 | 0.0137 | 0.9553 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | de1020c | 1/1 | ok | 2.2522 | 255 | 0.0167 | 0.0139 | 0.9480 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | main | 1/1 | recovered | 2.1696 | 255 | 0.0160 | 0.0133 | 0.9532 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.4838 | 255 | 0.0174 | 0.0146 | 0.9503 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | recovered | 2.3191 | 255 | 0.0164 | 0.0137 | 0.9553 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | de1020c | 1/1 | ok | 2.2538 | 255 | 0.0169 | 0.0140 | 0.9494 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | main | 1/1 | recovered | 2.1483 | 255 | 0.0160 | 0.0133 | 0.9546 | -/-/- | - | - | - | - | 0/0 | - |

## Diagnostic: native preview capture comparison (screen capture of the running FlashTeXMac preview vs reference PDF raster)

The actual SwiftUI Canvas preview, captured with `screencapture -l <window id>`, page region detected and resampled to the reference raster size (resample factor 2.839907192575406–2.839907192575406, display backing [1.0] px/pt; see provenance). The 'page N' caption corner is masked white. Word-box metrics are unavailable (a screenshot has no text layer). This is a separate, independent comparison from the export table above and from the weaker preview-equivalent table.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| | max | differing | ≥thr | SSIM₈ | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-plain-paragraph | lualatex | de1020c | 1/1 | ok | 0.4486 | 248 | 0.0191 | 0.0035 | 0.9928 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.5179 | 253 | 0.0215 | 0.0040 | 0.9916 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | de1020c | 1/1 | ok | 0.4526 | 255 | 0.0192 | 0.0038 | 0.9923 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | main | 1/1 | ok | 0.4644 | 252 | 0.0215 | 0.0039 | 0.9918 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.4239 | 248 | 0.0191 | 0.0034 | 0.9936 | -/-/- | - | - | - | - | 0/0 | [p1](images/01-plain-paragraph/pdflatex-de1020c-native-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.5124 | 253 | 0.0216 | 0.0040 | 0.9918 | -/-/- | - | - | - | - | 0/0 | [p1](images/01-plain-paragraph/pdflatex-main-native-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 0.4525 | 255 | 0.0192 | 0.0038 | 0.9923 | -/-/- | - | - | - | - | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | main | 1/1 | ok | 0.4642 | 252 | 0.0215 | 0.0039 | 0.9918 | -/-/- | - | - | - | - | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-main-native-p1-overlay.png) |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.4492 | 248 | 0.0191 | 0.0035 | 0.9928 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.5175 | 253 | 0.0216 | 0.0040 | 0.9916 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | de1020c | 1/1 | ok | 0.4527 | 255 | 0.0192 | 0.0038 | 0.9923 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | main | 1/1 | ok | 0.4645 | 252 | 0.0216 | 0.0039 | 0.9918 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 9.2863 | 255 | 0.1842 | 0.0722 | 0.8623 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 9.5766 | 255 | 0.2093 | 0.0736 | 0.8523 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | de1020c | 1/1 | ok | 8.6958 | 255 | 0.1846 | 0.0710 | 0.8597 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | main | 1/1 | ok | 8.7769 | 255 | 0.2092 | 0.0713 | 0.8533 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 9.2884 | 255 | 0.1842 | 0.0722 | 0.8628 | -/-/- | - | - | - | - | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-de1020c-native-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 9.5386 | 255 | 0.2093 | 0.0734 | 0.8534 | -/-/- | - | - | - | - | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-main-native-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 8.6949 | 255 | 0.1846 | 0.0710 | 0.8597 | -/-/- | - | - | - | - | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | main | 1/1 | ok | 8.7764 | 255 | 0.2092 | 0.0713 | 0.8533 | -/-/- | - | - | - | - | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-main-native-p1-overlay.png) |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 9.2871 | 255 | 0.1842 | 0.0722 | 0.8622 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 9.5719 | 255 | 0.2093 | 0.0736 | 0.8524 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | de1020c | 1/1 | ok | 8.6957 | 255 | 0.1846 | 0.0710 | 0.8597 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | main | 1/1 | ok | 8.7771 | 255 | 0.2092 | 0.0713 | 0.8533 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.5624 | 255 | 0.0367 | 0.0106 | 0.9775 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.4869 | 255 | 0.0379 | 0.0102 | 0.9789 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | de1020c | 1/1 | ok | 1.4569 | 255 | 0.0366 | 0.0101 | 0.9754 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | main | 1/1 | ok | 1.4033 | 255 | 0.0378 | 0.0098 | 0.9767 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.5713 | 255 | 0.0367 | 0.0105 | 0.9775 | -/-/- | - | - | - | - | 0/0 | [p1](images/03-section-heading/pdflatex-de1020c-native-p1-overlay.png) |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.4964 | 255 | 0.0379 | 0.0101 | 0.9789 | -/-/- | - | - | - | - | 0/0 | [p1](images/03-section-heading/pdflatex-main-native-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | de1020c | 1/1 | ok | 1.4569 | 255 | 0.0366 | 0.0101 | 0.9754 | -/-/- | - | - | - | - | 0/0 | [p1](images/03-section-heading/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | main | 1/1 | ok | 1.4032 | 255 | 0.0378 | 0.0098 | 0.9767 | -/-/- | - | - | - | - | 0/0 | [p1](images/03-section-heading/pdflatex-lm-main-native-p1-overlay.png) |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.5598 | 255 | 0.0368 | 0.0106 | 0.9775 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.4852 | 255 | 0.0379 | 0.0102 | 0.9790 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | de1020c | 1/1 | ok | 1.4569 | 255 | 0.0366 | 0.0101 | 0.9754 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | main | 1/1 | ok | 1.4033 | 255 | 0.0378 | 0.0098 | 0.9767 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.4841 | 251 | 0.0193 | 0.0038 | 0.9925 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.5209 | 254 | 0.0207 | 0.0040 | 0.9918 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | de1020c | 1/1 | ok | 0.4914 | 255 | 0.0195 | 0.0040 | 0.9917 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | main | 1/1 | ok | 0.4878 | 253 | 0.0207 | 0.0039 | 0.9916 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.4954 | 251 | 0.0193 | 0.0038 | 0.9922 | -/-/- | - | - | - | - | 0/0 | [p1](images/04-bold-emph/pdflatex-de1020c-native-p1-overlay.png) |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.5178 | 254 | 0.0207 | 0.0040 | 0.9918 | -/-/- | - | - | - | - | 0/0 | [p1](images/04-bold-emph/pdflatex-main-native-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | de1020c | 1/1 | ok | 0.4914 | 255 | 0.0195 | 0.0040 | 0.9917 | -/-/- | - | - | - | - | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | main | 1/1 | ok | 0.4882 | 253 | 0.0207 | 0.0039 | 0.9915 | -/-/- | - | - | - | - | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-main-native-p1-overlay.png) |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.4857 | 251 | 0.0193 | 0.0038 | 0.9925 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.5202 | 254 | 0.0207 | 0.0040 | 0.9918 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | de1020c | 1/1 | ok | 0.4914 | 255 | 0.0195 | 0.0040 | 0.9917 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | main | 1/1 | ok | 0.4853 | 253 | 0.0207 | 0.0039 | 0.9916 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.4392 | 253 | 0.0193 | 0.0035 | 0.9929 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.4572 | 253 | 0.0211 | 0.0035 | 0.9924 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex-lm | de1020c | 1/1 | ok | 0.4205 | 253 | 0.0193 | 0.0035 | 0.9923 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | main | 1/1 | ok | 0.4305 | 252 | 0.0211 | 0.0035 | 0.9919 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.4323 | 253 | 0.0193 | 0.0034 | 0.9932 | -/-/- | - | - | - | - | 2/0 | [p1](images/05-unicode/pdflatex-de1020c-native-p1-overlay.png) |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.4633 | 253 | 0.0211 | 0.0036 | 0.9925 | -/-/- | - | - | - | - | 2/0 | [p1](images/05-unicode/pdflatex-main-native-p1-overlay.png) |
| 05-unicode | pdflatex-lm | de1020c | 1/1 | ok | 0.4206 | 253 | 0.0193 | 0.0035 | 0.9923 | -/-/- | - | - | - | - | 0/0 | [p1](images/05-unicode/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 05-unicode | pdflatex-lm | main | 1/1 | ok | 0.4304 | 252 | 0.0211 | 0.0035 | 0.9919 | -/-/- | - | - | - | - | 0/0 | [p1](images/05-unicode/pdflatex-lm-main-native-p1-overlay.png) |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.4398 | 253 | 0.0193 | 0.0035 | 0.9929 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.4572 | 253 | 0.0211 | 0.0035 | 0.9924 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex-lm | de1020c | 1/1 | ok | 0.4206 | 253 | 0.0193 | 0.0035 | 0.9923 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | main | 1/1 | ok | 0.4305 | 252 | 0.0211 | 0.0035 | 0.9919 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.4667 | 255 | 0.0163 | 0.0036 | 0.9920 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | recovered | 0.3573 | 253 | 0.0161 | 0.0028 | 0.9935 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | de1020c | 1/1 | ok | 0.4422 | 255 | 0.0165 | 0.0035 | 0.9918 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | main | 1/1 | recovered | 0.3295 | 252 | 0.0161 | 0.0027 | 0.9935 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.4635 | 254 | 0.0163 | 0.0036 | 0.9922 | -/-/- | - | - | - | - | 0/0 | [p1](images/06-math-inline/pdflatex-de1020c-native-p1-overlay.png) |
| 06-math-inline | pdflatex | main | 1/1 | recovered | 0.3521 | 253 | 0.0161 | 0.0028 | 0.9936 | -/-/- | - | - | - | - | 0/0 | [p1](images/06-math-inline/pdflatex-main-native-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | de1020c | 1/1 | ok | 0.4420 | 255 | 0.0165 | 0.0035 | 0.9918 | -/-/- | - | - | - | - | 0/0 | [p1](images/06-math-inline/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | main | 1/1 | recovered | 0.3295 | 253 | 0.0161 | 0.0027 | 0.9935 | -/-/- | - | - | - | - | 0/0 | [p1](images/06-math-inline/pdflatex-lm-main-native-p1-overlay.png) |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.4669 | 255 | 0.0163 | 0.0036 | 0.9920 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | recovered | 0.3576 | 253 | 0.0161 | 0.0029 | 0.9935 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | de1020c | 1/1 | ok | 0.4422 | 255 | 0.0165 | 0.0035 | 0.9918 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | main | 1/1 | recovered | 0.3295 | 252 | 0.0161 | 0.0027 | 0.9935 | -/-/- | - | - | - | - | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.4193 | 255 | 0.0161 | 0.0034 | 0.9923 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | lualatex | main | 1/1 | recovered | 0.5273 | 255 | 0.0206 | 0.0041 | 0.9890 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | lualatex-lm | de1020c | 1/1 | ok | 0.4127 | 255 | 0.0161 | 0.0034 | 0.9919 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | lualatex-lm | main | 1/1 | recovered | 0.4980 | 255 | 0.0205 | 0.0040 | 0.9889 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.4183 | 255 | 0.0161 | 0.0034 | 0.9924 | -/-/- | - | - | - | - | 3/0 | [p1](images/07-math-display/pdflatex-de1020c-native-p1-overlay.png) |
| 07-math-display | pdflatex | main | 1/1 | recovered | 0.5268 | 255 | 0.0205 | 0.0041 | 0.9891 | -/-/- | - | - | - | - | 3/0 | [p1](images/07-math-display/pdflatex-main-native-p1-overlay.png) |
| 07-math-display | pdflatex-lm | de1020c | 1/1 | ok | 0.4159 | 255 | 0.0161 | 0.0035 | 0.9918 | -/-/- | - | - | - | - | 3/0 | [p1](images/07-math-display/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 07-math-display | pdflatex-lm | main | 1/1 | recovered | 0.4980 | 255 | 0.0205 | 0.0040 | 0.9889 | -/-/- | - | - | - | - | 3/0 | [p1](images/07-math-display/pdflatex-lm-main-native-p1-overlay.png) |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.4194 | 255 | 0.0161 | 0.0034 | 0.9923 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | xelatex | main | 1/1 | recovered | 0.5274 | 255 | 0.0206 | 0.0041 | 0.9890 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | xelatex-lm | de1020c | 1/1 | ok | 0.4128 | 255 | 0.0161 | 0.0034 | 0.9919 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | xelatex-lm | main | 1/1 | recovered | 0.4980 | 255 | 0.0205 | 0.0040 | 0.9889 | -/-/- | - | - | - | - | 3/0 | - |
| 08-two-page | lualatex | de1020c | 3/1 | ok | 34.0122 | 255 | 0.6194 | 0.2633 | 0.4781 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex | main | 3/1 | ok | 32.9255 | 255 | 0.6257 | 0.2502 | 0.4850 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | de1020c | 3/1 | ok | 30.1956 | 255 | 0.6171 | 0.2478 | 0.4959 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | main | 3/1 | ok | 28.8755 | 255 | 0.6246 | 0.2339 | 0.5023 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/1 | ok | 34.0037 | 255 | 0.6193 | 0.2631 | 0.4787 | -/-/- | - | - | - | - | 0/0 | [p1](images/08-two-page/pdflatex-de1020c-native-p1-overlay.png) |
| 08-two-page | pdflatex | main | 3/1 | ok | 32.8296 | 255 | 0.6257 | 0.2496 | 0.4884 | -/-/- | - | - | - | - | 0/0 | [p1](images/08-two-page/pdflatex-main-native-p1-overlay.png) |
| 08-two-page | pdflatex-lm | de1020c | 3/1 | ok | 30.1933 | 255 | 0.6171 | 0.2479 | 0.4959 | -/-/- | - | - | - | - | 0/0 | [p1](images/08-two-page/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 08-two-page | pdflatex-lm | main | 3/1 | ok | 28.8744 | 255 | 0.6246 | 0.2340 | 0.5023 | -/-/- | - | - | - | - | 0/0 | [p1](images/08-two-page/pdflatex-lm-main-native-p1-overlay.png) |
| 08-two-page | xelatex | de1020c | 3/1 | ok | 33.9862 | 255 | 0.6194 | 0.2636 | 0.4787 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | main | 3/1 | ok | 32.9069 | 255 | 0.6258 | 0.2505 | 0.4857 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | de1020c | 3/1 | ok | 30.1958 | 255 | 0.6171 | 0.2478 | 0.4959 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | main | 3/1 | ok | 28.8761 | 255 | 0.6246 | 0.2339 | 0.5023 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.8592 | 255 | 0.0763 | 0.0211 | 0.9507 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | recovered | 2.6696 | 255 | 0.0786 | 0.0200 | 0.9540 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | de1020c | 1/1 | ok | 2.6291 | 255 | 0.0758 | 0.0205 | 0.9497 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | main | 1/1 | recovered | 2.4636 | 255 | 0.0782 | 0.0194 | 0.9536 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.8586 | 255 | 0.0763 | 0.0211 | 0.9506 | -/-/- | - | - | - | - | 0/0 | [p1](images/09-mixed-document/pdflatex-de1020c-native-p1-overlay.png) |
| 09-mixed-document | pdflatex | main | 1/1 | recovered | 2.6725 | 255 | 0.0786 | 0.0200 | 0.9538 | -/-/- | - | - | - | - | 0/0 | [p1](images/09-mixed-document/pdflatex-main-native-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | de1020c | 1/1 | ok | 2.6271 | 255 | 0.0756 | 0.0204 | 0.9483 | -/-/- | - | - | - | - | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | main | 1/1 | recovered | 2.4810 | 255 | 0.0783 | 0.0194 | 0.9526 | -/-/- | - | - | - | - | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-main-native-p1-overlay.png) |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.8581 | 255 | 0.0763 | 0.0211 | 0.9506 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | recovered | 2.6779 | 255 | 0.0786 | 0.0201 | 0.9538 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | de1020c | 1/1 | ok | 2.6291 | 255 | 0.0758 | 0.0205 | 0.9497 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | main | 1/1 | recovered | 2.4636 | 255 | 0.0782 | 0.0194 | 0.9536 | -/-/- | - | - | - | - | 0/0 | - |

## Per-fixture diagnostic details (export side)

### 01-plain-paragraph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935537, 648, 477, 427, 319, 237, 160, 146, 149, 113, 138, 79, 113, 76, 54, 143]`; ink px ref/ours 2442/2421 (ratio 0.9914); SSIM blocks <0.9: 131/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-2.0, 0.0] pt; after undoing it: mean|Δ| 0.4931, differing 0.003386, SSIM₈ 0.9929 (vs unregistered 0.1575, 0.002279, 0.998) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `This` dx -0.75 dy 0.46; `is` dx -0.65 dy 0.46; `one` dx -0.62 dy 0.46

### 01-plain-paragraph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933434, 485, 443, 352, 320, 296, 304, 271, 268, 250, 228, 284, 235, 225, 285, 1136]`; ink px ref/ours 2442/2430 (ratio 0.9951); SSIM blocks <0.9: 241/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [21.5, 0.0] pt; after undoing it: mean|Δ| 0.4788, differing 0.003455, SSIM₈ 0.9927 (vs unregistered 0.4024, 0.00322, 0.9937) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx 54.81 dy 0.46; `single` dx 47.0 dy 0.46; `a` dx 45.93 dy 0.46

### 01-plain-paragraph — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933594, 461, 436, 345, 357, 334, 344, 324, 320, 261, 321, 233, 248, 209, 197, 832]`; ink px ref/ours 1890/2421 (ratio 1.281); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-10.5, 0.0] pt; after undoing it: mean|Δ| 0.4256, differing 0.00327, SSIM₈ 0.9932 (vs unregistered 0.3662, 0.003027, 0.9941) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx -24.74 dy -0.3; `single` dx -23.83 dy -0.3; `a` dx -22.41 dy -0.3

### 01-plain-paragraph — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933318, 461, 487, 322, 364, 359, 350, 361, 350, 268, 260, 246, 230, 233, 215, 992]`; ink px ref/ours 1890/2430 (ratio 1.2857); SSIM blocks <0.9: 252/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [13.0, 0.0] pt; after undoing it: mean|Δ| 0.4016, differing 0.003188, SSIM₈ 0.9933 (vs unregistered 0.3931, 0.003178, 0.9933) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx 29.94 dy -0.3; `on` dx 24.45 dy -0.3; `a` dx 23.55 dy -0.3

### 01-plain-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935388, 716, 424, 359, 290, 196, 196, 126, 136, 124, 122, 86, 78, 86, 81, 408]`; ink px ref/ours 2408/2421 (ratio 1.0054); SSIM blocks <0.9: 119/30294; [overlay](images/01-plain-paragraph/pdflatex-de1020c-export-p1-overlay.png) (21734 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (19120 B, ÷1)
  - registration (diagnostic): ink-centroid shift [-1.0, 0.0] pt; after undoing it: mean|Δ| 0.4485, differing 0.003181, SSIM₈ 0.9938 (vs unregistered 0.1873, 0.002361, 0.9975) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx 1.15 dy 0.46; `single` dx 1.03 dy 0.46; `a` dx 1.0 dy 0.46

### 01-plain-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933403, 559, 486, 299, 334, 257, 272, 310, 275, 265, 246, 249, 240, 235, 294, 1092]`; ink px ref/ours 2408/2430 (ratio 1.0091); SSIM blocks <0.9: 239/30294; [overlay](images/01-plain-paragraph/pdflatex-main-export-p1-overlay.png) (22432 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-main-export-p1-heatmap.png) (23482 B, ÷1)
  - registration (diagnostic): ink-centroid shift [22.5, 0.0] pt; after undoing it: mean|Δ| 0.4883, differing 0.003464, SSIM₈ 0.9929 (vs unregistered 0.3986, 0.003232, 0.9939) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx 55.83 dy 0.46; `single` dx 48.02 dy 0.46; `a` dx 46.96 dy 0.46

### 01-plain-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933593, 464, 437, 341, 371, 326, 338, 319, 328, 279, 309, 227, 246, 191, 211, 836]`; ink px ref/ours 1898/2421 (ratio 1.2756); SSIM blocks <0.9: 235/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (22469 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (23081 B, ÷1)
  - registration (diagnostic): ink-centroid shift [-9.5, 0.0] pt; after undoing it: mean|Δ| 0.417, differing 0.003226, SSIM₈ 0.9934 (vs unregistered 0.366, 0.003024, 0.9942) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx -24.75 dy 0.73; `single` dx -23.83 dy 0.73; `a` dx -22.41 dy 0.73

### 01-plain-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933329, 474, 458, 329, 378, 358, 352, 338, 361, 281, 245, 241, 233, 224, 218, 997]`; ink px ref/ours 1898/2430 (ratio 1.2803); SSIM blocks <0.9: 252/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-main-export-p1-overlay.png) (22623 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (23699 B, ÷1)
  - registration (diagnostic): ink-centroid shift [14.0, 0.0] pt; after undoing it: mean|Δ| 0.4223, differing 0.003245, SSIM₈ 0.993 (vs unregistered 0.3927, 0.003172, 0.9933) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx 29.93 dy 0.73; `on` dx 24.45 dy 0.73; `a` dx 23.55 dy 0.73

### 01-plain-paragraph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935492, 657, 509, 378, 314, 240, 190, 128, 161, 125, 139, 77, 101, 98, 61, 146]`; ink px ref/ours 2441/2421 (ratio 0.9918); SSIM blocks <0.9: 134/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-2.0, 0.0] pt; after undoing it: mean|Δ| 0.4941, differing 0.003385, SSIM₈ 0.9929 (vs unregistered 0.1607, 0.002287, 0.998) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `This` dx -0.75 dy 0.46; `is` dx -0.66 dy 0.46; `one` dx -0.62 dy 0.46

### 01-plain-paragraph — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933436, 464, 463, 334, 325, 294, 301, 272, 286, 245, 254, 249, 243, 222, 275, 1153]`; ink px ref/ours 2441/2430 (ratio 0.9955); SSIM blocks <0.9: 241/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [21.5, 0.0] pt; after undoing it: mean|Δ| 0.4791, differing 0.003452, SSIM₈ 0.9927 (vs unregistered 0.403, 0.003215, 0.9937) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx 54.78 dy 0.46; `single` dx 46.97 dy 0.46; `a` dx 45.91 dy 0.46

### 01-plain-paragraph — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933595, 460, 437, 342, 360, 330, 346, 325, 305, 282, 301, 246, 245, 198, 207, 837]`; ink px ref/ours 1891/2421 (ratio 1.2803); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-10.5, 0.0] pt; after undoing it: mean|Δ| 0.4257, differing 0.00327, SSIM₈ 0.9932 (vs unregistered 0.3665, 0.003027, 0.9941) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx -24.74 dy 0.73; `single` dx -23.83 dy 0.73; `a` dx -22.41 dy 0.73

### 01-plain-paragraph — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933324, 468, 475, 321, 365, 356, 353, 361, 339, 279, 249, 256, 232, 219, 226, 993]`; ink px ref/ours 1891/2430 (ratio 1.285); SSIM blocks <0.9: 252/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [13.0, 0.0] pt; after undoing it: mean|Δ| 0.4016, differing 0.003186, SSIM₈ 0.9933 (vs unregistered 0.3932, 0.00318, 0.9933) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx 29.94 dy 0.73; `on` dx 24.45 dy 0.73; `a` dx 23.55 dy 0.73

### 02-wrapping-paragraph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831807, 8259, 7034, 6392, 5883, 5835, 5922, 5218, 5432, 5617, 5346, 5301, 5198, 5202, 5188, 25182]`; ink px ref/ours 39630/39464 (ratio 0.9958); SSIM blocks <0.9: 4366/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-8.0, 4.5] pt; after undoing it: mean|Δ| 8.7706, differing 0.06321, SSIM₈ 0.8584 (vs unregistered 8.3842, 0.061305, 0.8656) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1830529, 8254, 7295, 6533, 5849, 5847, 6131, 5473, 5470, 5708, 5245, 5173, 5410, 5234, 5129, 25536]`; ink px ref/ours 39630/39432 (ratio 0.995); SSIM blocks <0.9: 4650/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-7.0, 15.5] pt; after undoing it: mean|Δ| 8.8428, differing 0.063653, SSIM₈ 0.8493 (vs unregistered 8.4755, 0.062117, 0.8581) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `every` dx -441.44 dy 29.03; `jumps` dx 425.04 dy 0.23; `oak` dx -415.78 dy 34.94

### 02-wrapping-paragraph — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834635, 8254, 7002, 6804, 6347, 6194, 6497, 6173, 6010, 5798, 5303, 5023, 4839, 4475, 4684, 20778]`; ink px ref/ours 30065/39464 (ratio 1.3126); SSIM blocks <0.9: 4465/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-12.5, 0.5] pt; after undoing it: mean|Δ| 7.835, differing 0.059948, SSIM₈ 0.8611 (vs unregistered 7.7868, 0.059701, 0.862) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `oak` dx 426.94 dy -14.75; `oak` dx 426.94 dy -9.07; `jumps` dx 416.92 dy -14.79

### 02-wrapping-paragraph — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1833918, 8296, 6971, 6865, 6351, 6165, 6582, 6391, 6106, 5825, 5163, 4805, 5065, 4666, 4643, 21004]`; ink px ref/ours 30065/39432 (ratio 1.3116); SSIM blocks <0.9: 4684/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-11.5, 12.0] pt; after undoing it: mean|Δ| 8.4249, differing 0.06371, SSIM₈ 0.8402 (vs unregistered 7.8526, 0.06005, 0.8552) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `fox` dx -419.35 dy 34.13; `fox` dx -419.35 dy 14.05; `brown` dx -417.19 dy 34.13

### 02-wrapping-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831980, 8181, 7016, 6406, 5963, 5948, 5852, 5289, 5188, 5656, 5267, 5378, 5255, 4876, 5221, 25340]`; ink px ref/ours 39390/39464 (ratio 1.0019); SSIM blocks <0.9: 4347/30294; [overlay](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-overlay.png) (32114 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (30971 B, ÷4)
  - registration (diagnostic): ink-centroid shift [-7.5, 5.0] pt; after undoing it: mean|Δ| 8.8672, differing 0.063197, SSIM₈ 0.8577 (vs unregistered 8.3726, 0.06114, 0.8663) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1830566, 8140, 7223, 6569, 5872, 6023, 5839, 5496, 5243, 5723, 5216, 5199, 5485, 5101, 5224, 25897]`; ink px ref/ours 39390/39432 (ratio 1.0011); SSIM blocks <0.9: 4640/30294; [overlay](images/02-wrapping-paragraph/pdflatex-main-export-p1-overlay.png) (32321 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-main-export-p1-heatmap.png) (32665 B, ÷4)
  - registration (diagnostic): ink-centroid shift [-6.5, 16.0] pt; after undoing it: mean|Δ| 8.7732, differing 0.063355, SSIM₈ 0.852 (vs unregistered 8.5059, 0.061969, 0.8582) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `every` dx -441.92 dy 29.03; `jumps` dx 425.04 dy 0.23; `oak` dx -415.72 dy 34.94

### 02-wrapping-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834636, 8219, 6952, 6893, 6423, 6140, 6491, 6184, 5976, 5757, 5362, 5028, 4837, 4417, 4790, 20711]`; ink px ref/ours 30080/39464 (ratio 1.312); SSIM blocks <0.9: 4465/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (31889 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (31275 B, ÷4)
  - registration (diagnostic): ink-centroid shift [-12.0, 1.0] pt; after undoing it: mean|Δ| 7.8805, differing 0.060286, SSIM₈ 0.8591 (vs unregistered 7.7862, 0.059683, 0.862) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.04; `jumps` dx 416.92 dy -13.77

### 02-wrapping-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1833953, 8230, 6974, 6905, 6403, 6107, 6617, 6350, 6137, 5792, 5191, 4793, 5048, 4633, 4748, 20935]`; ink px ref/ours 30080/39432 (ratio 1.3109); SSIM blocks <0.9: 4685/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-overlay.png) (32432 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (32280 B, ÷4)
  - registration (diagnostic): ink-centroid shift [-11.5, 12.0] pt; after undoing it: mean|Δ| 8.4252, differing 0.063685, SSIM₈ 0.8402 (vs unregistered 7.8514, 0.060014, 0.8552) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `fox` dx -419.35 dy 35.16; `fox` dx -419.35 dy 15.08; `brown` dx -417.2 dy 35.16

### 02-wrapping-paragraph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831948, 8135, 6967, 6429, 5850, 5891, 5942, 5391, 5434, 5606, 5386, 5299, 5280, 5136, 5055, 25067]`; ink px ref/ours 39556/39464 (ratio 0.9977); SSIM blocks <0.9: 4369/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-7.5, 5.0] pt; after undoing it: mean|Δ| 8.9306, differing 0.063578, SSIM₈ 0.8568 (vs unregistered 8.371, 0.06128, 0.8656) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `branch` dx -435.52 dy 20.54; `branch` dx -435.52 dy 14.86; `over` dx -406.99 dy 20.5

### 02-wrapping-paragraph — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1830415, 8062, 7253, 6686, 5876, 5895, 6016, 5628, 5414, 5736, 5274, 5145, 5553, 5263, 5069, 25531]`; ink px ref/ours 39556/39432 (ratio 0.9969); SSIM blocks <0.9: 4653/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-6.5, 16.0] pt; after undoing it: mean|Δ| 8.8095, differing 0.063668, SSIM₈ 0.8512 (vs unregistered 8.4942, 0.062133, 0.8578) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `every` dx -441.49 dy 29.03; `jumps` dx 425.04 dy 0.23; `oak` dx -415.83 dy 34.94

### 02-wrapping-paragraph — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834641, 8258, 6994, 6804, 6326, 6209, 6507, 6167, 6023, 5775, 5321, 5033, 4798, 4503, 4670, 20787]`; ink px ref/ours 30075/39464 (ratio 1.3122); SSIM blocks <0.9: 4467/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-12.5, 0.5] pt; after undoing it: mean|Δ| 7.835, differing 0.059939, SSIM₈ 0.8611 (vs unregistered 7.7866, 0.059691, 0.862) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.04; `jumps` dx 416.92 dy -13.77

### 02-wrapping-paragraph — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1833942, 8277, 6975, 6865, 6321, 6185, 6598, 6368, 6136, 5783, 5198, 4812, 5035, 4677, 4632, 21012]`; ink px ref/ours 30075/39432 (ratio 1.3111); SSIM blocks <0.9: 4686/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-11.5, 12.0] pt; after undoing it: mean|Δ| 8.4249, differing 0.063697, SSIM₈ 0.8401 (vs unregistered 7.8523, 0.060037, 0.8552) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `fox` dx -419.35 dy 35.16; `fox` dx -419.35 dy 15.08; `brown` dx -417.19 dy 35.16

### 03-section-heading — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923800, 859, 844, 658, 666, 649, 719, 577, 609, 557, 596, 606, 682, 626, 750, 5618]`; ink px ref/ours 5807/4705 (ratio 0.8102); SSIM blocks <0.9: 660/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [1.5, 21.0] pt; after undoing it: mean|Δ| 1.3132, differing 0.008142, SSIM₈ 0.9826 (vs unregistered 1.3394, 0.008456, 0.9793) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `heading.` dx 0.58 dy 26.34; `second` dx 0.44 dy 26.34; `a` dx 0.41 dy 26.34

### 03-section-heading — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924346, 807, 845, 662, 681, 672, 736, 575, 623, 629, 589, 584, 637, 622, 693, 5115]`; ink px ref/ours 5807/4731 (ratio 0.8147); SSIM blocks <0.9: 626/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [3.5, 22.0] pt; after undoing it: mean|Δ| 1.3315, differing 0.008366, SSIM₈ 0.9804 (vs unregistered 1.2663, 0.008128, 0.9802) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `heading.` dx 12.77 dy 32.34; `second` dx 9.6 dy 32.34; `a` dx 8.53 dy 32.34
- word-sequence differences: replace ref ['Body', 'text'] ours ['Bodytext']; replace ref ['More', 'body'] ours ['Morebody']

### 03-section-heading — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924467, 959, 778, 633, 646, 661, 738, 747, 647, 638, 596, 597, 643, 582, 684, 4800]`; ink px ref/ours 4775/4705 (ratio 0.9853); SSIM blocks <0.9: 718/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-2.5, 22.5] pt; after undoing it: mean|Δ| 1.1879, differing 0.007885, SSIM₈ 0.9808 (vs unregistered 1.2336, 0.008176, 0.9771) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `heading.` dx -12.94 dy 27.15; `second` dx -11.49 dy 27.15; `a` dx -10.06 dy 27.15

### 03-section-heading — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924926, 915, 756, 638, 635, 677, 762, 728, 638, 693, 577, 609, 607, 592, 633, 4430]`; ink px ref/ours 4775/4731 (ratio 0.9908); SSIM blocks <0.9: 686/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-0.5, 24.0] pt; after undoing it: mean|Δ| 1.2293, differing 0.008087, SSIM₈ 0.9786 (vs unregistered 1.1783, 0.007891, 0.978) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `text` dx -5.1 dy 33.15; `second` dx -2.33 dy 33.15; `under` dx -2.13 dy 33.15
- word-sequence differences: replace ref ['Body', 'text'] ours ['Bodytext']; replace ref ['More', 'body'] ours ['Morebody']

### 03-section-heading — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923855, 915, 733, 647, 632, 615, 615, 519, 606, 614, 675, 690, 668, 592, 707, 5733]`; ink px ref/ours 6093/4705 (ratio 0.7722); SSIM blocks <0.9: 662/30294; [overlay](images/03-section-heading/pdflatex-de1020c-export-p1-overlay.png) (34997 B, ÷1), [heatmap](images/03-section-heading/pdflatex-de1020c-export-p1-heatmap.png) (42932 B, ÷1)
  - registration (diagnostic): ink-centroid shift [1.5, 20.5] pt; after undoing it: mean|Δ| 1.3201, differing 0.008172, SSIM₈ 0.9823 (vs unregistered 1.3496, 0.008486, 0.9793) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `heading.` dx 0.75 dy 26.08; `second` dx 0.61 dy 26.08; `a` dx 0.58 dy 26.08

### 03-section-heading — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924386, 858, 723, 647, 631, 645, 611, 537, 599, 675, 654, 683, 648, 625, 649, 5245]`; ink px ref/ours 6093/4731 (ratio 0.7765); SSIM blocks <0.9: 628/30294; [overlay](images/03-section-heading/pdflatex-main-export-p1-overlay.png) (35002 B, ÷1), [heatmap](images/03-section-heading/pdflatex-main-export-p1-heatmap.png) (42656 B, ÷1)
  - registration (diagnostic): ink-centroid shift [3.5, 21.5] pt; after undoing it: mean|Δ| 1.3387, differing 0.008381, SSIM₈ 0.9795 (vs unregistered 1.2822, 0.008153, 0.9801) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `heading.` dx 12.94 dy 32.08; `second` dx 9.77 dy 32.08; `a` dx 8.7 dy 32.08
- word-sequence differences: replace ref ['Body', 'text'] ours ['Bodytext']; replace ref ['More', 'body'] ours ['Morebody']

### 03-section-heading — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924475, 952, 791, 621, 670, 652, 705, 745, 643, 649, 621, 599, 586, 629, 701, 4777]`; ink px ref/ours 4791/4705 (ratio 0.982); SSIM blocks <0.9: 718/30294; [overlay](images/03-section-heading/pdflatex-lm-de1020c-export-p1-overlay.png) (35334 B, ÷1), [heatmap](images/03-section-heading/pdflatex-lm-de1020c-export-p1-heatmap.png) (44120 B, ÷1)
  - registration (diagnostic): ink-centroid shift [-2.5, 22.5] pt; after undoing it: mean|Δ| 1.1888, differing 0.007875, SSIM₈ 0.9808 (vs unregistered 1.2335, 0.008161, 0.9771) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `heading.` dx -12.95 dy 28.22; `second` dx -11.5 dy 28.22; `a` dx -10.07 dy 28.22

### 03-section-heading — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924933, 903, 776, 625, 663, 669, 725, 728, 653, 688, 596, 604, 559, 637, 652, 4405]`; ink px ref/ours 4791/4731 (ratio 0.9875); SSIM blocks <0.9: 686/30294; [overlay](images/03-section-heading/pdflatex-lm-main-export-p1-overlay.png) (35460 B, ÷1), [heatmap](images/03-section-heading/pdflatex-lm-main-export-p1-heatmap.png) (43667 B, ÷1)
  - registration (diagnostic): ink-centroid shift [-0.5, 23.5] pt; after undoing it: mean|Δ| 1.224, differing 0.008042, SSIM₈ 0.9787 (vs unregistered 1.1779, 0.007871, 0.9781) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `text` dx -5.11 dy 34.22; `second` dx -2.34 dy 34.22; `under` dx -2.13 dy 34.22
- word-sequence differences: replace ref ['Body', 'text'] ours ['Bodytext']; replace ref ['More', 'body'] ours ['Morebody']

### 03-section-heading — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923747, 880, 847, 676, 683, 663, 736, 610, 596, 553, 616, 586, 654, 628, 733, 5608]`; ink px ref/ours 5744/4705 (ratio 0.8191); SSIM blocks <0.9: 660/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [1.5, 20.5] pt; after undoing it: mean|Δ| 1.3258, differing 0.008194, SSIM₈ 0.982 (vs unregistered 1.337, 0.008469, 0.9793) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `heading.` dx 0.57 dy 26.34; `second` dx 0.43 dy 26.34; `a` dx 0.4 dy 26.34

### 03-section-heading — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924301, 832, 850, 663, 678, 676, 737, 611, 606, 629, 619, 573, 624, 629, 667, 5121]`; ink px ref/ours 5744/4731 (ratio 0.8236); SSIM blocks <0.9: 626/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [3.5, 22.0] pt; after undoing it: mean|Δ| 1.3317, differing 0.008396, SSIM₈ 0.9804 (vs unregistered 1.2665, 0.008146, 0.9802) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `heading.` dx 12.76 dy 32.34; `second` dx 9.59 dy 32.34; `a` dx 8.52 dy 32.34
- word-sequence differences: replace ref ['Body', 'text'] ours ['Bodytext']; replace ref ['More', 'body'] ours ['Morebody']

### 03-section-heading — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924466, 954, 785, 634, 640, 664, 732, 752, 647, 641, 594, 601, 641, 581, 686, 4798]`; ink px ref/ours 4777/4705 (ratio 0.9849); SSIM blocks <0.9: 718/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-2.5, 22.5] pt; after undoing it: mean|Δ| 1.1879, differing 0.007886, SSIM₈ 0.9808 (vs unregistered 1.2337, 0.008177, 0.9771) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `heading.` dx -12.94 dy 28.18; `second` dx -11.49 dy 28.18; `a` dx -10.06 dy 28.18

### 03-section-heading — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924924, 911, 766, 633, 634, 674, 750, 742, 639, 693, 578, 613, 605, 591, 633, 4430]`; ink px ref/ours 4777/4731 (ratio 0.9904); SSIM blocks <0.9: 686/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-0.5, 24.0] pt; after undoing it: mean|Δ| 1.2293, differing 0.008088, SSIM₈ 0.9786 (vs unregistered 1.1784, 0.007892, 0.978) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `text` dx -5.1 dy 34.18; `second` dx -2.33 dy 34.18; `under` dx -2.13 dy 34.18
- word-sequence differences: replace ref ['Body', 'text'] ours ['Bodytext']; replace ref ['More', 'body'] ours ['Morebody']

### 04-bold-emph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933574, 491, 483, 352, 286, 278, 321, 275, 308, 283, 235, 240, 216, 217, 213, 1044]`; ink px ref/ours 2712/2465 (ratio 0.9089); SSIM blocks <0.9: 227/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [1.5, 0.0] pt; after undoing it: mean|Δ| 0.4632, differing 0.003359, SSIM₈ 0.9935 (vs unregistered 0.3808, 0.003147, 0.9945) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `emphasised` dx 5.28 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933226, 516, 453, 392, 323, 309, 314, 251, 256, 279, 230, 276, 221, 277, 228, 1265]`; ink px ref/ours 2712/2473 (ratio 0.9119); SSIM blocks <0.9: 251/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [22.5, 0.0] pt; after undoing it: mean|Δ| 0.4816, differing 0.00352, SSIM₈ 0.993 (vs unregistered 0.4195, 0.00332, 0.9939) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx 37.4 dy 0.46; `one` dx 36.31 dy 0.46; `on` dx 35.89 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933135, 462, 395, 357, 369, 349, 325, 343, 352, 298, 238, 265, 257, 274, 230, 1167]`; ink px ref/ours 2276/2465 (ratio 1.083); SSIM blocks <0.9: 258/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-17.5, 0.0] pt; after undoing it: mean|Δ| 0.4498, differing 0.00341, SSIM₈ 0.993 (vs unregistered 0.4244, 0.00332, 0.9933) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx -27.99 dy -0.63; `one` dx -26.86 dy -0.63; `on` dx -25.6 dy -0.63
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933064, 439, 414, 364, 346, 347, 330, 321, 345, 336, 287, 314, 251, 276, 253, 1129]`; ink px ref/ours 2276/2473 (ratio 1.0866); SSIM blocks <0.9: 265/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [3.5, 0.0] pt; after undoing it: mean|Δ| 0.4398, differing 0.003351, SSIM₈ 0.9931 (vs unregistered 0.4313, 0.003342, 0.9932) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `and` dx 17.95 dy -0.63; `bold` dx 16.55 dy -0.63; `emphasised` dx 15.62 dy -0.63
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933598, 513, 444, 319, 342, 300, 270, 233, 301, 263, 246, 223, 207, 251, 234, 1072]`; ink px ref/ours 2746/2465 (ratio 0.8977); SSIM blocks <0.9: 225/30294; [overlay](images/04-bold-emph/pdflatex-de1020c-export-p1-overlay.png) (23521 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-de1020c-export-p1-heatmap.png) (23530 B, ÷1)
  - registration (diagnostic): ink-centroid shift [3.5, 0.0] pt; after undoing it: mean|Δ| 0.4373, differing 0.003262, SSIM₈ 0.994 (vs unregistered 0.3836, 0.003143, 0.9944) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `emphasised` dx 5.27 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933132, 561, 432, 362, 327, 335, 272, 276, 263, 281, 255, 295, 202, 265, 251, 1307]`; ink px ref/ours 2746/2473 (ratio 0.9006); SSIM blocks <0.9: 254/30294; [overlay](images/04-bold-emph/pdflatex-main-export-p1-overlay.png) (23957 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-main-export-p1-heatmap.png) (24624 B, ÷1)
  - registration (diagnostic): ink-centroid shift [24.5, 0.0] pt; after undoing it: mean|Δ| 0.4928, differing 0.003557, SSIM₈ 0.9929 (vs unregistered 0.429, 0.003364, 0.9937) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx 37.97 dy 0.46; `one` dx 36.87 dy 0.46; `on` dx 36.45 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933105, 492, 380, 370, 376, 348, 330, 343, 330, 319, 248, 266, 243, 259, 233, 1174]`; ink px ref/ours 2268/2465 (ratio 1.0869); SSIM blocks <0.9: 259/30294; [overlay](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-overlay.png) (23967 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-heatmap.png) (24715 B, ÷1)
  - registration (diagnostic): ink-centroid shift [-17.0, 0.0] pt; after undoing it: mean|Δ| 0.4384, differing 0.003363, SSIM₈ 0.9932 (vs unregistered 0.4246, 0.003322, 0.9933) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx -28.1 dy 0.73; `one` dx -26.97 dy 0.73; `on` dx -25.72 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933053, 467, 407, 344, 342, 343, 333, 330, 334, 340, 286, 327, 239, 283, 244, 1144]`; ink px ref/ours 2268/2473 (ratio 1.0904); SSIM blocks <0.9: 265/30294; [overlay](images/04-bold-emph/pdflatex-lm-main-export-p1-overlay.png) (24083 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-main-export-p1-heatmap.png) (25158 B, ÷1)
  - registration (diagnostic): ink-centroid shift [4.0, 0.0] pt; after undoing it: mean|Δ| 0.4473, differing 0.003373, SSIM₈ 0.993 (vs unregistered 0.4325, 0.003339, 0.9931) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `and` dx 17.95 dy 0.73; `bold` dx 16.54 dy 0.73; `emphasised` dx 15.62 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933519, 519, 462, 366, 293, 287, 321, 265, 298, 295, 232, 216, 218, 229, 229, 1067]`; ink px ref/ours 2702/2465 (ratio 0.9123); SSIM blocks <0.9: 227/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [1.0, 0.0] pt; after undoing it: mean|Δ| 0.4614, differing 0.003361, SSIM₈ 0.9936 (vs unregistered 0.3848, 0.003146, 0.9944) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `emphasised` dx 5.26 dy 0.46; `bold` dx 5.16 dy 0.46; `and` dx 5.12 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933192, 532, 436, 403, 312, 330, 336, 228, 274, 278, 218, 286, 228, 261, 230, 1272]`; ink px ref/ours 2702/2473 (ratio 0.9152); SSIM blocks <0.9: 252/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [22.0, 0.0] pt; after undoing it: mean|Δ| 0.4902, differing 0.00352, SSIM₈ 0.993 (vs unregistered 0.4208, 0.003319, 0.9939) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx 37.49 dy 0.46; `one` dx 36.39 dy 0.46; `on` dx 35.97 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933144, 455, 411, 335, 384, 345, 332, 339, 340, 281, 249, 264, 266, 261, 226, 1184]`; ink px ref/ours 2260/2465 (ratio 1.0907); SSIM blocks <0.9: 255/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-16.0, 0.0] pt; after undoing it: mean|Δ| 0.4315, differing 0.003346, SSIM₈ 0.9933 (vs unregistered 0.4245, 0.003312, 0.9933) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `line.` dx -27.75 dy 0.73; `one` dx -26.62 dy 0.73; `on` dx -25.36 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933041, 452, 428, 367, 355, 330, 354, 356, 346, 314, 285, 327, 237, 257, 240, 1127]`; ink px ref/ours 2260/2473 (ratio 1.0942); SSIM blocks <0.9: 263/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [5.0, 0.0] pt; after undoing it: mean|Δ| 0.4195, differing 0.003258, SSIM₈ 0.9933 (vs unregistered 0.4288, 0.003357, 0.9932) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `and` dx 17.95 dy 0.73; `bold` dx 16.55 dy 0.73; `emphasised` dx 15.62 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 05-unicode — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934471, 531, 434, 351, 255, 281, 235, 186, 150, 219, 214, 199, 193, 149, 197, 751]`; ink px ref/ours 2129/2123 (ratio 0.9972); SSIM blocks <0.9: 193/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [5.0, 0.0] pt; after undoing it: mean|Δ| 0.3994, differing 0.003069, SSIM₈ 0.9941 (vs unregistered 0.2971, 0.002695, 0.9955) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `dash.` dx 30.41 dy 0.46; `also` dx -5.46 dy 0.46; `dash;` dx -4.07 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933840, 456, 420, 338, 287, 265, 246, 211, 205, 278, 259, 228, 217, 232, 273, 1061]`; ink px ref/ours 2129/2087 (ratio 0.9803); SSIM blocks <0.9: 229/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [23.0, 0.0] pt; after undoing it: mean|Δ| 0.4366, differing 0.003257, SSIM₈ 0.9932 (vs unregistered 0.3747, 0.002996, 0.9941) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `dash.` dx 72.93 dy 0.46; `also` dx 6.37 dy 0.46; `café` dx 4.48 dy 0.46
- word-sequence differences: replace ref ['Resume', '—'] ours ['Resume—']; replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na"', 'ive', 'caf', "'", 'e', "R'", 'esum', "'"]

### 05-unicode — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933784, 466, 389, 318, 316, 350, 300, 311, 265, 275, 232, 237, 245, 200, 242, 886]`; ink px ref/ours 1547/2123 (ratio 1.3723); SSIM blocks <0.9: 237/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-4.0, 0.0] pt; after undoing it: mean|Δ| 0.3604, differing 0.002889, SSIM₈ 0.9939 (vs unregistered 0.3619, 0.002925, 0.9938) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `dash.` dx 16.19 dy -0.3; `also` dx -13.35 dy -0.3; `dash;` dx -9.55 dy -0.3
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933606, 493, 373, 301, 281, 354, 305, 342, 239, 292, 257, 202, 257, 227, 246, 1041]`; ink px ref/ours 1547/2087 (ratio 1.3491); SSIM blocks <0.9: 249/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [14.0, 0.0] pt; after undoing it: mean|Δ| 0.3978, differing 0.003115, SSIM₈ 0.9931 (vs unregistered 0.3858, 0.003009, 0.9932) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `dash.` dx 58.71 dy -0.3; `dash;` dx -2.93 dy -0.3; `café` dx 2.79 dy -0.3
- word-sequence differences: replace ref ['Resume', '—'] ours ['Resume—']; replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na"', 'ive', 'caf', "'", 'e', "R'", 'esum', "'"]

### 05-unicode — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934287, 570, 367, 345, 277, 258, 239, 232, 233, 241, 245, 185, 188, 173, 231, 745]`; ink px ref/ours 2119/2123 (ratio 1.0019); SSIM blocks <0.9: 209/30294; [overlay](images/05-unicode/pdflatex-de1020c-export-p1-overlay.png) (22053 B, ÷1), [heatmap](images/05-unicode/pdflatex-de1020c-export-p1-heatmap.png) (22231 B, ÷1)
  - registration (diagnostic): ink-centroid shift [3.5, 0.0] pt; after undoing it: mean|Δ| 0.3893, differing 0.00301, SSIM₈ 0.9944 (vs unregistered 0.3127, 0.002744, 0.9955) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `dash.` dx 30.73 dy 0.46; `also` dx -5.31 dy 0.46; `dash;` dx -3.91 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933836, 574, 375, 303, 247, 303, 260, 232, 226, 227, 222, 236, 190, 225, 257, 1103]`; ink px ref/ours 2119/2087 (ratio 0.9849); SSIM blocks <0.9: 225/30294; [overlay](images/05-unicode/pdflatex-main-export-p1-overlay.png) (22329 B, ÷1), [heatmap](images/05-unicode/pdflatex-main-export-p1-heatmap.png) (22987 B, ÷1)
  - registration (diagnostic): ink-centroid shift [21.5, 0.0] pt; after undoing it: mean|Δ| 0.4223, differing 0.003194, SSIM₈ 0.9937 (vs unregistered 0.3712, 0.002979, 0.9945) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `dash.` dx 73.25 dy 0.46; `also` dx 6.52 dy 0.46; `café` dx 4.66 dy 0.46
- word-sequence differences: replace ref ['Resume', '—'] ours ['Resume—']; replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na"', 'ive', 'caf', "'", 'e', "R'", 'esum', "'"]

### 05-unicode — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933782, 470, 394, 304, 327, 351, 291, 322, 255, 269, 237, 246, 249, 203, 219, 897]`; ink px ref/ours 1537/2123 (ratio 1.3813); SSIM blocks <0.9: 237/30294; [overlay](images/05-unicode/pdflatex-lm-de1020c-export-p1-overlay.png) (21943 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-de1020c-export-p1-heatmap.png) (23176 B, ÷1)
  - registration (diagnostic): ink-centroid shift [-3.0, 0.0] pt; after undoing it: mean|Δ| 0.3554, differing 0.002884, SSIM₈ 0.994 (vs unregistered 0.3617, 0.002921, 0.9938) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `dash.` dx 16.16 dy 0.73; `also` dx -13.37 dy 0.73; `dash;` dx -9.57 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933608, 485, 386, 290, 276, 378, 284, 359, 232, 276, 254, 229, 243, 250, 218, 1048]`; ink px ref/ours 1537/2087 (ratio 1.3578); SSIM blocks <0.9: 249/30294; [overlay](images/05-unicode/pdflatex-lm-main-export-p1-overlay.png) (21959 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-main-export-p1-heatmap.png) (23437 B, ÷1)
  - registration (diagnostic): ink-centroid shift [15.0, 0.0] pt; after undoing it: mean|Δ| 0.3968, differing 0.003089, SSIM₈ 0.9932 (vs unregistered 0.3853, 0.003004, 0.9932) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `dash.` dx 58.68 dy 0.73; `dash;` dx -2.95 dy 0.73; `café` dx 2.8 dy 0.73
- word-sequence differences: replace ref ['Resume', '—'] ours ['Resume—']; replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na"', 'ive', 'caf', "'", 'e', "R'", 'esum', "'"]

### 05-unicode — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934491, 509, 440, 374, 248, 251, 251, 167, 176, 216, 207, 211, 173, 149, 193, 760]`; ink px ref/ours 2139/2123 (ratio 0.9925); SSIM blocks <0.9: 190/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [3.5, 0.0] pt; after undoing it: mean|Δ| 0.3966, differing 0.003046, SSIM₈ 0.994 (vs unregistered 0.2963, 0.002685, 0.9955) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `dash.` dx 30.37 dy 0.46; `also` dx -5.49 dy 0.46; `dash;` dx -4.09 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933837, 454, 385, 372, 269, 271, 266, 200, 235, 239, 269, 252, 196, 218, 276, 1077]`; ink px ref/ours 2139/2087 (ratio 0.9757); SSIM blocks <0.9: 230/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [21.5, 0.0] pt; after undoing it: mean|Δ| 0.4224, differing 0.003185, SSIM₈ 0.9934 (vs unregistered 0.3752, 0.002981, 0.9941) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `dash.` dx 72.89 dy 0.46; `also` dx 6.34 dy 0.46; `café` dx 4.48 dy 0.46
- word-sequence differences: replace ref ['Resume', '—'] ours ['Resume—']; replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na"', 'ive', 'caf', "'", 'e', "R'", 'esum', "'"]

### 05-unicode — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933783, 466, 389, 315, 321, 350, 297, 313, 265, 274, 231, 236, 249, 198, 242, 887]`; ink px ref/ours 1542/2123 (ratio 1.3768); SSIM blocks <0.9: 237/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-4.0, 0.0] pt; after undoing it: mean|Δ| 0.3605, differing 0.002888, SSIM₈ 0.9939 (vs unregistered 0.362, 0.002927, 0.9938) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `dash.` dx 16.19 dy 0.73; `also` dx -13.35 dy 0.73; `dash;` dx -9.55 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933605, 494, 373, 301, 279, 358, 303, 339, 243, 290, 257, 202, 259, 226, 245, 1042]`; ink px ref/ours 1542/2087 (ratio 1.3534); SSIM blocks <0.9: 249/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [14.0, 0.0] pt; after undoing it: mean|Δ| 0.3977, differing 0.003113, SSIM₈ 0.9931 (vs unregistered 0.3858, 0.003009, 0.9932) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `dash.` dx 58.71 dy 0.73; `dash;` dx -2.93 dy 0.73; `café` dx 2.79 dy 0.73
- word-sequence differences: replace ref ['Resume', '—'] ours ['Resume—']; replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na"', 'ive', 'caf', "'", 'e', "R'", 'esum', "'"]

### 06-math-inline — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933844, 435, 352, 317, 197, 257, 297, 287, 248, 200, 246, 306, 299, 245, 218, 1068]`; ink px ref/ours 1729/1833 (ratio 1.0602); SSIM blocks <0.9: 216/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.0, 3.5] pt; after undoing it: mean|Δ| 0.3289, differing 0.002618, SSIM₈ 0.9948 (vs unregistered 0.3832, 0.00292, 0.9934) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `inside` dx -8.16 dy 1.32; `a` dx -8.04 dy 1.32; `sentence` dx -8.01 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (10): error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: math mode is not implemented in this version; error: math mode is not implemented in this version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935166, 437, 365, 287, 214, 209, 224, 221, 194, 148, 135, 137, 227, 156, 161, 535]`; ink px ref/ours 1729/1692 (ratio 0.9786); SSIM blocks <0.9: 173/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [6.0, 0.0] pt; after undoing it: mean|Δ| 0.2924, differing 0.002443, SSIM₈ 0.995 (vs unregistered 0.2455, 0.002263, 0.9958) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `,` dx 22.47 dy -2.62; `text.` dx 18.76 dy -2.62; `b` dx 18.36 dy -2.62
- word-sequence differences: delete ref ['α'] ours []; replace ref ['β,'] ours [',']; replace ref ['√x'] ours ['x']

### 06-math-inline — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933869, 425, 339, 269, 252, 319, 324, 366, 281, 210, 256, 307, 229, 233, 196, 941]`; ink px ref/ours 1358/1833 (ratio 1.3498); SSIM blocks <0.9: 226/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-8.5, 3.5] pt; after undoing it: mean|Δ| 0.3235, differing 0.002608, SSIM₈ 0.9945 (vs unregistered 0.3682, 0.002867, 0.9931) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `text.` dx -25.34 dy 1.32; `of` dx -25.01 dy 1.32; `sentence` dx -22.78 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (10): error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: math mode is not implemented in this version; error: math mode is not implemented in this version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934991, 334, 334, 280, 257, 282, 265, 269, 213, 207, 187, 168, 172, 201, 150, 506]`; ink px ref/ours 1358/1692 (ratio 1.2459); SSIM blocks <0.9: 194/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [3.0, 0.0] pt; after undoing it: mean|Δ| 0.2859, differing 0.002355, SSIM₈ 0.9949 (vs unregistered 0.2606, 0.002249, 0.9953) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `,` dx 16.6 dy -2.62; `b` dx 12.5 dy -2.62; `inside` dx -11.38 dy -2.62
- word-sequence differences: delete ref ['α'] ours []; replace ref ['β,'] ours [',']; replace ref ['√x'] ours ['x']

### 06-math-inline — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933877, 403, 304, 272, 266, 302, 299, 308, 222, 194, 272, 315, 237, 246, 231, 1068]`; ink px ref/ours 1699/1833 (ratio 1.0789); SSIM blocks <0.9: 211/30294; [overlay](images/06-math-inline/pdflatex-de1020c-export-p1-overlay.png) (20351 B, ÷1), [heatmap](images/06-math-inline/pdflatex-de1020c-export-p1-heatmap.png) (22295 B, ÷1)
  - registration (diagnostic): ink-centroid shift [-4.5, 3.5] pt; after undoing it: mean|Δ| 0.339, differing 0.002633, SSIM₈ 0.9949 (vs unregistered 0.3821, 0.002886, 0.9936) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `a` dx -7.78 dy 1.32; `sentence` dx -7.76 dy 1.32; `of` dx -7.59 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']; replace ref ['√xinside'] ours ['√x', 'inside']

### 06-math-inline — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (10): error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: math mode is not implemented in this version; error: math mode is not implemented in this version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934966, 408, 304, 225, 247, 266, 228, 202, 210, 202, 166, 138, 230, 222, 187, 615]`; ink px ref/ours 1699/1692 (ratio 0.9959); SSIM blocks <0.9: 177/30294; [overlay](images/06-math-inline/pdflatex-main-export-p1-overlay.png) (19162 B, ÷1), [heatmap](images/06-math-inline/pdflatex-main-export-p1-heatmap.png) (19806 B, ÷1)
  - registration (diagnostic): ink-centroid shift [6.5, 0.0] pt; after undoing it: mean|Δ| 0.2924, differing 0.002405, SSIM₈ 0.9951 (vs unregistered 0.2741, 0.002322, 0.9955) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `,` dx 22.73 dy -2.62; `text.` dx 19.01 dy -2.62; `b` dx 18.63 dy -2.62
- word-sequence differences: replace ref ['α+', 'β,'] ours ['+', ',']; replace ref ['√xinside'] ours ['x', 'inside']

### 06-math-inline — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933850, 444, 339, 259, 271, 318, 327, 365, 277, 225, 258, 298, 218, 220, 185, 962]`; ink px ref/ours 1340/1833 (ratio 1.3679); SSIM blocks <0.9: 225/30294; [overlay](images/06-math-inline/pdflatex-lm-de1020c-export-p1-overlay.png) (20720 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-de1020c-export-p1-heatmap.png) (22915 B, ÷1)
  - registration (diagnostic): ink-centroid shift [-7.5, 3.5] pt; after undoing it: mean|Δ| 0.3147, differing 0.002573, SSIM₈ 0.9946 (vs unregistered 0.368, 0.002877, 0.9931) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `text.` dx -25.38 dy 1.32; `of` dx -25.05 dy 1.32; `sentence` dx -22.81 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']

### 06-math-inline — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (10): error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: math mode is not implemented in this version; error: math mode is not implemented in this version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934977, 348, 325, 262, 277, 285, 279, 251, 235, 205, 171, 171, 182, 184, 154, 510]`; ink px ref/ours 1340/1692 (ratio 1.2627); SSIM blocks <0.9: 194/30294; [overlay](images/06-math-inline/pdflatex-lm-main-export-p1-overlay.png) (19264 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-main-export-p1-heatmap.png) (19844 B, ÷1)
  - registration (diagnostic): ink-centroid shift [4.0, 0.0] pt; after undoing it: mean|Δ| 0.2979, differing 0.002433, SSIM₈ 0.9947 (vs unregistered 0.2609, 0.002247, 0.9953) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `,` dx 16.59 dy -2.62; `b` dx 12.49 dy -2.62; `inside` dx -11.4 dy -2.62
- word-sequence differences: replace ref ['α+', 'β,'] ours ['+', ',']; replace ref ['√x'] ours ['x']

### 06-math-inline — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933853, 435, 339, 322, 197, 256, 302, 277, 256, 193, 249, 311, 284, 255, 208, 1079]`; ink px ref/ours 1739/1833 (ratio 1.0541); SSIM blocks <0.9: 216/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-4.5, 3.5] pt; after undoing it: mean|Δ| 0.3486, differing 0.00267, SSIM₈ 0.9946 (vs unregistered 0.3831, 0.002915, 0.9934) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `inside` dx -8.16 dy 1.32; `a` dx -8.04 dy 1.32; `sentence` dx -8.02 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (10): error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: math mode is not implemented in this version; error: math mode is not implemented in this version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935179, 436, 363, 286, 208, 215, 221, 214, 190, 151, 135, 138, 220, 158, 161, 541]`; ink px ref/ours 1739/1692 (ratio 0.973); SSIM blocks <0.9: 172/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [7.0, 0.0] pt; after undoing it: mean|Δ| 0.3325, differing 0.002526, SSIM₈ 0.9944 (vs unregistered 0.2451, 0.00225, 0.9958) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `,` dx 22.47 dy -2.62; `text.` dx 18.74 dy -2.62; `b` dx 18.36 dy -2.62
- word-sequence differences: delete ref ['α'] ours []; replace ref ['β,'] ours [',']; replace ref ['√x'] ours ['x']

### 06-math-inline — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933869, 423, 342, 268, 250, 323, 325, 362, 281, 211, 255, 310, 230, 231, 194, 942]`; ink px ref/ours 1359/1833 (ratio 1.3488); SSIM blocks <0.9: 226/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-9.0, 3.5] pt; after undoing it: mean|Δ| 0.3255, differing 0.002633, SSIM₈ 0.9944 (vs unregistered 0.3682, 0.002868, 0.9931) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `text.` dx -25.34 dy 1.32; `of` dx -25.01 dy 1.32; `sentence` dx -22.78 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (10): error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: math mode is not implemented in this version; error: math mode is not implemented in this version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934988, 337, 334, 279, 259, 284, 264, 268, 213, 205, 190, 166, 174, 200, 148, 507]`; ink px ref/ours 1359/1692 (ratio 1.245); SSIM blocks <0.9: 195/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [2.5, 0.0] pt; after undoing it: mean|Δ| 0.2919, differing 0.002388, SSIM₈ 0.9948 (vs unregistered 0.2606, 0.002249, 0.9953) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `,` dx 16.6 dy -2.62; `b` dx 12.5 dy -2.62; `inside` dx -11.38 dy -2.62
- word-sequence differences: delete ref ['α'] ours []; replace ref ['β,'] ours [',']; replace ref ['√x'] ours ['x']

### 07-math-display — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934500, 594, 464, 552, 281, 307, 196, 220, 289, 154, 134, 152, 181, 132, 125, 535]`; ink px ref/ours 1954/1796 (ratio 0.9191); SSIM blocks <0.9: 191/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-8.5, 1.0] pt; after undoing it: mean|Δ| 0.4015, differing 0.003094, SSIM₈ 0.9922 (vs unregistered 0.2595, 0.002628, 0.995) — the remainder is rendering/layout error, not offset
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \sum is not supported by this compiler version; error: \frac is not supported by this compiler version
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933162, 516, 504, 380, 287, 292, 264, 344, 305, 194, 212, 252, 273, 282, 243, 1306]`; ink px ref/ours 1954/1992 (ratio 1.0194); SSIM blocks <0.9: 322/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [77.0, -22.0] pt; after undoing it: mean|Δ| 0.5136, differing 0.003637, SSIM₈ 0.989 (vs unregistered 0.4269, 0.003302, 0.9904) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `display.` dx 256.59 dy -49.17; `the` dx 252.83 dy -49.17; `After` dx 247.68 dy -49.17
- word-sequence differences: insert ref [] ours ['[', '_', 'i=1', '^', 'n', 'i', '=', 'n(n+1)']; delete ref ['n', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 07-math-display — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933827, 410, 355, 528, 348, 373, 280, 264, 264, 218, 237, 183, 297, 180, 203, 849]`; ink px ref/ours 1608/1796 (ratio 1.1169); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-24.0, 1.5] pt; after undoing it: mean|Δ| 0.4062, differing 0.00315, SSIM₈ 0.9917 (vs unregistered 0.3495, 0.002895, 0.9931) — the remainder is rendering/layout error, not offset
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.73 dy 1.09; `2` dx -3.54 dy 2.25; `display.` dx -3.41 dy -0.3
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \sum is not supported by this compiler version; error: \frac is not supported by this compiler version
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933267, 424, 438, 379, 356, 355, 342, 321, 267, 223, 273, 234, 336, 279, 238, 1084]`; ink px ref/ours 1608/1992 (ratio 1.2388); SSIM blocks <0.9: 335/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [61.5, -22.0] pt; after undoing it: mean|Δ| 0.4779, differing 0.003536, SSIM₈ 0.9887 (vs unregistered 0.4121, 0.00321, 0.9903) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `display.` dx 251.69 dy -50.03; `the` dx 250.48 dy -50.03; `After` dx 247.68 dy -50.03
- word-sequence differences: insert ref [] ours ['[', '_', 'i=1', '^', 'n', 'i', '=', 'n(n+1)']; delete ref ['n', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 07-math-display — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934567, 583, 430, 473, 321, 297, 196, 195, 302, 176, 128, 141, 187, 135, 135, 550]`; ink px ref/ours 1978/1796 (ratio 0.908); SSIM blocks <0.9: 193/30294; [overlay](images/07-math-display/pdflatex-de1020c-export-p1-overlay.png) (20903 B, ÷1), [heatmap](images/07-math-display/pdflatex-de1020c-export-p1-heatmap.png) (21635 B, ÷1)
  - registration (diagnostic): ink-centroid shift [-7.5, 1.0] pt; after undoing it: mean|Δ| 0.4225, differing 0.003149, SSIM₈ 0.992 (vs unregistered 0.2609, 0.00259, 0.9949) — the remainder is rendering/layout error, not offset
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.1; `display.` dx 0.17 dy 2.13; `the` dx 0.11 dy 2.13
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \sum is not supported by this compiler version; error: \frac is not supported by this compiler version
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933310, 520, 453, 277, 349, 281, 230, 270, 305, 224, 226, 261, 300, 293, 256, 1261]`; ink px ref/ours 1978/1992 (ratio 1.0071); SSIM blocks <0.9: 321/30294; [overlay](images/07-math-display/pdflatex-main-export-p1-overlay.png) (21521 B, ÷1), [heatmap](images/07-math-display/pdflatex-main-export-p1-heatmap.png) (24038 B, ÷1)
  - registration (diagnostic): ink-centroid shift [78.0, -22.5] pt; after undoing it: mean|Δ| 0.5083, differing 0.003576, SSIM₈ 0.989 (vs unregistered 0.4223, 0.003236, 0.9906) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `display.` dx 256.59 dy -48.99; `the` dx 252.83 dy -48.99; `After` dx 247.68 dy -48.99
- word-sequence differences: insert ref [] ours ['[', '_', 'i=1', '^', 'n', 'i', '=', 'n(n+1)']; delete ref ['n', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 07-math-display — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933745, 450, 367, 487, 356, 402, 264, 258, 285, 261, 208, 305, 184, 175, 192, 877]`; ink px ref/ours 1630/1796 (ratio 1.1018); SSIM blocks <0.9: 235/30294; [overlay](images/07-math-display/pdflatex-lm-de1020c-export-p1-overlay.png) (21299 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-de1020c-export-p1-heatmap.png) (23242 B, ÷1)
  - registration (diagnostic): ink-centroid shift [-21.0, 1.0] pt; after undoing it: mean|Δ| 0.4015, differing 0.003159, SSIM₈ 0.9917 (vs unregistered 0.3528, 0.00293, 0.9931) — the remainder is rendering/layout error, not offset
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.75 dy 2.53; `2` dx -3.54 dy 2.36; `display.` dx -3.42 dy 0.73
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \sum is not supported by this compiler version; error: \frac is not supported by this compiler version
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933207, 477, 453, 334, 378, 356, 313, 339, 273, 284, 243, 345, 214, 293, 221, 1086]`; ink px ref/ours 1630/1992 (ratio 1.2221); SSIM blocks <0.9: 336/30294; [overlay](images/07-math-display/pdflatex-lm-main-export-p1-overlay.png) (21506 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-main-export-p1-heatmap.png) (24336 B, ÷1)
  - registration (diagnostic): ink-centroid shift [64.5, -22.5] pt; after undoing it: mean|Δ| 0.4754, differing 0.003524, SSIM₈ 0.9886 (vs unregistered 0.4118, 0.003231, 0.9903) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `display.` dx 251.67 dy -48.59; `the` dx 250.48 dy -48.59; `After` dx 247.68 dy -48.59
- word-sequence differences: insert ref [] ours ['[', '_', 'i=1', '^', 'n', 'i', '=', 'n(n+1)']; delete ref ['n', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 07-math-display — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934499, 617, 467, 539, 276, 309, 204, 216, 283, 145, 142, 139, 179, 138, 120, 543]`; ink px ref/ours 1971/1796 (ratio 0.9112); SSIM blocks <0.9: 191/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-8.5, 1.0] pt; after undoing it: mean|Δ| 0.402, differing 0.003098, SSIM₈ 0.9922 (vs unregistered 0.2589, 0.00263, 0.995) — the remainder is rendering/layout error, not offset
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \sum is not supported by this compiler version; error: \frac is not supported by this compiler version
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933173, 509, 515, 368, 292, 296, 277, 305, 325, 194, 215, 242, 276, 282, 249, 1298]`; ink px ref/ours 1971/1992 (ratio 1.0107); SSIM blocks <0.9: 323/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [77.0, -22.5] pt; after undoing it: mean|Δ| 0.5126, differing 0.00364, SSIM₈ 0.9889 (vs unregistered 0.4264, 0.003306, 0.9905) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `display.` dx 256.59 dy -49.17; `the` dx 252.83 dy -49.17; `After` dx 247.68 dy -49.17
- word-sequence differences: insert ref [] ours ['[', '_', 'i=1', '^', 'n', 'i', '=', 'n(n+1)']; delete ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 07-math-display — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933826, 409, 357, 531, 338, 382, 277, 256, 275, 218, 233, 187, 294, 181, 204, 848]`; ink px ref/ours 1616/1796 (ratio 1.1114); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-24.5, 1.0] pt; after undoing it: mean|Δ| 0.407, differing 0.003158, SSIM₈ 0.9915 (vs unregistered 0.3495, 0.002894, 0.9931) — the remainder is rendering/layout error, not offset
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.73 dy 2.12; `2` dx -3.54 dy 2.25; `display.` dx -3.41 dy 0.73
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \sum is not supported by this compiler version; error: \frac is not supported by this compiler version
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933267, 421, 441, 382, 351, 358, 340, 313, 277, 224, 276, 230, 333, 281, 239, 1083]`; ink px ref/ours 1616/1992 (ratio 1.2327); SSIM blocks <0.9: 335/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [61.5, -22.0] pt; after undoing it: mean|Δ| 0.478, differing 0.003536, SSIM₈ 0.9887 (vs unregistered 0.4122, 0.00321, 0.9903) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `display.` dx 251.69 dy -49.0; `the` dx 250.48 dy -49.0; `After` dx 247.68 dy -49.0
- word-sequence differences: insert ref [] ours ['[', '_', 'i=1', '^', 'n', 'i', '=', 'n(n+1)']; delete ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 08-two-page — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1535837, 30452, 26147, 24533, 22450, 21872, 22500, 20593, 20315, 21336, 19731, 19135, 19424, 19064, 19858, 95569]`; ink px ref/ours 152381/134585 (ratio 0.8832); SSIM blocks <0.9: 16227/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.5, -4.0] pt; after undoing it: mean|Δ| 33.4783, differing 0.23959, SSIM₈ 0.4547 (vs unregistered 31.604, 0.230212, 0.4867) — the remainder is rendering/layout error, not offset
- page 2: |Δ| histogram (16 bins, pixel counts) `[1518144, 30651, 25967, 24422, 22157, 21752, 22680, 20693, 20985, 22451, 20965, 20617, 21112, 20181, 20664, 105375]`; ink px ref/ours 151462/131743 (ratio 0.8698); SSIM blocks <0.9: 16822/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.0, 9.0] pt; after undoing it: mean|Δ| 32.1035, differing 0.231913, SSIM₈ 0.4753 (vs unregistered 33.6579, 0.240085, 0.4507) — the remainder is rendering/layout error, not offset
- page 3: |Δ| histogram (16 bins, pixel counts) `[1774169, 11550, 9778, 9157, 8337, 8439, 8946, 8134, 7667, 8840, 8115, 7824, 8036, 7751, 8314, 43759]`; ink px ref/ours 33623/71008 (ratio 2.1119); SSIM blocks <0.9: 7433/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-6.0, 112.5] pt; after undoing it: mean|Δ| 12.2712, differing 0.087041, SSIM₈ 0.7797 (vs unregistered 13.3908, 0.093775, 0.7598) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `branch` dx -435.47 dy 214.06; `branch` dx -435.47 dy 167.32; `over` dx -407.04 dy 214.02

### 08-two-page — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1548695, 29233, 25670, 23719, 21651, 20365, 21497, 19722, 19775, 20729, 18734, 18131, 19542, 18754, 19003, 93596]`; ink px ref/ours 152381/122345 (ratio 0.8029); SSIM blocks <0.9: 16041/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.5, -6.5] pt; after undoing it: mean|Δ| 32.7654, differing 0.234172, SSIM₈ 0.455 (vs unregistered 30.7036, 0.222896, 0.4936) — the remainder is rendering/layout error, not offset
- page 2: |Δ| histogram (16 bins, pixel counts) `[1544809, 28759, 25246, 23507, 21305, 19998, 21397, 19557, 19795, 20870, 19229, 18870, 20047, 19467, 19074, 96886]`; ink px ref/ours 151462/120329 (ratio 0.7945); SSIM blocks <0.9: 16085/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-6.0, -2.0] pt; after undoing it: mean|Δ| 31.7432, differing 0.227433, SSIM₈ 0.4744 (vs unregistered 31.3281, 0.224947, 0.4815) — the remainder is rendering/layout error, not offset
- page 3: |Δ| histogram (16 bins, pixel counts) `[1752033, 13457, 12137, 11259, 10234, 9446, 10594, 9391, 9094, 10181, 8736, 8552, 9481, 9085, 8988, 46148]`; ink px ref/ours 33623/95315 (ratio 2.8348); SSIM blocks <0.9: 8915/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-6.0, 178.5] pt; after undoing it: mean|Δ| 11.9956, differing 0.087189, SSIM₈ 0.7866 (vs unregistered 14.8284, 0.106728, 0.7265) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `every` dx -421.18 dy 357.83; `leaf` dx -416.72 dy 357.83; `oak` dx -415.78 dy 343.66

### 08-two-page — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571209, 29050, 24879, 23374, 22427, 21235, 22990, 22122, 21052, 20710, 18292, 17708, 17105, 16387, 17019, 73257]`; ink px ref/ours 104967/134585 (ratio 1.2822); SSIM blocks <0.9: 15666/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-4.5, -3.5] pt; after undoing it: mean|Δ| 28.6287, differing 0.217024, SSIM₈ 0.4762 (vs unregistered 27.547, 0.210718, 0.5025) — the remainder is rendering/layout error, not offset
- page 2: |Δ| histogram (16 bins, pixel counts) `[1555826, 29617, 25174, 23833, 22680, 21623, 23309, 22906, 21714, 21966, 18997, 18388, 18461, 17121, 17761, 79440]`; ink px ref/ours 104819/131743 (ratio 1.2569); SSIM blocks <0.9: 16449/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.0, 9.0] pt; after undoing it: mean|Δ| 27.9971, differing 0.213645, SSIM₈ 0.4826 (vs unregistered 29.058, 0.219579, 0.4664) — the remainder is rendering/layout error, not offset
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746654, 14263, 12363, 11679, 11023, 10858, 11514, 11288, 10599, 11007, 9764, 9045, 9193, 8553, 9140, 41873]`; ink px ref/ours 46386/71008 (ratio 1.5308); SSIM blocks <0.9: 8607/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.0, 40.5] pt; after undoing it: mean|Δ| 13.9886, differing 0.105474, SSIM₈ 0.7462 (vs unregistered 14.7852, 0.11002, 0.721) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `oak` dx 426.94 dy 54.44; `oak` dx 426.94 dy 36.59; `oak` dx 426.94 dy 31.09

### 08-two-page — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1586607, 27876, 24188, 22614, 21507, 19876, 21903, 21310, 20562, 19953, 16953, 16799, 16985, 16514, 15786, 69383]`; ink px ref/ours 104967/122345 (ratio 1.1656); SSIM blocks <0.9: 15532/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-4.5, -5.5] pt; after undoing it: mean|Δ| 27.778, differing 0.210288, SSIM₈ 0.4785 (vs unregistered 26.3472, 0.202265, 0.5107) — the remainder is rendering/layout error, not offset
- page 2: |Δ| histogram (16 bins, pixel counts) `[1581680, 27794, 24301, 22939, 21610, 19915, 22180, 21802, 20510, 20338, 17085, 16909, 17674, 16626, 16275, 71178]`; ink px ref/ours 104819/120329 (ratio 1.148); SSIM blocks <0.9: 15695/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-6.0, -2.0] pt; after undoing it: mean|Δ| 27.118, differing 0.206801, SSIM₈ 0.4918 (vs unregistered 26.8328, 0.204959, 0.4977) — the remainder is rendering/layout error, not offset
- page 3: |Δ| histogram (16 bins, pixel counts) `[1717407, 16519, 14461, 13899, 12660, 11939, 13440, 12560, 12134, 12245, 10567, 10225, 11211, 10416, 10186, 48947]`; ink px ref/ours 46386/95315 (ratio 2.0548); SSIM blocks <0.9: 10239/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.0, 107.0] pt; after undoing it: mean|Δ| 16.2603, differing 0.121307, SSIM₈ 0.6886 (vs unregistered 17.0708, 0.126818, 0.6785) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `fox` dx -419.35 dy 212.84; `brown` dx -417.19 dy 212.84; `fox` dx -419.35 dy 146.29

### 08-two-page — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p1-overlay.png) (34905 B, ÷8), [heatmap](images/08-two-page/pdflatex-de1020c-export-p1-heatmap.png) (29165 B, ÷8)
  - registration (diagnostic): ink-centroid shift [-6.0, -5.0] pt; after undoing it: mean|Δ| 33.7951, differing 0.240957, SSIM₈ 0.4471 (vs unregistered 31.5764, 0.22951, 0.4881) — the remainder is rendering/layout error, not offset
- page 2: |Δ| histogram (16 bins, pixel counts) `[1518465, 30739, 25693, 24091, 22137, 22533, 22535, 20944, 20824, 22593, 20919, 20497, 20504, 19480, 20990, 105872]`; ink px ref/ours 150460/131743 (ratio 0.8756); SSIM blocks <0.9: 16800/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p2-overlay.png) (33626 B, ÷8), [heatmap](images/08-two-page/pdflatex-de1020c-export-p2-heatmap.png) (29884 B, ÷8)
  - registration (diagnostic): ink-centroid shift [-5.5, 8.0] pt; after undoing it: mean|Δ| 31.4181, differing 0.227828, SSIM₈ 0.4907 (vs unregistered 33.6289, 0.239479, 0.4516) — the remainder is rendering/layout error, not offset
- page 3: |Δ| histogram (16 bins, pixel counts) `[1774335, 11537, 9689, 9104, 8316, 8407, 8934, 8213, 7615, 9012, 8045, 7866, 7928, 7590, 8264, 43961]`; ink px ref/ours 33504/71008 (ratio 2.1194); SSIM blocks <0.9: 7408/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p3-overlay.png) (41443 B, ÷4), [heatmap](images/08-two-page/pdflatex-de1020c-export-p3-heatmap.png) (51354 B, ÷4)
  - registration (diagnostic): ink-centroid shift [-6.5, 111.5] pt; after undoing it: mean|Δ| 12.1564, differing 0.086095, SSIM₈ 0.7785 (vs unregistered 13.387, 0.093579, 0.7605) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `branch` dx -435.47 dy 214.06; `branch` dx -435.47 dy 167.32; `over` dx -407.04 dy 214.02

### 08-two-page — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1549572, 29117, 24955, 23023, 21429, 21097, 20957, 19777, 19815, 21019, 18877, 18345, 19028, 18538, 18698, 94569]`; ink px ref/ours 151753/122345 (ratio 0.8062); SSIM blocks <0.9: 15975/30294; [overlay](images/08-two-page/pdflatex-main-export-p1-overlay.png) (35247 B, ÷8), [heatmap](images/08-two-page/pdflatex-main-export-p1-heatmap.png) (29739 B, ÷8)
  - registration (diagnostic): ink-centroid shift [-6.0, -7.5] pt; after undoing it: mean|Δ| 32.6025, differing 0.232039, SSIM₈ 0.4611 (vs unregistered 30.7323, 0.222064, 0.4953) — the remainder is rendering/layout error, not offset
- page 2: |Δ| histogram (16 bins, pixel counts) `[1545165, 28799, 24888, 23299, 21335, 20712, 21197, 19722, 19780, 20973, 18979, 18828, 19731, 18910, 19216, 97282]`; ink px ref/ours 150460/120329 (ratio 0.7997); SSIM blocks <0.9: 16074/30294; [overlay](images/08-two-page/pdflatex-main-export-p2-overlay.png) (34677 B, ÷8), [heatmap](images/08-two-page/pdflatex-main-export-p2-heatmap.png) (30033 B, ÷8)
  - registration (diagnostic): ink-centroid shift [-6.5, -3.0] pt; after undoing it: mean|Δ| 31.8196, differing 0.226933, SSIM₈ 0.4763 (vs unregistered 31.2992, 0.224358, 0.4827) — the remainder is rendering/layout error, not offset
- page 3: |Δ| histogram (16 bins, pixel counts) `[1752289, 13298, 11951, 11251, 10131, 9703, 10597, 9350, 9171, 10218, 8691, 8641, 9427, 8933, 8973, 46192]`; ink px ref/ours 33504/95315 (ratio 2.8449); SSIM blocks <0.9: 8916/30294; [overlay](images/08-two-page/pdflatex-main-export-p3-overlay.png) (51981 B, ÷4), [heatmap](images/08-two-page/pdflatex-main-export-p3-heatmap.png) (59446 B, ÷4)
  - registration (diagnostic): ink-centroid shift [-6.5, 177.5] pt; after undoing it: mean|Δ| 12.5484, differing 0.089955, SSIM₈ 0.7718 (vs unregistered 14.8225, 0.106579, 0.7268) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `oak` dx -415.72 dy 343.66; `branch` dx -414.11 dy 343.66; `lazy` dx -397.58 dy 357.79

### 08-two-page — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p1-overlay.png) (33418 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p1-heatmap.png) (27829 B, ÷8)
  - registration (diagnostic): ink-centroid shift [-4.5, -3.5] pt; after undoing it: mean|Δ| 28.6302, differing 0.217042, SSIM₈ 0.4762 (vs unregistered 27.5467, 0.210674, 0.5026) — the remainder is rendering/layout error, not offset
- page 2: |Δ| histogram (16 bins, pixel counts) `[1556053, 29239, 25104, 24114, 22751, 21524, 23492, 22483, 22053, 21728, 19173, 18342, 18558, 16712, 18303, 79187]`; ink px ref/ours 105011/131743 (ratio 1.2546); SSIM blocks <0.9: 16449/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p2-overlay.png) (32573 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p2-heatmap.png) (28829 B, ÷8)
  - registration (diagnostic): ink-centroid shift [-5.0, 9.0] pt; after undoing it: mean|Δ| 27.9958, differing 0.213645, SSIM₈ 0.4826 (vs unregistered 29.0579, 0.219561, 0.4664) — the remainder is rendering/layout error, not offset
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746783, 14069, 12340, 11757, 11133, 10758, 11588, 11172, 10690, 10911, 9823, 9052, 9221, 8387, 9339, 41793]`; ink px ref/ours 46441/71008 (ratio 1.529); SSIM blocks <0.9: 8608/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p3-overlay.png) (53233 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p3-heatmap.png) (58710 B, ÷4)
  - registration (diagnostic): ink-centroid shift [-5.5, 40.5] pt; after undoing it: mean|Δ| 14.0353, differing 0.105604, SSIM₈ 0.7456 (vs unregistered 14.7842, 0.109995, 0.721) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `oak` dx 426.94 dy 55.47; `oak` dx 426.94 dy 37.61; `oak` dx 426.94 dy 32.12

### 08-two-page — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1586780, 27515, 24191, 22689, 21825, 19651, 22190, 20996, 20788, 19735, 17105, 16686, 17063, 16116, 16337, 69149]`; ink px ref/ours 105111/122345 (ratio 1.164); SSIM blocks <0.9: 15535/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p1-overlay.png) (33858 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-main-export-p1-heatmap.png) (28528 B, ÷8)
  - registration (diagnostic): ink-centroid shift [-4.5, -5.5] pt; after undoing it: mean|Δ| 27.776, differing 0.210241, SSIM₈ 0.4785 (vs unregistered 26.346, 0.202254, 0.5107) — the remainder is rendering/layout error, not offset
- page 2: |Δ| histogram (16 bins, pixel counts) `[1581857, 27538, 24208, 23147, 21722, 19772, 22373, 21455, 20741, 20203, 17195, 16893, 17757, 16235, 16730, 70990]`; ink px ref/ours 105011/120329 (ratio 1.1459); SSIM blocks <0.9: 15697/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p2-overlay.png) (33453 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-main-export-p2-heatmap.png) (28779 B, ÷8)
  - registration (diagnostic): ink-centroid shift [-6.0, -2.0] pt; after undoing it: mean|Δ| 27.1177, differing 0.206792, SSIM₈ 0.4918 (vs unregistered 26.8324, 0.204917, 0.4977) — the remainder is rendering/layout error, not offset
- page 3: |Δ| histogram (16 bins, pixel counts) `[1717502, 16398, 14422, 13936, 12780, 11836, 13530, 12501, 12184, 12138, 10646, 10188, 11221, 10281, 10436, 48817]`; ink px ref/ours 46441/95315 (ratio 2.0524); SSIM blocks <0.9: 10239/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p3-overlay.png) (62992 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-main-export-p3-heatmap.png) (68853 B, ÷4)
  - registration (diagnostic): ink-centroid shift [-5.5, 107.0] pt; after undoing it: mean|Δ| 16.2256, differing 0.121197, SSIM₈ 0.6894 (vs unregistered 17.0697, 0.126805, 0.6785) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `fox` dx -419.35 dy 213.87; `brown` dx -417.2 dy 213.87; `fox` dx -419.35 dy 147.32

### 08-two-page — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1535378, 29989, 25555, 24965, 22556, 22246, 22654, 20669, 20453, 21240, 20149, 19662, 19349, 18997, 19406, 95548]`; ink px ref/ours 152232/134585 (ratio 0.8841); SSIM blocks <0.9: 16215/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.5, -4.0] pt; after undoing it: mean|Δ| 33.3923, differing 0.239438, SSIM₈ 0.4562 (vs unregistered 31.6507, 0.230423, 0.4864) — the remainder is rendering/layout error, not offset
- page 2: |Δ| histogram (16 bins, pixel counts) `[1517468, 30567, 25348, 24959, 22402, 22356, 22688, 20537, 21047, 22558, 21491, 21046, 20679, 20011, 20481, 105178]`; ink px ref/ours 151458/131743 (ratio 0.8698); SSIM blocks <0.9: 16818/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.0, 9.0] pt; after undoing it: mean|Δ| 32.0767, differing 0.231809, SSIM₈ 0.4757 (vs unregistered 33.675, 0.240149, 0.4508) — the remainder is rendering/layout error, not offset
- page 3: |Δ| histogram (16 bins, pixel counts) `[1773969, 11449, 9657, 9335, 8367, 8575, 8971, 8031, 7696, 8844, 8288, 7974, 7985, 7726, 8258, 43691]`; ink px ref/ours 33659/71008 (ratio 2.1096); SSIM blocks <0.9: 7435/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-6.0, 112.0] pt; after undoing it: mean|Δ| 12.2627, differing 0.086613, SSIM₈ 0.7776 (vs unregistered 13.4019, 0.093836, 0.7597) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `branch` dx -435.52 dy 214.06; `branch` dx -435.52 dy 167.32; `over` dx -406.99 dy 214.02

### 08-two-page — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1548670, 28704, 25063, 24306, 21684, 20836, 21583, 19837, 19686, 20593, 19140, 18733, 19418, 18899, 18424, 93240]`; ink px ref/ours 152232/122345 (ratio 0.8037); SSIM blocks <0.9: 16034/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.5, -6.5] pt; after undoing it: mean|Δ| 32.7758, differing 0.234309, SSIM₈ 0.4555 (vs unregistered 30.7012, 0.222865, 0.4942) — the remainder is rendering/layout error, not offset
- page 2: |Δ| histogram (16 bins, pixel counts) `[1544263, 28723, 24614, 24103, 21402, 20775, 21409, 19315, 19962, 20913, 19563, 19355, 19624, 19271, 18951, 96573]`; ink px ref/ours 151458/120329 (ratio 0.7945); SSIM blocks <0.9: 16075/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-6.0, -2.0] pt; after undoing it: mean|Δ| 31.7438, differing 0.227442, SSIM₈ 0.475 (vs unregistered 31.3266, 0.224994, 0.482) — the remainder is rendering/layout error, not offset
- page 3: |Δ| histogram (16 bins, pixel counts) `[1751921, 13398, 11996, 11341, 10221, 9619, 10654, 9394, 9145, 10215, 8807, 8660, 9499, 9009, 8863, 46074]`; ink px ref/ours 33659/95315 (ratio 2.8318); SSIM blocks <0.9: 8919/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-6.0, 178.5] pt; after undoing it: mean|Δ| 12.0152, differing 0.087277, SSIM₈ 0.7863 (vs unregistered 14.83, 0.106817, 0.7266) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `every` dx -421.22 dy 357.83; `leaf` dx -416.78 dy 357.83; `oak` dx -415.83 dy 343.66

### 08-two-page — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571173, 29128, 24898, 23367, 22271, 21337, 23030, 22042, 21136, 20589, 18380, 17673, 17118, 16399, 17001, 73274]`; ink px ref/ours 105020/134585 (ratio 1.2815); SSIM blocks <0.9: 15670/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-4.5, -3.0] pt; after undoing it: mean|Δ| 28.4943, differing 0.21647, SSIM₈ 0.477 (vs unregistered 27.5472, 0.210665, 0.5025) — the remainder is rendering/layout error, not offset
- page 2: |Δ| histogram (16 bins, pixel counts) `[1555802, 29658, 25241, 23771, 22602, 21613, 23447, 22794, 21821, 21830, 19124, 18335, 18417, 17162, 17754, 79445]`; ink px ref/ours 104857/131743 (ratio 1.2564); SSIM blocks <0.9: 16449/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.0, 9.0] pt; after undoing it: mean|Δ| 27.9975, differing 0.213589, SSIM₈ 0.4826 (vs unregistered 29.0582, 0.219508, 0.4664) — the remainder is rendering/layout error, not offset
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746646, 14271, 12388, 11672, 10984, 10870, 11549, 11242, 10632, 10956, 9797, 9063, 9172, 8550, 9128, 41896]`; ink px ref/ours 46395/71008 (ratio 1.5305); SSIM blocks <0.9: 8610/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.0, 40.5] pt; after undoing it: mean|Δ| 13.9886, differing 0.105461, SSIM₈ 0.7462 (vs unregistered 14.7852, 0.109992, 0.7209) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `oak` dx 426.94 dy 55.47; `oak` dx 426.94 dy 37.61; `oak` dx 426.94 dy 32.12

### 08-two-page — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1586586, 27931, 24224, 22584, 21413, 19925, 21986, 21178, 20662, 19824, 17065, 16741, 17008, 16507, 15773, 69409]`; ink px ref/ours 105020/122345 (ratio 1.165); SSIM blocks <0.9: 15537/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-4.5, -5.5] pt; after undoing it: mean|Δ| 27.7782, differing 0.210233, SSIM₈ 0.4784 (vs unregistered 26.3469, 0.202213, 0.5107) — the remainder is rendering/layout error, not offset
- page 2: |Δ| histogram (16 bins, pixel counts) `[1581667, 27831, 24378, 22856, 21550, 19904, 22300, 21699, 20582, 20237, 17219, 16836, 17665, 16628, 16273, 71191]`; ink px ref/ours 104857/120329 (ratio 1.1476); SSIM blocks <0.9: 15697/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-6.0, -2.0] pt; after undoing it: mean|Δ| 27.1183, differing 0.206752, SSIM₈ 0.4918 (vs unregistered 26.8334, 0.204923, 0.4976) — the remainder is rendering/layout error, not offset
- page 3: |Δ| histogram (16 bins, pixel counts) `[1717405, 16510, 14483, 13905, 12636, 11945, 13456, 12525, 12179, 12174, 10620, 10234, 11182, 10411, 10185, 48966]`; ink px ref/ours 46395/95315 (ratio 2.0544); SSIM blocks <0.9: 10240/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [-5.0, 107.0] pt; after undoing it: mean|Δ| 16.2605, differing 0.121297, SSIM₈ 0.6886 (vs unregistered 17.0705, 0.126792, 0.6785) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `fox` dx -419.35 dy 213.87; `brown` dx -417.19 dy 213.87; `fox` dx -419.35 dy 147.32

### 09-mixed-document — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908424, 2029, 1775, 1940, 1563, 1506, 1540, 1401, 1384, 1449, 1711, 1473, 1402, 1450, 1477, 8292]`; ink px ref/ours 9691/9252 (ratio 0.9547); SSIM blocks <0.9: 1549/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [6.5, 32.0] pt; after undoing it: mean|Δ| 2.4817, differing 0.017338, SSIM₈ 0.9542 (vs unregistered 2.4844, 0.017426, 0.9504) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `twelve` dx -2.87 dy 39.06; `in` dx -2.66 dy 39.06; `set` dx -2.46 dy 39.06
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (5): error: math mode is not implemented in this version; error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: \frac is not supported by this compiler version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1910179, 2083, 1606, 1572, 1552, 1435, 1521, 1288, 1507, 1472, 1470, 1503, 1448, 1372, 1434, 7374]`; ink px ref/ours 9691/9829 (ratio 1.0142); SSIM blocks <0.9: 1417/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [14.5, 17.5] pt; after undoing it: mean|Δ| 2.232, differing 0.015682, SSIM₈ 0.962 (vs unregistered 2.3211, 0.016422, 0.9553) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `set` dx -407.05 dy 29.34; `in` dx -402.22 dy 29.34; `twelve` dx -399.41 dy 29.34
- word-sequence differences: replace ref ['A', 'paragraph'] ours ['Aparagraph']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['UTF-8', 'word'] ours ['UTF-8word']

### 09-mixed-document — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909444, 2180, 1960, 2265, 1611, 1596, 1560, 1754, 1443, 1429, 1644, 1400, 1251, 1263, 1288, 6728]`; ink px ref/ours 7447/9252 (ratio 1.2424); SSIM blocks <0.9: 1580/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [0.5, 35.0] pt; after undoing it: mean|Δ| 2.1216, differing 0.016018, SSIM₈ 0.9585 (vs unregistered 2.2538, 0.016899, 0.9494) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `twelve` dx 433.26 dy 24.19; `paper.` dx -52.18 dy 38.59; `letter` dx -48.63 dy 38.59
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (5): error: math mode is not implemented in this version; error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: \frac is not supported by this compiler version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1910897, 2169, 1761, 1857, 1586, 1481, 1559, 1629, 1587, 1412, 1367, 1461, 1340, 1261, 1287, 6162]`; ink px ref/ours 7447/9829 (ratio 1.3199); SSIM blocks <0.9: 1428/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [9.0, 20.0] pt; after undoing it: mean|Δ| 2.0887, differing 0.015488, SSIM₈ 0.9618 (vs unregistered 2.1482, 0.016033, 0.9546) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `set` dx -439.52 dy 28.92; `in` dx -436.88 dy 28.92; `.` dx -422.95 dy 34.6
- word-sequence differences: replace ref ['A', 'paragraph'] ours ['Aparagraph']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['UTF-8', 'word'] ours ['UTF-8word']

### 09-mixed-document — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908448, 1975, 1818, 1995, 1553, 1434, 1436, 1374, 1435, 1503, 1793, 1550, 1349, 1400, 1470, 8283]`; ink px ref/ours 9857/9252 (ratio 0.9386); SSIM blocks <0.9: 1551/30294; [overlay](images/09-mixed-document/pdflatex-de1020c-export-p1-overlay.png) (65093 B, ÷1), [heatmap](images/09-mixed-document/pdflatex-de1020c-export-p1-heatmap.png) (81088 B, ÷1)
  - registration (diagnostic): ink-centroid shift [7.5, 32.5] pt; after undoing it: mean|Δ| 2.4821, differing 0.01733, SSIM₈ 0.9561 (vs unregistered 2.4853, 0.017384, 0.9504) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `twelve` dx -3.05 dy 39.17; `in` dx -2.78 dy 39.17; `set` dx -2.52 dy 39.17
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (5): error: math mode is not implemented in this version; error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: \frac is not supported by this compiler version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1910227, 2061, 1671, 1613, 1534, 1352, 1406, 1277, 1543, 1501, 1490, 1564, 1428, 1377, 1452, 7320]`; ink px ref/ours 9857/9829 (ratio 0.9972); SSIM blocks <0.9: 1415/30294; [overlay](images/09-mixed-document/pdflatex-main-export-p1-overlay.png) (66505 B, ÷1), [heatmap](images/09-mixed-document/pdflatex-main-export-p1-heatmap.png) (79098 B, ÷1)
  - registration (diagnostic): ink-centroid shift [16.0, 18.0] pt; after undoing it: mean|Δ| 2.1971, differing 0.01558, SSIM₈ 0.9623 (vs unregistered 2.3188, 0.016364, 0.9553) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `set` dx -407.11 dy 29.45; `in` dx -402.34 dy 29.45; `twelve` dx -399.59 dy 29.45
- word-sequence differences: replace ref ['A', 'paragraph'] ours ['Aparagraph']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['UTF-8', 'word'] ours ['UTF-8word']

### 09-mixed-document — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909812, 2060, 1849, 1927, 1609, 1602, 1682, 1704, 1540, 1378, 1642, 1427, 1316, 1278, 1363, 6627]`; ink px ref/ours 7573/9252 (ratio 1.2217); SSIM blocks <0.9: 1632/30294; [overlay](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-overlay.png) (65163 B, ÷1), [heatmap](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-heatmap.png) (81460 B, ÷1)
  - registration (diagnostic): ink-centroid shift [-1.5, 34.5] pt; after undoing it: mean|Δ| 2.1931, differing 0.016303, SSIM₈ 0.9564 (vs unregistered 2.2522, 0.016745, 0.948) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `twelve` dx 433.26 dy 25.83; `paper.` dx -52.2 dy 40.23; `letter` dx -48.65 dy 40.23
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (5): error: math mode is not implemented in this version; error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: \frac is not supported by this compiler version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1910952, 2103, 1673, 1544, 1618, 1496, 1644, 1624, 1664, 1412, 1404, 1447, 1424, 1282, 1355, 6174]`; ink px ref/ours 7573/9829 (ratio 1.2979); SSIM blocks <0.9: 1470/30294; [overlay](images/09-mixed-document/pdflatex-lm-main-export-p1-overlay.png) (66138 B, ÷1), [heatmap](images/09-mixed-document/pdflatex-lm-main-export-p1-heatmap.png) (79410 B, ÷1)
  - registration (diagnostic): ink-centroid shift [7.0, 20.0] pt; after undoing it: mean|Δ| 2.0531, differing 0.01518, SSIM₈ 0.9619 (vs unregistered 2.1698, 0.016043, 0.9532) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `set` dx -439.52 dy 30.56; `in` dx -436.88 dy 30.56; `.` dx -423.01 dy 35.64
- word-sequence differences: replace ref ['A', 'paragraph'] ours ['Aparagraph']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['UTF-8', 'word'] ours ['UTF-8word']

### 09-mixed-document — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908485, 1993, 1726, 1891, 1568, 1563, 1547, 1422, 1387, 1527, 1642, 1498, 1332, 1389, 1540, 8306]`; ink px ref/ours 9680/9252 (ratio 0.9558); SSIM blocks <0.9: 1553/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [7.5, 32.0] pt; after undoing it: mean|Δ| 2.4783, differing 0.017337, SSIM₈ 0.9544 (vs unregistered 2.4838, 0.017418, 0.9503) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `twelve` dx -2.91 dy 39.06; `in` dx -2.65 dy 39.06; `set` dx -2.4 dy 39.06
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (5): error: math mode is not implemented in this version; error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: \frac is not supported by this compiler version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1910224, 2061, 1576, 1544, 1584, 1450, 1537, 1311, 1504, 1525, 1402, 1485, 1387, 1338, 1496, 7392]`; ink px ref/ours 9680/9829 (ratio 1.0154); SSIM blocks <0.9: 1418/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [16.0, 17.5] pt; after undoing it: mean|Δ| 2.1863, differing 0.015553, SSIM₈ 0.9625 (vs unregistered 2.3191, 0.016417, 0.9553) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `set` dx -406.99 dy 29.34; `in` dx -402.21 dy 29.34; `twelve` dx -399.45 dy 29.34
- word-sequence differences: replace ref ['A', 'paragraph'] ours ['Aparagraph']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['UTF-8', 'word'] ours ['UTF-8word']

### 09-mixed-document — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909464, 2171, 1943, 2272, 1611, 1596, 1548, 1766, 1439, 1468, 1597, 1418, 1229, 1257, 1310, 6727]`; ink px ref/ours 7446/9252 (ratio 1.2425); SSIM blocks <0.9: 1581/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [1.0, 35.0] pt; after undoing it: mean|Δ| 2.1261, differing 0.016042, SSIM₈ 0.9583 (vs unregistered 2.2539, 0.016899, 0.9494) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `twelve` dx 433.26 dy 25.22; `paper.` dx -52.18 dy 39.62; `letter` dx -48.63 dy 39.62
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (5): error: math mode is not implemented in this version; error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: \frac is not supported by this compiler version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1910914, 2163, 1747, 1859, 1588, 1483, 1545, 1642, 1582, 1445, 1325, 1472, 1331, 1249, 1310, 6161]`; ink px ref/ours 7446/9829 (ratio 1.32); SSIM blocks <0.9: 1429/30294; images not emitted for this engine
  - registration (diagnostic): ink-centroid shift [9.0, 20.0] pt; after undoing it: mean|Δ| 2.0889, differing 0.015486, SSIM₈ 0.9618 (vs unregistered 2.1483, 0.016033, 0.9546) — the remainder is rendering/layout error, not offset
- largest word displacements (pt): `set` dx -439.52 dy 29.95; `in` dx -436.88 dy 29.95; `.` dx -422.95 dy 35.63
- word-sequence differences: replace ref ['A', 'paragraph'] ours ['Aparagraph']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['UTF-8', 'word'] ours ['UTF-8word']

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

## Diagnostic regression check vs previous evidence (never acceptance)

- previous evidence: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a6f1b07cc82949ff0/tests/visual-corpus/evidence/20260912T053804Z`; comparable entries: 108
- worse: none

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
