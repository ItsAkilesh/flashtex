import XCTest
@testable import FlashTeXCompanion

/// Tests image validation per runtime-v1 constraints.
final class ImageValidatorTests: XCTestCase {

    func testSmallImagePassesValidation() {
        let image = UIGraphicsImageRenderer(size: CGSize(width: 100, height: 100)).image { ctx in
            UIColor.blue.setFill()
            ctx.fill(CGRect(x: 0, y: 0, width: 100, height: 100))
        }
        let result = ImageValidator.validate(image)
        XCTAssertTrue(result.isValid)
        XCTAssertNotNil(result.image)
        XCTAssertNotNil(result.encodedData)
        XCTAssertEqual(result.mimeType, "image/png")
        XCTAssertNil(result.error)
    }

    func testOversizedImageIsDownscaled() {
        let bigSize = CGSize(width: 5000, height: 5000)
        let image = UIGraphicsImageRenderer(size: bigSize).image { ctx in
            UIColor.green.setFill()
            ctx.fill(CGRect(origin: .zero, size: bigSize))
        }
        let result = ImageValidator.validate(image)
        XCTAssertTrue(result.isValid)
        guard let validated = result.image else {
            XCTFail("Expected validated image")
            return
        }
        XCTAssertLessThanOrEqual(validated.size.width, ImageValidator.maxDimension)
        XCTAssertLessThanOrEqual(validated.size.height, ImageValidator.maxDimension)
    }

    func testAcceptedMIMETypes() {
        XCTAssertTrue(ImageValidator.acceptedMIMETypes.contains("image/png"))
        XCTAssertTrue(ImageValidator.acceptedMIMETypes.contains("image/jpeg"))
        XCTAssertFalse(ImageValidator.acceptedMIMETypes.contains("image/gif"))
    }
}
