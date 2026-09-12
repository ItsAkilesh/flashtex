import Foundation
import FlashTeXProtocol
import FlashTeXAccessibility

/// Turns a `compile_result`'s diagnostics into editor underline marks for one
/// document. Pure: no AppKit. A diagnostic's UTF-8 byte range is rebased across
/// edits made since the compile (`SourceMapping`, one `changedRegion` per call)
/// and dropped when it overlaps the edited region, so a mark is never drawn
/// under the wrong text. Dropped marks are reported as stale with a count the
/// UI can show; the diagnostics themselves stay listed.
///
/// Every mark carries the exact identity of the diagnostic it came from
/// (result id + index in `diagnostics` + original byte span), so the underline,
/// its tooltip and the diagnostics list row can agree on which diagnostic they
/// describe even after the range has been rebased. Diagnostics without a
/// `source` are not marks; the preview pane lists them.
enum EditorDiagnostics {
    /// Stable identity of one diagnostic in one compile result. The byte span
    /// is the one the worker reported (in the compiled text), never the rebased
    /// one, so the identity survives edits that shift the mark.
    struct Identity: Hashable {
        /// Envelope id of the `compile_result` (nil for fixtures with no id).
        let resultID: String?
        /// Index into `CompileResult.diagnostics`.
        let index: Int
        /// The worker's `source` range, as reported.
        let source: RuntimeV1.SourceRange

        /// "id#index@path:start..<end" — one token the list row and the
        /// underline can both compute.
        var key: String {
            "\(resultID ?? "-")#\(index)@\(source.path):\(source.startByte)..<\(source.endByte)"
        }

        // `SourceRange` is only Equatable in the protocol module.
        func hash(into hasher: inout Hasher) {
            hasher.combine(resultID); hasher.combine(index)
            hasher.combine(source.path); hasher.combine(source.startByte); hasher.combine(source.endByte)
        }
    }

    struct Mark: Equatable, Identifiable {
        let identity: Identity
        /// UTF-16 range in the *current* editor text (rebased; snapped outward
        /// to grapheme cluster boundaries so an underline never splits one).
        let nsRange: NSRange
        let severity: RuntimeV1.Severity
        let message: String
        let recovery: String?
        /// Status of the result the diagnostic belongs to. A `recovered`
        /// result rendered provisionally, so its marks always show a recovery
        /// line — and stay errors; recovery never downgrades or hides one.
        let resultStatus: RuntimeV1.Status

        var id: String { identity.key }
        var diagnosticIndex: Int { identity.index }
        /// The original byte span (for the list row's "bytes a..<b" text).
        var originalSource: RuntimeV1.SourceRange { identity.source }

        /// The recovery line shared by the tooltip, the list row and VoiceOver:
        /// the worker's note, or "no provisional rendering" when the result is
        /// `recovered` but this diagnostic describes none. Nil for `ok`/`failed`
        /// results whose diagnostic has no recovery note.
        var recoveryLine: String? { EditorDiagnostics.recoveryLine(recovery: recovery, status: resultStatus) }

        /// Tooltip text: message plus the recovery line when present.
        var toolTip: String { message + (recoveryLine.map { "\n↳ " + $0 } ?? "") }

        /// "Error: message — recovery: …" as the accessibility layer speaks it.
        var spokenDescription: String {
            (severity == .error ? "Error: " : "Warning: ") + message + (recoveryLine.map { " — " + $0 } ?? "")
        }
    }

    /// A diagnostic whose span overlaps the edit made since the compile. Its
    /// underline is withheld (never stretched over new text) until the next
    /// result; the identity lets the list row show it as stale.
    struct Stale: Equatable {
        let identity: Identity
        let severity: RuntimeV1.Severity
        let message: String
    }

    /// One call's outcome: the drawable marks plus what was withheld.
    struct Report: Equatable {
        var marks: [Mark]
        var stale: [Stale]
        /// The edit the marks were rebased across (nil: compiled text unknown
        /// or unchanged).
        var edit: SourceMapping.ChangedRegion?

        static let empty = Report(marks: [], stale: [], edit: nil)

        var staleCount: Int { stale.count }
        var staleIdentities: Set<Identity> { Set(stale.map(\.identity)) }

        /// Footer/list caption, nil when nothing is withheld.
        var staleNote: String? {
            guard !stale.isEmpty else { return nil }
            let errors = stale.filter { $0.severity == .error }.count
            let what: String
            switch (errors, stale.count - errors) {
            case (0, let w): what = "\(w) warning\(w == 1 ? "" : "s")"
            case (let e, 0): what = "\(e) error\(e == 1 ? "" : "s")"
            case (let e, let w): what = "\(e) error\(e == 1 ? "" : "s") and \(w) warning\(w == 1 ? "" : "s")"
            }
            return "\(what) under edited text not underlined until the next compile"
        }
    }

    /// Recovery line for a diagnostic of a result with `status` (see `Mark.recoveryLine`).
    static func recoveryLine(recovery: String?, status: RuntimeV1.Status) -> String? {
        if let recovery { return "recovery: " + recovery }
        return status == .recovered ? "no provisional rendering" : nil
    }

    /// Identity of `result.diagnostics[index]`, nil when it has no source (the
    /// list row for such a diagnostic has no mark to agree with).
    static func identity(resultID: String?, index: Int, in result: RuntimeV1.CompileResult) -> Identity? {
        guard result.diagnostics.indices.contains(index), let source = result.diagnostics[index].source else { return nil }
        return Identity(resultID: resultID, index: index, source: source)
    }

    /// Marks only (see `report` for the stale set).
    /// - Parameters:
    ///   - resultID: envelope id of `result`, part of every mark's identity.
    ///   - path: the document shown in the editor.
    ///   - compiledText: the document text `result` was produced for (nil if unknown).
    ///   - currentText: the current editor buffer; marks are `NSRange`s into it.
    static func marks(for result: RuntimeV1.CompileResult, resultID: String? = nil, path: String,
                      compiledText: String?, currentText: String) -> [Mark] {
        report(for: result, resultID: resultID, path: path, compiledText: compiledText, currentText: currentText).marks
    }

    /// Marks plus the diagnostics withheld as stale. Cost is one byte diff of
    /// the two texts plus O(diagnostics) offset conversions; it never waits on
    /// a compile, so a keystroke can call it synchronously (see
    /// `EditorDiagnosticsTests.testRebaseOfLargeDocumentIsFast`).
    static func report(for result: RuntimeV1.CompileResult, resultID: String? = nil, path: String,
                       compiledText: String?, currentText: String) -> Report {
        let region: SourceMapping.ChangedRegion? = {
            guard let compiledText, !compiledText.sameBytes(as: currentText) else { return nil }
            return SourceMapping.changedRegion(from: compiledText, to: currentText)
        }()
        var marks: [Mark] = []
        var stale: [Stale] = []
        marks.reserveCapacity(result.diagnostics.count)
        for (index, diagnostic) in result.diagnostics.enumerated() {
            guard let source = diagnostic.source, source.path == path else { continue }
            let identity = Identity(resultID: resultID, index: index, source: source)
            var start = source.startByte, end = source.endByte
            if let region {
                guard case .rebased(let s, let e) = SourceMapping.rebase(start: start, end: end, across: region) else {
                    stale.append(Stale(identity: identity, severity: diagnostic.severity, message: diagnostic.message))
                    continue
                }
                start = s; end = e
            }
            // Offsets inside a multi-byte scalar, reversed or out of range are
            // refused (nil): a byte span that is not a valid slice of the
            // current buffer is not drawn anywhere.
            guard let ns = currentText.clusterAlignedNSRange(utf8Start: start, utf8End: end) else { continue }
            marks.append(Mark(identity: identity, nsRange: ns, severity: diagnostic.severity,
                              message: diagnostic.message, recovery: diagnostic.recovery, resultStatus: result.status))
        }
        return Report(marks: marks, stale: stale, edit: region)
    }

    // MARK: keyboard navigation (document order, wrapping, "n of m")

    /// The marks in document order as the accessibility navigator sees them.
    static func navigationItems(_ marks: [Mark]) -> [EditorDiagnosticNavigation.Item] {
        EditorDiagnosticNavigation.ordered(marks.map { m in
            EditorDiagnosticNavigation.Item(id: m.id, nsRange: m.nsRange, severity: m.severity,
                                            message: m.message, recoveryLine: m.recoveryLine)
        })
    }

    /// Next (or previous) diagnostic from the caret, wrapping around the
    /// document; `currentID` is the mark last navigated to, so two marks at
    /// the same offset are both reachable. Nil when there are no marks.
    static func step(_ marks: [Mark], fromUTF16 caret: Int, forward: Bool, currentID: String? = nil,
                     in text: String? = nil) -> EditorDiagnosticNavigation.Step? {
        let model = text.map { AccessibleEditorModel(text: $0) }
        return EditorDiagnosticNavigation.step(navigationItems(marks), fromUTF16: caret, forward: forward,
                                               currentID: currentID, lineOf: { model?.line(containingUTF16: $0)?.number })
    }
}

extension String {
    /// UTF-16 range for a UTF-8 byte span, widened outward to the grapheme
    /// clusters containing its ends so the range never splits one (a combining
    /// mark typed after the last marked character joins the underline instead
    /// of being cut in half). Nil when the bytes are not a valid scalar-aligned
    /// slice of the string.
    func clusterAlignedNSRange(utf8Start start: Int, utf8End end: Int) -> NSRange? {
        guard var r = rangeOfUTF8(start: start, end: end) else { return nil }
        if String.Index(r.lowerBound, within: self) == nil {
            // `index(after:)` rounds an unaligned index down to its cluster
            // first, so this is the start of the cluster containing lowerBound.
            r = index(before: index(after: r.lowerBound))..<r.upperBound
        }
        if r.upperBound < endIndex, String.Index(r.upperBound, within: self) == nil {
            r = r.lowerBound..<index(after: r.upperBound)
        }
        return NSRange(r, in: self)
    }
}
