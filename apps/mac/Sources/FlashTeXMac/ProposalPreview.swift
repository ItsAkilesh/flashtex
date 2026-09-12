import AppKit
import Combine
import Foundation
import PDFKit
import SwiftUI
import FlashTeXProtocol

/// Shadow compile of a capture proposal in document context (transfer-v1:
/// "Show the proposed edit and resulting diagnostics before approval").
///
/// The review sheet owns one of these. It builds the document set WITH the
/// proposed insertion applied at the pinned anchor (same rules as approval:
/// `Insertion.resolve` + `Insertion.insertionText`), compiles it on a SEPARATE
/// worker process (same compiler executable, request ids `preview-N`, project
/// id `<project>-preview`), and compiles the UNMODIFIED documents once per
/// document revision as a baseline so "new diagnostics introduced by this
/// insertion" is a real diff rather than a guess. Nothing here ever writes to
/// the editor buffer or to `ShellModel.result` / `editorRevision`.
@MainActor
final class ProposalPreview: ObservableObject {
    static let debounceInterval: TimeInterval = 0.3

    /// Snapshot of the model state the preview needs (read-only copy).
    struct Input: Equatable {
        var documents: [RuntimeV1.Document]
        var entryPath: String
        var anchor: InsertionAnchor?
        var editorRevision: Int
        var projectId: String

        @MainActor init(model: ShellModel) {
            documents = model.documents
            entryPath = model.activePath
            anchor = model.anchor
            editorRevision = model.editorRevision
            projectId = model.projectId
        }

        init(documents: [RuntimeV1.Document], entryPath: String, anchor: InsertionAnchor?,
             editorRevision: Int, projectId: String) {
            self.documents = documents; self.entryPath = entryPath; self.anchor = anchor
            self.editorRevision = editorRevision; self.projectId = projectId
        }
    }

    /// The documents with the insertion applied plus where it landed.
    struct Shadow: Equatable {
        var documents: [RuntimeV1.Document]
        var path: String
        /// Byte offset in the ORIGINAL text where the insertion text starts.
        var insertByte: Int
        /// Full insertion text (`Insertion.insertionText`), including any added newlines.
        var insertedText: String
        var insertedLength: Int { insertedText.utf8.count }
        /// Range of the inserted text in the shadow document.
        var insertedRange: Range<Int> { insertByte..<(insertByte + insertedLength) }
        /// Byte offset in the shadow document where the trimmed proposal body starts.
        var bodyStart: Int { insertByte + (insertedText.hasPrefix("\n") ? 1 : 0) }
        /// Range of the trimmed proposal body (the inserted text without the
        /// newlines `Insertion.insertionText` adds) in the shadow document. This
        /// is the only region a model may propose edits for (`explain()`).
        var bodyRange: Range<Int> { bodyStart..<(insertedRange.upperBound - (insertedText.hasSuffix("\n") ? 1 : 0)) }
        var shadowText: String { documents.first { $0.path == path }?.text ?? "" }
    }

    struct Finding: Equatable, Identifiable {
        var id: Int
        var diagnostic: RuntimeV1.Diagnostic
        /// Shadow text under the diagnostic's range (with a little context), for
        /// display: the sheet's TextEditor cannot select a range.
        var fragment: String?
        /// Offset of the diagnostic within the proposal body (UTF-8 bytes) when it
        /// lies inside the inserted text; nil when it is elsewhere.
        var proposalOffset: Int?
    }

    struct Report: Equatable {
        var status: RuntimeV1.Status
        var pageCount: Int
        /// Pages in the shadow result minus pages in the baseline (unmodified)
        /// result compiled by the same worker; nil when no baseline is available.
        var pageDelta: Int?
        /// Diagnostics whose range intersects the inserted text, its enclosing
        /// lines, or one line either side.
        var nearby: [Finding]
        /// Diagnostics not present in the baseline compile (message + range
        /// relative to the insertion).
        var new: [Finding]
        var unrelatedCount: Int
        /// Number of the shadow page whose text items map the inserted text
        /// (nil when nothing maps it, e.g. a failed compile).
        var insertionPage: Int?
        var newErrorCount: Int { new.filter { $0.diagnostic.severity == .error }.count }
    }

    enum State: Equatable {
        case idle
        case noCompiler
        case notPreviewable(String)
        case compiling
        case ready(Report)
        case failed(String)
    }

    @Published private(set) var state: State = .idle
    @Published private(set) var shadow: Shadow?
    @Published var highlightedFragment: String?
    /// Thumbnail of the shadow page containing the insertion, drawn from the
    /// shadow result with `PDFExport.render` (only that page) via PDFKit.
    @Published private(set) var thumbnail: NSImage?
    /// Requests sent so far, by kind (tests assert debounce/coalescing).
    @Published private(set) var shadowCompileCount = 0
    @Published private(set) var baselineCompileCount = 0

    let executable: URL?
    let arguments: [String]
    let explanationConfiguration: ExplanationConfiguration
    private var worker: WorkerClient?
    private var nextRequestID = 1
    private var debounce: DispatchWorkItem?
    private var latest: (input: Input, latex: String)?
    private var compiled: (input: Input, latex: String)?
    private enum Kind { case baseline, shadow }
    private struct InFlight { var kind: Kind; var projectId: String; var revision: Int; var shadow: Shadow? }
    private var inFlight: [String: InFlight] = [:]
    private var baselineKey: (documents: [RuntimeV1.Document], entryPath: String)?
    private var baseline: RuntimeV1.CompileResult?
    private var shadowResult: (shadow: Shadow, result: RuntimeV1.CompileResult, requestId: String)?

    // Assistant explanation (see the `explain()` section below). Model output
    // is displayed only; this object never writes source or the proposal text.
    @Published private(set) var explanationState: ExplanationState = .idle
    /// Explanation requests started so far (tests assert cancellation/reuse).
    @Published private(set) var explanationRequestCount = 0
    /// Helper/provider replies discarded because they answered a request that
    /// is no longer current (cancelled, superseded, or never sent).
    @Published private(set) var staleExplanationReplies = 0
    /// Every helper/provider launch so far (bounded to the last 64).
    @Published private(set) var childLaunches: [ChildLaunch] = []
    private var explanationProcess: OneShotProcess?
    private var explanationJob: ExplanationJob?
    private var explained: (input: Input, latex: String)?
    private var nextExplanationID = 1

    /// `executable == nil` means no compiler is attached: the preview reports
    /// that honestly and never launches anything. `explanation` locates the
    /// assistant-context helper and the (default: disabled) provider command;
    /// nil reads the environment (`ExplanationConfiguration.fromEnvironment`).
    init(executable: URL?, arguments: [String] = [], explanation: ExplanationConfiguration? = nil) {
        self.executable = executable
        self.arguments = arguments
        self.explanationConfiguration = explanation ?? .fromEnvironment()
        if executable == nil { state = .noCompiler }
    }

    var workerIsRunning: Bool { worker?.isRunning == true }
    var isInFlight: Bool { !inFlight.isEmpty }
    var debounceIdle: Bool { debounce == nil }

    var statusText: String {
        switch state {
        case .idle: return "preview: waiting for edits"
        case .noCompiler: return "no compiler attached — cannot preview"
        case .notPreviewable(let why): return "preview unavailable: \(why)"
        case .compiling: return "compiling preview…"
        case .failed(let why): return "preview failed: \(why)"
        case .ready(let r):
            let delta = r.pageDelta.map { $0 >= 0 ? "+\($0)" : "\($0)" } ?? "?"
            return "preview: \(r.status.rawValue), \(delta) pages, \(r.new.count) new diagnostic\(r.new.count == 1 ? "" : "s")"
        }
    }

    var hasNewErrors: Bool {
        if case .ready(let r) = state { return r.newErrorCount > 0 }
        return false
    }

    // MARK: driving

    /// Records the latest proposal text and model snapshot; compiles after the
    /// debounce interval. Bursts coalesce into one shadow compile (plus one
    /// baseline compile per document revision).
    func update(from model: ShellModel, latex: String) {
        update(input: Input(model: model), latex: latex)
    }

    func update(input: Input, latex: String) {
        latest = (input, latex)
        // An explanation is bound to one (documents, anchor, revision, proposal)
        // tuple; any change makes it stale, so it is cancelled or cleared.
        if let e = explained, e.input != input || e.latex != latex {
            cancelExplanation(reason: "proposal or document changed")
        }
        guard executable != nil else { state = .noCompiler; return }
        switch Self.makeShadow(input: input, latex: latex) {
        case .success(let s):
            shadow = s
            if case .failed = state {} else { state = .compiling }
        case .failure(let why):
            shadow = nil
            state = .notPreviewable(why.message)
            debounce?.cancel()
            return
        }
        debounce?.cancel()
        let item = DispatchWorkItem { [weak self] in self?.compileLatest() }
        debounce = item
        DispatchQueue.main.asyncAfter(deadline: .now() + Self.debounceInterval, execute: item)
    }

    /// Re-sends after a worker fault; no-op when nothing is pending.
    func retry() {
        if case .failed = state { state = .compiling }
        compileLatest()
    }

    /// Terminates the preview worker. The sheet calls this on dismiss.
    func close() {
        debounce?.cancel()
        debounce = nil
        worker?.terminate()
        worker = nil
        inFlight.removeAll()
        cancelExplanation(reason: "review closed")
    }

    private func compileLatest() {
        debounce = nil
        guard let latest, let executable else { return }
        if !inFlight.isEmpty { return } // coalesce: the newest text goes out when the current result lands
        if let compiled, compiled.input == latest.input, compiled.latex == latest.latex, shadowResult != nil { return }
        guard case .success(let s) = Self.makeShadow(input: latest.input, latex: latest.latex) else { return }
        if worker == nil || worker?.isRunning != true {
            do {
                worker = try WorkerClient(executable: executable, arguments: arguments) { [weak self] event in
                    self?.handle(event)
                }
            } catch {
                state = .failed("launch failed: \(error.localizedDescription)")
                return
            }
        }
        guard let worker else { return }
        let projectId = latest.input.projectId + "-preview"
        do {
            if baselineKey?.documents != latest.input.documents || baselineKey?.entryPath != latest.input.entryPath || baseline == nil {
                baseline = nil
                baselineKey = (latest.input.documents, latest.input.entryPath)
                let (id, revision) = nextRequest()
                let req = RuntimeV1.CompileRequest(projectId: projectId, revision: revision,
                                                   entryPath: latest.input.entryPath, documents: latest.input.documents)
                try worker.send(req, id: id)
                inFlight[id] = InFlight(kind: .baseline, projectId: projectId, revision: req.revision, shadow: nil)
                baselineCompileCount += 1
            }
            let (id, revision) = nextRequest()
            let req = RuntimeV1.CompileRequest(projectId: projectId, revision: revision,
                                               entryPath: latest.input.entryPath, documents: s.documents)
            try worker.send(req, id: id)
            inFlight[id] = InFlight(kind: .shadow, projectId: projectId, revision: req.revision, shadow: s)
            shadowCompileCount += 1
            shadowResult = nil
            compiled = latest
            state = .compiling
        } catch {
            state = .failed("send failed: \(error.localizedDescription)")
        }
    }

    /// Request ids are `preview-N`; the revision field carries N too so a
    /// mismatched reply can be rejected like the main worker's.
    private func nextRequest() -> (id: String, revision: Int) {
        defer { nextRequestID += 1 }
        return ("preview-\(nextRequestID)", nextRequestID)
    }

    private func handle(_ event: WorkerClient.Event) {
        switch event {
        case .result(let env):
            guard let sent = inFlight.removeValue(forKey: env.id) else { return } // not ours; never applied
            let r = env.payload
            guard r.projectId == sent.projectId, r.revision == sent.revision else {
                state = .failed("protocol violation: \(env.id) answered project \(r.projectId) revision \(r.revision)")
                return
            }
            switch sent.kind {
            case .baseline: baseline = r
            case .shadow: if let s = sent.shadow { shadowResult = (s, r, env.id) }
            }
            if inFlight.isEmpty { finish() }
        case .error(let id, let message):
            inFlight.removeValue(forKey: id)
            state = .failed("worker error: \(message)")
        case .protocolViolation(let message):
            inFlight.removeAll()
            state = .failed("protocol violation: \(message)")
        case .stderr:
            break
        case .exited(let code):
            let lost = !inFlight.isEmpty
            inFlight.removeAll()
            worker = nil
            // An idle exit after close() changes nothing; a crash mid-compile is
            // reported and Retry relaunches the worker.
            if lost { state = .failed("preview worker exited (\(code))") }
        }
    }

    private func finish() {
        if let (s, r, _) = shadowResult {
            let report = Self.report(shadow: s, result: r, baseline: baseline)
            thumbnail = report.insertionPage.flatMap { Self.thumbnail(of: $0, in: r) }
            state = .ready(report)
        }
        // A newer edit arrived while compiling: go again.
        if let latest, let compiled, latest.input != compiled.input || latest.latex != compiled.latex {
            compileLatest()
        }
    }

    // MARK: pure pieces (tested without a worker)

    struct ShadowError: Error, Equatable { var message: String }

    /// Applies the proposal at the anchor exactly as approval would, in a copy.
    static func makeShadow(input: Input, latex: String) -> Result<Shadow, ShadowError> {
        guard let anchor = input.anchor else { return .failure(.init(message: "no insertion point pinned")) }
        guard let doc = input.documents.first(where: { $0.path == anchor.path }) else {
            return .failure(.init(message: "anchor document \(anchor.path) is not open"))
        }
        let byte: Int
        switch Insertion.resolve(anchor, in: doc.text, revision: input.editorRevision) {
        case .exact(let b), .rebased(let b): byte = b
        case .needsReselection(let why): return .failure(.init(message: why))
        }
        guard !latex.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            return .failure(.init(message: "proposal is empty"))
        }
        guard let idx = doc.text.rangeOfUTF8(start: byte, end: byte)?.lowerBound else {
            return .failure(.init(message: "anchor offset is not a valid position"))
        }
        let insert = Insertion.insertionText(latex, into: doc.text, atByte: byte)
        var text = doc.text
        text.insert(contentsOf: insert, at: idx)
        var docs = input.documents
        if let i = docs.firstIndex(where: { $0.path == anchor.path }) { docs[i].text = text }
        return .success(Shadow(documents: docs, path: anchor.path, insertByte: byte, insertedText: insert))
    }

    /// The inserted range widened to its enclosing lines plus one line before
    /// and after, in shadow-document bytes.
    static func neighborhood(of inserted: Range<Int>, in text: String) -> Range<Int> {
        let bytes = Array(text.utf8)
        var start = min(inserted.lowerBound, bytes.count)
        var linesBack = 0
        while start > 0 {
            if bytes[start - 1] == UInt8(ascii: "\n") {
                linesBack += 1
                if linesBack > 1 { break }
            }
            start -= 1
        }
        var end = min(inserted.upperBound, bytes.count)
        var linesForward = 0
        while end < bytes.count {
            if bytes[end] == UInt8(ascii: "\n") {
                linesForward += 1
                if linesForward > 1 { break }
            }
            end += 1
        }
        return start..<end
    }

    private static func intersects(_ a: Range<Int>, _ b: Range<Int>) -> Bool {
        a.lowerBound <= b.upperBound && b.lowerBound <= a.upperBound
    }

    /// Diagnostics in `shadow` that have no counterpart in `baseline`. Ranges
    /// after the insertion are shifted back by the inserted length before
    /// comparing; anything overlapping the inserted text is new by definition.
    static func newDiagnostics(baseline: [RuntimeV1.Diagnostic], shadow diags: [RuntimeV1.Diagnostic],
                               shadow: Shadow) -> [RuntimeV1.Diagnostic] {
        func key(_ d: RuntimeV1.Diagnostic, _ range: Range<Int>?) -> String {
            let loc = range.map { "\(d.source!.path):\($0.lowerBound)..<\($0.upperBound)" } ?? "-"
            return "\(d.severity.rawValue)|\(d.message)|\(loc)"
        }
        var counts: [String: Int] = [:]
        for d in baseline {
            counts[key(d, d.source.map { $0.startByte..<max($0.startByte, $0.endByte) }), default: 0] += 1
        }
        var fresh: [RuntimeV1.Diagnostic] = []
        for d in diags {
            var normalized: Range<Int>? = nil
            if let s = d.source {
                let r = s.startByte..<max(s.startByte, s.endByte)
                if s.path == shadow.path {
                    if r.lowerBound >= shadow.insertedRange.upperBound {
                        normalized = (r.lowerBound - shadow.insertedLength)..<(r.upperBound - shadow.insertedLength)
                    } else if r.upperBound <= shadow.insertedRange.lowerBound {
                        normalized = r
                    } else {
                        fresh.append(d); continue
                    }
                } else {
                    normalized = r
                }
            }
            let k = key(d, normalized)
            if let n = counts[k], n > 0 { counts[k] = n - 1 } else { fresh.append(d) }
        }
        return fresh
    }

    static func report(shadow: Shadow, result: RuntimeV1.CompileResult, baseline: RuntimeV1.CompileResult?) -> Report {
        let text = shadow.shadowText
        let nb = neighborhood(of: shadow.insertedRange, in: text)
        let fresh = newDiagnostics(baseline: baseline?.diagnostics ?? [], shadow: result.diagnostics, shadow: shadow)
        func finding(_ i: Int, _ d: RuntimeV1.Diagnostic) -> Finding {
            var fragment: String? = nil
            var offset: Int? = nil
            if let s = d.source, s.path == shadow.path {
                let lo = max(0, min(s.startByte, text.utf8.count)), hi = max(lo, min(s.endByte, text.utf8.count))
                let ctxLo = max(0, lo - 12), ctxHi = min(text.utf8.count, max(hi, lo + 1) + 12)
                if let r = text.rangeOfUTF8(start: ctxLo, end: ctxHi) {
                    fragment = String(text[r]).replacingOccurrences(of: "\n", with: "⏎")
                }
                if lo >= shadow.bodyStart, lo < shadow.insertedRange.upperBound { offset = lo - shadow.bodyStart }
            }
            return Finding(id: i, diagnostic: d, fragment: fragment, proposalOffset: offset)
        }
        var nearby: [Finding] = []
        var unrelated = 0
        for (i, d) in result.diagnostics.enumerated() {
            if let s = d.source, s.path == shadow.path, intersects(s.startByte..<max(s.startByte, s.endByte), nb) {
                nearby.append(finding(i, d))
            } else if d.source == nil, fresh.contains(d) {
                nearby.append(finding(i, d)) // unlocated but new: show it rather than hide it
            } else {
                unrelated += 1
            }
        }
        let new = fresh.enumerated().map { finding(1000 + $0.offset, $0.element) }
        return Report(status: result.status, pageCount: result.pages.count,
                      pageDelta: baseline.map { result.pages.count - $0.pages.count },
                      nearby: nearby, new: new, unrelatedCount: unrelated,
                      insertionPage: insertionPage(in: result, shadow: shadow))
    }

    /// First page with a text item whose source range intersects the inserted
    /// text (touching counts, so an item ending exactly at the insertion point
    /// still locates the page). Nil when no item maps the insertion.
    static func insertionPage(in result: RuntimeV1.CompileResult, shadow: Shadow) -> Int? {
        for page in result.pages {
            for case .text(let t) in page.items {
                guard let s = t.source, s.path == shadow.path else { continue }
                if intersects(s.startByte..<max(s.startByte, s.endByte), shadow.insertedRange) { return page.number }
            }
        }
        return nil
    }

    /// Renders only `pageNumber` of `result` through `PDFExport` and asks
    /// PDFKit for a thumbnail. Nil when the page is missing or PDFKit refuses.
    static func thumbnail(of pageNumber: Int, in result: RuntimeV1.CompileResult,
                          width: CGFloat = 180) -> NSImage? {
        guard let page = result.pages.first(where: { $0.number == pageNumber }), page.widthPt > 0 else { return nil }
        var single = result
        single.pages = [page]
        let data = PDFExport.render(single)
        guard let doc = PDFDocument(data: data), let pdfPage = doc.page(at: 0) else { return nil }
        let size = NSSize(width: width, height: width * page.heightPt / page.widthPt)
        return pdfPage.thumbnail(of: size, for: .mediaBox)
    }
}

// MARK: - Assistant explanation (crates/assistant-context helper)

/// One bounded child process: `input` goes to stdin (then EOF), stdout is
/// collected up to `maxOutputBytes`, and the exit status is observed. The
/// timeout and `cancel()` terminate the process (SIGTERM, then SIGKILL after
/// a grace period). The completion runs exactly once, on the main queue.
final class OneShotProcess {
    enum Failure: Error, Equatable {
        case launch(String)
        case timeout(TimeInterval)
        /// Nonzero exit; `output` is whatever stdout held (a helper `error` reply).
        case exited(Int32, output: Data, stderr: String)
        case outputTooLarge(Int)
        case cancelled
    }
    struct Output: Equatable { var stdout: Data; var stderr: String }

    let executable: URL
    private let process = Process()
    private let stdin = Pipe(), stdout = Pipe(), stderr = Pipe()
    private let lock = NSLock()
    private var out = Data(), err = Data()
    private var timedOut = false, cancelled = false, overflow = false, delivered = false
    private var pipesOpen = 2
    private var exitStatus: Int32?
    private let maxOutputBytes: Int
    private let timeout: TimeInterval
    private let completion: (Result<Output, Failure>) -> Void
    private static let io = DispatchQueue(label: "flashtex.oneshot.io", attributes: .concurrent)

    var isRunning: Bool { process.isRunning }
    var processIdentifier: Int32 { process.processIdentifier }

    /// The process keeps `self` alive through its termination handler until it
    /// has exited and both pipes reached EOF; dropping the last outside
    /// reference never loses the completion.
    init(executable: URL, arguments: [String], input: Data, timeout: TimeInterval, maxOutputBytes: Int,
         environment: [String: String]? = nil,
         completion: @escaping (Result<Output, Failure>) -> Void) throws {
        self.executable = executable
        self.maxOutputBytes = maxOutputBytes
        self.timeout = timeout
        self.completion = completion
        signal(SIGPIPE, SIG_IGN) // a helper that exits early must not kill the app
        process.executableURL = executable
        process.arguments = arguments
        if let environment { process.environment = environment }
        process.standardInput = stdin
        process.standardOutput = stdout
        process.standardError = stderr
        process.terminationHandler = { [self] p in
            lock.withLock { exitStatus = p.terminationStatus }
            maybeFinish()
            // A descendant holding the pipes open must not defer the completion forever.
            Self.io.asyncAfter(deadline: .now() + 1) { [self] in maybeFinish(force: true) }
        }
        try process.run()
        Self.io.async { [stdin] in
            try? stdin.fileHandleForWriting.write(contentsOf: input)
            try? stdin.fileHandleForWriting.close()
        }
        // Blocking readers drain each pipe to EOF so the child never stalls on a
        // full pipe and no byte is lost between the last read and termination.
        Self.io.async { [self] in drain(stdout.fileHandleForReading, isStdout: true) }
        Self.io.async { [self] in drain(stderr.fileHandleForReading, isStdout: false) }
        Self.io.asyncAfter(deadline: .now() + timeout) { [self] in expire() }
    }

    /// Terminates the process; the completion then reports `.cancelled`.
    func cancel() {
        lock.withLock { cancelled = true }
        terminate()
    }

    private func terminate() {
        guard process.isRunning else { return }
        process.terminate()
        let pid = process.processIdentifier
        Self.io.asyncAfter(deadline: .now() + 2) { [self] in
            if process.isRunning { kill(pid, SIGKILL) }
        }
    }

    private func expire() {
        guard process.isRunning else { return }
        lock.withLock { timedOut = true }
        terminate()
    }

    private func drain(_ fh: FileHandle, isStdout: Bool) {
        while true {
            let data = fh.availableData
            if data.isEmpty { break }
            let tooBig: Bool = lock.withLock {
                if isStdout { out.append(data) } else if err.count < 64 * 1024 { err.append(data.prefix(64 * 1024 - err.count)) }
                if out.count > maxOutputBytes { overflow = true; return true }
                return false
            }
            if tooBig { terminate() }
        }
        try? fh.close()
        lock.withLock { pipesOpen -= 1 }
        maybeFinish()
    }

    private func maybeFinish(force: Bool = false) {
        let result: Result<Output, Failure>? = lock.withLock {
            guard !delivered, let status = exitStatus, force || pipesOpen == 0 else { return nil }
            delivered = true
            let stderrText = String(decoding: err, as: UTF8.self)
            if cancelled { return .failure(.cancelled) }
            if timedOut { return .failure(.timeout(timeout)) }
            if overflow { return .failure(.outputTooLarge(out.count)) }
            if status != 0 { return .failure(.exited(status, output: out, stderr: stderrText)) }
            return .success(Output(stdout: out, stderr: stderrText))
        }
        guard let result else { return }
        process.terminationHandler = nil
        DispatchQueue.main.async { self.completion(result) }
    }
}

extension ProposalPreview {
    /// Where the helper and the optional provider live. The provider is a
    /// local command the user enables explicitly (`FLASHTEX_ASSISTANT_PROVIDER`);
    /// unset means disabled: the prepared context is shown and sent nowhere.
    /// FlashTeX itself makes no network calls; a provider command may, and only
    /// because the user configured it.
    struct ExplanationConfiguration: Equatable {
        var helper: URL?
        var helperArguments: [String] = []
        var provider: URL?
        var providerArguments: [String] = []
        /// The helper is a local, bounded JSON transform; seconds suffice.
        var helperTimeout: TimeInterval = 10
        /// `ExplanationFlight` in the crate allows at most 120 s.
        var providerTimeout: TimeInterval = 60
        static let helperOutputLimit = 128 * 1024   // the helper's own cap
        static let providerOutputLimit = 64 * 1024  // `validate_response` refuses more

        /// The helper interface consumed here (`prepare`/`validate`/`review`/
        /// `approve`, one JSON request per process) as published on
        /// origin/agent/commander/assistant-context: interface commit
        /// c8a3e1b, recorded `examples/review-workflow.json` fixture c93bf0d.
        /// `Tests/FlashTeXMacTests/Fixtures/build_assistant_context.sh` builds
        /// exactly that source into `apps/mac/build/assistant-context`.
        static let helperInterfaceCommit = "c8a3e1ba0bb795635d3ff84e6c00d0544a0ee88a"
        static let helperSourceCommit = "c93bf0d7fd6c44bc656fdcb905e9a102e6cfad40"
        static let scratchHelperPath = "apps/mac/build/assistant-context/crates/assistant-context/target/release/flashtex-assistant-context"

        static let disabled = ExplanationConfiguration(helper: nil)

        @MainActor static func fromEnvironment(_ env: [String: String] = ProcessInfo.processInfo.environment,
                                               bundleExecutableDirectory: URL? = Bundle.main.executableURL?.deletingLastPathComponent()) -> ExplanationConfiguration {
            var c = ExplanationConfiguration(helper: locateHelper(env, bundleExecutableDirectory: bundleExecutableDirectory))
            if let p = env["FLASHTEX_ASSISTANT_PROVIDER"], FileManager.default.isExecutableFile(atPath: p) {
                c.provider = URL(fileURLWithPath: p)
            }
            if let s = env["FLASHTEX_ASSISTANT_TIMEOUT_S"], let t = TimeInterval(s), t > 0 { c.providerTimeout = min(t, 120) }
            return c
        }

        /// The helper's file name inside a packaged app (`FlashTeX.app/Contents/
        /// MacOS/flashtex-assistant-context`, next to the other bundled helpers;
        /// `scripts/make-app.sh --assistant <path>` is the packaging lane's entry).
        static let bundledHelperName = "flashtex-assistant-context"

        /// `FLASHTEX_ASSISTANT_CONTEXT`, then `bundledHelperName` in the bundle's
        /// MacOS directory (`bundleExecutableDirectory`, the running executable's
        /// directory by default), then the pinned scratch build
        /// (`scratchHelperPath`), then the checkout's own crate build (which may
        /// predate `review`/`approve`; those steps then fail honestly).
        @MainActor static func locateHelper(_ env: [String: String] = ProcessInfo.processInfo.environment,
                                            bundleExecutableDirectory: URL? = Bundle.main.executableURL?.deletingLastPathComponent()) -> URL? {
            let fm = FileManager.default
            if let p = env["FLASHTEX_ASSISTANT_CONTEXT"], fm.isExecutableFile(atPath: p) { return URL(fileURLWithPath: p) }
            if let bundled = bundleExecutableDirectory?.appendingPathComponent(bundledHelperName),
               fm.isExecutableFile(atPath: bundled.path) {
                return bundled
            }
            guard let root = ShellModel.locateRepoRoot() else { return nil }
            var candidates = [root.appendingPathComponent(scratchHelperPath)]
            for profile in ["release", "debug"] {
                candidates.append(root.appendingPathComponent("crates/assistant-context/target/\(profile)/flashtex-assistant-context"))
            }
            return candidates.first { fm.isExecutableFile(atPath: $0.path) }
        }

        /// What the sheet shows for the provider: the enabled command's file
        /// name (the user chose it) or "disabled". Never a URL or a key.
        var providerIdentityText: String {
            provider.map { "provider: \($0.lastPathComponent)" } ?? "provider disabled (set FLASHTEX_ASSISTANT_PROVIDER to a local command)"
        }

        enum ChildRole { case helper, provider }

        /// Environment handed to a child. The helper gets a minimal offline
        /// environment: no credential-, token- or proxy-like variables (the
        /// helper's default build has no network code either; the `grok`
        /// feature is off and `FLASHTEX_GROK_API_KEY` is always removed). The
        /// provider is the user's own command and inherits the user's
        /// environment unchanged, minus nothing this app adds.
        static func childEnvironment(for role: ChildRole, from env: [String: String] = ProcessInfo.processInfo.environment) -> [String: String] {
            switch role {
            case .provider: return env
            case .helper:
                let keep: Set<String> = ["PATH", "HOME", "TMPDIR", "LANG", "LC_ALL", "LC_CTYPE", "USER", "SHELL", "RUST_BACKTRACE"]
                return env.filter { keep.contains($0.key) && !isSensitiveVariable($0.key) }
            }
        }

        static func isSensitiveVariable(_ name: String) -> Bool {
            let upper = name.uppercased()
            return upper == "FLASHTEX_GROK_API_KEY" || upper.contains("API_KEY") || upper.contains("APIKEY")
                || upper.contains("TOKEN") || upper.contains("SECRET") || upper.contains("PASSWORD")
                || upper.contains("CREDENTIAL") || upper.hasSuffix("_PROXY") || upper == "PROXY"
        }
    }

    /// One child-process launch as the preview performed it (tests inspect argv
    /// and environment; nothing here is a network operation).
    struct ChildLaunch: Equatable {
        var role: ExplanationConfiguration.ChildRole
        var executable: URL
        var arguments: [String]
        var environmentKeys: [String]
        var stage: ExplanationStage
    }

    struct ByteRange: Equatable {
        var path: String
        var startByte: Int
        var endByte: Int
    }

    /// A context the helper prepared and hashed: bound to one shadow compile
    /// request/revision and the exact shadow sources.
    struct ExplanationContext: Equatable {
        var requestId: String
        var shadowRequestId: String
        var projectId: String
        var compileRevision: Int
        var contextId: String
        var compilerStatus: String
        var diagnosticCount: Int
        var omittedDiagnostics: Int
        /// Always explicit. Empty means explanation only; otherwise edits may
        /// touch only these bytes of the shadow document (the proposal body).
        var allowedEdits: [ByteRange]
        var providerIntent: String
        /// The exact `PromptPayload` bytes a provider receives.
        var payload: Data
        var payloadBytes: Int { payload.count }
        var editBoundaryText: String {
            allowedEdits.isEmpty ? "explanation only (no edit region)"
                : allowedEdits.map { "\($0.path) bytes \($0.startByte)..<\($0.endByte)" }.joined(separator: ", ")
        }
    }

    /// One model-proposed edit as validated by the helper. Displayed only.
    struct ProposedEdit: Equatable, Identifiable {
        var id: Int
        var range: ByteRange
        var removedText: String
        var replacement: String
        /// Offset within the proposal body when the edit lies inside it.
        var proposalOffset: Int?
    }

    struct Explanation: Equatable {
        var context: ExplanationContext
        var text: String
        var edits: [ProposedEdit]
        /// Echoed from the helper, which reports `applied:false`; a helper
        /// claiming otherwise is refused (`explanationState == .failed`).
        var applied: Bool
        /// Set when the helper's `review` operation produced a review digest
        /// for these exact edits (nil for explanation-only replies). Approval
        /// must name this exact id (`approveReviewedEdit()`).
        var reviewId: String?
    }

    /// The helper's `approved_group` for the reviewed edit, mapped onto the
    /// proposal text. Nothing has been written anywhere: `amendedLatex` is the
    /// proposal body with the approved edits applied, for the reviewer to put
    /// into the proposal draft; inserting into the document is still the
    /// sheet's own explicit approval.
    struct ApprovedAmendment: Equatable {
        var explanation: Explanation
        var reviewId: String
        var commandId: String
        var expectedRevision: Int
        var expectedSha256: String
        var label: String
        var edits: [ProposedEdit]
        var amendedLatex: String
        var applied: Bool
    }

    enum ExplanationState: Equatable {
        case idle
        case unavailable(String)
        case preparing
        /// Context bound and hashed; no provider is enabled so it went nowhere.
        case prepared(ExplanationContext)
        case awaitingProvider(ExplanationContext)
        case validating(ExplanationContext)
        case ready(Explanation)
        case approving(Explanation)
        case approved(ApprovedAmendment)
        case failed(String)
        case cancelled(String)

        var isInFlight: Bool {
            switch self {
            case .preparing, .awaitingProvider, .validating, .approving: return true
            default: return false
            }
        }
    }

    enum ExplanationStage: Equatable { case probe, prepare, provider, validate, review, approve, done }

    /// Per-request state carried across the helper/provider stages.
    struct ExplanationJob {
        var id: String
        var stage: ExplanationStage
        var shadow: Shadow
        var shadowRequestId: String
        var projectId: String
        var revision: Int
        var sources: [[String: Any]]
        var request: [String: Any]
        var context: ExplanationContext?
        var explanation: Explanation?
    }

    var explanationAvailable: Bool { explanationConfiguration.helper != nil }
    var explanationProviderEnabled: Bool { explanationConfiguration.provider != nil }
    var explanationInFlight: Bool { explanationState.isInFlight }
    /// True when the current shadow result can be explained right now.
    var canExplain: Bool {
        guard case .ready = state, shadowResult != nil, let latest, let compiled else { return false }
        return compiled.input == latest.input && compiled.latex == latest.latex && !explanationState.isInFlight
    }
    /// True when a reviewed edit is displayed and can be explicitly approved.
    var canApproveReviewedEdit: Bool {
        if case .ready(let e) = explanationState, e.reviewId != nil, explanationJob != nil { return true }
        return false
    }
    /// Proposal text with the reviewer-approved edit applied, once an approval
    /// completed; the sheet may offer it as the draft. Never written elsewhere.
    var amendedProposalLatex: String? {
        if case .approved(let a) = explanationState { return a.amendedLatex }
        return nil
    }

    var explanationStatusText: String {
        switch explanationState {
        case .idle: return explanationAvailable
            ? "not requested — runs the local assistant-context helper (\(explanationConfiguration.helper?.lastPathComponent ?? "?")) offline; \(explanationConfiguration.providerIdentityText)"
            : "unavailable: no flashtex-assistant-context helper found (bundle it, build crates/assistant-context, or set FLASHTEX_ASSISTANT_CONTEXT)"
        case .unavailable(let why): return "unavailable: \(why)"
        case .preparing: return "preparing bounded context…"
        case .prepared(let c):
            return "context \(c.contextId.prefix(8)) prepared for \(c.shadowRequestId): \(c.diagnosticCount) diagnostic\(c.diagnosticCount == 1 ? "" : "s")"
                + (c.omittedDiagnostics > 0 ? " (\(c.omittedDiagnostics) omitted)" : "") + ", \(c.payloadBytes) bytes; edits: \(c.editBoundaryText). No provider is enabled — nothing was sent."
        case .awaitingProvider(let c):
            return "context \(c.contextId.prefix(8)) handed to \(explanationConfiguration.provider?.lastPathComponent ?? "your provider command") on stdin; waiting…"
        case .validating(let c): return "validating the provider's reply against context \(c.contextId.prefix(8))…"
        case .ready(let e):
            let edits = "\(e.edits.count) proposed edit\(e.edits.count == 1 ? "" : "s"), none applied"
            return "model output for context \(e.context.contextId.prefix(8)) (\(e.context.shadowRequestId)); \(edits)"
                + (e.reviewId.map { "; review \($0.prefix(8)) awaits your approval" } ?? "")
        case .approving(let e): return "asking the helper to approve review \(e.reviewId?.prefix(8) ?? "?")…"
        case .approved(let a):
            return "review \(a.reviewId.prefix(8)) approved (\(a.commandId.prefix(18))…): \(a.edits.count) edit\(a.edits.count == 1 ? "" : "s") applied to the proposal text only; the document is unchanged until you approve the insertion"
        case .failed(let why): return "assistant failed: \(why)"
        case .cancelled(let why): return "explanation discarded: \(why)"
        }
    }

    // MARK: driving

    /// Asks the helper for a bounded, revision-bound context for the current
    /// shadow compile; then, only if the user enabled a provider command, runs
    /// it and has the helper validate (`validate`) or review (`review`, when
    /// edits are proposed) its reply. Output is displayed as model output.
    /// Nothing here touches the editor buffer or the proposal.
    func explain() {
        guard let helper = explanationConfiguration.helper else {
            explanationState = .unavailable("no flashtex-assistant-context helper found (build crates/assistant-context or set FLASHTEX_ASSISTANT_CONTEXT)")
            return
        }
        guard case .ready = state, let bound = shadowResult, let latest, let compiled,
              compiled.input == latest.input, compiled.latex == latest.latex else {
            explanationState = .failed("the preview compile is not current; wait for it to finish")
            return
        }
        cancelExplanationProcess()
        let id = "explain-\(nextExplanationID)"
        nextExplanationID += 1
        explanationRequestCount += 1
        explained = (latest.input, latest.latex)
        let projectId = latest.input.projectId + "-preview"
        let revision = max(1, latest.input.editorRevision)
        let sources = Self.ledgerDocuments(bound.shadow.documents, projectId: projectId, revision: revision)
        var identities: [String: Any] = [:]
        for doc in bound.shadow.documents { identities[doc.path] = ["revision": revision, "sha256": SourceDigest.sha256Hex(doc.text)] }
        let compilerResult: Any
        do {
            compilerResult = try Self.compilerResultJSON(bound.result, id: bound.requestId)
        } catch {
            explanationState = .failed("could not encode the shadow compile result: \(error.localizedDescription)")
            return
        }
        let selected = Self.selectedDiagnostics(shadow: bound.shadow, result: bound.result, baseline: baseline)
        let request: [String: Any] = [
            "operation": "prepare",
            "binding": ["request_id": bound.requestId, "project_id": projectId, "compile_revision": bound.result.revision,
                        "sources": identities],
            "sources": sources,
            "compiler_result": compilerResult,
            "user_instruction": Self.userInstruction(shadow: bound.shadow, revision: latest.input.editorRevision),
            "selected_diagnostics": selected,
        ]
        let job = ExplanationJob(id: id, stage: .probe, shadow: bound.shadow, shadowRequestId: bound.requestId,
                                 projectId: projectId, revision: revision, sources: sources, request: request,
                                 context: nil, explanation: nil)
        explanationJob = job
        explanationState = .preparing
        runHelper(helper, job: job)
    }

    /// Explicit reviewer approval of the displayed reviewed edit: the helper's
    /// `approve` operation with `user_approved:true` and the exact review id,
    /// against a fresh snapshot of the shadow sources. On success the approved
    /// group is mapped onto the proposal text (`amendedProposalLatex`); the
    /// editor buffer is never written by this object.
    func approveReviewedEdit() {
        guard let helper = explanationConfiguration.helper else { return }
        guard case .ready(let e) = explanationState, let reviewId = e.reviewId, var job = explanationJob,
              job.id == e.context.requestId, job.stage == .done,
              let latest, let e2 = explained, e2.input == latest.input, e2.latex == latest.latex else {
            explanationState = .failed("no reviewed edit is current; explain again first")
            return
        }
        job.stage = .approve
        job.request["operation"] = "approve"
        job.request["user_approved"] = true
        job.request["approved_review_id"] = reviewId
        job.request["current_sources"] = Self.ledgerDocuments(job.shadow.documents, projectId: job.projectId, revision: job.revision)
        job.explanation = e
        explanationJob = job
        explanationState = .approving(e)
        runHelper(helper, job: job)
    }

    /// Cancels any in-flight helper/provider process and clears the state.
    /// A reply that still arrives for the old request is discarded.
    func cancelExplanation(reason: String) {
        cancelExplanationProcess()
        explanationJob = nil
        explained = nil
        switch explanationState {
        case .idle, .unavailable: break
        default: explanationState = .cancelled(reason)
        }
    }

    private func cancelExplanationProcess() {
        explanationProcess?.cancel()
        explanationProcess = nil
    }

    private func runHelper(_ helper: URL, job: ExplanationJob) {
        let data: Data
        do { data = try JSONSerialization.data(withJSONObject: job.request) } catch {
            fail(job.id, "could not encode the helper request: \(error.localizedDescription)"); return
        }
        guard data.count <= 16 * 1024 * 1024 else { fail(job.id, "helper request of \(data.count) bytes exceeds 16 MiB"); return }
        launch(helper, arguments: explanationConfiguration.helperArguments, input: data,
               timeout: explanationConfiguration.helperTimeout, limit: ExplanationConfiguration.helperOutputLimit,
               id: job.id, stage: job.stage)
    }

    private func launch(_ executable: URL, arguments: [String], input: Data, timeout: TimeInterval, limit: Int,
                        id: String, stage: ExplanationStage) {
        let role: ExplanationConfiguration.ChildRole = stage == .provider ? .provider : .helper
        let environment = ExplanationConfiguration.childEnvironment(for: role)
        childLaunches.append(ChildLaunch(role: role, executable: executable, arguments: arguments,
                                         environmentKeys: environment.keys.sorted(), stage: stage))
        if childLaunches.count > 64 { childLaunches.removeFirst(childLaunches.count - 64) }
        do {
            explanationProcess = try OneShotProcess(executable: executable, arguments: arguments, input: input,
                                                    timeout: timeout, maxOutputBytes: limit,
                                                    environment: environment) { [weak self] result in
                self?.handleExplanationReply(id: id, stage: stage, result: result)
            }
        } catch {
            fail(id, "could not launch \(executable.lastPathComponent): \(error.localizedDescription)")
        }
    }

    private func fail(_ id: String, _ why: String) {
        guard explanationJob?.id == id else { staleExplanationReplies += 1; return }
        explanationJob = nil
        explanationProcess = nil
        explanationState = .failed(why)
    }

    /// Single entry point for every helper/provider completion. A reply whose
    /// request id or stage is not the current one is counted and dropped: it
    /// can never change the displayed state, whatever it contains.
    func handleExplanationReply(id: String, stage: ExplanationStage, result: Result<OneShotProcess.Output, OneShotProcess.Failure>) {
        guard var job = explanationJob, job.id == id, job.stage == stage, stage != .done else {
            staleExplanationReplies += 1
            return
        }
        explanationProcess = nil
        let output: OneShotProcess.Output
        switch result {
        case .success(let o): output = o
        case .failure(let f): fail(id, Self.describe(f, stage: stage)); return
        }
        do {
            switch stage {
            case .probe:
                let payload = try Self.preparedPayload(output.stdout)
                let destinations = Self.destinations(for: job.shadow, in: payload)
                job.request["destinations"] = destinations.map { ["path": $0.path, "start_byte": $0.startByte, "end_byte": $0.endByte] }
                job.stage = .prepare
                explanationJob = job
                guard let helper = explanationConfiguration.helper else { throw ExplanationError("helper vanished") }
                runHelper(helper, job: job)
            case .prepare:
                let payload = try Self.preparedPayload(output.stdout)
                let context = try Self.context(from: payload, job: job)
                job.context = context
                if let provider = explanationConfiguration.provider {
                    job.stage = .provider
                    explanationJob = job
                    explanationState = .awaitingProvider(context)
                    launch(provider, arguments: explanationConfiguration.providerArguments, input: context.payload,
                           timeout: explanationConfiguration.providerTimeout,
                           limit: ExplanationConfiguration.providerOutputLimit, id: job.id, stage: .provider)
                } else {
                    job.stage = .done
                    explanationJob = job
                    explanationState = .prepared(context)
                }
            case .provider:
                guard let context = job.context else { throw ExplanationError("no prepared context") }
                guard let response = try? JSONSerialization.jsonObject(with: output.stdout) as? [String: Any] else {
                    throw ExplanationError("the provider's reply is not a JSON object (\(output.stdout.count) bytes)")
                }
                // `review` (a review digest for the exact edits) needs at least one
                // edit; an explanation-only reply goes through `validate`.
                let hasEdits = !((response["edits"] as? [Any]) ?? []).isEmpty
                job.request["operation"] = hasEdits ? "review" : "validate"
                job.request["response"] = response
                job.request["current_sources"] = job.sources
                if hasEdits { job.request["explanation_request_id"] = job.id }
                job.stage = hasEdits ? .review : .validate
                explanationJob = job
                explanationState = .validating(context)
                guard let helper = explanationConfiguration.helper else { throw ExplanationError("helper vanished") }
                runHelper(helper, job: job)
            case .validate, .review:
                guard let context = job.context else { throw ExplanationError("no prepared context") }
                let explanation = try Self.explanation(from: output.stdout, context: context, shadow: job.shadow,
                                                       expecting: stage == .review ? "proposal_review" : "validated_proposal")
                job.stage = .done
                job.explanation = explanation
                explanationJob = job
                explanationState = .ready(explanation)
            case .approve:
                guard let explanation = job.explanation, let reviewId = explanation.reviewId else { throw ExplanationError("no reviewed edit") }
                let amendment = try Self.amendment(from: output.stdout, explanation: explanation, reviewId: reviewId, job: job)
                job.stage = .done
                explanationJob = job
                explanationState = .approved(amendment)
            case .done:
                staleExplanationReplies += 1
            }
        } catch {
            fail(id, (error as? ExplanationError)?.message ?? error.localizedDescription)
        }
    }

    struct ExplanationError: Error { var message: String; init(_ m: String) { message = m } }

    static func describe(_ failure: OneShotProcess.Failure, stage: ExplanationStage) -> String {
        let who = stage == .provider ? "provider" : "helper"
        switch failure {
        case .launch(let m): return "\(who) launch failed: \(m)"
        case .timeout(let t): return "\(who) gave no reply within \(Int(t.rounded(.up))) s (\(stage))"
        case .exited(let code, let output, let stderr):
            if let obj = try? JSONSerialization.jsonObject(with: output) as? [String: Any],
               obj["type"] as? String == "error", let m = obj["message"] as? String {
                return "\(who) refused (\(stage)): \(m)"
            }
            let tail = stderr.split(separator: "\n").last.map { ": \($0.prefix(160))" } ?? ""
            return "\(who) exited (\(code)) during \(stage)\(tail)"
        case .outputTooLarge(let n): return "\(who) reply of \(n) bytes exceeds the limit"
        case .cancelled: return "\(who) cancelled"
        }
    }

    // MARK: pure pieces (tested without processes)

    /// Edit-ledger `Document` objects (the helper's source type) for the shadow
    /// documents: project, path, revision, text and SHA-256 of the text.
    static func ledgerDocuments(_ documents: [RuntimeV1.Document], projectId: String, revision: Int) -> [[String: Any]] {
        documents.map { doc in
            ["project_id": projectId, "path": doc.path, "revision": revision, "text": doc.text,
             "source_sha256": SourceDigest.sha256Hex(doc.text)]
        }
    }

    /// The shadow compile result as the runtime-v1 `compile_result` envelope
    /// the helper binds to. Unlocated diagnostics carry an explicit `source:
    /// null` (the helper requires the key).
    static func compilerResultJSON(_ result: RuntimeV1.CompileResult, id: String) throws -> Any {
        let env = RuntimeV1.Envelope(protocolVersion: RuntimeV1.protocolVersion, id: id, type: "compile_result", payload: result)
        let data = try JSONEncoder().encode(env)
        guard var obj = try JSONSerialization.jsonObject(with: data) as? [String: Any],
              var payload = obj["payload"] as? [String: Any],
              var diags = payload["diagnostics"] as? [[String: Any]] else {
            throw ExplanationError("compile result did not round-trip")
        }
        for i in diags.indices where diags[i]["source"] == nil { diags[i]["source"] = NSNull() }
        payload["diagnostics"] = diags
        obj["payload"] = payload
        return obj
    }

    /// Indices (into the shadow result's diagnostics) of the diagnostics to
    /// explain: those the insertion introduced versus the baseline (when one
    /// exists), then those inside the inserted text, then those near it; at
    /// most 16, unique. The helper hashes this exact selection into the
    /// context id.
    static func selectedDiagnostics(shadow: Shadow, result: RuntimeV1.CompileResult, baseline: RuntimeV1.CompileResult?) -> [Int] {
        let rep = report(shadow: shadow, result: result, baseline: baseline)
        var picked: [Int] = []
        func take(_ d: RuntimeV1.Diagnostic) {
            if let i = result.diagnostics.indices.first(where: { !picked.contains($0) && result.diagnostics[$0] == d }) { picked.append(i) }
        }
        if baseline != nil { for f in rep.new { take(f.diagnostic) } }
        for d in result.diagnostics {
            if let s = d.source, s.path == shadow.path, intersects(s.startByte..<max(s.startByte, s.endByte), shadow.insertedRange) { take(d) }
        }
        for f in rep.nearby { take(f.diagnostic) }
        return Array(picked.prefix(16))
    }

    static func userInstruction(shadow: Shadow, revision: Int) -> String {
        let body = shadow.bodyRange
        return "A captured proposal was inserted into \(shadow.path) at bytes \(body.lowerBound)..<\(body.upperBound) "
            + "(editor revision \(revision)) for review. Explain the selected compiler diagnostics and whether the inserted text "
            + "is at fault. Propose edits only inside the inserted bytes. The reviewer decides; nothing is applied automatically."
    }

    /// Decodes a helper reply of `type` into its payload object; an `error`
    /// reply or another type throws with the helper's message.
    static func helperReply(_ data: Data, type: String) throws -> [String: Any] {
        guard let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            throw ExplanationError("helper reply is not a JSON object (\(data.count) bytes)")
        }
        if obj["type"] as? String == "error" { throw ExplanationError("helper refused: \(obj["message"] as? String ?? "?")") }
        guard obj["type"] as? String == type else {
            throw ExplanationError("helper answered \(obj["type"] as? String ?? "?") instead of \(type)")
        }
        return obj
    }

    /// Decodes a `prepared_context` reply into its payload object.
    static func preparedPayload(_ data: Data) throws -> [String: Any] {
        guard let payload = try helperReply(data, type: "prepared_context")["payload"] as? [String: Any] else {
            throw ExplanationError("prepared_context has no payload")
        }
        return payload
    }

    static func ranges(in list: Any?) -> [ByteRange] {
        (list as? [[String: Any]] ?? []).compactMap { r in
            guard let p = r["path"] as? String, let s = r["start_byte"] as? Int, let e = r["end_byte"] as? Int else { return nil }
            return ByteRange(path: p, startByte: s, endByte: e)
        }
    }

    /// The proposal body clipped to the supplied snippet that overlaps it most:
    /// the helper only accepts destinations inside a snippet, and an edit may
    /// never reach outside the proposal. Empty when no snippet covers the body.
    static func destinations(for shadow: Shadow, in payload: [String: Any]) -> [ByteRange] {
        var snippets: [ByteRange] = []
        for d in payload["diagnostics"] as? [[String: Any]] ?? [] {
            if let s = d["snippet"] as? [String: Any] { snippets += ranges(in: [s["location"] ?? [:]]) }
        }
        for r in payload["related"] as? [[String: Any]] ?? [] { snippets += ranges(in: [r["location"] ?? [:]]) }
        let body = shadow.bodyRange
        var best: ByteRange?
        for s in snippets where s.path == shadow.path {
            let lo = max(s.startByte, body.lowerBound), hi = min(s.endByte, body.upperBound)
            guard hi > lo else { continue }
            if best == nil || hi - lo > best!.endByte - best!.startByte { best = ByteRange(path: shadow.path, startByte: lo, endByte: hi) }
        }
        return best.map { [$0] } ?? []
    }

    /// Checks the prepared payload against the job's binding before trusting it.
    static func context(from payload: [String: Any], job: ExplanationJob) throws -> ExplanationContext {
        guard let contextId = payload["context_id"] as? String, contextId.count == 64,
              let revision = payload["compile_revision"] as? Int, let project = payload["project_id"] as? String else {
            throw ExplanationError("prepared context lacks context_id/compile_revision/project_id")
        }
        guard let bound = job.request["binding"] as? [String: Any], revision == bound["compile_revision"] as? Int, project == job.projectId else {
            throw ExplanationError("helper answered revision \(revision) of \(project), not \(job.shadowRequestId)")
        }
        guard let allowed = payload["allowed_edits"] as? [[String: Any]] else {
            throw ExplanationError("prepared context has no explicit edit boundary")
        }
        let diags = payload["diagnostics"] as? [[String: Any]] ?? []
        let bytes = try JSONSerialization.data(withJSONObject: payload)
        return ExplanationContext(requestId: job.id, shadowRequestId: job.shadowRequestId, projectId: project,
                                  compileRevision: revision, contextId: contextId,
                                  compilerStatus: payload["compiler_status"] as? String ?? "?",
                                  diagnosticCount: diags.count, omittedDiagnostics: payload["omitted_diagnostics"] as? Int ?? 0,
                                  allowedEdits: ranges(in: allowed), providerIntent: payload["provider_intent"] as? String ?? "?",
                                  payload: bytes)
    }

    /// Decodes a `validated_proposal` or `proposal_review` reply. Refuses
    /// anything not bound to `context`, any claim of application, a review
    /// without `requires_user_approval`, or edits outside the boundary.
    static func explanation(from data: Data, context: ExplanationContext, shadow: Shadow,
                            expecting type: String = "validated_proposal") throws -> Explanation {
        let obj: [String: Any]
        do { obj = try helperReply(data, type: type) } catch let e as ExplanationError {
            throw ExplanationError(e.message.replacingOccurrences(of: "helper refused:", with: "helper refused the provider's reply:"))
        }
        guard let payload = obj["payload"] as? [String: Any] else { throw ExplanationError("\(type) has no payload") }
        let applied = obj["applied"] as? Bool ?? true
        guard !applied else { throw ExplanationError("helper claims the edit was applied; refusing (nothing is applied from here)") }
        guard payload["context_id"] as? String == context.contextId else { throw ExplanationError("validated proposal answers another context") }
        guard let text = payload["explanation"] as? String, !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            throw ExplanationError("validated proposal has no explanation")
        }
        var reviewId: String? = nil
        if type == "proposal_review" {
            guard obj["requires_user_approval"] as? Bool == true, let id = obj["review_id"] as? String, id.count == 64 else {
                throw ExplanationError("proposal_review lacks review_id or requires_user_approval")
            }
            reviewId = id
        }
        var edits: [ProposedEdit] = []
        for (i, e) in (payload["edits"] as? [[String: Any]] ?? []).enumerated() {
            guard let range = ranges(in: [e["location"] ?? [:]]).first,
                  let removed = e["removed_text"] as? String, let replacement = e["replacement"] as? String else {
                throw ExplanationError("proposed edit \(i) is malformed")
            }
            guard context.allowedEdits.contains(where: { $0.path == range.path && range.startByte >= $0.startByte && range.endByte <= $0.endByte }) else {
                throw ExplanationError("proposed edit \(i) is outside the proposal boundary")
            }
            let inBody = range.path == shadow.path && range.startByte >= shadow.bodyRange.lowerBound && range.endByte <= shadow.bodyRange.upperBound
            let offset = inBody ? range.startByte - shadow.bodyStart : nil
            edits.append(ProposedEdit(id: i, range: range, removedText: removed, replacement: replacement, proposalOffset: offset))
        }
        if reviewId != nil, edits.isEmpty { throw ExplanationError("proposal_review carries no edits") }
        return Explanation(context: context, text: text, edits: edits, applied: false, reviewId: reviewId)
    }

    /// Decodes an `approved_group` reply and maps it onto the proposal body:
    /// the group must name this review, project and document, the shadow
    /// text's exact revision and SHA-256, and edits whose removed text matches
    /// inside the proposal body. Returns the amended proposal text.
    static func amendment(from data: Data, explanation: Explanation, reviewId: String, job: ExplanationJob) throws -> ApprovedAmendment {
        let obj = try helperReply(data, type: "approved_group")
        guard obj["applied"] as? Bool == false else { throw ExplanationError("helper claims the group was applied; refusing") }
        guard let payload = obj["payload"] as? [String: Any], let group = payload["group"] as? [String: Any] else {
            throw ExplanationError("approved_group has no group")
        }
        let shadow = job.shadow
        guard payload["review_id"] as? String == reviewId, payload["request_id"] as? String == job.id,
              payload["project_id"] as? String == job.projectId, payload["path"] as? String == shadow.path else {
            throw ExplanationError("approved group names another review, request, project or document")
        }
        guard let commandId = group["command_id"] as? String, commandId == "assistant-\(reviewId)",
              let expectedRevision = group["expected_revision"] as? Int, let expectedSha = group["expected_sha256"] as? String,
              let label = group["label"] as? String, let rawEdits = group["edits"] as? [[String: Any]] else {
            throw ExplanationError("approved group is malformed")
        }
        let text = shadow.shadowText
        guard expectedRevision == job.revision, expectedSha == SourceDigest.sha256Hex(text) else {
            throw ExplanationError("approved group expects revision \(expectedRevision)/\(expectedSha.prefix(8)), not the reviewed shadow text")
        }
        var edits: [ProposedEdit] = []
        for (i, e) in rawEdits.enumerated() {
            guard let s = e["start_byte"] as? Int, let en = e["end_byte"] as? Int, s <= en,
                  let removed = e["removed_text"] as? String, let replacement = e["replacement"] as? String else {
                throw ExplanationError("approved edit \(i) is malformed")
            }
            guard s >= shadow.bodyRange.lowerBound, en <= shadow.bodyRange.upperBound else {
                throw ExplanationError("approved edit \(i) reaches outside the proposal body")
            }
            guard let r = text.rangeOfUTF8(start: s, end: en), String(text[r]) == removed else {
                throw ExplanationError("approved edit \(i) does not match the reviewed text")
            }
            edits.append(ProposedEdit(id: i, range: ByteRange(path: shadow.path, startByte: s, endByte: en),
                                      removedText: removed, replacement: replacement, proposalOffset: s - shadow.bodyStart))
        }
        guard edits.map(\.range) == explanation.edits.map(\.range), edits.map(\.replacement) == explanation.edits.map(\.replacement) else {
            throw ExplanationError("approved group differs from the reviewed edits")
        }
        guard let bodyRange = text.rangeOfUTF8(start: shadow.bodyRange.lowerBound, end: shadow.bodyRange.upperBound) else {
            throw ExplanationError("proposal body is not a valid range")
        }
        var body = String(text[bodyRange])
        for e in edits.sorted(by: { $0.range.startByte > $1.range.startByte }) {
            guard let r = body.rangeOfUTF8(start: e.range.startByte - shadow.bodyStart, end: e.range.endByte - shadow.bodyStart) else {
                throw ExplanationError("approved edit is not a valid proposal range")
            }
            body.replaceSubrange(r, with: e.replacement)
        }
        return ApprovedAmendment(explanation: explanation, reviewId: reviewId, commandId: commandId,
                                 expectedRevision: expectedRevision, expectedSha256: expectedSha, label: label,
                                 edits: edits, amendedLatex: body, applied: false)
    }
}

/// Model-output section of the review sheet: explicit request, explicit
/// labeling, display only. The reviewer may copy a proposed replacement or
/// explicitly approve a reviewed edit (which amends the proposal text only);
/// the editor buffer is never written from here.
struct ProposalExplanationView: View {
    @ObservedObject var preview: ProposalPreview

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 6) {
                Image(systemName: "sparkles").font(.caption).foregroundStyle(.secondary)
                Text("Assistant").font(.caption.bold())
                Text("model output — never applied automatically · \(preview.explanationConfiguration.providerIdentityText)")
                    .font(.caption2).foregroundStyle(.secondary).lineLimit(1)
                Spacer()
                if preview.explanationInFlight {
                    ProgressView().controlSize(.mini)
                    Button("Cancel") { preview.cancelExplanation(reason: "cancelled by reviewer") }.controlSize(.mini)
                } else if preview.explanationAvailable {
                    Button(explainTitle) { preview.explain() }.controlSize(.mini).disabled(!preview.canExplain)
                }
            }
            .help("Runs the local flashtex-assistant-context helper as a child process on the shadow compile (offline: no network, no credentials in its environment): bounded bytes, bound to this proposal's revision. A provider command runs only when you enabled one with FLASHTEX_ASSISTANT_PROVIDER; nothing is applied without your approval.")
            Text(preview.explanationStatusText).font(.caption2).foregroundStyle(.secondary).lineLimit(3)
            switch preview.explanationState {
            case .ready(let e), .approving(let e):
                explanationBody(e)
                if let id = e.reviewId {
                    HStack(spacing: 6) {
                        Text("Review \(id.prefix(8)) — approving amends the proposal text above only; inserting into the document remains a separate approval.")
                            .font(.caption2).foregroundStyle(.secondary)
                        Spacer()
                        Button("Approve edit to proposal") { preview.approveReviewedEdit() }
                            .controlSize(.mini).disabled(!preview.canApproveReviewedEdit)
                    }
                }
            case .approved(let a):
                explanationBody(a.explanation)
                VStack(alignment: .leading, spacing: 2) {
                    Text("Approved (\(a.commandId.prefix(18))…) — amended proposal, not yet in the document:").font(.caption2).foregroundStyle(.secondary)
                    Text(a.amendedLatex).font(.system(.caption, design: .monospaced)).textSelection(.enabled)
                        .padding(6).frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color.green.opacity(0.08)).cornerRadius(4)
                    HStack {
                        Spacer()
                        Button("Copy amended proposal") {
                            NSPasteboard.general.clearContents()
                            NSPasteboard.general.setString(a.amendedLatex, forType: .string)
                        }.controlSize(.mini).help("Paste it into the proposal editor above; then approve the insertion as usual.")
                    }
                }
            default:
                EmptyView()
            }
        }
        .padding(.top, 2)
    }

    @ViewBuilder private func explanationBody(_ e: ProposalPreview.Explanation) -> some View {
        Text(e.text).font(.caption).textSelection(.enabled)
            .padding(6).frame(maxWidth: .infinity, alignment: .leading)
            .background(Color.purple.opacity(0.08)).cornerRadius(4)
            .accessibilityLabel("Model explanation, not applied")
        ForEach(e.edits) { edit in
            HStack(alignment: .top, spacing: 6) {
                Image(systemName: "pencil.slash").font(.caption).foregroundStyle(.secondary)
                VStack(alignment: .leading, spacing: 1) {
                    Text("Proposed edit (not applied)" + (edit.proposalOffset.map { " at proposal byte \($0)" } ?? "")).font(.caption2).foregroundStyle(.secondary)
                    Text("− " + edit.removedText).font(.system(.caption2, design: .monospaced)).foregroundStyle(.red).lineLimit(3)
                    Text("+ " + edit.replacement).font(.system(.caption2, design: .monospaced)).foregroundStyle(.green).lineLimit(3)
                }
                Spacer()
                Button("Copy") {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(edit.replacement, forType: .string)
                }.controlSize(.mini).help("Copies the replacement text; paste it into the proposal yourself if you agree.")
            }
        }
    }

    private var explainTitle: String {
        switch preview.explanationState {
        case .idle, .cancelled: return "Explain"
        case .failed: return "Explain again"
        default: return "Re-explain"
        }
    }
}

/// Status line + new-diagnostic list rendered under the sheet's LaTeX editor.
/// "Show" highlights the offending fragment here (a shadow buffer; the real
/// editor is never touched, and SwiftUI's TextEditor cannot select a range).
struct ProposalPreviewView: View {
    @ObservedObject var preview: ProposalPreview

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 6) {
                statusGlyph
                Text(preview.statusText).font(.caption).foregroundStyle(.secondary).lineLimit(1)
                Spacer()
                if case .failed = preview.state { Button("Retry") { preview.retry() }.controlSize(.mini) }
            }
            .help("Compiled on a separate preview worker with the proposal inserted at the anchor; the editor buffer is untouched.")
            if case .ready(let r) = preview.state {
                ForEach(r.new) { f in
                    HStack(alignment: .top, spacing: 6) {
                        Image(systemName: f.diagnostic.severity == .error ? "xmark.octagon.fill" : "exclamationmark.triangle.fill")
                            .foregroundStyle(f.diagnostic.severity == .error ? .red : .orange).font(.caption)
                        VStack(alignment: .leading, spacing: 1) {
                            Text(f.diagnostic.message).font(.caption)
                            Text(f.diagnostic.recovery.map { "↳ recovery: \($0)" } ?? "↳ no provisional rendering")
                                .font(.caption2).foregroundStyle(.secondary)
                            if let o = f.proposalOffset {
                                Text("at proposal byte \(o)").font(.caption2).foregroundStyle(.tertiary)
                            }
                        }
                        Spacer()
                        if f.fragment != nil { Button("Show") { preview.highlightedFragment = f.fragment }.controlSize(.mini) }
                    }
                }
                if let frag = preview.highlightedFragment {
                    Text(frag).font(.system(.caption, design: .monospaced))
                        .padding(4).background(Color.yellow.opacity(0.25)).cornerRadius(3)
                }
                let pre = r.nearby.count - r.nearby.filter { n in r.new.contains { $0.diagnostic == n.diagnostic } }.count
                if pre > 0 || r.unrelatedCount > 0 {
                    Text("\(pre) pre-existing near the insertion, \(r.unrelatedCount) elsewhere (unchanged)")
                        .font(.caption2).foregroundStyle(.tertiary)
                }
                if let image = preview.thumbnail, let page = r.insertionPage {
                    HStack(alignment: .top, spacing: 6) {
                        Image(nsImage: image).resizable().aspectRatio(contentMode: .fit)
                            .frame(maxWidth: 120).border(.separator)
                        Text("shadow page \(page) of \(r.pageCount), with the proposal inserted (not the live preview)")
                            .font(.caption2).foregroundStyle(.tertiary)
                    }
                }
            }
            ProposalExplanationView(preview: preview)
        }
    }

    private var statusGlyph: some View {
        Group {
            switch preview.state {
            case .compiling: ProgressView().controlSize(.mini)
            case .ready(let r):
                Image(systemName: r.newErrorCount > 0 ? "xmark.octagon.fill" : (r.status == .ok ? "checkmark.circle.fill" : "exclamationmark.triangle.fill"))
                    .foregroundStyle(r.newErrorCount > 0 ? .red : (r.status == .ok ? .green : .orange))
            case .failed, .noCompiler: Image(systemName: "bolt.slash").foregroundStyle(.orange)
            case .idle, .notPreviewable: Image(systemName: "eye.slash").foregroundStyle(.secondary)
            }
        }
        .font(.caption)
    }
}

/// Warning glyph shown next to Approve when the preview introduced errors.
/// Approval is never blocked: the reviewer decides.
struct ProposalApproveWarning: View {
    @ObservedObject var preview: ProposalPreview
    var body: some View {
        if preview.hasNewErrors, case .ready(let r) = preview.state {
            Image(systemName: "exclamationmark.triangle.fill").foregroundStyle(.orange)
                .help("Preview compile reports \(r.newErrorCount) new error\(r.newErrorCount == 1 ? "" : "s") from this insertion. You may still approve.")
        }
    }
}
