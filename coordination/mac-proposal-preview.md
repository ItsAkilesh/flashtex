# mac-proposal-preview (subagent of mac-claude-a, mac-m1max-a)

Agent / task / branch: mac-proposal-preview / shadow-compile preview of capture
proposals in the review sheet / `agent/mac-claude-a/proposal-preview`
(based on `agent/mac-claude-a/mac-shell`; merged through cf98810)
State: ready for integration (into mac-shell by the parent)
Owned paths: `apps/mac/Sources/FlashTeXMac/ProposalPreview.swift`,
`apps/mac/Tests/FlashTeXMacTests/ProposalPreviewTests.swift`; hook lines in
`ContentView.swift` (ProposalReviewSheet only); additive `%diag:<n>` / `%slow`
directives in `Tests/FlashTeXMacTests/Fixtures/fake_worker.py`.
Main integrated through: mac-shell cf98810 (not merged to main here).
Peer revisions reviewed and adaptations: mac-shell cf98810 (Latin Modern face;
`PDFExport.render` signature unchanged, thumbnail unaffected) — merged, 103 tests pass.

Ready behavior:
- `ProposalPreview` (sheet-owned ObservableObject): builds the shadow text at
  the resolved anchor with `Insertion.resolve` / `Insertion.insertionText`
  (same rules as approval), compiles it on a SEPARATE `WorkerClient` on the
  compiler `ShellModel.locateCompiler()` finds (ids `preview-N`, project id
  `<project>-preview`), debounced 300 ms and coalesced (one in flight; newest
  text follows). A baseline compile of the UNMODIFIED documents runs once per
  document revision; "new diagnostics" = shadow diagnostics with no baseline
  counterpart (severity+message+range, ranges after the insertion shifted back).
- Report: status, page count, page delta vs baseline, nearby diagnostics
  (inserted text ± enclosing lines ± one line), count of unrelated ones,
  fragment text + proposal byte offset for each new diagnostic.
- `ProposalPreviewView` under the LaTeX editor; `ProposalApproveWarning` glyph
  beside Approve when new errors exist (approval is never blocked). Worker
  terminates on sheet dismiss (`onDisappear`). Crash → "preview failed" + Retry.
- No compiler → "no compiler attached — cannot preview"; never launches.
- The real `ShellModel.result` / `editorRevision` / documents are untouched
  (tested).
- Thumbnail (PDFKit over `PDFExport.render` of only the shadow page whose text
  items map the inserted text) shown beside the report; labeled as the shadow
  page, not the live preview. Worker crash mid-compile / error envelope →
  "preview failed: …" with Retry (relaunches when the process is gone).
Incomplete behavior: thumbnail is only available when the compiler maps an
item to the inserted text (none for a failed compile); fault handling is
relaunch + Retry only (no automatic backoff). No README
entry yet (parent owns `apps/mac/README.md`).
Interface changes and required consumer actions: none to contracts. Fake worker
gains `%diag:<n>` (error diagnostic at directive byte + n) and `%slow`
(400 ms delay); existing tests unaffected (101/101 pass).
Validation: `swift build`, `swift test` (103 tests after merge, 3 skipped, 0 failures),
`xcodebuild -scheme FlashTeXMac -destination 'platform=macOS' build` succeeded.
Needs from others: parent to merge into mac-shell and add a README line.
Next action: none; awaiting parent review/merge.
Updated: 2026-09-12
