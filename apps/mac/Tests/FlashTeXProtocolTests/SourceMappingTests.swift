import XCTest
@testable import FlashTeXProtocol

final class SourceMappingTests: XCTestCase {
    func testUnchangedTextIsIdentity() {
        XCTAssertEqual(SourceMapping.rebase(start: 3, end: 7, from: "abcdefgh", to: "abcdefgh"), .unchanged)
    }

    func testRangesBeforeAndAfterAnEditShift() {
        let old = "Hello world. Second sentence."
        let new = "Hello brave new world. Second sentence."   // inserted "brave new " at byte 6
        XCTAssertEqual(SourceMapping.rebase(start: 0, end: 5, from: old, to: new), .rebased(start: 0, end: 5))
        XCTAssertEqual(SourceMapping.rebase(start: 13, end: 19, from: old, to: new), .rebased(start: 23, end: 29))
        XCTAssertEqual(String(new.utf8[new.utf8.index(new.utf8.startIndex, offsetBy: 23)..<new.utf8.index(new.utf8.startIndex, offsetBy: 29)])!, "Second")
    }

    func testRangeOverlappingEditIsRefused() {
        let old = "Hello world."
        let new = "Hello there world."
        XCTAssertEqual(SourceMapping.rebase(start: 6, end: 11, from: old, to: new), .rebased(start: 12, end: 17)) // "world" after insertion
        XCTAssertEqual(SourceMapping.rebase(start: 0, end: 11, from: old, to: new), .overlapsEdit)
        XCTAssertEqual(SourceMapping.rebase(start: 6, end: 11, from: old, to: "Hello wXrld."), .overlapsEdit)
    }

    func testDeletionShiftsLeftAndMultiByteIsByteAccurate() {
        let old = "naïve café — done"
        let new = "café — done"           // deleted "naïve " (7 bytes)
        XCTAssertEqual(SourceMapping.rebase(start: 7, end: 12, from: old, to: new), .rebased(start: 0, end: 5))
        let r = SourceMapping.rebase(.init(path: "m", startByte: 7, endByte: 12), from: old, to: new, expectedText: "café")
        XCTAssertEqual(r, .init(path: "m", startByte: 0, endByte: 5))
        XCTAssertNil(SourceMapping.rebase(.init(path: "m", startByte: 7, endByte: 12), from: old, to: new, expectedText: "cafe"),
                     "expected-text check rejects a mapping that no longer spells the item")
    }

    func testMultipleEditsCollapseConservatively() {
        let old = "aaa bbb ccc ddd"
        let new = "aXa bbb ccc dYd"  // two edits; common prefix "a", common suffix "d"
        XCTAssertEqual(SourceMapping.rebase(start: 4, end: 7, from: old, to: new), .overlapsEdit, "bbb lies between the edits and is refused, never guessed")
    }
}
