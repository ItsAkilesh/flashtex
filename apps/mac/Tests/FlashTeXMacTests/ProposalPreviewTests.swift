import SwiftUI
import XCTest
@testable import FlashTeXProtocol
@testable import FlashTeXMac

/// Shadow-compile preview of a capture proposal (ProposalPreview.swift).
/// Worker-backed cases use `Fixtures/fake_worker.py` (`%diag:<n>` emits an
/// error diagnostic `n` bytes after the directive). Assistant-explanation cases
/// use `Fixtures/fake_assistant_context.py` / `fake_assistant_provider.py`
/// (directives documented there) and, when built, the real helper binary.
@MainActor
final class ProposalPreviewTests: XCTestCase {
    static let fakeHelper = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent().appendingPathComponent("Fixtures/fake_assistant_context.py")
    static let fakeProvider = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent().appendingPathComponent("Fixtures/fake_assistant_provider.py")

    private func input(_ text: String, anchorByte: Int, revision: Int = 1, anchorRevision: Int? = nil,
                       context: String? = nil) -> ProposalPreview.Input {
        let ctx = context ?? String(decoding: Array(text.utf8.dropFirst(anchorByte).prefix(Insertion.contextLength)), as: UTF8.self)
        return .init(documents: [.init(path: "main.tex", text: text)], entryPath: "main.tex",
                     anchor: InsertionAnchor(id: "a1", path: "main.tex", byteOffset: anchorByte,
                                             revision: anchorRevision ?? revision, contextAfter: ctx),
                     editorRevision: revision, projectId: "demo")
    }

    private func makePreview(explanation: ProposalPreview.ExplanationConfiguration = .disabled) -> ProposalPreview {
        ProposalPreview(executable: WorkerClientTests.python, arguments: [WorkerClientTests.fakeWorker.path],
                        explanation: explanation)
    }

    /// Fake helper (python) plus optionally the fake provider (launched as the
    /// user's command would be: by path, via its shebang), short timeouts.
    private func fakeExplanation(provider: Bool = false, helperTimeout: TimeInterval = 5,
                                 providerTimeout: TimeInterval = 5) -> ProposalPreview.ExplanationConfiguration {
        var c = ProposalPreview.ExplanationConfiguration(helper: WorkerClientTests.python, helperArguments: [Self.fakeHelper.path])
        if provider { c.provider = Self.fakeProvider }
        c.helperTimeout = helperTimeout
        c.providerTimeout = providerTimeout
        return c
    }

    private func waitForReady(_ preview: ProposalPreview) async throws {
        try await waitUntil("preview ready") { if case .ready = preview.state { return true }; return false }
    }

    private func waitForExplanationSettled(_ preview: ProposalPreview) async throws {
        try await waitUntil("explanation settled", timeout: 8) { !preview.explanationInFlight }
    }

    private func waitUntil(_ what: String, timeout: TimeInterval = 5, _ cond: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if cond() { return }
            try await Task.sleep(nanoseconds: 20_000_000)
        }
        XCTFail("timed out waiting for \(what)")
    }

    // MARK: shadow text

    func testShadowAppliesInsertionWithNewlineRules() throws {
        // Mid-line anchor: newline added on both sides, matching approval.
        let mid = try ProposalPreview.makeShadow(input: input("abc def\n", anchorByte: 3), latex: "  X  \n").get()
        XCTAssertEqual(mid.shadowText, "abc\nX\n def\n")
        XCTAssertEqual(mid.insertedText, "\nX\n")
        XCTAssertEqual(mid.insertedRange, 3..<6)
        XCTAssertEqual(mid.bodyStart, 4)
        // Line start with text after: only a trailing newline.
        let start = try ProposalPreview.makeShadow(input: input("abc\ndef\n", anchorByte: 4), latex: "X").get()
        XCTAssertEqual(start.shadowText, "abc\nX\ndef\n")
        XCTAssertEqual(start.bodyStart, 4)
        // End of buffer after a newline: nothing added.
        let end = try ProposalPreview.makeShadow(input: input("abc\n", anchorByte: 4), latex: "X").get()
        XCTAssertEqual(end.shadowText, "abc\nX")
        // Same text as `Insertion.insertionText` would produce for approval.
        XCTAssertEqual(end.insertedText, Insertion.insertionText("X", into: "abc\n", atByte: 4))
        // Other documents are untouched and the original input is never mutated.
        var multi = input("abc\n", anchorByte: 0)
        multi.documents.append(.init(path: "other.tex", text: "keep"))
        let shadow = try ProposalPreview.makeShadow(input: multi, latex: "X").get()
        XCTAssertEqual(shadow.documents.map(\.text), ["X\nabc\n", "keep"])
        XCTAssertEqual(multi.documents.map(\.text), ["abc\n", "keep"])
    }

    func testShadowRebasesOrRefusesStaleAnchors() {
        // Anchor pinned at revision 1; buffer now at revision 3 with a prefix: rebased via context.
        let rebased = ProposalPreview.makeShadow(
            input: input("PRE abc\ndef\n", anchorByte: 4, revision: 3, anchorRevision: 1, context: "abc\ndef\n"), latex: "X")
        XCTAssertEqual(try rebased.get().insertByte, 4)
        // Context gone: refused, never guessed.
        let gone = ProposalPreview.makeShadow(
            input: input("zzz\n", anchorByte: 0, revision: 3, anchorRevision: 1, context: "abc"), latex: "X")
        guard case .failure(let why) = gone else { return XCTFail("expected refusal") }
        XCTAssertTrue(why.message.contains("deleted or changed"), why.message)
        // No anchor / empty proposal are not previewable.
        var noAnchor = input("abc\n", anchorByte: 0); noAnchor.anchor = nil
        XCTAssertEqual(ProposalPreview.makeShadow(input: noAnchor, latex: "X"), .failure(.init(message: "no insertion point pinned")))
        XCTAssertEqual(ProposalPreview.makeShadow(input: input("abc\n", anchorByte: 0), latex: "  \n"),
                       .failure(.init(message: "proposal is empty")))
    }

    // MARK: diffing (pure)

    func testNeighborhoodCoversEnclosingLinesPlusOneEitherSide() {
        let text = "l0\nl1\nl2\nl3\nl4\n" // each line 3 bytes
        XCTAssertEqual(ProposalPreview.neighborhood(of: 6..<8, in: text), 3..<11)   // l1..l3
        XCTAssertEqual(ProposalPreview.neighborhood(of: 0..<1, in: text), 0..<5)    // l0..l1
        XCTAssertEqual(ProposalPreview.neighborhood(of: 14..<15, in: text), 9..<15) // l3..end
        XCTAssertEqual(ProposalPreview.neighborhood(of: 2..<10, in: "abc"), 0..<3)  // clamped
    }

    func testNewDiagnosticsShiftRangesAfterTheInsertion() throws {
        let shadow = try ProposalPreview.makeShadow(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new").get()
        XCTAssertEqual(shadow.shadowText, "A\n%diag:0 new\n%diag:1 tail\n")
        func d(_ msg: String, _ at: Int, _ sev: RuntimeV1.Severity = .error) -> RuntimeV1.Diagnostic {
            .init(severity: sev, message: msg, source: .init(path: "main.tex", startByte: at, endByte: at + 1), recovery: nil)
        }
        let baseline = [d("fake diagnostic 1", 3), RuntimeV1.Diagnostic(severity: .warning, message: "global", source: nil, recovery: nil),
                        d("far", 100)]
        let after = [d("fake diagnostic 0", 2), d("fake diagnostic 1", 15),
                     RuntimeV1.Diagnostic(severity: .warning, message: "global", source: nil, recovery: nil),
                     RuntimeV1.Diagnostic(severity: .warning, message: "another", source: nil, recovery: nil),
                     d("far", 112)] // 100 shifted by the 12 inserted bytes
        let fresh = ProposalPreview.newDiagnostics(baseline: baseline, shadow: after, shadow: shadow)
        XCTAssertEqual(fresh.map(\.message), ["fake diagnostic 0", "another"])
        // Same message at an unshifted (wrong) offset after the insertion is new, not matched.
        let wrong = ProposalPreview.newDiagnostics(baseline: baseline, shadow: [d("fake diagnostic 1", 3)], shadow: shadow)
        XCTAssertEqual(wrong.count, 1)
        // Duplicates are matched as a multiset.
        let dup = ProposalPreview.newDiagnostics(baseline: [d("m", 0)], shadow: [d("m", 0), d("m", 0)], shadow: shadow)
        XCTAssertEqual(dup.count, 1)

        let result = RuntimeV1.CompileResult(projectId: "demo-preview", revision: 2, status: .recovered,
                                             pages: [], diagnostics: after, pdfPath: nil)
        let base = RuntimeV1.CompileResult(projectId: "demo-preview", revision: 1, status: .ok,
                                           pages: [.init(number: 1, widthPt: 1, heightPt: 1, items: [])], diagnostics: baseline, pdfPath: nil)
        let report = ProposalPreview.report(shadow: shadow, result: result, baseline: base)
        XCTAssertEqual(report.pageDelta, -1)
        XCTAssertEqual(report.new.map(\.diagnostic.message), ["fake diagnostic 0", "another"])
        XCTAssertEqual(report.new[0].proposalOffset, 0)
        XCTAssertEqual(report.new[0].fragment, "A⏎%diag:0 new⏎%")
        XCTAssertEqual(report.newErrorCount, 1)
        // Nearby: both located diagnostics (line 2 is adjacent) plus the unlocated new one; "far" and "global" are unrelated.
        XCTAssertEqual(report.nearby.map(\.diagnostic.message), ["fake diagnostic 0", "fake diagnostic 1", "another"])
        XCTAssertEqual(report.unrelatedCount, 2)
    }

    func testInsertionPageAndThumbnailComeFromTheShadowResult() throws {
        let shadow = try ProposalPreview.makeShadow(input: input("one\ntwo\n", anchorByte: 4), latex: "X").get()
        XCTAssertEqual(shadow.shadowText, "one\nX\ntwo\n")
        func item(_ text: String, _ start: Int, _ end: Int) -> RuntimeV1.PageItem {
            .text(.init(text: text, xPt: 72, baselineYPt: 84, fontSizePt: 12,
                        source: .init(path: "main.tex", startByte: start, endByte: end)))
        }
        let result = RuntimeV1.CompileResult(projectId: "demo-preview", revision: 1, status: .ok, pages: [
            .init(number: 1, widthPt: 612, heightPt: 792, items: [item("one", 0, 3)]),
            .init(number: 2, widthPt: 612, heightPt: 792, items: [item("X", 4, 5), item("two", 6, 9)]),
        ], diagnostics: [], pdfPath: nil)
        XCTAssertEqual(ProposalPreview.insertionPage(in: result, shadow: shadow), 2)
        let report = ProposalPreview.report(shadow: shadow, result: result, baseline: nil)
        XCTAssertEqual(report.insertionPage, 2)
        XCTAssertNil(report.pageDelta)
        let image = try XCTUnwrap(ProposalPreview.thumbnail(of: 2, in: result, width: 90))
        XCTAssertEqual(image.size.width, 90, accuracy: 1)
        XCTAssertEqual(image.size.height, 90 * 792 / 612, accuracy: 1)
        XCTAssertNil(ProposalPreview.thumbnail(of: 3, in: result))
        // No item maps the insertion (e.g. a failed compile with no pages).
        let empty = RuntimeV1.CompileResult(projectId: "demo-preview", revision: 1, status: .failed, pages: [], diagnostics: [], pdfPath: nil)
        XCTAssertNil(ProposalPreview.insertionPage(in: empty, shadow: shadow))
    }

    // MARK: worker-backed

    func testCompilesShadowAndBaselineAndReportsOnlyInsertedDiagnosticAsNew() async throws {
        let preview = makePreview()
        XCTAssertEqual(preview.state, .idle)
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new")
        XCTAssertEqual(preview.state, .compiling)
        try await waitUntil("preview ready") { if case .ready = preview.state { return true }; return false }
        guard case .ready(let r) = preview.state else { return XCTFail() }
        XCTAssertEqual(r.status, .ok)
        XCTAssertEqual(r.pageCount, 1)
        XCTAssertEqual(r.pageDelta, 0)
        XCTAssertEqual(r.new.map(\.diagnostic.message), ["fake diagnostic 0"])
        XCTAssertEqual(r.new[0].diagnostic.recovery, "test double: byte skipped")
        XCTAssertEqual(r.nearby.count, 2)
        XCTAssertNil(r.insertionPage, "the fake maps only the first line; the insertion is on line 2")
        XCTAssertNil(preview.thumbnail)
        XCTAssertEqual(preview.shadowCompileCount, 1)
        XCTAssertEqual(preview.baselineCompileCount, 1)
        XCTAssertTrue(preview.hasNewErrors)
        XCTAssertTrue(preview.statusText.hasPrefix("preview: ok, +0 pages, 1 new diagnostic"), preview.statusText)

        // Editing the proposal recompiles the shadow only; the baseline is reused.
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "clean")
        try await waitUntil("second result") { preview.shadowCompileCount == 2 && !preview.isInFlight }
        guard case .ready(let r2) = preview.state else { return XCTFail("\(preview.state)") }
        XCTAssertTrue(r2.new.isEmpty)
        XCTAssertFalse(preview.hasNewErrors)
        XCTAssertEqual(preview.baselineCompileCount, 1)
        XCTAssertTrue(preview.workerIsRunning)
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    func testBurstOfEditsIsDebouncedAndCoalesced() async throws {
        let preview = makePreview()
        let doc = input("abc\n", anchorByte: 4)
        for i in 1...5 {
            preview.update(input: doc, latex: String(repeating: "x", count: i))
            try await Task.sleep(nanoseconds: 30_000_000)
        }
        XCTAssertEqual(preview.shadowCompileCount, 0, "nothing is sent inside the debounce window")
        try await waitUntil("ready") { if case .ready = preview.state { return true }; return false }
        try await Task.sleep(nanoseconds: 400_000_000) // let any (wrong) extra compiles surface
        XCTAssertEqual(preview.shadowCompileCount, 1)
        XCTAssertEqual(preview.baselineCompileCount, 1)
        XCTAssertLessThanOrEqual(preview.shadowCompileCount + preview.baselineCompileCount, 2)
        XCTAssertEqual(preview.shadow?.shadowText, "abc\nxxxxx")

        // Edits arriving while a compile is in flight (the fake delays `%slow`
        // documents) coalesce into exactly one more compile carrying the newest text.
        let slow = input("%slow\nabc\n", anchorByte: 10)
        preview.update(input: slow, latex: "first")
        try await waitUntil("in flight") { preview.isInFlight }
        XCTAssertEqual(preview.shadowCompileCount, 2)
        XCTAssertEqual(preview.baselineCompileCount, 2, "documents changed: one new baseline")
        preview.update(input: slow, latex: "second")
        preview.update(input: slow, latex: "third")
        try await waitUntil("third compiled") {
            preview.shadow?.shadowText == "%slow\nabc\nthird" && !preview.isInFlight && preview.debounceIdle
        }
        try await Task.sleep(nanoseconds: 200_000_000)
        XCTAssertEqual(preview.shadowCompileCount, 3)
        XCTAssertEqual(preview.baselineCompileCount, 2)
        guard case .ready = preview.state else { return XCTFail("\(preview.state)") }
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    func testNoCompilerGivesClearStatusAndNeverLaunches() {
        let preview = ProposalPreview(executable: nil)
        XCTAssertEqual(preview.state, .noCompiler)
        preview.update(input: input("abc\n", anchorByte: 0), latex: "X")
        XCTAssertEqual(preview.state, .noCompiler)
        XCTAssertEqual(preview.statusText, "no compiler attached — cannot preview")
        XCTAssertFalse(preview.workerIsRunning)
        XCTAssertEqual(preview.shadowCompileCount, 0)
    }

    func testAnchorlessProposalIsNotPreviewable() {
        let preview = makePreview()
        var noAnchor = input("abc\n", anchorByte: 0); noAnchor.anchor = nil
        preview.update(input: noAnchor, latex: "X")
        XCTAssertEqual(preview.state, .notPreviewable("no insertion point pinned"))
        XCTAssertFalse(preview.workerIsRunning)
    }

    func testWorkerFaultIsReportedAndRetryRelaunches() async throws {
        let preview = makePreview()
        // `%trailing` makes the fake exit mid-line: a protocol violation, then exit.
        preview.update(input: input("%trailing\n", anchorByte: 10), latex: "X")
        try await waitUntil("failure") { if case .failed = preview.state { return true }; return false }
        try await waitUntil("worker gone") { !preview.workerIsRunning }
        XCTAssertTrue(preview.statusText.hasPrefix("preview failed: protocol violation"), preview.statusText)
        // Retry relaunches a worker and re-sends the same request (which faults again).
        preview.retry()
        XCTAssertEqual(preview.state, .compiling)
        XCTAssertEqual(preview.shadowCompileCount, 2)
        try await waitUntil("second failure") { if case .failed = preview.state { return true }; return false }
        try await waitUntil("worker gone again") { !preview.workerIsRunning }
        // A worker `error` envelope is reported too, and the worker stays usable.
        preview.update(input: input("%error\n", anchorByte: 7), latex: "X")
        try await waitUntil("error reported") { preview.statusText.hasPrefix("preview failed: worker error") }
        XCTAssertTrue(preview.workerIsRunning)
        // The next edit compiles normally on the live worker.
        preview.update(input: input("fine\n", anchorByte: 5), latex: "X")
        try await waitUntil("recovered") { if case .ready = preview.state { return true }; return false }
        XCTAssertTrue(preview.workerIsRunning)
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    func testRealModelResultAndRevisionAreUntouched() async throws {
        let model = ShellModel()
        XCTAssertNil(model.loadError, model.loadError ?? "")
        let fixture = model.result, revision = model.editorRevision, docs = model.documents
        model.caretUTF16 = 0
        model.pinAnchorAtCaret()
        XCTAssertNotNil(model.anchor)

        let preview = makePreview()
        preview.update(from: model, latex: "%diag:0 inserted")
        try await waitUntil("ready") { if case .ready = preview.state { return true }; return false }
        guard case .ready(let r) = preview.state else { return XCTFail() }
        XCTAssertEqual(r.new.count, 1)
        XCTAssertEqual(preview.shadow?.shadowText, "%diag:0 inserted\nHello FlashTeX.\n")
        XCTAssertEqual(r.insertionPage, 1, "the fake maps the first line, which the insertion now occupies")
        XCTAssertNotNil(preview.thumbnail)

        XCTAssertEqual(model.result, fixture)
        XCTAssertEqual(model.editorRevision, revision)
        XCTAssertEqual(model.documents, docs)
        XCTAssertTrue(model.isFixture)
        XCTAssertFalse(model.workerAttached)
        XCTAssertNil(model.pendingEdit)
        XCTAssertTrue(model.inFlightRequests.isEmpty)
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    // MARK: assistant explanation (crates/assistant-context helper)

    func testExplanationPurePieces() throws {
        // The compile result round-trips as the runtime-v1 envelope the helper binds
        // to; an unlocated diagnostic carries an explicit `source: null`.
        let located = RuntimeV1.Diagnostic(severity: .error, message: "e", source: .init(path: "main.tex", startByte: 2, endByte: 3), recovery: nil)
        let unlocated = RuntimeV1.Diagnostic(severity: .warning, message: "w", source: nil, recovery: nil)
        let result = RuntimeV1.CompileResult(projectId: "demo-preview", revision: 2, status: .recovered,
                                             pages: [.init(number: 1, widthPt: 1, heightPt: 1, items: [])],
                                             diagnostics: [located, unlocated], pdfPath: nil)
        let obj = try XCTUnwrap(ProposalPreview.compilerResultJSON(result, id: "preview-2") as? [String: Any])
        XCTAssertEqual(obj["id"] as? String, "preview-2")
        XCTAssertEqual(obj["type"] as? String, "compile_result")
        XCTAssertEqual(obj["protocol_version"] as? Int, 1)
        let payload = try XCTUnwrap(obj["payload"] as? [String: Any])
        XCTAssertEqual(payload["revision"] as? Int, 2)
        XCTAssertEqual(payload["status"] as? String, "recovered")
        let diags = try XCTUnwrap(payload["diagnostics"] as? [[String: Any]])
        XCTAssertEqual((diags[0]["source"] as? [String: Any])?["start_byte"] as? Int, 2)
        XCTAssertTrue(diags[1]["source"] is NSNull, "\(String(describing: diags[1]["source"]))")

        // Selection: diagnostics the insertion introduced first, then nearby ones, by index.
        let shadow = try ProposalPreview.makeShadow(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new").get()
        XCTAssertEqual(shadow.bodyRange, 2..<13)
        func d(_ msg: String, _ at: Int) -> RuntimeV1.Diagnostic {
            .init(severity: .error, message: msg, source: .init(path: "main.tex", startByte: at, endByte: at + 1), recovery: nil)
        }
        let after = RuntimeV1.CompileResult(projectId: "demo-preview", revision: 2, status: .ok, pages: [],
                                            diagnostics: [d("far", 200), d("fake diagnostic 1", 15), d("fake diagnostic 0", 2)], pdfPath: nil)
        XCTAssertEqual(ProposalPreview.selectedDiagnostics(shadow: shadow, result: after, baseline: nil), [2, 1])
        let base = RuntimeV1.CompileResult(projectId: "demo-preview", revision: 1, status: .ok, pages: [],
                                           diagnostics: [d("far", 188), d("fake diagnostic 1", 3)], pdfPath: nil)
        XCTAssertEqual(ProposalPreview.selectedDiagnostics(shadow: shadow, result: after, baseline: base), [2, 1])

        // The edit boundary is the proposal body clipped to a supplied snippet; no snippet, no edits.
        func snippetPayload(_ ranges: [(Int, Int)], path: String = "main.tex") -> [String: Any] {
            ["diagnostics": ranges.map { ["snippet": ["location": ["path": path, "start_byte": $0.0, "end_byte": $0.1]]] }, "related": []]
        }
        XCTAssertEqual(ProposalPreview.destinations(for: shadow, in: snippetPayload([(0, 27)])),
                       [.init(path: "main.tex", startByte: 2, endByte: 13)])
        XCTAssertEqual(ProposalPreview.destinations(for: shadow, in: snippetPayload([(0, 5), (4, 12)])),
                       [.init(path: "main.tex", startByte: 4, endByte: 12)], "the snippet overlapping the body most, clipped")
        XCTAssertEqual(ProposalPreview.destinations(for: shadow, in: snippetPayload([(14, 27)])), [])
        XCTAssertEqual(ProposalPreview.destinations(for: shadow, in: snippetPayload([(0, 27)], path: "other.tex")), [])

        // A validated proposal is refused when it claims application, answers
        // another context, or reaches outside the explicit boundary.
        let context = ProposalPreview.ExplanationContext(
            requestId: "explain-1", shadowRequestId: "preview-2", projectId: "demo-preview", compileRevision: 2,
            contextId: String(repeating: "a", count: 64), compilerStatus: "ok", diagnosticCount: 1, omittedDiagnostics: 0,
            allowedEdits: [.init(path: "main.tex", startByte: 2, endByte: 13)], providerIntent: "grok", payload: Data())
        func reply(_ applied: Bool, id: String = String(repeating: "a", count: 64), start: Int = 2, end: Int = 9) throws -> Data {
            try JSONSerialization.data(withJSONObject: [
                "type": "validated_proposal", "applied": applied,
                "payload": ["context_id": id, "explanation": "because",
                            "edits": [["location": ["path": "main.tex", "start_byte": start, "end_byte": end],
                                       "removed_text": "%diag:0", "replacement": "%ok"]]]] as [String: Any])
        }
        let ok = try ProposalPreview.explanation(from: reply(false), context: context, shadow: shadow)
        XCTAssertEqual(ok.text, "because")
        XCTAssertFalse(ok.applied)
        XCTAssertNil(ok.reviewId)
        XCTAssertEqual(ok.edits.map(\.proposalOffset), [0])
        XCTAssertEqual(ok.edits[0].replacement, "%ok")
        // A `proposal_review` must carry a review id and the approval requirement.
        func review(_ requires: Bool, id: String = String(repeating: "c", count: 64)) throws -> Data {
            var obj = try XCTUnwrap(JSONSerialization.jsonObject(with: reply(false)) as? [String: Any])
            obj["type"] = "proposal_review"; obj["review_id"] = id; obj["requires_user_approval"] = requires
            return try JSONSerialization.data(withJSONObject: obj)
        }
        let reviewed = try ProposalPreview.explanation(from: review(true), context: context, shadow: shadow, expecting: "proposal_review")
        XCTAssertEqual(reviewed.reviewId, String(repeating: "c", count: 64))
        XCTAssertThrowsError(try ProposalPreview.explanation(from: review(false), context: context, shadow: shadow, expecting: "proposal_review"))
        XCTAssertThrowsError(try ProposalPreview.explanation(from: review(true, id: "short"), context: context, shadow: shadow, expecting: "proposal_review"))
        XCTAssertThrowsError(try ProposalPreview.explanation(from: review(true), context: context, shadow: shadow)) {
            XCTAssertTrue("\($0)".contains("instead of validated_proposal"), "\($0)")
        }
        XCTAssertThrowsError(try ProposalPreview.explanation(from: reply(true), context: context, shadow: shadow)) {
            XCTAssertTrue("\($0)".contains("claims the edit was applied"), "\($0)")
        }
        XCTAssertThrowsError(try ProposalPreview.explanation(from: reply(false, id: String(repeating: "b", count: 64)), context: context, shadow: shadow))
        XCTAssertThrowsError(try ProposalPreview.explanation(from: reply(false, start: 0, end: 3), context: context, shadow: shadow)) {
            XCTAssertTrue("\($0)".contains("outside the proposal boundary"), "\($0)")
        }
        XCTAssertThrowsError(try ProposalPreview.explanation(from: Data("nope".utf8), context: context, shadow: shadow))

        // Provider disabled by default; only an executable path enables it.
        let none = ProposalPreview.ExplanationConfiguration.fromEnvironment(["FLASHTEX_KEYCHAIN_OFF": "1"])
        XCTAssertNil(none.provider)
        XCTAssertNil(none.grok, "no key in this environment: Grok auto mode stays off")
        XCTAssertNil(ProposalPreview.ExplanationConfiguration.fromEnvironment(["FLASHTEX_ASSISTANT_PROVIDER": "/nonexistent/provider"]).provider)
        let env = ProposalPreview.ExplanationConfiguration.fromEnvironment([
            "FLASHTEX_ASSISTANT_PROVIDER": WorkerClientTests.python.path, "FLASHTEX_ASSISTANT_CONTEXT": WorkerClientTests.python.path,
            "FLASHTEX_ASSISTANT_TIMEOUT_S": "500"])
        XCTAssertEqual(env.provider, WorkerClientTests.python)
        XCTAssertEqual(env.helper, WorkerClientTests.python)
        XCTAssertEqual(env.providerTimeout, 120, "capped at the crate's flight limit")
    }

    func testExplainWithoutHelperOrBeforeCompileIsRefusedHonestly() async throws {
        let preview = makePreview(explanation: .disabled)
        XCTAssertEqual(preview.explanationState, .idle)
        XCTAssertFalse(preview.explanationAvailable)
        XCTAssertFalse(preview.canExplain)
        preview.explain()
        guard case .unavailable(let why) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(why.contains("flashtex-assistant-context"), why)
        XCTAssertEqual(preview.explanationRequestCount, 0)

        // With a helper but before the shadow compile is current: no request is made.
        let slow = makePreview(explanation: fakeExplanation())
        slow.update(input: input("%slow\nabc\n", anchorByte: 10), latex: "X")
        XCTAssertFalse(slow.canExplain)
        slow.explain()
        XCTAssertEqual(slow.explanationState, .failed("the preview compile is not current; wait for it to finish"))
        XCTAssertEqual(slow.explanationRequestCount, 0)
        slow.close()
        try await waitUntil("worker terminated") { !slow.workerIsRunning }
    }

    func testExplainPreparesRevisionBoundContextAndSendsNothingWithoutProvider() async throws {
        let preview = makePreview(explanation: fakeExplanation())
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(preview)
        XCTAssertTrue(preview.canExplain)
        XCTAssertTrue(preview.explanationStatusText.contains("disabled (set FLASHTEX_ASSISTANT_PROVIDER"), preview.explanationStatusText)
        preview.explain()
        XCTAssertEqual(preview.explanationState, .preparing)
        XCTAssertTrue(preview.explanationInFlight)
        XCTAssertFalse(preview.canExplain)
        try await waitForExplanationSettled(preview)
        guard case .prepared(let c) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(c.shadowRequestId, "preview-2", "baseline was preview-1; the shadow compile is what gets explained")
        XCTAssertEqual(c.compileRevision, 2)
        XCTAssertEqual(c.projectId, "demo-preview")
        XCTAssertEqual(c.contextId.count, 64)
        XCTAssertEqual(c.diagnosticCount, 2, "the new diagnostic plus the pre-existing one on the next line")
        XCTAssertEqual(c.omittedDiagnostics, 0)
        XCTAssertEqual(c.allowedEdits, [.init(path: "main.tex", startByte: 2, endByte: 13)], "edits are confined to the proposal body")
        XCTAssertEqual(c.providerIntent, "fake")
        XCTAssertGreaterThan(c.payloadBytes, 0)
        let payload = try XCTUnwrap(JSONSerialization.jsonObject(with: c.payload) as? [String: Any])
        XCTAssertEqual(payload["context_id"] as? String, c.contextId)
        XCTAssertTrue((payload["user_instruction"] as? String ?? "").contains("bytes 2..<13"), "\(payload["user_instruction"] ?? "")")
        XCTAssertTrue(preview.explanationStatusText.contains("No provider is enabled — nothing was sent"), preview.explanationStatusText)
        XCTAssertEqual(preview.explanationRequestCount, 1)
        XCTAssertEqual(preview.staleExplanationReplies, 0)
        // The preview itself is untouched by the explanation.
        guard case .ready(let r) = preview.state else { return XCTFail("\(preview.state)") }
        XCTAssertEqual(r.new.map(\.diagnostic.message), ["fake diagnostic 0"])
        XCTAssertEqual(preview.shadow?.shadowText, "A\n%diag:0 new\n%diag:1 tail\n")
        preview.close()
        XCTAssertEqual(preview.explanationState, .cancelled("review closed"))
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    func testProviderReplyIsValidatedAndShownAsUnappliedModelOutput() async throws {
        let preview = makePreview(explanation: fakeExplanation(provider: true))
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(preview)
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .ready(let e) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertFalse(e.applied)
        XCTAssertTrue(e.text.hasPrefix("Fake explanation of 2 diagnostic(s)"), e.text)
        XCTAssertEqual(e.context.shadowRequestId, "preview-2")
        XCTAssertEqual(e.edits.count, 1)
        XCTAssertEqual(e.edits[0].range, .init(path: "main.tex", startByte: 2, endByte: 13))
        XCTAssertEqual(e.edits[0].removedText, "%diag:0 new")
        XCTAssertEqual(e.edits[0].replacement, "% reviewed: %diag:0 new")
        XCTAssertEqual(e.edits[0].proposalOffset, 0)
        let reviewId = try XCTUnwrap(e.reviewId)
        XCTAssertEqual(reviewId.count, 64)
        XCTAssertTrue(preview.explanationStatusText.contains("1 proposed edit, none applied; review \(reviewId.prefix(8)) awaits your approval"), preview.explanationStatusText)
        XCTAssertTrue(preview.canApproveReviewedEdit)
        XCTAssertNil(preview.amendedProposalLatex)
        // Display only: the shadow text and the compile report are exactly as before.
        XCTAssertEqual(preview.shadow?.shadowText, "A\n%diag:0 new\n%diag:1 tail\n")
        guard case .ready(let r) = preview.state else { return XCTFail("\(preview.state)") }
        XCTAssertEqual(r.new.count, 1)

        // Explicit approval of that exact review: the helper's `approve` returns a
        // grouped edit bound to the shadow text's revision/hash, mapped onto the
        // proposal text only. The shadow (and the document) are unchanged.
        preview.approveReviewedEdit()
        XCTAssertEqual(preview.explanationState, .approving(e))
        XCTAssertFalse(preview.canApproveReviewedEdit)
        try await waitForExplanationSettled(preview)
        guard case .approved(let a) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(a.reviewId, reviewId)
        XCTAssertEqual(a.commandId, "assistant-\(reviewId)")
        XCTAssertEqual(a.expectedRevision, 1)
        XCTAssertEqual(a.expectedSha256, SourceDigest.sha256Hex("A\n%diag:0 new\n%diag:1 tail\n"))
        XCTAssertEqual(a.label, "Apply reviewed AI proposal")
        XCTAssertFalse(a.applied)
        XCTAssertEqual(a.amendedLatex, "% reviewed: %diag:0 new")
        XCTAssertEqual(preview.amendedProposalLatex, "% reviewed: %diag:0 new")
        XCTAssertEqual(preview.shadow?.shadowText, "A\n%diag:0 new\n%diag:1 tail\n")
        XCTAssertTrue(preview.explanationStatusText.contains("applied to the proposal text only"), preview.explanationStatusText)
        XCTAssertEqual(preview.explanationRequestCount, 1)
        // Approving again is refused: nothing reviewed is current any more.
        preview.approveReviewedEdit()
        XCTAssertEqual(preview.explanationState, .failed("no reviewed edit is current; explain again first"))

        // Explanation-only replies go through `validate`: no review id, nothing to approve.
        preview.update(input: input("A\n%diag:1 tail %noeditprovider\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitUntil("recompiled") { preview.shadowCompileCount == 2 && !preview.isInFlight }
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .ready(let only) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertNil(only.reviewId)
        XCTAssertTrue(only.edits.isEmpty)
        XCTAssertFalse(preview.canApproveReviewedEdit)

        // A provider answering another context, or a helper claiming application, is refused.
        preview.update(input: input("A\n%diag:1 tail %badprovider\n", anchorByte: 2), latex: "%diag:0 new")
        XCTAssertEqual(preview.explanationState, .cancelled("proposal or document changed"))
        try await waitUntil("recompiled") { preview.shadowCompileCount == 3 && !preview.isInFlight }
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .failed(let why) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(why.contains("helper refused (review): invalid explanation identity or bounds"), why)
        preview.update(input: input("A\n%diag:1 tail %appliedexplain\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitUntil("recompiled") { preview.shadowCompileCount == 4 && !preview.isInFlight }
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .failed(let claim) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(claim.contains("claims the edit was applied; refusing"), claim)
        XCTAssertEqual(preview.explanationRequestCount, 4)
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    func testExplanationIsCancelledWhenProposalOrRevisionChangesAndLateRepliesAreDropped() async throws {
        let preview = makePreview(explanation: fakeExplanation(provider: true))
        // Both doubles sleep 600 ms so each cancellation lands mid-flight.
        let doc = input("A %slowexplain %slowprovider\n%diag:1 tail\n", anchorByte: 29)
        preview.update(input: doc, latex: "%diag:0 new")
        try await waitForReady(preview)
        preview.explain()
        XCTAssertEqual(preview.explanationState, .preparing)
        try await Task.sleep(nanoseconds: 100_000_000) // the helper is mid-sleep
        // Editing the proposal cancels the in-flight helper; its completion is
        // then a stale reply and changes nothing.
        preview.update(input: doc, latex: "%diag:0 changed")
        XCTAssertEqual(preview.explanationState, .cancelled("proposal or document changed"))
        XCTAssertFalse(preview.explanationInFlight)
        try await waitUntil("stale reply counted") { preview.staleExplanationReplies == 1 }
        XCTAssertEqual(preview.explanationState, .cancelled("proposal or document changed"))
        try await waitUntil("recompiled") { preview.shadowCompileCount == 2 && !preview.isInFlight }

        // Same for a document revision change while the provider is running.
        preview.explain()
        try await waitUntil("provider running") { if case .awaitingProvider = preview.explanationState { return true }; return false }
        var bumped = doc; bumped.editorRevision = 2
        preview.update(input: bumped, latex: "%diag:0 changed")
        XCTAssertEqual(preview.explanationState, .cancelled("proposal or document changed"))
        try await waitUntil("second stale reply") { preview.staleExplanationReplies == 2 }
        try await waitUntil("recompiled") { preview.shadowCompileCount == 3 && !preview.isInFlight }

        // An identical update (same documents, anchor, revision, text) keeps a
        // finished explanation; only real changes discard it.
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .ready(let e) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(e.context.shadowRequestId, "preview-4")
        preview.update(input: bumped, latex: "%diag:0 changed")
        XCTAssertEqual(preview.explanationState, .ready(e))
        var moved = bumped
        moved.anchor = InsertionAnchor(id: "a1", path: "main.tex", byteOffset: 16, revision: 2, contextAfter: "diag:1 tail\n")
        preview.update(input: moved, latex: "%diag:0 changed")
        XCTAssertEqual(preview.explanationState, .cancelled("proposal or document changed"))
        XCTAssertEqual(preview.explanationRequestCount, 3)
        XCTAssertEqual(preview.staleExplanationReplies, 2)
        try await waitUntil("recompiled") { preview.shadowCompileCount == 4 && !preview.isInFlight }
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    func testOutOfOrderOrMismatchedRepliesNeverChangeTheDisplayedState() async throws {
        let preview = makePreview(explanation: fakeExplanation())
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(preview)
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .prepared(let c) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        // A well-formed validated proposal for an older request id is dropped.
        let forged = try JSONSerialization.data(withJSONObject: [
            "type": "validated_proposal", "applied": false,
            "payload": ["context_id": c.contextId, "explanation": "late", "edits": []]] as [String: Any])
        for id in ["explain-0", "explain-1", "explain-2"] {
            preview.handleExplanationReply(id: id, stage: .validate, result: .success(.init(stdout: forged, stderr: "")))
        }
        XCTAssertEqual(preview.explanationState, .prepared(c))
        XCTAssertEqual(preview.staleExplanationReplies, 3)

        // Mid-flight, a reply with the right id but the wrong stage is dropped too.
        preview.update(input: input("A %slowexplain\n%diag:1 tail\n", anchorByte: 15), latex: "%diag:0 new")
        try await waitUntil("recompiled") { preview.shadowCompileCount == 2 && !preview.isInFlight }
        preview.explain()
        XCTAssertEqual(preview.explanationState, .preparing)
        preview.handleExplanationReply(id: "explain-2", stage: .validate, result: .success(.init(stdout: forged, stderr: "")))
        preview.handleExplanationReply(id: "explain-2", stage: .approve, result: .failure(.cancelled))
        XCTAssertEqual(preview.explanationState, .preparing, "a forged failure for the wrong stage does not fail the request")
        XCTAssertEqual(preview.staleExplanationReplies, 5)
        try await waitForExplanationSettled(preview)
        guard case .prepared(let c2) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(c2.shadowRequestId, "preview-4", "changed documents: baseline preview-3, shadow preview-4")
        XCTAssertNotEqual(c2.contextId, c.contextId)

        // A helper answering another compile revision is rejected outright.
        preview.update(input: input("A %wrongrevexplain\n%diag:1 tail\n", anchorByte: 19), latex: "%diag:0 new")
        try await waitUntil("recompiled") { preview.shadowCompileCount == 3 && !preview.isInFlight }
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .failed(let why) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(why.contains("helper answered revision 1006 of demo-preview, not preview-6"), why)
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    /// Native recovery: a helper or provider that crashes, hangs, or answers
    /// garbage mid-review leaves the sheet usable, the buffer untouched and the
    /// proposal approvable/rejectable.
    func testHelperCrashTimeoutOrGarbageLeavesReviewUsableAndBufferUntouched() async throws {
        let model = ShellModel()
        XCTAssertNil(model.loadError, model.loadError ?? "")
        let fixture = model.result, revision = model.editorRevision, docs = model.documents
        model.caretUTF16 = 0
        model.pinAnchorAtCaret()
        let proposal = RuntimeV1.CaptureProposal(captureId: "cap-recovery", latex: "%diag:0 inserted", ambiguities: [], requiredDependencies: [])
        model.enqueue(proposal)

        let preview = makePreview(explanation: fakeExplanation(provider: true, helperTimeout: 0.4, providerTimeout: 0.4))
        let cases: [(latex: String, expect: String)] = [
            ("%diag:0 %crashexplain", "helper exited (3) during probe"),
            ("%diag:0 %hangexplain", "helper gave no reply within 1 s (probe)"),
            ("%diag:0 %garbageexplain", "helper reply is not a JSON object"),
            ("%diag:0 %hugeexplain", "helper reply of "),
            ("%diag:0 %crashprovider", "provider exited (4) during provider"),
            ("%diag:0 %hangprovider", "provider gave no reply within 1 s (provider)"),
            ("%diag:0 %garbageprovider", "the provider's reply is not a JSON object"),
        ]
        for (i, c) in cases.enumerated() {
            preview.update(from: model, latex: c.latex)
            try await waitUntil("compiled \(c.latex)") { preview.shadowCompileCount == i + 1 && !preview.isInFlight }
            guard case .ready = preview.state else { return XCTFail("\(c.latex): \(preview.state)") }
            preview.explain()
            try await waitForExplanationSettled(preview)
            guard case .failed(let why) = preview.explanationState else { return XCTFail("\(c.latex): \(preview.explanationState)") }
            XCTAssertTrue(why.contains(c.expect), "\(c.latex): \(why)")
            XCTAssertTrue(preview.explanationStatusText.hasPrefix("assistant failed: "), preview.explanationStatusText)
            // The review is still live: preview ready, shadow intact, explain available again.
            guard case .ready(let r) = preview.state else { return XCTFail("\(c.latex): \(preview.state)") }
            XCTAssertEqual(r.new.count, 1)
            XCTAssertEqual(preview.shadow?.shadowText, c.latex + "\nHello FlashTeX.\n")
            XCTAssertTrue(preview.canExplain)
            XCTAssertTrue(preview.workerIsRunning)
        }
        XCTAssertEqual(preview.explanationRequestCount, cases.count)
        // A helper that dies during `approve` leaves the reviewed explanation
        // discarded, the sheet live and the proposal text unamended.
        preview.update(from: model, latex: "%diag:0 %crashapprove")
        try await waitUntil("compiled") { preview.shadowCompileCount == cases.count + 1 && !preview.isInFlight }
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .ready(let reviewed) = preview.explanationState, reviewed.reviewId != nil else { return XCTFail("\(preview.explanationState)") }
        preview.approveReviewedEdit()
        try await waitForExplanationSettled(preview)
        guard case .failed(let approveWhy) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(approveWhy.contains("helper exited (5) during approve"), approveWhy)
        XCTAssertNil(preview.amendedProposalLatex)
        guard case .ready = preview.state else { return XCTFail("\(preview.state)") }
        XCTAssertTrue(preview.canExplain)
        try await Task.sleep(nanoseconds: 300_000_000) // let any killed helper's late completion surface
        // Nothing reached the model: documents, result, revision and pending edits are as loaded.
        XCTAssertEqual(model.documents, docs)
        XCTAssertEqual(model.result, fixture)
        XCTAssertEqual(model.editorRevision, revision)
        XCTAssertNil(model.pendingEdit)
        XCTAssertEqual(model.proposals.map(\.captureId), ["cap-recovery"])

        // "Explain again" after a failure works on the same shadow compile.
        preview.update(from: model, latex: "%diag:0 fine")
        try await waitUntil("compiled") { preview.shadowCompileCount == cases.count + 2 && !preview.isInFlight }
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .ready(let e) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertFalse(e.applied)
        XCTAssertEqual(e.edits.first?.removedText, "%diag:0 fine")

        // The reviewer can still reject, and still approve (the sheet's own path).
        model.rejectProposal(proposal)
        XCTAssertTrue(model.proposals.isEmpty)
        let again = RuntimeV1.CaptureProposal(captureId: "cap-recovery-2", latex: "approved", ambiguities: [], requiredDependencies: [])
        model.enqueue(again)
        XCTAssertEqual(model.approveProposal(again, latex: "approved"), .inserted(byteOffset: 0))
        XCTAssertEqual(model.pendingEdit?.text, "approved\n", "the reviewer's text, not the model's proposed edit")
        XCTAssertEqual(model.documents, docs, "the buffer applies pending edits through the editor, never from the preview")
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    /// The real `flashtex-assistant-context` binary (built from crates/assistant-context;
    /// FLASHTEX_ASSISTANT_CONTEXT or the repo build) with the fake provider.
    func testRealHelperBindsContextAndValidatesProviderReply() async throws {
        guard let helper = ProposalPreview.ExplanationConfiguration.locateHelper() else {
            throw XCTSkip("build crates/assistant-context or set FLASHTEX_ASSISTANT_CONTEXT")
        }
        var config = ProposalPreview.ExplanationConfiguration(helper: helper)
        config.provider = WorkerClientTests.python
        config.providerArguments = [Self.fakeProvider.path]
        config.providerTimeout = 5
        let preview = makePreview(explanation: config)
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(preview)
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .ready(let e) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertFalse(e.applied)
        XCTAssertEqual(e.context.providerIntent, "grok")
        XCTAssertEqual(e.context.shadowRequestId, "preview-2")
        XCTAssertEqual(e.context.compileRevision, 2)
        XCTAssertEqual(e.context.diagnosticCount, 2)
        XCTAssertEqual(e.context.allowedEdits, [.init(path: "main.tex", startByte: 2, endByte: 13)])
        XCTAssertLessThanOrEqual(e.context.payloadBytes, 64 * 1024)
        XCTAssertEqual(e.edits.map(\.proposalOffset), [0])
        XCTAssertEqual(e.edits[0].removedText, "%diag:0 new")
        XCTAssertTrue(e.text.hasPrefix("Fake explanation of 2 diagnostic(s)"), e.text)
        let payload = try XCTUnwrap(JSONSerialization.jsonObject(with: e.context.payload) as? [String: Any])
        XCTAssertEqual((payload["diagnostics"] as? [[String: Any]])?.map { $0["diagnostic_index"] as? Int }, [0, 1])
        XCTAssertEqual(payload["compiler_status"] as? String, "ok")
        let reviewId = try XCTUnwrap(e.reviewId)
        // Explicit approval through the real helper's `approve` (review.rs): the
        // approved group is bound to the shadow text's revision and SHA-256.
        preview.approveReviewedEdit()
        try await waitForExplanationSettled(preview)
        guard case .approved(let a) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(a.commandId, "assistant-\(reviewId)")
        XCTAssertEqual(a.expectedSha256, SourceDigest.sha256Hex("A\n%diag:0 new\n%diag:1 tail\n"))
        XCTAssertEqual(a.expectedRevision, 1)
        XCTAssertEqual(a.amendedLatex, "% reviewed: %diag:0 new")
        XCTAssertFalse(a.applied)
        XCTAssertEqual(preview.shadow?.shadowText, "A\n%diag:0 new\n%diag:1 tail\n", "approval amends the proposal text only")

        // The real helper refuses an edit outside the explicit destination.
        preview.update(input: input("A\n%diag:1 tail %outsideprovider\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitUntil("recompiled") { preview.shadowCompileCount == 2 && !preview.isInFlight }
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .failed(let why) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(why.contains("outside explicit destination"), why)

        // With no diagnostics there is no snippet, hence explanation only.
        preview.update(input: input("clean\n", anchorByte: 6), latex: "still clean")
        try await waitUntil("recompiled") { preview.shadowCompileCount == 3 && !preview.isInFlight }
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .ready(let clean) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(clean.context.allowedEdits, [])
        XCTAssertEqual(clean.context.diagnosticCount, 0)
        XCTAssertTrue(clean.edits.isEmpty)
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    /// The pinned helper (c93bf0d) reproduces its own recorded
    /// `examples/review-workflow.json` byte for byte on this machine, and the
    /// Swift adapter decodes every recorded reply the way the live path does.
    func testPinnedHelperReproducesRecordedReviewWorkflowFixture() async throws {
        guard let helper = ProposalPreview.ExplanationConfiguration.locateHelper() else {
            throw XCTSkip("build crates/assistant-context or set FLASHTEX_ASSISTANT_CONTEXT")
        }
        let fixtureURL = helper.deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
            .appendingPathComponent("examples/review-workflow.json")
        guard let data = try? Data(contentsOf: fixtureURL) else {
            throw XCTSkip("no examples/review-workflow.json next to \(helper.path) (helper predates c93bf0d)")
        }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
        XCTAssertEqual(fixture["provider_called"] as? Bool, false)
        XCTAssertEqual(fixture["source_mutated"] as? Bool, false)
        let steps = try XCTUnwrap(fixture["steps"] as? [[String: Any]])
        XCTAssertEqual(steps.count, 4)
        var replies: [Data] = []
        for step in steps.dropFirst() { // step 0 is the compiler; 1–3 are helper calls
            let request = try JSONSerialization.data(withJSONObject: step["request"] as Any)
            let recorded = try XCTUnwrap(step["response"] as? [String: Any])
            var got: Result<OneShotProcess.Output, OneShotProcess.Failure>?
            let process = try OneShotProcess(executable: helper, arguments: [], input: request, timeout: 10,
                                             maxOutputBytes: 128 * 1024) { got = $0 }
            try await waitUntil("helper reply") { got != nil }
            XCTAssertFalse(process.isRunning)
            let output = try XCTUnwrap(got).get()
            let actual = try XCTUnwrap(JSONSerialization.jsonObject(with: output.stdout) as? [String: Any])
            XCTAssertTrue((actual as NSDictionary).isEqual(to: recorded), "\(step["request"].flatMap { ($0 as? [String: Any])?["operation"] } ?? "?") differs: \(actual)")
            replies.append(output.stdout)
        }
        // Decode the recorded replies with the same functions the live path uses.
        let text = "\\documentclass{article}\n\\begin{document}\n\\unknowncommand\n\\end{document}\n"
        let original = "\\documentclass{article}\n\\begin{document}\n\\end{document}\n"
        let shadow = try ProposalPreview.makeShadow(
            input: .init(documents: [.init(path: "main.tex", text: original)], entryPath: "main.tex",
                         anchor: InsertionAnchor(id: "f", path: "main.tex", byteOffset: 41, revision: 1, contextAfter: "\\end{"),
                         editorRevision: 1, projectId: "review"),
            latex: "\\unknowncommand").get()
        XCTAssertEqual(shadow.shadowText, text, "the fixture document is the proposal inserted before \\end{document}")
        XCTAssertEqual(shadow.bodyRange, 41..<56)
        let prepareRequest = try XCTUnwrap(steps[1]["request"] as? [String: Any])
        var job = ProposalPreview.ExplanationJob(id: "offline-fixture:1", stage: .prepare, shadow: shadow, shadowRequestId: "fixture-compile",
                                                 projectId: "review-fixture", revision: 1, sources: [], request: prepareRequest,
                                                 context: nil, explanation: nil)
        let prepared = try ProposalPreview.preparedPayload(replies[0])
        XCTAssertNil(prepared["allowed_edits"] as? [[String: Any]], "the recorded fixture did not restrict edits")
        XCTAssertThrowsError(try ProposalPreview.context(from: prepared, job: job)) {
            XCTAssertTrue("\($0)".contains("no explicit edit boundary"), "the live path always restricts edits; the fixture's open context is refused: \($0)")
        }
        var restricted = prepared
        restricted["allowed_edits"] = [["path": "main.tex", "start_byte": 41, "end_byte": 56]]
        let context = try ProposalPreview.context(from: restricted, job: job)
        XCTAssertEqual(context.contextId, "8da6580242106df1d599e2bf2f070a741162e12cdd735d46af3f56330966c589")
        let reviewed = try ProposalPreview.explanation(from: replies[1], context: context, shadow: shadow, expecting: "proposal_review")
        XCTAssertEqual(reviewed.reviewId, "144af470152eec519d771017216fcc7a43e54125addf05825509bdf91876baa6")
        XCTAssertEqual(reviewed.edits.map(\.proposalOffset), [0])
        XCTAssertEqual(reviewed.edits[0].replacement, "Example text")
        job.explanation = reviewed
        let amendment = try ProposalPreview.amendment(from: replies[2], explanation: reviewed,
                                                      reviewId: reviewed.reviewId!, job: job)
        XCTAssertEqual(amendment.commandId, "assistant-144af470152eec519d771017216fcc7a43e54125addf05825509bdf91876baa6")
        XCTAssertEqual(amendment.expectedSha256, "44c533e6ed2939f77ed31171e5acad180e7e166ed48d6b0a908f16ddc5ffb348")
        XCTAssertEqual(amendment.amendedLatex, "Example text")
        XCTAssertFalse(amendment.applied)
        // Provenance the parent can compare against the fixture's recorded helper hash.
        let provenance = try XCTUnwrap(fixture["provenance"] as? [String: Any])
        XCTAssertNotNil((provenance["helper"] as? [String: Any])?["sha256"])
    }

    /// Opt-in visual evidence: renders the sheet's preview + assistant section
    /// (the exact `ProposalPreviewView` ContentView embeds) in the `.ready`
    /// (reviewed edit awaiting approval) and `.approved` states to PNGs under
    /// `FLASHTEX_EVIDENCE_DIR`. The app's proposal sheet needs an NSOpenPanel
    /// and a pinned anchor, which cannot be driven without Accessibility.
    func testRendersAssistantSectionEvidenceWhenRequested() async throws {
        guard let dir = ProcessInfo.processInfo.environment["FLASHTEX_EVIDENCE_DIR"] else {
            throw XCTSkip("set FLASHTEX_EVIDENCE_DIR to write PNG evidence")
        }
        let preview = makePreview(explanation: fakeExplanation(provider: true))
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(preview)
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .ready = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        func write(_ name: String) throws {
            let view = ProposalPreviewView(preview: preview).frame(width: 488).padding(16).background(Color(nsColor: .windowBackgroundColor))
            let renderer = ImageRenderer(content: view)
            renderer.scale = 2
            let image = try XCTUnwrap(renderer.nsImage)
            let tiff = try XCTUnwrap(image.tiffRepresentation)
            let png = try XCTUnwrap(NSBitmapImageRep(data: tiff)?.representation(using: .png, properties: [:]))
            let url = URL(fileURLWithPath: dir).appendingPathComponent(name)
            try png.write(to: url)
            XCTAssertGreaterThan(image.size.height, 100, name)
        }
        try write("mac-ai-review-assistant-reviewed-2026-09-12.png")
        preview.approveReviewedEdit()
        try await waitForExplanationSettled(preview)
        guard case .approved = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        try write("mac-ai-review-assistant-approved-2026-09-12.png")
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    // MARK: packaged discovery, provider identity, offline helper

    func testLocateHelperFindsTheBundledBinaryInTheMacOSDirectory() throws {
        let macOSDir = FileManager.default.temporaryDirectory.appendingPathComponent("flashtex-bundle-\(UUID().uuidString)/Contents/MacOS")
        try FileManager.default.createDirectory(at: macOSDir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: macOSDir) }
        let name = ProposalPreview.ExplanationConfiguration.bundledHelperName
        XCTAssertEqual(name, "flashtex-assistant-context")
        // Nothing bundled yet: falls through to the environment/scratch/crate search.
        let before = ProposalPreview.ExplanationConfiguration.locateHelper([:], bundleExecutableDirectory: macOSDir)
        XCTAssertNotEqual(before, macOSDir.appendingPathComponent(name))
        // A non-executable file is ignored; an executable one is found.
        let bundled = macOSDir.appendingPathComponent(name)
        try Data("#!/bin/sh\nexit 0\n".utf8).write(to: bundled)
        XCTAssertNotEqual(ProposalPreview.ExplanationConfiguration.locateHelper([:], bundleExecutableDirectory: macOSDir), bundled)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: bundled.path)
        XCTAssertEqual(ProposalPreview.ExplanationConfiguration.locateHelper([:], bundleExecutableDirectory: macOSDir), bundled)
        // The environment override still wins over the bundle.
        XCTAssertEqual(ProposalPreview.ExplanationConfiguration.locateHelper(["FLASHTEX_ASSISTANT_CONTEXT": WorkerClientTests.python.path],
                                                                            bundleExecutableDirectory: macOSDir), WorkerClientTests.python)
        // fromEnvironment goes through the same lookup and reports the bundled name.
        let c = ProposalPreview.ExplanationConfiguration.fromEnvironment([:], bundleExecutableDirectory: macOSDir, keychain: MemoryKeychain())
        XCTAssertEqual(c.helper, bundled)
        XCTAssertNil(c.provider)
        XCTAssertEqual(c.providerIdentityText, "provider disabled (set FLASHTEX_ASSISTANT_PROVIDER to a local command)")
    }

    func testHelperChildEnvironmentIsOfflineAndCredentialFree() {
        let env = ["PATH": "/usr/bin", "HOME": "/Users/x", "LANG": "en_US.UTF-8", "TMPDIR": "/tmp/x",
                   "FLASHTEX_GROK_API_KEY": "k", "OPENAI_API_KEY": "k", "XAI_TOKEN": "k", "AWS_SECRET_ACCESS_KEY": "k",
                   "HTTPS_PROXY": "http://proxy", "http_proxy": "http://proxy", "MY_PASSWORD": "k", "SOME_CREDENTIAL": "k",
                   "FLASHTEX_COMPILER": "/x", "PWD": "/y"]
        let helper = ProposalPreview.ExplanationConfiguration.childEnvironment(for: .helper, from: env)
        XCTAssertEqual(helper, ["PATH": "/usr/bin", "HOME": "/Users/x", "LANG": "en_US.UTF-8", "TMPDIR": "/tmp/x"])
        for key in env.keys where ProposalPreview.ExplanationConfiguration.isSensitiveVariable(key) { XCTAssertNil(helper[key], key) }
        XCTAssertTrue(ProposalPreview.ExplanationConfiguration.isSensitiveVariable("FLASHTEX_GROK_API_KEY"))
        XCTAssertFalse(ProposalPreview.ExplanationConfiguration.isSensitiveVariable("PATH"))
        // The provider is the user's own command and gets the user's environment as is.
        XCTAssertEqual(ProposalPreview.ExplanationConfiguration.childEnvironment(for: .provider, from: env), env)
    }

    /// The helper is always launched with an empty argv (never `--session` /
    /// `--provider-session`) and a credential-free environment, whatever the
    /// app's own environment holds; the pinned binary carries no provider
    /// endpoint. The provider is launched by name from
    /// `FLASHTEX_ASSISTANT_PROVIDER` with the payload on stdin only.
    func testHelperIsLaunchedOfflineWithEmptyArgvAndSanitizedEnvironment() async throws {
        let preview = makePreview(explanation: fakeExplanation(provider: true))
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(preview)
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .ready = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        preview.approveReviewedEdit()
        try await waitForExplanationSettled(preview)
        guard case .approved = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(preview.childLaunches.map(\.stage), [.probe, .prepare, .provider, .review, .approve])
        for launch in preview.childLaunches {
            // The python helper double takes the script as its only argument
            // (the real helper takes none); the provider is launched by path.
            XCTAssertEqual(launch.arguments, launch.role == .helper ? [Self.fakeHelper.path] : [])
            XCTAssertFalse(launch.arguments.contains { $0.hasPrefix("--") }, "\(launch.arguments)")
            if launch.role == .helper {
                XCTAssertTrue(Set(launch.environmentKeys).isSubset(of: ["PATH", "HOME", "TMPDIR", "LANG", "LC_ALL", "LC_CTYPE", "USER", "SHELL", "RUST_BACKTRACE"]), "\(launch.environmentKeys)")
                XCTAssertFalse(launch.environmentKeys.contains { ProposalPreview.ExplanationConfiguration.isSensitiveVariable($0) })
                XCTAssertFalse(launch.environmentKeys.contains { $0.hasPrefix("FLASHTEX_") })
            }
        }
        if let helper = ProposalPreview.ExplanationConfiguration.locateHelper() {
            // The pinned scratch build (`scratchHelperPath`) has no HTTP client and no
            // endpoint string. A checkout build made with `--features grok` for the live
            // path (GrokLiveTests) carries both; even then the one-shot launches above
            // are argv-empty and credential-free, and that build sends nothing unless
            // started with `--provider-session` (crates/assistant-context README).
            let binary = try Data(contentsOf: helper)
            // A grok build answers `--provider-session` (without a key) with
            // "explicit provider credential missing"; the default build prints usage.
            let probe = Process()
            probe.executableURL = helper
            probe.arguments = ["--provider-session", "feature-probe", "grok-4.6"]
            probe.environment = ["PATH": "/usr/bin"]
            let stderr = Pipe()
            probe.standardInput = FileHandle.nullDevice; probe.standardOutput = FileHandle.nullDevice; probe.standardError = stderr
            try probe.run()
            let said = String(decoding: stderr.fileHandleForReading.readDataToEndOfFile(), as: UTF8.self)
            probe.waitUntilExit()
            XCTAssertEqual(probe.terminationStatus, 2, said)
            let grokBuild = said.contains("explicit provider credential missing")
            if helper.path.hasSuffix(ProposalPreview.ExplanationConfiguration.scratchHelperPath) {
                XCTAssertNil(binary.range(of: Data("api.x.ai".utf8)), "pinned helper contains the provider endpoint")
                XCTAssertFalse(grokBuild, "pinned helper was built with the grok feature")
            } else if !grokBuild {
                XCTAssertNil(binary.range(of: Data("api.x.ai".utf8)), "non-grok helper contains the provider endpoint")
            }
        }
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    /// End to end from the environment, as the packaged app would run it: the
    /// pinned helper plus a LOCAL provider command from
    /// `FLASHTEX_ASSISTANT_PROVIDER` (the fake, launched by its own shebang).
    /// explanation → helper validate/review → explicit approve → amended draft;
    /// cancellation mid-provider; an oversized provider reply refused.
    func testProviderPathEndToEndFromEnvironmentWithLocalCommand() async throws {
        guard let helper = ProposalPreview.ExplanationConfiguration.locateHelper() else {
            throw XCTSkip("build crates/assistant-context or set FLASHTEX_ASSISTANT_CONTEXT")
        }
        var env = ["FLASHTEX_ASSISTANT_CONTEXT": helper.path, "FLASHTEX_ASSISTANT_PROVIDER": Self.fakeProvider.path,
                   "FLASHTEX_ASSISTANT_TIMEOUT_S": "5", "FLASHTEX_GROK_API_KEY": "must-not-reach-the-helper"]
        env["PATH"] = ProcessInfo.processInfo.environment["PATH"] // the shebang needs python3
        let config = ProposalPreview.ExplanationConfiguration.fromEnvironment(env)
        XCTAssertEqual(config.helper, helper)
        XCTAssertEqual(config.provider, Self.fakeProvider)
        XCTAssertEqual(config.providerArguments, [])
        XCTAssertEqual(config.providerTimeout, 5)
        XCTAssertEqual(config.providerIdentityText, "provider: fake_assistant_provider.py")
        let preview = makePreview(explanation: config)
        XCTAssertTrue(preview.explanationStatusText.hasSuffix("provider: fake_assistant_provider.py"), preview.explanationStatusText)

        preview.update(input: input("A\n%diag:1 tail %envprovider\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(preview)
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .ready(let e) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(e.text.hasPrefix("Fake explanation of 2 diagnostic(s)"), e.text)
        XCTAssertEqual(e.context.providerIntent, "grok", "the real helper's payload intent; transport is still the user's local command")
        let reviewId = try XCTUnwrap(e.reviewId)
        XCTAssertEqual(e.edits.count, 1)
        // The provider ran as the user's command with the app's own environment
        // (it lists the FLASHTEX_* names it saw, exactly this process's set,
        // possibly none); the helper launches got none of it.
        let visible = ProcessInfo.processInfo.environment.keys.filter { $0.hasPrefix("FLASHTEX_") }.sorted().joined(separator: ",")
        XCTAssertTrue(e.text.hasSuffix(" env: " + visible), "\(e.text) vs \(visible)")
        let helperLaunches = preview.childLaunches.filter { $0.role == .helper }
        XCTAssertEqual(helperLaunches.map(\.stage), [.probe, .prepare, .review])
        for l in helperLaunches {
            XCTAssertEqual(l.executable, helper)
            XCTAssertEqual(l.arguments, [])
            XCTAssertFalse(l.environmentKeys.contains("FLASHTEX_GROK_API_KEY"))
            XCTAssertFalse(l.environmentKeys.contains { $0.hasPrefix("FLASHTEX_") })
        }
        let providerLaunch = try XCTUnwrap(preview.childLaunches.first { $0.role == .provider })
        XCTAssertEqual(providerLaunch.executable, Self.fakeProvider)
        XCTAssertEqual(providerLaunch.arguments, [])

        preview.approveReviewedEdit()
        try await waitForExplanationSettled(preview)
        guard case .approved(let a) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(a.reviewId, reviewId)
        XCTAssertEqual(a.amendedLatex, "% reviewed: %diag:0 new")
        XCTAssertEqual(preview.amendedProposalLatex, "% reviewed: %diag:0 new")
        XCTAssertEqual(a.expectedSha256, SourceDigest.sha256Hex("A\n%diag:0 new\n%diag:1 tail %envprovider\n"))
        XCTAssertFalse(a.applied)
        XCTAssertEqual(preview.shadow?.shadowText, "A\n%diag:0 new\n%diag:1 tail %envprovider\n")

        // Cancellation while the local provider is running (it sleeps 600 ms).
        preview.update(input: input("A\n%diag:1 tail %slowprovider\n", anchorByte: 2), latex: "%diag:0 new")
        XCTAssertEqual(preview.explanationState, .cancelled("proposal or document changed"))
        try await waitUntil("recompiled") { preview.shadowCompileCount == 2 && !preview.isInFlight }
        preview.explain()
        try await waitUntil("provider running") { if case .awaitingProvider = preview.explanationState { return true }; return false }
        XCTAssertTrue(preview.explanationStatusText.contains("handed to fake_assistant_provider.py"), preview.explanationStatusText)
        let stale = preview.staleExplanationReplies
        preview.update(input: input("A\n%diag:1 tail %slowprovider\n", anchorByte: 2), latex: "%diag:0 edited")
        XCTAssertEqual(preview.explanationState, .cancelled("proposal or document changed"))
        try await waitUntil("provider reply discarded") { preview.staleExplanationReplies == stale + 1 }
        XCTAssertEqual(preview.explanationState, .cancelled("proposal or document changed"))
        try await waitUntil("recompiled") { preview.shadowCompileCount == 3 && !preview.isInFlight }

        // An oversized provider reply (above the helper's 64 KiB response limit) is refused
        // before it reaches the helper; the review stays usable.
        preview.update(input: input("A\n%diag:1 tail %hugeprovider\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitUntil("recompiled") { preview.shadowCompileCount == 4 && !preview.isInFlight }
        preview.explain()
        try await waitForExplanationSettled(preview)
        guard case .failed(let why) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(why.hasPrefix("provider reply of ") && why.hasSuffix("bytes exceeds the limit"), why)
        XCTAssertEqual(preview.childLaunches.last?.stage, .provider, "no validate/review launch followed the refused reply")
        guard case .ready = preview.state else { return XCTFail("\(preview.state)") }
        XCTAssertTrue(preview.canExplain)
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }
}
