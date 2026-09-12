import Network
import XCTest
@testable import NearbyClient

/// `nearby-client doctor` against the fake Mac: every check's exact code and
/// exit code, in-process through `NearbyCLI.run` (what the executable runs).
final class NearbyDoctorTests: XCTestCase {
    let salt = NearbyCrypto.data(hex: "0f0e0d0c0b0a09080706050403020100")!
    let pairId = "00112233aabbccdd"
    let psk = Data(repeating: 0x42, count: 32)
    let anchor = NearbyWire.Destination(destinationId: "anchor-7", projectId: "demo", path: "main.tex", baseRevision: 3)
    var dir: URL!
    var store: String { dir.appendingPathComponent("pairs.json").path }

    override func setUpWithError() throws {
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("nearby-doctor-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    }
    override func tearDown() { try? FileManager.default.removeItem(at: dir) }

    func pair(fp: String? = nil, psk key: String? = nil, name: String = "Fake Mac") -> PairedMac {
        PairedMac(fingerprint: fp ?? NearbyCrypto.fingerprint(salt: salt), macName: name, pairId: pairId,
                  pairPsk: key ?? psk.base64EncodedString(), companionName: "Doctor iPad")
    }
    func storePairs(_ pairs: [PairedMac]) throws {
        let f = try PairFile(url: URL(fileURLWithPath: store))
        for p in pairs { try f.upsert(p) }
    }
    func mac(key: Data? = nil, destination: NearbyWire.Destination? = nil, advertise: String? = nil) throws -> FakeMac {
        let m = try FakeMac(keys: [.init(identity: pairId, psk: key ?? psk, bootstrap: false)], destination: destination, advertise: advertise,
                            salt: advertise == nil ? nil : salt)
        m.start()
        XCTAssertGreaterThan(m.port, 0)
        return m
    }
    func doctor(_ extra: [String]) async -> (code: Int32, out: [String]) {
        var out: [String] = []
        let code = await NearbyCLI.run(["doctor", "--store", store] + extra) { out.append($0) }
        return (code, out)
    }
    func report(_ out: [String]) throws -> NearbyDoctor.Report {
        let line = try XCTUnwrap(out.last(where: { $0.hasPrefix("{") }), "no --json line in \(out)")
        return try JSONDecoder().decode(NearbyDoctor.Report.self, from: Data(line.utf8))
    }

    func testHealthyPairingPassesEveryCheckAndPrintsJSON() async throws {
        let m = try mac(destination: anchor)
        defer { m.stop() }
        try storePairs([pair()])
        let started = Date()
        let r = await doctor(["--host", "127.0.0.1", "--port", "\(m.port)", "--json"])
        let elapsed = Date().timeIntervalSince(started)
        XCTAssertEqual(r.code, 0, r.out.joined(separator: "\n"))
        let rep = try report(r.out)
        XCTAssertEqual(rep.checks.map(\.name), ["store", "discovery", "connect", "tls", "hello", "destination"])
        XCTAssertEqual(Set(rep.checks.map(\.status)), [.ok])
        XCTAssertTrue(rep.healthy); XCTAssertEqual(rep.exit, 0)
        XCTAssertTrue(r.out.contains("doctor: healthy (exit 0)"), "\(r.out)")
        XCTAssertTrue(r.out.contains { $0.hasPrefix("check discovery: ok — skipped (--host/--port") })
        XCTAssertTrue(r.out.contains { $0.hasPrefix("check tls: ok — 1.2 suite 0x") }, "\(r.out)")
        XCTAssertTrue(r.out.contains("check destination: ok — anchor-7 (demo/main.tex @ rev 3)"), "\(r.out)")
        XCTAssertEqual(m.hellos.count, 1); XCTAssertTrue(m.captures.isEmpty, "doctor never sends a capture")
        print("measured: doctor healthy run in \(String(format: "%.3f", elapsed))s (loopback)")
    }

    func testNoDestinationIsAWarningNotAFailure() async throws {
        let m = try mac()
        defer { m.stop() }
        try storePairs([pair()])
        let r = await doctor(["--host", "127.0.0.1", "--port", "\(m.port)"])
        XCTAssertEqual(r.code, 0, r.out.joined(separator: "\n"))
        XCTAssertTrue(r.out.contains("check destination: warn code=no_destination — the Mac reports no pinned insertion point"), "\(r.out)")
        XCTAssertTrue(r.out.contains("doctor: healthy with warnings no_destination (exit 0)"), "\(r.out)")
    }

    func testRevokedKeyIsHandshakeRefusedExit3BeforeAnyLine() async throws {
        let m = try mac(key: Data(repeating: 0x99, count: 32)) // the Mac forgot this pairing
        defer { m.stop() }
        try storePairs([pair()])
        let r = await doctor(["--host", "127.0.0.1", "--port", "\(m.port)", "--json"])
        XCTAssertEqual(r.code, 3, r.out.joined(separator: "\n"))
        let rep = try report(r.out)
        XCTAssertEqual(rep.firstFailure?.name, "connect"); XCTAssertEqual(rep.firstFailure?.code, "handshake_refused")
        XCTAssertEqual(rep.checks.count, 3, "store, discovery, connect — nothing after the failure")
        XCTAssertTrue(r.out.contains { $0.hasPrefix("check connect: FAIL code=handshake_refused — TLS-PSK refused for pair_id \(pairId)") }, "\(r.out)")
        XCTAssertTrue(r.out.contains { $0.hasPrefix("doctor: FAIL code=handshake_refused (exit 3) — the Mac no longer accepts this pairing") }, "\(r.out)")
        XCTAssertTrue(m.hellos.isEmpty)
    }

    func testHelloRefusalsReportTheMacsCodeVerbatim() async throws {
        let m = try mac(destination: anchor)
        defer { m.stop() }
        try storePairs([pair()])
        m.refuseHellos = ("pair_mismatch", "unknown pair", 1)
        let mismatch = await doctor(["--host", "127.0.0.1", "--port", "\(m.port)"])
        XCTAssertEqual(mismatch.code, 3, mismatch.out.joined(separator: "\n"))
        XCTAssertTrue(mismatch.out.contains("check hello: FAIL code=pair_mismatch — unknown pair (the Mac closes after this code)"), "\(mismatch.out)")
        m.refuseHellos = ("too_many_sessions", "1 session per pairing", 1)
        let busy = await doctor(["--host", "127.0.0.1", "--port", "\(m.port)"])
        XCTAssertEqual(busy.code, 4, busy.out.joined(separator: "\n"))
        XCTAssertTrue(busy.out.contains { $0.hasPrefix("doctor: FAIL code=too_many_sessions (exit 4) — close the companion's other connections") }, "\(busy.out)")
        XCTAssertFalse(busy.out.contains { $0.hasPrefix("check destination") }, "stops at the failed check")
    }

    func testClosedPortIsUnreachableExit4Fast() async throws {
        try storePairs([pair()])
        let port = try NearbyReconnectTests().closedPort()
        let started = Date()
        let r = await doctor(["--host", "127.0.0.1", "--port", "\(port)"])
        let elapsed = Date().timeIntervalSince(started)
        XCTAssertEqual(r.code, 4, r.out.joined(separator: "\n"))
        XCTAssertTrue(r.out.contains { $0.hasPrefix("check connect: FAIL code=unreachable — ") }, "\(r.out)")
        XCTAssertLessThan(elapsed, 5, "reported from .waiting, not after the connect timeout")
        print("measured: doctor unreachable (refused port) reported in \(String(format: "%.3f", elapsed))s")
    }

    func testStoreProblemsAreExactCodes() async throws {
        let none = await doctor([])
        XCTAssertEqual(none.code, 1); XCTAssertTrue(none.out[0].hasPrefix("check store: FAIL code=no_pairing — no pairings stored in"), "\(none.out)")
        try storePairs([pair(), pair(fp: "other-fp", name: "Other Mac")])
        let several = await doctor([])
        XCTAssertEqual(several.code, 2); XCTAssertTrue(several.out[0].hasPrefix("check store: FAIL code=ambiguous_pairing — 2 pairings stored"), "\(several.out)")
        let unknown = await doctor(["--mac", "nobody"])
        XCTAssertEqual(unknown.code, 1); XCTAssertTrue(unknown.out[0].hasPrefix("check store: FAIL code=no_pairing — no stored pairing matches nobody"), "\(unknown.out)")
        try storePairs([pair(fp: "other-fp", psk: "not base64!!", name: "Other Mac")])
        let corrupt = await doctor(["--mac", "Other Mac"])
        XCTAssertEqual(corrupt.code, 2); XCTAssertTrue(corrupt.out[0].hasPrefix("check store: FAIL code=bad_pair_psk — pair_psk for Other Mac is not a 32-byte"), "\(corrupt.out)")
        XCTAssertEqual(NearbyDoctor.exitCode(for: "fp_mismatch"), 3); XCTAssertEqual(NearbyDoctor.exitCode(for: "not_advertised"), 1)
        XCTAssertEqual(NearbyDoctor.exitCode(for: "hello_timeout"), 4); XCTAssertEqual(NearbyDoctor.exitCode(for: "tls_not_1_2"), 2)
    }

    func testBonjourDiscoveryFindsThePairedMacAndFlagsAReSaltedOne() async throws {
        let name = "doctor test \(UUID().uuidString.prefix(6))"
        let m = try mac(destination: anchor, advertise: name)
        defer { m.stop() }
        try storePairs([pair()])
        let ok = await doctor(["--seconds", "8", "--json"])
        XCTAssertEqual(ok.code, 0, ok.out.joined(separator: "\n"))
        XCTAssertTrue(ok.out.contains { $0.hasPrefix("check discovery: ok — \(name) fp=\(NearbyCrypto.fingerprint(salt: salt)) v=1 at") }, "\(ok.out)")
        XCTAssertEqual(try report(ok.out).checks.map(\.status), Array(repeating: .ok, count: 6))

        // The Mac was re-salted (reinstalled): same name, another fp → re-pair, not "not advertised".
        try FileManager.default.removeItem(atPath: store)
        try storePairs([pair(fp: "0000000000000000000000000000000000000000000000000000000000000000", name: name)])
        let stale = await doctor(["--seconds", "3"])
        XCTAssertEqual(stale.code, 3, stale.out.joined(separator: "\n"))
        XCTAssertTrue(stale.out.contains { $0.hasPrefix("check discovery: FAIL code=fp_mismatch — \"\(name)\" advertises fp \(NearbyCrypto.fingerprint(salt: salt)), the pairing has 0000") }, "\(stale.out)")

        // Nobody advertises this fp at all.
        try FileManager.default.removeItem(atPath: store)
        try storePairs([pair(fp: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", name: "Absent Mac")])
        let absent = await doctor(["--seconds", "2"])
        XCTAssertEqual(absent.code, 1, absent.out.joined(separator: "\n"))
        XCTAssertTrue(absent.out.contains { $0.hasPrefix("check discovery: FAIL code=not_advertised — no _flashtex._tcp service with fp ffff") }, "\(absent.out)")
    }
}
