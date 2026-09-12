# Helper display-candidate route — keystroke → v2 paint (2026-09-12T140200Z)

Machine: Apple M1 Max, macOS 26.3.1. App: `agent/mac-helper-display/route-applied` @ `a2c5a21e`. Helper: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ae7ddcfb739815fac/crates/preview-controller/target/release/flashtex-preview-controller` (sha256 c95a26a3a80fa32a…). Producer: `/private/tmp/claude-501/-Users-jay3332-Projects-flashtex/e30fd4a4-f46a-4c3f-a28c-cbb8617b4425/scratchpad/render-src/crates/render-pipeline/target/release/flashtex-render` (sha256 d8d2040e7e8d5716…, built from origin/agent/mac-render-pipeline/unified f762f82a).

Paint point (from the app's summary): the v2 pane's bitmap blit of the validated candidate frame for the typed revision; the v1 control uses PreviewView's Canvas pass. Both are `first main-queue turn after the run-loop iteration whose SwiftUI render pass evaluated PreviewView for the revision (Canvas draw closures of every page ran inside that pass); the CoreAnimation commit has completed, the display's next vsync scan-out is not observed`.

Seeds:

```
seed p3   body x2   11708 bytes ->  4 page(s); v2 sibling: 3887910 bytes
seed p27  body x14  81296 bytes -> 28 page(s); v2 sibling: none (display_list_declined by the producer)
```

| cell | keystrokes | painted | coalesced | p50 ms | p95 ms | p99 ms | max ms | v1 previews | cand. admitted | painted | refused | invalid | dropped@paint | v2 declined | validation p50/p95 ms | load before→after |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---|
| helper-v1-p3-30ms | 200 | 200 | 11 | 61 | 75 | 101 | 155 | 190 | 0 | 0 | 0 | 0 | 0 | 0 | — | 29.9→26.2 (under shared load, not an isolated result) |
| helper-v2-p3-30ms | 200 | 200 | 158 | 275 | 416 | 445 | 479 | 44 | 44 | 43 | 0 | 0 | 1 | 0 | 23.6/29.1 | 32.0→29.9 (under shared load, not an isolated result) |

Reading: `helper-v2-*` cells paint ONLY when a candidate passed the native gate (session/request/generation/source-version correlation, off-main RenderingV2 + font + page validation, durable-text sha256/byte_length binding, paint-time recheck); a keystroke whose candidate was dropped by the helper's optional slot or refused natively is covered by the next painted revision (`coalesced`). `helper-v1-*` is the same helper/producer painting the v1 result. A p27 cell with `v2 declined` > 0 and 0 candidates is the producer's documented fallback (its v2 sibling exceeds the transport caps): those keystrokes cannot paint in the v2 pane and `painted` is expected to be 0 there — that is the fallback evidence, not a latency number.

Not claimed: pixel parity with any PDF raster; an isolated-machine latency (see the load column); behaviour of producers other than the pinned build.
