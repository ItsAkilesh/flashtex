# mac-validation-4 handoff — typing latency attribution on the packaged app

- Updated UTC: 2026-09-12T12:20:00Z
- Agent / parent / machine alias: `mac-validation-4` (Claude Code subagent) / `mac-claude-a` / `mac-m1max-a`
- Task: Commander replenishment (issue #2 comment 5646989044), native validation item 4.
  Current task: typing latency on the CURRENT packaged binaries (small / medium / large
  documents; direct v1, direct v2, helper routes) with the stage dominating the 200 ms
  budget attributed per route. Follow-up 1: rerun the mac-live acceptance on the packaged
  app, PASS/FAIL deltas vs report 20260912T110944Z. Follow-up 2: repeatable quiet-window
  script (waits for 1-min load < 8, runs every cell unattended, `uptime` per cell).
- Owned paths: `tools/native-validation/mac-live/**`, `coordination/mac-validation*`.
  No product source edited (rules: never `apps/mac/Sources`, `crates/**`, `tools/typing-bench`).
- Branch: `agent/mac-validation-4/mac-live-4` from `origin/agent/mac-claude-a/mac-shell`
  cd58fc2e (main c11c005 merged); the remote tip moved to fb16d07d during the run and the
  runner packages that tip (the "current packaged binaries").

## Coverage audit (existing evidence, before any new work)

Grep of `apps/mac/Tests/FlashTeXMacTests/*`, `tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`:

| already covered | where | what it does NOT cover |
|---|---|---|
| keystroke -> paint p50/p95/p99/max per cell, compile round trip, render pass, result -> paint | `apps/mac/Tests/FlashTeXMacTests/TypingBenchTests.swift` (recorder/config/stat tests); `tools/typing-bench/run.sh` + `evidence.py`; `docs/evidence/typing-bench-2026-09-12T*.md` (11 runs, swift-built `FlashTeXMac`, producers compiler/render/controller) | packaged `FlashTeX.app` binaries; per-stage attribution (the evidence totals only); hw1 and the 27-page render fixture as seeds |
| mac-live acceptance: fixture/demo/body60k x 30/0 ms on compiler, render, controller and historical routes; launch-check; capture cycle; render-attach; exact export; windows; worker relaunch; multifile; branch XCTests | `tools/native-validation/mac-live/reports/20260912T110944Z.md` (230/230 PASS, app 7ccbd7e9, helpers main 1884986f, render 6e69661) | current tips (app fb16d07d, main 3377748c, render 9aaec57a); the bench cells run the swift-built app, not the bundle; no v2 cells (`--producers "compiler render controller"`) |
| v2 route keystroke -> paint with ONE traced keystroke attributed (compile 27-30 ms, probe 2, off-main prepare 12, preraster 2.6, one main hop) | `docs/evidence/mac-preview-v2-live-2026-09-12.md` + `trace-v2-30ms-revision-100.txt` (demo only, render 4888a67, swift-built app, load 8-10) | statistics over all painted revisions; the other seeds; current render 9aaec57a; packaged binaries |
| helper display-candidate (helper-v2) route on p3/p27/pmax seeds, validation p50/p95 | `docs/evidence/helper-display-route-2026-09-12T1356Z/summary.md` (app c5fa71cd, render 9aaec57a; p27 = demo body x14 = 28 pages, v2 sibling declined) | helper v1 stage split; packaged binaries; the render-pipeline lane's own 27-page fixture (`tests/incremental.rs document(40)`) |
| bench timeline log lines (`compile: sending`, `worker: line … decoded in`, `worker: event on main`, `compile: applied`, `preview-v2: preparing/prepared/published/blit`, `paint: revision`) | emitted under `TypingBench.isBenchActive` in `WorkerClient.swift`, `ShellModel.swift`, `ShellModel+Controller.swift`, `PreviewV2View.swift`, `TypingBench.swift` | nothing parses them into per-stage statistics; `PreviewControllerClient.swift` emits no decode/main-hop lines (helper route = one round-trip stage) |

Verdict: the gap (packaged binaries, hw1 + render27 seeds, per-stage attribution across all
painted revisions, per route) is NOT covered. Implemented only the uncovered part: a runner-side
analyzer (`lib/typing_attribution.py`) that executes the bundle's `FlashTeX` directly through
the shell's existing bench and attributes from the existing log lines. No product change needed.

## Checkpoint block

- Branch / SHA: `agent/mac-validation-4/mac-live-4` @ see `git log -1`.
- Worktree: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a807b85fa768eadf9`.
- Dirty files: `tools/native-validation/mac-live/run.sh` (packages the preview controller too:
  `--controller`), `tools/native-validation/mac-live/lib/typing_attribution.py` (new),
  `tools/native-validation/mac-live/quiet-window.sh` (new), this file,
  `coordination/agents/mac-validation-4.json`.
- Consumed main SHA: c11c005 (via mac-shell cd58fc2e); helpers built from origin/main 3377748c.
- Scratch: package-only runner in progress, log
  `$SCRATCHPAD/mv4/package-run.log`, work dir reused from the finished mac-validation lane
  (`.claude/worktrees/agent-a19c42dc3e488a6de/tools/native-validation/mac-live/build`, gitignored).
- Next commands: (1) `python3 tools/native-validation/mac-live/lib/typing_attribution.py --app <work>/app/<sha>/apps/mac/build/FlashTeX.app --repo . --out tools/native-validation/mac-live/reports/attribution-<UTC>`; (2) full `run.sh` for follow-up 1; (3) `quiet-window.sh`.
- Decisions: reuse the previous lane's SHA-keyed scratch builds (cargo target cache) to fit the
  bound; measure only on the packaged bundle (`Contents/MacOS/FlashTeX` executed directly,
  `FLASHTEX_NO_ACTIVATE=1`, never `open`); helper route attributed as one round-trip stage
  (client has no finer log lines — reported as a limitation, not patched).
- Resource: Claude Max 20x shared quota via parent; no purchases; no API spend.
