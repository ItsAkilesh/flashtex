# mac-capture-list-handoff — transfer-v1 `capture_list` owner-scoped contract handoff

Lane: Claude Code subagent `mac-capture-list-handoff` (parent `mac-claude-a`, mac-m1max-a).
Branch: `agent/mac-capture-list-handoff/capture-list` from `origin/agent/mac-claude-a/mac-shell` @ 6fb77efd
(contains main ffe199d). Document-only lane, bounded ~45 min from 2026-09-12T14:06Z. Claude Max quota only;
no helper builds, no provider calls, no code changes.

Commander ruling being executed (issue #2 comment 5646362851): "the capture-list gap needs an owner-scoped
contract handoff, not a duplicate bridge implementation." `crates/bridge` and `docs/contracts/transfer-v1.md`
are Commander-owned and untouched here.

## Coverage audit (mandatory first step, 14:06–14:18Z)

Sources read: `docs/contracts/transfer-v1.md`, `crates/bridge/README.md`, `crates/bridge/src/{main,store,lib}.rs`
(dispatch table, journal layout `<capture_id>.json`, `CaptureRecord`, `receive`/`capture_anchor`), bridge tests
`tests/{bridge,storage_recovery,cli}.rs` (names only), `apps/mac/Sources/FlashTeXProtocol/TransferV1.swift`,
`apps/mac/Sources/FlashTeXMac/{BridgeClient,BridgeSession,ShellModel+Bridge,ShellModel+Nearby}.swift`,
`apps/mac/README.md` (bridge/ledger integration), `apps/mac/tools/nearby-client/Sources/NearbyClient/NearbyWire.swift`,
`docs/evidence/capture-acceptance-2026-09-12T1340Z.md`, `tools/native-validation/mac-live/reports/20260912T110944Z.md`
(rows 287–309), `coordination/mac-capture-acceptance.md`.

| Question | Already covered (file:test / evidence) | Verdict |
|---|---|---|
| Is there any capture-listing request in transfer-v1 / the bridge? | `crates/bridge/src/main.rs` dispatch has exactly `capture_validate, document_open, document_edit, destination_pin, capture_submit, capture_convert, capture_prepare_insert, capture_applied, capture_status, capture_reject`; `TransferV1.Request` (Swift) mirrors the same ten. `grep -rn capture_list` over apps/mac, crates/bridge, docs: no hits. | **Gap confirmed**: no listing op exists anywhere. |
| Is the relaunch gap reproduced by a test? | `CaptureAcceptanceTests.testAppRelaunchBetweenReceiptAndInsertKeepsOneJournaledCaptureWithoutDuplicates` (line 229 asserts `model2.bridgeCaptures.isEmpty` with the message "finding: no capture listing in transfer-v1"); evidence `docs/evidence/capture-acceptance-2026-09-12T1340Z.md` finding 1 (4/4 with real helpers). | Covered as a **documented reproduction**; nothing to add on the test side until the op exists. |
| Does an in-process bridge relaunch lose the rows? | `RealBridgeTests.testKilledBridgeIsRelaunchedWithJournalAndDestinationIntact`, `BridgeRecoveryTests.testBridgeCrashIsRelaunchedAndStateReconciled`, `testCrashDuringCaptureSubmitLeavesTheCaptureUncertainAndRetryable`; mac-live rows 287–290, 303–305. `BridgeSession.captures` is in-memory on the Swift side and survives a child relaunch. | Covered; **not** the gap (the gap is a new `ShellModel`/app process). |
| Are prepared/applied edits recovered after a relaunch? | `BridgeSession.reconcile` (`recovery_export` → `capture_status` per pending ledger receipt → `recovery_import`); `BridgeRecoveryTests.testMissingReceiptIsReplayedFromTheRetainedSnapshot`, `testTransientStatusFailuresRetainTransactionsAndEvidence`, `testReceiptInFlightAtBridgeCrashIsSettledFromTheLedgerExactlyOnce`, `testLedgerCrashWithPendingReceiptKeepsTheEvidence`; `RealEditLedgerTests`. | Covered for **staged/inserted** captures (they are in the edit ledger). Uncovered: **journaled-but-never-prepared** captures — the ledger has no entry for them, so nothing on the Mac side can discover them. This is exactly the scope of `capture_list`. |
| Nearby error classification of `destination_reselection_required` | `NearbyWire.captureInputErrorCodes` (NearbyWire.swift:156) lists `image_too_large, invalid_image, unsupported_image, revision_mismatch, capture_id_conflict, bad_request` only; finding 2 in the acceptance evidence. | Being fixed by lane `mac-nearby-errors` (`origin/agent/mac-nearby-errors/nearby-errors` exists). Cross-referenced only. |

Verdict: the gap is real and only the contract/handoff part is in scope for this lane. Implemented below as a document.

## Delivered

- `apps/mac/docs/handoffs/transfer-v1-capture-list.md` — the owner-scoped contract handoff (gap + reproduction,
  consumer requirement, proposed `capture_list` request/reply, refusals, idempotence with `capture_status` and the
  ledger's `recovery_import`, Mac state-machine sketch, bounded acceptance plans for the bridge owner and the Mac side,
  nearby cross-reference).
- This handoff + `coordination/agents/mac-capture-list-handoff.json`.

No code changes. Parent-retained files untouched; no diffs to apply. Test counts: none run (document-only lane; nothing
compiled changed). `swift build` not run for the same reason.

## Durable checkpoint
- Task: capture-list contract handoff (brief `prompt-mac-capture-list-handoff.md`); branch
  `agent/mac-capture-list-handoff/capture-list`; worktree `.claude/worktrees/agent-a85652b485b294f2d`.
- Consumed: mac-shell 6fb77efd; main ffe199d (origin/main tip at start d9239512, not consumed).
- Dirty files: none. Tip: 4da26cec (handoff doc + coordination) followed by this revision-pinning commit.
- Next commands (parent): review `apps/mac/docs/handoffs/transfer-v1-capture-list.md`; merge the branch; relay the
  handoff path to the Commander on issue #2 (bridge owner) for the contract decision. No swift test needed for this lane.
- Decisions: `rejected` kept as its own listed status (durable terminal state in the journal) in addition to the brief's
  journaled/staged/inserted/failed; ordering key is an additive `receipt_sequence` because the journal has no
  timestamps/sequence today; listing is read-only and never mutates the journal; the Mac never auto-inserts or
  auto-converts listed captures.
- State: ready_for_integration; lane stopping (bounded task complete).
