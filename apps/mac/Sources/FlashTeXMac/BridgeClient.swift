import Foundation
import FlashTeXProtocol

/// Transport to the FT-007 capture bridge (`flashtex-bridge --store <dir>`):
/// transfer-v1 JSON Lines on stdin/stdout, one reply per request, correlated by
/// the request `id`. Decoding happens off the main thread; completions are
/// delivered on `queue` (main by default) so callers never block the UI.
///
/// Lines over 12 MiB in either direction are refused: an oversized reply is a
/// protocol violation that terminates the bridge, an oversized request fails
/// locally before anything is written.
final class BridgeClient {
    enum Failure: Error, Equatable {
        /// The bridge answered with an `error` envelope.
        case bridge(TransferV1.ErrorPayload)
        case unexpectedReply(expected: String, actual: String)
        case undecodable(String)
        case requestTooLarge(Int)
        case notRunning
        case exited(Int32)
        case protocolViolation(String)

        var code: String? { if case .bridge(let e) = self { return e.code }; return nil }
        var text: String {
            switch self {
            case .bridge(let e): "\(e.code): \(e.message)"
            case .unexpectedReply(let expected, let actual): "expected \(expected) reply, got \(actual)"
            case .undecodable(let s): "undecodable reply: \(s)"
            case .requestTooLarge(let n): "request line of \(n) bytes exceeds \(TransferV1.maxLineBytes)"
            case .notRunning: "bridge is not running"
            case .exited(let code): "bridge exited (\(code))"
            case .protocolViolation(let s): "protocol violation: \(s)"
            }
        }
    }

    enum Event {
        case stderr(String)
        case protocolViolation(String)
        /// A reply whose id matched no pending request (or a null-id error).
        case unsolicited(id: String?, type: String, code: String?)
        case exited(Int32)
    }

    let executable: URL
    let storeDirectory: URL
    private let process = Process()
    private let stdin = Pipe()
    private let stdout = Pipe()
    private let stderr = Pipe()
    private var splitter = LineSplitter()
    private let queue: DispatchQueue
    private let events: (Event) -> Void
    private let writeLock = NSLock()
    private let stateLock = NSLock()
    private var violated = false
    private var nextID = 1
    private var pending: [String: (expected: String, done: (Result<Data, Failure>) -> Void)] = [:]

    /// `arguments` precede `--store <dir>` so a test double can be
    /// `python3 fake_bridge.py --store <dir>`.
    init(executable: URL, arguments: [String] = [], storeDirectory: URL, enableGrok: Bool = false,
         queue: DispatchQueue = .main, events: @escaping (Event) -> Void = { _ in }) throws {
        self.executable = executable
        self.storeDirectory = storeDirectory
        self.queue = queue
        self.events = events
        process.executableURL = executable
        process.arguments = arguments + ["--store", storeDirectory.path] + (enableGrok ? ["--enable-grok"] : [])
        process.standardInput = stdin
        process.standardOutput = stdout
        process.standardError = stderr
        stdout.fileHandleForReading.readabilityHandler = { [weak self] fh in
            self?.consume(fh.availableData)
        }
        stderr.fileHandleForReading.readabilityHandler = { [weak self] fh in
            let d = fh.availableData
            guard let self, !d.isEmpty else { return }
            let s = String(decoding: d, as: UTF8.self)
            self.queue.async { self.events(.stderr(s)) }
        }
        process.terminationHandler = { [weak self] p in
            guard let self else { return }
            self.stdout.fileHandleForReading.readabilityHandler = nil
            self.stderr.fileHandleForReading.readabilityHandler = nil
            self.consume(self.stdout.fileHandleForReading.readDataToEndOfFile())
            let pendingBytes = self.stateLock.withLock { self.splitter.pendingBytes }
            if pendingBytes > 0 {
                self.queue.async { self.events(.protocolViolation("bridge exited with \(pendingBytes) unterminated trailing bytes")) }
            }
            self.failAll(.exited(p.terminationStatus))
            self.queue.async { self.events(.exited(p.terminationStatus)) }
        }
        try process.run()
    }

    var isRunning: Bool { process.isRunning }

    func terminate() {
        try? stdin.fileHandleForWriting.close()
        if process.isRunning { process.terminate() }
    }

    // MARK: requests

    /// Sends one request and decodes the reply of `request.replyType`. An
    /// `error` envelope becomes `Failure.bridge`; any other type is
    /// `unexpectedReply`. Completion runs on `queue`.
    func send<Req: Encodable, Rep: Codable>(_ request: TransferV1.Request, _ payload: Req, as replyType: Rep.Type,
                                              completion: @escaping (Result<Rep, Failure>) -> Void) {
        let id: String = stateLock.withLock { defer { nextID += 1 }; return "bridge-\(nextID)" }
        let line: Data
        do {
            let enc = JSONEncoder()
            enc.outputFormatting = [.withoutEscapingSlashes]
            var data = try enc.encode(RequestEnvelope(id: id, type: request.rawValue, payload: payload))
            data.append(0x0A)
            line = data
        } catch {
            queue.async { completion(.failure(.undecodable("encoding failed: \(error)"))) }
            return
        }
        guard line.count <= TransferV1.maxLineBytes else {
            queue.async { completion(.failure(.requestTooLarge(line.count))) }
            return
        }
        guard process.isRunning else {
            queue.async { completion(.failure(.notRunning)) }
            return
        }
        stateLock.withLock {
            pending[id] = (request.replyType, { result in
                switch result {
                case .failure(let f): completion(.failure(f))
                case .success(let data):
                    do {
                        let env = try JSONDecoder().decode(RuntimeV1.Envelope<Rep>.self, from: data)
                        completion(.success(env.payload))
                    } catch {
                        completion(.failure(.undecodable("\(request.replyType): \(error)")))
                    }
                }
            })
        }
        writeLock.lock(); defer { writeLock.unlock() }
        do {
            try stdin.fileHandleForWriting.write(contentsOf: line)
        } catch {
            let entry = stateLock.withLock { pending.removeValue(forKey: id) }
            queue.async { entry?.done(.failure(.notRunning)) }
        }
    }

    /// async/await form of `send`.
    func request<Req: Encodable, Rep: Codable>(_ request: TransferV1.Request, _ payload: Req, as replyType: Rep.Type = Rep.self) async throws -> Rep {
        try await withCheckedThrowingContinuation { cont in
            send(request, payload, as: replyType) { cont.resume(with: $0) }
        }
    }

    // MARK: discovery

    /// `$FLASHTEX_BRIDGE`, a `flashtex-bridge` next to the executable in a
    /// bundle, then `crates/bridge/target/{release,debug}/flashtex-bridge`.
    @MainActor static func locateBridge() -> URL? {
        let fm = FileManager.default
        if let env = ProcessInfo.processInfo.environment["FLASHTEX_BRIDGE"], fm.isExecutableFile(atPath: env) {
            return URL(fileURLWithPath: env)
        }
        if let bundled = Bundle.main.executableURL?.deletingLastPathComponent().appendingPathComponent("flashtex-bridge"),
           fm.isExecutableFile(atPath: bundled.path) {
            return bundled
        }
        guard let root = ShellModel.locateRepoRoot() else { return nil }
        for profile in ["release", "debug"] {
            let url = root.appendingPathComponent("crates/bridge/target/\(profile)/flashtex-bridge")
            if fm.isExecutableFile(atPath: url.path) { return url }
        }
        return nil
    }

    /// `$FLASHTEX_BRIDGE_STORE` or `~/Library/Application Support/FlashTeX/captures`.
    static func defaultStoreDirectory() -> URL {
        if let env = ProcessInfo.processInfo.environment["FLASHTEX_BRIDGE_STORE"], !env.isEmpty {
            return URL(fileURLWithPath: env)
        }
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Library/Application Support")
        return base.appendingPathComponent("FlashTeX/captures")
    }

    // MARK: reading

    private struct LenientHeader: Decodable {
        var protocolVersion: Int
        var id: String?
        var type: String
        enum CodingKeys: String, CodingKey { case protocolVersion = "protocol_version", id, type }
    }

    private func consume(_ data: Data) {
        guard !data.isEmpty else { return }
        let (lines, pendingBytes, alreadyViolated) = stateLock.withLock {
            (splitter.append(data), splitter.pendingBytes, violated)
        }
        guard !alreadyViolated else { return }
        if let big = lines.first(where: { $0.count + 1 > TransferV1.maxLineBytes }) {
            violate("line of \(big.count) bytes exceeds the \(TransferV1.maxLineBytes)-byte limit")
            return
        }
        if pendingBytes >= TransferV1.maxLineBytes {
            violate("unterminated line exceeds the \(TransferV1.maxLineBytes)-byte limit")
            return
        }
        for line in lines where !line.isEmpty { deliver(line) }
    }

    private func deliver(_ line: Data) {
        let header: LenientHeader
        do {
            header = try JSONDecoder().decode(LenientHeader.self, from: line)
        } catch {
            queue.async { self.events(.protocolViolation("undecodable line: \(error)")) }
            return
        }
        guard header.protocolVersion == RuntimeV1.protocolVersion else {
            violate("unsupported protocol_version \(header.protocolVersion)")
            return
        }
        var errorPayload: TransferV1.ErrorPayload?
        if header.type == "error" {
            errorPayload = try? JSONDecoder().decode(RuntimeV1.Envelope<TransferV1.ErrorPayload>.self, from: line).payload
        }
        guard let id = header.id, let entry = stateLock.withLock({ pending.removeValue(forKey: id) }) else {
            queue.async { self.events(.unsolicited(id: header.id, type: header.type, code: errorPayload?.code)) }
            return
        }
        let result: Result<Data, Failure>
        if header.type == "error" {
            result = .failure(.bridge(errorPayload ?? .init(code: "undecodable_error", message: String(decoding: line, as: UTF8.self))))
        } else if header.type != entry.expected {
            result = .failure(.unexpectedReply(expected: entry.expected, actual: header.type))
        } else {
            result = .success(line)
        }
        queue.async { entry.done(result) }
    }

    private func violate(_ message: String) {
        stateLock.withLock { violated = true; splitter = LineSplitter() }
        queue.async { self.events(.protocolViolation(message)) }
        failAll(.protocolViolation(message))
        terminate()
    }

    private func failAll(_ failure: Failure) {
        let entries = stateLock.withLock { defer { pending.removeAll() }; return Array(pending.values) }
        for entry in entries { queue.async { entry.done(.failure(failure)) } }
    }
}

/// Outgoing envelope; `RuntimeV1.Envelope` requires a Codable payload.
private struct RequestEnvelope<P: Encodable>: Encodable {
    var protocolVersion = RuntimeV1.protocolVersion
    var id: String
    var type: String
    var payload: P
    enum CodingKeys: String, CodingKey { case protocolVersion = "protocol_version", id, type, payload }
}
