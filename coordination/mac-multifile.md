# mac-multifile handoff — multi-file LaTeX projects in the shell

- Updated UTC: see the agents JSON (`coordination/agents/mac-multifile.json`)
- Agent / parent / machine: `mac-multifile` (Claude Code subagent) / parent
  `mac-claude-a` / `mac-m1max-a`
- Lane: "Multi-file LaTeX projects in the shell" (parent dispatch, issue #2):
  discover `\input`/`\include` targets, open them through the durable helper
  (exact snapshot) or directly under the rooted project directory, keep
  `ShellModel.documents` in sync (entry first, stable order, per-document
  durable revisions), detach explicitly, switch the editor document with the
  caret/selection preserved and no unsaved text lost. Follow-ups done: (1)
  per-document save + flush-on-switch durability, (2) helper sync on attach
  + the parent diffs trial-applied.
- Owned paths: `apps/mac/Sources/FlashTeXMac/ProjectDocuments.swift` (new),
  `apps/mac/Tests/FlashTeXMacTests/ProjectDocumentsTests.swift` (new), this
  file, `coordination/agents/mac-multifile.json`.
- Branch / worktree: first lane `agent/mac-multifile/project-documents`
  (integrated by the parent into mac-shell 6f4ee94); refill
  `agent/mac-multifile/transitive` (pushed) from mac-shell 6f4ee94, in
  `.claude/worktrees/agent-a4c2c20d016d17c6d`. On each branch the last commit
  is the clearly labelled **"requested parent diffs"** commit that the parent
  can drop or cherry-pick; everything before it touches owned paths (plus the
  requested `docs/evidence/` directory).
- Rules honoured: no purchases; transferred crates untouched; parent-retained
  files changed only in the separate labelled commit; the app was launched
  only with `FLASHTEX_NO_ACTIVATE=1` for evidence; commits carry the user as
  primary author with truthful trailers.

## Refill (branch `agent/mac-multifile/transitive`, from mac-shell 6f4ee94)

The parent merged the first lane with the parent diffs (mac-shell 091cee8 →
6f4ee94). Refill items:

1. **Transitive discovery** — `ProjectDocuments.discoverClosure(maxDepth: 8)`:
   depth-first in source order from the entry (chapter.tex → \input{section}
   …), text from the open buffer else the rooted disk file (both routes),
   bounded by `maxIncludeDepth` (8; a deeper reference is listed as
   `nested deeper than 8 levels; not discovered`) and `maxClosureDocuments`
   (256, `truncated`). A reference to a document on the current include chain
   is refused as a cycle (`\input{main} closes an include cycle: main.tex →
   chapter.tex → ch/section.tex → main.tex`, self-includes too); a document
   reached again through another parent is listed once as `duplicate`
   (diamond, not a cycle) and not rescanned. `discoverIncludes(in:)` (one
   level) is unchanged and shares the resolver.
2. **Open All** — `openDiscoveredIncludes()` now opens the closure in stable
   order (parents before their includes; `role: .included(from: parent)`),
   rediscovering after each open so a document read from the helper's ledger
   contributes its includes; `lastOpenReport` lists opened / already open /
   refused / unresolvable (`\input{x} in chapter.tex: why`) / truncated and
   `status` carries the summary. The Project menu (parent diff, applied in
   the labelled commit) lists the closure indented by depth and shows the
   last report's unresolvable references.
3. **Evidence** — `docs/evidence/multifile-project-2026-09-12/` (window.png,
   app.log, project/, README.md): running app, real helper + compiler,
   three-file project, `FLASHTEX_OPEN_INCLUDES=1 FLASHTEX_NO_ACTIVATE=1`,
   window captured by id.
4. **⌘S routing verified end to end** (`testSaveCommandWritesOnlyTheActive
   NonEntryDocumentDirectly` / `…ThroughTheHelper`): with chapter.tex active,
   `saveTexInteractive()` writes chapter.tex only — `main.tex` bytes asserted
   unchanged, the entry `savedText` untouched, `captureNote == "Saved
   chapter.tex"`, `file_status` `matches_source` on the helper route.
5. **Detach persistence** — what the helper offers: nothing persistent.
   STDIO.md: "Detach excludes a non-entry document for this session … Project
   restart restores all retained documents; persistent exclusions are not
   implemented." The direct route has no project file to record an exclusion
   in either, and the include is rediscovered from the entry text on the next
   Open All. So a detach is session-only by construction;
   `ProjectDocuments.detachScopeNote` states it, the detach status/captureNote
   carries it, and the picker's Detach item is labelled "(this session)" with
   the note as help (parent diff). A persistent exclusion would need a helper
   feature (or a project-side file) — not in this lane.

Refill validation (full `swift test` with real workers on
`agent/mac-multifile/transitive` + the ContentView diff): three runs at load
47–78 — run 1 `461 tests, 8 skipped, 2 failures` (the filtered log lost the
failure lines), run 2 `461 tests, 8 skipped, 1 failure`
(`RuntimeTranscriptTests testShellAndValidatorBothRejectUnnegotiatedShapes…`:
`compile_result has no matching request id`, direct-worker transcript checker;
passes 3/3 in isolation; not this lane's code), run 3 **`Executed 461 tests,
with 8 tests skipped and 0 failures`** (skips: FLASHTEX_EXPLAIN,
FLASHTEX_PDF_EXACT, nearby evidence/serve env, an AX overlay probe,
assistant-context helper, evidence dir). `ProjectDocumentsTests`: 18/18.

## Ready behaviour (owned files)

`ProjectDocuments.swift`

- `ProjectIncludes.scan(text, limit:)` — lexical `\input{…}` / `\include{…}`
  discovery, a port of `crates/project-files/src/scan.rs` restricted to the
  two source-including commands: skips `%` comments, `\verb`, verbatim-like
  environments; command name = maximal ASCII-letter run (`\inputfoo` never
  matches); optional `[…]` argument; bare `\input name`; nested braces with
  backslash escapes; `literal == false` for arguments containing `\` or `#`.
  Spans are UTF-8 byte offsets (runtime-v1 convention: whole command and the
  argument). Bounded: at most `maxReferences` (64) per document; an
  unbalanced brace consumes to the end.
- `ProjectIncludes.normalize` mirrors `ProjectPath::normalize` (relative only,
  `.`/empty dropped, `..` pops or `escapesRoot`, no `\` `:` NUL/control, no
  leading `/` or `~`); `candidates(for:)` gives TeX order `name.tex`, `name`
  (a `.tex` name as written). References resolve against the project root
  (TeX working-directory rule, same as the Rust graph).
- `ProjectDocuments` (`model.project`, associated object like `files`):
  - `listing` — `ShellModel.documents` order with role (`entry` / `included(from:)`
    / `opened`), origin (`buffer` / `helper` / `disk`), the helper's durable
    revision (`controllerState.durable[path]`) and per-document dirtiness
    (entry vs `savedText`, others vs the text they were opened with).
  - `discoverIncludes(in:)` — each reference with candidates, resolved path
    and state `open` / `available` / `unresolvable(why)` (needs expansion,
    escapes root, symlink, missing, no project root).
  - `openDocument(path)` (exact rooted path) / `openInclude(argument)` (TeX
    candidates) / `openDiscoveredIncludes()`:
    - helper attached: `snapshot` → if the helper already lists the path
      (it discovers includes at startup) `document {path}`, else
      `open_document {path, source_versions, membership_generation}` with the
      exact snapshot; a `… snapshot is stale` refusal is reported (snapshot
      cleared) and never retried blindly; the reply's versions/generation are
      adopted; durable revision/sha/text recorded in `controllerState` so
      edits to that document go through the controller's normal edit route;
      disk hash captured via `file_status` for a later rooted save.
    - no helper: read under `documentURL`'s directory only; symlink components
      and escapes refused (`rootedFile`), 8 MiB bound, UTF-8 required.
    - appended after the entry (stable open order), baseline recorded.
  - `detachDocument(path, discardingEdits:)` — entry refused; unsaved edits
    refused unless discarded (text kept in `detachedBuffers`); on the helper
    route `detach_document` with the exact snapshot first (refusal leaves the
    membership unchanged), then the shell membership, durable state, caret
    memory and anchor are dropped; detaching the active document switches to
    the entry first.
  - `switchDocument(to:)` — remembers the outgoing caret/selection
    (`carets[path]`), sets `activePath`, restores the incoming range clamped
    to its text on a surrogate boundary via `ShellModel.selection` (+
    `caretUTF16`/`caretLengthUTF16`); refused while `pendingEdit` is set (the
    editor applies it without a document check); re-maps the incoming
    document's durable revision to the current editor revision when its
    buffer equals the durable text (previews are not misreported stale);
    flushes the outgoing buffer to the helper (`flushToHelper`).
    Navigation's direct `activePath` writes (Navigation.swift) still record
    the outgoing caret through `withObservationTracking` (fires before the
    change, while the caret still belongs to the outgoing document).
  - `saveDocument(path)` (async) — non-entry members save to their own rooted
    file: helper `export` after the buffer is durable, with the mandatory disk
    expectation (hash seen at open / null) so a changed or appeared file is a
    `DocumentConflict` and nothing is written; off the helper,
    `DocumentFilesState.save` compare-and-replace. `saveDocumentNow(path)` is
    the synchronous file-layer variant for the quit flow.
  - `flushToHelper(path)` — after a switch, keystrokes still queued in the
    controller's one-slot edit queue (which follows `activePath`) are
    submitted as one `edit`; also releases an in-flight edit of the
    switched-away document once its durable revision was previewed (the
    controller judges release by the *active* path; parent diff below fixes
    the root cause).
  - `syncWithHelper()` — on controller ready (observed through
    `controllerStatus`), members opened before the attach get a durable
    document (`document` / `open_document`), and a differing buffer is
    submitted as an edit. No request when nothing is missing.
  - `projectStatus()`, `refreshSnapshot()`, `helperRequest(type, payload)`
    (bounded round trip through `controllerState.awaiting`).
  - Launch hook for evidence/automation: `FLASHTEX_OPEN_INCLUDES=1`
    (+ `FLASHTEX_ACTIVE_PATH`), effective once something reads `model.project`
    (the ContentView diff does).

## Validation

- `swift test` in apps/mac with real workers (`FLASHTEX_PREVIEW_CONTROLLER`,
  `FLASHTEX_COMPILER`, `FLASHTEX_PDF`, `FLASHTEX_BRIDGE`,
  `FLASHTEX_EDIT_LEDGER`, `FLASHTEX_PROJECT_FILES` from the main checkout's
  release builds):
  - base 92a052c, owned files only: `Executed 380 tests, with 4 tests skipped
    and 0 failures` (skips: assistant-context helper / evidence-dir env).
  - base 92a052c + parent diffs: `Executed 383 tests, with 4 tests skipped and
    0 failures`.
  - merged mac-shell 271a366 + parent diffs (final tree): first run crashed in
    another lane's test under load 32–39 (`NearbyTranscriptAcceptanceTests
    testRecordedSessionIsAcceptedOnLoopbackWithDuplic…`, NearbyListenerTests.swift:1549
    `timed out waiting for …` then `Fatal error: Array index is out of range`);
    it passes 3/3 in isolation and the full rerun was green: `Executed 394
    tests, with 6 tests skipped and 0 failures` (extra skips are env-gated
    nearby/serve tests, not this lane). Reported here, not fixed (not owned).
  - the lane file (`ProjectDocumentsTests`, 13 tests) 13/13 across six
    consecutive runs with the real helper and compiler.
- `ProjectDocumentsTests` (13): scanner spans / skips / bounds / bytes-not-
  characters; path normalization and candidates; direct-mode two-file project
  (discover, open, switch with caret memory and clamping, dirty, pending-edit
  refusal, detach rules, recover-on-reopen); escape / symlink / missing /
  absolute / macro refusals; no-project-root refusal; navigation switch caret
  memory; direct-mode save + external-change conflict; real helper: open via
  `document` and `open_document`, generation advance, `project_status`, stale
  snapshot refused by the helper, edit of an opened document bound to the
  editor revision, detach; real helper: flush on switch, export save, refused
  export on external change; sync of directly opened documents on attach.
- Helper conversation traces (FLASHTEX_LOG) confirmed the routes taken:
  `opened chapter.tex (durable r1, membership g3)`, `opened appendix.tex
  (durable r1, membership g4)`, `releasing the in-flight edit of chapter.tex
  (durable r2 previewed while main.tex is active)`, `flushed chapter.tex …
  durable r3`, `saved chapter.tex through the preview controller (durable r3)`,
  `chapter.tex was modified on disk since it was opened; … kept unsaved`.

## Limitations / not verified

- Discovery is lexical and entry-document only (no transitive walk; the
  helper's own startup discovery covers nested includes on that route).
  `\input{\jobname}` and friends are reported as needing expansion.
- Detach on the helper route is session-local (helper restart restores
  retained documents; the helper does not implement persistent exclusions).
- The helper's `open_document` needs a file-project session (a seeded unsaved
  buffer uses the session temp project, where only the entry exists).
- Without the parent diffs: ⌘S / quit still act on the *active* buffer and the
  entry URL (`saveTex` writes `activeText` to `documentURL`) — with a non-
  entry document active that would overwrite the entry file; `compile()` and
  `attachController` name `activePath` as the entry. The diffs below fix all
  of these; until they are applied, treat document switching as model-only.
- No live click evidence of the picker (Accessibility not granted). Window
  capture with the parent diffs applied, the real helper, `FLASHTEX_SEED_FILE`
  = a two-file project, `FLASHTEX_OPEN_INCLUDES=1 FLASHTEX_ACTIVE_PATH=chapter.tex`,
  `FLASHTEX_NO_ACTIVATE=1`: picker shows `chapter.tex r1`, the Project menu,
  the editor holds chapter.tex, the preview (WORKER flashtex-preview-controller,
  revision 2, status ok) shows both sections with the caret-synced "2" of the
  chapter heading highlighted. Log: `opened chapter.tex (durable r1,
  membership g3)`. PNG at `/tmp/flashtex-multifile-evidence.png` (not committed;
  no image evidence directory exists in the repo).

## Requested parent diffs (applied in the labelled commit; parent may drop it)

The exact patch is the last commit on the branch ("requested parent diffs");
`git show <sha> -- apps/mac/Sources/FlashTeXMac/ContentView.swift` etc. Summary:

1. `ContentView.swift` EditorPane — Document picker bound to
   `model.project.switchDocument(to:)` over `model.project.listing` (dirty
   marker `•`, durable `rN`); a `ProjectMenu` (Open / Show each discovered
   include, Open All Includes, Save / Detach the active non-entry document);
   the file caption follows the active document.
2. `ShellModel.swift` `compile()` — `entryPath: project.entryPath`.
3. `ShellModel+Controller.swift` — `attachController` names
   `documents.first` as the entry (root = its directory), and
   `applyControllerPreview` releases the in-flight edit by
   `update.sourceVersions[inFlight.path]` instead of the active path.
4. `DocumentFiles.swift` (mac-document-files lane / parent) — `isDirty` and
   `saveTexInteractive` route a non-entry active document through
   `project.isDirty` / `project.saveDocument` (never the entry URL).
5. `FlashTeXMacApp.swift` — quit checks `project.anyDirty`, saves non-entry
   members with `saveDocumentNow`, then the entry.
