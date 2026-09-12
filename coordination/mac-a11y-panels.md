# mac-a11y-panels handoff — keyboard-only traversal of the preferences, history and search panels

- Updated UTC: see `coordination/agents/mac-a11y-panels.json` `updated_utc`
- Agent / parent / machine alias: `mac-a11y-panels` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: Gap 6 of the parent's coverage list — keyboard-only accessibility
  traversal (Tab / Shift-Tab reachability, labels/hints/identifiers, focus
  restoration to the editor) for the Settings scene (`EditorPreferencesView`,
  ⌘,), the Durable History window (`EditHistoryPanel`) and the Find in
  Project window (`agent/mac-search/panel`, session still running — its files
  are NOT edited here; tested against a local merge only).
- Owned paths: `apps/mac/Tests/FlashTeXMacTests/PanelAccessibilityTests.swift`,
  additions to `apps/mac/Sources/FlashTeXAccessibility/AccessibilityCommands.swift`
  and `AccessibilityViews.swift`, `apps/mac/Tests/FlashTeXAccessibilityTests/CommandTableTests.swift`,
  the README shortcut table rows for the panels, this handoff and
  `coordination/agents/mac-a11y-panels.json`. Parent-retained files
  (`ShellModel*.swift`, `ContentView.swift`, `PreviewView.swift`,
  `FlashTeXMacApp.swift`, `SourceEditorView.swift`) are not committed here;
  any hook is an exact diff in the final report.
- Branch: `agent/mac-a11y-panels/a11y-panels` from
  `origin/agent/mac-claude-a/mac-shell` `5bc3fc0f` (merge-base with main
  `527ae541`; main tip at start `ffe199d8`).

## Coverage audit (mandatory first step, done 13:32Z)

Grepped `apps/mac/Tests/FlashTeXMacTests/*`, `apps/mac/Tests/FlashTeXAccessibilityTests/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md` and `docs/evidence/*`
for `accessib|nextValidKeyView|firstResponder|canBecomeKeyView|keyView|focus`.

Already covered (file: test):

- `FlashTeXMacTests/EditorPreferencesTests.swift: testSettingsViewHostsAndReflectsChanges`
  — hosts `EditorPreferencesView` off-screen, asserts the hosted tree contains
  `NSControl`s and at least one `NSButton`, and that model changes flow in.
  It does NOT walk `nextValidKeyView`, does not assert per-control labels/
  hints/identifiers, and does not check first responder.
- `FlashTeXMacTests/EditorPreferencesTests.swift: testWritesSettingsViewEvidenceWhenRequested`
  — PNG evidence only (gated on `FLASHTEX_PREFS_EVIDENCE`).
- `FlashTeXMacTests/EditHistoryTests.swift` (all tests) — wire/format/client
  behaviour of the history panel; no hosted-view or accessibility test of
  `EditHistoryPanel` (the view carries labels/hints/identifiers in source,
  untested).
- `FlashTeXMacTests/SourceEditorViewTests.swift: testEditorExposesLabelValueAndSelectedTextRangeAndAnnouncesCaretMoves`
  — editor text view AX label/value/selection; `makeFirstResponder(tv)` is
  used as the hosting pattern (never key).
- `FlashTeXMacTests/CompletionTests.swift` (lines 773–842) — completion popup
  closes on `resignedFirstResponder`; `makeFirstResponder` pattern.
- `FlashTeXAccessibilityTests/CommandTableTests.swift: testCommandTableMatchesREADMEShortcuts`,
  `testEveryCommandHasAREADMERowAndEveryRowACommand`, `testMenuItemsMatchTheShellWiring`,
  `testEntriesAreDiscoverable`, `testFocusOrderMatchesContentViewPaneOrder`,
  `testFocusOrderHelpText`, `testHelpViewCoversEveryCommandAndMenu` — README /
  command-table / menu-wiring parity for the MAIN window only. The table has
  no entry for Settings (⌘,), Durable History (Edit > Durable History…) or
  Find in Project (⌘⇧F, on the search branch), and the wiring parser reads
  only `FlashTeXMacApp.swift` and `Navigation.swift`, so the search lane's
  `ProjectSearchCommands` (`ProjectSearchPanel.swift`, `CommandGroup(after:
  .textEditing)`) is invisible to it.
- `FlashTeXAccessibilityTests/OverlayTests.swift`, `CompletionAccessibilityTests.swift`,
  `EditorDiagnosticsAccessibilityTests.swift` — preview/completion/diagnostics
  trees; not the panels.
- `FlashTeXMacTests/NearbyViewTests.swift` — Nearby window model behaviour; no
  keyboard traversal (not in this gap).
- `ProjectSearchTests.swift` (search branch, 22 tests): `testMatchesAndAccessibilityLabels`
  covers the pure row/preview label strings only; no hosted traversal.
- mac-live report `20260912T110944Z.md`: `open-window/a11y-help` proves the
  Accessibility Help window opens (CGWindowList); no keyboard traversal
  (Accessibility permission not granted, so none is possible there).
- `docs/evidence/mac-history/`: one seeded-ledger JPEG of the history panel.

Verdict: the gap is NOT covered. Uncovered and implemented here: hosted
Tab/Shift-Tab traversal of each panel's key-view loop, per-control
label/hint/identifier assertions, first-responder restoration to the editor
text view, and the panels' commands in the command table / help window /
README with the parity tests extended to the search lane's command file.

## Durable checkpoint

- Branch `agent/mac-a11y-panels/a11y-panels`; HEAD = see `git log -1`.
- Dirty files: (none at commit time; refreshed before long steps)
- Next commands: `cd apps/mac && swift build --build-tests && swift test --filter 'PanelAccessibilityTests|CommandTableTests'`
- Consumed: mac-shell `5bc3fc0f`; search branch `origin/agent/mac-search/panel`
  `00e7b098` (merged only on the local `…-applied` branch).
- Staffing/billing: shared Claude Max quota with parent `mac-claude-a`; no
  purchases; bounded ~75 min from 13:30Z.
