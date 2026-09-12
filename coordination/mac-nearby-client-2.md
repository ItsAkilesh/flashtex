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

## Delivered

Current task (commit d436438): `apps/mac/tools/nearby-client/Tests/Fixtures/{duplicate-delivery,revocation-key-removed,revocation-pair-mismatch,reconnect-idle-drop}.jsonl` (captured client-side transcripts, header with truthful provenance), `Tests/NearbyClientTests/TranscriptRecorder.swift` (onLine/onEvent → normalised JSONL; compare, or record with `NEARBY_CLIENT_RECORD_FIXTURES=1`), `Tests/NearbyClientTests/NearbyTranscriptFixtureTests.swift` (5 tests: fixture well-formedness; duplicate delivery; revocation by same-port restart without the key → `handshakeFailed`/`needsRepair`, no wire line on connection 2, capture never re-sent; `pair_mismatch` at hello → `remote` terminal, no re-send; idle-drop reconnect), `FakeMac(port:)` + `FakeMac.restart(_:keys:)`. README test count 35 → 40.

Follow-up 1 (commit 9e3a5e78): `apps/mac/Sources/FlashTeXMac/CaptureList.swift` — `CaptureListFeature` (off by default; `FLASHTEX_CAPTURE_LIST=1` or defaults key `flashtex.captureList.enabled`), §3 wire shapes (`Request/Reply/Row/Binding/Prepared/Applied`, snake_case, the handoff example round-trips), §4 `Refusal` codes, §6 `outcome(for:ledger:)`, duplicate-free `merge(rows:into:ledger:)`, bounded `Pager` state machine (8 pages, cursor follow, failure keeps rows, reset), `canRestoreDestination` gate; `Tests/FlashTeXMacTests/CaptureListTests.swift` (8 hermetic tests). No bridge call anywhere; the parent's single hook (after `session.open` in `attachBridgeAndWait`) stays out until the bridge answers `capture_list`.

Follow-up 2 (HEAD): `Sources/NearbyClient/NearbyDoctor.swift` + CLI `doctor` (`NearbyCLI`: usage, `--json` flag, `doctor(_:emit:)`), `Tests/NearbyClientTests/NearbyDoctorTests.swift` (7 tests), `NearbyReferenceClientTests.testDoctorReportsExactCodesAgainstNearbyState` (real listener: healthy via Bonjour, `handshake_refused` exit 3 after `forget`, `not_advertised` exit 1 after advertising stops, inbox untouched). README documents the command.

## Validation

- `cd apps/mac/tools/nearby-client && swift test` → 40/40 after the current task (load avg 31) and 47/47 at HEAD (load avg 26; 10.5 s); NearbyTranscriptFixtureTests 5/5 in three consecutive compare-mode runs; NearbyDoctorTests 7/7 (Bonjour case 5.96 s).
- `cd apps/mac && swift test --filter CaptureListTests` → 8/8; `--filter NearbyReferenceClientTests/testDoctorReportsExactCodesAgainstNearbyState` → 1/1 (3.26 s); `--filter 'Nearby|Pairing|CaptureList'` → 110 tests, 3 env-gated skips (no FLASHTEX_BRIDGE / opt-in serve / screenshot), 0 failures (52.3 s, load avg 23–29). Full `swift test` with real helpers NOT run: 1-min load stayed 23–38 (> 15 threshold in the brief).
- Measured (loopback, this Mac, load ~30): revoked key via same-port restart reported terminal in 0.013 s with 1 refused reconnect; doctor healthy run 0.006 s; doctor refused port 0.004 s.

## Limitations

- No iPad simulator transcript: `apps/companion` is absent on this branch and every published companion connects in plaintext (mac-nearby-transport handoff), so the fixtures are FakeMac captures; each fixture header says so.
- Fixtures compare by normalised shape (ids/nonce/proof placeholders, error kinds), not bytes; the TLS alert text and close reasons vary by OS and are reduced to `handshake_failed` / `peer_closed`.
- `CaptureList` is a stub: no `TransferV1.Request.captureList` case, no `BridgeSession.list`, no `CaptureState.pending` (BridgeSession.swift and ShellModel+Bridge.swift untouched); `MergeDecision.Kind.add(Outcome)` is what the parent maps onto a new `CaptureState` when wiring.
- `doctor` validates one pairing per run; the `tls_unknown` warning path (framework reports no negotiated parameters) is not exercised by a test.

## Checkpoint (durable)

- Branch `agent/mac-nearby-client-2/fixtures` (pushed), base cd58fc2e; consumed main c11c005 (via mac-shell). Worktree `.claude/worktrees/agent-a4b2202c482239b72`.
- Commits: d436438 (current task), 9e3a5e78 (follow-up 1), ec026d4d (follow-up 2); this checkpoint commit on top.
- Dirty files after HEAD: none. No background jobs. No parent-retained file touched (no diffs to request).
- Next: parent reviews the branch diff against `origin/agent/mac-claude-a/mac-shell` and merges; optional full `swift test` in apps/mac with real helpers when load < 15.
- Decisions: fixtures normalise volatile fields (envelope ids, nonce, proof) to stable placeholders so a replay compares by shape; base64 of the 1×1 PNG is kept verbatim; `doctor` reuses the `send` exit-code table so scripts can treat both alike.
- Staffing/billing: parent's Claude Max 20x quota only; no purchases; no subagents.
