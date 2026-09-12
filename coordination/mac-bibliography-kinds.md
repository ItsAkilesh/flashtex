# mac-bibliography-kinds handoff

Agent / task / branch: `mac-bibliography-kinds` (Claude Code subagent, parent
`mac-claude-a`, machine `mac-m1max-a`) / Gap 3 "explicit bibliography kind
persistence across native project close/reopen after original disk deletion" /
`agent/mac-bibliography-kinds/bibkinds` (base `origin/agent/mac-claude-a/mac-shell`
5bc3fc0f; helper contract read from origin/main 6472a5d2, read-only).

State: ready for integration (parent applies the 3-line hook diff below)

Owned paths: `apps/mac/Sources/FlashTeXMac/DocumentKinds.swift`,
`apps/mac/Tests/FlashTeXMacTests/DocumentKindsTests.swift`,
`coordination/mac-bibliography-kinds.md`, `coordination/agents/mac-bibliography-kinds.json`.
Small non-retained edit: `PreviewControllerClient.Config.bibliographyPaths`
(`PreviewControllerClient.swift`, +7 lines). Parent-retained files are NOT
committed on `bibkinds`; the labelled local application lives on
`agent/mac-bibliography-kinds/bibkinds-applied` (tip 0b0c3719 = bibkinds + hooks).

## Coverage audit (2026-09-12T13:35Z, mandatory first step)

Searched `apps/mac/Sources/FlashTeXMac/*`, `apps/mac/Tests/FlashTeXMacTests/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`,
`coordination/*.md` for `bibliograph`, `document_kind(s)`, `bibliography_paths`,
`.bib`, `refs.bib`, `kind`:

- `apps/mac/Sources/FlashTeXMac/PreviewControllerClient.swift` `Config.json()`:
  emitted `session_id/project_id/entry_path/project_root/private_ledger_root/
  store_paths/compiler_path/compiler_max_frame_bytes` only — NO `bibliography_paths`.
- `apps/mac/Sources/FlashTeXMac/ProjectDocuments.swift`: `open_document` payloads
  (lines 732, 1081) carry `path/source_versions/membership_generation` only — NO
  `document_kind`; `refreshSnapshot()` (line 1123) reads `source_versions` and
  `membership_generation` only — `document_kinds` is dropped. `detachDocument`
  (line 782) is kind-agnostic. Nothing persists kinds.
- `apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift` `attachController`
  (line 71, parent-retained): builds the Config without declarations, so a reopen
  re-imports a retained `.bib` ledger as `latex` (measured below, control run).
- `apps/mac/Sources/FlashTeXMac/DocumentFiles*.swift`: no kind handling.
- `apps/mac/Tests/FlashTeXMacTests/NavigationTests.swift:743` only mentions a
  `thebibliography` environment as navigation fixture text (not kinds).
- `apps/mac/Tests/FlashTeXMacTests/ProjectDocumentsTests.swift`
  `testHelperRouteOpensWithExactSnapshotAndDetaches`,
  `testHelperSyncAttachesDocumentsOpenedBeforeTheController`,
  `testHelperRouteSavesThroughExportAndFlushesOnSwitch`: untyped open/detach with
  exact snapshot; no `document_kind`, no close/reopen, no disk deletion.
- `tools/native-validation/mac-live/reports/20260912T110944Z.md`: zero matches.
- `docs/evidence/*.md`: zero matches.
- `coordination/mac-bibliography.md`: `crates/bibliography` parser lane (Rust data
  layer), unrelated to helper document kinds.
- Helper side (read-only, origin/main 6472a5d2; local crate identical for kinds):
  `crates/preview-controller/docs/source-plans.md` — declarations "must be
  supplied again on reopen", "the application owns persistence of its startup
  configuration", "persist the bibliography paths from this [`document_kinds`]
  map ... do not reconstruct kinds from file extensions", "No live native
  application adoption has been verified yet."

Conclusion: the gap was entirely uncovered on the Mac side; only the uncovered
part was implemented.

## What was added

`apps/mac/Sources/FlashTeXMac/DocumentKinds.swift` (new, ~330 lines):
- `DocumentKind` (`latex`/`bibliography`, exactly the helper's strings).
- `DocumentKindsRecord` / `DocumentKindsStore`: `<controller ledger root>/
  document-kinds.json` (`{"version":1,"bibliography_by_entry":{"main.tex":
  ["refs.bib"]}}`), entry-keyed, sorted/de-duplicated, entry excluded; unknown
  record version is reported and never rewritten; corrupt file is reported by
  `load` and rewritten only by an explicit `persist`.
  `launchableBibliographyPaths(projectRoot:privateLedgerRoot:projectID:entry:)`
  keeps a persisted path only if it is on disk under the root or has a retained
  helper ledger slot (`project-sha256("<len>:<realpath root>:<project id>")/
  sha256(path)`, mirroring `file_project.rs`; `realpath(3)`, because Foundation's
  `resolvingSymlinksInPath` strips `/private` and would bind a different
  directory) — a stale record can therefore never stop the helper from starting
  ("bibliography source missing on disk"); dropped paths are reported in status.
- `DocumentKinds` (`@Observable`, `model.documentKinds` associated object):
  `refresh()` reads `document_kinds` + `membership_generation` from `snapshot`
  (never persists — a reopen without declarations must not clobber the record);
  `declareBibliography(path)` = exact-snapshot `open_document {document_kind:
  "bibliography"}` → `ProjectDocuments.openDocument` adopts the now-indexed
  durable document without a second membership change → refresh → persist;
  refuses the entry, an already-`latex` member (helper changes kinds only via
  detach + typed reattach), stale snapshots (reported, not retried);
  `undeclare(path)` = `ProjectDocuments.detachDocument` → refresh → persist;
  observation tracking on `controllerStatus`/`project.membershipGeneration`
  re-reads kinds on ready/open/detach and clears them on close.
  `startupBibliographyPaths(projectRoot:privateLedgerRoot:projectID:entry:)` is
  the launch hook.
- UI: `DocumentKindIndicator` (caption: "bibliography (declared)" for the active
  document or "N bibliography source(s)", hidden when none; help = status +
  persist error) and `DocumentKindsMenuSection` (list, "Undeclare … (detach,
  this session)", "Declare Bibliography Source…" via a rooted NSOpenPanel; the
  chosen file is passed to the helper as a rooted path — no extension check).

`PreviewControllerClient.Config.bibliographyPaths` → `bibliography_paths` (only
when non-empty).

## Tests — `apps/mac/Tests/FlashTeXMacTests/DocumentKindsTests.swift` (8)

Unit (no helper): `testConfigEmitsBibliographyPathsOnlyWhenDeclared`,
`testRecordIsEntryKeyedAndRoundTrips`, `testPersistUsesTheControllerLedgerRootForTheProject`,
`testLaunchFilterDropsDeclarationsTheHelperCouldNotImport`,
`testKindsAreNeverInferredWithoutTheHelper` (a `.bib` member has no kind, refresh/
declare/undeclare refuse without a helper).

Real helper (XCTSkip without `FLASHTEX_PREVIEW_CONTROLLER`+`FLASHTEX_COMPILER`):
- `testDeclaredBibliographyKindPersistsAcrossReopenAfterDiskDeletion`: open
  main.tex (+discovered chapter.tex) → kinds `{main.tex:latex, chapter.tex:latex}`,
  refs.bib not indexed; explicit `document_kind:"latex"` open of chapter.tex is
  refused as already a member (then adopted); declare refs.bib → helper reports
  `bibliography`, shell membership gains refs.bib with the ledger text, record
  written; re-declare → alreadyDeclared; chapter.tex → refused (latex); entry →
  refused; missing.bib → helper refusal, record unchanged. `detachController`
  (helper `close`), delete refs.bib on disk, launch the helper with the same
  roots + `bibliography_paths:["refs.bib"]` → `document_kinds` = `{main.tex:
  latex, chapter.tex:latex, refs.bib:bibliography}`, `document refs.bib` returns
  the durable text, `file_status` = `missing`. CONTROL launch without
  declarations (today's `attachController`) → `refs.bib:latex` (kind lost; the
  extension is never used).
- `testShellReopenRestoresTheDeclaredKindThroughAttachController`: the same
  through `ShellModel.attachController`; passes on `bibkinds-applied`, XCTSkips
  on `bibkinds` after asserting the gap reproduction (`refs.bib:latex`) and that
  the record is untouched.
- `testUndeclareDetachesAndForgetsTheDeclaration`: declare, generic open of
  chapter.tex is picked up by tracking, undeclare → detached, generation
  advances, record emptied; a latex member is not undeclared; close clears kinds.

Measured (under shared load, not an isolated result):
- `bibkinds` b5f29671: `swift build` OK; `swift test --filter DocumentKindsTests`
  with real helpers: Executed 8, 1 skipped (hook absent), 0 failures, 0.61 s;
  `uptime` 9:44 load 11.38 21.48 22.63. Without helper env: 8 executed, 3
  skipped cleanly. Narrow regression `--filter "ProjectDocumentsTests|
  PreviewControllerTests"`: 20/20, load 10.54.
- `bibkinds-applied` 0b0c3719: `swift build` OK; DocumentKindsTests Executed 8,
  0 skipped, 0 failures, 0.62 s; `uptime` 9:43 load 16.52 23.89 23.50.
- Full `swift test` NOT run (parent's heavy-build notice; load > 15).
- Helpers: `/Users/jay3332/Projects/flashtex/crates/{compiler,preview-controller}/
  target/release/*` (built 09:30 local from the mac-shell crate state, which
  contains the `bibliography_paths`/`document_kinds` protocol; confirmed by the
  helper replies above).

## Exact diffs for parent-retained files (apply on mac-shell)

```diff
diff --git a/apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift b/apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift
--- a/apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift
+++ b/apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift
@@ -96,6 +96,7 @@ extension ShellModel {
         if let s = ProcessInfo.processInfo.environment["FLASHTEX_CONTROLLER_MAX_FRAME_BYTES"], let n = Int(s) {
             config.compilerMaxFrameBytes = n
         }
+        config.bibliographyPaths = documentKinds.startupBibliographyPaths(projectRoot: projectRoot, privateLedgerRoot: ledgerRoot, projectID: projectId, entry: entryPath) // DocumentKinds.swift: persisted explicit declarations, never inferred
         controllerState = ControllerState()
         do {
             controller = try PreviewControllerClient(executable: url, config: config) { [weak self] event in
diff --git a/apps/mac/Sources/FlashTeXMac/ContentView.swift b/apps/mac/Sources/FlashTeXMac/ContentView.swift
--- a/apps/mac/Sources/FlashTeXMac/ContentView.swift
+++ b/apps/mac/Sources/FlashTeXMac/ContentView.swift
@@ -147,6 +147,7 @@ private struct EditorPane: View {
                 }
                 .labelsHidden().frame(maxWidth: 260)
                 ProjectMenu()
+                DocumentKindIndicator() // DocumentKinds.swift: helper-reported bibliography kind, read-only
                 if let url = model.documentURL {
                     let dirty = model.project.isDirty(model.activePath)
                     let name = model.activePath == model.project.entryPath ? url.lastPathComponent : model.activePath
@@ -231,6 +232,7 @@ private struct ProjectMenu: View {
                 }
                 .help("Session only: " + ProjectDocuments.detachScopeNote)
             }
+            DocumentKindsMenuSection() // DocumentKinds.swift: declare/undeclare bibliography sources
         } label: {
             Label("Project", systemImage: "doc.on.doc")
         }
```

## Limitations / notes for the parent

- The session temporary project (unsaved or seeded buffer whose file name is not
  the entry) never persists or supplies declarations (by design; the helper
  would try to import them from the temp root).
- `ProjectDocuments.detachScopeNote` still says detach is session-only; for a
  declared bibliography source, undeclare additionally removes it from the
  persisted record, so it is NOT re-supplied on reopen (the retained ledger
  itself is kept by the helper, as documented).
- Reopen keeps the kind only through the helper's retained private ledger; if
  the ledger root is cleared and the `.bib` is gone, the launch filter drops the
  declaration (status line) rather than failing the launch. Not tested with a
  UI capture; no screenshot (indicator is a caption; parent hook not on mac-shell).
- Search lane: a kind filter for `SearchPanel*` would be
  `model.documentKinds.kind(of:) == .bibliography` per path; not touched here.
- `projectId` passed to the helper is `ShellModel.projectId` (`result?.projectId
  ?? "demo"`), pre-existing; the ledger binding depends on it, so the launch
  filter uses the same value `attachController` passes.

## Durable checkpoint

- Branch `agent/mac-bibliography-kinds/bibkinds` @ b5f29671 (pushed); applied
  branch `agent/mac-bibliography-kinds/bibkinds-applied` @ 0b0c3719 (pushed);
  worktree `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a0c5c1743911589cd`.
- Dirty files: none besides this handoff/registration refresh.
- Consumed SHAs: mac-shell 5bc3fc0f (base); main 6472a5d2 (contract, read-only).
- Next: parent applies the diff above on mac-shell, merges `bibkinds`, runs the
  full `swift test` at load < 15.
- Staffing/billing: shared Claude Max quota with parent; no purchases.

Updated: 2026-09-12T13:47Z
