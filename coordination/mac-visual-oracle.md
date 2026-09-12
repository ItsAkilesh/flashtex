# mac-visual-oracle — FT-017 rev 2 (declared fixtures, registration, regions, provenance)

Agent / task / branch: `mac-visual-oracle` (Claude Code subagent, parent `mac-claude-a`,
machine `mac-m1max-a`) / FT-017 rev 2 (acked 06:30Z, input main 41c4616) /
`agent/mac-visual-oracle/reference-raster`
State: in progress — rev 2 code complete and self-tested; full corpus run in progress (see Validation)
Owned paths: `tests/visual-corpus/harness/`, `tests/visual-corpus/evidence/`,
`coordination/mac-visual-oracle.md`, `coordination/agents/mac-visual-oracle.json`
Main integrated through: a949f5b773a1bc7f1be7ae2d79581856af88a97b (merge b45bd17)

## Session note (resumed after an accidental stop)

The previous worker session was stopped by the user mid-run; its work was preserved as
78b9a64 (untested). Between that session and this one **BasicTeX was removed** from this
Mac (MacTeX install pending the user's sudo): `pdflatex`/`xelatex`/`lualatex` are absent.
The oracle renders the stopped run had produced (18 fixtures × 6 oracle variants, TeX Live
2026 BasicTeX) survived in the session scratchpad and are now stored durably in the
repository so the harness keeps working without an engine (see "Reference reuse").

## Ready behavior (rev 2)

- **18 declared fixtures** (`harness/fixtures/*.tex` + `.meta.json`; SHA-256 of both in every
  `provenance.json`, `metrics.json` entry and PNG footer): 01 plain, 02 wrapping, 03 section
  heading, 04 bold/emph, 05 Unicode, 06 inline math, 07 display math, 08 two-page, 09 mixed,
  10 long Unicode paragraph (dashes, curly/angle quotes, currency), 11 nested lists,
  12 justified paragraphs, 13 rich display math (`\int`/`\sum` limits, `\left(...\right)`),
  14 dense inline math, 15 three-page document (one section per page), 16 heading at a
  page break, **17 apostrophes (quotesingle vs quoteright regression)**, 18 ligature words.
- **Reference reuse** (new): `render_reference.sh --reference-from <evidence dir>` (repeatable);
  `run.sh --reference-from auto` (default) passes every evidence dir that has
  `references/manifest.json`, newest first. When an engine binary is missing, the stored
  `main.pdf` is reused only if its recorded fixture SHA-256 and preamble match; its raster and
  PDFKit word boxes are re-derived by the shared rasterizer; `engine.json` gets
  `reused_from {run, engine_version, path, flags}`; `provenance.json` → `references`
  lists fresh / reused (per pair: run, engine version, PDF SHA-256, fixture SHA-256) /
  unavailable (with reason). A pair with no stored reference is reported as
  **reference unavailable** (skipped row with reason), never a failure. Runs that render
  references with an installed engine write `<evidence>/references/` themselves (PDF +
  engine.json + manifest; ~3.7 MB for 108 pairs), so the chain continues after MacTeX arrives.
- **Reference store**: `evidence/20260912T064032Z/references/` — 108 PDFs (18 × pdflatex,
  pdflatex-lm, xelatex, xelatex-lm, lualatex, lualatex-lm) pinned to pdfTeX 1.40.29 /
  XeTeX 0.999998 / LuaHBTeX 1.24.0 (TeX Live 2026 BasicTeX), Times (`times` / Times New
  Roman TTF) and Latin Modern (`lmodern` / lmroman12 OTF by explicit path) variants.
- **Registration separated from rendering error**: per page, global (dx,dy) by 1-D
  ink-projection cross-correlation (±60 pt; scale assumed 1 because both sides are
  rasterized from equal MediaBoxes at the same DPI) plus the ink-centroid estimate; every
  table shows raw mean|Δ| / SSIM₈, the registration shift, and mean|Δ| / SSIM₈ after undoing
  the shift. Diagnostics only; never acceptance.
- **Region metrics**: text area (inside the 1 in margins), header band, footer band, and
  display-math boxes derived from the reference word boxes (lines indented ≥24 pt on both
  sides, merged, 4 pt margin), each with raw and post-registration mean|Δ| / SSIM₈ / differing
  fraction and the reference ink count.
- **Overlay/heatmap provenance**: `rasterize annotate` burns a footer band into every
  overlay/heatmap PNG (fixture name + SHA-256 prefix, oracle engine + version + font, "PDF
  reused from run …" when applicable, compiler label@SHA, flashtex-pdf SHA, side, DPI/colour
  profile, run stamp, page) and stores the same text as XMP `dc:description`; `diff.py` adds
  standard PNG `tEXt` chunks (`Description`, `Software`). PNGs stay ≤ 300 KB (downscaled by 2
  until ≤ 90 KB before the footer).
- **Builds**: `main` (origin/main at run time), `de1020c`, and `pipeline` =
  `origin/agent/mac-render-pipeline/unified:crates/render-pipeline:flashtex-render` — auto-added
  when the crate exists; a build failure is reported in the report/provenance and the label is
  skipped instead of aborting the run.
- **Self-test**: `python3 harness/selftest.py [--rasterize <bin>]` — synthetic rasters: known
  (7,−5) px shift recovered, post-registration error 0 while raw error > 0, an extra line
  survives registration as rendering error, region names/metrics, display-box derivation from
  word boxes, PNG tEXt round-trip, `rasterize annotate` footer, and the reference-reuse decision
  through `render_reference.sh` with a fake texbin (reused only on SHA + preamble match;
  changed fixture → "reference unavailable"). All green on this machine.

## Incomplete behavior

- No oracle can be rendered fresh on this machine until MacTeX is installed; every reference
  in the rev 2 evidence run is reused from 20260912T064032Z (stated per entry).
- Native SwiftUI preview capture was not run this session (rev 1 route unchanged; 1x display
  upsampling limitation still applies).
- `reference-profile.json` (raw PDF SHA-256 pins) was pinned in rev 1 for 9 fixtures × main/de1020c;
  the 9 new fixtures and the `pipeline` build are reported as unpinned. Not re-pinned here
  because a pin is a baseline, not a pass; Commander may ask for an explicit `--pin-profile`.
- The `main` build at run time was 462fb27 (origin/main moved during the session); the run
  reports the SHA it actually built.

## Interface changes and required consumer actions

None to runtime contracts. Harness CLI: `--reference-from` added to `render_reference.sh` and
`run.sh` (default `auto`; `none` disables reuse). `provenance.json` gains `references`;
`metrics.json` entries gain `provenance`, `raster.registration_shift_pt`,
`raster.registered_*`, and per-page `regions`.

## Validation

- `selftest.py --rasterize <built rasterize>`: PASS (18 checks).
- `render_reference.sh` without any engine: all 108 pairs reused from 20260912T064032Z
  (fixture SHA + preamble verified), 0 unavailable.
- Evidence run: `tests/visual-corpus/evidence/20260912T065946Z` (in progress at this
  checkpoint; numbers in the next update).

## Needs from others

- User: MacTeX install (sudo) to render fresh oracles again; the harness then writes a new
  reference store automatically.

## Resource state

- Allocation `claude-mac20x-visual-oracle-stage2` (Claude Max 20x on mac-m1max-a, shared
  account quota). Usage/remaining: unknown — no programmatic readout; one session in flight.

## Next action

Finish the evidence run, push it, update this handoff with registration-vs-rendering numbers,
report to Commander via `coord.py report`.

## Peer revisions reviewed and adaptations

- `origin/agent/mac-render-pipeline/unified` 7094ef7 (resume checkpoint; "drop
  out-of-ownership harness snapshot"): crate `crates/render-pipeline` builds with
  `cargo build --release` from a `git archive` of the whole `crates/` tree; added as the
  `pipeline` compiler label. Its binary is `flashtex-render` with the same JSON Lines interface.
- `origin/main` 462fb27 at run time: `crates/compiler` changed in 17 files since a949f5b
  (integrated base); it builds and is compared as `main`. The harness consumes only the
  runtime-v1 compile_result and U+2500 rule convention, both still present; a full merge of
  main into this branch is deferred to the next clean checkpoint.

Updated: 2026-09-12T07:10Z
