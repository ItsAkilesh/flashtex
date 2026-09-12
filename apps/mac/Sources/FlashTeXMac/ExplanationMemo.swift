import Foundation
import FlashTeXProtocol

/// Explanations remembered by WHAT was explained rather than by which compile
/// result reported it (AI review follow-up 2): a diagnostic is keyed by its
/// severity, message, recovery, source range and the SHA-256 of the source
/// document it points into (all documents for an unlocated one). A recompile
/// that reports the same diagnostic for unchanged text (a new result id after
/// an edit elsewhere, an undo, a reopen) is answered from here without
/// launching or waiting for `flashtex-explain`; anything the memo does not
/// know exactly is fetched as before. Bounded; nothing here is persisted.
extension EditorDiagnostics {
    struct ExplanationMemo: Equatable {
        static let defaultCapacity = 4096

        struct Key: Hashable {
            var severity: String
            var message: String
            var recovery: String?
            var path: String?
            var startByte: Int
            var endByte: Int
            /// SHA-256 of the document the diagnostic points into, or of every
            /// document (path + text, in order) for an unlocated diagnostic.
            var sourceSHA256: String
        }

        private(set) var entries: [Key: Explanation] = [:]
        private var order: [Key] = []
        let capacity: Int
        /// Reads answered from the memo / reads that had to fetch (introspection).
        private(set) var hits = 0
        private(set) var misses = 0

        init(capacity: Int = ExplanationMemo.defaultCapacity) { self.capacity = capacity }

        var count: Int { entries.count }

        /// Keys for every diagnostic of `result`, computing each document's
        /// digest once. nil when a diagnostic points into a document that is
        /// not in `documents` (its text is unknown, so nothing can be reused).
        static func keys(for result: RuntimeV1.CompileResult, documents: [RuntimeV1.Document]) -> [Key]? {
            var digests: [String: String] = [:]
            for d in documents { digests[d.path] = SourceDigest.sha256Hex(d.text) }
            lazy var all = SourceDigest.sha256Hex(documents.map { "\($0.path)\u{0}\($0.text)\u{0}" }.joined())
            var keys: [Key] = []
            keys.reserveCapacity(result.diagnostics.count)
            for d in result.diagnostics {
                let sha: String
                if let s = d.source {
                    guard let digest = digests[s.path] else { return nil }
                    sha = digest
                } else {
                    sha = all
                }
                keys.append(Key(severity: d.severity.rawValue, message: d.message, recovery: d.recovery,
                                path: d.source?.path, startByte: d.source?.startByte ?? -1, endByte: d.source?.endByte ?? -1,
                                sourceSHA256: sha))
            }
            return keys
        }

        /// Remembers `explanations[i]` under the key of `result.diagnostics[i]`
        /// (extra or missing entries are ignored: the crate answers one per
        /// diagnostic in order, and a short reply explains nothing beyond it).
        mutating func remember(_ explanations: [Explanation], for result: RuntimeV1.CompileResult, documents: [RuntimeV1.Document]) {
            guard let keys = Self.keys(for: result, documents: documents) else { return }
            for (key, explanation) in zip(keys, explanations) {
                if entries[key] == nil {
                    order.append(key)
                    while order.count > capacity, let oldest = order.first {
                        order.removeFirst(); entries[oldest] = nil
                    }
                }
                entries[key] = explanation
            }
        }

        /// The full list for `result` when EVERY diagnostic is remembered for
        /// exactly this source text; nil (and a counted miss) otherwise. A
        /// result without diagnostics is trivially answered with `[]`.
        mutating func recall(for result: RuntimeV1.CompileResult, documents: [RuntimeV1.Document]) -> [Explanation]? {
            guard let keys = Self.keys(for: result, documents: documents) else { misses += 1; return nil }
            var out: [Explanation] = []
            out.reserveCapacity(keys.count)
            for k in keys {
                guard let e = entries[k] else { misses += 1; return nil }
                out.append(e)
            }
            hits += 1
            return out
        }
    }
}
