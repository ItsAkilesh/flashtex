import XCTest
@testable import FlashTeXCompanion

/// Tests that capture payloads match runtime-v1 protocol contract.
final class CapturePayloadTests: XCTestCase {

    /// Create a 1x1 red pixel test image.
    private func makeTestImage() -> UIImage {
        UIGraphicsImageRenderer(size: CGSize(width: 1, height: 1)).image { ctx in
            UIColor.red.setFill()
            ctx.fill(CGRect(x: 0, y: 0, width: 1, height: 1))
        }
    }

    func testEnvelopeStructure() throws {
        let image = makeTestImage()
        let envelope = try XCTUnwrap(CaptureEnvelope.create(
            captureID: "test-001",
            destinationID: "anchor-1",
            baseRevision: 1,
            image: image,
            instructions: "Transcribe handwriting."
        ))

        XCTAssertEqual(envelope.protocolVersion, 1)
        XCTAssertEqual(envelope.type, "capture_submit")
        XCTAssertEqual(envelope.payload.captureID, "test-001")
        XCTAssertEqual(envelope.payload.destinationID, "anchor-1")
        XCTAssertEqual(envelope.payload.baseRevision, 1)
        XCTAssertEqual(envelope.payload.image.mimeType, "image/png")
        XCTAssertEqual(envelope.payload.instructions, "Transcribe handwriting.")
        XCTAssertFalse(envelope.payload.image.dataBase64.isEmpty)
    }

    func testJSONMatchesFixtureKeys() throws {
        let image = makeTestImage()
        let envelope = try XCTUnwrap(CaptureEnvelope.create(
            captureID: "fixture-capture-1",
            destinationID: "fixture-anchor-1",
            baseRevision: 1,
            image: image,
            instructions: "Faithfully transcribe the selected handwriting; preserve notation."
        ))

        let json = try XCTUnwrap(envelope.toJSONString())
        let data = try XCTUnwrap(json.data(using: .utf8))
        let obj = try JSONSerialization.jsonObject(with: data) as! [String: Any]

        // Top-level keys match fixture
        XCTAssertNotNil(obj["protocol_version"])
        XCTAssertNotNil(obj["id"])
        XCTAssertNotNil(obj["type"])
        XCTAssertNotNil(obj["payload"])
        XCTAssertEqual(obj["protocol_version"] as? Int, 1)
        XCTAssertEqual(obj["type"] as? String, "capture_submit")

        // Payload keys match fixture
        let payload = obj["payload"] as! [String: Any]
        XCTAssertNotNil(payload["capture_id"])
        XCTAssertNotNil(payload["destination_id"])
        XCTAssertNotNil(payload["base_revision"])
        XCTAssertNotNil(payload["image"])
        XCTAssertNotNil(payload["instructions"])

        // Image keys match fixture
        let imageObj = payload["image"] as! [String: Any]
        XCTAssertNotNil(imageObj["mime_type"])
        XCTAssertNotNil(imageObj["data_base64"])
        XCTAssertEqual(imageObj["mime_type"] as? String, "image/png")
    }

    func testBase64DecodesBackToImage() throws {
        let image = makeTestImage()
        let envelope = try XCTUnwrap(CaptureEnvelope.create(
            captureID: "round-trip-test",
            destinationID: "anchor-1",
            baseRevision: 1,
            image: image,
            instructions: "Test"
        ))

        let base64 = envelope.payload.image.dataBase64
        let decoded = try XCTUnwrap(Data(base64Encoded: base64))
        let roundTrip = try XCTUnwrap(UIImage(data: decoded))
        XCTAssertGreaterThan(roundTrip.size.width, 0)
        XCTAssertGreaterThan(roundTrip.size.height, 0)
    }

    func testJSONIsOneLine() throws {
        let image = makeTestImage()
        let envelope = try XCTUnwrap(CaptureEnvelope.create(
            captureID: "oneline-test",
            destinationID: "anchor-1",
            baseRevision: 1,
            image: image,
            instructions: "Test"
        ))

        let json = try XCTUnwrap(envelope.toJSONString())
        // JSON Lines requires one object per line
        XCTAssertFalse(json.contains("\n"), "JSON output must be a single line for JSON Lines protocol")
    }

    func testDuplicatePrevention() {
        let transport = CaptureTransport.shared
        let image = makeTestImage()

        let uniqueID = "dedup-test-\(UUID().uuidString)"
        guard let envelope = CaptureEnvelope.create(
            captureID: uniqueID,
            destinationID: "anchor-1",
            baseRevision: 1,
            image: image,
            instructions: "Test"
        ) else {
            XCTFail("Failed to create envelope")
            return
        }

        // First send should succeed
        XCTAssertTrue(transport.send(envelope))
        // Second send with same ID should be blocked
        XCTAssertFalse(transport.send(envelope))
        XCTAssertTrue(transport.wasSent(uniqueID))
    }
}
