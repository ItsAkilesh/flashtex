# mac-visual-oracle — FT-017 rev 1 (reference-render + raster-diff harness)

Agent / task / branch: `mac-visual-oracle` (Claude Code subagent, parent `mac-claude-a`,
machine `mac-m1max-a`) / FT-017 rev 1 / `agent/mac-visual-oracle/reference-raster`
State: ready for integration (rev 1 deliverables + coordinator addenda); queue steps 0/1 partially covered
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

- `harness/capture_native.sh` (coordinator addendum): launches the app built from
  origin/agent/mac-claude-a/mac-shell with the fixture seeded and the compiler auto-attached,
  waits for the `status: revision N` log line, `screencapture -l <CGWindowList id>`, detects the
  page rectangle (`rasterize find-page`), resamples to the raster size, masks the "page N"
  caption; the ACTUAL SwiftUI preview, reported in its own table and gate column.
- Latin Modern oracle variants (`pdflatex-lm`, `xelatex-lm`, `lualatex-lm`: lmodern / fontspec
  lmroman12-*.otf by explicit path) beside the Times variants; both reported for every fixture.
- Exact-equality gates (acceptance): export raster = preview-equivalent raster; export raster =
  native capture; raw PDF SHA-256 = `harness/reference-profile.json` (pinned explicitly with
  `--pin-profile`; a pin run is labelled "baseline, not a pass"). All tolerances, SSIM,
  registration shifts and `--regress` are labelled diagnostics and never count as acceptance.
- Registration diagnostics: ink-centroid shift and metrics after undoing it, per page.
- `run.sh` accepts `label=ref[:crate:bin]` and auto-adds
  `pipeline=origin/agent/mac-render-pipeline/unified:crates/render-pipeline:flashtex-render`
  when that branch/crate exists (it does not yet; the run says so).

## Incomplete behavior

- Native capture ran on the external 1x display (the app ignores a pre-set NSWindow Frame
  default, and no Accessibility is available to move/resize it), so the page is ~431 px wide
  and is UPSAMPLED x2.84 to the 144-DPI raster. The report states this; exact native equality
  cannot pass under these conditions and is reported as DIFFERENT with counts, not normalised.
- Only page 1 is captured natively (the pane shows page 1 at launch).
- Exact gates currently: export vs preview-equivalent DIFFERENT on every fixture (hundreds to
  tens of thousands of px; max |Δ| up to 255 where glyph substitution/hinting differs), export vs
  native DIFFERENT (resampling), PDF bytes = baseline pinned this run. This is the honest state.
- Queue step 0 "region metrics" beyond page-level SSIM blocks and registration: not done.

## Interface changes and required consumer actions

None. No runtime-v1 item kinds added or proposed; U+2500 rule convention consumed as-is.

## Validation

- Engines verified on this Mac: pdflatex (pdfTeX 1.40.29), xelatex (0.999998), lualatex
  (LuaHBTeX 1.24.0), all TeX Live 2026 BasicTeX; fontspec + Times New Roman and Latin Modern
  (explicit path) compile for every fixture (54 oracle renders, all exit 0).
- Evidence: `tests/visual-corpus/evidence/20260912T053804Z` (run 1, Times only) and
  `tests/visual-corpus/evidence/20260912T055841Z` (run 2: + LM variants, native captures,
  gates, registration, regress vs run 1 = no diagnostic worsened; metrics bit-identical between
  runs for the shared entries, i.e. the pipeline is deterministic).
- Headline (pdflatex vs de1020c export, 144 DPI): 01-plain SSIM₈ 0.9975 / mean|Δ| 0.19 /
  word dx 0.43 pt; 07-math-display SSIM₈ 0.9949 / 0.26 / dx 0.59 pt, dy 1.41 pt; 02-wrapping
  SSIM₈ 0.8663 / 8.37 / dx 116 pt (greedy unjustified breaking). Native capture of the same:
  SSIM₈ 0.9936 / 0.8628 / 0.9924 (upsampled).
- Commit-author note: commit 3226840 carries author email me@jay3332.tech instead of the
  repo-configured noreply address (my mistake, not rewritten per the no-rewrite rule); later
  commits use the repository configuration unchanged.

## Needs from others

- None blocking. Commander: confirm whether the native capture route (Screen Recording only,
  no Accessibility) is acceptable evidence for issue #2 once implemented.

## Resource state

- Allocation `claude-mac20x-visual-oracle` (Claude Max 20x on mac-m1max-a, shared account
  quota). Usage/reset/remaining: unknown — no programmatic quota readout is available to this
  subagent; no in-flight calls other than this session. No paid API, no Cursor.

## Next action

Queue step 0/1 refinements if assigned: region metrics (per text line / math region), a Retina
capture route (needs the app to open on the main display or a windowed size hook in apps/mac
owned by mac-claude-a), and `pipeline` build once origin/agent/mac-render-pipeline/unified exists.

## Peer revisions reviewed and adaptations

- Reviewed `origin/agent/mac-validation/native-verification` (oracle_compare.py,
  oracle_extract.swift, reports/oracle-20260912T050958Z.md): reused the variant-B preamble,
  the PDFKit word-box convention and the word normalisation; did not copy the report format.
- Reviewed `origin/agent/mac-claude-a/mac-shell` PDFExport.swift / PreviewView.swift /
  Rules.swift: preview-equivalent draw follows PDFExport.render exactly (Times-Roman CoreText
  at x/baseline; RuleConvention 0.5em advance, 0.0857em thickness).
- Reviewed `origin/agent/mac-pdf/pdf-output` 5b5f7b5 main.rs: `--embed-font auto` used so
  Unicode fixtures get real glyphs instead of substitutions.

Updated: 2026-09-12T06:25Z
