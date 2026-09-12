import XCTest
@testable import FlashTeXProtocol
@testable import FlashTeXMac

/// Assistant cancellation / expiry / timeout / helper-exit recovery in the
/// review sheet (ProposalPreview.swift + AssistantRequestState.swift) with the
/// REAL `flashtex-assistant-context` helper (`locateHelper()`: env, bundle,
/// pinned scratch build, or the crate build) and the deterministic local
/// provider `Fixtures/controlled_assistant_provider.py` driven by a control
/// file. No network, no credentials, no model. Every case ends by asserting the
/// proposal approval boundary: nothing reaches the document without the
/// sheet's explicit approve, and `ShellModel.editApplied` is never called here.
@MainActor
final class AssistantRecoveryTests: XCTestCase {
    static let controlledProvider = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent().appendingPathComponent("Fixtures/controlled_assistant_provider.py")

    private var scratch: URL!
    private var control: URL!
    private var pidFile: URL!
    private var logFile: URL!

    override func setUpWithError() throws {
        scratch = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent("flashtex-assistant-recovery-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: scratch, withIntermediateDirectories: true)
        control = scratch.appendingPathComponent("control.json")
        pidFile = scratch.appendingPathComponent("provider.pid")
        logFile = scratch.appendingPathComponent("provider.log")
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

    /// Timing-sensitive cases skip on a loaded host (the parent's rule: 1-min load > 20).
    private func requireQuietHost() throws {
        var loads = [Double](repeating: 0, count: 3)
        let n = getloadavg(&loads, 3)
        print("AssistantRecoveryTests load averages: \(n > 0 ? loads.map { String(format: "%.2f", $0) }.joined(separator: " ") : "unavailable")")
        if n > 0, loads[0] > 20 { throw XCTSkip("1-min load \(loads[0]) > 20; timing case skipped") }
    }

    private func setControl(delay: TimeInterval = 0, ignoreSigterm: Bool = false, mode: String = "answer", exitCode: Int = 4) throws {
        try? FileManager.default.removeItem(at: pidFile)
        let obj: [String: Any] = ["delay_s": delay, "ignore_sigterm": ignoreSigterm, "mode": mode, "exit_code": exitCode,
                                  "pid_file": pidFile.path, "log_file": logFile.path]
        try JSONSerialization.data(withJSONObject: obj).write(to: control)
    }

    private func providerLog() -> [String] {
        ((try? String(contentsOf: logFile, encoding: .utf8)) ?? "").split(separator: "\n").map(String.init)
    }

    private func configuration(helper: URL, providerTimeout: TimeInterval = 5) -> ProposalPreview.ExplanationConfiguration {
        var c = ProposalPreview.ExplanationConfiguration(helper: helper)
        c.provider = WorkerClientTests.python
        c.providerArguments = [Self.controlledProvider.path, control.path]
        c.providerTimeout = providerTimeout
        return c
    }

    private func makePreview(_ explanation: ProposalPreview.ExplanationConfiguration) -> ProposalPreview {
        ProposalPreview(executable: WorkerClientTests.python, arguments: [WorkerClientTests.fakeWorker.path], explanation: explanation)
    }

    private func input(_ text: String, anchorByte: Int, revision: Int = 1) -> ProposalPreview.Input {
        let ctx = String(decoding: Array(text.utf8.dropFirst(anchorByte).prefix(Insertion.contextLength)), as: UTF8.self)
        return .init(documents: [.init(path: "main.tex", text: text)], entryPath: "main.tex",
                     anchor: InsertionAnchor(id: "a1", path: "main.tex", byteOffset: anchorByte, revision: revision, contextAfter: ctx),
                     editorRevision: revision, projectId: "recovery")
    }

    private func waitUntil(_ what: String, timeout: TimeInterval = 8, _ cond: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if cond() { return }
            try await Task.sleep(nanoseconds: 20_000_000)
        }
        XCTFail("timed out waiting for \(what)")
    }

    private func waitForReady(_ preview: ProposalPreview) async throws {
        try await waitUntil("preview ready") { if case .ready = preview.state { return true }; return false }
    }

    private func waitForProvider(_ preview: ProposalPreview) async throws -> Int32 {
        try await waitUntil("provider running") { if case .awaitingProvider = preview.explanationState { return true }; return false }
        try await waitUntil("provider pid file") { [pidFile] in
            (try? String(contentsOf: pidFile!, encoding: .utf8)).flatMap { Int32($0.trimmingCharacters(in: .whitespacesAndNewlines)) } != nil
        }
        return try XCTUnwrap(Int32(String(contentsOf: pidFile, encoding: .utf8).trimmingCharacters(in: .whitespacesAndNewlines)))
    }

    private func waitForSettled(_ preview: ProposalPreview, timeout: TimeInterval = 8) async throws {
        try await waitUntil("explanation settled", timeout: timeout) { !preview.explanationInFlight }
    }

    private func processIsGone(_ pid: Int32) -> Bool { kill(pid, 0) == -1 && errno == ESRCH }

    private func waitForProcessGone(_ pid: Int32) async throws {
        try await waitUntil("provider \(pid) gone") { [self] in processIsGone(pid) }
    }

    private func closeAndAssertWorkerStopped(_ preview: ProposalPreview) async throws {
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    // MARK: (a) reviewer cancel → late provider result discarded, sheet idle, no edit

    func testReviewerCancelDiscardsLateProviderResultAndLeavesSheetIdle() async throws {
        let helper = try realHelper()
        try setControl(delay: 0.8)
        let model = ShellModel()
        XCTAssertNil(model.loadError, model.loadError ?? "")
        let docs = model.documents, revision = model.editorRevision, fixture = model.result
        model.caretUTF16 = 0
        model.pinAnchorAtCaret()
        let proposal = RuntimeV1.CaptureProposal(captureId: "cap-cancel", latex: "%diag:0 inserted", ambiguities: [], requiredDependencies: [])
        model.enqueue(proposal)

        let preview = makePreview(configuration(helper: helper))
        preview.update(from: model, latex: proposal.latex)
        try await waitForReady(preview)
        preview.explain()
        let pid = try await waitForProvider(preview)
        XCTAssertEqual(preview.runningChildProcessIdentifier, pid, "the running child is the provider we see in the pid file")

        // The sheet's Cancel button.
        preview.cancelExplanationByReviewer()
        XCTAssertEqual(preview.explanationState, .cancelled("cancelled by reviewer"))
        XCTAssertTrue(preview.explanationState.isIdleForReviewer)
        XCTAssertFalse(preview.explanationInFlight)
        XCTAssertNil(preview.runningChildProcessIdentifier)
        XCTAssertEqual(preview.recoveryLog.last, .cancelledByReviewer(requestId: "explain-1", stage: .provider))
        XCTAssertEqual(preview.recoveryNoteText, "cancelled during provider; any late reply is discarded. Explain starts a new request.")

        // The provider is terminated by the pid we launched; its completion is a
        // late reply for explain-1 and changes nothing.
        try await waitForProcessGone(pid)
        try await waitUntil("late reply discarded") { preview.staleExplanationReplies == 1 }
        XCTAssertEqual(preview.recoveryLog.events, [
            .cancelledByReviewer(requestId: "explain-1", stage: .provider),
            .lateReplyDiscarded(requestId: "explain-1", stage: .provider),
        ])
        XCTAssertEqual(preview.explanationState, .cancelled("cancelled by reviewer"))
        XCTAssertEqual(preview.childLaunches.map(\.stage), [.probe, .prepare, .provider], "no validate/review followed the cancel")
        XCTAssertEqual(providerLog().count, 1)
        XCTAssertNil(preview.amendedProposalLatex)
        XCTAssertFalse(preview.canApproveReviewedEdit)
        XCTAssertTrue(preview.canExplain, "the sheet is usable again")
        guard case .ready = preview.state else { return XCTFail("\(preview.state)") }

        // Approval boundary: nothing reached the model; no proposal was applied.
        XCTAssertEqual(model.documents, docs)
        XCTAssertEqual(model.result, fixture)
        XCTAssertEqual(model.editorRevision, revision)
        XCTAssertNil(model.pendingEdit)
        XCTAssertEqual(model.proposals.map(\.captureId), ["cap-cancel"])

        // Explain again is a NEW request with a fresh provider run.
        try setControl(delay: 0)
        preview.explain()
        try await waitForSettled(preview)
        guard case .ready(let e) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(e.context.requestId, "explain-2")
        XCTAssertEqual(providerLog().count, 2)
        XCTAssertNil(preview.recoveryNoteText, "no note while model output is displayed")
        XCTAssertNil(model.pendingEdit)
        try await closeAndAssertWorkerStopped(preview)
    }

    // MARK: (b) document changed under the request → expired, refused by the helper too

    func testDocumentChangeExpiresTheRequestAndTheHelperRefusesTheMovedSnapshot() async throws {
        let helper = try realHelper()
        try setControl(delay: 0)
        let preview = makePreview(configuration(helper: helper))
        let doc = input("A\n%diag:1 tail\n", anchorByte: 2)
        preview.update(input: doc, latex: "%diag:0 new")
        try await waitForReady(preview)
        preview.explain()
        try await waitForSettled(preview)
        guard case .ready(let reviewed) = preview.explanationState, reviewed.reviewId != nil else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(preview.canApproveReviewedEdit)
        // The exact `review` request the helper accepted for this snapshot.
        let accepted = try XCTUnwrap(preview.currentHelperRequest)
        XCTAssertEqual(accepted["operation"] as? String, "review")

        // The document moves (editor revision 1 → 2) while the reviewed edit is
        // displayed: the sheet discards it before anyone can approve it.
        var bumped = doc
        bumped.editorRevision = 2
        preview.update(input: bumped, latex: "%diag:0 new")
        XCTAssertEqual(preview.explanationState, .cancelled("proposal or document changed"))
        XCTAssertEqual(preview.recoveryLog.last, .expired(requestId: "explain-1", fromRevision: 1, toRevision: 2))
        XCTAssertTrue(try XCTUnwrap(preview.recoveryNoteText).hasPrefix("explanation expired: the document changed (revision 1 → 2)"))
        XCTAssertFalse(preview.canApproveReviewedEdit)
        XCTAssertNil(preview.amendedProposalLatex)
        XCTAssertNil(preview.currentHelperRequest)

        // Defense in depth: had the moved snapshot reached the helper, its own
        // binding check refuses it. Replay the accepted request with the
        // sources at revision 2 (same text) and then with changed text.
        var movedRevision = accepted
        movedRevision["current_sources"] = (accepted["current_sources"] as? [[String: Any]] ?? []).map { var d = $0; d["revision"] = 2; return d }
        let refusedRevision = try await runHelper(helper, request: movedRevision)
        XCTAssertEqual(refusedRevision.message, "source snapshot is stale")
        XCTAssertEqual(AssistantRecoveryEvent.classify(refusedRevision.failure, requestId: "explain-1", stage: .review),
                       .refused(requestId: "explain-1", stage: .review, message: "source snapshot is stale"))
        XCTAssertThrowsError(try ProposalPreview.explanation(from: refusedRevision.output, context: reviewed.context,
                                                             shadow: try XCTUnwrap(preview.shadow), expecting: "proposal_review")) {
            XCTAssertEqual(($0 as? ProposalPreview.ExplanationError)?.message, "helper refused the provider's reply: source snapshot is stale")
        }
        var movedText = accepted
        movedText["current_sources"] = (accepted["current_sources"] as? [[String: Any]] ?? []).map { d in
            var d = d
            let text = (d["text"] as? String ?? "") + "% edited\n"
            d["text"] = text
            d["source_sha256"] = SourceDigest.sha256Hex(text)
            return d
        }
        let refusedText = try await runHelper(helper, request: movedText)
        XCTAssertEqual(refusedText.message, "source snapshot is stale")

        // A provider reply that lands AFTER the document changed is dropped
        // before the helper ever sees it.
        try await waitUntil("recompiled") { preview.shadowCompileCount == 2 && !preview.isInFlight }
        try setControl(delay: 0.8)
        preview.explain()
        let pid = try await waitForProvider(preview)
        var bumpedAgain = bumped
        bumpedAgain.editorRevision = 3
        preview.update(input: bumpedAgain, latex: "%diag:0 new")
        XCTAssertEqual(preview.recoveryLog.last, .expired(requestId: "explain-2", fromRevision: 2, toRevision: 3))
        try await waitForProcessGone(pid)
        try await waitUntil("late reply discarded") { preview.recoveryLog.lateReplies == 1 }
        XCTAssertEqual(preview.recoveryLog.last, .lateReplyDiscarded(requestId: "explain-2", stage: .provider))
        XCTAssertEqual(preview.explanationState, .cancelled("proposal or document changed"))
        XCTAssertEqual(preview.childLaunches.filter { $0.stage == .review }.count, 1, "only the first (current) reply was reviewed")

        // Explain for the current text works and is bound to the new revision.
        try await waitUntil("recompiled") { preview.shadowCompileCount == 3 && !preview.isInFlight }
        try setControl(delay: 0)
        preview.explain()
        try await waitForSettled(preview)
        guard case .ready(let fresh) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(fresh.context.requestId, "explain-3")
        XCTAssertNotEqual(fresh.context.contextId, reviewed.context.contextId)
        XCTAssertEqual((preview.currentHelperRequest?["current_sources"] as? [[String: Any]])?.first?["revision"] as? Int, 3)
        try await closeAndAssertWorkerStopped(preview)
    }

    private struct HelperRefusal { var failure: OneShotProcess.Failure; var output: Data; var message: String? }

    /// Runs the real helper once on `request` (as `runHelper` in the app does)
    /// and returns its nonzero-exit refusal.
    private func runHelper(_ helper: URL, request: [String: Any]) async throws -> HelperRefusal {
        let data = try JSONSerialization.data(withJSONObject: request)
        var got: Result<OneShotProcess.Output, OneShotProcess.Failure>?
        let process = try OneShotProcess(executable: helper, arguments: [], input: data, timeout: 10, maxOutputBytes: 128 * 1024,
                                         environment: ProposalPreview.ExplanationConfiguration.childEnvironment(for: .helper)) { got = $0 }
        try await waitUntil("helper reply") { got != nil }
        XCTAssertFalse(process.isRunning)
        guard case .failure(let f) = try XCTUnwrap(got), case .exited(_, let output, _) = f else {
            XCTFail("expected a nonzero helper exit, got \(String(describing: got))"); return HelperRefusal(failure: .cancelled, output: Data(), message: nil)
        }
        let obj = try JSONSerialization.jsonObject(with: output) as? [String: Any]
        XCTAssertEqual(obj?["type"] as? String, "error")
        return HelperRefusal(failure: f, output: output, message: obj?["message"] as? String)
    }

    // MARK: (c) provider timeout from the environment → clean failure, retry = new id, late result ignored

    func testProviderTimeoutFromEnvironmentThenRetryStartsNewRequestAndIgnoresLateResult() async throws {
        let helper = try realHelper()
        try requireQuietHost()
        var env = ["FLASHTEX_ASSISTANT_CONTEXT": helper.path, "FLASHTEX_ASSISTANT_PROVIDER": Self.controlledProvider.path,
                   "FLASHTEX_ASSISTANT_TIMEOUT_S": "1"]
        env["PATH"] = ProcessInfo.processInfo.environment["PATH"]
        var config = ProposalPreview.ExplanationConfiguration.fromEnvironment(env)
        XCTAssertEqual(config.provider, Self.controlledProvider)
        XCTAssertEqual(config.providerTimeout, 1, "FLASHTEX_ASSISTANT_TIMEOUT_S")
        config.providerArguments = [control.path] // the control file; the command is still the env one
        let preview = makePreview(config)
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(preview)

        // A provider that never answers (30 s) is stopped at the 1 s deadline.
        try setControl(delay: 30)
        let started = Date()
        preview.explain()
        let pid = try await waitForProvider(preview)
        try await waitForSettled(preview)
        let elapsed = Date().timeIntervalSince(started)
        XCTAssertEqual(preview.explanationState, .failed("provider gave no reply within 1 s (provider)"))
        XCTAssertTrue(preview.explanationState.isIdleForReviewer)
        XCTAssertEqual(preview.recoveryLog.last, .timedOut(requestId: "explain-1", party: .provider, stage: .provider, seconds: 1))
        XCTAssertEqual(preview.recoveryNoteText, "provider gave no reply within 1 s (provider) and was stopped; a late reply is ignored. Explain again starts a new request.")
        XCTAssertLessThan(elapsed, 6, "timeout surfaced promptly (measured \(elapsed) s)")
        try await waitForProcessGone(pid)
        XCTAssertTrue(preview.canExplain)
        XCTAssertNil(preview.runningChildProcessIdentifier)

        // Retry: a NEW request id and a new provider run.
        try setControl(delay: 0)
        preview.explain()
        try await waitUntil("retry running") { preview.explanationInFlight }
        // The old request's result arriving now (after the retry started) is ignored…
        let forgedLate = try JSONSerialization.data(withJSONObject: ["context_id": String(repeating: "1", count: 64), "explanation": "late", "edits": []])
        preview.handleExplanationReply(id: "explain-1", stage: .provider, result: .success(.init(stdout: forgedLate, stderr: "")))
        XCTAssertTrue(preview.explanationInFlight, "the retry is unaffected")
        try await waitForSettled(preview)
        guard case .ready(let e) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(e.context.requestId, "explain-2")
        // …and still after the retry finished.
        preview.handleExplanationReply(id: "explain-1", stage: .provider, result: .success(.init(stdout: forgedLate, stderr: "")))
        preview.handleExplanationReply(id: "explain-1", stage: .provider, result: .failure(.timeout(1)))
        XCTAssertEqual(preview.explanationState, .ready(e))
        XCTAssertEqual(preview.recoveryLog.events.filter { $0 == .lateReplyDiscarded(requestId: "explain-1", stage: .provider) }.count, 3)
        let pids = providerLog().map { $0.split(separator: " ").first.map(String.init) ?? "" }
        XCTAssertEqual(pids.count, 2)
        XCTAssertNotEqual(pids[0], pids[1], "a fresh provider process per request")
        XCTAssertEqual(pids[0], String(pid))

        // A stubborn provider that ignores SIGTERM and answers a VALID reply
        // 0.5 s after the deadline: the reply is complete but late, so it is
        // never validated — the request stays a timeout, no output is shown.
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 later")
        try await waitUntil("recompiled") { preview.shadowCompileCount == 2 && !preview.isInFlight }
        try setControl(delay: 1.5, ignoreSigterm: true)
        preview.explain()
        let stubborn = try await waitForProvider(preview)
        try await waitForSettled(preview)
        XCTAssertEqual(preview.explanationState, .failed("provider gave no reply within 1 s (provider)"))
        XCTAssertEqual(preview.recoveryLog.last, .timedOut(requestId: "explain-3", party: .provider, stage: .provider, seconds: 1))
        XCTAssertEqual(preview.childLaunches.last?.stage, .provider, "its late valid answer never reached validate/review")
        try await waitForProcessGone(stubborn)
        XCTAssertEqual(providerLog().count, 3)
        XCTAssertNil(preview.amendedProposalLatex)
        try await closeAndAssertWorkerStopped(preview)
    }

    // MARK: (d) helper exits mid-request → surfaced, relaunched on the next request

    func testHelperExitMidRequestIsSurfacedAndRelaunchedOnNextRequest() async throws {
        let helper = try realHelper()
        try setControl(delay: 0)
        let preview = makePreview(configuration(helper: helper))
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(preview)

        // Kill the real helper (by the pid we launched) while it handles the
        // first request. explain() launches the probe synchronously, so the
        // process is running and no reply can have been delivered yet.
        preview.explain()
        XCTAssertEqual(preview.explanationState, .preparing)
        let helperPid = try XCTUnwrap(preview.runningChildProcessIdentifier)
        XCTAssertEqual(kill(helperPid, SIGKILL), 0, "the helper we launched is running")
        try await waitForSettled(preview)
        guard case .failed(let why) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(why.hasPrefix("helper exited (9) during "), why)
        guard case .exited("explain-1", .helper, let stage, 9)? = preview.recoveryLog.last else { return XCTFail("\(String(describing: preview.recoveryLog.last))") }
        XCTAssertTrue(stage == .probe || stage == .prepare, "\(stage)")
        XCTAssertEqual(preview.recoveryNoteText, "helper exited (9) during \(stage); it is relaunched on the next request. Nothing was applied.")
        try await waitForProcessGone(helperPid)
        XCTAssertNil(preview.runningChildProcessIdentifier)
        XCTAssertTrue(preview.canExplain)
        XCTAssertTrue(providerLog().isEmpty, "the provider never ran for the killed request")
        let launchesBefore = preview.childLaunches.count

        // The next request relaunches the helper and completes normally.
        preview.explain()
        XCTAssertNotEqual(preview.runningChildProcessIdentifier, helperPid)
        try await waitForSettled(preview)
        guard case .ready(let e) = preview.explanationState, e.reviewId != nil else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(e.context.requestId, "explain-2")
        XCTAssertEqual(preview.childLaunches.count - launchesBefore, 4, "probe, prepare, provider, review")
        XCTAssertEqual(providerLog().count, 1)
        XCTAssertNil(preview.recoveryNoteText)

        // The real helper's own refusal (nonzero exit + error reply) mid-request:
        // a provider answering for another context is refused during review.
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 other")
        try await waitUntil("recompiled") { preview.shadowCompileCount == 2 && !preview.isInFlight }
        try setControl(mode: "stale_context")
        preview.explain()
        try await waitForSettled(preview)
        XCTAssertEqual(preview.explanationState, .failed("helper refused (review): invalid explanation identity or bounds"))
        XCTAssertEqual(preview.recoveryLog.last, .refused(requestId: "explain-3", stage: .review, message: "invalid explanation identity or bounds"))
        XCTAssertNil(preview.amendedProposalLatex)
        XCTAssertTrue(preview.canExplain)

        // A provider that exits (4) mid-request is surfaced as such; the next request runs it again.
        try setControl(mode: "exit", exitCode: 4)
        preview.explain()
        try await waitForSettled(preview)
        XCTAssertEqual(preview.explanationState, .failed("provider exited (4) during provider"))
        XCTAssertEqual(preview.recoveryLog.last, .exited(requestId: "explain-4", party: .provider, stage: .provider, code: 4))
        try setControl(delay: 0)
        preview.explain()
        try await waitForSettled(preview)
        guard case .ready(let again) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(again.context.requestId, "explain-5")
        XCTAssertEqual(providerLog().count, 4)
        try await closeAndAssertWorkerStopped(preview)
    }

    // MARK: (e) the approval boundary survives every recovery

    func testNoEditReachesTheDocumentWithoutExplicitApproveAcrossRecoveries() async throws {
        let helper = try realHelper()
        let model = ShellModel()
        XCTAssertNil(model.loadError, model.loadError ?? "")
        let docs = model.documents, revision = model.editorRevision, fixture = model.result
        model.caretUTF16 = 0
        model.pinAnchorAtCaret()
        let proposal = RuntimeV1.CaptureProposal(captureId: "cap-boundary", latex: "%diag:0 inserted", ambiguities: [], requiredDependencies: [])
        model.enqueue(proposal)
        let preview = makePreview(configuration(helper: helper, providerTimeout: 1))
        func assertUntouched(_ step: String) {
            XCTAssertEqual(model.documents, docs, step)
            XCTAssertEqual(model.result, fixture, step)
            XCTAssertEqual(model.editorRevision, revision, step)
            XCTAssertNil(model.pendingEdit, step)
            XCTAssertEqual(model.proposals.map(\.captureId), ["cap-boundary"], step)
        }

        // Reviewer cancel mid-provider.
        try setControl(delay: 0.8)
        preview.update(from: model, latex: proposal.latex)
        try await waitForReady(preview)
        preview.explain()
        let p1 = try await waitForProvider(preview)
        preview.cancelExplanationByReviewer()
        try await waitForProcessGone(p1)
        assertUntouched("after reviewer cancel")

        // Provider timeout.
        try setControl(delay: 30)
        preview.explain()
        let p2 = try await waitForProvider(preview)
        try await waitForSettled(preview)
        XCTAssertEqual(preview.explanationState, .failed("provider gave no reply within 1 s (provider)"))
        try await waitForProcessGone(p2)
        assertUntouched("after provider timeout")

        // Helper killed mid-request.
        preview.explain()
        let h = try XCTUnwrap(preview.runningChildProcessIdentifier)
        XCTAssertEqual(kill(h, SIGKILL), 0)
        try await waitForSettled(preview)
        guard case .failed(let why) = preview.explanationState, why.hasPrefix("helper exited (9)") else { return XCTFail("\(preview.explanationState)") }
        assertUntouched("after helper kill")

        // A reviewed edit, then the document moves before anyone approves it.
        try setControl(delay: 0)
        preview.explain()
        try await waitForSettled(preview)
        guard case .ready(let reviewed) = preview.explanationState, reviewed.reviewId != nil else { return XCTFail("\(preview.explanationState)") }
        XCTAssertFalse(reviewed.applied)
        var moved = ProposalPreview.Input(model: model)
        moved.editorRevision += 1
        preview.update(input: moved, latex: proposal.latex)
        XCTAssertEqual(preview.recoveryLog.last, .expired(requestId: "explain-4", fromRevision: revision, toRevision: revision + 1))
        XCTAssertFalse(preview.canApproveReviewedEdit)
        assertUntouched("after expiry of a reviewed edit")
        try await waitUntil("recompiled") { preview.shadowCompileCount == 2 && !preview.isInFlight }

        // Explicit approve of a reviewed edit amends the PROPOSAL text only.
        preview.update(from: model, latex: proposal.latex)
        try await waitUntil("recompiled") { preview.shadowCompileCount == 3 && !preview.isInFlight }
        preview.explain()
        try await waitForSettled(preview)
        guard case .ready(let e) = preview.explanationState, e.reviewId != nil else { return XCTFail("\(preview.explanationState)") }
        preview.approveReviewedEdit()
        try await waitForSettled(preview)
        guard case .approved(let a) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(a.amendedLatex, "% reviewed: %diag:0 inserted")
        XCTAssertFalse(a.applied)
        assertUntouched("after approving the reviewed edit into the proposal text")
        XCTAssertEqual(preview.recoveryLog.events.map(\.requestId), ["explain-1", "explain-1", "explain-2", "explain-3", "explain-4"],
                       "cancel, its late reply, timeout, helper exit, expiry — nothing for the approved request")

        // Only the sheet's own approve produces a pending edit, and even that
        // does not touch the documents until the editor applies it
        // (`editApplied`, never called here).
        XCTAssertEqual(model.approveProposal(proposal, latex: a.amendedLatex), .inserted(byteOffset: 0))
        XCTAssertEqual(model.pendingEdit?.text, "% reviewed: %diag:0 inserted\n")
        XCTAssertEqual(model.documents, docs)
        XCTAssertEqual(model.editorRevision, revision)
        XCTAssertTrue(model.proposals.isEmpty)
        try await closeAndAssertWorkerStopped(preview)
    }

    // MARK: pure pieces (no processes)

    func testRecoveryEventsClassifyFailuresAndChanges() throws {
        typealias E = AssistantRecoveryEvent
        XCTAssertEqual(E.classify(.timeout(1.2), requestId: "r", stage: .provider),
                       .timedOut(requestId: "r", party: .provider, stage: .provider, seconds: 1.2))
        XCTAssertEqual(E.classify(.timeout(10), requestId: "r", stage: .prepare),
                       .timedOut(requestId: "r", party: .helper, stage: .prepare, seconds: 10))
        XCTAssertEqual(E.classify(.exited(4, output: Data(), stderr: ""), requestId: "r", stage: .provider),
                       .exited(requestId: "r", party: .provider, stage: .provider, code: 4))
        XCTAssertEqual(E.classify(.exited(9, output: Data(), stderr: ""), requestId: "r", stage: .probe),
                       .exited(requestId: "r", party: .helper, stage: .probe, code: 9))
        let refusal = try JSONSerialization.data(withJSONObject: ["type": "error", "message": "source snapshot is stale"])
        XCTAssertEqual(E.classify(.exited(1, output: refusal, stderr: ""), requestId: "r", stage: .review),
                       .refused(requestId: "r", stage: .review, message: "source snapshot is stale"))
        XCTAssertEqual(E.classify(.exited(1, output: refusal, stderr: ""), requestId: "r", stage: .provider),
                       .exited(requestId: "r", party: .provider, stage: .provider, code: 1), "only the helper's error replies are refusals")
        XCTAssertNil(E.classify(.cancelled, requestId: "r", stage: .provider))
        XCTAssertNil(E.classify(.launch("x"), requestId: "r", stage: .probe))
        XCTAssertNil(E.classify(.outputTooLarge(1), requestId: "r", stage: .provider))

        let base = input("A\n", anchorByte: 2)
        var rev = base; rev.editorRevision = 5
        XCTAssertEqual(E.forChange(requestId: "r", old: (base, "x"), new: (rev, "x")), .expired(requestId: "r", fromRevision: 1, toRevision: 5))
        var text = base; text.documents[0].text = "B\n"
        XCTAssertEqual(E.forChange(requestId: "r", old: (base, "x"), new: (text, "x")), .expired(requestId: "r", fromRevision: 1, toRevision: 1))
        XCTAssertEqual(E.forChange(requestId: "r", old: (base, "x"), new: (base, "y")), .superseded(requestId: "r", what: "proposal text"))
        var anchor = base; anchor.anchor = InsertionAnchor(id: "a2", path: "main.tex", byteOffset: 0, revision: 1, contextAfter: "A\n")
        XCTAssertEqual(E.forChange(requestId: "r", old: (base, "x"), new: (anchor, "x")), .superseded(requestId: "r", what: "insertion anchor"))
        XCTAssertEqual(E.forChange(requestId: "r", old: (base, "x"), new: (rev, "y")), .expired(requestId: "r", fromRevision: 1, toRevision: 5),
                       "a document move outranks a text change")

        for e in [E.cancelledByReviewer(requestId: "r", stage: .provider), .expired(requestId: "r", fromRevision: 1, toRevision: 2),
                  .superseded(requestId: "r", what: "proposal text"), .timedOut(requestId: "r", party: .provider, stage: .provider, seconds: 1),
                  .exited(requestId: "r", party: .helper, stage: .probe, code: 9), .refused(requestId: "r", stage: .review, message: "m"),
                  .lateReplyDiscarded(requestId: "r", stage: .validate)] {
            XCTAssertEqual(e.requestId, "r")
            XCTAssertFalse(e.note.isEmpty)
            // A note never claims an application: every "applied" is negated.
            XCTAssertEqual(e.note.lowercased().components(separatedBy: "applied").count - 1,
                           e.note.lowercased().components(separatedBy: "nothing was applied").count - 1, e.note)
        }

        var log = AssistantRecoveryLog()
        for i in 0..<(AssistantRecoveryLog.capacity + 5) { log.record(.lateReplyDiscarded(requestId: "r\(i)", stage: .provider)) }
        XCTAssertEqual(log.events.count, AssistantRecoveryLog.capacity)
        XCTAssertEqual(log.events.first?.requestId, "r5")
        XCTAssertEqual(log.lateReplies, AssistantRecoveryLog.capacity)
        XCTAssertEqual(log.events(for: "r7").count, 1)

        let ctx = ProposalPreview.ExplanationContext(requestId: "r", shadowRequestId: "p", projectId: "x", compileRevision: 1, contextId: "c",
                                                     compilerStatus: "ok", diagnosticCount: 0, omittedDiagnostics: 0, allowedEdits: [],
                                                     providerIntent: "grok", payload: Data())
        XCTAssertTrue(ProposalPreview.ExplanationState.idle.isIdleForReviewer)
        XCTAssertTrue(ProposalPreview.ExplanationState.cancelled("x").isIdleForReviewer)
        XCTAssertTrue(ProposalPreview.ExplanationState.failed("x").isIdleForReviewer)
        XCTAssertFalse(ProposalPreview.ExplanationState.preparing.isIdleForReviewer)
        XCTAssertFalse(ProposalPreview.ExplanationState.awaitingProvider(ctx).isIdleForReviewer)
        XCTAssertFalse(ProposalPreview.ExplanationState.prepared(ctx).isIdleForReviewer)
    }
}
