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
        XCTAssertEqual(model.navigationNote, "None of the 2 diagnostics has a source in an open document (main.tex).")
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

// MARK: - multi-file, multi-byte fixture

/// A two-document project (entry + `\input{chapter}`) with a ligature glyph,
/// a combining sequence, a ZWJ emoji, and CRLF line ends. The item spans are
/// the ones `flashtex-compiler` (crates/compiler, release build of 2026-09-12)
/// emitted for exactly these texts; `NavigationRealCompilerTests` re-derives
/// them from the binary when `FLASHTEX_COMPILER` is set, so a compiler change
/// that moves a span fails there rather than silently here. Item coordinates
/// are placeholders: navigation only reads `text` and `source`.
enum MultiFileFixture {
    static let main = "\\documentclass{article}\n\\begin{document}\n\\section{Intro}\nA na\u{EF}ve \u{FB01}le and office e\u{301} caf\u{E9} \u{1F600}\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467} end.\r\nCRLF line two.\r\n\\label{sec:a}\n\\input{chapter}\nSee \\ref{sec:b}.\n\\end{document}\n"
    static let chapter = "\\section{Chapter}\\label{sec:b}\nR\u{E9}sum\u{E9} of \u{FB03}cient \\textbf{oops\n"

    /// (path, start, end, generated text or nil when the text is the source slice), in page order.
    static let items: [(path: String, start: Int, end: Int, generated: String?)] = [
        ("main.tex", 41, 49, "1"), ("main.tex", 50, 55, nil), ("main.tex", 57, 58, nil), ("main.tex", 59, 65, nil),
        ("main.tex", 66, 71, nil), ("main.tex", 72, 75, nil), ("main.tex", 76, 82, nil), ("main.tex", 83, 86, nil),
        ("main.tex", 87, 92, nil), ("main.tex", 93, 115, nil), ("main.tex", 116, 120, nil), ("main.tex", 122, 126, nil),
        ("main.tex", 127, 131, nil), ("main.tex", 132, 136, nil),
        ("chapter.tex", 0, 8, "2"), ("chapter.tex", 9, 16, nil), ("chapter.tex", 31, 39, nil), ("chapter.tex", 40, 42, nil),
        ("chapter.tex", 43, 51, nil), ("chapter.tex", 60, 64, nil),
        ("main.tex", 168, 171, nil), ("main.tex", 172, 183, "2"), ("main.tex", 183, 184, nil),
    ]

    static let documents = [RuntimeV1.Document(path: "main.tex", text: main), .init(path: "chapter.tex", text: chapter)]

    static func text(of path: String) -> String { path == "main.tex" ? main : chapter }

    static func slice(_ path: String, _ start: Int, _ end: Int) -> String {
        let t = text(of: path)
        return String(t[t.rangeOfUTF8(start: start, end: end)!])
    }

    static var request: RuntimeV1.Envelope<RuntimeV1.CompileRequest> {
        .init(protocolVersion: 1, id: "mf-1", type: "compile",
              payload: .init(projectId: "demo", revision: 1, entryPath: "main.tex", documents: documents))
    }

    /// The compiler's `recovered` result: one page, the error on `\textbf{oops`
    /// in chapter.tex and the PDF-export warning on the ligature glyph.
    static var result: RuntimeV1.Envelope<RuntimeV1.CompileResult> {
        var y = 20.0
        let page = RuntimeV1.Page(number: 1, widthPt: 612, heightPt: 792, items: items.map { it in
            y += 14
            return .text(.init(text: it.generated ?? slice(it.path, it.start, it.end), xPt: 72, baselineYPt: y, fontSizePt: 10,
                               source: .init(path: it.path, startByte: it.start, endByte: it.end)))
        })
        return .init(protocolVersion: 1, id: "mf-1", type: "compile_result",
                     payload: .init(projectId: "demo", revision: 1, status: .recovered, pages: [page], diagnostics: [
                        .init(severity: .error, message: "argument to \\textbf is missing its closing brace",
                              source: .init(path: "chapter.tex", startByte: 59, endByte: 60), recovery: "closed the argument at end of input"),
                        .init(severity: .warning, message: "'\u{FB01}' (U+FB01) will not survive PDF export: no glyph for this character exists in the base-14 PDF fonts",
                              source: .init(path: "main.tex", startByte: 66, endByte: 71), recovery: "the preview shows it correctly; the exported PDF will not"),
                     ], pdfPath: nil))
    }

    /// Writes `mf-request.json`/`mf-result.json` to a fresh directory and returns the result URL.
    static func write() throws -> URL {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("flashtex-mf-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        try JSONEncoder().encode(request).write(to: dir.appendingPathComponent("mf-request.json"))
        try JSONEncoder().encode(result).write(to: dir.appendingPathComponent("mf-result.json"))
        return dir.appendingPathComponent("mf-result.json")
    }

    @MainActor
    static func loadedModel() throws -> ShellModel {
        let model = ShellModel()
        model.loadFixtures(request: nil, result: try write())
        XCTAssertNil(model.loadError, model.loadError ?? "")
        XCTAssertEqual(model.documents.map(\.path), ["main.tex", "chapter.tex"])
        XCTAssertEqual(model.activePath, "main.tex")
        XCTAssertFalse(model.previewIsStale)
        return model
    }
}

final class NavigationExactnessTests: XCTestCase {
    private func selected(_ mapping: Navigation.RangeMapping) -> NSRange? {
        if case .selected(let r, _) = mapping { return r } else { return nil }
    }

    /// The fixture's spans slice to whole composed character sequences, so
    /// UTF-16 selection is exact (never widened) for every item.
    func testFixtureSpansAreClusterExact() {
        let main = MultiFileFixture.main as NSString
        XCTAssertEqual(MultiFileFixture.slice("main.tex", 59, 65), "naïve")
        XCTAssertEqual(MultiFileFixture.slice("main.tex", 66, 71), "\u{FB01}le")
        XCTAssertEqual(MultiFileFixture.slice("main.tex", 83, 86), "e\u{301}")
        XCTAssertEqual(MultiFileFixture.slice("main.tex", 93, 115), "\u{1F600}\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}")
        XCTAssertEqual(MultiFileFixture.slice("main.tex", 122, 126), "CRLF")
        XCTAssertEqual(MultiFileFixture.slice("chapter.tex", 43, 51), "\u{FB03}cient")
        for it in MultiFileFixture.items {
            let text = MultiFileFixture.text(of: it.path)
            guard case .selected(let ns, let widened) = Navigation.editorRange(start: it.start, end: it.end, in: text, path: it.path) else {
                return XCTFail("\(it) refused")
            }
            XCTAssertNil(widened, "\(it) split a cluster")
            XCTAssertEqual((text as NSString).substring(with: ns), MultiFileFixture.slice(it.path, it.start, it.end))
        }
        // UTF-16 and UTF-8 offsets diverge by the emoji's surrogate pairs and the multi-byte scalars.
        XCTAssertEqual(main.range(of: "end.").location, 99)
        XCTAssertEqual(selected(Navigation.editorRange(start: 116, end: 120, in: MultiFileFixture.main)), NSRange(location: 99, length: 4))
    }

    func testEditorRangeRefusesSplitScalarsAndWidensSplitClusters() {
        let text = MultiFileFixture.main
        let ns = text as NSString
        // Inside the 2-byte "ï" (bytes 61..<63): refused, not rounded.
        guard case .refused(let why) = Navigation.editorRange(start: 62, end: 65, in: text, path: "main.tex") else { return XCTFail() }
        XCTAssertTrue(why.contains("inside a multi-byte character"), why)
        guard case .refused(let why2) = Navigation.editorRange(start: 59, end: 62, in: text) else { return XCTFail() }
        XCTAssertTrue(why2.contains("multi-byte"), why2)
        // Out of range / reversed: refused with the buffer size.
        guard case .refused(let why3) = Navigation.editorRange(start: 10, end: 9, in: text, path: "main.tex") else { return XCTFail() }
        XCTAssertTrue(why3.contains("not a valid range in main.tex"), why3)
        guard case .refused = Navigation.editorRange(start: 0, end: text.utf8.count + 1, in: text) else { return XCTFail() }

        // Scalar-aligned but cluster-splitting: "e" alone from "e" + U+0301 is widened to both.
        let e = ns.range(of: "e\u{301}")
        guard case .selected(let r, let from) = Navigation.editorRange(start: 83, end: 84, in: text) else { return XCTFail() }
        XCTAssertEqual(r, e)
        XCTAssertEqual(from, NSRange(location: e.location, length: 1))
        XCTAssertEqual(ns.substring(with: r), "e\u{301}")
        // The combining mark alone (bytes 84..<86) widens backwards to the base.
        XCTAssertEqual(selected(Navigation.editorRange(start: 84, end: 86, in: text)), e)
        // One scalar of the ZWJ family (👨, bytes 97..<101) widens to the whole family, not the preceding 😀.
        let family = ns.range(of: "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}")
        XCTAssertEqual(selected(Navigation.editorRange(start: 97, end: 101, in: text)), family)
        XCTAssertEqual(selected(Navigation.editorRange(start: 93, end: 97, in: text)), ns.range(of: "\u{1F600}"))
        // An empty span between the base and its combining mark moves to the cluster start and stays empty.
        XCTAssertEqual(selected(Navigation.editorRange(start: 84, end: 84, in: text)), NSRange(location: e.location, length: 0))
        // Empty span at a boundary and at the very end are kept as-is.
        XCTAssertEqual(selected(Navigation.editorRange(start: 83, end: 83, in: text)), NSRange(location: e.location, length: 0))
        XCTAssertEqual(selected(Navigation.editorRange(start: text.utf8.count, end: text.utf8.count, in: text)), NSRange(location: ns.length, length: 0))
        // CRLF: the text view treats "\r" and "\n" as separate characters, so a span ending after "\r" is exact.
        let crlf = ns.range(of: "end.\r\n")
        XCTAssertEqual(selected(Navigation.editorRange(start: 116, end: 121, in: text)), NSRange(location: crlf.location, length: 5))
        XCTAssertEqual(selected(Navigation.editorRange(start: 116, end: 122, in: text)), crlf)
    }

    func testCommandUsesSkipCommentsAndEscapes() {
        let text = "\\begin{a} % \\end{a} not real\n\\%\\end{a}\\\\\\ref{x}\n% \\label{x}\n\\label{x}"
        let uses = Navigation.commandUses(in: text)
        XCTAssertEqual(uses.map(\.name), ["begin", "end", "ref", "label"])
        XCTAssertEqual(uses.map(\.range.start), [0, 31, 40, 60])
        guard case .found(let r, _) = Navigation.matchingRange(in: text, caretByte: 0) else { return XCTFail() }
        XCTAssertEqual(r.start, 31, "the \\end inside the comment must not match")
        guard case .found(let l, _) = Navigation.matchingRange(in: text, caretByte: 42) else { return XCTFail() }
        XCTAssertEqual(l.start, 60, "the \\label inside the comment must not be the definition")
        // A `%` inside a braced argument terminates it (no use), never traps.
        XCTAssertEqual(Navigation.commandUses(in: "\\ref{a%b}").count, 0)
        XCTAssertEqual(Navigation.commandUses(in: "\\").count, 0)
        XCTAssertEqual(Navigation.commandUses(in: "%").count, 0)
    }

    func testNestedEnvironmentsMatchExactlyByNameAndDepth() {
        let text = "\\begin{a}\\begin{b}\\begin{a}\\end{a}\\end{b}\\end{a} \\begin{b}\\end{a}"
        let uses = Navigation.commandUses(in: text)
        func start(_ i: Int) -> Int { uses[i].range.start }
        // Outer \begin{a} (0) matches the last \end{a} (5), skipping the nested pair (2, 3).
        guard case .found(let r0, _) = Navigation.matchingRange(in: text, caretByte: start(0)) else { return XCTFail() }
        XCTAssertEqual(r0.start, start(5))
        guard case .found(let r2, _) = Navigation.matchingRange(in: text, caretByte: start(2) + 1) else { return XCTFail() }
        XCTAssertEqual(r2.start, start(3))
        guard case .found(let r5, _) = Navigation.matchingRange(in: text, caretByte: start(5)) else { return XCTFail() }
        XCTAssertEqual(r5.start, start(0))
        guard case .found(let r1, _) = Navigation.matchingRange(in: text, caretByte: start(1)) else { return XCTFail() }
        XCTAssertEqual(r1.start, start(4))
        // The dangling \begin{b} after the balanced block has no partner; the following \end{a} is unbalanced.
        guard case .notFound(let why6) = Navigation.matchingRange(in: text, caretByte: start(6)) else { return XCTFail() }
        XCTAssertTrue(why6.contains("no matching \\end{b}"), why6)
        guard case .notFound(let why7) = Navigation.matchingRange(in: text, caretByte: start(7)) else { return XCTFail() }
        XCTAssertTrue(why7.contains("no matching \\begin{a}"), why7)
    }

    func testLabelAndReferenceResolveAcrossDocuments() {
        let docs = MultiFileFixture.documents
        let refByte = MultiFileFixture.main.utf8.distance(from: MultiFileFixture.main.startIndex, to: MultiFileFixture.main.range(of: "\\ref{sec:b}")!.lowerBound)
        guard case .found(let path, let r, let note) = Navigation.matchingRange(in: docs, activePath: "main.tex", caretByte: refByte + 2) else { return XCTFail() }
        XCTAssertEqual(path, "chapter.tex")
        XCTAssertEqual(MultiFileFixture.slice(path, r.start, r.end), "\\label{sec:b}")
        XCTAssertTrue(note.hasSuffix(" in chapter.tex."), note)
        // From the label in chapter.tex: the only reference is in main.tex (wrapping through project order).
        let labelByte = MultiFileFixture.chapter.utf8.distance(from: MultiFileFixture.chapter.startIndex, to: MultiFileFixture.chapter.range(of: "\\label")!.lowerBound)
        guard case .found(let p2, let r2, let n2) = Navigation.matchingRange(in: docs, activePath: "chapter.tex", caretByte: labelByte) else { return XCTFail() }
        XCTAssertEqual(p2, "main.tex")
        XCTAssertEqual(MultiFileFixture.slice(p2, r2.start, r2.end), "\\ref{sec:b}")
        XCTAssertEqual(n2, "Reference 1 of 1 to label sec:b at byte \(r2.start) in main.tex.")
        // sec:a is labelled in main.tex but referenced nowhere.
        let labelA = MultiFileFixture.main.utf8.distance(from: MultiFileFixture.main.startIndex, to: MultiFileFixture.main.range(of: "\\label{sec:a}")!.lowerBound)
        guard case .notFound(let why) = Navigation.matchingRange(in: docs, activePath: "main.tex", caretByte: labelA + 1) else { return XCTFail() }
        XCTAssertEqual(why, "No reference to label sec:a in this document or any open document.")
        // Same-document match is preferred over a duplicate label elsewhere.
        let dup = [RuntimeV1.Document(path: "a.tex", text: "\\label{k}"), .init(path: "b.tex", text: "\\ref{k} \\label{k}")]
        guard case .found(let p3, let r3, _) = Navigation.matchingRange(in: dup, activePath: "b.tex", caretByte: 0) else { return XCTFail() }
        XCTAssertEqual(p3, "b.tex"); XCTAssertEqual(r3.start, 8)
        guard case .notFound = Navigation.matchingRange(in: dup, activePath: "missing.tex", caretByte: 0) else { return XCTFail() }
    }

    @MainActor
    func testGoToMatchingSwitchesDocumentForCrossFileReference() throws {
        let model = try MultiFileFixture.loadedModel()
        let main = MultiFileFixture.main as NSString
        model.caretUTF16 = main.range(of: "\\ref{sec:b}").location + 3
        model.goToMatching()
        XCTAssertEqual(model.activePath, "chapter.tex")
        let sel = try XCTUnwrap(model.selection)
        XCTAssertEqual(sel.path, "chapter.tex")
        XCTAssertEqual((model.activeText as NSString).substring(with: sel.nsRange), "\\label{sec:b}")
        XCTAssertEqual(model.caretUTF16, sel.nsRange.location)
        XCTAssertEqual(model.caretLengthUTF16, sel.nsRange.length)
        // And back: the label's only reference is in main.tex.
        model.goToMatching()
        XCTAssertEqual(model.activePath, "main.tex")
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "\\ref{sec:b}")
    }

    // MARK: preview → source

    @MainActor
    func testEveryItemNavigatesToExactlyItsBytesAndSwitchesDocuments() throws {
        let model = try MultiFileFixture.loadedModel()
        let result = try XCTUnwrap(model.result)
        var switches = 0
        var generated = 0
        for case .text(let item) in result.pages[0].items {
            let source = try XCTUnwrap(item.source)
            let wasActive = model.activePath
            model.navigateExactly(to: source, expectedText: item.text)
            let sel = try XCTUnwrap(model.selection, model.navigationNote ?? "nil")
            XCTAssertEqual(model.activePath, source.path)
            XCTAssertEqual(sel.path, source.path)
            let selectedText = (model.activeText as NSString).substring(with: sel.nsRange)
            XCTAssertEqual(selectedText, MultiFileFixture.slice(source.path, source.startByte, source.endByte))
            XCTAssertEqual(model.activeText.utf8ByteRange(of: sel.nsRange)?.start, source.startByte)
            XCTAssertEqual(model.activeText.utf8ByteRange(of: sel.nsRange)?.end, source.endByte)
            XCTAssertEqual(model.caretUTF16, sel.nsRange.location)
            XCTAssertEqual(model.caretLengthUTF16, sel.nsRange.length)
            let note = try XCTUnwrap(model.navigationNote)
            XCTAssertTrue(note.hasPrefix("Selected \(source.path) bytes \(source.startByte)..<\(source.endByte) → UTF-16 \(sel.nsRange.location)..<\(NSMaxRange(sel.nsRange))"), note)
            XCTAssertFalse(note.contains("widened"), note)
            if wasActive != source.path { switches += 1; XCTAssertTrue(note.contains("switched to \(source.path)"), note) }
            if item.text != selectedText { generated += 1; XCTAssertTrue(note.contains("“\(item.text)” is generated from this source"), note) }
        }
        XCTAssertEqual(switches, 2, "main → chapter → main")
        XCTAssertEqual(generated, 3, "section numbers and the \\ref value")
        // Ligature glyph vs ASCII: the ﬁ item is the 3-byte scalar, never "f" + "i".
        model.navigateExactly(to: .init(path: "main.tex", startByte: 66, endByte: 71), expectedText: "\u{FB01}le")
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "\u{FB01}le")
        XCTAssertEqual(model.selection?.nsRange.length, 3)
        // A byte span into the middle of the ligature scalar is refused, selection unchanged.
        let before = model.selection
        model.navigateExactly(to: .init(path: "main.tex", startByte: 67, endByte: 71), expectedText: nil)
        XCTAssertEqual(model.selection, before)
        XCTAssertTrue(model.navigationNote?.contains("inside a multi-byte character") == true, model.navigationNote ?? "nil")
        // Unknown document: refused and the open ones are listed.
        model.navigateExactly(to: .init(path: "missing.tex", startByte: 0, endByte: 1), expectedText: nil)
        XCTAssertEqual(model.navigationNote, "No open document named missing.tex (open: main.tex, chapter.tex).")
        model.navigateExactly(to: nil)
        XCTAssertEqual(model.navigationNote, "This item has no source mapping.")
        XCTAssertEqual(model.selection, before)
    }

    @MainActor
    func testStaleSpansAreRefusedWithTheEditedBytesAndOthersRebase() throws {
        let model = try MultiFileFixture.loadedModel()
        let ligature = RuntimeV1.SourceRange(path: "main.tex", startByte: 66, endByte: 71)   // "ﬁle"
        let cafe = RuntimeV1.SourceRange(path: "main.tex", startByte: 87, endByte: 92)       // "café"
        let resume = RuntimeV1.SourceRange(path: "chapter.tex", startByte: 31, endByte: 39)  // "Résumé"

        // Replace the ligature glyph with ASCII "fi": the same visible word, different bytes.
        model.updateActiveText(MultiFileFixture.main.replacingOccurrences(of: "\u{FB01}le", with: "file"))
        XCTAssertTrue(model.previewIsStale)
        let before = model.selection
        model.navigateExactly(to: ligature, expectedText: "\u{FB01}le")
        XCTAssertEqual(model.selection, before)
        XCTAssertEqual(model.navigationNote, "Source for this item was edited since revision 1 (bytes 66..<71 of main.tex overlap the edit at 66..<69, now 66..<68); recompile to navigate.")
        // A later span in the same document is rebased by the −1 byte delta and verified.
        model.navigateExactly(to: cafe, expectedText: "caf\u{E9}")
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "caf\u{E9}")
        XCTAssertEqual(model.activeText.utf8ByteRange(of: model.selection!.nsRange)?.start, 86)
        XCTAssertTrue(model.navigationNote?.contains("rebased from 87..<92 across edits") == true, model.navigationNote ?? "nil")
        // The other document is untouched: its spans navigate unchanged, switching documents.
        model.navigateExactly(to: resume, expectedText: "R\u{E9}sum\u{E9}")
        XCTAssertEqual(model.activePath, "chapter.tex")
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "R\u{E9}sum\u{E9}")
        XCTAssertFalse(model.navigationNote?.contains("rebased") == true)

        // Normalization-only edit: precomposed é → e + U+0301 is canonically equal
        // as a String but changes the bytes, so the span is stale, not "unchanged".
        model.activePath = "main.tex"
        model.updateActiveText(MultiFileFixture.main.replacingOccurrences(of: "caf\u{E9}", with: "cafe\u{301}"))
        XCTAssertEqual(model.activeText, MultiFileFixture.main, "String equality is canonical; the bytes differ")
        XCTAssertFalse(model.activeText.sameBytes(as: MultiFileFixture.main))
        let before2 = model.selection
        model.navigateExactly(to: cafe, expectedText: "caf\u{E9}")
        XCTAssertEqual(model.selection, before2)
        XCTAssertTrue(model.navigationNote?.contains("overlap the edit at 90..<92, now 90..<93") == true, model.navigationNote ?? "nil")
        // ...while the ligature before it still navigates exactly.
        model.navigateExactly(to: ligature, expectedText: "\u{FB01}le")
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "\u{FB01}le")

        // CRLF → LF conversion touches the whole tail; spans before the first CRLF survive, later ones are refused.
        model.updateActiveText(MultiFileFixture.main.replacingOccurrences(of: "\r\n", with: "\n"))
        model.navigateExactly(to: cafe, expectedText: nil)
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "caf\u{E9}")
        let crlfWord = RuntimeV1.SourceRange(path: "main.tex", startByte: 122, endByte: 126) // "CRLF"
        let before3 = model.selection
        model.navigateExactly(to: crlfWord, expectedText: "CRLF")
        XCTAssertEqual(model.selection, before3)
        XCTAssertTrue(model.navigationNote?.contains("recompile to navigate") == true, model.navigationNote ?? "nil")
        // Emoji edited: the family item (93..<115) overlaps, the neighbouring "end." rebases.
        model.updateActiveText(MultiFileFixture.main.replacingOccurrences(of: "\u{1F469}", with: "\u{1F468}"))
        model.navigateExactly(to: .init(path: "main.tex", startByte: 93, endByte: 115), expectedText: nil)
        XCTAssertTrue(model.navigationNote?.contains("overlap the edit") == true, model.navigationNote ?? "nil")
        model.navigateExactly(to: .init(path: "main.tex", startByte: 116, endByte: 120), expectedText: "end.")
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "end.")
    }

    // MARK: diagnostics across documents

    @MainActor
    func testDiagnosticsCycleAcrossDocumentsInProjectOrder() throws {
        let model = try MultiFileFixture.loadedModel()
        let result = try XCTUnwrap(model.result)
        let stops = Navigation.stops(in: result, documents: model.documents, compiledDocuments: ["main.tex": MultiFileFixture.main, "chapter.tex": MultiFileFixture.chapter])
        XCTAssertEqual(stops.map(\.source.path), ["main.tex", "chapter.tex"], "project order, not result order")
        XCTAssertEqual(stops.map(\.index), [1, 0])

        model.caretUTF16 = 0
        model.goToDiagnostic(forward: true)
        XCTAssertEqual(model.activePath, "main.tex")
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "\u{FB01}le")
        XCTAssertTrue(model.navigationNote?.hasPrefix("Diagnostic 1 of 2 (warning) in main.tex: '\u{FB01}'") == true, model.navigationNote ?? "nil")
        model.goToDiagnostic(forward: true)
        XCTAssertEqual(model.activePath, "chapter.tex")
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "{")
        XCTAssertEqual(model.activeText.utf8ByteRange(of: model.selection!.nsRange)?.start, 59)
        XCTAssertTrue(model.navigationNote?.hasPrefix("Diagnostic 2 of 2 (error) in chapter.tex:") == true, model.navigationNote ?? "nil")
        model.goToDiagnostic(forward: true) // wraps to main.tex
        XCTAssertEqual(model.activePath, "main.tex")
        XCTAssertTrue(model.navigationNote?.hasPrefix("Diagnostic 1 of 2") == true, model.navigationNote ?? "nil")
        model.goToDiagnostic(forward: false) // wraps backwards into chapter.tex
        XCTAssertEqual(model.activePath, "chapter.tex")
        XCTAssertTrue(model.navigationNote?.hasPrefix("Diagnostic 2 of 2") == true, model.navigationNote ?? "nil")
        model.goToDiagnostic(forward: false)
        XCTAssertEqual(model.activePath, "main.tex")

        // Edit the byte the chapter error points at (its "{"): it is refused, the warning in main.tex still cycles.
        model.activePath = "chapter.tex"
        model.updateActiveText(MultiFileFixture.chapter.replacingOccurrences(of: "\\textbf{oops", with: "\\textbf[oops"))
        model.activePath = "main.tex"
        model.caretUTF16 = (MultiFileFixture.main as NSString).range(of: "office").location
        let before = model.selection
        model.goToDiagnostic(forward: true)
        XCTAssertEqual(model.selection, before)
        XCTAssertEqual(model.activePath, "main.tex", "a refused stop must not switch documents")
        XCTAssertTrue(model.navigationNote?.contains("recompile to navigate") == true, model.navigationNote ?? "nil")
        model.goToDiagnostic(forward: false)
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "\u{FB01}le")
    }

    func testNextStopAcrossDocumentsOrdersByDocumentThenByte() {
        let docs = [RuntimeV1.Document(path: "a.tex", text: ""), .init(path: "b.tex", text: "")]
        let result = RuntimeV1.CompileResult(projectId: "p", revision: 1, status: .recovered, pages: [], diagnostics: [
            .init(severity: .error, message: "b10", source: .init(path: "b.tex", startByte: 10, endByte: 11), recovery: nil),
            .init(severity: .error, message: "a20", source: .init(path: "a.tex", startByte: 20, endByte: 21), recovery: nil),
            .init(severity: .error, message: "a5", source: .init(path: "a.tex", startByte: 5, endByte: 6), recovery: nil),
            .init(severity: .error, message: "c0", source: .init(path: "c.tex", startByte: 0, endByte: 1), recovery: nil),
        ], pdfPath: nil)
        let stops = Navigation.stops(in: result, documents: docs, compiledDocuments: [:])
        XCTAssertEqual(stops.map(\.diagnostic.message), ["a5", "a20", "b10"], "c.tex is not open")
        func next(_ path: String, _ byte: Int, _ forward: Bool) -> String? {
            Navigation.nextStop(stops, documents: docs, activePath: path, caretByte: byte, forward: forward)?.diagnostic.message
        }
        XCTAssertEqual(next("a.tex", 5, true), "a20")
        XCTAssertEqual(next("a.tex", 20, true), "b10")
        XCTAssertEqual(next("b.tex", 10, true), "a5")
        XCTAssertEqual(next("b.tex", 0, true), "b10")
        XCTAssertEqual(next("b.tex", 0, false), "a20")
        XCTAssertEqual(next("a.tex", 0, false), "b10")
        XCTAssertEqual(next("a.tex", 6, false), "a5")
        XCTAssertNil(Navigation.nextStop([], documents: docs, activePath: "a.tex", caretByte: 0, forward: true))
    }

    // MARK: caret → preview

    @MainActor
    func testRevealCaretInPreviewInsideClustersAndOtherDocuments() throws {
        let model = try MultiFileFixture.loadedModel()
        let main = MultiFileFixture.main as NSString
        // Caret between 👨 and the ZWJ (a position the text view never offers, but a byte inside the item).
        model.caretUTF16 = main.range(of: "\u{1F468}").location + 2
        model.revealCaretInPreview()
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "\u{1F600}\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}")
        XCTAssertEqual(model.navigationNote, "Caret is in page 1 item 9 “\u{1F600}\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}”.")
        // Caret after the ligature glyph's "l": the item is the whole "ﬁle".
        model.caretUTF16 = main.range(of: "\u{FB01}le").location + 2
        model.revealCaretInPreview()
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "\u{FB01}le")
        XCTAssertEqual(model.exactCaretItems, [1: [4]])
        // In chapter.tex the same UTF-16 offsets mean different bytes; the reveal is per document.
        model.activePath = "chapter.tex"
        model.caretUTF16 = (MultiFileFixture.chapter as NSString).range(of: "\u{FB03}cient").location + 1
        model.revealCaretInPreview()
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "\u{FB03}cient")
        XCTAssertEqual(model.navigationNote, "Caret is in page 1 item 18 “\u{FB03}cient”.")
        XCTAssertEqual(model.exactCaretItems, [1: [18]])
        // The gap after a word (byte == end_byte) is inside nothing.
        model.caretUTF16 = (MultiFileFixture.chapter as NSString).range(of: "\u{FB03}cient").location + 6
        let before = model.selection
        model.revealCaretInPreview()
        XCTAssertEqual(model.selection, before)
        XCTAssertTrue(model.navigationNote?.contains("inside no preview item") == true, model.navigationNote ?? "nil")
    }
}

/// Re-derives the fixture's spans from the real worker so the hand-copied
/// table above cannot drift from the compiler silently.
@MainActor
final class NavigationRealCompilerTests: XCTestCase {
    func testRealCompilerSpansMatchFixtureAndNavigateExactly() async throws {
        guard let binary = RealCompilerTests.binary, FileManager.default.isExecutableFile(atPath: binary.path) else {
            throw XCTSkip("set FLASHTEX_COMPILER to the built flashtex-compiler binary")
        }
        let model = ShellModel()
        model.attachWorker(at: binary)
        model.documents = MultiFileFixture.documents
        model.activePath = "main.tex"
        model.autoCompile = false
        model.compile()
        let start = Date()
        while model.inFlightRevision != nil {
            if Date().timeIntervalSince(start) > 10 { throw XCTSkip("compiler did not answer in 10 s") }
            try await Task.sleep(nanoseconds: 50_000_000)
        }
        let result = try XCTUnwrap(model.result)
        XCTAssertEqual(result.status, .recovered)
        let items: [RuntimeV1.PageItem.TextItem] = result.pages.flatMap(\.items).compactMap { if case .text(let t) = $0 { t } else { nil } }
        let spans = items.map { ($0.source!.path, $0.source!.startByte, $0.source!.endByte, $0.text) }
        let expected = MultiFileFixture.items.map { ($0.path, $0.start, $0.end, $0.generated ?? MultiFileFixture.slice($0.path, $0.start, $0.end)) }
        XCTAssertEqual(spans.count, expected.count)
        for (got, want) in zip(spans, expected) {
            XCTAssertTrue(got == want, "compiler emitted \(got), fixture table has \(want)")
        }
        let error = try XCTUnwrap(result.diagnostics.first { $0.severity == .error })
        XCTAssertEqual(error.source, .init(path: "chapter.tex", startByte: 59, endByte: 60))
        for item in items {
            model.navigateExactly(to: item.source, expectedText: item.text)
            let sel = try XCTUnwrap(model.selection, model.navigationNote ?? "nil")
            XCTAssertEqual(model.activePath, item.source!.path)
            XCTAssertEqual(model.activeText.utf8ByteRange(of: sel.nsRange)?.start, item.source!.startByte)
            XCTAssertEqual(model.activeText.utf8ByteRange(of: sel.nsRange)?.end, item.source!.endByte)
            XCTAssertFalse(model.navigationNote?.contains("widened") == true, model.navigationNote ?? "nil")
        }
        model.detachWorker()
    }
}
