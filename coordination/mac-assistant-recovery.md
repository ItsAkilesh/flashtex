# mac-assistant-recovery handoff — assistant cancellation/expiry late-result recovery (Gap 8)

- Updated UTC: see `coordination/agents/mac-assistant-recovery.json` `updated_utc`
- Agent / parent / machine alias: `mac-assistant-recovery` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: parent brief "Gap 8 — assistant cancellation/expiry late-result UI
  recovery using the bundled real `flashtex-assistant-context` helper and a
  local deterministic provider fixture (no network, no billing), preserving the
  proposal approval boundary". Owned paths:
  `apps/mac/Sources/FlashTeXMac/AssistantRequestState.swift` (new),
  `apps/mac/Tests/FlashTeXMacTests/AssistantRecoveryTests.swift` (new),
  `apps/mac/Tests/FlashTeXMacTests/Fixtures/controlled_assistant_provider.py` (new),
  minimal edits to `apps/mac/Sources/FlashTeXMac/ProposalPreview.swift` (not
  parent-retained), this handoff and `coordination/agents/mac-assistant-recovery.json`.
  Never touched: `ShellModel*.swift`, `ContentView.swift`, `PreviewView.swift`,
  `FlashTeXMacApp.swift`, `SourceEditorView.swift`, every crate.
- Branch: `agent/mac-assistant-recovery/assistant-recovery` from
  `origin/agent/mac-claude-a/mac-shell` `5bc3fc0f` (contains main `dda0b62`).
  origin/main observed at start: `ffe199d8` (not merged here; product files
  under apps/mac only).

## Coverage audit (mandatory first step, done 13:25–13:35Z)

Sources read: `apps/mac/Sources/FlashTeXMac/ProposalPreview.swift` (1475 lines:
`OneShotProcess`, `explain()`, `cancelExplanation(reason:)`, `handleExplanationReply`,
`fail`, `staleExplanationReplies`, `childLaunches`), `apps/mac/Tests/FlashTeXMacTests/
ProposalPreviewTests.swift` (25 tests), `EditorDiagnosticsExplanationsTests.swift`
(6 tests, all about `EditorDiagnostics` explain-diagnostic cache, not the review
sheet), `Fixtures/fake_assistant_provider.py`, `Fixtures/fake_assistant_context.py`,
`crates/assistant-context/src/{lib,main,review}.rs` (read only),
`tools/native-validation/mac-live/reports/20260912T110944Z.md` (assistant helper
`bundled:false` in that app build; no assistant recovery scenario in the report),
`docs/evidence/assistant-runtime-integration-2026-09-12.md` (Linux runtime
integration; no cancellation/expiry/timeout case), `docs/evidence/mac-ai-review-
assistant-{reviewed,approved}-2026-09-12.png` (static sheet renders).

Already covered (file:test):

- Cancellation by proposal/anchor/revision change through `update()` with fake
  helper+provider, late replies counted as stale, state `.cancelled("proposal or
  document changed")`: `ProposalPreviewTests.testExplanationIsCancelledWhenProposalOrRevisionChangesAndLateRepliesAreDropped`.
- Same cancellation mid-provider with the REAL helper and `FLASHTEX_ASSISTANT_PROVIDER`
  local command from the environment (`FLASHTEX_ASSISTANT_TIMEOUT_S=5`), plus an
  oversized provider reply refused: `ProposalPreviewTests.testProviderPathEndToEndFromEnvironmentWithLocalCommand`.
- Forged/out-of-order replies for old request ids or wrong stages never change
  state (validate/approve stages, injected via `handleExplanationReply`):
  `ProposalPreviewTests.testOutOfOrderOrMismatchedRepliesNeverChangeTheDisplayedState`.
- Fake helper crash (exit 3 probe, exit 5 approve), fake helper/provider hang →
  timeout (direct `providerTimeout: 0.4`, not the env variable), garbage, huge
  replies: review stays usable, `canExplain` true, `ShellModel` documents/result/
  revision/`pendingEdit` untouched, approve still goes through the sheet's own
  `approveProposal`: `ProposalPreviewTests.testHelperCrashTimeoutOrGarbageLeavesReviewUsableAndBufferUntouched`.
- Real helper binds context, validates/reviews, explicit `approve`, refuses an
  edit outside the destination: `ProposalPreviewTests.testRealHelperBindsContextAndValidatesProviderReply`;
  pinned fixture byte-for-byte: `testPinnedHelperReproducesRecordedReviewWorkflowFixture`.
- Helper launched offline with a sanitized environment: `testHelperIsLaunchedOfflineWithEmptyArgvAndSanitizedEnvironment`, `testHelperChildEnvironmentIsOfflineAndCredentialFree`.

NOT covered before this lane (implemented here):

- (a) The reviewer's explicit Cancel button path (`cancelExplanation(reason:
  "cancelled by reviewer")`) with the real helper and a provider whose late
  result lands after the cancel: no test drove the button path; no assertion
  that the reply is discarded, the sheet is idle-for-reviewer again, no
  proposal/edit exists.
- (b) The helper's own stale-snapshot refusal (`lib.rs` `Binding::check` →
  "source snapshot is stale") is never exercised by the app: `explain()` and
  `approveReviewedEdit()` always hand the helper the snapshot it bound to, so
  expiry is enforced app-side only (by `update()` cancelling). No distinct
  "expired" note existed; every change collapsed to "proposal or document changed".
- (c) Provider timeout configured through `FLASHTEX_ASSISTANT_TIMEOUT_S`; retry
  after a timeout using a NEW request id; the timed-out process's late result
  (a real process that ignores SIGTERM and answers after the retry started)
  ignored after the retry.
- (d) Helper exit mid-request with the REAL helper (nonzero exit + `error`
  reply), and relaunch evidence (`childLaunches`, a fresh pid) on the next request.
- (e) "No `editApplied` without approve" asserted for the cancel/expiry/timeout/
  exit cases with the real helper (existing test asserts it for fake-helper
  crashes only).

## What was added

- `apps/mac/Sources/FlashTeXMac/AssistantRequestState.swift` (new, 130 lines):
  `AssistantRecoveryEvent` (cancelledByReviewer / expired(from,to) / superseded /
  timedOut / exited / refused / lateReplyDiscarded; reviewer `note` that never
  claims an application), `classify(OneShotProcess.Failure)`, `forChange(old,new)`
  (document/revision move = expiry, proposal/anchor move = superseded),
  `AssistantRecoveryLog` (bounded 32, `lateReplies`, `events(for:)`), and
  `ProposalPreview.ExplanationState.isIdleForReviewer`.
- `apps/mac/Sources/FlashTeXMac/ProposalPreview.swift` (additive, +43/-3):
  `@Published recoveryLog`, `cancelExplanationByReviewer()` (the sheet's Cancel
  button now calls it), `runningChildProcessIdentifier`, `currentHelperRequest`,
  `recoveryNoteText`; `update()` records expiry/supersession before cancelling;
  `handleExplanationReply` records late replies and classified failures; one
  extra caption line in `ProposalExplanationView`. State strings unchanged
  (`.cancelled("proposal or document changed")` etc. still pinned by the old tests).
- `apps/mac/Tests/FlashTeXMacTests/Fixtures/controlled_assistant_provider.py`
  (new, executable): local provider double driven by a JSON control file
  (delay, ignore_sigterm, mode answer/exit/garbage/stale_context, pid_file,
  log_file). No network, no credentials.
- `apps/mac/Tests/FlashTeXMacTests/AssistantRecoveryTests.swift` (new, 6 tests):
  - (a) `testReviewerCancelDiscardsLateProviderResultAndLeavesSheetIdle`
  - (b) `testDocumentChangeExpiresTheRequestAndTheHelperRefusesTheMovedSnapshot`
    (replays the exact accepted `review` request with revision/sha moved through
    the REAL helper → `error` "source snapshot is stale", exit nonzero)
  - (c) `testProviderTimeoutFromEnvironmentThenRetryStartsNewRequestAndIgnoresLateResult`
    (`FLASHTEX_ASSISTANT_TIMEOUT_S=1`; load-aware XCTSkip > 20; a SIGTERM-ignoring
    provider whose complete valid reply lands 0.5 s late is still a timeout)
  - (d) `testHelperExitMidRequestIsSurfacedAndRelaunchedOnNextRequest`
    (real helper SIGKILLed by the pid we launched → "helper exited (9) during
    probe"; helper's own refusal; provider exit 4; relaunch on next request)
  - (e) `testNoEditReachesTheDocumentWithoutExplicitApproveAcrossRecoveries`
    (ShellModel documents/result/revision/pendingEdit/proposals untouched after
    cancel, timeout, helper kill, expiry, and reviewed-edit approve;
    `approveProposal` is the only producer of `pendingEdit`; `editApplied` never called)
  - pure: `testRecoveryEventsClassifyFailuresAndChanges`

## Evidence (all under shared load — not isolated results)

- `swift build` OK (36.6 s cold, 7 s incremental), 09:39 local, load 7.6/31.2/25.5.
- `FLASHTEX_ASSISTANT_CONTEXT=<worktree>/crates/assistant-context/target/release/flashtex-assistant-context`
  (built here with `cargo build --release --offline`, sha256
  `a638cbdf21626d3251d4dba1d98abf722ae3c9a4ff50673840b487268fc4d59b`):
  `swift test --filter AssistantRecoveryTests` → 6/6 passed in 8.5 s
  (uptime: `9:41 up 1 day, 8:33, load averages: 7.19 26.03 24.05`).
  First run had 2 failures fixed before commit: fixture lacked +x so
  `fromEnvironment` refused it; an over-strict note assertion.
- Helper hidden → `swift test --filter AssistantRecoveryTests` → 1 passed, 5 skipped cleanly.
- `swift test --filter ProposalPreviewTests` → 25/25 (1 env-gated skip) in 26.5 s,
  load 5.4→20.1 during the run.
- Full `swift test` NOT run (parent heavy-build window notice at 13:38Z).

## Durable checkpoint

- Branch `agent/mac-assistant-recovery/assistant-recovery`; base `5bc3fc0f`.
- Consumed main SHA: `dda0b62` (via the mac-shell base); origin/main `ffe199d8` observed, not merged.
- Dirty files: see `git status` at each commit; nothing outside the owned paths.
- Helper binary: building in-worktree `cargo build --release --offline` in
  `crates/assistant-context` (log in scratchpad `helper-build.log`); fallback
  `FLASHTEX_ASSISTANT_CONTEXT=/Users/jay3332/Projects/flashtex/crates/assistant-context/target/release/flashtex-assistant-context`.
- Next commands: `cd apps/mac && swift build && swift test --filter AssistantRecoveryTests`
  with `FLASHTEX_ASSISTANT_CONTEXT` set.
- Decisions: keep `.cancelled("proposal or document changed")` wording (existing
  tests pin it); add an additive `recoveryLog` on `ProposalPreview` fed by a
  pure classifier in `AssistantRequestState.swift`; no parent-retained file touched.
- Staffing/billing: shared Claude Max quota with parent; no purchases; MacTeX unused.
