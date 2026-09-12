import XCTest
import PDFKit
import FlashTeXProtocol
@testable import FlashTeXMac

@MainActor
final class RustPDFExportTests: XCTestCase {
    func testRustWriterProducesReadablePDFForFixture() throws {
        guard let env = ProcessInfo.processInfo.environment["FLASHTEX_PDF"],
              FileManager.default.isExecutableFile(atPath: env) else {
            throw XCTSkip("set FLASHTEX_PDF to a built flashtex-pdf binary")
        }
        let model = ShellModel()
        let result = try XCTUnwrap(model.result)
        let out = FileManager.default.temporaryDirectory.appendingPathComponent("flashtex-rust-\(UUID().uuidString).pdf")
        defer { try? FileManager.default.removeItem(at: out) }
        let notes = try RustPDFExport.export(result, id: "t", writer: URL(fileURLWithPath: env), to: out)
        let doc = try XCTUnwrap(PDFDocument(url: out))
        XCTAssertEqual(doc.pageCount, result.pages.count)
        let page = try XCTUnwrap(doc.page(at: 0))
        XCTAssertEqual(page.bounds(for: .mediaBox).size, CGSize(width: 612, height: 792))
        XCTAssertTrue(page.string?.contains("Hello") == true, "page text: \(page.string ?? "nil"); notes: \(notes)")
    }

    func testRustWriterRejectsMissingBinaryGracefully() {
        XCTAssertThrowsError(try RustPDFExport.export(
            RuntimeV1.CompileResult(projectId: "p", revision: 1, status: .ok, pages: [], diagnostics: [], pdfPath: nil),
            id: "t", writer: URL(fileURLWithPath: "/nonexistent/flashtex-pdf"),
            to: FileManager.default.temporaryDirectory.appendingPathComponent("x.pdf")))
    }
}
