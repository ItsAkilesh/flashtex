import Network
import XCTest
import FlashTeXProtocol
import NearbyClient
@testable import FlashTeXMac

/// The reference companion client (apps/mac/tools/nearby-client) against the
/// real Mac stack: Bonjour advertisement + TXT, TLS-PSK, bootstrap pairing,
/// long-term key hand-over, capture into the ShellModel inbox, forget.
/// The CLI commands run in-process (`NearbyCLI.run`), exactly what
/// `.build/debug/nearby-client` executes.
@MainActor
final class NearbyReferenceClientTests: XCTestCase {
    static let fixturePNG: Data = {
        // The 1×1 PNG from protocol/fixtures/capture-submission.json.
        let env = try! RuntimeV1.decodeCaptureSubmit(Data(contentsOf: NearbyListenerTests.fixtureURL))
        return Data(base64Encoded: env.payload.image.dataBase64)!
    }()

    private var tmp: URL!
    override func setUp() {
        tmp = FileManager.default.temporaryDirectory.appendingPathComponent("nearby-ref-\(UUID().uuidString)")
        try! FileManager.default.createDirectory(at: tmp, withIntermediateDirectories: true)
    }
    override func tearDown() { try? FileManager.default.removeItem(at: tmp) }

    private func cli(_ args: [String]) async -> (code: Int32, out: [String]) {
        var out: [String] = []
        let code = await NearbyCLI.run(args) { out.append($0) }
        return (code, out)
    }

    func testCLIPairsSendsRetriesAndIsForgottenThroughNearbyState() async throws {
        let store = PairStore(url: tmp.appendingPathComponent("mac-pairs.json"))
        let model = ShellModel()
        model.caretUTF16 = 6
        model.pinAnchorAtCaret()
        let anchor = try XCTUnwrap(model.nearbyDestination)
        let macName = "FlashTeX Ref \(UUID().uuidString.prefix(6))"
        let state = NearbyState(store: store, macName: macName, loopbackOnly: true)
        state.attach(sink: model, destinations: model)
        state.startAdvertising()
        try await waitUntil("advertising") { state.isAdvertising && state.port != nil }
        state.beginPairing()
        let code = try XCTUnwrap(state.pairingCode)
        try await waitUntil("restarted with bootstrap key") { state.log.filter { $0.hasPrefix("ready on port") }.count >= 2 }
        let fp = state.fingerprint
        let clientStore = tmp.appendingPathComponent("client-pairs.json").path
        let png = tmp.appendingPathComponent("dot.png")
        try Self.fixturePNG.write(to: png)

        // pair: browse by fp, read TXT salt, bootstrap TLS, hello → pair_psk.
        let pair = await cli(["pair", "--code", code, "--name", "Reference iPad", "--mac", fp, "--seconds", "10", "--store", clientStore, "-v"])
        XCTAssertEqual(pair.code, 0, pair.out.joined(separator: "\n"))
        XCTAssertEqual(pair.out.last, "paired")
        XCTAssertTrue(pair.out.contains { $0.hasPrefix("found \(macName) (fp \(fp), v=1)") }, "\(pair.out)")
        XCTAssertTrue(pair.out.contains("tls: 1.2 suite 0xa8"), "\(pair.out)")
        XCTAssertTrue(pair.out.contains { $0.contains("destination: \(anchor.destinationId) (\(anchor.projectId)/\(anchor.path) @ rev \(anchor.baseRevision))") }, "\(pair.out)")
        try await waitUntil("pair stored on the Mac") { state.pairs.count == 1 && state.pairingCode == nil }
        XCTAssertEqual(state.pairs.first?.companionName, "Reference iPad")
        let stored = try XCTUnwrap(try PairFile(url: URL(fileURLWithPath: clientStore)).pair(fingerprint: fp))
        XCTAssertEqual(stored.pairId, state.pairs.first?.pairId)
        XCTAssertEqual(stored.psk, state.pairs.first?.pskData, "client stored the long-term key the Mac minted")
        XCTAssertEqual(stored.macName, macName)
        // Mac restarted with the long-term key only.
        try await waitUntil("restarted with long-term key") { state.log.filter { $0.hasPrefix("ready on port") }.count >= 3 }

        // send: reconnect with pair_psk, capture into the ShellModel inbox.
        let send = await cli(["send", "--image", png.path, "--instructions", "transcribe the box", "--capture-id", "ref-cap-1",
                              "--mac", fp, "--seconds", "10", "--store", clientStore, "-v"])
        XCTAssertEqual(send.code, 0, send.out.joined(separator: "\n"))
        XCTAssertTrue(send.out.contains("capture_received ref-cap-1: durable=false has_proposal=false applied=false"), "\(send.out)")
        XCTAssertTrue(send.out.contains { $0.hasPrefix("<< ") && $0.contains("\"type\":\"hello_ack\"") && !$0.contains("pair_psk") },
                      "no key hand-over on a long-term connection: \(send.out)")
        try await waitUntil("inbox") { model.nearbyInbox.lastCaptureId == "ref-cap-1" }
        let received = try XCTUnwrap(model.nearbyInbox.received.first)
        XCTAssertEqual(received.destinationId, anchor.destinationId)
        XCTAssertEqual(received.baseRevision, anchor.baseRevision)
        XCTAssertEqual(received.image.mimeType, "image/png")
        XCTAssertEqual(Data(base64Encoded: received.image.dataBase64), Self.fixturePNG)
        XCTAssertEqual(received.instructions, "transcribe the box")
        XCTAssertEqual(state.lastReceivedCaptureId, "ref-cap-1")

        // Identical retry (same capture_id + payload): acknowledged again, stored once.
        let retry = await cli(["send", "--image", png.path, "--instructions", "transcribe the box", "--capture-id", "ref-cap-1",
                               "--mac", fp, "--store", clientStore])
        XCTAssertEqual(retry.code, 0, retry.out.joined(separator: "\n"))
        XCTAssertEqual(model.nearbyInbox.received.count, 1)
        // Same id, different payload: capture_id_conflict, session stays open (CLI reports it).
        let conflict = await cli(["send", "--image", png.path, "--instructions", "something else", "--capture-id", "ref-cap-1",
                                  "--mac", fp, "--store", clientStore])
        XCTAssertEqual(conflict.code, 2)
        XCTAssertTrue(conflict.out.contains { $0.contains("error capture_id_conflict") }, "\(conflict.out)")

        // status: the pairing is listed and the Mac is visible.
        let status = await cli(["status", "--seconds", "3", "--store", clientStore])
        XCTAssertEqual(status.code, 0)
        XCTAssertTrue(status.out.contains { $0.hasPrefix("\(macName) fp=\(fp) pair_id=\(stored.pairId) as \"Reference iPad\"") && $0.hasSuffix("visible now as \"\(macName)\"") }, "\(status.out)")

        // Forget on the Mac: the stored key is refused at the handshake.
        state.forget(pairId: stored.pairId)
        try await waitUntil("restarted without the key") { state.log.filter { $0.hasPrefix("ready on port") }.count >= 4 }
        let refused = await cli(["send", "--image", png.path, "--mac", fp, "--store", clientStore])
        XCTAssertEqual(refused.code, 2)
        XCTAssertTrue(refused.out.contains { $0.hasPrefix("error: TLS-PSK handshake failed") }, "\(refused.out)")
        XCTAssertEqual(model.nearbyInbox.received.count, 1)
        state.stopAdvertising()
    }

    func testCLIDirectModeAndListenerRefusalsAgainstRealListener() async throws {
        // Direct --host/--port (no Bonjour) against a bare NearbyListener with
        // a bootstrap key, then a wrong-proof hello and a pre-hello message.
        let store = PairStore(url: tmp.appendingPathComponent("mac-pairs.json"))
        let coordinator = PairingCoordinator(store: store)
        _ = coordinator.begin(code: "123456")
        let bootstrap = try XCTUnwrap(coordinator.bootstrapEntry)
        let sink = RecordingSink()
        let dest = FixedDestinations(.init(destinationId: "direct-anchor", projectId: "demo", path: "main.tex", baseRevision: 2))
        let h = ListenerHarness(psks: [bootstrap], sink: sink, destinations: dest, pairing: coordinator)
        try h.start()
        defer { h.stop() }
        let clientStore = tmp.appendingPathComponent("client-pairs.json").path

        let pair = await cli(["pair", "--code", "123456", "--name", "Direct iPad", "--host", "127.0.0.1", "--port", "\(h.port)",
                              "--salt", Pairing.hex(store.salt), "--store", clientStore, "-v"])
        XCTAssertEqual(pair.code, 0, pair.out.joined(separator: "\n"))
        XCTAssertTrue(pair.out.contains { $0.contains("pair_id \(bootstrap.identity)") })
        let record = try XCTUnwrap(store.pair(id: bootstrap.identity))
        XCTAssertEqual(record.companionName, "Direct iPad")
        let stored = try XCTUnwrap(try PairFile(url: URL(fileURLWithPath: clientStore)).pairs.first)
        XCTAssertEqual(stored.psk, record.pskData)
        XCTAssertEqual(stored.fingerprint, Pairing.fingerprint(salt: store.salt), "fp computed from --salt matches the Mac's")
        XCTAssertTrue(h.snapshot.contains(.hello(pairId: bootstrap.identity, companionName: "Direct iPad", bootstrap: true)))

        // Library level, same listener (still holding only the bootstrap key):
        // right key, wrong pair_id claim → pair_mismatch then close.
        let liar = NearbyConnection(host: "127.0.0.1", port: h.port, pairId: bootstrap.identity, psk: bootstrap.key)
        try await liar.connect()
        XCTAssertEqual(liar.negotiated?.tlsv12, true)
        XCTAssertEqual(liar.negotiated?.suite, 0x00A8)
        let lie = NearbyWire.Hello(pairId: "0000000000000000", companionName: "x", nonce: "n",
                                   proof: NearbyCrypto.helloProof(psk: bootstrap.key, nonce: "n"))
        do {
            let _: NearbyWire.Envelope<NearbyWire.HelloAck> = try await liar.request(type: "hello", lie, expecting: "hello_ack")
            XCTFail("expected pair_mismatch")
        } catch let e as NearbyError {
            XCTAssertEqual(e, .remote(code: "pair_mismatch", message: "pair_id is unknown or proof does not match its key"))
            XCTAssertTrue(e.isClosing)
        }
        // A message before hello → hello_required.
        let eager = NearbyConnection(host: "127.0.0.1", port: h.port, pairId: bootstrap.identity, psk: bootstrap.key)
        try await eager.connect()
        do {
            _ = try await eager.destinationQuery()
            XCTFail("expected hello_required")
        } catch let e as NearbyError {
            XCTAssertEqual(e, .remote(code: "hello_required", message: "first message must be hello"))
        }
        // The used code is consumed: a second bootstrap hello gets pairing_expired.
        let again = NearbyConnection(host: "127.0.0.1", port: h.port, pairId: bootstrap.identity, psk: bootstrap.key)
        try await again.connect()
        do {
            try await again.hello(companionName: "second device", expectPairPsk: true)
            XCTFail("expected pairing_expired")
        } catch let e as NearbyError {
            XCTAssertEqual(e, .remote(code: "pairing_expired", message: "pairing code is no longer valid"))
        }
        liar.close(); eager.close(); again.close()
    }

    /// Manual/live harness, skipped unless `FLASHTEX_NEARBY_SERVE_INFO=<path>` is
    /// set: advertises a real NearbyState with a fresh pairing code, writes
    /// `{code, port, fp, name}` to that path, then serves for
    /// `FLASHTEX_NEARBY_SERVE_SECONDS` (default 90) so `.build/debug/nearby-client`
    /// (or the iPad companion) can pair and send from outside the process. The
    /// Mac-side activity log is appended to `<path>.log`.
    func testServeForExternalClient() async throws {
        let env = ProcessInfo.processInfo.environment
        guard let infoPath = env["FLASHTEX_NEARBY_SERVE_INFO"], !infoPath.isEmpty else {
            throw XCTSkip("set FLASHTEX_NEARBY_SERVE_INFO=<path> to serve a live listener for an external client")
        }
        let seconds = Double(env["FLASHTEX_NEARBY_SERVE_SECONDS"] ?? "") ?? 90
        let store = PairStore(url: tmp.appendingPathComponent("mac-pairs.json"))
        let model = ShellModel()
        model.caretUTF16 = 6
        model.pinAnchorAtCaret()
        let state = NearbyState(store: store, macName: env["FLASHTEX_NEARBY_SERVE_NAME"] ?? "FlashTeX Serve", loopbackOnly: false)
        state.attach(sink: model, destinations: model)
        state.startAdvertising()
        try await waitUntil("advertising") { state.isAdvertising && state.port != nil }
        state.beginPairing()
        try await waitUntil("bootstrap key installed") { state.log.filter { $0.hasPrefix("ready on port") }.count >= 2 }
        let info: [String: Any] = ["code": state.pairingCode ?? "", "port": Int(state.port ?? 0), "fp": state.fingerprint,
                                   "name": state.macName, "salt": Pairing.hex(store.salt),
                                   "destination": model.nearbyDestination.map { ["destination_id": $0.destinationId, "base_revision": $0.baseRevision] } ?? [:]]
        try JSONSerialization.data(withJSONObject: info).write(to: URL(fileURLWithPath: infoPath))
        let logURL = URL(fileURLWithPath: infoPath + ".log")
        var written = 0
        let deadline = Date().addingTimeInterval(seconds)
        while Date() < deadline {
            let log = state.log
            if log.count > written {
                let chunk = log[written...].map { "mac: \($0)\n" }.joined()
                if let h = try? FileHandle(forWritingTo: logURL) { h.seekToEndOfFile(); h.write(Data(chunk.utf8)); try? h.close() }
                else { try? Data(chunk.utf8).write(to: logURL) }
                written = log.count
            }
            try await Task.sleep(nanoseconds: 200_000_000)
        }
        let summary = "mac: served \(Int(seconds))s; pairs=\(state.pairs.map { "\($0.companionName) \($0.pairId)" }); inbox=\(model.nearbyInbox.received.map(\.captureId))\n"
        if let h = try? FileHandle(forWritingTo: logURL) { h.seekToEndOfFile(); h.write(Data(summary.utf8)); try? h.close() }
        state.stopAdvertising()
    }

    private func waitUntil(_ what: String, timeout: TimeInterval = 15, _ cond: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if cond() { return }
            try await Task.sleep(nanoseconds: 20_000_000)
        }
        XCTFail("timed out waiting for \(what)")
    }
}
