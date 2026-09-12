# mac-visual-oracle — FT-017 rev 3 + exact-route column (GH-24 gates; flashtex-render --v2 → from-v2)

Agent / task / branch: `mac-visual-oracle` (Claude Code subagent, parent `mac-claude-a`,
machine `mac-m1max-a`) / FT-017 rev 3 (acked, input main 59a49ab; GH-24) /
`agent/mac-visual-oracle/reference-raster`
State: ready for integration (GH-24 gates, native evidence, and the exact-route candidate column all delivered with evidence)
Owned paths: `tests/visual-corpus/harness/`, `tests/visual-corpus/evidence/`,
`coordination/mac-visual-oracle.md`, `coordination/agents/mac-visual-oracle.json`
Main integrated through: 4248b49 (merged this session)

## Durable checkpoint (context-checkpoint policy; written at every push)

- Lane: FT-017 rev 3 / GH-24 (done), follow-ups (done), refill task "exact-route candidate
  column" (done: evidence 20260912T100307Z).
- Branch `agent/mac-visual-oracle/reference-raster`; worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a8603282c8958a68c` (local branch name
  `mvo/rev2`, pushes with `git push origin HEAD:agent/mac-visual-oracle/reference-raster`; the
  stopped worker's worktree still holds the branch name, so `scripts/coord.py` refuses this
  worktree — the agents JSON is written with the same schema by hand).
- Owned paths: `tests/visual-corpus/harness`, `tests/visual-corpus/evidence`,
  `coordination/mac-visual-oracle.md`, `coordination/agents/mac-visual-oracle.json`. No other
  path is touched; transferred crates untouched; no purchases; Claude Max 20x shared quota only.
- Context usage of this session: ~2.6% of the 15M-token window at this checkpoint (read from the
  tool budget counter; no other readout available).
- Scratch state (not in Git; session scratchpad `…/scratchpad/`): `vc/run-20260912T083316Z/`
  (work dir of the evidence run: reference/, flashtex/, native/, rasterize), `vc/builds/`
  (cargo builds by label-SHA), `r2/pinA/` (oracle pin render pass), `r2/*.sh|*.py` helper
  scripts, `app2-169c2a8…/apps/mac/.build/debug/FlashTeXMac` (app for native capture).
- Exact next commands if resumed: `git fetch origin && git status` (tree should be clean at the
  pushed SHA); await Commander review of GH-24; if a re-baseline of `reference-profile.json` is
  requested: `bash tests/visual-corpus/harness/run.sh --scratch <scratchpad>/vc --skip-build
  --pin-profile` (labelled baseline, not a pass).
- Dependency SHAs: origin/main 4248b49 (merged); compilers compared in the latest run: main c35ca8b,
  de1020c, pipeline 65dbe7d, exact = render 65dbe7d + pdf-exact 8789abf (origin/agent/mac-pdf/v2-adapter);
  flashtex-pdf 5b5f7b5; app origin/agent/mac-claude-a/mac-shell 169c2a8; MacTeX 2026 full, tlmgr r78301.

## Session note (resumed after an accidental stop)

The previous worker session was stopped by the user mid-run; its work was preserved as
78b9a64 (untested). Between that session and this one BasicTeX was removed from this Mac, so
the harness first gained a reference-reuse path (evidence run 20260912T065946Z, every oracle PDF
reused from the stopped run's renders); the coordinator then installed **MacTeX 2026 full**
(`/usr/local/texlive/2026`, tlmgr r78301; `/Library/TeX/texbin` → its bin dir) and the corpus was
re-run with live oracles (evidence run 20260912T072838Z, the primary rev 2 evidence).

## Ready behavior (refill: exact-route candidate column)

- **Candidate `exact`** in the closure matrix: `flashtex-render --secnumdepth 0 --v2` (built by
  `git archive` from origin/agent/mac-render-pipeline/unified) → `flashtex-pdf-exact from-v2
  [--font-dir]` (built from origin/agent/mac-pdf/v2-adapter, crates/pdf) → PDF → shared 144-DPI
  raster; same gates as every candidate (raw bytes vs pinned oracle, zero-pixel vs oracle raster,
  self-regression, preview parity) plus `flashtex-pdf-exact classify` categories against the
  pdflatex-lm reference, from-v2's summary, embedded fonts and every deviation note it prints.
  `run.sh --exact-route auto|none|<pdf ref>[:<render ref>]`, `--font-dir`; a producer build
  failure is reported in the report, not fatal. `render_flashtex.sh --exact/--font-dir/--reference`.
- Report: Gate 2 table gains a classify-categories column; an "Exact route per fixture" table
  lists from-v2 summary, categories, deviations, zero-pixel result and differing px per page; the
  builds list states both producer SHAs and whether the tool reported the sha256 deviation.

## Ready behavior (rev 3, GH-24)

- **Three separate gates, never merged, nothing normalised** (`diff.py`, report sections
  "Gate 1/2/3", `metrics.json` → `gates[]`, `gate_summary`, `oracle_self`):
  1. *FlashTeX self-regression*: candidate `flashtex.pdf` SHA-256 vs the pinned PRIOR FLASHTEX
     output in `harness/reference-profile.json` — labelled as reproducibility of FlashTeX against
     itself, "says nothing about LaTeX".
  2. *Candidate vs established engine*, per oracle variant: raw unmodified PDF bytes vs the pinned
     MacTeX oracle PDF (`harness/oracle-profile.json`) and vs this run's live oracle render;
     zero-pixel raster equality of the export raster vs the oracle raster page by page
     (page-count mismatch = DIFFERENT). pdflatex + pdflatex-lm in the report table, all six
     variants in metrics.
  3. *Native/export parity*: export raster = preview-equivalent raster; export raster = native
     screen capture.
  A plain statement at the top of the report gives the counts and says parity is NOT claimed
  while any pair differs. `--gate` fails on any of the three.
- **Established-engine pin that is a real pin**: `render_reference.sh` runs every engine with
  `SOURCE_DATE_EPOCH=0 FORCE_SOURCE_DATE=1` and gives lualatex an explicit
  `\pdfvariable trailerid` line (recorded in engine.json, the preamble and the pin). Verified:
  two independent renders of all 108 fixture/oracle pairs are byte-identical (108/108) and
  pixel-identical. `pin_oracle.py` / `run.sh --pin-oracle` write `oracle-profile.json` from fresh
  renders only (engine version, MacTeX distribution + tlmgr revision, texbin realpath, font files
  from the log, preamble, flags, env, page geometry, fixture SHA-256, PDF SHA-256); the diff
  verifies on every run that the live oracle bytes still equal the pin (`oracle_live_equals_pin`).
- **Reference store + reuse** (rev 2, unchanged): runs that render write `<evidence>/references/`;
  `--reference-from auto` reuses stored PDFs only on fixture-SHA + preamble match when an engine
  is missing (the env/trailerid change makes older stores non-matching for lualatex — correct).
- **Native capture** (`capture_native.sh` + `rasterize find-page`): page detection now tries the
  most frequent non-white colours as pane background (a diagnostics panel or scrollbar no longer
  outvotes it), tolerates scrollbar-merged rows and single rows cut by ink of the pane's own
  colour; the app now opens on the Retina display (backing 2×, resample ×1.52 vs rev 1's ×2.84).
- `selftest.py`: 25 checks (adds a synthetic gate tree: one candidate identical to the oracle,
  one differing; the three gates judged independently; report sections and no-parity statement).

### Rev 2 behavior still in force

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

- `selftest.py`: PASS (25 checks).
- **Evidence run `20260912T100307Z`** (MacTeX 2026 full live, pin verified live 108/108; candidates
  main@c35ca8b, de1020c, pipeline@65dbe7d, **exact** = render 65dbe7d → pdf-exact 8789abf;
  72 candidates × 6 oracles = 432 pairs, 864 diff entries, 368 PNGs ≤ 300 KB):
  - raw bytes = pinned oracle **0/432**, zero-pixel = oracle **0/432**, self-regression 9/72,
    export = preview-equivalent 0/72; native not captured this run (0/0). Parity NOT claimed.
  - **Exact route vs pdflatex-lm, per fixture** (18/18 rendered, from-v2 exit 0, all six classify
    categories differ on every fixture — ContentOperators/FontProgram/FontMetadata/ObjectLayout/
    Compression/DocumentIdentity, plus FontResources on the math/list/Unicode fixtures): zero-pixel
    DIFFERENT everywhere; differing px at 144 DPI: 01 1,754; 04 1,831; 05 1,262; 17 3,667;
    18 6,065; 03 2,047; 06 1,556; 07 2,127; 13 5,831; 14 9,959; 09 8,668; 10 14,033; 11 17,589;
    02 27,814; 12 45,570; 08 96,274/96,135/42,817; 15 80,681/80,717/80,723; 16 96,274/29,402
    (of 1,938,816 per page). Diagnostics: SSIM₈ 1.0000 and PDFKit word sequence = oracle with mean
    |dx| 0.00 pt on 01–05, 08, 12, 15–18 (the text fixtures differ at anti-aliasing level only,
    max |Δ| 4 on 01); math/list fixtures differ in layout (11 |dx| 19.4 pt, 13 12.2 pt).
  - **Deviation note**: from-v2 reported **no** sha256 deviation at these producer revisions
    (render-pipeline 919ad8b "raw font digests" switched to SHA-256(bytes)). The earlier attempt of
    this run at render 4888a67 / pdf-exact 654f626 (aborted before the diff stage by a harness
    JSON bug, fixed in 959467f) reported the SHA-256(bytes ‖ face_index) deviation on 18/18
    fixtures (31 font notes); recorded in `provenance.json` → `stage_notes`.
- Previous runs (083316Z GH-24 + native captures, 072838Z, 065946Z) preserved unchanged.
- Diagnostics (never acceptance): 4 threshold failures (pdflatex vs de1020c on 10/12/15/16);
  regress vs 083316Z: 6 entries, all label `pipeline` (tip moved 79ba728 → 65dbe7d).

## Needs from others

- None blocking. Commander: `reference-profile.json` (self-regression baseline) is still the rev 1
  9-fixture pin; say whether to re-baseline it to 18 fixtures × {main, de1020c, pipeline}
  (`--pin-profile`, labelled baseline-not-pass) and whether `pipeline`/`main` labels should be
  pinned to SHAs.

## Resource state

- Allocation `claude-mac20x-visual-oracle-stage2` (Claude Max 20x on mac-m1max-a, shared
  account quota). Usage/remaining: unknown — no programmatic readout; one session in flight.

## Next action

Await Commander review. Next if assigned: pin `pipeline`/`exact` producers to SHAs for a true
regression check; native capture of the exact route's PDF is not applicable (the app draws the v1
compile_result); per-line region metrics; re-baseline the self-regression profile if requested.

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

Updated: 2026-09-12T10:17Z
