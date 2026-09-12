# FlashTeX visual corpus: reference-render and raster-diff evidence

Generated 20260912T053804Z on mac-m1max-a by `tests/visual-corpus/harness/run.sh`.

**Scope statement.** These are narrow-case measurements over a small declared corpus. They never claim general pixel perfection, LaTeX compatibility, or parity outside these fixtures, these engines, this font, this page size, this DPI and these builds. The reference engines are test oracles only; FlashTeX never invokes them and remains an original Rust implementation.

## Provenance

- suite_branch: `agent/mac-visual-oracle/reference-raster`
- suite_sha: `984fa28f2537feadb9f848eb90f69de0327d1fa5`
- input_main_sha: `1dd26c5e0dd5e04a39f0b8e55c90abebf635c863`
- machine: `mac-m1max-a`
- os: `macOS 26.3.1 arm64`
- swift: `Apple Swift version 6.2.4 (swiftlang-6.2.4.1.4 clang-1700.6.4.2)`
- cargo: `cargo 1.99.0-nightly (3efb1f477 2026-07-17)`
- python: `3.12.0`
- pillow: `12.2.0`
- DPI: 144 (every raster: CoreGraphics bitmap, sRGB IEC61966-2.1, 8-bit RGBA, white opaque background, MediaBox mapped to width_pt*144/72 px; text antialiased, font smoothing off, subpixel positioning on)
- Overlay/heatmap PNGs emitted for engines: pdflatex (metrics are computed for every engine)
- Pixel threshold for `above_threshold_fraction`: |Δluma| ≥ 32/255; SSIM: 8×8 blocks, K1=0.01, K2=0.03
- Arithmetic backend: Pillow 12.2.0 (accelerator; identical integer results to the stdlib path)

### Reference engines (oracle only)

| Engine | Available | Version | Body font | Preamble |
|---|---|---|---|---|
| pdflatex | yes | pdfTeX 3.141592653-2.6-1.40.29 (TeX Live 2026) | URW Nimbus Roman (`times` package, T1 fontenc) | `\documentclass[12pt]{article} \usepackage[T1]{fontenc} \usepackage{times} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| xelatex | yes | XeTeX 3.141592653-2.6-0.999998 (TeX Live 2026) | Times New Roman (`fontspec`, /System/Library/Fonts/Supplemental/Times New Roman.ttf) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{Times New Roman} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| lualatex | yes | This is LuaHBTeX, Version 1.24.0 (TeX Live 2026) | Times New Roman (`fontspec`, /System/Library/Fonts/Supplemental/Times New Roman.ttf) | `\documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{Times New Roman} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |

Engine flags: `-interaction=batchmode -halt-on-error -file-line-error`. Page size: US letter 612×792 pt for every producer (checked per page from the MediaBox). LaTeX package versions: see `provenance.json` → `packages`.

### FlashTeX builds under test

- compiler `main`: `origin/main` @ `1dd26c5e0dd5e04a39f0b8e55c90abebf635c863` — coordination: activate Astra authority and authorized direct-commit fallback
- compiler `de1020c`: `de1020c` @ `de1020cd0be7cede11be2691e00e7f5b15cb2224` — compiler: add math, real font metrics and PDF output
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

Full SHAs and `.meta.json` contents are in `provenance.json`. Only the body after `\begin{document}` is shared by every producer; the harness substitutes the engine preamble and strips it for FlashTeX.

## Export comparison (flashtex-pdf PDF vs reference PDF, both rasterized identically)

This is the PDF-output comparison. Word boxes come from PDFKit on both PDFs; rules are ink rows ≥10pt long.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| | max | differing | ≥thr | SSIM₈ | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-plain-paragraph | lualatex | de1020c | 1/1 | ok | 0.1575 | 255 | 0.0023 | 0.0014 | 0.9980 | 13/13/13 | yes | 0.29 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.4024 | 255 | 0.0032 | 0.0025 | 0.9937 | 13/13/13 | yes | 26.11 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.1873 | 255 | 0.0024 | 0.0014 | 0.9975 | 13/13/13 | yes | 0.43 | 0.46 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.3986 | 255 | 0.0032 | 0.0025 | 0.9939 | 13/13/13 | yes | 26.62 | 0.46 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-main-export-p1-overlay.png) |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.1607 | 255 | 0.0023 | 0.0014 | 0.9980 | 13/13/13 | yes | 0.30 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.4030 | 255 | 0.0032 | 0.0025 | 0.9937 | 13/13/13 | yes | 26.09 | 0.46 | 1.0000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 8.3842 | 255 | 0.0613 | 0.0509 | 0.8656 | 210/210/210 | yes | 116.20 | 5.12 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 8.4755 | 255 | 0.0621 | 0.0516 | 0.8581 | 210/210/210 | yes | 146.62 | 16.30 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 8.3726 | 255 | 0.0611 | 0.0509 | 0.8663 | 210/210/210 | yes | 116.24 | 5.12 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 8.5059 | 255 | 0.0620 | 0.0516 | 0.8582 | 210/210/210 | yes | 146.59 | 16.30 | 0.8952 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-main-export-p1-overlay.png) |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 8.3710 | 255 | 0.0613 | 0.0509 | 0.8656 | 210/210/210 | yes | 119.39 | 5.19 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 8.4942 | 255 | 0.0621 | 0.0518 | 0.8578 | 210/210/210 | yes | 144.25 | 16.36 | 0.8952 | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.3394 | 255 | 0.0085 | 0.0073 | 0.9793 | 15/15/15 | yes | 0.25 | 21.02 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.2663 | 255 | 0.0081 | 0.0070 | 0.9802 | 15/13/11 | no | 5.51 | 22.60 | 1.0000 | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.3496 | 255 | 0.0085 | 0.0072 | 0.9793 | 15/15/15 | yes | 0.33 | 20.87 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-de1020c-export-p1-overlay.png) |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.2822 | 255 | 0.0082 | 0.0070 | 0.9801 | 15/13/11 | no | 5.62 | 22.46 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-main-export-p1-overlay.png) |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.3370 | 255 | 0.0085 | 0.0073 | 0.9793 | 15/15/15 | yes | 0.24 | 21.02 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.2665 | 255 | 0.0081 | 0.0071 | 0.9802 | 15/13/11 | no | 5.50 | 22.60 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.3808 | 255 | 0.0031 | 0.0024 | 0.9945 | 10/12/8 | no | 3.77 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.4195 | 255 | 0.0033 | 0.0026 | 0.9939 | 10/12/8 | no | 29.18 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.3836 | 255 | 0.0031 | 0.0024 | 0.9944 | 10/12/8 | no | 4.04 | 0.46 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-de1020c-export-p1-overlay.png) |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.4290 | 255 | 0.0034 | 0.0026 | 0.9937 | 10/12/8 | no | 29.44 | 0.46 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-main-export-p1-overlay.png) |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.3848 | 255 | 0.0031 | 0.0025 | 0.9944 | 10/12/8 | no | 3.81 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.4208 | 255 | 0.0033 | 0.0026 | 0.9939 | 10/12/8 | no | 29.21 | 0.46 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.2971 | 255 | 0.0027 | 0.0020 | 0.9955 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.3747 | 255 | 0.0030 | 0.0023 | 0.9941 | 11/16/5 | no | 17.27 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.3127 | 255 | 0.0027 | 0.0020 | 0.9955 | 11/19/7 | no | 5.88 | 0.46 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-de1020c-export-p1-overlay.png) |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.3712 | 255 | 0.0030 | 0.0023 | 0.9945 | 11/16/5 | no | 17.43 | 0.46 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-main-export-p1-overlay.png) |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.2963 | 255 | 0.0027 | 0.0020 | 0.9955 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.3752 | 255 | 0.0030 | 0.0023 | 0.9941 | 11/16/5 | no | 17.25 | 0.46 | 1.0000 | 2/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.3832 | 255 | 0.0029 | 0.0023 | 0.9934 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | recovered | 0.2455 | 255 | 0.0023 | 0.0017 | 0.9958 | 15/14/12 | no | 11.36 | 2.08 | 0.9167 | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.3821 | 255 | 0.0029 | 0.0023 | 0.9936 | 13/14/10 | no | 3.89 | 2.69 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-de1020c-export-p1-overlay.png) |
| 06-math-inline | pdflatex | main | 1/1 | recovered | 0.2741 | 255 | 0.0023 | 0.0018 | 0.9955 | 13/14/10 | no | 12.25 | 1.97 | 0.9000 | 0/0 | [p1](images/06-math-inline/pdflatex-main-export-p1-overlay.png) |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.3831 | 255 | 0.0029 | 0.0023 | 0.9934 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | recovered | 0.2451 | 255 | 0.0022 | 0.0017 | 0.9958 | 15/14/12 | no | 11.36 | 2.08 | 0.9167 | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.2595 | 255 | 0.0026 | 0.0019 | 0.9950 | 13/10/7 | no | 0.60 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | recovered | 0.4269 | 255 | 0.0033 | 0.0027 | 0.9904 | 13/16/6 | no | 128.09 | 24.82 | 0.8333 | 3/0 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.2609 | 255 | 0.0026 | 0.0019 | 0.9949 | 13/10/7 | no | 0.59 | 1.41 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-de1020c-export-p1-overlay.png) |
| 07-math-display | pdflatex | main | 1/1 | recovered | 0.4223 | 255 | 0.0032 | 0.0026 | 0.9906 | 13/16/6 | no | 128.09 | 24.72 | 0.8333 | 3/0 | [p1](images/07-math-display/pdflatex-main-export-p1-overlay.png) |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.2589 | 255 | 0.0026 | 0.0019 | 0.9950 | 14/10/7 | no | 0.59 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | recovered | 0.4264 | 255 | 0.0033 | 0.0026 | 0.9905 | 14/16/6 | no | 128.09 | 24.82 | 0.8333 | 3/0 | - |
| 08-two-page | lualatex | de1020c | 3/3 | ok | 26.2176 | 255 | 0.1880 | 0.1574 | 0.5657 | 1800/1800/1800 | yes | 164.00 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | lualatex | main | 3/3 | ok | 25.6200 | 255 | 0.1849 | 0.1546 | 0.5672 | 1800/1800/1800 | yes | 156.63 | 152.37 | 0.8900 | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/3 | ok | 26.1974 | 255 | 0.1875 | 0.1572 | 0.5667 | 1800/1800/1800 | yes | 143.13 | 92.17 | 0.8952 | 0/0 | [p1](images/08-two-page/pdflatex-de1020c-export-p1-overlay.png) |
| 08-two-page | pdflatex | main | 3/3 | ok | 25.6180 | 255 | 0.1843 | 0.1544 | 0.5683 | 1800/1800/1800 | yes | 152.18 | 151.60 | 0.9008 | 0/0 | [p1](images/08-two-page/pdflatex-main-export-p1-overlay.png) |
| 08-two-page | xelatex | de1020c | 3/3 | ok | 26.2425 | 255 | 0.1881 | 0.1578 | 0.5656 | 1800/1800/1800 | yes | 164.01 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | xelatex | main | 3/3 | ok | 25.6193 | 255 | 0.1849 | 0.1549 | 0.5676 | 1800/1800/1800 | yes | 156.64 | 152.37 | 0.8900 | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.4844 | 255 | 0.0174 | 0.0146 | 0.9504 | 54/54/45 | no | 2.41 | 31.76 | 0.9778 | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | recovered | 2.3211 | 255 | 0.0164 | 0.0137 | 0.9553 | 54/54/39 | no | 104.96 | 18.76 | 0.8974 | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.4853 | 255 | 0.0174 | 0.0146 | 0.9504 | 54/54/45 | no | 2.48 | 31.80 | 0.9778 | 0/0 | [p1](images/09-mixed-document/pdflatex-de1020c-export-p1-overlay.png) |
| 09-mixed-document | pdflatex | main | 1/1 | recovered | 2.3188 | 255 | 0.0164 | 0.0137 | 0.9553 | 54/54/39 | no | 105.07 | 18.80 | 0.8974 | 0/0 | [p1](images/09-mixed-document/pdflatex-main-export-p1-overlay.png) |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.4838 | 255 | 0.0174 | 0.0146 | 0.9503 | 54/54/45 | no | 2.45 | 31.76 | 0.9778 | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | recovered | 2.3191 | 255 | 0.0164 | 0.0137 | 0.9553 | 54/54/39 | no | 105.07 | 18.76 | 0.8974 | 0/0 | - |

## Preview-equivalent comparison (CoreText draw of compile_result vs reference PDF raster)

Weaker than a capture of the real preview: it re-implements the app's draw code path rather than exercising the SwiftUI Canvas. Word-box metrics are not available for this side (no PDF), so they are omitted.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| | max | differing | ≥thr | SSIM₈ | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-plain-paragraph | lualatex | de1020c | 1/1 | ok | 0.1686 | 255 | 0.0023 | 0.0014 | 0.9978 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.4049 | 255 | 0.0032 | 0.0025 | 0.9937 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.1741 | 255 | 0.0023 | 0.0014 | 0.9977 | -/-/- | - | - | - | - | 0/0 | [p1](images/01-plain-paragraph/pdflatex-de1020c-preview-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.3958 | 255 | 0.0032 | 0.0025 | 0.9940 | -/-/- | - | - | - | - | 0/0 | [p1](images/01-plain-paragraph/pdflatex-main-preview-p1-overlay.png) |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.1716 | 255 | 0.0023 | 0.0014 | 0.9978 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.4056 | 255 | 0.0032 | 0.0025 | 0.9937 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 8.3831 | 255 | 0.0613 | 0.0509 | 0.8656 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 8.4759 | 255 | 0.0621 | 0.0516 | 0.8582 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 8.3720 | 255 | 0.0611 | 0.0509 | 0.8664 | -/-/- | - | - | - | - | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-de1020c-preview-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 8.5059 | 255 | 0.0620 | 0.0516 | 0.8583 | -/-/- | - | - | - | - | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-main-preview-p1-overlay.png) |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 8.3703 | 255 | 0.0613 | 0.0509 | 0.8657 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 8.4944 | 255 | 0.0621 | 0.0518 | 0.8579 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.3394 | 255 | 0.0085 | 0.0073 | 0.9793 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.2662 | 255 | 0.0081 | 0.0070 | 0.9802 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.3496 | 255 | 0.0085 | 0.0072 | 0.9793 | -/-/- | - | - | - | - | 0/0 | [p1](images/03-section-heading/pdflatex-de1020c-preview-p1-overlay.png) |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.2820 | 255 | 0.0082 | 0.0070 | 0.9801 | -/-/- | - | - | - | - | 0/0 | [p1](images/03-section-heading/pdflatex-main-preview-p1-overlay.png) |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.3370 | 255 | 0.0085 | 0.0073 | 0.9793 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.2663 | 255 | 0.0081 | 0.0071 | 0.9802 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.3808 | 255 | 0.0031 | 0.0024 | 0.9945 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.4195 | 255 | 0.0033 | 0.0026 | 0.9939 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.3837 | 255 | 0.0031 | 0.0024 | 0.9944 | -/-/- | - | - | - | - | 0/0 | [p1](images/04-bold-emph/pdflatex-de1020c-preview-p1-overlay.png) |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.4290 | 255 | 0.0034 | 0.0026 | 0.9937 | -/-/- | - | - | - | - | 0/0 | [p1](images/04-bold-emph/pdflatex-main-preview-p1-overlay.png) |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.3849 | 255 | 0.0032 | 0.0025 | 0.9944 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.4209 | 255 | 0.0033 | 0.0026 | 0.9939 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.2971 | 255 | 0.0027 | 0.0020 | 0.9955 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.3747 | 255 | 0.0030 | 0.0023 | 0.9941 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.3125 | 255 | 0.0027 | 0.0020 | 0.9955 | -/-/- | - | - | - | - | 2/0 | [p1](images/05-unicode/pdflatex-de1020c-preview-p1-overlay.png) |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.3712 | 255 | 0.0030 | 0.0023 | 0.9945 | -/-/- | - | - | - | - | 2/0 | [p1](images/05-unicode/pdflatex-main-preview-p1-overlay.png) |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.2963 | 255 | 0.0027 | 0.0020 | 0.9955 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.3752 | 255 | 0.0030 | 0.0023 | 0.9941 | -/-/- | - | - | - | - | 2/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.3823 | 255 | 0.0029 | 0.0023 | 0.9934 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | recovered | 0.2455 | 255 | 0.0023 | 0.0017 | 0.9958 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.3812 | 255 | 0.0029 | 0.0023 | 0.9936 | -/-/- | - | - | - | - | 0/0 | [p1](images/06-math-inline/pdflatex-de1020c-preview-p1-overlay.png) |
| 06-math-inline | pdflatex | main | 1/1 | recovered | 0.2741 | 255 | 0.0023 | 0.0018 | 0.9955 | -/-/- | - | - | - | - | 0/0 | [p1](images/06-math-inline/pdflatex-main-preview-p1-overlay.png) |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.3823 | 255 | 0.0029 | 0.0023 | 0.9934 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | recovered | 0.2451 | 255 | 0.0023 | 0.0016 | 0.9958 | -/-/- | - | - | - | - | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.2575 | 255 | 0.0026 | 0.0019 | 0.9950 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | recovered | 0.4269 | 255 | 0.0033 | 0.0027 | 0.9904 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.2593 | 255 | 0.0026 | 0.0019 | 0.9950 | -/-/- | - | - | - | - | 3/1 | [p1](images/07-math-display/pdflatex-de1020c-preview-p1-overlay.png) |
| 07-math-display | pdflatex | main | 1/1 | recovered | 0.4222 | 255 | 0.0032 | 0.0026 | 0.9906 | -/-/- | - | - | - | - | 3/0 | [p1](images/07-math-display/pdflatex-main-preview-p1-overlay.png) |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.2573 | 255 | 0.0026 | 0.0019 | 0.9950 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | recovered | 0.4264 | 255 | 0.0033 | 0.0026 | 0.9905 | -/-/- | - | - | - | - | 3/0 | - |
| 08-two-page | lualatex | de1020c | 3/3 | ok | 26.2189 | 255 | 0.1880 | 0.1574 | 0.5659 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex | main | 3/3 | ok | 25.6206 | 255 | 0.1848 | 0.1547 | 0.5674 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/3 | ok | 26.1972 | 255 | 0.1875 | 0.1571 | 0.5669 | -/-/- | - | - | - | - | 0/0 | [p1](images/08-two-page/pdflatex-de1020c-preview-p1-overlay.png) |
| 08-two-page | pdflatex | main | 3/3 | ok | 25.6175 | 255 | 0.1843 | 0.1545 | 0.5685 | -/-/- | - | - | - | - | 0/0 | [p1](images/08-two-page/pdflatex-main-preview-p1-overlay.png) |
| 08-two-page | xelatex | de1020c | 3/3 | ok | 26.2441 | 255 | 0.1881 | 0.1577 | 0.5658 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | main | 3/3 | ok | 25.6199 | 255 | 0.1849 | 0.1549 | 0.5678 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.4844 | 255 | 0.0174 | 0.0146 | 0.9504 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | recovered | 2.3216 | 255 | 0.0164 | 0.0137 | 0.9553 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.4854 | 255 | 0.0174 | 0.0146 | 0.9504 | -/-/- | - | - | - | - | 0/0 | [p1](images/09-mixed-document/pdflatex-de1020c-preview-p1-overlay.png) |
| 09-mixed-document | pdflatex | main | 1/1 | recovered | 2.3187 | 255 | 0.0164 | 0.0137 | 0.9553 | -/-/- | - | - | - | - | 0/0 | [p1](images/09-mixed-document/pdflatex-main-preview-p1-overlay.png) |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.4838 | 255 | 0.0174 | 0.0146 | 0.9503 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | recovered | 2.3191 | 255 | 0.0164 | 0.0137 | 0.9553 | -/-/- | - | - | - | - | 0/0 | - |

## Per-fixture details

### 01-plain-paragraph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935537, 648, 477, 427, 319, 237, 160, 146, 149, 113, 138, 79, 113, 76, 54, 143]`; ink px ref/ours 2442/2421 (ratio 0.9914); SSIM blocks <0.9: 131/30294; images not emitted for this engine
- largest word displacements (pt): `This` dx -0.75 dy 0.46; `is` dx -0.65 dy 0.46; `one` dx -0.62 dy 0.46

### 01-plain-paragraph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933434, 485, 443, 352, 320, 296, 304, 271, 268, 250, 228, 284, 235, 225, 285, 1136]`; ink px ref/ours 2442/2430 (ratio 0.9951); SSIM blocks <0.9: 241/30294; images not emitted for this engine
- largest word displacements (pt): `line.` dx 54.81 dy 0.46; `single` dx 47.0 dy 0.46; `a` dx 45.93 dy 0.46

### 01-plain-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935388, 716, 424, 359, 290, 196, 196, 126, 136, 124, 122, 86, 78, 86, 81, 408]`; ink px ref/ours 2408/2421 (ratio 1.0054); SSIM blocks <0.9: 119/30294; [overlay](images/01-plain-paragraph/pdflatex-de1020c-export-p1-overlay.png) (21734 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (19120 B, ÷1)
- largest word displacements (pt): `line.` dx 1.15 dy 0.46; `single` dx 1.03 dy 0.46; `a` dx 1.0 dy 0.46

### 01-plain-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933403, 559, 486, 299, 334, 257, 272, 310, 275, 265, 246, 249, 240, 235, 294, 1092]`; ink px ref/ours 2408/2430 (ratio 1.0091); SSIM blocks <0.9: 239/30294; [overlay](images/01-plain-paragraph/pdflatex-main-export-p1-overlay.png) (22432 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-main-export-p1-heatmap.png) (23482 B, ÷1)
- largest word displacements (pt): `line.` dx 55.83 dy 0.46; `single` dx 48.02 dy 0.46; `a` dx 46.96 dy 0.46

### 01-plain-paragraph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935492, 657, 509, 378, 314, 240, 190, 128, 161, 125, 139, 77, 101, 98, 61, 146]`; ink px ref/ours 2441/2421 (ratio 0.9918); SSIM blocks <0.9: 134/30294; images not emitted for this engine
- largest word displacements (pt): `This` dx -0.75 dy 0.46; `is` dx -0.66 dy 0.46; `one` dx -0.62 dy 0.46

### 01-plain-paragraph — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933436, 464, 463, 334, 325, 294, 301, 272, 286, 245, 254, 249, 243, 222, 275, 1153]`; ink px ref/ours 2441/2430 (ratio 0.9955); SSIM blocks <0.9: 241/30294; images not emitted for this engine
- largest word displacements (pt): `line.` dx 54.78 dy 0.46; `single` dx 46.97 dy 0.46; `a` dx 45.91 dy 0.46

### 02-wrapping-paragraph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831807, 8259, 7034, 6392, 5883, 5835, 5922, 5218, 5432, 5617, 5346, 5301, 5198, 5202, 5188, 25182]`; ink px ref/ours 39630/39464 (ratio 0.9958); SSIM blocks <0.9: 4366/30294; images not emitted for this engine
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1830529, 8254, 7295, 6533, 5849, 5847, 6131, 5473, 5470, 5708, 5245, 5173, 5410, 5234, 5129, 25536]`; ink px ref/ours 39630/39432 (ratio 0.995); SSIM blocks <0.9: 4650/30294; images not emitted for this engine
- largest word displacements (pt): `every` dx -441.44 dy 29.03; `jumps` dx 425.04 dy 0.23; `oak` dx -415.78 dy 34.94

### 02-wrapping-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831980, 8181, 7016, 6406, 5963, 5948, 5852, 5289, 5188, 5656, 5267, 5378, 5255, 4876, 5221, 25340]`; ink px ref/ours 39390/39464 (ratio 1.0019); SSIM blocks <0.9: 4347/30294; [overlay](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-overlay.png) (229355 B, ÷1), [heatmap](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (258780 B, ÷1)
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1830566, 8140, 7223, 6569, 5872, 6023, 5839, 5496, 5243, 5723, 5216, 5199, 5485, 5101, 5224, 25897]`; ink px ref/ours 39390/39432 (ratio 1.0011); SSIM blocks <0.9: 4640/30294; [overlay](images/02-wrapping-paragraph/pdflatex-main-export-p1-overlay.png) (230641 B, ÷1), [heatmap](images/02-wrapping-paragraph/pdflatex-main-export-p1-heatmap.png) (262181 B, ÷1)
- largest word displacements (pt): `every` dx -441.92 dy 29.03; `jumps` dx 425.04 dy 0.23; `oak` dx -415.72 dy 34.94

### 02-wrapping-paragraph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831948, 8135, 6967, 6429, 5850, 5891, 5942, 5391, 5434, 5606, 5386, 5299, 5280, 5136, 5055, 25067]`; ink px ref/ours 39556/39464 (ratio 0.9977); SSIM blocks <0.9: 4369/30294; images not emitted for this engine
- largest word displacements (pt): `branch` dx -435.52 dy 20.54; `branch` dx -435.52 dy 14.86; `over` dx -406.99 dy 20.5

### 02-wrapping-paragraph — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1830415, 8062, 7253, 6686, 5876, 5895, 6016, 5628, 5414, 5736, 5274, 5145, 5553, 5263, 5069, 25531]`; ink px ref/ours 39556/39432 (ratio 0.9969); SSIM blocks <0.9: 4653/30294; images not emitted for this engine
- largest word displacements (pt): `every` dx -441.49 dy 29.03; `jumps` dx 425.04 dy 0.23; `oak` dx -415.83 dy 34.94

### 03-section-heading — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923800, 859, 844, 658, 666, 649, 719, 577, 609, 557, 596, 606, 682, 626, 750, 5618]`; ink px ref/ours 5807/4705 (ratio 0.8102); SSIM blocks <0.9: 660/30294; images not emitted for this engine
- largest word displacements (pt): `heading.` dx 0.58 dy 26.34; `second` dx 0.44 dy 26.34; `a` dx 0.41 dy 26.34

### 03-section-heading — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924346, 807, 845, 662, 681, 672, 736, 575, 623, 629, 589, 584, 637, 622, 693, 5115]`; ink px ref/ours 5807/4731 (ratio 0.8147); SSIM blocks <0.9: 626/30294; images not emitted for this engine
- largest word displacements (pt): `heading.` dx 12.77 dy 32.34; `second` dx 9.6 dy 32.34; `a` dx 8.53 dy 32.34
- word-sequence differences: replace ref ['Body', 'text'] ours ['Bodytext']; replace ref ['More', 'body'] ours ['Morebody']

### 03-section-heading — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923855, 915, 733, 647, 632, 615, 615, 519, 606, 614, 675, 690, 668, 592, 707, 5733]`; ink px ref/ours 6093/4705 (ratio 0.7722); SSIM blocks <0.9: 662/30294; [overlay](images/03-section-heading/pdflatex-de1020c-export-p1-overlay.png) (34997 B, ÷1), [heatmap](images/03-section-heading/pdflatex-de1020c-export-p1-heatmap.png) (42932 B, ÷1)
- largest word displacements (pt): `heading.` dx 0.75 dy 26.08; `second` dx 0.61 dy 26.08; `a` dx 0.58 dy 26.08

### 03-section-heading — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924386, 858, 723, 647, 631, 645, 611, 537, 599, 675, 654, 683, 648, 625, 649, 5245]`; ink px ref/ours 6093/4731 (ratio 0.7765); SSIM blocks <0.9: 628/30294; [overlay](images/03-section-heading/pdflatex-main-export-p1-overlay.png) (35002 B, ÷1), [heatmap](images/03-section-heading/pdflatex-main-export-p1-heatmap.png) (42656 B, ÷1)
- largest word displacements (pt): `heading.` dx 12.94 dy 32.08; `second` dx 9.77 dy 32.08; `a` dx 8.7 dy 32.08
- word-sequence differences: replace ref ['Body', 'text'] ours ['Bodytext']; replace ref ['More', 'body'] ours ['Morebody']

### 03-section-heading — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923747, 880, 847, 676, 683, 663, 736, 610, 596, 553, 616, 586, 654, 628, 733, 5608]`; ink px ref/ours 5744/4705 (ratio 0.8191); SSIM blocks <0.9: 660/30294; images not emitted for this engine
- largest word displacements (pt): `heading.` dx 0.57 dy 26.34; `second` dx 0.43 dy 26.34; `a` dx 0.4 dy 26.34

### 03-section-heading — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924301, 832, 850, 663, 678, 676, 737, 611, 606, 629, 619, 573, 624, 629, 667, 5121]`; ink px ref/ours 5744/4731 (ratio 0.8236); SSIM blocks <0.9: 626/30294; images not emitted for this engine
- largest word displacements (pt): `heading.` dx 12.76 dy 32.34; `second` dx 9.59 dy 32.34; `a` dx 8.52 dy 32.34
- word-sequence differences: replace ref ['Body', 'text'] ours ['Bodytext']; replace ref ['More', 'body'] ours ['Morebody']

### 04-bold-emph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933574, 491, 483, 352, 286, 278, 321, 275, 308, 283, 235, 240, 216, 217, 213, 1044]`; ink px ref/ours 2712/2465 (ratio 0.9089); SSIM blocks <0.9: 227/30294; images not emitted for this engine
- largest word displacements (pt): `emphasised` dx 5.28 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933226, 516, 453, 392, 323, 309, 314, 251, 256, 279, 230, 276, 221, 277, 228, 1265]`; ink px ref/ours 2712/2473 (ratio 0.9119); SSIM blocks <0.9: 251/30294; images not emitted for this engine
- largest word displacements (pt): `line.` dx 37.4 dy 0.46; `one` dx 36.31 dy 0.46; `on` dx 35.89 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933598, 513, 444, 319, 342, 300, 270, 233, 301, 263, 246, 223, 207, 251, 234, 1072]`; ink px ref/ours 2746/2465 (ratio 0.8977); SSIM blocks <0.9: 225/30294; [overlay](images/04-bold-emph/pdflatex-de1020c-export-p1-overlay.png) (23521 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-de1020c-export-p1-heatmap.png) (23530 B, ÷1)
- largest word displacements (pt): `emphasised` dx 5.27 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933132, 561, 432, 362, 327, 335, 272, 276, 263, 281, 255, 295, 202, 265, 251, 1307]`; ink px ref/ours 2746/2473 (ratio 0.9006); SSIM blocks <0.9: 254/30294; [overlay](images/04-bold-emph/pdflatex-main-export-p1-overlay.png) (23957 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-main-export-p1-heatmap.png) (24624 B, ÷1)
- largest word displacements (pt): `line.` dx 37.97 dy 0.46; `one` dx 36.87 dy 0.46; `on` dx 36.45 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933519, 519, 462, 366, 293, 287, 321, 265, 298, 295, 232, 216, 218, 229, 229, 1067]`; ink px ref/ours 2702/2465 (ratio 0.9123); SSIM blocks <0.9: 227/30294; images not emitted for this engine
- largest word displacements (pt): `emphasised` dx 5.26 dy 0.46; `bold` dx 5.16 dy 0.46; `and` dx 5.12 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933192, 532, 436, 403, 312, 330, 336, 228, 274, 278, 218, 286, 228, 261, 230, 1272]`; ink px ref/ours 2702/2473 (ratio 0.9152); SSIM blocks <0.9: 252/30294; images not emitted for this engine
- largest word displacements (pt): `line.` dx 37.49 dy 0.46; `one` dx 36.39 dy 0.46; `on` dx 35.97 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 05-unicode — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934471, 531, 434, 351, 255, 281, 235, 186, 150, 219, 214, 199, 193, 149, 197, 751]`; ink px ref/ours 2129/2123 (ratio 0.9972); SSIM blocks <0.9: 193/30294; images not emitted for this engine
- largest word displacements (pt): `dash.` dx 30.41 dy 0.46; `also` dx -5.46 dy 0.46; `dash;` dx -4.07 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933840, 456, 420, 338, 287, 265, 246, 211, 205, 278, 259, 228, 217, 232, 273, 1061]`; ink px ref/ours 2129/2087 (ratio 0.9803); SSIM blocks <0.9: 229/30294; images not emitted for this engine
- largest word displacements (pt): `dash.` dx 72.93 dy 0.46; `also` dx 6.37 dy 0.46; `café` dx 4.48 dy 0.46
- word-sequence differences: replace ref ['Resume', '—'] ours ['Resume—']; replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na"', 'ive', 'caf', "'", 'e', "R'", 'esum', "'"]

### 05-unicode — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934287, 570, 367, 345, 277, 258, 239, 232, 233, 241, 245, 185, 188, 173, 231, 745]`; ink px ref/ours 2119/2123 (ratio 1.0019); SSIM blocks <0.9: 209/30294; [overlay](images/05-unicode/pdflatex-de1020c-export-p1-overlay.png) (22053 B, ÷1), [heatmap](images/05-unicode/pdflatex-de1020c-export-p1-heatmap.png) (22231 B, ÷1)
- largest word displacements (pt): `dash.` dx 30.73 dy 0.46; `also` dx -5.31 dy 0.46; `dash;` dx -3.91 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933836, 574, 375, 303, 247, 303, 260, 232, 226, 227, 222, 236, 190, 225, 257, 1103]`; ink px ref/ours 2119/2087 (ratio 0.9849); SSIM blocks <0.9: 225/30294; [overlay](images/05-unicode/pdflatex-main-export-p1-overlay.png) (22329 B, ÷1), [heatmap](images/05-unicode/pdflatex-main-export-p1-heatmap.png) (22987 B, ÷1)
- largest word displacements (pt): `dash.` dx 73.25 dy 0.46; `also` dx 6.52 dy 0.46; `café` dx 4.66 dy 0.46
- word-sequence differences: replace ref ['Resume', '—'] ours ['Resume—']; replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na"', 'ive', 'caf', "'", 'e', "R'", 'esum', "'"]

### 05-unicode — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934491, 509, 440, 374, 248, 251, 251, 167, 176, 216, 207, 211, 173, 149, 193, 760]`; ink px ref/ours 2139/2123 (ratio 0.9925); SSIM blocks <0.9: 190/30294; images not emitted for this engine
- largest word displacements (pt): `dash.` dx 30.37 dy 0.46; `also` dx -5.49 dy 0.46; `dash;` dx -4.09 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933837, 454, 385, 372, 269, 271, 266, 200, 235, 239, 269, 252, 196, 218, 276, 1077]`; ink px ref/ours 2139/2087 (ratio 0.9757); SSIM blocks <0.9: 230/30294; images not emitted for this engine
- largest word displacements (pt): `dash.` dx 72.89 dy 0.46; `also` dx 6.34 dy 0.46; `café` dx 4.48 dy 0.46
- word-sequence differences: replace ref ['Resume', '—'] ours ['Resume—']; replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na"', 'ive', 'caf', "'", 'e', "R'", 'esum', "'"]

### 06-math-inline — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933844, 435, 352, 317, 197, 257, 297, 287, 248, 200, 246, 306, 299, 245, 218, 1068]`; ink px ref/ours 1729/1833 (ratio 1.0602); SSIM blocks <0.9: 216/30294; images not emitted for this engine
- largest word displacements (pt): `inside` dx -8.16 dy 1.32; `a` dx -8.04 dy 1.32; `sentence` dx -8.01 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (10): error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: math mode is not implemented in this version; error: math mode is not implemented in this version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935166, 437, 365, 287, 214, 209, 224, 221, 194, 148, 135, 137, 227, 156, 161, 535]`; ink px ref/ours 1729/1692 (ratio 0.9786); SSIM blocks <0.9: 173/30294; images not emitted for this engine
- largest word displacements (pt): `,` dx 22.47 dy -2.62; `text.` dx 18.76 dy -2.62; `b` dx 18.36 dy -2.62
- word-sequence differences: delete ref ['α'] ours []; replace ref ['β,'] ours [',']; replace ref ['√x'] ours ['x']

### 06-math-inline — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933877, 403, 304, 272, 266, 302, 299, 308, 222, 194, 272, 315, 237, 246, 231, 1068]`; ink px ref/ours 1699/1833 (ratio 1.0789); SSIM blocks <0.9: 211/30294; [overlay](images/06-math-inline/pdflatex-de1020c-export-p1-overlay.png) (20351 B, ÷1), [heatmap](images/06-math-inline/pdflatex-de1020c-export-p1-heatmap.png) (22295 B, ÷1)
- largest word displacements (pt): `a` dx -7.78 dy 1.32; `sentence` dx -7.76 dy 1.32; `of` dx -7.59 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']; replace ref ['√xinside'] ours ['√x', 'inside']

### 06-math-inline — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (10): error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: math mode is not implemented in this version; error: math mode is not implemented in this version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934966, 408, 304, 225, 247, 266, 228, 202, 210, 202, 166, 138, 230, 222, 187, 615]`; ink px ref/ours 1699/1692 (ratio 0.9959); SSIM blocks <0.9: 177/30294; [overlay](images/06-math-inline/pdflatex-main-export-p1-overlay.png) (19162 B, ÷1), [heatmap](images/06-math-inline/pdflatex-main-export-p1-heatmap.png) (19806 B, ÷1)
- largest word displacements (pt): `,` dx 22.73 dy -2.62; `text.` dx 19.01 dy -2.62; `b` dx 18.63 dy -2.62
- word-sequence differences: replace ref ['α+', 'β,'] ours ['+', ',']; replace ref ['√xinside'] ours ['x', 'inside']

### 06-math-inline — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933853, 435, 339, 322, 197, 256, 302, 277, 256, 193, 249, 311, 284, 255, 208, 1079]`; ink px ref/ours 1739/1833 (ratio 1.0541); SSIM blocks <0.9: 216/30294; images not emitted for this engine
- largest word displacements (pt): `inside` dx -8.16 dy 1.32; `a` dx -8.04 dy 1.32; `sentence` dx -8.02 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (10): error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: math mode is not implemented in this version; error: math mode is not implemented in this version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935179, 436, 363, 286, 208, 215, 221, 214, 190, 151, 135, 138, 220, 158, 161, 541]`; ink px ref/ours 1739/1692 (ratio 0.973); SSIM blocks <0.9: 172/30294; images not emitted for this engine
- largest word displacements (pt): `,` dx 22.47 dy -2.62; `text.` dx 18.74 dy -2.62; `b` dx 18.36 dy -2.62
- word-sequence differences: delete ref ['α'] ours []; replace ref ['β,'] ours [',']; replace ref ['√x'] ours ['x']

### 07-math-display — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934500, 594, 464, 552, 281, 307, 196, 220, 289, 154, 134, 152, 181, 132, 125, 535]`; ink px ref/ours 1954/1796 (ratio 0.9191); SSIM blocks <0.9: 191/30294; images not emitted for this engine
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \sum is not supported by this compiler version; error: \frac is not supported by this compiler version
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933162, 516, 504, 380, 287, 292, 264, 344, 305, 194, 212, 252, 273, 282, 243, 1306]`; ink px ref/ours 1954/1992 (ratio 1.0194); SSIM blocks <0.9: 322/30294; images not emitted for this engine
- largest word displacements (pt): `display.` dx 256.59 dy -49.17; `the` dx 252.83 dy -49.17; `After` dx 247.68 dy -49.17
- word-sequence differences: insert ref [] ours ['[', '_', 'i=1', '^', 'n', 'i', '=', 'n(n+1)']; delete ref ['n', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 07-math-display — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934567, 583, 430, 473, 321, 297, 196, 195, 302, 176, 128, 141, 187, 135, 135, 550]`; ink px ref/ours 1978/1796 (ratio 0.908); SSIM blocks <0.9: 193/30294; [overlay](images/07-math-display/pdflatex-de1020c-export-p1-overlay.png) (20903 B, ÷1), [heatmap](images/07-math-display/pdflatex-de1020c-export-p1-heatmap.png) (21635 B, ÷1)
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.1; `display.` dx 0.17 dy 2.13; `the` dx 0.11 dy 2.13
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \sum is not supported by this compiler version; error: \frac is not supported by this compiler version
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933310, 520, 453, 277, 349, 281, 230, 270, 305, 224, 226, 261, 300, 293, 256, 1261]`; ink px ref/ours 1978/1992 (ratio 1.0071); SSIM blocks <0.9: 321/30294; [overlay](images/07-math-display/pdflatex-main-export-p1-overlay.png) (21521 B, ÷1), [heatmap](images/07-math-display/pdflatex-main-export-p1-heatmap.png) (24038 B, ÷1)
- largest word displacements (pt): `display.` dx 256.59 dy -48.99; `the` dx 252.83 dy -48.99; `After` dx 247.68 dy -48.99
- word-sequence differences: insert ref [] ours ['[', '_', 'i=1', '^', 'n', 'i', '=', 'n(n+1)']; delete ref ['n', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 07-math-display — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934499, 617, 467, 539, 276, 309, 204, 216, 283, 145, 142, 139, 179, 138, 120, 543]`; ink px ref/ours 1971/1796 (ratio 0.9112); SSIM blocks <0.9: 191/30294; images not emitted for this engine
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \sum is not supported by this compiler version; error: \frac is not supported by this compiler version
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933173, 509, 515, 368, 292, 296, 277, 305, 325, 194, 215, 242, 276, 282, 249, 1298]`; ink px ref/ours 1971/1992 (ratio 1.0107); SSIM blocks <0.9: 323/30294; images not emitted for this engine
- largest word displacements (pt): `display.` dx 256.59 dy -49.17; `the` dx 252.83 dy -49.17; `After` dx 247.68 dy -49.17
- word-sequence differences: insert ref [] ours ['[', '_', 'i=1', '^', 'n', 'i', '=', 'n(n+1)']; delete ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)', '2'] ours []

### 08-two-page — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1535837, 30452, 26147, 24533, 22450, 21872, 22500, 20593, 20315, 21336, 19731, 19135, 19424, 19064, 19858, 95569]`; ink px ref/ours 152381/134585 (ratio 0.8832); SSIM blocks <0.9: 16227/30294; images not emitted for this engine
- page 2: |Δ| histogram (16 bins, pixel counts) `[1518144, 30651, 25967, 24422, 22157, 21752, 22680, 20693, 20985, 22451, 20965, 20617, 21112, 20181, 20664, 105375]`; ink px ref/ours 151462/131743 (ratio 0.8698); SSIM blocks <0.9: 16822/30294; images not emitted for this engine
- page 3: |Δ| histogram (16 bins, pixel counts) `[1774169, 11550, 9778, 9157, 8337, 8439, 8946, 8134, 7667, 8840, 8115, 7824, 8036, 7751, 8314, 43759]`; ink px ref/ours 33623/71008 (ratio 2.1119); SSIM blocks <0.9: 7433/30294; images not emitted for this engine
- largest word displacements (pt): `branch` dx -435.47 dy 214.06; `branch` dx -435.47 dy 167.32; `over` dx -407.04 dy 214.02

### 08-two-page — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1548695, 29233, 25670, 23719, 21651, 20365, 21497, 19722, 19775, 20729, 18734, 18131, 19542, 18754, 19003, 93596]`; ink px ref/ours 152381/122345 (ratio 0.8029); SSIM blocks <0.9: 16041/30294; images not emitted for this engine
- page 2: |Δ| histogram (16 bins, pixel counts) `[1544809, 28759, 25246, 23507, 21305, 19998, 21397, 19557, 19795, 20870, 19229, 18870, 20047, 19467, 19074, 96886]`; ink px ref/ours 151462/120329 (ratio 0.7945); SSIM blocks <0.9: 16085/30294; images not emitted for this engine
- page 3: |Δ| histogram (16 bins, pixel counts) `[1752033, 13457, 12137, 11259, 10234, 9446, 10594, 9391, 9094, 10181, 8736, 8552, 9481, 9085, 8988, 46148]`; ink px ref/ours 33623/95315 (ratio 2.8348); SSIM blocks <0.9: 8915/30294; images not emitted for this engine
- largest word displacements (pt): `every` dx -421.18 dy 357.83; `leaf` dx -416.72 dy 357.83; `oak` dx -415.78 dy 343.66

### 08-two-page — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p1-overlay.png) (114051 B, ÷4), [heatmap](images/08-two-page/pdflatex-de1020c-export-p1-heatmap.png) (110400 B, ÷4)
- page 2: |Δ| histogram (16 bins, pixel counts) `[1518465, 30739, 25693, 24091, 22137, 22533, 22535, 20944, 20824, 22593, 20919, 20497, 20504, 19480, 20990, 105872]`; ink px ref/ours 150460/131743 (ratio 0.8756); SSIM blocks <0.9: 16800/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p2-overlay.png) (111991 B, ÷4), [heatmap](images/08-two-page/pdflatex-de1020c-export-p2-heatmap.png) (118556 B, ÷4)
- page 3: |Δ| histogram (16 bins, pixel counts) `[1774335, 11537, 9689, 9104, 8316, 8407, 8934, 8213, 7615, 9012, 8045, 7866, 7928, 7590, 8264, 43961]`; ink px ref/ours 33504/71008 (ratio 2.1194); SSIM blocks <0.9: 7408/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p3-overlay.png) (299144 B, ÷1), [heatmap](images/08-two-page/pdflatex-de1020c-export-p3-heatmap.png) (155079 B, ÷2)
- largest word displacements (pt): `branch` dx -435.47 dy 214.06; `branch` dx -435.47 dy 167.32; `over` dx -407.04 dy 214.02

### 08-two-page — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1549572, 29117, 24955, 23023, 21429, 21097, 20957, 19777, 19815, 21019, 18877, 18345, 19028, 18538, 18698, 94569]`; ink px ref/ours 151753/122345 (ratio 0.8062); SSIM blocks <0.9: 15975/30294; [overlay](images/08-two-page/pdflatex-main-export-p1-overlay.png) (110474 B, ÷4), [heatmap](images/08-two-page/pdflatex-main-export-p1-heatmap.png) (109928 B, ÷4)
- page 2: |Δ| histogram (16 bins, pixel counts) `[1545165, 28799, 24888, 23299, 21335, 20712, 21197, 19722, 19780, 20973, 18979, 18828, 19731, 18910, 19216, 97282]`; ink px ref/ours 150460/120329 (ratio 0.7997); SSIM blocks <0.9: 16074/30294; [overlay](images/08-two-page/pdflatex-main-export-p2-overlay.png) (110256 B, ÷4), [heatmap](images/08-two-page/pdflatex-main-export-p2-heatmap.png) (112716 B, ÷4)
- page 3: |Δ| histogram (16 bins, pixel counts) `[1752289, 13298, 11951, 11251, 10131, 9703, 10597, 9350, 9171, 10218, 8691, 8641, 9427, 8933, 8973, 46192]`; ink px ref/ours 33504/95315 (ratio 2.8449); SSIM blocks <0.9: 8916/30294; [overlay](images/08-two-page/pdflatex-main-export-p3-overlay.png) (150457 B, ÷2), [heatmap](images/08-two-page/pdflatex-main-export-p3-heatmap.png) (178878 B, ÷2)
- largest word displacements (pt): `oak` dx -415.72 dy 343.66; `branch` dx -414.11 dy 343.66; `lazy` dx -397.58 dy 357.79

### 08-two-page — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1535378, 29989, 25555, 24965, 22556, 22246, 22654, 20669, 20453, 21240, 20149, 19662, 19349, 18997, 19406, 95548]`; ink px ref/ours 152232/134585 (ratio 0.8841); SSIM blocks <0.9: 16215/30294; images not emitted for this engine
- page 2: |Δ| histogram (16 bins, pixel counts) `[1517468, 30567, 25348, 24959, 22402, 22356, 22688, 20537, 21047, 22558, 21491, 21046, 20679, 20011, 20481, 105178]`; ink px ref/ours 151458/131743 (ratio 0.8698); SSIM blocks <0.9: 16818/30294; images not emitted for this engine
- page 3: |Δ| histogram (16 bins, pixel counts) `[1773969, 11449, 9657, 9335, 8367, 8575, 8971, 8031, 7696, 8844, 8288, 7974, 7985, 7726, 8258, 43691]`; ink px ref/ours 33659/71008 (ratio 2.1096); SSIM blocks <0.9: 7435/30294; images not emitted for this engine
- largest word displacements (pt): `branch` dx -435.52 dy 214.06; `branch` dx -435.52 dy 167.32; `over` dx -406.99 dy 214.02

### 08-two-page — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1548670, 28704, 25063, 24306, 21684, 20836, 21583, 19837, 19686, 20593, 19140, 18733, 19418, 18899, 18424, 93240]`; ink px ref/ours 152232/122345 (ratio 0.8037); SSIM blocks <0.9: 16034/30294; images not emitted for this engine
- page 2: |Δ| histogram (16 bins, pixel counts) `[1544263, 28723, 24614, 24103, 21402, 20775, 21409, 19315, 19962, 20913, 19563, 19355, 19624, 19271, 18951, 96573]`; ink px ref/ours 151458/120329 (ratio 0.7945); SSIM blocks <0.9: 16075/30294; images not emitted for this engine
- page 3: |Δ| histogram (16 bins, pixel counts) `[1751921, 13398, 11996, 11341, 10221, 9619, 10654, 9394, 9145, 10215, 8807, 8660, 9499, 9009, 8863, 46074]`; ink px ref/ours 33659/95315 (ratio 2.8318); SSIM blocks <0.9: 8919/30294; images not emitted for this engine
- largest word displacements (pt): `every` dx -421.22 dy 357.83; `leaf` dx -416.78 dy 357.83; `oak` dx -415.83 dy 343.66

### 09-mixed-document — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908424, 2029, 1775, 1940, 1563, 1506, 1540, 1401, 1384, 1449, 1711, 1473, 1402, 1450, 1477, 8292]`; ink px ref/ours 9691/9252 (ratio 0.9547); SSIM blocks <0.9: 1549/30294; images not emitted for this engine
- largest word displacements (pt): `twelve` dx -2.87 dy 39.06; `in` dx -2.66 dy 39.06; `set` dx -2.46 dy 39.06
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (5): error: math mode is not implemented in this version; error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: \frac is not supported by this compiler version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1910179, 2083, 1606, 1572, 1552, 1435, 1521, 1288, 1507, 1472, 1470, 1503, 1448, 1372, 1434, 7374]`; ink px ref/ours 9691/9829 (ratio 1.0142); SSIM blocks <0.9: 1417/30294; images not emitted for this engine
- largest word displacements (pt): `set` dx -407.05 dy 29.34; `in` dx -402.22 dy 29.34; `twelve` dx -399.41 dy 29.34
- word-sequence differences: replace ref ['A', 'paragraph'] ours ['Aparagraph']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['UTF-8', 'word'] ours ['UTF-8word']

### 09-mixed-document — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908448, 1975, 1818, 1995, 1553, 1434, 1436, 1374, 1435, 1503, 1793, 1550, 1349, 1400, 1470, 8283]`; ink px ref/ours 9857/9252 (ratio 0.9386); SSIM blocks <0.9: 1551/30294; [overlay](images/09-mixed-document/pdflatex-de1020c-export-p1-overlay.png) (65093 B, ÷1), [heatmap](images/09-mixed-document/pdflatex-de1020c-export-p1-heatmap.png) (81088 B, ÷1)
- largest word displacements (pt): `twelve` dx -3.05 dy 39.17; `in` dx -2.78 dy 39.17; `set` dx -2.52 dy 39.17
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (5): error: math mode is not implemented in this version; error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: \frac is not supported by this compiler version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1910227, 2061, 1671, 1613, 1534, 1352, 1406, 1277, 1543, 1501, 1490, 1564, 1428, 1377, 1452, 7320]`; ink px ref/ours 9857/9829 (ratio 0.9972); SSIM blocks <0.9: 1415/30294; [overlay](images/09-mixed-document/pdflatex-main-export-p1-overlay.png) (66505 B, ÷1), [heatmap](images/09-mixed-document/pdflatex-main-export-p1-heatmap.png) (79098 B, ÷1)
- largest word displacements (pt): `set` dx -407.11 dy 29.45; `in` dx -402.34 dy 29.45; `twelve` dx -399.59 dy 29.45
- word-sequence differences: replace ref ['A', 'paragraph'] ours ['Aparagraph']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['UTF-8', 'word'] ours ['UTF-8word']

### 09-mixed-document — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908485, 1993, 1726, 1891, 1568, 1563, 1547, 1422, 1387, 1527, 1642, 1498, 1332, 1389, 1540, 8306]`; ink px ref/ours 9680/9252 (ratio 0.9558); SSIM blocks <0.9: 1553/30294; images not emitted for this engine
- largest word displacements (pt): `twelve` dx -2.91 dy 39.06; `in` dx -2.65 dy 39.06; `set` dx -2.4 dy 39.06
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (5): error: math mode is not implemented in this version; error: math mode is not implemented in this version; error: \frac is not supported by this compiler version; error: \frac is not supported by this compiler version …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1910224, 2061, 1576, 1544, 1584, 1450, 1537, 1311, 1504, 1525, 1402, 1485, 1387, 1338, 1496, 7392]`; ink px ref/ours 9680/9829 (ratio 1.0154); SSIM blocks <0.9: 1418/30294; images not emitted for this engine
- largest word displacements (pt): `set` dx -406.99 dy 29.34; `in` dx -402.21 dy 29.34; `twelve` dx -399.45 dy 29.34
- word-sequence differences: replace ref ['A', 'paragraph'] ours ['Aparagraph']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['UTF-8', 'word'] ours ['UTF-8word']

## Per-fixture thresholds

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

## Limitations and honesty notes

- Reference engines and fonts: pdflatex uses the psnfss `times` package (URW Nimbus Roman clone); xelatex and lualatex use the macOS system `Times New Roman` TrueType via fontspec. Neither is byte-identical to the Times-Roman standard-14 face CoreGraphics substitutes when rasterizing the FlashTeX PDF.
- Compiler build `main` (origin/main) has no math support: math fixtures compile with status `recovered` and the math is rendered as plain text; `de1020c` typesets math with Unicode symbols and U+2500 rule runs.
- The preview-equivalent raster re-implements the app's CoreText draw; it is not a capture of the SwiftUI preview.
- Metrics are for these fixtures, this DPI, these builds and this machine only.
- PDFKit exposes text selections only; it cannot report rule/line geometry from the reference PDFs, so rule presence and position are compared from ink rows of the rasters (≥10pt contiguous dark run), and FlashTeX's own rectangle geometry (`re f`) is listed only for cross-checking.
- The pdflatex reference uses the URW Nimbus Roman clone (`times` package), xelatex/lualatex use the macOS Times New Roman TrueType, and FlashTeX's PDF uses the standard-14 `Times-Roman` name resolved by the rasterizer's CoreGraphics PDF engine (plus an embedded Times New Roman subset for non-WinAnsi glyphs). Glyph outlines therefore differ slightly even where positions agree; the metrics include that font-substitution noise.
- FlashTeX has no justification, hyphenation, kerning or ligatures; line breaks and word x positions diverge progressively along a line. Page-level SSIM over text is dominated by that, not by glyph rendering.
- Missing pages (page-count mismatch) are reported, not silently skipped.
