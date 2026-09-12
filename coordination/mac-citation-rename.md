# mac-citation-rename handoff

Agent / task / branch: `mac-citation-rename` (Claude Code subagent, parent
`mac-claude-a`, machine `mac-m1max-a`) / Commander follow-up "reviewed citation
rename" for the finished search lane / `agent/mac-citation-rename/citation-rename`
(base `origin/agent/mac-claude-a/mac-shell` 1630fdbc, which contains main ffe199d;
helper contract read from the local crates at that tip, read-only).

State: in progress (coverage audit done, implementing)

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

## Durable checkpoint

- Branch `agent/mac-citation-rename/citation-rename`, worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-adb6b3377dd30855e`,
  base 1630fdbc (consumed main ffe199d via the parent branch).
- Dirty files (14:12Z, uncommitted): `apps/mac/Sources/FlashTeXMac/ProjectSearchPanel.swift`
  (applyReviewedEdits/settleAfterApply/retryRetained factored out; `swift build` OK),
  `apps/mac/Sources/FlashTeXMac/CitationRename.swift` (new, builds), this handoff.
  Next: `CitationRenameTests.swift`, run tests, commit, push.
- Helper binaries (built 13:30Z from a tree containing b4802cf1, verified to
  contain `plan_citation_rename`): `/Users/jay3332/Projects/flashtex/crates/
  {preview-controller,compiler}/target/release/flashtex-*`.
- Next commands: write `CitationRename.swift`; factor
  `ProjectSearchClient.applyReviewedEdits/retryRetained/settleAfterApply` out of
  `applyPlan/retryUncertain`; write `CitationRenameTests.swift`; `swift build`;
  `swift test --filter 'CitationRename|ProjectSearch|CommandTable'` with
  `FLASHTEX_PREVIEW_CONTROLLER`/`FLASHTEX_COMPILER` set; commit; push.
- Decisions: `_at` span comes from the helper's own lexical `navigate` origin
  (exact symbol span, never a client guess); the client refuses a caret that
  is on no symbol or on a non-citation symbol before sending; the helper's
  `_at` refusal remains the authority. No default shortcut (menu only).
- Staffing/billing: shared Claude Max quota with parent mac-claude-a; no purchases.
