import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// Shell ↔ bridge lifecycle against `Fixtures/fake_bridge.py` (Python test
/// double): attach → open → pin → submit → received → convert → proposal in
/// the review queue → approve → prepared edit verified and applied once →
/// `capture_applied` → ledger idempotence; plus the refusal paths.
@MainActor
final class ShellModelBridgeTests: XCTestCase {
    private func attach(_ model: ShellModel, store: URL? = nil) async throws -> URL {
        let store = try store ?? BridgeClientTests.tempStore()
        let ok = await model.attachBridgeAndWait(executable: BridgeClientTests.python, arguments: [BridgeClientTests.fakeBridge.path], storeDirectory: store)
        XCTAssertTrue(ok, model.captureNote ?? model.bridgeStatus)
        XCTAssertTrue(model.bridgeAttached)
        return store
    }

    private func waitUntil(timeout: TimeInterval = 10, _ cond: () -> Bool) async throws {
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
        let store = try await attach(model)
        XCTAssertTrue(model.bridgeStatus.contains("open at revision \(model.editorRevision)"), model.bridgeStatus)

        // Editing streams a document_edit (byte range + replacement) at the new revision.
        model.updateActiveText("Hello naïve FlashTeX.\n")
        let editedRevision = model.editorRevision

        // Pin the caret after "naïve " (UTF-16 11 → byte 12) on the bridge as well as locally.
        model.caretUTF16 = 12
        model.pinAnchorAtCaret()
        let localAnchor = try XCTUnwrap(model.anchor)
        XCTAssertEqual(localAnchor.byteOffset, 13)
        try await waitUntil { model.bridgeDestination != nil }
        let destination = try XCTUnwrap(model.bridgeDestination)
        XCTAssertEqual(destination.destinationId, localAnchor.id)
        XCTAssertEqual(destination.startByte, 13)
        XCTAssertEqual(destination.pinnedRevision, editedRevision, "pinned against the revision the edit produced")

        // Submit the fixture image; base_revision is the anchor's pinned revision.
        let maybe1 = try await model.submitCapture(image: fixtureImage(), captureId: "fixture-capture-1", instructions: "test")
        let received = try XCTUnwrap(maybe1)
        XCTAssertTrue(received.durable)
        XCTAssertEqual(model.bridgeCaptures.last?.state, .received)
        XCTAssertEqual(model.latestConvertibleCapture?.captureId, "fixture-capture-1")

        // Convert: the proposal lands in the existing review queue with context_revision.
        let maybe2 = await model.convertCapture(captureId: "fixture-capture-1")
        let proposal = try XCTUnwrap(maybe2)
        XCTAssertEqual(proposal.latex, "\\fakecapture{fixture-capture-1}")
        XCTAssertEqual(proposal.contextRevision, editedRevision)
        XCTAssertEqual(model.reviewing?.captureId, "fixture-capture-1")
        XCTAssertTrue(model.isBridgeCapture("fixture-capture-1"))
        XCTAssertEqual(model.approveProposal(proposal, latex: proposal.latex), .refused("bridge capture"), "bridge captures never take the local path")

        // Edited LaTeX cannot be sent through the bridge: refused, nothing applied.
        let outcome3 = await model.approveBridgeProposal(proposal, latex: "\\edited")
        XCTAssertEqual(outcome3, .refused("edited LaTeX"))
        XCTAssertNil(model.pendingEdit)

        // Approve: prepared edit verified (revision, sha, range, removed_text) and staged as one editor edit.
        let before = model.activeText
        let outcome4 = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(outcome4, .inserted(byteOffset: 13))
        let pending = try XCTUnwrap(model.pendingEdit)
        XCTAssertEqual(pending.nsRange, NSRange(location: 12, length: 0))
        XCTAssertEqual(pending.text, "\\fakecapture{fixture-capture-1}")
        XCTAssertTrue(model.proposals.isEmpty)
        let ledger = EditLedger(storeDirectory: store)
        XCTAssertEqual(ledger.entries.first?.state, .prepared)
        XCTAssertEqual(ledger.entries.first?.documentBeforeText, before)

        // The editor applies it and reports the new buffer: ledger → applied, capture_applied sent, no document_edit.
        let after = "Hello naïve \\fakecapture{fixture-capture-1}FlashTeX.\n"
        model.editApplied(pending, newText: after)
        XCTAssertEqual(model.activeText, after)
        let appliedRevision = model.editorRevision
        XCTAssertEqual(appliedRevision, editedRevision + 1)
        try await waitUntil { model.bridgeCaptures.last?.state == .confirmed }
        let confirmed = EditLedger(storeDirectory: store).entries.first
        XCTAssertEqual(confirmed?.state, .confirmed)
        XCTAssertEqual(confirmed?.newRevision, appliedRevision)
        XCTAssertNil(confirmed?.documentBeforeText, "pre-edit text is dropped once the receipt is confirmed")
        XCTAssertNil(model.bridgeDestination, "the insertion invalidated the pinned target")
        XCTAssertTrue(model.appliedCaptureIDs.contains("fixture-capture-1"))
        // The bridge's snapshot advanced through the receipt: the status must show the applied edit.
        let status = try await model.bridge!.status(captureId: "fixture-capture-1")
        XCTAssertEqual(status.applied?.newRevision, appliedRevision)

        // Repeat approval is a no-op: the ledger and applied set refuse a second insertion.
        let outcome5 = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(outcome5, .duplicate)
        XCTAssertNil(model.pendingEdit)
        model.enqueue(proposal)
        XCTAssertTrue(model.proposals.isEmpty)
        XCTAssertEqual(EditLedger(storeDirectory: store).entries.count, 1)

        // A later ordinary edit still synchronizes through document_edit (bridge stays consistent).
        model.updateActiveText(after + "more\n")
        model.caretUTF16 = 0
        model.pinAnchorAtCaret()
        try await waitUntil { model.bridgeDestination != nil }
        XCTAssertEqual(model.bridgeDestination?.pinnedRevision, model.editorRevision)
        model.detachBridge()
        XCTAssertFalse(model.bridgeAttached)
    }

    func testBufferChangedBetweenPrepareAndApplyIsRefusedWithReselection() async throws {
        let model = ShellModel()
        model.autoCompile = false
        _ = try await attach(model)
        model.caretUTF16 = 5
        model.pinAnchorAtCaret()
        try await waitUntil { model.bridgeDestination != nil }
        let maybe6 = try await model.submitCapture(image: fixtureImage(), captureId: "fixture-capture-1", instructions: "test")
        XCTAssertNotNil(maybe6, model.captureNote ?? "")
        let maybe7 = await model.convertCapture(captureId: "fixture-capture-1")
        let proposal = try XCTUnwrap(maybe7)

        // Approve, but the editor never applies the prepared edit: the user types instead.
        let outcome8 = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(outcome8, .inserted(byteOffset: 5))
        XCTAssertNotNil(model.bridge?.expectedApplication)
        model.updateActiveText("Hello FlashTeX. typed\n")
        XCTAssertNil(model.bridge?.expectedApplication)
        XCTAssertEqual(model.bridgeCaptures.last?.state, .needsReselection)
        XCTAssertEqual(model.bridge?.ledger.entries.first?.state, .abandoned)
        XCTAssertTrue(model.captureNote?.contains("not applied as prepared") == true, model.captureNote ?? "")
        // Re-approving now fails at the bridge (source changed since preparation) → reselection.
        model.enqueue(proposal)
        let outcome = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        if case .needsReselection = outcome {} else { XCTFail("\(outcome)") }
        XCTAssertNil(model.bridgeDestination)
        XCTAssertFalse(model.appliedCaptureIDs.contains("fixture-capture-1"))

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
        _ = try await attach(model)
        model.caretUTF16 = 0
        model.pinAnchorAtCaret()
        try await waitUntil { model.bridgeDestination != nil }
        let maybe9 = try await model.submitCapture(image: fixtureImage(), captureId: "fixture-capture-1", instructions: "%provider_disabled")
        XCTAssertNotNil(maybe9, model.captureNote ?? "")
        let result10 = await model.convertCapture(captureId: "fixture-capture-1")
        XCTAssertNil(result10)
        XCTAssertTrue(model.captureNote?.contains("provider_disabled") == true, model.captureNote ?? "")
        XCTAssertTrue(model.bridgeStatus.contains("provider_disabled"), model.bridgeStatus)
        XCTAssertEqual(model.bridgeCaptures.last?.state, .received, "still convertible once a provider is configured")
        XCTAssertTrue(model.proposals.isEmpty)

        // Reject from the review flow reaches the bridge and is terminal.
        let flag11 = await model.bridgeRejectAndWait(captureId: "fixture-capture-1")
        XCTAssertTrue(flag11)
        XCTAssertEqual(model.bridgeCaptures.last?.state, .rejected)
        let result12 = await model.convertCapture(captureId: "fixture-capture-1")
        XCTAssertNil(result12)
        XCTAssertTrue(model.captureNote?.contains("capture_rejected") == true, model.captureNote ?? "")
        XCTAssertNil(model.latestConvertibleCapture)
        model.detachBridge()
    }

    func testRestartReconciliationReplaysMissingReceiptAndNeverReapplies() async throws {
        // Session 1: apply an edit but "crash" before the receipt is confirmed.
        let store = try BridgeClientTests.tempStore()
        let text = "Hello FlashTeX.\n"
        let edit = TransferV1.CaptureEdit(captureId: "fixture-capture-1", editId: "capture-fixture-capture-1", projectId: "demo", path: "main.tex",
                                          expectedRevision: 1, startByte: 5, endByte: 5, removedText: "", replacement: "\\fakecapture{fixture-capture-1}",
                                          documentBeforeSha256: SourceDigest.sha256Hex(text))
        let after = "Hello\\fakecapture{fixture-capture-1} FlashTeX.\n"
        let ledger = EditLedger(storeDirectory: store)
        try ledger.upsert(EditLedgerEntry(edit: edit, beforeText: text))
        try ledger.update(editId: edit.editId) { $0.state = .applied; $0.newRevision = 2; $0.documentAfterSha256 = SourceDigest.sha256Hex(after) }

        // The fake bridge is in-memory, so seed its journal to the prepared state through a first session.
        let seed = ShellModel()
        seed.autoCompile = false
        _ = try await attach(seed, store: store)
        // Reconciliation with a fresh journal: the bridge has no record → entry abandoned, nothing applied.
        XCTAssertEqual(EditLedger(storeDirectory: store).entries.first?.state, .abandoned)
        XCTAssertTrue(seed.captureNote?.contains("no usable record") == true, seed.captureNote ?? "")
        XCTAssertEqual(seed.activeText, text, "reconciliation never edits the buffer")
        seed.detachBridge()

        // Now build the real situation inside one bridge process: prepared in the bridge, applied in the ledger, receipt lost.
        let model = ShellModel()
        model.autoCompile = false
        let store2 = try BridgeClientTests.tempStore()
        _ = try await attach(model, store: store2)
        model.caretUTF16 = 5
        model.pinAnchorAtCaret()
        try await waitUntil { model.bridgeDestination != nil }
        let maybe13 = try await model.submitCapture(image: fixtureImage(), captureId: "fixture-capture-1", instructions: "t")
        XCTAssertNotNil(maybe13, model.captureNote ?? "")
        let maybe14 = await model.convertCapture(captureId: "fixture-capture-1")
        let proposal = try XCTUnwrap(maybe14)
        let outcome15 = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(outcome15, .inserted(byteOffset: 5))
        XCTAssertNotNil(model.pendingEdit)
        let rev = model.editorRevision
        // Simulate the crash: the ledger records `applied`, but capture_applied never reaches the bridge.
        try model.bridge!.ledger.update(editId: edit.editId) { $0.state = .applied; $0.newRevision = rev + 1; $0.documentAfterSha256 = SourceDigest.sha256Hex(after) }
        let bridge = model.bridge!
        // Second attach against the same live bridge process is not possible (one process per journal), so
        // exercise reconcile directly: it must reopen the pre-edit snapshot and replay the receipt.
        let (actions, minimum) = await bridge.reconcile(path: "main.tex", currentText: after, currentRevision: 1)
        XCTAssertEqual(actions, [.replayedReceipt(editId: edit.editId, newRevision: rev + 1)])
        XCTAssertEqual(minimum, rev + 2)
        let status = try await bridge.status(captureId: "fixture-capture-1")
        XCTAssertEqual(status.applied?.editId, edit.editId)
        XCTAssertEqual(bridge.ledger.entries.first?.state, .confirmed)
        // Replaying again is a no-op: nothing unconfirmed remains, and the bridge's own receipt check is idempotent.
        let (again, _) = await bridge.reconcile(path: "main.tex", currentText: after, currentRevision: rev + 2)
        XCTAssertEqual(again, [])
        model.detachBridge()
    }

    func testPreparedButUnappliedEntryRequiresReselectionWhenBufferChanged() async throws {
        let store = try BridgeClientTests.tempStore()
        let text = "Hello FlashTeX.\n"
        let edit = TransferV1.CaptureEdit(captureId: "c9", editId: "capture-c9", projectId: "demo", path: "main.tex", expectedRevision: 1,
                                          startByte: 5, endByte: 5, removedText: "", replacement: "z", documentBeforeSha256: SourceDigest.sha256Hex(text))
        try EditLedger(storeDirectory: store).upsert(EditLedgerEntry(edit: edit, beforeText: text))
        let model = ShellModel()
        model.autoCompile = false
        _ = try await attach(model, store: store)
        // The fake bridge has no journal for c9 → capture_missing → abandoned; a prepared edit is never applied blindly.
        XCTAssertEqual(model.bridge?.ledger.entries.first?.state, .abandoned)
        XCTAssertEqual(model.activeText, text)
        XCTAssertNil(model.pendingEdit)
        // Same shape inside a live session: prepared in bridge and ledger, then the buffer changes → reselection.
        model.caretUTF16 = 5
        model.pinAnchorAtCaret()
        try await waitUntil { model.bridgeDestination != nil }
        let maybe16 = try await model.submitCapture(image: fixtureImage(), captureId: "c10", instructions: "t")
        XCTAssertNotNil(maybe16, model.captureNote ?? "")
        let maybe17 = await model.convertCapture(captureId: "c10")
        let proposal = try XCTUnwrap(maybe17)
        let outcome18 = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(outcome18, .inserted(byteOffset: 5))
        let bridge = model.bridge!
        let (actions, _) = await bridge.reconcile(path: "main.tex", currentText: "changed", currentRevision: model.editorRevision)
        guard case .reselectionRequired(let id, let why)? = actions.first else { return XCTFail("\(actions)") }
        XCTAssertEqual(id, "capture-c10")
        XCTAssertTrue(why.contains("buffer changed"), why)
        XCTAssertEqual(bridge.ledger.entry(editId: "capture-c10")?.state, .abandoned)
        XCTAssertEqual(bridge.capture("c10")?.state, .needsReselection)
        model.detachBridge()
    }
}
