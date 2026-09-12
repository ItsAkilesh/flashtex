# mac-nearby-errors — capture-acceptance follow-ups (companion error classification, dropped-pin announcement, prepare state)

Lane: Claude Code subagent `mac-nearby-errors` (parent `mac-claude-a`, mac-m1max-a).
Branch: `agent/mac-nearby-errors/nearby-errors` from `origin/agent/mac-claude-a/mac-shell` @ 1630fdbc (contains main ffe199d).
Bounded ~75 min from 2026-09-12T09:57 local (uptime load 9.49 at start). Claude Max quota only; no provider calls.

## Coverage audit (mandatory first step, 09:57–10:10)

Sources read: `apps/mac/tools/nearby-client/Sources/NearbyClient/{NearbyWire,NearbyConnection,NearbyReconnector,NearbyCLI}.swift`,
`apps/mac/Sources/FlashTeXMac/{ShellModel+Nearby,ShellModel+Bridge,BridgeSession,NearbyProtocol}.swift`, `crates/bridge/src/lib.rs`
(`edit`, `capture_anchor`, `receive`; read only), `apps/mac/docs/nearby-v1-proposal.md` §4/§8, `coordination/mac-capture-acceptance.md`,
`docs/evidence/capture-acceptance-2026-09-12T1340Z.md`, and the tests below.

| Sub-gap | Already covered (file:test) | Verdict |
|---|---|---|
| (1) `destination_reselection_required` classified as "build a new capture" by the companion | `NearbyReceiveCapTests.testCodeClassification` (nearby-client package) asserts `needsNewCapture` only for `image_too_large`, `invalid_image`, `revision_mismatch`, `capture_id_conflict`, `unsupported_image`; `NearbyReferenceClientTests` lines 555/562/602/608 assert `needsNewCapture` for listener-side refusals only; `CaptureAcceptanceTests.testCaptureAgainstAChangedAnchorIsRefusedByTheRealBridgeAndNeverInserted` *measures* `needsNewCapture=false` for the bridge's `destination_reselection_required` (evidence `capture-acceptance-…T1340Z.md` line 30) and asserts only `isRetryable == false`. Doc §4 "Error codes used" lists listener codes only; §8 says bridge codes pass through verbatim but names none. | **Uncovered**: no test asserts the classification for any bridge-originated code (`destination_reselection_required`, `revision_conflict`, `instructions_too_large`, `invalid_id`), and the CLI has no hint for them. |
| (2) `hello_ack.destination` / `destination_query` after an edit overlapping the pin | `CaptureAcceptanceTests.testCaptureAgainstAChangedAnchorIsRefusedByTheRealBridgeAndNeverInserted` asserts the *finding* (`nearbyDestination == advertised` after the overlapping edit — the stale pin is still advertised); `BridgeRecoveryTests.testDestinationLostWhenSourceMovedPastThePinIsReported` (fake; relaunch path only); `BridgeSession.applicationApplied` drops the pin after an insertion (`ShellModelBridgeTests`). `NearbyReconnector.submit` already treats a changed/absent `hello_ack`/`destination_query` destination as terminal `destinationChanged` (`NearbyReconnectTests`). | **Uncovered**: the shell never mirrors the bridge's anchor rule on ordinary edits, so a companion is told a destination the bridge will refuse. Decision below: announce (never re-pin). |
| (3) `BridgeSession.prepare` on `proposal_missing` | `RealBridgeTests` line 81, `CaptureAcceptanceTests` lines 173/356 assert the error code only; no test asserts the row state afterwards (it becomes `.proposed` although no proposal exists). | **Uncovered** (cosmetic state). |

## Decision for (2): announce, never re-pin

Evidence: the bridge (`crates/bridge/src/lib.rs` `edit`) marks an anchor invalid when an edit overlaps it, sits exactly on it, or
inserts at a zero-width pin ("Insertion exactly at the target is ambiguous; invalidate instead of guessing affinity") and shifts
anchors that lie after the edit. It has no query op for anchor state and refuses captures at an invalid anchor with
`destination_reselection_required`. Re-pinning would require the shell to pick a new range on the user's behalf — exactly the
guess the bridge refuses to make — so the shell mirrors the bridge's rule locally (`DestinationTracking.follow`, same three
conditions), keeps the anchor with `valid = false` (ContentView already renders "(invalid)"), and `nearbyDestination` answers
`null` while a bridge is attached and its pin is not valid. The reference companion then reports `destinationChanged` from
`destination_query`/`hello_ack` before sending, and a client that skips that check gets the bridge's
`destination_reselection_required`, now classified `needsNewCapture`.

## Implementation (10:05–10:20 local, resumed after a user-side stop at ~14:03Z)

Files (all inside the lane; no parent-retained file touched):
- `apps/mac/Sources/FlashTeXMac/NearbyDestination.swift` (new): `DestinationTracking.follow` — the bridge's `edit` anchor rule
  copied condition for condition (insertion inside/at either end, overlap, deletion across a zero-width pin → `valid=false`;
  entirely before → shifted; after → untouched; `current_revision` follows; already-invalid anchors untouched, as the bridge
  filters on `valid`); `ShellModel.announcedNearbyDestination` — with a bridge attached only its valid pin is announced, else `nil`.
- `BridgeSession.swift`: `edited` calls `followDestination` (notes the drop, fires `onDestinationDropped`, keeps the row listed
  as invalid); `restoreDestination` drops an already-invalid pin on relaunch instead of re-pinning it; `prepare` maps
  `proposal_missing → .received`, `capture_rejected → .rejected`, `already_applied → .applied/.confirmed`,
  `destination_reselection_required → .needsReselection`, others `.proposed`.
- `ShellModel+Bridge.swift`: `onDestinationDropped` → `captureNote` ("…dropped by an edit…; pin again"); `submitCapture(image:)`
  and `submitSampleCapturePanel` refuse a dropped pin with that note instead of sending.
- `ShellModel+Nearby.swift`: `nearbyDestination` delegates to `announcedNearbyDestination`.
- nearby-client: `NearbyWire.captureInputErrorCodes` += `destination_reselection_required`, `revision_conflict`,
  `instructions_too_large`, `invalid_id`; new `destinationErrorCodes`; `NearbyError.needsNewDestination`;
  `.destinationChanged` now `needsNewCapture` (same conclusion as the bridge's refusal); CLI hints for the two destination codes.
- `apps/mac/docs/nearby-v1-proposal.md` §4: bridge pass-through code list and the announce-never-re-pin rule.

Tests:
- `Tests/FlashTeXMacTests/NearbyErrorsTests.swift` (new, 5): anchor-rule truth table; announcement table; fake-bridge end-to-end
  (before/after/overlap edits, user note, local submit refused, companion submit refused by the fake bridge with
  `destination_reselection_required`, fresh pin restores); `prepare` row state after `proposal_missing`/`capture_rejected`;
  real-bridge agreement (`FLASHTEX_BRIDGE`, skips otherwise).
- `CaptureAcceptanceTests.testCaptureAgainstAChangedAnchorIsRefusedByTheRealBridgeAndNeverInserted` rewritten to the announced
  behaviour: `nearbyDestination == nil`, `destination_query == nil`, reference client stops with `destinationChanged`
  (nothing reaches the listener/bridge), `requireCurrentDestination: false` reaches the real bridge → refused, classified
  `needsNewCapture && needsNewDestination`; mirror follows `current_revision`/bytes after the re-pin and a later edit.
- nearby-client `NearbyReceiveCapTests.testCodeClassification` extended for the bridge codes and `.destinationChanged`.

Measured (this Mac, 1-min load 14–65 during the runs — other agents active; no timing assertions of mine):
- `swift build --build-tests` clean (apps/mac).
- `swift test --filter "CaptureAcceptanceTests|NearbyErrorsTests|RealBridgeTests|ShellModelBridgeTests|BridgeRecoveryTests|NearbyReferenceClientTests|NearbyListenerTests|BridgeClientTests"`
  with real `FLASHTEX_BRIDGE`+`FLASHTEX_EDIT_LEDGER`: first run 58 executed, 2 skipped (preview-controller env, external
  client), 1 failure = the old finding assertion I then replaced; rerun of `CaptureAcceptanceTests|NearbyErrorsTests`: 9/9
  (1 env skip), 0 failures. Printed: `stale-anchor refusal code=destination_reselection_required needsNewCapture=true needsNewDestination=true`.
- nearby-client package `swift test`: first run 35 executed, 1 failure at 1-min load ≈33 (failure text not captured — the run
  was grep-filtered); three reruns 35/35, 35/35, 35/35 (last log: scratchpad `nearby-client-run3.log`). Not claimed flake-free.
- Full `swift test` NOT run: 1-min load stayed above the brief's 15 threshold.

Limitations: the mirror is a copy of the bridge's rule, not a query (transfer-v1 has no anchor-state op); if the bridge's rule
changes, `NearbyErrorsTests.testRealBridgeAgrees…` is the tripwire. Relaunch with an invalid pin is covered only by the
`restoreDestination` guard (no dedicated relaunch test). No parent-retained file diffs are needed.

## Durable checkpoint
- Task: brief `prompt-mac-nearby-errors.md`; branch `agent/mac-nearby-errors/nearby-errors`; worktree `.claude/worktrees/agent-af683ff540c120dbf`.
- Consumed: mac-shell 1630fdbc (main ffe199d). Audit commit 317c2092 pushed.
- Dirty files: none after the product + coord commits below (see `git log`).
- Next: parent reviews/merges; optional follow-up: a BridgeRecoveryTests case for relaunch with an invalid pin.
