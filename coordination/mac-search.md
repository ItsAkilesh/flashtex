# mac-search handoff — native project search through the durable helper

- Updated UTC: 2026-09-12T11:20Z
- Agent / parent / machine: `mac-search` (Claude Code subagent) / parent
  `mac-claude-a` / `mac-m1max-a`. Commander lane (issue #2, 10:41Z):
  "Native project search: new ProjectSearchPanel.swift + dedicated bounded
  client/tests, consume existing helper search_literal exact source_versions
  and partial termination. Show UTF8-safe matches and refuse stale navigation."
- Branch: `agent/mac-search/panel` from mac-shell f4c8aea; merged mac-shell
  5222b86 and a73bdf2. Tip at the time of writing: see
  `coordination/agents/mac-search.json` `code_revision`.
- Owned paths: `apps/mac/Sources/FlashTeXMac/ProjectSearchPanel.swift`,
  `apps/mac/Tests/FlashTeXMacTests/ProjectSearchTests.swift`,
  `docs/evidence/mac-search-2026-09-12/`, this file,
  `coordination/agents/mac-search.json`. Parent-retained
  `FlashTeXMacApp.swift` carries the requested 3-line diff below in the
  separate labelled commit 6235272 ("requested parent diffs") so the branch
  builds and launches as a whole; the parent may cherry-pick it or apply the
  diff by hand.
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
- Full `swift test` in `apps/mac` with the five real binaries from the main
  checkout (`FLASHTEX_PREVIEW_CONTROLLER/COMPILER/PDF/BRIDGE/EDIT_LEDGER`):
  **477 tests, 11 skipped, 0 failures** at 0cdf1c5; **503 / 13 / 0** at
  fe5815e (after merging mac-shell 5222b86); **506 / 13 / 0** at 9e89cc3
  (after merging a73bdf2). Every skip is another lane's optional route
  (project-files helper, explain helper, pdf-exact, nearby evidence,
  assistant-context, overlay).
- Live evidence: `docs/evidence/mac-search-2026-09-12/find-in-project-complete.png`
  (4 matches, complete badge) and `find-in-project-match-limit.png` (2 shown,
  "match limit" badge + "not exhaustive"), the debug app against the real
  helper with `FLASHTEX_NO_ACTIVATE=1 FLASHTEX_AUTOATTACH=1
  FLASHTEX_SEED_FILE=<project>/main.tex FLASHTEX_OPEN_WINDOW=project-search
  FLASHTEX_SEARCH_QUERY=café`, captured with `screencapture -l <window id>`.
- Helper binary used: `crates/preview-controller/target/release/
  flashtex-preview-controller` built 06:27 in the main checkout (contains
  `search_literal`; does not contain the plan endpoints main gained later).

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
- Follow-up named by the Commander (issue #2 10:41Z): consume the root-owned
  literal replacement / citation rename plan endpoints "after exact handoff".
  `plan_literal_replacement` / `plan_citation_rename(_at)` landed on
  origin/main 4e15783 (`crates/preview-controller/docs/source-plans.md`,
  revision 5 "in progress": read-only proposals with decimal-string offsets,
  application through per-document `apply_group`, no multi-document
  atomicity). Not started: the installed helper binary predates it and no
  handoff has reached this lane yet; the panel's search results (exact
  durable ranges + versions) are the natural input for it.

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
- `origin/main` 2d68926 / 1884986: `search_literal` handler and STDIO.md
  match the client (whole version map compared, limits 1…1000 / 1…1 000 000,
  termination strings including `cancelled`); 4e15783 added the plan
  endpoints noted above.

## Resources / next

- Resource pool: shared Claude Max 20x quota with parent mac-claude-a; no
  purchases; usage totals unknown to this worker. Context usage: well under
  20% of the 1M budget by the session's token readout.
- Running commands: none. App processes: none left running.
- Exact next step for the parent: cherry-pick 6235272 (or apply the diff),
  `swift test` in `apps/mac` with the worker binaries, merge
  `agent/mac-search/panel` into `mac-shell`.
- Resume reading list: this file, `ProjectSearchPanel.swift` header comment,
  `ProjectSearchTests.swift` `ProjectSearchHelperTests`,
  `crates/preview-controller/docs/source-plans.md` on main for the follow-up.
