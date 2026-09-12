import Foundation

/// Capture and insertion messages from runtime v1 ("Capture and insertion").
public extension RuntimeV1 {
    struct CaptureImage: Codable, Equatable {
        public var mimeType: String
        public var dataBase64: String
        enum CodingKeys: String, CodingKey { case mimeType = "mime_type", dataBase64 = "data_base64" }
        public init(mimeType: String, dataBase64: String) { self.mimeType = mimeType; self.dataBase64 = dataBase64 }
    }

    struct CaptureSubmit: Codable, Equatable {
        public var captureId: String
        public var destinationId: String
        public var baseRevision: Int
        public var image: CaptureImage
        public var instructions: String
        enum CodingKeys: String, CodingKey {
            case captureId = "capture_id", destinationId = "destination_id"
            case baseRevision = "base_revision", image, instructions
        }
    }

    struct CaptureReceived: Codable, Equatable {
        public var captureId: String
        enum CodingKeys: String, CodingKey { case captureId = "capture_id" }
        public init(captureId: String) { self.captureId = captureId }
    }

    /// Proposed LaTeX for a capture. Never inserted automatically.
    struct CaptureProposal: Codable, Equatable {
        public var captureId: String
        public var latex: String
        public var ambiguities: [String]
        public var requiredDependencies: [String]
        enum CodingKeys: String, CodingKey {
            case captureId = "capture_id", latex, ambiguities
            case requiredDependencies = "required_dependencies"
        }
        public init(captureId: String, latex: String, ambiguities: [String], requiredDependencies: [String]) {
            self.captureId = captureId; self.latex = latex
            self.ambiguities = ambiguities; self.requiredDependencies = requiredDependencies
        }
    }

    static let acceptedCaptureMimeTypes: Set<String> = ["image/png", "image/jpeg"]

    static func decodeCaptureSubmit(_ data: Data) throws -> Envelope<CaptureSubmit> {
        try decode(data, expectedType: "capture_submit")
    }

    static func decodeCaptureProposal(_ data: Data) throws -> Envelope<CaptureProposal> {
        try decode(data, expectedType: "capture_proposal")
    }
}
