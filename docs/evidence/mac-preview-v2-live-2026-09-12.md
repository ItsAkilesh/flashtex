# Mac v2 preview live while typing: keystroke→paint and export parity on live frames

Owner: mac-preview-v2 (Claude Code subagent, parent mac-claude-a). Measured 2026-09-12 on
mac-m1max-a, branch `agent/mac-preview-v2/live`. Linked from `coordination/mac-preview-v2.md`,
the apps/mac README "v2 preview" section and `docs/contracts/runtime-v1-display-list-v2.md`.
Artifacts: `mac-preview-v2-live-2026-09-12/` (bench summaries, per-frame parity reports, window
capture by id with `FLASHTEX_NO_ACTIVATE=1`, one keystroke's timeline, input SHA-256s).

## Setup

- Producer: `flashtex-render` built from a scratch archive of
  `origin/agent/mac-render-pipeline/unified` **4888a67** ("display-list-v2 sibling line" — the
  render-pipeline lane's ACK and implementation of the proposal), `cargo build --release`;
  SHA-256 in `inputs.sha256`. Fonts: `apps/mac/Fonts` (Latin Modern).
- App: `apps/mac` release build of this branch, launched as the typing bench does
  (`tools/typing-bench/run.sh` cell): `FLASHTEX_AUTOATTACH=1 FLASHTEX_NO_ACTIVATE=1
  FLASHTEX_COMPILER=<flashtex-render> FLASHTEX_SEED_FILE=apps/mac/Samples/demo.tex
  FLASHTEX_TYPING_BENCH=tools/typing-bench/typed-200.txt`, plus `FLASHTEX_PREVIEW_V2=1` for the
  v2 cells (the pane requests `display-list-v2`; every compile answers with the 1.98 MB
  `display_list` sibling line). Paint point for the v2 cells: the render pass that blits a page
  bitmap of the frame's revision (`TypingBench.willRender`/`didDraw` hooks in
  `PreviewV2View.swift`); v1 cells use the existing `PreviewView` hooks. Same 200-keystroke
  script, 30 ms and 0 ms (burst) intervals. Machine shared with other agents (load average
  8–10 during the recorded cells; 13–32 during earlier runs, which were discarded).

## Keystroke → paint (release, demo.tex, 200 keystrokes)

| cell | p50 | p95 | p99 | max | painted | compile p50/p95 | notes |
|---|---|---|---|---|---|---|---|
| v1 pane, 30 ms (`bench-v1-demo-30ms-3`) | 40 ms | 44 | 54 | 57 | 200/200 | 15 / 19 ms | 225 KB result line |
| **v2 pane, 30 ms** (`bench-v2-demo-30ms-3`) | **89 ms** | 116 | 134 | 134 | 200/200 | 30 / 41 ms | 200 lines of 1.98 MB, 200 frames published, 0 coalesced |
| v1 pane, 0 ms burst (`bench-v1-demo-0ms-3`) | 41 ms | 57 | 62 | 66 | 200/200 | 15 / 25 ms | |
| **v2 pane, 0 ms burst** (`bench-v2-demo-0ms-3`) | **92 ms** | 121 | 130 | 132 | 200/200 | 31 / 41 ms | 170 frames published, 0 coalesced |

Two earlier interleaved rounds (`-1`, `-2`): v2 30 ms p50 99 / 89 ms, v1 41 / 39 ms.

Where the v2 keystroke goes (`trace-v2-30ms-revision-100.txt`, typical): compile round trip
~27–30 ms (the producer also serializes the 2 MB line; 15 ms without it), v2 line probe 2 ms on
the reader thread and delivery to the main run loop, off-main preparation 12 ms (fast reader 9,
validate 1–2, prepare ≤2) plus pre-rasterization 2.6 ms for two pages, one main-thread pass that
publishes the frame and blits the pre-installed bitmaps, then the paint point on the next turn.
Under continuous invalidation each main-thread hop costs about one display frame.

What it took to get here (first live run, same producer): p50 20.3 s — every ~110 ms preparation
was superseded by the next 30 ms keystroke and dropped, so only the last frame painted (strict
supersession starves visible progress). Fixes, all in this lane's files: coalescing (one
preparation in flight, newest waits, in-between dropped undecoded) → p50 258 ms; typed fast
envelope reader (75 → 9 ms), arithmetic 7-digit quantization (~15 → 0 ms), CTFont per (font,
size) → p50 ~115 ms; byte-scan line probe (12 → 2 ms), scoped invalidation (own header view,
per-page bitmap slots, Equatable pages) and pre-rasterized publish (three main hops → one) →
p50 89 ms.

## Export parity on live frames (tolerance 0)

`FLASHTEX_V2_PARITY_OUT` during a 5-keystroke live session (`parity-live-mac-*-r*.json`):
six consecutive live frames (revisions 2–7, two pages each, 2 px/pt), each compared byte-for-byte
against the CoreGraphics PDF export rasterized back — **0 differing pixels on every page, preview
and export bitmap SHA-256 equal**.

Regression found and fixed on the way: with the producer's TeX/TFM metrics (4888a67), glyph
origins no longer coincide with the OpenType advances, and CoreGraphics' PDF writer — which merges
a run's glyphs into `Tj` strings positioned by the font's advances plus integer 1/1000 em `TJ`
adjustments — drifted by up to a pixel at line ends (444 differing pixels per two-page frame).
`GlyphRunRenderer.draw` now emits one glyph per `CTFontDrawGlyphs` call on PDF contexts
(`glyphByGlyph`), so each glyph gets its own `Tm`/`Td` at the 7 significant digits the
preparation already quantized to; bitmap contexts keep the batched call (positions are taken
exactly). Cost: export of the two-page demo 132 ms instead of ~20 ms (export/evidence path only,
off-main). Sweep on the 4888a67 demo envelope: 0 differing pixels at 1, 1.2, 1.37, 1.5, 1.8, 2,
3 px/pt; 10 at 0.5 (the documented thin-stem residual).

## Limits

- The producer's `--pdf` is still the legacy v1 writer, so parity is against the Mac CoreGraphics
  export; the product exporter is crates/pdf.
- The preview-controller (helper) route does not forward the sibling line yet (contract §
  consumer action); the live route works on the direct worker route only.
- `FLASHTEX_V2_PARITY_OUT` competes with the live route for `V2Loader.queue`; a parity session
  is not a latency measurement (its bench summary is not reported above).
- Numbers were taken on a shared machine; re-measure on a quiet one before quoting them as the
  route's floor.
