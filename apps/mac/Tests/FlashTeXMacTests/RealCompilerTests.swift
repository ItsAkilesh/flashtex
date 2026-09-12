import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// End-to-end check against the actual Rust worker (FT-002). Skipped unless
/// FLASHTEX_COMPILER points at a built `flashtex-compiler` binary, so `swift test`
/// stays hermetic on machines without the crate built.
@MainActor
final class RealCompilerTests: XCTestCase {
    static var binary: URL? {
        ProcessInfo.processInfo.environment["FLASHTEX_COMPILER"].map { URL(fileURLWithPath: $0) }
    }

    func testShellCompilesThroughRealWorkerAndNavigatesEverySpan() async throws {
        guard let binary = Self.binary, FileManager.default.isExecutableFile(atPath: binary.path) else {
            throw XCTSkip("set FLASHTEX_COMPILER to the built flashtex-compiler binary")
        }
        let model = ShellModel()
        model.attachWorker(at: binary)
        XCTAssertTrue(model.workerAttached)

        let source = "\\section{Intro}\nHello naïve FlashTeX — café. $x$ \\foo{bar}\n\nSecond paragraph.\n"
        model.updateActiveText(source)
        model.compile()
        try await waitUntil { model.inFlightRevision == nil }
        let result = try XCTUnwrap(model.result)
        XCTAssertFalse(model.isFixture)
        XCTAssertEqual(result.revision, model.editorRevision)
        XCTAssertEqual(result.projectId, "demo")
        XCTAssertFalse(result.pages.isEmpty)
        XCTAssertFalse(result.diagnostics.isEmpty, "math and \\foo should be diagnosed, not silently dropped")

        // Every item's UTF-8 span must convert to a UTF-16 selection that slices back to its text.
        var checked = 0
        for page in result.pages {
            for case .text(let item) in page.items {
                let src = try XCTUnwrap(item.source)
                model.navigate(to: src)
                let sel = try XCTUnwrap(model.selection, "navigation failed for \(item.text): \(model.navigationNote ?? "")")
                XCTAssertEqual((model.activeText as NSString).substring(with: sel.nsRange), item.text)
                checked += 1
            }
        }
        XCTAssertGreaterThan(checked, 5)
        // Diagnostics with a source range must navigate too.
        for d in result.diagnostics where d.source != nil {
            model.navigate(to: d.source)
            XCTAssertNotNil(model.selection)
            XCTAssertFalse(model.navigationNote?.contains("not a valid range") == true, model.navigationNote ?? "")
        }

        // Edit after the compiled text: spans before the edit still navigate (rebased and
        // verified against item text); an edit inside a span refuses navigation.
        let firstItems: [RuntimeV1.PageItem.TextItem] = result.pages.flatMap { $0.items }.compactMap { if case .text(let t) = $0 { t } else { nil } }
        let hello = try XCTUnwrap(firstItems.first { $0.text == "Hello" })
        let second = try XCTUnwrap(firstItems.first { $0.text == "Second" })
        model.autoCompile = false
        model.updateActiveText(source.replacingOccurrences(of: "Second paragraph.", with: "Edited paragraph."))
        model.navigate(to: hello.source!, expectedText: hello.text)
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "Hello")
        let before = model.selection
        model.navigate(to: second.source!, expectedText: second.text)
        XCTAssertEqual(model.selection, before, "span inside the edited region must not navigate")
        XCTAssertTrue(model.navigationNote?.contains("recompile") == true)
        model.updateActiveText("PREFIX " + source)
        model.navigate(to: second.source!, expectedText: second.text)
        XCTAssertEqual((model.activeText as NSString).substring(with: model.selection!.nsRange), "Second", "rebased across a prefix insertion")

        // Latency sample for the report (auto-compile burst, coalesced).
        model.autoCompile = true
        for i in 0..<10 {
            model.updateActiveText(source + "\nline \(i)\n")
            try await Task.sleep(nanoseconds: 60_000_000)
        }
        try await waitUntil { model.inFlightRevision == nil && model.result?.revision == model.editorRevision }
        let lat = model.latenciesMs.sorted()
        print("REAL-COMPILER LATENCY ms: n=\(lat.count) min=\(lat.first ?? 0) median=\(model.medianLatencyMs ?? 0) max=\(lat.last ?? 0)")
        XCTAssertFalse(lat.isEmpty)
        model.autoCompile = false

        // A second revision replaces the first; a repeat of the older one would be ignored.
        model.updateActiveText("Only one word\n")
        model.compile()
        try await waitUntil { model.inFlightRevision == nil }
        XCTAssertEqual(model.result?.revision, model.editorRevision)
        XCTAssertEqual(model.result?.status, .ok)
        model.detachWorker()
    }

    private func waitUntil(timeout: TimeInterval = 15, _ cond: () -> Bool) async throws {
        let start = Date()
        while !cond() {
            if Date().timeIntervalSince(start) > timeout { XCTFail("timeout waiting for worker"); return }
            try await Task.sleep(nanoseconds: 50_000_000)
        }
    }
}
