import XCTest
@testable import FlashTeXMac

/// Autosave (owner: "autosave should be on by default"). Real time is
/// suppressed under XCTest (`ShellModel.autosaveSuppressedUnderTest`, same
/// reasoning as `PreviewHUD.lingerSuppressed`): these tests drive the write
/// through `flushPendingAutosave()` instead of waiting out `autosaveInterval`.
///
/// `EditorPreferences.shared` is a process-wide singleton over
/// `UserDefaults.standard`; `autosave` is restored in `tearDown` so this
/// suite never leaks a changed default into another test or a real launch.
@MainActor
final class AutosaveTests: XCTestCase {
    private var originalAutosave = true

    override func setUp() {
        super.setUp()
        originalAutosave = EditorPreferences.shared.autosave
    }

    override func tearDown() {
        EditorPreferences.shared.autosave = originalAutosave
        super.tearDown()
    }

    private func tempDir(_ tag: String) throws -> URL {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("flashtex-autosave-\(tag)-\(UUID().uuidString)")
            .resolvingSymlinksInPath()
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        addTeardownBlock { try? FileManager.default.removeItem(at: dir) }
        return dir
    }

    private func disk(_ url: URL) throws -> String { try String(contentsOf: url, encoding: .utf8) }

    func testDefaultOnWritesToDiskAfterFlush() throws {
        EditorPreferences.shared.autosave = true
        let dir = try tempDir("on")
        let url = dir.appendingPathComponent("paper.tex")
        try "one\n".write(to: url, atomically: true, encoding: .utf8)

        let model = ShellModel()
        model.files.policy = .disabled(reason: "test: no helper binary")
        XCTAssertEqual(model.openTex(at: url), .opened)

        model.updateActiveText("two (unsaved)\n")
        XCTAssertEqual(try disk(url), "one\n", "nothing written until the debounce fires")
        model.flushPendingAutosave()
        XCTAssertEqual(try disk(url), "two (unsaved)\n")
        XCTAssertFalse(model.isDirty)
    }

    func testExplicitlyOffNeverWritesInTheBackground() throws {
        EditorPreferences.shared.autosave = false
        let dir = try tempDir("off")
        let url = dir.appendingPathComponent("paper.tex")
        try "one\n".write(to: url, atomically: true, encoding: .utf8)

        let model = ShellModel()
        model.files.policy = .disabled(reason: "test: no helper binary")
        XCTAssertEqual(model.openTex(at: url), .opened)

        model.updateActiveText("two (unsaved)\n")
        model.flushPendingAutosave()
        XCTAssertEqual(try disk(url), "one\n", "an explicit off must stay off")
        XCTAssertTrue(model.isDirty, "the edit is still only in the buffer; Command-S still works")
    }

    /// A buffer with no file yet must never autosave: `saveTex()` falls back
    /// to `saveTexAs()` for a `nil` `documentURL`, which would pop a Save
    /// panel while the user is mid-keystroke.
    func testNeverSavesABufferWithNoFileYet() {
        EditorPreferences.shared.autosave = true
        let model = ShellModel()
        model.updateActiveText("some text with no file behind it\n")
        model.flushPendingAutosave() // must be a no-op, not a Save panel
        XCTAssertNil(model.documentURL)
        XCTAssertTrue(model.isDirty)
    }

    func testFlushWithNothingPendingIsANoOp() throws {
        let dir = try tempDir("idle")
        let url = dir.appendingPathComponent("paper.tex")
        try "one\n".write(to: url, atomically: true, encoding: .utf8)
        let model = ShellModel()
        model.files.policy = .disabled(reason: "test: no helper binary")
        XCTAssertEqual(model.openTex(at: url), .opened)
        model.flushPendingAutosave() // no edit since open: nothing dirty, nothing to write
        XCTAssertEqual(try disk(url), "one\n")
    }

    /// Opens `main.tex` (entry, `\input{chapter}`) and `chapter.tex` as a
    /// project member, the way the sidebar does, with no helper attached.
    private func openProject(_ tag: String) async throws -> (ShellModel, main: URL, chapter: URL) {
        let dir = try tempDir(tag)
        let main = dir.appendingPathComponent("main.tex"), chapter = dir.appendingPathComponent("chapter.tex")
        try "\\input{chapter}\n".write(to: main, atomically: true, encoding: .utf8)
        try "Chapter.\n".write(to: chapter, atomically: true, encoding: .utf8)
        let model = ShellModel()
        model.files.policy = .disabled(reason: "test: no helper binary")
        XCTAssertEqual(model.openTex(at: main), .opened)
        let opened = await model.project.openDocument("chapter.tex")
        XCTAssertEqual(opened, .opened(path: "chapter.tex"))
        return (model, main, chapter)
    }

    /// Owner: "the autosave isn't working". A non-entry project member was
    /// never autosaved (`scheduleAutosave` was scoped to the entry document).
    func testNonEntryMemberEditAutosaves() async throws {
        EditorPreferences.shared.autosave = true
        let (model, _, chapter) = try await openProject("member")
        model.project.switchDocument(to: "chapter.tex")
        XCTAssertEqual(model.activePath, "chapter.tex")
        model.updateActiveText("Chapter, edited.\n")
        XCTAssertEqual(try disk(chapter), "Chapter.\n")
        model.flushPendingAutosave()
        XCTAssertEqual(try disk(chapter), "Chapter, edited.\n")
        XCTAssertFalse(model.project.isDirty("chapter.tex"))
    }

    /// Typing in the entry and switching tabs before the debounce fires must
    /// not drop the pending save — nor may a later edit in another document
    /// cancel it.
    func testSwitchingDocumentsKeepsAPendingAutosave() async throws {
        EditorPreferences.shared.autosave = true
        let (model, main, chapter) = try await openProject("switch")
        model.updateActiveText("\\input{chapter}\n% entry edit\n")
        model.project.switchDocument(to: "chapter.tex")
        model.flushPendingAutosave()
        XCTAssertEqual(try disk(main), "\\input{chapter}\n% entry edit\n", "switching away must not drop the entry's pending autosave")
        XCTAssertFalse(model.project.isDirty("main.tex"))

        model.project.switchDocument(to: "main.tex")
        model.updateActiveText("\\input{chapter}\n% second entry edit\n")
        model.project.switchDocument(to: "chapter.tex")
        model.updateActiveText("Chapter, edited.\n") // re-arms the debounce from another document
        model.flushPendingAutosave()
        XCTAssertEqual(try disk(main), "\\input{chapter}\n% second entry edit\n")
        XCTAssertEqual(try disk(chapter), "Chapter, edited.\n")
    }

    /// Autosave goes through the normal conflict-checked save: a file changed
    /// on disk is never clobbered, and no panel is raised.
    func testAutosaveNeverClobbersAFileChangedOnDisk() async throws {
        EditorPreferences.shared.autosave = true
        let (model, main, chapter) = try await openProject("conflict")
        try "\\input{chapter}\n% external\n".write(to: main, atomically: true, encoding: .utf8)
        try "Chapter, external.\n".write(to: chapter, atomically: true, encoding: .utf8)
        model.updateActiveText("\\input{chapter}\n% ours\n")
        model.project.switchDocument(to: "chapter.tex")
        model.updateActiveText("Chapter, ours.\n")
        model.flushPendingAutosave()
        XCTAssertEqual(try disk(main), "\\input{chapter}\n% external\n")
        XCTAssertEqual(try disk(chapter), "Chapter, external.\n")
        XCTAssertTrue(model.project.isDirty("main.tex"))
        XCTAssertTrue(model.project.isDirty("chapter.tex"))
    }
}
