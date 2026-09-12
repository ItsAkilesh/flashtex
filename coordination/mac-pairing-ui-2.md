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

## Delivered (current task, item 14) — branch tip 1e436184

1. PROGRESS: `PairingFlow.Step` (Pairing.swift, pure from the phase: 4 steps —
   show a code / companion enters the code / verify the companion / paired;
   interrupted at the step the attempt reached; nil while receiving or on the
   error banner) and `PairingStepIndicator` (NearbyView.swift) under the
   status row, one spoken element "Pairing step n of 4: …, in progress. Done: …",
   identifier `nearby.pairing.steps`, short capsule captions at min width.
2. CANCELLATION: `NearbyListener.adoptConnections` sends `error
   pairing_cancelled` (id null, "the Mac withdrew the pairing code before
   hello") and closes behind it when a session opened with a bootstrap key the
   new table no longer serves (cancel/expiry/replace/consumed); documented in
   `apps/mac/docs/nearby-v1-proposal.md`. No record can be written after a
   cancel (only `PairingCoordinator.confirm` writes one, and cancel clears it).
3. RECONNECT: machine inputs `companionConnected` / `companionDisconnected`
   (controller sends them for `hello(bootstrap:false)` and for a closed
   session whose pair id is in the store): "X reconnected." / "X disconnected:
   reason." announced, phase untouched; not announced during a receive from
   that pair (peerGone already reports it) or on the error banner. A revoked
   companion fails the TLS-PSK handshake (no name, nothing announced; the log
   says "closed unauthenticated: handshake failed…") — by design, no reason
   can be delivered before authentication.
4. ACCESSIBILITY: `PanelFocusOrder` "Nearby Companion" panel (14 controls in
   source = Tab order, when each is present, keyboard-only pairing: Return
   shows/resumes, Esc cancels/dismisses); `CommandTableTests` now reads
   `NearbyView.swift` (4 panels); Accessibility Help VoiceOver note; ⌘⇧N
   command description and README row spell the in-window keys and Tab order.
   `NearbyView` refused-capture section moved into `refusedSection(_:)`
   declared after the paired rows so source order is the Tab order.

Tests added: `PairingFlowMachineTests` +2 (`testStepIndicatorFollowsThePhase`,
`testKnownCompanionReconnectAndDisconnectAreAnnouncedWithoutChangingThePhase`),
`NearbyViewControllerTests` +2 against the real loopback listener
(`testCancelDuringVerifyingTellsTheCompanionAndLeavesNoRecord`,
`testKnownCompanionReconnectIsAnnouncedAndRevokedIsRefused`);
`CommandTableTests.testPanelFocusOrderMatchesThePanelSources` extended.

Measured: `swift build` clean; `PairingFlowMachineTests` 32/32,
`PairingPersistenceTests` 7/7, `NearbyViewControllerTests` 13/13 (3 consecutive
runs at load 24–44), `CommandTableTests` 8/8, `OverlayTests` 10 (1 env skip),
listener/state/reference-client suites 26 (1 env skip) — all 0 failures.
Evidence: `docs/evidence/nearby-pairing-2026-09-12/ui-2/` (6 real-app window
captures through a loopback pairing, `FLASHTEX_NO_ACTIVATE=1`, never activated).
Full `swift test` at 1e436184 with the seven release helper binaries from the
main checkout (`/Users/jay3332/Projects/flashtex/crates/*/target/release`,
built earlier by the parent; not rebuilt in this worktree): 664 tests, 30
env-gated skips, 1 failure, 200 s, 1-min load 10.6 at start / 23–30 during.
The failure is `DocumentKindsTests.testUndeclareDetachesAndForgetsTheDeclaration`
("helper snapshot failed: helper exited (detached)" vs "no preview controller
attached") — outside this lane (preview-controller detach ordering); it passes
3/3 in isolation with the same helpers (8/8 twice with both env vars, load
18–20). Not a regression from this branch, which touches no controller code.

Follow-up 1 (QR/code display + copyable fallback) and follow-up 2
(per-companion permissions) — NOT started inside the ~75 min bound; the
`textSelection(.enabled)` on the code is the only copy path today.

## Checkpoint

- Branch `agent/mac-pairing-ui-2/pairing-gaps` @ 1e436184 (+ coord commits), pushed. Dirty: none.
- No parent-retained files changed; no diffs requested from the parent.
- Consumed main: c11c005 (via mac-shell cd58fc2e).
- Billing: shared Claude Max quota via parent; no purchases.
