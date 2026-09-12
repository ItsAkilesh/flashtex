import Foundation
import FlashTeXProtocol

/// Talks to a Rust worker process over runtime v1 JSON Lines (stdin/stdout).
/// Decoding happens off the main thread; callbacks are delivered on `queue`.
final class WorkerClient {
    enum Event {
        case result(RuntimeV1.Envelope<RuntimeV1.CompileResult>)
        case error(id: String, message: String)
        case protocolViolation(String)
        case stderr(String)
        case exited(Int32)
    }

    let executable: URL
    private let process = Process()
    private let stdin = Pipe()
    private let stdout = Pipe()
    private let stderr = Pipe()
    private var splitter = LineSplitter()
    private let queue: DispatchQueue
    private let handler: (Event) -> Void
    private let lock = NSLock()

    init(executable: URL, arguments: [String] = [], queue: DispatchQueue = .main,
         handler: @escaping (Event) -> Void) throws {
        self.executable = executable
        self.queue = queue
        self.handler = handler
        process.executableURL = executable
        process.arguments = arguments
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
            self.queue.async { self.handler(.stderr(s)) }
        }
        process.terminationHandler = { [weak self] p in
            guard let self else { return }
            self.stdout.fileHandleForReading.readabilityHandler = nil
            self.stderr.fileHandleForReading.readabilityHandler = nil
            self.queue.async { self.handler(.exited(p.terminationStatus)) }
        }
        try process.run()
    }

    var isRunning: Bool { process.isRunning }

    func send(_ request: RuntimeV1.CompileRequest, id: String) throws {
        let line = try RuntimeV1.encodeLine(RuntimeV1.compileEnvelope(id: id, request))
        lock.lock(); defer { lock.unlock() }
        try stdin.fileHandleForWriting.write(contentsOf: line)
    }

    func terminate() {
        try? stdin.fileHandleForWriting.close()
        if process.isRunning { process.terminate() }
    }

    private func consume(_ data: Data) {
        guard !data.isEmpty else { return }
        let lines = splitter.append(data)
        if splitter.pendingBytes > RuntimeV1.maxLineBytes {
            queue.async { self.handler(.protocolViolation("line exceeds \(RuntimeV1.maxLineBytes) bytes")) }
            terminate()
            return
        }
        for line in lines where !line.isEmpty {
            let event = Self.decode(line)
            queue.async { self.handler(event) }
        }
    }

    static func decode(_ line: Data) -> Event {
        do {
            let header = try RuntimeV1.header(of: line)
            guard header.protocolVersion == RuntimeV1.protocolVersion else {
                return .protocolViolation("unsupported protocol_version \(header.protocolVersion)")
            }
            switch header.type {
            case "compile_result":
                return .result(try RuntimeV1.decodeCompileResult(line))
            case "error":
                let env = try JSONDecoder().decode(RuntimeV1.Envelope<RuntimeV1.ErrorPayload>.self, from: line)
                return .error(id: env.id, message: env.payload.message)
            default:
                return .protocolViolation("unexpected message type \(header.type)")
            }
        } catch {
            return .protocolViolation("undecodable line: \(error)")
        }
    }
}
