# mac-capture-acceptance — Gap 7 companion capture reconnect → review → explicit insert

Lane: Claude Code subagent `mac-capture-acceptance` (parent `mac-claude-a`, mac-m1max-a).
Branch: `agent/mac-capture-acceptance/capture-acceptance` from `origin/agent/mac-claude-a/mac-shell` @ 5bc3fc0f.
Bounded ~75 min from 2026-09-12T13:29Z. Claude Max quota only; no provider/assistant calls anywhere.

## Coverage audit (mandatory first step, 13:30–13:45Z)

Sources read: `apps/mac/Sources/FlashTeXMac/{ShellModel+Nearby,ShellModel+Bridge,BridgeSession,Insertion,NearbyProtocol}.swift`,
`crates/bridge/src/lib.rs` (`capture_anchor`/`receive`), the tests below,
`tools/native-validation/mac-live/reports/20260912T110944Z.md` (capture-cycle rows 293–309),
`docs/evidence/nearby-client-recovery-2026-09-12T092814Z.md`, `docs/evidence/companion-capture-build-verification.md`.

| Sub-gap | Already covered (file:test) | Verdict |
|---|---|---|
| (a) companion disconnect mid-capture → reconnect same pairing → re-delivered once | `NearbyReferenceClientTests.testReconnectorResendsSameCaptureAfterListenerDropAndSamePortRestart` (real listener + real reconnector, **in-memory inbox**, no bridge: stored once); `NearbyReferenceClientTests.testSameSessionRetryIsAcknowledgedWithoutRedeliveryAndRevisionMismatchIsTerminal`; `NearbyListenerTests.testDuplicateIsAcknowledgedNotRedeliveredAndMismatchesAreRefused`, `testRecordedSessionIsAcceptedOnLoopbackWithDuplicatesAbsorbed` (same-session dedup); `RealBridgeTests.testDurableReceiptDuplicateConflictProviderDisabledAndReject` (identical resubmission through `ShellModel.submitCapture`, **not** via the nearby listener); `NearbyBridgeForwardingTests.testNearbyCaptureIsForwardedToAttachedBridge` (**fake** bridge, no disconnect); evidence `nearby-client-recovery-…092814Z.md` (identical retry de-duplicated, inbox route). | **Uncovered**: drop → reconnect → resend through the nearby listener with the **real bridge journal** as the ledger (one journal file, one `bridgeCaptures` row, same durable receipt). |
| (b) review shows the capture; no edit until explicit Insert | `ShellModelBridgeTests.testFullReviewedInsertionFlowAndLedgerIdempotence` (`activeText == before`, `pendingEdit` nil before approve; fake bridge); `ShellModelCaptureTests.testReviewFlowInsertsOnceAndRequiresAnchor` (local path); `ProposalPreviewTests` (sheet); `RealBridgeTests` (real bridge: `proposals.isEmpty`, `status.applied == nil`); mac-live rows 296/301 (`applied: False`, no proposal). | Covered. A real-bridge proposal needs `--enable-grok` (provider call) which is forbidden, so nothing more can be added truthfully; the new tests still assert `pendingEdit == nil`, unchanged text and `status.applied == nil` after nearby receipt. |
| (c) explicit insert → one undoable edit at the pinned anchor; durable via helper route / bridge route | Bridge route: `RealEditLedgerTests.testShellFlowWithRealHelperAndFakeBridge` (**real edit-ledger**, fake bridge for the proposal: byte 13, on-disk durable, dup refused), `ShellModelBridgeTests.testFullReviewedInsertionFlowAndLedgerIdempotence`, `SourceEditorViewTests.testCaptureInsertionIsOneUndoStepDeliveredToTheModelOnce`, `testCaptureUndoRedoReachesTheDurableDocumentInOrder` (one undo step, durable order); real bridge up to `capture_prepare_insert → proposal_missing` (`RealBridgeTests`, mac-live rows 302/308); mac-live review stage applies a fixture proposal through the real ledger directly. Helper route (`FLASHTEX_PREVIEW_CONTROLLER`): `PreviewControllerTests.testEditsBecomeDurableAndPreviewsBindToEditorRevisions` covers **typing** edits only. | **Uncovered**: an explicit reviewed insert (fixture proposal, pinned anchor) while the **real preview controller** is attached → exactly one durable edit whose durable text contains the insertion once at the anchor, and ⌘Z-equivalent revert reaching the controller. Real bridge + real ledger reviewed insert is impossible without a provider (no proposal) — reported, not manufactured. |
| (d) capture arriving after the anchor text changed → rebase-or-reselect, never silent insert | `InsertionTests.testResolveExactRebasedAndReselection` (pure), `ShellModelCaptureTests` (reselection), `ShellModelBridgeTests.testBufferChangedBetweenPrepareAndApplyIsRefusedWithReselection` (fake bridge+ledger, `revision_conflict`), `BridgeRecoveryTests.testDestinationLostWhenSourceMovedPastThePinIsReported` (fake). | **Uncovered** with the real bridge: a nearby capture bound to the destination advertised in `hello_ack`, submitted after an edit that deleted the anchor text → real bridge answers `destination_reselection_required` to the companion, nothing journaled/inserted; an edit elsewhere keeps the anchor valid (accepted, still no insert). |
| (e) app restart between receipt and insert → survives or reported lost, never duplicated | `RealBridgeTests.testKilledBridgeIsRelaunchedWithJournalAndDestinationIntact` (bridge **process** crash, same ShellModel), `BridgeRecoveryTests.*` (fake), mac-live rows 303–305 (SIGKILL + restart, journal answers). No-bridge inbox: `NearbyListenerTests.testInboxAcknowledgesNonDurablyAndRefusesConflicts` (ack says `durable: false`). | **Uncovered**: a **new ShellModel** (app relaunch) attached to the same store: `capture_status` still answers, a companion resend of the same `capture_id` yields the identical receipt, one journal file, no duplicate row. Finding: transfer-v1 has no capture-list op, so the relaunched shell shows no received captures until the companion resends (documented in the test). |

## Implemented (13:45Z)

`apps/mac/Tests/FlashTeXMacTests/CaptureAcceptanceTests.swift` — 4 tests, all real helpers, XCTSkip without env:
- `testDropAfterDeliveryThenReconnectIsJournaledOnceByTheRealBridge` (a+b)
- `testAppRelaunchBetweenReceiptAndInsertKeepsOneJournaledCaptureWithoutDuplicates` (e)
- `testCaptureAgainstAChangedAnchorIsRefusedByTheRealBridgeAndNeverInserted` (d)
- `testExplicitInsertRidesTheRealPreviewControllerAsOneDurableEdit` (c, helper route)

Evidence: `docs/evidence/capture-acceptance-2026-09-12T1340Z.md` (run 2: 4/4 in 1.08 s, uptime load 10.94/29.27/25.05
under the parent's heavy-build window — not an isolated timing; env-less run: 4 skipped). `swift build --build-tests` clean.
Full `swift test` deliberately NOT run (parent's heavy-build notice; parent runs it at integration under load < 15).
Findings (not patched, product files are parent/Commander-owned): no capture-listing op in transfer-v1 (relaunched shell
shows no pending captures until the companion resends); `bridgeDestination`/`hello_ack.destination` not refreshed after
edits overlapping the pin (bridge refuses `destination_reselection_required`, which `NearbyError.needsNewCapture` does not
classify); `BridgeSession.prepare` marks `.proposed` on `proposal_missing`; a reviewed insert through the real bridge + real
ledger is impossible without a provider (stays covered by RealEditLedgerTests with the fake bridge proposal).
Parent-retained files: untouched; no diffs to apply.

## Durable checkpoint
- Task: Gap 7 (brief `prompt-mac-capture-acceptance.md`); branch `agent/mac-capture-acceptance/capture-acceptance` tip a9acc2a9 (+ this coordination commit); worktree `.claude/worktrees/agent-a93d1274f0bb3f813`.
- Consumed: mac-shell 5bc3fc0f (main dda0b62 preview-controller compact edit acks).
- Helpers built in-worktree: `crates/{bridge,edit-ledger,preview-controller,compiler}/target/release/*` (cargo release, zero deps).
- Dirty files: none after the coordination commit.
- Next commands (parent): merge branch; `cd apps/mac && FLASHTEX_BRIDGE=… FLASHTEX_EDIT_LEDGER=… FLASHTEX_PREVIEW_CONTROLLER=… FLASHTEX_COMPILER=… swift test` under load < 15.
- State: ready_for_integration; lane stopping (bounded task complete).
