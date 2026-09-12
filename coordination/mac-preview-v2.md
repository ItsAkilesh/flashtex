# mac-preview-v2 handoff

Agent / task / branch: mac-preview-v2 (Claude Code subagent, parent mac-claude-a) /
rendering-v2 display-list consumer for the Mac preview: exact glyph runs by original GID and
typed rules, off-main immutable page preparation with stale-paint suppression, zero-tolerance
export/preview parity, and (current follow-up) the v2 pane LIVE while typing through the
negotiated `display-list-v2` capability /
`agent/mac-preview-v2/live` (base `origin/agent/mac-claude-a/mac-shell` 92a052c, merged up to 04a4eaa).
Earlier branch `agent/mac-preview-v2/exact-glyphs` (aaa4c4b) is integrated in mac-shell fc90200.
State: ready for integration (checkpoint 72f3739; lane continues under the improvement policy)
Worktree: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a93c86d3bd8db3ed3`
Owned paths: `apps/mac/Sources/FlashTeXMac/PreviewV2View.swift`,
`apps/mac/Tests/FlashTeXMacTests/PreviewV2Tests.swift`, `docs/contracts/runtime-v1-display-list-v2.md`
(this follow-up), plus the lane files carried from the released branch (`GlyphRunRenderer.swift`,
`FlashTeXProtocol/RenderingV2.swift`, `FlashTeXProtocol/RenderingV2Fast.swift`, `RenderingV2Tests.swift`,
`Fixtures/display-list-v2-*`, `Fixtures/fake_worker_v2.py`), this handoff and `coordination/agents/mac-preview-v2.json`.
Parent/other-lane files touched ONLY in the labelled "shared shell edits" commits (6a9bb15, fea1ef1)
and the merge adaptation of `ExactPDFExport.swift` (72f3739): the parent cherry-picks or re-applies.
Never touched: crates/font-engine, paragraph-layout, math-layout, crates/render-pipeline.

Main integrated through: mac-shell 04a4eaa (which carries main); origin/main reviewed at dd75e15
(coordination-only changes for this lane).

## Completed behavior (tip 0dbdf77, pushed)

- c12346f: released lane files rebased onto the `@Observable` shell (`@Environment(ShellModel.self)`).
- 8cb9ade: shared shell hooks for the parent (see above).
- 3f4c899: `RenderingV2.validate` applies crates/rendering-core `DisplayList::validate` rules:
  used features declared (`glyph_run`/`rule` + `rgba-srgb`/`cluster-actualtext`; `static-truetype`
  deliberately not derived — pipeline paints Latin Modern as `opentype-cff`), every cluster has a
  glyph, source ranges within the declared document byte_length, ticks and tick sums within
  ±(2^53−1), rendering-core path rule and collection bounds (`RenderingV2.Bounds`).
- 00e2e76: `V2PreparedPage` (per page, once, off-main: CTFont per run, CGGlyph IDs, PDF-space
  origins, rule CGRects); `V2Loader` (serial queue, load tickets, run-loop delivery, superseded
  results dropped and counted; previous frame retained with an explicit STALE indicator while a
  load is in flight; a refusal drops it); `V2PageRasterizer` (@Observable, off-main page bitmaps
  through `GlyphRunRenderer.rasterize`, blitted 1:1, bounded bytes/LRU, stale bitmaps dropped on
  arrival); `V2Parity` (preview raster vs CG PDF export rasterized back, byte-for-byte, tolerance 0);
  `FLASHTEX_V2_PARITY_OUT=<dir>` evidence hook.
  Parity fixes in the shared draw routine: rules are path fills (CG `fill(rect)` differs from the
  PDF `re f` replay by one level along rule rows); prepared coordinates quantized to the 7
  significant digits CG's PDF writer serializes (≤5e-5 pt).

- 08f8496: merge of mac-shell 312cabc (FastJSON decode, preview-controller client, nearby, …).
- 0dbdf77: pane paints its off-main bitmap correctly (observed current-frame token so a page
  requested before `setCurrent` re-requests; y-flip of the blit; page label legible on the page
  background); README "v2 preview" section rewritten; evidence directory
  `docs/evidence/mac-preview-v2-parity-2026-09-12/` (parity JSON for the math+rules list and the
  20-page list, window capture by id with FLASHTEX_NO_ACTIVATE=1, input SHA-256s).

## Completed behavior — live follow-up (branch agent/mac-preview-v2/live)

- 3d1401d: `docs/contracts/runtime-v1-display-list-v2.md` — `display-list-v2` as a per-request
  layout capability; accepting producer echoes it and writes the `--v2` envelope as ONE sibling
  line right after the compile_result (none on failure; per-request decline with a warning when
  oversize). Posted to issue #2 for mac-render-pipeline; ACKed and implemented by that lane at
  `agent/mac-render-pipeline/unified` 4888a67.
- 41c908f: consumer — `V2Source` {file, worker(id, project, revision, line)}, `setLiveV2` (pane
  visibility adds/removes the capability), `receiveDisplayListV2` (applied only for the applied
  result's id with the capability accepted; stale/unsolicited dropped and counted;
  `correlation_mismatch` refusal), LIVE / "v1 only" / "frame revision N — applied result is M"
  header, typing-bench hooks in the paint path, `fake_worker_v2.py` + PreviewV2LiveTests (7).
- 6a9bb15, fea1ef1: shared edits for the parent (WorkerClient `.displayList` event + byte-scan
  header probe, ShellModel.handle case, ProposalPreview no-op case).
- febdfc1: keeping up with typing — coalescing (one preparation in flight, newest waits),
  `RenderingV2Fast` typed reader (75 → 9 ms for the 1.9 MB demo envelope; equals Codable, falls
  back), arithmetic 7-digit quantization, CTFont cache, pre-rasterized publish (three main hops →
  one), scoped invalidation (own header view, per-page slots, Equatable pages).
- 349ca14: PDF export draws glyph-by-glyph (CG's writer drifted ~1 px at line ends with TFM-metric
  origins: 444 → 0 differing pixels); evidence `docs/evidence/mac-preview-v2-live-2026-09-12.md`.
- 72f3739: merge of mac-shell 04a4eaa (bundled latinmodern-math.otf, missing-font fixture, exact PDF
  export menu); `ExactPDFExport` adapted so a live frame's retained line is written to a temp file.

## Tests (exact evidence)

`cd apps/mac && FLASHTEX_COMPILER=… FLASHTEX_PDF=… FLASHTEX_BRIDGE=… FLASHTEX_EDIT_LEDGER=… swift test`
(release binaries from the main checkout `/Users/jay3332/Projects/flashtex/crates/*/target/release`):
at 72f3739 (after merging mac-shell 04a4eaa) **449 tests, 0 failures, 20 skipped** (all other lanes'
optional helpers: preview-controller ×10, project-files ×3, assistant-context ×2, pdf-exact,
explain helper, nearby/evidence dirs). v2 suites 41/41: RenderingV2Tests 14, PreviewV2Tests 8,
PreviewV2ShellTests 8, PreviewV2ParityTests 4, PreviewV2LiveTests 7.
Live route (`docs/evidence/mac-preview-v2-live-2026-09-12.md`, flashtex-render 4888a67 scratch build,
release app, demo.tex, 200 keystrokes, machine load 8–10): v2 pane keystroke→paint p50 89 ms / p95 116 /
max 134 at 30 ms, p50 92 / p95 121 in a burst; 200/200 painted, every frame published; v1 pane with the
same producer p50 40–41 ms. Six consecutive live frames: 0 differing pixels vs the CG export at 2 px/pt.
Earlier checkpoint (exact-glyphs branch): 339 tests green after mac-shell 312cabc.
v2 suites 33/33: RenderingV2Tests 13, PreviewV2Tests 8, PreviewV2ShellTests 8, PreviewV2ParityTests 4.
Parity measurements (V2Parity, sRGB premultiplied RGBA, AA on, font smoothing off, subpixel positioning on):
- `display-list-v2-text.json` (pipeline 7094ef7, 1 page) and `display-list-v2-math-rules.json`
  (pipeline 79ba728 scratch build, 3 typed fraction rules, LatinModernMath by hash from
  `/usr/local/texlive/2026/texmf-dist/fonts/opentype/public/lm-math`): 0 differing pixels at
  0.5, 0.75, 1, 1.25, 1.5, 1.6, 1.7, 1.8, 1.9, 2, 2.2, 2.5, 3, 4 px/pt.
- 20-page/47,025-glyph document (demo body ×10, 19.5 MB JSON): 0 differing pixels at 1.0 and
  2.0 px/pt on all 20 pages; sweep 1.00…2.00 step 0.05: 0 at 11 scales, 210–740 px at 9
  (e.g. 52 px/page at 1.37: one 0.7 px-thick en dash; CoreGraphics renders thin glyph stems
  differently through a CTFont than through the PDF-embedded font — geometry identical).
  Gate pins 1 and 2 px/pt (display scales); other scales are reported, not asserted.
- Cost (release): decode+validate 990 ms, prepare 208 ms (off-main), raster 3 ms/page at 2 px/pt,
  CG PDF export 197 ms (20 pages).

## Known limitations

- Live route: p50 89 ms vs 40 ms for the v1 pane with the same producer — the producer serializes
  the 2 MB line (+15 ms round trip), preparation 12 ms + pre-raster 3 ms off-main, and one display
  frame per main-thread hop; the preview-controller (helper) route does not forward the sibling
  line (contract § consumer action; preview-controller owner's follow-up).
- `FLASHTEX_V2_PARITY_OUT` shares `V2Loader.queue` with the live route; not a latency measurement.

- The pipeline's own `--pdf` is still the legacy v1 writer (Times-Roman/Symbol, `(?)` for math
  glyphs, 3-decimal coordinates): not a v2-exact export, so parity is against the Mac CG export;
  the product exporter is crates/pdf (mac-pdf lane).
- `flashtex-render` still emits `opentype-cff` and SHA-256(bytes‖face0) (documented deviations).
- No render-format negotiation with the worker (file input only); no clip/rotation/image primitives.
- latinmodern-math.otf is not bundled in apps/mac/Fonts; math lists refuse unless a search
  directory holds it (tests skip the math-rules parity case without it).

## Running commands / pending

- Background: none. App instances launched for evidence were killed. Scratch: flashtex-render
  builds from 79ba728 (`pv2-render`) and 4888a67 (`pv2-live`), bench logs under `pv2/bench`.
- Pending for the parent: cherry-pick or re-apply 6a9bb15 + fea1ef1 (WorkerClient/ShellModel/
  ProposalPreview) and the ExactPDFExport adaptation in 72f3739; record the contract on main.

## Immediate next steps (lane continuation candidates)

1. Re-measure the live route on a quiet machine; then cut the remaining per-keystroke costs:
   ask the producer lane about a leaner line (the 2 MB envelope is 1.98 MB of JSON for two pages;
   compact number encoding or omitting `advance_*`/carets when unused), and overlap preparation of
   page N+1 with rasterization of page N.
2. Preview-controller route: forward the sibling line through the helper's `update` framing
   (preview-controller owner) and consume it here.
3. In-app timing of hover/caret repaints (bitmap blit + overlays; not measured in-app).
4. Preview-vs-pipeline-PDF parity once a v2-exact producer `--pdf` writer exists.

Dependency SHAs: mac-shell 04a4eaa merged; origin/main dd75e15 reviewed; render-pipeline unified
4888a67 (scratch build for the live evidence; 79ba728 for the fixtures); rendering-core validate
reviewed on origin/main.
Resources: shared Claude Max 20x quota with parent mac-claude-a; no purchases, no paid network calls;
MacTeX fonts used as font assets only (never the TeX engine in the product path).
Context usage: the harness does not expose an exact percentage to me; well below the 60% checkpoint
threshold by token accounting (≈15% of a 1M window at this checkpoint).
Updated: 2026-09-12T10:40:00Z
