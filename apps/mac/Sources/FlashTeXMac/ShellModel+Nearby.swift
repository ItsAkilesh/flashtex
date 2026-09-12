import Combine
import Foundation
import FlashTeXProtocol

/// Captures received from paired companions, kept in memory only. This is the
/// default `CaptureSink` until the bridge client (transfer-v1) replaces it:
/// nothing here is journaled, so acknowledgements say `durable: false`.
@MainActor
final class NearbyInbox: ObservableObject {
    @Published private(set) var received: [RuntimeV1.CaptureSubmit] = []
    @Published private(set) var lastCaptureId: String?
    @Published private(set) var lastNote: String?
    static let maxRetained = 50

    /// Stores a capture; a repeated `capture_id` with an identical payload is
    /// acknowledged again, a different payload for a known id is refused.
    func store(_ submit: RuntimeV1.CaptureSubmit) -> Result<NearbyV1.CaptureReceived, NearbyV1.ErrorPayload> {
        if let existing = received.first(where: { $0.captureId == submit.captureId }) {
            guard existing == submit else {
                lastNote = "Refused \(submit.captureId): different payload for a known capture_id."
                return .failure(.init(code: "capture_id_conflict", message: "capture_id \(submit.captureId) already received with a different payload"))
            }
            lastNote = "Duplicate \(submit.captureId) acknowledged again."
        } else {
            received.append(submit)
            if received.count > Self.maxRetained { received.removeFirst(received.count - Self.maxRetained) }
            lastNote = "Received \(submit.captureId) (\(submit.image.mimeType), \(submit.image.dataBase64.utf8.count) base64 bytes); not journaled."
        }
        lastCaptureId = submit.captureId
        return .success(.init(captureId: submit.captureId, durable: false, hasProposal: false, applied: false))
    }
}

extension NearbyV1.ErrorPayload: Error {}

extension ShellModel: CaptureSink, DestinationProvider {
    /// Accepts a capture from a paired companion. Returns the acknowledgement
    /// the listener sends back; `durable` is false because only the bridge can
    /// promise a journaled receipt.
    func receiveNearbyCapture(_ submit: RuntimeV1.CaptureSubmit) -> Result<NearbyV1.CaptureReceived, NearbyV1.ErrorPayload> {
        nearbyInbox.store(submit)
    }

    /// The pinned anchor as the companion sees it, or nil when nothing is pinned.
    var nearbyDestination: NearbyV1.Destination? {
        guard let anchor else { return nil }
        return .init(destinationId: anchor.id, projectId: result?.projectId ?? "demo",
                     path: anchor.path, baseRevision: anchor.revision)
    }

    // MARK: CaptureSink / DestinationProvider (called from the listener queue)

    nonisolated func submit(_ envelope: RuntimeV1.Envelope<RuntimeV1.CaptureSubmit>, reply: @escaping (Data) -> Void) {
        Task { @MainActor in
            switch self.receiveNearbyCapture(envelope.payload) {
            case .success(let ack): reply(NearbyV1.line(id: envelope.id, type: "capture_received", ack))
            case .failure(let err): reply(NearbyV1.errorLine(id: envelope.id, code: err.code, message: err.message))
            }
        }
    }

    nonisolated func currentDestination(_ reply: @escaping (NearbyV1.Destination?) -> Void) {
        Task { @MainActor in reply(self.nearbyDestination) }
    }
}
