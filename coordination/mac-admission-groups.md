# mac-admission-groups handoff — grouped admission consumption + D2 first-generation gate

- Updated UTC: see `coordination/agents/mac-admission-groups.json` `updated_utc`
- Agent / parent / machine alias: `mac-admission-groups` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: two Commander-named follow-ups (issue #2 5646803033 / 5646646403):
  (1) native consumption of the `apply_group` compile admission pair
  (helper main c11c005 / 3ffd57f, STDIO.md "Edit admission correlation");
  (2) the D2 first-generation gate (learn the authoritative membership
  generation on helper `ready`; refuse candidates before it is learned).
- Branch: `agent/mac-admission-groups/groups-and-gate` from
  `origin/agent/mac-claude-a/mac-shell` cd58fc2e (contains main c11c005).
- Owned paths: `apps/mac/Tests/FlashTeXMacTests/AdmissionGroupsTests.swift`,
  `apps/mac/Sources/FlashTeXMac/ShellModel+DisplayCandidates.swift` (minimal),
  `apps/mac/Sources/FlashTeXMac/ProjectDocuments.swift` (membership forget hook),
  `controllerAdoptHistoryResult` in `ShellModel+Controller.swift` (the one
  permitted function), D2 assertions in `V2ConformanceTests.swift`, this
  handoff and `coordination/agents/mac-admission-groups.json`.

## Coverage audit (mandatory first step)

Grepped `apps/mac/Tests/FlashTeXMacTests/*`, `apps/mac/Sources/FlashTeXMac/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`
for `apply_group`, `compile_request_id`, `replayed_command`, `command_revision`,
`controllerAdoptHistoryResult`, `membershipGeneration`, `D2`, `first-generation`.

Already covered for gap (1), grouped admissions:
- `AdmissionCorrelationTests.swift`: `testEditReplyPairDecodesAndNullsAreNil`
  (pair decode incl. JSON null), `testAdmittedIdentityReleasesOnlyTheMatchingOutcome`,
  `testNumericFallbackWithoutAPairIsThePreviousBehaviour`,
  `testSupersededRebindsOnlyTheAdmittedCompile` (pure), and real-helper
  `testEditReplyCarriesTheAdmittedPairAndItsPreviewReleases`,
  `testRapidEditsReleaseExactlyOncePerAdmittedCompile`,
  `testFailedOutcomeOfTheAdmittedCompileReleasesAndSurfaces`,
  `testForgedMismatchedOutcomeDoesNotRelease`,
  `testNumericFallbackAgainstAHelperWithoutThePair` — all for `edit` replies only.
- `PasteRecoveryTests.testMultiRangeReplacementIsOneGroupWithExactText`: real
  `apply_group` adopted through `controllerAdoptHistoryResult`, waits for the
  preview and asserts `replayed_command:false`/`command_revision:2`; it does NOT
  assert that the group's pair is recorded as the in-flight admission nor that
  the release is keyed on the group's request id.
- `ProjectSearchTests` (~741) / `CitationRenameTests` (~407): exact
  `apply_group` retry → `replayed_command:true`, changed payload refused — wire
  level only, no admission assertions.
- `EditHistoryTests.testEditUndoRedoRoundTripAdoptsExactDurableText`,
  `testIdenticalRetryAfterALostReplyIsIdempotent`, `testRetryAfterRelaunch…`:
  undo/redo adoption and replay; no admission assertions (undo/redo wire
  schemas carry no pair per STDIO.md, so the numeric fallback applies).
- No test in the tree reads `compile_request_id` from an `apply_group` reply.
  The mac-live report and docs/evidence carry no grouped-admission evidence.

Structural finding for (1): `controllerAdoptHistoryResult` → `applyDurableDocument`
already reads the top-level `compile_request_id`/`compile_revision` of the
`apply_group` reply (the helper puts them at the top level, main.rs
"apply_group" arm), so the pair IS recorded as `inFlight.admitted` when the
buffer adopts the text and nothing else is in flight; `history.command_revision`
is decoded only by `EditHistory.Result` (display) and never written into
`controllerState.durable`. Uncovered: tests proving it, and the retry/failure
semantics. Product change for (1): none needed beyond a doc comment.

Already covered for gap (2), D2:
- `V2ConformanceTests.testGateRefusesACandidateWhoseMembershipGenerationIsNotTheProjectsCurrentOne`
  (pure gate: nil generation SKIPS the check — the behaviour this lane changes),
  `testRealHelperRefusesACandidateFromBeforeAnOpenDocument` (real helper +
  flashtex-render: open_document between compile and candidate → refused; it
  learns the generation via an explicit `refreshSnapshot()` after the first paint).
- `DisplayCandidateTests.testGateRefusesStaleForeignAndToggledIdentities` (gate
  without a generation), `testHelperCandidatesPaintAndBindToEditorRevisions`
  (forged/foreign frames use `membershipGeneration: 1`).
- Uncovered: learning the generation on `ready` (no snapshot is requested at
  attach today — `ProjectDocuments` is created lazily and only snapshots for a
  member sync), refusing a candidate that arrives before a generation is known,
  forgetting it on detach/exit.

## Checkpoint (refresh before long steps)

- Branch/SHA: see the block at the end of this file.
- Consumed main: c11c005 (via mac-shell cd58fc2e).
- Next commands: `swift build`, `swift test --filter "AdmissionCorrelationTests|AdmissionGroupsTests|DisplayCandidateTests|EditHistoryTests|ProjectSearch|CitationRename|SearchReconcileTests|V2ConformanceTests"` with FLASHTEX_PREVIEW_CONTROLLER/FLASHTEX_COMPILER from this tree.
- Staffing/billing: shared Claude Max quota with parent mac-claude-a; no purchases.

## What was added (branch `agent/mac-admission-groups/groups-and-gate`, tip 8496956f, pushed)

Commits (all `jay3332` primary author, lane trailers):
- e83ce12a — item (1) `AdmissionGroupsTests.swift` (4 real-helper tests). No
  product change: the pair is already consumed by
  `controllerAdoptHistoryResult` → `applyDurableDocument`; an optional
  doc-comment diff for that function is in the final report (parent-retained
  file, not committed here).
- 10d39fcf — item (2) first-generation gate: `displayCandidatesLearnMembershipGeneration`
  (snapshot sent synchronously with the opt-in on ready/toggle/restart,
  before the `document` request; reply routed via `controllerState.awaiting`
  into `ProjectDocuments.adoptSnapshot`), typed `membership_unknown` refusal
  while nil, `forgetMembership()` on `displayCandidatesInvalidate`,
  `FirstGenerationGateTests.swift`, V2Conformance D2 pure row updated.
- 8496956f — measured correction: the helper's `membership_generation` is the
  project-index generation (crates/project-index `VersionSnapshot.generation`),
  which advances on EVERY durable source update (`replace_source`) as well as
  on open/detach (`replace_membership`); `edit`/`apply_group` replies do not
  carry it. Strict equality against the ready-time value refused every
  candidate after the first edit (log: "membership generation 3 is not the
  project's current generation 2"). The gate therefore treats the learned
  value as a FLOOR: older → refused ("is older than the project's learned
  generation N"), equal/newer → the applied-identity checks; nil → typed
  `membership_unknown`. Tests updated accordingly (V2ConformanceTests D2
  rows, DisplayCandidateTests pure gates + the paint test's forged frames).

Measured facts (helper + compiler built from this tree at cd58fc2e; flashtex-render 9aaec57a):
- `apply_group` reply: top-level `compile_request_id`/`compile_revision`
  non-null pair; `history.command_revision` 2, `replayed_command` false.
- Exact retry after one more edit: `replayed_command` true, `command_revision`
  2, `history.document.revision` 3 with the CURRENT text, fresh pair
  preview-6/6 (original preview-4/4); adopted as in-flight for durable r3;
  the original id never releases it, its own outcome does.
- Undo/redo replies carry no `compile_request_id`/`compile_revision` keys at all.
- Ready-time snapshot: generation 2 for a one-document project on this helper;
  +1 per durable edit; the first candidate names the same generation.

Test evidence (load 6–8 unless noted):
- `AdmissionGroupsTests` 4/4; `FirstGenerationGateTests` 5/5 (incl. the
  flashtex-render paint case: learned line precedes the first admitted
  candidate; refused 0; no `membership_unknown` logged).
- Required filter at 8496956f (`AdmissionCorrelationTests|AdmissionGroupsTests|
  DisplayCandidateTests|EditHistoryTests|ProjectSearch|CitationRename|
  SearchReconcileTests|V2ConformanceTests|FirstGenerationGateTests|
  ProjectDocumentsTests|PasteRecoveryTests`): every suite 0 failures except
  one run during concurrent cargo builds (load 15–23) where
  `EditHistoryTests.testRetryAfterRelaunchReplaysAnUndoAppliedBeforeTheLostReply`
  timed out (20 s) and `DisplayCandidateTests.testHelperDisableRestartAndCloseRequireFreshOptIn`
  skipped on its 30 s bound; rerun after the builds: EditHistory 12/12,
  DisplayCandidate 12/12 (no skips), PasteRecovery 5/5.

## Checkpoint

- Branch `agent/mac-admission-groups/groups-and-gate` @ 8496956f (pushed);
  worktree `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ab69dca0ab51abb6f`.
- Dirty: coordination files only (this handoff, `coordination/agents/mac-admission-groups.json`).
- Consumed main: c11c005 via mac-shell cd58fc2e.
- Next: full `swift test` with every helper from this tree if 1-min load < 15; final report.
