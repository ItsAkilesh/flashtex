# Completion pickup / keyboard latency and stale-context refusal — 2026-09-12

Lane `mac-completion-2` (Claude Code subagent of `mac-claude-a`, mac-m1max-a,
M1 Max, Xcode 26.3, Swift 6.2.4, debug `swift test` build). Branch
`agent/mac-completion-2/latency` from mac-shell `cd58fc2e`. Every number below
is from a test that prints it (`CompletionLatencyTests`, `CompletionTests`,
`CompletionLiveHelperTests`) together with the 1-minute load average it ran
under; bounds are enforced only below load 20 and the rows say which runs were
under load. The machine was shared with other agents' builds the whole time
(load 13–80), so the *best* of N is the meaningful column; medians and maxima
are load noise.

## Harness

`apps/mac/Tests/FlashTeXMacTests/CompletionLatencyTests.swift`
`testPickupNarrowArrowAndReturnLatencyBestOfN`: `Samples/demo.tex` (5 909 B)
with `\s` inserted before `\end{document}` in the real `CompletingTextView`
hosted in a never-key window; 15 iterations of ⌃Space → list shown, `u` →
list narrowed, ↓ → selection moved, Return → inserted and closed. Each stage is
timed from the `keyDown` call until the observable state holds while the main
run loop turns in 0.5 ms slices; the pickup stage is broken down into
keystroke-on-main, queue wait, off-main scan, run-loop delivery lag, `present`
(session + popup) and the remainder of the run-loop turn (the panel's display
cycle, an estimate by subtraction). The editor's own layout of the swapped
text is settled before the pickup timer starts.

## Results (demo.tex, best of 15, ms)

| run | load | pickup ⌃Space→list | narrow key→list | ↓→selection | Return→inserted | scan (off-main) | present | display rest |
|---|---|---|---|---|---|---|---|---|
| baseline (before popup changes, layout not settled) | 34.1 (under load) | 10.50 | 8.83 | 0.23 | 2.79 | 0.87 | – | – |
| baseline breakdown | 38.3 (under load) | 10.87 | 7.87 | 0.22 | 2.73 | 0.88 | 1.67 | delivery lag 4.50 (pending text layout) |
| popup fast paths + settled layout | 42.5 (under load) | 7.15 | 9.48 | 0.25 | 3.18 | 0.90 | 2.13 | delivery lag 0.02 |
| same, bounds enforced | 17.1 | 5.70 | 5.61 | 0.16 | 2.88 | 0.88 | 1.77 | – |
| same, bounds enforced | 13.7 | 5.64 | 5.12 | 0.16 | 3.00 | 0.87 | 1.60 | 2.67 |
| `setFrame(display: false)` | 13.1 | 5.53 | 6.04 | 0.18 | 2.81 | 0.88 | 1.49 | 1.30 |
| repeat | 13.1 | 6.89 | 6.46 | 0.25 | 1.16 | 0.90 | 2.18 | 3.24 |
| final code, 1-min load 10.3 (5-min still 27.7) | 10.3 | 7.40 | 7.56 | 0.22 | 3.70 | 1.08 | 1.99 | 0.01 |

The last row shows the 1-minute average lagging the actual contention (the
5-minute average was still 27.7 and every stage, scan included, was slower
than at load 13–17); it is kept because the bounds were enforced and passed.

Bounds enforced when load < 20: pickup best < 25 ms, narrow best < 25 ms,
arrow best < 5 ms, Return best < 15 ms (passed at load 13.1–17.1; skipped with
the numbers printed at higher load).

What the breakdown says: the off-main scan is ~0.9 ms and the queue wait
~0.01 ms; the keystroke itself costs ~0.02 ms on the main thread; run-loop
delivery (`CFRunLoopPerformBlock` + wake-up) is ~0.02 ms once the editor's own
pending layout is out of the way. The remaining ~4 ms of a 5.5 ms pickup is
AppKit: filling the table and ordering the child panel on screen (`present`
1.5–2 ms) and the display cycle that draws it (1.3–3 ms). Those are
window-server costs, not scan costs; a shadow-less panel was tried and could not
be evaluated (the machine went to load 30–80 during the runs, every stage
tripled), so the shadow stays.

Arrow keys through a full 12-row list (`testArrowSelectionThroughTwelveRowsAvoidsTheTableReload`,
120 presses, load 27.5, under load): ↓ keyDown best 0.010 / median 0.011 ms;
`popup.update` with the same rows (selection only) median 0.008 ms versus a
forced reload of the same rows median 0.016 ms. Microseconds either way — the
table is cheap; the arrow path was never the problem.

## Changes that affect latency (`Completion.swift`)

- `CompletionPopup.update` reloads the table only when the items changed; a
  pure selection move (↑/↓/Tab/⇧Tab) selects the row without `reloadData`.
- `CompletionPopup.show` while already on screen for the same parent: no
  `orderFront`, no `addChildWindow`, and `setFrame` only when the target frame
  differs (the caret moved); the frame is set with `display: false` so the
  panel is drawn once by the display cycle that shows it.
- `CompletionPopup.hide` skips `orderOut`/`removeChildWindow` when not needed.
- `CompletingTextView.requestCompletion` builds the panel on first use while
  the first scan runs off-main, so delivery only fills and shows it.
- Evidence fields: `CompletionScheduler.Outcome.queuedMs` / `computedAtNs`,
  `CompletingTextView.lastDeliveryLagMs` / `lastPresentMs`.

## Real helper route (`CompletionLiveHelperTests`, `flashtex-preview-controller` + compiler built from this branch's crates)

`testHelperVocabularyReachesThePopupBoundToTheEditorRevision`, `\cite{` pickup
best of 10 with the project-index key bound to the caret's revision
(⌃Space → list): best 0.97 / median 3.03 / max 4.35 ms at load 17.1; best 1.12 /
median 1.81 / max 3.74 ms at load 10.3; best 1.05 / median 2.85 / max 8.11 ms
at load 24.4; best 1.50 / median 8.08 / max 16.40 ms at load 43.1. Edit → index metadata bound (helper compile + `complete`
label/citation/command + `snapshot`): best 26.0 / median 28.4 ms (load 17.1);
best 28.4 / median 31.9 ms (load 24.4). Request → bound metadata for the first
snapshot 3.4–14.1 ms; rebound after an edit 1.0–3.2 ms. 12 vocabulary queries
per run were refused because a probe edit followed the request (reported,
never shown).

## Stale-context refusal

- `CompletionLatencyTests.testOutcomeForThePreviousDocumentNeverOpensOrMutatesTheList`:
  with a narrowing scan pending for document A, the editor swaps `string` to
  document B whose restored caret is the same UTF-16 offset the pending request
  carries; the A outcome is refused (`refusedStale`), the list stays closed and
  empty, the scheduler generation is untouched, nothing is inserted; a fresh
  request on B lists B's candidates only. Repeated with a session open on B and
  a swap back to a longer A′ at the same offset.
- `testHelperReplyAfterProjectReplacementIsRefusedAtBind`: the four helper
  replies for a query made at editor revision 5 land after File > Open moved
  the editor to revision 6; the metadata decodes, is held, but `boundMetadata`
  is nil, the `\cite{` list built afterwards carries no index key, and metadata
  decoded for revision 6 binds and shows it.
- Existing coverage kept: scheduler cancel/supersede/late refusal, caret move,
  programmatic replacement, resign first responder, helper-side "source
  versions changed", editor-side rebinding (`CompletionTests`).

## Follow-up 1 — keyboard traversal + VoiceOver

Tab / ⇧Tab now walk the open list (wrapping) like ↓ / ↑; Return / Enter
insert; Esc closes; Tab with no list open is the editor's own tab.
`AccessibilityCommand.completionList` (↑, ↓, Tab, ⇧Tab, Return) with a README
"Keyboard shortcuts" row, the help-window VoiceOver note and
`CompletionAccessibility.listHelp` naming Tab/Shift-Tab; `CommandTableTests`
(README parity both directions, menu wiring) 8/8.
`CompletionTests.testTabAndShiftTabTraverseTheListWithVoiceOverLabels`: the
walk through the real view, first responder and caret unchanged, and the real
popup read back through NSAccessibility (table label/help, per-row cell
descriptions "candidate, kind, origin", selected row index, "n of m"
announcement text). VoiceOver itself was not run (no Accessibility permission).

## Follow-up 2 — `\cite{` and declared bibliography kinds

`ProjectIndexCompletionFetcher` sends `snapshot` with the three `complete`
queries; `Completion.Metadata.documentKinds` holds `document_kinds` for exactly
those source versions (stale versions or an unknown kind value refuse the
query; a helper without `document_kinds` binds no kinds). `\cite{` details:
"record in refs.bib (declared bibliography)" (ranked first), "\bibitem in
main.tex", "defined in notes.bib (kind not reported by this helper)" (a path
without a reported kind is unknown, never a bibliography by its name), "cited
but not defined in a declared bibliography source (none declared: Project >
Document Kinds)". Unit: `testCitationCompletionUsesDeclaredKindsAndNeverInfersThem`,
`testFetcherQueriesThreeCategoriesPlusKindsAndRefusesStaleOrFailedReplies`.
Live: `testCiteCompletionUsesTheDeclaredBibliographyKind` — before declaring,
`knuth84` (cited in main.tex, defined only in refs.bib) is "cited but not
defined … (none declared …)"; after `model.documentKinds.declareBibliography("refs.bib")`
the next snapshot reports `refs.bib: bibliography` and the same key is
"record in refs.bib (declared bibliography) · 2 uses · revision 5".
`DocumentKinds.refresh` failing because the controller was detached now reports
"no preview controller attached" (a race that failed
`DocumentKindsTests.testUndeclareDetachesAndForgetsTheDeclaration` 2 of 3 runs
at load 36–80 once the fetcher's extra snapshot round trip lengthened the
helper queue).

## Full suite

`swift test` with real `flashtex-compiler` / `flashtex-preview-controller`
(built from this branch's crates) and `flashtex-pdf` / `flashtex-bridge` /
`flashtex-edit-ledger` / `flashtex-project-files` / `flashtex-assistant-context`
(the main checkout's release builds), started at 1-minute load 11.7: **667
tests, 0 failures, 23 skips** (4 load-gated timing bounds of this lane — the
suite itself raised the load — and 19 env-gated tests of other lanes), 205 s.
A first run crashed the runner in
`ControllerPipelineReviewTests.testCompletionQueryOutstandingAtHelperExitDoesNotSwallowTheRelaunchedHelpersReplies`
(`removeFirst` on an empty scripted-id list: it handed exactly three ids to
the fetcher, which now sends four); the test hands four ids and expects four
outstanding replies.

## Limitations

- No run at load < 10 was possible; every enforced-bound run was at load
  13–17 with other agents building. Numbers are debug-build.
- The "display rest" column is pickup minus the measured stages, an estimate;
  it went negative once under load (a keystroke sample of 21 ms).
- Shadow-less panel experiment: not evaluated (load spike), reverted.
- Typing-bench style app-level capture (real key events through the window
  server) is not measured: opening the list in the app needs keystrokes into
  the app, which would steal focus or need Accessibility.
