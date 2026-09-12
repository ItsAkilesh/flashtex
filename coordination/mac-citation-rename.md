# mac-citation-rename handoff

Agent / task / branch: `mac-citation-rename` (Claude Code subagent, parent
`mac-claude-a`, machine `mac-m1max-a`) / Commander follow-up "reviewed citation
rename" for the finished search lane / `agent/mac-citation-rename/citation-rename`
(base `origin/agent/mac-claude-a/mac-shell` 1630fdbc, which contains main ffe199d;
helper contract read from the local crates at that tip, read-only).

State: ready_for_integration (tip d4e360d9; parent hook diff below)

Owned paths: `apps/mac/Sources/FlashTeXMac/CitationRename.swift`,
`apps/mac/Tests/FlashTeXMacTests/CitationRenameTests.swift`,
`coordination/mac-citation-rename.md`, `coordination/agents/mac-citation-rename.json`.
Permitted shared edit (search lane finished): `apps/mac/Sources/FlashTeXMac/ProjectSearchPanel.swift`
(client-side reuse only; its 21 tests must stay green). Parent-retained files
(`FlashTeXMacApp.swift` etc.) are never committed on this branch; the App scene
hook is a diff in this handoff and in the final report.

## Coverage audit (2026-09-12T14:00Z, mandatory first step)

Searched `apps/mac/Sources/**`, `apps/mac/Tests/FlashTeXMacTests/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`
for `citation_rename`, `citationrename`, `rename citation`, `plan_citation`,
`citation`:

- `plan_citation_rename` / `plan_citation_rename_at` / `citation-rename-plan`:
  ZERO matches anywhere in `apps/mac` (sources or tests), the mac-live report,
  or `docs/evidence`.
- `apps/mac/Sources/FlashTeXMac/ProjectSearchPanel.swift`: literal-only
  `plan_literal_replacement` → `apply_group` machinery (`ProjectSearchClient
  .planReplacement/applyPlan/retryUncertain`, private `applyGroup/sendApplyGroup/
  reconcile`); schema check is hard-wired to `flashtex.literal-replacement-plan.v1`.
  Nothing about citation keys.
- `apps/mac/Tests/FlashTeXMacTests/ProjectSearchTests.swift`:
  `ProjectSearchPlanTests.testParsePlanGroupsEditsAndKeepsExactOffsets`,
  `testParsePlanRefusesAnythingUnexpected`, `testVerifyAndApplyGroupPayloadAndLabels`;
  `ProjectSearchPlanHelperTests.testPlanIsProposalOnlyAndApplyReplacesEveryMatchPerFile`,
  `testStaleProposalsAndUnsubmittedBuffersAreRefusedPerFile`,
  `testRetryingAnUncertainCommandReplaysExactly` — all literal replacement of
  `café`→`tea`; no bibliography, no citation, no `_at`.
- `apps/mac/Sources/FlashTeXMac/Completion.swift` (`citationCommands`,
  `citationSuggestions`) and `Navigation.swift` (`indexedArgumentCommands`,
  `helperProbeOffset`, `goToMatchingViaHelper` → helper `navigate`): citation
  *completion* and *navigation* only; reusable for locating the key span.
- `apps/mac/Tests/FlashTeXMacTests/NavigationTests.swift`
  `testHelperResolvesLabelsCitationsAndCommandsProjectWide`: `\cite` → `\bibitem`
  navigation; `CompletionTests.swift` lines 434-540: citation vocabulary.
- `apps/mac/Sources/FlashTeXMac/DocumentKinds.swift` + `DocumentKindsTests.swift`
  (`testDeclaredBibliographyKindPersistsAcrossReopenAfterDiskDeletion` …):
  explicit `document_kind: bibliography` declaration and persistence; no rename.
- `tools/native-validation/mac-live/reports/20260912T110944Z.md`: zero matches
  for `citation`/`rename`. `docs/evidence/*`: only `drained-products-2026-09-12.md`
  mentions citations (Rust side), `mac-search-2026-09-12/` is literal search.

Conclusion: the reviewed citation rename gap is entirely uncovered on the Mac
side; only the uncovered part is implemented (reusing the search lane's
apply/retry/reconcile core rather than duplicating it).

## Delivered (2026-09-12T14:16Z)

- `apps/mac/Sources/FlashTeXMac/CitationRename.swift` (new): `CitationRename`
  (pure: `_at`/typed request shapes, `parsePlan` for schema
  `flashtex.citation-rename-plan.v1` kind `citation_key_rename` with exact
  decimal offsets, `keyProblem` mirroring the helper's key rule, `explain` of
  helper refusal codes, previews, VoiceOver labels naming the declared kind);
  `CitationRenameClient` (`locateKeyAtCaret` → helper `navigate` origin span,
  cross-checked against `complete` category `citation` occurrences, refused
  for no symbol / non-citation symbol / \begin-\end / non-durable buffer;
  `planRename` refreshes `model.documentKinds` and refuses when no bibliography
  is declared, `_at` when a span is held else typed old key, stale-span
  refusal; `applyRename` → `ProjectSearchClient.applyReviewedEdits` with
  `citation-rename-<UUID>` ids, per-file outcomes, `settleAfterApply`;
  `retryUncertain` → `retryRetained`, identical id+payload); `CitationRenamePanel`
  (old key from caret or typed, new key, Plan Rename, Apply, per-file outcomes,
  Retry buttons, Esc closes; `FLASHTEX_CITATION_RENAME_OLD/NEW` automation seed,
  proposal only); `CitationRenameWindow` scene + `CitationRenameCommands`
  (Edit > Rename Citation…, no default shortcut, so no command-table change).
- `apps/mac/Sources/FlashTeXMac/ProjectSearchPanel.swift` (shared, search lane
  finished): `applyPlan`/`retryUncertain` now delegate to the new reusable
  `applyReviewedEdits(_:sourceVersions:membershipGeneration:label:commandPrefix:)`,
  `settleAfterApply()` and `retryRetained(commandID:)`; `applyGroup` takes the
  command-id prefix. Behaviour unchanged; its 21 tests are green.
- `apps/mac/Tests/FlashTeXMacTests/CitationRenameTests.swift` (new): 7 pure
  (`CitationRenamePureTests`) + 4 real-helper (`CitationRenameHelperTests`,
  XCTSkip without `FLASHTEX_PREVIEW_CONTROLLER`/`FLASHTEX_COMPILER` or when the
  helper predates the endpoint or refuses the declaration): bib declared through
  `model.documentKinds.declareBibliography("refs.bib")` (never inferred),
  `\cite{knuth84}` in main.tex + chapter.tex + `@article{knuth84,…}` in refs.bib →
  plan covers 3 documents with exact offsets/lines/previews; Apply → every
  durable revision r1→r2, text exact in buffers, `textByDurable`, the helper's
  `document`, history label `Rename citation “knuth84” to “knuth1984” (1 in
  refs.bib)`, kind still `bibliography`; MissingBibliographyDefinition and
  RenameCollision refusals explained; `_at` from a caret inside the key and on
  `\cite`; refusals for a caret on a label, on prose, on `\begin`, on a
  non-durable buffer; empty/invalid/equal keys refused before any request;
  stale refusal after a durable edit of main.tex and of refs.bib (nothing
  applied, plan dropped, caret span dropped and re-locatable at r2); exact
  `apply_group` replay (same id+payload → `replayed_command`, no new revision;
  changed payload refused), then the client's Apply sees the stale snapshot.

Measured: `swift build` OK (worktree, 14:10Z and with the hook applied 14:15Z);
`swift test --filter 'ProjectSearch|DocumentKinds|CitationRename'` → 40 tests,
0 failures (CitationRename 11, ProjectSearch 21, DocumentKinds 8) at 14:14Z with
the release helper binaries. Full `swift test` NOT run: 1-min load was 19–23
(> 15). No app launch / evidence captures (optional; not done).

## Parent-retained hook (request; compile-checked on `agent/mac-citation-rename/citation-rename-applied` at 14:15Z)

```diff
--- a/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift
+++ b/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift
@@ -89,12 +89,13 @@ struct FlashTeXMacApp: App {
                 .onAppear {
                     appDelegate.model = model; nearby.attach(sink: model, destinations: model); TypingBench.shared.install(model: model)
                     // Automation: open a secondary window at launch for evidence captures.
-                    if let id = ProcessInfo.processInfo.environment["FLASHTEX_OPEN_WINDOW"], ["nearby", AccessibilityHelpView.windowID, EditHistoryPanel.windowID, ProjectSearch.windowID].contains(id) { openWindow(id: id) }
+                    if let id = ProcessInfo.processInfo.environment["FLASHTEX_OPEN_WINDOW"], ["nearby", AccessibilityHelpView.windowID, EditHistoryPanel.windowID, ProjectSearch.windowID, CitationRename.windowID].contains(id) { openWindow(id: id) }
                 }
         }
         .commands {
             NavigationCommands(model: model) // Navigation.swift
             ProjectSearchCommands(openWindow: openWindow) // ProjectSearchPanel.swift: ⌘⇧F Find in Project…
+            CitationRenameCommands(openWindow: openWindow) // CitationRename.swift: Edit > Rename Citation… (no shortcut)
             CommandGroup(after: .help) {
                 Button("FlashTeX Accessibility Help") { openWindow(id: AccessibilityHelpView.windowID) }
             }
@@ -177,5 +178,6 @@ struct FlashTeXMacApp: App {
         }
         Settings { EditorPreferencesView() } // EditorPreferences.swift (⌘,)
         ProjectSearchWindow(model: model) // ProjectSearchPanel.swift: Find in Project (⌘⇧F)
+        CitationRenameWindow(model: model) // CitationRename.swift: Rename Citation (reviewed plan_citation_rename → apply_group)
     }
 }
```

## Durable checkpoint

- Branch `agent/mac-citation-rename/citation-rename`, worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-adb6b3377dd30855e`,
  base 1630fdbc (consumed main ffe199d via the parent branch); tip d4e360d9 +
  this registration/handoff commit. Hook-applied compile branch:
  `agent/mac-citation-rename/citation-rename-applied` (pushed; never integrate).
- Dirty files: none after the registration commit. Next commands: none (lane
  done); parent integrates.
- Decisions: `_at` span comes from the helper's own lexical `navigate` origin
  (exact symbol span, never a client guess), cross-checked against the
  helper's citation occurrences; the helper's `_at` refusal remains the
  authority. No default shortcut (menu only). Bibliography declarations only
  from `model.documentKinds` (helper-reported), never from file extensions.
- Staffing/billing: shared Claude Max quota with parent mac-claude-a; no purchases.
