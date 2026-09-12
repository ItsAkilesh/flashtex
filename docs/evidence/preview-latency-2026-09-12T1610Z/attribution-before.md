# v2 preview typing-to-paint: stage attribution BEFORE optimizing

Owner: mac-preview-latency (parent mac-claude-a), 2026-09-12T16:10Z. Status: measured
from the parent's helper-route bench logs of 2026-09-12T142134Z
(`docs/evidence/helper-display-route-2026-09-12T1356Z/summary.md`, app
`agent/mac-helper-display/route-applied` c5fa71cd, helper c95a26a3…, producer
flashtex-render 9aaec57a). Per-keystroke stage stamps were already in those logs
(`TypingBench.isBenchActive` lines); `tools/typing-bench/timeline.py` (this lane)
aggregates them per painted revision. Load during those cells: 4–7 (1-min average).

Columns (ms, p50 over painted v2 revisions; the last keystroke each paint covered):

| stage | meaning |
|---|---|
| key->send | last keystroke of the revision -> `compile: sending` (debounce 0 ms + the in-flight edit release held for the previous sibling) |
| send->v1 | request sent -> v1 preview applied on main (helper + producer round trip, transport, FastJSON decode) |
| v1->recv | v1 applied -> `display_candidate` frame decoded on the reader thread (producer sibling serialization + helper parse/forward + pipe + outer FastJSON parse) |
| recv->val | admission on main -> validation job started on `V2Loader.queue` |
| validate | `RenderingV2` decode (fast reader) + validate + font resolve + `V2Frame.prepare` + durable-text binding |
| preraster | `GlyphRunRenderer.rasterize` of every page at the pane's scale |
| deliver | off-main finish -> main run-loop delivery |
| pub->paint | `displayListV2 = .loaded` -> bench paint point (SwiftUI pass + CA commit + next run-loop turn) |
| key->paint | keystroke -> paint for those same revisions (sum of the above plus queueing) |

| cell | painted | key->send | send->v1 | v1->recv | recv->val | validate | preraster | deliver | pub->paint | key->paint (last covered key) | bench p50 (all keys) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| helper-v2-p3-30ms (4 pages, 3.9 MB) | 43 | 14.0 | 81.8 | 64.3 | 0.2 | 27.7 | 4.3 | 1.5 | 26.7 | 237.9 | 296 |
| helper-v2-p3-0ms | 27 | 7.7 | 63.4 | 48.8 | 2.2 | 23.3 | 3.9 | 2.4 | 25.7 | 178.7 | 209 |
| helper-v2-pmax-30ms (8 pages, 7.8 MB) | 32 | 12.6 | 107.5 | 99.4 | 0.2 | 46.1 | 7.6 | 3.7 | 28.3 | 307.3 | 382 |
| helper-v2-pmax-0ms | 14 | 6.9 | 105.4 | 98.5 | 2.2 | 46.2 | 7.6 | 2.2 | 23.2 | 292.6 | 359 |

Reading (p3-30ms): of 238 ms keystroke->paint, 146 ms (61%) is the helper +
producer round trip for v1 and the sibling (`send->v1` 82 + `v1->recv` 64) and
about 75 ms is Mac-side work this lane owns: validate 28 + preraster 4 + deliver
1.5 + publish->paint 27 + key->send 14. The bench p50 over ALL keystrokes (296)
is higher than the last-covered-key figure (238) because keystrokes typed while a
cycle is in flight wait for the next cycle: with the in-flight release held for
the sibling (`FLASHTEX_DISPLAY_CANDIDATES_HOLD`, needed because the helper
forwards a candidate only for unchanged source and evicts its optional slot on
any required enqueue), one cycle is `key->send + send->v1 + v1->recv` ≈ 160 ms,
so a keystroke waits on average half a cycle plus a full cycle plus the Mac-side
tail. Only the Mac-side tail is reducible from this repository's Swift code; the
cycle length is a helper/producer property (findings for their owners are in
`summary.md`).

Inside `validate` (27.7 ms for 4 pages): the fast reader decodes ~3.9 MB, then
every page is validated, font-resolved and prepared although pages 1–3 are
byte-identical to the previous sibling (checked on the producer's output:
`siblings.py`, canonical JSON of pages 1–3 equal between `p3` and `p3-typed`).
`pub->paint` (27 ms) includes a SwiftUI pass whose measured body->last-draw
time was 17 ms p50 (`render_pass_ms` in the raw JSON): every page view
re-evaluated (frame token changed), every visible page re-blitted twice per
keystroke (the stale toggle, then the new frame), and the caret lookup walked
every cluster of every page.

Not claimed: an isolated-machine number; a direct-route (no helper) figure — the
direct v2 route is measured after the optimizations, with the same script, in
`summary.md`.
