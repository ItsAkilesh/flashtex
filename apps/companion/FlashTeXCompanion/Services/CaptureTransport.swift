import Foundation

/// Outputs capture_submit payloads as JSON Lines to stdout.
/// This is the transport layer for communicating with the Rust worker
/// via the runtime-v1 protocol. Each payload is one complete JSON object
/// per line, matching the JSON Lines convention.
final class CaptureTransport {
    static let shared = CaptureTransport()
    private init() {}

    /// Write a capture_submit envelope as a single JSON line to stdout.
    func send(_ envelope: CaptureEnvelope) {
        guard let json = envelope.toJSONString() else {
            fputs("error: failed to serialize capture envelope\n", stderr)
            return
        }
        // JSON Lines: one object per line, no trailing comma
        print(json)
        fflush(stdout)
    }

    /// Write a capture_submit from raw components.
    func submit(captureID: String, destinationID: String, baseRevision: Int,
                image: UIImage, instructions: String) {
        guard let envelope = CaptureEnvelope.create(
            captureID: captureID,
            destinationID: destinationID,
            baseRevision: baseRevision,
            image: image,
            instructions: instructions
        ) else {
            fputs("error: failed to create capture envelope for \(captureID)\n", stderr)
            return
        }
        send(envelope)
    }
}
