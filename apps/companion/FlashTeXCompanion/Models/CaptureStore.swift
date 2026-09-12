import Foundation
import SwiftUI

/// Manages capture state and history for the companion app.
/// On each capture, validates the image, outputs the payload via CaptureTransport
/// (JSON Lines to stdout), and stores it locally for UI review.
@Observable
final class CaptureStore {
    var captures: [CaptureRecord] = []
    var currentDestinationID: String = "default-anchor"
    var currentBaseRevision: Int = 1
    var lastPayloadJSON: String?
    var showPayloadPreview: Bool = false
    var lastError: String?

    struct CaptureRecord: Identifiable {
        let id: String
        let timestamp: Date
        let source: CaptureSource
        let payloadJSON: String
        let imageThumbnail: UIImage?

        enum CaptureSource: String {
            case pencil = "Pencil Drawing"
            case camera = "Camera"
            case photoLibrary = "Photo Library"
        }
    }

    func addCapture(source: CaptureRecord.CaptureSource, image: UIImage, instructions: String = "Faithfully transcribe the selected handwriting; preserve notation.") {
        lastError = nil

        // Validate and constrain image
        let validation = ImageValidator.validate(image)
        guard validation.isValid, let validImage = validation.image else {
            lastError = validation.error ?? "Image validation failed"
            return
        }

        let captureID = "capture-\(UUID().uuidString.prefix(8))"
        guard let envelope = CaptureEnvelope.create(
            captureID: captureID,
            destinationID: currentDestinationID,
            baseRevision: currentBaseRevision,
            image: validImage,
            instructions: instructions
        ) else {
            lastError = "Failed to create capture envelope"
            return
        }

        // Send via transport (JSON Lines to stdout) with duplicate prevention
        let sent = CaptureTransport.shared.send(envelope)
        if !sent {
            lastError = "Duplicate capture ID (already sent)"
            return
        }

        guard let json = envelope.toJSONString() else { return }

        // Generate thumbnail for history list
        let thumbSize = CGSize(width: 60, height: 60)
        let thumbnail = UIGraphicsImageRenderer(size: thumbSize).image { _ in
            validImage.draw(in: CGRect(origin: .zero, size: thumbSize))
        }

        let record = CaptureRecord(
            id: captureID,
            timestamp: Date(),
            source: source,
            payloadJSON: json,
            imageThumbnail: thumbnail
        )
        captures.insert(record, at: 0)
        lastPayloadJSON = json
        showPayloadPreview = true
    }
}
