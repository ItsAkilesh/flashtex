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
