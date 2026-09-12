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

    /// Gate 3 evidence: the shell requests `rules-v1`/`font-hints-v1` by default.
    /// Whether the real compiler accepts them is observed, never assumed — if it
    /// does not, the shell must say so explicitly and still render legacy output.
    func testLayoutCapabilityNegotiationAgainstRealWorkerIsExplicit() async throws {
        guard let binary = Self.binary, FileManager.default.isExecutableFile(atPath: binary.path) else {
            throw XCTSkip("set FLASHTEX_COMPILER to the built flashtex-compiler binary")
        }
        let model = ShellModel()
        model.attachWorker(at: binary)
        model.autoCompile = false
        let requested = model.requestedLayoutCapabilities
        XCTAssertEqual(requested, ShellModel.defaultLayoutCapabilities())
        XCTAssertFalse(requested.isEmpty, "gate 3: capabilities are requested by default (override with FLASHTEX_LAYOUT_CAPABILITIES)")

        model.updateActiveText("\\section{Caps}\nInline $\\frac{1}{2}$ fraction and \\textbf{bold}.\n")
        model.compile()
        try await waitUntil { model.inFlightRevision == nil }
        let result = try XCTUnwrap(model.result)
        XCTAssertFalse(model.isFixture)
        XCTAssertFalse(model.workerStatus.contains("protocol violation"), model.workerStatus)
        XCTAssertEqual(model.negotiation.requested, requested, "bound to the request that produced it")
        let accepted = model.negotiation.accepted
        print("REAL-COMPILER CAPABILITIES: requested=\(requested) accepted=\(accepted.isEmpty ? ["<none>"] : accepted) notes=\(model.capabilityNotes)")

        let hasRule = result.pages.flatMap(\.items).contains { if case .rule = $0 { true } else { false } }
        let hasHint = result.pages.flatMap(\.items).contains { if case .text(let t) = $0 { t.font != nil } else { false } }
        for cap in requested {
            if accepted.contains(cap) {
                XCTAssertFalse(model.capabilityNotes.contains("capability \(cap) not accepted by the worker"))
            } else {
                XCTAssertTrue(model.capabilityNotes.contains("capability \(cap) not accepted by the worker"),
                              "non-acceptance of \(cap) must be reported explicitly: \(model.capabilityNotes)")
            }
        }
        if !accepted.contains(RuntimeV1.LayoutCapabilities.rulesV1) {
            XCTAssertFalse(hasRule, "a worker that did not accept rules-v1 may not emit rules (the shell would have rejected it)")
        }
        if !accepted.contains(RuntimeV1.LayoutCapabilities.fontHintsV1) {
            XCTAssertFalse(hasHint)
        }
        if accepted.isEmpty {
            // Legacy route: the fraction bar arrives as a U+2500 text run and is
            // approximated; unknown kinds would be skipped silently.
            XCTAssertEqual(model.negotiation.isNegotiated, false)
            XCTAssertTrue(model.layoutDiagnostics.isEmpty)
        }
        // Either way the result still exports.
        XCTAssertTrue(PDFExport.render(result).starts(with: Array("%PDF".utf8)))

        // Opting out sends no field and yields the legacy negotiation with no notes.
        model.requestedLayoutCapabilities = []
        model.updateActiveText("Legacy request.\n")
        model.compile()
        try await waitUntil { model.inFlightRevision == nil }
        XCTAssertEqual(model.negotiation, .legacy)
        XCTAssertTrue(model.capabilityNotes.isEmpty, "\(model.capabilityNotes)")
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
