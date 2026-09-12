# mac-visual-oracle — FT-017 rev 1 (reference-render + raster-diff harness)

Agent / task / branch: `mac-visual-oracle` (Claude Code subagent, parent `mac-claude-a`,
machine `mac-m1max-a`) / FT-017 rev 1 / `agent/mac-visual-oracle/reference-raster`
State: in progress — harness runs end to end; first evidence run committed; follow-ups pending
Owned paths: `tests/visual-corpus/harness/`, `tests/visual-corpus/evidence/`,
`coordination/mac-visual-oracle.md`, `coordination/agents/mac-visual-oracle.json`
Main integrated through: 984fa28f2537beadb9f848eb90f69de0327d1fa5 (branch base; assignment input 998e566)

## Ready behavior

- `harness/fixtures/*.tex` + `.meta.json`: 9 declared fixtures (plain, wrapping, section,
  bold/emph, Unicode, inline math, display math with sum limits, two-page, mixed).
  SHA-256 of each fixture is recorded in every run's `provenance.json` and `report.md`.
- `harness/rasterize.swift`: one CoreGraphics rasterizer for every side (`pdf`: CGPDFDocument
  → 144 DPI sRGB RGBA bitmap, PNG + raw `.rgba` + `.rgba.json`; `preview`: preview-equivalent
  CoreText draw of a runtime-v1 compile_result; `word-boxes`: PDFKit word boxes as in
  tools/native-validation/oracle_extract; `crop-scale`: for screen captures).
- `harness/render_reference.sh`: pdflatex (T1 + `times`), xelatex and lualatex (`fontspec`
  `Times New Roman`) with the apples-to-apples 12pt/1in/parindent 0/secnumdepth 0 preamble,
  `-interaction=batchmode -halt-on-error -file-line-error`; versions/availability in `engines.json`.
- `harness/render_flashtex.sh`: body-only compile through each compiler build → `flashtex-pdf
  --verify --embed-font auto` → same rasterizer (export side) + preview-equivalent raster;
  rule geometry from compile_result U+2500 items and `re f` rects from the PDF.
- `harness/diff.py` (stdlib; Pillow used only as accelerator when importable): per page
  |Δluma| histogram/mean/max, differing and ≥threshold fractions, 8×8-block SSIM, overlay
  (reference magenta / FlashTeX cyan), heatmap, PDFKit word alignment dx/dy/line starts,
  rule presence via ink rows; per-fixture thresholds (`thresholds.json`) and `--regress`.
- `harness/run.sh`: builds pinned refs via `git archive` + cargo, orchestrates everything into
  `evidence/<UTC>/{report.md,metrics.json,provenance.json,images/}`.

## Incomplete behavior

- Native preview capture (Commander addendum, issue #2 requirement): `screencapture` of the
  running FlashTeXMac window + page-region detection + resample → `native` side. Not yet
  implemented in this checkpoint; the report says so explicitly.
- Follow-ups: (1) thresholds + `--regress` are implemented but only exercised once; (2) export
  vs preview-equivalent tables are separate in the report (done), native table pending.

## Interface changes and required consumer actions

None. No runtime-v1 item kinds added or proposed; U+2500 rule convention consumed as-is.

## Validation

- Engines verified on this Mac: pdflatex (pdfTeX 1.40.29), xelatex (0.999998), lualatex
  (LuaHBTeX 1.24.0), all TeX Live 2026 BasicTeX; fontspec + Times New Roman compiles.
- First full run: see the newest `tests/visual-corpus/evidence/<UTC>/report.md`.

## Needs from others

- None blocking. Commander: confirm whether the native capture route (Screen Recording only,
  no Accessibility) is acceptable evidence for issue #2 once implemented.

## Resource state

- Allocation `claude-mac20x-visual-oracle` (Claude Max 20x on mac-m1max-a, shared account
  quota). Usage/reset/remaining: unknown — no programmatic quota readout is available to this
  subagent; no in-flight calls other than this session. No paid API, no Cursor.

## Next action

Implement native preview capture (`capture_native.sh`), rerun with `--native`, then finalize
thresholds and `--regress auto` against the first evidence dir.

## Peer revisions reviewed and adaptations

- Reviewed `origin/agent/mac-validation/native-verification` (oracle_compare.py,
  oracle_extract.swift, reports/oracle-20260912T050958Z.md): reused the variant-B preamble,
  the PDFKit word-box convention and the word normalisation; did not copy the report format.
- Reviewed `origin/agent/mac-claude-a/mac-shell` PDFExport.swift / PreviewView.swift /
  Rules.swift: preview-equivalent draw follows PDFExport.render exactly (Times-Roman CoreText
  at x/baseline; RuleConvention 0.5em advance, 0.0857em thickness).
- Reviewed `origin/agent/mac-pdf/pdf-output` 5b5f7b5 main.rs: `--embed-font auto` used so
  Unicode fixtures get real glyphs instead of substitutions.

Updated: 2026-09-12T05:55Z
