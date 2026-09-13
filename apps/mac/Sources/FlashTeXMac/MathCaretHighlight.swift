import Foundation
import FlashTeXProtocol

/// Source → preview for a caret inside math (lane mac-navigation-3).
///
/// Measured on flashtex-render 9aaec57a: there is no `math` item kind. A
/// formula is laid out as ordinary `glyph_run` items (math font, script
/// sizes) plus `rule` items (fraction bars, the `\sqrt` overbar), and EVERY
/// cluster and rule of the formula carries one source range — the whole
/// formula including its delimiters (`$…$`, `\[…\]`). No cluster maps its
/// bytes 1:1, so `V2Geometry.clusters(containing:)` returns every glyph
/// cluster of the formula with no exact caret and the rules not at all.
///
/// `caretHighlights` keeps the exact caret / whole-cluster contract for text
/// and adds the enclosing formula box: when no cluster containing the caret
/// byte maps 1:1 and one source span fans out over several items, the
/// highlight is the union of every item on the page with exactly that span
/// (glyph clusters AND rules). A single non-1:1 cluster (`\'e` → é) keeps
/// the documented whole-cluster fallback. If a producer ever maps a math
/// atom 1:1 (`x` of `\frac{x}{y}` → its own bytes), that exact caret wins.
extension V2Geometry {
    /// The enclosing box of one fanned-out source span.
    struct FormulaBox: Equatable {
        /// The span every member item carries (the formula including delimiters).
        var source: RenderingV2.SourceRange
        /// Page item indices in the box, in page order.
        var itemIndices: [Int]
        var clusterCount: Int
        var ruleCount: Int
        /// Every member rectangle (cluster hit rects and rule rects), in ticks.
        var rects: [RenderingV2.Rect]
        /// Union of `rects`, in ticks.
        var bounds: RenderingV2.Rect
    }

    enum CaretHighlight: Equatable {
        /// Exact caret bar (1:1 cluster with a caret at that byte) or the
        /// whole-cluster fallback, exactly as before.
        case cluster(CaretMatch)
        /// The caret is inside a formula: the enclosing box of every item
        /// carrying that span.
        case formula(FormulaBox)
    }

    /// Whether `match` maps its source bytes 1:1 onto its text bytes (an
    /// exact caret is derivable for some byte of it).
    private static func isOneToOne(_ cluster: RenderingV2.Cluster, _ source: RenderingV2.SourceRange) -> Bool {
        source.endByte - source.startByte == cluster.textEndByte - cluster.textStartByte
    }

    /// Every item of `page` whose sources contain exactly `span`, as one box.
    static func formulaBox(for span: RenderingV2.SourceRange, in page: RenderingV2.Page) -> FormulaBox? {
        var indices: [Int] = [], rects: [RenderingV2.Rect] = []
        var clusters = 0, rules = 0
        for (index, item) in page.items.enumerated() {
            switch item {
            case .rule(let r):
                guard r.sources?.contains(span) == true else { continue }
                rules += 1
                indices.append(index)
                rects.append(RenderingV2.Rect(x: r.x, top: r.top, width: r.width, height: r.height))
            case .glyphRun(let run):
                var member = false
                for c in run.clusters where c.sources?.contains(span) == true {
                    clusters += 1; member = true
                    rects.append(contentsOf: c.hitRects)
                }
                if member { indices.append(index) }
            case .image:
                continue // images are never part of a formula box
            }
        }
        guard let first = rects.first else { return nil }
        var minX = first.x, minTop = first.top, maxX = first.x &+ first.width, maxBottom = first.top &+ first.height
        for r in rects.dropFirst() {
            minX = min(minX, r.x); minTop = min(minTop, r.top)
            maxX = max(maxX, r.x &+ r.width); maxBottom = max(maxBottom, r.top &+ r.height)
        }
        return FormulaBox(source: span, itemIndices: indices, clusterCount: clusters, ruleCount: rules, rects: rects,
                          bounds: RenderingV2.Rect(x: minX, top: minTop, width: maxX &- minX, height: maxBottom &- minTop))
    }

    /// Highlights for the caret at `byte` of `path` on `page`:
    /// - every 1:1 cluster containing the byte, as `.cluster` (exact caret
    ///   when the producer supplied one at that byte, else its rects);
    /// - otherwise, for each distinct fanned-out span containing the byte
    ///   (carried by two or more items, or by any rule), one `.formula` box;
    /// - a span carried by a single glyph cluster stays `.cluster` (fallback).
    static func caretHighlights(containing byte: Int, path: String, in page: RenderingV2.Page) -> [CaretHighlight] {
        let matches = clusters(containing: byte, path: path, in: page)
        var exact: [CaretHighlight] = []
        var fanned: [RenderingV2.SourceRange] = []
        var single: [CaretHighlight] = []
        for m in matches {
            guard case .glyphRun(let run) = page.items[m.itemIndex] else { continue }
            let c = run.clusters[m.clusterIndex]
            guard let source = (c.sources ?? []).first(where: { s in
                s.path == path && (s.startByte == s.endByte ? byte == s.startByte : (s.startByte <= byte && byte < s.endByte))
            }) else { continue }
            if isOneToOne(c, source) {
                exact.append(.cluster(m))
            } else if !fanned.contains(source) {
                fanned.append(source)
                if let box = formulaBox(for: source, in: page), box.clusterCount + box.ruleCount > 1 {
                    single.append(.formula(box))
                } else {
                    single.append(.cluster(m))
                }
            } else {
                // Another cluster of a span already boxed: covered by the box.
            }
        }
        return exact.isEmpty ? single : exact
    }
}

extension ShellModel {
    /// What the v2 pane highlights for the editor caret, for the note and
    /// tests: the formula box(es) the caret is inside on the current frame,
    /// with page numbers. Empty when the caret is in no formula.
    func caretFormulaBoxes() -> [(page: Int, box: V2Geometry.FormulaBox)] {
        guard let frame = displayListV2?.frame, let byte = caretByte else { return [] }
        var out: [(page: Int, box: V2Geometry.FormulaBox)] = []
        for page in frame.list.pages {
            for h in V2Geometry.caretHighlights(containing: byte, path: activePath, in: page) {
                if case .formula(let box) = h { out.append((page.number, box)) }
            }
        }
        return out
    }
}
