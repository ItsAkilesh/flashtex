import Foundation
import FlashTeXProtocol

/// Messages of the proposed nearby companion transport (`docs/nearby-v1-proposal.md`).
/// Everything travels as runtime-v1 envelopes over an authenticated TLS-PSK
/// stream, one JSON object per line. `capture_submit` keeps its runtime-v1
/// payload unchanged; the types here are the nearby-only additions.
enum NearbyV1 {
    static let version = 1
    static let serviceType = "_flashtex._tcp"
    /// Same bound as the bridge (transfer-v1): a line including its newline.
    static let maxLineBytes = 12 * 1024 * 1024
    static let maxIDBytes = 128

    /// First line on every connection. `protocol_version` is the nearby
    /// protocol version (1), distinct from the envelope's runtime version.
    /// `proof` = `Pairing.helloProof(psk:nonce:)` binds the claimed `pair_id`
    /// to the PSK that opened the TLS session.
    struct Hello: Codable, Equatable {
        var pairId: String
        var companionName: String
        var protocolVersion: Int
        var nonce: String
        var proof: String
        enum CodingKeys: String, CodingKey {
            case pairId = "pair_id", companionName = "companion_name"
            case protocolVersion = "protocol_version", nonce, proof
        }
        init(pairId: String, companionName: String, protocolVersion: Int = NearbyV1.version, nonce: String, proof: String) {
            self.pairId = pairId; self.companionName = companionName
            self.protocolVersion = protocolVersion; self.nonce = nonce; self.proof = proof
        }
    }

    /// The Mac's current insertion destination, so the companion never types IDs.
    struct Destination: Codable, Equatable {
        var destinationId: String
        var projectId: String
        var path: String
        var baseRevision: Int
        enum CodingKeys: String, CodingKey {
            case destinationId = "destination_id", projectId = "project_id", path
            case baseRevision = "base_revision"
        }
        init(destinationId: String, projectId: String, path: String, baseRevision: Int) {
            self.destinationId = destinationId; self.projectId = projectId
            self.path = path; self.baseRevision = baseRevision
        }
    }

    /// `destination` is always present (explicit `null` when nothing is pinned).
    /// `pair_psk` (base64, 32 bytes) appears only on a bootstrap connection: the
    /// companion must persist it and use it for every later connection.
    struct HelloAck: Codable, Equatable {
        var macName: String
        var nonce: String
        var destination: Destination?
        var pairPsk: String?
        enum CodingKeys: String, CodingKey {
            case macName = "mac_name", nonce, destination, pairPsk = "pair_psk"
        }
        init(macName: String, nonce: String, destination: Destination?, pairPsk: String?) {
            self.macName = macName; self.nonce = nonce; self.destination = destination; self.pairPsk = pairPsk
        }
        func encode(to encoder: Encoder) throws {
            var c = encoder.container(keyedBy: CodingKeys.self)
            try c.encode(macName, forKey: .macName)
            try c.encode(nonce, forKey: .nonce)
            try c.encode(destination, forKey: .destination) // explicit null
            try c.encodeIfPresent(pairPsk, forKey: .pairPsk)
        }
    }

    struct DestinationReply: Codable, Equatable {
        var destination: Destination?
        init(destination: Destination?) { self.destination = destination }
        func encode(to encoder: Encoder) throws {
            var c = encoder.container(keyedBy: CodingKeys.self)
            try c.encode(destination, forKey: .destination)
        }
        enum CodingKeys: String, CodingKey { case destination }
    }

    /// transfer-v1 acknowledgement shape. `durable` is true only when the local
    /// bridge journaled the capture; the in-memory inbox always answers false.
    struct CaptureReceived: Codable, Equatable {
        var captureId: String
        var durable: Bool
        var hasProposal: Bool
        var applied: Bool
        enum CodingKeys: String, CodingKey {
            case captureId = "capture_id", durable, hasProposal = "has_proposal", applied
        }
        init(captureId: String, durable: Bool, hasProposal: Bool, applied: Bool) {
            self.captureId = captureId; self.durable = durable; self.hasProposal = hasProposal; self.applied = applied
        }
    }

    struct ErrorPayload: Codable, Equatable {
        var code: String
        var message: String
        init(code: String, message: String) { self.code = code; self.message = message }
    }

    struct Empty: Codable, Equatable { init() {} }

    // MARK: encoding helpers

    static func envelope<P: Codable>(id: String, type: String, _ payload: P) -> RuntimeV1.Envelope<P> {
        .init(protocolVersion: RuntimeV1.protocolVersion, id: id, type: type, payload: payload)
    }

    static func line<P: Codable>(id: String, type: String, _ payload: P) -> Data {
        // Encoding our own Codable structs cannot fail in practice; an empty line
        // would be a protocol violation, so fall back to a hand-written error.
        (try? RuntimeV1.encodeLine(envelope(id: id, type: type, payload)))
            ?? Data("{\"protocol_version\":1,\"id\":null,\"type\":\"error\",\"payload\":{\"code\":\"internal\",\"message\":\"encoding failed\"}}\n".utf8)
    }

    static func errorLine(id: String?, code: String, message: String) -> Data {
        // `id` is null when the request could not be identified (transfer-v1).
        let payload = ErrorPayload(code: code, message: message)
        if let id { return line(id: id, type: "error", payload) }
        struct NullID: Encodable {
            var protocolVersion: Int, type: String, payload: ErrorPayload
            enum CodingKeys: String, CodingKey { case protocolVersion = "protocol_version", id, type, payload }
            func encode(to encoder: Encoder) throws {
                var c = encoder.container(keyedBy: CodingKeys.self)
                try c.encode(protocolVersion, forKey: .protocolVersion)
                try c.encodeNil(forKey: .id)
                try c.encode(type, forKey: .type)
                try c.encode(payload, forKey: .payload)
            }
        }
        var data = (try? JSONEncoder().encode(NullID(protocolVersion: 1, type: "error", payload: payload))) ?? Data()
        data.append(0x0A)
        return data
    }

    static func isValidID(_ id: String) -> Bool {
        !id.isEmpty && id.utf8.count <= maxIDBytes
            && id.unicodeScalars.allSatisfy { $0.isASCII && (CharacterSet.alphanumerics.contains($0) || $0 == "-" || $0 == "_") }
    }
}

// MARK: - host-side hooks

/// Receives authenticated captures. `reply` gets one encoded line: a
/// `capture_received` or an `error` envelope carrying the request's id.
protocol CaptureSink: AnyObject {
    func submit(_ envelope: RuntimeV1.Envelope<RuntimeV1.CaptureSubmit>, reply: @escaping (Data) -> Void)
}

/// Answers `hello`/`destination_query` with the Mac's current anchor.
protocol DestinationProvider: AnyObject {
    func currentDestination(_ reply: @escaping (NearbyV1.Destination?) -> Void)
}

/// Turns a bootstrap (code-derived) connection into a long-term pairing.
/// Returns the freshly minted 32-byte PSK the companion must switch to.
protocol PairingConfirmer: AnyObject {
    func confirmPairing(pairId: String, companionName: String) -> Data?
    func notePairSeen(pairId: String)
}

// MARK: - per-connection session

/// Message handling for one authenticated connection, independent of
/// Network.framework so it can be driven with bytes in tests. TLS has already
/// authenticated the peer with one of `keys`; this layer enforces `hello`
/// first (with its proof naming which key), ids, sizes and message types, and
/// never parses more than one envelope per line.
final class NearbySession {
    enum Disposition: Equatable { case keepOpen, closeAfterFlush(String) }
    enum Event: Equatable {
        case hello(pairId: String, companionName: String, bootstrap: Bool)
        case capture(captureId: String)
        case violation(String)
    }

    let keys: [NearbyListener.PSKEntry]
    let macName: String
    private(set) var pairId: String?
    private(set) var isBootstrap = false
    /// Listener-wide freshness check; TLS already prevents cross-session replay,
    /// this only refuses a companion re-sending the same hello nonce.
    private let acceptNonce: (String) -> Bool
    private weak var sink: CaptureSink?
    private weak var destinations: DestinationProvider?
    private weak var pairing: PairingConfirmer?
    private let events: (Event) -> Void

    init(keys: [NearbyListener.PSKEntry], macName: String, sink: CaptureSink?,
         destinations: DestinationProvider?, pairing: PairingConfirmer?,
         acceptNonce: @escaping (String) -> Bool = { _ in true }, events: @escaping (Event) -> Void) {
        self.acceptNonce = acceptNonce
        self.keys = keys
        self.macName = macName
        self.sink = sink
        self.destinations = destinations
        self.pairing = pairing
        self.events = events
    }

    var helloCompleted: Bool { pairId != nil }

    /// Handles one complete line (without its newline). `emit` may be called
    /// synchronously or later (capture acknowledgements come from the sink).
    func handle(line: Data, emit: @escaping (Data) -> Void) -> Disposition {
        let header: RuntimeV1.EnvelopeHeader
        do { header = try RuntimeV1.header(of: line) } catch {
            emit(NearbyV1.errorLine(id: nil, code: "bad_request", message: "undecodable envelope"))
            events(.violation("undecodable envelope"))
            return .closeAfterFlush("undecodable envelope")
        }
        guard header.protocolVersion == RuntimeV1.protocolVersion else {
            emit(NearbyV1.errorLine(id: header.id, code: "unsupported_version",
                                    message: "protocol_version \(header.protocolVersion) is not supported"))
            return .closeAfterFlush("unsupported protocol_version")
        }
        guard !header.id.isEmpty, header.id.utf8.count <= NearbyV1.maxIDBytes else {
            emit(NearbyV1.errorLine(id: nil, code: "bad_request", message: "id must be 1–128 bytes"))
            return .closeAfterFlush("bad id")
        }
        let id = header.id
        guard helloCompleted || header.type == "hello" else {
            emit(NearbyV1.errorLine(id: id, code: "hello_required", message: "first message must be hello"))
            events(.violation("\(header.type) before hello"))
            return .closeAfterFlush("hello required")
        }
        switch header.type {
        case "hello":
            return handleHello(line: line, id: id, emit: emit)
        case "destination_query":
            let destinations = destinations
            guard let destinations else {
                emit(NearbyV1.line(id: id, type: "destination", NearbyV1.DestinationReply(destination: nil)))
                return .keepOpen
            }
            destinations.currentDestination { dest in
                emit(NearbyV1.line(id: id, type: "destination", NearbyV1.DestinationReply(destination: dest)))
            }
            return .keepOpen
        case "capture_submit":
            let env: RuntimeV1.Envelope<RuntimeV1.CaptureSubmit>
            do { env = try RuntimeV1.decodeCaptureSubmit(line) } catch {
                emit(NearbyV1.errorLine(id: id, code: "bad_request", message: "undecodable capture_submit: \(error)"))
                return .keepOpen
            }
            let p = env.payload
            guard NearbyV1.isValidID(p.captureId), NearbyV1.isValidID(p.destinationId) else {
                emit(NearbyV1.errorLine(id: id, code: "bad_request", message: "capture_id/destination_id must be 1–128 ASCII alphanumerics, - or _"))
                return .keepOpen
            }
            guard RuntimeV1.acceptedCaptureMimeTypes.contains(p.image.mimeType) else {
                emit(NearbyV1.errorLine(id: id, code: "unsupported_image", message: "mime_type \(p.image.mimeType) is not accepted"))
                return .keepOpen
            }
            guard p.instructions.utf8.count <= 4096 else {
                emit(NearbyV1.errorLine(id: id, code: "bad_request", message: "instructions exceed 4096 bytes"))
                return .keepOpen
            }
            guard let sink else {
                emit(NearbyV1.errorLine(id: id, code: "unavailable", message: "no capture sink attached"))
                return .keepOpen
            }
            let events = events
            sink.submit(env) { reply in
                events(.capture(captureId: p.captureId))
                emit(reply)
            }
            return .keepOpen
        default:
            emit(NearbyV1.errorLine(id: id, code: "unknown_type", message: "unknown message type \(header.type)"))
            return .keepOpen
        }
    }

    private func handleHello(line: Data, id: String, emit: @escaping (Data) -> Void) -> Disposition {
        guard !helloCompleted else {
            emit(NearbyV1.errorLine(id: id, code: "bad_request", message: "hello already completed"))
            return .closeAfterFlush("duplicate hello")
        }
        let env: RuntimeV1.Envelope<NearbyV1.Hello>
        do { env = try JSONDecoder().decode(RuntimeV1.Envelope<NearbyV1.Hello>.self, from: line) } catch {
            emit(NearbyV1.errorLine(id: id, code: "bad_request", message: "undecodable hello: \(error)"))
            return .closeAfterFlush("undecodable hello")
        }
        let hello = env.payload
        guard hello.protocolVersion == NearbyV1.version else {
            emit(NearbyV1.errorLine(id: id, code: "unsupported_version", message: "nearby protocol_version \(hello.protocolVersion) is not supported"))
            return .closeAfterFlush("unsupported nearby version")
        }
        guard !hello.nonce.isEmpty, hello.nonce.utf8.count <= 128 else {
            emit(NearbyV1.errorLine(id: id, code: "bad_request", message: "nonce must be a 1–128 byte string"))
            return .closeAfterFlush("bad nonce")
        }
        // The claimed pair must hold the key that opened this session; a paired
        // device cannot speak for another pairing (or for a pending one).
        guard let entry = keys.first(where: { $0.identity == hello.pairId }),
              Pairing.verifyHelloProof(hello.proof, psk: entry.key, nonce: hello.nonce) else {
            emit(NearbyV1.errorLine(id: id, code: "pair_mismatch", message: "pair_id is unknown or proof does not match its key"))
            events(.violation("hello for \(hello.pairId) failed proof"))
            return .closeAfterFlush("pair mismatch")
        }
        guard acceptNonce(hello.pairId + ":" + hello.nonce) else {
            emit(NearbyV1.errorLine(id: id, code: "bad_request", message: "hello nonce was already used"))
            events(.violation("stale hello nonce"))
            return .closeAfterFlush("stale nonce")
        }
        isBootstrap = entry.isBootstrap
        let name = String(hello.companionName.prefix(64))
        var pairPsk: String?
        if isBootstrap {
            guard let psk = pairing?.confirmPairing(pairId: hello.pairId, companionName: name) else {
                emit(NearbyV1.errorLine(id: id, code: "pairing_expired", message: "pairing code is no longer valid"))
                return .closeAfterFlush("pairing expired")
            }
            pairPsk = psk.base64EncodedString()
        } else {
            pairing?.notePairSeen(pairId: hello.pairId)
        }
        pairId = hello.pairId
        events(.hello(pairId: hello.pairId, companionName: name, bootstrap: isBootstrap))
        let macName = macName
        let nonce = hello.nonce
        let reply: (NearbyV1.Destination?) -> Void = { dest in
            emit(NearbyV1.line(id: id, type: "hello_ack",
                               NearbyV1.HelloAck(macName: macName, nonce: nonce, destination: dest, pairPsk: pairPsk)))
        }
        if let destinations { destinations.currentDestination(reply) } else { reply(nil) }
        return .keepOpen
    }
}
