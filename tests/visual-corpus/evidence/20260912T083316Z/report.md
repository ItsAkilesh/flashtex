# FlashTeX visual corpus: reference-render and raster-diff evidence

Generated 20260912T083316Z on mac-m1max-a by `tests/visual-corpus/harness/run.sh`.

**Scope statement.** These are narrow-case measurements over a small declared corpus. They never claim general pixel perfection, LaTeX compatibility, or parity outside these fixtures, these engines, this font, this page size, this DPI and these builds. The reference engines are test oracles only; FlashTeX never invokes them and remains an original Rust implementation.

**Acceptance vs diagnostics.** The only acceptance signals in this report are the exact-equality gates below. They are three different questions and are never merged: (1) does FlashTeX reproduce its OWN pinned prior output (self-regression); (2) does the candidate match the ESTABLISHED ENGINE — raw, unmodified PDF bytes against the pinned MacTeX oracle PDF, and zero-pixel raster equality against the oracle raster; (3) does the export raster match the app's preview rasters (native/export parity). Every tolerance, threshold, SSIM, registration shift or regression comparison further down is a diagnostic to explain *why* something differs; none of them ever counts as acceptance. Nothing is normalised for acceptance: not IDs, not timestamps, not offsets.

**Plain statement.** Established-engine parity is NOT claimed: raw PDF bytes equal the pinned oracle in 0/324 candidate×oracle pairs and rasters are zero-pixel equal in 0/324. FlashTeX self-regression (bytes equal to its own pinned prior output) holds for 9/54 candidates — that is reproducibility of FlashTeX against itself, not LaTeX parity. Export = preview-equivalent for 0/54; export = native capture for 0/54 available captures (0 unavailable). Oracle pins live this run: 108/108 pinned oracle PDFs were re-rendered byte-identically by the installed engine.

## Gate 2 — candidate vs established engine (acceptance)

Oracle: MacTeX/TeX Live pdflatex (see Provenance for distribution, version, fonts, preamble, flags and the pinned render environment). Raw bytes: SHA-256 of the candidate `flashtex.pdf` vs the pinned oracle PDF from `oracle-profile.json` and vs this run's live oracle render, unmodified. Pixels: the export raster vs the oracle raster, same rasterizer, same DPI, page by page, zero tolerance; a page-count mismatch is DIFFERENT. Times-font oracles (`pdflatex`) match the current compiler's metrics; the Latin Modern oracles (`pdflatex-lm`) are LaTeX's default look. xelatex/lualatex variants are in `metrics.json` → `gates[].oracle`.

| Fixture | Compiler | Oracle | raw PDF bytes = pinned oracle | raw PDF bytes = live oracle | oracle live = pin | zero-pixel raster = oracle |
|---|---|---|---|---|---|---|
| 01-plain-paragraph | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4577/1938816 px, max |Δ| 255 |
| 01-plain-paragraph | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5863/1938816 px, max |Δ| 255 |
| 01-plain-paragraph | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4225/1938816 px, max |Δ| 255 |
| 01-plain-paragraph | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5851/1938816 px, max |Δ| 255 |
| 01-plain-paragraph | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6011/1938816 px, max |Δ| 255 |
| 01-plain-paragraph | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4980/1938816 px, max |Δ| 255 |
| 02-wrapping-paragraph | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 118539/1938816 px, max |Δ| 255 |
| 02-wrapping-paragraph | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 115714/1938816 px, max |Δ| 255 |
| 02-wrapping-paragraph | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 119257/1938816 px, max |Δ| 255 |
| 02-wrapping-paragraph | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 115964/1938816 px, max |Δ| 255 |
| 02-wrapping-paragraph | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 107139/1938816 px, max |Δ| 255 |
| 02-wrapping-paragraph | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 86635/1938816 px, max |Δ| 255 |
| 03-section-heading | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 16452/1938816 px, max |Δ| 255 |
| 03-section-heading | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 15822/1938816 px, max |Δ| 255 |
| 03-section-heading | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 16646/1938816 px, max |Δ| 255 |
| 03-section-heading | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 15920/1938816 px, max |Δ| 255 |
| 03-section-heading | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 13564/1938816 px, max |Δ| 255 |
| 03-section-heading | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 12084/1938816 px, max |Δ| 255 |
| 04-bold-emph | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6094/1938816 px, max |Δ| 255 |
| 04-bold-emph | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6441/1938816 px, max |Δ| 255 |
| 04-bold-emph | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6089/1938816 px, max |Δ| 255 |
| 04-bold-emph | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6461/1938816 px, max |Δ| 255 |
| 04-bold-emph | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6389/1938816 px, max |Δ| 255 |
| 04-bold-emph | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5850/1938816 px, max |Δ| 255 |
| 05-unicode | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5321/1938816 px, max |Δ| 255 |
| 05-unicode | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5663/1938816 px, max |Δ| 255 |
| 05-unicode | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4870/1938816 px, max |Δ| 255 |
| 05-unicode | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5642/1938816 px, max |Δ| 255 |
| 05-unicode | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5581/1938816 px, max |Δ| 255 |
| 05-unicode | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4749/1938816 px, max |Δ| 255 |
| 06-math-inline | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5596/1938816 px, max |Δ| 255 |
| 06-math-inline | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5578/1938816 px, max |Δ| 255 |
| 06-math-inline | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5578/1938816 px, max |Δ| 255 |
| 06-math-inline | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5620/1938816 px, max |Δ| 255 |
| 06-math-inline | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4621/1938816 px, max |Δ| 255 |
| 06-math-inline | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4641/1938816 px, max |Δ| 255 |
| 07-math-display | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5022/1938816 px, max |Δ| 255 |
| 07-math-display | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5681/1938816 px, max |Δ| 255 |
| 07-math-display | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5059/1938816 px, max |Δ| 255 |
| 07-math-display | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5720/1938816 px, max |Δ| 255 |
| 07-math-display | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5224/1938816 px, max |Δ| 255 |
| 07-math-display | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5213/1938816 px, max |Δ| 255 |
| 08-two-page | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 444978/1938816 px, max |Δ| 255 |
| 08-two-page | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 408459/1938816 px, max |Δ| 255 |
| 08-two-page | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 462968/1938816 px, max |Δ| 255 |
| 08-two-page | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 426669/1938816 px, max |Δ| 255 |
| 08-two-page | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 398503/1938816 px, max |Δ| 255 |
| 08-two-page | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 305063/1938816 px, max |Δ| 255 |
| 09-mixed-document | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 33705/1938816 px, max |Δ| 255 |
| 09-mixed-document | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 32465/1938816 px, max |Δ| 255 |
| 09-mixed-document | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 33859/1938816 px, max |Δ| 255 |
| 09-mixed-document | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 32558/1938816 px, max |Δ| 255 |
| 09-mixed-document | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 26725/1938816 px, max |Δ| 255 |
| 09-mixed-document | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 24333/1938816 px, max |Δ| 255 |
| 10-unicode-paragraph | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 50878/1938816 px, max |Δ| 255 |
| 10-unicode-paragraph | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 50399/1938816 px, max |Δ| 255 |
| 10-unicode-paragraph | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 50533/1938816 px, max |Δ| 255 |
| 10-unicode-paragraph | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 50446/1938816 px, max |Δ| 255 |
| 10-unicode-paragraph | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 51066/1938816 px, max |Δ| 255 |
| 10-unicode-paragraph | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 42821/1938816 px, max |Δ| 255 |
| 11-nested-lists | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 20125/1938816 px, max |Δ| 255 |
| 11-nested-lists | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19495/1938816 px, max |Δ| 255 |
| 11-nested-lists | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 20270/1938816 px, max |Δ| 255 |
| 11-nested-lists | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19733/1938816 px, max |Δ| 255 |
| 11-nested-lists | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19988/1938816 px, max |Δ| 255 |
| 11-nested-lists | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19410/1938816 px, max |Δ| 255 |
| 12-justified-paragraphs | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 214743/1938816 px, max |Δ| 255 |
| 12-justified-paragraphs | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 203863/1938816 px, max |Δ| 255 |
| 12-justified-paragraphs | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 209291/1938816 px, max |Δ| 255 |
| 12-justified-paragraphs | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 204394/1938816 px, max |Δ| 255 |
| 12-justified-paragraphs | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 191213/1938816 px, max |Δ| 255 |
| 12-justified-paragraphs | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 148671/1938816 px, max |Δ| 255 |
| 13-math-display-rich | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 7998/1938816 px, max |Δ| 255 |
| 13-math-display-rich | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 8341/1938816 px, max |Δ| 255 |
| 13-math-display-rich | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 8006/1938816 px, max |Δ| 255 |
| 13-math-display-rich | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 8452/1938816 px, max |Δ| 255 |
| 13-math-display-rich | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 8689/1938816 px, max |Δ| 255 |
| 13-math-display-rich | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 8509/1938816 px, max |Δ| 255 |
| 14-math-inline-dense | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19206/1938816 px, max |Δ| 255 |
| 14-math-inline-dense | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 18723/1938816 px, max |Δ| 255 |
| 14-math-inline-dense | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19110/1938816 px, max |Δ| 255 |
| 14-math-inline-dense | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 18691/1938816 px, max |Δ| 255 |
| 14-math-inline-dense | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 15872/1938816 px, max |Δ| 255 |
| 14-math-inline-dense | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 15573/1938816 px, max |Δ| 255 |
| 15-three-page-sections | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 378473/1938816 px, max |Δ| 255 |
| 15-three-page-sections | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 365118/1938816 px, max |Δ| 255 |
| 15-three-page-sections | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 379291/1938816 px, max |Δ| 255 |
| 15-three-page-sections | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 365671/1938816 px, max |Δ| 255 |
| 15-three-page-sections | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 318972/1938816 px, max |Δ| 255 |
| 15-three-page-sections | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 249928/1938816 px, max |Δ| 255 |
| 16-heading-page-break | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 444978/1938816 px, max |Δ| 255 |
| 16-heading-page-break | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 408459/1938816 px, max |Δ| 255 |
| 16-heading-page-break | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 462968/1938816 px, max |Δ| 255 |
| 16-heading-page-break | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 426669/1938816 px, max |Δ| 255 |
| 16-heading-page-break | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 398503/1938816 px, max |Δ| 255 |
| 16-heading-page-break | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 305063/1938816 px, max |Δ| 255 |
| 17-apostrophes | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 12870/1938816 px, max |Δ| 255 |
| 17-apostrophes | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 13649/1938816 px, max |Δ| 255 |
| 17-apostrophes | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 12981/1938816 px, max |Δ| 255 |
| 17-apostrophes | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 13722/1938816 px, max |Δ| 255 |
| 17-apostrophes | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 14004/1938816 px, max |Δ| 255 |
| 17-apostrophes | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 11818/1938816 px, max |Δ| 255 |
| 18-ligatures | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 20371/1938816 px, max |Δ| 255 |
| 18-ligatures | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 20720/1938816 px, max |Δ| 255 |
| 18-ligatures | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 20691/1938816 px, max |Δ| 255 |
| 18-ligatures | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 21485/1938816 px, max |Δ| 255 |
| 18-ligatures | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 20995/1938816 px, max |Δ| 255 |
| 18-ligatures | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 17828/1938816 px, max |Δ| 255 |

## Gate 1 — FlashTeX self-regression (byte identity with its own pinned prior output)

Pinned in `reference-profile.json`: the SHA-256 of FlashTeX's own `flashtex.pdf` (writer + compiler at pin time). EQUAL means the candidate reproduces the earlier FlashTeX output byte for byte; it is a reproducibility/regression signal only and says nothing about LaTeX.

| Fixture | Compiler | candidate SHA-256 | pinned prior FlashTeX SHA-256 | self-regression |
|---|---|---|---|---|
| 01-plain-paragraph | de1020c | `88deda9f24394700…` | `88deda9f24394700…` | **EQUAL** (reproduces prior FlashTeX output) |
| 01-plain-paragraph | main | `f51a106475288081…` | `982579044f6e075e…` | DIFFERENT from prior FlashTeX output |
| 01-plain-paragraph | pipeline | `834462c4a861d0b2…` | `-` | unpinned |
| 02-wrapping-paragraph | de1020c | `1bc0994be86d3ece…` | `1bc0994be86d3ece…` | **EQUAL** (reproduces prior FlashTeX output) |
| 02-wrapping-paragraph | main | `fe7f57bbb3d8591b…` | `a648c982d1829fad…` | DIFFERENT from prior FlashTeX output |
| 02-wrapping-paragraph | pipeline | `0a14a5c29583a221…` | `-` | unpinned |
| 03-section-heading | de1020c | `7f0f7247434d3132…` | `7f0f7247434d3132…` | **EQUAL** (reproduces prior FlashTeX output) |
| 03-section-heading | main | `8a7152d70e730c8f…` | `456aedda047b3d9f…` | DIFFERENT from prior FlashTeX output |
| 03-section-heading | pipeline | `1b037f2feadffe9d…` | `-` | unpinned |
| 04-bold-emph | de1020c | `7a34b0a51a2992f4…` | `7a34b0a51a2992f4…` | **EQUAL** (reproduces prior FlashTeX output) |
| 04-bold-emph | main | `bcb439c70e286a0f…` | `483817723c7a07a7…` | DIFFERENT from prior FlashTeX output |
| 04-bold-emph | pipeline | `9e1c832d9e02118c…` | `-` | unpinned |
| 05-unicode | de1020c | `e721656e20b80d47…` | `e721656e20b80d47…` | **EQUAL** (reproduces prior FlashTeX output) |
| 05-unicode | main | `6241e0ee71a8dfb6…` | `4a9c9cd453fdc68b…` | DIFFERENT from prior FlashTeX output |
| 05-unicode | pipeline | `9c47f67aa9f7c91a…` | `-` | unpinned |
| 06-math-inline | de1020c | `0a1a2a6eebc4b2b1…` | `0a1a2a6eebc4b2b1…` | **EQUAL** (reproduces prior FlashTeX output) |
| 06-math-inline | main | `1d8ba2668e230825…` | `055d64e818f4342a…` | DIFFERENT from prior FlashTeX output |
| 06-math-inline | pipeline | `441cf7de98417253…` | `-` | unpinned |
| 07-math-display | de1020c | `915882632c74f40b…` | `915882632c74f40b…` | **EQUAL** (reproduces prior FlashTeX output) |
| 07-math-display | main | `2dca277385195a4f…` | `ada42128ad2cb2f6…` | DIFFERENT from prior FlashTeX output |
| 07-math-display | pipeline | `b19366141ee72b26…` | `-` | unpinned |
| 08-two-page | de1020c | `6e8c29a5f28f5429…` | `6e8c29a5f28f5429…` | **EQUAL** (reproduces prior FlashTeX output) |
| 08-two-page | main | `f07b9750acdf8421…` | `44e782b1fe089026…` | DIFFERENT from prior FlashTeX output |
| 08-two-page | pipeline | `fab2b9688b9ce8b4…` | `-` | unpinned |
| 09-mixed-document | de1020c | `eb9ed31b44ff065e…` | `eb9ed31b44ff065e…` | **EQUAL** (reproduces prior FlashTeX output) |
| 09-mixed-document | main | `87b92a2ba0720b4a…` | `ed1c2d77c3510fb2…` | DIFFERENT from prior FlashTeX output |
| 09-mixed-document | pipeline | `7bc8cfecb157efdf…` | `-` | unpinned |
| 10-unicode-paragraph | de1020c | `405e9d22c4c242b1…` | `-` | unpinned |
| 10-unicode-paragraph | main | `501aa2a833350637…` | `-` | unpinned |
| 10-unicode-paragraph | pipeline | `3c183aef95c9a183…` | `-` | unpinned |
| 11-nested-lists | de1020c | `dddb38b9e9050f15…` | `-` | unpinned |
| 11-nested-lists | main | `5a39365af707eabf…` | `-` | unpinned |
| 11-nested-lists | pipeline | `20c2decbaa702819…` | `-` | unpinned |
| 12-justified-paragraphs | de1020c | `47e5eefae9a4319a…` | `-` | unpinned |
| 12-justified-paragraphs | main | `a608f87667eba822…` | `-` | unpinned |
| 12-justified-paragraphs | pipeline | `188eebf8739994c9…` | `-` | unpinned |
| 13-math-display-rich | de1020c | `80f03c44b40cd1b1…` | `-` | unpinned |
| 13-math-display-rich | main | `2513d596dd1a032e…` | `-` | unpinned |
| 13-math-display-rich | pipeline | `91550bdbb656aad9…` | `-` | unpinned |
| 14-math-inline-dense | de1020c | `04244b7cb4eedb5c…` | `-` | unpinned |
| 14-math-inline-dense | main | `9538e1767e50a0c1…` | `-` | unpinned |
| 14-math-inline-dense | pipeline | `6b258f9ad199d8cc…` | `-` | unpinned |
| 15-three-page-sections | de1020c | `9f2923c8d7e3dba9…` | `-` | unpinned |
| 15-three-page-sections | main | `c518936b68c53d5f…` | `-` | unpinned |
| 15-three-page-sections | pipeline | `953f1337a1cfcf36…` | `-` | unpinned |
| 16-heading-page-break | de1020c | `a1b742b01edb41a0…` | `-` | unpinned |
| 16-heading-page-break | main | `e239f36d95b5fd69…` | `-` | unpinned |
| 16-heading-page-break | pipeline | `94c0f12fc9603adc…` | `-` | unpinned |
| 17-apostrophes | de1020c | `aed18719902a6924…` | `-` | unpinned |
| 17-apostrophes | main | `95e3fc1ba4ffb981…` | `-` | unpinned |
| 17-apostrophes | pipeline | `8a0833c6633d92e9…` | `-` | unpinned |
| 18-ligatures | de1020c | `cccda2e7ef9bd851…` | `-` | unpinned |
| 18-ligatures | main | `5029f2456cd0ea4f…` | `-` | unpinned |
| 18-ligatures | pipeline | `0e230eae1171cff7…` | `-` | unpinned |

## Gate 3 — native/export parity (acceptance)

| Fixture | Compiler | export = preview-equivalent | export = native preview capture |
|---|---|---|---|
| 01-plain-paragraph | de1020c | DIFFERENT: 1240/1938816 px, max |Δ| 255 | DIFFERENT: 40484/1938816 px, max |Δ| 255 |
| 01-plain-paragraph | main | DIFFERENT: 1331/1938816 px, max |Δ| 255 | DIFFERENT: 40848/1938816 px, max |Δ| 255 |
| 01-plain-paragraph | pipeline | DIFFERENT: 1385/1938816 px, max |Δ| 255 | DIFFERENT: 42099/1938816 px, max |Δ| 255 |
| 02-wrapping-paragraph | de1020c | DIFFERENT: 17142/1938816 px, max |Δ| 255 | DIFFERENT: 365319/1938816 px, max |Δ| 255 |
| 02-wrapping-paragraph | main | DIFFERENT: 17429/1938816 px, max |Δ| 255 | DIFFERENT: 365993/1938816 px, max |Δ| 255 |
| 02-wrapping-paragraph | pipeline | DIFFERENT: 16693/1938816 px, max |Δ| 255 | DIFFERENT: 373551/1938816 px, max |Δ| 255 |
| 03-section-heading | de1020c | DIFFERENT: 2140/1938816 px, max |Δ| 6 | DIFFERENT: 73082/1938816 px, max |Δ| 229 |
| 03-section-heading | main | DIFFERENT: 1969/1938816 px, max |Δ| 6 | DIFFERENT: 78828/1938816 px, max |Δ| 255 |
| 03-section-heading | pipeline | DIFFERENT: 1875/1938816 px, max |Δ| 6 | DIFFERENT: 83964/1938816 px, max |Δ| 255 |
| 04-bold-emph | de1020c | DIFFERENT: 1261/1938816 px, max |Δ| 4 | DIFFERENT: 40994/1938816 px, max |Δ| 255 |
| 04-bold-emph | main | DIFFERENT: 1228/1938816 px, max |Δ| 4 | DIFFERENT: 41652/1938816 px, max |Δ| 255 |
| 04-bold-emph | pipeline | DIFFERENT: 1622/1938816 px, max |Δ| 4 | DIFFERENT: 44159/1938816 px, max |Δ| 255 |
| 05-unicode | de1020c | DIFFERENT: 587/1938816 px, max |Δ| 3 | DIFFERENT: 40902/1938816 px, max |Δ| 255 |
| 05-unicode | main | DIFFERENT: 441/1938816 px, max |Δ| 2 | DIFFERENT: 41619/1938816 px, max |Δ| 255 |
| 05-unicode | pipeline | DIFFERENT: 508/1938816 px, max |Δ| 2 | DIFFERENT: 38445/1938816 px, max |Δ| 255 |
| 06-math-inline | de1020c | DIFFERENT: 1041/1938816 px, max |Δ| 254 | DIFFERENT: 34297/1938816 px, max |Δ| 221 |
| 06-math-inline | main | DIFFERENT: 1113/1938816 px, max |Δ| 254 | DIFFERENT: 34559/1938816 px, max |Δ| 220 |
| 06-math-inline | pipeline | DIFFERENT: 1502/1938816 px, max |Δ| 255 | DIFFERENT: 36319/1938816 px, max |Δ| 255 |
| 07-math-display | de1020c | DIFFERENT: 1021/1938816 px, max |Δ| 255 | DIFFERENT: 34659/1938816 px, max |Δ| 255 |
| 07-math-display | main | DIFFERENT: 927/1938816 px, max |Δ| 255 | DIFFERENT: 37884/1938816 px, max |Δ| 255 |
| 07-math-display | pipeline | DIFFERENT: 1281/1938816 px, max |Δ| 255 | DIFFERENT: 41067/1938816 px, max |Δ| 255 |
| 08-two-page | de1020c | DIFFERENT: 58237/1938816 px, max |Δ| 255 | DIFFERENT: 1219606/1938816 px, max |Δ| 255 |
| 08-two-page | main | DIFFERENT: 64246/1938816 px, max |Δ| 255 | DIFFERENT: 1252916/1938816 px, max |Δ| 255 |
| 08-two-page | pipeline | DIFFERENT: 58512/1938816 px, max |Δ| 255 | DIFFERENT: 1294559/1938816 px, max |Δ| 255 |
| 09-mixed-document | de1020c | DIFFERENT: 4608/1938816 px, max |Δ| 250 | DIFFERENT: 144838/1938816 px, max |Δ| 255 |
| 09-mixed-document | main | DIFFERENT: 5035/1938816 px, max |Δ| 246 | DIFFERENT: 149086/1938816 px, max |Δ| 255 |
| 09-mixed-document | pipeline | DIFFERENT: 5078/1938816 px, max |Δ| 255 | DIFFERENT: 154280/1938816 px, max |Δ| 255 |
| 10-unicode-paragraph | de1020c | DIFFERENT: 9269/1938816 px, max |Δ| 255 | DIFFERENT: 181617/1938816 px, max |Δ| 255 |
| 10-unicode-paragraph | main | DIFFERENT: 9257/1938816 px, max |Δ| 255 | DIFFERENT: 182312/1938816 px, max |Δ| 255 |
| 10-unicode-paragraph | pipeline | DIFFERENT: 8489/1938816 px, max |Δ| 255 | DIFFERENT: 189155/1938816 px, max |Δ| 255 |
| 11-nested-lists | de1020c | DIFFERENT: 2764/1938816 px, max |Δ| 6 | DIFFERENT: 76470/1938816 px, max |Δ| 255 |
| 11-nested-lists | main | DIFFERENT: 2620/1938816 px, max |Δ| 6 | DIFFERENT: 83884/1938816 px, max |Δ| 255 |
| 11-nested-lists | pipeline | DIFFERENT: 2899/1938816 px, max |Δ| 6 | DIFFERENT: 71916/1938816 px, max |Δ| 255 |
| 12-justified-paragraphs | de1020c | DIFFERENT: 30049/1938816 px, max |Δ| 255 | DIFFERENT: 675052/1938816 px, max |Δ| 255 |
| 12-justified-paragraphs | main | DIFFERENT: 30072/1938816 px, max |Δ| 255 | DIFFERENT: 611349/1938816 px, max |Δ| 255 |
| 12-justified-paragraphs | pipeline | DIFFERENT: 27963/1938816 px, max |Δ| 255 | DIFFERENT: 683672/1938816 px, max |Δ| 255 |
| 13-math-display-rich | de1020c | DIFFERENT: 1699/1938816 px, max |Δ| 255 | DIFFERENT: 47153/1938816 px, max |Δ| 255 |
| 13-math-display-rich | main | DIFFERENT: 1359/1938816 px, max |Δ| 255 | DIFFERENT: 50584/1938816 px, max |Δ| 255 |
| 13-math-display-rich | pipeline | DIFFERENT: 3433/1938816 px, max |Δ| 255 | DIFFERENT: 63024/1938816 px, max |Δ| 255 |
| 14-math-inline-dense | de1020c | DIFFERENT: 3142/1938816 px, max |Δ| 255 | DIFFERENT: 102888/1938816 px, max |Δ| 255 |
| 14-math-inline-dense | main | DIFFERENT: 2835/1938816 px, max |Δ| 255 | DIFFERENT: 103534/1938816 px, max |Δ| 253 |
| 14-math-inline-dense | pipeline | DIFFERENT: 7075/1938816 px, max |Δ| 255 | DIFFERENT: 94338/1938816 px, max |Δ| 255 |
| 15-three-page-sections | de1020c | DIFFERENT: 49094/1938816 px, max |Δ| 255 | DIFFERENT: 1089710/1938816 px, max |Δ| 255 |
| 15-three-page-sections | main | DIFFERENT: 50857/1938816 px, max |Δ| 255 | DIFFERENT: 1097721/1938816 px, max |Δ| 255 |
| 15-three-page-sections | pipeline | DIFFERENT: 48008/1938816 px, max |Δ| 255 | DIFFERENT: 1046096/1938816 px, max |Δ| 255 |
| 16-heading-page-break | de1020c | DIFFERENT: 58237/1938816 px, max |Δ| 255 | DIFFERENT: 1219606/1938816 px, max |Δ| 255 |
| 16-heading-page-break | main | DIFFERENT: 64246/1938816 px, max |Δ| 255 | DIFFERENT: 1252916/1938816 px, max |Δ| 255 |
| 16-heading-page-break | pipeline | DIFFERENT: 58512/1938816 px, max |Δ| 255 | DIFFERENT: 1294559/1938816 px, max |Δ| 255 |
| 17-apostrophes | de1020c | DIFFERENT: 1919/1938816 px, max |Δ| 4 | DIFFERENT: 74246/1938816 px, max |Δ| 255 |
| 17-apostrophes | main | DIFFERENT: 1436/1938816 px, max |Δ| 3 | DIFFERENT: 75419/1938816 px, max |Δ| 255 |
| 17-apostrophes | pipeline | DIFFERENT: 2287/1938816 px, max |Δ| 255 | DIFFERENT: 77193/1938816 px, max |Δ| 255 |
| 18-ligatures | de1020c | DIFFERENT: 9644/1938816 px, max |Δ| 255 | DIFFERENT: 95192/1938816 px, max |Δ| 255 |
| 18-ligatures | main | DIFFERENT: 10092/1938816 px, max |Δ| 255 | DIFFERENT: 96490/1938816 px, max |Δ| 255 |
| 18-ligatures | pipeline | DIFFERENT: 9883/1938816 px, max |Δ| 255 | DIFFERENT: 97768/1938816 px, max |Δ| 255 |

- Classification of native-preview differences: the native capture comes from a screen capture at the display's backing scale, resampled to the raster size, so a DIFFERENT result there is expected to be dominated by resampling and text rasterization (CoreText on screen vs CoreGraphics PDF rendering); it is reported as-is, without normalisation. Preview-equivalent vs export differences isolate the drawing path (CoreText glyph run vs the PDF writer's text operators) from any capture effects. 'unavailable' means no capture was made this run.

## Provenance

- suite_branch: `mvo/rev2`
- suite_sha: `c2619a6164060c6ed31d903dbd70aeae9215633e`
- input_main_sha: `3423970b300e878062f2a124fb8004545626200b`
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
| lualatex | yes | This is LuaHBTeX, Version 1.24.0 (TeX Live 2026) | Times New Roman (`fontspec`, /System/Library/Fonts/Supplemental/Times New Roman.ttf) | `\pdfvariable trailerid{[<00000000000000000000000000000000> <00000000000000000000000000000000>]} \documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{Times New Roman} \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |
| lualatex-lm | yes | This is LuaHBTeX, Version 1.24.0 (TeX Live 2026) | Latin Modern Roman OpenType (`fontspec`, lmroman12-*.otf from the TeX Live tree by explicit path; bold-italic uses lmroman10-bolditalic, the only LM bold-italic face) | `\pdfvariable trailerid{[<00000000000000000000000000000000> <00000000000000000000000000000000>]} \documentclass[12pt]{article} \usepackage{fontspec} \setmainfont{lmroman12-regular.otf}[Path=/usr/local/texlive/2026/texmf-dist/fonts/opentype/public/lm/,BoldFont=lmroman12-bold.otf,ItalicFont=lmroman12-italic.otf,BoldItalicFont=lmroman10-bolditalic.otf] \usepackage[margin=1in]{geometry} \setlength{\parindent}{0pt} \setcounter{secnumdepth}{0} \pagestyle{empty} ` |

### Reference availability this run

- rendered fresh by an installed engine: 108 fixture/oracle pairs

The `-lm` oracles are the intended primary apples-to-apples target once a Latin-Modern-metrics FlashTeX pipeline exists; the Times oracles match the current compiler's Times metrics. Both are reported for every fixture.

TeX distribution this run: **MacTeX / TeX Live full (/usr/local/texlive/2026)**, texbin → `/usr/local/texlive/2026/bin/universal-darwin`, tlmgr revision 78301 (2026-03-07 18:41:28 +0100).

Pinned render environment for every oracle: `SOURCE_DATE_EPOCH=0 FORCE_SOURCE_DATE=1` (pdfTeX/XeTeX/LuaTeX write fixed dates and a deterministic trailer /ID; lualatex additionally receives an explicit `\pdfvariable trailerid` line, visible in its preamble above). Under this environment the same fixture renders to byte-identical PDFs run after run, which is what makes the raw-byte oracle pin meaningful; nothing is normalised after rendering.

Engine flags: `-interaction=batchmode -halt-on-error -file-line-error`. Page size: US letter 612×792 pt for every producer (checked per page from the MediaBox). LaTeX package versions: see `provenance.json` → `packages`.

### FlashTeX builds under test

- compiler `main`: `origin/main` @ `5b6911090e2dcbe807c6fb4d9e75cd4f214a2440` (crates/compiler) — Issue fresh parent assignment revisions for immediate tandem worker pickup
- compiler `de1020c`: `de1020c` @ `de1020cd0be7cede11be2691e00e7f5b15cb2224` (crates/compiler) — compiler: add math, real font metrics and PDF output
- compiler `pipeline`: `origin/agent/mac-render-pipeline/unified` @ `79ba728fb825df5b120509c9be2e9f3ba3cd9405` (crates/render-pipeline) — Merge remote-tracking branch 'origin/main' into rp/resume
- PDF writer: `5b5f7b5` @ `5b5f7b5bcbc44ba9376b3a237afe93ba1503d13c` (`flashtex-pdf --verify --embed-font auto`; body font Times-Roman standard-14, Unicode fallback subset of embedding subset of TimesNewRomanPSMT from /System/Library/Fonts/Supplemental/Times New Roman.ttf)
- export raster: the flashtex-pdf PDF rasterized by the same CoreGraphics rasterizer as the references
- preview-equivalent raster: `rasterize preview` re-implements the Mac app's `PDFExport.render` draw (CoreText `Times-Roman` at x_pt/baseline_y_pt/font_size_pt, U+2500 runs as 0.5em×0.0857em rules) straight into the bitmap. It links nothing from apps/mac and is **not** the SwiftUI preview; it is labelled preview-equivalent throughout.
- native preview capture: screencapture -x -o -l <CGWindowList id> of the running FlashTeXMac window; page rectangle detected as the largest white region bounded by the pane background; resampled with CoreGraphics high-quality interpolation to the 144-DPI raster size; the 'page N' caption region (bottom-right of the page) is masked white on the capture only. App: `/private/tmp/claude-501/-Users-jay3332-Projects-flashtex/e30fd4a4-f46a-4c3f-a28c-cbb8617b4425/scratchpad/app2-169c2a80ad857c26bbf1e67d3168f5ca5fc03f23/apps/mac/.build/debug/FlashTeXMac`. Display(s):  Resolution: 3024 x 1964 Retina; Resolution: 1920 x 1080 (1080p FHD - Full High Definition);. Capture backing factor(s): [2.0] px/pt; resample factor to the 144-DPI raster: 1.5242839352428394–1.5242839352428394 (>1 means the capture was UPSAMPLED, so glyph edges are interpolated and this comparison is coarser than the export one).

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
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.2612 | 0.9965 | (-0.5,0) | 0.1503 | 0.9983 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.57 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex | pipeline | 1/1 | ok | 0.3845 | 0.9943 | (0.5,0) weak | 0.3810 | 0.9944 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 12.23 | 0.41 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | de1020c | 1/1 | ok | 0.3662 | 0.9941 | (0,0) | 0.3662 | 0.9941 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.30 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | main | 1/1 | ok | 0.3601 | 0.9942 | (0,0) | 0.3601 | 0.9942 | 255 | 0.0030 | 0.0024 | 13/13/13 | yes | 12.75 | 0.30 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | pipeline | 1/1 | ok | 0.2368 | 0.9967 | (0,0) | 0.2368 | 0.9967 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.04 | 0.36 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.1873 | 0.9975 | (0,0) | 0.1873 | 0.9975 | 255 | 0.0024 | 0.0014 | 13/13/13 | yes | 0.43 | 0.46 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.1285 | 0.9985 | (0,0) | 0.1285 | 0.9985 | 255 | 0.0022 | 0.0011 | 13/13/13 | yes | 0.22 | 0.46 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-main-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | pipeline | 1/1 | ok | 0.3946 | 0.9944 | (1,0) weak | 0.3788 | 0.9947 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 12.75 | 0.41 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-pipeline-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 0.3660 | 0.9942 | (0,0) | 0.3660 | 0.9942 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | main | 1/1 | ok | 0.3597 | 0.9943 | (0,0) | 0.3597 | 0.9943 | 255 | 0.0030 | 0.0024 | 13/13/13 | yes | 12.76 | 0.73 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 0.2380 | 0.9966 | (0,0) | 0.2380 | 0.9966 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.03 | 0.67 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.1607 | 0.9980 | (0,0) | 0.1607 | 0.9980 | 255 | 0.0023 | 0.0014 | 13/13/13 | yes | 0.30 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.2670 | 0.9964 | (-0.5,0) | 0.1528 | 0.9982 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.58 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex | pipeline | 1/1 | ok | 0.3847 | 0.9943 | (0.5,0) weak | 0.3811 | 0.9944 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 12.22 | 0.41 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | de1020c | 1/1 | ok | 0.3665 | 0.9941 | (0,0) | 0.3665 | 0.9941 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | main | 1/1 | ok | 0.3603 | 0.9942 | (0,0) | 0.3603 | 0.9942 | 255 | 0.0030 | 0.0024 | 13/13/13 | yes | 12.75 | 0.73 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | pipeline | 1/1 | ok | 0.2367 | 0.9967 | (0,0) | 0.2367 | 0.9967 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.03 | 0.67 | 1.0000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 8.3842 | 0.8656 | (-3,0) weak | 8.3713 | 0.8657 | 255 | 0.0613 | 0.0509 | 210/210/210 | yes | 116.20 | 5.12 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 8.4139 | 0.8657 | (0,0) | 8.4139 | 0.8657 | 255 | 0.0616 | 0.0511 | 210/210/210 | yes | 90.99 | 4.50 | 0.9048 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | pipeline | 1/1 | ok | 7.1751 | 0.8901 | (0,0) | 7.1751 | 0.8901 | 255 | 0.0554 | 0.0449 | 210/210/210 | yes | 169.48 | 4.26 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | de1020c | 1/1 | ok | 7.7868 | 0.8620 | (-0.5,-14.5) weak | 7.7691 | 0.8602 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.57 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | main | 1/1 | ok | 7.7923 | 0.8614 | (0,0) | 7.7923 | 0.8614 | 255 | 0.0598 | 0.0495 | 210/210/210 | yes | 117.54 | 3.99 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | pipeline | 1/1 | ok | 4.5384 | 0.9368 | (0,0) | 4.5384 | 0.9368 | 255 | 0.0447 | 0.0336 | 210/210/210 | yes | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 8.3726 | 0.8663 | (0,0) | 8.3726 | 0.8663 | 255 | 0.0611 | 0.0509 | 210/210/210 | yes | 116.24 | 5.12 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 8.4700 | 0.8655 | (-3.5,0) weak | 8.4128 | 0.8661 | 255 | 0.0615 | 0.0512 | 210/210/210 | yes | 91.03 | 4.50 | 0.9048 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-main-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | pipeline | 1/1 | ok | 7.1603 | 0.8910 | (1,0) weak | 7.0822 | 0.8919 | 255 | 0.0553 | 0.0447 | 210/210/210 | yes | 169.52 | 4.26 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-pipeline-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 7.7862 | 0.8620 | (0,0) | 7.7862 | 0.8620 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | main | 1/1 | ok | 7.7924 | 0.8614 | (0,0) | 7.7924 | 0.8614 | 255 | 0.0598 | 0.0495 | 210/210/210 | yes | 117.55 | 4.30 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 4.5467 | 0.9367 | (0,0) | 4.5467 | 0.9367 | 255 | 0.0447 | 0.0337 | 210/210/210 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 8.3710 | 0.8656 | (0,0) | 8.3710 | 0.8656 | 255 | 0.0613 | 0.0509 | 210/210/210 | yes | 119.39 | 5.19 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 8.4639 | 0.8648 | (-0.5,0) weak | 8.4585 | 0.8648 | 255 | 0.0618 | 0.0514 | 210/210/210 | yes | 94.99 | 4.57 | 0.9048 | 0/0 | - |
| 02-wrapping-paragraph | xelatex | pipeline | 1/1 | ok | 7.1985 | 0.8902 | (0.5,0) weak | 7.1662 | 0.8904 | 255 | 0.0556 | 0.0451 | 210/210/210 | yes | 170.98 | 4.33 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | de1020c | 1/1 | ok | 7.7866 | 0.8620 | (-0.5,-14.5) weak | 7.7691 | 0.8602 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | main | 1/1 | ok | 7.7922 | 0.8614 | (0,0) | 7.7922 | 0.8614 | 255 | 0.0598 | 0.0495 | 210/210/210 | yes | 117.54 | 4.30 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | pipeline | 1/1 | ok | 4.5391 | 0.9368 | (0,0) | 4.5391 | 0.9368 | 255 | 0.0446 | 0.0336 | 210/210/210 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.3394 | 0.9793 | (0,52.5) moderate | 1.1861 | 0.9829 | 255 | 0.0085 | 0.0073 | 15/15/15 | yes | 0.25 | 21.02 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.3597 | 0.9789 | (0,52.5) moderate | 1.2063 | 0.9826 | 255 | 0.0085 | 0.0074 | 15/17/15 | no | 2.62 | 21.02 | 0.8667 | 0/0 | - |
| 03-section-heading | lualatex | pipeline | 1/1 | ok | 1.0038 | 0.9865 | (0,-1.5) weak | 0.9816 | 0.9874 | 255 | 0.0069 | 0.0058 | 15/17/15 | no | 11.43 | 0.70 | 0.8667 | 0/0 | - |
| 03-section-heading | lualatex-lm | de1020c | 1/1 | ok | 1.2336 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | 15/15/15 | yes | 5.46 | 21.19 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex-lm | main | 1/1 | ok | 1.2426 | 0.9769 | (0,53.5) moderate | 1.1129 | 0.9812 | 255 | 0.0082 | 0.0070 | 15/17/15 | no | 6.81 | 21.19 | 0.8667 | 0/0 | - |
| 03-section-heading | lualatex-lm | pipeline | 1/1 | ok | 0.8537 | 0.9887 | (0,0) | 0.8537 | 0.9887 | 255 | 0.0062 | 0.0052 | 15/17/15 | no | 5.82 | 0.45 | 0.8667 | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.3496 | 0.9793 | (0.5,52.5) moderate | 1.2021 | 0.9827 | 255 | 0.0085 | 0.0072 | 15/15/15 | yes | 0.33 | 20.87 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-de1020c-export-p1-overlay.png) |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.3719 | 0.9789 | (0.5,52.5) moderate | 1.2200 | 0.9824 | 255 | 0.0086 | 0.0074 | 15/17/15 | no | 2.70 | 20.87 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-main-export-p1-overlay.png) |
| 03-section-heading | pdflatex | pipeline | 1/1 | ok | 1.0318 | 0.9862 | (0.5,-2) weak | 1.0233 | 0.9859 | 255 | 0.0070 | 0.0059 | 15/17/15 | no | 11.51 | 0.87 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-pipeline-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | de1020c | 1/1 | ok | 1.2335 | 0.9771 | (0,54) moderate | 1.0954 | 0.9811 | 255 | 0.0082 | 0.0069 | 15/15/15 | yes | 5.46 | 22.37 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | main | 1/1 | ok | 1.2428 | 0.9769 | (0,54) moderate | 1.1208 | 0.9807 | 255 | 0.0082 | 0.0070 | 15/17/15 | no | 6.81 | 22.37 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-main-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | pipeline | 1/1 | ok | 0.8533 | 0.9887 | (0,0) | 0.8533 | 0.9887 | 255 | 0.0062 | 0.0052 | 15/17/15 | no | 5.82 | 0.73 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.3370 | 0.9793 | (0,52.5) moderate | 1.1868 | 0.9829 | 255 | 0.0085 | 0.0073 | 15/15/15 | yes | 0.24 | 21.02 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.3601 | 0.9789 | (0,52.5) moderate | 1.2075 | 0.9826 | 255 | 0.0086 | 0.0075 | 15/17/15 | no | 2.61 | 21.02 | 0.8667 | 0/0 | - |
| 03-section-heading | xelatex | pipeline | 1/1 | ok | 1.0090 | 0.9864 | (0,-1.5) weak | 0.9853 | 0.9874 | 255 | 0.0070 | 0.0059 | 15/17/15 | no | 11.43 | 0.70 | 0.8667 | 0/0 | - |
| 03-section-heading | xelatex-lm | de1020c | 1/1 | ok | 1.2337 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | 15/15/15 | yes | 5.46 | 22.35 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex-lm | main | 1/1 | ok | 1.2425 | 0.9769 | (0,53.5) moderate | 1.1129 | 0.9812 | 255 | 0.0082 | 0.0070 | 15/17/15 | no | 6.81 | 22.35 | 0.8667 | 0/0 | - |
| 03-section-heading | xelatex-lm | pipeline | 1/1 | ok | 0.8537 | 0.9887 | (0,0) | 0.8537 | 0.9887 | 255 | 0.0062 | 0.0052 | 15/17/15 | no | 5.82 | 0.70 | 0.8667 | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.3808 | 0.9945 | (0,0) | 0.3808 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 3.77 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.3769 | 0.9945 | (0,0) | 0.3769 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 3.73 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | pipeline | 1/1 | ok | 0.4278 | 0.9938 | (0,0) | 0.4278 | 0.9938 | 255 | 0.0033 | 0.0026 | 10/11/9 | no | 18.67 | 0.41 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | de1020c | 1/1 | ok | 0.4244 | 0.9933 | (-49,0) weak | 0.4148 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.02 | 0.63 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | main | 1/1 | ok | 0.4253 | 0.9933 | (-49,0) weak | 0.4147 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.07 | 0.63 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | pipeline | 1/1 | ok | 0.3472 | 0.9950 | (-0.5,0) weak | 0.3305 | 0.9952 | 255 | 0.0030 | 0.0023 | 10/11/9 | no | 0.53 | 0.68 | 1.0000 | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.3836 | 0.9944 | (-0.5,0) weak | 0.3764 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 4.04 | 0.46 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-de1020c-export-p1-overlay.png) |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.3804 | 0.9944 | (-0.5,0) weak | 0.3777 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 3.99 | 0.46 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-main-export-p1-overlay.png) |
| 04-bold-emph | pdflatex | pipeline | 1/1 | ok | 0.4169 | 0.9939 | (0,0) | 0.4169 | 0.9939 | 255 | 0.0033 | 0.0026 | 10/11/9 | no | 18.91 | 0.41 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-pipeline-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | de1020c | 1/1 | ok | 0.4246 | 0.9933 | (-49,0) weak | 0.4162 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.08 | 0.73 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | main | 1/1 | ok | 0.4256 | 0.9933 | (-49,0) weak | 0.4164 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.13 | 0.73 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-main-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | pipeline | 1/1 | ok | 0.3564 | 0.9948 | (-0.5,0) moderate | 0.3296 | 0.9953 | 255 | 0.0030 | 0.0024 | 10/11/9 | no | 0.58 | 0.67 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.3848 | 0.9944 | (-0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0031 | 0.0025 | 10/12/8 | no | 3.81 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.3810 | 0.9945 | (-0.5,0) weak | 0.3801 | 0.9944 | 255 | 0.0031 | 0.0025 | 10/12/8 | no | 3.76 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex | pipeline | 1/1 | ok | 0.4274 | 0.9938 | (0,0) | 0.4274 | 0.9938 | 255 | 0.0033 | 0.0026 | 10/11/9 | no | 18.70 | 0.41 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | de1020c | 1/1 | ok | 0.4245 | 0.9933 | (-49,0) weak | 0.4138 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 15.90 | 0.73 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | main | 1/1 | ok | 0.4237 | 0.9933 | (-49,0) weak | 0.4127 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 15.95 | 0.73 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | pipeline | 1/1 | ok | 0.3307 | 0.9952 | (0,0) | 0.3307 | 0.9952 | 255 | 0.0029 | 0.0023 | 10/11/9 | no | 0.42 | 0.67 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.2971 | 0.9955 | (0,0) | 0.2971 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.2489 | 0.9963 | (0,0) | 0.2489 | 0.9963 | 255 | 0.0026 | 0.0018 | 11/19/7 | no | 5.19 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex | pipeline | 1/1 | ok | 0.3579 | 0.9948 | (11.5,0) moderate | 0.3266 | 0.9952 | 255 | 0.0029 | 0.0022 | 11/11/11 | yes | 7.39 | 0.41 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex-lm | de1020c | 1/1 | ok | 0.3619 | 0.9938 | (2.5,0) weak | 0.3494 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.30 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex-lm | main | 1/1 | ok | 0.3628 | 0.9938 | (-3.5,0) moderate | 0.3432 | 0.9941 | 255 | 0.0029 | 0.0023 | 11/19/7 | no | 6.34 | 0.30 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex-lm | pipeline | 1/1 | ok | 0.2635 | 0.9960 | (-0.5,0) moderate | 0.2087 | 0.9969 | 255 | 0.0024 | 0.0019 | 11/11/11 | yes | 0.02 | 0.36 | 1.0000 | 0/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.3127 | 0.9955 | (0,0) | 0.3127 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.88 | 0.46 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-de1020c-export-p1-overlay.png) |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.2342 | 0.9967 | (0,0) | 0.2342 | 0.9967 | 255 | 0.0025 | 0.0016 | 11/19/7 | no | 5.21 | 0.46 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-main-export-p1-overlay.png) |
| 05-unicode | pdflatex | pipeline | 1/1 | ok | 0.3571 | 0.9950 | (11.5,0) moderate | 0.3059 | 0.9957 | 255 | 0.0029 | 0.0022 | 11/11/11 | yes | 7.59 | 0.41 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-pipeline-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | de1020c | 1/1 | ok | 0.3617 | 0.9938 | (2.5,0) weak | 0.3496 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | main | 1/1 | ok | 0.3625 | 0.9938 | (-3.5,0) moderate | 0.3430 | 0.9941 | 255 | 0.0029 | 0.0023 | 11/19/7 | no | 6.34 | 0.73 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-main-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | pipeline | 1/1 | ok | 0.2654 | 0.9960 | (-0.5,0) moderate | 0.2082 | 0.9969 | 255 | 0.0024 | 0.0019 | 11/11/11 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.2963 | 0.9955 | (0,0) | 0.2963 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.2493 | 0.9963 | (0,0) | 0.2493 | 0.9963 | 255 | 0.0026 | 0.0018 | 11/19/7 | no | 5.19 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex | pipeline | 1/1 | ok | 0.3577 | 0.9948 | (11.5,0) moderate | 0.3296 | 0.9951 | 255 | 0.0029 | 0.0022 | 11/11/11 | yes | 7.36 | 0.41 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex-lm | de1020c | 1/1 | ok | 0.3620 | 0.9938 | (2.5,0) weak | 0.3493 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | - |
| 05-unicode | xelatex-lm | main | 1/1 | ok | 0.3628 | 0.9938 | (-3.5,0) moderate | 0.3432 | 0.9941 | 255 | 0.0029 | 0.0023 | 11/19/7 | no | 6.34 | 0.73 | 1.0000 | 0/0 | - |
| 05-unicode | xelatex-lm | pipeline | 1/1 | ok | 0.2637 | 0.9960 | (-0.5,0) moderate | 0.2088 | 0.9969 | 255 | 0.0024 | 0.0019 | 11/11/11 | yes | 0.02 | 0.67 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.3832 | 0.9934 | (0,3.5) | 0.2554 | 0.9959 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | ok | 0.3838 | 0.9933 | (-6.5,3.5) | 0.1998 | 0.9967 | 255 | 0.0029 | 0.0023 | 15/13/12 | no | 3.50 | 2.46 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | pipeline | 1/1 | ok | 0.2941 | 0.9948 | (16.5,0) weak | 0.2801 | 0.9952 | 255 | 0.0024 | 0.0019 | 15/17/10 | no | 8.76 | 1.56 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | de1020c | 1/1 | ok | 0.3682 | 0.9931 | (-32.5,3.5) moderate | 0.3134 | 0.9945 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 12.25 | 2.20 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | main | 1/1 | ok | 0.3735 | 0.9928 | (9.5,3.5) moderate | 0.3120 | 0.9945 | 255 | 0.0029 | 0.0024 | 15/13/12 | no | 12.48 | 2.27 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | pipeline | 1/1 | ok | 0.2944 | 0.9949 | (-1,0) moderate | 0.2401 | 0.9960 | 255 | 0.0024 | 0.0019 | 15/17/10 | no | 0.99 | 1.55 | 1.0000 | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.3821 | 0.9936 | (0,3.5) | 0.2595 | 0.9960 | 255 | 0.0029 | 0.0023 | 13/14/10 | no | 3.89 | 2.69 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-de1020c-export-p1-overlay.png) |
| 06-math-inline | pdflatex | main | 1/1 | ok | 0.3782 | 0.9935 | (-6,3.5) | 0.2119 | 0.9968 | 255 | 0.0029 | 0.0023 | 13/13/10 | no | 3.10 | 2.69 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-main-export-p1-overlay.png) |
| 06-math-inline | pdflatex | pipeline | 1/1 | ok | 0.2818 | 0.9951 | (16.5,0) weak | 0.2776 | 0.9954 | 255 | 0.0024 | 0.0018 | 13/17/8 | no | 9.13 | 1.56 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-pipeline-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | de1020c | 1/1 | ok | 0.3680 | 0.9931 | (9.5,3.5) moderate | 0.3234 | 0.9943 | 255 | 0.0029 | 0.0023 | 14/14/12 | no | 13.02 | 2.53 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | main | 1/1 | ok | 0.3729 | 0.9928 | (9.5,3.5) moderate | 0.3115 | 0.9945 | 255 | 0.0029 | 0.0024 | 14/13/12 | no | 12.50 | 2.53 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-main-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | pipeline | 1/1 | ok | 0.2947 | 0.9949 | (-1,0) moderate | 0.2416 | 0.9959 | 255 | 0.0024 | 0.0019 | 14/17/9 | no | 0.96 | 1.74 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.3831 | 0.9934 | (0,3.5) | 0.2548 | 0.9959 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | ok | 0.3839 | 0.9933 | (-6.5,3.5) | 0.1996 | 0.9967 | 255 | 0.0029 | 0.0023 | 15/13/12 | no | 3.50 | 2.46 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex | pipeline | 1/1 | ok | 0.2947 | 0.9948 | (16.5,0) weak | 0.2809 | 0.9952 | 255 | 0.0024 | 0.0019 | 15/17/10 | no | 8.76 | 1.56 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | de1020c | 1/1 | ok | 0.3682 | 0.9931 | (9.5,3.5) moderate | 0.3233 | 0.9943 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 12.25 | 2.44 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | main | 1/1 | ok | 0.3734 | 0.9928 | (9.5,3.5) moderate | 0.3119 | 0.9945 | 255 | 0.0029 | 0.0024 | 15/13/12 | no | 12.48 | 2.53 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | pipeline | 1/1 | ok | 0.2944 | 0.9949 | (-1,0) moderate | 0.2401 | 0.9960 | 255 | 0.0024 | 0.0019 | 15/17/10 | no | 0.99 | 1.61 | 1.0000 | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.2595 | 0.9950 | (0,0) | 0.2595 | 0.9950 | 255 | 0.0026 | 0.0019 | 13/10/7 | no | 0.60 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | ok | 0.2640 | 0.9946 | (0,0) | 0.2640 | 0.9946 | 255 | 0.0026 | 0.0019 | 13/11/7 | no | 0.41 | 1.33 | 1.0000 | 3/0 | - |
| 07-math-display | lualatex | pipeline | 1/1 | ok | 0.3170 | 0.9943 | (0,0) | 0.3170 | 0.9943 | 255 | 0.0027 | 0.0021 | 13/17/9 | no | 3.18 | 2.40 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex-lm | de1020c | 1/1 | ok | 0.3495 | 0.9931 | (0,0) | 0.3495 | 0.9931 | 255 | 0.0029 | 0.0024 | 13/10/7 | no | 2.12 | 0.92 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex-lm | main | 1/1 | ok | 0.3541 | 0.9928 | (0,0) | 0.3541 | 0.9928 | 255 | 0.0029 | 0.0024 | 13/11/7 | no | 1.94 | 0.92 | 1.0000 | 3/0 | - |
| 07-math-display | lualatex-lm | pipeline | 1/1 | ok | 0.3008 | 0.9945 | (-0.5,0) weak | 0.2917 | 0.9945 | 255 | 0.0026 | 0.0021 | 13/17/9 | no | 1.86 | 2.25 | 1.0000 | 3/1 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.2609 | 0.9949 | (0,0) | 0.2609 | 0.9949 | 255 | 0.0026 | 0.0019 | 13/10/7 | no | 0.59 | 1.41 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-de1020c-export-p1-overlay.png) |
| 07-math-display | pdflatex | main | 1/1 | ok | 0.2654 | 0.9946 | (0,0) | 0.2654 | 0.9946 | 255 | 0.0026 | 0.0019 | 13/11/7 | no | 0.41 | 1.41 | 1.0000 | 3/0 | [p1](images/07-math-display/pdflatex-main-export-p1-overlay.png) |
| 07-math-display | pdflatex | pipeline | 1/1 | ok | 0.3118 | 0.9944 | (0,0) | 0.3118 | 0.9944 | 255 | 0.0027 | 0.0020 | 13/17/9 | no | 3.18 | 2.46 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-pipeline-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | de1020c | 1/1 | ok | 0.3528 | 0.9931 | (0,0) | 0.3528 | 0.9931 | 255 | 0.0029 | 0.0024 | 13/10/7 | no | 2.13 | 1.73 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | main | 1/1 | ok | 0.3574 | 0.9927 | (0,0) | 0.3574 | 0.9927 | 255 | 0.0029 | 0.0024 | 13/11/7 | no | 1.94 | 1.73 | 1.0000 | 3/0 | [p1](images/07-math-display/pdflatex-lm-main-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | pipeline | 1/1 | ok | 0.3003 | 0.9945 | (-0.5,0) weak | 0.2904 | 0.9945 | 255 | 0.0027 | 0.0021 | 13/17/9 | no | 1.86 | 2.77 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.2589 | 0.9950 | (0,0) | 0.2589 | 0.9950 | 255 | 0.0026 | 0.0019 | 14/10/7 | no | 0.59 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | ok | 0.2634 | 0.9946 | (0,0) | 0.2634 | 0.9946 | 255 | 0.0026 | 0.0019 | 14/11/7 | no | 0.41 | 1.33 | 1.0000 | 3/0 | - |
| 07-math-display | xelatex | pipeline | 1/1 | ok | 0.3167 | 0.9943 | (0,0) | 0.3167 | 0.9943 | 255 | 0.0027 | 0.0021 | 14/17/9 | no | 3.18 | 2.40 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex-lm | de1020c | 1/1 | ok | 0.3495 | 0.9931 | (0,0) | 0.3495 | 0.9931 | 255 | 0.0029 | 0.0024 | 14/10/7 | no | 2.12 | 1.54 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex-lm | main | 1/1 | ok | 0.3541 | 0.9928 | (0,0) | 0.3541 | 0.9928 | 255 | 0.0029 | 0.0024 | 14/11/7 | no | 1.94 | 1.54 | 1.0000 | 3/0 | - |
| 07-math-display | xelatex-lm | pipeline | 1/1 | ok | 0.3009 | 0.9945 | (-0.5,0) weak | 0.2918 | 0.9945 | 255 | 0.0026 | 0.0021 | 14/17/9 | no | 1.86 | 2.59 | 1.0000 | 3/1 | - |
| 08-two-page | lualatex | de1020c | 3/3 | ok | 26.2176 | 0.5657 | (0,0); (-3,6) moderate; (-3,34.5) moderate | 24.6323 | 0.5959 | 255 | 0.1880 | 0.1574 | 1800/1800/1800 | yes | 164.00 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | lualatex | main | 3/3 | ok | 25.2812 | 0.5914 | (0,0); (0,2) weak; (0,49) moderate | 24.7462 | 0.6038 | 255 | 0.1832 | 0.1528 | 1800/1800/1800 | yes | 138.34 | 31.84 | 0.9011 | 0/0 | - |
| 08-two-page | lualatex | pipeline | 3/3 | ok | 21.1557 | 0.6700 | (0,0); (0,0); (1,0) weak | 21.1019 | 0.6710 | 255 | 0.1619 | 0.1322 | 1800/1800/1800 | yes | 163.81 | 67.95 | 0.8960 | 0/0 | - |
| 08-two-page | lualatex-lm | de1020c | 3/3 | ok | 23.7967 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7513 | 0.5869 | 255 | 0.1801 | 0.1495 | 1800/1800/1800 | yes | 142.51 | 30.71 | 0.8878 | 0/0 | - |
| 08-two-page | lualatex-lm | main | 3/3 | ok | 23.2786 | 0.5778 | (-0.5,-27) weak; (-0.5,-12.5) weak; (-0.5,5.5) moderate | 22.8311 | 0.5859 | 255 | 0.1773 | 0.1467 | 1800/1800/1800 | yes | 160.68 | 38.33 | 0.8958 | 0/0 | - |
| 08-two-page | lualatex-lm | pipeline | 3/3 | ok | 12.9946 | 0.8187 | (0,0); (0,0); (0,0) | 12.9946 | 0.8187 | 255 | 0.1280 | 0.0963 | 1800/1800/1800 | yes | 0.00 | 0.36 | 1.0000 | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/3 | ok | 26.1974 | 0.5667 | (0,-14.5) weak; (0,6) moderate; (0,34.5) moderate | 24.5504 | 0.5980 | 255 | 0.1875 | 0.1572 | 1800/1800/1800 | yes | 143.13 | 92.17 | 0.8952 | 0/0 | [p1](images/08-two-page/pdflatex-de1020c-export-p1-overlay.png) |
| 08-two-page | pdflatex | main | 3/3 | ok | 25.3242 | 0.5921 | (0,0); (-11,2) weak; (0,49) moderate | 24.7896 | 0.6035 | 255 | 0.1829 | 0.1528 | 1800/1800/1800 | yes | 112.56 | 31.07 | 0.9011 | 0/0 | [p1](images/08-two-page/pdflatex-main-export-p1-overlay.png) |
| 08-two-page | pdflatex | pipeline | 3/3 | ok | 21.1720 | 0.6711 | (-1.5,0) weak; (-1.5,0) weak; (-1.5,0) weak | 21.0316 | 0.6731 | 255 | 0.1614 | 0.1321 | 1800/1800/1800 | yes | 179.69 | 67.14 | 0.8961 | 0/0 | [p1](images/08-two-page/pdflatex-pipeline-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | de1020c | 3/3 | ok | 23.7963 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7519 | 0.5868 | 255 | 0.1801 | 0.1496 | 1800/1800/1800 | yes | 142.52 | 31.52 | 0.8878 | 0/0 | [p1](images/08-two-page/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | main | 3/3 | ok | 23.2793 | 0.5778 | (-0.5,-27) weak; (-0.5,2) weak; (-0.5,5.5) moderate | 22.8313 | 0.5887 | 255 | 0.1772 | 0.1468 | 1800/1800/1800 | yes | 160.69 | 37.39 | 0.8958 | 0/0 | [p1](images/08-two-page/pdflatex-lm-main-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | pipeline | 3/3 | ok | 13.0228 | 0.8182 | (0,0); (0,0); (0,0) | 13.0228 | 0.8182 | 255 | 0.1281 | 0.0965 | 1800/1800/1800 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | [p1](images/08-two-page/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 08-two-page | xelatex | de1020c | 3/3 | ok | 26.2425 | 0.5656 | (-3,-14.5) weak; (-3,6) moderate; (-3,34.5) moderate | 24.6595 | 0.5953 | 255 | 0.1881 | 0.1578 | 1800/1800/1800 | yes | 164.01 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | xelatex | main | 3/3 | ok | 25.2630 | 0.5918 | (0,0); (0,2) weak; (0,49) moderate | 24.7282 | 0.6041 | 255 | 0.1831 | 0.1530 | 1800/1800/1800 | yes | 138.34 | 31.84 | 0.9011 | 0/0 | - |
| 08-two-page | xelatex | pipeline | 3/3 | ok | 21.1736 | 0.6696 | (1.5,0) weak; (0,0); (1,0) weak | 21.1122 | 0.6705 | 255 | 0.1620 | 0.1325 | 1800/1800/1800 | yes | 163.83 | 67.95 | 0.8960 | 0/0 | - |
| 08-two-page | xelatex-lm | de1020c | 3/3 | ok | 23.7969 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7509 | 0.5869 | 255 | 0.1801 | 0.1495 | 1800/1800/1800 | yes | 142.51 | 31.52 | 0.8878 | 0/0 | - |
| 08-two-page | xelatex-lm | main | 3/3 | ok | 23.2788 | 0.5778 | (-0.5,-27) weak; (-0.5,-12.5) weak; (-0.5,5.5) moderate | 22.8312 | 0.5859 | 255 | 0.1772 | 0.1467 | 1800/1800/1800 | yes | 160.68 | 37.39 | 0.8958 | 0/0 | - |
| 08-two-page | xelatex-lm | pipeline | 3/3 | ok | 12.9979 | 0.8186 | (0,0); (0,0); (0,0) | 12.9979 | 0.8186 | 255 | 0.1280 | 0.0963 | 1800/1800/1800 | yes | 0.00 | 0.67 | 1.0000 | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.4844 | 0.9504 | (0,38.5) | 1.7912 | 0.9702 | 255 | 0.0174 | 0.0146 | 54/54/45 | no | 2.41 | 31.76 | 0.9778 | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | ok | 2.5088 | 0.9499 | (0,38.5) | 1.6551 | 0.9717 | 255 | 0.0175 | 0.0147 | 54/56/43 | no | 3.15 | 32.26 | 0.9767 | 0/0 | - |
| 09-mixed-document | lualatex | pipeline | 1/1 | ok | 1.7735 | 0.9736 | (0,0) | 1.7735 | 0.9736 | 255 | 0.0136 | 0.0112 | 54/58/47 | no | 30.73 | 0.71 | 0.9149 | 0/0 | - |
| 09-mixed-document | lualatex-lm | de1020c | 1/1 | ok | 2.2538 | 0.9494 | (-0.5,39) moderate | 1.9017 | 0.9641 | 255 | 0.0169 | 0.0140 | 54/54/45 | no | 30.16 | 31.13 | 0.9333 | 0/0 | - |
| 09-mixed-document | lualatex-lm | main | 1/1 | ok | 2.2680 | 0.9490 | (-0.5,39) moderate | 1.8972 | 0.9637 | 255 | 0.0169 | 0.0141 | 54/56/43 | no | 32.35 | 31.57 | 0.9302 | 0/0 | - |
| 09-mixed-document | lualatex-lm | pipeline | 1/1 | ok | 1.3023 | 0.9810 | (0,0.5) weak | 1.2552 | 0.9820 | 255 | 0.0118 | 0.0093 | 54/58/47 | no | 0.83 | 0.36 | 0.9362 | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.4853 | 0.9504 | (-0.5,38.5) moderate | 1.8886 | 0.9685 | 255 | 0.0174 | 0.0146 | 54/54/45 | no | 2.48 | 31.80 | 0.9778 | 0/0 | [p1](images/09-mixed-document/pdflatex-de1020c-export-p1-overlay.png) |
| 09-mixed-document | pdflatex | main | 1/1 | ok | 2.5100 | 0.9498 | (0,38.5) | 1.8697 | 0.9687 | 255 | 0.0175 | 0.0147 | 54/56/43 | no | 3.22 | 32.30 | 0.9767 | 0/0 | [p1](images/09-mixed-document/pdflatex-main-export-p1-overlay.png) |
| 09-mixed-document | pdflatex | pipeline | 1/1 | ok | 1.8209 | 0.9729 | (1,0) weak | 1.8051 | 0.9730 | 255 | 0.0138 | 0.0115 | 54/58/47 | no | 30.85 | 0.79 | 0.9149 | 0/0 | [p1](images/09-mixed-document/pdflatex-pipeline-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | de1020c | 1/1 | ok | 2.2522 | 0.9480 | (-0.5,39.5) moderate | 1.9012 | 0.9623 | 255 | 0.0167 | 0.0139 | 54/54/45 | no | 30.17 | 32.47 | 0.9333 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | main | 1/1 | ok | 2.2668 | 0.9476 | (-0.5,39.5) moderate | 1.8976 | 0.9619 | 255 | 0.0168 | 0.0140 | 54/56/43 | no | 32.36 | 32.96 | 0.9302 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-main-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | pipeline | 1/1 | ok | 1.4737 | 0.9767 | (0,1) moderate | 1.3175 | 0.9792 | 255 | 0.0126 | 0.0101 | 54/58/47 | no | 0.82 | 1.40 | 0.9574 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.4838 | 0.9503 | (-0.5,38.5) | 1.8537 | 0.9690 | 255 | 0.0174 | 0.0146 | 54/54/45 | no | 2.45 | 31.76 | 0.9778 | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | ok | 2.5096 | 0.9498 | (0,38.5) | 1.8378 | 0.9690 | 255 | 0.0175 | 0.0147 | 54/56/43 | no | 3.18 | 32.26 | 0.9767 | 0/0 | - |
| 09-mixed-document | xelatex | pipeline | 1/1 | ok | 1.7898 | 0.9733 | (1,0) weak | 1.7793 | 0.9733 | 255 | 0.0137 | 0.0113 | 54/58/47 | no | 30.86 | 0.71 | 0.9149 | 0/0 | - |
| 09-mixed-document | xelatex-lm | de1020c | 1/1 | ok | 2.2539 | 0.9494 | (-0.5,39) moderate | 1.9016 | 0.9641 | 255 | 0.0169 | 0.0140 | 54/54/45 | no | 30.16 | 32.06 | 0.9333 | 0/0 | - |
| 09-mixed-document | xelatex-lm | main | 1/1 | ok | 2.2680 | 0.9490 | (-0.5,39) moderate | 1.8972 | 0.9637 | 255 | 0.0169 | 0.0141 | 54/56/43 | no | 32.35 | 32.55 | 0.9302 | 0/0 | - |
| 09-mixed-document | xelatex-lm | pipeline | 1/1 | ok | 1.3038 | 0.9810 | (0,0.5) weak | 1.2567 | 0.9820 | 255 | 0.0118 | 0.0093 | 54/58/47 | no | 0.84 | 1.00 | 0.9574 | 0/0 | - |
| 10-unicode-paragraph | lualatex | de1020c | 1/1 | ok | 3.3153 | 0.9490 | (0,0) | 3.3153 | 0.9490 | 255 | 0.0261 | 0.0212 | 8/81/0 | no | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | main | 1/1 | recovered | 3.3305 | 0.9495 | (0.5,0) weak | 3.2478 | 0.9509 | 255 | 0.0262 | 0.0213 | 8/82/0 | no | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex | pipeline | 1/1 | recovered | 3.3993 | 0.9471 | (0,0) | 3.3993 | 0.9471 | 255 | 0.0265 | 0.0215 | 8/81/0 | no | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex-lm | de1020c | 1/1 | ok | 3.2356 | 0.9447 | (0,0) | 3.2356 | 0.9447 | 255 | 0.0259 | 0.0211 | 82/81/79 | no | 64.10 | 1.33 | 0.8987 | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | main | 1/1 | recovered | 3.2488 | 0.9448 | (0,0) | 3.2488 | 0.9448 | 255 | 0.0259 | 0.0212 | 82/82/81 | no | 104.26 | 2.38 | 0.8765 | 0/0 | - |
| 10-unicode-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 2.3198 | 0.9657 | (-0.5,0) moderate | 2.1847 | 0.9681 | 255 | 0.0218 | 0.0166 | 82/81/81 | no | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 10-unicode-paragraph | pdflatex | de1020c | 1/1 | ok | 3.3620 | 0.9486 | (0,0) | 3.3620 | 0.9486 | 255 | 0.0262 | 0.0213 | 85/81/76 | no | 75.04 | 1.87 | 0.9211 | 3/1 | [p1](images/10-unicode-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex | main | 1/1 | recovered | 3.3173 | 0.9499 | (0,0) | 3.3173 | 0.9499 | 255 | 0.0261 | 0.0211 | 85/82/78 | no | 32.01 | 0.90 | 0.9231 | 3/0 | [p1](images/10-unicode-paragraph/pdflatex-main-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex | pipeline | 1/1 | recovered | 3.3581 | 0.9479 | (0,0) | 3.3581 | 0.9479 | 255 | 0.0263 | 0.0211 | 85/81/77 | no | 121.15 | 2.66 | 0.9091 | 3/0 | [p1](images/10-unicode-paragraph/pdflatex-pipeline-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 3.2510 | 0.9445 | (0,0) | 3.2510 | 0.9445 | 255 | 0.0260 | 0.0212 | 82/81/80 | no | 65.00 | 1.32 | 0.9000 | 0/1 | [p1](images/10-unicode-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | main | 1/1 | recovered | 3.2644 | 0.9446 | (0,0) | 3.2644 | 0.9446 | 255 | 0.0260 | 0.0213 | 82/82/82 | yes | 104.99 | 2.35 | 0.8780 | 0/0 | [p1](images/10-unicode-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 2.3867 | 0.9643 | (-0.5,0) moderate | 2.2154 | 0.9673 | 255 | 0.0221 | 0.0169 | 82/81/81 | no | 0.41 | 0.36 | 1.0000 | 0/0 | [p1](images/10-unicode-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 10-unicode-paragraph | xelatex | de1020c | 1/1 | ok | 3.3629 | 0.9483 | (0,0) | 3.3629 | 0.9483 | 255 | 0.0262 | 0.0213 | 84/81/78 | no | 78.17 | 2.01 | 0.9231 | 3/1 | - |
| 10-unicode-paragraph | xelatex | main | 1/1 | recovered | 3.3247 | 0.9496 | (0.5,0) weak | 3.2333 | 0.9511 | 255 | 0.0261 | 0.0212 | 84/82/80 | no | 31.11 | 0.89 | 0.9250 | 3/0 | - |
| 10-unicode-paragraph | xelatex | pipeline | 1/1 | recovered | 3.4027 | 0.9470 | (0,0) | 3.4027 | 0.9470 | 255 | 0.0265 | 0.0214 | 84/81/79 | no | 128.14 | 2.97 | 0.8987 | 3/0 | - |
| 10-unicode-paragraph | xelatex-lm | de1020c | 1/1 | ok | 3.2358 | 0.9447 | (0,0) | 3.2358 | 0.9447 | 255 | 0.0259 | 0.0211 | 82/81/79 | no | 64.10 | 1.45 | 0.8987 | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | main | 1/1 | recovered | 3.2489 | 0.9448 | (0,0) | 3.2489 | 0.9448 | 255 | 0.0259 | 0.0212 | 82/82/81 | no | 104.26 | 2.42 | 0.8765 | 0/0 | - |
| 10-unicode-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 2.3204 | 0.9657 | (-0.5,0) moderate | 2.1847 | 0.9681 | 255 | 0.0218 | 0.0166 | 82/81/81 | no | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex | de1020c | 1/1 | recovered | 1.4653 | 0.9696 | (18,-49) weak | 1.4605 | 0.9709 | 255 | 0.0104 | 0.0085 | 29/24/24 | no | 123.00 | 61.96 | 0.9167 | 0/0 | - |
| 11-nested-lists | lualatex | main | 1/1 | ok | 1.4714 | 0.9699 | (-22.5,-8) moderate | 1.1220 | 0.9800 | 255 | 0.0105 | 0.0087 | 29/29/29 | yes | 23.37 | 9.35 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex | pipeline | 1/1 | ok | 1.4784 | 0.9703 | (-16.5,-20) moderate | 1.3615 | 0.9752 | 255 | 0.0104 | 0.0087 | 29/29/29 | yes | 18.47 | 26.65 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (-2,-49) weak | 1.3067 | 0.9711 | 255 | 0.0100 | 0.0083 | 29/24/24 | no | 121.01 | 62.61 | 0.9167 | 0/0 | - |
| 11-nested-lists | lualatex-lm | main | 1/1 | ok | 1.3743 | 0.9689 | (-30.5,-8) moderate | 1.2508 | 0.9755 | 255 | 0.0102 | 0.0085 | 29/29/29 | yes | 25.69 | 10.02 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex-lm | pipeline | 1/1 | ok | 1.3304 | 0.9702 | (-17,-20) moderate | 1.1995 | 0.9761 | 255 | 0.0100 | 0.0083 | 29/29/29 | yes | 20.01 | 27.33 | 1.0000 | 0/0 | - |
| 11-nested-lists | pdflatex | de1020c | 1/1 | recovered | 1.4657 | 0.9698 | (19,-49) weak | 1.4558 | 0.9711 | 255 | 0.0104 | 0.0085 | 29/24/24 | no | 123.27 | 61.96 | 0.9167 | 0/0 | [p1](images/11-nested-lists/pdflatex-de1020c-export-p1-overlay.png) |
| 11-nested-lists | pdflatex | main | 1/1 | ok | 1.4604 | 0.9704 | (-22,-8) moderate | 1.1824 | 0.9790 | 255 | 0.0105 | 0.0086 | 29/29/29 | yes | 22.70 | 9.35 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-main-export-p1-overlay.png) |
| 11-nested-lists | pdflatex | pipeline | 1/1 | ok | 1.4478 | 0.9709 | (-16,-20) moderate | 1.3661 | 0.9757 | 255 | 0.0103 | 0.0085 | 29/29/29 | yes | 17.80 | 26.65 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-pipeline-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | de1020c | 1/1 | recovered | 1.3674 | 0.9687 | (23.5,-49) weak | 1.3216 | 0.9706 | 255 | 0.0101 | 0.0083 | 29/24/24 | no | 121.22 | 62.61 | 0.9167 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | main | 1/1 | ok | 1.3761 | 0.9690 | (-30,-8) moderate | 1.2546 | 0.9752 | 255 | 0.0102 | 0.0085 | 29/29/29 | yes | 25.08 | 10.02 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-main-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | pipeline | 1/1 | ok | 1.3363 | 0.9701 | (-17,-20) moderate | 1.2000 | 0.9760 | 255 | 0.0100 | 0.0083 | 29/29/29 | yes | 19.39 | 27.33 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 11-nested-lists | xelatex | de1020c | 1/1 | recovered | 1.4648 | 0.9696 | (18,-49) weak | 1.4600 | 0.9709 | 255 | 0.0104 | 0.0085 | 29/24/24 | no | 123.00 | 61.96 | 0.9167 | 0/0 | - |
| 11-nested-lists | xelatex | main | 1/1 | ok | 1.4708 | 0.9699 | (-22.5,-8) moderate | 1.1192 | 0.9800 | 255 | 0.0105 | 0.0087 | 29/29/29 | yes | 23.37 | 9.35 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex | pipeline | 1/1 | ok | 1.4782 | 0.9703 | (-16.5,-20) moderate | 1.3610 | 0.9752 | 255 | 0.0104 | 0.0087 | 29/29/29 | yes | 18.46 | 26.65 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (-2,-49) weak | 1.3068 | 0.9711 | 255 | 0.0100 | 0.0083 | 29/24/24 | no | 121.01 | 61.76 | 0.9167 | 0/0 | - |
| 11-nested-lists | xelatex-lm | main | 1/1 | ok | 1.3743 | 0.9689 | (-30.5,-8) moderate | 1.2509 | 0.9755 | 255 | 0.0102 | 0.0085 | 29/29/29 | yes | 25.69 | 9.15 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex-lm | pipeline | 1/1 | ok | 1.3305 | 0.9702 | (-17,-20) moderate | 1.1997 | 0.9761 | 255 | 0.0100 | 0.0083 | 29/29/29 | yes | 20.01 | 26.44 | 1.0000 | 0/0 | - |
| 12-justified-paragraphs | lualatex | de1020c | 1/1 | ok | 15.4038 | 0.7396 | (-3,0) weak | 15.3872 | 0.7397 | 255 | 0.1111 | 0.0927 | 360/360/360 | yes | 99.24 | 32.55 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex | main | 1/1 | ok | 14.8416 | 0.7604 | (-3.5,0) weak | 14.8082 | 0.7605 | 255 | 0.1082 | 0.0900 | 360/360/360 | yes | 72.49 | 10.31 | 0.9111 | 0/0 | - |
| 12-justified-paragraphs | lualatex | pipeline | 1/1 | ok | 12.9690 | 0.7908 | (0,14.5) weak | 12.9455 | 0.7922 | 255 | 0.0987 | 0.0805 | 360/360/360 | yes | 155.85 | 25.45 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | de1020c | 1/1 | ok | 13.7995 | 0.7503 | (-8.5,2.5) weak | 13.7712 | 0.7503 | 255 | 0.1052 | 0.0873 | 360/360/360 | yes | 83.76 | 8.29 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | main | 1/1 | ok | 13.7762 | 0.7469 | (-0.5,-26) weak | 13.7623 | 0.7495 | 255 | 0.1054 | 0.0872 | 360/360/360 | yes | 109.31 | 15.49 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | pipeline | 1/1 | ok | 7.8017 | 0.8918 | (0,0) | 7.8017 | 0.8918 | 255 | 0.0766 | 0.0577 | 360/360/360 | yes | 0.00 | 0.36 | 1.0000 | 0/0 | - |
| 12-justified-paragraphs | pdflatex | de1020c | 1/1 | ok | 15.3708 | 0.7415 | (-3,0) weak | 15.3659 | 0.7414 | 255 | 0.1108 | 0.0925 | 360/360/360 | yes | 99.31 | 32.55 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-de1020c-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex | main | 1/1 | ok | 14.8933 | 0.7603 | (-4,0) weak | 14.8302 | 0.7612 | 255 | 0.1079 | 0.0901 | 360/360/360 | yes | 72.58 | 10.31 | 0.9111 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-main-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex | pipeline | 1/1 | ok | 13.0297 | 0.7911 | (1,14.5) weak | 12.9203 | 0.7936 | 255 | 0.0986 | 0.0807 | 360/360/360 | yes | 155.88 | 25.45 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-pipeline-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | de1020c | 1/1 | ok | 13.7992 | 0.7503 | (-8.5,2.5) weak | 13.7701 | 0.7503 | 255 | 0.1051 | 0.0873 | 360/360/360 | yes | 83.76 | 8.99 | 0.8889 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | main | 1/1 | ok | 13.7769 | 0.7469 | (-0.5,0) weak | 13.6913 | 0.7482 | 255 | 0.1054 | 0.0872 | 360/360/360 | yes | 109.32 | 14.74 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-main-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | pipeline | 1/1 | ok | 7.8191 | 0.8916 | (0,0) | 7.8191 | 0.8916 | 255 | 0.0767 | 0.0579 | 360/360/360 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 12-justified-paragraphs | xelatex | de1020c | 1/1 | ok | 15.3809 | 0.7404 | (0,0) | 15.3809 | 0.7404 | 255 | 0.1111 | 0.0928 | 360/360/360 | yes | 106.70 | 32.71 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex | main | 1/1 | ok | 14.9399 | 0.7591 | (0,0) | 14.9399 | 0.7591 | 255 | 0.1085 | 0.0906 | 360/360/360 | yes | 81.84 | 10.47 | 0.9111 | 0/0 | - |
| 12-justified-paragraphs | xelatex | pipeline | 1/1 | ok | 13.0485 | 0.7899 | (0.5,14.5) weak | 12.9800 | 0.7920 | 255 | 0.0990 | 0.0810 | 360/360/360 | yes | 159.34 | 25.61 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | de1020c | 1/1 | ok | 13.7994 | 0.7503 | (-8.5,2.5) weak | 13.7708 | 0.7503 | 255 | 0.1051 | 0.0873 | 360/360/360 | yes | 83.76 | 8.99 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | main | 1/1 | ok | 13.7765 | 0.7469 | (-1,-26) weak | 13.7693 | 0.7493 | 255 | 0.1054 | 0.0872 | 360/360/360 | yes | 109.31 | 14.74 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | pipeline | 1/1 | ok | 7.8017 | 0.8918 | (0,0) | 7.8017 | 0.8918 | 255 | 0.0766 | 0.0577 | 360/360/360 | yes | 0.00 | 0.67 | 1.0000 | 0/0 | - |
| 13-math-display-rich | lualatex | de1020c | 1/1 | recovered | 0.4724 | 0.9903 | (0,3.5) moderate | 0.4325 | 0.9914 | 255 | 0.0040 | 0.0031 | 19/17/7 | no | 1.15 | 5.89 | 1.0000 | 2/2 | - |
| 13-math-display-rich | lualatex | main | 1/1 | recovered | 0.4988 | 0.9897 | (-0.5,3.5) weak | 0.4849 | 0.9904 | 255 | 0.0041 | 0.0032 | 19/17/7 | no | 1.78 | 5.89 | 1.0000 | 2/0 | - |
| 13-math-display-rich | lualatex | pipeline | 1/1 | recovered | 0.5693 | 0.9876 | (5.5,1.5) weak | 0.5610 | 0.9880 | 255 | 0.0045 | 0.0036 | 19/33/11 | no | 10.46 | 3.93 | 0.9091 | 2/2 | - |
| 13-math-display-rich | lualatex-lm | de1020c | 1/1 | recovered | 0.5515 | 0.9887 | (-2,4) weak | 0.5477 | 0.9892 | 255 | 0.0043 | 0.0035 | 19/17/7 | no | 3.38 | 5.45 | 1.0000 | 4/2 | - |
| 13-math-display-rich | lualatex-lm | main | 1/1 | recovered | 0.5520 | 0.9884 | (-0.5,3.5) weak | 0.5303 | 0.9890 | 255 | 0.0044 | 0.0035 | 19/17/7 | no | 4.16 | 5.45 | 1.0000 | 4/0 | - |
| 13-math-display-rich | lualatex-lm | pipeline | 1/1 | recovered | 0.5464 | 0.9877 | (3.5,1.5) moderate | 0.4859 | 0.9895 | 255 | 0.0044 | 0.0036 | 19/33/11 | no | 8.92 | 3.67 | 0.9091 | 4/2 | - |
| 13-math-display-rich | pdflatex | de1020c | 1/1 | recovered | 0.5010 | 0.9899 | (1,3.5) weak | 0.4808 | 0.9907 | 255 | 0.0041 | 0.0033 | 19/17/7 | no | 1.58 | 6.00 | 1.0000 | 2/2 | [p1](images/13-math-display-rich/pdflatex-de1020c-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex | main | 1/1 | recovered | 0.4817 | 0.9900 | (0,3.5) weak | 0.4734 | 0.9904 | 255 | 0.0041 | 0.0032 | 19/17/7 | no | 1.50 | 6.00 | 1.0000 | 2/0 | [p1](images/13-math-display-rich/pdflatex-main-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex | pipeline | 1/1 | recovered | 0.5728 | 0.9876 | (4,1.5) moderate | 0.5371 | 0.9885 | 255 | 0.0045 | 0.0036 | 19/33/11 | no | 10.73 | 4.00 | 0.9091 | 2/2 | [p1](images/13-math-display-rich/pdflatex-pipeline-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | de1020c | 1/1 | recovered | 0.5530 | 0.9888 | (-2,4) weak | 0.5406 | 0.9893 | 255 | 0.0043 | 0.0035 | 19/17/7 | no | 3.38 | 6.19 | 1.0000 | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | main | 1/1 | recovered | 0.5543 | 0.9885 | (-0.5,4) weak | 0.5397 | 0.9888 | 255 | 0.0044 | 0.0036 | 19/17/7 | no | 4.17 | 6.19 | 1.0000 | 4/0 | [p1](images/13-math-display-rich/pdflatex-lm-main-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | pipeline | 1/1 | recovered | 0.5487 | 0.9877 | (3.5,1.5) moderate | 0.4959 | 0.9893 | 255 | 0.0044 | 0.0036 | 19/33/11 | no | 8.92 | 4.16 | 0.9091 | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 13-math-display-rich | xelatex | de1020c | 1/1 | recovered | 0.4941 | 0.9899 | (0,0) | 0.4941 | 0.9899 | 255 | 0.0041 | 0.0032 | 21/17/8 | no | 1.56 | 2.84 | 1.0000 | 2/2 | - |
| 13-math-display-rich | xelatex | main | 1/1 | recovered | 0.4767 | 0.9899 | (0,3.5) moderate | 0.4525 | 0.9906 | 255 | 0.0041 | 0.0032 | 21/17/9 | no | 1.53 | 3.43 | 1.0000 | 2/0 | - |
| 13-math-display-rich | xelatex | pipeline | 1/1 | recovered | 0.5734 | 0.9875 | (1.5,1.5) moderate | 0.5445 | 0.9882 | 255 | 0.0045 | 0.0037 | 21/33/13 | no | 12.97 | 2.97 | 0.7692 | 2/2 | - |
| 13-math-display-rich | xelatex-lm | de1020c | 1/1 | recovered | 0.5517 | 0.9887 | (-2,4) weak | 0.5476 | 0.9892 | 255 | 0.0043 | 0.0035 | 21/17/8 | no | 3.20 | 3.05 | 1.0000 | 4/2 | - |
| 13-math-display-rich | xelatex-lm | main | 1/1 | recovered | 0.5522 | 0.9884 | (-0.5,3.5) weak | 0.5303 | 0.9890 | 255 | 0.0044 | 0.0036 | 21/17/9 | no | 3.64 | 3.61 | 1.0000 | 4/0 | - |
| 13-math-display-rich | xelatex-lm | pipeline | 1/1 | recovered | 0.5466 | 0.9877 | (3.5,1.5) moderate | 0.4860 | 0.9895 | 255 | 0.0044 | 0.0036 | 21/33/13 | no | 11.48 | 3.13 | 0.7692 | 4/2 | - |
| 14-math-inline-dense | lualatex | de1020c | 1/1 | recovered | 1.3179 | 0.9705 | (-34.5,13.5) | 0.9604 | 0.9783 | 255 | 0.0099 | 0.0082 | 62/59/25 | no | 52.74 | 10.98 | 0.9600 | 2/1 | - |
| 14-math-inline-dense | lualatex | main | 1/1 | recovered | 1.3111 | 0.9706 | (-34.5,13.5) | 0.9460 | 0.9784 | 255 | 0.0099 | 0.0081 | 62/57/26 | no | 42.13 | 10.69 | 1.0000 | 2/0 | - |
| 14-math-inline-dense | lualatex | pipeline | 1/1 | recovered | 0.9935 | 0.9797 | (0,0) | 0.9935 | 0.9797 | 255 | 0.0083 | 0.0065 | 62/85/25 | no | 29.50 | 3.07 | 1.0000 | 2/2 | - |
| 14-math-inline-dense | lualatex-lm | de1020c | 1/1 | recovered | 1.2473 | 0.9702 | (5,13.5) moderate | 1.1675 | 0.9735 | 255 | 0.0097 | 0.0079 | 62/59/23 | no | 60.36 | 10.71 | 0.9565 | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | main | 1/1 | recovered | 1.2444 | 0.9703 | (5,13.5) moderate | 1.1626 | 0.9734 | 255 | 0.0096 | 0.0079 | 62/57/24 | no | 46.12 | 10.02 | 1.0000 | 2/0 | - |
| 14-math-inline-dense | lualatex-lm | pipeline | 1/1 | recovered | 0.9471 | 0.9801 | (0,1.5) moderate | 0.8319 | 0.9817 | 255 | 0.0080 | 0.0064 | 62/85/27 | no | 4.37 | 2.30 | 1.0000 | 2/2 | - |
| 14-math-inline-dense | pdflatex | de1020c | 1/1 | recovered | 1.3129 | 0.9708 | (-34,13.5) | 0.9608 | 0.9783 | 255 | 0.0099 | 0.0082 | 53/59/18 | no | 56.56 | 13.98 | 0.8889 | 2/1 | [p1](images/14-math-inline-dense/pdflatex-de1020c-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex | main | 1/1 | recovered | 1.3054 | 0.9708 | (-34,13.5) | 0.9332 | 0.9785 | 255 | 0.0099 | 0.0081 | 53/57/19 | no | 48.76 | 11.30 | 1.0000 | 2/0 | [p1](images/14-math-inline-dense/pdflatex-main-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex | pipeline | 1/1 | recovered | 0.9830 | 0.9799 | (0,0) | 0.9830 | 0.9799 | 255 | 0.0082 | 0.0064 | 53/85/21 | no | 24.56 | 2.58 | 1.0000 | 2/2 | [p1](images/14-math-inline-dense/pdflatex-pipeline-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | de1020c | 1/1 | recovered | 1.2469 | 0.9702 | (5,13.5) moderate | 1.1673 | 0.9735 | 255 | 0.0097 | 0.0079 | 55/59/19 | no | 64.42 | 13.17 | 0.8947 | 2/1 | [p1](images/14-math-inline-dense/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | main | 1/1 | recovered | 1.2440 | 0.9703 | (5,13.5) moderate | 1.1624 | 0.9734 | 255 | 0.0096 | 0.0079 | 55/57/19 | no | 48.95 | 10.98 | 1.0000 | 2/0 | [p1](images/14-math-inline-dense/pdflatex-lm-main-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | pipeline | 1/1 | recovered | 0.9478 | 0.9801 | (0,1.5) moderate | 0.8320 | 0.9817 | 255 | 0.0080 | 0.0064 | 55/85/24 | no | 13.39 | 3.13 | 1.0000 | 2/2 | [p1](images/14-math-inline-dense/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 14-math-inline-dense | xelatex | de1020c | 1/1 | recovered | 1.3179 | 0.9705 | (-34.5,13.5) | 0.9575 | 0.9783 | 255 | 0.0099 | 0.0082 | 63/59/25 | no | 52.75 | 12.18 | 0.9600 | 2/1 | - |
| 14-math-inline-dense | xelatex | main | 1/1 | recovered | 1.3112 | 0.9706 | (-34.5,13.5) | 0.9422 | 0.9785 | 255 | 0.0099 | 0.0081 | 63/57/26 | no | 42.13 | 11.84 | 1.0000 | 2/0 | - |
| 14-math-inline-dense | xelatex | pipeline | 1/1 | recovered | 0.9936 | 0.9797 | (0,0) | 0.9936 | 0.9797 | 255 | 0.0083 | 0.0065 | 63/85/26 | no | 28.67 | 6.45 | 1.0000 | 2/2 | - |
| 14-math-inline-dense | xelatex-lm | de1020c | 1/1 | recovered | 1.2473 | 0.9702 | (5,13.5) moderate | 1.1675 | 0.9735 | 255 | 0.0097 | 0.0079 | 63/59/23 | no | 60.36 | 11.58 | 0.9565 | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | main | 1/1 | recovered | 1.2444 | 0.9703 | (5,13.5) moderate | 1.1627 | 0.9734 | 255 | 0.0096 | 0.0079 | 63/57/24 | no | 46.12 | 11.02 | 1.0000 | 2/0 | - |
| 14-math-inline-dense | xelatex-lm | pipeline | 1/1 | recovered | 0.9471 | 0.9801 | (0,1.5) moderate | 0.8319 | 0.9817 | 255 | 0.0080 | 0.0064 | 63/85/28 | no | 4.40 | 5.65 | 1.0000 | 2/2 | - |
| 15-three-page-sections | lualatex | de1020c | 3/3 | recovered | 26.4239 | 0.5635 | (0.5,20) moderate; (0.5,19) weak; (0.5,-24.5) moderate | 25.2158 | 0.5876 | 255 | 0.1894 | 0.1584 | 1806/1806/1806 | yes | 154.50 | 39.85 | 0.8984 | 0/0 | - |
| 15-three-page-sections | lualatex | main | 3/3 | recovered | 26.2989 | 0.5653 | (0,20) moderate; (0,19) weak; (0,-24.5) weak | 25.1064 | 0.5895 | 255 | 0.1894 | 0.1581 | 1806/1809/1806 | no | 132.74 | 39.82 | 0.8978 | 0/0 | - |
| 15-three-page-sections | lualatex | pipeline | 3/3 | recovered | 21.9934 | 0.6558 | (0,14) weak; (0,14) weak; (0,14) weak | 21.0801 | 0.6751 | 255 | 0.1659 | 0.1373 | 1806/1809/1806 | no | 179.49 | 27.32 | 0.8987 | 0/0 | - |
| 15-three-page-sections | lualatex-lm | de1020c | 3/3 | recovered | 24.2612 | 0.5493 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0398 | 0.5821 | 255 | 0.1832 | 0.1520 | 1806/1806/1806 | yes | 122.99 | 44.85 | 0.8926 | 3/0 | - |
| 15-three-page-sections | lualatex-lm | main | 3/3 | recovered | 24.2777 | 0.5494 | (-0.5,22.5) moderate; (-0.5,-24) weak; (0,-24) weak | 22.9599 | 0.5839 | 255 | 0.1836 | 0.1521 | 1806/1809/1806 | no | 146.92 | 45.04 | 0.8919 | 3/0 | - |
| 15-three-page-sections | lualatex-lm | pipeline | 3/3 | recovered | 13.1332 | 0.8176 | (0,0); (0,0); (0,0) | 13.1332 | 0.8176 | 255 | 0.1288 | 0.0972 | 1806/1809/1806 | no | 0.10 | 0.37 | 0.9983 | 3/0 | - |
| 15-three-page-sections | pdflatex | de1020c | 3/3 | recovered | 26.6994 | 0.5541 | (0.5,36.5) moderate; (0,-24.5) moderate; (0,-24.5) weak | 25.0916 | 0.5919 | 255 | 0.1902 | 0.1599 | 1806/1806/1806 | yes | 136.77 | 43.12 | 0.8914 | 0/0 | [p1](images/15-three-page-sections/pdflatex-de1020c-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex | main | 3/3 | recovered | 26.7472 | 0.5539 | (0,36.5) moderate; (0,-24.5) moderate; (0,-24.5) moderate | 25.0604 | 0.5936 | 255 | 0.1907 | 0.1602 | 1806/1809/1806 | no | 109.74 | 43.31 | 0.8908 | 0/0 | [p1](images/15-three-page-sections/pdflatex-main-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex | pipeline | 3/3 | recovered | 21.7473 | 0.6642 | (1,-0.5) moderate; (1,-0.5) moderate; (1,-0.5) moderate | 20.5566 | 0.6872 | 255 | 0.1646 | 0.1363 | 1806/1809/1806 | no | 184.33 | 5.13 | 0.8920 | 0/0 | [p1](images/15-three-page-sections/pdflatex-pipeline-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | de1020c | 3/3 | recovered | 24.2615 | 0.5493 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0362 | 0.5822 | 255 | 0.1833 | 0.1521 | 1806/1806/1806 | yes | 123.00 | 44.54 | 0.8926 | 3/0 | [p1](images/15-three-page-sections/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | main | 3/3 | recovered | 24.2787 | 0.5493 | (-0.5,22.5) moderate; (-0.5,-24) weak; (-0.5,-24) weak | 22.9548 | 0.5839 | 255 | 0.1836 | 0.1522 | 1806/1809/1806 | no | 146.92 | 44.73 | 0.8919 | 3/0 | [p1](images/15-three-page-sections/pdflatex-lm-main-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | pipeline | 3/3 | recovered | 13.1519 | 0.8174 | (0,0); (0,0); (0,0) | 13.1519 | 0.8174 | 255 | 0.1290 | 0.0974 | 1806/1809/1806 | no | 0.11 | 0.67 | 0.9983 | 3/0 | [p1](images/15-three-page-sections/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 15-three-page-sections | xelatex | de1020c | 3/3 | recovered | 26.4202 | 0.5637 | (-3,20) moderate; (-3,19) weak; (-3,-24.5) moderate | 25.1892 | 0.5879 | 255 | 0.1893 | 0.1587 | 1806/1806/1806 | yes | 154.49 | 39.85 | 0.8984 | 0/0 | - |
| 15-three-page-sections | xelatex | main | 3/3 | recovered | 26.3412 | 0.5649 | (0,20) moderate; (0,19) weak; (0,-24.5) weak | 25.1635 | 0.5888 | 255 | 0.1895 | 0.1586 | 1806/1809/1806 | no | 132.72 | 39.82 | 0.8978 | 0/0 | - |
| 15-three-page-sections | xelatex | pipeline | 3/3 | recovered | 22.0502 | 0.6548 | (0,14) weak; (1,14) weak; (1,14) weak | 21.0103 | 0.6756 | 255 | 0.1662 | 0.1378 | 1806/1809/1806 | no | 179.48 | 27.32 | 0.8987 | 0/0 | - |
| 15-three-page-sections | xelatex-lm | de1020c | 3/3 | recovered | 24.2611 | 0.5493 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0397 | 0.5821 | 255 | 0.1832 | 0.1519 | 1806/1806/1806 | yes | 122.99 | 44.55 | 0.8926 | 3/0 | - |
| 15-three-page-sections | xelatex-lm | main | 3/3 | recovered | 24.2775 | 0.5494 | (-0.5,22.5) moderate; (-0.5,-24) weak; (0,-24) weak | 22.9598 | 0.5839 | 255 | 0.1836 | 0.1521 | 1806/1809/1806 | no | 146.92 | 44.73 | 0.8919 | 3/0 | - |
| 15-three-page-sections | xelatex-lm | pipeline | 3/3 | recovered | 13.1371 | 0.8175 | (0,0); (0,0); (0,0) | 13.1371 | 0.8175 | 255 | 0.1288 | 0.0972 | 1806/1809/1806 | no | 0.10 | 0.66 | 0.9983 | 3/0 | - |
| 16-heading-page-break | lualatex | de1020c | 2/2 | ok | 20.5835 | 0.6612 | (0,0); (0,20.5) moderate | 20.0992 | 0.6687 | 255 | 0.1484 | 0.1240 | 962/962/962 | yes | 161.32 | 45.97 | 0.8950 | 0/0 | - |
| 16-heading-page-break | lualatex | main | 2/2 | ok | 20.1428 | 0.6738 | (0,0); (0,14.5) weak | 20.0614 | 0.6772 | 255 | 0.1461 | 0.1217 | 962/963/962 | no | 135.98 | 15.96 | 0.9002 | 0/0 | - |
| 16-heading-page-break | lualatex | pipeline | 2/2 | ok | 17.5099 | 0.7211 | (0,0); (-1,57.5) weak | 17.3770 | 0.7259 | 255 | 0.1328 | 0.1087 | 962/963/962 | no | 162.19 | 36.53 | 0.8949 | 0/0 | - |
| 16-heading-page-break | lualatex-lm | de1020c | 2/2 | ok | 19.0181 | 0.6506 | (-1,0) weak; (-1,20) moderate | 18.0483 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/962/962 | yes | 137.29 | 13.93 | 0.8888 | 1/0 | - |
| 16-heading-page-break | lualatex-lm | main | 2/2 | ok | 18.3183 | 0.6671 | (-0.5,-27) weak; (0,0) | 18.3034 | 0.6668 | 255 | 0.1402 | 0.1159 | 962/963/962 | no | 156.23 | 21.71 | 0.8951 | 1/0 | - |
| 16-heading-page-break | lualatex-lm | pipeline | 2/2 | ok | 10.5761 | 0.8525 | (0,0); (0,0) | 10.5761 | 0.8525 | 255 | 0.1036 | 0.0780 | 962/963/962 | no | 0.07 | 0.36 | 0.9990 | 1/0 | - |
| 16-heading-page-break | pdflatex | de1020c | 2/2 | ok | 20.5977 | 0.6620 | (0,-14.5) weak; (0,20.5) moderate | 20.0449 | 0.6704 | 255 | 0.1481 | 0.1240 | 962/962/962 | yes | 141.32 | 45.21 | 0.8952 | 0/0 | [p1](images/16-heading-page-break/pdflatex-de1020c-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex | main | 2/2 | ok | 20.1472 | 0.6751 | (0,0); (-0.5,14.5) weak | 20.0432 | 0.6786 | 255 | 0.1457 | 0.1216 | 962/963/962 | no | 111.12 | 15.21 | 0.9003 | 0/0 | [p1](images/16-heading-page-break/pdflatex-main-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex | pipeline | 2/2 | ok | 17.4999 | 0.7228 | (-1.5,0) weak; (-1.5,57.5) weak | 17.2538 | 0.7296 | 255 | 0.1324 | 0.1086 | 962/963/962 | no | 178.07 | 35.76 | 0.8950 | 0/0 | [p1](images/16-heading-page-break/pdflatex-pipeline-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | de1020c | 2/2 | ok | 19.0175 | 0.6505 | (-1,0) weak; (-1,20) moderate | 18.0489 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/962/962 | yes | 137.30 | 14.56 | 0.8888 | 1/0 | [p1](images/16-heading-page-break/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | main | 2/2 | ok | 18.3187 | 0.6670 | (-0.5,-27) weak; (0,0) | 18.3039 | 0.6667 | 255 | 0.1403 | 0.1159 | 962/963/962 | no | 156.24 | 20.84 | 0.8951 | 1/0 | [p1](images/16-heading-page-break/pdflatex-lm-main-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | pipeline | 2/2 | ok | 10.5954 | 0.8522 | (0,0); (0,0) | 10.5954 | 0.8522 | 255 | 0.1036 | 0.0782 | 962/963/962 | no | 0.07 | 0.67 | 0.9990 | 1/0 | [p1](images/16-heading-page-break/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 16-heading-page-break | xelatex | de1020c | 2/2 | ok | 20.6100 | 0.6610 | (-3,-14.5) weak; (0,20.5) moderate | 20.1333 | 0.6680 | 255 | 0.1485 | 0.1243 | 962/962/962 | yes | 161.33 | 45.97 | 0.8950 | 0/0 | - |
| 16-heading-page-break | xelatex | main | 2/2 | ok | 20.1391 | 0.6739 | (0,0); (0,14.5) weak | 20.0551 | 0.6774 | 255 | 0.1461 | 0.1220 | 962/963/962 | no | 135.98 | 15.96 | 0.9002 | 0/0 | - |
| 16-heading-page-break | xelatex | pipeline | 2/2 | ok | 17.5404 | 0.7206 | (1.5,0) weak; (-1,57.5) weak | 17.3597 | 0.7256 | 255 | 0.1329 | 0.1090 | 962/963/962 | no | 162.22 | 36.53 | 0.8949 | 0/0 | - |
| 16-heading-page-break | xelatex-lm | de1020c | 2/2 | ok | 19.0182 | 0.6506 | (-1,0) weak; (-1,20) moderate | 18.0480 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/962/962 | yes | 137.29 | 14.56 | 0.8888 | 1/0 | - |
| 16-heading-page-break | xelatex-lm | main | 2/2 | ok | 18.3184 | 0.6671 | (-0.5,-27) weak; (0,0) | 18.3034 | 0.6668 | 255 | 0.1402 | 0.1159 | 962/963/962 | no | 156.23 | 20.84 | 0.8951 | 1/0 | - |
| 16-heading-page-break | xelatex-lm | pipeline | 2/2 | ok | 10.5787 | 0.8525 | (0,0); (0,0) | 10.5787 | 0.8525 | 255 | 0.1035 | 0.0781 | 962/963/962 | no | 0.07 | 0.67 | 0.9990 | 1/0 | - |
| 17-apostrophes | lualatex | de1020c | 1/1 | ok | 0.8502 | 0.9875 | (-1.5,0) weak | 0.8119 | 0.9882 | 255 | 0.0068 | 0.0053 | 10/25/7 | no | 1.20 | 0.42 | 1.0000 | 0/0 | - |
| 17-apostrophes | lualatex | main | 1/1 | ok | 0.8494 | 0.9875 | (-7.5,0) weak | 0.8284 | 0.9876 | 255 | 0.0068 | 0.0053 | 10/25/7 | no | 1.27 | 0.42 | 1.0000 | 0/0 | - |
| 17-apostrophes | lualatex | pipeline | 1/1 | ok | 0.9397 | 0.9852 | (53.5,0) moderate | 0.8851 | 0.9860 | 255 | 0.0073 | 0.0058 | 10/25/9 | no | 56.38 | 0.41 | 0.8889 | 0/0 | - |
| 17-apostrophes | lualatex-lm | de1020c | 1/1 | ok | 0.8622 | 0.9848 | (0,0) | 0.8622 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.56 | 0.33 | 0.9231 | 0/0 | - |
| 17-apostrophes | lualatex-lm | main | 1/1 | ok | 0.8761 | 0.9846 | (0,0) | 0.8761 | 0.9846 | 255 | 0.0071 | 0.0057 | 25/25/13 | no | 44.76 | 0.33 | 0.9231 | 0/0 | - |
| 17-apostrophes | lualatex-lm | pipeline | 1/1 | ok | 0.6750 | 0.9896 | (-0.5,0) moderate | 0.6315 | 0.9900 | 255 | 0.0061 | 0.0047 | 25/25/25 | yes | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 17-apostrophes | pdflatex | de1020c | 1/1 | ok | 0.8063 | 0.9887 | (0,0) | 0.8063 | 0.9887 | 255 | 0.0066 | 0.0052 | 25/25/13 | no | 2.67 | 0.44 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-de1020c-export-p1-overlay.png) |
| 17-apostrophes | pdflatex | main | 1/1 | ok | 0.8138 | 0.9885 | (0,0) | 0.8138 | 0.9885 | 255 | 0.0067 | 0.0053 | 25/25/13 | no | 2.86 | 0.44 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-main-export-p1-overlay.png) |
| 17-apostrophes | pdflatex | pipeline | 1/1 | ok | 0.9356 | 0.9856 | (0,0) | 0.9356 | 0.9856 | 255 | 0.0072 | 0.0058 | 25/25/25 | yes | 51.51 | 0.99 | 0.9200 | 0/0 | [p1](images/17-apostrophes/pdflatex-pipeline-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | de1020c | 1/1 | ok | 0.8624 | 0.9848 | (0,0) | 0.8624 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.55 | 0.70 | 0.9231 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | main | 1/1 | ok | 0.8762 | 0.9846 | (0,0) | 0.8762 | 0.9846 | 255 | 0.0071 | 0.0057 | 25/25/13 | no | 44.76 | 0.70 | 0.9231 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-main-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | pipeline | 1/1 | ok | 0.6744 | 0.9896 | (-0.5,0) moderate | 0.6313 | 0.9900 | 255 | 0.0061 | 0.0047 | 25/25/25 | yes | 0.02 | 0.67 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 17-apostrophes | xelatex | de1020c | 1/1 | ok | 0.7981 | 0.9883 | (0,0) | 0.7981 | 0.9883 | 255 | 0.0066 | 0.0051 | 25/25/13 | no | 2.69 | 0.44 | 1.0000 | 0/0 | - |
| 17-apostrophes | xelatex | main | 1/1 | ok | 0.8069 | 0.9882 | (0,0) | 0.8069 | 0.9882 | 255 | 0.0067 | 0.0051 | 25/25/13 | no | 2.87 | 0.44 | 1.0000 | 0/0 | - |
| 17-apostrophes | xelatex | pipeline | 1/1 | ok | 0.9402 | 0.9852 | (0,0) | 0.9402 | 0.9852 | 255 | 0.0073 | 0.0058 | 25/25/25 | yes | 51.51 | 0.99 | 0.9200 | 0/0 | - |
| 17-apostrophes | xelatex-lm | de1020c | 1/1 | ok | 0.8620 | 0.9848 | (0,0) | 0.8620 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.56 | 0.70 | 0.9231 | 0/0 | - |
| 17-apostrophes | xelatex-lm | main | 1/1 | ok | 0.8760 | 0.9846 | (0,0) | 0.8760 | 0.9846 | 255 | 0.0071 | 0.0057 | 25/25/13 | no | 44.76 | 0.70 | 0.9231 | 0/0 | - |
| 17-apostrophes | xelatex-lm | pipeline | 1/1 | ok | 0.6748 | 0.9896 | (-0.5,0) moderate | 0.6310 | 0.9900 | 255 | 0.0061 | 0.0047 | 25/25/25 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex | de1020c | 1/1 | ok | 1.3714 | 0.9790 | (0,0) | 1.3714 | 0.9790 | 255 | 0.0109 | 0.0087 | 36/36/36 | yes | 41.08 | 1.23 | 0.8889 | 0/0 | - |
| 18-ligatures | lualatex | main | 1/1 | ok | 1.0519 | 0.9851 | (0,0) | 1.0519 | 0.9851 | 255 | 0.0097 | 0.0071 | 36/35/34 | no | 4.33 | 0.43 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex | pipeline | 1/1 | ok | 1.3969 | 0.9783 | (0,0) | 1.3969 | 0.9783 | 255 | 0.0111 | 0.0087 | 36/36/36 | yes | 50.01 | 1.21 | 0.8889 | 0/0 | - |
| 18-ligatures | lualatex-lm | de1020c | 1/1 | ok | 1.3134 | 0.9787 | (0,0) | 1.3134 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.13 | 0.34 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex-lm | main | 1/1 | ok | 1.4225 | 0.9764 | (-1,0) weak | 1.3902 | 0.9769 | 255 | 0.0111 | 0.0092 | 36/35/34 | no | 55.14 | 1.19 | 0.8824 | 0/0 | - |
| 18-ligatures | lualatex-lm | pipeline | 1/1 | ok | 0.9754 | 0.9860 | (0,0) | 0.9754 | 0.9860 | 255 | 0.0092 | 0.0070 | 36/36/36 | yes | 0.21 | 0.36 | 1.0000 | 0/0 | - |
| 18-ligatures | pdflatex | de1020c | 1/1 | ok | 1.2894 | 0.9808 | (0,0) | 1.2894 | 0.9808 | 255 | 0.0105 | 0.0082 | 36/36/36 | yes | 40.70 | 1.23 | 0.8889 | 0/0 | [p1](images/18-ligatures/pdflatex-de1020c-export-p1-overlay.png) |
| 18-ligatures | pdflatex | main | 1/1 | ok | 1.3805 | 0.9805 | (-1.5,0) moderate | 1.2545 | 0.9824 | 255 | 0.0107 | 0.0085 | 36/35/34 | no | 5.04 | 0.43 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-main-export-p1-overlay.png) |
| 18-ligatures | pdflatex | pipeline | 1/1 | ok | 1.3694 | 0.9794 | (0,0) | 1.3694 | 0.9794 | 255 | 0.0108 | 0.0085 | 36/36/36 | yes | 49.41 | 1.21 | 0.8889 | 0/0 | [p1](images/18-ligatures/pdflatex-pipeline-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | de1020c | 1/1 | ok | 1.3148 | 0.9787 | (0,0) | 1.3148 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.14 | 0.69 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | main | 1/1 | ok | 1.4220 | 0.9764 | (-1,0) weak | 1.3906 | 0.9769 | 255 | 0.0111 | 0.0092 | 36/35/34 | no | 55.15 | 1.46 | 0.8824 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-main-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | pipeline | 1/1 | ok | 0.9763 | 0.9860 | (0,0) | 0.9763 | 0.9860 | 255 | 0.0092 | 0.0070 | 36/36/36 | yes | 0.22 | 0.67 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 18-ligatures | xelatex | de1020c | 1/1 | ok | 1.3498 | 0.9797 | (0,0) | 1.3498 | 0.9797 | 255 | 0.0108 | 0.0087 | 36/36/36 | yes | 40.84 | 1.23 | 0.8889 | 0/0 | - |
| 18-ligatures | xelatex | main | 1/1 | ok | 1.2336 | 0.9826 | (-0.5,0) moderate | 1.1599 | 0.9838 | 255 | 0.0103 | 0.0081 | 36/35/34 | no | 4.58 | 0.43 | 1.0000 | 0/0 | - |
| 18-ligatures | xelatex | pipeline | 1/1 | ok | 1.3851 | 0.9786 | (0,0) | 1.3851 | 0.9786 | 255 | 0.0111 | 0.0088 | 36/36/36 | yes | 49.75 | 1.21 | 0.8889 | 0/0 | - |
| 18-ligatures | xelatex-lm | de1020c | 1/1 | ok | 1.3133 | 0.9787 | (0,0) | 1.3133 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.13 | 0.69 | 1.0000 | 0/0 | - |
| 18-ligatures | xelatex-lm | main | 1/1 | ok | 1.4224 | 0.9764 | (-1,0) weak | 1.3902 | 0.9769 | 255 | 0.0111 | 0.0092 | 36/35/34 | no | 55.14 | 1.46 | 0.8824 | 0/0 | - |
| 18-ligatures | xelatex-lm | pipeline | 1/1 | ok | 0.9752 | 0.9860 | (0,0) | 0.9752 | 0.9860 | 255 | 0.0092 | 0.0070 | 36/36/36 | yes | 0.21 | 0.67 | 1.0000 | 0/0 | - |

## Diagnostic: preview-equivalent comparison (CoreText draw of compile_result vs reference PDF raster)

Weaker than a capture of the real preview: it re-implements the app's draw code path rather than exercising the SwiftUI Canvas. Word-box metrics are not available for this side (no PDF), so they are omitted.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| raw | SSIM₈ raw | registration Δ pt (dx,dy per page; `weak`/`moderate` = shift explains <25% of the error) | mean\|Δ\| after reg | SSIM₈ after reg | max | differing | ≥thr | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-plain-paragraph | lualatex | de1020c | 1/1 | ok | 0.1686 | 0.9978 | (0,0) | 0.1686 | 0.9978 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.2718 | 0.9963 | (-0.5,0) | 0.1560 | 0.9982 | 255 | 0.0027 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | pipeline | 1/1 | ok | 0.3905 | 0.9942 | (0.5,0) weak | 0.3836 | 0.9943 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | de1020c | 1/1 | ok | 0.3642 | 0.9942 | (0,0) | 0.3642 | 0.9942 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | main | 1/1 | ok | 0.3577 | 0.9943 | (0,0) | 0.3577 | 0.9943 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | pipeline | 1/1 | ok | 0.2320 | 0.9967 | (0,0) | 0.2320 | 0.9967 | 255 | 0.0026 | 0.0018 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.1741 | 0.9977 | (0,0) | 0.1741 | 0.9977 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.1143 | 0.9987 | (0,0) | 0.1143 | 0.9987 | 255 | 0.0021 | 0.0011 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | pipeline | 1/1 | ok | 0.3868 | 0.9945 | (0,0) | 0.3868 | 0.9945 | 255 | 0.0031 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 0.3639 | 0.9942 | (0,0) | 0.3639 | 0.9942 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | main | 1/1 | ok | 0.3573 | 0.9943 | (0,0) | 0.3573 | 0.9943 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 0.2332 | 0.9966 | (0,0) | 0.2332 | 0.9966 | 255 | 0.0026 | 0.0018 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.1716 | 0.9978 | (0,0) | 0.1716 | 0.9978 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.2772 | 0.9962 | (-0.5,0) | 0.1595 | 0.9981 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | pipeline | 1/1 | ok | 0.3905 | 0.9942 | (0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0031 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | de1020c | 1/1 | ok | 0.3644 | 0.9942 | (0,0) | 0.3644 | 0.9942 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | main | 1/1 | ok | 0.3579 | 0.9943 | (0,0) | 0.3579 | 0.9943 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | pipeline | 1/1 | ok | 0.2320 | 0.9967 | (0,0) | 0.2320 | 0.9967 | 255 | 0.0026 | 0.0018 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 8.3831 | 0.8656 | (-3,0) weak | 8.3728 | 0.8658 | 255 | 0.0613 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 8.4139 | 0.8658 | (0,0) | 8.4139 | 0.8658 | 255 | 0.0616 | 0.0511 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | pipeline | 1/1 | ok | 7.1754 | 0.8902 | (0,0) | 7.1754 | 0.8902 | 255 | 0.0554 | 0.0449 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | de1020c | 1/1 | ok | 7.7865 | 0.8620 | (-0.5,-14.5) weak | 7.7677 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | main | 1/1 | ok | 7.7929 | 0.8614 | (0,0) | 7.7929 | 0.8614 | 255 | 0.0598 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | pipeline | 1/1 | ok | 4.5417 | 0.9367 | (0,0) | 4.5417 | 0.9367 | 255 | 0.0447 | 0.0336 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 8.3720 | 0.8664 | (0,0) | 8.3720 | 0.8664 | 255 | 0.0611 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 8.4698 | 0.8655 | (-3.5,0) weak | 8.4134 | 0.8661 | 255 | 0.0615 | 0.0513 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | pipeline | 1/1 | ok | 7.1612 | 0.8910 | (1,0) weak | 7.0823 | 0.8919 | 255 | 0.0553 | 0.0448 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 7.7859 | 0.8621 | (-0.5,-14.5) weak | 7.7684 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | main | 1/1 | ok | 7.7930 | 0.8614 | (0,0) | 7.7930 | 0.8614 | 255 | 0.0598 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 4.5499 | 0.9366 | (0,0) | 4.5499 | 0.9366 | 255 | 0.0447 | 0.0337 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 8.3703 | 0.8657 | (0,0) | 8.3703 | 0.8657 | 255 | 0.0613 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 8.4639 | 0.8648 | (-0.5,0) weak | 8.4582 | 0.8649 | 255 | 0.0618 | 0.0514 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | pipeline | 1/1 | ok | 7.1991 | 0.8902 | (0.5,0) weak | 7.1660 | 0.8904 | 255 | 0.0556 | 0.0451 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | de1020c | 1/1 | ok | 7.7862 | 0.8620 | (-0.5,-14.5) weak | 7.7678 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | main | 1/1 | ok | 7.7928 | 0.8614 | (0,0) | 7.7928 | 0.8614 | 255 | 0.0598 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | pipeline | 1/1 | ok | 4.5425 | 0.9367 | (0,0) | 4.5425 | 0.9367 | 255 | 0.0447 | 0.0336 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.3394 | 0.9793 | (0,52.5) moderate | 1.1862 | 0.9829 | 255 | 0.0085 | 0.0073 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.3597 | 0.9789 | (0,52.5) moderate | 1.2063 | 0.9826 | 255 | 0.0085 | 0.0074 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | pipeline | 1/1 | ok | 1.0037 | 0.9865 | (0,-1.5) weak | 0.9815 | 0.9874 | 255 | 0.0069 | 0.0058 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | de1020c | 1/1 | ok | 1.2336 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | main | 1/1 | ok | 1.2425 | 0.9769 | (0,53.5) moderate | 1.1129 | 0.9812 | 255 | 0.0082 | 0.0070 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | pipeline | 1/1 | ok | 0.8538 | 0.9887 | (0,0) | 0.8538 | 0.9887 | 255 | 0.0062 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.3496 | 0.9793 | (0.5,52.5) moderate | 1.2021 | 0.9827 | 255 | 0.0085 | 0.0072 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.3719 | 0.9789 | (0.5,52.5) moderate | 1.2200 | 0.9824 | 255 | 0.0086 | 0.0074 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | pipeline | 1/1 | ok | 1.0317 | 0.9862 | (0.5,-2) weak | 1.0232 | 0.9859 | 255 | 0.0070 | 0.0059 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | de1020c | 1/1 | ok | 1.2335 | 0.9771 | (0,54) moderate | 1.0954 | 0.9811 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | main | 1/1 | ok | 1.2428 | 0.9769 | (0,54) moderate | 1.1208 | 0.9807 | 255 | 0.0082 | 0.0070 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | pipeline | 1/1 | ok | 0.8534 | 0.9887 | (0,0) | 0.8534 | 0.9887 | 255 | 0.0062 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.3370 | 0.9793 | (0,52.5) moderate | 1.1869 | 0.9829 | 255 | 0.0085 | 0.0073 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.3601 | 0.9789 | (0,52.5) moderate | 1.2075 | 0.9826 | 255 | 0.0086 | 0.0075 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | pipeline | 1/1 | ok | 1.0089 | 0.9864 | (0,-1.5) weak | 0.9852 | 0.9874 | 255 | 0.0070 | 0.0059 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | de1020c | 1/1 | ok | 1.2337 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | main | 1/1 | ok | 1.2424 | 0.9769 | (0,53.5) moderate | 1.1129 | 0.9812 | 255 | 0.0082 | 0.0070 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | pipeline | 1/1 | ok | 0.8537 | 0.9887 | (0,0) | 0.8537 | 0.9887 | 255 | 0.0062 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.3808 | 0.9945 | (0,0) | 0.3808 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.3769 | 0.9945 | (0,0) | 0.3769 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | pipeline | 1/1 | ok | 0.4278 | 0.9938 | (0,0) | 0.4278 | 0.9938 | 255 | 0.0033 | 0.0026 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | de1020c | 1/1 | ok | 0.4244 | 0.9933 | (-49,0) weak | 0.4148 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | main | 1/1 | ok | 0.4253 | 0.9933 | (-49,0) weak | 0.4146 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | pipeline | 1/1 | ok | 0.3473 | 0.9949 | (-0.5,0) weak | 0.3304 | 0.9952 | 255 | 0.0030 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.3837 | 0.9944 | (-0.5,0) weak | 0.3764 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.3805 | 0.9944 | (-0.5,0) weak | 0.3776 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | pipeline | 1/1 | ok | 0.4168 | 0.9939 | (0,0) | 0.4168 | 0.9939 | 255 | 0.0033 | 0.0026 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | de1020c | 1/1 | ok | 0.4246 | 0.9933 | (-49,0) weak | 0.4162 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | main | 1/1 | ok | 0.4257 | 0.9933 | (-49,0) weak | 0.4164 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | pipeline | 1/1 | ok | 0.3564 | 0.9948 | (-0.5,0) moderate | 0.3295 | 0.9953 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.3849 | 0.9944 | (-0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0032 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.3811 | 0.9945 | (-0.5,0) weak | 0.3801 | 0.9944 | 255 | 0.0031 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | pipeline | 1/1 | ok | 0.4274 | 0.9938 | (31.5,0) weak | 0.4233 | 0.9941 | 255 | 0.0033 | 0.0026 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | de1020c | 1/1 | ok | 0.4245 | 0.9933 | (-49,0) weak | 0.4138 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | main | 1/1 | ok | 0.4237 | 0.9933 | (-49,0) weak | 0.4127 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | pipeline | 1/1 | ok | 0.3308 | 0.9952 | (0,0) | 0.3308 | 0.9952 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.2971 | 0.9955 | (0,0) | 0.2971 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.2490 | 0.9963 | (0,0) | 0.2490 | 0.9963 | 255 | 0.0026 | 0.0018 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | pipeline | 1/1 | ok | 0.3579 | 0.9948 | (11.5,0) moderate | 0.3267 | 0.9952 | 255 | 0.0029 | 0.0022 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex-lm | de1020c | 1/1 | ok | 0.3619 | 0.9938 | (2.5,0) weak | 0.3494 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | main | 1/1 | ok | 0.3628 | 0.9938 | (-3.5,0) moderate | 0.3432 | 0.9941 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | pipeline | 1/1 | ok | 0.2636 | 0.9960 | (-0.5,0) moderate | 0.2087 | 0.9969 | 255 | 0.0024 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.3125 | 0.9955 | (0,0) | 0.3125 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.2341 | 0.9967 | (0,0) | 0.2341 | 0.9967 | 255 | 0.0025 | 0.0016 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex | pipeline | 1/1 | ok | 0.3571 | 0.9950 | (11.5,0) moderate | 0.3060 | 0.9957 | 255 | 0.0029 | 0.0022 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex-lm | de1020c | 1/1 | ok | 0.3618 | 0.9938 | (2.5,0) weak | 0.3496 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex-lm | main | 1/1 | ok | 0.3625 | 0.9938 | (-3.5,0) moderate | 0.3430 | 0.9941 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex-lm | pipeline | 1/1 | ok | 0.2655 | 0.9960 | (-0.5,0) moderate | 0.2082 | 0.9969 | 255 | 0.0024 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.2963 | 0.9955 | (0,0) | 0.2963 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.2493 | 0.9963 | (0,0) | 0.2493 | 0.9963 | 255 | 0.0026 | 0.0018 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | pipeline | 1/1 | ok | 0.3577 | 0.9948 | (11.5,0) moderate | 0.3296 | 0.9951 | 255 | 0.0029 | 0.0022 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex-lm | de1020c | 1/1 | ok | 0.3620 | 0.9938 | (2.5,0) weak | 0.3493 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | main | 1/1 | ok | 0.3628 | 0.9938 | (-3.5,0) moderate | 0.3432 | 0.9941 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | pipeline | 1/1 | ok | 0.2638 | 0.9960 | (-0.5,0) moderate | 0.2088 | 0.9969 | 255 | 0.0024 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.3823 | 0.9934 | (0,3.5) | 0.2551 | 0.9959 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | ok | 0.3834 | 0.9933 | (-6.5,3.5) | 0.2030 | 0.9966 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | pipeline | 1/1 | ok | 0.3048 | 0.9947 | (0,0) | 0.3048 | 0.9947 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | de1020c | 1/1 | ok | 0.3683 | 0.9931 | (-32.5,3.5) moderate | 0.3139 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | main | 1/1 | ok | 0.3734 | 0.9929 | (9.5,3.5) moderate | 0.3123 | 0.9945 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | pipeline | 1/1 | ok | 0.3070 | 0.9947 | (-1,0) moderate | 0.2372 | 0.9962 | 255 | 0.0025 | 0.0020 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.3812 | 0.9936 | (0,3.5) | 0.2591 | 0.9960 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | main | 1/1 | ok | 0.3779 | 0.9935 | (-6,3.5) | 0.2148 | 0.9967 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | pipeline | 1/1 | ok | 0.2915 | 0.9950 | (16.5,0) weak | 0.2870 | 0.9953 | 255 | 0.0025 | 0.0018 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | de1020c | 1/1 | ok | 0.3681 | 0.9931 | (-32.5,3.5) moderate | 0.3137 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | main | 1/1 | ok | 0.3728 | 0.9929 | (9.5,3.5) moderate | 0.3118 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | pipeline | 1/1 | ok | 0.3074 | 0.9947 | (-1,0) moderate | 0.2389 | 0.9961 | 255 | 0.0025 | 0.0020 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.3823 | 0.9934 | (0,3.5) | 0.2545 | 0.9959 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | ok | 0.3835 | 0.9933 | (-6.5,3.5) | 0.2028 | 0.9966 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | pipeline | 1/1 | ok | 0.3054 | 0.9947 | (0,0) | 0.3054 | 0.9947 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | de1020c | 1/1 | ok | 0.3684 | 0.9931 | (-32.5,3.5) moderate | 0.3141 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | main | 1/1 | ok | 0.3734 | 0.9929 | (9.5,3.5) moderate | 0.3122 | 0.9945 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | pipeline | 1/1 | ok | 0.3071 | 0.9947 | (-1,0) moderate | 0.2372 | 0.9962 | 255 | 0.0025 | 0.0020 | -/-/- | - | - | - | - | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.2575 | 0.9950 | (0,0) | 0.2575 | 0.9950 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | ok | 0.2605 | 0.9947 | (0,0) | 0.2605 | 0.9947 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | lualatex | pipeline | 1/1 | ok | 0.3174 | 0.9943 | (0,0) | 0.3174 | 0.9943 | 255 | 0.0027 | 0.0021 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | de1020c | 1/1 | ok | 0.3485 | 0.9931 | (0,0) | 0.3485 | 0.9931 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | main | 1/1 | ok | 0.3517 | 0.9928 | (0,0) | 0.3517 | 0.9928 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | lualatex-lm | pipeline | 1/1 | ok | 0.3019 | 0.9945 | (-0.5,0) weak | 0.2946 | 0.9945 | 255 | 0.0026 | 0.0021 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.2593 | 0.9950 | (0,0) | 0.2593 | 0.9950 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex | main | 1/1 | ok | 0.2623 | 0.9946 | (0,0) | 0.2623 | 0.9946 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | pdflatex | pipeline | 1/1 | ok | 0.3122 | 0.9944 | (0,0) | 0.3122 | 0.9944 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex-lm | de1020c | 1/1 | ok | 0.3516 | 0.9931 | (0,0) | 0.3516 | 0.9931 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex-lm | main | 1/1 | ok | 0.3549 | 0.9927 | (0,0) | 0.3549 | 0.9927 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | pdflatex-lm | pipeline | 1/1 | ok | 0.3014 | 0.9945 | (-0.5,0) weak | 0.2933 | 0.9945 | 255 | 0.0027 | 0.0021 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.2573 | 0.9950 | (0,0) | 0.2573 | 0.9950 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | ok | 0.2603 | 0.9947 | (0,0) | 0.2603 | 0.9947 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | xelatex | pipeline | 1/1 | ok | 0.3171 | 0.9943 | (0,0) | 0.3171 | 0.9943 | 255 | 0.0027 | 0.0021 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | de1020c | 1/1 | ok | 0.3485 | 0.9931 | (0,0) | 0.3485 | 0.9931 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | main | 1/1 | ok | 0.3517 | 0.9928 | (0,0) | 0.3517 | 0.9928 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | xelatex-lm | pipeline | 1/1 | ok | 0.3019 | 0.9945 | (-0.5,0) weak | 0.2947 | 0.9945 | 255 | 0.0026 | 0.0021 | -/-/- | - | - | - | - | 3/1 | - |
| 08-two-page | lualatex | de1020c | 3/3 | ok | 26.2189 | 0.5659 | (0,0); (-3,6) moderate; (-3,34.5) moderate | 24.6331 | 0.5959 | 255 | 0.1880 | 0.1574 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex | main | 3/3 | ok | 25.2811 | 0.5915 | (0,0); (0,2) weak; (0,49) moderate | 24.7455 | 0.6039 | 255 | 0.1831 | 0.1528 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex | pipeline | 3/3 | ok | 21.1582 | 0.6699 | (0,0); (0,0); (1,0) weak | 21.1037 | 0.6709 | 255 | 0.1619 | 0.1323 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | de1020c | 3/3 | ok | 23.7956 | 0.5635 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7517 | 0.5869 | 255 | 0.1801 | 0.1495 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | main | 3/3 | ok | 23.2796 | 0.5779 | (-0.5,-27) weak; (-0.5,-12.5) weak; (-0.5,5.5) moderate | 22.8321 | 0.5860 | 255 | 0.1772 | 0.1468 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | pipeline | 3/3 | ok | 13.0043 | 0.8184 | (0,0); (0,0); (0,0) | 13.0043 | 0.8184 | 255 | 0.1280 | 0.0963 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/3 | ok | 26.1972 | 0.5669 | (0,-14.5) weak; (0,6) moderate; (0,34.5) moderate | 24.5512 | 0.5981 | 255 | 0.1875 | 0.1571 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | main | 3/3 | ok | 25.3258 | 0.5923 | (0,0); (-11,2) weak; (0,49) moderate | 24.7916 | 0.6035 | 255 | 0.1828 | 0.1528 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | pipeline | 3/3 | ok | 21.1714 | 0.6711 | (-1.5,0) weak; (1.5,0) weak; (-1.5,0) weak | 21.0218 | 0.6737 | 255 | 0.1614 | 0.1321 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | de1020c | 3/3 | ok | 23.7951 | 0.5635 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7523 | 0.5869 | 255 | 0.1800 | 0.1496 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | main | 3/3 | ok | 23.2802 | 0.5779 | (-0.5,-27) weak; (-0.5,2) weak; (-0.5,5.5) moderate | 22.8314 | 0.5887 | 255 | 0.1772 | 0.1468 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | pipeline | 3/3 | ok | 13.0322 | 0.8180 | (0,0); (0,0); (0,0) | 13.0322 | 0.8180 | 255 | 0.1281 | 0.0966 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | de1020c | 3/3 | ok | 26.2441 | 0.5658 | (-3,-14.5) weak; (-3,6) moderate; (-3,34.5) moderate | 24.6604 | 0.5954 | 255 | 0.1881 | 0.1577 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | main | 3/3 | ok | 25.2633 | 0.5919 | (0,0); (0,2) weak; (0,49) moderate | 24.7282 | 0.6042 | 255 | 0.1831 | 0.1530 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | pipeline | 3/3 | ok | 21.1760 | 0.6695 | (1.5,0) weak; (0,0); (1,0) weak | 21.1126 | 0.6705 | 255 | 0.1620 | 0.1326 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | de1020c | 3/3 | ok | 23.7957 | 0.5634 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7513 | 0.5869 | 255 | 0.1800 | 0.1495 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | main | 3/3 | ok | 23.2798 | 0.5779 | (-0.5,-27) weak; (-0.5,-12.5) weak; (-0.5,5.5) moderate | 22.8322 | 0.5859 | 255 | 0.1772 | 0.1468 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | pipeline | 3/3 | ok | 13.0077 | 0.8183 | (0,0); (0,0); (0,0) | 13.0077 | 0.8183 | 255 | 0.1280 | 0.0964 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.4844 | 0.9504 | (0,38.5) | 1.7906 | 0.9701 | 255 | 0.0174 | 0.0146 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | ok | 2.5088 | 0.9499 | (0,38.5) | 1.6561 | 0.9716 | 255 | 0.0175 | 0.0147 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | pipeline | 1/1 | ok | 1.7825 | 0.9736 | (0,0) | 1.7825 | 0.9736 | 255 | 0.0137 | 0.0112 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | de1020c | 1/1 | ok | 2.2538 | 0.9494 | (-0.5,39) moderate | 1.9068 | 0.9641 | 255 | 0.0169 | 0.0140 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | main | 1/1 | ok | 2.2680 | 0.9491 | (-0.5,39) moderate | 1.9052 | 0.9637 | 255 | 0.0169 | 0.0141 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | pipeline | 1/1 | ok | 1.3020 | 0.9812 | (0,0.5) weak | 1.2548 | 0.9821 | 255 | 0.0118 | 0.0093 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.4854 | 0.9504 | (-0.5,38.5) moderate | 1.8777 | 0.9688 | 255 | 0.0174 | 0.0146 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | main | 1/1 | ok | 2.5100 | 0.9498 | (0,38.5) | 1.8502 | 0.9691 | 255 | 0.0175 | 0.0147 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | pipeline | 1/1 | ok | 1.8296 | 0.9729 | (1,0) weak | 1.8130 | 0.9730 | 255 | 0.0138 | 0.0115 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | de1020c | 1/1 | ok | 2.2522 | 0.9480 | (-0.5,39.5) moderate | 1.9062 | 0.9622 | 255 | 0.0167 | 0.0139 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | main | 1/1 | ok | 2.2668 | 0.9477 | (-0.5,39.5) moderate | 1.9051 | 0.9619 | 255 | 0.0168 | 0.0139 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | pipeline | 1/1 | ok | 1.4719 | 0.9770 | (0,1) moderate | 1.3182 | 0.9792 | 255 | 0.0125 | 0.0101 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.4838 | 0.9503 | (-0.5,38.5) | 1.8423 | 0.9692 | 255 | 0.0174 | 0.0146 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | ok | 2.5096 | 0.9498 | (0,38.5) | 1.8171 | 0.9694 | 255 | 0.0175 | 0.0147 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | pipeline | 1/1 | ok | 1.7997 | 0.9733 | (1,0) weak | 1.7869 | 0.9733 | 255 | 0.0137 | 0.0113 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | de1020c | 1/1 | ok | 2.2538 | 0.9494 | (-0.5,39) moderate | 1.9067 | 0.9641 | 255 | 0.0169 | 0.0140 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | main | 1/1 | ok | 2.2680 | 0.9491 | (-0.5,39) moderate | 1.9052 | 0.9637 | 255 | 0.0169 | 0.0141 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | pipeline | 1/1 | ok | 1.3035 | 0.9812 | (0,0.5) weak | 1.2562 | 0.9821 | 255 | 0.0118 | 0.0093 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | lualatex | de1020c | 1/1 | ok | 3.3162 | 0.9491 | (0,0) | 3.3162 | 0.9491 | 255 | 0.0261 | 0.0212 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | main | 1/1 | recovered | 3.3339 | 0.9494 | (0.5,0) weak | 3.2483 | 0.9508 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex | pipeline | 1/1 | recovered | 3.3969 | 0.9471 | (0,0) | 3.3969 | 0.9471 | 255 | 0.0265 | 0.0214 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex-lm | de1020c | 1/1 | ok | 3.2405 | 0.9447 | (0,0) | 3.2405 | 0.9447 | 255 | 0.0259 | 0.0211 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | main | 1/1 | recovered | 3.2500 | 0.9449 | (-5,0) weak | 3.2164 | 0.9449 | 255 | 0.0259 | 0.0212 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 2.3128 | 0.9658 | (-0.5,0) moderate | 2.1851 | 0.9680 | 255 | 0.0218 | 0.0166 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | pdflatex | de1020c | 1/1 | ok | 3.3644 | 0.9486 | (0,0) | 3.3644 | 0.9486 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | pdflatex | main | 1/1 | recovered | 3.3245 | 0.9498 | (0,0) | 3.3245 | 0.9498 | 255 | 0.0261 | 0.0212 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | pdflatex | pipeline | 1/1 | recovered | 3.3533 | 0.9479 | (0,0) | 3.3533 | 0.9479 | 255 | 0.0263 | 0.0211 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 3.2558 | 0.9444 | (0,0) | 3.2558 | 0.9444 | 255 | 0.0260 | 0.0212 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | pdflatex-lm | main | 1/1 | recovered | 3.2657 | 0.9446 | (0,0) | 3.2657 | 0.9446 | 255 | 0.0260 | 0.0213 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 2.3799 | 0.9644 | (-0.5,0) moderate | 2.2159 | 0.9672 | 255 | 0.0221 | 0.0169 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | xelatex | de1020c | 1/1 | ok | 3.3626 | 0.9484 | (0,0) | 3.3626 | 0.9484 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | xelatex | main | 1/1 | recovered | 3.3247 | 0.9495 | (0.5,0) weak | 3.2367 | 0.9510 | 255 | 0.0261 | 0.0212 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | xelatex | pipeline | 1/1 | recovered | 3.4020 | 0.9470 | (0,0) | 3.4020 | 0.9470 | 255 | 0.0265 | 0.0214 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | xelatex-lm | de1020c | 1/1 | ok | 3.2406 | 0.9446 | (0,0) | 3.2406 | 0.9446 | 255 | 0.0259 | 0.0211 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | main | 1/1 | recovered | 3.2502 | 0.9449 | (-5,0) weak | 3.2164 | 0.9449 | 255 | 0.0259 | 0.0212 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 2.3134 | 0.9658 | (-0.5,0) moderate | 2.1851 | 0.9680 | 255 | 0.0218 | 0.0166 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | de1020c | 1/1 | recovered | 1.4652 | 0.9696 | (18,-49) weak | 1.4604 | 0.9709 | 255 | 0.0104 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | main | 1/1 | ok | 1.4714 | 0.9699 | (-22.5,-8) moderate | 1.1219 | 0.9800 | 255 | 0.0105 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | pipeline | 1/1 | ok | 1.4784 | 0.9703 | (-16.5,-20) moderate | 1.3615 | 0.9753 | 255 | 0.0104 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (-2,-49) weak | 1.3068 | 0.9711 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | main | 1/1 | ok | 1.3743 | 0.9689 | (-30.5,-8) moderate | 1.2508 | 0.9755 | 255 | 0.0102 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | pipeline | 1/1 | ok | 1.3303 | 0.9702 | (-17,-20) moderate | 1.1997 | 0.9761 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | de1020c | 1/1 | recovered | 1.4655 | 0.9698 | (19,-49) weak | 1.4558 | 0.9711 | 255 | 0.0104 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | main | 1/1 | ok | 1.4603 | 0.9704 | (-22,-8) moderate | 1.1820 | 0.9790 | 255 | 0.0104 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | pipeline | 1/1 | ok | 1.4478 | 0.9709 | (-16.5,-20) moderate | 1.3597 | 0.9754 | 255 | 0.0103 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | de1020c | 1/1 | recovered | 1.3674 | 0.9687 | (23.5,-49) weak | 1.3215 | 0.9706 | 255 | 0.0101 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | main | 1/1 | ok | 1.3761 | 0.9690 | (-30,-8) moderate | 1.2546 | 0.9752 | 255 | 0.0102 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | pipeline | 1/1 | ok | 1.3363 | 0.9701 | (-17,-20) moderate | 1.2000 | 0.9760 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | de1020c | 1/1 | recovered | 1.4648 | 0.9696 | (18,-49) weak | 1.4599 | 0.9709 | 255 | 0.0104 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | main | 1/1 | ok | 1.4708 | 0.9699 | (-22.5,-8) moderate | 1.1192 | 0.9800 | 255 | 0.0105 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | pipeline | 1/1 | ok | 1.4782 | 0.9703 | (-16.5,-20) moderate | 1.3611 | 0.9753 | 255 | 0.0104 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (1,-49) weak | 1.3138 | 0.9708 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | main | 1/1 | ok | 1.3743 | 0.9690 | (-30.5,-8) moderate | 1.2509 | 0.9755 | 255 | 0.0101 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | pipeline | 1/1 | ok | 1.3303 | 0.9702 | (-17.5,-20) moderate | 1.1888 | 0.9763 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | de1020c | 1/1 | ok | 15.4036 | 0.7397 | (-3,0) weak | 15.3867 | 0.7398 | 255 | 0.1111 | 0.0927 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | main | 1/1 | ok | 14.8426 | 0.7604 | (-3.5,0) weak | 14.8089 | 0.7605 | 255 | 0.1081 | 0.0901 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | pipeline | 1/1 | ok | 12.9693 | 0.7908 | (0,14.5) weak | 12.9445 | 0.7924 | 255 | 0.0988 | 0.0805 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | de1020c | 1/1 | ok | 13.7975 | 0.7504 | (-8.5,2.5) weak | 13.7722 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | main | 1/1 | ok | 13.7769 | 0.7470 | (-1,0) weak | 13.6905 | 0.7481 | 255 | 0.1054 | 0.0872 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | pipeline | 1/1 | ok | 7.8075 | 0.8916 | (0,0) | 7.8075 | 0.8916 | 255 | 0.0766 | 0.0577 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | de1020c | 1/1 | ok | 15.3703 | 0.7416 | (-3,0) weak | 15.3659 | 0.7415 | 255 | 0.1107 | 0.0925 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | main | 1/1 | ok | 14.8942 | 0.7603 | (-4,0) weak | 14.8307 | 0.7612 | 255 | 0.1079 | 0.0901 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | pipeline | 1/1 | ok | 13.0307 | 0.7911 | (1,14.5) weak | 12.9210 | 0.7936 | 255 | 0.0986 | 0.0807 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | de1020c | 1/1 | ok | 13.7973 | 0.7504 | (-8.5,2.5) weak | 13.7711 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | main | 1/1 | ok | 13.7775 | 0.7470 | (-0.5,0) weak | 13.6919 | 0.7482 | 255 | 0.1054 | 0.0872 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | pipeline | 1/1 | ok | 7.8248 | 0.8913 | (0,0) | 7.8248 | 0.8913 | 255 | 0.0767 | 0.0579 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | de1020c | 1/1 | ok | 15.3817 | 0.7405 | (0,0) | 15.3817 | 0.7405 | 255 | 0.1110 | 0.0928 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | main | 1/1 | ok | 14.9414 | 0.7591 | (0,0) | 14.9414 | 0.7591 | 255 | 0.1084 | 0.0906 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | pipeline | 1/1 | ok | 13.0500 | 0.7899 | (0.5,14.5) weak | 12.9803 | 0.7921 | 255 | 0.0990 | 0.0810 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | de1020c | 1/1 | ok | 13.7974 | 0.7504 | (-8.5,2.5) weak | 13.7718 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | main | 1/1 | ok | 13.7772 | 0.7470 | (-1,0) weak | 13.6904 | 0.7481 | 255 | 0.1054 | 0.0872 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | pipeline | 1/1 | ok | 7.8076 | 0.8916 | (0,0) | 7.8076 | 0.8916 | 255 | 0.0766 | 0.0577 | -/-/- | - | - | - | - | 0/0 | - |
| 13-math-display-rich | lualatex | de1020c | 1/1 | recovered | 0.4665 | 0.9904 | (0,3.5) moderate | 0.4400 | 0.9913 | 255 | 0.0040 | 0.0031 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex | main | 1/1 | recovered | 0.5037 | 0.9895 | (-0.5,3.5) weak | 0.4862 | 0.9904 | 255 | 0.0042 | 0.0033 | -/-/- | - | - | - | - | 2/0 | - |
| 13-math-display-rich | lualatex | pipeline | 1/1 | recovered | 0.5940 | 0.9876 | (1.5,1.5) weak | 0.5782 | 0.9881 | 255 | 0.0046 | 0.0037 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex-lm | de1020c | 1/1 | recovered | 0.5457 | 0.9888 | (0,0) | 0.5457 | 0.9888 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | lualatex-lm | main | 1/1 | recovered | 0.5568 | 0.9883 | (-0.5,3.5) weak | 0.5416 | 0.9888 | 255 | 0.0044 | 0.0036 | -/-/- | - | - | - | - | 4/0 | - |
| 13-math-display-rich | lualatex-lm | pipeline | 1/1 | recovered | 0.5702 | 0.9877 | (3.5,1.5) moderate | 0.5183 | 0.9892 | 255 | 0.0045 | 0.0037 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | pdflatex | de1020c | 1/1 | recovered | 0.4914 | 0.9901 | (1,3.5) weak | 0.4902 | 0.9905 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | pdflatex | main | 1/1 | recovered | 0.4829 | 0.9899 | (0,3.5) moderate | 0.4561 | 0.9908 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/0 | - |
| 13-math-display-rich | pdflatex | pipeline | 1/1 | recovered | 0.5965 | 0.9876 | (4,1.5) moderate | 0.5424 | 0.9886 | 255 | 0.0046 | 0.0037 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | pdflatex-lm | de1020c | 1/1 | recovered | 0.5476 | 0.9889 | (-2,4) weak | 0.5407 | 0.9892 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | pdflatex-lm | main | 1/1 | recovered | 0.5594 | 0.9883 | (-0.5,4) weak | 0.5475 | 0.9886 | 255 | 0.0044 | 0.0036 | -/-/- | - | - | - | - | 4/0 | - |
| 13-math-display-rich | pdflatex-lm | pipeline | 1/1 | recovered | 0.5719 | 0.9878 | (3.5,1.5) moderate | 0.5292 | 0.9890 | 255 | 0.0045 | 0.0037 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | xelatex | de1020c | 1/1 | recovered | 0.4850 | 0.9901 | (0,0) | 0.4850 | 0.9901 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex | main | 1/1 | recovered | 0.4784 | 0.9899 | (0,3.5) moderate | 0.4351 | 0.9910 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/0 | - |
| 13-math-display-rich | xelatex | pipeline | 1/1 | recovered | 0.5975 | 0.9876 | (1.5,1.5) moderate | 0.5608 | 0.9883 | 255 | 0.0046 | 0.0038 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex-lm | de1020c | 1/1 | recovered | 0.5459 | 0.9888 | (0,0) | 0.5459 | 0.9888 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | xelatex-lm | main | 1/1 | recovered | 0.5569 | 0.9883 | (-0.5,3.5) weak | 0.5415 | 0.9888 | 255 | 0.0044 | 0.0036 | -/-/- | - | - | - | - | 4/0 | - |
| 13-math-display-rich | xelatex-lm | pipeline | 1/1 | recovered | 0.5704 | 0.9877 | (3.5,1.5) moderate | 0.5184 | 0.9892 | 255 | 0.0045 | 0.0037 | -/-/- | - | - | - | - | 4/2 | - |
| 14-math-inline-dense | lualatex | de1020c | 1/1 | recovered | 1.3153 | 0.9706 | (-34.5,13.5) | 0.9661 | 0.9780 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex | main | 1/1 | recovered | 1.3057 | 0.9707 | (-34.5,13.5) | 0.9504 | 0.9783 | 255 | 0.0099 | 0.0081 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | lualatex | pipeline | 1/1 | recovered | 1.0666 | 0.9799 | (0,0) | 1.0666 | 0.9799 | 255 | 0.0087 | 0.0069 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | lualatex-lm | de1020c | 1/1 | recovered | 1.2497 | 0.9701 | (2.5,13.5) moderate | 1.1687 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | main | 1/1 | recovered | 1.2412 | 0.9704 | (5,13.5) moderate | 1.1613 | 0.9733 | 255 | 0.0096 | 0.0079 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | lualatex-lm | pipeline | 1/1 | recovered | 1.0041 | 0.9807 | (-0.5,1.5) moderate | 0.8705 | 0.9836 | 255 | 0.0084 | 0.0068 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | pdflatex | de1020c | 1/1 | recovered | 1.3096 | 0.9709 | (-34,13.5) | 0.9598 | 0.9781 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | pdflatex | main | 1/1 | recovered | 1.3004 | 0.9709 | (-34,13.5) | 0.9333 | 0.9785 | 255 | 0.0098 | 0.0081 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | pdflatex | pipeline | 1/1 | recovered | 1.0580 | 0.9799 | (0,0) | 1.0580 | 0.9799 | 255 | 0.0086 | 0.0068 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | pdflatex-lm | de1020c | 1/1 | recovered | 1.2493 | 0.9701 | (2.5,13.5) moderate | 1.1681 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | pdflatex-lm | main | 1/1 | recovered | 1.2408 | 0.9704 | (5,13.5) moderate | 1.1612 | 0.9733 | 255 | 0.0096 | 0.0079 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | pdflatex-lm | pipeline | 1/1 | recovered | 1.0054 | 0.9807 | (-0.5,1.5) moderate | 0.8682 | 0.9836 | 255 | 0.0084 | 0.0068 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | xelatex | de1020c | 1/1 | recovered | 1.3153 | 0.9706 | (-34.5,13.5) | 0.9632 | 0.9781 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex | main | 1/1 | recovered | 1.3057 | 0.9707 | (-34.5,13.5) | 0.9466 | 0.9783 | 255 | 0.0099 | 0.0081 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | xelatex | pipeline | 1/1 | recovered | 1.0667 | 0.9799 | (0,0) | 1.0667 | 0.9799 | 255 | 0.0087 | 0.0069 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | xelatex-lm | de1020c | 1/1 | recovered | 1.2496 | 0.9701 | (2.5,13.5) moderate | 1.1687 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | main | 1/1 | recovered | 1.2412 | 0.9704 | (5,13.5) moderate | 1.1613 | 0.9733 | 255 | 0.0096 | 0.0079 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | xelatex-lm | pipeline | 1/1 | recovered | 1.0041 | 0.9807 | (-0.5,1.5) moderate | 0.8704 | 0.9836 | 255 | 0.0084 | 0.0068 | -/-/- | - | - | - | - | 2/2 | - |
| 15-three-page-sections | lualatex | de1020c | 3/3 | recovered | 26.4238 | 0.5637 | (0.5,20) moderate; (0.5,19) weak; (0.5,-24.5) moderate | 25.2152 | 0.5876 | 255 | 0.1893 | 0.1584 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex | main | 3/3 | recovered | 26.2996 | 0.5654 | (0,20) moderate; (0,19) weak; (0,-24.5) weak | 25.1074 | 0.5897 | 255 | 0.1893 | 0.1582 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex | pipeline | 3/3 | recovered | 21.9938 | 0.6558 | (0,14) weak; (0,14) weak; (0,14) weak | 21.0819 | 0.6750 | 255 | 0.1659 | 0.1373 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex-lm | de1020c | 3/3 | recovered | 24.2613 | 0.5495 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0400 | 0.5821 | 255 | 0.1832 | 0.1519 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | lualatex-lm | main | 3/3 | recovered | 24.2775 | 0.5496 | (-0.5,22.5) moderate; (-0.5,-24) weak; (0,-24) weak | 22.9605 | 0.5840 | 255 | 0.1835 | 0.1522 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | lualatex-lm | pipeline | 3/3 | recovered | 13.1431 | 0.8173 | (0,0); (0,0); (0,0) | 13.1431 | 0.8173 | 255 | 0.1288 | 0.0973 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex | de1020c | 3/3 | recovered | 26.6983 | 0.5543 | (0.5,36.5) moderate; (0.5,-24.5) weak; (0,-24.5) weak | 25.1309 | 0.5916 | 255 | 0.1901 | 0.1598 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex | main | 3/3 | recovered | 26.7478 | 0.5542 | (0,36.5) moderate; (0,-24.5) moderate; (0,-24.5) moderate | 25.0626 | 0.5936 | 255 | 0.1907 | 0.1602 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex | pipeline | 3/3 | recovered | 21.7473 | 0.6643 | (0.5,-0.5) moderate; (1,-0.5) moderate; (1,-0.5) moderate | 20.5348 | 0.6880 | 255 | 0.1646 | 0.1363 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex-lm | de1020c | 3/3 | recovered | 24.2616 | 0.5495 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0365 | 0.5822 | 255 | 0.1832 | 0.1521 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex-lm | main | 3/3 | recovered | 24.2785 | 0.5495 | (-0.5,22.5) moderate; (-0.5,-24) weak; (-0.5,-24) weak | 22.9554 | 0.5839 | 255 | 0.1836 | 0.1523 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex-lm | pipeline | 3/3 | recovered | 13.1614 | 0.8171 | (0,0); (0,0); (0,0) | 13.1614 | 0.8171 | 255 | 0.1290 | 0.0975 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex | de1020c | 3/3 | recovered | 26.4204 | 0.5639 | (-3,20) moderate; (-3,19) weak; (-3,-24.5) moderate | 25.1894 | 0.5880 | 255 | 0.1893 | 0.1587 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex | main | 3/3 | recovered | 26.3417 | 0.5650 | (0,20) moderate; (0,19) weak; (0,-24.5) weak | 25.1643 | 0.5890 | 255 | 0.1894 | 0.1586 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex | pipeline | 3/3 | recovered | 22.0513 | 0.6548 | (0,14) weak; (0,14) weak; (1,14) weak | 21.0634 | 0.6750 | 255 | 0.1662 | 0.1378 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex-lm | de1020c | 3/3 | recovered | 24.2611 | 0.5495 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0399 | 0.5821 | 255 | 0.1831 | 0.1519 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex-lm | main | 3/3 | recovered | 24.2773 | 0.5496 | (-0.5,22.5) moderate; (-0.5,-24) weak; (0,-24) weak | 22.9604 | 0.5840 | 255 | 0.1835 | 0.1521 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex-lm | pipeline | 3/3 | recovered | 13.1472 | 0.8172 | (0,0); (0,0); (0,0) | 13.1472 | 0.8172 | 255 | 0.1288 | 0.0973 | -/-/- | - | - | - | - | 3/0 | - |
| 16-heading-page-break | lualatex | de1020c | 2/2 | ok | 20.5841 | 0.6613 | (0,0); (0,20.5) moderate | 20.0997 | 0.6688 | 255 | 0.1484 | 0.1240 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex | main | 2/2 | ok | 20.1421 | 0.6739 | (0,0); (0,14.5) weak | 20.0611 | 0.6773 | 255 | 0.1460 | 0.1217 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex | pipeline | 2/2 | ok | 17.5113 | 0.7211 | (0,0); (-1,57.5) weak | 17.3779 | 0.7259 | 255 | 0.1328 | 0.1087 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex-lm | de1020c | 2/2 | ok | 19.0167 | 0.6509 | (-1,0) weak; (-1,20) moderate | 18.0483 | 0.6755 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | lualatex-lm | main | 2/2 | ok | 18.3185 | 0.6672 | (-0.5,-27) weak; (0,0) | 18.3042 | 0.6668 | 255 | 0.1402 | 0.1159 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | lualatex-lm | pipeline | 2/2 | ok | 10.5838 | 0.8523 | (0,0); (0,0) | 10.5838 | 0.8523 | 255 | 0.1036 | 0.0781 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex | de1020c | 2/2 | ok | 20.5976 | 0.6621 | (0,-14.5) weak; (0,20.5) moderate | 20.0459 | 0.6704 | 255 | 0.1481 | 0.1240 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex | main | 2/2 | ok | 20.1484 | 0.6753 | (0,0); (0,14.5) weak | 20.0507 | 0.6788 | 255 | 0.1456 | 0.1217 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex | pipeline | 2/2 | ok | 17.4993 | 0.7228 | (-1.5,0) weak; (-1.5,57.5) weak | 17.2523 | 0.7297 | 255 | 0.1324 | 0.1086 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex-lm | de1020c | 2/2 | ok | 19.0160 | 0.6507 | (-1,0) weak; (-1,20) moderate | 18.0489 | 0.6754 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex-lm | main | 2/2 | ok | 18.3190 | 0.6671 | (-0.5,-27) weak; (0,0) | 18.3048 | 0.6667 | 255 | 0.1402 | 0.1159 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex-lm | pipeline | 2/2 | ok | 10.6029 | 0.8520 | (0,0); (0,0) | 10.6029 | 0.8520 | 255 | 0.1036 | 0.0783 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex | de1020c | 2/2 | ok | 20.6109 | 0.6611 | (-3,-14.5) weak; (0.5,20.5) moderate | 20.1127 | 0.6683 | 255 | 0.1485 | 0.1243 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex | main | 2/2 | ok | 20.1392 | 0.6740 | (0,0); (0,14.5) weak | 20.0559 | 0.6774 | 255 | 0.1460 | 0.1220 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex | pipeline | 2/2 | ok | 17.5421 | 0.7206 | (1.5,0) weak; (-1,57.5) weak | 17.3587 | 0.7256 | 255 | 0.1330 | 0.1091 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex-lm | de1020c | 2/2 | ok | 19.0167 | 0.6509 | (-1,0) weak; (-1,20) moderate | 18.0480 | 0.6754 | 255 | 0.1438 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex-lm | main | 2/2 | ok | 18.3187 | 0.6672 | (-0.5,-27) weak; (0,0) | 18.3042 | 0.6668 | 255 | 0.1402 | 0.1159 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex-lm | pipeline | 2/2 | ok | 10.5866 | 0.8522 | (0,0); (0,0) | 10.5866 | 0.8522 | 255 | 0.1036 | 0.0781 | -/-/- | - | - | - | - | 1/0 | - |
| 17-apostrophes | lualatex | de1020c | 1/1 | ok | 0.8503 | 0.9875 | (-1.5,0) weak | 0.8116 | 0.9882 | 255 | 0.0068 | 0.0053 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex | main | 1/1 | ok | 0.8494 | 0.9875 | (-7.5,0) weak | 0.8284 | 0.9876 | 255 | 0.0068 | 0.0053 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex | pipeline | 1/1 | ok | 0.9451 | 0.9851 | (53.5,0) moderate | 0.8876 | 0.9859 | 255 | 0.0073 | 0.0058 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | de1020c | 1/1 | ok | 0.8624 | 0.9848 | (0,0) | 0.8624 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | main | 1/1 | ok | 0.8761 | 0.9846 | (0,0) | 0.8761 | 0.9846 | 255 | 0.0071 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | pipeline | 1/1 | ok | 0.6767 | 0.9896 | (-0.5,0) moderate | 0.6335 | 0.9899 | 255 | 0.0061 | 0.0047 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | de1020c | 1/1 | ok | 0.8064 | 0.9887 | (0,0) | 0.8064 | 0.9887 | 255 | 0.0066 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | main | 1/1 | ok | 0.8140 | 0.9885 | (0,0) | 0.8140 | 0.9885 | 255 | 0.0067 | 0.0053 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | pipeline | 1/1 | ok | 0.9301 | 0.9857 | (0,0) | 0.9301 | 0.9857 | 255 | 0.0072 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | de1020c | 1/1 | ok | 0.8626 | 0.9848 | (0,0) | 0.8626 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | main | 1/1 | ok | 0.8763 | 0.9846 | (0,0) | 0.8763 | 0.9846 | 255 | 0.0071 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | pipeline | 1/1 | ok | 0.6761 | 0.9896 | (-0.5,0) moderate | 0.6335 | 0.9899 | 255 | 0.0061 | 0.0047 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | de1020c | 1/1 | ok | 0.7982 | 0.9883 | (0,0) | 0.7982 | 0.9883 | 255 | 0.0066 | 0.0051 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | main | 1/1 | ok | 0.8071 | 0.9882 | (0,0) | 0.8071 | 0.9882 | 255 | 0.0067 | 0.0051 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | pipeline | 1/1 | ok | 0.9357 | 0.9853 | (53.5,0) weak | 0.9057 | 0.9856 | 255 | 0.0073 | 0.0058 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | de1020c | 1/1 | ok | 0.8621 | 0.9848 | (0,0) | 0.8621 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | main | 1/1 | ok | 0.8760 | 0.9846 | (0,0) | 0.8760 | 0.9846 | 255 | 0.0071 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | pipeline | 1/1 | ok | 0.6766 | 0.9896 | (-0.5,0) moderate | 0.6330 | 0.9899 | 255 | 0.0061 | 0.0047 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | de1020c | 1/1 | ok | 1.3878 | 0.9788 | (0,0) | 1.3878 | 0.9788 | 255 | 0.0109 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | main | 1/1 | ok | 1.1884 | 0.9831 | (-0.5,0) moderate | 1.1154 | 0.9842 | 255 | 0.0101 | 0.0077 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | pipeline | 1/1 | ok | 1.3808 | 0.9786 | (0,0) | 1.3808 | 0.9786 | 255 | 0.0110 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | de1020c | 1/1 | ok | 1.3042 | 0.9789 | (0,0) | 1.3042 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | main | 1/1 | ok | 1.3761 | 0.9771 | (-2.5,0) weak | 1.3651 | 0.9775 | 255 | 0.0109 | 0.0090 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | pipeline | 1/1 | ok | 0.9109 | 0.9874 | (0,0) | 0.9109 | 0.9874 | 255 | 0.0089 | 0.0067 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | de1020c | 1/1 | ok | 1.2717 | 0.9812 | (0,0) | 1.2717 | 0.9812 | 255 | 0.0104 | 0.0080 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | main | 1/1 | ok | 1.3112 | 0.9812 | (-2,0) moderate | 1.1522 | 0.9842 | 255 | 0.0105 | 0.0082 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | pipeline | 1/1 | ok | 1.3535 | 0.9796 | (0,0) | 1.3535 | 0.9796 | 255 | 0.0107 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | de1020c | 1/1 | ok | 1.3056 | 0.9789 | (0,0) | 1.3056 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | main | 1/1 | ok | 1.3757 | 0.9771 | (-5.5,0) weak | 1.3586 | 0.9773 | 255 | 0.0109 | 0.0090 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | pipeline | 1/1 | ok | 0.9120 | 0.9874 | (0,0) | 0.9120 | 0.9874 | 255 | 0.0089 | 0.0067 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | de1020c | 1/1 | ok | 1.3346 | 0.9799 | (0,0) | 1.3346 | 0.9799 | 255 | 0.0107 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | main | 1/1 | ok | 1.3063 | 0.9812 | (-1,0) moderate | 1.1838 | 0.9832 | 255 | 0.0105 | 0.0084 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | pipeline | 1/1 | ok | 1.3721 | 0.9788 | (0,0) | 1.3721 | 0.9788 | 255 | 0.0110 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | de1020c | 1/1 | ok | 1.3041 | 0.9789 | (0,0) | 1.3041 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | main | 1/1 | ok | 1.3760 | 0.9771 | (-2.5,0) weak | 1.3651 | 0.9775 | 255 | 0.0109 | 0.0090 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | pipeline | 1/1 | ok | 0.9112 | 0.9874 | (0,0) | 0.9112 | 0.9874 | 255 | 0.0089 | 0.0067 | -/-/- | - | - | - | - | 0/0 | - |

## Diagnostic: native preview capture comparison (screen capture of the running FlashTeXMac preview vs reference PDF raster)

The actual SwiftUI Canvas preview, captured with `screencapture -l <window id>`, page region detected and resampled to the reference raster size (resample factor 1.5242839352428394–1.5242839352428394, display backing [2.0] px/pt; see provenance). The 'page N' caption corner is masked white. Word-box metrics are unavailable (a screenshot has no text layer). This is a separate, independent comparison from the export table above and from the weaker preview-equivalent table.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| raw | SSIM₈ raw | registration Δ pt (dx,dy per page; `weak`/`moderate` = shift explains <25% of the error) | mean\|Δ\| after reg | SSIM₈ after reg | max | differing | ≥thr | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-plain-paragraph | lualatex | de1020c | 1/1 | ok | 0.4151 | 0.9942 | (-0.5,0.5) moderate | 0.3143 | 0.9964 | 255 | 0.0209 | 0.0029 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.4708 | 0.9933 | (-0.5,0.5) | 0.3198 | 0.9963 | 255 | 0.0211 | 0.0031 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | pipeline | 1/1 | ok | 0.4968 | 0.9922 | (0,0.5) moderate | 0.4596 | 0.9930 | 255 | 0.0217 | 0.0033 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | de1020c | 1/1 | ok | 0.5088 | 0.9924 | (0.5,0.5) weak | 0.4933 | 0.9928 | 255 | 0.0210 | 0.0034 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | main | 1/1 | ok | 0.5069 | 0.9924 | (0,0.5) moderate | 0.4810 | 0.9931 | 255 | 0.0212 | 0.0034 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | pipeline | 1/1 | ok | 0.3176 | 0.9949 | (0,0.5) | 0.2362 | 0.9972 | 250 | 0.0217 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.3969 | 0.9946 | (0,0.5) moderate | 0.3365 | 0.9957 | 255 | 0.0209 | 0.0028 | -/-/- | - | - | - | - | 0/0 | [p1](images/01-plain-paragraph/pdflatex-de1020c-native-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.3639 | 0.9952 | (0,0.5) moderate | 0.2926 | 0.9965 | 255 | 0.0211 | 0.0026 | -/-/- | - | - | - | - | 0/0 | [p1](images/01-plain-paragraph/pdflatex-main-native-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | pipeline | 1/1 | ok | 0.4956 | 0.9924 | (4,0.5) weak | 0.4802 | 0.9925 | 255 | 0.0217 | 0.0033 | -/-/- | - | - | - | - | 0/0 | [p1](images/01-plain-paragraph/pdflatex-pipeline-native-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 0.5086 | 0.9924 | (0.5,0.5) weak | 0.4932 | 0.9928 | 255 | 0.0210 | 0.0034 | -/-/- | - | - | - | - | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | main | 1/1 | ok | 0.5068 | 0.9925 | (0,0.5) moderate | 0.4809 | 0.9931 | 255 | 0.0212 | 0.0034 | -/-/- | - | - | - | - | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-main-native-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 0.3184 | 0.9949 | (0,0.5) | 0.2371 | 0.9972 | 250 | 0.0217 | 0.0025 | -/-/- | - | - | - | - | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.4178 | 0.9942 | (-0.5,0.5) | 0.3122 | 0.9964 | 255 | 0.0209 | 0.0029 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.4741 | 0.9932 | (-0.5,0.5) | 0.3228 | 0.9962 | 255 | 0.0211 | 0.0031 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | pipeline | 1/1 | ok | 0.4970 | 0.9922 | (0,0.5) moderate | 0.4596 | 0.9930 | 255 | 0.0217 | 0.0033 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | de1020c | 1/1 | ok | 0.5089 | 0.9924 | (0.5,0.5) weak | 0.4935 | 0.9928 | 255 | 0.0210 | 0.0034 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | main | 1/1 | ok | 0.5071 | 0.9924 | (0,0.5) moderate | 0.4812 | 0.9931 | 255 | 0.0212 | 0.0034 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | pipeline | 1/1 | ok | 0.3178 | 0.9949 | (0,0.5) | 0.2364 | 0.9972 | 250 | 0.0217 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 10.3044 | 0.8609 | (0,0.5) weak | 10.2325 | 0.8632 | 255 | 0.1894 | 0.0660 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 10.3268 | 0.8610 | (-0.5,0.5) weak | 10.2669 | 0.8634 | 255 | 0.1895 | 0.0662 | -/-/- | - | - | - | - | 0/1 | - |
| 02-wrapping-paragraph | lualatex | pipeline | 1/1 | ok | 8.3202 | 0.8822 | (0,0.5) weak | 8.2117 | 0.8835 | 255 | 0.1927 | 0.0557 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | de1020c | 1/1 | ok | 9.7854 | 0.8588 | (-1,0.5) weak | 9.7770 | 0.8603 | 255 | 0.1897 | 0.0647 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | main | 1/1 | ok | 9.8156 | 0.8579 | (-1,0.5) weak | 9.7711 | 0.8597 | 255 | 0.1898 | 0.0650 | -/-/- | - | - | - | - | 0/1 | - |
| 02-wrapping-paragraph | lualatex-lm | pipeline | 1/1 | ok | 4.3952 | 0.9612 | (0,0) | 4.3952 | 0.9612 | 254 | 0.1926 | 0.0366 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 10.3130 | 0.8610 | (-3,0.5) weak | 10.2559 | 0.8636 | 255 | 0.1894 | 0.0660 | -/-/- | - | - | - | - | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-de1020c-native-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 10.3777 | 0.8605 | (-4,0.5) weak | 10.2527 | 0.8639 | 255 | 0.1895 | 0.0664 | -/-/- | - | - | - | - | 0/1 | [p1](images/02-wrapping-paragraph/pdflatex-main-native-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | pipeline | 1/1 | ok | 8.3123 | 0.8826 | (0,0.5) weak | 8.2061 | 0.8840 | 255 | 0.1927 | 0.0557 | -/-/- | - | - | - | - | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-pipeline-native-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 9.7842 | 0.8588 | (0,0) | 9.7842 | 0.8588 | 255 | 0.1897 | 0.0648 | -/-/- | - | - | - | - | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | main | 1/1 | ok | 9.8161 | 0.8579 | (-1,0.5) weak | 9.7704 | 0.8598 | 255 | 0.1898 | 0.0651 | -/-/- | - | - | - | - | 0/1 | [p1](images/02-wrapping-paragraph/pdflatex-lm-main-native-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 4.4039 | 0.9610 | (0,0) | 4.4039 | 0.9610 | 254 | 0.1926 | 0.0366 | -/-/- | - | - | - | - | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 10.3077 | 0.8609 | (-3,0.5) weak | 10.2443 | 0.8633 | 255 | 0.1894 | 0.0661 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 10.3623 | 0.8604 | (-0.5,0.5) weak | 10.2875 | 0.8627 | 255 | 0.1895 | 0.0664 | -/-/- | - | - | - | - | 0/1 | - |
| 02-wrapping-paragraph | xelatex | pipeline | 1/1 | ok | 8.3423 | 0.8819 | (0,0.5) weak | 8.2366 | 0.8831 | 255 | 0.1927 | 0.0558 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | de1020c | 1/1 | ok | 9.7854 | 0.8588 | (0,0) | 9.7854 | 0.8588 | 255 | 0.1897 | 0.0647 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | main | 1/1 | ok | 9.8157 | 0.8579 | (-1,0.5) weak | 9.7713 | 0.8597 | 255 | 0.1898 | 0.0650 | -/-/- | - | - | - | - | 0/1 | - |
| 02-wrapping-paragraph | xelatex-lm | pipeline | 1/1 | ok | 4.3973 | 0.9612 | (0,0) | 4.3973 | 0.9612 | 254 | 0.1926 | 0.0366 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.6517 | 0.9776 | (0,52.5) moderate | 1.4359 | 0.9820 | 255 | 0.0398 | 0.0094 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.8461 | 0.9768 | (12,52) moderate | 1.6466 | 0.9801 | 255 | 0.0427 | 0.0103 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | pipeline | 1/1 | ok | 1.2629 | 0.9847 | (29.5,-0.5) weak | 1.2039 | 0.9852 | 255 | 0.0433 | 0.0075 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | de1020c | 1/1 | ok | 1.5532 | 0.9755 | (0,54) moderate | 1.3581 | 0.9806 | 255 | 0.0397 | 0.0090 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | main | 1/1 | ok | 1.7274 | 0.9749 | (0,54) moderate | 1.5791 | 0.9795 | 255 | 0.0423 | 0.0098 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | pipeline | 1/1 | ok | 1.0048 | 0.9898 | (29,0.5) moderate | 0.8342 | 0.9889 | 255 | 0.0433 | 0.0062 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.6590 | 0.9776 | (0,52.5) moderate | 1.4429 | 0.9820 | 255 | 0.0398 | 0.0093 | -/-/- | - | - | - | - | 0/0 | [p1](images/03-section-heading/pdflatex-de1020c-native-p1-overlay.png) |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.8508 | 0.9768 | (12,-18.5) moderate | 1.6468 | 0.9813 | 255 | 0.0427 | 0.0102 | -/-/- | - | - | - | - | 0/0 | [p1](images/03-section-heading/pdflatex-main-native-p1-overlay.png) |
| 03-section-heading | pdflatex | pipeline | 1/1 | ok | 1.2840 | 0.9845 | (29.5,-0.5) weak | 1.2260 | 0.9851 | 255 | 0.0433 | 0.0075 | -/-/- | - | - | - | - | 0/0 | [p1](images/03-section-heading/pdflatex-pipeline-native-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | de1020c | 1/1 | ok | 1.5534 | 0.9755 | (0,54) moderate | 1.3586 | 0.9806 | 255 | 0.0397 | 0.0090 | -/-/- | - | - | - | - | 0/0 | [p1](images/03-section-heading/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | main | 1/1 | ok | 1.7277 | 0.9750 | (0,-17) moderate | 1.5737 | 0.9798 | 255 | 0.0423 | 0.0098 | -/-/- | - | - | - | - | 0/0 | [p1](images/03-section-heading/pdflatex-lm-main-native-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | pipeline | 1/1 | ok | 1.0111 | 0.9897 | (29,0.5) moderate | 0.8336 | 0.9889 | 255 | 0.0433 | 0.0062 | -/-/- | - | - | - | - | 0/0 | [p1](images/03-section-heading/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.6495 | 0.9776 | (0,52.5) moderate | 1.4352 | 0.9821 | 255 | 0.0398 | 0.0094 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.8443 | 0.9768 | (12,52) moderate | 1.6448 | 0.9802 | 255 | 0.0427 | 0.0103 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | pipeline | 1/1 | ok | 1.2597 | 0.9848 | (29.5,-0.5) weak | 1.2081 | 0.9852 | 255 | 0.0433 | 0.0075 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | de1020c | 1/1 | ok | 1.5532 | 0.9755 | (0,54) moderate | 1.3581 | 0.9806 | 255 | 0.0397 | 0.0090 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | main | 1/1 | ok | 1.7273 | 0.9749 | (0,54) moderate | 1.5791 | 0.9795 | 255 | 0.0423 | 0.0099 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | pipeline | 1/1 | ok | 1.0049 | 0.9898 | (29,0.5) moderate | 0.8346 | 0.9889 | 255 | 0.0433 | 0.0062 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.5211 | 0.9928 | (-0.5,0.5) moderate | 0.4755 | 0.9934 | 255 | 0.0212 | 0.0034 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.5199 | 0.9928 | (-0.5,0.5) moderate | 0.4741 | 0.9934 | 255 | 0.0215 | 0.0034 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | pipeline | 1/1 | ok | 0.5812 | 0.9916 | (13.5,0.5) moderate | 0.5406 | 0.9920 | 255 | 0.0228 | 0.0038 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | de1020c | 1/1 | ok | 0.5519 | 0.9917 | (-49,0.5) weak | 0.5331 | 0.9919 | 255 | 0.0213 | 0.0036 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | main | 1/1 | ok | 0.5533 | 0.9917 | (-49,0.5) weak | 0.5341 | 0.9919 | 255 | 0.0216 | 0.0036 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | pipeline | 1/1 | ok | 0.4699 | 0.9929 | (0,0.5) moderate | 0.4311 | 0.9941 | 255 | 0.0228 | 0.0032 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.5283 | 0.9925 | (-0.5,0.5) moderate | 0.4743 | 0.9934 | 255 | 0.0212 | 0.0035 | -/-/- | - | - | - | - | 0/0 | [p1](images/04-bold-emph/pdflatex-de1020c-native-p1-overlay.png) |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.5275 | 0.9925 | (-0.5,0.5) moderate | 0.4751 | 0.9934 | 255 | 0.0215 | 0.0035 | -/-/- | - | - | - | - | 0/0 | [p1](images/04-bold-emph/pdflatex-main-native-p1-overlay.png) |
| 04-bold-emph | pdflatex | pipeline | 1/1 | ok | 0.5742 | 0.9916 | (10,0.5) moderate | 0.5273 | 0.9924 | 255 | 0.0228 | 0.0037 | -/-/- | - | - | - | - | 0/0 | [p1](images/04-bold-emph/pdflatex-pipeline-native-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | de1020c | 1/1 | ok | 0.5517 | 0.9917 | (-49,0.5) weak | 0.5337 | 0.9919 | 255 | 0.0213 | 0.0036 | -/-/- | - | - | - | - | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | main | 1/1 | ok | 0.5531 | 0.9917 | (-49,0.5) weak | 0.5352 | 0.9919 | 255 | 0.0216 | 0.0036 | -/-/- | - | - | - | - | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-main-native-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | pipeline | 1/1 | ok | 0.4762 | 0.9928 | (0,0.5) moderate | 0.4395 | 0.9940 | 255 | 0.0228 | 0.0032 | -/-/- | - | - | - | - | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.5245 | 0.9928 | (-0.5,0.5) moderate | 0.4785 | 0.9933 | 255 | 0.0212 | 0.0034 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.5228 | 0.9928 | (-0.5,0.5) moderate | 0.4775 | 0.9933 | 255 | 0.0215 | 0.0034 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | pipeline | 1/1 | ok | 0.5804 | 0.9916 | (13.5,0.5) moderate | 0.5414 | 0.9920 | 255 | 0.0228 | 0.0037 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | de1020c | 1/1 | ok | 0.5506 | 0.9917 | (-49,0.5) weak | 0.5323 | 0.9919 | 255 | 0.0213 | 0.0036 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | main | 1/1 | ok | 0.5517 | 0.9917 | (-49,0.5) weak | 0.5326 | 0.9919 | 255 | 0.0216 | 0.0036 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | pipeline | 1/1 | ok | 0.4575 | 0.9932 | (0,0.5) moderate | 0.4146 | 0.9945 | 255 | 0.0228 | 0.0031 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.4633 | 0.9932 | (0,0.5) moderate | 0.4274 | 0.9938 | 255 | 0.0211 | 0.0030 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.4452 | 0.9935 | (-0.5,0.5) moderate | 0.4050 | 0.9943 | 255 | 0.0215 | 0.0029 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | pipeline | 1/1 | ok | 0.4601 | 0.9928 | (9.5,0.5) moderate | 0.3949 | 0.9937 | 255 | 0.0198 | 0.0031 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex-lm | de1020c | 1/1 | ok | 0.4856 | 0.9920 | (2,0.5) moderate | 0.4575 | 0.9928 | 255 | 0.0211 | 0.0032 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | main | 1/1 | ok | 0.4874 | 0.9921 | (-3.5,0.5) moderate | 0.4560 | 0.9928 | 255 | 0.0215 | 0.0032 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | pipeline | 1/1 | ok | 0.2878 | 0.9952 | (0,0.5) | 0.2098 | 0.9975 | 254 | 0.0198 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.4600 | 0.9934 | (-5.5,0.5) weak | 0.4479 | 0.9936 | 255 | 0.0211 | 0.0030 | -/-/- | - | - | - | - | 2/0 | [p1](images/05-unicode/pdflatex-de1020c-native-p1-overlay.png) |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.4289 | 0.9939 | (0,0.5) moderate | 0.3819 | 0.9946 | 255 | 0.0215 | 0.0029 | -/-/- | - | - | - | - | 2/0 | [p1](images/05-unicode/pdflatex-main-native-p1-overlay.png) |
| 05-unicode | pdflatex | pipeline | 1/1 | ok | 0.4640 | 0.9929 | (10,0.5) moderate | 0.4052 | 0.9936 | 255 | 0.0198 | 0.0031 | -/-/- | - | - | - | - | 2/0 | [p1](images/05-unicode/pdflatex-pipeline-native-p1-overlay.png) |
| 05-unicode | pdflatex-lm | de1020c | 1/1 | ok | 0.4855 | 0.9920 | (2,0.5) moderate | 0.4572 | 0.9928 | 255 | 0.0211 | 0.0032 | -/-/- | - | - | - | - | 0/0 | [p1](images/05-unicode/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 05-unicode | pdflatex-lm | main | 1/1 | ok | 0.4872 | 0.9921 | (-3.5,0.5) moderate | 0.4558 | 0.9928 | 255 | 0.0215 | 0.0032 | -/-/- | - | - | - | - | 0/0 | [p1](images/05-unicode/pdflatex-lm-main-native-p1-overlay.png) |
| 05-unicode | pdflatex-lm | pipeline | 1/1 | ok | 0.2884 | 0.9952 | (0,0.5) | 0.2103 | 0.9975 | 254 | 0.0198 | 0.0023 | -/-/- | - | - | - | - | 0/0 | [p1](images/05-unicode/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.4641 | 0.9932 | (0,0.5) moderate | 0.4278 | 0.9938 | 255 | 0.0211 | 0.0030 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.4462 | 0.9935 | (-0.5,0.5) moderate | 0.4033 | 0.9943 | 255 | 0.0215 | 0.0029 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | pipeline | 1/1 | ok | 0.4596 | 0.9928 | (9.5,0.5) moderate | 0.3949 | 0.9937 | 255 | 0.0198 | 0.0031 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex-lm | de1020c | 1/1 | ok | 0.4856 | 0.9920 | (2,0.5) moderate | 0.4574 | 0.9928 | 255 | 0.0211 | 0.0032 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | main | 1/1 | ok | 0.4874 | 0.9921 | (-3.5,0.5) moderate | 0.4560 | 0.9928 | 255 | 0.0215 | 0.0032 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | pipeline | 1/1 | ok | 0.2878 | 0.9952 | (0,0.5) | 0.2098 | 0.9975 | 254 | 0.0198 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.4879 | 0.9930 | (0,3.5) | 0.3617 | 0.9945 | 255 | 0.0177 | 0.0031 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | ok | 0.4879 | 0.9930 | (-6.5,3.5) | 0.3224 | 0.9953 | 255 | 0.0178 | 0.0032 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | pipeline | 1/1 | ok | 0.4154 | 0.9927 | (10.5,0.5) moderate | 0.3672 | 0.9937 | 255 | 0.0187 | 0.0028 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | de1020c | 1/1 | ok | 0.4748 | 0.9927 | (-32.5,3.5) moderate | 0.4059 | 0.9933 | 255 | 0.0178 | 0.0031 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | main | 1/1 | ok | 0.4781 | 0.9926 | (9.5,3.5) moderate | 0.4090 | 0.9933 | 255 | 0.0179 | 0.0032 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | pipeline | 1/1 | ok | 0.3677 | 0.9933 | (-1.5,0.5) moderate | 0.2786 | 0.9958 | 255 | 0.0187 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.4856 | 0.9932 | (0,3.5) | 0.3602 | 0.9946 | 255 | 0.0177 | 0.0031 | -/-/- | - | - | - | - | 0/0 | [p1](images/06-math-inline/pdflatex-de1020c-native-p1-overlay.png) |
| 06-math-inline | pdflatex | main | 1/1 | ok | 0.4847 | 0.9932 | (-6.5,3.5) | 0.3197 | 0.9952 | 255 | 0.0178 | 0.0032 | -/-/- | - | - | - | - | 0/0 | [p1](images/06-math-inline/pdflatex-main-native-p1-overlay.png) |
| 06-math-inline | pdflatex | pipeline | 1/1 | ok | 0.4100 | 0.9929 | (-9,0.5) weak | 0.3981 | 0.9934 | 255 | 0.0187 | 0.0028 | -/-/- | - | - | - | - | 0/0 | [p1](images/06-math-inline/pdflatex-pipeline-native-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | de1020c | 1/1 | ok | 0.4745 | 0.9927 | (-0.5,3.5) moderate | 0.3894 | 0.9937 | 255 | 0.0178 | 0.0031 | -/-/- | - | - | - | - | 0/0 | [p1](images/06-math-inline/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | main | 1/1 | ok | 0.4776 | 0.9926 | (9.5,3.5) moderate | 0.4085 | 0.9933 | 255 | 0.0179 | 0.0032 | -/-/- | - | - | - | - | 0/0 | [p1](images/06-math-inline/pdflatex-lm-main-native-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | pipeline | 1/1 | ok | 0.3678 | 0.9933 | (-1.5,0.5) moderate | 0.2763 | 0.9958 | 255 | 0.0187 | 0.0025 | -/-/- | - | - | - | - | 0/0 | [p1](images/06-math-inline/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.4880 | 0.9930 | (0,3.5) | 0.3618 | 0.9945 | 255 | 0.0177 | 0.0031 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | ok | 0.4880 | 0.9930 | (-6.5,3.5) | 0.3229 | 0.9953 | 255 | 0.0178 | 0.0032 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | pipeline | 1/1 | ok | 0.4157 | 0.9927 | (10.5,0.5) moderate | 0.3673 | 0.9937 | 255 | 0.0187 | 0.0028 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | de1020c | 1/1 | ok | 0.4748 | 0.9927 | (-0.5,3.5) moderate | 0.3900 | 0.9937 | 255 | 0.0178 | 0.0031 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | main | 1/1 | ok | 0.4782 | 0.9926 | (9.5,3.5) moderate | 0.4089 | 0.9933 | 255 | 0.0179 | 0.0032 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | pipeline | 1/1 | ok | 0.3678 | 0.9933 | (-1.5,0.5) moderate | 0.2783 | 0.9958 | 255 | 0.0187 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.4020 | 0.9935 | (0,0.5) moderate | 0.3686 | 0.9942 | 255 | 0.0180 | 0.0028 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | ok | 0.4209 | 0.9931 | (0,0.5) moderate | 0.3877 | 0.9937 | 255 | 0.0197 | 0.0029 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex | pipeline | 1/1 | ok | 0.4345 | 0.9932 | (0,0.5) weak | 0.4208 | 0.9933 | 255 | 0.0212 | 0.0030 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | de1020c | 1/1 | ok | 0.4549 | 0.9923 | (0,2) weak | 0.4418 | 0.9929 | 255 | 0.0180 | 0.0030 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | main | 1/1 | ok | 0.4736 | 0.9918 | (-0.5,0.5) weak | 0.4531 | 0.9923 | 255 | 0.0196 | 0.0031 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | pipeline | 1/1 | ok | 0.3757 | 0.9941 | (0,0.5) moderate | 0.3542 | 0.9946 | 255 | 0.0212 | 0.0027 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.4008 | 0.9936 | (0,0.5) moderate | 0.3694 | 0.9941 | 255 | 0.0180 | 0.0027 | -/-/- | - | - | - | - | 3/1 | [p1](images/07-math-display/pdflatex-de1020c-native-p1-overlay.png) |
| 07-math-display | pdflatex | main | 1/1 | ok | 0.4198 | 0.9931 | (0,0.5) moderate | 0.3884 | 0.9936 | 255 | 0.0197 | 0.0029 | -/-/- | - | - | - | - | 3/1 | [p1](images/07-math-display/pdflatex-main-native-p1-overlay.png) |
| 07-math-display | pdflatex | pipeline | 1/1 | ok | 0.4376 | 0.9933 | (0,0.5) weak | 0.4244 | 0.9933 | 255 | 0.0212 | 0.0030 | -/-/- | - | - | - | - | 3/1 | [p1](images/07-math-display/pdflatex-pipeline-native-p1-overlay.png) |
| 07-math-display | pdflatex-lm | de1020c | 1/1 | ok | 0.4577 | 0.9923 | (-0.5,0.5) weak | 0.4379 | 0.9927 | 255 | 0.0180 | 0.0030 | -/-/- | - | - | - | - | 3/1 | [p1](images/07-math-display/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 07-math-display | pdflatex-lm | main | 1/1 | ok | 0.4761 | 0.9918 | (-0.5,0.5) weak | 0.4559 | 0.9922 | 255 | 0.0196 | 0.0032 | -/-/- | - | - | - | - | 3/1 | [p1](images/07-math-display/pdflatex-lm-main-native-p1-overlay.png) |
| 07-math-display | pdflatex-lm | pipeline | 1/1 | ok | 0.3815 | 0.9939 | (0,0.5) moderate | 0.3547 | 0.9946 | 255 | 0.0212 | 0.0027 | -/-/- | - | - | - | - | 3/1 | [p1](images/07-math-display/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.4021 | 0.9935 | (0,0.5) moderate | 0.3688 | 0.9942 | 255 | 0.0180 | 0.0028 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | ok | 0.4210 | 0.9930 | (0,0.5) moderate | 0.3878 | 0.9937 | 255 | 0.0197 | 0.0029 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | pipeline | 1/1 | ok | 0.4354 | 0.9932 | (0,0.5) weak | 0.4216 | 0.9932 | 255 | 0.0212 | 0.0030 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | de1020c | 1/1 | ok | 0.4550 | 0.9923 | (0,2) weak | 0.4418 | 0.9929 | 255 | 0.0180 | 0.0030 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | main | 1/1 | ok | 0.4737 | 0.9918 | (-0.5,2) weak | 0.4558 | 0.9926 | 255 | 0.0196 | 0.0031 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | pipeline | 1/1 | ok | 0.3758 | 0.9941 | (0,0.5) moderate | 0.3543 | 0.9946 | 255 | 0.0212 | 0.0027 | -/-/- | - | - | - | - | 3/1 | - |
| 08-two-page | lualatex | de1020c | 3/1 | ok | 37.7606 | 0.4761 | (0,2) weak | 37.6280 | 0.4790 | 255 | 0.6343 | 0.2407 | -/-/- | - | - | - | - | 0/2 | - |
| 08-two-page | lualatex | main | 3/1 | ok | 39.5859 | 0.4606 | (0,0) | 39.5859 | 0.4606 | 255 | 0.6490 | 0.2536 | -/-/- | - | - | - | - | 0/3 | - |
| 08-two-page | lualatex | pipeline | 3/1 | ok | 30.7523 | 0.5623 | (0,0.5) weak | 29.9541 | 0.5841 | 255 | 0.6678 | 0.2041 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | de1020c | 3/1 | ok | 34.3279 | 0.4929 | (0,0) | 34.3279 | 0.4929 | 255 | 0.6335 | 0.2252 | -/-/- | - | - | - | - | 0/2 | - |
| 08-two-page | lualatex-lm | main | 3/1 | ok | 36.3179 | 0.4695 | (-1,-26.5) weak | 36.3082 | 0.4710 | 255 | 0.6490 | 0.2385 | -/-/- | - | - | - | - | 0/3 | - |
| 08-two-page | lualatex-lm | pipeline | 3/1 | ok | 16.2749 | 0.8433 | (0,0.5) moderate | 14.6808 | 0.8871 | 255 | 0.6675 | 0.1326 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/1 | ok | 37.7343 | 0.4770 | (0,2) weak | 37.5275 | 0.4823 | 255 | 0.6342 | 0.2405 | -/-/- | - | - | - | - | 0/2 | [p1](images/08-two-page/pdflatex-de1020c-native-p1-overlay.png) |
| 08-two-page | pdflatex | main | 3/1 | ok | 39.6113 | 0.4610 | (0,2.5) weak | 39.5335 | 0.4626 | 255 | 0.6490 | 0.2534 | -/-/- | - | - | - | - | 0/3 | [p1](images/08-two-page/pdflatex-main-native-p1-overlay.png) |
| 08-two-page | pdflatex | pipeline | 3/1 | ok | 30.6738 | 0.5639 | (0,0.5) weak | 29.8708 | 0.5863 | 255 | 0.6678 | 0.2040 | -/-/- | - | - | - | - | 0/0 | [p1](images/08-two-page/pdflatex-pipeline-native-p1-overlay.png) |
| 08-two-page | pdflatex-lm | de1020c | 3/1 | ok | 34.3260 | 0.4929 | (0,0) | 34.3260 | 0.4929 | 255 | 0.6335 | 0.2253 | -/-/- | - | - | - | - | 0/2 | [p1](images/08-two-page/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 08-two-page | pdflatex-lm | main | 3/1 | ok | 36.3176 | 0.4695 | (-1,-26.5) weak | 36.3076 | 0.4710 | 255 | 0.6490 | 0.2386 | -/-/- | - | - | - | - | 0/3 | [p1](images/08-two-page/pdflatex-lm-main-native-p1-overlay.png) |
| 08-two-page | pdflatex-lm | pipeline | 3/1 | ok | 16.3017 | 0.8428 | (0,0.5) moderate | 14.7096 | 0.8866 | 255 | 0.6675 | 0.1327 | -/-/- | - | - | - | - | 0/0 | [p1](images/08-two-page/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 08-two-page | xelatex | de1020c | 3/1 | ok | 37.7872 | 0.4760 | (-3,1) weak | 37.5767 | 0.4761 | 255 | 0.6344 | 0.2414 | -/-/- | - | - | - | - | 0/2 | - |
| 08-two-page | xelatex | main | 3/1 | ok | 39.5602 | 0.4610 | (0,0) | 39.5602 | 0.4610 | 255 | 0.6490 | 0.2539 | -/-/- | - | - | - | - | 0/3 | - |
| 08-two-page | xelatex | pipeline | 3/1 | ok | 30.7483 | 0.5618 | (2.5,0.5) weak | 29.9578 | 0.5821 | 255 | 0.6678 | 0.2043 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | de1020c | 3/1 | ok | 34.3287 | 0.4929 | (0,0) | 34.3287 | 0.4929 | 255 | 0.6335 | 0.2253 | -/-/- | - | - | - | - | 0/2 | - |
| 08-two-page | xelatex-lm | main | 3/1 | ok | 36.3186 | 0.4695 | (-1,-26.5) weak | 36.3084 | 0.4710 | 255 | 0.6490 | 0.2385 | -/-/- | - | - | - | - | 0/3 | - |
| 08-two-page | xelatex-lm | pipeline | 3/1 | ok | 16.2817 | 0.8432 | (0,0.5) moderate | 14.6886 | 0.8870 | 255 | 0.6676 | 0.1326 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 3.1731 | 0.9489 | (-0.5,39) | 2.3357 | 0.9670 | 255 | 0.0824 | 0.0191 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | ok | 3.2377 | 0.9485 | (0,39) | 2.3224 | 0.9676 | 255 | 0.0845 | 0.0194 | -/-/- | - | - | - | - | 0/2 | - |
| 09-mixed-document | lualatex | pipeline | 1/1 | ok | 2.1323 | 0.9699 | (0.5,0.5) weak | 2.1088 | 0.9715 | 255 | 0.0795 | 0.0139 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | de1020c | 1/1 | ok | 2.9378 | 0.9480 | (-0.5,39) moderate | 2.4421 | 0.9617 | 255 | 0.0818 | 0.0184 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | main | 1/1 | ok | 3.0024 | 0.9476 | (-0.5,39) moderate | 2.5065 | 0.9613 | 255 | 0.0839 | 0.0188 | -/-/- | - | - | - | - | 0/2 | - |
| 09-mixed-document | lualatex-lm | pipeline | 1/1 | ok | 1.6175 | 0.9788 | (0,1) moderate | 1.3792 | 0.9858 | 255 | 0.0796 | 0.0116 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 3.1730 | 0.9489 | (-0.5,39) | 2.3594 | 0.9665 | 255 | 0.0823 | 0.0191 | -/-/- | - | - | - | - | 0/0 | [p1](images/09-mixed-document/pdflatex-de1020c-native-p1-overlay.png) |
| 09-mixed-document | pdflatex | main | 1/1 | ok | 3.2361 | 0.9484 | (0,39) | 2.3939 | 0.9667 | 255 | 0.0844 | 0.0194 | -/-/- | - | - | - | - | 0/2 | [p1](images/09-mixed-document/pdflatex-main-native-p1-overlay.png) |
| 09-mixed-document | pdflatex | pipeline | 1/1 | ok | 2.1606 | 0.9696 | (0.5,0.5) weak | 2.1363 | 0.9711 | 255 | 0.0796 | 0.0140 | -/-/- | - | - | - | - | 0/0 | [p1](images/09-mixed-document/pdflatex-pipeline-native-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | de1020c | 1/1 | ok | 2.9360 | 0.9466 | (-0.5,39.5) moderate | 2.4572 | 0.9609 | 255 | 0.0817 | 0.0183 | -/-/- | - | - | - | - | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | main | 1/1 | ok | 2.9999 | 0.9462 | (-0.5,40) moderate | 2.5094 | 0.9610 | 255 | 0.0838 | 0.0186 | -/-/- | - | - | - | - | 0/2 | [p1](images/09-mixed-document/pdflatex-lm-main-native-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | pipeline | 1/1 | ok | 1.7273 | 0.9756 | (0,1.5) moderate | 1.4597 | 0.9826 | 255 | 0.0796 | 0.0123 | -/-/- | - | - | - | - | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 3.1727 | 0.9488 | (-0.5,39) | 2.3566 | 0.9665 | 255 | 0.0824 | 0.0190 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | ok | 3.2374 | 0.9483 | (0,39) | 2.3967 | 0.9665 | 255 | 0.0844 | 0.0194 | -/-/- | - | - | - | - | 0/2 | - |
| 09-mixed-document | xelatex | pipeline | 1/1 | ok | 2.1400 | 0.9698 | (0.5,0.5) weak | 2.1048 | 0.9715 | 255 | 0.0795 | 0.0139 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | de1020c | 1/1 | ok | 2.9378 | 0.9480 | (-0.5,39) moderate | 2.4420 | 0.9617 | 255 | 0.0818 | 0.0184 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | main | 1/1 | ok | 3.0024 | 0.9476 | (-0.5,39) moderate | 2.5065 | 0.9613 | 255 | 0.0839 | 0.0188 | -/-/- | - | - | - | - | 0/2 | - |
| 09-mixed-document | xelatex-lm | pipeline | 1/1 | ok | 1.6197 | 0.9787 | (0,1) moderate | 1.3811 | 0.9857 | 255 | 0.0796 | 0.0116 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | lualatex | de1020c | 1/1 | ok | 4.1955 | 0.9446 | (0,0.5) weak | 4.1609 | 0.9459 | 255 | 0.0939 | 0.0277 | -/-/- | - | - | - | - | 3/2 | - |
| 10-unicode-paragraph | lualatex | main | 1/1 | recovered | 4.1731 | 0.9459 | (0.5,0.5) weak | 4.1034 | 0.9474 | 255 | 0.0942 | 0.0275 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | pipeline | 1/1 | recovered | 4.0180 | 0.9402 | (0.5,0.5) weak | 3.9210 | 0.9436 | 255 | 0.0976 | 0.0273 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex-lm | de1020c | 1/1 | ok | 4.1704 | 0.9415 | (0,0) | 4.1704 | 0.9415 | 255 | 0.0941 | 0.0279 | -/-/- | - | - | - | - | 0/2 | - |
| 10-unicode-paragraph | lualatex-lm | main | 1/1 | recovered | 4.1546 | 0.9419 | (-1,0.5) weak | 4.1319 | 0.9425 | 255 | 0.0944 | 0.0278 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 2.3013 | 0.9735 | (0,0.5) moderate | 1.9926 | 0.9817 | 255 | 0.0975 | 0.0186 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | pdflatex | de1020c | 1/1 | ok | 4.2305 | 0.9443 | (0,0.5) weak | 4.1964 | 0.9453 | 255 | 0.0939 | 0.0278 | -/-/- | - | - | - | - | 3/2 | [p1](images/10-unicode-paragraph/pdflatex-de1020c-native-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex | main | 1/1 | recovered | 4.1598 | 0.9461 | (0.5,0.5) weak | 4.0936 | 0.9475 | 255 | 0.0942 | 0.0275 | -/-/- | - | - | - | - | 3/1 | [p1](images/10-unicode-paragraph/pdflatex-main-native-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex | pipeline | 1/1 | recovered | 4.0222 | 0.9403 | (0.5,0.5) weak | 3.9256 | 0.9435 | 255 | 0.0976 | 0.0273 | -/-/- | - | - | - | - | 3/0 | [p1](images/10-unicode-paragraph/pdflatex-pipeline-native-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 4.1850 | 0.9412 | (0,0) | 4.1850 | 0.9412 | 255 | 0.0941 | 0.0280 | -/-/- | - | - | - | - | 0/2 | [p1](images/10-unicode-paragraph/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | main | 1/1 | recovered | 4.1699 | 0.9416 | (-1,0.5) weak | 4.1455 | 0.9422 | 255 | 0.0944 | 0.0279 | -/-/- | - | - | - | - | 0/1 | [p1](images/10-unicode-paragraph/pdflatex-lm-main-native-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 2.3729 | 0.9717 | (0,0.5) moderate | 2.0550 | 0.9803 | 255 | 0.0975 | 0.0190 | -/-/- | - | - | - | - | 0/0 | [p1](images/10-unicode-paragraph/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 10-unicode-paragraph | xelatex | de1020c | 1/1 | ok | 4.2336 | 0.9441 | (0,0.5) weak | 4.1967 | 0.9454 | 255 | 0.0939 | 0.0279 | -/-/- | - | - | - | - | 3/2 | - |
| 10-unicode-paragraph | xelatex | main | 1/1 | recovered | 4.1689 | 0.9460 | (0.5,0.5) weak | 4.0923 | 0.9476 | 255 | 0.0942 | 0.0276 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | xelatex | pipeline | 1/1 | recovered | 4.0346 | 0.9399 | (0.5,0.5) weak | 3.9405 | 0.9432 | 255 | 0.0976 | 0.0274 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | xelatex-lm | de1020c | 1/1 | ok | 4.1704 | 0.9414 | (0,0) | 4.1704 | 0.9414 | 255 | 0.0941 | 0.0279 | -/-/- | - | - | - | - | 0/2 | - |
| 10-unicode-paragraph | xelatex-lm | main | 1/1 | recovered | 4.1546 | 0.9419 | (-1,0.5) weak | 4.1317 | 0.9425 | 255 | 0.0944 | 0.0278 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 2.3018 | 0.9735 | (0,0.5) moderate | 1.9932 | 0.9817 | 255 | 0.0975 | 0.0186 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | de1020c | 1/1 | recovered | 1.9239 | 0.9663 | (18,-48.5) weak | 1.8287 | 0.9706 | 255 | 0.0440 | 0.0118 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | main | 1/1 | ok | 1.8974 | 0.9684 | (-22.5,-8) moderate | 1.5021 | 0.9785 | 255 | 0.0445 | 0.0116 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | pipeline | 1/1 | ok | 1.6906 | 0.9696 | (-15.5,-19) moderate | 1.5926 | 0.9732 | 255 | 0.0398 | 0.0109 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | de1020c | 1/1 | recovered | 1.8009 | 0.9657 | (-2.5,-48.5) moderate | 1.6972 | 0.9705 | 255 | 0.0435 | 0.0115 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | main | 1/1 | ok | 1.7827 | 0.9679 | (-30.5,-8) moderate | 1.5747 | 0.9750 | 255 | 0.0446 | 0.0113 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | pipeline | 1/1 | ok | 1.5480 | 0.9695 | (-17,-19.5) moderate | 1.3197 | 0.9766 | 255 | 0.0397 | 0.0104 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | de1020c | 1/1 | recovered | 1.9227 | 0.9665 | (6.5,-48.5) weak | 1.8281 | 0.9708 | 255 | 0.0440 | 0.0118 | -/-/- | - | - | - | - | 0/0 | [p1](images/11-nested-lists/pdflatex-de1020c-native-p1-overlay.png) |
| 11-nested-lists | pdflatex | main | 1/1 | ok | 1.8871 | 0.9689 | (-21.5,-8) moderate | 1.5294 | 0.9781 | 255 | 0.0445 | 0.0116 | -/-/- | - | - | - | - | 0/0 | [p1](images/11-nested-lists/pdflatex-main-native-p1-overlay.png) |
| 11-nested-lists | pdflatex | pipeline | 1/1 | ok | 1.6917 | 0.9698 | (-14.5,-19) moderate | 1.5893 | 0.9732 | 255 | 0.0398 | 0.0109 | -/-/- | - | - | - | - | 0/0 | [p1](images/11-nested-lists/pdflatex-pipeline-native-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | de1020c | 1/1 | recovered | 1.8008 | 0.9657 | (23.5,-48.5) moderate | 1.7075 | 0.9701 | 255 | 0.0435 | 0.0115 | -/-/- | - | - | - | - | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | main | 1/1 | ok | 1.7846 | 0.9679 | (-30,-8) moderate | 1.5781 | 0.9749 | 255 | 0.0446 | 0.0113 | -/-/- | - | - | - | - | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-main-native-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | pipeline | 1/1 | ok | 1.5382 | 0.9696 | (-16.5,-19.5) moderate | 1.3312 | 0.9763 | 255 | 0.0397 | 0.0104 | -/-/- | - | - | - | - | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 11-nested-lists | xelatex | de1020c | 1/1 | recovered | 1.9241 | 0.9663 | (18,-48.5) weak | 1.8282 | 0.9706 | 255 | 0.0440 | 0.0118 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | main | 1/1 | ok | 1.8974 | 0.9684 | (-22.5,-8) moderate | 1.5013 | 0.9785 | 255 | 0.0445 | 0.0116 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | pipeline | 1/1 | ok | 1.6905 | 0.9696 | (-15.5,-19) moderate | 1.5929 | 0.9732 | 255 | 0.0398 | 0.0109 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | de1020c | 1/1 | recovered | 1.8009 | 0.9657 | (-2.5,-48.5) moderate | 1.6973 | 0.9705 | 255 | 0.0435 | 0.0115 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | main | 1/1 | ok | 1.7827 | 0.9679 | (-30.5,-8) moderate | 1.5748 | 0.9750 | 255 | 0.0446 | 0.0113 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | pipeline | 1/1 | ok | 1.5479 | 0.9695 | (-17,-19.5) moderate | 1.3195 | 0.9766 | 255 | 0.0397 | 0.0104 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | de1020c | 1/1 | ok | 18.9529 | 0.7328 | (0,0.5) weak | 18.9424 | 0.7304 | 255 | 0.3510 | 0.1202 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | main | 1/1 | ok | 18.1988 | 0.7528 | (-0.5,18) weak | 18.0956 | 0.7576 | 255 | 0.3164 | 0.1166 | -/-/- | - | - | - | - | 0/2 | - |
| 12-justified-paragraphs | lualatex | pipeline | 1/1 | ok | 15.0506 | 0.7777 | (0,15) weak | 14.7579 | 0.7853 | 255 | 0.3527 | 0.0998 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | de1020c | 1/1 | ok | 17.3414 | 0.7451 | (-1,3) weak | 17.2815 | 0.7456 | 255 | 0.3506 | 0.1140 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | main | 1/1 | ok | 17.4246 | 0.7390 | (-1,-25.5) weak | 17.3516 | 0.7440 | 255 | 0.3200 | 0.1148 | -/-/- | - | - | - | - | 0/2 | - |
| 12-justified-paragraphs | lualatex-lm | pipeline | 1/1 | ok | 7.9394 | 0.9227 | (0,0.5) moderate | 7.1656 | 0.9441 | 254 | 0.3525 | 0.0649 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | de1020c | 1/1 | ok | 18.9317 | 0.7342 | (-3,0.5) weak | 18.9192 | 0.7320 | 255 | 0.3509 | 0.1200 | -/-/- | - | - | - | - | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-de1020c-native-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex | main | 1/1 | ok | 18.2307 | 0.7529 | (-4,18) weak | 18.0124 | 0.7597 | 255 | 0.3164 | 0.1166 | -/-/- | - | - | - | - | 0/2 | [p1](images/12-justified-paragraphs/pdflatex-main-native-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex | pipeline | 1/1 | ok | 15.0547 | 0.7785 | (0.5,15) weak | 14.7432 | 0.7868 | 255 | 0.3527 | 0.0998 | -/-/- | - | - | - | - | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-pipeline-native-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | de1020c | 1/1 | ok | 17.3395 | 0.7452 | (-8.5,3) weak | 17.2920 | 0.7441 | 255 | 0.3506 | 0.1140 | -/-/- | - | - | - | - | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | main | 1/1 | ok | 17.4244 | 0.7390 | (-1,-25.5) weak | 17.3510 | 0.7440 | 255 | 0.3200 | 0.1149 | -/-/- | - | - | - | - | 0/2 | [p1](images/12-justified-paragraphs/pdflatex-lm-main-native-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | pipeline | 1/1 | ok | 7.9555 | 0.9224 | (0,0.5) moderate | 7.1827 | 0.9438 | 254 | 0.3525 | 0.0650 | -/-/- | - | - | - | - | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 12-justified-paragraphs | xelatex | de1020c | 1/1 | ok | 18.9405 | 0.7335 | (0,0) | 18.9405 | 0.7335 | 255 | 0.3510 | 0.1202 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | main | 1/1 | ok | 18.2756 | 0.7516 | (-0.5,18) weak | 18.1324 | 0.7572 | 255 | 0.3164 | 0.1169 | -/-/- | - | - | - | - | 0/2 | - |
| 12-justified-paragraphs | xelatex | pipeline | 1/1 | ok | 15.0775 | 0.7775 | (0,15) weak | 14.7481 | 0.7858 | 255 | 0.3526 | 0.1000 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | de1020c | 1/1 | ok | 17.3415 | 0.7451 | (-4.5,3) weak | 17.2817 | 0.7455 | 255 | 0.3506 | 0.1140 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | main | 1/1 | ok | 17.4250 | 0.7390 | (-1,-25.5) weak | 17.3514 | 0.7440 | 255 | 0.3200 | 0.1148 | -/-/- | - | - | - | - | 0/2 | - |
| 12-justified-paragraphs | xelatex-lm | pipeline | 1/1 | ok | 7.9427 | 0.9226 | (0,0.5) moderate | 7.1686 | 0.9441 | 254 | 0.3525 | 0.0649 | -/-/- | - | - | - | - | 0/0 | - |
| 13-math-display-rich | lualatex | de1020c | 1/1 | recovered | 0.6386 | 0.9894 | (0,3.5) moderate | 0.5886 | 0.9909 | 255 | 0.0244 | 0.0043 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex | main | 1/1 | recovered | 0.6869 | 0.9885 | (-1,3.5) moderate | 0.6263 | 0.9902 | 255 | 0.0261 | 0.0046 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex | pipeline | 1/1 | recovered | 0.7678 | 0.9860 | (5,2) weak | 0.7369 | 0.9871 | 255 | 0.0325 | 0.0050 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex-lm | de1020c | 1/1 | recovered | 0.6881 | 0.9883 | (0,4) weak | 0.6742 | 0.9890 | 255 | 0.0244 | 0.0046 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | lualatex-lm | main | 1/1 | recovered | 0.7164 | 0.9877 | (-1.5,3.5) weak | 0.6871 | 0.9886 | 255 | 0.0262 | 0.0047 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | lualatex-lm | pipeline | 1/1 | recovered | 0.7140 | 0.9866 | (4,2) moderate | 0.6295 | 0.9894 | 255 | 0.0325 | 0.0047 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | pdflatex | de1020c | 1/1 | recovered | 0.6555 | 0.9892 | (1,3.5) weak | 0.6264 | 0.9901 | 255 | 0.0244 | 0.0044 | -/-/- | - | - | - | - | 2/2 | [p1](images/13-math-display-rich/pdflatex-de1020c-native-p1-overlay.png) |
| 13-math-display-rich | pdflatex | main | 1/1 | recovered | 0.6677 | 0.9888 | (0,3.5) moderate | 0.6055 | 0.9904 | 255 | 0.0261 | 0.0045 | -/-/- | - | - | - | - | 2/2 | [p1](images/13-math-display-rich/pdflatex-main-native-p1-overlay.png) |
| 13-math-display-rich | pdflatex | pipeline | 1/1 | recovered | 0.7702 | 0.9860 | (4.5,2) moderate | 0.7188 | 0.9877 | 255 | 0.0325 | 0.0050 | -/-/- | - | - | - | - | 2/2 | [p1](images/13-math-display-rich/pdflatex-pipeline-native-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | de1020c | 1/1 | recovered | 0.6919 | 0.9884 | (0,4) weak | 0.6715 | 0.9890 | 255 | 0.0244 | 0.0046 | -/-/- | - | - | - | - | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | main | 1/1 | recovered | 0.7209 | 0.9877 | (-1.5,4) weak | 0.6908 | 0.9887 | 255 | 0.0262 | 0.0047 | -/-/- | - | - | - | - | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-main-native-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | pipeline | 1/1 | recovered | 0.7171 | 0.9867 | (4,2) moderate | 0.6390 | 0.9893 | 255 | 0.0325 | 0.0047 | -/-/- | - | - | - | - | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 13-math-display-rich | xelatex | de1020c | 1/1 | recovered | 0.6497 | 0.9892 | (0.5,3.5) weak | 0.6183 | 0.9904 | 255 | 0.0244 | 0.0044 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex | main | 1/1 | recovered | 0.6664 | 0.9888 | (0,3.5) moderate | 0.6076 | 0.9904 | 255 | 0.0261 | 0.0045 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex | pipeline | 1/1 | recovered | 0.7685 | 0.9860 | (5,2) moderate | 0.7280 | 0.9872 | 255 | 0.0325 | 0.0050 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex-lm | de1020c | 1/1 | recovered | 0.6882 | 0.9883 | (0,4) weak | 0.6742 | 0.9890 | 255 | 0.0244 | 0.0046 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | xelatex-lm | main | 1/1 | recovered | 0.7164 | 0.9877 | (-1.5,3.5) weak | 0.6871 | 0.9886 | 255 | 0.0262 | 0.0047 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | xelatex-lm | pipeline | 1/1 | recovered | 0.7141 | 0.9866 | (4,2) moderate | 0.6296 | 0.9894 | 255 | 0.0325 | 0.0047 | -/-/- | - | - | - | - | 4/2 | - |
| 14-math-inline-dense | lualatex | de1020c | 1/1 | recovered | 1.6880 | 0.9690 | (-34.5,14.5) moderate | 1.3274 | 0.9759 | 255 | 0.0531 | 0.0107 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex | main | 1/1 | recovered | 1.6842 | 0.9690 | (-34.5,14.5) moderate | 1.3134 | 0.9760 | 255 | 0.0534 | 0.0107 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | lualatex | pipeline | 1/1 | recovered | 1.3813 | 0.9759 | (14,2) weak | 1.3532 | 0.9773 | 255 | 0.0487 | 0.0092 | -/-/- | - | - | - | - | 2/3 | - |
| 14-math-inline-dense | lualatex-lm | de1020c | 1/1 | recovered | 1.6163 | 0.9687 | (-18.5,14.5) moderate | 1.5039 | 0.9712 | 255 | 0.0532 | 0.0105 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | main | 1/1 | recovered | 1.6168 | 0.9688 | (1.5,14.5) moderate | 1.5220 | 0.9710 | 255 | 0.0535 | 0.0105 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | lualatex-lm | pipeline | 1/1 | recovered | 1.2726 | 0.9771 | (0,2) moderate | 1.0972 | 0.9835 | 255 | 0.0487 | 0.0088 | -/-/- | - | - | - | - | 2/3 | - |
| 14-math-inline-dense | pdflatex | de1020c | 1/1 | recovered | 1.6839 | 0.9693 | (-34,14.5) moderate | 1.3394 | 0.9757 | 255 | 0.0531 | 0.0107 | -/-/- | - | - | - | - | 2/1 | [p1](images/14-math-inline-dense/pdflatex-de1020c-native-p1-overlay.png) |
| 14-math-inline-dense | pdflatex | main | 1/1 | recovered | 1.6799 | 0.9693 | (-34,14.5) moderate | 1.3181 | 0.9761 | 255 | 0.0534 | 0.0107 | -/-/- | - | - | - | - | 2/2 | [p1](images/14-math-inline-dense/pdflatex-main-native-p1-overlay.png) |
| 14-math-inline-dense | pdflatex | pipeline | 1/1 | recovered | 1.3796 | 0.9759 | (0.5,2) weak | 1.3367 | 0.9786 | 255 | 0.0487 | 0.0092 | -/-/- | - | - | - | - | 2/3 | [p1](images/14-math-inline-dense/pdflatex-pipeline-native-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | de1020c | 1/1 | recovered | 1.6159 | 0.9687 | (-18.5,14.5) moderate | 1.5034 | 0.9712 | 255 | 0.0532 | 0.0105 | -/-/- | - | - | - | - | 2/1 | [p1](images/14-math-inline-dense/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | main | 1/1 | recovered | 1.6165 | 0.9688 | (1.5,14.5) moderate | 1.5221 | 0.9710 | 255 | 0.0535 | 0.0105 | -/-/- | - | - | - | - | 2/2 | [p1](images/14-math-inline-dense/pdflatex-lm-main-native-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | pipeline | 1/1 | recovered | 1.2742 | 0.9771 | (0,2) moderate | 1.1000 | 0.9834 | 255 | 0.0487 | 0.0088 | -/-/- | - | - | - | - | 2/3 | [p1](images/14-math-inline-dense/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 14-math-inline-dense | xelatex | de1020c | 1/1 | recovered | 1.6882 | 0.9690 | (-34.5,14.5) moderate | 1.3264 | 0.9759 | 255 | 0.0531 | 0.0107 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex | main | 1/1 | recovered | 1.6843 | 0.9690 | (-34.5,14.5) moderate | 1.3121 | 0.9760 | 255 | 0.0534 | 0.0107 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | xelatex | pipeline | 1/1 | recovered | 1.3817 | 0.9759 | (14,2) weak | 1.3539 | 0.9773 | 255 | 0.0487 | 0.0092 | -/-/- | - | - | - | - | 2/3 | - |
| 14-math-inline-dense | xelatex-lm | de1020c | 1/1 | recovered | 1.6162 | 0.9687 | (-18.5,14.5) moderate | 1.5039 | 0.9712 | 255 | 0.0532 | 0.0105 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | main | 1/1 | recovered | 1.6167 | 0.9688 | (1.5,14.5) moderate | 1.5220 | 0.9710 | 255 | 0.0535 | 0.0105 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | xelatex-lm | pipeline | 1/1 | recovered | 1.2726 | 0.9771 | (0,2) moderate | 1.0972 | 0.9835 | 255 | 0.0487 | 0.0088 | -/-/- | - | - | - | - | 2/3 | - |
| 15-three-page-sections | lualatex | de1020c | 3/1 | recovered | 33.3245 | 0.5304 | (0,20.5) moderate | 31.3780 | 0.5713 | 255 | 0.5690 | 0.2112 | -/-/- | - | - | - | - | 0/1 | - |
| 15-three-page-sections | lualatex | main | 3/1 | recovered | 33.3084 | 0.5316 | (-0.5,20.5) moderate | 31.5096 | 0.5713 | 255 | 0.5729 | 0.2117 | -/-/- | - | - | - | - | 0/5 | - |
| 15-three-page-sections | lualatex | pipeline | 3/1 | recovered | 23.8672 | 0.6725 | (0,0) | 23.8672 | 0.6725 | 255 | 0.5397 | 0.1580 | -/-/- | - | - | - | - | 0/1 | - |
| 15-three-page-sections | lualatex-lm | de1020c | 3/1 | recovered | 31.3719 | 0.5231 | (0,21) moderate | 29.2677 | 0.5722 | 255 | 0.5680 | 0.2054 | -/-/- | - | - | - | - | 1/1 | - |
| 15-three-page-sections | lualatex-lm | main | 3/1 | recovered | 31.4299 | 0.5237 | (-0.5,21) moderate | 29.2903 | 0.5737 | 255 | 0.5720 | 0.2059 | -/-/- | - | - | - | - | 1/5 | - |
| 15-three-page-sections | lualatex-lm | pipeline | 3/1 | recovered | 13.6802 | 0.8676 | (0,0.5) moderate | 12.0028 | 0.9141 | 255 | 0.5394 | 0.1097 | -/-/- | - | - | - | - | 1/1 | - |
| 15-three-page-sections | pdflatex | de1020c | 3/1 | recovered | 33.7507 | 0.5228 | (0,35) moderate | 30.7943 | 0.5843 | 255 | 0.5691 | 0.2137 | -/-/- | - | - | - | - | 0/1 | [p1](images/15-three-page-sections/pdflatex-de1020c-native-p1-overlay.png) |
| 15-three-page-sections | pdflatex | main | 3/1 | recovered | 33.8056 | 0.5227 | (0,35) moderate | 30.9834 | 0.5823 | 255 | 0.5731 | 0.2143 | -/-/- | - | - | - | - | 0/5 | [p1](images/15-three-page-sections/pdflatex-main-native-p1-overlay.png) |
| 15-three-page-sections | pdflatex | pipeline | 3/1 | recovered | 23.5176 | 0.6833 | (0,0) | 23.5176 | 0.6833 | 255 | 0.5397 | 0.1561 | -/-/- | - | - | - | - | 0/1 | [p1](images/15-three-page-sections/pdflatex-pipeline-native-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | de1020c | 3/1 | recovered | 31.3698 | 0.5230 | (0,21) moderate | 29.2623 | 0.5723 | 255 | 0.5681 | 0.2053 | -/-/- | - | - | - | - | 1/1 | [p1](images/15-three-page-sections/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | main | 3/1 | recovered | 31.4289 | 0.5236 | (-0.5,21) moderate | 29.2856 | 0.5737 | 255 | 0.5719 | 0.2059 | -/-/- | - | - | - | - | 1/5 | [p1](images/15-three-page-sections/pdflatex-lm-main-native-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | pipeline | 3/1 | recovered | 13.7894 | 0.8646 | (0,0.5) moderate | 12.0117 | 0.9143 | 255 | 0.5394 | 0.1104 | -/-/- | - | - | - | - | 1/1 | [p1](images/15-three-page-sections/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 15-three-page-sections | xelatex | de1020c | 3/1 | recovered | 33.3070 | 0.5308 | (0,20.5) moderate | 31.3799 | 0.5715 | 255 | 0.5690 | 0.2115 | -/-/- | - | - | - | - | 0/1 | - |
| 15-three-page-sections | xelatex | main | 3/1 | recovered | 33.3226 | 0.5314 | (-0.5,20.5) moderate | 31.5082 | 0.5712 | 255 | 0.5729 | 0.2121 | -/-/- | - | - | - | - | 0/5 | - |
| 15-three-page-sections | xelatex | pipeline | 3/1 | recovered | 23.8916 | 0.6715 | (0,0) | 23.8916 | 0.6715 | 255 | 0.5397 | 0.1583 | -/-/- | - | - | - | - | 0/1 | - |
| 15-three-page-sections | xelatex-lm | de1020c | 3/1 | recovered | 31.3717 | 0.5231 | (0,21) moderate | 29.2676 | 0.5722 | 255 | 0.5680 | 0.2054 | -/-/- | - | - | - | - | 1/1 | - |
| 15-three-page-sections | xelatex-lm | main | 3/1 | recovered | 31.4299 | 0.5237 | (-0.5,21) moderate | 29.2902 | 0.5736 | 255 | 0.5720 | 0.2059 | -/-/- | - | - | - | - | 1/5 | - |
| 15-three-page-sections | xelatex-lm | pipeline | 3/1 | recovered | 13.6871 | 0.8675 | (0,0.5) moderate | 12.0100 | 0.9141 | 255 | 0.5394 | 0.1097 | -/-/- | - | - | - | - | 1/1 | - |
| 16-heading-page-break | lualatex | de1020c | 2/1 | ok | 37.7606 | 0.4761 | (0,2) weak | 37.6280 | 0.4790 | 255 | 0.6343 | 0.2407 | -/-/- | - | - | - | - | 0/2 | - |
| 16-heading-page-break | lualatex | main | 2/1 | ok | 39.5859 | 0.4606 | (0,0) | 39.5859 | 0.4606 | 255 | 0.6490 | 0.2536 | -/-/- | - | - | - | - | 0/3 | - |
| 16-heading-page-break | lualatex | pipeline | 2/1 | ok | 30.7523 | 0.5623 | (0,0.5) weak | 29.9541 | 0.5841 | 255 | 0.6678 | 0.2041 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex-lm | de1020c | 2/1 | ok | 34.3279 | 0.4929 | (0,0) | 34.3279 | 0.4929 | 255 | 0.6335 | 0.2252 | -/-/- | - | - | - | - | 0/2 | - |
| 16-heading-page-break | lualatex-lm | main | 2/1 | ok | 36.3179 | 0.4695 | (-1,-26.5) weak | 36.3082 | 0.4710 | 255 | 0.6490 | 0.2385 | -/-/- | - | - | - | - | 0/3 | - |
| 16-heading-page-break | lualatex-lm | pipeline | 2/1 | ok | 16.2749 | 0.8433 | (0,0.5) moderate | 14.6808 | 0.8871 | 255 | 0.6675 | 0.1326 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex | de1020c | 2/1 | ok | 37.7343 | 0.4770 | (0,2) weak | 37.5275 | 0.4823 | 255 | 0.6342 | 0.2405 | -/-/- | - | - | - | - | 0/2 | [p1](images/16-heading-page-break/pdflatex-de1020c-native-p1-overlay.png) |
| 16-heading-page-break | pdflatex | main | 2/1 | ok | 39.6113 | 0.4610 | (0,2.5) weak | 39.5335 | 0.4626 | 255 | 0.6490 | 0.2534 | -/-/- | - | - | - | - | 0/3 | [p1](images/16-heading-page-break/pdflatex-main-native-p1-overlay.png) |
| 16-heading-page-break | pdflatex | pipeline | 2/1 | ok | 30.6738 | 0.5639 | (0,0.5) weak | 29.8708 | 0.5863 | 255 | 0.6678 | 0.2040 | -/-/- | - | - | - | - | 0/0 | [p1](images/16-heading-page-break/pdflatex-pipeline-native-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | de1020c | 2/1 | ok | 34.3260 | 0.4929 | (0,0) | 34.3260 | 0.4929 | 255 | 0.6335 | 0.2253 | -/-/- | - | - | - | - | 0/2 | [p1](images/16-heading-page-break/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | main | 2/1 | ok | 36.3176 | 0.4695 | (-1,-26.5) weak | 36.3076 | 0.4710 | 255 | 0.6490 | 0.2386 | -/-/- | - | - | - | - | 0/3 | [p1](images/16-heading-page-break/pdflatex-lm-main-native-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | pipeline | 2/1 | ok | 16.3017 | 0.8428 | (0,0.5) moderate | 14.7096 | 0.8866 | 255 | 0.6675 | 0.1327 | -/-/- | - | - | - | - | 0/0 | [p1](images/16-heading-page-break/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 16-heading-page-break | xelatex | de1020c | 2/1 | ok | 37.7872 | 0.4760 | (-3,1) weak | 37.5767 | 0.4761 | 255 | 0.6344 | 0.2414 | -/-/- | - | - | - | - | 0/2 | - |
| 16-heading-page-break | xelatex | main | 2/1 | ok | 39.5602 | 0.4610 | (0,0) | 39.5602 | 0.4610 | 255 | 0.6490 | 0.2539 | -/-/- | - | - | - | - | 0/3 | - |
| 16-heading-page-break | xelatex | pipeline | 2/1 | ok | 30.7483 | 0.5618 | (2.5,0.5) weak | 29.9578 | 0.5821 | 255 | 0.6678 | 0.2043 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex-lm | de1020c | 2/1 | ok | 34.3287 | 0.4929 | (0,0) | 34.3287 | 0.4929 | 255 | 0.6335 | 0.2253 | -/-/- | - | - | - | - | 0/2 | - |
| 16-heading-page-break | xelatex-lm | main | 2/1 | ok | 36.3186 | 0.4695 | (-1,-26.5) weak | 36.3084 | 0.4710 | 255 | 0.6490 | 0.2385 | -/-/- | - | - | - | - | 0/3 | - |
| 16-heading-page-break | xelatex-lm | pipeline | 2/1 | ok | 16.2817 | 0.8432 | (0,0.5) moderate | 14.6886 | 0.8870 | 255 | 0.6676 | 0.1326 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex | de1020c | 1/1 | ok | 1.1601 | 0.9845 | (-1.5,0.5) moderate | 1.0494 | 0.9862 | 255 | 0.0383 | 0.0074 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex | main | 1/1 | ok | 1.1632 | 0.9845 | (-8,0.5) moderate | 1.0769 | 0.9855 | 255 | 0.0389 | 0.0075 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex | pipeline | 1/1 | ok | 1.1477 | 0.9825 | (7.5,0.5) weak | 1.1085 | 0.9826 | 255 | 0.0398 | 0.0076 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | de1020c | 1/1 | ok | 1.1626 | 0.9822 | (12.5,0.5) weak | 1.1589 | 0.9821 | 255 | 0.0385 | 0.0077 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | main | 1/1 | ok | 1.1707 | 0.9821 | (0,0.5) weak | 1.1356 | 0.9827 | 255 | 0.0391 | 0.0078 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | pipeline | 1/1 | ok | 0.6823 | 0.9905 | (0,0.5) moderate | 0.5154 | 0.9952 | 254 | 0.0398 | 0.0054 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | de1020c | 1/1 | ok | 1.1248 | 0.9854 | (-1,0.5) moderate | 1.0536 | 0.9864 | 255 | 0.0383 | 0.0073 | -/-/- | - | - | - | - | 0/0 | [p1](images/17-apostrophes/pdflatex-de1020c-native-p1-overlay.png) |
| 17-apostrophes | pdflatex | main | 1/1 | ok | 1.1266 | 0.9853 | (-8,0.5) weak | 1.0848 | 0.9853 | 255 | 0.0389 | 0.0073 | -/-/- | - | - | - | - | 0/0 | [p1](images/17-apostrophes/pdflatex-main-native-p1-overlay.png) |
| 17-apostrophes | pdflatex | pipeline | 1/1 | ok | 1.1442 | 0.9827 | (-12,0.5) weak | 1.1144 | 0.9828 | 255 | 0.0398 | 0.0076 | -/-/- | - | - | - | - | 0/0 | [p1](images/17-apostrophes/pdflatex-pipeline-native-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | de1020c | 1/1 | ok | 1.1628 | 0.9822 | (-0.5,0.5) weak | 1.1216 | 0.9829 | 255 | 0.0385 | 0.0077 | -/-/- | - | - | - | - | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | main | 1/1 | ok | 1.1708 | 0.9821 | (0,0.5) weak | 1.1356 | 0.9827 | 255 | 0.0391 | 0.0078 | -/-/- | - | - | - | - | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-main-native-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | pipeline | 1/1 | ok | 0.6816 | 0.9905 | (0,0.5) moderate | 0.5148 | 0.9952 | 254 | 0.0398 | 0.0054 | -/-/- | - | - | - | - | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 17-apostrophes | xelatex | de1020c | 1/1 | ok | 1.1305 | 0.9850 | (-1,0.5) moderate | 1.0527 | 0.9864 | 255 | 0.0383 | 0.0073 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | main | 1/1 | ok | 1.1364 | 0.9850 | (0.5,0.5) weak | 1.1214 | 0.9847 | 255 | 0.0389 | 0.0073 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | pipeline | 1/1 | ok | 1.1429 | 0.9826 | (-12,0.5) weak | 1.1130 | 0.9827 | 255 | 0.0398 | 0.0076 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | de1020c | 1/1 | ok | 1.1625 | 0.9822 | (12.5,0.5) weak | 1.1587 | 0.9821 | 255 | 0.0385 | 0.0077 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | main | 1/1 | ok | 1.1707 | 0.9821 | (0,0.5) weak | 1.1356 | 0.9827 | 255 | 0.0391 | 0.0078 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | pipeline | 1/1 | ok | 0.6826 | 0.9905 | (0,0.5) moderate | 0.5158 | 0.9952 | 254 | 0.0398 | 0.0054 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | de1020c | 1/1 | ok | 1.8643 | 0.9748 | (0,0.5) moderate | 1.7572 | 0.9770 | 255 | 0.0493 | 0.0121 | -/-/- | - | - | - | - | 0/1 | - |
| 18-ligatures | lualatex | main | 1/1 | ok | 1.7577 | 0.9770 | (-0.5,0.5) moderate | 1.5121 | 0.9824 | 255 | 0.0498 | 0.0115 | -/-/- | - | - | - | - | 0/1 | - |
| 18-ligatures | lualatex | pipeline | 1/1 | ok | 1.7578 | 0.9748 | (0.5,0.5) moderate | 1.6535 | 0.9766 | 255 | 0.0504 | 0.0117 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | de1020c | 1/1 | ok | 1.7870 | 0.9755 | (-19,0.5) weak | 1.7078 | 0.9768 | 255 | 0.0493 | 0.0119 | -/-/- | - | - | - | - | 0/1 | - |
| 18-ligatures | lualatex-lm | main | 1/1 | ok | 1.8208 | 0.9741 | (-5.5,0.5) weak | 1.7374 | 0.9756 | 255 | 0.0499 | 0.0121 | -/-/- | - | - | - | - | 0/1 | - |
| 18-ligatures | lualatex-lm | pipeline | 1/1 | ok | 1.2301 | 0.9838 | (0,0.5) moderate | 1.0110 | 0.9894 | 255 | 0.0504 | 0.0093 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | de1020c | 1/1 | ok | 1.8033 | 0.9762 | (0,0.5) moderate | 1.6899 | 0.9784 | 255 | 0.0493 | 0.0118 | -/-/- | - | - | - | - | 0/1 | [p1](images/18-ligatures/pdflatex-de1020c-native-p1-overlay.png) |
| 18-ligatures | pdflatex | main | 1/1 | ok | 1.7593 | 0.9771 | (-2.5,0.5) moderate | 1.5092 | 0.9820 | 255 | 0.0498 | 0.0115 | -/-/- | - | - | - | - | 0/1 | [p1](images/18-ligatures/pdflatex-main-native-p1-overlay.png) |
| 18-ligatures | pdflatex | pipeline | 1/1 | ok | 1.7367 | 0.9755 | (6,0.5) moderate | 1.6406 | 0.9766 | 255 | 0.0504 | 0.0116 | -/-/- | - | - | - | - | 0/0 | [p1](images/18-ligatures/pdflatex-pipeline-native-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | de1020c | 1/1 | ok | 1.7879 | 0.9755 | (-19,0.5) weak | 1.7086 | 0.9767 | 255 | 0.0493 | 0.0119 | -/-/- | - | - | - | - | 0/1 | [p1](images/18-ligatures/pdflatex-lm-de1020c-native-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | main | 1/1 | ok | 1.8203 | 0.9741 | (-5.5,0.5) weak | 1.7380 | 0.9756 | 255 | 0.0499 | 0.0121 | -/-/- | - | - | - | - | 0/1 | [p1](images/18-ligatures/pdflatex-lm-main-native-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | pipeline | 1/1 | ok | 1.2332 | 0.9838 | (0,0.5) moderate | 1.0151 | 0.9893 | 255 | 0.0504 | 0.0093 | -/-/- | - | - | - | - | 0/0 | [p1](images/18-ligatures/pdflatex-lm-pipeline-native-p1-overlay.png) |
| 18-ligatures | xelatex | de1020c | 1/1 | ok | 1.8146 | 0.9756 | (0,0.5) moderate | 1.7027 | 0.9781 | 255 | 0.0493 | 0.0119 | -/-/- | - | - | - | - | 0/1 | - |
| 18-ligatures | xelatex | main | 1/1 | ok | 1.8114 | 0.9761 | (-1,0.5) moderate | 1.5521 | 0.9814 | 255 | 0.0498 | 0.0118 | -/-/- | - | - | - | - | 0/1 | - |
| 18-ligatures | xelatex | pipeline | 1/1 | ok | 1.7293 | 0.9752 | (0,0.5) moderate | 1.6159 | 0.9772 | 255 | 0.0504 | 0.0116 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | de1020c | 1/1 | ok | 1.7869 | 0.9755 | (-19,0.5) weak | 1.7077 | 0.9768 | 255 | 0.0493 | 0.0119 | -/-/- | - | - | - | - | 0/1 | - |
| 18-ligatures | xelatex-lm | main | 1/1 | ok | 1.8207 | 0.9741 | (-5.5,0.5) weak | 1.7373 | 0.9756 | 255 | 0.0499 | 0.0121 | -/-/- | - | - | - | - | 0/1 | - |
| 18-ligatures | xelatex-lm | pipeline | 1/1 | ok | 1.2305 | 0.9838 | (0,0.5) moderate | 1.0115 | 0.9894 | 255 | 0.0504 | 0.0093 | -/-/- | - | - | - | - | 0/0 | - |

## Per-fixture diagnostic details (export side)

### 01-plain-paragraph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935537, 648, 477, 427, 319, 237, 160, 146, 149, 113, 138, 79, 113, 76, 54, 143]`; ink px ref/ours 2442/2421 (ratio 0.9914); SSIM blocks <0.9: 131/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.95, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1575, differing 0.002279, SSIM₈ 0.998 (raw 0.1575, 0.002279, 0.998)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9969→0.9969 / 0.2517→0.2517; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `This` dx -0.75 dy 0.46; `is` dx -0.65 dy 0.46; `one` dx -0.62 dy 0.46

### 01-plain-paragraph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934633, 486, 426, 392, 398, 266, 276, 245, 196, 204, 148, 170, 179, 189, 168, 440]`; ink px ref/ours 2442/2408 (ratio 0.9861); SSIM blocks <0.9: 176/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.91, -0.01] pt); confidence strong (shift explains 42% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1503, differing 0.002291, SSIM₈ 0.9983 (raw 0.2612, 0.002629, 0.9965)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9944→0.9973 / 0.4175→0.2402; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `This` dx -0.87 dy 0.46; `on` dx -0.86 dy 0.46; `a` dx -0.81 dy 0.46

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933619, 479, 424, 376, 360, 326, 359, 327, 338, 262, 262, 240, 199, 242, 198, 805]`; ink px ref/ours 1890/2408 (ratio 1.2741); SSIM blocks <0.9: 232/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-10.49, -0.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3601, differing 0.003021, SSIM₈ 0.9942 (raw 0.3601, 0.003021, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9908→0.9908 / 0.5755→0.5755; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -25.52 dy -0.3; `single` dx -24.61 dy -0.3; `a` dx -23.19 dy -0.3

### 01-plain-paragraph — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934693, 513, 482, 413, 349, 354, 303, 222, 247, 229, 183, 134, 119, 107, 100, 368]`; ink px ref/ours 1890/2420 (ratio 1.2804); SSIM blocks <0.9: 183/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.68, -0.12] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2368, differing 0.002561, SSIM₈ 0.9967 (raw 0.2368, 0.002561, 0.9967)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9947→0.9947 / 0.3784→0.3784; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `a` dx 0.06 dy -0.36; `single` dx 0.06 dy -0.36; `line.` dx 0.06 dy -0.36

### 01-plain-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935388, 716, 424, 359, 290, 196, 196, 126, 136, 124, 122, 86, 78, 86, 81, 408]`; ink px ref/ours 2408/2421 (ratio 1.0054); SSIM blocks <0.9: 119/30294; [overlay](images/01-plain-paragraph/pdflatex-de1020c-export-p1-overlay.png) (74736 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (69170 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.1, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1873, differing 0.002361, SSIM₈ 0.9975 (raw 0.1873, 0.002361, 0.9975)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9961→0.9961 / 0.2993→0.2993; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 1.15 dy 0.46; `single` dx 1.03 dy 0.46; `a` dx 1.0 dy 0.46

### 01-plain-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935798, 790, 432, 279, 271, 238, 187, 198, 153, 120, 112, 82, 47, 65, 9, 35]`; ink px ref/ours 2408/2408 (ratio 1.0); SSIM blocks <0.9: 124/30294; [overlay](images/01-plain-paragraph/pdflatex-main-export-p1-overlay.png) (74616 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-main-export-p1-heatmap.png) (66063 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.07, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1285, differing 0.002179, SSIM₈ 0.9985 (raw 0.1285, 0.002179, 0.9985)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9976→0.9976 / 0.2053→0.2053; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `This` dx -0.48 dy 0.46; `is` dx -0.39 dy 0.46; `line.` dx 0.37 dy 0.46

### 01-plain-paragraph — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933678, 551, 315, 257, 292, 305, 273, 227, 249, 246, 233, 262, 242, 248, 295, 1143]`; ink px ref/ours 2408/2420 (ratio 1.005); SSIM blocks <0.9: 214/30294; [overlay](images/01-plain-paragraph/pdflatex-pipeline-export-p1-overlay.png) (76318 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-pipeline-export-p1-heatmap.png) (73572 B, ÷1)
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [8.74, 0.02] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3788, differing 0.003059, SSIM₈ 0.9947 (raw 0.3946, 0.0031, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9916 / 0.6306→0.6049; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 25.96 dy 0.41; `single` dx 24.93 dy 0.41; `a` dx 23.46 dy 0.41

### 01-plain-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933593, 464, 437, 341, 371, 326, 338, 319, 328, 279, 309, 227, 246, 191, 211, 836]`; ink px ref/ours 1898/2421 (ratio 1.2756); SSIM blocks <0.9: 235/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (74003 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (70540 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-9.72, -0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.366, differing 0.003024, SSIM₈ 0.9942 (raw 0.366, 0.003024, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9907→0.9907 / 0.5849→0.5849; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -24.75 dy 0.73; `single` dx -23.83 dy 0.73; `a` dx -22.41 dy 0.73

### 01-plain-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933631, 484, 408, 370, 373, 317, 366, 314, 345, 291, 229, 241, 207, 219, 214, 807]`; ink px ref/ours 1898/2408 (ratio 1.2687); SSIM blocks <0.9: 233/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-main-export-p1-overlay.png) (73780 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (70377 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-9.69, -0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3597, differing 0.003018, SSIM₈ 0.9943 (raw 0.3597, 0.003018, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9908→0.9908 / 0.5749→0.5749; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -25.53 dy 0.73; `single` dx -24.61 dy 0.73; `a` dx -23.19 dy 0.73

### 01-plain-paragraph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934687, 523, 461, 409, 357, 365, 307, 217, 258, 220, 166, 149, 110, 115, 91, 381]`; ink px ref/ours 1898/2420 (ratio 1.275); SSIM blocks <0.9: 183/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) (75797 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-pipeline-export-p1-heatmap.png) (68605 B, ÷1)
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934582, 499, 419, 375, 397, 271, 279, 221, 230, 183, 165, 165, 180, 214, 179, 457]`; ink px ref/ours 2441/2408 (ratio 0.9865); SSIM blocks <0.9: 177/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.08, -0.01] pt); confidence strong (shift explains 43% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1528, differing 0.002283, SSIM₈ 0.9982 (raw 0.267, 0.002638, 0.9964)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9942→0.9972 / 0.4268→0.2443; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `on` dx -0.89 dy 0.46; `This` dx -0.87 dy 0.46; `a` dx -0.83 dy 0.46

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933613, 488, 422, 373, 361, 323, 361, 329, 322, 280, 246, 253, 199, 228, 206, 812]`; ink px ref/ours 1891/2408 (ratio 1.2734); SSIM blocks <0.9: 232/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-10.49, -0.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3603, differing 0.003019, SSIM₈ 0.9942 (raw 0.3603, 0.003019, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9908→0.9908 / 0.5758→0.5758; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -25.52 dy 0.73; `single` dx -24.61 dy 0.73; `a` dx -23.19 dy 0.73

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831514, 8195, 7191, 6521, 5857, 5794, 5799, 5233, 5491, 5530, 5365, 5313, 5257, 5286, 5152, 25318]`; ink px ref/ours 39630/39525 (ratio 0.9974); SSIM blocks <0.9: 4341/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 8.4396, not lower; centroid estimate [-5.5, 4.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.4139, differing 0.061613, SSIM₈ 0.8657 (raw 8.4139, 0.061613, 0.8657)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7854→0.7854 / 13.4471→13.4471; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `the` dx -430.88 dy 20.5

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834664, 8119, 7066, 6869, 6302, 6214, 6405, 6114, 6192, 5700, 5266, 5008, 5008, 4594, 4592, 20703]`; ink px ref/ours 30065/39525 (ratio 1.3147); SSIM blocks <0.9: 4481/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-1.0, -14.5] pt REJECTED: applying it gives mean|Δ| 7.8165, not lower; centroid estimate [-9.93, 0.08] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7923, differing 0.059833, SSIM₈ 0.8614 (raw 7.7923, 0.059833, 0.8614)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7786→0.7786 / 12.4538→12.4538; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -14.75; `oak` dx 425.86 dy -9.07; `jumps` dx 414.46 dy -14.79

### 02-wrapping-paragraph — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1865202, 8541, 6901, 6635, 6187, 5810, 5205, 4902, 4174, 3820, 3188, 2946, 2731, 2484, 2302, 7788]`; ink px ref/ours 30065/39220 (ratio 1.3045); SSIM blocks <0.9: 3163/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.9, 0.11] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 4.5384, differing 0.044666, SSIM₈ 0.9368 (raw 4.5384, 0.044666, 0.9368)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8992→0.8992 / 7.2527→7.2527; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `river` dx 0.07 dy -0.36; `below.` dx 0.07 dy -0.36; `the` dx 0.06 dy -0.36

### 02-wrapping-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831980, 8181, 7016, 6406, 5963, 5948, 5852, 5289, 5188, 5656, 5267, 5378, 5255, 4876, 5221, 25340]`; ink px ref/ours 39390/39464 (ratio 1.0019); SSIM blocks <0.9: 4347/30294; [overlay](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-overlay.png) (55220 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (42331 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 8.3959, not lower; centroid estimate [-7.52, 4.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.3726, differing 0.06114, SSIM₈ 0.8663 (raw 8.3726, 0.06114, 0.8663)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7864→0.7864 / 13.3813→13.3813; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831417, 8047, 6981, 6472, 5822, 6083, 5579, 5280, 5281, 5759, 5144, 5427, 5366, 5118, 5239, 25801]`; ink px ref/ours 39390/39525 (ratio 1.0034); SSIM blocks <0.9: 4321/30294; [overlay](images/02-wrapping-paragraph/pdflatex-main-export-p1-overlay.png) (54926 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-main-export-p1-heatmap.png) (42509 B, ÷4)
  - registration error (diagnostic): global shift [-3.5, 0.0] pt by ink-projection correlation (centroid estimate [-5.15, 4.11] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.4128, differing 0.061395, SSIM₈ 0.8661 (raw 8.47, 0.06151, 0.8655)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7851→0.7863 / 13.5368→13.4326; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `the` dx -430.7 dy 20.5

### 02-wrapping-paragraph — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1843949, 8117, 7107, 5998, 5577, 5552, 4958, 5101, 4781, 4976, 4671, 4733, 4299, 4229, 4583, 20185]`; ink px ref/ours 39390/39220 (ratio 0.9957); SSIM blocks <0.9: 3826/30294; [overlay](images/02-wrapping-paragraph/pdflatex-pipeline-export-p1-overlay.png) (55295 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-pipeline-export-p1-heatmap.png) (39459 B, ÷4)
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [3.87, 4.14] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.0822, differing 0.054998, SSIM₈ 0.8919 (raw 7.1603, 0.05526, 0.891)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8261→0.8283 / 11.4425→11.3054; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 14.85; `oak` dx -415.72 dy 14.85; `branch` dx -413.61 dy 14.85

### 02-wrapping-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834636, 8219, 6952, 6893, 6423, 6140, 6491, 6184, 5976, 5757, 5362, 5028, 4837, 4417, 4790, 20711]`; ink px ref/ours 30080/39464 (ratio 1.312); SSIM blocks <0.9: 4465/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (55415 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (42776 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-8.5, -14.5] pt REJECTED: applying it gives mean|Δ| 7.8128, not lower; centroid estimate [-12.18, 0.82] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7862, differing 0.059683, SSIM₈ 0.862 (raw 7.7862, 0.059683, 0.862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7795→0.7795 / 12.4441→12.4441; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.04; `jumps` dx 416.92 dy -13.77

### 02-wrapping-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834659, 8126, 7025, 6903, 6328, 6172, 6467, 6066, 6218, 5645, 5310, 5027, 5017, 4505, 4688, 20660]`; ink px ref/ours 30080/39525 (ratio 1.314); SSIM blocks <0.9: 4481/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-overlay.png) (55450 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (42717 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-1.0, -14.5] pt REJECTED: applying it gives mean|Δ| 7.8161, not lower; centroid estimate [-9.82, 0.17] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7924, differing 0.059812, SSIM₈ 0.8614 (raw 7.7924, 0.059812, 0.8614)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7786→0.7786 / 12.4539→12.4539; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -13.72; `oak` dx 425.86 dy -8.04; `jumps` dx 414.46 dy -13.77

### 02-wrapping-paragraph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1865112, 8452, 6961, 6696, 6112, 5868, 5236, 4937, 4180, 3697, 3259, 2976, 2729, 2442, 2381, 7778]`; ink px ref/ours 30080/39220 (ratio 1.3039); SSIM blocks <0.9: 3164/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) (54477 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-pipeline-export-p1-heatmap.png) (84947 B, ÷2)
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831130, 8033, 7195, 6541, 5874, 5890, 5749, 5378, 5496, 5555, 5369, 5280, 5402, 5183, 5139, 25602]`; ink px ref/ours 39556/39525 (ratio 0.9992); SSIM blocks <0.9: 4347/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-5.29, 4.11] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.4585, differing 0.061737, SSIM₈ 0.8648 (raw 8.4639, 0.061774, 0.8648)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.784→0.7842 / 13.5271→13.5175; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 20.54; `branch` dx -435.52 dy 14.86; `the` dx -430.89 dy 20.5

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834680, 8112, 7072, 6862, 6267, 6228, 6437, 6084, 6232, 5660, 5283, 5015, 4983, 4608, 4579, 20714]`; ink px ref/ours 30075/39525 (ratio 1.3142); SSIM blocks <0.9: 4482/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-1.0, -14.5] pt REJECTED: applying it gives mean|Δ| 7.8165, not lower; centroid estimate [-10.09, 0.09] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7922, differing 0.059811, SSIM₈ 0.8614 (raw 7.7922, 0.059811, 0.8614)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7786→0.7786 / 12.4536→12.4536; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -13.72; `oak` dx 425.86 dy -8.04; `jumps` dx 414.46 dy -13.77

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923544, 879, 837, 674, 664, 667, 710, 587, 619, 568, 692, 639, 718, 648, 749, 5621]`; ink px ref/ours 5807/4945 (ratio 0.8516); SSIM blocks <0.9: 672/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 52.5] pt by ink-projection correlation (centroid estimate [5.02, 20.12] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2063, differing 0.00779, SSIM₈ 0.9826 (raw 1.3597, 0.008549, 0.9789)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9663→0.9767 / 2.1732→1.6847; header-band 1.0→0.969 / 0.0→1.6747; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.4 dy 26.34; `second` dx 0.26 dy 26.34; `a` dx 0.23 dy 26.34
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926641, 841, 779, 675, 585, 697, 639, 549, 551, 520, 510, 501, 586, 575, 590, 3577]`; ink px ref/ours 5807/4968 (ratio 0.8555); SSIM blocks <0.9: 455/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -1.5] pt by ink-projection correlation (centroid estimate [17.33, 2.48] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9816, differing 0.006792, SSIM₈ 0.9874 (raw 1.0038, 0.00691, 0.9865)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9783→0.9799 / 1.6044→1.5689; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Section` dx 38.01 dy -0.57; `Introduction` dx 29.06 dy 0.59; `Second` dx 29.06 dy -0.57
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924467, 959, 778, 633, 646, 661, 738, 747, 647, 638, 596, 597, 643, 582, 684, 4800]`; ink px ref/ours 4775/4705 (ratio 0.9853); SSIM blocks <0.9: 718/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 53.5] pt by ink-projection correlation (centroid estimate [-2.67, 22.65] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0886, differing 0.007454, SSIM₈ 0.9817 (raw 1.2336, 0.008176, 0.9771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9634→0.9751 / 1.9716→1.5009; header-band 1.0→0.9699 / 0.0→1.6444; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.94 dy 27.15; `second` dx -11.49 dy 27.15; `a` dx -10.06 dy 27.15

### 03-section-heading — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924306, 982, 748, 662, 647, 696, 746, 737, 670, 652, 669, 604, 653, 601, 677, 4766]`; ink px ref/ours 4775/4945 (ratio 1.0356); SSIM blocks <0.9: 730/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 53.5] pt by ink-projection correlation (centroid estimate [0.65, 21.79] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1129, differing 0.007544, SSIM₈ 0.9812 (raw 1.2426, 0.008227, 0.9769)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9631→0.9747 / 1.986→1.5244; header-band 1.0→0.9672 / 0.0→1.7503; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -13.12 dy 27.15; `second` dx -11.67 dy 27.15; `a` dx -10.24 dy 27.15
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927930, 837, 737, 653, 634, 721, 591, 598, 531, 520, 417, 378, 451, 443, 447, 2928]`; ink px ref/ours 4775/4968 (ratio 1.0404); SSIM blocks <0.9: 431/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [12.97, 4.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8537, differing 0.006238, SSIM₈ 0.9887 (raw 0.8537, 0.006238, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9819→0.9819 / 1.3644→1.3644; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Second` dx 29.06 dy -0.7; `Section` dx 29.06 dy -0.7; `Introduction` dx 29.06 dy -0.67
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923855, 915, 733, 647, 632, 615, 615, 519, 606, 614, 675, 690, 668, 592, 707, 5733]`; ink px ref/ours 6093/4705 (ratio 0.7722); SSIM blocks <0.9: 662/30294; [overlay](images/03-section-heading/pdflatex-de1020c-export-p1-overlay.png) (39787 B, ÷2), [heatmap](images/03-section-heading/pdflatex-de1020c-export-p1-heatmap.png) (41642 B, ÷2)
  - registration error (diagnostic): global shift [0.5, 52.5] pt by ink-projection correlation (centroid estimate [1.59, 20.53] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2021, differing 0.007752, SSIM₈ 0.9827 (raw 1.3496, 0.008486, 0.9793)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9669→0.9767 / 2.157→1.6932; header-band 1.0→0.9709 / 0.0→1.5688; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.75 dy 26.08; `second` dx 0.61 dy 26.08; `a` dx 0.58 dy 26.08

### 03-section-heading — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923577, 931, 710, 654, 644, 650, 633, 512, 630, 633, 759, 710, 708, 614, 708, 5743]`; ink px ref/ours 6093/4945 (ratio 0.8116); SSIM blocks <0.9: 674/30294; [overlay](images/03-section-heading/pdflatex-main-export-p1-overlay.png) (39619 B, ÷2), [heatmap](images/03-section-heading/pdflatex-main-export-p1-heatmap.png) (41564 B, ÷2)
  - registration error (diagnostic): global shift [0.5, 52.5] pt by ink-projection correlation (centroid estimate [4.92, 19.67] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.22, differing 0.007837, SSIM₈ 0.9824 (raw 1.3719, 0.008586, 0.9789)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9663→0.9765 / 2.1927→1.7064; header-band 1.0→0.9684 / 0.0→1.6747; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.57 dy 26.08; `second` dx 0.43 dy 26.08; `a` dx 0.4 dy 26.08
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926579, 859, 637, 622, 571, 686, 590, 481, 616, 558, 554, 592, 607, 539, 559, 3766]`; ink px ref/ours 6093/4968 (ratio 0.8154); SSIM blocks <0.9: 461/30294; [overlay](images/03-section-heading/pdflatex-pipeline-export-p1-overlay.png) (39689 B, ÷2), [heatmap](images/03-section-heading/pdflatex-pipeline-export-p1-heatmap.png) (89624 B, ÷1)
  - registration error (diagnostic): global shift [0.5, -2.0] pt by ink-projection correlation (centroid estimate [17.23, 2.02] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0233, differing 0.006976, SSIM₈ 0.9859 (raw 1.0318, 0.006996, 0.9862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9779→0.9776 / 1.6491→1.6353; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Section` dx 38.01 dy -0.64; `Introduction` dx 29.06 dy 0.71; `Second` dx 29.06 dy -0.64
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924475, 952, 791, 621, 670, 652, 705, 745, 643, 649, 621, 599, 586, 629, 701, 4777]`; ink px ref/ours 4791/4705 (ratio 0.982); SSIM blocks <0.9: 718/30294; [overlay](images/03-section-heading/pdflatex-lm-de1020c-export-p1-overlay.png) (39975 B, ÷2), [heatmap](images/03-section-heading/pdflatex-lm-de1020c-export-p1-heatmap.png) (41999 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 54.0] pt by ink-projection correlation (centroid estimate [-2.62, 22.29] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0954, differing 0.00748, SSIM₈ 0.9811 (raw 1.2335, 0.008161, 0.9771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9634→0.975 / 1.9715→1.4903; header-band 1.0→0.9643 / 0.0→1.793; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.95 dy 28.22; `second` dx -11.5 dy 28.22; `a` dx -10.07 dy 28.22

### 03-section-heading — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924312, 961, 779, 647, 672, 690, 707, 738, 668, 659, 696, 603, 593, 642, 704, 4745]`; ink px ref/ours 4791/4945 (ratio 1.0321); SSIM blocks <0.9: 730/30294; [overlay](images/03-section-heading/pdflatex-lm-main-export-p1-overlay.png) (39819 B, ÷2), [heatmap](images/03-section-heading/pdflatex-lm-main-export-p1-heatmap.png) (41818 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 54.0] pt by ink-projection correlation (centroid estimate [0.7, 21.43] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1208, differing 0.007572, SSIM₈ 0.9807 (raw 1.2428, 0.008211, 0.9769)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9631→0.9747 / 1.9863→1.5155; header-band 1.0→0.962 / 0.0→1.8988; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -13.13 dy 28.22; `second` dx -11.68 dy 28.22; `a` dx -10.25 dy 28.22
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927942, 856, 713, 645, 680, 677, 621, 565, 528, 522, 425, 385, 411, 447, 449, 2950]`; ink px ref/ours 4791/4968 (ratio 1.0369); SSIM blocks <0.9: 430/30294; [overlay](images/03-section-heading/pdflatex-lm-pipeline-export-p1-overlay.png) (39241 B, ÷2), [heatmap](images/03-section-heading/pdflatex-lm-pipeline-export-p1-heatmap.png) (86636 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [13.02, 3.79] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8533, differing 0.006233, SSIM₈ 0.9887 (raw 0.8533, 0.006233, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9819→0.9819 / 1.3638→1.3638; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Introduction` dx 29.06 dy 0.96; `Second` dx 29.06 dy 0.96; `Section` dx 29.06 dy 0.96
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923747, 880, 847, 676, 683, 663, 736, 610, 596, 553, 616, 586, 654, 628, 733, 5608]`; ink px ref/ours 5744/4705 (ratio 0.8191); SSIM blocks <0.9: 660/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 52.5] pt by ink-projection correlation (centroid estimate [1.75, 20.6] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1868, differing 0.007742, SSIM₈ 0.9829 (raw 1.337, 0.008469, 0.9793)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9669→0.9769 / 2.137→1.669; header-band 1.0→0.9712 / 0.0→1.5688; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.57 dy 26.34; `second` dx 0.43 dy 26.34; `a` dx 0.4 dy 26.34

### 03-section-heading — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923470, 890, 833, 686, 684, 694, 739, 619, 613, 570, 702, 619, 690, 649, 738, 5620]`; ink px ref/ours 5744/4945 (ratio 0.8609); SSIM blocks <0.9: 672/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 52.5] pt by ink-projection correlation (centroid estimate [5.07, 19.74] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2075, differing 0.007813, SSIM₈ 0.9826 (raw 1.3601, 0.00857, 0.9789)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9663→0.9767 / 2.1738→1.6866; header-band 1.0→0.969 / 0.0→1.6747; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.39 dy 26.34; `second` dx 0.25 dy 26.34; `a` dx 0.22 dy 26.34
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926547, 852, 783, 699, 603, 677, 672, 579, 554, 520, 514, 474, 562, 552, 601, 3627]`; ink px ref/ours 5744/4968 (ratio 0.8649); SSIM blocks <0.9: 455/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -1.5] pt by ink-projection correlation (centroid estimate [17.38, 2.09] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9853, differing 0.006835, SSIM₈ 0.9874 (raw 1.009, 0.006952, 0.9864)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9783→0.9799 / 1.6127→1.5749; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Section` dx 38.01 dy -0.57; `Introduction` dx 29.06 dy 0.59; `Second` dx 29.06 dy -0.57
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924466, 954, 785, 634, 640, 664, 732, 752, 647, 641, 594, 601, 641, 581, 686, 4798]`; ink px ref/ours 4777/4705 (ratio 0.9849); SSIM blocks <0.9: 718/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 53.5] pt by ink-projection correlation (centroid estimate [-2.67, 22.65] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0886, differing 0.007455, SSIM₈ 0.9817 (raw 1.2337, 0.008177, 0.9771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9634→0.9751 / 1.9717→1.501; header-band 1.0→0.9699 / 0.0→1.6444; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.94 dy 28.18; `second` dx -11.49 dy 28.18; `a` dx -10.06 dy 28.18

### 03-section-heading — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924306, 986, 747, 663, 641, 697, 745, 739, 669, 654, 666, 611, 648, 608, 672, 4764]`; ink px ref/ours 4777/4945 (ratio 1.0352); SSIM blocks <0.9: 730/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 53.5] pt by ink-projection correlation (centroid estimate [0.65, 21.79] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1129, differing 0.007544, SSIM₈ 0.9812 (raw 1.2425, 0.008228, 0.9769)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9631→0.9747 / 1.9858→1.5244; header-band 1.0→0.9672 / 0.0→1.7503; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -13.12 dy 28.18; `second` dx -11.67 dy 28.18; `a` dx -10.24 dy 28.18
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927929, 833, 743, 652, 632, 721, 590, 602, 530, 517, 423, 378, 448, 441, 449, 2928]`; ink px ref/ours 4777/4968 (ratio 1.04); SSIM blocks <0.9: 431/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [12.97, 4.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8537, differing 0.006237, SSIM₈ 0.9887 (raw 0.8537, 0.006237, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9819→0.9819 / 1.3644→1.3644; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Introduction` dx 29.06 dy 0.96; `Second` dx 29.06 dy 0.93; `Section` dx 29.06 dy 0.93
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933581, 495, 476, 346, 302, 277, 341, 277, 321, 274, 237, 267, 204, 216, 208, 994]`; ink px ref/ours 2712/2460 (ratio 0.9071); SSIM blocks <0.9: 227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.75, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3769, differing 0.003131, SSIM₈ 0.9945 (raw 0.3769, 0.003131, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9913→0.9913 / 0.6024→0.6024; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933099, 458, 398, 356, 373, 363, 333, 348, 353, 303, 239, 305, 251, 265, 241, 1131]`; ink px ref/ours 2276/2460 (ratio 1.0808); SSIM blocks <0.9: 259/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-19.75, 0.03] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4147, differing 0.003329, SSIM₈ 0.9934 (raw 0.4253, 0.003331, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6797→0.6627; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -28.11 dy -0.63; `one` dx -26.98 dy -0.63; `on` dx -25.72 dy -0.63
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933598, 513, 444, 319, 342, 300, 270, 233, 301, 263, 246, 223, 207, 251, 234, 1072]`; ink px ref/ours 2746/2465 (ratio 0.8977); SSIM blocks <0.9: 225/30294; [overlay](images/04-bold-emph/pdflatex-de1020c-export-p1-overlay.png) (76685 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-de1020c-export-p1-heatmap.png) (70812 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [3.52, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3764, differing 0.003158, SSIM₈ 0.9945 (raw 0.3836, 0.003143, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9912 / 0.6131→0.6017; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.27 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933606, 529, 426, 332, 337, 306, 282, 233, 293, 251, 239, 262, 205, 244, 250, 1021]`; ink px ref/ours 2746/2460 (ratio 0.8958); SSIM blocks <0.9: 225/30294; [overlay](images/04-bold-emph/pdflatex-main-export-p1-overlay.png) (76417 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-main-export-p1-heatmap.png) (73174 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.48, 0.04] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3777, differing 0.003162, SSIM₈ 0.9945 (raw 0.3804, 0.003141, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9912 / 0.608→0.6036; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.27 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933369, 499, 380, 334, 314, 321, 249, 275, 277, 265, 256, 281, 274, 236, 287, 1199]`; ink px ref/ours 2746/2449 (ratio 0.8918); SSIM blocks <0.9: 242/30294; [overlay](images/04-bold-emph/pdflatex-pipeline-export-p1-overlay.png) (77108 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-pipeline-export-p1-heatmap.png) (74300 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [13.8, 0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4169, differing 0.003295, SSIM₈ 0.9939 (raw 0.4169, 0.003295, 0.9939)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9903→0.9903 / 0.6663→0.6663; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 31.97 dy 0.41; `one` dx 30.75 dy 0.41; `on` dx 29.44 dy 0.41
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

### 04-bold-emph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933105, 492, 380, 370, 376, 348, 330, 343, 330, 319, 248, 266, 243, 259, 233, 1174]`; ink px ref/ours 2268/2465 (ratio 1.0869); SSIM blocks <0.9: 259/30294; [overlay](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-overlay.png) (77476 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-heatmap.png) (72126 B, ÷1)
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-17.16, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4162, differing 0.003327, SSIM₈ 0.9934 (raw 0.4246, 0.003322, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6786→0.6653; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -28.1 dy 0.73; `one` dx -26.97 dy 0.73; `on` dx -25.72 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933067, 485, 392, 372, 376, 351, 346, 341, 345, 316, 246, 309, 232, 254, 238, 1146]`; ink px ref/ours 2268/2460 (ratio 1.0847); SSIM blocks <0.9: 259/30294; [overlay](images/04-bold-emph/pdflatex-lm-main-export-p1-overlay.png) (77164 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-main-export-p1-heatmap.png) (71737 B, ÷1)
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-19.2, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4164, differing 0.003329, SSIM₈ 0.9934 (raw 0.4256, 0.003332, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6803→0.6655; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -28.22 dy 0.73; `one` dx -27.09 dy 0.73; `on` dx -25.84 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933726, 491, 398, 359, 367, 347, 320, 302, 255, 274, 281, 229, 215, 212, 218, 822]`; ink px ref/ours 2268/2449 (ratio 1.0798); SSIM blocks <0.9: 225/30294; [overlay](images/04-bold-emph/pdflatex-lm-pipeline-export-p1-overlay.png) (74796 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-pipeline-export-p1-heatmap.png) (70790 B, ÷1)
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933534, 525, 447, 350, 302, 290, 343, 267, 303, 288, 244, 261, 213, 217, 228, 1004]`; ink px ref/ours 2702/2460 (ratio 0.9104); SSIM blocks <0.9: 227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.27, 0.0] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3801, differing 0.003144, SSIM₈ 0.9944 (raw 0.381, 0.003132, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9912→0.991 / 0.609→0.6075; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933109, 469, 421, 338, 378, 357, 345, 342, 342, 280, 242, 304, 251, 250, 228, 1160]`; ink px ref/ours 2260/2460 (ratio 1.0885); SSIM blocks <0.9: 256/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-18.2, 0.02] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4127, differing 0.003305, SSIM₈ 0.9934 (raw 0.4237, 0.003311, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6772→0.6596; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -27.87 dy 0.73; `one` dx -26.74 dy 0.73; `on` dx -25.48 dy 0.73
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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934812, 608, 496, 375, 300, 247, 183, 147, 142, 184, 149, 137, 156, 150, 142, 588]`; ink px ref/ours 2129/2125 (ratio 0.9981); SSIM blocks <0.9: 166/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [8.87, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2489, differing 0.002552, SSIM₈ 0.9963 (raw 0.2489, 0.002552, 0.9963)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9941→0.9941 / 0.3978→0.3978; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 35.45 dy 0.46; `café` dx -0.37 dy 0.46; `Résumé` dx -0.27 dy 0.46
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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933797, 474, 369, 300, 325, 326, 312, 319, 258, 287, 223, 211, 270, 219, 240, 886]`; ink px ref/ours 1547/2125 (ratio 1.3736); SSIM blocks <0.9: 238/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.5, 0.0] pt by ink-projection correlation (centroid estimate [-0.15, -0.15] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3432, differing 0.00286, SSIM₈ 0.9941 (raw 0.3628, 0.002915, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9901→0.9906 / 0.5798→0.5486; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 21.23 dy -0.3; `also` dx -7.83 dy -0.3; `dash;` dx -5.53 dy -0.3
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934712, 472, 304, 331, 283, 384, 333, 300, 200, 240, 174, 123, 169, 155, 138, 498]`; ink px ref/ours 1547/2085 (ratio 1.3478); SSIM blocks <0.9: 186/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [0.03, -0.12] pt); confidence moderate (shift explains 21% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2087, differing 0.002248, SSIM₈ 0.9969 (raw 0.2635, 0.002446, 0.996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9936→0.9951 / 0.4211→0.3336; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 0.05 dy -0.36; `Résumé` dx 0.04 dy -0.36; `—` dx 0.04 dy -0.36

### 05-unicode — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934287, 570, 367, 345, 277, 258, 239, 232, 233, 241, 245, 185, 188, 173, 231, 745]`; ink px ref/ours 2119/2123 (ratio 1.0019); SSIM blocks <0.9: 209/30294; [overlay](images/05-unicode/pdflatex-de1020c-export-p1-overlay.png) (76006 B, ÷1), [heatmap](images/05-unicode/pdflatex-de1020c-export-p1-heatmap.png) (72684 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-5.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.331, not lower; centroid estimate [3.28, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3127, differing 0.002744, SSIM₈ 0.9955 (raw 0.3127, 0.002744, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9928→0.9928 / 0.4998→0.4998; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 30.73 dy 0.46; `also` dx -5.31 dy 0.46; `dash;` dx -3.91 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934942, 727, 440, 283, 275, 291, 190, 189, 117, 160, 132, 96, 131, 144, 138, 561]`; ink px ref/ours 2119/2125 (ratio 1.0028); SSIM blocks <0.9: 150/30294; [overlay](images/05-unicode/pdflatex-main-export-p1-overlay.png) (74960 B, ÷1), [heatmap](images/05-unicode/pdflatex-main-export-p1-heatmap.png) (70829 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [6.99, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2342, differing 0.002512, SSIM₈ 0.9967 (raw 0.2342, 0.002512, 0.9967)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9947→0.9947 / 0.3743→0.3743; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 35.77 dy 0.46; `also` dx 0.21 dy 0.46; `café` dx -0.19 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934082, 524, 279, 255, 267, 278, 233, 249, 262, 254, 228, 236, 215, 222, 277, 955]`; ink px ref/ours 2119/2085 (ratio 0.984); SSIM blocks <0.9: 195/30294; [overlay](images/05-unicode/pdflatex-pipeline-export-p1-overlay.png) (74914 B, ÷1), [heatmap](images/05-unicode/pdflatex-pipeline-export-p1-heatmap.png) (72894 B, ÷1)
  - registration error (diagnostic): global shift [11.5, 0.0] pt by ink-projection correlation (centroid estimate [7.18, 0.02] pt); confidence moderate (shift explains 14% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3059, differing 0.002681, SSIM₈ 0.9957 (raw 0.3571, 0.002879, 0.995)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9921→0.9935 / 0.5708→0.4685; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 14.58 dy 0.41; `—` dx 13.92 dy 0.41; `Résumé` dx 11.63 dy 0.41

### 05-unicode — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933782, 470, 394, 304, 327, 351, 291, 322, 255, 269, 237, 246, 249, 203, 219, 897]`; ink px ref/ours 1537/2123 (ratio 1.3813); SSIM blocks <0.9: 237/30294; [overlay](images/05-unicode/pdflatex-lm-de1020c-export-p1-overlay.png) (75651 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-de1020c-export-p1-heatmap.png) (73226 B, ÷1)
  - registration error (diagnostic): global shift [2.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.9, -0.14] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3496, differing 0.00287, SSIM₈ 0.994 (raw 0.3617, 0.002921, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.9905 / 0.5782→0.5533; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 16.16 dy 0.73; `also` dx -13.37 dy 0.73; `dash;` dx -9.57 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933811, 462, 385, 277, 331, 324, 306, 331, 263, 270, 232, 211, 269, 225, 224, 895]`; ink px ref/ours 1537/2125 (ratio 1.3826); SSIM blocks <0.9: 238/30294; [overlay](images/05-unicode/pdflatex-lm-main-export-p1-overlay.png) (75092 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-main-export-p1-heatmap.png) (73013 B, ÷1)
  - registration error (diagnostic): global shift [-3.5, 0.0] pt by ink-projection correlation (centroid estimate [0.81, -0.15] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.343, differing 0.002853, SSIM₈ 0.9941 (raw 0.3625, 0.00291, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9901→0.9906 / 0.5794→0.5482; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 21.2 dy 0.73; `also` dx -7.85 dy 0.73; `dash;` dx -5.55 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934693, 462, 325, 315, 299, 379, 324, 309, 193, 237, 188, 127, 158, 184, 117, 506]`; ink px ref/ours 1537/2085 (ratio 1.3565); SSIM blocks <0.9: 188/30294; [overlay](images/05-unicode/pdflatex-lm-pipeline-export-p1-overlay.png) (75219 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-pipeline-export-p1-heatmap.png) (70945 B, ÷1)
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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934854, 565, 485, 386, 275, 253, 206, 129, 157, 167, 165, 148, 139, 142, 149, 596]`; ink px ref/ours 2139/2125 (ratio 0.9935); SSIM blocks <0.9: 164/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [7.33, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2493, differing 0.002552, SSIM₈ 0.9963 (raw 0.2493, 0.002552, 0.9963)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9941→0.9941 / 0.3984→0.3984; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 35.41 dy 0.46; `café` dx -0.37 dy 0.46; `Résumé` dx -0.29 dy 0.46
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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933805, 465, 372, 296, 327, 328, 308, 322, 256, 288, 223, 209, 274, 217, 239, 887]`; ink px ref/ours 1542/2125 (ratio 1.3781); SSIM blocks <0.9: 238/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.5, 0.0] pt by ink-projection correlation (centroid estimate [-0.15, -0.14] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3432, differing 0.002859, SSIM₈ 0.9941 (raw 0.3628, 0.002916, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9901→0.9906 / 0.5798→0.5485; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 21.23 dy 0.73; `also` dx -7.83 dy 0.73; `dash;` dx -5.53 dy 0.73
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
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933815, 451, 328, 345, 235, 254, 250, 292, 244, 238, 235, 299, 284, 234, 255, 1057]`; ink px ref/ours 1729/1837 (ratio 1.0625); SSIM blocks <0.9: 218/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-6.5, 3.5] pt by ink-projection correlation (centroid estimate [-4.37, 3.53] pt); confidence strong (shift explains 48% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1998, differing 0.002181, SSIM₈ 0.9967 (raw 0.3838, 0.002925, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9947 / 0.6135→0.3194; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5582→0.8266 / 21.5408→9.4933 [131.7,62.4–343.8,93.6 pt]
- largest word displacements (pt): `b` dx -0.95 dy 7.52; `inside` dx -6.73 dy 1.32; `a` dx -6.62 dy 1.32
- word-sequence differences: replace ref ['α', '+', 'β,'] ours ['α+β,']

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
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933794, 457, 322, 320, 272, 308, 264, 352, 278, 271, 255, 282, 215, 237, 221, 968]`; ink px ref/ours 1358/1837 (ratio 1.3527); SSIM blocks <0.9: 229/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-7.5, 3.45] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.312, differing 0.002602, SSIM₈ 0.9945 (raw 0.3735, 0.002891, 0.9928)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9886→0.9915 / 0.5969→0.478; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5818→0.6793 / 18.5969→15.5792 [137.6,62.4–364.2,93.6 pt]
- largest word displacements (pt): `text.` dx -23.92 dy 1.32; `of` dx -23.58 dy 1.32; `sentence` dx -21.35 dy 1.32
- word-sequence differences: replace ref ['α', '+', 'β,'] ours ['α+β,']

### 06-math-inline — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [139.855, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934728, 387, 269, 272, 275, 242, 241, 259, 235, 243, 216, 218, 179, 174, 194, 684]`; ink px ref/ours 1358/1739 (ratio 1.2806); SSIM blocks <0.9: 186/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [4.57, -0.45] pt); confidence moderate (shift explains 18% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2401, differing 0.002192, SSIM₈ 0.996 (raw 0.2944, 0.002391, 0.9949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.992→0.9937 / 0.4646→0.3778; header-band 0.9986→0.9989 / 0.0409→0.0409; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.666→0.7476 / 15.7606→12.3873 [137.6,62.4–364.2,93.6 pt]
- largest word displacements (pt): `inside` dx -1.12 dy -2.68; `a` dx -1.11 dy -2.68; `sentence` dx -1.11 dy -2.68
- word-sequence differences: replace ref ['a', 'b'] ours ['?', '?']; replace ref ['α'] ours ['?']; replace ref ['β,'] ours ['?', ',']

### 06-math-inline — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933877, 403, 304, 272, 266, 302, 299, 308, 222, 194, 272, 315, 237, 246, 231, 1068]`; ink px ref/ours 1699/1833 (ratio 1.0789); SSIM blocks <0.9: 211/30294; [overlay](images/06-math-inline/pdflatex-de1020c-export-p1-overlay.png) (73578 B, ÷1), [heatmap](images/06-math-inline/pdflatex-de1020c-export-p1-heatmap.png) (73101 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-4.61, 3.51] pt); confidence strong (shift explains 32% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2595, differing 0.002359, SSIM₈ 0.996 (raw 0.3821, 0.002886, 0.9936)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9898→0.9937 / 0.6107→0.4148; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5648→0.6749 / 21.4145→17.7703 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `a` dx -7.78 dy 1.32; `sentence` dx -7.76 dy 1.32; `of` dx -7.59 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']; replace ref ['√xinside'] ours ['√x', 'inside']

### 06-math-inline — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933871, 414, 298, 313, 307, 308, 238, 295, 227, 240, 241, 299, 238, 236, 260, 1031]`; ink px ref/ours 1699/1837 (ratio 1.0812); SSIM blocks <0.9: 214/30294; [overlay](images/06-math-inline/pdflatex-main-export-p1-overlay.png) (73776 B, ÷1), [heatmap](images/06-math-inline/pdflatex-main-export-p1-heatmap.png) (72872 B, ÷1)
  - registration error (diagnostic): global shift [-6.0, 3.5] pt by ink-projection correlation (centroid estimate [-3.76, 3.53] pt); confidence strong (shift explains 44% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2119, differing 0.002214, SSIM₈ 0.9968 (raw 0.3782, 0.002877, 0.9935)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9896→0.9949 / 0.6045→0.3386; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5621→0.8104 / 21.1356→10.6633 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `b` dx -0.68 dy 7.52; `a` dx -6.36 dy 1.32; `sentence` dx -6.33 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α+β,']; replace ref ['√xinside'] ours ['√x', 'inside']

### 06-math-inline — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [139.855, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934994, 362, 239, 227, 246, 248, 242, 237, 191, 176, 203, 218, 167, 179, 191, 696]`; ink px ref/ours 1699/1739 (ratio 1.0235); SSIM blocks <0.9: 181/30294; [overlay](images/06-math-inline/pdflatex-pipeline-export-p1-overlay.png) (70827 B, ÷1), [heatmap](images/06-math-inline/pdflatex-pipeline-export-p1-heatmap.png) (68343 B, ÷1)
  - registration error (diagnostic): global shift [16.5, 0.0] pt by ink-projection correlation (centroid estimate [8.31, -0.38] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2776, differing 0.002395, SSIM₈ 0.9954 (raw 0.2818, 0.002383, 0.9951)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9924→0.9935 / 0.4444→0.4036; header-band 0.9986→0.9986 / 0.0409→0.0409; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6579→0.7347 / 16.2885→14.0222 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `text.` dx 16.71 dy -2.68; `of` dx 16.32 dy -2.68; `sentence` dx 13.91 dy -2.68
- word-sequence differences: replace ref ['a', 'b'] ours ['?', '?']; replace ref ['α+', 'β,'] ours ['?', '+', '?', ',']; replace ref ['√xinside'] ours ['√', '?', 'inside']

### 06-math-inline — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933850, 444, 339, 259, 271, 318, 327, 365, 277, 225, 258, 298, 218, 220, 185, 962]`; ink px ref/ours 1340/1833 (ratio 1.3679); SSIM blocks <0.9: 225/30294; [overlay](images/06-math-inline/pdflatex-lm-de1020c-export-p1-overlay.png) (74162 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-de1020c-export-p1-heatmap.png) (73606 B, ÷1)
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-7.35, 3.41] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3234, differing 0.002634, SSIM₈ 0.9943 (raw 0.368, 0.002877, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9912 / 0.5881→0.4963; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5867→0.6637 / 18.2641→16.2875 [137.6,62.4–364.3,93.6 pt]
- largest word displacements (pt): `text.` dx -25.38 dy 1.32; `of` dx -25.05 dy 1.32; `sentence` dx -22.81 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']

### 06-math-inline — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933785, 471, 301, 328, 282, 317, 264, 358, 282, 261, 252, 288, 208, 228, 206, 985]`; ink px ref/ours 1340/1837 (ratio 1.3709); SSIM blocks <0.9: 228/30294; [overlay](images/06-math-inline/pdflatex-lm-main-export-p1-overlay.png) (74020 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-main-export-p1-heatmap.png) (73219 B, ÷1)
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-6.5, 3.43] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3115, differing 0.002605, SSIM₈ 0.9945 (raw 0.3729, 0.002899, 0.9928)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9886→0.9915 / 0.596→0.4773; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5824→0.6797 / 18.5569→15.5459 [137.6,62.4–364.3,93.6 pt]
- largest word displacements (pt): `text.` dx -23.96 dy 1.32; `of` dx -23.62 dy 1.32; `sentence` dx -21.38 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α+β,']

### 06-math-inline — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [139.855, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934717, 389, 246, 288, 291, 239, 249, 267, 227, 249, 231, 206, 165, 163, 184, 705]`; ink px ref/ours 1340/1739 (ratio 1.2978); SSIM blocks <0.9: 186/30294; [overlay](images/06-math-inline/pdflatex-lm-pipeline-export-p1-overlay.png) (72993 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-pipeline-export-p1-heatmap.png) (70571 B, ÷1)
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
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933820, 455, 314, 349, 235, 259, 253, 279, 253, 229, 244, 294, 274, 245, 243, 1070]`; ink px ref/ours 1739/1837 (ratio 1.0564); SSIM blocks <0.9: 218/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-6.5, 3.5] pt by ink-projection correlation (centroid estimate [-3.63, 3.54] pt); confidence strong (shift explains 48% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1996, differing 0.002172, SSIM₈ 0.9967 (raw 0.3839, 0.00292, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9948 / 0.6136→0.3191; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5581→0.8263 / 21.5494→9.5045 [131.7,62.1–343.9,93.6 pt]
- largest word displacements (pt): `b` dx -0.95 dy 7.52; `inside` dx -6.73 dy 1.32; `a` dx -6.62 dy 1.32
- word-sequence differences: replace ref ['α', '+', 'β,'] ours ['α+β,']

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
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933798, 451, 326, 322, 265, 314, 263, 349, 281, 269, 256, 281, 217, 236, 219, 969]`; ink px ref/ours 1359/1837 (ratio 1.3517); SSIM blocks <0.9: 229/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-8.11, 3.46] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3119, differing 0.002604, SSIM₈ 0.9945 (raw 0.3734, 0.002892, 0.9928)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9886→0.9915 / 0.5969→0.4779; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5818→0.6794 / 18.596→15.5742 [137.6,62.1–364.2,93.6 pt]
- largest word displacements (pt): `text.` dx -23.92 dy 1.32; `of` dx -23.58 dy 1.32; `sentence` dx -21.35 dy 1.32
- word-sequence differences: replace ref ['α', '+', 'β,'] ours ['α+β,']

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
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [308.06, 686.88, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934495, 592, 459, 435, 298, 298, 223, 245, 320, 157, 148, 182, 187, 128, 165, 484]`; ink px ref/ours 1954/1865 (ratio 0.9545); SSIM blocks <0.9: 204/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [13.54, 1.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.264, differing 0.002646, SSIM₈ 0.9946 (raw 0.264, 0.002646, 0.9946)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9914→0.9914 / 0.422→0.422; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5021→0.5021 / 17.8342→17.8342 [261.4,84.5–349.7,122.4 pt]
- largest word displacements (pt): `2` dx -2.26 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'n(n+1)', 'i=1i=']; insert ref [] ours ['(1)']

### 07-math-display — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [303.542, 690.0, 44.64, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934247, 503, 388, 386, 254, 254, 313, 271, 249, 204, 202, 186, 191, 179, 153, 836]`; ink px ref/ours 1954/1965 (ratio 1.0056); SSIM blocks <0.9: 216/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3222, not lower; centroid estimate [7.35, -0.4] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.317, differing 0.002745, SSIM₈ 0.9943 (raw 0.317, 0.002745, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9908→0.9908 / 0.5066→0.5066; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6063→0.6063 / 16.6593→16.6593 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 1.75 Δy 0.0 len 44.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx 8.82 dy 0.34; `display.` dx 3.63 dy 6.37; `the` dx 1.06 dy 6.37
- word-sequence differences: insert ref [] ours ['?', '∑', '?=', '1']; replace ref ['n', 'i=1', 'i=', 'n(n'] ours ['?', '=', '?', '(?']

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
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [308.06, 686.88, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933817, 404, 354, 424, 362, 363, 307, 286, 290, 217, 258, 212, 307, 173, 241, 801]`; ink px ref/ours 1608/1865 (ratio 1.1598); SSIM blocks <0.9: 248/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.82, 1.4] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3541, differing 0.002914, SSIM₈ 0.9928 (raw 0.3541, 0.002914, 0.9928)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9885→0.9885 / 0.566→0.566; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5083→0.5083 / 17.6196→17.6196 [261.4,84.4–349.7,122.2 pt]
- largest word displacements (pt): `display.` dx -4.73 dy 1.09; `display.` dx -3.41 dy -0.3; `2` dx -2.26 dy 2.25
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'n(n+1)', 'i=1i=']; insert ref [] ours ['(1)']

### 07-math-display — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [303.542, 690.0, 44.64, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934305, 385, 429, 317, 373, 338, 321, 311, 289, 229, 231, 166, 177, 143, 157, 645]`; ink px ref/ours 1608/1965 (ratio 1.222); SSIM blocks <0.9: 212/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-8.01, -0.01] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2917, differing 0.002611, SSIM₈ 0.9945 (raw 0.3008, 0.002646, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9912→0.9913 / 0.4808→0.4662; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6253→0.6163 / 15.617→15.7462 [261.4,84.4–349.7,122.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 1.75 Δy 0.0 len 44.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx 0.01 dy 5.6; `Before` dx 0 dy 5.6; `the` dx 0.0 dy 5.6
- word-sequence differences: insert ref [] ours ['?', '∑', '?=', '1']; replace ref ['n', 'i=1', 'i=', 'n(n'] ours ['?', '=', '?', '(?']

### 07-math-display — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934567, 583, 430, 473, 321, 297, 196, 195, 302, 176, 128, 141, 187, 135, 135, 550]`; ink px ref/ours 1978/1796 (ratio 0.908); SSIM blocks <0.9: 193/30294; [overlay](images/07-math-display/pdflatex-de1020c-export-p1-overlay.png) (74943 B, ÷1), [heatmap](images/07-math-display/pdflatex-de1020c-export-p1-heatmap.png) (70926 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.55, 0.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2609, differing 0.00259, SSIM₈ 0.9949 (raw 0.2609, 0.00259, 0.9949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9919→0.9919 / 0.417→0.417; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4801→0.4801 / 19.0485→19.0485 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.1; `display.` dx 0.17 dy 2.13; `the` dx 0.11 dy 2.13
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [308.06, 686.88, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934563, 580, 425, 363, 331, 289, 220, 222, 334, 178, 144, 169, 193, 130, 176, 499]`; ink px ref/ours 1978/1865 (ratio 0.9429); SSIM blocks <0.9: 206/30294; [overlay](images/07-math-display/pdflatex-main-export-p1-overlay.png) (75270 B, ÷1), [heatmap](images/07-math-display/pdflatex-main-export-p1-heatmap.png) (71368 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [14.57, 0.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2654, differing 0.002609, SSIM₈ 0.9946 (raw 0.2654, 0.002609, 0.9946)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9914→0.9914 / 0.4242→0.4242; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5021→0.5021 / 17.8407→17.8407 [261.4,84.5–349.7,122.4 pt]
- largest word displacements (pt): `2` dx -2.26 dy 2.1; `display.` dx 0.17 dy 2.13; `the` dx 0.11 dy 2.13
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'n(n+1)', 'i=1i=']; insert ref [] ours ['(1)']

### 07-math-display — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [303.542, 690.0, 44.64, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934444, 504, 315, 253, 325, 229, 276, 215, 271, 213, 212, 177, 222, 201, 176, 783]`; ink px ref/ours 1978/1965 (ratio 0.9934); SSIM blocks <0.9: 208/30294; [overlay](images/07-math-display/pdflatex-pipeline-export-p1-overlay.png) (74951 B, ÷1), [heatmap](images/07-math-display/pdflatex-pipeline-export-p1-heatmap.png) (73751 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3293, not lower; centroid estimate [8.38, -0.66] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3118, differing 0.002694, SSIM₈ 0.9944 (raw 0.3118, 0.002694, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.991→0.991 / 0.4984→0.4984; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6049→0.6049 / 16.7328→16.7328 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 1.75 Δy 0.0 len 44.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx 8.81 dy 0.53; `display.` dx 3.62 dy 6.37; `the` dx 1.06 dy 6.37
- word-sequence differences: insert ref [] ours ['?', '∑', '?=', '1']; replace ref ['n', 'i=1', 'i=', 'n(n'] ours ['?', '=', '?', '(?']

### 07-math-display — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933745, 450, 367, 487, 356, 402, 264, 258, 285, 261, 208, 305, 184, 175, 192, 877]`; ink px ref/ours 1630/1796 (ratio 1.1018); SSIM blocks <0.9: 235/30294; [overlay](images/07-math-display/pdflatex-lm-de1020c-export-p1-overlay.png) (75778 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-de1020c-export-p1-heatmap.png) (74379 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-21.21, 0.91] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3528, differing 0.00293, SSIM₈ 0.9931 (raw 0.3528, 0.00293, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9889 / 0.5639→0.5639; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4864→0.4864 / 18.773→18.773 [261.4,84.2–349.7,122.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.75 dy 2.53; `2` dx -3.54 dy 2.36; `display.` dx -3.42 dy 0.73
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [308.06, 686.88, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933739, 449, 360, 378, 367, 395, 291, 280, 316, 265, 216, 336, 196, 171, 230, 827]`; ink px ref/ours 1630/1865 (ratio 1.1442); SSIM blocks <0.9: 248/30294; [overlay](images/07-math-display/pdflatex-lm-main-export-p1-overlay.png) (76113 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-main-export-p1-heatmap.png) (74783 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.92, 0.9] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3574, differing 0.00295, SSIM₈ 0.9927 (raw 0.3574, 0.00295, 0.9927)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9884→0.9884 / 0.5713→0.5713; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5052→0.5052 / 17.5969→17.5969 [261.4,84.2–349.7,122.1 pt]
- largest word displacements (pt): `display.` dx -4.75 dy 2.53; `display.` dx -3.42 dy 0.73; `the` dx -2.24 dy 2.53
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'n(n+1)', 'i=1i=']; insert ref [] ours ['(1)']

### 07-math-display — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [303.542, 690.0, 44.64, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934251, 496, 328, 387, 356, 349, 303, 317, 294, 245, 199, 186, 154, 158, 142, 651]`; ink px ref/ours 1630/1965 (ratio 1.2055); SSIM blocks <0.9: 212/30294; [overlay](images/07-math-display/pdflatex-lm-pipeline-export-p1-overlay.png) (75590 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-pipeline-export-p1-heatmap.png) (73672 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-5.27, -0.52] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2904, differing 0.002651, SSIM₈ 0.9945 (raw 0.3003, 0.002689, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9912→0.9913 / 0.48→0.4641; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6199→0.6103 / 15.8368→15.9842 [261.4,84.2–349.7,122.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 1.75 Δy 0.0 len 44.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -0.01 dy 6.63; `Before` dx 0 dy 6.63; `the` dx -0.0 dy 6.63
- word-sequence differences: insert ref [] ours ['?', '∑', '?=', '1']; replace ref ['n', 'i=1', 'i=', 'n(n'] ours ['?', '=', '?', '(?']

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
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [308.06, 686.88, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934494, 615, 462, 422, 293, 300, 231, 241, 314, 148, 156, 169, 185, 134, 160, 492]`; ink px ref/ours 1971/1865 (ratio 0.9462); SSIM blocks <0.9: 204/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [13.42, 0.92] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2634, differing 0.002649, SSIM₈ 0.9946 (raw 0.2634, 0.002649, 0.9946)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9914→0.9914 / 0.421→0.421; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5775→0.5775 / 14.5445→14.5445 [261.1,82.4–349.7,128.4 pt]
- largest word displacements (pt): `2` dx -2.26 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'n(n+1)', 'i=1i=']; insert ref [] ours ['(1)']

### 07-math-display — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [303.542, 690.0, 44.64, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934253, 496, 391, 379, 260, 262, 311, 264, 249, 210, 203, 180, 192, 176, 163, 827]`; ink px ref/ours 1971/1965 (ratio 0.997); SSIM blocks <0.9: 216/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3217, not lower; centroid estimate [7.23, -0.49] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3167, differing 0.002749, SSIM₈ 0.9943 (raw 0.3167, 0.002749, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9908→0.9908 / 0.5062→0.5062; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6618→0.6618 / 13.5871→13.5871 [261.1,82.4–349.7,128.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 1.75 Δy 0.0 len 44.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx 8.81 dy 0.34; `display.` dx 3.62 dy 6.37; `the` dx 1.06 dy 6.37
- word-sequence differences: insert ref [] ours ['?', '∑', '?=', '1']; replace ref ['n', '∑', 'i=1', 'i=', 'n(n'] ours ['?', '=', '?', '(?']

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
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [308.06, 686.88, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933816, 403, 356, 427, 352, 372, 304, 278, 301, 217, 254, 216, 304, 174, 242, 800]`; ink px ref/ours 1616/1865 (ratio 1.1541); SSIM blocks <0.9: 248/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-2.26, 1.24] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3541, differing 0.002914, SSIM₈ 0.9928 (raw 0.3541, 0.002914, 0.9928)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9885→0.9885 / 0.566→0.566; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5943→0.5943 / 14.561→14.561 [261.1,82.3–349.7,128.3 pt]
- largest word displacements (pt): `display.` dx -4.73 dy 2.12; `display.` dx -3.41 dy 0.73; `2` dx -2.26 dy 2.25
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'n(n+1)', 'i=1i=']; insert ref [] ours ['(1)']

### 07-math-display — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [303.542, 690.0, 44.64, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934304, 381, 434, 321, 355, 352, 320, 314, 287, 230, 229, 168, 174, 146, 154, 647]`; ink px ref/ours 1616/1965 (ratio 1.216); SSIM blocks <0.9: 212/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-8.45, -0.18] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2918, differing 0.002611, SSIM₈ 0.9945 (raw 0.3009, 0.002645, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9912→0.9913 / 0.4809→0.4663; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7012→0.7031 / 12.8967→13.0014 [261.1,82.3–349.7,128.3 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 1.75 Δy 0.0 len 44.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx 0.01 dy 6.63; `Before` dx 0 dy 6.63; `the` dx 0.0 dy 6.63
- word-sequence differences: insert ref [] ours ['?', '∑', '?=', '1']; replace ref ['n', '∑', 'i=1', 'i=', 'n(n'] ours ['?', '=', '?', '(?']

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1520055, 31716, 27435, 25609, 22467, 22765, 22818, 20885, 21580, 21981, 20475, 20075, 20772, 20283, 20130, 99770]`; ink px ref/ours 152381/145750 (ratio 0.9565); SSIM blocks <0.9: 16622/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.0, 1.0] pt REJECTED: applying it gives mean|Δ| 33.0253, not lower; centroid estimate [-3.46, -2.97] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 32.9083, differing 0.239416, SSIM₈ 0.4723 (raw 32.9083, 0.239416, 0.4723)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1574→0.1574 / 52.584→52.584; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9989→0.9989 / 0.0437→0.0437
- page 2: |Δ| histogram (16 bins, pixel counts) `[1514949, 31797, 27435, 25329, 23194, 22178, 23111, 20991, 21686, 22172, 20716, 20366, 21318, 20116, 20455, 103003]`; ink px ref/ours 151462/145127 (ratio 0.9582); SSIM blocks <0.9: 16869/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 2.0] pt by ink-projection correlation (centroid estimate [-3.05, -1.06] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 33.1666, differing 0.240652, SSIM₈ 0.4703 (raw 33.4992, 0.242511, 0.4593)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1367→0.1543 / 53.5293→52.9976; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9989→0.9989 / 0.0284→0.0284
- page 3: |Δ| histogram (16 bins, pixel counts) `[1820542, 8704, 7443, 6964, 6190, 6267, 6561, 5937, 5954, 6000, 5650, 5558, 5845, 5591, 5774, 29836]`; ink px ref/ours 33623/47458 (ratio 1.4115); SSIM blocks <0.9: 4943/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 49.0] pt by ink-projection correlation (centroid estimate [-2.99, 36.31] pt); confidence moderate (shift explains 13% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.1637, differing 0.061231, SSIM₈ 0.8687 (raw 9.4361, 0.067678, 0.8426)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7486→0.8337 / 15.0799→10.8334; header-band 1.0→0.7017 / 0.0→15.23; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 78.46; `branch` dx -435.47 dy 66.06; `branch` dx -435.47 dy 60.52

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1555403, 30464, 26222, 24502, 22637, 22420, 23203, 22359, 22283, 21332, 18820, 18660, 18216, 17639, 17127, 77529]`; ink px ref/ours 104967/145750 (ratio 1.3885); SSIM blocks <0.9: 16296/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -27.0] pt by ink-projection correlation (centroid estimate [-2.53, -2.2] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 28.7887, differing 0.220223, SSIM₈ 0.4819 (raw 28.8185, 0.220086, 0.4824)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.174→0.198 / 46.0469→44.613; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.8271 / 0.0747→9.5919
- page 2: |Δ| histogram (16 bins, pixel counts) `[1549152, 30396, 26629, 24608, 23293, 22050, 23893, 23096, 22154, 21587, 18822, 18552, 19106, 17435, 17852, 80191]`; ink px ref/ours 104819/145127 (ratio 1.3845); SSIM blocks <0.9: 16511/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -12.5] pt by ink-projection correlation (centroid estimate [-3.26, -0.89] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 29.2792, differing 0.222977, SSIM₈ 0.4703 (raw 29.4271, 0.223709, 0.472)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.157→0.1656 / 47.0258→46.1545; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9981→0.9221 / 0.033→4.3781
- page 3: |Δ| histogram (16 bins, pixel counts) `[1785744, 11727, 10270, 9645, 8931, 8772, 9558, 9189, 8972, 8152, 7465, 7261, 7531, 6985, 7129, 31485]`; ink px ref/ours 46386/47458 (ratio 1.0231); SSIM blocks <0.9: 6884/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 5.5] pt by ink-projection correlation (centroid estimate [-2.24, -35.61] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 10.4253, differing 0.08179, SSIM₈ 0.8055 (raw 11.5903, 0.087963, 0.779)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.647→0.6914 / 18.5235→16.6245; header-band 1.0→0.9874 / 0.0→0.2458; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -81.16; `oak` dx 425.86 dy -79.12; `oak` dx 425.86 dy -70.21

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p1-overlay.png) (56489 B, ÷8), [heatmap](images/08-two-page/pdflatex-de1020c-export-p1-heatmap.png) (38381 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -14.5] pt by ink-projection correlation (centroid estimate [-6.07, -5.06] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.4928, differing 0.229367, SSIM₈ 0.4891 (raw 31.5764, 0.22951, 0.4881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1821→0.1892 / 50.4615→50.1371; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.96 / 0.0468→1.3594
- page 2: |Δ| histogram (16 bins, pixel counts) `[1518465, 30739, 25693, 24091, 22137, 22533, 22535, 20944, 20824, 22593, 20919, 20497, 20504, 19480, 20990, 105872]`; ink px ref/ours 150460/131743 (ratio 0.8756); SSIM blocks <0.9: 16800/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p2-overlay.png) (54928 B, ÷8), [heatmap](images/08-two-page/pdflatex-de1020c-export-p2-heatmap.png) (38766 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 6.0] pt by ink-projection correlation (centroid estimate [-5.52, 7.86] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.0952, differing 0.225993, SSIM₈ 0.4952 (raw 33.6289, 0.239479, 0.4516)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1237→0.1935 / 53.7443→49.6907; header-band 1.0→0.999 / 0.0→0.0276; footer-band 0.9986→0.9986 / 0.0304→0.0304
- page 3: |Δ| histogram (16 bins, pixel counts) `[1774335, 11537, 9689, 9104, 8316, 8407, 8934, 8213, 7615, 9012, 8045, 7866, 7928, 7590, 8264, 43961]`; ink px ref/ours 33504/71008 (ratio 2.1194); SSIM blocks <0.9: 7408/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p3-overlay.png) (66508 B, ÷4), [heatmap](images/08-two-page/pdflatex-de1020c-export-p3-heatmap.png) (59486 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 34.5] pt by ink-projection correlation (centroid estimate [-6.67, 111.28] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.0633, differing 0.081669, SSIM₈ 0.8097 (raw 13.387, 0.093579, 0.7605)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6172→0.7105 / 21.3963→16.9124; header-band 1.0→0.8991 / 0.0→5.299; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 214.06; `branch` dx -435.47 dy 167.32; `over` dx -407.04 dy 214.02

### 08-two-page — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1520838, 31563, 27074, 24807, 22509, 23307, 22498, 21004, 21495, 22304, 20775, 19954, 20253, 19497, 19992, 100946]`; ink px ref/ours 151753/145750 (ratio 0.9604); SSIM blocks <0.9: 16544/30294; [overlay](images/08-two-page/pdflatex-main-export-p1-overlay.png) (57476 B, ÷8), [heatmap](images/08-two-page/pdflatex-main-export-p1-heatmap.png) (38625 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-11.0, 0.0] pt REJECTED: applying it gives mean|Δ| 32.933, not lower; centroid estimate [-4.21, -3.98] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 32.9189, differing 0.238789, SSIM₈ 0.4737 (raw 32.9189, 0.238789, 0.4737)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1593→0.1593 / 52.6066→52.6066; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.9984 / 0.0468→0.0468
- page 2: |Δ| histogram (16 bins, pixel counts) `[1513922, 31893, 26939, 25121, 23067, 23160, 22863, 21203, 21382, 22317, 20570, 20477, 21252, 19697, 20719, 104234]`; ink px ref/ours 150460/145127 (ratio 0.9646); SSIM blocks <0.9: 16824/30294; [overlay](images/08-two-page/pdflatex-main-export-p2-overlay.png) (57246 B, ÷8), [heatmap](images/08-two-page/pdflatex-main-export-p2-heatmap.png) (38914 B, ÷8)
  - registration error (diagnostic): global shift [-11.0, 2.0] pt by ink-projection correlation (centroid estimate [-3.62, -2.08] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 33.2713, differing 0.240489, SSIM₈ 0.4674 (raw 33.6428, 0.242458, 0.4594)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1364→0.152 / 53.7661→53.0399; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9986→0.9986 / 0.0304→0.0304
- page 3: |Δ| histogram (16 bins, pixel counts) `[1820902, 8714, 7360, 6872, 6087, 6378, 6484, 6020, 5875, 6066, 5575, 5674, 5771, 5456, 5700, 29882]`; ink px ref/ours 33504/47458 (ratio 1.4165); SSIM blocks <0.9: 4939/30294; [overlay](images/08-two-page/pdflatex-main-export-p3-overlay.png) (55350 B, ÷4), [heatmap](images/08-two-page/pdflatex-main-export-p3-heatmap.png) (45393 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 49.0] pt by ink-projection correlation (centroid estimate [-3.76, 35.26] pt); confidence moderate (shift explains 13% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.1787, differing 0.061169, SSIM₈ 0.8693 (raw 9.4108, 0.067455, 0.8433)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7496→0.8346 / 15.0411→10.8589; header-band 1.0→0.7017 / 0.0→15.23; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 78.46; `branch` dx -435.47 dy 66.06; `branch` dx -435.47 dy 60.52

### 08-two-page — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1583040, 30686, 25741, 23074, 21550, 20486, 19257, 19723, 18532, 18146, 17688, 17169, 15740, 15834, 16380, 75770]`; ink px ref/ours 151753/137064 (ratio 0.9032); SSIM blocks <0.9: 14229/30294; [overlay](images/08-two-page/pdflatex-pipeline-export-p1-overlay.png) (57740 B, ÷8), [heatmap](images/08-two-page/pdflatex-pipeline-export-p1-heatmap.png) (36459 B, ÷8)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.1, -2.23] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.6362, differing 0.205191, SSIM₈ 0.5954 (raw 26.7545, 0.205539, 0.5933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3515→0.3576 / 42.7356→42.4824; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9951→0.9953 / 0.1484→0.1484
- page 2: |Δ| histogram (16 bins, pixel counts) `[1583820, 30040, 25563, 22832, 21473, 20749, 19258, 19461, 18706, 18132, 17581, 17090, 16116, 15669, 16524, 75802]`; ink px ref/ours 150460/137140 (ratio 0.9115); SSIM blocks <0.9: 14259/30294; [overlay](images/08-two-page/pdflatex-pipeline-export-p2-overlay.png) (57790 B, ÷8), [heatmap](images/08-two-page/pdflatex-pipeline-export-p2-heatmap.png) (36509 B, ÷8)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-0.89, -1.32] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.5326, differing 0.204525, SSIM₈ 0.5956 (raw 26.7672, 0.204965, 0.593)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3508→0.3578 / 42.7661→42.3267; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9971→0.9966 / 0.0778→0.0778
- page 3: |Δ| histogram (16 bins, pixel counts) `[1810605, 10099, 8603, 7598, 7344, 6930, 6661, 7094, 6413, 6678, 6224, 6551, 5790, 5958, 6244, 30024]`; ink px ref/ours 33504/60680 (ratio 1.8111); SSIM blocks <0.9: 5804/30294; [overlay](images/08-two-page/pdflatex-pipeline-export-p3-overlay.png) (63250 B, ÷4), [heatmap](images/08-two-page/pdflatex-pipeline-export-p3-heatmap.png) (49081 B, ÷4)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.41, 71.68] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 9.9261, differing 0.073644, SSIM₈ 0.8282 (raw 9.9944, 0.07373, 0.8269)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7236→0.7269 / 15.9719→15.8354; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 144.86; `branch` dx -413.61 dy 144.86; `oak` dx -415.72 dy 130.42

### 08-two-page — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p1-overlay.png) (55039 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p1-heatmap.png) (37635 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.44, -3.32] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4403, differing 0.210395, SSIM₈ 0.5038 (raw 27.5467, 0.210674, 0.5026)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.208 / 44.0147→43.8447; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1556053, 29239, 25104, 24114, 22751, 21524, 23492, 22483, 22053, 21728, 19173, 18342, 18558, 16712, 18303, 79187]`; ink px ref/ours 105011/131743 (ratio 1.2546); SSIM blocks <0.9: 16449/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p2-overlay.png) (54063 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p2-heatmap.png) (38385 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 36.5] pt by ink-projection correlation (centroid estimate [-5.19, 9.09] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.3288, differing 0.209484, SSIM₈ 0.5027 (raw 29.0579, 0.219561, 0.4664)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.148→0.2198 / 46.4361→42.915; header-band 1.0→0.9042 / 0.0→5.2131; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746783, 14069, 12340, 11757, 11133, 10758, 11588, 11172, 10690, 10911, 9823, 9052, 9221, 8387, 9339, 41793]`; ink px ref/ours 46441/71008 (ratio 1.529); SSIM blocks <0.9: 8608/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p3-overlay.png) (80952 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p3-heatmap.png) (66037 B, ÷4)
  - registration error (diagnostic): global shift [-1.0, 52.0] pt by ink-projection correlation (centroid estimate [-5.44, 40.42] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.4866, differing 0.102874, SSIM₈ 0.754 (raw 14.7842, 0.109995, 0.721)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5542→0.6347 / 23.6284→20.0792; header-band 1.0→0.8091 / 0.0→10.1537; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 55.47; `oak` dx 426.94 dy 37.61; `oak` dx 426.94 dy 32.12

### 08-two-page — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1555521, 30291, 26058, 24656, 22827, 22307, 23426, 22015, 22563, 21069, 18990, 18590, 18306, 17279, 17582, 77336]`; ink px ref/ours 105111/145750 (ratio 1.3866); SSIM blocks <0.9: 16299/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p1-overlay.png) (56098 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-main-export-p1-heatmap.png) (38202 B, ÷8)
  - registration error (diagnostic): global shift [-0.5, -27.0] pt by ink-projection correlation (centroid estimate [-2.58, -2.25] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 28.7887, differing 0.220226, SSIM₈ 0.482 (raw 28.8184, 0.220067, 0.4825)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.174→0.198 / 46.0467→44.613; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.8271 / 0.0747→9.5919
- page 2: |Δ| histogram (16 bins, pixel counts) `[1549298, 30146, 26517, 24812, 23404, 21960, 24086, 22807, 22411, 21396, 18925, 18468, 19176, 17068, 18355, 79987]`; ink px ref/ours 105011/145127 (ratio 1.382); SSIM blocks <0.9: 16512/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p2-overlay.png) (56046 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-main-export-p2-heatmap.png) (38574 B, ÷8)
  - registration error (diagnostic): global shift [-0.5, 2.0] pt by ink-projection correlation (centroid estimate [-3.28, -0.84] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 29.2792, differing 0.222867, SSIM₈ 0.4785 (raw 29.4286, 0.223692, 0.472)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.157→0.1678 / 47.0281→46.7859; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1785844, 11520, 10293, 9712, 9016, 8675, 9645, 9066, 9095, 8053, 7541, 7219, 7542, 6841, 7359, 31395]`; ink px ref/ours 46441/47458 (ratio 1.0219); SSIM blocks <0.9: 6883/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p3-overlay.png) (69381 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-main-export-p3-heatmap.png) (56302 B, ÷4)
  - registration error (diagnostic): global shift [-0.5, 5.5] pt by ink-projection correlation (centroid estimate [-2.52, -35.59] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 10.426, differing 0.081791, SSIM₈ 0.8055 (raw 11.5909, 0.087982, 0.779)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.647→0.6914 / 18.5244→16.6256; header-band 1.0→0.9874 / 0.0→0.2458; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -80.13; `oak` dx 425.86 dy -78.09; `oak` dx 425.86 dy -69.19

### 08-two-page — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1678891, 30013, 24898, 23141, 21598, 20781, 18655, 17662, 14743, 12707, 11482, 10476, 9413, 8881, 8415, 27060]`; ink px ref/ours 105111/137064 (ratio 1.304); SSIM blocks <0.9: 11045/30294; [overlay](images/08-two-page/pdflatex-lm-pipeline-export-p1-overlay.png) (54604 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-pipeline-export-p1-heatmap.png) (81964 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.48, -0.49] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9984, differing 0.157345, SSIM₈ 0.7771 (raw 15.9984, 0.157345, 0.7771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6447→0.6447 / 25.5546→25.5546; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9981→0.9981 / 0.0813→0.0813
- page 2: |Δ| histogram (16 bins, pixel counts) `[1678712, 30153, 24763, 23163, 21564, 20782, 18781, 17551, 14720, 12926, 11335, 10477, 9551, 8754, 8450, 27134]`; ink px ref/ours 105011/137140 (ratio 1.306); SSIM blocks <0.9: 11088/30294; [overlay](images/08-two-page/pdflatex-lm-pipeline-export-p2-overlay.png) (54589 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-pipeline-export-p2-heatmap.png) (82331 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.55, -0.09] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 16.0143, differing 0.157415, SSIM₈ 0.7761 (raw 16.0143, 0.157415, 0.7761)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.643→0.643 / 25.5873→25.5873; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9993→0.9993 / 0.0303→0.0303
- page 3: |Δ| histogram (16 bins, pixel counts) `[1824092, 13273, 10951, 10271, 9628, 9232, 8087, 7928, 6429, 5489, 5060, 4590, 4207, 3857, 3709, 12013]`; ink px ref/ours 46441/60680 (ratio 1.3066); SSIM blocks <0.9: 4904/30294; [overlay](images/08-two-page/pdflatex-lm-pipeline-export-p3-overlay.png) (74129 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-pipeline-export-p3-heatmap.png) (45946 B, ÷4)
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1519809, 31249, 27150, 25991, 22791, 23249, 22910, 20903, 21466, 21981, 20847, 20488, 20635, 20015, 19732, 99600]`; ink px ref/ours 152232/145750 (ratio 0.9574); SSIM blocks <0.9: 16615/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.0, 1.0] pt REJECTED: applying it gives mean|Δ| 33.0193, not lower; centroid estimate [-3.49, -2.94] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 32.9002, differing 0.239463, SSIM₈ 0.4726 (raw 32.9002, 0.239463, 0.4726)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1579→0.1579 / 52.571→52.571; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9987→0.9987 / 0.0438→0.0438
- page 2: |Δ| histogram (16 bins, pixel counts) `[1514989, 31510, 26933, 25750, 23302, 22720, 23071, 21028, 21896, 22251, 21212, 20698, 21009, 19836, 20053, 102558]`; ink px ref/ours 151458/145127 (ratio 0.9582); SSIM blocks <0.9: 16861/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 2.0] pt by ink-projection correlation (centroid estimate [-3.13, -1.16] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 33.1057, differing 0.240378, SSIM₈ 0.4714 (raw 33.452, 0.242246, 0.4602)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1381→0.1561 / 53.4537→52.9001; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9989→0.9989 / 0.0285→0.0285
- page 3: |Δ| histogram (16 bins, pixel counts) `[1820465, 8634, 7329, 7150, 6195, 6357, 6539, 5878, 5920, 6008, 5825, 5693, 5800, 5563, 5743, 29717]`; ink px ref/ours 33659/47458 (ratio 1.41); SSIM blocks <0.9: 4939/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 49.0] pt by ink-projection correlation (centroid estimate [-2.98, 36.17] pt); confidence moderate (shift explains 13% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.1788, differing 0.061329, SSIM₈ 0.8684 (raw 9.4369, 0.067727, 0.8427)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7487→0.8331 / 15.0812→10.8575; header-band 1.0→0.7017 / 0.0→15.23; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 78.46; `branch` dx -435.52 dy 66.06; `branch` dx -435.52 dy 60.52

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1555389, 30504, 26238, 24493, 22519, 22489, 23291, 22233, 22382, 21192, 18927, 18604, 18239, 17654, 17107, 77555]`; ink px ref/ours 105020/145750 (ratio 1.3878); SSIM blocks <0.9: 16300/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -27.0] pt by ink-projection correlation (centroid estimate [-2.6, -2.13] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 28.7888, differing 0.220187, SSIM₈ 0.4819 (raw 28.8189, 0.22004, 0.4824)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.174→0.198 / 46.0476→44.6131; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.8271 / 0.0747→9.592
- page 2: |Δ| histogram (16 bins, pixel counts) `[1549136, 30439, 26676, 24552, 23232, 22053, 24028, 22993, 22213, 21450, 18947, 18507, 19084, 17457, 17862, 80187]`; ink px ref/ours 104857/145127 (ratio 1.384); SSIM blocks <0.9: 16513/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -12.5] pt by ink-projection correlation (centroid estimate [-3.34, -0.91] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 29.2793, differing 0.222935, SSIM₈ 0.4703 (raw 29.427, 0.223651, 0.472)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.157→0.1656 / 47.0256→46.1546; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9981→0.9221 / 0.033→4.3782
- page 3: |Δ| histogram (16 bins, pixel counts) `[1785725, 11741, 10294, 9632, 8894, 8798, 9580, 9178, 8992, 8081, 7510, 7264, 7520, 6983, 7116, 31508]`; ink px ref/ours 46395/47458 (ratio 1.0229); SSIM blocks <0.9: 6885/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 5.5] pt by ink-projection correlation (centroid estimate [-2.3, -35.58] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 10.4255, differing 0.081775, SSIM₈ 0.8055 (raw 11.5905, 0.087945, 0.7789)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.647→0.6914 / 18.5238→16.6248; header-band 1.0→0.9874 / 0.0→0.2458; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -80.13; `oak` dx 425.86 dy -78.09; `oak` dx 425.86 dy -69.19

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
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [288.61, 632.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908246, 2035, 1831, 1910, 1508, 1488, 1544, 1411, 1385, 1423, 1698, 1502, 1432, 1498, 1436, 8469]`; ink px ref/ours 9691/9418 (ratio 0.9718); SSIM blocks <0.9: 1564/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 38.5] pt by ink-projection correlation (centroid estimate [9.64, 31.03] pt); confidence strong (shift explains 34% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.6551, differing 0.013719, SSIM₈ 0.9717 (raw 2.5088, 0.017511, 0.9499)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9199→0.9573 / 4.0097→2.5079; header-band 1.0→0.9829 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6325→0.7572 / 15.3393→9.3809 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -3.65 dy 39.06; `in` dx -3.44 dy 39.06; `set` dx -3.24 dy 39.06
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.364, 653.052, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1915175, 1926, 1678, 1574, 1523, 1310, 1446, 1293, 1233, 1219, 1193, 1110, 1052, 1125, 1042, 4917]`; ink px ref/ours 9691/9269 (ratio 0.9565); SSIM blocks <0.9: 911/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.0, 0.0] pt REJECTED: applying it gives mean|Δ| 1.804, not lower; centroid estimate [15.7, 1.11] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.7735, differing 0.013618, SSIM₈ 0.9736 (raw 1.7735, 0.013618, 0.9736)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9578→0.9578 / 2.8346→2.8346; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.799→0.799 / 11.2987→11.2987 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -436.13 dy 15.03; `paper.` dx 53.16 dy 0.58; `letter` dx 49.48 dy 0.58
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2'] ours ['?', '2']

### 09-mixed-document — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909444, 2180, 1960, 2265, 1611, 1596, 1560, 1754, 1443, 1429, 1644, 1400, 1251, 1263, 1288, 6728]`; ink px ref/ours 7447/9252 (ratio 1.2424); SSIM blocks <0.9: 1580/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 39.0] pt by ink-projection correlation (centroid estimate [0.63, 34.84] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9017, differing 0.014708, SSIM₈ 0.9641 (raw 2.2538, 0.016899, 0.9494)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9192→0.9448 / 3.6023→2.9175; header-band 1.0→0.9856 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6352→0.7524 / 13.0739→8.01 [275.7,96.4–519.8,156 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 24.19; `paper.` dx -52.18 dy 38.59; `letter` dx -48.63 dy 38.59
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [288.61, 632.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909340, 2188, 2000, 2239, 1567, 1573, 1552, 1764, 1454, 1434, 1625, 1423, 1278, 1325, 1232, 6822]`; ink px ref/ours 7447/9418 (ratio 1.2647); SSIM blocks <0.9: 1591/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 39.0] pt by ink-projection correlation (centroid estimate [4.0, 33.73] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8972, differing 0.014691, SSIM₈ 0.9637 (raw 2.268, 0.016945, 0.949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9186→0.9446 / 3.6249→2.8949; header-band 1.0→0.9825 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.632→0.7532 / 13.0739→7.8507 [275.7,96.4–519.8,156 pt]
- largest word displacements (pt): `twelve` dx 432.48 dy 24.19; `paper.` dx -52.82 dy 38.59; `letter` dx -49.27 dy 38.59
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.364, 653.052, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1918849, 1938, 1870, 1875, 1690, 1403, 1288, 1202, 1011, 993, 871, 812, 801, 747, 714, 2752]`; ink px ref/ours 7447/9269 (ratio 1.2447); SSIM blocks <0.9: 815/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.5] pt by ink-projection correlation (centroid estimate [10.06, 3.81] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2552, differing 0.011554, SSIM₈ 0.982 (raw 1.3023, 0.01178, 0.981)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9697→0.9713 / 2.0815→2.0059; header-band 1.0→0.9994 / 0.0→0.002; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.837→0.8239 / 8.4236→9.418 [275.7,96.4–519.8,156 pt]
- largest word displacements (pt): `Mixed` dx 29.06 dy -0.67; `=` dx 0.86 dy 0.66; `inline` dx 0.91 dy -0.48
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2'] ours ['?', '2']

### 09-mixed-document — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908448, 1975, 1818, 1995, 1553, 1434, 1436, 1374, 1435, 1503, 1793, 1550, 1349, 1400, 1470, 8283]`; ink px ref/ours 9857/9252 (ratio 0.9386); SSIM blocks <0.9: 1551/30294; [overlay](images/09-mixed-document/pdflatex-de1020c-export-p1-overlay.png) (53663 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-de1020c-export-p1-heatmap.png) (55253 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 38.5] pt by ink-projection correlation (centroid estimate [7.57, 32.71] pt); confidence moderate (shift explains 24% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8886, differing 0.014393, SSIM₈ 0.9685 (raw 2.4853, 0.017384, 0.9504)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9207→0.9518 / 3.9723→2.8965; header-band 1.0→0.9857 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6361→0.7546 / 15.3285→9.5698 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `twelve` dx -3.05 dy 39.17; `in` dx -2.78 dy 39.17; `set` dx -2.52 dy 39.17
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [288.61, 632.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908288, 1965, 1866, 1969, 1496, 1406, 1434, 1380, 1445, 1488, 1794, 1574, 1372, 1454, 1428, 8457]`; ink px ref/ours 9857/9418 (ratio 0.9555); SSIM blocks <0.9: 1566/30294; [overlay](images/09-mixed-document/pdflatex-main-export-p1-overlay.png) (52796 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-main-export-p1-heatmap.png) (55066 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 38.5] pt by ink-projection correlation (centroid estimate [10.94, 31.6] pt); confidence strong (shift explains 26% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8697, differing 0.014363, SSIM₈ 0.9687 (raw 2.51, 0.017464, 0.9498)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9198→0.9525 / 4.0117→2.8509; header-band 1.0→0.9829 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.633→0.7576 / 15.3348→9.382 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `twelve` dx -3.83 dy 39.17; `in` dx -3.56 dy 39.17; `set` dx -3.3 dy 39.17
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.364, 653.052, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1914707, 1842, 1699, 1691, 1614, 1346, 1307, 1304, 1334, 1202, 1265, 1094, 1049, 1116, 1071, 5175]`; ink px ref/ours 9857/9269 (ratio 0.9403); SSIM blocks <0.9: 916/30294; [overlay](images/09-mixed-document/pdflatex-pipeline-export-p1-overlay.png) (53725 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-pipeline-export-p1-heatmap.png) (46239 B, ÷2)
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [17.0, 1.68] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8051, differing 0.01372, SSIM₈ 0.973 (raw 1.8209, 0.013784, 0.9729)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9568→0.9571 / 2.9104→2.8835; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.791→0.7672 / 11.7097→12.6597 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `twelve` dx -436.31 dy 15.14; `paper.` dx 53.76 dy 0.69; `letter` dx 50.09 dy 0.69
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2'] ours ['?', '2']

### 09-mixed-document — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909812, 2060, 1849, 1927, 1609, 1602, 1682, 1704, 1540, 1378, 1642, 1427, 1316, 1278, 1363, 6627]`; ink px ref/ours 7573/9252 (ratio 1.2217); SSIM blocks <0.9: 1632/30294; [overlay](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-overlay.png) (54205 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-heatmap.png) (55534 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 39.5] pt by ink-projection correlation (centroid estimate [-1.26, 34.72] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9012, differing 0.014685, SSIM₈ 0.9623 (raw 2.2522, 0.016745, 0.948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9169→0.9418 / 3.5997→2.9167; header-band 1.0→0.9856 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.381→0.5284 / 23.145→12.7296 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 25.83; `paper.` dx -52.2 dy 40.23; `letter` dx -48.65 dy 40.23
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [288.61, 632.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909701, 2067, 1886, 1907, 1559, 1585, 1671, 1726, 1552, 1374, 1628, 1446, 1343, 1338, 1307, 6726]`; ink px ref/ours 7573/9418 (ratio 1.2436); SSIM blocks <0.9: 1643/30294; [overlay](images/09-mixed-document/pdflatex-lm-main-export-p1-overlay.png) (53331 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-main-export-p1-heatmap.png) (55351 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 39.5] pt by ink-projection correlation (centroid estimate [2.11, 33.61] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8976, differing 0.01467, SSIM₈ 0.9619 (raw 2.2668, 0.016793, 0.9476)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9163→0.9417 / 3.623→2.8955; header-band 1.0→0.9826 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3685→0.5048 / 23.2695→12.7299 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `twelve` dx 432.48 dy 25.83; `paper.` dx -52.84 dy 40.23; `letter` dx -49.29 dy 40.23
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.364, 653.052, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1917322, 1842, 1719, 1764, 1735, 1449, 1463, 1337, 1225, 1098, 1037, 956, 847, 906, 837, 3279]`; ink px ref/ours 7573/9269 (ratio 1.224); SSIM blocks <0.9: 885/30294; [overlay](images/09-mixed-document/pdflatex-lm-pipeline-export-p1-overlay.png) (53166 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-pipeline-export-p1-heatmap.png) (45581 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 1.0] pt by ink-projection correlation (centroid estimate [8.17, 3.68] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3175, differing 0.011856, SSIM₈ 0.9792 (raw 1.4737, 0.01255, 0.9767)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9628→0.967 / 2.3555→2.1013; header-band 1.0→0.9975 / 0.0→0.0308; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6904→0.6976 / 11.9769→12.2545 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `Mixed` dx 29.06 dy 0.96; `second` dx 0.03 dy 1.8; `line` dx 0.03 dy 1.8
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2'] ours ['?', '2']

### 09-mixed-document — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908485, 1993, 1726, 1891, 1568, 1563, 1547, 1422, 1387, 1527, 1642, 1498, 1332, 1389, 1540, 8306]`; ink px ref/ours 9680/9252 (ratio 0.9558); SSIM blocks <0.9: 1553/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 38.5] pt by ink-projection correlation (centroid estimate [7.53, 32.15] pt); confidence strong (shift explains 25% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8537, differing 0.01428, SSIM₈ 0.969 (raw 2.4838, 0.017418, 0.9503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9206→0.9525 / 3.9698→2.8407; header-band 1.0→0.9857 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6356→0.7542 / 15.3342→9.5697 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -2.91 dy 39.06; `in` dx -2.65 dy 39.06; `set` dx -2.4 dy 39.06
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [288.61, 632.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908309, 1986, 1783, 1855, 1518, 1546, 1545, 1430, 1391, 1495, 1633, 1525, 1365, 1450, 1503, 8482]`; ink px ref/ours 9680/9418 (ratio 0.9729); SSIM blocks <0.9: 1568/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 38.5] pt by ink-projection correlation (centroid estimate [10.9, 31.05] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8378, differing 0.014258, SSIM₈ 0.969 (raw 2.5096, 0.017501, 0.9498)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9197→0.953 / 4.011→2.8; header-band 1.0→0.9829 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6325→0.7572 / 15.3393→9.3809 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -3.69 dy 39.06; `in` dx -3.43 dy 39.06; `set` dx -3.18 dy 39.06
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.364, 653.052, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1915096, 1895, 1625, 1569, 1549, 1391, 1421, 1284, 1237, 1234, 1146, 1090, 986, 1121, 1086, 5086]`; ink px ref/ours 9680/9269 (ratio 0.9575); SSIM blocks <0.9: 912/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [16.95, 1.12] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.7793, differing 0.013616, SSIM₈ 0.9733 (raw 1.7898, 0.013667, 0.9733)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9573→0.9575 / 2.8607→2.8422; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7992→0.7702 / 11.2835→12.5688 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -436.17 dy 15.03; `paper.` dx 53.77 dy 0.58; `letter` dx 50.1 dy 0.58
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2'] ours ['?', '2']

### 09-mixed-document — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909464, 2171, 1943, 2272, 1611, 1596, 1548, 1766, 1439, 1468, 1597, 1418, 1229, 1257, 1310, 6727]`; ink px ref/ours 7446/9252 (ratio 1.2425); SSIM blocks <0.9: 1581/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 39.0] pt by ink-projection correlation (centroid estimate [0.78, 34.82] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9016, differing 0.014705, SSIM₈ 0.9641 (raw 2.2539, 0.016899, 0.9494)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9191→0.9448 / 3.6023→2.9173; header-band 1.0→0.9856 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3967→0.5342 / 23.5725→12.9044 [275.7,121.1–336.2,156 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 25.22; `paper.` dx -52.18 dy 39.62; `letter` dx -48.63 dy 39.62
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [288.61, 632.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909360, 2179, 1983, 2246, 1567, 1573, 1540, 1776, 1450, 1473, 1578, 1441, 1256, 1318, 1254, 6822]`; ink px ref/ours 7446/9418 (ratio 1.2648); SSIM blocks <0.9: 1592/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 39.0] pt by ink-projection correlation (centroid estimate [4.15, 33.71] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8972, differing 0.01469, SSIM₈ 0.9637 (raw 2.268, 0.016944, 0.949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9186→0.9446 / 3.6249→2.8948; header-band 1.0→0.9825 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3845→0.5106 / 23.6675→12.9047 [275.7,121.1–336.2,156 pt]
- largest word displacements (pt): `twelve` dx 432.48 dy 25.22; `paper.` dx -52.82 dy 39.62; `letter` dx -49.27 dy 39.62
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.364, 653.052, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1918822, 1962, 1859, 1843, 1730, 1385, 1313, 1174, 1038, 999, 854, 781, 822, 748, 729, 2757]`; ink px ref/ours 7446/9269 (ratio 1.2448); SSIM blocks <0.9: 816/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.5] pt by ink-projection correlation (centroid estimate [10.21, 3.79] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2567, differing 0.011562, SSIM₈ 0.982 (raw 1.3038, 0.011779, 0.981)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9697→0.9712 / 2.0839→2.0083; header-band 1.0→0.9994 / 0.0→0.002; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7786→0.7704 / 9.4805→10.214 [275.7,121.1–336.2,156 pt]
- largest word displacements (pt): `Mixed` dx 29.06 dy 0.96; `letter` dx 0.04 dy 1.18; `paper.` dx 0.04 dy 1.18
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2'] ours ['?', '2']

### 10-unicode-paragraph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893839, 3905, 3475, 2974, 2739, 2765, 2584, 2505, 2344, 2294, 2314, 2129, 2034, 2031, 2138, 8746]`; ink px ref/ours 18767/18449 (ratio 0.9831); SSIM blocks <0.9: 1824/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.58, 1.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3153, differing 0.026129, SSIM₈ 0.949 (raw 3.3153, 0.026129, 0.949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9186→0.9186 / 5.2982→5.2982; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 11.75 Δy 0.0 len 10.0 vs 11.5, thickness px 1 vs 1
- word-sequence differences: replace ref ['Cremebrulee,jalapeno,facade,naive,cooperate,Angstrom,Ærø,Þorr,Øresund,Skoda,Łodz,Is-', 'tanbul,Zurich—anemdash;1990–1995anendash;“curlydoublequotes”,‘curlysinglequotes’,', '«guillemets»,and‚Germanlowquotes‘.', 'SenorMuller’sresumelistsSaoPaulo,Krakow,Reyk-', 'javik,MalmoandBordeaux;thecafe’smenuofferscrepes,souffleandapatethatcosts€12—', 'or£10,¥1500—perserving,1⁄2portionavailable.', 'Sæglopur,manana,Nandu,cedilla,y,ø,a,æ,', 'œuvre,Œdipe,ß,andDzclosetheline.'] ours ['Creme', 'brulee,', 'jalapeno,', 'facade,', 'naive,', 'cooperate,', 'Angstrom,', 'Ærø,']

### 10-unicode-paragraph — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (5): warning: Times-Roman has no glyph for 'ǅ' (U+01C5); warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893755, 3859, 3353, 2941, 2821, 2780, 2466, 2555, 2464, 2482, 2263, 2132, 2104, 2050, 2046, 8745]`; ink px ref/ours 18767/18531 (ratio 0.9874); SSIM blocks <0.9: 1788/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.22, 0.47] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2478, differing 0.025886, SSIM₈ 0.9509 (raw 3.3305, 0.026154, 0.9495)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9193→0.9218 / 5.3225→5.1896; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
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

- FlashTeX diagnostics (5): warning: Times-Roman has no glyph for 'ǅ' (U+01C5); warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893990, 3688, 3311, 3001, 2824, 2884, 2803, 2873, 2751, 2374, 2291, 2126, 2114, 1938, 1949, 7899]`; ink px ref/ours 14211/18531 (ratio 1.304); SSIM blocks <0.9: 1914/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-2.85, -1.45] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2488, differing 0.02593, SSIM₈ 0.9448 (raw 3.2488, 0.02593, 0.9448)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9118→0.9118 / 5.1925→5.1925; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Łódź,` dx 428.36 dy -14.75; `Kraków,` dx 426.16 dy -14.84; `single` dx 411.7 dy -14.79
- word-sequence differences: replace ref ['�'] ours ['Dz']

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893459, 4089, 3397, 2841, 2581, 2734, 2699, 2550, 2363, 2423, 2285, 2125, 2111, 2043, 2055, 9061]`; ink px ref/ours 18728/18449 (ratio 0.9851); SSIM blocks <0.9: 1812/30294; [overlay](images/10-unicode-paragraph/pdflatex-de1020c-export-p1-overlay.png) (78108 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (60324 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.7, 1.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.362, differing 0.026242, SSIM₈ 0.9486 (raw 3.362, 0.026242, 0.9486)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.918→0.918 / 5.373→5.373; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 12.0 Δy 0.0 len 10.0 vs 12.0, thickness px 1 vs 1
- largest word displacements (pt): `quotes’,` dx -429.81 dy 14.82; `æ,` dx -421.71 dy 14.68; `å,` dx -421.21 dy 14.68
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['C12'] ours ['€12']

### 10-unicode-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (5): warning: Times-Roman has no glyph for 'ǅ' (U+01C5); warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893850, 4023, 3381, 2853, 2611, 2771, 2581, 2546, 2501, 2376, 2328, 2074, 2157, 2017, 2012, 8735]`; ink px ref/ours 18728/18531 (ratio 0.9895); SSIM blocks <0.9: 1783/30294; [overlay](images/10-unicode-paragraph/pdflatex-main-export-p1-overlay.png) (76912 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-main-export-p1-heatmap.png) (59141 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-2.5, 0.0] pt REJECTED: applying it gives mean|Δ| 3.3648, not lower; centroid estimate [-2.34, 0.6] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3173, differing 0.026064, SSIM₈ 0.9499 (raw 3.3173, 0.026064, 0.9499)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.92→0.92 / 5.3014→5.3014; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `æ,` dx -457.04 dy 14.68; `—` dx -456.05 dy 14.72; `quotes’,` dx -429.81 dy 14.82
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['C12'] ours ['€12']

### 10-unicode-paragraph — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893840, 4023, 3312, 2914, 2657, 2593, 2410, 2548, 2288, 2352, 2210, 2193, 2198, 2090, 2089, 9099]`; ink px ref/ours 18728/18366 (ratio 0.9807); SSIM blocks <0.9: 1843/30294; [overlay](images/10-unicode-paragraph/pdflatex-pipeline-export-p1-overlay.png) (78596 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-pipeline-export-p1-heatmap.png) (61346 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.86, 1.94] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3581, differing 0.026339, SSIM₈ 0.9479 (raw 3.3581, 0.026339, 0.9479)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9168→0.9168 / 5.3666→5.3666; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `single` dx -398.5 dy 14.85; `quotes’,` dx -398.1 dy 14.85; `Kraków,` dx -394.31 dy 14.85
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['C12'] ours ['€12']

### 10-unicode-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893882, 3869, 3242, 2959, 2871, 2862, 2862, 2749, 2745, 2384, 2325, 2135, 2070, 1985, 2056, 7820]`; ink px ref/ours 14393/18449 (ratio 1.2818); SSIM blocks <0.9: 1933/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (79141 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (62679 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.76, -1.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.251, differing 0.025995, SSIM₈ 0.9445 (raw 3.251, 0.025995, 0.9445)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9112→0.9112 / 5.1961→5.1961; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 95.5 pt with NO reference rule on this page
- largest word displacements (pt): `that` dx 447.55 dy -14.89; `Łódź,` dx 428.89 dy -14.75; `single` dx 411.29 dy -14.79
- word-sequence differences: replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']

### 10-unicode-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (5): warning: Times-Roman has no glyph for 'ǅ' (U+01C5); warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893824, 3716, 3242, 2971, 2934, 2899, 2776, 2848, 2826, 2318, 2322, 2126, 2206, 1915, 1955, 7938]`; ink px ref/ours 14393/18531 (ratio 1.2875); SSIM blocks <0.9: 1921/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-main-export-p1-overlay.png) (78685 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (61956 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-3.4, -1.84] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2644, differing 0.026019, SSIM₈ 0.9446 (raw 3.2644, 0.026019, 0.9446)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9114→0.9114 / 5.2174→5.2174; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Łódź,` dx 428.36 dy -14.75; `Kraków,` dx 426.16 dy -14.84; `single` dx 411.7 dy -14.79

### 10-unicode-paragraph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1902002, 3984, 3355, 3028, 2858, 2712, 2438, 2400, 2014, 1784, 1705, 1752, 1517, 1290, 1293, 4684]`; ink px ref/ours 14393/18366 (ratio 1.276); SSIM blocks <0.9: 1581/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) (76444 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-pipeline-export-p1-heatmap.png) (56334 B, ÷2)
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

- FlashTeX diagnostics (5): warning: Times-Roman has no glyph for 'ǅ' (U+01C5); warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893751, 3917, 3409, 2937, 2824, 2770, 2413, 2631, 2372, 2332, 2367, 2109, 2151, 2033, 2129, 8671]`; ink px ref/ours 18684/18531 (ratio 0.9918); SSIM blocks <0.9: 1791/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.88, 0.49] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2333, differing 0.025836, SSIM₈ 0.9511 (raw 3.3247, 0.026086, 0.9496)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9195→0.9221 / 5.3132→5.1664; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `æ,` dx -457.07 dy 14.68; `—` dx -456.09 dy 14.72; `quotes’,` dx -429.86 dy 14.82
- word-sequence differences: replace ref ['Is-', 'tanbul,'] ours ['Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']

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

- FlashTeX diagnostics (5): warning: Times-Roman has no glyph for 'ǅ' (U+01C5); warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893964, 3727, 3292, 3001, 2828, 2884, 2800, 2879, 2750, 2368, 2291, 2127, 2122, 1928, 1956, 7899]`; ink px ref/ours 14217/18531 (ratio 1.3034); SSIM blocks <0.9: 1914/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-3.3, -1.46] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2489, differing 0.025925, SSIM₈ 0.9448 (raw 3.2489, 0.025925, 0.9448)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9118→0.9118 / 5.1927→5.1927; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Łódź,` dx 428.36 dy -13.72; `Kraków,` dx 426.16 dy -13.81; `single` dx 411.7 dy -13.77
- word-sequence differences: replace ref ['\uffff'] ours ['Dz']

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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920575, 1363, 1139, 1084, 981, 933, 934, 877, 847, 877, 880, 866, 865, 812, 813, 4970]`; ink px ref/ours 6059/6137 (ratio 1.0129); SSIM blocks <0.9: 942/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-22.5, -8.0] pt by ink-projection correlation (centroid estimate [-26.07, -9.4] pt); confidence moderate (shift explains 24% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.122, differing 0.008873, SSIM₈ 0.98 (raw 1.4714, 0.010489, 0.9699)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9519→0.968 / 2.3517→1.7932; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.2153→0.259 / 35.1216→36.3028 [109.2,144.9–266,183.2 pt]
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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921056, 1232, 1170, 1162, 1016, 1005, 1027, 957, 1021, 942, 819, 982, 916, 731, 758, 4022]`; ink px ref/ours 4803/6137 (ratio 1.2777); SSIM blocks <0.9: 992/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-30.5, -8.0] pt by ink-projection correlation (centroid estimate [-28.01, -9.32] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2508, differing 0.009374, SSIM₈ 0.9755 (raw 1.3743, 0.010157, 0.9689)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9504→0.9608 / 2.1965→1.9992; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3467→0.4901 / 30.8091→28.0342 [109.1,139.7–273.7,184 pt]
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920969, 1439, 1023, 930, 793, 1108, 857, 760, 794, 725, 868, 911, 886, 817, 931, 5005]`; ink px ref/ours 6062/5958 (ratio 0.9828); SSIM blocks <0.9: 995/30294; [overlay](images/11-nested-lists/pdflatex-de1020c-export-p1-overlay.png) (42624 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-de1020c-export-p1-heatmap.png) (43699 B, ÷2)
  - registration error (diagnostic): global shift [19.0, -49.0] pt by ink-projection correlation (centroid estimate [102.36, -62.51] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.4558, differing 0.010208, SSIM₈ 0.9711 (raw 1.4657, 0.01038, 0.9698)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9518→0.9553 / 2.3427→2.2545; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.506→0.3995 / 22.5175→28.0218 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `numbered` dx 309.05 dy -72.76; `First` dx 308.96 dy -72.76; `list.` dx 272.83 dy -126.61
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920680, 1452, 1071, 1083, 924, 959, 967, 834, 829, 919, 814, 882, 860, 749, 888, 4905]`; ink px ref/ours 6062/6137 (ratio 1.0124); SSIM blocks <0.9: 935/30294; [overlay](images/11-nested-lists/pdflatex-main-export-p1-overlay.png) (43183 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-main-export-p1-heatmap.png) (43928 B, ÷2)
  - registration error (diagnostic): global shift [-22.0, -8.0] pt by ink-projection correlation (centroid estimate [-24.76, -9.67] pt); confidence moderate (shift explains 19% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1824, differing 0.009146, SSIM₈ 0.979 (raw 1.4604, 0.010455, 0.9704)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9527→0.9664 / 2.3342→1.8898; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.2574→0.3573 / 34.7602→36.4381 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `First` dx -43.02 dy -11.56; `numbered` dx -42.93 dy -11.56; `Second` dx -43.02 dy -10.59

### 11-nested-lists — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920959, 1469, 995, 913, 887, 1011, 883, 901, 877, 847, 867, 895, 780, 812, 911, 4809]`; ink px ref/ours 6062/6022 (ratio 0.9934); SSIM blocks <0.9: 916/30294; [overlay](images/11-nested-lists/pdflatex-pipeline-export-p1-overlay.png) (43447 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-pipeline-export-p1-heatmap.png) (43455 B, ÷2)
  - registration error (diagnostic): global shift [-16.0, -20.0] pt by ink-projection correlation (centroid estimate [-17.39, -27.85] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3661, differing 0.009829, SSIM₈ 0.9757 (raw 1.4478, 0.010309, 0.9709)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9535→0.9611 / 2.3139→2.1834; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4616→0.2937 / 26.395→38.5846 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `list.` dx 4.91 dy -54.39; `the` dx 2.35 dy -54.39; `After` dx 0 dy -54.39

### 11-nested-lists — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921429, 1208, 1106, 1054, 881, 1201, 884, 878, 1059, 686, 919, 983, 802, 823, 761, 4142]`; ink px ref/ours 4790/5958 (ratio 1.2438); SSIM blocks <0.9: 1048/30294; [overlay](images/11-nested-lists/pdflatex-lm-de1020c-export-p1-overlay.png) (43099 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-de1020c-export-p1-heatmap.png) (43242 B, ÷2)
  - registration error (diagnostic): global shift [23.5, -49.0] pt by ink-projection correlation (centroid estimate [101.54, -62.0] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3216, differing 0.009723, SSIM₈ 0.9706 (raw 1.3674, 0.010055, 0.9687)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.95→0.9547 / 2.1855→2.0271; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6045→0.4278 / 14.6672→25.9327 [108.1,139.7–272.5,184 pt]
- largest word displacements (pt): `First` dx 308.96 dy -73.53; `numbered` dx 305.46 dy -73.53; `After` dx 272.65 dy -127.37
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921052, 1206, 1155, 1188, 1007, 1051, 998, 964, 1103, 884, 858, 948, 784, 760, 727, 4131]`; ink px ref/ours 4790/6137 (ratio 1.2812); SSIM blocks <0.9: 994/30294; [overlay](images/11-nested-lists/pdflatex-lm-main-export-p1-overlay.png) (43702 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-main-export-p1-heatmap.png) (43709 B, ÷2)
  - registration error (diagnostic): global shift [-30.0, -8.0] pt by ink-projection correlation (centroid estimate [-25.58, -9.16] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2546, differing 0.009404, SSIM₈ 0.9752 (raw 1.3761, 0.010178, 0.969)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9504→0.9604 / 2.1994→2.0053; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3438→0.4843 / 30.9535→28.3211 [108.1,139.7–272.5,184 pt]
- largest word displacements (pt): `sub-item.` dx -49.84 dy -12.33; `sub-item.` dx -47.67 dy -11.36; `numbered` dx -46.52 dy -12.33

### 11-nested-lists — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921335, 1309, 1178, 1113, 1018, 1088, 924, 1043, 1099, 793, 881, 924, 683, 756, 744, 3928]`; ink px ref/ours 4790/6022 (ratio 1.2572); SSIM blocks <0.9: 960/30294; [overlay](images/11-nested-lists/pdflatex-lm-pipeline-export-p1-overlay.png) (43755 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-pipeline-export-p1-heatmap.png) (42943 B, ÷2)
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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920552, 1403, 1152, 1070, 987, 916, 919, 890, 857, 852, 866, 874, 876, 833, 814, 4955]`; ink px ref/ours 6057/6137 (ratio 1.0132); SSIM blocks <0.9: 942/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-22.5, -8.0] pt by ink-projection correlation (centroid estimate [-26.02, -9.37] pt); confidence moderate (shift explains 24% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1192, differing 0.008866, SSIM₈ 0.98 (raw 1.4708, 0.01049, 0.9699)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9519→0.968 / 2.3507→1.7888; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.2152→0.2588 / 35.0092→36.1854 [109.2,144.9–266,183.2 pt]
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

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921045, 1235, 1170, 1168, 1018, 980, 1046, 973, 1012, 932, 832, 983, 909, 734, 749, 4030]`; ink px ref/ours 4790/6137 (ratio 1.2812); SSIM blocks <0.9: 992/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-30.5, -8.0] pt by ink-projection correlation (centroid estimate [-28.19, -9.39] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2509, differing 0.009371, SSIM₈ 0.9755 (raw 1.3743, 0.010152, 0.9689)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9504→0.9608 / 2.1966→1.9993; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3001→0.3765 / 29.4433→31.6327 [109.1,143.5–273.7,182.9 pt]
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1749889, 14370, 12460, 11361, 10312, 10070, 10547, 9139, 9936, 9787, 9317, 9183, 9050, 9324, 8981, 45090]`; ink px ref/ours 67719/67696 (ratio 0.9997); SSIM blocks <0.9: 7586/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.96, 9.73] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 14.8082, differing 0.108048, SSIM₈ 0.7605 (raw 14.8416, 0.108175, 0.7604)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6171→0.6177 / 23.721→23.6417; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 32.17; `branch` dx -435.47 dy 26.4; `branch` dx -435.47 dy 20.63

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755413, 14403, 12338, 11855, 11064, 10774, 11057, 10981, 10835, 10360, 8867, 8620, 8720, 8243, 7948, 37338]`; ink px ref/ours 51487/67696 (ratio 1.3148); SSIM blocks <0.9: 8020/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -26.0] pt by ink-projection correlation (centroid estimate [-0.88, -15.57] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7623, differing 0.105511, SSIM₈ 0.7495 (raw 13.7762, 0.105431, 0.7469)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5959→0.6 / 22.0172→21.9931; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -40.77; `oak` dx 425.86 dy -32.1; `oak` dx 425.86 dy -23.42

### 12-justified-paragraphs — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1812418, 14567, 11640, 11333, 10796, 10018, 8952, 8503, 7141, 6678, 5336, 5188, 4639, 4442, 3784, 13381]`; ink px ref/ours 51487/67185 (ratio 1.3049); SSIM blocks <0.9: 5388/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.05, -0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.8017, differing 0.076628, SSIM₈ 0.8918 (raw 7.8017, 0.076628, 0.8918)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8273→0.8273 / 12.4677→12.4677; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `jumps` dx 0.01 dy -0.36; `over` dx 0.01 dy -0.36; `the` dx 0.01 dy -0.36

### 12-justified-paragraphs — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1744770, 14653, 12288, 11028, 10978, 10548, 10474, 9663, 9503, 10351, 9533, 9493, 9214, 8786, 9639, 47895]`; ink px ref/ours 67213/67475 (ratio 1.0039); SSIM blocks <0.9: 8223/30294; [overlay](images/12-justified-paragraphs/pdflatex-de1020c-export-p1-overlay.png) (82937 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-de1020c-export-p1-heatmap.png) (63609 B, ÷4)
  - registration error (diagnostic): global shift [-3.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.27, 32.27] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.3659, differing 0.110805, SSIM₈ 0.7414 (raw 15.3708, 0.11076, 0.7415)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5868→0.5868 / 24.5671→24.5592; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 75.37; `branch` dx -435.47 dy 55.2; `branch` dx -435.47 dy 35.03

### 12-justified-paragraphs — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1749913, 14175, 12238, 11252, 10474, 10395, 9908, 9502, 9515, 10002, 9354, 9262, 9131, 8949, 9210, 45536]`; ink px ref/ours 67213/67696 (ratio 1.0072); SSIM blocks <0.9: 7550/30294; [overlay](images/12-justified-paragraphs/pdflatex-main-export-p1-overlay.png) (83296 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-main-export-p1-heatmap.png) (60528 B, ÷4)
  - registration error (diagnostic): global shift [-4.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.66, 9.72] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 14.8302, differing 0.107799, SSIM₈ 0.7612 (raw 14.8933, 0.107948, 0.7603)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.617→0.6189 / 23.8036→23.6765; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 32.17; `branch` dx -435.47 dy 26.4; `branch` dx -435.47 dy 20.63

### 12-justified-paragraphs — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1768237, 14151, 11981, 10636, 10509, 9453, 8881, 9247, 8652, 8589, 8455, 8706, 7785, 7668, 8237, 37629]`; ink px ref/ours 67213/67185 (ratio 0.9996); SSIM blocks <0.9: 7184/30294; [overlay](images/12-justified-paragraphs/pdflatex-pipeline-export-p1-overlay.png) (82138 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-pipeline-export-p1-heatmap.png) (57593 B, ÷4)
  - registration error (diagnostic): global shift [1.0, 14.5] pt by ink-projection correlation (centroid estimate [-1.83, 25.23] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 12.9203, differing 0.098407, SSIM₈ 0.7936 (raw 13.0297, 0.098624, 0.7911)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6665→0.6845 / 20.8232→19.9565; header-band 1.0→0.9141 / 0.0→4.6025; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 58.19; `oak` dx -415.72 dy 43.74; `branch` dx -413.61 dy 58.19

### 12-justified-paragraphs — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755333, 14131, 12262, 11805, 11161, 10699, 11419, 10985, 10442, 10508, 9029, 8874, 8613, 7950, 8550, 37055]`; ink px ref/ours 51535/67475 (ratio 1.3093); SSIM blocks <0.9: 7894/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-de1020c-export-p1-overlay.png) (84541 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-de1020c-export-p1-heatmap.png) (62693 B, ÷4)
  - registration error (diagnostic): global shift [-8.5, 2.5] pt by ink-projection correlation (centroid estimate [-6.35, 7.1] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7701, differing 0.105126, SSIM₈ 0.7503 (raw 13.7992, 0.105148, 0.7503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6012→0.6012 / 22.0542→22.0076; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.0; `oak` dx 426.94 dy 3.45

### 12-justified-paragraphs — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755374, 14451, 12262, 12007, 11000, 10784, 11079, 10904, 10909, 10250, 8963, 8590, 8773, 8067, 8103, 37300]`; ink px ref/ours 51535/67696 (ratio 1.3136); SSIM blocks <0.9: 8024/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-main-export-p1-overlay.png) (83706 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-main-export-p1-heatmap.png) (62797 B, ÷4)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-0.73, -15.45] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.6913, differing 0.105143, SSIM₈ 0.7482 (raw 13.7769, 0.105422, 0.7469)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5958→0.5979 / 22.0181→21.8797; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -39.75; `oak` dx 425.86 dy -31.07; `oak` dx 425.86 dy -22.4

### 12-justified-paragraphs — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1812192, 14403, 11858, 11505, 10596, 10109, 8910, 8620, 7186, 6472, 5479, 5180, 4646, 4342, 3941, 13377]`; ink px ref/ours 51535/67185 (ratio 1.3037); SSIM blocks <0.9: 5401/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-pipeline-export-p1-overlay.png) (80147 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-pipeline-export-p1-heatmap.png) (49796 B, ÷4)
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1749319, 13924, 12542, 11373, 10258, 10254, 10377, 9353, 9938, 9889, 9331, 9231, 9181, 9122, 9123, 45601]`; ink px ref/ours 67565/67696 (ratio 1.0019); SSIM blocks <0.9: 7582/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.64, 9.96] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 14.9399, differing 0.108468, SSIM₈ 0.7591 (raw 14.9399, 0.108468, 0.7591)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.615→0.615 / 23.8781→23.8781; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 32.17; `branch` dx -435.52 dy 26.4; `branch` dx -435.52 dy 20.63

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755442, 14379, 12354, 11848, 11010, 10815, 11077, 10918, 10881, 10353, 8886, 8602, 8701, 8265, 7942, 37343]`; ink px ref/ours 51522/67696 (ratio 1.3139); SSIM blocks <0.9: 8024/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, -26.0] pt by ink-projection correlation (centroid estimate [-0.99, -15.54] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7693, differing 0.10552, SSIM₈ 0.7493 (raw 13.7765, 0.105388, 0.7469)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5958→0.5997 / 22.0176→22.0028; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -39.75; `oak` dx 425.86 dy -31.07; `oak` dx 425.86 dy -22.4

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
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [241.73, 685.5, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931855, 671, 515, 455, 384, 425, 660, 401, 348, 319, 304, 278, 299, 295, 330, 1277]`; ink px ref/ours 2479/2648 (ratio 1.0682); SSIM blocks <0.9: 332/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 3.5] pt by ink-projection correlation (centroid estimate [8.38, 2.49] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4849, differing 0.004134, SSIM₈ 0.9904 (raw 0.4988, 0.004148, 0.9897)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9835→0.9846 / 0.7972→0.775; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3982→0.3741 / 24.4873→24.7014 [232.8,85.1–384.7,123.2 pt]
- largest word displacements (pt): `√x` dx -9.86 dy 4.97; `display:` dx 0.1 dy -10.41; `Rich` dx 0 dy -10.41
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [239.44, 701.731, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931138, 641, 492, 452, 361, 473, 651, 530, 526, 327, 345, 321, 357, 372, 384, 1446]`; ink px ref/ours 2479/2728 (ratio 1.1004); SSIM blocks <0.9: 393/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [5.5, 1.5] pt by ink-projection correlation (centroid estimate [12.7, 0.09] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.561, differing 0.004401, SSIM₈ 0.988 (raw 0.5693, 0.004494, 0.9876)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9801→0.9813 / 0.9099→0.8795; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3719→0.4136 / 23.2555→21.1424 [232.8,85.1–384.7,123.2 pt]
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
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [241.73, 685.5, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931161, 774, 542, 414, 434, 436, 519, 512, 461, 443, 476, 336, 339, 312, 369, 1288]`; ink px ref/ours 2285/2648 (ratio 1.1589); SSIM blocks <0.9: 373/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 3.5] pt by ink-projection correlation (centroid estimate [-13.18, 2.89] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5303, differing 0.004307, SSIM₈ 0.989 (raw 0.552, 0.004382, 0.9884)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9815→0.9825 / 0.8822→0.8476; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3961→0.3849 / 24.1269→24.358 [232.8,84.9–384.7,123.1 pt]
- largest word displacements (pt): `√x` dx -9.86 dy 5.11; `display:` dx -1.48 dy -10.27; `Rich` dx 0 dy -10.27
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [239.44, 701.731, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931139, 736, 563, 455, 460, 437, 450, 581, 555, 405, 525, 321, 292, 314, 381, 1202]`; ink px ref/ours 2285/2728 (ratio 1.1939); SSIM blocks <0.9: 402/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [3.5, 1.5] pt by ink-projection correlation (centroid estimate [-8.87, 0.5] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4859, differing 0.004103, SSIM₈ 0.9895 (raw 0.5464, 0.004424, 0.9877)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9803→0.9834 / 0.8734→0.7665; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3782→0.4413 / 23.043→20.6144 [232.8,84.9–384.7,123.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -27.75 Δy 1.5 len 30.5 vs 31.0, thickness px 1 vs 1; Δx 4.75 Δy 1.5 len 33.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `1` dx -27.25 dy 2.3; `+` dx -27.12 dy 2.3; `∞` dx -19.71 dy -5.84
- word-sequence differences: replace ref ['1', '0'] ours ['∫']; replace ref ['√x'] ours ['0', '1', '√', '?']; replace ref ['x2', 'dx='] ours ['?', '2', ',', '??', '=']

### 13-math-display-rich — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931825, 678, 566, 390, 402, 560, 481, 438, 376, 334, 290, 257, 295, 302, 330, 1292]`; ink px ref/ours 2470/2605 (ratio 1.0547); SSIM blocks <0.9: 320/30294; [overlay](images/13-math-display-rich/pdflatex-de1020c-export-p1-overlay.png) (82885 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-de1020c-export-p1-heatmap.png) (81097 B, ÷1)
  - registration error (diagnostic): global shift [1.0, 3.5] pt by ink-projection correlation (centroid estimate [0.39, 2.75] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4808, differing 0.004039, SSIM₈ 0.9907 (raw 0.501, 0.004125, 0.9899)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9839→0.9853 / 0.8007→0.767; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4017→0.3792 / 25.1214→25.5101 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -66.25 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2; Δx 18.75 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `display:` dx 0.1 dy -10.42; `Rich` dx 0 dy -10.42; `√x` dx -7.41 dy 4.96
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (3): error: \left is not supported in math mode; error: \right is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [241.73, 685.5, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931869, 761, 582, 393, 427, 567, 527, 377, 382, 346, 278, 261, 303, 248, 356, 1139]`; ink px ref/ours 2470/2648 (ratio 1.0721); SSIM blocks <0.9: 329/30294; [overlay](images/13-math-display-rich/pdflatex-main-export-p1-overlay.png) (83257 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-main-export-p1-heatmap.png) (81567 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [8.96, 2.59] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4734, differing 0.004074, SSIM₈ 0.9904 (raw 0.4817, 0.004129, 0.99)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.984→0.9847 / 0.7699→0.7567; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.398→0.3669 / 24.4984→25.1129 [232.8,85.1–384.7,123.2 pt]
- largest word displacements (pt): `√x` dx -9.86 dy 4.96; `display:` dx 0.1 dy -10.42; `Rich` dx 0 dy -10.42
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [239.44, 701.731, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931163, 631, 481, 401, 416, 558, 507, 522, 551, 322, 364, 330, 336, 340, 423, 1471]`; ink px ref/ours 2470/2728 (ratio 1.1045); SSIM blocks <0.9: 389/30294; [overlay](images/13-math-display-rich/pdflatex-pipeline-export-p1-overlay.png) (83036 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-pipeline-export-p1-heatmap.png) (82433 B, ÷1)
  - registration error (diagnostic): global shift [4.0, 1.5] pt by ink-projection correlation (centroid estimate [13.28, 0.2] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5371, differing 0.004336, SSIM₈ 0.9885 (raw 0.5728, 0.004482, 0.9876)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9801→0.9817 / 0.9155→0.8475; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.372→0.4381 / 23.2576→20.2166 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -82.25 Δy 8.0 len 30.5 vs 12.0, thickness px 1 vs 1; Δx 40.25 Δy -5.25 len 33.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `1` dx -27.25 dy 2.15; `+` dx -27.12 dy 2.15; `∞` dx -19.71 dy -5.99
- word-sequence differences: replace ref ['1', '0'] ours ['∫']; replace ref ['√x'] ours ['0', '1', '√', '?']; replace ref ['x2', 'dx='] ours ['?', '2', ',', '??', '=']

### 13-math-display-rich — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931353, 583, 548, 467, 406, 416, 441, 540, 508, 387, 355, 345, 457, 339, 349, 1322]`; ink px ref/ours 2283/2605 (ratio 1.141); SSIM blocks <0.9: 363/30294; [overlay](images/13-math-display-rich/pdflatex-lm-de1020c-export-p1-overlay.png) (83631 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-de1020c-export-p1-heatmap.png) (82579 B, ÷1)
  - registration error (diagnostic): global shift [-2.0, 4.0] pt by ink-projection correlation (centroid estimate [-23.81, 3.31] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5406, differing 0.004269, SSIM₈ 0.9893 (raw 0.553, 0.004302, 0.9888)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.982→0.983 / 0.8838→0.8638; header-band 1.0→0.9992 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4037→0.4126 / 25.0041→24.5413 [232.8,84.8–384.7,123 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -11.5 Δy 4.0 len 21.0 vs 32.0, thickness px 1 vs 1; Δx -16.75 Δy 4.0 len 21.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `display:` dx -1.48 dy -10.16; `Rich` dx 0 dy -10.16; `√x` dx -7.41 dy 5.22
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (3): error: \left is not supported in math mode; error: \right is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [241.73, 685.5, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931230, 674, 565, 466, 406, 434, 479, 488, 526, 393, 354, 342, 483, 305, 361, 1310]`; ink px ref/ours 2283/2648 (ratio 1.1599); SSIM blocks <0.9: 373/30294; [overlay](images/13-math-display-rich/pdflatex-lm-main-export-p1-overlay.png) (83925 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-main-export-p1-heatmap.png) (83080 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 4.0] pt by ink-projection correlation (centroid estimate [-15.24, 3.15] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5397, differing 0.004318, SSIM₈ 0.9888 (raw 0.5543, 0.004359, 0.9885)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9816→0.9823 / 0.8859→0.8623; header-band 1.0→0.9991 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3993→0.3721 / 24.416→24.8563 [232.8,84.8–384.7,123 pt]
- largest word displacements (pt): `√x` dx -9.86 dy 5.22; `display:` dx -1.48 dy -10.16; `Rich` dx 0 dy -10.16
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [239.44, 701.731, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931247, 607, 562, 482, 435, 441, 427, 581, 617, 347, 386, 335, 450, 316, 378, 1205]`; ink px ref/ours 2283/2728 (ratio 1.1949); SSIM blocks <0.9: 402/30294; [overlay](images/13-math-display-rich/pdflatex-lm-pipeline-export-p1-overlay.png) (83762 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-pipeline-export-p1-heatmap.png) (82930 B, ÷1)
  - registration error (diagnostic): global shift [3.5, 1.5] pt by ink-projection correlation (centroid estimate [-10.92, 0.76] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4959, differing 0.004121, SSIM₈ 0.9893 (raw 0.5487, 0.004389, 0.9877)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9803→0.983 / 0.877→0.7824; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3793→0.4385 / 23.3686→21.0828 [232.8,84.8–384.7,123 pt]
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
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [241.73, 685.5, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1932016, 688, 534, 464, 374, 411, 631, 411, 343, 331, 310, 285, 308, 273, 297, 1140]`; ink px ref/ours 2475/2648 (ratio 1.0699); SSIM blocks <0.9: 331/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [8.14, 2.45] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4525, differing 0.004, SSIM₈ 0.9906 (raw 0.4767, 0.004081, 0.9899)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9839→0.985 / 0.7618→0.7232; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6001→0.5898 / 16.8071→17.2259 [227.3,75.6–384.7,129 pt]
- largest word displacements (pt): `√x` dx -9.86 dy 4.37; `∫1` dx 2.95 dy -8.2; `after` dx -0.09 dy 3.86
- word-sequence differences: replace ref ['1', '+', 'x2', 'dx=', '∞', '∑', 'k=0', '((−1)k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [239.44, 701.731, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931082, 653, 502, 437, 361, 464, 641, 550, 542, 321, 376, 336, 368, 368, 371, 1444]`; ink px ref/ours 2475/2728 (ratio 1.1022); SSIM blocks <0.9: 393/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.5, 1.5] pt by ink-projection correlation (centroid estimate [12.46, 0.05] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5445, differing 0.004366, SSIM₈ 0.9882 (raw 0.5734, 0.004502, 0.9875)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.98→0.9814 / 0.9164→0.8674; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5686→0.6017 / 16.3424→14.924 [227.3,75.6–384.7,129 pt]
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
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [241.73, 685.5, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931163, 771, 543, 411, 437, 433, 521, 510, 463, 440, 480, 338, 338, 315, 365, 1288]`; ink px ref/ours 2286/2648 (ratio 1.1584); SSIM blocks <0.9: 373/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 3.5] pt by ink-projection correlation (centroid estimate [-13.18, 2.84] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5303, differing 0.004307, SSIM₈ 0.989 (raw 0.5522, 0.004382, 0.9884)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9815→0.9825 / 0.8825→0.8475; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5972→0.5939 / 16.7743→16.9329 [227.3,75.4–384.7,128.9 pt]
- largest word displacements (pt): `√x` dx -9.86 dy 4.51; `display.` dx -8.48 dy 4.15; `∫1` dx 2.95 dy -8.06
- word-sequence differences: replace ref ['1', '+', 'x2', 'dx=', '∞', '∑', 'k=0', '((−1)k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [239.44, 701.731, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931135, 739, 562, 457, 462, 436, 447, 582, 554, 400, 531, 323, 291, 316, 378, 1203]`; ink px ref/ours 2286/2728 (ratio 1.1934); SSIM blocks <0.9: 402/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [3.5, 1.5] pt by ink-projection correlation (centroid estimate [-8.87, 0.45] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.486, differing 0.004104, SSIM₈ 0.9895 (raw 0.5466, 0.004424, 0.9877)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9803→0.9834 / 0.8737→0.7666; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.559→0.5957 / 16.3873→15.0108 [227.3,75.4–384.7,128.9 pt]
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
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [218.54, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921732, 1323, 1191, 1028, 950, 895, 929, 981, 866, 1003, 805, 1104, 843, 801, 832, 3533]`; ink px ref/ours 4839/5652 (ratio 1.168); SSIM blocks <0.9: 950/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-34.5, 13.5] pt by ink-projection correlation (centroid estimate [-13.23, 6.91] pt); confidence strong (shift explains 28% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.946, differing 0.008388, SSIM₈ 0.9784 (raw 1.3111, 0.009906, 0.9706)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.953→0.9736 / 2.0955→1.1258; header-band 1.0→0.9475 / 0.0→2.5818; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3425→0.3522 / 26.5831→26.1603 [96.4,69.7–279.9,108.1 pt]
- largest word displacements (pt): `,` dx -447.23 dy 29.13; `z2` dx -43.92 dy 3.61; `u2` dx -35.85 dy 11.6
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2', ',', 'πr2', ','] ours ['a2+b2', '=c2']; delete ref ['1', 'x'] ours []

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
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [218.54, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922131, 1374, 1176, 1039, 991, 888, 913, 1031, 1009, 984, 812, 1041, 791, 719, 824, 3093]`; ink px ref/ours 4245/5652 (ratio 1.3314); SSIM blocks <0.9: 954/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-8.47, 6.54] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1626, differing 0.009214, SSIM₈ 0.9734 (raw 1.2444, 0.009642, 0.9703)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9526→0.9654 / 1.9889→1.4699; header-band 1.0→0.9474 / 0.0→2.5818; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3441→0.2989 / 23.9997→26.3202 [176,69.7–288.8,108.1 pt]
- largest word displacements (pt): `must` dx -73.88 dy 13.68; `that` dx -69.88 dy 13.68; `line` dx -69.15 dy 13.68
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'z2', ',', 'p1,', 'σ2', ','] ours ['a2+b2', '=c2']; delete ref ['1', 'x'] ours []

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921723, 1256, 1319, 999, 1015, 858, 878, 898, 911, 966, 857, 1132, 834, 737, 861, 3572]`; ink px ref/ours 4824/5720 (ratio 1.1857); SSIM blocks <0.9: 948/30294; [overlay](images/14-math-inline-dense/pdflatex-de1020c-export-p1-overlay.png) (45651 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-de1020c-export-p1-heatmap.png) (44426 B, ÷2)
  - registration error (diagnostic): global shift [-34.0, 13.5] pt by ink-projection correlation (centroid estimate [-13.94, 7.12] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9608, differing 0.008388, SSIM₈ 0.9783 (raw 1.3129, 0.009906, 0.9708)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9533→0.9734 / 2.0983→1.1478; header-band 1.0→0.9469 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3428→0.3554 / 26.6779→25.5089 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -407.82 dy 29.13; `,` dx -87.71 dy 29.13; `,` dx 84.05 dy -10.04
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours ['a2+b2', '=c2']; insert ref [] ours ['α', 'βγ,', ',', '√2', ',', 'xj', 'i', ',']

### 14-math-inline-dense — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [218.54, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921877, 1256, 1248, 993, 983, 883, 867, 932, 870, 916, 839, 1099, 839, 807, 890, 3517]`; ink px ref/ours 4824/5652 (ratio 1.1716); SSIM blocks <0.9: 944/30294; [overlay](images/14-math-inline-dense/pdflatex-main-export-p1-overlay.png) (45111 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-main-export-p1-heatmap.png) (44017 B, ÷2)
  - registration error (diagnostic): global shift [-34.0, 13.5] pt by ink-projection correlation (centroid estimate [-12.75, 6.87] pt); confidence strong (shift explains 29% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9332, differing 0.008315, SSIM₈ 0.9785 (raw 1.3054, 0.009857, 0.9708)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9534→0.974 / 2.0863→1.1058; header-band 1.0→0.9454 / 0.0→2.5818; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3465→0.3563 / 26.3679→25.2453 [96.4,69.7–279.8,108.1 pt]
- largest word displacements (pt): `,` dx -447.23 dy 29.13; `z2` dx -43.92 dy 3.61; `u2` dx -35.85 dy 11.6
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2', ',', 'πr2', ','] ours ['a2+b2', '=c2']; replace ref ['1', 'x,', '√2,'] ours [',', '√2', ',']

### 14-math-inline-dense — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 14.08pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [235.512, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1925067, 1312, 1093, 881, 953, 804, 751, 786, 703, 752, 658, 907, 660, 657, 601, 2231]`; ink px ref/ours 4824/4873 (ratio 1.0102); SSIM blocks <0.9: 674/30294; [overlay](images/14-math-inline-dense/pdflatex-pipeline-export-p1-overlay.png) (44378 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-pipeline-export-p1-heatmap.png) (40046 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [4.0, 1.5] pt REJECTED: applying it gives mean|Δ| 1.0535, not lower; centroid estimate [2.97, 0.25] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.983, differing 0.008186, SSIM₈ 0.9799 (raw 0.983, 0.008186, 0.9799)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9684→0.9684 / 1.5546→1.5546; header-band 0.9986→0.9986 / 0.0547→0.0547; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4346→0.4346 / 22.5256→22.5256 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 22.25 Δy 1.5 len 28.0 vs 27.5, thickness px 1 vs 1; Δx -16.75 Δy 7.0 len 14.0 vs 27.5, thickness px 1 vs 1
- largest word displacements (pt): `,` dx 83.73 dy 0.89; `,` dx -55.72 dy 16.32; `,` dx -51.73 dy 3.99
- word-sequence differences: replace ref ['a2'] ours ['?2']; replace ref ['b2', 'p1,', 'σ2'] ours ['?2', '=', '?2', '?2']; replace ref ['πr2'] ours ['?1,', '?', '2']

### 14-math-inline-dense — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922114, 1292, 1255, 994, 1030, 864, 932, 987, 1070, 1032, 804, 1077, 790, 689, 773, 3113]`; ink px ref/ours 4254/5720 (ratio 1.3446); SSIM blocks <0.9: 957/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-de1020c-export-p1-overlay.png) (45802 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-de1020c-export-p1-heatmap.png) (44318 B, ÷2)
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-7.87, 6.82] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1673, differing 0.009234, SSIM₈ 0.9735 (raw 1.2469, 0.009657, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9655 / 1.9928→1.475; header-band 1.0→0.9479 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3525→0.2973 / 24.7689→27.2474 [183.4,69.8–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -51.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `z2` dx 408.28 dy -8.86; `must` dx -73.95 dy 13.68; `that` dx -69.96 dy 13.68
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2'] ours ['a2+b2', '=c2', ',', 'α', 'βγ,', ',', '√2', ',']; insert ref [] ours ['πr2', 'a+b']

### 14-math-inline-dense — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [218.54, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922163, 1332, 1203, 1012, 991, 900, 908, 1027, 1040, 978, 799, 1043, 776, 743, 802, 3099]`; ink px ref/ours 4254/5652 (ratio 1.3286); SSIM blocks <0.9: 953/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-main-export-p1-overlay.png) (45384 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-main-export-p1-heatmap.png) (43999 B, ÷2)
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-6.68, 6.57] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1624, differing 0.009208, SSIM₈ 0.9734 (raw 1.244, 0.00964, 0.9703)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9526→0.9654 / 1.9883→1.4696; header-band 1.0→0.9474 / 0.0→2.5818; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3519→0.2989 / 24.3244→26.8863 [183.4,69.8–288.8,108.1 pt]
- largest word displacements (pt): `must` dx -73.89 dy 13.68; `that` dx -69.9 dy 13.68; `line` dx -69.17 dy 13.68
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'z2', ',', 'p1,', 'σ2', ','] ours ['a2+b2', '=c2']; replace ref ['1', 'x,', '√2,'] ours [',', '√2', ',']

### 14-math-inline-dense — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 14.08pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [235.512, 710.795, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1925027, 1308, 1099, 978, 997, 880, 822, 865, 920, 814, 655, 815, 562, 583, 547, 1944]`; ink px ref/ours 4254/4873 (ratio 1.1455); SSIM blocks <0.9: 676/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-pipeline-export-p1-overlay.png) (44745 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-pipeline-export-p1-heatmap.png) (39605 B, ÷2)
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
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [218.54, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921752, 1305, 1204, 1007, 971, 886, 922, 969, 880, 997, 810, 1087, 848, 802, 829, 3547]`; ink px ref/ours 4852/5652 (ratio 1.1649); SSIM blocks <0.9: 950/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-34.5, 13.5] pt by ink-projection correlation (centroid estimate [-13.46, 6.89] pt); confidence strong (shift explains 28% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9422, differing 0.00837, SSIM₈ 0.9785 (raw 1.3112, 0.009898, 0.9706)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.953→0.9736 / 2.0955→1.1198; header-band 1.0→0.9475 / 0.0→2.5818; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3424→0.3522 / 26.5817→26.1576 [96.4,69.7–279.8,108.1 pt]
- largest word displacements (pt): `,` dx -447.23 dy 29.13; `z2` dx -43.92 dy -13.61; `u2` dx -35.85 dy 11.6
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2', ',', 'πr2', ','] ours ['a2+b2', '=c2']; delete ref ['1', 'x'] ours []

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
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [218.54, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922125, 1378, 1181, 1034, 994, 887, 907, 1037, 1011, 982, 816, 1038, 791, 721, 822, 3092]`; ink px ref/ours 4244/5652 (ratio 1.3318); SSIM blocks <0.9: 954/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-8.39, 6.54] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1627, differing 0.009214, SSIM₈ 0.9734 (raw 1.2444, 0.009642, 0.9703)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9526→0.9654 / 1.9889→1.47; header-band 1.0→0.9474 / 0.0→2.5818; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3521→0.3007 / 24.3679→26.9031 [183.4,69.7–288.8,108.1 pt]
- largest word displacements (pt): `must` dx -73.88 dy 13.68; `that` dx -69.88 dy 13.68; `line` dx -69.15 dy 13.68
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'z2', ',', 'p1,', 'σ2', ','] ours ['a2+b2', '=c2']; delete ref ['1', 'x'] ours []

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1598328, 24968, 21326, 20340, 17583, 18131, 18551, 16669, 16935, 17779, 16507, 16139, 17183, 16715, 16366, 85296]`; ink px ref/ours 112785/117085 (ratio 1.0381); SSIM blocks <0.9: 14157/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 20.0] pt by ink-projection correlation (centroid estimate [-14.39, 65.0] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.5712, differing 0.186095, SSIM₈ 0.5837 (raw 27.1751, 0.194439, 0.5431)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2822→0.3374 / 42.7648→40.7578; header-band 1.0→0.986 / 0.0→0.7294; footer-band 0.9191→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1594734, 25563, 21828, 20932, 18080, 18320, 19087, 17099, 17237, 18306, 16718, 16615, 17277, 16575, 16728, 83717]`; ink px ref/ours 112804/123031 (ratio 1.0907); SSIM blocks <0.9: 14143/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 19.0] pt by ink-projection correlation (centroid estimate [-13.02, 40.73] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.4852, differing 0.19319, SSIM₈ 0.5603 (raw 27.2525, 0.196892, 0.5494)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2805→0.3168 / 43.5509→41.5669; header-band 1.0→0.8707 / 0.0→5.2138; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1630074, 23065, 19742, 18581, 16129, 16443, 16813, 15348, 15748, 15998, 15092, 15073, 15415, 15130, 14823, 75342]`; ink px ref/ours 112799/99249 (ratio 0.8799); SSIM blocks <0.9: 12438/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-13.47, -17.82] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.2629, differing 0.170035, SSIM₈ 0.6246 (raw 24.4692, 0.176725, 0.6033)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3666→0.4007 / 39.1024→37.1744; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 95.02; `branch` dx -435.47 dy 74.98; `the` dx -430.88 dy 94.97
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1649434, 23254, 19621, 18509, 16773, 16892, 16158, 16100, 15606, 15557, 14859, 14352, 13635, 13602, 14032, 60432]`; ink px ref/ours 112785/111937 (ratio 0.9925); SSIM blocks <0.9: 11597/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 14.0] pt by ink-projection correlation (centroid estimate [-5.3, 28.54] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.0743, differing 0.161917, SSIM₈ 0.6752 (raw 21.9855, 0.165833, 0.656)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.451→0.4839 / 35.1313→33.5657; header-band 1.0→0.9856 / 0.0→0.7483; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1649308, 23273, 19642, 18521, 16774, 16880, 16163, 16079, 15617, 15563, 14866, 14356, 13630, 13606, 14055, 60483]`; ink px ref/ours 112804/112014 (ratio 0.993); SSIM blocks <0.9: 11600/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 14.0] pt by ink-projection correlation (centroid estimate [-5.35, 28.38] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.0851, differing 0.16199, SSIM₈ 0.675 (raw 21.9967, 0.165904, 0.6558)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4508→0.4839 / 35.1491→33.5713; header-band 1.0→0.9839 / 0.0→0.8283; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1649289, 23270, 19644, 18529, 16773, 16887, 16173, 16077, 15614, 15569, 14863, 14347, 13630, 13604, 14035, 60512]`; ink px ref/ours 112799/111986 (ratio 0.9928); SSIM blocks <0.9: 11603/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 14.0] pt by ink-projection correlation (centroid estimate [-5.33, 28.49] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.081, differing 0.161985, SSIM₈ 0.675 (raw 21.9979, 0.16592, 0.6557)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4507→0.4837 / 35.1511→33.5693; header-band 1.0→0.9848 / 0.0→0.7969; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 57.73; `oak` dx -415.78 dy 57.73; `oak` dx -415.78 dy 57.73
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610314, 25400, 21390, 20774, 19031, 18587, 19988, 19464, 18866, 17829, 15879, 15814, 15866, 15011, 15073, 69530]`; ink px ref/ours 85641/117085 (ratio 1.3672); SSIM blocks <0.9: 14319/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.36, 37.5] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.2382, differing 0.178146, SSIM₈ 0.5841 (raw 25.0181, 0.18856, 0.5353)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2695→0.3379 / 39.3223→37.0305; header-band 1.0→0.9865 / 0.0→0.7294; footer-band 0.9191→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1605944, 25755, 21744, 21209, 19399, 18804, 20690, 19697, 19008, 18177, 15935, 16119, 15893, 15144, 15262, 70036]`; ink px ref/ours 85678/123031 (ratio 1.436); SSIM blocks <0.9: 14651/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -24.0] pt by ink-projection correlation (centroid estimate [-6.92, 13.33] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.1631, differing 0.185078, SSIM₈ 0.564 (raw 25.298, 0.191124, 0.5322)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2527→0.3176 / 40.432→37.8902; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.905 / 0.0→4.9832
- page 3: |Δ| histogram (16 bins, pixel counts) `[1641012, 23200, 19701, 18922, 17609, 16950, 18284, 17874, 17585, 16173, 14448, 14432, 14187, 13531, 13708, 61200]`; ink px ref/ours 85675/99249 (ratio 1.1584); SSIM blocks <0.9: 13044/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -24.0] pt by ink-projection correlation (centroid estimate [-7.35, -45.28] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.4784, differing 0.16548, SSIM₈ 0.6037 (raw 22.5169, 0.171065, 0.5806)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3301→0.367 / 35.9871→34.3273; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -93.06; `oak` dx 425.86 dy -87.47; `oak` dx 425.86 dy -81.88
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1725694, 24756, 19814, 19047, 17552, 17341, 15397, 14698, 11863, 10892, 9240, 8525, 7610, 7192, 6718, 22477]`; ink px ref/ours 85641/111937 (ratio 1.307); SSIM blocks <0.9: 9098/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.74, 1.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1206, differing 0.128742, SSIM₈ 0.8178 (raw 13.1206, 0.128742, 0.8178)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7091→0.7091 / 20.9686→20.9686; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1725509, 24770, 19838, 19065, 17547, 17345, 15406, 14666, 11876, 10889, 9241, 8538, 7625, 7210, 6734, 22557]`; ink px ref/ours 85678/112014 (ratio 1.3074); SSIM blocks <0.9: 9102/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.75, 0.98] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1383, differing 0.128845, SSIM₈ 0.8175 (raw 13.1383, 0.128845, 0.8175)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7087→0.7087 / 20.9969→20.9969; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1725479, 24778, 19840, 19069, 17546, 17334, 15411, 14664, 11884, 10903, 9238, 8532, 7625, 7203, 6721, 22589]`; ink px ref/ours 85675/111986 (ratio 1.3071); SSIM blocks <0.9: 9105/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.78, 1.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1408, differing 0.128868, SSIM₈ 0.8175 (raw 13.1408, 0.128868, 0.8175)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7086→0.7086 / 21.001→21.001; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Part` dx 29.06 dy -0.67; `1` dx 29.06 dy -0.67; `Part` dx 29.06 dy -0.67
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1595899, 24122, 20968, 20023, 17718, 17966, 18322, 17267, 17094, 18406, 16944, 15990, 16755, 16056, 16950, 88336]`; ink px ref/ours 112156/116938 (ratio 1.0426); SSIM blocks <0.9: 14346/30294; [overlay](images/15-three-page-sections/pdflatex-de1020c-export-p1-overlay.png) (48175 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-de1020c-export-p1-heatmap.png) (35435 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 36.5] pt by ink-projection correlation (centroid estimate [-13.64, 42.99] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.376, differing 0.183932, SSIM₈ 0.5937 (raw 27.5937, 0.195208, 0.5337)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2665→0.3542 / 43.4405→40.4605; header-band 1.0→0.9879 / 0.0→0.6238; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1592865, 24793, 21494, 20314, 18311, 18548, 19317, 17570, 17258, 18450, 17153, 16254, 16690, 16013, 17183, 86603]`; ink px ref/ours 112190/122724 (ratio 1.0939); SSIM blocks <0.9: 14238/30294; [overlay](images/15-three-page-sections/pdflatex-de1020c-export-p2-overlay.png) (49089 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-de1020c-export-p2-heatmap.png) (35561 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.72, 17.61] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.0991, differing 0.18994, SSIM₈ 0.5742 (raw 27.5896, 0.197186, 0.5428)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2693→0.3363 / 44.0962→41.0127; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8846 / 0.0→4.8261
- page 3: |Δ| histogram (16 bins, pixel counts) `[1626455, 22468, 19561, 18552, 16338, 16950, 16970, 15864, 15658, 16281, 15491, 14672, 14985, 14631, 15599, 78341]`; ink px ref/ours 112177/99417 (ratio 0.8863); SSIM blocks <0.9: 12897/30294; [overlay](images/15-three-page-sections/pdflatex-de1020c-export-p3-overlay.png) (45381 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-de1020c-export-p3-heatmap.png) (33419 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.66, -40.6] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.7997, differing 0.172056, SSIM₈ 0.6079 (raw 24.915, 0.178224, 0.5858)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.338→0.3733 / 39.8213→38.0388; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy -63.97; `branch` dx -435.47 dy -58.38; `branch` dx -435.47 dy -52.79

### 15-three-page-sections — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1595624, 24256, 21315, 20032, 17341, 18209, 18594, 16970, 17235, 18149, 16721, 15961, 17001, 16536, 16579, 88293]`; ink px ref/ours 112156/117085 (ratio 1.0439); SSIM blocks <0.9: 14325/30294; [overlay](images/15-three-page-sections/pdflatex-main-export-p1-overlay.png) (47801 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-main-export-p1-heatmap.png) (35179 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 36.5] pt by ink-projection correlation (centroid estimate [-13.9, 42.18] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.3272, differing 0.184066, SSIM₈ 0.596 (raw 27.5995, 0.19563, 0.5345)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2679→0.3565 / 43.4492→40.3738; header-band 1.0→0.986 / 0.0→0.7294; footer-band 0.9191→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1592027, 24727, 21909, 20507, 17993, 18645, 19436, 17426, 17343, 18297, 16862, 16253, 17006, 16596, 16958, 86831]`; ink px ref/ours 112190/123031 (ratio 1.0966); SSIM blocks <0.9: 14280/30294; [overlay](images/15-three-page-sections/pdflatex-main-export-p2-overlay.png) (49025 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-main-export-p2-heatmap.png) (35583 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.47, 17.97] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.2045, differing 0.190709, SSIM₈ 0.5744 (raw 27.6711, 0.198001, 0.5413)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2671→0.3371 / 44.2259→41.1544; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8816 / 0.0→5.0073
- page 3: |Δ| histogram (16 bins, pixel counts) `[1625955, 22340, 19955, 18569, 16150, 16962, 17030, 15580, 16014, 16211, 15217, 14637, 15137, 15070, 15296, 78693]`; ink px ref/ours 112177/99249 (ratio 0.8848); SSIM blocks <0.9: 12858/30294; [overlay](images/15-three-page-sections/pdflatex-main-export-p3-overlay.png) (44902 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-main-export-p3-heatmap.png) (33193 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.92, -40.65] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.6494, differing 0.171634, SSIM₈ 0.6105 (raw 24.9709, 0.178566, 0.586)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3385→0.3777 / 39.9103→37.7982; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy -63.97; `branch` dx -435.47 dy -58.38; `branch` dx -435.47 dy -52.79
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1652030, 22602, 19942, 18498, 16483, 16676, 16225, 16430, 15807, 15420, 14923, 13803, 13105, 13385, 13618, 59869]`; ink px ref/ours 112156/111937 (ratio 0.998); SSIM blocks <0.9: 11249/30294; [overlay](images/15-three-page-sections/pdflatex-pipeline-export-p1-overlay.png) (48281 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-pipeline-export-p1-heatmap.png) (79367 B, ÷4)
  - registration error (diagnostic): global shift [1.0, -0.5] pt by ink-projection correlation (centroid estimate [-4.8, 5.72] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.5505, differing 0.159374, SSIM₈ 0.6873 (raw 21.7408, 0.164519, 0.6643)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.464→0.5033 / 34.7457→32.8088; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1651928, 22622, 19962, 18507, 16474, 16650, 16229, 16404, 15817, 15424, 14936, 13830, 13103, 13396, 13631, 59903]`; ink px ref/ours 112190/112014 (ratio 0.9984); SSIM blocks <0.9: 11253/30294; [overlay](images/15-three-page-sections/pdflatex-pipeline-export-p2-overlay.png) (48342 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-pipeline-export-p2-heatmap.png) (79443 B, ÷4)
  - registration error (diagnostic): global shift [1.0, -0.5] pt by ink-projection correlation (centroid estimate [-4.8, 5.62] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.5618, differing 0.159453, SSIM₈ 0.6871 (raw 21.7507, 0.164587, 0.6642)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4637→0.5031 / 34.7617→32.8265; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1651919, 22624, 19965, 18514, 16476, 16652, 16231, 16405, 15819, 15437, 14929, 13813, 13101, 13390, 13616, 59925]`; ink px ref/ours 112177/111986 (ratio 0.9983); SSIM blocks <0.9: 11255/30294; [overlay](images/15-three-page-sections/pdflatex-pipeline-export-p3-overlay.png) (48363 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-pipeline-export-p3-heatmap.png) (79443 B, ÷4)
  - registration error (diagnostic): global shift [1.0, -0.5] pt by ink-projection correlation (centroid estimate [-4.78, 5.67] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.5575, differing 0.159436, SSIM₈ 0.6873 (raw 21.7504, 0.164599, 0.6641)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4637→0.5033 / 34.7612→32.8199; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 14.32; `oak` dx -415.72 dy 14.32; `oak` dx -415.72 dy 14.32
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610609, 24967, 21053, 20726, 19661, 18340, 19742, 19412, 18598, 18249, 16287, 15994, 15576, 14410, 15675, 69517]`; ink px ref/ours 85692/116938 (ratio 1.3646); SSIM blocks <0.9: 14337/30294; [overlay](images/15-three-page-sections/pdflatex-lm-de1020c-export-p1-overlay.png) (48439 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-de1020c-export-p1-heatmap.png) (35277 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.11, 38.21] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.3365, differing 0.178246, SSIM₈ 0.5819 (raw 25.0354, 0.18832, 0.5339)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2672→0.3354 / 39.3505→37.2; header-band 1.0→0.9885 / 0.0→0.6238; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606782, 25413, 21423, 20910, 20004, 18862, 20556, 19676, 18675, 18670, 16345, 16142, 15704, 14246, 15727, 69681]`; ink px ref/ours 85729/122724 (ratio 1.4315); SSIM blocks <0.9: 14619/30294; [overlay](images/15-three-page-sections/pdflatex-lm-de1020c-export-p2-overlay.png) (49616 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-de1020c-export-p2-heatmap.png) (35882 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.23, 12.88] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.2389, differing 0.184974, SSIM₈ 0.5622 (raw 25.2323, 0.19048, 0.5336)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2548→0.3156 / 40.3276→38.0352; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9087 / 0.0→4.802
- page 3: |Δ| histogram (16 bins, pixel counts) `[1641126, 23072, 19471, 19055, 17908, 16873, 18097, 18161, 17038, 16322, 14757, 14561, 14158, 12872, 14209, 61136]`; ink px ref/ours 85728/99417 (ratio 1.1597); SSIM blocks <0.9: 13069/30294; [overlay](images/15-three-page-sections/pdflatex-lm-de1020c-export-p3-overlay.png) (45691 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-de1020c-export-p3-heatmap.png) (33521 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.13, -45.32] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.5333, differing 0.165392, SSIM₈ 0.6026 (raw 22.5169, 0.171027, 0.5804)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3295→0.3666 / 35.9875→34.4099; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -92.02; `oak` dx 426.94 dy -86.43; `oak` dx 426.94 dy -80.84

### 15-three-page-sections — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610416, 25113, 21419, 20803, 19258, 18585, 19969, 19377, 18674, 17907, 15994, 15840, 15987, 14926, 15214, 69334]`; ink px ref/ours 85692/117085 (ratio 1.3663); SSIM blocks <0.9: 14317/30294; [overlay](images/15-three-page-sections/pdflatex-lm-main-export-p1-overlay.png) (48112 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-main-export-p1-heatmap.png) (35003 B, ÷8)
  - registration error (diagnostic): global shift [-0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.37, 37.41] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.2375, differing 0.178188, SSIM₈ 0.5842 (raw 25.0184, 0.188605, 0.5352)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2693→0.338 / 39.3229→37.0293; header-band 1.0→0.9865 / 0.0→0.7294; footer-band 0.9191→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606072, 25375, 21965, 21062, 19569, 18901, 20739, 19515, 18885, 18204, 16020, 16124, 16068, 15003, 15475, 69839]`; ink px ref/ours 85729/123031 (ratio 1.4351); SSIM blocks <0.9: 14647/30294; [overlay](images/15-three-page-sections/pdflatex-lm-main-export-p2-overlay.png) (49404 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-main-export-p2-heatmap.png) (35847 B, ÷8)
  - registration error (diagnostic): global shift [-0.5, -24.0] pt by ink-projection correlation (centroid estimate [-6.98, 13.23] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.1593, differing 0.185092, SSIM₈ 0.5642 (raw 25.3001, 0.191187, 0.5323)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2529→0.3178 / 40.4354→37.8841; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.905 / 0.0→4.9832
- page 3: |Δ| histogram (16 bins, pixel counts) `[1641132, 22873, 19894, 18879, 17656, 17043, 18330, 17670, 17451, 16197, 14625, 14425, 14301, 13434, 13942, 60964]`; ink px ref/ours 85728/99249 (ratio 1.1577); SSIM blocks <0.9: 13044/30294; [overlay](images/15-three-page-sections/pdflatex-lm-main-export-p3-overlay.png) (45317 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-main-export-p3-heatmap.png) (33260 B, ÷8)
  - registration error (diagnostic): global shift [-0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.39, -45.37] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.4675, differing 0.165418, SSIM₈ 0.6033 (raw 22.5176, 0.1711, 0.5805)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3299→0.3666 / 35.9882→34.3068; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -92.02; `oak` dx 425.86 dy -86.43; `oak` dx 425.86 dy -80.84
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1725546, 24556, 20199, 18970, 17718, 17379, 15357, 14691, 11677, 10774, 9381, 8437, 7600, 7210, 6896, 22425]`; ink px ref/ours 85692/111937 (ratio 1.3063); SSIM blocks <0.9: 9102/30294; [overlay](images/15-three-page-sections/pdflatex-lm-pipeline-export-p1-overlay.png) (47679 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-pipeline-export-p1-heatmap.png) (71051 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.73, 0.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1391, differing 0.128908, SSIM₈ 0.8176 (raw 13.1391, 0.128908, 0.8176)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7089→0.7089 / 20.9982→20.9982; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1725348, 24585, 20220, 18991, 17712, 17379, 15369, 14658, 11692, 10771, 9380, 8454, 7611, 7228, 6896, 22522]`; ink px ref/ours 85729/112014 (ratio 1.3066); SSIM blocks <0.9: 9106/30294; [overlay](images/15-three-page-sections/pdflatex-lm-pipeline-export-p2-overlay.png) (47743 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-pipeline-export-p2-heatmap.png) (71138 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.69, 0.88] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.157, differing 0.12901, SSIM₈ 0.8174 (raw 13.157, 0.12901, 0.8174)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7085→0.7085 / 21.0267→21.0267; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1725318, 24591, 20219, 18997, 17714, 17374, 15368, 14656, 11705, 10778, 9381, 8442, 7615, 7222, 6883, 22553]`; ink px ref/ours 85728/111986 (ratio 1.3063); SSIM blocks <0.9: 9109/30294; [overlay](images/15-three-page-sections/pdflatex-lm-pipeline-export-p3-overlay.png) (47767 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-pipeline-export-p3-heatmap.png) (71138 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.74, 0.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1595, differing 0.129032, SSIM₈ 0.8173 (raw 13.1595, 0.129032, 0.8173)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7084→0.7084 / 21.0308→21.0308; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `1` dx 29.09 dy 0.96; `2` dx 29.09 dy 0.96; `3` dx 29.09 dy 0.96
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1598072, 24431, 21106, 20760, 17537, 18579, 18586, 16764, 16839, 17980, 16658, 16580, 16974, 16320, 16215, 85415]`; ink px ref/ours 112652/117085 (ratio 1.0394); SSIM blocks <0.9: 14151/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 20.0] pt by ink-projection correlation (centroid estimate [-14.54, 64.86] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.6189, differing 0.186217, SSIM₈ 0.5832 (raw 27.1981, 0.194509, 0.5428)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2818→0.3365 / 42.8015→40.8337; header-band 1.0→0.986 / 0.0→0.7294; footer-band 0.9191→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1594048, 25081, 21676, 21164, 18089, 18816, 19283, 17259, 17053, 18455, 16857, 16969, 17113, 16324, 16595, 84034]`; ink px ref/ours 112671/123031 (ratio 1.0919); SSIM blocks <0.9: 14141/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 19.0] pt by ink-projection correlation (centroid estimate [-13.19, 40.58] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.5474, differing 0.193375, SSIM₈ 0.5595 (raw 27.3174, 0.197135, 0.5487)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2795→0.3154 / 43.6543→41.6662; header-band 1.0→0.8707 / 0.0→5.2138; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1629793, 22464, 19346, 18927, 16188, 17047, 16876, 15463, 15575, 16172, 15284, 15364, 15248, 14936, 14590, 75543]`; ink px ref/ours 112666/99249 (ratio 0.8809); SSIM blocks <0.9: 12431/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-13.66, -17.97] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.3242, differing 0.170222, SSIM₈ 0.6238 (raw 24.5081, 0.176828, 0.6031)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3663→0.3994 / 39.1644→37.2722; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 95.02; `branch` dx -435.52 dy 74.98; `the` dx -430.89 dy 94.97
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1648831, 22951, 19301, 18581, 16876, 17397, 16238, 16200, 15602, 15712, 14968, 14797, 13632, 13342, 13823, 60565]`; ink px ref/ours 112652/111937 (ratio 0.9937); SSIM blocks <0.9: 11598/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 14.0] pt by ink-projection correlation (centroid estimate [-5.44, 28.39] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.109, differing 0.162119, SSIM₈ 0.6746 (raw 22.0424, 0.166138, 0.655)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4494→0.483 / 35.222→33.6211; header-band 1.0→0.9856 / 0.0→0.7483; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1648706, 22969, 19322, 18593, 16876, 17386, 16242, 16180, 15613, 15721, 14972, 14801, 13642, 13333, 13845, 60615]`; ink px ref/ours 112671/112014 (ratio 0.9942); SSIM blocks <0.9: 11601/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 14.0] pt by ink-projection correlation (centroid estimate [-5.52, 28.23] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.9629, differing 0.161597, SSIM₈ 0.6761 (raw 22.0535, 0.166208, 0.6548)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4492→0.4879 / 35.2397→33.3415; header-band 1.0→0.9849 / 0.0→0.8283; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1648687, 22966, 19324, 18600, 16876, 17393, 16252, 16178, 15609, 15727, 14971, 14791, 13642, 13329, 13826, 60645]`; ink px ref/ours 112666/111986 (ratio 0.994); SSIM blocks <0.9: 11604/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 14.0] pt by ink-projection correlation (centroid estimate [-5.52, 28.35] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.9589, differing 0.161593, SSIM₈ 0.676 (raw 22.0547, 0.166227, 0.6547)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4491→0.4877 / 35.2417→33.3396; header-band 1.0→0.9854 / 0.0→0.7969; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 57.73; `oak` dx -415.83 dy 57.73; `oak` dx -415.83 dy 57.73
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610318, 25425, 21373, 20765, 19023, 18579, 20017, 19398, 18959, 17683, 15949, 15816, 15930, 14976, 15077, 69528]`; ink px ref/ours 85700/117085 (ratio 1.3662); SSIM blocks <0.9: 14321/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.38, 37.44] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.2379, differing 0.178125, SSIM₈ 0.5841 (raw 25.0181, 0.188527, 0.5353)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2695→0.3379 / 39.3224→37.0299; header-band 1.0→0.9865 / 0.0→0.7294; footer-band 0.9191→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1605962, 25795, 21685, 21190, 19399, 18827, 20728, 19623, 19095, 18008, 16024, 16087, 15990, 15105, 15255, 70043]`; ink px ref/ours 85737/123031 (ratio 1.435); SSIM blocks <0.9: 14653/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -24.0] pt by ink-projection correlation (centroid estimate [-6.94, 13.29] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.1632, differing 0.185064, SSIM₈ 0.564 (raw 25.2979, 0.191097, 0.5322)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2527→0.3176 / 40.4318→37.8903; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.905 / 0.0→4.9832
- page 3: |Δ| histogram (16 bins, pixel counts) `[1641003, 23259, 19653, 18937, 17565, 16980, 18298, 17808, 17679, 16024, 14541, 14418, 14252, 13491, 13706, 61202]`; ink px ref/ours 85734/99249 (ratio 1.1576); SSIM blocks <0.9: 13047/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -24.0] pt by ink-projection correlation (centroid estimate [-7.37, -45.33] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.4784, differing 0.165451, SSIM₈ 0.6037 (raw 22.5164, 0.171032, 0.5806)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3301→0.3669 / 35.9864→34.3274; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -92.03; `oak` dx 425.86 dy -86.44; `oak` dx 425.86 dy -80.85
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1725717, 24686, 19843, 18956, 17584, 17329, 15373, 14819, 11828, 10838, 9307, 8521, 7667, 7067, 6807, 22474]`; ink px ref/ours 85700/111937 (ratio 1.3061); SSIM blocks <0.9: 9096/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.72, 0.97] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1245, differing 0.128723, SSIM₈ 0.8177 (raw 13.1245, 0.128723, 0.8177)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.709→0.709 / 20.9748→20.9748; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1725532, 24700, 19867, 18974, 17579, 17333, 15382, 14787, 11841, 10835, 9308, 8534, 7682, 7085, 6823, 22554]`; ink px ref/ours 85737/112014 (ratio 1.3065); SSIM blocks <0.9: 9100/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.73, 0.94] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1422, differing 0.128826, SSIM₈ 0.8175 (raw 13.1422, 0.128826, 0.8175)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7086→0.7086 / 21.0031→21.0031; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1725502, 24708, 19869, 18978, 17578, 17322, 15387, 14785, 11849, 10849, 9305, 8528, 7682, 7078, 6810, 22586]`; ink px ref/ours 85734/111986 (ratio 1.3062); SSIM blocks <0.9: 9103/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.76, 0.99] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1447, differing 0.128849, SSIM₈ 0.8174 (raw 13.1447, 0.128849, 0.8174)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7085→0.7085 / 21.0072→21.0072; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Part` dx 29.06 dy 0.96; `1` dx 29.06 dy 0.96; `Part` dx 29.06 dy 0.96
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1520055, 31716, 27435, 25609, 22467, 22765, 22818, 20885, 21580, 21981, 20475, 20075, 20772, 20283, 20130, 99770]`; ink px ref/ours 152381/145750 (ratio 0.9565); SSIM blocks <0.9: 16622/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.0, 1.0] pt REJECTED: applying it gives mean|Δ| 33.0253, not lower; centroid estimate [-3.46, -2.97] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 32.9083, differing 0.239416, SSIM₈ 0.4723 (raw 32.9083, 0.239416, 0.4723)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1574→0.1574 / 52.584→52.584; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9989→0.9989 / 0.0437→0.0437
- page 2: |Δ| histogram (16 bins, pixel counts) `[1846833, 6944, 5793, 5739, 4609, 4735, 4584, 4457, 4418, 4676, 4347, 4432, 4703, 4258, 4447, 23841]`; ink px ref/ours 29740/36694 (ratio 1.2338); SSIM blocks <0.9: 4067/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 14.5] pt by ink-projection correlation (centroid estimate [2.01, 13.68] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.2145, differing 0.051795, SSIM₈ 0.882 (raw 7.3773, 0.052706, 0.8753)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8008→0.8251 / 11.7895→10.8075; header-band 1.0→0.907 / 0.0→4.9674; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 49.3; `branch` dx -435.47 dy 37.03; `branch` dx -435.47 dy 31.49
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1582532, 30565, 26059, 23430, 21516, 20347, 19326, 19431, 18874, 18237, 17636, 17063, 16112, 16206, 16728, 74754]`; ink px ref/ours 152381/137064 (ratio 0.8995); SSIM blocks <0.9: 14333/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 26.7909, not lower; centroid estimate [-1.35, -1.22] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.76, differing 0.206087, SSIM₈ 0.5915 (raw 26.76, 0.206087, 0.5915)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.349→0.349 / 42.7396→42.7396; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9954→0.9954 / 0.1454→0.1454
- page 2: |Δ| histogram (16 bins, pixel counts) `[1835170, 7811, 6562, 6090, 5720, 5564, 5257, 5613, 4906, 5202, 4998, 5001, 4717, 4602, 4797, 26806]`; ink px ref/ours 29740/42779 (ratio 1.4384); SSIM blocks <0.9: 4835/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 57.5] pt by ink-projection correlation (centroid estimate [7.47, 30.15] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.994, differing 0.057773, SSIM₈ 0.8603 (raw 8.2597, 0.059532, 0.8508)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.762→0.8335 / 13.1983→10.0367; header-band 1.0→0.6129 / 0.0→18.8087; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 85.92; `branch` dx -413.6 dy 85.92; `oak` dx -415.78 dy 58.19
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1555403, 30464, 26222, 24502, 22637, 22420, 23203, 22359, 22283, 21332, 18820, 18660, 18216, 17639, 17127, 77529]`; ink px ref/ours 104967/145750 (ratio 1.3885); SSIM blocks <0.9: 16296/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -27.0] pt by ink-projection correlation (centroid estimate [-2.53, -2.2] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 28.7887, differing 0.220223, SSIM₈ 0.4819 (raw 28.8185, 0.220086, 0.4824)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.174→0.198 / 46.0469→44.613; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.8271 / 0.0747→9.5919
- page 2: |Δ| histogram (16 bins, pixel counts) `[1833852, 8634, 7497, 6954, 6525, 6128, 6355, 6191, 5696, 5450, 5031, 4992, 4946, 4724, 4746, 21095]`; ink px ref/ours 33407/36694 (ratio 1.0984); SSIM blocks <0.9: 4932/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 7.8306, not lower; centroid estimate [-2.86, -17.23] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.8181, differing 0.060403, SSIM₈ 0.8517 (raw 7.8181, 0.060403, 0.8517)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7632→0.7632 / 12.4946→12.4946; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -51.41; `oak` dx 425.86 dy -41.46; `oak` dx 425.86 dy -32.56
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1679221, 30257, 24613, 23116, 21715, 20687, 18617, 17592, 14612, 13077, 11214, 10487, 9393, 9073, 7998, 27144]`; ink px ref/ours 104967/137064 (ratio 1.3058); SSIM blocks <0.9: 11040/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.42, -0.44] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9638, differing 0.157272, SSIM₈ 0.7777 (raw 15.9638, 0.157272, 0.7777)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6455→0.6455 / 25.4994→25.4994; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9982→0.9982 / 0.0806→0.0806
- page 2: |Δ| histogram (16 bins, pixel counts) `[1856196, 9316, 7711, 7085, 6839, 6464, 5921, 5530, 4547, 4079, 3580, 3328, 3105, 2821, 2548, 9746]`; ink px ref/ours 33407/42779 (ratio 1.2805); SSIM blocks <0.9: 3542/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.6, -0.75] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 5.1883, differing 0.049859, SSIM₈ 0.9274 (raw 5.1883, 0.049859, 0.9274)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8842→0.8842 / 8.291→8.291; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Orphan` dx 29.06 dy -0.69; `Heading` dx 29.06 dy -0.69; `below.` dx 0.04 dy -0.39
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/16-heading-page-break/pdflatex-de1020c-export-p1-overlay.png) (56499 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-de1020c-export-p1-heatmap.png) (38281 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -14.5] pt by ink-projection correlation (centroid estimate [-6.07, -5.06] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.4928, differing 0.229367, SSIM₈ 0.4891 (raw 31.5764, 0.22951, 0.4881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1821→0.1892 / 50.4615→50.1371; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.96 / 0.0468→1.3594
- page 2: |Δ| histogram (16 bins, pixel counts) `[1821555, 8081, 6675, 6513, 5830, 5892, 6026, 5614, 5619, 6564, 5700, 5695, 5919, 5335, 5734, 32064]`; ink px ref/ours 29765/46935 (ratio 1.5769); SSIM blocks <0.9: 5085/30294; [overlay](images/16-heading-page-break/pdflatex-de1020c-export-p2-overlay.png) (53012 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-de1020c-export-p2-heatmap.png) (47437 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 20.5] pt by ink-projection correlation (centroid estimate [1.21, 55.26] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.5969, differing 0.061448, SSIM₈ 0.8517 (raw 9.6191, 0.066682, 0.8358)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7375→0.764 / 15.3741→13.6882; header-band 1.0→0.9933 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 127.11; `branch` dx -435.47 dy 74.69; `branch` dx -435.47 dy 54.75

### 16-heading-page-break — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1520838, 31563, 27074, 24807, 22509, 23307, 22498, 21004, 21495, 22304, 20775, 19954, 20253, 19497, 19992, 100946]`; ink px ref/ours 151753/145750 (ratio 0.9604); SSIM blocks <0.9: 16544/30294; [overlay](images/16-heading-page-break/pdflatex-main-export-p1-overlay.png) (57478 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-main-export-p1-heatmap.png) (38528 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-11.0, 0.0] pt REJECTED: applying it gives mean|Δ| 32.933, not lower; centroid estimate [-4.21, -3.98] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 32.9189, differing 0.238789, SSIM₈ 0.4737 (raw 32.9189, 0.238789, 0.4737)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1593→0.1593 / 52.6066→52.6066; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.9984 / 0.0468→0.0468
- page 2: |Δ| histogram (16 bins, pixel counts) `[1846805, 6715, 5911, 5450, 4855, 4877, 4776, 4476, 4610, 4607, 4225, 4310, 4650, 4145, 4422, 23982]`; ink px ref/ours 29765/36694 (ratio 1.2328); SSIM blocks <0.9: 4055/30294; [overlay](images/16-heading-page-break/pdflatex-main-export-p2-overlay.png) (49272 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-main-export-p2-heatmap.png) (40544 B, ÷4)
  - registration error (diagnostic): global shift [-0.5, 14.5] pt by ink-projection correlation (centroid estimate [1.5, 12.9] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.1674, differing 0.05157, SSIM₈ 0.8836 (raw 7.3756, 0.052561, 0.8765)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8026→0.8275 / 11.7883→10.7329; header-band 1.0→0.9077 / 0.0→4.9674; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 49.11; `branch` dx -435.47 dy 37.03; `branch` dx -435.47 dy 31.49
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1583040, 30686, 25741, 23074, 21550, 20486, 19257, 19723, 18532, 18146, 17688, 17169, 15740, 15834, 16380, 75770]`; ink px ref/ours 151753/137064 (ratio 0.9032); SSIM blocks <0.9: 14229/30294; [overlay](images/16-heading-page-break/pdflatex-pipeline-export-p1-overlay.png) (57758 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-pipeline-export-p1-heatmap.png) (36401 B, ÷8)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.1, -2.23] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.6362, differing 0.205191, SSIM₈ 0.5954 (raw 26.7545, 0.205539, 0.5933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3515→0.3576 / 42.7356→42.4824; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9951→0.9953 / 0.1484→0.1484
- page 2: |Δ| histogram (16 bins, pixel counts) `[1835163, 7566, 6693, 5802, 5953, 5722, 5427, 5579, 5192, 5291, 4897, 4809, 4739, 4495, 4809, 26679]`; ink px ref/ours 29765/42779 (ratio 1.4372); SSIM blocks <0.9: 4811/30294; [overlay](images/16-heading-page-break/pdflatex-pipeline-export-p2-overlay.png) (52723 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-pipeline-export-p2-heatmap.png) (44101 B, ÷4)
  - registration error (diagnostic): global shift [-1.5, 57.5] pt by ink-projection correlation (centroid estimate [6.96, 29.38] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.8713, differing 0.057358, SSIM₈ 0.8637 (raw 8.2453, 0.05925, 0.8523)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7642→0.8388 / 13.1768→9.8322; header-band 1.0→0.6153 / 0.0→18.8087; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 85.73; `branch` dx -413.61 dy 85.73; `oak` dx -415.72 dy 58.19
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/16-heading-page-break/pdflatex-lm-de1020c-export-p1-overlay.png) (54993 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-de1020c-export-p1-heatmap.png) (37553 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.44, -3.32] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4403, differing 0.210395, SSIM₈ 0.5038 (raw 27.5467, 0.210674, 0.5026)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.208 / 44.0147→43.8447; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1804006, 9924, 8335, 8540, 7532, 7526, 7743, 7505, 7333, 7253, 6797, 6490, 6663, 5901, 6567, 30701]`; ink px ref/ours 33505/46935 (ratio 1.4008); SSIM blocks <0.9: 6195/30294; [overlay](images/16-heading-page-break/pdflatex-lm-de1020c-export-p2-overlay.png) (59722 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-de1020c-export-p2-heatmap.png) (52924 B, ÷4)
  - registration error (diagnostic): global shift [-1.0, 20.0] pt by ink-projection correlation (centroid estimate [-2.85, 25.0] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.6575, differing 0.066846, SSIM₈ 0.8469 (raw 10.4882, 0.077169, 0.7985)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6782→0.7566 / 16.7624→13.7844; header-band 1.0→0.9928 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 27.65; `oak` dx 426.94 dy 21.13; `oak` dx 426.94 dy -13.72

### 16-heading-page-break — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1555521, 30291, 26058, 24656, 22827, 22307, 23426, 22015, 22563, 21069, 18990, 18590, 18306, 17279, 17582, 77336]`; ink px ref/ours 105111/145750 (ratio 1.3866); SSIM blocks <0.9: 16299/30294; [overlay](images/16-heading-page-break/pdflatex-lm-main-export-p1-overlay.png) (56078 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-main-export-p1-heatmap.png) (38113 B, ÷8)
  - registration error (diagnostic): global shift [-0.5, -27.0] pt by ink-projection correlation (centroid estimate [-2.58, -2.25] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 28.7887, differing 0.220226, SSIM₈ 0.482 (raw 28.8184, 0.220067, 0.4825)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.174→0.198 / 46.0467→44.613; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.8271 / 0.0747→9.5919
- page 2: |Δ| histogram (16 bins, pixel counts) `[1833836, 8612, 7357, 7269, 6381, 6293, 6217, 6112, 5814, 5336, 5154, 4953, 4927, 4585, 4899, 21071]`; ink px ref/ours 33505/36694 (ratio 1.0952); SSIM blocks <0.9: 4934/30294; [overlay](images/16-heading-page-break/pdflatex-lm-main-export-p2-overlay.png) (55495 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-main-export-p2-heatmap.png) (44296 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 7.8298, not lower; centroid estimate [-2.57, -17.36] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.819, differing 0.060445, SSIM₈ 0.8514 (raw 7.819, 0.060445, 0.8514)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7628→0.7628 / 12.4961→12.4961; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -50.35; `oak` dx 425.86 dy -40.43; `oak` dx 425.86 dy -31.53
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1678891, 30013, 24898, 23141, 21598, 20781, 18655, 17662, 14743, 12707, 11482, 10476, 9413, 8881, 8415, 27060]`; ink px ref/ours 105111/137064 (ratio 1.304); SSIM blocks <0.9: 11045/30294; [overlay](images/16-heading-page-break/pdflatex-lm-pipeline-export-p1-overlay.png) (54606 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-pipeline-export-p1-heatmap.png) (82298 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.48, -0.49] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9984, differing 0.157345, SSIM₈ 0.7771 (raw 15.9984, 0.157345, 0.7771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6447→0.6447 / 25.5546→25.5546; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9981→0.9981 / 0.0813→0.0813
- page 2: |Δ| histogram (16 bins, pixel counts) `[1856090, 9303, 7748, 7230, 6775, 6555, 5805, 5649, 4503, 3952, 3673, 3277, 3106, 2781, 2627, 9742]`; ink px ref/ours 33505/42779 (ratio 1.2768); SSIM blocks <0.9: 3547/30294; [overlay](images/16-heading-page-break/pdflatex-lm-pipeline-export-p2-overlay.png) (58664 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-pipeline-export-p2-heatmap.png) (38932 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.89, -0.89] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 5.1925, differing 0.049894, SSIM₈ 0.9274 (raw 5.1925, 0.049894, 0.9274)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8842→0.8842 / 8.2978→8.2978; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Heading` dx 29.07 dy 0.96; `Orphan` dx 29.06 dy 0.96; `below.` dx 0.04 dy 0.67
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1519809, 31249, 27150, 25991, 22791, 23249, 22910, 20903, 21466, 21981, 20847, 20488, 20635, 20015, 19732, 99600]`; ink px ref/ours 152232/145750 (ratio 0.9574); SSIM blocks <0.9: 16615/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.0, 1.0] pt REJECTED: applying it gives mean|Δ| 33.0193, not lower; centroid estimate [-3.49, -2.94] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 32.9002, differing 0.239463, SSIM₈ 0.4726 (raw 32.9002, 0.239463, 0.4726)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1579→0.1579 / 52.571→52.571; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9987→0.9987 / 0.0438→0.0438
- page 2: |Δ| histogram (16 bins, pixel counts) `[1846798, 6861, 5781, 5815, 4667, 4750, 4542, 4455, 4388, 4640, 4462, 4616, 4641, 4187, 4434, 23779]`; ink px ref/ours 29777/36694 (ratio 1.2323); SSIM blocks <0.9: 4070/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 14.5] pt by ink-projection correlation (centroid estimate [1.96, 13.75] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.2101, differing 0.051785, SSIM₈ 0.8821 (raw 7.378, 0.052705, 0.8752)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8008→0.8252 / 11.7906→10.8004; header-band 1.0→0.907 / 0.0→4.9674; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 49.3; `branch` dx -435.52 dy 37.03; `branch` dx -435.52 dy 31.49
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1581634, 30214, 25515, 23782, 21671, 20933, 19589, 19568, 18768, 18249, 17860, 17780, 16063, 16138, 16195, 74857]`; ink px ref/ours 152232/137064 (ratio 0.9004); SSIM blocks <0.9: 14337/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.39, -1.18] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.718, differing 0.206581, SSIM₈ 0.5911 (raw 26.825, 0.206396, 0.5903)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3471→0.3519 / 42.8432→42.5781; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9952→0.9948 / 0.1455→0.1455
- page 2: |Δ| histogram (16 bins, pixel counts) `[1835154, 7792, 6528, 6150, 5741, 5655, 5130, 5566, 4831, 5249, 5137, 5179, 4670, 4481, 4896, 26657]`; ink px ref/ours 29777/42779 (ratio 1.4366); SSIM blocks <0.9: 4833/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 57.5] pt by ink-projection correlation (centroid estimate [7.42, 30.22] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.0014, differing 0.057851, SSIM₈ 0.8601 (raw 8.2559, 0.05949, 0.8509)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7621→0.8333 / 13.1921→10.0483; header-band 1.0→0.6129 / 0.0→18.8087; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 85.92; `branch` dx -413.65 dy 85.92; `oak` dx -415.83 dy 58.19
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1555389, 30504, 26238, 24493, 22519, 22489, 23291, 22233, 22382, 21192, 18927, 18604, 18239, 17654, 17107, 77555]`; ink px ref/ours 105020/145750 (ratio 1.3878); SSIM blocks <0.9: 16300/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -27.0] pt by ink-projection correlation (centroid estimate [-2.6, -2.13] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 28.7888, differing 0.220187, SSIM₈ 0.4819 (raw 28.8189, 0.22004, 0.4824)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.174→0.198 / 46.0476→44.6131; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.8271 / 0.0747→9.592
- page 2: |Δ| histogram (16 bins, pixel counts) `[1833847, 8630, 7536, 6929, 6488, 6147, 6387, 6170, 5713, 5406, 5078, 4988, 4920, 4721, 4764, 21092]`; ink px ref/ours 33418/36694 (ratio 1.098); SSIM blocks <0.9: 4932/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 7.8302, not lower; centroid estimate [-2.82, -17.23] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.818, differing 0.060377, SSIM₈ 0.8517 (raw 7.818, 0.060377, 0.8517)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7632→0.7632 / 12.4945→12.4945; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -50.38; `oak` dx 425.86 dy -40.43; `oak` dx 425.86 dy -31.53
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1679213, 30197, 24622, 23082, 21587, 20834, 18656, 17658, 14519, 13108, 11247, 10477, 9366, 9061, 8023, 27166]`; ink px ref/ours 105020/137064 (ratio 1.3051); SSIM blocks <0.9: 11040/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.5, -0.38] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.968, differing 0.15725, SSIM₈ 0.7776 (raw 15.968, 0.15725, 0.7776)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6454→0.6454 / 25.506→25.506; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9982→0.9982 / 0.0808→0.0808
- page 2: |Δ| histogram (16 bins, pixel counts) `[1856213, 9290, 7721, 7034, 6830, 6505, 5913, 5568, 4513, 4087, 3592, 3329, 3087, 2825, 2563, 9746]`; ink px ref/ours 33418/42779 (ratio 1.2801); SSIM blocks <0.9: 3545/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.65, -0.75] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 5.1895, differing 0.049844, SSIM₈ 0.9274 (raw 5.1895, 0.049844, 0.9274)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8841→0.8841 / 8.2931→8.2931; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Orphan` dx 29.06 dy 0.94; `Heading` dx 29.06 dy 0.94; `below.` dx 0.04 dy 0.67
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927470, 1158, 889, 729, 618, 585, 586, 551, 613, 500, 573, 515, 560, 474, 531, 2464]`; ink px ref/ours 5219/5222 (ratio 1.0006); SSIM blocks <0.9: 453/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-7.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.81, 0.07] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8284, differing 0.006836, SSIM₈ 0.9876 (raw 0.8494, 0.00683, 0.9875)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.98→0.9803 / 1.3576→1.3223; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -3.05 dy 0.42; `quotes,` dx -1.61 dy 0.42; `and` dx -1.47 dy 0.42
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926664, 1040, 842, 822, 792, 771, 791, 717, 808, 626, 650, 592, 590, 544, 535, 2032]`; ink px ref/ours 3891/5222 (ratio 1.3421); SSIM blocks <0.9: 567/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [12.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8967, not lower; centroid estimate [-11.81, -1.08] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8761, differing 0.007071, SSIM₈ 0.9846 (raw 0.8761, 0.007071, 0.9846)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9754→0.9754 / 1.4002→1.4002; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.63 dy -0.35; `plain` dx -61.38 dy -0.35; `a` dx -59.95 dy -0.35
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1928624, 1020, 806, 796, 801, 765, 702, 568, 695, 586, 516, 487, 435, 360, 371, 1284]`; ink px ref/ours 3891/5184 (ratio 1.3323); SSIM blocks <0.9: 460/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.83, -0.2] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.6315, differing 0.005967, SSIM₈ 0.99 (raw 0.675, 0.006094, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9834→0.984 / 1.0788→1.0092; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `’` dx 0.04 dy -0.36; `apostrophe.` dx 0.04 dy -0.36; `a` dx 0.03 dy -0.36

### 17-apostrophes — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927689, 1087, 914, 809, 742, 653, 583, 605, 537, 460, 552, 502, 496, 521, 444, 2222]`; ink px ref/ours 5173/5099 (ratio 0.9857); SSIM blocks <0.9: 463/30294; [overlay](images/17-apostrophes/pdflatex-de1020c-export-p1-overlay.png) (41421 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-de1020c-export-p1-heatmap.png) (86595 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-7.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8459, not lower; centroid estimate [-5.05, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8063, differing 0.006638, SSIM₈ 0.9887 (raw 0.8063, 0.006638, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9819→0.9819 / 1.2887→1.2887; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -7.83 dy 0.46; `roll,` dx -6.97 dy 0.46; `the` dx -6.5 dy 0.46
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927557, 1042, 927, 864, 724, 650, 665, 624, 530, 509, 592, 484, 478, 492, 471, 2207]`; ink px ref/ours 5173/5222 (ratio 1.0095); SSIM blocks <0.9: 461/30294; [overlay](images/17-apostrophes/pdflatex-main-export-p1-overlay.png) (40797 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-main-export-p1-heatmap.png) (86866 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-8.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8537, not lower; centroid estimate [-3.29, 0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8138, differing 0.006695, SSIM₈ 0.9885 (raw 0.8138, 0.006695, 0.9885)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9817→0.9817 / 1.3007→1.3007; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -8.25 dy 0.46; `roll,` dx -7.39 dy 0.46; `the` dx -6.92 dy 0.46
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926596, 1012, 822, 811, 701, 702, 623, 608, 613, 642, 649, 626, 601, 559, 636, 2615]`; ink px ref/ours 5173/5184 (ratio 1.0021); SSIM blocks <0.9: 540/30294; [overlay](images/17-apostrophes/pdflatex-pipeline-export-p1-overlay.png) (41338 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-pipeline-export-p1-heatmap.png) (89674 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-7.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.976, not lower; centroid estimate [10.35, 0.92] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9356, differing 0.007223, SSIM₈ 0.9856 (raw 0.9356, 0.007223, 0.9856)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.977→0.977 / 1.4954→1.4954; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `“quoted”` dx -424.84 dy 14.85; `apostrophe.` dx 62.72 dy 0.41; `’` dx 62.54 dy 0.41

### 17-apostrophes — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926770, 1005, 918, 847, 826, 791, 712, 833, 730, 582, 566, 566, 556, 521, 500, 2093]`; ink px ref/ours 3904/5099 (ratio 1.3061); SSIM blocks <0.9: 564/30294; [overlay](images/17-apostrophes/pdflatex-lm-de1020c-export-p1-overlay.png) (42110 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-lm-de1020c-export-p1-heatmap.png) (87856 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.84, -1.09] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8624, differing 0.00704, SSIM₈ 0.9848 (raw 0.8624, 0.00704, 0.9848)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9757→0.9757 / 1.3784→1.3784; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.51 dy 0.68; `plain` dx -61.24 dy 0.68; `a` dx -59.82 dy 0.68
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926650, 1037, 849, 834, 791, 782, 743, 772, 803, 632, 636, 590, 589, 528, 517, 2063]`; ink px ref/ours 3904/5222 (ratio 1.3376); SSIM blocks <0.9: 566/30294; [overlay](images/17-apostrophes/pdflatex-lm-main-export-p1-overlay.png) (41398 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-lm-main-export-p1-heatmap.png) (87609 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [12.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8973, not lower; centroid estimate [-12.08, -1.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8762, differing 0.007078, SSIM₈ 0.9846 (raw 0.8762, 0.007078, 0.9846)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9754→0.9754 / 1.4005→1.4005; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.63 dy 0.68; `plain` dx -61.36 dy 0.68; `a` dx -59.94 dy 0.68
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1928632, 1020, 778, 820, 806, 744, 673, 625, 677, 612, 490, 486, 428, 362, 371, 1292]`; ink px ref/ours 3904/5184 (ratio 1.3279); SSIM blocks <0.9: 461/30294; [overlay](images/17-apostrophes/pdflatex-lm-pipeline-export-p1-overlay.png) (40946 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-lm-pipeline-export-p1-heatmap.png) (85532 B, ÷1)
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927694, 1210, 922, 781, 623, 622, 609, 567, 538, 472, 564, 477, 533, 497, 495, 2212]`; ink px ref/ours 5210/5222 (ratio 1.0023); SSIM blocks <0.9: 444/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.848, not lower; centroid estimate [-2.02, 0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8069, differing 0.006687, SSIM₈ 0.9882 (raw 0.8069, 0.006687, 0.9882)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9811→0.9811 / 1.2897→1.2897; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -8.33 dy 0.46; `roll,` dx -7.25 dy 0.46; `the` dx -6.73 dy 0.46
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926655, 1050, 844, 818, 792, 770, 798, 713, 806, 633, 643, 584, 603, 539, 543, 2025]`; ink px ref/ours 3889/5222 (ratio 1.3428); SSIM blocks <0.9: 566/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [12.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8968, not lower; centroid estimate [-11.28, -1.08] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.876, differing 0.00707, SSIM₈ 0.9846 (raw 0.876, 0.00707, 0.9846)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9754→0.9754 / 1.4001→1.4001; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.63 dy 0.68; `plain` dx -61.38 dy 0.68; `a` dx -59.95 dy 0.68
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923086, 2009, 1584, 1323, 1042, 951, 872, 743, 719, 715, 653, 578, 670, 587, 615, 2669]`; ink px ref/ours 8390/8245 (ratio 0.9827); SSIM blocks <0.9: 583/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-3.42, 0.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0519, differing 0.009669, SSIM₈ 0.9851 (raw 1.0519, 0.009669, 0.9851)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9762→0.9762 / 1.6812→1.6812; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `ruffled,` dx -14.51 dy 0.42; `offline,` dx -13.45 dy 0.42; `baffling,` dx -12.39 dy 0.42
- word-sequence differences: replace ref ['fluffy,', 'suffix,'] ours ['fluffy,suffix,']

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1919468, 1514, 1315, 1392, 1274, 1173, 1187, 1186, 1171, 993, 941, 901, 1013, 830, 844, 3614]`; ink px ref/ours 6488/8245 (ratio 1.2708); SSIM blocks <0.9: 827/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [3.72, -1.27] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3902, differing 0.010983, SSIM₈ 0.9769 (raw 1.4225, 0.011085, 0.9764)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9623→0.9631 / 2.2735→2.222; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx 442.91 dy -14.75; `ruffled,` dx 417.97 dy -14.79; `offline,` dx -53.54 dy -0.35
- word-sequence differences: replace ref ['fluffy,', 'suffix,'] ours ['fluffy,suffix,']

### 18-ligatures — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923492, 1777, 1322, 1424, 1289, 1091, 1053, 973, 820, 654, 652, 694, 583, 492, 503, 1997]`; ink px ref/ours 6488/8210 (ratio 1.2654); SSIM blocks <0.9: 682/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [5.75, -0.37] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9754, differing 0.009185, SSIM₈ 0.986 (raw 0.9754, 0.009185, 0.986)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9778→0.9778 / 1.5566→1.5566; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `office,` dx -1.49 dy -0.36; `effect,` dx -1.33 dy -0.36; `affine,` dx -1.16 dy -0.36

### 18-ligatures — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921093, 1851, 1396, 1273, 1121, 942, 936, 833, 874, 895, 836, 811, 990, 726, 718, 3521]`; ink px ref/ours 8210/8297 (ratio 1.0106); SSIM blocks <0.9: 724/30294; [overlay](images/18-ligatures/pdflatex-de1020c-export-p1-overlay.png) (47916 B, ÷2), [heatmap](images/18-ligatures/pdflatex-de1020c-export-p1-heatmap.png) (40344 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.13, 0.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2894, differing 0.010507, SSIM₈ 0.9808 (raw 1.2894, 0.010507, 0.9808)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9693→0.9693 / 2.0608→2.0608; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.86 dy 14.86; `ruffled,` dx -433.44 dy 14.82; `muffin.` dx 40.63 dy 0.37

### 18-ligatures — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920596, 1679, 1357, 1302, 1018, 948, 1054, 850, 849, 737, 810, 872, 952, 822, 894, 4076]`; ink px ref/ours 8210/8245 (ratio 1.0043); SSIM blocks <0.9: 677/30294; [overlay](images/18-ligatures/pdflatex-main-export-p1-overlay.png) (46785 B, ÷2), [heatmap](images/18-ligatures/pdflatex-main-export-p1-heatmap.png) (39061 B, ÷2)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-4.03, -0.01] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2545, differing 0.010252, SSIM₈ 0.9824 (raw 1.3805, 0.010672, 0.9805)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9689→0.9719 / 2.2064→2.0051; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `ruffled,` dx -15.47 dy 0.42; `offline,` dx -14.74 dy 0.42; `baffling,` dx -14.05 dy 0.42
- word-sequence differences: replace ref ['fluffy,', 'suffix,'] ours ['fluffy,suffix,']

### 18-ligatures — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920475, 1775, 1339, 1190, 1091, 1041, 1012, 935, 815, 878, 851, 913, 1034, 852, 864, 3751]`; ink px ref/ours 8210/8210 (ratio 1.0); SSIM blocks <0.9: 767/30294; [overlay](images/18-ligatures/pdflatex-pipeline-export-p1-overlay.png) (47785 B, ÷2), [heatmap](images/18-ligatures/pdflatex-pipeline-export-p1-heatmap.png) (40297 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.4074, not lower; centroid estimate [-2.0, 0.89] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3694, differing 0.010829, SSIM₈ 0.9794 (raw 1.3694, 0.010829, 0.9794)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9672→0.9672 / 2.1863→2.1863; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.86 dy 14.85; `ruffled,` dx -433.44 dy 14.85; `muffin.` dx 46.44 dy 0.41

### 18-ligatures — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920380, 1602, 1329, 1334, 1326, 1152, 1112, 1125, 1111, 987, 890, 943, 938, 768, 690, 3129]`; ink px ref/ours 6473/8297 (ratio 1.2818); SSIM blocks <0.9: 798/30294; [overlay](images/18-ligatures/pdflatex-lm-de1020c-export-p1-overlay.png) (49358 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-de1020c-export-p1-heatmap.png) (41456 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-18.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3343, not lower; centroid estimate [-5.55, -0.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3148, differing 0.010687, SSIM₈ 0.9787 (raw 1.3148, 0.010687, 0.9787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.1014→2.1014; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `coffin,` dx -21.57 dy 0.73; `official,` dx -20.57 dy 0.73; `offline,` dx -20.28 dy 0.68

### 18-ligatures — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1919497, 1494, 1326, 1385, 1259, 1161, 1201, 1216, 1133, 1055, 906, 890, 996, 814, 862, 3621]`; ink px ref/ours 6473/8245 (ratio 1.2738); SSIM blocks <0.9: 826/30294; [overlay](images/18-ligatures/pdflatex-lm-main-export-p1-overlay.png) (48416 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-main-export-p1-heatmap.png) (41384 B, ÷2)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [3.55, -1.26] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3906, differing 0.010975, SSIM₈ 0.9769 (raw 1.422, 0.011082, 0.9764)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9623→0.9631 / 2.2727→2.2225; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx 442.91 dy -13.72; `ruffled,` dx 417.97 dy -13.77; `offline,` dx -53.54 dy 0.68
- word-sequence differences: replace ref ['fluffy,', 'suffix,'] ours ['fluffy,suffix,']

### 18-ligatures — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923523, 1748, 1334, 1401, 1274, 1082, 1110, 928, 839, 689, 626, 662, 611, 465, 512, 2012]`; ink px ref/ours 6473/8210 (ratio 1.2683); SSIM blocks <0.9: 679/30294; [overlay](images/18-ligatures/pdflatex-lm-pipeline-export-p1-overlay.png) (47691 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-pipeline-export-p1-heatmap.png) (38618 B, ÷2)
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921437, 1689, 1490, 1303, 1178, 1026, 1058, 915, 928, 912, 721, 713, 818, 596, 757, 3275]`; ink px ref/ours 8366/8245 (ratio 0.9855); SSIM blocks <0.9: 673/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.17, -0.01] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1599, differing 0.010122, SSIM₈ 0.9838 (raw 1.2336, 0.010273, 0.9826)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9723→0.9741 / 1.9717→1.8539; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `ruffled,` dx -14.77 dy 0.42; `offline,` dx -13.73 dy 0.42; `baffling,` dx -12.7 dy 0.42
- word-sequence differences: replace ref ['fluffy,', 'suffix,'] ours ['fluffy,suffix,']

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1919469, 1510, 1327, 1382, 1288, 1147, 1214, 1175, 1166, 998, 937, 902, 1013, 821, 851, 3616]`; ink px ref/ours 6486/8245 (ratio 1.2712); SSIM blocks <0.9: 827/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [3.8, -1.27] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3902, differing 0.01098, SSIM₈ 0.9769 (raw 1.4224, 0.011082, 0.9764)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9623→0.9631 / 2.2734→2.222; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx 442.91 dy -13.72; `ruffled,` dx 417.97 dy -13.77; `offline,` dx -53.54 dy 0.68
- word-sequence differences: replace ref ['fluffy,', 'suffix,'] ours ['fluffy,suffix,']

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

- previous evidence: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a8603282c8958a68c/tests/visual-corpus/evidence/20260912T072838Z`; comparable entries: 648
- worse: 
  - 12-justified-paragraphs/lualatex-lm/main/export: ssim_8x8_mean 0.7503 -> 0.7469
  - 12-justified-paragraphs/lualatex-lm/main/preview: ssim_8x8_mean 0.7504 -> 0.747
  - 12-justified-paragraphs/pdflatex-lm/main/export: ssim_8x8_mean 0.7503 -> 0.7469
  - 12-justified-paragraphs/pdflatex-lm/main/preview: ssim_8x8_mean 0.7504 -> 0.747
  - 12-justified-paragraphs/xelatex-lm/main/export: ssim_8x8_mean 0.7503 -> 0.7469
  - 12-justified-paragraphs/xelatex-lm/main/preview: ssim_8x8_mean 0.7504 -> 0.747
  - 18-ligatures/lualatex-lm/main/export: ssim_8x8_mean 0.9787 -> 0.9764
  - 18-ligatures/pdflatex-lm/main/export: ssim_8x8_mean 0.9787 -> 0.9764
  - 18-ligatures/xelatex-lm/main/export: ssim_8x8_mean 0.9787 -> 0.9764

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
