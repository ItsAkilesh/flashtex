# FlashTeX visual corpus: reference-render and raster-diff evidence

Generated 20260912T100307Z on mac-m1max-a by `tests/visual-corpus/harness/run.sh`.

**Scope statement.** These are narrow-case measurements over a small declared corpus. They never claim general pixel perfection, LaTeX compatibility, or parity outside these fixtures, these engines, this font, this page size, this DPI and these builds. The reference engines are test oracles only; FlashTeX never invokes them and remains an original Rust implementation.

**Acceptance vs diagnostics.** The only acceptance signals in this report are the exact-equality gates below. They are three different questions and are never merged: (1) does FlashTeX reproduce its OWN pinned prior output (self-regression); (2) does the candidate match the ESTABLISHED ENGINE — raw, unmodified PDF bytes against the pinned MacTeX oracle PDF, and zero-pixel raster equality against the oracle raster; (3) does the export raster match the app's preview rasters (native/export parity). Every tolerance, threshold, SSIM, registration shift or regression comparison further down is a diagnostic to explain *why* something differs; none of them ever counts as acceptance. Nothing is normalised for acceptance: not IDs, not timestamps, not offsets.

**Plain statement.** Established-engine parity is NOT claimed: raw PDF bytes equal the pinned oracle in 0/432 candidate×oracle pairs and rasters are zero-pixel equal in 0/432. FlashTeX self-regression (bytes equal to its own pinned prior output) holds for 9/72 candidates — that is reproducibility of FlashTeX against itself, not LaTeX parity. Export = preview-equivalent for 0/72; export = native capture for 0/0 available captures (72 unavailable). Oracle pins live this run: 108/108 pinned oracle PDFs were re-rendered byte-identically by the installed engine.

## Gate 2 — candidate vs established engine (acceptance)

Oracle: MacTeX/TeX Live pdflatex (see Provenance for distribution, version, fonts, preamble, flags and the pinned render environment). Raw bytes: SHA-256 of the candidate `flashtex.pdf` vs the pinned oracle PDF from `oracle-profile.json` and vs this run's live oracle render, unmodified. Pixels: the export raster vs the oracle raster, same rasterizer, same DPI, page by page, zero tolerance; a page-count mismatch is DIFFERENT. Times-font oracles (`pdflatex`) match the current compiler's metrics; the Latin Modern oracles (`pdflatex-lm`) are LaTeX's default look. xelatex/lualatex variants are in `metrics.json` → `gates[].oracle`.

| Fixture | Candidate | Oracle | raw PDF bytes = pinned oracle | raw PDF bytes = live oracle | oracle live = pin | zero-pixel raster = oracle | classify categories (exact route vs pdflatex-lm) |
|---|---|---|---|---|---|---|---|
| 01-plain-paragraph | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4577/1938816 px, max |Δ| 255 | - |
| 01-plain-paragraph | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5863/1938816 px, max |Δ| 255 | - |
| 01-plain-paragraph | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5927/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 01-plain-paragraph | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 1754/1938816 px, max |Δ| 4 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 01-plain-paragraph | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4225/1938816 px, max |Δ| 255 | - |
| 01-plain-paragraph | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5851/1938816 px, max |Δ| 255 | - |
| 01-plain-paragraph | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6053/1938816 px, max |Δ| 255 | - |
| 01-plain-paragraph | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5030/1938816 px, max |Δ| 255 | - |
| 02-wrapping-paragraph | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 118539/1938816 px, max |Δ| 255 | - |
| 02-wrapping-paragraph | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 115714/1938816 px, max |Δ| 255 | - |
| 02-wrapping-paragraph | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 104818/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 02-wrapping-paragraph | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 27814/1938816 px, max |Δ| 6 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 02-wrapping-paragraph | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 119257/1938816 px, max |Δ| 255 | - |
| 02-wrapping-paragraph | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 115964/1938816 px, max |Δ| 255 | - |
| 02-wrapping-paragraph | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 107112/1938816 px, max |Δ| 255 | - |
| 02-wrapping-paragraph | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 86660/1938816 px, max |Δ| 255 | - |
| 03-section-heading | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 16452/1938816 px, max |Δ| 255 | - |
| 03-section-heading | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 15822/1938816 px, max |Δ| 255 | - |
| 03-section-heading | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 13227/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 03-section-heading | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 2047/1938816 px, max |Δ| 5 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 03-section-heading | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 16646/1938816 px, max |Δ| 255 | - |
| 03-section-heading | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 15920/1938816 px, max |Δ| 255 | - |
| 03-section-heading | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 13560/1938816 px, max |Δ| 255 | - |
| 03-section-heading | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 12099/1938816 px, max |Δ| 255 | - |
| 04-bold-emph | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6094/1938816 px, max |Δ| 255 | - |
| 04-bold-emph | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6441/1938816 px, max |Δ| 255 | - |
| 04-bold-emph | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6748/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 04-bold-emph | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 1831/1938816 px, max |Δ| 4 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 04-bold-emph | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6089/1938816 px, max |Δ| 255 | - |
| 04-bold-emph | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6461/1938816 px, max |Δ| 255 | - |
| 04-bold-emph | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6445/1938816 px, max |Δ| 255 | - |
| 04-bold-emph | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5951/1938816 px, max |Δ| 255 | - |
| 05-unicode | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5321/1938816 px, max |Δ| 255 | - |
| 05-unicode | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5663/1938816 px, max |Δ| 255 | - |
| 05-unicode | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5687/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 05-unicode | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 1262/1938816 px, max |Δ| 4 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 05-unicode | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4870/1938816 px, max |Δ| 255 | - |
| 05-unicode | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5642/1938816 px, max |Δ| 255 | - |
| 05-unicode | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5585/1938816 px, max |Δ| 255 | - |
| 05-unicode | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4766/1938816 px, max |Δ| 255 | - |
| 06-math-inline | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5596/1938816 px, max |Δ| 255 | - |
| 06-math-inline | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5578/1938816 px, max |Δ| 255 | - |
| 06-math-inline | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4896/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 06-math-inline | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 1556/1938816 px, max |Δ| 114 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 06-math-inline | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5578/1938816 px, max |Δ| 255 | - |
| 06-math-inline | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5620/1938816 px, max |Δ| 255 | - |
| 06-math-inline | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4809/1938816 px, max |Δ| 255 | - |
| 06-math-inline | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4117/1938816 px, max |Δ| 255 | - |
| 07-math-display | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5022/1938816 px, max |Δ| 255 | - |
| 07-math-display | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5681/1938816 px, max |Δ| 255 | - |
| 07-math-display | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4839/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 07-math-display | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 2127/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 07-math-display | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5059/1938816 px, max |Δ| 255 | - |
| 07-math-display | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5720/1938816 px, max |Δ| 255 | - |
| 07-math-display | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4718/1938816 px, max |Δ| 255 | - |
| 07-math-display | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 4484/1938816 px, max |Δ| 255 | - |
| 08-two-page | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 444978/1938816 px, max |Δ| 255 | - |
| 08-two-page | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 408459/1938816 px, max |Δ| 255 | - |
| 08-two-page | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 389069/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 08-two-page | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 96274/1938816 px, max |Δ| 6 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 08-two-page | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 462968/1938816 px, max |Δ| 255 | - |
| 08-two-page | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 426669/1938816 px, max |Δ| 255 | - |
| 08-two-page | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 398372/1938816 px, max |Δ| 255 | - |
| 08-two-page | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 305075/1938816 px, max |Δ| 255 | - |
| 09-mixed-document | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 33705/1938816 px, max |Δ| 255 | - |
| 09-mixed-document | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 32465/1938816 px, max |Δ| 255 | - |
| 09-mixed-document | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 26324/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 09-mixed-document | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 8668/1938816 px, max |Δ| 145 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 09-mixed-document | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 33859/1938816 px, max |Δ| 255 | - |
| 09-mixed-document | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 32558/1938816 px, max |Δ| 255 | - |
| 09-mixed-document | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 27351/1938816 px, max |Δ| 255 | - |
| 09-mixed-document | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 21569/1938816 px, max |Δ| 255 | - |
| 10-unicode-paragraph | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 50878/1938816 px, max |Δ| 255 | - |
| 10-unicode-paragraph | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 50399/1938816 px, max |Δ| 255 | - |
| 10-unicode-paragraph | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 50064/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 10-unicode-paragraph | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 14033/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 10-unicode-paragraph | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 50533/1938816 px, max |Δ| 255 | - |
| 10-unicode-paragraph | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 50446/1938816 px, max |Δ| 255 | - |
| 10-unicode-paragraph | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 51058/1938816 px, max |Δ| 255 | - |
| 10-unicode-paragraph | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 42803/1938816 px, max |Δ| 255 | - |
| 11-nested-lists | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 20125/1938816 px, max |Δ| 255 | - |
| 11-nested-lists | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19495/1938816 px, max |Δ| 255 | - |
| 11-nested-lists | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19422/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 11-nested-lists | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 17589/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 11-nested-lists | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 20270/1938816 px, max |Δ| 255 | - |
| 11-nested-lists | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19733/1938816 px, max |Δ| 255 | - |
| 11-nested-lists | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19977/1938816 px, max |Δ| 255 | - |
| 11-nested-lists | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19404/1938816 px, max |Δ| 255 | - |
| 12-justified-paragraphs | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 214743/1938816 px, max |Δ| 255 | - |
| 12-justified-paragraphs | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 203863/1938816 px, max |Δ| 255 | - |
| 12-justified-paragraphs | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 185355/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 12-justified-paragraphs | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 45570/1938816 px, max |Δ| 5 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 12-justified-paragraphs | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 209291/1938816 px, max |Δ| 255 | - |
| 12-justified-paragraphs | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 204394/1938816 px, max |Δ| 255 | - |
| 12-justified-paragraphs | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 191189/1938816 px, max |Δ| 255 | - |
| 12-justified-paragraphs | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 148653/1938816 px, max |Δ| 255 | - |
| 13-math-display-rich | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 7998/1938816 px, max |Δ| 255 | - |
| 13-math-display-rich | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 8341/1938816 px, max |Δ| 255 | - |
| 13-math-display-rich | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 8453/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 13-math-display-rich | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 5831/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 13-math-display-rich | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 8006/1938816 px, max |Δ| 255 | - |
| 13-math-display-rich | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 8452/1938816 px, max |Δ| 255 | - |
| 13-math-display-rich | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 8281/1938816 px, max |Δ| 255 | - |
| 13-math-display-rich | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 8015/1938816 px, max |Δ| 255 | - |
| 14-math-inline-dense | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19206/1938816 px, max |Δ| 255 | - |
| 14-math-inline-dense | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 18723/1938816 px, max |Δ| 255 | - |
| 14-math-inline-dense | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 15860/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 14-math-inline-dense | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 9959/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity |
| 14-math-inline-dense | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 19110/1938816 px, max |Δ| 255 | - |
| 14-math-inline-dense | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 18691/1938816 px, max |Δ| 255 | - |
| 14-math-inline-dense | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 15776/1938816 px, max |Δ| 255 | - |
| 14-math-inline-dense | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 14024/1938816 px, max |Δ| 255 | - |
| 15-three-page-sections | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 378473/1938816 px, max |Δ| 255 | - |
| 15-three-page-sections | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 365118/1938816 px, max |Δ| 255 | - |
| 15-three-page-sections | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 309522/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 15-three-page-sections | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 80681/1938816 px, max |Δ| 6 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 15-three-page-sections | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 379291/1938816 px, max |Δ| 255 | - |
| 15-three-page-sections | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 365671/1938816 px, max |Δ| 255 | - |
| 15-three-page-sections | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 318906/1938816 px, max |Δ| 255 | - |
| 15-three-page-sections | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 249961/1938816 px, max |Δ| 255 | - |
| 16-heading-page-break | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 444978/1938816 px, max |Δ| 255 | - |
| 16-heading-page-break | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 408459/1938816 px, max |Δ| 255 | - |
| 16-heading-page-break | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 389069/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 16-heading-page-break | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 96274/1938816 px, max |Δ| 6 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 16-heading-page-break | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 462968/1938816 px, max |Δ| 255 | - |
| 16-heading-page-break | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 426669/1938816 px, max |Δ| 255 | - |
| 16-heading-page-break | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 398372/1938816 px, max |Δ| 255 | - |
| 16-heading-page-break | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 305075/1938816 px, max |Δ| 255 | - |
| 17-apostrophes | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 12870/1938816 px, max |Δ| 255 | - |
| 17-apostrophes | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 13649/1938816 px, max |Δ| 255 | - |
| 17-apostrophes | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 13723/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 17-apostrophes | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 3667/1938816 px, max |Δ| 5 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 17-apostrophes | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 12981/1938816 px, max |Δ| 255 | - |
| 17-apostrophes | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 13722/1938816 px, max |Δ| 255 | - |
| 17-apostrophes | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 14024/1938816 px, max |Δ| 255 | - |
| 17-apostrophes | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 11839/1938816 px, max |Δ| 255 | - |
| 18-ligatures | de1020c | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 20371/1938816 px, max |Δ| 255 | - |
| 18-ligatures | de1020c | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 20720/1938816 px, max |Δ| 255 | - |
| 18-ligatures | exact (exact route) | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 20599/1938816 px, max |Δ| 255 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 18-ligatures | exact (exact route) | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 6065/1938816 px, max |Δ| 5 | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity |
| 18-ligatures | main | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 20691/1938816 px, max |Δ| 255 | - |
| 18-ligatures | main | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 21485/1938816 px, max |Δ| 255 | - |
| 18-ligatures | pipeline | pdflatex | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 21013/1938816 px, max |Δ| 255 | - |
| 18-ligatures | pipeline | pdflatex-lm | DIFFERENT | DIFFERENT | **EQUAL** | DIFFERENT: 17973/1938816 px, max |Δ| 255 | - |

### Exact route per fixture (candidate `exact`: flashtex-render --v2 → flashtex-pdf-exact from-v2)

Categories come from `flashtex-pdf-exact classify <pdflatex-lm reference> <ours>` (ContentOperators / FontProgram / FontMetadata / ObjectLayout / Compression / DocumentIdentity / page geometry); an empty list would mean byte-identical content. Zero-pixel and byte columns above are the acceptance signals; the categories explain the difference, never excuse it.

| Fixture | from-v2 summary | classify categories | deviations reported by the tool | zero-pixel vs pdflatex-lm | differing px |
|---|---|---|---|---|---|
| 01-plain-paragraph | 1 page(s), 13 glyph run(s), 55 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 9595 bytes | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 1754/1938816 px, max |Δ| 4 | p1 1754/1938816 max 4 |
| 02-wrapping-paragraph | 1 page(s), 210 glyph run(s), 896 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 76903 bytes | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 27814/1938816 px, max |Δ| 6 | p1 27814/1938816 max 6 |
| 03-section-heading | 1 page(s), 15 glyph run(s), 79 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 14765 bytes | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 2047/1938816 px, max |Δ| 5 | p1 2047/1938816 max 5 |
| 04-bold-emph | 1 page(s), 12 glyph run(s), 54 glyph(s) (3 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 17232 bytes | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 1831/1938816 px, max |Δ| 4 | p1 1831/1938816 max 4 |
| 05-unicode | 1 page(s), 11 glyph run(s), 46 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 8859 bytes | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 1262/1938816 px, max |Δ| 4 | p1 1262/1938816 max 4 |
| 06-math-inline | 1 page(s), 13 glyph run(s), 45 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 2 rule(s), 11694 bytes | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 1556/1938816 px, max |Δ| 114 | p1 1556/1938816 max 114 |
| 07-math-display | 1 page(s), 10 glyph run(s), 47 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 1 rule(s), 11536 bytes | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 2127/1938816 px, max |Δ| 255 | p1 2127/1938816 max 255 |
| 08-two-page | 3 page(s), 1800 glyph run(s), 7680 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 608568 bytes | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 96274/1938816 px, max |Δ| 6 | p1 96274/1938816 max 6; p2 96135/1938816 max 6; p3 42817/1938816 max 6 |
| 09-mixed-document | 1 page(s), 50 glyph run(s), 206 glyph(s) (2 continued at the natural advance, 0 with an exact TJ kern), 3 rule(s), 32849 bytes | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 8668/1938816 px, max |Δ| 145 | p1 8668/1938816 max 145 |
| 10-unicode-paragraph | 1 page(s), 81 glyph run(s), 425 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 47876 bytes | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 14033/1938816 px, max |Δ| 255 | p1 14033/1938816 max 255 |
| 11-nested-lists | 1 page(s), 29 glyph run(s), 147 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 17444 bytes | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 17589/1938816 px, max |Δ| 255 | p1 17589/1938816 max 255 |
| 12-justified-paragraphs | 1 page(s), 360 glyph run(s), 1536 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 126682 bytes | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 45570/1938816 px, max |Δ| 5 | p1 45570/1938816 max 5 |
| 13-math-display-rich | 1 page(s), 17 glyph run(s), 72 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 3 rule(s), 17022 bytes | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 5831/1938816 px, max |Δ| 255 | p1 5831/1938816 max 255 |
| 14-math-inline-dense | 1 page(s), 71 glyph run(s), 147 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 4 rule(s), 27588 bytes | ContentOperators; FontProgram; FontMetadata; FontResources; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 9959/1938816 px, max |Δ| 255 | p1 9959/1938816 max 255 |
| 15-three-page-sections | 3 page(s), 1806 glyph run(s), 7695 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 611989 bytes | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 80681/1938816 px, max |Δ| 6 | p1 80681/1938816 max 6; p2 80717/1938816 max 6; p3 80723/1938816 max 6 |
| 16-heading-page-break | 2 page(s), 962 glyph run(s), 4109 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 332089 bytes | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 96274/1938816 px, max |Δ| 6 | p1 96274/1938816 max 6; p2 29402/1938816 max 5 |
| 17-apostrophes | 1 page(s), 25 glyph run(s), 132 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 17238 bytes | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 3667/1938816 px, max |Δ| 5 | p1 3667/1938816 max 5 |
| 18-ligatures | 1 page(s), 36 glyph run(s), 170 glyph(s) (0 continued at the natural advance, 0 with an exact TJ kern), 0 rule(s), 19958 bytes | ContentOperators; FontProgram; FontMetadata; ObjectLayout; Compression; DocumentIdentity | none | DIFFERENT: 6065/1938816 px, max |Δ| 5 | p1 6065/1938816 max 5 |

## Gate 1 — FlashTeX self-regression (byte identity with its own pinned prior output)

Pinned in `reference-profile.json`: the SHA-256 of FlashTeX's own `flashtex.pdf` (writer + compiler at pin time). EQUAL means the candidate reproduces the earlier FlashTeX output byte for byte; it is a reproducibility/regression signal only and says nothing about LaTeX.

| Fixture | Compiler | candidate SHA-256 | pinned prior FlashTeX SHA-256 | self-regression |
|---|---|---|---|---|
| 01-plain-paragraph | de1020c | `88deda9f24394700…` | `88deda9f24394700…` | **EQUAL** (reproduces prior FlashTeX output) |
| 01-plain-paragraph | exact | `fdaf6f847e78fa6d…` | `-` | unpinned |
| 01-plain-paragraph | main | `f51a106475288081…` | `982579044f6e075e…` | DIFFERENT from prior FlashTeX output |
| 01-plain-paragraph | pipeline | `89cd110c6261fa5b…` | `-` | unpinned |
| 02-wrapping-paragraph | de1020c | `1bc0994be86d3ece…` | `1bc0994be86d3ece…` | **EQUAL** (reproduces prior FlashTeX output) |
| 02-wrapping-paragraph | exact | `60b05c16de97fcae…` | `-` | unpinned |
| 02-wrapping-paragraph | main | `fe7f57bbb3d8591b…` | `a648c982d1829fad…` | DIFFERENT from prior FlashTeX output |
| 02-wrapping-paragraph | pipeline | `3b12e424dcef422e…` | `-` | unpinned |
| 03-section-heading | de1020c | `7f0f7247434d3132…` | `7f0f7247434d3132…` | **EQUAL** (reproduces prior FlashTeX output) |
| 03-section-heading | exact | `9a4136ab3bde5a3d…` | `-` | unpinned |
| 03-section-heading | main | `8a7152d70e730c8f…` | `456aedda047b3d9f…` | DIFFERENT from prior FlashTeX output |
| 03-section-heading | pipeline | `f9c19038d87b5f9a…` | `-` | unpinned |
| 04-bold-emph | de1020c | `7a34b0a51a2992f4…` | `7a34b0a51a2992f4…` | **EQUAL** (reproduces prior FlashTeX output) |
| 04-bold-emph | exact | `3f3b4fc4ab9c86de…` | `-` | unpinned |
| 04-bold-emph | main | `bcb439c70e286a0f…` | `483817723c7a07a7…` | DIFFERENT from prior FlashTeX output |
| 04-bold-emph | pipeline | `b772ca7ed6aa5a39…` | `-` | unpinned |
| 05-unicode | de1020c | `e721656e20b80d47…` | `e721656e20b80d47…` | **EQUAL** (reproduces prior FlashTeX output) |
| 05-unicode | exact | `382bf05a9ca01f15…` | `-` | unpinned |
| 05-unicode | main | `6241e0ee71a8dfb6…` | `4a9c9cd453fdc68b…` | DIFFERENT from prior FlashTeX output |
| 05-unicode | pipeline | `b27d24cd3ece49d8…` | `-` | unpinned |
| 06-math-inline | de1020c | `0a1a2a6eebc4b2b1…` | `0a1a2a6eebc4b2b1…` | **EQUAL** (reproduces prior FlashTeX output) |
| 06-math-inline | exact | `5983a7f211fdc99d…` | `-` | unpinned |
| 06-math-inline | main | `1d8ba2668e230825…` | `055d64e818f4342a…` | DIFFERENT from prior FlashTeX output |
| 06-math-inline | pipeline | `9723161fa588fe0f…` | `-` | unpinned |
| 07-math-display | de1020c | `915882632c74f40b…` | `915882632c74f40b…` | **EQUAL** (reproduces prior FlashTeX output) |
| 07-math-display | exact | `88e82f56bbcbcced…` | `-` | unpinned |
| 07-math-display | main | `2dca277385195a4f…` | `ada42128ad2cb2f6…` | DIFFERENT from prior FlashTeX output |
| 07-math-display | pipeline | `3576f2b79905fc1c…` | `-` | unpinned |
| 08-two-page | de1020c | `6e8c29a5f28f5429…` | `6e8c29a5f28f5429…` | **EQUAL** (reproduces prior FlashTeX output) |
| 08-two-page | exact | `c02d9fd35035bc50…` | `-` | unpinned |
| 08-two-page | main | `f07b9750acdf8421…` | `44e782b1fe089026…` | DIFFERENT from prior FlashTeX output |
| 08-two-page | pipeline | `e3b1119d08723003…` | `-` | unpinned |
| 09-mixed-document | de1020c | `eb9ed31b44ff065e…` | `eb9ed31b44ff065e…` | **EQUAL** (reproduces prior FlashTeX output) |
| 09-mixed-document | exact | `1ffaba9b0b3c6813…` | `-` | unpinned |
| 09-mixed-document | main | `87b92a2ba0720b4a…` | `ed1c2d77c3510fb2…` | DIFFERENT from prior FlashTeX output |
| 09-mixed-document | pipeline | `fb163deb0a4f4003…` | `-` | unpinned |
| 10-unicode-paragraph | de1020c | `405e9d22c4c242b1…` | `-` | unpinned |
| 10-unicode-paragraph | exact | `4d33a4138656a6a7…` | `-` | unpinned |
| 10-unicode-paragraph | main | `501aa2a833350637…` | `-` | unpinned |
| 10-unicode-paragraph | pipeline | `e2f85ed5e31586c5…` | `-` | unpinned |
| 11-nested-lists | de1020c | `dddb38b9e9050f15…` | `-` | unpinned |
| 11-nested-lists | exact | `2a90ef836f5406fd…` | `-` | unpinned |
| 11-nested-lists | main | `5a39365af707eabf…` | `-` | unpinned |
| 11-nested-lists | pipeline | `7cd8e65e610bfba9…` | `-` | unpinned |
| 12-justified-paragraphs | de1020c | `47e5eefae9a4319a…` | `-` | unpinned |
| 12-justified-paragraphs | exact | `bed378f8c7ed1487…` | `-` | unpinned |
| 12-justified-paragraphs | main | `a608f87667eba822…` | `-` | unpinned |
| 12-justified-paragraphs | pipeline | `75780032af9cf272…` | `-` | unpinned |
| 13-math-display-rich | de1020c | `80f03c44b40cd1b1…` | `-` | unpinned |
| 13-math-display-rich | exact | `fdc0e39f1586e39a…` | `-` | unpinned |
| 13-math-display-rich | main | `2513d596dd1a032e…` | `-` | unpinned |
| 13-math-display-rich | pipeline | `05e59d706d52d954…` | `-` | unpinned |
| 14-math-inline-dense | de1020c | `04244b7cb4eedb5c…` | `-` | unpinned |
| 14-math-inline-dense | exact | `fbdd8a87ef22b585…` | `-` | unpinned |
| 14-math-inline-dense | main | `9538e1767e50a0c1…` | `-` | unpinned |
| 14-math-inline-dense | pipeline | `a1dd10eb08a84f5a…` | `-` | unpinned |
| 15-three-page-sections | de1020c | `9f2923c8d7e3dba9…` | `-` | unpinned |
| 15-three-page-sections | exact | `197f680211c052c1…` | `-` | unpinned |
| 15-three-page-sections | main | `c518936b68c53d5f…` | `-` | unpinned |
| 15-three-page-sections | pipeline | `6dd7e8ffd642c9c6…` | `-` | unpinned |
| 16-heading-page-break | de1020c | `a1b742b01edb41a0…` | `-` | unpinned |
| 16-heading-page-break | exact | `761b6510bdc838f5…` | `-` | unpinned |
| 16-heading-page-break | main | `e239f36d95b5fd69…` | `-` | unpinned |
| 16-heading-page-break | pipeline | `eccc9b532bd0d302…` | `-` | unpinned |
| 17-apostrophes | de1020c | `aed18719902a6924…` | `-` | unpinned |
| 17-apostrophes | exact | `923299a08825fecf…` | `-` | unpinned |
| 17-apostrophes | main | `95e3fc1ba4ffb981…` | `-` | unpinned |
| 17-apostrophes | pipeline | `cf9492a2f48abc1c…` | `-` | unpinned |
| 18-ligatures | de1020c | `cccda2e7ef9bd851…` | `-` | unpinned |
| 18-ligatures | exact | `a6bcd0e2aabef5a6…` | `-` | unpinned |
| 18-ligatures | main | `5029f2456cd0ea4f…` | `-` | unpinned |
| 18-ligatures | pipeline | `08a360a3d37d7b7a…` | `-` | unpinned |

## Gate 3 — native/export parity (acceptance)

| Fixture | Compiler | export = preview-equivalent | export = native preview capture |
|---|---|---|---|
| 01-plain-paragraph | de1020c | DIFFERENT: 1240/1938816 px, max |Δ| 255 | unavailable |
| 01-plain-paragraph | exact | DIFFERENT: 5001/1938816 px, max |Δ| 255 | unavailable |
| 01-plain-paragraph | main | DIFFERENT: 1331/1938816 px, max |Δ| 255 | unavailable |
| 01-plain-paragraph | pipeline | DIFFERENT: 1246/1938816 px, max |Δ| 255 | unavailable |
| 02-wrapping-paragraph | de1020c | DIFFERENT: 17142/1938816 px, max |Δ| 255 | unavailable |
| 02-wrapping-paragraph | exact | DIFFERENT: 86718/1938816 px, max |Δ| 255 | unavailable |
| 02-wrapping-paragraph | main | DIFFERENT: 17429/1938816 px, max |Δ| 255 | unavailable |
| 02-wrapping-paragraph | pipeline | DIFFERENT: 15655/1938816 px, max |Δ| 255 | unavailable |
| 03-section-heading | de1020c | DIFFERENT: 2140/1938816 px, max |Δ| 6 | unavailable |
| 03-section-heading | exact | DIFFERENT: 11433/1938816 px, max |Δ| 255 | unavailable |
| 03-section-heading | main | DIFFERENT: 1969/1938816 px, max |Δ| 6 | unavailable |
| 03-section-heading | pipeline | DIFFERENT: 1834/1938816 px, max |Δ| 6 | unavailable |
| 04-bold-emph | de1020c | DIFFERENT: 1261/1938816 px, max |Δ| 4 | unavailable |
| 04-bold-emph | exact | DIFFERENT: 5945/1938816 px, max |Δ| 255 | unavailable |
| 04-bold-emph | main | DIFFERENT: 1228/1938816 px, max |Δ| 4 | unavailable |
| 04-bold-emph | pipeline | DIFFERENT: 1554/1938816 px, max |Δ| 4 | unavailable |
| 05-unicode | de1020c | DIFFERENT: 587/1938816 px, max |Δ| 3 | unavailable |
| 05-unicode | exact | DIFFERENT: 4774/1938816 px, max |Δ| 255 | unavailable |
| 05-unicode | main | DIFFERENT: 441/1938816 px, max |Δ| 2 | unavailable |
| 05-unicode | pipeline | DIFFERENT: 543/1938816 px, max |Δ| 2 | unavailable |
| 06-math-inline | de1020c | DIFFERENT: 1041/1938816 px, max |Δ| 254 | unavailable |
| 06-math-inline | exact | DIFFERENT: 4160/1938816 px, max |Δ| 255 | unavailable |
| 06-math-inline | main | DIFFERENT: 1113/1938816 px, max |Δ| 254 | unavailable |
| 06-math-inline | pipeline | DIFFERENT: 1024/1938816 px, max |Δ| 254 | unavailable |
| 07-math-display | de1020c | DIFFERENT: 1021/1938816 px, max |Δ| 255 | unavailable |
| 07-math-display | exact | DIFFERENT: 4455/1938816 px, max |Δ| 255 | unavailable |
| 07-math-display | main | DIFFERENT: 927/1938816 px, max |Δ| 255 | unavailable |
| 07-math-display | pipeline | DIFFERENT: 775/1938816 px, max |Δ| 255 | unavailable |
| 08-two-page | de1020c | DIFFERENT: 58237/1938816 px, max |Δ| 255 | unavailable |
| 08-two-page | exact | DIFFERENT: 305324/1938816 px, max |Δ| 255 | unavailable |
| 08-two-page | main | DIFFERENT: 64246/1938816 px, max |Δ| 255 | unavailable |
| 08-two-page | pipeline | DIFFERENT: 53969/1938816 px, max |Δ| 255 | unavailable |
| 09-mixed-document | de1020c | DIFFERENT: 4608/1938816 px, max |Δ| 250 | unavailable |
| 09-mixed-document | exact | DIFFERENT: 21376/1938816 px, max |Δ| 255 | unavailable |
| 09-mixed-document | main | DIFFERENT: 5035/1938816 px, max |Δ| 246 | unavailable |
| 09-mixed-document | pipeline | DIFFERENT: 4324/1938816 px, max |Δ| 252 | unavailable |
| 10-unicode-paragraph | de1020c | DIFFERENT: 9269/1938816 px, max |Δ| 255 | unavailable |
| 10-unicode-paragraph | exact | DIFFERENT: 42232/1938816 px, max |Δ| 255 | unavailable |
| 10-unicode-paragraph | main | DIFFERENT: 9257/1938816 px, max |Δ| 255 | unavailable |
| 10-unicode-paragraph | pipeline | DIFFERENT: 8847/1938816 px, max |Δ| 255 | unavailable |
| 11-nested-lists | de1020c | DIFFERENT: 2764/1938816 px, max |Δ| 6 | unavailable |
| 11-nested-lists | exact | DIFFERENT: 14212/1938816 px, max |Δ| 255 | unavailable |
| 11-nested-lists | main | DIFFERENT: 2620/1938816 px, max |Δ| 6 | unavailable |
| 11-nested-lists | pipeline | DIFFERENT: 2877/1938816 px, max |Δ| 6 | unavailable |
| 12-justified-paragraphs | de1020c | DIFFERENT: 30049/1938816 px, max |Δ| 255 | unavailable |
| 12-justified-paragraphs | exact | DIFFERENT: 148714/1938816 px, max |Δ| 255 | unavailable |
| 12-justified-paragraphs | main | DIFFERENT: 30072/1938816 px, max |Δ| 255 | unavailable |
| 12-justified-paragraphs | pipeline | DIFFERENT: 26659/1938816 px, max |Δ| 255 | unavailable |
| 13-math-display-rich | de1020c | DIFFERENT: 1699/1938816 px, max |Δ| 255 | unavailable |
| 13-math-display-rich | exact | DIFFERENT: 6985/1938816 px, max |Δ| 255 | unavailable |
| 13-math-display-rich | main | DIFFERENT: 1359/1938816 px, max |Δ| 255 | unavailable |
| 13-math-display-rich | pipeline | DIFFERENT: 1361/1938816 px, max |Δ| 255 | unavailable |
| 14-math-inline-dense | de1020c | DIFFERENT: 3142/1938816 px, max |Δ| 255 | unavailable |
| 14-math-inline-dense | exact | DIFFERENT: 12693/1938816 px, max |Δ| 255 | unavailable |
| 14-math-inline-dense | main | DIFFERENT: 2835/1938816 px, max |Δ| 255 | unavailable |
| 14-math-inline-dense | pipeline | DIFFERENT: 2956/1938816 px, max |Δ| 255 | unavailable |
| 15-three-page-sections | de1020c | DIFFERENT: 49094/1938816 px, max |Δ| 255 | unavailable |
| 15-three-page-sections | exact | DIFFERENT: 249651/1938816 px, max |Δ| 255 | unavailable |
| 15-three-page-sections | main | DIFFERENT: 50857/1938816 px, max |Δ| 255 | unavailable |
| 15-three-page-sections | pipeline | DIFFERENT: 44781/1938816 px, max |Δ| 255 | unavailable |
| 16-heading-page-break | de1020c | DIFFERENT: 58237/1938816 px, max |Δ| 255 | unavailable |
| 16-heading-page-break | exact | DIFFERENT: 305324/1938816 px, max |Δ| 255 | unavailable |
| 16-heading-page-break | main | DIFFERENT: 64246/1938816 px, max |Δ| 255 | unavailable |
| 16-heading-page-break | pipeline | DIFFERENT: 53969/1938816 px, max |Δ| 255 | unavailable |
| 17-apostrophes | de1020c | DIFFERENT: 1919/1938816 px, max |Δ| 4 | unavailable |
| 17-apostrophes | exact | DIFFERENT: 11872/1938816 px, max |Δ| 255 | unavailable |
| 17-apostrophes | main | DIFFERENT: 1436/1938816 px, max |Δ| 3 | unavailable |
| 17-apostrophes | pipeline | DIFFERENT: 2114/1938816 px, max |Δ| 255 | unavailable |
| 18-ligatures | de1020c | DIFFERENT: 9644/1938816 px, max |Δ| 255 | unavailable |
| 18-ligatures | exact | DIFFERENT: 16894/1938816 px, max |Δ| 255 | unavailable |
| 18-ligatures | main | DIFFERENT: 10092/1938816 px, max |Δ| 255 | unavailable |
| 18-ligatures | pipeline | DIFFERENT: 10024/1938816 px, max |Δ| 255 | unavailable |

- Classification of native-preview differences: the native capture comes from a screen capture at the display's backing scale, resampled to the raster size, so a DIFFERENT result there is expected to be dominated by resampling and text rasterization (CoreText on screen vs CoreGraphics PDF rendering); it is reported as-is, without normalisation. Preview-equivalent vs export differences isolate the drawing path (CoreText glyph run vs the PDF writer's text operators) from any capture effects. 'unavailable' means no capture was made this run.

## Provenance

- suite_branch: `mvo/rev2`
- suite_sha: `959467f9b43879d55167ca2f0f52524769ed5261`
- input_main_sha: `02de1162386ba3278e149025dd1d2e623911de0a`
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

- compiler `main`: `origin/main` @ `c35ca8bc4e3801fbbe308ee731d718af7ebe78eb` (crates/compiler) — Merge remote-tracking branch 'origin/main' into agent/orchestrator-astra/drained-products
- compiler `de1020c`: `de1020c` @ `de1020cd0be7cede11be2691e00e7f5b15cb2224` (crates/compiler) — compiler: add math, real font metrics and PDF output
- compiler `pipeline`: `origin/agent/mac-render-pipeline/unified` @ `65dbe7da7a182e99322070e2c9763cc3b69a342b` (crates/render-pipeline) — render-pipeline: shaping cache outlives requests (bounded)
- **exact route** `exact`: `flashtex-render --secnumdepth 0 --v2` from `origin/agent/mac-render-pipeline/unified` @ `65dbe7da7a182e99322070e2c9763cc3b69a342b` (crates/render-pipeline) → `flashtex-pdf-exact from-v2` from `origin/agent/mac-pdf/v2-adapter` @ `8789abf2f82624684b6615e5014240ad25b26c95` (crates/pdf) — pdf docs + coord: before/after for Subr pruning and exact TJ kerning in the v2-adapter gap report. Glyphs by original GID at the producer's exact tick origins, GID-preserving CFF subsets of the Latin Modern OTFs resolved by content hash. from-v2 reported no deviation on any fixture this run (the producer's font `sha256` matched SHA-256(bytes); at earlier producer revisions the tool reported the SHA-256(bytes ‖ face_index) deviation on every fixture).
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
| 01-plain-paragraph | lualatex | exact | 1/1 | ok | 0.3675 | 0.9942 | (0,0) | 0.3675 | 0.9942 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 12.21 | 0.77 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.2612 | 0.9965 | (-0.5,0) | 0.1503 | 0.9983 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.57 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex | pipeline | 1/1 | ok | 0.3850 | 0.9943 | (0.5,0) weak | 0.3813 | 0.9944 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 12.21 | 0.41 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | de1020c | 1/1 | ok | 0.3662 | 0.9941 | (0,0) | 0.3662 | 0.9941 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.30 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | exact | 1/1 | ok | 0.0045 | 1.0000 | (0,0) | 0.0045 | 1.0000 | 16 | 0.0011 | 0.0000 | 13/13/13 | yes | 0.01 | 0.00 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | main | 1/1 | ok | 0.3601 | 0.9942 | (0,0) | 0.3601 | 0.9942 | 255 | 0.0030 | 0.0024 | 13/13/13 | yes | 12.75 | 0.30 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | pipeline | 1/1 | ok | 0.2392 | 0.9966 | (0,0) | 0.2392 | 0.9966 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.1873 | 0.9975 | (0,0) | 0.1873 | 0.9975 | 255 | 0.0024 | 0.0014 | 13/13/13 | yes | 0.43 | 0.46 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | exact | 1/1 | ok | 0.3691 | 0.9943 | (0,0) | 0.3691 | 0.9943 | 255 | 0.0031 | 0.0025 | 13/13/13 | yes | 12.72 | 0.77 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-exact-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.1285 | 0.9985 | (0,0) | 0.1285 | 0.9985 | 255 | 0.0022 | 0.0011 | 13/13/13 | yes | 0.22 | 0.46 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-main-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex | pipeline | 1/1 | ok | 0.3927 | 0.9944 | (1,0) weak | 0.3792 | 0.9947 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 12.72 | 0.41 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-pipeline-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 0.3660 | 0.9942 | (0,0) | 0.3660 | 0.9942 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | exact | 1/1 | ok | 0.0015 | 1.0000 | (0,0) | 0.0015 | 1.0000 | 4 | 0.0009 | 0.0000 | 13/13/13 | yes | 0.00 | 1.03 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-exact-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | main | 1/1 | ok | 0.3597 | 0.9943 | (0,0) | 0.3597 | 0.9943 | 255 | 0.0030 | 0.0024 | 13/13/13 | yes | 12.76 | 0.73 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 01-plain-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 0.2405 | 0.9966 | (0,0) | 0.2405 | 0.9966 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.00 | 0.67 | 1.0000 | 0/0 | [p1](images/01-plain-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.1607 | 0.9980 | (0,0) | 0.1607 | 0.9980 | 255 | 0.0023 | 0.0014 | 13/13/13 | yes | 0.30 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex | exact | 1/1 | ok | 0.3677 | 0.9942 | (0,0) | 0.3677 | 0.9942 | 255 | 0.0030 | 0.0024 | 13/13/13 | yes | 12.19 | 0.77 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.2670 | 0.9964 | (-0.5,0) | 0.1528 | 0.9982 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.58 | 0.46 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex | pipeline | 1/1 | ok | 0.3853 | 0.9943 | (0.5,0) weak | 0.3814 | 0.9943 | 255 | 0.0031 | 0.0024 | 13/13/13 | yes | 12.19 | 0.41 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | de1020c | 1/1 | ok | 0.3665 | 0.9941 | (0,0) | 0.3665 | 0.9941 | 255 | 0.0030 | 0.0025 | 13/13/13 | yes | 12.45 | 0.73 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | exact | 1/1 | ok | 0.0049 | 1.0000 | (0,0) | 0.0049 | 1.0000 | 14 | 0.0012 | 0.0000 | 13/13/13 | yes | 0.01 | 1.03 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | main | 1/1 | ok | 0.3603 | 0.9942 | (0,0) | 0.3603 | 0.9942 | 255 | 0.0030 | 0.0024 | 13/13/13 | yes | 12.75 | 0.73 | 1.0000 | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | pipeline | 1/1 | ok | 0.2392 | 0.9966 | (0,0) | 0.2392 | 0.9966 | 255 | 0.0026 | 0.0019 | 13/13/13 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 8.3842 | 0.8656 | (-3,0) weak | 8.3713 | 0.8657 | 255 | 0.0613 | 0.0509 | 210/210/210 | yes | 116.20 | 5.12 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | exact | 1/1 | ok | 6.7860 | 0.8871 | (0,0) | 6.7860 | 0.8871 | 255 | 0.0543 | 0.0442 | 210/210/210 | yes | 169.48 | 4.62 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 8.4139 | 0.8657 | (0,0) | 8.4139 | 0.8657 | 255 | 0.0616 | 0.0511 | 210/210/210 | yes | 90.99 | 4.50 | 0.9048 | 0/0 | - |
| 02-wrapping-paragraph | lualatex | pipeline | 1/1 | ok | 7.1755 | 0.8901 | (0,0) | 7.1755 | 0.8901 | 255 | 0.0554 | 0.0449 | 210/210/210 | yes | 169.48 | 4.26 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | de1020c | 1/1 | ok | 7.7868 | 0.8620 | (-0.5,-14.5) weak | 7.7691 | 0.8602 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.57 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | exact | 1/1 | ok | 0.0777 | 1.0000 | (0,0) | 0.0777 | 1.0000 | 13 | 0.0218 | 0.0000 | 210/210/210 | yes | 0.01 | 0.00 | 1.0000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | main | 1/1 | ok | 7.7923 | 0.8614 | (0,0) | 7.7923 | 0.8614 | 255 | 0.0598 | 0.0495 | 210/210/210 | yes | 117.54 | 3.99 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | pipeline | 1/1 | ok | 4.5437 | 0.9367 | (0,0) | 4.5437 | 0.9367 | 255 | 0.0447 | 0.0336 | 210/210/210 | yes | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 8.3726 | 0.8663 | (0,0) | 8.3726 | 0.8663 | 255 | 0.0611 | 0.0509 | 210/210/210 | yes | 116.24 | 5.12 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | exact | 1/1 | ok | 6.7540 | 0.8880 | (0,0) | 6.7540 | 0.8880 | 255 | 0.0541 | 0.0441 | 210/210/210 | yes | 169.52 | 4.62 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-exact-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 8.4700 | 0.8655 | (-3.5,0) weak | 8.4128 | 0.8661 | 255 | 0.0615 | 0.0512 | 210/210/210 | yes | 91.03 | 4.50 | 0.9048 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-main-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex | pipeline | 1/1 | ok | 7.1603 | 0.8910 | (1,0) weak | 7.0842 | 0.8919 | 255 | 0.0552 | 0.0447 | 210/210/210 | yes | 169.52 | 4.26 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-pipeline-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 7.7862 | 0.8620 | (0,0) | 7.7862 | 0.8620 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | exact | 1/1 | ok | 0.0256 | 1.0000 | (0,0) | 0.0256 | 1.0000 | 6 | 0.0143 | 0.0000 | 210/210/210 | yes | 0.00 | 1.03 | 1.0000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-exact-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | main | 1/1 | ok | 7.7924 | 0.8614 | (0,0) | 7.7924 | 0.8614 | 255 | 0.0598 | 0.0495 | 210/210/210 | yes | 117.55 | 4.30 | 0.9000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 02-wrapping-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 4.5510 | 0.9366 | (0,0) | 4.5510 | 0.9366 | 255 | 0.0447 | 0.0336 | 210/210/210 | yes | 0.00 | 0.67 | 1.0000 | 0/0 | [p1](images/02-wrapping-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 8.3710 | 0.8656 | (0,0) | 8.3710 | 0.8656 | 255 | 0.0613 | 0.0509 | 210/210/210 | yes | 119.39 | 5.19 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex | exact | 1/1 | ok | 6.8111 | 0.8870 | (0,0) | 6.8111 | 0.8870 | 255 | 0.0545 | 0.0444 | 210/210/210 | yes | 170.98 | 4.69 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 8.4639 | 0.8648 | (-0.5,0) weak | 8.4585 | 0.8648 | 255 | 0.0618 | 0.0514 | 210/210/210 | yes | 94.99 | 4.57 | 0.9048 | 0/0 | - |
| 02-wrapping-paragraph | xelatex | pipeline | 1/1 | ok | 7.1981 | 0.8902 | (0.5,0) weak | 7.1671 | 0.8904 | 255 | 0.0556 | 0.0451 | 210/210/210 | yes | 170.98 | 4.33 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | de1020c | 1/1 | ok | 7.7866 | 0.8620 | (-0.5,-14.5) weak | 7.7691 | 0.8602 | 255 | 0.0597 | 0.0495 | 210/210/210 | yes | 91.94 | 3.95 | 0.8952 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | exact | 1/1 | ok | 0.0821 | 0.9999 | (0,0) | 0.0821 | 0.9999 | 18 | 0.0218 | 0.0000 | 210/210/210 | yes | 0.01 | 1.03 | 1.0000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | main | 1/1 | ok | 7.7922 | 0.8614 | (0,0) | 7.7922 | 0.8614 | 255 | 0.0598 | 0.0495 | 210/210/210 | yes | 117.54 | 4.30 | 0.9000 | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | pipeline | 1/1 | ok | 4.5446 | 0.9367 | (0,0) | 4.5446 | 0.9367 | 255 | 0.0447 | 0.0336 | 210/210/210 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.3394 | 0.9793 | (0,52.5) moderate | 1.1861 | 0.9829 | 255 | 0.0085 | 0.0073 | 15/15/15 | yes | 0.25 | 21.02 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex | exact | 1/1 | ok | 0.9612 | 0.9868 | (0,-1.5) weak | 0.9311 | 0.9876 | 255 | 0.0068 | 0.0057 | 15/15/15 | yes | 5.62 | 0.60 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.3597 | 0.9789 | (0,52.5) moderate | 1.2063 | 0.9826 | 255 | 0.0085 | 0.0074 | 15/17/15 | no | 2.62 | 21.02 | 0.8667 | 0/0 | - |
| 03-section-heading | lualatex | pipeline | 1/1 | ok | 1.0037 | 0.9865 | (0,-1.5) weak | 0.9814 | 0.9874 | 255 | 0.0069 | 0.0059 | 15/17/15 | no | 11.43 | 0.70 | 0.8667 | 0/0 | - |
| 03-section-heading | lualatex-lm | de1020c | 1/1 | ok | 1.2336 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | 15/15/15 | yes | 5.46 | 21.19 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex-lm | exact | 1/1 | ok | 0.0234 | 1.0000 | (0,0) | 0.0234 | 1.0000 | 27 | 0.0029 | 0.0000 | 15/15/15 | yes | 0.00 | 0.03 | 1.0000 | 0/0 | - |
| 03-section-heading | lualatex-lm | main | 1/1 | ok | 1.2426 | 0.9769 | (0,53.5) moderate | 1.1129 | 0.9812 | 255 | 0.0082 | 0.0070 | 15/17/15 | no | 6.81 | 21.19 | 0.8667 | 0/0 | - |
| 03-section-heading | lualatex-lm | pipeline | 1/1 | ok | 0.8550 | 0.9887 | (0,0) | 0.8550 | 0.9887 | 255 | 0.0062 | 0.0052 | 15/17/15 | no | 5.81 | 0.45 | 0.8667 | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.3496 | 0.9793 | (0.5,52.5) moderate | 1.2021 | 0.9827 | 255 | 0.0085 | 0.0072 | 15/15/15 | yes | 0.33 | 20.87 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-de1020c-export-p1-overlay.png) |
| 03-section-heading | pdflatex | exact | 1/1 | ok | 0.9881 | 0.9865 | (0,-1.5) weak | 0.9536 | 0.9874 | 255 | 0.0068 | 0.0057 | 15/15/15 | yes | 5.69 | 0.70 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-exact-export-p1-overlay.png) |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.3719 | 0.9789 | (0.5,52.5) moderate | 1.2200 | 0.9824 | 255 | 0.0086 | 0.0074 | 15/17/15 | no | 2.70 | 20.87 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-main-export-p1-overlay.png) |
| 03-section-heading | pdflatex | pipeline | 1/1 | ok | 1.0316 | 0.9862 | (0.5,-2) weak | 1.0233 | 0.9859 | 255 | 0.0070 | 0.0059 | 15/17/15 | no | 11.51 | 0.87 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-pipeline-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | de1020c | 1/1 | ok | 1.2335 | 0.9771 | (0,54) moderate | 1.0954 | 0.9811 | 255 | 0.0082 | 0.0069 | 15/15/15 | yes | 5.46 | 22.37 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | exact | 1/1 | ok | 0.0021 | 1.0000 | (0,0) | 0.0021 | 1.0000 | 5 | 0.0011 | 0.0000 | 15/15/15 | yes | 0.00 | 1.15 | 1.0000 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-exact-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | main | 1/1 | ok | 1.2428 | 0.9769 | (0,54) moderate | 1.1208 | 0.9807 | 255 | 0.0082 | 0.0070 | 15/17/15 | no | 6.81 | 22.37 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-main-export-p1-overlay.png) |
| 03-section-heading | pdflatex-lm | pipeline | 1/1 | ok | 0.8547 | 0.9887 | (0,0) | 0.8547 | 0.9887 | 255 | 0.0062 | 0.0052 | 15/17/15 | no | 5.81 | 0.73 | 0.8667 | 0/0 | [p1](images/03-section-heading/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.3370 | 0.9793 | (0,52.5) moderate | 1.1868 | 0.9829 | 255 | 0.0085 | 0.0073 | 15/15/15 | yes | 0.24 | 21.02 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex | exact | 1/1 | ok | 0.9621 | 0.9868 | (0,-1.5) weak | 0.9334 | 0.9876 | 255 | 0.0068 | 0.0057 | 15/15/15 | yes | 5.61 | 0.60 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.3601 | 0.9789 | (0,52.5) moderate | 1.2075 | 0.9826 | 255 | 0.0086 | 0.0075 | 15/17/15 | no | 2.61 | 21.02 | 0.8667 | 0/0 | - |
| 03-section-heading | xelatex | pipeline | 1/1 | ok | 1.0089 | 0.9864 | (0,-1.5) weak | 0.9851 | 0.9874 | 255 | 0.0070 | 0.0059 | 15/17/15 | no | 11.42 | 0.70 | 0.8667 | 0/0 | - |
| 03-section-heading | xelatex-lm | de1020c | 1/1 | ok | 1.2337 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | 15/15/15 | yes | 5.46 | 22.35 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex-lm | exact | 1/1 | ok | 0.0239 | 1.0000 | (0,0) | 0.0239 | 1.0000 | 27 | 0.0029 | 0.0000 | 15/15/15 | yes | 0.00 | 1.13 | 1.0000 | 0/0 | - |
| 03-section-heading | xelatex-lm | main | 1/1 | ok | 1.2425 | 0.9769 | (0,53.5) moderate | 1.1129 | 0.9812 | 255 | 0.0082 | 0.0070 | 15/17/15 | no | 6.81 | 22.35 | 0.8667 | 0/0 | - |
| 03-section-heading | xelatex-lm | pipeline | 1/1 | ok | 0.8550 | 0.9887 | (0,0) | 0.8550 | 0.9887 | 255 | 0.0062 | 0.0052 | 15/17/15 | no | 5.81 | 0.70 | 0.8667 | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.3808 | 0.9945 | (0,0) | 0.3808 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 3.77 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | exact | 1/1 | ok | 0.4519 | 0.9930 | (13.5,0) weak | 0.4468 | 0.9929 | 255 | 0.0035 | 0.0028 | 10/10/10 | yes | 17.00 | 1.09 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.3769 | 0.9945 | (0,0) | 0.3769 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 3.73 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex | pipeline | 1/1 | ok | 0.4246 | 0.9938 | (0,0) | 0.4246 | 0.9938 | 255 | 0.0033 | 0.0026 | 10/11/9 | no | 18.51 | 0.41 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | de1020c | 1/1 | ok | 0.4244 | 0.9933 | (-49,0) weak | 0.4148 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.02 | 0.63 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | exact | 1/1 | ok | 0.0187 | 0.9999 | (0,0) | 0.0187 | 0.9999 | 60 | 0.0014 | 0.0002 | 10/10/10 | yes | 0.05 | 0.00 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | main | 1/1 | ok | 0.4253 | 0.9933 | (-49,0) weak | 0.4147 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.07 | 0.63 | 1.0000 | 0/0 | - |
| 04-bold-emph | lualatex-lm | pipeline | 1/1 | ok | 0.3651 | 0.9947 | (0,0) | 0.3651 | 0.9947 | 255 | 0.0031 | 0.0024 | 10/11/9 | no | 0.05 | 0.68 | 1.0000 | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.3836 | 0.9944 | (-0.5,0) weak | 0.3764 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 4.04 | 0.46 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-de1020c-export-p1-overlay.png) |
| 04-bold-emph | pdflatex | exact | 1/1 | ok | 0.4549 | 0.9931 | (-29.5,0) weak | 0.4518 | 0.9929 | 255 | 0.0035 | 0.0028 | 10/10/10 | yes | 17.21 | 1.09 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-exact-export-p1-overlay.png) |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.3804 | 0.9944 | (-0.5,0) weak | 0.3777 | 0.9945 | 255 | 0.0031 | 0.0024 | 10/12/8 | no | 3.99 | 0.46 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-main-export-p1-overlay.png) |
| 04-bold-emph | pdflatex | pipeline | 1/1 | ok | 0.4235 | 0.9938 | (0,0) | 0.4235 | 0.9938 | 255 | 0.0033 | 0.0026 | 10/11/9 | no | 18.75 | 0.41 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-pipeline-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | de1020c | 1/1 | ok | 0.4246 | 0.9933 | (-49,0) weak | 0.4162 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.08 | 0.73 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | exact | 1/1 | ok | 0.0016 | 1.0000 | (0,0) | 0.0016 | 1.0000 | 4 | 0.0009 | 0.0000 | 10/10/10 | yes | 0.00 | 1.35 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-exact-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | main | 1/1 | ok | 0.4256 | 0.9933 | (-49,0) weak | 0.4164 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 16.13 | 0.73 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-main-export-p1-overlay.png) |
| 04-bold-emph | pdflatex-lm | pipeline | 1/1 | ok | 0.3653 | 0.9947 | (0,0) | 0.3653 | 0.9947 | 255 | 0.0031 | 0.0024 | 10/11/9 | no | 0.00 | 0.67 | 1.0000 | 0/0 | [p1](images/04-bold-emph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.3848 | 0.9944 | (-0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0031 | 0.0025 | 10/12/8 | no | 3.81 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex | exact | 1/1 | ok | 0.4502 | 0.9931 | (38.5,0) weak | 0.4471 | 0.9932 | 255 | 0.0035 | 0.0028 | 10/10/10 | yes | 17.04 | 1.09 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.3810 | 0.9945 | (-0.5,0) weak | 0.3801 | 0.9944 | 255 | 0.0031 | 0.0025 | 10/12/8 | no | 3.76 | 0.46 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex | pipeline | 1/1 | ok | 0.4236 | 0.9938 | (0,0) | 0.4236 | 0.9938 | 255 | 0.0033 | 0.0026 | 10/11/9 | no | 18.55 | 0.41 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | de1020c | 1/1 | ok | 0.4245 | 0.9933 | (-49,0) weak | 0.4138 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 15.90 | 0.73 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | exact | 1/1 | ok | 0.0503 | 0.9995 | (0,0) | 0.0503 | 0.9995 | 182 | 0.0015 | 0.0004 | 10/10/10 | yes | 0.14 | 1.35 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | main | 1/1 | ok | 0.4237 | 0.9933 | (-49,0) weak | 0.4127 | 0.9934 | 255 | 0.0033 | 0.0027 | 10/12/8 | no | 15.95 | 0.73 | 1.0000 | 0/0 | - |
| 04-bold-emph | xelatex-lm | pipeline | 1/1 | ok | 0.3770 | 0.9945 | (0,0) | 0.3770 | 0.9945 | 255 | 0.0031 | 0.0025 | 10/11/9 | no | 0.16 | 0.67 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.2971 | 0.9955 | (0,0) | 0.2971 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex | exact | 1/1 | ok | 0.3668 | 0.9938 | (9.5,0) moderate | 0.3164 | 0.9946 | 255 | 0.0029 | 0.0024 | 11/11/11 | yes | 7.38 | 0.77 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.2489 | 0.9963 | (0,0) | 0.2489 | 0.9963 | 255 | 0.0026 | 0.0018 | 11/19/7 | no | 5.19 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex | pipeline | 1/1 | ok | 0.3578 | 0.9948 | (11.5,0) moderate | 0.3282 | 0.9951 | 255 | 0.0029 | 0.0022 | 11/11/11 | yes | 7.38 | 0.41 | 1.0000 | 2/0 | - |
| 05-unicode | lualatex-lm | de1020c | 1/1 | ok | 0.3619 | 0.9938 | (2.5,0) weak | 0.3494 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.30 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex-lm | exact | 1/1 | ok | 0.0051 | 1.0000 | (0,0) | 0.0051 | 1.0000 | 16 | 0.0011 | 0.0000 | 11/11/11 | yes | 0.01 | 0.00 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex-lm | main | 1/1 | ok | 0.3628 | 0.9938 | (-3.5,0) moderate | 0.3432 | 0.9941 | 255 | 0.0029 | 0.0023 | 11/19/7 | no | 6.34 | 0.30 | 1.0000 | 0/0 | - |
| 05-unicode | lualatex-lm | pipeline | 1/1 | ok | 0.2655 | 0.9960 | (-0.5,0) moderate | 0.2087 | 0.9969 | 255 | 0.0025 | 0.0019 | 11/11/11 | yes | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.3127 | 0.9955 | (0,0) | 0.3127 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.88 | 0.46 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-de1020c-export-p1-overlay.png) |
| 05-unicode | pdflatex | exact | 1/1 | ok | 0.3708 | 0.9940 | (10,0) moderate | 0.3228 | 0.9948 | 255 | 0.0029 | 0.0023 | 11/11/11 | yes | 7.58 | 0.77 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-exact-export-p1-overlay.png) |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.2342 | 0.9967 | (0,0) | 0.2342 | 0.9967 | 255 | 0.0025 | 0.0016 | 11/19/7 | no | 5.21 | 0.46 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-main-export-p1-overlay.png) |
| 05-unicode | pdflatex | pipeline | 1/1 | ok | 0.3573 | 0.9950 | (11.5,0) moderate | 0.3051 | 0.9957 | 255 | 0.0029 | 0.0022 | 11/11/11 | yes | 7.58 | 0.41 | 1.0000 | 2/0 | [p1](images/05-unicode/pdflatex-pipeline-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | de1020c | 1/1 | ok | 0.3617 | 0.9938 | (2.5,0) weak | 0.3496 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | exact | 1/1 | ok | 0.0011 | 1.0000 | (0,0) | 0.0011 | 1.0000 | 4 | 0.0007 | 0.0000 | 11/11/11 | yes | 0.00 | 1.03 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-exact-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | main | 1/1 | ok | 0.3625 | 0.9938 | (-3.5,0) moderate | 0.3430 | 0.9941 | 255 | 0.0029 | 0.0023 | 11/19/7 | no | 6.34 | 0.73 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-main-export-p1-overlay.png) |
| 05-unicode | pdflatex-lm | pipeline | 1/1 | ok | 0.2675 | 0.9959 | (-0.5,0) moderate | 0.2083 | 0.9969 | 255 | 0.0025 | 0.0019 | 11/11/11 | yes | 0.00 | 0.67 | 1.0000 | 0/0 | [p1](images/05-unicode/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.2963 | 0.9955 | (0,0) | 0.2963 | 0.9955 | 255 | 0.0027 | 0.0020 | 11/19/7 | no | 5.80 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex | exact | 1/1 | ok | 0.3660 | 0.9938 | (9.5,0) moderate | 0.3163 | 0.9946 | 255 | 0.0029 | 0.0024 | 11/11/11 | yes | 7.35 | 0.77 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.2493 | 0.9963 | (0,0) | 0.2493 | 0.9963 | 255 | 0.0026 | 0.0018 | 11/19/7 | no | 5.19 | 0.46 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex | pipeline | 1/1 | ok | 0.3576 | 0.9948 | (11.5,0) moderate | 0.3312 | 0.9951 | 255 | 0.0029 | 0.0022 | 11/11/11 | yes | 7.35 | 0.41 | 1.0000 | 2/0 | - |
| 05-unicode | xelatex-lm | de1020c | 1/1 | ok | 0.3620 | 0.9938 | (2.5,0) weak | 0.3493 | 0.9940 | 255 | 0.0029 | 0.0024 | 11/19/7 | no | 6.78 | 0.73 | 1.0000 | 0/0 | - |
| 05-unicode | xelatex-lm | exact | 1/1 | ok | 0.0051 | 1.0000 | (0,0) | 0.0051 | 1.0000 | 16 | 0.0011 | 0.0000 | 11/11/11 | yes | 0.01 | 1.03 | 1.0000 | 0/0 | - |
| 05-unicode | xelatex-lm | main | 1/1 | ok | 0.3628 | 0.9938 | (-3.5,0) moderate | 0.3432 | 0.9941 | 255 | 0.0029 | 0.0023 | 11/19/7 | no | 6.34 | 0.73 | 1.0000 | 0/0 | - |
| 05-unicode | xelatex-lm | pipeline | 1/1 | ok | 0.2657 | 0.9960 | (-0.5,0) moderate | 0.2088 | 0.9969 | 255 | 0.0025 | 0.0019 | 11/11/11 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.3832 | 0.9934 | (0,3.5) | 0.2554 | 0.9959 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | exact | 1/1 | ok | 0.2978 | 0.9945 | (0,0) | 0.2978 | 0.9945 | 255 | 0.0025 | 0.0020 | 15/15/15 | yes | 8.95 | 0.15 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | ok | 0.3838 | 0.9933 | (-6.5,3.5) | 0.1998 | 0.9967 | 255 | 0.0029 | 0.0023 | 15/13/12 | no | 3.50 | 2.46 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex | pipeline | 1/1 | ok | 0.2979 | 0.9948 | (0,0) | 0.2979 | 0.9948 | 255 | 0.0025 | 0.0019 | 15/16/14 | no | 8.83 | 1.24 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | de1020c | 1/1 | ok | 0.3682 | 0.9931 | (-32.5,3.5) moderate | 0.3134 | 0.9945 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 12.25 | 2.20 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | exact | 1/1 | ok | 0.0133 | 0.9999 | (0,0) | 0.0133 | 0.9999 | 110 | 0.0012 | 0.0001 | 15/15/15 | yes | 0.02 | 0.00 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | main | 1/1 | ok | 0.3735 | 0.9928 | (9.5,3.5) moderate | 0.3120 | 0.9945 | 255 | 0.0029 | 0.0024 | 15/13/12 | no | 12.48 | 2.27 | 1.0000 | 0/0 | - |
| 06-math-inline | lualatex-lm | pipeline | 1/1 | ok | 0.2287 | 0.9963 | (-0.5,0) moderate | 0.2164 | 0.9963 | 255 | 0.0021 | 0.0016 | 15/16/14 | no | 0.02 | 1.23 | 1.0000 | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.3821 | 0.9936 | (0,3.5) | 0.2595 | 0.9960 | 255 | 0.0029 | 0.0023 | 13/14/10 | no | 3.89 | 2.69 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-de1020c-export-p1-overlay.png) |
| 06-math-inline | pdflatex | exact | 1/1 | ok | 0.3045 | 0.9946 | (4,0) moderate | 0.2819 | 0.9949 | 255 | 0.0025 | 0.0020 | 13/15/11 | no | 9.11 | 0.21 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-exact-export-p1-overlay.png) |
| 06-math-inline | pdflatex | main | 1/1 | ok | 0.3782 | 0.9935 | (-6,3.5) | 0.2119 | 0.9968 | 255 | 0.0029 | 0.0023 | 13/13/10 | no | 3.10 | 2.69 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-main-export-p1-overlay.png) |
| 06-math-inline | pdflatex | pipeline | 1/1 | ok | 0.3055 | 0.9949 | (12,0) weak | 0.2957 | 0.9952 | 255 | 0.0025 | 0.0019 | 13/16/11 | no | 9.11 | 1.25 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-pipeline-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | de1020c | 1/1 | ok | 0.3680 | 0.9931 | (9.5,3.5) moderate | 0.3234 | 0.9943 | 255 | 0.0029 | 0.0023 | 14/14/12 | no | 13.02 | 2.53 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | exact | 1/1 | ok | 0.0069 | 0.9999 | (0,0) | 0.0069 | 0.9999 | 114 | 0.0008 | 0.0001 | 14/15/13 | no | 0.00 | 0.24 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-exact-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | main | 1/1 | ok | 0.3729 | 0.9928 | (9.5,3.5) moderate | 0.3115 | 0.9945 | 255 | 0.0029 | 0.0024 | 14/13/12 | no | 12.50 | 2.53 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-main-export-p1-overlay.png) |
| 06-math-inline | pdflatex-lm | pipeline | 1/1 | ok | 0.2299 | 0.9963 | (-0.5,0) moderate | 0.2150 | 0.9964 | 255 | 0.0021 | 0.0016 | 14/16/12 | no | 0.00 | 1.43 | 1.0000 | 0/0 | [p1](images/06-math-inline/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.3831 | 0.9934 | (0,3.5) | 0.2548 | 0.9959 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 4.26 | 2.37 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex | exact | 1/1 | ok | 0.2979 | 0.9945 | (0,0) | 0.2979 | 0.9945 | 255 | 0.0025 | 0.0020 | 15/15/15 | yes | 8.95 | 0.15 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | ok | 0.3839 | 0.9933 | (-6.5,3.5) | 0.1996 | 0.9967 | 255 | 0.0029 | 0.0023 | 15/13/12 | no | 3.50 | 2.46 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex | pipeline | 1/1 | ok | 0.2979 | 0.9948 | (0,0) | 0.2979 | 0.9948 | 255 | 0.0025 | 0.0019 | 15/16/14 | no | 8.82 | 1.24 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | de1020c | 1/1 | ok | 0.3682 | 0.9931 | (9.5,3.5) moderate | 0.3233 | 0.9943 | 255 | 0.0029 | 0.0023 | 15/14/13 | no | 12.25 | 2.44 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | exact | 1/1 | ok | 0.0127 | 0.9999 | (0,0) | 0.0127 | 0.9999 | 110 | 0.0012 | 0.0001 | 15/15/15 | yes | 0.02 | 0.21 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | main | 1/1 | ok | 0.3734 | 0.9928 | (9.5,3.5) moderate | 0.3119 | 0.9945 | 255 | 0.0029 | 0.0024 | 15/13/12 | no | 12.48 | 2.53 | 1.0000 | 0/0 | - |
| 06-math-inline | xelatex-lm | pipeline | 1/1 | ok | 0.2288 | 0.9963 | (-0.5,0) moderate | 0.2164 | 0.9963 | 255 | 0.0021 | 0.0016 | 15/16/14 | no | 0.02 | 1.29 | 1.0000 | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.2595 | 0.9950 | (0,0) | 0.2595 | 0.9950 | 255 | 0.0026 | 0.0019 | 13/10/7 | no | 0.60 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex | exact | 1/1 | ok | 0.2571 | 0.9957 | (0,0) | 0.2571 | 0.9957 | 255 | 0.0025 | 0.0019 | 13/15/12 | no | 1.00 | 0.77 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | ok | 0.2640 | 0.9946 | (0,0) | 0.2640 | 0.9946 | 255 | 0.0026 | 0.0019 | 13/11/7 | no | 0.41 | 1.33 | 1.0000 | 3/0 | - |
| 07-math-display | lualatex | pipeline | 1/1 | ok | 0.2678 | 0.9954 | (0,0) | 0.2678 | 0.9954 | 255 | 0.0026 | 0.0019 | 13/16/12 | no | 2.33 | 1.48 | 0.9167 | 3/1 | - |
| 07-math-display | lualatex-lm | de1020c | 1/1 | ok | 0.3495 | 0.9931 | (0,0) | 0.3495 | 0.9931 | 255 | 0.0029 | 0.0024 | 13/10/7 | no | 2.12 | 0.92 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex-lm | exact | 1/1 | ok | 0.0847 | 0.9986 | (0,0) | 0.0847 | 0.9986 | 255 | 0.0017 | 0.0008 | 13/15/12 | no | 0.00 | 0.50 | 1.0000 | 3/1 | - |
| 07-math-display | lualatex-lm | main | 1/1 | ok | 0.3541 | 0.9928 | (0,0) | 0.3541 | 0.9928 | 255 | 0.0029 | 0.0024 | 13/11/7 | no | 1.94 | 0.92 | 1.0000 | 3/0 | - |
| 07-math-display | lualatex-lm | pipeline | 1/1 | ok | 0.2506 | 0.9957 | (-0.5,0) moderate | 0.2265 | 0.9961 | 255 | 0.0025 | 0.0019 | 13/16/12 | no | 1.34 | 1.68 | 0.9167 | 3/1 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.2609 | 0.9949 | (0,0) | 0.2609 | 0.9949 | 255 | 0.0026 | 0.0019 | 13/10/7 | no | 0.59 | 1.41 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-de1020c-export-p1-overlay.png) |
| 07-math-display | pdflatex | exact | 1/1 | ok | 0.2563 | 0.9958 | (0,0) | 0.2563 | 0.9958 | 255 | 0.0025 | 0.0019 | 13/15/12 | no | 1.00 | 0.82 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-exact-export-p1-overlay.png) |
| 07-math-display | pdflatex | main | 1/1 | ok | 0.2654 | 0.9946 | (0,0) | 0.2654 | 0.9946 | 255 | 0.0026 | 0.0019 | 13/11/7 | no | 0.41 | 1.41 | 1.0000 | 3/0 | [p1](images/07-math-display/pdflatex-main-export-p1-overlay.png) |
| 07-math-display | pdflatex | pipeline | 1/1 | ok | 0.2521 | 0.9957 | (0,0) | 0.2521 | 0.9957 | 255 | 0.0024 | 0.0017 | 13/16/12 | no | 2.33 | 1.52 | 0.9167 | 3/1 | [p1](images/07-math-display/pdflatex-pipeline-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | de1020c | 1/1 | ok | 0.3528 | 0.9931 | (0,0) | 0.3528 | 0.9931 | 255 | 0.0029 | 0.0024 | 13/10/7 | no | 2.13 | 1.73 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | exact | 1/1 | ok | 0.0389 | 0.9992 | (0,0) | 0.0389 | 0.9992 | 255 | 0.0011 | 0.0003 | 13/15/12 | no | 0.00 | 0.85 | 1.0000 | 3/1 | [p1](images/07-math-display/pdflatex-lm-exact-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | main | 1/1 | ok | 0.3574 | 0.9927 | (0,0) | 0.3574 | 0.9927 | 255 | 0.0029 | 0.0024 | 13/11/7 | no | 1.94 | 1.73 | 1.0000 | 3/0 | [p1](images/07-math-display/pdflatex-lm-main-export-p1-overlay.png) |
| 07-math-display | pdflatex-lm | pipeline | 1/1 | ok | 0.2416 | 0.9959 | (-0.5,0) moderate | 0.2141 | 0.9963 | 255 | 0.0023 | 0.0018 | 13/16/12 | no | 1.34 | 1.77 | 0.9167 | 3/1 | [p1](images/07-math-display/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.2589 | 0.9950 | (0,0) | 0.2589 | 0.9950 | 255 | 0.0026 | 0.0019 | 14/10/7 | no | 0.59 | 1.33 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex | exact | 1/1 | ok | 0.2579 | 0.9957 | (0,0) | 0.2579 | 0.9957 | 255 | 0.0026 | 0.0019 | 14/15/13 | no | 1.28 | 2.76 | 0.9231 | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | ok | 0.2634 | 0.9946 | (0,0) | 0.2634 | 0.9946 | 255 | 0.0026 | 0.0019 | 14/11/7 | no | 0.41 | 1.33 | 1.0000 | 3/0 | - |
| 07-math-display | xelatex | pipeline | 1/1 | ok | 0.2675 | 0.9955 | (0,0) | 0.2675 | 0.9955 | 255 | 0.0026 | 0.0019 | 14/16/13 | no | 2.48 | 3.49 | 0.9231 | 3/1 | - |
| 07-math-display | xelatex-lm | de1020c | 1/1 | ok | 0.3495 | 0.9931 | (0,0) | 0.3495 | 0.9931 | 255 | 0.0029 | 0.0024 | 14/10/7 | no | 2.12 | 1.54 | 1.0000 | 3/1 | - |
| 07-math-display | xelatex-lm | exact | 1/1 | ok | 0.0848 | 0.9986 | (0,0) | 0.0848 | 0.9986 | 255 | 0.0017 | 0.0008 | 14/15/13 | no | 0.36 | 2.80 | 0.9231 | 3/1 | - |
| 07-math-display | xelatex-lm | main | 1/1 | ok | 0.3541 | 0.9928 | (0,0) | 0.3541 | 0.9928 | 255 | 0.0029 | 0.0024 | 14/11/7 | no | 1.94 | 1.54 | 1.0000 | 3/0 | - |
| 07-math-display | xelatex-lm | pipeline | 1/1 | ok | 0.2506 | 0.9957 | (-0.5,0) moderate | 0.2268 | 0.9961 | 255 | 0.0025 | 0.0019 | 14/16/13 | no | 1.57 | 3.65 | 0.9231 | 3/1 | - |
| 08-two-page | lualatex | de1020c | 3/3 | ok | 26.2176 | 0.5657 | (0,0); (-3,6) moderate; (-3,34.5) moderate | 24.6323 | 0.5959 | 255 | 0.1880 | 0.1574 | 1800/1800/1800 | yes | 164.00 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | lualatex | exact | 3/3 | ok | 19.5677 | 0.6675 | (0,0); (0,0); (0,29) weak | 19.5528 | 0.6678 | 255 | 0.1573 | 0.1279 | 1800/1800/1800 | yes | 163.81 | 68.31 | 0.8960 | 0/0 | - |
| 08-two-page | lualatex | main | 3/3 | ok | 25.2812 | 0.5914 | (0,0); (0,2) weak; (0,49) moderate | 24.7462 | 0.6038 | 255 | 0.1832 | 0.1528 | 1800/1800/1800 | yes | 138.34 | 31.84 | 0.9011 | 0/0 | - |
| 08-two-page | lualatex | pipeline | 3/3 | ok | 21.1539 | 0.6700 | (0,0); (0,0); (1,0) weak | 21.1011 | 0.6710 | 255 | 0.1618 | 0.1323 | 1800/1800/1800 | yes | 163.81 | 67.95 | 0.8960 | 0/0 | - |
| 08-two-page | lualatex-lm | de1020c | 3/3 | ok | 23.7967 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7513 | 0.5869 | 255 | 0.1801 | 0.1495 | 1800/1800/1800 | yes | 142.51 | 30.71 | 0.8878 | 0/0 | - |
| 08-two-page | lualatex-lm | exact | 3/3 | ok | 0.2057 | 0.9999 | (0,0); (0,0); (0,0) | 0.2057 | 0.9999 | 14 | 0.0609 | 0.0000 | 1800/1800/1800 | yes | 0.01 | 0.00 | 1.0000 | 0/0 | - |
| 08-two-page | lualatex-lm | main | 3/3 | ok | 23.2786 | 0.5778 | (-0.5,-27) weak; (-0.5,-12.5) weak; (-0.5,5.5) moderate | 22.8311 | 0.5859 | 255 | 0.1773 | 0.1467 | 1800/1800/1800 | yes | 160.68 | 38.33 | 0.8958 | 0/0 | - |
| 08-two-page | lualatex-lm | pipeline | 3/3 | ok | 12.9945 | 0.8187 | (0,0); (0,0); (0,0) | 12.9945 | 0.8187 | 255 | 0.1280 | 0.0962 | 1800/1800/1800 | yes | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/3 | ok | 26.1974 | 0.5667 | (0,-14.5) weak; (0,6) moderate; (0,34.5) moderate | 24.5504 | 0.5980 | 255 | 0.1875 | 0.1572 | 1800/1800/1800 | yes | 143.13 | 92.17 | 0.8952 | 0/0 | [p1](images/08-two-page/pdflatex-de1020c-export-p1-overlay.png) |
| 08-two-page | pdflatex | exact | 3/3 | ok | 19.5228 | 0.6688 | (0,0); (0,0); (0,0) | 19.5228 | 0.6688 | 255 | 0.1567 | 0.1275 | 1800/1800/1800 | yes | 179.69 | 67.50 | 0.8961 | 0/0 | [p1](images/08-two-page/pdflatex-exact-export-p1-overlay.png) |
| 08-two-page | pdflatex | main | 3/3 | ok | 25.3242 | 0.5921 | (0,0); (-11,2) weak; (0,49) moderate | 24.7896 | 0.6035 | 255 | 0.1829 | 0.1528 | 1800/1800/1800 | yes | 112.56 | 31.07 | 0.9011 | 0/0 | [p1](images/08-two-page/pdflatex-main-export-p1-overlay.png) |
| 08-two-page | pdflatex | pipeline | 3/3 | ok | 21.1721 | 0.6711 | (-1.5,0) weak; (1.5,0) weak; (-1.5,0) weak | 21.0243 | 0.6736 | 255 | 0.1614 | 0.1321 | 1800/1800/1800 | yes | 179.69 | 67.14 | 0.8961 | 0/0 | [p1](images/08-two-page/pdflatex-pipeline-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | de1020c | 3/3 | ok | 23.7963 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7519 | 0.5868 | 255 | 0.1801 | 0.1496 | 1800/1800/1800 | yes | 142.52 | 31.52 | 0.8878 | 0/0 | [p1](images/08-two-page/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | exact | 3/3 | ok | 0.0716 | 1.0000 | (0,0); (0,0); (0,0) | 0.0716 | 1.0000 | 6 | 0.0404 | 0.0000 | 1800/1800/1800 | yes | 0.00 | 1.03 | 1.0000 | 0/0 | [p1](images/08-two-page/pdflatex-lm-exact-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | main | 3/3 | ok | 23.2793 | 0.5778 | (-0.5,-27) weak; (-0.5,2) weak; (-0.5,5.5) moderate | 22.8313 | 0.5887 | 255 | 0.1772 | 0.1468 | 1800/1800/1800 | yes | 160.69 | 37.39 | 0.8958 | 0/0 | [p1](images/08-two-page/pdflatex-lm-main-export-p1-overlay.png) |
| 08-two-page | pdflatex-lm | pipeline | 3/3 | ok | 13.0212 | 0.8183 | (0,0); (0,0); (0,0) | 13.0212 | 0.8183 | 255 | 0.1281 | 0.0964 | 1800/1800/1800 | yes | 0.00 | 0.67 | 1.0000 | 0/0 | [p1](images/08-two-page/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 08-two-page | xelatex | de1020c | 3/3 | ok | 26.2425 | 0.5656 | (-3,-14.5) weak; (-3,6) moderate; (-3,34.5) moderate | 24.6595 | 0.5953 | 255 | 0.1881 | 0.1578 | 1800/1800/1800 | yes | 164.01 | 93.00 | 0.8951 | 0/0 | - |
| 08-two-page | xelatex | exact | 3/3 | ok | 19.5602 | 0.6674 | (0,0); (0,0); (0,29) weak | 19.5568 | 0.6675 | 255 | 0.1573 | 0.1282 | 1800/1800/1800 | yes | 163.84 | 68.31 | 0.8960 | 0/0 | - |
| 08-two-page | xelatex | main | 3/3 | ok | 25.2630 | 0.5918 | (0,0); (0,2) weak; (0,49) moderate | 24.7282 | 0.6041 | 255 | 0.1831 | 0.1530 | 1800/1800/1800 | yes | 138.34 | 31.84 | 0.9011 | 0/0 | - |
| 08-two-page | xelatex | pipeline | 3/3 | ok | 21.1729 | 0.6696 | (1.5,0) weak; (0,0); (1,0) weak | 21.1140 | 0.6705 | 255 | 0.1619 | 0.1325 | 1800/1800/1800 | yes | 163.84 | 67.95 | 0.8960 | 0/0 | - |
| 08-two-page | xelatex-lm | de1020c | 3/3 | ok | 23.7969 | 0.5633 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7509 | 0.5869 | 255 | 0.1801 | 0.1495 | 1800/1800/1800 | yes | 142.51 | 31.52 | 0.8878 | 0/0 | - |
| 08-two-page | xelatex-lm | exact | 3/3 | ok | 0.2162 | 0.9998 | (0,0); (0,0); (0,0) | 0.2162 | 0.9998 | 15 | 0.0605 | 0.0000 | 1800/1800/1800 | yes | 0.01 | 1.03 | 1.0000 | 0/0 | - |
| 08-two-page | xelatex-lm | main | 3/3 | ok | 23.2788 | 0.5778 | (-0.5,-27) weak; (-0.5,-12.5) weak; (-0.5,5.5) moderate | 22.8312 | 0.5859 | 255 | 0.1772 | 0.1467 | 1800/1800/1800 | yes | 160.68 | 37.39 | 0.8958 | 0/0 | - |
| 08-two-page | xelatex-lm | pipeline | 3/3 | ok | 12.9979 | 0.8186 | (0,0); (0,0); (0,0) | 12.9979 | 0.8186 | 255 | 0.1280 | 0.0962 | 1800/1800/1800 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.4844 | 0.9504 | (0,38.5) | 1.7912 | 0.9702 | 255 | 0.0174 | 0.0146 | 54/54/45 | no | 2.41 | 31.76 | 0.9778 | 0/0 | - |
| 09-mixed-document | lualatex | exact | 1/1 | ok | 1.7278 | 0.9714 | (0,-1) moderate | 1.6074 | 0.9744 | 255 | 0.0136 | 0.0112 | 54/54/54 | yes | 28.18 | 0.66 | 0.9630 | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | ok | 2.5088 | 0.9499 | (0,38.5) | 1.6551 | 0.9717 | 255 | 0.0175 | 0.0147 | 54/56/43 | no | 3.15 | 32.26 | 0.9767 | 0/0 | - |
| 09-mixed-document | lualatex | pipeline | 1/1 | ok | 1.9266 | 0.9706 | (1,-1) moderate | 1.7954 | 0.9729 | 255 | 0.0142 | 0.0118 | 54/57/50 | no | 30.58 | 0.62 | 0.9200 | 0/0 | - |
| 09-mixed-document | lualatex-lm | de1020c | 1/1 | ok | 2.2538 | 0.9494 | (-0.5,39) moderate | 1.9017 | 0.9641 | 255 | 0.0169 | 0.0140 | 54/54/45 | no | 30.16 | 31.13 | 0.9333 | 0/0 | - |
| 09-mixed-document | lualatex-lm | exact | 1/1 | ok | 0.4905 | 0.9918 | (0,-0.5) moderate | 0.3817 | 0.9955 | 255 | 0.0085 | 0.0043 | 54/54/54 | yes | 0.01 | 0.40 | 0.9815 | 0/0 | - |
| 09-mixed-document | lualatex-lm | main | 1/1 | ok | 2.2680 | 0.9490 | (-0.5,39) moderate | 1.8972 | 0.9637 | 255 | 0.0169 | 0.0141 | 54/56/43 | no | 32.35 | 31.57 | 0.9302 | 0/0 | - |
| 09-mixed-document | lualatex-lm | pipeline | 1/1 | ok | 1.3481 | 0.9800 | (-0.5,-0.5) moderate | 1.2413 | 0.9824 | 255 | 0.0119 | 0.0095 | 54/57/50 | no | 0.59 | 0.74 | 0.9400 | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.4853 | 0.9504 | (-0.5,38.5) moderate | 1.8886 | 0.9685 | 255 | 0.0174 | 0.0146 | 54/54/45 | no | 2.48 | 31.80 | 0.9778 | 0/0 | [p1](images/09-mixed-document/pdflatex-de1020c-export-p1-overlay.png) |
| 09-mixed-document | pdflatex | exact | 1/1 | ok | 1.7369 | 0.9713 | (0,-1) moderate | 1.6135 | 0.9744 | 255 | 0.0136 | 0.0112 | 54/54/54 | yes | 28.28 | 0.59 | 0.9630 | 0/0 | [p1](images/09-mixed-document/pdflatex-exact-export-p1-overlay.png) |
| 09-mixed-document | pdflatex | main | 1/1 | ok | 2.5100 | 0.9498 | (0,38.5) | 1.8697 | 0.9687 | 255 | 0.0175 | 0.0147 | 54/56/43 | no | 3.22 | 32.30 | 0.9767 | 0/0 | [p1](images/09-mixed-document/pdflatex-main-export-p1-overlay.png) |
| 09-mixed-document | pdflatex | pipeline | 1/1 | ok | 1.9404 | 0.9705 | (1,-1) moderate | 1.7758 | 0.9733 | 255 | 0.0141 | 0.0119 | 54/57/50 | no | 30.69 | 0.60 | 0.9200 | 0/0 | [p1](images/09-mixed-document/pdflatex-pipeline-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | de1020c | 1/1 | ok | 2.2522 | 0.9480 | (-0.5,39.5) moderate | 1.9012 | 0.9623 | 255 | 0.0167 | 0.0139 | 54/54/45 | no | 30.17 | 32.47 | 0.9333 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | exact | 1/1 | ok | 0.0423 | 0.9998 | (0,0) | 0.0423 | 0.9998 | 145 | 0.0045 | 0.0002 | 54/54/54 | yes | 0.02 | 0.92 | 1.0000 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-exact-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | main | 1/1 | ok | 2.2668 | 0.9476 | (-0.5,39.5) moderate | 1.8976 | 0.9619 | 255 | 0.0168 | 0.0140 | 54/56/43 | no | 32.36 | 32.96 | 0.9302 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-main-export-p1-overlay.png) |
| 09-mixed-document | pdflatex-lm | pipeline | 1/1 | ok | 1.1782 | 0.9835 | (-0.5,0) weak | 1.1628 | 0.9837 | 255 | 0.0111 | 0.0085 | 54/57/50 | no | 0.60 | 0.68 | 0.9600 | 0/0 | [p1](images/09-mixed-document/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.4838 | 0.9503 | (-0.5,38.5) | 1.8537 | 0.9690 | 255 | 0.0174 | 0.0146 | 54/54/45 | no | 2.45 | 31.76 | 0.9778 | 0/0 | - |
| 09-mixed-document | xelatex | exact | 1/1 | ok | 1.7368 | 0.9712 | (0,-1) moderate | 1.6157 | 0.9743 | 255 | 0.0136 | 0.0112 | 54/54/54 | yes | 28.28 | 0.66 | 0.9630 | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | ok | 2.5096 | 0.9498 | (0,38.5) | 1.8378 | 0.9690 | 255 | 0.0175 | 0.0147 | 54/56/43 | no | 3.18 | 32.26 | 0.9767 | 0/0 | - |
| 09-mixed-document | xelatex | pipeline | 1/1 | ok | 1.9462 | 0.9703 | (1,-1) moderate | 1.7686 | 0.9732 | 255 | 0.0142 | 0.0119 | 54/57/50 | no | 30.69 | 0.62 | 0.9200 | 0/0 | - |
| 09-mixed-document | xelatex-lm | de1020c | 1/1 | ok | 2.2539 | 0.9494 | (-0.5,39) moderate | 1.9016 | 0.9641 | 255 | 0.0169 | 0.0140 | 54/54/45 | no | 30.16 | 32.06 | 0.9333 | 0/0 | - |
| 09-mixed-document | xelatex-lm | exact | 1/1 | ok | 0.4909 | 0.9918 | (0,-0.5) moderate | 0.3815 | 0.9955 | 255 | 0.0086 | 0.0043 | 54/54/54 | yes | 0.01 | 0.65 | 1.0000 | 0/0 | - |
| 09-mixed-document | xelatex-lm | main | 1/1 | ok | 2.2680 | 0.9490 | (-0.5,39) moderate | 1.8972 | 0.9637 | 255 | 0.0169 | 0.0141 | 54/56/43 | no | 32.35 | 32.55 | 0.9302 | 0/0 | - |
| 09-mixed-document | xelatex-lm | pipeline | 1/1 | ok | 1.3478 | 0.9800 | (-0.5,-0.5) moderate | 1.2414 | 0.9824 | 255 | 0.0119 | 0.0095 | 54/57/50 | no | 0.59 | 0.30 | 0.9600 | 0/0 | - |
| 10-unicode-paragraph | lualatex | de1020c | 1/1 | ok | 3.3153 | 0.9490 | (0,0) | 3.3153 | 0.9490 | 255 | 0.0261 | 0.0212 | 8/81/0 | no | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | exact | 1/1 | recovered | 3.1816 | 0.9455 | (0,0) | 3.1816 | 0.9455 | 255 | 0.0258 | 0.0209 | 8/84/0 | no | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex | main | 1/1 | recovered | 3.3305 | 0.9495 | (0.5,0) weak | 3.2478 | 0.9509 | 255 | 0.0262 | 0.0213 | 8/82/0 | no | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex | pipeline | 1/1 | recovered | 3.3978 | 0.9472 | (0,0) | 3.3978 | 0.9472 | 255 | 0.0265 | 0.0214 | 8/81/0 | no | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex-lm | de1020c | 1/1 | ok | 3.2356 | 0.9447 | (0,0) | 3.2356 | 0.9447 | 255 | 0.0259 | 0.0211 | 82/81/79 | no | 64.10 | 1.33 | 0.8987 | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | exact | 1/1 | recovered | 0.0550 | 0.9999 | (0,0) | 0.0550 | 0.9999 | 26 | 0.0105 | 0.0000 | 82/84/78 | no | 0.02 | 0.00 | 1.0000 | 0/0 | - |
| 10-unicode-paragraph | lualatex-lm | main | 1/1 | recovered | 3.2488 | 0.9448 | (0,0) | 3.2488 | 0.9448 | 255 | 0.0259 | 0.0212 | 82/82/81 | no | 104.26 | 2.38 | 0.8765 | 0/0 | - |
| 10-unicode-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 2.3178 | 0.9657 | (-0.5,0) moderate | 2.1870 | 0.9680 | 255 | 0.0218 | 0.0166 | 82/81/81 | no | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 10-unicode-paragraph | pdflatex | de1020c | 1/1 | ok | 3.3620 | 0.9486 | (0,0) | 3.3620 | 0.9486 | 255 | 0.0262 | 0.0213 | 85/81/76 | no | 75.04 | 1.87 | 0.9211 | 3/1 | [p1](images/10-unicode-paragraph/pdflatex-de1020c-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex | exact | 1/1 | recovered | 3.1919 | 0.9456 | (0,0) | 3.1919 | 0.9456 | 255 | 0.0258 | 0.0209 | 85/84/74 | no | 118.73 | 2.91 | 0.9054 | 3/0 | [p1](images/10-unicode-paragraph/pdflatex-exact-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex | main | 1/1 | recovered | 3.3173 | 0.9499 | (0,0) | 3.3173 | 0.9499 | 255 | 0.0261 | 0.0211 | 85/82/78 | no | 32.01 | 0.90 | 0.9231 | 3/0 | [p1](images/10-unicode-paragraph/pdflatex-main-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex | pipeline | 1/1 | recovered | 3.3570 | 0.9479 | (0,0) | 3.3570 | 0.9479 | 255 | 0.0263 | 0.0211 | 85/81/77 | no | 121.16 | 2.66 | 0.9091 | 3/0 | [p1](images/10-unicode-paragraph/pdflatex-pipeline-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 3.2510 | 0.9445 | (0,0) | 3.2510 | 0.9445 | 255 | 0.0260 | 0.0212 | 82/81/80 | no | 65.00 | 1.32 | 0.9000 | 0/1 | [p1](images/10-unicode-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | exact | 1/1 | recovered | 0.1071 | 0.9980 | (0,0) | 0.1071 | 0.9980 | 255 | 0.0072 | 0.0006 | 82/84/78 | no | 0.42 | 0.00 | 1.0000 | 0/0 | [p1](images/10-unicode-paragraph/pdflatex-lm-exact-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | main | 1/1 | recovered | 3.2644 | 0.9446 | (0,0) | 3.2644 | 0.9446 | 255 | 0.0260 | 0.0213 | 82/82/82 | yes | 104.99 | 2.35 | 0.8780 | 0/0 | [p1](images/10-unicode-paragraph/pdflatex-lm-main-export-p1-overlay.png) |
| 10-unicode-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 2.3844 | 0.9643 | (-0.5,0) moderate | 2.2190 | 0.9672 | 255 | 0.0221 | 0.0169 | 82/81/81 | no | 0.40 | 0.36 | 1.0000 | 0/0 | [p1](images/10-unicode-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 10-unicode-paragraph | xelatex | de1020c | 1/1 | ok | 3.3629 | 0.9483 | (0,0) | 3.3629 | 0.9483 | 255 | 0.0262 | 0.0213 | 84/81/78 | no | 78.17 | 2.01 | 0.9231 | 3/1 | - |
| 10-unicode-paragraph | xelatex | exact | 1/1 | recovered | 3.1976 | 0.9453 | (0,0) | 3.1976 | 0.9453 | 255 | 0.0258 | 0.0209 | 84/84/76 | no | 126.06 | 3.24 | 0.8947 | 3/0 | - |
| 10-unicode-paragraph | xelatex | main | 1/1 | recovered | 3.3247 | 0.9496 | (0.5,0) weak | 3.2333 | 0.9511 | 255 | 0.0261 | 0.0212 | 84/82/80 | no | 31.11 | 0.89 | 0.9250 | 3/0 | - |
| 10-unicode-paragraph | xelatex | pipeline | 1/1 | recovered | 3.4005 | 0.9470 | (0,0) | 3.4005 | 0.9470 | 255 | 0.0265 | 0.0214 | 84/81/79 | no | 128.14 | 2.97 | 0.8987 | 3/0 | - |
| 10-unicode-paragraph | xelatex-lm | de1020c | 1/1 | ok | 3.2358 | 0.9447 | (0,0) | 3.2358 | 0.9447 | 255 | 0.0259 | 0.0211 | 82/81/79 | no | 64.10 | 1.45 | 0.8987 | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | exact | 1/1 | recovered | 0.0548 | 0.9999 | (0,0) | 0.0548 | 0.9999 | 26 | 0.0105 | 0.0000 | 82/84/78 | no | 0.01 | 1.03 | 1.0000 | 0/0 | - |
| 10-unicode-paragraph | xelatex-lm | main | 1/1 | recovered | 3.2489 | 0.9448 | (0,0) | 3.2489 | 0.9448 | 255 | 0.0259 | 0.0212 | 82/82/81 | no | 104.26 | 2.42 | 0.8765 | 0/0 | - |
| 10-unicode-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 2.3184 | 0.9657 | (-0.5,0) moderate | 2.1870 | 0.9680 | 255 | 0.0218 | 0.0166 | 82/81/81 | no | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex | de1020c | 1/1 | recovered | 1.4653 | 0.9696 | (18,-49) weak | 1.4605 | 0.9709 | 255 | 0.0104 | 0.0085 | 29/24/24 | no | 123.00 | 61.96 | 0.9167 | 0/0 | - |
| 11-nested-lists | lualatex | exact | 1/1 | ok | 1.3325 | 0.9699 | (-18,-20) weak | 1.2704 | 0.9742 | 255 | 0.0100 | 0.0083 | 29/29/29 | yes | 18.47 | 26.36 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex | main | 1/1 | ok | 1.4714 | 0.9699 | (-22.5,-8) moderate | 1.1220 | 0.9800 | 255 | 0.0105 | 0.0087 | 29/29/29 | yes | 23.37 | 9.35 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex | pipeline | 1/1 | ok | 1.4785 | 0.9703 | (-16.5,-20) moderate | 1.3614 | 0.9752 | 255 | 0.0104 | 0.0087 | 29/29/29 | yes | 18.47 | 26.65 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (-2,-49) weak | 1.3067 | 0.9711 | 255 | 0.0100 | 0.0083 | 29/24/24 | no | 121.01 | 62.61 | 0.9167 | 0/0 | - |
| 11-nested-lists | lualatex-lm | exact | 1/1 | ok | 1.1456 | 0.9705 | (-17,-20) moderate | 0.9509 | 0.9783 | 255 | 0.0092 | 0.0074 | 29/29/29 | yes | 20.01 | 26.97 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex-lm | main | 1/1 | ok | 1.3743 | 0.9689 | (-30.5,-8) moderate | 1.2508 | 0.9755 | 255 | 0.0102 | 0.0085 | 29/29/29 | yes | 25.69 | 10.02 | 1.0000 | 0/0 | - |
| 11-nested-lists | lualatex-lm | pipeline | 1/1 | ok | 1.3304 | 0.9702 | (-17,-20) moderate | 1.1995 | 0.9761 | 255 | 0.0100 | 0.0083 | 29/29/29 | yes | 20.01 | 27.33 | 1.0000 | 0/0 | - |
| 11-nested-lists | pdflatex | de1020c | 1/1 | recovered | 1.4657 | 0.9698 | (19,-49) weak | 1.4558 | 0.9711 | 255 | 0.0104 | 0.0085 | 29/24/24 | no | 123.27 | 61.96 | 0.9167 | 0/0 | [p1](images/11-nested-lists/pdflatex-de1020c-export-p1-overlay.png) |
| 11-nested-lists | pdflatex | exact | 1/1 | ok | 1.3374 | 0.9701 | (-17,-20) moderate | 1.2620 | 0.9748 | 255 | 0.0100 | 0.0083 | 29/29/29 | yes | 17.80 | 26.36 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-exact-export-p1-overlay.png) |
| 11-nested-lists | pdflatex | main | 1/1 | ok | 1.4604 | 0.9704 | (-22,-8) moderate | 1.1824 | 0.9790 | 255 | 0.0105 | 0.0086 | 29/29/29 | yes | 22.70 | 9.35 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-main-export-p1-overlay.png) |
| 11-nested-lists | pdflatex | pipeline | 1/1 | ok | 1.4479 | 0.9709 | (-16,-20) moderate | 1.3660 | 0.9757 | 255 | 0.0103 | 0.0085 | 29/29/29 | yes | 17.80 | 26.65 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-pipeline-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | de1020c | 1/1 | recovered | 1.3674 | 0.9687 | (23.5,-49) weak | 1.3216 | 0.9706 | 255 | 0.0101 | 0.0083 | 29/24/24 | no | 121.22 | 62.61 | 0.9167 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | exact | 1/1 | ok | 1.1360 | 0.9705 | (-16.5,-20) moderate | 0.9949 | 0.9774 | 255 | 0.0091 | 0.0074 | 29/29/29 | yes | 19.39 | 26.97 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-exact-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | main | 1/1 | ok | 1.3761 | 0.9690 | (-30,-8) moderate | 1.2546 | 0.9752 | 255 | 0.0102 | 0.0085 | 29/29/29 | yes | 25.08 | 10.02 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-main-export-p1-overlay.png) |
| 11-nested-lists | pdflatex-lm | pipeline | 1/1 | ok | 1.3364 | 0.9701 | (-17,-20) moderate | 1.1999 | 0.9760 | 255 | 0.0100 | 0.0083 | 29/29/29 | yes | 19.39 | 27.33 | 1.0000 | 0/0 | [p1](images/11-nested-lists/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 11-nested-lists | xelatex | de1020c | 1/1 | recovered | 1.4648 | 0.9696 | (18,-49) weak | 1.4600 | 0.9709 | 255 | 0.0104 | 0.0085 | 29/24/24 | no | 123.00 | 61.96 | 0.9167 | 0/0 | - |
| 11-nested-lists | xelatex | exact | 1/1 | ok | 1.3324 | 0.9699 | (-18,-20) weak | 1.2710 | 0.9742 | 255 | 0.0100 | 0.0083 | 29/29/29 | yes | 18.46 | 26.36 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex | main | 1/1 | ok | 1.4708 | 0.9699 | (-22.5,-8) moderate | 1.1192 | 0.9800 | 255 | 0.0105 | 0.0087 | 29/29/29 | yes | 23.37 | 9.35 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex | pipeline | 1/1 | ok | 1.4783 | 0.9703 | (-16.5,-20) moderate | 1.3609 | 0.9752 | 255 | 0.0104 | 0.0087 | 29/29/29 | yes | 18.47 | 26.65 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (-2,-49) weak | 1.3068 | 0.9711 | 255 | 0.0100 | 0.0083 | 29/24/24 | no | 121.01 | 61.76 | 0.9167 | 0/0 | - |
| 11-nested-lists | xelatex-lm | exact | 1/1 | ok | 1.1456 | 0.9705 | (-17,-20) moderate | 0.9497 | 0.9783 | 255 | 0.0092 | 0.0074 | 29/29/29 | yes | 20.01 | 26.15 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex-lm | main | 1/1 | ok | 1.3743 | 0.9689 | (-30.5,-8) moderate | 1.2509 | 0.9755 | 255 | 0.0102 | 0.0085 | 29/29/29 | yes | 25.69 | 9.15 | 1.0000 | 0/0 | - |
| 11-nested-lists | xelatex-lm | pipeline | 1/1 | ok | 1.3305 | 0.9702 | (-17,-20) moderate | 1.1997 | 0.9761 | 255 | 0.0100 | 0.0083 | 29/29/29 | yes | 20.01 | 26.44 | 1.0000 | 0/0 | - |
| 12-justified-paragraphs | lualatex | de1020c | 1/1 | ok | 15.4038 | 0.7396 | (-3,0) weak | 15.3872 | 0.7397 | 255 | 0.1111 | 0.0927 | 360/360/360 | yes | 99.24 | 32.55 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex | exact | 1/1 | ok | 12.0437 | 0.7870 | (0,29) weak | 12.0040 | 0.7893 | 255 | 0.0959 | 0.0781 | 360/360/360 | yes | 155.86 | 25.80 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex | main | 1/1 | ok | 14.8416 | 0.7604 | (-3.5,0) weak | 14.8082 | 0.7605 | 255 | 0.1082 | 0.0900 | 360/360/360 | yes | 72.49 | 10.31 | 0.9111 | 0/0 | - |
| 12-justified-paragraphs | lualatex | pipeline | 1/1 | ok | 12.9681 | 0.7908 | (0,0) | 12.9681 | 0.7908 | 255 | 0.0987 | 0.0805 | 360/360/360 | yes | 155.86 | 25.45 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | de1020c | 1/1 | ok | 13.7995 | 0.7503 | (-8.5,2.5) weak | 13.7712 | 0.7503 | 255 | 0.1052 | 0.0873 | 360/360/360 | yes | 83.76 | 8.29 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | exact | 1/1 | ok | 0.1236 | 0.9999 | (0,0) | 0.1236 | 0.9999 | 13 | 0.0362 | 0.0000 | 360/360/360 | yes | 0.01 | 0.00 | 1.0000 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | main | 1/1 | ok | 13.7762 | 0.7469 | (-0.5,-26) weak | 13.7623 | 0.7495 | 255 | 0.1054 | 0.0872 | 360/360/360 | yes | 109.31 | 15.49 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | pipeline | 1/1 | ok | 7.7969 | 0.8919 | (0,0) | 7.7969 | 0.8919 | 255 | 0.0766 | 0.0576 | 360/360/360 | yes | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 12-justified-paragraphs | pdflatex | de1020c | 1/1 | ok | 15.3708 | 0.7415 | (-3,0) weak | 15.3659 | 0.7414 | 255 | 0.1108 | 0.0925 | 360/360/360 | yes | 99.31 | 32.55 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-de1020c-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex | exact | 1/1 | ok | 12.0428 | 0.7880 | (0,29) weak | 11.9747 | 0.7908 | 255 | 0.0956 | 0.0779 | 360/360/360 | yes | 155.88 | 25.80 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-exact-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex | main | 1/1 | ok | 14.8933 | 0.7603 | (-4,0) weak | 14.8302 | 0.7612 | 255 | 0.1079 | 0.0901 | 360/360/360 | yes | 72.58 | 10.31 | 0.9111 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-main-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex | pipeline | 1/1 | ok | 13.0291 | 0.7911 | (1,14.5) weak | 12.9218 | 0.7936 | 255 | 0.0986 | 0.0807 | 360/360/360 | yes | 155.88 | 25.45 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-pipeline-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | de1020c | 1/1 | ok | 13.7992 | 0.7503 | (-8.5,2.5) weak | 13.7701 | 0.7503 | 255 | 0.1051 | 0.0873 | 360/360/360 | yes | 83.76 | 8.99 | 0.8889 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | exact | 1/1 | ok | 0.0418 | 1.0000 | (0,0) | 0.0418 | 1.0000 | 5 | 0.0235 | 0.0000 | 360/360/360 | yes | 0.00 | 1.03 | 1.0000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-exact-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | main | 1/1 | ok | 13.7769 | 0.7469 | (-0.5,0) weak | 13.6913 | 0.7482 | 255 | 0.1054 | 0.0872 | 360/360/360 | yes | 109.32 | 14.74 | 0.9000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-main-export-p1-overlay.png) |
| 12-justified-paragraphs | pdflatex-lm | pipeline | 1/1 | ok | 7.8132 | 0.8917 | (0,0) | 7.8132 | 0.8917 | 255 | 0.0767 | 0.0578 | 360/360/360 | yes | 0.00 | 0.67 | 1.0000 | 0/0 | [p1](images/12-justified-paragraphs/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 12-justified-paragraphs | xelatex | de1020c | 1/1 | ok | 15.3809 | 0.7404 | (0,0) | 15.3809 | 0.7404 | 255 | 0.1111 | 0.0928 | 360/360/360 | yes | 106.70 | 32.71 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex | exact | 1/1 | ok | 12.0683 | 0.7868 | (0,29) weak | 12.0273 | 0.7890 | 255 | 0.0961 | 0.0784 | 360/360/360 | yes | 159.34 | 25.97 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex | main | 1/1 | ok | 14.9399 | 0.7591 | (0,0) | 14.9399 | 0.7591 | 255 | 0.1085 | 0.0906 | 360/360/360 | yes | 81.84 | 10.47 | 0.9111 | 0/0 | - |
| 12-justified-paragraphs | xelatex | pipeline | 1/1 | ok | 13.0463 | 0.7899 | (0.5,14.5) weak | 12.9774 | 0.7921 | 255 | 0.0990 | 0.0810 | 360/360/360 | yes | 159.34 | 25.61 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | de1020c | 1/1 | ok | 13.7994 | 0.7503 | (-8.5,2.5) weak | 13.7708 | 0.7503 | 255 | 0.1051 | 0.0873 | 360/360/360 | yes | 83.76 | 8.99 | 0.8889 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | exact | 1/1 | ok | 0.1288 | 0.9999 | (0,0) | 0.1288 | 0.9999 | 14 | 0.0363 | 0.0000 | 360/360/360 | yes | 0.01 | 1.03 | 1.0000 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | main | 1/1 | ok | 13.7765 | 0.7469 | (-1,-26) weak | 13.7693 | 0.7493 | 255 | 0.1054 | 0.0872 | 360/360/360 | yes | 109.31 | 14.74 | 0.9000 | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | pipeline | 1/1 | ok | 7.7970 | 0.8919 | (0,0) | 7.7970 | 0.8919 | 255 | 0.0766 | 0.0576 | 360/360/360 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 13-math-display-rich | lualatex | de1020c | 1/1 | recovered | 0.4724 | 0.9903 | (0,3.5) moderate | 0.4325 | 0.9914 | 255 | 0.0040 | 0.0031 | 19/17/7 | no | 1.15 | 5.89 | 1.0000 | 2/2 | - |
| 13-math-display-rich | lualatex | exact | 1/1 | recovered | 0.5252 | 0.9884 | (-1.5,0) weak | 0.5152 | 0.9887 | 255 | 0.0043 | 0.0035 | 19/27/15 | no | 13.33 | 0.48 | 0.9333 | 2/2 | - |
| 13-math-display-rich | lualatex | main | 1/1 | recovered | 0.4988 | 0.9897 | (-0.5,3.5) weak | 0.4849 | 0.9904 | 255 | 0.0041 | 0.0032 | 19/17/7 | no | 1.78 | 5.89 | 1.0000 | 2/0 | - |
| 13-math-display-rich | lualatex | pipeline | 1/1 | recovered | 0.5457 | 0.9886 | (0,0) | 0.5457 | 0.9886 | 255 | 0.0043 | 0.0035 | 19/32/15 | no | 12.98 | 0.58 | 0.8667 | 2/2 | - |
| 13-math-display-rich | lualatex-lm | de1020c | 1/1 | recovered | 0.5515 | 0.9887 | (-2,4) weak | 0.5477 | 0.9892 | 255 | 0.0043 | 0.0035 | 19/17/7 | no | 3.38 | 5.45 | 1.0000 | 4/2 | - |
| 13-math-display-rich | lualatex-lm | exact | 1/1 | recovered | 0.3458 | 0.9917 | (0,0) | 0.3458 | 0.9917 | 255 | 0.0034 | 0.0025 | 19/27/15 | no | 12.20 | 0.33 | 0.9333 | 4/2 | - |
| 13-math-display-rich | lualatex-lm | main | 1/1 | recovered | 0.5520 | 0.9884 | (-0.5,3.5) weak | 0.5303 | 0.9890 | 255 | 0.0044 | 0.0035 | 19/17/7 | no | 4.16 | 5.45 | 1.0000 | 4/0 | - |
| 13-math-display-rich | lualatex-lm | pipeline | 1/1 | recovered | 0.5142 | 0.9891 | (-0.5,0) moderate | 0.4599 | 0.9903 | 255 | 0.0042 | 0.0034 | 19/32/15 | no | 11.85 | 0.78 | 0.8667 | 4/2 | - |
| 13-math-display-rich | pdflatex | de1020c | 1/1 | recovered | 0.5010 | 0.9899 | (1,3.5) weak | 0.4808 | 0.9907 | 255 | 0.0041 | 0.0033 | 19/17/7 | no | 1.58 | 6.00 | 1.0000 | 2/2 | [p1](images/13-math-display-rich/pdflatex-de1020c-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex | exact | 1/1 | recovered | 0.5343 | 0.9882 | (-1.5,0) weak | 0.5136 | 0.9887 | 255 | 0.0044 | 0.0036 | 19/27/15 | no | 13.53 | 0.53 | 0.9333 | 2/2 | [p1](images/13-math-display-rich/pdflatex-exact-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex | main | 1/1 | recovered | 0.4817 | 0.9900 | (0,3.5) weak | 0.4734 | 0.9904 | 255 | 0.0041 | 0.0032 | 19/17/7 | no | 1.50 | 6.00 | 1.0000 | 2/0 | [p1](images/13-math-display-rich/pdflatex-main-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex | pipeline | 1/1 | recovered | 0.5415 | 0.9885 | (0,0) | 0.5415 | 0.9885 | 255 | 0.0043 | 0.0034 | 19/32/15 | no | 13.18 | 0.62 | 0.8667 | 2/2 | [p1](images/13-math-display-rich/pdflatex-pipeline-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | de1020c | 1/1 | recovered | 0.5530 | 0.9888 | (-2,4) weak | 0.5406 | 0.9893 | 255 | 0.0043 | 0.0035 | 19/17/7 | no | 3.38 | 6.19 | 1.0000 | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | exact | 1/1 | recovered | 0.3074 | 0.9921 | (0,0) | 0.3074 | 0.9921 | 255 | 0.0030 | 0.0020 | 19/27/15 | no | 12.20 | 0.45 | 0.9333 | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-exact-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | main | 1/1 | recovered | 0.5543 | 0.9885 | (-0.5,4) weak | 0.5397 | 0.9888 | 255 | 0.0044 | 0.0036 | 19/17/7 | no | 4.17 | 6.19 | 1.0000 | 4/0 | [p1](images/13-math-display-rich/pdflatex-lm-main-export-p1-overlay.png) |
| 13-math-display-rich | pdflatex-lm | pipeline | 1/1 | recovered | 0.5070 | 0.9893 | (-0.5,0) moderate | 0.4487 | 0.9904 | 255 | 0.0041 | 0.0034 | 19/32/15 | no | 11.85 | 0.82 | 0.8667 | 4/2 | [p1](images/13-math-display-rich/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 13-math-display-rich | xelatex | de1020c | 1/1 | recovered | 0.4941 | 0.9899 | (0,0) | 0.4941 | 0.9899 | 255 | 0.0041 | 0.0032 | 21/17/8 | no | 1.56 | 2.84 | 1.0000 | 2/2 | - |
| 13-math-display-rich | xelatex | exact | 1/1 | recovered | 0.5315 | 0.9882 | (-1.5,0) weak | 0.5161 | 0.9886 | 255 | 0.0044 | 0.0036 | 21/27/13 | no | 10.99 | 2.09 | 0.9231 | 2/2 | - |
| 13-math-display-rich | xelatex | main | 1/1 | recovered | 0.4767 | 0.9899 | (0,3.5) moderate | 0.4525 | 0.9906 | 255 | 0.0041 | 0.0032 | 21/17/9 | no | 1.53 | 3.43 | 1.0000 | 2/0 | - |
| 13-math-display-rich | xelatex | pipeline | 1/1 | recovered | 0.5457 | 0.9884 | (0,0) | 0.5457 | 0.9884 | 255 | 0.0043 | 0.0035 | 21/32/14 | no | 11.27 | 3.78 | 0.7143 | 2/2 | - |
| 13-math-display-rich | xelatex-lm | de1020c | 1/1 | recovered | 0.5517 | 0.9887 | (-2,4) weak | 0.5476 | 0.9892 | 255 | 0.0043 | 0.0035 | 21/17/8 | no | 3.20 | 3.05 | 1.0000 | 4/2 | - |
| 13-math-display-rich | xelatex-lm | exact | 1/1 | recovered | 0.3460 | 0.9917 | (0,0) | 0.3460 | 0.9917 | 255 | 0.0034 | 0.0025 | 21/27/13 | no | 9.50 | 2.14 | 0.9231 | 4/2 | - |
| 13-math-display-rich | xelatex-lm | main | 1/1 | recovered | 0.5522 | 0.9884 | (-0.5,3.5) weak | 0.5303 | 0.9890 | 255 | 0.0044 | 0.0036 | 21/17/9 | no | 3.64 | 3.61 | 1.0000 | 4/0 | - |
| 13-math-display-rich | xelatex-lm | pipeline | 1/1 | recovered | 0.5144 | 0.9891 | (-0.5,0) moderate | 0.4600 | 0.9903 | 255 | 0.0042 | 0.0034 | 21/32/14 | no | 9.89 | 3.88 | 0.7143 | 4/2 | - |
| 14-math-inline-dense | lualatex | de1020c | 1/1 | recovered | 1.3179 | 0.9705 | (-34.5,13.5) | 0.9604 | 0.9783 | 255 | 0.0099 | 0.0082 | 62/59/25 | no | 52.74 | 10.98 | 0.9600 | 2/1 | - |
| 14-math-inline-dense | lualatex | exact | 1/1 | recovered | 0.9576 | 0.9792 | (1,0) weak | 0.9281 | 0.9797 | 255 | 0.0082 | 0.0066 | 62/67/50 | no | 40.90 | 1.06 | 0.9400 | 2/2 | - |
| 14-math-inline-dense | lualatex | main | 1/1 | recovered | 1.3111 | 0.9706 | (-34.5,13.5) | 0.9460 | 0.9784 | 255 | 0.0099 | 0.0081 | 62/57/26 | no | 42.13 | 10.69 | 1.0000 | 2/0 | - |
| 14-math-inline-dense | lualatex | pipeline | 1/1 | recovered | 0.9874 | 0.9807 | (0,0) | 0.9874 | 0.9807 | 255 | 0.0082 | 0.0065 | 62/72/46 | no | 15.72 | 0.71 | 0.9565 | 2/2 | - |
| 14-math-inline-dense | lualatex-lm | de1020c | 1/1 | recovered | 1.2473 | 0.9702 | (5,13.5) moderate | 1.1675 | 0.9735 | 255 | 0.0097 | 0.0079 | 62/59/23 | no | 60.36 | 10.71 | 0.9565 | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | exact | 1/1 | recovered | 0.4203 | 0.9900 | (0,0) | 0.4203 | 0.9900 | 255 | 0.0056 | 0.0030 | 62/67/33 | no | 0.16 | 0.00 | 1.0000 | 2/2 | - |
| 14-math-inline-dense | lualatex-lm | main | 1/1 | recovered | 1.2444 | 0.9703 | (5,13.5) moderate | 1.1626 | 0.9734 | 255 | 0.0096 | 0.0079 | 62/57/24 | no | 46.12 | 10.02 | 1.0000 | 2/0 | - |
| 14-math-inline-dense | lualatex-lm | pipeline | 1/1 | recovered | 0.7827 | 0.9851 | (0,0) | 0.7827 | 0.9851 | 255 | 0.0072 | 0.0055 | 62/72/49 | no | 1.98 | 0.60 | 1.0000 | 2/2 | - |
| 14-math-inline-dense | pdflatex | de1020c | 1/1 | recovered | 1.3129 | 0.9708 | (-34,13.5) | 0.9608 | 0.9783 | 255 | 0.0099 | 0.0082 | 53/59/18 | no | 56.56 | 13.98 | 0.8889 | 2/1 | [p1](images/14-math-inline-dense/pdflatex-de1020c-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex | exact | 1/1 | recovered | 0.9552 | 0.9792 | (0.5,0) weak | 0.9102 | 0.9801 | 255 | 0.0082 | 0.0065 | 53/67/26 | no | 36.82 | 0.81 | 0.8846 | 2/2 | [p1](images/14-math-inline-dense/pdflatex-exact-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex | main | 1/1 | recovered | 1.3054 | 0.9708 | (-34,13.5) | 0.9332 | 0.9785 | 255 | 0.0099 | 0.0081 | 53/57/19 | no | 48.76 | 11.30 | 1.0000 | 2/0 | [p1](images/14-math-inline-dense/pdflatex-main-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex | pipeline | 1/1 | recovered | 0.9751 | 0.9808 | (0,0) | 0.9751 | 0.9808 | 255 | 0.0081 | 0.0064 | 53/72/30 | no | 17.00 | 0.78 | 0.9333 | 2/2 | [p1](images/14-math-inline-dense/pdflatex-pipeline-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | de1020c | 1/1 | recovered | 1.2469 | 0.9702 | (5,13.5) moderate | 1.1673 | 0.9735 | 255 | 0.0097 | 0.0079 | 55/59/19 | no | 64.42 | 13.17 | 0.8947 | 2/1 | [p1](images/14-math-inline-dense/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | exact | 1/1 | recovered | 0.4137 | 0.9900 | (0,0) | 0.4137 | 0.9900 | 255 | 0.0051 | 0.0030 | 55/67/29 | no | 0.17 | 0.36 | 1.0000 | 2/2 | [p1](images/14-math-inline-dense/pdflatex-lm-exact-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | main | 1/1 | recovered | 1.2440 | 0.9703 | (5,13.5) moderate | 1.1624 | 0.9734 | 255 | 0.0096 | 0.0079 | 55/57/19 | no | 48.95 | 10.98 | 1.0000 | 2/0 | [p1](images/14-math-inline-dense/pdflatex-lm-main-export-p1-overlay.png) |
| 14-math-inline-dense | pdflatex-lm | pipeline | 1/1 | recovered | 0.7836 | 0.9851 | (0,0) | 0.7836 | 0.9851 | 255 | 0.0072 | 0.0055 | 55/72/37 | no | 1.47 | 0.81 | 1.0000 | 2/2 | [p1](images/14-math-inline-dense/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 14-math-inline-dense | xelatex | de1020c | 1/1 | recovered | 1.3179 | 0.9705 | (-34.5,13.5) | 0.9575 | 0.9783 | 255 | 0.0099 | 0.0082 | 63/59/25 | no | 52.75 | 12.18 | 0.9600 | 2/1 | - |
| 14-math-inline-dense | xelatex | exact | 1/1 | recovered | 0.9581 | 0.9792 | (7.5,0) weak | 0.9392 | 0.9800 | 255 | 0.0082 | 0.0066 | 63/67/32 | no | 34.53 | 0.66 | 0.9062 | 2/2 | - |
| 14-math-inline-dense | xelatex | main | 1/1 | recovered | 1.3112 | 0.9706 | (-34.5,13.5) | 0.9422 | 0.9785 | 255 | 0.0099 | 0.0081 | 63/57/26 | no | 42.13 | 11.84 | 1.0000 | 2/0 | - |
| 14-math-inline-dense | xelatex | pipeline | 1/1 | recovered | 0.9874 | 0.9807 | (0,0) | 0.9874 | 0.9807 | 255 | 0.0082 | 0.0065 | 63/72/45 | no | 15.92 | 5.92 | 0.9556 | 2/2 | - |
| 14-math-inline-dense | xelatex-lm | de1020c | 1/1 | recovered | 1.2473 | 0.9702 | (5,13.5) moderate | 1.1675 | 0.9735 | 255 | 0.0097 | 0.0079 | 63/59/23 | no | 60.36 | 11.58 | 0.9565 | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | exact | 1/1 | recovered | 0.4203 | 0.9900 | (0,0) | 0.4203 | 0.9900 | 255 | 0.0056 | 0.0030 | 63/67/33 | no | 0.16 | 0.31 | 1.0000 | 2/2 | - |
| 14-math-inline-dense | xelatex-lm | main | 1/1 | recovered | 1.2444 | 0.9703 | (5,13.5) moderate | 1.1627 | 0.9734 | 255 | 0.0096 | 0.0079 | 63/57/24 | no | 46.12 | 11.02 | 1.0000 | 2/0 | - |
| 14-math-inline-dense | xelatex-lm | pipeline | 1/1 | recovered | 0.7826 | 0.9851 | (0,0) | 0.7826 | 0.9851 | 255 | 0.0072 | 0.0055 | 63/72/48 | no | 1.91 | 5.64 | 1.0000 | 2/2 | - |
| 15-three-page-sections | lualatex | de1020c | 3/3 | recovered | 26.4239 | 0.5635 | (0.5,20) moderate; (0.5,19) weak; (0.5,-24.5) moderate | 25.2158 | 0.5876 | 255 | 0.1894 | 0.1584 | 1806/1806/1806 | yes | 154.50 | 39.85 | 0.8984 | 0/0 | - |
| 15-three-page-sections | lualatex | exact | 3/3 | recovered | 20.1637 | 0.6591 | (0,43) weak; (0,43) weak; (0,43) weak | 19.4451 | 0.6757 | 255 | 0.1605 | 0.1315 | 1806/1806/1806 | yes | 179.40 | 27.66 | 0.9003 | 0/3 | - |
| 15-three-page-sections | lualatex | main | 3/3 | recovered | 26.2989 | 0.5653 | (0,20) moderate; (0,19) weak; (0,-24.5) weak | 25.1064 | 0.5895 | 255 | 0.1894 | 0.1581 | 1806/1809/1806 | no | 132.74 | 39.82 | 0.8978 | 0/0 | - |
| 15-three-page-sections | lualatex | pipeline | 3/3 | recovered | 21.9942 | 0.6558 | (0.5,14) weak; (0.5,14) weak; (0.5,14) weak | 21.0468 | 0.6753 | 255 | 0.1659 | 0.1373 | 1806/1809/1806 | no | 179.49 | 27.32 | 0.8987 | 0/0 | - |
| 15-three-page-sections | lualatex-lm | de1020c | 3/3 | recovered | 24.2612 | 0.5493 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0398 | 0.5821 | 255 | 0.1832 | 0.1520 | 1806/1806/1806 | yes | 122.99 | 44.85 | 0.8926 | 3/0 | - |
| 15-three-page-sections | lualatex-lm | exact | 3/3 | recovered | 0.3443 | 0.9996 | (0,0); (0,0); (0,0) | 0.3443 | 0.9996 | 19 | 0.0777 | 0.0000 | 1806/1806/1806 | yes | 0.01 | 0.01 | 1.0000 | 3/3 | - |
| 15-three-page-sections | lualatex-lm | main | 3/3 | recovered | 24.2777 | 0.5494 | (-0.5,22.5) moderate; (-0.5,-24) weak; (0,-24) weak | 22.9599 | 0.5839 | 255 | 0.1836 | 0.1521 | 1806/1809/1806 | no | 146.92 | 45.04 | 0.8919 | 3/0 | - |
| 15-three-page-sections | lualatex-lm | pipeline | 3/3 | recovered | 13.1412 | 0.8175 | (0,0); (0,0); (0,0) | 13.1412 | 0.8175 | 255 | 0.1288 | 0.0972 | 1806/1809/1806 | no | 0.11 | 0.37 | 0.9983 | 3/0 | - |
| 15-three-page-sections | pdflatex | de1020c | 3/3 | recovered | 26.6994 | 0.5541 | (0.5,36.5) moderate; (0,-24.5) moderate; (0,-24.5) weak | 25.0916 | 0.5919 | 255 | 0.1902 | 0.1599 | 1806/1806/1806 | yes | 136.77 | 43.12 | 0.8914 | 0/0 | [p1](images/15-three-page-sections/pdflatex-de1020c-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex | exact | 3/3 | recovered | 20.1271 | 0.6652 | (0,-0.5) weak; (0,-0.5) weak; (0,-0.5) weak | 19.3955 | 0.6800 | 255 | 0.1597 | 0.1314 | 1806/1806/1806 | yes | 184.24 | 5.32 | 0.8937 | 0/3 | [p1](images/15-three-page-sections/pdflatex-exact-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex | main | 3/3 | recovered | 26.7472 | 0.5539 | (0,36.5) moderate; (0,-24.5) moderate; (0,-24.5) moderate | 25.0604 | 0.5936 | 255 | 0.1907 | 0.1602 | 1806/1809/1806 | no | 109.74 | 43.31 | 0.8908 | 0/0 | [p1](images/15-three-page-sections/pdflatex-main-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex | pipeline | 3/3 | recovered | 21.7448 | 0.6643 | (1,-0.5) moderate; (1,-0.5) moderate; (1,-0.5) moderate | 20.5622 | 0.6872 | 255 | 0.1645 | 0.1363 | 1806/1809/1806 | no | 184.34 | 5.13 | 0.8920 | 0/0 | [p1](images/15-three-page-sections/pdflatex-pipeline-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | de1020c | 3/3 | recovered | 24.2615 | 0.5493 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0362 | 0.5822 | 255 | 0.1833 | 0.1521 | 1806/1806/1806 | yes | 123.00 | 44.54 | 0.8926 | 3/0 | [p1](images/15-three-page-sections/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | exact | 3/3 | recovered | 0.0736 | 1.0000 | (0,0); (0,0); (0,0) | 0.0736 | 1.0000 | 6 | 0.0416 | 0.0000 | 1806/1806/1806 | yes | 0.00 | 1.03 | 1.0000 | 3/3 | [p1](images/15-three-page-sections/pdflatex-lm-exact-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | main | 3/3 | recovered | 24.2787 | 0.5493 | (-0.5,22.5) moderate; (-0.5,-24) weak; (-0.5,-24) weak | 22.9548 | 0.5839 | 255 | 0.1836 | 0.1522 | 1806/1809/1806 | no | 146.92 | 44.73 | 0.8919 | 3/0 | [p1](images/15-three-page-sections/pdflatex-lm-main-export-p1-overlay.png) |
| 15-three-page-sections | pdflatex-lm | pipeline | 3/3 | recovered | 13.1578 | 0.8174 | (0,0); (0,0); (0,0) | 13.1578 | 0.8174 | 255 | 0.1290 | 0.0973 | 1806/1809/1806 | no | 0.10 | 0.67 | 0.9983 | 3/0 | [p1](images/15-three-page-sections/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 15-three-page-sections | xelatex | de1020c | 3/3 | recovered | 26.4202 | 0.5637 | (-3,20) moderate; (-3,19) weak; (-3,-24.5) moderate | 25.1892 | 0.5879 | 255 | 0.1893 | 0.1587 | 1806/1806/1806 | yes | 154.49 | 39.85 | 0.8984 | 0/0 | - |
| 15-three-page-sections | xelatex | exact | 3/3 | recovered | 20.1901 | 0.6582 | (0,43) weak; (0,43) weak; (0,43) weak | 19.4578 | 0.6752 | 255 | 0.1607 | 0.1320 | 1806/1806/1806 | yes | 179.39 | 27.66 | 0.9003 | 0/3 | - |
| 15-three-page-sections | xelatex | main | 3/3 | recovered | 26.3412 | 0.5649 | (0,20) moderate; (0,19) weak; (0,-24.5) weak | 25.1635 | 0.5888 | 255 | 0.1895 | 0.1586 | 1806/1809/1806 | no | 132.72 | 39.82 | 0.8978 | 0/0 | - |
| 15-three-page-sections | xelatex | pipeline | 3/3 | recovered | 22.0522 | 0.6548 | (1,14) weak; (1,14) weak; (1,14) weak | 20.9581 | 0.6761 | 255 | 0.1662 | 0.1378 | 1806/1809/1806 | no | 179.48 | 27.32 | 0.8987 | 0/0 | - |
| 15-three-page-sections | xelatex-lm | de1020c | 3/3 | recovered | 24.2611 | 0.5493 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0397 | 0.5821 | 255 | 0.1832 | 0.1519 | 1806/1806/1806 | yes | 122.99 | 44.55 | 0.8926 | 3/0 | - |
| 15-three-page-sections | xelatex-lm | exact | 3/3 | recovered | 0.3530 | 0.9996 | (0,0); (0,0); (0,0) | 0.3530 | 0.9996 | 20 | 0.0777 | 0.0000 | 1806/1806/1806 | yes | 0.01 | 1.02 | 1.0000 | 3/3 | - |
| 15-three-page-sections | xelatex-lm | main | 3/3 | recovered | 24.2775 | 0.5494 | (-0.5,22.5) moderate; (-0.5,-24) weak; (0,-24) weak | 22.9598 | 0.5839 | 255 | 0.1836 | 0.1521 | 1806/1809/1806 | no | 146.92 | 44.73 | 0.8919 | 3/0 | - |
| 15-three-page-sections | xelatex-lm | pipeline | 3/3 | recovered | 13.1454 | 0.8174 | (0,0); (0,0); (0,0) | 13.1454 | 0.8174 | 255 | 0.1288 | 0.0973 | 1806/1809/1806 | no | 0.11 | 0.66 | 0.9983 | 3/0 | - |
| 16-heading-page-break | lualatex | de1020c | 2/2 | ok | 20.5835 | 0.6612 | (0,0); (0,20.5) moderate | 20.0992 | 0.6687 | 255 | 0.1484 | 0.1240 | 962/962/962 | yes | 161.32 | 45.97 | 0.8950 | 0/0 | - |
| 16-heading-page-break | lualatex | exact | 2/2 | ok | 16.2511 | 0.7183 | (0,0); (-0.5,58) weak | 16.1365 | 0.7248 | 255 | 0.1292 | 0.1053 | 962/962/962 | yes | 162.13 | 36.89 | 0.8960 | 0/1 | - |
| 16-heading-page-break | lualatex | main | 2/2 | ok | 20.1428 | 0.6738 | (0,0); (0,14.5) weak | 20.0614 | 0.6772 | 255 | 0.1461 | 0.1217 | 962/963/962 | no | 135.98 | 15.96 | 0.9002 | 0/0 | - |
| 16-heading-page-break | lualatex | pipeline | 2/2 | ok | 17.5094 | 0.7212 | (0,0); (-1,57.5) weak | 17.3768 | 0.7259 | 255 | 0.1328 | 0.1087 | 962/963/962 | no | 162.19 | 36.53 | 0.8949 | 0/0 | - |
| 16-heading-page-break | lualatex-lm | de1020c | 2/2 | ok | 19.0181 | 0.6506 | (-1,0) weak; (-1,20) moderate | 18.0483 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/962/962 | yes | 137.29 | 13.93 | 0.8888 | 1/0 | - |
| 16-heading-page-break | lualatex-lm | exact | 2/2 | ok | 0.1903 | 0.9998 | (0,0); (0,0) | 0.1903 | 0.9998 | 25 | 0.0503 | 0.0000 | 962/962/962 | yes | 0.01 | 0.00 | 1.0000 | 1/1 | - |
| 16-heading-page-break | lualatex-lm | main | 2/2 | ok | 18.3183 | 0.6671 | (-0.5,-27) weak; (0,0) | 18.3034 | 0.6668 | 255 | 0.1402 | 0.1159 | 962/963/962 | no | 156.23 | 21.71 | 0.8951 | 1/0 | - |
| 16-heading-page-break | lualatex-lm | pipeline | 2/2 | ok | 10.5773 | 0.8525 | (0,0); (0,0) | 10.5773 | 0.8525 | 255 | 0.1036 | 0.0780 | 962/963/962 | no | 0.07 | 0.36 | 0.9990 | 1/0 | - |
| 16-heading-page-break | pdflatex | de1020c | 2/2 | ok | 20.5977 | 0.6620 | (0,-14.5) weak; (0,20.5) moderate | 20.0449 | 0.6704 | 255 | 0.1481 | 0.1240 | 962/962/962 | yes | 141.32 | 45.21 | 0.8952 | 0/0 | [p1](images/16-heading-page-break/pdflatex-de1020c-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex | exact | 2/2 | ok | 16.1697 | 0.7203 | (0,0); (0,58) weak | 16.0890 | 0.7264 | 255 | 0.1286 | 0.1049 | 962/962/962 | yes | 178.01 | 36.12 | 0.8962 | 0/1 | [p1](images/16-heading-page-break/pdflatex-exact-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex | main | 2/2 | ok | 20.1472 | 0.6751 | (0,0); (-0.5,14.5) weak | 20.0432 | 0.6786 | 255 | 0.1457 | 0.1216 | 962/963/962 | no | 111.12 | 15.21 | 0.9003 | 0/0 | [p1](images/16-heading-page-break/pdflatex-main-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex | pipeline | 2/2 | ok | 17.5005 | 0.7228 | (-1.5,0) weak; (-1.5,57.5) weak | 17.2555 | 0.7296 | 255 | 0.1323 | 0.1086 | 962/963/962 | no | 178.07 | 35.76 | 0.8950 | 0/0 | [p1](images/16-heading-page-break/pdflatex-pipeline-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | de1020c | 2/2 | ok | 19.0175 | 0.6505 | (-1,0) weak; (-1,20) moderate | 18.0489 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/962/962 | yes | 137.30 | 14.56 | 0.8888 | 1/0 | [p1](images/16-heading-page-break/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | exact | 2/2 | ok | 0.0574 | 1.0000 | (0,0); (0,0) | 0.0574 | 1.0000 | 6 | 0.0324 | 0.0000 | 962/962/962 | yes | 0.00 | 1.03 | 1.0000 | 1/1 | [p1](images/16-heading-page-break/pdflatex-lm-exact-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | main | 2/2 | ok | 18.3187 | 0.6670 | (-0.5,-27) weak; (0,0) | 18.3039 | 0.6667 | 255 | 0.1403 | 0.1159 | 962/963/962 | no | 156.24 | 20.84 | 0.8951 | 1/0 | [p1](images/16-heading-page-break/pdflatex-lm-main-export-p1-overlay.png) |
| 16-heading-page-break | pdflatex-lm | pipeline | 2/2 | ok | 10.5954 | 0.8523 | (0,0); (0,0) | 10.5954 | 0.8523 | 255 | 0.1036 | 0.0781 | 962/963/962 | no | 0.06 | 0.67 | 0.9990 | 1/0 | [p1](images/16-heading-page-break/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 16-heading-page-break | xelatex | de1020c | 2/2 | ok | 20.6100 | 0.6610 | (-3,-14.5) weak; (0,20.5) moderate | 20.1333 | 0.6680 | 255 | 0.1485 | 0.1243 | 962/962/962 | yes | 161.33 | 45.97 | 0.8950 | 0/0 | - |
| 16-heading-page-break | xelatex | exact | 2/2 | ok | 16.2497 | 0.7180 | (0,0); (-0.5,58) weak | 16.1299 | 0.7246 | 255 | 0.1294 | 0.1056 | 962/962/962 | yes | 162.15 | 36.89 | 0.8960 | 0/1 | - |
| 16-heading-page-break | xelatex | main | 2/2 | ok | 20.1391 | 0.6739 | (0,0); (0,14.5) weak | 20.0551 | 0.6774 | 255 | 0.1461 | 0.1220 | 962/963/962 | no | 135.98 | 15.96 | 0.9002 | 0/0 | - |
| 16-heading-page-break | xelatex | pipeline | 2/2 | ok | 17.5404 | 0.7206 | (1.5,0) weak; (-1,57.5) weak | 17.3620 | 0.7255 | 255 | 0.1329 | 0.1091 | 962/963/962 | no | 162.22 | 36.53 | 0.8949 | 0/0 | - |
| 16-heading-page-break | xelatex-lm | de1020c | 2/2 | ok | 19.0182 | 0.6506 | (-1,0) weak; (-1,20) moderate | 18.0480 | 0.6754 | 255 | 0.1439 | 0.1195 | 962/962/962 | yes | 137.29 | 14.56 | 0.8888 | 1/0 | - |
| 16-heading-page-break | xelatex-lm | exact | 2/2 | ok | 0.1985 | 0.9998 | (0,0); (0,0) | 0.1985 | 0.9998 | 26 | 0.0499 | 0.0000 | 962/962/962 | yes | 0.01 | 1.03 | 1.0000 | 1/1 | - |
| 16-heading-page-break | xelatex-lm | main | 2/2 | ok | 18.3184 | 0.6671 | (-0.5,-27) weak; (0,0) | 18.3034 | 0.6668 | 255 | 0.1402 | 0.1159 | 962/963/962 | no | 156.23 | 20.84 | 0.8951 | 1/0 | - |
| 16-heading-page-break | xelatex-lm | pipeline | 2/2 | ok | 10.5801 | 0.8524 | (0,0); (0,0) | 10.5801 | 0.8524 | 255 | 0.1036 | 0.0781 | 962/963/962 | no | 0.07 | 0.67 | 0.9990 | 1/0 | - |
| 17-apostrophes | lualatex | de1020c | 1/1 | ok | 0.8502 | 0.9875 | (-1.5,0) weak | 0.8119 | 0.9882 | 255 | 0.0068 | 0.0053 | 10/25/7 | no | 1.20 | 0.42 | 1.0000 | 0/0 | - |
| 17-apostrophes | lualatex | exact | 1/1 | ok | 0.8901 | 0.9843 | (0,0) | 0.8901 | 0.9843 | 255 | 0.0072 | 0.0058 | 10/25/9 | no | 56.35 | 0.77 | 0.8889 | 0/0 | - |
| 17-apostrophes | lualatex | main | 1/1 | ok | 0.8494 | 0.9875 | (-7.5,0) weak | 0.8284 | 0.9876 | 255 | 0.0068 | 0.0053 | 10/25/7 | no | 1.27 | 0.42 | 1.0000 | 0/0 | - |
| 17-apostrophes | lualatex | pipeline | 1/1 | ok | 0.9409 | 0.9852 | (53.5,0) moderate | 0.8847 | 0.9860 | 255 | 0.0073 | 0.0058 | 10/25/9 | no | 56.35 | 0.41 | 0.8889 | 0/0 | - |
| 17-apostrophes | lualatex-lm | de1020c | 1/1 | ok | 0.8622 | 0.9848 | (0,0) | 0.8622 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.56 | 0.33 | 0.9231 | 0/0 | - |
| 17-apostrophes | lualatex-lm | exact | 1/1 | ok | 0.0119 | 1.0000 | (0,0) | 0.0119 | 1.0000 | 12 | 0.0029 | 0.0000 | 25/25/25 | yes | 0.01 | 0.00 | 1.0000 | 0/0 | - |
| 17-apostrophes | lualatex-lm | main | 1/1 | ok | 0.8761 | 0.9846 | (0,0) | 0.8761 | 0.9846 | 255 | 0.0071 | 0.0057 | 25/25/13 | no | 44.76 | 0.33 | 0.9231 | 0/0 | - |
| 17-apostrophes | lualatex-lm | pipeline | 1/1 | ok | 0.6778 | 0.9896 | (-0.5,0) moderate | 0.6324 | 0.9900 | 255 | 0.0061 | 0.0048 | 25/25/25 | yes | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 17-apostrophes | pdflatex | de1020c | 1/1 | ok | 0.8063 | 0.9887 | (0,0) | 0.8063 | 0.9887 | 255 | 0.0066 | 0.0052 | 25/25/13 | no | 2.67 | 0.44 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-de1020c-export-p1-overlay.png) |
| 17-apostrophes | pdflatex | exact | 1/1 | ok | 0.8805 | 0.9847 | (0,0) | 0.8805 | 0.9847 | 255 | 0.0071 | 0.0057 | 25/25/25 | yes | 51.49 | 1.35 | 0.9200 | 0/0 | [p1](images/17-apostrophes/pdflatex-exact-export-p1-overlay.png) |
| 17-apostrophes | pdflatex | main | 1/1 | ok | 0.8138 | 0.9885 | (0,0) | 0.8138 | 0.9885 | 255 | 0.0067 | 0.0053 | 25/25/13 | no | 2.86 | 0.44 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-main-export-p1-overlay.png) |
| 17-apostrophes | pdflatex | pipeline | 1/1 | ok | 0.9369 | 0.9856 | (0,0) | 0.9369 | 0.9856 | 255 | 0.0072 | 0.0058 | 25/25/25 | yes | 51.49 | 0.99 | 0.9200 | 0/0 | [p1](images/17-apostrophes/pdflatex-pipeline-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | de1020c | 1/1 | ok | 0.8624 | 0.9848 | (0,0) | 0.8624 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.55 | 0.70 | 0.9231 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | exact | 1/1 | ok | 0.0034 | 1.0000 | (0,0) | 0.0034 | 1.0000 | 5 | 0.0019 | 0.0000 | 25/25/25 | yes | 0.00 | 1.03 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-exact-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | main | 1/1 | ok | 0.8762 | 0.9846 | (0,0) | 0.8762 | 0.9846 | 255 | 0.0071 | 0.0057 | 25/25/13 | no | 44.76 | 0.70 | 0.9231 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-main-export-p1-overlay.png) |
| 17-apostrophes | pdflatex-lm | pipeline | 1/1 | ok | 0.6774 | 0.9896 | (-0.5,0) moderate | 0.6321 | 0.9900 | 255 | 0.0061 | 0.0048 | 25/25/25 | yes | 0.00 | 0.67 | 1.0000 | 0/0 | [p1](images/17-apostrophes/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 17-apostrophes | xelatex | de1020c | 1/1 | ok | 0.7981 | 0.9883 | (0,0) | 0.7981 | 0.9883 | 255 | 0.0066 | 0.0051 | 25/25/13 | no | 2.69 | 0.44 | 1.0000 | 0/0 | - |
| 17-apostrophes | xelatex | exact | 1/1 | ok | 0.8839 | 0.9843 | (0,0) | 0.8839 | 0.9843 | 255 | 0.0072 | 0.0058 | 25/25/25 | yes | 51.49 | 1.35 | 0.9200 | 0/0 | - |
| 17-apostrophes | xelatex | main | 1/1 | ok | 0.8069 | 0.9882 | (0,0) | 0.8069 | 0.9882 | 255 | 0.0067 | 0.0051 | 25/25/13 | no | 2.87 | 0.44 | 1.0000 | 0/0 | - |
| 17-apostrophes | xelatex | pipeline | 1/1 | ok | 0.9417 | 0.9852 | (0,0) | 0.9417 | 0.9852 | 255 | 0.0073 | 0.0059 | 25/25/25 | yes | 51.49 | 0.99 | 0.9200 | 0/0 | - |
| 17-apostrophes | xelatex-lm | de1020c | 1/1 | ok | 0.8620 | 0.9848 | (0,0) | 0.8620 | 0.9848 | 255 | 0.0070 | 0.0057 | 25/25/13 | no | 44.56 | 0.70 | 0.9231 | 0/0 | - |
| 17-apostrophes | xelatex-lm | exact | 1/1 | ok | 0.0133 | 1.0000 | (0,0) | 0.0133 | 1.0000 | 16 | 0.0030 | 0.0000 | 25/25/25 | yes | 0.01 | 1.03 | 1.0000 | 0/0 | - |
| 17-apostrophes | xelatex-lm | main | 1/1 | ok | 0.8760 | 0.9846 | (0,0) | 0.8760 | 0.9846 | 255 | 0.0071 | 0.0057 | 25/25/13 | no | 44.76 | 0.70 | 0.9231 | 0/0 | - |
| 17-apostrophes | xelatex-lm | pipeline | 1/1 | ok | 0.6777 | 0.9896 | (-0.5,0) moderate | 0.6319 | 0.9900 | 255 | 0.0061 | 0.0048 | 25/25/25 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex | de1020c | 1/1 | ok | 1.3714 | 0.9790 | (0,0) | 1.3714 | 0.9790 | 255 | 0.0109 | 0.0087 | 36/36/36 | yes | 41.08 | 1.23 | 0.8889 | 0/0 | - |
| 18-ligatures | lualatex | exact | 1/1 | ok | 1.3323 | 0.9778 | (0,0) | 1.3323 | 0.9778 | 255 | 0.0108 | 0.0087 | 36/36/36 | yes | 50.22 | 1.57 | 0.8889 | 0/0 | - |
| 18-ligatures | lualatex | main | 1/1 | ok | 1.0519 | 0.9851 | (0,0) | 1.0519 | 0.9851 | 255 | 0.0097 | 0.0071 | 36/35/34 | no | 4.33 | 0.43 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex | pipeline | 1/1 | ok | 1.4150 | 0.9780 | (0,0) | 1.4150 | 0.9780 | 255 | 0.0111 | 0.0088 | 36/36/36 | yes | 50.22 | 1.21 | 0.8889 | 0/0 | - |
| 18-ligatures | lualatex-lm | de1020c | 1/1 | ok | 1.3134 | 0.9787 | (0,0) | 1.3134 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.13 | 0.34 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex-lm | exact | 1/1 | ok | 0.0168 | 1.0000 | (0,0) | 0.0168 | 1.0000 | 15 | 0.0043 | 0.0000 | 36/36/36 | yes | 0.01 | 0.00 | 1.0000 | 0/0 | - |
| 18-ligatures | lualatex-lm | main | 1/1 | ok | 1.4225 | 0.9764 | (-1,0) weak | 1.3902 | 0.9769 | 255 | 0.0111 | 0.0092 | 36/35/34 | no | 55.14 | 1.19 | 0.8824 | 0/0 | - |
| 18-ligatures | lualatex-lm | pipeline | 1/1 | ok | 0.9982 | 0.9858 | (0,0) | 0.9982 | 0.9858 | 255 | 0.0093 | 0.0071 | 36/36/36 | yes | 0.01 | 0.36 | 1.0000 | 0/0 | - |
| 18-ligatures | pdflatex | de1020c | 1/1 | ok | 1.2894 | 0.9808 | (0,0) | 1.2894 | 0.9808 | 255 | 0.0105 | 0.0082 | 36/36/36 | yes | 40.70 | 1.23 | 0.8889 | 0/0 | [p1](images/18-ligatures/pdflatex-de1020c-export-p1-overlay.png) |
| 18-ligatures | pdflatex | exact | 1/1 | ok | 1.3107 | 0.9788 | (0,0) | 1.3107 | 0.9788 | 255 | 0.0106 | 0.0086 | 36/36/36 | yes | 49.62 | 1.57 | 0.8889 | 0/0 | [p1](images/18-ligatures/pdflatex-exact-export-p1-overlay.png) |
| 18-ligatures | pdflatex | main | 1/1 | ok | 1.3805 | 0.9805 | (-1.5,0) moderate | 1.2545 | 0.9824 | 255 | 0.0107 | 0.0085 | 36/35/34 | no | 5.04 | 0.43 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-main-export-p1-overlay.png) |
| 18-ligatures | pdflatex | pipeline | 1/1 | ok | 1.3778 | 0.9792 | (0,0) | 1.3778 | 0.9792 | 255 | 0.0108 | 0.0086 | 36/36/36 | yes | 49.62 | 1.21 | 0.8889 | 0/0 | [p1](images/18-ligatures/pdflatex-pipeline-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | de1020c | 1/1 | ok | 1.3148 | 0.9787 | (0,0) | 1.3148 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.14 | 0.69 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-de1020c-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | exact | 1/1 | ok | 0.0056 | 1.0000 | (0,0) | 0.0056 | 1.0000 | 5 | 0.0031 | 0.0000 | 36/36/36 | yes | 0.00 | 1.03 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-exact-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | main | 1/1 | ok | 1.4220 | 0.9764 | (-1,0) weak | 1.3906 | 0.9769 | 255 | 0.0111 | 0.0092 | 36/35/34 | no | 55.15 | 1.46 | 0.8824 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-main-export-p1-overlay.png) |
| 18-ligatures | pdflatex-lm | pipeline | 1/1 | ok | 1.0000 | 0.9858 | (0,0) | 1.0000 | 0.9858 | 255 | 0.0093 | 0.0071 | 36/36/36 | yes | 0.00 | 0.67 | 1.0000 | 0/0 | [p1](images/18-ligatures/pdflatex-lm-pipeline-export-p1-overlay.png) |
| 18-ligatures | xelatex | de1020c | 1/1 | ok | 1.3498 | 0.9797 | (0,0) | 1.3498 | 0.9797 | 255 | 0.0108 | 0.0087 | 36/36/36 | yes | 40.84 | 1.23 | 0.8889 | 0/0 | - |
| 18-ligatures | xelatex | exact | 1/1 | ok | 1.3330 | 0.9780 | (0,0) | 1.3330 | 0.9780 | 255 | 0.0108 | 0.0088 | 36/36/36 | yes | 49.96 | 1.57 | 0.8889 | 0/0 | - |
| 18-ligatures | xelatex | main | 1/1 | ok | 1.2336 | 0.9826 | (-0.5,0) moderate | 1.1599 | 0.9838 | 255 | 0.0103 | 0.0081 | 36/35/34 | no | 4.58 | 0.43 | 1.0000 | 0/0 | - |
| 18-ligatures | xelatex | pipeline | 1/1 | ok | 1.4011 | 0.9784 | (0,0) | 1.4011 | 0.9784 | 255 | 0.0111 | 0.0088 | 36/36/36 | yes | 49.96 | 1.21 | 0.8889 | 0/0 | - |
| 18-ligatures | xelatex-lm | de1020c | 1/1 | ok | 1.3133 | 0.9787 | (0,0) | 1.3133 | 0.9787 | 255 | 0.0107 | 0.0087 | 36/36/36 | yes | 9.13 | 0.69 | 1.0000 | 0/0 | - |
| 18-ligatures | xelatex-lm | exact | 1/1 | ok | 0.0167 | 1.0000 | (0,0) | 0.0167 | 1.0000 | 15 | 0.0042 | 0.0000 | 36/36/36 | yes | 0.01 | 1.03 | 1.0000 | 0/0 | - |
| 18-ligatures | xelatex-lm | main | 1/1 | ok | 1.4224 | 0.9764 | (-1,0) weak | 1.3902 | 0.9769 | 255 | 0.0111 | 0.0092 | 36/35/34 | no | 55.14 | 1.46 | 0.8824 | 0/0 | - |
| 18-ligatures | xelatex-lm | pipeline | 1/1 | ok | 0.9979 | 0.9859 | (0,0) | 0.9979 | 0.9859 | 255 | 0.0093 | 0.0071 | 36/36/36 | yes | 0.01 | 0.67 | 1.0000 | 0/0 | - |

## Diagnostic: preview-equivalent comparison (CoreText draw of compile_result vs reference PDF raster)

Weaker than a capture of the real preview: it re-implements the app's draw code path rather than exercising the SwiftUI Canvas. Word-box metrics are not available for this side (no PDF), so they are omitted.

| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\|Δ\| raw | SSIM₈ raw | registration Δ pt (dx,dy per page; `weak`/`moderate` = shift explains <25% of the error) | mean\|Δ\| after reg | SSIM₈ after reg | max | differing | ≥thr | words ref/ours/aligned | seq= | mean\|dx\| pt | mean\|dy\| pt | line-start agree | rules ref/ours | overlay |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 01-plain-paragraph | lualatex | de1020c | 1/1 | ok | 0.1686 | 0.9978 | (0,0) | 0.1686 | 0.9978 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | exact | 1/1 | ok | 0.3908 | 0.9942 | (0.5,0) weak | 0.3840 | 0.9943 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | main | 1/1 | ok | 0.2718 | 0.9963 | (-0.5,0) | 0.1560 | 0.9982 | 255 | 0.0027 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex | pipeline | 1/1 | ok | 0.3908 | 0.9942 | (0.5,0) weak | 0.3840 | 0.9943 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | de1020c | 1/1 | ok | 0.3642 | 0.9942 | (0,0) | 0.3642 | 0.9942 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | exact | 1/1 | ok | 0.2352 | 0.9966 | (0,0) | 0.2352 | 0.9966 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | main | 1/1 | ok | 0.3577 | 0.9943 | (0,0) | 0.3577 | 0.9943 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | lualatex-lm | pipeline | 1/1 | ok | 0.2352 | 0.9966 | (0,0) | 0.2352 | 0.9966 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | de1020c | 1/1 | ok | 0.1741 | 0.9977 | (0,0) | 0.1741 | 0.9977 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | exact | 1/1 | ok | 0.3852 | 0.9945 | (10.5,0) weak | 0.3792 | 0.9947 | 255 | 0.0031 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | main | 1/1 | ok | 0.1143 | 0.9987 | (0,0) | 0.1143 | 0.9987 | 255 | 0.0021 | 0.0011 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex | pipeline | 1/1 | ok | 0.3852 | 0.9945 | (10.5,0) weak | 0.3792 | 0.9947 | 255 | 0.0031 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 0.3639 | 0.9942 | (0,0) | 0.3639 | 0.9942 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | exact | 1/1 | ok | 0.2364 | 0.9966 | (0,0) | 0.2364 | 0.9966 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | main | 1/1 | ok | 0.3573 | 0.9943 | (0,0) | 0.3573 | 0.9943 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 0.2364 | 0.9966 | (0,0) | 0.2364 | 0.9966 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | de1020c | 1/1 | ok | 0.1716 | 0.9978 | (0,0) | 0.1716 | 0.9978 | 255 | 0.0023 | 0.0014 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | exact | 1/1 | ok | 0.3909 | 0.9942 | (0.5,0) weak | 0.3836 | 0.9943 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | main | 1/1 | ok | 0.2772 | 0.9962 | (-0.5,0) | 0.1595 | 0.9981 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex | pipeline | 1/1 | ok | 0.3909 | 0.9942 | (0.5,0) weak | 0.3836 | 0.9943 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | de1020c | 1/1 | ok | 0.3644 | 0.9942 | (0,0) | 0.3644 | 0.9942 | 255 | 0.0030 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | exact | 1/1 | ok | 0.2352 | 0.9966 | (0,0) | 0.2352 | 0.9966 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | main | 1/1 | ok | 0.3579 | 0.9943 | (0,0) | 0.3579 | 0.9943 | 255 | 0.0030 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 01-plain-paragraph | xelatex-lm | pipeline | 1/1 | ok | 0.2352 | 0.9966 | (0,0) | 0.2352 | 0.9966 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | de1020c | 1/1 | ok | 8.3831 | 0.8656 | (-3,0) weak | 8.3728 | 0.8658 | 255 | 0.0613 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | exact | 1/1 | ok | 7.1756 | 0.8902 | (0,0) | 7.1756 | 0.8902 | 255 | 0.0554 | 0.0449 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | main | 1/1 | ok | 8.4139 | 0.8658 | (0,0) | 8.4139 | 0.8658 | 255 | 0.0616 | 0.0511 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex | pipeline | 1/1 | ok | 7.1756 | 0.8902 | (0,0) | 7.1756 | 0.8902 | 255 | 0.0554 | 0.0449 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | de1020c | 1/1 | ok | 7.7865 | 0.8620 | (-0.5,-14.5) weak | 7.7677 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | exact | 1/1 | ok | 4.5475 | 0.9366 | (0,0) | 4.5475 | 0.9366 | 255 | 0.0447 | 0.0336 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | main | 1/1 | ok | 7.7929 | 0.8614 | (0,0) | 7.7929 | 0.8614 | 255 | 0.0598 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | lualatex-lm | pipeline | 1/1 | ok | 4.5475 | 0.9366 | (0,0) | 4.5475 | 0.9366 | 255 | 0.0447 | 0.0336 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | de1020c | 1/1 | ok | 8.3720 | 0.8664 | (0,0) | 8.3720 | 0.8664 | 255 | 0.0611 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | exact | 1/1 | ok | 7.1610 | 0.8910 | (1,0) weak | 7.0842 | 0.8919 | 255 | 0.0553 | 0.0447 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | main | 1/1 | ok | 8.4698 | 0.8655 | (-3.5,0) weak | 8.4134 | 0.8661 | 255 | 0.0615 | 0.0513 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex | pipeline | 1/1 | ok | 7.1610 | 0.8910 | (1,0) weak | 7.0842 | 0.8919 | 255 | 0.0553 | 0.0447 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 7.7859 | 0.8621 | (-0.5,-14.5) weak | 7.7684 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | exact | 1/1 | ok | 4.5547 | 0.9365 | (0,0) | 4.5547 | 0.9365 | 255 | 0.0447 | 0.0337 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | main | 1/1 | ok | 7.7930 | 0.8614 | (0,0) | 7.7930 | 0.8614 | 255 | 0.0598 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | pdflatex-lm | pipeline | 1/1 | ok | 4.5547 | 0.9365 | (0,0) | 4.5547 | 0.9365 | 255 | 0.0447 | 0.0337 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | de1020c | 1/1 | ok | 8.3703 | 0.8657 | (0,0) | 8.3703 | 0.8657 | 255 | 0.0613 | 0.0509 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | exact | 1/1 | ok | 7.1986 | 0.8903 | (0.5,0) weak | 7.1669 | 0.8904 | 255 | 0.0556 | 0.0451 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | main | 1/1 | ok | 8.4639 | 0.8648 | (-0.5,0) weak | 8.4582 | 0.8649 | 255 | 0.0618 | 0.0514 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex | pipeline | 1/1 | ok | 7.1986 | 0.8903 | (0.5,0) weak | 7.1669 | 0.8904 | 255 | 0.0556 | 0.0451 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | de1020c | 1/1 | ok | 7.7862 | 0.8620 | (-0.5,-14.5) weak | 7.7678 | 0.8603 | 255 | 0.0597 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | exact | 1/1 | ok | 4.5484 | 0.9366 | (0,0) | 4.5484 | 0.9366 | 255 | 0.0447 | 0.0336 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | main | 1/1 | ok | 7.7928 | 0.8614 | (0,0) | 7.7928 | 0.8614 | 255 | 0.0598 | 0.0495 | -/-/- | - | - | - | - | 0/0 | - |
| 02-wrapping-paragraph | xelatex-lm | pipeline | 1/1 | ok | 4.5484 | 0.9366 | (0,0) | 4.5484 | 0.9366 | 255 | 0.0447 | 0.0336 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | de1020c | 1/1 | ok | 1.3394 | 0.9793 | (0,52.5) moderate | 1.1862 | 0.9829 | 255 | 0.0085 | 0.0073 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | exact | 1/1 | ok | 0.9273 | 0.9876 | (-0.5,-1.5) weak | 0.8913 | 0.9885 | 255 | 0.0065 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | main | 1/1 | ok | 1.3597 | 0.9789 | (0,52.5) moderate | 1.2063 | 0.9826 | 255 | 0.0085 | 0.0074 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex | pipeline | 1/1 | ok | 1.0038 | 0.9865 | (0,-1.5) weak | 0.9814 | 0.9874 | 255 | 0.0069 | 0.0059 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | de1020c | 1/1 | ok | 1.2336 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | exact | 1/1 | ok | 0.7892 | 0.9898 | (0,0) | 0.7892 | 0.9898 | 255 | 0.0059 | 0.0049 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | main | 1/1 | ok | 1.2425 | 0.9769 | (0,53.5) moderate | 1.1129 | 0.9812 | 255 | 0.0082 | 0.0070 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | lualatex-lm | pipeline | 1/1 | ok | 0.8550 | 0.9887 | (0,0) | 0.8550 | 0.9887 | 255 | 0.0062 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | de1020c | 1/1 | ok | 1.3496 | 0.9793 | (0.5,52.5) moderate | 1.2021 | 0.9827 | 255 | 0.0085 | 0.0072 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | exact | 1/1 | ok | 0.9542 | 0.9873 | (-0.5,-2) weak | 0.9354 | 0.9868 | 255 | 0.0066 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | main | 1/1 | ok | 1.3719 | 0.9789 | (0.5,52.5) moderate | 1.2200 | 0.9824 | 255 | 0.0086 | 0.0074 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex | pipeline | 1/1 | ok | 1.0316 | 0.9862 | (0.5,-2) weak | 1.0232 | 0.9859 | 255 | 0.0070 | 0.0059 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | de1020c | 1/1 | ok | 1.2335 | 0.9771 | (0,54) moderate | 1.0954 | 0.9811 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | exact | 1/1 | ok | 0.7888 | 0.9898 | (0,0) | 0.7888 | 0.9898 | 255 | 0.0059 | 0.0049 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | main | 1/1 | ok | 1.2428 | 0.9769 | (0,54) moderate | 1.1208 | 0.9807 | 255 | 0.0082 | 0.0070 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | pdflatex-lm | pipeline | 1/1 | ok | 0.8546 | 0.9887 | (0,0) | 0.8546 | 0.9887 | 255 | 0.0062 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | de1020c | 1/1 | ok | 1.3370 | 0.9793 | (0,52.5) moderate | 1.1869 | 0.9829 | 255 | 0.0085 | 0.0073 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | exact | 1/1 | ok | 0.9296 | 0.9876 | (-0.5,-1.5) weak | 0.8896 | 0.9886 | 255 | 0.0065 | 0.0055 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | main | 1/1 | ok | 1.3601 | 0.9789 | (0,52.5) moderate | 1.2075 | 0.9826 | 255 | 0.0086 | 0.0075 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex | pipeline | 1/1 | ok | 1.0089 | 0.9864 | (0,-1.5) weak | 0.9851 | 0.9874 | 255 | 0.0070 | 0.0059 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | de1020c | 1/1 | ok | 1.2337 | 0.9771 | (0,53.5) moderate | 1.0886 | 0.9817 | 255 | 0.0082 | 0.0069 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | exact | 1/1 | ok | 0.7894 | 0.9898 | (0,0) | 0.7894 | 0.9898 | 255 | 0.0059 | 0.0049 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | main | 1/1 | ok | 1.2424 | 0.9769 | (0,53.5) moderate | 1.1129 | 0.9812 | 255 | 0.0082 | 0.0070 | -/-/- | - | - | - | - | 0/0 | - |
| 03-section-heading | xelatex-lm | pipeline | 1/1 | ok | 0.8550 | 0.9887 | (0,0) | 0.8550 | 0.9887 | 255 | 0.0062 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | de1020c | 1/1 | ok | 0.3808 | 0.9945 | (0,0) | 0.3808 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | exact | 1/1 | ok | 0.4247 | 0.9938 | (0,0) | 0.4247 | 0.9938 | 255 | 0.0033 | 0.0026 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | main | 1/1 | ok | 0.3769 | 0.9945 | (0,0) | 0.3769 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex | pipeline | 1/1 | ok | 0.4247 | 0.9938 | (0,0) | 0.4247 | 0.9938 | 255 | 0.0033 | 0.0026 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | de1020c | 1/1 | ok | 0.4244 | 0.9933 | (-49,0) weak | 0.4148 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | exact | 1/1 | ok | 0.3651 | 0.9947 | (0,0) | 0.3651 | 0.9947 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | main | 1/1 | ok | 0.4253 | 0.9933 | (-49,0) weak | 0.4146 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | lualatex-lm | pipeline | 1/1 | ok | 0.3651 | 0.9947 | (0,0) | 0.3651 | 0.9947 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | de1020c | 1/1 | ok | 0.3837 | 0.9944 | (-0.5,0) weak | 0.3764 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | exact | 1/1 | ok | 0.4235 | 0.9938 | (0,0) | 0.4235 | 0.9938 | 255 | 0.0033 | 0.0026 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | main | 1/1 | ok | 0.3805 | 0.9944 | (-0.5,0) weak | 0.3776 | 0.9945 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex | pipeline | 1/1 | ok | 0.4235 | 0.9938 | (0,0) | 0.4235 | 0.9938 | 255 | 0.0033 | 0.0026 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | de1020c | 1/1 | ok | 0.4246 | 0.9933 | (-49,0) weak | 0.4162 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | exact | 1/1 | ok | 0.3654 | 0.9947 | (0,0) | 0.3654 | 0.9947 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | main | 1/1 | ok | 0.4257 | 0.9933 | (-49,0) weak | 0.4164 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | pdflatex-lm | pipeline | 1/1 | ok | 0.3654 | 0.9947 | (0,0) | 0.3654 | 0.9947 | 255 | 0.0031 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | de1020c | 1/1 | ok | 0.3849 | 0.9944 | (-0.5,0) weak | 0.3834 | 0.9943 | 255 | 0.0032 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | exact | 1/1 | ok | 0.4236 | 0.9938 | (0,0) | 0.4236 | 0.9938 | 255 | 0.0033 | 0.0026 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | main | 1/1 | ok | 0.3811 | 0.9945 | (-0.5,0) weak | 0.3801 | 0.9944 | 255 | 0.0031 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex | pipeline | 1/1 | ok | 0.4236 | 0.9938 | (0,0) | 0.4236 | 0.9938 | 255 | 0.0033 | 0.0026 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | de1020c | 1/1 | ok | 0.4245 | 0.9933 | (-49,0) weak | 0.4138 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | exact | 1/1 | ok | 0.3770 | 0.9945 | (0,0) | 0.3770 | 0.9945 | 255 | 0.0031 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | main | 1/1 | ok | 0.4237 | 0.9933 | (-49,0) weak | 0.4127 | 0.9934 | 255 | 0.0033 | 0.0027 | -/-/- | - | - | - | - | 0/0 | - |
| 04-bold-emph | xelatex-lm | pipeline | 1/1 | ok | 0.3770 | 0.9945 | (0,0) | 0.3770 | 0.9945 | 255 | 0.0031 | 0.0025 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex | de1020c | 1/1 | ok | 0.2971 | 0.9955 | (0,0) | 0.2971 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | exact | 1/1 | ok | 0.3578 | 0.9948 | (11.5,0) moderate | 0.3283 | 0.9951 | 255 | 0.0029 | 0.0022 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | main | 1/1 | ok | 0.2490 | 0.9963 | (0,0) | 0.2490 | 0.9963 | 255 | 0.0026 | 0.0018 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex | pipeline | 1/1 | ok | 0.3578 | 0.9948 | (11.5,0) moderate | 0.3283 | 0.9951 | 255 | 0.0029 | 0.0022 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | lualatex-lm | de1020c | 1/1 | ok | 0.3619 | 0.9938 | (2.5,0) weak | 0.3494 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | exact | 1/1 | ok | 0.2657 | 0.9960 | (-0.5,0) moderate | 0.2087 | 0.9969 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | main | 1/1 | ok | 0.3628 | 0.9938 | (-3.5,0) moderate | 0.3432 | 0.9941 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | lualatex-lm | pipeline | 1/1 | ok | 0.2657 | 0.9960 | (-0.5,0) moderate | 0.2087 | 0.9969 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex | de1020c | 1/1 | ok | 0.3125 | 0.9955 | (0,0) | 0.3125 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex | exact | 1/1 | ok | 0.3573 | 0.9950 | (11.5,0) moderate | 0.3051 | 0.9957 | 255 | 0.0029 | 0.0022 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex | main | 1/1 | ok | 0.2341 | 0.9967 | (0,0) | 0.2341 | 0.9967 | 255 | 0.0025 | 0.0016 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex | pipeline | 1/1 | ok | 0.3573 | 0.9950 | (11.5,0) moderate | 0.3051 | 0.9957 | 255 | 0.0029 | 0.0022 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | pdflatex-lm | de1020c | 1/1 | ok | 0.3618 | 0.9938 | (2.5,0) weak | 0.3496 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex-lm | exact | 1/1 | ok | 0.2676 | 0.9959 | (-0.5,0) moderate | 0.2083 | 0.9969 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex-lm | main | 1/1 | ok | 0.3625 | 0.9938 | (-3.5,0) moderate | 0.3430 | 0.9941 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | pdflatex-lm | pipeline | 1/1 | ok | 0.2676 | 0.9959 | (-0.5,0) moderate | 0.2083 | 0.9969 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex | de1020c | 1/1 | ok | 0.2963 | 0.9955 | (0,0) | 0.2963 | 0.9955 | 255 | 0.0027 | 0.0020 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | exact | 1/1 | ok | 0.3576 | 0.9948 | (11.5,0) moderate | 0.3313 | 0.9951 | 255 | 0.0029 | 0.0022 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | main | 1/1 | ok | 0.2493 | 0.9963 | (0,0) | 0.2493 | 0.9963 | 255 | 0.0026 | 0.0018 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex | pipeline | 1/1 | ok | 0.3576 | 0.9948 | (11.5,0) moderate | 0.3313 | 0.9951 | 255 | 0.0029 | 0.0022 | -/-/- | - | - | - | - | 2/0 | - |
| 05-unicode | xelatex-lm | de1020c | 1/1 | ok | 0.3620 | 0.9938 | (2.5,0) weak | 0.3493 | 0.9940 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | exact | 1/1 | ok | 0.2659 | 0.9960 | (-0.5,0) moderate | 0.2088 | 0.9969 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | main | 1/1 | ok | 0.3628 | 0.9938 | (-3.5,0) moderate | 0.3432 | 0.9941 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 05-unicode | xelatex-lm | pipeline | 1/1 | ok | 0.2659 | 0.9960 | (-0.5,0) moderate | 0.2088 | 0.9969 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | de1020c | 1/1 | ok | 0.3823 | 0.9934 | (0,3.5) | 0.2551 | 0.9959 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | exact | 1/1 | ok | 0.2981 | 0.9947 | (0,0) | 0.2981 | 0.9947 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | main | 1/1 | ok | 0.3834 | 0.9933 | (-6.5,3.5) | 0.2030 | 0.9966 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex | pipeline | 1/1 | ok | 0.2981 | 0.9947 | (0,0) | 0.2981 | 0.9947 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | de1020c | 1/1 | ok | 0.3683 | 0.9931 | (-32.5,3.5) moderate | 0.3139 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | exact | 1/1 | ok | 0.2285 | 0.9963 | (-0.5,0) moderate | 0.2163 | 0.9963 | 255 | 0.0021 | 0.0016 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | main | 1/1 | ok | 0.3734 | 0.9929 | (9.5,3.5) moderate | 0.3123 | 0.9945 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | lualatex-lm | pipeline | 1/1 | ok | 0.2285 | 0.9963 | (-0.5,0) moderate | 0.2163 | 0.9963 | 255 | 0.0021 | 0.0016 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | de1020c | 1/1 | ok | 0.3812 | 0.9936 | (0,3.5) | 0.2591 | 0.9960 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | exact | 1/1 | ok | 0.3056 | 0.9948 | (12,0) weak | 0.2964 | 0.9952 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | main | 1/1 | ok | 0.3779 | 0.9935 | (-6,3.5) | 0.2148 | 0.9967 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex | pipeline | 1/1 | ok | 0.3056 | 0.9948 | (12,0) weak | 0.2964 | 0.9952 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | de1020c | 1/1 | ok | 0.3681 | 0.9931 | (-32.5,3.5) moderate | 0.3137 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | exact | 1/1 | ok | 0.2296 | 0.9963 | (-0.5,0) moderate | 0.2148 | 0.9964 | 255 | 0.0021 | 0.0016 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | main | 1/1 | ok | 0.3728 | 0.9929 | (9.5,3.5) moderate | 0.3118 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | pdflatex-lm | pipeline | 1/1 | ok | 0.2296 | 0.9963 | (-0.5,0) moderate | 0.2148 | 0.9964 | 255 | 0.0021 | 0.0016 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | de1020c | 1/1 | ok | 0.3823 | 0.9934 | (0,3.5) | 0.2545 | 0.9959 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | exact | 1/1 | ok | 0.2981 | 0.9947 | (0,0) | 0.2981 | 0.9947 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | main | 1/1 | ok | 0.3835 | 0.9933 | (-6.5,3.5) | 0.2028 | 0.9966 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex | pipeline | 1/1 | ok | 0.2981 | 0.9947 | (0,0) | 0.2981 | 0.9947 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | de1020c | 1/1 | ok | 0.3684 | 0.9931 | (-32.5,3.5) moderate | 0.3141 | 0.9945 | 255 | 0.0029 | 0.0023 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | exact | 1/1 | ok | 0.2285 | 0.9963 | (-0.5,0) moderate | 0.2163 | 0.9963 | 255 | 0.0021 | 0.0016 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | main | 1/1 | ok | 0.3734 | 0.9929 | (9.5,3.5) moderate | 0.3122 | 0.9945 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 0/0 | - |
| 06-math-inline | xelatex-lm | pipeline | 1/1 | ok | 0.2285 | 0.9963 | (-0.5,0) moderate | 0.2163 | 0.9963 | 255 | 0.0021 | 0.0016 | -/-/- | - | - | - | - | 0/0 | - |
| 07-math-display | lualatex | de1020c | 1/1 | ok | 0.2575 | 0.9950 | (0,0) | 0.2575 | 0.9950 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex | exact | 1/1 | ok | 0.2654 | 0.9955 | (0,0) | 0.2654 | 0.9955 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex | main | 1/1 | ok | 0.2605 | 0.9947 | (0,0) | 0.2605 | 0.9947 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | lualatex | pipeline | 1/1 | ok | 0.2654 | 0.9955 | (0,0) | 0.2654 | 0.9955 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | de1020c | 1/1 | ok | 0.3485 | 0.9931 | (0,0) | 0.3485 | 0.9931 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | exact | 1/1 | ok | 0.2499 | 0.9958 | (-0.5,0) moderate | 0.2239 | 0.9962 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | lualatex-lm | main | 1/1 | ok | 0.3517 | 0.9928 | (0,0) | 0.3517 | 0.9928 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | lualatex-lm | pipeline | 1/1 | ok | 0.2499 | 0.9958 | (-0.5,0) moderate | 0.2239 | 0.9962 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex | de1020c | 1/1 | ok | 0.2593 | 0.9950 | (0,0) | 0.2593 | 0.9950 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex | exact | 1/1 | ok | 0.2498 | 0.9958 | (0,0) | 0.2498 | 0.9958 | 255 | 0.0024 | 0.0017 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex | main | 1/1 | ok | 0.2623 | 0.9946 | (0,0) | 0.2623 | 0.9946 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | pdflatex | pipeline | 1/1 | ok | 0.2498 | 0.9958 | (0,0) | 0.2498 | 0.9958 | 255 | 0.0024 | 0.0017 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex-lm | de1020c | 1/1 | ok | 0.3516 | 0.9931 | (0,0) | 0.3516 | 0.9931 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex-lm | exact | 1/1 | ok | 0.2416 | 0.9959 | (-0.5,0) moderate | 0.2119 | 0.9964 | 255 | 0.0024 | 0.0018 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | pdflatex-lm | main | 1/1 | ok | 0.3549 | 0.9927 | (0,0) | 0.3549 | 0.9927 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | pdflatex-lm | pipeline | 1/1 | ok | 0.2416 | 0.9959 | (-0.5,0) moderate | 0.2119 | 0.9964 | 255 | 0.0024 | 0.0018 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | de1020c | 1/1 | ok | 0.2573 | 0.9950 | (0,0) | 0.2573 | 0.9950 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | exact | 1/1 | ok | 0.2651 | 0.9955 | (0,0) | 0.2651 | 0.9955 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex | main | 1/1 | ok | 0.2603 | 0.9947 | (0,0) | 0.2603 | 0.9947 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | xelatex | pipeline | 1/1 | ok | 0.2651 | 0.9955 | (0,0) | 0.2651 | 0.9955 | 255 | 0.0026 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | de1020c | 1/1 | ok | 0.3485 | 0.9931 | (0,0) | 0.3485 | 0.9931 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | exact | 1/1 | ok | 0.2499 | 0.9958 | (-0.5,0) moderate | 0.2243 | 0.9962 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 07-math-display | xelatex-lm | main | 1/1 | ok | 0.3517 | 0.9928 | (0,0) | 0.3517 | 0.9928 | 255 | 0.0029 | 0.0024 | -/-/- | - | - | - | - | 3/0 | - |
| 07-math-display | xelatex-lm | pipeline | 1/1 | ok | 0.2499 | 0.9958 | (-0.5,0) moderate | 0.2243 | 0.9962 | 255 | 0.0025 | 0.0019 | -/-/- | - | - | - | - | 3/1 | - |
| 08-two-page | lualatex | de1020c | 3/3 | ok | 26.2189 | 0.5659 | (0,0); (-3,6) moderate; (-3,34.5) moderate | 24.6331 | 0.5959 | 255 | 0.1880 | 0.1574 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex | exact | 3/3 | ok | 21.1558 | 0.6700 | (0,0); (0,0); (1,0) weak | 21.1024 | 0.6710 | 255 | 0.1619 | 0.1322 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex | main | 3/3 | ok | 25.2811 | 0.5915 | (0,0); (0,2) weak; (0,49) moderate | 24.7455 | 0.6039 | 255 | 0.1831 | 0.1528 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex | pipeline | 3/3 | ok | 21.1558 | 0.6700 | (0,0); (0,0); (1,0) weak | 21.1024 | 0.6710 | 255 | 0.1619 | 0.1322 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | de1020c | 3/3 | ok | 23.7956 | 0.5635 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7517 | 0.5869 | 255 | 0.1801 | 0.1495 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | exact | 3/3 | ok | 13.0054 | 0.8184 | (0,0); (0,0); (0,0) | 13.0054 | 0.8184 | 255 | 0.1281 | 0.0963 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | main | 3/3 | ok | 23.2796 | 0.5779 | (-0.5,-27) weak; (-0.5,-12.5) weak; (-0.5,5.5) moderate | 22.8321 | 0.5860 | 255 | 0.1772 | 0.1468 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | lualatex-lm | pipeline | 3/3 | ok | 13.0054 | 0.8184 | (0,0); (0,0); (0,0) | 13.0054 | 0.8184 | 255 | 0.1281 | 0.0963 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | de1020c | 3/3 | ok | 26.1972 | 0.5669 | (0,-14.5) weak; (0,6) moderate; (0,34.5) moderate | 24.5512 | 0.5981 | 255 | 0.1875 | 0.1571 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | exact | 3/3 | ok | 21.1709 | 0.6711 | (-1.5,0) weak; (1.5,0) weak; (-1.5,0) weak | 21.0236 | 0.6737 | 255 | 0.1614 | 0.1321 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | main | 3/3 | ok | 25.3258 | 0.5923 | (0,0); (-11,2) weak; (0,49) moderate | 24.7916 | 0.6035 | 255 | 0.1828 | 0.1528 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex | pipeline | 3/3 | ok | 21.1709 | 0.6711 | (-1.5,0) weak; (1.5,0) weak; (-1.5,0) weak | 21.0236 | 0.6737 | 255 | 0.1614 | 0.1321 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | de1020c | 3/3 | ok | 23.7951 | 0.5635 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7523 | 0.5869 | 255 | 0.1800 | 0.1496 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | exact | 3/3 | ok | 13.0318 | 0.8180 | (0,0); (0,0); (0,0) | 13.0318 | 0.8180 | 255 | 0.1282 | 0.0964 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | main | 3/3 | ok | 23.2802 | 0.5779 | (-0.5,-27) weak; (-0.5,2) weak; (-0.5,5.5) moderate | 22.8314 | 0.5887 | 255 | 0.1772 | 0.1468 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | pdflatex-lm | pipeline | 3/3 | ok | 13.0318 | 0.8180 | (0,0); (0,0); (0,0) | 13.0318 | 0.8180 | 255 | 0.1282 | 0.0964 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | de1020c | 3/3 | ok | 26.2441 | 0.5658 | (-3,-14.5) weak; (-3,6) moderate; (-3,34.5) moderate | 24.6604 | 0.5954 | 255 | 0.1881 | 0.1577 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | exact | 3/3 | ok | 21.1748 | 0.6696 | (1.5,0) weak; (0,0); (1,0) weak | 21.1146 | 0.6705 | 255 | 0.1620 | 0.1325 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | main | 3/3 | ok | 25.2633 | 0.5919 | (0,0); (0,2) weak; (0,49) moderate | 24.7282 | 0.6042 | 255 | 0.1831 | 0.1530 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex | pipeline | 3/3 | ok | 21.1748 | 0.6696 | (1.5,0) weak; (0,0); (1,0) weak | 21.1146 | 0.6705 | 255 | 0.1620 | 0.1325 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | de1020c | 3/3 | ok | 23.7957 | 0.5634 | (-1,0) weak; (-1,36.5) moderate; (-1,52) moderate | 22.7513 | 0.5869 | 255 | 0.1800 | 0.1495 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | exact | 3/3 | ok | 13.0090 | 0.8183 | (0,0); (0,0); (0,0) | 13.0090 | 0.8183 | 255 | 0.1281 | 0.0963 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | main | 3/3 | ok | 23.2798 | 0.5779 | (-0.5,-27) weak; (-0.5,-12.5) weak; (-0.5,5.5) moderate | 22.8322 | 0.5859 | 255 | 0.1772 | 0.1468 | -/-/- | - | - | - | - | 0/0 | - |
| 08-two-page | xelatex-lm | pipeline | 3/3 | ok | 13.0090 | 0.8183 | (0,0); (0,0); (0,0) | 13.0090 | 0.8183 | 255 | 0.1281 | 0.0963 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | de1020c | 1/1 | ok | 2.4844 | 0.9504 | (0,38.5) | 1.7906 | 0.9701 | 255 | 0.0174 | 0.0146 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | exact | 1/1 | ok | 1.8428 | 0.9716 | (0,-1) moderate | 1.7242 | 0.9743 | 255 | 0.0138 | 0.0115 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | main | 1/1 | ok | 2.5088 | 0.9499 | (0,38.5) | 1.6561 | 0.9716 | 255 | 0.0175 | 0.0147 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex | pipeline | 1/1 | ok | 1.9289 | 0.9706 | (1,-1) moderate | 1.7930 | 0.9729 | 255 | 0.0142 | 0.0118 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | de1020c | 1/1 | ok | 2.2538 | 0.9494 | (-0.5,39) moderate | 1.9068 | 0.9641 | 255 | 0.0169 | 0.0140 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | exact | 1/1 | ok | 1.3287 | 0.9804 | (-0.5,-0.5) moderate | 1.2193 | 0.9828 | 255 | 0.0118 | 0.0094 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | main | 1/1 | ok | 2.2680 | 0.9491 | (-0.5,39) moderate | 1.9052 | 0.9637 | 255 | 0.0169 | 0.0141 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | lualatex-lm | pipeline | 1/1 | ok | 1.3483 | 0.9800 | (-0.5,-0.5) moderate | 1.2485 | 0.9823 | 255 | 0.0119 | 0.0095 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | de1020c | 1/1 | ok | 2.4854 | 0.9504 | (-0.5,38.5) moderate | 1.8777 | 0.9688 | 255 | 0.0174 | 0.0146 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | exact | 1/1 | ok | 1.8513 | 0.9716 | (-1.5,-1) moderate | 1.7481 | 0.9731 | 255 | 0.0137 | 0.0115 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | main | 1/1 | ok | 2.5100 | 0.9498 | (0,38.5) | 1.8502 | 0.9691 | 255 | 0.0175 | 0.0147 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex | pipeline | 1/1 | ok | 1.9422 | 0.9705 | (1,-1) moderate | 1.7766 | 0.9733 | 255 | 0.0141 | 0.0119 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | de1020c | 1/1 | ok | 2.2522 | 0.9480 | (-0.5,39.5) moderate | 1.9062 | 0.9622 | 255 | 0.0167 | 0.0139 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | exact | 1/1 | ok | 1.1585 | 0.9839 | (-0.5,0) weak | 1.1377 | 0.9841 | 255 | 0.0110 | 0.0084 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | main | 1/1 | ok | 2.2668 | 0.9477 | (-0.5,39.5) moderate | 1.9051 | 0.9619 | 255 | 0.0168 | 0.0139 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | pdflatex-lm | pipeline | 1/1 | ok | 1.1781 | 0.9835 | (-0.5,0) weak | 1.1700 | 0.9836 | 255 | 0.0111 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | de1020c | 1/1 | ok | 2.4838 | 0.9503 | (-0.5,38.5) | 1.8423 | 0.9692 | 255 | 0.0174 | 0.0146 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | exact | 1/1 | ok | 1.8628 | 0.9713 | (-1.5,-1) moderate | 1.7439 | 0.9731 | 255 | 0.0139 | 0.0115 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | main | 1/1 | ok | 2.5096 | 0.9498 | (0,38.5) | 1.8171 | 0.9694 | 255 | 0.0175 | 0.0147 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex | pipeline | 1/1 | ok | 1.9487 | 0.9703 | (1,-1) moderate | 1.7690 | 0.9732 | 255 | 0.0142 | 0.0119 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | de1020c | 1/1 | ok | 2.2538 | 0.9494 | (-0.5,39) moderate | 1.9067 | 0.9641 | 255 | 0.0169 | 0.0140 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | exact | 1/1 | ok | 1.3284 | 0.9804 | (-0.5,-0.5) moderate | 1.2194 | 0.9828 | 255 | 0.0118 | 0.0094 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | main | 1/1 | ok | 2.2680 | 0.9491 | (-0.5,39) moderate | 1.9052 | 0.9637 | 255 | 0.0169 | 0.0141 | -/-/- | - | - | - | - | 0/0 | - |
| 09-mixed-document | xelatex-lm | pipeline | 1/1 | ok | 1.3480 | 0.9800 | (-0.5,-0.5) moderate | 1.2486 | 0.9823 | 255 | 0.0119 | 0.0095 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | lualatex | de1020c | 1/1 | ok | 3.3162 | 0.9491 | (0,0) | 3.3162 | 0.9491 | 255 | 0.0261 | 0.0212 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | lualatex | exact | 1/1 | recovered | 3.3953 | 0.9472 | (0,0) | 3.3953 | 0.9472 | 255 | 0.0265 | 0.0214 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex | main | 1/1 | recovered | 3.3339 | 0.9494 | (0.5,0) weak | 3.2483 | 0.9508 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex | pipeline | 1/1 | recovered | 3.3953 | 0.9472 | (0,0) | 3.3953 | 0.9472 | 255 | 0.0265 | 0.0214 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | lualatex-lm | de1020c | 1/1 | ok | 3.2405 | 0.9447 | (0,0) | 3.2405 | 0.9447 | 255 | 0.0259 | 0.0211 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | lualatex-lm | exact | 1/1 | recovered | 2.3106 | 0.9658 | (-0.5,0) moderate | 2.1874 | 0.9679 | 255 | 0.0218 | 0.0166 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | lualatex-lm | main | 1/1 | recovered | 3.2500 | 0.9449 | (-5,0) weak | 3.2164 | 0.9449 | 255 | 0.0259 | 0.0212 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | lualatex-lm | pipeline | 1/1 | recovered | 2.3106 | 0.9658 | (-0.5,0) moderate | 2.1874 | 0.9679 | 255 | 0.0218 | 0.0166 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | pdflatex | de1020c | 1/1 | ok | 3.3644 | 0.9486 | (0,0) | 3.3644 | 0.9486 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | pdflatex | exact | 1/1 | recovered | 3.3525 | 0.9480 | (0,0) | 3.3525 | 0.9480 | 255 | 0.0263 | 0.0211 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | pdflatex | main | 1/1 | recovered | 3.3245 | 0.9498 | (0,0) | 3.3245 | 0.9498 | 255 | 0.0261 | 0.0212 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | pdflatex | pipeline | 1/1 | recovered | 3.3525 | 0.9480 | (0,0) | 3.3525 | 0.9480 | 255 | 0.0263 | 0.0211 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | pdflatex-lm | de1020c | 1/1 | ok | 3.2558 | 0.9444 | (0,0) | 3.2558 | 0.9444 | 255 | 0.0260 | 0.0212 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | pdflatex-lm | exact | 1/1 | recovered | 2.3773 | 0.9644 | (-0.5,0) moderate | 2.2194 | 0.9671 | 255 | 0.0221 | 0.0169 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | pdflatex-lm | main | 1/1 | recovered | 3.2657 | 0.9446 | (0,0) | 3.2657 | 0.9446 | 255 | 0.0260 | 0.0213 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | pdflatex-lm | pipeline | 1/1 | recovered | 2.3773 | 0.9644 | (-0.5,0) moderate | 2.2194 | 0.9671 | 255 | 0.0221 | 0.0169 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | xelatex | de1020c | 1/1 | ok | 3.3626 | 0.9484 | (0,0) | 3.3626 | 0.9484 | 255 | 0.0262 | 0.0213 | -/-/- | - | - | - | - | 3/1 | - |
| 10-unicode-paragraph | xelatex | exact | 1/1 | recovered | 3.3997 | 0.9470 | (0,0) | 3.3997 | 0.9470 | 255 | 0.0265 | 0.0214 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | xelatex | main | 1/1 | recovered | 3.3247 | 0.9495 | (0.5,0) weak | 3.2367 | 0.9510 | 255 | 0.0261 | 0.0212 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | xelatex | pipeline | 1/1 | recovered | 3.3997 | 0.9470 | (0,0) | 3.3997 | 0.9470 | 255 | 0.0265 | 0.0214 | -/-/- | - | - | - | - | 3/0 | - |
| 10-unicode-paragraph | xelatex-lm | de1020c | 1/1 | ok | 3.2406 | 0.9446 | (0,0) | 3.2406 | 0.9446 | 255 | 0.0259 | 0.0211 | -/-/- | - | - | - | - | 0/1 | - |
| 10-unicode-paragraph | xelatex-lm | exact | 1/1 | recovered | 2.3112 | 0.9658 | (-0.5,0) moderate | 2.1873 | 0.9679 | 255 | 0.0218 | 0.0166 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | xelatex-lm | main | 1/1 | recovered | 3.2502 | 0.9449 | (-5,0) weak | 3.2164 | 0.9449 | 255 | 0.0259 | 0.0212 | -/-/- | - | - | - | - | 0/0 | - |
| 10-unicode-paragraph | xelatex-lm | pipeline | 1/1 | recovered | 2.3112 | 0.9658 | (-0.5,0) moderate | 2.1873 | 0.9679 | 255 | 0.0218 | 0.0166 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | de1020c | 1/1 | recovered | 1.4652 | 0.9696 | (18,-49) weak | 1.4604 | 0.9709 | 255 | 0.0104 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | exact | 1/1 | ok | 1.4785 | 0.9703 | (-16.5,-20) moderate | 1.3614 | 0.9753 | 255 | 0.0104 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | main | 1/1 | ok | 1.4714 | 0.9699 | (-22.5,-8) moderate | 1.1219 | 0.9800 | 255 | 0.0105 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex | pipeline | 1/1 | ok | 1.4785 | 0.9703 | (-16.5,-20) moderate | 1.3614 | 0.9753 | 255 | 0.0104 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (-2,-49) weak | 1.3068 | 0.9711 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | exact | 1/1 | ok | 1.3303 | 0.9702 | (-17,-20) moderate | 1.1998 | 0.9761 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | main | 1/1 | ok | 1.3743 | 0.9689 | (-30.5,-8) moderate | 1.2508 | 0.9755 | 255 | 0.0102 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | lualatex-lm | pipeline | 1/1 | ok | 1.3303 | 0.9702 | (-17,-20) moderate | 1.1998 | 0.9761 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | de1020c | 1/1 | recovered | 1.4655 | 0.9698 | (19,-49) weak | 1.4558 | 0.9711 | 255 | 0.0104 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | exact | 1/1 | ok | 1.4480 | 0.9709 | (-16.5,-20) moderate | 1.3600 | 0.9754 | 255 | 0.0103 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | main | 1/1 | ok | 1.4603 | 0.9704 | (-22,-8) moderate | 1.1820 | 0.9790 | 255 | 0.0104 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex | pipeline | 1/1 | ok | 1.4480 | 0.9709 | (-16.5,-20) moderate | 1.3600 | 0.9754 | 255 | 0.0103 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | de1020c | 1/1 | recovered | 1.3674 | 0.9687 | (23.5,-49) weak | 1.3215 | 0.9706 | 255 | 0.0101 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | exact | 1/1 | ok | 1.3363 | 0.9701 | (-17,-20) moderate | 1.2000 | 0.9760 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | main | 1/1 | ok | 1.3761 | 0.9690 | (-30,-8) moderate | 1.2546 | 0.9752 | 255 | 0.0102 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | pdflatex-lm | pipeline | 1/1 | ok | 1.3363 | 0.9701 | (-17,-20) moderate | 1.2000 | 0.9760 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | de1020c | 1/1 | recovered | 1.4648 | 0.9696 | (18,-49) weak | 1.4599 | 0.9709 | 255 | 0.0104 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | exact | 1/1 | ok | 1.4783 | 0.9703 | (-16.5,-20) moderate | 1.3609 | 0.9753 | 255 | 0.0104 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | main | 1/1 | ok | 1.4708 | 0.9699 | (-22.5,-8) moderate | 1.1192 | 0.9800 | 255 | 0.0105 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex | pipeline | 1/1 | ok | 1.4783 | 0.9703 | (-16.5,-20) moderate | 1.3609 | 0.9753 | 255 | 0.0104 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | de1020c | 1/1 | recovered | 1.3675 | 0.9687 | (1,-49) weak | 1.3138 | 0.9708 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | exact | 1/1 | ok | 1.3303 | 0.9702 | (-17.5,-20) moderate | 1.1889 | 0.9763 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | main | 1/1 | ok | 1.3743 | 0.9690 | (-30.5,-8) moderate | 1.2509 | 0.9755 | 255 | 0.0101 | 0.0085 | -/-/- | - | - | - | - | 0/0 | - |
| 11-nested-lists | xelatex-lm | pipeline | 1/1 | ok | 1.3303 | 0.9702 | (-17.5,-20) moderate | 1.1889 | 0.9763 | 255 | 0.0100 | 0.0083 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | de1020c | 1/1 | ok | 15.4036 | 0.7397 | (-3,0) weak | 15.3867 | 0.7398 | 255 | 0.1111 | 0.0927 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | exact | 1/1 | ok | 12.9682 | 0.7909 | (0,0) | 12.9682 | 0.7909 | 255 | 0.0988 | 0.0805 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | main | 1/1 | ok | 14.8426 | 0.7604 | (-3.5,0) weak | 14.8089 | 0.7605 | 255 | 0.1081 | 0.0901 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex | pipeline | 1/1 | ok | 12.9682 | 0.7909 | (0,0) | 12.9682 | 0.7909 | 255 | 0.0988 | 0.0805 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | de1020c | 1/1 | ok | 13.7975 | 0.7504 | (-8.5,2.5) weak | 13.7722 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | exact | 1/1 | ok | 7.8034 | 0.8916 | (0,0) | 7.8034 | 0.8916 | 255 | 0.0767 | 0.0577 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | main | 1/1 | ok | 13.7769 | 0.7470 | (-1,0) weak | 13.6905 | 0.7481 | 255 | 0.1054 | 0.0872 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | lualatex-lm | pipeline | 1/1 | ok | 7.8034 | 0.8916 | (0,0) | 7.8034 | 0.8916 | 255 | 0.0767 | 0.0577 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | de1020c | 1/1 | ok | 15.3703 | 0.7416 | (-3,0) weak | 15.3659 | 0.7415 | 255 | 0.1107 | 0.0925 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | exact | 1/1 | ok | 13.0299 | 0.7911 | (1,14.5) weak | 12.9225 | 0.7936 | 255 | 0.0986 | 0.0807 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | main | 1/1 | ok | 14.8942 | 0.7603 | (-4,0) weak | 14.8307 | 0.7612 | 255 | 0.1079 | 0.0901 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex | pipeline | 1/1 | ok | 13.0299 | 0.7911 | (1,14.5) weak | 12.9225 | 0.7936 | 255 | 0.0986 | 0.0807 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | de1020c | 1/1 | ok | 13.7973 | 0.7504 | (-8.5,2.5) weak | 13.7711 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | exact | 1/1 | ok | 7.8197 | 0.8914 | (0,0) | 7.8197 | 0.8914 | 255 | 0.0767 | 0.0578 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | main | 1/1 | ok | 13.7775 | 0.7470 | (-0.5,0) weak | 13.6919 | 0.7482 | 255 | 0.1054 | 0.0872 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | pdflatex-lm | pipeline | 1/1 | ok | 7.8197 | 0.8914 | (0,0) | 7.8197 | 0.8914 | 255 | 0.0767 | 0.0578 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | de1020c | 1/1 | ok | 15.3817 | 0.7405 | (0,0) | 15.3817 | 0.7405 | 255 | 0.1110 | 0.0928 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | exact | 1/1 | ok | 13.0477 | 0.7900 | (0.5,14.5) weak | 12.9777 | 0.7921 | 255 | 0.0990 | 0.0810 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | main | 1/1 | ok | 14.9414 | 0.7591 | (0,0) | 14.9414 | 0.7591 | 255 | 0.1084 | 0.0906 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex | pipeline | 1/1 | ok | 13.0477 | 0.7900 | (0.5,14.5) weak | 12.9777 | 0.7921 | 255 | 0.0990 | 0.0810 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | de1020c | 1/1 | ok | 13.7974 | 0.7504 | (-8.5,2.5) weak | 13.7718 | 0.7504 | 255 | 0.1051 | 0.0873 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | exact | 1/1 | ok | 7.8036 | 0.8916 | (0,0) | 7.8036 | 0.8916 | 255 | 0.0766 | 0.0577 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | main | 1/1 | ok | 13.7772 | 0.7470 | (-1,0) weak | 13.6904 | 0.7481 | 255 | 0.1054 | 0.0872 | -/-/- | - | - | - | - | 0/0 | - |
| 12-justified-paragraphs | xelatex-lm | pipeline | 1/1 | ok | 7.8036 | 0.8916 | (0,0) | 7.8036 | 0.8916 | 255 | 0.0766 | 0.0577 | -/-/- | - | - | - | - | 0/0 | - |
| 13-math-display-rich | lualatex | de1020c | 1/1 | recovered | 0.4665 | 0.9904 | (0,3.5) moderate | 0.4400 | 0.9913 | 255 | 0.0040 | 0.0031 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex | exact | 1/1 | recovered | 0.5550 | 0.9884 | (-1.5,-0.5) weak | 0.5442 | 0.9886 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex | main | 1/1 | recovered | 0.5037 | 0.9895 | (-0.5,3.5) weak | 0.4862 | 0.9904 | 255 | 0.0042 | 0.0033 | -/-/- | - | - | - | - | 2/0 | - |
| 13-math-display-rich | lualatex | pipeline | 1/1 | recovered | 0.5550 | 0.9884 | (-1.5,-0.5) weak | 0.5442 | 0.9886 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | lualatex-lm | de1020c | 1/1 | recovered | 0.5457 | 0.9888 | (0,0) | 0.5457 | 0.9888 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | lualatex-lm | exact | 1/1 | recovered | 0.5201 | 0.9890 | (-0.5,0) moderate | 0.4624 | 0.9902 | 255 | 0.0042 | 0.0034 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | lualatex-lm | main | 1/1 | recovered | 0.5568 | 0.9883 | (-0.5,3.5) weak | 0.5416 | 0.9888 | 255 | 0.0044 | 0.0036 | -/-/- | - | - | - | - | 4/0 | - |
| 13-math-display-rich | lualatex-lm | pipeline | 1/1 | recovered | 0.5201 | 0.9890 | (-0.5,0) moderate | 0.4624 | 0.9902 | 255 | 0.0042 | 0.0034 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | pdflatex | de1020c | 1/1 | recovered | 0.4914 | 0.9901 | (1,3.5) weak | 0.4902 | 0.9905 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | pdflatex | exact | 1/1 | recovered | 0.5236 | 0.9888 | (0,0) | 0.5236 | 0.9888 | 255 | 0.0042 | 0.0034 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | pdflatex | main | 1/1 | recovered | 0.4829 | 0.9899 | (0,3.5) moderate | 0.4561 | 0.9908 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/0 | - |
| 13-math-display-rich | pdflatex | pipeline | 1/1 | recovered | 0.5236 | 0.9888 | (0,0) | 0.5236 | 0.9888 | 255 | 0.0042 | 0.0034 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | pdflatex-lm | de1020c | 1/1 | recovered | 0.5476 | 0.9889 | (-2,4) weak | 0.5407 | 0.9892 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | pdflatex-lm | exact | 1/1 | recovered | 0.5149 | 0.9891 | (-0.5,0) moderate | 0.4541 | 0.9903 | 255 | 0.0041 | 0.0034 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | pdflatex-lm | main | 1/1 | recovered | 0.5594 | 0.9883 | (-0.5,4) weak | 0.5475 | 0.9886 | 255 | 0.0044 | 0.0036 | -/-/- | - | - | - | - | 4/0 | - |
| 13-math-display-rich | pdflatex-lm | pipeline | 1/1 | recovered | 0.5149 | 0.9891 | (-0.5,0) moderate | 0.4541 | 0.9903 | 255 | 0.0041 | 0.0034 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | xelatex | de1020c | 1/1 | recovered | 0.4850 | 0.9901 | (0,0) | 0.4850 | 0.9901 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex | exact | 1/1 | recovered | 0.5323 | 0.9887 | (0,0) | 0.5323 | 0.9887 | 255 | 0.0043 | 0.0034 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex | main | 1/1 | recovered | 0.4784 | 0.9899 | (0,3.5) moderate | 0.4351 | 0.9910 | 255 | 0.0041 | 0.0032 | -/-/- | - | - | - | - | 2/0 | - |
| 13-math-display-rich | xelatex | pipeline | 1/1 | recovered | 0.5323 | 0.9887 | (0,0) | 0.5323 | 0.9887 | 255 | 0.0043 | 0.0034 | -/-/- | - | - | - | - | 2/2 | - |
| 13-math-display-rich | xelatex-lm | de1020c | 1/1 | recovered | 0.5459 | 0.9888 | (0,0) | 0.5459 | 0.9888 | 255 | 0.0043 | 0.0035 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | xelatex-lm | exact | 1/1 | recovered | 0.5203 | 0.9890 | (-0.5,0) moderate | 0.4625 | 0.9902 | 255 | 0.0042 | 0.0034 | -/-/- | - | - | - | - | 4/2 | - |
| 13-math-display-rich | xelatex-lm | main | 1/1 | recovered | 0.5569 | 0.9883 | (-0.5,3.5) weak | 0.5415 | 0.9888 | 255 | 0.0044 | 0.0036 | -/-/- | - | - | - | - | 4/0 | - |
| 13-math-display-rich | xelatex-lm | pipeline | 1/1 | recovered | 0.5203 | 0.9890 | (-0.5,0) moderate | 0.4625 | 0.9902 | 255 | 0.0042 | 0.0034 | -/-/- | - | - | - | - | 4/2 | - |
| 14-math-inline-dense | lualatex | de1020c | 1/1 | recovered | 1.3153 | 0.9706 | (-34.5,13.5) | 0.9661 | 0.9780 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex | exact | 1/1 | recovered | 0.9846 | 0.9806 | (0,0) | 0.9846 | 0.9806 | 255 | 0.0082 | 0.0065 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | lualatex | main | 1/1 | recovered | 1.3057 | 0.9707 | (-34.5,13.5) | 0.9504 | 0.9783 | 255 | 0.0099 | 0.0081 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | lualatex | pipeline | 1/1 | recovered | 0.9846 | 0.9806 | (0,0) | 0.9846 | 0.9806 | 255 | 0.0082 | 0.0065 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | lualatex-lm | de1020c | 1/1 | recovered | 1.2497 | 0.9701 | (2.5,13.5) moderate | 1.1687 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | lualatex-lm | exact | 1/1 | recovered | 0.7853 | 0.9850 | (0,0) | 0.7853 | 0.9850 | 255 | 0.0073 | 0.0056 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | lualatex-lm | main | 1/1 | recovered | 1.2412 | 0.9704 | (5,13.5) moderate | 1.1613 | 0.9733 | 255 | 0.0096 | 0.0079 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | lualatex-lm | pipeline | 1/1 | recovered | 0.7853 | 0.9850 | (0,0) | 0.7853 | 0.9850 | 255 | 0.0073 | 0.0056 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | pdflatex | de1020c | 1/1 | recovered | 1.3096 | 0.9709 | (-34,13.5) | 0.9598 | 0.9781 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | pdflatex | exact | 1/1 | recovered | 0.9719 | 0.9808 | (0,0) | 0.9719 | 0.9808 | 255 | 0.0081 | 0.0064 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | pdflatex | main | 1/1 | recovered | 1.3004 | 0.9709 | (-34,13.5) | 0.9333 | 0.9785 | 255 | 0.0098 | 0.0081 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | pdflatex | pipeline | 1/1 | recovered | 0.9719 | 0.9808 | (0,0) | 0.9719 | 0.9808 | 255 | 0.0081 | 0.0064 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | pdflatex-lm | de1020c | 1/1 | recovered | 1.2493 | 0.9701 | (2.5,13.5) moderate | 1.1681 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | pdflatex-lm | exact | 1/1 | recovered | 0.7860 | 0.9850 | (0,0) | 0.7860 | 0.9850 | 255 | 0.0073 | 0.0056 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | pdflatex-lm | main | 1/1 | recovered | 1.2408 | 0.9704 | (5,13.5) moderate | 1.1612 | 0.9733 | 255 | 0.0096 | 0.0079 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | pdflatex-lm | pipeline | 1/1 | recovered | 0.7860 | 0.9850 | (0,0) | 0.7860 | 0.9850 | 255 | 0.0073 | 0.0056 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | xelatex | de1020c | 1/1 | recovered | 1.3153 | 0.9706 | (-34.5,13.5) | 0.9632 | 0.9781 | 255 | 0.0099 | 0.0082 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex | exact | 1/1 | recovered | 0.9846 | 0.9806 | (0,0) | 0.9846 | 0.9806 | 255 | 0.0082 | 0.0065 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | xelatex | main | 1/1 | recovered | 1.3057 | 0.9707 | (-34.5,13.5) | 0.9466 | 0.9783 | 255 | 0.0099 | 0.0081 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | xelatex | pipeline | 1/1 | recovered | 0.9846 | 0.9806 | (0,0) | 0.9846 | 0.9806 | 255 | 0.0082 | 0.0065 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | xelatex-lm | de1020c | 1/1 | recovered | 1.2496 | 0.9701 | (2.5,13.5) moderate | 1.1687 | 0.9731 | 255 | 0.0097 | 0.0080 | -/-/- | - | - | - | - | 2/1 | - |
| 14-math-inline-dense | xelatex-lm | exact | 1/1 | recovered | 0.7852 | 0.9850 | (0,0) | 0.7852 | 0.9850 | 255 | 0.0073 | 0.0056 | -/-/- | - | - | - | - | 2/2 | - |
| 14-math-inline-dense | xelatex-lm | main | 1/1 | recovered | 1.2412 | 0.9704 | (5,13.5) moderate | 1.1613 | 0.9733 | 255 | 0.0096 | 0.0079 | -/-/- | - | - | - | - | 2/0 | - |
| 14-math-inline-dense | xelatex-lm | pipeline | 1/1 | recovered | 0.7852 | 0.9850 | (0,0) | 0.7852 | 0.9850 | 255 | 0.0073 | 0.0056 | -/-/- | - | - | - | - | 2/2 | - |
| 15-three-page-sections | lualatex | de1020c | 3/3 | recovered | 26.4238 | 0.5637 | (0.5,20) moderate; (0.5,19) weak; (0.5,-24.5) moderate | 25.2152 | 0.5876 | 255 | 0.1893 | 0.1584 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex | exact | 3/3 | recovered | 21.9555 | 0.6563 | (0.5,14) weak; (0.5,14) weak; (0.5,14) weak | 21.0352 | 0.6756 | 255 | 0.1657 | 0.1372 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex | main | 3/3 | recovered | 26.2996 | 0.5654 | (0,20) moderate; (0,19) weak; (0,-24.5) weak | 25.1074 | 0.5897 | 255 | 0.1893 | 0.1582 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex | pipeline | 3/3 | recovered | 21.9944 | 0.6558 | (0.5,14) weak; (0.5,14) weak; (0.5,14) weak | 21.0470 | 0.6753 | 255 | 0.1659 | 0.1373 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | lualatex-lm | de1020c | 3/3 | recovered | 24.2613 | 0.5495 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0400 | 0.5821 | 255 | 0.1832 | 0.1519 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | lualatex-lm | exact | 3/3 | recovered | 13.1004 | 0.8177 | (-0.5,0) weak; (-0.5,0) weak; (-0.5,0) weak | 13.0975 | 0.8160 | 255 | 0.1286 | 0.0970 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | lualatex-lm | main | 3/3 | recovered | 24.2775 | 0.5496 | (-0.5,22.5) moderate; (-0.5,-24) weak; (0,-24) weak | 22.9605 | 0.5840 | 255 | 0.1835 | 0.1522 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | lualatex-lm | pipeline | 3/3 | recovered | 13.1515 | 0.8171 | (0,0); (-0.5,0) weak; (-0.5,0) weak | 13.1505 | 0.8159 | 255 | 0.1288 | 0.0973 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex | de1020c | 3/3 | recovered | 26.6983 | 0.5543 | (0.5,36.5) moderate; (0.5,-24.5) weak; (0,-24.5) weak | 25.1309 | 0.5916 | 255 | 0.1901 | 0.1598 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex | exact | 3/3 | recovered | 21.6993 | 0.6649 | (1,-0.5) moderate; (1,-0.5) moderate; (1,-0.5) moderate | 20.5473 | 0.6876 | 255 | 0.1644 | 0.1361 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex | main | 3/3 | recovered | 26.7478 | 0.5542 | (0,36.5) moderate; (0,-24.5) moderate; (0,-24.5) moderate | 25.0626 | 0.5936 | 255 | 0.1907 | 0.1602 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex | pipeline | 3/3 | recovered | 21.7445 | 0.6643 | (1,-0.5) moderate; (1,-0.5) moderate; (1,-0.5) moderate | 20.5603 | 0.6874 | 255 | 0.1646 | 0.1363 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | pdflatex-lm | de1020c | 3/3 | recovered | 24.2616 | 0.5495 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0365 | 0.5822 | 255 | 0.1832 | 0.1521 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex-lm | exact | 3/3 | recovered | 13.1165 | 0.8176 | (-0.5,0) weak; (-0.5,0) weak; (-0.5,0) weak | 13.0474 | 0.8171 | 255 | 0.1288 | 0.0972 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex-lm | main | 3/3 | recovered | 24.2785 | 0.5495 | (-0.5,22.5) moderate; (-0.5,-24) weak; (-0.5,-24) weak | 22.9554 | 0.5839 | 255 | 0.1836 | 0.1523 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | pdflatex-lm | pipeline | 3/3 | recovered | 13.1678 | 0.8171 | (0,0); (-0.5,0) weak; (-0.5,0) weak | 13.1224 | 0.8167 | 255 | 0.1290 | 0.0974 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex | de1020c | 3/3 | recovered | 26.4204 | 0.5639 | (-3,20) moderate; (-3,19) weak; (-3,-24.5) moderate | 25.1894 | 0.5880 | 255 | 0.1893 | 0.1587 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex | exact | 3/3 | recovered | 22.0143 | 0.6553 | (1,14) weak; (1,14) weak; (1,14) weak | 20.9451 | 0.6764 | 255 | 0.1660 | 0.1376 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex | main | 3/3 | recovered | 26.3417 | 0.5650 | (0,20) moderate; (0,19) weak; (0,-24.5) weak | 25.1643 | 0.5890 | 255 | 0.1894 | 0.1586 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex | pipeline | 3/3 | recovered | 22.0532 | 0.6548 | (1,14) weak; (1,14) weak; (1,14) weak | 20.9569 | 0.6761 | 255 | 0.1662 | 0.1378 | -/-/- | - | - | - | - | 0/0 | - |
| 15-three-page-sections | xelatex-lm | de1020c | 3/3 | recovered | 24.2611 | 0.5495 | (0.5,22.5) moderate; (0.5,-24) weak; (0.5,-24) weak | 23.0399 | 0.5821 | 255 | 0.1831 | 0.1519 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex-lm | exact | 3/3 | recovered | 13.1046 | 0.8176 | (0,0); (-0.5,0) weak; (-0.5,0) weak | 13.0965 | 0.8166 | 255 | 0.1286 | 0.0971 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex-lm | main | 3/3 | recovered | 24.2773 | 0.5496 | (-0.5,22.5) moderate; (-0.5,-24) weak; (0,-24) weak | 22.9604 | 0.5840 | 255 | 0.1835 | 0.1521 | -/-/- | - | - | - | - | 3/0 | - |
| 15-three-page-sections | xelatex-lm | pipeline | 3/3 | recovered | 13.1558 | 0.8171 | (0,0); (0,0); (0,0) | 13.1558 | 0.8171 | 255 | 0.1288 | 0.0973 | -/-/- | - | - | - | - | 3/0 | - |
| 16-heading-page-break | lualatex | de1020c | 2/2 | ok | 20.5841 | 0.6613 | (0,0); (0,20.5) moderate | 20.0997 | 0.6688 | 255 | 0.1484 | 0.1240 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex | exact | 2/2 | ok | 17.5047 | 0.7213 | (0,0); (-1,57.5) weak | 17.3725 | 0.7261 | 255 | 0.1328 | 0.1087 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex | main | 2/2 | ok | 20.1421 | 0.6739 | (0,0); (0,14.5) weak | 20.0611 | 0.6773 | 255 | 0.1460 | 0.1217 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex | pipeline | 2/2 | ok | 17.5104 | 0.7211 | (0,0); (-1,57.5) weak | 17.3773 | 0.7260 | 255 | 0.1328 | 0.1087 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | lualatex-lm | de1020c | 2/2 | ok | 19.0167 | 0.6509 | (-1,0) weak; (-1,20) moderate | 18.0483 | 0.6755 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | lualatex-lm | exact | 2/2 | ok | 10.5614 | 0.8528 | (0,0); (0,0) | 10.5614 | 0.8528 | 255 | 0.1036 | 0.0780 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | lualatex-lm | main | 2/2 | ok | 18.3185 | 0.6672 | (-0.5,-27) weak; (0,0) | 18.3042 | 0.6668 | 255 | 0.1402 | 0.1159 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | lualatex-lm | pipeline | 2/2 | ok | 10.5861 | 0.8522 | (0,0); (0,0) | 10.5861 | 0.8522 | 255 | 0.1037 | 0.0781 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex | de1020c | 2/2 | ok | 20.5976 | 0.6621 | (0,-14.5) weak; (0,20.5) moderate | 20.0459 | 0.6704 | 255 | 0.1481 | 0.1240 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex | exact | 2/2 | ok | 17.4931 | 0.7230 | (-1.5,0) weak; (-1.5,57.5) weak | 17.2497 | 0.7298 | 255 | 0.1324 | 0.1086 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex | main | 2/2 | ok | 20.1484 | 0.6753 | (0,0); (0,14.5) weak | 20.0507 | 0.6788 | 255 | 0.1456 | 0.1217 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex | pipeline | 2/2 | ok | 17.4995 | 0.7228 | (-1.5,0) weak; (-1.5,57.5) weak | 17.2547 | 0.7297 | 255 | 0.1324 | 0.1086 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | pdflatex-lm | de1020c | 2/2 | ok | 19.0160 | 0.6507 | (-1,0) weak; (-1,20) moderate | 18.0489 | 0.6754 | 255 | 0.1439 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex-lm | exact | 2/2 | ok | 10.5789 | 0.8525 | (0,0); (0,0) | 10.5789 | 0.8525 | 255 | 0.1036 | 0.0781 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex-lm | main | 2/2 | ok | 18.3190 | 0.6671 | (-0.5,-27) weak; (0,0) | 18.3048 | 0.6667 | 255 | 0.1402 | 0.1159 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | pdflatex-lm | pipeline | 2/2 | ok | 10.6040 | 0.8520 | (0,0); (0,0) | 10.6040 | 0.8520 | 255 | 0.1037 | 0.0782 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex | de1020c | 2/2 | ok | 20.6109 | 0.6611 | (-3,-14.5) weak; (0.5,20.5) moderate | 20.1127 | 0.6683 | 255 | 0.1485 | 0.1243 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex | exact | 2/2 | ok | 17.5361 | 0.7208 | (1.5,0) weak; (-0.5,57.5) weak | 17.3322 | 0.7261 | 255 | 0.1329 | 0.1090 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex | main | 2/2 | ok | 20.1392 | 0.6740 | (0,0); (0,14.5) weak | 20.0559 | 0.6774 | 255 | 0.1460 | 0.1220 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex | pipeline | 2/2 | ok | 17.5417 | 0.7206 | (1.5,0) weak; (-1,57.5) weak | 17.3614 | 0.7256 | 255 | 0.1330 | 0.1090 | -/-/- | - | - | - | - | 0/0 | - |
| 16-heading-page-break | xelatex-lm | de1020c | 2/2 | ok | 19.0167 | 0.6509 | (-1,0) weak; (-1,20) moderate | 18.0480 | 0.6754 | 255 | 0.1438 | 0.1195 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex-lm | exact | 2/2 | ok | 10.5643 | 0.8527 | (0,0); (0,0) | 10.5643 | 0.8527 | 255 | 0.1035 | 0.0780 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex-lm | main | 2/2 | ok | 18.3187 | 0.6672 | (-0.5,-27) weak; (0,0) | 18.3042 | 0.6668 | 255 | 0.1402 | 0.1159 | -/-/- | - | - | - | - | 1/0 | - |
| 16-heading-page-break | xelatex-lm | pipeline | 2/2 | ok | 10.5890 | 0.8522 | (0,0); (0,0) | 10.5890 | 0.8522 | 255 | 0.1037 | 0.0781 | -/-/- | - | - | - | - | 1/0 | - |
| 17-apostrophes | lualatex | de1020c | 1/1 | ok | 0.8503 | 0.9875 | (-1.5,0) weak | 0.8116 | 0.9882 | 255 | 0.0068 | 0.0053 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex | exact | 1/1 | ok | 0.9462 | 0.9851 | (53.5,0) moderate | 0.8874 | 0.9859 | 255 | 0.0073 | 0.0058 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex | main | 1/1 | ok | 0.8494 | 0.9875 | (-7.5,0) weak | 0.8284 | 0.9876 | 255 | 0.0068 | 0.0053 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex | pipeline | 1/1 | ok | 0.9462 | 0.9851 | (53.5,0) moderate | 0.8874 | 0.9859 | 255 | 0.0073 | 0.0058 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | de1020c | 1/1 | ok | 0.8624 | 0.9848 | (0,0) | 0.8624 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | exact | 1/1 | ok | 0.6795 | 0.9896 | (-0.5,0) moderate | 0.6343 | 0.9899 | 255 | 0.0061 | 0.0048 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | main | 1/1 | ok | 0.8761 | 0.9846 | (0,0) | 0.8761 | 0.9846 | 255 | 0.0071 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | lualatex-lm | pipeline | 1/1 | ok | 0.6795 | 0.9896 | (-0.5,0) moderate | 0.6343 | 0.9899 | 255 | 0.0061 | 0.0048 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | de1020c | 1/1 | ok | 0.8064 | 0.9887 | (0,0) | 0.8064 | 0.9887 | 255 | 0.0066 | 0.0052 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | exact | 1/1 | ok | 0.9313 | 0.9857 | (0,0) | 0.9313 | 0.9857 | 255 | 0.0072 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | main | 1/1 | ok | 0.8140 | 0.9885 | (0,0) | 0.8140 | 0.9885 | 255 | 0.0067 | 0.0053 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex | pipeline | 1/1 | ok | 0.9313 | 0.9857 | (0,0) | 0.9313 | 0.9857 | 255 | 0.0072 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | de1020c | 1/1 | ok | 0.8626 | 0.9848 | (0,0) | 0.8626 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | exact | 1/1 | ok | 0.6789 | 0.9896 | (-0.5,0) moderate | 0.6342 | 0.9899 | 255 | 0.0061 | 0.0048 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | main | 1/1 | ok | 0.8763 | 0.9846 | (0,0) | 0.8763 | 0.9846 | 255 | 0.0071 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | pdflatex-lm | pipeline | 1/1 | ok | 0.6789 | 0.9896 | (-0.5,0) moderate | 0.6342 | 0.9899 | 255 | 0.0061 | 0.0048 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | de1020c | 1/1 | ok | 0.7982 | 0.9883 | (0,0) | 0.7982 | 0.9883 | 255 | 0.0066 | 0.0051 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | exact | 1/1 | ok | 0.9371 | 0.9853 | (53.5,0) weak | 0.9051 | 0.9856 | 255 | 0.0073 | 0.0058 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | main | 1/1 | ok | 0.8071 | 0.9882 | (0,0) | 0.8071 | 0.9882 | 255 | 0.0067 | 0.0051 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex | pipeline | 1/1 | ok | 0.9371 | 0.9853 | (53.5,0) weak | 0.9051 | 0.9856 | 255 | 0.0073 | 0.0058 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | de1020c | 1/1 | ok | 0.8621 | 0.9848 | (0,0) | 0.8621 | 0.9848 | 255 | 0.0070 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | exact | 1/1 | ok | 0.6794 | 0.9896 | (-0.5,0) moderate | 0.6338 | 0.9899 | 255 | 0.0061 | 0.0048 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | main | 1/1 | ok | 0.8760 | 0.9846 | (0,0) | 0.8760 | 0.9846 | 255 | 0.0071 | 0.0057 | -/-/- | - | - | - | - | 0/0 | - |
| 17-apostrophes | xelatex-lm | pipeline | 1/1 | ok | 0.6794 | 0.9896 | (-0.5,0) moderate | 0.6338 | 0.9899 | 255 | 0.0061 | 0.0048 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | de1020c | 1/1 | ok | 1.3878 | 0.9788 | (0,0) | 1.3878 | 0.9788 | 255 | 0.0109 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | exact | 1/1 | ok | 1.3858 | 0.9785 | (0,0) | 1.3858 | 0.9785 | 255 | 0.0110 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | main | 1/1 | ok | 1.1884 | 0.9831 | (-0.5,0) moderate | 1.1154 | 0.9842 | 255 | 0.0101 | 0.0077 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex | pipeline | 1/1 | ok | 1.3858 | 0.9785 | (0,0) | 1.3858 | 0.9785 | 255 | 0.0110 | 0.0087 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | de1020c | 1/1 | ok | 1.3042 | 0.9789 | (0,0) | 1.3042 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | exact | 1/1 | ok | 0.8648 | 0.9881 | (0,0) | 0.8648 | 0.9881 | 255 | 0.0087 | 0.0065 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | main | 1/1 | ok | 1.3761 | 0.9771 | (-2.5,0) weak | 1.3651 | 0.9775 | 255 | 0.0109 | 0.0090 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | lualatex-lm | pipeline | 1/1 | ok | 0.8648 | 0.9881 | (0,0) | 0.8648 | 0.9881 | 255 | 0.0087 | 0.0065 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | de1020c | 1/1 | ok | 1.2717 | 0.9812 | (0,0) | 1.2717 | 0.9812 | 255 | 0.0104 | 0.0080 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | exact | 1/1 | ok | 1.3760 | 0.9793 | (0,0) | 1.3760 | 0.9793 | 255 | 0.0108 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | main | 1/1 | ok | 1.3112 | 0.9812 | (-2,0) moderate | 1.1522 | 0.9842 | 255 | 0.0105 | 0.0082 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex | pipeline | 1/1 | ok | 1.3760 | 0.9793 | (0,0) | 1.3760 | 0.9793 | 255 | 0.0108 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | de1020c | 1/1 | ok | 1.3056 | 0.9789 | (0,0) | 1.3056 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | exact | 1/1 | ok | 0.8670 | 0.9881 | (0,0) | 0.8670 | 0.9881 | 255 | 0.0087 | 0.0065 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | main | 1/1 | ok | 1.3757 | 0.9771 | (-5.5,0) weak | 1.3586 | 0.9773 | 255 | 0.0109 | 0.0090 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | pdflatex-lm | pipeline | 1/1 | ok | 0.8670 | 0.9881 | (0,0) | 0.8670 | 0.9881 | 255 | 0.0087 | 0.0065 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | de1020c | 1/1 | ok | 1.3346 | 0.9799 | (0,0) | 1.3346 | 0.9799 | 255 | 0.0107 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | exact | 1/1 | ok | 1.3838 | 0.9786 | (0,0) | 1.3838 | 0.9786 | 255 | 0.0110 | 0.0088 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | main | 1/1 | ok | 1.3063 | 0.9812 | (-1,0) moderate | 1.1838 | 0.9832 | 255 | 0.0105 | 0.0084 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex | pipeline | 1/1 | ok | 1.3838 | 0.9786 | (0,0) | 1.3838 | 0.9786 | 255 | 0.0110 | 0.0088 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | de1020c | 1/1 | ok | 1.3041 | 0.9789 | (0,0) | 1.3041 | 0.9789 | 255 | 0.0106 | 0.0086 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | exact | 1/1 | ok | 0.8650 | 0.9881 | (0,0) | 0.8650 | 0.9881 | 255 | 0.0087 | 0.0065 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | main | 1/1 | ok | 1.3760 | 0.9771 | (-2.5,0) weak | 1.3651 | 0.9775 | 255 | 0.0109 | 0.0090 | -/-/- | - | - | - | - | 0/0 | - |
| 18-ligatures | xelatex-lm | pipeline | 1/1 | ok | 0.8650 | 0.9881 | (0,0) | 0.8650 | 0.9881 | 255 | 0.0087 | 0.0065 | -/-/- | - | - | - | - | 0/0 | - |

## Per-fixture diagnostic details (export side)

### 01-plain-paragraph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935537, 648, 477, 427, 319, 237, 160, 146, 149, 113, 138, 79, 113, 76, 54, 143]`; ink px ref/ours 2442/2421 (ratio 0.9914); SSIM blocks <0.9: 131/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.95, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1575, differing 0.002279, SSIM₈ 0.998 (raw 0.1575, 0.002279, 0.998)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9969→0.9969 / 0.2517→0.2517; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `This` dx -0.75 dy 0.46; `is` dx -0.65 dy 0.46; `one` dx -0.62 dy 0.46

### 01-plain-paragraph — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933659, 453, 367, 397, 366, 316, 332, 274, 316, 238, 294, 284, 216, 203, 270, 831]`; ink px ref/ours 2442/1906 (ratio 0.7805); SSIM blocks <0.9: 224/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3748, not lower; centroid estimate [7.99, 0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3675, differing 0.003051, SSIM₈ 0.9942 (raw 0.3675, 0.003051, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9907→0.9907 / 0.5874→0.5874; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 24.88 dy 0.77; `single` dx 23.84 dy 0.77; `a` dx 22.38 dy 0.77

### 01-plain-paragraph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934633, 486, 426, 392, 398, 266, 276, 245, 196, 204, 148, 170, 179, 189, 168, 440]`; ink px ref/ours 2442/2408 (ratio 0.9861); SSIM blocks <0.9: 176/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.91, -0.01] pt); confidence strong (shift explains 42% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1503, differing 0.002291, SSIM₈ 0.9983 (raw 0.2612, 0.002629, 0.9965)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9944→0.9973 / 0.4175→0.2402; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `This` dx -0.87 dy 0.46; `on` dx -0.86 dy 0.46; `a` dx -0.81 dy 0.46

### 01-plain-paragraph — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933596, 518, 337, 402, 343, 318, 288, 237, 238, 231, 223, 295, 274, 213, 292, 1011]`; ink px ref/ours 2442/2427 (ratio 0.9939); SSIM blocks <0.9: 217/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [9.91, 0.02] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3813, differing 0.003129, SSIM₈ 0.9944 (raw 0.385, 0.003136, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9909→0.991 / 0.6153→0.6094; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 24.88 dy 0.41; `single` dx 23.84 dy 0.41; `a` dx 22.38 dy 0.41

### 01-plain-paragraph — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933594, 461, 436, 345, 357, 334, 344, 324, 320, 261, 321, 233, 248, 209, 197, 832]`; ink px ref/ours 1890/2421 (ratio 1.281); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-10.52, -0.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3662, differing 0.003027, SSIM₈ 0.9941 (raw 0.3662, 0.003027, 0.9941)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9906→0.9906 / 0.5854→0.5854; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -24.74 dy -0.3; `single` dx -23.83 dy -0.3; `a` dx -22.41 dy -0.3

### 01-plain-paragraph — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938798, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 1890/1906 (ratio 1.0085); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.58, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0045, differing 0.001127, SSIM₈ 1.0 (raw 0.0045, 0.001127, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0072→0.0072; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `short` dx 0.03 dy 0.0; `is` dx 0.02 dy 0.0; `one` dx 0.02 dy 0.0

### 01-plain-paragraph — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933619, 479, 424, 376, 360, 326, 359, 327, 338, 262, 262, 240, 199, 242, 198, 805]`; ink px ref/ours 1890/2408 (ratio 1.2741); SSIM blocks <0.9: 232/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-10.49, -0.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3601, differing 0.003021, SSIM₈ 0.9942 (raw 0.3601, 0.003021, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9908→0.9908 / 0.5755→0.5755; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -25.52 dy -0.3; `single` dx -24.61 dy -0.3; `a` dx -23.19 dy -0.3

### 01-plain-paragraph — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934650, 542, 450, 391, 412, 346, 309, 227, 245, 206, 200, 149, 131, 73, 113, 372]`; ink px ref/ours 1890/2427 (ratio 1.2841); SSIM blocks <0.9: 184/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [1.34, -0.12] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2392, differing 0.002589, SSIM₈ 0.9966 (raw 0.2392, 0.002589, 0.9966)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9946→0.9946 / 0.3823→0.3823; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `short` dx 0.03 dy -0.36; `is` dx 0.02 dy -0.36; `one` dx 0.02 dy -0.36

### 01-plain-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935388, 716, 424, 359, 290, 196, 196, 126, 136, 124, 122, 86, 78, 86, 81, 408]`; ink px ref/ours 2408/2421 (ratio 1.0054); SSIM blocks <0.9: 119/30294; [overlay](images/01-plain-paragraph/pdflatex-de1020c-export-p1-overlay.png) (74757 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (69159 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.1, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1873, differing 0.002361, SSIM₈ 0.9975 (raw 0.1873, 0.002361, 0.9975)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9961→0.9961 / 0.2993→0.2993; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 1.15 dy 0.46; `single` dx 1.03 dy 0.46; `a` dx 1.0 dy 0.46

### 01-plain-paragraph — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933584, 476, 432, 302, 365, 338, 348, 325, 365, 265, 270, 226, 219, 202, 253, 846]`; ink px ref/ours 2408/1906 (ratio 0.7915); SSIM blocks <0.9: 231/30294; [overlay](images/01-plain-paragraph/pdflatex-exact-export-p1-overlay.png) (78123 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-exact-export-p1-heatmap.png) (72170 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [16.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3952, not lower; centroid estimate [8.84, 0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3691, differing 0.003057, SSIM₈ 0.9943 (raw 0.3691, 0.003057, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9908→0.9908 / 0.5899→0.5899; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 25.9 dy 0.77; `single` dx 24.87 dy 0.77; `a` dx 23.41 dy 0.77

### 01-plain-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935798, 790, 432, 279, 271, 238, 187, 198, 153, 120, 112, 82, 47, 65, 9, 35]`; ink px ref/ours 2408/2408 (ratio 1.0); SSIM blocks <0.9: 124/30294; [overlay](images/01-plain-paragraph/pdflatex-main-export-p1-overlay.png) (74593 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-main-export-p1-heatmap.png) (66029 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.07, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1285, differing 0.002179, SSIM₈ 0.9985 (raw 0.1285, 0.002179, 0.9985)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9976→0.9976 / 0.2053→0.2053; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `This` dx -0.48 dy 0.46; `is` dx -0.39 dy 0.46; `line.` dx 0.37 dy 0.46

### 01-plain-paragraph — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933679, 569, 294, 282, 282, 300, 277, 226, 261, 255, 215, 266, 270, 210, 292, 1138]`; ink px ref/ours 2408/2427 (ratio 1.0079); SSIM blocks <0.9: 213/30294; [overlay](images/01-plain-paragraph/pdflatex-pipeline-export-p1-overlay.png) (76278 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-pipeline-export-p1-heatmap.png) (73317 B, ÷1)
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [10.76, 0.02] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3792, differing 0.003097, SSIM₈ 0.9947 (raw 0.3927, 0.003122, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9916 / 0.6276→0.6056; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 25.9 dy 0.41; `single` dx 24.87 dy 0.41; `a` dx 23.41 dy 0.41

### 01-plain-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933593, 464, 437, 341, 371, 326, 338, 319, 328, 279, 309, 227, 246, 191, 211, 836]`; ink px ref/ours 1898/2421 (ratio 1.2756); SSIM blocks <0.9: 235/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (73990 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (70534 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-9.72, -0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.366, differing 0.003024, SSIM₈ 0.9942 (raw 0.366, 0.003024, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9907→0.9907 / 0.5849→0.5849; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -24.75 dy 0.73; `single` dx -23.83 dy 0.73; `a` dx -22.41 dy 0.73

### 01-plain-paragraph — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 1898/1906 (ratio 1.0042); SSIM blocks <0.9: 0/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-exact-export-p1-overlay.png) (74346 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-exact-export-p1-heatmap.png) (62449 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.22, 0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0015, differing 0.000905, SSIM₈ 1.0 (raw 0.0015, 0.000905, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0024→0.0024; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Hello` dx 0 dy 1.03; `world.` dx 0.0 dy 1.03; `This` dx 0.0 dy 1.03

### 01-plain-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933631, 484, 408, 370, 373, 317, 366, 314, 345, 291, 229, 241, 207, 219, 214, 807]`; ink px ref/ours 1898/2408 (ratio 1.2687); SSIM blocks <0.9: 233/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-main-export-p1-overlay.png) (73817 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (70439 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-9.69, -0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3597, differing 0.003018, SSIM₈ 0.9943 (raw 0.3597, 0.003018, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9908→0.9908 / 0.5749→0.5749; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -25.53 dy 0.73; `single` dx -24.61 dy 0.73; `a` dx -23.19 dy 0.73

### 01-plain-paragraph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934641, 547, 428, 417, 393, 354, 321, 220, 258, 197, 191, 159, 119, 80, 109, 382]`; ink px ref/ours 1898/2427 (ratio 1.2787); SSIM blocks <0.9: 184/30294; [overlay](images/01-plain-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) (76158 B, ÷1), [heatmap](images/01-plain-paragraph/pdflatex-lm-pipeline-export-p1-heatmap.png) (68623 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.14, -0.13] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2405, differing 0.002594, SSIM₈ 0.9966 (raw 0.2405, 0.002594, 0.9966)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9946→0.9946 / 0.3843→0.3843; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Hello` dx 0 dy 0.67; `world.` dx 0.0 dy 0.67; `This` dx 0.0 dy 0.67

### 01-plain-paragraph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935492, 657, 509, 378, 314, 240, 190, 128, 161, 125, 139, 77, 101, 98, 61, 146]`; ink px ref/ours 2441/2421 (ratio 0.9918); SSIM blocks <0.9: 134/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-2.11, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1607, differing 0.002287, SSIM₈ 0.998 (raw 0.1607, 0.002287, 0.998)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9968→0.9968 / 0.2569→0.2569; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `This` dx -0.75 dy 0.46; `is` dx -0.66 dy 0.46; `one` dx -0.62 dy 0.46

### 01-plain-paragraph — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933631, 466, 377, 382, 372, 337, 326, 284, 315, 226, 328, 231, 238, 207, 260, 836]`; ink px ref/ours 2441/1906 (ratio 0.7808); SSIM blocks <0.9: 223/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [7.82, 0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3677, differing 0.003047, SSIM₈ 0.9942 (raw 0.3677, 0.003047, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9907→0.9907 / 0.5877→0.5877; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 24.85 dy 0.77; `single` dx 23.81 dy 0.77; `a` dx 22.36 dy 0.77

### 01-plain-paragraph — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934582, 499, 419, 375, 397, 271, 279, 221, 230, 183, 165, 165, 180, 214, 179, 457]`; ink px ref/ours 2441/2408 (ratio 0.9865); SSIM blocks <0.9: 177/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.08, -0.01] pt); confidence strong (shift explains 43% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1528, differing 0.002283, SSIM₈ 0.9982 (raw 0.267, 0.002638, 0.9964)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9942→0.9972 / 0.4268→0.2443; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `on` dx -0.89 dy 0.46; `This` dx -0.87 dy 0.46; `a` dx -0.83 dy 0.46

### 01-plain-paragraph — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933595, 517, 336, 365, 378, 328, 282, 232, 250, 214, 256, 265, 288, 212, 285, 1013]`; ink px ref/ours 2441/2427 (ratio 0.9943); SSIM blocks <0.9: 217/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [9.75, 0.02] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3814, differing 0.003116, SSIM₈ 0.9943 (raw 0.3853, 0.003131, 0.9943)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9909→0.991 / 0.6158→0.6095; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 24.85 dy 0.41; `single` dx 23.81 dy 0.41; `a` dx 22.36 dy 0.41

### 01-plain-paragraph — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933595, 460, 437, 342, 360, 330, 346, 325, 305, 282, 301, 246, 245, 198, 207, 837]`; ink px ref/ours 1891/2421 (ratio 1.2803); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-10.52, -0.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3665, differing 0.003027, SSIM₈ 0.9941 (raw 0.3665, 0.003027, 0.9941)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9906→0.9906 / 0.5857→0.5857; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -24.74 dy 0.73; `single` dx -23.83 dy 0.73; `a` dx -22.41 dy 0.73

### 01-plain-paragraph — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 1891/1906 (ratio 1.0079); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.58, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0049, differing 0.001173, SSIM₈ 1.0 (raw 0.0049, 0.001173, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0079→0.0079; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `short` dx 0.03 dy 1.03; `is` dx 0.02 dy 1.03; `one` dx 0.02 dy 1.03

### 01-plain-paragraph — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933613, 488, 422, 373, 361, 323, 361, 329, 322, 280, 246, 253, 199, 228, 206, 812]`; ink px ref/ours 1891/2408 (ratio 1.2734); SSIM blocks <0.9: 232/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-10.49, -0.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3603, differing 0.003019, SSIM₈ 0.9942 (raw 0.3603, 0.003019, 0.9942)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9908→0.9908 / 0.5758→0.5758; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -25.52 dy 0.73; `single` dx -24.61 dy 0.73; `a` dx -23.19 dy 0.73

### 01-plain-paragraph — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934651, 556, 430, 395, 427, 328, 313, 226, 249, 206, 200, 145, 132, 73, 108, 377]`; ink px ref/ours 1891/2427 (ratio 1.2834); SSIM blocks <0.9: 184/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [1.34, -0.12] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2392, differing 0.002593, SSIM₈ 0.9966 (raw 0.2392, 0.002593, 0.9966)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9946→0.9946 / 0.3823→0.3823; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `short` dx 0.03 dy 0.67; `is` dx 0.02 dy 0.67; `one` dx 0.02 dy 0.67

### 02-wrapping-paragraph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831807, 8259, 7034, 6392, 5883, 5835, 5922, 5218, 5432, 5617, 5346, 5301, 5198, 5202, 5188, 25182]`; ink px ref/ours 39630/39464 (ratio 0.9958); SSIM blocks <0.9: 4366/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.86, 4.65] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.3713, differing 0.061277, SSIM₈ 0.8657 (raw 8.3842, 0.061305, 0.8656)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7852→0.7854 / 13.3998→13.3792; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1845173, 8005, 6958, 6461, 6060, 5553, 5874, 5702, 5560, 4820, 4798, 4474, 4392, 4368, 4088, 16530]`; ink px ref/ours 39630/30093 (ratio 0.7593); SSIM blocks <0.9: 3931/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [4.4, 3.81] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 6.786, differing 0.054285, SSIM₈ 0.8871 (raw 6.786, 0.054285, 0.8871)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8197→0.8197 / 10.8449→10.8449; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 15.21; `oak` dx -415.78 dy 15.21; `branch` dx -413.61 dy 15.21

### 02-wrapping-paragraph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831514, 8195, 7191, 6521, 5857, 5794, 5799, 5233, 5491, 5530, 5365, 5313, 5257, 5286, 5152, 25318]`; ink px ref/ours 39630/39525 (ratio 0.9974); SSIM blocks <0.9: 4341/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 8.4396, not lower; centroid estimate [-5.5, 4.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.4139, differing 0.061613, SSIM₈ 0.8657 (raw 8.4139, 0.061613, 0.8657)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7854→0.7854 / 13.4471→13.4471; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `the` dx -430.88 dy 20.5

### 02-wrapping-paragraph — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1843512, 8229, 7087, 6193, 5707, 5309, 5062, 5168, 4953, 4939, 4637, 4648, 4233, 4424, 4575, 20140]`; ink px ref/ours 39630/39245 (ratio 0.9903); SSIM blocks <0.9: 3873/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [3.37, 3.98] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.1755, differing 0.055429, SSIM₈ 0.8901 (raw 7.1755, 0.055429, 0.8901)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8247→0.8247 / 11.4669→11.4669; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 14.85; `oak` dx -415.78 dy 14.85; `branch` dx -413.61 dy 14.85

### 02-wrapping-paragraph — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834635, 8254, 7002, 6804, 6347, 6194, 6497, 6173, 6010, 5798, 5303, 5023, 4839, 4475, 4684, 20778]`; ink px ref/ours 30065/39464 (ratio 1.3126); SSIM blocks <0.9: 4465/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -14.5] pt by ink-projection correlation (centroid estimate [-12.29, 0.73] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7691, differing 0.059721, SSIM₈ 0.8602 (raw 7.7868, 0.059701, 0.862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7795→0.7767 / 12.445→12.4168; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -14.75; `oak` dx 426.94 dy -9.07; `jumps` dx 416.92 dy -14.79

### 02-wrapping-paragraph — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 30065/30093 (ratio 1.0009); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.03, -0.1] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0777, differing 0.021769, SSIM₈ 1.0 (raw 0.0777, 0.021769, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.1242→0.1242; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `old` dx 0.03 dy -0.0; `old` dx 0.03 dy -0.0; `over` dx 0.02 dy 0.0

### 02-wrapping-paragraph — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834664, 8119, 7066, 6869, 6302, 6214, 6405, 6114, 6192, 5700, 5266, 5008, 5008, 4594, 4592, 20703]`; ink px ref/ours 30065/39525 (ratio 1.3147); SSIM blocks <0.9: 4481/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-1.0, -14.5] pt REJECTED: applying it gives mean|Δ| 7.8165, not lower; centroid estimate [-9.93, 0.08] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7923, differing 0.059833, SSIM₈ 0.8614 (raw 7.7923, 0.059833, 0.8614)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7786→0.7786 / 12.4538→12.4538; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -14.75; `oak` dx 425.86 dy -9.07; `jumps` dx 414.46 dy -14.79

### 02-wrapping-paragraph — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1865298, 8419, 6888, 6498, 6503, 5638, 5139, 4842, 4223, 3915, 3128, 2930, 2806, 2491, 2275, 7823]`; ink px ref/ours 30065/39245 (ratio 1.3053); SSIM blocks <0.9: 3163/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.07, 0.06] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 4.5437, differing 0.044682, SSIM₈ 0.9367 (raw 4.5437, 0.044682, 0.9367)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8991→0.8991 / 7.2612→7.2612; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `old` dx 0.03 dy -0.36; `old` dx 0.03 dy -0.36; `over` dx 0.02 dy -0.36

### 02-wrapping-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831980, 8181, 7016, 6406, 5963, 5948, 5852, 5289, 5188, 5656, 5267, 5378, 5255, 4876, 5221, 25340]`; ink px ref/ours 39390/39464 (ratio 1.0019); SSIM blocks <0.9: 4347/30294; [overlay](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-overlay.png) (55223 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (42333 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 8.3959, not lower; centroid estimate [-7.52, 4.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.3726, differing 0.06114, SSIM₈ 0.8663 (raw 8.3726, 0.06114, 0.8663)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7864→0.7864 / 13.3813→13.3813; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `over` dx -407.04 dy 20.5

### 02-wrapping-paragraph — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1845448, 7947, 7118, 6377, 6232, 5762, 5621, 5718, 5398, 4902, 4529, 4509, 4313, 4108, 4079, 16755]`; ink px ref/ours 39390/30093 (ratio 0.764); SSIM blocks <0.9: 3915/30294; [overlay](images/02-wrapping-paragraph/pdflatex-exact-export-p1-overlay.png) (55256 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-exact-export-p1-heatmap.png) (39284 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [4.74, 3.93] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 6.754, differing 0.054063, SSIM₈ 0.888 (raw 6.754, 0.054063, 0.888)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8212→0.8212 / 10.7938→10.7938; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 15.21; `oak` dx -415.72 dy 15.21; `branch` dx -413.61 dy 15.21

### 02-wrapping-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831417, 8047, 6981, 6472, 5822, 6083, 5579, 5280, 5281, 5759, 5144, 5427, 5366, 5118, 5239, 25801]`; ink px ref/ours 39390/39525 (ratio 1.0034); SSIM blocks <0.9: 4321/30294; [overlay](images/02-wrapping-paragraph/pdflatex-main-export-p1-overlay.png) (54904 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-main-export-p1-heatmap.png) (42498 B, ÷4)
  - registration error (diagnostic): global shift [-3.5, 0.0] pt by ink-projection correlation (centroid estimate [-5.15, 4.11] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.4128, differing 0.061395, SSIM₈ 0.8661 (raw 8.47, 0.06151, 0.8655)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7851→0.7863 / 13.5368→13.4326; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 20.54; `branch` dx -435.47 dy 14.86; `the` dx -430.7 dy 20.5

### 02-wrapping-paragraph — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1843946, 8127, 7137, 6016, 5619, 5432, 4964, 5079, 4801, 4920, 4605, 4857, 4323, 4249, 4612, 20129]`; ink px ref/ours 39390/39245 (ratio 0.9963); SSIM blocks <0.9: 3826/30294; [overlay](images/02-wrapping-paragraph/pdflatex-pipeline-export-p1-overlay.png) (55367 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-pipeline-export-p1-heatmap.png) (39577 B, ÷4)
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [3.71, 4.09] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.0842, differing 0.055002, SSIM₈ 0.8919 (raw 7.1603, 0.055246, 0.891)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8261→0.8284 / 11.4425→11.3087; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 14.85; `oak` dx -415.72 dy 14.85; `branch` dx -413.61 dy 14.85

### 02-wrapping-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834636, 8219, 6952, 6893, 6423, 6140, 6491, 6184, 5976, 5757, 5362, 5028, 4837, 4417, 4790, 20711]`; ink px ref/ours 30080/39464 (ratio 1.312); SSIM blocks <0.9: 4465/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (55416 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (42776 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-8.5, -14.5] pt REJECTED: applying it gives mean|Δ| 7.8128, not lower; centroid estimate [-12.18, 0.82] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7862, differing 0.059683, SSIM₈ 0.862 (raw 7.7862, 0.059683, 0.862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7795→0.7795 / 12.4441→12.4441; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.04; `jumps` dx 416.92 dy -13.77

### 02-wrapping-paragraph — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 30080/30093 (ratio 1.0004); SSIM blocks <0.9: 0/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-exact-export-p1-overlay.png) (41042 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-exact-export-p1-heatmap.png) (39157 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.08, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0256, differing 0.014346, SSIM₈ 1.0 (raw 0.0256, 0.014346, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0409→0.0409; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `quick` dx 0.01 dy 1.03; `old` dx -0.01 dy 1.03; `into` dx -0.01 dy 1.03

### 02-wrapping-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834659, 8126, 7025, 6903, 6328, 6172, 6467, 6066, 6218, 5645, 5310, 5027, 5017, 4505, 4688, 20660]`; ink px ref/ours 30080/39525 (ratio 1.314); SSIM blocks <0.9: 4481/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-overlay.png) (55443 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (42720 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-1.0, -14.5] pt REJECTED: applying it gives mean|Δ| 7.8161, not lower; centroid estimate [-9.82, 0.17] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7924, differing 0.059812, SSIM₈ 0.8614 (raw 7.7924, 0.059812, 0.8614)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7786→0.7786 / 12.4539→12.4539; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -13.72; `oak` dx 425.86 dy -8.04; `jumps` dx 414.46 dy -13.77

### 02-wrapping-paragraph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1865239, 8346, 6891, 6699, 6284, 5795, 5094, 4862, 4272, 3776, 3210, 2895, 2807, 2471, 2378, 7797]`; ink px ref/ours 30080/39245 (ratio 1.3047); SSIM blocks <0.9: 3162/30294; [overlay](images/02-wrapping-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) (54260 B, ÷4), [heatmap](images/02-wrapping-paragraph/pdflatex-lm-pipeline-export-p1-heatmap.png) (85115 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.95, 0.16] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 4.551, differing 0.044697, SSIM₈ 0.9366 (raw 4.551, 0.044697, 0.9366)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8989→0.8989 / 7.2728→7.2728; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `quick` dx 0.01 dy 0.67; `into` dx -0.01 dy 0.67; `the` dx 0.01 dy 0.67

### 02-wrapping-paragraph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831948, 8135, 6967, 6429, 5850, 5891, 5942, 5391, 5434, 5606, 5386, 5299, 5280, 5136, 5055, 25067]`; ink px ref/ours 39556/39464 (ratio 0.9977); SSIM blocks <0.9: 4369/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 8.3798, not lower; centroid estimate [-7.66, 4.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.371, differing 0.06128, SSIM₈ 0.8656 (raw 8.371, 0.06128, 0.8656)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7852→0.7852 / 13.3786→13.3786; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 20.54; `branch` dx -435.52 dy 14.86; `over` dx -406.99 dy 20.5

### 02-wrapping-paragraph — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1844777, 7903, 6989, 6645, 6044, 5722, 5923, 5792, 5576, 4813, 4591, 4528, 4367, 4436, 4043, 16667]`; ink px ref/ours 39556/30093 (ratio 0.7608); SSIM blocks <0.9: 3929/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [4.61, 3.92] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 6.8111, differing 0.054456, SSIM₈ 0.887 (raw 6.8111, 0.054456, 0.887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8195→0.8195 / 10.8851→10.8851; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 15.21; `oak` dx -415.83 dy 15.21; `branch` dx -413.66 dy 15.21

### 02-wrapping-paragraph — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1831130, 8033, 7195, 6541, 5874, 5890, 5749, 5378, 5496, 5555, 5369, 5280, 5402, 5183, 5139, 25602]`; ink px ref/ours 39556/39525 (ratio 0.9992); SSIM blocks <0.9: 4347/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-5.29, 4.11] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.4585, differing 0.061737, SSIM₈ 0.8648 (raw 8.4639, 0.061774, 0.8648)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.784→0.7842 / 13.5271→13.5175; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 20.54; `branch` dx -435.52 dy 14.86; `the` dx -430.89 dy 20.5

### 02-wrapping-paragraph — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1843234, 8159, 7188, 6363, 5398, 5412, 5043, 5312, 4899, 4983, 4544, 4875, 4353, 4399, 4465, 20189]`; ink px ref/ours 39556/39245 (ratio 0.9921); SSIM blocks <0.9: 3863/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [3.57, 4.09] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.1671, differing 0.055327, SSIM₈ 0.8904 (raw 7.1981, 0.055568, 0.8902)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8249→0.8255 / 11.5029→11.4505; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 14.85; `oak` dx -415.83 dy 14.85; `branch` dx -413.66 dy 14.85

### 02-wrapping-paragraph — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834641, 8258, 6994, 6804, 6326, 6209, 6507, 6167, 6023, 5775, 5321, 5033, 4798, 4503, 4670, 20787]`; ink px ref/ours 30075/39464 (ratio 1.3122); SSIM blocks <0.9: 4467/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -14.5] pt by ink-projection correlation (centroid estimate [-12.45, 0.74] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7691, differing 0.059703, SSIM₈ 0.8602 (raw 7.7866, 0.059691, 0.862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7795→0.7767 / 12.4447→12.4168; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.04; `jumps` dx 416.92 dy -13.77

### 02-wrapping-paragraph — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938718, 98, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 30075/30093 (ratio 1.0006); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.19, -0.09] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0821, differing 0.021841, SSIM₈ 0.9999 (raw 0.0821, 0.021841, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.1312→0.1312; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `over` dx 0.02 dy 1.03; `patient` dx 0.02 dy 1.03; `quick` dx 0.02 dy 1.03

### 02-wrapping-paragraph — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1834680, 8112, 7072, 6862, 6267, 6228, 6437, 6084, 6232, 5660, 5283, 5015, 4983, 4608, 4579, 20714]`; ink px ref/ours 30075/39525 (ratio 1.3142); SSIM blocks <0.9: 4482/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-1.0, -14.5] pt REJECTED: applying it gives mean|Δ| 7.8165, not lower; centroid estimate [-10.09, 0.09] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7922, differing 0.059811, SSIM₈ 0.8614 (raw 7.7922, 0.059811, 0.8614)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7786→0.7786 / 12.4536→12.4536; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -13.72; `oak` dx 425.86 dy -8.04; `jumps` dx 414.46 dy -13.77

### 02-wrapping-paragraph — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1865305, 8398, 6898, 6483, 6473, 5658, 5149, 4860, 4211, 3921, 3149, 2922, 2790, 2477, 2292, 7830]`; ink px ref/ours 30075/39245 (ratio 1.3049); SSIM blocks <0.9: 3163/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.22, 0.07] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 4.5446, differing 0.044668, SSIM₈ 0.9367 (raw 4.5446, 0.044668, 0.9367)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.899→0.899 / 7.2626→7.2626; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `over` dx 0.02 dy 0.67; `patient` dx 0.02 dy 0.67; `quick` dx 0.02 dy 0.67

### 03-section-heading — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923800, 859, 844, 658, 666, 649, 719, 577, 609, 557, 596, 606, 682, 626, 750, 5618]`; ink px ref/ours 5807/4705 (ratio 0.8102); SSIM blocks <0.9: 660/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 52.5] pt by ink-projection correlation (centroid estimate [1.69, 20.98] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1861, differing 0.007719, SSIM₈ 0.9829 (raw 1.3394, 0.008456, 0.9793)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9669→0.9769 / 2.1407→1.6679; header-band 1.0→0.9712 / 0.0→1.5688; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.58 dy 26.34; `second` dx 0.44 dy 26.34; `a` dx 0.41 dy 26.34

### 03-section-heading — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926883, 931, 808, 664, 639, 594, 676, 633, 579, 490, 487, 469, 485, 513, 548, 3417]`; ink px ref/ours 5807/4792 (ratio 0.8252); SSIM blocks <0.9: 442/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -1.5] pt by ink-projection correlation (centroid estimate [4.49, -1.38] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9311, differing 0.006678, SSIM₈ 0.9876 (raw 0.9612, 0.006795, 0.9868)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9789→0.9802 / 1.5362→1.4882; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 13.54 dy -0.85; `second` dx 11.94 dy -0.85; `a` dx 10.48 dy -0.85

### 03-section-heading — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923544, 879, 837, 674, 664, 667, 710, 587, 619, 568, 692, 639, 718, 648, 749, 5621]`; ink px ref/ours 5807/4945 (ratio 0.8516); SSIM blocks <0.9: 672/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 52.5] pt by ink-projection correlation (centroid estimate [5.02, 20.12] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2063, differing 0.00779, SSIM₈ 0.9826 (raw 1.3597, 0.008549, 0.9789)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9663→0.9767 / 2.1732→1.6847; header-band 1.0→0.969 / 0.0→1.6747; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.4 dy 26.34; `second` dx 0.26 dy 26.34; `a` dx 0.23 dy 26.34
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926649, 818, 798, 675, 567, 705, 614, 576, 576, 498, 509, 519, 582, 569, 579, 3582]`; ink px ref/ours 5807/4955 (ratio 0.8533); SSIM blocks <0.9: 455/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -1.5] pt by ink-projection correlation (centroid estimate [17.27, 2.68] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9814, differing 0.00679, SSIM₈ 0.9874 (raw 1.0037, 0.006913, 0.9865)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9783→0.9799 / 1.6043→1.5686; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Section` dx 38.0 dy -0.57; `Introduction` dx 29.05 dy 0.59; `Second` dx 29.05 dy -0.57
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924467, 959, 778, 633, 646, 661, 738, 747, 647, 638, 596, 597, 643, 582, 684, 4800]`; ink px ref/ours 4775/4705 (ratio 0.9853); SSIM blocks <0.9: 718/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 53.5] pt by ink-projection correlation (centroid estimate [-2.67, 22.65] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0886, differing 0.007454, SSIM₈ 0.9817 (raw 1.2336, 0.008176, 0.9771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9634→0.9751 / 1.9716→1.5009; header-band 1.0→0.9699 / 0.0→1.6444; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.94 dy 27.15; `second` dx -11.49 dy 27.15; `a` dx -10.06 dy 27.15

### 03-section-heading — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1937808, 1008, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 4775/4792 (ratio 1.0036); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.13, 0.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0234, differing 0.002922, SSIM₈ 1.0 (raw 0.0234, 0.002922, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.0373→0.0373; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `a` dx 0.01 dy -0.04; `second` dx 0.01 dy -0.04; `heading.` dx 0.01 dy -0.04

### 03-section-heading — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924306, 982, 748, 662, 647, 696, 746, 737, 670, 652, 669, 604, 653, 601, 677, 4766]`; ink px ref/ours 4775/4945 (ratio 1.0356); SSIM blocks <0.9: 730/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 53.5] pt by ink-projection correlation (centroid estimate [0.65, 21.79] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1129, differing 0.007544, SSIM₈ 0.9812 (raw 1.2426, 0.008227, 0.9769)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9631→0.9747 / 1.986→1.5244; header-band 1.0→0.9672 / 0.0→1.7503; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -13.12 dy 27.15; `second` dx -11.67 dy 27.15; `a` dx -10.24 dy 27.15
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927925, 833, 707, 687, 631, 702, 590, 615, 533, 516, 428, 381, 440, 456, 435, 2937]`; ink px ref/ours 4775/4955 (ratio 1.0377); SSIM blocks <0.9: 432/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [12.91, 4.36] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.855, differing 0.006246, SSIM₈ 0.9887 (raw 0.855, 0.006246, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9819→0.9819 / 1.3665→1.3665; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Second` dx 29.05 dy -0.7; `Section` dx 29.05 dy -0.7; `Introduction` dx 29.05 dy -0.67
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923855, 915, 733, 647, 632, 615, 615, 519, 606, 614, 675, 690, 668, 592, 707, 5733]`; ink px ref/ours 6093/4705 (ratio 0.7722); SSIM blocks <0.9: 662/30294; [overlay](images/03-section-heading/pdflatex-de1020c-export-p1-overlay.png) (39734 B, ÷2), [heatmap](images/03-section-heading/pdflatex-de1020c-export-p1-heatmap.png) (41593 B, ÷2)
  - registration error (diagnostic): global shift [0.5, 52.5] pt by ink-projection correlation (centroid estimate [1.59, 20.53] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2021, differing 0.007752, SSIM₈ 0.9827 (raw 1.3496, 0.008486, 0.9793)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9669→0.9767 / 2.157→1.6932; header-band 1.0→0.9709 / 0.0→1.5688; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.75 dy 26.08; `second` dx 0.61 dy 26.08; `a` dx 0.58 dy 26.08

### 03-section-heading — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926833, 919, 674, 632, 640, 572, 628, 588, 611, 537, 527, 536, 492, 479, 538, 3610]`; ink px ref/ours 6093/4792 (ratio 0.7865); SSIM blocks <0.9: 445/30294; [overlay](images/03-section-heading/pdflatex-exact-export-p1-overlay.png) (41148 B, ÷2), [heatmap](images/03-section-heading/pdflatex-exact-export-p1-heatmap.png) (89122 B, ÷1)
  - registration error (diagnostic): global shift [0.0, -1.5] pt by ink-projection correlation (centroid estimate [4.39, -1.83] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9536, differing 0.006687, SSIM₈ 0.9874 (raw 0.9881, 0.006822, 0.9865)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9784→0.9799 / 1.5792→1.5241; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 13.7 dy -1.12; `second` dx 12.1 dy -1.12; `a` dx 10.65 dy -1.12

### 03-section-heading — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923577, 931, 710, 654, 644, 650, 633, 512, 630, 633, 759, 710, 708, 614, 708, 5743]`; ink px ref/ours 6093/4945 (ratio 0.8116); SSIM blocks <0.9: 674/30294; [overlay](images/03-section-heading/pdflatex-main-export-p1-overlay.png) (39645 B, ÷2), [heatmap](images/03-section-heading/pdflatex-main-export-p1-heatmap.png) (41586 B, ÷2)
  - registration error (diagnostic): global shift [0.5, 52.5] pt by ink-projection correlation (centroid estimate [4.92, 19.67] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.22, differing 0.007837, SSIM₈ 0.9824 (raw 1.3719, 0.008586, 0.9789)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9663→0.9765 / 2.1927→1.7064; header-band 1.0→0.9684 / 0.0→1.6747; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.57 dy 26.08; `second` dx 0.43 dy 26.08; `a` dx 0.4 dy 26.08
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926588, 834, 656, 616, 569, 703, 554, 513, 625, 535, 564, 606, 599, 537, 546, 3771]`; ink px ref/ours 6093/4955 (ratio 0.8132); SSIM blocks <0.9: 461/30294; [overlay](images/03-section-heading/pdflatex-pipeline-export-p1-overlay.png) (39794 B, ÷2), [heatmap](images/03-section-heading/pdflatex-pipeline-export-p1-heatmap.png) (87837 B, ÷1)
  - registration error (diagnostic): global shift [0.5, -2.0] pt by ink-projection correlation (centroid estimate [17.17, 2.23] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0233, differing 0.006977, SSIM₈ 0.9859 (raw 1.0316, 0.006994, 0.9862)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9779→0.9776 / 1.6488→1.6352; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Section` dx 38.0 dy -0.64; `Introduction` dx 29.05 dy 0.71; `Second` dx 29.05 dy -0.64
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924475, 952, 791, 621, 670, 652, 705, 745, 643, 649, 621, 599, 586, 629, 701, 4777]`; ink px ref/ours 4791/4705 (ratio 0.982); SSIM blocks <0.9: 718/30294; [overlay](images/03-section-heading/pdflatex-lm-de1020c-export-p1-overlay.png) (39914 B, ÷2), [heatmap](images/03-section-heading/pdflatex-lm-de1020c-export-p1-heatmap.png) (41936 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 54.0] pt by ink-projection correlation (centroid estimate [-2.62, 22.29] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0954, differing 0.00748, SSIM₈ 0.9811 (raw 1.2335, 0.008161, 0.9771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9634→0.975 / 1.9715→1.4903; header-band 1.0→0.9643 / 0.0→1.793; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.95 dy 28.22; `second` dx -11.5 dy 28.22; `a` dx -10.07 dy 28.22

### 03-section-heading — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 4791/4792 (ratio 1.0002); SSIM blocks <0.9: 0/30294; [overlay](images/03-section-heading/pdflatex-lm-exact-export-p1-overlay.png) (85810 B, ÷1), [heatmap](images/03-section-heading/pdflatex-lm-exact-export-p1-heatmap.png) (63946 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.18, -0.07] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0021, differing 0.001056, SSIM₈ 1.0 (raw 0.0021, 0.001056, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0034→0.0034; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Introduction` dx 0 dy 1.64; `Second` dx 0 dy 1.64; `Section` dx 0.0 dy 1.64

### 03-section-heading — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924312, 961, 779, 647, 672, 690, 707, 738, 668, 659, 696, 603, 593, 642, 704, 4745]`; ink px ref/ours 4791/4945 (ratio 1.0321); SSIM blocks <0.9: 730/30294; [overlay](images/03-section-heading/pdflatex-lm-main-export-p1-overlay.png) (39841 B, ÷2), [heatmap](images/03-section-heading/pdflatex-lm-main-export-p1-heatmap.png) (41827 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 54.0] pt by ink-projection correlation (centroid estimate [0.7, 21.43] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1208, differing 0.007572, SSIM₈ 0.9807 (raw 1.2428, 0.008211, 0.9769)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9631→0.9747 / 1.9863→1.5155; header-band 1.0→0.962 / 0.0→1.8988; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -13.13 dy 28.22; `second` dx -11.68 dy 28.22; `a` dx -10.25 dy 28.22
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927932, 844, 705, 675, 669, 674, 589, 610, 518, 514, 435, 397, 400, 454, 436, 2964]`; ink px ref/ours 4791/4955 (ratio 1.0342); SSIM blocks <0.9: 431/30294; [overlay](images/03-section-heading/pdflatex-lm-pipeline-export-p1-overlay.png) (39379 B, ÷2), [heatmap](images/03-section-heading/pdflatex-lm-pipeline-export-p1-heatmap.png) (86688 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [12.96, 3.99] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8547, differing 0.00624, SSIM₈ 0.9887 (raw 0.8547, 0.00624, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9819→0.9819 / 1.366→1.366; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Introduction` dx 29.05 dy 0.96; `Second` dx 29.05 dy 0.96; `Section` dx 29.05 dy 0.96
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923747, 880, 847, 676, 683, 663, 736, 610, 596, 553, 616, 586, 654, 628, 733, 5608]`; ink px ref/ours 5744/4705 (ratio 0.8191); SSIM blocks <0.9: 660/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 52.5] pt by ink-projection correlation (centroid estimate [1.75, 20.6] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1868, differing 0.007742, SSIM₈ 0.9829 (raw 1.337, 0.008469, 0.9793)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9669→0.9769 / 2.137→1.669; header-band 1.0→0.9712 / 0.0→1.5688; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.57 dy 26.34; `second` dx 0.43 dy 26.34; `a` dx 0.4 dy 26.34

### 03-section-heading — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926819, 957, 817, 672, 623, 616, 677, 658, 594, 492, 491, 473, 452, 505, 555, 3415]`; ink px ref/ours 5744/4792 (ratio 0.8343); SSIM blocks <0.9: 442/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -1.5] pt by ink-projection correlation (centroid estimate [4.55, -1.76] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9334, differing 0.006708, SSIM₈ 0.9876 (raw 0.9621, 0.006824, 0.9868)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9789→0.9801 / 1.5378→1.4918; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 13.52 dy -0.85; `second` dx 11.93 dy -0.85; `a` dx 10.47 dy -0.85

### 03-section-heading — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923470, 890, 833, 686, 684, 694, 739, 619, 613, 570, 702, 619, 690, 649, 738, 5620]`; ink px ref/ours 5744/4945 (ratio 0.8609); SSIM blocks <0.9: 672/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 52.5] pt by ink-projection correlation (centroid estimate [5.07, 19.74] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2075, differing 0.007813, SSIM₈ 0.9826 (raw 1.3601, 0.00857, 0.9789)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9663→0.9767 / 2.1738→1.6866; header-band 1.0→0.969 / 0.0→1.6747; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx 0.39 dy 26.34; `second` dx 0.25 dy 26.34; `a` dx 0.22 dy 26.34
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926553, 840, 799, 686, 594, 687, 642, 623, 553, 500, 521, 489, 559, 549, 585, 3636]`; ink px ref/ours 5744/4955 (ratio 0.8626); SSIM blocks <0.9: 455/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -1.5] pt by ink-projection correlation (centroid estimate [17.33, 2.3] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9851, differing 0.006834, SSIM₈ 0.9874 (raw 1.0089, 0.006953, 0.9864)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9783→0.9799 / 1.6125→1.5745; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Section` dx 38.0 dy -0.57; `Introduction` dx 29.05 dy 0.59; `Second` dx 29.05 dy -0.57
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924466, 954, 785, 634, 640, 664, 732, 752, 647, 641, 594, 601, 641, 581, 686, 4798]`; ink px ref/ours 4777/4705 (ratio 0.9849); SSIM blocks <0.9: 718/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 53.5] pt by ink-projection correlation (centroid estimate [-2.67, 22.65] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0886, differing 0.007455, SSIM₈ 0.9817 (raw 1.2337, 0.008177, 0.9771)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9634→0.9751 / 1.9717→1.501; header-band 1.0→0.9699 / 0.0→1.6444; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -12.94 dy 28.18; `second` dx -11.49 dy 28.18; `a` dx -10.06 dy 28.18

### 03-section-heading — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1937740, 1076, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 4777/4792 (ratio 1.0031); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.13, 0.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0239, differing 0.002905, SSIM₈ 1.0 (raw 0.0239, 0.002905, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.0381→0.0381; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Introduction` dx 0 dy 1.64; `Second` dx 0 dy 1.61; `Section` dx -0.0 dy 1.61

### 03-section-heading — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924306, 986, 747, 663, 641, 697, 745, 739, 669, 654, 666, 611, 648, 608, 672, 4764]`; ink px ref/ours 4777/4945 (ratio 1.0352); SSIM blocks <0.9: 730/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 53.5] pt by ink-projection correlation (centroid estimate [0.65, 21.79] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1129, differing 0.007544, SSIM₈ 0.9812 (raw 1.2425, 0.008228, 0.9769)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9631→0.9747 / 1.9858→1.5244; header-band 1.0→0.9672 / 0.0→1.7503; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `heading.` dx -13.12 dy 28.18; `second` dx -11.67 dy 28.18; `a` dx -10.24 dy 28.18
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 03-section-heading — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927925, 826, 714, 684, 635, 701, 590, 615, 532, 515, 433, 381, 438, 449, 441, 2937]`; ink px ref/ours 4777/4955 (ratio 1.0373); SSIM blocks <0.9: 432/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [12.91, 4.36] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.855, differing 0.006246, SSIM₈ 0.9887 (raw 0.855, 0.006246, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9819→0.9819 / 1.3665→1.3665; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Introduction` dx 29.05 dy 0.96; `Second` dx 29.05 dy 0.93; `Section` dx 29.05 dy 0.93
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']

### 04-bold-emph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933574, 491, 483, 352, 286, 278, 321, 275, 308, 283, 235, 240, 216, 217, 213, 1044]`; ink px ref/ours 2712/2465 (ratio 0.9089); SSIM blocks <0.9: 227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3809, not lower; centroid estimate [1.29, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3808, differing 0.003147, SSIM₈ 0.9945 (raw 0.3808, 0.003147, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9912→0.9912 / 0.6087→0.6087; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.28 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1932834, 499, 422, 359, 364, 342, 343, 354, 358, 322, 248, 311, 257, 243, 253, 1307]`; ink px ref/ours 2712/2268 (ratio 0.8363); SSIM blocks <0.9: 263/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [13.5, 0.0] pt by ink-projection correlation (centroid estimate [18.58, -0.04] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4468, differing 0.003405, SSIM₈ 0.9929 (raw 0.4519, 0.003466, 0.993)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9892 / 0.7222→0.6924; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 31.86 dy 1.09; `one` dx 30.66 dy 1.09; `on` dx 29.35 dy 1.09

### 04-bold-emph — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933581, 495, 476, 346, 302, 277, 341, 277, 321, 274, 237, 267, 204, 216, 208, 994]`; ink px ref/ours 2712/2460 (ratio 0.9071); SSIM blocks <0.9: 227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.75, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3769, differing 0.003131, SSIM₈ 0.9945 (raw 0.3769, 0.003131, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9913→0.9913 / 0.6024→0.6024; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.28 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933167, 523, 418, 379, 326, 317, 282, 335, 278, 280, 233, 256, 258, 279, 271, 1214]`; ink px ref/ours 2712/2424 (ratio 0.8938); SSIM blocks <0.9: 247/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [11.31, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4246, differing 0.003316, SSIM₈ 0.9938 (raw 0.4246, 0.003316, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.99 / 0.6787→0.6787; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 31.86 dy 0.41; `one` dx 30.66 dy 0.41; `on` dx 29.35 dy 0.41
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

### 04-bold-emph — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933135, 462, 395, 357, 369, 349, 325, 343, 352, 298, 238, 265, 257, 274, 230, 1167]`; ink px ref/ours 2276/2465 (ratio 1.083); SSIM blocks <0.9: 258/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-17.7, 0.03] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4148, differing 0.003328, SSIM₈ 0.9934 (raw 0.4244, 0.00332, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6784→0.663; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -27.99 dy -0.63; `one` dx -26.86 dy -0.63; `on` dx -25.6 dy -0.63
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938168, 177, 119, 352, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 2276/2268 (ratio 0.9965); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.42, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0187, differing 0.001425, SSIM₈ 0.9999 (raw 0.0187, 0.001425, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.0299→0.0299; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `words` dx 0.12 dy 0.0; `on` dx 0.12 dy 0.0; `one` dx 0.12 dy 0.0

### 04-bold-emph — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933099, 458, 398, 356, 373, 363, 333, 348, 353, 303, 239, 305, 251, 265, 241, 1131]`; ink px ref/ours 2276/2460 (ratio 1.0808); SSIM blocks <0.9: 259/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-19.75, 0.03] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4147, differing 0.003329, SSIM₈ 0.9934 (raw 0.4253, 0.003331, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6797→0.6627; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -28.11 dy -0.63; `one` dx -26.98 dy -0.63; `on` dx -25.72 dy -0.63
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933647, 504, 408, 385, 351, 317, 311, 348, 271, 257, 240, 194, 228, 219, 221, 915]`; ink px ref/ours 2276/2424 (ratio 1.065); SSIM blocks <0.9: 222/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.68, 0.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3651, differing 0.003066, SSIM₈ 0.9947 (raw 0.3651, 0.003066, 0.9947)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9916→0.9916 / 0.5835→0.5835; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `words` dx 0.12 dy -0.68; `on` dx 0.12 dy -0.68; `line.` dx 0.12 dy -0.68
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

### 04-bold-emph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933598, 513, 444, 319, 342, 300, 270, 233, 301, 263, 246, 223, 207, 251, 234, 1072]`; ink px ref/ours 2746/2465 (ratio 0.8977); SSIM blocks <0.9: 225/30294; [overlay](images/04-bold-emph/pdflatex-de1020c-export-p1-overlay.png) (76686 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-de1020c-export-p1-heatmap.png) (70802 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [3.52, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3764, differing 0.003158, SSIM₈ 0.9945 (raw 0.3836, 0.003143, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9912 / 0.6131→0.6017; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.27 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1932780, 522, 438, 353, 364, 372, 290, 343, 363, 345, 298, 280, 256, 232, 264, 1316]`; ink px ref/ours 2746/2268 (ratio 0.8259); SSIM blocks <0.9: 264/30294; [overlay](images/04-bold-emph/pdflatex-exact-export-p1-overlay.png) (79733 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-exact-export-p1-heatmap.png) (76704 B, ÷1)
  - registration error (diagnostic): global shift [-29.5, 0.0] pt by ink-projection correlation (centroid estimate [20.81, -0.01] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4518, differing 0.003471, SSIM₈ 0.9929 (raw 0.4549, 0.00348, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9886 / 0.727→0.7221; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 32.43 dy 1.09; `one` dx 31.22 dy 1.09; `on` dx 29.91 dy 1.09

### 04-bold-emph — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933606, 529, 426, 332, 337, 306, 282, 233, 293, 251, 239, 262, 205, 244, 250, 1021]`; ink px ref/ours 2746/2460 (ratio 0.8958); SSIM blocks <0.9: 225/30294; [overlay](images/04-bold-emph/pdflatex-main-export-p1-overlay.png) (76456 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-main-export-p1-heatmap.png) (73192 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.48, 0.04] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3777, differing 0.003162, SSIM₈ 0.9945 (raw 0.3804, 0.003141, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.9912 / 0.608→0.6036; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.27 dy 0.46; `bold` dx 5.19 dy 0.46; `and` dx 5.11 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933279, 527, 372, 337, 321, 299, 230, 337, 302, 274, 256, 249, 259, 261, 265, 1248]`; ink px ref/ours 2746/2424 (ratio 0.8827); SSIM blocks <0.9: 240/30294; [overlay](images/04-bold-emph/pdflatex-pipeline-export-p1-overlay.png) (74821 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-pipeline-export-p1-heatmap.png) (71680 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [13.54, 0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4235, differing 0.003324, SSIM₈ 0.9938 (raw 0.4235, 0.003324, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9901→0.9901 / 0.6769→0.6769; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 32.43 dy 0.41; `one` dx 31.22 dy 0.41; `on` dx 29.91 dy 0.41
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

### 04-bold-emph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933105, 492, 380, 370, 376, 348, 330, 343, 330, 319, 248, 266, 243, 259, 233, 1174]`; ink px ref/ours 2268/2465 (ratio 1.0869); SSIM blocks <0.9: 259/30294; [overlay](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-overlay.png) (77492 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-de1020c-export-p1-heatmap.png) (72110 B, ÷1)
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-17.16, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4162, differing 0.003327, SSIM₈ 0.9934 (raw 0.4246, 0.003322, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6786→0.6653; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -28.1 dy 0.73; `one` dx -26.97 dy 0.73; `on` dx -25.72 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 2268/2268 (ratio 1.0); SSIM blocks <0.9: 0/30294; [overlay](images/04-bold-emph/pdflatex-lm-exact-export-p1-overlay.png) (75945 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-exact-export-p1-heatmap.png) (61784 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.13, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0016, differing 0.000944, SSIM₈ 1.0 (raw 0.0016, 0.000944, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0025→0.0025; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 0.01 dy 1.35; `Plain,` dx 0 dy 1.35; `bold,` dx 0.0 dy 1.35

### 04-bold-emph — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933067, 485, 392, 372, 376, 351, 346, 341, 345, 316, 246, 309, 232, 254, 238, 1146]`; ink px ref/ours 2268/2460 (ratio 1.0847); SSIM blocks <0.9: 259/30294; [overlay](images/04-bold-emph/pdflatex-lm-main-export-p1-overlay.png) (77281 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-main-export-p1-heatmap.png) (71858 B, ÷1)
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-19.2, 0.04] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4164, differing 0.003329, SSIM₈ 0.9934 (raw 0.4256, 0.003332, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6803→0.6655; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -28.22 dy 0.73; `one` dx -27.09 dy 0.73; `on` dx -25.84 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933595, 526, 380, 439, 354, 334, 344, 297, 280, 260, 231, 201, 239, 217, 216, 903]`; ink px ref/ours 2268/2424 (ratio 1.0688); SSIM blocks <0.9: 221/30294; [overlay](images/04-bold-emph/pdflatex-lm-pipeline-export-p1-overlay.png) (77097 B, ÷1), [heatmap](images/04-bold-emph/pdflatex-lm-pipeline-export-p1-heatmap.png) (73517 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.14, 0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3653, differing 0.003069, SSIM₈ 0.9947 (raw 0.3653, 0.003069, 0.9947)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9916→0.9916 / 0.5839→0.5839; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 0.01 dy 0.67; `Plain,` dx 0 dy 0.67; `emphasised,` dx 0.0 dy 0.67
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

### 04-bold-emph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933519, 519, 462, 366, 293, 287, 321, 265, 298, 295, 232, 216, 218, 229, 229, 1067]`; ink px ref/ours 2702/2465 (ratio 0.9123); SSIM blocks <0.9: 227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [0.77, 0.0] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3834, differing 0.003146, SSIM₈ 0.9943 (raw 0.3848, 0.003146, 0.9944)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9911→0.991 / 0.6151→0.6128; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.26 dy 0.46; `bold` dx 5.16 dy 0.46; `and` dx 5.12 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1932839, 504, 404, 389, 354, 350, 345, 352, 349, 325, 247, 301, 260, 249, 254, 1294]`; ink px ref/ours 2702/2268 (ratio 0.8394); SSIM blocks <0.9: 263/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [38.5, 0.0] pt by ink-projection correlation (centroid estimate [18.06, -0.04] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4471, differing 0.003456, SSIM₈ 0.9932 (raw 0.4502, 0.003462, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9903 / 0.7196→0.6629; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 31.95 dy 1.09; `one` dx 30.74 dy 1.09; `on` dx 29.43 dy 1.09

### 04-bold-emph — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933534, 525, 447, 350, 302, 290, 343, 267, 303, 288, 244, 261, 213, 217, 228, 1004]`; ink px ref/ours 2702/2460 (ratio 0.9104); SSIM blocks <0.9: 227/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.27, 0.0] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3801, differing 0.003144, SSIM₈ 0.9944 (raw 0.381, 0.003132, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9912→0.991 / 0.609→0.6075; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `emphasised` dx 5.26 dy 0.46; `bold` dx 5.16 dy 0.46; `and` dx 5.12 dy 0.46
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933177, 537, 407, 381, 298, 312, 308, 330, 295, 272, 228, 251, 264, 282, 261, 1213]`; ink px ref/ours 2702/2424 (ratio 0.8971); SSIM blocks <0.9: 248/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [10.79, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4236, differing 0.003314, SSIM₈ 0.9938 (raw 0.4236, 0.003314, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9901→0.9901 / 0.677→0.677; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx 31.95 dy 0.41; `one` dx 30.74 dy 0.41; `on` dx 29.43 dy 0.41
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

### 04-bold-emph — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933144, 455, 411, 335, 384, 345, 332, 339, 340, 281, 249, 264, 266, 261, 226, 1184]`; ink px ref/ours 2260/2465 (ratio 1.0907); SSIM blocks <0.9: 255/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-16.16, 0.03] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4138, differing 0.003314, SSIM₈ 0.9934 (raw 0.4245, 0.003312, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6784→0.6613; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -27.75 dy 0.73; `one` dx -26.62 dy 0.73; `on` dx -25.36 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1937861, 131, 82, 96, 110, 85, 98, 87, 71, 39, 79, 77, 0, 0, 0, 0]`; ink px ref/ours 2260/2268 (ratio 1.0035); SSIM blocks <0.9: 50/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [1.13, -0.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0503, differing 0.00154, SSIM₈ 0.9995 (raw 0.0503, 0.00154, 0.9995)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9991→0.9991 / 0.0803→0.0803; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `words` dx 0.36 dy 1.35; `line.` dx 0.36 dy 1.35; `on` dx 0.35 dy 1.35

### 04-bold-emph — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933109, 469, 421, 338, 378, 357, 345, 342, 342, 280, 242, 304, 251, 250, 228, 1160]`; ink px ref/ours 2260/2460 (ratio 1.0885); SSIM blocks <0.9: 256/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-49.0, 0.0] pt by ink-projection correlation (centroid estimate [-18.2, 0.02] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4127, differing 0.003305, SSIM₈ 0.9934 (raw 0.4237, 0.003311, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9894 / 0.6772→0.6596; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `line.` dx -27.87 dy 0.73; `one` dx -26.74 dy 0.73; `on` dx -25.48 dy 0.73
- word-sequence differences: replace ref ['bold,', 'emphasised,'] ours ['bold', ',', 'emphasised', ',']

### 04-bold-emph — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933527, 487, 407, 382, 379, 337, 306, 346, 272, 276, 237, 199, 256, 229, 235, 941]`; ink px ref/ours 2260/2424 (ratio 1.0726); SSIM blocks <0.9: 229/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-6.14, 0.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.377, differing 0.003105, SSIM₈ 0.9945 (raw 0.377, 0.003105, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9913→0.9913 / 0.6025→0.6025; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `words` dx 0.36 dy 0.67; `line.` dx 0.36 dy 0.67; `on` dx 0.35 dy 0.67
- word-sequence differences: replace ref ['bold,'] ours ['bold', ',']

### 05-unicode — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934471, 531, 434, 351, 255, 281, 235, 186, 150, 219, 214, 199, 193, 149, 197, 751]`; ink px ref/ours 2129/2123 (ratio 0.9972); SSIM blocks <0.9: 193/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [5.16, -0.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2971, differing 0.002695, SSIM₈ 0.9955 (raw 0.2971, 0.002695, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9928→0.9928 / 0.4749→0.4749; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 30.41 dy 0.46; `also` dx -5.46 dy 0.46; `dash;` dx -4.07 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933803, 444, 374, 323, 284, 317, 307, 307, 259, 261, 258, 256, 264, 240, 258, 861]`; ink px ref/ours 2129/1541 (ratio 0.7238); SSIM blocks <0.9: 224/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [9.5, 0.0] pt by ink-projection correlation (centroid estimate [8.42, 0.12] pt); confidence moderate (shift explains 14% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3164, differing 0.002718, SSIM₈ 0.9946 (raw 0.3668, 0.00292, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9901→0.9917 / 0.5862→0.4935; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 14.25 dy 0.77; `—` dx 13.59 dy 0.77; `Résumé` dx 11.29 dy 0.77

### 05-unicode — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934812, 608, 496, 375, 300, 247, 183, 147, 142, 184, 149, 137, 156, 150, 142, 588]`; ink px ref/ours 2129/2125 (ratio 0.9981); SSIM blocks <0.9: 166/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [8.87, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2489, differing 0.002552, SSIM₈ 0.9963 (raw 0.2489, 0.002552, 0.9963)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9941→0.9941 / 0.3978→0.3978; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 35.45 dy 0.46; `café` dx -0.37 dy 0.46; `Résumé` dx -0.27 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933990, 500, 335, 317, 259, 302, 249, 218, 251, 235, 260, 232, 259, 226, 264, 919]`; ink px ref/ours 2129/2085 (ratio 0.9793); SSIM blocks <0.9: 201/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [11.5, 0.0] pt by ink-projection correlation (centroid estimate [8.5, 0.01] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3282, differing 0.002771, SSIM₈ 0.9951 (raw 0.3578, 0.002898, 0.9948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9916→0.9926 / 0.5718→0.5041; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 14.25 dy 0.41; `—` dx 13.59 dy 0.41; `Résumé` dx 11.29 dy 0.41

### 05-unicode — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933784, 466, 389, 318, 316, 350, 300, 311, 265, 275, 232, 237, 245, 200, 242, 886]`; ink px ref/ours 1547/2123 (ratio 1.3723); SSIM blocks <0.9: 237/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [2.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.87, -0.15] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3494, differing 0.002871, SSIM₈ 0.994 (raw 0.3619, 0.002925, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.9905 / 0.5784→0.5529; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 16.19 dy -0.3; `also` dx -13.35 dy -0.3; `dash;` dx -9.55 dy -0.3
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938761, 55, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 1547/1541 (ratio 0.9961); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.6, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0051, differing 0.001077, SSIM₈ 1.0 (raw 0.0051, 0.001077, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0082→0.0082; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 0.03 dy 0.0; `dash;` dx 0.02 dy 0.0; `also` dx 0.02 dy 0.0

### 05-unicode — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933797, 474, 369, 300, 325, 326, 312, 319, 258, 287, 223, 211, 270, 219, 240, 886]`; ink px ref/ours 1547/2125 (ratio 1.3736); SSIM blocks <0.9: 238/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.5, 0.0] pt by ink-projection correlation (centroid estimate [-0.15, -0.15] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3432, differing 0.00286, SSIM₈ 0.9941 (raw 0.3628, 0.002915, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9901→0.9906 / 0.5798→0.5486; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 21.23 dy -0.3; `also` dx -7.83 dy -0.3; `dash;` dx -5.53 dy -0.3
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934700, 459, 316, 321, 285, 378, 343, 283, 220, 232, 187, 114, 179, 144, 146, 509]`; ink px ref/ours 1547/2085 (ratio 1.3478); SSIM blocks <0.9: 188/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-0.52, -0.11] pt); confidence moderate (shift explains 21% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2087, differing 0.002247, SSIM₈ 0.9969 (raw 0.2655, 0.002451, 0.996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9936→0.9951 / 0.4243→0.3336; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 0.03 dy -0.36; `dash;` dx 0.02 dy -0.36; `also` dx 0.02 dy -0.36

### 05-unicode — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934287, 570, 367, 345, 277, 258, 239, 232, 233, 241, 245, 185, 188, 173, 231, 745]`; ink px ref/ours 2119/2123 (ratio 1.0019); SSIM blocks <0.9: 209/30294; [overlay](images/05-unicode/pdflatex-de1020c-export-p1-overlay.png) (76011 B, ÷1), [heatmap](images/05-unicode/pdflatex-de1020c-export-p1-heatmap.png) (72664 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-5.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.331, not lower; centroid estimate [3.28, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3127, differing 0.002744, SSIM₈ 0.9955 (raw 0.3127, 0.002744, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9928→0.9928 / 0.4998→0.4998; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 30.73 dy 0.46; `also` dx -5.31 dy 0.46; `dash;` dx -3.91 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933758, 524, 330, 297, 279, 333, 277, 346, 288, 253, 248, 227, 214, 233, 261, 948]`; ink px ref/ours 2119/1541 (ratio 0.7272); SSIM blocks <0.9: 220/30294; [overlay](images/05-unicode/pdflatex-exact-export-p1-overlay.png) (74726 B, ÷1), [heatmap](images/05-unicode/pdflatex-exact-export-p1-heatmap.png) (74536 B, ÷1)
  - registration error (diagnostic): global shift [10.0, 0.0] pt by ink-projection correlation (centroid estimate [6.54, 0.13] pt); confidence moderate (shift explains 13% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3228, differing 0.002745, SSIM₈ 0.9948 (raw 0.3708, 0.002933, 0.994)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9904→0.9921 / 0.5926→0.5033; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 14.57 dy 0.77; `—` dx 13.9 dy 0.77; `Résumé` dx 11.61 dy 0.77

### 05-unicode — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934942, 727, 440, 283, 275, 291, 190, 189, 117, 160, 132, 96, 131, 144, 138, 561]`; ink px ref/ours 2119/2125 (ratio 1.0028); SSIM blocks <0.9: 150/30294; [overlay](images/05-unicode/pdflatex-main-export-p1-overlay.png) (75198 B, ÷1), [heatmap](images/05-unicode/pdflatex-main-export-p1-heatmap.png) (71053 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [6.99, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2342, differing 0.002512, SSIM₈ 0.9967 (raw 0.2342, 0.002512, 0.9967)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9947→0.9947 / 0.3743→0.3743; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 35.77 dy 0.46; `also` dx 0.21 dy 0.46; `café` dx -0.19 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934073, 533, 284, 253, 268, 263, 248, 243, 253, 256, 240, 229, 218, 204, 289, 962]`; ink px ref/ours 2119/2085 (ratio 0.984); SSIM blocks <0.9: 194/30294; [overlay](images/05-unicode/pdflatex-pipeline-export-p1-overlay.png) (75722 B, ÷1), [heatmap](images/05-unicode/pdflatex-pipeline-export-p1-heatmap.png) (72807 B, ÷1)
  - registration error (diagnostic): global shift [11.5, 0.0] pt by ink-projection correlation (centroid estimate [6.62, 0.02] pt); confidence moderate (shift explains 15% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3051, differing 0.002682, SSIM₈ 0.9957 (raw 0.3573, 0.002881, 0.995)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9921→0.9935 / 0.5711→0.4672; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 14.57 dy 0.41; `—` dx 13.9 dy 0.41; `Résumé` dx 11.61 dy 0.41

### 05-unicode — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933782, 470, 394, 304, 327, 351, 291, 322, 255, 269, 237, 246, 249, 203, 219, 897]`; ink px ref/ours 1537/2123 (ratio 1.3813); SSIM blocks <0.9: 237/30294; [overlay](images/05-unicode/pdflatex-lm-de1020c-export-p1-overlay.png) (75651 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-de1020c-export-p1-heatmap.png) (73202 B, ÷1)
  - registration error (diagnostic): global shift [2.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.9, -0.14] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3496, differing 0.00287, SSIM₈ 0.994 (raw 0.3617, 0.002921, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.9905 / 0.5782→0.5533; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 16.16 dy 0.73; `also` dx -13.37 dy 0.73; `dash;` dx -9.57 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 1537/1541 (ratio 1.0026); SSIM blocks <0.9: 0/30294; [overlay](images/05-unicode/pdflatex-lm-exact-export-p1-overlay.png) (73286 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-exact-export-p1-heatmap.png) (59617 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.36, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0011, differing 0.000651, SSIM₈ 1.0 (raw 0.0011, 0.000651, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0018→0.0018; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `also` dx 0.01 dy 1.03; `—` dx -0.01 dy 1.03; `naïve` dx 0 dy 1.03

### 05-unicode — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933811, 462, 385, 277, 331, 324, 306, 331, 263, 270, 232, 211, 269, 225, 224, 895]`; ink px ref/ours 1537/2125 (ratio 1.3826); SSIM blocks <0.9: 238/30294; [overlay](images/05-unicode/pdflatex-lm-main-export-p1-overlay.png) (75138 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-main-export-p1-heatmap.png) (73048 B, ÷1)
  - registration error (diagnostic): global shift [-3.5, 0.0] pt by ink-projection correlation (centroid estimate [0.81, -0.15] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.343, differing 0.002853, SSIM₈ 0.9941 (raw 0.3625, 0.00291, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9901→0.9906 / 0.5794→0.5482; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 21.2 dy 0.73; `also` dx -7.85 dy 0.73; `dash;` dx -5.55 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934680, 452, 329, 322, 290, 366, 344, 285, 217, 230, 202, 115, 168, 173, 127, 516]`; ink px ref/ours 1537/2085 (ratio 1.3565); SSIM blocks <0.9: 189/30294; [overlay](images/05-unicode/pdflatex-lm-pipeline-export-p1-overlay.png) (75235 B, ÷1), [heatmap](images/05-unicode/pdflatex-lm-pipeline-export-p1-heatmap.png) (70971 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [0.44, -0.11] pt); confidence moderate (shift explains 22% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2083, differing 0.002248, SSIM₈ 0.9969 (raw 0.2675, 0.002458, 0.9959)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9935→0.9951 / 0.4275→0.3329; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `—` dx -0.01 dy 0.67; `naïve` dx 0 dy 0.67; `café` dx 0.0 dy 0.67

### 05-unicode — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934491, 509, 440, 374, 248, 251, 251, 167, 176, 216, 207, 211, 173, 149, 193, 760]`; ink px ref/ours 2139/2123 (ratio 0.9925); SSIM blocks <0.9: 190/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [3.62, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2963, differing 0.002685, SSIM₈ 0.9955 (raw 0.2963, 0.002685, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9928→0.9928 / 0.4736→0.4736; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 30.37 dy 0.46; `also` dx -5.49 dy 0.46; `dash;` dx -4.09 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933819, 427, 369, 323, 298, 312, 313, 290, 282, 247, 277, 260, 249, 212, 272, 866]`; ink px ref/ours 2139/1541 (ratio 0.7204); SSIM blocks <0.9: 225/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [9.5, 0.0] pt by ink-projection correlation (centroid estimate [6.88, 0.11] pt); confidence moderate (shift explains 14% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3163, differing 0.002713, SSIM₈ 0.9946 (raw 0.366, 0.002902, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9901→0.9917 / 0.585→0.4934; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 14.21 dy 0.77; `—` dx 13.54 dy 0.77; `Résumé` dx 11.25 dy 0.77

### 05-unicode — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934854, 565, 485, 386, 275, 253, 206, 129, 157, 167, 165, 148, 139, 142, 149, 596]`; ink px ref/ours 2139/2125 (ratio 0.9935); SSIM blocks <0.9: 164/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [7.33, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2493, differing 0.002552, SSIM₈ 0.9963 (raw 0.2493, 0.002552, 0.9963)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9941→0.9941 / 0.3984→0.3984; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 35.41 dy 0.46; `café` dx -0.37 dy 0.46; `Résumé` dx -0.29 dy 0.46
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934016, 486, 309, 320, 280, 283, 256, 210, 263, 239, 266, 232, 263, 208, 254, 931]`; ink px ref/ours 2139/2085 (ratio 0.9748); SSIM blocks <0.9: 201/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [11.5, 0.0] pt by ink-projection correlation (centroid estimate [6.96, 0.01] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3312, differing 0.002777, SSIM₈ 0.9951 (raw 0.3576, 0.002881, 0.9948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9916→0.9925 / 0.5715→0.5089; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 14.21 dy 0.41; `—` dx 13.54 dy 0.41; `Résumé` dx 11.25 dy 0.41

### 05-unicode — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933783, 466, 389, 315, 321, 350, 297, 313, 265, 274, 231, 236, 249, 198, 242, 887]`; ink px ref/ours 1542/2123 (ratio 1.3768); SSIM blocks <0.9: 237/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [2.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.87, -0.13] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3493, differing 0.002871, SSIM₈ 0.994 (raw 0.362, 0.002927, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.99→0.9905 / 0.5785→0.5529; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 16.19 dy 0.73; `also` dx -13.35 dy 0.73; `dash;` dx -9.55 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938761, 55, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 1542/1541 (ratio 0.9994); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.6, 0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0051, differing 0.001096, SSIM₈ 1.0 (raw 0.0051, 0.001096, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0081→0.0081; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 0.03 dy 1.03; `dash;` dx 0.02 dy 1.03; `also` dx 0.02 dy 1.03

### 05-unicode — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933805, 465, 372, 296, 327, 328, 308, 322, 256, 288, 223, 209, 274, 217, 239, 887]`; ink px ref/ours 1542/2125 (ratio 1.3781); SSIM blocks <0.9: 238/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.5, 0.0] pt by ink-projection correlation (centroid estimate [-0.15, -0.14] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3432, differing 0.002859, SSIM₈ 0.9941 (raw 0.3628, 0.002916, 0.9938)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9901→0.9906 / 0.5798→0.5485; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 21.23 dy 0.73; `also` dx -7.83 dy 0.73; `dash;` dx -5.53 dy 0.73
- word-sequence differences: replace ref ['Naive', 'cafe', 'Resume', '—'] ours ['Na', '"', 'ive', 'caf', "'", 'e', 'R', "'"]

### 05-unicode — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934702, 455, 316, 320, 286, 377, 345, 283, 219, 231, 188, 113, 181, 143, 148, 509]`; ink px ref/ours 1542/2085 (ratio 1.3521); SSIM blocks <0.9: 188/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-0.52, -0.1] pt); confidence moderate (shift explains 21% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2088, differing 0.002246, SSIM₈ 0.9969 (raw 0.2657, 0.002452, 0.996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9935→0.9951 / 0.4247→0.3337; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `dash.` dx 0.03 dy 0.67; `dash;` dx 0.02 dy 0.67; `also` dx 0.02 dy 0.67

### 06-math-inline — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933844, 435, 352, 317, 197, 257, 297, 287, 248, 200, 246, 306, 299, 245, 218, 1068]`; ink px ref/ours 1729/1833 (ratio 1.0602); SSIM blocks <0.9: 216/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-5.21, 3.5] pt); confidence strong (shift explains 33% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2554, differing 0.002365, SSIM₈ 0.9959 (raw 0.3832, 0.00292, 0.9934)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9895→0.9934 / 0.6124→0.4082; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5653→0.68 / 21.4908→17.5131 [131.7,62.4–343.8,93.6 pt]
- largest word displacements (pt): `inside` dx -8.16 dy 1.32; `a` dx -8.04 dy 1.32; `sentence` dx -8.01 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [141.15278244018555, 710.8343820571899, 4.49800968170166, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934485, 423, 385, 276, 261, 291, 302, 297, 255, 264, 174, 160, 254, 197, 204, 588]`; ink px ref/ours 1729/1357 (ratio 0.7848); SSIM blocks <0.9: 204/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-5.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3071, not lower; centroid estimate [2.15, 0.07] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2978, differing 0.002534, SSIM₈ 0.9945 (raw 0.2978, 0.002534, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9913→0.9913 / 0.476→0.476; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6507→0.6507 / 15.9827→15.9827 [131.7,62.4–343.8,93.6 pt]
- largest word displacements (pt): `text.` dx 17.61 dy -0.0; `of` dx 17.22 dy -0.0; `sentence` dx 14.8 dy -0.0

### 06-math-inline — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933815, 451, 328, 345, 235, 254, 250, 292, 244, 238, 235, 299, 284, 234, 255, 1057]`; ink px ref/ours 1729/1837 (ratio 1.0625); SSIM blocks <0.9: 218/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-6.5, 3.5] pt by ink-projection correlation (centroid estimate [-4.37, 3.53] pt); confidence strong (shift explains 48% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1998, differing 0.002181, SSIM₈ 0.9967 (raw 0.3838, 0.002925, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9947 / 0.6135→0.3194; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5582→0.8266 / 21.5408→9.4933 [131.7,62.4–343.8,93.6 pt]
- largest word displacements (pt): `b` dx -0.95 dy 7.52; `inside` dx -6.73 dy 1.32; `a` dx -6.62 dy 1.32
- word-sequence differences: replace ref ['α', '+', 'β,'] ours ['α+β,']

### 06-math-inline — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [141.153, 710.834, 4.65, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934714, 415, 337, 229, 239, 241, 244, 258, 228, 188, 170, 186, 238, 180, 219, 730]`; ink px ref/ours 1729/1843 (ratio 1.0659); SSIM blocks <0.9: 185/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [5.3, -0.17] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2979, differing 0.002493, SSIM₈ 0.9948 (raw 0.2979, 0.002493, 0.9948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9919→0.9919 / 0.4703→0.4703; header-band 0.9987→0.9987 / 0.0404→0.0404; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6426→0.6426 / 17.2107→17.2107 [131.7,62.4–343.8,93.6 pt]
- largest word displacements (pt): `text.` dx 17.61 dy -2.68; `of` dx 17.22 dy -2.68; `sentence` dx 14.8 dy -2.68
- word-sequence differences: replace ref ['√x'] ours ['√', 'x']

### 06-math-inline — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933869, 425, 339, 269, 252, 319, 324, 366, 281, 210, 256, 307, 229, 233, 196, 941]`; ink px ref/ours 1358/1833 (ratio 1.3498); SSIM blocks <0.9: 226/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-32.5, 3.5] pt by ink-projection correlation (centroid estimate [-8.35, 3.43] pt); confidence moderate (shift explains 15% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3134, differing 0.002591, SSIM₈ 0.9945 (raw 0.3682, 0.002867, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9912 / 0.5885→0.5009; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5865→0.6624 / 18.2797→17.0519 [137.6,62.4–364.2,93.6 pt]
- largest word displacements (pt): `text.` dx -25.34 dy 1.32; `of` dx -25.01 dy 1.32; `sentence` dx -22.78 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [141.15278244018555, 710.8343820571899, 4.49800968170166, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938280, 398, 58, 35, 23, 15, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 1358/1357 (ratio 0.9993); SSIM blocks <0.9: 7/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.99, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0133, differing 0.001242, SSIM₈ 0.9999 (raw 0.0133, 0.001242, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.0213→0.0213; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.9969→0.9969 / 0.872→0.872 [137.6,62.4–364.2,93.6 pt]
- largest word displacements (pt): `of` dx 0.04 dy -0.0; `text.` dx 0.04 dy -0.0; `a` dx 0.03 dy -0.0

### 06-math-inline — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933794, 457, 322, 320, 272, 308, 264, 352, 278, 271, 255, 282, 215, 237, 221, 968]`; ink px ref/ours 1358/1837 (ratio 1.3527); SSIM blocks <0.9: 229/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-7.5, 3.45] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.312, differing 0.002602, SSIM₈ 0.9945 (raw 0.3735, 0.002891, 0.9928)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9886→0.9915 / 0.5969→0.478; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5818→0.6793 / 18.5969→15.5792 [137.6,62.4–364.2,93.6 pt]
- largest word displacements (pt): `text.` dx -23.92 dy 1.32; `of` dx -23.58 dy 1.32; `sentence` dx -21.35 dy 1.32
- word-sequence differences: replace ref ['α', '+', 'β,'] ours ['α+β,']

### 06-math-inline — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [141.153, 710.834, 4.65, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935279, 405, 312, 282, 309, 238, 213, 224, 203, 184, 154, 187, 131, 117, 147, 431]`; ink px ref/ours 1358/1843 (ratio 1.3571); SSIM blocks <0.9: 168/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [2.17, -0.24] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2164, differing 0.002077, SSIM₈ 0.9963 (raw 0.2287, 0.002124, 0.9963)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9943→0.9944 / 0.3597→0.34; header-band 0.9987→0.9986 / 0.0404→0.0404; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7821→0.7746 / 11.2986→11.5097 [137.6,62.4–364.2,93.6 pt]
- largest word displacements (pt): `of` dx 0.04 dy -2.68; `text.` dx 0.04 dy -2.68; `a` dx 0.03 dy -2.68
- word-sequence differences: replace ref ['√x'] ours ['√', 'x']

### 06-math-inline — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933877, 403, 304, 272, 266, 302, 299, 308, 222, 194, 272, 315, 237, 246, 231, 1068]`; ink px ref/ours 1699/1833 (ratio 1.0789); SSIM blocks <0.9: 211/30294; [overlay](images/06-math-inline/pdflatex-de1020c-export-p1-overlay.png) (73580 B, ÷1), [heatmap](images/06-math-inline/pdflatex-de1020c-export-p1-heatmap.png) (73085 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-4.61, 3.51] pt); confidence strong (shift explains 32% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2595, differing 0.002359, SSIM₈ 0.996 (raw 0.3821, 0.002886, 0.9936)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9898→0.9937 / 0.6107→0.4148; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5648→0.6749 / 21.4145→17.7703 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `a` dx -7.78 dy 1.32; `sentence` dx -7.76 dy 1.32; `of` dx -7.59 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']; replace ref ['√xinside'] ours ['√x', 'inside']

### 06-math-inline — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [141.15278244018555, 710.8343820571899, 4.49800968170166, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934483, 397, 325, 264, 279, 311, 299, 315, 220, 246, 230, 186, 229, 206, 214, 612]`; ink px ref/ours 1699/1357 (ratio 0.7987); SSIM blocks <0.9: 199/30294; [overlay](images/06-math-inline/pdflatex-exact-export-p1-overlay.png) (73118 B, ÷1), [heatmap](images/06-math-inline/pdflatex-exact-export-p1-heatmap.png) (69886 B, ÷1)
  - registration error (diagnostic): global shift [4.0, 0.0] pt by ink-projection correlation (centroid estimate [2.75, 0.07] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2819, differing 0.002452, SSIM₈ 0.9949 (raw 0.3045, 0.002525, 0.9946)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9914→0.992 / 0.4867→0.4424; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6445→0.6794 / 16.39→15.2004 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `text.` dx 17.85 dy -0.0; `of` dx 17.47 dy -0.0; `sentence` dx 15.06 dy -0.0
- word-sequence differences: replace ref ['α+'] ours ['α', '+']; replace ref ['√xinside'] ours ['√x', 'inside']

### 06-math-inline — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933871, 414, 298, 313, 307, 308, 238, 295, 227, 240, 241, 299, 238, 236, 260, 1031]`; ink px ref/ours 1699/1837 (ratio 1.0812); SSIM blocks <0.9: 214/30294; [overlay](images/06-math-inline/pdflatex-main-export-p1-overlay.png) (73789 B, ÷1), [heatmap](images/06-math-inline/pdflatex-main-export-p1-heatmap.png) (72894 B, ÷1)
  - registration error (diagnostic): global shift [-6.0, 3.5] pt by ink-projection correlation (centroid estimate [-3.76, 3.53] pt); confidence strong (shift explains 44% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2119, differing 0.002214, SSIM₈ 0.9968 (raw 0.3782, 0.002877, 0.9935)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9896→0.9949 / 0.6045→0.3386; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5621→0.8104 / 21.1356→10.6633 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `b` dx -0.68 dy 7.52; `a` dx -6.36 dy 1.32; `sentence` dx -6.33 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α+β,']; replace ref ['√xinside'] ours ['√x', 'inside']

### 06-math-inline — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [141.153, 710.834, 4.65, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934780, 343, 249, 196, 247, 273, 234, 258, 220, 206, 212, 206, 212, 197, 225, 758]`; ink px ref/ours 1699/1843 (ratio 1.0848); SSIM blocks <0.9: 179/30294; [overlay](images/06-math-inline/pdflatex-pipeline-export-p1-overlay.png) (73212 B, ÷1), [heatmap](images/06-math-inline/pdflatex-pipeline-export-p1-heatmap.png) (70736 B, ÷1)
  - registration error (diagnostic): global shift [12.0, 0.0] pt by ink-projection correlation (centroid estimate [5.91, -0.17] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2957, differing 0.002471, SSIM₈ 0.9952 (raw 0.3055, 0.00248, 0.9949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.992→0.9929 / 0.4823→0.4398; header-band 0.9987→0.9987 / 0.0404→0.0404; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6377→0.7204 / 17.9093→14.2452 [131.5,62.4–343.4,93.6 pt]
- largest word displacements (pt): `text.` dx 17.85 dy -2.68; `of` dx 17.47 dy -2.68; `sentence` dx 15.06 dy -2.68
- word-sequence differences: replace ref ['α+'] ours ['α', '+']; replace ref ['√xinside'] ours ['√', 'x', 'inside']

### 06-math-inline — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933850, 444, 339, 259, 271, 318, 327, 365, 277, 225, 258, 298, 218, 220, 185, 962]`; ink px ref/ours 1340/1833 (ratio 1.3679); SSIM blocks <0.9: 225/30294; [overlay](images/06-math-inline/pdflatex-lm-de1020c-export-p1-overlay.png) (74143 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-de1020c-export-p1-heatmap.png) (73576 B, ÷1)
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-7.35, 3.41] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3234, differing 0.002634, SSIM₈ 0.9943 (raw 0.368, 0.002877, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9912 / 0.5881→0.4963; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5867→0.6637 / 18.2641→16.2875 [137.6,62.4–364.3,93.6 pt]
- largest word displacements (pt): `text.` dx -25.38 dy 1.32; `of` dx -25.05 dy 1.32; `sentence` dx -22.81 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α', '+β,']

### 06-math-inline — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [141.15278244018555, 710.8343820571899, 4.49800968170166, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938570, 111, 61, 40, 16, 11, 6, 1, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 1340/1357 (ratio 1.0127); SSIM blocks <0.9: 6/30294; [overlay](images/06-math-inline/pdflatex-lm-exact-export-p1-overlay.png) (72450 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-exact-export-p1-heatmap.png) (63295 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.01, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0069, differing 0.000803, SSIM₈ 0.9999 (raw 0.0069, 0.000803, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.011→0.011; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.9978→0.9978 / 0.4442→0.4442 [137.6,62.4–364.3,93.6 pt]
- largest word displacements (pt): `Inline` dx 0 dy 1.03; `math:` dx 0.0 dy 1.03; `a` dx -0.0 dy 1.03
- word-sequence differences: replace ref ['α+'] ours ['α', '+']

### 06-math-inline — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933785, 471, 301, 328, 282, 317, 264, 358, 282, 261, 252, 288, 208, 228, 206, 985]`; ink px ref/ours 1340/1837 (ratio 1.3709); SSIM blocks <0.9: 228/30294; [overlay](images/06-math-inline/pdflatex-lm-main-export-p1-overlay.png) (74167 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-main-export-p1-heatmap.png) (73397 B, ÷1)
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-6.5, 3.43] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3115, differing 0.002605, SSIM₈ 0.9945 (raw 0.3729, 0.002899, 0.9928)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9886→0.9915 / 0.596→0.4773; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5824→0.6797 / 18.5569→15.5459 [137.6,62.4–364.3,93.6 pt]
- largest word displacements (pt): `text.` dx -23.96 dy 1.32; `of` dx -23.62 dy 1.32; `sentence` dx -21.38 dy 1.32
- word-sequence differences: replace ref ['α+', 'β,'] ours ['α+β,']

### 06-math-inline — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [141.153, 710.834, 4.65, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935260, 396, 307, 286, 315, 256, 218, 213, 199, 210, 165, 160, 134, 95, 146, 456]`; ink px ref/ours 1340/1843 (ratio 1.3754); SSIM blocks <0.9: 170/30294; [overlay](images/06-math-inline/pdflatex-lm-pipeline-export-p1-overlay.png) (73699 B, ÷1), [heatmap](images/06-math-inline/pdflatex-lm-pipeline-export-p1-heatmap.png) (69747 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [3.16, -0.27] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.215, differing 0.002085, SSIM₈ 0.9964 (raw 0.2299, 0.002123, 0.9963)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9943→0.9944 / 0.3615→0.3378; header-band 0.9987→0.9986 / 0.0404→0.0404; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.781→0.7765 / 11.3671→11.4121 [137.6,62.4–364.3,93.6 pt]
- largest word displacements (pt): `text.` dx 0.01 dy -2.68; `inside` dx 0.0 dy -2.68; `a` dx -0.0 dy -2.68
- word-sequence differences: replace ref ['α+'] ours ['α', '+']; replace ref ['√x'] ours ['√', 'x']

### 06-math-inline — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933853, 435, 339, 322, 197, 256, 302, 277, 256, 193, 249, 311, 284, 255, 208, 1079]`; ink px ref/ours 1739/1833 (ratio 1.0541); SSIM blocks <0.9: 216/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-4.48, 3.52] pt); confidence strong (shift explains 33% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2548, differing 0.002356, SSIM₈ 0.9959 (raw 0.3831, 0.002915, 0.9934)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9895→0.9934 / 0.6123→0.4073; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5653→0.6801 / 21.4919→17.5101 [131.7,62.1–343.9,93.6 pt]
- largest word displacements (pt): `inside` dx -8.16 dy 1.32; `a` dx -8.04 dy 1.32; `sentence` dx -8.02 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [141.15278244018555, 710.8343820571899, 4.49800968170166, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934493, 420, 379, 281, 254, 296, 298, 287, 268, 258, 178, 161, 240, 207, 193, 603]`; ink px ref/ours 1739/1357 (ratio 0.7803); SSIM blocks <0.9: 204/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-5.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.3072, not lower; centroid estimate [2.88, 0.08] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2979, differing 0.002526, SSIM₈ 0.9945 (raw 0.2979, 0.002526, 0.9945)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9913→0.9913 / 0.4762→0.4762; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6507→0.6507 / 15.9815→15.9815 [131.7,62.1–343.9,93.6 pt]
- largest word displacements (pt): `text.` dx 17.59 dy -0.0; `of` dx 17.2 dy -0.0; `sentence` dx 14.79 dy -0.0

### 06-math-inline — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933820, 455, 314, 349, 235, 259, 253, 279, 253, 229, 244, 294, 274, 245, 243, 1070]`; ink px ref/ours 1739/1837 (ratio 1.0564); SSIM blocks <0.9: 218/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-6.5, 3.5] pt by ink-projection correlation (centroid estimate [-3.63, 3.54] pt); confidence strong (shift explains 48% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1996, differing 0.002172, SSIM₈ 0.9967 (raw 0.3839, 0.00292, 0.9933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9893→0.9948 / 0.6136→0.3191; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5581→0.8263 / 21.5494→9.5045 [131.7,62.1–343.9,93.6 pt]
- largest word displacements (pt): `b` dx -0.95 dy 7.52; `inside` dx -6.73 dy 1.32; `a` dx -6.62 dy 1.32
- word-sequence differences: replace ref ['α', '+', 'β,'] ours ['α+β,']

### 06-math-inline — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [141.153, 710.834, 4.65, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934709, 431, 331, 238, 229, 242, 247, 242, 235, 184, 174, 187, 224, 192, 203, 748]`; ink px ref/ours 1739/1843 (ratio 1.0598); SSIM blocks <0.9: 185/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [6.04, -0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2979, differing 0.002487, SSIM₈ 0.9948 (raw 0.2979, 0.002487, 0.9948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9919→0.9919 / 0.4703→0.4703; header-band 0.9987→0.9987 / 0.0404→0.0404; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6428→0.6428 / 17.1917→17.1917 [131.7,62.1–343.9,93.6 pt]
- largest word displacements (pt): `text.` dx 17.59 dy -2.68; `of` dx 17.2 dy -2.68; `sentence` dx 14.79 dy -2.68
- word-sequence differences: replace ref ['√x'] ours ['√', 'x']

### 06-math-inline — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933869, 423, 342, 268, 250, 323, 325, 362, 281, 211, 255, 310, 230, 231, 194, 942]`; ink px ref/ours 1359/1833 (ratio 1.3488); SSIM blocks <0.9: 226/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-8.96, 3.44] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3233, differing 0.002624, SSIM₈ 0.9943 (raw 0.3682, 0.002868, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9912 / 0.5885→0.4961; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5865→0.6641 / 18.2832→16.2833 [137.6,62.1–364.2,93.6 pt]
- largest word displacements (pt): `text.` dx -25.34 dy 1.32; `of` dx -25.01 dy 1.32; `sentence` dx -22.78 dy 1.32
- word-sequence differences: replace ref ['+', 'β,'] ours ['+β,']

### 06-math-inline — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [141.15278244018555, 710.8343820571899, 4.49800968170166, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938366, 312, 58, 35, 23, 15, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 1359/1357 (ratio 0.9985); SSIM blocks <0.9: 7/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.6, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0127, differing 0.001233, SSIM₈ 0.9999 (raw 0.0127, 0.001233, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.0203→0.0203; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.997→0.997 / 0.8331→0.8331 [137.6,62.1–364.2,93.6 pt]
- largest word displacements (pt): `math:` dx 0.01 dy 1.03; `a` dx 0.01 dy 1.03; `Inline` dx 0 dy 1.03

### 06-math-inline — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [133.33, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933798, 451, 326, 322, 265, 314, 263, 349, 281, 269, 256, 281, 217, 236, 219, 969]`; ink px ref/ours 1359/1837 (ratio 1.3517); SSIM blocks <0.9: 229/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [9.5, 3.5] pt by ink-projection correlation (centroid estimate [-8.11, 3.46] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3119, differing 0.002604, SSIM₈ 0.9945 (raw 0.3734, 0.002892, 0.9928)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9886→0.9915 / 0.5969→0.4779; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5818→0.6794 / 18.596→15.5742 [137.6,62.1–364.2,93.6 pt]
- largest word displacements (pt): `text.` dx -23.92 dy 1.32; `of` dx -23.58 dy 1.32; `sentence` dx -21.35 dy 1.32
- word-sequence differences: replace ref ['α', '+', 'β,'] ours ['α+β,']

### 06-math-inline — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [141.153, 710.834, 4.65, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1935279, 402, 315, 282, 309, 238, 214, 224, 202, 187, 150, 188, 134, 115, 145, 432]`; ink px ref/ours 1359/1843 (ratio 1.3561); SSIM blocks <0.9: 168/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.55, -0.24] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2164, differing 0.002083, SSIM₈ 0.9963 (raw 0.2288, 0.00213, 0.9963)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9943→0.9944 / 0.3598→0.3401; header-band 0.9987→0.9986 / 0.0404→0.0404; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.782→0.7746 / 11.3029→11.5115 [137.6,62.1–364.2,93.6 pt]
- largest word displacements (pt): `of` dx 0.04 dy -2.68; `text.` dx 0.04 dy -2.68; `a` dx 0.03 dy -2.68
- word-sequence differences: replace ref ['√x'] ours ['√', 'x']

### 07-math-display — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934500, 594, 464, 552, 281, 307, 196, 220, 289, 154, 134, 152, 181, 132, 125, 535]`; ink px ref/ours 1954/1796 (ratio 0.9191); SSIM blocks <0.9: 191/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-8.59, 1.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2595, differing 0.002628, SSIM₈ 0.995 (raw 0.2595, 0.002628, 0.995)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.992→0.992 / 0.4148→0.4148; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4802→0.4802 / 19.0416→19.0416 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [302.3859872817993, 690.1341943740845, 43.35099124908447, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934519, 565, 526, 422, 308, 340, 269, 257, 239, 159, 127, 149, 142, 135, 133, 526]`; ink px ref/ours 1954/1548 (ratio 0.7922); SSIM blocks <0.9: 197/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [8.57, -0.33] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2571, differing 0.002547, SSIM₈ 0.9957 (raw 0.2571, 0.002547, 0.9957)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9932→0.9932 / 0.411→0.411; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.8154→0.8154 / 8.2045→8.2045 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 43.0 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx 4.92 dy 0.44; `n` dx -0.0 dy -4.3; `display.` dx 3.63 dy 0.77
- word-sequence differences: insert ref [] ours ['∑']; replace ref ['i='] ours ['i', '=']

### 07-math-display — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [308.06, 686.88, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934495, 592, 459, 435, 298, 298, 223, 245, 320, 157, 148, 182, 187, 128, 165, 484]`; ink px ref/ours 1954/1865 (ratio 0.9545); SSIM blocks <0.9: 204/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [13.54, 1.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.264, differing 0.002646, SSIM₈ 0.9946 (raw 0.264, 0.002646, 0.9946)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9914→0.9914 / 0.422→0.422; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5021→0.5021 / 17.8342→17.8342 [261.4,84.5–349.7,122.4 pt]
- largest word displacements (pt): `2` dx -2.26 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'n(n+1)', 'i=1i=']; insert ref [] ours ['(1)']

### 07-math-display — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [302.386, 690.134, 44.175, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934505, 611, 509, 374, 332, 260, 226, 276, 188, 151, 161, 165, 150, 152, 119, 637]`; ink px ref/ours 1954/1989 (ratio 1.0179); SSIM blocks <0.9: 208/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.31, -0.6] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2678, differing 0.002568, SSIM₈ 0.9954 (raw 0.2678, 0.002568, 0.9954)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9927→0.9927 / 0.428→0.428; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7586→0.7586 / 11.57→11.57 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.5 Δy 0.0 len 44.0 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `i=` dx -16.04 dy 11.33; `display.` dx 4.92 dy 0.09; `display.` dx 3.63 dy 0.41
- word-sequence differences: replace ref ['i=1'] ours ['∑']; insert ref [] ours ['1', 'i', '=']

### 07-math-display — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933827, 410, 355, 528, 348, 373, 280, 264, 264, 218, 237, 183, 297, 180, 203, 849]`; ink px ref/ours 1608/1796 (ratio 1.1169); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-23.95, 1.41] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3495, differing 0.002895, SSIM₈ 0.9931 (raw 0.3495, 0.002895, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.989→0.989 / 0.5586→0.5586; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4893→0.4893 / 18.7965→18.7965 [261.4,84.4–349.7,122.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.73 dy 1.09; `2` dx -3.54 dy 2.25; `display.` dx -3.41 dy -0.3
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [302.3859872817993, 690.1341943740845, 43.35099124908447, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1936871, 436, 301, 253, 180, 180, 108, 99, 103, 42, 37, 60, 25, 29, 12, 80]`; ink px ref/ours 1608/1548 (ratio 0.9627); SSIM blocks <0.9: 98/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-6.79, 0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0847, differing 0.001692, SSIM₈ 0.9986 (raw 0.0847, 0.001692, 0.9986)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9978→0.9978 / 0.1354→0.1354; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.8723→0.8723 / 6.3654→6.3654 [261.4,84.4–349.7,122.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 43.0 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `n` dx -0.0 dy -4.16; `display.` dx 0.01 dy -0.42; `After` dx 0 dy -0.42
- word-sequence differences: insert ref [] ours ['∑']; replace ref ['i='] ours ['i', '=']

### 07-math-display — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [308.06, 686.88, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933817, 404, 354, 424, 362, 363, 307, 286, 290, 217, 258, 212, 307, 173, 241, 801]`; ink px ref/ours 1608/1865 (ratio 1.1598); SSIM blocks <0.9: 248/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.82, 1.4] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3541, differing 0.002914, SSIM₈ 0.9928 (raw 0.3541, 0.002914, 0.9928)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9885→0.9885 / 0.566→0.566; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5083→0.5083 / 17.6196→17.6196 [261.4,84.4–349.7,122.2 pt]
- largest word displacements (pt): `display.` dx -4.73 dy 1.09; `display.` dx -3.41 dy -0.3; `2` dx -2.26 dy 2.25
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'n(n+1)', 'i=1i=']; insert ref [] ours ['(1)']

### 07-math-display — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [302.386, 690.134, 44.175, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934636, 548, 404, 379, 349, 333, 316, 248, 251, 188, 185, 160, 129, 121, 97, 472]`; ink px ref/ours 1608/1989 (ratio 1.2369); SSIM blocks <0.9: 198/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-13.05, -0.21] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2265, differing 0.002348, SSIM₈ 0.9961 (raw 0.2506, 0.002463, 0.9957)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9932→0.9938 / 0.4005→0.3619; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.8002→0.7965 / 10.1918→9.9117 [261.4,84.4–349.7,122.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.5 Δy 0.0 len 44.0 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `i=` dx -16.04 dy 11.47; `n` dx 0.0 dy -3.11; `display.` dx 0.01 dy -0.77
- word-sequence differences: replace ref ['i=1'] ours ['∑']; insert ref [] ours ['1', 'i', '=']

### 07-math-display — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934567, 583, 430, 473, 321, 297, 196, 195, 302, 176, 128, 141, 187, 135, 135, 550]`; ink px ref/ours 1978/1796 (ratio 0.908); SSIM blocks <0.9: 193/30294; [overlay](images/07-math-display/pdflatex-de1020c-export-p1-overlay.png) (74937 B, ÷1), [heatmap](images/07-math-display/pdflatex-de1020c-export-p1-heatmap.png) (70907 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.55, 0.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2609, differing 0.00259, SSIM₈ 0.9949 (raw 0.2609, 0.00259, 0.9949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9919→0.9919 / 0.417→0.417; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4801→0.4801 / 19.0485→19.0485 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.1; `display.` dx 0.17 dy 2.13; `the` dx 0.11 dy 2.13
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [302.3859872817993, 690.1341943740845, 43.35099124908447, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934612, 523, 383, 435, 426, 320, 246, 227, 214, 186, 133, 155, 174, 169, 133, 480]`; ink px ref/ours 1978/1548 (ratio 0.7826); SSIM blocks <0.9: 197/30294; [overlay](images/07-math-display/pdflatex-exact-export-p1-overlay.png) (76157 B, ÷1), [heatmap](images/07-math-display/pdflatex-exact-export-p1-heatmap.png) (73946 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [9.61, -0.59] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2563, differing 0.002496, SSIM₈ 0.9958 (raw 0.2563, 0.002496, 0.9958)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9932→0.9932 / 0.4096→0.4096; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.8119→0.8119 / 8.3318→8.3318 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 43.0 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx 4.91 dy 0.63; `n` dx -0.0 dy -4.31; `display.` dx 3.62 dy 0.77
- word-sequence differences: insert ref [] ours ['∑']; replace ref ['i='] ours ['i', '=']

### 07-math-display — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [308.06, 686.88, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934563, 580, 425, 363, 331, 289, 220, 222, 334, 178, 144, 169, 193, 130, 176, 499]`; ink px ref/ours 1978/1865 (ratio 0.9429); SSIM blocks <0.9: 206/30294; [overlay](images/07-math-display/pdflatex-main-export-p1-overlay.png) (75297 B, ÷1), [heatmap](images/07-math-display/pdflatex-main-export-p1-heatmap.png) (71360 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [14.57, 0.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2654, differing 0.002609, SSIM₈ 0.9946 (raw 0.2654, 0.002609, 0.9946)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9914→0.9914 / 0.4242→0.4242; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5021→0.5021 / 17.8407→17.8407 [261.4,84.5–349.7,122.4 pt]
- largest word displacements (pt): `2` dx -2.26 dy 2.1; `display.` dx 0.17 dy 2.13; `the` dx 0.11 dy 2.13
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'n(n+1)', 'i=1i=']; insert ref [] ours ['(1)']

### 07-math-display — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [302.386, 690.134, 44.175, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934904, 591, 313, 366, 332, 213, 180, 194, 196, 157, 167, 153, 166, 154, 125, 605]`; ink px ref/ours 1978/1989 (ratio 1.0056); SSIM blocks <0.9: 181/30294; [overlay](images/07-math-display/pdflatex-pipeline-export-p1-overlay.png) (74389 B, ÷1), [heatmap](images/07-math-display/pdflatex-pipeline-export-p1-heatmap.png) (72299 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [3.35, -0.86] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2521, differing 0.002433, SSIM₈ 0.9957 (raw 0.2521, 0.002433, 0.9957)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9932→0.9932 / 0.403→0.403; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7561→0.7561 / 11.6713→11.6713 [261.4,84.5–349.7,122.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.5 Δy 0.0 len 44.0 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `i=` dx -16.04 dy 11.32; `display.` dx 4.91 dy 0.27; `display.` dx 3.62 dy 0.41
- word-sequence differences: replace ref ['i=1'] ours ['∑']; insert ref [] ours ['1', 'i', '=']

### 07-math-display — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933745, 450, 367, 487, 356, 402, 264, 258, 285, 261, 208, 305, 184, 175, 192, 877]`; ink px ref/ours 1630/1796 (ratio 1.1018); SSIM blocks <0.9: 235/30294; [overlay](images/07-math-display/pdflatex-lm-de1020c-export-p1-overlay.png) (75775 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-de1020c-export-p1-heatmap.png) (74399 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-21.21, 0.91] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3528, differing 0.00293, SSIM₈ 0.9931 (raw 0.3528, 0.00293, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9889→0.9889 / 0.5639→0.5639; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4864→0.4864 / 18.773→18.773 [261.4,84.2–349.7,122.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.75 dy 2.53; `2` dx -3.54 dy 2.36; `display.` dx -3.42 dy 0.73
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [302.3859872817993, 690.1341943740845, 43.35099124908447, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938005, 194, 148, 112, 61, 55, 16, 24, 31, 28, 14, 17, 12, 9, 11, 79]`; ink px ref/ours 1630/1548 (ratio 0.9497); SSIM blocks <0.9: 32/30294; [overlay](images/07-math-display/pdflatex-lm-exact-export-p1-overlay.png) (74400 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-exact-export-p1-heatmap.png) (65384 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.05, -0.45] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0389, differing 0.001097, SSIM₈ 0.9992 (raw 0.0389, 0.001097, 0.9992)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9987→0.9987 / 0.0621→0.0621; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.8852→0.8852 / 5.4724→5.4724 [261.4,84.2–349.7,122.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 43.0 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `n` dx -0.0 dy -4.05; `Before` dx 0 dy 1.03; `the` dx 0.0 dy 1.03
- word-sequence differences: insert ref [] ours ['∑']; replace ref ['i='] ours ['i', '=']

### 07-math-display — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [308.06, 686.88, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933739, 449, 360, 378, 367, 395, 291, 280, 316, 265, 216, 336, 196, 171, 230, 827]`; ink px ref/ours 1630/1865 (ratio 1.1442); SSIM blocks <0.9: 248/30294; [overlay](images/07-math-display/pdflatex-lm-main-export-p1-overlay.png) (76254 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-main-export-p1-heatmap.png) (74918 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.92, 0.9] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3574, differing 0.00295, SSIM₈ 0.9927 (raw 0.3574, 0.00295, 0.9927)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9884→0.9884 / 0.5713→0.5713; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5052→0.5052 / 17.5969→17.5969 [261.4,84.2–349.7,122.1 pt]
- largest word displacements (pt): `display.` dx -4.75 dy 2.53; `display.` dx -3.42 dy 0.73; `the` dx -2.24 dy 2.53
- word-sequence differences: replace ref ['n', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'n(n+1)', 'i=1i=']; insert ref [] ours ['(1)']

### 07-math-display — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [302.386, 690.134, 44.175, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934948, 386, 385, 324, 340, 327, 262, 290, 225, 195, 174, 178, 106, 120, 93, 463]`; ink px ref/ours 1630/1989 (ratio 1.2202); SSIM blocks <0.9: 190/30294; [overlay](images/07-math-display/pdflatex-lm-pipeline-export-p1-overlay.png) (72318 B, ÷1), [heatmap](images/07-math-display/pdflatex-lm-pipeline-export-p1-heatmap.png) (72101 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-10.31, -0.71] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2141, differing 0.002203, SSIM₈ 0.9963 (raw 0.2416, 0.002313, 0.9959)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9934→0.9941 / 0.3862→0.3422; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.8026→0.7981 / 9.6539→9.5137 [261.4,84.2–349.7,122.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.5 Δy 0.0 len 44.0 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `i=` dx -16.04 dy 11.58; `n` dx 0.0 dy -3.0; `display.` dx -0.01 dy 0.67
- word-sequence differences: replace ref ['i=1'] ours ['∑']; insert ref [] ours ['1', 'i', '=']

### 07-math-display — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934499, 617, 467, 539, 276, 309, 204, 216, 283, 145, 142, 139, 179, 138, 120, 543]`; ink px ref/ours 1971/1796 (ratio 0.9112); SSIM blocks <0.9: 191/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-8.7, 0.93] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2589, differing 0.00263, SSIM₈ 0.995 (raw 0.2589, 0.00263, 0.995)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.992→0.992 / 0.4138→0.4138; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5723→0.5723 / 15.5285→15.5285 [261.1,82.4–349.7,128.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `2` dx -3.54 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [302.3859872817993, 690.1341943740845, 43.35099124908447, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934517, 567, 518, 415, 315, 353, 270, 238, 236, 174, 128, 146, 138, 133, 146, 522]`; ink px ref/ours 1971/1548 (ratio 0.7854); SSIM blocks <0.9: 196/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [8.46, -0.42] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2579, differing 0.002555, SSIM₈ 0.9957 (raw 0.2579, 0.002555, 0.9957)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9931→0.9931 / 0.4122→0.4122; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.8465→0.8465 / 6.7055→6.7055 [261.1,82.4–349.7,128.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 43.0 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `∑` dx 4.7 dy -27.81; `display.` dx 4.91 dy 0.44; `display.` dx 3.62 dy 0.77
- word-sequence differences: replace ref ['i='] ours ['i', '=']

### 07-math-display — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [308.06, 686.88, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934494, 615, 462, 422, 293, 300, 231, 241, 314, 148, 156, 169, 185, 134, 160, 492]`; ink px ref/ours 1971/1865 (ratio 0.9462); SSIM blocks <0.9: 204/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [13.42, 0.92] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2634, differing 0.002649, SSIM₈ 0.9946 (raw 0.2634, 0.002649, 0.9946)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9914→0.9914 / 0.421→0.421; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5775→0.5775 / 14.5445→14.5445 [261.1,82.4–349.7,128.4 pt]
- largest word displacements (pt): `2` dx -2.26 dy 2.11; `display.` dx 0.17 dy 1.95; `the` dx 0.11 dy 1.95
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'n(n+1)', 'i=1i=']; insert ref [] ours ['(1)']

### 07-math-display — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [302.386, 690.134, 44.175, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934507, 604, 524, 375, 322, 271, 224, 266, 191, 150, 166, 158, 150, 144, 133, 631]`; ink px ref/ours 1971/1989 (ratio 1.0091); SSIM blocks <0.9: 208/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.2, -0.69] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2675, differing 0.002574, SSIM₈ 0.9955 (raw 0.2675, 0.002574, 0.9955)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9927→0.9927 / 0.4276→0.4276; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7961→0.7961 / 9.426→9.426 [261.1,82.4–349.7,128.4 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.5 Δy 0.0 len 44.0 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `∑` dx 4.31 dy -26.77; `i=` dx -16.04 dy 11.33; `display.` dx 4.91 dy 0.09
- word-sequence differences: delete ref ['i=1'] ours []; insert ref [] ours ['1', 'i', '=']

### 07-math-display — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [304.99, 686.88, 29.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933826, 409, 357, 531, 338, 382, 277, 256, 275, 218, 233, 187, 294, 181, 204, 848]`; ink px ref/ours 1616/1796 (ratio 1.1114); SSIM blocks <0.9: 235/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-24.38, 1.24] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3495, differing 0.002894, SSIM₈ 0.9931 (raw 0.3495, 0.002894, 0.9931)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.989→0.989 / 0.5587→0.5587; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.583→0.583 / 15.5332→15.5332 [261.1,82.3–349.7,128.3 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -4.25 Δy 3.0 len 29.5 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `display.` dx -4.73 dy 2.12; `2` dx -3.54 dy 2.25; `display.` dx -3.41 dy 0.73
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'i=1i=', 'n(n+1)']

### 07-math-display — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [302.3859872817993, 690.1341943740845, 43.35099124908447, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1936871, 435, 294, 260, 182, 179, 108, 97, 106, 42, 36, 60, 25, 29, 12, 80]`; ink px ref/ours 1616/1548 (ratio 0.9579); SSIM blocks <0.9: 98/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.22, -0.11] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0848, differing 0.001686, SSIM₈ 0.9986 (raw 0.0848, 0.001686, 0.9986)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9978→0.9978 / 0.1355→0.1355; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.9105→0.9105 / 5.2787→5.2787 [261.1,82.3–349.7,128.3 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 43.0 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `∑` dx 4.7 dy -27.67; `n` dx -0.0 dy 3.26; `the` dx 0.01 dy 1.03
- word-sequence differences: replace ref ['i='] ours ['i', '=']

### 07-math-display — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [308.06, 686.88, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933816, 403, 356, 427, 352, 372, 304, 278, 301, 217, 254, 216, 304, 174, 242, 800]`; ink px ref/ours 1616/1865 (ratio 1.1541); SSIM blocks <0.9: 248/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-2.26, 1.24] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3541, differing 0.002914, SSIM₈ 0.9928 (raw 0.3541, 0.002914, 0.9928)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9885→0.9885 / 0.566→0.566; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5943→0.5943 / 14.561→14.561 [261.1,82.3–349.7,128.3 pt]
- largest word displacements (pt): `display.` dx -4.73 dy 2.12; `display.` dx -3.41 dy 0.73; `2` dx -2.26 dy 2.25
- word-sequence differences: replace ref ['n', '∑', 'i=1', 'i=', 'n(n', '+', '1)'] ours ['∑n', 'n(n+1)', 'i=1i=']; insert ref [] ours ['(1)']

### 07-math-display — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 1 U+2500 items in compile_result, 1 `re f` rectangles in the PDF (first: [302.386, 690.134, 44.175, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934636, 554, 396, 389, 326, 349, 312, 253, 248, 188, 185, 160, 130, 121, 97, 472]`; ink px ref/ours 1616/1989 (ratio 1.2308); SSIM blocks <0.9: 198/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-13.48, -0.38] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2268, differing 0.002348, SSIM₈ 0.9961 (raw 0.2506, 0.002463, 0.9957)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9932→0.9938 / 0.4006→0.3625; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.8468→0.8444 / 8.4129→8.2082 [261.1,82.3–349.7,128.3 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.5 Δy 0.0 len 44.0 vs 43.0, thickness px 1 vs 1
- largest word displacements (pt): `∑` dx 4.31 dy -26.63; `i=` dx -16.04 dy 11.47; `n` dx 0.0 dy 4.31
- word-sequence differences: delete ref ['i=1'] ours []; insert ref [] ours ['1', 'i', '=']

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

### 08-two-page — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1590288, 30354, 26518, 24126, 22843, 20824, 21659, 20925, 20664, 18454, 16954, 16211, 16153, 15866, 15704, 61273]`; ink px ref/ours 152381/105096 (ratio 0.6897); SSIM blocks <0.9: 14470/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.81, -0.73] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.1489, differing 0.201659, SSIM₈ 0.5866 (raw 25.1489, 0.201659, 0.5866)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3408→0.3408 / 40.1703→40.1703; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9948→0.9948 / 0.1184→0.1184
- page 2: |Δ| histogram (16 bins, pixel counts) `[1593736, 30264, 26769, 24062, 22925, 20635, 21460, 20993, 20267, 18179, 16764, 16059, 15926, 15359, 14935, 60483]`; ink px ref/ours 151462/104953 (ratio 0.6929); SSIM blocks <0.9: 14464/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 24.8403, not lower; centroid estimate [0.13, -0.17] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.7812, differing 0.199974, SSIM₈ 0.5916 (raw 24.7812, 0.199974, 0.5916)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3487→0.3487 / 39.5892→39.5892; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.997→0.997 / 0.0614→0.0614
- page 3: |Δ| histogram (16 bins, pixel counts) `[1817615, 10093, 9023, 8058, 7939, 7309, 7773, 7734, 7469, 6457, 6047, 5502, 5829, 5405, 5574, 20989]`; ink px ref/ours 33623/46427 (ratio 1.3808); SSIM blocks <0.9: 5881/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 29.0] pt by ink-projection correlation (centroid estimate [-0.55, 71.9] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.7284, differing 0.070129, SSIM₈ 0.8253 (raw 8.7731, 0.070177, 0.8244)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7197→0.7477 / 14.0194→12.8616; header-band 1.0→0.8156 / 0.0→7.4803; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 145.22; `branch` dx -413.61 dy 145.22; `oak` dx -415.78 dy 130.78

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1582337, 30713, 26245, 23275, 21832, 19918, 19505, 19322, 18915, 18230, 17487, 16985, 16381, 16218, 16797, 74656]`; ink px ref/ours 152381/137086 (ratio 0.8996); SSIM blocks <0.9: 14334/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 26.7936, not lower; centroid estimate [-1.56, -1.23] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.7588, differing 0.206058, SSIM₈ 0.5916 (raw 26.7588, 0.206058, 0.5916)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3491→0.3491 / 42.7375→42.7375; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9954→0.9954 / 0.1454→0.1454
- page 2: |Δ| histogram (16 bins, pixel counts) `[1583860, 30433, 26190, 23262, 21900, 19802, 19696, 19000, 18890, 18127, 17471, 16967, 16466, 16225, 16176, 74351]`; ink px ref/ours 151462/137137 (ratio 0.9054); SSIM blocks <0.9: 14339/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 26.8353, not lower; centroid estimate [-0.41, -0.4] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.6246, differing 0.205183, SSIM₈ 0.5934 (raw 26.6246, 0.205183, 0.5934)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3518→0.3518 / 42.5316→42.5316; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9974→0.9974 / 0.0759→0.0759
- page 3: |Δ| histogram (16 bins, pixel counts) `[1809829, 10032, 8755, 7645, 7424, 6762, 6824, 6958, 6604, 6506, 6185, 6538, 6109, 5957, 6259, 30429]`; ink px ref/ours 33623/60651 (ratio 1.8039); SSIM blocks <0.9: 5829/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.63, 72.66] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 9.92, differing 0.073681, SSIM₈ 0.8281 (raw 10.0784, 0.074192, 0.8251)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7209→0.7271 / 16.1046→15.8316; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 144.86; `branch` dx -413.61 dy 144.86; `oak` dx -415.78 dy 130.42

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

### 08-two-page — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 104967/105096 (ratio 1.0012); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.12, 0.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2525, differing 0.074948, SSIM₈ 0.9999 (raw 0.2525, 0.074948, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9998→0.9998 / 0.4034→0.4034; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0014→0.0014
- page 2: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 104819/104953 (ratio 1.0013); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.07, 0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2527, differing 0.074889, SSIM₈ 0.9999 (raw 0.2527, 0.074889, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9998→0.9998 / 0.4039→0.4039; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0007→0.0007
- page 3: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 46386/46427 (ratio 1.0009); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.21, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1118, differing 0.032786, SSIM₈ 0.9999 (raw 0.1118, 0.032786, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.1787→0.1787; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `old` dx 0.03 dy -0.0; `old` dx 0.03 dy -0.0; `old` dx 0.03 dy -0.0

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1679427, 30109, 24663, 22687, 22717, 19885, 18563, 17399, 14725, 13531, 10970, 10329, 9758, 9023, 7799, 27231]`; ink px ref/ours 104967/137086 (ratio 1.306); SSIM blocks <0.9: 11034/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.63, -0.46] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9631, differing 0.157284, SSIM₈ 0.7777 (raw 15.9631, 0.157284, 0.7777)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6456→0.6456 / 25.4983→25.4983; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9982→0.9982 / 0.0799→0.0799
- page 2: |Δ| histogram (16 bins, pixel counts) `[1679157, 30408, 24397, 22668, 22685, 19926, 18748, 17229, 14723, 13796, 10800, 10275, 9953, 8909, 7844, 27298]`; ink px ref/ours 104819/137137 (ratio 1.3083); SSIM blocks <0.9: 11071/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.62, -0.22] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9816, differing 0.157322, SSIM₈ 0.7767 (raw 15.9816, 0.157322, 0.7767)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6438→0.6438 / 25.535→25.535; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9992→0.9992 / 0.0303→0.0303
- page 3: |Δ| histogram (16 bins, pixel counts) `[1824340, 13341, 10866, 10041, 10138, 8870, 7985, 7754, 6473, 5897, 4832, 4491, 4362, 3879, 3503, 12044]`; ink px ref/ours 46386/60651 (ratio 1.3075); SSIM blocks <0.9: 4896/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.88, 0.74] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.0389, differing 0.069428, SSIM₈ 0.9017 (raw 7.0389, 0.069428, 0.9017)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8431→0.8431 / 11.2484→11.2484; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `old` dx 0.03 dy -0.36; `old` dx 0.03 dy -0.36; `old` dx 0.03 dy -0.36

### 08-two-page — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p1-overlay.png) (56448 B, ÷8), [heatmap](images/08-two-page/pdflatex-de1020c-export-p1-heatmap.png) (38339 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -14.5] pt by ink-projection correlation (centroid estimate [-6.07, -5.06] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.4928, differing 0.229367, SSIM₈ 0.4891 (raw 31.5764, 0.22951, 0.4881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1821→0.1892 / 50.4615→50.1371; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.96 / 0.0468→1.3594
- page 2: |Δ| histogram (16 bins, pixel counts) `[1518465, 30739, 25693, 24091, 22137, 22533, 22535, 20944, 20824, 22593, 20919, 20497, 20504, 19480, 20990, 105872]`; ink px ref/ours 150460/131743 (ratio 0.8756); SSIM blocks <0.9: 16800/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p2-overlay.png) (54879 B, ÷8), [heatmap](images/08-two-page/pdflatex-de1020c-export-p2-heatmap.png) (38715 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 6.0] pt by ink-projection correlation (centroid estimate [-5.52, 7.86] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.0952, differing 0.225993, SSIM₈ 0.4952 (raw 33.6289, 0.239479, 0.4516)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1237→0.1935 / 53.7443→49.6907; header-band 1.0→0.999 / 0.0→0.0276; footer-band 0.9986→0.9986 / 0.0304→0.0304
- page 3: |Δ| histogram (16 bins, pixel counts) `[1774335, 11537, 9689, 9104, 8316, 8407, 8934, 8213, 7615, 9012, 8045, 7866, 7928, 7590, 8264, 43961]`; ink px ref/ours 33504/71008 (ratio 2.1194); SSIM blocks <0.9: 7408/30294; [overlay](images/08-two-page/pdflatex-de1020c-export-p3-overlay.png) (66512 B, ÷4), [heatmap](images/08-two-page/pdflatex-de1020c-export-p3-heatmap.png) (59488 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 34.5] pt by ink-projection correlation (centroid estimate [-6.67, 111.28] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.0633, differing 0.081669, SSIM₈ 0.8097 (raw 13.387, 0.093579, 0.7605)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6172→0.7105 / 21.3963→16.9124; header-band 1.0→0.8991 / 0.0→5.299; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 214.06; `branch` dx -435.47 dy 167.32; `over` dx -407.04 dy 214.02

### 08-two-page — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1591651, 30518, 26558, 23731, 22710, 21180, 21265, 21374, 20408, 18257, 17018, 15784, 15591, 15111, 15517, 62143]`; ink px ref/ours 151753/105096 (ratio 0.6925); SSIM blocks <0.9: 14421/30294; [overlay](images/08-two-page/pdflatex-exact-export-p1-overlay.png) (55809 B, ÷8), [heatmap](images/08-two-page/pdflatex-exact-export-p1-heatmap.png) (35152 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.56, -1.74] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.0309, differing 0.200674, SSIM₈ 0.5888 (raw 25.0309, 0.200674, 0.5888)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3441→0.3441 / 39.9869→39.9869; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9945→0.9945 / 0.1214→0.1214
- page 2: |Δ| histogram (16 bins, pixel counts) `[1594094, 30584, 26325, 23580, 22906, 21765, 20898, 21265, 20109, 18001, 16553, 15561, 15493, 14758, 15454, 61470]`; ink px ref/ours 150460/104953 (ratio 0.6975); SSIM blocks <0.9: 14375/30294; [overlay](images/08-two-page/pdflatex-exact-export-p2-overlay.png) (55874 B, ÷8), [heatmap](images/08-two-page/pdflatex-exact-export-p2-heatmap.png) (35192 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.44, -1.19] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.7861, differing 0.199421, SSIM₈ 0.5923 (raw 24.7861, 0.199421, 0.5923)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3492→0.3492 / 39.6041→39.6041; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9967→0.9967 / 0.0634→0.0634
- page 3: |Δ| histogram (16 bins, pixel counts) `[1817782, 10145, 9022, 7869, 7960, 7397, 7544, 8061, 7451, 6566, 6005, 5599, 5668, 5158, 5575, 21014]`; ink px ref/ours 33504/46427 (ratio 1.3857); SSIM blocks <0.9: 5849/30294; [overlay](images/08-two-page/pdflatex-exact-export-p3-overlay.png) (63067 B, ÷4), [heatmap](images/08-two-page/pdflatex-exact-export-p3-heatmap.png) (49082 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.0, 29.0] pt REJECTED: applying it gives mean|Δ| 8.7633, not lower; centroid estimate [-1.32, 70.86] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.7513, differing 0.070019, SSIM₈ 0.8254 (raw 8.7513, 0.070019, 0.8254)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7212→0.7212 / 13.9861→13.9861; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 145.22; `branch` dx -413.61 dy 145.22; `oak` dx -415.72 dy 130.78

### 08-two-page — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1520838, 31563, 27074, 24807, 22509, 23307, 22498, 21004, 21495, 22304, 20775, 19954, 20253, 19497, 19992, 100946]`; ink px ref/ours 151753/145750 (ratio 0.9604); SSIM blocks <0.9: 16544/30294; [overlay](images/08-two-page/pdflatex-main-export-p1-overlay.png) (57448 B, ÷8), [heatmap](images/08-two-page/pdflatex-main-export-p1-heatmap.png) (38593 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-11.0, 0.0] pt REJECTED: applying it gives mean|Δ| 32.933, not lower; centroid estimate [-4.21, -3.98] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 32.9189, differing 0.238789, SSIM₈ 0.4737 (raw 32.9189, 0.238789, 0.4737)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1593→0.1593 / 52.6066→52.6066; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.9984 / 0.0468→0.0468
- page 2: |Δ| histogram (16 bins, pixel counts) `[1513922, 31893, 26939, 25121, 23067, 23160, 22863, 21203, 21382, 22317, 20570, 20477, 21252, 19697, 20719, 104234]`; ink px ref/ours 150460/145127 (ratio 0.9646); SSIM blocks <0.9: 16824/30294; [overlay](images/08-two-page/pdflatex-main-export-p2-overlay.png) (57202 B, ÷8), [heatmap](images/08-two-page/pdflatex-main-export-p2-heatmap.png) (38865 B, ÷8)
  - registration error (diagnostic): global shift [-11.0, 2.0] pt by ink-projection correlation (centroid estimate [-3.62, -2.08] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 33.2713, differing 0.240489, SSIM₈ 0.4674 (raw 33.6428, 0.242458, 0.4594)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1364→0.152 / 53.7661→53.0399; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9986→0.9986 / 0.0304→0.0304
- page 3: |Δ| histogram (16 bins, pixel counts) `[1820902, 8714, 7360, 6872, 6087, 6378, 6484, 6020, 5875, 6066, 5575, 5674, 5771, 5456, 5700, 29882]`; ink px ref/ours 33504/47458 (ratio 1.4165); SSIM blocks <0.9: 4939/30294; [overlay](images/08-two-page/pdflatex-main-export-p3-overlay.png) (55326 B, ÷4), [heatmap](images/08-two-page/pdflatex-main-export-p3-heatmap.png) (45387 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 49.0] pt by ink-projection correlation (centroid estimate [-3.76, 35.26] pt); confidence moderate (shift explains 13% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.1787, differing 0.061169, SSIM₈ 0.8693 (raw 9.4108, 0.067455, 0.8433)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7496→0.8346 / 15.0411→10.8589; header-band 1.0→0.7017 / 0.0→15.23; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 78.46; `branch` dx -435.47 dy 66.06; `branch` dx -435.47 dy 60.52

### 08-two-page — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1582932, 30741, 25916, 23000, 21677, 20174, 19443, 19715, 18492, 18092, 17570, 17152, 15948, 15841, 16452, 75671]`; ink px ref/ours 151753/137086 (ratio 0.9033); SSIM blocks <0.9: 14233/30294; [overlay](images/08-two-page/pdflatex-pipeline-export-p1-overlay.png) (57762 B, ÷8), [heatmap](images/08-two-page/pdflatex-pipeline-export-p1-heatmap.png) (36409 B, ÷8)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.31, -2.24] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.6378, differing 0.2051, SSIM₈ 0.5954 (raw 26.7553, 0.205472, 0.5933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3515→0.3576 / 42.7368→42.4844; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9951→0.9953 / 0.1484→0.1484
- page 2: |Δ| histogram (16 bins, pixel counts) `[1583717, 30098, 25723, 22749, 21674, 20382, 19428, 19452, 18740, 18024, 17479, 17008, 16345, 15787, 16470, 75740]`; ink px ref/ours 150460/137137 (ratio 0.9115); SSIM blocks <0.9: 14261/30294; [overlay](images/08-two-page/pdflatex-pipeline-export-p2-overlay.png) (57759 B, ÷8), [heatmap](images/08-two-page/pdflatex-pipeline-export-p2-heatmap.png) (36470 B, ÷8)
  - registration error (diagnostic): global shift [1.5, 0.0] pt by ink-projection correlation (centroid estimate [-0.98, -1.41] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.5085, differing 0.204248, SSIM₈ 0.5971 (raw 26.7677, 0.204916, 0.593)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3508→0.3611 / 42.7669→42.2568; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9971→0.9971 / 0.0778→0.0778
- page 3: |Δ| histogram (16 bins, pixel counts) `[1810616, 10076, 8716, 7487, 7476, 6760, 6752, 7055, 6375, 6723, 6142, 6465, 6007, 5946, 6195, 30025]`; ink px ref/ours 33504/60651 (ratio 1.8103); SSIM blocks <0.9: 5802/30294; [overlay](images/08-two-page/pdflatex-pipeline-export-p3-overlay.png) (63090 B, ÷4), [heatmap](images/08-two-page/pdflatex-pipeline-export-p3-heatmap.png) (48880 B, ÷4)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.4, 71.62] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 9.9266, differing 0.073595, SSIM₈ 0.8282 (raw 9.9934, 0.073678, 0.8269)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7237→0.7269 / 15.9702→15.8359; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 144.86; `branch` dx -413.61 dy 144.86; `oak` dx -415.72 dy 130.42

### 08-two-page — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p1-overlay.png) (55001 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p1-heatmap.png) (37594 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.44, -3.32] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4403, differing 0.210395, SSIM₈ 0.5038 (raw 27.5467, 0.210674, 0.5026)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.208 / 44.0147→43.8447; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1556053, 29239, 25104, 24114, 22751, 21524, 23492, 22483, 22053, 21728, 19173, 18342, 18558, 16712, 18303, 79187]`; ink px ref/ours 105011/131743 (ratio 1.2546); SSIM blocks <0.9: 16449/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p2-overlay.png) (54009 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p2-heatmap.png) (38335 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 36.5] pt by ink-projection correlation (centroid estimate [-5.19, 9.09] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.3288, differing 0.209484, SSIM₈ 0.5027 (raw 29.0579, 0.219561, 0.4664)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.148→0.2198 / 46.4361→42.915; header-band 1.0→0.9042 / 0.0→5.2131; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1746783, 14069, 12340, 11757, 11133, 10758, 11588, 11172, 10690, 10911, 9823, 9052, 9221, 8387, 9339, 41793]`; ink px ref/ours 46441/71008 (ratio 1.529); SSIM blocks <0.9: 8608/30294; [overlay](images/08-two-page/pdflatex-lm-de1020c-export-p3-overlay.png) (80953 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-de1020c-export-p3-heatmap.png) (66039 B, ÷4)
  - registration error (diagnostic): global shift [-1.0, 52.0] pt by ink-projection correlation (centroid estimate [-5.44, 40.42] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.4866, differing 0.102874, SSIM₈ 0.754 (raw 14.7842, 0.109995, 0.721)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5542→0.6347 / 23.6284→20.0792; header-band 1.0→0.8091 / 0.0→10.1537; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 55.47; `oak` dx 426.94 dy 37.61; `oak` dx 426.94 dy 32.12

### 08-two-page — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 105111/105096 (ratio 0.9999); SSIM blocks <0.9: 0/30294; [overlay](images/08-two-page/pdflatex-lm-exact-export-p1-overlay.png) (36063 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-exact-export-p1-heatmap.png) (63632 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.07, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0879, differing 0.049656, SSIM₈ 1.0 (raw 0.0879, 0.049656, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.1404→0.1404; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0005→0.0005
- page 2: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 105011/104953 (ratio 0.9994); SSIM blocks <0.9: 0/30294; [overlay](images/08-two-page/pdflatex-lm-exact-export-p2-overlay.png) (36413 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-exact-export-p2-heatmap.png) (63772 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.1, 0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0878, differing 0.049584, SSIM₈ 1.0 (raw 0.0878, 0.049584, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.1403→0.1403; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0002→0.0002
- page 3: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 46441/46427 (ratio 0.9997); SSIM blocks <0.9: 0/30294; [overlay](images/08-two-page/pdflatex-lm-exact-export-p3-overlay.png) (53475 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-exact-export-p3-heatmap.png) (43324 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.08, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0392, differing 0.022084, SSIM₈ 1.0 (raw 0.0392, 0.022084, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0626→0.0626; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `quick` dx 0.01 dy 1.03; `old` dx -0.01 dy 1.03; `into` dx -0.01 dy 1.03

### 08-two-page — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1555521, 30291, 26058, 24656, 22827, 22307, 23426, 22015, 22563, 21069, 18990, 18590, 18306, 17279, 17582, 77336]`; ink px ref/ours 105111/145750 (ratio 1.3866); SSIM blocks <0.9: 16299/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p1-overlay.png) (56069 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-main-export-p1-heatmap.png) (38167 B, ÷8)
  - registration error (diagnostic): global shift [-0.5, -27.0] pt by ink-projection correlation (centroid estimate [-2.58, -2.25] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 28.7887, differing 0.220226, SSIM₈ 0.482 (raw 28.8184, 0.220067, 0.4825)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.174→0.198 / 46.0467→44.613; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.8271 / 0.0747→9.5919
- page 2: |Δ| histogram (16 bins, pixel counts) `[1549298, 30146, 26517, 24812, 23404, 21960, 24086, 22807, 22411, 21396, 18925, 18468, 19176, 17068, 18355, 79987]`; ink px ref/ours 105011/145127 (ratio 1.382); SSIM blocks <0.9: 16512/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p2-overlay.png) (55998 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-main-export-p2-heatmap.png) (38522 B, ÷8)
  - registration error (diagnostic): global shift [-0.5, 2.0] pt by ink-projection correlation (centroid estimate [-3.28, -0.84] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 29.2792, differing 0.222867, SSIM₈ 0.4785 (raw 29.4286, 0.223692, 0.472)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.157→0.1678 / 47.0281→46.7859; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9981→0.9981 / 0.033→0.033
- page 3: |Δ| histogram (16 bins, pixel counts) `[1785844, 11520, 10293, 9712, 9016, 8675, 9645, 9066, 9095, 8053, 7541, 7219, 7542, 6841, 7359, 31395]`; ink px ref/ours 46441/47458 (ratio 1.0219); SSIM blocks <0.9: 6883/30294; [overlay](images/08-two-page/pdflatex-lm-main-export-p3-overlay.png) (69373 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-main-export-p3-heatmap.png) (56293 B, ÷4)
  - registration error (diagnostic): global shift [-0.5, 5.5] pt by ink-projection correlation (centroid estimate [-2.52, -35.59] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 10.426, differing 0.081791, SSIM₈ 0.8055 (raw 11.5909, 0.087982, 0.779)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.647→0.6914 / 18.5244→16.6256; header-band 1.0→0.9874 / 0.0→0.2458; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -80.13; `oak` dx 425.86 dy -78.09; `oak` dx 425.86 dy -69.19

### 08-two-page — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1679192, 29965, 24493, 23156, 22217, 20414, 18450, 17405, 14935, 13089, 11268, 10162, 9717, 8951, 8308, 27094]`; ink px ref/ours 105111/137086 (ratio 1.3042); SSIM blocks <0.9: 11030/30294; [overlay](images/08-two-page/pdflatex-lm-pipeline-export-p1-overlay.png) (54460 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-pipeline-export-p1-heatmap.png) (82008 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.69, -0.5] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9957, differing 0.157351, SSIM₈ 0.7772 (raw 15.9957, 0.157351, 0.7772)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6448→0.6448 / 25.5504→25.5504; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9982→0.9982 / 0.0807→0.0807
- page 2: |Δ| histogram (16 bins, pixel counts) `[1678985, 30149, 24297, 23242, 22089, 20485, 18552, 17293, 14865, 13387, 11134, 10107, 9908, 8818, 8275, 27230]`; ink px ref/ours 105011/137137 (ratio 1.3059); SSIM blocks <0.9: 11093/30294; [overlay](images/08-two-page/pdflatex-lm-pipeline-export-p2-overlay.png) (54589 B, ÷8), [heatmap](images/08-two-page/pdflatex-lm-pipeline-export-p2-heatmap.png) (82344 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.64, -0.18] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 16.0132, differing 0.157422, SSIM₈ 0.7762 (raw 16.0132, 0.157422, 0.7762)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.643→0.643 / 25.5855→25.5855; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9993→0.9993 / 0.0305→0.0305
- page 3: |Δ| histogram (16 bins, pixel counts) `[1824272, 13224, 10799, 10254, 9898, 9061, 7948, 7845, 6510, 5677, 4980, 4445, 4353, 3831, 3690, 12029]`; ink px ref/ours 46441/60651 (ratio 1.306); SSIM blocks <0.9: 4897/30294; [overlay](images/08-two-page/pdflatex-lm-pipeline-export-p3-overlay.png) (73559 B, ÷4), [heatmap](images/08-two-page/pdflatex-lm-pipeline-export-p3-heatmap.png) (46037 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.16, 0.76] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.0546, differing 0.069485, SSIM₈ 0.9014 (raw 7.0546, 0.069485, 0.9014)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8427→0.8427 / 11.2735→11.2735; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `quick` dx 0.01 dy 0.67; `into` dx -0.01 dy 0.67; `the` dx 0.01 dy 0.67

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

### 08-two-page — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1589554, 30035, 26334, 24859, 22978, 21404, 21766, 21185, 20523, 18166, 17141, 16749, 15769, 15639, 15257, 61457]`; ink px ref/ours 152232/105096 (ratio 0.6904); SSIM blocks <0.9: 14476/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.85, -0.69] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.1569, differing 0.20198, SSIM₈ 0.586 (raw 25.1569, 0.20198, 0.586)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.34→0.34 / 40.1829→40.1829; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9945→0.9945 / 0.1185→0.1185
- page 2: |Δ| histogram (16 bins, pixel counts) `[1593196, 30421, 26231, 24302, 23135, 21582, 21319, 20860, 19900, 18292, 17282, 16550, 15488, 14926, 14780, 60552]`; ink px ref/ours 151458/104953 (ratio 0.693); SSIM blocks <0.9: 14462/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.05, -0.27] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.7849, differing 0.19994, SSIM₈ 0.5912 (raw 24.7849, 0.19994, 0.5912)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3481→0.3481 / 39.595→39.595; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.997→0.997 / 0.0615→0.0615
- page 3: |Δ| histogram (16 bins, pixel counts) `[1817816, 10026, 8866, 8215, 7987, 7380, 7700, 7799, 7545, 6396, 6224, 5705, 5661, 5334, 5453, 20709]`; ink px ref/ours 33659/46427 (ratio 1.3793); SSIM blocks <0.9: 5879/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 29.0] pt by ink-projection correlation (centroid estimate [-0.54, 71.77] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.7285, differing 0.07013, SSIM₈ 0.8252 (raw 8.7388, 0.070108, 0.825)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7206→0.7475 / 13.9645→12.8618; header-band 1.0→0.8156 / 0.0→7.4803; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 145.22; `branch` dx -413.66 dy 145.22; `oak` dx -415.83 dy 130.78

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1581514, 30310, 25655, 23625, 21884, 20614, 19810, 19511, 18729, 18276, 17709, 17748, 16267, 16096, 16284, 74784]`; ink px ref/ours 152232/137086 (ratio 0.9005); SSIM blocks <0.9: 14338/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.6, -1.19] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.7219, differing 0.206537, SSIM₈ 0.591 (raw 26.8247, 0.206341, 0.5903)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3471→0.3519 / 42.8427→42.5843; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9952→0.9948 / 0.1455→0.1455
- page 2: |Δ| histogram (16 bins, pixel counts) `[1583267, 30507, 25568, 23451, 21864, 20550, 19560, 19158, 18824, 18350, 18087, 17587, 15995, 15799, 16076, 74173]`; ink px ref/ours 151458/137137 (ratio 0.9054); SSIM blocks <0.9: 14359/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 26.8131, not lower; centroid estimate [-0.49, -0.49] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.6431, differing 0.205257, SSIM₈ 0.593 (raw 26.6431, 0.205257, 0.593)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3512→0.3512 / 42.5609→42.5609; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9975→0.9975 / 0.0759→0.0759
- page 3: |Δ| histogram (16 bins, pixel counts) `[1809987, 9969, 8587, 7647, 7574, 6914, 6757, 6873, 6716, 6539, 6297, 6628, 6044, 6080, 6158, 30046]`; ink px ref/ours 33659/60651 (ratio 1.8019); SSIM blocks <0.9: 5837/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.62, 72.52] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 9.977, differing 0.073917, SSIM₈ 0.8275 (raw 10.0509, 0.074152, 0.8255)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7216→0.726 / 16.0605→15.9227; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 144.86; `branch` dx -413.66 dy 144.86; `oak` dx -415.83 dy 130.42

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

### 08-two-page — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 105020/105096 (ratio 1.0007); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.05, 0.11] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2647, differing 0.074223, SSIM₈ 0.9998 (raw 0.2647, 0.074223, 0.9998)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9997→0.9997 / 0.4228→0.4228; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.001→0.001
- page 2: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 104857/104953 (ratio 1.0009); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.16, -0.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2657, differing 0.074348, SSIM₈ 0.9998 (raw 0.2657, 0.074348, 0.9998)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9997→0.9997 / 0.4245→0.4245; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0009→0.0009
- page 3: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 46395/46427 (ratio 1.0007); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.14, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1181, differing 0.032782, SSIM₈ 0.9999 (raw 0.1181, 0.032782, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.1887→0.1887; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `over` dx 0.02 dy 1.03; `patient` dx 0.02 dy 1.03; `quick` dx 0.02 dy 1.03

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1679357, 30124, 24638, 22660, 22616, 20044, 18589, 17414, 14674, 13544, 11017, 10313, 9730, 9018, 7823, 27255]`; ink px ref/ours 105020/137086 (ratio 1.3053); SSIM blocks <0.9: 11038/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.71, -0.39] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9674, differing 0.157256, SSIM₈ 0.7776 (raw 15.9674, 0.157256, 0.7776)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6454→0.6454 / 25.5052→25.5052; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9982→0.9982 / 0.0801→0.0801
- page 2: |Δ| histogram (16 bins, pixel counts) `[1679110, 30421, 24375, 22610, 22614, 20086, 18738, 17244, 14689, 13787, 10894, 10243, 9896, 8924, 7861, 27324]`; ink px ref/ours 104857/137137 (ratio 1.3078); SSIM blocks <0.9: 11078/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.7, -0.24] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9858, differing 0.157306, SSIM₈ 0.7766 (raw 15.9858, 0.157306, 0.7766)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6437→0.6437 / 25.5417→25.5417; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9992→0.9992 / 0.0304→0.0304
- page 3: |Δ| histogram (16 bins, pixel counts) `[1824302, 13338, 10890, 9997, 10133, 8910, 7982, 7785, 6419, 5910, 4861, 4508, 4332, 3874, 3512, 12063]`; ink px ref/ours 46395/60651 (ratio 1.3073); SSIM blocks <0.9: 4895/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.94, 0.77] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.0406, differing 0.069424, SSIM₈ 0.9017 (raw 7.0406, 0.069424, 0.9017)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8431→0.8431 / 11.2511→11.2511; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `over` dx 0.02 dy 0.67; `patient` dx 0.02 dy 0.67; `quick` dx 0.02 dy 0.67

### 09-mixed-document — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908424, 2029, 1775, 1940, 1563, 1506, 1540, 1401, 1384, 1449, 1711, 1473, 1402, 1450, 1477, 8292]`; ink px ref/ours 9691/9252 (ratio 0.9547); SSIM blocks <0.9: 1549/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 38.5] pt by ink-projection correlation (centroid estimate [6.27, 32.14] pt); confidence strong (shift explains 28% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.7912, differing 0.014073, SSIM₈ 0.9702 (raw 2.4844, 0.017426, 0.9504)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9208→0.9543 / 3.9708→2.7408; header-band 1.0→0.9861 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6356→0.7568 / 15.3342→9.5623 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -2.87 dy 39.06; `in` dx -2.66 dy 39.06; `set` dx -2.46 dy 39.06
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.7470407485962, 653.6572866439819, 5.8530473709106445, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1915305, 1868, 1640, 1537, 1411, 1477, 1587, 1507, 1429, 1122, 1266, 1158, 1091, 1118, 1032, 4268]`; ink px ref/ours 9691/7613 (ratio 0.7856); SSIM blocks <0.9: 945/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -1.0] pt by ink-projection correlation (centroid estimate [6.89, -2.51] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.6074, differing 0.013036, SSIM₈ 0.9744 (raw 1.7278, 0.013589, 0.9714)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9542→0.9591 / 2.7615→2.569; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7994→0.8404 / 10.5606→9.1291 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -436.13 dy 14.25; `paper.` dx 53.14 dy -0.19; `letter` dx 49.47 dy -0.19

### 09-mixed-document — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [288.61, 632.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908246, 2035, 1831, 1910, 1508, 1488, 1544, 1411, 1385, 1423, 1698, 1502, 1432, 1498, 1436, 8469]`; ink px ref/ours 9691/9418 (ratio 0.9718); SSIM blocks <0.9: 1564/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 38.5] pt by ink-projection correlation (centroid estimate [9.64, 31.03] pt); confidence strong (shift explains 34% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.6551, differing 0.013719, SSIM₈ 0.9717 (raw 2.5088, 0.017511, 0.9499)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9199→0.9573 / 4.0097→2.5079; header-band 1.0→0.9829 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6325→0.7572 / 15.3393→9.3809 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -3.65 dy 39.06; `in` dx -3.44 dy 39.06; `set` dx -3.24 dy 39.06
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.747, 653.657, 6.975, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1914074, 1839, 1590, 1512, 1330, 1480, 1376, 1353, 1308, 1154, 1326, 1253, 1241, 1294, 1177, 5509]`; ink px ref/ours 9691/9434 (ratio 0.9735); SSIM blocks <0.9: 943/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, -1.0] pt by ink-projection correlation (centroid estimate [14.81, 0.87] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.7954, differing 0.013621, SSIM₈ 0.9729 (raw 1.9266, 0.014163, 0.9706)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.953→0.9569 / 3.0792→2.868; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7831→0.7632 / 12.188→13.2958 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -436.13 dy 13.9; `paper.` dx 53.14 dy -0.55; `letter` dx 49.47 dy -0.55
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; delete ref ['2', '+'] ours []

### 09-mixed-document — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909444, 2180, 1960, 2265, 1611, 1596, 1560, 1754, 1443, 1429, 1644, 1400, 1251, 1263, 1288, 6728]`; ink px ref/ours 7447/9252 (ratio 1.2424); SSIM blocks <0.9: 1580/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 39.0] pt by ink-projection correlation (centroid estimate [0.63, 34.84] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9017, differing 0.014708, SSIM₈ 0.9641 (raw 2.2538, 0.016899, 0.9494)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9192→0.9448 / 3.6023→2.9175; header-band 1.0→0.9856 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6352→0.7524 / 13.0739→8.01 [275.7,96.4–519.8,156 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 24.19; `paper.` dx -52.18 dy 38.59; `letter` dx -48.63 dy 38.59
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.7470407485962, 653.6572866439819, 5.8530473709106445, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1929092, 1462, 1151, 1111, 872, 875, 884, 793, 610, 460, 408, 388, 236, 207, 91, 176]`; ink px ref/ours 7447/7613 (ratio 1.0223); SSIM blocks <0.9: 526/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -0.5] pt by ink-projection correlation (centroid estimate [1.25, 0.19] pt); confidence moderate (shift explains 22% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3817, differing 0.008255, SSIM₈ 0.9955 (raw 0.4905, 0.008508, 0.9918)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.987→0.9928 / 0.784→0.61; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.9839→0.9253 / 1.2156→4.4015 [275.7,96.4–519.8,156 pt]
- largest word displacements (pt): `=` dx 0.0 dy 1.12; `z2` dx 0.0 dy 1.12; `second` dx -0.03 dy -0.62

### 09-mixed-document — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [288.61, 632.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909340, 2188, 2000, 2239, 1567, 1573, 1552, 1764, 1454, 1434, 1625, 1423, 1278, 1325, 1232, 6822]`; ink px ref/ours 7447/9418 (ratio 1.2647); SSIM blocks <0.9: 1591/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 39.0] pt by ink-projection correlation (centroid estimate [4.0, 33.73] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8972, differing 0.014691, SSIM₈ 0.9637 (raw 2.268, 0.016945, 0.949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9186→0.9446 / 3.6249→2.8949; header-band 1.0→0.9825 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.632→0.7532 / 13.0739→7.8507 [275.7,96.4–519.8,156 pt]
- largest word displacements (pt): `twelve` dx 432.48 dy 24.19; `paper.` dx -52.82 dy 38.59; `letter` dx -49.27 dy 38.59
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.747, 653.657, 6.975, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1918421, 2044, 1751, 1790, 1567, 1425, 1211, 1351, 1184, 1001, 1045, 934, 877, 782, 689, 2744]`; ink px ref/ours 7447/9434 (ratio 1.2668); SSIM blocks <0.9: 835/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -0.5] pt by ink-projection correlation (centroid estimate [9.16, 3.57] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2413, differing 0.011396, SSIM₈ 0.9824 (raw 1.3481, 0.011945, 0.98)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.968→0.9719 / 2.1547→1.984; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.8776→0.8461 / 7.1704→8.1838 [275.7,96.4–519.8,156 pt]
- largest word displacements (pt): `Mixed` dx 29.05 dy -0.67; `second` dx -0.03 dy -0.97; `wrap` dx -0.02 dy -0.97
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908448, 1975, 1818, 1995, 1553, 1434, 1436, 1374, 1435, 1503, 1793, 1550, 1349, 1400, 1470, 8283]`; ink px ref/ours 9857/9252 (ratio 0.9386); SSIM blocks <0.9: 1551/30294; [overlay](images/09-mixed-document/pdflatex-de1020c-export-p1-overlay.png) (53601 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-de1020c-export-p1-heatmap.png) (55198 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 38.5] pt by ink-projection correlation (centroid estimate [7.57, 32.71] pt); confidence moderate (shift explains 24% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8886, differing 0.014393, SSIM₈ 0.9685 (raw 2.4853, 0.017384, 0.9504)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9207→0.9518 / 3.9723→2.8965; header-band 1.0→0.9857 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6361→0.7546 / 15.3285→9.5698 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `twelve` dx -3.05 dy 39.17; `in` dx -2.78 dy 39.17; `set` dx -2.52 dy 39.17
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.7470407485962, 653.6572866439819, 5.8530473709106445, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1915290, 1842, 1658, 1523, 1434, 1315, 1495, 1484, 1480, 1229, 1390, 1239, 1107, 1075, 1032, 4223]`; ink px ref/ours 9857/7613 (ratio 0.7723); SSIM blocks <0.9: 945/30294; [overlay](images/09-mixed-document/pdflatex-exact-export-p1-overlay.png) (54998 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-exact-export-p1-heatmap.png) (48486 B, ÷2)
  - registration error (diagnostic): global shift [0.0, -1.0] pt by ink-projection correlation (centroid estimate [8.19, -1.94] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.6135, differing 0.013097, SSIM₈ 0.9744 (raw 1.7369, 0.013577, 0.9713)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9541→0.9591 / 2.776→2.5788; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7966→0.845 / 10.7018→8.8062 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `twelve` dx -436.31 dy 14.37; `paper.` dx 53.73 dy -0.08; `letter` dx 50.08 dy -0.08

### 09-mixed-document — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [288.61, 632.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908288, 1965, 1866, 1969, 1496, 1406, 1434, 1380, 1445, 1488, 1794, 1574, 1372, 1454, 1428, 8457]`; ink px ref/ours 9857/9418 (ratio 0.9555); SSIM blocks <0.9: 1566/30294; [overlay](images/09-mixed-document/pdflatex-main-export-p1-overlay.png) (52825 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-main-export-p1-heatmap.png) (55083 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 38.5] pt by ink-projection correlation (centroid estimate [10.94, 31.6] pt); confidence strong (shift explains 26% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8697, differing 0.014363, SSIM₈ 0.9687 (raw 2.51, 0.017464, 0.9498)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9198→0.9525 / 4.0117→2.8509; header-band 1.0→0.9829 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.633→0.7576 / 15.3348→9.382 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `twelve` dx -3.83 dy 39.17; `in` dx -3.56 dy 39.17; `set` dx -3.3 dy 39.17
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.747, 653.657, 6.975, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1914037, 1785, 1621, 1470, 1379, 1367, 1241, 1362, 1382, 1237, 1440, 1325, 1168, 1199, 1203, 5600]`; ink px ref/ours 9857/9434 (ratio 0.9571); SSIM blocks <0.9: 944/30294; [overlay](images/09-mixed-document/pdflatex-pipeline-export-p1-overlay.png) (53576 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-pipeline-export-p1-heatmap.png) (47083 B, ÷2)
  - registration error (diagnostic): global shift [1.0, -1.0] pt by ink-projection correlation (centroid estimate [16.11, 1.44] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.7758, differing 0.013569, SSIM₈ 0.9733 (raw 1.9404, 0.014107, 0.9705)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9529→0.9575 / 3.1014→2.8367; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7819→0.7652 / 12.2367→13.1783 [275.7,96.9–488.1,156.6 pt]
- largest word displacements (pt): `twelve` dx -436.31 dy 14.01; `paper.` dx 53.73 dy -0.44; `letter` dx 50.08 dy -0.44
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909812, 2060, 1849, 1927, 1609, 1602, 1682, 1704, 1540, 1378, 1642, 1427, 1316, 1278, 1363, 6627]`; ink px ref/ours 7573/9252 (ratio 1.2217); SSIM blocks <0.9: 1632/30294; [overlay](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-overlay.png) (54147 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-de1020c-export-p1-heatmap.png) (55497 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 39.5] pt by ink-projection correlation (centroid estimate [-1.26, 34.72] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9012, differing 0.014685, SSIM₈ 0.9623 (raw 2.2522, 0.016745, 0.948)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9169→0.9418 / 3.5997→2.9167; header-band 1.0→0.9856 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.381→0.5284 / 23.145→12.7296 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 25.83; `paper.` dx -52.2 dy 40.23; `letter` dx -48.65 dy 40.23
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.7470407485962, 653.6572866439819, 5.8530473709106445, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1936717, 1744, 226, 75, 14, 11, 14, 8, 6, 1, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 7573/7613 (ratio 1.0053); SSIM blocks <0.9: 13/30294; [overlay](images/09-mixed-document/pdflatex-lm-exact-export-p1-overlay.png) (49750 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-exact-export-p1-heatmap.png) (79006 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.64, 0.06] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0423, differing 0.004471, SSIM₈ 0.9998 (raw 0.0423, 0.004471, 0.9998)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9997→0.9997 / 0.0677→0.0677; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.9903→0.9903 / 1.3383→1.3383 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `Mixed` dx 0 dy 1.64; `bold,` dx -0.06 dy 1.14; `the` dx -0.06 dy 1.14

### 09-mixed-document — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [288.61, 632.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909701, 2067, 1886, 1907, 1559, 1585, 1671, 1726, 1552, 1374, 1628, 1446, 1343, 1338, 1307, 6726]`; ink px ref/ours 7573/9418 (ratio 1.2436); SSIM blocks <0.9: 1643/30294; [overlay](images/09-mixed-document/pdflatex-lm-main-export-p1-overlay.png) (53334 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-main-export-p1-heatmap.png) (55351 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 39.5] pt by ink-projection correlation (centroid estimate [2.11, 33.61] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8976, differing 0.01467, SSIM₈ 0.9619 (raw 2.2668, 0.016793, 0.9476)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9163→0.9417 / 3.623→2.8955; header-band 1.0→0.9826 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3685→0.5048 / 23.2695→12.7299 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `twelve` dx 432.48 dy 25.83; `paper.` dx -52.84 dy 40.23; `letter` dx -49.29 dy 40.23
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.747, 653.657, 6.975, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920390, 1986, 1767, 1642, 1519, 1350, 1262, 1258, 938, 826, 824, 682, 678, 637, 540, 2517]`; ink px ref/ours 7573/9434 (ratio 1.2457); SSIM blocks <0.9: 770/30294; [overlay](images/09-mixed-document/pdflatex-lm-pipeline-export-p1-overlay.png) (53110 B, ÷2), [heatmap](images/09-mixed-document/pdflatex-lm-pipeline-export-p1-heatmap.png) (43815 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [7.27, 3.44] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1628, differing 0.011074, SSIM₈ 0.9837 (raw 1.1782, 0.011125, 0.9835)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9736→0.974 / 1.8832→1.8585; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.8592→0.7778 / 6.2733→9.3895 [275.7,120.7–336.3,155.7 pt]
- largest word displacements (pt): `Mixed` dx 29.05 dy 0.96; `the` dx -0.06 dy 0.67; `UTF-8` dx -0.06 dy 0.67
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; delete ref ['2', '+'] ours []

### 09-mixed-document — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908485, 1993, 1726, 1891, 1568, 1563, 1547, 1422, 1387, 1527, 1642, 1498, 1332, 1389, 1540, 8306]`; ink px ref/ours 9680/9252 (ratio 0.9558); SSIM blocks <0.9: 1553/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 38.5] pt by ink-projection correlation (centroid estimate [7.53, 32.15] pt); confidence strong (shift explains 25% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8537, differing 0.01428, SSIM₈ 0.969 (raw 2.4838, 0.017418, 0.9503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9206→0.9525 / 3.9698→2.8407; header-band 1.0→0.9857 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6356→0.7542 / 15.3342→9.5697 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -2.91 dy 39.06; `in` dx -2.65 dy 39.06; `set` dx -2.4 dy 39.06
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.7470407485962, 653.6572866439819, 5.8530473709106445, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1915252, 1910, 1609, 1474, 1420, 1456, 1586, 1531, 1420, 1160, 1232, 1238, 1049, 1098, 1082, 4299]`; ink px ref/ours 9680/7613 (ratio 0.7865); SSIM blocks <0.9: 946/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -1.0] pt by ink-projection correlation (centroid estimate [8.15, -2.5] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.6157, differing 0.013073, SSIM₈ 0.9743 (raw 1.7368, 0.01362, 0.9712)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.954→0.9589 / 2.7759→2.5823; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7992→0.8402 / 10.5676→9.135 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -436.17 dy 14.25; `paper.` dx 53.75 dy -0.19; `letter` dx 50.09 dy -0.19

### 09-mixed-document — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [288.61, 632.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1908309, 1986, 1783, 1855, 1518, 1546, 1545, 1430, 1391, 1495, 1633, 1525, 1365, 1450, 1503, 8482]`; ink px ref/ours 9680/9418 (ratio 0.9729); SSIM blocks <0.9: 1568/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 38.5] pt by ink-projection correlation (centroid estimate [10.9, 31.05] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8378, differing 0.014258, SSIM₈ 0.969 (raw 2.5096, 0.017501, 0.9498)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9197→0.953 / 4.011→2.8; header-band 1.0→0.9829 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6325→0.7572 / 15.3393→9.3809 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -3.69 dy 39.06; `in` dx -3.43 dy 39.06; `set` dx -3.18 dy 39.06
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.747, 653.657, 6.975, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1914018, 1791, 1541, 1452, 1356, 1519, 1361, 1378, 1287, 1209, 1260, 1278, 1165, 1270, 1242, 5689]`; ink px ref/ours 9680/9434 (ratio 0.9746); SSIM blocks <0.9: 944/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, -1.0] pt by ink-projection correlation (centroid estimate [16.06, 0.88] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.7686, differing 0.013519, SSIM₈ 0.9732 (raw 1.9462, 0.014203, 0.9703)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9525→0.9574 / 3.1105→2.8252; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7828→0.7632 / 12.2055→13.2965 [275.7,96.8–488.2,156.5 pt]
- largest word displacements (pt): `twelve` dx -436.17 dy 13.9; `paper.` dx 53.75 dy -0.55; `letter` dx 50.09 dy -0.55
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; delete ref ['2', '+'] ours []

### 09-mixed-document — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [286.63, 632.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909464, 2171, 1943, 2272, 1611, 1596, 1548, 1766, 1439, 1468, 1597, 1418, 1229, 1257, 1310, 6727]`; ink px ref/ours 7446/9252 (ratio 1.2425); SSIM blocks <0.9: 1581/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 39.0] pt by ink-projection correlation (centroid estimate [0.78, 34.82] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.9016, differing 0.014705, SSIM₈ 0.9641 (raw 2.2539, 0.016899, 0.9494)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9191→0.9448 / 3.6023→2.9173; header-band 1.0→0.9856 / 0.0→0.8398; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3967→0.5342 / 23.5725→12.9044 [275.7,121.1–336.2,156 pt]
- largest word displacements (pt): `twelve` dx 433.26 dy 25.22; `paper.` dx -52.18 dy 39.62; `letter` dx -48.63 dy 39.62
- word-sequence differences: replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']; delete ref ['2', '+'] ours []

### 09-mixed-document — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.7470407485962, 653.6572866439819, 5.8530473709106445, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1929102, 1457, 1149, 1114, 868, 870, 885, 792, 616, 458, 410, 384, 235, 209, 91, 176]`; ink px ref/ours 7446/7613 (ratio 1.0224); SSIM blocks <0.9: 526/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, -0.5] pt by ink-projection correlation (centroid estimate [1.4, 0.17] pt); confidence moderate (shift explains 22% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3815, differing 0.008295, SSIM₈ 0.9955 (raw 0.4909, 0.008582, 0.9918)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.987→0.9928 / 0.7846→0.6098; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.8727→0.9354 / 6.0046→4.337 [275.7,121.1–336.2,156 pt]
- largest word displacements (pt): `Mixed` dx 0 dy 1.64; `with` dx -0.02 dy 1.12; `emphasis,` dx -0.02 dy 1.12

### 09-mixed-document — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (1): warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [288.61, 632.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1909360, 2179, 1983, 2246, 1567, 1573, 1540, 1776, 1450, 1473, 1578, 1441, 1256, 1318, 1254, 6822]`; ink px ref/ours 7446/9418 (ratio 1.2648); SSIM blocks <0.9: 1592/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 39.0] pt by ink-projection correlation (centroid estimate [4.15, 33.71] pt); confidence moderate (shift explains 16% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.8972, differing 0.01469, SSIM₈ 0.9637 (raw 2.268, 0.016944, 0.949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9186→0.9446 / 3.6249→2.8948; header-band 1.0→0.9825 / 0.0→0.9457; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3845→0.5106 / 23.6675→12.9047 [275.7,121.1–336.2,156 pt]
- largest word displacements (pt): `twelve` dx 432.48 dy 25.22; `paper.` dx -52.82 dy 39.62; `letter` dx -49.27 dy 39.62
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; replace ref ['x2', '+', 'y2', '=', 'z2'] ours ['x', '2+y2', '=z2']

### 09-mixed-document — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [279.747, 653.657, 6.975, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1918432, 2046, 1750, 1772, 1582, 1416, 1212, 1370, 1163, 1032, 1018, 930, 868, 787, 695, 2743]`; ink px ref/ours 7446/9434 (ratio 1.267); SSIM blocks <0.9: 836/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -0.5] pt by ink-projection correlation (centroid estimate [9.31, 3.55] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2414, differing 0.011396, SSIM₈ 0.9824 (raw 1.3478, 0.011945, 0.98)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.968→0.9718 / 2.1541→1.9842; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7845→0.7536 / 9.1973→10.5708 [275.7,121.1–336.2,156 pt]
- largest word displacements (pt): `Mixed` dx 29.05 dy 0.96; `with` dx -0.02 dy 0.66; `math` dx 0.02 dy 0.66
- word-sequence differences: insert ref [] ours ['1']; replace ref ['bold,', 'emphasis,'] ours ['bold', ',', 'emphasis', ',']; delete ref ['2', '+'] ours []

### 10-unicode-paragraph — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893839, 3905, 3475, 2974, 2739, 2765, 2584, 2505, 2344, 2294, 2314, 2129, 2034, 2031, 2138, 8746]`; ink px ref/ours 18767/18449 (ratio 0.9831); SSIM blocks <0.9: 1824/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.58, 1.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3153, differing 0.026129, SSIM₈ 0.949 (raw 3.3153, 0.026129, 0.949)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9186→0.9186 / 5.2982→5.2982; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 11.75 Δy 0.0 len 10.0 vs 11.5, thickness px 1 vs 1
- word-sequence differences: replace ref ['Cremebrulee,jalapeno,facade,naive,cooperate,Angstrom,Ærø,Þorr,Øresund,Skoda,Łodz,Is-', 'tanbul,Zurich—anemdash;1990–1995anendash;“curlydoublequotes”,‘curlysinglequotes’,', '«guillemets»,and‚Germanlowquotes‘.', 'SenorMuller’sresumelistsSaoPaulo,Krakow,Reyk-', 'javik,MalmoandBordeaux;thecafe’smenuofferscrepes,souffleandapatethatcosts€12—', 'or£10,¥1500—perserving,1⁄2portionavailable.', 'Sæglopur,manana,Nandu,cedilla,y,ø,a,æ,', 'œuvre,Œdipe,ß,andDzclosetheline.'] ours ['Creme', 'brulee,', 'jalapeno,', 'facade,', 'naive,', 'cooperate,', 'Angstrom,', 'Ærø,']

### 10-unicode-paragraph — lualatex vs compiler `exact` (export)

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894484, 3892, 3212, 3068, 2851, 2743, 2716, 2845, 2792, 2312, 2291, 2205, 2092, 1966, 1916, 7431]`; ink px ref/ours 18767/14269 (ratio 0.7603); SSIM blocks <0.9: 1930/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.74, 2.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.1816, differing 0.025808, SSIM₈ 0.9455 (raw 3.1816, 0.025808, 0.9455)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9129→0.9129 / 5.0844→5.0844; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893194, 4061, 3326, 3020, 2761, 2683, 2417, 2506, 2365, 2404, 2324, 2206, 2143, 2140, 2169, 9097]`; ink px ref/ours 18767/18401 (ratio 0.9805); SSIM blocks <0.9: 1866/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.87, 1.79] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3978, differing 0.026531, SSIM₈ 0.9472 (raw 3.3978, 0.026531, 0.9472)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9156→0.9156 / 5.43→5.43; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- word-sequence differences: replace ref ['Cremebrulee,jalapeno,facade,naive,cooperate,Angstrom,Ærø,Þorr,Øresund,Skoda,Łodz,Is-', 'tanbul,Zurich—anemdash;1990–1995anendash;“curlydoublequotes”,‘curlysinglequotes’,', '«guillemets»,and‚Germanlowquotes‘.', 'SenorMuller’sresumelistsSaoPaulo,Krakow,Reyk-', 'javik,MalmoandBordeaux;thecafe’smenuofferscrepes,souffleandapatethatcosts€12—', 'or£10,¥1500—perserving,1⁄2portionavailable.', 'Sæglopur,manana,Nandu,cedilla,y,ø,a,æ,', 'œuvre,Œdipe,ß,andDzclosetheline.'] ours ['Creme', 'brulee,', 'jalapeno,', 'facade,', 'naive,', 'cooperate,', 'Angstrom,', 'Ærø,']

### 10-unicode-paragraph — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894069, 3840, 3317, 2950, 2789, 2858, 2845, 2787, 2685, 2416, 2321, 2100, 2019, 1981, 2036, 7803]`; ink px ref/ours 14211/18449 (ratio 1.2982); SSIM blocks <0.9: 1927/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.22, -0.91] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2356, differing 0.025902, SSIM₈ 0.9447 (raw 3.2356, 0.025902, 0.9447)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9117→0.9117 / 5.1715→5.1715; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 95.5 pt with NO reference rule on this page
- largest word displacements (pt): `that` dx 447.55 dy -14.89; `Łódź,` dx 428.89 dy -14.75; `single` dx 411.29 dy -14.79
- word-sequence differences: replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']; replace ref ['�'] ours ['Dz']

### 10-unicode-paragraph — lualatex-lm vs compiler `exact` (export)

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938018, 798, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 14211/14269 (ratio 1.0041); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.11, 0.11] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.055, differing 0.010486, SSIM₈ 0.9999 (raw 0.055, 0.010486, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.0878→0.0878; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `portion` dx 0.05 dy 0.0; `serving,` dx 0.04 dy 0.0; `½` dx 0.04 dy 0.0
- word-sequence differences: replace ref ['quotes”,'] ours ['quotes”', ',']; replace ref ['quotes’,'] ours ['quotes’', ',']; replace ref ['quotes‘.'] ours ['quotes‘', '.']

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1902682, 3966, 3360, 3086, 2736, 2752, 2442, 2310, 1968, 1763, 1657, 1723, 1438, 1261, 1221, 4451]`; ink px ref/ours 14211/18401 (ratio 1.2948); SSIM blocks <0.9: 1548/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [2.24, -0.14] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 2.187, differing 0.021255, SSIM₈ 0.968 (raw 2.3178, 0.021806, 0.9657)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9452→0.9489 / 3.7045→3.4955; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `portion` dx 0.05 dy -0.36; `serving,` dx 0.04 dy -0.36; `½` dx 0.04 dy -0.36
- word-sequence differences: delete ref ['�'] ours []

### 10-unicode-paragraph — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893459, 4089, 3397, 2841, 2581, 2734, 2699, 2550, 2363, 2423, 2285, 2125, 2111, 2043, 2055, 9061]`; ink px ref/ours 18728/18449 (ratio 0.9851); SSIM blocks <0.9: 1812/30294; [overlay](images/10-unicode-paragraph/pdflatex-de1020c-export-p1-overlay.png) (78053 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-de1020c-export-p1-heatmap.png) (60267 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.7, 1.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.362, differing 0.026242, SSIM₈ 0.9486 (raw 3.362, 0.026242, 0.9486)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.918→0.918 / 5.373→5.373; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 12.0 Δy 0.0 len 10.0 vs 12.0, thickness px 1 vs 1
- largest word displacements (pt): `quotes’,` dx -429.81 dy 14.82; `æ,` dx -421.71 dy 14.68; `å,` dx -421.21 dy 14.68
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['C12'] ours ['€12']

### 10-unicode-paragraph — pdflatex vs compiler `exact` (export)

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894472, 3883, 3271, 3013, 2698, 2733, 2725, 2928, 2742, 2372, 2286, 2210, 2120, 1937, 1903, 7523]`; ink px ref/ours 18728/14269 (ratio 0.7619); SSIM blocks <0.9: 1908/30294; [overlay](images/10-unicode-paragraph/pdflatex-exact-export-p1-overlay.png) (79894 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-exact-export-p1-heatmap.png) (63259 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 3.1981, not lower; centroid estimate [0.62, 2.16] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.1919, differing 0.025822, SSIM₈ 0.9456 (raw 3.1919, 0.025822, 0.9456)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9131→0.9131 / 5.1011→5.1011; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `single` dx -398.5 dy 15.21; `Kraków,` dx -394.31 dy 15.21; `that` dx -381.98 dy 15.21
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['quotes”,'] ours ['quotes”', ',']; replace ref ['quotes’,'] ours ['quotes’', ',']

### 10-unicode-paragraph — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (5): warning: Times-Roman has no glyph for 'ǅ' (U+01C5); warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893850, 4023, 3381, 2853, 2611, 2771, 2581, 2546, 2501, 2376, 2328, 2074, 2157, 2017, 2012, 8735]`; ink px ref/ours 18728/18531 (ratio 0.9895); SSIM blocks <0.9: 1783/30294; [overlay](images/10-unicode-paragraph/pdflatex-main-export-p1-overlay.png) (76962 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-main-export-p1-heatmap.png) (59192 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-2.5, 0.0] pt REJECTED: applying it gives mean|Δ| 3.3648, not lower; centroid estimate [-2.34, 0.6] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3173, differing 0.026064, SSIM₈ 0.9499 (raw 3.3173, 0.026064, 0.9499)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.92→0.92 / 5.3014→5.3014; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `æ,` dx -457.04 dy 14.68; `—` dx -456.05 dy 14.72; `quotes’,` dx -429.81 dy 14.82
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['C12'] ours ['€12']

### 10-unicode-paragraph — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893858, 4054, 3244, 2959, 2618, 2638, 2434, 2503, 2285, 2338, 2245, 2178, 2138, 2135, 2078, 9111]`; ink px ref/ours 18728/18401 (ratio 0.9825); SSIM blocks <0.9: 1842/30294; [overlay](images/10-unicode-paragraph/pdflatex-pipeline-export-p1-overlay.png) (77688 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-pipeline-export-p1-heatmap.png) (61372 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.75, 1.91] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.357, differing 0.026335, SSIM₈ 0.9479 (raw 3.357, 0.026335, 0.9479)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9168→0.9168 / 5.3649→5.3649; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `single` dx -398.5 dy 14.85; `quotes’,` dx -398.1 dy 14.85; `Kraków,` dx -394.31 dy 14.85
- word-sequence differences: replace ref ['Łod z,', ' ', 'Is-', 'tanbul,'] ours ['Łodz,', 'Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['C12'] ours ['€12']

### 10-unicode-paragraph — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893882, 3869, 3242, 2959, 2871, 2862, 2862, 2749, 2745, 2384, 2325, 2135, 2070, 1985, 2056, 7820]`; ink px ref/ours 14393/18449 (ratio 1.2818); SSIM blocks <0.9: 1933/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-de1020c-export-p1-overlay.png) (79085 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-de1020c-export-p1-heatmap.png) (62617 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.76, -1.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.251, differing 0.025995, SSIM₈ 0.9445 (raw 3.251, 0.025995, 0.9445)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9112→0.9112 / 5.1961→5.1961; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 95.5 pt with NO reference rule on this page
- largest word displacements (pt): `that` dx 447.55 dy -14.89; `Łódź,` dx 428.89 dy -14.75; `single` dx 411.29 dy -14.79
- word-sequence differences: replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']

### 10-unicode-paragraph — pdflatex-lm vs compiler `exact` (export)

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1937445, 148, 98, 95, 84, 128, 65, 114, 75, 65, 54, 59, 51, 49, 54, 232]`; ink px ref/ours 14393/14269 (ratio 0.9914); SSIM blocks <0.9: 73/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-exact-export-p1-overlay.png) (64639 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-exact-export-p1-heatmap.png) (87986 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.44, -0.28] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1071, differing 0.007238, SSIM₈ 0.998 (raw 0.1071, 0.007238, 0.998)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9968→0.9968 / 0.1712→0.1712; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `the` dx -10.8 dy 0.0; `close` dx -10.79 dy 0.0; `line.` dx -10.79 dy 0.0
- word-sequence differences: replace ref ['quotes”,'] ours ['quotes”', ',']; replace ref ['quotes’,'] ours ['quotes’', ',']; replace ref ['quotes‘.'] ours ['quotes‘', '.']

### 10-unicode-paragraph — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (5): warning: Times-Roman has no glyph for 'ǅ' (U+01C5); warning: 'Ł' (U+0141) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'ź' (U+017A) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts; warning: 'İ' (U+0130) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893824, 3716, 3242, 2971, 2934, 2899, 2776, 2848, 2826, 2318, 2322, 2126, 2206, 1915, 1955, 7938]`; ink px ref/ours 14393/18531 (ratio 1.2875); SSIM blocks <0.9: 1921/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-main-export-p1-overlay.png) (78702 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-main-export-p1-heatmap.png) (61968 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-3.4, -1.84] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2644, differing 0.026019, SSIM₈ 0.9446 (raw 3.2644, 0.026019, 0.9446)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9114→0.9114 / 5.2174→5.2174; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Łódź,` dx 428.36 dy -14.75; `Kraków,` dx 426.16 dy -14.84; `single` dx 411.7 dy -14.79

### 10-unicode-paragraph — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1902033, 3951, 3293, 3134, 2786, 2824, 2408, 2344, 2007, 1809, 1715, 1730, 1490, 1324, 1296, 4672]`; ink px ref/ours 14393/18401 (ratio 1.2785); SSIM blocks <0.9: 1579/30294; [overlay](images/10-unicode-paragraph/pdflatex-lm-pipeline-export-p1-overlay.png) (76579 B, ÷2), [heatmap](images/10-unicode-paragraph/pdflatex-lm-pipeline-export-p1-heatmap.png) (56465 B, ÷2)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.7, -0.53] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 2.219, differing 0.021424, SSIM₈ 0.9672 (raw 2.3844, 0.022077, 0.9643)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.943→0.9476 / 3.811→3.5466; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `the` dx -10.8 dy -0.36; `close` dx -10.79 dy -0.36; `line.` dx -10.79 dy -0.36
- word-sequence differences: delete ref ['Dz'] ours []

### 10-unicode-paragraph — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893605, 3862, 3352, 2965, 2706, 2743, 2608, 2494, 2326, 2303, 2340, 2187, 2140, 2037, 2209, 8939]`; ink px ref/ours 18684/18449 (ratio 0.9874); SSIM blocks <0.9: 1829/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.24, 1.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.3629, differing 0.026184, SSIM₈ 0.9483 (raw 3.3629, 0.026184, 0.9483)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9175→0.9175 / 5.3742→5.3742; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 11.75 Δy 0.0 len 10.0 vs 11.5, thickness px 1 vs 1
- largest word displacements (pt): `quotes’,` dx -429.86 dy 14.82; `æ,` dx -421.74 dy 14.68; `å,` dx -421.3 dy 14.68
- word-sequence differences: replace ref ['Is-', 'tanbul,'] ours ['Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']

### 10-unicode-paragraph — xelatex vs compiler `exact` (export)

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894392, 3916, 3168, 3015, 2830, 2767, 2689, 2944, 2747, 2237, 2374, 2230, 2122, 1944, 1975, 7466]`; ink px ref/ours 18684/14269 (ratio 0.7637); SSIM blocks <0.9: 1934/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 3.2148, not lower; centroid estimate [1.08, 2.06] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.1976, differing 0.025785, SSIM₈ 0.9453 (raw 3.1976, 0.025785, 0.9453)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9126→0.9126 / 5.11→5.11; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Łódź,` dx -425.09 dy 15.21; `single` dx -398.55 dy 15.21; `Kraków,` dx -394.27 dy 15.21
- word-sequence differences: replace ref ['Is-', 'tanbul,'] ours ['Istanbul,']; replace ref ['quotes”,'] ours ['quotes”', ',']; replace ref ['quotes’,'] ours ['quotes’', ',']

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1893270, 4078, 3322, 2935, 2777, 2697, 2422, 2568, 2213, 2294, 2340, 2199, 2234, 2101, 2150, 9216]`; ink px ref/ours 18684/18401 (ratio 0.9849); SSIM blocks <0.9: 1863/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [3.22, 1.8] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.4005, differing 0.026487, SSIM₈ 0.947 (raw 3.4005, 0.026487, 0.947)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9154→0.9154 / 5.4343→5.4343; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Łódź,` dx -425.09 dy 14.85; `single` dx -398.55 dy 14.85; `quotes’,` dx -398.14 dy 14.85
- word-sequence differences: replace ref ['Is-', 'tanbul,'] ours ['Istanbul,']; replace ref ['Reyk-', 'javik,'] ours ['Reykjavik,']; delete ref ['Dz'] ours []

### 10-unicode-paragraph — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1894039, 3880, 3308, 2945, 2794, 2860, 2838, 2793, 2672, 2427, 2320, 2102, 2019, 1974, 2036, 7809]`; ink px ref/ours 14217/18449 (ratio 1.2977); SSIM blocks <0.9: 1927/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-5.66, -0.92] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 3.2358, differing 0.025899, SSIM₈ 0.9447 (raw 3.2358, 0.025899, 0.9447)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9117→0.9117 / 5.1718→5.1718; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 95.5 pt with NO reference rule on this page
- largest word displacements (pt): `that` dx 447.55 dy -13.86; `Łódź,` dx 428.89 dy -13.72; `single` dx 411.29 dy -13.77
- word-sequence differences: replace ref ['œuvre,', 'Œdipe,'] ours ['œuvre,Œdipe,']; replace ref ['\uffff'] ours ['Dz']

### 10-unicode-paragraph — xelatex-lm vs compiler `exact` (export)

- FlashTeX diagnostics (1): warning: U+01C5 'ǅ' has no glyph in lmroman12-regular; nothing drawn for it
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938041, 775, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 14217/14269 (ratio 1.0037); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.34, 0.1] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0548, differing 0.0105, SSIM₈ 0.9999 (raw 0.0548, 0.0105, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.0875→0.0875; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `portion` dx 0.05 dy 1.03; `serving,` dx 0.04 dy 1.03; `½` dx 0.04 dy 1.03
- word-sequence differences: replace ref ['quotes”,'] ours ['quotes”', ',']; replace ref ['quotes’,'] ours ['quotes’', ',']; replace ref ['quotes‘.'] ours ['quotes‘', '.']

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1902659, 3980, 3362, 3090, 2735, 2742, 2455, 2303, 1980, 1750, 1661, 1718, 1445, 1259, 1221, 4456]`; ink px ref/ours 14217/18401 (ratio 1.2943); SSIM blocks <0.9: 1549/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.79, -0.15] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 2.187, differing 0.021256, SSIM₈ 0.968 (raw 2.3184, 0.021807, 0.9657)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9452→0.9489 / 3.7055→3.4955; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `portion` dx 0.05 dy 0.67; `serving,` dx 0.04 dy 0.67; `½` dx 0.04 dy 0.67
- word-sequence differences: delete ref ['\uffff'] ours []

### 11-nested-lists — lualatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920945, 1355, 1097, 945, 846, 1093, 822, 796, 797, 678, 928, 902, 872, 877, 856, 5007]`; ink px ref/ours 6059/5958 (ratio 0.9833); SSIM blocks <0.9: 999/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [18.0, -49.0] pt by ink-projection correlation (centroid estimate [101.05, -62.24] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.4605, differing 0.010223, SSIM₈ 0.9709 (raw 1.4653, 0.010385, 0.9696)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9514→0.9548 / 2.342→2.2656; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4507→0.3438 / 22.2877→28.6457 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `numbered` dx 307.89 dy -72.76; `First` dx 307.79 dy -72.76; `list.` dx 272.83 dy -126.61
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921458, 1298, 1149, 1072, 947, 989, 949, 1109, 1126, 807, 943, 852, 785, 769, 786, 3777]`; ink px ref/ours 6059/4773 (ratio 0.7878); SSIM blocks <0.9: 963/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-18.0, -20.0] pt by ink-projection correlation (centroid estimate [-21.05, -27.31] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2704, differing 0.009522, SSIM₈ 0.9742 (raw 1.3325, 0.010005, 0.9699)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.952→0.9588 / 2.1298→2.0305; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3907→0.2262 / 25.8273→35.4471 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `list.` dx 4.92 dy -54.03; `the` dx 2.35 dy -54.03; `After` dx 0 dy -54.03

### 11-nested-lists — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920575, 1363, 1139, 1084, 981, 933, 934, 877, 847, 877, 880, 866, 865, 812, 813, 4970]`; ink px ref/ours 6059/6137 (ratio 1.0129); SSIM blocks <0.9: 942/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-22.5, -8.0] pt by ink-projection correlation (centroid estimate [-26.07, -9.4] pt); confidence moderate (shift explains 24% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.122, differing 0.008873, SSIM₈ 0.98 (raw 1.4714, 0.010489, 0.9699)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9519→0.968 / 2.3517→1.7932; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.2153→0.259 / 35.1216→36.3028 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `First` dx -44.19 dy -11.56; `numbered` dx -44.09 dy -11.56; `Second` dx -44.19 dy -10.59

### 11-nested-lists — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920569, 1409, 1052, 1021, 967, 943, 901, 969, 899, 808, 906, 886, 798, 866, 860, 4962]`; ink px ref/ours 6059/6035 (ratio 0.996); SSIM blocks <0.9: 933/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-16.5, -20.0] pt by ink-projection correlation (centroid estimate [-18.77, -27.65] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3614, differing 0.009822, SSIM₈ 0.9752 (raw 1.4785, 0.010437, 0.9703)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9525→0.9604 / 2.3631→2.1759; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3714→0.2291 / 28.2012→39.4119 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `list.` dx 4.92 dy -54.39; `the` dx 2.35 dy -54.39; `After` dx 0 dy -54.39

### 11-nested-lists — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921444, 1232, 1126, 1001, 890, 1148, 910, 879, 962, 748, 869, 1022, 949, 799, 793, 4044]`; ink px ref/ours 4803/5958 (ratio 1.2405); SSIM blocks <0.9: 1044/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.0, -49.0] pt by ink-projection correlation (centroid estimate [99.1, -62.16] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3067, differing 0.00967, SSIM₈ 0.9711 (raw 1.3675, 0.010034, 0.9687)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.95→0.9538 / 2.1857→2.0885; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6066→0.4338 / 14.6679→25.5897 [109.1,139.7–273.7,184 pt]
- largest word displacements (pt): `First` dx 307.79 dy -73.53; `numbered` dx 304.29 dy -73.53; `After` dx 272.65 dy -127.37
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923291, 1130, 1092, 1037, 856, 953, 884, 1106, 1158, 794, 790, 885, 802, 671, 662, 2705]`; ink px ref/ours 4803/4773 (ratio 0.9938); SSIM blocks <0.9: 926/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-17.0, -20.0] pt by ink-projection correlation (centroid estimate [-22.99, -27.23] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9509, differing 0.008459, SSIM₈ 0.9783 (raw 1.1456, 0.009203, 0.9705)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9528→0.9652 / 1.831→1.5198; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4656→0.3814 / 21.2247→28.6817 [109.1,139.7–273.7,184 pt]
- largest word displacements (pt): `list.` dx 0.01 dy -54.79; `After` dx 0 dy -54.79; `the` dx 0.0 dy -54.79

### 11-nested-lists — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921056, 1232, 1170, 1162, 1016, 1005, 1027, 957, 1021, 942, 819, 982, 916, 731, 758, 4022]`; ink px ref/ours 4803/6137 (ratio 1.2777); SSIM blocks <0.9: 992/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-30.5, -8.0] pt by ink-projection correlation (centroid estimate [-28.01, -9.32] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2508, differing 0.009374, SSIM₈ 0.9755 (raw 1.3743, 0.010157, 0.9689)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9504→0.9608 / 2.1965→1.9992; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3467→0.4901 / 30.8091→28.0342 [109.1,139.7–273.7,184 pt]
- largest word displacements (pt): `sub-item.` dx -51.0 dy -12.33; `sub-item.` dx -48.84 dy -11.36; `numbered` dx -47.69 dy -12.33

### 11-nested-lists — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921403, 1350, 1182, 1075, 1013, 1039, 951, 1018, 998, 876, 805, 972, 824, 745, 763, 3802]`; ink px ref/ours 4803/6035 (ratio 1.2565); SSIM blocks <0.9: 956/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-17.0, -20.0] pt by ink-projection correlation (centroid estimate [-20.71, -27.57] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1995, differing 0.009224, SSIM₈ 0.9761 (raw 1.3304, 0.009978, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9619 / 2.1264→1.9172; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4897→0.3922 / 22.1153→32.6997 [109.1,139.7–273.7,184 pt]
- largest word displacements (pt): `list.` dx 0.01 dy -55.15; `After` dx 0 dy -55.15; `the` dx 0.0 dy -55.15

### 11-nested-lists — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920969, 1439, 1023, 930, 793, 1108, 857, 760, 794, 725, 868, 911, 886, 817, 931, 5005]`; ink px ref/ours 6062/5958 (ratio 0.9828); SSIM blocks <0.9: 995/30294; [overlay](images/11-nested-lists/pdflatex-de1020c-export-p1-overlay.png) (42578 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-de1020c-export-p1-heatmap.png) (43654 B, ÷2)
  - registration error (diagnostic): global shift [19.0, -49.0] pt by ink-projection correlation (centroid estimate [102.36, -62.51] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.4558, differing 0.010208, SSIM₈ 0.9711 (raw 1.4657, 0.01038, 0.9698)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9518→0.9553 / 2.3427→2.2545; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.506→0.3995 / 22.5175→28.0218 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `numbered` dx 309.05 dy -72.76; `First` dx 308.96 dy -72.76; `list.` dx 272.83 dy -126.61
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921432, 1339, 1098, 1041, 905, 1069, 972, 1121, 1068, 887, 853, 856, 798, 729, 851, 3797]`; ink px ref/ours 6062/4773 (ratio 0.7874); SSIM blocks <0.9: 952/30294; [overlay](images/11-nested-lists/pdflatex-exact-export-p1-overlay.png) (45367 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-exact-export-p1-heatmap.png) (45296 B, ÷2)
  - registration error (diagnostic): global shift [-17.0, -20.0] pt by ink-projection correlation (centroid estimate [-19.74, -27.58] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.262, differing 0.009493, SSIM₈ 0.9748 (raw 1.3374, 0.010017, 0.9701)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9522→0.9596 / 2.1375→2.0171; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4388→0.2781 / 26.5776→35.4723 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `list.` dx 4.91 dy -54.03; `the` dx 2.35 dy -54.03; `After` dx 0 dy -54.03

### 11-nested-lists — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920680, 1452, 1071, 1083, 924, 959, 967, 834, 829, 919, 814, 882, 860, 749, 888, 4905]`; ink px ref/ours 6062/6137 (ratio 1.0124); SSIM blocks <0.9: 935/30294; [overlay](images/11-nested-lists/pdflatex-main-export-p1-overlay.png) (43218 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-main-export-p1-heatmap.png) (43956 B, ÷2)
  - registration error (diagnostic): global shift [-22.0, -8.0] pt by ink-projection correlation (centroid estimate [-24.76, -9.67] pt); confidence moderate (shift explains 19% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1824, differing 0.009146, SSIM₈ 0.979 (raw 1.4604, 0.010455, 0.9704)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9527→0.9664 / 2.3342→1.8898; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.2574→0.3573 / 34.7602→36.4381 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `First` dx -43.02 dy -11.56; `numbered` dx -42.93 dy -11.56; `Second` dx -43.02 dy -10.59

### 11-nested-lists — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920964, 1465, 977, 937, 878, 1008, 886, 891, 876, 870, 850, 902, 789, 799, 922, 4802]`; ink px ref/ours 6062/6035 (ratio 0.9955); SSIM blocks <0.9: 916/30294; [overlay](images/11-nested-lists/pdflatex-pipeline-export-p1-overlay.png) (43612 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-pipeline-export-p1-heatmap.png) (43651 B, ÷2)
  - registration error (diagnostic): global shift [-16.0, -20.0] pt by ink-projection correlation (centroid estimate [-17.46, -27.92] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.366, differing 0.009824, SSIM₈ 0.9757 (raw 1.4479, 0.010304, 0.9709)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9535→0.9611 / 2.3142→2.1833; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4616→0.2936 / 26.3978→38.5864 [108.2,145.1–264.8,183.2 pt]
- largest word displacements (pt): `list.` dx 4.91 dy -54.39; `the` dx 2.35 dy -54.39; `After` dx 0 dy -54.39

### 11-nested-lists — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921429, 1208, 1106, 1054, 881, 1201, 884, 878, 1059, 686, 919, 983, 802, 823, 761, 4142]`; ink px ref/ours 4790/5958 (ratio 1.2438); SSIM blocks <0.9: 1048/30294; [overlay](images/11-nested-lists/pdflatex-lm-de1020c-export-p1-overlay.png) (43045 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-de1020c-export-p1-heatmap.png) (43187 B, ÷2)
  - registration error (diagnostic): global shift [23.5, -49.0] pt by ink-projection correlation (centroid estimate [101.54, -62.0] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3216, differing 0.009723, SSIM₈ 0.9706 (raw 1.3674, 0.010055, 0.9687)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.95→0.9547 / 2.1855→2.0271; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6045→0.4278 / 14.6672→25.9327 [108.1,139.7–272.5,184 pt]
- largest word displacements (pt): `First` dx 308.96 dy -73.53; `numbered` dx 305.46 dy -73.53; `After` dx 272.65 dy -127.37
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923362, 1065, 1100, 1085, 830, 999, 864, 1136, 1254, 736, 827, 847, 670, 666, 636, 2739]`; ink px ref/ours 4790/4773 (ratio 0.9965); SSIM blocks <0.9: 930/30294; [overlay](images/11-nested-lists/pdflatex-lm-exact-export-p1-overlay.png) (45461 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-exact-export-p1-heatmap.png) (43569 B, ÷2)
  - registration error (diagnostic): global shift [-16.5, -20.0] pt by ink-projection correlation (centroid estimate [-20.56, -27.07] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9949, differing 0.008625, SSIM₈ 0.9774 (raw 1.136, 0.009072, 0.9705)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9529→0.9639 / 1.8157→1.5901; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4762→0.364 / 20.7603→29.269 [108.1,139.7–272.5,184 pt]
- largest word displacements (pt): `After` dx 0 dy -54.79; `the` dx 0.0 dy -54.79; `list.` dx -0.0 dy -54.79

### 11-nested-lists — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921052, 1206, 1155, 1188, 1007, 1051, 998, 964, 1103, 884, 858, 948, 784, 760, 727, 4131]`; ink px ref/ours 4790/6137 (ratio 1.2812); SSIM blocks <0.9: 994/30294; [overlay](images/11-nested-lists/pdflatex-lm-main-export-p1-overlay.png) (43738 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-main-export-p1-heatmap.png) (43738 B, ÷2)
  - registration error (diagnostic): global shift [-30.0, -8.0] pt by ink-projection correlation (centroid estimate [-25.58, -9.16] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2546, differing 0.009404, SSIM₈ 0.9752 (raw 1.3761, 0.010178, 0.969)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9504→0.9604 / 2.1994→2.0053; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3438→0.4843 / 30.9535→28.3211 [108.1,139.7–272.5,184 pt]
- largest word displacements (pt): `sub-item.` dx -49.84 dy -12.33; `sub-item.` dx -47.67 dy -11.36; `numbered` dx -46.52 dy -12.33

### 11-nested-lists — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921343, 1306, 1155, 1137, 1008, 1088, 938, 1022, 1096, 817, 863, 936, 693, 746, 741, 3927]`; ink px ref/ours 4790/6035 (ratio 1.2599); SSIM blocks <0.9: 960/30294; [overlay](images/11-nested-lists/pdflatex-lm-pipeline-export-p1-overlay.png) (43938 B, ÷2), [heatmap](images/11-nested-lists/pdflatex-lm-pipeline-export-p1-heatmap.png) (43084 B, ÷2)
  - registration error (diagnostic): global shift [-17.0, -20.0] pt by ink-projection correlation (centroid estimate [-18.28, -27.41] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1999, differing 0.009215, SSIM₈ 0.976 (raw 1.3364, 0.010008, 0.9701)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9522→0.9616 / 2.1359→1.9178; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.47→0.3837 / 22.7427→33.2089 [108.1,139.7–272.5,184 pt]
- largest word displacements (pt): `After` dx 0 dy -55.15; `the` dx 0.0 dy -55.15; `list.` dx -0.0 dy -55.15

### 11-nested-lists — xelatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920921, 1399, 1106, 932, 854, 1068, 813, 810, 810, 650, 914, 907, 884, 897, 861, 4990]`; ink px ref/ours 6057/5958 (ratio 0.9837); SSIM blocks <0.9: 999/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [18.0, -49.0] pt by ink-projection correlation (centroid estimate [101.1, -62.21] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.46, differing 0.010222, SSIM₈ 0.9709 (raw 1.4648, 0.010387, 0.9696)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9514→0.9548 / 2.3412→2.2648; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4507→0.3437 / 22.2161→28.5578 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `numbered` dx 307.89 dy -72.76; `First` dx 307.79 dy -72.76; `list.` dx 272.83 dy -126.61
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921435, 1320, 1165, 1070, 958, 970, 933, 1126, 1122, 796, 929, 857, 794, 788, 793, 3760]`; ink px ref/ours 6057/4773 (ratio 0.788); SSIM blocks <0.9: 963/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-18.0, -20.0] pt by ink-projection correlation (centroid estimate [-21.0, -27.28] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.271, differing 0.009523, SSIM₈ 0.9742 (raw 1.3324, 0.010003, 0.9699)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.952→0.9588 / 2.1295→2.0314; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3907→0.2261 / 25.7444→35.3285 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `list.` dx 4.91 dy -54.03; `the` dx 2.35 dy -54.03; `After` dx 0 dy -54.03

### 11-nested-lists — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920552, 1403, 1152, 1070, 987, 916, 919, 890, 857, 852, 866, 874, 876, 833, 814, 4955]`; ink px ref/ours 6057/6137 (ratio 1.0132); SSIM blocks <0.9: 942/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-22.5, -8.0] pt by ink-projection correlation (centroid estimate [-26.02, -9.37] pt); confidence moderate (shift explains 24% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1192, differing 0.008866, SSIM₈ 0.98 (raw 1.4708, 0.01049, 0.9699)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9519→0.968 / 2.3507→1.7888; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.2152→0.2588 / 35.0092→36.1854 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `First` dx -44.19 dy -11.56; `numbered` dx -44.09 dy -11.56; `Second` dx -44.19 dy -10.59

### 11-nested-lists — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920545, 1431, 1069, 1017, 979, 926, 887, 988, 905, 779, 891, 893, 808, 882, 870, 4946]`; ink px ref/ours 6057/6035 (ratio 0.9964); SSIM blocks <0.9: 933/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-16.5, -20.0] pt by ink-projection correlation (centroid estimate [-18.72, -27.62] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3609, differing 0.009823, SSIM₈ 0.9752 (raw 1.4783, 0.010438, 0.9703)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9525→0.9604 / 2.3628→2.1751; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3713→0.229 / 28.1108→39.2887 [109.2,144.9–266,183.2 pt]
- largest word displacements (pt): `list.` dx 4.91 dy -54.39; `the` dx 2.35 dy -54.39; `After` dx 0 dy -54.39

### 11-nested-lists — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (7): warning: environment 'itemize' is not implemented; its body is typeset as plain text; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \item is not supported by this compiler version; unrestricted TeX math mode is not implemented; warning: environment 'enumerate' is not implemented; its body is typeset as plain text …
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921432, 1237, 1125, 1009, 893, 1121, 930, 896, 951, 735, 882, 1028, 939, 802, 786, 4050]`; ink px ref/ours 4790/5958 (ratio 1.2438); SSIM blocks <0.9: 1044/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.0, -49.0] pt by ink-projection correlation (centroid estimate [98.92, -62.23] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3068, differing 0.009665, SSIM₈ 0.9711 (raw 1.3675, 0.010029, 0.9687)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.95→0.9538 / 2.1857→2.0886; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5188→0.4109 / 16.5479→24.3869 [109.1,143.5–273.7,182.9 pt]
- largest word displacements (pt): `First` dx 307.8 dy -72.5; `numbered` dx 304.3 dy -72.5; `After` dx 272.65 dy -126.34
- word-sequence differences: delete ref ['•'] ours []; delete ref ['•'] ours []; delete ref ['1.'] ours []

### 11-nested-lists — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923278, 1137, 1094, 1045, 854, 941, 885, 1129, 1142, 799, 788, 894, 792, 673, 649, 2716]`; ink px ref/ours 4790/4773 (ratio 0.9965); SSIM blocks <0.9: 926/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-17.0, -20.0] pt by ink-projection correlation (centroid estimate [-23.17, -27.3] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9497, differing 0.008453, SSIM₈ 0.9783 (raw 1.1456, 0.009199, 0.9705)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9528→0.9653 / 1.831→1.518; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4265→0.3357 / 21.2356→29.8605 [109.1,143.5–273.7,182.9 pt]
- largest word displacements (pt): `Second` dx -41.87 dy -33.84; `numbered` dx -41.87 dy -33.84; `sub-item.` dx -41.86 dy -33.84

### 11-nested-lists — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921045, 1235, 1170, 1168, 1018, 980, 1046, 973, 1012, 932, 832, 983, 909, 734, 749, 4030]`; ink px ref/ours 4790/6137 (ratio 1.2812); SSIM blocks <0.9: 992/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-30.5, -8.0] pt by ink-projection correlation (centroid estimate [-28.19, -9.39] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2509, differing 0.009371, SSIM₈ 0.9755 (raw 1.3743, 0.010152, 0.9689)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9504→0.9608 / 2.1966→1.9993; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3001→0.3765 / 29.4433→31.6327 [109.1,143.5–273.7,182.9 pt]
- largest word displacements (pt): `sub-item.` dx -50.99 dy -11.3; `sub-item.` dx -48.83 dy -10.33; `numbered` dx -47.68 dy -11.3

### 11-nested-lists — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921390, 1362, 1170, 1093, 1016, 1011, 961, 1040, 986, 872, 810, 976, 817, 750, 748, 3814]`; ink px ref/ours 4790/6035 (ratio 1.2599); SSIM blocks <0.9: 956/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-17.0, -20.0] pt by ink-projection correlation (centroid estimate [-20.89, -27.64] pt); confidence moderate (shift explains 10% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1997, differing 0.00922, SSIM₈ 0.9761 (raw 1.3305, 0.009973, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9619 / 2.1265→1.9175; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4494→0.3419 / 21.5144→33.5966 [109.1,143.5–273.7,182.9 pt]
- largest word displacements (pt): `list.` dx 0.01 dy -54.13; `After` dx 0 dy -54.13; `the` dx 0.0 dy -54.13

### 12-justified-paragraphs — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1744402, 14700, 12306, 11286, 10602, 10260, 11020, 9359, 9682, 10331, 9550, 9486, 9124, 9371, 9695, 47642]`; ink px ref/ours 67719/67475 (ratio 0.9964); SSIM blocks <0.9: 8260/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.58, 32.29] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.3872, differing 0.111127, SSIM₈ 0.7397 (raw 15.4038, 0.111111, 0.7396)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5838→0.584 / 24.6198→24.5932; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 75.37; `branch` dx -435.47 dy 55.2; `branch` dx -435.47 dy 35.03

### 12-justified-paragraphs — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1772971, 14433, 12189, 11260, 10685, 9675, 10345, 10216, 10045, 8836, 7958, 7850, 7511, 7622, 7256, 29964]`; ink px ref/ours 67719/51544 (ratio 0.7611); SSIM blocks <0.9: 7323/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 29.0] pt by ink-projection correlation (centroid estimate [-1.3, 25.16] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 12.004, differing 0.095629, SSIM₈ 0.7893 (raw 12.0437, 0.095941, 0.787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6599→0.6899 / 19.2482→18.1146; header-band 1.0→0.8177 / 0.0→7.3666; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 58.55; `oak` dx -415.78 dy 44.1; `branch` dx -413.61 dy 58.55

### 12-justified-paragraphs — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1749889, 14370, 12460, 11361, 10312, 10070, 10547, 9139, 9936, 9787, 9317, 9183, 9050, 9324, 8981, 45090]`; ink px ref/ours 67719/67696 (ratio 0.9997); SSIM blocks <0.9: 7586/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-3.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.96, 9.73] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 14.8082, differing 0.108048, SSIM₈ 0.7605 (raw 14.8416, 0.108175, 0.7604)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6171→0.6177 / 23.721→23.6417; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 32.17; `branch` dx -435.47 dy 26.4; `branch` dx -435.47 dy 20.63

### 12-justified-paragraphs — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1768307, 14443, 12099, 10825, 10344, 9097, 9288, 9110, 8863, 8802, 8185, 8466, 7793, 8008, 7969, 37217]`; ink px ref/ours 67719/67211 (ratio 0.9925); SSIM blocks <0.9: 7218/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 14.5] pt REJECTED: applying it gives mean|Δ| 12.9698, not lower; centroid estimate [-2.35, 25.22] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 12.9681, differing 0.098738, SSIM₈ 0.7908 (raw 12.9681, 0.098738, 0.7908)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.666→0.666 / 20.7248→20.7248; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 58.19; `oak` dx -415.78 dy 43.74; `branch` dx -413.61 dy 58.19

### 12-justified-paragraphs — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755357, 14133, 12350, 11646, 11189, 10634, 11430, 11055, 10405, 10568, 8991, 8890, 8563, 8103, 8348, 37154]`; ink px ref/ours 51487/67475 (ratio 1.3105); SSIM blocks <0.9: 7892/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-8.5, 2.5] pt by ink-projection correlation (centroid estimate [-6.5, 6.98] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7712, differing 0.105152, SSIM₈ 0.7503 (raw 13.7995, 0.105157, 0.7503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6011→0.6011 / 22.0545→22.0094; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -14.75; `oak` dx 426.94 dy -9.02; `oak` dx 426.94 dy -3.3

### 12-justified-paragraphs — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 51487/51544 (ratio 1.0011); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.22, -0.14] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1236, differing 0.03615, SSIM₈ 0.9999 (raw 0.1236, 0.03615, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.1975→0.1975; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `old` dx 0.03 dy -0.0; `old` dx 0.03 dy -0.0; `old` dx 0.03 dy -0.0

### 12-justified-paragraphs — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755413, 14403, 12338, 11855, 11064, 10774, 11057, 10981, 10835, 10360, 8867, 8620, 8720, 8243, 7948, 37338]`; ink px ref/ours 51487/67696 (ratio 1.3148); SSIM blocks <0.9: 8020/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, -26.0] pt by ink-projection correlation (centroid estimate [-0.88, -15.57] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7623, differing 0.105511, SSIM₈ 0.7495 (raw 13.7762, 0.105431, 0.7469)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5959→0.6 / 22.0172→21.9931; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -40.77; `oak` dx 425.86 dy -32.1; `oak` dx 425.86 dy -23.42

### 12-justified-paragraphs — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1812650, 14416, 11616, 11162, 11337, 9571, 8950, 8353, 7206, 6926, 5224, 5080, 4804, 4450, 3682, 13389]`; ink px ref/ours 51487/67211 (ratio 1.3054); SSIM blocks <0.9: 5380/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.27, -0.08] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.7969, differing 0.076623, SSIM₈ 0.8919 (raw 7.7969, 0.076623, 0.8919)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8275→0.8275 / 12.46→12.46; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `old` dx 0.03 dy -0.36; `old` dx 0.03 dy -0.36; `old` dx 0.03 dy -0.36

### 12-justified-paragraphs — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1744770, 14653, 12288, 11028, 10978, 10548, 10474, 9663, 9503, 10351, 9533, 9493, 9214, 8786, 9639, 47895]`; ink px ref/ours 67213/67475 (ratio 1.0039); SSIM blocks <0.9: 8223/30294; [overlay](images/12-justified-paragraphs/pdflatex-de1020c-export-p1-overlay.png) (82936 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-de1020c-export-p1-heatmap.png) (63610 B, ÷4)
  - registration error (diagnostic): global shift [-3.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.27, 32.27] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.3659, differing 0.110805, SSIM₈ 0.7414 (raw 15.3708, 0.11076, 0.7415)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5868→0.5868 / 24.5671→24.5592; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 75.37; `branch` dx -435.47 dy 55.2; `branch` dx -435.47 dy 35.03

### 12-justified-paragraphs — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1773408, 14291, 12176, 10835, 10960, 9947, 9701, 10443, 9936, 8884, 7840, 7887, 7596, 7212, 7407, 30293]`; ink px ref/ours 67213/51544 (ratio 0.7669); SSIM blocks <0.9: 7285/30294; [overlay](images/12-justified-paragraphs/pdflatex-exact-export-p1-overlay.png) (82015 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-exact-export-p1-heatmap.png) (56774 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 29.0] pt by ink-projection correlation (centroid estimate [-1.0, 25.14] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 11.9747, differing 0.095251, SSIM₈ 0.7908 (raw 12.0428, 0.095602, 0.788)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6615→0.6924 / 19.2468→18.0677; header-band 1.0→0.8177 / 0.0→7.3666; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 58.55; `oak` dx -415.72 dy 44.1; `branch` dx -413.61 dy 58.55

### 12-justified-paragraphs — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1749913, 14175, 12238, 11252, 10474, 10395, 9908, 9502, 9515, 10002, 9354, 9262, 9131, 8949, 9210, 45536]`; ink px ref/ours 67213/67696 (ratio 1.0072); SSIM blocks <0.9: 7550/30294; [overlay](images/12-justified-paragraphs/pdflatex-main-export-p1-overlay.png) (83287 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-main-export-p1-heatmap.png) (60517 B, ÷4)
  - registration error (diagnostic): global shift [-4.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.66, 9.72] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 14.8302, differing 0.107799, SSIM₈ 0.7612 (raw 14.8933, 0.107948, 0.7603)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.617→0.6189 / 23.8036→23.6765; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 32.17; `branch` dx -435.47 dy 26.4; `branch` dx -435.47 dy 20.63

### 12-justified-paragraphs — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1768201, 14181, 12058, 10574, 10629, 9233, 9081, 9151, 8588, 8646, 8335, 8721, 7905, 7658, 8236, 37619]`; ink px ref/ours 67213/67211 (ratio 1.0); SSIM blocks <0.9: 7184/30294; [overlay](images/12-justified-paragraphs/pdflatex-pipeline-export-p1-overlay.png) (82165 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-pipeline-export-p1-heatmap.png) (57606 B, ÷4)
  - registration error (diagnostic): global shift [1.0, 14.5] pt by ink-projection correlation (centroid estimate [-2.05, 25.2] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 12.9218, differing 0.098411, SSIM₈ 0.7936 (raw 13.0291, 0.098611, 0.7911)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6665→0.6845 / 20.8222→19.959; header-band 1.0→0.9141 / 0.0→4.6024; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 58.19; `oak` dx -415.72 dy 43.74; `branch` dx -413.61 dy 58.19

### 12-justified-paragraphs — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755333, 14131, 12262, 11805, 11161, 10699, 11419, 10985, 10442, 10508, 9029, 8874, 8613, 7950, 8550, 37055]`; ink px ref/ours 51535/67475 (ratio 1.3093); SSIM blocks <0.9: 7894/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-de1020c-export-p1-overlay.png) (84539 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-de1020c-export-p1-heatmap.png) (62693 B, ÷4)
  - registration error (diagnostic): global shift [-8.5, 2.5] pt by ink-projection correlation (centroid estimate [-6.35, 7.1] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7701, differing 0.105126, SSIM₈ 0.7503 (raw 13.7992, 0.105148, 0.7503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6012→0.6012 / 22.0542→22.0076; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.0; `oak` dx 426.94 dy 3.45

### 12-justified-paragraphs — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 51535/51544 (ratio 1.0002); SSIM blocks <0.9: 0/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-exact-export-p1-overlay.png) (56965 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-exact-export-p1-heatmap.png) (44905 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.08, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0418, differing 0.023504, SSIM₈ 1.0 (raw 0.0418, 0.023504, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0668→0.0668; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `quick` dx 0.01 dy 1.03; `old` dx -0.01 dy 1.03; `into` dx -0.01 dy 1.03

### 12-justified-paragraphs — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755374, 14451, 12262, 12007, 11000, 10784, 11079, 10904, 10909, 10250, 8963, 8590, 8773, 8067, 8103, 37300]`; ink px ref/ours 51535/67696 (ratio 1.3136); SSIM blocks <0.9: 8024/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-main-export-p1-overlay.png) (83704 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-main-export-p1-heatmap.png) (62796 B, ÷4)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-0.73, -15.45] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.6913, differing 0.105143, SSIM₈ 0.7482 (raw 13.7769, 0.105422, 0.7469)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5958→0.5979 / 22.0181→21.8797; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -39.75; `oak` dx 425.86 dy -31.07; `oak` dx 425.86 dy -22.4

### 12-justified-paragraphs — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1812494, 14322, 11585, 11596, 10906, 9834, 8829, 8463, 7266, 6714, 5341, 5030, 4797, 4375, 3896, 13368]`; ink px ref/ours 51535/67211 (ratio 1.3042); SSIM blocks <0.9: 5394/30294; [overlay](images/12-justified-paragraphs/pdflatex-lm-pipeline-export-p1-overlay.png) (80218 B, ÷4), [heatmap](images/12-justified-paragraphs/pdflatex-lm-pipeline-export-p1-heatmap.png) (49959 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.13, 0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.8132, differing 0.076672, SSIM₈ 0.8917 (raw 7.8132, 0.076672, 0.8917)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8271→0.8271 / 12.4861→12.4861; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `quick` dx 0.01 dy 0.67; `into` dx -0.01 dy 0.67; `the` dx 0.01 dy 0.67

### 12-justified-paragraphs — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1744655, 14295, 12346, 11375, 10598, 10442, 10890, 9620, 9798, 10256, 9531, 9562, 9258, 9139, 9507, 47544]`; ink px ref/ours 67565/67475 (ratio 0.9987); SSIM blocks <0.9: 8255/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-3.0, 0.0] pt REJECTED: applying it gives mean|Δ| 15.3833, not lower; centroid estimate [-6.26, 32.52] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.3809, differing 0.111072, SSIM₈ 0.7404 (raw 15.3809, 0.111072, 0.7404)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5852→0.5852 / 24.5832→24.5832; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 75.37; `branch` dx -435.52 dy 55.2; `branch` dx -435.52 dy 35.03

### 12-justified-paragraphs — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1772757, 14098, 12353, 11294, 10688, 10005, 10130, 10456, 9907, 8833, 7933, 7935, 7629, 7473, 7160, 30165]`; ink px ref/ours 67565/51544 (ratio 0.7629); SSIM blocks <0.9: 7336/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 29.0] pt by ink-projection correlation (centroid estimate [0.01, 25.39] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 12.0273, differing 0.095735, SSIM₈ 0.789 (raw 12.0683, 0.096081, 0.7868)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6594→0.6894 / 19.2875→18.1517; header-band 1.0→0.8177 / 0.0→7.3666; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 58.55; `oak` dx -415.83 dy 44.1; `branch` dx -413.66 dy 58.55

### 12-justified-paragraphs — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1749319, 13924, 12542, 11373, 10258, 10254, 10377, 9353, 9938, 9889, 9331, 9231, 9181, 9122, 9123, 45601]`; ink px ref/ours 67565/67696 (ratio 1.0019); SSIM blocks <0.9: 7582/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.64, 9.96] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 14.9399, differing 0.108468, SSIM₈ 0.7591 (raw 14.9399, 0.108468, 0.7591)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.615→0.615 / 23.8781→23.8781; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.52 dy 32.17; `branch` dx -435.52 dy 26.4; `branch` dx -435.52 dy 20.63

### 12-justified-paragraphs — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1767728, 14063, 12284, 10871, 10064, 9392, 9235, 9348, 8874, 8860, 8082, 8615, 7900, 7879, 8002, 37619]`; ink px ref/ours 67565/67211 (ratio 0.9948); SSIM blocks <0.9: 7229/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 14.5] pt by ink-projection correlation (centroid estimate [-1.03, 25.45] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 12.9774, differing 0.098736, SSIM₈ 0.7921 (raw 13.0463, 0.098977, 0.7899)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6646→0.6812 / 20.8498→20.0656; header-band 1.0→0.9144 / 0.0→4.6024; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 58.19; `oak` dx -415.83 dy 43.74; `branch` dx -413.66 dy 58.19

### 12-justified-paragraphs — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755364, 14133, 12378, 11631, 11131, 10674, 11450, 11022, 10428, 10570, 8986, 8886, 8529, 8142, 8340, 37152]`; ink px ref/ours 51522/67475 (ratio 1.3096); SSIM blocks <0.9: 7895/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-8.5, 2.5] pt by ink-projection correlation (centroid estimate [-6.61, 7.02] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7708, differing 0.105112, SSIM₈ 0.7503 (raw 13.7994, 0.105112, 0.7503)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6011→0.6011 / 22.0544→22.0087; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -13.72; `oak` dx 426.94 dy -8.0; `oak` dx 426.94 dy 3.45

### 12-justified-paragraphs — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 51522/51544 (ratio 1.0004); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.33, -0.11] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1288, differing 0.036282, SSIM₈ 0.9999 (raw 0.1288, 0.036282, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9999→0.9999 / 0.2058→0.2058; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `over` dx 0.02 dy 1.03; `patient` dx 0.02 dy 1.03; `quick` dx 0.02 dy 1.03

### 12-justified-paragraphs — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1755442, 14379, 12354, 11848, 11010, 10815, 11077, 10918, 10881, 10353, 8886, 8602, 8701, 8265, 7942, 37343]`; ink px ref/ours 51522/67696 (ratio 1.3139); SSIM blocks <0.9: 8024/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, -26.0] pt by ink-projection correlation (centroid estimate [-0.99, -15.54] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.7693, differing 0.10552, SSIM₈ 0.7493 (raw 13.7765, 0.105388, 0.7469)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.5958→0.5997 / 22.0176→22.0028; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -39.75; `oak` dx 425.86 dy -31.07; `oak` dx 425.86 dy -22.4

### 12-justified-paragraphs — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1812654, 14422, 11613, 11139, 11258, 9661, 8948, 8384, 7164, 6955, 5230, 5055, 4787, 4452, 3706, 13388]`; ink px ref/ours 51522/67211 (ratio 1.3045); SSIM blocks <0.9: 5379/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.38, -0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.797, differing 0.07658, SSIM₈ 0.8919 (raw 7.797, 0.07658, 0.8919)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8275→0.8275 / 12.4601→12.4601; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `over` dx 0.02 dy 0.67; `patient` dx 0.02 dy 0.67; `quick` dx 0.02 dy 0.67

### 13-math-display-rich — lualatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1932109, 636, 516, 481, 385, 417, 581, 452, 336, 325, 292, 257, 284, 309, 277, 1159]`; ink px ref/ours 2479/2605 (ratio 1.0508); SSIM blocks <0.9: 323/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [-0.19, 2.65] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4325, differing 0.003871, SSIM₈ 0.9914 (raw 0.4724, 0.004014, 0.9903)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9845→0.9863 / 0.755→0.6913; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.402→0.3954 / 25.1108→24.6697 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -66.25 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2; Δx 18.75 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `display:` dx 0.1 dy -10.41; `Rich` dx 0 dy -10.41; `√x` dx -7.41 dy 4.97
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — lualatex vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [236.87286281585693, 703.4119424819946, 6.652087211608887, 0.47818756103515625] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931387, 606, 494, 492, 430, 459, 734, 468, 509, 397, 391, 356, 373, 332, 322, 1066]`; ink px ref/ours 2479/2390 (ratio 0.9641); SSIM blocks <0.9: 387/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [20.72, -1.13] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5152, differing 0.004281, SSIM₈ 0.9887 (raw 0.5252, 0.004347, 0.9884)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9815→0.9819 / 0.8395→0.8234; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3813→0.4626 / 22.6414→18.9083 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -84.25 Δy 6.5 len 31.5 vs 12.0, thickness px 1 vs 1; Δx 34.0 Δy -6.75 len 32.5 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `1` dx -29.78 dy -0.26; `0` dx -29.78 dy -0.26; `√x` dx -29.78 dy -0.26
- word-sequence differences: insert ref [] ours ['∫']; insert ref [] ours ['∞', '∑']; replace ref ['dx=', '∞', 'k=0', '(−1)k'] ours [',', 'dx', '=', '\\lef', 't(', '(-', '1)k']

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
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [236.873, 703.412, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931439, 629, 492, 445, 413, 451, 647, 417, 373, 336, 363, 342, 365, 356, 364, 1384]`; ink px ref/ours 2479/2847 (ratio 1.1484); SSIM blocks <0.9: 374/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [2.0, -0.5] pt REJECTED: applying it gives mean|Δ| 0.5514, not lower; centroid estimate [19.57, -1.34] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5457, differing 0.004309, SSIM₈ 0.9886 (raw 0.5457, 0.004309, 0.9886)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9817→0.9817 / 0.8722→0.8722; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3813→0.3813 / 23.3605→23.3605 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -83.75 Δy 6.5 len 32.5 vs 12.0, thickness px 1 vs 1; Δx 34.0 Δy -6.75 len 32.5 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `1` dx -29.78 dy 0.41; `+` dx -29.78 dy 0.41; `x2` dx -29.78 dy 0.41
- word-sequence differences: insert ref [] ours ['∫']; replace ref ['√x'] ours ['√', 'x']; replace ref ['dx='] ours [',', 'dx', '=']

### 13-math-display-rich — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931240, 712, 529, 432, 438, 438, 472, 553, 438, 442, 462, 316, 320, 342, 358, 1324]`; ink px ref/ours 2285/2605 (ratio 1.14); SSIM blocks <0.9: 363/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.0, 4.0] pt by ink-projection correlation (centroid estimate [-21.76, 3.05] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5477, differing 0.004333, SSIM₈ 0.9892 (raw 0.5515, 0.004325, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.982→0.9828 / 0.8815→0.875; header-band 1.0→0.9992 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4003→0.4082 / 24.7221→24.3784 [232.8,84.9–384.7,123.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -11.5 Δy 4.0 len 21.0 vs 31.0, thickness px 1 vs 1; Δx -16.75 Δy 4.0 len 21.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `display:` dx -1.48 dy -10.27; `Rich` dx 0 dy -10.27; `√x` dx -7.41 dy 5.11
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — lualatex-lm vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [236.87286281585693, 703.4119424819946, 6.652087211608887, 0.47818756103515625] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933416, 632, 508, 422, 404, 327, 389, 325, 330, 328, 329, 197, 221, 184, 223, 581]`; ink px ref/ours 2285/2390 (ratio 1.046); SSIM blocks <0.9: 298/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.84, -0.72] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3458, differing 0.003434, SSIM₈ 0.9917 (raw 0.3458, 0.003434, 0.9917)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9867→0.9867 / 0.5526→0.5526; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3996→0.3996 / 21.8619→21.8619 [232.8,84.9–384.7,123.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.75 Δy 0.0 len 31.5 vs 31.0, thickness px 1 vs 1; Δx -1.5 Δy 0.0 len 32.5 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `1` dx -29.78 dy -0.11; `0` dx -29.78 dy -0.11; `√x` dx -29.78 dy -0.11
- word-sequence differences: insert ref [] ours ['∫']; insert ref [] ours ['∞', '∑']; replace ref ['dx=', '∞', 'k=0', '(−1)k'] ours [',', 'dx', '=', '\\lef', 't(', '(-', '1)k']

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
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [236.873, 703.412, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931493, 714, 599, 468, 503, 413, 480, 477, 393, 387, 449, 304, 318, 321, 366, 1131]`; ink px ref/ours 2285/2847 (ratio 1.246); SSIM blocks <0.9: 373/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.99, -0.93] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4599, differing 0.003981, SSIM₈ 0.9903 (raw 0.5142, 0.004196, 0.9891)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9827→0.9845 / 0.8219→0.7351; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3953→0.4138 / 22.5882→22.6043 [232.8,84.9–384.7,123.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 0.0 len 32.5 vs 31.0, thickness px 1 vs 1; Δx -1.5 Δy 0.0 len 32.5 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `1` dx -29.78 dy 0.56; `+` dx -29.78 dy 0.56; `x2` dx -29.78 dy 0.56
- word-sequence differences: insert ref [] ours ['∫']; replace ref ['√x'] ours ['√', 'x']; replace ref ['dx='] ours [',', 'dx', '=']

### 13-math-display-rich — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931825, 678, 566, 390, 402, 560, 481, 438, 376, 334, 290, 257, 295, 302, 330, 1292]`; ink px ref/ours 2470/2605 (ratio 1.0547); SSIM blocks <0.9: 320/30294; [overlay](images/13-math-display-rich/pdflatex-de1020c-export-p1-overlay.png) (82867 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-de1020c-export-p1-heatmap.png) (81096 B, ÷1)
  - registration error (diagnostic): global shift [1.0, 3.5] pt by ink-projection correlation (centroid estimate [0.39, 2.75] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4808, differing 0.004039, SSIM₈ 0.9907 (raw 0.501, 0.004125, 0.9899)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9839→0.9853 / 0.8007→0.767; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4017→0.3792 / 25.1214→25.5101 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -66.25 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2; Δx 18.75 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `display:` dx 0.1 dy -10.42; `Rich` dx 0 dy -10.42; `√x` dx -7.41 dy 4.96
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — pdflatex vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [236.87286281585693, 703.4119424819946, 6.652087211608887, 0.47818756103515625] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931289, 609, 519, 468, 465, 523, 671, 476, 494, 412, 383, 356, 383, 318, 374, 1076]`; ink px ref/ours 2470/2390 (ratio 0.9676); SSIM blocks <0.9: 386/30294; [overlay](images/13-math-display-rich/pdflatex-exact-export-p1-overlay.png) (85204 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-exact-export-p1-heatmap.png) (83757 B, ÷1)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [21.31, -1.02] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5136, differing 0.00427, SSIM₈ 0.9887 (raw 0.5343, 0.00436, 0.9882)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9812→0.9819 / 0.854→0.8209; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3802→0.4609 / 22.6726→18.9598 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -84.25 Δy 6.5 len 31.5 vs 12.0, thickness px 1 vs 1; Δx 34.0 Δy -6.75 len 32.5 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `1` dx -29.78 dy -0.26; `0` dx -29.78 dy -0.26; `√x` dx -29.78 dy -0.26
- word-sequence differences: insert ref [] ours ['∫']; insert ref [] ours ['∞', '∑']; replace ref ['dx=', '∞', 'k=0', '(−1)k'] ours [',', 'dx', '=', '\\lef', 't(', '(-', '1)k']

### 13-math-display-rich — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (3): error: \left is not supported in math mode; error: \right is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [241.73, 685.5, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931869, 761, 582, 393, 427, 567, 527, 377, 382, 346, 278, 261, 303, 248, 356, 1139]`; ink px ref/ours 2470/2648 (ratio 1.0721); SSIM blocks <0.9: 329/30294; [overlay](images/13-math-display-rich/pdflatex-main-export-p1-overlay.png) (83311 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-main-export-p1-heatmap.png) (81661 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 3.5] pt by ink-projection correlation (centroid estimate [8.96, 2.59] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4734, differing 0.004074, SSIM₈ 0.9904 (raw 0.4817, 0.004129, 0.99)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.984→0.9847 / 0.7699→0.7567; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.398→0.3669 / 24.4984→25.1129 [232.8,85.1–384.7,123.2 pt]
- largest word displacements (pt): `√x` dx -9.86 dy 4.96; `display:` dx 0.1 dy -10.42; `Rich` dx 0 dy -10.42
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [236.873, 703.412, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931513, 620, 464, 431, 503, 455, 584, 411, 364, 315, 364, 315, 373, 343, 416, 1345]`; ink px ref/ours 2470/2847 (ratio 1.1526); SSIM blocks <0.9: 370/30294; [overlay](images/13-math-display-rich/pdflatex-pipeline-export-p1-overlay.png) (83199 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-pipeline-export-p1-heatmap.png) (81612 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.5654, not lower; centroid estimate [20.16, -1.23] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5415, differing 0.004271, SSIM₈ 0.9885 (raw 0.5415, 0.004271, 0.9885)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9816→0.9816 / 0.8654→0.8654; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3802→0.3802 / 23.3923→23.3923 [232.8,85.1–384.7,123.2 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -83.75 Δy 6.5 len 32.5 vs 12.0, thickness px 1 vs 1; Δx 34.0 Δy -6.75 len 32.5 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `1` dx -29.78 dy 0.41; `+` dx -29.78 dy 0.41; `x2` dx -29.78 dy 0.41
- word-sequence differences: insert ref [] ours ['∫']; replace ref ['√x'] ours ['√', 'x']; replace ref ['dx='] ours [',', 'dx', '=']

### 13-math-display-rich — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931353, 583, 548, 467, 406, 416, 441, 540, 508, 387, 355, 345, 457, 339, 349, 1322]`; ink px ref/ours 2283/2605 (ratio 1.141); SSIM blocks <0.9: 363/30294; [overlay](images/13-math-display-rich/pdflatex-lm-de1020c-export-p1-overlay.png) (83611 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-de1020c-export-p1-heatmap.png) (82596 B, ÷1)
  - registration error (diagnostic): global shift [-2.0, 4.0] pt by ink-projection correlation (centroid estimate [-23.81, 3.31] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5406, differing 0.004269, SSIM₈ 0.9893 (raw 0.553, 0.004302, 0.9888)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.982→0.983 / 0.8838→0.8638; header-band 1.0→0.9992 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4037→0.4126 / 25.0041→24.5413 [232.8,84.8–384.7,123 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -11.5 Δy 4.0 len 21.0 vs 32.0, thickness px 1 vs 1; Δx -16.75 Δy 4.0 len 21.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `display:` dx -1.48 dy -10.16; `Rich` dx 0 dy -10.16; `√x` dx -7.41 dy 5.22
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫', '1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — pdflatex-lm vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [236.87286281585693, 703.4119424819946, 6.652087211608887, 0.47818756103515625] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1934594, 299, 319, 263, 241, 228, 265, 306, 323, 239, 254, 226, 316, 165, 194, 584]`; ink px ref/ours 2283/2390 (ratio 1.0469); SSIM blocks <0.9: 250/30294; [overlay](images/13-math-display-rich/pdflatex-lm-exact-export-p1-overlay.png) (84419 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-exact-export-p1-heatmap.png) (77599 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-2.89, -0.46] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3074, differing 0.003008, SSIM₈ 0.9921 (raw 0.3074, 0.003008, 0.9921)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9874→0.9874 / 0.4913→0.4913; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4043→0.4043 / 21.9097→21.9097 [232.8,84.8–384.7,123 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.75 Δy 0.0 len 31.5 vs 32.0, thickness px 1 vs 1; Δx -1.5 Δy 0.0 len 32.5 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `1` dx -29.78 dy 0.0; `0` dx -29.78 dy 0.0; `√x` dx -29.78 dy 0.0
- word-sequence differences: insert ref [] ours ['∫']; insert ref [] ours ['∞', '∑']; replace ref ['dx=', '∞', 'k=0', '(−1)k'] ours [',', 'dx', '=', '\\lef', 't(', '(-', '1)k']

### 13-math-display-rich — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (3): error: \left is not supported in math mode; error: \right is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [241.73, 685.5, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931230, 674, 565, 466, 406, 434, 479, 488, 526, 393, 354, 342, 483, 305, 361, 1310]`; ink px ref/ours 2283/2648 (ratio 1.1599); SSIM blocks <0.9: 373/30294; [overlay](images/13-math-display-rich/pdflatex-lm-main-export-p1-overlay.png) (83905 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-main-export-p1-heatmap.png) (83064 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 4.0] pt by ink-projection correlation (centroid estimate [-15.24, 3.15] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5397, differing 0.004318, SSIM₈ 0.9888 (raw 0.5543, 0.004359, 0.9885)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9816→0.9823 / 0.8859→0.8623; header-band 1.0→0.9991 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3993→0.3721 / 24.416→24.8563 [232.8,84.8–384.7,123 pt]
- largest word displacements (pt): `√x` dx -9.86 dy 5.22; `display:` dx -1.48 dy -10.16; `Rich` dx 0 dy -10.16
- word-sequence differences: delete ref ['1', '0'] ours []; insert ref [] ours ['∫1', '0']; replace ref ['1', '+', 'x2', 'dx=', '∞', 'k=0', '(−1)k', '4k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)', '(1)']

### 13-math-display-rich — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [236.873, 703.412, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931719, 593, 550, 474, 492, 416, 435, 530, 418, 336, 372, 318, 391, 301, 332, 1139]`; ink px ref/ours 2283/2847 (ratio 1.247); SSIM blocks <0.9: 372/30294; [overlay](images/13-math-display-rich/pdflatex-lm-pipeline-export-p1-overlay.png) (82734 B, ÷1), [heatmap](images/13-math-display-rich/pdflatex-lm-pipeline-export-p1-heatmap.png) (81055 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-4.05, -0.67] pt); confidence moderate (shift explains 12% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4487, differing 0.003895, SSIM₈ 0.9904 (raw 0.507, 0.004134, 0.9893)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9828→0.9847 / 0.8103→0.7171; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3984→0.418 / 22.6942→22.6865 [232.8,84.8–384.7,123 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 0.0 len 32.5 vs 32.0, thickness px 1 vs 1; Δx -1.5 Δy 0.0 len 32.5 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `1` dx -29.78 dy 0.67; `+` dx -29.78 dy 0.67; `x2` dx -29.78 dy 0.67
- word-sequence differences: insert ref [] ours ['∫']; replace ref ['√x'] ours ['√', 'x']; replace ref ['dx='] ours [',', 'dx', '=']

### 13-math-display-rich — xelatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931959, 611, 525, 464, 361, 421, 572, 475, 341, 330, 316, 270, 301, 321, 289, 1260]`; ink px ref/ours 2475/2605 (ratio 1.0525); SSIM blocks <0.9: 322/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 3.5] pt REJECTED: applying it gives mean|Δ| 0.4946, not lower; centroid estimate [-0.43, 2.61] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4941, differing 0.004089, SSIM₈ 0.9899 (raw 0.4941, 0.004089, 0.9899)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9839→0.9839 / 0.7898→0.7898; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5932→0.5932 / 17.259→17.259 [227.3,75.6–384.7,129 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -66.25 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2; Δx 18.75 Δy -2.75 len 21.0 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `√x` dx -7.41 dy 4.37; `display.` dx 1.09 dy 3.86; `the` dx 1.02 dy 3.86
- word-sequence differences: replace ref ['∫1'] ours ['∫', '1']; replace ref ['1', '+', 'x2', 'dx=', '∞', '∑', 'k=0', '((−1)k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — xelatex vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [236.87286281585693, 703.4119424819946, 6.652087211608887, 0.47818756103515625] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931318, 601, 508, 479, 408, 478, 720, 502, 531, 398, 400, 350, 397, 341, 324, 1061]`; ink px ref/ours 2475/2390 (ratio 0.9657); SSIM blocks <0.9: 388/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [20.48, -1.17] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5161, differing 0.004295, SSIM₈ 0.9886 (raw 0.5315, 0.004365, 0.9882)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9812→0.9818 / 0.8495→0.8249; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5787→0.6393 / 15.8896→13.2299 [227.3,75.6–384.7,129 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -84.25 Δy 6.5 len 31.5 vs 12.0, thickness px 1 vs 1; Δx 34.0 Δy -6.75 len 32.5 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `√x` dx -29.78 dy -0.85; `1` dx -29.78 dy -0.26; `x2` dx -29.78 dy -0.26
- word-sequence differences: insert ref [] ours ['∫', '1', '0']; delete ref ['∫1', '0'] ours []; insert ref [] ours ['∞', '∑']

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
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [236.873, 703.412, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931416, 610, 479, 436, 445, 442, 647, 454, 390, 358, 379, 309, 392, 377, 371, 1311]`; ink px ref/ours 2475/2847 (ratio 1.1503); SSIM blocks <0.9: 377/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [8.5, -0.5] pt REJECTED: applying it gives mean|Δ| 0.5495, not lower; centroid estimate [19.33, -1.38] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5457, differing 0.004307, SSIM₈ 0.9884 (raw 0.5457, 0.004307, 0.9884)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9815→0.9815 / 0.8722→0.8722; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5835→0.5835 / 16.6902→16.6902 [227.3,75.6–384.7,129 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -83.75 Δy 6.5 len 32.5 vs 12.0, thickness px 1 vs 1; Δx 34.0 Δy -6.75 len 32.5 vs 12.5, thickness px 1 vs 2
- largest word displacements (pt): `∑` dx -20.22 dy -26.77; `1` dx -29.78 dy 0.41; `x2` dx -29.78 dy 0.41
- word-sequence differences: insert ref [] ours ['∫', '1', '0']; replace ref ['∫1', '0', '√x'] ours ['√', 'x']; replace ref ['dx='] ours [',', 'dx', '=']

### 13-math-display-rich — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [242.98, 685.5, 21.0, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931243, 707, 529, 432, 440, 437, 471, 554, 438, 438, 467, 317, 319, 346, 354, 1324]`; ink px ref/ours 2286/2605 (ratio 1.1395); SSIM blocks <0.9: 363/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-2.0, 4.0] pt by ink-projection correlation (centroid estimate [-21.76, 3.0] pt); confidence weak (shift explains 1% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.5476, differing 0.004334, SSIM₈ 0.9892 (raw 0.5517, 0.004326, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.982→0.9828 / 0.8818→0.875; header-band 1.0→0.9992 / 0.0→0.0021; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.591→0.6091 / 17.212→16.9465 [227.3,75.4–384.7,128.9 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -11.5 Δy 4.0 len 21.0 vs 31.0, thickness px 1 vs 1; Δx -16.75 Δy 4.0 len 21.0 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `√x` dx -7.41 dy 4.51; `display.` dx -7.46 dy 4.15; `the` dx -4.97 dy 4.15
- word-sequence differences: replace ref ['∫1'] ours ['∫', '1']; replace ref ['1', '+', 'x2', 'dx=', '∞', '∑', 'k=0', '((−1)k'] ours ['2', ',dx=∑∞', 'k=0\\left(', '(-1)k', '1+x', '4k+3', '\\right)']

### 13-math-display-rich — xelatex-lm vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \left is not supported in math mode; error: \right is not supported in math mode
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [236.87286281585693, 703.4119424819946, 6.652087211608887, 0.47818756103515625] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1933415, 633, 505, 430, 400, 327, 387, 324, 332, 324, 335, 197, 219, 186, 220, 582]`; ink px ref/ours 2286/2390 (ratio 1.0455); SSIM blocks <0.9: 298/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.84, -0.77] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.346, differing 0.003445, SSIM₈ 0.9917 (raw 0.346, 0.003445, 0.9917)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9867→0.9867 / 0.5531→0.5531; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5865→0.5865 / 15.5508→15.5508 [227.3,75.4–384.7,128.9 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.75 Δy 0.0 len 31.5 vs 31.0, thickness px 1 vs 1; Δx -1.5 Δy 0.0 len 32.5 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `√x` dx -29.78 dy -0.71; `1` dx -29.78 dy -0.11; `x2` dx -29.78 dy -0.11
- word-sequence differences: insert ref [] ours ['∫', '1', '0']; delete ref ['∫1', '0'] ours []; insert ref [] ours ['∞', '∑']

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
- FlashTeX rule rectangles: 3 U+2500 items in compile_result, 3 `re f` rectangles in the PDF (first: [236.873, 703.412, 5.58, 0.478] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931488, 716, 600, 467, 507, 414, 479, 474, 392, 382, 454, 308, 318, 321, 364, 1132]`; ink px ref/ours 2286/2847 (ratio 1.2454); SSIM blocks <0.9: 373/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.99, -0.98] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.46, differing 0.00398, SSIM₈ 0.9903 (raw 0.5144, 0.004196, 0.9891)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9826→0.9845 / 0.8222→0.7353; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5936→0.593 / 16.3614→16.3457 [227.3,75.4–384.7,128.9 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 0.0 len 32.5 vs 31.0, thickness px 1 vs 1; Δx -1.5 Δy 0.0 len 32.5 vs 32.5, thickness px 1 vs 1
- largest word displacements (pt): `∑` dx -20.22 dy -26.63; `1` dx -29.78 dy 0.56; `x2` dx -29.78 dy 0.56
- word-sequence differences: insert ref [] ours ['∫', '1', '0']; replace ref ['∫1', '0', '√x'] ours ['√', 'x']; replace ref ['dx='] ours [',', 'dx', '=']

### 14-math-inline-dense — lualatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921604, 1302, 1274, 1027, 985, 855, 941, 950, 907, 1044, 839, 1126, 837, 737, 803, 3585]`; ink px ref/ours 4839/5720 (ratio 1.1821); SSIM blocks <0.9: 954/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-34.5, 13.5] pt by ink-projection correlation (centroid estimate [-14.42, 7.16] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9604, differing 0.008451, SSIM₈ 0.9783 (raw 1.3179, 0.009946, 0.9705)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9529→0.9733 / 2.1063→1.1471; header-band 1.0→0.9477 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3388→0.3463 / 26.8632→26.291 [96.4,69.7–279.9,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -447.81 dy 29.13; `,` dx 105.07 dy -10.04; `,` dx 84.05 dy -10.04
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours ['a2+b2', '=c2']; replace ref ['πr2'] ours ['α', 'βγ,']

### 14-math-inline-dense — lualatex vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 5.81pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [236.29573345184326, 710.8343820571899, 4.766899108886719, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924742, 1297, 1121, 1029, 1036, 815, 995, 954, 918, 858, 648, 794, 663, 573, 570, 1803]`; ink px ref/ours 4839/4438 (ratio 0.9171); SSIM blocks <0.9: 707/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.59, -0.03] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9281, differing 0.008093, SSIM₈ 0.9797 (raw 0.9576, 0.008246, 0.9792)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9673→0.9684 / 1.5149→1.4681; header-band 0.9979→0.9976 / 0.069→0.069; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3929→0.4555 / 24.759→21.9837 [96.4,69.7–279.9,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 22.75 Δy 0.0 len 27.0 vs 27.5, thickness px 1 vs 1; Δx -16.0 Δy 5.5 len 14.5 vs 27.5, thickness px 1 vs 1
- largest word displacements (pt): `,` dx -454.31 dy 15.21; `z2` dx -454.31 dy 12.47; `wrap.` dx -429.33 dy 13.29
- word-sequence differences: insert ref [] ours ['1', '∫']; delete ref ['p1,', 'σ2', ',', 'πr2', ',', 'δx,', 'n!,'] ours []; delete ref ['1'] ours []

### 14-math-inline-dense — lualatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [218.54, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921732, 1323, 1191, 1028, 950, 895, 929, 981, 866, 1003, 805, 1104, 843, 801, 832, 3533]`; ink px ref/ours 4839/5652 (ratio 1.168); SSIM blocks <0.9: 950/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-34.5, 13.5] pt by ink-projection correlation (centroid estimate [-13.23, 6.91] pt); confidence strong (shift explains 28% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.946, differing 0.008388, SSIM₈ 0.9784 (raw 1.3111, 0.009906, 0.9706)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.953→0.9736 / 2.0955→1.1258; header-band 1.0→0.9475 / 0.0→2.5818; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3425→0.3522 / 26.5831→26.1603 [96.4,69.7–279.9,108.1 pt]
- largest word displacements (pt): `,` dx -447.23 dy 29.13; `z2` dx -43.92 dy 3.61; `u2` dx -35.85 dy 11.6
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2', ',', 'πr2', ','] ours ['a2+b2', '=c2']; delete ref ['1', 'x'] ours []

### 14-math-inline-dense — lualatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 5.81pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [236.296, 710.834, 4.65, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924842, 1312, 1061, 1012, 1009, 849, 766, 829, 747, 809, 649, 829, 698, 574, 621, 2209]`; ink px ref/ours 4839/5593 (ratio 1.1558); SSIM blocks <0.9: 668/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [4.0, 0.0] pt REJECTED: applying it gives mean|Δ| 1.0155, not lower; centroid estimate [-3.86, -0.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9874, differing 0.008238, SSIM₈ 0.9807 (raw 0.9874, 0.008238, 0.9807)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9697→0.9697 / 1.5534→1.5534; header-band 0.9978→0.9978 / 0.1179→0.1179; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4405→0.4405 / 24.8193→24.8193 [96.4,69.7–279.9,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 23.25 Δy 0.0 len 28.0 vs 27.5, thickness px 1 vs 1; Δx -16.5 Δy 5.5 len 13.5 vs 27.5, thickness px 1 vs 1
- largest word displacements (pt): `must` dx 39.81 dy 0.45; `that` dx 35.74 dy 0.45; `line` dx 34.91 dy 0.45
- word-sequence differences: insert ref [] ours ['=', 'c2', 'z2', ',']; replace ref ['πr2'] ours ['πr', '2']; replace ref ['=', 'c2'] ours ['wrap.', '1', '∑', '√', '∫', 'a+', 'b', 'c']

### 14-math-inline-dense — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922082, 1331, 1233, 1019, 1029, 845, 940, 995, 1041, 1032, 818, 1076, 808, 667, 795, 3105]`; ink px ref/ours 4245/5720 (ratio 1.3475); SSIM blocks <0.9: 958/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-9.66, 6.78] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1675, differing 0.009235, SSIM₈ 0.9735 (raw 1.2473, 0.00966, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9655 / 1.9935→1.4755; header-band 1.0→0.9479 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3397→0.2823 / 24.4382→26.6867 [176,69.7–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -51.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -114.06 dy 11.6; `,` dx 110.02 dy -10.8; `,` dx 106.77 dy -10.8
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'z2'] ours ['a2+b2', '=c2']; replace ref ['p1,', 'σ2'] ours ['α', 'βγ,']

### 14-math-inline-dense — lualatex-lm vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 5.81pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [236.29573345184326, 710.8343820571899, 4.766899108886719, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931954, 1065, 647, 617, 486, 418, 444, 417, 439, 348, 236, 279, 303, 243, 249, 671]`; ink px ref/ours 4245/4438 (ratio 1.0455); SSIM blocks <0.9: 351/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.18, -0.41] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4203, differing 0.005552, SSIM₈ 0.99 (raw 0.4203, 0.005552, 0.99)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9846→0.9846 / 0.6563→0.6563; header-band 0.9979→0.9979 / 0.069→0.069; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.7044→0.7044 / 11.2978→11.2978 [176,69.7–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 27.0 vs 27.0, thickness px 1 vs 1; Δx 0.0 Δy 0.0 len 14.5 vs 14.5, thickness px 1 vs 1
- largest word displacements (pt): `a2` dx -1.65 dy 0.0; `+` dx -1.26 dy 0.0; `inline:` dx -1.1 dy 0.0
- word-sequence differences: insert ref [] ours ['1', '∫']; insert ref [] ours ['=', 'c2', ',', 'αβγ,', 'x', ',', '√2,', 'xj']; delete ref ['wrap.', '=', 'c2', ',', 'αβγ,', '1', 'x', ','] ours []

### 14-math-inline-dense — lualatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [218.54, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922131, 1374, 1176, 1039, 991, 888, 913, 1031, 1009, 984, 812, 1041, 791, 719, 824, 3093]`; ink px ref/ours 4245/5652 (ratio 1.3314); SSIM blocks <0.9: 954/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-8.47, 6.54] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1626, differing 0.009214, SSIM₈ 0.9734 (raw 1.2444, 0.009642, 0.9703)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9526→0.9654 / 1.9889→1.4699; header-band 1.0→0.9474 / 0.0→2.5818; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3441→0.2989 / 23.9997→26.3202 [176,69.7–288.8,108.1 pt]
- largest word displacements (pt): `must` dx -73.88 dy 13.68; `that` dx -69.88 dy 13.68; `line` dx -69.15 dy 13.68
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'z2', ',', 'p1,', 'σ2', ','] ours ['a2+b2', '=c2']; delete ref ['1', 'x'] ours []

### 14-math-inline-dense — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 5.81pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [236.296, 710.834, 4.65, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926740, 1345, 1004, 1032, 1012, 920, 712, 744, 723, 620, 504, 512, 515, 424, 473, 1536]`; ink px ref/ours 4245/5593 (ratio 1.3176); SSIM blocks <0.9: 596/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.9, -0.53] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7827, differing 0.007237, SSIM₈ 0.9851 (raw 0.7827, 0.007237, 0.9851)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9768→0.9768 / 1.2264→1.2264; header-band 0.9978→0.9978 / 0.1179→0.1179; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5389→0.5389 / 19.0242→19.0242 [176,69.7–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.5 Δy 0.0 len 28.0 vs 27.0, thickness px 1 vs 1; Δx -0.5 Δy 0.0 len 13.5 vs 14.5, thickness px 1 vs 1
- largest word displacements (pt): `g,` dx -6.3 dy -0.72; `f` dx -6.3 dy 0.19; `(x)` dx -6.3 dy 0.19
- word-sequence differences: insert ref [] ours ['=', 'c2']; replace ref ['πr2'] ours ['πr', '2']; replace ref ['=', 'c2'] ours ['1', '∑', '√', '∫', 'a+', 'b', 'c', ',']

### 14-math-inline-dense — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921723, 1256, 1319, 999, 1015, 858, 878, 898, 911, 966, 857, 1132, 834, 737, 861, 3572]`; ink px ref/ours 4824/5720 (ratio 1.1857); SSIM blocks <0.9: 948/30294; [overlay](images/14-math-inline-dense/pdflatex-de1020c-export-p1-overlay.png) (45598 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-de1020c-export-p1-heatmap.png) (44377 B, ÷2)
  - registration error (diagnostic): global shift [-34.0, 13.5] pt by ink-projection correlation (centroid estimate [-13.94, 7.12] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9608, differing 0.008388, SSIM₈ 0.9783 (raw 1.3129, 0.009906, 0.9708)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9533→0.9734 / 2.0983→1.1478; header-band 1.0→0.9469 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3428→0.3554 / 26.6779→25.5089 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -407.82 dy 29.13; `,` dx -87.71 dy 29.13; `,` dx 84.05 dy -10.04
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours ['a2+b2', '=c2']; insert ref [] ours ['α', 'βγ,', ',', '√2', ',', 'xj', 'i', ',']

### 14-math-inline-dense — pdflatex vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 5.81pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [236.29573345184326, 710.8343820571899, 4.766899108886719, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924884, 1269, 1120, 1041, 982, 793, 911, 934, 921, 804, 651, 838, 662, 618, 599, 1789]`; ink px ref/ours 4824/4438 (ratio 0.92); SSIM blocks <0.9: 706/30294; [overlay](images/14-math-inline-dense/pdflatex-exact-export-p1-overlay.png) (45700 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-exact-export-p1-heatmap.png) (41244 B, ÷2)
  - registration error (diagnostic): global shift [0.5, 0.0] pt by ink-projection correlation (centroid estimate [-4.11, -0.07] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9102, differing 0.008023, SSIM₈ 0.9801 (raw 0.9552, 0.00818, 0.9792)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9674→0.9688 / 1.5112→1.4397; header-band 0.9979→0.9976 / 0.069→0.069; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3878→0.4336 / 24.6992→22.7655 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 22.75 Δy 0.0 len 27.0 vs 27.5, thickness px 1 vs 1; Δx -16.0 Δy 5.5 len 14.5 vs 27.5, thickness px 1 vs 1
- largest word displacements (pt): `wrap.` dx -429.06 dy 13.29; `must` dx 40.08 dy -0.0; `that` dx 36.02 dy -0.0
- word-sequence differences: insert ref [] ours ['1', '∫']; insert ref [] ours ['=', 'c2', ',', 'αβγ,', 'x', ',', '√2,', 'xj']; delete ref ['=', 'c2', ',', 'αβγ,', '1', 'x,', '√2,', 'xj'] ours []

### 14-math-inline-dense — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [218.54, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921877, 1256, 1248, 993, 983, 883, 867, 932, 870, 916, 839, 1099, 839, 807, 890, 3517]`; ink px ref/ours 4824/5652 (ratio 1.1716); SSIM blocks <0.9: 944/30294; [overlay](images/14-math-inline-dense/pdflatex-main-export-p1-overlay.png) (45170 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-main-export-p1-heatmap.png) (44054 B, ÷2)
  - registration error (diagnostic): global shift [-34.0, 13.5] pt by ink-projection correlation (centroid estimate [-12.75, 6.87] pt); confidence strong (shift explains 29% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9332, differing 0.008315, SSIM₈ 0.9785 (raw 1.3054, 0.009857, 0.9708)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9534→0.974 / 2.0863→1.1058; header-band 1.0→0.9454 / 0.0→2.5818; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3465→0.3563 / 26.3679→25.2453 [96.4,69.7–279.8,108.1 pt]
- largest word displacements (pt): `,` dx -447.23 dy 29.13; `z2` dx -43.92 dy 3.61; `u2` dx -35.85 dy 11.6
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2', ',', 'πr2', ','] ours ['a2+b2', '=c2']; replace ref ['1', 'x,', '√2,'] ours [',', '√2', ',']

### 14-math-inline-dense — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 5.81pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [236.296, 710.834, 4.65, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1925157, 1263, 1073, 965, 959, 825, 727, 770, 692, 734, 638, 845, 715, 623, 617, 2213]`; ink px ref/ours 4824/5593 (ratio 1.1594); SSIM blocks <0.9: 659/30294; [overlay](images/14-math-inline-dense/pdflatex-pipeline-export-p1-overlay.png) (44896 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-pipeline-export-p1-heatmap.png) (39758 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-3.38, -0.19] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9751, differing 0.008137, SSIM₈ 0.9808 (raw 0.9751, 0.008137, 0.9808)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9699→0.9699 / 1.5338→1.5338; header-band 0.9978→0.9978 / 0.1179→0.1179; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4394→0.4394 / 24.6448→24.6448 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 23.25 Δy 0.0 len 28.0 vs 27.5, thickness px 1 vs 1; Δx -16.5 Δy 5.5 len 13.5 vs 27.5, thickness px 1 vs 1
- largest word displacements (pt): `must` dx 40.08 dy 0.45; `that` dx 36.02 dy 0.45; `line` dx 35.2 dy 0.45
- word-sequence differences: insert ref [] ours ['=', 'c2', 'z2', ',']; replace ref ['πr2'] ours ['πr', '2']; replace ref ['=', 'c2'] ours ['wrap.', '1', '∑', '√', '∫', 'a+', 'b', 'c']

### 14-math-inline-dense — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922114, 1292, 1255, 994, 1030, 864, 932, 987, 1070, 1032, 804, 1077, 790, 689, 773, 3113]`; ink px ref/ours 4254/5720 (ratio 1.3446); SSIM blocks <0.9: 957/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-de1020c-export-p1-overlay.png) (45740 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-de1020c-export-p1-heatmap.png) (44270 B, ÷2)
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-7.87, 6.82] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1673, differing 0.009234, SSIM₈ 0.9735 (raw 1.2469, 0.009657, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9655 / 1.9928→1.475; header-band 1.0→0.9479 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3525→0.2973 / 24.7689→27.2474 [183.4,69.8–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -51.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `z2` dx 408.28 dy -8.86; `must` dx -73.95 dy 13.68; `that` dx -69.96 dy 13.68
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2'] ours ['a2+b2', '=c2', ',', 'α', 'βγ,', ',', '√2', ',']; insert ref [] ours ['πr2', 'a+b']

### 14-math-inline-dense — pdflatex-lm vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 5.81pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [236.29573345184326, 710.8343820571899, 4.766899108886719, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1932087, 959, 651, 606, 485, 396, 442, 400, 468, 346, 242, 269, 292, 259, 238, 676]`; ink px ref/ours 4254/4438 (ratio 1.0433); SSIM blocks <0.9: 350/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-exact-export-p1-overlay.png) (44948 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-exact-export-p1-heatmap.png) (86217 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [1.96, -0.37] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4137, differing 0.005137, SSIM₈ 0.99 (raw 0.4137, 0.005137, 0.99)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9847→0.9847 / 0.6457→0.6457; header-band 0.9979→0.9979 / 0.069→0.069; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.683→0.683 / 11.7789→11.7789 [183.4,69.8–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 27.0 vs 27.0, thickness px 1 vs 1; Δx 0.0 Δy 0.0 len 14.5 vs 14.5, thickness px 1 vs 1
- largest word displacements (pt): `a2` dx -1.66 dy 1.03; `+` dx -1.27 dy 1.03; `inline:` dx -1.11 dy 1.03
- word-sequence differences: insert ref [] ours ['1', '∫']; insert ref [] ours ['=', 'c2', ',', 'αβγ,', 'x', ',', '√2,', 'xj']; delete ref ['wrap.', '=', 'c2', ',', 'αβγ,', '1', 'x,', '√2,'] ours []

### 14-math-inline-dense — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [218.54, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922163, 1332, 1203, 1012, 991, 900, 908, 1027, 1040, 978, 799, 1043, 776, 743, 802, 3099]`; ink px ref/ours 4254/5652 (ratio 1.3286); SSIM blocks <0.9: 953/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-main-export-p1-overlay.png) (45443 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-main-export-p1-heatmap.png) (44053 B, ÷2)
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-6.68, 6.57] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1624, differing 0.009208, SSIM₈ 0.9734 (raw 1.244, 0.00964, 0.9703)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9526→0.9654 / 1.9883→1.4696; header-band 1.0→0.9474 / 0.0→2.5818; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3519→0.2989 / 24.3244→26.8863 [183.4,69.8–288.8,108.1 pt]
- largest word displacements (pt): `must` dx -73.89 dy 13.68; `that` dx -69.9 dy 13.68; `line` dx -69.17 dy 13.68
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'z2', ',', 'p1,', 'σ2', ','] ours ['a2+b2', '=c2']; replace ref ['1', 'x,', '√2,'] ours [',', '√2', ',']

### 14-math-inline-dense — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 5.81pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [236.296, 710.834, 4.65, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926753, 1316, 1022, 995, 1015, 924, 734, 726, 749, 610, 526, 497, 499, 440, 472, 1538]`; ink px ref/ours 4254/5593 (ratio 1.3148); SSIM blocks <0.9: 596/30294; [overlay](images/14-math-inline-dense/pdflatex-lm-pipeline-export-p1-overlay.png) (44531 B, ÷2), [heatmap](images/14-math-inline-dense/pdflatex-lm-pipeline-export-p1-heatmap.png) (38200 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.69, -0.5] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7836, differing 0.007233, SSIM₈ 0.9851 (raw 0.7836, 0.007233, 0.9851)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9768→0.9768 / 1.2278→1.2278; header-band 0.9978→0.9978 / 0.1179→0.1179; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5233→0.5233 / 19.4536→19.4536 [183.4,69.8–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.5 Δy 0.0 len 28.0 vs 27.0, thickness px 1 vs 1; Δx -0.5 Δy 0.0 len 13.5 vs 14.5, thickness px 1 vs 1
- largest word displacements (pt): `g,` dx -6.31 dy -0.72; `=` dx -6.31 dy 0.19; `b,` dx 5.8 dy -0.72
- word-sequence differences: insert ref [] ours ['=', 'c2']; replace ref ['πr2'] ours ['πr', '2']; replace ref ['=', 'c2'] ours ['1', '∑', '√', '∫', 'a+', 'b', 'c', ',']

### 14-math-inline-dense — xelatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921627, 1283, 1284, 1007, 1006, 848, 933, 938, 920, 1037, 846, 1107, 844, 738, 798, 3600]`; ink px ref/ours 4852/5720 (ratio 1.1789); SSIM blocks <0.9: 954/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-34.5, 13.5] pt by ink-projection correlation (centroid estimate [-14.66, 7.14] pt); confidence strong (shift explains 27% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9575, differing 0.00844, SSIM₈ 0.9783 (raw 1.3179, 0.009938, 0.9705)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9529→0.9733 / 2.1063→1.1425; header-band 1.0→0.9477 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3387→0.3463 / 26.8617→26.2884 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -29.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -447.81 dy 29.13; `,` dx 105.07 dy -10.04; `,` dx 84.05 dy -10.04
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2'] ours ['a2+b2', '=c2']; replace ref ['πr2'] ours ['α', 'βγ,']

### 14-math-inline-dense — xelatex vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 5.81pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [236.29573345184326, 710.8343820571899, 4.766899108886719, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924756, 1288, 1118, 1023, 1035, 829, 982, 919, 949, 870, 631, 799, 661, 572, 573, 1811]`; ink px ref/ours 4852/4438 (ratio 0.9147); SSIM blocks <0.9: 707/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [7.5, 0.0] pt by ink-projection correlation (centroid estimate [-4.82, -0.05] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9392, differing 0.008094, SSIM₈ 0.98 (raw 0.9581, 0.008247, 0.9792)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9673→0.9693 / 1.5158→1.4618; header-band 0.9979→0.9979 / 0.069→0.069; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3928→0.3725 / 24.7528→24.3021 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 22.75 Δy 0.0 len 27.0 vs 27.5, thickness px 1 vs 1; Δx -16.0 Δy 5.5 len 14.5 vs 27.5, thickness px 1 vs 1
- largest word displacements (pt): `wrap.` dx -429.36 dy 13.29; `must` dx 39.78 dy -0.0; `that` dx 35.72 dy -0.0
- word-sequence differences: insert ref [] ours ['1', '∫']; insert ref [] ours ['=', 'c2', ',', 'αβγ,', 'x', ',', '√2,', 'xj']; delete ref ['=', 'c2', ',', 'αβγ,', '1', 'x', ',', '√2,'] ours []

### 14-math-inline-dense — xelatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [218.54, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921752, 1305, 1204, 1007, 971, 886, 922, 969, 880, 997, 810, 1087, 848, 802, 829, 3547]`; ink px ref/ours 4852/5652 (ratio 1.1649); SSIM blocks <0.9: 950/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-34.5, 13.5] pt by ink-projection correlation (centroid estimate [-13.46, 6.89] pt); confidence strong (shift explains 28% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9422, differing 0.00837, SSIM₈ 0.9785 (raw 1.3112, 0.009898, 0.9706)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.953→0.9736 / 2.0955→1.1198; header-band 1.0→0.9475 / 0.0→2.5818; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3424→0.3522 / 26.5817→26.1576 [96.4,69.7–279.8,108.1 pt]
- largest word displacements (pt): `,` dx -447.23 dy 29.13; `z2` dx -43.92 dy -13.61; `u2` dx -35.85 dy 11.6
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'p1,', 'σ2', ',', 'πr2', ','] ours ['a2+b2', '=c2']; delete ref ['1', 'x'] ours []

### 14-math-inline-dense — xelatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 5.81pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [236.296, 710.834, 4.65, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1924859, 1310, 1058, 1006, 1010, 857, 747, 802, 763, 822, 637, 838, 698, 574, 610, 2225]`; ink px ref/ours 4852/5593 (ratio 1.1527); SSIM blocks <0.9: 668/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [4.0, 0.0] pt REJECTED: applying it gives mean|Δ| 1.0155, not lower; centroid estimate [-4.09, -0.17] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9874, differing 0.008232, SSIM₈ 0.9807 (raw 0.9874, 0.008232, 0.9807)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9697→0.9697 / 1.5535→1.5535; header-band 0.9978→0.9978 / 0.1179→0.1179; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.4408→0.4408 / 24.8006→24.8006 [96.4,69.7–279.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 23.25 Δy 0.0 len 28.0 vs 27.5, thickness px 1 vs 1; Δx -16.5 Δy 5.5 len 13.5 vs 27.5, thickness px 1 vs 1
- largest word displacements (pt): `must` dx 39.78 dy 0.45; `that` dx 35.72 dy 0.45; `line` dx 34.9 dy 0.45
- word-sequence differences: insert ref [] ours ['=', 'c2', 'z2', ',']; replace ref ['πr2'] ours ['πr', '2']; replace ref ['=', 'c2'] ours ['wrap.', '1', '∑', '√', '∫', 'a+', 'b', 'c']

### 14-math-inline-dense — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (1): error: \nu is not supported in math mode
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [217.45, 707.28, 8.4, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922076, 1334, 1239, 1015, 1031, 844, 933, 1003, 1046, 1026, 822, 1075, 807, 668, 793, 3104]`; ink px ref/ours 4244/5720 (ratio 1.3478); SSIM blocks <0.9: 958/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-9.59, 6.78] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1675, differing 0.009235, SSIM₈ 0.9735 (raw 1.2473, 0.00966, 0.9702)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9524→0.9655 / 1.9935→1.4755; header-band 1.0→0.9479 / 0.0→2.5978; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3525→0.2992 / 24.8173→27.2669 [183.4,69.7–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx -51.25 Δy 13.75 len 17.0 vs 14.5, thickness px 2 vs 1
- largest word displacements (pt): `,` dx -114.06 dy 11.6; `,` dx 110.02 dy -9.77; `,` dx 106.77 dy -9.77
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'z2'] ours ['a2+b2', '=c2']; replace ref ['p1,', 'σ2'] ours ['α', 'βγ,']

### 14-math-inline-dense — xelatex-lm vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 5.81pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [236.29573345184326, 710.8343820571899, 4.766899108886719, 0.3984842300415039] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1931948, 1066, 651, 614, 488, 420, 443, 419, 452, 333, 240, 275, 308, 244, 245, 670]`; ink px ref/ours 4244/4438 (ratio 1.0457); SSIM blocks <0.9: 351/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.25, -0.41] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.4203, differing 0.005567, SSIM₈ 0.99 (raw 0.4203, 0.005567, 0.99)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9846→0.9846 / 0.6563→0.6563; header-band 0.9979→0.9979 / 0.069→0.069; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.6853→0.6853 / 11.9155→11.9155 [183.4,69.7–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 27.0 vs 27.0, thickness px 1 vs 1; Δx 0.0 Δy 0.0 len 14.5 vs 14.5, thickness px 1 vs 1
- largest word displacements (pt): `a2` dx -1.65 dy 1.03; `+` dx -1.26 dy 1.03; `inline:` dx -1.1 dy 1.03
- word-sequence differences: insert ref [] ours ['1', '∫']; insert ref [] ours ['=', 'c2', ',', 'αβγ,', 'x', ',', '√2,', 'xj']; delete ref ['wrap.', '=', 'c2', ',', 'αβγ,', '1', 'x', ','] ours []

### 14-math-inline-dense — xelatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully
- FlashTeX rule rectangles: 2 U+2500 items in compile_result, 2 `re f` rectangles in the PDF (first: [218.54, 707.28, 4.2, 0.72] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1922125, 1378, 1181, 1034, 994, 887, 907, 1037, 1011, 982, 816, 1038, 791, 721, 822, 3092]`; ink px ref/ours 4244/5652 (ratio 1.3318); SSIM blocks <0.9: 954/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [5.0, 13.5] pt by ink-projection correlation (centroid estimate [-8.39, 6.54] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1627, differing 0.009214, SSIM₈ 0.9734 (raw 1.2444, 0.009642, 0.9703)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9526→0.9654 / 1.9889→1.47; header-band 1.0→0.9474 / 0.0→2.5818; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.3521→0.3007 / 24.3679→26.9031 [183.4,69.7–288.8,108.1 pt]
- largest word displacements (pt): `must` dx -73.88 dy 13.68; `that` dx -69.88 dy 13.68; `line` dx -69.15 dy 13.68
- word-sequence differences: insert ref [] ours ['1']; replace ref ['a2', '+', 'b2', 'z2', ',', 'p1,', 'σ2', ','] ours ['a2+b2', '=c2']; delete ref ['1', 'x'] ours []

### 14-math-inline-dense — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \nu is not supported in math mode; warning: overfull line: 5.81pt too wide (no hyphenation available)
- FlashTeX rule rectangles: 4 U+2500 items in compile_result, 4 `re f` rectangles in the PDF (first: [236.296, 710.834, 4.65, 0.399] pt, PDF bottom-left origin)
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926738, 1347, 1009, 1028, 1011, 917, 711, 749, 733, 608, 508, 510, 517, 426, 471, 1533]`; ink px ref/ours 4244/5593 (ratio 1.3179); SSIM blocks <0.9: 596/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.98, -0.53] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7826, differing 0.007222, SSIM₈ 0.9851 (raw 0.7826, 0.007222, 0.9851)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9768→0.9768 / 1.2263→1.2263; header-band 0.9978→0.9978 / 0.1179→0.1179; footer-band 1.0→1.0 / 0.0→0.0; display-1 0.5252→0.5252 / 19.455→19.455 [183.4,69.7–288.8,108.1 pt]
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.5 Δy 0.0 len 28.0 vs 27.0, thickness px 1 vs 1; Δx -0.5 Δy 0.0 len 13.5 vs 14.5, thickness px 1 vs 1
- largest word displacements (pt): `g,` dx -6.3 dy -17.95; `b,` dx 5.79 dy -17.95; `+` dx 5.41 dy -17.95
- word-sequence differences: insert ref [] ours ['=', 'c2']; replace ref ['πr2'] ours ['πr', '2']; replace ref ['=', 'c2'] ours ['1', '∑', '√', '∫', 'a+', 'b', 'c', ',']

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

### 15-three-page-sections — lualatex vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1659873, 24097, 20545, 18752, 17987, 17024, 17208, 17675, 16637, 15074, 13920, 13389, 13104, 12733, 12543, 48255]`; ink px ref/ours 112785/85708 (ratio 0.7599); SSIM blocks <0.9: 11501/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 43.0] pt by ink-projection correlation (centroid estimate [-5.99, 27.56] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.4387, differing 0.15717, SSIM₈ 0.6758 (raw 20.1579, 0.16047, 0.6592)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.456→0.4976 / 32.2111→30.4047; header-band 1.0→0.8965 / 0.0→4.5209; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 72.5 pt with NO reference rule on this page
- page 2: |Δ| histogram (16 bins, pixel counts) `[1659750, 24098, 20566, 18763, 17998, 17032, 17215, 17683, 16638, 15073, 13922, 13394, 13091, 12735, 12553, 48305]`; ink px ref/ours 112804/85745 (ratio 0.7601); SSIM blocks <0.9: 11504/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 43.0] pt by ink-projection correlation (centroid estimate [-6.02, 27.44] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.4502, differing 0.157243, SSIM₈ 0.6756 (raw 20.1676, 0.160541, 0.6591)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4558→0.4976 / 32.2266→30.4131; header-band 1.0→0.8947 / 0.0→4.5892; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 72.5 pt with NO reference rule on this page
- page 3: |Δ| histogram (16 bins, pixel counts) `[1659758, 24092, 20564, 18769, 17998, 17037, 17217, 17677, 16655, 15076, 13929, 13390, 13094, 12732, 12527, 48301]`; ink px ref/ours 112799/85743 (ratio 0.7601); SSIM blocks <0.9: 11508/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 43.0] pt by ink-projection correlation (centroid estimate [-6.06, 27.51] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.4465, differing 0.157235, SSIM₈ 0.6756 (raw 20.1656, 0.160541, 0.6591)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4557→0.4975 / 32.2234→30.4085; header-band 1.0→0.8951 / 0.0→4.5807; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 72.5 pt with NO reference rule on this page
- largest word displacements (pt): `oak` dx -415.78 dy 58.09; `oak` dx -415.78 dy 58.09; `oak` dx -415.78 dy 58.09

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1649331, 23266, 19875, 18304, 16921, 16827, 16131, 16034, 15670, 15476, 14846, 14223, 13864, 13618, 13867, 60563]`; ink px ref/ours 112785/111942 (ratio 0.9925); SSIM blocks <0.9: 11598/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 14.0] pt by ink-projection correlation (centroid estimate [-5.4, 28.58] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.041, differing 0.161923, SSIM₈ 0.6754 (raw 21.9864, 0.165834, 0.656)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4511→0.4854 / 35.1327→33.5043; header-band 1.0→0.986 / 0.0→0.7482; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1649202, 23289, 19897, 18316, 16925, 16809, 16118, 16049, 15663, 15486, 14853, 14225, 13857, 13627, 13885, 60615]`; ink px ref/ours 112804/112002 (ratio 0.9929); SSIM blocks <0.9: 11601/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 14.0] pt by ink-projection correlation (centroid estimate [-5.45, 28.42] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.0517, differing 0.161993, SSIM₈ 0.6753 (raw 21.9975, 0.165903, 0.6558)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4508→0.4854 / 35.1504→33.5099; header-band 1.0→0.9848 / 0.0→0.8282; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1649183, 23289, 19895, 18319, 16928, 16822, 16126, 16044, 15665, 15487, 14851, 14219, 13855, 13624, 13866, 60643]`; ink px ref/ours 112799/111975 (ratio 0.9927); SSIM blocks <0.9: 11604/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.5, 14.0] pt by ink-projection correlation (centroid estimate [-5.44, 28.56] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.0477, differing 0.161986, SSIM₈ 0.6752 (raw 21.9988, 0.165918, 0.6557)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4507→0.4852 / 35.1524→33.5079; header-band 1.0→0.9855 / 0.0→0.7969; footer-band 1.0→1.0 / 0.0→0.0
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

### 15-three-page-sections — lualatex-lm vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938684, 132, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 85641/85708 (ratio 1.0008); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.04, 0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3442, differing 0.077704, SSIM₈ 0.9996 (raw 0.3442, 0.077704, 0.9996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9994→0.9994 / 0.5501→0.5501; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 10.0 vs 10.0, thickness px 1 vs 1
- page 2: |Δ| histogram (16 bins, pixel counts) `[1938701, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 85678/85745 (ratio 1.0008); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.08, 0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3444, differing 0.077731, SSIM₈ 0.9996 (raw 0.3444, 0.077731, 0.9996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9994→0.9994 / 0.5503→0.5503; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 10.0 vs 10.0, thickness px 1 vs 1
- page 3: |Δ| histogram (16 bins, pixel counts) `[1938692, 124, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 85675/85743 (ratio 1.0008); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.06, 0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3444, differing 0.077741, SSIM₈ 0.9996 (raw 0.3444, 0.077741, 0.9996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9994→0.9994 / 0.5504→0.5504; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 10.0 vs 10.0, thickness px 1 vs 1
- largest word displacements (pt): `old` dx 0.03 dy -0.01; `patient` dx 0.03 dy -0.01; `old` dx 0.03 dy -0.01

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1725760, 24640, 19980, 18764, 18005, 16844, 15509, 14490, 12004, 11081, 9073, 8478, 7928, 7093, 6498, 22669]`; ink px ref/ours 85641/111942 (ratio 1.3071); SSIM blocks <0.9: 9093/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.63, 1.08] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1286, differing 0.128738, SSIM₈ 0.8177 (raw 13.1286, 0.128738, 0.8177)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7089→0.7089 / 20.9815→20.9815; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1725574, 24655, 20005, 18785, 17999, 16844, 15500, 14494, 12001, 11080, 9071, 8500, 7932, 7113, 6519, 22744]`; ink px ref/ours 85678/112002 (ratio 1.3072); SSIM blocks <0.9: 9097/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.65, 1.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1463, differing 0.128839, SSIM₈ 0.8174 (raw 13.1463, 0.128839, 0.8174)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7085→0.7085 / 21.0097→21.0097; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1725548, 24659, 20006, 18782, 18005, 16838, 15502, 14490, 12012, 11091, 9072, 8487, 7938, 7104, 6505, 22777]`; ink px ref/ours 85675/111975 (ratio 1.307); SSIM blocks <0.9: 9100/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.67, 1.1] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1488, differing 0.128861, SSIM₈ 0.8174 (raw 13.1488, 0.128861, 0.8174)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7084→0.7084 / 21.0138→21.0138; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Part` dx 29.05 dy -0.67; `Part` dx 29.05 dy -0.67; `Part` dx 29.05 dy -0.67
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1595899, 24122, 20968, 20023, 17718, 17966, 18322, 17267, 17094, 18406, 16944, 15990, 16755, 16056, 16950, 88336]`; ink px ref/ours 112156/116938 (ratio 1.0426); SSIM blocks <0.9: 14346/30294; [overlay](images/15-three-page-sections/pdflatex-de1020c-export-p1-overlay.png) (48132 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-de1020c-export-p1-heatmap.png) (35395 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 36.5] pt by ink-projection correlation (centroid estimate [-13.64, 42.99] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.376, differing 0.183932, SSIM₈ 0.5937 (raw 27.5937, 0.195208, 0.5337)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2665→0.3542 / 43.4405→40.4605; header-band 1.0→0.9879 / 0.0→0.6238; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1592865, 24793, 21494, 20314, 18311, 18548, 19317, 17570, 17258, 18450, 17153, 16254, 16690, 16013, 17183, 86603]`; ink px ref/ours 112190/122724 (ratio 1.0939); SSIM blocks <0.9: 14238/30294; [overlay](images/15-three-page-sections/pdflatex-de1020c-export-p2-overlay.png) (49039 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-de1020c-export-p2-heatmap.png) (35508 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.72, 17.61] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.0991, differing 0.18994, SSIM₈ 0.5742 (raw 27.5896, 0.197186, 0.5428)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2693→0.3363 / 44.0962→41.0127; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8846 / 0.0→4.8261
- page 3: |Δ| histogram (16 bins, pixel counts) `[1626455, 22468, 19561, 18552, 16338, 16950, 16970, 15864, 15658, 16281, 15491, 14672, 14985, 14631, 15599, 78341]`; ink px ref/ours 112177/99417 (ratio 0.8863); SSIM blocks <0.9: 12897/30294; [overlay](images/15-three-page-sections/pdflatex-de1020c-export-p3-overlay.png) (45316 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-de1020c-export-p3-heatmap.png) (33359 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.66, -40.6] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.7997, differing 0.172056, SSIM₈ 0.6079 (raw 24.915, 0.178224, 0.5858)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.338→0.3733 / 39.8213→38.0388; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy -63.97; `branch` dx -435.47 dy -58.38; `branch` dx -435.47 dy -52.79

### 15-three-page-sections — pdflatex vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1660815, 23312, 20508, 18998, 17449, 17086, 17420, 18280, 17149, 14929, 13717, 12822, 12352, 12575, 12541, 48863]`; ink px ref/ours 112156/85708 (ratio 0.7642); SSIM blocks <0.9: 11182/30294; [overlay](images/15-three-page-sections/pdflatex-exact-export-p1-overlay.png) (46370 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-exact-export-p1-heatmap.png) (78618 B, ÷4)
  - registration error (diagnostic): global shift [0.0, -0.5] pt by ink-projection correlation (centroid estimate [-5.49, 4.74] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.3897, differing 0.155664, SSIM₈ 0.6801 (raw 20.1213, 0.159645, 0.6653)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4653→0.4889 / 32.1587→30.9893; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 72.5 pt with NO reference rule on this page
- page 2: |Δ| histogram (16 bins, pixel counts) `[1660690, 23329, 20526, 19012, 17460, 17077, 17420, 18283, 17154, 14926, 13732, 12834, 12342, 12575, 12545, 48911]`; ink px ref/ours 112190/85745 (ratio 0.7643); SSIM blocks <0.9: 11185/30294; [overlay](images/15-three-page-sections/pdflatex-exact-export-p2-overlay.png) (46424 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-exact-export-p2-heatmap.png) (78721 B, ÷4)
  - registration error (diagnostic): global shift [0.0, -0.5] pt by ink-projection correlation (centroid estimate [-5.47, 4.69] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.3993, differing 0.155739, SSIM₈ 0.68 (raw 20.1309, 0.15972, 0.6652)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4651→0.4888 / 32.174→31.0047; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 72.5 pt with NO reference rule on this page
- page 3: |Δ| histogram (16 bins, pixel counts) `[1660700, 23321, 20522, 19015, 17465, 17081, 17427, 18281, 17167, 14928, 13728, 12827, 12343, 12576, 12519, 48916]`; ink px ref/ours 112177/85743 (ratio 0.7644); SSIM blocks <0.9: 11189/30294; [overlay](images/15-three-page-sections/pdflatex-exact-export-p3-overlay.png) (46436 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-exact-export-p3-heatmap.png) (78723 B, ÷4)
  - registration error (diagnostic): global shift [0.0, -0.5] pt by ink-projection correlation (centroid estimate [-5.51, 4.69] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.3976, differing 0.155736, SSIM₈ 0.68 (raw 20.1292, 0.159717, 0.6651)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.465→0.4887 / 32.1713→31.0019; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 72.5 pt with NO reference rule on this page
- largest word displacements (pt): `oak` dx -415.72 dy 14.68; `oak` dx -415.72 dy 14.68; `oak` dx -415.72 dy 14.68

### 15-three-page-sections — pdflatex vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1595624, 24256, 21315, 20032, 17341, 18209, 18594, 16970, 17235, 18149, 16721, 15961, 17001, 16536, 16579, 88293]`; ink px ref/ours 112156/117085 (ratio 1.0439); SSIM blocks <0.9: 14325/30294; [overlay](images/15-three-page-sections/pdflatex-main-export-p1-overlay.png) (47781 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-main-export-p1-heatmap.png) (35148 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 36.5] pt by ink-projection correlation (centroid estimate [-13.9, 42.18] pt); confidence moderate (shift explains 8% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.3272, differing 0.184066, SSIM₈ 0.596 (raw 27.5995, 0.19563, 0.5345)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2679→0.3565 / 43.4492→40.3738; header-band 1.0→0.986 / 0.0→0.7294; footer-band 0.9191→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1592027, 24727, 21909, 20507, 17993, 18645, 19436, 17426, 17343, 18297, 16862, 16253, 17006, 16596, 16958, 86831]`; ink px ref/ours 112190/123031 (ratio 1.0966); SSIM blocks <0.9: 14280/30294; [overlay](images/15-three-page-sections/pdflatex-main-export-p2-overlay.png) (48996 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-main-export-p2-heatmap.png) (35539 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.47, 17.97] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.2045, differing 0.190709, SSIM₈ 0.5744 (raw 27.6711, 0.198001, 0.5413)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2671→0.3371 / 44.2259→41.1544; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.8816 / 0.0→5.0073
- page 3: |Δ| histogram (16 bins, pixel counts) `[1625955, 22340, 19955, 18569, 16150, 16962, 17030, 15580, 16014, 16211, 15217, 14637, 15137, 15070, 15296, 78693]`; ink px ref/ours 112177/99249 (ratio 0.8848); SSIM blocks <0.9: 12858/30294; [overlay](images/15-three-page-sections/pdflatex-main-export-p3-overlay.png) (44863 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-main-export-p3-heatmap.png) (33141 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -24.5] pt by ink-projection correlation (centroid estimate [-12.92, -40.65] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.6494, differing 0.171634, SSIM₈ 0.6105 (raw 24.9709, 0.178566, 0.586)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3385→0.3777 / 39.9103→37.7982; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy -63.97; `branch` dx -435.47 dy -58.38; `branch` dx -435.47 dy -52.79
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1651971, 22582, 20106, 18490, 16494, 16682, 16053, 16533, 15680, 15406, 15004, 13815, 13095, 13434, 13586, 59885]`; ink px ref/ours 112156/111942 (ratio 0.9981); SSIM blocks <0.9: 11248/30294; [overlay](images/15-three-page-sections/pdflatex-pipeline-export-p1-overlay.png) (48197 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-pipeline-export-p1-heatmap.png) (79696 B, ÷4)
  - registration error (diagnostic): global shift [1.0, -0.5] pt by ink-projection correlation (centroid estimate [-4.9, 5.76] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.5561, differing 0.159398, SSIM₈ 0.6872 (raw 21.7383, 0.164485, 0.6644)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4641→0.5031 / 34.7417→32.8178; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1651865, 22607, 20127, 18499, 16488, 16651, 16038, 16543, 15673, 15412, 15017, 13841, 13091, 13449, 13596, 59919]`; ink px ref/ours 112190/112002 (ratio 0.9983); SSIM blocks <0.9: 11252/30294; [overlay](images/15-three-page-sections/pdflatex-pipeline-export-p2-overlay.png) (48253 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-pipeline-export-p2-heatmap.png) (79810 B, ÷4)
  - registration error (diagnostic): global shift [1.0, -0.5] pt by ink-projection correlation (centroid estimate [-4.9, 5.67] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.5674, differing 0.159475, SSIM₈ 0.6871 (raw 21.7482, 0.164551, 0.6643)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4639→0.503 / 34.7576→32.8355; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1651857, 22611, 20126, 18501, 16494, 16658, 16039, 16541, 15680, 15421, 15013, 13824, 13088, 13442, 13580, 59941]`; ink px ref/ours 112177/111975 (ratio 0.9982); SSIM blocks <0.9: 11254/30294; [overlay](images/15-three-page-sections/pdflatex-pipeline-export-p3-overlay.png) (48260 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-pipeline-export-p3-heatmap.png) (79782 B, ÷4)
  - registration error (diagnostic): global shift [1.0, -0.5] pt by ink-projection correlation (centroid estimate [-4.89, 5.74] pt); confidence moderate (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.5631, differing 0.159457, SSIM₈ 0.6872 (raw 21.7479, 0.164562, 0.6642)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4638→0.5032 / 34.7571→32.8289; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 14.32; `oak` dx -415.72 dy 14.32; `oak` dx -415.72 dy 14.32
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610609, 24967, 21053, 20726, 19661, 18340, 19742, 19412, 18598, 18249, 16287, 15994, 15576, 14410, 15675, 69517]`; ink px ref/ours 85692/116938 (ratio 1.3646); SSIM blocks <0.9: 14337/30294; [overlay](images/15-three-page-sections/pdflatex-lm-de1020c-export-p1-overlay.png) (48393 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-de1020c-export-p1-heatmap.png) (35233 B, ÷8)
  - registration error (diagnostic): global shift [0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.11, 38.21] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.3365, differing 0.178246, SSIM₈ 0.5819 (raw 25.0354, 0.18832, 0.5339)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2672→0.3354 / 39.3505→37.2; header-band 1.0→0.9885 / 0.0→0.6238; footer-band 0.9183→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606782, 25413, 21423, 20910, 20004, 18862, 20556, 19676, 18675, 18670, 16345, 16142, 15704, 14246, 15727, 69681]`; ink px ref/ours 85729/122724 (ratio 1.4315); SSIM blocks <0.9: 14619/30294; [overlay](images/15-three-page-sections/pdflatex-lm-de1020c-export-p2-overlay.png) (49560 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-de1020c-export-p2-heatmap.png) (35826 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.23, 12.88] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.2389, differing 0.184974, SSIM₈ 0.5622 (raw 25.2323, 0.19048, 0.5336)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2548→0.3156 / 40.3276→38.0352; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.9087 / 0.0→4.802
- page 3: |Δ| histogram (16 bins, pixel counts) `[1641126, 23072, 19471, 19055, 17908, 16873, 18097, 18161, 17038, 16322, 14757, 14561, 14158, 12872, 14209, 61136]`; ink px ref/ours 85728/99417 (ratio 1.1597); SSIM blocks <0.9: 13069/30294; [overlay](images/15-three-page-sections/pdflatex-lm-de1020c-export-p3-overlay.png) (45620 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-de1020c-export-p3-heatmap.png) (33456 B, ÷8)
  - registration error (diagnostic): global shift [0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.13, -45.32] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.5333, differing 0.165392, SSIM₈ 0.6026 (raw 22.5169, 0.171027, 0.5804)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3295→0.3666 / 35.9875→34.4099; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy -92.02; `oak` dx 426.94 dy -86.43; `oak` dx 426.94 dy -80.84

### 15-three-page-sections — pdflatex-lm vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 85692/85708 (ratio 1.0002); SSIM blocks <0.9: 0/30294; [overlay](images/15-three-page-sections/pdflatex-lm-exact-export-p1-overlay.png) (82786 B, ÷4), [heatmap](images/15-three-page-sections/pdflatex-lm-exact-export-p1-heatmap.png) (58363 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.03, -0.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0735, differing 0.041614, SSIM₈ 1.0 (raw 0.0735, 0.041614, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.1175→0.1175; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 10.0 vs 10.0, thickness px 1 vs 1
- page 2: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 85729/85745 (ratio 1.0002); SSIM blocks <0.9: 0/30294; [overlay](images/15-three-page-sections/pdflatex-lm-exact-export-p2-overlay.png) (82892 B, ÷4), [heatmap](images/15-three-page-sections/pdflatex-lm-exact-export-p2-heatmap.png) (58453 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.02, -0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0736, differing 0.041632, SSIM₈ 1.0 (raw 0.0736, 0.041632, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.1176→0.1176; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 10.0 vs 10.0, thickness px 1 vs 1
- page 3: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 85728/85743 (ratio 1.0002); SSIM blocks <0.9: 0/30294; [overlay](images/15-three-page-sections/pdflatex-lm-exact-export-p3-overlay.png) (82885 B, ÷4), [heatmap](images/15-three-page-sections/pdflatex-lm-exact-export-p3-heatmap.png) (58451 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.02, -0.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0736, differing 0.041635, SSIM₈ 1.0 (raw 0.0736, 0.041635, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.1176→0.1176; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 10.0 vs 10.0, thickness px 1 vs 1
- largest word displacements (pt): `Part` dx 0 dy 1.64; `1` dx -0.0 dy 1.64; `Part` dx 0 dy 1.64

### 15-three-page-sections — pdflatex-lm vs compiler `main` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1610416, 25113, 21419, 20803, 19258, 18585, 19969, 19377, 18674, 17907, 15994, 15840, 15987, 14926, 15214, 69334]`; ink px ref/ours 85692/117085 (ratio 1.3663); SSIM blocks <0.9: 14317/30294; [overlay](images/15-three-page-sections/pdflatex-lm-main-export-p1-overlay.png) (48087 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-main-export-p1-heatmap.png) (34970 B, ÷8)
  - registration error (diagnostic): global shift [-0.5, 22.5] pt by ink-projection correlation (centroid estimate [-8.37, 37.41] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 23.2375, differing 0.178188, SSIM₈ 0.5842 (raw 25.0184, 0.188605, 0.5352)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2693→0.338 / 39.3229→37.0293; header-band 1.0→0.9865 / 0.0→0.7294; footer-band 0.9191→1.0 / 4.5584→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1606072, 25375, 21965, 21062, 19569, 18901, 20739, 19515, 18885, 18204, 16020, 16124, 16068, 15003, 15475, 69839]`; ink px ref/ours 85729/123031 (ratio 1.4351); SSIM blocks <0.9: 14647/30294; [overlay](images/15-three-page-sections/pdflatex-lm-main-export-p2-overlay.png) (49372 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-main-export-p2-heatmap.png) (35795 B, ÷8)
  - registration error (diagnostic): global shift [-0.5, -24.0] pt by ink-projection correlation (centroid estimate [-6.98, 13.23] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 24.1593, differing 0.185092, SSIM₈ 0.5642 (raw 25.3001, 0.191187, 0.5323)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.2529→0.3178 / 40.4354→37.8841; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→0.905 / 0.0→4.9832
- page 3: |Δ| histogram (16 bins, pixel counts) `[1641132, 22873, 19894, 18879, 17656, 17043, 18330, 17670, 17451, 16197, 14625, 14425, 14301, 13434, 13942, 60964]`; ink px ref/ours 85728/99249 (ratio 1.1577); SSIM blocks <0.9: 13044/30294; [overlay](images/15-three-page-sections/pdflatex-lm-main-export-p3-overlay.png) (45275 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-main-export-p3-heatmap.png) (33208 B, ÷8)
  - registration error (diagnostic): global shift [-0.5, -24.0] pt by ink-projection correlation (centroid estimate [-7.39, -45.37] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 21.4675, differing 0.165418, SSIM₈ 0.6033 (raw 22.5176, 0.1711, 0.5805)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3299→0.3666 / 35.9882→34.3068; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -92.02; `oak` dx 425.86 dy -86.43; `oak` dx 425.86 dy -80.84
- word-sequence differences: insert ref [] ours ['1']; insert ref [] ours ['2']; insert ref [] ours ['3']

### 15-three-page-sections — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1725690, 24520, 19956, 18982, 17953, 17203, 15287, 14563, 11761, 10977, 9181, 8377, 7907, 7107, 6759, 22593]`; ink px ref/ours 85692/111942 (ratio 1.3063); SSIM blocks <0.9: 9090/30294; [overlay](images/15-three-page-sections/pdflatex-lm-pipeline-export-p1-overlay.png) (47659 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-pipeline-export-p1-heatmap.png) (70994 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.62, 0.99] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1451, differing 0.128925, SSIM₈ 0.8176 (raw 13.1451, 0.128925, 0.8176)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7088→0.7088 / 21.0078→21.0078; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1725491, 24550, 19978, 19005, 17947, 17199, 15281, 14566, 11758, 10978, 9177, 8403, 7907, 7127, 6765, 22684]`; ink px ref/ours 85729/112002 (ratio 1.3065); SSIM blocks <0.9: 9094/30294; [overlay](images/15-three-page-sections/pdflatex-lm-pipeline-export-p2-overlay.png) (47702 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-pipeline-export-p2-heatmap.png) (71111 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.59, 0.93] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1629, differing 0.129026, SSIM₈ 0.8173 (raw 13.1629, 0.129026, 0.8173)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7084→0.7084 / 21.0362→21.0362; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1725465, 24553, 19975, 19005, 17954, 17200, 15277, 14562, 11776, 10981, 9180, 8385, 7917, 7119, 6750, 22717]`; ink px ref/ours 85728/111975 (ratio 1.3062); SSIM blocks <0.9: 9097/30294; [overlay](images/15-three-page-sections/pdflatex-lm-pipeline-export-p3-overlay.png) (47726 B, ÷8), [heatmap](images/15-three-page-sections/pdflatex-lm-pipeline-export-p3-heatmap.png) (71110 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.64, 1.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1655, differing 0.129045, SSIM₈ 0.8173 (raw 13.1655, 0.129045, 0.8173)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7083→0.7083 / 21.0403→21.0403; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Part` dx 29.05 dy 0.96; `1` dx 29.05 dy 0.96; `Part` dx 29.05 dy 0.96
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

### 15-three-page-sections — xelatex vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1659385, 23638, 20461, 19370, 18076, 17388, 17145, 17715, 16763, 15029, 13896, 13719, 12818, 12441, 12430, 48542]`; ink px ref/ours 112652/85708 (ratio 0.7608); SSIM blocks <0.9: 11499/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 43.0] pt by ink-projection correlation (centroid estimate [-6.14, 27.41] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.4514, differing 0.157221, SSIM₈ 0.6753 (raw 20.1843, 0.160695, 0.6583)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4546→0.4968 / 32.2532→30.4248; header-band 1.0→0.8965 / 0.0→4.5209; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 72.5 pt with NO reference rule on this page
- page 2: |Δ| histogram (16 bins, pixel counts) `[1659263, 23637, 20483, 19381, 18086, 17395, 17155, 17722, 16765, 15028, 13899, 13721, 12824, 12425, 12442, 48590]`; ink px ref/ours 112671/85745 (ratio 0.761); SSIM blocks <0.9: 11502/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 43.0] pt by ink-projection correlation (centroid estimate [-6.19, 27.3] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.4629, differing 0.157294, SSIM₈ 0.6751 (raw 20.194, 0.160766, 0.6582)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4544→0.4967 / 32.2687→30.4333; header-band 1.0→0.8947 / 0.0→4.5892; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 72.5 pt with NO reference rule on this page
- page 3: |Δ| histogram (16 bins, pixel counts) `[1659270, 23635, 20478, 19386, 18086, 17401, 17157, 17716, 16782, 15032, 13903, 13720, 12826, 12420, 12417, 48587]`; ink px ref/ours 112666/85743 (ratio 0.761); SSIM blocks <0.9: 11506/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 43.0] pt by ink-projection correlation (centroid estimate [-6.25, 27.37] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 19.4592, differing 0.157286, SSIM₈ 0.6751 (raw 20.192, 0.160767, 0.6582)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4543→0.4967 / 32.2655→30.4286; header-band 1.0→0.8951 / 0.0→4.5807; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.0 at y 72.5 pt with NO reference rule on this page
- largest word displacements (pt): `oak` dx -415.83 dy 58.09; `oak` dx -415.83 dy 58.09; `oak` dx -415.83 dy 58.09

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1648740, 22971, 19514, 18408, 16977, 17275, 16322, 16190, 15586, 15692, 14901, 14690, 13809, 13358, 13650, 60733]`; ink px ref/ours 112652/111942 (ratio 0.9937); SSIM blocks <0.9: 11599/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 14.0] pt by ink-projection correlation (centroid estimate [-5.55, 28.44] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.9523, differing 0.161509, SSIM₈ 0.6762 (raw 22.0444, 0.166142, 0.655)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4494→0.4879 / 35.2253→33.3361; header-band 1.0→0.9859 / 0.0→0.7482; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1648612, 22993, 19536, 18420, 16980, 17258, 16308, 16206, 15579, 15705, 14906, 14691, 13818, 13352, 13668, 60784]`; ink px ref/ours 112671/112002 (ratio 0.9941); SSIM blocks <0.9: 11602/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 14.0] pt by ink-projection correlation (centroid estimate [-5.62, 28.28] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.963, differing 0.16158, SSIM₈ 0.6761 (raw 22.0554, 0.166211, 0.6548)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4492→0.4879 / 35.2428→33.3417; header-band 1.0→0.9849 / 0.0→0.8282; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1648593, 22994, 19533, 18423, 16983, 17271, 16316, 16201, 15580, 15706, 14906, 14684, 13816, 13347, 13650, 60813]`; ink px ref/ours 112666/111975 (ratio 0.9939); SSIM blocks <0.9: 11605/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.0, 14.0] pt by ink-projection correlation (centroid estimate [-5.63, 28.42] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 20.959, differing 0.161574, SSIM₈ 0.676 (raw 22.0567, 0.166227, 0.6547)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.4491→0.4877 / 35.2449→33.3397; header-band 1.0→0.9854 / 0.0→0.7969; footer-band 1.0→1.0 / 0.0→0.0
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

### 15-three-page-sections — xelatex-lm vs compiler `exact` (export)

- FlashTeX diagnostics (2): error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented; error: \newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented
- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938527, 289, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 85700/85708 (ratio 1.0001); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.03, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3529, differing 0.077706, SSIM₈ 0.9996 (raw 0.3529, 0.077706, 0.9996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9993→0.9993 / 0.564→0.564; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 10.0 vs 10.0, thickness px 1 vs 1
- page 2: |Δ| histogram (16 bins, pixel counts) `[1938544, 272, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 85737/85745 (ratio 1.0001); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.06, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.353, differing 0.077733, SSIM₈ 0.9996 (raw 0.353, 0.077733, 0.9996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9993→0.9993 / 0.5642→0.5642; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 10.0 vs 10.0, thickness px 1 vs 1
- page 3: |Δ| histogram (16 bins, pixel counts) `[1938536, 280, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 85734/85743 (ratio 1.0001); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.04, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.3531, differing 0.077743, SSIM₈ 0.9996 (raw 0.3531, 0.077743, 0.9996)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9993→0.9993 / 0.5643→0.5643; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 10.0 vs 10.0, thickness px 1 vs 1
- largest word displacements (pt): `1` dx -0.03 dy 1.64; `2` dx -0.03 dy 1.64; `3` dx -0.03 dy 1.64

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1725768, 24597, 19971, 18667, 18068, 16865, 15457, 14619, 11943, 11027, 9159, 8471, 7983, 6968, 6592, 22661]`; ink px ref/ours 85700/111942 (ratio 1.3062); SSIM blocks <0.9: 9091/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.62, 1.02] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1328, differing 0.128731, SSIM₈ 0.8176 (raw 13.1328, 0.128731, 0.8176)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7088→0.7088 / 20.9881→20.9881; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 2: |Δ| histogram (16 bins, pixel counts) `[1725582, 24612, 19996, 18688, 18062, 16865, 15448, 14623, 11940, 11026, 9157, 8493, 7987, 6988, 6613, 22736]`; ink px ref/ours 85737/112002 (ratio 1.3063); SSIM blocks <0.9: 9095/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.64, 0.99] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.1504, differing 0.128832, SSIM₈ 0.8174 (raw 13.1504, 0.128832, 0.8174)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7084→0.7084 / 21.0163→21.0163; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- page 3: |Δ| histogram (16 bins, pixel counts) `[1725556, 24616, 19997, 18685, 18068, 16859, 15450, 14619, 11951, 11037, 9158, 8480, 7993, 6979, 6599, 22769]`; ink px ref/ours 85734/111975 (ratio 1.3061); SSIM blocks <0.9: 9098/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.66, 1.06] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 13.153, differing 0.128853, SSIM₈ 0.8173 (raw 13.153, 0.128853, 0.8173)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7083→0.7083 / 21.0204→21.0204; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Part` dx 29.05 dy 0.96; `Part` dx 29.05 dy 0.96; `Part` dx 29.05 dy 0.96
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

### 16-heading-page-break — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1590288, 30354, 26518, 24126, 22843, 20824, 21659, 20925, 20664, 18454, 16954, 16211, 16153, 15866, 15704, 61273]`; ink px ref/ours 152381/105096 (ratio 0.6897); SSIM blocks <0.9: 14470/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.81, -0.73] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.1489, differing 0.201659, SSIM₈ 0.5866 (raw 25.1489, 0.201659, 0.5866)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3408→0.3408 / 40.1703→40.1703; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9948→0.9948 / 0.1184→0.1184
- page 2: |Δ| histogram (16 bins, pixel counts) `[1840505, 8140, 6735, 6834, 5764, 5848, 5578, 5987, 5508, 4899, 4723, 4649, 4732, 4526, 4472, 19916]`; ink px ref/ours 29740/33457 (ratio 1.125); SSIM blocks <0.9: 4940/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 58.0] pt by ink-projection correlation (centroid estimate [4.51, 31.03] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.1242, differing 0.055179, SSIM₈ 0.8629 (raw 7.3533, 0.056786, 0.8499)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7605→0.8387 / 11.7504→9.2165; header-band 1.0→0.6042 / 0.0→14.9203; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.5 at y 247.5 pt with NO reference rule on this page
- largest word displacements (pt): `oak` dx -415.78 dy 86.28; `branch` dx -413.61 dy 86.28; `oak` dx -415.78 dy 58.55

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1582337, 30713, 26245, 23275, 21832, 19918, 19505, 19322, 18915, 18230, 17487, 16985, 16381, 16218, 16797, 74656]`; ink px ref/ours 152381/137086 (ratio 0.8996); SSIM blocks <0.9: 14334/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 26.7936, not lower; centroid estimate [-1.56, -1.23] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.7588, differing 0.206058, SSIM₈ 0.5916 (raw 26.7588, 0.206058, 0.5916)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3491→0.3491 / 42.7375→42.7375; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9954→0.9954 / 0.1454→0.1454
- page 2: |Δ| histogram (16 bins, pixel counts) `[1835127, 7846, 6580, 6109, 5814, 5420, 5277, 5600, 5006, 5060, 4965, 4940, 4875, 4616, 4825, 26756]`; ink px ref/ours 29740/42783 (ratio 1.4386); SSIM blocks <0.9: 4837/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 57.5] pt by ink-projection correlation (centroid estimate [7.61, 30.13] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.9948, differing 0.057751, SSIM₈ 0.8603 (raw 8.26, 0.059499, 0.8509)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.762→0.8335 / 13.1987→10.0379; header-band 1.0→0.6129 / 0.0→18.8083; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.78 dy 85.92; `branch` dx -413.61 dy 85.92; `oak` dx -415.78 dy 58.19
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

### 16-heading-page-break — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 104967/105096 (ratio 1.0012); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.12, 0.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2525, differing 0.074948, SSIM₈ 0.9999 (raw 0.2525, 0.074948, 0.9999)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9998→0.9998 / 0.4034→0.4034; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0014→0.0014
- page 2: |Δ| histogram (16 bins, pixel counts) `[1936289, 2527, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 33407/33457 (ratio 1.0015); SSIM blocks <0.9: 11/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.37, 0.13] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.128, differing 0.025656, SSIM₈ 0.9997 (raw 0.128, 0.025656, 0.9997)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9995→0.9995 / 0.2045→0.2045; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 10.5 vs 10.5, thickness px 1 vs 1
- largest word displacements (pt): `old` dx 0.03 dy -0.03; `over` dx 0.02 dy -0.03; `patient` dx 0.02 dy -0.03

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1679427, 30109, 24663, 22687, 22717, 19885, 18563, 17399, 14725, 13531, 10970, 10329, 9758, 9023, 7799, 27231]`; ink px ref/ours 104967/137086 (ratio 1.306); SSIM blocks <0.9: 11034/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.63, -0.46] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9631, differing 0.157284, SSIM₈ 0.7777 (raw 15.9631, 0.157284, 0.7777)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6456→0.6456 / 25.4983→25.4983; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9982→0.9982 / 0.0799→0.0799
- page 2: |Δ| histogram (16 bins, pixel counts) `[1856186, 9326, 7680, 7032, 7113, 6227, 5850, 5485, 4585, 4243, 3491, 3264, 3272, 2775, 2503, 9784]`; ink px ref/ours 33407/42783 (ratio 1.2807); SSIM blocks <0.9: 3545/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.73, -0.78] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 5.1915, differing 0.049869, SSIM₈ 0.9273 (raw 5.1915, 0.049869, 0.9273)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8841→0.8841 / 8.2962→8.2962; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Orphan` dx 29.05 dy -0.69; `Heading` dx 29.04 dy -0.69; `old` dx 0.03 dy -0.39
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1536790, 30381, 25973, 23758, 22440, 22370, 21997, 20676, 20267, 21593, 19997, 19234, 19069, 18240, 19686, 96345]`; ink px ref/ours 151753/134585 (ratio 0.8869); SSIM blocks <0.9: 16162/30294; [overlay](images/16-heading-page-break/pdflatex-de1020c-export-p1-overlay.png) (56459 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-de1020c-export-p1-heatmap.png) (38239 B, ÷8)
  - registration error (diagnostic): global shift [0.0, -14.5] pt by ink-projection correlation (centroid estimate [-6.07, -5.06] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 31.4928, differing 0.229367, SSIM₈ 0.4891 (raw 31.5764, 0.22951, 0.4881)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1821→0.1892 / 50.4615→50.1371; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.96 / 0.0468→1.3594
- page 2: |Δ| histogram (16 bins, pixel counts) `[1821555, 8081, 6675, 6513, 5830, 5892, 6026, 5614, 5619, 6564, 5700, 5695, 5919, 5335, 5734, 32064]`; ink px ref/ours 29765/46935 (ratio 1.5769); SSIM blocks <0.9: 5085/30294; [overlay](images/16-heading-page-break/pdflatex-de1020c-export-p2-overlay.png) (53017 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-de1020c-export-p2-heatmap.png) (47439 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 20.5] pt by ink-projection correlation (centroid estimate [1.21, 55.26] pt); confidence moderate (shift explains 11% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.5969, differing 0.061448, SSIM₈ 0.8517 (raw 9.6191, 0.066682, 0.8358)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7375→0.764 / 15.3741→13.6882; header-band 1.0→0.9933 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 127.11; `branch` dx -435.47 dy 74.69; `branch` dx -435.47 dy 54.75

### 16-heading-page-break — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1591651, 30518, 26558, 23731, 22710, 21180, 21265, 21374, 20408, 18257, 17018, 15784, 15591, 15111, 15517, 62143]`; ink px ref/ours 151753/105096 (ratio 0.6925); SSIM blocks <0.9: 14421/30294; [overlay](images/16-heading-page-break/pdflatex-exact-export-p1-overlay.png) (55793 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-exact-export-p1-heatmap.png) (35098 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-1.56, -1.74] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.0309, differing 0.200674, SSIM₈ 0.5888 (raw 25.0309, 0.200674, 0.5888)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3441→0.3441 / 39.9869→39.9869; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9945→0.9945 / 0.1214→0.1214
- page 2: |Δ| histogram (16 bins, pixel counts) `[1840538, 7997, 7099, 6521, 6132, 6049, 5732, 5907, 5676, 4881, 4676, 4328, 4578, 4311, 4529, 19862]`; ink px ref/ours 29765/33457 (ratio 1.124); SSIM blocks <0.9: 4911/30294; [overlay](images/16-heading-page-break/pdflatex-exact-export-p2-overlay.png) (52608 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-exact-export-p2-heatmap.png) (43916 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 58.0] pt by ink-projection correlation (centroid estimate [3.99, 30.26] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.1471, differing 0.05506, SSIM₈ 0.864 (raw 7.3085, 0.056509, 0.8517)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7632→0.8402 / 11.6803→9.2546; header-band 1.0→0.6052 / 0.0→14.9203; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.5 at y 247.5 pt with NO reference rule on this page
- largest word displacements (pt): `oak` dx -415.72 dy 86.09; `branch` dx -413.61 dy 86.09; `oak` dx -415.72 dy 58.55

### 16-heading-page-break — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1520838, 31563, 27074, 24807, 22509, 23307, 22498, 21004, 21495, 22304, 20775, 19954, 20253, 19497, 19992, 100946]`; ink px ref/ours 151753/145750 (ratio 0.9604); SSIM blocks <0.9: 16544/30294; [overlay](images/16-heading-page-break/pdflatex-main-export-p1-overlay.png) (57437 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-main-export-p1-heatmap.png) (38477 B, ÷8)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-11.0, 0.0] pt REJECTED: applying it gives mean|Δ| 32.933, not lower; centroid estimate [-4.21, -3.98] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 32.9189, differing 0.238789, SSIM₈ 0.4737 (raw 32.9189, 0.238789, 0.4737)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.1593→0.1593 / 52.6066→52.6066; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9984→0.9984 / 0.0468→0.0468
- page 2: |Δ| histogram (16 bins, pixel counts) `[1846805, 6715, 5911, 5450, 4855, 4877, 4776, 4476, 4610, 4607, 4225, 4310, 4650, 4145, 4422, 23982]`; ink px ref/ours 29765/36694 (ratio 1.2328); SSIM blocks <0.9: 4055/30294; [overlay](images/16-heading-page-break/pdflatex-main-export-p2-overlay.png) (49263 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-main-export-p2-heatmap.png) (40517 B, ÷4)
  - registration error (diagnostic): global shift [-0.5, 14.5] pt by ink-projection correlation (centroid estimate [1.5, 12.9] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.1674, differing 0.05157, SSIM₈ 0.8836 (raw 7.3756, 0.052561, 0.8765)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8026→0.8275 / 11.7883→10.7329; header-band 1.0→0.9077 / 0.0→4.9674; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `branch` dx -435.47 dy 49.11; `branch` dx -435.47 dy 37.03; `branch` dx -435.47 dy 31.49
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1582932, 30741, 25916, 23000, 21677, 20174, 19443, 19715, 18492, 18092, 17570, 17152, 15948, 15841, 16452, 75671]`; ink px ref/ours 151753/137086 (ratio 0.9033); SSIM blocks <0.9: 14233/30294; [overlay](images/16-heading-page-break/pdflatex-pipeline-export-p1-overlay.png) (57788 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-pipeline-export-p1-heatmap.png) (36359 B, ÷8)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-2.31, -2.24] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.6378, differing 0.2051, SSIM₈ 0.5954 (raw 26.7553, 0.205472, 0.5933)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3515→0.3576 / 42.7368→42.4844; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9951→0.9953 / 0.1484→0.1484
- page 2: |Δ| histogram (16 bins, pixel counts) `[1835119, 7575, 6768, 5770, 6049, 5593, 5452, 5606, 5215, 5179, 4878, 4732, 4899, 4524, 4850, 26607]`; ink px ref/ours 29765/42783 (ratio 1.4374); SSIM blocks <0.9: 4813/30294; [overlay](images/16-heading-page-break/pdflatex-pipeline-export-p2-overlay.png) (52334 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-pipeline-export-p2-heatmap.png) (43804 B, ÷4)
  - registration error (diagnostic): global shift [-1.5, 57.5] pt by ink-projection correlation (centroid estimate [7.09, 29.35] pt); confidence weak (shift explains 5% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.8733, differing 0.057341, SSIM₈ 0.8637 (raw 8.2458, 0.059217, 0.8523)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7642→0.8388 / 13.1776→9.8354; header-band 1.0→0.6154 / 0.0→18.8083; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.72 dy 85.73; `branch` dx -413.61 dy 85.73; `oak` dx -415.72 dy 58.19
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1571374, 28848, 24688, 23563, 22599, 21205, 23080, 21890, 21176, 20559, 18462, 17621, 17144, 16082, 17481, 73044]`; ink px ref/ours 105111/134585 (ratio 1.2804); SSIM blocks <0.9: 15666/30294; [overlay](images/16-heading-page-break/pdflatex-lm-de1020c-export-p1-overlay.png) (54964 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-de1020c-export-p1-heatmap.png) (37515 B, ÷8)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [-4.44, -3.32] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 27.4403, differing 0.210395, SSIM₈ 0.5038 (raw 27.5467, 0.210674, 0.5026)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.206→0.208 / 44.0147→43.8447; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.9959 / 0.0747→0.0747
- page 2: |Δ| histogram (16 bins, pixel counts) `[1804006, 9924, 8335, 8540, 7532, 7526, 7743, 7505, 7333, 7253, 6797, 6490, 6663, 5901, 6567, 30701]`; ink px ref/ours 33505/46935 (ratio 1.4008); SSIM blocks <0.9: 6195/30294; [overlay](images/16-heading-page-break/pdflatex-lm-de1020c-export-p2-overlay.png) (59720 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-de1020c-export-p2-heatmap.png) (52927 B, ÷4)
  - registration error (diagnostic): global shift [-1.0, 20.0] pt by ink-projection correlation (centroid estimate [-2.85, 25.0] pt); confidence moderate (shift explains 17% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.6575, differing 0.066846, SSIM₈ 0.8469 (raw 10.4882, 0.077169, 0.7985)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6782→0.7566 / 16.7624→13.7844; header-band 1.0→0.9928 / 0.0→0.3586; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 426.94 dy 27.65; `oak` dx 426.94 dy 21.13; `oak` dx 426.94 dy -13.72

### 16-heading-page-break — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 105111/105096 (ratio 0.9999); SSIM blocks <0.9: 0/30294; [overlay](images/16-heading-page-break/pdflatex-lm-exact-export-p1-overlay.png) (35972 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-exact-export-p1-heatmap.png) (64162 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.07, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0879, differing 0.049656, SSIM₈ 1.0 (raw 0.0879, 0.049656, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.1404→0.1404; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0005→0.0005
- page 2: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 33505/33457 (ratio 0.9986); SSIM blocks <0.9: 0/30294; [overlay](images/16-heading-page-break/pdflatex-lm-exact-export-p2-overlay.png) (44096 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-exact-export-p2-heatmap.png) (39710 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.08, -0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0268, differing 0.015165, SSIM₈ 1.0 (raw 0.0268, 0.015165, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0428→0.0428; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 10.5 vs 10.5, thickness px 1 vs 1
- largest word displacements (pt): `Orphan` dx 0 dy 1.64; `Heading` dx 0.0 dy 1.64; `quick` dx 0.01 dy 1.03

### 16-heading-page-break — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1555521, 30291, 26058, 24656, 22827, 22307, 23426, 22015, 22563, 21069, 18990, 18590, 18306, 17279, 17582, 77336]`; ink px ref/ours 105111/145750 (ratio 1.3866); SSIM blocks <0.9: 16299/30294; [overlay](images/16-heading-page-break/pdflatex-lm-main-export-p1-overlay.png) (56037 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-main-export-p1-heatmap.png) (38055 B, ÷8)
  - registration error (diagnostic): global shift [-0.5, -27.0] pt by ink-projection correlation (centroid estimate [-2.58, -2.25] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 28.7887, differing 0.220226, SSIM₈ 0.482 (raw 28.8184, 0.220067, 0.4825)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.174→0.198 / 46.0467→44.613; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9959→0.8271 / 0.0747→9.5919
- page 2: |Δ| histogram (16 bins, pixel counts) `[1833836, 8612, 7357, 7269, 6381, 6293, 6217, 6112, 5814, 5336, 5154, 4953, 4927, 4585, 4899, 21071]`; ink px ref/ours 33505/36694 (ratio 1.0952); SSIM blocks <0.9: 4934/30294; [overlay](images/16-heading-page-break/pdflatex-lm-main-export-p2-overlay.png) (55486 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-main-export-p2-heatmap.png) (44282 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 7.8298, not lower; centroid estimate [-2.57, -17.36] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.819, differing 0.060445, SSIM₈ 0.8514 (raw 7.819, 0.060445, 0.8514)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7628→0.7628 / 12.4961→12.4961; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx 425.86 dy -50.35; `oak` dx 425.86 dy -40.43; `oak` dx 425.86 dy -31.53
- word-sequence differences: insert ref [] ours ['1']

### 16-heading-page-break — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1679192, 29965, 24493, 23156, 22217, 20414, 18450, 17405, 14935, 13089, 11268, 10162, 9717, 8951, 8308, 27094]`; ink px ref/ours 105111/137086 (ratio 1.3042); SSIM blocks <0.9: 11030/30294; [overlay](images/16-heading-page-break/pdflatex-lm-pipeline-export-p1-overlay.png) (54476 B, ÷8), [heatmap](images/16-heading-page-break/pdflatex-lm-pipeline-export-p1-heatmap.png) (82320 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.69, -0.5] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9957, differing 0.157351, SSIM₈ 0.7772 (raw 15.9957, 0.157351, 0.7772)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6448→0.6448 / 25.5504→25.5504; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9982→0.9982 / 0.0807→0.0807
- page 2: |Δ| histogram (16 bins, pixel counts) `[1856144, 9310, 7630, 7264, 6919, 6455, 5694, 5598, 4550, 4099, 3580, 3195, 3230, 2767, 2608, 9773]`; ink px ref/ours 33505/42783 (ratio 1.2769); SSIM blocks <0.9: 3557/30294; [overlay](images/16-heading-page-break/pdflatex-lm-pipeline-export-p2-overlay.png) (58684 B, ÷4), [heatmap](images/16-heading-page-break/pdflatex-lm-pipeline-export-p2-heatmap.png) (38538 B, ÷4)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [3.02, -0.92] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 5.1952, differing 0.049905, SSIM₈ 0.9274 (raw 5.1952, 0.049905, 0.9274)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.8841→0.8841 / 8.302→8.302; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Orphan` dx 29.05 dy 0.96; `Heading` dx 29.05 dy 0.96; `quick` dx 0.01 dy 0.67
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

### 16-heading-page-break — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1589554, 30035, 26334, 24859, 22978, 21404, 21766, 21185, 20523, 18166, 17141, 16749, 15769, 15639, 15257, 61457]`; ink px ref/ours 152232/105096 (ratio 0.6904); SSIM blocks <0.9: 14476/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.85, -0.69] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 25.1569, differing 0.20198, SSIM₈ 0.586 (raw 25.1569, 0.20198, 0.586)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.34→0.34 / 40.1829→40.1829; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9945→0.9945 / 0.1185→0.1185
- page 2: |Δ| histogram (16 bins, pixel counts) `[1840520, 8089, 6694, 6844, 5951, 5991, 5509, 5915, 5356, 4942, 4822, 4850, 4590, 4417, 4471, 19855]`; ink px ref/ours 29777/33457 (ratio 1.1236); SSIM blocks <0.9: 4941/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 58.0] pt by ink-projection correlation (centroid estimate [4.45, 31.1] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 7.1029, differing 0.055137, SSIM₈ 0.8631 (raw 7.3425, 0.056748, 0.8501)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7607→0.839 / 11.7331→9.1825; header-band 1.0→0.6042 / 0.0→14.9203; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): FlashTeX rule len 10.5 at y 247.5 pt with NO reference rule on this page
- largest word displacements (pt): `oak` dx -415.83 dy 86.28; `branch` dx -413.66 dy 86.28; `oak` dx -415.83 dy 58.55

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1581514, 30310, 25655, 23625, 21884, 20614, 19810, 19511, 18729, 18276, 17709, 17748, 16267, 16096, 16284, 74784]`; ink px ref/ours 152232/137086 (ratio 0.9005); SSIM blocks <0.9: 14338/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [1.5, 0.0] pt by ink-projection correlation (centroid estimate [-1.6, -1.19] pt); confidence weak (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 26.7219, differing 0.206537, SSIM₈ 0.591 (raw 26.8247, 0.206341, 0.5903)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.3471→0.3519 / 42.8427→42.5843; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9952→0.9948 / 0.1455→0.1455
- page 2: |Δ| histogram (16 bins, pixel counts) `[1835123, 7797, 6623, 6101, 5814, 5554, 5143, 5549, 4926, 5103, 5115, 5114, 4829, 4499, 4911, 26615]`; ink px ref/ours 29777/42783 (ratio 1.4368); SSIM blocks <0.9: 4834/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 57.5] pt by ink-projection correlation (centroid estimate [7.55, 30.19] pt); confidence weak (shift explains 3% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 8.0022, differing 0.05783, SSIM₈ 0.8601 (raw 8.2562, 0.059456, 0.8509)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.7621→0.8333 / 13.1926→10.0496; header-band 1.0→0.6129 / 0.0→18.8083; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `oak` dx -415.83 dy 85.92; `branch` dx -413.66 dy 85.92; `oak` dx -415.83 dy 58.19
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

### 16-heading-page-break — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 105020/105096 (ratio 1.0007); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.05, 0.11] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.2647, differing 0.074223, SSIM₈ 0.9998 (raw 0.2647, 0.074223, 0.9998)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9997→0.9997 / 0.4228→0.4228; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.001→0.001
- page 2: |Δ| histogram (16 bins, pixel counts) `[1936230, 2586, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 33418/33457 (ratio 1.0012); SSIM blocks <0.9: 11/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.32, 0.13] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.1323, differing 0.025657, SSIM₈ 0.9997 (raw 0.1323, 0.025657, 0.9997)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9995→0.9995 / 0.2115→0.2115; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
  - rules (FlashTeX → nearest reference ink row, pt): Δx 0.0 Δy 0.0 len 10.5 vs 10.5, thickness px 1 vs 1
- largest word displacements (pt): `Heading` dx -0.01 dy 1.62; `Orphan` dx 0 dy 1.62; `over` dx 0.02 dy 1.03

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
- page 1: |Δ| histogram (16 bins, pixel counts) `[1679357, 30124, 24638, 22660, 22616, 20044, 18589, 17414, 14674, 13544, 11017, 10313, 9730, 9018, 7823, 27255]`; ink px ref/ours 105020/137086 (ratio 1.3053); SSIM blocks <0.9: 11038/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.71, -0.39] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 15.9674, differing 0.157256, SSIM₈ 0.7776 (raw 15.9674, 0.157256, 0.7776)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.6454→0.6454 / 25.5052→25.5052; header-band 1.0→1.0 / 0.0→0.0; footer-band 0.9982→0.9982 / 0.0801→0.0801
- page 2: |Δ| histogram (16 bins, pixel counts) `[1856159, 9332, 7690, 6998, 7099, 6274, 5836, 5503, 4572, 4243, 3512, 3271, 3241, 2782, 2517, 9787]`; ink px ref/ours 33418/42783 (ratio 1.2802); SSIM blocks <0.9: 3548/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [2.78, -0.78] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 5.1929, differing 0.04986, SSIM₈ 0.9273 (raw 5.1929, 0.04986, 0.9273)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.884→0.884 / 8.2984→8.2984; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `Orphan` dx 29.05 dy 0.94; `Heading` dx 29.04 dy 0.94; `over` dx 0.02 dy 0.67
- word-sequence differences: insert ref [] ours ['1']

### 17-apostrophes — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927391, 1130, 902, 756, 663, 601, 596, 579, 591, 504, 530, 551, 566, 535, 531, 2390]`; ink px ref/ours 5219/5099 (ratio 0.977); SSIM blocks <0.9: 459/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-5.57, 0.01] pt); confidence weak (shift explains 4% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8119, differing 0.00674, SSIM₈ 0.9882 (raw 0.8502, 0.006811, 0.9875)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9801→0.9811 / 1.3589→1.2977; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -2.93 dy 0.42; `quotes,` dx -1.49 dy 0.42; `and` dx -1.35 dy 0.42
- word-sequence differences: replace ref ['It’stheowl’sbranch;don’t,can’t,won’t,o’clock,rock’n’roll,the’90s,Muller’sresume,“quoted”'] ours ["It's", 'the', "owl's", 'branch;', "don't,", "can't,", "won't,", "o'clock,"]; replace ref ['‘single’'] ours ["single'"]; replace ref ['’'] ours ["'"]

### 17-apostrophes — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926580, 1068, 912, 789, 761, 667, 774, 822, 740, 668, 589, 553, 639, 544, 577, 2133]`; ink px ref/ours 5219/3896 (ratio 0.7465); SSIM blocks <0.9: 564/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-12.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.9055, not lower; centroid estimate [8.07, 1.11] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8901, differing 0.007167, SSIM₈ 0.9843 (raw 0.8901, 0.007167, 0.9843)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.975→0.975 / 1.4227→1.4227; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx 62.57 dy 0.77; `’` dx 62.39 dy 0.77; `plain` dx 60.01 dy 0.77
- word-sequence differences: replace ref ['It’stheowl’sbranch;don’t,can’t,won’t,o’clock,rock’n’roll,the’90s,Muller’sresume,“quoted”'] ours ['It’s', 'the', 'owl’s', 'branch;', 'don’t,', 'can’t,', 'won’t,', 'o’clock,']

### 17-apostrophes — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927470, 1158, 889, 729, 618, 585, 586, 551, 613, 500, 573, 515, 560, 474, 531, 2464]`; ink px ref/ours 5219/5222 (ratio 1.0006); SSIM blocks <0.9: 453/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-7.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.81, 0.07] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8284, differing 0.006836, SSIM₈ 0.9876 (raw 0.8494, 0.00683, 0.9875)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.98→0.9803 / 1.3576→1.3223; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -3.05 dy 0.42; `quotes,` dx -1.61 dy 0.42; `and` dx -1.47 dy 0.42
- word-sequence differences: replace ref ['It’stheowl’sbranch;don’t,can’t,won’t,o’clock,rock’n’roll,the’90s,Muller’sresume,“quoted”'] ours ["It's", 'the', "owl's", 'branch;', "don't,", "can't,", "won't,", "o'clock,"]; replace ref ['‘single’'] ours ["single'"]; replace ref ['’'] ours ["'"]

### 17-apostrophes — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926470, 1101, 890, 778, 651, 683, 615, 664, 568, 667, 652, 607, 660, 556, 636, 2618]`; ink px ref/ours 5219/5180 (ratio 0.9925); SSIM blocks <0.9: 541/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [53.5, 0.0] pt by ink-projection correlation (centroid estimate [9.44, 0.95] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8847, differing 0.007148, SSIM₈ 0.986 (raw 0.9409, 0.007291, 0.9852)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9764→0.9805 / 1.5039→1.2691; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx 62.57 dy 0.41; `’` dx 62.39 dy 0.41; `plain` dx 60.01 dy 0.41
- word-sequence differences: replace ref ['It’stheowl’sbranch;don’t,can’t,won’t,o’clock,rock’n’roll,the’90s,Muller’sresume,“quoted”'] ours ['It’s', 'the', 'owl’s', 'branch;', 'don’t,', 'can’t,', 'won’t,', 'o’clock,']

### 17-apostrophes — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926769, 1003, 928, 840, 817, 779, 749, 812, 699, 601, 585, 566, 552, 538, 509, 2069]`; ink px ref/ours 3891/5099 (ratio 1.3105); SSIM blocks <0.9: 564/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.57, -1.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8622, differing 0.007037, SSIM₈ 0.9848 (raw 0.8622, 0.007037, 0.9848)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9757→0.9757 / 1.378→1.378; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.51 dy -0.35; `plain` dx -61.26 dy -0.35; `a` dx -59.83 dy -0.35
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 3891/3896 (ratio 1.0013); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.08, -0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0119, differing 0.002929, SSIM₈ 1.0 (raw 0.0119, 0.002929, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0191→0.0191; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `the` dx 0.01 dy 0.0; `owl’s` dx 0.01 dy 0.0; `branch;` dx 0.01 dy 0.0

### 17-apostrophes — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926664, 1040, 842, 822, 792, 771, 791, 717, 808, 626, 650, 592, 590, 544, 535, 2032]`; ink px ref/ours 3891/5222 (ratio 1.3421); SSIM blocks <0.9: 567/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [12.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8967, not lower; centroid estimate [-11.81, -1.08] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8761, differing 0.007071, SSIM₈ 0.9846 (raw 0.8761, 0.007071, 0.9846)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9754→0.9754 / 1.4002→1.4002; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.63 dy -0.35; `plain` dx -61.38 dy -0.35; `a` dx -59.95 dy -0.35
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1928614, 974, 818, 812, 815, 749, 704, 571, 683, 589, 537, 487, 417, 391, 359, 1296]`; ink px ref/ours 3891/5180 (ratio 1.3313); SSIM blocks <0.9: 459/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.44, -0.21] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.6324, differing 0.005978, SSIM₈ 0.99 (raw 0.6778, 0.006099, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9833→0.984 / 1.0834→1.0108; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `the` dx 0.01 dy -0.36; `owl’s` dx 0.01 dy -0.36; `branch;` dx 0.01 dy -0.36

### 17-apostrophes — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927689, 1087, 914, 809, 742, 653, 583, 605, 537, 460, 552, 502, 496, 521, 444, 2222]`; ink px ref/ours 5173/5099 (ratio 0.9857); SSIM blocks <0.9: 463/30294; [overlay](images/17-apostrophes/pdflatex-de1020c-export-p1-overlay.png) (41365 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-de1020c-export-p1-heatmap.png) (86591 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-7.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8459, not lower; centroid estimate [-5.05, -0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8063, differing 0.006638, SSIM₈ 0.9887 (raw 0.8063, 0.006638, 0.9887)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9819→0.9819 / 1.2887→1.2887; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -7.83 dy 0.46; `roll,` dx -6.97 dy 0.46; `the` dx -6.5 dy 0.46
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926711, 1082, 815, 841, 767, 730, 704, 775, 737, 679, 582, 593, 537, 563, 518, 2182]`; ink px ref/ours 5173/3896 (ratio 0.7531); SSIM blocks <0.9: 563/30294; [overlay](images/17-apostrophes/pdflatex-exact-export-p1-overlay.png) (43678 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-exact-export-p1-heatmap.png) (89985 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [8.6, 1.07] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8805, differing 0.007078, SSIM₈ 0.9847 (raw 0.8805, 0.007078, 0.9847)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9755→0.9755 / 1.4073→1.4073; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `“quoted”` dx -424.84 dy 15.21; `apostrophe.` dx 62.67 dy 0.77; `’` dx 62.49 dy 0.77

### 17-apostrophes — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927557, 1042, 927, 864, 724, 650, 665, 624, 530, 509, 592, 484, 478, 492, 471, 2207]`; ink px ref/ours 5173/5222 (ratio 1.0095); SSIM blocks <0.9: 461/30294; [overlay](images/17-apostrophes/pdflatex-main-export-p1-overlay.png) (40829 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-main-export-p1-heatmap.png) (87141 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-8.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8537, not lower; centroid estimate [-3.29, 0.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8138, differing 0.006695, SSIM₈ 0.9885 (raw 0.8138, 0.006695, 0.9885)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9817→0.9817 / 1.3007→1.3007; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -8.25 dy 0.46; `roll,` dx -7.39 dy 0.46; `the` dx -6.92 dy 0.46
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926563, 1012, 841, 796, 721, 729, 597, 599, 591, 678, 651, 622, 596, 577, 611, 2632]`; ink px ref/ours 5173/5180 (ratio 1.0014); SSIM blocks <0.9: 540/30294; [overlay](images/17-apostrophes/pdflatex-pipeline-export-p1-overlay.png) (41569 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-pipeline-export-p1-heatmap.png) (89035 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-7.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.9741, not lower; centroid estimate [9.96, 0.9] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9369, differing 0.007233, SSIM₈ 0.9856 (raw 0.9369, 0.007233, 0.9856)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.977→0.977 / 1.4974→1.4974; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `“quoted”` dx -424.84 dy 14.85; `apostrophe.` dx 62.67 dy 0.41; `’` dx 62.49 dy 0.41

### 17-apostrophes — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926770, 1005, 918, 847, 826, 791, 712, 833, 730, 582, 566, 566, 556, 521, 500, 2093]`; ink px ref/ours 3904/5099 (ratio 1.3061); SSIM blocks <0.9: 564/30294; [overlay](images/17-apostrophes/pdflatex-lm-de1020c-export-p1-overlay.png) (42062 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-lm-de1020c-export-p1-heatmap.png) (87857 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.84, -1.09] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8624, differing 0.00704, SSIM₈ 0.9848 (raw 0.8624, 0.00704, 0.9848)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9757→0.9757 / 1.3784→1.3784; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.51 dy 0.68; `plain` dx -61.24 dy 0.68; `a` dx -59.82 dy 0.68
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 3904/3896 (ratio 0.998); SSIM blocks <0.9: 0/30294; [overlay](images/17-apostrophes/pdflatex-lm-exact-export-p1-overlay.png) (86071 B, ÷1), [heatmap](images/17-apostrophes/pdflatex-lm-exact-export-p1-heatmap.png) (66667 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.2, 0.01] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0034, differing 0.001891, SSIM₈ 1.0 (raw 0.0034, 0.001891, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0054→0.0054; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `’90s,` dx -0.01 dy 1.03; `résumé,` dx 0.01 dy 1.03; `’` dx 0.01 dy 1.03

### 17-apostrophes — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926650, 1037, 849, 834, 791, 782, 743, 772, 803, 632, 636, 590, 589, 528, 517, 2063]`; ink px ref/ours 3904/5222 (ratio 1.3376); SSIM blocks <0.9: 566/30294; [overlay](images/17-apostrophes/pdflatex-lm-main-export-p1-overlay.png) (41422 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-lm-main-export-p1-heatmap.png) (87773 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [12.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8973, not lower; centroid estimate [-12.08, -1.03] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8762, differing 0.007078, SSIM₈ 0.9846 (raw 0.8762, 0.007078, 0.9846)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9754→0.9754 / 1.4005→1.4005; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.63 dy 0.68; `plain` dx -61.36 dy 0.68; `a` dx -59.94 dy 0.68
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1928623, 960, 826, 828, 791, 756, 674, 617, 676, 594, 528, 478, 411, 388, 361, 1305]`; ink px ref/ours 3904/5180 (ratio 1.3268); SSIM blocks <0.9: 459/30294; [overlay](images/17-apostrophes/pdflatex-lm-pipeline-export-p1-overlay.png) (41040 B, ÷2), [heatmap](images/17-apostrophes/pdflatex-lm-pipeline-export-p1-heatmap.png) (83568 B, ÷1)
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.17, -0.16] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.6321, differing 0.005979, SSIM₈ 0.99 (raw 0.6774, 0.006106, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9833→0.984 / 1.0826→1.0102; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `the` dx 0.01 dy 0.67; `’90s,` dx -0.01 dy 0.67; `résumé,` dx 0.01 dy 0.67

### 17-apostrophes — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927755, 1202, 1003, 779, 683, 594, 527, 566, 500, 452, 495, 487, 558, 563, 513, 2139]`; ink px ref/ours 5210/5099 (ratio 0.9787); SSIM blocks <0.9: 446/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-7.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8494, not lower; centroid estimate [-3.78, -0.06] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.7981, differing 0.006649, SSIM₈ 0.9883 (raw 0.7981, 0.006649, 0.9883)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9813→0.9813 / 1.2756→1.2756; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -7.91 dy 0.46; `roll,` dx -6.83 dy 0.46; `the` dx -6.31 dy 0.46
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926561, 1102, 972, 825, 732, 701, 734, 802, 757, 632, 600, 546, 620, 550, 563, 2119]`; ink px ref/ours 5210/3896 (ratio 0.7478); SSIM blocks <0.9: 565/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-12.0, 0.0] pt REJECTED: applying it gives mean|Δ| 0.9057, not lower; centroid estimate [9.87, 1.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8839, differing 0.00715, SSIM₈ 0.9843 (raw 0.8839, 0.00715, 0.9843)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9749→0.9749 / 1.4127→1.4127; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `“quoted”` dx -424.88 dy 15.21; `apostrophe.` dx 62.55 dy 0.77; `’` dx 62.37 dy 0.77

### 17-apostrophes — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1927694, 1210, 922, 781, 623, 622, 609, 567, 538, 472, 564, 477, 533, 497, 495, 2212]`; ink px ref/ours 5210/5222 (ratio 1.0023); SSIM blocks <0.9: 444/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-1.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.848, not lower; centroid estimate [-2.02, 0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.8069, differing 0.006687, SSIM₈ 0.9882 (raw 0.8069, 0.006687, 0.9882)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9811→0.9811 / 1.2897→1.2897; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `résumé,` dx -8.33 dy 0.46; `roll,` dx -7.25 dy 0.46; `the` dx -6.73 dy 0.46
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926398, 1073, 975, 762, 674, 703, 645, 671, 573, 636, 638, 604, 646, 567, 654, 2597]`; ink px ref/ours 5210/5180 (ratio 0.9942); SSIM blocks <0.9: 547/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [20.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.9494, not lower; centroid estimate [11.23, 0.87] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9417, differing 0.007316, SSIM₈ 0.9852 (raw 0.9417, 0.007316, 0.9852)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9763→0.9763 / 1.5051→1.5051; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `“quoted”` dx -424.88 dy 14.85; `apostrophe.` dx 62.55 dy 0.41; `’` dx 62.37 dy 0.41

### 17-apostrophes — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926762, 1016, 928, 836, 824, 766, 758, 808, 697, 605, 580, 562, 563, 533, 517, 2061]`; ink px ref/ours 3889/5099 (ratio 1.3111); SSIM blocks <0.9: 564/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.04, -1.15] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.862, differing 0.007035, SSIM₈ 0.9848 (raw 0.862, 0.007035, 0.9848)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9757→0.9757 / 1.3777→1.3777; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.51 dy 0.68; `plain` dx -61.26 dy 0.68; `a` dx -59.83 dy 0.68
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938746, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 3889/3896 (ratio 1.0018); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.6, -0.05] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0133, differing 0.003028, SSIM₈ 1.0 (raw 0.0133, 0.003028, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0212→0.0212; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `the` dx 0.01 dy 1.03; `owl’s` dx 0.01 dy 1.03; `branch;` dx 0.01 dy 1.03

### 17-apostrophes — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1926655, 1050, 844, 818, 792, 770, 798, 713, 806, 633, 643, 584, 603, 539, 543, 2025]`; ink px ref/ours 3889/5222 (ratio 1.3428); SSIM blocks <0.9: 566/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [12.5, 0.0] pt REJECTED: applying it gives mean|Δ| 0.8968, not lower; centroid estimate [-11.28, -1.08] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.876, differing 0.00707, SSIM₈ 0.9846 (raw 0.876, 0.00707, 0.9846)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9754→0.9754 / 1.4001→1.4001; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `apostrophe.` dx -65.63 dy 0.68; `plain` dx -61.38 dy 0.68; `a` dx -59.95 dy 0.68
- word-sequence differences: replace ref ['It’s'] ours ["It's"]; replace ref ['owl’s'] ours ["owl's"]; replace ref ['don’t,', 'can’t,', 'won’t,', 'o’clock,'] ours ["don't,", "can't,", "won't,", "o'clock,"]

### 17-apostrophes — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1928596, 995, 813, 806, 822, 733, 722, 570, 682, 588, 538, 475, 432, 387, 368, 1289]`; ink px ref/ours 3889/5180 (ratio 1.332); SSIM blocks <0.9: 459/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [1.97, -0.21] pt); confidence moderate (shift explains 7% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.6319, differing 0.005977, SSIM₈ 0.99 (raw 0.6777, 0.006097, 0.9896)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9833→0.984 / 1.0832→1.01; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `the` dx 0.01 dy 0.67; `owl’s` dx 0.01 dy 0.67; `branch;` dx 0.01 dy 0.67

### 18-ligatures — lualatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920329, 1644, 1438, 1260, 1111, 1044, 1010, 908, 954, 966, 919, 985, 1012, 775, 800, 3661]`; ink px ref/ours 8390/8297 (ratio 0.9889); SSIM blocks <0.9: 773/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-12.52, 1.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3714, differing 0.01092, SSIM₈ 0.979 (raw 1.3714, 0.01092, 0.979)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9664→0.9664 / 2.192→2.192; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -444.42 dy 14.86; `ruffled,` dx -432.48 dy 14.82; `muffin.` dx 39.04 dy 0.37

### 18-ligatures — lualatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920270, 1687, 1295, 1312, 1222, 1127, 1170, 1145, 1096, 986, 853, 906, 1006, 763, 846, 3132]`; ink px ref/ours 8390/6473 (ratio 0.7715); SSIM blocks <0.9: 822/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.06, 1.31] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3323, differing 0.010821, SSIM₈ 0.9778 (raw 1.3323, 0.010821, 0.9778)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9645→0.9645 / 2.1294→2.1294; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -444.42 dy 15.21; `ruffled,` dx -432.48 dy 15.21; `muffin.` dx 44.83 dy 0.76

### 18-ligatures — lualatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923086, 2009, 1584, 1323, 1042, 951, 872, 743, 719, 715, 653, 578, 670, 587, 615, 2669]`; ink px ref/ours 8390/8245 (ratio 0.9827); SSIM blocks <0.9: 583/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-3.42, 0.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0519, differing 0.009669, SSIM₈ 0.9851 (raw 1.0519, 0.009669, 0.9851)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9762→0.9762 / 1.6812→1.6812; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `ruffled,` dx -14.51 dy 0.42; `offline,` dx -13.45 dy 0.42; `baffling,` dx -12.39 dy 0.42
- word-sequence differences: replace ref ['fluffy,', 'suffix,'] ours ['fluffy,suffix,']

### 18-ligatures — lualatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920007, 1780, 1452, 1190, 1035, 988, 1042, 899, 944, 910, 873, 891, 1105, 802, 1009, 3889]`; ink px ref/ours 8390/8216 (ratio 0.9793); SSIM blocks <0.9: 794/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.432, not lower; centroid estimate [-2.78, 1.04] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.415, differing 0.011095, SSIM₈ 0.978 (raw 1.415, 0.011095, 0.978)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.965→0.965 / 2.2592→2.2592; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -444.42 dy 14.85; `ruffled,` dx -432.48 dy 14.85; `muffin.` dx 44.83 dy 0.41

### 18-ligatures — lualatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920361, 1636, 1302, 1341, 1350, 1189, 1095, 1088, 1119, 970, 874, 974, 952, 771, 683, 3111]`; ink px ref/ours 6488/8297 (ratio 1.2788); SSIM blocks <0.9: 799/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-18.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3339, not lower; centroid estimate [-5.38, -0.31] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3134, differing 0.010683, SSIM₈ 0.9787 (raw 1.3134, 0.010683, 0.9787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.0992→2.0992; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `coffin,` dx -21.56 dy -0.3; `official,` dx -20.56 dy -0.3; `offline,` dx -20.28 dy -0.35

### 18-ligatures — lualatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 6488/6473 (ratio 0.9977); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.09, -0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0168, differing 0.004304, SSIM₈ 1.0 (raw 0.0168, 0.004304, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0268→0.0268; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `affirm,` dx 0.03 dy -0.0; `cliff,` dx 0.02 dy -0.0; `and` dx 0.02 dy -0.0

### 18-ligatures — lualatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1919468, 1514, 1315, 1392, 1274, 1173, 1187, 1186, 1171, 993, 941, 901, 1013, 830, 844, 3614]`; ink px ref/ours 6488/8245 (ratio 1.2708); SSIM blocks <0.9: 827/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [3.72, -1.27] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3902, differing 0.010983, SSIM₈ 0.9769 (raw 1.4225, 0.011085, 0.9764)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9623→0.9631 / 2.2735→2.222; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx 442.91 dy -14.75; `ruffled,` dx 417.97 dy -14.79; `offline,` dx -53.54 dy -0.35
- word-sequence differences: replace ref ['fluffy,', 'suffix,'] ours ['fluffy,suffix,']

### 18-ligatures — lualatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923264, 1835, 1307, 1400, 1257, 1163, 1056, 904, 790, 689, 710, 710, 653, 525, 524, 2029]`; ink px ref/ours 6488/8216 (ratio 1.2663); SSIM blocks <0.9: 679/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [4.36, -0.27] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9982, differing 0.009265, SSIM₈ 0.9858 (raw 0.9982, 0.009265, 0.9858)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9775→0.9775 / 1.5931→1.5931; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `affirm,` dx 0.03 dy -0.36; `cliff,` dx 0.02 dy -0.36; `and` dx 0.02 dy -0.36

### 18-ligatures — pdflatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921093, 1851, 1396, 1273, 1121, 942, 936, 833, 874, 895, 836, 811, 990, 726, 718, 3521]`; ink px ref/ours 8210/8297 (ratio 1.0106); SSIM blocks <0.9: 724/30294; [overlay](images/18-ligatures/pdflatex-de1020c-export-p1-overlay.png) (47849 B, ÷2), [heatmap](images/18-ligatures/pdflatex-de1020c-export-p1-heatmap.png) (40283 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-13.13, 0.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2894, differing 0.010507, SSIM₈ 0.9808 (raw 1.2894, 0.010507, 0.9808)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9693→0.9693 / 2.0608→2.0608; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.86 dy 14.86; `ruffled,` dx -433.44 dy 14.82; `muffin.` dx 40.63 dy 0.37

### 18-ligatures — pdflatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920559, 1567, 1220, 1353, 1311, 1096, 1116, 1164, 1088, 963, 966, 851, 964, 709, 831, 3058]`; ink px ref/ours 8210/6473 (ratio 0.7884); SSIM blocks <0.9: 802/30294; [overlay](images/18-ligatures/pdflatex-exact-export-p1-overlay.png) (50157 B, ÷2), [heatmap](images/18-ligatures/pdflatex-exact-export-p1-heatmap.png) (42371 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-7.67, 1.26] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3107, differing 0.010625, SSIM₈ 0.9788 (raw 1.3107, 0.010625, 0.9788)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9661→0.9661 / 2.0948→2.0948; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.86 dy 15.21; `ruffled,` dx -433.44 dy 15.21; `muffin.` dx 46.43 dy 0.76

### 18-ligatures — pdflatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920596, 1679, 1357, 1302, 1018, 948, 1054, 850, 849, 737, 810, 872, 952, 822, 894, 4076]`; ink px ref/ours 8210/8245 (ratio 1.0043); SSIM blocks <0.9: 677/30294; [overlay](images/18-ligatures/pdflatex-main-export-p1-overlay.png) (46838 B, ÷2), [heatmap](images/18-ligatures/pdflatex-main-export-p1-heatmap.png) (39101 B, ÷2)
  - registration error (diagnostic): global shift [-1.5, 0.0] pt by ink-projection correlation (centroid estimate [-4.03, -0.01] pt); confidence moderate (shift explains 9% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.2545, differing 0.010252, SSIM₈ 0.9824 (raw 1.3805, 0.010672, 0.9805)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9689→0.9719 / 2.2064→2.0051; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `ruffled,` dx -15.47 dy 0.42; `offline,` dx -14.74 dy 0.42; `baffling,` dx -14.05 dy 0.42
- word-sequence differences: replace ref ['fluffy,', 'suffix,'] ours ['fluffy,suffix,']

### 18-ligatures — pdflatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920478, 1754, 1336, 1186, 1070, 1028, 953, 927, 836, 882, 868, 895, 1071, 806, 941, 3785]`; ink px ref/ours 8210/8216 (ratio 1.0007); SSIM blocks <0.9: 776/30294; [overlay](images/18-ligatures/pdflatex-pipeline-export-p1-overlay.png) (47853 B, ÷2), [heatmap](images/18-ligatures/pdflatex-pipeline-export-p1-heatmap.png) (39447 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-3.39, 0.99] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3778, differing 0.010838, SSIM₈ 0.9792 (raw 1.3778, 0.010838, 0.9792)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9669→0.9669 / 2.1998→2.1998; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.86 dy 14.85; `ruffled,` dx -433.44 dy 14.85; `muffin.` dx 46.43 dy 0.41

### 18-ligatures — pdflatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920380, 1602, 1329, 1334, 1326, 1152, 1112, 1125, 1111, 987, 890, 943, 938, 768, 690, 3129]`; ink px ref/ours 6473/8297 (ratio 1.2818); SSIM blocks <0.9: 798/30294; [overlay](images/18-ligatures/pdflatex-lm-de1020c-export-p1-overlay.png) (49292 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-de1020c-export-p1-heatmap.png) (41396 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-18.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3343, not lower; centroid estimate [-5.55, -0.3] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3148, differing 0.010687, SSIM₈ 0.9787 (raw 1.3148, 0.010687, 0.9787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.1014→2.1014; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `coffin,` dx -21.57 dy 0.73; `official,` dx -20.57 dy 0.73; `offline,` dx -20.28 dy 0.68

### 18-ligatures — pdflatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 6473/6473 (ratio 1.0); SSIM blocks <0.9: 0/30294; [overlay](images/18-ligatures/pdflatex-lm-exact-export-p1-overlay.png) (43072 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-exact-export-p1-heatmap.png) (70184 B, ÷1)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [-0.09, 0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0056, differing 0.003128, SSIM₈ 1.0 (raw 0.0056, 0.003128, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.009→0.009; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `effect,` dx 0.01 dy 1.03; `shuffle,` dx -0.01 dy 1.03; `affirm,` dx 0.01 dy 1.03

### 18-ligatures — pdflatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1919497, 1494, 1326, 1385, 1259, 1161, 1201, 1216, 1133, 1055, 906, 890, 996, 814, 862, 3621]`; ink px ref/ours 6473/8245 (ratio 1.2738); SSIM blocks <0.9: 826/30294; [overlay](images/18-ligatures/pdflatex-lm-main-export-p1-overlay.png) (48424 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-main-export-p1-heatmap.png) (41380 B, ÷2)
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [3.55, -1.26] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3906, differing 0.010975, SSIM₈ 0.9769 (raw 1.422, 0.011082, 0.9764)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9623→0.9631 / 2.2727→2.2225; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx 442.91 dy -13.72; `ruffled,` dx 417.97 dy -13.77; `offline,` dx -53.54 dy 0.68
- word-sequence differences: replace ref ['fluffy,', 'suffix,'] ours ['fluffy,suffix,']

### 18-ligatures — pdflatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923291, 1748, 1366, 1370, 1243, 1151, 1072, 907, 842, 700, 671, 687, 680, 497, 551, 2040]`; ink px ref/ours 6473/8216 (ratio 1.2693); SSIM blocks <0.9: 681/30294; [overlay](images/18-ligatures/pdflatex-lm-pipeline-export-p1-overlay.png) (47721 B, ÷2), [heatmap](images/18-ligatures/pdflatex-lm-pipeline-export-p1-heatmap.png) (38629 B, ÷2)
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [4.19, -0.27] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.0, differing 0.00927, SSIM₈ 0.9858 (raw 1.0, 0.00927, 0.9858)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9775→0.9775 / 1.596→1.596; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `office,` dx -0.01 dy 0.67; `effect,` dx 0.01 dy 0.67; `and` dx -0.01 dy 0.67

### 18-ligatures — xelatex vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920422, 1579, 1465, 1271, 1273, 1053, 999, 990, 930, 938, 916, 878, 984, 761, 739, 3618]`; ink px ref/ours 8366/8297 (ratio 0.9918); SSIM blocks <0.9: 758/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-0.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3891, not lower; centroid estimate [-12.27, 0.95] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3498, differing 0.010823, SSIM₈ 0.9797 (raw 1.3498, 0.010823, 0.9797)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9675→0.9675 / 2.1573→2.1573; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.26 dy 14.86; `ruffled,` dx -432.74 dy 14.82; `muffin.` dx 39.24 dy 0.37

### 18-ligatures — xelatex vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920141, 1602, 1299, 1436, 1270, 1203, 1160, 1104, 1233, 934, 914, 917, 952, 686, 829, 3136]`; ink px ref/ours 8366/6473 (ratio 0.7737); SSIM blocks <0.9: 811/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-6.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3786, not lower; centroid estimate [-6.8, 1.25] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.333, differing 0.010848, SSIM₈ 0.978 (raw 1.333, 0.010848, 0.978)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9649→0.9649 / 2.1306→2.1306; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.26 dy 15.21; `ruffled,` dx -432.74 dy 15.21; `muffin.` dx 45.04 dy 0.76

### 18-ligatures — xelatex vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1921437, 1689, 1490, 1303, 1178, 1026, 1058, 915, 928, 912, 721, 713, 818, 596, 757, 3275]`; ink px ref/ours 8366/8245 (ratio 0.9855); SSIM blocks <0.9: 673/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-0.5, 0.0] pt by ink-projection correlation (centroid estimate [-3.17, -0.01] pt); confidence moderate (shift explains 6% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.1599, differing 0.010122, SSIM₈ 0.9838 (raw 1.2336, 0.010273, 0.9826)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9723→0.9741 / 1.9717→1.8539; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `ruffled,` dx -14.77 dy 0.42; `offline,` dx -13.73 dy 0.42; `baffling,` dx -12.7 dy 0.42
- word-sequence differences: replace ref ['fluffy,', 'suffix,'] ours ['fluffy,suffix,']

### 18-ligatures — xelatex vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920016, 1775, 1437, 1220, 1133, 1066, 1003, 939, 972, 858, 928, 877, 1020, 774, 975, 3823]`; ink px ref/ours 8366/8216 (ratio 0.9821); SSIM blocks <0.9: 792/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [1.0, 0.0] pt REJECTED: applying it gives mean|Δ| 1.4559, not lower; centroid estimate [-2.53, 0.98] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.4011, differing 0.011088, SSIM₈ 0.9784 (raw 1.4011, 0.011088, 0.9784)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9656→0.9656 / 2.237→2.237; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx -445.26 dy 14.85; `ruffled,` dx -432.74 dy 14.85; `muffin.` dx 45.04 dy 0.41

### 18-ligatures — xelatex-lm vs compiler `de1020c` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1920366, 1631, 1312, 1334, 1353, 1175, 1117, 1063, 1128, 976, 869, 980, 948, 757, 705, 3102]`; ink px ref/ours 6486/8297 (ratio 1.2792); SSIM blocks <0.9: 800/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (correlation candidate [-18.5, 0.0] pt REJECTED: applying it gives mean|Δ| 1.3341, not lower; centroid estimate [-5.31, -0.31] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3133, differing 0.010682, SSIM₈ 0.9787 (raw 1.3133, 0.010682, 0.9787)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.966→0.966 / 2.099→2.099; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `coﬀin,` dx -21.56 dy 0.73; `oﬀicial,` dx -20.56 dy 0.73; `offline,` dx -20.28 dy 0.68

### 18-ligatures — xelatex-lm vs compiler `exact` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1938816, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`; ink px ref/ours 6486/6473 (ratio 0.998); SSIM blocks <0.9: 0/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [0.16, 0.0] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.0167, differing 0.004235, SSIM₈ 1.0 (raw 0.0167, 0.004235, 1.0)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 1.0→1.0 / 0.0267→0.0267; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `aﬀirm,` dx 0.03 dy 1.03; `cliff,` dx 0.02 dy 1.03; `and` dx 0.02 dy 1.03

### 18-ligatures — xelatex-lm vs compiler `main` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1919469, 1510, 1327, 1382, 1288, 1147, 1214, 1175, 1166, 998, 937, 902, 1013, 821, 851, 3616]`; ink px ref/ours 6486/8245 (ratio 1.2712); SSIM blocks <0.9: 827/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [-1.0, 0.0] pt by ink-projection correlation (centroid estimate [3.8, -1.27] pt); confidence weak (shift explains 2% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 1.3902, differing 0.01098, SSIM₈ 0.9769 (raw 1.4224, 0.011082, 0.9764)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9623→0.9631 / 2.2734→2.222; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `fifty,` dx 442.91 dy -13.72; `ruffled,` dx 417.97 dy -13.77; `offline,` dx -53.54 dy 0.68
- word-sequence differences: replace ref ['fluffy,', 'suffix,'] ours ['fluffy,suffix,']

### 18-ligatures — xelatex-lm vs compiler `pipeline` (export)

- FlashTeX rule rectangles: 0 U+2500 items in compile_result, 0 `re f` rectangles in the PDF
- page 1: |Δ| histogram (16 bins, pixel counts) `[1923268, 1833, 1318, 1392, 1254, 1160, 1076, 886, 797, 696, 695, 711, 647, 531, 530, 2022]`; ink px ref/ours 6486/8216 (ratio 1.2667); SSIM blocks <0.9: 679/30294; images not emitted for this engine
  - registration error (diagnostic): global shift [0.0, 0.0] pt by ink-projection correlation (centroid estimate [4.44, -0.27] pt); confidence none (shift explains 0% of raw mean|Δ|); rendering error after undoing it: mean|Δ| 0.9979, differing 0.009263, SSIM₈ 0.9859 (raw 0.9979, 0.009263, 0.9859)
  - regions (raw → after registration, SSIM₈ / mean|Δ|): text-area 0.9775→0.9775 / 1.5926→1.5926; header-band 1.0→1.0 / 0.0→0.0; footer-band 1.0→1.0 / 0.0→0.0
- largest word displacements (pt): `aﬀirm,` dx 0.03 dy 0.67; `cliff,` dx 0.02 dy 0.67; `and` dx 0.02 dy 0.67

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

- previous evidence: `tests/visual-corpus/evidence/20260912T083316Z`; comparable entries: 648
- worse: 
  - 09-mixed-document/lualatex/pipeline/export: ssim_8x8_mean 0.9736 -> 0.9706
  - 09-mixed-document/lualatex/pipeline/preview: ssim_8x8_mean 0.9736 -> 0.9706
  - 09-mixed-document/pdflatex/pipeline/export: ssim_8x8_mean 0.9729 -> 0.9705
  - 09-mixed-document/pdflatex/pipeline/preview: ssim_8x8_mean 0.9729 -> 0.9705
  - 09-mixed-document/xelatex/pipeline/export: ssim_8x8_mean 0.9733 -> 0.9703
  - 09-mixed-document/xelatex/pipeline/preview: ssim_8x8_mean 0.9733 -> 0.9703

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
