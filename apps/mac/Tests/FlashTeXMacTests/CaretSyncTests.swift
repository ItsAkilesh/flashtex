import XCTest
@testable import FlashTeXProtocol
@testable import FlashTeXMac

/// Source → preview sync against `Samples/multipage-{request,result}.json`.
final class CaretSyncTests: XCTestCase {
    static let samples = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        .appendingPathComponent("Samples")
    static let requestURL = samples.appendingPathComponent("multipage-request.json")
    static let resultURL = samples.appendingPathComponent("multipage-result.json")

    private func load() throws -> (text: String, result: RuntimeV1.CompileResult) {
        let req = try RuntimeV1.decodeCompileRequest(Data(contentsOf: Self.requestURL))
        let res = try RuntimeV1.decodeCompileResult(Data(contentsOf: Self.resultURL))
        XCTAssertEqual(req.id, res.id)
        XCTAssertEqual(res.id, "sample-multipage-1")
        XCTAssertEqual(req.payload.revision, res.payload.revision)
        let doc = try XCTUnwrap(req.payload.documents.first { $0.path == req.payload.entryPath })
        return (doc.text, res.payload)
    }

    /// (a) Every item's source range slices the request text to exactly its `text`.
    func testEveryItemSourceSlicesToItsText() throws {
        let (text, result) = try load()
        XCTAssertEqual(result.pages.count, 2)
        XCTAssertEqual(result.diagnostics.count, 2)
        XCTAssertNotEqual(text.utf8.count, text.count, "sample must contain non-ASCII text")
        var checked = 0
        for page in result.pages {
            for item in page.items {
                guard case .text(let t) = item else { XCTFail("unexpected item kind"); continue }
                let source = try XCTUnwrap(t.source, "\(t.text) has no source")
                XCTAssertEqual(source.path, "main.tex")
                let range = try XCTUnwrap(text.range(utf8Bytes: source), "invalid range for \(t.text)")
                XCTAssertEqual(String(text[range]), t.text)
                checked += 1
            }
        }
        XCTAssertEqual(checked, 9)

        // Diagnostics: one error with a source range and recovery, one warning without.
        let error = try XCTUnwrap(result.diagnostics.first { $0.severity == .error })
        let errRange = try XCTUnwrap(text.range(utf8Bytes: try XCTUnwrap(error.source)))
        XCTAssertEqual(String(text[errRange]), "\\textbf{oops")
        XCTAssertNotNil(error.recovery)
        let warning = try XCTUnwrap(result.diagnostics.first { $0.severity == .warning })
        XCTAssertNil(warning.source)
        XCTAssertNil(warning.recovery)
    }

    /// (b) Byte inside the non-ASCII word hits it; whitespace between items hits nothing.
    func testItemsContainingFindsNonASCIIWordAndIgnoresGaps() throws {
        let (text, result) = try load()
        let naive = try XCTUnwrap(text.utf8ByteRange(of: (text as NSString).range(of: "naïve")))
        XCTAssertEqual(naive.start, 66)
        XCTAssertEqual(naive.end, 72) // "ï" is 2 bytes
        // Second byte of "ï" (not a scalar boundary) is still inside the item.
        let hits = CaretSync.itemsContaining(byte: naive.start + 3, path: "main.tex", in: result)
        XCTAssertEqual(hits.count, 1)
        XCTAssertEqual(hits.first?.page, 1)
        XCTAssertEqual(hits.first?.index, 2)
        guard case .text(let t) = result.pages[0].items[2] else { return XCTFail() }
        XCTAssertEqual(t.text, "naïve")

        // Boundaries: start is inclusive, end is exclusive.
        XCTAssertEqual(CaretSync.itemsContaining(byte: naive.start, path: "main.tex", in: result).first?.index, 2)
        XCTAssertTrue(CaretSync.itemsContaining(byte: naive.end, path: "main.tex", in: result).isEmpty) // the space
        XCTAssertEqual(CaretSync.itemsContaining(byte: naive.end + 1, path: "main.tex", in: result).first?.index, 3) // "approach"
        // Whitespace between "Introduction}" and "A" maps to nothing; wrong path maps to nothing.
        XCTAssertTrue(CaretSync.itemsContaining(byte: 63, path: "main.tex", in: result).isEmpty)
        XCTAssertTrue(CaretSync.itemsContaining(byte: naive.start, path: "other.tex", in: result).isEmpty)

        // Page 2: "Résumé".
        let resume = try XCTUnwrap(text.utf8ByteRange(of: (text as NSString).range(of: "Résumé")))
        let p2 = CaretSync.itemsContaining(byte: resume.start + 1, path: "main.tex", in: result)
        XCTAssertEqual(p2.first?.page, 2)
        XCTAssertEqual(p2.first?.index, 1)
        XCTAssertEqual(CaretSync.indicesByPage(byte: resume.start, path: "main.tex", in: result), [2: [1]])
    }

    /// Empty source ranges match only an exactly equal byte.
    func testEmptyRangeMatchesOnlyExactByte() {
        let item = RuntimeV1.PageItem.text(.init(text: "", xPt: 0, baselineYPt: 0, fontSizePt: 10,
                                                  source: .init(path: "main.tex", startByte: 5, endByte: 5)))
        let result = RuntimeV1.CompileResult(
            projectId: "demo", revision: 1, status: .ok,
            pages: [.init(number: 1, widthPt: 10, heightPt: 10, items: [item, .unknown(kind: "image")])],
            diagnostics: [], pdfPath: nil)
        XCTAssertEqual(CaretSync.itemsContaining(byte: 5, path: "main.tex", in: result).map(\.index), [0])
        XCTAssertTrue(CaretSync.itemsContaining(byte: 4, path: "main.tex", in: result).isEmpty)
        XCTAssertTrue(CaretSync.itemsContaining(byte: 6, path: "main.tex", in: result).isEmpty)
    }
}

/// (c) ShellModel: the UTF-16 caret maps to a UTF-8 byte and then to preview items.
@MainActor
final class ShellModelCaretSyncTests: XCTestCase {
    func testCaretMapsToByteAndPreviewItems() throws {
        let model = ShellModel()
        // Pass no request: the `<name>-request.json` sibling must be discovered.
        model.loadFixtures(request: nil, result: CaretSyncTests.resultURL)
        XCTAssertNil(model.loadError, model.loadError ?? "")
        XCTAssertEqual(model.resultID, "sample-multipage-1")
        XCTAssertEqual(model.activePath, "main.tex")
        XCTAssertTrue(model.activeText.contains("naïve"))
        XCTAssertFalse(model.previewIsStale)

        // Caret on the "v" of "naïve": UTF-16 offset 69, UTF-8 byte 70 (ï is 2 bytes).
        let ns = (model.activeText as NSString).range(of: "naïve")
        XCTAssertEqual(ns.location, 66)
        model.caretUTF16 = ns.location + 3
        XCTAssertEqual(model.caretByte, 70)
        XCTAssertEqual(model.caretItems, [1: [2]])
        XCTAssertEqual(CaretSync.itemsContaining(byte: 70, path: "main.tex", in: model.result!).first?.index, 2)

        // After the non-ASCII word, UTF-16 and UTF-8 offsets diverge by one.
        let resume = (model.activeText as NSString).range(of: "Résumé")
        XCTAssertEqual(resume.location, 114)
        model.caretUTF16 = resume.location
        XCTAssertEqual(model.caretByte, 115)
        XCTAssertEqual(model.caretItems, [2: [1]])

        // Gap and out-of-range carets highlight nothing.
        model.caretUTF16 = 63
        XCTAssertEqual(model.caretItems, [:])
        model.caretUTF16 = (model.activeText as NSString).length + 10
        XCTAssertNil(model.caretByte)
        XCTAssertEqual(model.caretItems, [:])

        // A different active document never matches items sourced from main.tex.
        model.caretUTF16 = 70
        model.activePath = "other.tex"
        XCTAssertEqual(model.caretItems, [:])
    }

    func testSiblingRequestURLDerivation() {
        let dir = URL(fileURLWithPath: "/tmp/x")
        XCTAssertEqual(ShellModel.siblingRequestURL(forResult: dir.appendingPathComponent("multipage-result.json")),
                       dir.appendingPathComponent("multipage-request.json"))
        XCTAssertEqual(ShellModel.siblingRequestURL(forResult: dir.appendingPathComponent("compile-result.json")),
                       dir.appendingPathComponent("compile-request.json"))
        XCTAssertNil(ShellModel.siblingRequestURL(forResult: dir.appendingPathComponent("something.json")))
    }
}
