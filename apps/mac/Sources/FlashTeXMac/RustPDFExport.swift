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
        guard let root = ShellModel.locateRepoRoot() else { return nil }
        for profile in ["release", "debug"] {
            let url = root.appendingPathComponent("crates/pdf/target/\(profile)/flashtex-pdf")
            if fm.isExecutableFile(atPath: url.path) { return url }
        }
        return nil
    }

    /// Runs the writer synchronously (it is fast); returns its stderr (warnings).
    @discardableResult
    static func export(_ result: RuntimeV1.CompileResult, id: String, writer: URL, to output: URL) throws -> String {
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
        try process.run()
        try stdin.fileHandleForWriting.write(contentsOf: input)
        try stdin.fileHandleForWriting.close()
        process.waitUntilExit()
        let err = String(decoding: stderr.fileHandleForReading.readDataToEndOfFile(), as: UTF8.self)
        let out = String(decoding: stdout.fileHandleForReading.readDataToEndOfFile(), as: UTF8.self)
        guard process.terminationStatus == 0 else {
            throw Failure(description: "flashtex-pdf exited \(process.terminationStatus): \(err.isEmpty ? out : err)")
        }
        return (err + out).trimmingCharacters(in: .whitespacesAndNewlines)
    }
}

extension ShellModel {
    var rustPDFWriterAvailable: Bool { RustPDFExport.locateWriter() != nil }

    /// `File > Export PDF via Rust Writer…`
    func exportPDFViaRust() {
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
        do {
            let notes = try RustPDFExport.export(result, id: resultID ?? "mac-export", writer: writer, to: url)
            captureNote = "Rust writer exported \(url.lastPathComponent)" + (notes.isEmpty ? "" : " — \(notes)")
        } catch {
            captureNote = "Rust PDF export failed: \(error)"
        }
    }
}
