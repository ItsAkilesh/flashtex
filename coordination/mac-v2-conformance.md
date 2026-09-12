# mac-v2-conformance handoff — Mac consumer conformance to the display-list-v2 draft (review D1/D2/D3/D6/D9)

- Updated UTC: see `coordination/agents/mac-v2-conformance.json` `updated_utc`
- Agent / parent / machine alias: `mac-v2-conformance` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: consumer-side discrepancies D1, D2, D3, D9, D6 from
  `docs/proposals/contract-draft-review-ad922ea.md` (draft contract
  `docs/contracts/runtime-v1-display-list-v2.md` at main ad922ea, read only).
  Owned paths: `apps/mac/Sources/FlashTeXMac/PreviewV2View.swift`,
  `GlyphRunRenderer.swift`, `ShellModel+DisplayCandidates.swift`,
  `PreviewControllerClient.swift` (outgoing bound only), `LineProcessClient.swift`,
  `apps/mac/Sources/FlashTeXProtocol/RenderingV2.swift`, `TransferV1.swift`,
  `apps/mac/Tests/FlashTeXMacTests/{PreviewV2,DisplayCandidate,RenderingV2,V2FontStoreIdentity,V2Conformance}Tests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/Fixtures/display-list-v2-*.json`,
  `Fixtures/fake_worker_v2.py`, this handoff, `coordination/agents/mac-v2-conformance.json`.
  Parent-retained (diff requests only): `ShellModel.swift`, `ShellModel+Controller.swift`,
  `ContentView.swift`, `PreviewView.swift`, `FlashTeXMacApp.swift`, `SourceEditorView.swift`.
- Branch: `agent/mac-v2-conformance/contract` from `origin/agent/mac-claude-a/mac-shell` 5997f372.

## Coverage audit (mandatory first step; sources at 5997f372)

Grepped `apps/mac/Tests/FlashTeXMacTests/*`, `tools/native-validation/mac-live/reports/20260912T110944Z.md`,
`docs/evidence/*` for each item. "Covered" = a test that fails if the draft rule is violated.

| Item | Already covered (file:test) | Verdict |
|---|---|---|
| D1 direct route binds `documents[].sha256`/`byte_length` before paint | `PreviewV2Tests.swift:PreviewV2LiveTests.testLiveFrameArrivesWithTheCompileResultAndNavigates` asserts the producer's digest EQUALS the request text (observes, never refuses); `testStaleUnsolicitedAndMismatchedLinesNeverApply` covers `correlation_mismatch` (project/revision) only; `PreviewV2ShellTests.testStaleBufferIsRefusedAndSyntheticContentHasNoSource` covers the NAVIGATION-time digest refusal (`navigateV2`). Live report / evidence: nothing on pre-paint document binding for the direct route. | NOT covered: refusal before paint on sha256 or byte_length mismatch; retained previous frame. |
| D2 helper gate compares `membership_generation` | `DisplayCandidateTests.swift:testDecoderAcceptsOnlyContractShapedCandidates` decodes it (`:43`); `testGateRefusesStaleForeignAndToggledIdentities` covers session/project/request/compile revision/source versions/active path/display floor — no generation case; `testHelperCandidatesPaintAndBindToEditorRevisions` builds frames with `membershipGeneration: 1` but never varies it. `ProjectDocumentsTests.swift:295-337` cover `ProjectDocuments.membershipGeneration` itself. | NOT covered: refusal on generation mismatch (gate + pre-paint recheck). |
| D3 drop the `bytes ‖ face0` alias; comment in `RenderingV2.swift:22-26` | `PreviewV2Tests.swift:testPipelineHashConventionResolvesToTheSameBytes` asserts the alias IS accepted (the opposite); `RenderingV2Tests.swift:testRealTextFixturePreparesAgainstBundledFontsByPipelineHash` asserts `hashConvention == "bytes+face0"` on the text fixture; `V2FontStoreIdentityTests.swift:49,93-94` resolve through the face0 key (read-once/verify semantics); `DisplayCandidateTests.swift:382` asserts the REAL helper/producer frames resolve as `"bytes"` (confirms the producer emits raw digests). All four `Fixtures/display-list-v2-*.json` carry face0 digests (checked by script: every font hash equals SHA-256(bundled bytes ‖ 00000000)); the pinned producer 9aaec57a run on `display-list-v2-text.tex` emits raw-byte digests (font_id == sha256 == SHA-256(bytes)). | NOT covered: refusal of the face0 spelling. Fixtures must be rebound to raw digests (font identity only; geometry unchanged — regenerating with 9aaec57a changes item counts 15→16 and would break index-based tests). |
| D9 direct route requests `display-list-v2` while a fixture is shown | `PreviewV2Tests.swift:testPaneVisibilityRequestsTheCapability` covers `setLiveV2` add/remove only. `receiveDisplayListV2` drops the sibling as stale when `previewSource == .fixture` (`PreviewV2View.swift:216`), and `compile()` sends `requestedLayoutCapabilities` regardless of the fixture (`ShellModel.swift:605`). | NOT covered. |
| D6 outgoing helper line bound vs the helper's 1 MiB stdin | `BridgeClientTests.swift:168` covers the `LineProcessClient` 12 MiB local refusal (bridge route; `TransferV1.maxLineBytes` is the BRIDGE contract's bound, not the helper's); `OutputBoundsTests` cover oversized helper OUTPUT only. `PreviewControllerClient.send` (`:146-153`) has NO outgoing bound at all; the helper's stdin reader (`crates/preview-controller/src/main.rs:30,189-201`) is `MAX_FRAME = 1 MiB` and on overflow emits `failure(... "truncated or oversized input")` and BREAKS its reader loop. | NOT covered; needs measurement with the real helper (below). |

## Checkpoint block

- Branch `agent/mac-v2-conformance/contract`; consumed parent SHA 5997f372 (mac-shell), main consumed through the parent's merge (see `git log`).
- Worktree `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ad8b60e180d0f9722`; scratch build of the pinned producer at `.scratch/render-9aaec57a` (untracked, never committed).
- Real helpers built in this worktree: `crates/preview-controller/target/release/flashtex-preview-controller`, `crates/compiler/target/release/flashtex-compiler` (from 5997f372 sources).
- Dirty files / next commands / decisions: see the per-item sections below (updated per commit).
- Staffing/billing: Claude Max quota shared with parent; no purchases.
