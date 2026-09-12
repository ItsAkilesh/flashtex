import Foundation

/// The iPad's review gate for one reviewed proposal. Mirrors the host
/// responsibilities the assistant-context README assigns to the native host:
/// display the proposal, obtain explicit approval, then apply the approved
/// group exactly once against a fresh source snapshot (revision + SHA-256 +
/// exact removed text), or cancel and write nothing.
///
/// State is `pending` → `cancelled` | `applied`; every transition out of
/// `pending` is final, so a second approve cannot insert twice and a cancel
/// after an approve cannot undo the receipt.
public struct ReviewSession: Equatable {
    public enum State: Equatable {
        case pending
        case cancelled(reason: String)
        case applied(Receipt)
        case refused(String)
    }

    /// Echo of what was inserted: the helper's ids plus the buffer binding
    /// after the single application.
    public struct Receipt: Equatable {
        public var commandId: String
        public var reviewId: String
        public var expectedRevision: Int
        public var newRevision: Int
        public var sha256Before: String
        public var sha256After: String
        public var editCount: Int
        /// The UTF-8 byte range each replacement occupies in the new text.
        public var insertedRanges: [(start: Int, end: Int)]

        public static func == (a: Receipt, b: Receipt) -> Bool {
            a.commandId == b.commandId && a.reviewId == b.reviewId && a.newRevision == b.newRevision
                && a.sha256After == b.sha256After && a.insertedRanges.map { $0.start } == b.insertedRanges.map { $0.start }
                && a.insertedRanges.map { $0.end } == b.insertedRanges.map { $0.end }
        }
    }

    public let review: ReviewedProposal.Review
    public let approved: ReviewedProposal.Approved
    public private(set) var state: State = .pending

    public init(review: ReviewedProposal.Review, approved: ReviewedProposal.Approved) {
        self.review = review; self.approved = approved
    }

    public var isPending: Bool { state == .pending }

    /// Cancel: nothing is written. Idempotent once terminal.
    public mutating func cancel(reason: String = "cancelled by user") {
        guard case .pending = state else { return }
        state = .cancelled(reason: reason)
    }

    public enum ApplyError: Error, Equatable, CustomStringConvertible {
        case notPending
        case reviewMismatch
        case pathMismatch(expected: String, actual: String)
        case revisionMismatch(expected: Int, actual: Int)
        case shaMismatch
        case badOffsets(start: Int, end: Int)
        case removedTextMismatch(index: Int)
        case overlapping
        public var description: String {
            switch self {
            case .notPending: return "review is no longer pending"
            case .reviewMismatch: return "approved_group.review_id does not name this review"
            case .pathMismatch(let e, let a): return "approved for \(e), buffer is \(a)"
            case .revisionMismatch(let e, let a): return "expected revision \(e), buffer is at \(a)"
            case .shaMismatch: return "buffer SHA-256 differs from expected_sha256"
            case .badOffsets(let s, let e): return "edit \(s)..<\(e) is not on UTF-8 scalar boundaries of the buffer"
            case .removedTextMismatch(let i): return "edit \(i): removed_text does not match the buffer"
            case .overlapping: return "edits overlap or share a boundary"
            }
        }
    }

    /// Explicit approval: verifies the approved group against `document`
    /// exactly as the ledger would (path, revision, SHA-256, offsets on scalar
    /// boundaries, exact removed text, non-overlapping), applies every edit
    /// once, bumps the revision, and records the receipt. Any failure leaves
    /// the document untouched and the session `refused`.
    @discardableResult
    public mutating func approve(applyingTo document: inout PadDocument) throws -> Receipt {
        guard case .pending = state else { throw ApplyError.notPending }
        let g = approved.payload.group
        do {
            guard approved.payload.reviewId == review.reviewId else { throw ApplyError.reviewMismatch }
            guard approved.payload.path == document.path else {
                throw ApplyError.pathMismatch(expected: approved.payload.path, actual: document.path)
            }
            guard g.expectedRevision == document.revision else {
                throw ApplyError.revisionMismatch(expected: g.expectedRevision, actual: document.revision)
            }
            let before = document.sha256Hex
            guard g.expectedSha256 == before else { throw ApplyError.shaMismatch }
            let sorted = g.edits.enumerated().sorted { $0.element.startByte < $1.element.startByte }
            var lastEnd = -1
            for (_, e) in sorted {
                guard e.endByte >= e.startByte else { throw ApplyError.badOffsets(start: e.startByte, end: e.endByte) }
                // Sharing a boundary is refused too (assistant-context rule).
                if lastEnd >= 0, e.startByte <= lastEnd { throw ApplyError.overlapping }
                lastEnd = e.endByte
            }
            for (i, e) in sorted {
                guard let slice = document.slice(startByte: e.startByte, endByte: e.endByte) else {
                    throw ApplyError.badOffsets(start: e.startByte, end: e.endByte)
                }
                guard slice == e.removedText else { throw ApplyError.removedTextMismatch(index: i) }
            }
            // Apply from the back so earlier offsets stay valid.
            var text = document.text
            var inserted: [(Int, Int)] = []
            var shift = 0
            for (_, e) in sorted {
                let r = text.rangeOfUTF8(start: e.startByte + shift, end: e.endByte + shift)!
                text.replaceSubrange(r, with: e.replacement)
                let newStart = e.startByte + shift
                inserted.append((newStart, newStart + e.replacement.utf8.count))
                shift += e.replacement.utf8.count - (e.endByte - e.startByte)
            }
            document.text = text
            document.revision += 1
            let receipt = Receipt(commandId: g.commandId, reviewId: review.reviewId, expectedRevision: g.expectedRevision,
                                  newRevision: document.revision, sha256Before: before, sha256After: document.sha256Hex,
                                  editCount: g.edits.count, insertedRanges: inserted.map { (start: $0.0, end: $0.1) })
            state = .applied(receipt)
            return receipt
        } catch let e as ApplyError {
            state = .refused(e.description)
            throw e
        }
    }
}
