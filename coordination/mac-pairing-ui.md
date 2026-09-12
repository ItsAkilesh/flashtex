# mac-pairing-ui handoff

- Updated UTC: 2026-09-12T10:05Z
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
- Branch / code revision / main integrated through: refill task on
  `agent/mac-pairing-ui/events-consumer` from `origin/agent/mac-claude-a/mac-shell`
  40d53b7 (which already carries the first lane branch
  `agent/mac-pairing-ui/recovery` fbc3832 integrated by the parent, plus
  mac-nearby-transport's `onEvent`/`receiving`/`closeConnection`/
  `beginPairing(resuming:expiresAt:generation:)` and pairs.json v2). Worktree
  `.claude/worktrees/agent-a4c2724311989ec2b`.
- State: ready for integration (refill task done: live events wired, receiving
  cancel, resume through `beginPairing(resuming:)`, persisted-generation stale
  defence, real-app window evidence)

## Refill task (events consumer) — what changed on this branch

- `PairingFlowController` subscribes to `NearbyState.onEvent` (chained after
  any earlier consumer) and drives the machine from the real listener events:
  `.connectionOpened` → verifying, `.hello(bootstrap:false)` → back to code
  shown, `.hello(bootstrap:true)` → paired with the generation persisted on the
  record (`PairRecord.generation`, pairs.json v2; a stored generation from
  another attempt is stale), `.connectionClosed` → peer gone / receive failed,
  `.receiving(identity:bytes:expected:)` → `receiving n bytes` (total unknown
  by design: JSON Lines carry no length hint), `.capture`/`.captureDuplicate`
  → received, `.captureRefused` → back with the error code, `.failed` → error.
  The placeholder `observeProgress` and the `$status`/`$lastReceivedCaptureId`
  string subscriptions are gone (`listener failed:` from a `start()` throw has
  no event and is still read from `status`).
- Generations: `showCode()` issues the code through
  `beginPairing(resuming: Pairing.generateCode(), expiresAt:, generation:
  journal.nextGeneration())`, so the transport's `Pending.generation`, the
  stored record and the journal agree. Resume uses the same API with the
  journaled code/expiry/generation, so `NearbyState.pairingCode` is set for a
  resumed attempt and the transport runs its own expiry timer.
- Receiving is cancellable: Cancel (Esc) → `NearbyState.closeConnection(pairId:)`;
  the flow shows "Receive from X cancelled after n bytes; the companion can
  resend it with the same capture_id." and the `.connectionClosed` that follows
  does not overwrite it. A resend replaces the notice with live progress.
- Refused-capture section (parent's addition) got accessibility label/value and
  identifiers `nearby.refused.*`.
- `FLASHTEX_NEARBY_AUTOSTART=code` (NearbyView) shows a code as soon as the
  window opens, for evidence runs together with the parent's
  `FLASHTEX_OPEN_WINDOW=nearby`.
- Tests: `PairingFlowMachineTests` 30 (cancel-in-receiving, refused capture,
  resend after cancel added), `PairingPersistenceTests` 7,
  `NearbyViewControllerTests` 11 against the real loopback transport: live
  verifying/paired with persisted generation; relaunch → resume through
  `beginPairing(resuming:)` (published code, same generation) → pairs; bootstrap
  session with a bad proof → `interrupted(peerGone, "pair mismatch")` → resume
  without a listener restart → pairs; an already paired companion connecting
  during a code is not the pairing peer; a ghost record with an older stored
  generation reconnecting is stale; large capture trickled in 16 KiB chunks →
  `receiving` with growing bytes → received; cancel mid-line → session closed by
  the Mac, notice kept, pairing kept, capture never lands; a refused capture
  (junk PNG) ends receiving with `unsupported_image`.
- Evidence (`docs/evidence/nearby-pairing-2026-09-12/nearby-app-{1..6}-*.png`):
  the real `.build/debug/FlashTeXMac` launched by `NearbyAppEvidenceTests`
  (opt-in `FLASHTEX_NEARBY_APP_EVIDENCE_DIR`) with `FLASHTEX_OPEN_WINDOW=nearby
  FLASHTEX_NO_ACTIVATE=1 FLASHTEX_NEARBY_AUTOSTART=code` and redirected
  store/journal; the test reads the code from the journal, finds the port with
  `lsof`, pairs over loopback as "Evidence iPad", trickles a 320×320 noise PNG
  capture, and captures the "Nearby Companion" window by id at code shown,
  verifying, paired, receiving (197 KB so far), received, disconnected. The app
  is never activated. `nearby-{1..8}-*.png` from the first lane are the same
  view hosted in a test window with synthetic inputs (interrupted/relaunch/error
  states that the loopback run does not reach).
- Full suite on this branch: `swift test` with the four release worker
  binaries → 385 tests, 0 failures, 12 skipped (this lane's opt-in evidence
  test and pre-existing skips), load average ~27 from other agents.

## Ready behavior and evidence (first lane, unchanged)

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

- Fresh codes are issued through `beginPairing(resuming:expiresAt:generation:)`
  so the transport uses the journal's generation; `NearbyState` therefore logs
  "pairing code resumed for …" for a fresh code. Cosmetic; a
  `beginPairing(generation:)` overload (or a neutral log line) on the transport
  side would remove it. A code minted by a direct `beginPairing()` call (no
  caller in the app does this) carries the coordinator's own counter; the
  controller then takes the next journal generation and the record's persisted
  generation would not match the window's — documented, not observed.
- `receiving` never shows a total (`n of m`): the transport reports
  `expected: nil` because JSON Lines carry no length hint. The machine, the
  status text and the progress bar already handle a total if the protocol ever
  adds one.
- Keychain storage: unchanged (still the 0600 file, per the proposal).

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

Everything committed and pushed; no background jobs (the evidence test
terminates the app it launches). Next: parent review and integration into
mac-shell.

## Resume reading list

AGENTS.md, this file, `apps/mac/docs/nearby-v1-proposal.md`,
`apps/mac/Sources/FlashTeXMac/Pairing.swift` (PairingFlow), `NearbyView.swift`
(PairingFlowController), the two test files.
