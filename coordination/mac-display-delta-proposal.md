# mac-display-delta-proposal — handoff

Lane: Claude Code subagent of mac-claude-a (mac-m1max-a). Document-only lane
(Commander ruling issue #2 comment 5646362851): a reviewable producer/consumer
schema + acceptance plan for a bounded, opt-in delta sibling on `display-list-v2`.
No wire activation, no producer code, no edits to Commander-owned contract files.

## Checkpoint (durable)

- Branch `agent/mac-render-pipeline/delta-proposal` from
  `origin/agent/mac-render-pipeline/unified` @ 9aaec57a (consumed main: d5440b0
  via 51289c9b; origin/main at fetch time 1975bb43, not merged — doc-only lane).
- Worktree `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-abb74bcd15b6666b0`.
- Owned paths: `crates/render-pipeline/docs/proposals/display-list-v2-delta.md`,
  `coordination/mac-display-delta-proposal.md`,
  `coordination/agents/mac-display-delta-proposal.json`.
- Dirty files: none after commit (see git log).
- Next commands: `git push -u origin agent/mac-render-pipeline/delta-proposal`.
- Reference script: `dl2_delta_example.py` (scratchpad, reproduced verbatim in
  the proposal's Appendix A) — the `dl2-canon-1` digest reference and the worked
  example generator; its printed digests are the ones embedded in the proposal.

## Coverage audit (mandatory first step, done 2026-09-12 ~10:10 local)

Searched `origin/agent/mac-claude-a/mac-shell` (`apps/mac/Tests/FlashTeXMacTests/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`),
`origin/agent/mac-helper-display/route` (`coordination/mac-helper-display.md`,
`ShellModel+DisplayCandidates.swift`, `RenderingV2.swift`, `GlyphRunRenderer.swift`),
`origin/main` (`crates/document-runtime/docs/*`, `docs/contracts/*`) and
`origin/agent/mac-render-pipeline/unified` @ 9aaec57a for `delta`, `page-scoped`,
`per-page`, `digest`, `display-list-v2-`:

- No existing test, evidence or contract text covers a delta/page-scoped
  `display-list-v2` reply. Matches were unrelated (`ProposalPreviewTests.pageDelta`
  = page-count difference of an assistant proposal; `NavigationTests` byte-delta
  rebasing of spans; fixture names `display-list-v2-*.json`).
- What IS covered and is reused by the proposal, not re-done:
  - producer full-reply gate: `crates/render-pipeline/tests/incremental.rs`
    (`incremental_output_is_byte_identical_over_200_edits_of_a_27_page_document`,
    ignored `…30_edits_of_a_107_page_document`), per-block caches in
    `src/incremental.rs`, decline path in `src/protocol.rs:229-260`;
  - consumer full-frame gate: `RenderingV2Tests.swift` (decode/validate/refusals),
    `PreviewV2Tests.swift` (stale/unsolicited sibling, V2Parity), helper-route
    admission gate `ShellModel+DisplayCandidates.swift` (`DisplayCandidateGate`);
  - runtime sibling transport: `crates/document-runtime/docs/display-candidates.md`,
    `producer-size-contract.md` (26-page fixture: v2 line 25 120 854 bytes,
    declined at 16 MiB; runtime default frame 8 MiB incl. newline);
  - the runtime requires sibling `type == "display_list"`
    (`crates/document-runtime/src/display_candidate.rs:48`, `raw_display.rs`),
    so the helper route needs an FT-049-owned change before any delta can pass.
- Conclusion: the gap (a reviewable schema + acceptance plan) is uncovered;
  proceed with the document only.

## Status

Proposal written and pushed (see git log for the tip). Final report to the
parent is in the lane's completion message; the 10-line summary is section 0
of the proposal.
