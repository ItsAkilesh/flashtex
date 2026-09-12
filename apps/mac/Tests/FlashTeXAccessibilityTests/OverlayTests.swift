import XCTest
import SwiftUI
import FlashTeXProtocol
@testable import FlashTeXAccessibility

/// What can be checked about the SwiftUI attachment without VoiceOver or
/// Accessibility permission: the overlay's element order, labels, and frames.
final class OverlayTests: XCTestCase {
    func loadMultipage() throws -> RuntimeV1.CompileResult {
        let url = DocumentModelTests.samples.appendingPathComponent("multipage-result.json")
        return try RuntimeV1.decodeCompileResult(Data(contentsOf: url)).payload
    }

    func testOverlaySlotsFollowReadingOrderWithDescendingPriority() throws {
        let res = try loadMultipage()
        let overlay = AccessibilityOverlay(page: res.pages[0], totalPages: 2, scale: 0.5) { _, _ in }
        XCTAssertEqual(overlay.slots.map(\.element.text), ["Introduction", "A", "naïve", "approach", "fails."])
        XCTAssertEqual(overlay.slots.map(\.priority), [5, 4, 3, 2, 1])
        XCTAssertEqual(overlay.slots.map(\.id), [0, 1, 2, 3, 4], "ids are page item indices")
        XCTAssertEqual(overlay.pageLabel, "Page 1 of 2, 2 lines")
        XCTAssertEqual(AccessibilityOverlay(page: res.pages[1], scale: 1) { _, _ in }.pageLabel, "Page 2, 3 lines")
    }

    func testOverlayFramesTrackItemGeometryAndScale() throws {
        let res = try loadMultipage()
        guard case .text(let intro) = res.pages[0].items[0] else { return XCTFail() }
        let full = AccessibilityOverlay(page: res.pages[0], scale: 1) { _, _ in }
        let half = AccessibilityOverlay(page: res.pages[0], scale: 0.5) { _, _ in }
        let f = full.slots[0].frame, h = half.slots[0].frame
        XCTAssertEqual(f.minX, intro.xPt)
        XCTAssertLessThan(f.minY, intro.baselineYPt, "box starts above the baseline")
        XCTAssertGreaterThan(f.maxY, intro.baselineYPt, "and ends below it (descender)")
        XCTAssertGreaterThan(f.width, intro.fontSizePt * 4, "twelve glyphs of Times at 17 pt")
        XCTAssertEqual(h.minX, f.minX / 2, accuracy: 0.001)
        XCTAssertEqual(h.width, f.width / 2, accuracy: 0.001)
        XCTAssertEqual(h.height, f.height / 2, accuracy: 0.001)
        // A rule item gets the RuleConvention rectangle.
        let rule = try JSONDecoder().decode(RuntimeV1.PageItem.self, from: Data(
            "{\"kind\":\"text\",\"text\":\"──\",\"x_pt\":100,\"baseline_y_pt\":97.36,\"font_size_pt\":8.4}".utf8))
        var page = res.pages[0]
        page.items = [rule]
        let r = AccessibilityOverlay(page: page, scale: 2) { _, _ in }.slots[0].frame
        XCTAssertEqual(r.minX, 200)
        XCTAssertEqual(r.width, 2 * RuleConvention.advanceEm * 8.4 * 2, accuracy: 0.001, "2 segments × 0.5 em × 8.4 pt × scale 2")
        XCTAssertEqual(r.height, RuleConvention.thicknessEm * 8.4 * 2, accuracy: 0.001)
    }

    func testOverlayActionForwardsSourceAndText() throws {
        let res = try loadMultipage()
        var received: (RuntimeV1.SourceRange?, String?)?
        let overlay = AccessibilityOverlay(page: res.pages[1], scale: 1) { received = ($0, $1) }
        // The action closure is what VoiceOver's "Go to source" invokes; drive it directly.
        let slot = overlay.slots[1]
        overlay.onSelect(slot.element.source, slot.element.text)
        XCTAssertEqual(received?.0, .init(path: "main.tex", startByte: 115, endByte: 123))
        XCTAssertEqual(received?.1, "Résumé")
        XCTAssertEqual(slot.element.actions, ["Go to source"])
    }

    func testModifiersAttach() {
        // Compile-time attachment check: the modifiers accept the shell's call shapes.
        let d = try! JSONDecoder().decode(RuntimeV1.Diagnostic.self, from: Data("{\"severity\":\"error\",\"message\":\"m\"}".utf8))
        _ = Text("row").accessibleDiagnostic(d, index: 0, total: 1) {}
        _ = Text("bar").accessibleCaptureBar(anchor: nil, proposals: 0)
        _ = AccessibilityHelpView()
    }
}
