import Foundation
import FlashTeXProtocol

/// Screen-reader view of a `compile_result`: an ordered reading sequence of
/// preview items grouped into lines and pages, with human labels, plus the
/// diagnostics as actionable elements. Pure and deterministic — no AppKit,
/// no view state — so the labels VoiceOver will read can be unit-tested.
///
/// Line grouping: items sharing a baseline form a cluster. A cluster whose
/// items are markedly smaller than a nearby cluster's and whose baseline sits
/// within `scriptReachEm` of it is attached to that cluster (the compiler
/// places superscripts 0.45 em above, subscripts 0.2 em below, and fraction
/// numerators/denominators about 0.6 em above/below the parent baseline, at
/// 0.7× or 0.5× size). Everything else is a body line. Attachment is
/// transitive, so a second-order script belongs to its root line.
public struct AccessibleDocumentModel: Equatable {
    /// Ratio at or below which a smaller cluster may be a script of a larger
    /// one. Compiler script scales are 0.7 and 0.5; a 17 pt heading over 12 pt
    /// body is ~0.7 too, so the vertical reach is what keeps a body line off
    /// a heading (body leading is ≥ 1.2 em, scripts are ≤ 0.65 em away).
    public static let scriptSizeRatio = 0.85
    /// Max |baseline delta| / parent size for a script cluster to attach.
    public static let scriptReachEm = 0.75
    /// Baselines closer than this (points) are the same cluster.
    public static let baselineTolerancePt = 0.5

    public enum Role: String, Equatable {
        case text, rule, superscript, `subscript`, numerator, denominator
    }

    /// One preview text item as a reading element.
    public struct Element: Equatable {
        public var page: Int
        /// 1-based line number within the page (reading order).
        public var line: Int
        /// Index into `Page.items`.
        public var itemIndex: Int
        public var text: String
        public var role: Role
        public var fontSizePt: Double
        /// Size of the body line the element belongs to (== fontSizePt for body text).
        public var lineSizePt: Double
        public var source: RuntimeV1.SourceRange?
        /// UTF-16 range in the current text of `source.path`, when the source
        /// maps onto it (exact, or rebased across edits when the compiled
        /// text was supplied). Nil when unmapped, stale, or no text was given.
        public var utf16Range: NSRange?

        /// How the element is spoken inside a line summary ("superscript 2").
        public var spoken: String {
            switch role {
            case .text: return text
            case .rule: return "fraction bar"
            case .superscript: return "superscript \(text)"
            case .subscript: return "subscript \(text)"
            case .numerator: return "numerator \(text)"
            case .denominator: return "denominator \(text)"
            }
        }

        /// Label VoiceOver reads when the element is focused.
        public var label: String { spoken }

        /// Secondary description: size context and source availability.
        public var value: String {
            var parts: [String] = []
            switch role {
            case .text:
                parts.append(Self.pt(fontSizePt) + " point")
            case .rule:
                let n = text.unicodeScalars.count
                parts.append("\(Self.pt(fontSizePt)) point, \(n) segment\(n == 1 ? "" : "s")")
            default:
                let pct = Int((fontSizePt / lineSizePt * 100).rounded())
                parts.append("\(Self.pt(fontSizePt)) point, \(pct)% of the \(Self.pt(lineSizePt)) point line")
            }
            if source == nil { parts.append("no source mapping") }
            else if utf16Range == nil { parts.append("source not mapped to the current text") }
            return parts.joined(separator: ", ")
        }

        public var actions: [String] { source == nil ? [] : ["Go to source"] }

        static func pt(_ v: Double) -> String {
            v == v.rounded() ? String(Int(v)) : String(format: "%.1f", v)
        }
    }

    public struct Line: Equatable {
        public var page: Int
        public var number: Int
        public var baselineYPt: Double
        public var fontSizePt: Double
        public var elements: [Element]

        /// "Page 1, line 3: Hello naïve FlashTeX"
        public var label: String { "Page \(page), line \(number): \(summary)" }
        /// Element spoken forms joined in reading (x) order.
        public var summary: String { elements.map(\.spoken).joined(separator: " ") }
    }

    public struct PageSummary: Equatable {
        public var number: Int
        public var lines: [Line]
        public var totalPages: Int
        public var label: String {
            "Page \(number) of \(totalPages), \(lines.count) line\(lines.count == 1 ? "" : "s")"
        }
    }

    /// Where a diagnostic's source overlaps the preview.
    public struct LineRef: Equatable {
        public var page: Int
        public var line: Int
    }

    /// A diagnostic as an actionable element.
    public struct DiagnosticElement: Equatable {
        public var index: Int
        public var severity: RuntimeV1.Severity
        public var message: String
        public var recovery: String?
        public var source: RuntimeV1.SourceRange?
        public var utf16Range: NSRange?
        /// Preview lines whose items overlap the diagnostic's source bytes.
        public var lines: [LineRef]

        public var severityWord: String { severity == .error ? "Error" : "Warning" }
        /// "Error: Missing } inserted for \textbf."
        public var label: String { "\(severityWord): \(message)" }
        /// "recovery: …" or "no provisional rendering", plus where it lands.
        public var value: String {
            var parts = [recovery.map { "recovery: \($0)" } ?? "no provisional rendering"]
            if source == nil { parts.append("no source mapping") }
            else if utf16Range == nil { parts.append("source not mapped to the current text, recompile to navigate") }
            if !lines.isEmpty {
                parts.append("in " + lines.map { "page \($0.page) line \($0.line)" }.joined(separator: ", "))
            }
            return parts.joined(separator: "; ")
        }
        public var actions: [String] { source == nil ? [] : ["Go to source"] }
        /// Everything VoiceOver would announce: "label — value; actions".
        public var announcement: String {
            label + " — " + value + (actions.isEmpty ? "" : "; " + actions.joined(separator: ", "))
        }
    }

    public var status: RuntimeV1.Status
    public var pages: [PageSummary]
    public var diagnostics: [DiagnosticElement]

    /// Every element in reading order: page by page, line by line, left to right.
    public var readingSequence: [Element] { pages.flatMap { $0.lines.flatMap(\.elements) } }
    public var lines: [Line] { pages.flatMap(\.lines) }

    /// "Compile result: recovered, 2 pages, 9 items, 1 error, 1 warning"
    public var summary: String {
        let items = readingSequence.count
        let errors = diagnostics.filter { $0.severity == .error }.count
        let warnings = diagnostics.count - errors
        return "Compile result: \(status.rawValue), \(pages.count) page\(pages.count == 1 ? "" : "s"), "
            + "\(items) item\(items == 1 ? "" : "s"), \(errors) error\(errors == 1 ? "" : "s"), "
            + "\(warnings) warning\(warnings == 1 ? "" : "s")"
    }

    /// - Parameters:
    ///   - result: the compile result to describe.
    ///   - documents: current editor text by path (for UTF-16 ranges).
    ///   - compiledDocuments: the text each document had when `result` was
    ///     produced; when it differs from `documents`, ranges are rebased with
    ///     `SourceMapping` or left nil (never mapped onto the wrong text).
    public init(result: RuntimeV1.CompileResult, documents: [String: String] = [:],
                compiledDocuments: [String: String]? = nil) {
        status = result.status
        let total = result.pages.count
        var pages: [PageSummary] = []
        for page in result.pages {
            let lines = Self.lines(of: page, documents: documents, compiledDocuments: compiledDocuments)
            pages.append(PageSummary(number: page.number, lines: lines, totalPages: total))
        }
        self.pages = pages
        var diags: [DiagnosticElement] = []
        for (i, d) in result.diagnostics.enumerated() {
            let utf16 = Self.utf16Range(for: d.source, expectedText: nil, documents: documents,
                                         compiledDocuments: compiledDocuments)
            var hits: [LineRef] = []
            if let s = d.source {
                for p in pages {
                    for l in p.lines where l.elements.contains(where: { e in
                        guard let es = e.source, es.path == s.path else { return false }
                        return es.startByte < s.endByte && s.startByte < es.endByte
                    }) { hits.append(LineRef(page: p.number, line: l.number)) }
                }
            }
            diags.append(DiagnosticElement(index: i, severity: d.severity, message: d.message,
                                           recovery: d.recovery, source: d.source, utf16Range: utf16, lines: hits))
        }
        diagnostics = diags
    }

    // MARK: line grouping

    static func utf16Range(for source: RuntimeV1.SourceRange?, expectedText: String?,
                           documents: [String: String], compiledDocuments: [String: String]?) -> NSRange? {
        guard let source, let current = documents[source.path] else { return nil }
        if let compiled = compiledDocuments?[source.path], compiled != current {
            guard let rebased = SourceMapping.rebase(source, from: compiled, to: current, expectedText: expectedText)
            else { return nil }
            return current.nsRange(utf8Bytes: rebased)
        }
        return current.nsRange(utf8Bytes: source)
    }

    private struct Cluster {
        var baseline: Double
        var size: Double      // max font size in the cluster
        var minX: Double
        var items: [(index: Int, item: RuntimeV1.PageItem.TextItem)]
        var parent: Int?      // cluster index it attaches to
    }

    static func lines(of page: RuntimeV1.Page, documents: [String: String],
                      compiledDocuments: [String: String]?) -> [Line] {
        // 1. Cluster by baseline.
        var clusters: [Cluster] = []
        for (index, item) in page.items.enumerated() {
            guard case .text(let t) = item else { continue }
            if let ci = clusters.firstIndex(where: { abs($0.baseline - t.baselineYPt) <= baselineTolerancePt }) {
                clusters[ci].items.append((index, t))
                clusters[ci].size = max(clusters[ci].size, t.fontSizePt)
                clusters[ci].minX = min(clusters[ci].minX, t.xPt)
            } else {
                clusters.append(Cluster(baseline: t.baselineYPt, size: t.fontSizePt, minX: t.xPt,
                                        items: [(index, t)], parent: nil))
            }
        }
        clusters.sort { $0.baseline != $1.baseline ? $0.baseline < $1.baseline : $0.items[0].index < $1.items[0].index }

        // 2. Attach small clusters to the nearest larger neighbour within reach.
        for ci in clusters.indices {
            let c = clusters[ci]
            var best: (index: Int, reach: Double)?
            for pi in clusters.indices where pi != ci {
                let p = clusters[pi]
                guard c.size <= p.size * scriptSizeRatio else { continue }
                let reach = abs(c.baseline - p.baseline) / p.size
                guard reach <= scriptReachEm, c.minX >= p.minX - p.size else { continue }
                if best == nil || reach < best!.reach { best = (pi, reach) }
            }
            clusters[ci].parent = best?.index
        }
        func root(_ i: Int) -> Int {
            var i = i, hops = 0
            while let p = clusters[i].parent, hops < clusters.count { i = p; hops += 1 }
            return i
        }

        // 3. Build lines from root clusters in baseline order.
        var lines: [Line] = []
        for ri in clusters.indices where clusters[ri].parent == nil {
            let rootC = clusters[ri]
            let members = clusters.indices.filter { root($0) == ri }
            var all: [(index: Int, item: RuntimeV1.PageItem.TextItem, attached: Bool)] = []
            for mi in members { for e in clusters[mi].items { all.append((e.index, e.item, mi != ri)) } }
            let rules = all.compactMap { e in RuleConvention.rect(for: e.item).map { (rect: $0, baseline: e.item.baselineYPt) } }
            let number = lines.count + 1
            // Role first, then reading order: left to right by x, except that a
            // fraction reads numerator, bar, denominator at the bar's position.
            var keyed: [(key: (Double, Int, Double, Int), element: Element)] = []
            for e in all {
                let role: Role
                var anchorX = e.item.xPt, order = 1
                if let rect = RuleConvention.rect(for: e.item) {
                    role = .rule
                    anchorX = rect.x
                } else if !e.attached {
                    role = .text
                } else {
                    let above = e.item.baselineYPt < rootC.baseline
                    // Numerator/denominator: a rule in this line spans the item's x
                    // and sits on the far side of it.
                    let rule = rules.first { r in
                        e.item.xPt >= r.rect.x - 0.5 && e.item.xPt <= r.rect.x + r.rect.width + 0.5
                            && (above ? e.item.baselineYPt < r.baseline : e.item.baselineYPt > r.baseline)
                    }
                    if let rule {
                        role = above ? .numerator : .denominator
                        anchorX = rule.rect.x
                        order = above ? 0 : 2
                    } else {
                        role = above ? .superscript : .subscript
                    }
                }
                let utf16 = utf16Range(for: e.item.source, expectedText: e.item.text,
                                       documents: documents, compiledDocuments: compiledDocuments)
                keyed.append(((anchorX, order, e.item.xPt, e.index),
                              Element(page: page.number, line: number, itemIndex: e.index, text: e.item.text,
                                      role: role, fontSizePt: e.item.fontSizePt, lineSizePt: rootC.size,
                                      source: e.item.source, utf16Range: utf16)))
            }
            keyed.sort { a, b in
                if a.key.0 != b.key.0 { return a.key.0 < b.key.0 }
                if a.key.1 != b.key.1 { return a.key.1 < b.key.1 }
                if a.key.2 != b.key.2 { return a.key.2 < b.key.2 }
                return a.key.3 < b.key.3
            }
            let elements = keyed.map(\.element)
            lines.append(Line(page: page.number, number: number, baselineYPt: rootC.baseline,
                              fontSizePt: rootC.size, elements: elements))
        }
        return lines
    }
}
