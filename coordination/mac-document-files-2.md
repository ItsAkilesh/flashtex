# mac-document-files-2 handoff

- Updated UTC: 2026-09-12T16:36:09Z
- Agent / parent / machine alias: mac-document-files-2 (Claude Code subagent) /
  mac-claude-a / mac-m1max-a
- Task: Commander replenishment issue #2 comment 5646989044 item 11, "Document
  files: finish dirty-source preservation across external changes, reload,
  detach and reopen" (no FT number / assignment file, so no `coord.py ack`).
  Follow-up 1 (atomic save with exact SHA guard for non-entry members via the
  helper export route) and follow-up 2 (DispatchSource file watcher driving
  `refreshDiskStatus`) only if the current task finishes inside the bound.
- Owned paths: `apps/mac/Sources/FlashTeXMac/DocumentFiles.swift`,
  `DocumentFilesClient.swift`, `ProjectDocuments.swift` (previous owners
  finished, merged into mac-shell), new `DirtySnapshots.swift`,
  `Tests/FlashTeXMacTests/DocumentFilesTests.swift`, `ProjectDocumentsTests.swift`,
  new `DirtySnapshotsTests.swift`, this handoff and
  `coordination/agents/mac-document-files-2.json`. Parent-retained files
  (`ShellModel*.swift` core, `ContentView.swift`, `FlashTeXMacApp.swift`, …)
  get exact diffs in the final report only.

## Durable checkpoint

- Branch: `agent/mac-document-files-2/dirty-preserve` from
  `origin/agent/mac-claude-a/mac-shell` cd58fc2e (main c11c005 merged there).
  Worktree `.claude/worktrees/agent-ac41c19ffef1a7d1f`.
- Consumed main SHA: c11c005 (through the mac-shell base; no direct main merge yet).
- Tip: 14c0ac70 (3 commits: 7ab319a6 snapshots, b6703457 export guard,
  14c0ac70 watcher), pushed. Dirty files: none. State: ready for integration.
- Helper binaries for tests (built, read-only use, main checkout):
  `FLASHTEX_PROJECT_FILES=/Users/jay3332/Projects/flashtex/crates/project-files/target/release/flashtex-project-files`,
  `FLASHTEX_PREVIEW_CONTROLLER=/Users/jay3332/Projects/flashtex/crates/preview-controller/target/release/flashtex-preview-controller`,
  `FLASHTEX_COMPILER=/Users/jay3332/Projects/flashtex/crates/compiler/target/release/flashtex-compiler`.
- Next action: parent applies the FlashTeXMacApp.swift diff from the final
  report (Don't Save → `preserveDirtyBuffers`, File > Restore Unsaved
  Snapshot…) and merges the branch into mac-shell.
- Load at start: 1-min 7.98 (uptime 12:04).

## Coverage audit (≤10 min, done 12:20Z)

Already covered for the gap (file: test):

- External change while the buffer is dirty → conflict, nothing written,
  buffer kept, disk text readable for review:
  `DocumentFilesTests.swift: testHelperSaveRefusesExternalChangesUntilOverwriteOrReload`
  (real project-files helper, entry document), `testDirectFallbackIsReportedAndStillDetectsConflicts`
  (direct path), `DocumentFilesControllerTests.testControllerReloadIsReviewedPinnedAndImportedIntoTheDurableSource`
  (real preview controller, `file_status`), `ProjectDocumentsTests.swift:
  testDirectModeSavesNonEntryDocumentsAndRefusesChangedDisk`,
  `testHelperRouteSavesThroughExportAndFlushesOnSwitch` (non-entry members).
- Reload with a dirty buffer → explicit choice, discarded text recoverable
  *this session* (`recoverableBuffer`, single slot): `DocumentFilesTests:
  testReloadIsReviewedAndPinnedToTheReviewedSnapshot`, the controller test
  above (prior text stays in the ledger's undo history).
- Lost/late helper replies never lose the dirty buffer:
  `DocumentFilesTests: testHangingHelperKeepsTheDirtyBufferAndIsRestartedNextTime`,
  `testHelperExitingMidRequestKeepsTheDirtyBuffer`, `testLateSaveReceiptIsReconciledWithoutLosingLaterEdits`,
  `testGarbageReplyIsAProtocolFailureNotASave`.
- Project-files helper detach/respawn with a dirty buffer, open/reopen across
  roots, restore of a discarded buffer re-baselined on disk:
  `DocumentFilesTests: testMultiFileOpenDetachReopenAcrossProjectRoots`.
- Preview-controller detach + reattach (second model) resubmits the buffer:
  `PreviewControllerTests: testEditsBecomeDurableAndPreviewsBindToEditorRevisions`;
  crash relaunch keeps durable text: `testKilledHelperIsRelaunchedAndDurableTextSurvives`;
  members opened before the controller are synced: `ProjectDocumentsTests:
  testHelperSyncAttachesDocumentsOpenedBeforeTheController`.
- Non-entry detach with unsaved edits refused unless discarding; text kept in
  `detachedBuffers` (session only): `ProjectDocumentsTests: testDirectModeOpensIncludesSwitchesAndDetaches`.
- Fable-found case (file not named `main.tex` through the helper — save wrote
  the helper's session copy): `ControllerPipelineReviewTests:
  testSaveOfAFileNotNamedMainTexWritesThatFileNotTheHelpersSessionCopy` (save only).
- Quit prompt: `FlashTeXMacApp.applicationShouldTerminate` (parent-retained)
  lists every dirty member and saves each through `saveDocumentNow` + `saveTex`;
  the modal itself has no test (Accessibility not granted; mac-live report
  20260912T110944Z line 607 says so). "Don't Save" drops every dirty text.

NOT covered (this lane implements):

1. No durable copy of a discarded/dirty text when the preview controller is
   not attached (reload-discard, open-discard, non-entry detach, quit without
   saving): `recoverableBuffer`/`detachedBuffers` are process memory, one slot
   for the entry. Brief requires an app-support snapshot in that case.
2. Reopen the project → dirty snapshots offered: nothing exists.
3. Fable case for *dirty preservation*: a not-`main.tex` file with the
   controller attached goes through the direct reload path (`controllerRoutesFiles`
   false); no test that its discarded text becomes a durable snapshot and that
   a detach/reattach with a dirty buffer makes the text durable again in the
   same model.
4. Both texts durable at the moment of an external change while dirty (only
   in memory today).

## Delivered (all three items)

1. Current task — `apps/mac/Sources/FlashTeXMac/DirtySnapshots.swift` (+ hooks in
   `DocumentFiles.swift`, `ProjectDocuments.swift`), `Tests/…/DirtySnapshotsTests.swift`
   (4 + 1 controller tests). Durable per-file JSON snapshots under
   `~/Library/Application Support/FlashTeX/dirty-snapshots` (`FLASHTEX_DIRTY_SNAPSHOTS`
   overrides; XCTest processes use a per-pid temp dir) written on: external
   change while dirty (entry, not ledger-routed), refused save (entry and
   members), reload/open discard, member detach with unsaved edits, and
   `preserveDirtyBuffers(reason:)` (quit without saving; parent hook). Offered
   (`files.offeredSnapshots`, `captureNote`, `project.status`) on open of the
   entry or member; `restoreDirtySnapshot` explicit (buffer dirty against the
   current disk text, nothing written), `discardDirtySnapshot`,
   `restoreDirtySnapshotsInteractive()` modal; moot/saved snapshots removed.
2. Follow-up 1 — `ProjectDocuments.exportVerdict` (+ `MemberExportGuardTests`, 3 tests):
   export receipt must name the path and hash/byte-count to the text sent; a
   project-files refusal is a conflict with the `file_status` disk hash; any
   other error is settled by inspecting disk ("an error can follow rename").
3. Follow-up 2 — `DocumentWatcher.swift` (+ `DocumentWatcherTests`, 2 tests, one
   load-gated): vnode DispatchSource on the open document → `refreshDiskStatus()`;
   re-arms across atomic replace/delete; deferred while a file-layer request is
   outstanding (a status check would restart the helper and lose the late
   receipt — found by `testLateSaveReceiptIsReconciledWithoutLosingLaterEdits`
   before the fix). `FLASHTEX_NO_FILE_WATCH=1` opts out.

## Evidence

- `swift build` clean at 14c0ac70. Real helpers (read-only, main checkout):
  project-files, preview-controller, compiler, bridge, edit-ledger.
- DirtySnapshotsTests 4/4 + DirtySnapshotsControllerTests 1/1;
  MemberExportGuardTests 3/3; DocumentWatcherTests 2/2 at load ≤ 20 (the
  live test XCTSkips above 20; it skipped in later runs at load 30–80);
  DocumentFilesTests 10/10, DocumentFilesControllerTests 1/1,
  ProjectDocumentsTests 18/18, ControllerPipelineReviewTests 7/7,
  PreviewControllerTests, BridgeRecoveryTests, RealBridgeTests,
  ShellModelBridgeTests, ShellModelTests, EditHistoryTests green in the
  file-related run (`scratchpad/run3.log`: 82 tests, 3 failures, all below).
- Not run: full `swift test` (1-min load 30–80 throughout, brief gate < 15).
- Load-flaky, NOT this lane's regression (verified 4/4 failing with the base
  cd58fc2e sources restored in place, same message, watcher disabled too):
  `DocumentKindsTests/testUndeclareDetachesAndForgetsTheDeclaration`
  ("helper snapshot failed: helper exited (detached)" vs "no preview
  controller attached" after `detachController()`: a DocumentKinds `snapshot`
  request in flight at detach). `EditHistoryTests/testHelperDetachMarksThePendingCommandUncertainAndRetryConvergesAfterReattach`
  timed out (20 s) once at load 80, passed in isolation 2/3 at load 35.
- Test-process hygiene: the first run wrote 4 snapshots for temp files into
  the user's real `~/Library/Application Support/FlashTeX/dirty-snapshots`;
  removed, and the store now keeps XCTest processes in a per-pid temp dir.
