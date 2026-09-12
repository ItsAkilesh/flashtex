import Foundation
import SwiftUI

/// Manages capture state and history for the companion app.
/// Sends via BonjourTransport (network) when connected; falls back to CaptureTransport (stdout).
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
        var networkSent: Bool   // true = went over Wi-Fi, false = stdout fallback

        enum CaptureSource: String {
            case pencil = "Pencil Drawing"
            case camera = "Camera"
            case photoLibrary = "Photo Library"
        }
    }

    func addCapture(
        source: CaptureRecord.CaptureSource,
        image: UIImage,
        instructions: String = "Faithfully transcribe this handwriting or equation; preserve all notation."
    ) {
        lastError = nil

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

        // Prefer network transport; fall back to stdout (CaptureTransport)
        var networkSent = false
        if let json = envelope.toJSONString() {
            let sent = CaptureTransport.shared.send(envelope)
            if !sent {
                lastError = "Duplicate capture ID (already sent)"
                return
            }
            // Also try network
            networkSent = BonjourTransport.shared.send(json)
        }

        guard let json = envelope.toJSONString() else { return }

        let thumbSize = CGSize(width: 60, height: 60)
        let thumbnail = UIGraphicsImageRenderer(size: thumbSize).image { _ in
            validImage.draw(in: CGRect(origin: .zero, size: thumbSize))
        }

        let record = CaptureRecord(
            id: captureID,
            timestamp: Date(),
            source: source,
            payloadJSON: json,
            imageThumbnail: thumbnail,
            networkSent: networkSent
        )
        captures.insert(record, at: 0)
        lastPayloadJSON = json
        showPayloadPreview = true
    }
}
