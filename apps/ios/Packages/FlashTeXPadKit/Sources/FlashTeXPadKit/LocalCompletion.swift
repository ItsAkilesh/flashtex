import Foundation

/// Source-derived completion for the iPad editor. Local only: nearby-v1 does
/// not carry the Mac's revision-bound completion metadata (preview-controller
/// `complete`, compile_result vocabulary), so this ranks what the document
/// itself proves — the same first four sources as the Mac's `Completion`
/// (apps/mac/Sources/FlashTeXMac/Completion.swift), without Rust vocabulary.
/// Works on UTF-8 bytes; a caret out of range yields nothing, never a trap.
public enum LocalCompletion {
    public enum Kind: String, Equatable { case environmentClose, command, environment, reference, citation }

    public struct Suggestion: Equatable, Identifiable {
        public var id: String { "\(kind.rawValue):\(text)" }
        public var kind: Kind
        public var text: String
        public var detail: String
        /// UTF-8 byte range of the prefix the suggestion replaces.
        public var replaceStart: Int
        public var replaceEnd: Int
    }

    public static func suggestions(in text: String, caretByte: Int, limit: Int = 12) -> [Suggestion] {
        let bytes = Array(text.utf8)
        guard caretByte >= 0, caretByte <= bytes.count else { return [] }
        let before = bytes[0..<caretByte]

        // 1. Word/command prefix under the caret.
        var start = caretByte
        while start > 0, isWordByte(bytes[start - 1]) { start -= 1 }
        var isCommand = false
        if start > 0, bytes[start - 1] == 0x5C /* \ */ { isCommand = true; start -= 1 }
        let prefix = String(decoding: bytes[start..<caretByte], as: UTF8.self)

        // 2. Argument context: `\begin{`, `\end{`, `\ref{`, `\cite{`.
        var argCommand: String?
        if !isCommand, let brace = before.lastIndex(of: 0x7B), brace >= start - 1 {
            var s = brace
            while s > 0, isWordByte(bytes[s - 1]) { s -= 1 }
            if s > 0, bytes[s - 1] == 0x5C { argCommand = String(decoding: bytes[s..<brace], as: UTF8.self) }
        }

        var out: [Suggestion] = []
        func add(_ kind: Kind, _ t: String, _ detail: String) {
            guard !out.contains(where: { $0.text == t && $0.kind == kind }) else { return }
            out.append(Suggestion(kind: kind, text: t, detail: detail, replaceStart: start, replaceEnd: caretByte))
        }

        let full = String(decoding: bytes, as: UTF8.self)
        let openEnvs = openEnvironments(in: String(decoding: before, as: UTF8.self))

        if let argCommand {
            let pool: [(Kind, [String])]
            switch argCommand {
            case "begin", "end": pool = [(.environment, (argCommand == "end" ? openEnvs.reversed() : []) + names(after: "\\begin{", in: full))]
            case "ref", "eqref", "pageref": pool = [(.reference, names(after: "\\label{", in: full))]
            case "cite", "citep", "citet": pool = [(.citation, names(after: "\\cite{", in: full))]
            default: pool = []
            }
            for (kind, list) in pool { for n in list where n.hasPrefix(prefix) { add(kind, n, "seen in document") } }
            return Array(out.prefix(limit))
        }

        if isCommand {
            for env in openEnvs.reversed() { let t = "\\end{\(env)}"; if t.hasPrefix(prefix) { add(.environmentClose, t, "closes open \\begin{\(env)}") } }
            for c in commands(in: full) where c.hasPrefix(prefix) && c != prefix { add(.command, c, "typed elsewhere in this document") }
            return Array(out.prefix(limit))
        }
        return []
    }

    static func isWordByte(_ b: UInt8) -> Bool {
        (b >= 0x41 && b <= 0x5A) || (b >= 0x61 && b <= 0x7A) || (b >= 0x30 && b <= 0x39) || b == 0x2A || b == 0x3A || b == 0x2D || b == 0x5F
    }

    /// Environments opened before the caret and not yet closed, in order.
    public static func openEnvironments(in text: String) -> [String] {
        var stack: [String] = []
        for (cmd, name) in commandArguments(in: text, commands: ["begin", "end"]) {
            if cmd == "begin" { stack.append(name) } else if let i = stack.lastIndex(of: name) { stack.remove(at: i) }
        }
        return stack
    }

    static func names(after marker: String, in text: String) -> [String] {
        var out: [String] = []
        var search = text.startIndex
        while let r = text.range(of: marker, range: search..<text.endIndex) {
            if let close = text[r.upperBound...].firstIndex(of: "}") {
                let n = String(text[r.upperBound..<close])
                if !n.isEmpty, !out.contains(n) { out.append(n) }
                search = close
            } else { break }
        }
        return out
    }

    static func commandArguments(in text: String, commands: [String]) -> [(String, String)] {
        var out: [(String, String)] = []
        for c in commands {
            for n in names(after: "\\\(c){", in: text) { out.append((c, n)) }
        }
        // Keep document order by re-scanning positions.
        return text.indices.compactMap { i -> (String.Index, (String, String))? in
            guard text[i] == "\\" else { return nil }
            for (c, n) in out where text[i...].hasPrefix("\\\(c){\(n)}") { return (i, (c, n)) }
            return nil
        }.sorted { $0.0 < $1.0 }.map { $0.1 }
    }

    /// Distinct `\command` tokens in the document, most frequent first.
    public static func commands(in text: String) -> [String] {
        var counts: [String: Int] = [:]
        var order: [String] = []
        let bytes = Array(text.utf8)
        var i = 0
        while i < bytes.count {
            if bytes[i] == 0x5C {
                var j = i + 1
                while j < bytes.count, (bytes[j] >= 0x41 && bytes[j] <= 0x5A) || (bytes[j] >= 0x61 && bytes[j] <= 0x7A) { j += 1 }
                if j > i + 1 {
                    let t = String(decoding: bytes[i..<j], as: UTF8.self)
                    if counts[t] == nil { order.append(t) }
                    counts[t, default: 0] += 1
                }
                i = max(j, i + 1)
            } else { i += 1 }
        }
        return order.sorted { (counts[$0]!, $1) > (counts[$1]!, $0) }
    }
}
