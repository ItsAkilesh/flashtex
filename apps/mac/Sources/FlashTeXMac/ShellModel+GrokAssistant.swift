import AppKit
import Foundation
import FlashTeXProtocol

/// Ask Grok on the live document (GrokAssistant.swift): how the shell opens
/// the panel, what it hands the assistant, and how an approved edit reaches
/// the editor. The only stored state is `ShellModel.grokAssistant`.
extension ShellModel {
    /// Edit › Ask Grok… (⌘⌥G), the toolbar button and the palette: opens the
    /// panel over the current selection. `prefill` replaces the instruction;
    /// `diagnosticIndex` pins that diagnostic into the request.
    func askGrok(prefill: String? = nil, diagnosticIndex: Int? = nil) {
        if let prefill { grokAssistant.instruction = prefill }
        grokAssistant.pinnedDiagnostic = diagnosticIndex
        grokAssistant.refreshConfiguration()
        grokAssistant.shown = true
    }

    /// "Fix with Grok" on a Problems row: selects the whole line(s) holding the
    /// diagnostic's span in the editor (a span is often one byte — the `$` or
    /// `}` the compiler stopped at — and the selection is the only region the
    /// model may edit; live check grok-assistant-20260912T221103Z-fix) and
    /// opens the panel pre-filled with "Fix this: <message>".
    func fixWithGrok(diagnosticIndex i: Int) {
        guard let result, result.diagnostics.indices.contains(i) else {
            navigationNote = "No diagnostic \(i + 1) in the current result."; return
        }
        let d = result.diagnostics[i]
        if let s = d.source, s.path == activePath, let lines = Self.lineSpan(covering: s.startByte..<max(s.startByte, s.endByte), in: activeText),
           let ns = activeText.nsRange(utf8Bytes: .init(path: s.path, startByte: lines.lowerBound, endByte: lines.upperBound)) {
            selection = .init(path: s.path, nsRange: ns, token: nextEditToken())
            caretUTF16 = ns.location
            caretLengthUTF16 = ns.length
        }
        let message = d.message.split(separator: "\n").first.map(String.init) ?? d.message
        askGrok(prefill: "Fix this: \(message.prefix(400))", diagnosticIndex: i)
    }

    /// The byte range of the whole line(s) containing `span` (LF-delimited,
    /// without the trailing newline); nil when `span` is outside `text`.
    static func lineSpan(covering span: Range<Int>, in text: String) -> Range<Int>? {
        let bytes = Array(text.utf8)
        guard span.lowerBound >= 0, span.upperBound <= bytes.count else { return nil }
        var lo = span.lowerBound, hi = span.upperBound
        while lo > 0, bytes[lo - 1] != 0x0A { lo -= 1 }
        if hi > lo, bytes[hi - 1] == 0x0A { hi -= 1 }
        while hi < bytes.count, bytes[hi] != 0x0A { hi += 1 }
        return lo..<hi
    }

    /// What one Ask binds to: the last compile (result id, compiled sources)
    /// and the live buffer with the editor selection in UTF-8 bytes. Nil
    /// without a compile result (the panel says to compile first).
    func grokSnapshot() -> GrokAssistant.Snapshot? {
        guard let result, let resultID else { return nil }
        let text = activeText
        var selectionBytes: Range<Int>?
        if caretLengthUTF16 > 0, let r = text.utf8ByteRange(of: NSRange(location: caretUTF16, length: caretLengthUTF16)) {
            selectionBytes = r.start..<r.end
        }
        return .init(resultID: resultID, result: result, compiledDocuments: compiledDocuments, activePath: activePath,
                     activeText: text, editorRevision: editorRevision, selection: selectionBytes, caretByte: caretByte)
    }

    /// The panel's Ask button.
    func askGrokNow() {
        guard let snapshot = grokSnapshot() else {
            grokAssistant.noteNoCompile("no compile result to bind to — attach a producer and compile (⌘B) first")
            return
        }
        // A buffer that moved on since the compile is refused by `makeRequest`
        // with that reason; nudge a compile so the next Ask binds to it.
        if previewIsStale, workerAttached { compile() }
        grokAssistant.ask(snapshot: snapshot)
    }

    /// The panel's Apply: helper approval, then ONE grouped edit through the
    /// same `pendingEdit` path quick fixes and capture insertions use (the
    /// editor applies it as one undoable step and reports back through
    /// `editApplied`). Never automatic.
    func applyGrokEdit() {
        grokAssistant.apply(activeText: activeText) { [weak self] grouped in
            guard let self else { return }
            guard grouped.path == self.activePath, grouped.matches(self.activeText) else {
                self.navigationNote = "Grok's edit not applied: the document changed since the preview; ask again."
                return
            }
            self.pendingEdit = .init(path: grouped.path, nsRange: grouped.nsRange, text: grouped.text,
                                     token: self.nextEditToken(), revision: self.editorRevision)
            self.navigationNote = "Applied Grok's edit (undo with ⌘Z)"
        }
    }
}

extension GrokAssistant {
    /// The shell has no compile result: say so without starting anything.
    func noteNoCompile(_ why: String) { setFailed(why) }
}
