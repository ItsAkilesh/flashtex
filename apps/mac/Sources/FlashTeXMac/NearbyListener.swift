import Foundation
import Network
import Security
import FlashTeXProtocol

/// TLS-PSK listener for paired companions (proposal nearby-v1).
///
/// Transport: TCP, TLS 1.2 only, single cipher suite
/// `TLS_PSK_WITH_AES_128_GCM_SHA256` (0x00A8), one PSK per pairing keyed by its
/// `pair_id` as the TLS PSK identity. A peer without a matching PSK fails the
/// handshake, so no application byte is ever parsed from an unpaired device.
/// Bonjour (`_flashtex._tcp`) is advertised only for discovery.
///
/// Everything below runs on `queue`; callbacks into the sink/destination
/// provider are theirs to dispatch, and their replies are re-queued here.
final class NearbyListener {
    struct PSKEntry: Equatable {
        let identity: String
        let key: Data
        /// Code-derived key that only authenticates the pairing connection.
        let isBootstrap: Bool
    }

    struct Advertisement: Equatable {
        var name: String
        var txt: [String: String]
    }

    struct Configuration {
        var psks: [PSKEntry]
        var macName: String
        var port: UInt16? = nil
        var advertisement: Advertisement? = nil
        var loopbackOnly = false
        var maxLineBytes = NearbyV1.maxLineBytes
    }

    enum Event: Equatable {
        case ready(port: UInt16)
        case failed(String)
        case stopped
        case connectionOpened
        case hello(pairId: String, companionName: String, bootstrap: Bool)
        case capture(captureId: String)
        case connectionClosed(identity: String?, reason: String)
    }

    static let cipherSuite = tls_ciphersuite_t(rawValue: UInt16(TLS_PSK_WITH_AES_128_GCM_SHA256))!
    static let cipherSuiteName = "TLS_PSK_WITH_AES_128_GCM_SHA256"

    let configuration: Configuration
    let queue: DispatchQueue
    private weak var sink: CaptureSink?
    private weak var destinations: DestinationProvider?
    private weak var pairing: PairingConfirmer?
    private let events: (Event) -> Void
    private var listener: NWListener?
    private static let queueKey = DispatchSpecificKey<Bool>()
    private var connections: [ObjectIdentifier: Connection] = [:]
    private(set) var port: UInt16?
    private(set) var advertisedName: String?
    /// Recent `pair_id:nonce` values (bounded); a repeat is refused.
    private var recentNonces: [String] = []
    static let rememberedNonces = 256

    init(configuration: Configuration, sink: CaptureSink?, destinations: DestinationProvider?,
         pairing: PairingConfirmer?, queue: DispatchQueue = DispatchQueue(label: "flashtex.nearby"),
         events: @escaping (Event) -> Void) {
        self.configuration = configuration
        self.queue = queue
        queue.setSpecific(key: Self.queueKey, value: true)
        self.sink = sink
        self.destinations = destinations
        self.pairing = pairing
        self.events = events
    }

    // MARK: TLS parameters (shared with the companion-side client)

    static func tlsOptions(psks: [(identity: String, key: Data)]) -> NWProtocolTLS.Options {
        let tls = NWProtocolTLS.Options()
        let sec = tls.securityProtocolOptions
        sec_protocol_options_set_min_tls_protocol_version(sec, .TLSv12)
        sec_protocol_options_set_max_tls_protocol_version(sec, .TLSv12)
        sec_protocol_options_append_tls_ciphersuite(sec, cipherSuite)
        // No resumption: every connection must prove the PSK again, so a
        // forgotten or expired key cannot ride a cached session.
        sec_protocol_options_set_tls_resumption_enabled(sec, false)
        sec_protocol_options_set_tls_tickets_enabled(sec, false)
        for entry in psks {
            sec_protocol_options_add_pre_shared_key(sec, dispatchData(entry.key), dispatchData(Data(entry.identity.utf8)))
        }
        return tls
    }

    static func parameters(psks: [(identity: String, key: Data)], loopbackOnly: Bool) -> NWParameters {
        let tcp = NWProtocolTCP.Options()
        tcp.enableKeepalive = true
        tcp.noDelay = true
        let params = NWParameters(tls: tlsOptions(psks: psks), tcp: tcp)
        params.allowLocalEndpointReuse = true
        params.includePeerToPeer = false
        if loopbackOnly { params.requiredInterfaceType = .loopback }
        return params
    }

    /// Parameters a companion uses to connect with one pairing.
    static func clientParameters(identity: String, psk: Data) -> NWParameters {
        parameters(psks: [(identity, psk)], loopbackOnly: false)
    }

    static func dispatchData(_ data: Data) -> __DispatchData {
        data.withUnsafeBytes { DispatchData(bytes: $0) } as __DispatchData
    }

    /// Negotiated TLS version and cipher suite of an established connection.
    static func negotiated(_ connection: NWConnection) -> (version: tls_protocol_version_t, suite: tls_ciphersuite_t)? {
        guard let meta = connection.metadata(definition: NWProtocolTLS.definition) as? NWProtocolTLS.Metadata else { return nil }
        let sec = meta.securityProtocolMetadata
        return (sec_protocol_metadata_get_negotiated_tls_protocol_version(sec),
                sec_protocol_metadata_get_negotiated_tls_ciphersuite(sec))
    }

    // MARK: lifecycle

    func start() throws {
        let params = Self.parameters(psks: configuration.psks.map { ($0.identity, $0.key) },
                                     loopbackOnly: configuration.loopbackOnly)
        let nwPort: NWEndpoint.Port = configuration.port.flatMap { NWEndpoint.Port(rawValue: $0) } ?? .any
        let listener = try NWListener(using: params, on: nwPort)
        if let ad = configuration.advertisement {
            listener.service = NWListener.Service(name: ad.name, type: NearbyV1.serviceType, domain: nil,
                                                  txtRecord: NWTXTRecord(ad.txt))
            listener.serviceRegistrationUpdateHandler = { [weak self] change in
                guard let self else { return }
                if case .add(let endpoint) = change, case .service(let name, _, _, _) = endpoint {
                    self.advertisedName = name
                }
            }
        }
        listener.stateUpdateHandler = { [weak self] state in
            guard let self else { return }
            switch state {
            case .ready:
                let p = self.listener?.port?.rawValue ?? 0
                self.port = p
                self.events(.ready(port: p))
            case .failed(let error):
                self.events(.failed(String(describing: error)))
            case .waiting(let error):
                self.events(.failed("waiting: \(error)"))
            case .cancelled:
                self.events(.stopped)
            default: break
            }
        }
        listener.newConnectionHandler = { [weak self] nw in
            guard let self else { nw.cancel(); return }
            let c = Connection(nw, owner: self)
            self.connections[ObjectIdentifier(c)] = c
            c.start()
        }
        self.listener = listener
        listener.start(queue: queue)
    }

    /// Stops accepting and advertising. `keepConnections` leaves authenticated
    /// sessions open (used when the PSK table changes after a pairing).
    /// `completion` runs on `queue` once the socket is released, so a
    /// replacement can bind the same port without racing the old one.
    func stop(keepConnections: Bool = false, completion: (() -> Void)? = nil) {
        queue.async {
            if let l = self.listener {
                // Independent of `self`'s lifetime: the owner may drop this
                // instance as soon as it has asked for the stop.
                let events = self.events
                l.stateUpdateHandler = { state in
                    if case .cancelled = state { events(.stopped); completion?() }
                }
                l.cancel()
                self.listener = nil
            } else {
                completion?()
            }
            if !keepConnections {
                for c in self.connections.values { c.close(reason: "listener stopped") }
            }
        }
    }

    /// Runs `work` on `queue`, inline when already there (keeps reply order).
    fileprivate func onQueue(_ work: @escaping () -> Void) {
        if DispatchQueue.getSpecific(key: Self.queueKey) == true { work() } else { queue.async(execute: work) }
    }

    /// Call from outside `queue` only.
    var openConnectionCount: Int { queue.sync { connections.count } }

    /// Moves live sessions to another listener instance (same queue), so a PSK
    /// table change does not drop a companion that just paired. Call from
    /// outside `queue` only.
    func adoptConnections(from other: NearbyListener) {
        precondition(other.queue === queue)
        let keep = Set(configuration.psks.map(\.identity))
        queue.sync {
            for (k, c) in other.connections {
                c.owner = self
                connections[k] = c
                // A forgotten pairing loses its live session too; a session that
                // never said hello cannot be re-keyed and is dropped.
                if let id = c.identity {
                    if !keep.contains(id) { c.close(reason: "pairing forgotten") }
                } else {
                    c.close(reason: "key table changed before hello")
                }
            }
            other.connections.removeAll()
            recentNonces = other.recentNonces
        }
    }

    fileprivate func forget(_ c: Connection) { connections.removeValue(forKey: ObjectIdentifier(c)) }

    /// On `queue`. Nonces survive a restart only through `adoptConnections`.
    fileprivate func acceptNonce(_ key: String) -> Bool {
        guard !recentNonces.contains(key) else { return false }
        recentNonces.append(key)
        if recentNonces.count > Self.rememberedNonces { recentNonces.removeFirst(recentNonces.count - Self.rememberedNonces) }
        return true
    }
    fileprivate func emitEvent(_ e: Event) { events(e) }

    // MARK: one connection

    fileprivate final class Connection {
        let nw: NWConnection
        weak var owner: NearbyListener?
        private var splitter = LineSplitter()
        private var session: NearbySession?
        private var closing = false
        private(set) var identity: String?
        private let maxLineBytes: Int
        private let queue: DispatchQueue

        init(_ nw: NWConnection, owner: NearbyListener) {
            self.nw = nw
            self.owner = owner
            self.maxLineBytes = owner.configuration.maxLineBytes
            self.queue = owner.queue
        }

        func start() {
            nw.stateUpdateHandler = { [weak self] state in
                guard let self else { return }
                switch state {
                case .ready: self.authenticated()
                case .failed(let error): self.finish(reason: "failed before ready: \(error)")
                case .cancelled: self.finish(reason: "cancelled")
                default: break
                }
            }
            nw.start(queue: queue)
        }

        /// Called only after TLS completed with one of the table keys. The
        /// stack does not say which, so the session is named by `hello`'s proof.
        private func authenticated() {
            guard let owner else { close(reason: "listener gone"); return }
            guard let n = NearbyListener.negotiated(nw), n.version == .TLSv12, n.suite == NearbyListener.cipherSuite else {
                close(reason: "unexpected TLS parameters")
                return
            }
            session = NearbySession(keys: owner.configuration.psks, macName: owner.configuration.macName,
                                    sink: owner.sink, destinations: owner.destinations, pairing: owner.pairing,
                                    acceptNonce: { [weak self] key in self?.owner?.acceptNonce(key) ?? false }) { [weak self] e in
                guard let self else { return }
                switch e {
                case .hello(let p, let n, let b):
                    self.identity = p
                    self.owner?.emitEvent(.hello(pairId: p, companionName: n, bootstrap: b))
                case .capture(let id): self.owner?.emitEvent(.capture(captureId: id))
                case .violation: break
                }
            }
            owner.emitEvent(.connectionOpened)
            receiveLoop()
        }

        private func receiveLoop() {
            nw.receive(minimumIncompleteLength: 1, maximumLength: 64 * 1024) { [weak self] data, _, isComplete, error in
                guard let self, !self.closing else { return }
                if let data, !data.isEmpty { self.consume(data) }
                if let error { self.finish(reason: "receive error: \(error)"); return }
                if isComplete { self.close(reason: "peer closed"); return }
                if !self.closing { self.receiveLoop() }
            }
        }

        private func consume(_ data: Data) {
            guard let session else { return }
            let lines = splitter.append(data)
            for line in lines {
                if line.count + 1 > maxLineBytes {
                    send(NearbyV1.errorLine(id: nil, code: "line_too_long", message: "line exceeds \(maxLineBytes) bytes"))
                    closeAfterFlush(reason: "line too long")
                    return
                }
                let disposition = session.handle(line: line) { [weak self] reply in
                    guard let self, let owner = self.owner else { return }
                    owner.onQueue { self.send(reply) }
                }
                if case .closeAfterFlush(let why) = disposition {
                    closeAfterFlush(reason: why)
                    return
                }
            }
            if splitter.pendingBytes >= maxLineBytes {
                send(NearbyV1.errorLine(id: nil, code: "line_too_long", message: "unterminated line exceeds \(maxLineBytes) bytes"))
                closeAfterFlush(reason: "unterminated line too long")
            }
        }

        private func send(_ data: Data) {
            guard !closing else { return }
            nw.send(content: data, completion: .contentProcessed { _ in })
        }

        /// Queues a FIN behind everything already sent, then cancels.
        private func closeAfterFlush(reason: String) {
            guard !closing else { return }
            closing = true
            nw.send(content: nil, contentContext: .finalMessage, isComplete: true,
                    completion: .contentProcessed { [weak self] _ in self?.nw.cancel() })
            reportClosed(reason)
        }

        func close(reason: String) {
            guard !closing else { return }
            closing = true
            nw.cancel()
            reportClosed(reason)
        }

        /// Terminal state from the stack: the table entry is released only here,
        /// so the NWConnection outlives any FIN still being flushed.
        private func finish(reason: String) {
            if !closing {
                closing = true
                reportClosed(reason)
            }
            owner?.forget(self)
        }

        private func reportClosed(_ reason: String) {
            owner?.emitEvent(.connectionClosed(identity: identity, reason: reason))
        }
    }
}
