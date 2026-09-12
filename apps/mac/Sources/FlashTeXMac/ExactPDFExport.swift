import AppKit
import Foundation

/// `File > Export PDF (exact, v2)…`: the loaded rendering-v2 display list is
/// handed to `flashtex-pdf-exact from-v2` (crates/pdf), which embeds the
/// exact font programs (GID-preserving CFF subsets), places every glyph by
/// original GID at its exact tick position, writes typed rules and ToUnicode,
/// and refuses anything it cannot express (alpha, images, missing fonts,
/// non-integer ticks) naming the item — never a silent approximation.
///
/// Runs off the main thread with the same drained pipes and timeout as the
/// runtime-v1 export; the result is reported in `captureNote`.
@MainActor
enum ExactPDFExport {
    /// $FLASHTEX_PDF_EXACT, the app bundle, then crates/pdf/target/{release,debug}.
    static func locateTool() -> URL? {
        let fm = FileManager.default
        if let env = ProcessInfo.processInfo.environment["FLASHTEX_PDF_EXACT"], fm.isExecutableFile(atPath: env) {
            return URL(fileURLWithPath: env)
        }
        if let bundled = Bundle.main.executableURL?.deletingLastPathComponent().appendingPathComponent("flashtex-pdf-exact"),
           fm.isExecutableFile(atPath: bundled.path) {
            return bundled
        }
        guard let root = ShellModel.locateRepoRoot() else { return nil }
        for profile in ["release", "debug"] {
            let url = root.appendingPathComponent("crates/pdf/target/\(profile)/flashtex-pdf-exact")
            if fm.isExecutableFile(atPath: url.path) { return url }
        }
        return nil
    }

    /// Font directories handed to the tool in addition to its own defaults:
    /// the display list's producer directories are unknown here, so the
    /// bundled/vendored Latin Modern and `FLASHTEX_LM_DIR` are offered.
    static func fontDirectories() -> [String] {
        var dirs: [String] = []
        if let env = ProcessInfo.processInfo.environment["FLASHTEX_FONT_DIRS"] {
            dirs += env.split(separator: ":").map(String.init)
        }
        dirs += PreviewFonts.latinModernSearchPaths
        return dirs.filter { FileManager.default.fileExists(atPath: $0) }
    }

    struct Outcome: Equatable {
        var exitCode: Int32
        var stdout: String
        var stderr: String
        var bytes: Int
        var succeeded: Bool { exitCode == 0 && bytes > 0 }
    }

    /// Runs `from-v2 LIST --out PDF [--font-dir …]`, draining both pipes;
    /// kills the tool after `timeout` seconds.
    nonisolated static func run(tool: URL, list: URL, out: URL, fontDirs: [String], timeout: TimeInterval = 60) throws -> Outcome {
        let p = Process()
        p.executableURL = tool
        var args = ["from-v2", list.path, "--out", out.path]
        for d in fontDirs { args += ["--font-dir", d] }
        p.arguments = args
        let stdout = Pipe(), stderr = Pipe()
        p.standardOutput = stdout; p.standardError = stderr
        var outData = Data(), errData = Data()
        let group = DispatchGroup()
        group.enter(); DispatchQueue.global().async { outData = stdout.fileHandleForReading.readDataToEndOfFile(); group.leave() }
        group.enter(); DispatchQueue.global().async { errData = stderr.fileHandleForReading.readDataToEndOfFile(); group.leave() }
        try p.run()
        let deadline = DispatchTime.now() + timeout
        let waiter = DispatchGroup()
        waiter.enter(); DispatchQueue.global().async { p.waitUntilExit(); waiter.leave() }
        if waiter.wait(timeout: deadline) == .timedOut {
            p.terminate()
            _ = waiter.wait(timeout: .now() + 5)
        }
        group.wait()
        let bytes = (try? FileManager.default.attributesOfItem(atPath: out.path)[.size] as? Int) ?? 0
        return Outcome(exitCode: p.terminationStatus, stdout: String(decoding: outData, as: UTF8.self),
                       stderr: String(decoding: errData, as: UTF8.self), bytes: bytes)
    }
}

extension ShellModel {
    /// Export the loaded v2 display list through the exact route. The list's
    /// source JSON (already verified by the pane) is handed to the tool as is.
    func exportPDFExact() {
        guard case .loaded(let frame, let listURL)? = displayListV2 else {
            captureNote = displayListV2?.isLoading == true ? "Nothing to export yet: a display list is still loading." : "Nothing to export: no v2 display list loaded (File > Open Display List (v2)…)."
            return
        }
        guard let tool = ExactPDFExport.locateTool() else {
            captureNote = "No flashtex-pdf-exact found (build crates/pdf or set FLASHTEX_PDF_EXACT); exact export unavailable."
            return
        }
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.pdf]
        panel.nameFieldStringValue = "\(frame.list.projectId)-r\(frame.list.revision)-exact.pdf"
        panel.message = "Export the v2 display list through flashtex-pdf-exact from-v2 (exact glyphs by original GID, embedded font programs)"
        guard panel.runModal() == .OK, let out = panel.url else { return }
        exportPDFExact(listURL: listURL, tool: tool, to: out)
    }

    /// Non-interactive core (tests, automation). Reports on the main actor.
    func exportPDFExact(listURL: URL, tool: URL, to out: URL, completion: (@MainActor (ExactPDFExport.Outcome?) -> Void)? = nil) {
        captureNote = "Exporting exact PDF…"
        let fontDirs = ExactPDFExport.fontDirectories()
        Task.detached {
            let result: Result<ExactPDFExport.Outcome, Error> = Result { try ExactPDFExport.run(tool: tool, list: listURL, out: out, fontDirs: fontDirs) }
            await MainActor.run {
                switch result {
                case .success(let o) where o.succeeded:
                    self.captureNote = "Exported exact PDF (\(o.bytes) bytes) to \(out.path)"
                    completion?(o)
                case .success(let o):
                    let why = (o.stderr + o.stdout).trimmingCharacters(in: .whitespacesAndNewlines)
                    self.captureNote = "Exact export refused (exit \(o.exitCode)): \(why.isEmpty ? "no message" : why)"
                    completion?(o)
                case .failure(let e):
                    self.captureNote = "Exact export failed to run: \(e.localizedDescription)"
                    completion?(nil)
                }
            }
        }
    }
}
