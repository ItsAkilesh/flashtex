import Foundation
@testable import NearbyClient

/// Captures what the reference client puts on the wire and what its
/// reconnector reports, as a JSON Lines transcript that can be compared with
/// (or written to) a fixture under `Tests/Fixtures`.
///
/// One transcript line per wire line or reconnector event, in the order the
/// client observed them:
///
///     {"conn":1,"dir":"sent","line":{…envelope…}}
///     {"conn":1,"dir":"received","line":{…envelope…}}
///     {"conn":1,"event":"attempt","of":4}
///     {"conn":1,"event":"connected","mac_name":"Fake Mac","destination":"anchor-7 @ rev 3"}
///     {"conn":1,"event":"disconnected","reason":"peer closed"}
///     {"conn":1,"event":"failed","error":"closed","retry_in":0.25}
///     {"conn":2,"event":"gave_up","error":"handshake_failed"}
///
/// `conn` counts the reconnector's dials (event `attempt` opens the next
/// one). Volatile values are replaced so a replay compares by shape: envelope
/// ids become `<id-N>` in order of first appearance, `nonce`/`proof`/`pair_psk`
/// become `<nonce>`/`<proof>`/`<pair_psk>`, and error text is reduced to the
/// `NearbyError` case (`closed`, `handshake_failed`, `remote:pair_mismatch`,
/// …) because the OS's wording of a reset or a TLS alert is not stable. The
/// 1×1 PNG base64 is kept verbatim (it is the fixture image).
///
/// The first line of a fixture file is a header:
///
///     {"fixture":"…","scenario":"…","provenance":"…","recorded_utc":"…"}
///
/// Everything after it is compared line by line. With
/// `NEARBY_CLIENT_RECORD_FIXTURES=1` the tests rewrite the fixture instead.
final class TranscriptRecorder: @unchecked Sendable {
    private let lock = NSLock()
    private var connection = 0
    private var ids: [String: Int] = [:]
    private(set) var lines: [String] = []

    /// Hook for `NearbyConnection.onLine` / `NearbyReconnector(onLine:)`.
    func line(_ direction: NearbyConnection.Direction, _ data: Data) {
        lock.withLock {
            var object = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] ?? ["undecodable": String(decoding: data, as: UTF8.self)]
            if let id = object["id"] as? String {
                let n = ids[id] ?? ids.count + 1
                ids[id] = n
                object["id"] = "<id-\(n)>"
            }
            if var payload = object["payload"] as? [String: Any] {
                for key in ["nonce", "proof", "pair_psk"] where payload[key] != nil { payload[key] = "<\(key)>" }
                object["payload"] = payload
            }
            append(["conn": max(connection, 1), "dir": direction == .sent ? "sent" : "received", "line": object])
        }
    }

    /// Hook for `NearbyReconnector(onEvent:)`.
    func event(_ e: NearbyReconnector.Event) {
        lock.withLock {
            switch e {
            case .attempt(_, let of):
                connection += 1
                append(["conn": connection, "event": "attempt", "of": of])
            case .connected(let mac, _, let destination):
                append(["conn": connection, "event": "connected", "mac_name": mac,
                        "destination": destination.map { "\($0.destinationId) @ rev \($0.baseRevision)" } ?? NSNull()])
            case .failed(_, let error, let retryIn):
                append(["conn": connection, "event": "failed", "error": Self.kind(of: error), "retry_in": retryIn ?? NSNull()])
            case .disconnected(let reason):
                append(["conn": connection, "event": "disconnected", "reason": Self.kind(ofCloseReason: reason)])
            case .waitingForAcks(let pending):
                append(["conn": connection, "event": "waiting_for_acks", "pending": pending])
            case .gaveUp(let error):
                append(["conn": connection, "event": "gave_up", "error": Self.kind(of: error)])
            }
        }
    }

    private func append(_ object: [String: Any]) {
        let data = try! JSONSerialization.data(withJSONObject: object, options: [.sortedKeys, .withoutEscapingSlashes])
        lines.append(String(decoding: data, as: UTF8.self))
    }

    /// `NearbyError.description` → the case name (plus the code for `remote`).
    static func kind(of description: String) -> String {
        let prefixes: [(String, String)] = [
            ("TLS-PSK handshake failed", "handshake_failed"), ("connection closed", "closed"), ("Mac unreachable", "unreachable"),
            ("timed out", "timeout"), ("protocol violation", "protocol_violation"), ("invalid input", "invalid_input"),
            ("client overloaded", "overloaded"), ("destination changed", "destination_changed"), ("gave up after", "attempts_exhausted"),
            ("cancelled", "cancelled"), ("Bonjour browse failed", "browse_failed"), ("no matching Mac", "no_matching_mac"),
        ]
        if description.hasPrefix("Mac replied error ") {
            let rest = description.dropFirst("Mac replied error ".count)
            return "remote:" + String(rest.prefix { $0 != ":" })
        }
        return prefixes.first { description.hasPrefix($0.0) }?.1 ?? "other"
    }

    /// `NearbyConnection.closeReason` → a stable word.
    static func kind(ofCloseReason reason: String) -> String {
        if reason.hasPrefix("peer closed") || reason.hasPrefix("receive error") || reason.hasPrefix("connection closed") { return "peer_closed" }
        if reason.hasPrefix("handshake failed") { return "handshake_failed" }
        if reason.hasPrefix("closed by client") { return "closed_by_client" }
        if reason.hasPrefix("unreachable") { return "unreachable" }
        return kind(of: reason)
    }

    // MARK: fixtures

    static let fixturesDirectory = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent()
        .appendingPathComponent("Fixtures")
    static var recording: Bool { ProcessInfo.processInfo.environment["NEARBY_CLIENT_RECORD_FIXTURES"] == "1" }

    struct Fixture {
        var header: [String: Any]
        var lines: [String]
        static func load(_ name: String) throws -> Fixture {
            let raw = try String(contentsOf: fixturesDirectory.appendingPathComponent(name), encoding: .utf8)
            var lines = raw.split(separator: "\n", omittingEmptySubsequences: false).map(String.init)
            if lines.last == "" { lines.removeLast() }
            guard let first = lines.first, let header = try JSONSerialization.jsonObject(with: Data(first.utf8)) as? [String: Any],
                  header["fixture"] as? String == name else {
                throw NearbyError.invalidInput("\(name): first line is not a fixture header")
            }
            return Fixture(header: header, lines: Array(lines.dropFirst()))
        }
    }

    /// Records (env `NEARBY_CLIENT_RECORD_FIXTURES=1`) or returns the mismatch
    /// against the fixture, nil when the transcript matches line by line.
    func check(against name: String, scenario: String) throws -> String? {
        let mine = lock.withLock { lines }
        if Self.recording {
            let header: [String: Any] = [
                "fixture": name, "scenario": scenario,
                "provenance": "captured from the reference client (apps/mac/tools/nearby-client, NearbyConnection.onLine + NearbyReconnector events) against the loopback FakeMac in NearbyTranscriptFixtureTests; ids/nonce/proof normalised; not an iPad simulator run (the companion app cannot authenticate yet)",
                "recorded_utc": ISO8601DateFormatter().string(from: Date()),
            ]
            let head = try JSONSerialization.data(withJSONObject: header, options: [.sortedKeys, .withoutEscapingSlashes])
            try FileManager.default.createDirectory(at: Self.fixturesDirectory, withIntermediateDirectories: true)
            try (([String(decoding: head, as: UTF8.self)] + mine).joined(separator: "\n") + "\n")
                .write(to: Self.fixturesDirectory.appendingPathComponent(name), atomically: true, encoding: .utf8)
            return nil
        }
        let fixture = try Fixture.load(name)
        guard fixture.header["scenario"] as? String == scenario else { return "\(name): scenario header differs" }
        for (i, (want, got)) in zip(fixture.lines, mine).enumerated() where want != got {
            return "\(name) line \(i + 2):\n  fixture: \(want)\n  actual:  \(got)"
        }
        if fixture.lines.count != mine.count {
            return "\(name): fixture has \(fixture.lines.count) lines, transcript \(mine.count):\n" + mine.joined(separator: "\n")
        }
        return nil
    }
}
