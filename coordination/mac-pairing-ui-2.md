# mac-pairing-ui-2 handoff

- Updated UTC: 2026-09-12T12:20Z
- Agent / parent / machine alias: `mac-pairing-ui-2` (Claude Code subagent) /
  `mac-claude-a` / `mac-m1max-a`
- Task: Commander replenishment (issue #2 comment 5646989044) item 14 "Pairing
  UI": progress, cancellation, reconnect, accessibility gaps; follow-up 1
  QR/code display with copyable fallback; follow-up 2 per-companion permissions.
- Owned: `apps/mac/Sources/FlashTeXMac/Pairing.swift`, `NearbyView.swift`,
  `apps/mac/Tests/FlashTeXMacTests/PairingTests.swift`, `NearbyViewTests.swift`,
  this file, `coordination/agents/mac-pairing-ui-2.json`. Files created by
  finished, merged lanes that this lane may edit (audit list): `NearbyListener.swift`,
  `NearbyState.swift`, `NearbyProtocol.swift` (mac-nearby-transport);
  `FlashTeXAccessibility/AccessibilityCommands.swift`,
  `Tests/FlashTeXAccessibilityTests/CommandTableTests.swift` (mac-editor-accessibility);
  `apps/mac/README.md` nearby/shortcut rows. Parent-retained files stay diff requests.
- Branch: `agent/mac-pairing-ui-2/pairing-gaps` from
  `origin/agent/mac-claude-a/mac-shell` cd58fc2e (main c11c005 merged).
  Worktree `.claude/worktrees/agent-a260d28501947001c`.

## Coverage audit (≤10 min, done 12:20Z)

Already covered on mac-shell cd58fc2e (file: test):

- PROGRESS — `PairingTests.swift: PairingFlowMachineTests` (30: every phase
  transition, stale/ignored inputs, receiving n/m bytes,
  `testPhaseTextIsPreciseAndCountdownFree`), `NearbyViewTests.swift:
  NearbyViewControllerTests` (11 against the real loopback TLS-PSK listener:
  `testShowCodePairsAndJournalsThroughTheController`,
  `testLiveReceivingProgressCompletesAndCanBeCancelled`,
  `testRefusedCaptureEndsReceivingWithTheErrorCode`), status row
  `nearby.pairing.state` with title + sentence. **Uncovered:** no numbered
  step indicator (step k of n) — the row names the phase but not where it
  sits in the pairing sequence.
- CANCELLATION — `PairingFlowMachineTests.testCancelAtEachCancellableStep`,
  `NearbyViewControllerTests.testCancelWithdrawsTheBootstrapKeyAndClearsTheJournal`
  (bootstrap key refused after cancel, journal cleared),
  `NearbyListenerTests.testBootstrapPairingHandsOverLongTermPSKAndSurvivesRestart`,
  `PairingGenerationTests` (replaced attempt cannot confirm). A record is only
  written by `PairingCoordinator.confirm` on a valid bootstrap hello, so no
  half-paired record can exist. **Uncovered:** a companion that already opened
  the bootstrap TLS session (verifying) is closed with a bare TCP close
  ("key table changed before hello", `NearbyListener.adoptConnections`) —
  it is not told why; no test asserts `store.pairs` is empty after a cancel
  during verifying.
- RECONNECT — `NearbyListenerTests.testBootstrapPairingHandsOverLongTermPSKAndSurvivesRestart`
  (long-term key reconnects across restart), `NearbyStateTests.
  testPairForgetAndRestartThroughState`, `NearbyReferenceClientTests.
  testReconnectorResendsSameCaptureAfterListenerDropAndSamePortRestart`,
  `testRevokedPairingIsTerminalAfterOneRefusedReconnect`,
  `testRevokedDestinationIsTerminalAgainstShellModel` (companion side:
  revoked → refused at the TLS handshake, terminal after one retry),
  `NearbyViewControllerTests.testAlreadyPairedCompanionConnectingDuringACodeIsNotThePairingPeer`,
  `testReplacedCodeMakesTheOldSessionStaleEndToEnd`; the device row shows the
  "connected" badge from `connectedPairIds`. By design (nearby-v1 proposal) a
  revoked key fails the TLS-PSK handshake, so no reason can reach the
  companion before authentication; a stale bootstrap key that still
  handshakes gets `pair_mismatch`. **Uncovered:** the Mac UI/VoiceOver says
  nothing when a known companion reconnects or drops while no pairing is in
  progress (`otherCompanionConnected`/`peerGone` are ignored in `advertising`).
- ACCESSIBILITY — labels/values/identifiers on every Nearby control
  (`NearbyView.swift`), focus moves with the phase (`moveFocus`), Return/Esc
  shortcuts, spoken digits, `.updatesFrequently` countdown; `CommandTableTests`
  covers ⌘⇧N in the command table and README. **Uncovered:** the Nearby
  Companion window is not in `PanelFocusOrder.panels` (Settings, Durable
  History, Find in Project only), so its Tab order and in-window keys are not
  in the Accessibility Help window nor checked against the source; the README
  ⌘⇧N row does not spell the in-window keys.
- Evidence already on disk: `docs/evidence/nearby-pairing-2026-09-12/` (14 PNGs:
  8 hosted-view states, 6 real-app states), validation report
  `tools/native-validation/mac-live/reports/20260912T110944Z.md` (open-window/nearby PASS).

## Plan (uncovered parts only)

1. `PairingFlow.Step` (pure) + step indicator row in `NearbyFlowView`; tests.
2. Listener tells an unauthenticated bootstrap session `pairing_cancelled`
   before closing when the bootstrap key is withdrawn; test asserts no record.
3. Reconnect/disconnect of a known companion announced to VoiceOver.
4. `PanelFocusOrder` Nearby panel + `CommandTableTests` + README row.

## Checkpoint

- Branch `agent/mac-pairing-ui-2/pairing-gaps` @ cd58fc2e (no commits yet).
- Dirty: this file, `coordination/agents/mac-pairing-ui-2.json`.
- Next: implement 1–4, `swift build`, `swift test --filter "Pairing|NearbyView|CommandTable"`, commit, push.
- Consumed main: c11c005 (via mac-shell cd58fc2e).
- Billing: shared Claude Max quota via parent; no purchases.
