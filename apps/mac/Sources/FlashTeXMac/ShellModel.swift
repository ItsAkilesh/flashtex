import AppKit
import Combine
import FlashTeXProtocol

/// State for the editor/preview shell. The preview is fixture-backed: nothing
/// here compiles LaTeX. `isFixture` is surfaced in the UI so the shell never
/// implies a real compiler ran.
@MainActor
final class ShellModel: ObservableObject {
    struct Selection: Equatable {
        var path: String
        var nsRange: NSRange
        var token = 0 // bump so the same range re-applies
    }

    @Published var documents: [RuntimeV1.Document] = []
    @Published var activePath: String = "main.tex"
    @Published var result: RuntimeV1.CompileResult?
    @Published var resultID: String?
    @Published var fixtureURL: URL?
    @Published var loadError: String?
    @Published var selection: Selection?
    @Published var navigationNote: String?
    @Published var darkPreview = false
    @Published var previewSource: PreviewSource = .none
    /// File backing the entry document, if any, and its last saved contents.
    @Published var documentURL: URL?
    @Published var savedText: String?

    // Capture review / insertion (contract: "Capture and insertion").
    struct PendingEdit: Equatable { var path: String; var nsRange: NSRange; var text: String; var token: Int }
    @Published var caretUTF16: Int = 0
    @Published var anchor: InsertionAnchor?
    @Published var proposals: [RuntimeV1.CaptureProposal] = []
    @Published var reviewing: RuntimeV1.CaptureProposal?
    @Published var pendingEdit: PendingEdit?
    @Published var captureNote: String?
    private(set) var appliedCaptureIDs: Set<String> = []
    let nearbyInbox = NearbyInbox() // captures from paired companions (ShellModel+Nearby.swift)
    private var nextAnchorNumber = 1
    @Published var workerStatus: String = "no worker attached"
    @Published var workerLog: [String] = []
    private var worker: WorkerClient?
    private var nextRequestID = 1
    /// Text each document had when the current `result` was produced, so stale
    /// byte offsets can be rebased (or refused) after edits.
    private(set) var compiledDocuments: [String: String] = [:]
    struct InFlight { var projectId: String; var revision: Int; var documents: [RuntimeV1.Document]; var sentAt: Date }
    private(set) var inFlightRequests: [String: InFlight] = [:]
    @Published var autoCompile = true
    @Published private(set) var lastLatencyMs: Double?
    @Published private(set) var latenciesMs: [Double] = []
    private var debounce: DispatchWorkItem?
    private var compileQueued = false
    static let debounceInterval: TimeInterval = 0.25
    /// Revision of the compile request currently in flight (nil if idle).
    @Published private(set) var inFlightRevision: Int?
    /// Revision the editor buffer corresponds to. Bumps on every edit so the
    /// UI can say when the preview's source ranges no longer match the buffer.
    @Published private(set) var editorRevision = 1

    enum PreviewSource: Equatable { case none, fixture, worker(String) }

    var isFixture: Bool { previewSource == .fixture }
    var previewIsStale: Bool { (result?.revision ?? editorRevision) != editorRevision }
    var medianLatencyMs: Double? {
        guard !latenciesMs.isEmpty else { return nil }
        let sorted = latenciesMs.sorted()
        return sorted[sorted.count / 2]
    }
    var workerAttached: Bool { worker?.isRunning == true }

    var activeText: String {
        get { documents.first { $0.path == activePath }?.text ?? "" }
    }

    /// Diagnostic underlines for the active document, rebased across edits or
    /// dropped (see `EditorDiagnostics`).
    var editorMarks: [EditorDiagnostics.Mark] {
        guard let result else { return [] }
        return EditorDiagnostics.marks(for: result, path: activePath,
                                       compiledText: compiledDocuments[activePath], currentText: activeText)
    }

    // MARK: caret sync (source -> preview)

    /// UTF-8 byte offset of the editor caret in `activeText`, or nil when the
    /// UTF-16 caret is out of range for the buffer.
    var caretByte: Int? {
        activeText.utf8ByteRange(of: NSRange(location: caretUTF16, length: 0))?.start
    }

    /// Preview items under the caret, as `page number -> item indices`.
    /// Empty when there is no result or the caret maps to nothing.
    var caretItems: [Int: Set<Int>] {
        guard let result, let byte = caretByte else { return [:] }
        return CaretSync.indicesByPage(byte: byte, path: activePath, in: result)
    }

    init() {
        let env = ProcessInfo.processInfo.environment
        defer {
            // Demo/automation hooks: seed the editor from a .tex file and attach the
            // built compiler at launch when FLASHTEX_AUTOATTACH=1 (opt-in so tests
            // that construct ShellModel stay fixture-backed).
            if let seed = env["FLASHTEX_SEED_FILE"], let text = try? String(contentsOfFile: seed, encoding: .utf8) {
                replaceProject(entryText: text)
                documentURL = URL(fileURLWithPath: seed)
                savedText = text
            }
            // A compiler shipped inside the .app bundle attaches by default.
            let bundledCompiler = Bundle.main.executableURL?.deletingLastPathComponent()
                .appendingPathComponent("flashtex-compiler").path
            let hasBundled = bundledCompiler.map { FileManager.default.isExecutableFile(atPath: $0) } ?? false
            if env["FLASHTEX_AUTOATTACH"] != "0", (env["FLASHTEX_AUTOATTACH"] == "1" || hasBundled),
               Self.locateCompiler() != nil {
                attachDiscoveredWorker()
                compile()
            }
        }
        if let fixtures = Self.locateFixturesDirectory() {
            loadFixtures(request: fixtures.appendingPathComponent("compile-request.json"),
                         result: fixtures.appendingPathComponent("compile-result.json"))
        } else {
            loadError = "Could not locate protocol/fixtures (set FLASHTEX_REPO or use File > Open)."
            documents = [.init(path: "main.tex", text: "")]
        }
    }

    // MARK: loading

    func loadFixtures(request: URL?, result: URL) {
        loadError = nil
        do {
            let res = try RuntimeV1.decodeCompileResult(Data(contentsOf: result))
            self.result = res.payload
            self.resultID = res.id
            self.fixtureURL = result
            self.previewSource = .fixture
            // Seed the editor from `request` or, for `<name>-result.json`, a
            // sibling `<name>-request.json` (e.g. Samples/multipage-*.json).
            var candidates: [URL] = []
            if let request { candidates.append(request) }
            if let sibling = Self.siblingRequestURL(forResult: result) { candidates.append(sibling) }
            if let req = candidates.lazy.compactMap({ url -> RuntimeV1.Envelope<RuntimeV1.CompileRequest>? in
                guard let data = try? Data(contentsOf: url) else { return nil }
                return try? RuntimeV1.decodeCompileRequest(data)
            }).first {
                documents = req.payload.documents
                activePath = req.payload.entryPath
                editorRevision = req.payload.revision
                compiledDocuments = Dictionary(uniqueKeysWithValues: req.payload.documents.map { ($0.path, $0.text) })
            } else {
                if documents.isEmpty { documents = [.init(path: "main.tex", text: "")] }
                compiledDocuments = [:]
            }
            selection = nil
            navigationNote = nil
        } catch {
            loadError = "Failed to load \(result.lastPathComponent): \(error)"
        }
    }

    /// `<dir>/<name>-request.json` for a result named `<name>-result.json`, else nil.
    static func siblingRequestURL(forResult result: URL) -> URL? {
        let name = result.deletingPathExtension().lastPathComponent
        guard name.hasSuffix("-result"), result.pathExtension == "json" else { return nil }
        let stem = String(name.dropLast("-result".count))
        return result.deletingLastPathComponent().appendingPathComponent("\(stem)-request.json")
    }

    func reloadFixture() {
        guard let url = fixtureURL else { return }
        let request = url.deletingLastPathComponent().appendingPathComponent("compile-request.json")
        loadFixtures(request: request, result: url)
    }

    func openFixturePanel() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.json]
        panel.message = "Choose a runtime v1 compile_result JSON file"
        if panel.runModal() == .OK, let url = panel.url {
            let request = url.deletingLastPathComponent().appendingPathComponent("compile-request.json")
            loadFixtures(request: request, result: url)
        }
    }

    // MARK: editing

    /// Replaces the whole project with one entry document (File > Open).
    func replaceProject(entryText text: String) {
        documents = [.init(path: "main.tex", text: text)]
        activePath = "main.tex"
        compiledDocuments = [:]
        result = nil
        resultID = nil
        previewSource = .none
        selection = nil
        anchor = nil
        editorRevision += 1
    }

    func updateActiveText(_ text: String) {
        guard let i = documents.firstIndex(where: { $0.path == activePath }) else { return }
        guard documents[i].text != text else { return }
        documents[i].text = text
        editorRevision += 1
        scheduleAutoCompile()
    }

    private func scheduleAutoCompile() {
        guard autoCompile, workerAttached else { return }
        debounce?.cancel()
        let item = DispatchWorkItem { [weak self] in self?.compile() }
        debounce = item
        DispatchQueue.main.asyncAfter(deadline: .now() + Self.debounceInterval, execute: item)
    }

    // MARK: navigation (preview -> source)

    /// Converts the contract's UTF-8 byte range to a UTF-16 selection in the
    /// matching document and asks the editor to select it.
    func navigate(to source: RuntimeV1.SourceRange?) {
        guard let source else {
            navigationNote = "This item has no source mapping."
            return
        }
        navigate(to: source, expectedText: nil)
    }

    /// `expectedText` (an item's text) lets a rebased range be verified.
    func navigate(to source: RuntimeV1.SourceRange, expectedText: String?) {
        guard let doc = documents.first(where: { $0.path == source.path }) else {
            navigationNote = "No open document named \(source.path)."
            return
        }
        var target = source
        var rebasedNote = ""
        if let compiled = compiledDocuments[source.path], compiled != doc.text {
            guard let mapped = SourceMapping.rebase(source, from: compiled, to: doc.text, expectedText: expectedText) else {
                navigationNote = "Source for this item was edited since revision \(result?.revision ?? 0); recompile to navigate."
                return
            }
            if mapped != source {
                rebasedNote = " (rebased from \(source.startByte)..<\(source.endByte) across edits)"
            }
            target = mapped
        } else if compiledDocuments[source.path] == nil, previewIsStale {
            navigationNote = "Buffer edited since revision \(result?.revision ?? 0) and no compiled text is recorded; recompile to navigate."
            return
        }
        guard let ns = doc.text.nsRange(utf8Bytes: target) else {
            navigationNote = "Bytes \(target.startByte)..<\(target.endByte) are not a valid range in \(source.path) (buffer is \(doc.text.utf8.count) bytes)."
            return
        }
        activePath = source.path
        selection = .init(path: source.path, nsRange: ns, token: (selection?.token ?? 0) + 1)
        navigationNote = "Selected \(source.path) bytes \(target.startByte)..<\(target.endByte) → UTF-16 \(ns.location)..<\(ns.location + ns.length)" + rebasedNote
    }

    // MARK: worker transport (runtime v1 JSON Lines)

    /// Finds a built FT-002 worker: $FLASHTEX_COMPILER, then
    /// crates/compiler/target/{release,debug}/flashtex-compiler under the repo root.
    static func locateCompiler() -> URL? {
        let fm = FileManager.default
        if let env = ProcessInfo.processInfo.environment["FLASHTEX_COMPILER"], fm.isExecutableFile(atPath: env) {
            return URL(fileURLWithPath: env)
        }
        if let bundled = Bundle.main.executableURL?.deletingLastPathComponent().appendingPathComponent("flashtex-compiler"),
           fm.isExecutableFile(atPath: bundled.path) {
            return bundled
        }
        guard let root = locateRepoRoot() else { return nil }
        for profile in ["release", "debug"] {
            let url = root.appendingPathComponent("crates/compiler/target/\(profile)/flashtex-compiler")
            if fm.isExecutableFile(atPath: url.path) { return url }
        }
        return nil
    }

    /// Attaches the discovered worker if any; returns whether one was found.
    @discardableResult
    func attachDiscoveredWorker() -> Bool {
        guard let url = Self.locateCompiler() else {
            workerStatus = "no built flashtex-compiler found (build crates/compiler or set FLASHTEX_COMPILER)"
            return false
        }
        attachWorker(at: url)
        return true
    }

    func attachWorkerPanel() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.message = "Choose the Rust worker executable (runtime v1 JSON Lines on stdin/stdout)"
        if panel.runModal() == .OK, let url = panel.url { attachWorker(at: url) }
    }

    func attachWorker(at url: URL, arguments: [String] = []) {
        detachWorker()
        do {
            worker = try WorkerClient(executable: url, arguments: arguments) { [weak self] event in
                self?.handle(event)
            }
            workerStatus = "attached: \(url.lastPathComponent)"
            log("launched \(url.path)")
        } catch {
            workerStatus = "launch failed: \(error.localizedDescription)"
        }
    }

    func detachWorker() {
        debounce?.cancel()
        worker?.terminate()
        worker = nil
        inFlightRequests.removeAll()
        inFlightRevision = nil
        compileQueued = false
        if previewSource != .fixture { workerStatus = "no worker attached" }
    }

    /// Sends the current buffers as a `compile` request. Never blocks the UI.
    func compile() {
        guard let worker, worker.isRunning else {
            workerStatus = "no worker attached"
            return
        }
        debounce?.cancel()
        if inFlightRevision != nil {
            // Coalesce: one request in flight; the newest buffer goes out when it returns.
            compileQueued = true
            return
        }
        if let current = result, previewSource != .fixture, current.revision == editorRevision { return }
        let id = "mac-\(nextRequestID)"
        nextRequestID += 1
        let request = RuntimeV1.CompileRequest(
            projectId: result?.projectId ?? "demo",
            revision: editorRevision,
            entryPath: activePath,
            documents: documents)
        do {
            try worker.send(request, id: id)
            inFlightRequests[id] = InFlight(projectId: request.projectId, revision: request.revision,
                                            documents: documents, sentAt: Date())
            inFlightRevision = editorRevision
            workerStatus = "compiling revision \(editorRevision) (\(id))…"
        } catch {
            workerStatus = "send failed: \(error.localizedDescription)"
        }
    }

    func handleForTesting(_ event: WorkerClient.Event) { handle(event) }

    private func handle(_ event: WorkerClient.Event) {
        switch event {
        case .result(let env):
            let incoming = env.payload
            // Correlate to the exact in-flight request: id, project, and revision must
            // all match. Unsolicited or mismatched results are logged and never applied.
            guard let sent = inFlightRequests[env.id] else {
                log("ignored compile_result with unknown id \(env.id) (revision \(incoming.revision))")
                return
            }
            guard incoming.projectId == sent.projectId, incoming.revision == sent.revision else {
                inFlightRequests.removeValue(forKey: env.id)
                if inFlightRevision == sent.revision { inFlightRevision = nil }
                let msg = "compile_result \(env.id) reports project \(incoming.projectId) revision \(incoming.revision); request was project \(sent.projectId) revision \(sent.revision)"
                log("rejected mismatched " + msg)
                workerStatus = "protocol violation: " + msg
                return
            }
            inFlightRequests.removeValue(forKey: env.id)
            // Contract: never replace a newer preview with an older revision.
            if let current = result, previewSource != .fixture, incoming.revision < current.revision {
                log("ignored stale compile_result revision \(incoming.revision) < \(current.revision)")
                if inFlightRevision == incoming.revision { inFlightRevision = nil }
                return
            }
            result = incoming
            resultID = env.id
            previewSource = .worker(worker?.executable.lastPathComponent ?? "worker")
            compiledDocuments = Dictionary(uniqueKeysWithValues: sent.documents.map { ($0.path, $0.text) })
            let ms = Date().timeIntervalSince(sent.sentAt) * 1000
            lastLatencyMs = ms
            latenciesMs.append(ms)
            if latenciesMs.count > 100 { latenciesMs.removeFirst(latenciesMs.count - 100) }
            let latencyText = String(format: " in %.0f ms", ms)
            if inFlightRevision == incoming.revision { inFlightRevision = nil }
            workerStatus = "revision \(incoming.revision): \(incoming.status.rawValue), \(incoming.diagnostics.count) diagnostics\(latencyText)"
            selection = nil
            if compileQueued {
                compileQueued = false
                if editorRevision != incoming.revision { compile() }
            }
        case .error(let id, let message):
            inFlightRequests.removeValue(forKey: id)
            inFlightRevision = nil
            compileQueued = false
            workerStatus = "worker error for \(id): \(message)"
            log("error \(id): \(message)")
        case .protocolViolation(let message):
            workerStatus = "protocol violation: \(message)"
            log("protocol violation: \(message)")
        case .stderr(let text):
            log(text.trimmingCharacters(in: .whitespacesAndNewlines))
        case .exited(let code):
            inFlightRequests.removeAll()
            inFlightRevision = nil
            compileQueued = false
            workerStatus = "worker exited (\(code))"
            log("worker exited with status \(code)")
            worker = nil
        }
    }

    private func log(_ line: String) {
        workerLog.append(line)
        if workerLog.count > 200 { workerLog.removeFirst(workerLog.count - 200) }
    }

    // MARK: capture review and insertion

    /// Pins the current caret as the insertion destination (`destination_id`).
    func pinAnchorAtCaret() {
        guard let anchor = Insertion.makeAnchor(id: "mac-anchor-\(nextAnchorNumber)", path: activePath,
                                                text: activeText, caretUTF16: caretUTF16, revision: editorRevision)
        else { captureNote = "Caret position is not valid."; return }
        nextAnchorNumber += 1
        self.anchor = anchor
        captureNote = "Pinned \(anchor.id) at \(anchor.path) byte \(anchor.byteOffset) (revision \(anchor.revision))."
    }

    func openProposalPanel() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.json]
        panel.message = "Choose a runtime v1 capture_proposal JSON file"
        if panel.runModal() == .OK, let url = panel.url { loadProposal(from: url) }
    }

    func loadProposal(from url: URL) {
        do {
            let env = try RuntimeV1.decodeCaptureProposal(Data(contentsOf: url))
            enqueue(env.payload)
        } catch {
            captureNote = "Failed to load proposal: \(error)"
        }
    }

    /// Queues a proposal for review. Repeated capture IDs never insert twice.
    func enqueue(_ proposal: RuntimeV1.CaptureProposal) {
        if appliedCaptureIDs.contains(proposal.captureId) {
            captureNote = "Capture \(proposal.captureId) was already inserted; ignoring duplicate."
            return
        }
        if proposals.contains(where: { $0.captureId == proposal.captureId }) {
            captureNote = "Capture \(proposal.captureId) is already awaiting review."
            return
        }
        proposals.append(proposal)
        captureNote = "Proposal \(proposal.captureId) awaiting review (\(proposals.count) queued)."
        if reviewing == nil { reviewing = proposals.first }
    }

    func rejectProposal(_ proposal: RuntimeV1.CaptureProposal) {
        proposals.removeAll { $0.captureId == proposal.captureId }
        if reviewing?.captureId == proposal.captureId { reviewing = proposals.first }
        captureNote = "Rejected \(proposal.captureId)."
    }

    enum ApproveOutcome: Equatable { case inserted(byteOffset: Int), needsReselection(String), duplicate, noAnchor }

    /// Applies one undoable edit after explicit approval. `latex` may have been
    /// edited by the reviewer. Returns what happened so the UI can explain it.
    @discardableResult
    func approveProposal(_ proposal: RuntimeV1.CaptureProposal, latex: String) -> ApproveOutcome {
        guard !appliedCaptureIDs.contains(proposal.captureId) else {
            rejectProposal(proposal); captureNote = "Capture \(proposal.captureId) already inserted."; return .duplicate
        }
        guard let anchor else {
            captureNote = "Pin an insertion point first (Edit > Pin Insertion Point)."; return .noAnchor
        }
        guard let doc = documents.first(where: { $0.path == anchor.path }) else {
            captureNote = "Anchor document \(anchor.path) is not open."; return .needsReselection("document closed")
        }
        let byte: Int
        switch Insertion.resolve(anchor, in: doc.text, revision: editorRevision) {
        case .exact(let b), .rebased(let b): byte = b
        case .needsReselection(let why):
            self.anchor = nil
            captureNote = "Cannot insert \(proposal.captureId): \(why). Pin a new insertion point."
            return .needsReselection(why)
        }
        let insert = Insertion.insertionText(latex, into: doc.text, atByte: byte)
        guard let ns = doc.text.nsRange(utf8Bytes: .init(path: anchor.path, startByte: byte, endByte: byte)) else {
            captureNote = "Anchor offset is not a valid position."; return .needsReselection("invalid offset")
        }
        activePath = anchor.path
        pendingEdit = .init(path: anchor.path, nsRange: ns, text: insert, token: (pendingEdit?.token ?? 0) + 1)
        appliedCaptureIDs.insert(proposal.captureId)
        proposals.removeAll { $0.captureId == proposal.captureId }
        reviewing = proposals.first
        // Keep the anchor after the inserted text so successive captures append in order.
        self.anchor = InsertionAnchor(id: anchor.id, path: anchor.path, byteOffset: byte + insert.utf8.count,
                                      revision: editorRevision, contextAfter: anchor.contextAfter)
        captureNote = "Inserted \(proposal.captureId) at byte \(byte) (undo with ⌘Z)."
        return .inserted(byteOffset: byte)
    }

    /// Called by the editor once it has applied a pending edit (with undo registered).
    func editApplied(_ edit: PendingEdit, newText: String) {
        if pendingEdit == edit { pendingEdit = nil }
        updateActiveText(newText)
        // The insertion is the edit that bumped the revision; the anchor is exact for it.
        if let a = anchor, a.path == edit.path {
            anchor = InsertionAnchor(id: a.id, path: a.path, byteOffset: a.byteOffset,
                                     revision: editorRevision, contextAfter: a.contextAfter)
        }
    }

    // MARK: repo discovery

    /// Fixtures directory: `Contents/Resources/Samples` when running as a packaged
    /// `FlashTeX.app` (see `scripts/make-app.sh`), else `protocol/fixtures` under
    /// the repo root.
    static func locateFixturesDirectory() -> URL? {
        if let samples = Bundle.main.resourceURL?.appendingPathComponent("Samples"),
           FileManager.default.fileExists(atPath: samples.appendingPathComponent("compile-result.json").path) {
            return samples
        }
        return locateRepoRoot()?.appendingPathComponent("protocol/fixtures")
    }

    static func locateRepoRoot() -> URL? {
        var candidates: [URL] = []
        if let env = ProcessInfo.processInfo.environment["FLASHTEX_REPO"] {
            candidates.append(URL(fileURLWithPath: env))
        }
        candidates.append(URL(fileURLWithPath: FileManager.default.currentDirectoryPath))
        candidates.append(Bundle.main.bundleURL)
        candidates.append(URL(fileURLWithPath: #filePath))
        for start in candidates {
            var url = start
            for _ in 0..<8 {
                if FileManager.default.fileExists(atPath: url.appendingPathComponent("protocol/fixtures/compile-result.json").path) {
                    return url
                }
                let parent = url.deletingLastPathComponent()
                if parent == url { break }
                url = parent
            }
        }
        return nil
    }
}
