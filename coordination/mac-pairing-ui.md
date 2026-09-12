# mac-pairing-ui handoff

- Updated UTC: 2026-09-12T08:58Z
- Agent / parent / machine alias: `mac-pairing-ui` (Claude Code subagent) /
  `mac-claude-a` / `mac-m1max-a`
- Task / acceptance gate / owned paths: lane "Native pairing state recovery and
  accessibility" plus follow-ups "Prevent stale reconnect state from overwriting
  current pairing" and "User-visible progress/cancel states with precise
  semantics" (issue #2 dispatch; no FT assignment file, so no `ack`). Owned:
  `apps/mac/Sources/FlashTeXMac/Pairing.swift`, `NearbyView.swift`,
  `apps/mac/Tests/FlashTeXMacTests/PairingTests.swift`, `NearbyViewTests.swift`,
  `docs/evidence/nearby-pairing-2026-09-12/`, this file and
  `coordination/agents/mac-pairing-ui.json`. Not touched: `NearbyListener.swift`,
  `NearbyProtocol.swift`, `NearbyState.swift` (mac-nearby-transport),
  `apps/mac/tools/nearby-client` (mac-nearby-client), parent-retained
  `ShellModel*.swift`, `ContentView.swift`, `PreviewView.swift`,
  `FlashTeXMacApp.swift`, transferred crates.
- Branch / code revision / main integrated through:
  `agent/mac-pairing-ui/recovery` from `origin/agent/mac-claude-a/mac-shell`
  6b43a3a, merged up to 71675cd, then `origin/main` 254f662 merged (worktree
  `.claude/worktrees/agent-a4c2724311989ec2b`).
- State: ready for integration (lane + both follow-ups implemented and tested;
  the incomplete list names what still needs transport-owner APIs)

## Ready behavior and evidence

- `PairingFlow` (Pairing.swift): explicit pairing state machine — phases `off`,
  `advertising`, `codeShown`, `verifying`, `paired`, `receiving(n/m bytes)`,
  `interrupted(relaunch | peerGone | transportStopped)`, `failed(reason)`;
  inputs carry an attempt `generation` (monotonic, journaled) and/or `pairId`;
  anything older than the current attempt or for another pairing is reported
  as `stale` and never changes state. Pure: `Machine.apply` returns effects
  (`persist`, `clearJournal`, `cancelTransport`, `resumeTransport`, `announce`).
- `PairingJournal` (`~/Library/Application Support/FlashTeX/pairing-session.json`,
  schema version 1, mode 0600, dir 0700, atomic): generation counter plus the
  one pending attempt (code, pair_id, started/expires). Never holds a PSK.
  Unknown version → refused without overwrite. `FLASHTEX_PAIRING_JOURNAL`
  overrides the path.
- `PairStore`: explicit `schemaVersion` (1), `decode()`/`upgrade()` hooks,
  `loadOutcome` evidence (`created | loaded | upgraded | refused`); a file from a
  newer build is refused and never overwritten; records without a 32-byte key
  or a 16-byte salt are refused. `FLASHTEX_PAIR_STORE` overrides the path.
  `PairRecord` `description`/`debugDescription` redact the PSK.
- `PairingFlowController` (NearbyView.swift): drives the machine from
  `NearbyState`'s published values (`isAdvertising`, `pairingCode`, `pairs`,
  `status`, `lastReceivedCaptureId`) and runs effects: journal writes, cancel
  via `NearbyState.cancelPairing()` (or `coordinator.cancel()` + restart for a
  resumed attempt), resume via `coordinator.begin(code:lifetime:)` +
  `startAdvertising()`, expiry timer, VoiceOver announcements
  (`AccessibilityNotification.Announcement`). One controller per `NearbyState`
  (survives window close/reopen). `observe(_ event: NearbyListener.Event)` and
  `observeProgress(...)` accept the events the transport does not publish yet.
- Relaunch recovery: a code valid at quit is restored as `interrupted(relaunch)`
  with Resume (same code, remaining time, served again through the coordinator)
  or Cancel; an expired one is `interrupted` with Dismiss / Show New Code only.
- NearbyView: precise status row (title + sentence, `nearby.pairing.state`),
  code shown as digits with spoken-digit accessibility value
  (`nearby.pairing.code`), countdown with `.updatesFrequently`, Cancel (Esc) at
  code shown / verifying / interrupted, Resume (Return) when interrupted, Dismiss
  (Esc) on paired / error, Show Pairing Code (Return) when idle; focus moves
  with the phase; device rows combine name/pair id/connected/paired/seen into
  one label+value; capture rows list inbox (not durable) and bridge captures
  with their state; identifiers `nearby.*` for automation.

Validation (this worktree, 2026-09-12): `swift build` clean; `swift test
--filter "PairingFlowMachineTests|PairingPersistenceTests"` 36/36;
`swift test --filter NearbyViewControllerTests` 8/8 (real loopback TLS-PSK
NearbyState: show → pair → capture → forget; cancel refuses the bootstrap key;
relaunch restore + resume pairs with the same code; expired relaunch; resumed
code expiry drops the key; replaced code makes the old session stale end to
end; listener events → verifying/peer gone/receiving/error); evidence run
`FLASHTEX_NEARBY_SCREENSHOT_DIR=docs/evidence/nearby-pairing-2026-09-12 swift
test --filter NearbyViewScreenshotTests` → 8 PNGs (real `NearbyFlowView` hosted
in an NSWindow of the test process, `screencapture -x -o -l <windowNumber>`, no
activation, no Accessibility permission). Full suite after merging
`origin/agent/mac-claude-a/mac-shell` 71675cd: `swift test` with
`FLASHTEX_COMPILER/PDF/BRIDGE/EDIT_LEDGER` set to the release worker binaries →
251 tests, 0 failures, 2 skipped (this lane's opt-in screenshot test and a
pre-existing skip). One earlier run had `CompletionTests.
testCompletionOnOneMegabyteBufferIsFast` at 41 ms vs its 20 ms wall-clock limit
with load average 15 from other agents; it passes in isolation (0.7 s) and in
the repeated full run — a timing test outside this lane, not a regression.
After merging `origin/main` 254f662: `swift test` again 251 tests, 0 failures,
2 skipped. Smoke launch of `.build/debug/FlashTeXMac` with
`FLASHTEX_NO_ACTIVATE=1 FLASHTEX_AUTOATTACH=0 FLASHTEX_PAIR_STORE=<tmp>
FLASHTEX_PAIRING_JOURNAL=<tmp>` for 6 s: starts, creates the redirected
`pairs.json` (0600, version 1), no crash; the Nearby window itself cannot be
opened from outside without the requested app hook (Accessibility is not
granted), so window evidence comes from the test-hosted view.

## Incomplete behavior / blockers / needs from others

- `verifying`, `peerGone` during pairing, `receiving n/m bytes` and the
  `error <reason>` of a closed session are modelled and tested but not fed from
  the live transport: `NearbyState` publishes no per-connection events or byte
  progress. Needed from mac-nearby-transport (exact request): on `NearbyState`
  add `var onEvent: ((NearbyListener.Event) -> Void)?` invoked from
  `handle(_:)`, and a new `NearbyListener.Event.receiving(identity: String?,
  bytes: Int, expected: Int?)` emitted from `Connection.consume` when
  `splitter.pendingBytes` grows (expected stays nil unless the protocol gains a
  length hint). The controller's `observe(_:)` is the consumer.
- Resume after relaunch/advertising-off goes through
  `NearbyState.coordinator.begin(code:lifetime:)` + `startAdvertising()`, so
  `NearbyState.pairingCode` stays nil for a resumed attempt. Requested API:
  `NearbyState.beginPairing(resuming code: String, expiresAt: Date)` that sets
  its published code/expiry and timer like `beginPairing()`.
- Receiving is not cancellable from the Mac (no API to close one session).
  Requested: `NearbyState.closeConnection(pairId:)`.
- Stale-reconnect defence at the durable layer: `PairingCoordinator.confirmPairing`
  would accept a same-`pair_id` bootstrap session from a replaced pending code
  only if two codes collide (10^-6); a `generation` on `Pending` and on
  `PairRecord` (pairs.json v2 via `PairStore.upgrade`) would close that.
  Not done because the coordinator is transport-owned.
- Keychain storage: unchanged (still the 0600 file, per the proposal).
- Window automation hook: `FLASHTEX_OPEN_WINDOW=nearby` would let evidence be
  taken from the real app window; exact diff for `FlashTeXMacApp.swift` is in the
  final report (parent-retained; not applied).

## Interface changes / consumer actions

None to shared contracts. `NearbyView` now renders through `NearbyFlowView`
(`PairingFlowController`, `NearbyState`, `ShellModel`); the `NearbyView()`
entry point and its environment requirements are unchanged, so
`FlashTeXMacApp.swift` needs no change. `PairStore` API is source-compatible
(added `schemaVersion`, `loadOutcome`, `decode`, `upgrade`).

## Reviewed peer revisions / adaptations

`origin/agent/mac-claude-a/mac-shell` 6b43a3a (base; read `NearbyState.swift`,
`NearbyListener.swift`, `NearbyProtocol.swift`, `ShellModel+Nearby.swift`,
`apps/mac/docs/nearby-v1-proposal.md`): consumed the current APIs only.
`origin/main` not yet merged (checked `git fetch`; parent instruction is to
merge at clean checkpoints).

## Resources / deadline

Resource pool: parent `mac-claude-a`'s Claude Max 20x allowance on mac-m1max-a
(shared quota; no purchases, no overages). Allocation id
`claude-mac20x-pairing-ui`; maximum and confirmed spend unknown (subagent cannot
read plan usage). Deadline: none beyond the project's; user-stop-only
continuity. Context usage at this checkpoint: about 1.5% of the 15M-token
window (measured from the harness counter), far below the 60–80% compaction
thresholds in `docs/context-checkpoints.md`.

## Dirty files / running jobs / next action

Everything committed and pushed; no background jobs. Next: parent review;
if mac-nearby-transport adds `onEvent`/`receiving`/`closeConnection`/
`beginPairing(resuming:)`, wire them in `PairingFlowController` (consumer side
is already written and tested through `observe(_:)`).

## Resume reading list

AGENTS.md, this file, `apps/mac/docs/nearby-v1-proposal.md`,
`apps/mac/Sources/FlashTeXMac/Pairing.swift` (PairingFlow), `NearbyView.swift`
(PairingFlowController), the two test files.
