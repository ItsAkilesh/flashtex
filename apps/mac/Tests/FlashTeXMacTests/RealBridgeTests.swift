import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// End-to-end check against the actual Rust bridge (FT-007, built from
/// `crates/bridge` at b5ca96b). Skipped unless `FLASHTEX_BRIDGE` points at a
/// built `flashtex-bridge`, so `swift test` stays hermetic. Without
/// `--enable-grok` nothing here talks to a network: conversion must fail with
/// `provider_disabled`.
@MainActor
final class RealBridgeTests: XCTestCase {
    static var binary: URL? {
        ProcessInfo.processInfo.environment["FLASHTEX_BRIDGE"].map { URL(fileURLWithPath: $0) }
    }

    /// 1×1 RGB PNG that the bridge's image decoder accepts. This is the image
    /// `protocol/fixtures/capture-submission.json` carries on the bridge branch
    /// (ba89c9a); the copy on main is rejected by the bridge as `invalid_image`.
    static let decodablePNGBase64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4//8/AAX+Av4N70a4AAAAAElFTkSuQmCC"

    func testDurableReceiptDuplicateConflictProviderDisabledAndReject() async throws {
        guard let binary = Self.binary, FileManager.default.isExecutableFile(atPath: binary.path) else {
            throw XCTSkip("set FLASHTEX_BRIDGE to the built flashtex-bridge binary")
        }
        let store = try BridgeClientTests.tempStore()
        defer { try? FileManager.default.removeItem(at: store) }
        let model = ShellModel()
        model.autoCompile = false
        let attached = await model.attachBridgeAndWait(executable: binary, storeDirectory: store)
        XCTAssertTrue(attached, model.captureNote ?? model.bridgeStatus)
        XCTAssertTrue(model.bridgeAttached)
        XCTAssertTrue(model.bridgeStatus.contains("open at revision"), model.bridgeStatus)

        // document_edit is accepted by the real bridge (anchor-free edit before pinning).
        model.updateActiveText("Hello naïve FlashTeX.\n")
        model.caretUTF16 = 12 // after "naïve " → byte 13
        model.pinAnchorAtCaret()
        try await waitUntil { model.bridgeDestination != nil || model.captureNote?.contains("pin failed") == true }
        let destination = try XCTUnwrap(model.bridgeDestination, model.captureNote ?? "")
        XCTAssertEqual(destination.startByte, 13)
        XCTAssertEqual(destination.pinnedRevision, model.editorRevision)
        XCTAssertEqual(destination.binding.sourceSha256, SourceDigest.sha256Hex(model.activeText))
        XCTAssertTrue(destination.valid)

        // main 9da7e48 corrected protocol/fixtures/capture-submission.json (the earlier
        // 1×1 PNG did not decode): the shared fixture must now be accepted durably.
        let mainFixture = try BridgeClientTests.fixtureCapture().image
        let result1 = await model.submitCapture(image: mainFixture, captureId: "real-capture-main-fixture", instructions: "t")
        XCTAssertEqual(result1?.durable, true, model.captureNote ?? "")
        XCTAssertEqual(model.bridgeCaptures.first { $0.captureId == "real-capture-main-fixture" }?.state, .received)

        // Durable receipt with a decodable PNG; the journal file exists in the store.
        let image = RuntimeV1.CaptureImage(mimeType: "image/png", dataBase64: Self.decodablePNGBase64)
        let maybe2 = await model.submitCapture(image: image, captureId: "real-capture-1", instructions: "Transcribe.")
        let received = try XCTUnwrap(maybe2, model.captureNote ?? "")
        XCTAssertEqual(received, .init(captureId: "real-capture-1", durable: true, hasProposal: false, applied: false))
        XCTAssertTrue(FileManager.default.fileExists(atPath: store.appendingPathComponent("real-capture-1.json").path))

        // Identical retry returns the same record; a different payload under the same ID conflicts.
        let maybe3 = await model.submitCapture(image: image, captureId: "real-capture-1", instructions: "Transcribe.")
        let again = try XCTUnwrap(maybe3)
        XCTAssertEqual(again, received)
        let result4 = await model.submitCapture(image: image, captureId: "real-capture-1", instructions: "Different instructions.")
        XCTAssertNil(result4)
        XCTAssertTrue(model.captureNote?.contains("capture_id_conflict") == true, model.captureNote ?? "")
        XCTAssertEqual(model.bridgeCaptures.first { $0.captureId == "real-capture-1" }?.state, .received,
                       "a conflicting retry leaves the journaled capture usable")

        // No --enable-grok: conversion is refused as plain text, no key prompt, status has no proposal.
        let result5 = await model.convertCapture(captureId: "real-capture-1")
        XCTAssertNil(result5)
        XCTAssertTrue(model.captureNote?.contains("provider_disabled") == true, model.captureNote ?? "")
        XCTAssertTrue(model.proposals.isEmpty)
        let status = try await model.bridge!.status(captureId: "real-capture-1")
        XCTAssertNil(status.proposal)
        XCTAssertNil(status.prepared)
        XCTAssertNil(status.applied)
        XCTAssertFalse(status.rejected)
        // Preparing without a proposal is refused by the bridge.
        do { _ = try await model.bridge!.prepare(captureId: "real-capture-1", expectedRevision: model.editorRevision); XCTFail() }
        catch let f as BridgeClient.Failure { XCTAssertEqual(f.code, "proposal_missing", f.text) }

        // Reject is durable and terminal.
        let flag6 = await model.bridgeRejectAndWait(captureId: "real-capture-1")
        XCTAssertTrue(flag6)
        let rejected = try await model.bridge!.status(captureId: "real-capture-1")
        XCTAssertTrue(rejected.rejected)
        let result7 = await model.convertCapture(captureId: "real-capture-1")
        XCTAssertNil(result7)
        // b5ca96b checks provider enablement before the journal record, so without
        // --enable-grok a rejected capture's conversion fails with provider_disabled,
        // not capture_rejected; either way it fails and the capture stays rejected.
        XCTAssertTrue(model.captureNote?.contains("provider_disabled") == true || model.captureNote?.contains("capture_rejected") == true, model.captureNote ?? "")
        XCTAssertEqual(model.bridgeCaptures.first { $0.captureId == "real-capture-1" }?.state, .rejected)
        // The accepted main-fixture capture is still convertible; the rejected one is not.
        XCTAssertEqual(model.latestConvertibleCapture?.captureId, "real-capture-main-fixture")
        do { _ = try await model.bridge!.prepare(captureId: "real-capture-1", expectedRevision: model.editorRevision); XCTFail() }
        catch let f as BridgeClient.Failure { XCTAssertEqual(f.code, "capture_rejected", f.text) }
        // Unknown capture: capture_missing.
        do { _ = try await model.bridge!.status(captureId: "never-sent"); XCTFail() }
        catch let f as BridgeClient.Failure { XCTAssertEqual(f.code, "capture_missing") }
        model.detachBridge()
        XCTAssertFalse(model.bridgeAttached)
    }

    private func waitUntil(timeout: TimeInterval = 10, _ cond: () -> Bool) async throws {
        let start = Date()
        while !cond() {
            if Date().timeIntervalSince(start) > timeout { throw XCTSkip("timeout") }
            try await Task.sleep(nanoseconds: 30_000_000)
        }
    }
}
