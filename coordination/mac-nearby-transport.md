# mac-nearby-transport handoff

- Updated UTC: 2026-09-12T09:16Z
- Agent / parent / machine alias: mac-nearby-transport (Claude Code subagent) /
  mac-claude-a / mac-m1max-a
- Task / acceptance gate / owned paths: lane "Bounded nearby receive and
  off-main decoding" + follow-ups "Session/capture/revision dedup with explicit
  error state" and "Actual iPad or simulator transcript acceptance" (parent
  dispatch, issue #2 thread; no FT number / assignment file, so no ack) /
  full `swift test` in apps/mac green /
  `apps/mac/Sources/FlashTeXMac/{NearbyListener,NearbyProtocol,NearbyState}.swift`,
  `apps/mac/Tests/FlashTeXMacTests/NearbyListenerTests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/Fixtures/nearby-companion-session.jsonl`,
  `coordination/mac-nearby-transport.md`, `coordination/agents/mac-nearby-transport.json`
  (plus a documentation-only edit to `apps/mac/docs/nearby-v1-proposal.md` §4/§5)
- Branch / code revision / main integrated through:
  `agent/mac-nearby-transport/events` (from origin/agent/mac-claude-a/mac-shell
  d8baed6, which integrated the first lane at 1d76c2e) / see JSON /
  origin/main as contained in mac-shell d8baed6. Earlier branch
  `agent/mac-nearby-transport/bounded` (b3baa25) is merged.
- State: ready for integration (events follow-up requested by mac-pairing-ui)
- Ready behavior and evidence:
  - `NearbyReceiveLimits` (frame 12 MiB, image 8 MiB / 8192² / 64 MiB decoded,
    per-session in-flight 24 MiB, listener-wide in-flight 64 MiB, 4 sessions
    per pair, 16 connections, 256 remembered captures per session) on
    `NearbyListener.Configuration.limits`; `NearbyReceiveBudget` shared across
    listener restarts (`adoptConnections`).
  - Every cap is an explicit `error` reply (`too_many_in_flight`, `inbox_full`,
    `too_many_sessions`, `image_too_large`, `invalid_image`) except the
    unauthenticated connection cap, which closes before the handshake and
    emits `.connectionClosed(nil, "too many connections (N)")`.
  - Capture frames are budgeted on the listener queue, then JSON-decoded,
    field-checked and image-validated on a concurrent utility queue
    (`NearbyImageCheck`: PNG chunk walk with CRCs + streaming IDAT inflate with
    exact size and Adler-32; JPEG marker walk to SOF and EOI; MIME/container
    match; dimension and decoded-size caps). Only validated captures reach the
    sink, which hops to the main actor (`ShellModel+Nearby` unchanged).
  - Dedup per session by `(capture_id, base_revision, digest)`: identical retry
    acknowledged with the new id and not re-delivered (pending retries are
    coalesced), `revision_mismatch` for another revision, `capture_id_conflict`
    for another payload; sink refusals are forgotten so a retry re-delivers.
  - `NearbyState.lastReceiveError` / `receiveErrors` / `duplicateCaptureCount` /
    `lastDuplicateCaptureId` / `clearReceiveErrors()` plus log lines
    (`refused …`, `duplicate …`) from new listener events `.captureRefused` /
    `.captureDuplicate`.
  - Transcript acceptance: `Fixtures/nearby-companion-session.jsonl`
    (recorded-shape companion session, see test doc comment for provenance)
    replayed on loopback by `NearbyTranscriptAcceptanceTests`: hello_ack, four
    capture_received with duplicates absorbed, two inbox entries, verbatim
    replay refused at hello (stale nonce), reconnect retry acknowledged
    without re-storing.
  - Events follow-up (mac-pairing-ui requests 1–5): `NearbyState.onEvent`
    called first in `handle(_:)`; `NearbyListener.Event.receiving(identity:
    bytes:expected:)` from `Connection.consume` (interim reports at least
    64 KiB apart, one on completion of a reported/large line, never for small
    lines; `expected` always nil) mirrored in `NearbyState.receivingBytes`;
    `NearbyListener.closeConnections(identity:)` + `NearbyState.closeConnection
    (pairId:)` (session mid-line stops, budget released, pairing kept);
    `NearbyState.beginPairing(resuming:expiresAt:generation:)` (exact expiry,
    same timer; expired attempts refused with a log line);
    `PairingCoordinator.Pending.generation` carried on the bootstrap
    `PSKEntry.generation` and checked by `confirmPairing(pairId:companionName:
    generation:)` (`PairingConfirmer` signature changed), `lastRefusal`,
    `confirmedGeneration(pairId:)`.
- Incomplete behavior / blockers / needs from others:
  - pairs.json v2 (`PairRecord.generation`, `PairStore.schemaVersion = 2`,
    `upgrade`) lives in `Pairing.swift` (mac-pairing-ui); requested diff is in
    the final report. Until then the confirming generation is kept in memory
    on the coordinator only.
  - `NearbyView.swift` `observe(_:)` needed one exhaustive-switch arm for the
    new `.receiving` event; applied here as the minimal wiring to
    `observeProgress` (3 lines) so the build stays green — mac-pairing-ui to
    review.
  - `NearbyInbox` (parent-retained `ShellModel+Nearby.swift`) still bounds by
    count (50), not bytes; see suggested diff in the final report.
  - `NearbyView.swift` (mac-pairing-ui) does not yet show
    `lastReceiveError`/`duplicateCaptureCount`; suggested diff in the report.
  - JPEG entropy data is not decoded (structure only); the bridge's full decode
    remains authoritative.
  - The iPad simulator was not driven: the companion on every published
    branch still connects with plain `NWParameters.tcp` (no TLS-PSK, no v1
    hello), so a live run can only reproduce the handshake refusal already
    covered by `NearbyPlaintextTests`; the fixture is re-synthesized to the
    recorded shape because the original stdout log was never committed.
- Interface changes / consumer actions: wire version unchanged (nearby v1);
  new additive error codes documented in `nearby-v1-proposal.md` §4. The
  reference client (mac-nearby-client) should treat `too_many_in_flight` and
  `inbox_full` as "wait for outstanding acks, then retry same capture_id", and
  `too_many_sessions` as "close older connections, then reconnect".
  `NearbyListener.Configuration.maxLineBytes` is now a computed accessor over
  `limits`; the memberwise init takes `limits:` (test harness updated).
- Reviewed peer revisions / resulting adaptations: origin/agent/mac-claude-a/
  mac-shell 6b43a3a (base). Companion branches (aarush-macbook/companion-capture
  e7ce5b9, chatgpt-a/companion-reliability) still connect with plain
  `NWParameters.tcp`, so a simulator run cannot authenticate; the transcript
  follow-up will replay a recorded-shape transcript on loopback.
- Validation commands / results / artifact paths:
  `cd apps/mac && swift build` clean; `swift test --filter 'Nearby|Pairing'`
  79 tests pass (3 consecutive runs, 1 opt-in screenshot test skipped); full
  `swift test` with FLASHTEX_COMPILER/PDF/BRIDGE/EDIT_LEDGER/
  PREVIEW_CONTROLLER/PROJECT_FILES real binaries on mac-shell d8baed6:
  310 tests, 0 failures, 1 skipped (FLASHTEX_NEARBY_SCREENSHOT_DIR opt-in).
- Exact deadline UTC / remaining time / integration reserve: no fixed deadline
  (continuous authorization); 20% reserve kept for integration.
- ETA remaining: 0 / 0 / 0 (lane and follow-ups done; awaiting review).
- Resource pool / allocation ID / maximum: Claude Max 20x on mac-m1max-a
  (shared account quota) / claude-mac20x-nearby-transport / parent's allowance;
  quota not visible to the subagent.
- Confirmed spend / estimated usage / in-flight reservation / remaining: unknown.
- Billing evidence / freshness / unknowns: none available to a subagent.
- Child tasks and their deducted allocations: none.
- Dirty files / unpushed work / running jobs: none after this checkpoint.
- Decisions / failed approaches / linked findings: ImageIO
  (`CGImageSourceCreateThumbnailAtIndex`, incremental sources) accepts
  truncated/corrupt PNG and JPEG without an error status, so validation is a
  hand-written structural check (probe in scratchpad, not committed).
  Cancelling an accepted `NWConnection` before `start()` does release it (the
  client sees a reset in `.waiting`, never `.ready`).
- Exact next action or command: none pending; parent to review/integrate
  `agent/mac-nearby-transport/events`; mac-pairing-ui to apply the pairs.json
  v2 diff and review the `observe(_:)` arm.
- Resume reading list: this file, `apps/mac/docs/nearby-v1-proposal.md` §4–5,
  `docs/evidence/companion-simulator/README.md` §5 (the recorded companion
  output shape).
- Context checkpoint (docs/context-checkpoints.md): worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ac78436e7821c1d3e`,
  branch `agent/mac-nearby-transport/events`, HEAD = the commit carrying this
  file, base d8baed6. Context usage at this checkpoint is roughly a quarter of
  the 1M window (counted from the session's token budget; no exact percentage
  is exposed to the subagent). Rules carried: no purchases/overages,
  transferred crates (font-engine, paragraph-layout, math-layout) untouched,
  parent-retained Mac files untouched, user is the primary Git author on this
  machine, truthful trailers on every commit.
