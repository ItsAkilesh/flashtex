# mac-preview-latency handoff — v2 preview typing-to-paint latency

- Agent / parent / machine: `mac-preview-latency` (Claude Code subagent, Fable) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: Commander issue #2 comment 5646989044 items 2+3 — profile and reduce
  the measured typing-to-paint latency of the v2 preview route; remove redundant
  font/text preparation with bounded immutable resource identity (raw-byte
  sha256 + size), keeping exact page content and every stale/identity check.
- Branch: `agent/mac-preview-latency/profile` from
  `origin/agent/mac-claude-a/mac-shell` `cd58fc2e`.
- Owned paths: `apps/mac/Sources/FlashTeXProtocol/RenderingV2Fast.swift`,
  `apps/mac/Sources/FlashTeXMac/{PreviewV2View,GlyphRunRenderer,WorkerClient,
  ShellModel+DisplayCandidates,TypingBench}.swift`, new
  `apps/mac/Sources/FlashTeXMac/V2PageCache.swift`, tests
  `apps/mac/Tests/FlashTeXMacTests/PreviewLatencyTests.swift`,
  `tools/typing-bench/timeline.py`, `docs/evidence/preview-latency-<UTC>/`,
  this handoff and `coordination/agents/mac-preview-latency.json`.
  Parent-retained files may be edited only for a measured win (minimal,
  test-covered, one commit each): none edited so far.
- Rules: `FLASHTEX_NO_ACTIVATE=1` for every app launch; no UI scripting; only
  pids this lane launched are signalled; Rust crates are never edited (producer
  / helper findings are reported with reproductions); no purchases.

## Durable checkpoint

- Updated: 2026-09-12T16:10Z. Worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a96ab6bafe259bfb2`.
- HEAD: (see git) on `agent/mac-preview-latency/profile`; dirty: none after the
  registration commit.
- Helpers: compiler / preview-controller / pdf built in this tree
  (`crates/*/target/release`, 42 s); producer `flashtex-render` from
  `origin/agent/mac-render-pipeline/unified` 9aaec57a at
  `<scratchpad>/render-9aaec57a/crates/render-pipeline/target/release/flashtex-render`
  (sha256 ed729b02…); prior helper c95a26a3… in worktree agent-ae7ddcfb739815fac.
- Seeds/siblings for in-process measurement: `<scratchpad>/latency/seeds/`
  (p3: 4 pages 3,887,809 B; pmax: 8 pages 7,793,557 B; p3-typed: pages 1–3
  byte-identical to p3, page 4 differs).
- Pre-optimization attribution from the parent's 2026-09-12T142134Z logs
  (helper-v2-p3-30ms, 43 painted revisions, p50 ms): key->send 14,
  send->v1 82, v1->sibling received 64, admission 0.2, validate 28,
  preraster 4, deliver 1.5, publish->paint 27; key->paint (last covered
  keystroke) 238. pmax: 13 / 108 / 99 / 0.2 / 46 / 8 / 4 / 28; 307.
- Next: (1) instrumentation commit (worker display_list receipt stamp,
  timeline.py, in-process stage bench test); (2) evidence dir with the table;
  (3) per-page reuse keyed by sha256(page bytes)+size+fonts-manifest digest,
  per-page bitmap identity, caret lookup bounded by page source ranges;
  (4) re-measure helper + direct routes in a quiet window (`uptime` recorded).
- Staffing/billing: shared Claude Max 20x quota with the parent; stop when the
  parent says the 5-hour meter reached 85%.
