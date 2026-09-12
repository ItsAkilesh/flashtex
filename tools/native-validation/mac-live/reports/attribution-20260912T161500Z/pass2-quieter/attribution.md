# Typing latency on the packaged app, attributed per stage (pass2-quieter)

App: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a19c42dc3e488a6de/tools/native-validation/mac-live/build/app/fb16d07d1e0d062a59188fb267e98227a33c2d28/apps/mac/build/FlashTeX.app`. Bundled binaries (sha256): `FlashTeX` 0dc406a0c4a7, `flashtex-bridge` 6be9a94978d5, `flashtex-compiler` 78be02f13089, `flashtex-edit-ledger` ab15be4a26b2, `flashtex-pdf` 3387a4f0f784, `flashtex-pdf-exact` 0519b1f7629e, `flashtex-preview-controller` 4ef8898f4a95, `flashtex-render` 79c2be4c5859.
`components.json`: app=fb16d07d, bridge=3377748c, compiler=3377748c, edit_ledger=3377748c, pdf=3377748c, pdf_exact=20e52778, preview_controller=3377748c, render=9aaec57a
Seeds: `demo` 5909 bytes (sha256 97313ec9b1cb); `hw1` 5126 bytes (sha256 f725e23897df). Typed script sha256 8fddfb8d3231, 30 ms interval.
Quiet gate: 1-minute load < 8.0 before each cell (waited up to 30 s); load-affected = load before/after > 10.0. `uptime` at start `12:41  up 1 day, 11:33, 3 users, load averages: 6.91 23.15 30.21`, end `12:47  up 1 day, 11:40, 3 users, load averages: 33.21 21.73 26.60`. Other FlashTeX at start: 83388 FlashTeX.

Stages (ms, p50 over painted revisions whose own keystroke is the last one the paint covers; see the module docstring for the log lines): key->send = keystroke to `compile: sending`; producer = sending to the last decoded line minus decode; decode = reader JSON decode; ->main = decoded to `event on main`; apply = main to `compile: applied`; helper = sending to applied (helper route, one stage); v2 wait/prepare/preraster/deliver/publish from the `preview-v2:` lines; paint = applied (or v2 published) to `paint:`. Producer CPU = child processes' cumulative CPU (`ps -o cputime`, last sample) / results decoded.

## Route `v2`

| seed | bench p50 / p95 / max ms | painted / keys / coalesced | key->send | producer | decode | ->main | apply | helper | v2 wait | v2 prepare | v2 preraster | v2 deliver | v2 publish | paint | dominant stage | producer CPU ms/result | load before -> after | 200 ms target |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| demo | 136.592291 / 357.248042 / 577.912583 | 200 / 200 / 30 | 1.0 | 33.0 | 8.7 | 14.3 | 1.1 | — | 0.1 | 16.6 | 3.5 | 8.0 | 0.1 | 37.1 | paint 29% | 24.49 | 11.3 -> 12.6 (load-affected) | NOT met |
| hw1 | 680.045834 / 1114.393 / 1408.265292 | 171 / 171 / 5 | 0.8 | 16.8 | 7.2 | 47.2 | 1.1 | — | 0.0 | 9.7 | 3.1 | 121.7 | 0.1 | 470.3 | paint 69% | 16.69 | 14.5 -> 15.3 (load-affected) | NOT met |

## Route `v1`

| seed | bench p50 / p95 / max ms | painted / keys / coalesced | key->send | producer | decode | ->main | apply | helper | v2 wait | v2 prepare | v2 preraster | v2 deliver | v2 publish | paint | dominant stage | producer CPU ms/result | load before -> after | 200 ms target |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| demo | 28.859541 / 48.207791 / 105.191458 | 200 / 200 / 0 | 0.7 | 1.5 | 3.1 | 4.8 | 1.0 | — | — | — | — | — | — | 18.6 | paint 64% | 1.2 | 13.2 -> 12.7 (load-affected) | met |
| hw1 | 77.825625 / 92.041417 / 106.855416 | 200 / 200 / 0 | 0.9 | 2.7 | 3.2 | 26.7 | 1.5 | — | — | — | — | — | — | 48.0 | paint 61% | 2.35 | 9.9 -> 8.7 | met |

## Route `controller`

| seed | bench p50 / p95 / max ms | painted / keys / coalesced | key->send | producer | decode | ->main | apply | helper | v2 wait | v2 prepare | v2 preraster | v2 deliver | v2 publish | paint | dominant stage | producer CPU ms/result | load before -> after | 200 ms target |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| demo | 46.676875 / 59.625083 / 69.187458 | 200 / 200 / 2 | 0.9 | — | — | — | — | 23.5 | — | — | — | — | — | 16.2 | helper 50% | 4.9 | 7.9 -> 7.8 | met |
| hw1 | 147.357792 / 245.235458 / 288.368792 | 200 / 200 / 12 | 1.5 | — | — | — | — | 56.6 | — | — | — | — | — | 81.2 | paint 56% | 13.32 | 23.6 -> 33.2 (load-affected) | NOT met |

## Per-cell uptime

- `v2-demo-30ms`: before `12:41  up 1 day, 11:34, 3 users, load averages: 11.30 22.51 29.72`; after `12:42  up 1 day, 11:34, 3 users, load averages: 12.55 22.26 29.51`
- `v2-hw1-30ms`: before `12:42  up 1 day, 11:35, 3 users, load averages: 14.51 21.77 29.08`; after `12:44  up 1 day, 11:37, 3 users, load averages: 15.30 20.76 27.75`
- `v1-demo-30ms`: before `12:45  up 1 day, 11:37, 3 users, load averages: 13.20 19.70 27.12`; after `12:45  up 1 day, 11:37, 3 users, load averages: 12.70 19.49 27.00`
- `v1-hw1-30ms`: before `12:45  up 1 day, 11:38, 3 users, load averages: 9.93 18.17 26.26`; after `12:46  up 1 day, 11:38, 3 users, load averages: 8.69 17.35 25.78`
- `controller-demo-30ms`: before `12:46  up 1 day, 11:38, 3 users, load averages: 7.91 16.89 25.52`; after `12:46  up 1 day, 11:39, 3 users, load averages: 7.75 16.71 25.41`
- `controller-hw1-30ms`: before `12:46  up 1 day, 11:39, 3 users, load averages: 23.60 19.25 25.95`; after `12:47  up 1 day, 11:40, 3 users, load averages: 33.21 21.73 26.60`

Raw: `attribution.json` (cells, processes sampled, env), `<cell>.json` (the shell's bench summary), `<cell>.attribution.json` (per painted revision stages), `<cell>.log` (FLASHTEX_LOG with the timeline).
