import Network
import XCTest
@testable import NearbyClient

/// Duplicate delivery, revocation and reconnect as captured client-side
/// transcripts (`Tests/Fixtures/*.jsonl`): every wire line the reference
/// client sends or receives, plus its reconnector events, in order. A
/// companion that copies `Sources/NearbyClient` must produce the same shape;
/// the listener-side counterpart is apps/mac's `NearbyTranscriptAcceptanceTests`.
///
/// Provenance: the fixtures were recorded by these very tests against the
/// loopback `FakeMac` (`NEARBY_CLIENT_RECORD_FIXTURES=1 swift test --filter
/// NearbyTranscriptFixtureTests`), not from an iPad simulator — the companion
/// app cannot authenticate yet (plain TCP, no v1 hello), so a simulator run can
/// only reproduce a handshake refusal. Re-record after a deliberate wire change.
final class NearbyTranscriptFixtureTests: XCTestCase {
    let pairId = "00112233aabbccdd"
    let psk = Data(repeating: 0x42, count: 32)
    let anchor = NearbyWire.Destination(destinationId: "anchor-7", projectId: "demo", path: "main.tex", baseRevision: 3)
    static let fixtureNames = ["duplicate-delivery.jsonl", "revocation-key-removed.jsonl", "revocation-pair-mismatch.jsonl", "reconnect-idle-drop.jsonl"]

    func pair() -> PairedMac {
        PairedMac(fingerprint: NearbyCrypto.fingerprint(salt: NearbyCrypto.data(hex: "0f0e0d0c0b0a09080706050403020100")!),
                  macName: "Fake Mac", pairId: pairId, pairPsk: psk.base64EncodedString(), companionName: "Transcript iPad")
    }
    func mac() throws -> FakeMac {
        let m = try FakeMac(keys: [.init(identity: pairId, psk: psk, bootstrap: false)], destination: anchor)
        m.start()
        XCTAssertGreaterThan(m.port, 0)
        return m
    }
    func endpoint(_ port: UInt16) -> NWEndpoint { .hostPort(host: "127.0.0.1", port: NWEndpoint.Port(rawValue: port)!) }
    func capture(_ id: String) -> NearbyWire.CaptureSubmit {
        .init(captureId: id, destinationId: anchor.destinationId, baseRevision: anchor.baseRevision,
              image: .init(mimeType: "image/png", dataBase64: TestImages.png1x1.base64EncodedString()), instructions: "from the transcript test")
    }

    /// Deterministic schedule: no jitter, sleeps recorded instead of waited.
    final class Sleeper: @unchecked Sendable {
        private let lock = NSLock()
        private(set) var sleeps: [TimeInterval] = []
        var beforeSleep: (@Sendable () -> Void)?
        func sleep(_ s: TimeInterval) async throws { beforeSleep?(); lock.withLock { sleeps.append(s) } }
    }
    let policy = ReconnectPolicy(maxAttempts: 4, initialDelay: 0.25, maxDelay: 4, jitter: 0, connectTimeout: 5, requestTimeout: 10)

    func reconnector(_ port: @escaping @Sendable () -> UInt16, _ rec: TranscriptRecorder, _ sleeper: Sleeper) -> NearbyReconnector {
        NearbyReconnector(pair: pair(), policy: policy, endpoints: { [self] in self.endpoint(port()) },
                          onEvent: { rec.event($0) }, onLine: { rec.line($0, $1) }, sleep: { try await sleeper.sleep($0) })
    }

    func assertMatches(_ rec: TranscriptRecorder, _ name: String, scenario: String, file: StaticString = #filePath, line: UInt = #line) throws {
        if let mismatch = try rec.check(against: name, scenario: scenario) { XCTFail(mismatch, file: file, line: line) }
        if TranscriptRecorder.recording { print("recorded \(name) (\(rec.lines.count) lines)") }
    }

    /// The fixture files themselves: a header with truthful provenance, then
    /// well-formed lines with normalised volatile fields. Runs even where the
    /// network tests cannot.
    func testFixturesAreWellFormedAndNormalised() throws {
        for name in Self.fixtureNames {
            let f = try TranscriptRecorder.Fixture.load(name)
            XCTAssertTrue((f.header["provenance"] as? String ?? "").contains("FakeMac"), "\(name): provenance names the FakeMac")
            XCTAssertTrue((f.header["provenance"] as? String ?? "").contains("not an iPad simulator run"), "\(name): no simulator claim")
            XCTAssertNotNil(f.header["scenario"] as? String)
            XCTAssertGreaterThan(f.lines.count, 3, name)
            var conn = 0
            for (i, l) in f.lines.enumerated() {
                let o = try XCTUnwrap(try JSONSerialization.jsonObject(with: Data(l.utf8)) as? [String: Any], "\(name) line \(i + 2)")
                let c = try XCTUnwrap(o["conn"] as? Int)
                XCTAssertTrue(c == conn || c == conn + 1, "\(name) line \(i + 2): connections are numbered in order")
                conn = c
                if let line = o["line"] as? [String: Any] {
                    XCTAssertTrue(["sent", "received"].contains(o["dir"] as? String ?? ""))
                    XCTAssertEqual(line["protocol_version"] as? Int, NearbyWire.envelopeVersion)
                    XCTAssertTrue((line["id"] as? String ?? "").hasPrefix("<id-"), "\(name) line \(i + 2): envelope id normalised")
                    let payload = line["payload"] as? [String: Any] ?? [:]
                    for key in ["nonce", "proof", "pair_psk"] where payload[key] != nil { XCTAssertEqual(payload[key] as? String, "<\(key)>") }
                    XCTAssertFalse(l.contains(psk.base64EncodedString()), "no key material in a fixture")
                } else {
                    XCTAssertNotNil(o["event"] as? String, "\(name) line \(i + 2)")
                }
            }
            // The first sent line on every connection is `hello`.
            let firstSent = Dictionary(grouping: f.lines.compactMap { l -> (Int, [String: Any])? in
                guard let o = try? JSONSerialization.jsonObject(with: Data(l.utf8)) as? [String: Any], o["dir"] as? String == "sent",
                      let c = o["conn"] as? Int, let line = o["line"] as? [String: Any] else { return nil }
                return (c, line)
            }, by: \.0).mapValues { $0.first!.1 }
            for (c, line) in firstSent { XCTAssertEqual(line["type"] as? String, "hello", "\(name) connection \(c)") }
        }
    }

    // MARK: duplicate delivery

    /// The Mac swallows an acknowledged-nothing `capture_submit` and cuts the
    /// connection; the client reconnects after one backoff and re-sends the
    /// identical payload under the same `capture_id` (new envelope id), which
    /// the Mac acknowledges. Both deliveries are in the transcript.
    func testDuplicateDeliveryTranscript() async throws {
        let m = try mac()
        defer { m.stop() }
        m.dropCapturesBeforeReply = 1
        let rec = TranscriptRecorder(), sleeper = Sleeper()
        let r = reconnector({ [port = m.port] in port }, rec, sleeper)
        let ack = try await r.submit(capture("cap-dup-1"))
        XCTAssertEqual(ack.captureId, "cap-dup-1")
        XCTAssertEqual(m.captures.count, 2, "delivered twice")
        XCTAssertEqual(m.captures[0], m.captures[1], "byte-identical payload, same capture_id")
        XCTAssertEqual(m.acceptedConnections, 2)
        XCTAssertEqual(sleeper.sleeps, [0.25])
        await r.shutdown()

        let sentCaptures = rec.lines.filter { $0.contains("\"dir\":\"sent\"") && $0.contains("\"type\":\"capture_submit\"") }
        XCTAssertEqual(sentCaptures.count, 2)
        XCTAssertTrue(sentCaptures[0].contains("\"conn\":1") && sentCaptures[1].contains("\"conn\":2"))
        XCTAssertEqual(rec.lines.filter { $0.contains("\"type\":\"capture_received\"") }.count, 1, "acknowledged once, on the second connection")
        try assertMatches(rec, "duplicate-delivery.jsonl",
                          scenario: "capture_submit swallowed and connection cut before the ack; reconnect after 0.25s backoff; identical re-send acknowledged")
    }

    // MARK: revocation

    /// The Mac forgets the pairing between an unacknowledged delivery and the
    /// reconnect: its listener comes back on the same port without our key
    /// (what `NearbyState.forget` does). The reconnect is refused at the TLS
    /// handshake; the client reports `handshakeFailed` (`needsRepair`), makes
    /// no further attempt, and the held capture is never re-sent.
    func testRevokedKeyIsRefusedAtHandshakeAndNeverRedelivers() async throws {
        let first = try mac()
        first.dropCapturesBeforeReply = 1
        let box = MacBox(first)
        defer { box.current.stop() }
        let rec = TranscriptRecorder(), sleeper = Sleeper()
        sleeper.beforeSleep = {
            // Revocation during the backoff: same port, key table without our pair_id.
            box.current = try! FakeMac.restart(box.current, keys: [.init(identity: "someone-else", psk: Data(repeating: 0x77, count: 32), bootstrap: false)])
        }
        let r = reconnector({ box.current.port }, rec, sleeper)
        let started = Date()
        do { _ = try await r.submit(capture("cap-revoked-1")); XCTFail("a revoked pairing must not deliver") } catch let e as NearbyError {
            guard case .handshakeFailed = e else { return XCTFail("unexpected \(e)") }
            XCTAssertTrue(e.needsRepair); XCTAssertFalse(e.isRetryable)
        }
        let elapsed = Date().timeIntervalSince(started)
        XCTAssertEqual(first.captures.map(\.captureId), ["cap-revoked-1"], "the first Mac saw the one unacknowledged delivery")
        XCTAssertTrue(box.current.captures.isEmpty, "the Mac that revoked us never receives the capture")
        XCTAssertTrue(box.current.hellos.isEmpty, "no line got past TLS")
        XCTAssertEqual(sleeper.sleeps, [0.25], "one backoff before the refused reconnect, none after")
        let attempts = await r.attemptsMade
        XCTAssertEqual(attempts, 2)
        let live = await r.currentSession
        XCTAssertNil(live)
        print("measured: revoked key (same-port restart) reported terminal in \(String(format: "%.3f", elapsed))s, 1 refused reconnect")
        await r.shutdown()

        XCTAssertFalse(rec.lines.contains { $0.contains("\"conn\":2") && $0.contains("\"dir\":") }, "no wire line on the refused connection: \(rec.lines)")
        XCTAssertTrue(rec.lines.last?.contains("\"error\":\"handshake_failed\",\"event\":\"gave_up\"") == true, "\(rec.lines.last ?? "")")
        try assertMatches(rec, "revocation-key-removed.jsonl",
                          scenario: "capture_submit swallowed; Mac restarts on the same port without the pairing; reconnect refused at TLS; handshakeFailed terminal, capture never re-sent")
    }

    /// Same revocation, answered at the application layer: TLS still accepts
    /// the key but `hello` is refused with `pair_mismatch` and the Mac closes.
    /// The client reports `remote(pair_mismatch)` (`needsRepair`), does not
    /// retry, and never re-sends the held capture.
    func testPairMismatchOnReconnectIsTerminalAndNeverRedelivers() async throws {
        let m = try mac()
        defer { m.stop() }
        m.dropCapturesBeforeReply = 1
        let rec = TranscriptRecorder(), sleeper = Sleeper()
        sleeper.beforeSleep = { m.refuseHellos = ("pair_mismatch", "pairing forgotten on the Mac", 1) }
        let r = reconnector({ [port = m.port] in port }, rec, sleeper)
        do { _ = try await r.submit(capture("cap-revoked-2")); XCTFail("a revoked pairing must not deliver") } catch let e as NearbyError {
            XCTAssertEqual(e, .remote(code: "pair_mismatch", message: "pairing forgotten on the Mac"))
            XCTAssertTrue(e.needsRepair); XCTAssertTrue(e.isClosing); XCTAssertFalse(e.isRetryable)
        }
        XCTAssertEqual(m.captures.map(\.captureId), ["cap-revoked-2"], "one unacknowledged delivery, no re-send")
        XCTAssertEqual(m.hellos.count, 2, "the refused hello reached the Mac")
        XCTAssertEqual(sleeper.sleeps, [0.25])
        let attempts = await r.attemptsMade
        XCTAssertEqual(attempts, 2)
        await r.shutdown()

        XCTAssertTrue(rec.lines.contains { $0.contains("\"conn\":2") && $0.contains("\"code\":\"pair_mismatch\"") }, "\(rec.lines)")
        XCTAssertEqual(rec.lines.filter { $0.contains("\"type\":\"capture_submit\"") }.count, 1)
        try assertMatches(rec, "revocation-pair-mismatch.jsonl",
                          scenario: "capture_submit swallowed; reconnect hello answered error pair_mismatch and closed; remote(pair_mismatch) terminal, capture never re-sent")
    }

    // MARK: reconnect

    /// A reused session pays a `destination_query` before each capture; after
    /// the Mac drops it while idle, the next submit dials again without a
    /// backoff wait and the fresh `hello_ack` supplies the destination.
    func testReconnectAfterIdleDropTranscript() async throws {
        let m = try mac()
        defer { m.stop() }
        let rec = TranscriptRecorder(), sleeper = Sleeper()
        let r = reconnector({ [port = m.port] in port }, rec, sleeper)
        let s1 = try await r.connect()
        _ = try await r.submit(capture("cap-reconnect-1"))
        m.dropConnections()
        for _ in 0..<500 where s1.isOpen { try await Task.sleep(nanoseconds: 10_000_000) }
        XCTAssertFalse(s1.isOpen)
        _ = try await r.submit(capture("cap-reconnect-2"))
        XCTAssertEqual(m.acceptedConnections, 2)
        XCTAssertEqual(m.captures.map(\.captureId), ["cap-reconnect-1", "cap-reconnect-2"])
        XCTAssertEqual(sleeper.sleeps, [], "a dead idle session is replaced without a backoff wait")
        let reconnects = await r.reconnects
        XCTAssertEqual(reconnects, 1)
        await r.shutdown()

        XCTAssertEqual(rec.lines.filter { $0.contains("\"type\":\"destination_query\"") }.count, 1, "reused session: one query; fresh session: none")
        XCTAssertTrue(rec.lines.contains { $0.contains("\"event\":\"disconnected\"") && $0.contains("\"conn\":1") }, "\(rec.lines)")
        try assertMatches(rec, "reconnect-idle-drop.jsonl",
                          scenario: "connect; submit on the reused session (destination_query first); Mac drops the idle session; next submit reconnects without backoff")
    }
}

/// Lets a `@Sendable` closure swap the FakeMac a reconnector dials.
final class MacBox: @unchecked Sendable {
    private let lock = NSLock()
    private var mac: FakeMac
    init(_ mac: FakeMac) { self.mac = mac }
    var current: FakeMac {
        get { lock.withLock { mac } }
        set { lock.withLock { mac = newValue } }
    }
}
