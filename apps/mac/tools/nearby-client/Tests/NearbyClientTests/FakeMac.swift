import Foundation
import Network
import Security
@testable import NearbyClient

enum TestImages {
    /// The 1×1 PNG from protocol/fixtures/capture-submission.json.
    static let png1x1 = Data(base64Encoded: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4//8/AAX+Av4N70a4AAAAAElFTkSuQmCC")!
    /// PNG signature followed by garbage: the Mac's structural check refuses it.
    static let brokenPNG = Data([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) + Data(repeating: 0x42, count: 64)
}

/// Minimal stand-in for the Mac listener: same TLS-PSK parameters, verifies
/// `hello.proof`, hands over a `pair_psk` on the bootstrap key, answers
/// `destination_query` and `capture_submit`, errors on anything else.
/// Loopback only, even when advertising over Bonjour (mDNS still finds it).
/// The real listener is exercised by apps/mac's NearbyReferenceClientTests.
///
/// Fault injection for the reconnect tests: `dropCapturesBeforeReply` swallows
/// that many `capture_submit`s (recorded, never acknowledged) and cuts the
/// connection instead; `replyDelay` holds every reply; `junkBytesBeforeReply`
/// prefixes the next reply with an unterminated run of bytes.
final class FakeMac {
    struct Key { let identity: String; let psk: Data; let bootstrap: Bool }
    let keys: [Key]
    let macName: String
    let longTermPSK = Data((0..<32).map { _ in UInt8.random(in: 0...255) })
    private var _destination: NearbyWire.Destination?
    private var _captures: [NearbyWire.CaptureSubmit] = []
    private var _hellos: [NearbyWire.Hello] = []
    private var _dropCapturesBeforeReply = 0
    /// Error replies to give the next capture_submits instead of an ack (session stays open).
    private var _refuseCaptures: [(code: String, message: String)] = []
    /// Refuse the next N hellos with this code and close (e.g. `too_many_sessions`).
    private var _refuseHellos: (code: String, message: String, remaining: Int)?
    private var _replyDelay: TimeInterval = 0
    private var _junkBytesBeforeReply = 0
    private var _acceptedConnections = 0
    private var listener: NWListener!
    private let queue = DispatchQueue(label: "fake.mac")
    private var connections: [NWConnection] = []
    private(set) var port: UInt16 = 0
    private let ready = DispatchSemaphore(value: 0)

    init(keys: [Key], macName: String = "Fake Mac", destination: NearbyWire.Destination? = nil, advertise: String? = nil, salt: Data? = nil) throws {
        self.keys = keys
        self.macName = macName
        self._destination = destination
        let tls = NWProtocolTLS.Options()
        let sec = tls.securityProtocolOptions
        sec_protocol_options_set_min_tls_protocol_version(sec, .TLSv12)
        sec_protocol_options_set_max_tls_protocol_version(sec, .TLSv12)
        sec_protocol_options_append_tls_ciphersuite(sec, tls_ciphersuite_t(rawValue: NearbyCrypto.cipherSuiteRawValue)!)
        sec_protocol_options_set_tls_resumption_enabled(sec, false)
        sec_protocol_options_set_tls_tickets_enabled(sec, false)
        for k in keys {
            sec_protocol_options_add_pre_shared_key(sec, NearbyCrypto.dispatchData(k.psk), NearbyCrypto.dispatchData(Data(k.identity.utf8)))
        }
        let params = NWParameters(tls: tls, tcp: NWProtocolTCP.Options())
        params.allowLocalEndpointReuse = true
        params.requiredInterfaceType = .loopback // tests never open a port beyond loopback
        listener = try NWListener(using: params, on: .any)
        if let advertise, let salt {
            listener.service = NWListener.Service(name: advertise, type: NearbyWire.serviceType, domain: nil, txtRecord: NWTXTRecord(
                ["v": "1", "name": macName, "fp": NearbyCrypto.fingerprint(salt: salt), "salt": NearbyCrypto.hex(salt)]))
        }
        listener.stateUpdateHandler = { [weak self] state in
            guard let self else { return }
            if case .ready = state { self.port = self.listener.port?.rawValue ?? 0; self.ready.signal() }
            if case .failed = state { self.ready.signal() }
        }
        listener.newConnectionHandler = { [weak self] c in self?.accept(c) }
    }

    func start() {
        listener.start(queue: queue)
        _ = ready.wait(timeout: .now() + 5)
    }

    func stop() { listener.cancel(); queue.sync { connections.forEach { $0.cancel() }; connections.removeAll() } }

    /// Cuts every live connection (the listener keeps accepting).
    func dropConnections() { queue.sync { connections.forEach { $0.cancel() }; connections.removeAll() } }

    // Thread-safe views (the handlers run on `queue`).
    var destination: NearbyWire.Destination? {
        get { queue.sync { _destination } }
        set { queue.sync { _destination = newValue } }
    }
    var captures: [NearbyWire.CaptureSubmit] { queue.sync { _captures } }
    var hellos: [NearbyWire.Hello] { queue.sync { _hellos } }
    var acceptedConnections: Int { queue.sync { _acceptedConnections } }
    var dropCapturesBeforeReply: Int {
        get { queue.sync { _dropCapturesBeforeReply } }
        set { queue.sync { _dropCapturesBeforeReply = newValue } }
    }
    var refuseCaptures: [(code: String, message: String)] {
        get { queue.sync { _refuseCaptures } }
        set { queue.sync { _refuseCaptures = newValue } }
    }
    var refuseHellos: (code: String, message: String, remaining: Int)? {
        get { queue.sync { _refuseHellos } }
        set { queue.sync { _refuseHellos = newValue } }
    }
    var replyDelay: TimeInterval {
        get { queue.sync { _replyDelay } }
        set { queue.sync { _replyDelay = newValue } }
    }
    var junkBytesBeforeReply: Int {
        get { queue.sync { _junkBytesBeforeReply } }
        set { queue.sync { _junkBytesBeforeReply = newValue } }
    }

    private func accept(_ c: NWConnection) {
        connections.append(c)
        _acceptedConnections += 1
        var splitter = LineSplitter()
        var helloDone = false
        c.stateUpdateHandler = { state in if case .failed = state { c.cancel() } }
        func receive() {
            c.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self] data, _, complete, error in
                guard let self else { return }
                if let data {
                    for line in splitter.append(data) {
                        let header = try! NearbyWire.header(of: line)
                        let id = header.id ?? "?"
                        func send(_ bytes: Data) {
                            let junk = self._junkBytesBeforeReply
                            self._junkBytesBeforeReply = 0
                            let payload = junk > 0 ? Data(repeating: 0x41, count: junk) + bytes : bytes
                            let delay = self._replyDelay
                            if delay > 0 {
                                self.queue.asyncAfter(deadline: .now() + delay) { c.send(content: payload, completion: .idempotent) }
                            } else {
                                c.send(content: payload, completion: .idempotent)
                            }
                        }
                        func reply<P: Codable>(_ type: String, _ p: P) { send(try! NearbyWire.line(id: id, type: type, p)) }
                        func fail(_ code: String, _ msg: String) { reply("error", NearbyWire.ErrorPayload(code: code, message: msg)) }
                        // Like the listener's closeAfterFlush: FIN behind the error line, then cancel.
                        func closeAfterFlush() {
                            c.send(content: nil, contentContext: .finalMessage, isComplete: true, completion: .contentProcessed { _ in c.cancel() })
                        }
                        switch header.type {
                        case "hello":
                            let h = try! NearbyWire.decode(line, as: NearbyWire.Hello.self).payload
                            self._hellos.append(h)
                            guard let key = self.keys.first(where: { $0.identity == h.pairId }),
                                  NearbyCrypto.verifyHelloProof(h.proof, psk: key.psk, nonce: h.nonce) else {
                                fail("pair_mismatch", "unknown pair or bad proof"); closeAfterFlush(); return
                            }
                            if var r = self._refuseHellos, r.remaining > 0 {
                                r.remaining -= 1
                                self._refuseHellos = r
                                fail(r.code, r.message); closeAfterFlush(); return
                            }
                            helloDone = true
                            reply("hello_ack", NearbyWire.HelloAck(macName: self.macName, nonce: h.nonce, destination: self._destination,
                                                                   pairPsk: key.bootstrap ? self.longTermPSK.base64EncodedString() : nil))
                        case _ where !helloDone:
                            fail("hello_required", "first message must be hello"); closeAfterFlush(); return
                        case "destination_query":
                            reply("destination", NearbyWire.DestinationReply(destination: self._destination))
                        case "capture_submit":
                            let cap = try! NearbyWire.decode(line, as: NearbyWire.CaptureSubmit.self).payload
                            self._captures.append(cap)
                            if !self._refuseCaptures.isEmpty {
                                let r = self._refuseCaptures.removeFirst()
                                fail(r.code, r.message)
                                continue // session stays open; keep reading
                            }
                            if self._dropCapturesBeforeReply > 0 {
                                // Delivered but never acknowledged: the client must re-send the same id.
                                self._dropCapturesBeforeReply -= 1
                                c.cancel()
                                return
                            }
                            reply("capture_received", NearbyWire.CaptureReceived(captureId: cap.captureId, durable: false, hasProposal: false, applied: false))
                        default:
                            fail("unknown_type", "unknown message type \(header.type)")
                        }
                    }
                }
                if complete || error != nil { c.cancel(); return }
                receive()
            }
        }
        c.start(queue: queue)
        receive()
    }
}
