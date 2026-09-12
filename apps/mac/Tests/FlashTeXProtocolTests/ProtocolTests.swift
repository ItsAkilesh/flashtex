import XCTest
@testable import FlashTeXProtocol

final class ProtocolTests: XCTestCase {
    static let fixtureDir: URL = {
        // Tests/FlashTeXProtocolTests/ProtocolTests.swift -> repo root
        var url = URL(fileURLWithPath: #filePath)
        for _ in 0..<5 { url.deleteLastPathComponent() }
        return url.appendingPathComponent("protocol/fixtures")
    }()

    func testDecodesCompileResultFixture() throws {
        let data = try Data(contentsOf: Self.fixtureDir.appendingPathComponent("compile-result.json"))
        let env = try RuntimeV1.decodeCompileResult(data)
        XCTAssertEqual(env.id, "fixture-compile-1")
        XCTAssertEqual(env.payload.status, .ok)
        XCTAssertEqual(env.payload.revision, 1)
        XCTAssertNil(env.payload.pdfPath)
        XCTAssertEqual(env.payload.pages.count, 1)
        guard case .text(let item) = env.payload.pages[0].items[0] else { return XCTFail("expected text item") }
        XCTAssertEqual(item.text, "Hello FlashTeX.")
        XCTAssertEqual(item.source, .init(path: "main.tex", startByte: 0, endByte: 14))
    }

    func testDecodesCompileRequestFixtureAndRangeMatchesText() throws {
        let req = try RuntimeV1.decodeCompileRequest(Data(contentsOf: Self.fixtureDir.appendingPathComponent("compile-request.json")))
        let res = try RuntimeV1.decodeCompileResult(Data(contentsOf: Self.fixtureDir.appendingPathComponent("compile-result.json")))
        let text = req.payload.documents[0].text
        guard case .text(let item) = res.payload.pages[0].items[0], let src = item.source else { return XCTFail() }
        let r = try XCTUnwrap(text.range(utf8Bytes: src))
        XCTAssertEqual(String(text[r]), "Hello FlashTeX")
    }

    func testRejectsWrongVersionAndType() {
        let bad = #"{"protocol_version":2,"id":"x","type":"compile_result","payload":{}}"#.data(using: .utf8)!
        XCTAssertThrowsError(try RuntimeV1.decodeCompileResult(bad))
        let wrongType = #"{"protocol_version":1,"id":"x","type":"compile","payload":{"project_id":"p","revision":1,"entry_path":"m","documents":[]}}"#.data(using: .utf8)!
        XCTAssertThrowsError(try RuntimeV1.decodeCompileResult(wrongType))
    }

    func testUnknownItemKindDoesNotFail() throws {
        let json = #"{"protocol_version":1,"id":"x","type":"compile_result","payload":{"project_id":"p","revision":1,"status":"ok","pages":[{"number":1,"width_pt":1,"height_pt":1,"items":[{"kind":"image"}]}],"diagnostics":[],"pdf_path":null}}"#
        let env = try RuntimeV1.decodeCompileResult(json.data(using: .utf8)!)
        XCTAssertEqual(env.payload.pages[0].items, [.unknown(kind: "image")])
    }

    func testUTF8OffsetsConvertToUTF16Selection() throws {
        // "é" is 2 UTF-8 bytes / 1 UTF-16 unit; "😀" is 4 UTF-8 bytes / 2 UTF-16 units.
        let text = "aé😀b"
        let ns = try XCTUnwrap(text.nsRange(utf8Bytes: .init(path: "m", startByte: 3, endByte: 8)))
        XCTAssertEqual(ns, NSRange(location: 2, length: 3))
        XCTAssertEqual((text as NSString).substring(with: ns), "😀b")
        // Inside the 4-byte scalar: rejected.
        XCTAssertNil(text.rangeOfUTF8(start: 4, end: 8))
        // Out of bounds / reversed: rejected.
        XCTAssertNil(text.rangeOfUTF8(start: 0, end: 99))
        XCTAssertNil(text.rangeOfUTF8(start: 5, end: 2))
        // Round trip.
        let back = try XCTUnwrap(text.utf8ByteRange(of: ns))
        XCTAssertEqual(back.start, 3); XCTAssertEqual(back.end, 8)
    }
}
