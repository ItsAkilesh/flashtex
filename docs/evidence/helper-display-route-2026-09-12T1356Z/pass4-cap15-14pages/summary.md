# Helper display-candidate route — keystroke → v2 paint (2026-09-12T142516Z)

Machine: Apple M1 Max, macOS 26.3.1. App: `agent/mac-helper-display/route-applied` @ `c5fa71cd`. Helper: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ae7ddcfb739815fac/crates/preview-controller/target/release/flashtex-preview-controller` (sha256 c95a26a3a80fa32a…). Producer: `/private/tmp/claude-501/-Users-jay3332-Projects-flashtex/e30fd4a4-f46a-4c3f-a28c-cbb8617b4425/scratchpad/render-9aaec57a/crates/render-pipeline/target/release/flashtex-render` (sha256 ed729b02befeb7d3…, built from origin/agent/mac-render-pipeline/unified 9aaec57a).

Paint point (from the app's summary): the v2 pane's bitmap blit of the validated candidate frame for the typed revision; the v1 control uses PreviewView's Canvas pass. Both are `first main-queue turn after the run-loop iteration whose SwiftUI render pass evaluated PreviewView for the revision (Canvas draw closures of every page ran inside that pass); the CoreAnimation commit has completed, the display's next vsync scan-out is not observed`.

Seeds:

```
seed pmax body x7   40703 bytes -> 14 page(s); v2 sibling: 13655250 bytes
```

| cell | keystrokes | painted | coalesced | p50 ms | p95 ms | p99 ms | max ms | v1 previews | cand. admitted | painted | refused | invalid | dropped@paint | v2 declined | session failed | frame cap | validation p50/p95 ms | load before→after |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---|---|
| helper-v1-pmax-30ms | 200 | 200 | 124 | 136 | 254 | 304 | 313 | 77 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8 MiB | — | 6.4→9.4 (under shared load, not an isolated result) |
| helper-v2-pmax-30ms | 0 | 0 | 0 | — | — | — | — | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 8 MiB | — | 4.8→4.9 (load < 8 during the cell) |
| helper-v2cap15-pmax-30ms | 200 | 200 | 195 | 1239 | 2205 | 2446 | 2506 | 22 | 20 | 7 | 0 | 0 | 13 | 0 | 0 | 15 MiB | 92.5/113.8 | 4.9→6.4 (load < 8 during the cell) |

Reading: `helper-v2-*` cells paint ONLY when a candidate passed the native gate (session/request/generation/source-version correlation, off-main RenderingV2 + font + page validation, durable-text sha256/byte_length binding, paint-time recheck); a keystroke whose candidate was dropped by the helper's optional slot or refused natively is covered by the next painted revision (`coalesced`). `helper-v1-*` is the same helper/producer painting the v1 result. A p27 cell with `v2 declined` > 0 and 0 candidates is the producer's documented fallback (its v2 sibling exceeds the transport caps): those keystrokes cannot paint in the v2 pane, the bench never observes a first v2 paint and records 0 keystrokes / 0 painted (`worker never attached/painted`) — that is the fallback evidence, not a latency number; the `helper-v1-p27-*` control is that seed's latency through the same helper. `pmax` is the largest demo-body multiple whose sibling fits the helper's compiler-frame cap (8 MiB by default; `helper-v2cap15-*` cells raise it to the helper's 15 MiB maximum through `FLASHTEX_CONTROLLER_MAX_FRAME_BYTES`). A sibling over the helper's cap but under the producer's 16 MiB line limit does not drop the frame: the helper fails the compiler session (`session failed` > 0, the v1 pane stops updating until a restart) — the oversized-frame case is a session loss, not a candidate drop.

Not claimed: pixel parity with any PDF raster; an isolated-machine latency (see the load column); behaviour of producers other than the pinned build.
