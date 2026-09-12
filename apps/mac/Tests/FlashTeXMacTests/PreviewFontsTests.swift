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

    // MARK: resource generation

    func testResourceGenerationMovesOnlyWhenAnInputOfResolutionChanges() {
        let original = PreviewFonts.producerFace, env = PreviewFonts.environmentFace
        defer { PreviewFonts.producerFace = original; PreviewFonts.overrideEnvironmentFace(env) }

        let g = PreviewFonts.resourceGeneration
        PreviewFonts.producerFace = original
        XCTAssertEqual(PreviewFonts.resourceGeneration, g, "re-setting the same producer face is not a change")
        PreviewFonts.overrideEnvironmentFace(env)
        XCTAssertEqual(PreviewFonts.resourceGeneration, g, "re-setting the same override is not a change")

        let other: PreviewFonts.Face = original == .times ? .latinModern : .times
        PreviewFonts.producerFace = other
        XCTAssertEqual(PreviewFonts.resourceGeneration, g + 1)
        PreviewFonts.producerFace = original
        XCTAssertEqual(PreviewFonts.resourceGeneration, g + 2)

        PreviewFonts.overrideEnvironmentFace(.times)
        let afterTimes = PreviewFonts.resourceGeneration
        XCTAssertEqual(afterTimes, env == .times ? g + 2 : g + 3)
        XCTAssertEqual(PreviewFonts.requested, .times, "the override wins over the producer face")
        PreviewFonts.overrideEnvironmentFace(.latinModern)
        XCTAssertEqual(PreviewFonts.resourceGeneration, afterTimes + 1)
        XCTAssertEqual(PreviewFonts.requested, .latinModern)
        PreviewFonts.overrideEnvironmentFace(nil)
        XCTAssertEqual(PreviewFonts.resourceGeneration, afterTimes + 2)
        XCTAssertEqual(PreviewFonts.requested, original, "no override: the producer face")

        PreviewFonts.invalidateResources()
        XCTAssertEqual(PreviewFonts.resourceGeneration, afterTimes + 3, "explicit invalidation always moves it")
    }

    func testLatinModernRegistrationIsAGenerationAndRecordsItsDirectory() throws {
        guard PreviewFonts.latinModernRegistered else {
            XCTAssertNil(PreviewFonts.latinModernDirectory)
            throw XCTSkip("Latin Modern not found in any search path on this machine")
        }
        let dir = try XCTUnwrap(PreviewFonts.latinModernDirectory)
        XCTAssertTrue(PreviewFonts.latinModernSearchPaths.contains(dir), dir)
        XCTAssertTrue(FileManager.default.fileExists(atPath: dir + "/lmroman10-regular.otf"), dir)
        XCTAssertGreaterThanOrEqual(PreviewFonts.resourceGeneration, 1, "registration moved the generation from 0")
        // A key made now is stamped with a generation that includes the registration.
        let key = PreviewTextCache.key(text: "x", postScriptName: "LMRoman10-Regular", size: 10)
        XCTAssertEqual(key.generation, PreviewFonts.resourceGeneration)
        XCTAssertGreaterThanOrEqual(key.generation, 1)
    }
}
