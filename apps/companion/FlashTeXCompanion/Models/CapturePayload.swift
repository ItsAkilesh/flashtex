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

    static func create(captureID: String, destinationID: String, baseRevision: Int, image: UIImage, instructions: String) -> CaptureEnvelope? {
        guard let pngData = image.pngData() else { return nil }
        return create(
            captureID: captureID,
            destinationID: destinationID,
            baseRevision: baseRevision,
            imageData: pngData,
            mimeType: "image/png",
            instructions: instructions
        )
    }

    static func create(
        captureID: String,
        destinationID: String,
        baseRevision: Int,
        imageData: Data,
        mimeType: String,
        instructions: String
    ) -> CaptureEnvelope {
        let base64 = imageData.base64EncodedString()

        return CaptureEnvelope(
            protocolVersion: 1,
            id: UUID().uuidString,
            type: "capture_submit",
            payload: CaptureSubmitPayload(
                captureID: captureID,
                destinationID: destinationID,
                baseRevision: baseRevision,
                image: CaptureImage(mimeType: mimeType, dataBase64: base64),
                instructions: instructions
            )
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
