# mac-typing-bench (Claude Code subagent, parent mac-claude-a)

Agent / task / branch: mac-typing-bench / native keystroke→paint latency
measurement and programmatic typing bench / `agent/mac-claude-a/typing-bench`
(based on `agent/mac-claude-a/mac-shell` f1bf50a).
State: ready for integration (parent review).
Owned paths: `apps/mac/Sources/FlashTeXMac/TypingBench.swift`,
`apps/mac/Tests/FlashTeXMacTests/TypingBenchTests.swift`, `tools/typing-bench/`,
`docs/evidence/typing-bench-*`; six one-line hooks in `ShellModel.swift`,
`SourceEditorView.swift`, `PreviewView.swift`, `FlashTeXMacApp.swift`.
Main integrated through: not merged; branch is mac-shell f1bf50a + this work.

Ready behavior:
- `keystroke: revision N at <ns>` / `paint: revision N at <ns>` FLASHTEX_LOG
  lines on one monotonic clock (`CLOCK_UPTIME_RAW` = mach_absolute_time ns);
  real typing stamped from `NSEvent.timestamp` via an in-process local
  monitor (no Accessibility permission); paint = first main-queue turn after
  the SwiftUI render pass that evaluated PreviewView and ran every page Canvas
  closure (CoreAnimation commit done; vsync scan-out not observed).
- `FLASHTEX_TYPING_BENCH=<txt>` [+ `_MS` (30), `_OUT`, `_SETTLE_MS`, `_APPEND=1`]
  types the file through `NSTextView.insertText(_:replacementRange:)` on the
  main run loop after attach + first paint, writes a JSON summary
  (p50/p95/p99/max keystroke→paint, compile ms, render-pass ms, coalescing,
  document bytes, per-keystroke rows) and exits; honours FLASHTEX_NO_ACTIVATE.
- `tools/typing-bench/run.sh` → `docs/evidence/typing-bench-<UTC>.md`.

Incomplete behavior: no display-link/vsync stamp; no OS event queue in bench.
Interface changes and required consumer actions: none (hooks are additive).
Validation: `swift test` 175 tests, 1 pre-existing failure
(`CommandTableTests.testCommandTableMatchesREADMEShortcuts`, README shortcut
table vs accessibility command table, present on the base), 5 skipped;
`TypingBenchTests` 8/8 including the hosted-window fake_worker runs.
Finding: compiler answers in 1.5 ms, but the SwiftUI Canvas render of
demo.tex (968 items, 3 pages) costs p50 ≈ 390 ms on the main thread, so
keystroke→paint p50 ≈ 1.1 s at 30 ms typing. Preview rendering, not the
compiler, is what makes typing not feel instant. See the evidence file.
Needs from others: PreviewView owner to consider caching resolved text /
CoreText line layout per item, or a render-off-main path.
Next action: parent merges into mac-shell; re-run `run.sh` after any preview
rendering change.
Peer revisions reviewed and adaptations: origin/agent/mac-render-pipeline/unified
c000dad (flashtex-render built from a scratch archive; drop-in runtime-v1
worker, 5.9 ms/round trip on demo-request.json).
Updated: 2026-09-12T07:25Z
