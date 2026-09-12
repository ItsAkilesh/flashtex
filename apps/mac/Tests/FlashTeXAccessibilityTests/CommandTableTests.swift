import XCTest
@testable import FlashTeXAccessibility

/// The command table must match the README's "Keyboard shortcuts" table exactly
/// (both directions), and the focus order must be documented with rationale.
final class CommandTableTests: XCTestCase {
    static let readme = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        .appendingPathComponent("README.md")

    /// Shortcut cells of the README table, split on " / ".
    func readmeShortcuts() throws -> [String] {
        let text = try String(contentsOf: Self.readme, encoding: .utf8)
        guard let section = text.range(of: "## Keyboard shortcuts") else { throw XCTSkip("README has no shortcuts table") }
        var out: [String] = []
        for line in text[section.upperBound...].split(separator: "\n", omittingEmptySubsequences: false) {
            if line.hasPrefix("## ") { break }
            guard line.hasPrefix("|") else { continue }
            let cells = line.split(separator: "|", omittingEmptySubsequences: false).map { $0.trimmingCharacters(in: .whitespaces) }
            guard cells.count >= 3, cells[1] != "Shortcut", !cells[1].hasPrefix("---") else { continue }
            out += cells[1].components(separatedBy: " / ")
        }
        return out
    }

    func testCommandTableMatchesREADMEShortcuts() throws {
        let readme = try readmeShortcuts()
        XCTAssertGreaterThan(readme.count, 15, "table parsed")
        XCTAssertEqual(Set(readme).count, readme.count, "README shortcuts are unique")
        let table = AccessibilityCommand.allShortcuts
        XCTAssertEqual(Set(readme).subtracting(table), [], "README shortcuts missing from the command table")
        XCTAssertEqual(table.subtracting(readme), [], "command-table shortcuts not documented in the README")
        // Each shortcut belongs to exactly one command.
        let owners = AccessibilityCommand.entries.flatMap { e in e.shortcuts.map { ($0, e.command) } }
        XCTAssertEqual(owners.count, Set(owners.map(\.0)).count)
    }

    func testEntriesAreDiscoverable() {
        for e in AccessibilityCommand.entries {
            XCTAssertFalse(e.title.isEmpty)
            XCTAssertFalse(e.shortcuts.isEmpty, e.title)
            XCTAssertFalse(e.menu.isEmpty, e.title)
            XCTAssertGreaterThan(e.description.count, 20, e.title)
            XCTAssertTrue(e.helpLine.hasPrefix(e.title + " — "), e.helpLine)
        }
        XCTAssertEqual(AccessibilityCommand.helpLines.count, AccessibilityCommand.allCases.count)
        XCTAssertEqual(AccessibilityCommand.nextDiagnostic.entry.helpLine,
                       "Next diagnostic — ⌘⇧] (Navigate): Selects the next diagnostic with a source in the active document (wrapping); refused if its span was edited since the compile. Requires a compile result.")
        XCTAssertEqual(AccessibilityCommand.completion.entry.shortcuts, ["Esc", "⌃Space"])
        XCTAssertEqual(AccessibilityCommand.helpLines, AccessibilityCommand.helpLines, "deterministic")
    }

    func testFocusOrder() {
        XCTAssertEqual(FocusOrder.description, "Editor → Preview → Diagnostics → Capture bar")
        XCTAssertEqual(FocusOrder.panes.count, 4)
        for p in FocusOrder.panes {
            XCTAssertGreaterThan(p.rationale.count, 30, p.name)
            XCTAssertGreaterThan(p.contents.count, 20, p.name)
        }
        XCTAssertTrue(FocusOrder.helpLines[0].hasPrefix("1. Editor: "))
        XCTAssertTrue(FocusOrder.helpLines[3].hasPrefix("4. Capture bar: "))
    }
}
