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

## Ready behavior and evidence (26a72a02)

- `ExportSession.swift` (`@MainActor @Observable`): `state` =
  `idle | running(pid, started) | cancelled | failed(reason) | succeeded(bytes, sha256)`,
  `conflict: DocumentConflict?`, `Destination{url, expectedDiskSHA256}` with
  `.recordingCurrentDisk(url)`, `Report{state, exitCode, stdout, stderr, conflict}`.
  `start(tool:list:destination:fontDirs:timeout:completion:)` refuses while
  running, checks the destination conflict before launch, points the tool at a
  sibling temp `.<name>.flashtex-export-<uuid>.tmp`, drains pipes off the main
  actor, terminates on timeout (SIGTERM via the launched `Process`, pid
  re-checked; SIGKILL after 5 s only if the same live `Process` still reports
  that pid), re-checks the destination before `rename(2)`, and reports
  bytes/SHA of the renamed file. `cancel()` signals only the launched process.
  `DocumentConflict.exportSummary` gives the export wording.
- `ShellModel+ExportSession.swift`: `exportSession` (associated object),
  `cancelExactExport()`, `exportPDFExact(listURL:tool:destination:timeout:completion:)`
  mirroring the outcome into `captureNote` (`Exported exact PDF (N bytes, sha256 …) to …`,
  `Exact export cancelled; nothing was written to …`, `Exact export refused (exit N): …`,
  `Export refused: <file> was modified on disk since you chose it (…); the existing file is kept…`,
  `Exact export failed to run: … did not finish within N s; terminated …`).
- `ExactPDFExport.swift`: the panel path records the approved disk state
  (`.recordingCurrentDisk(out)`) after the user confirmed the save panel; the
  pre-session `exportPDFExact(listURL:tool:to:completion:)` (Outcome shape)
  routes through the session, so `ExactPDFExportTests` still passes unchanged.
- Tests `ExportSessionTests` (6): (a) cancel on a FIFO list — tool pid gone,
  `/bin/sleep` bystander still running, existing file bytes+mtime identical, no
  temp left, second `start` refused, `captureNote` cancelled text; (b) refusal
  (`{}` list) with existing and fresh destinations — SHA+mtime unchanged, tool
  message surfaced, nothing created; (c) conflict before launch —
  modifiedExternally / alreadyExists / deletedExternally with a non-launchable
  tool (and the unchanged case reaches the launch failure instead); conflict
  during render via a tampering script tool — `modifiedDuringSave`, other
  writer's file kept, temp removed; (d) timeout 1 s — terminated, pid gone,
  destination untouched; (e) success — `succeeded(bytes, sha)` equal to the
  file, `%PDF-` header, atomic replace of a planted file, legacy Outcome shape
  agrees. Real-tool tests `XCTSkip` without `FLASHTEX_PDF_EXACT`; elapsed
  bounds asserted only at 1-min load ≤ 20 (values always printed).
- Evidence: `swift test --filter 'ExportSessionTests|ExactPDFExportTests'`
  with `FLASHTEX_PDF_EXACT=crates/pdf/target/release/flashtex-pdf-exact`
  (built here from this worktree's crates/pdf): 7 passed / 0 failed / 0 skipped.
  `uptime`: 09:37 load 33.82 45.33 28.70 — under shared load, not an isolated
  result (12 lanes compiling); measured cancel 0.35 s, timeout path 1.04 s.
  Without the env var: 6 tests, 2 passed, 4 skipped. Full `swift test` NOT run
  (parent heavy-build window notice); `swift build` clean for the lane files.
- Labelled local application: `agent/mac-export-ux/export-ux-applied` (6aacda63)
  carries the ContentView diff below; compiled, not for integration.

## Diff request for parent-retained `ContentView.swift` (Footer: progress + Cancel)

```diff
diff --git a/apps/mac/Sources/FlashTeXMac/ContentView.swift b/apps/mac/Sources/FlashTeXMac/ContentView.swift
index cfe9c963..af4cda3d 100644
--- a/apps/mac/Sources/FlashTeXMac/ContentView.swift
+++ b/apps/mac/Sources/FlashTeXMac/ContentView.swift
@@ -395,7 +395,15 @@ private struct Footer: View {
                  ?? "Click text in the preview to select its source range.")
                 .font(.caption).foregroundStyle(.secondary).lineLimit(1)
             Spacer()
-            if let note = model.captureNote {
+            if case .running(let pid, _) = model.exportSession.state { // ShellModel+ExportSession.swift
+                ProgressView().controlSize(.small)
+                Text("Exporting exact PDF (flashtex-pdf-exact pid \(pid))…")
+                    .font(.caption).foregroundStyle(.secondary).lineLimit(1)
+                Button("Cancel") { model.cancelExactExport() }
+                    .controlSize(.small)
+                    .help("Terminate flashtex-pdf-exact; nothing is written to the destination")
+                    .accessibilityIdentifier("export.cancel")
+            } else if let note = model.captureNote {
                 Text(note).font(.caption).foregroundStyle(.secondary).lineLimit(1)
             }
         }
```

## Durable checkpoint

- Branch / HEAD: `agent/mac-export-ux/export-ux` @ 26a72a02 (+ this coord
  commit), pushed; worktree `.claude/worktrees/agent-a8655ef8c50a4ea11`.
  Applied branch `agent/mac-export-ux/export-ux-applied` @ 6aacda63, pushed.
- Consumed: `origin/agent/mac-claude-a/mac-shell` `5bc3fc0f` (main merge base `527ae54`).
- Tool: `crates/pdf/target/release/flashtex-pdf-exact` built in this worktree
  (cargo release, 4.8 s) — `FLASHTEX_PDF_EXACT` for the tests.
- Dirty files: none. Rerun: `cd apps/mac && FLASHTEX_PDF_EXACT=$PWD/../../crates/pdf/target/release/flashtex-pdf-exact swift test --filter 'ExportSessionTests|ExactPDFExportTests'`.
- State: ready for integration; lane stops here (bounded task complete).
- Decisions: session is a new `@Observable` class stored on `ShellModel` via an
  associated object (same pattern as `DocumentFilesState`); tool writes to a
  sibling temp file (`.<name>.flashtex-export-<uuid>.tmp`) and the session
  renames on success only; cancel uses `Process.terminate()` on the launched
  `Process` (its pid is recorded in the state and re-checked before signalling);
  timeout is reported as `failed(reason: "timed out …")` with cancel semantics.
- Staffing/billing: shared Claude Max quota with parent mac-claude-a; no purchases.
