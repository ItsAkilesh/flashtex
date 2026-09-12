import FlashTeXProtocol

/// Source → preview sync: which preview items the editor caret is inside.
/// Pure so it can be tested without a view or a model.
enum CaretSync {
    /// Returns `(page number, index in that page's `items`)` for every text
    /// item whose `source` names `path` and contains the UTF-8 offset `byte`
    /// (`start_byte <= byte < end_byte`). Empty ranges only match when
    /// `byte == start_byte == end_byte`. Items without a source and non-text
    /// items are never returned.
    static func itemsContaining(byte: Int, path: String,
                                in result: RuntimeV1.CompileResult) -> [(page: Int, index: Int)] {
        var hits: [(page: Int, index: Int)] = []
        for page in result.pages {
            for (index, item) in page.items.enumerated() {
                guard case .text(let t) = item, let s = t.source, s.path == path else { continue }
                let contains = s.startByte == s.endByte
                    ? byte == s.startByte
                    : (s.startByte <= byte && byte < s.endByte)
                if contains { hits.append((page.number, index)) }
            }
        }
        return hits
    }

    /// Convenience for the preview: item indices grouped by page number.
    static func indicesByPage(byte: Int, path: String,
                              in result: RuntimeV1.CompileResult) -> [Int: Set<Int>] {
        var out: [Int: Set<Int>] = [:]
        for hit in itemsContaining(byte: byte, path: path, in: result) {
            out[hit.page, default: []].insert(hit.index)
        }
        return out
    }
}
