# mac-navigation-3 handoff — navigation follow-ups (unopened document, caret in math, no-glyph refusal)

- Updated UTC: 2026-09-12T18:25Z (resumed after quota cut ~17:25Z)
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

## Durable checkpoint

- Branch `agent/mac-navigation-3/unopened-math`; base 82749c26; consumed
  main as merged into mac-shell 82749c26.
- Dirty files (uncommitted at resume): PreviewV2View.swift, ShellModel+DisplayCandidates.swift,
  FlashTeXProtocol/RenderingV2.swift, DisplayCandidateTests.swift (modified);
  MathCaretHighlight.swift, ShellModel+UnopenedNavigation.swift,
  MathCaretHighlightTests.swift, NoGlyphRefusalTests.swift, Fixtures/display-list-v2-math-nav.{json,tex},
  Fixtures/display-list-v2-no-glyph-cluster{.json,-main.tex,-chapter.tex} (new).
- Next commands: `swift build` + run MathCaretHighlightTests/NoGlyphRefusalTests/DisplayCandidateTests;
  commit; write UnopenedNavigationTests.swift (real helper + render, \input); merge origin mac-shell 9ba9851c; push.
- Decisions: formula box = union of every item (glyph clusters + rules) on
  the page whose source span is exactly the fanned-out span containing the
  caret byte, used only when no 1:1 cluster contains the byte; single-cluster
  non-1:1 spans keep the documented whole-cluster fallback (unchanged tests).
