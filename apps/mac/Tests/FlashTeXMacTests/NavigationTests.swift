import XCTest
@testable import FlashTeXProtocol
@testable import FlashTeXMac

final class NavigationTests: XCTestCase {
    private func byte(of needle: String, in text: String, occurrence: Int = 0) -> Int {
        var search = text.startIndex
        var found: Range<String.Index>?
        for _ in 0...occurrence {
            found = text.range(of: needle, range: search..<text.endIndex)
            guard let f = found else { break }
            search = f.upperBound
        }
        let r = found!
        return text.utf8.distance(from: text.utf8.startIndex, to: r.lowerBound)
    }

    private func slice(_ text: String, _ r: Navigation.ByteRange) -> String {
        String(text[text.rangeOfUTF8(start: r.start, end: r.end)!])
    }

    // MARK: label / ref

    func testRefGoesToLabelAndLabelCyclesReferencesWithUnicodeNames() {
        let text = "Résumé \\label{sec:naïve} text\nsee \\ref{sec:naïve} and \\eqref{sec:naïve}; \\ref{missing}\n\\label{unused}"
        let refByte = byte(of: "\\ref{sec:naïve}", in: text)
        guard case .found(let r, let note) = Navigation.matchingRange(in: text, caretByte: refByte + 3) else { return XCTFail() }
        XCTAssertEqual(slice(text, r), "\\label{sec:naïve}")
        XCTAssertTrue(note.contains("Definition of sec:naïve"), note)

        // From the label: first reference after it, then wrap to the first.
        let labelByte = byte(of: "\\label{sec:naïve}", in: text)
        guard case .found(let r1, let n1) = Navigation.matchingRange(in: text, caretByte: labelByte) else { return XCTFail() }
        XCTAssertEqual(slice(text, r1), "\\ref{sec:naïve}")
        XCTAssertTrue(n1.hasPrefix("Reference 1 of 2"), n1)
        // Caret exactly at the closing brace end is still "inside".
        let labelEnd = labelByte + "\\label{sec:naïve}".utf8.count
        guard case .found(let r2, _) = Navigation.matchingRange(in: text, caretByte: labelEnd) else { return XCTFail() }
        XCTAssertEqual(r2, r1)

        // A reference from the second ref-like command also resolves; a missing label is reported.
        let eqrefByte = byte(of: "\\eqref{sec:naïve}", in: text)
        guard case .found(let r3, _) = Navigation.matchingRange(in: text, caretByte: eqrefByte + 1) else { return XCTFail() }
        XCTAssertEqual(slice(text, r3), "\\label{sec:naïve}")
        let missing = byte(of: "\\ref{missing}", in: text)
        guard case .notFound(let why) = Navigation.matchingRange(in: text, caretByte: missing + 5) else { return XCTFail() }
        XCTAssertEqual(why, "No \\label{missing} in this document.")
        let unused = byte(of: "\\label{unused}", in: text)
        guard case .notFound(let why2) = Navigation.matchingRange(in: text, caretByte: unused + 2) else { return XCTFail() }
        XCTAssertEqual(why2, "No reference to label unused in this document.")
        // Plain text: not inside anything.
        guard case .notFound(let why3) = Navigation.matchingRange(in: text, caretByte: 0) else { return XCTFail() }
        XCTAssertTrue(why3.contains("not inside"), why3)
    }

    // MARK: begin / end

    func testBeginEndMatchingHonoursNesting() {
        let text = "\\begin{itemize}\n\\item a \\begin{itemize} \\item naïve \\end{itemize}\n\\item b\n\\end{itemize}\n\\begin{lonely}"
        let outerBegin = byte(of: "\\begin{itemize}", in: text, occurrence: 0)
        let innerBegin = byte(of: "\\begin{itemize}", in: text, occurrence: 1)
        let innerEnd = byte(of: "\\end{itemize}", in: text, occurrence: 0)
        let outerEnd = byte(of: "\\end{itemize}", in: text, occurrence: 1)

        guard case .found(let a, let noteA) = Navigation.matchingRange(in: text, caretByte: outerBegin) else { return XCTFail() }
        XCTAssertEqual(a.start, outerEnd)
        XCTAssertTrue(noteA.contains("→ \\end{itemize} at byte \(outerEnd)"), noteA)
        guard case .found(let b, _) = Navigation.matchingRange(in: text, caretByte: innerBegin + 7) else { return XCTFail() }
        XCTAssertEqual(b.start, innerEnd)
        guard case .found(let c, _) = Navigation.matchingRange(in: text, caretByte: innerEnd + 1) else { return XCTFail() }
        XCTAssertEqual(c.start, innerBegin)
        guard case .found(let d, _) = Navigation.matchingRange(in: text, caretByte: outerEnd + 12) else { return XCTFail() }
        XCTAssertEqual(d.start, outerBegin)
        XCTAssertEqual(slice(text, d), "\\begin{itemize}")

        let lonely = byte(of: "\\begin{lonely}", in: text)
        guard case .notFound(let why) = Navigation.matchingRange(in: text, caretByte: lonely + 3) else { return XCTFail() }
        XCTAssertEqual(why, "\\begin{lonely} at byte \(lonely) has no matching \\end{lonely}.")
        // An `\end` with no begin, and malformed arguments, never trap.
        guard case .notFound = Navigation.matchingRange(in: "\\end{x} \\begin{ \\end{", caretByte: 2) else { return XCTFail() }
        XCTAssertEqual(Navigation.commandUses(in: "\\begin{ \\end{\n\\ref{a").count, 0)
    }

    @MainActor
    func testGoToMatchingSelectsUTF16RangeInModel() throws {
        let model = ShellModel()
        let text = "Résumé \\begin{itemize} naïve \\end{itemize}\n"
        model.replaceProject(entryText: text)
        let ns = text as NSString
        model.caretUTF16 = ns.range(of: "\\begin").location + 2
        model.goToMatching()
        let sel = try XCTUnwrap(model.selection)
        XCTAssertEqual(ns.substring(with: sel.nsRange), "\\end{itemize}")
        XCTAssertEqual(model.caretUTF16, sel.nsRange.location)
        XCTAssertTrue(model.navigationNote?.hasPrefix("Matched \\begin{itemize}") == true, model.navigationNote ?? "nil")
        // Back again from the end.
        model.goToMatching()
        XCTAssertEqual(ns.substring(with: model.selection!.nsRange), "\\begin{itemize}")
        // Invalid caret (inside nothing) leaves the selection alone and explains.
        let before = model.selection
        model.caretUTF16 = 1
        model.goToMatching()
        XCTAssertEqual(model.selection, before)
        XCTAssertTrue(model.navigationNote?.contains("not inside") == true)
        model.caretUTF16 = ns.length + 10
        model.goToMatching()
        XCTAssertTrue(model.navigationNote?.contains("not valid") == true, model.navigationNote ?? "nil")
    }

    // MARK: diagnostics (multipage sample)

    /// Writes a copy of the multipage sample whose result carries two sourced
    /// diagnostics so cycling and wrapping can be observed. Returns the result URL.
    private func twoDiagnosticSample() throws -> URL {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("flashtex-nav-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let reqData = try Data(contentsOf: CaretSyncTests.requestURL)
        var res = try RuntimeV1.decodeCompileResult(Data(contentsOf: CaretSyncTests.resultURL))
        let req = try RuntimeV1.decodeCompileRequest(reqData)
        let text = req.payload.documents[0].text
        // Add a warning on "naïve" (bytes 66..<72 per CaretSyncTests) before the existing error at 157..<169.
        res.payload.diagnostics.insert(.init(severity: .warning, message: "Suspicious word.",
                                             source: .init(path: "main.tex", startByte: 66, endByte: 72), recovery: nil), at: 0)
        XCTAssertEqual(String(text[text.rangeOfUTF8(start: 66, end: 72)!]), "naïve")
        try reqData.write(to: dir.appendingPathComponent("nav-request.json"))
        try JSONEncoder().encode(res).write(to: dir.appendingPathComponent("nav-result.json"))
        return dir.appendingPathComponent("nav-result.json")
    }

    @MainActor
    func testNextPreviousDiagnosticCyclesWrapsAndRefusesAfterOverlappingEdit() throws {
        let model = ShellModel()
        model.loadFixtures(request: nil, result: try twoDiagnosticSample())
        XCTAssertNil(model.loadError)
        let text = model.activeText
        let ns = text as NSString
        XCTAssertEqual(model.result?.diagnostics.count, 3) // warning(naïve), error(\textbf{oops), warning(no source)

        model.caretUTF16 = 0
        model.goToDiagnostic(forward: true)
        XCTAssertEqual(ns.substring(with: model.selection!.nsRange), "naïve")
        XCTAssertTrue(model.navigationNote?.hasPrefix("Diagnostic 1 of 2 (warning)") == true, model.navigationNote ?? "nil")
        model.goToDiagnostic(forward: true)
        XCTAssertEqual(ns.substring(with: model.selection!.nsRange), "\\textbf{oops")
        XCTAssertTrue(model.navigationNote?.hasPrefix("Diagnostic 2 of 2 (error)") == true, model.navigationNote ?? "nil")
        model.goToDiagnostic(forward: true) // wraps
        XCTAssertEqual(ns.substring(with: model.selection!.nsRange), "naïve")
        model.goToDiagnostic(forward: false) // wraps backwards
        XCTAssertEqual(ns.substring(with: model.selection!.nsRange), "\\textbf{oops")
        model.goToDiagnostic(forward: false)
        XCTAssertEqual(ns.substring(with: model.selection!.nsRange), "naïve")

        // An edit after both spans: the error is rebased (same UTF-16 range since the edit is later).
        model.updateActiveText(text + "% trailing comment\n")
        model.goToDiagnostic(forward: true)
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "\\textbf{oops")
        // An edit before both spans shifts them.
        model.updateActiveText("% lead\n" + text)
        model.caretUTF16 = 0
        model.goToDiagnostic(forward: true)
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "naïve")
        XCTAssertTrue(model.navigationNote?.hasPrefix("Diagnostic 1 of 2") == true, model.navigationNote ?? "nil")

        // An edit inside the error's span: navigation to it is refused, selection unchanged.
        model.updateActiveText(text.replacingOccurrences(of: "{oops", with: "{o0ps"))
        model.caretUTF16 = ns.range(of: "naïve").location + 1
        let before = model.selection
        model.goToDiagnostic(forward: true)
        XCTAssertEqual(model.selection, before)
        XCTAssertTrue(model.navigationNote?.contains("recompile") == true, model.navigationNote ?? "nil")
        // The other diagnostic is still reachable.
        model.goToDiagnostic(forward: false)
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "naïve")
    }

    @MainActor
    func testDiagnosticNavigationWithoutResultOrSourcesExplains() {
        let model = ShellModel()
        model.replaceProject(entryText: "plain text\n")
        XCTAssertNil(model.result)
        model.goToDiagnostic(forward: true)
        XCTAssertEqual(model.navigationNote, "No compile result loaded; nothing to navigate to.")
        XCTAssertNil(model.selection)
        model.revealCaretInPreview()
        XCTAssertTrue(model.navigationNote?.contains("No compile result") == true)

        // A result whose diagnostics have no source in the active document.
        model.result = RuntimeV1.CompileResult(projectId: "p", revision: model.editorRevision, status: .recovered, pages: [], diagnostics: [
            .init(severity: .warning, message: "w", source: nil, recovery: nil),
            .init(severity: .error, message: "e", source: .init(path: "other.tex", startByte: 0, endByte: 1), recovery: nil),
        ], pdfPath: nil)
        model.goToDiagnostic(forward: false)
        XCTAssertEqual(model.navigationNote, "None of the 2 diagnostics has a source in main.tex.")
        model.result = RuntimeV1.CompileResult(projectId: "p", revision: model.editorRevision, status: .ok, pages: [], diagnostics: [], pdfPath: nil)
        model.goToDiagnostic(forward: true)
        XCTAssertTrue(model.navigationNote?.contains("no diagnostics") == true, model.navigationNote ?? "nil")
        XCTAssertNil(model.selection)
    }

    // MARK: caret → preview

    @MainActor
    func testRevealCaretInPreviewSelectsItemAndReportsPage() {
        let model = ShellModel()
        model.loadFixtures(request: nil, result: CaretSyncTests.resultURL)
        let ns = model.activeText as NSString
        model.caretUTF16 = ns.range(of: "Résumé").location + 3
        model.revealCaretInPreview()
        XCTAssertEqual(ns.substring(with: model.selection!.nsRange), "Résumé")
        XCTAssertEqual(model.navigationNote, "Caret is in page 2 item 1 “Résumé”.")
        XCTAssertEqual(model.caretItems, [2: [1]])
        // Whitespace maps to nothing; the note says so without touching the selection.
        let before = model.selection
        model.caretUTF16 = ns.range(of: "Introduction}").location + "Introduction}".count
        model.revealCaretInPreview()
        XCTAssertEqual(model.selection, before)
        XCTAssertTrue(model.navigationNote?.contains("inside no preview item") == true, model.navigationNote ?? "nil")
    }

    // MARK: pure helpers

    func testStopsOrderAndWrap() {
        let result = RuntimeV1.CompileResult(projectId: "p", revision: 1, status: .recovered, pages: [], diagnostics: [
            .init(severity: .error, message: "late", source: .init(path: "a.tex", startByte: 20, endByte: 25), recovery: nil),
            .init(severity: .warning, message: "none", source: nil, recovery: nil),
            .init(severity: .error, message: "early", source: .init(path: "a.tex", startByte: 2, endByte: 5), recovery: nil),
            .init(severity: .error, message: "other", source: .init(path: "b.tex", startByte: 0, endByte: 1), recovery: nil),
        ], pdfPath: nil)
        let stops = Navigation.stops(in: result, path: "a.tex", compiledText: nil, currentText: "")
        XCTAssertEqual(stops.map(\.diagnostic.message), ["early", "late"])
        XCTAssertEqual(Navigation.nextStop(stops, from: 0, forward: true)?.diagnostic.message, "early")
        XCTAssertEqual(Navigation.nextStop(stops, from: 2, forward: true)?.diagnostic.message, "late")
        XCTAssertEqual(Navigation.nextStop(stops, from: 20, forward: true)?.diagnostic.message, "early")
        XCTAssertEqual(Navigation.nextStop(stops, from: 20, forward: false)?.diagnostic.message, "early")
        XCTAssertEqual(Navigation.nextStop(stops, from: 2, forward: false)?.diagnostic.message, "late")
        XCTAssertNil(Navigation.nextStop([], from: 0, forward: true))
        // Rebased ordering: an insertion before "early" shifts it; the overlapped one keeps its offset.
        let old = String(repeating: "x", count: 30)
        var new = old; new.insert(contentsOf: "INS", at: new.index(new.startIndex, offsetBy: 1))
        let shifted = Navigation.stops(in: result, path: "a.tex", compiledText: old, currentText: new)
        XCTAssertEqual(shifted.map(\.currentStart), [5, 23])
        var overlapped = old
        overlapped.replaceSubrange(overlapped.index(overlapped.startIndex, offsetBy: 3)..<overlapped.index(overlapped.startIndex, offsetBy: 4), with: "Y")
        let o = Navigation.stops(in: result, path: "a.tex", compiledText: old, currentText: overlapped)
        XCTAssertEqual(o.map(\.currentStart), [2, 20])
    }
}
