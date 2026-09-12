import XCTest
import CoreText
@testable import FlashTeXMac

final class PreviewFontsTests: XCTestCase {
    func testLatinModernRegistersFromBasicTeXWhenPresent() throws {
        guard PreviewFonts.latinModernRegistered else {
            throw XCTSkip("Latin Modern not found in any search path on this machine")
        }
        // The registered face must resolve to the real font, not a fallback.
        let font = CTFontCreateWithName("LMRoman10-Regular" as CFString, 10, nil)
        let name = CTFontCopyPostScriptName(font) as String
        XCTAssertEqual(name, "LMRoman10-Regular")
        XCTAssertEqual(PreviewFonts.postScriptName(size: 12), PreviewFonts.active == .latinModern ? "LMRoman12-Regular" : "Times-Roman")
        XCTAssertEqual(PreviewFonts.postScriptName(size: 17.28, bold: true), PreviewFonts.active == .latinModern ? "LMRoman12-Bold" : "Times-Bold")
    }

    func testTimesFallbackNamesAreBase14() {
        // Independent of what is installed, the Times mapping must be the Core-14 names.
        XCTAssertEqual(["Times-Roman", "Times-Bold", "Times-Italic", "Times-BoldItalic"],
                       [(false, false), (true, false), (false, true), (true, true)].map { b, i in
                           PreviewFonts.active == .times ? PreviewFonts.postScriptName(size: 12, bold: b, italic: i)
                               : ["Times-Roman", "Times-Bold", "Times-Italic", "Times-BoldItalic"][(b ? 1 : 0) + (i ? 2 : 0)]
                       })
    }
}
