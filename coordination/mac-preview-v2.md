# mac-preview-v2 handoff

Agent / task / branch: mac-preview-v2 (Claude Code subagent, parent mac-claude-a) /
rendering-v2 display-list consumer for the Mac preview: exact glyph runs by original
GID and typed rules from `flashtex-render --v2` envelopes, off-main immutable page
preparation with stale-paint suppression, zero-tolerance export/preview pixel parity /
`agent/mac-preview-v2/exact-glyphs` (base `origin/agent/mac-claude-a/mac-shell` 6b43a3a;
previous lane branch `agent/mac-claude-a/preview-v2` bc28c59 was rebased here)
State: in progress
Worktree: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a93c86d3bd8db3ed3`
Owned paths (lane, per `coordination/machines/mac-m1max-a-resume.json`):
`apps/mac/Sources/FlashTeXMac/PreviewV2View.swift`, `apps/mac/Tests/FlashTeXMacTests/PreviewV2Tests.swift`.
Also carried from the released lane (no other owner; required by the owned files):
`apps/mac/Sources/FlashTeXMac/GlyphRunRenderer.swift`, `apps/mac/Sources/FlashTeXProtocol/RenderingV2.swift`,
`apps/mac/Tests/FlashTeXMacTests/RenderingV2Tests.swift`, `apps/mac/Tests/FlashTeXMacTests/Fixtures/display-list-v2-*`,
this handoff and `coordination/agents/mac-preview-v2.json`.
Parent-retained files are touched ONLY in commit 8cb9ade "shared shell edits for parent
integration" (ShellModel.swift +2, ContentView.swift +4/−1, FlashTeXMacApp.swift +1, README +65);
the parent cherry-picks or re-applies it. Never touched: crates/font-engine, paragraph-layout, math-layout.

Main integrated through: origin/main dd75e15 reviewed (coordination-only changes since b1cf8b9);
code base is mac-shell 6b43a3a (mac-shell has since advanced to 312cabc — merge pending at the next checkpoint).

## Completed behavior (tip 00e2e76, pushed)

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

## Tests (exact evidence)

`cd apps/mac && FLASHTEX_COMPILER=… FLASHTEX_PDF=… FLASHTEX_BRIDGE=… FLASHTEX_EDIT_LEDGER=… swift test`
(release binaries from the main checkout `/Users/jay3332/Projects/flashtex/crates/*/target/release`):
218 tests, 0 failures, 0 skipped (log: scratch `pv2/fulltest.log`).
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

- The pipeline's own `--pdf` is still the legacy v1 writer (Times-Roman/Symbol, `(?)` for math
  glyphs, 3-decimal coordinates): not a v2-exact export, so parity is against the Mac CG export;
  the product exporter is crates/pdf (mac-pdf lane).
- `flashtex-render` still emits `opentype-cff` and SHA-256(bytes‖face0) (documented deviations).
- No render-format negotiation with the worker (file input only); no clip/rotation/image primitives.
- latinmodern-math.otf is not bundled in apps/mac/Fonts; math lists refuse unless a search
  directory holds it (tests skip the math-rules parity case without it).

## Running commands / pending

- Background: none (scratch flashtex-render build finished; tests finished).
- Pending: merge origin/agent/mac-claude-a/mac-shell 312cabc; window screenshot by id with
  `FLASHTEX_NO_ACTIVATE=1`; `FLASHTEX_V2_PARITY_OUT` evidence into `docs/evidence/`;
  README v2 section update; then measure hover/caret repaint cost.

## Immediate next steps

1. `git merge origin/agent/mac-claude-a/mac-shell` → `swift test` → commit.
2. Launch `.build/debug/FlashTeXMac` with `FLASHTEX_NO_ACTIVATE=1 FLASHTEX_V2_FILE=<big.v2.json>
   FLASHTEX_V2_PARITY_OUT=<dir>`; `screencapture -l <windowid>`; write
   `docs/evidence/mac-preview-v2-parity-2026-09-12.md` (+ parity.json, one PNG).
3. README "v2 preview" section: threading model, parity gate, measured numbers.

Dependency SHAs: origin/main dd75e15; mac-shell base 6b43a3a (advanced 312cabc);
render-pipeline unified 79ba728 (scratch build used for fixtures; ba5611f current, display.rs unchanged);
rendering-core validate reviewed on origin/main.
Resources: shared Claude Max 20x quota with parent mac-claude-a; no purchases, no paid network calls;
MacTeX fonts used as font assets only (never the TeX engine in the product path).
Context usage: the harness does not expose an exact percentage to me; well below the 60% checkpoint
threshold by token accounting (≈15% of a 1M window at this checkpoint).
Updated: 2026-09-12T05:20:00Z
