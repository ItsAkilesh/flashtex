import Foundation

/// Codable model plus a fail-closed validator for the EXPERIMENTAL rendering-v2
/// `display_list` envelope (`protocol/rendering-v2.schema.json` and
/// `docs/contracts/rendering-v2-proposal.md` on main; not a negotiated production
/// wire — runtime-v1 remains authoritative). Shapes follow the schema; the fields
/// below are what `flashtex-render --v2` (crates/render-pipeline) actually emits.
///
/// Coordinates are `bp_2pow20` ticks: signed integers, 1,048,576 per PDF point,
/// origin at the page's top-left, y down. Glyph origins are absolute baseline
/// origins; advances are never re-added. Glyph IDs index the ORIGINAL font's
/// glyph order. Every cluster is an end-exclusive UTF-8 byte range of the run's
/// logical `text` with its own hit rectangles, carets and source provenance.
///
/// Documented deviations between the schema and the pipeline that this model
/// accepts (each one is explicit here, never silently widened):
/// - `fonts[].format`: schema allows only `static-truetype`; the pipeline emits
///   `opentype-cff` (Latin Modern) and `core14-afm` (Times metrics, no program
///   bytes). The model decodes all three; only `static-truetype`/`opentype-cff`
///   are paintable — a run that references a `core14-afm` font fails resolution.
/// - `fonts[].byte_length`: schema minimum 1; `core14-afm` entries carry 0.
/// - `fonts[].sha256`/`font_id`: the pipeline's value is SHA-256(program bytes
///   ‖ face_index as 4-byte big-endian), font-engine's `content_sha256`, not
///   SHA-256(bytes). Consumers resolving by hash must try both conventions
///   (see `GlyphRunRenderer`'s font store); the model only checks the format.
/// - Unknown JSON keys are ignored by the decoder (Swift `Codable`), where the
///   schema says `additionalProperties: false`. Unknown `kind` values, unknown
///   `required_features`, unknown protocol versions and message types are
///   rejected.
/// - The schema does not require clusters to partition the run text or source
///   paths to name a declared document; this validator requires both (as
///   crates/rendering-core does) because hit-testing depends on them. It also
///   applies rendering-core's other structural rules: every cluster is
///   referenced by at least one glyph, source ranges lie within the declared
///   document byte length, used features (`glyph_run`, `rule`, `rgba-srgb`,
///   `cluster-actualtext`) are declared in `required_features`, all ticks and
///   tick sums stay within ±(2^53−1), and collection sizes are bounded
///   (`Bounds`). `static-truetype` is not derived from glyph runs (see above).
public enum RenderingV2 {
    public static let protocolVersion = 2
    public static let messageType = "display_list"
    public static let renderFormat = "display-list-v2"
    public static let coordinateUnit = "bp_2pow20"
    public static let colorSpace = "srgb"
    public static let textExtraction = "cluster-actualtext"
    public static let ticksPerPoint: Int64 = 1 << 20
    /// Largest integer JSON carries exactly (2^53 − 1); every tick, revision and
    /// byte count must stay within ±this, and tick sums are checked, as
    /// crates/rendering-core's `Tick::validate`/`checked_add` do.
    public static let maxExactInteger: Int64 = (1 << 53) - 1
    /// Bounded collection sizes (crates/rendering-core `validate`): a list that
    /// exceeds one is refused, never truncated.
    public enum Bounds {
        public static let documents = 1...4096
        public static let fonts = 0...256
        public static let pages = 0...10000
        public static let pageItems = 0...100000
        public static let diagnostics = 0...10000
        public static let runTextBytes = 1...1_048_576
        public static let glyphs = 1...65536
        public static let clusters = 1...65536
        public static let hitRects = 1...128
        public static let carets = 0...128
        public static let sourceRanges = 1...128
        public static let diagnosticSources = 0...128
        public static let diagnosticMessageBytes = 1...4096
        public static let syntheticReasonBytes = 1...1024
        public static let postscriptNameBytes = 1...256
        public static let fontByteLength: ClosedRange<Int64> = 1...67_108_864
        public static let documentByteLength: ClosedRange<Int64> = 0...8_388_608
    }
    /// Features this consumer understands (schema `feature` enum).
    public static let knownFeatures: Set<String> = ["glyph_run", "rule", "static-truetype", "rgba-srgb", "cluster-actualtext"]
    /// Font formats whose bytes this consumer can paint from.
    public static let paintableFontFormats: Set<String> = ["static-truetype", "opentype-cff"]
    /// Formats the pipeline may declare that carry no program (never paintable).
    public static let metricsOnlyFontFormats: Set<String> = ["core14-afm"]

    public typealias SourceRange = RuntimeV1.SourceRange

    /// Ticks → PDF points (exact for |ticks| < 2^53).
    public static func points(_ ticks: Int64) -> Double { Double(ticks) / Double(ticksPerPoint) }

    public struct Paint: Codable, Hashable {
        public var r: Double, g: Double, b: Double, a: Double
        public init(r: Double, g: Double, b: Double, a: Double) { self.r = r; self.g = g; self.b = b; self.a = a }
        public static let black = Paint(r: 0, g: 0, b: 0, a: 1)
    }

    /// Hit rectangle: top-left anchored, y down, in ticks.
    public struct Rect: Codable, Hashable {
        public var x: Int64, top: Int64, width: Int64, height: Int64
        public init(x: Int64, top: Int64, width: Int64, height: Int64) { self.x = x; self.top = top; self.width = width; self.height = height }
        /// Half-open containment in ticks (`x <= px < x+width`, same for y).
        public func contains(x px: Int64, y py: Int64) -> Bool {
            px >= x && px < x &+ width && py >= top && py < top &+ height
        }
    }

    public struct Caret: Codable, Hashable {
        public var textByte: Int, x: Int64, top: Int64, height: Int64
        enum CodingKeys: String, CodingKey { case textByte = "text_byte", x, top, height }
        public init(textByte: Int, x: Int64, top: Int64, height: Int64) { self.textByte = textByte; self.x = x; self.top = top; self.height = height }
    }

    public struct Cluster: Codable, Equatable {
        public var textStartByte: Int
        public var textEndByte: Int
        public var hitRects: [Rect]
        public var carets: [Caret]
        public var sources: [SourceRange]?
        public var syntheticReason: String?
        enum CodingKeys: String, CodingKey {
            case textStartByte = "text_start_byte", textEndByte = "text_end_byte", hitRects = "hit_rects", carets, sources, syntheticReason = "synthetic_reason"
        }
        public init(textStartByte: Int, textEndByte: Int, hitRects: [Rect], carets: [Caret], sources: [SourceRange]?, syntheticReason: String? = nil) {
            self.textStartByte = textStartByte; self.textEndByte = textEndByte; self.hitRects = hitRects; self.carets = carets
            self.sources = sources; self.syntheticReason = syntheticReason
        }
    }

    public struct Glyph: Codable, Hashable {
        /// Original glyph ID in the run's font (never 0).
        public var gid: Int
        public var originX: Int64, baselineY: Int64, advanceX: Int64, advanceY: Int64
        public var cluster: Int
        enum CodingKeys: String, CodingKey {
            case gid, originX = "origin_x", baselineY = "baseline_y", advanceX = "advance_x", advanceY = "advance_y", cluster
        }
        public init(gid: Int, originX: Int64, baselineY: Int64, advanceX: Int64, advanceY: Int64, cluster: Int) {
            self.gid = gid; self.originX = originX; self.baselineY = baselineY; self.advanceX = advanceX; self.advanceY = advanceY; self.cluster = cluster
        }
    }

    public struct GlyphRun: Codable, Equatable {
        public var fontId: String
        public var fontSize: Int64
        public var text: String
        public var glyphs: [Glyph]
        public var clusters: [Cluster]
        public var paint: Paint
        enum CodingKeys: String, CodingKey { case fontId = "font_id", fontSize = "font_size", text, glyphs, clusters, paint }
        public init(fontId: String, fontSize: Int64, text: String, glyphs: [Glyph], clusters: [Cluster], paint: Paint) {
            self.fontId = fontId; self.fontSize = fontSize; self.text = text; self.glyphs = glyphs; self.clusters = clusters; self.paint = paint
        }
        /// The logical text of `cluster` (its UTF-8 byte range of `text`).
        public func clusterText(_ i: Int) -> String {
            let bytes = Array(text.utf8)
            guard i < clusters.count, clusters[i].textStartByte <= clusters[i].textEndByte, clusters[i].textEndByte <= bytes.count else { return "" }
            return String(decoding: bytes[clusters[i].textStartByte..<clusters[i].textEndByte], as: UTF8.self)
        }
    }

    public struct Rule: Codable, Equatable {
        public var x: Int64, top: Int64, width: Int64, height: Int64
        public var paint: Paint
        public var sources: [SourceRange]?
        public var syntheticReason: String?
        enum CodingKeys: String, CodingKey { case x, top, width, height, paint, sources, syntheticReason = "synthetic_reason" }
        public init(x: Int64, top: Int64, width: Int64, height: Int64, paint: Paint, sources: [SourceRange]?, syntheticReason: String? = nil) {
            self.x = x; self.top = top; self.width = width; self.height = height; self.paint = paint; self.sources = sources; self.syntheticReason = syntheticReason
        }
    }

    /// Paint-ordered page item. Decoding an unknown `kind` throws
    /// `ValidationError.unknownItemKind`: nothing is skipped silently.
    public enum Item: Codable, Equatable {
        case glyphRun(GlyphRun)
        case rule(Rule)

        private enum KindKey: String, CodingKey { case kind }

        public init(from decoder: Decoder) throws {
            let kind = try decoder.container(keyedBy: KindKey.self).decode(String.self, forKey: .kind)
            switch kind {
            case "glyph_run": self = .glyphRun(try GlyphRun(from: decoder))
            case "rule": self = .rule(try Rule(from: decoder))
            default: throw ValidationError(code: "unknown_item_kind", message: "display list item kind '\(kind)' is not supported by this consumer")
            }
        }

        public func encode(to encoder: Encoder) throws {
            var kind = encoder.container(keyedBy: KindKey.self)
            switch self {
            case .glyphRun(let r): try kind.encode("glyph_run", forKey: .kind); try r.encode(to: encoder)
            case .rule(let r): try kind.encode("rule", forKey: .kind); try r.encode(to: encoder)
            }
        }
    }

    public struct Page: Codable, Equatable {
        public var number: Int
        public var width: Int64
        public var height: Int64
        public var items: [Item]
        public init(number: Int, width: Int64, height: Int64, items: [Item]) { self.number = number; self.width = width; self.height = height; self.items = items }
        public var widthPt: Double { RenderingV2.points(width) }
        public var heightPt: Double { RenderingV2.points(height) }
    }

    public struct FontResource: Codable, Equatable {
        public var fontId: String
        public var sha256: String
        public var byteLength: Int64
        public var format: String
        public var faceIndex: Int
        public var unitsPerEm: Int
        public var glyphCount: Int
        public var postscriptName: String
        enum CodingKeys: String, CodingKey {
            case fontId = "font_id", sha256, byteLength = "byte_length", format, faceIndex = "face_index", unitsPerEm = "units_per_em", glyphCount = "glyph_count", postscriptName = "postscript_name"
        }
        public init(fontId: String, sha256: String, byteLength: Int64, format: String, faceIndex: Int, unitsPerEm: Int, glyphCount: Int, postscriptName: String) {
            self.fontId = fontId; self.sha256 = sha256; self.byteLength = byteLength; self.format = format; self.faceIndex = faceIndex
            self.unitsPerEm = unitsPerEm; self.glyphCount = glyphCount; self.postscriptName = postscriptName
        }
        public var isPaintable: Bool { RenderingV2.paintableFontFormats.contains(format) }
    }

    public struct DocumentResource: Codable, Equatable {
        public var path: String
        public var revision: Int
        public var sha256: String
        public var byteLength: Int64
        enum CodingKeys: String, CodingKey { case path, revision, sha256, byteLength = "byte_length" }
        public init(path: String, revision: Int, sha256: String, byteLength: Int64) { self.path = path; self.revision = revision; self.sha256 = sha256; self.byteLength = byteLength }
    }

    public struct Diagnostic: Codable, Equatable {
        public enum Severity: String, Codable { case warning, error }
        public var code: String
        public var message: String
        public var severity: Severity
        public var sources: [SourceRange]
        public init(code: String, message: String, severity: Severity, sources: [SourceRange]) { self.code = code; self.message = message; self.severity = severity; self.sources = sources }
        /// The runtime-v1 shape the shell's diagnostics list already renders.
        public var asRuntimeV1: RuntimeV1.Diagnostic {
            RuntimeV1.Diagnostic(severity: severity == .error ? .error : .warning, message: "[\(code)] \(message)", source: sources.first, recovery: nil)
        }
    }

    public struct DisplayList: Codable, Equatable {
        public var renderFormat: String
        public var coordinateUnit: String
        public var colorSpace: String
        public var textExtraction: String
        public var projectId: String
        public var revision: Int
        public var requiredFeatures: [String]
        public var documents: [DocumentResource]
        public var fonts: [FontResource]
        public var pages: [Page]
        public var diagnostics: [Diagnostic]
        enum CodingKeys: String, CodingKey {
            case renderFormat = "render_format", coordinateUnit = "coordinate_unit", colorSpace = "color_space", textExtraction = "text_extraction"
            case projectId = "project_id", revision, requiredFeatures = "required_features", documents, fonts, pages, diagnostics
        }
        public init(renderFormat: String = RenderingV2.renderFormat, coordinateUnit: String = RenderingV2.coordinateUnit,
                    colorSpace: String = RenderingV2.colorSpace, textExtraction: String = RenderingV2.textExtraction,
                    projectId: String, revision: Int, requiredFeatures: [String], documents: [DocumentResource],
                    fonts: [FontResource], pages: [Page], diagnostics: [Diagnostic]) {
            self.renderFormat = renderFormat; self.coordinateUnit = coordinateUnit; self.colorSpace = colorSpace; self.textExtraction = textExtraction
            self.projectId = projectId; self.revision = revision; self.requiredFeatures = requiredFeatures; self.documents = documents
            self.fonts = fonts; self.pages = pages; self.diagnostics = diagnostics
        }
        public func font(id: String) -> FontResource? { fonts.first { $0.fontId == id } }
    }

    public struct Envelope: Codable, Equatable {
        public var protocolVersion: Int
        public var id: String
        public var type: String
        public var payload: DisplayList
        enum CodingKeys: String, CodingKey { case protocolVersion = "protocol_version", id, type, payload }
        public init(protocolVersion: Int = RenderingV2.protocolVersion, id: String, type: String = RenderingV2.messageType, payload: DisplayList) {
            self.protocolVersion = protocolVersion; self.id = id; self.type = type; self.payload = payload
        }
    }

    /// A diagnostic-bearing refusal. The consumer never renders partially: any
    /// error here means no frame is published for this display list.
    public struct ValidationError: Error, Equatable, CustomStringConvertible {
        public var code: String
        public var message: String
        public var source: SourceRange?
        public init(code: String, message: String, source: SourceRange? = nil) { self.code = code; self.message = message; self.source = source }
        public var description: String { "\(code): \(message)" }
        public var asRuntimeV1: RuntimeV1.Diagnostic {
            RuntimeV1.Diagnostic(severity: .error, message: "[\(code)] \(message)", source: source, recovery: nil)
        }
    }

    // MARK: decoding + validation

    /// Header-only probe so version/type refusals name what was found rather
    /// than failing on an unrelated payload key.
    private struct Header: Decodable {
        var protocolVersion: Int?
        var id: String?
        var type: String?
        enum CodingKeys: String, CodingKey { case protocolVersion = "protocol_version", id, type }
    }

    /// Decodes and validates a `display_list` envelope. Fails closed: an unknown
    /// protocol version, message type, item kind, feature, font reference, or
    /// malformed geometry/cluster is an error, never a partial result.
    public static func decode(_ data: Data) throws -> Envelope {
        let decoder = JSONDecoder()
        let header: Header
        do { header = try decoder.decode(Header.self, from: data) } catch {
            throw ValidationError(code: "malformed_json", message: "not a JSON object: \(error.localizedDescription)")
        }
        guard let version = header.protocolVersion else {
            throw ValidationError(code: "missing_protocol_version", message: "protocol_version is required")
        }
        guard version == protocolVersion else {
            throw ValidationError(code: "unsupported_protocol_version", message: "protocol version \(version) is not supported; this consumer speaks rendering-v2 (protocol_version 2)")
        }
        guard let type = header.type else { throw ValidationError(code: "missing_type", message: "type is required") }
        guard type == messageType else {
            throw ValidationError(code: "unsupported_message_type", message: "message type '\(type)' is not a display_list")
        }
        let envelope: Envelope
        do { envelope = try decoder.decode(Envelope.self, from: data) } catch let e as ValidationError {
            throw e
        } catch let DecodingError.dataCorrupted(ctx) {
            if let inner = ctx.underlyingError as? ValidationError { throw inner }
            throw ValidationError(code: "malformed_payload", message: ctx.debugDescription)
        } catch {
            throw ValidationError(code: "malformed_payload", message: describe(error))
        }
        try validate(envelope.payload)
        return envelope
    }

    private static func describe(_ error: Error) -> String {
        switch error {
        case DecodingError.keyNotFound(let key, let ctx): return "missing key '\(key.stringValue)' at \(path(ctx))"
        case DecodingError.typeMismatch(_, let ctx): return "type mismatch at \(path(ctx)): \(ctx.debugDescription)"
        case DecodingError.valueNotFound(_, let ctx): return "null at \(path(ctx))"
        default: return error.localizedDescription
        }
    }
    private static func path(_ ctx: DecodingError.Context) -> String {
        ctx.codingPath.map { $0.intValue.map { "[\($0)]" } ?? $0.stringValue }.joined(separator: ".").replacingOccurrences(of: ".[", with: "[")
    }

    private static let hex64 = try! NSRegularExpression(pattern: "^[0-9a-f]{64}$")
    private static func isHex64(_ s: String) -> Bool {
        hex64.firstMatch(in: s, range: NSRange(location: 0, length: (s as NSString).length)) != nil
    }

    /// Semantic validation of a decoded payload (see `decode`).
    public static func validate(_ list: DisplayList) throws {
        func fail(_ code: String, _ message: String, _ source: SourceRange? = nil) -> ValidationError { ValidationError(code: code, message: message, source: source) }
        guard list.renderFormat == renderFormat else { throw fail("unsupported_format", "render_format '\(list.renderFormat)' is not \(renderFormat)") }
        guard list.coordinateUnit == coordinateUnit else { throw fail("unsupported_coordinate_unit", "coordinate_unit '\(list.coordinateUnit)' is not \(coordinateUnit)") }
        guard list.colorSpace == colorSpace else { throw fail("unsupported_color_space", "color_space '\(list.colorSpace)' is not \(colorSpace)") }
        guard list.textExtraction == textExtraction else { throw fail("unsupported_text_extraction", "text_extraction '\(list.textExtraction)' is not \(textExtraction)") }
        guard !list.requiredFeatures.isEmpty else { throw fail("invalid_display_list", "required_features must list at least one feature") }
        for f in list.requiredFeatures where !knownFeatures.contains(f) {
            throw fail("unsupported_feature", "required feature '\(f)' is not supported by this consumer")
        }
        guard list.revision >= 0, Int64(list.revision) <= maxExactInteger else { throw fail("invalid_display_list", "revision must be a nonnegative exact integer") }
        guard Bounds.documents.contains(list.documents.count) else { throw fail("invalid_display_list", "documents must declare 1...\(Bounds.documents.upperBound) source documents (found \(list.documents.count))") }
        guard Bounds.fonts.contains(list.fonts.count) else { throw fail("invalid_display_list", "fonts must declare at most \(Bounds.fonts.upperBound) resources (found \(list.fonts.count))") }
        guard Bounds.pages.contains(list.pages.count) else { throw fail("invalid_display_list", "at most \(Bounds.pages.upperBound) pages (found \(list.pages.count))") }
        guard Bounds.diagnostics.contains(list.diagnostics.count) else { throw fail("invalid_display_list", "at most \(Bounds.diagnostics.upperBound) diagnostics (found \(list.diagnostics.count))") }
        var documents: [String: DocumentResource] = [:]
        for d in list.documents {
            guard isProjectPath(d.path) else {
                throw fail("invalid_resource", "document path '\(d.path)' must be project-relative: no empty, '.' or '..' components, no backslash, colon or NUL")
            }
            guard documents.updateValue(d, forKey: d.path) == nil else { throw fail("invalid_resource", "document '\(d.path)' is declared twice") }
            guard isHex64(d.sha256) else { throw fail("invalid_resource", "document '\(d.path)' sha256 is not 64 lowercase hex digits") }
            guard d.revision >= 0, Int64(d.revision) <= maxExactInteger else { throw fail("invalid_resource", "document '\(d.path)' revision must be a nonnegative exact integer") }
            guard Bounds.documentByteLength.contains(d.byteLength) else { throw fail("invalid_resource", "document '\(d.path)' byte_length \(d.byteLength) is outside 0...\(Bounds.documentByteLength.upperBound)") }
        }
        var fontsById: [String: FontResource] = [:]
        for f in list.fonts {
            guard !f.fontId.isEmpty, fontsById.updateValue(f, forKey: f.fontId) == nil else {
                throw fail("invalid_resource", "font resource id '\(f.fontId)' is empty or declared twice")
            }
            guard isHex64(f.sha256) else { throw fail("invalid_resource", "font resource \(f.fontId) sha256 is not 64 lowercase hex digits") }
            guard paintableFontFormats.contains(f.format) || metricsOnlyFontFormats.contains(f.format) else {
                throw fail("unsupported_feature", "font resource \(f.fontId) format '\(f.format)' is not supported (paintable: \(paintableFontFormats.sorted().joined(separator: ", ")))")
            }
            guard f.faceIndex == 0 else { throw fail("unsupported_feature", "font resource \(f.fontId) face_index \(f.faceIndex): only face 0 is supported") }
            guard f.isPaintable ? Bounds.fontByteLength.contains(f.byteLength) : f.byteLength >= 0 else {
                throw fail("invalid_resource", "font resource \(f.fontId) byte_length \(f.byteLength) is outside \(Bounds.fontByteLength)")
            }
            guard (16...16384).contains(f.unitsPerEm) else { throw fail("invalid_resource", "font resource \(f.fontId) units_per_em \(f.unitsPerEm) is out of range") }
            guard (2...65536).contains(f.glyphCount) else { throw fail("invalid_resource", "font resource \(f.fontId) glyph_count \(f.glyphCount) is out of range") }
            guard Bounds.postscriptNameBytes.contains(f.postscriptName.utf8.count) else { throw fail("invalid_resource", "font resource \(f.fontId) postscript_name must be 1...\(Bounds.postscriptNameBytes.upperBound) bytes") }
        }
        // Features the list actually uses must all be declared (rendering-core:
        // "undeclared rendering feature"). `static-truetype` is deliberately not
        // derived from glyph runs: the pipeline declares it only for TrueType
        // resources and paints Latin Modern as `opentype-cff` (documented deviation).
        var usedFeatures: Set<String> = ["rgba-srgb", "cluster-actualtext"]
        var lastPage = 0
        for page in list.pages {
            guard page.number == lastPage + 1 else { throw fail("invalid_display_list", "page numbers must be contiguous from 1 (found \(page.number) after \(lastPage))") }
            lastPage = page.number
            guard isPositiveTick(page.width), isPositiveTick(page.height) else { throw fail("invalid_display_list", "page \(page.number) must have positive exact width and height") }
            guard Bounds.pageItems.contains(page.items.count) else { throw fail("invalid_display_list", "page \(page.number) has \(page.items.count) items (limit \(Bounds.pageItems.upperBound))") }
            for (index, item) in page.items.enumerated() {
                let at = "page \(page.number) item \(index)"
                switch item {
                case .rule(let r):
                    usedFeatures.insert("rule")
                    guard isTick(r.x), isTick(r.top), isPositiveTick(r.width), isPositiveTick(r.height),
                          isTick(r.x &+ r.width), isTick(r.top &+ r.height), isTick(page.height &- r.top &- r.height) else {
                        throw fail("invalid_display_list", "\(at): rule needs positive width/height and exact-range coordinates")
                    }
                    try validatePaint(r.paint, at)
                    try validateProvenance(sources: r.sources, synthetic: r.syntheticReason, documents: documents, at)
                case .glyphRun(let run):
                    usedFeatures.insert("glyph_run")
                    guard let font = fontsById[run.fontId] else { throw fail("invalid_resource", "\(at): font resource '\(run.fontId)' is not declared in fonts") }
                    guard isPositiveTick(run.fontSize) else { throw fail("invalid_display_list", "\(at): font_size must be a positive exact tick count") }
                    guard Bounds.runTextBytes.contains(run.text.utf8.count) else { throw fail("invalid_display_list", "\(at): glyph run text must be 1...\(Bounds.runTextBytes.upperBound) bytes") }
                    guard Bounds.glyphs.contains(run.glyphs.count) else { throw fail("invalid_display_list", "\(at): glyph run must carry 1...\(Bounds.glyphs.upperBound) glyphs (found \(run.glyphs.count))") }
                    guard Bounds.clusters.contains(run.clusters.count) else { throw fail("invalid_display_list", "\(at): glyph run must carry 1...\(Bounds.clusters.upperBound) clusters (found \(run.clusters.count))") }
                    try validatePaint(run.paint, at)
                    var referencedClusters = Set<Int>()
                    for (gi, g) in run.glyphs.enumerated() {
                        guard g.gid >= 1, g.gid < font.glyphCount else {
                            throw fail("invalid_display_list", "\(at) glyph \(gi): gid \(g.gid) is outside 1..<\(font.glyphCount) of font \(font.postscriptName)")
                        }
                        guard g.cluster >= 0, g.cluster < run.clusters.count else {
                            throw fail("invalid_display_list", "\(at) glyph \(gi): cluster \(g.cluster) is outside 0..<\(run.clusters.count)")
                        }
                        referencedClusters.insert(g.cluster)
                        guard isTick(g.originX), isTick(g.baselineY), isTick(g.advanceX), isTick(g.advanceY),
                              isTick(g.originX &+ g.advanceX), isTick(g.baselineY &+ g.advanceY), isTick(page.height &- g.baselineY) else {
                            throw fail("invalid_display_list", "\(at) glyph \(gi): origin/advance outside the exact tick range")
                        }
                    }
                    guard referencedClusters.count == run.clusters.count else {
                        throw fail("invalid_display_list", "\(at): \(run.clusters.count - referencedClusters.count) cluster(s) have no glyph (every cluster needs at least one glyph)")
                    }
                    let textBytes = Array(run.text.utf8)
                    var expectedStart = 0
                    for (ci, c) in run.clusters.enumerated() {
                        let cat = "\(at) cluster \(ci)"
                        guard c.textStartByte == expectedStart, c.textEndByte > c.textStartByte, c.textEndByte <= textBytes.count else {
                            throw fail("invalid_display_list", "\(cat): byte range \(c.textStartByte)..<\(c.textEndByte) does not partition the \(textBytes.count)-byte run text (expected a nonempty range starting at \(expectedStart))")
                        }
                        guard isBoundary(textBytes, c.textStartByte), isBoundary(textBytes, c.textEndByte) else {
                            throw fail("invalid_display_list", "\(cat): byte range \(c.textStartByte)..<\(c.textEndByte) splits a UTF-8 sequence")
                        }
                        expectedStart = c.textEndByte
                        guard Bounds.hitRects.contains(c.hitRects.count) else { throw fail("invalid_display_list", "\(cat): hit_rects must carry 1...\(Bounds.hitRects.upperBound) rectangles (found \(c.hitRects.count))") }
                        guard Bounds.carets.contains(c.carets.count) else { throw fail("invalid_display_list", "\(cat): at most \(Bounds.carets.upperBound) carets (found \(c.carets.count))") }
                        for r in c.hitRects {
                            guard isTick(r.x), isTick(r.top), isTick(r.width), isTick(r.height), r.width >= 0, r.height >= 0,
                                  isTick(r.x &+ r.width), isTick(r.top &+ r.height) else {
                                throw fail("invalid_display_list", "\(cat): hit rect has a negative size or coordinates outside the exact tick range")
                            }
                        }
                        for k in c.carets {
                            guard k.textByte >= c.textStartByte, k.textByte <= c.textEndByte, isBoundary(textBytes, k.textByte) else {
                                throw fail("invalid_display_list", "\(cat): caret text_byte \(k.textByte) is outside the cluster or splits a UTF-8 sequence")
                            }
                            guard isTick(k.x), isTick(k.top), isPositiveTick(k.height), isTick(k.top &+ k.height) else {
                                throw fail("invalid_display_list", "\(cat): caret needs a positive height and exact-range coordinates")
                            }
                        }
                        try validateProvenance(sources: c.sources, synthetic: c.syntheticReason, documents: documents, cat)
                    }
                    guard expectedStart == textBytes.count else {
                        throw fail("invalid_display_list", "\(at): clusters cover \(expectedStart) of \(textBytes.count) text bytes")
                    }
                }
            }
        }
        for d in list.diagnostics {
            guard !d.code.isEmpty, Bounds.diagnosticMessageBytes.contains(d.message.utf8.count) else {
                throw fail("invalid_display_list", "diagnostics must carry a code and a 1...\(Bounds.diagnosticMessageBytes.upperBound)-byte message")
            }
            guard Bounds.diagnosticSources.contains(d.sources.count) else { throw fail("invalid_display_list", "diagnostic '\(d.code)' lists \(d.sources.count) sources (limit \(Bounds.diagnosticSources.upperBound))") }
            for s in d.sources { try validateSource(s, documents: documents, "diagnostic '\(d.code)'") }
        }
        let declared = Set(list.requiredFeatures)
        let undeclared = usedFeatures.subtracting(declared).sorted()
        guard undeclared.isEmpty else {
            throw fail("invalid_display_list", "the list uses feature(s) \(undeclared.joined(separator: ", ")) that required_features does not declare (\(list.requiredFeatures.joined(separator: ", ")))")
        }
    }

    /// `|t| <= 2^53 − 1`: representable exactly in JSON and in a Double.
    static func isTick(_ t: Int64) -> Bool { t >= -maxExactInteger && t <= maxExactInteger }
    static func isPositiveTick(_ t: Int64) -> Bool { t > 0 && t <= maxExactInteger }

    /// crates/rendering-core `path`: no backslash, colon or NUL, and no empty,
    /// `.` or `..` component (so no leading `/` either).
    static func isProjectPath(_ p: String) -> Bool {
        !p.isEmpty && !p.contains("\\") && !p.contains(":") && !p.contains("\0")
            && p.split(separator: "/", omittingEmptySubsequences: false).allSatisfy { !($0.isEmpty || $0 == "." || $0 == "..") }
    }

    private static func isBoundary(_ bytes: [UInt8], _ i: Int) -> Bool {
        i == bytes.count || (i >= 0 && i < bytes.count && (bytes[i] & 0xC0) != 0x80)
    }

    private static func validatePaint(_ p: Paint, _ at: String) throws {
        for v in [p.r, p.g, p.b, p.a] where !(v >= 0 && v <= 1) {
            throw ValidationError(code: "invalid_display_list", message: "\(at): paint components must be within 0...1")
        }
    }

    /// A source range must name a declared document and lie within its
    /// declared byte length (rendering-core `source`).
    private static func validateSource(_ s: SourceRange, documents: [String: DocumentResource], _ at: String) throws {
        guard let doc = documents[s.path] else {
            throw ValidationError(code: "invalid_display_list", message: "\(at): source path '\(s.path)' is not a declared document", source: s)
        }
        guard s.startByte >= 0, s.endByte >= s.startByte, Int64(s.endByte) <= doc.byteLength else {
            throw ValidationError(code: "invalid_display_list", message: "\(at): source range \(s.startByte)..<\(s.endByte) is malformed or outside \(s.path)'s \(doc.byteLength) bytes", source: s)
        }
    }

    private static func validateProvenance(sources: [SourceRange]?, synthetic: String?, documents: [String: DocumentResource], _ at: String) throws {
        switch (sources, synthetic) {
        case (nil, nil): throw ValidationError(code: "invalid_display_list", message: "\(at): needs sources or synthetic_reason")
        case (.some, .some): throw ValidationError(code: "invalid_display_list", message: "\(at): sources and synthetic_reason are mutually exclusive")
        case (nil, .some(let reason)):
            guard Bounds.syntheticReasonBytes.contains(reason.utf8.count) else { throw ValidationError(code: "invalid_display_list", message: "\(at): synthetic_reason must be 1...\(Bounds.syntheticReasonBytes.upperBound) bytes") }
        case (.some(let ranges), nil):
            guard Bounds.sourceRanges.contains(ranges.count) else { throw ValidationError(code: "invalid_display_list", message: "\(at): sources must list 1...\(Bounds.sourceRanges.upperBound) ranges (found \(ranges.count))") }
            for s in ranges { try validateSource(s, documents: documents, at) }
        }
    }
}
