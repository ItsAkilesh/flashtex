import AppKit
import CoreText
import FlashTeXProtocol

/// Cached CoreText lines for the v1 preview. Resolving a SwiftUI `Text` per
/// item inside `Canvas` cost ~0.3 ms each (demo.tex: 968 items ≈ 300 ms per
/// keystroke on the main thread); a `CTLine` keyed by (string, face, size) is
/// built once per distinct run and survives across compile revisions, so an
/// unchanged paragraph costs only its draw call. Colour is taken from the
/// graphics context (`kCTForegroundColorFromContextAttributeName`), so light
/// and dark previews share one entry. Main-thread only (the draw closure runs
/// on the main thread and the cache is not synchronized).
final class PreviewTextCache {
    nonisolated(unsafe) static let shared = PreviewTextCache()

    struct Key: Hashable {
        var text: String
        var postScriptName: String
        /// Font size in screen points, rounded to 1/64 pt so display-scale
        /// jitter does not fragment the cache.
        var sizeQ: Int
    }

    struct Line {
        var line: CTLine
        var ascent: CGFloat
        var descent: CGFloat
        var width: CGFloat
    }

    /// Bound on cached entries; when exceeded the cache is dropped wholesale
    /// (simpler than LRU, and a full rebuild is one revision's cost).
    var capacity = 20_000
    private var lines: [Key: Line] = [:]
    private var fonts: [String: CTFont] = [:]
    private(set) var hits = 0, misses = 0

    var count: Int { lines.count }

    func clear() { lines.removeAll(keepingCapacity: true); fonts.removeAll(); hits = 0; misses = 0 }

    static func key(text: String, postScriptName: String, size: CGFloat) -> Key {
        Key(text: text, postScriptName: postScriptName, sizeQ: Int((size * 64).rounded()))
    }

    private func font(_ name: String, sizeQ: Int) -> CTFont {
        let id = "\(name)@\(sizeQ)"
        if let f = fonts[id] { return f }
        let f = CTFontCreateWithName(name as CFString, CGFloat(sizeQ) / 64, nil)
        fonts[id] = f
        return f
    }

    func line(for key: Key) -> Line {
        if let l = lines[key] { hits += 1; return l }
        misses += 1
        if lines.count >= capacity { lines.removeAll(keepingCapacity: true) }
        let attributed = NSAttributedString(string: key.text, attributes: [
            .font: font(key.postScriptName, sizeQ: key.sizeQ),
            kCTForegroundColorFromContextAttributeName as NSAttributedString.Key: true,
        ])
        let ct = CTLineCreateWithAttributedString(attributed)
        var ascent: CGFloat = 0, descent: CGFloat = 0
        let width = CGFloat(CTLineGetTypographicBounds(ct, &ascent, &descent, nil))
        let l = Line(line: ct, ascent: ascent, descent: descent, width: width)
        lines[key] = l
        return l
    }

    /// Screen rect of a text item drawn at `scale`, from the cached metrics.
    func rect(for t: RuntimeV1.PageItem.TextItem, scale: CGFloat) -> CGRect {
        let name = PreviewFonts.resolve(hint: t.font, size: t.fontSizePt).postScriptName
        let l = line(for: Self.key(text: t.text, postScriptName: name, size: t.fontSizePt * scale))
        return CGRect(x: t.xPt * scale, y: t.baselineYPt * scale - l.ascent, width: l.width, height: l.ascent + l.descent)
    }
}

/// Where each item of a page lands on screen, for hover, click and caret
/// highlights. A pure function of the page and scale (through the cache), so
/// the draw closure never has to publish state back into the view.
struct PreviewHitRects {
    struct Hit { var index: Int; var rect: CGRect; var source: RuntimeV1.SourceRange?; var text: String? }

    static func compute(page: RuntimeV1.Page, scale: CGFloat, rulesNegotiated: Bool,
                        cache: PreviewTextCache = .shared) -> [Hit] {
        var out: [Hit] = []
        out.reserveCapacity(page.items.count)
        for (index, item) in page.items.enumerated() {
            switch item {
            case .rule(let rule):
                out.append(Hit(index: index, rect: RuleGeometry.previewRect(rule, scale: scale), source: rule.source, text: nil))
            case .text(let t):
                if !rulesNegotiated, let r = RuleConvention.rect(for: t) {
                    let rect = CGRect(x: r.x * scale, y: r.y * scale, width: r.width * scale, height: max(0.5, r.height * scale))
                    out.append(Hit(index: index, rect: rect, source: t.source, text: t.text))
                } else {
                    out.append(Hit(index: index, rect: cache.rect(for: t, scale: scale), source: t.source, text: t.text))
                }
            default:
                continue
            }
        }
        return out
    }

    static func hit(_ hits: [Hit], at p: CGPoint) -> Hit? { hits.first { $0.rect.contains(p) } }
}
