# Helper display-candidate route — keystroke → v2 paint (2026-09-12T142134Z)

Machine: Apple M1 Max, macOS 26.3.1. App: `agent/mac-helper-display/route-applied` @ `c5fa71cd`. Helper: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ae7ddcfb739815fac/crates/preview-controller/target/release/flashtex-preview-controller` (sha256 c95a26a3a80fa32a…). Producer: `/private/tmp/claude-501/-Users-jay3332-Projects-flashtex/e30fd4a4-f46a-4c3f-a28c-cbb8617b4425/scratchpad/render-9aaec57a/crates/render-pipeline/target/release/flashtex-render` (sha256 ed729b02befeb7d3…, built from origin/agent/mac-render-pipeline/unified 9aaec57a).

Paint point (from the app's summary): the v2 pane's bitmap blit of the validated candidate frame for the typed revision; the v1 control uses PreviewView's Canvas pass. Both are `first main-queue turn after the run-loop iteration whose SwiftUI render pass evaluated PreviewView for the revision (Canvas draw closures of every page ran inside that pass); the CoreAnimation commit has completed, the display's next vsync scan-out is not observed`.

Seeds:

```
seed p3   body x2   11708 bytes ->  4 page(s); v2 sibling: 3887910 bytes
seed p27  body x14  81296 bytes -> 28 page(s); v2 sibling: none (display_list_declined by the producer)
seed pmax body x4   23306 bytes ->  8 page(s); v2 sibling: 7793756 bytes
```

| cell | keystrokes | painted | coalesced | p50 ms | p95 ms | p99 ms | max ms | v1 previews | cand. admitted | painted | refused | invalid | dropped@paint | v2 declined | session failed | frame cap | validation p50/p95 ms | load before→after |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---|---|
| helper-v1-p27-0ms | 200 | 200 | 178 | 196 | 236 | 247 | 252 | 24 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8 MiB | — | 4.4→4.2 (load < 8 during the cell) |
| helper-v1-p27-30ms | 200 | 200 | 146 | 202 | 241 | 250 | 263 | 55 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8 MiB | — | 4.7→4.4 (load < 8 during the cell) |
| helper-v1-p3-0ms | 200 | 200 | 52 | 61 | 80 | 154 | 188 | 149 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8 MiB | — | 4.8→4.7 (load < 8 during the cell) |
| helper-v1-p3-30ms | 200 | 200 | 11 | 61 | 83 | 96 | 125 | 191 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8 MiB | — | 4.6→4.8 (load < 8 during the cell) |
| helper-v1-pmax-0ms | 200 | 200 | 133 | 80 | 98 | 109 | 116 | 68 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8 MiB | — | 4.8→4.7 (load < 8 during the cell) |
| helper-v1-pmax-30ms | 200 | 200 | 69 | 81 | 104 | 115 | 123 | 132 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8 MiB | — | 4.7→4.8 (load < 8 during the cell) |
| helper-v2-p27-0ms | 0 | 0 | 0 | — | — | — | — | 3 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 8 MiB | — | 5.8→4.3 (load < 8 during the cell) |
| helper-v2-p27-30ms | 0 | 0 | 0 | — | — | — | — | 3 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 8 MiB | — | 6.8→5.8 (load < 8 during the cell) |
| helper-v2-p3-0ms | 200 | 200 | 173 | 209 | 286 | 333 | 378 | 29 | 29 | 28 | 0 | 0 | 1 | 0 | 0 | 8 MiB | 23.3/23.7 | 6.3→6.0 (load < 8 during the cell) |
| helper-v2-p3-30ms | 200 | 200 | 157 | 296 | 405 | 435 | 452 | 45 | 45 | 44 | 0 | 0 | 1 | 0 | 0 | 8 MiB | 27.7/33.4 | 4.2→6.3 (load < 8 during the cell) |
| helper-v2-pmax-0ms | 200 | 200 | 186 | 359 | 474 | 502 | 526 | 16 | 16 | 15 | 0 | 0 | 1 | 0 | 0 | 8 MiB | 46.2/46.9 | 6.7→6.8 (load < 8 during the cell) |
| helper-v2-pmax-30ms | 200 | 200 | 168 | 382 | 481 | 560 | 623 | 35 | 35 | 33 | 0 | 0 | 2 | 0 | 0 | 8 MiB | 46.1/47.3 | 6.0→6.7 (load < 8 during the cell) |
| helper-v2cap15-pmax-0ms | 200 | 200 | 186 | 354 | 475 | 494 | 506 | 16 | 16 | 15 | 0 | 0 | 1 | 0 | 0 | 15 MiB | 46.4/47.1 | 4.6→4.6 (load < 8 during the cell) |
| helper-v2cap15-pmax-30ms | 200 | 200 | 167 | 365 | 459 | 485 | 487 | 35 | 35 | 34 | 0 | 0 | 1 | 0 | 0 | 15 MiB | 46.0/47.5 | 4.3→4.6 (load < 8 during the cell) |

Reading: `helper-v2-*` cells paint ONLY when a candidate passed the native gate (session/request/generation/source-version correlation, off-main RenderingV2 + font + page validation, durable-text sha256/byte_length binding, paint-time recheck); a keystroke whose candidate was dropped by the helper's optional slot or refused natively is covered by the next painted revision (`coalesced`). `helper-v1-*` is the same helper/producer painting the v1 result. A p27 cell with `v2 declined` > 0 and 0 candidates is the producer's documented fallback (its v2 sibling exceeds the transport caps): those keystrokes cannot paint in the v2 pane, the bench never observes a first v2 paint and records 0 keystrokes / 0 painted (`worker never attached/painted`) — that is the fallback evidence, not a latency number; the `helper-v1-p27-*` control is that seed's latency through the same helper. `pmax` is the largest demo-body multiple whose sibling fits the helper's compiler-frame cap (8 MiB by default; `helper-v2cap15-*` cells raise it to the helper's 15 MiB maximum through `FLASHTEX_CONTROLLER_MAX_FRAME_BYTES`). A sibling over the helper's cap but under the producer's 16 MiB line limit does not drop the frame: the helper fails the compiler session (`session failed` > 0, the v1 pane stops updating until a restart) — the oversized-frame case is a session loss, not a candidate drop.

Not claimed: pixel parity with any PDF raster; an isolated-machine latency (see the load column); behaviour of producers other than the pinned build.
