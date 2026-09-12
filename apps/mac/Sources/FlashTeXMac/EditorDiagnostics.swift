import Foundation
import FlashTeXProtocol

/// Turns a `compile_result`'s diagnostics into editor underline marks for one
/// document. Pure: no AppKit. A diagnostic's UTF-8 byte range is rebased across
/// edits made since the compile (`SourceMapping`) and dropped when it overlaps
/// the edited region, so a mark is never drawn under the wrong text.
/// Diagnostics without a `source` are not marks; the preview pane lists them.
enum EditorDiagnostics {
    struct Mark: Equatable {
        let nsRange: NSRange
        let severity: RuntimeV1.Severity
        let message: String
        let recovery: String?

        /// Tooltip text: message plus the recovery line when present.
        var toolTip: String { message + (recovery.map { "\n↳ " + $0 } ?? "") }
    }

    /// - Parameters:
    ///   - path: the document shown in the editor.
    ///   - compiledText: the document text `result` was produced for (nil if unknown).
    ///   - currentText: the current editor buffer; marks are `NSRange`s into it.
    static func marks(for result: RuntimeV1.CompileResult, path: String,
                      compiledText: String?, currentText: String) -> [Mark] {
        // One byte diff of compiled vs current text for every diagnostic.
        let region: SourceMapping.ChangedRegion? = {
            guard let compiledText, !compiledText.sameBytes(as: currentText) else { return nil }
            return SourceMapping.changedRegion(from: compiledText, to: currentText)
        }()
        return result.diagnostics.compactMap { diagnostic in
            guard let source = diagnostic.source, source.path == path else { return nil }
            var range = source
            if let region {
                guard case .rebased(let s, let e) = SourceMapping.rebase(start: source.startByte, end: source.endByte, across: region)
                else { return nil }
                range = RuntimeV1.SourceRange(path: source.path, startByte: s, endByte: e)
            }
            guard let ns = currentText.nsRange(utf8Bytes: range) else { return nil }
            return Mark(nsRange: ns, severity: diagnostic.severity,
                        message: diagnostic.message, recovery: diagnostic.recovery)
        }
    }
}
