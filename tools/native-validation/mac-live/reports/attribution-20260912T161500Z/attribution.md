# Typing latency on the packaged app, attributed per stage (attribution-20260912T161500Z)

App: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a19c42dc3e488a6de/tools/native-validation/mac-live/build/app/fb16d07d1e0d062a59188fb267e98227a33c2d28/apps/mac/build/FlashTeX.app`. Bundled binaries (sha256): `FlashTeX` 0dc406a0c4a7, `flashtex-bridge` 6be9a94978d5, `flashtex-compiler` 78be02f13089, `flashtex-edit-ledger` ab15be4a26b2, `flashtex-pdf` 3387a4f0f784, `flashtex-pdf-exact` 0519b1f7629e, `flashtex-preview-controller` 4ef8898f4a95, `flashtex-render` 79c2be4c5859.
`components.json`: app=fb16d07d, bridge=3377748c, compiler=3377748c, edit_ledger=3377748c, pdf=3377748c, pdf_exact=20e52778, preview_controller=3377748c, render=9aaec57a
Seeds: `body60k` 63899 bytes (sha256 6e83f04d4c0f); `demo` 5909 bytes (sha256 97313ec9b1cb); `fixture` 16 bytes (sha256 f93bafb765f7); `hw1` 5126 bytes (sha256 f725e23897df); `render27` 76929 bytes (sha256 b7c8468d4398). Typed script sha256 8fddfb8d3231, 30 ms interval.
Quiet gate: 1-minute load < 8.0 before each cell (waited up to 240 s); load-affected = load before/after > 10.0. `uptime` at start `12:15  up 1 day, 11:07, 3 users, load averages: 34.53 30.61 21.00`, end `12:40  up 1 day, 11:33, 3 users, load averages: 9.33 24.98 31.05`. Other FlashTeX at start: 83388 FlashTeX.

Stages (ms, p50 over painted revisions whose own keystroke is the last one the paint covers; see the module docstring for the log lines): key->send = keystroke to `compile: sending`; producer = sending to the last decoded line minus decode; decode = reader JSON decode; ->main = decoded to `event on main`; apply = main to `compile: applied`; helper = sending to applied (helper route, one stage); v2 wait/prepare/preraster/deliver/publish from the `preview-v2:` lines; paint = applied (or v2 published) to `paint:`. Producer CPU = child processes' cumulative CPU (`ps -o cputime`, last sample) / results decoded.

## Route `v1`

| seed | bench p50 / p95 / max ms | painted / keys / coalesced | key->send | producer | decode | ->main | apply | helper | v2 wait | v2 prepare | v2 preraster | v2 deliver | v2 publish | paint | dominant stage | producer CPU ms/result | load before -> after | 200 ms target |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| fixture | 23.296042 / 40.958291 / 46.478542 | 200 / 200 / 0 | 0.6 | 0.5 | 0.1 | 5.3 | 0.5 | — | — | — | — | — | — | 15.5 | paint 66% | 0.25 | 16.2 -> 17.6 (load-affected) | met |
| demo | 28.743542 / 55.839792 / 78.035583 | 200 / 200 / 1 | 0.7 | 1.5 | 3.1 | 2.9 | 1.0 | — | — | — | — | — | — | 18.3 | paint 64% | 1.4 | 26.4 -> 24.3 (load-affected) | met |
| hw1 | 82.362 / 125.238541 / 218.241625 | 200 / 200 / 1 | 1.0 | 4.2 | 3.7 | 19.9 | 1.5 | — | — | — | — | — | — | 51.7 | paint 63% | 2.95 | 26.1 -> 27.3 (load-affected) | met |
| render27 | 187.750125 / 239.016333 / 453.702875 | 200 / 200 / 129 | 24.4 | 51.4 | 45.2 | 0.1 | 8.5 | — | — | — | — | — | — | 21.7 | producer 32% | 48.73 | 19.9 -> 17.8 (load-affected) | NOT met |
| body60k | 937.706291 / 1502.1835 / 1769.024042 | 200 / 200 / 188 | 32.8 | 95.3 | 455.6 | 1.5 | 12.8 | — | — | — | — | — | — | 39.2 | decode 70% | 27.5 | 32.7 -> 48.1 (load-affected) | NOT met |

## Route `v2`

| seed | bench p50 / p95 / max ms | painted / keys / coalesced | key->send | producer | decode | ->main | apply | helper | v2 wait | v2 prepare | v2 preraster | v2 deliver | v2 publish | paint | dominant stage | producer CPU ms/result | load before -> after | 200 ms target |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| demo | 768.364375 / 1139.762958 / 1382.551875 | 200 / 200 / 180 | 22.9 | 435.4 | 14.6 | 3.4 | 1.7 | — | 0.1 | 25.3 | 4.8 | 6.6 | 0.1 | 40.9 | producer 75% | 41.5 | 71.4 -> 78.3 (load-affected) | NOT met |
| hw1 | 1549.391958 / 1663.139583 / 2153.262166 | 82 / 82 / 0 | 1.2 | 43.9 | 16.1 | 76.0 | 2.2 | — | 0.0 | 20.2 | 9.1 | 179.2 | 0.2 | 1197.8 | paint 77% | 30.12 | 60.0 -> 83.4 (load-affected) | NOT met |
| render27 | None / None / None | 0 / 0 / 0 | — | — | — | — | — | — | — | — | — | — | — | — | ? | None | 58.0 -> 33.8 (load-affected) | NOT met |
| body60k | None / None / None | 0 / 0 / 0 | — | — | — | — | — | — | — | — | — | — | — | — | ? | None | 30.6 -> 27.3 (load-affected) | NOT met |

## Route `controller`

| seed | bench p50 / p95 / max ms | painted / keys / coalesced | key->send | producer | decode | ->main | apply | helper | v2 wait | v2 prepare | v2 preraster | v2 deliver | v2 publish | paint | dominant stage | producer CPU ms/result | load before -> after | 200 ms target |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| demo | 104.57125 / 157.952084 / 222.219542 | 200 / 200 / 34 | 3.9 | — | — | — | — | 46.0 | — | — | — | — | — | 38.2 | helper 46% | 10.46 | 30.0 -> 34.6 (load-affected) | met |
| hw1 | 161.047042 / 207.852125 / 284.492667 | 200 / 200 / 5 | 1.5 | — | — | — | — | 61.2 | — | — | — | — | — | 97.5 | paint 61% | 14.4 | 36.0 -> 57.4 (load-affected) | NOT met |
| render27 | 353.862209 / 532.072375 / 656.67975 | 200 / 200 / 162 | 21.8 | — | — | — | — | 216.9 | — | — | — | — | — | 47.9 | helper 76% | 132.11 | 61.0 -> 54.4 (load-affected) | NOT met |
| body60k | 283.898709 / 401.470792 / 467.281667 | 200 / 200 / 155 | 21.5 | — | — | — | — | 176.0 | — | — | — | — | — | 31.0 | helper 78% | 72.67 | 33.5 -> 34.3 (load-affected) | NOT met |

## Route `v1-render`

| seed | bench p50 / p95 / max ms | painted / keys / coalesced | key->send | producer | decode | ->main | apply | helper | v2 wait | v2 prepare | v2 preraster | v2 deliver | v2 publish | paint | dominant stage | producer CPU ms/result | load before -> after | 200 ms target |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| demo | 46.141125 / 214.647167 / 476.121625 | 200 / 200 / 17 | 0.8 | 2.9 | 3.9 | 9.2 | 1.0 | — | — | — | — | — | — | 27.7 | paint 61% | 2.75 | 21.4 -> 21.2 (load-affected) | NOT met |
| hw1 | 94.32475 / 168.264709 / 246.182833 | 200 / 200 / 2 | 1.1 | 2.7 | 5.0 | 26.6 | 1.7 | — | — | — | — | — | — | 56.0 | paint 59% | 3.0 | 22.5 -> 21.7 (load-affected) | met |
| render27 | 247.764958 / 449.457292 / 622.773916 | 200 / 200 / 142 | 27.7 | 53.7 | 68.9 | 0.9 | 8.7 | — | — | — | — | — | — | 34.8 | decode 37% | 51.03 | 11.9 -> 11.6 (load-affected) | NOT met |
| body60k | 143.272792 / 195.721333 / 236.083625 | 200 / 200 / 113 | 22.1 | 28.3 | 46.5 | 0.1 | 6.1 | — | — | — | — | — | — | 21.7 | decode 36% | 27.36 | 9.4 -> 9.3 | met |

## Per-cell uptime

- `v1-fixture-30ms`: before `12:19  up 1 day, 11:11, 3 users, load averages: 16.18 25.27 21.45`; after `12:19  up 1 day, 11:11, 3 users, load averages: 17.61 25.42 21.52`
- `v1-demo-30ms`: before `12:21  up 1 day, 11:13, 3 users, load averages: 26.38 27.05 22.71`; after `12:21  up 1 day, 11:13, 3 users, load averages: 24.29 26.58 22.59`
- `v1-hw1-30ms`: before `12:22  up 1 day, 11:14, 3 users, load averages: 26.13 26.36 22.72`; after `12:22  up 1 day, 11:15, 3 users, load averages: 27.30 26.69 22.92`
- `v1-render27-30ms`: before `12:23  up 1 day, 11:15, 3 users, load averages: 19.88 24.80 22.44`; after `12:23  up 1 day, 11:16, 3 users, load averages: 17.75 24.18 22.25`
- `v1-body60k-30ms`: before `12:24  up 1 day, 11:16, 3 users, load averages: 32.73 26.83 23.31`; after `12:24  up 1 day, 11:17, 3 users, load averages: 48.09 30.32 24.59`
- `v2-demo-30ms`: before `12:25  up 1 day, 11:17, 3 users, load averages: 71.36 39.72 28.44`; after `12:25  up 1 day, 11:18, 3 users, load averages: 78.30 42.26 29.47`
- `v2-hw1-30ms`: before `12:26  up 1 day, 11:18, 3 users, load averages: 60.00 43.38 30.65`; after `12:28  up 1 day, 11:21, 3 users, load averages: 83.40 54.53 36.62`
- `v2-render27-30ms`: before `12:29  up 1 day, 11:21, 3 users, load averages: 58.00 52.75 37.02`; after `12:30  up 1 day, 11:22, 3 users, load averages: 33.84 46.79 35.94`
- `v2-body60k-30ms`: before `12:31  up 1 day, 11:23, 3 users, load averages: 30.62 43.90 35.49`; after `12:32  up 1 day, 11:24, 3 users, load averages: 27.27 40.15 34.64`
- `controller-demo-30ms`: before `12:33  up 1 day, 11:25, 3 users, load averages: 29.97 39.26 34.64`; after `12:33  up 1 day, 11:25, 3 users, load averages: 34.60 39.69 34.90`
- `controller-hw1-30ms`: before `12:34  up 1 day, 11:26, 3 users, load averages: 35.97 39.14 34.96`; after `12:34  up 1 day, 11:27, 3 users, load averages: 57.40 43.52 36.66`
- `controller-render27-30ms`: before `12:35  up 1 day, 11:28, 3 users, load averages: 61.02 48.48 39.00`; after `12:35  up 1 day, 11:28, 3 users, load averages: 54.41 47.49 38.76`
- `controller-body60k-30ms`: before `12:36  up 1 day, 11:29, 3 users, load averages: 33.45 42.81 37.57`; after `12:36  up 1 day, 11:29, 3 users, load averages: 34.30 42.83 37.60`
- `v1-render-demo-30ms`: before `12:37  up 1 day, 11:30, 3 users, load averages: 21.39 37.95 36.09`; after `12:37  up 1 day, 11:30, 3 users, load averages: 21.17 37.36 35.90`
- `v1-render-hw1-30ms`: before `12:38  up 1 day, 11:31, 3 users, load averages: 22.53 34.94 35.09`; after `12:38  up 1 day, 11:31, 3 users, load averages: 21.74 34.05 34.77`
- `v1-render-render27-30ms`: before `12:39  up 1 day, 11:32, 3 users, load averages: 11.91 29.44 33.01`; after `12:39  up 1 day, 11:32, 3 users, load averages: 11.60 28.80 32.74`
- `v1-render-body60k-30ms`: before `12:40  up 1 day, 11:33, 3 users, load averages: 9.38 25.53 31.31`; after `12:40  up 1 day, 11:33, 3 users, load averages: 9.33 24.98 31.05`

Raw: `attribution.json` (cells, processes sampled, env), `<cell>.json` (the shell's bench summary), `<cell>.attribution.json` (per painted revision stages), `<cell>.log` (FLASHTEX_LOG with the timeline).
