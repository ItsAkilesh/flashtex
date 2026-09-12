import AppKit
import SwiftUI
import FlashTeXProtocol
import FlashTeXAccessibility

/// Source-aware navigation. The text functions are pure and return UTF-8 byte
/// ranges into the current buffers (no compile result involved, so they are
/// never stale). Result-backed navigation (preview click, diagnostics, caret
/// reveal) goes through `ShellModel.navigateExactly`, which maps runtime-v1
/// UTF-8 byte spans onto the editor's UTF-16 selection exactly: a span whose
/// bytes were edited since the compile is refused with an explanation, a span
/// is never split inside a scalar or a composed character sequence, and a span
/// in another open document switches the active document.
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
    private static let interestingCommands: Set<String> = referenceCommands.union(["begin", "end", "label"])

    /// Every `\begin{…}`, `\end{…}`, `\label{…}`, and reference command with a
    /// complete braced argument, in document order. Commands inside a `%`
    /// comment (to the end of the line) are skipped; `\%` and `\\` are escapes.
    static func commandUses(in text: String) -> [CommandUse] {
        var out: [CommandUse] = []
        Completion.withBytes(text) { b in
            guard let p = b.baseAddress else { return }
            let n = b.count
            let table = Completion.wordByteClass
            let backslash = Completion.backslash, percent = UInt8(ascii: "%"), newline = UInt8(ascii: "\n")
            let open = UInt8(ascii: "{"), close = UInt8(ascii: "}")
            var i = 0
            while i < n {
                let c = p[i]
                if c == percent {
                    while i < n, p[i] != newline { i += 1 }
                    continue
                }
                guard c == backslash else { i += 1; continue }
                var j = i + 1
                while j < n, table[Int(p[j])] == 1 { j += 1 }
                guard j > i + 1 else { i = min(n, i + 2); continue } // `\%`, `\\`, `\{`, trailing `\`
                let nameBytes = UnsafeBufferPointer(start: p + i + 1, count: j - i - 1)
                let name = String(decoding: nameBytes, as: UTF8.self)
                guard interestingCommands.contains(name), j < n, p[j] == open else { i = j; continue }
                var k = j + 1
                while k < n, p[k] != close, p[k] != open, p[k] != backslash, p[k] != newline, p[k] != percent { k += 1 }
                guard k < n, p[k] == close else { i = j; continue }
                out.append(CommandUse(name: name,
                                      arg: String(decoding: UnsafeBufferPointer(start: p + j + 1, count: k - j - 1), as: UTF8.self),
                                      range: ByteRange(start: i, end: k + 1), argRange: ByteRange(start: j + 1, end: k)))
                i = k + 1
            }
        }
        return out
    }

    enum Target: Equatable {
        case found(ByteRange, note: String)
        case notFound(String)
    }

    /// Counterpart of the command under the caret (`caretByte`, UTF-8) within
    /// one document: `\ref{X}` → its `\label{X}`; `\label{X}` → the next
    /// reference to it (wrapping); `\begin{X}` ↔ `\end{X}` honouring nesting
    /// of the same name. See `matchingRange(in:activePath:caretByte:)` for
    /// labels and references that live in other open documents.
    static func matchingRange(in text: String, caretByte: Int) -> Target {
        switch matchingRange(in: [.init(path: "", text: text)], activePath: "", caretByte: caretByte) {
        case .found(_, let range, let note): return .found(range, note: note)
        case .notFound(let why): return .notFound(why)
        }
    }

    enum DocumentTarget: Equatable {
        case found(path: String, ByteRange, note: String)
        case notFound(String)
    }

    /// Multi-document counterpart lookup. `\begin`/`\end` match within the
    /// active document. `\ref{X}` finds `\label{X}` in the active document
    /// first, then the other open documents in project order. `\label{X}`
    /// cycles through every reference to it: those after the caret in the
    /// active document, then the following documents, wrapping around.
    static func matchingRange(in documents: [RuntimeV1.Document], activePath: String, caretByte: Int) -> DocumentTarget {
        guard let active = documents.firstIndex(where: { $0.path == activePath }) else {
            return .notFound("No open document named \(activePath).")
        }
        let uses = commandUses(in: documents[active].text)
        // A caret at the byte where one command ends and the next begins
        // (`\end{a}\begin{b}`) belongs to the one that starts there.
        let here = uses.firstIndex(where: { $0.range.start == caretByte })
            ?? uses.firstIndex(where: { $0.range.start <= caretByte && caretByte <= $0.range.end })
        guard let here else {
            return .notFound("Caret is not inside \\begin, \\end, \\label, or a \\ref-style command.")
        }
        let use = uses[here]
        let elsewhere = documents.count > 1 ? " or any open document" : ""
        switch use.name {
        case "begin":
            var depth = 0
            for other in uses[(here + 1)...] where other.arg == use.arg {
                if other.name == "begin" { depth += 1 }
                else if other.name == "end" {
                    if depth == 0 { return .found(path: activePath, other.range, note: "Matched \\begin{\(use.arg)} → \\end{\(use.arg)} at byte \(other.range.start).") }
                    depth -= 1
                }
            }
            return .notFound("\\begin{\(use.arg)} at byte \(use.range.start) has no matching \\end{\(use.arg)}.")
        case "end":
            var depth = 0
            for other in uses[..<here].reversed() where other.arg == use.arg {
                if other.name == "end" { depth += 1 }
                else if other.name == "begin" {
                    if depth == 0 { return .found(path: activePath, other.range, note: "Matched \\end{\(use.arg)} → \\begin{\(use.arg)} at byte \(other.range.start).") }
                    depth -= 1
                }
            }
            return .notFound("\\end{\(use.arg)} at byte \(use.range.start) has no matching \\begin{\(use.arg)}.")
        case "label":
            // All references in project order, starting with the active document.
            var refs: [(path: String, use: CommandUse)] = []
            for offset in 0..<documents.count {
                let doc = documents[(active + offset) % documents.count]
                let docUses = offset == 0 ? uses : commandUses(in: doc.text)
                for r in docUses where referenceCommands.contains(r.name) && r.arg == use.arg {
                    refs.append((doc.path, r))
                }
            }
            guard !refs.isEmpty else { return .notFound("No reference to label \(use.arg) in this document\(elsewhere).") }
            let after = refs.firstIndex { $0.path != activePath || $0.use.range.start > use.range.start }
            let next = refs[after ?? 0]
            let index = (after ?? 0) + 1
            let place = next.path == activePath ? "" : " in \(next.path)"
            return .found(path: next.path, next.use.range,
                          note: "Reference \(index) of \(refs.count) to label \(use.arg) at byte \(next.use.range.start)\(place).")
        default:
            for offset in 0..<documents.count {
                let doc = documents[(active + offset) % documents.count]
                let docUses = offset == 0 ? uses : commandUses(in: doc.text)
                if let label = docUses.first(where: { $0.name == "label" && $0.arg == use.arg }) {
                    let place = doc.path == activePath ? "" : " in \(doc.path)"
                    return .found(path: doc.path, label.range, note: "Definition of \(use.arg): \\label at byte \(label.range.start)\(place).")
                }
            }
            return .notFound("No \\label{\(use.arg)} in this document\(elsewhere).")
        }
    }

    // MARK: - byte span → editor selection

    enum RangeMapping: Equatable {
        /// `widenedFrom` is set when the span started or ended inside a
        /// composed character sequence (`e` + U+0301, a ZWJ emoji, a ligature
        /// glyph is one scalar and never splits) and was extended to cover it.
        case selected(NSRange, widenedFrom: NSRange?)
        case refused(String)
    }

    /// Exact UTF-16 selection for UTF-8 bytes `start..<end` of `text`.
    /// Refused when out of range, reversed, or inside a multi-byte scalar;
    /// widened outward to whole composed character sequences (the units the
    /// text view selects and moves the caret by), so the selection is never a
    /// split cluster. An empty span at a cluster-interior position is moved
    /// to the start of that cluster and stays empty.
    static func editorRange(start: Int, end: Int, in text: String, path: String = "") -> RangeMapping {
        let byteCount = text.utf8.count
        let label = path.isEmpty ? "" : " in \(path)"
        guard start >= 0, end >= start, end <= byteCount else {
            return .refused("Bytes \(start)..<\(end) are not a valid range\(label) (buffer is \(byteCount) bytes).")
        }
        guard let r = text.rangeOfUTF8(start: start, end: end) else {
            return .refused("Bytes \(start)..<\(end)\(label) start or end inside a multi-byte character; refusing to split it.")
        }
        let ns = NSRange(r, in: text)
        let nsText = text as NSString
        let whole: NSRange
        if ns.length == 0 {
            whole = ns.location < nsText.length
                ? NSRange(location: nsText.rangeOfComposedCharacterSequence(at: ns.location).location, length: 0)
                : ns
        } else {
            whole = nsText.rangeOfComposedCharacterSequences(for: ns)
        }
        return .selected(whole, widenedFrom: whole == ns ? nil : ns)
    }
}

// MARK: - model actions

extension ShellModel {
    /// Exact preview → source navigation. Same contract as `navigate(to:expectedText:)`
    /// with these guarantees:
    /// - staleness is decided byte-for-byte (`sameBytes`), so a normalization-only
    ///   edit (é → e + U+0301) is an edit, not a match;
    /// - a span overlapping the edited region is refused, and the note says
    ///   which bytes were edited;
    /// - a rebased span is verified to spell the same bytes it did at compile
    ///   time (item text is informational: generated text such as a section
    ///   number legitimately differs from its source);
    /// - the selection covers whole composed character sequences;
    /// - the active document switches when the span lives in another open one.
    func navigateExactly(to source: RuntimeV1.SourceRange?, expectedText: String? = nil) {
        guard let source else {
            navigationNote = "This item has no source mapping."
            return
        }
        guard let doc = documents.first(where: { $0.path == source.path }) else {
            let open = documents.map(\.path).joined(separator: ", ")
            navigationNote = "No open document named \(source.path) (open: \(open))."
            return
        }
        var start = source.startByte, end = source.endByte
        var notes: [String] = []
        let revision = result?.revision ?? 0
        if let compiled = compiledDocuments[source.path] {
            if !compiled.sameBytes(as: doc.text) {
                let region = SourceMapping.changedRegion(from: compiled, to: doc.text)
                switch SourceMapping.rebase(start: start, end: end, across: region) {
                case .overlapsEdit:
                    navigationNote = "Source for this item was edited since revision \(revision) (bytes \(source.startByte)..<\(source.endByte) of \(source.path) overlap the edit at \(region.startByte)..<\(region.oldEndByte), now \(region.startByte)..<\(region.newEndByte)); recompile to navigate."
                    return
                case .rebased(let s, let e):
                    if s != start { notes.append("rebased from \(start)..<\(end) across edits") }
                    start = s; end = e
                case .unchanged:
                    break
                }
                // The single-region rebase keeps the bytes outside the edit identical;
                // check anyway so a wrong span can never be selected silently.
                guard let was = compiled.rangeOfUTF8(start: source.startByte, end: source.endByte),
                      let now = doc.text.rangeOfUTF8(start: start, end: end),
                      String(compiled[was]).sameBytes(as: String(doc.text[now])) else {
                    navigationNote = "Bytes \(source.startByte)..<\(source.endByte) of \(source.path) no longer spell the compiled text after rebasing to \(start)..<\(end); recompile to navigate."
                    return
                }
            }
        } else if previewIsStale {
            navigationNote = "Buffer edited since revision \(revision) and no compiled text is recorded; recompile to navigate."
            return
        }
        let ns: NSRange
        switch Navigation.editorRange(start: start, end: end, in: doc.text, path: source.path) {
        case .refused(let why):
            navigationNote = why
            return
        case .selected(let range, let widenedFrom):
            ns = range
            if let widenedFrom {
                notes.append("widened from UTF-16 \(widenedFrom.location)..<\(NSMaxRange(widenedFrom)) to whole characters")
            }
        }
        if let expectedText, !(doc.text as NSString).substring(with: ns).sameBytes(as: expectedText) {
            notes.append("“\(expectedText)” is generated from this source")
        }
        if activePath != source.path {
            activePath = source.path
            notes.append("switched to \(source.path)")
        }
        selection = .init(path: source.path, nsRange: ns, token: (selection?.token ?? 0) + 1)
        caretUTF16 = ns.location
        caretLengthUTF16 = ns.length
        navigationNote = "Selected \(source.path) bytes \(start)..<\(end) → UTF-16 \(ns.location)..<\(NSMaxRange(ns))"
            + (notes.isEmpty ? "" : " (" + notes.joined(separator: "; ") + ")")
    }

    /// ⌘⇧D: select the counterpart of the `\begin`/`\end`/`\label`/`\ref`
    /// under the caret; a label or reference in another open document
    /// switches to it.
    func goToMatching() {
        guard let caretByte = CaretSync.byteOffset(ofCaretUTF16: caretUTF16, in: activeText) else {
            navigationNote = "Caret position \(caretUTF16) is not valid in \(activePath)."
            return
        }
        switch Navigation.matchingRange(in: documents, activePath: activePath, caretByte: caretByte) {
        case .notFound(let why):
            navigationNote = why
        case .found(let path, let range, let note):
            guard let text = documents.first(where: { $0.path == path })?.text,
                  case .selected(let ns, _) = Navigation.editorRange(start: range.start, end: range.end, in: text, path: path) else {
                navigationNote = "Bytes \(range.start)..<\(range.end) are not a valid range in \(path)."
                return
            }
            activePath = path
            selection = .init(path: path, nsRange: ns, token: (selection?.token ?? 0) + 1)
            caretUTF16 = ns.location
            caretLengthUTF16 = ns.length
            navigationNote = note
        }
    }

    /// ⌘⇧] / ⌘⇧[: step through the underlined diagnostics (`editorMarkReport`,
    /// exact mark identities, stale spans withheld) of the active document in
    /// document order; past its last mark the step continues into the next
    /// open document with marks (project order, wrapping), switching to it.
    /// Diagnostics under edited text are skipped and counted in the note.
    func goToDiagnostic(forward: Bool) {
        guard let result else {
            navigationNote = "No compile result loaded; nothing to navigate to."
            return
        }
        let report = editorMarkReport
        let here = EditorDiagnostics.step(report.marks, fromUTF16: caretUTF16, forward: forward,
                                          currentID: currentDiagnosticID, in: activeText)
        var chosen: (path: String, step: EditorDiagnosticNavigation.Step, report: EditorDiagnostics.Report)?
        if let here, !here.wrapped || documents.count == 1 {
            chosen = (activePath, here, report)
        } else if let active = documents.firstIndex(where: { $0.path == activePath }), documents.count > 1 {
            // Leaving the active document at either end: the first (or last)
            // mark of the next open document that has one.
            for offset in 1..<documents.count {
                let doc = documents[(active + offset) % documents.count]
                let other = EditorDiagnostics.report(for: result, resultID: resultID, path: doc.path,
                                                     compiledText: compiledDocuments[doc.path], currentText: doc.text)
                if let step = EditorDiagnostics.step(other.marks, fromUTF16: forward ? -1 : Int.max, forward: forward, in: doc.text) {
                    chosen = (doc.path, step, other)
                    break
                }
            }
            if chosen == nil, let here { chosen = (activePath, here, report) }
        }
        guard let chosen else {
            let total = result.diagnostics.count
            let open = documents.count > 1 ? "an open document (\(documents.map(\.path).joined(separator: ", ")))" : activePath
            navigationNote = total == 0 ? "Revision \(result.revision) has no diagnostics."
                : report.staleNote.map { "No diagnostic can be selected: " + $0 + "." }
                ?? "None of the \(total) diagnostic\(total == 1 ? "" : "s") has a source in \(open)."
            return
        }
        let switched = chosen.path != activePath
        activePath = chosen.path
        currentDiagnosticID = chosen.step.item.id
        selection = .init(path: chosen.path, nsRange: chosen.step.item.nsRange, token: (selection?.token ?? 0) + 1)
        caretUTF16 = chosen.step.item.nsRange.location
        caretLengthUTF16 = chosen.step.item.nsRange.length
        navigationNote = chosen.step.announcement
            + (switched ? " (in \(chosen.path))" : "")
            + (chosen.report.staleNote.map { "; " + $0 } ?? "")
    }

    /// ⌘⇧J: select the full source span of the preview item under the caret
    /// (so the preview's caret highlight and page scroll follow) and say where
    /// it landed.
    func revealCaretInPreview() {
        guard let result else {
            navigationNote = "No compile result loaded; the caret maps to no preview item."
            return
        }
        guard let byte = CaretSync.byteOffset(ofCaretUTF16: caretUTF16, in: activeText) else {
            navigationNote = "Caret position \(caretUTF16) is not valid in \(activePath)."
            return
        }
        let hits = caretIndex()?.itemsContaining(byte: byte) ?? []
        guard let hit = hits.first,
              let page = result.pages.first(where: { $0.number == hit.page }),
              hit.index < page.items.count,
              case .text(let item) = page.items[hit.index], let source = item.source else {
            navigationNote = "Caret byte \(byte) is inside no preview item" + (previewIsStale ? " (preview is from an older revision)." : ".")
            return
        }
        let before = selection
        navigateExactly(to: source, expectedText: item.text)
        if let sel = selection, sel != before {
            let pages = Set(hits.map(\.page)).sorted()
            let more = hits.count > 1
                ? " (+\(hits.count - 1) more" + (pages.count > 1 ? ", pages \(pages.map(String.init).joined(separator: ", "))" : "") + ")."
                : "."
            navigationNote = "Caret is in page \(hit.page) item \(hit.index) “\(item.text)”" + more
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
