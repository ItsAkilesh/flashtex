import Foundation

/// Conversion between contract UTF-8 byte offsets and Swift/AppKit editing
/// coordinates. AppKit text views select in UTF-16 units, so the conversion must
/// be explicit; offsets that are out of range or land inside a scalar are rejected.
public extension String {
    /// Returns the `String.Index` range for a UTF-8 byte range, or nil if either
    /// offset is out of bounds, reversed, or not on a scalar boundary.
    func range(utf8Bytes range: RuntimeV1.SourceRange) -> Range<String.Index>? {
        rangeOfUTF8(start: range.startByte, end: range.endByte)
    }

    func rangeOfUTF8(start: Int, end: Int) -> Range<String.Index>? {
        guard start >= 0, end >= start, end <= utf8.count else { return nil }
        let u = utf8
        guard let s = u.index(u.startIndex, offsetBy: start, limitedBy: u.endIndex),
              let e = u.index(u.startIndex, offsetBy: end, limitedBy: u.endIndex)
        else { return nil }
        // Reject offsets inside a multi-byte scalar.
        guard s.samePosition(in: unicodeScalars) != nil,
              e.samePosition(in: unicodeScalars) != nil
        else { return nil }
        return s..<e
    }

    /// UTF-16 `NSRange` for a UTF-8 byte range, for `NSTextView` selection.
    func nsRange(utf8Bytes range: RuntimeV1.SourceRange) -> NSRange? {
        guard let r = self.range(utf8Bytes: range) else { return nil }
        return NSRange(r, in: self)
    }

    /// UTF-8 byte range for a UTF-16 `NSRange` (reverse mapping for the editor).
    func utf8ByteRange(of nsRange: NSRange) -> (start: Int, end: Int)? {
        guard let r = Range(nsRange, in: self) else { return nil }
        let start = utf8.distance(from: utf8.startIndex, to: r.lowerBound)
        let end = utf8.distance(from: utf8.startIndex, to: r.upperBound)
        return (start, end)
    }
}
