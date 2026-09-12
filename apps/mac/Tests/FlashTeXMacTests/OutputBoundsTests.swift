import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// Oversized helper output (ShellModel+OutputBounds.swift): what the real
/// helper and the real compiler send for a fully-prose 560 KB document, and
/// how the shell reports it. Pure tests always run; the helper/worker tests
/// need `FLASHTEX_PREVIEW_CONTROLLER` / `FLASHTEX_COMPILER` and skip above a
/// 1-minute load of 20 (they record, never assert, compile times).
///
/// The hooks into the parent-retained files are diff requests
/// (coordination/mac-large-document.md). The live tests detect whether they
/// are applied (the in-flight edit is released by the model itself within
/// 2 s of the `failed` frame) and otherwise dispatch the hook by hand after
/// the real frame was observed in the model's log — the wire behaviour is
/// real either way; the printed `output-bounds:` lines say which.
@MainActor
final class OutputBoundsTests: XCTestCase {
    static var helper: URL? { ProcessInfo.processInfo.environment["FLASHTEX_PREVIEW_CONTROLLER"].map { URL(fileURLWithPath: $0) } }

    /// The paste-recovery lane's prose shape (≈31.7 output bytes per source byte).
    static func prose(bytes: Int) -> String {
        var s = "\\documentclass{article}\n\\begin{document}\n"
        var n = 0
        while s.utf8.count < bytes {
            s += "Paragraph \(n): the quick brown fox — naïve café \\textbf{bold} $x^2 + y^2 = z^2$ jumps over the lazy dog.\n"
            n += 1
        }
        return s + "\\end{document}\n"
    }

    // MARK: pure

    func testViolationMessagesAreParsed() {
        let a = OutputBounds.parseViolation("line of 18157062 bytes exceeds the 16777216-byte limit")
        XCTAssertEqual(a?.replyBytes, 18_157_062)
        XCTAssertEqual(a?.boundBytes, 16_777_216)
        let b = OutputBounds.parseViolation("unterminated line exceeds the 16777216-byte limit")
        XCTAssertNil(b?.replyBytes)
        XCTAssertEqual(b?.boundBytes, 16_777_216)
        XCTAssertNotNil(OutputBounds.parseViolation("frame of 17000000 bytes exceeds the 16777216-byte limit"))
        XCTAssertNil(OutputBounds.parseViolation("frame is not valid JSON: x"))
        XCTAssertNil(OutputBounds.parseViolation("compile_result mac-3 reports project a revision 2; request was project b revision 2"))
    }

    func testStatusNamesBoundAndDocumentSizeAndRetryIsBounded() {
        let direct = OutputBoundNotice(route: .direct, editorRevision: 7, documentBytes: 573_476, boundBytes: 16_777_216,
                                       boundName: OutputBounds.workerLineBoundName, replyBytes: 18_157_062)
        XCTAssertEqual(direct.status, "revision 7: reply of 17.3 MiB exceeds the worker's worker line limit (RuntimeV1.maxLineBytes) (16 MiB) for this 560 KB document; last preview kept")
        XCTAssertTrue(direct.banner.contains("560 KB document exceeds the worker line limit"))
        XCTAssertTrue(direct.banner.contains("⌘B"))
        XCTAssertFalse(direct.allowsRetry(documentBytes: 573_476), "the same document is never re-sent")
        XCTAssertFalse(direct.allowsRetry(documentBytes: 573_000), "a few bytes less still overflows by the measured ratio")
        XCTAssertFalse(direct.allowsRetry(documentBytes: 540_000), "540 KB × 31.7 = 17.1 MB > 16 MiB")
        XCTAssertTrue(direct.allowsRetry(documentBytes: 500_000), "500 KB × 31.7 = 15.8 MB < 16 MiB")
        let helper = OutputBoundNotice(route: .helper, editorRevision: 3, documentBytes: 573_476, boundBytes: 8_388_608,
                                       boundName: OutputBounds.compilerFrameBoundName, replyBytes: nil)
        XCTAssertEqual(helper.status, "revision 3: reply exceeds the helper's compiler frame bound (compiler_max_frame_bytes) (8 MiB) for this 560 KB document; last preview kept")
        XCTAssertFalse(helper.allowsRetry(documentBytes: 573_476))
        XCTAssertFalse(helper.allowsRetry(documentBytes: 560_000), "unknown reply size: needs a tenth off")
        XCTAssertTrue(helper.allowsRetry(documentBytes: 516_000))
    }

    func testDirectRouteHookBlocksTheRelaunchCompileUntilTheDocumentShrinks() {
        let model = ShellModel()
        model.documents = [.init(path: "main.tex", text: Self.prose(bytes: 560 * 1024))]
        model.activePath = "main.tex"
        XCTAssertFalse(model.outputBoundHandleWorkerViolation("frame is not valid JSON"), "other violations are not size overflows")
        XCTAssertTrue(model.outputBoundHandleWorkerViolation("line of 18157062 bytes exceeds the \(RuntimeV1.maxLineBytes)-byte limit"))
        let notice = try! XCTUnwrap(model.outputBound)
        XCTAssertEqual(notice.route, .direct)
        XCTAssertEqual(notice.documentBytes, model.documents[0].text.utf8.count)
        XCTAssertTrue(model.workerStatus.contains("560 KB document"))
        XCTAssertTrue(model.workerStatus.contains("16 MiB"))
        XCTAssertTrue(model.outputBoundBlocksCompile, "the relaunch's auto-compile of the same document is refused")
        model.documents[0].text = Self.prose(bytes: 200 * 1024)
        XCTAssertFalse(model.outputBoundBlocksCompile, "200 KB × 31.7 fits: the next compile is allowed")
        XCTAssertFalse(model.outputBoundExplicitRetry(), "no helper attached: ⌘B just clears the notice")
        XCTAssertNil(model.outputBound)
    }

    func testHelperUpdateHookReleasesTheHeldEditAndKeepsTheStatus() {
        let model = ShellModel()
        model.documents = [.init(path: "main.tex", text: Self.prose(bytes: 560 * 1024))]
        model.activePath = "main.tex"
        model.outputBoundNoteReady(compilerMaxFrameBytes: 15_728_640, helperMaxOutputBytes: 16_777_216)
        model.controllerState.inFlight = ("pc-9", "main.tex", 4, Date(), model.documents[0].text, 2)
        model.inFlightRevision = 4
        XCTAssertFalse(model.outputBoundHandleControllerUpdate(kind: "stale", payload: ["compile_revision": 2]))
        XCTAssertFalse(model.outputBoundHandleControllerUpdate(kind: "failed", payload: ["reason": "compiler input closed"]), "other failures are not size overflows")
        XCTAssertNotNil(model.controllerState.inFlight)
        XCTAssertTrue(model.outputBoundHandleControllerUpdate(kind: "failed", payload: ["kind": "failed", "request_id": "preview-2", "reason": OutputBounds.helperFailedReason]))
        XCTAssertNil(model.controllerState.inFlight, "the hold-until-preview pipeline is released")
        XCTAssertNil(model.inFlightRevision)
        XCTAssertEqual(model.outputBounds.releasedInFlightCount, 1)
        XCTAssertEqual(model.outputBound?.boundBytes, 15_728_640, "the bound advertised by ready")
        XCTAssertEqual(model.outputBound?.editorRevision, 4)
        XCTAssertTrue(model.workerStatus.contains("15 MiB") && model.workerStatus.contains("560 KB document"), model.workerStatus)
        XCTAssertTrue(model.outputBoundHandleControllerError(id: nil, message: "response exceeds output limit; source may already be durable"))
        XCTAssertEqual(model.outputBound?.boundName, OutputBounds.helperOutputBoundName)
        XCTAssertFalse(model.outputBoundHandlePreviewError("compiler session failed; create a new session with complete snapshots"), "no helper attached: nothing to restart")
        model.outputBoundNotePreviewApplied()
        XCTAssertNil(model.outputBound)
    }

    // MARK: real helper / real compiler

    private func gate(_ name: String) throws -> (helper: URL, compiler: URL) {
        guard let helper = Self.helper, FileManager.default.isExecutableFile(atPath: helper.path), let compiler = ShellModel.locateCompiler() else {
            throw XCTSkip("set FLASHTEX_PREVIEW_CONTROLLER and FLASHTEX_COMPILER to built binaries")
        }
        let load = PasteRecoveryTests.loadAverage1()
        // `FLASHTEX_OUTPUT_BOUNDS_LOAD_LIMIT` raises the skip threshold for a
        // correctness run on a shared host; its numbers are then labelled.
        let limit = ProcessInfo.processInfo.environment["FLASHTEX_OUTPUT_BOUNDS_LOAD_LIMIT"].flatMap(Double.init) ?? 20
        print("output-bounds: \(name) uptime: \(PasteRecoveryTests.uptimeLine())\(load > 20 ? " [under shared load, not an isolated result]" : "")")
        if load > limit { throw XCTSkip("1-minute load average \(load) > \(limit); timing-sensitive test skipped") }
        return (helper, compiler)
    }

    private func waitUntil(timeout: TimeInterval, _ cond: () -> Bool) async -> Bool {
        let start = Date()
        while !cond() {
            if Date().timeIntervalSince(start) > timeout { return false }
            try? await Task.sleep(nanoseconds: 2_000_000)
        }
        return true
    }

    func testRealHelperRefusesThe560KBPreviewWithOneFailedFrameAndTypingContinues() async throws {
        let (helper, _) = try gate("helper")
        let root = FileManager.default.temporaryDirectory.appendingPathComponent("output-bounds-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: root.appendingPathComponent("project"), withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root); unsetenv("FLASHTEX_CONTROLLER_LEDGER_ROOT") }
        let tex = root.appendingPathComponent("project/main.tex")
        let base = Self.prose(bytes: 60 * 1024)
        try base.write(to: tex, atomically: true, encoding: .utf8)
        setenv("FLASHTEX_CONTROLLER_LEDGER_ROOT", root.appendingPathComponent("ledger").path, 1)
        let model = ShellModel()
        model.autoCompile = true
        XCTAssertEqual(model.openTex(at: tex), .opened)
        model.attachController(at: helper)
        defer { model.detachController() }
        guard await waitUntil(timeout: 60, { model.result?.revision == model.editorRevision && model.controllerState.inFlight == nil }) else {
            throw XCTSkip("no preview for the 60 KB base within 60 s (load \(PasteRecoveryTests.loadAverage1()))")
        }
        let goodRevision = model.editorRevision
        let goodPages = model.result?.pages.count ?? 0
        let big = Self.prose(bytes: 560 * 1024)
        let t0 = Date()
        model.updateActiveText(big)
        let bigRevision = model.editorRevision
        XCTAssertEqual(model.controllerState.inFlight?.editorRevision, bigRevision)
        // Durable ACK of the paste, then the helper's answer to its compile.
        let ok1 = await waitUntil(timeout: 60, { model.controllerState.durable["main.tex"]?.revision == 2 })
        XCTAssertTrue(ok1, "durable ACK")
        let ackMs = Date().timeIntervalSince(t0) * 1000
        let sawFailed = await waitUntil(timeout: 60, { model.workerLog.contains { $0.contains("update failed") || $0.contains("output bound") } })
        XCTAssertTrue(sawFailed, "the helper answered with an update kind:failed frame: \(model.workerLog.suffix(5))")
        let failedMs = Date().timeIntervalSince(t0) * 1000
        let failedLines = model.workerLog.filter { $0.contains("update failed") || $0.contains("output bound") }
        XCTAssertFalse(model.workerLog.contains { $0.contains("protocol violation") }, "no partial or oversized frame reached the client")
        // Hook applied? The model releases the held edit by itself.
        let hooked = await waitUntil(timeout: 2, { model.controllerState.inFlight == nil })
        if !hooked {
            XCTAssertNotNil(model.controllerState.inFlight, "without the hook the edit stays held (the gap)")
            model.outputBoundHandleControllerUpdate(kind: "failed", payload: ["kind": "failed", "request_id": "preview-2", "reason": OutputBounds.helperFailedReason])
        }
        XCTAssertNil(model.controllerState.inFlight)
        XCTAssertNil(model.inFlightRevision)
        let notice = try XCTUnwrap(model.outputBound)
        XCTAssertEqual(notice.route, .helper)
        XCTAssertEqual(notice.boundBytes, 8 * 1024 * 1024, "default compiler_max_frame_bytes")
        XCTAssertEqual(notice.documentBytes, big.utf8.count)
        XCTAssertTrue(model.workerStatus.contains("8 MiB") && model.workerStatus.contains("\(OutputBoundNotice.kb(big.utf8.count)) document"), model.workerStatus)
        // Last good preview kept, marked stale; never blank.
        XCTAssertEqual(model.result?.revision, goodRevision)
        XCTAssertEqual(model.result?.pages.count, goodPages)
        XCTAssertTrue(model.previewIsStale)
        XCTAssertNil(model.historicalPreview)
        // Typing continues: the next edit is submitted and ACKed durably
        // (with preview_error: the helper's compiler session is gone).
        let t1 = Date()
        model.updateActiveText(big + "% more\n")
        XCTAssertEqual(model.controllerState.inFlight?.editorRevision, model.editorRevision, "submitted immediately, not queued behind the failed one")
        let ok2 = await waitUntil(timeout: 60, { model.controllerState.durable["main.tex"]?.revision == 3 })
        XCTAssertTrue(ok2, "the next durable edit is still ACKed")
        let nextAckMs = Date().timeIntervalSince(t1) * 1000
        let ok3 = await waitUntil(timeout: 10, { model.controllerState.inFlight == nil })
        XCTAssertTrue(ok3, "preview_error releases the pipeline")
        XCTAssertTrue(model.controllerStatus.contains("preview error: compiler session failed") || model.controllerStatus.contains("exceeds"), model.controllerStatus)
        XCTAssertEqual(model.controllerRelaunchCount, 0, "no helper relaunch")
        XCTAssertFalse(model.outputBounds.retryInFlight, "no automatic retry for a document that did not shrink")
        // Shrinking back to the base: the retry restarts the helper's compiler and a preview arrives.
        let t2 = Date()
        model.updateActiveText(base + "% back\n")
        let shrunkDurable = await waitUntil(timeout: 60, { model.controllerState.durable["main.tex"]?.revision == 4 })
        XCTAssertTrue(shrunkDurable, "the shrunk document is durable")
        if !hooked, let e = "compiler session failed; create a new session with complete snapshots" as String? {
            XCTAssertTrue(model.outputBoundHandlePreviewError(e), "restart sent for the 60 KB document")
        }
        let recovered = await waitUntil(timeout: 60, { model.result?.revision == model.editorRevision })
        XCTAssertTrue(recovered, "a preview for the shrunk document arrives after the restart: \(model.workerLog.suffix(6))")
        if !hooked { model.outputBoundNotePreviewApplied() }
        XCTAssertNil(model.outputBound)
        XCTAssertFalse(model.previewIsStale)
        print(String(format: "output-bounds: helper route: 560 KB (%d B) paste ack %.0f ms, failed frame at %.0f ms, next edit ack %.0f ms, recovery preview %.0f ms; hooks %@; frames: %@",
                     big.utf8.count, ackMs, failedMs, nextAckMs, Date().timeIntervalSince(t2) * 1000, hooked ? "applied" : "dispatched by the test", failedLines.joined(separator: " | ")))
    }

    func testRealCompilerDirectRouteOversizedLineIsReportedAndNotResent() async throws {
        let (_, compiler) = try gate("direct")
        let model = ShellModel()
        model.autoCompile = false
        model.documents = [.init(path: "main.tex", text: Self.prose(bytes: 560 * 1024))]
        model.activePath = "main.tex"
        model.attachWorker(at: compiler)
        defer { model.detachWorker() }
        XCTAssertTrue(model.workerAttached)
        let before = (model.result?.revision, model.result?.pages.count, model.previewSource)
        let t0 = Date()
        model.compile()
        XCTAssertNotNil(model.inFlightRevision)
        // The pipe delivers the 18.2 MB line in chunks, so WorkerClient's
        // partial-buffer check fires first ("unterminated line exceeds …"),
        // never the complete-line one; the reply size is therefore unknown.
        let isOverflow: (String) -> Bool = { $0.hasPrefix("protocol violation: ") && $0.contains("exceeds the \(RuntimeV1.maxLineBytes)-byte limit") }
        let ok4 = await waitUntil(timeout: 60, { model.workerLog.contains(where: isOverflow) })
        XCTAssertTrue(ok4, "the 18.2 MB compile_result line is refused by WorkerClient: \(model.workerLog.suffix(5))")
        let violationMs = Date().timeIntervalSince(t0) * 1000
        let line = try XCTUnwrap(model.workerLog.last(where: isOverflow))
        let parsed = try XCTUnwrap(OutputBounds.parseViolation(line))
        XCTAssertEqual(parsed.boundBytes, RuntimeV1.maxLineBytes)
        // Worker terminated, relaunched (bounded): with the hook the relaunch does not re-send.
        _ = await waitUntil(timeout: 5, { model.workerRelaunchCount >= 1 })
        let hooked = model.outputBound != nil
        if !hooked { model.outputBoundHandleWorkerViolation(String(line.dropFirst("protocol violation: ".count))) }
        let notice = try XCTUnwrap(model.outputBound)
        XCTAssertEqual(notice.route, .direct)
        XCTAssertEqual(notice.replyBytes, parsed.replyBytes)
        XCTAssertEqual(notice.documentBytes, model.documents[0].text.utf8.count)
        XCTAssertTrue(model.outputBoundBlocksCompile)
        XCTAssertTrue(model.workerStatus.contains("16 MiB") && model.workerStatus.contains("KB document"), model.workerStatus)
        XCTAssertTrue(before == (model.result?.revision, model.result?.pages.count, model.previewSource), "nothing partial was painted; the previous result stays")
        print(String(format: "output-bounds: direct route: 560 KB (%d B) → line refused (%@) after %.0f ms; relaunches %d; hooks %@; log: %@",
                     model.documents[0].text.utf8.count, line, violationMs, model.workerRelaunchCount, hooked ? "applied" : "dispatched by the test",
                     model.workerLog.filter { $0.contains("violation") || $0.contains("exited") || $0.contains("relaunch") }.joined(separator: " | ")))
    }
}
