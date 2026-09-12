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
    ///
    /// Identity is the raw bytes: `byHash` is only a lookup key (either hash
    /// spelling of the same bytes); the bytes actually handed to CoreGraphics
    /// are re-hashed at load time and must equal the discovered raw SHA-256
    /// and length (GH31). A bytes+face0 match therefore never admits a file
    /// whose raw bytes changed since discovery.
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
            // Verified once from bytes whose raw SHA-256 was `bytesSha256`;
            // CGFont is immutable, so frames prepared earlier keep exactly it.
            cg = cached
        } else {
            // GH31: discovery hashed the file at init, but the path can change
            // afterwards (an installer, a partial copy, a tampered directory).
            // Read the file ONCE, authenticate exactly those bytes (length and
            // raw SHA-256 against the immutable discovery record) and build
            // the CGFont from those same bytes. A file that no longer matches
            // is refused before any CGFont exists and nothing is cached, so a
            // stale discovery record can never lend its hash to new bytes.
            guard let data = try? Data(contentsOf: file.url) else {
                throw RenderingV2.ValidationError(code: "font_resource_unavailable", message: "font resource \(short)…: \(file.url.lastPathComponent) could not be read")
            }
            guard Int64(data.count) == file.byteLength, Self.hex(SHA256.hash(data: data)) == file.bytesSha256 else {
                throw RenderingV2.ValidationError(code: "font_resource_mismatch",
                                                  message: "font resource \(short)…: \(file.url.lastPathComponent) on disk (\(data.count) bytes) no longer matches the bytes discovered at startup (\(file.byteLength) bytes, sha256 \(file.bytesSha256.prefix(12))…); refusing to load changed font bytes under the discovered hash")
            }
            guard let provider = CGDataProvider(data: data as CFData), let loaded = CGFont(provider) else {
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

/// One page of a frame, converted ONCE into what CoreGraphics consumes:
/// glyph IDs, absolute origins in PDF space (y up, points), the exact CTFont
/// per run, and rule rectangles. Built off the main thread by
/// `V2Frame.prepare`; painting a prepared page allocates nothing and does no
/// tick arithmetic. Immutable: CTFont/CGFont are immutable CoreFoundation
/// objects, so a prepared page can be painted from any thread.
struct V2PreparedPage: @unchecked Sendable {
    struct Run {
        var font: CTFont
        var glyphs: [CGGlyph]
        /// Baseline origins in PDF space (`x`, `pageHeight − baselineY`), points.
        var positions: [CGPoint]
        var paint: RenderingV2.Paint
    }
    enum Item {
        case rule(CGRect, RenderingV2.Paint)
        case run(Run)
    }
    var number: Int
    var widthPt: Double
    var heightPt: Double
    var items: [Item]
    var glyphCount: Int

    init(page: RenderingV2.Page, fonts: [String: V2FontStore.ResolvedFont]) throws {
        let heightPt = page.heightPt
        number = page.number
        widthPt = page.widthPt
        self.heightPt = heightPt
        var items: [Item] = []
        items.reserveCapacity(page.items.count)
        var glyphs = 0
        let q = V2PreparedPage.serialized
        // One CTFont per (font, size) on this page: CTFontCreateWithGraphicsFont
        // per run cost ~20 µs × 973 runs on a two-page document.
        var ctFonts: [String: CTFont] = [:]
        for item in page.items {
            switch item {
            case .rule(let r):
                let rect = GlyphRunRenderer.pdfRect(x: r.x, top: r.top, width: r.width, height: r.height, pageHeight: heightPt)
                items.append(.rule(CGRect(x: q(rect.origin.x), y: q(rect.origin.y), width: q(rect.width), height: q(rect.height)), r.paint))
            case .glyphRun(let run):
                guard let font = fonts[run.fontId] else {
                    throw RenderingV2.ValidationError(code: "invalid_resource", message: "page \(page.number): font resource '\(run.fontId)' did not resolve")
                }
                let size = q(RenderingV2.points(run.fontSize))
                let key = "\(run.fontId)@\(size)"
                let ct: CTFont
                if let cached = ctFonts[key] { ct = cached } else { ct = font.ctFont(size: size); ctFonts[key] = ct }
                items.append(.run(Run(font: ct,
                                      glyphs: run.glyphs.map { CGGlyph($0.gid) },
                                      positions: run.glyphs.map { CGPoint(x: q(RenderingV2.points($0.originX)), y: q(heightPt - RenderingV2.points($0.baselineY))) },
                                      paint: run.paint)))
                glyphs += run.glyphs.count
            }
        }
        self.items = items
        glyphCount = glyphs
    }

    /// The value CoreGraphics' PDF writer serializes for `v`: 7 significant
    /// digits (`%.7g`, measured on its content streams: `637.706`,
    /// `0.3985052`, `12.20423`). Preparing every page coordinate through this
    /// once makes the preview raster and the export raster start from
    /// identical numbers; the displacement from the tick geometry is at most
    /// half a unit in the 7th digit (5e-5 pt for coordinates below 1000 pt).
    static func serialized(_ v: Double) -> Double {
        guard v != 0, v.isFinite else { return v }
        // Round to 7 significant digits arithmetically (String(format:) per
        // coordinate cost ~15 ms on a two-page document). The result is the
        // nearest double to a ≤7-digit decimal, which the writer's `%.7g`
        // reproduces exactly, so preview and export still share the numbers.
        let e = floor(log10(abs(v)))
        let scale = pow(10.0, 6 - e)
        return (v * scale).rounded() / scale
    }
}

/// A validated display list whose referenced fonts all resolved and whose
/// pages are prepared: the only input the renderers accept, so a frame is
/// either fully paintable or absent. Immutable once built (value type over
/// immutable CoreFoundation fonts), so it may be built on a background queue
/// and handed to the main thread.
struct V2Frame: @unchecked Sendable {
    var id: String
    var list: RenderingV2.DisplayList
    /// Keyed by `font_id`; only fonts referenced by at least one glyph run.
    var fonts: [String: V2FontStore.ResolvedFont]
    /// One entry per `list.pages`, same order.
    var prepared: [V2PreparedPage]
    /// Distinct for every `prepare` call: identifies this frame instance
    /// (bitmap cache keys), independent of the envelope id or file.
    var preparedNonce: UInt64 = V2Frame.nextNonce()

    private static let nonceLock = NSLock()
    private static var nonce: UInt64 = 0
    static func nextNonce() -> UInt64 { nonceLock.lock(); defer { nonceLock.unlock() }; nonce += 1; return nonce }

    /// Resolves every font referenced by a glyph run and prepares every page;
    /// the first failure aborts (no partial frame). Pure: safe off-main.
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
        let prepared = try envelope.payload.pages.map { try V2PreparedPage(page: $0, fonts: fonts) }
        return V2Frame(id: envelope.id, list: envelope.payload, fonts: fonts, prepared: prepared)
    }

    func page(number: Int) -> RenderingV2.Page? { list.pages.first { $0.number == number } }
    func preparedPage(number: Int) -> V2PreparedPage? { prepared.first { $0.number == number } }
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
    /// `glyphByGlyph`: one `CTFontDrawGlyphs` call per glyph. Required on a PDF
    /// context: CoreGraphics' PDF writer merges a run's glyphs into `Tj`
    /// strings positioned by the font's own advances plus integer 1/1000 em
    /// `TJ` adjustments, so origins that deviate from those advances (TeX/TFM
    /// metrics on an OpenType font) drift by up to a pixel over a line; a
    /// glyph drawn alone gets its own `Tm`/`Td` at 7 significant digits — the
    /// value the preparation already quantized to. Bitmap contexts take the
    /// positions array exactly, so the preview batches (measured: 0 differing
    /// pixels either way when positions match the advances).
    static func draw(_ page: V2PreparedPage, in ctx: CGContext, dark: Bool = false, glyphByGlyph: Bool = false) {
        ctx.textMatrix = .identity
        for item in page.items {
            switch item {
            case .rule(let rect, let paint):
                // A path fill, not `fill(rect)`: CoreGraphics' fast rectangle
                // fill computes edge coverage differently from the scan
                // converter that replays the PDF's `re f`, and differed by one
                // gray level along every rule row (measured, see V2Parity).
                ctx.setFillColor(color(paint, dark: dark))
                ctx.beginPath()
                ctx.addRect(rect)
                ctx.fillPath()
            case .run(let run):
                ctx.setFillColor(color(run.paint, dark: dark))
                if glyphByGlyph {
                    run.glyphs.withUnsafeBufferPointer { g in
                        run.positions.withUnsafeBufferPointer { p in
                            for i in 0..<g.count { CTFontDrawGlyphs(run.font, g.baseAddress! + i, p.baseAddress! + i, 1, ctx) }
                        }
                    }
                } else {
                    CTFontDrawGlyphs(run.font, run.glyphs, run.positions, run.glyphs.count, ctx)
                }
            }
        }
    }

    /// Same routine, addressed by display-list page. A page number the frame
    /// does not hold paints nothing (a prepared frame has every page).
    static func draw(page: RenderingV2.Page, frame: V2Frame, in ctx: CGContext, dark: Bool = false) {
        guard let prepared = frame.preparedPage(number: page.number) else { return }
        draw(prepared, in: ctx, dark: dark)
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

    /// Rasterizes one prepared page through `draw`: what the v2 pane blits on
    /// screen (`V2PageRasterizer`), what tests and the parity check compare.
    /// Safe off-main: the page is immutable and the context is private.
    static func rasterize(_ page: V2PreparedPage, scale: Double, dark: Bool = false) -> CGImage? {
        guard let ctx = bitmapContext(widthPt: page.widthPt, heightPt: page.heightPt, scale: scale, dark: dark) else { return nil }
        draw(page, in: ctx, dark: dark)
        return ctx.makeImage()
    }

    static func rasterize(page: RenderingV2.Page, frame: V2Frame, scale: Double, dark: Bool = false) -> CGImage? {
        guard let prepared = frame.preparedPage(number: page.number) else { return nil }
        return rasterize(prepared, scale: scale, dark: dark)
    }

    /// One PDF page per display-list page, through the same `draw`. The
    /// document keeps the list's paint (never the dark preview colors).
    static func pdfData(frame: V2Frame) -> Data {
        let data = NSMutableData()
        guard let consumer = CGDataConsumer(data: data), let ctx = CGContext(consumer: consumer, mediaBox: nil, nil) else { return Data() }
        for page in frame.prepared {
            var mediaBox = CGRect(x: 0, y: 0, width: page.widthPt, height: page.heightPt)
            ctx.beginPDFPage([kCGPDFContextMediaBox as String: NSData(bytes: &mediaBox, length: MemoryLayout<CGRect>.size)] as CFDictionary)
            ctx.setFillColor(CGColor(gray: 1, alpha: 1))
            ctx.fill(mediaBox)
            draw(page, in: ctx, dark: false, glyphByGlyph: true)
            ctx.endPDFPage()
        }
        ctx.closePDF()
        return data as Data
    }
}

/// Export-versus-preview comparison with NO tolerance: every page of the
/// frame is rasterized through `GlyphRunRenderer.rasterize` (the bitmap the
/// v2 pane shows) and, separately, exported to PDF through
/// `GlyphRunRenderer.pdfData` and rasterized back by CoreGraphics into an
/// identically configured bitmap. Any pixel whose RGBA differs counts.
/// Same rasterizer, same DPI, same color space and font resources, so a
/// nonzero count is a placement/resource disagreement, never "antialiasing".
enum V2Parity {
    struct PageResult: Codable, Equatable {
        var page: Int
        var widthPx: Int
        var heightPx: Int
        var differingPixels: Int
        /// SHA-256 of the raw premultiplied RGBA bytes of each bitmap.
        var previewSha256: String
        var exportSha256: String
        var identical: Bool { differingPixels == 0 && previewSha256 == exportSha256 }
    }
    struct Report: Codable, Equatable {
        var frameId: String
        var projectId: String
        var revision: Int
        var scale: Double
        var tolerance: Int = 0
        var pdfBytes: Int
        var pages: [PageResult]
        var identical: Bool { pages.allSatisfy(\.identical) }
        var totalDifferingPixels: Int { pages.reduce(0) { $0 + $1.differingPixels } }
    }

    /// Raw RGBA bytes of an image drawn into the renderer's bitmap configuration.
    static func rgba(_ image: CGImage) -> [UInt8] {
        guard let ctx = CGContext(data: nil, width: image.width, height: image.height, bitsPerComponent: 8, bytesPerRow: image.width * 4,
                                  space: CGColorSpace(name: CGColorSpace.sRGB)!, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue),
              let base = ctx.data else { return [] }
        ctx.draw(image, in: CGRect(x: 0, y: 0, width: image.width, height: image.height))
        return Array(UnsafeBufferPointer(start: base.assumingMemoryBound(to: UInt8.self), count: image.width * image.height * 4))
    }

    static func differingPixels(_ a: [UInt8], _ b: [UInt8]) -> Int {
        guard a.count == b.count else { return max(a.count, b.count) / 4 }
        var n = 0
        var i = 0
        while i < a.count {
            if a[i] != b[i] || a[i + 1] != b[i + 1] || a[i + 2] != b[i + 2] || a[i + 3] != b[i + 3] { n += 1 }
            i += 4
        }
        return n
    }

    /// Per-page bitmaps for `compare` (also what the evidence hook writes).
    struct PageBitmaps { var page: Int; var preview: CGImage; var export: CGImage }

    /// Exports `frame` once, then rasterizes every PDF page next to the
    /// preview raster of the same page. Off-main safe.
    static func compare(frame: V2Frame, scale: Double, bitmaps: ((PageBitmaps) -> Void)? = nil) -> Report {
        let pdf = GlyphRunRenderer.pdfData(frame: frame)
        var results: [PageResult] = []
        let document = CGDataProvider(data: pdf as CFData).flatMap { CGPDFDocument($0) }
        for (index, page) in frame.prepared.enumerated() {
            guard let preview = GlyphRunRenderer.rasterize(page, scale: scale) else { continue }
            let previewBytes = rgba(preview)
            var exportBytes: [UInt8] = []
            var exportImage: CGImage?
            if let document, let pdfPage = document.page(at: index + 1),
               let ctx = GlyphRunRenderer.bitmapContext(widthPt: page.widthPt, heightPt: page.heightPt, scale: scale) {
                ctx.drawPDFPage(pdfPage)
                if let image = ctx.makeImage() { exportImage = image; exportBytes = rgba(image) }
            }
            results.append(PageResult(page: page.number, widthPx: preview.width, heightPx: preview.height,
                                      differingPixels: differingPixels(previewBytes, exportBytes),
                                      previewSha256: V2FontStore.hex(SHA256.hash(data: Data(previewBytes))),
                                      exportSha256: V2FontStore.hex(SHA256.hash(data: Data(exportBytes)))))
            if let bitmaps, let exportImage { bitmaps(PageBitmaps(page: page.number, preview: preview, export: exportImage)) }
        }
        return Report(frameId: frame.id, projectId: frame.list.projectId, revision: frame.list.revision, scale: scale, pdfBytes: pdf.count, pages: results)
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
