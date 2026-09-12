# mac-document-files-2 handoff

- Updated UTC: 2026-09-12T12:20Z
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
- Dirty files: none yet (audit phase).
- Helper binaries for tests (built, read-only use, main checkout):
  `FLASHTEX_PROJECT_FILES=/Users/jay3332/Projects/flashtex/crates/project-files/target/release/flashtex-project-files`,
  `FLASHTEX_PREVIEW_CONTROLLER=/Users/jay3332/Projects/flashtex/crates/preview-controller/target/release/flashtex-preview-controller`,
  `FLASHTEX_COMPILER=/Users/jay3332/Projects/flashtex/crates/compiler/target/release/flashtex-compiler`.
- Next commands: implement `DirtySnapshots.swift` + tests; `cd apps/mac && swift build`;
  `swift test --filter 'DirtySnapshotsTests|DocumentFilesTests|ProjectDocumentsTests'` with the env above.
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
