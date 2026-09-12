# mac-pairing-ui-3 handoff

- Updated UTC: 2026-09-12T12:45Z
- Agent / parent / machine alias: `mac-pairing-ui-3` (Claude Code subagent) /
  `mac-claude-a` / `mac-m1max-a`
- Task: Commander replenishment (issue #2 comment 5646989044) item 14 follow-ups
  left unstarted by mac-pairing-ui-2: (1) pairing code as a QR image plus
  copyable text fallback; (2) per-companion permissions (captures allowed /
  view only) persisted with the pairing record and enforced by the listener.
- Owned (new files preferred): `apps/mac/Sources/FlashTeXMac/PairingQR.swift`,
  `Tests/FlashTeXMacTests/PairingQRTests.swift`, `Tests/FlashTeXMacTests/CompanionPermissionTests.swift`,
  this file, `coordination/agents/mac-pairing-ui-3.json`. Files created by
  finished, merged lanes this lane may edit: `Pairing.swift`, `NearbyView.swift`,
  `NearbyState.swift`, `NearbyProtocol.swift`, `NearbyListener.swift`,
  `PairingTests.swift`, `NearbyViewTests.swift`, `tools/nearby-client/*`
  (mac-nearby-client), `FlashTeXAccessibility/AccessibilityCommands.swift`,
  `Tests/FlashTeXAccessibilityTests/CommandTableTests.swift`, `apps/mac/docs/nearby-v1-proposal.md`.
  Parent-retained files stay diff requests.
- Branch: `agent/mac-pairing-ui-3/qr-permissions` from
  `origin/agent/mac-claude-a/mac-shell` 82749c26 (main 486b759c at start).
  Worktree `.claude/worktrees/agent-a7cbf7978cc4be2a1`.

## Coverage audit (≤10 min, done 12:45Z)

Searched `apps/mac/Sources/FlashTeXMac/{Pairing,NearbyView,NearbyState,NearbyProtocol,NearbyListener}.swift`,
`apps/mac/Tests/FlashTeXMacTests/*`, `apps/mac/tools/nearby-client/Sources/NearbyClient/*`,
`apps/mac/docs/nearby-v1-proposal.md`, `coordination/mac-pairing-ui-2.md`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/nearby-pairing-2026-09-12/`.

Follow-up 1 (QR + copyable code):
- Already covered: the text code `Pairing.displayCode` ("123 456", `NearbyView.swift: codeRow` with
  `.textSelection(.enabled)` — the only copy path), VoiceOver one-digit-per-word
  `Pairing.spokenCode` (`PairingTests.swift: PairingFlowMachineTests.testPhaseTextIsPreciseAndCountdownFree`
  asserts "Code 1 2 3 4 5 6…"; `NearbyViewTests.swift: NearbyViewControllerTests.testShowCodePairsAndJournalsThroughTheController`
  asserts the announcement). Reference client pairs from `--code` + Bonjour `--mac <fp>` or
  `--host/--port/--salt` (`NearbyReferenceClientTests.testCLIPairsSendsRetriesAndIsForgottenThroughNearbyState`,
  `testCLIDirectModeAndListenerRefusalsAgainstRealListener`).
- Uncovered (no source, no test): `CIQRCodeGenerator`/QR anywhere in `apps/mac` (grep `QR|CIQRCode`: 0 hits
  outside the proposal's threat-model remark about a 32-byte QR); no "Copy code" button, no ⌘C/`onCopyCommand`
  on the code, no pair-grouped VoiceOver value, no bootstrap payload format shared with the reference client,
  no test decoding a QR and pairing the reference client from it.

Follow-up 2 (per-companion permissions):
- Already covered: `PairStore` schema versioning v1→v2 with upgrade/refusal tests
  (`PairingTests.swift: PairingPersistenceTests` 7 tests: newer version refused, v1 upgraded, mode 0600);
  refusal plumbing `NearbySession.refuse` → `NearbyListener.Event.captureRefused` → `NearbyState.lastReceiveError`
  → `NearbyView.refusedSection` (`NearbyViewControllerTests.testRefusedCaptureEndsReceivingWithTheErrorCode`);
  reference client error classes `needsRepair`/`needsNewCapture`/`isRetryable` and CLI exit codes
  (`NearbyReferenceClientTests.testImageRefusalsFromTheRealListenerAreTerminal`, `testDoctorReportsExactCodesAgainstNearbyState`).
- Uncovered: no permission field on `PairRecord` (grep `permission|viewOnly|view_only|capturesAllowed`: only POSIX
  file modes); no `capture_not_permitted` code on either side (grep: 0 hits); no per-device control in
  `NearbyView.pairedSection`; `PanelFocusOrder` "Nearby Companion" has no such control; no client mapping.

Conclusion: both follow-ups are entirely uncovered; implementing both.

## Checkpoint

- Branch `agent/mac-pairing-ui-3/qr-permissions` @ 82749c26 (audit commit pending). Dirty: this file, agents json.
- Next commands: implement `PairingQR.swift`, `PairRecord.permission` + store v3, listener check, client `--qr` + mapping,
  `NearbyView` QR/copy/permission controls, tests; `swift build`; run new test classes.
- Consumed main: 486b759c. Billing: shared Claude Max quota via parent; no purchases.
