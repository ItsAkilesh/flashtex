# Lane mac-grok-assistant — Ask Grok on the live document

Child of `mac-claude-a` on mac-m1max-a. Branches:

- `agent/mac-grok-assistant/ask` (base `origin/agent/mac-claude-a/mac-shell` 66e9842e) — the Mac feature.
- `agent/mac-assistant-context/relocate-edits` (base `origin/main` ce330d42, d1795267) — the helper change, for the crate owner's review.

## Coverage audit (before implementing)

Existing Grok surfaces at the base tip: the capture-review sheet's Explain
(`ProposalPreview.explain()`, `ProposalPreviewTests`, `GrokLiveTests`,
`GrokLiveAcceptanceTests`, `AssistantRecoveryTests`, `ReviewRecoveryTests`),
Preferences (`GrokPreferencesView`, `PanelAccessibilityTests.testGrokPreferencesSectionControlsTakeKeyboardFocus`),
the status pill (`GrokStatusPill`), capture→LaTeX through the bridge. No editor
surface, no live-document binding, no Problems-row hook: the gap was uncovered.
`apps/mac/docs/grok-live.md` records that the fast model was refused 2/2
("removed source differs").

## Durable checkpoint (2026-09-12T22:13Z)

- Branch `agent/mac-grok-assistant/ask` at 7898941c (pushed); helper branch at d1795267 (pushed).
- Dirty at checkpoint: `apps/mac/docs/grok-live.md` (Ask Grok section), `coordination/agents/mac-grok-assistant.json`, this file — committed next.
- Consumed main: ce330d42 (helper branch base); mac-shell 66e9842e.
- Tested: `GrokAssistantTests` 11/11; `swift test --filter "GrokLiveTests|ProposalPreviewTests|AssistantRecoveryTests|ReviewRecoveryTests|CommandTableTests|WorkspaceShellTests|DiagnosticsPanelTests|PanelAccessibilityTests|ExplanationMemoTests"` → 79 tests, 2 skipped, 2 failures, both in `PanelAccessibilityTests.testGrokPreferencesSectionControlsTakeKeyboardFocus` (Preferences window keyboard walk; files this lane did not touch; not verified on the base tip); `FlashTeXAccessibilityTests` 8/8 + 3 + 2 + 8; crate `cargo test --release` 18 passed, `--features grok` 20 passed.
- Live: 3 xAI calls (the brief's maximum), evidence `docs/evidence/grok-assistant-20260912T221011Z/INDEX.md` (+ `…221103Z-fix`, `…221228Z-fix`).
- Full `swift test` with real helpers: started at load ~2 after the live checks; result recorded below when known.
- Next commands: `git push`; parent applies/merges; crate owner reviews relocate-edits.
- Restrictions: Claude Max shared quota; no purchases; MacTeX oracle only (unused here); no focus stealing (`FLASHTEX_NO_ACTIVATE=1`).

## Parent-retained hunks (also in the lane report)

`ShellModel.swift` (+2: `@ObservationIgnored let grokAssistant = GrokAssistant()`),
`ContentView.swift` (+4: `.grokAssistantSheet()`; toolbar Ask Grok button),
`FlashTeXMacApp.swift` (+2: `Button("Ask Grok…")` ⌘⌥G in the Edit group).
These are committed on the lane branch (the brief allows minimal wiring edits).

## Shortcut note

The brief asked for ⌘⇧G; that is `Edit › Convert Capture` (README table,
`AccessibilityCommand.convertCapture`). The lane used ⌘⌥G and added the
README row + `AccessibilityCommand.askGrok`; swapping is a two-line change in
`FlashTeXMacApp.swift` + two table rows if the parent prefers ⌘⇧G.

## Full-suite result

`swift test` with the real compiler/pdf/bridge/edit-ledger/preview-controller/
project-files/assistant-context helpers (main-checkout builds), load 2–6, at
7898941c: **877 tests, 30 skipped, 48 failing assertions in exactly two test
methods**: `CompletionLatencyTests.testPickupNarrowArrowAndReturnLatencyBestOfN`
(46 assertions: completion popup narrowing timed out — a hosted-window
keyboard/latency test under `FLASHTEX_NO_ACTIVATE=1`) and
`PanelAccessibilityTests.testGrokPreferencesSectionControlsTakeKeyboardFocus`
(2: Preferences keyboard walk). Neither test nor the files they exercise
(Completion.swift, GrokPreferencesView.swift, EditorPreferences.swift) were
touched by this lane; not re-verified on the base tip. Every other suite,
including all 11 `GrokAssistantTests` and the fixture-sharing
GrokLive/ProposalPreview/AssistantRecovery/ReviewRecovery suites, passed.
