# Typing bench: keystroke → paint latency (2026-09-12T111202Z)

Branch `HEAD` @ `7ccbd7e9`; Apple M1 Max; macOS 26.3.1; release build of `FlashTeXMac` (`swift build -c release`).
Script: `tools/typing-bench/run.sh`; raw JSON summaries in `typing-bench-2026-09-12T111202Z/`.

Producers:
- compiler: crates/compiler @ 7ccbd7e9 (this checkout)
- render: crates/render-pipeline/target/release (this checkout)
- controller: crates/preview-controller/target/release (this checkout), owning compiler /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a19c42dc3e488a6de/tools/native-validation/mac-live/build/app/7ccbd7e9d330479b3f35927bbf968316270b7495/crates/compiler/target/release/flashtex-compiler

## Results

Latency is keystroke → paint per typed character (ms); a coalesced keystroke is measured to the first paint that showed it. `compile` is the shell's send → result time on the main thread; `render` is PreviewView body → last page Canvas draw. A cell marked ⚠ ran while the 1-minute load average exceeded the limit (10.0) before or after it; such cells are reported, never used as a gate.

### Route `compiler`

`compiler`: direct worker route — `FLASHTEX_COMPILER` = main's `flashtex-compiler`, runtime-v1 JSON Lines on stdin/stdout, one request in flight.

| route | producer | seed | bytes | interval | keys typed | paints | coalesced | k→p p50 | p95 | p99 | max | compile p50 | compile p95 | render p50 | render p95 | unpainted | load before/after |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| compiler | flashtex-compiler | body60k | 64103 | 0ms | 200 | 61 | 139 | 97 | 113 | 118 | 130 | 53 | 57 | 17 | 19 | 0 | 7.5 / 7.4 |
| compiler | flashtex-compiler | body60k | 64103 | 30ms | 200 | 118 | 82 | 94 | 119 | 125 | 137 | 52 | 61 | 7.8 | 19 | 0 | 6.6 / 7.5 |
| compiler | flashtex-compiler | demo | 6113 | 0ms | 200 | 186 | 14 | 25 | 41 | 49 | 70 | 6.9 | 18 | 11 | 15 | 0 | 6.8 / 6.6 |
| compiler | flashtex-compiler | demo | 6113 | 30ms | 200 | 200 | 0 | 23 | 42 | 44 | 48 | 6.9 | 17 | 6.3 | 14 | 0 | 6.9 / 6.8 |
| compiler | flashtex-compiler | fixture | 220 | 0ms | 200 | 192 | 8 | 21 | 39 | 41 | 44 | 6.1 | 15 | 11 | 14 | 0 | 7.8 / 6.9 |
| compiler | flashtex-compiler | fixture | 220 | 30ms | 200 | 200 | 0 | 23 | 41 | 43 | 46 | 6.4 | 18 | 9.6 | 12 | 0 | 7.7 / 7.8 |

### Route `render`

`render`: direct worker route with `flashtex-render` (render-pipeline branch) as the runtime-v1 worker.

| route | producer | seed | bytes | interval | keys typed | paints | coalesced | k→p p50 | p95 | p99 | max | compile p50 | compile p95 | render p50 | render p95 | unpainted | load before/after |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| render | flashtex-render | body60k | 64103 | 0ms | 200 | 36 | 164 | 130 | 155 | 160 | 167 | 75 | 77 | 21 | 22 | 0 | 6.9 / 6.5 |
| render | flashtex-render | body60k | 64103 | 30ms | 200 | 83 | 117 | 132 | 151 | 171 | 182 | 73 | 78 | 11 | 22 | 0 | 7.0 / 6.9 |
| render | flashtex-render | demo | 6113 | 0ms | 200 | 169 | 31 | 41 | 51 | 55 | 56 | 15 | 22 | 15 | 17 | 0 | 7.0 / 7.0 |
| render | flashtex-render | demo | 6113 | 30ms | 200 | 189 | 11 | 41 | 46 | 50 | 57 | 16 | 19 | 15 | 17 | 0 | 7.0 / 7.0 |
| render | flashtex-render | fixture | 220 | 0ms | 200 | 186 | 14 | 24 | 40 | 42 | 42 | 6.3 | 17 | 11 | 14 | 0 | 7.2 / 7.0 |
| render | flashtex-render | fixture | 220 | 30ms | 200 | 200 | 0 | 22 | 40 | 43 | 44 | 6.3 | 17 | 9.5 | 12 | 0 | 7.4 / 7.2 |

### Route `controller`

`controller`: durable helper route — `FLASHTEX_PREVIEW_CONTROLLER` = `flashtex-preview-controller` (crates/preview-controller on origin/main), which owns the edit ledger (fsync per edit under a temporary `FLASHTEX_CONTROLLER_LEDGER_ROOT`), the lexical index and main's compiler; previews arrive as asynchronous `update` frames.

| route | producer | seed | bytes | interval | keys typed | paints | coalesced | k→p p50 | p95 | p99 | max | compile p50 | compile p95 | render p50 | render p95 | unpainted | load before/after |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| controller | flashtex-preview-controller | body60k | 64103 | 0ms | 200 | 29 | 171 | 149 | 190 | 201 | 203 | 45 | 58 | 6.9 | 18 | 0 | 6.2 / 5.9 |
| controller | flashtex-preview-controller | body60k | 64103 | 30ms | 200 | 64 | 136 | 164 | 206 | 229 | 239 | 47 | 59 | 16 | 18 | 0 | 6.1 / 6.2 |
| controller | flashtex-preview-controller | demo | 6113 | 0ms | 200 | 181 | 20 | 52 | 69 | 75 | 79 | 16 | 24 | 11 | 18 | 0 | 6.0 / 6.1 |
| controller | flashtex-preview-controller | demo | 6113 | 30ms | 200 | 177 | 23 | 55 | 116 | 145 | 169 | 16 | 40 | 11 | 17 | 0 | 6.3 / 6.0 |
| controller | flashtex-preview-controller | fixture | 220 | 0ms | 200 | 153 | 48 | 50 | 91 | 121 | 131 | 18 | 31 | 13 | 24 | 0 | 6.5 / 6.3 |
| controller | flashtex-preview-controller | fixture | 220 | 30ms | 200 | 198 | 2 | 47 | 62 | 68 | 72 | 19 | 30 | 12 | 16 | 0 | 6.5 / 6.5 |

### Cross-route comparison (keystroke → paint p50 / p95 ms)

| seed | interval | compiler | render | controller |
|---|---:|---:|---:|---:|
| body60k | 30ms | 94 / 119 | 132 / 151 | 164 / 206 |
| body60k | 0ms | 97 / 113 | 130 / 155 | 149 / 190 |
| demo | 30ms | 23 / 42 | 41 / 46 | 55 / 116 |
| demo | 0ms | 25 / 41 | 41 / 51 | 52 / 69 |
| fixture | 30ms | 23 / 41 | 22 / 40 | 47 / 62 |
| fixture | 0ms | 21 / 39 | 24 / 40 | 50 / 91 |

Typed script: `tools/typing-bench/typed-200.txt` (200 characters, inserted before `\end{document}` when present, else at the end).
Seeds: `demo` = `apps/mac/Samples/demo.tex`; `body60k` = the demo's paragraphs repeated to ≥ 60 KB in one document; `fixture` = the entry document of `protocol/fixtures/compile-request.json`.

## Methodology

- The app is launched with `FLASHTEX_AUTOATTACH=1 FLASHTEX_NO_ACTIVATE=1 FLASHTEX_SEED_FILE=<seed> FLASHTEX_TYPING_BENCH=<script>` plus the route's environment (`FLASHTEX_COMPILER=<worker>` for the direct routes; `FLASHTEX_PREVIEW_CONTROLLER=<helper>` with a fresh temporary `FLASHTEX_CONTROLLER_LEDGER_ROOT` per cell for the durable route; `FLASHTEX_PREVIEW_V2=1` for the v2 pane). After the producer attached and its first result was painted, `TypingBenchDriver` (`apps/mac/Sources/FlashTeXMac/TypingBench.swift`) inserts the script one extended grapheme cluster at a time into the real editor `NSTextView` through `insertText(_:replacementRange:)` from a main-run-loop `Timer` at the configured interval (30 ms ≈ a fast typist; 0 ms = one keystroke per run-loop turn, a burst). Each insertion takes the production path: `NSTextViewDelegate.textDidChange` → SwiftUI binding → `ShellModel.updateActiveText` (revision bump, `keystroke:` log line) → auto-compile / `edit` submission (one in flight, newest buffer coalesced) → result → `PreviewView` render.
- Keystroke time is stamped immediately before `insertText` on the monotonic clock (`clock_gettime_nsec_np(CLOCK_UPTIME_RAW)`, i.e. `mach_absolute_time` in ns). For a person typing, the same recorder uses the `NSEvent.timestamp` of the `keyDown` seen by an in-process local event monitor (HID time on the same clock) — no Accessibility permission or event tap is involved, and a key that changes no text is discarded at the end of its dispatch.
- Paint time (`paint:` log line): first main-queue turn after the run-loop iteration whose SwiftUI render pass evaluated PreviewView for the revision (Canvas draw closures of every page ran inside that pass); the CoreAnimation commit has completed, the display's next vsync scan-out is not observed. A revision whose result changed nothing visible (SwiftUI skipped the canvas redraw) is still recorded as painted, flagged `redrawn: false`.
- A paint of revision N makes every unpainted keystroke with revision ≤ N visible; those with revision < N are `coalesced`. p50/p95/p99 are nearest-rank percentiles over the per-keystroke latencies. The run ends when every keystroke is painted (or after the settle timeout), and the app writes the JSON summary and exits.
- Machine load: `sysctl vm.loadavg` (1-minute average) is recorded before and after every cell; run.sh waits for the load to drop below `--quiet-load` before each cell (bounded by `--quiet-wait`) and flags cells whose before/after load exceeded `--load-limit`.

## Limitations

- The bench inserts text programmatically: there is no OS keyboard event, no event-queue wait, no key repeat and no input-method composition; real typing adds the HID → WindowServer → `NSApplication.sendEvent` hop, which the local-monitor path measures but this bench cannot.
- `paint` is the completed CoreAnimation commit, not the display scan-out: the pixels reach the panel at the next vsync (up to one frame, 8–17 ms at 60–120 Hz) after the stamp, and later still if the render server is behind. No IOSurface presentation callback is observed. The window is ordered back (`FLASHTEX_NO_ACTIVATE=1`) and may be occluded during the run; commits still happen, on-screen visibility is not verified.
- Compile time is measured on the main thread from send to result application, so it includes any time the reply waited behind main-thread work; for the durable route it is edit submission → preview `update` applied (the helper's fsync and compile are inside it).
- Typing stops after `FLASHTEX_TYPING_BENCH_MAX_MS` (120 s here); a cell marked 'of 200 (budget)' typed fewer characters because each keystroke waited for main-thread work.
- One machine, one run per cell, no warm-up discard beyond the first compile; numbers are indicative, not a regression gate. Load-affected cells are marked, not excluded.
