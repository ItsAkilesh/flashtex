# mac-visual-oracle — FT-017 rev 2 (declared fixtures, registration, regions, provenance)

Agent / task / branch: `mac-visual-oracle` (Claude Code subagent, parent `mac-claude-a`,
machine `mac-m1max-a`) / FT-017 rev 2 (acked 06:30Z, input main 41c4616) /
`agent/mac-visual-oracle/reference-raster`
State: ready for integration (rev 2 delivered; evidence with live MacTeX oracles and with reused references)
Owned paths: `tests/visual-corpus/harness/`, `tests/visual-corpus/evidence/`,
`coordination/mac-visual-oracle.md`, `coordination/agents/mac-visual-oracle.json`
Main integrated through: a949f5b773a1bc7f1be7ae2d79581856af88a97b (merge b45bd17)

## Session note (resumed after an accidental stop)

The previous worker session was stopped by the user mid-run; its work was preserved as
78b9a64 (untested). Between that session and this one BasicTeX was removed from this Mac, so
the harness first gained a reference-reuse path (evidence run 20260912T065946Z, every oracle PDF
reused from the stopped run's renders); the coordinator then installed **MacTeX 2026 full**
(`/usr/local/texlive/2026`, tlmgr r78301; `/Library/TeX/texbin` → its bin dir) and the corpus was
re-run with live oracles (evidence run 20260912T072838Z, the primary rev 2 evidence).

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

- Native SwiftUI preview capture was not run this session (rev 1 route unchanged; the 1x-display
  upsampling limitation still applies; the native gate column reads "unavailable").
- `reference-profile.json` (raw PDF SHA-256 pins) is still the rev 1 pin (9 fixtures × main/de1020c);
  the 9 new fixtures and `pipeline` are reported "unpinned"; `main` differs because its SHA moved.
  Not re-pinned here: a pin is a baseline, not a pass — Commander may request `--pin-profile`.
- Labels `main` and `pipeline` track moving branch tips (main 462fb27 → 5f9f4ec, pipeline
  7094ef7 → c000dad between the two runs); the `--regress` diagnostic therefore flags the label,
  not a harness change: 40 (run 065946Z, all `main`) and 48 (run 072838Z, all `pipeline`) entries
  "worse" vs the previous run, `de1020c` never. Pin exact SHAs with `--compiler-ref` for a true
  regression check.
- Registration is translation-only (scale assumed 1); the correlation candidate is accepted only
  when it lowers mean|Δ| and is labelled `weak`/`moderate`/`strong` by the fraction of error it
  explains. Weak/moderate large shifts (e.g. 03-section-heading dy 54 pt, 11%) are coincidental
  line alignments, not offsets — read them as "no reliable global offset".

## Interface changes and required consumer actions

None to runtime contracts. Harness CLI: `--reference-from` added to `render_reference.sh` and
`run.sh` (default `auto`; `none` disables reuse). `provenance.json` gains `references`;
`metrics.json` entries gain `provenance`, `raster.registration_shift_pt`,
`raster.registered_*`, and per-page `regions`.

## Validation

- `selftest.py --rasterize <built rasterize>`: PASS (18 checks; run after every diff.py change).
- Run 20260912T065946Z (no engine installed): all 108 reference pairs reused from
  20260912T064032Z after fixture-SHA + preamble checks, 0 unavailable; 648 entries, 0 skipped;
  276 PNGs, none > 300 KB.
- Run 20260912T072838Z (MacTeX 2026 full, live): 108/108 oracle renders exit 0 (pdfTeX 1.40.29,
  XeTeX 0.999998, LuaHBTeX 1.24.0 — same versions as BasicTeX), new reference store written;
  648 entries, 0 skipped; 276 PNGs ≤ 300 KB; compilers main@5f9f4ec, de1020c, pipeline@c000dad
  all built.
- **Live vs recorded references** (`072838Z/reference-vs-recorded.json`): 108/108 pairs
  pixel-identical at 144 DPI (max differing px 0); 0/108 PDFs byte-identical — differences are
  CreationDate/ModDate, trailer /ID and compressed object-stream bytes only. The recorded
  BasicTeX references were therefore a faithful stand-in; the MacTeX provenance is now the
  primary one and the reuse path stays as fallback.
- Exact gates (acceptance): export vs preview-equivalent DIFFERENT on all 54 fixture/compiler
  pairs (as in rev 1), native unavailable, PDF bytes = pin only for de1020c on the 9 pinned
  fixtures. Diagnostics: 4 threshold failures (pdflatex vs de1020c on 10/12/15/16 — long
  wrapping text, greedy unjustified breaking).
- Registration vs rendering error, pdflatex-lm oracle, export side, page 1 (mean|Δ| raw → after
  registration; shift pt; confidence):
  - 17-apostrophes: de1020c 0.862 → 0.862, shift (0,0) none — the residual is rendering error
    (quotesingle vs quoteright glyphs, mean word |dx| 44.6 pt); pipeline 0.674 → 0.631, shift
    (−0.5,0) moderate 6%; pipeline word sequence = oracle, mean |dx| 0.02 pt.
  - 09-mixed-document: de1020c 2.252 → 1.901, shift (−0.5,39.5) moderate 16% (display box SSIM₈
    0.381 → 0.528); pipeline 1.859 → 1.609, shift (0,−16.5) moderate 13%.
  - 18-ligatures: de1020c 1.315, correlation candidate (−18.5,0) REJECTED (would raise the error)
    → shift 0, all rendering error (ligature advance widths); pipeline 0.976, shift 0.
  - 02-wrapping-paragraph: de1020c 7.786, candidate (−8.5,−14.5) rejected; pipeline 4.547 (raw
    SSIM₈ 0.937 vs de1020c 0.862) — line breaking, not offset.

## Needs from others

- None blocking. Commander: say whether `reference-profile.json` should be re-pinned to the
  18-fixture corpus, and whether `pipeline` should be pinned to a SHA rather than the branch tip.

## Resource state

- Allocation `claude-mac20x-visual-oracle-stage2` (Claude Max 20x on mac-m1max-a, shared
  account quota). Usage/remaining: unknown — no programmatic readout; one session in flight.

## Next action

Await Commander review. Next if assigned: Retina native capture route, scale-aware registration
for native captures, per-line region metrics, merge origin/main into this branch.

## Peer revisions reviewed and adaptations

- `origin/agent/mac-render-pipeline/unified` 7094ef7 → c000dad ("secnumdepth option and
  \setcounter{secnumdepth} parsing"): `crates/render-pipeline` builds from a `git archive` of
  the whole `crates/` tree; compared as `pipeline` (binary `flashtex-render`, same JSON Lines
  interface). Finding for its owner: against the Latin Modern oracle it reproduces the word
  sequence of 17-apostrophes exactly (mean |dx| 0.02 pt) and halves the raw error of de1020c on
  02-wrapping-paragraph; display-math boxes remain the weakest region (SSIM₈ 0.37–0.53).
- `origin/main` 462fb27 / 5f9f4ec at run time: `crates/compiler` changed in 17 files since
  a949f5b (integrated base); it builds and is compared as `main`. The harness consumes only the
  runtime-v1 compile_result and U+2500 rule convention, both still present; a full merge of
  main into this branch is deferred to the next clean checkpoint.

Updated: 2026-09-12T07:42Z
