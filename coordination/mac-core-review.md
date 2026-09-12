# mac-core-review — adversarial review of the Mac shell edit → durable → preview pipeline

Lane: `mac-core-review` (Claude Code subagent, Fable; parent `mac-claude-a`, mac-m1max-a).
Scope: read-only review of the parent-retained core plus minimal, test-covered fixes for
reproduced defects. Never edits `crates/*`; helper defects are reported with reproductions.

## Durable checkpoint

- Branch: `agent/mac-core-review/pipeline`; pass 1 from `origin/agent/mac-claude-a/mac-shell`
  @ 6fb77efd (three defect commits, integrated by the parent at 3162b89a); pass 2 on top of
  3162b89a, pushed through 59a3c78d (three more defect commits, see "Pass 2").
- Worktree: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a07346630f0c4534d`
- Dirty files at this checkpoint: this file and `coordination/agents/mac-core-review.json` only.
- Helpers built (release): compiler, preview-controller, pdf, bridge, edit-ledger,
  project-files, assistant-context under `crates/<c>/target/release/`.
- Test command: `swift test` in `apps/mac` with `FLASHTEX_NO_ACTIVATE=1` and the seven
  `FLASHTEX_*` helper paths exported (runner kept in the session scratchpad).
- Full-suite run started at 1-min load 5.5 (other lanes running); result recorded below.

## Fixed (test + fix, one commit each)

| # | Defect | Reproduction | Test | Fix |
|---|--------|--------------|------|-----|
| 1 | `failed`/`cancelled` helper updates never released the in-flight edit (default hold-until-preview policy): after a compiler crash the edit stayed in flight, every keystroke queued forever, nothing durable until relaunch. The existing `stale`/`discarded` guard also compares the helper's global compile *generation* with the document's durable revision, and `discarded` frames carry no `compile_revision` at all. | Real helper + compiler wrapper that dies on a marked request → helper emits `update {kind:"failed", reason:"compiler output closed"}`; `inFlight` stuck, durable stuck at r2. | `ControllerPipelineReviewTests.testCompilerFailureReleasesTheInFlightEditSoTypingStaysDurable` | fc89836b — release a durable in-flight edit on `failed`/`cancelled`, surface the reason. |
| 2 | ⌘S with the helper attached for a file not named `main.tex` exported the helper's *session temp copy*, reported "Saved paper.tex", cleared the dirty flag; the real file was never written (data loss). `saveTexInteractive` gated on `controllerAttached` instead of `controllerRoutesFiles`. | Real helper; open `paper.tex`, edit, `saveTexInteractive()` → disk unchanged, `isDirty == false`. | `…testSaveOfAFileNotNamedMainTexWritesThatFileNotTheHelpersSessionCopy` | a7fea28f — use `controllerRoutesFiles`; falls back to the direct SHA-guarded write. |
| 3 | `ProjectDocuments` `unowned model` read after awaits (`flushToHelper`/`awaitInFlight` polling ≤10 s, `syncWithHelper`, `openDiscoveredIncludes`, `helperRequest` timeout) from Tasks that hold ProjectDocuments strongly (`switchDocument`, `armControllerTracking`, `init`): a model torn down mid-wait → fatal unowned read (SIGABRT, kills the test process). | Fake helper; in-flight slot held; `Task { flushToHelper }`; release the model → `Fatal error: Attempted to read an unowned reference…`. | `…testProjectDocumentsFlushSurvivesTheModelBeingReleasedMidWait` | d1828e6a — strong local for the call's duration (as `DocumentKinds.refresh`); timeout block captures weakly. |

## Pass 2 (after merging origin/agent/mac-claude-a/mac-shell 3162b89a)

| # | Defect | Reproduction | Test | Fix |
|---|--------|--------------|------|-----|
| 8 | A capture approved during an IME composition was applied at once with a range computed against the model text (behind the storage by the marked run): it landed inside the composed run and the commit duplicated the composition (`かなかなAB`). An out-of-range edit was still reported through `onEditApplied`, so `appliedCaptureIDs` recorded a never-inserted capture. Also found: pending-edit tokens restarted at 1 whenever `pendingEdit` was nil, colliding with the token the view last consumed, so a second capture through the hosted editor was silently skipped. | IMEHarness + real helper: pin after "AB", compose "かな" at 0, approve. | `…testCaptureApprovedDuringCompositionIsNotMisplacedOrSilentlyLost`; `SourceEditorViewTests.testPendingEditOutsideTheBufferIsRefusedWithoutChangingText` (rewritten from "reported") | e32852b7 — wait for the composition; `PendingEdit.revision` + `onEditRefused`; `ShellModel.editRefused` re-queues the capture with its pre-insertion anchor; monotonic `nextEditToken()`. |
| 9 | Worker `error` envelope for the latest request dropped the coalesced keystroke's recompile. | Real compiler + `handleForTesting(.error)` while a second edit is queued: preview stays stale. | `…testWorkerErrorForTheLatestRequestStillCompilesTheQueuedBuffer` | e0cd128d — recompile once when the errored request was the latest and a buffer is queued. |
| 6 | Completion query outstanding at helper exit swallowed the relaunched helper's `document` reply (ids restart at `pc-1`); durable never learned, typing never durable. | Real helper; inject an outstanding query for `pc-2`; SIGKILL the launched pid; auto-relaunch. Fails without the fix (verified by reverting). | `…testCompletionQueryOutstandingAtHelperExitDoesNotSwallowTheRelaunchedHelpersReplies` | 59a3c78d — `completionFetcher.discard()` on exit and detach. |
| 5 | `controllerSave`/`controllerFileStatus` continuations dropped by an explicit detach (hung Task). | Real helper SIGSTOPped, `controllerFileStatus` awaiting, `detachController()`. Fails without the fix (verified). | `…testExplicitDetachResumesAwaitingFileStatusAndSave` | 59a3c78d — detach resumes `awaiting` waiters with "helper detached". |

Not done in pass 2 (time-boxed by the quota note): #7 (search reconcile overwrite window) —
reproduction as below; #4 relayed to the Commander by the parent; #10 unchanged.

## Reported only (not fixed; reproductions)

4. `stale` release guard compares units that do not correspond: `compile_revision` is the
   helper's compile generation (crates/preview-controller/src/lib.rs `compile_revision()` =
   `generation`, counts every admitted compile incl. `compile`/`configure_layout`), the guard's
   `want` is the document's durable revision (persisted ledger; can be far larger). Under
   hybrid, a stale for an OLDER compile can release the in-flight edit early when
   generation ≥ durable revision (fresh ledger), and never release it when the ledger's
   revision is high (a later preview then releases). Policy-timing only, no stall found;
   the edit reply does not name its compile request id, so exact correlation needs a
   helper-side field (`compile_request_id` on the `edit` result) — helper owners.
5. `controllerSave` / `controllerFileStatus` register `awaiting` continuations without a
   timeout; `detachController()` resets `controllerState` without resuming them
   (`.exited` fails them first; an explicit detach does not). Only reachable today from
   `attachController` (init/relaunch) and tests; would hang the save Task forever.
   Reproduction: attach real helper, start `controllerSave()` then `detachController()`
   before the export reply → the await never returns.
6. `completionFetcher` is not reset on helper exit/relaunch; request ids restart at `pc-1`
   on the new client. An outstanding completion id from the old session that collides with
   a new `edit`/`document` id makes `handleController` return `.refused` before
   `applyDurableDocument`, swallowing the durable reply (pipeline stalls until the next
   preview). Needs the relaunched helper's first edit to fail to preview; narrow.
7. `ProjectSearchPanel.reconcile` replaces the active buffer with the helper's post-apply
   text after an await; keystrokes typed in the editor during the `apply_group` round trip
   (ms window; the buffer-equals-durable check is before the send) are overwritten.
8. `SourceEditorView.updateNSView` applies a `pendingEdit` before the `hasMarkedText()`
   guard: a capture approved during an IME composition uses a UTF-16 range computed from
   the model text (behind the storage by the marked text), and an out-of-range edit is
   still reported through `onEditApplied` so `appliedCaptureIDs` marks a capture inserted
   that was never inserted (duplicate refusal on retry). Reproduction outline: IMEHarness
   with marked text before the anchor, then `approveProposal`.
9. Worker path: an `error` envelope for the latest request clears `compileQueued`, so a
   keystroke coalesced behind that request is not recompiled until the next keystroke
   (`ShellModel.handle(.error)`); the direct compiler never emits `error` for well-formed
   requests, so not reproduced with a real helper.
10. `TypingBenchDriver` has the same `unowned model` + Timer pattern (`pollReady`); the
    driver is retained by `TypingBench.shared` while the model is app-lifetime, so not
    reproducible in the app; tests keep the driver local.

## Verification (pass 2)

Every pass-2 test was verified to FAIL with its fix reverted (`git checkout` of the source
files, run, `git apply` back) and to pass with it. Full `swift test` at 59a3c78d with the
seven helpers plus `FLASHTEX_PDF_EXACT`: 642 tests, 13 skipped, **1 failure** —
`EditHistoryTests.testHelperDetachMarksThePendingCommandUncertain…`, caused by the #5 fix's
failure wording ("helper detached" was classified as a refusal, not uncertain); fixed in
fc7a3841 by using the exit prefix ("helper exited (detached)"). Filtered rerun at fc7a3841:
EditHistory 12/12, ProjectSearch, review 7/7, PreviewController — 22/22. Final full-suite
rerun at fc7a3841: see the line appended below. `FLASHTEX_RENDER` was not built (the v2
parity test skips); the 13 skips are env-gated (RENDER, EXPLAIN ×2, DisplayCandidate ×3
wanting their own env, PREFS/EVIDENCE/NEARBY, one NSAccessibility-environment skip).

## Verification (pass 1)

Full `swift test` (apps/mac) at d1828e6a with FLASHTEX_NO_ACTIVATE=1 and the seven helper
paths exported: **601 tests, 13 skipped, 0 failures**, 174.7 s; load averages at finish
4.84 / 6.53 / 12.01 (started at ~5.5 1-min, other lanes running). All 13 skips are
env-gated (FLASHTEX_RENDER, FLASHTEX_EXPLAIN, FLASHTEX_PDF_EXACT ×5, FLASHTEX_PREFS_EVIDENCE,
FLASHTEX_EVIDENCE_DIR, FLASHTEX_NEARBY_*, one OverlayTests NSAccessibility-environment skip).
Baseline for comparison: 590 / 7 skips / 0 failures at c60abba (the parent's tip added
tests since; this lane added 3). Note: `flashtex-pdf-exact` is built at
`crates/pdf/target/release/flashtex-pdf-exact` but was not in the requested env list, so
the five ExportSession/ExactPDF tests were not exercised here.

Final full `swift test` at fc7a3841 (helpers + FLASHTEX_PDF_EXACT; FLASHTEX_RENDER unset): **642 tests, 13 skipped, 0 failures**, 181.3 s; load averages at finish 5.44 / 4.92 / 6.16 (14:58Z).
