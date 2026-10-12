import AppKit
import XCTest
import FlashTeXDisplayListV3
import FlashTeXPreviewV3
import HostedWindows
@testable import FlashTeXMac

/// A page view placed while its hosted layer is detached from the layer
/// tree, as AppKit leaves it for a pass after the view is added (the page
/// label's flake, testEachPageIsLabelled, 6 of 25 hosted panes on
/// 2026-10-12): its zoom tiles and "page N" label are framed for the flip the
/// layer is shown with once attached, not the detached layer's (#1821, #1823).
/// The detached state is made on purpose, not raced for: a flipped holder
/// with no layer of its own, made layer-backed afterwards. No host needed.
@MainActor
final class EngineV3PageFlipTests: XCTestCase {
    final class FlippedHolder: NSView { override var isFlipped: Bool { true } }

    func settle(_ what: String, timeout: TimeInterval = 30, _ cond: () -> Bool) {
        let end = Date().addingTimeInterval(timeout)
        while !cond() {
            if Date() > end { XCTFail("timeout waiting for \(what)"); return }
            RunLoop.main.run(until: Date().addingTimeInterval(0.01))
        }
    }

    func testTilesAndLabelPlacedWhileDetachedAreRightOnceAttached() throws {
        let doc = try DL3Document(frames: Array(try Data(contentsOf: EngineV3PageTilesTests.fixtures.appendingPathComponent("tile-paths.dl3"))))
        let page = try XCTUnwrap(doc.orderedPages.first)
        let scale = 4.0 // above the tile threshold: tiles, 2 px per view point
        let src = EngineV3TileSource(prepared: page, forms: doc.forms, pdf: nil, key: page.page.hash, pixelsPerPoint: scale,
                                     displayScale: 2, screenPixelsPerPoint: scale)
        XCTAssertTrue(EngineV3TileGrid.tiles(scale))
        let size = CGSize(width: page.widthPt * scale / 2, height: page.heightPt * scale / 2)

        // Placed while detached: the holder has no layer, so the page's has no superlayer.
        let holder = FlippedHolder(frame: CGRect(origin: .zero, size: size))
        let v = EngineV3PageView(frame: holder.bounds)
        holder.addSubview(v)
        v.setLabel(number: 1, dark: false, contentsScale: 2)
        let l = try XCTUnwrap(v.layer)
        v.tiles.show(src, visible: v.bounds, compileID: nil)
        settle("the tiles") { v.tiles.pending == 0 && v.tiles.count > 0 }
        XCTAssertNil(l.superlayer, "the tiles and the label were placed while the page's layer was detached")
        XCTAssertFalse(l.contentsAreFlipped(), "a detached layer reads unflipped (the state the old code placed for)")

        // Attached as in the pane: the holder becomes layer-backed in a window.
        let window = HostedWindowSupport.window(contentRect: CGRect(origin: .zero, size: size), styleMask: [.titled])
        window.isReleasedWhenClosed = false
        let content = NSView(frame: CGRect(origin: .zero, size: size))
        content.wantsLayer = true
        window.contentView = content
        defer { v.tiles.removeAll(); window.contentView = nil }
        content.addSubview(holder)
        settle("the page's layer in the layer tree") { l.superlayer != nil }
        let flipped = l.contentsAreFlipped()
        XCTAssertTrue(flipped, "under a flipped layer-backed view the page's layer is shown flipped")

        // Where the tree shows them, from the page's top, in view points
        // (`convertFromLayer` does not apply the tree's flip).
        let container = v.tiles.container
        XCTAssertEqual(container.contentsAreFlipped(), flipped)
        let (pw, ph) = src.pixelSize
        let ds = CGFloat(src.pixelsPerViewPoint)
        for (index, tile) in v.tiles.layers {
            let r = EngineV3TileGrid.rect(index, pageWidth: pw, pageHeight: ph)
            let f = tile.frame
            let shownTop = flipped ? f.minY : container.bounds.height - f.maxY
            XCTAssertEqual(shownTop, CGFloat(r.y) / ds, accuracy: 0.5, "tile \(index) at \(f) in \(container.bounds)")
            XCTAssertEqual(f.minX, CGFloat(r.x) / ds, accuracy: 0.5, "tile \(index)")
        }
        let label = v.label.frame
        let labelShownY = flipped ? label.minY : v.bounds.height - label.maxY
        XCTAssertEqual(labelShownY, v.labelFrameInView.minY, accuracy: 0.5, "the label at the bottom on screen: \(label) in \(v.bounds)")
    }
}
