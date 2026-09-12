# mac-nearby-client-2 handoff

- Updated UTC: 2026-09-12T16:15:00Z
- Agent / parent / machine alias: mac-nearby-client-2 (Claude Code subagent, Opus) / mac-claude-a / mac-m1max-a
- Task: Commander replenishment issue #2 comment 5646989044, item 13 "Nearby client". Current task: exercise DUPLICATE DELIVERY, REVOCATION, RECONNECT with offline/simulator fixtures; add missing fixtures as captured JSONL transcripts. Follow-up 1: `capture_list` client-side state machine stub behind a feature flag. Follow-up 2: CLI `nearby-client doctor`.
- Owned: `apps/mac/tools/nearby-client/**`, `apps/mac/Tests/FlashTeXMacTests/NearbyReferenceClientTests.swift`, this file, `coordination/agents/mac-nearby-client-2.json`. Files created by the finished lanes mac-nearby-client / mac-nearby-transport that I may edit: `apps/mac/tools/nearby-client/{Package.swift,Sources/**,Tests/NearbyClientTests/{FakeMac,NearbyClientTests,NearbyReconnectTests,NearbyReceiveCapTests}.swift,scripts/measure-native.py}`. Not touched: `NearbyListener*.swift`, `NearbyState`, `ShellModel+Nearby.swift`, `Fixtures/nearby-companion-session.jsonl` (mac-nearby-transport, listener side).
- Branch: `agent/mac-nearby-client-2/fixtures` from `origin/agent/mac-claude-a/mac-shell` cd58fc2e (main c11c005 merged). Worktree `.claude/worktrees/agent-a4b2202c482239b72`.

## Coverage audit (existing, before this lane)

Duplicate delivery (same `capture_id` re-sent after a drop before ack; Mac stores once):
- `apps/mac/tools/nearby-client/Tests/NearbyClientTests/NearbyReconnectTests.swift`: `testSubmitResendsSameCaptureAfterDropBeforeAck` (FakeMac swallows the capture, cuts; identical re-send).
- `apps/mac/Tests/FlashTeXMacTests/NearbyReferenceClientTests.swift`: `testReconnectorResendsSameCaptureAfterListenerDropAndSamePortRestart` (real listener, same-port restart, inbox stores once, "Duplicate … acknowledged again."), `testSameSessionRetryIsAcknowledgedWithoutRedeliveryAndRevisionMismatchIsTerminal`.
- `apps/mac/Tests/FlashTeXMacTests/NearbyListenerTests.swift`: `testDuplicateIsAcknowledgedNotRedeliveredAndMismatchesAreRefused`, `testRefusalsAndDuplicatesAreVisibleState`, `NearbyTranscriptAcceptanceTests.testTranscriptMatchesTheRecordedSessionShape` + `testRecordedSessionIsAcceptedOnLoopbackWithDuplicatesAbsorbed` (replays `Fixtures/nearby-companion-session.jsonl`, the recorded-shape simulator transcript, listener side: doubles absorbed, verbatim replay refused on nonce, fresh-hello reconnect re-sends acknowledged without storing twice).
- Native: `docs/evidence/nearby-client-recovery-2026-09-12T092814Z.md` (3 s outage + port change recovered on attempt 2).

Revocation (forgotten pairing refused on reconnect, typed error, no re-delivery):
- `NearbyReconnectTests.testRefusedKeyIsTerminalWithoutRetry` (FakeMac with another key under the pair_id → `handshakeFailed`, `needsRepair`, zero sleeps, zero captures).
- `NearbyReferenceClientTests.testRevokedPairingIsTerminalAfterOneRefusedReconnect` (real `NearbyState.forget` while live: session closed by the Mac, one refused reconnect, `handshakeFailed`/`needsRepair`, inbox unchanged, CLI exit 3 without "attempt 2"), `testRevokedDestinationIsTerminalAgainstShellModel` (destination revoked → `destinationChanged`).
- `NearbyListenerTests`: `testPairForgetAndRestartThroughState`, `testWrongPSKIsRefusedBeforeAnyLineIsParsed`, `testSessionKeyedByReplacedCodeIsRefusedAtHello`, `testPersistedGenerationRefusesOlderAttemptsAndAllowsNewerRepair`.
- Native evidence: revoked pairing exit 3 in 0.28 s with no retry.

Reconnect (bounded attempts/backoff, drop while idle, unreachable, deadline, cancellation, destination re-check):
- `NearbyReconnectTests`: `testDropWhileIdleReconnectsOnNextSubmit`, `testAttemptsExhaustedAfterBoundedUnreachableAttempts`, `testDestinationChangedOnRetryIsTerminal`, `testReusedSessionRechecksDestinationBeforeEachSubmit`, `testDeadlineStopsRetriesBeforeMaxAttempts`, `testCancellationSurfacesAsCancelled`, `testConnectionRefusedIsUnreachableFast`, `testRequestTimeoutIsBoundedAndRetryable`, `testCLISendRetriesAndReportsTerminalExitCodes`; `NearbyReceiveCapTests.testTooManySessionsIsRetriedAfterReconnect`; `NearbyClientTests.testPairSendReconnectViaLibrary`, `testCLICommandsAgainstFakeMac`.
- `NearbyReferenceClientTests.testCLIPairsSendsRetriesAndIsForgottenThroughNearbyState`, `testTooManySessionsIsRetriedAfterClosingTheOlderConnection`.

Gaps found (the uncovered part this lane implements):
1. No client-side transcript fixtures exist: `apps/mac/tools/nearby-client` has no `Tests/Fixtures`; the only JSONL fixture is the listener-side `nearby-companion-session.jsonl` (mac-nearby-transport), and the client package (which the FT-004 companion copies verbatim) cannot depend on apps/mac. The three scenarios are asserted through counters/events, never as a captured wire transcript, so the exact line-by-line shape a companion must reproduce (hello → hello_ack → capture_submit → cut → hello → hello_ack → identical capture_submit → capture_received; revocation → TLS refusal with no line, or `error pair_mismatch` then close) is not pinned anywhere.
2. Revocation *between* an unacknowledged capture and the reconnect is not covered: in the existing tests the capture was acknowledged before `forget`. The must-hold property "a revoked pairing never re-delivers a capture" after a drop-before-ack (the reconnector holds an identical re-send ready) is untested. Both the TLS refusal (Mac restarted without the key) and the application-level `pair_mismatch` refusal need a case.
3. The simulator target cannot be used for these scenarios: `apps/companion` is absent on this branch and, per `coordination/mac-nearby-transport.md`, every published companion connects with plain `NWParameters.tcp` (no TLS-PSK / v1 hello), so a simulator run can only reproduce the handshake refusal. Fixtures are therefore captured from the reference client against `FakeMac` on loopback (truthful provenance recorded in each fixture's first line).

## Checkpoint (durable)

- Branch `agent/mac-nearby-client-2/fixtures`, base cd58fc2e; consumed main c11c005 (via mac-shell).
- Dirty files: this file (audit written); implementation not started.
- Next commands: add `FakeMac(port:)` + `stopKeepingPort`, `Tests/NearbyClientTests/TranscriptRecorder.swift`, `Tests/Fixtures/*.jsonl` (recorded with `NEARBY_CLIENT_RECORD_FIXTURES=1`), `Tests/NearbyClientTests/NearbyTranscriptFixtureTests.swift`; `cd apps/mac/tools/nearby-client && swift test --filter NearbyTranscriptFixtureTests`, then full package `swift test`; register json; commit with the lane trailers; push.
- Decisions: fixtures normalise volatile fields (envelope ids, nonce, proof) to stable placeholders so a replay compares by shape; base64 of the 1×1 PNG is kept verbatim.
- Staffing/billing: parent's Claude Max 20x quota only; no purchases; no subagents.
