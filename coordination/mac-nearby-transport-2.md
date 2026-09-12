# mac-nearby-transport-2 — nearby disconnect/reconnect + bounded decoding (item 12)

Lane: Claude Code subagent `mac-nearby-transport-2` (parent `mac-claude-a`, mac-m1max-a).
Branch: `agent/mac-nearby-transport-2/reconnect` from `origin/agent/mac-claude-a/mac-shell` @ cd58fc2e (contains main c11c005).
Bounded ~75–90 min from 2026-09-12T12:04 local (uptime load 8.3/10.1/9.0 at start). Claude Max quota only; no provider calls.
Owned: `apps/mac/Sources/FlashTeXMac/{NearbyListener,NearbyProtocol,NearbyState,NearbyDestination,ShellModel+Nearby}.swift`
(+ new `Nearby*.swift`), `apps/mac/Tests/FlashTeXMacTests/Nearby*Tests.swift`, this file,
`coordination/agents/mac-nearby-transport-2.json`. Parent-retained files stay diff requests.

## Coverage audit (mandatory first step, 12:04–12:16 local)

Sources read: `NearbyListener.swift`, `NearbyProtocol.swift`, `NearbyState.swift`, `ShellModel+Nearby.swift`,
`FlashTeXProtocol/JSONLines.swift` (`LineSplitter`), the four `Nearby*Tests.swift`, the nearby-client package tests
(`NearbyReconnectTests`), handoffs `coordination/mac-nearby-{transport,client,errors}.md`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md` (no nearby transport rows beyond the pairing window),
`docs/evidence/nearby-client-recovery-2026-09-12T{084502Z,092814Z}.md`.

| Sub-gap (from the brief) | Already covered (file:test) | Verdict |
|---|---|---|
| Companion drops mid-frame (Mac-initiated) | `NearbyListenerTests.testOnEventCloseConnectionAndResumedPairing` (Mac closes a session mid-line; budget released, pairing kept) | covered for the Mac side |
| Companion drops mid-frame (peer-initiated) | none: no test sends a partial `capture_submit` and cancels from the client side, then reconnects with the same pairing | **uncovered** |
| Listener restart on the same port | `NearbyReferenceClientTests.testReconnectorResendsSameCaptureAfterListenerDropAndSamePortRestart`, `…testRevokedPairingIsTerminalAfterOneRefusedReconnect` (`state.port` equal across the key-table restart), `NearbyListenerTests.testBootstrapPairingHandsOverLongTermPSKAndSurvivesRestart`, `…testPairForgetAndRestartThroughState` | covered |
| Half-open sockets (peer vanished without FIN) | none: `NWProtocolTCP.Options.enableKeepalive = true` only (system default idle ≈ 2 h); no listener-side handshake, hello or frame deadline exists, so a TLS-accepted connection that never sends `hello`, or a TCP peer that never starts TLS, holds one of the 16 connection slots forever | **uncovered** |
| Backoff with jitter (companion) | nearby-client `NearbyReconnectTests.testPolicyScheduleIsBoundedAndDeterministic` (±20 % jitter bounds, cap, deterministic), `…testAttemptsExhaustedAfterBoundedUnreachableAttempts`, `…testDeadlineStopsRetriesBeforeMaxAttempts`; `NearbyReferenceClientTests` (against the real listener) | covered (client side; the listener never dials) |
| Session resumption with the same pairing, never a duplicate delivery | same-session dedup: `NearbyListenerTests.testDuplicateIsAcknowledgedNotRedeliveredAndMismatchesAreRefused`, `…testSinkRefusalIsNotRememberedAndBoundsHold`, `…testRecordedSessionIsAcceptedOnLoopbackWithDuplicatesAbsorbed`, `NearbyReferenceClientTests.testSameSessionRetryIsAcknowledgedWithoutRedeliveryAndRevisionMismatchIsTerminal`; across a reconnect the sink dedups (`NearbyInbox.store` → "Duplicate … acknowledged again", asserted by `testReconnectorResendsSameCaptureAfterListenerDropAndSamePortRestart`: `sink.deliveries == 1` on the first listener, inbox count 1) | covered at the sink; **uncovered at the transport**: `NearbySession.remembered` is per session, so a retry on a new session of the same pairing is delivered to the sink again (a second `capture_submit` reaches the bridge/inbox and relies on their dedup) |
| Frame size limits | `NearbyListenerTests.testOversizedLineClosesConnection` (complete and unterminated line > `maxLineBytes` → `line_too_long`, close), nearby-client `testOversizedInboundLineClosesTheConnection` | covered |
| Malformed/oversized frames refused with typed errors before allocation | error *codes* are strings produced inline; `Connection.consume` appends the chunk to `LineSplitter` first and checks `pendingBytes >= maxLineBytes` afterwards, so the over-limit buffer is allocated before the refusal; no typed error value | **uncovered** (typed error + pre-append bound) |
| Slow-loris partial frames timed out | none (no timer anywhere in `NearbyListener`) | **uncovered** |
| Exact capture/session/destination identity retained | `testHelloCaptureAndDestinationRoundTrip`, `testTranscriptMatchesTheRecordedSessionShape`, `testDestinationFollowsPinnedAnchor`, `NearbyErrorsTests.*` | covered; new work must keep these green |

Follow-up 1 (IPv6/link-local, multiple interfaces): `testBonjourAdvertisingReachesReady` only waits for `.ready` with a
service; no IPv6 connection test, no TXT/interface assertions. **Uncovered.**
Follow-up 2 (metrics line in the Nearby window): `NearbyViewTests` show code/progress/errors; no frames/refusals/reconnects
counters exist in `NearbyState` or `NearbyView`. **Uncovered.**

## Plan (current task, uncovered parts only)
1. `NearbyReceiveLimits`: `handshakeTimeout` (10 s), `helloTimeout` (10 s), `frameTimeout` (60 s per unterminated line),
   TCP keepalive idle/interval/count (15 s / 5 s / 4) on the listener parameters.
2. `NearbyFrameError` (typed): `lineTooLong`, `unterminatedLineTooLong`, `frameTimeout`, `helloTimeout`, `handshakeTimeout`
   with `code`/`message`; the pre-append bound refuses a chunk that cannot complete a line inside `maxLineBytes` without
   appending it.
3. Per-pairing acknowledgement memory shared across sessions of one listener (and adopted on restart) so a retry after a
   reconnect is acknowledged from memory, never re-delivered to the sink; pending deliveries coalesce the retry.
4. Tests in a new `NearbyReconnectTests.swift` (loopback only, no devices): peer drop mid-frame + reconnect exactly-once,
   handshake/hello/frame timeouts, pre-append bound, cross-session duplicate, keepalive parameters.

## Checkpoint
- Branch `agent/mac-nearby-transport-2/reconnect`, worktree `.claude/worktrees/agent-a9d555eb622fb7706`, HEAD: see git.
- Consumed main: c11c005 (via mac-shell cd58fc2e).
- Dirty files: none yet (audit commit next).
- Next: implement 1–4, `swift build`, `swift test --filter Nearby`, commit, push, register, report.
