import XCTest

/// Drives the app in the simulator: opens the bundled sample, then the review
/// fixture, cancels, re-opens, approves, and checks the receipt is shown.
/// Screenshots are taken from the host with `xcrun simctl io <udid> screenshot`
/// (docs/evidence/ios-acceptance-2026-09-12/); XCTAttachments are kept too.
final class FlashTeXPadUITests: XCTestCase {
    override func setUpWithError() throws { continueAfterFailure = false }

    private func attach(_ app: XCUIApplication, _ name: String) {
        let a = XCTAttachment(screenshot: app.screenshot())
        a.name = name; a.lifetime = .keepAlways
        add(a)
    }

    func testOpenSampleThenCancelThenApprove() throws {
        let app = XCUIApplication()
        app.launch()

        app.buttons["open.sample"].tap()
        XCTAssertTrue(app.staticTexts["document.title"].waitForExistence(timeout: 5))
        XCTAssertEqual(app.staticTexts["document.title"].label, "demo.tex")
        XCTAssertTrue(app.textViews["editor.textview"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.textViews["editor.textview"].value.debugDescription.contains("FlashTeX demo"))
        attach(app, "01-sample-open")

        // Review fixture → cancel
        app.buttons["open.fixture"].tap()
        app.staticTexts["panel.Review"].firstMatch.tap()
        XCTAssertTrue(app.staticTexts["review.state"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.staticTexts["review.state"].label.hasPrefix("pending"))
        attach(app, "02-review-pending")
        app.buttons["review.cancel"].tap()
        XCTAssertTrue(app.staticTexts["review.state"].label.hasPrefix("cancelled"))
        XCTAssertFalse(app.staticTexts["review.insertions"].exists)
        attach(app, "03-review-cancelled")

        // Re-open fixture → approve
        app.buttons["open.fixture"].tap()
        XCTAssertTrue(app.staticTexts["review.state"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.staticTexts["review.state"].label.hasPrefix("pending"))
        app.buttons["review.approve"].tap()
        XCTAssertTrue(app.staticTexts["review.insertions"].waitForExistence(timeout: 5))
        XCTAssertEqual(app.staticTexts["review.insertions"].label, "insertions this session: 1")
        XCTAssertTrue(app.staticTexts["review.state"].label.hasPrefix("applied once"))
        XCTAssertFalse(app.buttons["review.approve"].isEnabled)
        attach(app, "04-review-applied")

        app.staticTexts["panel.Editor"].firstMatch.tap()
        XCTAssertTrue(app.textViews["editor.textview"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.textViews["editor.textview"].value.debugDescription.contains("Example text"))
        attach(app, "05-editor-after-insert")

        app.staticTexts["panel.Diagnostics"].firstMatch.tap()
        XCTAssertTrue(app.staticTexts["diagnostics.source"].waitForExistence(timeout: 5))
        attach(app, "06-diagnostics")
    }
}
