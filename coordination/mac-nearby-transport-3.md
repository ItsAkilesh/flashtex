# mac-nearby-transport-3 — multi-interface LAN advertisement + persisted ack memory

Lane: Claude Code subagent of mac-claude-a on mac-m1max-a. Branch
`agent/mac-nearby-transport-3/lan-advertise` from `origin/agent/mac-claude-a/mac-shell` @ 82749c26.
Gap: Commander list 5646989044 item 12 follow-up (mac-nearby-transport-2 measured loopback only) plus the
`stopAdvertising()` ack-memory drop noted in `coordination/mac-nearby-transport-2.md`.

## Coverage audit (existing, at 82749c26)
Already covered — not redone:
- `NearbyInterfaceTests.testIPv4IPv6AndLinkLocalConnectionsReachTheSameListener` — `127.0.0.1`, `::1`,
  `fe80::1%lo0` against a **loopback-only** listener (`loopbackOnly: true`); same identity per family.
- `NearbyInterfaceTests.testBonjourAdvertisementCarriesTXTAndResolvesToTheListener` — NWBrowser finds the service,
  TXT == `NearbyState.txtRecord`, `NearbyResolver.resolve` (kDNSServiceInterfaceIndexAny) → port; loopback-only, so
  `result.interfaces` was `lo0` only.
- `NearbyListenerTests.testBonjourAdvertisingReachesReady` — the only all-interface bind; waits for `.ready` only.
- `NearbyReconnectTests.testAckMemorySurvivesSamePortRestartAndFollowsForgottenPairings` — memory across
  `adoptConnections` (key-table restart), NOT across `NearbyState.stopAdvertising()` (listener = nil drops memory).
- `NearbyReconnectTests.testRetryAfterDropBeforeAckIsAcknowledgedFromMemoryNotRedelivered`,
  `testPeerDropMidFrameReleasesEverythingAndTheReconnectDeliversOnce`, `testDropDuringValidationForgetsThePendingEntry`
  — loopback reconnects on one address.
- `NearbyReconnectTests.testHandshakeHelloAndFrameDeadlinesCloseStalledPeers` — deadlines measured on loopback only.
- `NearbyReferenceClientTests.testReconnectorResendsSameCaptureAfterListenerDropAndSamePortRestart` — reference client
  reconnector on 127.0.0.1.
- `NearbyListenerTests.testWrongPSKIsRefusedBeforeAnyLineIsParsed` — wrong key refused on loopback.
- `tools/native-validation/mac-live/reports/20260912T110944Z.md` — only opens the Nearby window (screenshot); no
  transport measurements. `docs/evidence/nearby-*` — pairing/client recovery, loopback.

Uncovered (this lane):
1. Listener bound to all interfaces reached over the Mac's real non-loopback interfaces (`en*` IPv4 + `fe80::%en*`),
   client scoped to the interface; Bonjour `result.interfaces` per interface (utun excluded) and per-interface
   DNSServiceResolve; wrong-key/fp-mismatch refusal on a non-loopback address; reconnect via a second address after the
   first is closed (retry acknowledged from memory); frame timeout measured on a non-loopback path.
2. Ack memory across `NearbyState.stopAdvertising()`/`startAdvertising()` within a session, and persisted (bounded)
   with the pairing record.

Machine state at start: `en0` 172.26.0.157 + `fe80::1cb7:cf63:4020:a044%en0`; `en9` 169.254.173.203 +
`fe80::82f:598a:7574:62ec%en9`; `utun0–5`, `bridge0`, `awdl0`, `llw0` present. Load 11.3/22.5/29.7 at 12:41.

## Checkpoint
- Branch `agent/mac-nearby-transport-3/lan-advertise` @ 82749c26 (not yet pushed); worktree
  `.claude/worktrees/agent-af561f87772128874`; dirty: this file.
- Consumed mac-shell 82749c26. Next: implement `NearbyLANInterfaceTests.swift`, persisted ack memory, register, push.
