# mac-editor-diagnostics (Claude Code subagent, parent mac-claude-a)

- Updated UTC: 2026-09-12T08:40Z
- Agent / parent / machine alias: mac-editor-diagnostics / mac-claude-a / mac-m1max-a
- Task / acceptance gate / owned paths: lane "Partial recovery error marks
  preserve exact source identity" + follow-ups "Stale diagnostic invalidation
  without editor blocking" and "Accessible keyboard error navigation" (gate 3:
  error recovery and source navigation). Owned:
  `apps/mac/Sources/FlashTeXMac/EditorDiagnostics.swift`,
  `apps/mac/Tests/FlashTeXMacTests/EditorDiagnosticsTests.swift`,
  `apps/mac/Sources/FlashTeXAccessibility/EditorDiagnosticsAccessibility.swift` (new),
  `apps/mac/Tests/FlashTeXAccessibilityTests/EditorDiagnosticsAccessibilityTests.swift` (new),
  this handoff and `coordination/agents/mac-editor-diagnostics.json`.
- Branch / code revision / main integrated through:
  `agent/mac-editor-diagnostics/exact-marks` from
  `origin/agent/mac-claude-a/mac-shell` 6b43a3a (which contains main 2fd3026).
- State: ready for integration (parent review; consumer diffs below not applied).

## Ready behavior and evidence

- `EditorDiagnostics.Identity` = result envelope id + index into
  `CompileResult.diagnostics` + the worker's original byte span (`key` =
  `"<id>#<index>@<path>:<start>..<end>"`). Every `Mark` carries it (`mark.id`,
  `mark.diagnosticIndex`, `mark.originalSource`); the identity never changes
  when the range is rebased, and two identical diagnostics on the same bytes
  stay distinct. `EditorDiagnostics.identity(resultID:index:in:)` lets the
  diagnostics list row compute the same key (nil for unsourced diagnostics).
- `EditorDiagnostics.report(for:resultID:path:compiledText:currentText:) ->
  Report { marks, stale, edit }`: one `SourceMapping.changedRegion` per call;
  spans overlapping the edit are withheld (never stretched) and listed in
  `stale` with identity/severity/message; `staleCount`, `staleIdentities`
  and `staleNote` ("1 error and 1 warning under edited text not underlined
  until the next compile") are ready for the footer/list. The old
  `marks(for:path:compiledText:currentText:)` still compiles (resultID
  defaults to nil) so ShellModel/SourceEditorView build unchanged.
- Recovered results never hide the error: severity is untouched and every
  mark of a `recovered` result has a `recoveryLine` — `"recovery: <note>"`
  or `"no provisional rendering"` — shared by `toolTip`
  (`"message\n↳ recovery: …"`), `spokenDescription` and the list row text.
  `ok`/`failed` results show a recovery line only when the worker gave one.
- Multi-byte safety: offsets inside a scalar are refused as before; valid
  ranges are widened outward to grapheme clusters
  (`String.clusterAlignedNSRange(utf8Start:utf8End:)`), so a combining mark
  typed after a marked character joins the underline and a span inside a
  ZWJ emoji sequence covers the whole sequence (tests cover both).
- Stale invalidation cost (`testRebaseOfLargeDocumentIsFast`, 62 000-byte
  document, 200 diagnostics, one keystroke mid-document, 25 calls, fresh
  String instance each call): **debug best 0.826 ms / median 0.829 ms /
  worst 0.893 ms; release best 0.245 ms / median 0.245 ms / worst 0.322 ms**
  (Apple M1 Max, `swift test` and `swift test -c release -Xswiftc
  -enable-testing`). Asserted `< 2 ms` on the best sample in every build.
- Keyboard error navigation model (`FlashTeXAccessibility.EditorDiagnosticNavigation`):
  `ordered` (start offset, longer range first, then caller order), `step`
  (next/previous with wrap; by identity when `currentID` is given so marks
  sharing a start are each visited), `current` (mark under the caret),
  `summary` ("1 error, 2 warnings"); `Step.announcement` =
  `"Error 2 of 5, line 12: message — recovery: … (wrapped to start)"`.
  `EditorDiagnostics.navigationItems(_:)` / `EditorDiagnostics.step(_:fromUTF16:forward:currentID:in:)`
  bridge marks to it (line numbers via `AccessibleEditorModel`).
- Validation: `swift test` in `apps/mac` with the four real worker binaries
  (`FLASHTEX_COMPILER/PDF/BRIDGE/EDIT_LEDGER` from the main checkout's
  `crates/*/target/release`, `FLASHTEX_NO_ACTIVATE=1`): **195 tests, 0
  failures, 0 skipped** at 7361570. `EditorDiagnosticsTests` 12/12,
  `EditorDiagnosticsAccessibilityTests` 3/3.

## Incomplete behavior / needs from others

- Nothing in the UI consumes the new data yet: the footer does not show
  `staleNote`, the list row does not use the identity, and ⌘⇧]/⌘⇧[ still go
  through `Navigation.stops` (which orders by compiled offsets and refuses an
  edited span with "recompile to navigate" instead of skipping it). Diffs
  below. No live VoiceOver run was possible (Accessibility permission not
  granted on this machine); the announcement text is unit-tested only.
- `SourceEditorView.applyMarks` (not owned) still keys temporary attributes
  by range only; no change needed, but the tooltip text changed from
  `"↳ note"` to `"↳ recovery: note"` to match the list row (test updated).

## Exact diffs needed in files I do not own (not applied)

`apps/mac/Sources/FlashTeXMac/ShellModel.swift` (replace the `editorMarks`
block, lines 181–196 at 6b43a3a):

```swift
    /// Diagnostic underlines for the active document, rebased across edits or
    /// withheld as stale (see `EditorDiagnostics`).
    var editorMarks: [EditorDiagnostics.Mark] { editorMarkReport.marks }

    /// Marks plus the diagnostics withheld after an edit; `staleNote` is
    /// shown in the footer. Memoized: ContentView reads this on every body
    /// evaluation and the rebase compares the compiled and current texts.
    var editorMarkReport: EditorDiagnostics.Report {
        guard let result else { return .empty }
        let key = EditorMarksKey(resultID: resultID, resultRevision: result.revision, editorRevision: editorRevision, path: activePath)
        if let cached = editorMarksCache, cached.key == key { return cached.report }
        let report = EditorDiagnostics.report(for: result, resultID: resultID, path: activePath,
                                              compiledText: compiledDocuments[activePath], currentText: activeText)
        editorMarksCache = (key, report)
        return report
    }
    private struct EditorMarksKey: Equatable { var resultID: String?; var resultRevision: Int; var editorRevision: Int; var path: String }
    @ObservationIgnored private var editorMarksCache: (key: EditorMarksKey, report: EditorDiagnostics.Report)?
    /// Identity of the mark last reached by ⌘⇧]/⌘⇧[, so marks sharing a
    /// start offset are each visited once.
    @ObservationIgnored var currentDiagnosticID: String?
```

`apps/mac/Sources/FlashTeXMac/Navigation.swift` (replace the body of
`ShellModel.goToDiagnostic(forward:)`; `Navigation.stops`/`nextStop` can then
be deleted together with their tests, or kept for the "refused" wording):

```swift
    func goToDiagnostic(forward: Bool) {
        guard let result else {
            navigationNote = "No compile result loaded; nothing to navigate to."
            return
        }
        let report = editorMarkReport
        guard let step = EditorDiagnostics.step(report.marks, fromUTF16: caretUTF16, forward: forward,
                                                currentID: currentDiagnosticID, in: activeText) else {
            let total = result.diagnostics.count
            navigationNote = total == 0 ? "Revision \(result.revision) has no diagnostics."
                : report.staleNote.map { "No diagnostic can be selected: " + $0 + "." }
                ?? "None of the \(total) diagnostic\(total == 1 ? "" : "s") has a source in \(activePath)."
            return
        }
        currentDiagnosticID = step.item.id
        selection = .init(path: activePath, nsRange: step.item.nsRange, token: (selection?.token ?? 0) + 1)
        caretUTF16 = step.item.nsRange.location
        navigationNote = step.announcement + (report.staleNote.map { "; " + $0 } ?? "")
    }
```

`apps/mac/Sources/FlashTeXMac/ContentView.swift`:

```swift
// Footer, first Text:
            Text(model.navigationNote ?? model.editorMarkReport.staleNote
                 ?? "Click text in the preview to select its source range.")
// diagnosticsList row, replace the recovery Text pair with the shared line:
                        if let line = EditorDiagnostics.recoveryLine(recovery: d.recovery, status: model.result?.status ?? .ok) {
                            Text("↳ \(line)").font(.caption).foregroundStyle(.secondary)
                        }
                        if let result = model.result,
                           let id = EditorDiagnostics.identity(resultID: model.resultID, index: i, in: result),
                           model.editorMarkReport.staleIdentities.contains(id) {
                            Text("underline withheld: span edited since the compile").font(.caption2).foregroundStyle(.orange)
                        }
```

(Behavior change to decide: today the row prints "↳ no provisional
rendering" for every diagnostic without a note, even in `ok` results; the
shared line omits it there. Keep the old `else` branch if that is wanted.
`accessibleDiagnostic` in `FlashTeXAccessibility/AccessibilityViews.swift`,
owner mac-accessibility, could take `recoveryLine` for its value instead of
rebuilding `"recovery: …"`; not required for agreement, the strings match.)

`apps/mac/Sources/FlashTeXAccessibility/AccessibilityCommands.swift`
(`.nextDiagnostic` description): "Selects the next underlined diagnostic in
document order (wrapping) and announces 'Error n of m, line l: message —
recovery'; diagnostics under edited text are skipped and counted in the footer."

## Reviewed peer revisions / adaptations

- `origin/agent/mac-claude-a/mac-shell` 6b43a3a: base. Kept the old
  `marks(for:path:compiledText:currentText:)` signature source-compatible so
  `ShellModel.editorMarks`, `SourceEditorView.applyMarks` and
  `AccessibleEditorModel.Mark` mapping build unchanged.
- `origin/main` 60a498a: coordination-only ahead of the base; no code merge needed.

## Resources / next

- Resource pool: shared Claude Max 20x quota with parent mac-claude-a; no
  purchases; usage totals unknown to this worker.
- Dirty files / running jobs: none after commit; no app launched.
- Exact next action: parent reviews 7361570+, applies the diffs above (or
  asks for a follow-up branch), re-runs `swift test` in `apps/mac` with the
  worker binaries, merges into `mac-shell`.
- Resume reading list: this file, `EditorDiagnostics.swift` header comment,
  `EditorDiagnosticsTests.swift` (bench test prints the timing line).
