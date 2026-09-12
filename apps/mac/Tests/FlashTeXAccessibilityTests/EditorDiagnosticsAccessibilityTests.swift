import XCTest
import FlashTeXProtocol
@testable import FlashTeXAccessibility

/// Keyboard error navigation as VoiceOver hears it: document order, wrapping
/// at both ends, "n of m" announcements with line and recovery line.
final class EditorDiagnosticsAccessibilityTests: XCTestCase {
    typealias Nav = EditorDiagnosticNavigation

    // Given out of document order on purpose.
    let items: [Nav.Item] = [
        .init(id: "r#2@main.tex:40..<45", nsRange: NSRange(location: 40, length: 5), severity: .warning, message: "late", recoveryLine: nil),
        .init(id: "r#0@main.tex:10..<12", nsRange: NSRange(location: 10, length: 2), severity: .warning, message: "inner", recoveryLine: "no provisional rendering"),
        .init(id: "r#1@main.tex:10..<20", nsRange: NSRange(location: 10, length: 10), severity: .error, message: "outer", recoveryLine: "recovery: rendered plain"),
    ]
    let text = "0123456789\nabcdefghij\nklmnopqrst\nuvwxyz0123\nABCDEFGHIJ"

    func lineOf(_ utf16: Int) -> Int? { AccessibleEditorModel(text: text).line(containingUTF16: utf16)?.number }

    func testDocumentOrderPutsEnclosingRangeFirst() {
        XCTAssertEqual(Nav.ordered(items).map(\.id), ["r#1@main.tex:10..<20", "r#0@main.tex:10..<12", "r#2@main.tex:40..<45"])
        XCTAssertEqual(Nav.ordered(items), Nav.ordered(Nav.ordered(items)), "idempotent")
        XCTAssertEqual(Nav.summary(items), "1 error, 2 warnings")
        XCTAssertEqual(Nav.summary([items[0]]), "1 warning")
        XCTAssertEqual(Nav.summary([]), "no diagnostics")
    }

    func testNextAndPreviousWrapAndAnnounceNOfM() throws {
        let first = try XCTUnwrap(Nav.step(items, fromUTF16: 0, forward: true, lineOf: lineOf))
        XCTAssertEqual(first.ordinal, 1); XCTAssertEqual(first.total, 3); XCTAssertEqual(first.line, 1)
        XCTAssertEqual(first.announcement, "Error 1 of 3, line 1: outer — recovery: rendered plain")
        // Same start offset: stepping by identity reaches the inner one.
        let second = try XCTUnwrap(Nav.step(items, fromUTF16: 10, forward: true, currentID: first.item.id, lineOf: lineOf))
        XCTAssertEqual(second.announcement, "Warning 2 of 3, line 1: inner — no provisional rendering")
        // Without the identity the caret at 10 skips both (they do not start after it).
        let byPosition = try XCTUnwrap(Nav.step(items, fromUTF16: 10, forward: true, lineOf: lineOf))
        XCTAssertEqual(byPosition.ordinal, 3)
        XCTAssertEqual(byPosition.announcement, "Warning 3 of 3, line 4: late")
        let wrapped = try XCTUnwrap(Nav.step(items, fromUTF16: 40, forward: true, currentID: byPosition.item.id, lineOf: lineOf))
        XCTAssertTrue(wrapped.wrapped)
        XCTAssertEqual(wrapped.announcement, "Error 1 of 3, line 1: outer — recovery: rendered plain (wrapped to start)")
        let back = try XCTUnwrap(Nav.step(items, fromUTF16: 5, forward: false, lineOf: lineOf))
        XCTAssertTrue(back.wrapped)
        XCTAssertEqual(back.announcement, "Warning 3 of 3, line 4: late (wrapped to end)")
        let backByID = try XCTUnwrap(Nav.step(items, fromUTF16: 40, forward: false, currentID: back.item.id, lineOf: lineOf))
        XCTAssertEqual(backByID.ordinal, 2); XCTAssertFalse(backByID.wrapped)
        XCTAssertNil(Nav.step([], fromUTF16: 0, forward: true))
    }

    func testCurrentDescribesTheDiagnosticUnderTheCaret() throws {
        let here = try XCTUnwrap(Nav.current(items, atUTF16: 15, lineOf: lineOf))
        XCTAssertEqual(here.ordinal, 1, "the enclosing error is first in document order")
        XCTAssertFalse(here.wrapped)
        XCTAssertEqual(here.announcement, "Error 1 of 3, line 1: outer — recovery: rendered plain")
        XCTAssertEqual(Nav.current(items, atUTF16: 45)?.ordinal, 3, "a caret at the end of a mark counts")
        XCTAssertNil(Nav.current(items, atUTF16: 30))
    }
}
