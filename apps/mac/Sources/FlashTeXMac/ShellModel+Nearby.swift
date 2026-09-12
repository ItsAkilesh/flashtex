import Combine
import Foundation
import FlashTeXProtocol

/// Captures received from paired companions while no bridge is attached, kept
/// in memory only. Nothing here is journaled, so acknowledgements say
/// `durable: false`; with a bridge attached (`ShellModel+Bridge.swift`) the
/// capture is forwarded there instead and its durable acknowledgement returned.
@MainActor
final class NearbyInbox: ObservableObject {
    private(set) var received: [RuntimeV1.CaptureSubmit] = []
    private(set) var lastCaptureId: String?
    private(set) var lastNote: String?
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

    func noteForwarded(_ submit: RuntimeV1.CaptureSubmit, _ ack: NearbyV1.CaptureReceived) {
        lastCaptureId = submit.captureId
        lastNote = "Forwarded \(submit.captureId) to the bridge (durable: \(ack.durable))."
    }
}

extension NearbyV1.ErrorPayload: Error {}

extension ShellModel: CaptureSink, DestinationProvider {
    /// Accepts a capture from a paired companion while no bridge is attached.
    /// `durable` is false because only the bridge can promise a journaled receipt.
    func receiveNearbyCapture(_ submit: RuntimeV1.CaptureSubmit) -> Result<NearbyV1.CaptureReceived, NearbyV1.ErrorPayload> {
        nearbyInbox.store(submit)
    }

    /// Forwards to the attached bridge (transfer-v1 `capture_submit`) and
    /// returns its acknowledgement; bridge `error` envelopes pass through.
    func forwardNearbyCapture(_ submit: RuntimeV1.CaptureSubmit) async -> Result<NearbyV1.CaptureReceived, NearbyV1.ErrorPayload> {
        guard let bridge, bridge.running else { return receiveNearbyCapture(submit) }
        do {
            let ack = try await bridge.submit(submit)
            let received = NearbyV1.CaptureReceived(captureId: ack.captureId, durable: ack.durable,
                                                    hasProposal: ack.hasProposal, applied: ack.applied)
            nearbyInbox.noteForwarded(submit, received)
            return .success(received)
        } catch let f as BridgeClient.Failure {
            if case .bridge(let e) = f { return .failure(.init(code: e.code, message: e.message)) }
            return .failure(.init(code: "unavailable", message: "bridge: \(f.text)"))
        } catch {
            return .failure(.init(code: "unavailable", message: "bridge: \(error)"))
        }
    }

    /// The pinned anchor as the companion sees it, or nil when nothing is
    /// pinned. With a bridge attached the bridge's anchor is authoritative
    /// (`base_revision` = its pinned revision); otherwise the local anchor.
    var nearbyDestination: NearbyV1.Destination? {
        if let a = bridgeDestination, a.valid {
            return .init(destinationId: a.destinationId, projectId: a.projectId, path: a.path, baseRevision: a.pinnedRevision)
        }
        guard let anchor else { return nil }
        return .init(destinationId: anchor.id, projectId: projectId, path: anchor.path, baseRevision: anchor.revision)
    }

    // MARK: CaptureSink / DestinationProvider (called from the listener queue)

    nonisolated func submit(_ envelope: RuntimeV1.Envelope<RuntimeV1.CaptureSubmit>, reply: @escaping (Data) -> Void) {
        Task { @MainActor in
            switch await self.forwardNearbyCapture(envelope.payload) {
            case .success(let ack): reply(NearbyV1.line(id: envelope.id, type: "capture_received", ack))
            case .failure(let err): reply(NearbyV1.errorLine(id: envelope.id, code: err.code, message: err.message))
            }
        }
    }

    nonisolated func currentDestination(_ reply: @escaping (NearbyV1.Destination?) -> Void) {
        Task { @MainActor in reply(self.nearbyDestination) }
    }
}
