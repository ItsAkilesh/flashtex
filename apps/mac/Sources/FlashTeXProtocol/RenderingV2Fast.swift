import Foundation

/// Typed byte-level reader for the rendering-v2 `display_list` envelope
/// (`RenderingV2.decode`'s fast path). `JSONDecoder` + `Codable` read a
/// 1.9 MB two-page envelope (4.7k glyphs) in ~75 ms — most of the per-
/// keystroke cost of the live v2 route; this reader builds the same
/// `RenderingV2` values directly from the bytes in a fraction of that.
///
/// Semantics follow `JSONDecoder` where it matters: RFC 8259 syntax only,
/// unknown keys ignored, duplicate keys keep the last value, integers must be
/// integer literals within `Int`, doubles are read with the correctly rounded
/// `Double(_:)`, `null` for an optional field is `nil`. Anything this reader
/// does not accept throws `RenderingV2Fast.Error` and the caller falls back to
/// `JSONDecoder`, whose error then stands — the fast path never changes which
/// inputs are rejected, only how fast valid ones are read. Validation
/// (`RenderingV2.validate`) runs on the result exactly as on the slow path.
public struct RenderingV2Fast {
    public struct Error: Swift.Error, CustomStringConvertible {
        public var offset: Int
        public var message: String
        public var description: String { "fast rendering-v2 reader: \(message) at byte \(offset)" }
    }

    private let b: UnsafeBufferPointer<UInt8>
    private var i = 0

    private init(_ bytes: UnsafeBufferPointer<UInt8>) { b = bytes }

    /// Reads one `display_list` envelope. Throws `Error` for anything outside
    /// the accepted syntax/shape, including an unknown item kind; the caller
    /// then falls back to the `Codable` path, whose diagnostics stand.
    public static func envelope(_ data: Data) throws -> RenderingV2.Envelope {
        try data.withUnsafeBytes { raw -> RenderingV2.Envelope in
            var p = RenderingV2Fast(raw.bindMemory(to: UInt8.self))
            p.ws()
            let env = try p.envelope()
            p.ws()
            guard p.i == p.b.count else { throw p.err("trailing characters") }
            return env
        }
    }

    /// Top-level `protocol_version`, `id` and `type` of an envelope line with the
    /// payload skipped by a string/nesting-aware byte scan (no number or string
    /// decoding): ~1 ms for a 2 MB line, against ~12 ms for a full parse. Nil
    /// when the line is not a JSON object with those three fields.
    public struct Header: Equatable {
        public var protocolVersion: Int
        public var id: String
        public var type: String
    }

    public static func header(_ data: Data) -> Header? {
        data.withUnsafeBytes { raw -> Header? in
            var p = RenderingV2Fast(raw.bindMemory(to: UInt8.self))
            var version: Int?, id: String?, type: String?
            do {
                try p.object { key, p in
                    switch key {
                    case "protocol_version": version = try p.int()
                    case "id": id = try p.string()
                    case "type": type = try p.string()
                    default: try p.skipRaw()
                    }
                }
            } catch { return nil }
            guard let version, let id, let type else { return nil }
            return Header(protocolVersion: version, id: id, type: type)
        }
    }

    /// Skips one value by bracket depth and string boundaries only.
    private mutating func skipRaw() throws {
        ws()
        guard i < b.count else { throw err("unexpected end") }
        var depth = 0
        repeat {
            let c = b[i]
            switch c {
            case 0x22: // string: skip to the closing quote, honoring escapes
                i += 1
                while i < b.count, b[i] != 0x22 { i += b[i] == 0x5C ? 2 : 1 }
                guard i < b.count else { throw err("unterminated string") }
                i += 1
            case 0x7B, 0x5B: depth += 1; i += 1
            case 0x7D, 0x5D: depth -= 1; i += 1
            case 0x2C where depth == 0: return
            default: i += 1
            }
        } while depth > 0 && i < b.count
        guard depth == 0 else { throw err("unbalanced value") }
        // A scalar value ends at ',' or '}' (handled by the caller); scan up to it.
        while i < b.count, b[i] != 0x2C, b[i] != 0x7D { i += 1 }
    }

    // MARK: envelope

    private mutating func envelope() throws -> RenderingV2.Envelope {
        var version: Int?, id: String?, type: String?, payload: RenderingV2.DisplayList?
        try object { key, p in
            switch key {
            case "protocol_version": version = try p.int()
            case "id": id = try p.string()
            case "type": type = try p.string()
            case "payload": payload = try p.displayList()
            default: try p.skip(depth: 1)
            }
        }
        guard let version else { throw err("missing protocol_version") }
        guard let id else { throw err("missing id") }
        guard let type else { throw err("missing type") }
        guard let payload else { throw err("missing payload") }
        return RenderingV2.Envelope(protocolVersion: version, id: id, type: type, payload: payload)
    }

    private mutating func displayList() throws -> RenderingV2.DisplayList {
        var renderFormat: String?, unit: String?, colorSpace: String?, extraction: String?
        var projectId: String?, revision: Int?, features: [String]?
        var documents: [RenderingV2.DocumentResource]?, fonts: [RenderingV2.FontResource]?
        var pages: [RenderingV2.Page]?, diagnostics: [RenderingV2.Diagnostic]?
        try object { key, p in
            switch key {
            case "render_format": renderFormat = try p.string()
            case "coordinate_unit": unit = try p.string()
            case "color_space": colorSpace = try p.string()
            case "text_extraction": extraction = try p.string()
            case "project_id": projectId = try p.string()
            case "revision": revision = try p.int()
            case "required_features": features = try p.array { try $0.string() }
            case "documents": documents = try p.array { try $0.document() }
            case "fonts": fonts = try p.array { try $0.font() }
            case "pages": pages = try p.array { try $0.page() }
            case "diagnostics": diagnostics = try p.array { try $0.diagnostic() }
            default: try p.skip(depth: 2)
            }
        }
        guard let renderFormat, let unit, let colorSpace, let extraction, let projectId, let revision,
              let features, let documents, let fonts, let pages, let diagnostics else { throw err("missing display_list field") }
        return RenderingV2.DisplayList(renderFormat: renderFormat, coordinateUnit: unit, colorSpace: colorSpace, textExtraction: extraction,
                                       projectId: projectId, revision: revision, requiredFeatures: features, documents: documents,
                                       fonts: fonts, pages: pages, diagnostics: diagnostics)
    }

    private mutating func document() throws -> RenderingV2.DocumentResource {
        var path: String?, revision: Int?, sha: String?, length: Int64?
        try object { key, p in
            switch key {
            case "path": path = try p.string()
            case "revision": revision = try p.int()
            case "sha256": sha = try p.string()
            case "byte_length": length = try p.int64()
            default: try p.skip(depth: 3)
            }
        }
        guard let path, let revision, let sha, let length else { throw err("missing document field") }
        return RenderingV2.DocumentResource(path: path, revision: revision, sha256: sha, byteLength: length)
    }

    private mutating func font() throws -> RenderingV2.FontResource {
        var id: String?, sha: String?, length: Int64?, format: String?, face: Int?, upem: Int?, count: Int?, name: String?
        try object { key, p in
            switch key {
            case "font_id": id = try p.string()
            case "sha256": sha = try p.string()
            case "byte_length": length = try p.int64()
            case "format": format = try p.string()
            case "face_index": face = try p.int()
            case "units_per_em": upem = try p.int()
            case "glyph_count": count = try p.int()
            case "postscript_name": name = try p.string()
            default: try p.skip(depth: 3)
            }
        }
        guard let id, let sha, let length, let format, let face, let upem, let count, let name else { throw err("missing font field") }
        return RenderingV2.FontResource(fontId: id, sha256: sha, byteLength: length, format: format, faceIndex: face,
                                        unitsPerEm: upem, glyphCount: count, postscriptName: name)
    }

    private mutating func diagnostic() throws -> RenderingV2.Diagnostic {
        var code: String?, message: String?, severity: RenderingV2.Diagnostic.Severity?, sources: [RenderingV2.SourceRange]?
        try object { key, p in
            switch key {
            case "code": code = try p.string()
            case "message": message = try p.string()
            case "severity":
                let s = try p.string()
                guard let v = RenderingV2.Diagnostic.Severity(rawValue: s) else { throw p.err("unknown severity \(s)") }
                severity = v
            case "sources": sources = try p.array { try $0.sourceRange() }
            default: try p.skip(depth: 3)
            }
        }
        guard let code, let message, let severity, let sources else { throw err("missing diagnostic field") }
        return RenderingV2.Diagnostic(code: code, message: message, severity: severity, sources: sources)
    }

    private mutating func page() throws -> RenderingV2.Page {
        var number: Int?, width: Int64?, height: Int64?, items: [RenderingV2.Item]?
        try object { key, p in
            switch key {
            case "number": number = try p.int()
            case "width": width = try p.int64()
            case "height": height = try p.int64()
            case "items": items = try p.array { try $0.item() }
            default: try p.skip(depth: 3)
            }
        }
        guard let number, let width, let height, let items else { throw err("missing page field") }
        return RenderingV2.Page(number: number, width: width, height: height, items: items)
    }

    private mutating func item() throws -> RenderingV2.Item {
        // All fields of both kinds are collected in one pass (keys may come in any order).
        var kind: String?
        var fontId: String?, fontSize: Int64?, text: String?, glyphs: [RenderingV2.Glyph]?, clusters: [RenderingV2.Cluster]?
        var paint: RenderingV2.Paint?
        var x: Int64?, top: Int64?, width: Int64?, height: Int64?
        var sources: [RenderingV2.SourceRange]?, synthetic: String?
        let start = i
        try object { key, p in
            switch key {
            case "kind": kind = try p.string()
            case "font_id": fontId = try p.string()
            case "font_size": fontSize = try p.int64()
            case "text": text = try p.string()
            case "glyphs": glyphs = try p.array { try $0.glyph() }
            case "clusters": clusters = try p.array { try $0.cluster() }
            case "paint": paint = try p.paint()
            case "x": x = try p.int64()
            case "top": top = try p.int64()
            case "width": width = try p.int64()
            case "height": height = try p.int64()
            case "sources": sources = try p.optionalArray { try $0.sourceRange() }
            case "synthetic_reason": synthetic = try p.optionalString()
            default: try p.skip(depth: 4)
            }
        }
        guard let kind else { throw Error(offset: start, message: "item without kind") }
        switch kind {
        case "glyph_run":
            guard let fontId, let fontSize, let text, let glyphs, let clusters, let paint else { throw Error(offset: start, message: "missing glyph_run field") }
            return .glyphRun(RenderingV2.GlyphRun(fontId: fontId, fontSize: fontSize, text: text, glyphs: glyphs, clusters: clusters, paint: paint))
        case "rule":
            guard let x, let top, let width, let height, let paint else { throw Error(offset: start, message: "missing rule field") }
            return .rule(RenderingV2.Rule(x: x, top: top, width: width, height: height, paint: paint, sources: sources, syntheticReason: synthetic))
        default:
            // Fall back so the slow path reports it after the header checks
            // (a runtime-v1 compile_result must be refused for its version, not its items).
            throw Error(offset: start, message: "item kind '\(kind)' is not supported by the fast reader")
        }
    }

    private mutating func glyph() throws -> RenderingV2.Glyph {
        var gid: Int?, ox: Int64?, by: Int64?, ax: Int64?, ay: Int64?, cluster: Int?
        try object { key, p in
            switch key {
            case "gid": gid = try p.int()
            case "origin_x": ox = try p.int64()
            case "baseline_y": by = try p.int64()
            case "advance_x": ax = try p.int64()
            case "advance_y": ay = try p.int64()
            case "cluster": cluster = try p.int()
            default: try p.skip(depth: 5)
            }
        }
        guard let gid, let ox, let by, let ax, let ay, let cluster else { throw err("missing glyph field") }
        return RenderingV2.Glyph(gid: gid, originX: ox, baselineY: by, advanceX: ax, advanceY: ay, cluster: cluster)
    }

    private mutating func cluster() throws -> RenderingV2.Cluster {
        var start: Int?, end: Int?, rects: [RenderingV2.Rect]?, carets: [RenderingV2.Caret]?
        var sources: [RenderingV2.SourceRange]?, synthetic: String?
        try object { key, p in
            switch key {
            case "text_start_byte": start = try p.int()
            case "text_end_byte": end = try p.int()
            case "hit_rects": rects = try p.array { try $0.rect() }
            case "carets": carets = try p.array { try $0.caret() }
            case "sources": sources = try p.optionalArray { try $0.sourceRange() }
            case "synthetic_reason": synthetic = try p.optionalString()
            default: try p.skip(depth: 5)
            }
        }
        guard let start, let end, let rects, let carets else { throw err("missing cluster field") }
        return RenderingV2.Cluster(textStartByte: start, textEndByte: end, hitRects: rects, carets: carets, sources: sources, syntheticReason: synthetic)
    }

    private mutating func rect() throws -> RenderingV2.Rect {
        var x: Int64?, top: Int64?, width: Int64?, height: Int64?
        try object { key, p in
            switch key {
            case "x": x = try p.int64()
            case "top": top = try p.int64()
            case "width": width = try p.int64()
            case "height": height = try p.int64()
            default: try p.skip(depth: 6)
            }
        }
        guard let x, let top, let width, let height else { throw err("missing rect field") }
        return RenderingV2.Rect(x: x, top: top, width: width, height: height)
    }

    private mutating func caret() throws -> RenderingV2.Caret {
        var byte: Int?, x: Int64?, top: Int64?, height: Int64?
        try object { key, p in
            switch key {
            case "text_byte": byte = try p.int()
            case "x": x = try p.int64()
            case "top": top = try p.int64()
            case "height": height = try p.int64()
            default: try p.skip(depth: 6)
            }
        }
        guard let byte, let x, let top, let height else { throw err("missing caret field") }
        return RenderingV2.Caret(textByte: byte, x: x, top: top, height: height)
    }

    private mutating func sourceRange() throws -> RenderingV2.SourceRange {
        var path: String?, start: Int?, end: Int?
        try object { key, p in
            switch key {
            case "path": path = try p.string()
            case "start_byte": start = try p.int()
            case "end_byte": end = try p.int()
            default: try p.skip(depth: 6)
            }
        }
        guard let path, let start, let end else { throw err("missing source range field") }
        return RenderingV2.SourceRange(path: path, startByte: start, endByte: end)
    }

    private mutating func paint() throws -> RenderingV2.Paint {
        var r: Double?, g: Double?, bl: Double?, a: Double?
        try object { key, p in
            switch key {
            case "r": r = try p.double()
            case "g": g = try p.double()
            case "b": bl = try p.double()
            case "a": a = try p.double()
            default: try p.skip(depth: 5)
            }
        }
        guard let r, let g, let bl, let a else { throw err("missing paint component") }
        return RenderingV2.Paint(r: r, g: g, b: bl, a: a)
    }

    // MARK: primitives

    private func err(_ message: String) -> Error { Error(offset: i, message: message) }

    private mutating func ws() {
        while i < b.count, b[i] == 0x20 || b[i] == 0x0A || b[i] == 0x0D || b[i] == 0x09 { i += 1 }
    }

    private mutating func expect(_ byte: UInt8) throws {
        guard i < b.count, b[i] == byte else { throw err("expected '\(Character(UnicodeScalar(byte)))'") }
        i += 1
    }

    /// `{ "key": value, ... }` — `body` reads each value (and must consume it).
    private mutating func object(_ body: (String, inout RenderingV2Fast) throws -> Void) throws {
        ws(); try expect(0x7B) // {
        ws()
        if i < b.count, b[i] == 0x7D { i += 1; return }
        while true {
            ws()
            let key = try string()
            ws(); try expect(0x3A) // :
            ws()
            try body(key, &self)
            ws()
            guard i < b.count else { throw err("unterminated object") }
            if b[i] == 0x2C { i += 1; continue }
            if b[i] == 0x7D { i += 1; return }
            throw err("expected ',' or '}'")
        }
    }

    private mutating func array<T>(_ element: (inout RenderingV2Fast) throws -> T) throws -> [T] {
        ws(); try expect(0x5B) // [
        var out: [T] = []
        ws()
        if i < b.count, b[i] == 0x5D { i += 1; return out }
        while true {
            ws()
            out.append(try element(&self))
            ws()
            guard i < b.count else { throw err("unterminated array") }
            if b[i] == 0x2C { i += 1; continue }
            if b[i] == 0x5D { i += 1; return out }
            throw err("expected ',' or ']'")
        }
    }

    private mutating func optionalArray<T>(_ element: (inout RenderingV2Fast) throws -> T) throws -> [T]? {
        ws()
        if try literalNull() { return nil }
        return try array(element)
    }

    private mutating func optionalString() throws -> String? {
        ws()
        if try literalNull() { return nil }
        return try string()
    }

    private mutating func literalNull() throws -> Bool {
        guard i + 4 <= b.count, b[i] == 0x6E else { return false }
        guard b[i + 1] == 0x75, b[i + 2] == 0x6C, b[i + 3] == 0x6C else { throw err("invalid literal") }
        i += 4
        return true
    }

    private mutating func string() throws -> String {
        ws(); try expect(0x22) // "
        let start = i
        // Fast path: no escapes → one UTF-8 validation over the slice.
        while i < b.count {
            let c = b[i]
            if c == 0x22 {
                guard let s = String(validatingUTF8Slice: b, start, i) else { throw err("invalid UTF-8 in string") }
                i += 1
                return s
            }
            if c == 0x5C { break }
            if c < 0x20 { throw err("control character in string") }
            i += 1
        }
        // Escapes present: decode into a scalar buffer.
        var out: [UInt8] = Array(b[start..<i])
        while i < b.count {
            let c = b[i]
            if c == 0x22 {
                i += 1
                guard let s = String(bytes: out, encoding: .utf8) else { throw err("invalid UTF-8 in string") }
                return s
            }
            if c < 0x20 { throw err("control character in string") }
            if c != 0x5C { out.append(c); i += 1; continue }
            i += 1
            guard i < b.count else { throw err("unterminated escape") }
            let e = b[i]; i += 1
            switch e {
            case 0x22: out.append(0x22)
            case 0x5C: out.append(0x5C)
            case 0x2F: out.append(0x2F)
            case 0x62: out.append(0x08)
            case 0x66: out.append(0x0C)
            case 0x6E: out.append(0x0A)
            case 0x72: out.append(0x0D)
            case 0x74: out.append(0x09)
            case 0x75:
                var scalar = try hex4()
                if (0xD800...0xDBFF).contains(scalar) {
                    guard i + 1 < b.count, b[i] == 0x5C, b[i + 1] == 0x75 else { throw err("unpaired surrogate") }
                    i += 2
                    let low = try hex4()
                    guard (0xDC00...0xDFFF).contains(low) else { throw err("invalid low surrogate") }
                    scalar = 0x10000 + ((scalar - 0xD800) << 10) + (low - 0xDC00)
                } else if (0xDC00...0xDFFF).contains(scalar) {
                    throw err("unpaired surrogate")
                }
                guard let u = Unicode.Scalar(scalar) else { throw err("invalid scalar") }
                out.append(contentsOf: Array(String(Character(u)).utf8))
            default: throw err("invalid escape")
            }
        }
        throw err("unterminated string")
    }

    private mutating func hex4() throws -> UInt32 {
        guard i + 4 <= b.count else { throw err("short \\u escape") }
        var v: UInt32 = 0
        for _ in 0..<4 {
            let c = b[i]; i += 1
            let d: UInt32
            switch c {
            case 0x30...0x39: d = UInt32(c - 0x30)
            case 0x41...0x46: d = UInt32(c - 0x41 + 10)
            case 0x61...0x66: d = UInt32(c - 0x61 + 10)
            default: throw err("invalid hex digit")
            }
            v = v << 4 | d
        }
        return v
    }

    /// Integer literal (`-?digits`, no fraction/exponent) within `Int64`.
    private mutating func int64() throws -> Int64 {
        ws()
        let start = i
        var negative = false
        if i < b.count, b[i] == 0x2D { negative = true; i += 1 }
        guard i < b.count, (0x30...0x39).contains(b[i]) else { throw err("expected integer") }
        if b[i] == 0x30, i + 1 < b.count, (0x30...0x39).contains(b[i + 1]) { throw err("leading zero") }
        var v: Int64 = 0
        while i < b.count, (0x30...0x39).contains(b[i]) {
            let d = Int64(b[i] - 0x30)
            let (m, o1) = v.multipliedReportingOverflow(by: 10)
            let (s, o2) = negative ? m.subtractingReportingOverflow(d) : m.addingReportingOverflow(d)
            guard !o1, !o2 else { throw Error(offset: start, message: "integer out of range") }
            v = s
            i += 1
        }
        if i < b.count, b[i] == 0x2E || b[i] == 0x65 || b[i] == 0x45 { throw Error(offset: start, message: "integer expected, found fraction/exponent") }
        return v
    }

    private mutating func int() throws -> Int {
        let v = try int64()
        guard let x = Int(exactly: v) else { throw err("integer out of range") }
        return x
    }

    /// JSON number → Double via the standard library's correctly rounded parser.
    private mutating func double() throws -> Double {
        ws()
        let start = i
        if i < b.count, b[i] == 0x2D { i += 1 }
        guard i < b.count, (0x30...0x39).contains(b[i]) else { throw err("expected number") }
        if b[i] == 0x30, i + 1 < b.count, (0x30...0x39).contains(b[i + 1]) { throw err("leading zero") }
        while i < b.count, (0x30...0x39).contains(b[i]) { i += 1 }
        if i < b.count, b[i] == 0x2E {
            i += 1
            guard i < b.count, (0x30...0x39).contains(b[i]) else { throw err("digit expected after '.'") }
            while i < b.count, (0x30...0x39).contains(b[i]) { i += 1 }
        }
        if i < b.count, b[i] == 0x65 || b[i] == 0x45 {
            i += 1
            if i < b.count, b[i] == 0x2B || b[i] == 0x2D { i += 1 }
            guard i < b.count, (0x30...0x39).contains(b[i]) else { throw err("digit expected in exponent") }
            while i < b.count, (0x30...0x39).contains(b[i]) { i += 1 }
        }
        guard let text = String(validatingUTF8Slice: b, start, i), let v = Double(text) else { throw Error(offset: start, message: "invalid number") }
        return v
    }

    /// Skips any value (used for unknown keys). Bounded nesting.
    private mutating func skip(depth: Int) throws {
        guard depth < 64 else { throw err("nesting too deep") }
        ws()
        guard i < b.count else { throw err("unexpected end") }
        switch b[i] {
        case 0x7B: try object { _, p in try p.skip(depth: depth + 1) }
        case 0x5B: _ = try array { p in try p.skip(depth: depth + 1) }
        case 0x22: _ = try string()
        case 0x74: try literal("true")
        case 0x66: try literal("false")
        case 0x6E: try literal("null")
        default: _ = try double()
        }
    }

    private mutating func literal(_ word: StaticString) throws {
        let n = word.utf8CodeUnitCount
        guard i + n <= b.count else { throw err("invalid literal") }
        var ok = true
        word.withUTF8Buffer { w in for k in 0..<n where b[i + k] != w[k] { ok = false } }
        guard ok else { throw err("invalid literal") }
        i += n
    }
}

private extension String {
    /// Validated UTF-8 from `bytes[start..<end]`, nil when invalid.
    init?(validatingUTF8Slice bytes: UnsafeBufferPointer<UInt8>, _ start: Int, _ end: Int) {
        guard let base = bytes.baseAddress else { self = ""; return }
        let slice = UnsafeBufferPointer(start: base + start, count: end - start)
        // Validate strictly (no U+FFFD substitution) before decoding.
        var it = slice.makeIterator()
        var decoder = UTF8()
        loop: while true {
            switch decoder.decode(&it) {
            case .scalarValue: continue
            case .emptyInput: break loop
            case .error: return nil
            }
        }
        self.init(decoding: slice, as: UTF8.self)
    }
}
