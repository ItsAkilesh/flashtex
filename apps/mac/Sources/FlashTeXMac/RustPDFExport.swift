import AppKit
import FlashTeXProtocol

/// Export through the original Rust writer (crates/pdf, FT-009): the current
/// `compile_result` is re-encoded as a runtime-v1 envelope and piped to
/// `flashtex-pdf --out <file> --verify`. Always white; independent of dark preview.
@MainActor
enum RustPDFExport {
    struct Failure: Error, CustomStringConvertible { let description: String }

    /// $FLASHTEX_PDF, then crates/pdf/target/{release,debug}/flashtex-pdf under the repo.
    static func locateWriter() -> URL? {
        let fm = FileManager.default
        if let env = ProcessInfo.processInfo.environment["FLASHTEX_PDF"], fm.isExecutableFile(atPath: env) {
            return URL(fileURLWithPath: env)
        }
        if let bundled = Bundle.main.executableURL?.deletingLastPathComponent().appendingPathComponent("flashtex-pdf"),
           fm.isExecutableFile(atPath: bundled.path) {
            return bundled
        }
        guard let root = ShellModel.locateRepoRoot() else { return nil }
        for profile in ["release", "debug"] {
            let url = root.appendingPathComponent("crates/pdf/target/\(profile)/flashtex-pdf")
            if fm.isExecutableFile(atPath: url.path) { return url }
        }
        return nil
    }

    /// Runs the writer off the main actor with both output pipes drained
    /// concurrently (bounded), so a chatty writer can never fill a pipe and
    /// deadlock against the parent. Returns the writer's notes (stderr+stdout).
    nonisolated static func export(_ result: RuntimeV1.CompileResult, id: String, writer: URL, to output: URL,
                                   timeout: TimeInterval = 30, maxCapturedBytes: Int = 4 * 1024 * 1024) throws -> String {
        let envelope = RuntimeV1.Envelope(protocolVersion: RuntimeV1.protocolVersion, id: id,
                                          type: "compile_result", payload: result)
        let input = try RuntimeV1.encodeLine(envelope)
        let process = Process()
        process.executableURL = writer
        process.arguments = ["--out", output.path, "--verify"]
        let stdin = Pipe(), stderr = Pipe(), stdout = Pipe()
        process.standardInput = stdin
        process.standardError = stderr
        process.standardOutput = stdout

        // Drain both pipes on background threads from the moment the child
        // starts; keep at most `maxCapturedBytes` per stream, discard the rest.
        let group = DispatchGroup()
        let lock = NSLock()
        var captured: [Int32: Data] = [:]
        var truncated: Set<Int32> = []
        func drain(_ handle: FileHandle, tag: Int32) {
            group.enter()
            DispatchQueue.global(qos: .userInitiated).async {
                defer { group.leave() }
                var buffer = Data()
                while true {
                    let chunk = handle.availableData
                    if chunk.isEmpty { break }
                    lock.lock()
                    if buffer.count < maxCapturedBytes {
                        buffer.append(chunk.prefix(maxCapturedBytes - buffer.count))
                        if buffer.count >= maxCapturedBytes { truncated.insert(tag) }
                    } else { truncated.insert(tag) }
                    lock.unlock()
                }
                lock.lock(); captured[tag] = buffer; lock.unlock()
            }
        }
        try process.run()
        drain(stdout.fileHandleForReading, tag: 1)
        drain(stderr.fileHandleForReading, tag: 2)
        // Write the request on its own thread too: a writer that emits before
        // reading could otherwise block us on a full stdin pipe.
        var writeError: Error?
        group.enter()
        DispatchQueue.global(qos: .userInitiated).async {
            defer { group.leave() }
            do {
                try stdin.fileHandleForWriting.write(contentsOf: input)
                try stdin.fileHandleForWriting.close()
            } catch { lock.lock(); writeError = error; lock.unlock() }
        }
        let deadline = DispatchTime.now() + timeout
        if group.wait(timeout: deadline) == .timedOut || !waitForExit(process, until: deadline) {
            process.terminate()
            _ = group.wait(timeout: .now() + 2)
            throw Failure(description: "flashtex-pdf did not finish within \(Int(timeout)) s; terminated")
        }
        lock.lock()
        let err = String(decoding: captured[2] ?? Data(), as: UTF8.self)
        let out = String(decoding: captured[1] ?? Data(), as: UTF8.self)
        let cut = truncated
        let werr = writeError
        lock.unlock()
        if let werr, process.terminationStatus == 0 {
            throw Failure(description: "could not send the document to flashtex-pdf: \(werr.localizedDescription)")
        }
        let note = cut.isEmpty ? "" : " (writer output truncated to \(maxCapturedBytes) bytes)"
        guard process.terminationStatus == 0 else {
            throw Failure(description: "flashtex-pdf exited \(process.terminationStatus): \((err.isEmpty ? out : err).prefix(2000))\(note)")
        }
        return (err + out).trimmingCharacters(in: .whitespacesAndNewlines) + note
    }

    /// Polls for exit until the deadline without blocking the caller's queue forever.
    nonisolated private static func waitForExit(_ process: Process, until deadline: DispatchTime) -> Bool {
        while process.isRunning {
            if DispatchTime.now() >= deadline { return false }
            Thread.sleep(forTimeInterval: 0.01)
        }
        return true
    }
}

extension ShellModel {
    var rustPDFWriterAvailable: Bool { RustPDFExport.locateWriter() != nil }

    /// `File > Export PDF via Rust Writer…`
    func exportPDFViaRust() {
        if let why = historicalRefusal(of: "export") { captureNote = why; return }
        guard let result else { captureNote = "Nothing to export: no compile result loaded."; return }
        guard let writer = RustPDFExport.locateWriter() else {
            captureNote = "No built flashtex-pdf found (cargo build --release in crates/pdf, or set FLASHTEX_PDF)."
            return
        }
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.pdf]
        panel.nameFieldStringValue = "\(result.projectId)-r\(result.revision)-rust.pdf"
        panel.message = "Export via the original Rust PDF writer (crates/pdf); always white"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        let id = resultID ?? "mac-export"
        captureNote = "Exporting \(url.lastPathComponent) via flashtex-pdf…"
        Task.detached(priority: .userInitiated) {
            let outcome: Result<String, Error> = Result { try RustPDFExport.export(result, id: id, writer: writer, to: url) }
            await MainActor.run {
                switch outcome {
                case .success(let notes): self.captureNote = "Rust writer exported \(url.lastPathComponent)" + (notes.isEmpty ? "" : " — \(notes)")
                case .failure(let error): self.captureNote = "Rust PDF export failed: \(error)"
                }
            }
        }
    }
}
