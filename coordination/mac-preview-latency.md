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

- Updated: 2026-09-12T18:34Z (resumed after the 17:25Z quota cut). Worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a96ab6bafe259bfb2`,
  branch `agent/mac-preview-latency/profile`, pushed through 68d4ded4.
- Commits: f7b86584 registration; 1c07cf93 instrumentation + before table;
  95a75546 page reuse by raw-byte identity + per-page bitmaps/views;
  247e1cdb layer-backed page blit + equatable/lazy diagnostics list + evidence
  rounds (raw/, raw-after2/, raw-hw1/); 68d4ded4 merge of
  origin/agent/mac-claude-a/mac-shell 9ba9851c (clean).
- Dirty (uncommitted): PreviewV2View.swift — `PreviewV2Pane.shownFrame`, the
  pages/diagnostics at ONE structural position for loaded and stale frames
  (each loaded<->stale toggle rebuilt the scroll view, page views and bitmap
  layers: 374 blits of an unchanged page over 187 revisions on p3, 346 on HW1);
  its test `testPaneShowsTheLoadedFrameAndThePreviousFrameStaleFromOnePosition`
  in PreviewLatencyTests.swift; this handoff + agents JSON.
- HW1 attribution (raw-hw1/after3-direct-v2-hw1-30ms.log, revision 60): the
  three main-thread hops recv->val 15.9, deliver 14.8, publish->pass 19.6 ms
  dominate (main busy with the v1 apply of 130 diagnostics + a full-window
  SwiftUI pass, 3 passes per keystroke); the v2 pane's own eager 130-row
  diagnostics VStack was the dominant paint cost before 247e1cdb
  (pub->paint 455 -> 38 ms). Page 3 (the edited page, offscreen in the
  LazyVStack) is never blitted in the bench; the recorded paint is the
  re-install of visible pages 1/2 caused by the structural toggle above.
- Running (background, pids launched by this lane): `<scratchpad>/latency/
  build-test.sh` (release build 4 of the merged tree at 68d4ded4, then the
  v2/preview/latency `swift test` filter; log build-test.log, test-filter.log);
  `<scratchpad>/latency/copytest.log` (control cell: after3 binary copied out
  of .build on HW1 — tests whether the after1/after2 HW1 no-paint cells were a
  copied-binary artifact). Load 20-80 (other agents building).
- Next: release build 5 with the dirty fix; bench direct-v2 hw1/p3 + helper-v2
  p3 at 30 ms (before-app 1c07cf93 vs build 5), summary.md, commit, full/filtered
  swift test with helpers (`source <scratchpad>/helpers.env`, FLASHTEX_RENDER
  = <scratchpad>/render-9aaec57a/.../flashtex-render, FLASHTEX_REVIEW_HISTORY_DIR=off),
  final report to the parent.
- Staffing/billing: shared Claude Max 20x quota with the parent; ~90-minute
  heavier-model window from 18:20Z; bounded to ~60 minutes of work.
