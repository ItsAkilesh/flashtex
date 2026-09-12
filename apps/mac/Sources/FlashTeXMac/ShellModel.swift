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

    // Capture review / insertion (contract: "Capture and insertion").
    struct PendingEdit: Equatable { var path: String; var nsRange: NSRange; var text: String; var token: Int }
    @Published var caretUTF16: Int = 0
    @Published var anchor: InsertionAnchor?
    @Published var proposals: [RuntimeV1.CaptureProposal] = []
    @Published var reviewing: RuntimeV1.CaptureProposal?
    @Published var pendingEdit: PendingEdit?
    @Published var captureNote: String?
    private(set) var appliedCaptureIDs: Set<String> = []
    private var nextAnchorNumber = 1
    @Published var workerStatus: String = "no worker attached"
    @Published var workerLog: [String] = []
    private var worker: WorkerClient?
    private var nextRequestID = 1
    /// Revision of the compile request currently in flight (nil if idle).
    @Published private(set) var inFlightRevision: Int?
    /// Revision the editor buffer corresponds to. Bumps on every edit so the
    /// UI can say when the preview's source ranges no longer match the buffer.
    @Published private(set) var editorRevision = 1

    enum PreviewSource: Equatable { case none, fixture, worker(String) }

    var isFixture: Bool { previewSource == .fixture }
    var previewIsStale: Bool { (result?.revision ?? editorRevision) != editorRevision }
    var workerAttached: Bool { worker?.isRunning == true }

    var activeText: String {
        get { documents.first { $0.path == activePath }?.text ?? "" }
    }

    init() {
        if let root = Self.locateRepoRoot() {
            let fixtures = root.appendingPathComponent("protocol/fixtures")
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
            if let request, let data = try? Data(contentsOf: request),
               let req = try? RuntimeV1.decodeCompileRequest(data) {
                documents = req.payload.documents
                activePath = req.payload.entryPath
                editorRevision = req.payload.revision
            } else if documents.isEmpty {
                documents = [.init(path: "main.tex", text: "")]
            }
            selection = nil
            navigationNote = nil
        } catch {
            loadError = "Failed to load \(result.lastPathComponent): \(error)"
        }
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

    func updateActiveText(_ text: String) {
        guard let i = documents.firstIndex(where: { $0.path == activePath }) else { return }
        guard documents[i].text != text else { return }
        documents[i].text = text
        editorRevision += 1
    }

    // MARK: navigation (preview -> source)

    /// Converts the contract's UTF-8 byte range to a UTF-16 selection in the
    /// matching document and asks the editor to select it.
    func navigate(to source: RuntimeV1.SourceRange?) {
        guard let source else {
            navigationNote = "This item has no source mapping."
            return
        }
        guard let doc = documents.first(where: { $0.path == source.path }) else {
            navigationNote = "No open document named \(source.path)."
            return
        }
        guard let ns = doc.text.nsRange(utf8Bytes: source) else {
            navigationNote = "Bytes \(source.startByte)..<\(source.endByte) are not a valid range in \(source.path) (buffer is \(doc.text.utf8.count) bytes)."
            return
        }
        activePath = source.path
        selection = .init(path: source.path, nsRange: ns, token: (selection?.token ?? 0) + 1)
        navigationNote = "Selected \(source.path) bytes \(source.startByte)..<\(source.endByte) → UTF-16 \(ns.location)..<\(ns.location + ns.length)"
            + (previewIsStale ? " (buffer edited since revision \(result?.revision ?? 0); mapping may be off)" : "")
    }

    // MARK: worker transport (runtime v1 JSON Lines)

    /// Finds a built FT-002 worker: $FLASHTEX_COMPILER, then
    /// crates/compiler/target/{release,debug}/flashtex-compiler under the repo root.
    static func locateCompiler() -> URL? {
        let fm = FileManager.default
        if let env = ProcessInfo.processInfo.environment["FLASHTEX_COMPILER"], fm.isExecutableFile(atPath: env) {
            return URL(fileURLWithPath: env)
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
        worker?.terminate()
        worker = nil
        inFlightRevision = nil
        if previewSource != .fixture { workerStatus = "no worker attached" }
    }

    /// Sends the current buffers as a `compile` request. Never blocks the UI.
    func compile() {
        guard let worker, worker.isRunning else {
            workerStatus = "no worker attached"
            return
        }
        let id = "mac-\(nextRequestID)"
        nextRequestID += 1
        let request = RuntimeV1.CompileRequest(
            projectId: result?.projectId ?? "demo",
            revision: editorRevision,
            entryPath: activePath,
            documents: documents)
        do {
            try worker.send(request, id: id)
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
            // Contract: never replace a newer preview with an older revision.
            if let current = result, previewSource != .fixture, incoming.revision < current.revision {
                log("ignored stale compile_result revision \(incoming.revision) < \(current.revision)")
                return
            }
            result = incoming
            resultID = env.id
            previewSource = .worker(worker?.executable.lastPathComponent ?? "worker")
            if inFlightRevision == incoming.revision { inFlightRevision = nil }
            workerStatus = "revision \(incoming.revision): \(incoming.status.rawValue), \(incoming.diagnostics.count) diagnostics"
            selection = nil
        case .error(let id, let message):
            inFlightRevision = nil
            workerStatus = "worker error for \(id): \(message)"
            log("error \(id): \(message)")
        case .protocolViolation(let message):
            workerStatus = "protocol violation: \(message)"
            log("protocol violation: \(message)")
        case .stderr(let text):
            log(text.trimmingCharacters(in: .whitespacesAndNewlines))
        case .exited(let code):
            inFlightRevision = nil
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
