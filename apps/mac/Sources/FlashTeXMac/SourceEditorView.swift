import AppKit
import SwiftUI
import FlashTeXProtocol

/// NSTextView wrapper. Uses a monospaced font, reports edits, applies UTF-16
/// selections requested by preview navigation, and underlines diagnostic marks
/// with layout-manager temporary attributes (never touching the text storage,
/// so undo and the `text` binding are unaffected).
struct SourceEditorView: NSViewRepresentable {
    @Binding var text: String
    var selection: ShellModel.Selection?
    var pendingEdit: ShellModel.PendingEdit?
    var marks: [EditorDiagnostics.Mark] = []
    var result: RuntimeV1.CompileResult? // for completion (Completion.swift)
    var onCaretChange: (Int) -> Void = { _ in }
    var onSelectionChange: (NSRange) -> Void = { _ in }
    var onEditApplied: (ShellModel.PendingEdit, String) -> Void = { _, _ in }

    func makeCoordinator() -> Coordinator { Coordinator(self) }

    func makeNSView(context: Context) -> NSScrollView {
        let scroll = CompletingTextView.scrollable() // Completion.swift
        let tv = scroll.documentView as! NSTextView
        tv.delegate = context.coordinator
        tv.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        tv.isRichText = false
        tv.isAutomaticQuoteSubstitutionEnabled = false
        tv.isAutomaticDashSubstitutionEnabled = false
        tv.isAutomaticTextReplacementEnabled = false
        tv.allowsUndo = true
        tv.textContainerInset = NSSize(width: 8, height: 8)
        tv.setAccessibilityLabel("LaTeX source") // FlashTeXAccessibility: VoiceOver names the editor
        tv.string = text
        return scroll
    }

    func updateNSView(_ scroll: NSScrollView, context: Context) {
        let tv = scroll.documentView as! NSTextView
        context.coordinator.parent = self
        (tv as? CompletingTextView)?.compileResult = result
        if let edit = pendingEdit, edit.token != context.coordinator.appliedEditToken {
            context.coordinator.appliedEditToken = edit.token
            let ns = edit.nsRange
            if NSMaxRange(ns) <= (tv.string as NSString).length,
               tv.shouldChangeText(in: ns, replacementString: edit.text) {
                tv.textStorage?.replaceCharacters(in: ns, with: edit.text)
                tv.didChangeText() // registers undo, fires textDidChange
                tv.undoManager?.setActionName("Insert Capture")
                let inserted = NSRange(location: ns.location, length: (edit.text as NSString).length)
                tv.setSelectedRange(inserted)
                tv.scrollRangeToVisible(inserted)
                tv.showFindIndicator(for: inserted)
                tv.window?.makeFirstResponder(tv)
            }
            context.coordinator.lastKnownText = tv.string
            DispatchQueue.main.async { onEditApplied(edit, tv.string) }
            return
        }
        // `tv.string` bridges a fresh copy and compares it character by character
        // (Unicode-normalized) on every update. The coordinator keeps the exact
        // String instance last exchanged with the text view; when the binding
        // still holds that instance the comparison is a pointer check.
        var textReset = false
        if text != context.coordinator.lastKnownText {
            tv.string = text // drops temporary attributes; reapply marks below
            context.coordinator.lastKnownText = text
            textReset = true
        }
        if textReset || marks != context.coordinator.lastMarks {
            context.coordinator.lastMarks = marks
            Self.applyMarks(marks, to: tv)
        }
        if let selection, selection.token != context.coordinator.appliedToken {
            context.coordinator.appliedToken = selection.token
            let range = selection.nsRange
            if NSMaxRange(range) <= (tv.string as NSString).length {
                tv.setSelectedRange(range)
                tv.scrollRangeToVisible(range)
                tv.showFindIndicator(for: range)
                tv.window?.makeFirstResponder(tv)
            }
        }
    }

    /// Replaces underline/tooltip temporary attributes over the whole text.
    /// Ranges outside the current string are skipped; errors are applied after
    /// warnings so an error wins where they overlap.
    static func applyMarks(_ marks: [EditorDiagnostics.Mark], to tv: NSTextView) {
        guard let lm = tv.layoutManager else { return }
        let length = (tv.string as NSString).length
        let whole = NSRange(location: 0, length: length)
        for key in [NSAttributedString.Key.underlineStyle, .underlineColor, .toolTip] {
            lm.removeTemporaryAttribute(key, forCharacterRange: whole)
        }
        let ordered = marks.filter { $0.severity == .warning } + marks.filter { $0.severity == .error }
        for mark in ordered {
            let r = mark.nsRange
            guard r.location >= 0, r.length > 0, NSMaxRange(r) <= length else { continue }
            let color: NSColor = mark.severity == .error ? .systemRed : .systemOrange
            lm.addTemporaryAttributes([
                .underlineStyle: NSUnderlineStyle.thick.rawValue | NSUnderlineStyle.patternDot.rawValue,
                .underlineColor: color,
                .toolTip: mark.toolTip,
            ], forCharacterRange: r)
        }
    }

    final class Coordinator: NSObject, NSTextViewDelegate {
        var parent: SourceEditorView
        var appliedToken = 0
        var appliedEditToken = 0
        var lastMarks: [EditorDiagnostics.Mark] = []
        /// The String instance last set on, or read from, the text view.
        var lastKnownText: String
        init(_ parent: SourceEditorView) { self.parent = parent; lastKnownText = parent.text }

        func textDidChange(_ notification: Notification) {
            guard let tv = notification.object as? NSTextView else { return }
            TypingBench.shared.textViewDidChange() // stamps the delegate time for keystroke -> paint
            let s = tv.string
            lastKnownText = s
            parent.text = s
        }

        func textViewDidChangeSelection(_ notification: Notification) {
            guard let tv = notification.object as? NSTextView else { return }
            parent.onCaretChange(tv.selectedRange().location)
            parent.onSelectionChange(tv.selectedRange())
        }
    }
}
