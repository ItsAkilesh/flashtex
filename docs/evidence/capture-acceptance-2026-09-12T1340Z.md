# Capture acceptance (Gap 7) — real-helper run, 2026-09-12T13:40Z

Lane mac-capture-acceptance (Claude Code subagent of mac-claude-a, mac-m1max-a, Xcode 26.3 / Swift 6.2.4).
Branch `agent/mac-capture-acceptance/capture-acceptance`; test file
`apps/mac/Tests/FlashTeXMacTests/CaptureAcceptanceTests.swift` (4 tests).
Helpers: in-worktree `cargo build --release` of crates/{bridge,edit-ledger,preview-controller,compiler}
at mac-shell 5bc3fc0f (zero external deps).
Command: `swift test --skip-build --filter CaptureAcceptanceTests` in `apps/mac` with `FLASHTEX_BRIDGE`,
`FLASHTEX_EDIT_LEDGER`, `FLASHTEX_PREVIEW_CONTROLLER`, `FLASHTEX_COMPILER` pointing at those binaries and
`FLASHTEX_NO_ACTIVATE=1`. `--enable-grok` is never passed: no provider, no network beyond loopback.

`uptime` at the run: `load averages: 10.94 29.27 25.05` (the parent's heavy-build window, 12 lanes
compiling) — the timings below are **under shared load, not an isolated result**.

Run 1 (load 9–10): 3/4 passed; the failing assertion was my own (a direct `capture_prepare_insert`
probe leaves the shell row `.proposed`, see finding 3). Assertion corrected; run 2: 4/4.
With the env vars pointing at non-executable paths: 4 skipped, 0 failures.
No helper process outlived the run (`pgrep -f "flashtex-bridge --store /var/folders"` empty afterwards).

## Sanitized transcript of run 2 (worktree path elided)

```
Test Suite 'CaptureAcceptanceTests' started at 2026-09-12 09:40:33.863.
Test Case '-[FlashTeXMacTests.CaptureAcceptanceTests testAppRelaunchBetweenReceiptAndInsertKeepsOneJournaledCaptureWithoutDuplicates]' passed (0.304 seconds).
Test Case '-[FlashTeXMacTests.CaptureAcceptanceTests testCaptureAgainstAChangedAnchorIsRefusedByTheRealBridgeAndNeverInserted]' passed (0.132 seconds).
Test Case '-[FlashTeXMacTests.CaptureAcceptanceTests testDropAfterDeliveryThenReconnectIsJournaledOnceByTheRealBridge]' passed (0.296 seconds).
Test Case '-[FlashTeXMacTests.CaptureAcceptanceTests testExplicitInsertRidesTheRealPreviewControllerAsOneDurableEdit]' passed (0.348 seconds).
Test Suite 'CaptureAcceptanceTests' passed at 2026-09-12 09:40:34.944.
	 Executed 4 tests, with 0 failures (0 unexpected) in 1.080 (1.081) seconds
measured: stale-anchor refusal code=destination_reselection_required needsNewCapture=false
measured: real-bridge drop→reconnect→duplicate-ack in 0.222s; attempts 2; 1-min load 10.939453125
measured: capture insert → durable+preview via flashtex-preview-controller in 32 ms; 1-min load 10.939453125
```

## What each test establishes (all against the real helpers)

| Sub-gap | Test | Observed |
|---|---|---|
| (a)+(b) | `testDropAfterDeliveryThenReconnectIsJournaledOnceByTheRealBridge` | Listener cut after the Mac forwarded to the real bridge (ack lost); companion reconnected with the same pairing (attempt 2) to a listener on the same port and re-sent the same `capture_id`; the second session delivered it again (fresh per-session dedup) and the bridge journal answered with the original durable receipt: 1 journal file, 1 `bridgeCaptures` row (`.received`), inbox empty, `capture_status` no proposal/prepared/applied. No `pendingEdit`, no proposal, buffer and durable ledger document untouched; convert → `provider_disabled`, prepare → `proposal_missing`. |
| (e) | `testAppRelaunchBetweenReceiptAndInsertKeepsOneJournaledCaptureWithoutDuplicates` | Bridge detached (process gone, store kept); a fresh `ShellModel` attached to the same store: `bridgeCaptures` empty (finding 1), `capture_status` answers; resend before a re-pin is refused client-side (`destinationChanged`, nothing sent); after re-pinning the identical destination the resend returns the identical receipt, still 1 journal file / 1 row; a conflicting resend → `capture_id_conflict`, capture stays usable. Nothing inserted. |
| (d) | `testCaptureAgainstAChangedAnchorIsRefusedByTheRealBridgeAndNeverInserted` | Edit overlapping the pin (`document_edit` to the real bridge) → capture bound to the advertised destination refused by the real bridge with `destination_reselection_required` (terminal, not journaled, row `.failed`, `capture_missing` on status, listener `captureRefused`); a fresh pin accepts a new capture; an edit after the anchor keeps it valid (accepted) and prepare is still `proposal_missing`. Buffer and durable document unchanged throughout. |
| (c) helper route | `testExplicitInsertRidesTheRealPreviewControllerAsOneDurableEdit` | Sample proposal reviewed: no staged edit, no durable revision until the explicit approval; approval stages one edit at byte 17; the editor's adoption produced exactly one new durable revision (text with the LaTeX once, SHA matches), compiled into the preview; duplicate approval `.duplicate`; the ⌘Z-equivalent revert became the next durable revision; tombstone kept. |

## Findings (reported, not patched — product files are parent/Commander-owned)

1. transfer-v1 has no capture-listing request: after an app relaunch the shell's `bridgeCaptures` is
   empty although the journal answers `capture_status`; the companion's resend restores the row
   (idempotent). The user sees nothing pending until then.
2. `ShellModel.bridgeDestination` (hence `hello_ack.destination`) is not refreshed after edits that
   overlap or shift the pin; the real bridge is authoritative (`destination_reselection_required`), so
   nothing is inserted, but the companion's client-side destination check passes and the refusal only
   arrives from the bridge. `NearbyWire.captureInputErrorCodes` does not list that code, so
   `NearbyError.needsNewCapture` is false for it (the CLI would not phrase it as "capture again").
3. `BridgeSession.prepare` marks a row `.proposed` on `proposal_missing` (cosmetic; the product only
   prepares after a proposal exists).
4. A reviewed insert through the real bridge + real edit ledger cannot be exercised without a provider
   (`capture_convert` is `provider_disabled` by design); that path stays covered by
   `RealEditLedgerTests.testShellFlowWithRealHelperAndFakeBridge` (real ledger, fake bridge proposal).
