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
