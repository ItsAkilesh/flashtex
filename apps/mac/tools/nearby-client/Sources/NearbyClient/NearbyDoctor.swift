import Foundation
import Network

/// `nearby-client doctor`: validates one stored pairing against a running
/// listener, one check at a time, and prints the exact code of the first
/// thing that is wrong — never a guess, never a retry. Read-only on the Mac:
/// it opens one session (`hello`, `destination_query`) and closes it; no
/// capture is sent.
///
/// Checks, in order (a failed check stops the run; a warning does not):
///
/// | check       | codes                                                              |
/// |-------------|--------------------------------------------------------------------|
/// | store       | `store_unreadable`, `no_pairing`, `ambiguous_pairing`, `bad_pair_psk` |
/// | discovery   | `not_advertised`, `unsupported_service`, `fp_mismatch` (skipped with `--host`) |
/// | connect     | `unreachable`, `handshake_refused`, `connect_timeout`              |
/// | tls         | `tls_not_1_2`, `tls_suite_mismatch`                                 |
/// | hello       | the Mac's `error` code verbatim (`pair_mismatch`, `pairing_expired`, `too_many_sessions`, …), `protocol_violation`, `hello_timeout`, `closed` |
/// | destination | warn `no_destination` (nothing pinned on the Mac)                  |
///
/// Exit codes follow `send`: 0 healthy (warnings allowed) · 1 no pairing /
/// not advertised · 3 re-pair (`handshake_refused`, `pair_mismatch`,
/// `pairing_expired`, `fp_mismatch`) · 4 try later (`unreachable`,
/// `connect_timeout`, `hello_timeout`, `too_many_sessions`) · 2 anything else.
public enum NearbyDoctor {
    public struct Check: Equatable, Codable {
        public enum Status: String, Codable { case ok, warn, fail }
        public var name: String
        public var status: Status
        /// Exact code for `warn`/`fail`; nil when ok.
        public var code: String?
        public var detail: String
        public var line: String {
            switch status {
            case .ok: return "check \(name): ok — \(detail)"
            case .warn: return "check \(name): warn code=\(code ?? "?") — \(detail)"
            case .fail: return "check \(name): FAIL code=\(code ?? "?") — \(detail)"
            }
        }
    }

    public struct Report: Equatable, Codable {
        public var checks: [Check] = []
        public var exit: Int32 = 0
        public var healthy: Bool { !checks.contains { $0.status == .fail } }
        public var firstFailure: Check? { checks.first { $0.status == .fail } }
        public var summary: String {
            if let f = firstFailure { return "doctor: FAIL code=\(f.code ?? "?") (exit \(exit)) — \(hint(for: f.code ?? ""))" }
            let warns = checks.filter { $0.status == .warn }.compactMap(\.code)
            return warns.isEmpty ? "doctor: healthy (exit 0)" : "doctor: healthy with warnings \(warns.joined(separator: ", ")) (exit 0)"
        }
        public var json: String {
            let enc = JSONEncoder(); enc.outputFormatting = [.sortedKeys, .withoutEscapingSlashes]
            return String(decoding: try! enc.encode(self), as: UTF8.self)
        }
    }

    /// Exit code for a failing code (table above).
    public static func exitCode(for code: String) -> Int32 {
        switch code {
        case "no_pairing", "not_advertised", "store_unreadable": return 1
        case "handshake_refused", "pair_mismatch", "pairing_expired", "fp_mismatch": return 3
        case "unreachable", "connect_timeout", "hello_timeout", "too_many_sessions": return 4
        default: return 2
        }
    }

    public static func hint(for code: String) -> String {
        switch code {
        case "no_pairing": return "run `nearby-client pair` first"
        case "ambiguous_pairing": return "pick one with --mac <name|fp>"
        case "bad_pair_psk": return "the stored key is corrupt; `nearby-client forget` it and pair again"
        case "store_unreadable": return "fix the pair file path/permissions (--store)"
        case "not_advertised": return "the Mac is not advertising (Edit > Nearby Companion… > Advertise) or is on another network"
        case "unsupported_service": return "the Mac advertises a version/TXT this client does not support; update one side"
        case "fp_mismatch": return "the Mac's identity changed (re-salted or reinstalled); forget this pairing and pair again"
        case "unreachable": return "the port is closed or unroutable; the Mac may be restarting — try again"
        case "connect_timeout", "hello_timeout": return "no answer in time; check the network and try again"
        case "handshake_refused", "pair_mismatch", "pairing_expired": return "the Mac no longer accepts this pairing; run `nearby-client pair` again"
        case "tls_not_1_2", "tls_suite_mismatch": return "the listener negotiated parameters outside nearby v1; this is not a FlashTeX listener"
        case "too_many_sessions": return "close the companion's other connections to this Mac and try again"
        case "no_destination": return "nothing is pinned on the Mac (Edit > Pin Insertion Point); sends will be refused until then"
        default: return "see the Mac's Activity log for this code"
        }
    }

    public struct Options {
        public var store: PairFile
        public var macKey: String?
        public var fixedEndpoint: NWEndpoint?
        public var browseSeconds: TimeInterval = 5
        public var connectTimeout: TimeInterval = 10
        public var onLine: ((NearbyConnection.Direction, Data) -> Void)?
        public init(store: PairFile, macKey: String? = nil, fixedEndpoint: NWEndpoint? = nil, browseSeconds: TimeInterval = 5,
                    connectTimeout: TimeInterval = 10, onLine: ((NearbyConnection.Direction, Data) -> Void)? = nil) {
            self.store = store; self.macKey = macKey; self.fixedEndpoint = fixedEndpoint; self.browseSeconds = browseSeconds
            self.connectTimeout = connectTimeout; self.onLine = onLine
        }
    }

    /// Runs every check, emitting each line as it completes.
    public static func run(_ o: Options, emit: @escaping (String) -> Void) async -> Report {
        var report = Report()
        func add(_ c: Check) { report.checks.append(c); emit(c.line) }
        func fail(_ name: String, _ code: String, _ detail: String) -> Report {
            add(Check(name: name, status: .fail, code: code, detail: detail))
            report.exit = exitCode(for: code)
            emit(report.summary)
            return report
        }

        // store
        let pair: PairedMac
        if let key = o.macKey {
            guard let p = o.store.pair(matching: key) else { return fail("store", "no_pairing", "no stored pairing matches \(key) in \(o.store.url.path)") }
            pair = p
        } else if o.store.pairs.count == 1 {
            pair = o.store.pairs[0]
        } else if o.store.pairs.isEmpty {
            return fail("store", "no_pairing", "no pairings stored in \(o.store.url.path)")
        } else {
            return fail("store", "ambiguous_pairing", "\(o.store.pairs.count) pairings stored in \(o.store.url.path)")
        }
        guard let psk = pair.psk, psk.count == NearbyCrypto.pskLength else {
            return fail("store", "bad_pair_psk", "pair_psk for \(pair.macName) is not a 32-byte base64 key")
        }
        add(Check(name: "store", status: .ok, code: nil,
                  detail: "\(pair.macName) fp=\(pair.fingerprint) pair_id=\(pair.pairId) as \"\(pair.companionName)\" (\(o.store.url.path))"))

        // discovery
        let endpoint: NWEndpoint
        if let fixed = o.fixedEndpoint {
            endpoint = fixed
            add(Check(name: "discovery", status: .ok, code: nil, detail: "skipped (--host/--port \(fixed))"))
        } else {
            let results: [DiscoveredMac]
            do {
                results = try await NearbyBrowser.discover(seconds: o.browseSeconds) { $0.fingerprint == pair.fingerprint && $0.isSupported }
            } catch let e as NearbyError {
                return fail("discovery", "not_advertised", e.description)
            } catch { return fail("discovery", "not_advertised", "\(error)") }
            if let mac = results.first(where: { $0.fingerprint == pair.fingerprint }) {
                if let why = mac.unsupportedReason { return fail("discovery", "unsupported_service", "\(mac.name): \(why)") }
                endpoint = mac.endpoint
                add(Check(name: "discovery", status: .ok, code: nil, detail: "\(mac.name) fp=\(pair.fingerprint) v=\(mac.version ?? "?") at \(mac.endpoint)"))
            } else if let same = results.first(where: { $0.macName == pair.macName || $0.name == pair.macName }) {
                return fail("discovery", "fp_mismatch", "\"\(same.name)\" advertises fp \(same.fingerprint ?? "?"), the pairing has \(pair.fingerprint)")
            } else {
                let seen = results.map { "\($0.name) fp=\($0.fingerprint ?? "?")" }
                return fail("discovery", "not_advertised", "no \(NearbyWire.serviceType) service with fp \(pair.fingerprint) within \(Int(o.browseSeconds))s (seen: \(seen.isEmpty ? "none" : seen.joined(separator: "; ")))")
            }
        }

        // connect (TCP + TLS-PSK)
        let conn = NearbyConnection(endpoint: endpoint, pairId: pair.pairId, psk: psk)
        conn.onLine = o.onLine
        defer { conn.close() }
        let dialed = Date()
        do { try await conn.connect(timeout: o.connectTimeout) } catch let e as NearbyError {
            switch e {
            case .unreachable(let why): return fail("connect", "unreachable", why)
            case .handshakeFailed(let why): return fail("connect", "handshake_refused", "TLS-PSK refused for pair_id \(pair.pairId): \(why)")
            case .timeout(let why): return fail("connect", "connect_timeout", why)
            default: return fail("connect", "unreachable", e.description)
            }
        } catch { return fail("connect", "unreachable", "\(error)") }
        add(Check(name: "connect", status: .ok, code: nil,
                  detail: "TLS-PSK ready in \(String(format: "%.3f", Date().timeIntervalSince(dialed)))s (\(endpoint))"))

        // tls
        if let n = conn.negotiated {
            guard n.tlsv12 else { return fail("tls", "tls_not_1_2", "negotiated protocol is not TLS 1.2") }
            guard n.suite == NearbyCrypto.cipherSuiteRawValue else {
                return fail("tls", "tls_suite_mismatch", "suite 0x\(String(n.suite, radix: 16)) != 0x\(String(NearbyCrypto.cipherSuiteRawValue, radix: 16))")
            }
            add(Check(name: "tls", status: .ok, code: nil, detail: "1.2 suite 0x\(String(n.suite, radix: 16))"))
        } else {
            add(Check(name: "tls", status: .warn, code: "tls_unknown", detail: "negotiated parameters not reported by the framework"))
        }

        // hello
        let ack: NearbyWire.HelloAck
        do { ack = try await conn.hello(companionName: pair.companionName, timeout: o.connectTimeout) } catch let e as NearbyError {
            switch e {
            case .remote(let code, let message): return fail("hello", code, message + (e.isClosing ? " (the Mac closes after this code)" : ""))
            case .timeout(let why): return fail("hello", "hello_timeout", why)
            case .closed(let why): return fail("hello", "closed", why)
            case .protocolViolation(let why): return fail("hello", "protocol_violation", why)
            default: return fail("hello", "protocol_violation", e.description)
            }
        } catch { return fail("hello", "protocol_violation", "\(error)") }
        add(Check(name: "hello", status: .ok, code: nil, detail: "hello_ack from \"\(ack.macName)\"" + (ack.pairPsk != nil ? " (unexpected pair_psk on a long-term key)" : "")))

        // destination
        do {
            let d = try await conn.destinationQuery(timeout: o.connectTimeout)
            if let d {
                add(Check(name: "destination", status: .ok, code: nil, detail: "\(d.destinationId) (\(d.projectId)/\(d.path) @ rev \(d.baseRevision))"))
            } else {
                add(Check(name: "destination", status: .warn, code: "no_destination", detail: "the Mac reports no pinned insertion point"))
            }
        } catch let e as NearbyError {
            if case .remote(let code, let message) = e { return fail("destination", code, message) }
            return fail("destination", "closed", e.description)
        } catch { return fail("destination", "closed", "\(error)") }

        emit(report.summary)
        return report
    }
}
