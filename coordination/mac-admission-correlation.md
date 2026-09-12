# mac-admission-correlation — exact compile correlation on the helper route

Lane: Claude Code subagent of parent `mac-claude-a` on mac-m1max-a (shared Claude
Max 20x quota; no purchases/overages). Branch
`agent/mac-admission-correlation/compile-id` from
`origin/agent/mac-claude-a/mac-shell` 3a3a6f21 merged with `origin/main` 55bcf124
(merge 644fcc9e). Bounded ~75 min.

## Coverage audit (mandatory first step, 2026-09-12T11:00Z)

Searched `apps/mac/Sources`, `apps/mac/Tests/FlashTeXMacTests/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`,
`coordination/*` for `compile_request_id` / `compileRequestId` and the
stale/discarded/failed release path.

Already covered (keep green, not this lane's work):
- `ControllerPipelineReviewTests.swift:testCompilerFailureReleasesTheInFlightEditSoTypingStaysDurable`
  — `failed` update releases an in-flight durable edit (real helper; no id check).
- `ControllerReleaseTests.swift:testHybridReleasesADurableEditAtTheBoundAndHoldDoesNot`
  and `testHybridWithHistoricalPaintsTheSupersededCompileAsHistory` — fake helper
  `%hold` → `stale {request_id, compile_revision}` (no admission pair on the edit
  reply); hybrid bound release; hold policy keeps the edit in flight.
- `OutputBoundsTests.swift:testHelperUpdateHookReleasesTheHeldEditAndKeepsTheStatus`
  (`stale` payload with only `compile_revision` passes through the output-bound hook
  as `false`) and `testRealHelperRefusesThe560KBPreviewWithOneFailedFrameAndTypingContinues`.
- `PreviewControllerTests.swift:testEditsBecomeDurableAndPreviewsBindToEditorRevisions`
  — real helper; previews bind by `source_versions` to editor revisions.
- Helper side (not shell): `crates/preview-controller/tests/stdio.rs:
  full_and_metadata_edit_admissions_match_wire_previews` proves the edit reply's
  `compile_request_id`/`compile_revision` equal the later `preview`'s
  `request_id`/`compile_revision`.
- `coordination/mac-core-review.md` finding 4 (reported only, not fixed): the
  `stale`/`discarded` guard compares the helper's compile generation with the
  document's durable revision.
- Report 20260912T110944Z.md: only the direct compiler's `compile_result`
  id/project/revision correlation rows (318, 489); nothing on the helper's admission pair.

NOT covered (this lane): no shell code reads `compile_request_id`; no
`ControllerState.inFlight` admission identity; `handleController(.update)` for
`stale`/`discarded` and `applyControllerPreview` still compare `compile_revision` /
`source_versions` numerically against the in-flight edit's DURABLE revision;
`failed` releases any durable in-flight edit regardless of `request_id`; no test
that a forged/mismatched `stale` id does not release; no test of the
`discarded`-with-`compile_revision` wire from main 55bcf12.

## What was added (ae0226a0)

- `apps/mac/Sources/FlashTeXMac/AdmissionCorrelation.swift` (new):
  `ControllerCompileAdmission {requestID, compileRevision?}` decoded from the
  edit reply (`from(editResult:)` — nil when either field is absent/null);
  `AdmissionCorrelation.decision(kind:requestID:compileRevision:previewVersion:admitted:durableRevision:)`
  — with a recorded pair, only the outcome (`preview`/`stale`/`discarded`/
  `failed`/`cancelled`) naming that request id (and the same generation when
  both carry one) releases; a mismatched id never releases; with no pair the
  previous numeric comparison is kept verbatim (stale/discarded generation >=
  durable, failed/cancelled any durable edit, preview compiled version of the
  in-flight path >= durable). `rebinding(...)` moves the wait to `by_id` on
  `superseded {request_id, by_id}` for the admitted compile. Two ShellModel
  helpers `controllerAdmissionReleases(kind:payload:)` / `(preview:)` log
  `controller release: …` / `controller hold: …` lines naming the ids.
- `ShellModel+Controller.swift` (lane exception): `inFlight` tuple gains
  `admitted: ControllerCompileAdmission?`; set from the edit reply next to
  `durableRevision`; the `stale`/`discarded` (+`superseded`), `failed`/`cancelled`
  and `applyControllerPreview` release sites call the decision. 7 hunks; the
  concurrent core-review lane's hunks (detach/exited) do not overlap.
- `ControllerPipelineReviewTests.swift:173`, `OutputBoundsTests.swift:87`: the
  positional tuple gains `, nil` (compile fix only).
- `Tests/FlashTeXMacTests/AdmissionCorrelationTests.swift` (9 tests): decision
  matrix (3), `superseded` rebinding, real helper (worktree build of main
  55bcf12) + real compiler through a sh wrapper (slow/dying markers):
  (a) reply pair non-null → its preview releases by id; (b) rapid edits with a
  slow compile + explicit `compile`: A released exactly once by its own
  `stale`/`discarded` carrying its compile_revision, the intermediate compile's
  outcome is a logged hold, B held until its own preview; (c) `failed` for the
  admitted id releases and surfaces `preview failed: …`; (e) forged
  stale/discarded/failed ids (generation past durable) never release, a
  same-id/other-generation stale is held; (d) fake helper (no pair) → admitted
  nil, numeric fallback holds below and releases at the durable revision.
  Real-helper tests XCTSkip without FLASHTEX_PREVIEW_CONTROLLER/FLASHTEX_COMPILER;
  (b) also skips when 1-min load > 20.

## Evidence
- `swift build` clean (apps/mac).
- `swift test --filter AdmissionCorrelationTests` with
  FLASHTEX_PREVIEW_CONTROLLER=<worktree>/crates/preview-controller/target/release/flashtex-preview-controller
  (built from 644fcc9e = main 55bcf124 merge) and FLASHTEX_COMPILER=<worktree>/crates/compiler/target/release/flashtex-compiler:
  9/9, 0 skipped, in 4 consecutive runs (4.3–4.7 s; load 6–10).
- Required filter `PreviewControllerTests|ControllerReleaseTests|ControllerPipelineReviewTests|EditHistoryTests|HistoricalPreviewTests|OutputBoundsTests|DisplayCandidateTests|AdmissionCorrelationTests`
  with the real helpers (pdf/bridge/edit-ledger/project-files/assistant-context from
  the main checkout's release builds): 63 tests, 5 skipped, 0 failures (23.6 s,
  load 9.5–10.5). The 5 skips are DisplayCandidateTests helper cases requiring
  FLASHTEX_RENDER (no flashtex-render binary exists on this Mac; pre-existing).
- Full `swift test` (apps/mac, same real-helper environment, FLASHTEX_NO_ACTIVATE=1)
  at ae0226a0: 647 tests, 19 skipped, 0 failures in 184 s; load 6.4 at start,
  5.5 at end (`uptime` recorded). Skips are the pre-existing env gates
  (FLASHTEX_RENDER, bundled fonts, capture/nearby hardware), none in this lane's file.

## Limitations
- The `superseded` rebinding is covered by the decision test and the log path of
  (b) only if the runtime reports it; in the observed runs the outcome was
  `stale` (the slow compile was already active), so the rebinding branch was not
  exercised end-to-end against the real helper.
- `outputBoundHandleControllerUpdate` (ShellModel+OutputBounds.swift) still
  releases the held edit on an oversized-reply `failed` regardless of id; out of
  this lane's scope and unchanged.
- `ProjectDocuments.awaitInFlight` keeps its own durable-text comparison for the
  switched-away path; unchanged.

## Checkpoint
- branch `agent/mac-admission-correlation/compile-id` @ ae0226a0 (pushed), base
  644fcc9e = mac-shell 3a3a6f21 + main 55bcf124
- dirty: none after this commit
- consumed main SHA: 55bcf124
- next: parent merges; nothing pending on this lane
