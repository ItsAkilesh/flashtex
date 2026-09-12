import Foundation
import FlashTeXProtocol

/// Keyboard error navigation for the editor, as a pure model the shell and
/// VoiceOver share: diagnostics are visited in document order, next/previous
/// wrap around the document, and every stop is announced as "n of m" with its
/// severity, line, message and recovery line. Items carry the diagnostic's
/// stable identity (the shell's `EditorDiagnostics.Mark.id`) so the underline
/// the caret lands on, the announcement and the diagnostics list row agree.
public enum EditorDiagnosticNavigation {
    public struct Item: Equatable {
        /// Stable diagnostic identity (result id + index + original byte span).
        public var id: String
        /// UTF-16 range in the current editor text.
        public var nsRange: NSRange
        public var severity: RuntimeV1.Severity
        public var message: String
        /// "recovery: …" / "no provisional rendering", or nil.
        public var recoveryLine: String?

        public init(id: String, nsRange: NSRange, severity: RuntimeV1.Severity, message: String, recoveryLine: String?) {
            self.id = id; self.nsRange = nsRange; self.severity = severity; self.message = message; self.recoveryLine = recoveryLine
        }
    }

    /// One navigation stop.
    public struct Step: Equatable {
        public var item: Item
        /// 1-based position in document order.
        public var ordinal: Int
        public var total: Int
        /// True when the step crossed the start or end of the document.
        public var wrapped: Bool
        /// 1-based line of the item's start when the caller could resolve it.
        public var line: Int?

        /// "Error 2 of 5, line 12: message — recovery: … (wrapped to start)".
        public var announcement: String {
            var s = (item.severity == .error ? "Error" : "Warning") + " \(ordinal) of \(total)"
            if let line { s += ", line \(line)" }
            s += ": " + item.message
            if let r = item.recoveryLine { s += " — " + r }
            if wrapped { s += ordinal == 1 ? " (wrapped to start)" : " (wrapped to end)" }
            return s
        }
    }

    /// Document order: by start offset, then longer ranges first (an enclosing
    /// error before the warning inside it), then the caller's order. Stable.
    public static func ordered(_ items: [Item]) -> [Item] {
        items.enumerated().sorted { a, b in
            if a.element.nsRange.location != b.element.nsRange.location {
                return a.element.nsRange.location < b.element.nsRange.location
            }
            if a.element.nsRange.length != b.element.nsRange.length {
                return a.element.nsRange.length > b.element.nsRange.length
            }
            return a.offset < b.offset
        }.map(\.element)
    }

    /// "3 errors, 1 warning" summary for the rotor/status line; "no diagnostics" when empty.
    public static func summary(_ items: [Item]) -> String {
        let e = items.filter { $0.severity == .error }.count, w = items.count - e
        guard !items.isEmpty else { return "no diagnostics" }
        var parts: [String] = []
        if e > 0 { parts.append("\(e) error\(e == 1 ? "" : "s")") }
        if w > 0 { parts.append("\(w) warning\(w == 1 ? "" : "s")") }
        return parts.joined(separator: ", ")
    }

    /// The next stop after the caret (`forward`) or the previous one, wrapping.
    /// When `currentID` names an item, the step is taken from that item's
    /// ordinal, so items sharing a start offset are each visited once; else
    /// the next item starting strictly after the caret (previous: strictly
    /// before). Nil when `items` is empty. `items` need not be ordered.
    public static func step(_ items: [Item], fromUTF16 caret: Int, forward: Bool, currentID: String? = nil,
                            lineOf: (Int) -> Int? = { _ in nil }) -> Step? {
        let ordered = ordered(items)
        guard !ordered.isEmpty else { return nil }
        let n = ordered.count
        var index: Int
        var wrapped = false
        if let currentID, let cur = ordered.firstIndex(where: { $0.id == currentID }) {
            index = cur + (forward ? 1 : -1)
        } else if forward {
            index = ordered.firstIndex { $0.nsRange.location > caret } ?? n
        } else {
            index = ordered.lastIndex { $0.nsRange.location < caret } ?? -1
        }
        if index >= n { index = 0; wrapped = true }
        if index < 0 { index = n - 1; wrapped = true }
        let item = ordered[index]
        return Step(item: item, ordinal: index + 1, total: n, wrapped: wrapped, line: lineOf(item.nsRange.location))
    }

    /// The stop for the item the caret is in (a caret at an item's end counts),
    /// for announcing the diagnostic under the caret without moving. Nil when clear.
    public static func current(_ items: [Item], atUTF16 caret: Int, lineOf: (Int) -> Int? = { _ in nil }) -> Step? {
        let ordered = ordered(items)
        guard let index = ordered.firstIndex(where: { $0.nsRange.location <= caret && caret <= NSMaxRange($0.nsRange) })
        else { return nil }
        let item = ordered[index]
        return Step(item: item, ordinal: index + 1, total: ordered.count, wrapped: false, line: lineOf(item.nsRange.location))
    }
}
