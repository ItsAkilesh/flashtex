# mac-diagnostics-2 (Claude Code subagent, parent mac-claude-a)

- Updated UTC: 2026-09-12T16:20Z
- Agent / parent / machine alias: mac-diagnostics-2 / mac-claude-a / mac-m1max-a
- Task: Commander replenishment (issue #2 comment 5646989044) — editor
  diagnostics item 9: partial-output error marking, stale diagnostic
  invalidation (verified on HW1), keyboard navigation parity; follow-up 1
  (group identical diagnostics), follow-up 2 (non-active document jump).
- Branch: `agent/mac-diagnostics-2/partial-output` from
  `origin/agent/mac-claude-a/mac-shell` cd58fc2e (main c11c005 merged).
- Owned: `apps/mac/Sources/FlashTeXMac/EditorDiagnostics.swift`,
  `apps/mac/Tests/FlashTeXMacTests/EditorDiagnostics*Tests.swift`, this
  handoff, `coordination/agents/mac-diagnostics-2.json`. Parent-retained
  files (`ShellModel*.swift`, `ContentView.swift`, …) are diff requests only.

## Coverage audit (≤10 min, done first)

Sources read: `EditorDiagnostics.swift` (744 lines), `Navigation.swift`
(`goToDiagnostic`, `navigateExactly`, `NavigationCommands`), `ShellModel.swift`
(`editorMarkReport`, worker `compileResult` binding), `ContentView.swift`
(`diagnosticsList`, `Footer`), `apps/mac/README.md` 716–722 and 758,
`coordination/mac-editor-diagnostics.md`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md` (diagnostics
only appear as counts/one warning payload: lines 319, 490, 501–502),
`docs/evidence/*` (no diagnostics-marking evidence beyond the mac-live report).

Already covered (file:test):

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
- Keyboard navigation (⌘⇧] / ⌘⇧[): `Navigation.swift:619 goToDiagnostic`,
  menu `NavigationCommands` (Navigate > Next/Previous Diagnostic), README
  716 (prose) and 758 (command table) — parity already present;
  `EditorDiagnosticsTests.testKeyboardNavigationOrderWrapAndAnnouncement`,
  `.testNavigationVisitsMarksSharingAStart`,
  `NavigationTests.testNextPreviousDiagnosticCyclesWrapsAndSkipsMarksUnderEditedText`,
  `.testDiagnosticNavigationWithoutResultOrSourcesExplains`,
  `EditorDiagnosticsAccessibilityTests` (5 tests: order, wrap, "n of m"
  announcement, row/navigator agreement). Announcement text is composed by
  `EditorDiagnosticNavigation.Step.announcement` and shown in the footer.
- Follow-up 2 (diagnostics in a non-active document, open on jump): ALREADY
  COVERED. `ShellModel.documents` is the exact membership the compiler sees
  (`ProjectDocuments.swift` header), so a diagnostic can only name an open
  document; `goToDiagnostic` steps into the next open document with marks
  and switches `activePath` (`NavigationTests.testDiagnosticsCycleAcrossDocumentsInProjectOrder`),
  and "Go to source" → `navigateExactly` switches documents
  (`NavigationTests` line 474 "switched to", line 490 "No open document named").
  No work manufactured for this follow-up.

NOT covered (this lane's work):

1. Partial output on a real result: no test runs the live compiler over a
   document where it skipped regions and checks every mark. HW1
   (`fixtures/real-world/hw1/HW1.tex`, 5126 bytes) compiles to `recovered`,
   2 pages, 119 sourced diagnostics (12× `\in`, 11× `\mathbb`, 10× `\forall`,
   7× `\subsection requires a braced argument`, `\setlength`/`\parindent`
   "skipped the command and did not typeset preamble content", …); measured
   with `crates/compiler/target/release/flashtex-compiler` (main checkout,
   built 11:23Z). Zero existing tests reference hw1.
2. `failed` result with no pages: `ShellModel` binds `result = incoming`
   unconditionally, so a `failed` result (the compiler emits pages `[]`
   and usually one unsourced error, `crates/compiler/src/protocol.rs:427`)
   makes `editorMarkReport` return no marks — the previous marks are
   CLEARED instead of kept and flagged. No test covers it.
3. Stale invalidation across a SEQUENCE of edits (N → N+1 → N+2 with no
   result in between) on a real 119-mark result: `report` uses one covering
   `changedRegion` (conservative); not verified with real spans.
4. Follow-up 1 (grouping identical diagnostics with a count and
   per-occurrence jump): nothing exists in the panel or model.

## Checkpoint

- Branch `agent/mac-diagnostics-2/partial-output`, base cd58fc2e; consumed
  main c11c005 (via mac-shell). Dirty files: this handoff, the agent record.
- Helpers: main-checkout release builds (`/Users/jay3332/Projects/flashtex/crates/*/target/release/*`);
  `flashtex-explain` is not built anywhere (explanations tests skip).
- Next commands: write `EditorDiagnosticsPartialOutputTests.swift` (HW1,
  real compiler, XCTSkip without `FLASHTEX_COMPILER`); add retention +
  grouping APIs to `EditorDiagnostics.swift`; ShellModel/ContentView diffs
  in this handoff; `swift build`, run the new file; full `swift test` only
  if 1-min load < 15 (16:04Z load 8.55/10.28/9.00).
- Resource: shared Claude Max 20x quota with parent; no purchases.
