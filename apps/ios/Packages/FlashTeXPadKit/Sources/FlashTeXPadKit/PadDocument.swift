import CryptoKit
import Foundation
import FlashTeXProtocol

/// One open `.tex` buffer on the iPad. Offsets follow the contract discipline
/// (docs/contracts/runtime-v1.md): zero-based, end-exclusive UTF-8 bytes,
/// converted through `FlashTeXProtocol`'s `String.rangeOfUTF8` so an offset
/// inside a multi-byte scalar is refused, never truncated.
public struct PadDocument: Equatable {
    public var path: String
    public var text: String
    /// Local revision counter: starts at 1 for a freshly opened file and moves
    /// by one per applied edit (the same monotonic rule as transfer-v1
    /// `document_edit` / `capture_applied.new_revision`).
    public var revision: Int

    public init(path: String, text: String, revision: Int = 1) {
        self.path = path; self.text = text; self.revision = revision
    }

    /// Lowercase hex SHA-256 of the UTF-8 bytes — the `expected_sha256` /
    /// `document_before_sha256` binding used by the assistant-context review
    /// contract and transfer-v1 `capture_edit`.
    public var sha256Hex: String { PadDocument.sha256Hex(of: text) }

    public static func sha256Hex(of text: String) -> String {
        SHA256.hash(data: Data(text.utf8)).map { String(format: "%02x", $0) }.joined()
    }

    public var utf8Count: Int { text.utf8.count }

    /// Text of a UTF-8 byte range, or nil when the range is not on scalar boundaries.
    public func slice(startByte: Int, endByte: Int) -> String? {
        guard let r = text.rangeOfUTF8(start: startByte, end: endByte) else { return nil }
        return String(text[r])
    }

    /// UTF-8 byte offset of a UTF-16 caret (`NSRange.location`), or nil if the
    /// caret is out of range or splits a surrogate pair.
    public func byteOffset(ofUTF16 caret: Int) -> Int? {
        text.utf8ByteRange(of: NSRange(location: caret, length: 0))?.start
    }
}
