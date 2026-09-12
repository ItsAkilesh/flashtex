# mac-navigation handoff — exact source ↔ preview navigation

- Updated UTC: 2026-09-12T08:55Z
- Agent / parent / machine: `mac-navigation` (Claude Code subagent) / parent
  `mac-claude-a` / `mac-m1max-a`
- Lane: "Exact source-to-preview and preview-to-source navigation" (parent
  dispatch, issue #2). Follow-ups: "UTF8 cluster/ligature stale selection
  tests" (done, see tests), "Multi-file current revision navigation
  acceptance" (done at the model level; live-app click evidence not possible
  without Accessibility permission, see limitations).
- Owned paths: `apps/mac/Sources/FlashTeXMac/Navigation.swift`,
  `apps/mac/Sources/FlashTeXMac/CaretSync.swift`,
  `apps/mac/Tests/FlashTeXMacTests/NavigationTests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/CaretSyncTests.swift`, this file,
  `coordination/agents/mac-navigation.json`.
- Branch / worktree: `agent/mac-navigation/exact` (pushed) in
  `.claude/worktrees/agent-aa3ca1d9a088b9f09`; based on
  `origin/agent/mac-claude-a/mac-shell` 6b43a3a, merged mac-shell b973b89 and
  71675cd and `origin/main` 254f662 + c9f1b9e. Dirty files: none besides this handoff
  and the agents JSON at the time of writing.
- Rules honoured: no purchases; no edits outside owned paths (ShellModel,
  ContentView, PreviewView untouched — diffs below); transferred crates
  (font-engine, paragraph-layout, math-layout) untouched; no app launched;
  commits carry the user as primary author with truthful trailers.
- Context usage: the harness does not expose an exact percentage to this
  worker; the session-level token budget readout shows well under 20% of the
  1M-context budget consumed, so no compaction is expected. This checkpoint is
  written to the user's context-checkpoint policy regardless.

## Ready behavior (on the branch)

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
