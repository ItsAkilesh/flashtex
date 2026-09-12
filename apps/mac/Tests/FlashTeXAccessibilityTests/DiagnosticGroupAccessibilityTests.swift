import XCTest
import FlashTeXProtocol
@testable import FlashTeXAccessibility

/// A grouped diagnostics row (identical diagnostics folded into one row)
/// speaks its count and the occurrence it stands on after the message:
/// "Diagnostic 1 of 14: Error: \in is not supported, 12 places, 3 of 12,
/// main.tex line 41". A single place is spoken exactly as before.
final class DiagnosticGroupAccessibilityTests: XCTestCase {
    let diagnostic = RuntimeV1.Diagnostic(severity: .error, message: "\\in is not supported in math mode",
                                          source: .init(path: "main.tex", startByte: 10, endByte: 20), recovery: "rendered as text")

    func testGroupInfoSpokenForm() {
        XCTAssertEqual(DiagnosticRowAccessibility.GroupInfo(count: 12, occurrence: 2, location: "main.tex line 41").spoken,
                       "12 places, 3 of 12, main.tex line 41")
        XCTAssertEqual(DiagnosticRowAccessibility.GroupInfo(count: 2, occurrence: 0, location: "no source").spoken,
                       "2 places, 1 of 2, no source")
        XCTAssertNil(DiagnosticRowAccessibility.GroupInfo(count: 1, occurrence: 0, location: "main.tex line 4").spoken,
                     "one place is not a group")
    }

    func testGroupedRowLabelCarriesCountAndOccurrence() {
        let plain = DiagnosticRowAccessibility(diagnostic, index: 0, total: 14, status: .recovered)
        XCTAssertEqual(plain.label, "Diagnostic 1 of 14: Error: \\in is not supported in math mode")
        let grouped = DiagnosticRowAccessibility(diagnostic, index: 0, total: 14, status: .recovered,
                                                 group: .init(count: 12, occurrence: 2, location: "main.tex line 41"))
        XCTAssertEqual(grouped.label, "Diagnostic 1 of 14: Error: \\in is not supported in math mode, 12 places, 3 of 12, main.tex line 41")
        // The value, hint and action are those of the row's diagnostic, unchanged by grouping.
        XCTAssertEqual(grouped.value, plain.value)
        XCTAssertEqual(grouped.value, "recovery: rendered as text; main.tex bytes 10 to 20")
        XCTAssertNil(grouped.hint)
        XCTAssertEqual(grouped.actions, ["Go to source"])
        // A group of one reads exactly like the ungrouped row.
        let single = DiagnosticRowAccessibility(diagnostic, index: 0, total: 14, status: .recovered,
                                                group: .init(count: 1, occurrence: 0, location: "main.tex line 41"))
        XCTAssertEqual(single, plain)
        // The explanation line still sits in the value, not the label.
        let explained = DiagnosticRowAccessibility(diagnostic, index: 3, total: 14, status: .ok, explanation: "Use \\in inside $…$.",
                                                   group: .init(count: 3, occurrence: 0, location: "chapter.tex line 2"))
        XCTAssertEqual(explained.label, "Diagnostic 4 of 14: Error: \\in is not supported in math mode, 3 places, 1 of 3, chapter.tex line 2")
        XCTAssertEqual(explained.value, "recovery: rendered as text; Use \\in inside $…$.; main.tex bytes 10 to 20")
    }
}
