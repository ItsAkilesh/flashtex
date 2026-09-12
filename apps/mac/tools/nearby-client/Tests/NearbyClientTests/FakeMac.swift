import Foundation
import Network
import Security
@testable import NearbyClient

/// Minimal stand-in for the Mac listener: same TLS-PSK parameters, verifies
/// `hello.proof`, hands over a `pair_psk` on the bootstrap key, answers
/// `destination_query` and `capture_submit`, errors on anything else.
/// The real listener is exercised by apps/mac's NearbyReferenceClientTests.
final class FakeMac {
    struct Key { let identity: String; let psk: Data; let bootstrap: Bool }
    let keys: [Key]
    let macName: String
    var destination: NearbyWire.Destination?
    let longTermPSK = Data((0..<32).map { _ in UInt8.random(in: 0...255) })
    private(set) var captures: [NearbyWire.CaptureSubmit] = []
    private(set) var hellos: [NearbyWire.Hello] = []
    private var listener: NWListener!
    private let queue = DispatchQueue(label: "fake.mac")
    private(set) var connections: [NWConnection] = []
    private(set) var port: UInt16 = 0
    private let ready = DispatchSemaphore(value: 0)

    init(keys: [Key], macName: String = "Fake Mac", destination: NearbyWire.Destination? = nil, advertise: String? = nil, salt: Data? = nil) throws {
        self.keys = keys
        self.macName = macName
        self.destination = destination
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
        if advertise == nil { params.requiredInterfaceType = .loopback }
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

    func stop() { listener.cancel(); queue.sync { connections.forEach { $0.cancel() } } }

    private func accept(_ c: NWConnection) {
        connections.append(c)
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
                        func reply<P: Codable>(_ type: String, _ p: P) { c.send(content: try! NearbyWire.line(id: id, type: type, p), completion: .idempotent) }
                        func fail(_ code: String, _ msg: String) { reply("error", NearbyWire.ErrorPayload(code: code, message: msg)) }
                        // Like the listener's closeAfterFlush: FIN behind the error line, then cancel.
                        func closeAfterFlush() {
                            c.send(content: nil, contentContext: .finalMessage, isComplete: true, completion: .contentProcessed { _ in c.cancel() })
                        }
                        switch header.type {
                        case "hello":
                            let h = try! NearbyWire.decode(line, as: NearbyWire.Hello.self).payload
                            self.hellos.append(h)
                            guard let key = self.keys.first(where: { $0.identity == h.pairId }),
                                  NearbyCrypto.verifyHelloProof(h.proof, psk: key.psk, nonce: h.nonce) else {
                                fail("pair_mismatch", "unknown pair or bad proof"); closeAfterFlush(); return
                            }
                            helloDone = true
                            reply("hello_ack", NearbyWire.HelloAck(macName: self.macName, nonce: h.nonce, destination: self.destination,
                                                                   pairPsk: key.bootstrap ? self.longTermPSK.base64EncodedString() : nil))
                        case _ where !helloDone:
                            fail("hello_required", "first message must be hello"); closeAfterFlush(); return
                        case "destination_query":
                            reply("destination", NearbyWire.DestinationReply(destination: self.destination))
                        case "capture_submit":
                            let cap = try! NearbyWire.decode(line, as: NearbyWire.CaptureSubmit.self).payload
                            self.captures.append(cap)
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
