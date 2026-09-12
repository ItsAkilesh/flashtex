# mac-navigation-2 handoff — exact multi-file source ⇄ preview navigation (v2 clusters, edits, revisions)

- Updated UTC: 2026-09-12T16:33Z
- Agent / parent / machine: `mac-navigation-2` (Claude Code subagent) / parent
  `mac-claude-a` / `mac-m1max-a`
- Task: Commander replenishment (issue #2 comment 5646989044), Navigation
  (item 8): verify EXACT multi-file source⇄preview navigation after
  (a) UTF-8 edits before/after the target, (b) producer ligature clusters
  ff/fi/fl/ffi/ffl (flashtex-render 9aaec57a), (c) revision changes, across
  `main.tex` + `\input{chapter}` through the helper route; then follow-ups
  1 (preview→source into a document that is not open) and 2 (caret in math).
- Owned paths: `apps/mac/Sources/FlashTeXMac/Navigation.swift`,
  `apps/mac/Sources/FlashTeXMac/CaretSync.swift`,
  `apps/mac/Tests/FlashTeXMacTests/NavigationTests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/CaretSyncTests.swift`, new files per
  feature, this file, `coordination/agents/mac-navigation-2.json`.
  Files created by finished lanes in this area that this lane may edit:
  `PreviewV2View.swift`, `GlyphRunRenderer.swift` (mac-preview-v2 /
  mac-render-pipeline, merged into mac-shell), `ShellModel+DisplayCandidates.swift`
  (mac-display-candidates, merged). Parent-retained files stay diff requests.
- Branch: `agent/mac-navigation-2/multifile-nav` from
  `origin/agent/mac-claude-a/mac-shell` cd58fc2e (main c11c005 merged).
- Worktree: `.claude/worktrees/agent-a2ba7baf8a98eec29`.

## Coverage audit (≤10 min, done first)

Already covered on mac-shell cd58fc2e (file:test):

- v1 preview→source, multi-file (`main.tex` + `chapter.tex` fixture,
  `MultiFileFixture`), ligature scalar U+FB01, document switch, generated
  text: `NavigationTests.swift:testEveryItemNavigatesToExactlyItsBytesAndSwitchesDocuments`.
- v1 after edits: multi-byte (é), combining mark (e + U+0301 normalization-
  only edit is an edit), surrogate pairs (👩→👨), CRLF→LF, spans overlapping
  the edit refused with the edited bytes named, spans after it rebased and
  verified, other document untouched:
  `NavigationTests.swift:testStaleSpansAreRefusedWithTheEditedBytesAndOthersRebase`,
  `NavigationTests.swift:testRebaseExactlyMapsRefusesAndVerifies`,
  `NavigationTests.swift:testEditorRangeRefusesSplitScalarsAndWidensSplitClusters`.
- v1 real compiler multi-file spans + navigation:
  `NavigationTests.swift:NavigationRealCompilerTests.testRealCompilerSpansMatchFixtureAndNavigateExactly`
  (FLASHTEX_COMPILER).
- v1 source→preview (caret) inside clusters and in the other document:
  `NavigationTests.swift:testRevealCaretInPreviewInsideClustersAndOtherDocuments`,
  `CaretSyncTests.swift:*` (index == linear scan, surrogate halves, 10 000 items).
- Helper (project index) ⌘⇧D across files, stale versions refused:
  `NavigationTests.swift:testHelperResolvesLabelsCitationsAndCommandsProjectWide`,
  `testHelperRefusesStaleVersionsAndUnindexedBuffers` (FLASHTEX_PREVIEW_CONTROLLER + FLASHTEX_COMPILER).
- v2 cluster click → source, SINGLE file (`display-list-v2-text.json`
  fixture produced by flashtex-render; ffi in "office", `\'e`):
  `PreviewV2Tests.swift:testOpeningTheRealDisplayListNavigatesLigatureClustersToSourceBytes`,
  `testHitTestReturnsLigatureClusterWithTwoSourceBytes` (CoreText fi),
  `testCaretMapsToExactClusterCaretOrWholeClusterFallback`,
  `RenderingV2Tests.swift:testDecodesRealPipelineDisplayListAndConsumesItsFields`.
- v2 stale buffer refused (digest mismatch → no selection), synthetic
  content has no source: `PreviewV2Tests.swift:testStaleBufferIsRefusedAndSyntheticContentHasNoSource`.
- v2 live frame (fake worker template) navigates ffi:
  `PreviewV2Tests.swift:testLiveFrameArrivesWithTheCompileResultAndNavigates`.
- Helper route + real flashtex-render, SINGLE file, candidates bound to
  editor revisions, no click navigation:
  `DisplayCandidateTests.swift:testHelperCandidatesPaintAndBindToEditorRevisions`,
  `V2ConformanceTests.swift:testRealHelperRefusesACandidateFromBeforeAnOpenDocument`
  (opens appendix.tex by hand, not via `\input`; no navigation).
- Live report `tools/native-validation/mac-live/reports/20260912T110944Z.md`:
  no multi-file v2 click evidence; ligature rows are PDF text-extraction
  only. `docs/evidence/multifile-project-2026-09-12/`: include opening via
  `FLASHTEX_OPEN_INCLUDES=1`, caret highlight at byte 0 of ch/section.tex
  (v1), no v2 cluster navigation.

NOT covered (this lane's work):

1. v2 cluster → source across `main.tex` + `\input{chapter}` through the real
   helper + real flashtex-render (`FLASHTEX_PREVIEW_CONTROLLER`,
   `FLASHTEX_RENDER`), includes opened the way `FLASHTEX_OPEN_INCLUDES=1`
   does (`project.openDiscoveredIncludes()`): every cluster on the page
   selects exactly its source bytes in the right document (switching
   documents), for all five producer ligature clusters ff/fi/fl/ffi/ffl in
   both files, with multi-byte (é, ï, —), combining-mark (e + U+0301) and
   surrogate-pair (emoji; missing glyph in Latin Modern, dropped by the
   producer) text before and after them.
2. v2 after edits before/after the target: a stale frame (buffer moved on)
   REFUSED everything before this lane; the v1 route rebases across the
   recorded edit. Rebase-or-refuse for v2 through the recorded compile
   text when its digest is the one the list attests (never wrong bytes).
3. v2 revision change: the next candidate's clusters carry the shifted
   bytes and navigate exactly; an old hit is never applied silently.

Measured producer behaviour (flashtex-render 9aaec57a, direct run,
`FLASHTEX_FONT_DIRS=apps/mac/Fonts`, request in this lane's scratch): both
documents declared (`main.tex` r1 142 bytes, `chapter.tex` r1 74 bytes),
every cluster carries one exact source range; ligature clusters are one
glyph over 2–3 source bytes (ff/fi/fl 2, ffi/ffl 3) in both files;
`e` + U+0301 is one 3-byte cluster; 😀, 👨‍👩 and 当 produce
`missing_glyph` diagnostics and no item, the following items keep exact
byte spans.

## What this lane added (branch agent/mac-navigation-2/multifile-nav)

- `apps/mac/Tests/FlashTeXMacTests/NavigationMultiFileV2Tests.swift` (2 tests,
  real `flashtex-preview-controller` + real `flashtex-render`, skip otherwise;
  load-aware 40 s waits skip with the model state in the message):
  - `testHelperMultiFileClustersNavigateToExactBytesInBothDocuments`: attach
    with candidates on, `project.openDiscoveredIncludes()` opens chapter.tex
    at its durable text, ⌘B; the frame declares both documents with the
    buffers' digests; EVERY cluster hit-tested at its own centre navigates to
    exactly its source bytes in its own document (main → chapter → main
    switches), the selected text is byte-identical to the cluster text; all
    five ligatures ff/fi/fl/ffi/ffl seen as one-glyph clusters in BOTH files;
    ffi of "office" = bytes o+1..<o+4, cluster 1, one glyph; e + U+0301 is one
    3-byte cluster and every byte of it maps back to that cluster; "end" after
    the dropped 😀 and "waffle" after the dropped 👨‍👩 keep exact bytes; editor
    caret inside the chapter ffl → that cluster, no exact caret (whole-cluster
    fallback), caret at its first byte → run caret text_byte 3; v1 sibling
    agrees (⌘⇧J on the caret inside the ligature selects "shuffle").
  - `testHelperEditsRebaseOrRefuseOnTheStaleFrameAndTheNextRevisionNavigatesExactly`:
    with autoCompile off (frame stale): "office"→"offÀce" refuses the ffi
    cluster (overlaps the edit, "recompile to navigate"), the é of Café before
    the edit keeps its bytes ("rebased onto the edited buffer" note), "end"
    after it shifts +1 and spells "e" ("rebased from … across edits"), the
    untouched chapter.tex navigates unchanged switching documents; ⌘B → the
    next frame: every cluster exact again, the edited word has no ffi, "end"
    at +1; then chapter.tex edited before its targets with e+U+0301 and 😀
    (+9 bytes): ffl rebases +9 on the stale frame, Résumé's é unchanged,
    main.tex unaffected; ⌘B → third frame exact, ffl at +9; a hit kept from
    the first frame never claims the edited-away "ffi".
- `Navigation.swift`: `navigateExactly(to:expectedText:compiledText:)` takes an
  explicit baseline; `compiledText(for:)` falls back to the applied preview's
  durable text (`displayCandidates.applied.sourceVersions` →
  `controllerState.textByDurable`) for an include the helper compiled before
  this window read it (v1 preview→source into such an include was refused
  "no compiled text is recorded" while any edit was pending).
- `PreviewV2View.swift` `navigateV2`: a stale frame (buffer digest ≠ declared)
  is rebased through the recorded compile text when THAT carries the declared
  digest (`Navigation.rebaseExactly`: refused on overlap, verified bytes),
  else refused as before (note now says "…and from the recorded compile text").
- `ShellModel+DisplayCandidates.swift`:
  1. `displayCandidatesStartPending`: a candidate naming a version this session
     has not read (the first candidate of a project whose include the helper
     discovered at startup: "invalid preview-4: no durable text retained for
     chapter.tex r1", measured) requests `document` from the helper once,
     holds the candidate pending (bounded 2 s), then validates.
  2. `DisplayCandidateGate`: refuses a candidate whose membership generation is
     OLDER than the last learned one (was: not equal). Measured: after
     `openDiscoveredIncludes` adopted g3, the candidate after the first edit
     named g4 and was refused "membership generation 4 is not the project's
     current generation 3" — the helper's `membership_generation` is its
     project-index generation (`crates/project-index/src/lib.rs check_update`),
     advanced by every durable edit, so equality refused every v2 frame after
     the first keystroke in any project with an opened include.
- `V2ConformanceTests.swift` gate test: a newer generation is admitted (was
  asserted refused); the older-generation D2 refusal and the real-helper D2
  test are unchanged and pass.

## Findings for other owners (not patched here)

- flashtex-render 9aaec57a (render-pipeline lane): a missing-glyph scalar
  INSIDE a word — chapter.tex text `e\u{301} \u{1F600}shuffle` (😀 glued to
  "shuffle") — produces a display list the fail-closed validator refuses:
  `[invalid_display_list] page 1 item 16: 1 cluster(s) have no glyph (every
  cluster needs at least one glyph)`; the v1 sibling is fine. A standalone
  emoji word is dropped cleanly (missing_glyph diagnostic, no item). Exact
  repro: the request in this lane's scratch (`mkreq.py`) with "😀shuffle".
- Model-level API note: `navigateV2(_ hit:)` trusts the hit to come from the
  frame on screen (the pane computes it from its own `page` in the tap
  closure, so a user click is always current); a hit retained across a
  republish resolves against the new frame's attestation. Binding the hit to
  the frame nonce would close that at the API level (not done: not reachable
  from the UI).

## Durable checkpoint

- Branch `agent/mac-navigation-2/multifile-nav`; base cd58fc2e; consumed
  main c11c005 (as merged into mac-shell). Pushed to origin after the commit.
- Product commit: c4523085 (tests + fixes); dirty files: none after the coord commit.
- Evidence: `swift build --build-tests` OK; NavigationMultiFileV2Tests 2/2
  passed with `FLASHTEX_PREVIEW_CONTROLLER=crates/preview-controller/target/release/flashtex-preview-controller`
  (main checkout) and `FLASHTEX_RENDER=<worktree agent-ad8b60e180d0f9722>/.scratch/render-9aaec57a/crates/render-pipeline/target/release/flashtex-render`
  (0.19 s / 0.69 s); touched suites Navigation*/CaretSync/PreviewV2/
  DisplayCandidate/V2Conformance/NavigationMultiFileV2: 45 tests, 0
  failures (0–2 load skips) at 1-min load 24–42. Full `swift test` NOT run
  (1-min load never below 15 during the lane; 25–68 measured).
- Follow-ups 1 and 2: not started (bound reached).
