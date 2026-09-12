import Foundation
import FlashTeXProtocol

/// Client for `flashtex-preview-controller` (crates/preview-controller,
/// "Native helper protocol v1", STDIO.md): one durable-source helper per
/// project that owns the edit ledger, the lexical index and the original
/// compiler behind it. Frames are JSON Lines with `protocol_version:1`, the
/// configured `session_id`, a request `id` (null on asynchronous `update`
/// frames), `type` and `payload`.
///
/// Threading: stdin writes are serialized under a lock and never wait for a
/// reply; stdout is drained by the pipe's readability handler, decoded there,
/// and delivered to the main run loop with an explicit wake-up (same path as
/// `WorkerClient`, so a preview update is applied on the next iteration, not
/// the next dispatch-queue drain). Nothing here blocks the UI thread.
final class PreviewControllerClient {
    /// A decoded frame. `update` carries the helper's asynchronous events
    /// (`kind: preview | stale | discarded | …`); `result`/`error` correlate
    /// to a request id.
    enum Event {
        case ready(compilerError: String?, compilerMaxFrameBytes: Int, helperMaxOutputBytes: Int)
        case result(id: String, payload: JSONObject)
        case error(id: String?, message: String)
        case preview(PreviewUpdate)
        case update(kind: String, payload: JSONObject)
        case protocolViolation(String)
        case stderr(String)
        case exited(Int32)
    }

    /// `type:update, kind:preview`: a compile result for exact source versions.
    struct PreviewUpdate {
        var requestID: String
        var compileRevision: Int
        /// Durable revision per document path the preview was compiled from.
        var sourceVersions: [String: Int]
        var missingLayoutCapabilities: [String]
        var controllerTotalMs: Double?
        var runtimeTotalMs: Double?
        var result: RuntimeV1.Envelope<RuntimeV1.CompileResult>
    }

    typealias JSONObject = [String: Any]

    /// Startup configuration written to the JSON file the helper is launched with.
    struct Config {
        var sessionID: String
        var projectID: String
        var entryPath: String
        /// File-backed mode: the rooted project directory and an application
        /// owned private ledger root (mutually exclusive with `storePaths`).
        var projectRoot: URL?
        var privateLedgerRoot: URL?
        /// Store-backed mode: initialized ledger directories.
        var storePaths: [URL] = []
        var compilerPath: URL?
        var compilerMaxFrameBytes: Int?

        func json() -> JSONObject {
            var o: JSONObject = ["session_id": sessionID, "project_id": projectID, "entry_path": entryPath]
            if let projectRoot { o["project_root"] = projectRoot.path }
            if let privateLedgerRoot { o["private_ledger_root"] = privateLedgerRoot.path }
            if !storePaths.isEmpty { o["store_paths"] = storePaths.map(\.path) }
            if let compilerPath { o["compiler_path"] = compilerPath.path }
            if let compilerMaxFrameBytes { o["compiler_max_frame_bytes"] = compilerMaxFrameBytes }
            return o
        }
    }

    /// Helper output frames are bounded at 16 MiB each (STDIO.md).
    static let maxFrameBytes = 16 * 1024 * 1024

    let executable: URL
    let config: Config
    let configURL: URL
    private let process = Process()
    private let stdin = Pipe()
    private let stdout = Pipe()
    private let stderr = Pipe()
    private var splitter = LineSplitter()
    private let handler: (Event) -> Void
    private let writeLock = NSLock()
    private let stateLock = NSLock()
    private var violated = false
    private var nextID = 1

    init(executable: URL, config: Config, handler: @escaping (Event) -> Void) throws {
        self.executable = executable
        self.config = config
        self.handler = handler
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("flashtex-controller-\(config.sessionID)")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        configURL = dir.appendingPathComponent("config.json")
        try JSONSerialization.data(withJSONObject: config.json(), options: [.sortedKeys]).write(to: configURL)
        process.executableURL = executable
        process.arguments = [configURL.path]
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
            self.deliver { self.handler(.stderr(s)) }
        }
        process.terminationHandler = { [weak self] p in
            guard let self else { return }
            self.stdout.fileHandleForReading.readabilityHandler = nil
            self.stderr.fileHandleForReading.readabilityHandler = nil
            self.consume(self.stdout.fileHandleForReading.readDataToEndOfFile())
            let pending = self.stateLock.withLock { self.splitter.pendingBytes }
            if pending > 0 {
                self.deliver { self.handler(.protocolViolation("helper exited with \(pending) unterminated trailing bytes")) }
            }
            self.deliver { self.handler(.exited(p.terminationStatus)) }
        }
        try process.run()
    }

    var isRunning: Bool { process.isRunning }
    var processIdentifier: Int32 { process.processIdentifier }

    // MARK: requests

    /// Sends one operation; returns its request id. Never waits for the reply.
    @discardableResult
    func send(_ type: String, _ payload: JSONObject, id explicitID: String? = nil) throws -> String {
        let id = explicitID ?? stateLock.withLock { defer { nextID += 1 }; return "pc-\(nextID)" }
        let frame: JSONObject = ["protocol_version": 1, "session_id": config.sessionID, "id": id, "type": type, "payload": payload]
        var line = try JSONSerialization.data(withJSONObject: frame, options: [.sortedKeys, .withoutEscapingSlashes])
        line.append(0x0A)
        writeLock.lock(); defer { writeLock.unlock() }
        try stdin.fileHandleForWriting.write(contentsOf: line)
        return id
    }

    func document(path: String) throws -> String { try send("document", ["path": path]) }

    func edit(path: String, expectedRevision: Int, expectedSHA256: String, text: String) throws -> String {
        try send("edit", ["path": path, "expected_revision": expectedRevision, "expected_sha256": expectedSHA256, "text": text])
    }

    func compile() throws -> String { try send("compile", [:]) }
    func restart() throws -> String { try send("restart", [:]) }
    func snapshot() throws -> String { try send("snapshot", [:]) }

    func configureLayout(capabilities: [String]) throws -> String {
        try send("configure_layout", ["layout_capabilities": capabilities, "renderer_support_confirmed": true])
    }

    func close() {
        _ = try? send("close", [:])
        try? stdin.fileHandleForWriting.close()
    }

    func terminate() {
        try? stdin.fileHandleForWriting.close()
        if process.isRunning { process.terminate() }
    }

    // MARK: decoding

    private func consume(_ data: Data) {
        guard !data.isEmpty else { return }
        let (lines, pending, alreadyViolated) = stateLock.withLock {
            (splitter.append(data), splitter.pendingBytes, violated)
        }
        guard !alreadyViolated else { return }
        if let big = lines.first(where: { $0.count > Self.maxFrameBytes }) {
            violate("frame of \(big.count) bytes exceeds the \(Self.maxFrameBytes)-byte limit")
            return
        }
        if pending > Self.maxFrameBytes {
            violate("unterminated frame exceeds the \(Self.maxFrameBytes)-byte limit")
            return
        }
        for line in lines where !line.isEmpty {
            let event = Self.decode(line, sessionID: config.sessionID)
            deliver { self.handler(event) }
        }
    }

    private func violate(_ message: String) {
        stateLock.withLock { violated = true; splitter = LineSplitter() }
        deliver { self.handler(.protocolViolation(message)) }
        terminate()
    }

    /// Main run-loop delivery with an explicit wake-up (see `WorkerClient.deliver`).
    private func deliver(_ block: @escaping @Sendable () -> Void) {
        CFRunLoopPerformBlock(CFRunLoopGetMain(), CFRunLoopMode.commonModes.rawValue, block)
        CFRunLoopWakeUp(CFRunLoopGetMain())
    }

    static func decode(_ line: Data, sessionID: String) -> Event {
        guard let obj = (try? JSONSerialization.jsonObject(with: line)) as? JSONObject else {
            return .protocolViolation("frame is not a JSON object")
        }
        guard (obj["protocol_version"] as? Int) == 1 else {
            return .protocolViolation("unsupported protocol_version \(obj["protocol_version"] ?? "missing")")
        }
        guard (obj["session_id"] as? String) == sessionID else {
            return .protocolViolation("frame for session \(obj["session_id"] ?? "missing"), expected \(sessionID)")
        }
        let type = obj["type"] as? String ?? ""
        let id = obj["id"] as? String
        let payload = obj["payload"] as? JSONObject ?? [:]
        switch type {
        case "ready":
            return .ready(compilerError: payload["compiler_error"] as? String,
                          compilerMaxFrameBytes: payload["compiler_max_frame_bytes"] as? Int ?? 0,
                          helperMaxOutputBytes: payload["helper_max_output_bytes"] as? Int ?? 0)
        case "result":
            guard let id else { return .protocolViolation("result without id") }
            return .result(id: id, payload: payload)
        case "error":
            return .error(id: id, message: payload["message"] as? String ?? "unspecified helper error")
        case "update":
            let kind = payload["kind"] as? String ?? ""
            guard kind == "preview" else { return .update(kind: kind, payload: payload) }
            do {
                guard let resultObject = payload["result"] else { return .protocolViolation("preview update without result") }
                let resultData = try JSONSerialization.data(withJSONObject: resultObject)
                let env = try RuntimeV1.decodeCompileResult(resultData)
                let versions = (payload["source_versions"] as? [String: Any] ?? [:]).compactMapValues { $0 as? Int }
                return .preview(PreviewUpdate(
                    requestID: payload["request_id"] as? String ?? "",
                    compileRevision: payload["compile_revision"] as? Int ?? 0,
                    sourceVersions: versions,
                    missingLayoutCapabilities: payload["missing_layout_capabilities"] as? [String] ?? [],
                    controllerTotalMs: payload["controller_total_ms"] as? Double,
                    runtimeTotalMs: payload["runtime_total_ms"] as? Double,
                    result: env))
            } catch {
                return .protocolViolation("preview update: \(error)")
            }
        default:
            return .protocolViolation("unknown frame type \(type)")
        }
    }
}
