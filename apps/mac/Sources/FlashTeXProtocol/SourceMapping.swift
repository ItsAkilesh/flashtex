import Foundation

/// Rebases UTF-8 byte ranges from the text a `compile_result` was produced for
/// onto the current editor text. Uses the common prefix/suffix of the two byte
/// strings: ranges entirely before the changed region keep their offsets, ranges
/// entirely after it shift by the length delta, and anything overlapping the
/// changed region is refused. Multiple edits collapse into one region, which is
/// conservative — it never maps a range onto different text.
public enum SourceMapping {
    public enum Outcome: Equatable {
        case unchanged
        case rebased(start: Int, end: Int)
        case overlapsEdit
    }

    public static func rebase(start: Int, end: Int, from old: String, to new: String) -> Outcome {
        if old == new { return .unchanged }
        let o = Array(old.utf8), n = Array(new.utf8)
        var prefix = 0
        while prefix < o.count, prefix < n.count, o[prefix] == n[prefix] { prefix += 1 }
        var suffix = 0
        while suffix < o.count - prefix, suffix < n.count - prefix,
              o[o.count - 1 - suffix] == n[n.count - 1 - suffix] { suffix += 1 }
        let oldChangedEnd = o.count - suffix   // exclusive, in old bytes
        let delta = n.count - o.count
        if end <= prefix { return .rebased(start: start, end: end) }
        if start >= oldChangedEnd { return .rebased(start: start + delta, end: end + delta) }
        return .overlapsEdit
    }

    /// Rebases and additionally checks that the mapped bytes still spell
    /// `expectedText` when one is known (an item's `text`).
    public static func rebase(_ range: RuntimeV1.SourceRange, from old: String, to new: String,
                              expectedText: String?) -> RuntimeV1.SourceRange? {
        switch rebase(start: range.startByte, end: range.endByte, from: old, to: new) {
        case .unchanged:
            return range
        case .rebased(let s, let e):
            let mapped = RuntimeV1.SourceRange(path: range.path, startByte: s, endByte: e)
            if let expectedText, let r = new.range(utf8Bytes: mapped), String(new[r]) != expectedText { return nil }
            return mapped
        case .overlapsEdit:
            return nil
        }
    }
}
