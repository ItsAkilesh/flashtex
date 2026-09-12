import AppKit
import SwiftUI
import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// Ask Grok on the live document (GrokAssistant.swift, ShellModel+GrokAssistant.swift,
/// GrokAssistantView.swift): the request shape, the coverage rule, the whole
/// pipeline through the helper/session doubles (`Fixtures/fake_assistant_context.py`,
/// `Fixtures/fake_grok_session.py`), Apply as one undoable edit in the real
/// editor, cancel, and the no-key / refused paths. No network, no key value.
@MainActor
final class GrokAssistantTests: XCTestCase {
    static let fixtures = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("Fixtures")
    static let fakeSession = fixtures.appendingPathComponent("fake_grok_session.py")
    static let fakeHelper = fixtures.appendingPathComponent("fake_assistant_context.py")

    static let text = "\\documentclass{article}\n\\begin{document}\nHello $x^2$ world.\n\\bad{one}\nTail line.\n\\end{document}\n"

    static func result(diagnostics: [RuntimeV1.Diagnostic] = [
        .init(severity: .error, message: "Undefined control sequence \\bad", source: .init(path: "main.tex", startByte: 60, endByte: 64), recovery: nil),
        .init(severity: .warning, message: "Overfull box", source: .init(path: "main.tex", startByte: 41, endByte: 46), recovery: nil),
        .init(severity: .warning, message: "Elsewhere", source: .init(path: "other.tex", startByte: 0, endByte: 1), recovery: nil),
    ]) -> RuntimeV1.CompileResult {
        .init(projectId: "demo", revision: 3, status: .recovered, pages: [], diagnostics: diagnostics, pdfPath: nil)
    }

    static func snapshot(text: String = text, selection: Range<Int>? = nil, caret: Int? = 0, active: String = text,
                         pinned: Int? = nil) -> GrokAssistant.Snapshot {
        .init(resultID: "mac-7", result: result(), compiledDocuments: ["main.tex": text, "other.tex": "x\n"], activePath: "main.tex",
              activeText: active, editorRevision: 3, selection: selection, caretByte: caret, pinnedDiagnostic: pinned)
    }

    // MARK: request shape (pure)

    func testSelectionShapesRelatedSelectedAndInstruction() throws {
        let bad = try XCTUnwrap(Self.text.range(of: "\\bad{one}"))
        XCTAssertEqual(Self.text.utf8.distance(from: Self.text.utf8.startIndex, to: bad.lowerBound), 60)
        XCTAssertEqual(Self.text.utf8.count, 96)
        let sel = 41..<69 // "Hello $x^2$ world.\n\\bad{one}"
        let r = try XCTUnwrap(try? GrokAssistant.makeRequest(snapshot: Self.snapshot(selection: sel), instruction: "  fix the error on this line ").get())
        XCTAssertEqual(r.requestId, "mac-7")
        XCTAssertEqual(r.projectId, "demo")
        XCTAssertEqual(r.compileRevision, 3)
        XCTAssertEqual(r.relatedPaths, ["main.tex"], "the whole live document is the related snippet")
        XCTAssertEqual(r.selectedDiagnostics, [0, 1], "the two diagnostics intersecting the selection, never other.tex's")
        XCTAssertTrue(r.userInstruction.hasPrefix("The user selected bytes 41..<69 (lines 3–4) of main.tex (editor revision 3). Propose edits only inside the selection."), r.userInstruction)
        XCTAssertTrue(r.userInstruction.hasSuffix("\nRequest: fix the error on this line"), "trimmed and framed: \(r.userInstruction)")
        XCTAssertEqual(r.contextLine, "Selection: lines 3–4 of main.tex")
        XCTAssertEqual(r.diagnosticsLine, "2 diagnostics in range")

        // Only the error's line.
        let one = try XCTUnwrap(try? GrokAssistant.makeRequest(snapshot: Self.snapshot(selection: 60..<69), instruction: "x").get())
        XCTAssertEqual(one.selectedDiagnostics, [0])
        XCTAssertEqual(one.contextLine, "Selection: line 4 of main.tex")
        XCTAssertEqual(one.diagnosticsLine, "1 diagnostic in range")

        // No selection: whole document, no diagnostics, caret in the frame.
        let whole = try XCTUnwrap(try? GrokAssistant.makeRequest(snapshot: Self.snapshot(caret: 45), instruction: "write the TikZ for a right triangle").get())
        XCTAssertEqual(whole.selectedDiagnostics, [])
        XCTAssertEqual(whole.relatedPaths, ["main.tex"])
        XCTAssertEqual(whole.contextLine, "Whole document: main.tex")
        XCTAssertEqual(whole.diagnosticsLine, "2 diagnostics in the document (select a range to include them)")
        XCTAssertTrue(whole.userInstruction.contains("with no selection. The caret is at byte 45 (line 3)."), whole.userInstruction)
        // An empty selection is a caret.
        XCTAssertEqual(try? GrokAssistant.makeRequest(snapshot: Self.snapshot(selection: 45..<45, caret: 45), instruction: "x").get().contextLine, "Whole document: main.tex")

        // Fix with Grok pins its diagnostic even without a selection.
        let pinned = try XCTUnwrap(try? GrokAssistant.makeRequest(snapshot: Self.snapshot(pinned: 0), instruction: "Fix this: Undefined control sequence").get())
        XCTAssertEqual(pinned.selectedDiagnostics, [0])
        XCTAssertEqual(pinned.diagnosticsLine, "1 diagnostic pinned")

        // Bounded by the helper's 8 KiB instruction limit (UTF-8 safe).
        let long = String(repeating: "é", count: 6000)
        let bounded = try XCTUnwrap(try? GrokAssistant.makeRequest(snapshot: Self.snapshot(), instruction: long).get())
        XCTAssertLessThanOrEqual(bounded.userInstruction.utf8.count, GrokAssistant.maxInstructionBytes)
        XCTAssertGreaterThan(bounded.userInstruction.utf8.count, GrokAssistant.maxInstructionBytes - 4)

        // Refusals: empty instruction, buffer ahead of the compile, document not compiled.
        guard case .failure(let empty) = GrokAssistant.makeRequest(snapshot: Self.snapshot(), instruction: "  \n") else { return XCTFail() }
        XCTAssertEqual(empty.message, "type what Grok should do first")
        guard case .failure(let stale) = GrokAssistant.makeRequest(snapshot: Self.snapshot(active: Self.text + "typed"), instruction: "x") else { return XCTFail() }
        XCTAssertTrue(stale.message.hasPrefix("the buffer changed since the last compile (revision 3)"), stale.message)
        var other = Self.snapshot(); other.activePath = "new.tex"; other.activeText = ""
        guard case .failure(let missing) = GrokAssistant.makeRequest(snapshot: other, instruction: "x") else { return XCTFail() }
        XCTAssertEqual(missing.message, "new.tex was not part of the last compile; compile first (⌘B)")
    }

    func testDestinationsClipTheSelectionToSnippetsAndSayWhenTheyCannot() {
        typealias R = ProposalPreview.ByteRange
        let head = R(path: "main.tex", startByte: 0, endByte: 2048)
        let around = R(path: "main.tex", startByte: 4000, endByte: 6048)
        // No selection: every snippet is a destination (explicit boundary).
        XCTAssertEqual(GrokAssistant.destinations(snippets: [head, around], selection: nil), [head, around])
        XCTAssertNil(GrokAssistant.coverageNote(snippets: [head], selection: nil, destinations: [head], path: "main.tex"))
        // Inside the head: the selection itself, no note.
        XCTAssertEqual(GrokAssistant.destinations(snippets: [head], selection: 41..<70), [R(path: "main.tex", startByte: 41, endByte: 70)])
        XCTAssertNil(GrokAssistant.coverageNote(snippets: [head], selection: 41..<70, destinations: [R(path: "main.tex", startByte: 41, endByte: 70)], path: "main.tex"))
        // Straddling the head's end: clipped, and the note says so.
        let clipped = GrokAssistant.destinations(snippets: [head], selection: 2000..<2100)
        XCTAssertEqual(clipped, [R(path: "main.tex", startByte: 2000, endByte: 2048)])
        let note = GrokAssistant.coverageNote(snippets: [head], selection: 2000..<2100, destinations: clipped, path: "main.tex")
        XCTAssertEqual(note, "The selection (bytes 2000..<2100) is not fully inside the helper's context window of main.tex (bytes 0..<2048: the first 2 KiB of the document plus 2 KiB around each selected diagnostic); edits are limited to bytes 2000..<2048.")
        // Outside every snippet: explanation only, said plainly.
        XCTAssertEqual(GrokAssistant.destinations(snippets: [head, around], selection: 3000..<3100), [])
        XCTAssertEqual(GrokAssistant.coverageNote(snippets: [head, around], selection: 3000..<3100, destinations: [], path: "main.tex"),
                       "The selection (bytes 3000..<3100) is not fully inside the helper's context window of main.tex (bytes 0..<2048, bytes 4000..<6048: the first 2 KiB of the document plus 2 KiB around each selected diagnostic); Grok can explain but cannot propose edits there.")
        // Two overlapping snippets: one destination per snippet, never merged across them, at most 8.
        let many = (0..<10).map { R(path: "main.tex", startByte: $0 * 100, endByte: $0 * 100 + 150) }
        XCTAssertEqual(GrokAssistant.destinations(snippets: many, selection: 0..<5000).count, 8)
        XCTAssertEqual(GrokAssistant.destinations(snippets: [head, R(path: "main.tex", startByte: 1000, endByte: 3000)], selection: 1500..<2500),
                       [R(path: "main.tex", startByte: 1500, endByte: 2048), R(path: "main.tex", startByte: 1500, endByte: 2500)])
    }

    // MARK: the pipeline through the doubles

    private func configuration(key: Bool = true, grok: Bool = true, providerTimeout: TimeInterval = 5) -> ProposalPreview.ExplanationConfiguration {
        var c = ProposalPreview.ExplanationConfiguration(helper: WorkerClientTests.python, helperArguments: [Self.fakeHelper.path])
        c.helperTimeout = 5
        c.providerTimeout = providerTimeout
        if grok {
            c.grok = .init(model: "grok-fast-test",
                           credential: key ? GrokCredential.resolve(environment: ["FLASHTEX_KEYCHAIN_OFF": "1", "XAI_API_KEY": "fixture-key"], keychain: MemoryKeychain()) : nil,
                           helper: Self.fakeSession)
        }
        return c
    }

    /// A shell with one compiled document (the binding) and the assistant on the doubles.
    private func shell(text: String = text, configuration: ProposalPreview.ExplanationConfiguration? = nil) -> ShellModel {
        let model = ShellModel()
        model.autoCompile = false
        model.replaceProject(entryText: text)
        model.result = Self.result()
        model.resultID = "mac-7"
        model.setCompiledDocuments(["main.tex": text, "other.tex": "x\n"])
        model.grokAssistant.injected = configuration ?? self.configuration()
        return model
    }

    private func select(_ model: ShellModel, bytes: Range<Int>) {
        let ns = model.activeText.nsRange(utf8Bytes: .init(path: "main.tex", startByte: bytes.lowerBound, endByte: bytes.upperBound))!
        model.caretUTF16 = ns.location
        model.caretLengthUTF16 = ns.length
    }

    private func waitUntil(_ what: String, timeout: TimeInterval = 10, _ cond: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if cond() { return }
            try await Task.sleep(nanoseconds: 20_000_000)
        }
        XCTFail("timed out waiting for \(what)")
    }

    private func waitSettled(_ a: GrokAssistant) async throws {
        try await waitUntil("assistant settled", timeout: 15) { !a.inFlight }
    }

    func testAskRunsProbePrepareSessionReviewAndShowsTheDiff() async throws {
        let model = shell()
        let a = model.grokAssistant
        XCTAssertFalse(a.canAsk, "no instruction yet")
        a.instruction = "convert this to an align environment"
        XCTAssertTrue(a.canAsk)
        select(model, bytes: 41..<59) // "Hello $x^2$ world."
        model.askGrokNow()
        XCTAssertEqual(a.state, .preparing)
        XCTAssertEqual(a.contextLine, "Selection: line 3 of main.tex")
        XCTAssertEqual(a.diagnosticsLine, "1 diagnostic in range")
        try await waitUntil("awaiting provider") { if case .awaitingProvider = a.state { return true }; return false }
        XCTAssertTrue(a.statusText.hasPrefix("Asking Grok (grok-fast-test)… up to 5 s"), a.statusText)
        XCTAssertNotNil(a.providerStartedAt)
        XCTAssertNotNil(a.runningChildProcessIdentifier)
        try await waitSettled(a)
        guard case .ready(let reply) = a.state else { return XCTFail("\(a.state)") }
        XCTAssertTrue(reply.text.hasPrefix("Fake Grok (grok-fast-test) explanation of 1 diagnostic(s)"), reply.text)
        XCTAssertEqual(reply.edits.count, 1)
        XCTAssertEqual(reply.edits[0].range, .init(path: "main.tex", startByte: 41, endByte: 59), "the edit lies inside the selection")
        XCTAssertEqual(reply.edits[0].removedText, "Hello $x^2$ world.")
        XCTAssertEqual(reply.edits[0].replacement, "% grok-reviewed: Hello $x^2$ world.")
        XCTAssertFalse(reply.edits[0].relocated)
        XCTAssertNotNil(reply.reviewId)
        XCTAssertEqual(reply.model, "grok-fast-test")
        XCTAssertNil(a.coverageNote, "the selection sits inside the document head snippet")
        XCTAssertEqual(a.childLaunches.map(\.stage), [.probe, .prepare, .provider, .review])
        XCTAssertEqual(a.childLaunches.map(\.role), [.helper, .helper, .grok, .helper])
        for l in a.childLaunches where l.role == .helper {
            XCTAssertFalse(l.environmentKeys.contains("FLASHTEX_GROK_API_KEY"), "\(l.stage)")
        }
        let grokLaunch = try XCTUnwrap(a.lastGrokLaunch)
        XCTAssertEqual(grokLaunch.model, "grok-fast-test")
        XCTAssertTrue(grokLaunch.sessionId.hasPrefix("mac-ask-1-"), grokLaunch.sessionId)
        XCTAssertTrue(grokLaunch.environmentKeys.contains("FLASHTEX_GROK_API_KEY"))
        // Exactly what was sent: the live document as related, the one diagnostic, the framed instruction.
        let req = try XCTUnwrap(a.lastRequest)
        XCTAssertEqual(req.relatedPaths, ["main.tex"])
        XCTAssertEqual(req.selectedDiagnostics, [1])
        XCTAssertTrue(req.userInstruction.hasSuffix("Request: convert this to an align environment"))
        // The before/after diff of the live buffer, and nothing written.
        let preview = try XCTUnwrap(a.applicationPreview)
        XCTAssertEqual(preview.before, "Hello $x^2$ world.")
        XCTAssertEqual(preview.after, "% grok-reviewed: Hello $x^2$ world.")
        XCTAssertNil(a.applicationRefusal)
        XCTAssertTrue(a.canApply)
        XCTAssertEqual(model.activeText, Self.text, "nothing is written before Apply")
        XCTAssertNil(model.pendingEdit)
        XCTAssertTrue(a.statusText.contains("answered in") && a.statusText.contains("1 proposed edit; nothing applied — Apply inserts"), a.statusText)
    }

    func testNoSelectionBoundsEditsToEverySnippetAndExplanationOnlyRepliesHaveNoApply() async throws {
        let model = shell(text: Self.text + "%groknoedit\n")
        let a = model.grokAssistant
        a.instruction = "explain this document"
        model.askGrokNow()
        try await waitSettled(a)
        guard case .ready(let reply) = a.state else { return XCTFail("\(a.state)") }
        XCTAssertEqual(reply.edits, [])
        XCTAssertNil(reply.reviewId)
        XCTAssertFalse(a.canApply)
        XCTAssertNil(a.applicationPreview)
        XCTAssertEqual(a.childLaunches.map(\.stage), [.probe, .prepare, .provider, .validate], "explanation-only replies go through validate")
        XCTAssertEqual(a.contextLine, "Whole document: main.tex")
        XCTAssertEqual(a.lastRequest?.selectedDiagnostics, [])
        let destinations = try XCTUnwrap(a.job?.helperRequest["destinations"] as? [[String: Any]])
        XCTAssertEqual(destinations.count, 1, "the document head is the only snippet without diagnostics")
        XCTAssertEqual(destinations[0]["start_byte"] as? Int, 0)
        XCTAssertEqual(destinations[0]["end_byte"] as? Int, (Self.text + "%groknoedit\n").utf8.count)
    }

    func testSelectionOutsideTheContextWindowIsSaidNotTruncated() async throws {
        // A 3 KiB document whose selection lies past the 2 KiB head, with no diagnostic near it.
        let filler = String(repeating: "filler line of text here\n", count: 120) // 3000 bytes
        let text = Self.text + filler
        let model = shell(text: text)
        model.result = Self.result(diagnostics: [])
        let a = model.grokAssistant
        a.instruction = "rewrite this paragraph"
        select(model, bytes: 2500..<2600)
        model.askGrokNow()
        try await waitSettled(a)
        guard case .ready(let reply) = a.state else { return XCTFail("\(a.state)") }
        XCTAssertEqual(reply.edits, [], "no destination: the double proposes no edit")
        let note = try XCTUnwrap(a.coverageNote)
        XCTAssertTrue(note.hasPrefix("The selection (bytes 2500..<2600) is not fully inside the helper's context window of main.tex (bytes 0..<2048:"), note)
        XCTAssertTrue(note.hasSuffix("Grok can explain but cannot propose edits there."), note)
        XCTAssertEqual((a.job?.helperRequest["destinations"] as? [[String: Any]])?.count, 0, "explanation only")
    }

    /// Apply: helper `approve` of the exact review → one `pendingEdit` → the real
    /// editor applies it as one undo step → ⌘Z restores the text and the model.
    func testApplyIsOneUndoableEditInTheRealEditor() async throws {
        let model = shell()
        let a = model.grokAssistant
        let probe = Probe()
        let (window, tv) = try await host(model, probe: probe)
        defer { window.orderOut(nil) }
        let undo = try XCTUnwrap(tv.undoManager)
        a.instruction = "prefix it"
        select(model, bytes: 60..<69) // "\\bad{one}"
        model.askGrokNow()
        try await waitSettled(a)
        guard case .ready(let reply) = a.state, let reviewId = reply.reviewId else { return XCTFail("\(a.state)") }
        XCTAssertEqual(model.activeText, Self.text)
        let base = model.editorRevision
        model.applyGrokEdit()
        guard case .approving = a.state else { return XCTFail("\(a.state)") }
        try await waitSettled(a)
        guard case .applied(let note) = a.state else { return XCTFail("\(a.state)") }
        XCTAssertTrue(note.hasPrefix("Applied review \(reviewId.prefix(8))"), note)
        XCTAssertEqual(a.childLaunches.last?.stage, .approve)
        let expected = Self.text.replacingOccurrences(of: "\\bad{one}", with: "% grok-reviewed: \\bad{one}")
        try await waitUntil("editor applied the edit") { tv.string == expected }
        try await waitUntil("model told once") { probe.editApplied.count == 1 }
        XCTAssertEqual(model.activeText, expected)
        XCTAssertEqual(model.editorRevision, base + 1, "one revision bump for the whole edit")
        XCTAssertNil(model.pendingEdit)
        XCTAssertEqual(model.navigationNote, "Applied Grok's edit (undo with ⌘Z)")
        XCTAssertEqual(probe.editApplied.count, 1)
        XCTAssertTrue(undo.canUndo)
        undo.undo()
        XCTAssertEqual(tv.string, Self.text, "one undo step restores the document")
        XCTAssertEqual(model.activeText, Self.text)
        undo.redo()
        XCTAssertEqual(tv.string, expected)
        XCTAssertEqual(model.activeText, expected)
        XCTAssertEqual(probe.editApplied.count, 1, "undo/redo never re-deliver the edit")
        XCTAssertFalse(a.canApply, "the reviewed edit was consumed")
    }

    func testApplyAfterTypingRebasesOrRefusesNeverGuesses() async throws {
        let model = shell()
        let a = model.grokAssistant
        a.instruction = "prefix it"
        select(model, bytes: 60..<69)
        model.askGrokNow()
        try await waitSettled(a)
        guard case .ready = a.state else { return XCTFail("\(a.state)") }
        // Typing elsewhere (before the edit) rebases the grouped edit byte-exactly.
        model.updateActiveText("% note\n" + Self.text)
        model.applyGrokEdit()
        try await waitSettled(a)
        guard case .applied = a.state else { return XCTFail("\(a.state)") }
        let edit = try XCTUnwrap(model.pendingEdit)
        XCTAssertEqual(edit.text, "% grok-reviewed: \\bad{one}")
        XCTAssertEqual(edit.nsRange, NSRange(location: 60 + 7, length: 9), "rebased across the 7 inserted bytes")
        XCTAssertEqual(edit.revision, model.editorRevision)
        // Typing INSIDE the edited bytes refuses the application; the text is untouched.
        let again = shell()
        let b = again.grokAssistant
        b.instruction = "prefix it"
        select(again, bytes: 60..<69)
        again.askGrokNow()
        try await waitSettled(b)
        guard case .ready = b.state else { return XCTFail("\(b.state)") }
        again.updateActiveText(Self.text.replacingOccurrences(of: "\\bad{one}", with: "\\bad{two}"))
        again.applyGrokEdit()
        guard case .failed(let why) = b.state else { return XCTFail("\(b.state)") }
        XCTAssertTrue(why.hasPrefix("the edit no longer fits the buffer:"), why)
        XCTAssertNil(again.pendingEdit)
        XCTAssertTrue(b.childLaunches.allSatisfy { $0.stage != .approve }, "no approval was requested for a stale edit")
    }

    func testCancelTerminatesTheSessionAndDropsTheLateReply() async throws {
        let model = shell(text: Self.text + "%grokslow\n")
        let a = model.grokAssistant
        a.instruction = "anything"
        model.askGrokNow()
        try await waitUntil("session running") { if case .awaitingProvider = a.state { return a.runningChildProcessIdentifier != nil }; return false }
        let pid = try XCTUnwrap(a.runningChildProcessIdentifier)
        let stale = a.staleReplies
        a.cancel()
        XCTAssertEqual(a.state, .cancelled)
        XCTAssertNil(a.providerStartedAt)
        XCTAssertFalse(a.inFlight)
        try await waitUntil("late reply discarded") { a.staleReplies == stale + 1 }
        try await waitUntil("child gone") { kill(pid, 0) != 0 }
        XCTAssertEqual(a.state, .cancelled)
        XCTAssertEqual(model.activeText, Self.text + "%grokslow\n")
    }

    func testNoKeyRefusedAndHelperErrorsAreShownVerbatim() async throws {
        // Grok selected, no key: the context is prepared, nothing is sent, Preferences is named.
        let model = shell(configuration: configuration(key: false))
        let a = model.grokAssistant
        a.instruction = "anything"
        model.askGrokNow()
        try await waitSettled(a)
        guard case .failed(let noKey) = a.state else { return XCTFail("\(a.state)") }
        XCTAssertTrue(noKey.hasPrefix("Grok (xAI) is selected but no API key is present — add one in Preferences (⌘,)"), noKey)
        XCTAssertTrue(noKey.hasSuffix("sent nowhere"), noKey)
        XCTAssertEqual(a.childLaunches.map(\.stage), [.probe, .prepare], "no provider launch")
        XCTAssertNil(a.lastGrokLaunch)
        XCTAssertEqual(a.providerStatusText, "Grok: off")

        // The provider's reply with misplaced offsets: the (pre-relocation) helper double
        // refuses the review; the refusal reaches the panel verbatim.
        let misplaced = shell(text: Self.text + "%grokmisplaced\n")
        let b = misplaced.grokAssistant
        b.instruction = "fix"
        select(misplaced, bytes: 60..<69)
        misplaced.askGrokNow()
        try await waitSettled(b)
        guard case .failed(let refused) = b.state else { return XCTFail("\(b.state)") }
        XCTAssertEqual(refused, "helper refused (review): proposed edit is outside explicit destination", "the double checks the destination before the removed text; either refusal is shown as the helper wrote it")
        XCTAssertEqual(misplaced.activeText, Self.text + "%grokmisplaced\n")

        // The session reports failure (401/403/429 collapsed by the helper): said so.
        let failing = shell(text: Self.text + "%grokfail\n")
        let c = failing.grokAssistant
        c.instruction = "fix"
        failing.askGrokNow()
        try await waitSettled(c)
        guard case .failed(let failed) = c.state else { return XCTFail("\(c.state)") }
        XCTAssertTrue(failed.hasPrefix("provider refused (provider): Grok request failed (helper state \"failed\""), failed)

        // A helper exit without output names the exit code and stage.
        let crashing = shell(text: Self.text + "%crashexplain\n")
        let d = crashing.grokAssistant
        d.instruction = "fix"
        crashing.askGrokNow()
        try await waitSettled(d)
        guard case .failed(let crashed) = d.state else { return XCTFail("\(d.state)") }
        XCTAssertEqual(crashed, "helper exited (3) during probe")

        // No provider at all: the context is prepared and shown as sent nowhere.
        let offline = shell(configuration: configuration(grok: false))
        let e = offline.grokAssistant
        e.instruction = "explain"
        offline.askGrokNow()
        try await waitSettled(e)
        guard case .prepared(let id, let bytes) = e.state else { return XCTFail("\(e.state)") }
        XCTAssertEqual(id.count, 64)
        XCTAssertGreaterThan(bytes, 100)
        XCTAssertTrue(e.statusText.contains("No provider is enabled — nothing was sent."), e.statusText)

        // No compile result: refused before any launch.
        let fresh = ShellModel()
        fresh.result = nil
        fresh.resultID = nil
        fresh.grokAssistant.injected = configuration()
        fresh.grokAssistant.instruction = "x"
        fresh.askGrokNow()
        guard case .failed(let none) = fresh.grokAssistant.state else { return XCTFail("\(fresh.grokAssistant.state)") }
        XCTAssertTrue(none.hasPrefix("no compile result to bind to"), none)
        XCTAssertTrue(fresh.grokAssistant.childLaunches.isEmpty)
    }

    func testFixWithGrokSelectsTheSpanAndPrefillsTheInstruction() async throws {
        let model = shell()
        let a = model.grokAssistant
        model.fixWithGrok(diagnosticIndex: 0)
        XCTAssertTrue(a.shown)
        XCTAssertEqual(a.instruction, "Fix this: Undefined control sequence \\bad")
        XCTAssertEqual(a.pinnedDiagnostic, 0)
        XCTAssertEqual(model.selection?.nsRange, NSRange(location: 60, length: 4))
        XCTAssertEqual(model.caretUTF16, 60)
        XCTAssertEqual(model.caretLengthUTF16, 4)
        let snapshot = try XCTUnwrap(model.grokSnapshot())
        XCTAssertEqual(snapshot.selection, 60..<64)
        model.askGrokNow()
        try await waitSettled(a)
        guard case .ready(let reply) = a.state else { return XCTFail("\(a.state)") }
        XCTAssertEqual(a.lastRequest?.selectedDiagnostics, [0])
        XCTAssertEqual(reply.edits.first?.removedText, "\\bad")
        // Opening the panel plainly afterwards keeps the text but drops the pin.
        model.askGrok()
        XCTAssertNil(a.pinnedDiagnostic)
        XCTAssertEqual(a.instruction, "Fix this: Undefined control sequence \\bad")
        model.fixWithGrok(diagnosticIndex: 9)
        XCTAssertEqual(model.navigationNote, "No diagnostic 10 in the current result.")
    }

    func testAskModelAndConfigurationDefaults() {
        let prefs = GrokPreferences(defaults: UserDefaults(suiteName: "GrokAssistantTests.\(UUID())")!)
        XCTAssertEqual(GrokCredential.askModel(environment: [:], preferences: prefs), GrokCredential.fastModel, "Ask defaults to the fast model")
        XCTAssertEqual(GrokCredential.model(environment: [:], preferences: prefs), GrokCredential.reasoningModel, "the review sheet keeps its default")
        XCTAssertEqual(GrokCredential.askModel(environment: ["FLASHTEX_GROK_MODEL": "grok-4.6"], preferences: prefs), "grok-4.6")
        XCTAssertEqual(GrokCredential.askModel(environment: ["FLASHTEX_GROK_MODEL": "grok-4.6", "FLASHTEX_GROK_ASK_MODEL": "grok-x"], preferences: prefs), "grok-x")
        prefs.model = "grok-custom"
        XCTAssertEqual(GrokCredential.askModel(environment: [:], preferences: prefs), "grok-custom")
        XCTAssertEqual(GrokCredential.askModel(environment: ["FLASHTEX_GROK_ASK_MODEL": "bad id!"], preferences: prefs), "grok-custom", "invalid ids fall through")
        // The configuration carries the Ask model and its bound; without a key the pill says off.
        let env = ["FLASHTEX_KEYCHAIN_OFF": "1", "FLASHTEX_ASSISTANT_PROVIDER": "grok", "XAI_API_KEY": "fixture-key", "FLASHTEX_ASSISTANT_CONTEXT": Self.fakeHelper.path]
        let c = GrokAssistant.configuration(env, preferences: GrokPreferences(defaults: UserDefaults(suiteName: "GrokAssistantTests.\(UUID())")!), keychain: MemoryKeychain())
        XCTAssertEqual(c.grok?.model, GrokCredential.fastModel)
        XCTAssertEqual(c.providerTimeout, 30)
        XCTAssertEqual(c.grokStatusText, "Grok: on (\(GrokCredential.fastModel))")
        let a = GrokAssistant(configuration: c)
        XCTAssertEqual(a.model, GrokCredential.fastModel)
        XCTAssertTrue(a.grokLive)
        XCTAssertEqual(GrokAssistant(configuration: .disabled).statusText.hasPrefix("unavailable: no flashtex-assistant-context helper"), true)
    }

    // MARK: editor host (as SourceEditorViewTests)

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

    private func host(_ model: ShellModel, probe: Probe) async throws -> (NSWindow, NSTextView) {
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 400), styleMask: [.titled],
                              backing: .buffered, defer: false)
        window.contentView = NSHostingView(rootView: Host(model: model, probe: probe))
        window.orderFrontRegardless() // never makeKey
        var found: NSTextView?
        try await waitUntil("editor text view") {
            found = TypingBenchDriver.findTextView(in: [window.contentView!]); return found != nil
        }
        let tv = try XCTUnwrap(found)
        XCTAssertTrue(window.makeFirstResponder(tv))
        return (window, tv)
    }
}
