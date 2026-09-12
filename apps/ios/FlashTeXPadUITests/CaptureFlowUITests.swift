import NearbyClient
import XCTest

/// The capture-companion proof, driven through the UI in the simulator:
/// the UI-test runner hosts a `FakeMac` (loopback TLS-PSK listener with the
/// Mac's parameters, no provider), the app pairs with it via
/// `-flashtexpad-test-mac`, the test draws strokes on the PencilKit canvas
/// (finger drags), sets an instruction, sends, and the runner asserts what
/// the Mac side received: a structurally valid PNG plus the instruction, for
/// the destination the fixture advertised. Also: discard before send, and the
/// bundled-sample-image path (the camera does not exist in the simulator).
final class CaptureFlowUITests: XCTestCase {
    let salt = Data((0..<16).map { UInt8($0 * 3 + 5) })
    let code = "731904"
    var mac: FakeMac!

    override func setUpWithError() throws {
        continueAfterFailure = false
        XCUIDevice.shared.orientation = .landscapeLeft
        let derived = NearbyCrypto.derive(code: code, salt: salt)
        mac = try FakeMac(keys: [.init(identity: derived.pairId, psk: derived.psk, bootstrap: true)], macName: "Runner Mac",
                          destination: NearbyWire.Destination(destinationId: "dest-tikz", projectId: "demo", path: "main.tex", baseRevision: 3))
        mac.start()
        XCTAssertNotEqual(mac.port, 0)
    }

    override func tearDownWithError() throws { mac.stop() }

    private func attach(_ app: XCUIApplication, _ name: String) {
        let a = XCTAttachment(screenshot: app.screenshot())
        a.name = name; a.lifetime = .keepAlways
        add(a)
        Thread.sleep(forTimeInterval: 2) // host-side `xcrun simctl io` window
    }

    private func el(_ app: XCUIApplication, _ id: String) -> XCUIElement {
        app.descendants(matching: .any).matching(identifier: id).firstMatch
    }

    private func text(_ app: XCUIApplication, startingWith p: String) -> XCUIElement {
        app.staticTexts.matching(NSPredicate(format: "label BEGINSWITH %@", p)).firstMatch
    }

    private func launchPaired() -> XCUIApplication {
        let app = XCUIApplication()
        app.launchArguments = ["-flashtexpad-test-mac", "127.0.0.1:\(mac.port):\(NearbyCrypto.hex(salt)):\(NearbyCrypto.fingerprint(salt: salt)):\(code)"]
        app.launch()
        XCTAssertTrue(text(app, startingWith: "Connected to Runner Mac").waitForExistence(timeout: 15), app.debugDescription)
        XCTAssertEqual(mac.hellos.count, 1)
        return app
    }

    private func draw(_ app: XCUIApplication) {
        let canvas = el(app, "capture.canvas") // the SwiftUI wrapper is what XCUITest exposes (ScrollView)
        XCTAssertTrue(canvas.waitForExistence(timeout: 5))
        // A triangle: three finger drags (drawingPolicy = .anyInput).
        let pts = [(0.2, 0.8), (0.8, 0.8), (0.5, 0.2), (0.2, 0.8)]
        for i in 0..<3 {
            let a = canvas.coordinate(withNormalizedOffset: CGVector(dx: pts[i].0, dy: pts[i].1))
            let b = canvas.coordinate(withNormalizedOffset: CGVector(dx: pts[i + 1].0, dy: pts[i + 1].1))
            a.press(forDuration: 0.05, thenDragTo: b)
        }
        XCTAssertTrue(text(app, startingWith: "3 strokes").waitForExistence(timeout: 5), app.debugDescription)
    }

    func testDrawSendReceipt() throws {
        let app = launchPaired()
        draw(app)
        attach(app, "10-canvas-drawn")

        // The default instruction is sent as-is (typing would raise the software
        // keyboard over the buttons; keyboard-free keeps the run deterministic).
        XCTAssertTrue(el(app, "capture.instructions").exists)

        el(app, "capture.prepare").tap()
        XCTAssertTrue(el(app, "capture.send").waitForExistence(timeout: 5))
        attach(app, "11-capture-prepared")
        el(app, "capture.send").tap()
        XCTAssertTrue(text(app, startingWith: "received — Mac inbox").waitForExistence(timeout: 15), app.debugDescription)
        attach(app, "12-capture-received")

        // What the Mac side got, over the real TLS-PSK session.
        XCTAssertEqual(mac.captures.count, 1)
        let cap = try XCTUnwrap(mac.captures.first)
        XCTAssertEqual(cap.destinationId, "dest-tikz")
        XCTAssertEqual(cap.baseRevision, 3)
        XCTAssertEqual(cap.instructions, "Convert this drawing to TikZ")
        XCTAssertEqual(cap.image.mimeType, "image/png")
        let png = try XCTUnwrap(Data(base64Encoded: cap.image.dataBase64))
        XCTAssertNil(NearbyWire.checkImage(png, mimeType: "image/png"))
        XCTAssertGreaterThan(png.count, 1000, "a drawn triangle is more than a blank PNG")
        XCTAssertTrue(NearbyWire.isValidID(cap.captureId))
        XCTAssertTrue(text(app, startingWith: "capture_received capture_id=\(cap.captureId) durable=false").exists)
    }

    func testDiscardBeforeSendSendsNothing() throws {
        let app = launchPaired()
        draw(app)
        el(app, "capture.prepare").tap()
        XCTAssertTrue(el(app, "capture.discard").waitForExistence(timeout: 5))
        el(app, "capture.discard").tap()
        XCTAssertTrue(text(app, startingWith: "discarded before sending").waitForExistence(timeout: 5))
        Thread.sleep(forTimeInterval: 1)
        XCTAssertEqual(mac.captures.count, 0, "nothing reaches the Mac")
        attach(app, "13-capture-discarded")
    }

    func testSampleImagePath() throws {
        let app = launchPaired()
        el(app, "capture.sample").tap()
        XCTAssertTrue(el(app, "capture.pickedImage").waitForExistence(timeout: 5))
        el(app, "capture.prepare").tap()
        XCTAssertTrue(el(app, "capture.send").waitForExistence(timeout: 5))
        attach(app, "14-sample-image-prepared")
        el(app, "capture.send").tap()
        XCTAssertTrue(text(app, startingWith: "received — Mac inbox").waitForExistence(timeout: 15), app.debugDescription)
        XCTAssertEqual(mac.captures.count, 1)
        let png = try XCTUnwrap(Data(base64Encoded: mac.captures[0].image.dataBase64))
        XCTAssertNil(NearbyWire.checkImage(png, mimeType: "image/png"))
        attach(app, "15-sample-image-received")
    }
}
