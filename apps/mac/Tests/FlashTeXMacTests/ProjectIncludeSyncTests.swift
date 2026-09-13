import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// Issue #75: the entry keeps its real file name, included files load from
/// disk automatically, and membership/disk changes recompile.
@MainActor
final class ProjectIncludeSyncTests: XCTestCase {
    // MARK: pure logic

    func testEntryPathIsTheRealFileName() {
        XCTAssertEqual(ProjectIncludes.entryPath(for: URL(fileURLWithPath: "/tmp/hw/HW1.tex")), "HW1.tex")
        XCTAssertEqual(ProjectIncludes.entryPath(for: URL(fileURLWithPath: "/tmp/My Paper.tex")), "My Paper.tex")
        XCTAssertEqual(ProjectIncludes.entryPath(for: nil), "main.tex", "an unsaved buffer keeps the placeholder")
        XCTAssertEqual(ProjectIncludes.entryPath(for: URL(fileURLWithPath: "/tmp/a:b.tex")), "main.tex", "a name the rooted path rules refuse falls back")
    }

    func testScanFindsSubfileAlongsideInputAndInclude() {
        let refs = ProjectIncludes.scan("\\input{a}\\include{b}\\subfile{sections/c}\\subfileinclude{d}% \\subfile{e}\n")
        XCTAssertEqual(refs.map(\.kind), [.input, .include, .subfile])
        XCTAssertEqual(refs.map(\.argument), ["a", "b", "sections/c"])
        XCTAssertEqual(try ProjectIncludes.candidates(for: refs[2].argument), ["sections/c.tex", "sections/c"])
    }

    func testReloadDecision() {
        XCTAssertEqual(ProjectDocuments.reloadDecision(buffer: "x", baseline: "x", disk: "x"), .unchanged)
        XCTAssertEqual(ProjectDocuments.reloadDecision(buffer: "x", baseline: "x", disk: "y"), .reload)
        XCTAssertEqual(ProjectDocuments.reloadDecision(buffer: "edited", baseline: "x", disk: "y"), .keepEdits)
        XCTAssertEqual(ProjectDocuments.reloadDecision(buffer: "y", baseline: "x", disk: "y"), .unchanged, "an edit that matches disk is not a conflict")
        XCTAssertEqual(ProjectDocuments.reloadDecision(buffer: "x", baseline: nil, disk: "y"), .keepEdits, "no baseline: never overwrite")
    }

    func testMembershipChangeDefeatsTheUnchangedBufferShortcut() {
        let model = ShellModel()
        model.replaceProject(entryText: "\\input{a}\n", path: "HW1.tex")
        model.setCompiledDocuments(["HW1.tex": "\\input{a}\n"])
        XCTAssertFalse(model.projectMembershipChangedSinceResult)
        model.documents.append(.init(path: "a.tex", text: "A\n"))
        XCTAssertTrue(model.projectMembershipChangedSinceResult)
        model.setCompiledDocuments(["HW1.tex": "\\input{a}\n", "a.tex": "A\n"])
        XCTAssertFalse(model.projectMembershipChangedSinceResult)
        model.documents[1].text = "A reloaded\n"
        XCTAssertTrue(model.projectMembershipChangedSinceResult)
    }

    // MARK: direct route on disk

    private struct TempProject {
        let root: URL
        let entry: URL
        init() throws {
            root = FileManager.default.temporaryDirectory.appendingPathComponent("pis-test-\(UUID().uuidString)")
            try FileManager.default.createDirectory(at: root.appendingPathComponent("sections"), withIntermediateDirectories: true)
            entry = root.appendingPathComponent("HW1.tex")
            try "\\begin{document}\n\\input{sections/intro}\n\\subfile{sections/method}\n\\input{missing}\n\\end{document}\n"
                .write(to: entry, atomically: true, encoding: .utf8)
            try "\\section{Intro}\\label{sec:intro}\n\\input{sections/nested}\n".write(to: url("sections/intro.tex"), atomically: true, encoding: .utf8)
            try "Nested.\n".write(to: url("sections/nested.tex"), atomically: true, encoding: .utf8)
            try "\\section{Method}\n".write(to: url("sections/method.tex"), atomically: true, encoding: .utf8)
        }
        func url(_ path: String) -> URL { root.appendingPathComponent(path) }
        func remove() { try? FileManager.default.removeItem(at: root) }
    }

    func testOpeningAProjectLoadsItsIncludeClosureUnderTheRealEntryName() throws {
        let project = try TempProject()
        defer { project.remove() }
        let model = ShellModel()
        model.detachWorker()
        XCTAssertEqual(model.openTex(at: project.entry), .opened)
        XCTAssertEqual(model.documents.map(\.path), ["HW1.tex", "sections/intro.tex", "sections/nested.tex", "sections/method.tex"])
        XCTAssertEqual(model.activePath, "HW1.tex")
        XCTAssertEqual(model.project.entryPath, "HW1.tex")
        XCTAssertEqual(model.windowTitle, "HW1.tex")
        XCTAssertEqual(model.project.listing.map(\.isDirty), [false, false, false, false])
        XCTAssertEqual(model.project.discoverClosure().unresolvable.map(\.reference.argument), ["missing"], "a missing file stays reported, never invented")
        if ProcessInfo.processInfo.environment["FLASHTEX_NO_FILE_WATCH"] != "1" {
            XCTAssertEqual(Set(model.project.includeWatchers.keys), ["sections/intro.tex", "sections/nested.tex", "sections/method.tex"])
        }
        model.project.switchDocument(to: "sections/intro.tex")
        XCTAssertEqual(model.windowTitle, "sections/intro.tex — HW1.tex")
    }

    func testDiskChangeReloadsAnUneditedIncludeAndKeepsEdits() throws {
        let project = try TempProject()
        defer { project.remove() }
        let model = ShellModel()
        model.detachWorker()
        XCTAssertEqual(model.openTex(at: project.entry), .opened)
        let p = model.project

        // Unedited, not active: replaced in place; newly named includes load too.
        try "\\section{Method, revised}\n\\input{sections/extra}\n".write(to: project.url("sections/method.tex"), atomically: true, encoding: .utf8)
        try "Extra.\n".write(to: project.url("sections/extra.tex"), atomically: true, encoding: .utf8)
        p.includeChangedOnDisk("sections/method.tex")
        XCTAssertEqual(model.documents.first { $0.path == "sections/method.tex" }?.text, "\\section{Method, revised}\n\\input{sections/extra}\n")
        XCTAssertFalse(p.isDirty("sections/method.tex"))
        XCTAssertTrue(p.isOpen("sections/extra.tex"))

        // Unedited and active: taken as an edit (new revision).
        p.switchDocument(to: "sections/intro.tex")
        let revision = model.editorRevision
        try "\\section{Intro v2}\n".write(to: project.url("sections/intro.tex"), atomically: true, encoding: .utf8)
        XCTAssertEqual(p.reloadFromDisk("sections/intro.tex"), .reloaded)
        XCTAssertEqual(model.activeText, "\\section{Intro v2}\n")
        XCTAssertGreaterThan(model.editorRevision, revision)
        XCTAssertFalse(p.isDirty("sections/intro.tex"))

        // Edited: the buffer is kept and stays dirty.
        model.updateActiveText("my edit\n")
        try "\\section{Intro v3}\n".write(to: project.url("sections/intro.tex"), atomically: true, encoding: .utf8)
        XCTAssertEqual(p.reloadFromDisk("sections/intro.tex"), .keptEdits)
        XCTAssertEqual(model.activeText, "my edit\n")
        XCTAssertTrue(p.isDirty("sections/intro.tex"))
    }

    func testSaveAsRenamesAPlaceholderEntry() throws {
        let project = try TempProject()
        defer { project.remove() }
        let model = ShellModel()
        model.detachWorker()
        model.replaceProject(entryText: "Hello\n")
        XCTAssertEqual(model.activePath, "main.tex")
        model.documentURL = project.url("paper.tex")
        model.adoptEntryFileName()
        XCTAssertEqual(model.documents.map(\.path), ["paper.tex"])
        XCTAssertEqual(model.activePath, "paper.tex")
    }

    // MARK: recompile through a worker

    private func waitUntil(timeout: TimeInterval = 10, _ cond: () -> Bool) async throws {
        let start = Date()
        while !cond() {
            if Date().timeIntervalSince(start) > timeout { XCTFail("timeout"); return }
            try await Task.sleep(nanoseconds: 10_000_000)
        }
    }

    func testIncludesAreCompiledAndOpeningOrReloadingRecompiles() async throws {
        let project = try TempProject()
        defer { project.remove() }
        let model = ShellModel()
        model.attachWorker(at: WorkerClientTests.python, arguments: [WorkerClientTests.fakeWorker.path])
        XCTAssertEqual(model.openTex(at: project.entry), .opened)
        try await waitUntil { model.inFlightRevision == nil && model.compiledDocuments["sections/method.tex"] != nil }
        XCTAssertEqual(Set(model.compiledDocuments.keys), ["HW1.tex", "sections/intro.tex", "sections/nested.tex", "sections/method.tex"],
                       "the first compile already carries every include")
        XCTAssertEqual(model.result?.revision, model.editorRevision)

        // Detach then reopen from the sidebar: both recompile at the same revision.
        let detached = await model.project.detachDocument("sections/method.tex")
        XCTAssertEqual(detached, .detached(path: "sections/method.tex"))
        try await waitUntil { model.inFlightRevision == nil && model.compiledDocuments["sections/method.tex"] == nil }
        model.project.autoLoadIncludes = false // keep the edit-settle rediscovery from reopening it first
        let reopened = await model.project.openDocument("sections/method.tex", role: .included(from: "HW1.tex"))
        XCTAssertEqual(reopened, .opened(path: "sections/method.tex"))
        try await waitUntil { model.inFlightRevision == nil && model.compiledDocuments["sections/method.tex"] != nil }

        // A disk change of an unedited, inactive include is compiled.
        model.project.autoLoadIncludes = true
        try "Nested, changed.\n".write(to: project.url("sections/nested.tex"), atomically: true, encoding: .utf8)
        model.project.includeChangedOnDisk("sections/nested.tex")
        try await waitUntil { model.inFlightRevision == nil && model.compiledDocuments["sections/nested.tex"] == "Nested, changed.\n" }
        model.detachWorker()
    }
}
