import Network
import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

// MARK: - test doubles

final class RecordingSink: CaptureSink {
    private let lock = NSLock()
    private(set) var envelopes: [RuntimeV1.Envelope<RuntimeV1.CaptureSubmit>] = []
    var durable = false
    func submit(_ envelope: RuntimeV1.Envelope<RuntimeV1.CaptureSubmit>, reply: @escaping (Data) -> Void) {
        lock.withLock { envelopes.append(envelope) }
        let ack = NearbyV1.CaptureReceived(captureId: envelope.payload.captureId, durable: durable, hasProposal: false, applied: false)
        reply(NearbyV1.line(id: envelope.id, type: "capture_received", ack))
    }
    var count: Int { lock.withLock { envelopes.count } }
}

final class FixedDestinations: DestinationProvider {
    var destination: NearbyV1.Destination?
    init(_ d: NearbyV1.Destination?) { destination = d }
    func currentDestination(_ reply: @escaping (NearbyV1.Destination?) -> Void) { reply(destination) }
}

/// Companion-side client: NWConnection with the same TLS-PSK parameters,
/// JSON Lines framing, and a queue of received lines.
final class NearbyTestClient {
    let connection: NWConnection
    private let queue = DispatchQueue(label: "nearby.test.client")
    private var splitter = LineSplitter()
    private let lock = NSLock()
    private var lines: [Data] = []
    private var waiters: [(Int, XCTestExpectation)] = []
    let ready = XCTestExpectation(description: "client ready")
    let failed = XCTestExpectation(description: "client failed")
    let closed = XCTestExpectation(description: "client closed")
    private(set) var failure: NWError?
    // Flags for async (main-actor) tests, which must not block on XCTWaiter.
    private(set) var isReady = false
    private(set) var isFailed = false
    private(set) var isClosed = false

    init(port: UInt16, identity: String, psk: Data) {
        connection = NWConnection(host: "127.0.0.1", port: NWEndpoint.Port(rawValue: port)!,
                                  using: NearbyListener.clientParameters(identity: identity, psk: psk))
        connection.stateUpdateHandler = { [weak self] state in
            guard let self else { return }
            switch state {
            case .ready: self.isReady = true; self.ready.fulfill(); self.receiveLoop()
            case .failed(let e): self.failure = e; self.isFailed = true; self.isClosed = true; self.failed.fulfill(); self.closed.fulfill()
            case .waiting(let e): self.failure = e; self.isFailed = true; self.failed.fulfill()
            case .cancelled: self.isClosed = true; self.closed.fulfill()
            default: break
            }
        }
        connection.start(queue: queue)
    }

    private func receiveLoop() {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self] data, _, complete, error in
            guard let self else { return }
            if let data, !data.isEmpty {
                let new = self.splitter.append(data)
                self.lock.withLock {
                    self.lines.append(contentsOf: new)
                    self.waiters.removeAll { n, exp in
                        if self.lines.count >= n { exp.fulfill(); return true }
                        return false
                    }
                }
            }
            if complete || error != nil { self.isClosed = true; self.closed.fulfill(); return }
            self.receiveLoop()
        }
    }

    func send(_ data: Data) {
        connection.send(content: data, completion: .contentProcessed { _ in })
    }

    func send<P: Codable>(id: String, type: String, _ payload: P) {
        send(NearbyV1.line(id: id, type: type, payload))
    }

    /// Waits until at least `count` lines have arrived; returns them all.
    func lines(atLeast count: Int, timeout: TimeInterval = 5, file: StaticString = #filePath, line: UInt = #line) -> [Data] {
        let exp = XCTestExpectation(description: "\(count) lines")
        lock.withLock {
            if lines.count >= count { exp.fulfill() } else { waiters.append((count, exp)) }
        }
        if XCTWaiter.wait(for: [exp], timeout: timeout) != .completed {
            XCTFail("timed out waiting for \(count) lines (have \(lock.withLock { lines.count }))", file: file, line: line)
        }
        return lock.withLock { lines }
    }

    var lineCount: Int { lock.withLock { lines.count } }
    var allLines: [Data] { lock.withLock { lines } }

    func cancel() { connection.cancel() }
}

/// Listener wrapper that collects events and waits for `.ready`.
final class ListenerHarness {
    let listener: NearbyListener
    private let lock = NSLock()
    private(set) var events: [NearbyListener.Event] = []
    let ready = XCTestExpectation(description: "listener ready")
    private(set) var port: UInt16 = 0

    init(psks: [NearbyListener.PSKEntry], sink: CaptureSink?, destinations: DestinationProvider?,
         pairing: PairingConfirmer? = nil, advertise: NearbyListener.Advertisement? = nil,
         maxLineBytes: Int = NearbyV1.maxLineBytes, port: UInt16? = nil, queue: DispatchQueue? = nil) {
        let config = NearbyListener.Configuration(psks: psks, macName: "Test Mac", port: port, advertisement: advertise,
                                                  loopbackOnly: advertise == nil, maxLineBytes: maxLineBytes)
        var capture: ((NearbyListener.Event) -> Void)!
        let box = EventBox()
        capture = { box.handler?($0) }
        listener = NearbyListener(configuration: config, sink: sink, destinations: destinations, pairing: pairing,
                                  queue: queue ?? DispatchQueue(label: "nearby.test.listener"), events: capture)
        box.handler = { [weak self] e in
            guard let self else { return }
            self.lock.withLock { self.events.append(e) }
            if case .ready(let p) = e { self.port = p; self.ready.fulfill() }
        }
    }
    private final class EventBox { var handler: ((NearbyListener.Event) -> Void)? }

    func start(file: StaticString = #filePath, line: UInt = #line) throws {
        try listener.start()
        if XCTWaiter.wait(for: [ready], timeout: 5) != .completed {
            XCTFail("listener never became ready: \(snapshot)", file: file, line: line)
        }
    }
    var snapshot: [NearbyListener.Event] { lock.withLock { events } }
    func stop() { listener.stop() }
}

// MARK: - tests

final class NearbyListenerTests: XCTestCase {
    static let pskA = Data(repeating: 0xA5, count: 32)
    static let pskB = Data(repeating: 0x5A, count: 32)
    static let fixtureURL = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        .deletingLastPathComponent().deletingLastPathComponent()
        .appendingPathComponent("protocol/fixtures/capture-submission.json")

    func hello(_ client: NearbyTestClient, pairId: String, psk: Data, nonce: String = UUID().uuidString) {
        client.send(id: "h1", type: "hello", NearbyV1.Hello(pairId: pairId, companionName: "Test iPad", nonce: nonce,
                                                            proof: Pairing.helloProof(psk: psk, nonce: nonce)))
    }

    func decode<P: Codable>(_ line: Data, as: P.Type = P.self) throws -> RuntimeV1.Envelope<P> {
        try JSONDecoder().decode(RuntimeV1.Envelope<P>.self, from: line)
    }

    /// `error` envelopes may carry `"id": null` (transfer-v1) for unidentifiable requests.
    struct ErrorLine: Decodable {
        var protocolVersion: Int, id: String?, type: String, payload: NearbyV1.ErrorPayload
        enum CodingKeys: String, CodingKey { case protocolVersion = "protocol_version", id, type, payload }
    }
    func decodeError(_ line: Data) throws -> ErrorLine {
        let e = try JSONDecoder().decode(ErrorLine.self, from: line)
        XCTAssertEqual(e.type, "error")
        return e
    }

    func testHelloCaptureAndDestinationRoundTrip() throws {
        let sink = RecordingSink()
        let dest = FixedDestinations(.init(destinationId: "mac-anchor-1", projectId: "demo", path: "main.tex", baseRevision: 3))
        let h = ListenerHarness(psks: [.init(identity: "pair-a", key: Self.pskA, isBootstrap: false)], sink: sink, destinations: dest)
        try h.start()
        defer { h.stop() }

        let client = NearbyTestClient(port: h.port, identity: "pair-a", psk: Self.pskA)
        XCTAssertEqual(XCTWaiter.wait(for: [client.ready], timeout: 5), .completed, "\(client.failure.map(String.init(describing:)) ?? "no error")")
        hello(client, pairId: "pair-a", psk: Self.pskA, nonce: "n-1")
        var lines = client.lines(atLeast: 1)
        let ack: RuntimeV1.Envelope<NearbyV1.HelloAck> = try decode(lines[0])
        XCTAssertEqual(ack.type, "hello_ack")
        XCTAssertEqual(ack.id, "h1")
        XCTAssertEqual(ack.payload.macName, "Test Mac")
        XCTAssertEqual(ack.payload.nonce, "n-1")
        XCTAssertEqual(ack.payload.destination?.destinationId, "mac-anchor-1")
        XCTAssertEqual(ack.payload.destination?.baseRevision, 3)
        XCTAssertNil(ack.payload.pairPsk, "long-term connections never receive a new PSK")

        // The exact fixture line, as the companion produces it.
        var fixture = try Data(contentsOf: Self.fixtureURL)
        while fixture.last == 0x0A { fixture.removeLast() }
        fixture.append(0x0A)
        client.send(fixture)
        lines = client.lines(atLeast: 2)
        let received: RuntimeV1.Envelope<NearbyV1.CaptureReceived> = try decode(lines[1])
        XCTAssertEqual(received.type, "capture_received")
        XCTAssertEqual(received.id, "fixture-capture-request")
        XCTAssertEqual(received.payload, .init(captureId: "fixture-capture-1", durable: false, hasProposal: false, applied: false))
        XCTAssertEqual(sink.count, 1)
        XCTAssertEqual(sink.envelopes.first?.payload.destinationId, "fixture-anchor-1")

        dest.destination = nil
        client.send(id: "q1", type: "destination_query", NearbyV1.Empty())
        lines = client.lines(atLeast: 3)
        let q: RuntimeV1.Envelope<NearbyV1.DestinationReply> = try decode(lines[2])
        XCTAssertEqual(q.type, "destination")
        XCTAssertNil(q.payload.destination)
        XCTAssertTrue(String(decoding: lines[2], as: UTF8.self).contains("\"destination\":null"), "null must be explicit")

        // Unknown types get an error but keep the session open.
        client.send(id: "x1", type: "bogus", NearbyV1.Empty())
        lines = client.lines(atLeast: 4)
        let err: RuntimeV1.Envelope<NearbyV1.ErrorPayload> = try decode(lines[3])
        XCTAssertEqual(err.payload.code, "unknown_type")
        client.send(id: "q2", type: "destination_query", NearbyV1.Empty())
        XCTAssertEqual(client.lines(atLeast: 5).count, 5)

        let events = h.snapshot
        XCTAssertTrue(events.contains(.connectionOpened), "\(events)")
        XCTAssertTrue(events.contains(.hello(pairId: "pair-a", companionName: "Test iPad", bootstrap: false)))
        XCTAssertTrue(events.contains(.capture(captureId: "fixture-capture-1")))
        client.cancel()
    }

    func testWrongPSKIsRefusedBeforeAnyLineIsParsed() throws {
        let sink = RecordingSink()
        let h = ListenerHarness(psks: [.init(identity: "pair-a", key: Self.pskA, isBootstrap: false)], sink: sink, destinations: nil)
        try h.start()
        defer { h.stop() }

        // Right identity, wrong key.
        let wrongKey = NearbyTestClient(port: h.port, identity: "pair-a", psk: Self.pskB)
        XCTAssertEqual(XCTWaiter.wait(for: [wrongKey.failed], timeout: 5), .completed, "handshake should fail")
        XCTAssertEqual(XCTWaiter.wait(for: [wrongKey.ready], timeout: 0.5), .timedOut)
        // Unknown identity.
        let unknown = NearbyTestClient(port: h.port, identity: "nobody", psk: Self.pskA)
        XCTAssertEqual(XCTWaiter.wait(for: [unknown.failed], timeout: 5), .completed)
        XCTAssertEqual(XCTWaiter.wait(for: [unknown.ready], timeout: 0.5), .timedOut)

        XCTAssertEqual(sink.count, 0)
        let events = h.snapshot
        XCTAssertFalse(events.contains { if case .connectionOpened = $0 { return true }; return false },
                       "no session may exist for an unauthenticated peer: \(events)")
        XCTAssertFalse(events.contains { if case .hello = $0 { return true }; return false })
        wrongKey.cancel(); unknown.cancel()
    }

    func testOversizedLineClosesConnection() throws {
        let sink = RecordingSink()
        let h = ListenerHarness(psks: [.init(identity: "pair-a", key: Self.pskA, isBootstrap: false)], sink: sink,
                                destinations: nil, maxLineBytes: 4096)
        try h.start()
        defer { h.stop() }

        // A complete line over the limit.
        let c1 = NearbyTestClient(port: h.port, identity: "pair-a", psk: Self.pskA)
        XCTAssertEqual(XCTWaiter.wait(for: [c1.ready], timeout: 5), .completed)
        hello(c1, pairId: "pair-a", psk: Self.pskA)
        _ = c1.lines(atLeast: 1)
        var big = Data(repeating: 0x20, count: 5000); big.append(0x0A)
        c1.send(big)
        let lines = c1.lines(atLeast: 2)
        let err = try decodeError(lines[1])
        XCTAssertEqual(err.payload.code, "line_too_long")
        XCTAssertNil(err.id)
        XCTAssertEqual(XCTWaiter.wait(for: [c1.closed], timeout: 5), .completed, "connection must close after the error")

        // An unterminated line that grows past the limit.
        let c2 = NearbyTestClient(port: h.port, identity: "pair-a", psk: Self.pskA)
        XCTAssertEqual(XCTWaiter.wait(for: [c2.ready], timeout: 5), .completed)
        hello(c2, pairId: "pair-a", psk: Self.pskA)
        _ = c2.lines(atLeast: 1)
        c2.send(Data(repeating: 0x7B, count: 5000))
        let lines2 = c2.lines(atLeast: 2)
        let err2 = try decodeError(lines2[1])
        XCTAssertEqual(err2.payload.code, "line_too_long")
        XCTAssertEqual(XCTWaiter.wait(for: [c2.closed], timeout: 5), .completed)
        XCTAssertEqual(sink.count, 0)
    }

    func testMessagesBeforeHelloAndMismatchedHelloAreRefused() throws {
        let sink = RecordingSink()
        let h = ListenerHarness(psks: [.init(identity: "pair-a", key: Self.pskA, isBootstrap: false),
                                       .init(identity: "pair-b", key: Self.pskB, isBootstrap: false)],
                                sink: sink, destinations: nil)
        try h.start()
        defer { h.stop() }

        let c1 = NearbyTestClient(port: h.port, identity: "pair-a", psk: Self.pskA)
        XCTAssertEqual(XCTWaiter.wait(for: [c1.ready], timeout: 5), .completed)
        var fixture = try Data(contentsOf: Self.fixtureURL)
        if fixture.last != 0x0A { fixture.append(0x0A) }
        c1.send(fixture)
        let l1 = c1.lines(atLeast: 1)
        guard !l1.isEmpty else { return XCTFail("events: \(h.snapshot)") }
        let e1: RuntimeV1.Envelope<NearbyV1.ErrorPayload> = try decode(l1[0])
        XCTAssertEqual(e1.payload.code, "hello_required")
        XCTAssertEqual(XCTWaiter.wait(for: [c1.closed], timeout: 5), .completed)
        XCTAssertEqual(sink.count, 0)

        // Authenticated as pair-b, claiming pair-a (proof made with its own key).
        let c2 = NearbyTestClient(port: h.port, identity: "pair-b", psk: Self.pskB)
        XCTAssertEqual(XCTWaiter.wait(for: [c2.ready], timeout: 5), .completed, "\(String(describing: c2.failure))")
        hello(c2, pairId: "pair-a", psk: Self.pskB)
        let e2: RuntimeV1.Envelope<NearbyV1.ErrorPayload> = try decode(c2.lines(atLeast: 1)[0])
        XCTAssertEqual(e2.payload.code, "pair_mismatch")
        XCTAssertEqual(XCTWaiter.wait(for: [c2.closed], timeout: 5), .completed)

        // Correct pair-b hello works, proving the server picked the right PSK.
        let c3 = NearbyTestClient(port: h.port, identity: "pair-b", psk: Self.pskB)
        XCTAssertEqual(XCTWaiter.wait(for: [c3.ready], timeout: 5), .completed)
        hello(c3, pairId: "pair-b", psk: Self.pskB, nonce: "same")
        let l3 = c3.lines(atLeast: 1)
        guard !l3.isEmpty else { return XCTFail("events: \(h.snapshot)") }
        let ack: RuntimeV1.Envelope<NearbyV1.HelloAck> = try decode(l3[0])
        XCTAssertEqual(ack.type, "hello_ack")
        XCTAssertTrue(h.snapshot.contains(.hello(pairId: "pair-b", companionName: "Test iPad", bootstrap: false)))

        // Re-using a hello nonce on a new connection is refused.
        let c4 = NearbyTestClient(port: h.port, identity: "pair-b", psk: Self.pskB)
        XCTAssertEqual(XCTWaiter.wait(for: [c4.ready], timeout: 5), .completed)
        hello(c4, pairId: "pair-b", psk: Self.pskB, nonce: "same")
        let e4: RuntimeV1.Envelope<NearbyV1.ErrorPayload> = try decode(c4.lines(atLeast: 1)[0])
        XCTAssertEqual(e4.payload.code, "bad_request")
        XCTAssertEqual(XCTWaiter.wait(for: [c4.closed], timeout: 5), .completed)
        c3.cancel()
    }

    func testBootstrapPairingHandsOverLongTermPSKAndSurvivesRestart() throws {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("nearby-\(UUID().uuidString)")
        let store = PairStore(url: dir.appendingPathComponent("pairs.json"))
        XCTAssertNil(store.loadError)
        let coordinator = PairingCoordinator(store: store)
        let pending = coordinator.begin(code: "123456")
        let bootstrap = try XCTUnwrap(coordinator.bootstrapEntry)
        XCTAssertTrue(bootstrap.isBootstrap)

        let sink = RecordingSink()
        let queue = DispatchQueue(label: "nearby.test.shared")
        let h1 = ListenerHarness(psks: [bootstrap], sink: sink, destinations: nil, pairing: coordinator, queue: queue)
        try h1.start()

        // Companion side: same derivation from the typed code and the TXT salt.
        let derived = Pairing.derive(code: "123456", salt: store.salt)
        XCTAssertEqual(derived, pending.derived)
        let client = NearbyTestClient(port: h1.port, identity: derived.pairId, psk: derived.psk)
        XCTAssertEqual(XCTWaiter.wait(for: [client.ready], timeout: 5), .completed, "\(String(describing: client.failure))")
        hello(client, pairId: derived.pairId, psk: derived.psk)
        let ack: RuntimeV1.Envelope<NearbyV1.HelloAck> = try decode(client.lines(atLeast: 1)[0])
        let longTerm = try XCTUnwrap(Data(base64Encoded: try XCTUnwrap(ack.payload.pairPsk)))
        XCTAssertEqual(longTerm.count, 32)
        XCTAssertNotEqual(longTerm, derived.psk)
        let record = try XCTUnwrap(store.pair(id: derived.pairId))
        XCTAssertEqual(record.pskData, longTerm)
        XCTAssertEqual(record.companionName, "Test iPad")
        XCTAssertNil(coordinator.current, "a confirmed code is consumed")
        XCTAssertTrue(h1.snapshot.contains(.hello(pairId: derived.pairId, companionName: "Test iPad", bootstrap: true)))

        // Restart with the long-term key only (what NearbyState does), same port,
        // adopting the live session.
        let h2 = ListenerHarness(psks: [.init(identity: derived.pairId, key: longTerm, isBootstrap: false)], sink: sink,
                                 destinations: nil, pairing: coordinator, port: h1.port, queue: queue)
        h2.listener.adoptConnections(from: h1.listener)
        let stopped = XCTestExpectation(description: "old listener released its port")
        h1.listener.stop(keepConnections: true) { stopped.fulfill() }
        XCTAssertEqual(XCTWaiter.wait(for: [stopped], timeout: 5), .completed)
        try h2.start()
        XCTAssertEqual(h2.port, h1.port)
        XCTAssertEqual(h2.listener.openConnectionCount, 1)

        // The adopted session keeps working…
        client.send(id: "q", type: "destination_query", NearbyV1.Empty())
        XCTAssertEqual(client.lines(atLeast: 2).count, 2)
        // …the bootstrap key no longer authenticates…
        let stale = NearbyTestClient(port: h2.port, identity: derived.pairId, psk: derived.psk)
        XCTAssertEqual(XCTWaiter.wait(for: [stale.failed], timeout: 5), .completed)
        // …and the long-term key does, without another hand-over.
        let again = NearbyTestClient(port: h2.port, identity: derived.pairId, psk: longTerm)
        XCTAssertEqual(XCTWaiter.wait(for: [again.ready], timeout: 5), .completed, "\(String(describing: again.failure))")
        hello(again, pairId: derived.pairId, psk: longTerm)
        let ack2: RuntimeV1.Envelope<NearbyV1.HelloAck> = try decode(again.lines(atLeast: 1)[0])
        XCTAssertNil(ack2.payload.pairPsk)
        XCTAssertNotNil(store.pair(id: derived.pairId)?.lastSeenAt)

        // A second bootstrap attempt with an expired code is refused at hello.
        _ = coordinator.begin(code: "654321", lifetime: -1)
        XCTAssertNil(coordinator.bootstrapEntry)
        XCTAssertNil(coordinator.confirmPairing(pairId: Pairing.derive(code: "654321", salt: store.salt).pairId, companionName: "x"))

        client.cancel(); again.cancel(); stale.cancel()
        h2.stop()
        try? FileManager.default.removeItem(at: dir)
    }

    func testBonjourAdvertisingReachesReady() throws {
        let h = ListenerHarness(psks: [.init(identity: "pair-a", key: Self.pskA, isBootstrap: false)], sink: nil,
                                destinations: nil, advertise: .init(name: "FlashTeX Test \(UUID().uuidString.prefix(6))",
                                                                    txt: ["v": "1", "name": "Test Mac", "fp": "0123456789abcdef", "salt": "00"]))
        try h.start()
        XCTAssertGreaterThan(h.port, 0)
        h.stop()
    }

    // MARK: pure session (no network)

    func testSessionRejectsUnsupportedVersionsAndDuplicateHello() {
        let key = NearbyListener.PSKEntry(identity: "p", key: NearbyListenerTests.pskA, isBootstrap: false)
        let session = NearbySession(keys: [key], macName: "M", sink: nil, destinations: nil, pairing: nil) { _ in }
        var out: [Data] = []
        let emit: (Data) -> Void = { out.append($0) }
        XCTAssertEqual(session.handle(line: Data("not json".utf8), emit: emit), .closeAfterFlush("undecodable envelope"))
        XCTAssertTrue(String(decoding: out[0], as: UTF8.self).contains("\"id\":null"))
        let v2 = Data("{\"protocol_version\":2,\"id\":\"a\",\"type\":\"hello\",\"payload\":{}}".utf8)
        XCTAssertEqual(session.handle(line: v2, emit: emit), .closeAfterFlush("unsupported protocol_version"))
        let proof = Pairing.helloProof(psk: key.key, nonce: "n")
        let old = NearbyV1.line(id: "h", type: "hello", NearbyV1.Hello(pairId: "p", companionName: "c", protocolVersion: 0, nonce: "n", proof: proof))
        XCTAssertEqual(session.handle(line: old.dropLast(), emit: emit), .closeAfterFlush("unsupported nearby version"))
        let badProof = NearbyV1.line(id: "h", type: "hello", NearbyV1.Hello(pairId: "p", companionName: "c", nonce: "n",
                                                                            proof: Pairing.helloProof(psk: key.key, nonce: "m")))
        XCTAssertEqual(session.handle(line: badProof.dropLast(), emit: emit), .closeAfterFlush("pair mismatch"))
        XCTAssertFalse(session.helloCompleted)
        let good = NearbyV1.line(id: "h", type: "hello", NearbyV1.Hello(pairId: "p", companionName: "c", nonce: "n", proof: proof))
        XCTAssertEqual(session.handle(line: good.dropLast(), emit: emit), .keepOpen)
        XCTAssertTrue(session.helloCompleted)
        XCTAssertEqual(session.handle(line: good.dropLast(), emit: emit), .closeAfterFlush("duplicate hello"))
        XCTAssertEqual(out.count, 6)
    }

    func testSessionValidatesCaptureSubmitFields() throws {
        let sink = RecordingSink()
        let key = NearbyListener.PSKEntry(identity: "p", key: NearbyListenerTests.pskA, isBootstrap: false)
        let session = NearbySession(keys: [key], macName: "M", sink: sink, destinations: nil, pairing: nil) { _ in }
        var out: [Data] = []
        let emit: (Data) -> Void = { out.append($0) }
        let hello = NearbyV1.line(id: "h", type: "hello", NearbyV1.Hello(pairId: "p", companionName: "c", nonce: "n",
                                                                         proof: Pairing.helloProof(psk: key.key, nonce: "n")))
        _ = session.handle(line: hello.dropLast(), emit: emit)
        var env = try RuntimeV1.decodeCaptureSubmit(Data(contentsOf: Self.fixtureURL))
        env.payload.image.mimeType = "image/gif"
        _ = session.handle(line: try RuntimeV1.encodeLine(env).dropLast(), emit: emit)
        let e1: RuntimeV1.Envelope<NearbyV1.ErrorPayload> = try decode(out[1])
        XCTAssertEqual(e1.payload.code, "unsupported_image")
        env.payload.image.mimeType = "image/png"
        env.payload.captureId = "bad id!"
        _ = session.handle(line: try RuntimeV1.encodeLine(env).dropLast(), emit: emit)
        let e2: RuntimeV1.Envelope<NearbyV1.ErrorPayload> = try decode(out[2])
        XCTAssertEqual(e2.payload.code, "bad_request")
        env.payload.captureId = "ok-1"
        env.payload.instructions = String(repeating: "x", count: 4097)
        _ = session.handle(line: try RuntimeV1.encodeLine(env).dropLast(), emit: emit)
        let e3: RuntimeV1.Envelope<NearbyV1.ErrorPayload> = try decode(out[3])
        XCTAssertEqual(e3.payload.code, "bad_request")
        XCTAssertEqual(sink.count, 0)
        env.payload.instructions = "ok"
        XCTAssertEqual(session.handle(line: try RuntimeV1.encodeLine(env).dropLast(), emit: emit), .keepOpen)
        XCTAssertEqual(sink.count, 1)
        let ack: RuntimeV1.Envelope<NearbyV1.CaptureReceived> = try decode(out[4])
        XCTAssertEqual(ack.payload.captureId, "ok-1")
    }
}

// MARK: - pairing and store

final class PairingTests: XCTestCase {
    func testHKDFDerivationIsDeterministicAndPinned() throws {
        let salt = try XCTUnwrap(Pairing.data(hex: "000102030405060708090a0b0c0d0e0f"))
        let a = Pairing.derive(code: "123456", salt: salt)
        let b = Pairing.derive(code: "123456", salt: salt)
        XCTAssertEqual(a, b)
        XCTAssertEqual(a.psk.count, 32)
        XCTAssertEqual(a.pairId.count, 16)
        XCTAssertNotEqual(a, Pairing.derive(code: "123457", salt: salt))
        XCTAssertNotEqual(a, Pairing.derive(code: "123456", salt: Pairing.generateSalt()))
        // Pinned vector for the companion implementation (see docs/nearby-v1-proposal.md).
        XCTAssertEqual(a.pairId, Pairing.vectorPairID)
        XCTAssertEqual(Pairing.hex(a.psk), Pairing.vectorPSKHex)
        XCTAssertEqual(Pairing.fingerprint(salt: salt), Pairing.vectorFingerprint)
        XCTAssertEqual(Pairing.helloProof(psk: a.psk, nonce: "n-1"), Pairing.vectorHelloProof)
        XCTAssertTrue(Pairing.verifyHelloProof(Pairing.vectorHelloProof, psk: a.psk, nonce: "n-1"))
        XCTAssertFalse(Pairing.verifyHelloProof(Pairing.vectorHelloProof, psk: a.psk, nonce: "n-2"))
        XCTAssertFalse(Pairing.verifyHelloProof("not base64!", psk: a.psk, nonce: "n-1"))
        XCTAssertEqual(Pairing.generateCode().count, 6)
        XCTAssertTrue(Pairing.generateCode().allSatisfy(\.isNumber))
        XCTAssertNotEqual(Pairing.mintLongTermPSK(), Pairing.mintLongTermPSK())
    }

    func testPairStoreRoundTripAndPermissions() throws {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("nearby-store-\(UUID().uuidString)")
        let url = dir.appendingPathComponent("FlashTeX/pairs.json")
        let store = PairStore(url: url)
        XCTAssertNil(store.loadError)
        XCTAssertEqual(store.salt.count, Pairing.saltLength)
        let record = PairRecord(pairId: "abcdef0123456789", psk: Pairing.mintLongTermPSK().base64EncodedString(),
                                companionName: "iPad", createdAt: Date(timeIntervalSince1970: 1_700_000_000), lastSeenAt: nil)
        XCTAssertTrue(store.upsert(record))
        let perms = try FileManager.default.attributesOfItem(atPath: url.path)[.posixPermissions] as? Int
        XCTAssertEqual(perms, 0o600)

        let reloaded = PairStore(url: url)
        XCTAssertEqual(reloaded.salt, store.salt)
        XCTAssertEqual(reloaded.pairs, [record])
        reloaded.touch(pairId: record.pairId)
        XCTAssertNotNil(reloaded.pair(id: record.pairId)?.lastSeenAt)
        XCTAssertTrue(reloaded.remove(pairId: record.pairId))
        XCTAssertEqual(PairStore(url: url).pairs, [])

        // A corrupt file is left alone and reported.
        try Data("{}".utf8).write(to: url)
        let corrupt = PairStore(url: url)
        XCTAssertNotNil(corrupt.loadError)
        XCTAssertFalse(corrupt.upsert(record))
        XCTAssertEqual(try Data(contentsOf: url), Data("{}".utf8))
        try? FileManager.default.removeItem(at: dir)
    }
}

// MARK: - ShellModel integration

@MainActor
final class ShellModelNearbyTests: XCTestCase {
    func testInboxAcknowledgesNonDurablyAndRefusesConflicts() throws {
        let model = ShellModel()
        let env = try RuntimeV1.decodeCaptureSubmit(Data(contentsOf: NearbyListenerTests.fixtureURL))
        guard case .success(let ack) = model.receiveNearbyCapture(env.payload) else { return XCTFail() }
        XCTAssertEqual(ack, .init(captureId: "fixture-capture-1", durable: false, hasProposal: false, applied: false))
        XCTAssertEqual(model.nearbyInbox.received.count, 1)
        XCTAssertEqual(model.nearbyInbox.lastCaptureId, "fixture-capture-1")
        // Identical retry: acknowledged again, stored once.
        guard case .success = model.receiveNearbyCapture(env.payload) else { return XCTFail() }
        XCTAssertEqual(model.nearbyInbox.received.count, 1)
        var other = env.payload
        other.instructions = "different"
        guard case .failure(let err) = model.receiveNearbyCapture(other) else { return XCTFail() }
        XCTAssertEqual(err.code, "capture_id_conflict")
        XCTAssertTrue(model.proposals.isEmpty, "a received capture is not a proposal")

        // Sink wrapper produces the wire line.
        let exp = expectation(description: "reply")
        var reply = Data()
        model.submit(env) { reply = $0; exp.fulfill() }
        wait(for: [exp], timeout: 2)
        let decoded = try JSONDecoder().decode(RuntimeV1.Envelope<NearbyV1.CaptureReceived>.self, from: reply)
        XCTAssertEqual(decoded.id, env.id)
        XCTAssertFalse(decoded.payload.durable)
    }

    func testDestinationFollowsPinnedAnchor() {
        let model = ShellModel()
        XCTAssertNil(model.nearbyDestination)
        model.caretUTF16 = 6
        model.pinAnchorAtCaret()
        let d = model.nearbyDestination
        XCTAssertEqual(d?.destinationId, model.anchor?.id)
        XCTAssertEqual(d?.path, "main.tex")
        XCTAssertEqual(d?.projectId, model.result?.projectId)
        XCTAssertEqual(d?.baseRevision, model.anchor?.revision)
        let exp = expectation(description: "dest")
        var got: NearbyV1.Destination?
        model.currentDestination { got = $0; exp.fulfill() }
        wait(for: [exp], timeout: 2)
        XCTAssertEqual(got, d)
    }
}

// MARK: - NearbyState (what the window drives)

@MainActor
final class NearbyStateTests: XCTestCase {
    func testPairForgetAndRestartThroughState() async throws {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("nearby-state-\(UUID().uuidString)")
        let store = PairStore(url: dir.appendingPathComponent("pairs.json"))
        let model = ShellModel()
        let state = NearbyState(store: store, macName: "State Mac", loopbackOnly: true)
        state.attach(sink: model, destinations: model)
        XCTAssertEqual(state.txtRecord["fp"], state.fingerprint)
        XCTAssertEqual(state.txtRecord["v"], "1")

        state.startAdvertising()
        try await waitUntil("advertising") { state.isAdvertising && state.port != nil }
        let port = try XCTUnwrap(state.port)

        state.beginPairing()
        let code = try XCTUnwrap(state.pairingCode)
        XCTAssertEqual(code.count, 6)
        XCTAssertNotNil(state.codeExpiresAt)
        // Restarting for the bootstrap key keeps the port.
        try await waitUntil("restarted with bootstrap key") {
            state.log.filter { $0.hasPrefix("ready on port") }.count >= 2
        }
        XCTAssertEqual(state.port, port)

        let derived = Pairing.derive(code: code, salt: store.salt)
        let client = NearbyTestClient(port: port, identity: derived.pairId, psk: derived.psk)
        try await waitUntil("client ready (\(String(describing: client.failure)))") { client.isReady }
        let nonce = UUID().uuidString
        client.send(id: "h", type: "hello", NearbyV1.Hello(pairId: derived.pairId, companionName: "State iPad", nonce: nonce,
                                                           proof: Pairing.helloProof(psk: derived.psk, nonce: nonce)))
        try await waitUntil("hello_ack") { client.lineCount >= 1 }
        let ack = try JSONDecoder().decode(RuntimeV1.Envelope<NearbyV1.HelloAck>.self, from: client.allLines[0])
        XCTAssertEqual(ack.payload.macName, "State Mac")
        XCTAssertNotNil(ack.payload.pairPsk)
        try await waitUntil("pair stored") { state.pairs.count == 1 && state.pairingCode == nil }
        XCTAssertEqual(state.pairs.first?.companionName, "State iPad")
        try await waitUntil("connected") { state.connectedPairIds == [derived.pairId] }

        // Capture through the state → ShellModel inbox.
        var fixture = try Data(contentsOf: NearbyListenerTests.fixtureURL)
        if fixture.last != 0x0A { fixture.append(0x0A) }
        client.send(fixture)
        try await waitUntil("capture_received") { client.lineCount >= 2 }
        try await waitUntil("capture noted") { state.lastReceivedCaptureId == "fixture-capture-1" }
        XCTAssertEqual(model.nearbyInbox.lastCaptureId, "fixture-capture-1")

        // Forget: record gone, session closed, listener still up on the same port.
        state.forget(pairId: derived.pairId)
        XCTAssertEqual(state.pairs, [])
        try await waitUntil("session closed") { client.isClosed }
        try await waitUntil("still advertising") { state.isAdvertising && state.port == port && state.connectedPairIds.isEmpty }
        let longTerm = try XCTUnwrap(Data(base64Encoded: try XCTUnwrap(ack.payload.pairPsk)))
        let gone = NearbyTestClient(port: port, identity: derived.pairId, psk: longTerm)
        try await waitUntil("forgotten key refused") { gone.isFailed }
        XCTAssertFalse(gone.isReady)

        state.stopAdvertising()
        XCTAssertFalse(state.isAdvertising)
        XCTAssertNil(state.port)
        try? FileManager.default.removeItem(at: dir)
    }

    private func waitUntil(_ what: String, timeout: TimeInterval = 5, _ cond: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if cond() { return }
            try await Task.sleep(nanoseconds: 20_000_000)
        }
        XCTFail("timed out waiting for \(what)")
    }
}
