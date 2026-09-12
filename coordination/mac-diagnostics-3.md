# mac-diagnostics-3 (Claude Code subagent, parent mac-claude-a)

- Updated UTC: 2026-09-12T18:32Z (resumed after a quota cut at ~17:25Z)
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

## What this lane added (commit 084523e7, merged forward in bf056974)

1. **Grouped-row spoken label + within-group stepping.**
   `DiagnosticRowAccessibility.GroupInfo` (FlashTeXAccessibility) appends
   "12 places, 3 of 12, main.tex line 41" to the row label;
   `accessibleDiagnostic(..., group:)` carries it. Command table rows
   Next/Previous Occurrence (⌘⌥] / ⌘⌥[, Navigate menu — `Navigation.swift`
   menu items over `ShellModel.stepOccurrence`, wrapping within the group and
   feeding `currentDiagnosticID` so ⌘⇧]/⌘⇧[ continue from there). README
   shortcut rows and the panel paragraph are in parity (CommandTableTests).
2. **Panel keyboard traversal.** `DiagnosticsListView` (new
   `DiagnosticsPanel.swift`) gives the list a selection
   (`DiagnosticsPanelState`), Return → `goToSelectedOccurrence`, Esc →
   `returnKeyboardToEditor` (re-applies the caret selection with a new token so
   `SourceEditorView` takes first responder and announces the line/column).
   Off-screen test hosts the real editor + panel in a never-key window.
3. **Copy as text.** `EditorDiagnostics.copyLine/copyText`
   ("path:line: error|warning: message"; `path:byte<n>:` without the compiled
   text; `-:0:` unsourced; selected group's occurrences in document order, or
   everything), `ShellModel.copyDiagnosticsAsText` (injectable `NSPasteboard`),
   `DiagnosticsCommands` (Edit > Copy Diagnostics as Text, ⌘⌥C) and
   `.copyable` on the focused list (⌘C).

## Measured results (resumed session, 18:25–18:30Z, on the `-applied` branch 9d133437)

- `swift build` (apps/mac): Build complete, 102.6 s, load 24 (1-min) at start.
- `swift test --filter 'DiagnosticsPanelTests|DiagnosticGroupAccessibilityTests|CommandTableTests'`:
  **17 executed, 0 failures, 0 skipped** — DiagnosticsPanelTests 7/7 (the
  off-screen traversal test `testPanelKeyboardTraversalReturnJumpsAndEscReturnsTheEditor`
  ran, 1.39 s, not skipped), DiagnosticGroupAccessibilityTests 2/2,
  CommandTableTests 8/8 (README/command-table/menu-wiring parity).
- Neighbouring suites with real helpers (`FLASHTEX_COMPILER`, `_PREVIEW_CONTROLLER`,
  `_PDF`, `_BRIDGE`, `_EDIT_LEDGER`, `_PROJECT_FILES`, `_ASSISTANT_CONTEXT`
  from the main checkout's release builds):
  `--filter 'EditorDiagnostics|NavigationTests|OverlayTests|PanelAccessibilityTests|DocumentModelTests'`
  → **61 executed, 0 failures, 3 skipped** (pre-existing skips: two
  `FLASHTEX_EXPLAIN` helper tests, and `OverlayTests.testDiagnosticRowsExpose…`
  whose in-process AX host exposes 0 rows — same skip at 82749c26).
- Full `swift test` NOT run: 1-min load was 24–78 during this window (>15).
- Lane tests contain no timing assertions; no MacTeX involvement.

## Diff requests for parent-retained files (applied locally only on
`agent/mac-diagnostics-3/a11y-grouping-applied` @ 9d133437; the lane branch
does not touch them)

```diff
diff --git a/apps/mac/Sources/FlashTeXMac/ContentView.swift b/apps/mac/Sources/FlashTeXMac/ContentView.swift
index 0693ad14..391c209c 100644
--- a/apps/mac/Sources/FlashTeXMac/ContentView.swift
+++ b/apps/mac/Sources/FlashTeXMac/ContentView.swift
@@ -348,65 +348,9 @@ private struct PreviewPane: View {
     }
 
     private func diagnosticsList(_ diags: [RuntimeV1.Diagnostic]) -> some View {
-        VStack(alignment: .leading, spacing: 0) {
-            // Identical diagnostics (same severity and message) are one row with
-            // a count and a per-occurrence jump (EditorDiagnostics.groups).
-            let groups = EditorDiagnostics.groups(of: diags, documentOrder: model.documents.map(\.path))
-            Text("Diagnostics (\(diags.count)\(groups.count < diags.count ? " in \(groups.count) groups" : "")) — the preview above is still shown; errors are not hidden")
-                .font(.caption.bold()).padding(.horizontal, 8).padding(.vertical, 4)
-            if let carried = model.editorMarkReport.carried {
-                Text("Underlines \(carried.line); the list below is the failed result's.")
-                    .font(.caption).foregroundStyle(.orange).padding(.horizontal, 8).padding(.bottom, 4)
-            }
-            List(groups) { g in
-                let i = g.first
-                let d = diags[i]
-                HStack(alignment: .top) {
-                    Image(systemName: d.severity == .error ? "xmark.octagon.fill" : "exclamationmark.triangle.fill")
-                        .foregroundStyle(d.severity == .error ? .red : .orange)
-                    VStack(alignment: .leading) {
-                        Text(g.title)
-                        if let line = EditorDiagnostics.recoveryLine(recovery: d.recovery, status: model.result?.status ?? .ok) {
-                            Text("↳ \(line)").font(.caption).foregroundStyle(d.recovery == nil ? .tertiary : .secondary)
-                        }
-                        if let explain = model.explanations.explanation(resultID: model.resultID, index: i)?.line {
-                            Text("↳ \(explain)").font(.caption).foregroundStyle(.secondary)
-                        }
-                        if let result = model.result,
-                           let id = EditorDiagnostics.identity(resultID: model.resultID, index: i, in: result),
-                           model.editorMarkReport.staleIdentities.contains(id) {
-                            Text("underline withheld: span edited since the compile").font(.caption2).foregroundStyle(.orange)
-                        }
-                        if let src = d.source {
-                            Text("\(src.path) bytes \(src.startByte)..<\(src.endByte)\(g.count > 1 ? " (first of \(g.count))" : "")").font(.caption2).foregroundStyle(.tertiary)
-                        } else {
-                            Text("no source mapping").font(.caption2).foregroundStyle(.tertiary)
-                        }
-                    }
-                    Spacer()
-                    if g.count > 1 {
-                        Menu("\(g.count) places") {
-                            ForEach(0..<g.count, id: \.self) { k in
-                                Button(EditorDiagnostics.occurrenceLabel(k, of: g, in: diags, texts: model.compiledDocuments)) {
-                                    model.navigate(to: EditorDiagnostics.occurrence(k, of: g, in: diags))
-                                }
-                                .disabled(EditorDiagnostics.occurrence(k, of: g, in: diags) == nil)
-                            }
-                        }
-                        .fixedSize()
-                        .help("Jump to one occurrence of this diagnostic")
-                    } else if d.source != nil { Button("Go to source") { model.navigate(to: d.source) } }
-                    if let x = model.explanations.explanation(resultID: model.resultID, index: i),
-                       x.suggestions.contains(where: { !$0.edits.isEmpty }) {
-                        Button("Fix…") { model.previewQuickFix(diagnosticIndex: i) }
-                            .help(x.suggestions.first { !$0.edits.isEmpty }?.text ?? "Preview a suggested fix")
-                    }
-                }
-                .accessibleDiagnostic(d, index: i, total: diags.count, status: model.result?.status ?? .ok,
-                                      explanation: model.explanations.explanation(resultID: model.resultID, index: i)?.line) { model.navigate(to: d.source) } // FlashTeXAccessibility
-            }
-            .frame(minHeight: 80, maxHeight: 180)
-        }
+        // Grouped rows with a selection, Return / Esc / ⌘C and the spoken group
+        // count/occurrence (DiagnosticsPanel.swift, mac-diagnostics-3).
+        DiagnosticsListView(diagnostics: diags)
     }
 }
 
diff --git a/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift b/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift
index c29d82e8..c33566e4 100644
--- a/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift
+++ b/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift
@@ -97,6 +97,7 @@ struct FlashTeXMacApp: App {
         }
         .commands {
             NavigationCommands(model: model) // Navigation.swift
+            DiagnosticsCommands(model: model) // DiagnosticsPanel.swift: Edit > Copy Diagnostics as Text (⌘⌥C)
             ProjectSearchCommands(openWindow: openWindow) // ProjectSearchPanel.swift: ⌘⇧F Find in Project…
             CitationRenameCommands(openWindow: openWindow) // CitationRename.swift: Edit > Rename Citation… (no shortcut)
             CommandGroup(after: .toolbar) {
```

## Limitations

- Within-group stepping without a panel selection falls back to the group under
  the caret's current diagnostic, else the first multi-place group; single-place
  groups report "Only one place".
- `.copyable` on the list only serves ⌘C while the list is focused; the Edit
  menu item (⌘⌥C) works from anywhere the panel is in the scene.
- The off-screen traversal test drives NSTableView key events; VoiceOver's
  own reading was not exercised (no Accessibility permission).
- The grouped-row "N places" menu's per-occurrence jump still goes through
  `goToOccurrence`, so the spoken "k of n" follows the last jump.

## Checkpoint

- Lane branch `agent/mac-diagnostics-3/a11y-grouping` @ bf056974 = 084523e7
  (implementation) + merge of parent tip 9ba9851c (conflict resolved in
  `AccessibilityCommands.swift` Command enum: kept completion-2's
  `completionList` plus this lane's three cases). Pushed to origin.
- `-applied` branch @ 9d133437 = lane + the two parent-retained hooks above
  (local only, never pushed as the lane).
- Dirty: none after the handoff/registration commit. Next: parent integrates the
  lane branch into mac-shell and applies the two hooks; lane is DONE.
- Consumed main: as in mac-shell 9ba9851c. Resource: shared Claude Max 20x
  quota with parent; no purchases. Loads: 10.5 (start of resume), 24 (build),
  78/43/36 during tests.
