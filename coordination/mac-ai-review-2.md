# mac-ai-review-2 handoff — AI review (item 15) replenishment: cancellation / out-of-order rejection / reviewed-insertion recovery

- Agent / parent / machine: `mac-ai-review-2` (Claude Code subagent) / parent `mac-claude-a` / `mac-m1max-a`
- Task: Commander replenishment (issue #2 comment 5646989044). Current task: proposal CANCELLATION at every stage, OUT-OF-ORDER REJECTION, REVIEWED-INSERTION RECOVERY with the real assistant-context helper + real preview-controller. Follow-up 1: review history. Follow-up 2: explanation cache keyed by (diagnostic, source sha).
- Owned: `ProposalPreview.swift`, `AssistantRequestState.swift`, `ProposalPreviewTests.swift`, `AssistantRecoveryTests.swift`, the deterministic provider fixture (`Fixtures/controlled_assistant_provider.py`), new files per feature. Parent-retained files (ShellModel*, ContentView, PreviewView, FlashTeXMacApp, SourceEditorView) are never committed here.
- Branch: `agent/mac-ai-review-2/recovery` from `origin/agent/mac-claude-a/mac-shell` cd58fc2e (main c11c005 merged). Worktree `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a358626085f62f63e`.
- Helpers used by tests (already built in the main checkout, pointed at by env vars, never rebuilt here): compiler/preview-controller (Sep 12 11:23), assistant-context (07:17), pinned scratch helper `apps/mac/build/assistant-context/...` (05:07).

## Coverage audit (mandatory first step, 12:04–12:16Z)

Sources read: `ProposalPreview.swift` (1511 lines), `AssistantRequestState.swift`, `ShellModel.swift` (approveProposal / editRefused / editApplied / captureRefund), `ShellModel+Bridge.swift` (approveBridgeProposal), `SourceEditorView.swift` (applyPendingEdit refusal reasons), `ProposalPreviewTests.swift` (26 tests), `AssistantRecoveryTests.swift` (6), `EditorDiagnosticsExplanationsTests.swift` (6), `InsertionTests.swift`, `SourceEditorViewTests.swift`, bridge/ledger recovery suites (names), `coordination/mac-ai-review.md`, `coordination/mac-assistant-recovery.md`, `tools/native-validation/mac-live/reports/20260912T110944Z.md` (assistant `bundled:false`; no review recovery scenario), `docs/evidence/mac-ai-review-assistant-{reviewed,approved}-2026-09-12.png`.

### Already covered (file:test)

Cancellation
- Reviewer Cancel button mid-provider, real helper, late provider reply discarded, sheet idle, no edit: `AssistantRecoveryTests.testReviewerCancelDiscardsLateProviderResultAndLeavesSheetIdle`.
- Cancellation by proposal / revision / anchor change mid-helper (probe) and mid-provider, fake doubles: `ProposalPreviewTests.testExplanationIsCancelledWhenProposalOrRevisionChangesAndLateRepliesAreDropped`; same with the real helper (expired note, helper's own stale-snapshot refusal replayed): `AssistantRecoveryTests.testDocumentChangeExpiresTheRequestAndTheHelperRefusesTheMovedSnapshot`.
- Cancellation mid-provider via `FLASHTEX_ASSISTANT_PROVIDER` env path: `ProposalPreviewTests.testProviderPathEndToEndFromEnvironmentWithLocalCommand`.
- Provider timeout + retry with a new id + stubborn late valid reply ignored: `AssistantRecoveryTests.testProviderTimeoutFromEnvironmentThenRetryStartsNewRequestAndIgnoresLateResult`.
- Helper killed mid-request (probe/prepare), relaunched next request; helper refusal at review; provider exit: `AssistantRecoveryTests.testHelperExitMidRequestIsSurfacedAndRelaunchedOnNextRequest`.
- Fake helper crash/hang/garbage/huge at probe, provider crash/hang/garbage, helper crash during approve: `ProposalPreviewTests.testHelperCrashTimeoutOrGarbageLeavesReviewUsableAndBufferUntouched`.

Out-of-order rejection
- Forged validate replies for OLDER request ids while `.prepared`; wrong-stage (validate/approve) replies while `.preparing`; helper answering another compile revision: `ProposalPreviewTests.testOutOfOrderOrMismatchedRepliesNeverChangeTheDisplayedState`.
- Late provider reply for the timed-out request after the retry started and after it finished: `AssistantRecoveryTests.testProviderTimeoutFromEnvironmentThenRetryStartsNewRequestAndIgnoresLateResult`.
- Provider answering another context refused by the real helper at review: `AssistantRecoveryTests.testHelperExitMidRequestIsSurfacedAndRelaunchedOnNextRequest` (`stale_context`).

Reviewed insertion / approval boundary
- Approval boundary across cancel/timeout/kill/expiry, explicit approve amends the proposal text only, `approveProposal` produces one pending edit: `AssistantRecoveryTests.testNoEditReachesTheDocumentWithoutExplicitApproveAcrossRecoveries`.
- Editor refuses a pending edit outside the buffer (no capture refund): `SourceEditorViewTests.testPendingEditOutsideTheBufferIsRefusedWithoutChangingText`; one undo step delivered once: `testCaptureInsertionIsOneUndoStepDeliveredToTheModelOnce`.
- Bridge path: buffer changed between prepare and apply → reselection: `ShellModelBridgeTests.testBufferChangedBetweenPrepareAndApplyIsRefusedWithReselection`; ledger/bridge crash mid-apply exactly once, durable edit survives relaunch, receipt settled once: `BridgeRecoveryTests.testLedgerCrashDuringApplyInsertsExactlyOnce`, `testDurableEditAwaitingAdoptionSurvivesABridgeRelaunch`, `testReceiptInFlightAtBridgeCrashIsSettledFromTheLedgerExactlyOnce`; real preview-controller explicit insert = one durable edit: `CaptureAcceptanceTests.testExplicitInsertRidesTheRealPreviewControllerAsOneDurableEdit`; changed anchor refused by the real bridge: `CaptureAcceptanceTests.testCaptureAgainstAChangedAnchorIsRefusedByTheRealBridgeAndNeverInserted`.

Follow-ups
- Review history: NOTHING exists (no list of past proposals with outcomes; `ShellModel.appliedCaptureIDs` is a set of ids only).
- Explanation cache: `EditorDiagnostics.ExplanationCache` is keyed by compile RESULT id (per result, 8 results), not by (diagnostic, source sha): re-explaining an unchanged diagnostic after a recompile is a new helper request. The review sheet's `explain()` has no cache at all (each Explain = new helper run).

### NOT covered before this lane (implemented here)

1. Reviewer cancel at EVERY stage with the real helper: `.probe`, `.prepare` (synchronously after launch), `.validate`/`.review` (helper stage after the provider), `.approve` (during `approveReviewedEdit`), plus sheet close (`close()` → "review closed") and proposal REJECT mid-flight; each with the late reply discarded, `recoveryLog` naming the stage, no `pendingEdit`.
2. Out-of-order: a reply for stage N-1 after stage N with the SAME request id (probe/prepare replies while at provider; provider reply while at review; review reply after `.done`); a reply carrying ANOTHER proposal's context id (second ProposalPreview for a second capture) refused as "answers another context"; replies after the reviewed edit was approved (`.approved`) and after the proposal was approved for insertion and the sheet closed.
3. Reviewed-insertion recovery on the LOCAL (AI-reviewed) path with the REAL hosted editor: approve amended proposal → editor refuses because the buffer moved (revision mismatch) → `editRefused` puts the proposal (amended latex) back at the front of the queue with the exact reason, anchor restored, `appliedCaptureIDs` cleared → re-explain against the new revision (real helper) → re-approve inserts exactly once (no duplicate on re-enqueue). Helper killed mid-`approve` (real helper, by pid) → exact reason, proposal still reviewable, no loss; real preview-controller attached so the eventual insertion is one durable edit.
