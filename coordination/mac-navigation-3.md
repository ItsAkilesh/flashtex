# mac-navigation-3 handoff — navigation follow-ups (unopened document, caret in math, no-glyph refusal)

- Updated UTC: 2026-09-12T18:40Z (resumed after quota cut ~17:25Z; done)
- Agent / parent / machine: `mac-navigation-3` (Claude Code subagent) / parent
  `mac-claude-a` / `mac-m1max-a`
- Task: Commander replenishment (issue #2 comment 5646989044), Navigation
  item 8 follow-ups not started by mac-navigation-2: (1) preview→source into a
  document that is NOT open; (2) source→preview for a caret inside math;
  (3) the `😀shuffle` producer finding (validator refusal names the source
  span, v1 fallback offered, exact reproduction filed).
- Owned paths (new files per feature): `apps/mac/Sources/FlashTeXMac/ShellModel+UnopenedNavigation.swift`,
  `apps/mac/Sources/FlashTeXMac/MathCaretHighlight.swift`,
  `apps/mac/Tests/FlashTeXMacTests/UnopenedNavigationTests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/MathCaretHighlightTests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/Fixtures/display-list-v2-math-nav.{json,tex}`,
  this file, `coordination/agents/mac-navigation-3.json`. Files created by
  finished lanes in this area (merged into mac-shell) that this lane edits:
  `PreviewV2View.swift`, `GlyphRunRenderer.swift`, `ShellModel+DisplayCandidates.swift`,
  `Navigation.swift`, `FlashTeXProtocol/RenderingV2.swift`, `DisplayCandidateTests.swift`.
  Parent-retained files stay diff requests (see the final report).
- Branch: `agent/mac-navigation-3/unopened-math` from
  `origin/agent/mac-claude-a/mac-shell` 82749c26.
- Worktree: `.claude/worktrees/agent-a50fd432339d8dfc0`.

## Coverage audit (≤10 min, done first)

Already covered on mac-shell 82749c26 (file:test):

- Preview→source into an OPEN other document (v1 and v2, switching
  documents): `NavigationTests.swift:testEveryItemNavigatesToExactlyItsBytesAndSwitchesDocuments`,
  `NavigationMultiFileV2Tests.swift:testHelperMultiFileClustersNavigateToExactBytesInBothDocuments`
  (real helper + real flashtex-render, `\input{chapter}` opened via
  `project.openDiscoveredIncludes()` BEFORE any click).
- Helper ⌘⇧D (project index) into a document not open in the window opens
  it at the reported durable revision: `Navigation.swift:selectIndexLocation`
  (`controller.document` read, "opened X at durable rN"), exercised by
  `NavigationTests.swift:testHelperResolvesLabelsCitationsAndCommandsProjectWide`.
  That is the index route (a `navigate` reply with an explicit revision), not
  a preview hit.
- Opening a rooted document through the helper with the exact snapshot:
  `ProjectDocumentsTests.swift:testHelperRouteOpensWithExactSnapshotAndDetaches`,
  `testHelperSyncAttachesDocumentsOpenedBeforeTheController`.
- The first display candidate of a project whose include the helper
  discovered at startup reads the include's durable text before validation:
  `ShellModel+DisplayCandidates.swift:displayCandidatesStartPending` (lane
  mac-navigation-2; measured "no durable text retained for chapter.tex r1").
- Caret → v2 cluster (text): exact caret for 1:1 clusters, whole-cluster
  fallback inside a ligature: `PreviewV2Tests.swift:testCaretMapsToExactClusterCaretOrWholeClusterFallback`;
  caret inside ligatures on the real producer's frame:
  `NavigationMultiFileV2Tests.swift:testHelperMultiFileClustersNavigateToExactBytesInBothDocuments`.
- Caret → v1 items (⌘⇧J) incl. other documents: `NavigationTests.swift:testRevealCaretInPreviewInsideClustersAndOtherDocuments`,
  `CaretSyncTests.swift:*`.
- Math display lists decode/validate/export (typed rules, math font by hash):
  `RenderingV2Tests.swift:testDecodesRealPipelineRulesAndMathFontManifest`,
  `PreviewV2Tests.swift:testRealMathDisplayListWithTypedRulesExportsPixelIdentical`
  (fixture `display-list-v2-math-rules.json`: `\frac` only; every math
  cluster and rule carries the WHOLE formula span, no caret navigation).
- Validator refuses a cluster with no glyph: `RenderingV2.swift` L461
  (`"N cluster(s) have no glyph"`; no cluster index, text or source span
  named; no typed `source`). No test for that refusal text found.
- Live report `tools/native-validation/mac-live/reports/20260912T110944Z.md`
  and `docs/evidence/*`: no unopened-document navigation, no caret-in-math
  highlight evidence; `😀shuffle` appears only in
  `coordination/mac-navigation-2.md` (finding, repro in that lane's scratch).

NOT covered (this lane's work):

1. A v1/v2 preview hit naming `chapter.tex` while only `main.tex` is open in
   the window: `navigateExactly` refuses "No open document named chapter.tex"
   and `navigateV2` likewise; nothing opens the document at its durable
   revision, checks it against the compiled revision, or refuses typed when
   they differ.
2. A caret inside `$…$` / `\[…\]`: `V2Geometry.clusters(containing:)` returns
   every glyph cluster of the formula (all carry the formula span, none 1:1)
   and the pane paints them one by one; rules (fraction bar, `\sqrt` overbar)
   are not part of the highlight; no "enclosing formula box" and no test on
   inline/display math with `\frac`, `\sqrt`, sub/superscripts.
3. The no-glyph refusal does not name the cluster's source span, carries no
   typed `source`, and the v2 pane offers no route back to the v1 preview of
   that revision; no reproduction filed for the render-pipeline owner.

Measured producer behaviour (flashtex-render 9aaec57a, direct run,
`FLASHTEX_FONT_DIRS=apps/mac/Fonts`, request `scratchpad/nav3/mkreq.py`):
there is no `math` item kind; math glyphs are ordinary `glyph_run` items in
`LatinModernMath-Regular` / `LMRoman7` (scripts), and EVERY cluster of a formula
carries one source range = the whole formula including its delimiters
(`$a^2 + b_1 = \frac{x}{y}$` = main.tex 48..<73; `\[ … \]` = 108..<147;
chapter.tex `$e^{i\pi} + 1 = 0$` = 8..<26). Fraction bars and the `\sqrt`
overbar are `rule` items with the same formula span. The v1 sibling maps each
math glyph item to the whole formula span too. `\sqrt` outside math is a
compiler error ("\sqrt requires math mode") and its argument is typeset as
text with exact per-letter spans.

## What this lane added (all tests on the REAL helper + flashtex-render 9aaec57a)

Commits on `agent/mac-navigation-3/unopened-math`: 2eaebc3c (items 1–3 code,
items 2–3 tests), a29c82de (merge of mac-shell 9ba9851c, clean), cc473ca2
(item 1 tests + comparator fix). No parent-retained file was touched: no diff
requests for the parent.

1. **Preview→source into an unopened document** —
   `apps/mac/Sources/FlashTeXMac/ShellModel+UnopenedNavigation.swift` (new):
   `UnopenedNavigation.Verdict` (opened / alreadyOpen / notAttached /
   noCompiledRevision / openRefused / revisionDiffers / textDiffers, each with
   a note naming the document), `ShellModel.openForNavigation(path:compiledRevision:compiledSHA256:)`
   (opens through `project.openDocument` = the helper's exact snapshot, never
   disk; places nothing unless the helper's durable revision equals the
   revision the applied preview was compiled from and, for v2, the opened
   text hashes to the list's declared sha256), `navigateOpeningIfNeeded(to:)`
   (v1 route) and `navigateV2Opening(_:)` (v2 route; `navigateV2` routes such
   hits through it asynchronously, note "Opening chapter.tex through the
   helper…"). Tests `UnopenedNavigationTests.swift` (3): typed verdicts with
   no helper; helper opens chapter.tex at durable r1 from a v2 cluster hit and
   from a v1 item (exact bytes of `s`, `ffl`, and the include's whole-formula
   span `$e^{i\pi} + 1 = 0$`); after edit+compile+detach, a hit compiled from
   r1 opens the document at r2 and is refused `revisionDiffers` (no caret,
   window stays on main.tex, document stays open at its durable text), the
   current frame then navigates exactly.
   **Measured:** the producer's `documents[].revision` is its own counter
   (5 while the helper's durable revision of chapter.tex was 1 and the frame
   revision 2) — the first draft compared it and was refused on every hit;
   the comparator is now `displayCandidates.applied?.sourceVersions[path]`
   (verified equal to the painted candidate's `source_versions` by the gate).
2. **Caret inside math** — `MathCaretHighlight.swift` (new):
   `V2Geometry.caretHighlights(containing:path:in:)` → `.cluster` (unchanged
   exact caret / whole-cluster contract) or `.formula(FormulaBox)` = union of
   every glyph cluster AND rule on the page carrying exactly the fanned-out
   span containing the caret; an exact 1:1 cluster always wins; a single
   non-1:1 cluster keeps the whole-cluster fallback. `PreviewV2View` paints
   the box (outline + member rects). `ShellModel.caretFormulaBoxes()` for
   tests. Fixture `Fixtures/display-list-v2-math-nav.{json,tex}` from the real
   producer. Tests `MathCaretHighlightTests.swift` (5): inline `\frac` +
   sub/superscripts (8 clusters + 1 rule, items 1…8, every byte of the
   formula incl. delimiters), `\sqrt` overbar in the box, display
   `\sum`/`\frac{\sqrt{i}}{2}` (8 clusters + 2 rules), `$x$` and `fi`
   ligature fallbacks, exact-cluster-wins, model-level box under the caret.
3. **No-glyph refusal** — `FlashTeXProtocol/RenderingV2.swift`: the
   `cluster(s) have no glyph` refusal now names each orphaned cluster's
   index, text and source span(s) and carries the first span typed
   (`ValidationError.source`); `ShellModel+DisplayCandidates.swift`:
   `DisplayCandidateValidator.Outcome.refused(_, source:)`,
   `DisplayCandidateRefusal` recorded as `lastInvalidCandidate` (cleared when
   a later candidate paints); `PreviewV2View`: refusal views show
   "Go to source (chapter.tex bytes 13..<17)" (routes through
   `navigateOpeningIfNeeded`) and state that the v1 preview of that revision
   remains the product preview (the fallback frame). Fixture
   `Fixtures/display-list-v2-no-glyph-cluster{.json,-main.tex,-chapter.tex}`.
   Tests `NoGlyphRefusalTests.swift` (2).

Test evidence: `docs/evidence/mac-navigation-3-tests-20260912T1836Z.log` —
46 tests across UnopenedNavigation/MathCaretHighlight/NoGlyphRefusal/
NavigationMultiFileV2/DisplayCandidate/PreviewV2/RenderingV2, 1 skipped
(needs main's flashtex-compiler), 0 failures, with
FLASHTEX_PREVIEW_CONTROLLER=crates/preview-controller/target/release (main
checkout build 11:23) and FLASHTEX_RENDER=flashtex-render 9aaec57a. `swift
build` clean. Full `swift test` NOT run: 1-min load was 50–317 during this
session (the brief's threshold is 15).

### Producer reproduction for the render-pipeline owner (item 3)

flashtex-render 9aaec57a, run directly, `FLASHTEX_FONT_DIRS=apps/mac/Fonts`,
one JSONL request on stdin:

```json
{"protocol_version":1,"id":"nav3-1","type":"compile","payload":{"project_id":"p","revision":1,"entry_path":"main.tex","documents":[{"path":"main.tex","text":"\\documentclass{article}\n\\begin{document}\nCaf\u00e9 office \\input{chapter}\nend.\n\\end{document}\n"},{"path":"chapter.tex","text":"R\u00e9sum\u00e9 e\u0301 \ud83d\ude00shuffle ffl fi waffle.\n"}],"layout_capabilities":["display-list-v2"]}}
```

Observed (the display list is the committed fixture
`display-list-v2-no-glyph-cluster.json`): page 1 item 4 is ONE glyph_run
with text `😀shuffle` and 6 clusters; cluster 0 (`😀`, chapter.tex bytes
13..<17) is referenced by no glyph while clusters 1–5 (s h u ffl e) carry
the 5 glyphs. The fail-closed Mac validator refuses the whole list
(`page 1 item 4: 1 cluster(s) have no glyph … cluster 0 “😀” (chapter.tex
bytes 13..<17)`). A missing-glyph scalar that is its own word (`end 😀 done`)
is dropped cleanly (no cluster); glued to a word it leaves an empty cluster.
Expected: either drop the cluster with the glyph (as for the standalone
case) or emit a notdef glyph for it. Not patched here (render-pipeline is
not this lane's).

## Limitations

- The v2 pane's `navigateV2` completes asynchronously for an unopened
  document; tests exercise the awaited `navigateV2Opening` directly.
- `textDiffers` (durable revision equal but sha256 differs) has no real-helper
  test: the helper never produced that state; it is defensive.
- Formula box = items carrying exactly the same span; a producer that
  emits differing spans for parts of one formula would yield several boxes.
- No live packaged-app screenshot of the formula box / refusal actions
  (Accessibility not granted; not attempted).

## Durable checkpoint

- Branch `agent/mac-navigation-3/unopened-math` tip cc473ca2 (pushed);
  base 82749c26; mac-shell 9ba9851c merged (a29c82de); consumed main as
  merged into mac-shell 9ba9851c.
- Dirty files: none after the final commit (this file + registration JSON
  are committed in the closing commit).
- Next commands (if resumed): none pending; parent merges the branch.
- Decisions: formula box = union of every item (glyph clusters + rules) on
  the page whose source span is exactly the fanned-out span containing the
  caret, used only when no 1:1 cluster contains the byte; unopened-document
  comparator = helper `source_versions` of the applied preview + declared
  sha256, never the producer's `documents[].revision`.
