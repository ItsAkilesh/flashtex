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
        let region = changedRegion(from: old, to: new)
        let delta = new.utf8.count - old.utf8.count
        if end <= region.startByte { return .rebased(start: start, end: end) }
        if start >= region.oldEndByte { return .rebased(start: start + delta, end: end + delta) }
        return .overlapsEdit
    }

    /// The single byte region that differs between `old` and `new`: bytes
    /// `startByte..<oldEndByte` of `old` were replaced by `replacement`
    /// (= `new` bytes `startByte..<newEndByte`). Computed from the common
    /// prefix/suffix, then widened outward so both ends sit on UTF-8 scalar
    /// boundaries — the bridge rejects offsets inside a multi-byte scalar.
    /// Identical strings yield an empty region at the end of the text.
    public struct ChangedRegion: Equatable {
        public var startByte: Int
        public var oldEndByte: Int
        public var newEndByte: Int
        public var replacement: String
        public init(startByte: Int, oldEndByte: Int, newEndByte: Int, replacement: String) {
            self.startByte = startByte; self.oldEndByte = oldEndByte; self.newEndByte = newEndByte; self.replacement = replacement
        }
    }

    public static func changedRegion(from old: String, to new: String) -> ChangedRegion {
        let o = Array(old.utf8), n = Array(new.utf8)
        func isContinuation(_ b: UInt8) -> Bool { b & 0xC0 == 0x80 }
        var prefix = 0
        while prefix < o.count, prefix < n.count, o[prefix] == n[prefix] { prefix += 1 }
        while prefix > 0, (prefix < o.count && isContinuation(o[prefix])) || (prefix < n.count && isContinuation(n[prefix])) {
            prefix -= 1
        }
        var suffix = 0
        while suffix < o.count - prefix, suffix < n.count - prefix,
              o[o.count - 1 - suffix] == n[n.count - 1 - suffix] { suffix += 1 }
        // The suffix bytes are identical in both strings, so one boundary check covers both.
        while suffix > 0, isContinuation(o[o.count - suffix]) { suffix -= 1 }
        let oldEnd = o.count - suffix, newEnd = n.count - suffix
        let replacement = String(decoding: n[prefix..<newEnd], as: UTF8.self)
        return ChangedRegion(startByte: prefix, oldEndByte: oldEnd, newEndByte: newEnd, replacement: replacement)
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
