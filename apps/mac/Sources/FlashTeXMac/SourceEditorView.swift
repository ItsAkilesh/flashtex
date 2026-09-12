import AppKit
import SwiftUI

/// NSTextView wrapper. Uses a monospaced font, reports edits, and applies
/// UTF-16 selections requested by preview navigation.
struct SourceEditorView: NSViewRepresentable {
    @Binding var text: String
    var selection: ShellModel.Selection?

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
        init(_ parent: SourceEditorView) { self.parent = parent }

        func textDidChange(_ notification: Notification) {
            guard let tv = notification.object as? NSTextView else { return }
            parent.text = tv.string
        }
    }
}
