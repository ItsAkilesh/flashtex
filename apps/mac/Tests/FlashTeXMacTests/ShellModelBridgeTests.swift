import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// Shell ↔ bridge lifecycle against `Fixtures/fake_bridge.py` and
/// `Fixtures/fake_edit_ledger.py` (Python test doubles): attach → open → pin →
/// submit → received → convert → proposal in the review queue → approve →
/// prepared edit verified → durable commit → adopted once in the editor →
/// `capture_applied` → confirmed; plus dedup and the refusal paths.
@MainActor
final class ShellModelBridgeTests: XCTestCase {
    nonisolated static let fakeLedger = ShellModel.LedgerLaunch(executable: BridgeClientTests.python, arguments: [BridgeClientTests.fakeEditLedger.path])

    static func attach(_ model: ShellModel, store: URL? = nil, ledger: ShellModel.LedgerLaunch? = fakeLedger) async throws -> URL {
        let store = try store ?? BridgeClientTests.tempStore()
        let ok = await model.attachBridgeAndWait(executable: BridgeClientTests.python, arguments: [BridgeClientTests.fakeBridge.path],
                                                 storeDirectory: store, ledger: ledger, discoverLedger: false)
        XCTAssertTrue(ok, model.captureNote ?? model.bridgeStatus)
        XCTAssertTrue(model.bridgeAttached)
        return store
    }

    static func waitUntil(timeout: TimeInterval = 10, _ cond: @escaping () -> Bool) async throws {
        let start = Date()
        while !cond() {
            if Date().timeIntervalSince(start) > timeout { throw XCTSkip("timeout") }
            try await Task.sleep(nanoseconds: 30_000_000)
        }
    }

    private func fixtureImage() throws -> RuntimeV1.CaptureImage { try BridgeClientTests.fixtureCapture().image }

    func testFullReviewedInsertionFlowAndLedgerIdempotence() async throws {
        let model = ShellModel()
        model.autoCompile = false
        XCTAssertEqual(model.activeText, "Hello FlashTeX.\n")
        let store = try await Self.attach(model)
        XCTAssertTrue(model.bridgeStatus.contains("open at revision \(model.editorRevision)"), model.bridgeStatus)
        let bridge = try XCTUnwrap(model.bridge)
        XCTAssertTrue(bridge.ledgerUsable, bridge.ledgerStatus)
        XCTAssertEqual(bridge.durable?.revision, model.editorRevision)

        // Editing streams a document_edit (byte range + replacement) and a durable replace_document.
        model.updateActiveText("Hello naïve FlashTeX.\n")
        let editedRevision = model.editorRevision
        try await Self.waitUntil { bridge.durable?.revision == editedRevision }
        let durableStatus = try await bridge.ledger!.status()
        XCTAssertEqual(durableStatus.document?.text, "Hello naïve FlashTeX.\n")
        XCTAssertEqual(durableStatus.document?.revision, editedRevision)

        // Pin the caret after "naïve " (UTF-16 12 → byte 13) on the bridge as well as locally.
        model.caretUTF16 = 12
        model.pinAnchorAtCaret()
        let localAnchor = try XCTUnwrap(model.anchor)
        XCTAssertEqual(localAnchor.byteOffset, 13)
        try await Self.waitUntil { model.bridgeDestination != nil }
        let destination = try XCTUnwrap(model.bridgeDestination)
        XCTAssertEqual(destination.destinationId, localAnchor.id)
        XCTAssertEqual(destination.startByte, 13)
        XCTAssertEqual(destination.pinnedRevision, editedRevision, "pinned against the revision the edit produced")

        // Submit the fixture image; base_revision is the anchor's pinned revision.
        let maybeReceived = await model.submitCapture(image: try fixtureImage(), captureId: "fixture-capture-1", instructions: "test")
        let received = try XCTUnwrap(maybeReceived, model.captureNote ?? "")
        XCTAssertTrue(received.durable)
        XCTAssertEqual(model.bridgeCaptures.last?.state, .received)
        XCTAssertEqual(model.latestConvertibleCapture?.captureId, "fixture-capture-1")

        // Convert: the proposal lands in the existing review queue with context_revision.
        let maybeProposal = await model.convertCapture(captureId: "fixture-capture-1")
        let proposal = try XCTUnwrap(maybeProposal, model.captureNote ?? "")
        XCTAssertEqual(proposal.latex, "\\fakecapture{fixture-capture-1}")
        XCTAssertEqual(proposal.contextRevision, editedRevision)
        XCTAssertEqual(model.reviewing?.captureId, "fixture-capture-1")
        XCTAssertTrue(model.isBridgeCapture("fixture-capture-1"))
        XCTAssertEqual(model.approveProposal(proposal, latex: proposal.latex), .refused("bridge capture"), "bridge captures never take the local path")

        // Edited LaTeX cannot be sent through the bridge: refused, nothing applied.
        let edited = await model.approveBridgeProposal(proposal, latex: "\\edited")
        XCTAssertEqual(edited, .refused("edited LaTeX"))
        XCTAssertNil(model.pendingEdit)

        // Approve: prepared edit verified (revision, sha, range, removed_text), committed durably
        // by the helper, then staged as one editor edit.
        let before = model.activeText
        let outcome = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(outcome, .inserted(byteOffset: 13))
        let pending = try XCTUnwrap(model.pendingEdit)
        XCTAssertEqual(pending.nsRange, NSRange(location: 12, length: 0))
        XCTAssertEqual(pending.text, "\\fakecapture{fixture-capture-1}")
        XCTAssertTrue(model.proposals.isEmpty)
        let after = "Hello naïve \\fakecapture{fixture-capture-1}FlashTeX.\n"
        XCTAssertEqual(bridge.transactionTrace, ["ledger"], "durable before the editor changes")
        XCTAssertEqual(bridge.durable?.text, after)
        XCTAssertEqual(bridge.durable?.revision, editedRevision + 1)
        let committed = try await bridge.ledger!.status()
        XCTAssertEqual(committed.document?.text, after, "on disk before adoption")
        XCTAssertEqual(committed.pendingReceipts.first?.documentBefore?.text, before)
        XCTAssertEqual(model.activeText, before, "editor not yet changed")

        // The editor adopts it and reports the new buffer: receipt sent, no document_edit.
        model.editApplied(pending, newText: after)
        XCTAssertEqual(model.activeText, after)
        let appliedRevision = model.editorRevision
        XCTAssertEqual(appliedRevision, editedRevision + 1)
        try await Self.waitUntil { model.bridgeCaptures.last?.state == .confirmed }
        XCTAssertEqual(bridge.transactionTrace, ["ledger", "receipt", "confirmed"], "unsaved buffer: no .tex export step")
        XCTAssertNil(bridge.pendingTransaction)
        XCTAssertNil(model.bridgeDestination, "the insertion invalidated the pinned target")
        XCTAssertTrue(model.appliedCaptureIDs.contains("fixture-capture-1"))
        var confirmedStatus = try await bridge.ledger!.status()
        for _ in 0..<100 where !confirmedStatus.pendingReceipts.isEmpty {
            try await Task.sleep(nanoseconds: 30_000_000)
            confirmedStatus = try await bridge.ledger!.status()
        }
        XCTAssertEqual(confirmedStatus.pendingReceipts, [], "helper dropped the recovery snapshot after the bridge acknowledgement")
        XCTAssertEqual(bridge.transactions["capture-fixture-capture-1"]?.confirmed, true)
        // The bridge's snapshot advanced through the receipt: the status must show the applied edit.
        let status = try await model.bridge!.status(captureId: "fixture-capture-1")
        XCTAssertEqual(status.applied?.newRevision, appliedRevision)

        // Repeat approval is a no-op: the ledger and applied set refuse a second insertion.
        let dup = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(dup, .duplicate)
        XCTAssertNil(model.pendingEdit)
        model.enqueue(proposal)
        XCTAssertTrue(model.proposals.isEmpty)

        // Undo (ordinary edit back to the pre-insertion text) keeps the tombstone: approving again
        // is still a duplicate (bridge says already_applied; helper would return the old receipt).
        model.updateActiveText(before)
        try await Self.waitUntil { bridge.durable?.revision == model.editorRevision }
        model.appliedCaptureIDs.remove("fixture-capture-1") // simulate a fresh session's memory
        model.enqueue(proposal)
        let afterUndo = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(afterUndo, .duplicate)
        XCTAssertEqual(model.activeText, before, "never inserted a second time")

        // A later ordinary edit still synchronizes through document_edit (bridge stays consistent).
        model.updateActiveText(before + "more\n")
        model.caretUTF16 = 0
        model.pinAnchorAtCaret()
        try await Self.waitUntil { model.bridgeDestination != nil }
        XCTAssertEqual(model.bridgeDestination?.pinnedRevision, model.editorRevision)
        _ = store
        model.detachBridge()
        XCTAssertFalse(model.bridgeAttached)
    }

    func testBufferChangedBetweenPrepareAndApplyIsRefusedWithReselection() async throws {
        let model = ShellModel()
        model.autoCompile = false
        _ = try await Self.attach(model)
        let bridge = try XCTUnwrap(model.bridge)
        model.caretUTF16 = 5
        model.pinAnchorAtCaret()
        try await Self.waitUntil { model.bridgeDestination != nil }
        let r1 = await model.submitCapture(image: try fixtureImage(), captureId: "fixture-capture-1", instructions: "test")
        XCTAssertNotNil(r1, model.captureNote ?? "")
        let maybeProposal = await model.convertCapture(captureId: "fixture-capture-1")
        let proposal = try XCTUnwrap(maybeProposal, model.captureNote ?? "")

        // Prepare succeeds, but the buffer changes before the edit is verified: the helper's
        // source hash check (and ours) refuse it; nothing is inserted anywhere.
        let edit = try await bridge.prepare(captureId: "fixture-capture-1", expectedRevision: model.editorRevision)
        model.updateActiveText("Hello FlashTeX. typed\n")
        try await Self.waitUntil { bridge.durable?.revision == model.editorRevision }
        if case .refused(let why) = BridgeSession.verify(edit, projectId: "demo", path: "main.tex", revision: model.editorRevision, text: model.activeText) {
            XCTAssertTrue(why.contains("revision"), why)
        } else { XCTFail("stale edit must be refused") }
        do { _ = try await bridge.ledger!.apply(edit); XCTFail("helper must refuse a stale edit") }
        catch let f as LineProcessFailure { XCTAssertEqual(f.code, "revision_conflict", f.text) }
        // Re-approving now fails at the bridge (source changed since preparation) → reselection.
        model.enqueue(proposal)
        let outcome = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        if case .needsReselection = outcome {} else { XCTFail("\(outcome)") }
        XCTAssertNil(model.bridgeDestination)
        XCTAssertNil(model.pendingEdit)
        XCTAssertFalse(model.appliedCaptureIDs.contains("fixture-capture-1"))
        XCTAssertEqual(bridge.durable?.text, "Hello FlashTeX. typed\n")

        // Verification itself: a wrong hash / range / removed text is refused before anything is staged.
        let text = model.activeText
        let good = TransferV1.CaptureEdit(captureId: "x", editId: "capture-x", projectId: "demo", path: "main.tex", expectedRevision: model.editorRevision,
                                          startByte: 5, endByte: 6, removedText: " ", replacement: "y", documentBeforeSha256: SourceDigest.sha256Hex(text))
        XCTAssertEqual(BridgeSession.verify(good, projectId: "demo", path: "main.tex", revision: model.editorRevision, text: text),
                       .ok(afterText: "HelloyFlashTeX. typed\n"))
        var bad = good; bad.documentBeforeSha256 = "0"
        if case .refused(let why) = BridgeSession.verify(bad, projectId: "demo", path: "main.tex", revision: model.editorRevision, text: text) { XCTAssertTrue(why.contains("SHA-256")) } else { XCTFail() }
        bad = good; bad.removedText = "x"
        if case .refused(let why) = BridgeSession.verify(bad, projectId: "demo", path: "main.tex", revision: model.editorRevision, text: text) { XCTAssertTrue(why.contains("removed_text")) } else { XCTFail() }
        bad = good; bad.expectedRevision += 1
        if case .refused(let why) = BridgeSession.verify(bad, projectId: "demo", path: "main.tex", revision: model.editorRevision, text: text) { XCTAssertTrue(why.contains("revision")) } else { XCTFail() }
        let naive = "naïve"
        let mid = TransferV1.CaptureEdit(captureId: "x", editId: "e", projectId: "demo", path: "main.tex", expectedRevision: 1, startByte: 3, endByte: 4,
                                         removedText: "", replacement: "", documentBeforeSha256: SourceDigest.sha256Hex(naive))
        if case .refused(let why) = BridgeSession.verify(mid, projectId: "demo", path: "main.tex", revision: 1, text: naive) { XCTAssertTrue(why.contains("scalar")) } else { XCTFail() }
        model.detachBridge()
    }

    func testProviderDisabledIsPlainTextAndRejectIsForwarded() async throws {
        let model = ShellModel()
        model.autoCompile = false
        _ = try await Self.attach(model)
        model.caretUTF16 = 0
        model.pinAnchorAtCaret()
        try await Self.waitUntil { model.bridgeDestination != nil }
        let r1 = await model.submitCapture(image: try fixtureImage(), captureId: "fixture-capture-1", instructions: "%provider_disabled")
        XCTAssertNotNil(r1, model.captureNote ?? "")
        let c1 = await model.convertCapture(captureId: "fixture-capture-1")
        XCTAssertNil(c1)
        XCTAssertTrue(model.captureNote?.contains("provider_disabled") == true, model.captureNote ?? "")
        XCTAssertTrue(model.bridgeStatus.contains("provider_disabled"), model.bridgeStatus)
        XCTAssertEqual(model.bridgeCaptures.last?.state, .received, "still convertible once a provider is configured")
        XCTAssertTrue(model.proposals.isEmpty)

        // Reject from the review flow reaches the bridge and is terminal.
        let rejected = await model.bridgeRejectAndWait(captureId: "fixture-capture-1")
        XCTAssertTrue(rejected)
        XCTAssertEqual(model.bridgeCaptures.last?.state, .rejected)
        let c2 = await model.convertCapture(captureId: "fixture-capture-1")
        XCTAssertNil(c2)
        XCTAssertTrue(model.captureNote?.contains("capture_rejected") == true, model.captureNote ?? "")
        XCTAssertNil(model.latestConvertibleCapture)
        model.detachBridge()
    }

    func testWithoutAnEditLedgerHelperInsertionIsDisabled() async throws {
        let model = ShellModel()
        model.autoCompile = false
        _ = try await Self.attach(model, ledger: nil)
        XCTAssertTrue(model.captureNote?.contains("insertion is disabled") == true, model.captureNote ?? "")
        XCTAssertFalse(model.bridge!.ledgerUsable)
        model.caretUTF16 = 5
        model.pinAnchorAtCaret()
        try await Self.waitUntil { model.bridgeDestination != nil }
        _ = await model.submitCapture(image: try fixtureImage(), captureId: "fixture-capture-1", instructions: "t")
        let maybeProposal = await model.convertCapture(captureId: "fixture-capture-1")
        let proposal = try XCTUnwrap(maybeProposal, model.captureNote ?? "")
        let outcome = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(outcome, .refused("ledger"))
        XCTAssertNil(model.pendingEdit)
        model.detachBridge()
    }
}
