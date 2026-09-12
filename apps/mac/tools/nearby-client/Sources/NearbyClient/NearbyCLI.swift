import Foundation
import Network

/// Command logic behind the `nearby-client` executable, kept in the library so
/// tests (including apps/mac's, against the real listener) can run the exact
/// CLI paths in-process. Output goes through `emit` one line at a time.
public enum NearbyCLI {
    public static let usage = """
    usage: nearby-client <command> [options]

      pair   --code 123456 --name "iPad Sim" [--mac <name|fp>] [--seconds 5]
             [--host 127.0.0.1 --port N --salt <32 hex>]      (skip Bonjour)
      send   --image file.png|jpg [--instructions "..."] [--capture-id ID]
             [--mac <name|fp>] [--host H --port N]
             [--destination-id ID --base-revision N]          (override the Mac's pin)
      status [--seconds 2]        stored pairings and which Macs are visible now
      browse [--seconds 2]        every _flashtex._tcp service and its TXT record
      forget --mac <name|fp>

    common: --store <path> (default $NEARBY_CLIENT_STORE or
            ~/Library/Application Support/FlashTeX/nearby-client-pairs.json),
            --timeout <s> (TLS connect, default 10), -v (print every JSON line)
    """

    public struct Failure: Error { public let code: Int32; public let message: String }

    /// Returns the process exit code. `emit` receives output lines (stdout).
    public static func run(_ arguments: [String], emit: @escaping (String) -> Void) async -> Int32 {
        do {
            try await main(arguments, emit: emit)
            return 0
        } catch let f as Failure {
            emit("error: \(f.message)")
            return f.code
        } catch let e as NearbyError {
            emit("error: \(e.description)" + (e.isClosing ? " (the Mac closes after this code: re-pair or fix the input, do not retry)" : ""))
            if case .handshakeFailed = e {
                emit("hint: the key was refused — the pairing code expired/was used, the Mac forgot this pairing, or this is not a FlashTeX nearby listener")
            }
            return 2
        } catch {
            emit("error: \(error)")
            return 2
        }
    }

    // MARK: argument parsing

    struct Options {
        var command = ""
        var values: [String: String] = [:]
        var verbose = false
        func string(_ k: String) -> String? { values[k] }
        func require(_ k: String) throws -> String {
            guard let v = values[k], !v.isEmpty else { throw Failure(code: 64, message: "missing --\(k)\n\(usage)") }
            return v
        }
        func double(_ k: String, default d: Double) throws -> Double {
            guard let s = values[k] else { return d }
            guard let v = Double(s) else { throw Failure(code: 64, message: "--\(k) must be a number") }
            return v
        }
        func int(_ k: String) throws -> Int? {
            guard let s = values[k] else { return nil }
            guard let v = Int(s) else { throw Failure(code: 64, message: "--\(k) must be an integer") }
            return v
        }
    }

    static func parse(_ args: [String]) throws -> Options {
        var o = Options()
        var it = args.makeIterator()
        guard let cmd = it.next() else { throw Failure(code: 64, message: usage) }
        o.command = cmd
        while let a = it.next() {
            if a == "-v" || a == "--verbose" { o.verbose = true; continue }
            guard a.hasPrefix("--") else { throw Failure(code: 64, message: "unexpected argument \(a)\n\(usage)") }
            let key = String(a.dropFirst(2))
            if let eq = key.firstIndex(of: "=") {
                o.values[String(key[..<eq])] = String(key[key.index(after: eq)...])
            } else {
                guard let v = it.next() else { throw Failure(code: 64, message: "--\(key) needs a value") }
                o.values[key] = v
            }
        }
        return o
    }

    // MARK: commands

    static func main(_ args: [String], emit: @escaping (String) -> Void) async throws {
        let o = try parse(args)
        switch o.command {
        case "pair": try await pair(o, emit: emit)
        case "send": try await send(o, emit: emit)
        case "status": try await status(o, emit: emit)
        case "browse": try await browse(o, emit: emit)
        case "forget": try forget(o, emit: emit)
        case "help", "--help", "-h": emit(usage)
        default: throw Failure(code: 64, message: "unknown command \(o.command)\n\(usage)")
        }
    }

    static func store(_ o: Options) throws -> PairFile {
        try PairFile(url: o.string("store").map { URL(fileURLWithPath: $0) } ?? PairFile.defaultURL())
    }

    static func lineLogger(_ o: Options, emit: @escaping (String) -> Void) -> ((NearbyConnection.Direction, Data) -> Void)? {
        guard o.verbose else { return nil }
        return { dir, line in emit((dir == .sent ? ">> " : "<< ") + abbreviate(line)) }
    }

    /// Keeps transcripts readable: the base64 image is replaced by its size.
    static func abbreviate(_ line: Data) -> String {
        var s = String(decoding: line, as: UTF8.self)
        while s.last == "\n" { s.removeLast() }
        if let r = s.range(of: "\"data_base64\":\""), let end = s[r.upperBound...].firstIndex(of: "\"") {
            let n = s.distance(from: r.upperBound, to: end)
            s.replaceSubrange(r.upperBound..<end, with: "<\(n) base64 chars>")
        }
        return s
    }

    /// Picks the Mac to talk to: `--host/--port` directly, else Bonjour by
    /// `--mac` (name or fp) or the only/first supported service.
    static func resolveEndpoint(_ o: Options, emit: @escaping (String) -> Void, wantFP: String? = nil) async throws -> (NWEndpoint, DiscoveredMac?) {
        if let host = o.string("host") {
            guard let p = try o.int("port"), let port = NWEndpoint.Port(rawValue: UInt16(clamping: p)) else {
                throw Failure(code: 64, message: "--host needs --port")
            }
            return (.hostPort(host: NWEndpoint.Host(host), port: port), nil)
        }
        let seconds = try o.double("seconds", default: 5)
        let key = o.string("mac") ?? wantFP
        emit("browsing \(NearbyWire.serviceType) for \(key.map { "\"\($0)\"" } ?? "any Mac") (up to \(Int(seconds))s)…")
        let results = try await NearbyBrowser.discover(seconds: seconds) { m in
            guard let key else { return m.isSupported }
            return m.isSupported && (m.fingerprint == key || m.name == key || m.macName == key)
        }
        let candidates = results.filter { m in
            guard let key else { return true }
            return m.fingerprint == key || m.name == key || m.macName == key
        }
        guard let mac = candidates.first(where: \.isSupported) else {
            let seen = results.map { "\($0.name) fp=\($0.fingerprint ?? "?")\($0.unsupportedReason.map { " (\($0))" } ?? "")" }
            throw NearbyError.noMatchingMac("\(key ?? "no supported service") (seen: \(seen.isEmpty ? "none" : seen.joined(separator: "; ")))")
        }
        if candidates.count > 1 { emit("note: \(candidates.count) matching services; using \(mac.name) fp=\(mac.fingerprint ?? "?")") }
        emit("found \(mac.name) (fp \(mac.fingerprint ?? "?"), v=\(mac.version ?? "?"))")
        return (mac.endpoint, mac)
    }

    static func pair(_ o: Options, emit: @escaping (String) -> Void) async throws {
        let code = try o.require("code")
        let name = try o.require("name")
        let file = try store(o)
        let timeout = try o.double("timeout", default: 10)
        let (endpoint, mac) = try await resolveEndpoint(o, emit: emit)
        let salt: Data, fp: String, macName: String
        if let mac {
            salt = mac.salt!; fp = mac.fingerprint!; macName = mac.macName
        } else {
            guard let s = NearbyCrypto.data(hex: try o.require("salt")), s.count == NearbyCrypto.saltLength else {
                throw Failure(code: 64, message: "--salt must be the Mac's 32-hex TXT salt")
            }
            salt = s; fp = NearbyCrypto.fingerprint(salt: s); macName = o.string("host")!
        }
        let derived = NearbyCrypto.derive(code: code, salt: salt)
        emit("pairing as \"\(name)\" with pair_id \(derived.pairId) (bootstrap key from code + salt \(NearbyCrypto.hex(salt)))")
        let (pair, session) = try await NearbyClient.pair(endpoint: endpoint, salt: salt, fingerprint: fp, macName: macName,
                                                          code: code, companionName: name, connectTimeout: timeout,
                                                          onLine: lineLogger(o, emit: emit))
        defer { session.close() }
        if let n = session.connection.negotiated { emit("tls: \(n.tlsv12 ? "1.2" : "NOT 1.2") suite 0x\(String(n.suite, radix: 16))") }
        emit("hello_ack from \"\(session.macName)\": received long-term pair_psk (\(pair.psk?.count ?? 0) bytes)")
        emit("destination: \(describe(session.destination))")
        try file.upsert(pair)
        emit("stored pairing for fp \(fp) in \(file.url.path)")
        // The bootstrap connection stays usable; prove it with a query.
        let d = try await session.destinationQuery()
        emit("destination_query on the same connection: \(describe(d))")
        emit("paired")
    }

    static func send(_ o: Options, emit: @escaping (String) -> Void) async throws {
        let file = try store(o)
        let imagePath = try o.require("image")
        let timeout = try o.double("timeout", default: 10)
        guard let image = FileManager.default.contents(atPath: imagePath) else {
            throw Failure(code: 66, message: "cannot read \(imagePath)")
        }
        guard let mime = sniffMime(image) else {
            throw Failure(code: 65, message: "\(imagePath) is not a PNG or JPEG (the Mac accepts image/png and image/jpeg)")
        }
        let pair: PairedMac
        if let key = o.string("mac") {
            guard let p = file.pair(matching: key) else { throw Failure(code: 1, message: "no stored pairing matches \(key); run `nearby-client pair`") }
            pair = p
        } else if file.pairs.count == 1 {
            pair = file.pairs[0]
        } else if file.pairs.isEmpty {
            throw Failure(code: 1, message: "no pairings stored in \(file.url.path); run `nearby-client pair`")
        } else {
            throw Failure(code: 64, message: "several pairings stored; pick one with --mac <name|fp>")
        }
        let (endpoint, _) = try await resolveEndpoint(o, emit: emit, wantFP: pair.fingerprint)
        emit("connecting to \"\(pair.macName)\" as pair_id \(pair.pairId) with the stored pair_psk")
        let session = try await NearbyClient.connect(endpoint: endpoint, pair: pair, connectTimeout: timeout,
                                                     onLine: lineLogger(o, emit: emit))
        defer { session.close() }
        emit("hello_ack from \"\(session.macName)\"; destination: \(describe(session.destination))")
        var dest = session.destination
        if let id = o.string("destination-id") {
            guard let rev = try o.int("base-revision") else { throw Failure(code: 64, message: "--destination-id needs --base-revision") }
            dest = .init(destinationId: id, projectId: dest?.projectId ?? "", path: dest?.path ?? "", baseRevision: rev)
        }
        let captureId = o.string("capture-id") ?? "cli-" + UUID().uuidString.lowercased()
        let capture = try session.makeCapture(captureId: captureId, image: image, mimeType: mime,
                                              instructions: o.string("instructions") ?? "", destination: dest)
        emit("capture_submit \(captureId): \(mime), \(image.count) bytes → destination \(capture.destinationId) @ rev \(capture.baseRevision)")
        let ack = try await session.submitCapture(capture, requestID: o.string("request-id"))
        emit("capture_received \(ack.captureId): durable=\(ack.durable) has_proposal=\(ack.hasProposal) applied=\(ack.applied)")
        if !ack.durable { emit("note: durable=false — the Mac's in-memory inbox took it (no capture bridge attached)") }
        var updated = pair
        updated.lastSeenAt = Date()
        updated.lastDestination = session.destination
        try file.upsert(updated)
        emit("sent")
    }

    static func status(_ o: Options, emit: @escaping (String) -> Void) async throws {
        let file = try store(o)
        emit("store: \(file.url.path)")
        if file.pairs.isEmpty { emit("no pairings stored") }
        let seconds = try o.double("seconds", default: 2)
        let visible = (try? await NearbyBrowser.discover(seconds: seconds)) ?? []
        for p in file.pairs {
            let seen = visible.first { $0.fingerprint == p.fingerprint }
            let iso = ISO8601DateFormatter()
            emit("\(p.macName) fp=\(p.fingerprint) pair_id=\(p.pairId) as \"\(p.companionName)\" paired \(iso.string(from: p.pairedAt))"
                 + (p.lastSeenAt.map { " last seen \(iso.string(from: $0))" } ?? "")
                 + (seen.map { " — visible now as \"\($0.name)\"" } ?? " — not visible"))
            if let d = p.lastDestination { emit("  last destination: \(describe(d))") }
        }
        let unpaired = visible.filter { m in !file.pairs.contains { $0.fingerprint == m.fingerprint } }
        for m in unpaired {
            emit("unpaired: \(m.name) fp=\(m.fingerprint ?? "?") v=\(m.version ?? "?")\(m.unsupportedReason.map { " (\($0))" } ?? "")")
        }
    }

    static func browse(_ o: Options, emit: @escaping (String) -> Void) async throws {
        let seconds = try o.double("seconds", default: 2)
        let results = try await NearbyBrowser.discover(seconds: seconds)
        if results.isEmpty { emit("no \(NearbyWire.serviceType) services within \(Int(seconds))s") }
        for m in results {
            let txt = m.txt.keys.sorted().map { "\($0)=\(m.txt[$0]!)" }.joined(separator: " ")
            emit("\(m.name): \(txt)\(m.unsupportedReason.map { " — unsupported: \($0)" } ?? "")")
        }
    }

    static func forget(_ o: Options, emit: @escaping (String) -> Void) throws {
        let file = try store(o)
        let key = try o.require("mac")
        guard let p = file.pair(matching: key) else { throw Failure(code: 1, message: "no stored pairing matches \(key)") }
        try file.remove(fingerprint: p.fingerprint)
        emit("forgot \(p.macName) (fp \(p.fingerprint)); the Mac still lists it until you Forget there too")
    }

    // MARK: helpers

    static func describe(_ d: NearbyWire.Destination?) -> String {
        guard let d else { return "null (nothing pinned on the Mac — Edit > Pin Insertion Point)" }
        return "\(d.destinationId) (\(d.projectId)/\(d.path) @ rev \(d.baseRevision))"
    }

    public static func sniffMime(_ data: Data) -> String? {
        if data.starts(with: [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) { return "image/png" }
        if data.starts(with: [0xFF, 0xD8, 0xFF]) { return "image/jpeg" }
        return nil
    }
}
