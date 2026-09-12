# mac-navigation handoff — exact source ↔ preview navigation

- Updated UTC: 2026-09-12T10:05Z
- Agent / parent / machine: `mac-navigation` (Claude Code subagent) / parent
  `mac-claude-a` / `mac-m1max-a`
- Lanes: (1) "Exact source-to-preview and preview-to-source navigation" —
  integrated into mac-shell at 40d53b7 with the parent diff applied;
  (2) refill "Go to Matching through the helper's lexical project index" —
  branch `agent/mac-navigation/helper-navigate` (from mac-shell 40d53b7,
  merged 271a366), pushed, ready for integration.
- Owned paths: `apps/mac/Sources/FlashTeXMac/Navigation.swift`,
  `apps/mac/Sources/FlashTeXMac/CaretSync.swift`,
  `apps/mac/Tests/FlashTeXMacTests/NavigationTests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/CaretSyncTests.swift`, this file,
  `coordination/agents/mac-navigation.json`.
- Worktree: `.claude/worktrees/agent-aa3ca1d9a088b9f09`. Dirty files: none
  besides this handoff and the agents JSON at the time of writing.
- Rules honoured: no purchases; no edits outside owned paths; transferred
  crates untouched; no app launched; user is primary author, truthful trailers.
- Context usage: not exposed as a percentage to this worker; the session
  token-budget readout is well under 20% of the 1M budget.

## Refill: helper-backed Go to Matching (branch helper-navigate, 37c7970)

- With `flashtex-preview-controller` attached and ready, ⌘⇧D on
  `\ref`/`\eqref`/`\pageref`/`\autoref`/`\cref`/`\nameref`, the `\cite`
  family, and user commands (`\mycmd` → its `\newcommand`) goes to the
  helper's lexical index (STDIO.md): `snapshot` → exact `source_versions`
  → `navigate {source_versions, path, byte_offset}`. `\begin`/`\end` stay
  in-buffer; without a helper the in-buffer matcher (`goToMatchingInBuffer`)
  is unchanged.
- Refusals (never guessed): the active buffer differs from the durable text
  the helper indexed for the snapshot's revision ("has edits the helper has
  not indexed yet; try again"); the helper's stale-version error ("Project
  changed while looking up the index …"); a reported location whose revision
  is no longer the snapshot's; a location that overlaps an edit made since.
- Selection: the definition (or, from a definition, the next reference in
  helper path order via `complete` occurrences, wrapping) is selected
  exactly in its document. A project document not open in the window
  (chapter.tex discovered through `\input`) is read with `document` (the
  parent's handler records durable text/revision) and appended to
  `documents` at its durable revision, then made active. An open buffer that
  moved on is rebased across the edit with `Navigation.rebaseExactly`.
- `Navigation.helperProbeOffset(in:caretByte:)`: the index keys symbols by
  name bytes (the argument of `\ref{…}`, the letters after the backslash),
  so a caret on the backslash/command name is moved onto the name; inside an
  argument the caret is sent as is (the helper picks the `\cite{a, b}` item).
- Request/reply: `controllerState.awaiting[id]` + `controller.send` (the
  `controllerSave`/`controllerFileStatus` pattern). Both are internal, so
  **no ShellModel/ShellModel+Controller diff was needed**.
- Validation: `NavigationHelperProbeTests` (2, pure), `NavigationHelperTests`
  (3; two run against the real helper + real compiler with
  `FLASHTEX_PREVIEW_CONTROLLER`/`FLASHTEX_COMPILER` set — they ran here;
  skip otherwise). Full apps/mac suite with compiler/pdf/bridge/edit-ledger/
  preview-controller binaries from the main checkout: **386 tests, 9
  skipped (other lanes' optional routes), 0 failures** at 37c7970.
- Risk found for the parent/document-files owner (not in my paths): once a
  second project document is active (`activePath == "chapter.tex"`),
  `ShellModel.isDirty` compares `savedText` (main.tex) with `activeText`
  and `saveTex()` writes `activeText` to `documentURL` (main.tex). The menu
  save with the helper attached goes through `controllerSave` (exports
  `activePath` — correct), but the direct `saveTex()` callers (quit flow in
  FlashTeXMacApp, `openTex(.saveFirst)`) would write chapter text into
  main.tex. Suggested guard in `saveTex()`: `guard documentURL?.lastPathComponent
  == activePath else { captureNote = "\(activePath) is saved through the
  preview controller"; return false }` (or route to `controllerSave`). This
  predates the refill (the fixture route already switches documents) but
  the helper route makes it reachable in a real project.

## Lane 1 (integrated into mac-shell 40d53b7): ready behavior

- `ShellModel.navigateExactly(to:expectedText:)` (Navigation.swift):
  preview click / diagnostic "Go to source" route with these guarantees:
  staleness decided byte-for-byte (`sameBytes`; an NFC→NFD edit of the same
  visible text is an edit, the parent's `!=` treats it as unchanged and would
  select shifted bytes); a span overlapping the edited region is refused and
  the footer names the edited bytes ("bytes 66..<71 of main.tex overlap the
  edit at 66..<69, now 66..<68; recompile to navigate"); a rebased span is
  re-verified to spell the compiled bytes; the selection covers whole
  composed character sequences (never a split scalar or cluster; widening is
  noted); the item text only annotates ("“1” is generated from this source")
  instead of refusing section numbers / `\ref` values after a rebase; a span in
  another open document switches `activePath` and notes it; `caretUTF16` /
  `caretLengthUTF16` follow the selection so caret→preview sync stays exact.
- `Navigation.editorRange(start:end:in:path:)`: pure UTF-8 → UTF-16 mapping
  with the same guarantees (refuses out-of-range / reversed / mid-scalar;
  widens split clusters; an empty span inside a cluster moves to its start).
- `Navigation.commandUses` skips `%` comments and `\%` / `\\` escapes; a
  caret on the shared byte of `\end{a}\begin{b}` belongs to the command that
  starts there; nested same-name environments match by depth.
- `Navigation.matchingRange(in:activePath:caretByte:)`: `\ref` → `\label` in
  the active document first, then other open documents in project order;
  `\label` cycles its references across documents; `goToMatching` switches
  the active document.
- `goToDiagnostic(forward:)`: per the mac-editor-diagnostics lane's diff,
  steps through `editorMarkReport.marks` with `EditorDiagnostics.step`
  (exact mark identity via `currentDiagnosticID`, marks under edited text
  withheld and counted, "Error n of m, line l: message — recovery"
  announcements). Extension from this lane: when the step would wrap at
  either end of the active document and another open document has marks,
  the step continues into that document (project order, wrapping) and
  switches to it, noting "(in chapter.tex)"; withheld marks never cause a
  switch. `Navigation.stops`/`nextStop` deleted (unused).
- `CaretSync.Index`: start-sorted entries with a prefix maximum of ends;
  build O(n log n), query O(log n + m) where m = hits plus earlier entries
  shadowed by a longer span (compiler items are disjoint per word, so m = hits).
  Measured in `CaretSyncIndexTests.testTenThousandItemsQueryIsLogarithmic`
  (debug build, this Mac): 10 000 items, build 5.6 ms, **1.16 µs per caret
  query** over 20 000 queries vs ~3.0 ms per query for the linear scan
  (extrapolated from 200); the test asserts the index is ≥20× faster and
  equal in results. `CaretSync.byteOffset(ofCaretUTF16:in:)` rounds a
  mid-surrogate caret to the cluster start and refuses out-of-range carets.
  `ShellModel.caretIndex()` / `exactCaretItems` memoize the index per model
  (weak-keyed `NSMapTable`, key = result id + revision + path + page/item
  counts) until the parent moves it into a stored property.
- `revealCaretInPreview` uses the index and reports extra pages when a span
  repeats across pages.

## Validation

- `swift test` in `apps/mac` with the four real worker binaries
  (`FLASHTEX_COMPILER/PDF/BRIDGE/EDIT_LEDGER` from the main checkout's
  `crates/*/target/release`): **222 tests, 1 skipped
  (`PreviewControllerTests`, helper binary not built here), 0 failures** at
  8feec0d (and 6030201). Before this lane: 185 tests + 7 skipped at 6b43a3a.
- New tests: `NavigationExactnessTests` (11), `NavigationRealCompilerTests`
  (1, runs against the real compiler when `FLASHTEX_COMPILER` is set — it
  ran, not skipped), `CaretSyncIndexTests` (6); rewritten diagnostics tests
  in `NavigationTests`.
- Multi-file fixture (`MultiFileFixture` in NavigationTests.swift): entry
  `main.tex` + `\input{chapter}` with `naïve`, the `ﬁ` (U+FB01) ligature
  glyph, `e` + U+0301, a ZWJ emoji family, `\r\n` line ends, and `ﬃcient`,
  `Résumé`, an unterminated `\textbf{` in `chapter.tex`. Item spans are the
  ones `flashtex-compiler` (release build in the main checkout, 2026-09-12)
  emitted; `NavigationRealCompilerTests` re-derives them and fails if the
  compiler drifts. Stale-selection cases covered: `ﬁle` → `file` (ligature
  glyph → ASCII), `café` → `cafe` + U+0301 (normalization-only edit), `\r\n`
  → `\n`, an emoji scalar swapped inside the family, an edit after the span
  (rebased by −1 byte and verified), spans in the untouched other document.
- One full-suite run (fulltest2) saw
  `CompletionTests.testCompletionOnOneMegabyteBufferIsFast` at 22.2 ms vs
  its 20 ms bound while other agents were building; it passes alone (0.7 s
  wall) and in the two later full runs. Timing test in another lane.
- With the parent diff below trial-applied locally (then reverted, not
  committed): 222 tests, 0 failures — the parent's `ShellModelTests` and
  `RealCompilerTests` navigation expectations still hold.

## Incomplete / limitations / needs from others

- Until the parent applies the diff below, the live preview click and the
  diagnostics-list "Go to source" still go through `ShellModel.navigate`
  (canonical-equality staleness, no cluster widening, no caret follow) and
  the preview highlight through the O(n) `caretItems`. The menu actions
  (⌘⇧D, ⌘⇧], ⌘⇧[, ⌘⇧J) already use the exact routes.
- No live-app click evidence: Accessibility permission is not granted on this
  Mac, so a preview click cannot be scripted; the behaviour is verified
  through `ShellModel` with the real compiler's result only.
- Compiler finding (not this lane, crates/compiler): every "will not survive
  PDF export" glyph warning carries the *same* source span (`main.tex`
  66..<71, the first offending item) — including the U+FB03 one whose glyph
  is in `chapter.tex` and the U+0301/emoji ones later in `main.tex`. The
  fixture keeps only the first (correct) one. Reported here for the compiler
  owner.
- Behaviour change to confirm with the parent: a rebased span whose item
  text differs from the source (generated text, or the contract fixture's
  "Hello FlashTeX." vs its 14-byte range) now navigates with a note instead
  of being refused.

## Exact diffs needed in parent-retained files (not applied)

`apps/mac/Sources/FlashTeXMac/ShellModel.swift` (at 8feec0d lines 215–223 and
377–415):

```diff
     var caretByte: Int? {
-        activeText.utf8ByteRange(of: NSRange(location: caretUTF16, length: 0))?.start
+        CaretSync.byteOffset(ofCaretUTF16: caretUTF16, in: activeText)
     }

     /// Preview items under the caret, as `page number -> item indices`.
     /// Empty when there is no result or the caret maps to nothing.
-    var caretItems: [Int: Set<Int>] {
-        guard let result, let byte = caretByte else { return [:] }
-        return CaretSync.indicesByPage(byte: byte, path: activePath, in: result)
-    }
+    var caretItems: [Int: Set<Int>] { exactCaretItems }
```

```diff
     /// Converts the contract's UTF-8 byte range to a UTF-16 selection in the
-    /// matching document and asks the editor to select it.
+    /// matching document and asks the editor to select it. See
+    /// `navigateExactly` (Navigation.swift) for the exactness guarantees.
     func navigate(to source: RuntimeV1.SourceRange?) {
-        guard let source else {
-            navigationNote = "This item has no source mapping."
-            return
-        }
-        navigate(to: source, expectedText: nil)
+        navigateExactly(to: source, expectedText: nil)
     }

-    /// `expectedText` (an item's text) lets a rebased range be verified.
+    /// `expectedText` (an item's text) is reported when it differs from the
+    /// selected source (generated text such as a section number).
     func navigate(to source: RuntimeV1.SourceRange, expectedText: String?) {
-        ... (whole body, 27 lines) ...
+        navigateExactly(to: source, expectedText: expectedText)
     }
```

Optional, once applied: move the `caretIndexCache` map table in
CaretSync.swift into `@ObservationIgnored private var caretIndexCache:
(key: …, index: CaretSync.Index)?` on `ShellModel` (same key) and delete the
map table. `ContentView` needs no change (it calls `model.navigate`).

## Reviewed peer revisions / adaptations

- `origin/agent/mac-claude-a/mac-shell` 6b43a3a (base), b973b89
  (preview-cache + editor-diagnostics integration: adopted
  `editorMarkReport` / `currentDiagnosticID` in `goToDiagnostic` as the
  coordinator asked), 71675cd (coordination only). `origin/main` 254f662
  (no apps/mac or contract change beyond what mac-shell carried).
- `origin/agent/mac-editor-diagnostics/exact-marks` ef4ea4c via mac-shell:
  its Navigation.swift diff applied verbatim, then extended for multiple
  documents; its `clusterAlignedNSRange` and this lane's `editorRange` agree
  on cluster widening (NSString composed-sequence semantics for the
  selection, Swift `Character` semantics for the underline; both never split
  a scalar).

## Resources / next

- Resource pool: shared Claude Max 20x quota with parent mac-claude-a; no
  purchases; usage totals unknown to this worker.
- Running commands: none. Pending messages: none unanswered.
- Exact next step for the parent: apply the ShellModel diff above, run
  `swift test` in `apps/mac` with the worker binaries, merge
  `agent/mac-navigation/exact` into `mac-shell`.
- Resume reading list: this file, `Navigation.swift` header comments,
  `CaretSync.swift` `Index` doc comment, `NavigationTests.swift`
  `MultiFileFixture`.
