import Network
import XCTest
@testable import NearbyClient

/// The Mac's explicit receive-cap and validation codes (proposal §4), as the
/// client must treat them: backpressure retried on the same session after
/// its acknowledgements, `too_many_sessions` retried after a reconnect,
/// image/revision/conflict refusals terminal. Against the fake Mac here; the
/// same codes come from the real listener in apps/mac
/// `NearbyReferenceClientTests`.
final class NearbyReceiveCapTests: XCTestCase {
    let salt = NearbyCrypto.data(hex: "0f0e0d0c0b0a09080706050403020100")!
    let pairId = "00112233aabbccdd"
    let psk = Data(repeating: 0x42, count: 32)
    let anchor = NearbyWire.Destination(destinationId: "anchor-7", projectId: "demo", path: "main.tex", baseRevision: 3)

    func pair() -> PairedMac {
        PairedMac(fingerprint: NearbyCrypto.fingerprint(salt: salt), macName: "Fake Mac", pairId: pairId,
                  pairPsk: psk.base64EncodedString(), companionName: "Cap iPad")
    }
    func mac() throws -> FakeMac {
        let m = try FakeMac(keys: [.init(identity: pairId, psk: psk, bootstrap: false)], destination: anchor)
        m.start()
        return m
    }
    func endpoint(_ port: UInt16) -> NWEndpoint { .hostPort(host: "127.0.0.1", port: NWEndpoint.Port(rawValue: port)!) }
    func capture(_ id: String = "cap-1", image: Data = TestImages.png1x1, mime: String = "image/png", revision: Int? = nil) -> NearbyWire.CaptureSubmit {
        .init(captureId: id, destinationId: anchor.destinationId, baseRevision: revision ?? anchor.baseRevision,
              image: .init(mimeType: mime, dataBase64: image.base64EncodedString()), instructions: "")
    }

    final class Recorder: @unchecked Sendable {
        private let lock = NSLock()
        private(set) var events: [NearbyReconnector.Event] = []
        private(set) var sleeps: [TimeInterval] = []
        var beforeSleep: (@Sendable () -> Void)?
        func event(_ e: NearbyReconnector.Event) { lock.withLock { events.append(e) } }
        func sleep(_ s: TimeInterval) async throws { beforeSleep?(); lock.withLock { sleeps.append(s) } }
        var kinds: [String] {
            events.map {
                switch $0 {
                case .attempt: return "attempt"; case .connected: return "connected"; case .failed: return "failed"
                case .disconnected: return "disconnected"; case .waitingForAcks: return "waitingForAcks"; case .gaveUp: return "gaveUp"
                }
            }
        }
    }

    func testCodeClassification() {
        for code in ["too_many_in_flight", "inbox_full"] {
            let e = NearbyError.remote(code: code, message: "")
            XCTAssertTrue(e.isRetryable, code); XCTAssertTrue(e.isBackpressure, code); XCTAssertFalse(e.isClosing, code)
            XCTAssertFalse(e.needsNewCapture, code); XCTAssertFalse(e.needsRepair, code)
        }
        let sessions = NearbyError.remote(code: "too_many_sessions", message: "")
        XCTAssertTrue(sessions.isRetryable); XCTAssertFalse(sessions.isBackpressure); XCTAssertTrue(sessions.isClosing)
        for code in ["image_too_large", "invalid_image", "revision_mismatch", "capture_id_conflict", "unsupported_image"] {
            let e = NearbyError.remote(code: code, message: "")
            XCTAssertFalse(e.isRetryable, code); XCTAssertTrue(e.needsNewCapture, code); XCTAssertFalse(e.needsRepair, code)
        }
        XCTAssertNil(NearbyWire.checkImage(TestImages.png1x1, mimeType: "image/png"))
        XCTAssertEqual(NearbyWire.checkImage(TestImages.png1x1, mimeType: "image/jpeg"), "mime_type image/jpeg but the bytes are not a JPEG")
        XCTAssertEqual(NearbyWire.checkImage(Data(), mimeType: "image/png"), "image is empty")
        XCTAssertEqual(NearbyWire.checkImage(Data(repeating: 0, count: NearbyWire.maxImageBytes + 1), mimeType: "image/png"),
                       "image is \(NearbyWire.maxImageBytes + 1) bytes; the Mac accepts at most \(NearbyWire.maxImageBytes)")
        XCTAssertNil(NearbyWire.checkImage(TestImages.brokenPNG, mimeType: "image/png"), "structure is the Mac's check")
    }

    func testBackpressureIsRetriedOnTheSameSessionAfterAcks() async throws {
        let m = try mac()
        defer { m.stop() }
        m.refuseCaptures = [("too_many_in_flight", "session has 20 MiB awaiting acknowledgement"), ("inbox_full", "listener busy")]
        let rec = Recorder()
        let r = NearbyReconnector(pair: pair(), policy: ReconnectPolicy(maxAttempts: 5, initialDelay: 0.1, jitter: 0),
                                  endpoints: { [ep = endpoint(m.port)] in ep }, onEvent: { rec.event($0) }, sleep: { try await rec.sleep($0) })
        let ack = try await r.submit(capture())
        XCTAssertEqual(ack.captureId, "cap-1")
        XCTAssertEqual(m.acceptedConnections, 1, "backpressure never reconnects")
        XCTAssertEqual(m.hellos.count, 1)
        XCTAssertEqual(m.captures.map(\.captureId), ["cap-1", "cap-1", "cap-1"], "same capture_id re-sent after each wait")
        XCTAssertEqual(m.captures[0], m.captures[2])
        XCTAssertEqual(rec.sleeps, [0.1, 0.2])
        XCTAssertEqual(rec.kinds, ["attempt", "connected", "waitingForAcks", "failed", "waitingForAcks", "failed"], "\(rec.events)")
        XCTAssertTrue(rec.events.contains { if case .waitingForAcks(0) = $0 { return true }; return false }, "the error reply itself resolved the only request")
        XCTAssertFalse(rec.events.contains { if case .disconnected = $0 { return true }; return false })
        let made = await r.attemptsMade
        XCTAssertEqual(made, 1)
        await r.shutdown()
    }

    func testBackpressureWaitsForOutstandingAcksBeforeRetrying() async throws {
        let m = try mac()
        defer { m.stop() }
        let r = NearbyReconnector(pair: pair(), policy: .immediate, endpoints: { [ep = endpoint(m.port)] in ep })
        let session = try await r.connect()
        // An earlier capture is still awaiting its (slow) acknowledgement when the Mac pushes back.
        m.replyDelay = 0.5
        let slow = Task { try await session.connection.submitCapture(self.capture("cap-slow")) }
        try await Task.sleep(nanoseconds: 50_000_000)
        m.replyDelay = 0
        m.refuseCaptures = [("inbox_full", "busy")]
        let started = Date()
        let ack = try await r.submit(capture("cap-2"))
        let elapsed = Date().timeIntervalSince(started)
        XCTAssertEqual(ack.captureId, "cap-2")
        XCTAssertGreaterThan(elapsed, 0.3, "the retry waited for cap-slow's acknowledgement (\(elapsed)s)")
        let slowAck = try await slow.value
        XCTAssertEqual(slowAck.captureId, "cap-slow")
        XCTAssertEqual(m.captures.map(\.captureId), ["cap-slow", "cap-2", "cap-2"])
        XCTAssertEqual(m.acceptedConnections, 1)
        await r.shutdown()
    }

    func testBackpressureExhaustsTheBudgetWithoutReconnecting() async throws {
        let m = try mac()
        defer { m.stop() }
        m.refuseCaptures = Array(repeating: ("too_many_in_flight", "still busy"), count: 5)
        let rec = Recorder()
        let r = NearbyReconnector(pair: pair(), policy: ReconnectPolicy(maxAttempts: 3, initialDelay: 0.1, jitter: 0),
                                  endpoints: { [ep = endpoint(m.port)] in ep }, onEvent: { rec.event($0) }, sleep: { try await rec.sleep($0) })
        do { _ = try await r.submit(capture()); XCTFail() } catch let e as NearbyError {
            guard case .attemptsExhausted(let n, let last) = e else { return XCTFail("unexpected \(e)") }
            XCTAssertEqual(n, 3)
            XCTAssertTrue(last.contains("too_many_in_flight"), last)
        }
        XCTAssertEqual(m.acceptedConnections, 1)
        XCTAssertEqual(m.captures.count, 3)
        XCTAssertEqual(rec.sleeps, [0.1, 0.2])
        await r.shutdown()
    }

    func testTooManySessionsIsRetriedAfterReconnect() async throws {
        let m = try mac()
        defer { m.stop() }
        m.refuseHellos = ("too_many_sessions", "4 sessions already open for this pair_id", 2)
        let rec = Recorder()
        let r = NearbyReconnector(pair: pair(), policy: ReconnectPolicy(maxAttempts: 4, initialDelay: 0.1, jitter: 0),
                                  endpoints: { [ep = endpoint(m.port)] in ep }, onEvent: { rec.event($0) }, sleep: { try await rec.sleep($0) })
        let ack = try await r.submit(capture())
        XCTAssertEqual(ack.captureId, "cap-1")
        XCTAssertEqual(m.acceptedConnections, 3, "two refused hellos, then accepted")
        XCTAssertEqual(m.hellos.count, 3)
        XCTAssertEqual(m.captures.count, 1)
        XCTAssertEqual(rec.sleeps, [0.1, 0.2])
        XCTAssertEqual(rec.kinds.filter { $0 == "attempt" }.count, 3)
        XCTAssertTrue(rec.events.contains { if case .failed(_, let why, _) = $0 { return why.contains("too_many_sessions") }; return false }, "\(rec.events)")
        await r.shutdown()
        // Budget spent while the Mac keeps refusing: terminal, with the code in the message.
        let m2 = try mac()
        defer { m2.stop() }
        m2.refuseHellos = ("too_many_sessions", "still full", 10)
        let r2 = NearbyReconnector(pair: pair(), policy: ReconnectPolicy(maxAttempts: 2, initialDelay: 0, jitter: 0), endpoints: { [ep = self.endpoint(m2.port)] in ep })
        do { _ = try await r2.submit(capture()); XCTFail() } catch let e as NearbyError {
            guard case .attemptsExhausted(2, let last) = e, last.contains("too_many_sessions") else { return XCTFail("unexpected \(e)") }
        }
        XCTAssertEqual(m2.acceptedConnections, 2)
    }

    func testCaptureRefusalsAreTerminalAndNeverRetried() async throws {
        let m = try mac()
        defer { m.stop() }
        let rec = Recorder()
        let r = NearbyReconnector(pair: pair(), policy: ReconnectPolicy(maxAttempts: 5, initialDelay: 0, jitter: 0),
                                  endpoints: { [ep = endpoint(m.port)] in ep }, onEvent: { rec.event($0) }, sleep: { try await rec.sleep($0) })
        for code in ["image_too_large", "invalid_image", "revision_mismatch", "capture_id_conflict"] {
            m.refuseCaptures = [(code, "refused")]
            do { _ = try await r.submit(capture("cap-\(code)")); XCTFail(code) } catch let e as NearbyError {
                XCTAssertEqual(e, .remote(code: code, message: "refused"))
                XCTAssertTrue(e.needsNewCapture)
            }
        }
        XCTAssertEqual(m.captures.count, 4, "each refusal was sent exactly once")
        XCTAssertEqual(rec.sleeps, [])
        XCTAssertEqual(m.acceptedConnections, 1, "the session stays open across refusals")
        // Local checks refuse before anything is sent.
        do { _ = try await r.submit(capture("cap-big", image: Data(repeating: 0x89, count: NearbyWire.maxImageBytes + 1))); XCTFail() } catch let e as NearbyError {
            guard case .invalidInput(let why) = e, why.hasPrefix("image is \(NearbyWire.maxImageBytes + 1) bytes") else { return XCTFail("unexpected \(e)") }
            XCTAssertTrue(e.needsNewCapture)
        }
        do { _ = try await r.submit(capture("cap-mime", mime: "image/jpeg")); XCTFail() } catch let e as NearbyError {
            XCTAssertEqual(e, .invalidInput("mime_type image/jpeg but the bytes are not a JPEG"))
        }
        do { _ = try await r.submit(capture("cap-neg", revision: -1), requireCurrentDestination: false); XCTFail() } catch let e as NearbyError {
            XCTAssertEqual(e, .invalidInput("base_revision must be non-negative"))
        }
        XCTAssertEqual(m.captures.count, 4, "nothing reached the Mac")
        // Bypassing the local check sends the bytes; a stand-in Mac cannot judge structure, the real one does (apps/mac tests).
        _ = try await r.submit(capture("cap-bypass", image: TestImages.brokenPNG), validateImage: false)
        XCTAssertEqual(m.captures.last?.captureId, "cap-bypass")
        await r.shutdown()
    }

    func testCLIReportsCapCodes() async throws {
        let m = try mac()
        defer { m.stop() }
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("nearby-caps-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }
        let store = try PairFile(url: dir.appendingPathComponent("pairs.json"))
        try store.upsert(pair())
        let png = dir.appendingPathComponent("dot.png")
        try TestImages.png1x1.write(to: png)
        func send(_ extra: [String]) async -> (Int32, [String]) {
            var out: [String] = []
            let code = await NearbyCLI.run(["send", "--image", png.path, "--host", "127.0.0.1", "--port", "\(m.port)", "--store", store.url.path,
                                            "--retry-delay", "0", "--max-delay", "0"] + extra) { out.append($0) }
            return (code, out)
        }
        m.refuseCaptures = [("inbox_full", "busy")]
        let (ok, out) = await send(["--capture-id", "cli-bp"])
        XCTAssertEqual(ok, 0, out.joined(separator: "\n"))
        XCTAssertTrue(out.contains("the Mac asked to wait: 0 requests still awaiting acknowledgement"), "\(out)")
        XCTAssertTrue(out.contains { $0.hasPrefix("attempt 1 failed: Mac replied error inbox_full") }, "\(out)")
        XCTAssertTrue(out.contains { $0.hasPrefix("capture_received cli-bp:") && !$0.contains("connection attempts") }, "same connection: \(out)")

        m.refuseHellos = ("too_many_sessions", "full", 10)
        let (full, out4) = await send(["--attempts", "2"])
        XCTAssertEqual(full, 4, out4.joined(separator: "\n"))
        XCTAssertTrue(out4.contains { $0.contains("close the companion's other connections") }, "\(out4)")
        m.refuseHellos = nil

        for (code, hint) in [("image_too_large", "shrink the image"), ("invalid_image", "could not parse the image"),
                             ("revision_mismatch", "another base_revision"), ("capture_id_conflict", "different content")] {
            m.refuseCaptures = [(code, "no")]
            let (rc, o) = await send(["--capture-id", "cli-\(code)"])
            XCTAssertEqual(rc, 2, o.joined(separator: "\n"))
            XCTAssertTrue(o.contains { $0.hasPrefix("error: Mac replied error \(code)") }, "\(o)")
            XCTAssertTrue(o.contains { $0.contains(hint) }, "\(o)")
            XCTAssertFalse(o.contains { $0.hasPrefix("attempt 2") }, "no retry for \(code): \(o)")
        }
        // A too-large file is refused before connecting (exit 65).
        let big = dir.appendingPathComponent("big.png")
        try (TestImages.png1x1 + Data(repeating: 0, count: NearbyWire.maxImageBytes)).write(to: big)
        var out65: [String] = []
        let rc65 = await NearbyCLI.run(["send", "--image", big.path, "--host", "127.0.0.1", "--port", "\(m.port)", "--store", store.url.path]) { out65.append($0) }
        XCTAssertEqual(rc65, 65, out65.joined(separator: "\n"))
        XCTAssertTrue(out65.contains { $0.contains("the Mac accepts at most \(NearbyWire.maxImageBytes)") }, "\(out65)")
    }
}
