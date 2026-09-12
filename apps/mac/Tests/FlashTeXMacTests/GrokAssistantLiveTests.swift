import Foundation
import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// ONE live Ask Grok call on a small real document, opt-in only:
/// `FLASHTEX_GROK_LIVE=1` plus a resolvable key (Keychain service
/// `tech.jay3332.flashtex.xai` or `XAI_API_KEY`), `FLASHTEX_COMPILER` (the real
/// compile is the binding), `FLASHTEX_ASSISTANT_CONTEXT` (offline helper) and
/// `FLASHTEX_ASSISTANT_CONTEXT_GROK` (the `--features grok` build). Writes
/// timing, model, edit/relocation counts and notes — never the key or the
/// prompt body — to `$FLASHTEX_EVIDENCE_DIR/grok-assistant-<UTC>/live.json`
/// and `README.md`. Skips cleanly otherwise. The paid call is bounded by the
/// provider timeout; Apply afterwards is offline (helper `approve`).
@MainActor
final class GrokAssistantLiveTests: XCTestCase {
    static let document = """
    \\documentclass{article}
    \\begin{document}
    The identity
    $$ (a+b)^2 = a^2 + 2ab + b^2 $$
    holds for all real numbers.
    \\end{document}

    """

    func testLiveAskConvertsTheDisplayedEquationAndAppliesOneEdit() async throws {
        let env = ProcessInfo.processInfo.environment
        guard env["FLASHTEX_GROK_LIVE"] == "1" else { throw XCTSkip("set FLASHTEX_GROK_LIVE=1 for the one live Ask Grok call") }
        guard let compiler = env["FLASHTEX_COMPILER"].map(URL.init(fileURLWithPath:)), FileManager.default.isExecutableFile(atPath: compiler.path) else {
            throw XCTSkip("set FLASHTEX_COMPILER")
        }
        guard let evidenceRoot = env["FLASHTEX_EVIDENCE_DIR"] else { throw XCTSkip("set FLASHTEX_EVIDENCE_DIR") }
        let configuration = GrokAssistant.configuration(env)
        guard let grok = configuration.grok, grok.credential != nil else { throw XCTSkip("no xAI key resolved (Keychain tech.jay3332.flashtex.xai or XAI_API_KEY)") }
        guard configuration.helper != nil, env["FLASHTEX_ASSISTANT_CONTEXT_GROK"] != nil else { throw XCTSkip("set FLASHTEX_ASSISTANT_CONTEXT and FLASHTEX_ASSISTANT_CONTEXT_GROK") }

        let stamp = ISO8601DateFormatter().string(from: Date()).replacingOccurrences(of: "[-:]", with: "", options: .regularExpression)
        let dir = URL(fileURLWithPath: evidenceRoot).appendingPathComponent("grok-assistant-\(stamp)")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)

        let model = ShellModel()
        model.autoCompile = false
        model.replaceProject(entryText: Self.document)
        model.attachWorker(at: compiler)
        model.compile()
        try await waitUntil("real compile") { model.inFlightRevision == nil && model.result != nil }
        let result = try XCTUnwrap(model.result)
        XCTAssertEqual(model.compiledDocuments["main.tex"], Self.document)

        let a = model.grokAssistant
        a.injected = configuration
        a.instruction = "convert this displayed equation to an align* environment"
        let ns = NSRange(try XCTUnwrap(Self.document.range(of: "$$ (a+b)^2 = a^2 + 2ab + b^2 $$")), in: Self.document)
        model.caretUTF16 = ns.location
        model.caretLengthUTF16 = ns.length
        let t0 = Date()
        model.askGrokNow()
        try await waitUntil("live reply", timeout: configuration.providerTimeout + 30) { !a.inFlight }
        let elapsed = Date().timeIntervalSince(t0)
        var record: [String: Any] = [
            "utc": ISO8601DateFormatter().string(from: t0), "model": grok.model, "credential": grok.credential?.description ?? "?",
            "provider_timeout_s": configuration.providerTimeout, "elapsed_s": elapsed,
            "compile": ["request_id": model.resultID ?? "?", "revision": result.revision, "status": result.status.rawValue, "diagnostics": result.diagnostics.count],
            "request": ["related_paths": a.lastRequest?.relatedPaths ?? [], "selected_diagnostics": a.lastRequest?.selectedDiagnostics ?? [],
                        "instruction_bytes": a.lastRequest?.userInstruction.utf8.count ?? 0, "context_line": a.contextLine],
            "stages": a.childLaunches.map { "\($0.stage)" },
            "coverage_note": a.coverageNote ?? NSNull(),
            "state": "\(a.state)".prefix(60).description,
        ]
        var summary = "state: \(a.statusText)\n"
        switch a.state {
        case .ready(let reply):
            record["reply"] = ["explanation_bytes": reply.text.utf8.count, "explanation_head": String(reply.text.prefix(200)),
                               "edits": reply.edits.map { ["start": $0.range.startByte, "end": $0.range.endByte, "removed_bytes": $0.removedText.utf8.count,
                                                            "replacement_bytes": $0.replacement.utf8.count, "relocated": $0.relocated, "replacement": $0.replacement] },
                               "notes": reply.notes, "review_id_prefix": String(reply.reviewId?.prefix(8) ?? "-")]
            summary += "explanation (\(reply.text.utf8.count) bytes): \(reply.text.prefix(200))\n"
            for e in reply.edits { summary += "edit \(e.range.startByte)..<\(e.range.endByte)\(e.relocated ? " RELOCATED" : ""): −\(e.removedText.utf8.count) +\(e.replacement.utf8.count) bytes\n\(e.replacement)\n" }
            for n in reply.notes { summary += "note: \(n)\n" }
            if let p = a.applicationPreview { summary += "before:\n\(p.before)\nafter:\n\(p.after)\n" }
            if let r = a.applicationRefusal { summary += "cannot apply: \(r)\n" }
            if a.canApply {
                model.applyGrokEdit()
                try await waitUntil("approve", timeout: 20) { !a.inFlight }
                record["apply"] = ["state": "\(a.state)".prefix(80).description, "pending_edit": model.pendingEdit.map { "\($0.nsRange) \($0.text.utf8.count) bytes" } ?? "none"]
                summary += "apply: \(a.statusText)\npending edit: \(model.pendingEdit.map { "\($0.nsRange)" } ?? "none")\n"
                if let edit = model.pendingEdit, let r = Range(edit.nsRange, in: model.activeText) {
                    var text = model.activeText
                    text.replaceSubrange(r, with: edit.text)
                    model.editApplied(edit, newText: text)
                    // Compile the edited document with the real compiler as the acceptance gate.
                    model.compile()
                    try await waitUntil("recompile") { model.inFlightRevision == nil && model.result?.revision == model.editorRevision }
                    let after = try XCTUnwrap(model.result)
                    record["compile_after_apply"] = ["status": after.status.rawValue, "diagnostics": after.diagnostics.map(\.message)]
                    summary += "compile after apply: \(after.status.rawValue), \(after.diagnostics.count) diagnostics\n"
                    try text.write(to: dir.appendingPathComponent("after.tex"), atomically: true, encoding: .utf8)
                }
            }
        case .failed(let why): summary += "failed: \(why)\n"
        default: break
        }
        try Self.document.write(to: dir.appendingPathComponent("before.tex"), atomically: true, encoding: .utf8)
        try JSONSerialization.data(withJSONObject: record, options: [.prettyPrinted, .sortedKeys]).write(to: dir.appendingPathComponent("live.json"))
        try ("# Ask Grok live check \(stamp)\n\nmodel \(grok.model), \(String(format: "%.2f", elapsed)) s end to end, "
             + "\(grok.credential?.description ?? "?"). Stages: \(a.childLaunches.map { "\($0.stage)" }.joined(separator: " → ")).\n\n" + summary)
            .write(to: dir.appendingPathComponent("README.md"), atomically: true, encoding: .utf8)
        model.detachWorker()
        print("grok-assistant live evidence: \(dir.path)")
        switch a.state {
        case .ready, .applied: break
        default: XCTFail("live Ask did not reach a reply: \(a.statusText)")
        }
    }

    /// Second (optional) live call: "Fix with Grok" on a real compiler
    /// diagnostic, `FLASHTEX_GROK_LIVE_FIX=1`. Same evidence shape, no Apply.
    func testLiveFixWithGrokOnARealDiagnostic() async throws {
        let env = ProcessInfo.processInfo.environment
        guard env["FLASHTEX_GROK_LIVE_FIX"] == "1" else { throw XCTSkip("set FLASHTEX_GROK_LIVE_FIX=1 for the live Fix with Grok call") }
        guard let compiler = env["FLASHTEX_COMPILER"].map(URL.init(fileURLWithPath:)), FileManager.default.isExecutableFile(atPath: compiler.path) else {
            throw XCTSkip("set FLASHTEX_COMPILER")
        }
        guard let evidenceRoot = env["FLASHTEX_EVIDENCE_DIR"] else { throw XCTSkip("set FLASHTEX_EVIDENCE_DIR") }
        let configuration = GrokAssistant.configuration(env)
        guard let grok = configuration.grok, grok.credential != nil, configuration.helper != nil, env["FLASHTEX_ASSISTANT_CONTEXT_GROK"] != nil else {
            throw XCTSkip("key and helpers required")
        }
        let broken = "\\documentclass{article}\n\\begin{document}\nA fraction: $\\frac{1}{2$ and text.\n\\end{document}\n"
        let model = ShellModel()
        model.autoCompile = false
        model.replaceProject(entryText: broken)
        model.attachWorker(at: compiler)
        model.compile()
        try await waitUntil("real compile") { model.inFlightRevision == nil && model.result != nil }
        let result = try XCTUnwrap(model.result)
        guard let index = result.diagnostics.indices.first(where: { result.diagnostics[$0].source?.path == "main.tex" }) else {
            throw XCTSkip("the compiler reported no located diagnostic for the broken document: \(result.diagnostics.map(\.message))")
        }
        let a = model.grokAssistant
        a.injected = configuration
        model.fixWithGrok(diagnosticIndex: index)
        XCTAssertTrue(a.instruction.hasPrefix("Fix this: "))
        let t0 = Date()
        model.askGrokNow()
        try await waitUntil("live reply", timeout: configuration.providerTimeout + 30) { !a.inFlight }
        let elapsed = Date().timeIntervalSince(t0)
        let stamp = ISO8601DateFormatter().string(from: t0).replacingOccurrences(of: "[-:]", with: "", options: .regularExpression)
        let dir = URL(fileURLWithPath: evidenceRoot).appendingPathComponent("grok-assistant-\(stamp)-fix")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        var summary = "# Fix with Grok live check \(stamp)\n\nmodel \(grok.model), \(String(format: "%.2f", elapsed)) s end to end, \(grok.credential?.description ?? "?").\n"
        summary += "diagnostic \(index): \(result.diagnostics[index].message) at \(result.diagnostics[index].source.map { "\($0.startByte)..<\($0.endByte)" } ?? "-")\n"
        summary += "instruction: \(a.instruction)\ncontext: \(a.contextLine) · \(a.diagnosticsLine)\nselected_diagnostics: \(a.lastRequest?.selectedDiagnostics ?? [])\n"
        summary += "stages: \(a.childLaunches.map { "\($0.stage)" }.joined(separator: " → "))\nstate: \(a.statusText)\n"
        if case .ready(let reply) = a.state {
            summary += "explanation (\(reply.text.utf8.count) bytes): \(reply.text.prefix(300))\n"
            for e in reply.edits { summary += "edit \(e.range.startByte)..<\(e.range.endByte)\(e.relocated ? " RELOCATED" : ""): −\(e.removedText.utf8.count) +\(e.replacement.utf8.count) bytes\n−\(e.removedText)\n+\(e.replacement)\n" }
            for n in reply.notes { summary += "note: \(n)\n" }
            if let p = a.applicationPreview { summary += "before:\n\(p.before)\nafter:\n\(p.after)\n" }
            if let r = a.applicationRefusal { summary += "cannot apply: \(r)\n" }
        }
        try broken.write(to: dir.appendingPathComponent("before.tex"), atomically: true, encoding: .utf8)
        try summary.write(to: dir.appendingPathComponent("README.md"), atomically: true, encoding: .utf8)
        model.detachWorker()
        print("grok-assistant live evidence: \(dir.path)")
        guard case .ready = a.state else { return XCTFail("live Fix did not reach a reply: \(a.statusText)") }
    }

    private func waitUntil(_ what: String, timeout: TimeInterval = 30, _ cond: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if cond() { return }
            try await Task.sleep(nanoseconds: 50_000_000)
        }
        XCTFail("timed out waiting for \(what)")
    }
}
