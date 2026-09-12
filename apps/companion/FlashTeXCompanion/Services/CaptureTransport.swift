import Foundation
import UIKit

/// Outputs capture_submit payloads as JSON Lines to stdout.
/// This is the transport layer for communicating with the Rust worker
/// via the runtime-v1 protocol. Each payload is one complete JSON object
/// per line, matching the JSON Lines convention.
///
/// Tracks sent capture IDs to prevent duplicate submissions per protocol requirement.
final class CaptureTransport {
    static let shared = CaptureTransport()
    private var sentCaptureIDs: Set<String> = []
    private let lock = NSLock()
    private init() {}

    /// Write a capture_submit envelope as a single JSON line to stdout.
    /// Returns false if this capture_id was already sent (duplicate prevention).
    @discardableResult
    func send(_ envelope: CaptureEnvelope) -> Bool {
        let captureID = envelope.payload.captureID

        lock.lock()
        let isNew = sentCaptureIDs.insert(captureID).inserted
        lock.unlock()

        guard isNew else {
            fputs("warning: duplicate capture_id \(captureID) suppressed\n", stderr)
            return false
        }

        guard let json = envelope.toJSONString() else {
            fputs("error: failed to serialize capture envelope \(captureID)\n", stderr)
            return false
        }

        // JSON Lines: one object per line, no trailing comma
        print(json)
        fflush(stdout)
        return true
    }

    /// Write a capture_submit from raw components.
    @discardableResult
    func submit(captureID: String, destinationID: String, baseRevision: Int,
                image: UIImage, instructions: String) -> Bool {
        guard let envelope = CaptureEnvelope.create(
            captureID: captureID,
            destinationID: destinationID,
            baseRevision: baseRevision,
            image: image,
            instructions: instructions
        ) else {
            fputs("error: failed to create capture envelope for \(captureID)\n", stderr)
            return false
        }
        return send(envelope)
    }

    /// Check if a capture ID has already been sent.
    func wasSent(_ captureID: String) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        return sentCaptureIDs.contains(captureID)
    }

    /// Number of unique captures sent this session.
    var sentCount: Int {
        lock.lock()
        defer { lock.unlock() }
        return sentCaptureIDs.count
    }
}
