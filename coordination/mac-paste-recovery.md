# mac-paste-recovery — large paste / multi-range group / kill-recovery / hold-until-preview

Lane: Claude Code subagent of parent `mac-claude-a` on `mac-m1max-a` (Claude Max 20x
quota shared with the parent; no purchases). Gap 2 of the parent's coverage brief.

## Durable checkpoint

- Branch `agent/mac-paste-recovery/paste`, worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a327390482be18e7c`,
  cut from `origin/agent/mac-claude-a/mac-shell` = `5bc3fc0f` (merge-base with
  `origin/main` `ffe199d8` is `527ae541`; the branch carries main dda0b62's
  preview-controller with compact edit acknowledgements).
- Consumed main SHA: `527ae541` (through the parent branch); no main files touched.
- Helper binaries built in this worktree at 5bc3fc0f: `crates/preview-controller/
  target/release/flashtex-preview-controller`, `crates/compiler/target/release/
  flashtex-compiler` (both `cargo build --release`, exit 0).
- Owned files: `apps/mac/Tests/FlashTeXMacTests/PasteRecoveryTests.swift`,
  `docs/evidence/mac-paste-recovery-2026-09-12.md`, this file,
  `coordination/agents/mac-paste-recovery.json`. No source file under
  `apps/mac/Sources` is changed; no crate is changed.
- Next commands: `swift build --package-path apps/mac`; `FLASHTEX_COMPILER=… FLASHTEX_PREVIEW_CONTROLLER=… swift test --package-path apps/mac --filter PasteRecoveryTests`.

## Coverage audit (≤10 min, done first)

Searched `apps/mac/Tests/FlashTeXMacTests/*`, the mac-live report
`tools/native-validation/mac-live/reports/20260912T110944Z.md` and `docs/evidence/*`
for paste, large-document, multi-range/`apply_group`, kill-by-pid and hold/queued
coverage of the durable preview-controller route. Already covered:

| Surface | Where | What it proves |
| --- | --- | --- |
| Edits durable, previews bound to editor revisions, 6-keystroke burst coalesced, ledger persistence across detach/attach (differing disk text resubmitted as a NEW revision) | `PreviewControllerTests.testEditsBecomeDurableAndPreviewsBindToEditorRevisions` | small (~50 B) edits only; no paste-sized text, no kill |
| Durable save/export exactness and disk-conflict refusal | `PreviewControllerTests.testControllerSaveExportsDurableSourceAndRefusesChangedDisk` | export path, not paste |
| Undo/redo adopt exact durable text; stale revision refused and document re-read | `EditHistoryTests.testEditUndoRedoRoundTripAdoptsExactDurableText`, `.testStaleRevisionIsRefusedAndTheDocumentIsReread` | history commands, small text |
| Retry with the same command id after a lost reply is idempotent (`replayed_command`) | `EditHistoryTests.testIdenticalRetryAfterALostReplyIsIdempotent` | undo/redo ids, not the `edit` op |
| Graceful `detachController` → relaunch → retry converges; undo applied before the lost reply is replayed and the differing buffer is resubmitted once (r4) | `EditHistoryTests.testHelperDetachMarksThePendingCommandUncertainAndRetryConvergesAfterReattach`, `.testRetryAfterRelaunchReplaysAnUndoAppliedBeforeTheLostReply` | graceful detach (`close` + `terminate`), never SIGKILL by pid; small text |
| Ledger capacity errors surfaced | `EditHistoryTests.testCapacityErrorsAreSurfaced` | 256 raw edits, small |
| Hold-until-preview vs hybrid release bound (pure + one real-helper release) | `ControllerReleaseTests.*` (5 tests) | policy math and one released edit; not "two edits both durable in order after a large paste" |
| SIGKILL of the *edit-ledger* helper (bridge route) reopens the store and typing while down is not lost | `RealEditLedgerTests.testKilledHelperIsRelaunchedAndTheStoreReopens` | bridge/edit-ledger helper, not the preview-controller; bridge auto-relaunches, the controller route does not |
| SIGKILL of the *bridge* helper mid-session | `RealBridgeTests` (line 106–134) | bridge route |
| Compiler child SIGKILL ×4 with bounded relaunch; ledger + bridge SIGKILL + restart keep the document/receipt durable | mac-live report sections (e) worker-relaunch (rows 351–356, 572–590) and capture-cycle (rows 303–317) | direct-worker route and bridge/ledger helpers; app-level, not the controller `edit` op; no paste-sized text |
| `apply_group` idempotent replay with a fixed id; plan refused when the project moved on | `ProjectSearchTests.testRetryingAnUncertainCommandReplaysExactly` on `origin/agent/mac-search/panel` (NOT on mac-shell/main yet) | replay only; does not assert the buffer adopts the grouped text or that the follow-up preview binds |
| 60 KB document keystroke/rebase costs | `EditorDiagnosticsTests.testRebaseOfLargeDocumentIsFast`, `SourceEditorViewTests.largeDocument` | editor-side only, no helper |

Not covered anywhere (implemented here, `PasteRecoveryTests.swift`):

- (a) one 500 KB paste into a 60 KB document through the controller `edit` op →
  exactly one durable revision (r1→r2), `document.text` byte-identical to the
  buffer, `source_sha256` == SHA-256 of the buffer, follow-up preview bound to
  that editor revision; measured ACK and preview times with `uptime`.
- (b) programmatic multi-range `apply_group` (3 non-overlapping ranges on one
  snapshot) → one durable revision, exact text adopted by the buffer through
  `controllerAdoptHistoryResult`, preview bound to the adopted editor revision,
  the ledger records exactly one history step.
- (c) SIGKILL of the preview-controller by the pid we launched (i) after the
  paste ACK and (ii) immediately after the paste was sent, before the ACK →
  relaunch/reopen: the durable document equals the last ACKed revision or the
  buffer is resubmitted exactly once; final revision is exactly r2 with the
  paste's SHA (no duplicate application, no lost bytes). The controller route
  has no auto-relaunch (the shell resets state on `.exited`); the test re-attaches
  explicitly, as the user's reopen would.
- (d) a second edit typed before the paste's preview arrives is queued behind
  the in-flight edit (hold-until-preview) and both end durable in order
  (r2 = paste, r3 = paste + typed), previews bound in order, nothing lost.

## Status — 13:45Z: tests written, run, passing; ready for the parent's integration

- `swift build --package-path apps/mac --build-tests`: exit 0 at 5bc3fc0f + this lane.
- `swift test --filter PasteRecoveryTests` with the real helpers: run 1 4/5 (one test
  bug: `preview_error` arrives as JSON null; fixed), run 2 **5/5 passed, 0 failures,
  19.7 s**. Measured numbers, `uptime` lines and the exact fixture SHAs are in
  `docs/evidence/mac-paste-recovery-2026-09-12.md`, labelled "under shared load, not
  an isolated result" (parent's heavy-build window; 5-min load 33–39).
- No full `swift test` sweep in this lane (parent's window notice); the parent runs
  it at integration under load < 15.
- No parent-retained file changed, so there are no diffs for the parent to apply;
  the tests use only existing shell API (`updateActiveText`, `attachController`,
  `controllerAdoptHistoryResult`, `controllerState`, `PreviewControllerClient.send`).
- Observation (not patched, outside this lane): a fully-prose 560 KB document
  produces 18.2 MB of compiler preview JSON, over the helper's 16 MiB
  `MAX_OUTPUT_BYTES`; the paste fixture therefore mixes comments and prose. Whether
  the helper's behaviour at that limit is acceptable is a separate gap.
- Load skip: default threshold 1-min load > 20 (brief); `FLASHTEX_PASTE_RECOVERY_LOAD_LIMIT`
  raises it for correctness runs on a shared host and the run labels its numbers.
