import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// End-to-end against the real `flashtex-preview-controller` helper
/// (crates/preview-controller, STDIO.md) with the real compiler. Skipped unless
/// `FLASHTEX_PREVIEW_CONTROLLER` and `FLASHTEX_COMPILER` point at built binaries.
@MainActor
final class PreviewControllerTests: XCTestCase {
    static var helper: URL? {
        ProcessInfo.processInfo.environment["FLASHTEX_PREVIEW_CONTROLLER"].map { URL(fileURLWithPath: $0) }
    }

    func testEditsBecomeDurableAndPreviewsBindToEditorRevisions() async throws {
        guard let helper = Self.helper, FileManager.default.isExecutableFile(atPath: helper.path),
              ShellModel.locateCompiler() != nil else {
            throw XCTSkip("set FLASHTEX_PREVIEW_CONTROLLER and FLASHTEX_COMPILER to built binaries")
        }
        let root = FileManager.default.temporaryDirectory.appendingPathComponent("pc-test-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: root.appendingPathComponent("project"), withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }
        let tex = root.appendingPathComponent("project/main.tex")
        try "\\begin{document}\nHello durable world.\n\\end{document}\n".write(to: tex, atomically: true, encoding: .utf8)
        setenv("FLASHTEX_CONTROLLER_LEDGER_ROOT", root.appendingPathComponent("ledger").path, 1)
        defer { unsetenv("FLASHTEX_CONTROLLER_LEDGER_ROOT") }

        let model = ShellModel()
        model.autoCompile = true
        XCTAssertEqual(model.openTex(at: tex), .opened)
        model.attachController(at: helper)
        XCTAssertTrue(model.controllerAttached)
        XCTAssertTrue(model.workerAttached)
        // ready → document → initial compile → preview bound to the current editor revision.
        try await waitUntil { model.result?.revision == model.editorRevision && model.previewSource == .worker("flashtex-preview-controller") }
        XCTAssertFalse(model.previewIsStale)
        XCTAssertEqual(model.controllerState.durable["main.tex"]?.revision, 1)
        let firstItems = model.result!.pages[0].items.count

        // A burst of keystrokes: one edit in flight, newest buffer coalesced; the
        // final preview is bound to the final editor revision and never stale.
        for i in 1...6 { model.updateActiveText("\\begin{document}\nHello durable world \(String(repeating: "x", count: i)).\n\\end{document}\n") }
        let final = model.editorRevision
        try await waitUntil { model.result?.revision == final && model.inFlightRevision == nil }
        XCTAssertFalse(model.previewIsStale)
        XCTAssertGreaterThanOrEqual(model.result!.pages[0].items.count, firstItems)
        let durable = try XCTUnwrap(model.controllerState.durable["main.tex"])
        XCTAssertGreaterThanOrEqual(durable.revision, 2)
        XCTAssertLessThanOrEqual(durable.revision, 7, "six keystrokes coalesce into at most six durable edits")
        XCTAssertEqual(model.controllerState.textByDurable["main.tex"]?[durable.revision], model.activeText, "the durable text is the buffer")
        XCTAssertEqual(model.compiledDocuments["main.tex"], model.activeText, "the preview was compiled from the durable text")
        XCTAssertEqual(model.negotiation.accepted, model.requestedLayoutCapabilities, "configure_layout opted into the negotiated primitives")

        // The ledger persists: a second helper on the same project restores the durable text.
        model.detachController()
        XCTAssertFalse(model.controllerAttached)
        let model2 = ShellModel()
        model2.autoCompile = true
        XCTAssertEqual(model2.openTex(at: tex), .opened) // disk still has the ORIGINAL text
        model2.attachController(at: helper)
        try await waitUntil { model2.controllerState.durable["main.tex"] != nil && model2.result?.revision == model2.editorRevision && model2.inFlightRevision == nil }
        // The disk text differs from the ledger: the buffer (disk) was submitted as a new durable revision.
        XCTAssertGreaterThan(model2.controllerState.durable["main.tex"]!.revision, durable.revision)
        XCTAssertEqual(model2.controllerState.textByDurable["main.tex"]?[model2.controllerState.durable["main.tex"]!.revision], model2.activeText)
        model2.detachController()
    }

    func testControllerSaveExportsDurableSourceAndRefusesChangedDisk() async throws {
        guard let helper = Self.helper, FileManager.default.isExecutableFile(atPath: helper.path),
              ShellModel.locateCompiler() != nil else {
            throw XCTSkip("set FLASHTEX_PREVIEW_CONTROLLER and FLASHTEX_COMPILER to built binaries")
        }
        let root = FileManager.default.temporaryDirectory.appendingPathComponent("pc-save-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: root.appendingPathComponent("project"), withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }
        let tex = root.appendingPathComponent("project/main.tex")
        try "\\begin{document}\nSave me.\n\\end{document}\n".write(to: tex, atomically: true, encoding: .utf8)
        setenv("FLASHTEX_CONTROLLER_LEDGER_ROOT", root.appendingPathComponent("ledger").path, 1)
        defer { unsetenv("FLASHTEX_CONTROLLER_LEDGER_ROOT") }
        let model = ShellModel()
        model.autoCompile = true
        XCTAssertEqual(model.openTex(at: tex), .opened)
        model.attachController(at: helper)
        try await waitUntil { model.result?.revision == model.editorRevision && model.controllerState.durable["main.tex"] != nil }

        // Edit, save through the helper: the file holds exactly the buffer, the buffer is clean.
        model.updateActiveText("\\begin{document}\nSave me, durably.\n\\end{document}\n")
        XCTAssertTrue(model.isDirty)
        let saved = await model.controllerSave()
        guard case .saved(let sha) = saved else { return XCTFail("\(saved)") }
        XCTAssertEqual(try String(contentsOf: tex, encoding: .utf8), model.activeText)
        XCTAssertEqual(sha, SourceDigest.sha256Hex(model.activeText))
        XCTAssertFalse(model.isDirty)
        let disk = await model.controllerFileStatus(path: "main.tex")
        XCTAssertEqual(disk?.state, "matches_source")

        // Someone else changes the file: the next save is refused as a conflict, nothing overwritten.
        try "external edit\n".write(to: tex, atomically: true, encoding: .utf8)
        model.updateActiveText("\\begin{document}\nSave me again.\n\\end{document}\n")
        let refused = await model.controllerSave()
        guard case .conflict(let c) = refused else { return XCTFail("expected a conflict, got \(refused)") }
        XCTAssertTrue(c.viaHelper)
        XCTAssertEqual(c.kind, .modifiedExternally)
        XCTAssertEqual(c.theirs, SourceDigest.sha256Hex("external edit\n"), "the conflict names the hash now on disk")
        XCTAssertEqual(try String(contentsOf: tex, encoding: .utf8), "external edit\n", "the external edit was not overwritten")
        XCTAssertTrue(model.isDirty, "the buffer stays dirty and intact")
        XCTAssertNotNil(model.files.conflict)
        model.detachController()
    }

    private func waitUntil(timeout: TimeInterval = 15, _ cond: () -> Bool) async throws {
        let start = Date()
        while !cond() {
            if Date().timeIntervalSince(start) > timeout { throw XCTSkip("timeout") }
            try await Task.sleep(nanoseconds: 30_000_000)
        }
    }
}
