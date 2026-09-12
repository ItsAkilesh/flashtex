import XCTest
@testable import FlashTeXCompanion

/// Tests image validation per runtime-v1 constraints.
final class ImageValidatorTests: XCTestCase {

    private func solid(_ color: UIColor, size: CGSize) -> UIImage {
        UIGraphicsImageRenderer(size: size).image { ctx in
            color.setFill()
            ctx.fill(CGRect(origin: .zero, size: size))
        }
    }

    // MARK: – validate() legacy interface

    func testSmallImagePassesValidation() {
        let image = solid(.blue, size: CGSize(width: 100, height: 100))
        let result = ImageValidator.validate(image)
        XCTAssertTrue(result.isValid)
        XCTAssertNotNil(result.image)
        XCTAssertNil(result.error)
    }

    func testOversizedImageIsDownscaled() {
        let bigSize = CGSize(width: 5000, height: 5000)
        let image = solid(.green, size: bigSize)
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

    // MARK: – encode() — MIME type agreement

    func testEncodeSmallImageReturnsPNG() {
        let image = solid(.red, size: CGSize(width: 64, height: 64))
        guard let (data, mime) = ImageValidator.encode(image) else {
            XCTFail("encode() must succeed for a small image")
            return
        }
        XCTAssertEqual(mime, "image/png",
            "Small images must encode as PNG")
        XCTAssertFalse(data.isEmpty)
        // Verify PNG magic bytes
        XCTAssertEqual(data.prefix(4), Data([0x89, 0x50, 0x4E, 0x47]),
            "PNG data must start with \\x89PNG magic bytes")
    }

    func testEncodeMIMETypeMatchesActualEncoding() {
        let image = solid(.blue, size: CGSize(width: 200, height: 200))
        guard let (data, mime) = ImageValidator.encode(image) else {
            XCTFail("encode() must succeed")
            return
        }
        // The declared MIME type must match what the bytes actually are.
        if mime == "image/png" {
            XCTAssertEqual(data.prefix(4), Data([0x89, 0x50, 0x4E, 0x47]),
                "mime_type image/png must correspond to actual PNG bytes")
        } else if mime == "image/jpeg" {
            XCTAssertEqual(data.prefix(2), Data([0xFF, 0xD8]),
                "mime_type image/jpeg must correspond to actual JPEG bytes")
        } else {
            XCTFail("Unexpected mime_type: \(mime)")
        }
        XCTAssertTrue(ImageValidator.acceptedMIMETypes.contains(mime),
            "Returned MIME type must be one of the accepted types")
    }

    func testEncodeOversizedImageDownscales() {
        let bigSize = CGSize(width: 5000, height: 5000)
        let image = solid(.green, size: bigSize)
        guard let (data, _) = ImageValidator.encode(image) else {
            XCTFail("encode() must succeed even for oversized images")
            return
        }
        XCTAssertFalse(data.isEmpty)
        XCTAssertLessThanOrEqual(data.count, ImageValidator.maxDataSize)
    }

    // MARK: – normalizeOrientation

    func testUpOrientationIsUnchanged() {
        let image = solid(.red, size: CGSize(width: 10, height: 20))
        // A freshly-rendered UIGraphicsImageRenderer image is always .up
        XCTAssertEqual(image.imageOrientation, .up)
        let normalized = ImageValidator.normalizeOrientation(image)
        // Same object is returned when already .up
        XCTAssertTrue(normalized === image)
    }
}
