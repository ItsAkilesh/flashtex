import Foundation
import SwiftUI

/// Manages capture state and history for the companion app.
@Observable
final class CaptureStore {
    var captures: [CaptureRecord] = []
    var currentDestinationID: String = "default-anchor"
    var currentBaseRevision: Int = 1
    var lastPayloadJSON: String?
    var showPayloadPreview: Bool = false

    struct CaptureRecord: Identifiable {
        let id: String
        let timestamp: Date
        let source: CaptureSource
        let payloadJSON: String

        enum CaptureSource: String {
            case pencil = "Pencil Drawing"
            case camera = "Camera"
            case photoLibrary = "Photo Library"
        }
    }

    func addCapture(source: CaptureRecord.CaptureSource, image: UIImage, instructions: String = "Faithfully transcribe the selected handwriting; preserve notation.") {
        let captureID = "capture-\(UUID().uuidString.prefix(8))"
        guard let envelope = CaptureEnvelope.create(
            captureID: captureID,
            destinationID: currentDestinationID,
            baseRevision: currentBaseRevision,
            image: image,
            instructions: instructions
        ) else { return }

        guard let json = envelope.toJSONString() else { return }

        let record = CaptureRecord(
            id: captureID,
            timestamp: Date(),
            source: source,
            payloadJSON: json
        )
        captures.insert(record, at: 0)
        lastPayloadJSON = json
        showPayloadPreview = true
    }
}
