import AppKit
import SwiftUI
import XCTest
@testable import FlashTeXProtocol
@testable import FlashTeXMac

/// AI review (item 15) replenishment: proposal cancellation at EVERY helper/
/// provider stage, out-of-order reply rejection (earlier stage after a later
/// one, another proposal's context, replies after approval), and reviewed-
/// insertion recovery (the editor refuses the approved insertion because the
/// buffer moved → the proposal returns to review with the exact reason → the
/// next approval inserts exactly once through the real preview-controller;
/// the helper killed mid-`approve` loses nothing). The REAL
/// `flashtex-assistant-context` helper does every helper stage, behind
/// `Fixtures/delaying_assistant_context_proxy.py` when a stage must be caught
/// mid-flight; the provider is `Fixtures/controlled_assistant_provider.py`.
/// No network, no model, no credentials; nothing here is evidence of any
/// live provider.
@MainActor
final class ReviewRecoveryTests: XCTestCase {
    static let fixtures = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("Fixtures")
    static let proxy = fixtures.appendingPathComponent("delaying_assistant_context_proxy.py")
    static let controlledProvider = fixtures.appendingPathComponent("controlled_assistant_provider.py")

    private var scratch: URL!
    private var helperControl: URL!
    private var helperPid: URL!
    private var helperLog: URL!
    private var providerControl: URL!
    private var providerPid: URL!
    private var providerLog: URL!

    override func setUpWithError() throws {
        scratch = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent("flashtex-review-recovery-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: scratch, withIntermediateDirectories: true)
        helperControl = scratch.appendingPathComponent("helper-control.json")
        helperPid = scratch.appendingPathComponent("helper.pid")
        helperLog = scratch.appendingPathComponent("helper.log")
        providerControl = scratch.appendingPathComponent("provider-control.json")
        providerPid = scratch.appendingPathComponent("provider.pid")
        providerLog = scratch.appendingPathComponent("provider.log")
        try setHelperDelays([:])
        try setProvider(delay: 0)
    }

    override func tearDownWithError() throws {
        if let scratch { try? FileManager.default.removeItem(at: scratch) }
    }

    // MARK: fixtures

    private func realHelper() throws -> URL {
        guard let helper = ProposalPreview.ExplanationConfiguration.locateHelper() else {
            throw XCTSkip("build crates/assistant-context or set FLASHTEX_ASSISTANT_CONTEXT")
        }
        return helper
    }

    private static func env(_ name: String) -> URL? {
        guard let p = ProcessInfo.processInfo.environment[name], FileManager.default.isExecutableFile(atPath: p) else { return nil }
        return URL(fileURLWithPath: p)
    }

    private func loadAverage() -> Double {
        var l = [Double](repeating: 0, count: 3)
        return getloadavg(&l, 3) >= 1 ? l[0] : -1
    }

    /// Per-operation delays for the proxy (`probe` = the first prepare call).
    private func setHelperDelays(_ delays: [String: Double]) throws {
        try? FileManager.default.removeItem(at: helperPid)
        let obj: [String: Any] = ["delay_s": delays, "pid_file": helperPid.path, "log_file": helperLog.path]
        try JSONSerialization.data(withJSONObject: obj).write(to: helperControl)
    }

    private func setProvider(delay: TimeInterval) throws {
        try? FileManager.default.removeItem(at: providerPid)
        let obj: [String: Any] = ["delay_s": delay, "mode": "answer", "pid_file": providerPid.path, "log_file": providerLog.path]
        try JSONSerialization.data(withJSONObject: obj).write(to: providerControl)
    }

    private func lines(_ url: URL) -> [String] {
        ((try? String(contentsOf: url, encoding: .utf8)) ?? "").split(separator: "\n").map(String.init)
    }

    /// Real helper behind the delaying proxy (`python3 proxy.py <helper> <control>`), controlled provider.
    private func configuration(helper: URL, proxied: Bool = true) -> ProposalPreview.ExplanationConfiguration {
        var c: ProposalPreview.ExplanationConfiguration
        if proxied {
            c = .init(helper: WorkerClientTests.python, helperArguments: [Self.proxy.path, helper.path, helperControl.path])
        } else {
            c = .init(helper: helper)
        }
        c.provider = WorkerClientTests.python
        c.providerArguments = [Self.controlledProvider.path, providerControl.path]
        c.providerTimeout = 5
        c.helperTimeout = 10
        return c
    }

    private func makePreview(_ explanation: ProposalPreview.ExplanationConfiguration) -> ProposalPreview {
        ProposalPreview(executable: WorkerClientTests.python, arguments: [WorkerClientTests.fakeWorker.path], explanation: explanation)
    }

    private func input(_ text: String, anchorByte: Int, revision: Int = 1, projectId: String = "recovery2") -> ProposalPreview.Input {
        let ctx = String(decoding: Array(text.utf8.dropFirst(anchorByte).prefix(Insertion.contextLength)), as: UTF8.self)
        return .init(documents: [.init(path: "main.tex", text: text)], entryPath: "main.tex",
                     anchor: InsertionAnchor(id: "a1", path: "main.tex", byteOffset: anchorByte, revision: revision, contextAfter: ctx),
                     editorRevision: revision, projectId: projectId)
    }

    private func waitUntil(_ what: String, timeout: TimeInterval = 10, file: StaticString = #filePath, line: UInt = #line,
                           _ cond: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if cond() { return }
            try await Task.sleep(nanoseconds: 20_000_000)
        }
        XCTFail("timed out waiting for \(what)", file: file, line: line)
    }

    private func waitForReady(_ preview: ProposalPreview) async throws {
        try await waitUntil("preview ready") { if case .ready = preview.state { return true }; return false }
    }

    private func waitForSettled(_ preview: ProposalPreview) async throws {
        try await waitUntil("explanation settled") { !preview.explanationInFlight }
    }

    /// Waits until the current request is running at `stage` (the launch is
    /// recorded synchronously; the child pid follows once it wrote its pid file).
    private func waitForStage(_ preview: ProposalPreview, _ stage: ProposalPreview.ExplanationStage) async throws -> Int32 {
        try await waitUntil("stage \(stage) launched") { preview.childLaunches.last?.stage == stage && preview.explanationInFlight }
        let file = stage == .provider ? providerPid! : helperPid!
        let running = try XCTUnwrap(preview.runningChildProcessIdentifier, "a child is running at \(stage)")
        // The pid file may still hold an earlier stage's pid for a moment.
        try await waitUntil("\(stage) pid file names \(running)") {
            (try? String(contentsOf: file, encoding: .utf8)).flatMap { Int32($0.trimmingCharacters(in: .whitespacesAndNewlines)) } == running
        }
        return running
    }

    private func processIsGone(_ pid: Int32) -> Bool { kill(pid, 0) == -1 && errno == ESRCH }

    private func waitForProcessGone(_ pid: Int32) async throws {
        try await waitUntil("process \(pid) gone") { [self] in processIsGone(pid) }
    }

    private func closeAndAssertWorkerStopped(_ preview: ProposalPreview) async throws {
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    private func forged(type: String, contextId: String, reviewId: String? = nil, extra: [String: Any] = [:]) throws -> Data {
        var obj: [String: Any] = ["type": type, "applied": false,
                                  "payload": ["context_id": contextId, "explanation": "forged", "edits": []] as [String: Any]]
        if let reviewId { obj["review_id"] = reviewId; obj["requires_user_approval"] = true }
        for (k, v) in extra { obj[k] = v }
        return try JSONSerialization.data(withJSONObject: obj)
    }

    // MARK: 1. cancellation at every stage

    /// Reviewer Cancel at probe, prepare, provider, review and approve; the
    /// sheet closing mid-prepare; the proposal rejected mid-provider. Every
    /// child is stopped by the pid we launched, its late reply is discarded,
    /// the recovery log names the stage, and nothing reaches the document.
    func testReviewerCancelAtEveryStageDiscardsLateRepliesAndAppliesNothing() async throws {
        let helper = try realHelper()
        let model = ShellModel()
        XCTAssertNil(model.loadError, model.loadError ?? "")
        let docs = model.documents, revision = model.editorRevision
        model.caretUTF16 = 0
        model.pinAnchorAtCaret()
        let proposal = RuntimeV1.CaptureProposal(captureId: "cap-stages", latex: "%diag:0 inserted", ambiguities: [], requiredDependencies: [])
        model.enqueue(proposal)
        let preview = makePreview(configuration(helper: helper))
        preview.update(from: model, latex: proposal.latex)
        try await waitForReady(preview)
        func assertUntouched(_ step: String) {
            XCTAssertEqual(model.documents, docs, step)
            XCTAssertEqual(model.editorRevision, revision, step)
            XCTAssertNil(model.pendingEdit, step)
            XCTAssertEqual(model.proposals.map(\.captureId), ["cap-stages"], step)
            XCTAssertNil(preview.amendedProposalLatex, step)
        }

        var requestNumber = 0
        for stage in [ProposalPreview.ExplanationStage.probe, .prepare, .provider, .review, .approve] {
            let hint = stage == .provider ? "provider" : "\(stage)"
            try setHelperDelays(stage == .provider ? [:] : [hint: 0.8])
            try setProvider(delay: stage == .provider ? 0.8 : 0)
            if stage == .approve {
                // A reviewed edit must be displayed before approve can run.
                try setHelperDelays([:])
                preview.explain()
                requestNumber += 1
                try await waitForSettled(preview)
                guard case .ready(let e) = preview.explanationState, e.reviewId != nil else { return XCTFail("\(preview.explanationState)") }
                try setHelperDelays(["approve": 0.8])
                preview.approveReviewedEdit()
                guard case .approving = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
            } else {
                preview.explain()
                requestNumber += 1
            }
            let id = "explain-\(requestNumber)"
            let pid = try await waitForStage(preview, stage)
            let launchesAtCancel = preview.childLaunches.count
            let staleBefore = preview.staleExplanationReplies

            preview.cancelExplanationByReviewer()
            XCTAssertEqual(preview.explanationState, .cancelled("cancelled by reviewer"), "\(stage)")
            XCTAssertTrue(preview.explanationState.isIdleForReviewer)
            XCTAssertFalse(preview.explanationInFlight)
            XCTAssertNil(preview.runningChildProcessIdentifier)
            XCTAssertEqual(preview.recoveryLog.last, .cancelledByReviewer(requestId: id, stage: stage))
            XCTAssertEqual(preview.recoveryNoteText, "cancelled during \(stage); any late reply is discarded. Explain starts a new request.")
            try await waitForProcessGone(pid)
            try await waitUntil("late \(stage) reply discarded") { preview.staleExplanationReplies == staleBefore + 1 }
            XCTAssertEqual(preview.recoveryLog.last, .lateReplyDiscarded(requestId: id, stage: stage))
            XCTAssertEqual(preview.explanationState, .cancelled("cancelled by reviewer"), "\(stage): the late reply changed nothing")
            XCTAssertEqual(preview.childLaunches.count, launchesAtCancel, "\(stage): no later stage was launched after the cancel")
            XCTAssertFalse(preview.canApproveReviewedEdit)
            XCTAssertTrue(preview.canExplain, "\(stage): the sheet is usable again")
            guard case .ready = preview.state else { return XCTFail("\(stage): \(preview.state)") }
            assertUntouched("after cancel at \(stage)")
        }
        // The proxy saw the helper launched (and stopped before answering) at each helper stage.
        let helperRuns = lines(helperLog).map { $0.split(separator: " ").dropFirst().joined(separator: " ") }
        XCTAssertEqual(helperRuns.prefix(3), ["prepare probe", "prepare probe", "prepare prepare"], "\(helperRuns)")
        XCTAssertEqual(helperRuns.filter { $0 == "approve approve" }.count, 1)
        XCTAssertEqual(lines(providerLog).count, 3, "provider: cancelled run, the review-stage run, the approve setup run")

        // The sheet closes (approve-for-insertion / reject / dismiss) mid-prepare:
        // the child is stopped, the reply discarded unseen, the worker gone.
        try setHelperDelays(["prepare": 0.8])
        preview.explain()
        requestNumber += 1
        let closing = try await waitForStage(preview, .prepare)
        preview.close()
        XCTAssertEqual(preview.explanationState, .cancelled("review closed"))
        XCTAssertEqual(preview.recoveryLog.last, .reviewClosed(requestId: "explain-\(requestNumber)", stage: .prepare))
        XCTAssertEqual(preview.recoveryNoteText, "review closed during prepare; the request was stopped and any late reply is discarded. Nothing was applied.")
        try await waitForProcessGone(closing)
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
        try await waitUntil("late reply after close discarded") { preview.recoveryLog.last == .lateReplyDiscarded(requestId: "explain-\(requestNumber)", stage: .prepare) }
        XCTAssertEqual(preview.explanationState, .cancelled("review closed"))
        assertUntouched("after close")

        // The proposal is REJECTED while the provider runs (the sheet's Reject
        // then dismiss): the queue empties, the provider is stopped, nothing is
        // inserted, and the late provider reply lands on a closed preview.
        try setHelperDelays([:])
        try setProvider(delay: 0.8)
        let second = makePreview(configuration(helper: helper))
        second.update(from: model, latex: proposal.latex)
        try await waitForReady(second)
        second.explain()
        let providerPid = try await waitForStage(second, .provider)
        model.rejectProposal(proposal)
        second.close()
        XCTAssertTrue(model.proposals.isEmpty)
        XCTAssertNil(model.reviewing)
        XCTAssertEqual(second.recoveryLog.last, .reviewClosed(requestId: "explain-1", stage: .provider))
        try await waitForProcessGone(providerPid)
        try await waitUntil("late provider reply discarded") { second.staleExplanationReplies == 1 }
        XCTAssertEqual(second.childLaunches.map(\.stage), [.probe, .prepare, .provider], "no review followed")
        XCTAssertEqual(model.documents, docs)
        XCTAssertNil(model.pendingEdit)
        XCTAssertFalse(model.appliedCaptureIDs.contains("cap-stages"))
        try await waitUntil("second worker terminated") { !second.workerIsRunning }
    }

    // MARK: 2. out-of-order rejection

    /// Replies for an EARLIER stage than the current one, replies carrying
    /// ANOTHER proposal's context under the same request id, and replies
    /// after the reviewed edit / the insertion were approved are refused.
    func testOutOfOrderRepliesForEarlierStagesOtherProposalsAndApprovedRequestsAreRefused() async throws {
        let helper = try realHelper()
        let model = ShellModel()
        XCTAssertNil(model.loadError, model.loadError ?? "")
        model.caretUTF16 = 0
        model.pinAnchorAtCaret()
        let proposal = RuntimeV1.CaptureProposal(captureId: "cap-order", latex: "%diag:0 inserted", ambiguities: [], requiredDependencies: [])
        model.enqueue(proposal)
        let a = makePreview(configuration(helper: helper))
        a.update(from: model, latex: proposal.latex)
        try await waitForReady(a)

        // (i) Stage N-1 after stage N, same request id: while explain-1 waits for
        // the provider, well-formed probe/prepare replies for explain-1 are dropped.
        try setProvider(delay: 0.8)
        a.explain()
        let providerPid = try await waitForStage(a, .provider)
        guard case .awaitingProvider(let c1) = a.explanationState else { return XCTFail("\(a.explanationState)") }
        let prepared = try JSONSerialization.data(withJSONObject: ["type": "prepared_context", "payload": try JSONSerialization.jsonObject(with: c1.payload)])
        a.handleExplanationReply(id: "explain-1", stage: .probe, result: .success(.init(stdout: prepared, stderr: "")))
        a.handleExplanationReply(id: "explain-1", stage: .prepare, result: .success(.init(stdout: prepared, stderr: "")))
        a.handleExplanationReply(id: "explain-1", stage: .prepare, result: .failure(.exited(1, output: Data(), stderr: "forged")))
        XCTAssertEqual(a.explanationState, .awaitingProvider(c1))
        XCTAssertEqual(a.staleExplanationReplies, 3)
        XCTAssertEqual(a.recoveryLog.events, [
            .lateReplyDiscarded(requestId: "explain-1", stage: .probe),
            .lateReplyDiscarded(requestId: "explain-1", stage: .prepare),
            .lateReplyDiscarded(requestId: "explain-1", stage: .prepare),
        ])
        XCTAssertEqual(a.childLaunches.count, 3, "a dropped reply never launches a stage")
        try await waitForSettled(a)
        try await waitForProcessGone(providerPid)
        guard case .ready(let e1) = a.explanationState, let review1 = e1.reviewId else { return XCTFail("\(a.explanationState)") }
        // Now at .done: a provider reply (N-1) and a re-delivered review reply for explain-1 are dropped.
        let providerReply = try JSONSerialization.data(withJSONObject: ["context_id": c1.contextId, "explanation": "again", "edits": []])
        a.handleExplanationReply(id: "explain-1", stage: .provider, result: .success(.init(stdout: providerReply, stderr: "")))
        a.handleExplanationReply(id: "explain-1", stage: .review, result: .success(.init(stdout: try forged(type: "proposal_review", contextId: c1.contextId, reviewId: review1), stderr: "")))
        a.handleExplanationReply(id: "explain-1", stage: .done, result: .success(.init(stdout: Data(), stderr: "")))
        XCTAssertEqual(a.explanationState, .ready(e1))
        XCTAssertEqual(a.staleExplanationReplies, 6)
        XCTAssertEqual(a.childLaunches.count, 4)

        // (ii) Another proposal's reply under the same request id: preview B reviews
        // a different capture (its own explain-1); its context id in a
        // well-formed proposal_review for A's explain-2 at A's review stage is
        // refused as answering another context, never displayed.
        let other = RuntimeV1.CaptureProposal(captureId: "cap-other", latex: "%diag:0 other proposal", ambiguities: [], requiredDependencies: [])
        let b = makePreview(configuration(helper: helper))
        b.update(input: input("B\n%diag:1 tail\n", anchorByte: 2, projectId: "other"), latex: other.latex)
        try await waitForReady(b)
        try setProvider(delay: 0)
        b.explain()
        try await waitForSettled(b)
        guard case .ready(let eb) = b.explanationState, let reviewB = eb.reviewId else { return XCTFail("\(b.explanationState)") }
        XCTAssertEqual(eb.context.requestId, "explain-1", "B numbers its own requests: same id as A's first")
        XCTAssertNotEqual(eb.context.contextId, c1.contextId)

        a.update(from: model, latex: "%diag:0 inserted again")
        try await waitUntil("A recompiled") { a.shadowCompileCount == 2 && !a.isInFlight }
        try setHelperDelays(["review": 0.8])
        a.explain()
        let reviewPid = try await waitForStage(a, .review)
        guard case .validating(let c2) = a.explanationState else { return XCTFail("\(a.explanationState)") }
        let fromB = try forged(type: "proposal_review", contextId: eb.context.contextId, reviewId: reviewB)
        a.handleExplanationReply(id: "explain-2", stage: .review, result: .success(.init(stdout: fromB, stderr: "")))
        XCTAssertEqual(a.explanationState, .failed("validated proposal answers another context"))
        XCTAssertNil(a.amendedProposalLatex)
        XCTAssertFalse(a.canApproveReviewedEdit)
        XCTAssertEqual(b.explanationState, .ready(eb), "B is untouched")
        XCTAssertNotEqual(c2.contextId, eb.context.contextId)
        // The proxy's real review reply for explain-2 arrives afterwards: late, dropped.
        try await waitForProcessGone(reviewPid)
        try await waitUntil("A's real review reply discarded") { a.recoveryLog.last == .lateReplyDiscarded(requestId: "explain-2", stage: .review) }
        XCTAssertEqual(a.explanationState, .failed("validated proposal answers another context"))
        XCTAssertTrue(a.canExplain)
        try await closeAndAssertWorkerStopped(b)

        // (iii) Replies after the reviewed edit was approved, and after the
        // proposal was approved for insertion and the sheet closed.
        try setHelperDelays([:])
        a.explain()
        try await waitForSettled(a)
        guard case .ready(let e3) = a.explanationState, e3.reviewId != nil else { return XCTFail("\(a.explanationState)") }
        a.approveReviewedEdit()
        try await waitForSettled(a)
        guard case .approved(let approved) = a.explanationState else { return XCTFail("\(a.explanationState)") }
        XCTAssertEqual(approved.amendedLatex, "% reviewed: %diag:0 inserted again")
        let staleBefore = a.staleExplanationReplies
        for stage in [ProposalPreview.ExplanationStage.approve, .review, .provider, .prepare, .probe] {
            a.handleExplanationReply(id: "explain-3", stage: stage, result: .success(.init(stdout: try forged(type: "approved_group", contextId: e3.context.contextId), stderr: "")))
            a.handleExplanationReply(id: "explain-3", stage: stage, result: .failure(.exited(9, output: Data(), stderr: "")))
        }
        XCTAssertEqual(a.explanationState, .approved(approved), "the approved amendment is never replaced or failed by a late reply")
        XCTAssertEqual(a.staleExplanationReplies, staleBefore + 10)
        XCTAssertEqual(a.amendedProposalLatex, approved.amendedLatex)

        XCTAssertEqual(model.approveProposal(proposal, latex: approved.amendedLatex), .inserted(byteOffset: 0))
        let pending = try XCTUnwrap(model.pendingEdit)
        XCTAssertEqual(pending.text, "% reviewed: %diag:0 inserted again\n")
        a.close()
        for stage in [ProposalPreview.ExplanationStage.approve, .review, .provider] {
            a.handleExplanationReply(id: "explain-3", stage: stage, result: .success(.init(stdout: try forged(type: "approved_group", contextId: e3.context.contextId), stderr: "")))
        }
        XCTAssertEqual(a.explanationState, .cancelled("review closed"))
        XCTAssertEqual(model.pendingEdit, pending, "the staged insertion is exactly the reviewer's approved text")
        XCTAssertTrue(model.proposals.isEmpty)
        XCTAssertTrue(model.appliedCaptureIDs.contains("cap-order"))
        try await waitUntil("A worker terminated") { !a.workerIsRunning }
    }

    // MARK: 3. reviewed-insertion recovery

    private final class Probe {
        var editApplied: [(ShellModel.PendingEdit, String)] = []
        var editRefused: [(ShellModel.PendingEdit, String)] = []
    }

    private struct Host: View {
        var model: ShellModel
        var probe: Probe
        var body: some View {
            SourceEditorView(
                text: Binding(get: { model.activeText }, set: { model.updateActiveText($0) }),
                selection: model.selection,
                pendingEdit: model.pendingEdit,
                marks: model.editorMarks,
                result: model.result,
                editorRevision: model.editorRevision, // ContentView passes it; the revision guard depends on it
                onCaretChange: { model.caretUTF16 = $0 },
                onSelectionChange: { model.caretLengthUTF16 = $0.length },
                onEditApplied: { edit, text in
                    probe.editApplied.append((edit, text))
                    model.editApplied(edit, newText: text)
                },
                onEditRefused: { edit, reason in
                    probe.editRefused.append((edit, reason))
                    model.editRefused(edit, reason: reason)
                }
            )
        }
    }

    /// The real editor in a never-key window, bound like `ContentView` binds it.
    private func host(_ model: ShellModel, probe: Probe) async throws -> (NSWindow, NSTextView) {
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 400), styleMask: [.titled],
                              backing: .buffered, defer: false)
        window.contentView = NSHostingView(rootView: Host(model: model, probe: probe))
        window.orderFrontRegardless() // never makeKey
        var found: NSTextView?
        try await waitUntil("editor text view") {
            found = TypingBenchDriver.findTextView(in: [window.contentView!]); return found != nil
        }
        return (window, try XCTUnwrap(found))
    }

    /// Approve the AI-reviewed proposal → the real editor refuses the insertion
    /// because the buffer moved on → the proposal (amended text) is back at
    /// the front of the review queue with the editor's exact reason, the
    /// anchor restored, nothing inserted, nothing durable → re-explained
    /// against the new revision with the real helper; the helper killed
    /// mid-`approve` loses nothing → the next approval inserts exactly once,
    /// one undo step, one durable revision in the real preview-controller;
    /// re-enqueueing the capture is refused as a duplicate.
    func testRefusedReviewedInsertionReturnsToReviewAndReapprovalInsertsOnceThroughTheRealController() async throws {
        let helper = try realHelper()
        guard let controller = Self.env("FLASHTEX_PREVIEW_CONTROLLER"), ShellModel.locateCompiler() != nil else {
            throw XCTSkip("set FLASHTEX_PREVIEW_CONTROLLER and FLASHTEX_COMPILER to built binaries")
        }
        let root = scratch.appendingPathComponent("controller")
        try FileManager.default.createDirectory(at: root.appendingPathComponent("project"), withIntermediateDirectories: true)
        let tex = root.appendingPathComponent("project/main.tex")
        let before = "\\begin{document}\nHello reviewed world.\n\\end{document}\n"
        try before.write(to: tex, atomically: true, encoding: .utf8)
        setenv("FLASHTEX_CONTROLLER_LEDGER_ROOT", root.appendingPathComponent("ledger").path, 1)
        defer { unsetenv("FLASHTEX_CONTROLLER_LEDGER_ROOT") }
        let load = loadAverage()

        let model = ShellModel()
        model.autoCompile = true
        XCTAssertEqual(model.openTex(at: tex), .opened)
        model.attachController(at: controller)
        defer { model.detachController() }
        XCTAssertTrue(model.controllerAttached)
        try await waitUntil("initial durable document + preview", timeout: 30) {
            model.controllerState.durable["main.tex"] != nil && model.result?.revision == model.editorRevision && model.inFlightRevision == nil
        }
        let durable0 = try XCTUnwrap(model.controllerState.durable["main.tex"]).revision
        let probe = Probe()
        let (window, tv) = try await host(model, probe: probe)
        defer { window.orderOut(nil) }
        XCTAssertEqual(tv.string, before)

        model.caretUTF16 = 17 // after "\begin{document}\n"
        model.pinAnchorAtCaret()
        let anchorBefore = try XCTUnwrap(model.anchor)
        XCTAssertEqual(anchorBefore.byteOffset, 17)
        let proposal = RuntimeV1.CaptureProposal(captureId: "cap-refused", latex: "%diag:0 inserted", ambiguities: [], requiredDependencies: [])
        model.enqueue(proposal)
        let revision0 = model.editorRevision

        // AI review with the real helper: explain → reviewed edit → explicit approve → amended proposal text.
        let preview = makePreview(configuration(helper: helper))
        preview.update(from: model, latex: proposal.latex)
        try await waitForReady(preview)
        preview.explain()
        try await waitForSettled(preview)
        guard case .ready(let reviewed) = preview.explanationState, reviewed.reviewId != nil else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(reviewed.context.compileRevision, 2)
        preview.approveReviewedEdit()
        try await waitForSettled(preview)
        guard case .approved(let amendment) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        let amended = amendment.amendedLatex
        XCTAssertEqual(amended, "% reviewed: %diag:0 inserted")
        XCTAssertEqual(amendment.expectedRevision, revision0)
        XCTAssertEqual(tv.string, before, "review changed nothing in the editor")

        // Approve the insertion; the buffer moves on (a model-side edit, e.g. a
        // companion or paste) before the editor could apply it. The editor
        // refuses the stale range explicitly.
        XCTAssertEqual(model.approveProposal(proposal, latex: amended), .inserted(byteOffset: 17))
        let staged = try XCTUnwrap(model.pendingEdit)
        XCTAssertEqual(staged.revision, revision0)
        XCTAssertEqual(staged.captureRefund?.proposal.latex, amended)
        XCTAssertTrue(model.proposals.isEmpty)
        XCTAssertTrue(model.appliedCaptureIDs.contains("cap-refused"))
        preview.close()
        let moved = before + "% typed while the insertion was staged\n"
        model.updateActiveText(moved)
        XCTAssertEqual(model.editorRevision, revision0 + 1)
        try await waitUntil("editor refused the stale insertion") { probe.editRefused.count == 1 }
        let (refusedEdit, reason) = try XCTUnwrap(probe.editRefused.first)
        XCTAssertEqual(refusedEdit, staged)
        XCTAssertEqual(reason, "the document changed since the edit was prepared (revision \(revision0), now \(revision0 + 1))")
        XCTAssertTrue(probe.editApplied.isEmpty, "never reported as applied")
        XCTAssertNil(model.pendingEdit)
        // Back to reviewable with the exact reason, the amended text and the pre-insertion anchor.
        XCTAssertEqual(model.captureNote, "Capture cap-refused was not inserted: \(reason). It is back in the review queue; approve it again.")
        XCTAssertEqual(model.proposals.map(\.captureId), ["cap-refused"])
        let refunded = try XCTUnwrap(model.reviewing)
        XCTAssertEqual(refunded.latex, amended, "the reviewer's amended text survives the refusal")
        XCTAssertFalse(model.appliedCaptureIDs.contains("cap-refused"))
        XCTAssertEqual(model.anchor, anchorBefore)
        try await waitUntil("editor synced to the moved text") { tv.string == moved }
        XCTAssertFalse(tv.string.contains("% reviewed"), "nothing was inserted")
        XCTAssertFalse(tv.undoManager?.canUndo ?? true, "no undo step for a refused insertion")
        try await waitUntil("typed edit durable", timeout: 30) {
            model.controllerState.durable["main.tex"]?.revision == durable0 + 1 && model.inFlightRevision == nil
        }
        XCTAssertEqual(model.controllerState.textByDurable["main.tex"]?[durable0 + 1], moved)

        // Re-review against the new revision: the real helper binds the fresh
        // shadow; the helper killed mid-approve (by the pid we launched) fails
        // with the exact reason and loses nothing; the retry approves.
        let again = makePreview(configuration(helper: helper))
        again.update(from: model, latex: refunded.latex)
        try await waitForReady(again)
        XCTAssertEqual(again.shadow?.shadowText, "\\begin{document}\n\(amended)\nHello reviewed world.\n\\end{document}\n% typed while the insertion was staged\n")
        again.explain()
        try await waitForSettled(again)
        guard case .ready(let reReviewed) = again.explanationState, reReviewed.reviewId != nil else { return XCTFail("\(again.explanationState)") }
        XCTAssertEqual(reReviewed.edits.first?.removedText, amended)
        try setHelperDelays(["approve": 0.8])
        again.approveReviewedEdit()
        let approvePid = try await waitForStage(again, .approve)
        XCTAssertEqual(kill(approvePid, SIGKILL), 0)
        try await waitForSettled(again)
        XCTAssertEqual(again.explanationState, .failed("helper exited (9) during approve"))
        XCTAssertEqual(again.recoveryLog.last, .exited(requestId: "explain-1", party: .helper, stage: .approve, code: 9))
        XCTAssertEqual(again.recoveryNoteText, "helper exited (9) during approve; it is relaunched on the next request. Nothing was applied.")
        XCTAssertNil(again.amendedProposalLatex)
        XCTAssertEqual(model.proposals.map(\.captureId), ["cap-refused"], "the proposal is still reviewable")
        XCTAssertNil(model.pendingEdit)
        XCTAssertEqual(tv.string, moved)
        try setHelperDelays([:])
        again.explain()
        try await waitForSettled(again)
        guard case .ready(let third) = again.explanationState, third.reviewId != nil else { return XCTFail("\(again.explanationState)") }
        XCTAssertEqual(third.context.requestId, "explain-2")
        again.approveReviewedEdit()
        try await waitForSettled(again)
        guard case .approved(let reAmendment) = again.explanationState else { return XCTFail("\(again.explanationState)") }
        XCTAssertEqual(reAmendment.expectedRevision, revision0 + 1, "bound to the moved buffer")
        XCTAssertEqual(reAmendment.expectedSha256, SourceDigest.sha256Hex(try XCTUnwrap(again.shadow?.shadowText)))
        XCTAssertEqual(reAmendment.amendedLatex, "% reviewed: " + amended)

        // Second approval: the editor applies it once (one undo step), the
        // model hears it once, the controller commits one durable revision.
        XCTAssertEqual(model.approveProposal(refunded, latex: reAmendment.amendedLatex), .inserted(byteOffset: 17))
        again.close()
        try await waitUntil("editor applied the insertion") { probe.editApplied.count == 1 }
        XCTAssertEqual(probe.editRefused.count, 1, "no second refusal")
        let inserted = Insertion.insertionText(reAmendment.amendedLatex, into: moved, atByte: 17)
        let after = "\\begin{document}\n" + inserted + "Hello reviewed world.\n\\end{document}\n% typed while the insertion was staged\n"
        XCTAssertEqual(tv.string, after)
        XCTAssertEqual(model.activeText, after)
        XCTAssertNil(model.pendingEdit)
        XCTAssertTrue(tv.undoManager?.canUndo ?? false, "the insertion is undoable")
        XCTAssertTrue(model.proposals.isEmpty)
        XCTAssertTrue(model.appliedCaptureIDs.contains("cap-refused"))
        try await waitUntil("insertion durable + previewed", timeout: 30) {
            model.controllerState.durable["main.tex"]?.revision == durable0 + 2 && model.inFlightRevision == nil && model.result?.revision == model.editorRevision
        }
        let durable = try XCTUnwrap(model.controllerState.durable["main.tex"])
        XCTAssertEqual(durable.revision, durable0 + 2, "exactly one durable edit for the insertion (after the typed one)")
        let durableText = try XCTUnwrap(model.controllerState.textByDurable["main.tex"]?[durable.revision])
        XCTAssertEqual(durableText, after)
        XCTAssertEqual(durableText.components(separatedBy: "% reviewed: % reviewed: %diag:0 inserted").count - 1, 1, "the reviewed LaTeX appears exactly once")
        XCTAssertEqual(durable.sha256, SourceDigest.sha256Hex(after))
        XCTAssertEqual(model.compiledDocuments["main.tex"], after, "the live preview compiled the inserted text")
        XCTAssertEqual(model.anchor?.byteOffset, 17 + inserted.utf8.count, "anchor advanced past the insertion")
        // The same capture can never be inserted twice.
        model.enqueue(proposal)
        XCTAssertTrue(model.proposals.isEmpty)
        XCTAssertEqual(model.approveProposal(proposal, latex: amended), .duplicate)
        XCTAssertNil(model.pendingEdit)
        try await waitUntil("previews closed") { !preview.workerIsRunning && !again.workerIsRunning }
        print("measured: refused reviewed insertion → re-review → durable insert via flashtex-preview-controller; 1-min load \(load)")
    }
}
