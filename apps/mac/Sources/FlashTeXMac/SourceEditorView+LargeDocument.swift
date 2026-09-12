import AppKit

/// Large-document guards for the editor (lane mac-editor-a11y-2).
///
/// Measured on a 560 KB / 10 316-line prose buffer (LargeDocumentEditorTests,
/// debug build): a keystroke cost 5.5 ms CPU before the delegate's own binding
/// work (storage edit 0.26 ms, model update 0.06 ms). The rest was O(n) string
/// work done for the delimiter highlight on every keystroke — the native
/// UTF-8 conversion of the storage (1.2 ms) and the UTF-16 breadcrumbs Swift
/// builds the first time a fresh String is indexed by UTF-16 offset (1.3 ms) —
/// and it ran twice per keystroke because AppKit posts the selection change
/// before the text change. A prose keystroke is almost never adjacent to a
/// delimiter, so an O(1) look at the storage decides whether any of that is
/// needed.
extension SourceEditorView.BraceMatcher {
    /// UTF-16 units that can start or end a highlighted pair: `{ } [ ] $`.
    static let delimiterUnits: Set<unichar> = [0x7B, 0x7D, 0x5B, 0x5D, 0x24]

    /// Whether a delimiter pair can exist around `caret`: the UTF-16 unit just
    /// before or just at the caret is one of `{}[]$`. Reads the text storage's
    /// own NSString (no bridging, no copy), so this is O(1) on any buffer size;
    /// `match(in:caretUTF16:)` is only worth calling when this is true.
    static func delimiterAdjacent(in storage: NSTextStorage?, caretUTF16 caret: Int) -> Bool {
        guard let storage else { return false }
        let ns = storage.string as NSString
        let length = ns.length
        guard caret >= 0, caret <= length else { return false }
        if caret > 0, delimiterUnits.contains(ns.character(at: caret - 1)) { return true }
        if caret < length, delimiterUnits.contains(ns.character(at: caret)) { return true }
        return false
    }
}

/// Bounded selection announcement (lane mac-editor-a11y-3).
///
/// A selection change is announced as "Selected N characters, line a column b
/// to line c column d". The character count is a grapheme walk over the
/// selected text — O(n) in the selection, 2.6 ms for a 560 KB select-all
/// (LargeDocumentEditorTests, debug build) — and it is the only O(n) work the
/// coordinator does for a large selection: the delimiter highlight is already
/// skipped for any non-empty selection, and the line/column lookups are
/// `memchr` over the prefix (0.27 ms at the end of that buffer). The rest of a
/// select-all (33–250 ms measured) is AppKit/TextKit 1 laying out the
/// selection and is not reachable from here.
///
/// Above `largeSelectionAnnouncementLimit` the announcement names the line
/// span instead of the character count, so the announcement cost is bounded
/// by the limit (≈0.3 ms) whatever the selection size. Both forms stay exact.
extension SourceEditorView {
    /// UTF-16 length above which a selection is announced by its line span.
    /// 64 K units is beyond any selection that fits on a screen and costs a
    /// ~0.3 ms grapheme walk at most.
    static let largeSelectionAnnouncementLimit = 65_536

    /// `selectionAnnouncement(text:range:)` up to the limit; above it,
    /// "Selected N lines, line a column b to line c column d" where N counts
    /// the lines the selection covers (a selection ending at column 1 of a
    /// line does not count that line). Nil for an invalid range, as the
    /// unbounded form.
    static func boundedSelectionAnnouncement(text: String, range: NSRange) -> String? {
        guard range.length > largeSelectionAnnouncementLimit else { return selectionAnnouncement(text: text, range: range) }
        guard range.location >= 0, let start = lineColumn(text: text, utf16: range.location),
              let end = lineColumn(text: text, utf16: NSMaxRange(range)) else { return nil }
        let lines = end.line - start.line + (end.column > 1 ? 1 : 0)
        return "Selected \(lines) lines, line \(start.line) column \(start.column) to line \(end.line) column \(end.column)"
    }
}
