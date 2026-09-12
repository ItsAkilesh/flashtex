import Foundation
import NearbyClient

/// One capture the iPad drew or picked, and what the Mac said about it.
/// Status vocabulary is exactly what nearby-v1/transfer-v1 give a companion:
///   drafted  → sending → received {durable, has_proposal, applied}  |  refused {code}  |  disconnected (retry same id)
/// Conversion progress after the receipt (proposal ready, inserted, rejected)
/// is NOT carried back over nearby-v1 (proposal §6: "the companion only learns
/// capture_received"), and the Mac's listener answers an identical retry with
/// the *cached* acknowledgement (`NearbyAckMemory`), so re-sending is a
/// delivery guarantee, not a status probe. The list says so instead of guessing.
public struct CaptureRecord: Identifiable, Equatable {
    public enum Source: String, Equatable { case pencil = "Apple Pencil canvas", photo = "photo picker", sample = "bundled sample image", fixture = "1×1 PNG fixture" }
    public enum Status: Equatable {
        case drafted
        case sending(attempt: Int)
        case received(NearbyWire.CaptureReceived)
        case refused(code: String, message: String)
        case disconnected(reason: String, attempt: Int)
        case discarded

        public var label: String {
            switch self {
            case .drafted: return "drafted — not sent"
            case .sending(let n): return n == 1 ? "sending…" : "re-sending (attempt \(n), same capture_id)…"
            case .received(let r):
                var s = r.durable ? "received — journaled by the Mac's bridge (durable)" : "received — Mac inbox, not journaled (durable:false)"
                if r.applied { s += "; already applied" } else if r.hasProposal { s += "; proposal already existed" }
                return s
            case .refused(let code, _): return "refused by the Mac: \(code)"
            case .disconnected(let why, _): return "not acknowledged (\(why)) — retry re-sends the same capture_id"
            case .discarded: return "discarded before sending"
            }
        }
        public var isTerminal: Bool {
            switch self { case .received, .refused, .discarded: return true; default: return false }
        }
    }

    public let id: String // capture_id
    public var source: Source
    public var png: Data
    public var instructions: String
    public var destinationId: String?
    public var baseRevision: Int?
    public var status: Status
    public var createdAt: Date
    public var pixelSize: (width: Int, height: Int)?

    public static func == (a: CaptureRecord, b: CaptureRecord) -> Bool {
        a.id == b.id && a.status == b.status && a.instructions == b.instructions && a.png == b.png
    }

    public init(id: String = "cap-" + UUID().uuidString.lowercased(), source: Source, png: Data, instructions: String,
                pixelSize: (width: Int, height: Int)? = nil) {
        self.id = id; self.source = source; self.png = png; self.instructions = instructions
        self.status = .drafted; self.createdAt = Date(); self.pixelSize = pixelSize
    }
}

/// Sends captures through `MacLink` (transfer-v1 `capture_submit` over the
/// nearby session) and keeps one record per capture_id. Retries after a
/// disconnect re-send the identical payload with the same id; the Mac
/// de-duplicates (nearby-v1 §4).
public final class CaptureQueue {
    public private(set) var records: [CaptureRecord] = []
    public let link: MacLink
    public static let maxInstructionBytes = 4096

    public init(link: MacLink) { self.link = link }

    public func record(_ id: String) -> CaptureRecord? { records.first { $0.id == id } }

    private func update(_ id: String, _ f: (inout CaptureRecord) -> Void) {
        guard let i = records.firstIndex(where: { $0.id == id }) else { return }
        f(&records[i])
    }

    /// Client-side checks the Mac would fail anyway (`NearbyWire.checkImage`,
    /// instruction bound), before anything is queued.
    public static func validate(png: Data, instructions: String) -> String? {
        if let why = NearbyWire.checkImage(png, mimeType: "image/png") { return why }
        if instructions.utf8.count > maxInstructionBytes { return "instructions exceed \(maxInstructionBytes) bytes" }
        return nil
    }

    @discardableResult
    public func draft(_ r: CaptureRecord) -> CaptureRecord {
        records.insert(r, at: 0)
        return r
    }

    public func discard(_ id: String) {
        update(id) { if !$0.status.isTerminal { $0.status = .discarded } }
    }

    /// `capture_submit` → `capture_received`; any `error` reply is terminal for
    /// this payload (`refused`); a closed connection leaves the record
    /// retryable with the same id.
    public func send(_ id: String) async {
        guard let r = record(id), !r.status.isTerminal else { return }
        let attempt: Int
        if case .disconnected(_, let n) = r.status { attempt = n + 1 } else { attempt = 1 }
        update(id) { $0.status = .sending(attempt: attempt) }
        do {
            guard let session = link.session else { throw NearbyError.closed("not connected to a Mac") }
            let dest = try await session.destinationQuery()
            guard let dest else {
                throw NearbyError.invalidInput("the Mac has no pinned insertion point (Edit > Pin Insertion Point on the Mac)")
            }
            update(id) { $0.destinationId = dest.destinationId; $0.baseRevision = dest.baseRevision }
            let submit = try session.makeCapture(captureId: id, image: r.png, mimeType: "image/png", instructions: r.instructions, destination: dest)
            let ack = try await session.submitCapture(submit)
            update(id) { $0.status = .received(ack) }
        } catch let e as NearbyError {
            switch e {
            case .remote(let code, let message) where e.needsNewCapture || e.isClosing:
                update(id) { $0.status = .refused(code: code, message: message) }
            case .closed, .timeout, .unreachable:
                update(id) { $0.status = .disconnected(reason: e.description, attempt: attempt) }
            case .invalidInput(let why):
                update(id) { $0.status = .refused(code: "invalid_input", message: why) }
            default:
                update(id) { $0.status = .disconnected(reason: e.description, attempt: attempt) }
            }
        } catch {
            update(id) { $0.status = .disconnected(reason: "\(error)", attempt: attempt) }
        }
    }
}
