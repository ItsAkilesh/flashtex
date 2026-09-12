import SwiftUI
import XCTest
@testable import FlashTeXProtocol
@testable import FlashTeXMac

/// Review history (ReviewHistory.swift): every proposal decision recorded
/// with its outcome and exact reason, bounded in memory, persisted atomically
/// under Application Support and reloaded on the next launch. The recorder
/// is driven exactly as the `ShellModel` hooks would drive it (see the
/// parent diff in coordination/mac-ai-review-2.md) from real model outcomes.
@MainActor
final class ReviewHistoryTests: XCTestCase {
    private var scratch: URL!

    override func setUpWithError() throws {
        scratch = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent("flashtex-review-history-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: scratch, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        if let scratch { try? FileManager.default.removeItem(at: scratch) }
    }

    func testEveryModelOutcomeIsRecordedWithItsReasonAndPersisted() throws {
        let url = scratch.appendingPathComponent("support/FlashTeX/review-history/demo.json")
        let recorder = ReviewHistoryRecorder(url: url)
        XCTAssertNil(recorder.persistenceProblem)
        XCTAssertTrue(recorder.entries.isEmpty)
        let model = ShellModel()
        XCTAssertNil(model.loadError, model.loadError ?? "")
        let p = RuntimeV1.CaptureProposal(captureId: "c1", latex: "x^2", ambiguities: [], requiredDependencies: [])
        model.enqueue(p)

        // No anchor.
        var outcome = model.approveProposal(p, latex: "x^2")
        recorder.record(outcome, proposal: p, latex: "x^2", path: model.activePath, editorRevision: model.editorRevision)
        XCTAssertEqual(recorder.entries.last?.outcome, .noAnchor)
        XCTAssertEqual(recorder.entries.last?.id, "c1#1")

        // Anchor pinned; approved (staged), then the editor refuses it: back to review with the exact reason.
        model.caretUTF16 = 5
        model.pinAnchorAtCaret()
        outcome = model.approveProposal(p, latex: " y^2 ")
        recorder.record(outcome, proposal: p, latex: " y^2 ", path: model.activePath, editorRevision: model.editorRevision)
        XCTAssertEqual(outcome, .inserted(byteOffset: 5))
        XCTAssertEqual(recorder.entries.last?.outcome, .inserted)
        XCTAssertEqual(recorder.entries.last?.byteOffset, 5)
        XCTAssertEqual(recorder.entries.last?.path, "main.tex")
        let staged = try XCTUnwrap(model.pendingEdit)
        let refund = try XCTUnwrap(staged.captureRefund)
        let reason = "the document changed since the edit was prepared (revision 1, now 2)"
        model.editRefused(staged, reason: reason)
        recorder.recordRefused(refund, reason: reason, editorRevision: model.editorRevision)
        XCTAssertEqual(recorder.entries.last?.outcome, .refused)
        XCTAssertEqual(recorder.entries.last?.reason, reason)
        XCTAssertEqual(recorder.entries.last?.latex, " y^2 ", "the reviewer's text at decision time")
        XCTAssertEqual(model.proposals.map(\.captureId), ["c1"], "the model put it back; the history only records that")

        // Approved again and applied by the editor.
        let again = try XCTUnwrap(model.reviewing)
        outcome = model.approveProposal(again, latex: again.latex)
        recorder.record(outcome, proposal: again, latex: again.latex, path: model.activePath, editorRevision: model.editorRevision)
        let edit = try XCTUnwrap(model.pendingEdit)
        model.editApplied(edit, newText: "Hello\ny^2\n FlashTeX.\n")
        XCTAssertNotNil(recorder.recordApplied(edit, editorRevision: model.editorRevision))
        XCTAssertEqual(recorder.entries.last?.outcome, .applied)
        XCTAssertEqual(recorder.entries.last?.editorRevision, model.editorRevision)
        XCTAssertNil(recorder.recordApplied(.init(path: "main.tex", nsRange: NSRange(location: 0, length: 0), text: "", token: 99),
                                            editorRevision: 1), "a non-capture edit is not a review decision")

        // Duplicate, rejected, reselection and declined outcomes.
        model.enqueue(p)
        outcome = model.approveProposal(p, latex: "again")
        recorder.record(outcome, proposal: p, latex: "again", path: model.activePath, editorRevision: model.editorRevision)
        XCTAssertEqual(recorder.entries.last?.outcome, .duplicate)
        let p2 = RuntimeV1.CaptureProposal(captureId: "c2", latex: "z", ambiguities: [], requiredDependencies: [])
        model.enqueue(p2)
        model.rejectProposal(p2)
        recorder.recordRejected(p2, editorRevision: model.editorRevision)
        XCTAssertEqual(recorder.entries.last?.outcome, .rejected)
        XCTAssertEqual(recorder.entries.last?.captureId, "c2")
        recorder.record(.needsReselection("destination text was deleted or changed"), proposal: p2, latex: "z", path: "main.tex", editorRevision: 3)
        XCTAssertEqual(recorder.entries.last?.outcome, .needsReselection)
        XCTAssertEqual(recorder.entries.last?.reason, "destination text was deleted or changed")
        recorder.record(.refused("bridge capture"), proposal: p2, latex: "z", path: "main.tex", editorRevision: 3)
        XCTAssertEqual(recorder.entries.last?.outcome, .declined)
        XCTAssertEqual(recorder.entries.last?.reason, "bridge capture")

        XCTAssertEqual(recorder.entries.map(\.outcome), [.noAnchor, .inserted, .refused, .inserted, .applied, .duplicate, .rejected, .needsReselection, .declined])
        XCTAssertEqual(recorder.history.entries(for: "c1").count, 6)
        XCTAssertEqual(recorder.saveCount, 9, "every decision was persisted")
        XCTAssertNil(recorder.persistenceProblem)
        XCTAssertEqual(recorder.history.newestFirst.first?.id, "c2#9")

        // A new recorder on the same file (the next launch) sees the same list.
        let reloaded = ReviewHistoryRecorder(url: url)
        XCTAssertNil(reloaded.persistenceProblem)
        XCTAssertEqual(reloaded.history, recorder.history)
        XCTAssertEqual(reloaded.entries.count, 9)
        // ...and continues numbering after it.
        reloaded.recordRejected(p2, editorRevision: 4)
        XCTAssertEqual(reloaded.entries.last?.id, "c2#10")
        // The file is plain JSON with ISO-8601 dates and sorted keys (diffable).
        let text = try String(contentsOf: url, encoding: .utf8)
        XCTAssertTrue(text.contains("\"version\":1"), text.prefix(200).description)
        XCTAssertTrue(text.contains("\"outcome\":\"refused\""))
        XCTAssertTrue(text.contains("\"recordedAt\":\"20"))
        // Every outcome label is non-empty and the view renders the list.
        for o in ReviewHistoryEntry.Outcome.allCases { XCTAssertFalse(o.label.isEmpty) }
        let renderer = ImageRenderer(content: ReviewHistoryView(recorder: reloaded).frame(width: 480).padding(8))
        XCTAssertNotNil(renderer.nsImage)
    }

    func testHistoryIsBoundedClipsLongLatexAndSurvivesCorruptOrForeignFiles() throws {
        var h = ReviewHistory()
        for i in 0..<(ReviewHistory.capacity + 7) {
            _ = h.record(captureId: "c\(i)", latex: "l\(i)", outcome: .rejected, editorRevision: i)
        }
        XCTAssertEqual(h.count, ReviewHistory.capacity)
        XCTAssertEqual(h.entries.first?.captureId, "c7", "oldest dropped")
        XCTAssertEqual(h.last?.id, "c\(ReviewHistory.capacity + 6)#\(ReviewHistory.capacity + 7)")

        // Long LaTeX is clipped to the byte limit on a scalar boundary and flagged.
        let long = String(repeating: "é", count: ReviewHistory.latexLimit) // 2 bytes each
        let e = h.record(captureId: "big", latex: long, outcome: .inserted, editorRevision: 1)
        XCTAssertTrue(e.latexClipped)
        XCTAssertEqual(e.latex.utf8.count, ReviewHistory.latexLimit)
        XCTAssertEqual(e.latex, String(repeating: "é", count: ReviewHistory.latexLimit / 2))
        let short = h.record(captureId: "small", latex: "x", outcome: .inserted, editorRevision: 1)
        XCTAssertFalse(short.latexClipped)

        // Round trip through the file, then corrupt and foreign-schema files
        // start empty with a problem note and the file left in place.
        let url = scratch.appendingPathComponent("h.json")
        try h.save(to: url)
        let (back, problem) = ReviewHistory.load(from: url)
        XCTAssertNil(problem)
        XCTAssertEqual(back.count, ReviewHistory.capacity)
        XCTAssertEqual(back.last?.latex, "x")
        try Data("{not json".utf8).write(to: url)
        let corrupt = ReviewHistoryRecorder(url: url)
        XCTAssertTrue(corrupt.entries.isEmpty)
        XCTAssertTrue(try XCTUnwrap(corrupt.persistenceProblem).contains("unreadable"))
        XCTAssertEqual(try String(contentsOf: url, encoding: .utf8), "{not json", "the unreadable file is kept, never overwritten on load")
        try Data("{\"version\":99,\"entries\":[],\"nextNumber\":1}".utf8).write(to: url)
        let foreign = ReviewHistoryRecorder(url: url)
        XCTAssertTrue(try XCTUnwrap(foreign.persistenceProblem).contains("schema 99"))
        // Recording after a load problem saves a fresh history (the decision is never lost).
        foreign.recordRejected(.init(captureId: "n", latex: "n", ambiguities: [], requiredDependencies: []), editorRevision: 1)
        XCTAssertEqual(ReviewHistory.load(from: url).history.count, 1)

        // Missing file = empty, no problem; in-memory only when no URL; unwritable path = problem, entry kept.
        XCTAssertNil(ReviewHistory.load(from: scratch.appendingPathComponent("missing.json")).problem)
        let memory = ReviewHistoryRecorder(url: nil)
        memory.recordRejected(.init(captureId: "m", latex: "m", ambiguities: [], requiredDependencies: []), editorRevision: 1)
        XCTAssertEqual(memory.entries.count, 1)
        XCTAssertEqual(memory.saveCount, 0)
        let blocked = scratch.appendingPathComponent("file-not-dir")
        try Data("x".utf8).write(to: blocked)
        let unwritable = ReviewHistoryRecorder(url: blocked.appendingPathComponent("h.json"))
        unwritable.recordRejected(.init(captureId: "u", latex: "u", ambiguities: [], requiredDependencies: []), editorRevision: 1)
        XCTAssertEqual(unwritable.entries.count, 1)
        XCTAssertTrue(try XCTUnwrap(unwritable.persistenceProblem).contains("could not save"))

        // Default location under Application Support, project id sanitized.
        let support = URL(fileURLWithPath: "/tmp/support")
        XCTAssertEqual(ReviewHistory.defaultURL(projectId: "my project/1", applicationSupport: support, environment: [:])?.path,
                       "/tmp/support/FlashTeX/review-history/my_project_1.json")
        XCTAssertEqual(ReviewHistory.defaultURL(projectId: "", applicationSupport: support, environment: [:])?.lastPathComponent, "project.json")
        XCTAssertNil(ReviewHistory.defaultURL(projectId: "x", applicationSupport: nil, environment: [:]))
        XCTAssertTrue(ReviewHistory.defaultURL(projectId: "demo", environment: [:])!.path.contains("/Library/Application Support/FlashTeX/review-history/demo.json"))
        // The environment override (test runs, sandboxes) and the in-memory switch.
        XCTAssertEqual(ReviewHistory.defaultURL(projectId: "demo", applicationSupport: support, environment: ["FLASHTEX_REVIEW_HISTORY_DIR": "/tmp/override"])?.path,
                       "/tmp/override/FlashTeX/review-history/demo.json")
        XCTAssertNil(ReviewHistory.defaultURL(projectId: "demo", applicationSupport: support, environment: ["FLASHTEX_REVIEW_HISTORY_DIR": "off"]))
    }
}
