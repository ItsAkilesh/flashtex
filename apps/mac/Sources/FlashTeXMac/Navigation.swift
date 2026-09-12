import AppKit
import SwiftUI
import FlashTeXProtocol

/// Source-aware navigation. The text functions are pure and return UTF-8 byte
/// ranges into the current buffer (no compile result involved, so they are
/// never stale). Diagnostic navigation goes through `ShellModel.navigate`, so
/// a diagnostic whose source overlaps an edit made since the compile is
/// refused ("recompile to navigate") rather than mapped onto the wrong text.
enum Navigation {
    struct ByteRange: Equatable {
        let start: Int
        let end: Int
    }

    /// One `\name{arg}` occurrence: `range` spans the backslash through `}`,
    /// `argRange` the argument text.
    struct CommandUse: Equatable {
        let name: String
        let arg: String
        let range: ByteRange
        let argRange: ByteRange
    }

    static let referenceCommands: Set<String> = ["ref", "eqref", "pageref", "autoref"]

    /// Every `\begin{…}`, `\end{…}`, `\label{…}`, and reference command with a
    /// complete braced argument, in document order.
    static func commandUses(in text: String) -> [CommandUse] {
        var out: [CommandUse] = []
        Completion.withBytes(text) { b in
            guard let p = b.baseAddress else { return }
            let n = b.count
            let table = Completion.wordByteClass
            var i = Completion.nextBackslash(p, from: 0, count: n)
            while i < n {
                var j = i + 1
                while j < n, table[Int(p[j])] == 1 { j += 1 }
                guard j > i + 1 else { i = Completion.nextBackslash(p, from: j + 1, count: n); continue }
                let nameBytes = UnsafeBufferPointer(start: p + i + 1, count: j - i - 1)
                let interesting = Completion.bytes(nameBytes, equal: "begin") || Completion.bytes(nameBytes, equal: "end")
                    || Completion.bytes(nameBytes, equal: "label") || Completion.bytes(nameBytes, equal: "ref")
                    || Completion.bytes(nameBytes, equal: "eqref") || Completion.bytes(nameBytes, equal: "pageref")
                    || Completion.bytes(nameBytes, equal: "autoref")
                guard interesting, j < n, p[j] == UInt8(ascii: "{") else { i = Completion.nextBackslash(p, from: j, count: n); continue }
                var k = j + 1
                while k < n, p[k] != UInt8(ascii: "}"), p[k] != UInt8(ascii: "{"), p[k] != Completion.backslash,
                      p[k] != UInt8(ascii: "\n") { k += 1 }
                guard k < n, p[k] == UInt8(ascii: "}") else { i = Completion.nextBackslash(p, from: j, count: n); continue }
                out.append(CommandUse(name: String(decoding: nameBytes, as: UTF8.self),
                                      arg: String(decoding: UnsafeBufferPointer(start: p + j + 1, count: k - j - 1), as: UTF8.self),
                                      range: ByteRange(start: i, end: k + 1), argRange: ByteRange(start: j + 1, end: k)))
                i = Completion.nextBackslash(p, from: k + 1, count: n)
            }
        }
        return out
    }

    enum Target: Equatable {
        case found(ByteRange, note: String)
        case notFound(String)
    }

    /// Counterpart of the command under the caret (`caretByte`, UTF-8):
    /// `\ref{X}` → its `\label{X}`; `\label{X}` → the next reference to it
    /// (wrapping); `\begin{X}` ↔ `\end{X}` honouring nesting of the same name.
    static func matchingRange(in text: String, caretByte: Int) -> Target {
        let uses = commandUses(in: text)
        guard let here = uses.firstIndex(where: { $0.range.start <= caretByte && caretByte <= $0.range.end }) else {
            return .notFound("Caret is not inside \\begin, \\end, \\label, or a \\ref-style command.")
        }
        let use = uses[here]
        switch use.name {
        case "begin":
            var depth = 0
            for other in uses[(here + 1)...] where other.arg == use.arg {
                if other.name == "begin" { depth += 1 }
                else if other.name == "end" {
                    if depth == 0 { return .found(other.range, note: "Matched \\begin{\(use.arg)} → \\end{\(use.arg)} at byte \(other.range.start).") }
                    depth -= 1
                }
            }
            return .notFound("\\begin{\(use.arg)} at byte \(use.range.start) has no matching \\end{\(use.arg)}.")
        case "end":
            var depth = 0
            for other in uses[..<here].reversed() where other.arg == use.arg {
                if other.name == "end" { depth += 1 }
                else if other.name == "begin" {
                    if depth == 0 { return .found(other.range, note: "Matched \\end{\(use.arg)} → \\begin{\(use.arg)} at byte \(other.range.start).") }
                    depth -= 1
                }
            }
            return .notFound("\\end{\(use.arg)} at byte \(use.range.start) has no matching \\begin{\(use.arg)}.")
        case "label":
            let refs = uses.filter { referenceCommands.contains($0.name) && $0.arg == use.arg }
            guard !refs.isEmpty else { return .notFound("No reference to label \(use.arg) in this document.") }
            let next = refs.first { $0.range.start > use.range.start } ?? refs[0]
            let index = refs.firstIndex(of: next)! + 1
            return .found(next.range, note: "Reference \(index) of \(refs.count) to label \(use.arg) at byte \(next.range.start).")
        default:
            guard let label = uses.first(where: { $0.name == "label" && $0.arg == use.arg }) else {
                return .notFound("No \\label{\(use.arg)} in this document.")
            }
            return .found(label.range, note: "Definition of \(use.arg): \\label at byte \(label.range.start).")
        }
    }

    /// Diagnostics with a source in `path`, ordered by their start offset in
    /// the current buffer (rebased across edits when possible; a diagnostic
    /// whose range overlaps an edit keeps its compiled offset for ordering and
    /// is refused by `ShellModel.navigate` when chosen).
    struct Stop: Equatable {
        let index: Int          // index into `result.diagnostics`
        let diagnostic: RuntimeV1.Diagnostic
        let source: RuntimeV1.SourceRange
        let currentStart: Int   // best-known start byte in the current buffer
    }

    static func stops(in result: RuntimeV1.CompileResult, path: String,
                      compiledText: String?, currentText: String) -> [Stop] {
        var out: [Stop] = []
        for (i, d) in result.diagnostics.enumerated() {
            guard let s = d.source, s.path == path else { continue }
            var start = s.startByte
            if let compiledText, compiledText != currentText,
               let rebased = SourceMapping.rebase(s, from: compiledText, to: currentText, expectedText: nil) {
                start = rebased.startByte
            }
            out.append(Stop(index: i, diagnostic: d, source: s, currentStart: start))
        }
        return out.sorted { a, b in a.currentStart != b.currentStart ? a.currentStart < b.currentStart : a.index < b.index }
    }

    /// Next stop strictly after `caretByte`, wrapping to the first; `forward: false`
    /// gives the previous one, wrapping to the last.
    static func nextStop(_ stops: [Stop], from caretByte: Int, forward: Bool) -> Stop? {
        guard !stops.isEmpty else { return nil }
        if forward { return stops.first { $0.currentStart > caretByte } ?? stops[0] }
        return stops.last { $0.currentStart < caretByte } ?? stops[stops.count - 1]
    }
}

// MARK: - model actions

extension ShellModel {
    /// ⌘⇧D: select the counterpart of the `\begin`/`\end`/`\label`/`\ref`
    /// under the caret in the current buffer.
    func goToMatching() {
        let text = activeText
        guard let caretByte else {
            navigationNote = "Caret position \(caretUTF16) is not valid in \(activePath)."
            return
        }
        switch Navigation.matchingRange(in: text, caretByte: caretByte) {
        case .notFound(let why):
            navigationNote = why
        case .found(let range, let note):
            guard let ns = text.nsRange(utf8Bytes: .init(path: activePath, startByte: range.start, endByte: range.end)) else {
                navigationNote = "Bytes \(range.start)..<\(range.end) are not a valid range in \(activePath)."
                return
            }
            selection = .init(path: activePath, nsRange: ns, token: (selection?.token ?? 0) + 1)
            caretUTF16 = ns.location
            navigationNote = note
        }
    }

    /// ⌘⇧] / ⌘⇧[: cycle through the result's diagnostics that have a source
    /// in the active document. Uses `navigate(to:expectedText:)`, so an edited
    /// span is refused with "recompile to navigate".
    func goToDiagnostic(forward: Bool) {
        guard let result else {
            navigationNote = "No compile result loaded; nothing to navigate to."
            return
        }
        let stops = Navigation.stops(in: result, path: activePath,
                                     compiledText: compiledDocuments[activePath], currentText: activeText)
        guard let stop = Navigation.nextStop(stops, from: caretByte ?? 0, forward: forward) else {
            let total = result.diagnostics.count
            navigationNote = total == 0
                ? "Revision \(result.revision) has no diagnostics."
                : "None of the \(total) diagnostic\(total == 1 ? "" : "s") has a source in \(activePath)."
            return
        }
        let before = selection
        navigate(to: stop.source, expectedText: nil)
        if let sel = selection, sel != before {
            caretUTF16 = sel.nsRange.location
            let position = stops.firstIndex(of: stop).map { $0 + 1 } ?? 0
            navigationNote = "Diagnostic \(position) of \(stops.count) (\(stop.diagnostic.severity.rawValue)): \(stop.diagnostic.message)"
        }
    }

    /// ⌘⇧J: select the full source span of the preview item under the caret
    /// (so the preview's caret highlight and page scroll follow) and say where
    /// it landed.
    func revealCaretInPreview() {
        guard let result else {
            navigationNote = "No compile result loaded; the caret maps to no preview item."
            return
        }
        guard let byte = caretByte else {
            navigationNote = "Caret position \(caretUTF16) is not valid in \(activePath)."
            return
        }
        let hits = CaretSync.itemsContaining(byte: byte, path: activePath, in: result)
        guard let hit = hits.first,
              let page = result.pages.first(where: { $0.number == hit.page }),
              hit.index < page.items.count,
              case .text(let item) = page.items[hit.index], let source = item.source else {
            navigationNote = "Caret byte \(byte) is inside no preview item" + (previewIsStale ? " (preview is from an older revision)." : ".")
            return
        }
        let before = selection
        navigate(to: source, expectedText: item.text)
        if let sel = selection, sel != before {
            caretUTF16 = sel.nsRange.location
            navigationNote = "Caret is in page \(hit.page) item \(hit.index) “\(item.text)”" + (hits.count > 1 ? " (+\(hits.count - 1) more)." : ".")
        }
    }
}

// MARK: - menu

/// `Navigate` menu, added from `FlashTeXMacApp` with one line.
struct NavigationCommands: Commands {
    var model: ShellModel

    var body: some Commands {
        CommandMenu("Navigate") {
            Button("Go to Matching \\begin/\\end or \\label/\\ref") { model.goToMatching() }
                .keyboardShortcut("d", modifiers: [.command, .shift])
            Divider()
            Button("Next Diagnostic") { model.goToDiagnostic(forward: true) }
                .keyboardShortcut("]", modifiers: [.command, .shift])
                .disabled(model.result == nil)
            Button("Previous Diagnostic") { model.goToDiagnostic(forward: false) }
                .keyboardShortcut("[", modifiers: [.command, .shift])
                .disabled(model.result == nil)
            Divider()
            Button("Reveal Caret in Preview") { model.revealCaretInPreview() }
                .keyboardShortcut("j", modifiers: [.command, .shift])
                .disabled(model.result == nil)
        }
    }
}
