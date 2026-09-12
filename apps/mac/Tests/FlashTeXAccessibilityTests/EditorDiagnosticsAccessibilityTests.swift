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

    // MARK: the diagnostics-list row against the keyboard navigator

    func diagnostic(_ severity: RuntimeV1.Severity, _ message: String, recovery: String? = nil,
                    source: (Int, Int)? = (10, 20)) -> RuntimeV1.Diagnostic {
        RuntimeV1.Diagnostic(severity: severity, message: message,
                             source: source.map { .init(path: "main.tex", startByte: $0.0, endByte: $0.1) }, recovery: recovery)
    }

    func testRowLabelValueAndActionsFollowTheResultStatus() {
        let withNote = diagnostic(.error, "outer", recovery: "rendered plain")
        let bare = diagnostic(.warning, "late", source: nil)
        let recovered = DiagnosticRowAccessibility(withNote, index: 0, total: 3, status: .recovered)
        XCTAssertEqual(recovered.label, "Diagnostic 1 of 3: Error: outer")
        XCTAssertEqual(recovered.value, "recovery: rendered plain; main.tex bytes 10 to 20")
        XCTAssertNil(recovered.hint)
        XCTAssertEqual(recovered.actions, ["Go to source"])
        // No note: "no provisional rendering" only when the result recovered.
        let bareRecovered = DiagnosticRowAccessibility(bare, index: 2, total: 3, status: .recovered)
        XCTAssertEqual(bareRecovered.label, "Diagnostic 3 of 3: Warning: late")
        XCTAssertEqual(bareRecovered.value, "no provisional rendering")
        XCTAssertEqual(bareRecovered.hint, "No source mapping; listed only.")
        XCTAssertEqual(bareRecovered.actions, [])
        let bareOK = DiagnosticRowAccessibility(bare, index: 2, total: 3, status: .ok)
        XCTAssertEqual(bareOK.value, "", "an ok result without a note says nothing about recovery")
        XCTAssertEqual(DiagnosticRowAccessibility(bare, index: 0, total: 1, status: .failed).value, "")
        XCTAssertEqual(DiagnosticRowAccessibility.recoveryLine(recovery: nil, status: .recovered), "no provisional rendering")
        XCTAssertNil(DiagnosticRowAccessibility.recoveryLine(recovery: nil, status: .ok))
        XCTAssertEqual(DiagnosticRowAccessibility.recoveryLine(recovery: "x", status: .ok), "recovery: x")
        // The explanation line sits between the recovery line and the source
        // bytes, in the order the navigator announces it (message — recovery — explanation).
        let explained = DiagnosticRowAccessibility(withNote, index: 0, total: 3, status: .recovered,
                                                   explanation: "A group was opened by \\textbf and never closed.")
        XCTAssertEqual(explained.value, "recovery: rendered plain; A group was opened by \\textbf and never closed.; main.tex bytes 10 to 20")
        let step = Nav.Step(item: Nav.Item(id: "r#0", nsRange: NSRange(location: 10, length: 10), severity: .error, message: "outer",
                                           recoveryLine: "recovery: rendered plain", explanation: "A group was opened by \\textbf and never closed."),
                            ordinal: 1, total: 3, wrapped: false, line: 1)
        XCTAssertEqual(step.announcement, "Error 1 of 3, line 1: outer — recovery: rendered plain — A group was opened by \\textbf and never closed.")
        XCTAssertEqual(DiagnosticRowAccessibility(withNote, index: 0, total: 3, status: .recovered, explanation: "").value,
                       "recovery: rendered plain; main.tex bytes 10 to 20", "an empty explanation adds nothing")
    }

    /// The list row, the keyboard navigator's "n of m" announcement and the
    /// document model's diagnostic element speak the same severity word,
    /// message and recovery line for the same diagnostic.
    func testRowAgreesWithNavigatorAnnouncementsAndDocumentModel() throws {
        let diags = [diagnostic(.error, "outer", recovery: "rendered plain"),
                     diagnostic(.warning, "inner", source: (10, 12)),
                     diagnostic(.warning, "late", source: (40, 45))]
        let status = RuntimeV1.Status.recovered
        let rows = diags.enumerated().map { DiagnosticRowAccessibility($0.element, index: $0.offset, total: diags.count, status: status) }
        // Navigator items built the way the shell builds them (recovery line from the same rule).
        let navItems = diags.enumerated().map { i, d in
            Nav.Item(id: "r#\(i)", nsRange: NSRange(location: d.source!.startByte, length: d.source!.endByte - d.source!.startByte),
                     severity: d.severity, message: d.message,
                     recoveryLine: DiagnosticRowAccessibility.recoveryLine(recovery: d.recovery, status: status))
        }
        var step = try XCTUnwrap(Nav.step(navItems, fromUTF16: 0, forward: true))
        var visited: [String] = []
        for _ in navItems {
            visited.append(step.item.id)
            let i = Int(step.item.id.dropFirst(2))!
            let row = rows[i]
            let severityWord = step.item.severity == .error ? "Error" : "Warning"
            XCTAssertTrue(step.announcement.hasPrefix("\(severityWord) \(step.ordinal) of \(step.total): \(step.item.message)"))
            XCTAssertEqual(row.label, "Diagnostic \(i + 1) of 3: \(severityWord): \(step.item.message)")
            // The row's value starts with the exact recovery line the announcement ends with.
            let line = try XCTUnwrap(step.item.recoveryLine)
            XCTAssertTrue(step.announcement.contains(" — " + line), step.announcement)
            XCTAssertTrue(row.value.hasPrefix(line + "; "), row.value)
            step = try XCTUnwrap(Nav.step(navItems, fromUTF16: step.item.nsRange.location, forward: true, currentID: step.item.id))
        }
        XCTAssertEqual(Set(visited).count, 3, "every diagnostic visited once before wrapping")
        // The document model's element uses the same label and leads with the same recovery line.
        let result = RuntimeV1.CompileResult(projectId: "t", revision: 1, status: status, pages: [], diagnostics: diags, pdfPath: nil)
        let model = AccessibleDocumentModel(result: result)
        for (row, element) in zip(rows, model.diagnostics) {
            XCTAssertTrue(row.label.hasSuffix(element.label), row.label)
            XCTAssertTrue(row.value.hasPrefix(element.value.components(separatedBy: ";")[0]), "\(row.value) vs \(element.value)")
            XCTAssertEqual(row.actions, element.actions)
        }
    }
}
