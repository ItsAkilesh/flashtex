import Foundation
import SwiftUI

// MARK: – UserDefaults keys

private enum Defaults {
    static let destinationID   = "ft.capturestore.destinationID"
    static let baseRevision    = "ft.capturestore.baseRevision"
}

// MARK: – CaptureStore

/// Manages capture state and history for the companion app.
///
/// **Review-before-send flow** (rev 3):
///   1. `stagePendingCapture(source:image:instructions:)` – validates and encodes
///      the image, builds the payload, then stores it as `pendingCapture` WITHOUT
///      sending.  The UI shows `PayloadPreviewView` for the user to inspect.
///   2. `confirmAndSend()` – called when the user taps "Send".  Deduplicates the
///      capture ID, dispatches to CaptureTransport / BonjourTransport, and commits
///      the record to history.
///   3. `discardPending()` – called when the user taps "Cancel" or dismisses the
///      preview.  The pending capture is dropped; no submission ID is consumed.
///
/// Cancellation is safe: if the user discards, the capture ID is never submitted to
/// `CaptureTransport`, so a retry (another `stagePendingCapture`) gets a fresh UUID
/// and there is no duplicate.
@Observable
final class CaptureStore {

    // MARK: Published state

    /// Ordered history — newest first.
    var captures: [CaptureRecord] = []

    /// Destination document anchor.  Persisted across launches via UserDefaults.
    var currentDestinationID: String {
        didSet { UserDefaults.standard.set(currentDestinationID, forKey: Defaults.destinationID) }
    }

    /// Base revision at the destination.  Persisted across launches.
    var currentBaseRevision: Int {
        didSet { UserDefaults.standard.set(currentBaseRevision, forKey: Defaults.baseRevision) }
    }

    /// Non-nil while the user is reviewing a staged capture before sending.
    var pendingCapture: PendingCapture?

    /// Last error to surface in the UI.
    var lastError: String?

    // MARK: Types

    struct PendingCapture {
        let captureID: String
        let source: CaptureRecord.CaptureSource
        let envelope: CaptureEnvelope
        let payloadJSON: String
        let thumbnail: UIImage
        /// The destination and revision locked at staging time.
        let destinationID: String
        let baseRevision: Int
    }

    struct CaptureRecord: Identifiable {
        let id: String
        let timestamp: Date
        let source: CaptureSource
        let payloadJSON: String
        let imageThumbnail: UIImage?
        /// `true` if the capture was also dispatched over the local network.
        var networkSent: Bool

        enum CaptureSource: String {
            case pencil = "Pencil Drawing"
            case camera = "Camera"
            case photoLibrary = "Photo Library"
        }
    }

    // MARK: Init

    init() {
        // Restore persisted destination settings, or use safe defaults.
        currentDestinationID = UserDefaults.standard.string(forKey: Defaults.destinationID)
            ?? "default-anchor"
        currentBaseRevision = UserDefaults.standard.integer(forKey: Defaults.baseRevision)
            .nonZeroOrDefault(1)
    }

    // MARK: Review-before-send pipeline

    /// Stage a capture for user review.  Nothing is sent until `confirmAndSend()`.
    ///
    /// - Parameters:
    ///   - source: Which input modality produced the image.
    ///   - image: The raw image as delivered by the picker / camera delegate.
    ///     Orientation normalisation happens here.
    ///   - instructions: Optional context the user typed; forwarded verbatim in the
    ///     payload `instructions` field.
    func stagePendingCapture(
        source: CaptureRecord.CaptureSource,
        image: UIImage,
        instructions: String = "Faithfully transcribe this handwriting or equation; preserve all notation."
    ) {
        lastError = nil
        pendingCapture = nil

        // Validate and orient-normalise.
        let validation = ImageValidator.validate(image)
        guard validation.isValid, let validImage = validation.image else {
            lastError = validation.error ?? "Image validation failed"
            return
        }

        // Snapshot destination ID and revision so review shows exactly what will be sent.
        let destID   = currentDestinationID
        let revision = currentBaseRevision
        let captureID = "capture-\(UUID().uuidString.prefix(8))"

        guard let envelope = CaptureEnvelope.create(
            captureID: captureID,
            destinationID: destID,
            baseRevision: revision,
            image: validImage,
            instructions: instructions
        ) else {
            lastError = "Failed to build capture payload"
            return
        }

        guard let json = envelope.toJSONString() else {
            lastError = "Failed to serialise payload"
            return
        }

        let thumbSize = CGSize(width: 60, height: 60)
        let thumbnail = UIGraphicsImageRenderer(size: thumbSize).image { _ in
            validImage.draw(in: CGRect(origin: .zero, size: thumbSize))
        }

        pendingCapture = PendingCapture(
            captureID: captureID,
            source: source,
            envelope: envelope,
            payloadJSON: json,
            thumbnail: thumbnail,
            destinationID: destID,
            baseRevision: revision
        )
    }

    /// Confirm the staged capture and dispatch it.
    ///
    /// Deduplication happens here (not at staging time) so the capture ID is only
    /// consumed once the user has confirmed.  If the user stages twice before
    /// confirming, only the most recent staging is present in `pendingCapture`.
    @discardableResult
    func confirmAndSend() -> Bool {
        guard let pending = pendingCapture else { return false }
        pendingCapture = nil
        lastError = nil

        let sent = CaptureTransport.shared.send(pending.envelope)
        if !sent {
            lastError = "Duplicate capture ID — not sent"
            return false
        }

        let networkSent = BonjourTransport.shared.send(pending.payloadJSON)

        let record = CaptureRecord(
            id: pending.captureID,
            timestamp: Date(),
            source: pending.source,
            payloadJSON: pending.payloadJSON,
            imageThumbnail: pending.thumbnail,
            networkSent: networkSent
        )
        captures.insert(record, at: 0)
        return true
    }

    /// Discard the staged capture without sending.
    ///
    /// The capture ID in the staging record is never submitted to `CaptureTransport`,
    /// so a subsequent `stagePendingCapture` gets a fresh UUID without triggering
    /// the deduplication guard.
    func discardPending() {
        pendingCapture = nil
        lastError = nil
    }

    // MARK: Legacy compatibility shim

    /// Convenience wrapper that immediately stages and sends without a review step.
    ///
    /// Kept for call-sites that do not yet use the review flow.  New code should
    /// prefer `stagePendingCapture` + `confirmAndSend`.
    func addCapture(
        source: CaptureRecord.CaptureSource,
        image: UIImage,
        instructions: String = "Faithfully transcribe this handwriting or equation; preserve all notation."
    ) {
        stagePendingCapture(source: source, image: image, instructions: instructions)
        if pendingCapture != nil {
            confirmAndSend()
        }
    }
}

// MARK: – Helpers

private extension Int {
    /// Returns `self` if nonzero, otherwise `default`.
    func nonZeroOrDefault(_ default: Int) -> Int {
        self == 0 ? `default` : self
    }
}
