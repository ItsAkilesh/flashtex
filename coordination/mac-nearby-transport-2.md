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

## Result (16:04Z–16:36Z, ~32 min)

Branch `agent/mac-nearby-transport-2/reconnect`, tip d16474b2 (pushed), base mac-shell cd58fc2e (main c11c005).
Commits: 20b2b6b9 audit · 5645a6d2 current task · 1d52b8f6 follow-up 1 · d16474b2 follow-up 2.

### Current task (disconnect/reconnect + bounded decoding) — 5645a6d2
- `NearbyProtocol.swift`: `NearbyReceiveLimits.handshakeTimeout` 10 s, `helloTimeout` 10 s, `frameTimeout` 60 s,
  keepalive idle/interval/count 15/5/4; `NearbyFrameError` (typed: `lineTooLong`, `unterminatedLineTooLong`,
  `frameTimeout`, `helloTimeout`, `handshakeTimeout` → `code`/`message`/`reason`); `NearbyAckMemory` (per-pairing
  pending/acknowledged captures, thread-safe, bounded 256/pair, `wait`/`acknowledge`/`forget`); `NearbySession`
  dedup moved from per-session `remembered` to the shared memory (a session's `end()` keeps it; sink replies that
  land after the session is gone still acknowledge the pairing's entry and answer queued retries; a session that
  ends before validation forgets the entry so the retry is delivered afresh).
- `NearbyListener.swift`: `memory` shared across `adoptConnections` (forgotten pairings dropped); TCP keepalive
  schedule on `parameters(psks:loopbackOnly:limits:)`; `Connection` deadlines (`arm`/`disarm`, one at a time:
  handshake → hello → per-frame, absolute, cleared by the newline; before hello the frame deadline is
  `min(frame, hello)`); `refuse(NearbyFrameError)`; pre-append bound in `consume` (a chunk that cannot finish the
  pending line inside `maxLineBytes` is refused without buffering); `.capture`/`.refused`/`.duplicate` events
  still reach the listener after the delivering connection closed.
- `nearby-v1-proposal.md` §4: deadline rows, pre-append bound, pairing-scoped dedup, `frame_timeout`/`hello_timeout`.
- Tests `NearbyReconnectTests` (7): `testPeerDropMidFrameReleasesEverythingAndTheReconnectDeliversOnce`,
  `testRetryAfterDropBeforeAckIsAcknowledgedFromMemoryNotRedelivered`, `testDropDuringValidationForgetsThePendingEntry`,
  `testAckMemorySurvivesSamePortRestartAndFollowsForgottenPairings`,
  `testHandshakeHelloAndFrameDeadlinesCloseStalledPeers` (load-aware, skips at 1-min load > 20; measured at load 12.9:
  handshake close 0.41 s / hello 0.41 s / frame 0.61 s for deadlines 0.4/0.4/0.6 s, no application bytes to the
  silent TCP peer, live peer untouched through 0.9 s idle),
  `testOversizedPartialFrameIsRefusedBeforeItIsBuffered`, `testKeepaliveScheduleIsOnTheListenerParameters`.
- Two existing assertions updated for pairing-scoped memory: `NearbySessionBoundsTests.testSinkRefusalIsNotRememberedAndBoundsHold`
  (`end()` keeps 3 entries; `memory.forget(pairId:)` clears) and
  `NearbyTranscriptAcceptanceTests.testRecordedSessionIsAcceptedOnLoopbackWithDuplicatesAbsorbed` (reconnect
  retries are `.captureDuplicate` at the listener — 6 duplicate events — and never reach the inbox again).

### Follow-up 1 (IPv6/link-local, Bonjour) — 1d52b8f6
No listener defect found; `NearbyInterfaceTests` (2): the same pairing over `127.0.0.1`, `::1` and scoped
`fe80::1%lo0` (remote endpoints measured `::1.<port>`, `fe80::1%lo0.<port>`) with identical hello_ack identity;
NWBrowser finds the advertised service, TXT == `NearbyState.txtRecord` (v/name/fp/salt), advertised on `lo0/loopback`
(loopback-only state), `NearbyResolver.resolve` (the reference client's DNSServiceResolve path) returns the listener's
port and the resolved host accepts the TLS-PSK session. Multi-interface LAN advertisement was not measured (no
non-loopback port is opened by tests; the existing `testBonjourAdvertisingReachesReady` is the only all-interface bind).

### Follow-up 2 (metrics line) — d16474b2
`NearbyMetrics.swift` `NearbyTransportMetrics` (frames/accepted/refused/duplicates, sessions/reconnects,
closed/timedOut, listenerRestarts; `line`; `reset()` keeps started/seen state); `NearbyState.metrics` (+`resetMetrics()`)
fed from `handle(_:)`; `NearbyView` "Transport" grid row (`nearby.transport.metrics`, also in the combined
service-details accessibility value). `NearbyMetricsTests` (2): event counters + reset; real loopback sessions
(capture, reconnect retry = duplicate with inbox count 1, refusal, hello deadline at 0.3 s, key-table restart) →
`frames 3 (1 refused, 1 duplicate) · sessions 2 (1 reconnect) · closed 2 (1 timed out)`.

### Evidence
- `cd apps/mac && swift build` clean (warnings only from another lane's `DocumentKindsTests.swift`).
- `swift test --filter 'Nearby|Pairing|Capture'` at d16474b2: 124 tests, 0 failures, 9 skipped (8 env-gated in
  other lanes + the deadline test when load > 20) at load 31–34; `NearbyReconnectTests` 3 consecutive runs green.
- Full `swift test` with real helpers NOT run: 1-min load was 24–87 during the lane (bound: < 15).
- Under load 65 the first version of `testDropDuringValidationForgetsThePendingEntry` timed out (utility-QoS image
  validation of a 1 MiB PNG starved) and crashed on an index; fixed (96² image, 15 s waits, guarded index).

### Limitations / notes for the parent
- Parent-retained files untouched; no diffs requested. `NearbyView.swift` (mac-pairing-ui, finished) got the one
  grid row; `NearbyListenerTests.swift` (mac-nearby-transport, finished) two assertions.
- Ack memory lives with the listener: `stopAdvertising()` drops it (sink/bridge dedup still applies, as before).
- Frame deadline is absolute per frame (60 s ⇒ ≥ 200 KiB/s for a full 12 MiB frame); tune `NearbyReceiveLimits.frameTimeout`
  if slower links must be supported.
- Keepalive half-open detection is asserted on the NWParameters only (a real half-open socket cannot be produced on loopback).
- Reference client (`tools/nearby-client`) untouched: `frame_timeout`/`hello_timeout` are followed by a close, which
  its reconnector already treats as retryable.

## Checkpoint (final)
- Branch `agent/mac-nearby-transport-2/reconnect` @ d16474b2, pushed; worktree `.claude/worktrees/agent-a9d555eb622fb7706`; dirty: none.
- Consumed main c11c005 via mac-shell cd58fc2e. Next: parent review/merge into mac-shell; rerun full suite when load < 15.
