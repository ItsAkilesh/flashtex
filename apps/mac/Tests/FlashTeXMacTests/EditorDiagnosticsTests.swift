import AppKit
import XCTest
@testable import FlashTeXProtocol
@testable import FlashTeXMac

/// Inline diagnostic marks: byte ranges → UTF-16 ranges, rebased across edits
/// or dropped, never drawn on the wrong text.
final class EditorDiagnosticsTests: XCTestCase {
    // "Hello wörld end": "ö" is 2 bytes, so "wörld" is bytes 6..<12 and UTF-16 6..<11.
    static let text = "Hello wörld end"

    private func result(_ diagnostics: [RuntimeV1.Diagnostic]) -> RuntimeV1.CompileResult {
        .init(projectId: "demo", revision: 1, status: .recovered, pages: [], diagnostics: diagnostics, pdfPath: nil)
    }

    private func diagnostic(_ severity: RuntimeV1.Severity, path: String = "main.tex",
                            start: Int = 6, end: Int = 12, source: Bool = true) -> RuntimeV1.Diagnostic {
        .init(severity: severity, message: "\(severity.rawValue) here",
              source: source ? .init(path: path, startByte: start, endByte: end) : nil,
              recovery: severity == .error ? "rendered without bold" : nil)
    }

    /// (a) A sourced error becomes one mark with the UTF-16 range of "wörld";
    /// a warning with null source is not a mark.
    func testSourcedErrorMarksAndUnsourcedWarningDoesNot() throws {
        let marks = EditorDiagnostics.marks(
            for: result([diagnostic(.error), diagnostic(.warning, source: false)]),
            path: "main.tex", compiledText: Self.text, currentText: Self.text)
        XCTAssertEqual(marks.count, 1)
        let mark = try XCTUnwrap(marks.first)
        XCTAssertEqual(mark.nsRange, NSRange(location: 6, length: 5))
        XCTAssertEqual((Self.text as NSString).substring(with: mark.nsRange), "wörld")
        XCTAssertEqual(mark.severity, .error)
        XCTAssertEqual(mark.message, "error here")
        XCTAssertEqual(mark.recovery, "rendered without bold")
        XCTAssertEqual(mark.toolTip, "error here\n↳ rendered without bold")
    }

    /// (b) An edit before the range shifts the mark; an edit inside drops it.
    func testMarksRebaseAcrossPrefixEditAndDropWhenEditOverlaps() throws {
        let res = result([diagnostic(.warning)])
        let shifted = EditorDiagnostics.marks(for: res, path: "main.tex",
                                              compiledText: Self.text, currentText: "XY " + Self.text)
        XCTAssertEqual(shifted.count, 1)
        let mark = try XCTUnwrap(shifted.first)
        XCTAssertEqual(mark.nsRange, NSRange(location: 9, length: 5))
        XCTAssertEqual(("XY " + Self.text as NSString).substring(with: mark.nsRange), "wörld")
        XCTAssertEqual(mark.severity, .warning)
        XCTAssertNil(mark.recovery)
        XCTAssertEqual(mark.toolTip, "warning here")

        let inside = EditorDiagnostics.marks(for: res, path: "main.tex",
                                             compiledText: Self.text, currentText: "Hello wöXrld end")
        XCTAssertEqual(inside, [], "a range overlapping the edit must be dropped, not guessed")

        // Without compiled text there is nothing to rebase against; the raw
        // offsets are applied only if they are a valid range in the buffer.
        let unknown = EditorDiagnostics.marks(for: res, path: "main.tex", compiledText: nil, currentText: "Hi")
        XCTAssertEqual(unknown, [])
        let invalid = EditorDiagnostics.marks(for: result([diagnostic(.error, start: 7, end: 8)]),
                                              path: "main.tex", compiledText: Self.text, currentText: Self.text)
        XCTAssertEqual(invalid, [], "a range starting inside a multi-byte scalar is refused")
    }

    /// (c) Diagnostics for another document do not mark this one.
    func testOtherPathProducesNoMark() {
        let marks = EditorDiagnostics.marks(for: result([diagnostic(.error, path: "chapter.tex")]),
                                            path: "main.tex", compiledText: Self.text, currentText: Self.text)
        XCTAssertEqual(marks, [])
    }

    /// (d) The multipage sample's error mark slices to `\textbf{oops`.
    func testMultipageSampleErrorMarkSlicesToItsText() throws {
        let req = try RuntimeV1.decodeCompileRequest(Data(contentsOf: CaretSyncTests.requestURL))
        let res = try RuntimeV1.decodeCompileResult(Data(contentsOf: CaretSyncTests.resultURL))
        let doc = try XCTUnwrap(req.payload.documents.first { $0.path == req.payload.entryPath })
        let marks = EditorDiagnostics.marks(for: res.payload, path: doc.path,
                                            compiledText: doc.text, currentText: doc.text)
        XCTAssertEqual(marks.count, 1, "one sourced error; the warning has null source")
        let mark = try XCTUnwrap(marks.first)
        XCTAssertEqual(mark.severity, .error)
        XCTAssertEqual((doc.text as NSString).substring(with: mark.nsRange), "\\textbf{oops")
        XCTAssertNotNil(mark.recovery)
    }

    /// Marks are temporary layout attributes: the text storage stays plain and
    /// out-of-range marks are ignored.
    @MainActor
    func testApplyMarksUsesTemporaryAttributesOnly() throws {
        let tv = NSTextView(frame: NSRect(x: 0, y: 0, width: 200, height: 100))
        tv.string = Self.text
        let marks = EditorDiagnostics.marks(for: result([diagnostic(.error)]), path: "main.tex",
                                            compiledText: Self.text, currentText: Self.text)
        let bogus = EditorDiagnostics.Mark(nsRange: NSRange(location: 100, length: 5), severity: .warning,
                                           message: "stale", recovery: nil)
        SourceEditorView.applyMarks(marks + [bogus], to: tv)
        let lm = try XCTUnwrap(tv.layoutManager)
        var effective = NSRange()
        let attrs = lm.temporaryAttributes(atCharacterIndex: 6, effectiveRange: &effective)
        XCTAssertEqual(effective, NSRange(location: 6, length: 5))
        XCTAssertEqual(attrs[.underlineColor] as? NSColor, .systemRed)
        XCTAssertEqual(attrs[.toolTip] as? String, "error here\n↳ rendered without bold")
        XCTAssertNil(lm.temporaryAttribute(.underlineStyle, atCharacterIndex: 0, effectiveRange: nil))
        XCTAssertEqual(tv.string, Self.text)
        let storage = try XCTUnwrap(tv.textStorage)
        XCTAssertNil(storage.attribute(.underlineStyle, at: 6, effectiveRange: nil))

        SourceEditorView.applyMarks([], to: tv)
        XCTAssertNil(lm.temporaryAttribute(.underlineStyle, atCharacterIndex: 6, effectiveRange: nil))
    }
}
