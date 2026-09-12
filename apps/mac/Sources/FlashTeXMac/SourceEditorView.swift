import AppKit
import SwiftUI

/// NSTextView wrapper. Uses a monospaced font, reports edits, and applies
/// UTF-16 selections requested by preview navigation.
struct SourceEditorView: NSViewRepresentable {
    @Binding var text: String
    var selection: ShellModel.Selection?
    var pendingEdit: ShellModel.PendingEdit?
    var onCaretChange: (Int) -> Void = { _ in }
    var onEditApplied: (ShellModel.PendingEdit, String) -> Void = { _, _ in }

    func makeCoordinator() -> Coordinator { Coordinator(self) }

    func makeNSView(context: Context) -> NSScrollView {
        let scroll = NSTextView.scrollableTextView()
        let tv = scroll.documentView as! NSTextView
        tv.delegate = context.coordinator
        tv.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        tv.isRichText = false
        tv.isAutomaticQuoteSubstitutionEnabled = false
        tv.isAutomaticDashSubstitutionEnabled = false
        tv.isAutomaticTextReplacementEnabled = false
        tv.allowsUndo = true
        tv.textContainerInset = NSSize(width: 8, height: 8)
        tv.string = text
        return scroll
    }

    func updateNSView(_ scroll: NSScrollView, context: Context) {
        let tv = scroll.documentView as! NSTextView
        context.coordinator.parent = self
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
            DispatchQueue.main.async { onEditApplied(edit, tv.string) }
            return
        }
        if tv.string != text {
            tv.string = text
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

    final class Coordinator: NSObject, NSTextViewDelegate {
        var parent: SourceEditorView
        var appliedToken = 0
        var appliedEditToken = 0
        init(_ parent: SourceEditorView) { self.parent = parent }

        func textDidChange(_ notification: Notification) {
            guard let tv = notification.object as? NSTextView else { return }
            parent.text = tv.string
        }

        func textViewDidChangeSelection(_ notification: Notification) {
            guard let tv = notification.object as? NSTextView else { return }
            parent.onCaretChange(tv.selectedRange().location)
        }
    }
}
