import AppKit
import XCTest
@testable import FlashTeXMac

/// Pure decision logic for GH74: Tab/Shift-Tab indentation and the
/// completion-snippet overtype hook. No NSTextView involved — these are the
/// same functions the Coordinator (SourceEditorView.swift) and Completion.swift
/// call through their small hooks.
final class EditorKeyHandlingTests: XCTestCase {
    typealias EKH = EditorKeyHandling

    // MARK: line starts / multi-line

    func testLineStartsTouchesEveryLineACaretOrSelectionSpans() {
        let text = "aaa\nbbb\nccc\nddd"
        XCTAssertEqual(EKH.lineStarts(in: text, range: NSRange(location: 1, length: 0)), [0], "caret: just its own line")
        XCTAssertEqual(EKH.lineStarts(in: text, range: NSRange(location: 1, length: 6)), [0, 4], "selection crossing one newline touches two lines")
        XCTAssertEqual(EKH.lineStarts(in: text, range: NSRange(location: 0, length: 4)), [0],
                       "ends exactly at the start of line 2: line 2 is not touched")
        XCTAssertEqual(EKH.lineStarts(in: text, range: NSRange(location: 0, length: 15)), [0, 4, 8, 12], "whole buffer")
        XCTAssertFalse(EKH.isMultiLine(text, range: NSRange(location: 1, length: 0)))
        XCTAssertFalse(EKH.isMultiLine(text, range: NSRange(location: 0, length: 2)), "same line")
        XCTAssertTrue(EKH.isMultiLine(text, range: NSRange(location: 1, length: 6)))
    }

    // MARK: indent

    func testIndentPrefixesEveryTouchedLineAndGrowsTheSelection() {
        let text = "aa\nbb\ncc"
        let (edits, selection) = EKH.indentEdits(in: text, range: NSRange(location: 1, length: 5), unit: "  ")!
        XCTAssertEqual(edits, [
            EKH.LineEdit(range: NSRange(location: 0, length: 0), replacement: "  "),
            EKH.LineEdit(range: NSRange(location: 3, length: 0), replacement: "  "),
        ])
        // Apply in the order the Coordinator does (last line first) to check the result text.
        var s = text as NSString
        for e in edits.sorted(by: { $0.range.location > $1.range.location }) {
            s = s.replacingCharacters(in: e.range, with: e.replacement) as NSString
        }
        XCTAssertEqual(s as String, "  aa\n  bb\ncc")
        XCTAssertEqual(selection, NSRange(location: 0, length: 10), "selects both full (now indented) lines, through their shared boundary")
    }

    func testIndentAtCaretIndentsOnlyItsOwnLineAndKeepsTheCaretACaret() {
        let (edits, selection) = EKH.indentEdits(in: "aa\nbb", range: NSRange(location: 4, length: 0), unit: "\t")!
        XCTAssertEqual(edits, [EKH.LineEdit(range: NSRange(location: 3, length: 0), replacement: "\t")])
        XCTAssertEqual(selection, NSRange(location: 5, length: 0), "shifts with the inserted tab, stays a caret")
    }

    // MARK: outdent

    func testOutdentRemovesUpToOneUnitOfMatchingLeadingWhitespace() {
        let text = "    aa\n  bb\ncc\n\tdd"
        let (edits, _) = EKH.outdentEdits(in: text, range: NSRange(location: 0, length: text.utf16.count), unit: "    ")!
        // Line 1: 4 leading spaces, all removed. Line 2: only 2 (< 4), all removed.
        // Line 3: none, no edit. Line 4: a tab, not a space, no edit (unit is spaces).
        XCTAssertEqual(edits.count, 2)
        XCTAssertTrue(edits.contains(EKH.LineEdit(range: NSRange(location: 0, length: 4), replacement: "")))
        XCTAssertTrue(edits.contains(EKH.LineEdit(range: NSRange(location: 7, length: 2), replacement: "")))
    }

    func testOutdentWithTabUnitRemovesOneLeadingTabOnly() {
        let (edits, _) = EKH.outdentEdits(in: "\t\taa", range: NSRange(location: 0, length: 0), unit: "\t")!
        XCTAssertEqual(edits, [EKH.LineEdit(range: NSRange(location: 0, length: 1), replacement: "")])
    }

    func testOutdentWithNoLeadingWhitespaceIsANoOp() {
        let (edits, selection) = EKH.outdentEdits(in: "aa", range: NSRange(location: 1, length: 0), unit: "    ")!
        XCTAssertEqual(edits, [])
        XCTAssertEqual(selection, NSRange(location: 1, length: 0), "nothing to remove: the caret does not move")
    }

    // MARK: completion-snippet closer (Completion.swift's hook)

    func testProgrammaticCloserFindsTheCloserRightAfterTheSnippetCaret() {
        // `\section{}` with the caret placed right after `{` (Completion.swift's `Vocabulary.Entry.snippet`).
        XCTAssertEqual(EKH.programmaticCloser(in: "\\section{}", insertedAt: 5, caretUTF16: 5 + 9), 14)
        XCTAssertNil(EKH.programmaticCloser(in: "\\section{}", insertedAt: 5, caretUTF16: 5 + 3), "not before a closer")
        XCTAssertNil(EKH.programmaticCloser(in: "word", insertedAt: 0, caretUTF16: 4), "caret at the very end: nothing follows")
    }
}
