# mac-search handoff — native project search through the durable helper

- Updated UTC: 2026-09-12T13:40Z (resumed after a Claude Max 429 at ~11:2xZ; worktree verified clean against Git before continuing)
- Agent / parent / machine: `mac-search` (Claude Code subagent) / parent
  `mac-claude-a` / `mac-m1max-a`. Commander lane (issue #2, 10:41Z):
  "Native project search: new ProjectSearchPanel.swift + dedicated bounded
  client/tests, consume existing helper search_literal exact source_versions
  and partial termination. Show UTF8-safe matches and refuse stale navigation."
- Branch: `agent/mac-search/panel` from mac-shell f4c8aea; merged mac-shell
  5222b86, a73bdf2, ff574d7 and 5bc3fc0 (preferences, history panel,
  validation-3) and origin/main 1884986. Tip at the time of writing: see
  `coordination/agents/mac-search.json` `code_revision`.
- Owned paths: `apps/mac/Sources/FlashTeXMac/ProjectSearchPanel.swift`,
  `apps/mac/Tests/FlashTeXMacTests/ProjectSearchTests.swift`,
  `docs/evidence/mac-search-2026-09-12/`, this file,
  `coordination/agents/mac-search.json`. Parent-retained
  `FlashTeXMacApp.swift` carries the requested 3-line diff below in the
  separate labelled commit 6235272 ("requested parent diffs"), kept through
  the merges (conflicts with the preferences `Settings` scene and the
  history panel's `FLASHTEX_OPEN_WINDOW` id were resolved by keeping both
  lanes' lines), so the branch builds and launches as a whole; the parent
  may merge the branch as is or apply the diff by hand.
- Worktree: `.claude/worktrees/agent-a5be5c296cd452670`. Dirty files: none
  besides this handoff and the agents JSON at the time of writing.
- Rules honoured: no purchases or paid network calls; no edits to
  Navigation.swift, ShellModel*, ContentView, PreviewView; transferred crates
  untouched; app launched only with `FLASHTEX_NO_ACTIVATE=1`; user is primary
  author with truthful trailers.

## Ready behaviour (ProjectSearchPanel.swift)

- `ProjectSearch` (pure): `request(...)` builds
  `search_literal {source_versions, literal, max_matches, max_work, documents?}`
  with the limits clamped into the helper's ranges (1…1000, 1…1 000 000);
  `parse(reply:)` accepts `{source_versions, matches:[{path,revision,
  start_byte,end_byte}], termination, work_used}` and refuses a match whose
  revision is not the reply's map; `Termination` maps `complete` /
  `match_limit` / `work_limit` / `cancelled` / unknown — only `complete` is
  exhaustive, every other value is shown as a "partial" badge with "not
  exhaustive" and an explanation naming the limit; `locate(start:end:in:)`
  gives the 1-based line, byte column and a snippet (line context clipped on
  `Character` boundaries, so never inside a composed sequence; `\r\n` lines
  drop the `\r`; a multi-line literal shows ⏎); `bytesSpell` is a `memcmp`
  check that a range still spells the literal; `accessibilityLabel` is
  "match n of m, path, line l, snippet".
- `ProjectSearchClient` (bounded, main actor): `snapshot` →
  `search_literal` through the parent's `controllerRequest` / `awaiting`
  pattern; one automatic retry after the helper's "source versions changed"
  error (the user was typing), the second refusal is shown; durable text for
  snippets comes from `controllerState.textByDurable` or one `document` read
  awaited ≤ 2 s (a document not open in the window, e.g. `\input{chapter}`,
  is not opened by a search). Scope: whole project or the active document
  (`documents:[activePath]`; a path the helper does not index is explained,
  nothing sent). `knownStaleNote` names durable versions that moved since
  the search from the session's receipts.
- Navigation (`navigate(to:of:)`): refresh `snapshot`; the full version map
  must equal the one the search ran on, otherwise "project changed since this
  search (main.tex r1→r2); search again" (a durable edit anywhere — the
  helper's own stale rule, so "match n of m" is always about the current
  project when it navigates); the durable bytes at the range must still
  spell the literal; then `ShellModel.selectIndexLocation` (Navigation.swift)
  applies the `navigateExactly` rule to the open buffer — a local unsubmitted
  edit overlapping the range is refused ("edited since the index was built …
  overlap the edit at …"), an edit elsewhere is rebased byte-exactly and
  re-verified, the selection covers whole composed character sequences, and
  a project document not open in the window is opened at its durable
  revision. Refusals set the status line ("Not navigated — …") and select
  nothing.
- Window: `Find in Project` (`ProjectSearchWindow` scene, id
  `project-search`), ⌘⇧F via `ProjectSearchCommands`. The field says
  "case-sensitive literal, no regex"; Return searches when the query or scope
  differs from the list, otherwise goes to the selected match; ↑/↓ move the
  selection from the field or the list; ⌘G "Next Match" wraps; Esc closes
  (cancel-action button). Rows: `path:line` + snippet with the match bold in
  the accent colour; VoiceOver label per row as above, the summary line and
  the status line are labelled. Header shows "durable chapter.tex r1,
  main.tex r1" so the user sees which revisions were searched.
- No helper: "Search requires the durable helper (flashtex-preview-controller)
  to be attached and ready." is shown in the panel and as the search status.
- Automation hooks (evidence only): `FLASHTEX_SEARCH_QUERY` /
  `FLASHTEX_SEARCH_MAX_MATCHES` seed and run a search once the helper is
  ready; `FLASHTEX_OPEN_WINDOW=project-search` (parent diff) opens the window.

## Follow-up delivered: reviewed literal replacement (a7d13af, bf56a90)

Named in issue #2 (10:41Z): "consume the root-owned literal replacement …
plan endpoints". `plan_literal_replacement` landed on origin/main 4e15783
(`crates/preview-controller/docs/source-plans.md`, revision 5, schema
`flashtex.literal-replacement-plan.v1`); this lane merged main and built
the helper in its worktree to consume it. No exact handoff from the root
engineer reached this lane; the published contract and its helper tests
were used as the source of truth.

- `ProjectSearch.parsePlan`: decimal-string offsets/revisions parsed as
  exact integers (`exactInt`, never floating point); refuses a wrong schema,
  missing `proposal_only`/`requires_user_approval`, an unknown
  `application_order`, a snapshot whose versions/generation differ from the
  reply's, an incomplete search, a literal/replacement mismatch, an edit at
  another revision, a wrong-length range, overlapping/unsorted edits.
- Panel: "Replace with" field + "Plan Replacement" (enabled only for a
  complete, non-empty search of the current query) shows the proposal per
  file with exact before → after lines (VoiceOver: "replacement n of m,
  path, line l, <line> becomes <line>") and the status "Nothing is changed
  until you click Apply". Apply (the approval) refreshes `snapshot` and
  requires the plan's full version map and membership generation, then per
  file: waits (≤2 s) for an in-flight edit of that path, reads the durable
  text, verifies every range spells `expected_text` byte-for-byte, requires
  the session's durable hash at that revision, refuses when the open buffer
  is not yet durable, sends one `apply_group` with a fresh retained
  `search-replace-<uuid>` command id, `removed_text = expected_text` and the
  reviewed revision/hash, then records the returned `history.document`
  exactly as the parent's `document`/`edit` handler does (durable revision/
  hash/text, editor mapping) and replaces the open buffer with it
  (`updateActiveText` for the active document — already durable, so no edit
  is sent; direct assignment for another open document; a non-open project
  document is edited durably without being opened). Outcomes are listed per
  file ("applied as durable rN" / "not applied — why" / "uncertain — retry
  command <id> unchanged" with a Retry button that resends the identical
  payload); the status never claims all-or-nothing. The search re-runs on
  the new versions afterwards; if the preview did not follow within 1 s and
  the active buffer is exactly durable, one `compile` is requested (never a
  submission of pending edits).
- Older helper builds: "This helper build has no plan_literal_replacement
  (needs crates/preview-controller from main ≥ 4e15783)."
- Tests: `ProjectSearchPlanPureTests` (4) and `ProjectSearchPlanHelperTests`
  (3, real helper built from merged main in this worktree +
  `flashtex-compiler`; they skip with an older helper — verified against the
  main checkout's 06:27 build): proposal → apply across two files including
  the non-open chapter.tex (durable r2 each, buffer equals the helper's
  text, no extra durable revision from a resubmitted buffer, history label
  "Replace “café” with “tea” (2 in main.tex)"), per-file refusal with an
  unsubmitted main.tex buffer while chapter.tex applies, stale-proposal
  refusal after a durable edit ("main.tex r2→r3; nothing applied"), partial
  search refused by the helper, replay of a fixed command id (same revision,
  `replayed_command:true`) and refusal of a changed payload under that id.
- Evidence: `docs/evidence/mac-search-2026-09-12/find-in-project-replacement-plan.png`
  (live window: 4 matches + the proposal per file, Apply button, nothing
  applied).
- Not done: citation rename (`plan_citation_rename(_at)`) — a different
  input (a citation key, bibliography document kinds) than the search
  panel's literal; undo of an applied group is the history panel's
  (mac-history) ledger undo, referenced by label.

## Validation

- `ProjectSearchPureTests` (9): termination labelling, summary wording,
  request clamping, reply parsing/refusals, line/column/snippet on
  multi-byte text (naïve, café, Résumé, e+U+0301 clusters, ZWJ family,
  `\r\n`, multi-line literal, end of text), byte-exact verification (NFC vs
  NFD is different), labels, no-helper client state.
- `ProjectSearchHelperTests` (5, real `flashtex-preview-controller` + real
  `flashtex-compiler`, two-file project main.tex + `\input{chapter}` with
  multi-byte text; skip without the binaries — they ran here): complete
  search lists both documents in helper order with durable lines/snippets
  (comments included, case-sensitive "Café" → 0 complete); match limit 1 / 4
  → `match_limit` partial, 5 → complete, clamped 0 → 1, work budget 1 →
  `work_limit` with 0 matches; exact navigation to `café` in chapter.tex
  (opened at durable r1, UTF-16 length 4) and after `naïve ` in main.tex,
  arrow clamping, Return search-vs-navigate; active-document scope (2 matches
  in main.tex only), ⌘G wrap, unindexed path refusal; refusal after a local
  overlapping edit (and rebase of the later match), refusal of every match
  after the edit became durable ("main.tex r1→r2"), the helper's own
  stale-version error, fresh search sees 3 matches, forged literal refused.
- Full `swift test` in `apps/mac` with the five real binaries
  (`FLASHTEX_PREVIEW_CONTROLLER/COMPILER/PDF/BRIDGE/EDIT_LEDGER`):
  **477 tests, 11 skipped, 0 failures** at 0cdf1c5; **503 / 13 / 0** at
  fe5815e (mac-shell 5222b86 merged); **506 / 13 / 0** at 9e89cc3 (a73bdf2
  merged); **513 / 13 / 0** at bf56a90 and **531 / 14 / 0** at 00e7b09
  (ff574d7 merged) with the helper built from merged main (plan tests ran);
  **543 / 14 / 0** at 490f4f2 (mac-shell 5bc3fc0 merged, helper from merged main). Every skip is another lane's optional route (project-files
  helper, explain helper, pdf-exact, nearby evidence, assistant-context,
  overlay, provider path).
- Live evidence: `docs/evidence/mac-search-2026-09-12/find-in-project-complete.png`
  (4 matches, complete badge) and `find-in-project-match-limit.png` (2 shown,
  "match limit" badge + "not exhaustive"), the debug app against the real
  helper with `FLASHTEX_NO_ACTIVATE=1 FLASHTEX_AUTOATTACH=1
  FLASHTEX_SEED_FILE=<project>/main.tex FLASHTEX_OPEN_WINDOW=project-search
  FLASHTEX_SEARCH_QUERY=café`, captured with `screencapture -l <window id>`.
- Helper binaries used: the main checkout's 06:27 build (has
  `search_literal`, no plan endpoints — plan tests skip) and this worktree's
  `crates/preview-controller/target/release/flashtex-preview-controller`
  built from merged main 1884986 (plan endpoints; all 21 search tests run).
  The parent's launch/test scripts should point `FLASHTEX_PREVIEW_CONTROLLER`
  at a helper built from main ≥ 4e15783 for the replacement flow.

## Incomplete / limitations / needs from others

- Until the parent applies the diff below, the window and ⌘⇧F are not
  reachable from the app (the client and tests do not need it).
- Helper semantics worth knowing: reaching the match limit is reported as
  `match_limit` even when no further match exists (unless the last match ends
  at the very end of the last document), so "4 matches with limit 4" shows as
  partial — truthfully, the helper stopped there. The default limit is 200.
- No mid-query cancellation and no regex (helper limitation, stated in the
  panel). Searching while the user types is refused by the helper as stale;
  one retry is automatic, then the panel asks to search again.
- Navigation strictness: a durable edit in *any* document invalidates the
  result set for navigation (full-map equality). This keeps "match n of m"
  honest; a per-document rule would be a one-line relaxation if the parent
  prefers it.
- Replacement application writes `controllerState.durable` /
  `textByDurable` / `editorRevisionByDurable` from this lane's file (they are
  internal, as `selectIndexLocation` relies on) — the same fields the
  parent's private `applyDurableDocument` maintains. If the parent prefers a
  single writer, a small hook `func recordDurableDocument(path:revision:
  sha256:text:)` on ShellModel+Controller exposing that logic would let
  `ProjectSearchClient.reconcile` call it instead; not required today.
- A preview compiled for the applied revision can race the buffer update
  and be logged as "versions unknown to this session"; the bounded
  `compile` fallback covers it (observed in tests: no extra durable revision).
- source-plans.md is marked revision 5 "in progress"; if the schema or the
  apply contract changes, `parsePlan` refuses the new shape explicitly
  rather than guessing.

## Exact diff needed in parent-retained files (applied locally in 6235272)

`apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift`:

```diff
@@ -89,7 +89,7 @@ struct FlashTeXMacApp: App {
-                    if let id = ProcessInfo.processInfo.environment["FLASHTEX_OPEN_WINDOW"], ["nearby", AccessibilityHelpView.windowID].contains(id) { openWindow(id: id) }
+                    if let id = ProcessInfo.processInfo.environment["FLASHTEX_OPEN_WINDOW"], ["nearby", AccessibilityHelpView.windowID, ProjectSearch.windowID].contains(id) { openWindow(id: id) }
@@ -94,6 +94,7 @@ struct FlashTeXMacApp: App {
         .commands {
             NavigationCommands(model: model) // Navigation.swift
+            ProjectSearchCommands(openWindow: openWindow) // ProjectSearchPanel.swift: ⌘⇧F Find in Project…
@@ -170,5 +171,6 @@ struct FlashTeXMacApp: App {
         Window("Accessibility Help", id: AccessibilityHelpView.windowID) {
             AccessibilityHelpView() // FlashTeXAccessibility: focus order, VoiceOver notes, command table
         }
+        ProjectSearchWindow(model: model) // ProjectSearchPanel.swift: Find in Project (⌘⇧F)
```

No ShellModel / ShellModel+Controller / Navigation diff is needed:
`controllerRequest`, `controllerSourceVersions`, `selectIndexLocation`,
`controllerState.textByDurable` and `controller.document(path:)` are internal
and already on mac-shell.

## Reviewed peer revisions / adaptations

- `origin/agent/mac-claude-a/mac-shell` f4c8aea (base), 5222b86 and a73bdf2
  (merged; hybrid historical release, delimiter matching, multifile evidence,
  nearby generation API — no overlap with the new files, suite green after
  each), 7ccbd7e (packaging scripts only; reviewed, not merged).
- `origin/main` 2d68926 / 1884986 (merged at ba5d0a2): `search_literal`
  handler and STDIO.md match the client (whole version map compared, limits
  1…1000 / 1…1 000 000, termination strings including `cancelled`); 4e15783's
  plan endpoints are consumed as described above. mac-shell ff574d7 and
  5bc3fc0 (preferences Settings scene, history panel window id, validation
  lanes) merged with two trivial conflicts in FlashTeXMacApp.swift, both
  resolved by keeping both lanes' lines.

## Resources / next

- Resource pool: shared Claude Max 20x quota with parent mac-claude-a; no
  purchases; usage totals unknown to this worker. Context usage: well under
  20% of the 1M budget by the session's token readout.
- Running commands: none. App processes: none left running.
- Exact next step for the parent: merge `agent/mac-search/panel` (it already
  contains mac-shell 5bc3fc0 and the 3-line FlashTeXMacApp diff), `swift
  test` in `apps/mac` with the worker binaries (helper from main ≥ 4e15783
  to exercise the plan tests). Resource note: this lane's session was cut by
  a Claude Max 429 (monthly spend limit, reset 13:20Z, no purchase) and
  resumed from a clean worktree.
- Resume reading list: this file, `ProjectSearchPanel.swift` header comment,
  `ProjectSearchTests.swift` `ProjectSearchHelperTests`,
  `crates/preview-controller/docs/source-plans.md` on main for the follow-up.
