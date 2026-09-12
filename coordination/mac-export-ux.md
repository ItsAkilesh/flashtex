# mac-export-ux handoff — exact PDF export cancel/error/conflict UX

- Updated UTC: see `coordination/agents/mac-export-ux.json` `updated_utc`
- Agent / parent / machine alias: `mac-export-ux` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: Gap 4 — native exact PDF export cancellation / error / overwrite-conflict
  UX preserving the existing file on refusal. Owned paths:
  `apps/mac/Sources/FlashTeXMac/ExportSession.swift`,
  `apps/mac/Sources/FlashTeXMac/ShellModel+ExportSession.swift`,
  `apps/mac/Sources/FlashTeXMac/ExactPDFExport.swift` (not parent-retained),
  `apps/mac/Tests/FlashTeXMacTests/ExportSessionTests.swift`, this handoff and
  `coordination/agents/mac-export-ux.json`. Parent-retained files (`ShellModel*.swift`
  core, `ContentView.swift`, `FlashTeXMacApp.swift`, …) are NOT committed; any
  hook is a unified diff in the final report.
- Branch: `agent/mac-export-ux/export-ux` from `origin/agent/mac-claude-a/mac-shell`
  `5bc3fc0f` (contains main `dda0b62`).

## Coverage audit (mandatory first step, 2026-09-12 09:30Z)

Grepped `apps/mac/Tests/FlashTeXMacTests/*`, `apps/mac/Sources/FlashTeXMac/*PDF*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`
for cancel / timeout / overwrite / conflict / sha / mtime / atomic around export.

Already covered:

| Where | What |
| --- | --- |
| `ExactPDFExportTests.testLoadedDisplayListExportsThroughTheExactRoute` | exact route success (page count + ToUnicode text) and refusal of an unreadable list (`{}`): exit ≠ 0, no file created at a *fresh* destination, `captureNote` prefix `Exact export refused`. |
| `RustPDFExportPipeTests.testTimeoutTerminatesAStuckWriter` | v1 `flashtex-pdf` route (fake writer) timeout → terminated within 5 s, error text `did not finish`. Not the exact route; no assertion on the destination file. |
| `RustPDFExportPipeTests.testNonZeroExitIsReportedWithStderr` | v1 route exit 3 reported. No destination assertion. |
| `RustPDFExportPipeTests.testOversizedStderrDoesNotDeadlockAndIsTruncated`, `testMainThreadHeartbeatKeepsRunningDuringExport` | pipe draining / main-thread liveness (v1 route). |
| `PDFExportTests` (3 tests) | CoreGraphics route output validity; `PDFExport.swift:94` writes with `.atomic`. |
| `RustPDFExportTests` (2) | v1 writer fixture PDF; missing binary reported. |
| `DocumentFilesTests.testHelperSaveRefusesExternalChangesUntilOverwriteOrReload`, `testDirectFallbackIsReportedAndStillDetectsConflicts`; `ProjectDocumentsTests.testDirectModeSavesNonEntryDocumentsAndRefusesChangedDisk` | `DocumentConflict` / `expected_disk_sha256` refusal — for `.tex` source saves via `flashtex-project-files`/controller `export`, not for PDF export destinations. |
| mac-live `20260912T110944Z.md` `exact-export` rows 335–342 | bundled `flashtex-pdf-exact from-v2` exit 0, PDF written, PDFKit page count/text. No cancel/error/conflict rows. |
| `docs/evidence/*` | no export cancel/conflict evidence (grep `export` hits are drained-products/parity notes only). |

NOT covered (implemented in this lane):

- (a) cancel mid-run of the exact route: tool terminated by the pid the session
  launched, no partial/temp output left at or beside the destination.
- (b) exact-route refusal / exit ≠ 0 when the destination ALREADY EXISTS: the
  existing file's bytes (SHA-256) and mtime unchanged.
- (c) overwrite conflict: destination changed since the user chose it
  (recorded disk SHA) → `DocumentConflict`-style refusal, existing file kept.
- (d) exact-route timeout (`ExactPDFExport.run`'s 60 s kill was untested) →
  terminated, destination untouched.
- (e) success → atomic replace (sibling temp + `rename(2)`), reported
  bytes/SHA equal to the file on disk.
- observable session state (`idle/running(pid,started)/cancelled/failed/succeeded`)
  for a cancel/progress affordance.

## Durable checkpoint

- Branch / HEAD: `agent/mac-export-ux/export-ux` @ (see `git log -1`), worktree
  `.claude/worktrees/agent-a8655ef8c50a4ea11`.
- Consumed: `origin/agent/mac-claude-a/mac-shell` `5bc3fc0f`.
- Tool: `crates/pdf/target/release/flashtex-pdf-exact` built in this worktree
  (cargo release, 4.8 s) — `FLASHTEX_PDF_EXACT` for the tests.
- Dirty files / next commands: see git status; `cd apps/mac && FLASHTEX_PDF_EXACT=… swift test --filter ExportSessionTests`.
- Decisions: session is a new `@Observable` class stored on `ShellModel` via an
  associated object (same pattern as `DocumentFilesState`); tool writes to a
  sibling temp file (`.<name>.flashtex-export-<uuid>.tmp`) and the session
  renames on success only; cancel uses `Process.terminate()` on the launched
  `Process` (its pid is recorded in the state and re-checked before signalling);
  timeout is reported as `failed(reason: "timed out …")` with cancel semantics.
- Staffing/billing: shared Claude Max quota with parent mac-claude-a; no purchases.
