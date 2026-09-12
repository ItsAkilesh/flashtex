import AppKit
import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// LIVE acceptance of the Mac's Grok/xAI wiring: one explanation through the
/// real `flashtex-assistant-context --provider-session` and one capture
/// conversion through the real `flashtex-bridge --enable-grok`, both with the
/// key from the Mac credential adapter. Runs only when
/// `FLASHTEX_GROK_LIVE=1` AND a key resolves (Keychain or `XAI_API_KEY`);
/// otherwise every case is skipped, so `swift test` never contacts xAI.
/// `apps/mac/scripts/grok-live-check.sh` is the entry point; it sets
/// `FLASHTEX_GROK_EVIDENCE_DIR`, where this writes `explanation.json`,
/// `conversion.json` and `probe.json` (ids, model, sizes, timings, HTTP class —
/// never a key, a prompt body or a reply body).
@MainActor
final class GrokLiveAcceptanceTests: XCTestCase {
    static var live: Bool { ProcessInfo.processInfo.environment["FLASHTEX_GROK_LIVE"] == "1" }
    static var evidenceDirectory: URL? {
        ProcessInfo.processInfo.environment["FLASHTEX_GROK_EVIDENCE_DIR"].map { URL(fileURLWithPath: $0, isDirectory: true) }
    }

    private func requireLive() throws -> GrokCredential.Resolution {
        guard Self.live else { throw XCTSkip("FLASHTEX_GROK_LIVE is not 1: no live xAI call") }
        guard let credential = GrokCredential.resolve() else { throw XCTSkip("no xAI key in the Keychain or XAI_API_KEY: no live xAI call") }
        return credential
    }

    private func record(_ name: String, _ object: [String: Any]) throws {
        guard let dir = Self.evidenceDirectory else { return }
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        var object = object
        object["recorded_utc"] = ISO8601DateFormatter().string(from: Date())
        let data = try JSONSerialization.data(withJSONObject: object, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: dir.appendingPathComponent(name))
    }

    private func waitUntil(_ what: String, timeout: TimeInterval, _ cond: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if cond() { return }
            try await Task.sleep(nanoseconds: 50_000_000)
        }
        XCTFail("timed out waiting for \(what)")
    }

    func testProbeReportsTheKeyClass() async throws {
        let credential = try requireLive()
        let started = Date()
        var listed: [String] = []
        let outcome: GrokProbe.Outcome = await withCheckedContinuation { k in
            GrokProbe.probe(credential: credential, timeout: 20, models: { listed = $0 }) { k.resume(returning: $0) }
        }
        try record("probe.json", ["endpoint": GrokProbe.baseURL().appendingPathComponent(GrokProbe.path).absoluteString,
                                  "key_source": credential.source.rawValue, "outcome": outcome.text,
                                  "key_accepted": outcome.keyAccepted, "seconds": Date().timeIntervalSince(started),
                                  "models_listed": listed])
        XCTAssertTrue(outcome.keyAccepted, outcome.text)
    }

    /// prepare (real helper) → admit/poll (real helper, live xAI) → review (real helper).
    func testLiveExplanationThroughTheProviderSession() async throws {
        let credential = try requireLive()
        var env = ProcessInfo.processInfo.environment
        env["FLASHTEX_ASSISTANT_PROVIDER"] = "grok"
        let config = ProposalPreview.ExplanationConfiguration.fromEnvironment(env)
        guard let helper = config.grok?.helper else { throw XCTSkip("no flashtex-assistant-context helper (build crates/assistant-context --features grok)") }
        XCTAssertEqual(config.grok?.credential, credential)
        // The shadow compile runs on the deterministic worker double so the
        // diagnostic Grok explains is known; the context/binding is the real helper's.
        let preview = ProposalPreview(executable: WorkerClientTests.python, arguments: [WorkerClientTests.fakeWorker.path], explanation: config)
        let input = ProposalPreview.Input(documents: [.init(path: "main.tex", text: "\\documentclass{article}\n\\begin{document}\nHello\n%diag:1 tail\n\\end{document}\n")],
                                          entryPath: "main.tex",
                                          anchor: InsertionAnchor(id: "a1", path: "main.tex", byteOffset: 46, revision: 1, contextAfter: "%diag:1 tail\n\\end{document}\n"),
                                          editorRevision: 1, projectId: "grok-live")
        preview.update(input: input, latex: "$x^2 + y^2 = z^2$ %diag:0 new")
        try await waitUntil("preview ready", timeout: 20) { if case .ready = preview.state { return true }; return false }
        let started = Date()
        preview.explain()
        try await waitUntil("explanation settled", timeout: config.providerTimeout + 30) { !preview.explanationInFlight }
        let seconds = Date().timeIntervalSince(started)
        var evidence: [String: Any] = [
            "helper": helper.path, "model": config.grok?.model ?? "?", "key_source": credential.source.rawValue,
            "seconds": seconds, "launches": preview.childLaunches.map { "\($0.role):\($0.stage)" },
            "session_id": preview.lastGrokLaunch?.sessionId ?? "",
            "session_argv_has_key": preview.lastGrokLaunch?.arguments.contains { $0.contains("xai-") } ?? false,
            "session_env_keys": preview.lastGrokLaunch?.environmentKeys ?? [],
            "note": "xAI response id/model/usage are not surfaced by the helper's provider session (helper request R4 in apps/mac/docs/grok-live.md)",
        ]
        switch preview.explanationState {
        case .ready(let e):
            evidence["outcome"] = "ready"
            evidence["context_id"] = e.context.contextId
            evidence["explanation_bytes"] = e.text.utf8.count
            evidence["edits"] = e.edits.count
            evidence["review_id"] = e.reviewId ?? ""
            evidence["applied"] = e.applied
        case .failed(let why):
            evidence["outcome"] = "failed"
            evidence["failure"] = why
        default:
            evidence["outcome"] = "\(preview.explanationState)"
        }
        let final = preview.explanationState
        try record("explanation.json", evidence)
        preview.close()
        guard case .ready(let e) = final else { return XCTFail("\(final)") }
        XCTAssertFalse(e.text.isEmpty)
        XCTAssertFalse(e.applied)
    }

    /// The deterministic "photo-simulated" capture: handwriting-style text of a
    /// small identity rendered to a PNG (no fixture image exists in the repo;
    /// the 1×1 protocol fixture is rejected by xAI with HTTP 400).
    static func renderedCapture() throws -> RuntimeV1.CaptureImage {
        let size = NSSize(width: 900, height: 260)
        let image = NSImage(size: size)
        image.lockFocus()
        NSColor.white.setFill()
        NSRect(origin: .zero, size: size).fill()
        let font = NSFont(name: "Bradley Hand", size: 64) ?? NSFont(name: "Noteworthy", size: 64) ?? NSFont.systemFont(ofSize: 64)
        let text = NSAttributedString(string: "∫₀¹ x² dx = 1/3   and   α + β ≤ γ", attributes: [.font: font, .foregroundColor: NSColor.black])
        text.draw(at: NSPoint(x: 40, y: 90))
        image.unlockFocus()
        guard let tiff = image.tiffRepresentation, let rep = NSBitmapImageRep(data: tiff),
              let png = rep.representation(using: .png, properties: [:]) else { throw XCTSkip("could not render the capture image") }
        return RuntimeV1.CaptureImage(mimeType: "image/png", dataBase64: png.base64EncodedString())
    }

    /// Distinct matches of `pattern` (or its capture `group`) in `text`, sorted.
    static func matches(_ pattern: String, in text: String, group: Int = 0) -> [String] {
        guard let re = try? NSRegularExpression(pattern: pattern) else { return [] }
        let ns = text as NSString
        let found = re.matches(in: text, range: NSRange(location: 0, length: ns.length)).map { ns.substring(with: $0.range(at: group)) }
        return Array(Set(found)).sorted()
    }

    /// Demo gate: the returned LaTeX inserted at the pin and compiled by OUR
    /// compiler (the review sheet's shadow compile, `ProposalPreview`).
    private func compileGate(latex: String, document: String, anchorByte: Int) async throws -> [String: Any] {
        guard let compiler = ShellModel.locateCompiler() else { return ["compiled": false, "reason": "no flashtex-compiler (build crates/compiler or set FLASHTEX_COMPILER)"] }
        let preview = ProposalPreview(executable: compiler, explanation: .disabled)
        let ctx = String(decoding: Array(document.utf8.dropFirst(anchorByte).prefix(Insertion.contextLength)), as: UTF8.self)
        let input = ProposalPreview.Input(documents: [.init(path: "main.tex", text: document)], entryPath: "main.tex",
                                          anchor: InsertionAnchor(id: "gate", path: "main.tex", byteOffset: anchorByte, revision: 1, contextAfter: ctx),
                                          editorRevision: 1, projectId: "grok-gate")
        preview.update(input: input, latex: latex)
        try await waitUntil("gate compile", timeout: 30) { if case .ready = preview.state { return true }; if case .failed = preview.state { return true }; return false }
        defer { preview.close() }
        switch preview.state {
        case .ready(let r):
            return ["compiled": true, "compiler": compiler.path, "status": r.status.rawValue, "pages": r.pageCount,
                    "new_diagnostics": r.new.count, "new_errors": r.newErrorCount, "nearby_diagnostics": r.nearby.count,
                    "new_messages": r.new.prefix(8).map { $0.diagnostic.message },
                    "zero_errors": r.newErrorCount == 0 && (r.status == .ok || r.status == .recovered)]
        case .failed(let why): return ["compiled": false, "reason": why]
        default: return ["compiled": false, "reason": "\(preview.state)"]
        }
    }

    /// submit (real bridge) → capture_convert (live Grok vision, with the
    /// compiler-derived `supported_features`) → proposal queued for the review
    /// sheet, nothing inserted → the proposal compiled by our compiler.
    func testLiveCaptureConversionThroughTheBridge() async throws {
        let credential = try requireLive()
        guard let bridge = BridgeClient.locateBridge() else { throw XCTSkip("no flashtex-bridge (build crates/bridge or set FLASHTEX_BRIDGE)") }
        let launch = ShellModel.bridgeGrokLaunch()
        XCTAssertTrue(launch.enableGrok)
        XCTAssertEqual(launch.credential, credential)
        let store = try BridgeClientTests.tempStore()
        defer { try? FileManager.default.removeItem(at: store) }
        let model = ShellModel()
        model.autoCompile = false
        let attached = await model.attachBridgeAndWait(executable: bridge, storeDirectory: store, enableGrok: launch.enableGrok,
                                                       environment: launch.environment, discoverLedger: false)
        XCTAssertTrue(attached, model.captureNote ?? model.bridgeStatus)
        XCTAssertEqual(model.bridge?.grokEnabled, true)
        let document = "\\documentclass{article}\n\\begin{document}\nThe identity \n\\end{document}\n"
        model.updateActiveText(document)
        model.caretUTF16 = 53
        model.pinAnchorAtCaret()
        try await waitUntil("pin", timeout: 10) { model.bridgeDestination != nil }
        let image = try Self.renderedCapture()
        let captureId = "grok-live-\(UUID().uuidString.lowercased().prefix(8))"
        let received = await model.submitCapture(image: image, captureId: captureId, instructions: CaptureFeatures.defaultInstructions)
        XCTAssertNotNil(received, model.captureNote ?? "")
        let features = CaptureFeatures.supportedFeatures()
        let started = Date()
        let proposal = await model.convertCapture(captureId: captureId, supportedFeatures: features)
        let seconds = Date().timeIntervalSince(started)
        var evidence: [String: Any] = [
            "bridge": bridge.path, "model": launch.environment["FLASHTEX_GROK_MODEL"] ?? "?", "key_source": credential.source.rawValue,
            "capture_id": captureId, "seconds": seconds, "enable_grok": launch.enableGrok,
            "image_bytes": Data(base64Encoded: image.dataBase64)?.count ?? 0,
            "supported_features_count": features.count, "supported_features_sha": CaptureFeatures.compilerSHA,
            "bridge_env_has_key": launch.environment[GrokCredential.bridgeVariable] != nil,
            "bridge_env_sensitive_names_other_than_key": launch.environment.keys.filter { $0 != GrokCredential.bridgeVariable && ProposalPreview.ExplanationConfiguration.isSensitiveVariable($0) },
            "note": "xAI response id/usage are not surfaced by the bridge's capture_proposal (helper request R4 in apps/mac/docs/grok-live.md)",
        ]
        if let proposal {
            evidence["outcome"] = "proposal"
            evidence["latex_bytes"] = proposal.latex.utf8.count
            evidence["ambiguities"] = proposal.ambiguities.count
            evidence["required_dependencies"] = proposal.requiredDependencies
            evidence["context_revision"] = proposal.contextRevision ?? -1
            evidence["latex_commands"] = Self.matches(#"\\[A-Za-z]+"#, in: proposal.latex)
            evidence["latex_environments"] = Self.matches(#"\\begin\{([A-Za-z*]+)\}"#, in: proposal.latex, group: 1)
            evidence["compile_gate"] = try await compileGate(latex: proposal.latex, document: document, anchorByte: 53)
        } else {
            evidence["outcome"] = "failed"
            evidence["failure"] = model.captureNote ?? model.bridgeStatus
        }
        try record("conversion.json", evidence)
        model.detachBridge()
        XCTAssertNotNil(proposal, model.captureNote ?? "")
        XCTAssertEqual(model.proposals.count, proposal == nil ? 0 : 1, "queued for review; never inserted")
        XCTAssertEqual(model.activeText, document)
        if let gate = evidence["compile_gate"] as? [String: Any] {
            XCTAssertEqual(gate["zero_errors"] as? Bool, true, "\(gate)")
        }
    }
}
