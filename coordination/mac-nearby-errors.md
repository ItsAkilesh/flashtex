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

## Durable checkpoint
- Task: brief `prompt-mac-nearby-errors.md`; branch `agent/mac-nearby-errors/nearby-errors`; worktree `.claude/worktrees/agent-af683ff540c120dbf`.
- Consumed: mac-shell 1630fdbc (main ffe199d).
- Dirty files: this handoff (audit commit pending).
- Next: implement (1) NearbyWire + client tests + doc note, (2) `NearbyDestination.swift` + `BridgeSession.edited` hook + `ShellModel+Nearby.nearbyDestination`, (3) `prepare` state; `NearbyErrorsTests.swift`; update `CaptureAcceptanceTests` stale-anchor test to the announced behaviour.
- Helpers: building `crates/{bridge,edit-ledger}/target/release` in-worktree (cargo release, zero deps).
