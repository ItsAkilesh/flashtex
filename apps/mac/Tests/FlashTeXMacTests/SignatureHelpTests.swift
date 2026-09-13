import AppKit
import XCTest
@testable import FlashTeXMac

/// Signature help (SignatureHelp.swift): the pure model and the panel on the
/// real hosted text view (`{` after a command opens it, `}`/Esc/caret leave
/// close it, ⌘⇧Space opens it on demand).
final class SignatureHelpTests: XCTestCase {
    func testInfoFindsTheCommandAndTheActiveArgument() {
        let frac = "$\\frac{a}{b"
        let i = SignatureHelp.info(in: frac, caretUTF16: (frac as NSString).length)
        XCTAssertEqual(i?.command, "frac")
        XCTAssertEqual(i?.arguments, "{num}{den}")
        XCTAssertEqual(i?.activeArgument, 1)
        XCTAssertEqual(i?.display.text, "\\frac{num}{den}")
        XCTAssertEqual(i?.display.active, NSRange(location: 10, length: 5))
        XCTAssertEqual(i?.description, "\\frac{num}{den}: a fraction.")
        XCTAssertEqual(i?.openerUTF16, 9)

        // First argument, caret between auto-closed braces; nested braces are balanced.
        let first = SignatureHelp.info(in: "\\frac{}{}", caretUTF16: 6)
        XCTAssertEqual(first?.activeArgument, 0)
        XCTAssertEqual(first?.display.active, NSRange(location: 5, length: 5))
        let nested = SignatureHelp.info(in: "\\section{The \\emph{best} id", caretUTF16: 27)
        XCTAssertEqual(nested?.command, "section")
        XCTAssertEqual(nested?.activeArgument, 0)
        // An optional argument counts as a group: `\sqrt[3]{` is argument 1 of `[index]{x}`.
        let opt = SignatureHelp.info(in: "\\sqrt[3]{", caretUTF16: 9)
        XCTAssertEqual(opt?.activeArgument, 1)
        XCTAssertEqual(opt?.groups, ["[index]", "{x}"])
        // Past the pattern: no active group, the help still names the command.
        let extra = SignatureHelp.info(in: "\\section{a}{", caretUTF16: 12)
        XCTAssertEqual(extra?.activeArgument, 1)
        XCTAssertNil(extra?.display.active)
        // Only CommandDocs knows it (the compiler does not render \chapter):
        // the pattern is empty, the doc line shows.
        let doc = SignatureHelp.info(in: "\\chapter{", caretUTF16: 9)
        XCTAssertNil(Completion.Vocabulary.byName["chapter"])
        XCTAssertEqual(doc?.arguments, "")
        XCTAssertEqual(doc?.description, "\\chapter{title}: a chapter heading (report and book classes).")
        // In the vocabulary and CommandDocs: the compiler's shape, CommandDocs' line.
        let foot = SignatureHelp.info(in: "\\footnote{", caretUTF16: 10)
        XCTAssertEqual(foot?.arguments, "[n]{...}")
        XCTAssertEqual(foot?.description, "\\footnote{text}: a numbered footnote.")

        // Nothing: outside braces, after the closing brace, escaped braces,
        // a comment, an unknown command, a brace with no command, a bad caret.
        XCTAssertNil(SignatureHelp.info(in: "\\frac{a}{b}", caretUTF16: 11))
        XCTAssertNil(SignatureHelp.info(in: "plain text", caretUTF16: 5))
        XCTAssertNil(SignatureHelp.info(in: "\\{ x", caretUTF16: 4))
        XCTAssertNil(SignatureHelp.info(in: "% \\frac{", caretUTF16: 8))
        XCTAssertNil(SignatureHelp.info(in: "\\zzzq{", caretUTF16: 6))
        XCTAssertNil(SignatureHelp.info(in: "a {b", caretUTF16: 4))
        XCTAssertNil(SignatureHelp.info(in: "\\frac{", caretUTF16: 99))
        XCTAssertNil(SignatureHelp.info(in: "\\frac{", caretUTF16: -1))
        // Bounded to the caret's line.
        XCTAssertNil(SignatureHelp.info(in: "\\frac{\nx", caretUTF16: 8))
    }

    @MainActor
    private func key(_ tv: NSTextView, _ chars: String, code: UInt16, flags: NSEvent.ModifierFlags = []) {
        let e = NSEvent.keyEvent(with: .keyDown, location: .zero, modifierFlags: flags, timestamp: ProcessInfo.processInfo.systemUptime,
                                 windowNumber: tv.window?.windowNumber ?? 0, context: nil, characters: chars,
                                 charactersIgnoringModifiers: chars, isARepeat: false, keyCode: code)!
        tv.keyDown(with: e)
    }

    @MainActor
    func testPanelOpensOnBraceFollowsTheArgumentAndCloses() throws {
        HostedWindowSupport.prepare() // non-activating: hosted windows must never pull the app forward
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 400), styleMask: [.titled], backing: .buffered, defer: false)
        let scroll = CompletingTextView.scrollable()
        scroll.frame = window.contentView!.bounds
        window.contentView!.addSubview(scroll)
        let tv = try XCTUnwrap(scroll.documentView as? CompletingTextView)
        window.orderFrontRegardless()
        window.makeFirstResponder(tv)
        defer { window.orderOut(nil) }
        tv.string = "$"
        tv.setSelectedRange(NSRange(location: 1, length: 0))
        for ch in "\\frac" { key(tv, String(ch), code: 0) }
        XCTAssertFalse(tv.isSignatureHelpVisible)
        key(tv, "{", code: 0)
        XCTAssertTrue(tv.isSignatureHelpVisible, "`{` after a command opens the help")
        let panel = tv.signatureHelpPanel
        XCTAssertEqual(panel.info?.command, "frac")
        XCTAssertEqual(panel.info?.activeArgument, 0)
        XCTAssertEqual(panel.patternText, "\\frac{num}{den}")
        XCTAssertEqual(panel.docText, "\\frac{num}{den}: a fraction.")
        XCTAssertTrue(panel.isVisible)
        XCTAssertTrue(panel.parent === window)
        for ch in "a}{" { key(tv, String(ch), code: 0) }
        XCTAssertEqual(panel.info?.activeArgument, 1, "the second brace moves the highlight")
        XCTAssertEqual(SignatureHelpPanel.attributed(try XCTUnwrap(panel.info)).string, "\\frac{num}{den}")
        key(tv, "b", code: 0)
        XCTAssertTrue(tv.isSignatureHelpVisible)
        key(tv, "}", code: 0)
        XCTAssertFalse(tv.isSignatureHelpVisible, "the closing brace leaves the argument")
        XCTAssertFalse(panel.isVisible)
        XCTAssertEqual(tv.string, "$\\frac{a}{b}")

        // ⌘⇧Space inside an argument opens it on demand; Esc closes it without opening the completion list.
        tv.setSelectedRange(NSRange(location: 7, length: 0))
        key(tv, " ", code: 49, flags: [.command, .shift])
        XCTAssertTrue(tv.isSignatureHelpVisible)
        XCTAssertEqual(panel.info?.activeArgument, 0)
        key(tv, "\u{1B}", code: 53)
        XCTAssertFalse(tv.isSignatureHelpVisible)
        XCTAssertNil(tv.session)
        // The caret leaving the argument closes it; moving within it keeps it.
        key(tv, " ", code: 49, flags: [.command, .shift])
        XCTAssertTrue(tv.isSignatureHelpVisible)
        tv.setSelectedRange(NSRange(location: 8, length: 0))
        XCTAssertTrue(tv.isSignatureHelpVisible, "still inside the first argument")
        tv.setSelectedRange(NSRange(location: 0, length: 0))
        XCTAssertFalse(tv.isSignatureHelpVisible)
        // ⌘⇧Space outside any argument shows nothing.
        key(tv, " ", code: 49, flags: [.command, .shift])
        XCTAssertFalse(tv.isSignatureHelpVisible)
        // A plain brace opens nothing.
        tv.string = "x "
        tv.setSelectedRange(NSRange(location: 2, length: 0))
        key(tv, "{", code: 0)
        XCTAssertFalse(tv.isSignatureHelpVisible)
    }
}
