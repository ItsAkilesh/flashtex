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
        /// One bounded line from the explanation catalogue
        /// (`EditorDiagnostics.Explanation.line`), attached by `attach(_:to:)`
        /// once the helper has answered for this result; nil until then.
        var explanation: String? = nil

        var id: String { identity.key }
        var diagnosticIndex: Int { identity.index }
        /// The original byte span (for the list row's "bytes a..<b" text).
        var originalSource: RuntimeV1.SourceRange { identity.source }

        /// The recovery line shared by the tooltip, the list row and VoiceOver:
        /// the worker's note, or "no provisional rendering" when the result is
        /// `recovered` but this diagnostic describes none. Nil for `ok`/`failed`
        /// results whose diagnostic has no recovery note.
        var recoveryLine: String? { EditorDiagnostics.recoveryLine(recovery: recovery, status: resultStatus) }

        /// Tooltip text: message, then the recovery line and the explanation
        /// line when present.
        var toolTip: String {
            message + (recoveryLine.map { "\n↳ " + $0 } ?? "") + (explanation.map { "\n↳ " + $0 } ?? "")
        }

        /// "Error: message — recovery: … — explanation" as the accessibility layer speaks it.
        var spokenDescription: String {
            (severity == .error ? "Error: " : "Warning: ") + message
                + (recoveryLine.map { " — " + $0 } ?? "") + (explanation.map { " — " + $0 } ?? "")
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
                                            message: m.message, recoveryLine: m.recoveryLine, explanation: m.explanation)
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

// MARK: - Explanations (crates/diagnostic-explanations)

/// Consumer of `flashtex-diagnostic-explanations`, the offline, deterministic
/// explanation catalogue for compiler diagnostics (no provider, no network).
/// The crate is a Rust library; the app talks to it through the JSON Lines
/// helper `flashtex-explain` (request/reply shape below, one line each).
/// Replies are bounded in bytes, count and text length, decoded once, cached
/// per result id, and attached to marks as one extra line; a fetch never runs
/// on the caller's thread, so typing is never blocked by an explanation.
extension EditorDiagnostics {
    /// One explanation, decoded from the crate's fixed-key JSON
    /// (`catalog_id, title, category, severity, message, why, what_happened,
    /// suggestions, context`). Text fields are truncated to `ExplanationLimits`.
    struct Explanation: Equatable, Decodable {
        struct Edit: Equatable, Decodable {
            var path: String
            var startByte: Int
            var endByte: Int
            var replacement: String
            enum CodingKeys: String, CodingKey { case path, startByte = "start_byte", endByte = "end_byte", replacement }
        }
        struct Suggestion: Equatable, Decodable {
            var text: String
            /// "low" | "medium" | "high" (unknown values are kept verbatim).
            var confidence: String
            /// Candidate edits; nothing is ever applied by this consumer.
            var edits: [Edit]
        }
        struct Context: Equatable, Decodable {
            var path: String
            var startByte: Int
            var endByte: Int
            var text: String
            var spanStart: Int
            var spanEnd: Int
            var line: Int
            var column: Int
            var spanInBounds: Bool
            enum CodingKeys: String, CodingKey {
                case path, startByte = "start_byte", endByte = "end_byte", text, spanStart = "span_start", spanEnd = "span_end"
                case line, column, spanInBounds = "span_in_bounds"
            }
        }

        /// Catalogue entry id, nil when the message fell back to `unknown`.
        var catalogID: String?
        var title: String
        var category: String
        var severity: String
        var message: String
        var why: String
        var whatHappened: String
        var suggestions: [Suggestion]
        var context: Context?

        enum CodingKeys: String, CodingKey {
            case catalogID = "catalog_id", title, category, severity, message, why, whatHappened = "what_happened", suggestions, context
        }

        var isCatalogued: Bool { catalogID != nil }

        /// The one line shown under the mark, in the list row and spoken:
        /// "explain: <title> — <why>" for catalogued entries, "explain: <title>
        /// (not in the catalogue)" otherwise. Bounded to
        /// `ExplanationLimits.maxLineCharacters`.
        var line: String {
            let body = isCatalogued ? "\(title) — \(why)" : "\(title) (not in the catalogue)"
            return ExplanationLimits.truncate("explain: " + body, to: ExplanationLimits.maxLineCharacters)
        }
    }

    /// Bounds applied to every reply before it is cached. A reply outside
    /// `maxReplyBytes` / `maxCount` is refused whole; text fields are cut.
    enum ExplanationLimits {
        static let maxReplyBytes = 4 * 1024 * 1024
        static let maxCount = 2000
        static let maxLineCharacters = 240
        static let maxTitleCharacters = 120
        static let maxParagraphCharacters = 600
        static let maxSuggestions = 4
        static let maxEditsPerSuggestion = 8
        static let maxContextCharacters = 2000
        /// Results kept in an `ExplanationCache`.
        static let maxCachedResults = 8

        static func truncate(_ s: String, to limit: Int) -> String {
            guard s.count > limit else { return s }
            return String(s.prefix(max(0, limit - 1))) + "…"
        }

        static func bounded(_ x: Explanation) -> Explanation {
            var x = x
            x.title = truncate(x.title, to: maxTitleCharacters)
            x.why = truncate(x.why, to: maxParagraphCharacters)
            x.whatHappened = truncate(x.whatHappened, to: maxParagraphCharacters)
            x.message = truncate(x.message, to: maxParagraphCharacters)
            x.suggestions = x.suggestions.prefix(maxSuggestions).map { s in
                var s = s
                s.text = truncate(s.text, to: maxParagraphCharacters)
                s.edits = s.edits.prefix(maxEditsPerSuggestion).map { e in
                    var e = e
                    e.replacement = truncate(e.replacement, to: maxContextCharacters)
                    return e
                }
                return s
            }
            if var c = x.context { c.text = truncate(c.text, to: maxContextCharacters); x.context = c }
            return x
        }
    }

    enum ExplanationFailure: Error, Equatable {
        /// The reply line is larger than `ExplanationLimits.maxReplyBytes`.
        case oversized(bytes: Int)
        /// The reply is not the documented shape.
        case malformed(String)
        /// The helper answered for a different number of diagnostics.
        case countMismatch(expected: Int, actual: Int)
        /// The helper answered `type: error`.
        case helper(code: String, message: String)
        /// Process/transport failure (`LineProcessFailure.text`).
        case transport(String)
        /// No helper executable is configured or found.
        case unavailable

        var text: String {
            switch self {
            case .oversized(let n): "explanation reply of \(n) bytes exceeds \(ExplanationLimits.maxReplyBytes)"
            case .malformed(let s): "malformed explanation reply: \(s)"
            case .countMismatch(let e, let a): "explanation reply has \(a) entries for \(e) diagnostics"
            case .helper(let c, let m): "flashtex-explain error \(c): \(m)"
            case .transport(let s): "flashtex-explain: \(s)"
            case .unavailable: "flashtex-explain is not available"
            }
        }
    }

    /// Decodes the `explanations` array of one reply line (or a bare array)
    /// and applies the bounds. `expectedCount` is the result's diagnostic
    /// count: a reply for a different number is refused, since entries are
    /// matched to diagnostics by index. `maxBytes` defaults to the limit.
    static func decodeExplanations(_ data: Data, expectedCount: Int,
                                   maxBytes: Int = ExplanationLimits.maxReplyBytes) throws -> [Explanation] {
        guard data.count <= maxBytes else { throw ExplanationFailure.oversized(bytes: data.count) }
        struct Reply: Decodable { var explanations: [Explanation] }
        let decoded: [Explanation]
        do {
            let first = data.first { $0 != 0x20 && $0 != 0x0A && $0 != 0x0D && $0 != 0x09 }
            if first == UInt8(ascii: "[") {
                decoded = try JSONDecoder().decode([Explanation].self, from: data)
            } else {
                decoded = try JSONDecoder().decode(Reply.self, from: data).explanations
            }
        } catch {
            throw ExplanationFailure.malformed(Self.describe(error))
        }
        guard decoded.count == expectedCount, decoded.count <= ExplanationLimits.maxCount else {
            throw ExplanationFailure.countMismatch(expected: expectedCount, actual: decoded.count)
        }
        return decoded.map(ExplanationLimits.bounded)
    }

    private static func describe(_ error: Error) -> String {
        guard let d = error as? DecodingError else { return String(describing: error) }
        func path(_ c: DecodingError.Context) -> String { c.codingPath.map(\.stringValue).joined(separator: ".") }
        switch d {
        case .keyNotFound(let k, let c): return "missing key '\(k.stringValue)' at '\(path(c))'"
        case .typeMismatch(let t, let c): return "type mismatch (\(t)) at '\(path(c))'"
        case .valueNotFound(let t, let c): return "null for \(t) at '\(path(c))'"
        case .dataCorrupted(let c): return "not JSON: \(c.debugDescription)"
        @unknown default: return String(describing: d)
        }
    }

    /// Explanations kept per result id, bounded to
    /// `ExplanationLimits.maxCachedResults` results (oldest evicted). The
    /// diagnostics of a result never change, so an entry is never refreshed.
    struct ExplanationCache: Equatable {
        private(set) var entries: [String: [Explanation]] = [:]
        private var order: [String] = []
        let maxResults: Int

        init(maxResults: Int = ExplanationLimits.maxCachedResults) { self.maxResults = maxResults }

        mutating func store(_ explanations: [Explanation], for resultID: String) {
            if entries[resultID] == nil {
                order.append(resultID)
                while order.count > maxResults, let oldest = order.first {
                    order.removeFirst(); entries[oldest] = nil
                }
            }
            entries[resultID] = explanations
        }

        subscript(resultID: String?) -> [Explanation]? {
            guard let resultID else { return nil }
            return entries[resultID]
        }

        func explanation(resultID: String?, index: Int) -> Explanation? {
            guard let list = self[resultID], list.indices.contains(index) else { return nil }
            return list[index]
        }

        var count: Int { entries.count }
    }

    /// `report` with each mark's `explanation` line set from `explanations`
    /// (matched by diagnostic index; nil leaves the marks untouched). O(marks):
    /// this is all a keystroke pays once the helper has answered.
    static func attach(_ explanations: [Explanation]?, to report: Report) -> Report {
        guard let explanations, !explanations.isEmpty else { return report }
        var out = report
        for i in out.marks.indices {
            let index = out.marks[i].diagnosticIndex
            out.marks[i].explanation = explanations.indices.contains(index) ? explanations[index].line : nil
        }
        return out
    }
}

/// The `flashtex-explain` helper: one request line
/// `{"id","type":"explain","compile_result":<payload>,"documents":[{path,text}],"supported":[…]}`,
/// one reply line `{"id","type":"explanations","explanations":[…]}` or
/// `{"id","type":"error","error":{code,message}}`. Runs on private pipes via
/// `LineProcessClient`; completions are delivered on `queue` (main by default).
final class ExplanationClient {
    typealias Explanation = EditorDiagnostics.Explanation
    typealias Failure = EditorDiagnostics.ExplanationFailure

    static let label = "explain"
    /// Default deadline for one reply; the crate answers in milliseconds.
    static let defaultTimeout: TimeInterval = 10

    let executable: URL
    let maxReplyBytes: Int
    private let client: LineProcessClient
    private let queue: DispatchQueue

    /// `FLASHTEX_EXPLAIN`, then `flashtex-explain` beside the app executable,
    /// then the repository's `crates/diagnostic-explanations/target/{release,debug}`.
    @MainActor static func locate() -> URL? {
        let fm = FileManager.default
        if let env = ProcessInfo.processInfo.environment["FLASHTEX_EXPLAIN"], fm.isExecutableFile(atPath: env) {
            return URL(fileURLWithPath: env)
        }
        if let bundled = Bundle.main.executableURL?.deletingLastPathComponent().appendingPathComponent("flashtex-explain"),
           fm.isExecutableFile(atPath: bundled.path) {
            return bundled
        }
        guard let root = ShellModel.locateRepoRoot() else { return nil }
        for profile in ["release", "debug"] {
            let url = root.appendingPathComponent("crates/diagnostic-explanations/target/\(profile)/flashtex-explain")
            if fm.isExecutableFile(atPath: url.path) { return url }
        }
        return nil
    }

    init(executable: URL, arguments: [String] = [], queue: DispatchQueue = .main,
         maxReplyBytes: Int = EditorDiagnostics.ExplanationLimits.maxReplyBytes,
         events: @escaping (String) -> Void = { _ in }) throws {
        self.executable = executable
        self.queue = queue
        self.maxReplyBytes = maxReplyBytes
        struct Header: Decodable { var id: String?; var type: String; var error: TransferV1.ErrorPayload? }
        client = try LineProcessClient(
            executable: executable, arguments: arguments, label: Self.label, queue: queue,
            classify: { line in
                guard let h = try? JSONDecoder().decode(Header.self, from: line) else { return nil }
                return .init(id: h.id, type: h.type, error: h.error)
            },
            events: { event in
                let text: String
                switch event {
                case .stderr(let s): text = "stderr: " + s.trimmingCharacters(in: .whitespacesAndNewlines)
                case .protocolViolation(let s): text = s
                case .unsolicited(let id, let type, _): text = "unsolicited \(type) reply for \(id ?? "?")"
                case .exited(let code): text = "exited (\(code))"
                }
                events(text)
            })
    }

    var isRunning: Bool { client.isRunning }
    func terminate() { client.terminate() }

    private struct Request: Encodable {
        var id: String
        var type = "explain"
        var compileResult: RuntimeV1.CompileResult
        var documents: [RuntimeV1.Document]
        var supported: [String]
        enum CodingKeys: String, CodingKey { case id, type, compileResult = "compile_result", documents, supported }
    }

    /// Asks for explanations of every diagnostic of `result` (compiled from
    /// `documents`). The reply is decoded and bounded, then delivered on
    /// `queue`; the caller stores it in an `ExplanationCache` under the
    /// result's id. Nothing here blocks the caller beyond encoding the request.
    func explain(result: RuntimeV1.CompileResult, documents: [RuntimeV1.Document], supported: [String],
                 timeout: TimeInterval? = ExplanationClient.defaultTimeout,
                 completion: @escaping (Result<[Explanation], Failure>) -> Void) {
        let id = client.makeID()
        let line: Data
        do {
            let enc = JSONEncoder()
            enc.outputFormatting = [.withoutEscapingSlashes]
            var data = try enc.encode(Request(id: id, compileResult: result, documents: documents, supported: supported))
            data.append(0x0A)
            line = data
        } catch {
            queue.async { completion(.failure(.transport("request encoding failed: \(error)"))) }
            return
        }
        let expected = result.diagnostics.count
        let maxBytes = maxReplyBytes
        client.enqueue(id: id, line: line, expected: "explanations", timeout: timeout) { outcome in
            switch outcome {
            case .failure(.bridge(let e)):
                completion(.failure(.helper(code: e.code, message: e.message)))
            case .failure(let f):
                completion(.failure(.transport(f.text)))
            case .success(let reply):
                do {
                    completion(.success(try EditorDiagnostics.decodeExplanations(reply, expectedCount: expected, maxBytes: maxBytes)))
                } catch let f as Failure {
                    completion(.failure(f))
                } catch {
                    completion(.failure(.malformed(String(describing: error))))
                }
            }
        }
    }
}
