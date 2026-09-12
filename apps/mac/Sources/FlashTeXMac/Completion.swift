import AppKit
import FlashTeXProtocol
import os

/// Source-aware completion for the editor. `suggestions` is pure (no AppKit)
/// and works on UTF-8 bytes with the caret given in UTF-16 units, so it is
/// safe on non-ASCII text and on caret positions that are out of range or
/// inside a surrogate pair (those yield no suggestions, never a trap).
///
/// Sources, in rank order:
/// 1. `\end{X}` for every `\begin{X}` before the caret that is still open.
/// 2. Commands the compiler supports (`supported`; default list below).
/// 3. Commands typed elsewhere in the document that are not in `supported`,
///    marked "not supported by this compiler version".
/// 4. Environment names (after `\begin{`/`\end{`), labels (after `\ref{`) and
///    citation keys (after `\cite{`) seen in the document.
/// 5. Words longer than 3 characters from the document, frequency-ranked.
///
/// Rust-produced vocabulary (`Metadata`) is consumed only when it is bound to
/// the revision of the text being completed: the runtime-v1 `compile_result`
/// (its `revision` plus the diagnostics naming commands, references and
/// environments) and the preview-controller `complete` reply
/// (`crates/preview-controller/STDIO.md`: project-index labels, citations and
/// declared commands, bound to a `source_versions` map). Metadata from an
/// older revision than the caret's is refused, never shown.
enum Completion {
    enum Kind: Equatable { case command, environment, reference, citation, word }

    struct Suggestion: Equatable {
        /// Display form, e.g. `\section` or `naïve`.
        let label: String
        /// Text that replaces the partial token (`completionRange`).
        let insertText: String
        let kind: Kind
        let detail: String
    }

    static let maxSuggestions = 12

    /// Commands parsed by the compiler on `main` (`crates/compiler/README.md`,
    /// "Supported commands"). Names are stored without the leading backslash;
    /// `\\` is the single-character name `\`.
    static let coreCommands = ["section", "subsection", "textbf", "emph", "textit", "begin", "end", "par", "\\"]

    /// Math commands present in `crates/compiler/src/math.rs` on
    /// `agent/claude/compiler-foundation` at `mathVerifiedAt`: each name was
    /// grepped in that file (the `"frac"`/`"sqrt"` parser arms and the
    /// `"alpha" => "α"` … `"int" => "∫"` symbol table).
    static let mathCommands = [
        "frac", "sqrt",
        "alpha", "beta", "gamma", "delta", "theta", "lambda", "mu", "pi", "sigma", "phi", "omega",
        "times", "div", "pm", "leq", "geq", "neq", "approx", "cdot", "infty", "sum", "int",
    ]
    static let mathVerifiedAt = "de1020c"

    static let defaultSupported: [String] = coreCommands + mathCommands

    /// Environments the compiler names explicitly (`document` is the only
    /// meaningful one; others typeset their body as plain text with a warning).
    static let knownEnvironments = ["document"]

    /// Citation commands whose `{` argument completes citation keys (the
    /// project-index reference set in `crates/project-index/README.md`).
    static let citationCommands = ["cite", "citep", "citet", "citeauthor", "citeyear", "parencite", "textcite", "autocite", "nocite"]

    // MARK: token at the caret

    /// The partial token ending at the caret, as UTF-8 byte offsets into `text`.
    enum Token: Equatable {
        /// `\` followed by `name` (possibly empty, or `\` for `\\`).
        case command(name: String, start: Int, end: Int)
        /// A run of letters, with what immediately precedes it (`\begin{`, `\ref{`, …).
        case word(text: String, start: Int, end: Int, context: Context)

        enum Context: Equatable { case none, beginEnvironment, endEnvironment, reference, citation }

        var start: Int {
            switch self { case .command(_, let s, _), .word(_, let s, _, _): return s }
        }
        var end: Int {
            switch self { case .command(_, _, let e), .word(_, _, let e, _): return e }
        }
    }

    /// Returns nil when the caret is not on a scalar boundary or there is no
    /// token immediately before it.
    static func token(in text: String, caretUTF16: Int) -> Token? {
        guard let caretByte = utf8Offset(of: caretUTF16, in: text) else { return nil }
        return withBytes(text) { b in token(in: b, caretByte: caretByte) }
    }

    private static func token(in b: UnsafeBufferPointer<UInt8>, caretByte: Int) -> Token? {
        guard caretByte <= b.count, let p = b.baseAddress else { return nil }
        let table = wordByteClass
        var start = caretByte
        var asciiLettersOnly = true
        while start > 0, table[Int(p[start - 1])] != 0 {
            if p[start - 1] >= 0x80 { asciiLettersOnly = false }
            start -= 1
        }
        if start == caretByte, caretByte >= 2, b[caretByte - 1] == backslash, b[caretByte - 2] == backslash {
            return .command(name: "\\", start: caretByte - 2, end: caretByte) // `\\`
        }
        if start > 0, b[start - 1] == backslash, asciiLettersOnly {
            let name = String(decoding: b[start..<caretByte], as: UTF8.self)
            return .command(name: name, start: start - 1, end: caretByte)
        }
        let context = context(in: b, before: start)
        // Right after an argument brace (`\ref{`, `\cite{`, …) an empty prefix
        // lists everything; elsewhere there must be letters before the caret.
        if start == caretByte { return context == .none ? nil : .word(text: "", start: start, end: caretByte, context: context) }
        let word = String(decoding: b[start..<caretByte], as: UTF8.self)
        guard word.unicodeScalars.allSatisfy({ $0.properties.isAlphabetic }) else { return nil }
        return .word(text: word, start: start, end: caretByte, context: context)
    }

    /// What the word starting at `start` completes, judged by the argument
    /// opener immediately before it.
    private static func context(in b: UnsafeBufferPointer<UInt8>, before start: Int) -> Token.Context {
        guard start > 0, b[start - 1] == UInt8(ascii: "{") else { return .none }
        if endsWith(b, upTo: start, suffix: "\\begin{") { return .beginEnvironment }
        if endsWith(b, upTo: start, suffix: "\\end{") { return .endEnvironment }
        if endsWith(b, upTo: start, suffix: "\\ref{") || endsWith(b, upTo: start, suffix: "\\eqref{")
            || endsWith(b, upTo: start, suffix: "\\pageref{") || endsWith(b, upTo: start, suffix: "\\autoref{") {
            return .reference
        }
        if citationCommands.contains(where: { endsWith(b, upTo: start, suffix: "\\" + $0 + "{") }) { return .citation }
        return .none
    }

    /// UTF-16 range the chosen suggestion replaces: the token before the caret
    /// including a leading `\`, or an empty range at the caret.
    static func completionRange(in text: String, caretUTF16: Int) -> NSRange {
        let length = (text as NSString).length
        let caret = max(0, min(caretUTF16, length))
        guard let token = token(in: text, caretUTF16: caret),
              let ns = text.nsRange(utf8Bytes: .init(path: "", startByte: token.start, endByte: token.end))
        else { return NSRange(location: caret, length: 0) }
        return ns
    }

    // MARK: suggestions

    /// Compatibility entry point: the caller asserts that `result` was compiled
    /// from `text` (it is bound as-is). Prefer `suggestions(in:caretUTF16:metadata:)`
    /// with metadata the caller has already bound to the caret's revision.
    static func suggestions(in text: String, caretUTF16: Int, result: RuntimeV1.CompileResult?,
                            supported: [String] = defaultSupported) -> [Suggestion] {
        suggestions(in: text, caretUTF16: caretUTF16, metadata: result.map(Metadata.from), supported: supported)
    }

    /// `metadata` must already be bound to the revision of `text` (see
    /// `Metadata.bound(to:)`); unbound metadata is the caller's bug, never
    /// this function's to detect. `cancelled` is polled between scan phases
    /// so an off-main computation stops early; a cancelled call returns `[]`.
    static func suggestions(in text: String, caretUTF16: Int, metadata: Metadata?,
                            supported: [String] = defaultSupported,
                            cancelled: () -> Bool = { false }) -> [Suggestion] {
        guard let token = token(in: text, caretUTF16: caretUTF16), !cancelled() else { return [] }
        let out: [Suggestion]
        switch token {
        case .command(let prefix, _, _):
            out = commandSuggestions(prefix: prefix, tokenStart: token.start, text: text,
                                     metadata: metadata, supported: supported, cancelled: cancelled)
        case .word(let prefix, _, _, let context):
            guard prefix.unicodeScalars.count >= 2 || context != .none else { return [] }
            switch context {
            case .beginEnvironment, .endEnvironment:
                out = environmentSuggestions(prefix: prefix, tokenStart: token.start, text: text,
                                             closing: context == .endEnvironment, metadata: metadata)
            case .reference:
                out = referenceSuggestions(prefix: prefix, text: text, metadata: metadata)
            case .citation:
                out = citationSuggestions(prefix: prefix, text: text, metadata: metadata)
            case .none:
                out = wordSuggestions(prefix: prefix, tokenStart: token.start, text: text)
            }
        }
        return cancelled() ? [] : out
    }

    private static func commandSuggestions(prefix: String, tokenStart: Int, text: String, metadata: Metadata?,
                                           supported: [String], cancelled: () -> Bool) -> [Suggestion] {
        var out: [Suggestion] = []
        let scan = scanCommands(in: text, tokenStart: tokenStart, prefix: prefix)
        if cancelled() { return [] }
        // 1. Close environments still open at the caret.
        for open in scan.open.reversed() where "end".hasPrefix(prefix) {
            out.append(Suggestion(label: "\\end{\(open.name)}", insertText: "\\end{\(open.name)}", kind: .environment,
                                  detail: "closes \\begin{\(open.name)} at byte \(open.byte)"))
        }
        // 2. Supported commands.
        var offered = Set(supported)
        for name in supported where name.hasPrefix(prefix) {
            let isMath = mathCommands.contains(name) && !coreCommands.contains(name)
            let detail = isMath ? "math · verified in compiler at \(mathVerifiedAt)" : "supported by this compiler"
            out.append(Suggestion(label: "\\" + name, insertText: "\\" + name, kind: .command, detail: detail))
        }
        // 3. Commands the project index saw declared (`\newcommand` and friends)
        //    at this exact revision.
        if let metadata {
            for item in metadata.commands where item.name.hasPrefix(prefix) && item.name != prefix && offered.insert(item.name).inserted {
                out.append(Suggestion(label: "\\" + item.name, insertText: "\\" + item.name, kind: .command,
                                      detail: item.detail(noun: "declared", revision: metadata.revision)))
            }
        }
        // 4. Commands typed in the document that the compiler does not support,
        //    with the compiler's own diagnostic when it named the command at
        //    this revision.
        for name in scan.commands where !offered.contains(name) {
            var detail = "not supported by this compiler version"
            if let message = metadata?.diagnosticsByCommand[name] { detail += " — " + message }
            out.append(Suggestion(label: "\\" + name, insertText: "\\" + name, kind: .command, detail: detail))
        }
        return Array(out.prefix(maxSuggestions))
    }

    private static func environmentSuggestions(prefix: String, tokenStart: Int, text: String, closing: Bool,
                                               metadata: Metadata?) -> [Suggestion] {
        var names: [String] = []
        if closing {
            names += openEnvironments(in: text, beforeByte: tokenStart).reversed().map(\.name)
        }
        names += knownEnvironments
        names += documentEnvironments(in: text)
        var seen = Set<String>()
        var out: [Suggestion] = []
        for name in names where name.hasPrefix(prefix) && seen.insert(name).inserted {
            var detail = knownEnvironments.contains(name) ? "supported by this compiler" : "seen in this document"
            if let message = metadata?.diagnosticsByEnvironment[name] { detail += " — " + message }
            out.append(Suggestion(label: name, insertText: name + "}", kind: .environment, detail: detail))
        }
        return Array(out.prefix(maxSuggestions))
    }

    private static func referenceSuggestions(prefix: String, text: String, metadata: Metadata?) -> [Suggestion] {
        var seen = Set<String>()
        var out: [Suggestion] = []
        for label in labels(in: text) where label.hasPrefix(prefix) && seen.insert(label).inserted {
            var detail = "\\label in this document"
            if let metadata, metadata.unresolvedReferences.contains(label) {
                detail += " — undefined when revision \(metadata.revision) compiled"
            }
            out.append(Suggestion(label: label, insertText: label + "}", kind: .reference, detail: detail))
        }
        // Labels the project index knows from other documents of the project.
        if let metadata {
            for item in metadata.labels where item.name.hasPrefix(prefix) && seen.insert(item.name).inserted {
                out.append(Suggestion(label: item.name, insertText: item.name + "}", kind: .reference,
                                      detail: item.detail(noun: "defined", revision: metadata.revision)))
            }
        }
        return Array(out.prefix(maxSuggestions))
    }

    private static func citationSuggestions(prefix: String, text: String, metadata: Metadata?) -> [Suggestion] {
        var seen = Set<String>()
        var out: [Suggestion] = []
        for key in bibitems(in: text) where key.hasPrefix(prefix) && seen.insert(key).inserted {
            out.append(Suggestion(label: key, insertText: key + "}", kind: .citation, detail: "\\bibitem in this document"))
        }
        if let metadata {
            for item in metadata.citations where item.name.hasPrefix(prefix) && seen.insert(item.name).inserted {
                out.append(Suggestion(label: item.name, insertText: item.name + "}", kind: .citation,
                                      detail: item.detail(noun: "defined", revision: metadata.revision)))
            }
        }
        return Array(out.prefix(maxSuggestions))
    }

    private static func wordSuggestions(prefix: String, tokenStart: Int, text: String) -> [Suggestion] {
        let counts = wordFrequencies(in: text, prefix: prefix, excludingTokenAt: tokenStart)
        let ranked = counts.sorted { a, b in a.value != b.value ? a.value > b.value : a.key < b.key }
        return ranked.prefix(maxSuggestions).map {
            Suggestion(label: $0.key, insertText: $0.key, kind: .word, detail: "\($0.value)× in this document")
        }
    }

    // MARK: document scans (UTF-8 bytes; only matches become Strings)

    struct OpenEnvironment: Equatable { let name: String; let byte: Int }

    /// One pass for the `\`-token case: environments still open before the
    /// token plus every distinct control word starting with `prefix` (the token
    /// itself excluded).
    static func scanCommands(in text: String, tokenStart: Int, prefix: String) -> (open: [OpenEnvironment], commands: [String]) {
        var stack: [OpenEnvironment] = []
        var commands: [String] = []
        var seen = Set<String>()
        let pre = Array(prefix.utf8)
        withBytes(text) { b in
            forEachCommand(in: b, upTo: b.count) { name, nameStart, arg in
                let byte = nameStart - 1
                if byte < tokenStart, let arg {
                    if bytes(name, equal: "begin") {
                        stack.append(OpenEnvironment(name: String(decoding: arg, as: UTF8.self), byte: byte))
                    } else if bytes(name, equal: "end") {
                        let env = String(decoding: arg, as: UTF8.self)
                        if let i = stack.lastIndex(where: { $0.name == env }) { stack.remove(at: i) }
                    }
                }
                guard byte != tokenStart, name.count >= pre.count,
                      pre.isEmpty || memcmp(name.baseAddress, pre, pre.count) == 0 else { return }
                let word = String(decoding: name, as: UTF8.self)
                if seen.insert(word).inserted { commands.append(word) }
            }
        }
        return (stack, commands)
    }

    /// `\begin{X}` before `byte` with no matching `\end{X}` before it, outermost first.
    static func openEnvironments(in text: String, beforeByte byte: Int) -> [OpenEnvironment] {
        var stack: [OpenEnvironment] = []
        withBytes(text) { b in
            let limit = min(byte, b.count)
            forEachCommand(in: b, upTo: limit) { name, nameStart, arg in
                guard let arg else { return }
                if bytes(name, equal: "begin") {
                    stack.append(OpenEnvironment(name: String(decoding: arg, as: UTF8.self), byte: nameStart - 1))
                } else if bytes(name, equal: "end") {
                    let env = String(decoding: arg, as: UTF8.self)
                    if let i = stack.lastIndex(where: { $0.name == env }) { stack.remove(at: i) }
                }
            }
        }
        return stack
    }

    /// Names of environments appearing in `\begin{…}` anywhere in the document.
    static func documentEnvironments(in text: String) -> [String] {
        var out: [String] = []
        withBytes(text) { b in
            forEachCommand(in: b, upTo: b.count) { name, _, arg in
                guard let arg, bytes(name, equal: "begin") else { return }
                let env = String(decoding: arg, as: UTF8.self)
                if !out.contains(env) { out.append(env) }
            }
        }
        return out
    }

    /// Arguments of `\label{…}` anywhere in the document, in order.
    static func labels(in text: String) -> [String] {
        var out: [String] = []
        withBytes(text) { b in
            forEachCommand(in: b, upTo: b.count) { name, _, arg in
                if let arg, bytes(name, equal: "label") { out.append(String(decoding: arg, as: UTF8.self)) }
            }
        }
        return out
    }

    /// Keys of `\bibitem{…}` anywhere in the document, in order. `\bibitem[x]{key}`
    /// is not matched (the brace does not follow the name); the project index
    /// covers that form.
    static func bibitems(in text: String) -> [String] {
        var out: [String] = []
        withBytes(text) { b in
            forEachCommand(in: b, upTo: b.count) { name, _, arg in
                if let arg, bytes(name, equal: "bibitem") { out.append(String(decoding: arg, as: UTF8.self)) }
            }
        }
        return out
    }

    /// Control words typed anywhere in the document that start with `prefix`,
    /// first occurrence first, excluding the one being typed (whose `\` is at
    /// `excludingTokenAt`).
    static func documentCommands(in text: String, excludingTokenAt tokenStart: Int, prefix: String = "") -> [String] {
        var out: [String] = []
        var seen = Set<String>()
        let pre = Array(prefix.utf8)
        withBytes(text) { b in
            forEachCommand(in: b, upTo: b.count) { name, nameStart, _ in
                guard nameStart - 1 != tokenStart, name.count >= pre.count,
                      pre.isEmpty || memcmp(name.baseAddress, pre, pre.count) == 0 else { return }
                let word = String(decoding: name, as: UTF8.self)
                if seen.insert(word).inserted { out.append(word) }
            }
        }
        return out
    }

    /// Words (letters only, more than 3 characters) starting with `prefix`
    /// (ASCII case-insensitive), counted. The word under the caret is not counted.
    static func wordFrequencies(in text: String, prefix: String, excludingTokenAt tokenStart: Int) -> [String: Int] {
        var counts: [String: Int] = [:]
        let lowered = prefix.utf8.map { c -> UInt8 in (c >= 0x41 && c <= 0x5A) ? c + 0x20 : c }
        guard let lower = lowered.first else { return counts }
        let upper: UInt8 = (lower >= 0x61 && lower <= 0x7A) ? lower - 0x20 : lower
        withBytes(text) { b in
            guard let p = b.baseAddress else { return }
            let table = wordByteClass
            let n = b.count
            // Jump between occurrences of the prefix's first byte (either case) with
            // memchr; only words starting there are walked byte by byte.
            var nextLower = next(lower, in: p, from: 0, count: n)
            var nextUpper = upper == lower ? n : next(upper, in: p, from: 0, count: n)
            while true {
                let start = min(nextLower, nextUpper)
                guard start < n else { break }
                var from = start + 1
                if start == 0 || table[Int(p[start - 1])] == 0 { // at a word start
                    var i = start + 1
                    var ascii = p[start] < 0x80
                    while i < n, table[Int(p[i])] != 0 { if p[i] >= 0x80 { ascii = false }; i += 1 }
                    from = i
                    // Cheap rejections first: length, control words (`\foo`), the token itself.
                    if i - start > 3, start != tokenStart, start == 0 || p[start - 1] != backslash,
                       hasPrefixCaseInsensitive(p, start: start, end: i, prefix: lowered) {
                        let word = String(decoding: UnsafeBufferPointer(start: p + start, count: i - start), as: UTF8.self)
                        // ASCII words are letters by construction; others are checked scalar by scalar.
                        if ascii || (word.unicodeScalars.count > 3 && word.unicodeScalars.allSatisfy({ $0.properties.isAlphabetic })) {
                            counts[word, default: 0] += 1
                        }
                    }
                }
                if nextLower < from { nextLower = next(lower, in: p, from: from, count: n) }
                if nextUpper < from { nextUpper = next(upper, in: p, from: from, count: n) }
            }
        }
        return counts
    }

    // MARK: revision-bound metadata from Rust producers

    /// Completion vocabulary produced by a Rust worker for one exact editor
    /// revision. Everything is bounded (`Limits`) and carries the revision it
    /// was produced for; `bound(to:)` is the only way to obtain metadata for a
    /// caret, and it refuses any other revision.
    struct Metadata: Equatable {
        enum Origin: Equatable {
            /// runtime-v1 `compile_result` for `projectId`.
            case compileResult(projectId: String)
            /// preview-controller `complete` reply, bound to the helper's
            /// per-document `source_versions` map (ledger revisions, not the
            /// editor revision; the caller maps between them).
            case projectIndex(sourceVersions: [String: Int])
        }

        /// One project-index name with its location counts (locations
        /// themselves are not retained; the helper caps them at 100 and flags
        /// truncation, which is preserved).
        struct Item: Equatable {
            let name: String
            let definitions: Int
            let occurrences: Int
            let locationsTruncated: Bool
            /// Project-relative path of the first definition, when any.
            let definedIn: String?

            func detail(noun: String, revision: Int) -> String {
                var s: String
                if definitions == 0 {
                    s = "unresolved in the project index"
                } else {
                    s = "\(noun) in \(definedIn ?? "project")"
                    if definitions > 1 { s += " (+\(definitions - 1) more)" }
                }
                if occurrences > 0 { s += " · \(occurrences)\(locationsTruncated ? "+" : "") use\(occurrences == 1 ? "" : "s")" }
                return s + " · revision \(revision)"
            }
        }

        enum Limits {
            /// Matches the helper's `limit` upper bound and location cap.
            static let maxItemsPerCategory = 100
            /// project-index bounds keys to 4096 bytes; longer names are dropped.
            static let maxNameBytes = 4096
            /// Largest `complete` reply decoded (100 items × 200 locations × ~80 B
            /// is ~1.6 MB; the helper frame limit is 16 MiB).
            static let maxReplyBytes = 4 << 20
            static let maxDiagnostics = 256
            static let maxMessageCharacters = 512
        }

        enum DecodeError: Error, Equatable {
            case tooLarge(bytes: Int)
            case malformed(String)
            /// The reply's `source_versions` differ from the versions the caller
            /// queried with: the helper answered for another snapshot.
            case staleSourceVersions(expected: [String: Int], got: [String: Int])
        }

        let origin: Origin
        /// Editor revision this metadata describes.
        let revision: Int
        var labels: [Item] = []
        var citations: [Item] = []
        var commands: [Item] = []
        /// First diagnostic message naming `\name`, keyed by name (no backslash).
        var diagnosticsByCommand: [String: String] = [:]
        /// `environment 'X' is not implemented; …` messages keyed by `X`.
        var diagnosticsByEnvironment: [String: String] = [:]
        /// Keys from `undefined reference 'key'` diagnostics.
        var unresolvedReferences: Set<String> = []
        /// Some cap in `Limits` dropped data.
        var truncated = false

        /// The metadata when it was produced for exactly `revision`, else nil.
        /// A nil `revision` (the editor has not told the view its revision)
        /// binds nothing.
        func bound(to revision: Int?) -> Metadata? {
            guard let revision, revision == self.revision else { return nil }
            return self
        }

        /// Vocabulary from the compiler's own result: only the revision and its
        /// diagnostics carry completion information in runtime-v1.
        static func from(_ result: RuntimeV1.CompileResult) -> Metadata {
            var m = Metadata(origin: .compileResult(projectId: result.projectId), revision: result.revision)
            var used = 0
            for d in result.diagnostics {
                guard used < Limits.maxDiagnostics else { m.truncated = true; break }
                let message = d.message.count > Limits.maxMessageCharacters
                    ? String(d.message.prefix(Limits.maxMessageCharacters)) + "…" : d.message
                if let key = quoted(after: "undefined reference '", in: d.message) {
                    if m.unresolvedReferences.insert(key).inserted { used += 1 }
                } else if let env = quoted(after: "environment '", in: d.message) {
                    if m.diagnosticsByEnvironment[env] == nil { m.diagnosticsByEnvironment[env] = message; used += 1 }
                } else if let bs = d.message.firstIndex(of: "\\") {
                    let name = d.message[d.message.index(after: bs)...].prefix { $0.isASCII && $0.isLetter }
                    if !name.isEmpty, m.diagnosticsByCommand[String(name)] == nil {
                        m.diagnosticsByCommand[String(name)] = message
                        used += 1
                    }
                }
            }
            return m
        }

        /// Decodes a preview-controller `complete` reply payload
        /// (`{"source_versions": {path: revision}, "completions": [...]}`) for
        /// one `category` into metadata bound to `editorRevision`, the editor
        /// revision at which `expectedSourceVersions` was the current snapshot.
        /// A reply for other source versions is refused as stale.
        static func decodeProjectIndexReply(_ data: Data, category: Kind, editorRevision: Int,
                                            expectedSourceVersions: [String: Int]) throws -> Metadata {
            guard data.count <= Limits.maxReplyBytes else { throw DecodeError.tooLarge(bytes: data.count) }
            let reply: ProjectIndexReply
            do { reply = try JSONDecoder().decode(ProjectIndexReply.self, from: data) }
            catch { throw DecodeError.malformed("\(error)") }
            guard reply.sourceVersions == expectedSourceVersions else {
                throw DecodeError.staleSourceVersions(expected: expectedSourceVersions, got: reply.sourceVersions)
            }
            var m = Metadata(origin: .projectIndex(sourceVersions: reply.sourceVersions), revision: editorRevision)
            var items: [Item] = []
            var seen = Set<String>()
            for c in reply.completions {
                guard items.count < Limits.maxItemsPerCategory else { m.truncated = true; break }
                guard c.name.utf8.count <= Limits.maxNameBytes, !c.name.isEmpty else { m.truncated = true; continue }
                guard seen.insert(c.name).inserted else { continue }
                items.append(Item(name: c.name, definitions: c.definitions.count, occurrences: c.occurrences.count,
                                  locationsTruncated: c.locationsTruncated, definedIn: c.definitions.first?.path))
            }
            switch category {
            case .reference: m.labels = items
            case .citation: m.citations = items
            case .command: m.commands = items
            case .environment, .word: throw DecodeError.malformed("project index has no \(category) category")
            }
            return m
        }

        /// Combines metadata produced for the same revision (compile-result
        /// diagnostics plus one or more project-index categories). Nil when the
        /// revisions differ: metadata never straddles revisions.
        func merged(with other: Metadata) -> Metadata? {
            guard other.revision == revision else { return nil }
            var m = self
            m.labels = Self.union(labels, other.labels)
            m.citations = Self.union(citations, other.citations)
            m.commands = Self.union(commands, other.commands)
            m.diagnosticsByCommand.merge(other.diagnosticsByCommand) { mine, _ in mine }
            m.diagnosticsByEnvironment.merge(other.diagnosticsByEnvironment) { mine, _ in mine }
            m.unresolvedReferences.formUnion(other.unresolvedReferences)
            m.truncated = truncated || other.truncated
            return m
        }

        private static func union(_ a: [Item], _ b: [Item]) -> [Item] {
            var seen = Set(a.map(\.name))
            return a + b.filter { seen.insert($0.name).inserted }
        }

        private static func quoted(after prefix: String, in message: String) -> String? {
            guard let r = message.range(of: prefix) else { return nil }
            let rest = message[r.upperBound...]
            guard let close = rest.firstIndex(of: "'") else { return nil }
            let key = rest[..<close]
            return key.isEmpty ? nil : String(key)
        }

        /// Wire shape of the helper's `complete` reply payload.
        struct ProjectIndexReply: Decodable {
            struct Location: Decodable {
                let path: String
                let revision: Int
                let startByte: Int
                let endByte: Int
                enum CodingKeys: String, CodingKey { case path, revision, startByte = "start_byte", endByte = "end_byte" }
            }
            struct Item: Decodable {
                let name: String
                let definitions: [Location]
                let occurrences: [Location]
                let locationsTruncated: Bool
                enum CodingKeys: String, CodingKey { case name, definitions, occurrences, locationsTruncated = "locations_truncated" }
            }
            let sourceVersions: [String: Int]
            let completions: [Item]
            enum CodingKeys: String, CodingKey { case sourceVersions = "source_versions", completions }
        }
    }

    // MARK: byte helpers

    static let backslash = UInt8(ascii: "\\")

    /// 256-entry table: 1 for ASCII letters, 2 for every non-ASCII byte
    /// (multi-byte scalars stay whole; matches are re-checked as alphabetic
    /// scalars before use), 0 otherwise. A raw pointer keeps debug-build
    /// scans free of array bounds checks and retain/release traffic.
    static let wordByteClass: UnsafePointer<UInt8> = {
        let table = UnsafeMutablePointer<UInt8>.allocate(capacity: 256)
        for c in 0..<256 {
            let letter = (c >= 0x41 && c <= 0x5A) || (c >= 0x61 && c <= 0x7A)
            table[c] = letter ? 1 : (c >= 0x80 ? 2 : 0)
        }
        return UnsafePointer(table)
    }()

    static func isWordByte(_ c: UInt8) -> Bool { wordByteClass[Int(c)] != 0 }

    /// Index of the next `byte` at or after `from`, or `n` (vectorised libc scan).
    @inline(__always) static func next(_ byte: UInt8, in p: UnsafePointer<UInt8>, from: Int, count n: Int) -> Int {
        guard from < n, let hit = memchr(p + from, Int32(byte), n - from) else { return n }
        return UnsafePointer<UInt8>(hit.assumingMemoryBound(to: UInt8.self)) - p
    }

    @inline(__always) static func nextBackslash(_ p: UnsafePointer<UInt8>, from: Int, count n: Int) -> Int {
        next(backslash, in: p, from: from, count: n)
    }

    /// Calls `body` for each control word `\name` (and its `{arg}` when one
    /// immediately follows) whose backslash lies before `limit`.
    /// `name` is the control word's bytes (decode with `String(decoding:as:)`
    /// only when needed — most callers compare against a few literals first).
    static func forEachCommand(in b: UnsafeBufferPointer<UInt8>, upTo limit: Int,
                               _ body: (_ name: UnsafeBufferPointer<UInt8>, _ nameStart: Int, _ arg: UnsafeBufferPointer<UInt8>?) -> Void) {
        guard let p = b.baseAddress else { return }
        let n = min(limit, b.count), total = b.count
        let table = wordByteClass
        var i = nextBackslash(p, from: 0, count: n)
        while i < n {
            let nameStart = i + 1
            var j = nameStart
            while j < total, table[Int(p[j])] == 1 { j += 1 }
            guard j > nameStart else { i = nextBackslash(p, from: min(j + 1, total), count: n); continue } // `\\`, `\{`, …
            let name = UnsafeBufferPointer(start: p + nameStart, count: j - nameStart)
            var arg: UnsafeBufferPointer<UInt8>?
            if j < total, p[j] == UInt8(ascii: "{") {
                var k = j + 1
                while k < total, p[k] != UInt8(ascii: "}"), p[k] != UInt8(ascii: "{"), p[k] != backslash,
                      p[k] != UInt8(ascii: "\n") { k += 1 }
                if k < total, p[k] == UInt8(ascii: "}") {
                    arg = UnsafeBufferPointer(start: p + j + 1, count: k - j - 1)
                    j = k + 1
                }
            }
            body(name, nameStart, arg)
            i = nextBackslash(p, from: j, count: n)
        }
    }

    @inline(__always) static func bytes(_ b: UnsafeBufferPointer<UInt8>, equal literal: StaticString) -> Bool {
        b.count == literal.utf8CodeUnitCount && memcmp(b.baseAddress, literal.utf8Start, b.count) == 0
    }

    private static func endsWith(_ b: UnsafeBufferPointer<UInt8>, upTo end: Int, suffix: String) -> Bool {
        let s = Array(suffix.utf8)
        guard end >= s.count else { return false }
        for (k, c) in s.enumerated() where b[end - s.count + k] != c { return false }
        return true
    }

    private static func hasPrefixCaseInsensitive(_ p: UnsafePointer<UInt8>, start: Int, end: Int, prefix: [UInt8]) -> Bool {
        guard end - start >= prefix.count else { return false }
        var k = 0
        while k < prefix.count {
            var c = p[start + k]
            if c >= 0x41 && c <= 0x5A { c += 0x20 }
            if c != prefix[k] { return false }
            k += 1
        }
        return true
    }

    /// UTF-8 offset of a UTF-16 caret, or nil when out of range or inside a
    /// surrogate pair.
    static func utf8Offset(of caretUTF16: Int, in text: String) -> Int? {
        // A negative location traps inside Foundation; past-the-end and
        // surrogate-interior locations already come back as nil. Avoid
        // `utf16.count`, which is O(n) on a freshly built native string.
        guard caretUTF16 >= 0 else { return nil }
        return text.utf8ByteRange(of: NSRange(location: caretUTF16, length: 0))?.start
    }

    /// Runs `body` over the string's UTF-8 bytes without copying when the
    /// string is already contiguous (native Swift strings are).
    static func withBytes<R>(_ text: String, _ body: (UnsafeBufferPointer<UInt8>) -> R) -> R {
        var copy = text
        return copy.withUTF8(body)
    }
}

// MARK: - Off-main candidate computation

/// Computes candidates off the main thread and delivers them on the main run
/// loop only while the request is still current. `cancel()` (every text change
/// or caret move) and any newer `schedule` invalidate the pending job: its
/// result is refused at delivery and never reaches the view. The keystroke
/// path therefore only copies the text and enqueues; it never scans.
@MainActor
final class CompletionScheduler {
    struct Request {
        var text: String
        var caretUTF16: Int
        /// Already bound to the revision of `text` (`Metadata.bound(to:)`).
        var metadata: Completion.Metadata?
        var supported: [String] = Completion.defaultSupported
    }

    struct Outcome: Equatable {
        let generation: Int
        let caretUTF16: Int
        /// UTF-16 range the items replace, computed from the request's text.
        let range: NSRange
        let items: [Completion.Suggestion]
        /// Wall time of the off-main scan.
        let computeMs: Double
    }

    struct Statistics: Equatable {
        var scheduled = 0
        /// Outcomes handed to the caller.
        var delivered = 0
        /// Outcomes that reached the main thread after their job was cancelled
        /// or superseded, and were dropped.
        var refusedStale = 0
        /// Jobs cancelled (explicitly or by a newer request) before delivery.
        var cancelled = 0
    }

    /// Runs one job somewhere off the main thread. The default is a serial
    /// user-initiated queue; tests inject an executor that holds jobs so the
    /// order of caret moves and job completion is deterministic.
    typealias Executor = (@escaping @Sendable () -> Void) -> Void

    /// Cancellation flag shared with the running job; polled between scan phases.
    final class Job: @unchecked Sendable {
        let generation: Int
        private let cancelledFlag = OSAllocatedUnfairLock(initialState: false)
        init(generation: Int) { self.generation = generation }
        var isCancelled: Bool { cancelledFlag.withLock { $0 } }
        func cancel() { cancelledFlag.withLock { $0 = true } }
    }

    private(set) var generation = 0
    private(set) var statistics = Statistics()
    private(set) var pending: Job?
    private let execute: Executor

    init(executor: Executor? = nil) {
        if let executor {
            execute = executor
        } else {
            let queue = DispatchQueue(label: "flashtex.completion", qos: .userInitiated)
            execute = { queue.async(execute: $0) }
        }
    }

    /// Enqueues `request`; `deliver` runs on the main run loop with the outcome
    /// unless the job was cancelled or superseded first. Returns the job's
    /// generation.
    @discardableResult
    func schedule(_ request: Request, deliver: @escaping @MainActor (Outcome) -> Void) -> Int {
        cancelPending()
        generation += 1
        let job = Job(generation: generation)
        pending = job
        statistics.scheduled += 1
        execute { [weak self] in
            let t0 = MonotonicClock.nowNs()
            let range = Completion.completionRange(in: request.text, caretUTF16: request.caretUTF16)
            let items = job.isCancelled ? [] : Completion.suggestions(in: request.text, caretUTF16: request.caretUTF16,
                                                                     metadata: request.metadata, supported: request.supported,
                                                                     cancelled: { job.isCancelled })
            let outcome = Outcome(generation: job.generation, caretUTF16: request.caretUTF16, range: range, items: items,
                                  computeMs: Double(MonotonicClock.nowNs() - t0) / 1e6)
            Self.onMain { [weak self] in
                MainActor.assumeIsolated { [weak self] in
                    guard let self else { return }
                    // Stale refusal: only the exact current, uncancelled job is delivered.
                    guard !job.isCancelled, job.generation == self.generation, self.pending === job else {
                        self.statistics.refusedStale += 1
                        return
                    }
                    self.pending = nil
                    self.statistics.delivered += 1
                    deliver(outcome)
                }
            }
        }
        return job.generation
    }

    /// Invalidates the pending job (if any) and every outcome computed so far.
    func cancel() {
        cancelPending()
        generation += 1
    }

    private func cancelPending() {
        guard let job = pending, !job.isCancelled else { return }
        job.cancel()
        statistics.cancelled += 1
        pending = nil
    }

    /// Head-of-run-loop delivery (see `WorkerClient.deliver`): a plain
    /// `DispatchQueue.main.async` waits for AppKit to reach the dispatch port.
    nonisolated private static func onMain(_ block: @escaping @Sendable () -> Void) {
        CFRunLoopPerformBlock(CFRunLoopGetMain(), CFRunLoopMode.commonModes.rawValue, block)
        CFRunLoopWakeUp(CFRunLoopGetMain())
    }
}

// MARK: - Session state and popup

/// The completion list shown for one token. Owned by `CompletingTextView`;
/// the popup only renders it.
struct CompletionSession: Equatable {
    var items: [Completion.Suggestion]
    /// UTF-16 range the chosen item replaces.
    var range: NSRange
    var selectedIndex: Int
    /// Scheduler generation the items were computed for; any text change or
    /// caret move since then has advanced it.
    var generation: Int
    /// Editor revision the metadata was bound to (nil: no metadata used).
    var metadataRevision: Int?

    var selected: Completion.Suggestion? { items.indices.contains(selectedIndex) ? items[selectedIndex] : nil }
}

/// Non-activating list under the caret. Never becomes key, so it cannot take
/// keyboard focus from the editor (or from another app in tests).
final class CompletionPopup: NSPanel, NSTableViewDataSource, NSTableViewDelegate {
    private let table = NSTableView()
    private(set) var items: [Completion.Suggestion] = []
    static let rowHeight: CGFloat = 22
    static let width: CGFloat = 420

    init() {
        super.init(contentRect: NSRect(x: 0, y: 0, width: Self.width, height: Self.rowHeight * 4),
                   styleMask: [.nonactivatingPanel, .borderless], backing: .buffered, defer: false)
        isFloatingPanel = true
        hidesOnDeactivate = false
        level = .popUpMenu
        hasShadow = true
        isReleasedWhenClosed = false
        isExcludedFromWindowsMenu = true
        animationBehavior = .none
        let column = NSTableColumn(identifier: .init("completion"))
        column.width = Self.width - 4
        table.addTableColumn(column)
        table.headerView = nil
        table.rowHeight = Self.rowHeight
        table.allowsEmptySelection = false
        table.allowsMultipleSelection = false
        table.refusesFirstResponder = true
        table.focusRingType = .none
        table.dataSource = self
        table.delegate = self
        table.setAccessibilityLabel("Completions")
        let scroll = NSScrollView(frame: contentView!.bounds)
        scroll.documentView = table
        scroll.hasVerticalScroller = true
        scroll.autoresizingMask = [.width, .height]
        scroll.borderType = .noBorder
        contentView?.addSubview(scroll)
        contentView?.wantsLayer = true
        contentView?.layer?.cornerRadius = 6
        contentView?.layer?.borderWidth = 1
        contentView?.layer?.borderColor = NSColor.separatorColor.cgColor
        backgroundColor = .windowBackgroundColor
    }

    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }

    func show(items: [Completion.Suggestion], selected: Int, below caretRect: NSRect, parent: NSWindow) {
        update(items: items, selected: selected)
        let rows = CGFloat(min(items.count, Completion.maxSuggestions))
        let height = rows * Self.rowHeight + 4
        var origin = NSPoint(x: caretRect.minX, y: caretRect.minY - height - 2)
        if let screen = parent.screen ?? NSScreen.main {
            let visible = screen.visibleFrame
            if origin.y < visible.minY { origin.y = caretRect.maxY + 2 } // flip above the caret
            origin.x = min(max(origin.x, visible.minX), max(visible.minX, visible.maxX - Self.width))
        }
        setFrame(NSRect(origin: origin, size: NSSize(width: Self.width, height: height)), display: true)
        if self.parent !== parent {
            self.parent?.removeChildWindow(self)
            parent.addChildWindow(self, ordered: .above)
        }
        orderFront(nil)
    }

    func update(items: [Completion.Suggestion], selected: Int) {
        self.items = items
        table.reloadData()
        if items.indices.contains(selected) {
            table.selectRowIndexes(IndexSet(integer: selected), byExtendingSelection: false)
            table.scrollRowToVisible(selected)
        }
    }

    func hide() {
        parent?.removeChildWindow(self)
        orderOut(nil)
        items = []
        table.reloadData()
    }

    func numberOfRows(in tableView: NSTableView) -> Int { items.count }

    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        let id = NSUserInterfaceItemIdentifier("row")
        let field = (tableView.makeView(withIdentifier: id, owner: nil) as? NSTextField) ?? {
            let f = NSTextField(labelWithString: "")
            f.identifier = id
            f.lineBreakMode = .byTruncatingTail
            f.allowsDefaultTighteningForTruncation = true
            return f
        }()
        field.attributedStringValue = Self.attributed(items[row])
        return field
    }

    /// `\section  cmd · supported by this compiler` — label in the editor's
    /// monospaced font, kind and detail in the secondary colour.
    static func attributed(_ s: Completion.Suggestion) -> NSAttributedString {
        let out = NSMutableAttributedString(string: s.label, attributes: [
            .font: NSFont.monospacedSystemFont(ofSize: 12, weight: .medium), .foregroundColor: NSColor.labelColor,
        ])
        out.append(NSAttributedString(string: "  \(s.kind.badge) · \(s.detail)", attributes: [
            .font: NSFont.systemFont(ofSize: 11), .foregroundColor: NSColor.secondaryLabelColor,
        ]))
        return out
    }
}

extension Completion.Kind {
    var badge: String {
        switch self {
        case .command: return "cmd"
        case .environment: return "env"
        case .reference: return "ref"
        case .citation: return "cite"
        case .word: return "word"
        }
    }
}

// MARK: - NSTextView integration

/// `NSTextView` with its own completion session: candidates are computed off
/// the main thread by `CompletionScheduler`, shown in `CompletionPopup`, and
/// chosen with ↑/↓, inserted with Return/Tab, dismissed with Esc. Any caret
/// move or text change that is not the user's own typing through the list
/// closes the session and cancels in-flight work. Esc and ⌃Space open the
/// list. Rust metadata (`compileResult`, `accept(projectIndex:)`) is used only
/// when bound to `editorRevision`, which the owner sets on every update.
final class CompletingTextView: NSTextView {
    var compileResult: RuntimeV1.CompileResult? {
        didSet { resultMetadata = compileResult.map(Completion.Metadata.from) }
    }
    var supportedCommands = Completion.defaultSupported

    /// Revision of `string` as the owner (ShellModel) counts it. Nil until the
    /// owner sets it, and nil binds no metadata: candidates then come from the
    /// document text alone, never from a result of unknown age.
    var editorRevision: Int?

    private(set) var resultMetadata: Completion.Metadata?
    private(set) var projectIndexMetadata: Completion.Metadata?

    /// Accepts metadata decoded from the helper's `complete` reply. Refuses
    /// (returns false) metadata older than the one already held; equal
    /// revisions merge (one reply per category).
    @discardableResult
    func accept(projectIndex metadata: Completion.Metadata) -> Bool {
        if let held = projectIndexMetadata {
            if metadata.revision < held.revision { return false }
            if let merged = held.merged(with: metadata) { projectIndexMetadata = merged; return true }
        }
        projectIndexMetadata = metadata
        return true
    }

    /// Metadata bound to `editorRevision`, merged across producers. Nil when
    /// nothing was produced for exactly this revision.
    var boundMetadata: Completion.Metadata? {
        let r = resultMetadata?.bound(to: editorRevision)
        let p = projectIndexMetadata?.bound(to: editorRevision)
        switch (r, p) {
        case (let r?, let p?): return r.merged(with: p)
        case (let r?, nil): return r
        case (nil, let p?): return p
        case (nil, nil): return nil
        }
    }

    /// Replaceable so tests can inject a manual executor.
    var scheduler = CompletionScheduler()
    private(set) var session: CompletionSession?
    var isCompletionActive: Bool { session != nil }

    enum CloseReason: Equatable { case accepted, escape, caretMoved, textChanged, noCandidates, resignedFirstResponder }
    private(set) var lastCloseReason: CloseReason?
    /// Latest outcome presented, for evidence (compute time off-main).
    private(set) var lastOutcome: CompletionScheduler.Outcome?

    private lazy var popup = CompletionPopup()
    private var typingThroughSession = false
    private var applyingCompletion = false
    private var lastCaret: NSRange?
    private var storageObserver: NSObjectProtocol?

    /// Scroll view + text view pair, like `NSTextView.scrollableTextView()`
    /// but with this subclass as the document view.
    static func scrollable() -> NSScrollView {
        let scroll = scrollableTextView()
        if scroll.documentView is CompletingTextView { return scroll }
        // Fallback if AppKit did not instantiate the subclass.
        let tv = CompletingTextView(frame: scroll.contentView.bounds)
        tv.autoresizingMask = [.width]
        tv.isVerticallyResizable = true
        tv.isHorizontallyResizable = false
        tv.maxSize = NSSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude)
        tv.textContainer?.widthTracksTextView = true
        tv.textContainer?.containerSize = NSSize(width: scroll.contentView.bounds.width, height: CGFloat.greatestFiniteMagnitude)
        scroll.documentView = tv
        return scroll
    }

    deinit {
        if let storageObserver { NotificationCenter.default.removeObserver(storageObserver) }
    }

    // MARK: synchronous AppKit completion API (kept for callers and tests)

    override var rangeForUserCompletion: NSRange {
        Completion.completionRange(in: string, caretUTF16: selectedRange().location)
    }

    /// Synchronous candidates for `charRange` (bound metadata only). The popup
    /// never uses this path; it exists for AppKit callers and tests.
    override func completions(forPartialWordRange charRange: NSRange,
                              indexOfSelectedItem index: UnsafeMutablePointer<Int>) -> [String]? {
        index.pointee = 0
        // The range came from `rangeForUserCompletion`; the text may have changed
        // since (or the caller may pass anything). Never index past the string.
        let text = string
        guard charRange.location != NSNotFound, charRange.location >= 0, charRange.length >= 0,
              NSMaxRange(charRange) <= (text as NSString).length else { return nil }
        let items = Completion.suggestions(in: text, caretUTF16: NSMaxRange(charRange), metadata: boundMetadata,
                                          supported: supportedCommands)
        return items.isEmpty ? nil : items.map(\.insertText)
    }

    /// AppKit calls this with the chosen string; the default implementation
    /// replaces `charRange`. Guard the range the same way so a stale range can
    /// never splice text at the wrong place.
    override func insertCompletion(_ word: String, forPartialWordRange charRange: NSRange,
                                   movement: Int, isFinal flag: Bool) {
        guard charRange.location != NSNotFound, charRange.location >= 0, charRange.length >= 0,
              NSMaxRange(charRange) <= (string as NSString).length else { return }
        super.insertCompletion(word, forPartialWordRange: charRange, movement: movement, isFinal: flag)
    }

    // MARK: session lifecycle

    /// Esc (AppKit's `cancelOperation:` → `complete:`) and ⌃Space land here.
    override func complete(_ sender: Any?) { requestCompletion() }

    /// Copies the text and enqueues the scan; the popup opens (or refreshes)
    /// when the outcome is delivered and still current.
    func requestCompletion() {
        observeStorageIfNeeded()
        let caret = selectedRange()
        guard caret.length == 0 else { return }
        lastCaret = caret
        let metadata = boundMetadata
        let request = CompletionScheduler.Request(text: string, caretUTF16: caret.location, metadata: metadata,
                                                  supported: supportedCommands)
        scheduler.schedule(request) { [weak self] outcome in
            self?.present(outcome, metadataRevision: metadata?.revision)
        }
    }

    private func present(_ outcome: CompletionScheduler.Outcome, metadataRevision: Int?) {
        lastOutcome = outcome
        // The scheduler refused other generations; the caret must also still
        // be where the request was made and the range must fit the text.
        let caret = selectedRange()
        guard caret.length == 0, caret.location == outcome.caretUTF16,
              NSMaxRange(outcome.range) <= (string as NSString).length else { return }
        guard !outcome.items.isEmpty else {
            if session != nil { close(.noCandidates) }
            return
        }
        var selected = 0
        if let current = session?.selected, let i = outcome.items.firstIndex(where: { $0.label == current.label }) {
            selected = i // the same item stays chosen while typing narrows the list
        }
        session = CompletionSession(items: outcome.items, range: outcome.range, selectedIndex: selected,
                                    generation: outcome.generation, metadataRevision: metadataRevision)
        showPopup()
    }

    private func showPopup() {
        guard let session, let window else { return }
        let caretRect = firstRect(forCharacterRange: NSRange(location: selectedRange().location, length: 0), actualRange: nil)
        popup.show(items: session.items, selected: session.selectedIndex, below: caretRect, parent: window)
    }

    func close(_ reason: CloseReason) {
        lastCloseReason = reason
        guard session != nil else { return }
        session = nil
        popup.hide()
    }

    func moveSelection(by delta: Int) {
        guard var s = session, !s.items.isEmpty else { return }
        s.selectedIndex = (s.selectedIndex + delta + s.items.count) % s.items.count
        session = s
        popup.update(items: s.items, selected: s.selectedIndex)
    }

    /// Replaces the session's range with the selected item, as one undoable
    /// edit, and closes the list. Refused when the text or caret changed since
    /// the items were computed.
    func acceptSelectedCompletion() {
        guard let s = session, let item = s.selected else { return }
        guard s.generation == scheduler.generation, NSMaxRange(s.range) <= (string as NSString).length,
              selectedRange() == NSRange(location: NSMaxRange(s.range), length: 0) else {
            close(.textChanged)
            return
        }
        applyingCompletion = true
        insertCompletion(item.insertText, forPartialWordRange: s.range, movement: NSReturnTextMovement, isFinal: true)
        applyingCompletion = false
        scheduler.cancel()
        close(.accepted)
    }

    // MARK: events

    override func keyDown(with event: NSEvent) {
        if event.modifierFlags.contains(.control), event.charactersIgnoringModifiers == " " {
            requestCompletion()
            return
        }
        guard session != nil else {
            // Esc opens the list (AppKit's own `cancelOperation:` → `complete:`
            // binding is not reliable outside a key window, so it is explicit).
            if event.keyCode == 53, event.modifierFlags.intersection([.command, .option, .control]).isEmpty {
                requestCompletion()
            } else {
                super.keyDown(with: event)
            }
            return
        }
        switch event.keyCode {
        case 125: moveSelection(by: 1) // ↓
        case 126: moveSelection(by: -1) // ↑
        case 36, 76, 48: acceptSelectedCompletion() // Return, Enter, Tab
        case 53: scheduler.cancel(); close(.escape) // Esc
        case 123, 124, 115, 119, 116, 121: // ←, →, Home, End, Page Up/Down leave the token
            close(.caretMoved)
            super.keyDown(with: event)
        default:
            // Ordinary typing (and Delete) narrows or widens the list: the
            // keystroke goes to the editor unchanged and a fresh scan is queued.
            typingThroughSession = true
            super.keyDown(with: event)
            typingThroughSession = false
            if session != nil { requestCompletion() }
        }
    }

    override func setSelectedRanges(_ ranges: [NSValue], affinity: NSSelectionAffinity, stillSelecting stillSelectingFlag: Bool) {
        super.setSelectedRanges(ranges, affinity: affinity, stillSelecting: stillSelectingFlag)
        guard !typingThroughSession, !applyingCompletion else { return }
        let caret = selectedRange()
        guard let last = lastCaret, caret != last else { return }
        lastCaret = caret
        scheduler.cancel()
        if session != nil { close(.caretMoved) }
    }

    override func didChangeText() {
        super.didChangeText()
        textChanged()
    }

    /// Programmatic replacement (`string =`, the owner's `replaceCharacters`)
    /// does not call `didChangeText`; the storage notification covers it.
    private func observeStorageIfNeeded() {
        guard storageObserver == nil, let storage = textStorage else { return }
        storageObserver = NotificationCenter.default.addObserver(forName: NSTextStorage.didProcessEditingNotification,
                                                                 object: storage, queue: nil) { [weak self] note in
            guard let storage = note.object as? NSTextStorage, storage.editedMask.contains(.editedCharacters) else { return }
            self?.textChanged()
        }
    }

    private func textChanged() {
        guard !typingThroughSession, !applyingCompletion else { return }
        scheduler.cancel()
        if session != nil { close(.textChanged) }
    }

    override func resignFirstResponder() -> Bool {
        let ok = super.resignFirstResponder()
        if ok, session != nil { scheduler.cancel(); close(.resignedFirstResponder) }
        return ok
    }
}
