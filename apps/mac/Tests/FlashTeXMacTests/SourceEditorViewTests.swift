import AppKit
import SwiftUI
import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// `SourceEditorView`: VoiceOver label/value/selection and line-column
/// announcements, windowed diagnostic marks on a large buffer, navigation
/// selections that never fight typing, one-step capture undo through the
/// model, and the large-document keystroke round trip with correct UTF-8
/// caret offsets. Hosted tests put the real `NSTextView` in an `NSWindow`
/// through `NSHostingView` (never key: the tests must not steal focus).
@MainActor
final class SourceEditorViewTests: XCTestCase {

    // MARK: fixtures

    /// ASCII plus multi-byte text on every line, `bytes` or slightly more UTF-8 bytes.
    static func largeDocument(bytes: Int) -> String {
        var s = "\\documentclass{article}\n\\begin{document}\n"
        var n = 0
        while s.utf8.count < bytes {
            s += "Paragraph \(n): the quick brown fox — naïve café \\textbf{bold} $x^2 + y^2 = z^2$ jumps over the lazy dog.\n"
            n += 1
        }
        s += "\\end{document}\n"
        return s
    }

    static func marks(count: Int, in text: String) -> [EditorDiagnostics.Mark] {
        let len = (text as NSString).length
        return (0..<count).map { i in
            EditorDiagnostics.Mark(nsRange: NSRange(location: i * (len / count), length: 5),
                                   severity: i % 3 == 0 ? .error : .warning, message: "mark \(i)", recovery: nil)
        }
    }

    /// Hosts the real editor bound to a `ShellModel`, like `ContentView` does.
    private final class Probe {
        var editApplied: [(ShellModel.PendingEdit, String)] = []
        var bindingSetNs: UInt64 = 0
        var coordinator: SourceEditorView.Coordinator?
    }

    private struct Host: View {
        var model: ShellModel
        var probe: Probe
        var body: some View {
            SourceEditorView(
                text: Binding(get: { model.activeText }, set: { new in
                    model.updateActiveText(new)
                    probe.bindingSetNs = MonotonicClock.nowNs()
                }),
                selection: model.selection,
                pendingEdit: model.pendingEdit,
                marks: model.editorMarks,
                result: model.result,
                onCaretChange: { model.caretUTF16 = $0 },
                onSelectionChange: { model.caretLengthUTF16 = $0.length },
                onEditApplied: { edit, text in
                    probe.editApplied.append((edit, text))
                    model.editApplied(edit, newText: text)
                }
            )
        }
    }

    private func host(_ model: ShellModel, probe: Probe) async throws -> (NSWindow, NSTextView) {
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 400), styleMask: [.titled],
                              backing: .buffered, defer: false)
        window.contentView = NSHostingView(rootView: Host(model: model, probe: probe))
        window.orderFrontRegardless() // never makeKey
        var found: NSTextView?
        try await waitUntil("editor text view") {
            found = TypingBenchDriver.findTextView(in: [window.contentView!]); return found != nil
        }
        let tv = try XCTUnwrap(found)
        probe.coordinator = tv.delegate as? SourceEditorView.Coordinator
        XCTAssertNotNil(probe.coordinator)
        XCTAssertTrue(window.makeFirstResponder(tv))
        return (window, tv)
    }

    private func waitUntil(_ what: String, timeout: TimeInterval = 10, _ cond: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if cond() { return }
            try await Task.sleep(nanoseconds: 10_000_000)
        }
        XCTFail("timed out waiting for \(what)")
    }

    /// Lets the current run-loop turn end (coalesced announcements, async edit delivery).
    private func turn() async throws { try await Task.sleep(nanoseconds: 30_000_000) }

    // MARK: line / column (pure)

    func testSelectionAnnouncementCountsLinesAndUserPerceivedColumns() {
        let text = "ab\nnaïve 👩‍💻x\r\nlast\rline"
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: 0, length: 0)), "Line 1, column 1")
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: 2, length: 0)), "Line 1, column 3")
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: 3, length: 0)), "Line 2, column 1")
        // "naïve " is 6 user characters; the emoji is 5 UTF-16 units (ZWJ sequence).
        let emoji = (text as NSString).range(of: "👩‍💻")
        XCTAssertEqual(emoji.length, 5)
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: emoji.location, length: 0)), "Line 2, column 7")
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: NSMaxRange(emoji), length: 0)), "Line 2, column 8")
        XCTAssertNil(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: emoji.location + 1, length: 0)),
                     "inside a surrogate pair is not a caret position")
        // CRLF is one line break; a bare CR is a line break too.
        let last = (text as NSString).range(of: "last").location
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: last, length: 0)), "Line 3, column 1")
        let line = (text as NSString).range(of: "line").location
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: line, length: 2)),
                       "Selected 2 characters, line 4 column 1 to 3")
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: 1, length: 3)),
                       "Selected 3 characters, line 1 column 2 to line 2 column 2")
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: emoji.location, length: 5)),
                       "Selected 1 character, line 2 column 7 to 8")
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: (text as NSString).length, length: 0)), "Line 4, column 5")
        XCTAssertNil(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: (text as NSString).length + 1, length: 0)))
        XCTAssertNil(SourceEditorView.selectionAnnouncement(text: text, range: NSRange(location: 1, length: 100)))
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: "", range: NSRange(location: 0, length: 0)), "Line 1, column 1")
        XCTAssertEqual(SourceEditorView.caretByte(text: text, utf16: NSMaxRange(emoji)), "ab\nnaïve 👩‍💻".utf8.count)
        XCTAssertNil(SourceEditorView.caretByte(text: text, utf16: emoji.location + 1))
    }

    func testLineColumnOnLargeBufferIsSubMillisecond() {
        let text = Self.largeDocument(bytes: 60_000)
        let end = (text as NSString).length
        let t0 = MonotonicClock.nowNs()
        let lc = SourceEditorView.lineColumn(text: text, utf16: end)
        let ms = Double(MonotonicClock.nowNs() - t0) / 1e6
        XCTAssertEqual(lc?.line, text.split(separator: "\n", omittingEmptySubsequences: false).count)
        XCTAssertEqual(lc?.column, 1)
        XCTAssertLessThan(ms, 1.0, "line/column at the end of a 60 KB buffer took \(ms) ms")
        // Column counting is per line, so a caret inside the last line is exact too.
        let lastLine = "\\end{document}\n"
        XCTAssertEqual(SourceEditorView.lineColumn(text: text, utf16: end - 1)?.column, (lastLine as NSString).length)
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: "a\r\nb", range: NSRange(location: 3, length: 0)), "Line 2, column 1")
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: "a\rb\nc", range: NSRange(location: 4, length: 0)), "Line 3, column 1")
        XCTAssertEqual(SourceEditorView.selectionAnnouncement(text: "a\rb\nc", range: NSRange(location: 3, length: 0)), "Line 2, column 2")
    }

    func testNativeTextIsByteEqualToTheStorageAndCheapToCompare() {
        let tv = NSTextView(frame: .zero)
        XCTAssertEqual(SourceEditorView.nativeText(of: tv), "")
        let text = Self.largeDocument(bytes: 60_000) + "👩‍💻 end"
        tv.string = text
        let t0 = MonotonicClock.nowNs()
        let native = SourceEditorView.nativeText(of: tv)
        let convertMs = Double(MonotonicClock.nowNs() - t0) / 1e6
        XCTAssertTrue(native.sameBytes(as: text))
        XCTAssertTrue(native.isContiguousUTF8)
        let t1 = MonotonicClock.nowNs()
        XCTAssertTrue(native.sameBytes(as: text))
        let compareMs = Double(MonotonicClock.nowNs() - t1) / 1e6
        XCTAssertLessThan(convertMs, 1.0, "native conversion took \(convertMs) ms")
        XCTAssertLessThan(compareMs, 0.2, "byte comparison of the native copy took \(compareMs) ms")
        // An unpaired surrogate cannot be encoded: the bridge's replacement is used.
        tv.string = "a" + String(utf16CodeUnits: [0xD800], count: 1) + "b"
        XCTAssertEqual(SourceEditorView.nativeText(of: tv), tv.string)
        XCTAssertEqual(SourceEditorView.nativeText(of: tv).unicodeScalars.count, 3)
    }

    // MARK: VoiceOver

    func testEditorExposesLabelValueAndSelectedTextRangeAndAnnouncesCaretMoves() async throws {
        let model = ShellModel()
        model.replaceProject(entryText: "first line\nsecond naïve line\n")
        let probe = Probe()
        let (window, tv) = try await host(model, probe: probe)
        defer { window.orderOut(nil) }
        let co = try XCTUnwrap(probe.coordinator)
        var spoken: [String] = []
        co.announce = { spoken.append($0) }

        XCTAssertEqual(tv.accessibilityLabel(), "LaTeX source")
        XCTAssertEqual(tv.accessibilityRole(), .textArea)
        XCTAssertEqual(tv.accessibilityValue() as? String, model.activeText)
        XCTAssertEqual(tv.accessibilityNumberOfCharacters(), (model.activeText as NSString).length)
        XCTAssertNotNil(tv.accessibilityHelp())
        XCTAssertNil(tv.textLayoutManager, "TextKit 1 was selected up front (temporary attributes)")

        // A caret move (arrow key / click) announces once per turn, after the turn.
        tv.setSelectedRange(NSRange(location: 11, length: 0))
        tv.setSelectedRange(NSRange(location: 18, length: 0))
        XCTAssertEqual(spoken, [])
        try await turn()
        XCTAssertEqual(spoken, ["Line 2, column 8"])
        XCTAssertEqual(tv.accessibilitySelectedTextRange(), NSRange(location: 18, length: 0))
        XCTAssertEqual(model.caretUTF16, 18)

        // Selecting announces the extent.
        tv.setSelectedRange(NSRange(location: 0, length: 5))
        try await turn()
        XCTAssertEqual(spoken.last, "Selected 5 characters, line 1 column 1 to 6")
        XCTAssertEqual(tv.accessibilitySelectedTextRange(), NSRange(location: 0, length: 5))
        XCTAssertEqual(model.caretLengthUTF16, 5)

        // Typing is not announced (VoiceOver reads the typed text itself), and
        // the binding carried the edit.
        spoken = []
        tv.setSelectedRange(NSRange(location: 5, length: 0))
        try await turn()
        spoken = []
        tv.insertText("!", replacementRange: NSRange(location: 5, length: 0))
        tv.insertText("?", replacementRange: NSRange(location: 6, length: 0))
        try await turn()
        XCTAssertEqual(spoken, [], "typing steps are not announced")
        XCTAssertEqual(model.activeText, "first!? line\nsecond naïve line\n")
        XCTAssertEqual(tv.accessibilityValue() as? String, model.activeText)
        XCTAssertEqual(model.caretUTF16, 7)

        // A navigation selection announces immediately and is exposed to AX.
        model.selection = .init(path: "main.tex", nsRange: NSRange(location: 13, length: 6), token: 1)
        try await waitUntil("navigation applied") { tv.selectedRange() == NSRange(location: 13, length: 6) }
        XCTAssertEqual(spoken, ["Selected 6 characters, line 2 column 1 to 7"])
        XCTAssertEqual(tv.accessibilitySelectedTextRange(), NSRange(location: 13, length: 6))
        try await turn()
        XCTAssertEqual(spoken.count, 1, "the programmatic selection change is not announced twice")
    }

    // MARK: marks

    func testMarksArePaintedOnlyAroundTheVisibleWindowAndFast() throws {
        let text = Self.largeDocument(bytes: 60_000)
        let scroll = CompletingTextView.scrollable()
        let tv = scroll.documentView as! NSTextView
        _ = tv.layoutManager
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 400), styleMask: [.titled],
                              backing: .buffered, defer: false)
        window.contentView = scroll
        window.orderFrontRegardless()
        defer { window.orderOut(nil) }
        tv.string = text
        let lm = try XCTUnwrap(tv.layoutManager)
        // Steady state: the document is laid out before diagnostics arrive
        // (TextKit 1 lays the whole text out in the background after display).
        lm.ensureLayout(for: try XCTUnwrap(tv.textContainer))
        let marks = Self.marks(count: 200, in: text)
        XCTAssertEqual(marks.count, 200)
        let painter = SourceEditorView.MarkPainter()

        var t0 = MonotonicClock.nowNs()
        painter.update(marks, in: tv, reset: false)
        let firstMs = Double(MonotonicClock.nowNs() - t0) / 1e6
        XCTAssertEqual(painter.paints, 1)
        let window0 = SourceEditorView.MarkPainter.window(for: tv)
        XCTAssertEqual(window0.location, 0)
        XCTAssertLessThan(window0.length, (text as NSString).length / 2, "only a window around the visible text is painted")
        XCTAssertEqual(painter.painted, [window0])

        func underlined(_ mark: EditorDiagnostics.Mark) -> Bool {
            lm.temporaryAttribute(.underlineStyle, atCharacterIndex: mark.nsRange.location, effectiveRange: nil) != nil
        }
        let inside = marks.filter { NSMaxRange($0.nsRange) <= NSMaxRange(window0) }
        let outside = marks.filter { $0.nsRange.location >= NSMaxRange(window0) }
        XCTAssertGreaterThan(inside.count, 0); XCTAssertGreaterThan(outside.count, 100)
        XCTAssertTrue(inside.allSatisfy(underlined))
        XCTAssertFalse(outside.contains(where: underlined))
        XCTAssertEqual(lm.temporaryAttribute(.underlineColor, atCharacterIndex: marks[0].nsRange.location, effectiveRange: nil) as? NSColor, .systemRed)
        XCTAssertEqual(lm.temporaryAttribute(.toolTip, atCharacterIndex: marks[1].nsRange.location, effectiveRange: nil) as? String, "mark 1")

        // Unchanged marks cost nothing.
        t0 = MonotonicClock.nowNs()
        painter.update(marks, in: tv, reset: false)
        let sameMs = Double(MonotonicClock.nowNs() - t0) / 1e6
        XCTAssertEqual(painter.paints, 1)

        // Every mark moved (typing before them rebases all 200): clear + repaint the window.
        let shifted = marks.map { EditorDiagnostics.Mark(nsRange: NSRange(location: $0.nsRange.location + 1, length: 5),
                                                         severity: $0.severity, message: $0.message, recovery: $0.recovery) }
        t0 = MonotonicClock.nowNs()
        painter.update(shifted, in: tv, reset: false)
        let shiftedMs = Double(MonotonicClock.nowNs() - t0) / 1e6
        XCTAssertEqual(painter.paints, 2)
        XCTAssertNil(lm.temporaryAttribute(.underlineStyle, atCharacterIndex: marks[1].nsRange.location, effectiveRange: nil))
        XCTAssertNotNil(lm.temporaryAttribute(.underlineStyle, atCharacterIndex: shifted[1].nsRange.location, effectiveRange: nil))

        // Scrolling to the end paints the newly visible window only.
        tv.scrollRangeToVisible(NSRange(location: (text as NSString).length, length: 0))
        t0 = MonotonicClock.nowNs()
        painter.scrolled(tv)
        let scrollMs = Double(MonotonicClock.nowNs() - t0) / 1e6
        XCTAssertEqual(painter.paints, 3)
        XCTAssertTrue(underlined(shifted[199]), "the last mark is painted once its window is visible")
        XCTAssertEqual(painter.painted.count, 2, "two disjoint painted windows; the text between them is untouched")
        XCTAssertFalse(underlined(shifted[100]), "marks between the two windows are still unpainted")
        painter.scrolled(tv)
        XCTAssertEqual(painter.paints, 3, "a scroll inside the painted range repaints nothing")

        print("marks: first \(firstMs) ms, unchanged \(sameMs) ms, all-shifted \(shiftedMs) ms, scroll-to-end \(scrollMs) ms (60 KB, 200 marks, debug build)")
        for (name, ms) in [("first", firstMs), ("unchanged", sameMs), ("shifted", shiftedMs), ("scroll", scrollMs)] {
            XCTAssertLessThan(ms, 2.0, "\(name) mark pass took \(ms) ms")
        }

        // A text reset drops every temporary attribute; the painter starts over.
        tv.string = text
        painter.update(shifted, in: tv, reset: true)
        XCTAssertEqual(painter.paints, 4)
        XCTAssertEqual(painter.painted, [SourceEditorView.MarkPainter.window(for: tv)])
        painter.update([], in: tv, reset: false)
        XCTAssertEqual(painter.paints, 4)
        XCTAssertFalse(shifted.contains(where: underlined), "clearing marks removes every painted underline")
    }

    func testPainterTracksGapsAndEdits() {
        let painter = SourceEditorView.MarkPainter()
        XCTAssertEqual(painter.gaps(in: NSRange(location: 10, length: 20)), [NSRange(location: 10, length: 20)])
        XCTAssertEqual(SourceEditorView.MarkPainter.merged([NSRange(location: 30, length: 10), NSRange(location: 0, length: 10),
                                                            NSRange(location: 10, length: 5), NSRange(location: 12, length: 0)]),
                       [NSRange(location: 0, length: 15), NSRange(location: 30, length: 10)])
        // Gaps around and between painted ranges.
        let tv = NSTextView(frame: .zero)
        tv.string = String(repeating: "x", count: 100)
        let mark = EditorDiagnostics.Mark(nsRange: NSRange(location: 50, length: 2), severity: .warning, message: "m", recovery: nil)
        painter.update([mark], in: tv, reset: false) // offscreen: the whole text is the window
        XCTAssertEqual(painter.painted, [NSRange(location: 0, length: 100)])
        XCTAssertEqual(painter.gaps(in: NSRange(location: 20, length: 30)), [])
        painter.update([], in: tv, reset: false)
        XCTAssertEqual(painter.painted, [NSRange(location: 0, length: 100)])
        // An insertion before a painted range grows it to keep covering the shifted attributes.
        painter.noteEdit(range: NSRange(location: 5, length: 0), replacementLength: 7)
        XCTAssertEqual(painter.painted, [NSRange(location: 0, length: 107)])
        painter.noteEdit(range: NSRange(location: 500, length: 0), replacementLength: 3)
        XCTAssertEqual(painter.painted, [NSRange(location: 0, length: 107)], "an edit after every painted range changes nothing")
        painter.noteEdit(range: NSRange(location: 10, length: 50), replacementLength: 1)
        XCTAssertEqual(painter.painted, [NSRange(location: 0, length: 107)], "a deletion never shrinks the estimate")
    }

    func testWholeDocumentApplyMarksStillPaintsEverything() throws {
        let text = Self.largeDocument(bytes: 60_000)
        let tv = NSTextView(frame: NSRect(x: 0, y: 0, width: 300, height: 100))
        tv.string = text
        let marks = Self.marks(count: 200, in: text)
        let t0 = MonotonicClock.nowNs()
        SourceEditorView.applyMarks(marks, to: tv)
        let ms = Double(MonotonicClock.nowNs() - t0) / 1e6
        let lm = try XCTUnwrap(tv.layoutManager)
        XCTAssertTrue(marks.allSatisfy { lm.temporaryAttribute(.underlineStyle, atCharacterIndex: $0.nsRange.location, effectiveRange: nil) != nil })
        // Errors win over warnings where they overlap.
        let a = EditorDiagnostics.Mark(nsRange: NSRange(location: 10, length: 10), severity: .warning, message: "w", recovery: nil)
        let b = EditorDiagnostics.Mark(nsRange: NSRange(location: 15, length: 10), severity: .error, message: "e", recovery: "fix")
        SourceEditorView.applyMarks([b, a], to: tv)
        XCTAssertEqual(lm.temporaryAttribute(.underlineColor, atCharacterIndex: 12, effectiveRange: nil) as? NSColor, .systemOrange)
        XCTAssertEqual(lm.temporaryAttribute(.underlineColor, atCharacterIndex: 17, effectiveRange: nil) as? NSColor, .systemRed)
        XCTAssertEqual(lm.temporaryAttribute(.toolTip, atCharacterIndex: 17, effectiveRange: nil) as? String, "e\n↳ fix")
        XCTAssertNil(lm.temporaryAttribute(.underlineStyle, atCharacterIndex: marks[3].nsRange.location, effectiveRange: nil))
        print("whole-document applyMarks: \(ms) ms (60 KB, 200 marks, offscreen view)")
    }

    func testHostedEditorRepaintsMarksWhenScrolled() async throws {
        let model = ShellModel()
        let text = Self.largeDocument(bytes: 60_000)
        model.replaceProject(entryText: text)
        let probe = Probe()
        let (window, tv) = try await host(model, probe: probe)
        defer { window.orderOut(nil) }
        let co = try XCTUnwrap(probe.coordinator)
        let lm = try XCTUnwrap(tv.layoutManager)
        // Diagnostics through the model: a result whose diagnostics point at the buffer.
        let last = text.utf8.count - 20
        let diagnostics = (0..<200).map { i in
            RuntimeV1.Diagnostic(severity: .warning, message: "d\(i)", source: .init(path: "main.tex", startByte: i == 199 ? last : i * 200, endByte: (i == 199 ? last : i * 200) + 4), recovery: nil)
        }
        model.result = RuntimeV1.CompileResult(projectId: "p", revision: model.editorRevision, status: .ok, pages: [],
                                               diagnostics: diagnostics, pdfPath: nil)
        try await waitUntil("marks painted") { co.marks.paints >= 1 }
        // Byte ranges that land inside a multi-byte scalar are not marks.
        XCTAssertEqual(co.marks.marks, model.editorMarks)
        XCTAssertGreaterThan(co.marks.marks.count, 150)
        let lastMark = try XCTUnwrap(co.marks.marks.last)
        XCTAssertNil(lm.temporaryAttribute(.underlineStyle, atCharacterIndex: lastMark.nsRange.location, effectiveRange: nil),
                     "the last mark is far below the visible window")
        tv.scrollRangeToVisible(NSRange(location: (text as NSString).length, length: 0))
        try await waitUntil("scroll repaint") {
            lm.temporaryAttribute(.underlineStyle, atCharacterIndex: lastMark.nsRange.location, effectiveRange: nil) != nil
        }
        XCTAssertGreaterThanOrEqual(co.marks.paints, 2)
    }

    // MARK: navigation vs typing

    func testNavigationSelectionWaitsForATypingPauseInsteadOfMovingTheCaretBack() async throws {
        let model = ShellModel()
        model.replaceProject(entryText: "alpha\nbeta\n")
        let probe = Probe()
        let (window, tv) = try await host(model, probe: probe)
        defer { window.orderOut(nil) }
        let co = try XCTUnwrap(probe.coordinator)
        let end = (model.activeText as NSString).length
        tv.setSelectedRange(NSRange(location: end, length: 0))
        tv.insertText("x", replacementRange: NSRange(location: end, length: 0))
        XCTAssertEqual(model.activeText, "alpha\nbeta\nx")
        XCTAssertNotEqual(co.lastUserEditNs, 0)

        // Navigation to an earlier range while typing: deferred, caret stays.
        model.selection = .init(path: "main.tex", nsRange: NSRange(location: 0, length: 5), token: 1)
        try await turn()
        XCTAssertEqual(tv.selectedRange(), NSRange(location: end + 1, length: 0), "the caret did not move backwards while typing")
        XCTAssertEqual(co.deferredSelection?.token, 1)
        XCTAssertEqual(co.appliedToken, 1)
        // Still typing: still deferred.
        tv.insertText("y", replacementRange: NSRange(location: end + 1, length: 0))
        try await turn()
        XCTAssertEqual(tv.selectedRange(), NSRange(location: end + 2, length: 0))
        XCTAssertEqual(model.activeText, "alpha\nbeta\nxy")
        // Typing pauses: the newest navigation applies.
        try await waitUntil("deferred navigation", timeout: 3) { tv.selectedRange() == NSRange(location: 0, length: 5) }
        XCTAssertNil(co.deferredSelection)
        XCTAssertEqual(co.announcements.last, "Selected 5 characters, line 1 column 1 to 6")

        // A navigation forward of the caret applies at once, even mid-typing.
        tv.setSelectedRange(NSRange(location: 0, length: 0))
        tv.insertText("z", replacementRange: NSRange(location: 0, length: 0))
        model.selection = .init(path: "main.tex", nsRange: NSRange(location: 7, length: 4), token: 2)
        try await waitUntil("forward navigation") { tv.selectedRange() == NSRange(location: 7, length: 4) }
        XCTAssertNil(co.deferredSelection)
        XCTAssertEqual((tv.string as NSString).substring(with: tv.selectedRange()), "beta")

        // A superseded deferred token is dropped: only the newest applies.
        tv.setSelectedRange(NSRange(location: 12, length: 0))
        tv.insertText("w", replacementRange: NSRange(location: 12, length: 0))
        model.selection = .init(path: "main.tex", nsRange: NSRange(location: 1, length: 1), token: 3)
        try await turn()
        XCTAssertEqual(co.deferredSelection?.token, 3)
        model.selection = .init(path: "main.tex", nsRange: NSRange(location: 2, length: 2), token: 4)
        try await waitUntil("newest navigation", timeout: 3) { tv.selectedRange() == NSRange(location: 2, length: 2) }
        XCTAssertNil(co.deferredSelection)
        XCTAssertEqual(co.appliedToken, 4)
    }

    func testRecreatedCoordinatorDoesNotReplayAnOldNavigation() {
        let view = SourceEditorView(text: .constant("abc"), selection: .init(path: "main.tex", nsRange: NSRange(location: 0, length: 1), token: 7))
        let co = view.makeCoordinator()
        XCTAssertEqual(co.appliedToken, 7, "an old navigation token is treated as already applied")
        XCTAssertEqual(co.appliedEditToken, 0, "a pending edit the model still waits on is applied")
        XCTAssertEqual(SourceEditorView(text: .constant("")).makeCoordinator().appliedToken, 0)
    }

    // MARK: capture insertion undo

    func testCaptureInsertionIsOneUndoStepDeliveredToTheModelOnce() async throws {
        let model = ShellModel()
        model.replaceProject(entryText: "\\begin{document}\n\\end{document}\n")
        let probe = Probe()
        let (window, tv) = try await host(model, probe: probe)
        defer { window.orderOut(nil) }
        let undo = try XCTUnwrap(tv.undoManager)
        let base = model.editorRevision

        // The user types (one coalesced typing step), then a capture is approved.
        let at = ("\\begin{document}\n" as NSString).length
        tv.setSelectedRange(NSRange(location: at, length: 0))
        for (i, ch) in ["a", "b", "c"].enumerated() { tv.insertText(ch, replacementRange: NSRange(location: at + i, length: 0)) }
        XCTAssertEqual(model.activeText, "\\begin{document}\nabc\\end{document}\n")
        XCTAssertEqual(model.editorRevision, base + 3)
        try await turn() // the keystrokes' event (and its by-event undo group) ends before the capture is approved
        let insert = "\n\\[ E = mc^2 \\]\n"
        let edit = ShellModel.PendingEdit(path: "main.tex", nsRange: NSRange(location: at + 3, length: 0), text: insert, token: 1)
        model.pendingEdit = edit
        try await waitUntil("edit applied in the view") { tv.string.contains(insert) }
        let afterInsert = "\\begin{document}\nabc" + insert + "\\end{document}\n"
        XCTAssertEqual(tv.string, afterInsert)
        XCTAssertEqual(tv.selectedRange(), NSRange(location: at + 3, length: (insert as NSString).length), "the insertion is selected")
        try await waitUntil("model told once") { probe.editApplied.count == 1 }
        XCTAssertEqual(probe.editApplied[0].0, edit)
        XCTAssertEqual(probe.editApplied[0].1, afterInsert)
        XCTAssertEqual(model.activeText, afterInsert, "the model adopted the edited text through onEditApplied")
        XCTAssertEqual(model.editorRevision, base + 4, "the insertion bumped the revision exactly once")
        XCTAssertNil(model.pendingEdit)
        XCTAssertEqual(undo.undoActionName, "Insert Capture")
        XCTAssertEqual(probe.coordinator?.announcements.last, "Inserted capture. Selected 16 characters, line 2 column 4 to line 4 column 1")

        // More typing after the capture is its own step.
        let tail = at + 3 + (insert as NSString).length
        tv.setSelectedRange(NSRange(location: tail, length: 0))
        tv.insertText("d", replacementRange: NSRange(location: tail, length: 0))
        tv.insertText("e", replacementRange: NSRange(location: tail + 1, length: 0))
        let afterTail = "\\begin{document}\nabc" + insert + "de\\end{document}\n"
        XCTAssertEqual(model.activeText, afterTail)
        try await turn()

        // Undo peels the steps in order; each reaches the model through the binding.
        XCTAssertTrue(undo.canUndo)
        undo.undo()
        XCTAssertEqual(tv.string, afterInsert, "undo 1: the trailing typing")
        XCTAssertEqual(model.activeText, afterInsert)
        undo.undo()
        XCTAssertEqual(tv.string, "\\begin{document}\nabc\\end{document}\n", "undo 2: the capture insertion as one step")
        XCTAssertEqual(model.activeText, "\\begin{document}\nabc\\end{document}\n")
        undo.undo()
        XCTAssertEqual(tv.string, "\\begin{document}\n\\end{document}\n", "undo 3: the leading typing")
        XCTAssertEqual(model.activeText, tv.string)
        XCTAssertEqual(probe.editApplied.count, 1, "undo/redo never re-deliver the pending edit")

        undo.redo()
        XCTAssertEqual(tv.string, "\\begin{document}\nabc\\end{document}\n")
        undo.redo()
        XCTAssertEqual(tv.string, afterInsert, "redo restores the capture as one step")
        XCTAssertEqual(model.activeText, afterInsert)
        undo.redo()
        XCTAssertEqual(tv.string, afterTail)
        XCTAssertEqual(model.activeText, afterTail)
        XCTAssertFalse(undo.canRedo)
        XCTAssertEqual(probe.editApplied.count, 1)
        try await turn()
        XCTAssertEqual(model.activeText, afterTail, "no stale binding write after the turn")
    }

    func testPendingEditOutsideTheBufferIsReportedWithoutChangingText() async throws {
        let model = ShellModel()
        model.replaceProject(entryText: "short\n")
        let probe = Probe()
        let (window, tv) = try await host(model, probe: probe)
        defer { window.orderOut(nil) }
        let edit = ShellModel.PendingEdit(path: "main.tex", nsRange: NSRange(location: 50, length: 0), text: "x", token: 1)
        model.pendingEdit = edit
        try await waitUntil("model told") { probe.editApplied.count == 1 }
        XCTAssertEqual(probe.editApplied[0].1, "short\n")
        XCTAssertEqual(tv.string, "short\n")
        XCTAssertNil(model.pendingEdit)
        XCTAssertFalse(tv.undoManager?.canUndo ?? true)
    }

    // MARK: large document keystrokes

    func testLargeDocumentKeystrokeRoundTripAndCaretBytesStayCorrect() async throws {
        let model = ShellModel()
        let seed = Self.largeDocument(bytes: 60_000)
        model.replaceProject(entryText: seed)
        let probe = Probe()
        let (window, tv) = try await host(model, probe: probe)
        defer { window.orderOut(nil) }
        XCTAssertGreaterThanOrEqual(seed.utf8.count, 60_000)

        // Type 200 keystrokes before `\end{document}`: ASCII, 2-, 3- and 4-byte scalars, newlines.
        let script = Array(repeating: ["x", "é", "→", "👩‍💻", " ", "\n", "y", "ü"], count: 25).flatMap { $0 }
        XCTAssertEqual(script.count, 200)
        var caret = (seed as NSString).range(of: "\\end{document}", options: .backwards).location
        tv.setSelectedRange(NSRange(location: caret, length: 0))
        try await turn()
        var roundTripsMs: [Double] = []
        var typed = ""
        let insertAtByte = SourceEditorView.caretByte(text: seed, utf16: caret)!
        for key in script {
            TypingBench.shared.recorder.delegateReported(at: 0)
            tv.insertText(key, replacementRange: NSRange(location: caret, length: 0))
            caret += (key as NSString).length
            typed += key
            let delegateNs = try XCTUnwrap(TypingBench.shared.recorder.pendingDelegateNs)
            XCTAssertNotEqual(delegateNs, 0, "the delegate stamped this keystroke")
            XCTAssertGreaterThanOrEqual(probe.bindingSetNs, delegateNs)
            roundTripsMs.append(Double(probe.bindingSetNs &- delegateNs) / 1e6)
            // The model holds the edited text and a caret whose UTF-8 offset is exact.
            XCTAssertEqual(tv.selectedRange(), NSRange(location: caret, length: 0))
            XCTAssertEqual(model.caretUTF16, caret)
            XCTAssertEqual(model.caretLengthUTF16, 0)
            XCTAssertEqual(model.caretByte, insertAtByte + typed.utf8.count, "caret byte after typing \(typed.suffix(3).debugDescription)")
        }
        XCTAssertEqual(model.activeText.utf8.count, seed.utf8.count + typed.utf8.count)
        XCTAssertTrue(model.activeText.sameBytes(as: tv.string))
        let stats = LatencyStats(roundTripsMs)
        print("large-document keystrokes: \(stats.count) round trips, p50 \(stats.p50Ms!) ms, p99 \(stats.p99Ms!) ms, max \(stats.maxMs!) ms (60 KB, debug build)")
        for (i, ms) in roundTripsMs.enumerated() {
            XCTAssertLessThan(ms, 1.0, "keystroke \(i) (\(script[i].debugDescription)) textDidChange -> binding took \(ms) ms")
        }
        // The last keystroke's caret maps back through the contract conversion.
        let byte = try XCTUnwrap(model.caretByte)
        XCTAssertEqual(model.activeText.nsRange(utf8Bytes: .init(path: "main.tex", startByte: byte, endByte: byte)), NSRange(location: caret, length: 0))
    }
}
