import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// Fault/recovery tests for the bridge integration (issue #2 review of 48780a8,
/// plus the edit-ledger adoption addendum): receipt only after the helper's
/// durable commit (and the .tex export when file-backed), a corrupt/unreadable
/// durable store disables application, transient status failures keep
/// evidence, bridge writes never block the main thread, and a detached session
/// cannot overwrite its successor. Runs against `Fixtures/fake_bridge.py` and
/// `Fixtures/fake_edit_ledger.py`.
@MainActor
final class BridgeRecoveryTests: XCTestCase {
    typealias T = ShellModelBridgeTests

    private func attach(_ model: ShellModel, store: URL, ledgerStore: URL? = nil, ledger: ShellModel.LedgerLaunch? = T.fakeLedger) async -> Bool {
        await model.attachBridgeAndWait(executable: BridgeClientTests.python, arguments: [BridgeClientTests.fakeBridge.path],
                                        storeDirectory: store, ledger: ledger, discoverLedger: false, ledgerStore: ledgerStore)
    }

    /// Attach, pin at byte 5, submit, convert: returns the proposal.
    private func stage(_ model: ShellModel, store: URL, ledgerStore: URL? = nil, captureId: String = "fixture-capture-1") async throws -> RuntimeV1.CaptureProposal {
        let ok = await attach(model, store: store, ledgerStore: ledgerStore)
        XCTAssertTrue(ok, model.captureNote ?? model.bridgeStatus)
        model.caretUTF16 = 5
        model.pinAnchorAtCaret()
        try await T.waitUntil { model.bridgeDestination != nil }
        let received = await model.submitCapture(image: try BridgeClientTests.fixtureCapture().image, captureId: captureId, instructions: "t")
        XCTAssertNotNil(received, model.captureNote ?? "")
        let maybeProposal = await model.convertCapture(captureId: captureId)
        return try XCTUnwrap(maybeProposal, model.captureNote ?? "")
    }

    private func chmod(_ url: URL, _ mode: Int) throws {
        try FileManager.default.setAttributes([.posixPermissions: mode], ofItemAtPath: url.path)
    }

    // MARK: 1. receipt only after the durable commit (helper) and the .tex export

    func testPersistenceFailureInsertsNothingAndSendsNoReceipt() async throws {
        let store = try BridgeClientTests.tempStore()
        let ledgerStore = store.appendingPathComponent("documents/doc")
        let model = ShellModel()
        model.autoCompile = false
        let proposal = try await stage(model, store: store, ledgerStore: ledgerStore)
        let bridge = try XCTUnwrap(model.bridge)
        let before = model.activeText
        // Inject: the helper's store directory becomes read-only, so its atomic commit fails.
        try chmod(ledgerStore, 0o500)
        defer { try? chmod(ledgerStore, 0o700) }
        let outcome = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(outcome, .refused("ledger"), model.captureNote ?? "")
        XCTAssertNil(model.pendingEdit, "nothing staged for the editor")
        XCTAssertEqual(model.activeText, before, "nothing inserted")
        XCTAssertEqual(bridge.transactionTrace, [])
        XCTAssertNil(bridge.pendingTransaction)
        XCTAssertTrue(model.captureNote?.contains("nothing inserted") == true, model.captureNote ?? "")
        try await Task.sleep(nanoseconds: 200_000_000)
        let st = try await bridge.status(captureId: "fixture-capture-1")
        XCTAssertNil(st.applied, "no capture_applied may reach the bridge without a durable commit")
        XCTAssertNotNil(st.prepared, "the bridge prepared the edit; it stays reusable")
        // The helper was reopened after the poisoned handle and is usable again once the store is writable.
        XCTAssertTrue(bridge.ledgerUsable, bridge.ledgerStatus)
        let durable = try await bridge.ledger!.status()
        XCTAssertEqual(durable.document?.text, before)
        XCTAssertEqual(durable.pendingReceipts, [])

        // Repair and approve again: same prepared edit, now committed, adopted, acknowledged.
        try chmod(ledgerStore, 0o700)
        model.enqueue(proposal)
        let retry = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(retry, .inserted(byteOffset: 5), model.captureNote ?? "")
        XCTAssertEqual(bridge.transactionTrace, ["ledger"])
        let pending = try XCTUnwrap(model.pendingEdit)
        model.editApplied(pending, newText: "Hello\\fakecapture{fixture-capture-1} FlashTeX.\n")
        try await T.waitUntil { model.bridgeCaptures.last?.state == .confirmed }
        XCTAssertEqual(bridge.transactionTrace, ["ledger", "receipt", "confirmed"])
        model.detachBridge()
    }

    func testFileBackedDocumentIsExportedBeforeTheReceipt() async throws {
        let store = try BridgeClientTests.tempStore()
        let docURL = store.appendingPathComponent("doc.tex")
        try "Hello FlashTeX.\n".write(to: docURL, atomically: true, encoding: .utf8)
        let model = ShellModel()
        model.autoCompile = false
        model.openTex(at: docURL)
        let proposal = try await stage(model, store: store)
        let bridge = try XCTUnwrap(model.bridge)
        XCTAssertEqual(bridge.ledger?.storeDirectory.path, EditLedgerClient.storeDirectory(under: store, documentURL: docURL).path, "store keyed by the file")
        let outcome = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(outcome, .inserted(byteOffset: 5), model.captureNote ?? "")
        let after = "Hello\\fakecapture{fixture-capture-1} FlashTeX.\n"
        XCTAssertEqual(try String(contentsOf: docURL, encoding: .utf8), "Hello FlashTeX.\n", "export happens after adoption, not before")
        let pending = try XCTUnwrap(model.pendingEdit)
        model.editApplied(pending, newText: after)
        // Synchronously after adoption: durable commit, then the .tex export, then the receipt was queued.
        XCTAssertEqual(bridge.transactionTrace, ["ledger", "source", "receipt"])
        XCTAssertEqual(try String(contentsOf: docURL, encoding: .utf8), after, "the .tex holds the post-edit text before the receipt")
        XCTAssertFalse(model.isDirty, "the atomic export is the saved state")
        try await T.waitUntil { model.bridgeCaptures.last?.state == .confirmed }
        XCTAssertEqual(bridge.transactionTrace, ["ledger", "source", "receipt", "confirmed"])
        let durable = try await bridge.ledger!.status()
        XCTAssertEqual(durable.document?.text, after)
        XCTAssertEqual(durable.document?.sourceSha256, SourceDigest.sha256Hex(try String(contentsOf: docURL, encoding: .utf8)))
        model.detachBridge()
    }

    func testExportFailureWithholdsReceiptWhileTheDurableCommitStands() async throws {
        let store = try BridgeClientTests.tempStore()
        let model = ShellModel()
        model.autoCompile = false
        let proposal = try await stage(model, store: store)
        let bridge = try XCTUnwrap(model.bridge)
        model.documentURL = store.appendingPathComponent("missing-dir/doc.tex") // atomic write cannot succeed here
        model.savedText = model.activeText
        let outcome = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(outcome, .inserted(byteOffset: 5), model.captureNote ?? "")
        let after = "Hello\\fakecapture{fixture-capture-1} FlashTeX.\n"
        model.editApplied(try XCTUnwrap(model.pendingEdit), newText: after)
        XCTAssertEqual(bridge.transactionTrace, ["ledger"], "durable commit stands; export failed; no receipt")
        XCTAssertTrue(bridge.pendingTransaction?.lastFailure?.contains("export") == true, bridge.pendingTransaction?.lastFailure ?? "")
        XCTAssertTrue(model.bridgeStatus.contains("receipt withheld"), model.bridgeStatus)
        try await Task.sleep(nanoseconds: 150_000_000)
        let st = try await bridge.status(captureId: "fixture-capture-1")
        XCTAssertNil(st.applied)
        let held = try await bridge.ledger!.status()
        XCTAssertEqual(held.pendingReceipts.count, 1, "helper retains the transaction and its snapshot")
        // Edits made meanwhile are held back from the bridge, not sent ahead of the receipt.
        model.updateActiveText(after + "typed\n")
        try await Task.sleep(nanoseconds: 100_000_000)
        XCTAssertFalse(bridge.log.contains { $0.contains("refused") }, bridge.log.joined(separator: "\n"))
        // Saving somewhere writable records the export and releases the receipt; the deferred edit follows.
        model.documentURL = store.appendingPathComponent("doc.tex")
        XCTAssertTrue(model.saveTex())
        try await T.waitUntil { model.bridgeCaptures.last?.state == .confirmed }
        XCTAssertEqual(bridge.transactionTrace, ["ledger", "source", "receipt", "confirmed"])
        try await Task.sleep(nanoseconds: 150_000_000)
        XCTAssertFalse(bridge.log.contains { $0.contains("refused") }, "deferred document_edit applied after the receipt: \(bridge.log)")
        model.caretUTF16 = 0
        model.pinAnchorAtCaret()
        try await T.waitUntil { model.bridgeDestination != nil }
        XCTAssertEqual(model.bridgeDestination?.pinnedRevision, model.editorRevision)
        model.detachBridge()
    }

    // MARK: 2. corrupt / unreadable durable store fails closed

    func testCorruptOrUnreadableDurableStoreDisablesApplication() async throws {
        let store = try BridgeClientTests.tempStore()
        let ledgerStore = store.appendingPathComponent("documents/doc")
        try FileManager.default.createDirectory(at: ledgerStore, withIntermediateDirectories: true)
        let record = ledgerStore.appendingPathComponent("document.json")
        try Data("{not json".utf8).write(to: record)

        let model = ShellModel()
        model.autoCompile = false
        let proposal = try await stage(model, store: store, ledgerStore: ledgerStore)
        let bridge = try XCTUnwrap(model.bridge)
        XCTAssertFalse(bridge.ledgerUsable)
        XCTAssertTrue(bridge.ledgerError?.contains("unavailable") == true, bridge.ledgerError ?? "")
        XCTAssertTrue(model.bridgeStatus.contains("edit ledger unavailable"), model.bridgeStatus)
        XCTAssertTrue(bridge.reconciliationIncomplete)
        let outcome = await model.approveBridgeProposal(proposal, latex: proposal.latex)
        XCTAssertEqual(outcome, .refused("ledger"))
        XCTAssertNil(model.pendingEdit, "nothing is applied while the durable store is unusable")
        XCTAssertTrue(model.captureNote?.contains("Edit ledger unusable") == true, model.captureNote ?? "")
        XCTAssertEqual(try Data(contentsOf: record), Data("{not json".utf8), "a broken store is never overwritten")
        model.detachBridge()

        // Unreadable (permissions) is the same failure; a missing record is a fresh store.
        try Data("{}".utf8).write(to: record)
        try chmod(record, 0o000)
        defer { try? chmod(record, 0o600) }
        let model2 = ShellModel()
        model2.autoCompile = false
        let ok2 = await attach(model2, store: store, ledgerStore: ledgerStore)
        XCTAssertTrue(ok2)
        XCTAssertFalse(model2.bridge!.ledgerUsable)
        model2.detachBridge()
        try chmod(record, 0o600)
        try FileManager.default.removeItem(at: record)
        let model3 = ShellModel()
        model3.autoCompile = false
        let ok3 = await attach(model3, store: store, ledgerStore: ledgerStore)
        XCTAssertTrue(ok3)
        XCTAssertTrue(model3.bridge!.ledgerUsable, model3.bridge!.ledgerStatus)
        XCTAssertEqual(model3.bridge!.durable?.text, model3.activeText)
        model3.detachBridge()
    }

    // MARK: 3. transient status failures keep evidence

    /// Pre-populates a helper store with applied-but-unconfirmed edits (each
    /// inserting at byte 5) and returns the resulting document text.
    private func seedPendingTransactions(store: URL, captureIds: [String], text: String, undoAfterwards: Bool = false) async throws -> String {
        let client = try EditLedgerClient(executable: BridgeClientTests.python, arguments: [BridgeClientTests.fakeEditLedger.path],
                                          storeDirectory: store, queue: DispatchQueue(label: "seed"))
        var doc = try await client.initialize(.init(projectId: "demo", path: "main.tex", revision: 1, text: text))
        for id in captureIds {
            let edit = TransferV1.CaptureEdit(captureId: id, editId: "capture-\(id)", projectId: "demo", path: "main.tex", expectedRevision: doc.revision,
                                              startByte: 5, endByte: 5, removedText: "", replacement: "[\(id)]", documentBeforeSha256: doc.sourceSha256)
            doc = try await client.apply(edit).document
        }
        if undoAfterwards { doc = try await client.replaceDocument(expectedRevision: doc.revision, expectedSha256: doc.sourceSha256, text: text) }
        client.terminate()
        try await Task.sleep(nanoseconds: 100_000_000) // release the store lock
        return doc.text
    }

    func testTransientStatusFailuresRetainTransactionsAndEvidence() async throws {
        let store = try BridgeClientTests.tempStore()
        let ledgerStore = store.appendingPathComponent("documents/doc")
        let model = ShellModel()
        model.autoCompile = false
        model.bridgeStatusTimeout = 1
        let text = model.activeText
        let durableText = try await seedPendingTransactions(store: ledgerStore, captureIds: ["err-storage_error-1", "garbage-1", "err-capture_missing-1"], text: text)
        let ok = await attach(model, store: store, ledgerStore: ledgerStore)
        XCTAssertTrue(ok, model.captureNote ?? model.bridgeStatus)
        let bridge = try XCTUnwrap(model.bridge)
        // The durable document (with the three insertions) is authoritative: adopted as one undoable edit.
        let adoption = try XCTUnwrap(model.pendingEdit)
        XCTAssertEqual(adoption.text, durableText)
        XCTAssertEqual(adoption.nsRange, NSRange(location: 0, length: (text as NSString).length))
        XCTAssertGreaterThanOrEqual(model.editorRevision, 4)
        // Bridge-side error that is not a missing/conflicting record, and a garbage reply (no answer
        // within the deadline): transactions untouched, evidence kept, retry offered.
        XCTAssertTrue(bridge.reconciliationIncomplete)
        XCTAssertTrue(model.captureNote?.contains("Reconciliation incomplete") == true, model.captureNote ?? "")
        let st = try await bridge.ledger!.status()
        XCTAssertEqual(st.pendingReceipts.count, 3, "no transaction was confirmed or dropped")
        for tx in st.pendingReceipts { XCTAssertNotNil(tx.documentBefore, "snapshot retained for \(tx.edit.captureId)") }
        XCTAssertEqual(bridge.needsReconciliation.keys.sorted(), ["capture-err-capture_missing-1"])
        XCTAssertTrue(bridge.needsReconciliation["capture-err-capture_missing-1"]?.contains("evidence kept") == true)
        XCTAssertEqual(bridge.transactions.values.filter { !$0.confirmed }.count, 3)
        // Adopting the durable text keeps store and editor aligned (no divergence).
        model.editApplied(adoption, newText: durableText)
        try await T.waitUntil { bridge.durable?.revision == model.editorRevision }
        XCTAssertNil(bridge.ledgerError, bridge.ledgerError ?? "")
        // Explicit resolution is the only path that drops evidence (helper confirm).
        try await bridge.resolveReconciliation(editId: "capture-err-capture_missing-1", note: "operator verified the insertion is in the document")
        let resolved = try await bridge.ledger!.status()
        XCTAssertEqual(resolved.pendingReceipts.map(\.edit.captureId).sorted(), ["err-storage_error-1", "garbage-1"])
        XCTAssertTrue(bridge.needsReconciliation.isEmpty)
        model.detachBridge()
    }

    func testMissingReceiptIsReplayedFromTheRetainedSnapshot() async throws {
        let store = try BridgeClientTests.tempStore()
        let text = "Hello FlashTeX.\n"
        // Bring the fake bridge to "prepared" for a capture (its journal is in-memory, so the
        // reconciliation below runs against this live session rather than a process restart).
        let seed = ShellModel()
        seed.autoCompile = false
        let seeded = await attach(seed, store: store, ledger: nil)
        XCTAssertTrue(seeded)
        seed.caretUTF16 = 5
        seed.pinAnchorAtCaret()
        try await T.waitUntil { seed.bridgeDestination != nil }
        _ = await seed.submitCapture(image: try BridgeClientTests.fixtureCapture().image, captureId: "fixture-capture-1", instructions: "t")
        _ = await seed.convertCapture(captureId: "fixture-capture-1")
        let bridge = seed.bridge!
        let prepared = try await bridge.prepare(captureId: "fixture-capture-1", expectedRevision: seed.editorRevision)
        XCTAssertEqual(prepared.replacement, "\\fakecapture{fixture-capture-1}")

        // A helper store holding a *different* applied edit for the same capture: held, never replayed blindly.
        let ledgerStore = store.appendingPathComponent("documents/doc")
        _ = try await seedPendingTransactions(store: ledgerStore, captureIds: ["fixture-capture-1"], text: text, undoAfterwards: true)
        let open1 = await bridge.openLedger(executable: BridgeClientTests.python, arguments: [BridgeClientTests.fakeEditLedger.path],
                                            store: ledgerStore, path: "main.tex", currentText: text, currentRevision: seed.editorRevision)
        guard case .aligned = open1 else { return XCTFail("\(open1)") }
        let (held, _) = await bridge.reconcile(path: "main.tex", currentSource: { (text, seed.editorRevision) }, statusTimeout: 2)
        guard case .needsReconciliation(let id, let why)? = held.first else { return XCTFail("\(held)") }
        XCTAssertEqual(id, "capture-fixture-capture-1")
        XCTAssertTrue(why.contains("no matching prepared edit"), why)
        let kept = try await bridge.ledger!.status()
        XCTAssertEqual(kept.pendingReceipts.count, 1, "evidence kept")

        // The genuine case: a helper transaction whose edit matches the bridge's prepared edit exactly
        // (applied durably, receipt lost). Reconciliation reopens the pre-edit snapshot and replays it.
        let ledgerStore2 = store.appendingPathComponent("documents/doc2")
        let client = try EditLedgerClient(executable: BridgeClientTests.python, arguments: [BridgeClientTests.fakeEditLedger.path],
                                          storeDirectory: ledgerStore2, queue: DispatchQueue(label: "seed2"))
        _ = try await client.initialize(.init(projectId: "demo", path: "main.tex", revision: prepared.expectedRevision, text: text))
        let applied = try await client.apply(prepared)
        let afterText = applied.document.text
        client.terminate()
        try await Task.sleep(nanoseconds: 100_000_000)
        let open2 = await bridge.openLedger(executable: BridgeClientTests.python, arguments: [BridgeClientTests.fakeEditLedger.path],
                                            store: ledgerStore2, path: "main.tex", currentText: afterText, currentRevision: applied.receipt.newRevision)
        guard case .aligned = open2 else { return XCTFail("\(open2)") }
        let (replay, minimum) = await bridge.reconcile(path: "main.tex", currentSource: { (afterText, applied.receipt.newRevision) }, statusTimeout: 2)
        XCTAssertEqual(replay, [.replayedReceipt(editId: prepared.editId, newRevision: applied.receipt.newRevision)])
        XCTAssertEqual(minimum, applied.receipt.newRevision + 1)
        let st = try await bridge.status(captureId: "fixture-capture-1")
        XCTAssertEqual(st.applied?.editId, prepared.editId, "the bridge now holds the replayed receipt")
        try await T.waitUntil { bridge.transactions[prepared.editId]?.confirmed == true }
        let dropped = try await bridge.ledger!.status()
        XCTAssertEqual(dropped.pendingReceipts, [], "helper dropped the snapshot after the exact acknowledgement")
        // Replaying again is a no-op.
        let (again, _) = await bridge.reconcile(path: "main.tex", currentSource: { (afterText, applied.receipt.newRevision + 1) }, statusTimeout: 2)
        XCTAssertEqual(again, [])
        seed.detachBridge()
    }

    // MARK: 4. writes never block the main thread

    func testStalledBridgeDoesNotBlockMainThreadAndBoundsQueuedBytes() async throws {
        let store = try BridgeClientTests.tempStore()
        let model = ShellModel()
        model.autoCompile = false
        let ok = await attach(model, store: store)
        XCTAssertTrue(ok)
        let bridge = try XCTUnwrap(model.bridge)
        model.caretUTF16 = 5
        model.pinAnchorAtCaret()
        try await T.waitUntil { model.bridgeDestination != nil }
        let image = try BridgeClientTests.fixtureCapture().image
        // Main-thread heartbeat: must keep ticking while the bridge stops reading stdin.
        var ticks = 0
        var maxGap: TimeInterval = 0
        var lastTick = Date()
        let heartbeat = DispatchSource.makeTimerSource(queue: .main)
        heartbeat.schedule(deadline: .now(), repeating: .milliseconds(25))
        heartbeat.setEventHandler { ticks += 1; let now = Date(); maxGap = max(maxGap, now.timeIntervalSince(lastTick)); lastTick = now }
        heartbeat.resume()
        defer { heartbeat.cancel() }

        // Requests are issued directly on the main thread so the measurement is of send() itself.
        let destination = try XCTUnwrap(model.bridgeDestination)
        func submit(_ id: String, _ img: RuntimeV1.CaptureImage, _ instructions: String) -> RuntimeV1.CaptureSubmit {
            .init(captureId: id, destinationId: destination.destinationId, baseRevision: destination.pinnedRevision, image: img, instructions: instructions)
        }
        var stalledReply: Result<TransferV1.CaptureReceived, BridgeClient.Failure>?
        var largeReply: Result<TransferV1.CaptureReceived, BridgeClient.Failure>?
        // 1. A "slow conversion": the fake bridge sleeps 2 s before replying and reads nothing meanwhile.
        bridge.client.send(.captureSubmit, submit("stall-1", image, "%stall:2"), as: TransferV1.CaptureReceived.self) { stalledReply = $0 }
        try await Task.sleep(nanoseconds: 100_000_000)
        // 2. A multi-MB capture behind it: fills the pipe; the write blocks the I/O queue only.
        let big = RuntimeV1.CaptureImage(mimeType: "image/png", dataBase64: Data(repeating: 0x41, count: 3 * 1024 * 1024).base64EncodedString())
        let start = Date()
        bridge.client.send(.captureSubmit, submit("large-1", big, "t"), as: TransferV1.CaptureReceived.self) { largeReply = $0 }
        let sendDuration = Date().timeIntervalSince(start)
        XCTAssertLessThan(sendDuration, 0.2, "send() returned to the main thread immediately (took \(sendDuration) s)")
        XCTAssertGreaterThan(bridge.client.pendingWriteBytes, 3 * 1024 * 1024, "the large request is queued behind the stalled reader")
        try await Task.sleep(nanoseconds: 800_000_000)
        XCTAssertGreaterThan(bridge.client.pendingWriteBytes, 0, "still queued: the bridge has not resumed reading")
        XCTAssertGreaterThanOrEqual(ticks, 20, "main thread kept running during the stall (\(ticks) ticks)")
        XCTAssertLessThan(maxGap, 0.5, "a blocked main thread shows as a multi-second heartbeat gap (max gap \(maxGap) s)")
        XCTAssertLessThan(Date().timeIntervalSince(start), 1.5, "the main actor was not held for the duration of the stall")
        XCTAssertNil(bridge.pendingTransaction)
        // 3. Backpressure: beyond the bound, requests are refused immediately with a clear failure.
        let filler = String(repeating: "x", count: 11 * 1024 * 1024)
        var refused: BridgeClient.Failure?
        let refusal = expectation(description: "backpressure")
        for i in 0..<3 {
            bridge.client.send(.documentOpen, TransferV1.DocumentOpen(projectId: "demo", path: "big\(i).tex", revision: 1, text: filler), as: TransferV1.Empty.self) { result in
                if case .failure(let f) = result, case .backpressure = f, refused == nil { refused = f; refusal.fulfill() }
            }
        }
        await fulfillment(of: [refusal], timeout: 2)
        XCTAssertTrue(refused?.text.contains("not reading") == true, refused?.text ?? "")
        XCTAssertGreaterThanOrEqual(ticks, 20)
        // The stalled and the large capture both complete once the bridge resumes reading.
        try await T.waitUntil(timeout: 30) { stalledReply != nil && largeReply != nil }
        XCTAssertEqual(try stalledReply?.get().durable, true)
        XCTAssertEqual(try largeReply?.get().durable, true)
        XCTAssertGreaterThan(Date().timeIntervalSince(start), 1.0, "the large request really waited for the stall")
        try await T.waitUntil(timeout: 30) { bridge.client.pendingWriteBytes == 0 }
        model.detachBridge()
    }

    // MARK: 5. detached sessions cannot overwrite their successor

    func testDetachAndReattachIgnoresOldSessionEvents() async throws {
        let model = ShellModel()
        model.autoCompile = false
        model.bridgeStatusTimeout = 3
        let storeA = try BridgeClientTests.tempStore(), storeB = try BridgeClientTests.tempStore()
        let text = model.activeText
        // A's attach is still awaiting a slow capture_status (seeded pending transaction) when it is detached.
        let ledgerA = storeA.appendingPathComponent("documents/doc")
        _ = try await seedPendingTransactions(store: ledgerA, captureIds: ["stall-1-a"], text: text, undoAfterwards: true)
        let attachA = Task { await self.attach(model, store: storeA, ledgerStore: ledgerA) }
        try await Task.sleep(nanoseconds: 300_000_000)
        let sessionA = try XCTUnwrap(model.bridge)
        model.detachBridge()
        XCTAssertFalse(model.bridgeAttached)
        let okB = await attach(model, store: storeB)
        XCTAssertTrue(okB, model.captureNote ?? model.bridgeStatus)
        let sessionB = try XCTUnwrap(model.bridge)
        XCTAssertFalse(sessionA === sessionB)
        let resultA = await attachA.value
        XCTAssertFalse(resultA, "the detached attach must report failure, not open on top of B")
        // A's queued exit/termination events and its finished reconcile land now; B's state must survive them.
        try await Task.sleep(nanoseconds: 1_300_000_000)
        XCTAssertFalse(sessionA.running)
        XCTAssertTrue(sessionB.running)
        // Deterministically fire a late callback from the old session (its onChange runs synchronously).
        sessionA.terminate()
        XCTAssertEqual(sessionA.status, "bridge detached")
        XCTAssertNotEqual(model.bridgeStatus, sessionA.status, "a detached session's callback must not reach the model")
        XCTAssertTrue(model.bridgeStatus.contains("open at revision"), model.bridgeStatus)
        XCTAssertFalse(model.bridgeStatus.contains("exited") || model.bridgeStatus.contains("detached"), model.bridgeStatus)
        XCTAssertEqual(model.bridge?.storeDirectory, storeB)
        // B is fully usable.
        model.caretUTF16 = 5
        model.pinAnchorAtCaret()
        try await T.waitUntil { model.bridgeDestination != nil }
        XCTAssertEqual(model.bridgeDestination?.pinnedRevision, model.editorRevision)
        // Immediate detach/reattach without an await in between.
        model.detachBridge()
        let storeC = try BridgeClientTests.tempStore()
        let okC = await attach(model, store: storeC)
        XCTAssertTrue(okC)
        try await Task.sleep(nanoseconds: 300_000_000)
        XCTAssertTrue(model.bridgeAttached)
        XCTAssertTrue(model.bridgeStatus.contains("open at revision"), model.bridgeStatus)
        model.detachBridge()
    }

    // MARK: edits while reconciliation is awaiting the bridge

    func testEditsDuringReconciliationAreCarriedByTheInitialOpen() async throws {
        let store = try BridgeClientTests.tempStore()
        let ledgerStore = store.appendingPathComponent("documents/doc")
        let model = ShellModel()
        model.autoCompile = false
        model.bridgeStatusTimeout = 3
        let text = model.activeText
        _ = try await seedPendingTransactions(store: ledgerStore, captureIds: ["stall-1-e"], text: text, undoAfterwards: true)
        let attachTask = Task { await self.attach(model, store: store, ledgerStore: ledgerStore) }
        try await Task.sleep(nanoseconds: 300_000_000)
        XCTAssertNotNil(model.bridge)
        model.updateActiveText("edited while reconciling\n")
        model.updateActiveText("edited twice while reconciling\n")
        let revisionDuring = model.editorRevision
        let attached = await attachTask.value
        XCTAssertTrue(attached, model.captureNote ?? model.bridgeStatus)
        let bridge = try XCTUnwrap(model.bridge)
        // No document_edit went out before document_open (that would be document_missing → refused).
        XCTAssertFalse(bridge.log.contains { $0.contains("refused") }, bridge.log.joined(separator: "\n"))
        XCTAssertEqual(bridge.shadow["main.tex"]?.text, "edited twice while reconciling\n")
        XCTAssertEqual(bridge.shadow["main.tex"]?.revision, revisionDuring)
        // The durable document followed the edits (ordinary replaces); the pending transaction was
        // held (fake bridge has no record → needsReconciliation) with its evidence intact.
        try await T.waitUntil { bridge.durable?.revision == revisionDuring }
        XCTAssertEqual(bridge.durable?.text, "edited twice while reconciling\n")
        XCTAssertNil(bridge.ledgerError, bridge.ledgerError ?? "")
        XCTAssertEqual(bridge.needsReconciliation.keys.sorted(), ["capture-stall-1-e"])
        let st = try await bridge.ledger!.status()
        XCTAssertEqual(st.pendingReceipts.count, 1)
        // The bridge holds the edited text: a pin at the current revision and range succeeds.
        model.caretUTF16 = 6
        model.pinAnchorAtCaret()
        try await T.waitUntil { model.bridgeDestination != nil }
        XCTAssertEqual(model.bridgeDestination?.pinnedRevision, revisionDuring)
        XCTAssertEqual(model.bridgeDestination?.binding.sourceSha256, SourceDigest.sha256Hex("edited twice while reconciling\n"))
        model.detachBridge()
    }
}
