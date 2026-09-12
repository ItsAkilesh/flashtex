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

## Follow-up 4 (Commander 5646803033 → r5, same branch)

- `display-list-v2-delta.md` r5: chose (b) exact cached accounting — the delta
  carries `page_bytes[]` (exact per-page serialised length of the full line's
  page objects; accounting metadata OUTSIDE the digests, vectors unchanged); the
  consumer verifies each entry before allocation (changed page: measured length
  in the delta line; unchanged page: cached installed length + exact decimal-width
  change of relocated offsets, valid because the writer prints integrals as
  plain decimal, `json.rs:312-313`), mismatch = `delta_page_bytes_mismatch(n)`;
  target = framing + header parts measured on the delta line + Σ page_bytes +
  separators; `estimated_json_bytes()` is not used anywhere. In-flight PRERASTER
  bitmaps added to the peak (two raster sets). Appendix A re-run: digests
  identical, exact target 3 498 == fresh line length; digit-boundary arithmetic
  checked at +1/+3/−90/+903.

## Implementation lane (Commander 5647057936: ISOLATED implementation/testing only)

Not production, not native/helper activation, not completed acceptance.

### Producer — `agent/mac-render-pipeline/delta-proposal` @ f9eff0e7 (b956304a + probe)

Files: `crates/render-pipeline/src/delta.rs` (new), `src/protocol.rs`,
`src/incremental.rs` (snapshot slot), `src/display.rs` (`page_json` pub,
`to_json_with`), `src/lib.rs`, `tests/delta.rs` (new), `examples/delta_probe.rs`.
No text/font code touched. `cargo test --release --test delta` (load 30–60,
Latin Modern present): 5 passed, 1 ignored (107-page, `--ignored`):

| gate (review-r5.md) | test | result |
|---|---|---|
| actual writer / raw-range equivalence, escapes, UTF-8, digit relocations | `delta_wire_reconstruction_is_byte_identical_over_200_edits_of_a_14_page_document` (changed page objects on the wire byte-equal the fresh line's; `page_bytes` == fresh raw page ranges for ALL pages; header parts equal; `to_json(reconstruction)` == fresh line; exact target == fresh length; policy) | 200 edits: 112 deltas / 88 full replies; sibling bytes 1 302 728 345 vs fresh full 2 210 407 637; changed pages per delta 3.57 |
| 27-page shape | `delta_model_reconstruction_is_byte_identical_over_200_edits_of_a_27_page_document` (in-process: fresh `render` vs cached + `build` + `apply`, `to_json` bytes equal) | 199 deltas / 1 full; changed pages per delta 13.03 |
| relocation arithmetic vs writer across digit boundaries / signed moves / escapes+UTF-8 | `relocated_page_bytes_match_the_writer` | ok |
| checked size arithmetic, `page_bytes` count/range before indexing | `Delta::from_wire` (counts == page_count, ≤ MAX_SNAPSHOT_PAGES, ascending changed numbers), `apply` (bounds before any page is built), `full_line_bytes` checked adds | ok (unit) |
| zero-page separators max(N−1,0) | `delta::tests::zero_pages_have_zero_separators` | ok |
| cap+1 pre-allocation refusal; forged page_bytes changed/unchanged; wrong base; tampered digest | `delta_apply_refusals` | ok |
| negotiation (no ack / old ack → silent full; not requested clears; `-delta` without `display-list-v2`) | `delta_negotiation_refusals_answer_full` | ok |

Finding: the script's 27-page document's full `display_list` line is ~24 MB
(estimate 23 968 281) and is declined at 16 MiB today, so it never obtains a
base (proposal §8); the wire gate therefore runs at 14 pages (11.1 MB full
line). A full page of this fixture is ~0.8–1 MB of JSON.

### Consumer — `agent/mac-display-delta-impl/consumer` @ ac3d8969 (from mac-shell 2fa51729)

Files: `apps/mac/Sources/FlashTeXMac/DisplayListDelta.swift` (new),
`Tests/FlashTeXMacTests/DisplayListDeltaTests.swift` (new),
`FlashTeXProtocol/RenderingV2Fast.swift` (delta reader + page byte ranges),
`FlashTeXProtocol/RuntimeV1.swift` (`CompileRequest.displayListBase`),
`FlashTeXMac/WorkerClient.swift` (delta lines through `.displayList` only
under `FLASHTEX_DISPLAY_DELTA=1`). `swift build` ok;
`FLASHTEX_COMPILER=<flashtex-render from f9eff0e7> swift test --filter
DisplayListDeltaTests`: 5/5 passed (39 s, load ~40):

| gate | test | result |
|---|---|---|
| fresh-full semantic + pixel oracle; exact size vs real writer incl. escapes/UTF-8 | `testReconstructionEqualsFreshDecodeWithExactSizeAndParity` (5 edits, worker A delta chain vs worker B fresh full) | reconstruction == fresh decode; page_bytes == fresh raw ranges; exact target == fresh line length; V2Parity 0 px @1 px/pt; 5 deltas 286 417 B vs 10 510 908 B full (3 pages) |
| cap+1 before allocation; forged page_bytes ±1 (changed/unchanged); wrong base; tampered digest | `testCapPlusOneAndForgedPageBytesRefusedBeforeAllocation` | ok |
| producer wrong-base → full | `testAcknowledgingAnOlderSnapshotYieldsFull` | ok |
| one in-flight through delayed main-thread completion, queued replaced, clear never terminates running work | `testOneInFlightHeldThroughDelayedMainThreadCompletion` | ok |
| residency measured (installed + in-flight incl. preraster + queued) | `testResidencyMeasuredWithRealFrames` | see table |

Residency table (real frames, 3 pages, 2 px/pt prerasters, exact accounting):

| slot | model (exact serialised) | wire line | raster |
|---|---|---|---|
| installed | 2 091 932 | — | 23 265 792 (painted) |
| in-flight | 2 094 336 (target) | 49 480 (delta line) | 23 265 792 (candidate preraster) |
| queued | — | 2 094 336 (stand-in: the fresh line) | — |
| total accounted | 52 861 668 B; live slots 2 (+installed); RSS delta over the probe 376 832 B (process-wide, not a bound) | | |

Cross-check: `dl2-canon-1` list digest of the same frame computed by the Swift
consumer and the Rust producer on identical text: `690bf7fe…eac05` on both
(header canonical bytes byte-identical, 1 155 B).

### Recovery issue #50 (consumer test stall on a v1-only producer) — fixed at 65f06169

`Worker` reads on a background thread; `readLine(timeout:)` is a semaphore wait
with a real deadline (EOF-aware); `session()` checks the negotiation echo before
awaiting any sibling (no `display-list-v2` → skip after one reply; no `-delta`
→ skip; `FLASHTEX_COMPILER` named `flashtex-compiler` → immediate skip); all
other reads bounded (60 s → skip). Measured: parent `helpers.env` plain compiler
→ 4 skipped + 1 passed, 0.21 s test time; renamed plain compiler (echo path) →
skips in ~7 s wall; delta-capable producer → 5/5 in 41 s (load ~40). Branch
merged with mac-shell 9ba9851c.

### Not done / diff requests (parent-retained files)

- `ShellModel.swift` `compile()`: when `DisplayListDelta.enabled`, append
  `DisplayListDelta.capability` to the requested capabilities and set
  `request.displayListBase = <installed base>.acknowledgement` (only while an
  installed base exists and no request is in flight); clear the installed
  base on any result without the accepted `display-list-v2`, on worker
  restart/detach, and on pane hidden.
- `PreviewV2View.swift` (`receiveDisplayListV2`, not retained but left
  unwired in this lane): dispatch on `RenderingV2Fast.header(line).type ==
  DisplayListDelta.messageType` → `RenderingV2Fast.delta` →
  `DisplayListDelta.apply(to: installed)` inside the existing
  `startDisplayListV2` prepare closure, then `V2Frame.prepare`; install on
  publish; refusals → `V2Loader.Outcome.failed(refusal.asValidationError)`;
  drive `DisplayDeltaResidency` from arrive/dispatch/deliver.
- Helper route: untouched (FT-049 sibling type change pending).
- Whole `swift test` with all helpers not run (load 30–60 throughout).

## Status

r1 committed as c797c5cf; r2 96e95628; r3 05c0e619; r4 0569b742; r5 77477b70; implementation b956304a/f9eff0e7 (producer) + ac3d8969/65f06169 (consumer); r2 + review committed after (see the log) and pushed to
`origin/agent/mac-render-pipeline/delta-proposal`. Final report to the parent is
in the lane's completion message; the 10-line summary is section 0 of the
proposal. Lane complete; no further steps owned here. Limitations: no code, no
build/test run (nothing to build); digests/vectors are from the Python reference
only (Rust/Swift implementations are gate items P5/C1); the helper route needs
an FT-049-owned runtime change before any delta can pass.
