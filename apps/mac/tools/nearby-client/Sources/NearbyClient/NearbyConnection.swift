import Foundation
import Network

public enum NearbyError: Error, CustomStringConvertible, Equatable {
    case browseFailed(String)
    case noMatchingMac(String)
    case unsupportedService(String)
    /// TLS/TCP never reached `.ready` — wrong or forgotten PSK, code expired
    /// on the Mac, or a plaintext/other listener.
    case handshakeFailed(String)
    /// The Mac closed (or the network dropped) while a request was pending.
    case closed(String)
    /// An `error` reply. `closing` is true for codes the Mac follows with a close.
    case remote(code: String, message: String)
    case protocolViolation(String)
    case timeout(String)
    case invalidInput(String)

    public var description: String {
        switch self {
        case .browseFailed(let s): return "Bonjour browse failed: \(s)"
        case .noMatchingMac(let s): return "no matching Mac: \(s)"
        case .unsupportedService(let s): return "unsupported service: \(s)"
        case .handshakeFailed(let s): return "TLS-PSK handshake failed: \(s)"
        case .closed(let s): return "connection closed: \(s)"
        case .remote(let c, let m): return "Mac replied error \(c): \(m)"
        case .protocolViolation(let s): return "protocol violation: \(s)"
        case .timeout(let s): return "timed out: \(s)"
        case .invalidInput(let s): return "invalid input: \(s)"
        }
    }

    /// True when the Mac closes after this error (§8): re-pair or fix input.
    public var isClosing: Bool {
        if case .remote(let code, _) = self { return NearbyWire.closingErrorCodes.contains(code) }
        return false
    }
}

/// One TLS-PSK connection to a Mac carrying runtime-v1 JSON Lines
/// (proposal §3–§4). Requests are matched to replies by envelope `id`.
/// All Network.framework callbacks and state live on `queue`.
public final class NearbyConnection: @unchecked Sendable { // all mutable state is confined to `queue`
    public enum Direction { case sent, received }

    public let pairId: String
    public let psk: Data
    public let endpoint: NWEndpoint
    /// Bonjour service endpoints are resolved to host:port before dialing so a
    /// refused handshake is reported instead of hanging in `.preparing`
    /// (see NearbyResolver). Set false to dial the service endpoint directly.
    public var resolveServiceEndpoints = true
    /// The endpoint actually dialed (after resolution).
    public private(set) var dialed: NWEndpoint?
    private var nw: NWConnection!
    private let queue: DispatchQueue
    private var splitter = LineSplitter()
    private var pending: [String: (type: String, resume: (Result<Data, Error>) -> Void)] = [:]
    private var connectWaiter: ((Result<Void, Error>) -> Void)?
    private var lastWaitingError: NWError?
    private var isReady = false
    private var closedReason: String?
    /// Observes every line both ways (the CLI's `-v`). Called on `queue`.
    public var onLine: ((Direction, Data) -> Void)?
    /// Called once, on `queue`, when the connection ends for any reason.
    public var onClose: ((String) -> Void)?
    public private(set) var negotiated: (tlsv12: Bool, suite: UInt16)?

    public init(endpoint: NWEndpoint, pairId: String, psk: Data,
                queue: DispatchQueue = DispatchQueue(label: "flashtex.nearby.connection")) {
        self.pairId = pairId
        self.psk = psk
        self.queue = queue
        self.endpoint = endpoint
    }

    public convenience init(host: String, port: UInt16, pairId: String, psk: Data,
                            queue: DispatchQueue = DispatchQueue(label: "flashtex.nearby.connection")) {
        self.init(endpoint: .hostPort(host: NWEndpoint.Host(host), port: NWEndpoint.Port(rawValue: port)!),
                  pairId: pairId, psk: psk, queue: queue)
    }

    // MARK: lifecycle

    /// Starts the connection and returns once TLS is `.ready` (the PSK
    /// handshake succeeded). Throws `handshakeFailed` or `timeout` otherwise.
    public func connect(timeout: TimeInterval = 10) async throws {
        var target = endpoint
        if resolveServiceEndpoints, case .service = endpoint {
            target = try await NearbyResolver.resolve(endpoint, timeout: timeout).endpoint
        }
        let dial = target
        try await withCheckedThrowingContinuation { (cont: CheckedContinuation<Void, Error>) in
            queue.async {
                self.dialed = dial
                self.nw = NWConnection(to: dial, using: NearbyCrypto.parameters(pairId: self.pairId, psk: self.psk))
                self.connectWaiter = { cont.resume(with: $0) }
                self.nw.stateUpdateHandler = { [weak self] state in self?.stateChanged(state) }
                self.nw.start(queue: self.queue)
                self.queue.asyncAfter(deadline: .now() + timeout) { [weak self] in
                    guard let self, let w = self.connectWaiter else { return }
                    self.connectWaiter = nil
                    let why = self.lastWaitingError.map { "waiting: \($0)" } ?? "no TLS handshake within \(timeout)s"
                    w(.failure(NearbyError.timeout(why)))
                    self.finish(reason: why)
                    self.nw.cancel()
                }
            }
        }
    }

    public func close() {
        queue.async { self.finish(reason: "closed by client"); self.nw?.cancel() }
    }

    private func stateChanged(_ state: NWConnection.State) {
        switch state {
        case .ready:
            isReady = true
            negotiated = NearbyCrypto.negotiated(nw)
            if let w = connectWaiter { connectWaiter = nil; w(.success(())) }
            receiveLoop()
        case .waiting(let error):
            // Transient for path changes; final for a refused handshake, which
            // Network.framework sometimes reports here rather than as .failed.
            lastWaitingError = error
            if case .tls = error, let w = connectWaiter {
                connectWaiter = nil
                w(.failure(NearbyError.handshakeFailed(String(describing: error))))
                finish(reason: "handshake failed: \(error)")
                nw.cancel()
            }
        case .failed(let error):
            if let w = connectWaiter {
                connectWaiter = nil
                w(.failure(NearbyError.handshakeFailed(String(describing: error))))
            }
            finish(reason: isReady ? "failed: \(error)" : "handshake failed: \(error)")
            nw.cancel()
        case .cancelled:
            finish(reason: "cancelled")
        default: break
        }
    }

    private func finish(reason: String) {
        guard closedReason == nil else { return }
        closedReason = reason
        let waiting = pending
        pending.removeAll()
        for (_, p) in waiting { p.resume(.failure(NearbyError.closed(reason))) }
        onClose?(reason)
    }

    private func receiveLoop() {
        nw.receive(minimumIncompleteLength: 1, maximumLength: 64 * 1024) { [weak self] data, _, isComplete, error in
            guard let self, self.closedReason == nil else { return }
            if let data, !data.isEmpty {
                for line in self.splitter.append(data) { self.handle(line: line) }
                if self.splitter.pendingBytes > NearbyWire.maxLineBytes {
                    self.finish(reason: "peer sent an oversized line"); self.nw.cancel(); return
                }
            }
            if let error { self.finish(reason: "receive error: \(error)"); self.nw.cancel(); return }
            if isComplete { self.finish(reason: "peer closed"); self.nw.cancel(); return }
            self.receiveLoop()
        }
    }

    private func handle(line: Data) {
        onLine?(.received, line)
        guard let header = try? NearbyWire.header(of: line) else {
            finish(reason: "undecodable line from Mac"); nw.cancel(); return
        }
        if header.type == "error" {
            let payload = (try? JSONDecoder().decode(NearbyWire.ErrorLine.self, from: line))?.payload
                ?? .init(code: "unknown", message: String(decoding: line, as: UTF8.self))
            let err = NearbyError.remote(code: payload.code, message: payload.message)
            if let id = header.id, let p = pending.removeValue(forKey: id) {
                p.resume(.failure(err))
            } else {
                // `id: null` (line_too_long, undecodable envelope): nothing can be
                // attributed, and the Mac closes next.
                let waiting = pending
                pending.removeAll()
                for (_, p) in waiting { p.resume(.failure(err)) }
            }
            return
        }
        guard let id = header.id, let p = pending.removeValue(forKey: id) else { return } // unsolicited: ignored
        guard header.type == p.type else {
            p.resume(.failure(NearbyError.protocolViolation("expected \(p.type) for \(id), got \(header.type)")))
            return
        }
        p.resume(.success(line))
    }

    // MARK: requests

    /// Sends one envelope and waits for the reply with the same `id` and
    /// `replyType`, or an `error` reply, or the connection closing.
    public func request<Req: Codable, Rep: Codable>(type: String, _ payload: Req, expecting replyType: String,
                                                    id: String? = nil, timeout: TimeInterval = 30) async throws -> NearbyWire.Envelope<Rep> {
        let reqID = id ?? "c-" + UUID().uuidString.lowercased()
        let line = try NearbyWire.line(id: reqID, type: type, payload)
        let reply: Data = try await withCheckedThrowingContinuation { cont in
            queue.async {
                if let why = self.closedReason { cont.resume(throwing: NearbyError.closed(why)); return }
                guard self.nw != nil else { cont.resume(throwing: NearbyError.closed("connect() was not called")); return }
                self.pending[reqID] = (replyType, { cont.resume(with: $0) })
                self.onLine?(.sent, line)
                self.nw.send(content: line, completion: .contentProcessed { [weak self] error in
                    guard let self, let error, let p = self.pending.removeValue(forKey: reqID) else { return }
                    p.resume(.failure(NearbyError.closed("send failed: \(error)")))
                })
                self.queue.asyncAfter(deadline: .now() + timeout) { [weak self] in
                    guard let self, let p = self.pending.removeValue(forKey: reqID) else { return }
                    p.resume(.failure(NearbyError.timeout("no \(replyType) for \(type) \(reqID) within \(timeout)s")))
                }
            }
        }
        return try NearbyWire.decode(reply, as: Rep.self)
    }

    /// `hello` (must be the first request). Verifies the echoed nonce; when
    /// `expectPairPsk` (bootstrap connection) the ack must carry a 32-byte key.
    @discardableResult
    public func hello(companionName: String, nonce: String = UUID().uuidString, expectPairPsk: Bool = false) async throws -> NearbyWire.HelloAck {
        let h = NearbyWire.Hello(pairId: pairId, companionName: companionName, nonce: nonce,
                                 proof: NearbyCrypto.helloProof(psk: psk, nonce: nonce))
        let ack: NearbyWire.Envelope<NearbyWire.HelloAck> = try await request(type: "hello", h, expecting: "hello_ack")
        guard ack.payload.nonce == nonce else {
            throw NearbyError.protocolViolation("hello_ack nonce \(ack.payload.nonce) != sent \(nonce)")
        }
        if expectPairPsk {
            guard let b64 = ack.payload.pairPsk, let key = Data(base64Encoded: b64), key.count == NearbyCrypto.pskLength else {
                throw NearbyError.protocolViolation("bootstrap hello_ack without a 32-byte pair_psk")
            }
        }
        return ack.payload
    }

    public func destinationQuery() async throws -> NearbyWire.Destination? {
        let r: NearbyWire.Envelope<NearbyWire.DestinationReply> =
            try await request(type: "destination_query", NearbyWire.Empty(), expecting: "destination")
        return r.payload.destination
    }

    /// `capture_submit` → `capture_received`. Retry after a disconnect with the
    /// *same* `capture_id` and payload; the Mac de-duplicates.
    public func submitCapture(_ capture: NearbyWire.CaptureSubmit, requestID: String? = nil,
                              timeout: TimeInterval = 60) async throws -> NearbyWire.CaptureReceived {
        guard NearbyWire.isValidID(capture.captureId) else { throw NearbyError.invalidInput("capture_id must be 1–128 ASCII [A-Za-z0-9_-]") }
        guard NearbyWire.isValidID(capture.destinationId) else { throw NearbyError.invalidInput("destination_id must be 1–128 ASCII [A-Za-z0-9_-]") }
        guard NearbyWire.acceptedMimeTypes.contains(capture.image.mimeType) else { throw NearbyError.invalidInput("mime_type must be image/png or image/jpeg") }
        guard capture.instructions.utf8.count <= 4096 else { throw NearbyError.invalidInput("instructions exceed 4096 bytes") }
        let r: NearbyWire.Envelope<NearbyWire.CaptureReceived> =
            try await request(type: "capture_submit", capture, expecting: "capture_received", id: requestID, timeout: timeout)
        return r.payload
    }
}
