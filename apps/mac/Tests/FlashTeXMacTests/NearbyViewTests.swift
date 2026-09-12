import AppKit
import Foundation
import Network
import SwiftUI
import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// `PairingFlowController` against a real `NearbyState` (loopback TLS-PSK):
/// what the Nearby Companion window shows for each transport event, durable
/// recovery across a simulated relaunch, stale reconnects, and cancel/resume.
/// Owner: mac-pairing-ui.
@MainActor
final class NearbyViewControllerTests: XCTestCase {
    private var dir: URL!
    private var announced: [String] = []

    override func setUp() {
        super.setUp()
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("nearby-view-\(UUID().uuidString)")
        announced = []
    }

    override func tearDown() {
        try? FileManager.default.removeItem(at: dir)
        super.tearDown()
    }

    private func makeState(name: String = "Flow Mac") -> (NearbyState, ShellModel) {
        let store = PairStore(url: dir.appendingPathComponent("pairs.json"))
        let model = ShellModel()
        let state = NearbyState(store: store, macName: name, loopbackOnly: true)
        state.attach(sink: model, destinations: model)
        return (state, model)
    }

    private func makeController(_ state: NearbyState) -> PairingFlowController {
        PairingFlowController(nearby: state, journal: PairingJournal(url: dir.appendingPathComponent("pairing-session.json")),
                              announcer: { [weak self] in self?.announced.append($0) })
    }

    private func waitUntil(_ what: String, timeout: TimeInterval = 6, file: StaticString = #filePath, line: UInt = #line,
                           _ cond: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if cond() { return }
            try await Task.sleep(nanoseconds: 20_000_000)
        }
        XCTFail("timed out waiting for \(what)", file: file, line: line)
    }

    /// Opens a bootstrap connection with `code` and completes hello; returns
    /// the client and the long-term key from `hello_ack`.
    private func pair(code: String, port: UInt16, salt: Data, companion: String) async throws -> (NearbyTestClient, Data?) {
        let derived = Pairing.derive(code: code, salt: salt)
        let client = NearbyTestClient(port: port, identity: derived.pairId, psk: derived.psk)
        try await waitUntil("client ready (\(String(describing: client.failure)))") { client.isReady }
        let nonce = UUID().uuidString
        client.send(id: "h", type: "hello", NearbyV1.Hello(pairId: derived.pairId, companionName: companion, nonce: nonce,
                                                           proof: Pairing.helloProof(psk: derived.psk, nonce: nonce)))
        try await waitUntil("hello_ack") { client.lineCount >= 1 }
        let ack = try JSONDecoder().decode(RuntimeV1.Envelope<NearbyV1.HelloAck>.self, from: client.allLines[0])
        return (client, ack.payload.pairPsk.flatMap { Data(base64Encoded: $0) })
    }

    // MARK: live flow

    func testShowCodePairsAndJournalsThroughTheController() async throws {
        let (state, model) = makeState()
        let c = makeController(state)
        XCTAssertEqual(c.phase, .off)
        XCTAssertEqual(c.phase.title, "Off")

        c.showCode()
        guard case .codeShown(let a) = c.phase else { return XCTFail("\(c.phase)") }
        XCTAssertEqual(a.code, state.pairingCode)
        XCTAssertEqual(a.generation, 1)
        XCTAssertEqual(c.journal.pending, a, "the attempt is durable as soon as it is shown")
        XCTAssertEqual(c.journal.generation, 1)
        XCTAssertEqual(announced.last, "Pairing code \(Pairing.spokenCode(a.code)), valid for 120 seconds.")
        try await waitUntil("advertising with the bootstrap key") { state.isAdvertising && state.port != nil }
        XCTAssertTrue(c.machine.isAdvertising)
        XCTAssertFalse(c.phase.canShowCode(), "one code at a time")
        XCTAssertTrue(c.phase.canCancel())

        let (client, longTerm) = try await pair(code: a.code, port: state.port!, salt: state.store.salt, companion: "Flow iPad")
        XCTAssertNotNil(longTerm)
        try await waitUntil("paired") { if case .paired = c.phase { return true }; return false }
        XCTAssertEqual(c.phase, .paired(.init(pairId: a.pairId, companionName: "Flow iPad", generation: 1)))
        XCTAssertNil(c.journal.pending, "journal cleared once the pairing is stored")
        XCTAssertEqual(state.pairs.map(\.companionName), ["Flow iPad"])
        XCTAssertTrue(announced.contains("Paired with Flow iPad."), "\(announced)")
        XCTAssertTrue(c.staleInputs.isEmpty, "\(c.staleInputs)")
        // The transport's own code state is consistent with ours.
        XCTAssertNil(state.pairingCode)
        try await waitUntil("connected") { state.connectedPairIds == [a.pairId] }
        XCTAssertEqual(PairingAccessibility.deviceRow(state.pairs[0], connected: true).value.hasPrefix("Connected."), true)

        // A capture is announced with its id; the inbox row is not durable.
        var fixture = try Data(contentsOf: NearbyListenerTests.fixtureURL)
        if fixture.last != 0x0A { fixture.append(0x0A) }
        client.send(fixture)
        try await waitUntil("capture") { state.lastReceivedCaptureId == "fixture-capture-1" }
        XCTAssertTrue(announced.contains("Received capture fixture-capture-1."), "\(announced)")
        XCTAssertEqual(model.nearbyInbox.received.map(\.captureId), ["fixture-capture-1"])

        c.dismiss()
        XCTAssertEqual(c.phase, .advertising)
        // Forget through the controller ends with the device gone and the state advertising.
        c.forget(pairId: a.pairId)
        XCTAssertEqual(state.pairs, [])
        XCTAssertEqual(c.phase, .advertising)
        try await waitUntil("session closed") { client.isClosed }
        state.stopAdvertising()
        XCTAssertEqual(c.phase, .off)
    }

    func testCancelWithdrawsTheBootstrapKeyAndClearsTheJournal() async throws {
        let (state, _) = makeState()
        let c = makeController(state)
        c.showCode()
        guard case .codeShown(let a) = c.phase else { return XCTFail("\(c.phase)") }
        try await waitUntil("advertising") { state.isAdvertising && state.port != nil }
        let port = state.port!

        c.cancel()
        XCTAssertEqual(c.phase, .advertising, "cancel returns to advertising; the listener stays up")
        XCTAssertNil(state.pairingCode)
        XCTAssertNil(state.coordinator.current)
        XCTAssertNil(c.journal.pending)
        XCTAssertEqual(announced.last, "Pairing cancelled.")
        XCTAssertTrue(c.staleInputs.isEmpty, "our own withdrawal is not mistaken for expiry: \(c.staleInputs)")
        try await waitUntil("restarted without the key") { state.log.filter { $0.hasPrefix("ready on port") }.count >= 2 }
        XCTAssertEqual(state.port, port)
        let derived = Pairing.derive(code: a.code, salt: state.store.salt)
        let late = NearbyTestClient(port: port, identity: derived.pairId, psk: derived.psk)
        try await waitUntil("cancelled code refused") { late.isFailed }
        XCTAssertFalse(late.isReady)
        XCTAssertEqual(c.phase, .advertising, "a stale bootstrap attempt changes nothing")
        state.stopAdvertising()
    }

    // MARK: durable recovery

    func testRelaunchRestoresThePendingCodeAndResumeServesTheSameCode() async throws {
        // Launch 1: show a code, then "quit" (drop the state and controller).
        var code = ""
        var generation = 0
        do {
            let (state, _) = makeState()
            let c = makeController(state)
            c.showCode()
            guard case .codeShown(let a) = c.phase else { return XCTFail("\(c.phase)") }
            code = a.code
            generation = a.generation
            try await waitUntil("advertising") { state.port != nil }
            c.setAdvertising(false)
            XCTAssertEqual(c.phase, .interrupted(a, .transportStopped, detail: "advertising was turned off"))
            XCTAssertFalse(state.isAdvertising)
            XCTAssertEqual(c.journal.pending, a, "still pending: the user has not cancelled")
        }

        // Launch 2: same store and journal, fresh transport.
        let (state, _) = makeState()
        XCTAssertEqual(state.pairs, [])
        let c = makeController(state)
        guard case .interrupted(let a, .relaunch, let detail) = c.phase else { return XCTFail("\(c.phase)") }
        XCTAssertEqual(a.code, code)
        XCTAssertEqual(a.generation, generation)
        XCTAssertEqual(detail, "FlashTeX was quit while the code was valid")
        XCTAssertEqual(c.phase.title, "Pairing interrupted")
        XCTAssertTrue(c.phase.canResume())
        XCTAssertTrue(c.phase.canCancel())
        XCTAssertFalse(c.phase.canShowCode())
        XCTAssertEqual(announced.last, "A pairing from a previous launch is waiting. Resume it or cancel.")
        XCTAssertNil(state.pairingCode, "the transport has not been asked for anything yet")

        c.resume()
        XCTAssertEqual(c.phase, .codeShown(a))
        XCTAssertEqual(state.coordinator.current?.code, code, "resumed through the coordinator with the same code")
        XCTAssertNil(state.pairingCode, "NearbyState only mints fresh codes; see handoff for the requested API")
        try await waitUntil("advertising after resume") { state.isAdvertising && state.port != nil }
        let (client, longTerm) = try await pair(code: code, port: state.port!, salt: state.store.salt, companion: "Resumed iPad")
        XCTAssertNotNil(longTerm, "the resumed code confirms a pairing")
        try await waitUntil("paired") { if case .paired = c.phase { return true }; return false }
        XCTAssertEqual(state.pairs.map(\.companionName), ["Resumed iPad"])
        XCTAssertNil(c.journal.pending)
        XCTAssertNil(state.coordinator.current, "bootstrap key dropped after confirmation")
        _ = client
        state.stopAdvertising()
    }

    func testRelaunchWithExpiredCodeIsDismissedAndNewCodeUsesNewGeneration() async throws {
        let journal = PairingJournal(url: dir.appendingPathComponent("pairing-session.json"))
        let old = PairingFlow.Attempt(generation: journal.nextGeneration(), code: "111111", pairId: "stale",
                                      startedAt: Date().addingTimeInterval(-300), expiresAt: Date().addingTimeInterval(-180))
        journal.setPending(old)
        let (state, _) = makeState()
        let c = makeController(state)
        guard case .interrupted(let a, .relaunch, _) = c.phase, a.code == old.code, a.generation == 1 else { return XCTFail("\(c.phase)") }
        XCTAssertFalse(c.phase.canResume())
        XCTAssertTrue(c.phase.canDismiss())
        XCTAssertTrue(c.phase.canShowCode())
        c.dismiss()
        XCTAssertEqual(c.phase, .off)
        XCTAssertNil(c.journal.pending)
        c.showCode()
        guard case .codeShown(let b) = c.phase else { return XCTFail("\(c.phase)") }
        XCTAssertEqual(b.generation, 2)
        XCTAssertTrue(c.apply(.confirmed(pairId: "stale", companionName: "ghost", generation: 1)).stale,
                      "a reconnect from the pre-relaunch attempt cannot pair")
        XCTAssertEqual(c.phase, .codeShown(b))
        c.cancel()
        state.stopAdvertising()
    }

    func testResumedCodeExpiresIntoAnErrorStateAndDropsTheKey() async throws {
        let journal = PairingJournal(url: dir.appendingPathComponent("pairing-session.json"))
        let short = PairingFlow.Attempt(generation: journal.nextGeneration(), code: "222222", pairId: "short",
                                        startedAt: Date(), expiresAt: Date().addingTimeInterval(1.2))
        journal.setPending(short)
        let (state, _) = makeState()
        let c = makeController(state)
        XCTAssertTrue(c.phase.canResume())
        c.resume()
        guard case .codeShown(let shown) = c.phase, shown.code == short.code else { return XCTFail("\(c.phase)") }
        XCTAssertEqual(state.coordinator.current?.code, "222222")
        try await waitUntil("expired", timeout: 5) { if case .failed = c.phase { return true }; return false }
        XCTAssertEqual(c.phase, .failed(reason: "The pairing code expired before a companion paired.", generation: 1))
        XCTAssertNil(c.journal.pending)
        XCTAssertNil(state.coordinator.bootstrapEntry, "an expired bootstrap key is no longer offered")
        XCTAssertTrue(c.phase.canDismiss())
        c.dismiss()
        XCTAssertEqual(c.phase, .advertising)
        state.stopAdvertising()
    }

    // MARK: stale reconnects and replacement

    func testReplacedCodeMakesTheOldSessionStaleEndToEnd() async throws {
        let (state, _) = makeState()
        let c = makeController(state)
        c.showCode()
        guard case .codeShown(let first) = c.phase else { return XCTFail("\(c.phase)") }
        try await waitUntil("advertising") { state.port != nil }
        c.cancel()
        c.showCode()
        guard case .codeShown(let second) = c.phase else { return XCTFail("\(c.phase)") }
        XCTAssertEqual(second.generation, 2)
        XCTAssertNotEqual(first.pairId, second.pairId)
        try await waitUntil("second key served") { state.coordinator.current?.code == second.code && state.isAdvertising }
        try await waitUntil("listener restarted for the second code") {
            // Restarts coalesce; what matters is a ready after the second code was issued.
            let issued = state.log.lastIndex { $0.hasPrefix("pairing code issued") } ?? -1
            let ready = state.log.lastIndex { $0.hasPrefix("ready on port") } ?? -1
            return ready > issued
        }

        // The old session reports late through every path the machine accepts.
        XCTAssertTrue(c.apply(.confirmed(pairId: first.pairId, companionName: "Old", generation: first.generation)).stale)
        c.observe(.hello(pairId: first.pairId, companionName: "Old", bootstrap: true))
        XCTAssertEqual(c.staleInputs.count, 2, "\(c.staleInputs)")
        XCTAssertTrue(c.apply(.peerGone(pairId: first.pairId, reason: "old session gone", generation: 2)).ignored, "a close for a session we are not verifying is a no-op")
        XCTAssertEqual(c.phase, .codeShown(second))
        XCTAssertEqual(state.pairs, [], "nothing was stored for the stale attempt")

        // The old bootstrap key is refused by TLS; the new one pairs.
        let oldDerived = Pairing.derive(code: first.code, salt: state.store.salt)
        let old = NearbyTestClient(port: state.port!, identity: oldDerived.pairId, psk: oldDerived.psk)
        try await waitUntil("old code refused") { old.isFailed }
        XCTAssertEqual(c.phase, .codeShown(second))
        let (_, key) = try await pair(code: second.code, port: state.port!, salt: state.store.salt, companion: "New iPad")
        XCTAssertNotNil(key)
        try await waitUntil("paired") { if case .paired = c.phase { return true }; return false }
        XCTAssertEqual(state.pairs.map(\.pairId), [second.pairId])
        state.stopAdvertising()
    }

    func testListenerEventsDriveVerifyingPeerGoneAndReceiving() async throws {
        let (state, _) = makeState()
        let c = makeController(state)
        c.showCode()
        guard case .codeShown(let a) = c.phase else { return XCTFail("\(c.phase)") }
        c.observe(.connectionOpened)
        XCTAssertEqual(c.phase, .verifying(a))
        XCTAssertEqual(c.phase.title, "Verifying companion")
        c.observe(.hello(pairId: "someone-else", companionName: "Other", bootstrap: false))
        XCTAssertEqual(c.phase, .codeShown(a), "an already paired companion connecting is not the pairing peer")
        c.observe(.connectionOpened)
        c.observe(.connectionClosed(identity: nil, reason: "handshake failed"))
        XCTAssertEqual(c.phase, .interrupted(a, .peerGone, detail: "handshake failed"))
        XCTAssertTrue(c.phase.canResume())
        c.resume()
        XCTAssertEqual(c.phase, .codeShown(a))
        XCTAssertEqual(state.pairingCode, a.code, "the transport still serves the code; no restart needed")
        c.cancel()
        XCTAssertEqual(c.phase, .advertising)

        c.observeProgress(pairId: "p", companionName: "iPad", captureId: nil, bytes: 512, total: 2048)
        XCTAssertEqual(c.phase.detail(), "Receiving 512 of 2048 bytes from iPad.")
        XCTAssertFalse(c.phase.canCancel())
        c.observe(.capture(captureId: "c-1"))
        XCTAssertEqual(c.phase, .advertising)
        c.observe(.failed("listener error: simulated"))
        XCTAssertEqual(c.phase, .failed(reason: "listener error: simulated", generation: 1))
        c.dismiss()
        state.stopAdvertising()
    }

    func testControllerIsSharedPerStateAndSurvivesWindowReopen() {
        let (state, _) = makeState()
        let journal = PairingJournal(url: dir.appendingPathComponent("pairing-session.json"))
        let a = PairingFlowController.controller(for: state, journal: journal)
        let b = PairingFlowController.controller(for: state)
        XCTAssertTrue(a === b)
        XCTAssertTrue(a.journal === journal)
    }
}

// MARK: - evidence: window screenshots per state

/// Opt-in (`FLASHTEX_NEARBY_SCREENSHOT_DIR=<dir>`): hosts the real
/// `NearbyFlowView` in an NSWindow without activating the test process and
/// captures each pairing state by window id (`screencapture -l`). Skipped
/// otherwise so the suite never needs Screen Recording permission.
@MainActor
final class NearbyViewScreenshotTests: XCTestCase {
    func testCaptureEachPairingState() async throws {
        guard let out = ProcessInfo.processInfo.environment["FLASHTEX_NEARBY_SCREENSHOT_DIR"], !out.isEmpty else {
            throw XCTSkip("set FLASHTEX_NEARBY_SCREENSHOT_DIR to capture window evidence")
        }
        let outDir = URL(fileURLWithPath: out)
        try FileManager.default.createDirectory(at: outDir, withIntermediateDirectories: true)
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("nearby-shot-\(UUID().uuidString)")
        defer { try? FileManager.default.removeItem(at: dir) }
        let store = PairStore(url: dir.appendingPathComponent("pairs.json"))
        let model = ShellModel()
        let state = NearbyState(store: store, macName: "Evidence Mac", loopbackOnly: true)
        state.attach(sink: model, destinations: model)
        let controller = PairingFlowController(nearby: state, journal: PairingJournal(url: dir.appendingPathComponent("pairing-session.json")))
        controller.announcer = { _ in }

        _ = NSApplication.shared
        NSApp.setActivationPolicy(.accessory)
        let host = NSHostingView(rootView: NearbyFlowView(controller: controller, nearby: state, model: model))
        let window = NSWindow(contentRect: NSRect(x: 80, y: 80, width: 520, height: 640),
                              styleMask: [.titled, .closable, .resizable], backing: .buffered, defer: false)
        window.title = "Nearby Companion (evidence)"
        window.contentView = host
        window.orderFrontRegardless() // never `makeKey`: the user keeps focus
        defer { window.orderOut(nil) }

        func shot(_ name: String) async throws {
            for _ in 0..<4 { try await Task.sleep(nanoseconds: 200_000_000) }
            let path = outDir.appendingPathComponent("nearby-\(name).png").path
            let p = Process()
            p.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
            p.arguments = ["-x", "-o", "-l", String(window.windowNumber), path]
            try p.run()
            p.waitUntilExit()
            XCTAssertEqual(p.terminationStatus, 0, "screencapture \(name)")
            XCTAssertTrue(FileManager.default.fileExists(atPath: path), path)
        }

        try await shot("1-off")
        controller.showCode()
        try await shot("2-code-shown")
        controller.observe(.connectionOpened)
        try await shot("3-verifying")
        guard case .verifying(let a) = controller.phase else { return XCTFail("\(controller.phase)") }
        controller.observe(.connectionClosed(identity: nil, reason: "peer closed"))
        try await shot("4-interrupted-peer-gone")
        controller.resume()
        controller.apply(.confirmed(pairId: a.pairId, companionName: "Evidence iPad", generation: a.generation))
        XCTAssertTrue(store.upsert(PairRecord(pairId: a.pairId, psk: Pairing.mintLongTermPSK().base64EncodedString(),
                                              companionName: "Evidence iPad", createdAt: Date(), lastSeenAt: Date())))
        state.refreshPairs()
        try await shot("5-paired")
        controller.dismiss()
        controller.observeProgress(pairId: a.pairId, companionName: "Evidence iPad", captureId: "cap-7", bytes: 3_145_728, total: 8_388_608)
        try await shot("6-receiving")
        controller.observe(.capture(captureId: "cap-7"))
        controller.observe(.failed("listener error: simulated port loss"))
        try await shot("7-error")
        controller.dismiss()
        state.stopAdvertising()
        controller.apply(.restored(PairingFlow.Attempt(generation: 9, code: "424242", pairId: "relaunch",
                                                        startedAt: Date().addingTimeInterval(-30), expiresAt: Date().addingTimeInterval(90))))
        try await shot("8-interrupted-relaunch")
        controller.cancel()
    }
}
