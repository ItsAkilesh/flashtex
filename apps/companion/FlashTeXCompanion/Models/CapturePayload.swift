import Foundation
import UIKit

/// Runtime-v1 capture_submit message model.
/// Matches protocol/fixtures/capture-submission.json exactly.
struct CaptureEnvelope: Codable {
    let protocolVersion: Int
    let id: String
    let type: String
    let payload: CaptureSubmitPayload

    enum CodingKeys: String, CodingKey {
        case protocolVersion = "protocol_version"
        case id, type, payload
    }

    /// Create a capture envelope from pre-validated image data.
    ///
    /// Accepts raw image bytes + MIME type from `ImageValidator.encode()` so that
    /// the declared `mime_type` always matches the actual encoding on the wire.
    /// Passing the data directly avoids the re-encode-to-PNG bug that occurred when
    /// the validator's JPEG fallback was subsequently re-encoded by `UIImage.pngData()`.
    static func create(
        captureID: String,
        destinationID: String,
        baseRevision: Int,
        imageData: Data,
        mimeType: String,
        instructions: String
    ) -> CaptureEnvelope {
        CaptureEnvelope(
            protocolVersion: 1,
            id: UUID().uuidString,
            type: "capture_submit",
            payload: CaptureSubmitPayload(
                captureID: captureID,
                destinationID: destinationID,
                baseRevision: baseRevision,
                image: CaptureImage(
                    mimeType: mimeType,
                    dataBase64: imageData.base64EncodedString()
                ),
                instructions: instructions
            )
        )
    }

    /// Convenience overload: validates and encodes a UIImage, then creates the envelope.
    ///
    /// Returns `nil` only when `ImageValidator.encode(_:)` cannot produce any
    /// acceptable encoding (image is malformed or exceeds all size limits).
    static func create(
        captureID: String,
        destinationID: String,
        baseRevision: Int,
        image: UIImage,
        instructions: String
    ) -> CaptureEnvelope? {
        guard let (data, mime) = ImageValidator.encode(image) else { return nil }
        return create(
            captureID: captureID,
            destinationID: destinationID,
            baseRevision: baseRevision,
            imageData: data,
            mimeType: mime,
            instructions: instructions
        )
    }

    func toJSONString() -> String? {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        guard let data = try? encoder.encode(self) else { return nil }
        return String(data: data, encoding: .utf8)
    }
}

struct CaptureSubmitPayload: Codable {
    let captureID: String
    let destinationID: String
    let baseRevision: Int
    let image: CaptureImage
    let instructions: String

    enum CodingKeys: String, CodingKey {
        case captureID = "capture_id"
        case destinationID = "destination_id"
        case baseRevision = "base_revision"
        case image, instructions
    }
}

struct CaptureImage: Codable {
    let mimeType: String
    let dataBase64: String

    enum CodingKeys: String, CodingKey {
        case mimeType = "mime_type"
        case dataBase64 = "data_base64"
    }
}
