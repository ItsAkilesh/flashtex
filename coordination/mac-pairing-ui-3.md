# mac-pairing-ui-3 handoff

- Updated UTC: 2026-09-12T14:30Z
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

## Implementation (both follow-ups, 14:28Z)

Follow-up 1 — QR + copyable code:
- `apps/mac/Sources/FlashTeXMac/PairingQR.swift` (new): `PairingBootstrapPayload`
  (`flashtex-nearby://pair?v=1&code=…&salt=…&fp=…&name=…`, strict `parse`) and
  `PairingQR` (CoreImage `CIQRCodeGenerator`, level M, software renderer, nearest-neighbour scale).
- `apps/mac/tools/nearby-client/Sources/NearbyClient/NearbyBootstrap.swift` (new):
  `NearbyBootstrapPayload` — byte-identical string, same refusals; `NearbyCLI.pair --qr <text>`
  supplies code/fp/salt (explicit `--code`/`--mac` must agree; `resolveEndpoint(wantFP:)`).
- `NearbyView.swift`: `PairingFlowController.bootstrapPayload` (codeShown/verifying only),
  `copyCode(to:)` (bare digits, announces pairs); `codeRow` shows the QR image (`nearby.pairing.qr`)
  beside the code, `.onCopyCommand` (⌘C) on the focused code, "Copy code" button (`nearby.pairing.copy`).
- `Pairing.swift`: `spokenCode` now grouped in pairs ("1 2, 3 4, 5 6"); `clipboardCode`.
- `AccessibilityCommands.swift`: Nearby Companion description + `PanelFocusOrder` rows
  (code, Copy code, Permission pop-up) with source markers verified by `CommandTableTests`.

Follow-up 2 — per-companion permissions:
- `Pairing.swift`: `CompanionPermission` (`captures` | `view_only`, `refusalCode = "capture_not_permitted"`),
  `PairRecord.permission` (+`effectivePermission`), `PairStore.schemaVersion = 3` with `upgrade`
  writing explicit `captures` for v1/v2 records, `setPermission`, `capturesPermitted`;
  `PairingAccessibility.deviceRow` speaks the permission.
- `NearbyProtocol.swift`: `PairingConfirmer.capturesPermitted(pairId:)` (default true);
  `NearbySession` refuses `capture_submit` with `capture_not_permitted` before dedup memory, session open.
- `NearbyState.swift`: `PairingCoordinator.capturesPermitted` reads the store per capture;
  `NearbyState.setPermission` (persist + log, idempotent, no listener restart).
- `NearbyView.swift`: per-device `Picker("Permission")` menu (`nearby.device.<id>.permission`).
- Reference client: `NearbyWire.permissionErrorCodes`, `NearbyError.needsPermission`
  (needsRepair=false, needsNewCapture=false, not retryable), CLI exit 6 + hint.
- `apps/mac/docs/nearby-v1-proposal.md`: §2 v3 schema + QR bootstrap, error-code list.

Tests (measured 14:27–14:28Z, load 1-min 14→79 during other agents' builds; `swift build` + `swift build --build-tests` clean):
- `PairingQRTests` 4/4: pinned payload string equals the client's; CoreImage→Vision round trip;
  `testDecodedQRPairsTheReferenceClient` (real loopback listener, Vision-decoded QR passed verbatim
  to `nearby-client pair --qr`, long-term key handed over, disagreeing `--code` refused exit 64);
  copy to a private pasteboard + announcement in pairs.
- `CompanionPermissionTests` 3/3: v2→v3 upgrade/persist/reload; view-only refused with
  `capture_not_permitted` (exit 6, no re-pair hint, session open, inbox empty, no restart) then the
  identical capture accepted after the change; view-only survives relaunch against a fresh listener.
- `PairingFlowMachineTests` 32/32 and `PairingPersistenceTests` 7/7 (3 assertions updated to the
  paired spoken format / v3), `CommandTableTests` 8/8, `NearbyViewControllerTests` 13/13,
  `NearbyReferenceClientTests` 11 (1 skipped, pre-existing), nearby-client package 50/50
  (incl. `NearbyBootstrapPayloadTests` 3/3). Full `swift test` not run: 1-min load ≥ 15.

Parent-retained files: none touched (no hooks needed — the Nearby window is reached through
the existing `.nearbyCompanion` command).

## Checkpoint

- Branch `agent/mac-pairing-ui-3/qr-permissions`; implementation commit follows 706508d5; then merge
  `origin/agent/mac-claude-a/mac-shell` 9ba9851c forward and re-run the classes above.
- Consumed main: 486b759c (parent tip 9ba9851c). Billing: shared Claude Max quota via parent; no purchases.
