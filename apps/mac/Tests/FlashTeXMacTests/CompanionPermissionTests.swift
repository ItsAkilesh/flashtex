import Foundation
import XCTest
import FlashTeXProtocol
import NearbyClient
@testable import FlashTeXMac

/// Pairing follow-up 2 (mac-pairing-ui-3): per-companion permission stored
/// with the pairing record (pairs.json v3, key `permission`), enforced by the
/// real listener with `capture_not_permitted`, and mapped by the reference
/// client as "pairing intact" (needsRepair=false, exit 6).
@MainActor
final class CompanionPermissionTests: XCTestCase {
    private var dir: URL!

    override func setUp() {
        super.setUp()
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("companion-permission-\(UUID().uuidString)")
        try! FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    }

    override func tearDown() {
        try? FileManager.default.removeItem(at: dir)
        super.tearDown()
    }

    private func waitUntil(_ what: String, timeout: TimeInterval = 15, file: StaticString = #filePath, line: UInt = #line,
                           _ cond: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if cond() { return }
            try await Task.sleep(nanoseconds: 20_000_000)
        }
        XCTFail("timed out waiting for \(what)", file: file, line: line)
    }

    private func cli(_ args: [String]) async -> (code: Int32, out: [String]) {
        var out: [String] = []
        let code = await NearbyCLI.run(args) { out.append($0) }
        return (code, out)
    }

    // MARK: store migration

    /// A v2 file (no `permission`) is upgraded in place to v3 with the
    /// explicit `captures` value it behaved as; the change is persisted with
    /// the versioned key and survives a reload; the listener's check reads it.
    func testV2StoreUpgradesToV3WithExplicitPermissionAndPersistsChanges() throws {
        let url = dir.appendingPathComponent("pairs.json")
        let key = Data(repeating: 7, count: Pairing.pskLength).base64EncodedString()
        try Data("""
        {"version": 2, "salt": "000102030405060708090a0b0c0d0e0f", "pairs": [
          {"pair_id": "p1", "psk": "\(key)", "companion_name": "Old iPad", "created_at": "2026-09-12T00:00:00Z", "generation": 4},
          {"pair_id": "p2", "psk": "\(key)", "companion_name": "Older iPad", "created_at": "2026-09-11T00:00:00Z"}
        ]}
        """.utf8).write(to: url)
        let store = PairStore(url: url)
        XCTAssertEqual(store.loadOutcome, .upgraded(from: 2))
        XCTAssertNil(store.loadError)
        XCTAssertEqual(store.pairs.map(\.permission), [.captures, .captures], "v2 records get the explicit value they behaved as")
        XCTAssertEqual(store.pair(id: "p1")?.generation, 4, "nothing else is rewritten")
        XCTAssertTrue(store.capturesPermitted(pairId: "p1"))
        XCTAssertTrue(store.capturesPermitted(pairId: "unknown"), "a pairing the store does not hold is not this check's business")

        let rewritten = try JSONSerialization.jsonObject(with: Data(contentsOf: url)) as? [String: Any]
        XCTAssertEqual(rewritten?["version"] as? Int, 3)
        let records = try XCTUnwrap(rewritten?["pairs"] as? [[String: Any]])
        XCTAssertEqual(records.compactMap { $0["permission"] as? String }, ["captures", "captures"])

        XCTAssertTrue(store.setPermission(pairId: "p2", .viewOnly))
        XCTAssertFalse(store.setPermission(pairId: "nope", .viewOnly))
        XCTAssertFalse(store.capturesPermitted(pairId: "p2"))
        XCTAssertTrue(store.capturesPermitted(pairId: "p1"))
        let reloaded = PairStore(url: url)
        XCTAssertEqual(reloaded.loadOutcome, .loaded(version: 3))
        XCTAssertEqual(reloaded.pair(id: "p2")?.permission, .viewOnly)
        XCTAssertEqual(reloaded.pair(id: "p2")?.effectivePermission, .viewOnly)
        XCTAssertEqual(reloaded.pair(id: "p1")?.effectivePermission, .captures)
        XCTAssertEqual(CompanionPermission.viewOnly.rawValue, "view_only")
        XCTAssertEqual(CompanionPermission.allCases.map(\.title), ["Captures allowed", "View only"])
        // A v3 record without the key still decodes (nil → captures), so a
        // hand-edited file that drops it does not lock the companion out.
        try Data("""
        {"version": 3, "salt": "000102030405060708090a0b0c0d0e0f", "pairs": [
          {"pair_id": "p3", "psk": "\(key)", "companion_name": "Edited", "created_at": "2026-09-12T00:00:00Z"}
        ]}
        """.utf8).write(to: url)
        XCTAssertEqual(PairStore(url: url).pair(id: "p3")?.effectivePermission, .captures)
    }

    // MARK: real listener + reference client

    /// Pair the reference client, set it to view-only in `NearbyState` (what
    /// the window's pop-up does), send: the real listener answers
    /// `capture_not_permitted` on the open session, nothing reaches the
    /// inbox, the window shows the refusal, the client exits 6 with
    /// needsRepair=false (no re-pair hint). Back to captures allowed: the
    /// identical capture is accepted on the same pairing.
    func testViewOnlyCompanionIsRefusedWithCaptureNotPermittedAndAcceptedAgainAfterTheChange() async throws {
        let store = PairStore(url: dir.appendingPathComponent("mac-pairs.json"))
        let model = ShellModel()
        model.caretUTF16 = 6
        model.pinAnchorAtCaret()
        let macName = "FlashTeX Perm \(UUID().uuidString.prefix(6))"
        let state = NearbyState(store: store, macName: macName, loopbackOnly: true)
        state.attach(sink: model, destinations: model)
        state.startAdvertising()
        try await waitUntil("advertising") { state.isAdvertising && state.port != nil }
        state.beginPairing()
        let code = try XCTUnwrap(state.pairingCode)
        try await waitUntil("restarted with bootstrap key") { state.log.filter { $0.hasPrefix("ready on port") }.count >= 2 }
        let fp = state.fingerprint
        let clientStore = dir.appendingPathComponent("client-pairs.json").path
        let png = dir.appendingPathComponent("dot.png")
        try NearbyReferenceClientTests.fixturePNG.write(to: png)

        let pair = await cli(["pair", "--code", code, "--name", "View iPad", "--mac", fp, "--seconds", "10", "--store", clientStore])
        XCTAssertEqual(pair.code, 0, pair.out.joined(separator: "\n"))
        try await waitUntil("pair stored on the Mac") { state.pairs.count == 1 && state.pairingCode == nil }
        let pairId = try XCTUnwrap(state.pairs.first?.pairId)
        XCTAssertEqual(state.pairs.first?.effectivePermission, .captures)
        try await waitUntil("restarted with long-term key") { state.log.filter { $0.hasPrefix("ready on port") }.count >= 3 }
        let restarts = state.log.filter { $0.hasPrefix("ready on port") }.count

        // The window's pop-up: view only. No listener restart is needed.
        state.setPermission(pairId: pairId, .viewOnly)
        XCTAssertEqual(state.pairs.first?.effectivePermission, .viewOnly)
        XCTAssertEqual(PairStore(url: store.url).pair(id: pairId)?.permission, .viewOnly, "persisted at once")
        XCTAssertTrue(state.log.last?.hasSuffix("permission: view_only") == true, "\(state.log.suffix(2))")
        XCTAssertEqual(PairingAccessibility.deviceRow(state.pairs[0], connected: false).value.contains("Permission: view only, captures refused."), true)
        state.setPermission(pairId: pairId, .viewOnly) // idempotent: no second log line
        XCTAssertEqual(state.log.filter { $0.hasSuffix("permission: view_only") }.count, 1)

        let refused = await cli(["send", "--image", png.path, "--instructions", "not allowed", "--capture-id", "perm-cap-1",
                                 "--mac", fp, "--seconds", "10", "--store", clientStore, "-v"])
        XCTAssertEqual(refused.code, 6, refused.out.joined(separator: "\n"))
        XCTAssertTrue(refused.out.contains { $0.hasPrefix("error: Mac replied error capture_not_permitted:") }, "\(refused.out)")
        XCTAssertTrue(refused.out.contains { $0.hasPrefix("hint: this companion is view-only") && $0.contains("no re-pair") }, "\(refused.out)")
        XCTAssertFalse(refused.out.contains { $0.contains("the Mac closes after this code") }, "session kept open: \(refused.out)")
        XCTAssertFalse(refused.out.contains { $0.contains("run `nearby-client pair` again") }, "needsRepair=false: \(refused.out)")
        try await waitUntil("refusal shown in the window") { state.lastReceiveError?.code == "capture_not_permitted" }
        XCTAssertEqual(state.lastReceiveError?.captureId, "perm-cap-1")
        XCTAssertEqual(state.lastReceiveError?.pairId, pairId)
        XCTAssertTrue(model.nearbyInbox.received.isEmpty, "nothing reached the inbox")
        XCTAssertNil(state.lastReceivedCaptureId)
        XCTAssertEqual(state.pairs.first?.captureCount ?? 0, 0, "a refused capture is not counted")
        XCTAssertEqual(state.log.filter { $0.hasPrefix("ready on port") }.count, restarts, "permission changes never restart the listener")
        XCTAssertEqual(state.pairs.count, 1, "the pairing is intact")

        // Captures allowed again: the same capture id and payload are delivered.
        state.setPermission(pairId: pairId, .captures)
        XCTAssertEqual(PairStore(url: store.url).pair(id: pairId)?.permission, .captures)
        let accepted = await cli(["send", "--image", png.path, "--instructions", "not allowed", "--capture-id", "perm-cap-1",
                                  "--mac", fp, "--seconds", "10", "--store", clientStore])
        XCTAssertEqual(accepted.code, 0, accepted.out.joined(separator: "\n"))
        try await waitUntil("inbox") { model.nearbyInbox.lastCaptureId == "perm-cap-1" }
        XCTAssertEqual(model.nearbyInbox.received.count, 1)
        XCTAssertEqual(state.pairs.first?.captureCount, 1)
        XCTAssertEqual(state.pairs.first?.effectivePermission, .captures)
        state.stopAdvertising()
    }

    /// The window's refusal state and the store agree after a relaunch: a
    /// view-only pairing loaded from disk is still refused by a fresh
    /// listener (the check reads the store, not a cached flag).
    func testViewOnlyPermissionSurvivesRelaunchAgainstTheRealListener() async throws {
        let url = dir.appendingPathComponent("mac-pairs.json")
        let first = NearbyState(store: PairStore(url: url), macName: "Relaunch Mac", loopbackOnly: true)
        let model = ShellModel()
        model.caretUTF16 = 6
        model.pinAnchorAtCaret()
        first.attach(sink: model, destinations: model)
        first.startAdvertising()
        try await waitUntil("advertising") { first.isAdvertising }
        first.beginPairing()
        let code = try XCTUnwrap(first.pairingCode)
        try await waitUntil("bootstrap served") { first.log.filter { $0.hasPrefix("ready on port") }.count >= 2 }
        let clientStore = dir.appendingPathComponent("client-pairs.json").path
        let pair = await cli(["pair", "--code", code, "--name", "Relaunch iPad", "--mac", first.fingerprint, "--seconds", "10", "--store", clientStore])
        XCTAssertEqual(pair.code, 0, pair.out.joined(separator: "\n"))
        try await waitUntil("paired") { first.pairs.count == 1 }
        let pairId = try XCTUnwrap(first.pairs.first?.pairId)
        first.setPermission(pairId: pairId, .viewOnly)
        first.stopAdvertising()

        let second = NearbyState(store: PairStore(url: url), macName: "Relaunch Mac", loopbackOnly: true)
        second.attach(sink: model, destinations: model)
        XCTAssertEqual(second.pairs.first?.effectivePermission, .viewOnly, "loaded from pairs.json v3")
        second.startAdvertising()
        try await waitUntil("advertising again") { second.isAdvertising && second.port != nil }
        let png = dir.appendingPathComponent("dot.png")
        try NearbyReferenceClientTests.fixturePNG.write(to: png)
        let refused = await cli(["send", "--image", png.path, "--capture-id", "relaunch-cap", "--mac", second.fingerprint,
                                 "--seconds", "10", "--store", clientStore])
        XCTAssertEqual(refused.code, 6, refused.out.joined(separator: "\n"))
        try await waitUntil("refusal recorded") { second.lastReceiveError?.code == "capture_not_permitted" }
        XCTAssertTrue(model.nearbyInbox.received.isEmpty)
        second.stopAdvertising()
    }
}
