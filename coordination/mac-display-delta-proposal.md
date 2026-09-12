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

## Follow-up (Commander 5646477457, same lane/branch)

- `crates/render-pipeline/docs/proposals/contract-draft-review-ad922ea.md`: line-by-line
  review of main ad922ea1 `docs/contracts/runtime-v1-display-list-v2.md` against
  producer 9aaec57a and the Mac consumer at `origin/agent/mac-helper-display/route`
  3db719cf (incl. GH31 a9b55af7): 9 discrepancies (D1 direct-route paint admission
  does not bind document sha256 — medium; D2 helper gate ignores
  `membership_generation` — medium; D6 no client-side 1 MiB outgoing helper bound —
  medium; D3/D4/D5/D7/D8/D9 low) + 19 confirmed matches, all with file:line.
- `display-list-v2-delta.md` bumped to r2: installed-base acknowledgement via the
  additive request field `display_list_base` (§2, §3, §6.1), complete
  reconstruction table (§5.5), old+new residency with caps and eviction order
  (§6.2), honest full-resync refusal table (§8); acceptance plan P1/P3/C3 and the
  worked request example updated. Delta wire shape unchanged; Appendix A vectors
  re-run for r2: identical (reconstruction == fresh: True).

## Follow-up 2 (Commander 5646611117 → r3, same branch)

- `display-list-v2-delta.md` r3: (1) `MAX_SNAPSHOT_BYTES` charges the RECONSTRUCTED
  target via the `estimated_json_bytes()` constants, peak ≤ 2 × cap + one 16 MiB
  line (+ producer texts); (2) stale refusal preserved — stale siblings are never
  installed/acknowledged, no reconstruction cache; residency table is exactly
  two rows per side; (3) digests prove reconstruction fidelity only — completeness
  of diagnostics/omitted parts is the P2 fresh-full oracle's job, stated in §5.1,
  §5.5, §10; (4) canon fixes −0.0 → +0.0 and refuses non-finite; Python reference
  updated and checked (−0 hashes equal, NaN raises); vectors identical;
  (5) §5.4a names the real entry points (`RenderingV2.validate` →
  `V2FontStore.resolve` → `V2Frame.prepare`; rendering-core `PipelineCff::bind`
  / `helper_candidate::bind`) and states that rendering-core binds ORIGINAL bytes,
  so delta consumption is scoped to the Mac model-level path. §8 unchanged.

## Follow-up 3 (Commander 5646664026 → r4, same branch)

- `display-list-v2-delta.md` r4: over-cap reconstructed target is REJECTED before
  allocation/paint (`delta_target_oversize` naming estimated size vs cap; estimate
  computable from delta header + changed pages + cached per-page base estimates;
  last installed frame kept marked stale; chain cleared → full resync with §8's
  honest refusal); residency now `installed` (model + painted-frame resources) +
  `in-flight` (one line + one target; a running callback is never terminated by
  replacing the queued one) + `queued` (one line); producer `old` + `new` + reply
  line + request line; §6.1–6.3, §7, §10.2 C3, §10.3 measurements updated; §10.4
  maps the renderer's `refusal-scenarios.json` cases to gate tests. Vectors
  unchanged (re-run: reconstruction == fresh: True).

## Status

r1 committed as c797c5cf; r2 96e95628; r3 05c0e619; r4 committed after (see the log); r2 + review committed after (see the log) and pushed to
`origin/agent/mac-render-pipeline/delta-proposal`. Final report to the parent is
in the lane's completion message; the 10-line summary is section 0 of the
proposal. Lane complete; no further steps owned here. Limitations: no code, no
build/test run (nothing to build); digests/vectors are from the Python reference
only (Rust/Swift implementations are gate items P5/C1); the helper route needs
an FT-049-owned runtime change before any delta can pass.
