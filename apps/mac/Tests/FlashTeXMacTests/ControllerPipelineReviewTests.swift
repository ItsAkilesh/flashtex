import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// Adversarial review of the edit → durable → preview pipeline through the
/// real `flashtex-preview-controller` (lane mac-core-review). Each test is
/// a reproduced defect; the assertions are explicit failures, never skips,
/// once the helper is available.
@MainActor
final class ControllerPipelineReviewTests: XCTestCase {
    static var helper: URL? {
        ProcessInfo.processInfo.environment["FLASHTEX_PREVIEW_CONTROLLER"].map { URL(fileURLWithPath: $0) }
    }

    /// A compiler that answers every request through the real compiler until
    /// one carries `marker`, on which it dies: the helper reports that compile
    /// as `update {kind:"failed", reason:"compiler output closed"}` and its
    /// compiler session is gone (crates/document-runtime `fail`;
    /// crates/preview-controller/src/main.rs).
    private func writeDyingCompiler(in dir: URL, real: URL, marker: String) throws -> URL {
        let script = dir.appendingPathComponent("dying-compiler.sh")
        let text = """
        #!/bin/sh
        while IFS= read -r line; do
          case "$line" in *\(marker)*) exit 3;; esac
          printf '%s\\n' "$line" | "\(real.path)"
        done
        exit 0

        """
        try text.write(to: script, atomically: true, encoding: .utf8)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: script.path)
        return script
    }

    /// Finding 1: under the default hold-until-preview policy the in-flight edit
    /// is released only by a `preview` for its durable revision or by a
    /// `stale`/`discarded` update whose `compile_revision` passes a numeric
    /// guard. The helper reports a compile that failed (compiler crash) as
    /// `update {kind:"failed", request_id, reason}`, which the shell ignored:
    /// the edit stayed in flight forever, every later keystroke was queued and
    /// never sent (typing stall until relaunch), and the buffer stopped being
    /// made durable.
    func testCompilerFailureReleasesTheInFlightEditSoTypingStaysDurable() async throws {
        guard let helper = Self.helper, FileManager.default.isExecutableFile(atPath: helper.path),
              let realCompiler = ShellModel.locateCompiler() else {
            throw XCTSkip("set FLASHTEX_PREVIEW_CONTROLLER and FLASHTEX_COMPILER to built binaries")
        }
        XCTAssertEqual(ControllerReleasePolicy.fromEnvironment(), .holdUntilPreview, "this reproduction is for the default policy")
        let root = FileManager.default.temporaryDirectory.appendingPathComponent("pc-review-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: root.appendingPathComponent("project"), withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }
        let tex = root.appendingPathComponent("project/main.tex")
        try "\\begin{document}\nHello failing compiler.\n\\end{document}\n".write(to: tex, atomically: true, encoding: .utf8)
        setenv("FLASHTEX_CONTROLLER_LEDGER_ROOT", root.appendingPathComponent("ledger").path, 1)
        defer { unsetenv("FLASHTEX_CONTROLLER_LEDGER_ROOT") }
        let dying = try writeDyingCompiler(in: root, real: realCompiler, marker: "KILLCOMPILER")
        let originalCompiler = ProcessInfo.processInfo.environment["FLASHTEX_COMPILER"]
        setenv("FLASHTEX_COMPILER", dying.path, 1)
        defer { if let originalCompiler { setenv("FLASHTEX_COMPILER", originalCompiler, 1) } else { unsetenv("FLASHTEX_COMPILER") } }

        let model = ShellModel()
        model.autoCompile = true
        XCTAssertEqual(model.openTex(at: tex), .opened)
        model.attachController(at: helper)
        defer { model.detachController() }
        // First compile goes through the real compiler (via the wrapper): a current preview.
        let ok1 = await settles(15) { model.result?.revision == model.editorRevision && model.previewSource == .worker("flashtex-preview-controller") && model.inFlightRevision == nil }
        XCTAssertTrue(ok1,
                      "initial preview never arrived: \(model.controllerStatus) / \(model.workerStatus)")
        XCTAssertEqual(model.controllerState.durable["main.tex"]?.revision, 1)
        let initial = model.editorRevision

        // The edit is durable (r2); its compile carries the marker: the compiler dies.
        model.updateActiveText("\\begin{document}\nHello failing compiler, KILLCOMPILER.\n\\end{document}\n")
        let edited = model.editorRevision
        let ok2 = await settles(10) { model.controllerState.durable["main.tex"]?.revision == 2 }
        XCTAssertTrue(ok2,
                      "edit never became durable: \(model.controllerStatus)")
        // The helper reports the failed compile; the pipeline must be released.
        let ok3 = await settles(10) { model.controllerState.inFlight == nil && model.inFlightRevision == nil }
        XCTAssertTrue(ok3,
                      "in-flight edit for revision \(edited) never released after the compiler failure: \(model.controllerStatus); log tail: \(model.workerLog.suffix(6))")
        XCTAssertTrue(model.workerLog.contains { $0.contains("failed") && $0.contains("compiler output closed") }, "the failure is logged: \(model.workerLog.suffix(6))")
        XCTAssertTrue(model.controllerStatus.contains("compiler output closed"), "the failure is visible: \(model.controllerStatus)")

        // Typing continues: each later edit is sent and made durable (the helper
        // answers with a separate preview_error since its compiler session is gone).
        model.updateActiveText("\\begin{document}\nHello failing compiler, edited twice.\n\\end{document}\n")
        let ok4 = await settles(10) { model.controllerState.durable["main.tex"]?.revision == 3 && model.inFlightRevision == nil }
        XCTAssertTrue(ok4,
                      "typing after the failure stalled: durable \(String(describing: model.controllerState.durable["main.tex"]?.revision)), inFlight \(String(describing: model.inFlightRevision)), \(model.controllerStatus)")
        XCTAssertEqual(model.controllerState.textByDurable["main.tex"]?[3], model.activeText)
        // The last good preview is kept; nothing older or newer was invented.
        XCTAssertEqual(model.result?.revision, initial)
    }

    /// Finding 2: `openTex` names the entry document `main.tex` whatever the
    /// file is called, and `attachController` therefore roots the helper in a
    /// SESSION TEMPORARY project (a copy of the buffer) whenever the file name
    /// differs. `saveTexInteractive` routed the save through the helper on
    /// `controllerAttached` alone, so the helper's rooted export wrote the
    /// temporary copy, the shell reported "Saved paper.tex" and marked the
    /// buffer clean, and the real file was never written. The file-routing
    /// predicate the reload/status paths use (`controllerRoutesFiles`) already
    /// refuses this case; the save path must use it too.
    func testSaveOfAFileNotNamedMainTexWritesThatFileNotTheHelpersSessionCopy() async throws {
        guard let helper = Self.helper, FileManager.default.isExecutableFile(atPath: helper.path),
              ShellModel.locateCompiler() != nil else {
            throw XCTSkip("set FLASHTEX_PREVIEW_CONTROLLER and FLASHTEX_COMPILER to built binaries")
        }
        let root = FileManager.default.temporaryDirectory.appendingPathComponent("pc-review-save-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: root.appendingPathComponent("project"), withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }
        let tex = root.appendingPathComponent("project/paper.tex")
        let original = "\\begin{document}\nA paper.\n\\end{document}\n"
        try original.write(to: tex, atomically: true, encoding: .utf8)
        setenv("FLASHTEX_CONTROLLER_LEDGER_ROOT", root.appendingPathComponent("ledger").path, 1)
        defer { unsetenv("FLASHTEX_CONTROLLER_LEDGER_ROOT") }

        let model = ShellModel()
        model.autoCompile = true
        XCTAssertEqual(model.openTex(at: tex), .opened)
        XCTAssertEqual(model.activePath, "main.tex", "the entry document keeps the fixed name")
        model.attachController(at: helper)
        defer { model.detachController() }
        let ready = await settles(15) { model.result?.revision == model.editorRevision && model.controllerState.durable["main.tex"] != nil && model.inFlightRevision == nil }
        XCTAssertTrue(ready, "initial preview never arrived: \(model.controllerStatus)")
        XCTAssertFalse(model.controllerRoutesFiles, "a session project never routes the open file's saves through the helper")

        let edited = "\\begin{document}\nA paper, edited.\n\\end{document}\n"
        model.updateActiveText(edited)
        XCTAssertTrue(model.isDirty)
        model.saveTexInteractive()
        let saved = await settles(10) { !model.isDirty || model.captureNote?.contains("failed") == true }
        XCTAssertTrue(saved, "the save never completed: \(model.captureNote ?? "-")")
        XCTAssertEqual(try String(contentsOf: tex, encoding: .utf8), edited, "the open file holds the buffer (note: \(model.captureNote ?? "-"))")
        XCTAssertFalse(model.isDirty, model.captureNote ?? "-")
        XCTAssertEqual(model.savedText, edited)
    }

    static let fakeHelper = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent().appendingPathComponent("Fixtures/fake_preview_controller.py")

    /// Finding 3 (the DocumentKinds use-after-free pattern, elsewhere):
    /// `ProjectDocuments` keeps an `unowned` back-reference to the model and
    /// its async helpers (`flushToHelper` polling the in-flight edit,
    /// `syncWithHelper`, `helperRequest`'s timeout) read it after awaits, while
    /// the app launches them from `Task`s that hold `ProjectDocuments`
    /// strongly (`switchDocument`, `armControllerTracking`, `init`). A model
    /// torn down during such a wait (tests do; a closed window would) left the
    /// task touching a freed object: a fatal unowned read that takes the whole
    /// process down. The fix keeps the model alive for the duration of the
    /// call, as `DocumentKinds.refresh` does.
    func testProjectDocumentsFlushSurvivesTheModelBeingReleasedMidWait() async throws {
        let root = FileManager.default.temporaryDirectory.appendingPathComponent("pc-review-unowned-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: root.appendingPathComponent("project"), withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }
        let tex = root.appendingPathComponent("project/main.tex")
        try "Hello\n".write(to: tex, atomically: true, encoding: .utf8)
        setenv("FLASHTEX_CONTROLLER_LEDGER_ROOT", root.appendingPathComponent("ledger").path, 1)
        defer { unsetenv("FLASHTEX_CONTROLLER_LEDGER_ROOT") }

        var model: ShellModel? = ShellModel()
        model!.autoCompile = true
        XCTAssertEqual(model!.openTex(at: tex), .opened)
        model!.attachController(at: Self.fakeHelper)
        let ready = await settles(10) { model!.controllerState.ready && model!.result?.revision == model!.editorRevision }
        XCTAssertTrue(ready, model!.controllerStatus)
        weak var weakModel = model
        let project = model!.project
        // An edit whose preview is slow to arrive: the flush polls the in-flight slot for it.
        model!.controllerState.inFlight = ("review-held", "main.tex", model!.editorRevision, Date(), model!.activeText, 99)
        model!.inFlightRevision = model!.editorRevision
        let flush = Task { @MainActor in await project.flushToHelper("main.tex", timeout: 0.5) }
        await Task.yield()
        await Task.yield()
        // The last strong reference goes while the flush is waiting.
        model = nil
        let flushed = await flush.value
        XCTAssertFalse(flushed, "the held edit never became durable within the bounded wait")
        // Run-loop blocks queued by the helper client may hold the model for a turn.
        let released = await settles(2) { weakModel == nil }
        XCTAssertTrue(released, "nothing retains the model once the flush has returned")
    }

    /// Polls `cond` on the main actor until it holds or `timeout` elapses.
    private func settles(_ timeout: TimeInterval, _ cond: () -> Bool) async -> Bool {
        let start = Date()
        while !cond() {
            if Date().timeIntervalSince(start) > timeout { return false }
            try? await Task.sleep(nanoseconds: 30_000_000)
        }
        return true
    }
}
