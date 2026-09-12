import AppKit
import FlashTeXProtocol

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
/// 4. Environment names (after `\begin{`/`\end{`) and labels (after `\ref{`)
///    seen in the document.
/// 5. Words longer than 3 characters from the document, frequency-ranked.
enum Completion {
    enum Kind: Equatable { case command, environment, reference, word }

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

    // MARK: token at the caret

    /// The partial token ending at the caret, as UTF-8 byte offsets into `text`.
    enum Token: Equatable {
        /// `\` followed by `name` (possibly empty, or `\` for `\\`).
        case command(name: String, start: Int, end: Int)
        /// A run of letters, with what immediately precedes it (`\begin{`, `\ref{`, …).
        case word(text: String, start: Int, end: Int, context: Context)

        enum Context: Equatable { case none, beginEnvironment, endEnvironment, reference }

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
        if start == caretByte { return nil }
        let word = String(decoding: b[start..<caretByte], as: UTF8.self)
        guard word.unicodeScalars.allSatisfy({ $0.properties.isAlphabetic }) else { return nil }
        let context: Token.Context
        if endsWith(b, upTo: start, suffix: "\\begin{") { context = .beginEnvironment }
        else if endsWith(b, upTo: start, suffix: "\\end{") { context = .endEnvironment }
        else if endsWith(b, upTo: start, suffix: "\\ref{") || endsWith(b, upTo: start, suffix: "\\eqref{")
                    || endsWith(b, upTo: start, suffix: "\\pageref{") || endsWith(b, upTo: start, suffix: "\\autoref{") {
            context = .reference
        } else { context = .none }
        return .word(text: word, start: start, end: caretByte, context: context)
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

    static func suggestions(in text: String, caretUTF16: Int, result: RuntimeV1.CompileResult?,
                            supported: [String] = defaultSupported) -> [Suggestion] {
        guard let token = token(in: text, caretUTF16: caretUTF16) else { return [] }
        switch token {
        case .command(let prefix, _, _):
            return commandSuggestions(prefix: prefix, tokenStart: token.start, text: text,
                                      result: result, supported: supported)
        case .word(let prefix, _, _, let context):
            guard prefix.unicodeScalars.count >= 2 || context != .none else { return [] }
            switch context {
            case .beginEnvironment, .endEnvironment:
                return environmentSuggestions(prefix: prefix, tokenStart: token.start, text: text,
                                              closing: context == .endEnvironment)
            case .reference:
                return referenceSuggestions(prefix: prefix, text: text)
            case .none:
                return wordSuggestions(prefix: prefix, tokenStart: token.start, text: text)
            }
        }
    }

    private static func commandSuggestions(prefix: String, tokenStart: Int, text: String,
                                           result: RuntimeV1.CompileResult?, supported: [String]) -> [Suggestion] {
        var out: [Suggestion] = []
        let scan = scanCommands(in: text, tokenStart: tokenStart, prefix: prefix)
        // 1. Close environments still open at the caret.
        for open in scan.open.reversed() where "end".hasPrefix(prefix) {
            out.append(Suggestion(label: "\\end{\(open.name)}", insertText: "\\end{\(open.name)}", kind: .environment,
                                  detail: "closes \\begin{\(open.name)} at byte \(open.byte)"))
        }
        // 2. Supported commands.
        let supportedSet = Set(supported)
        for name in supported where name.hasPrefix(prefix) {
            let isMath = mathCommands.contains(name) && !coreCommands.contains(name)
            let detail = isMath ? "math · verified in compiler at \(mathVerifiedAt)" : "supported by this compiler"
            out.append(Suggestion(label: "\\" + name, insertText: "\\" + name, kind: .command, detail: detail))
        }
        // 3. Commands typed in the document that the compiler does not support.
        let mentioned = diagnosticsByCommand(result)
        for name in scan.commands where !supportedSet.contains(name) {
            var detail = "not supported by this compiler version"
            if let message = mentioned[name] { detail += " — " + message }
            out.append(Suggestion(label: "\\" + name, insertText: "\\" + name, kind: .command, detail: detail))
        }
        return Array(out.prefix(maxSuggestions))
    }

    private static func environmentSuggestions(prefix: String, tokenStart: Int, text: String, closing: Bool) -> [Suggestion] {
        var names: [String] = []
        if closing {
            names += openEnvironments(in: text, beforeByte: tokenStart).reversed().map(\.name)
        }
        names += knownEnvironments
        names += documentEnvironments(in: text)
        var seen = Set<String>()
        var out: [Suggestion] = []
        for name in names where name.hasPrefix(prefix) && seen.insert(name).inserted {
            let detail = knownEnvironments.contains(name) ? "supported by this compiler" : "seen in this document"
            out.append(Suggestion(label: name, insertText: name + "}", kind: .environment, detail: detail))
        }
        return Array(out.prefix(maxSuggestions))
    }

    private static func referenceSuggestions(prefix: String, text: String) -> [Suggestion] {
        var seen = Set<String>()
        var out: [Suggestion] = []
        for label in labels(in: text) where label.hasPrefix(prefix) && seen.insert(label).inserted {
            out.append(Suggestion(label: label, insertText: label + "}", kind: .reference, detail: "\\label in this document"))
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

    /// Message of the first diagnostic that names `\name`, keyed by name.
    private static func diagnosticsByCommand(_ result: RuntimeV1.CompileResult?) -> [String: String] {
        var out: [String: String] = [:]
        for d in result?.diagnostics ?? [] {
            guard let bs = d.message.firstIndex(of: "\\") else { continue }
            let name = d.message[d.message.index(after: bs)...].prefix { $0.isASCII && $0.isLetter }
            if !name.isEmpty, out[String(name)] == nil { out[String(name)] = d.message }
        }
        return out
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

// MARK: - NSTextView integration

/// `NSTextView` whose user-completion range includes a leading `\` and whose
/// completions come from `Completion.suggestions`. Esc and ⌃Space open the
/// popup. The compile result is pushed in by `SourceEditorView`.
final class CompletingTextView: NSTextView {
    var compileResult: RuntimeV1.CompileResult?
    var supportedCommands = Completion.defaultSupported

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

    override var rangeForUserCompletion: NSRange {
        Completion.completionRange(in: string, caretUTF16: selectedRange().location)
    }

    override func completions(forPartialWordRange charRange: NSRange,
                              indexOfSelectedItem index: UnsafeMutablePointer<Int>) -> [String]? {
        index.pointee = 0
        // The range came from `rangeForUserCompletion`; the text may have changed
        // since (or the caller may pass anything). Never index past the string.
        let text = string
        guard charRange.location != NSNotFound, charRange.location >= 0, charRange.length >= 0,
              NSMaxRange(charRange) <= (text as NSString).length else { return nil }
        let items = Completion.suggestions(in: text, caretUTF16: NSMaxRange(charRange), result: compileResult,
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

    override func keyDown(with event: NSEvent) {
        if event.modifierFlags.contains(.control), event.charactersIgnoringModifiers == " " {
            complete(nil)
            return
        }
        super.keyDown(with: event)
    }
}
