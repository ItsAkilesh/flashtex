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
        public init(captureId: String, destinationId: String, baseRevision: Int, image: CaptureImage, instructions: String) {
            self.captureId = captureId; self.destinationId = destinationId; self.baseRevision = baseRevision
            self.image = image; self.instructions = instructions
        }
    }

    struct CaptureReceived: Codable, Equatable {
        public var captureId: String
        enum CodingKeys: String, CodingKey { case captureId = "capture_id" }
        public init(captureId: String) { self.captureId = captureId }
    }

    /// Proposed LaTeX for a capture. Never inserted automatically.
    /// `context_revision` (transfer-v1) is the source revision the bridge
    /// assembled context from; absent in plain runtime-v1 proposal files.
    struct CaptureProposal: Codable, Equatable {
        public var captureId: String
        public var latex: String
        public var ambiguities: [String]
        public var requiredDependencies: [String]
        public var contextRevision: Int?
        enum CodingKeys: String, CodingKey {
            case captureId = "capture_id", latex, ambiguities
            case requiredDependencies = "required_dependencies"
            case contextRevision = "context_revision"
        }
        public init(captureId: String, latex: String, ambiguities: [String], requiredDependencies: [String],
                    contextRevision: Int? = nil) {
            self.captureId = captureId; self.latex = latex
            self.ambiguities = ambiguities; self.requiredDependencies = requiredDependencies
            self.contextRevision = contextRevision
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
