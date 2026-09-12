# v2 preview typing-to-paint latency — rounds and attribution

Lane `mac-preview-latency` (parent `mac-claude-a`, mac-m1max-a). Harness:
`run.sh` (interleaved before/after cells, real FlashTeXMac release builds,
`FLASHTEX_NO_ACTIVATE=1`, TypingBench 200 keystrokes), per-stage attribution
`tools/typing-bench/timeline.py`, tables `summarize.py <raw dir>...`.
`uptime` is recorded per cell (load before -> after); cells above the 1-min
load limit 15 are marked load-affected. Producer: scratch `flashtex-render`
build 9aaec57a; helper: this tree's `flashtex-preview-controller`.

Builds: `before` = 1c07cf93 (instrumentation only); `after`/`after1` =
95a75546 (page reuse by raw-byte identity, per-page bitmaps/views);
`after2` = + layer-backed page blit; `after3` = + equatable/lazy diagnostics
list (247e1cdb); `after4` = + pages at one structural position for loaded and
stale frames (this round, below).

## Rounds raw/ (round 1), raw-after2/ (layer blit), raw-hw1/ (HW1.tex)

| raw | build | route | seed | interval | keystrokes | painted | coalesced | no-redraw paints | p50 | p95 | p99 | max | load before→after |
|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| raw | after | direct-v2 | p3 | 0 ms | 200 | 200 | 53 | 16 | 144 | 230 | 285 | 302 | 9.4→9.3 |
| raw | after | direct-v2 | p3 | 30 ms | 200 | 200 | 25 | 21 | 132 | 209 | 241 | 263 | 11.1→10.1 |
| raw | after | direct-v2 | pmax | 0 ms | 200 | 200 | 144 | 0 | 184 | 238 | 248 | 262 | 14.0→13.2 |
| raw | after | direct-v2 | pmax | 30 ms | 200 | 200 | 116 | 0 | 199 | 247 | 271 | 298 | 13.0→12.7 |
| raw | after | helper-v2 | p3 | 0 ms | 200 | 200 | 171 | 0 | 202 | 272 | 311 | 332 | 9.9→9.7 |
| raw | after | helper-v2 | p3 | 30 ms | 200 | 200 | 144 | 1 | 220 | 278 | 302 | 337 | 12.1→11.9 |
| raw | after | helper-v2 | pmax | 0 ms | 0 | 0 | 0 | 0 | — | — | — | — | 7.9→26.8 (load-affected) |
| raw | after | helper-v2 | pmax | 30 ms | 0 | 0 | 0 | 0 | — | — | — | — | 13.2→16.0 (load-affected) |
| raw | after | helper-v2cap15 | pmax | 0 ms | 200 | 200 | 187 | 0 | 367 | 581 | 715 | 726 | 14.7→14.3 |
| raw | after | helper-v2cap15 | pmax | 30 ms | 200 | 200 | 176 | 0 | 534 | 913 | 999 | 1033 | 12.8→15.5 (load-affected) |
| raw | before | direct-v2 | p3 | 0 ms | 200 | 200 | 19 | 41 | 147 | 188 | 202 | 216 | 9.7→9.4 |
| raw | before | direct-v2 | p3 | 30 ms | 200 | 200 | 12 | 37 | 176 | 302 | 394 | 443 | 11.9→11.1 |
| raw | before | direct-v2 | pmax | 0 ms | 200 | 200 | 174 | 7 | 330 | 410 | 438 | 456 | 14.8→15.8 (load-affected) |
| raw | before | direct-v2 | pmax | 30 ms | 200 | 200 | 109 | 10 | 241 | 279 | 289 | 302 | 13.8→13.0 |
| raw | before | helper-v2 | p3 | 0 ms | 200 | 200 | 173 | 4 | 220 | 297 | 328 | 378 | 10.1→9.9 |
| raw | before | helper-v2 | p3 | 30 ms | 200 | 200 | 147 | 21 | 239 | 297 | 327 | 359 | 13.7→12.1 |
| raw | before | helper-v2 | pmax | 0 ms | 0 | 0 | 0 | 0 | — | — | — | — | 12.7→7.9 |
| raw | before | helper-v2 | pmax | 30 ms | 0 | 0 | 0 | 0 | — | — | — | — | 9.3→13.2 |
| raw | before | helper-v2cap15 | pmax | 0 ms | 200 | 200 | 196 | 1 | 754 | 1390 | 1466 | 1506 | 15.0→19.2 (load-affected) |
| raw | before | helper-v2cap15 | pmax | 30 ms | 200 | 200 | 168 | 7 | 390 | 492 | 547 | 579 | 12.7→12.8 |
| raw-after2 | after | direct-v2 | p3 | 0 ms | 200 | 200 | 19 | 0 | 111 | 128 | 138 | 153 | 14.2→12.0 |
| raw-after2 | after | direct-v2 | p3 | 30 ms | 200 | 200 | 13 | 0 | 111 | 168 | 207 | 236 | 14.1→14.2 |
| raw-after2 | after | direct-v2 | pmax | 0 ms | 200 | 200 | 151 | 0 | 170 | 227 | 242 | 243 | 4.8→5.1 |
| raw-after2 | after | direct-v2 | pmax | 30 ms | 200 | 200 | 155 | 0 | 299 | 445 | 522 | 559 | 6.5→7.9 |
| raw-after2 | after | helper-v2 | p3 | 0 ms | 200 | 200 | 179 | 0 | 205 | 290 | 325 | 361 | 14.2→14.2 |
| raw-after2 | after | helper-v2 | p3 | 30 ms | 200 | 200 | 151 | 0 | 242 | 357 | 406 | 441 | 11.3→14.1 |
| raw-after2 | after | helper-v2 | pmax | 0 ms | 0 | 0 | 0 | 0 | — | — | — | — | 7.9→4.8 |
| raw-after2 | after | helper-v2 | pmax | 30 ms | 0 | 0 | 0 | 0 | — | — | — | — | 12.0→6.5 |
| raw-after2 | after | helper-v2cap15 | pmax | 0 ms | 200 | 200 | 187 | 0 | 319 | 441 | 448 | 454 | 4.9→4.8 |
| raw-after2 | after | helper-v2cap15 | pmax | 30 ms | 200 | 200 | 166 | 0 | 343 | 428 | 451 | 481 | 5.1→4.9 |
| raw-hw1 | after1 | direct-v2 | hw1 | 30 ms | 0 | 0 | 0 | 0 | — | — | — | — | 3.3→2.4 |
| raw-hw1 | after2 | direct-v2 | hw1 | 30 ms | 0 | 0 | 0 | 0 | — | — | — | — | 2.4→2.8 |
| raw-hw1 | after3 | direct-v2 | hw1 | 30 ms | 200 | 200 | 20 | 7 | 98 | 125 | 139 | 149 | 4.5→4.0 |
| raw-hw1 | after3 | helper-v2 | hw1 | 30 ms | 200 | 200 | 127 | 0 | 171 | 246 | 259 | 265 | 3.1→3.3 |
| raw-hw1 | before | direct-v2 | hw1 | 30 ms | 200 | 200 | 0 | 199 | 600 | 642 | 655 | 673 | 5.4→4.5 |
| raw-hw1 | before | helper-v2 | hw1 | 30 ms | 200 | 200 | 165 | 34 | 804 | 2330 | 2735 | 3395 | 4.0→3.1 |

Per-stage p50 (ms) over painted v2 revisions (timeline.py):

| raw | build | route | seed | interval | painted revs | pages (reused) | key->send | send->v1 | v1->recv | recv->val | validate | preraster | deliver | pub->paint | key->paint | helper parse / serialize |
|---|---|---|---|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| raw | after | direct-v2 | p3 | 0 ms | 147 | 4 (3) | 2.4 | 55.4 | 1.4 | 10.2 | 15.8 | 1.4 | 5.8 | 32.3 | 136.7 | — |
| raw | after | direct-v2 | p3 | 30 ms | 175 | 4 (3) | 9.9 | 52.7 | 1.2 | 10.8 | 15.6 | 1.4 | 5.1 | 30.5 | 129.1 | — |
| raw | after | direct-v2 | pmax | 0 ms | 56 | 8 (7) | 8.9 | 89.1 | 13.1 | 3.3 | 24.8 | 1.4 | 5.2 | 23.6 | 169.6 | — |
| raw | after | direct-v2 | pmax | 30 ms | 84 | 8 (7) | 15.1 | 89.3 | 13.8 | 1.4 | 25.1 | 1.3 | 4.5 | 23.0 | 176.0 | — |
| raw | after | helper-v2 | p3 | 0 ms | 29 | 4 (3) | 7.2 | 62.0 | 47.9 | 3.0 | 15.4 | 1.4 | 7.0 | 35.1 | 179.6 | 17.4 / 6.2 |
| raw | after | helper-v2 | p3 | 30 ms | 56 | 4 (3) | 24.0 | 60.7 | 51.2 | 0.2 | 15.3 | 1.3 | 7.4 | 24.2 | 182.8 | 17.5 / 6.5 |
| raw | after | helper-v2 | pmax | 0 ms | 0 | — | — | — | — | — | — | — | — | — | — | — |
| raw | after | helper-v2 | pmax | 30 ms | 0 | — | — | — | — | — | — | — | — | — | — | — |
| raw | after | helper-v2cap15 | pmax | 0 ms | 13 | 8 (7) | 7.4 | 106.3 | 100.8 | 1.3 | 25.5 | 1.4 | 2.7 | 30.9 | 282.2 | 34.7 / 13.7 |
| raw | after | helper-v2cap15 | pmax | 30 ms | 24 | 8 (7) | 10.1 | 156.9 | 152.7 | 0.2 | 29.6 | 1.4 | 1.8 | 26.4 | 408.9 | 42.4 / 21.5 |
| raw | before | direct-v2 | p3 | 0 ms | 181 | 4 | 11.5 | 54.1 | -2.7 | 8.5 | 23.5 | 5.2 | 6.7 | 32.2 | 143.2 | — |
| raw | before | direct-v2 | p3 | 30 ms | 188 | 4 | 10.3 | 62.7 | 0.1 | 8.8 | 26.7 | 6.2 | 8.2 | 38.9 | 174.7 | — |
| raw | before | direct-v2 | pmax | 0 ms | 26 | 8 | 10.4 | 141.9 | 25.7 | 2.1 | 47.1 | 10.7 | 1.8 | 31.2 | 277.1 | — |
| raw | before | direct-v2 | pmax | 30 ms | 91 | 8 | 36.0 | 90.6 | 9.3 | 7.1 | 45.8 | 10.1 | 0.0 | 20.6 | 226.3 | — |
| raw | before | helper-v2 | p3 | 0 ms | 27 | 4 | 7.3 | 61.8 | 50.5 | 2.3 | 23.5 | 5.2 | 1.0 | 30.1 | 182.9 | 18.4 / 6.4 |
| raw | before | helper-v2 | p3 | 30 ms | 53 | 4 | 25.3 | 62.5 | 51.6 | 0.2 | 23.4 | 5.1 | 1.0 | 25.9 | 200.3 | 18.2 / 6.5 |
| raw | before | helper-v2 | pmax | 0 ms | 0 | — | — | — | — | — | — | — | — | — | — | — |
| raw | before | helper-v2 | pmax | 30 ms | 0 | — | — | — | — | — | — | — | — | — | — | — |
| raw | before | helper-v2cap15 | pmax | 0 ms | 4 | 8 | 11.7 | 163.0 | 138.2 | 2.6 | 46.8 | 11.5 | 1.1 | 33.0 | 441.5 | 38.1 / — |
| raw | before | helper-v2cap15 | pmax | 30 ms | 32 | 8 | 14.5 | 111.3 | 100.4 | 0.2 | 45.8 | 10.1 | 0.0 | 20.0 | 309.5 | 35.7 / 13.7 |
| raw-after2 | after | direct-v2 | p3 | 0 ms | 181 | 4 (3) | 8.6 | 47.3 | 2.8 | 8.7 | 15.4 | 1.3 | 4.5 | 16.3 | 110.2 | — |
| raw-after2 | after | direct-v2 | p3 | 30 ms | 187 | 4 (3) | 2.1 | 48.2 | 3.6 | 8.5 | 15.4 | 1.4 | 4.6 | 18.4 | 108.7 | — |
| raw-after2 | after | direct-v2 | pmax | 0 ms | 49 | 8 (7) | 7.7 | 88.1 | 13.3 | 2.3 | 24.6 | 1.4 | 5.1 | 14.5 | 157.4 | — |
| raw-after2 | after | direct-v2 | pmax | 30 ms | 45 | 8 (7) | 13.6 | 136.9 | 28.5 | 0.3 | 25.3 | 1.4 | 2.2 | 16.9 | 238.2 | — |
| raw-after2 | after | helper-v2 | p3 | 0 ms | 21 | 4 (3) | 6.8 | 60.2 | 50.1 | 1.9 | 15.3 | 1.4 | 3.6 | 19.2 | 161.1 | 17.5 / 6.4 |
| raw-after2 | after | helper-v2 | p3 | 30 ms | 49 | 4 (3) | 23.9 | 65.7 | 57.1 | 0.2 | 15.3 | 1.4 | 7.7 | 17.0 | 193.6 | 18.1 / 6.5 |
| raw-after2 | after | helper-v2 | pmax | 0 ms | 0 | — | — | — | — | — | — | — | — | — | — | — |
| raw-after2 | after | helper-v2 | pmax | 30 ms | 0 | — | — | — | — | — | — | — | — | — | — | — |
| raw-after2 | after | helper-v2cap15 | pmax | 0 ms | 14 | 8 (7) | 6.0 | 104.0 | 99.0 | 1.6 | 24.9 | 1.4 | 3.3 | 14.8 | 261.1 | 34.4 / 13.1 |
| raw-after2 | after | helper-v2cap15 | pmax | 30 ms | 34 | 8 (7) | 13.8 | 104.8 | 100.6 | 0.2 | 24.8 | 1.4 | 0.5 | 19.7 | 267.2 | 34.8 / 13.2 |
| raw-hw1 | after1 | direct-v2 | hw1 | 30 ms | 0 | — | — | — | — | — | — | — | — | — | — | — |
| raw-hw1 | after2 | direct-v2 | hw1 | 30 ms | 0 | — | — | — | — | — | — | — | — | — | — | — |
| raw-hw1 | after3 | direct-v2 | hw1 | 30 ms | 180 | 3 (2) | 0.6 | 27.6 | -7.2 | 15.9 | 5.7 | 0.8 | 19.1 | 37.6 | 97.6 | — |
| raw-hw1 | after3 | helper-v2 | hw1 | 30 ms | 73 | 3 (2) | 7.6 | 35.1 | 15.5 | 6.7 | 5.7 | 0.8 | 14.9 | 34.2 | 120.1 | 6.2 / 2.3 |
| raw-hw1 | before | direct-v2 | hw1 | 30 ms | 200 | 3 | 0.6 | 68.7 | -47.8 | 47.9 | 8.6 | 2.7 | 74.1 | 455.2 | 603.3 | — |
| raw-hw1 | before | helper-v2 | hw1 | 30 ms | 35 | 3 | 19.4 | 54.8 | -4.1 | 41.5 | 8.5 | 2.7 | 74.4 | 458.9 | 635.0 | 6.2 / 2.3 |

Notes on the tables above:

- `helper-v2 pmax` cells (0 painted) are not app failures: the producer
  declines the 7.8 MB pmax sibling through the helper's default 8 MiB frame
  cap (`compiler declined layout capabilities`); `helper-v2cap15` raises
  `FLASHTEX_CONTROLLER_MAX_FRAME_BYTES` to 15 MiB and paints.
- `raw-hw1/after1` and `after2` (0 painted, "worker never attached/painted")
  ran binaries copied out of `.build/release`; see the copied-binary control
  cell in the after4 section for whether that is a harness artifact.
- Negative `v1->recv` p50s: the `display_list` sibling line is received on
  the reader thread before the main thread finishes applying the
  `compile_result`; the stages are stamped on different threads.

## HW1 attribution (fixtures/real-world/hw1/HW1.tex, 130 recovered diagnostics)

Before (1c07cf93, direct v2, 30 ms): p50 600 ms, pub->paint 455 ms, 199 of
200 paints "without redraw". The dominant paint cost was the v2 pane's own
diagnostics list: an eager 130-row `VStack` under the pages, rebuilt on every
frame (and torn down / rebuilt on every loaded<->stale toggle). After3
(equatable `V2DiagnosticsList`, lazy rows, bounded height): p50 98 ms,
pub->paint 38 ms; helper route 804 -> 171 ms.

What remains on HW1 after3 (raw-hw1/after3-direct-v2-hw1-30ms.log, revision
60, ms after the keystroke): send 0.5; v1 line decoded 17.5 (producer 18 ms);
applied 19.1; display_list received 20.3; **main thread free again only at
36.1** (recv->val 15.8: the v1 apply with 130 diagnostics — editor underline
marks, `ContentView.diagnosticsList` grouping/list rows, status — plus the
full-window SwiftUI pass); prepare 5.6 + preraster 0.8 off-main; **delivered
14.8 ms later** (stale pass + blits + the next keystroke); published 57.4;
**pane pass at 77.0** (publish->pass 19.6: another full-window pass); blit
78.5; paint stamp 101. Three full-window SwiftUI passes per keystroke at
15-20 ms each (vs ~8 ms on the 0-diagnostic p3 document) are the remaining
HW1 cost; they are parent-side (`ContentView.diagnosticsList`, editor marks)
and are reported, not edited by this lane.

The bench's HW1/p3 keystroke lands at the end of the document (page 3 of 3 /
page 4 of 4), which the `LazyVStack` never materializes at the bench window
size: `blit page 3` never appears. The recorded paint of after1..3 is the
re-install of the visible pages 1-2 caused by the structural toggle fixed in
after4; before after4 those pages re-blitted twice per keystroke (374 blits
of an unchanged page over 187 p3 revisions, 346 on HW1).
