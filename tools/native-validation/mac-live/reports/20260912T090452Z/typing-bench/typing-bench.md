# Typing bench: keystroke → paint latency (2026-09-12T090644Z)

Branch `HEAD` @ `d8baed6`; Apple M1 Max; macOS 26.3.1; release build of `FlashTeXMac` (`swift build -c release`).
Script: `tools/typing-bench/run.sh`; raw JSON summaries in `typing-bench-2026-09-12T090644Z/`.

Producers:
- crates/compiler @ d8baed6 (this checkout)

## Results

Latency is keystroke → paint per typed character (ms); a coalesced keystroke is measured to the first paint that showed it. `compile` is the shell's send → result time on the main thread (waits behind any render pass in progress); `render` is PreviewView body → last page Canvas draw.

| producer | seed | bytes | interval | keys typed | paints | coalesced | k→p p50 | p95 | p99 | max | compile p50 | compile p95 | render p50 | render p95 | unpainted |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| flashtex-compiler | body60k | 64103 | 0ms | 200 | 78 | 122 | 354 | 628 | 794 | 906 | 174 | 375 | 31 | 46 | 0 |
| flashtex-compiler | body60k | 64103 | 30ms | 200 | 144 | 56 | 230 | 394 | 439 | 463 | 105 | 214 | 24 | 35 | 0 |
| flashtex-compiler | demo | 6113 | 0ms | 200 | 186 | 14 | 29 | 47 | 51 | 59 | 10.0 | 22 | 12 | 15 | 0 |
| flashtex-compiler | demo | 6113 | 30ms | 200 | 196 | 4 | 28 | 46 | 48 | 55 | 9.7 | 21 | 13 | 15 | 0 |
| flashtex-compiler | fixture | 220 | 0ms | 200 | 197 | 3 | 15 | 28 | 37 | 39 | 0.7 | 11 | 10 | 14 | 0 |
| flashtex-compiler | fixture | 220 | 30ms | 200 | 200 | 0 | 10 | 21 | 38 | 41 | 0.7 | 6.6 | 2.1 | 10 | 0 |

Typed script: `tools/typing-bench/typed-200.txt` (200 characters, inserted before `\end{document}` when present, else at the end).
Seeds: `demo` = `apps/mac/Samples/demo.tex`; `body60k` = the demo's paragraphs repeated to ≥ 60 KB in one document; `fixture` = the entry document of `protocol/fixtures/compile-request.json`.

## Methodology

- The app is launched with `FLASHTEX_AUTOATTACH=1 FLASHTEX_NO_ACTIVATE=1 FLASHTEX_SEED_FILE=<seed> FLASHTEX_TYPING_BENCH=<script>` and the producer as `FLASHTEX_COMPILER`. After the worker attached and its first result was painted, `TypingBenchDriver` (`apps/mac/Sources/FlashTeXMac/TypingBench.swift`) inserts the script one extended grapheme cluster at a time into the real editor `NSTextView` through `insertText(_:replacementRange:)` from a main-run-loop `Timer` at the configured interval (30 ms ≈ a fast typist; 0 ms = one keystroke per run-loop turn, a burst). Each insertion takes the production path: `NSTextViewDelegate.textDidChange` → SwiftUI binding → `ShellModel.updateActiveText` (revision bump, `keystroke:` log line) → auto-compile (`FLASHTEX_DEBOUNCE_MS` default 0, one request in flight, newest buffer coalesced) → `compile_result` → `PreviewView` render.
- Keystroke time is stamped immediately before `insertText` on the monotonic clock (`clock_gettime_nsec_np(CLOCK_UPTIME_RAW)`, i.e. `mach_absolute_time` in ns). For a person typing, the same recorder uses the `NSEvent.timestamp` of the `keyDown` seen by an in-process local event monitor (HID time on the same clock) — no Accessibility permission or event tap is involved, and a key that changes no text is discarded at the end of its dispatch.
- Paint time (`paint:` log line) is the first main-queue turn after the run-loop iteration whose SwiftUI render pass evaluated `PreviewView.body` for the result revision; the page `Canvas` draw closures run inside that pass and are counted, so the CoreAnimation commit containing the new pages has completed when the stamp is taken. A revision whose result changed nothing visible (SwiftUI skipped the canvas redraw) is still recorded as painted, flagged `redrawn: false`.
- A paint of revision N makes every unpainted keystroke with revision ≤ N visible; those with revision < N are `coalesced`. p50/p95/p99 are nearest-rank percentiles over the per-keystroke latencies. The run ends when every keystroke is painted (or after the settle timeout), and the app writes the JSON summary and exits.

## Limitations

- The bench inserts text programmatically: there is no OS keyboard event, no event-queue wait, no key repeat and no input-method composition; real typing adds the HID → WindowServer → `NSApplication.sendEvent` hop, which the local-monitor path measures but this bench cannot.
- `paint` is the completed CoreAnimation commit, not the display scan-out: the pixels reach the panel at the next vsync (up to one frame, 8–17 ms at 60–120 Hz) after the stamp, and later still if the render server is behind. No IOSurface presentation callback is observed. The window is ordered back (`FLASHTEX_NO_ACTIVATE=1`) and may be occluded during the run; commits still happen, on-screen visibility is not verified.
- Compile time is measured on the main thread from send to result application, so it includes any time the reply waited behind a render pass; the producers' own round trip is 1–6 ms for these documents when measured directly.
- Typing stops after `FLASHTEX_TYPING_BENCH_MAX_MS` (120 s here); a cell marked 'of 200 (budget)' typed fewer characters because each keystroke waited for a main-thread render pass.
- One machine, one run per cell, no warm-up discard beyond the first compile; numbers are indicative, not a regression gate.
