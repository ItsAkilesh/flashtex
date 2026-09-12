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
        /// Exact font size in screen points (bit pattern, so 10.000 and 10.004 pt
        /// are distinct entries and the CTFont is built at the size asked for;
        /// no quantization — geometry must stay exact for oracle comparison).
        var sizeBits: UInt64
        var size: CGFloat { CGFloat(Double(bitPattern: sizeBits)) }
    }

    struct Line {
        var line: CTLine
        var ascent: CGFloat
        var descent: CGFloat
        var width: CGFloat
    }

    /// Bound on cached lines; when exceeded both dictionaries are dropped
    /// wholesale (simpler than LRU, and a full rebuild is one revision's cost).
    var capacity = 20_000
    /// Bound on cached fonts (one per distinct face × exact size).
    var fontCapacity = 512
    private var lines: [Key: Line] = [:]
    private var fonts: [FontKey: CTFont] = [:]
    private(set) var hits = 0, misses = 0

    struct FontKey: Hashable { var postScriptName: String; var sizeBits: UInt64 }

    var count: Int { lines.count }
    var fontCount: Int { fonts.count }

    func clear() { lines.removeAll(keepingCapacity: true); fonts.removeAll(); hits = 0; misses = 0 }

    static func key(text: String, postScriptName: String, size: CGFloat) -> Key {
        Key(text: text, postScriptName: postScriptName, sizeBits: Double(size).bitPattern)
    }

    /// The CTFont for a face at an exact size (`CTFontGetSize` == `size`).
    func font(_ name: String, size: CGFloat) -> CTFont {
        let id = FontKey(postScriptName: name, sizeBits: Double(size).bitPattern)
        if let f = fonts[id] { return f }
        if fonts.count >= fontCapacity { fonts.removeAll(keepingCapacity: true); lines.removeAll(keepingCapacity: true) }
        let f = CTFontCreateWithName(name as CFString, size, nil)
        fonts[id] = f
        return f
    }

    func line(for key: Key) -> Line {
        if let l = lines[key] { hits += 1; return l }
        misses += 1
        if lines.count >= capacity { lines.removeAll(keepingCapacity: true); fonts.removeAll(keepingCapacity: true) }
        let attributed = NSAttributedString(string: key.text, attributes: [
            .font: font(key.postScriptName, size: key.size),
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
