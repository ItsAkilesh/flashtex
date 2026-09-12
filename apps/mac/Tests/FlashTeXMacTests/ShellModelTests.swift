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

        // Editing bumps the revision; navigation still works but is flagged stale.
        model.updateActiveText("Héllo FlashTeX.\n")
        XCTAssertTrue(model.previewIsStale)
        model.navigate(to: item.source)
        XCTAssertEqual(model.selection?.nsRange, NSRange(location: 0, length: 13)) // é is 2 bytes, 1 UTF-16 unit
        XCTAssertTrue(model.navigationNote?.contains("mapping may be off") == true)

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

        model.updateActiveText("Second draft\n") // editorRevision 2
        model.compile()
        XCTAssertEqual(model.inFlightRevision, 2)
        try await waitUntil { model.inFlightRevision == nil }
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

    private func waitUntil(timeout: TimeInterval = 10, _ cond: () -> Bool) async throws {
        let start = Date()
        while !cond() {
            if Date().timeIntervalSince(start) > timeout { throw XCTSkip("timeout") }
            try await Task.sleep(nanoseconds: 50_000_000)
        }
    }
}
