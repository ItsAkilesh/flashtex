import Foundation

/// Models for FlashTeX runtime protocol v1 (docs/contracts/runtime-v1.md).
/// Envelope: `protocol_version`, `id`, `type`, `payload`. Replies preserve `id`.
public enum RuntimeV1 {
    public static let protocolVersion = 1

    public struct Envelope<Payload: Codable>: Codable {
        public var protocolVersion: Int
        public var id: String
        public var type: String
        public var payload: Payload

        enum CodingKeys: String, CodingKey {
            case protocolVersion = "protocol_version", id, type, payload
        }
        public init(protocolVersion: Int, id: String, type: String, payload: Payload) {
            self.protocolVersion = protocolVersion; self.id = id; self.type = type; self.payload = payload
        }
    }

    // MARK: compile

    public struct Document: Codable, Equatable {
        public var path: String
        public var text: String
        public init(path: String, text: String) { self.path = path; self.text = text }
    }

    public struct CompileRequest: Codable, Equatable {
        public var projectId: String
        public var revision: Int
        public var entryPath: String
        public var documents: [Document]

        enum CodingKeys: String, CodingKey {
            case projectId = "project_id", revision, entryPath = "entry_path", documents
        }
        public init(projectId: String, revision: Int, entryPath: String, documents: [Document]) {
            self.projectId = projectId; self.revision = revision
            self.entryPath = entryPath; self.documents = documents
        }
    }

    // MARK: compile_result

    public enum Status: String, Codable { case ok, recovered, failed }

    /// Zero-based, end-exclusive UTF-8 byte offsets into the input revision.
    public struct SourceRange: Codable, Equatable {
        public var path: String
        public var startByte: Int
        public var endByte: Int

        enum CodingKeys: String, CodingKey {
            case path, startByte = "start_byte", endByte = "end_byte"
        }
        public init(path: String, startByte: Int, endByte: Int) {
            self.path = path; self.startByte = startByte; self.endByte = endByte
        }
    }

    /// Only `kind: text` exists in v1. Unknown kinds are decoded as `.unknown`
    /// so a newer contract revision does not crash the shell.
    public enum PageItem: Codable, Equatable {
        case text(TextItem)
        case unknown(kind: String)

        public struct TextItem: Codable, Equatable {
            public var text: String
            public var xPt: Double
            public var baselineYPt: Double
            public var fontSizePt: Double
            public var source: SourceRange?

            enum CodingKeys: String, CodingKey {
                case text, xPt = "x_pt", baselineYPt = "baseline_y_pt"
                case fontSizePt = "font_size_pt", source
            }
        }

        enum CodingKeys: String, CodingKey { case kind }

        public init(from decoder: Decoder) throws {
            let kind = try decoder.container(keyedBy: CodingKeys.self).decode(String.self, forKey: .kind)
            switch kind {
            case "text": self = .text(try TextItem(from: decoder))
            default: self = .unknown(kind: kind)
            }
        }

        public func encode(to encoder: Encoder) throws {
            var c = encoder.container(keyedBy: CodingKeys.self)
            switch self {
            case .text(let item):
                try c.encode("text", forKey: .kind)
                try item.encode(to: encoder)
            case .unknown(let kind):
                try c.encode(kind, forKey: .kind)
            }
        }
    }

    public struct Page: Codable, Equatable {
        public var number: Int
        public var widthPt: Double
        public var heightPt: Double
        public var items: [PageItem]

        enum CodingKeys: String, CodingKey {
            case number, widthPt = "width_pt", heightPt = "height_pt", items
        }
    }

    public enum Severity: String, Codable { case error, warning }

    public struct Diagnostic: Codable, Equatable {
        public var severity: Severity
        public var message: String
        public var source: SourceRange?
        public var recovery: String?
    }

    public struct CompileResult: Codable, Equatable {
        public var projectId: String
        public var revision: Int
        public var status: Status
        public var pages: [Page]
        public var diagnostics: [Diagnostic]
        public var pdfPath: String?

        enum CodingKeys: String, CodingKey {
            case projectId = "project_id", revision, status, pages, diagnostics
            case pdfPath = "pdf_path"
        }
        public init(projectId: String, revision: Int, status: Status, pages: [Page],
                    diagnostics: [Diagnostic], pdfPath: String?) {
            self.projectId = projectId; self.revision = revision; self.status = status
            self.pages = pages; self.diagnostics = diagnostics; self.pdfPath = pdfPath
        }
    }

    // MARK: decoding

    public enum DecodeError: Error, Equatable {
        case unsupportedVersion(Int)
        case unexpectedType(expected: String, actual: String)
    }

    public static func decodeCompileResult(_ data: Data) throws -> Envelope<CompileResult> {
        try decode(data, expectedType: "compile_result")
    }

    public static func decodeCompileRequest(_ data: Data) throws -> Envelope<CompileRequest> {
        try decode(data, expectedType: "compile")
    }

    static func decode<P: Codable>(_ data: Data, expectedType: String) throws -> Envelope<P> {
        let env = try JSONDecoder().decode(Envelope<P>.self, from: data)
        guard env.protocolVersion == protocolVersion else {
            throw DecodeError.unsupportedVersion(env.protocolVersion)
        }
        guard env.type == expectedType else {
            throw DecodeError.unexpectedType(expected: expectedType, actual: env.type)
        }
        return env
    }
}
