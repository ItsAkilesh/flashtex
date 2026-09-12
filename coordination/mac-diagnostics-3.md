# mac-diagnostics-3 (Claude Code subagent, parent mac-claude-a)

- Updated UTC: 2026-09-12T16:46Z
- Agent / parent / machine alias: mac-diagnostics-3 / mac-claude-a / mac-m1max-a
- Task: Commander replenishment (issue #2 comment 5646989044) item 9
  diagnostics follow-up: (1) grouped-row spoken label with count + occurrence
  and within-group keyboard stepping; (2) diagnostics panel keyboard
  traversal test off-screen; (3) "Copy as text" for the selection.
- Branch: `agent/mac-diagnostics-3/a11y-grouping` from
  `origin/agent/mac-claude-a/mac-shell` 82749c26.
- Owned (new files preferred): `apps/mac/Sources/FlashTeXMac/DiagnosticsPanel.swift`,
  `apps/mac/Sources/FlashTeXMac/ShellModel+DiagnosticGroups.swift`,
  `apps/mac/Sources/FlashTeXAccessibility/AccessibilityViews.swift`
  (`DiagnosticRowAccessibility` group info only), `AccessibilityCommands.swift`
  (new command-table rows), `Navigation.swift` (two Navigate menu items only —
  the navigation lane is finished/merged), tests under
  `apps/mac/Tests/FlashTeXMacTests/DiagnosticsPanel*Tests.swift` and
  `apps/mac/Tests/FlashTeXAccessibilityTests/DiagnosticGroupAccessibilityTests.swift`,
  `apps/mac/README.md` shortcut rows, this handoff,
  `coordination/agents/mac-diagnostics-3.json`. Parent-retained files
  (`ContentView.swift`, `FlashTeXMacApp.swift`, `ShellModel*.swift` core,
  `SourceEditorView.swift`) are diff requests only.

## Coverage audit (done first, 16:42–16:46Z)

Sources read: `FlashTeXAccessibility/AccessibilityViews.swift`
(`DiagnosticRowAccessibility`, `accessibleDiagnostic`),
`EditorDiagnosticsAccessibility.swift`, `AccessibilityCommands.swift`
(command table, `FocusOrder`, `PanelFocusOrder`), `EditorDiagnostics.swift`
285–380 (`Group`, `groups(of:)`, `occurrence`, `occurrenceLabel`),
`ContentView.swift` 350–410 (`diagnosticsList`: `List(groups)`, "N places"
menu, `accessibleDiagnostic(d, index: i, total: diags.count …)` with
`i = g.first`), `Navigation.swift` 629–670 / 717–730 (`goToDiagnostic`,
`NavigationCommands`), `ShellModel.swift` (`Selection.token`,
`currentDiagnosticID`, `navigate(to:)`), README 753–800 (shortcut table),
`coordination/mac-diagnostics-2.md` (limitations section names this exact
follow-up), `tools/native-validation/mac-live/reports/20260912T110944Z.md`
(diagnostics only as counts), `docs/evidence/*` (no panel-keyboard or
clipboard evidence). `grep -rn NSPasteboard apps/mac` hits only
`ProposalPreview.swift` (capture amendments), nothing for diagnostics.

Already covered (file:test) — NOT redone:

- Row label/value/hint/action for an UNGROUPED row:
  `FlashTeXAccessibilityTests/EditorDiagnosticsAccessibilityTests.testRowLabelValueAndActionsFollowTheResultStatus`,
  `.testRowAgreesWithNavigatorAnnouncementsAndDocumentModel`;
  through NSAccessibility in an off-screen host:
  `OverlayTests.testDiagnosticRowsExposeLabelValueAndActionThroughNSAccessibility`;
  document-model element `DocumentModelTests.testDiagnosticsAreActionable`.
- Grouping model (count, document order, per-occurrence jump label):
  `FlashTeXMacTests/EditorDiagnosticsPartialOutputTests.testHW1GroupsIdenticalDiagnosticsWithCountAndPerOccurrenceJump`,
  `.testGroupingRulesAcrossSeverityDocumentsAndUnsourced`.
- Next/previous diagnostic (⌘⇧] / ⌘⇧[) incl. menu wiring and README parity:
  `EditorDiagnosticsTests.testKeyboardNavigationOrderWrapAndAnnouncement`,
  `NavigationTests.testNextPreviousDiagnosticCyclesWrapsAndSkipsMarksUnderEditedText`,
  `CommandTableTests.testCommandTableMatchesREADMEShortcuts`,
  `.testMenuItemsMatchTheShellWiring`.
- Off-screen keyboard traversal harness (never-key `NSHostingView`, focus
  walk, editor keeps first responder): `PanelAccessibilityTests` for
  Settings / Durable History / Find in Project — the diagnostics panel is
  NOT among them (`PanelFocusOrder.panels` = 4 panels, none is Diagnostics).

Uncovered (implemented by this lane):

1. Grouped row's spoken label carries no count/occurrence — `ContentView`
   passes `index: g.first, total: diags.count` (confirmed by
   mac-diagnostics-2's own limitation note). No within-group next/previous.
2. No diagnostics-panel keyboard test (Tab order, Return jumps, Esc returns
   the keyboard to the editor); the panel's `List` has no selection, so
   Return/Esc have nothing to act on today.
3. No "Copy as text" for diagnostics; no clipboard-format test.

## Checkpoint

- Branch `agent/mac-diagnostics-3/a11y-grouping` @ 82749c26 (no commits yet).
- Dirty: this handoff. Next: registration JSON; implement 1–3; tests; build.
- Consumed main: as in mac-shell 82749c26. Resource: shared Claude Max 20x
  quota with parent; no purchases. Load at start: 12.9 (1-min).
