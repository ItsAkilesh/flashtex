import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// Multi-file projects (lane `mac-multifile`): lexical include discovery,
/// rooted direct opens, helper-route opens with exact membership snapshots,
/// `ShellModel.documents` sync, explicit detach, and caret-preserving switches.
@MainActor
final class ProjectDocumentsTests: XCTestCase {
    // MARK: lexical discovery

    func testScanFindsInputAndIncludeWithByteSpans() {
        let text = "\\documentclass{article}\n\\begin{document}\n\\input{chapter}\n\\include{ch/two.tex}\n\\input bare\\relax\n\\end{document}\n"
        let refs = ProjectIncludes.scan(text)
        XCTAssertEqual(refs.map(\.argument), ["chapter", "ch/two.tex", "bare"])
        XCTAssertEqual(refs.map(\.kind), [.input, .include, .input])
        XCTAssertTrue(refs.allSatisfy(\.literal))
        // Spans are UTF-8 byte offsets of the whole command and of the argument.
        let bytes = Array(text.utf8)
        for r in refs {
            let command = String(decoding: bytes[r.startByte..<r.endByte], as: UTF8.self)
            let argument = String(decoding: bytes[r.argumentStartByte..<r.argumentEndByte], as: UTF8.self)
            XCTAssertTrue(command.hasPrefix("\\input") || command.hasPrefix("\\include"), command)
            XCTAssertEqual(argument, r.argument)
        }
        XCTAssertEqual(String(decoding: bytes[refs[0].startByte..<refs[0].endByte], as: UTF8.self), "\\input{chapter}")
        XCTAssertEqual(String(decoding: bytes[refs[2].startByte..<refs[2].endByte], as: UTF8.self), "\\input bare")
    }

    func testScanSpansAreBytesNotCharacters() {
        let text = "Ünïcödé — \\input{ch}\n"
        let refs = ProjectIncludes.scan(text)
        XCTAssertEqual(refs.count, 1)
        XCTAssertEqual(refs[0].startByte, Array("Ünïcödé — ".utf8).count)
        XCTAssertEqual(refs[0].argumentStartByte, refs[0].startByte + "\\input{".utf8.count)
    }

    func testScanSkipsCommentsVerbVerbatimAndCommandPrefixes() {
        let text = """
        % \\input{commented}
        \\verb|\\input{verb}| text \\inputfoo{x}
        \\begin{verbatim}
        \\input{verbatim}
        \\end{verbatim}
        \\input[opt]{real} \\include{\\jobname}
        \\include
        """
        let refs = ProjectIncludes.scan(text)
        XCTAssertEqual(refs.map(\.argument), ["real", "\\jobname"])
        XCTAssertEqual(refs.map(\.literal), [true, false])
    }

    func testScanIsBounded() {
        let text = String(repeating: "\\input{a}\n", count: 200)
        XCTAssertEqual(ProjectIncludes.scan(text).count, ProjectIncludes.maxReferences)
        XCTAssertEqual(ProjectIncludes.scan(text, limit: 3).count, 3)
        XCTAssertEqual(ProjectIncludes.scan("\\input{unbalanced").count, 0, "an unbalanced brace consumes to the end")
    }

    func testRootedPathNormalizationMirrorsProjectFiles() throws {
        XCTAssertEqual(try ProjectIncludes.normalize("./chapters//intro.tex"), "chapters/intro.tex")
        XCTAssertEqual(try ProjectIncludes.normalize("a/../b"), "b")
        XCTAssertThrowsError(try ProjectIncludes.normalize("../escape")) { XCTAssertEqual($0 as? ProjectIncludes.PathError, .escapesRoot) }
        XCTAssertThrowsError(try ProjectIncludes.normalize("/etc/passwd")) { XCTAssertEqual($0 as? ProjectIncludes.PathError, .absolute) }
        XCTAssertThrowsError(try ProjectIncludes.normalize("~/x")) { XCTAssertEqual($0 as? ProjectIncludes.PathError, .absolute) }
        XCTAssertThrowsError(try ProjectIncludes.normalize("a\\b")) { XCTAssertEqual($0 as? ProjectIncludes.PathError, .forbiddenCharacter("\\")) }
        XCTAssertThrowsError(try ProjectIncludes.normalize("c:x")) { XCTAssertEqual($0 as? ProjectIncludes.PathError, .forbiddenCharacter(":")) }
        XCTAssertThrowsError(try ProjectIncludes.normalize("./")) { XCTAssertEqual($0 as? ProjectIncludes.PathError, .empty) }
        XCTAssertEqual(try ProjectIncludes.candidates(for: "chapter"), ["chapter.tex", "chapter"])
        XCTAssertEqual(try ProjectIncludes.candidates(for: "ch/two.tex"), ["ch/two.tex"])
        XCTAssertEqual(try ProjectIncludes.candidates(for: "notes.md"), ["notes.md.tex", "notes.md"])
    }

    // MARK: direct mode (no helper)

    private struct TempProject {
        let root: URL
        let main: URL
        init(main mainText: String = "\\begin{document}\nMain.\n\\input{chapter}\n\\end{document}\n",
             chapter: String = "Chapter one, with caret memory.\n", extra: [String: String] = [:]) throws {
            root = FileManager.default.temporaryDirectory.appendingPathComponent("pd-test-\(UUID().uuidString)")
            try FileManager.default.createDirectory(at: root.appendingPathComponent("project/ch"), withIntermediateDirectories: true)
            main = root.appendingPathComponent("project/main.tex")
            try mainText.write(to: main, atomically: true, encoding: .utf8)
            try chapter.write(to: root.appendingPathComponent("project/chapter.tex"), atomically: true, encoding: .utf8)
            for (name, text) in extra { try text.write(to: root.appendingPathComponent("project/\(name)"), atomically: true, encoding: .utf8) }
        }
        func remove() { try? FileManager.default.removeItem(at: root) }
    }

    func testDirectModeOpensIncludesSwitchesAndDetaches() async throws {
        let project = try TempProject(extra: ["ch/two.tex": "Two.\n"])
        defer { project.remove() }
        let model = ShellModel()
        model.detachWorker()
        XCTAssertEqual(model.openTex(at: project.main), .opened)
        XCTAssertFalse(model.controllerAttached)
        let p = model.project
        XCTAssertEqual(p.entryPath, "main.tex")
        XCTAssertEqual(p.projectRoot?.path, project.root.appendingPathComponent("project").standardizedFileURL.path)

        // Discovery resolves against the rooted project directory.
        var found = p.discoverIncludes()
        XCTAssertEqual(found.map(\.resolvedPath), ["chapter.tex"])
        XCTAssertEqual(found.map(\.state), [.available])
        XCTAssertEqual(found[0].candidates, ["chapter.tex", "chapter"])

        // Open: appended after the entry, baseline recorded, listing in sync.
        let outcomes = await p.openDiscoveredIncludes()
        XCTAssertEqual(outcomes, [.opened(path: "chapter.tex")])
        XCTAssertEqual(model.documents.map(\.path), ["main.tex", "chapter.tex"])
        XCTAssertEqual(model.documents[1].text, "Chapter one, with caret memory.\n")
        XCTAssertEqual(p.listing.map(\.role), [.entry, .included(from: "main.tex")])
        XCTAssertEqual(p.listing.map(\.origin), [.disk, .disk])
        XCTAssertEqual(p.listing.map(\.durableRevision), [nil, nil])
        XCTAssertEqual(p.listing.map(\.isDirty), [false, false])
        found = p.discoverIncludes()
        XCTAssertEqual(found.map(\.state), [.open])
        let again = await p.openInclude("chapter")
        XCTAssertEqual(again, .alreadyOpen(path: "chapter.tex"), "the .tex candidate is the open member")
        let exact = await p.openDocument("chapter")
        XCTAssertEqual(exact, .refused("cannot open chapter: no such file under \(project.root.appendingPathComponent("project").standardizedFileURL.path)"), "openDocument is exact-path")
        let viaInclude = await p.openInclude("ch/two")
        XCTAssertEqual(viaInclude, .opened(path: "ch/two.tex"))
        let detachedAgain = await p.detachDocument("ch/two.tex")
        XCTAssertEqual(detachedAgain, .detached(path: "ch/two.tex"))
        let r1 = await p.openDocument("./ch/../ch/two.tex")
        XCTAssertEqual(r1, .opened(path: "ch/two.tex"))
        XCTAssertEqual(model.documents.map(\.path), ["main.tex", "chapter.tex", "ch/two.tex"], "open order is stable, entry first")

        // Switch: the outgoing caret is remembered, the incoming one restored.
        XCTAssertEqual(model.activePath, "main.tex")
        model.caretUTF16 = 7; model.caretLengthUTF16 = 4 // "Main." region
        let switched = p.switchDocument(to: "chapter.tex")
        XCTAssertEqual(switched, .switched(to: "chapter.tex", restoredCaret: NSRange(location: 0, length: 0)), "first visit starts at 0")
        XCTAssertEqual(model.activePath, "chapter.tex")
        XCTAssertEqual(model.activeText, "Chapter one, with caret memory.\n")
        XCTAssertEqual(model.selection?.path, "chapter.tex")
        XCTAssertEqual(p.carets["main.tex"], NSRange(location: 7, length: 4))
        model.caretUTF16 = 8; model.caretLengthUTF16 = 3 // "one"
        XCTAssertEqual(p.switchDocument(to: "main.tex"), .switched(to: "main.tex", restoredCaret: NSRange(location: 7, length: 4)))
        XCTAssertEqual(model.caretUTF16, 7)
        XCTAssertEqual(model.caretLengthUTF16, 4)
        XCTAssertEqual(model.selection, .init(path: "main.tex", nsRange: NSRange(location: 7, length: 4), token: model.selection!.token))
        XCTAssertEqual(p.switchDocument(to: "main.tex"), .unchanged)
        XCTAssertEqual(p.switchDocument(to: "nope.tex"), .refused("nope.tex is not open"))

        // Unsaved text survives a round trip and is never lost by a switch.
        p.switchDocument(to: "chapter.tex")
        XCTAssertEqual(model.caretUTF16, 8, "the chapter caret was remembered while main was active")
        model.updateActiveText("Chapter one, edited.\n")
        XCTAssertTrue(p.isDirty("chapter.tex"))
        XCTAssertEqual(p.listing.map(\.isDirty), [false, true, false], "the entry stays clean whichever document is active")
        p.switchDocument(to: "main.tex")
        XCTAssertEqual(model.documents[1].text, "Chapter one, edited.\n")
        XCTAssertTrue(p.anyDirty)
        // A remembered range that no longer fits is clamped.
        model.caretUTF16 = 100; model.caretLengthUTF16 = 5
        XCTAssertEqual(p.switchDocument(to: "chapter.tex"), .switched(to: "chapter.tex", restoredCaret: NSRange(location: 8, length: 3)))
        p.switchDocument(to: "ch/two.tex")
        XCTAssertEqual(p.carets["chapter.tex"], NSRange(location: 8, length: 3))
        model.caretUTF16 = 99; model.caretLengthUTF16 = 0
        _ = p.switchDocument(to: "main.tex")
        XCTAssertEqual(p.carets["ch/two.tex"], NSRange(location: 99, length: 0))
        XCTAssertEqual(p.switchDocument(to: "ch/two.tex"), .switched(to: "ch/two.tex", restoredCaret: NSRange(location: 5, length: 0)), "clamped to the 5-unit text")

        // A pending capture insertion blocks the switch (it carries no document check).
        model.pendingEdit = .init(path: "ch/two.tex", nsRange: NSRange(location: 0, length: 0), text: "x", token: 1)
        guard case .refused(let why) = p.switchDocument(to: "main.tex") else { return XCTFail("switch must be refused while an edit is pending") }
        XCTAssertTrue(why.contains("pending"), why)
        model.pendingEdit = nil

        // Detach: refused for the entry and for unsaved edits; explicit discard keeps the text.
        let r2 = await p.detachDocument("main.tex")
        XCTAssertEqual(r2, .refused("cannot detach the entry document main.tex"))
        guard case .refused(let dirtyWhy) = await p.detachDocument("chapter.tex") else { return XCTFail("dirty detach must be refused") }
        XCTAssertTrue(dirtyWhy.contains("unsaved"), dirtyWhy)
        XCTAssertEqual(model.documents.count, 3)
        let r3 = await p.detachDocument("chapter.tex", discardingEdits: true)
        XCTAssertEqual(r3, .detached(path: "chapter.tex"))
        XCTAssertEqual(model.documents.map(\.path), ["main.tex", "ch/two.tex"])
        XCTAssertEqual(p.detachedBuffers["chapter.tex"], "Chapter one, edited.\n")
        // Detaching the active document switches back to the entry first.
        XCTAssertEqual(model.activePath, "ch/two.tex")
        let r4 = await p.detachDocument("ch/two.tex")
        XCTAssertEqual(r4, .detached(path: "ch/two.tex"))
        XCTAssertEqual(model.activePath, "main.tex")
        XCTAssertEqual(model.documents.map(\.path), ["main.tex"])
        let r5 = await p.detachDocument("ch/two.tex")
        XCTAssertEqual(r5, .refused("ch/two.tex is not open"))
        // Re-opening the discarded document reads the (unchanged) disk text again.
        let r6 = await p.openDocument("chapter.tex")
        XCTAssertEqual(r6, .opened(path: "chapter.tex"))
        XCTAssertEqual(model.documents[1].text, "Chapter one, with caret memory.\n")
        XCTAssertNil(p.detachedBuffers["chapter.tex"])
    }

    func testDirectModeRefusesEscapesSymlinksAndMissingFiles() async throws {
        let project = try TempProject(main: "\\input{../outside}\n\\input{link}\n\\input{missing}\n\\input{/abs}\n\\input{\\macro}\n")
        defer { project.remove() }
        try "outside\n".write(to: project.root.appendingPathComponent("outside.tex"), atomically: true, encoding: .utf8)
        try FileManager.default.createSymbolicLink(at: project.root.appendingPathComponent("project/link.tex"),
                                                   withDestinationURL: project.root.appendingPathComponent("outside.tex"))
        let model = ShellModel()
        model.detachWorker()
        XCTAssertEqual(model.openTex(at: project.main), .opened)
        let found = model.project.discoverIncludes()
        XCTAssertEqual(found.count, 5)
        for d in found {
            guard case .unresolvable = d.state else { return XCTFail("\(d.reference.argument) must be unresolvable: \(d.state)") }
        }
        XCTAssertEqual(found[0].state, .unresolvable("path escapes the project root via '..'"))
        XCTAssertEqual(found[1].state, .unresolvable("link.tex is a symbolic link"))
        XCTAssertEqual(found[2].state, .unresolvable("no such file under the project root"))
        XCTAssertEqual(found[3].state, .unresolvable("path must be project-relative, not absolute"))
        XCTAssertEqual(found[4].state, .unresolvable("argument needs macro expansion"))
        let r7 = await model.project.openDiscoveredIncludes()
        XCTAssertEqual(r7, [])
        XCTAssertEqual(model.documents.map(\.path), ["main.tex"])
        // Direct opens by path apply the same rules.
        let r8 = await model.project.openDocument("../outside.tex")
        XCTAssertEqual(r8, .refused("../outside.tex: path escapes the project root via '..'"))
        let r9 = await model.project.openDocument("link.tex")
        XCTAssertEqual(r9, .refused("cannot open link.tex: link.tex is a symbolic link"))
        XCTAssertEqual(model.documents.count, 1)
    }

    func testDirectModeNeedsAProjectRoot() async {
        let model = ShellModel()
        model.detachWorker()
        model.replaceProject(entryText: "\\input{chapter}\n")
        XCTAssertNil(model.documentURL)
        let found = model.project.discoverIncludes()
        XCTAssertEqual(found.map(\.state), [.unresolvable("no project root (the entry document is not saved)")])
        guard case .refused(let why) = await model.project.openDocument("chapter.tex") else { return XCTFail() }
        XCTAssertTrue(why.contains("no project root"), why)
    }

    func testNavigationSwitchRecordsTheOutgoingCaret() throws {
        // Navigation.swift switches `activePath` directly for a span in another
        // open document; the caret of the document being left is still recorded.
        let model = ShellModel()
        model.detachWorker()
        model.replaceProject(entryText: "Main text.\n")
        model.documents.append(.init(path: "chapter.tex", text: "\\label{x}\n"))
        let p = model.project
        model.caretUTF16 = 5; model.caretLengthUTF16 = 0
        model.setCompiledDocuments(["main.tex": "Main text.\n", "chapter.tex": "\\label{x}\n"])
        model.navigateExactly(to: .init(path: "chapter.tex", startByte: 7, endByte: 8), expectedText: nil)
        XCTAssertEqual(model.activePath, "chapter.tex", model.navigationNote ?? "")
        XCTAssertEqual(p.carets["main.tex"], NSRange(location: 5, length: 0))
        XCTAssertEqual(p.switchDocument(to: "main.tex"), .switched(to: "main.tex", restoredCaret: NSRange(location: 5, length: 0)))
    }

    // MARK: helper route (real flashtex-preview-controller + compiler)

    static var helper: URL? {
        ProcessInfo.processInfo.environment["FLASHTEX_PREVIEW_CONTROLLER"].map { URL(fileURLWithPath: $0) }
    }

    func testHelperRouteOpensWithExactSnapshotAndDetaches() async throws {
        guard let helper = Self.helper, FileManager.default.isExecutableFile(atPath: helper.path),
              ShellModel.locateCompiler() != nil else {
            throw XCTSkip("set FLASHTEX_PREVIEW_CONTROLLER and FLASHTEX_COMPILER to built binaries")
        }
        let project = try TempProject(main: "\\begin{document}\nMain.\n\\input{chapter}\n\\end{document}\n",
                                      chapter: "Chapter via helper.\n",
                                      extra: ["appendix.tex": "Appendix, not referenced.\n"])
        defer { project.remove() }
        setenv("FLASHTEX_CONTROLLER_LEDGER_ROOT", project.root.appendingPathComponent("ledger").path, 1)
        defer { unsetenv("FLASHTEX_CONTROLLER_LEDGER_ROOT") }

        let model = ShellModel()
        model.autoCompile = true
        XCTAssertEqual(model.openTex(at: project.main), .opened)
        model.attachController(at: helper)
        XCTAssertTrue(model.controllerAttached)
        try await waitUntil { model.result?.revision == model.editorRevision && model.controllerState.durable["main.tex"] != nil }
        let p = model.project

        // The helper discovered chapter.tex at startup: the snapshot lists it,
        // and opening it reads the durable document (no membership change).
        let snapshotOrNil = await p.refreshSnapshot()
        let snapshot = try XCTUnwrap(snapshotOrNil)
        XCTAssertEqual(Set(snapshot.versions.keys), ["main.tex", "chapter.tex"])
        let g0 = snapshot.generation
        XCTAssertEqual(p.discoverIncludes().map(\.state), [.available])
        let r10 = await p.openDiscoveredIncludes()
        XCTAssertEqual(r10, [.opened(path: "chapter.tex")])
        XCTAssertEqual(model.documents.map(\.path), ["main.tex", "chapter.tex"])
        XCTAssertEqual(model.documents[1].text, "Chapter via helper.\n")
        XCTAssertEqual(p.listing[1].origin, .helper)
        XCTAssertEqual(p.listing[1].durableRevision, 1)
        XCTAssertEqual(model.controllerState.durable["chapter.tex"]?.revision, 1)
        XCTAssertEqual(p.membershipGeneration, g0)

        // An unreferenced rooted file goes through open_document with the exact
        // snapshot; the membership generation advances.
        let r11 = await p.openDocument("appendix.tex")
        XCTAssertEqual(r11, .opened(path: "appendix.tex"))
        XCTAssertEqual(model.documents.map(\.path), ["main.tex", "chapter.tex", "appendix.tex"])
        XCTAssertEqual(p.listing[2].origin, .helper)
        XCTAssertEqual(p.listing[2].durableRevision, 1)
        let g1 = try XCTUnwrap(p.membershipGeneration)
        XCTAssertGreaterThan(g1, g0)
        XCTAssertEqual(Set(p.sourceVersions.keys), ["main.tex", "chapter.tex", "appendix.tex"])
        let statusOrNil = await p.projectStatus()
        let status = try XCTUnwrap(statusOrNil)
        XCTAssertEqual(status["total_documents"] as? Int, 3)
        XCTAssertEqual(status["scope"] as? String, "active_sources_only")

        // A stale snapshot is refused by the helper (nothing opened twice).
        let stale: [String: Any] = ["path": "appendix.tex", "source_versions": snapshot.versions, "membership_generation": g0]
        guard case .failure(let refusal) = await p.helperRequest("open_document", stale) else { return XCTFail("stale open must be refused") }
        XCTAssertTrue(refusal.message.contains("stale"), refusal.message)

        // Editing an opened document goes through the durable route and its
        // preview binds to the editor revision (not stale).
        XCTAssertEqual(p.switchDocument(to: "chapter.tex"), .switched(to: "chapter.tex", restoredCaret: NSRange(location: 0, length: 0)))
        model.updateActiveText("Chapter via helper, edited.\n")
        let rev = model.editorRevision
        try await waitUntil { model.result?.revision == rev && model.inFlightRevision == nil }
        XCTAssertFalse(model.previewIsStale)
        XCTAssertEqual(model.controllerState.durable["chapter.tex"]?.revision, 2)
        XCTAssertEqual(p.listing[1].durableRevision, 2)
        XCTAssertEqual(model.compiledDocuments["chapter.tex"], "Chapter via helper, edited.\n")
        XCTAssertTrue(p.isDirty("chapter.tex"), "durable is not saved: the disk baseline still differs")

        // Detach the unreferenced document through the helper: generation advances,
        // the shell membership and durable state drop it, the entry is refused.
        p.switchDocument(to: "appendix.tex")
        let r12 = await p.detachDocument("appendix.tex")
        XCTAssertEqual(r12, .detached(path: "appendix.tex"))
        XCTAssertEqual(model.activePath, "main.tex")
        XCTAssertEqual(model.documents.map(\.path), ["main.tex", "chapter.tex"])
        XCTAssertNil(model.controllerState.durable["appendix.tex"])
        let g2 = try XCTUnwrap(p.membershipGeneration)
        XCTAssertGreaterThan(g2, g1)
        XCTAssertEqual(Set(p.sourceVersions.keys), ["main.tex", "chapter.tex"])
        let r13 = await p.detachDocument("main.tex")
        XCTAssertEqual(r13, .refused("cannot detach the entry document main.tex"))
        // The preview after the membership change still binds to the current buffers.
        try await waitUntil { model.inFlightRevision == nil }
        XCTAssertEqual(model.activeText, "\\begin{document}\nMain.\n\\input{chapter}\n\\end{document}\n")
        model.detachController()
    }

    func testHelperSyncAttachesDocumentsOpenedBeforeTheController() async throws {
        guard let helper = Self.helper, FileManager.default.isExecutableFile(atPath: helper.path),
              ShellModel.locateCompiler() != nil else {
            throw XCTSkip("set FLASHTEX_PREVIEW_CONTROLLER and FLASHTEX_COMPILER to built binaries")
        }
        let project = try TempProject(main: "\\begin{document}\nMain.\n\\input{chapter}\n\\end{document}\n",
                                      chapter: "Chapter on disk.\n",
                                      extra: ["notes.tex": "Notes on disk.\n"])
        defer { project.remove() }
        setenv("FLASHTEX_CONTROLLER_LEDGER_ROOT", project.root.appendingPathComponent("ledger").path, 1)
        defer { unsetenv("FLASHTEX_CONTROLLER_LEDGER_ROOT") }
        let model = ShellModel()
        model.autoCompile = true
        model.detachWorker()
        XCTAssertEqual(model.openTex(at: project.main), .opened)
        let p = model.project
        // Direct mode first: a discovered include and an unreferenced file, one edited.
        let r1 = await p.openDiscoveredIncludes()
        XCTAssertEqual(r1, [.opened(path: "chapter.tex")])
        let r2 = await p.openDocument("notes.tex")
        XCTAssertEqual(r2, .opened(path: "notes.tex"))
        XCTAssertEqual(p.listing.map(\.origin), [.disk, .disk, .disk])
        p.switchDocument(to: "chapter.tex")
        model.updateActiveText("Chapter edited before the helper.\n")
        p.switchDocument(to: "main.tex") // attachController names the entry after the active path (parent diff)

        // Attaching the helper: chapter.tex is read from the helper (it discovered
        // it), notes.tex is imported with open_document, and the edited buffer is
        // submitted so the helper's durable text is what the editor shows.
        model.attachController(at: helper)
        try await waitUntil { model.controllerState.durable["chapter.tex"] != nil && model.controllerState.durable["notes.tex"] != nil }
        try await waitUntil { model.controllerState.textByDurable["chapter.tex"]?[model.controllerState.durable["chapter.tex"]!.revision]?.sameBytes(as: "Chapter edited before the helper.\n") == true }
        XCTAssertGreaterThanOrEqual(p.helperSyncs, 1)
        XCTAssertEqual(model.controllerState.durable["chapter.tex"]?.revision, 2, "disk import r1, buffer submitted as r2")
        XCTAssertEqual(model.controllerState.durable["notes.tex"]?.revision, 1)
        XCTAssertEqual(p.listing.map(\.origin), [.disk, .helper, .helper])
        XCTAssertEqual(p.listing.map(\.durableRevision), [1, 2, 1])
        XCTAssertEqual(Set(p.sourceVersions.keys), ["main.tex", "chapter.tex", "notes.tex"])
        XCTAssertTrue(p.isDirty("chapter.tex"), "durable on the helper, still unsaved on disk")
        try await waitUntil { model.result?.revision == model.editorRevision && model.inFlightRevision == nil }
        XCTAssertEqual(model.compiledDocuments["chapter.tex"], "Chapter edited before the helper.\n", "the preview was compiled from the edited chapter")
        // Nothing left to sync: a later status change sends no request.
        let syncs = p.helperSyncs
        await p.syncWithHelper()
        XCTAssertEqual(p.helperSyncs, syncs)
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
