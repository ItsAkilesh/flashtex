import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// Pure `ProjectSearch` helpers: reply parsing, termination labelling,
/// UTF-8-exact line/snippet extraction, verification, VoiceOver labels.
final class ProjectSearchPureTests: XCTestCase {
    private func byte(_ needle: String, in text: String) -> Int {
        text.utf8.distance(from: text.startIndex, to: text.range(of: needle)!.lowerBound)
    }

    func testTerminationOnlyCompleteIsExhaustive() {
        XCTAssertTrue(ProjectSearch.Termination(wire: "complete").isExhaustive)
        for wire in ["match_limit", "work_limit", "cancelled", "something_new"] {
            let t = ProjectSearch.Termination(wire: wire)
            XCTAssertFalse(t.isExhaustive, wire)
            XCTAssertTrue(t.explanation(maxMatches: 5, maxWork: 9).hasPrefix("Partial"), wire)
        }
        XCTAssertEqual(ProjectSearch.Termination(wire: "something_new"), .unknown("something_new"))
        XCTAssertEqual(ProjectSearch.Termination.matchLimit.explanation(maxMatches: 5, maxWork: 9), "Partial: the helper stopped at the match limit (5); more matches may exist.")
        XCTAssertEqual(ProjectSearch.Termination.workLimit.explanation(maxMatches: 5, maxWork: 9), "Partial: the helper stopped at the work budget (9 byte comparisons); more matches may exist.")
    }

    func testResultsSummaryNeverCallsPartialResultsComplete() {
        let loc = ShellModel.IndexLocation(["path": "main.tex", "revision": 1, "start_byte": 0, "end_byte": 1])!
        let m = ProjectSearch.Match(location: loc, line: 1, column: 1, snippet: nil)
        var r = ProjectSearch.Results(literal: "x", sourceVersions: ["main.tex": 1], matches: [m], termination: .complete, workUsed: 3, maxMatches: 1, maxWork: 10)
        XCTAssertEqual(r.summary, "1 match (complete)")
        r.termination = .matchLimit
        XCTAssertEqual(r.summary, "1 match shown — partial (match limit)")
        r.matches = []
        r.termination = .workLimit
        XCTAssertEqual(r.summary, "0 matches shown — partial (work limit)")
    }

    func testRequestClampsLimitsIntoTheHelpersRanges() {
        let p = ProjectSearch.request(sourceVersions: ["main.tex": 2], literal: "é", maxMatches: 0, maxWork: 5_000_000)
        XCTAssertEqual(p["max_matches"] as? Int, 1)
        XCTAssertEqual(p["max_work"] as? Int, 1_000_000)
        XCTAssertEqual(p["literal"] as? String, "é")
        XCTAssertEqual(p["source_versions"] as? [String: Int], ["main.tex": 2])
        XCTAssertNil(p["documents"])
        let q = ProjectSearch.request(sourceVersions: [:], literal: "a", maxMatches: 5000, maxWork: 0, documents: ["a.tex"])
        XCTAssertEqual(q["max_matches"] as? Int, 1000)
        XCTAssertEqual(q["max_work"] as? Int, 1)
        XCTAssertEqual(q["documents"] as? [String], ["a.tex"])
    }

    func testParseReplyBindsMatchesToTheReplyVersions() {
        let good: [String: Any] = [
            "source_versions": ["chapter.tex": 1, "main.tex": 3],
            "matches": [["path": "chapter.tex", "revision": 1, "start_byte": 4, "end_byte": 9],
                        ["path": "main.tex", "revision": 3, "start_byte": 0, "end_byte": 5]],
            "termination": "match_limit", "work_used": 77,
        ]
        guard case .success(let raw) = ProjectSearch.parse(reply: good) else { return XCTFail() }
        XCTAssertEqual(raw.matches.map(\.path), ["chapter.tex", "main.tex"])
        XCTAssertEqual(raw.termination, .matchLimit)
        XCTAssertEqual(raw.workUsed, 77)
        // A match at a revision that is not the reply's map is refused as a whole.
        var forged = good
        forged["matches"] = [["path": "main.tex", "revision": 2, "start_byte": 0, "end_byte": 5]]
        guard case .failure(let e) = ProjectSearch.parse(reply: forged) else { return XCTFail() }
        XCTAssertTrue(e.message.contains("names main.tex r2, not the reply's r3"), e.message)
        for missing in ["source_versions", "matches", "termination"] {
            var bad = good
            bad.removeValue(forKey: missing)
            guard case .failure(let e) = ProjectSearch.parse(reply: bad) else { return XCTFail(missing) }
            XCTAssertTrue(e.message.contains(missing), e.message)
        }
        var malformed = good
        malformed["matches"] = [["path": "main.tex", "start_byte": 0]]
        guard case .failure(let e2) = ProjectSearch.parse(reply: malformed) else { return XCTFail() }
        XCTAssertTrue(e2.message.contains("malformed"), e2.message)
    }

    func testLocateGivesLineColumnAndClusterSafeSnippetsForMultiByteText() {
        let text = "\\section{Intro}\nnaïve café — see \\ref{sec:b}; café again.\r\nRésumé\n"
        let start = byte("café —", in: text)
        guard let at = ProjectSearch.locate(start: start, end: start + "café".utf8.count, in: text) else { return XCTFail() }
        XCTAssertEqual(at.line, 2)
        XCTAssertEqual(at.column, "naïve ".utf8.count + 1)
        XCTAssertEqual(at.snippet, .init(before: "naïve ", match: "café", after: " — see \\ref{sec:b}; café again."))
        XCTAssertEqual(at.snippet.text, "naïve café — see \\ref{sec:b}; café again.")
        // A `\r\n` line: the trailing `\r` is not part of the snippet.
        let r = byte("Résumé", in: text)
        let second = byte("café again", in: text)
        guard let at2 = ProjectSearch.locate(start: second, end: second + 5, in: text) else { return XCTFail() }
        XCTAssertEqual(at2.snippet.after, " again.")
        guard let at3 = ProjectSearch.locate(start: r, end: r + "Résumé".utf8.count, in: text) else { return XCTFail() }
        XCTAssertEqual(at3.line, 3)
        XCTAssertEqual(at3.snippet, .init(before: "", match: "Résumé", after: ""))
        // Mid-scalar offsets are refused, never rounded.
        XCTAssertNil(ProjectSearch.locate(start: start + 4, end: start + 5, in: text), "inside é")
        XCTAssertNil(ProjectSearch.locate(start: 3, end: 2, in: text))
        XCTAssertNil(ProjectSearch.locate(start: 0, end: text.utf8.count + 1, in: text))
    }

    func testLocateClipsContextOnCharacterBoundariesAndShowsLineBreaksInMatches() {
        // Context of 3 characters around a match inside composed sequences: the
        // clipped `before` ends after a whole cluster (e + U+0301 is one Character).
        let cluster = "e\u{0301}"
        let text = String(repeating: cluster, count: 5) + "XY" + String(repeating: "👨‍👩‍👧", count: 4) + "\n"
        let start = byte("XY", in: text)
        guard let at = ProjectSearch.locate(start: start, end: start + 2, in: text, context: 3) else { return XCTFail() }
        XCTAssertEqual(at.snippet.before, String(repeating: cluster, count: 3))
        XCTAssertTrue(at.snippet.clippedBefore)
        XCTAssertEqual(at.snippet.after, String(repeating: "👨‍👩‍👧", count: 3))
        XCTAssertTrue(at.snippet.clippedAfter)
        XCTAssertEqual(at.snippet.text, "…" + String(repeating: cluster, count: 3) + "XY" + String(repeating: "👨‍👩‍👧", count: 3) + "…")
        // A literal spanning lines: the match shows ⏎, the line is the start line.
        let multi = "a\nb\nc\n"
        guard let m = ProjectSearch.locate(start: 2, end: 5, in: multi) else { return XCTFail() }
        XCTAssertEqual(m.line, 2)
        XCTAssertEqual(m.snippet, .init(before: "", match: "b⏎c", after: ""))
        // A match ending at end of text without a newline.
        guard let tail = ProjectSearch.locate(start: 4, end: 5, in: "ab\ncd") else { return XCTFail() }
        XCTAssertEqual(tail.line, 2)
        XCTAssertEqual(tail.snippet, .init(before: "c", match: "d", after: ""))
    }

    func testBytesSpellIsByteExact() {
        let text = "naïve café\n"
        let s = byte("café", in: text)
        XCTAssertTrue(ProjectSearch.bytesSpell("café", in: text, start: s, end: s + 5))
        XCTAssertFalse(ProjectSearch.bytesSpell("cafe\u{0301}", in: text, start: s, end: s + 5), "canonically equal, different bytes")
        XCTAssertFalse(ProjectSearch.bytesSpell("café", in: text, start: s, end: s + 4), "length differs")
        XCTAssertFalse(ProjectSearch.bytesSpell("café", in: text, start: s + 1, end: s + 6))
        XCTAssertFalse(ProjectSearch.bytesSpell("café", in: text, start: s, end: text.utf8.count + 1))
        XCTAssertTrue(ProjectSearch.bytesSpell("", in: text, start: 3, end: 3))
    }

    func testMatchesAndAccessibilityLabels() {
        let main = "one\ncafé two\n"
        let locs = [ShellModel.IndexLocation(["path": "main.tex", "revision": 1, "start_byte": 4, "end_byte": 9])!,
                    ShellModel.IndexLocation(["path": "other.tex", "revision": 2, "start_byte": 0, "end_byte": 5])!]
        let matches = ProjectSearch.matches(from: locs, texts: ["main.tex": main])
        XCTAssertEqual(matches[0].line, 2)
        XCTAssertEqual(matches[0].column, 1)
        XCTAssertEqual(matches[0].snippet?.text, "café two")
        XCTAssertEqual(matches[0].id, "main.tex@1:4..<9")
        XCTAssertEqual(matches[1].line, 0)
        XCTAssertNil(matches[1].snippet)
        XCTAssertEqual(ProjectSearch.accessibilityLabel(index: 0, count: 2, match: matches[0]), "match 1 of 2, main.tex, line 2, café two")
        XCTAssertEqual(ProjectSearch.accessibilityLabel(index: 1, count: 2, match: matches[1]), "match 2 of 2, other.tex, line unknown, text unavailable")
        XCTAssertEqual(ProjectSearch.changedVersions(from: ["a": 1, "b": 2], to: ["a": 1, "b": 3, "c": 1]), ["b r2→r3", "c r–→r1"])
    }

    @MainActor
    func testWithoutHelperSearchExplainsAndNothingIsSelected() async {
        let model = ShellModel()
        model.documents = [.init(path: "main.tex", text: "café\n")]
        model.activePath = "main.tex"
        XCTAssertFalse(model.controllerAttached)
        let client = ProjectSearchClient(model: model)
        XCTAssertFalse(client.helperAvailable)
        client.query = "café"
        await client.search()
        XCTAssertNil(client.results)
        XCTAssertEqual(client.status, ProjectSearch.noHelperMessage)
        XCTAssertTrue(client.status.contains("requires the durable helper"))
        await client.navigateToSelected()
        XCTAssertEqual(client.status, "No match selected.")
        XCTAssertNil(model.selection)
        XCTAssertEqual(client.navigationCount, 0)
        // An empty literal is explained without touching the helper route.
        client.query = ""
        await client.search()
        XCTAssertEqual(client.status, ProjectSearch.noHelperMessage, "helper state is reported first")
        client.moveSelection(by: 1)
        XCTAssertNil(client.selectedID)
    }
}

/// Against the real `flashtex-preview-controller` helper with the real
/// compiler (`FLASHTEX_PREVIEW_CONTROLLER` + `FLASHTEX_COMPILER`; skipped
/// otherwise): a two-file project with multi-byte text, complete / limit
/// terminations, exact navigation across documents, and refusals after
/// durable and local edits.
@MainActor
final class ProjectSearchHelperTests: XCTestCase {
    static let main = "\\documentclass{article}\n\\begin{document}\n\\section{Intro}\\label{sec:a}\nnaïve café — see \\ref{sec:b}; café again.\n\\input{chapter}\n\\end{document}\n"
    static let chapter = "\\section{Chapter}\\label{sec:b}\nRésumé of café.\n% café in a comment\n"

    private func byte(_ needle: String, in text: String) -> Int {
        text.utf8.distance(from: text.startIndex, to: text.range(of: needle)!.lowerBound)
    }

    private func selected(_ model: ShellModel) -> String { (model.activeText as NSString).substring(with: model.selection!.nsRange) }

    private func waitUntil(timeout: TimeInterval = 15, _ cond: () -> Bool) async throws {
        let start = Date()
        while !cond() {
            if Date().timeIntervalSince(start) > timeout { throw XCTSkip("timeout") }
            try await Task.sleep(nanoseconds: 30_000_000)
        }
    }

    private func attachedModel() async throws -> (ShellModel, URL) {
        guard let helper = PreviewControllerTests.helper, FileManager.default.isExecutableFile(atPath: helper.path),
              ShellModel.locateCompiler() != nil else {
            throw XCTSkip("set FLASHTEX_PREVIEW_CONTROLLER and FLASHTEX_COMPILER to built binaries")
        }
        let root = FileManager.default.temporaryDirectory.appendingPathComponent("search-helper-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: root.appendingPathComponent("project"), withIntermediateDirectories: true)
        try Self.main.write(to: root.appendingPathComponent("project/main.tex"), atomically: true, encoding: .utf8)
        try Self.chapter.write(to: root.appendingPathComponent("project/chapter.tex"), atomically: true, encoding: .utf8)
        setenv("FLASHTEX_CONTROLLER_LEDGER_ROOT", root.appendingPathComponent("ledger").path, 1)
        let model = ShellModel()
        model.autoCompile = true
        XCTAssertEqual(model.openTex(at: root.appendingPathComponent("project/main.tex")), .opened)
        model.attachController(at: helper)
        XCTAssertTrue(model.controllerAttached)
        try await waitUntil { model.controllerState.durable["main.tex"] != nil && model.result?.revision == model.editorRevision && model.inFlightRevision == nil }
        XCTAssertEqual(model.documents.map(\.path), ["main.tex"], "only the entry document is open in the window")
        return (model, root)
    }

    private func cleanup(_ model: ShellModel, _ root: URL) {
        unsetenv("FLASHTEX_CONTROLLER_LEDGER_ROOT")
        model.detachController()
        try? FileManager.default.removeItem(at: root)
    }

    func testCompleteSearchListsBothDocumentsWithDurableLinesAndSnippets() async throws {
        let (model, root) = try await attachedModel()
        defer { cleanup(model, root) }
        let client = ProjectSearchClient(model: model)
        XCTAssertTrue(client.helperAvailable)
        client.query = "café"
        await client.search()
        guard let r = client.results else { return XCTFail(client.status) }
        XCTAssertEqual(r.termination, .complete)
        XCTAssertEqual(r.summary, "4 matches (complete)")
        XCTAssertEqual(r.sourceVersions.keys.sorted(), ["chapter.tex", "main.tex"], "\\input{chapter} is part of the helper's project")
        XCTAssertTrue(client.status.hasPrefix("4 matches (complete) for “café” in durable source (work "), client.status)
        XCTAssertGreaterThan(r.workUsed, 0)
        // Helper order: path, then byte offset. chapter.tex was not open in the
        // window; its durable text was read through `document` for the snippets.
        XCTAssertEqual(r.matches.map(\.path), ["chapter.tex", "chapter.tex", "main.tex", "main.tex"])
        XCTAssertEqual(r.matches.map(\.line), [2, 3, 4, 4])
        XCTAssertEqual(r.matches.map(\.location.start), [byte("café.", in: Self.chapter), byte("café in", in: Self.chapter),
                                                         byte("café —", in: Self.main), byte("café again", in: Self.main)])
        XCTAssertEqual(r.matches.map { $0.location.end - $0.location.start }, [5, 5, 5, 5], "UTF-8 byte ranges")
        XCTAssertEqual(r.matches[0].snippet?.text, "Résumé of café.")
        XCTAssertEqual(r.matches[1].snippet?.text, "% café in a comment", "comments are searched")
        XCTAssertEqual(r.matches[2].snippet, .init(before: "naïve ", match: "café", after: " — see \\ref{sec:b}; café again."))
        XCTAssertEqual(r.matches[3].snippet?.before, "naïve café — see \\ref{sec:b}; ")
        XCTAssertEqual(client.selectedID, r.matches[0].id)
        XCTAssertEqual(ProjectSearch.accessibilityLabel(index: 2, count: 4, match: r.matches[2]), "match 3 of 4, main.tex, line 4, naïve café — see \\ref{sec:b}; café again.")
        XCTAssertEqual(model.documents.map(\.path), ["main.tex"], "reading durable text for snippets opens nothing")
        XCTAssertNil(client.knownStaleNote)

        // Case-sensitive: "Café" finds nothing, and says so as complete.
        client.query = "Café"
        await client.search()
        XCTAssertEqual(client.results?.matches.count, 0)
        XCTAssertEqual(client.results?.termination, .complete)
        XCTAssertEqual(client.results?.summary, "0 matches (complete)")
        XCTAssertNil(client.selectedID)
    }

    func testMatchAndWorkLimitsAreExplicitAndNeverLabelledExhaustive() async throws {
        let (model, root) = try await attachedModel()
        defer { cleanup(model, root) }
        let client = ProjectSearchClient(model: model)
        client.query = "café"
        client.maxMatches = 1
        await client.search()
        guard let limited = client.results else { return XCTFail(client.status) }
        XCTAssertEqual(limited.termination, .matchLimit)
        XCTAssertEqual(limited.matches.count, 1)
        XCTAssertEqual(limited.maxMatches, 1)
        XCTAssertEqual(limited.summary, "1 match shown — partial (match limit)")
        XCTAssertFalse(limited.termination.isExhaustive)
        XCTAssertTrue(client.status.hasPrefix("1 match shown — partial (match limit) for “café”"), client.status)

        // Reaching the limit is `match_limit` even when no further match exists
        // (the helper stopped there and does not claim otherwise), unless the
        // last match ends at the very end of the last document: 4 matches with
        // limit 4 → match_limit (partial), limit 5 → complete.
        client.maxMatches = 4
        await client.search()
        XCTAssertEqual(client.results?.termination, .matchLimit, client.status)
        XCTAssertEqual(client.results?.summary, "4 matches shown — partial (match limit)")
        client.maxMatches = 5
        await client.search()
        XCTAssertEqual(client.results?.termination, .complete, client.status)
        XCTAssertEqual(client.results?.summary, "4 matches (complete)")

        // Out-of-range limits are clamped, not refused: 0 → 1 match.
        client.maxMatches = 0
        await client.search()
        XCTAssertEqual(client.results?.matches.count, 1)
        XCTAssertEqual(client.results?.termination, .matchLimit)

        // A work budget too small to finish: explicit work_limit, partial.
        client.maxMatches = 1000
        client.maxWork = 1
        await client.search()
        guard let starved = client.results else { return XCTFail(client.status) }
        XCTAssertEqual(starved.termination, .workLimit)
        XCTAssertEqual(starved.matches.count, 0)
        XCTAssertEqual(starved.summary, "0 matches shown — partial (work limit)")
        XCTAssertEqual(starved.workUsed, 1)
        XCTAssertTrue(starved.termination.explanation(maxMatches: 1000, maxWork: 1).contains("work budget (1 byte comparisons)"))
    }

    func testNavigatesExactlyToMultiByteMatchesAcrossDocuments() async throws {
        let (model, root) = try await attachedModel()
        defer { cleanup(model, root) }
        let client = ProjectSearchClient(model: model)
        client.query = "café"
        await client.search()
        guard let r = client.results, r.matches.count == 4 else { return XCTFail(client.status) }

        // Return on the first (chapter.tex) match: the document is opened in the
        // window at its durable revision and the exact bytes are selected.
        client.submit()
        try await waitUntil { client.navigationCount == 1 || client.status.hasPrefix("Not navigated") }
        XCTAssertEqual(client.navigationCount, 1, client.status)
        XCTAssertEqual(model.activePath, "chapter.tex")
        XCTAssertEqual(model.documents.map(\.path), ["main.tex", "chapter.tex"])
        XCTAssertEqual(selected(model), "café")
        XCTAssertEqual(model.activeText.utf8ByteRange(of: model.selection!.nsRange)?.start, byte("café.", in: Self.chapter))
        XCTAssertEqual(model.caretUTF16, model.selection!.nsRange.location)
        XCTAssertEqual(model.caretLengthUTF16, 4, "UTF-16 length of café")
        XCTAssertTrue(client.status.hasPrefix("Match 1 of 4 for “café”: chapter.tex bytes"), client.status)
        XCTAssertTrue(client.status.contains("opened chapter.tex at durable r1"), client.status)

        // ↓ ↓ then Return: the third match, back in main.tex after "naïve " (multi-byte before it).
        client.moveSelection(by: 1)
        client.moveSelection(by: 1)
        XCTAssertEqual(client.selectedIndex, 2)
        await client.navigateToSelected()
        XCTAssertEqual(client.navigationCount, 2, client.status)
        XCTAssertEqual(model.activePath, "main.tex")
        XCTAssertEqual(selected(model), "café")
        let expectedByte = byte("café —", in: Self.main)
        XCTAssertEqual(model.activeText.utf8ByteRange(of: model.selection!.nsRange)?.start, expectedByte)
        XCTAssertEqual(model.selection?.nsRange, NSRange(location: (Self.main as NSString).range(of: "café —").location, length: 4))
        XCTAssertTrue(client.status.hasPrefix("Match 3 of 4 for “café”: main.tex bytes \(expectedByte)..<\(expectedByte + 5)"), client.status)

        // Arrow keys clamp at both ends.
        client.moveSelection(by: 10)
        XCTAssertEqual(client.selectedIndex, 3)
        client.moveSelection(by: -10)
        XCTAssertEqual(client.selectedIndex, 0)
        // Return with an unchanged query navigates; a changed query searches.
        client.query = "Résumé"
        client.submit()
        try await waitUntil { client.results?.literal == "Résumé" }
        XCTAssertEqual(client.results?.matches.count, 1)
        XCTAssertEqual(client.results?.matches.first?.snippet, .init(before: "", match: "Résumé", after: " of café."))
        await client.navigateToSelected()
        XCTAssertEqual(selected(model), "Résumé")
        XCTAssertEqual(model.activePath, "chapter.tex")
    }

    func testNavigationIsRefusedAfterLocalOverlappingEditAndAfterDurableEdit() async throws {
        let (model, root) = try await attachedModel()
        defer { cleanup(model, root) }
        let client = ProjectSearchClient(model: model)
        client.query = "café"
        await client.search()
        guard let r = client.results, r.matches.count == 4 else { return XCTFail(client.status) }
        let versions = r.sourceVersions

        // A local edit (not submitted: autoCompile off) that overwrites the first
        // main.tex match: navigating there is refused, the byte-exact rule.
        model.autoCompile = false
        model.updateActiveText(Self.main.replacingOccurrences(of: "café —", with: "cafe —"))
        let before = model.selection
        await client.navigate(to: 2, of: r)
        XCTAssertEqual(model.selection, before)
        XCTAssertEqual(client.navigationCount, 0)
        XCTAssertTrue(client.status.hasPrefix("Not navigated — Match 3 of 4 for “café”: edited since the index was built"), client.status)
        XCTAssertTrue(client.status.contains("overlap the edit at"), client.status)
        // The second main.tex match lies after the edit: rebased byte-exactly
        // (the navigateExactly rule) and re-verified, so it still navigates.
        await client.navigate(to: 3, of: r)
        XCTAssertEqual(client.navigationCount, 1, client.status)
        XCTAssertEqual(selected(model), "café")
        XCTAssertEqual(model.activeText.utf8ByteRange(of: model.selection!.nsRange)?.start, byte("café again", in: model.activeText))
        XCTAssertTrue(client.status.contains("rebased from"), client.status)
        XCTAssertNil(client.knownStaleNote, "nothing durable changed yet")

        // Once the edit is durable the helper's versions moved: every match of
        // this search is refused as stale (the helper's own rule), including
        // one in the untouched chapter.tex, until a new search runs.
        model.autoCompile = true
        model.controllerSubmitEdit()
        try await waitUntil { model.controllerState.durable["main.tex"]?.revision ?? 0 > versions["main.tex"]! && model.inFlightRevision == nil }
        XCTAssertEqual(client.knownStaleNote, "Project changed since this search (main.tex r1→r2); search again.")
        let before2 = model.selection
        await client.navigate(to: 0, of: r)
        XCTAssertEqual(model.selection, before2)
        XCTAssertEqual(client.navigationCount, 1)
        XCTAssertEqual(client.status, "Match 1 of 4 for “café”: project changed since this search (main.tex r1→r2); search again.")
        // The helper refuses the old version map too.
        let stale = await model.controllerRequest("search_literal", ProjectSearch.request(sourceVersions: versions, literal: "café", maxMatches: 10, maxWork: 1000))
        guard case .failure(let e) = stale else { return XCTFail("\(String(describing: stale))") }
        XCTAssertTrue(e.message.contains("source versions changed"), e.message)

        // A fresh search sees the durable edit: three matches now.
        await client.search()
        XCTAssertEqual(client.results?.matches.count, 3, client.status)
        XCTAssertEqual(client.results?.sourceVersions["main.tex"], versions["main.tex"]! + 1)
        XCTAssertNil(client.knownStaleNote)
        await client.navigateToSelected()
        XCTAssertEqual(client.navigationCount, 2, client.status)
        XCTAssertEqual(model.activePath, "chapter.tex")
        XCTAssertEqual(selected(model), "café")

        // A forged result whose bytes no longer spell the literal is refused
        // before anything is selected.
        var forged = client.results!
        forged.literal = "xyzzy"
        await client.navigate(to: 0, of: forged)
        XCTAssertEqual(client.navigationCount, 2)
        XCTAssertTrue(client.status.contains("no longer spell the literal"), client.status)
    }
}
