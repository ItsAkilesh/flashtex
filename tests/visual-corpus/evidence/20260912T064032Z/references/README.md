# Reference store — run 20260912T064032Z (interrupted run)

This run was started by the FT-017 rev 2 worker with TeX Live 2026 BasicTeX installed and was
stopped by the user during the diff stage: no `metrics.json` and no comparison report exist for
it, and its partial, footer-less overlay images were removed. It is kept for what it rendered:

- `../provenance.json` — engine/font/package/system-font pins of that session (pdfTeX
  3.141592653-2.6-1.40.29, XeTeX 0.999998, LuaHBTeX 1.24.0; `times`/`lmodern`/`fontspec` from the
  BasicTeX tree; Times New Roman TTF from macOS), the 18 fixture SHA-256s and the compiler builds
  it had made.
- this directory — the 108 oracle PDFs (18 fixtures × pdflatex, pdflatex-lm, xelatex, xelatex-lm,
  lualatex, lualatex-lm), each with an `engine.json` recording engine version, path, flags,
  preamble, exit status, fixture SHA-256 and PDF SHA-256, plus `manifest.json`.

BasicTeX was removed from the machine afterwards (MacTeX install pending), so these PDFs are the
only oracle references available until an engine is installed again. `run.sh --reference-from auto`
reuses them for any missing engine after checking fixture SHA-256 and preamble; the comparison
that consumed them is `evidence/20260912T065946Z`. Rasters are not stored: they are re-derived
deterministically from the PDF by `harness/rasterize.swift`.
