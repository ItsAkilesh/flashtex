import Network
import XCTest
@testable import NearbyClient

/// Bounded disconnect/reconnect against the fake Mac: identical re-send of an
/// unacknowledged capture, terminal errors that are never retried, the
/// attempt/deadline budget, cancellation, and the connection's own bounds
/// (in-flight requests, inbound line size, fail-fast unreachable).
final class NearbyReconnectTests: XCTestCase {
    let salt = NearbyCrypto.data(hex: "0f0e0d0c0b0a09080706050403020100")!
    let pairId = "00112233aabbccdd"
    let psk = Data(repeating: 0x42, count: 32)
    let anchor = NearbyWire.Destination(destinationId: "anchor-7", projectId: "demo", path: "main.tex", baseRevision: 3)

    func pair() -> PairedMac {
        PairedMac(fingerprint: NearbyCrypto.fingerprint(salt: salt), macName: "Fake Mac", pairId: pairId,
                  pairPsk: psk.base64EncodedString(), companionName: "Retry iPad")
    }
    func mac(destination: NearbyWire.Destination? = nil, key: Data? = nil) throws -> FakeMac {
        let m = try FakeMac(keys: [.init(identity: pairId, psk: key ?? psk, bootstrap: false)], destination: destination ?? anchor)
        m.start()
        XCTAssertGreaterThan(m.port, 0)
        return m
    }
    func endpoint(_ port: UInt16) -> NWEndpoint { .hostPort(host: "127.0.0.1", port: NWEndpoint.Port(rawValue: port)!) }
    func capture(_ id: String = "cap-retry-1") -> NearbyWire.CaptureSubmit {
        .init(captureId: id, destinationId: anchor.destinationId, baseRevision: anchor.baseRevision,
              image: .init(mimeType: "image/png", dataBase64: TestImages.png1x1.base64EncodedString()), instructions: "retry me")
    }

    /// A loopback port nothing listens on: bound with a BSD socket to learn a
    /// free number, then closed, so a dial there is refused at once.
    func closedPort() throws -> UInt16 {
        let fd = socket(AF_INET, SOCK_STREAM, 0)
        XCTAssertGreaterThanOrEqual(fd, 0)
        defer { close(fd) }
        var addr = sockaddr_in()
        addr.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
        addr.sin_family = sa_family_t(AF_INET)
        addr.sin_port = 0
        addr.sin_addr.s_addr = inet_addr("127.0.0.1")
        var len = socklen_t(MemoryLayout<sockaddr_in>.size)
        let bound = withUnsafePointer(to: &addr) { $0.withMemoryRebound(to: sockaddr.self, capacity: 1) { Darwin.bind(fd, $0, len) } }
        XCTAssertEqual(bound, 0)
        let named = withUnsafeMutablePointer(to: &addr) { $0.withMemoryRebound(to: sockaddr.self, capacity: 1) { getsockname(fd, $0, &len) } }
        XCTAssertEqual(named, 0)
        return UInt16(bigEndian: addr.sin_port)
    }

    /// Records events and sleeps without actually waiting.
    final class Recorder: @unchecked Sendable {
        private let lock = NSLock()
        private(set) var events: [NearbyReconnector.Event] = []
        private(set) var sleeps: [TimeInterval] = []
        var beforeSleep: (@Sendable () -> Void)?
        func event(_ e: NearbyReconnector.Event) { lock.withLock { events.append(e) } }
        func sleep(_ s: TimeInterval) async throws { beforeSleep?(); lock.withLock { sleeps.append(s) } }
        var attempts: [Int] { events.compactMap { if case .attempt(let n, _) = $0 { return n }; return nil } }
        var gaveUp: [String] { events.compactMap { if case .gaveUp(let e) = $0 { return e }; return nil } }
    }

    // MARK: policy

    func testPolicyScheduleIsBoundedAndDeterministic() {
        let p = ReconnectPolicy(maxAttempts: 6, initialDelay: 0.25, maxDelay: 2, multiplier: 2, jitter: 0.2)
        let mid: (ClosedRange<Double>) -> Double = { ($0.lowerBound + $0.upperBound) / 2 }
        XCTAssertEqual(p.delay(beforeAttempt: 1, random: mid), 0)
        XCTAssertEqual(p.delay(beforeAttempt: 2, random: mid), 0.25)
        XCTAssertEqual(p.delay(beforeAttempt: 3, random: mid), 0.5)
        XCTAssertEqual(p.delay(beforeAttempt: 4, random: mid), 1)
        XCTAssertEqual(p.delay(beforeAttempt: 5, random: mid), 2)
        XCTAssertEqual(p.delay(beforeAttempt: 6, random: mid), 2, "capped at maxDelay")
        XCTAssertEqual(p.delay(beforeAttempt: 99, random: mid), 2)
        XCTAssertEqual(p.delay(beforeAttempt: 3, random: { $0.lowerBound }), 0.4, accuracy: 1e-9)
        XCTAssertEqual(p.delay(beforeAttempt: 3, random: { $0.upperBound }), 0.6, accuracy: 1e-9)
        XCTAssertEqual(ReconnectPolicy.immediate.delay(beforeAttempt: 4), 0)
        XCTAssertEqual(ReconnectPolicy(maxAttempts: 0).maxAttempts, 1, "at least the first attempt")
        XCTAssertEqual(ReconnectPolicy(jitter: 5).jitter, 1)
        XCTAssertTrue(NearbyError.unreachable("x").isRetryable)
        XCTAssertTrue(NearbyError.closed("x").isRetryable)
        XCTAssertTrue(NearbyError.timeout("x").isRetryable)
        XCTAssertFalse(NearbyError.handshakeFailed("x").isRetryable)
        XCTAssertFalse(NearbyError.remote(code: "capture_id_conflict", message: "").isRetryable)
        XCTAssertFalse(NearbyError.destinationChanged(captureDestination: "a", current: nil).isRetryable)
        XCTAssertFalse(NearbyError.attemptsExhausted(attempts: 3, last: "x").isRetryable)
        XCTAssertTrue(NearbyError.handshakeFailed("x").needsRepair)
        XCTAssertTrue(NearbyError.remote(code: "pair_mismatch", message: "").needsRepair)
        XCTAssertFalse(NearbyError.remote(code: "unknown_type", message: "").needsRepair)
    }

    // MARK: reconnect + duplicate delivery

    func testSubmitResendsSameCaptureAfterDropBeforeAck() async throws {
        let m = try mac()
        defer { m.stop() }
        m.dropCapturesBeforeReply = 1
        let rec = Recorder()
        let r = NearbyReconnector(pair: pair(), policy: ReconnectPolicy(maxAttempts: 4, initialDelay: 0.25, maxDelay: 4, jitter: 0),
                                  endpoints: { [ep = endpoint(m.port)] in ep }, onEvent: { rec.event($0) }, sleep: { try await rec.sleep($0) })
        let started = Date()
        let ack = try await r.submit(capture())
        let elapsed = Date().timeIntervalSince(started)
        XCTAssertEqual(ack.captureId, "cap-retry-1")
        // Delivered twice (once unacknowledged), byte-identical, same capture_id: the Mac de-duplicates.
        XCTAssertEqual(m.captures.count, 2)
        XCTAssertEqual(m.captures[0], m.captures[1])
        XCTAssertEqual(m.captures[0].captureId, "cap-retry-1")
        XCTAssertEqual(m.hellos.count, 2, "one hello per connection")
        XCTAssertEqual(m.acceptedConnections, 2)
        let attempts = await r.attemptsMade
        XCTAssertEqual(attempts, 2)
        let reconnects = await r.reconnects
        XCTAssertEqual(reconnects, 1)
        XCTAssertEqual(rec.sleeps, [0.25], "one backoff wait before the second attempt")
        XCTAssertEqual(rec.attempts, [1, 2])
        XCTAssertTrue(rec.events.contains { if case .disconnected = $0 { return true }; return false }, "\(rec.events)")
        XCTAssertTrue(rec.events.contains { if case .failed(1, _, 0.25) = $0 { return true }; return false }, "\(rec.events)")
        XCTAssertTrue(rec.events.contains { if case .connected("Fake Mac", 2, _) = $0 { return true }; return false }, "\(rec.events)")
        XCTAssertLessThan(elapsed, 5, "measured: drop + reconnect + re-send took \(elapsed)s")
        print("measured: drop-before-ack recovery in \(String(format: "%.3f", elapsed))s over 2 connections (sleep injected)")
        // The session stays usable afterwards, without a further connection.
        _ = try await r.submit(capture("cap-retry-2"))
        XCTAssertEqual(m.acceptedConnections, 2)
        XCTAssertEqual(m.captures.count, 3)
        await r.shutdown()
        do { _ = try await r.submit(capture("cap-after-shutdown")); XCTFail("shut down") } catch let e as NearbyError {
            XCTAssertEqual(e, .closed("reconnector shut down"))
        }
    }

    func testDropWhileIdleReconnectsOnNextSubmit() async throws {
        let m = try mac()
        defer { m.stop() }
        let rec = Recorder()
        let r = NearbyReconnector(pair: pair(), policy: .immediate, endpoints: { [ep = endpoint(m.port)] in ep }, onEvent: { rec.event($0) }, sleep: { try await rec.sleep($0) })
        let s1 = try await r.connect()
        XCTAssertTrue(s1.isOpen)
        m.dropConnections()
        // Wait for the client side to observe the close.
        for _ in 0..<200 where s1.isOpen { try await Task.sleep(nanoseconds: 10_000_000) }
        XCTAssertFalse(s1.isOpen)
        let current = await r.currentSession
        XCTAssertNil(current)
        _ = try await r.submit(capture())
        XCTAssertEqual(m.acceptedConnections, 2)
        XCTAssertEqual(m.captures.count, 1, "nothing was sent on the dead session")
        XCTAssertEqual(rec.sleeps, [], "a dead idle session is replaced without a backoff wait")
        await r.shutdown()
    }

    // MARK: terminal errors

    func testAttemptsExhaustedAfterBoundedUnreachableAttempts() async throws {
        let port = try closedPort()
        let rec = Recorder()
        let policy = ReconnectPolicy(maxAttempts: 3, initialDelay: 0.5, maxDelay: 4, jitter: 0, connectTimeout: 2)
        let r = NearbyReconnector(pair: pair(), policy: policy, endpoints: { [ep = self.endpoint(port)] in ep }, onEvent: { rec.event($0) }, sleep: { try await rec.sleep($0) })
        let started = Date()
        do { _ = try await r.submit(capture()); XCTFail("nothing listens") } catch let e as NearbyError {
            guard case .attemptsExhausted(let n, let last) = e else { return XCTFail("unexpected \(e)") }
            XCTAssertEqual(n, 3)
            XCTAssertTrue(last.hasPrefix("Mac unreachable:"), last)
        }
        let elapsed = Date().timeIntervalSince(started)
        XCTAssertEqual(rec.sleeps, [0.5, 1.0], "backoff before attempts 2 and 3, none after the last")
        XCTAssertEqual(rec.attempts, [1, 2, 3])
        XCTAssertEqual(rec.gaveUp.count, 1)
        let made = await r.attemptsMade
        XCTAssertEqual(made, 3)
        XCTAssertLessThan(elapsed, 3, "a refused port must fail fast (\(elapsed)s for 3 attempts), not wait out connectTimeout")
        print("measured: 3 refused dials classified unreachable in \(String(format: "%.3f", elapsed))s")
    }

    func testRefusedKeyIsTerminalWithoutRetry() async throws {
        let m = try mac(key: Data(repeating: 0x99, count: 32)) // the Mac forgot us: another key under our pair_id
        defer { m.stop() }
        let rec = Recorder()
        let r = NearbyReconnector(pair: pair(), policy: ReconnectPolicy(maxAttempts: 5, initialDelay: 0.1, jitter: 0),
                                  endpoints: { [ep = endpoint(m.port)] in ep }, onEvent: { rec.event($0) }, sleep: { try await rec.sleep($0) })
        let started = Date()
        do { _ = try await r.submit(capture()); XCTFail("key must be refused") } catch let e as NearbyError {
            guard case .handshakeFailed = e else { return XCTFail("unexpected \(e)") }
            XCTAssertTrue(e.needsRepair)
        }
        let elapsed = Date().timeIntervalSince(started)
        XCTAssertEqual(rec.sleeps, [], "a refused key is never retried")
        XCTAssertEqual(rec.attempts, [1])
        XCTAssertEqual(rec.gaveUp.count, 1)
        XCTAssertTrue(m.captures.isEmpty)
        XCTAssertLessThan(elapsed, 3)
        print("measured: refused key reported terminal in \(String(format: "%.3f", elapsed))s")
    }

    func testRemoteErrorIsTerminalWithoutRetry() async throws {
        let m = try mac()
        defer { m.stop() }
        let rec = Recorder()
        let r = NearbyReconnector(pair: pair(), policy: .immediate, endpoints: { [ep = endpoint(m.port)] in ep }, onEvent: { rec.event($0) }, sleep: { try await rec.sleep($0) })
        let s = try await r.connect()
        do {
            let _: NearbyWire.Envelope<NearbyWire.Empty> = try await s.connection.request(type: "bogus", NearbyWire.Empty(), expecting: "never")
            XCTFail()
        } catch let e as NearbyError {
            XCTAssertEqual(e, .remote(code: "unknown_type", message: "unknown message type bogus"))
            XCTAssertFalse(e.isRetryable)
        }
        // Through the reconnector: a bad capture_id never reaches the wire.
        var bad = capture(); bad.captureId = "no spaces allowed"
        do { _ = try await r.submit(bad); XCTFail() } catch let e as NearbyError {
            guard case .invalidInput = e else { return XCTFail("unexpected \(e)") }
        }
        XCTAssertEqual(rec.sleeps, [])
        XCTAssertEqual(m.captures.count, 0)
        await r.shutdown()
    }

    func testDestinationChangedOnRetryIsTerminal() async throws {
        let m = try mac()
        defer { m.stop() }
        m.dropCapturesBeforeReply = 1
        let rec = Recorder()
        // While the client backs off, the Mac re-pins (new destination id / revision).
        rec.beforeSleep = { m.destination = .init(destinationId: "anchor-8", projectId: "demo", path: "main.tex", baseRevision: 4) }
        let r = NearbyReconnector(pair: pair(), policy: ReconnectPolicy(maxAttempts: 4, initialDelay: 0.1, jitter: 0),
                                  endpoints: { [ep = endpoint(m.port)] in ep }, onEvent: { rec.event($0) }, sleep: { try await rec.sleep($0) })
        do { _ = try await r.submit(capture()); XCTFail("the destination moved") } catch let e as NearbyError {
            XCTAssertEqual(e, .destinationChanged(captureDestination: "anchor-7 @ rev 3", current: "anchor-8 @ rev 4"))
            XCTAssertFalse(e.isRetryable)
        }
        XCTAssertEqual(m.captures.count, 1, "the stale capture was not re-sent to the new anchor")
        XCTAssertEqual(rec.attempts, [1, 2])
        XCTAssertEqual(rec.sleeps, [0.1])
        // Unpinned entirely → also terminal, with `current: nil`.
        rec.beforeSleep = nil
        m.destination = nil
        m.dropConnections()
        do { _ = try await r.submit(capture()); XCTFail() } catch let e as NearbyError {
            XCTAssertEqual(e, .destinationChanged(captureDestination: "anchor-7 @ rev 3", current: nil))
        }
        // The caller can opt out (explicit --destination-id override).
        let ack = try await r.submit(capture("cap-override"), requireCurrentDestination: false)
        XCTAssertEqual(ack.captureId, "cap-override")
        await r.shutdown()
    }

    func testReusedSessionRechecksDestinationBeforeEachSubmit() async throws {
        let m = try mac()
        defer { m.stop() }
        let r = NearbyReconnector(pair: pair(), policy: .immediate, endpoints: { [ep = endpoint(m.port)] in ep })
        _ = try await r.submit(capture("first"))
        m.destination = .init(destinationId: "anchor-7", projectId: "demo", path: "main.tex", baseRevision: 9) // same id, newer revision
        do { _ = try await r.submit(capture("second")); XCTFail() } catch let e as NearbyError {
            XCTAssertEqual(e, .destinationChanged(captureDestination: "anchor-7 @ rev 3", current: "anchor-7 @ rev 9"))
        }
        XCTAssertEqual(m.captures.map(\.captureId), ["first"], "checked with destination_query on the reused session")
        XCTAssertEqual(m.acceptedConnections, 1)
        await r.shutdown()
    }

    func testDeadlineStopsRetriesBeforeMaxAttempts() async throws {
        let port = try closedPort()
        let rec = Recorder()
        final class FakeClock: @unchecked Sendable { let lock = NSLock(); var now: TimeInterval = 1000 }
        let fake = FakeClock()
        let clock: @Sendable () -> TimeInterval = { fake.lock.withLock { fake.now } }
        rec.beforeSleep = { fake.lock.withLock { fake.now += 30 } } // each wait "takes" 30 s
        let policy = ReconnectPolicy(maxAttempts: 10, initialDelay: 1, jitter: 0, overallDeadline: 45, connectTimeout: 2)
        let r = NearbyReconnector(pair: pair(), policy: policy, endpoints: { [ep = self.endpoint(port)] in ep },
                                  onEvent: { rec.event($0) }, sleep: { try await rec.sleep($0) }, clock: clock)
        do { _ = try await r.submit(capture()); XCTFail() } catch let e as NearbyError {
            guard case .attemptsExhausted(let n, let last) = e else { return XCTFail("unexpected \(e)") }
            XCTAssertEqual(n, 3, "t=0 fail, wait 1 → t=30 fail, wait 2 → t=60 fail: past the 45 s deadline, no fourth wait")
            XCTAssertTrue(last.contains("deadline 45.0s reached"), last)
        }
        XCTAssertEqual(rec.sleeps, [1, 2])
    }

    func testCancellationSurfacesAsCancelled() async throws {
        let port = try closedPort()
        let policy = ReconnectPolicy(maxAttempts: 5, initialDelay: 10, jitter: 0, connectTimeout: 2) // real 10 s waits
        let r = NearbyReconnector(pair: pair(), policy: policy, endpoints: { [ep = self.endpoint(port)] in ep })
        let task = Task { try await r.submit(self.capture()) }
        try await Task.sleep(nanoseconds: 300_000_000) // into the first backoff wait
        let started = Date()
        task.cancel()
        do { _ = try await task.value; XCTFail() } catch let e as NearbyError {
            XCTAssertEqual(e, .cancelled)
        }
        XCTAssertLessThan(Date().timeIntervalSince(started), 2, "cancel interrupts the wait")
    }

    // MARK: connection bounds

    func testConnectionRefusedIsUnreachableFast() async throws {
        let port = try closedPort()
        let c = NearbyConnection(host: "127.0.0.1", port: port, pairId: pairId, psk: psk)
        let started = Date()
        do { try await c.connect(timeout: 10); XCTFail() } catch let e as NearbyError {
            guard case .unreachable(let why) = e else { return XCTFail("unexpected \(e)") }
            XCTAssertTrue(why.contains("refused") || why.contains("61"), why)
            XCTAssertTrue(e.isRetryable)
        }
        XCTAssertLessThan(Date().timeIntervalSince(started), 2)
        XCTAssertFalse(c.isOpen)
        XCTAssertTrue(c.closeReason?.hasPrefix("unreachable:") == true, c.closeReason ?? "nil")
        do { _ = try await c.destinationQuery(); XCTFail() } catch let e as NearbyError {
            guard case .closed = e else { return XCTFail("unexpected \(e)") }
        }
    }

    func testInFlightBoundRefusesTheNinthRequest() async throws {
        let m = try mac()
        defer { m.stop() }
        m.replyDelay = 0.4
        let c = NearbyConnection(host: "127.0.0.1", port: m.port, pairId: pairId, psk: psk)
        try await c.connect()
        try await c.hello(companionName: "bound")
        XCTAssertEqual(c.maxInFlightRequests, 8)
        let group = DispatchGroup()
        var results = [Result<NearbyWire.Destination?, Error>]()
        let lock = NSLock()
        for _ in 0..<8 {
            group.enter()
            Task { defer { group.leave() }
                let r: Result<NearbyWire.Destination?, Error>
                do { r = .success(try await c.destinationQuery()) } catch { r = .failure(error) }
                lock.withLock { results.append(r) }
            }
        }
        try await Task.sleep(nanoseconds: 100_000_000) // the eight are on the wire
        do { _ = try await c.destinationQuery(); XCTFail("ninth must be refused") } catch let e as NearbyError {
            guard case .overloaded(let why) = e else { return XCTFail("unexpected \(e)") }
            XCTAssertTrue(why.contains("8 requests already await a reply (limit 8)"), why)
            XCTAssertFalse(e.isRetryable)
        }
        XCTAssertEqual(group.wait(timeout: .now() + 5), .success)
        XCTAssertEqual(results.count, 8)
        for r in results { if case .failure(let e) = r { XCTFail("\(e)") } }
        // Capacity is back once replies arrived.
        _ = try await c.destinationQuery()
        XCTAssertTrue(c.isOpen)
        c.close()
    }

    func testOversizedInboundLineClosesTheConnection() async throws {
        let m = try mac()
        defer { m.stop() }
        let c = NearbyConnection(host: "127.0.0.1", port: m.port, pairId: pairId, psk: psk)
        c.maxInboundLineBytes = 4096
        try await c.connect()
        try await c.hello(companionName: "bound")
        m.junkBytesBeforeReply = 8192 // an unterminated 8 KiB run ahead of the next reply
        do { _ = try await c.destinationQuery(); XCTFail() } catch let e as NearbyError {
            guard case .closed(let why) = e, why.hasPrefix("peer sent an oversized") else { return XCTFail("unexpected \(e)") }
        }
        XCTAssertFalse(c.isOpen)
        // A complete oversized line is refused too.
        let c2 = NearbyConnection(host: "127.0.0.1", port: m.port, pairId: pairId, psk: psk)
        c2.maxInboundLineBytes = 64
        try await c2.connect()
        do { try await c2.hello(companionName: "bound"); XCTFail("hello_ack is longer than 64 bytes") } catch let e as NearbyError {
            guard case .closed(let why) = e, why.hasPrefix("peer sent an oversized line") else { return XCTFail("unexpected \(e)") }
        }
        XCTAssertFalse(c2.isOpen)
    }

    func testRequestTimeoutIsBoundedAndRetryable() async throws {
        let m = try mac()
        defer { m.stop() }
        let c = NearbyConnection(host: "127.0.0.1", port: m.port, pairId: pairId, psk: psk)
        try await c.connect()
        try await c.hello(companionName: "bound")
        m.replyDelay = 2
        let started = Date()
        do { _ = try await c.destinationQuery(timeout: 0.2); XCTFail() } catch let e as NearbyError {
            guard case .timeout = e else { return XCTFail("unexpected \(e)") }
            XCTAssertTrue(e.isRetryable)
        }
        XCTAssertLessThan(Date().timeIntervalSince(started), 1.5)
        XCTAssertTrue(c.isOpen, "a timed-out request does not close the connection; the caller decides")
        c.close()
    }

    // MARK: CLI

    func testCLISendRetriesAndReportsTerminalExitCodes() async throws {
        let m = try mac()
        defer { m.stop() }
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("nearby-retry-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }
        let store = try PairFile(url: dir.appendingPathComponent("pairs.json"))
        try store.upsert(pair())
        let png = dir.appendingPathComponent("dot.png")
        try Data(base64Encoded: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4//8/AAX+Av4N70a4AAAAAElFTkSuQmCC")!.write(to: png)
        func send(_ extra: [String]) async -> (Int32, [String]) {
            var out: [String] = []
            let code = await NearbyCLI.run(["send", "--image", png.path, "--host", "127.0.0.1", "--store", store.url.path] + extra) { out.append($0) }
            return (code, out)
        }

        // Dropped before the ack: re-sent with the same id, exit 0.
        m.dropCapturesBeforeReply = 1
        let (ok, out) = await send(["--port", "\(m.port)", "--capture-id", "cli-retry-1", "--retry-delay", "0.05", "--max-delay", "0.05"])
        XCTAssertEqual(ok, 0, out.joined(separator: "\n"))
        XCTAssertTrue(out.contains { $0.hasPrefix("attempt 1 failed: connection closed:") && $0.contains("retrying in 0.0") }, "\(out)")
        XCTAssertTrue(out.contains("attempt 2 of 5"), "\(out)")
        XCTAssertTrue(out.contains { $0.hasPrefix("capture_received cli-retry-1:") && $0.hasSuffix("after 2 connection attempts (same capture_id re-sent)") }, "\(out)")
        XCTAssertEqual(m.captures.map(\.captureId), ["cli-retry-1", "cli-retry-1"])

        // Nothing listening: bounded attempts, exit 4.
        let port = try closedPort()
        let (gone, out4) = await send(["--port", "\(port)", "--attempts", "2", "--retry-delay", "0", "--timeout", "2"])
        XCTAssertEqual(gone, 4, out4.joined(separator: "\n"))
        XCTAssertTrue(out4.contains { $0.hasPrefix("error: gave up after 2 attempts; last failure: Mac unreachable:") }, "\(out4)")
        XCTAssertTrue(out4.contains { $0.contains("(exit 4)") })

        // The Mac forgot us: exit 3, no retry.
        let forgot = try mac(key: Data(repeating: 0x77, count: 32))
        defer { forgot.stop() }
        let (refused, out3) = await send(["--port", "\(forgot.port)", "--attempts", "5", "--retry-delay", "0"])
        XCTAssertEqual(refused, 3, out3.joined(separator: "\n"))
        XCTAssertTrue(out3.contains { $0.hasPrefix("error: TLS-PSK handshake failed") })
        XCTAssertFalse(out3.contains { $0.hasPrefix("attempt 2") }, "no retry after a refused key: \(out3)")
        XCTAssertTrue(out3.contains { $0.contains("run `nearby-client pair` again (exit 3)") })

        // The Mac unpinned between hello_ack and the (retried) submit: exit 5.
        m.destination = anchor
        m.dropCapturesBeforeReply = 1
        let unpin = DispatchWorkItem { m.destination = nil }
        DispatchQueue.global().asyncAfter(deadline: .now() + 0.15, execute: unpin)
        let (moved, out5) = await send(["--port", "\(m.port)", "--capture-id", "cli-moved", "--retry-delay", "0.4", "--max-delay", "0.4"])
        XCTAssertEqual(moved, 5, out5.joined(separator: "\n"))
        XCTAssertTrue(out5.contains { $0.hasPrefix("error: destination changed: capture targets anchor-7 @ rev 3 but the Mac now reports nothing pinned") }, "\(out5)")
        XCTAssertTrue(out5.contains { $0.contains("(exit 5)") })
        XCTAssertEqual(m.captures.filter { $0.captureId == "cli-moved" }.count, 1, "not re-sent against a vanished anchor")

        // Nothing pinned at all and no override: refused before any capture is built (exit 2).
        let (nopin, out2) = await send(["--port", "\(m.port)"])
        XCTAssertEqual(nopin, 2, out2.joined(separator: "\n"))
        XCTAssertTrue(out2.contains { $0.contains("no pinned insertion point") }, "\(out2)")
        // An explicit override skips the current-destination check.
        let (forced, outF) = await send(["--port", "\(m.port)", "--destination-id", "manual-1", "--base-revision", "1", "--capture-id", "cli-forced"])
        XCTAssertEqual(forced, 0, outF.joined(separator: "\n"))
        XCTAssertEqual(m.captures.last?.destinationId, "manual-1")
    }
}
