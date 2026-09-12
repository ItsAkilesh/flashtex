import AppKit
import CoreGraphics
import CoreText
import CryptoKit
import FlashTeXProtocol

// Experimental rendering-v2 consumer: content-addressed font resolution, one
// CoreGraphics draw routine shared by the preview and the PDF export, and
// exact hit/caret geometry from the display list's clusters. Nothing here
// touches the runtime-v1 path, which remains the default preview.

/// Bounded, content-addressed font resolution. Every `.otf`/`.ttf` in the
/// existing directories of `PreviewFonts.latinModernSearchPaths` (the bundle's
/// `Fonts`, the repository copy, `FLASHTEX_LM_DIR`) is hashed once; a display
/// list's font resource resolves only if its `sha256` names one of those files
/// exactly. Platform font names are never an identity and nothing is
/// substituted: a missing hash is a render failure for the whole list.
final class V2FontStore {
    struct BundledFont {
        var url: URL
        /// SHA-256 of the file bytes (the schema's reading of `sha256`).
        var bytesSha256: String
        /// SHA-256 of the file bytes followed by face index 0 as a 4-byte
        /// big-endian integer: what `flashtex-render` puts on the wire
        /// (font-engine's `content_sha256`). Documented deviation.
        var face0Sha256: String
        var byteLength: Int64
    }

    struct ResolvedFont {
        var resource: RenderingV2.FontResource
        var file: BundledFont
        var cgFont: CGFont
        /// Which hash convention matched: `bytes` or `bytes+face0`.
        var hashConvention: String
        func ctFont(size: Double) -> CTFont { CTFontCreateWithGraphicsFont(cgFont, size, nil, nil) }
    }

    static let shared = V2FontStore(directories: PreviewFonts.latinModernSearchPaths)

    private(set) var fonts: [BundledFont] = []
    private var byHash: [String: BundledFont] = [:]
    private var cgFonts: [String: CGFont] = [:]
    private let lock = NSLock()

    init(directories: [String]) {
        for dir in directories {
            let url = URL(fileURLWithPath: dir)
            guard let files = try? FileManager.default.contentsOfDirectory(at: url, includingPropertiesForKeys: nil) else { continue }
            for file in files.sorted(by: { $0.lastPathComponent < $1.lastPathComponent })
            where ["otf", "ttf"].contains(file.pathExtension.lowercased()) {
                guard let data = try? Data(contentsOf: file) else { continue }
                let bytes = Self.hex(SHA256.hash(data: data))
                var face0 = data; face0.append(contentsOf: [0, 0, 0, 0])
                let entry = BundledFont(url: file, bytesSha256: bytes, face0Sha256: Self.hex(SHA256.hash(data: face0)), byteLength: Int64(data.count))
                // First directory wins for duplicate bytes (identical content anyway).
                if byHash[bytes] == nil { byHash[bytes] = entry; byHash[entry.face0Sha256] = entry; fonts.append(entry) }
            }
        }
    }

    static func hex(_ digest: SHA256.Digest) -> String { digest.map { String(format: "%02x", $0) }.joined() }

    /// Resolves a manifest entry to the exact bundled bytes or throws a
    /// diagnostic-bearing error (`font_resource_unavailable` /
    /// `font_resource_mismatch`). The loaded font's glyph count, units per em
    /// and PostScript name must agree with the manifest.
    func resolve(_ resource: RenderingV2.FontResource) throws -> ResolvedFont {
        let short = String(resource.sha256.prefix(12))
        guard resource.isPaintable else {
            throw RenderingV2.ValidationError(code: "font_resource_unavailable",
                                              message: "font resource \(resource.sha256) (\(resource.postscriptName)) unavailable: format '\(resource.format)' carries no program bytes")
        }
        lock.lock(); defer { lock.unlock() }
        guard let file = byHash[resource.sha256] else {
            throw RenderingV2.ValidationError(code: "font_resource_unavailable",
                                              message: "font resource \(resource.sha256) (\(resource.postscriptName)) unavailable: no bundled font has this content hash (\(fonts.count) bundled fonts checked)")
        }
        guard file.byteLength == resource.byteLength else {
            throw RenderingV2.ValidationError(code: "font_resource_mismatch",
                                              message: "font resource \(short)… byte_length \(resource.byteLength) differs from the bundled file (\(file.byteLength) bytes)")
        }
        let cg: CGFont
        if let cached = cgFonts[file.bytesSha256] {
            cg = cached
        } else {
            guard let data = try? Data(contentsOf: file.url), let provider = CGDataProvider(data: data as CFData), let loaded = CGFont(provider) else {
                throw RenderingV2.ValidationError(code: "font_resource_unavailable", message: "font resource \(short)…: \(file.url.lastPathComponent) could not be loaded by CoreGraphics")
            }
            cgFonts[file.bytesSha256] = loaded
            cg = loaded
        }
        guard Int(cg.numberOfGlyphs) == resource.glyphCount else {
            throw RenderingV2.ValidationError(code: "font_resource_mismatch",
                                              message: "font resource \(short)… glyph_count \(resource.glyphCount) differs from the loaded font (\(cg.numberOfGlyphs))")
        }
        guard Int(cg.unitsPerEm) == resource.unitsPerEm else {
            throw RenderingV2.ValidationError(code: "font_resource_mismatch",
                                              message: "font resource \(short)… units_per_em \(resource.unitsPerEm) differs from the loaded font (\(cg.unitsPerEm))")
        }
        let psName = (cg.postScriptName as String?) ?? ""
        guard psName == resource.postscriptName else {
            throw RenderingV2.ValidationError(code: "font_resource_mismatch",
                                              message: "font resource \(short)… postscript_name '\(resource.postscriptName)' differs from the loaded font ('\(psName)')")
        }
        return ResolvedFont(resource: resource, file: file, cgFont: cg,
                            hashConvention: file.bytesSha256 == resource.sha256 ? "bytes" : "bytes+face0")
    }
}

/// A validated display list whose referenced fonts all resolved: the only
/// input the renderers accept, so a frame is either fully paintable or absent.
struct V2Frame {
    var id: String
    var list: RenderingV2.DisplayList
    /// Keyed by `font_id`; only fonts referenced by at least one glyph run.
    var fonts: [String: V2FontStore.ResolvedFont]

    /// Resolves every font referenced by a glyph run; the first failure aborts.
    static func prepare(_ envelope: RenderingV2.Envelope, store: V2FontStore = .shared) throws -> V2Frame {
        var referenced: [String] = []
        for page in envelope.payload.pages {
            for case .glyphRun(let run) in page.items where !referenced.contains(run.fontId) { referenced.append(run.fontId) }
        }
        var fonts: [String: V2FontStore.ResolvedFont] = [:]
        for id in referenced {
            guard let resource = envelope.payload.font(id: id) else {
                throw RenderingV2.ValidationError(code: "invalid_resource", message: "font resource '\(id)' is not declared")
            }
            fonts[id] = try store.resolve(resource)
        }
        return V2Frame(id: envelope.id, list: envelope.payload, fonts: fonts)
    }

    func page(number: Int) -> RenderingV2.Page? { list.pages.first { $0.number == number } }
}

/// The one draw routine. `ctx`'s user space must be PDF space for the page:
/// 1 unit = 1 PDF point, origin at the page's bottom-left, y up (the natural
/// space of a `CGContext` PDF page or a `CGBitmapContext`). The preview flips
/// its SwiftUI canvas into that space before calling this, so preview and
/// export paint identical geometry by construction. Items paint in list order;
/// glyphs are drawn by ORIGINAL glyph ID with `CTFontDrawGlyphs` at their
/// absolute origins (advances are never re-added; no reshaping, no kerning).
enum GlyphRunRenderer {
    /// `dark` inverts paint colors for the on-screen dark preview only; export
    /// callers pass `false` so the document keeps the list's colors.
    static func draw(page: RenderingV2.Page, frame: V2Frame, in ctx: CGContext, dark: Bool = false) {
        let pageHeight = page.heightPt
        ctx.textMatrix = .identity
        for item in page.items {
            switch item {
            case .rule(let r):
                ctx.setFillColor(color(r.paint, dark: dark))
                ctx.fill(pdfRect(x: r.x, top: r.top, width: r.width, height: r.height, pageHeight: pageHeight))
            case .glyphRun(let run):
                // `V2Frame.prepare` resolved every referenced font; a missing entry
                // cannot happen for a prepared frame, and is never painted around.
                guard let font = frame.fonts[run.fontId] else { continue }
                let ct = font.ctFont(size: RenderingV2.points(run.fontSize))
                let glyphs = run.glyphs.map { CGGlyph($0.gid) }
                let positions = run.glyphs.map { CGPoint(x: RenderingV2.points($0.originX), y: pageHeight - RenderingV2.points($0.baselineY)) }
                ctx.setFillColor(color(run.paint, dark: dark))
                CTFontDrawGlyphs(ct, glyphs, positions, glyphs.count, ctx)
            }
        }
    }

    static func color(_ p: RenderingV2.Paint, dark: Bool) -> CGColor {
        dark ? CGColor(srgbRed: 1 - p.r, green: 1 - p.g, blue: 1 - p.b, alpha: p.a)
             : CGColor(srgbRed: p.r, green: p.g, blue: p.b, alpha: p.a)
    }

    /// Top-left anchored tick rectangle → PDF-space points.
    static func pdfRect(x: Int64, top: Int64, width: Int64, height: Int64, pageHeight: Double) -> CGRect {
        CGRect(x: RenderingV2.points(x), y: pageHeight - RenderingV2.points(top) - RenderingV2.points(height),
               width: RenderingV2.points(width), height: RenderingV2.points(height))
    }

    /// A fresh sRGB bitmap context in PDF space (y up) for one page at
    /// `scale` pixels per point, filled with the page background.
    static func bitmapContext(widthPt: Double, heightPt: Double, scale: Double, dark: Bool = false) -> CGContext? {
        let w = Int((widthPt * scale).rounded(.up)), h = Int((heightPt * scale).rounded(.up))
        guard w > 0, h > 0,
              let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: 0,
                                  space: CGColorSpace(name: CGColorSpace.sRGB)!,
                                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else { return nil }
        ctx.setFillColor(dark ? CGColor(gray: 0.16, alpha: 1) : CGColor(gray: 1, alpha: 1))
        ctx.fill(CGRect(x: 0, y: 0, width: w, height: h))
        ctx.scaleBy(x: scale, y: scale)
        ctx.setShouldAntialias(true)
        ctx.setShouldSmoothFonts(false)
        ctx.setAllowsFontSubpixelPositioning(true)
        ctx.setShouldSubpixelPositionFonts(true)
        return ctx
    }

    /// Rasterizes one page through `draw` (used by tests and evidence capture).
    static func rasterize(page: RenderingV2.Page, frame: V2Frame, scale: Double, dark: Bool = false) -> CGImage? {
        guard let ctx = bitmapContext(widthPt: page.widthPt, heightPt: page.heightPt, scale: scale, dark: dark) else { return nil }
        draw(page: page, frame: frame, in: ctx, dark: dark)
        return ctx.makeImage()
    }

    /// One PDF page per display-list page, through the same `draw`. The
    /// document keeps the list's paint (never the dark preview colors).
    static func pdfData(frame: V2Frame) -> Data {
        let data = NSMutableData()
        guard let consumer = CGDataConsumer(data: data), let ctx = CGContext(consumer: consumer, mediaBox: nil, nil) else { return Data() }
        for page in frame.list.pages {
            var mediaBox = CGRect(x: 0, y: 0, width: page.widthPt, height: page.heightPt)
            ctx.beginPDFPage([kCGPDFContextMediaBox as String: NSData(bytes: &mediaBox, length: MemoryLayout<CGRect>.size)] as CFDictionary)
            ctx.setFillColor(CGColor(gray: 1, alpha: 1))
            ctx.fill(mediaBox)
            draw(page: page, frame: frame, in: ctx, dark: false)
            ctx.endPDFPage()
        }
        ctx.closePDF()
        return data as Data
    }
}

/// Exact hit-testing and caret geometry from the display list's clusters.
/// Rectangles are half-open in ticks; overlaps resolve to the item painted
/// last. Nothing is measured on the consumer side.
enum V2Geometry {
    struct Hit: Equatable {
        var itemIndex: Int
        /// Nil for a rule.
        var clusterIndex: Int?
        /// The cluster's logical text (a rule has none).
        var text: String?
        var sources: [RenderingV2.SourceRange]
        var syntheticReason: String?
        /// The rectangle that contained the query, in ticks.
        var rect: RenderingV2.Rect
    }

    /// Page point (y down, PDF points) → ticks, flooring so the half-open
    /// rectangle test is exact at integer edges.
    static func ticks(_ pt: Double) -> Int64 { Int64((pt * Double(RenderingV2.ticksPerPoint)).rounded(.down)) }

    static func hit(page: RenderingV2.Page, atPointX x: Double, y: Double) -> Hit? {
        hit(page: page, tickX: ticks(x), tickY: ticks(y))
    }

    static func hit(page: RenderingV2.Page, tickX x: Int64, tickY y: Int64) -> Hit? {
        for (index, item) in page.items.enumerated().reversed() {
            switch item {
            case .rule(let r):
                let rect = RenderingV2.Rect(x: r.x, top: r.top, width: r.width, height: r.height)
                if rect.contains(x: x, y: y) {
                    return Hit(itemIndex: index, clusterIndex: nil, text: nil, sources: r.sources ?? [], syntheticReason: r.syntheticReason, rect: rect)
                }
            case .glyphRun(let run):
                for (ci, c) in run.clusters.enumerated().reversed() {
                    if let rect = c.hitRects.last(where: { $0.contains(x: x, y: y) }) {
                        return Hit(itemIndex: index, clusterIndex: ci, text: run.clusterText(ci), sources: c.sources ?? [], syntheticReason: c.syntheticReason, rect: rect)
                    }
                }
            }
        }
        return nil
    }

    struct CaretMatch: Equatable {
        var itemIndex: Int
        var clusterIndex: Int
        /// Exact caret geometry when the cluster maps its source bytes 1:1 and
        /// the compiler supplied a caret at that logical byte; nil means the
        /// whole cluster is the documented selection fallback.
        var caret: RenderingV2.Caret?
        var hitRects: [RenderingV2.Rect]
    }

    /// Clusters whose sources contain `byte` of `path` (`start <= byte < end`;
    /// an empty range matches only its own offset), with exact caret geometry
    /// when derivable.
    static func clusters(containing byte: Int, path: String, in page: RenderingV2.Page) -> [CaretMatch] {
        var out: [CaretMatch] = []
        for (index, item) in page.items.enumerated() {
            guard case .glyphRun(let run) = item else { continue }
            for (ci, c) in run.clusters.enumerated() {
                guard let source = (c.sources ?? []).first(where: { s in
                    s.path == path && (s.startByte == s.endByte ? byte == s.startByte : (s.startByte <= byte && byte < s.endByte))
                }) else { continue }
                var caret: RenderingV2.Caret?
                if source.endByte - source.startByte == c.textEndByte - c.textStartByte {
                    let textByte = c.textStartByte + (byte - source.startByte)
                    caret = c.carets.first { $0.textByte == textByte }
                }
                out.append(CaretMatch(itemIndex: index, clusterIndex: ci, caret: caret, hitRects: c.hitRects))
            }
        }
        return out
    }
}
