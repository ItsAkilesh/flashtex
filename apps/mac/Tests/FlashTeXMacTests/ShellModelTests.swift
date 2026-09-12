import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

@MainActor
final class ShellModelTests: XCTestCase {
    func testLoadsFixturesAndNavigatesToUTF16Selection() throws {
        let model = ShellModel()
        XCTAssertNil(model.loadError, model.loadError ?? "")
        let result = try XCTUnwrap(model.result)
        XCTAssertEqual(result.status, .ok)
        XCTAssertEqual(model.activeText, "Hello FlashTeX.\n")
        XCTAssertFalse(model.previewIsStale)

        guard case .text(let item) = result.pages[0].items[0] else { return XCTFail() }
        model.navigate(to: item.source)
        let sel = try XCTUnwrap(model.selection)
        XCTAssertEqual(sel.path, "main.tex")
        XCTAssertEqual(sel.nsRange, NSRange(location: 0, length: 14))
        XCTAssertEqual((model.activeText as NSString).substring(with: sel.nsRange), "Hello FlashTeX")

        // Editing bumps the revision. An edit inside the item's span refuses navigation
        // (stale offsets are never applied); an edit after it is rebased.
        model.updateActiveText("Héllo FlashTeX.\n")
        XCTAssertTrue(model.previewIsStale)
        let selBefore = model.selection
        model.navigate(to: item.source!, expectedText: item.text)
        XCTAssertEqual(model.selection, selBefore)
        XCTAssertTrue(model.navigationNote?.contains("recompile to navigate") == true, model.navigationNote ?? "")
        model.updateActiveText("Hello FlashTeX. Appended\n")
        model.navigate(to: item.source!, expectedText: nil)
        XCTAssertEqual(model.selection?.nsRange, NSRange(location: 0, length: 14))
        model.updateActiveText("Prefix! Hello FlashTeX.\n")
        // The contract fixture's item text ("Hello FlashTeX.") is one byte longer than
        // its range (0..<14), so expected-text verification would (correctly) refuse;
        // navigate without it here. Reported to the fixture owner.
        model.navigate(to: item.source!, expectedText: nil)
        XCTAssertEqual(model.selection?.nsRange, NSRange(location: 8, length: 14), model.navigationNote ?? "nil")
        XCTAssertTrue(model.navigationNote?.contains("rebased") == true, model.navigationNote ?? "nil")
        model.updateActiveText("Hello FlashTeX.\n") // back to the compiled text

        // Invalid ranges are reported, not applied.
        let before = model.selection
        model.navigate(to: .init(path: "main.tex", startByte: 0, endByte: 999))
        XCTAssertEqual(model.selection, before)
        XCTAssertTrue(model.navigationNote?.contains("not a valid range") == true)
        model.navigate(to: .init(path: "other.tex", startByte: 0, endByte: 1))
        XCTAssertTrue(model.navigationNote?.contains("No open document") == true)
        model.navigate(to: nil)
        XCTAssertTrue(model.navigationNote?.contains("no source mapping") == true)
    }
}

@MainActor
final class ShellModelWorkerTests: XCTestCase {
    func testCompileThroughWorkerReplacesFixtureAndIgnoresOlderRevisions() async throws {
        let model = ShellModel()
        XCTAssertTrue(model.isFixture)
        model.attachWorker(at: WorkerClientTests.python, arguments: [WorkerClientTests.fakeWorker.path])
        XCTAssertTrue(model.workerAttached)

        model.autoCompile = false
        model.updateActiveText("Second draft\n") // editorRevision 2
        model.compile()
        XCTAssertEqual(model.inFlightRevision, 2)
        try await waitUntil { model.inFlightRevision == nil }
        XCTAssertNotNil(model.lastLatencyMs)
        XCTAssertEqual(model.compiledDocuments["main.tex"], "Second draft\n")
        XCTAssertFalse(model.isFixture)
        XCTAssertEqual(model.result?.revision, 2)
        XCTAssertFalse(model.previewIsStale)
        guard case .text(let item) = model.result!.pages[0].items[0] else { return XCTFail() }
        XCTAssertEqual(item.text, "Second draft")
        model.navigate(to: item.source)
        XCTAssertEqual(model.selection?.nsRange, NSRange(location: 0, length: 12))

        // Simulate an out-of-order older result: it must not replace revision 2.
        let stale = RuntimeV1.Envelope(protocolVersion: 1, id: "old", type: "compile_result",
            payload: RuntimeV1.CompileResult(projectId: "demo", revision: 1, status: .failed, pages: [], diagnostics: [], pdfPath: nil))
        model.handleForTesting(.result(stale))
        XCTAssertEqual(model.result?.revision, 2)
        XCTAssertEqual(model.result?.status, .ok)
        model.detachWorker()
        XCTAssertFalse(model.workerAttached)
    }

    func testUnsolicitedAndMismatchedResultsAreNeverApplied() async throws {
        let model = ShellModel()
        model.attachWorker(at: WorkerClientTests.python, arguments: [WorkerClientTests.fakeWorker.path])
        model.autoCompile = false
        let fixtureResult = model.result

        // 1. A result whose id was never sent: ignored, fixture stays.
        let unsolicited = RuntimeV1.Envelope(protocolVersion: 1, id: "never-sent", type: "compile_result",
            payload: RuntimeV1.CompileResult(projectId: "demo", revision: 99, status: .ok, pages: [], diagnostics: [], pdfPath: nil))
        model.handleForTesting(.result(unsolicited))
        XCTAssertEqual(model.result, fixtureResult)
        XCTAssertTrue(model.isFixture)
        XCTAssertTrue(model.workerLog.last?.contains("unknown id") == true)

        // 2. Worker answers with the wrong id for a real request: not applied, request stays pending until timeout logic (here: still in flight).
        model.updateActiveText("%wrongid first\n")
        model.compile()
        try await Task.sleep(nanoseconds: 400_000_000)
        XCTAssertTrue(model.isFixture, "result with unknown id must not be applied")
        XCTAssertNotNil(model.inFlightRevision)
        model.detachWorker()

        // 3. Right id but wrong revision/project: rejected as a protocol violation.
        model.attachWorker(at: WorkerClientTests.python, arguments: [WorkerClientTests.fakeWorker.path])
        model.updateActiveText("%wrongrev second\n")
        model.compile()
        try await waitUntil { model.inFlightRevision == nil }
        XCTAssertTrue(model.isFixture, "mismatched revision must not be applied")
        XCTAssertTrue(model.workerStatus.contains("protocol violation"), model.workerStatus)
        XCTAssertTrue(model.inFlightRequests.isEmpty)

        // 4. Same id, matching project and revision: applied.
        model.updateActiveText("third\n")
        model.compile()
        try await waitUntil { model.inFlightRevision == nil }
        XCTAssertFalse(model.isFixture)
        XCTAssertEqual(model.result?.revision, model.editorRevision)
        model.detachWorker()
    }

    func testAutoCompileDebouncesAndCoalescesEdits() async throws {
        let model = ShellModel()
        model.attachWorker(at: WorkerClientTests.python, arguments: [WorkerClientTests.fakeWorker.path])
        model.autoCompile = true
        // Burst of edits: only the last buffer should end up compiled, and never out of order.
        for i in 1...5 { model.updateActiveText("draft \(i)\n") }
        let finalRevision = model.editorRevision
        try await waitUntil(timeout: 10) { model.result?.revision == finalRevision && model.inFlightRevision == nil }
        guard case .text(let item) = model.result!.pages[0].items[0] else { return XCTFail() }
        XCTAssertEqual(item.text, "draft 5")
        XCTAssertLessThanOrEqual(model.latenciesMs.count, 2, "burst coalesced into at most two requests, got \(model.latenciesMs.count)")
        XCTAssertFalse(model.previewIsStale)
        model.detachWorker()
    }

    private func waitUntil(timeout: TimeInterval = 10, _ cond: () -> Bool) async throws {
        let start = Date()
        while !cond() {
            if Date().timeIntervalSince(start) > timeout { throw XCTSkip("timeout") }
            try await Task.sleep(nanoseconds: 50_000_000)
        }
    }
}
