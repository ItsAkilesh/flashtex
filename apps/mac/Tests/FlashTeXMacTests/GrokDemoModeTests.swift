import XCTest
@testable import FlashTeXMac

/// TEMPORARY (demo): removed with the demo commit.
@MainActor
final class GrokDemoModeTests: XCTestCase {
    func testFlagGatesTheDemo() {
        XCTAssertFalse(GrokDemo.enabled([:]))
        XCTAssertTrue(GrokDemo.enabled(["FLASHTEX_DEMO_MODE": "1"]))
        XCTAssertEqual(GrokDemo.model(["FLASHTEX_GROK_MODEL": "grok-4.20-0309-non-reasoning"]), "grok-4.20-0309-non-reasoning")
    }

    func testCannedAnswerStreamsWithoutTheModel() async throws {
        let demo = GrokDemoModel.shared
        demo.messages = []
        ReduceMotion.override = true
        defer { ReduceMotion.override = nil }
        demo.ask("Explain this error: \\mathbb undefined", context: "", allowModel: false)
        XCTAssertTrue(demo.panelShown)
        for _ in 0..<200 where demo.busy { try await Task.sleep(nanoseconds: 10_000_000) }
        XCTAssertFalse(demo.busy)
        let reply = try XCTUnwrap(demo.messages.last)
        XCTAssertEqual(reply.origin, "demo:canned")
        XCTAssertTrue(reply.text.contains("amssymb"))
        XCTAssertEqual(reply.insertable, "\\usepackage{amssymb}")
        let tikz = GrokDemo.canned(intent: .convertTikZ, prompt: "").insert ?? ""
        XCTAssertTrue(tikz.hasPrefix("\\begin{tikzpicture}") && tikz.hasSuffix("\\end{tikzpicture}"))
    }
}
