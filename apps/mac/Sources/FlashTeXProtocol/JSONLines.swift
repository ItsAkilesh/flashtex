import Foundation

/// JSON Lines helpers for runtime v1: one envelope per line, UTF-8, no
/// embedded newlines. Peek at the header before decoding a typed payload.
public extension RuntimeV1 {
    struct EnvelopeHeader: Codable {
        public var protocolVersion: Int
        public var id: String
        public var type: String
        enum CodingKeys: String, CodingKey { case protocolVersion = "protocol_version", id, type }
    }

    struct ErrorPayload: Codable, Equatable {
        public var message: String
        public init(message: String) { self.message = message }
    }

    /// Default line size limit (bytes); oversized lines are rejected.
    static let maxLineBytes = 16 * 1024 * 1024

    static func encodeLine<P: Codable>(_ envelope: Envelope<P>) throws -> Data {
        let enc = JSONEncoder()
        enc.outputFormatting = [.withoutEscapingSlashes]
        var data = try enc.encode(envelope)
        data.append(0x0A)
        return data
    }

    static func header(of line: Data) throws -> EnvelopeHeader {
        try JSONDecoder().decode(EnvelopeHeader.self, from: line)
    }

    static func compileEnvelope(id: String, _ request: CompileRequest) -> Envelope<CompileRequest> {
        .init(protocolVersion: protocolVersion, id: id, type: "compile", payload: request)
    }
}

/// Splits a byte stream into complete lines, keeping a partial trailing line.
public struct LineSplitter {
    private var buffer = Data()
    public init() {}

    public mutating func append(_ data: Data) -> [Data] {
        buffer.append(data)
        var lines: [Data] = []
        while let nl = buffer.firstIndex(of: 0x0A) {
            lines.append(buffer.subdata(in: buffer.startIndex..<nl))
            buffer.removeSubrange(buffer.startIndex...nl)
        }
        return lines
    }

    public var pendingBytes: Int { buffer.count }
}
