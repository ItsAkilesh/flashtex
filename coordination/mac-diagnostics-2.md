# mac-diagnostics-2 (Claude Code subagent, parent mac-claude-a)

- Updated UTC: 2026-09-12T16:50Z
- Agent / parent / machine alias: mac-diagnostics-2 / mac-claude-a / mac-m1max-a
- Task: Commander replenishment (issue #2 comment 5646989044) — editor
  diagnostics item 9: partial-output error marking, stale diagnostic
  invalidation (verified on HW1), keyboard navigation parity; follow-up 1
  (group identical diagnostics), follow-up 2 (non-active document jump).
- Branch: `agent/mac-diagnostics-2/partial-output` (tip 90e8d6d6) from
  `origin/agent/mac-claude-a/mac-shell` cd58fc2e (main c11c005 merged).
  Labelled local application of the parent-file hooks:
  `agent/mac-diagnostics-2/partial-output-applied` (afc8adf9 = lane tip +
  one commit touching parent-retained files; compile/test only, not for merge
  as-is).
- Owned: `apps/mac/Sources/FlashTeXMac/EditorDiagnostics.swift`,
  `apps/mac/Tests/FlashTeXMacTests/EditorDiagnostics*Tests.swift` (new:
  `EditorDiagnosticsPartialOutputTests.swift`), `apps/mac/README.md`
  diagnostics paragraph, `docs/evidence/mac-diagnostics-2-hooks-2026-09-12.diff`,
  this handoff, `coordination/agents/mac-diagnostics-2.json`. Parent-retained
  files are diff requests only (section "Requested diffs").
- State: ready for integration (lane branch); parent applies the hooks diff.

## Coverage audit (done first, ≤10 min)

Sources read: `EditorDiagnostics.swift` (744 lines at cd58fc2e),
`Navigation.swift` (`goToDiagnostic`, `navigateExactly`, `NavigationCommands`),
`ShellModel.swift` (`editorMarkReport`, worker/fixture binding),
`ShellModel+Controller.swift` (controller binding), `ContentView.swift`
(`diagnosticsList`, `Footer`), `apps/mac/README.md` 716–722 and 758,
`coordination/mac-editor-diagnostics.md`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md` (diagnostics
appear only as counts and one warning payload: lines 319, 490, 501–502),
`docs/evidence/*` (no diagnostics-marking evidence beyond the mac-live report),
`crates/compiler/src/protocol.rs:427` (`failed` = pages `[]`, usually one
unsourced error).

Already covered (file:test) — NOT redone:

- Marks from byte spans, exact identity, recovered/failed status lines:
  `EditorDiagnosticsTests.testSourcedErrorMarksAndUnsourcedWarningDoesNot`,
  `.testIdentityDistinguishesDuplicatesAndSurvivesRebase`,
  `.testRecoveredResultsAlwaysShowRecoveryAndKeepErrors`,
  `.testOtherPathProducesNoMark`, `.testMultipageSampleErrorMarkSlicesToItsText`
  (one sourced error from the multipage fixture), `.testMarksNeverSplitGraphemeClusters`.
- Rebase/withhold across ONE edit (synthetic text):
  `EditorDiagnosticsTests.testMarksRebaseAcrossPrefixEditAndDropWhenEditOverlaps`,
  `.testOnlyOverlappingMarksGoStale`, `.testRebaseOfLargeDocumentIsFast`;
  shell level `NavigationTests.testNextPreviousDiagnosticCyclesWrapsAndSkipsMarksUnderEditedText`.
- Keyboard navigation (⌘⇧] / ⌘⇧[) with caret move + announcement:
  `Navigation.swift:619 goToDiagnostic` (selection, caret, `navigationNote` =
  `EditorDiagnosticNavigation.Step.announcement` "Error n of m, line L: …"),
  menu `NavigationCommands` (Navigate > Next/Previous Diagnostic), README 716
  (prose) and 758 (command table) — parity already present; tests
  `EditorDiagnosticsTests.testKeyboardNavigationOrderWrapAndAnnouncement`,
  `.testNavigationVisitsMarksSharingAStart`,
  `NavigationTests.testNextPreviousDiagnosticCyclesWrapsAndSkipsMarksUnderEditedText`,
  `.testDiagnosticNavigationWithoutResultOrSourcesExplains`,
  `EditorDiagnosticsAccessibilityTests` (5). Not manufactured again; the new
  HW1 test additionally checks the first stop's announcement on a real result.
- Follow-up 2 (diagnostic in a non-active document, open on jump): ALREADY
  COVERED. `ShellModel.documents` is the exact membership the compiler sees
  (`ProjectDocuments.swift` header), so a diagnostic can only name an open
  document; `goToDiagnostic` steps into the next open document with marks and
  switches `activePath` (`NavigationTests.testDiagnosticsCycleAcrossDocumentsInProjectOrder`),
  and "Go to source" → `navigateExactly` switches documents (`NavigationTests`
  line 474 "switched to", line 490 "No open document named"). No work done.

Uncovered before this lane (now implemented):

1. Partial output on a REAL result with skipped regions — no test ran the
   live compiler over such a document; zero tests referenced HW1.
2. `failed` result with no pages: `ShellModel` binds `result = incoming`
   unconditionally, so every underline was CLEARED on a failed compile and
   came back on the next success (no test).
3. Stale invalidation across a SEQUENCE of edits on a real 119-mark result.
4. Follow-up 1 (grouping identical diagnostics with count + per-occurrence
   jump): nothing existed.

## Measured: what the live compiler does with HW1

`crates/compiler/target/release/flashtex-compiler` (main checkout, built
2026-09-12 11:23Z) on `fixtures/real-world/hw1/HW1.tex` (5126 bytes, ASCII):
status `recovered`, 2 pages, **119 diagnostics, all sourced, sorted by
start**; 12× `\in is not supported in math mode`, 11× `\mathbb`, 10×
`\forall`, 7× `\subsection requires a braced argument`, 7× `\hfill`, 7×
`\normalfont`, 6× `\exists`, 5× `\text`/`\qquad`/`\bigl`/`\bigr`, …;
`\setlength`/`\parindent`/`\parskip` "skipped the command and did not
typeset preamble content".

Finding (not a bug, documented in the README): diagnostics raised while
expanding a user macro are reported at the **call site** — every `\problem`
use (line 18: `\newcommand{\problem}[2]{\subsection*{Problem #1 \hfill
\normalfont[#2 points]}}`) carries three diagnostics on one 8-byte span
(`\subsection requires a braced argument`, `\hfill`, `\normalfont`), and
`\Z`/`\R`/`\Q` carry `\mathbb`. The marks map to the right bytes (the macro
invocation); the message names the expanded command. The test's oracle
accepts a mark iff its text starts with the named command or is a
`\newcommand` whose body contains it: 119/119 pass (80+ direct, 20+ via
macro, 10+ inside skipped regions).

## Ready behavior (lane branch 90e8d6d6)

- `EditorDiagnostics.Mark.carried: Carried?` (`revision`, `failedRevision`,
  `line` = "kept from revision N: revision M failed with no output"), appended
  to `toolTip` ("↳ …") and `spokenDescription` (" — …").
- `Report.carried`; `Report.staleNote` now = carried line ("119 underlines
  kept from revision 1: revision 2 failed with no output"), the withheld count
  (`withheldNote`, unchanged wording), or both joined with "; ".
- `Retained { resultID, result, compiledDocuments }`,
  `keepsPreviousMarks(result)` = `status == .failed && pages.isEmpty` (a
  `failed` WITH pages, and `ok`/`recovered` with no pages — an empty document —
  replace), `retained(after:resultID:compiledDocuments:previous:)` (never
  chains through failures), `report(for:resultID:retained:path:compiledText:currentText:)`:
  fresh marks of the failed result (a sourced refused span) first, then the
  retained result's marks rebased from ITS compiled text and flagged; stale
  sets concatenated; otherwise identical to the plain `report`.
- `attach(_:carried:to:)`: carried marks take explanation lines from the
  retained result's cache entry, never the newest result's (index mismatch).
- `Group { severity, message, recovery (shared or nil), occurrences }`,
  `groups(of: result|diagnostics, documentOrder:)` keyed by (severity,
  message), occurrences in document order (project path order, start byte,
  unsourced last), groups ordered by first occurrence; `title` "12× …";
  `occurrence(k, of:in:)`, `occurrenceLabel` "3 of 12: main.tex line 41" /
  "bytes a..<b" / "no source"; `lineNumber(ofByte:in:)`.
- README diagnostics paragraph extended (partial output, retention, grouping).

## Tests and evidence

`EditorDiagnosticsPartialOutputTests` (6; 4 need `FLASHTEX_COMPILER`, XCTSkip
with the build hint otherwise — verified: "Executed 6 tests, with 4 tests
skipped and 0 failures" without the env var). HW1 is compiled behind a
multi-byte first line (`% naïve “HW1” — 👩‍💻 …`) so every UTF-16 offset differs
from its byte offset:

- `testHW1PartialOutputMarksSliceToTheNamedCommand`: recovered, ≥1 page, 119
  sourced diagnostics → 119 marks, 0 stale; each mark's UTF-16 substring is
  byte-identical to the reported span and starts with the named command or
  its macro; `\setlength` inside the skipped preamble maps exactly; three
  distinct marks per `\problem` call site; first navigation stop =
  `\usepackage[T1]{fontenc}` on line 4 with announcement "Warning 1 of 119,
  line 4: packages fontenc are recognised but not implemented — recovery: …".
- `testHW1MarksRebaseOntoTheSameBytesOrAreWithheldThroughEdits`: invariant at
  every revision (mark text == its revision-1 text, withheld ⇒ overlaps the
  covering edit, marks+withheld == 119, ids unique) through: prefix insert
  (119 shifted, 0 withheld); prefix + 5th `\in` → `\notin` with no compile
  between (the 5th occurrence withheld; the single covering region also
  withholds the marks between the two edits — conservative, never
  misplaced); middle-line deletion (exactly the diagnostics on that line
  withheld); append (ranges identical to base).
- `testHW1FailedFollowUpKeepsMarksFlaggedNotClearedNorDuplicated`: real
  `failed` result (empty project, revision 2: pages `[]`, one unsourced
  error) → without retention 0 marks; with it 119 flagged, same ranges as the
  good report, staleNote/toolTip/spoken lines as above; an edit rebases them
  from the retained text; a second failure (revision 3) shows them once with
  `failedRevision: 3`; a failed result with its own sourced diagnostic gives
  1 fresh + 119 carried; a new good result (revision 5) replaces (carried nil,
  ids `hw1-5`); an empty document (`ok`, 0 pages) clears.
- `testRetentionRuleIsFailedWithNoPagesOnly`, `testGroupingRulesAcrossSeverityDocumentsAndUnsourced`
  (no compiler): rule table; severity splits groups, project order, unsourced
  last, mixed recoveries → nil, labels, line numbers, empty result.
- `testHW1GroupsIdenticalDiagnosticsWithCountAndPerOccurrenceJump`: groups
  partition the 119; `\in` group count 12, error, title "12× \in is not
  supported in math mode", shared recovery, occurrences ascending and
  distinct, each slices to `\in`, labels "k of 12: main.tex line L" (L > 1),
  out-of-range nil, first group = the fontenc warning.

Runs (lane branch, debug, `FLASHTEX_NO_ACTIVATE=1`, `FLASHTEX_COMPILER`
main-checkout release; 1-min load 22–46 during the runs, so NO full `swift test`
— the brief's < 15 condition was never met):

- `swift build --build-tests`: Build complete.
- filter `EditorDiagnostics`: 33 tests, 2 skipped (`FLASHTEX_EXPLAIN` helper
  not built anywhere on this machine), 0 failures.
- filter `HistoricalPreviewTests|SourceEditorViewTests|EditorDiagnostics|NavigationTests|EditorDiagnosticsAccessibilityTests`
  (every suite that reads `staleNote`/`editorMarkReport`/`EditorDiagnostics.`):
  **72 tests, 2 skipped, 0 failures** in 10.8 s.
- Applied branch afc8adf9: `swift build --build-tests` Build complete;
  `EditorDiagnosticsRetentionShellTests` (1, shell-level with real HW1 +
  real failed results: kept 119, navigation announces "Warning 1 of 119, line
  4: …; 119 underlines kept from revision 1: revision 2 failed with no
  output", edit rebases, second failure once, new result replaces,
  `replaceProject` forgets) passed; with `EditorDiagnosticsPartialOutputTests`,
  `EditorDiagnosticsExplanationsTests`, `NavigationTests`: 19 tests, 1
  skipped, 0 failures.

## Requested diffs (parent-retained files; applied only on `…-applied` afc8adf9)

Full unified diff (268 lines, `git diff 81960dff afc8adf9`):
`docs/evidence/mac-diagnostics-2-hooks-2026-09-12.diff`. Summary per file:

`apps/mac/Sources/FlashTeXMac/ShellModel.swift`

```diff
@@ editorMarkReport
-        let key = EditorMarksKey(resultID: resultID, resultRevision: result.revision, editorRevision: editorRevision, path: activePath,
-                                 explanationsCount: explanations[resultID]?.count ?? -1)
+        let key = EditorMarksKey(resultID: resultID, resultRevision: result.revision, editorRevision: editorRevision, path: activePath,
+                                 explanationsCount: explanations[resultID]?.count ?? -1,
+                                 carriedExplanationsCount: explanations[retainedMarks?.resultID]?.count ?? -1)
         if let cached = editorMarksCache, cached.key == key { return cached.report }
-        let report = EditorDiagnostics.attach(explanations[resultID], to: EditorDiagnostics.report(
-            for: result, resultID: resultID, path: activePath,
-            compiledText: compiledDocuments[activePath], currentText: activeText))
+        // Retention: a failed result with no pages keeps the last marks, flagged
+        // (ShellModel+DiagnosticRetention.swift); carried marks take their
+        // explanation lines from the retained result's cache entry.
+        let report = EditorDiagnostics.attach(explanations[resultID], carried: explanations[retainedMarks?.resultID],
+                                              to: diagnosticReport(for: activePath, currentText: activeText))
         editorMarksCache = (key, report)
         return report
     }
-    private struct EditorMarksKey: Equatable { var resultID: String?; var resultRevision: Int; var editorRevision: Int; var path: String; var explanationsCount: Int }
+    /// The last result that produced output (`EditorDiagnostics.Retained`),
+    /// kept while a later result fails with no pages; see `retainMarksAfterResultBound`.
+    var retainedMarks: EditorDiagnostics.Retained?
+    private struct EditorMarksKey: Equatable {
+        var resultID: String?; var resultRevision: Int; var editorRevision: Int; var path: String
+        var explanationsCount: Int; var carriedExplanationsCount: Int
+    }
@@ loadFixtures (after the `else { … compiledDocuments = [:] }` block)
+            retainMarksAfterResultBound() // ShellModel+DiagnosticRetention.swift
             selection = nil
             navigationNote = nil
@@ replaceProject
         result = nil
         resultID = nil
+        retainedMarks = nil
@@ worker compileResult binding
             compiledDocuments = Dictionary(uniqueKeysWithValues: sent.documents.map { ($0.path, $0.text) })
+            retainMarksAfterResultBound() // ShellModel+DiagnosticRetention.swift
             fetchExplanations(for: incoming, id: env.id, documents: sent.documents)
```

`apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift`

```diff
         setCompiledDocuments(compiled)
+        retainMarksAfterResultBound() // ShellModel+DiagnosticRetention.swift
```

`apps/mac/Sources/FlashTeXMac/Navigation.swift` (`goToDiagnostic`, other documents):

```diff
-                let other = EditorDiagnostics.report(for: result, resultID: resultID, path: doc.path,
-                                                     compiledText: compiledDocuments[doc.path], currentText: doc.text)
+                let other = diagnosticReport(for: doc.path, currentText: doc.text) // retention rule applied (ShellModel+DiagnosticRetention.swift)
```

NEW `apps/mac/Sources/FlashTeXMac/ShellModel+DiagnosticRetention.swift` (needs
the `retainedMarks` stored property above, which is why it is on the applied
branch only):

```swift
import Foundation
import FlashTeXProtocol

extension ShellModel {
    /// Call after `result`, `resultID` and `compiledDocuments` are bound
    /// (fixture, worker and controller paths): updates `retainedMarks`.
    func retainMarksAfterResultBound() {
        guard let result else { retainedMarks = nil; return }
        retainedMarks = EditorDiagnostics.retained(after: result, resultID: resultID, compiledDocuments: compiledDocuments,
                                                   previous: retainedMarks)
    }

    /// The report for `path` (any open document) with the retention rule
    /// applied — what `editorMarkReport` memoizes for the active document
    /// and what diagnostic navigation reads for the others.
    func diagnosticReport(for path: String, currentText: String) -> EditorDiagnostics.Report {
        guard let result else { return .empty }
        return EditorDiagnostics.report(for: result, resultID: resultID, retained: retainedMarks, path: path,
                                        compiledText: compiledDocuments[path], currentText: currentText)
    }
}
```

`apps/mac/Sources/FlashTeXMac/ContentView.swift` (`diagnosticsList`): group
rows — see the .diff for the exact hunk; in short:

```diff
-            Text("Diagnostics (\(diags.count)) — …")
-            List(Array(diags.enumerated()), id: \.offset) { i, d in
+            let groups = EditorDiagnostics.groups(of: diags, documentOrder: model.documents.map(\.path))
+            Text("Diagnostics (\(diags.count)\(groups.count < diags.count ? " in \(groups.count) groups" : "")) — …")
+            if let carried = model.editorMarkReport.carried {
+                Text("Underlines \(carried.line); the list below is the failed result's.")
+                    .font(.caption).foregroundStyle(.orange).padding(.horizontal, 8).padding(.bottom, 4)
+            }
+            List(groups) { g in
+                let i = g.first
+                let d = diags[i]
                 …
-                        Text(d.message)
+                        Text(g.title)
                 …
-                            Text("\(src.path) bytes \(src.startByte)..<\(src.endByte)")
+                            Text("\(src.path) bytes \(src.startByte)..<\(src.endByte)\(g.count > 1 ? " (first of \(g.count))" : "")")
                 …
-                    if d.source != nil { Button("Go to source") { model.navigate(to: d.source) } }
+                    if g.count > 1 {
+                        Menu("\(g.count) places") {
+                            ForEach(0..<g.count, id: \.self) { k in
+                                Button(EditorDiagnostics.occurrenceLabel(k, of: g, in: diags, texts: model.compiledDocuments)) {
+                                    model.navigate(to: EditorDiagnostics.occurrence(k, of: g, in: diags))
+                                }
+                                .disabled(EditorDiagnostics.occurrence(k, of: g, in: diags) == nil)
+                            }
+                        }
+                        .fixedSize()
+                        .help("Jump to one occurrence of this diagnostic")
+                    } else if d.source != nil { Button("Go to source") { model.navigate(to: d.source) } }
```

The row's explanation line, "Fix…", stale caption and `accessibleDiagnostic`
keep using `i = g.first` (the first occurrence).

Also on the applied branch: `apps/mac/Tests/FlashTeXMacTests/EditorDiagnosticsRetentionShellTests.swift`
(shell-level test; move with the hooks).

## Limitations / not measured

- No live VoiceOver run (Accessibility permission not granted); the carried
  line and the "N places" menu are unit-tested text and a standard control.
  `accessibleDiagnostic` (mac-accessibility lane) still labels the row as
  the first occurrence without the count — a one-line follow-up for its owner.
- The panel groups by exact message; two `\in` diagnostics with different
  wording would not group.
- Multiple edits between compiles are still collapsed into ONE covering
  region (unchanged `SourceMapping` contract): correct (never misplaced) but
  withholds marks between two distant edits until the next result.
- Full `swift test` not run: 1-min load never dropped below 22 during the
  lane (brief: only if < 15).
- `flashtex-explain` is not built on this machine; explanation tests skip as
  before.
- No app screenshot: the grouped panel was compiled (applied branch) but not
  launched.

## Checkpoint

- Lane branch `agent/mac-diagnostics-2/partial-output` 90e8d6d6, pushed;
  applied branch `…-applied` afc8adf9, pushed. Base cd58fc2e; mac-shell has
  since moved to 2fa51729 (visual-oracle merge; not re-based, no overlap with
  owned files expected — parent to verify on merge). Consumed main c11c005.
- Dirty files: none after the final commit. Helpers: main-checkout release
  builds. Resource: shared Claude Max 20x quota with parent; no purchases.
- Exact next action for the parent: merge the lane branch; apply
  `docs/evidence/mac-diagnostics-2-hooks-2026-09-12.diff` (or cherry-pick
  afc8adf9) and move `EditorDiagnosticsRetentionShellTests.swift` with it;
  run `swift test --filter 'EditorDiagnostics|NavigationTests'` with
  `FLASHTEX_COMPILER`.
