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

## Plan
1. `PreviewControllerClient.swift`: no wire change needed (edit results are
   bridged as `[String: Any]`; `NSNull` for null) — only if a typed accessor helps.
2. `ShellModel+Controller.swift` (explicit exception for this lane): add
   `admitted: (requestID: String, compileRevision: Int)?` to `inFlight`; set it from
   the edit reply in `applyDurableDocument`; new `controllerUpdateReleasesInFlight(kind:payload:)`
   / `controllerPreviewReleasesInFlight(_:)` pure decision in a NEW file
   `ShellModel+AdmissionCorrelation.swift` so the diff in the retained file is minimal.
3. `Tests/FlashTeXMacTests/AdmissionCorrelationTests.swift`: (a)–(e) per brief.

## Checkpoint
- branch `agent/mac-admission-correlation/compile-id` @ 644fcc9e (merge of main 55bcf124)
- dirty: this file
- consumed main SHA: 55bcf124
- next: build helper+compiler in worktree (background), implement, test, commit, push, register
