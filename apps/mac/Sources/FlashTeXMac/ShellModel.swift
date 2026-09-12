import AppKit
import Combine
import Observation
import FlashTeXProtocol

/// State for the editor/preview shell. The preview is fixture-backed: nothing
/// here compiles LaTeX. `isFixture` is surfaced in the UI so the shell never
/// implies a real compiler ran.
/// `@Observable`: SwiftUI tracks the properties each view actually reads, so a
/// keystroke (documents/editorRevision) re-evaluates the editor pane and the
/// footer, and a compile result the preview — not the whole window.
@MainActor
@Observable
final class ShellModel {
    struct Selection: Equatable {
        var path: String
        var nsRange: NSRange
        var token = 0 // bump so the same range re-applies
    }

    var documents: [RuntimeV1.Document] = []
    var activePath: String = "main.tex"
    var result: RuntimeV1.CompileResult?
    var resultID: String?
    var fixtureURL: URL?
    var loadError: String?
    var selection: Selection?
    var navigationNote: String?
    var darkPreview = false
    var previewV2 = ProcessInfo.processInfo.environment["FLASHTEX_PREVIEW_V2"] == "1" // experimental v2 pane (PreviewV2View.swift)
    var displayListV2: V2PreviewState?
    var previewSource: PreviewSource = .none
    /// File backing the entry document, if any, and its last saved contents.
    var documentURL: URL?
    var savedText: String?
    var recoverableBuffer: RecoverableBuffer?

    // Capture review / insertion (contract: "Capture and insertion").
    struct PendingEdit: Equatable { var path: String; var nsRange: NSRange; var text: String; var token: Int }
    var caretUTF16: Int = 0
    var anchor: InsertionAnchor?
    var proposals: [RuntimeV1.CaptureProposal] = []
    var reviewing: RuntimeV1.CaptureProposal?
    var pendingEdit: PendingEdit?
    var captureNote: String?
    var appliedCaptureIDs: Set<String> = []
    var nextAnchorNumber = 1
    /// UTF-16 length of the editor selection starting at `caretUTF16` (0 = caret only).
    var caretLengthUTF16: Int = 0
    // Capture bridge (contract: transfer-v1). See ShellModel+Bridge.swift.
    var bridgeStatus: String = "no bridge attached" { didSet { FlashTeXLog.write("bridge: " + bridgeStatus) } }
    var bridgeCaptures: [BridgeSession.Capture] = []
    var bridgeDestination: TransferV1.Anchor?
    private(set) var bridge: BridgeSession?
    /// Deadline for each `capture_status` during restart reconciliation.
    var bridgeStatusTimeout: TimeInterval = 15
    var workerStatus: String = "no worker attached" { didSet { FlashTeXLog.write("status: " + workerStatus) } }
    let nearbyInbox = NearbyInbox() // captures from paired companions (ShellModel+Nearby.swift)
    var workerLog: [String] = []
    @ObservationIgnored private var worker: WorkerClient?
    /// How the current worker was launched, so an abnormal exit can relaunch
    /// the same executable (bounded: `maxWorkerRelaunches` per minute).
    @ObservationIgnored private var workerLaunch: (url: URL, arguments: [String])?
    @ObservationIgnored private var workerRelaunchTimes: [Date] = []
    @ObservationIgnored private var workerRelaunchWork: DispatchWorkItem?
    static let maxWorkerRelaunches = 3
    /// Relaunch delays after the 1st, 2nd, 3rd abnormal exit within a minute.
    static let workerRelaunchDelays: [TimeInterval] = [0.2, 1.0, 3.0]
    /// Number of automatic relaunches performed so far (for status/tests).
    private(set) var workerRelaunchCount = 0
    /// Durable-source helper (`flashtex-preview-controller`), see ShellModel+Controller.swift.
    @ObservationIgnored var controller: PreviewControllerClient?
    @ObservationIgnored var controllerState = ControllerState()
    /// Status of the helper route (attached / ready / durable revision / errors).
    var controllerStatus: String = "no preview controller attached"
    /// Set while the displayed result is a completed OLDER snapshot from the
    /// helper (HistoricalPreview.swift): navigation, diagnostic jump, caret
    /// sync, capture destinations and export are disabled until a current
    /// preview replaces it. Explicit, never inferred from staleness.
    var historicalPreview: HistoricalDisplay?
    @ObservationIgnored var historicalState = HistoricalPreviewState()
    /// Project-index completion vocabulary (labels/citations/commands) bound to
    /// the editor revision it was fetched for (Completion.swift).
    var completionMetadata: Completion.Metadata?
    @ObservationIgnored let completionFetcher = ProjectIndexCompletionFetcher()
    @ObservationIgnored private var nextRequestID = 1
    /// Text each document had when the current `result` was produced, so stale
    /// byte offsets can be rebased (or refused) after edits.
    private(set) var compiledDocuments: [String: String] = [:]
    struct InFlight {
        var projectId: String; var revision: Int; var documents: [RuntimeV1.Document]; var sentAt: Date
        /// `layout_capabilities` this request carried; the reply is checked against it.
        var layoutCapabilities: [String] = []
    }
    private(set) var inFlightRequests: [String: InFlight] = [:]
    /// Id of the most recently sent compile request. A reply to any older
    /// request is valid but stale (`scripts/check_runtime.py`: `stale_ignore`):
    /// it is checked, logged and dropped, and never changes the preview or the
    /// renderer mode. Only one request is normally in flight (edits coalesce);
    /// a capability-set switch is the exception and goes out immediately.
    private(set) var latestRequestID: String?
    /// Optional verbatim JSON Lines record for `scripts/check_runtime.py`
    /// (`FLASHTEX_TRANSCRIPT=<path>`; tests inject their own).
    var transcript: RuntimeTranscript? = RuntimeTranscript.fromEnvironment()

    // MARK: negotiated layout capabilities (runtime-v1-layout-capabilities.md)

    /// Capabilities sent with every compile request. Default: `rules-v1` and
    /// `font-hints-v1` (gate 3: consumer tests pass), overridable with
    /// `FLASHTEX_LAYOUT_CAPABILITIES` (comma-separated; empty string disables).
    /// Changing the set is a mode switch: with auto-compile on, the current
    /// buffers are re-requested at the *same* revision under the new set.
    var requestedLayoutCapabilities: [String] = ShellModel.defaultLayoutCapabilities() {
        didSet { if oldValue != requestedLayoutCapabilities, autoCompile, workerAttached { compile() } }
    }
    /// Negotiation bound to the *applied* result: what its own request asked for
    /// and what the producer accepted. A late reply for another request never
    /// changes this (results are correlated by id + project + revision first).
    private(set) var negotiation: LayoutNegotiation = .legacy
    var acceptedLayoutCapabilities: [String] { negotiation.accepted }
    /// Font hints the applied result carries that could not be honored exactly.
    private(set) var fontSubstitutions: [PreviewFonts.Substitution] = []
    /// Source-aware errors for unknown primitives in the applied result
    /// (negotiated route only; the legacy route still skips unknown kinds).
    private(set) var layoutDiagnostics: [RuntimeV1.Diagnostic] = []
    /// Producer diagnostics followed by the shell's own layout diagnostics.
    var displayedDiagnostics: [RuntimeV1.Diagnostic] { (result?.diagnostics ?? []) + layoutDiagnostics }

    /// Explicit banner notes: requested-but-unaccepted capabilities and font
    /// substitutions. Never inferred from item shapes.
    var capabilityNotes: [String] {
        negotiation.missing.map { "capability \($0) not accepted by the worker" } + fontSubstitutions.map(\.description)
    }

    /// Gate 3 (contract): requested by default now that the consumer tests
    /// (LayoutCapabilityTests, LayoutCapabilityConsumerTests) pass.
    static let builtInLayoutCapabilities = RuntimeV1.LayoutCapabilities.supported

    /// `FLASHTEX_LAYOUT_CAPABILITIES` (unset → built-in default; "" → none).
    static func defaultLayoutCapabilities(environment: [String: String] = ProcessInfo.processInfo.environment) -> [String] {
        guard let raw = environment["FLASHTEX_LAYOUT_CAPABILITIES"] else { return builtInLayoutCapabilities }
        return raw.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) }.filter { !$0.isEmpty }
    }

    /// Binds a result's negotiation state, substitutions, and layout diagnostics.
    func bindLayout(of applied: RuntimeV1.CompileResult, requested: [String]) {
        negotiation = LayoutNegotiation(requested: requested, accepted: applied.layoutCapabilities ?? [])
        fontSubstitutions = PreviewFonts.substitutions(in: applied)
        layoutDiagnostics = LayoutNegotiation.unsupportedPrimitiveDiagnostics(in: applied, negotiation: negotiation)
        for note in capabilityNotes { log(note) }
        for d in layoutDiagnostics { log(d.message) }
    }
    var autoCompile = true
    private(set) var lastLatencyMs: Double?
    private(set) var latenciesMs: [Double] = []
    @ObservationIgnored private var debounce: DispatchWorkItem?
    @ObservationIgnored private var compileQueued = false
    /// Keystroke-to-compile delay. The compiler answers in ~1–9 ms for typical
    /// documents, so the default is 0: every edit submits immediately and the
    /// one-in-flight coalescing absorbs bursts. `FLASHTEX_DEBOUNCE_MS` overrides.
    static let debounceInterval: TimeInterval = {
        if let s = ProcessInfo.processInfo.environment["FLASHTEX_DEBOUNCE_MS"], let ms = Double(s) { return max(0, ms) / 1000 }
        return 0
    }()
    /// Revision of the compile request currently in flight (nil if idle).
    var inFlightRevision: Int?
    /// Revision the editor buffer corresponds to. Bumps on every edit so the
    /// UI can say when the preview's source ranges no longer match the buffer.
    private(set) var editorRevision = 1

    enum PreviewSource: Equatable { case none, fixture, worker(String) }

    /// Project ID used for compile requests and the bridge (`demo` until a
    /// result names one). Bridge IDs must be 1–128 `[A-Za-z0-9_-]`.
    var projectId: String { result?.projectId ?? "demo" }

    func setBridge(_ session: BridgeSession?) {
        bridge?.terminate()
        bridge = session
        bridgeDestination = nil
        bridgeCaptures = session?.captures ?? []
        bridgeStatus = session?.status ?? "no bridge attached"
    }

    /// Restart reconciliation may need the editor revision to pass a revision
    /// the bridge already confirmed; revisions only ever advance.
    func advanceEditorRevision(atLeast revision: Int) {
        guard revision > editorRevision else { return }
        editorRevision = revision
        // The buffer did not change, but a result compiled at the old revision
        // would now read as stale forever (until the next keystroke); recompile
        // so the preview and caret sync bind to the revision the bridge knows.
        if result != nil { scheduleAutoCompile() }
    }

    var isFixture: Bool { previewSource == .fixture }
    var previewIsStale: Bool { (result?.revision ?? editorRevision) != editorRevision }
    var medianLatencyMs: Double? {
        guard !latenciesMs.isEmpty else { return nil }
        let sorted = latenciesMs.sorted()
        return sorted[sorted.count / 2]
    }
    var workerAttached: Bool { worker?.isRunning == true || controller?.isRunning == true }
    /// True when previews come from the durable helper instead of the direct worker.
    var controllerAttached: Bool { controller?.isRunning == true }

    var activeText: String {
        get { documents.first { $0.path == activePath }?.text ?? "" }
    }

    /// Diagnostic underlines for the active document, rebased across edits or
    /// dropped (see `EditorDiagnostics`).
    var editorMarks: [EditorDiagnostics.Mark] { editorMarkReport.marks }

    /// Marks plus the diagnostics withheld after an edit; `staleNote` is
    /// shown in the footer. Memoized: ContentView reads this on every body
    /// evaluation and the rebase compares the compiled and current texts.
    var editorMarkReport: EditorDiagnostics.Report {
        // Historical spans are inert: not drawn even when their offsets are in bounds.
        guard let result, historicalPreview == nil else { return .empty }
        let key = EditorMarksKey(resultID: resultID, resultRevision: result.revision, editorRevision: editorRevision, path: activePath,
                                 explanationsCount: explanations[resultID]?.count ?? -1)
        if let cached = editorMarksCache, cached.key == key { return cached.report }
        let report = EditorDiagnostics.attach(explanations[resultID], to: EditorDiagnostics.report(
            for: result, resultID: resultID, path: activePath,
            compiledText: compiledDocuments[activePath], currentText: activeText))
        editorMarksCache = (key, report)
        return report
    }
    private struct EditorMarksKey: Equatable { var resultID: String?; var resultRevision: Int; var editorRevision: Int; var path: String; var explanationsCount: Int }
    @ObservationIgnored private var editorMarksCache: (key: EditorMarksKey, report: EditorDiagnostics.Report)?
    /// Identity of the mark last reached by ⌘⇧]/⌘⇧[, so marks sharing a
    /// start offset are each visited once (Navigation.swift).
    @ObservationIgnored var currentDiagnosticID: String?

    /// Offline explanations per result id (crates/diagnostic-explanations via
    /// flashtex-explain); attached to marks, never blocking a keystroke.
    private(set) var explanations = EditorDiagnostics.ExplanationCache()
    @ObservationIgnored private var explanationClient: ExplanationClient?
    var explanationStatus: String?

    /// A reviewed quick fix being previewed (sheet); nil when none.
    var quickFix: EditorDiagnostics.QuickFix.Preview?
    var quickFixIndex: Int?

    /// "Fix…" on a diagnostics row: prepare the suggestion against the current
    /// buffer and show the preview; refusals go to the footer.
    func previewQuickFix(diagnosticIndex: Int, suggestion: Int = 0) {
        guard let x = explanations.explanation(resultID: resultID, index: diagnosticIndex) else {
            navigationNote = "No explanation for this diagnostic yet."; return
        }
        switch EditorDiagnostics.QuickFix.prepare(x, suggestion: suggestion, path: activePath,
                                                  in: activeText, compiledText: compiledDocuments[activePath]) {
        case .success(let preview): quickFix = preview; quickFixIndex = diagnosticIndex; navigationNote = nil
        case .failure(let why): quickFix = nil; navigationNote = "Fix not applied: " + why.text
        }
    }

    /// "Apply" in the preview sheet: one grouped replacement through the
    /// existing pendingEdit path (single undoable edit, never automatic).
    func applyQuickFix() {
        guard let preview = quickFix else { return }
        let grouped = preview.apply()
        guard grouped.path == activePath, grouped.matches(activeText) else {
            navigationNote = "Fix not applied: the document changed since the preview; open Fix… again."
            quickFix = nil; return
        }
        pendingEdit = .init(path: grouped.path, nsRange: grouped.nsRange, text: grouped.text,
                            token: (pendingEdit?.token ?? 0) + 1)
        navigationNote = "Applied: \(preview.summary) (undo with ⌘Z)"
        quickFix = nil
    }

    /// Asks the helper once per result; the cache is read by `editorMarkReport`.
    private func fetchExplanations(for result: RuntimeV1.CompileResult, id: String, documents: [RuntimeV1.Document]) {
        guard explanations[id] == nil else { return }
        if explanationClient == nil || explanationClient?.isRunning == false {
            guard let exe = ExplanationClient.locate() else { explanationStatus = nil; return }
            explanationClient = try? ExplanationClient(executable: exe) { [weak self] e in self?.explanationStatus = "flashtex-explain: " + e }
        }
        explanationClient?.explain(result: result, documents: documents, supported: Completion.defaultSupported) { [weak self] outcome in
            guard let self else { return }
            switch outcome {
            case .success(let list):
                self.explanations.store(list, for: id)
                self.editorMarksCache = nil // re-attach on the next read
                self.explanationStatus = nil
            case .failure(let f):
                self.explanationStatus = f.text // shown in the footer; marks stay as they are
            }
        }
    }

    // MARK: caret sync (source -> preview)

    /// UTF-8 byte offset of the editor caret in `activeText`, or nil when the
    /// UTF-16 caret is out of range for the buffer.
    var caretByte: Int? {
        CaretSync.byteOffset(ofCaretUTF16: caretUTF16, in: activeText) // mid-surrogate carets rounded, never split
    }

    /// Preview items under the caret, as `page number -> item indices`.
    /// Empty when there is no result or the caret maps to nothing.
    var caretItems: [Int: Set<Int>] { exactCaretItems } // CaretSync.swift: O(log n) index, memoized per result

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
            if env["FLASHTEX_AUTOATTACH"] == "1", let helper = env["FLASHTEX_PREVIEW_CONTROLLER"],
               FileManager.default.isExecutableFile(atPath: helper) {
                // Durable helper route (STDIO.md): the helper owns the ledger and the
                // compiler; the direct worker is not attached alongside it.
                attachController(at: URL(fileURLWithPath: helper))
            } else if env["FLASHTEX_AUTOATTACH"] != "0", (env["FLASHTEX_AUTOATTACH"] == "1" || hasBundled),
               Self.locateCompiler() != nil {
                attachDiscoveredWorker()
                compile()
            }
            let bundledBridge = Bundle.main.executableURL?.deletingLastPathComponent()
                .appendingPathComponent("flashtex-bridge").path
            let hasBundledBridge = bundledBridge.map { FileManager.default.isExecutableFile(atPath: $0) } ?? false
            if env["FLASHTEX_AUTOATTACH"] != "0", (env["FLASHTEX_AUTOATTACH"] == "1" || hasBundledBridge),
               BridgeClient.locateBridge() != nil {
                attachDiscoveredBridge()
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
            // Seed the editor from `request` or, for `<name>-result.json`, a
            // sibling `<name>-request.json` (e.g. Samples/multipage-*.json).
            var candidates: [URL] = []
            if let request { candidates.append(request) }
            if let sibling = Self.siblingRequestURL(forResult: result) { candidates.append(sibling) }
            let req = candidates.lazy.compactMap({ url -> RuntimeV1.Envelope<RuntimeV1.CompileRequest>? in
                guard let data = try? Data(contentsOf: url) else { return nil }
                return try? RuntimeV1.decodeCompileRequest(data)
            }).first
            // A fixture is checked against its request's capabilities when that
            // request is available, else against the set it declares itself.
            let requested = req?.payload.layoutCapabilities ?? res.payload.layoutCapabilities ?? []
            if let violation = LayoutNegotiation.violation(in: res.payload, requested: requested) {
                loadError = "Rejected \(result.lastPathComponent): \(violation)"
                return
            }
            self.result = res.payload
            self.resultID = res.id
            self.fixtureURL = result
            self.previewSource = .fixture
            self.historicalPreview = nil
            bindLayout(of: res.payload, requested: requested)
            if let req {
                documents = req.payload.documents
                activePath = req.payload.entryPath
                editorRevision = req.payload.revision
                compiledDocuments = Dictionary(uniqueKeysWithValues: req.payload.documents.map { ($0.path, $0.text) })
                fetchExplanations(for: res.payload, id: res.id, documents: req.payload.documents)
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
        historicalPreview = nil
        negotiation = .legacy
        fontSubstitutions = []
        layoutDiagnostics = []
        selection = nil
        anchor = nil
        editorRevision += 1
        bridgeDocumentReplaced()
    }

    func updateActiveText(_ text: String) {
        guard let i = documents.firstIndex(where: { $0.path == activePath }) else { return }
        guard !documents[i].text.sameBytes(as: text) else { return }
        let old = documents[i].text, base = editorRevision
        documents[i].text = text
        editorRevision += 1
        TypingBench.shared.noteRevision(editorRevision) // keystroke -> paint instrumentation
        scheduleAutoCompile()
        bridgeTextChanged(path: activePath, old: old, new: text, base: base, revision: editorRevision)
    }

    private func scheduleAutoCompile() {
        guard autoCompile, workerAttached else { return }
        if controllerAttached { controllerSubmitEdit(); return }
        debounce?.cancel()
        if Self.debounceInterval == 0 { compile(); return }
        let item = DispatchWorkItem { [weak self] in self?.compile() }
        debounce = item
        DispatchQueue.main.asyncAfter(deadline: .now() + Self.debounceInterval, execute: item)
    }

    // MARK: navigation (preview -> source)

    /// Converts the contract's UTF-8 byte range to a UTF-16 selection in the
    /// matching document and asks the editor to select it.
    /// Preview/diagnostic navigation: see `navigateExactly` (Navigation.swift)
    /// for the byte-exact staleness and cluster guarantees.
    func navigate(to source: RuntimeV1.SourceRange?) {
        navigateExactly(to: source, expectedText: nil)
    }

    /// `expectedText` (an item's text) annotates generated text after a rebase.
    func navigate(to source: RuntimeV1.SourceRange, expectedText: String?) {
        navigateExactly(to: source, expectedText: expectedText)
    }

    // MARK: worker transport (runtime v1 JSON Lines)

    /// Finds a built FT-002 worker: $FLASHTEX_COMPILER, then
    /// crates/compiler/target/{release,debug}/flashtex-compiler under the repo root.
    /// Finds a built render pipeline (`flashtex-render`, crates/render-pipeline:
    /// a drop-in runtime-v1 producer measured with Latin Modern metrics, so
    /// the preview draws Computer Modern-style text): $FLASHTEX_RENDER, the
    /// app bundle, then crates/render-pipeline/target/{release,debug}.
    static func locateRenderPipeline() -> URL? {
        let fm = FileManager.default
        if let env = ProcessInfo.processInfo.environment["FLASHTEX_RENDER"], fm.isExecutableFile(atPath: env) {
            return URL(fileURLWithPath: env)
        }
        if let bundled = Bundle.main.executableURL?.deletingLastPathComponent().appendingPathComponent("flashtex-render"),
           fm.isExecutableFile(atPath: bundled.path) {
            return bundled
        }
        guard let root = locateRepoRoot() else { return nil }
        for profile in ["release", "debug"] {
            let url = root.appendingPathComponent("crates/render-pipeline/target/\(profile)/flashtex-render")
            if fm.isExecutableFile(atPath: url.path) { return url }
        }
        return nil
    }

    /// File > Attach Render Pipeline: the Latin Modern producer when it is built.
    @discardableResult
    func attachDiscoveredRenderPipeline() -> Bool {
        guard let url = Self.locateRenderPipeline() else {
            workerStatus = "no built flashtex-render found (build crates/render-pipeline or set FLASHTEX_RENDER)"
            return false
        }
        attachWorker(at: url)
        compile()
        return true
    }

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
        workerRelaunchTimes = []
        launchWorker(at: url, arguments: arguments)
    }

    private func launchWorker(at url: URL, arguments: [String]) {
        do {
            worker = try WorkerClient(executable: url, arguments: arguments, transcript: transcript) { [weak self] event in
                self?.handle(event)
            }
            workerLaunch = (url, arguments)
            workerStatus = "attached: \(url.lastPathComponent)"
            PreviewFonts.producerFace = PreviewFonts.face(forProducer: url.lastPathComponent)
            log("launched \(url.path) (preview face: \(PreviewFonts.active.rawValue))")
        } catch {
            workerStatus = "launch failed: \(error.localizedDescription)"
        }
    }

    func detachWorker() {
        debounce?.cancel()
        workerRelaunchWork?.cancel()
        workerRelaunchWork = nil
        workerLaunch = nil
        worker?.terminate()
        worker = nil
        inFlightRequests.removeAll()
        latestRequestID = nil
        inFlightRevision = nil
        compileQueued = false
        if previewSource != .fixture { workerStatus = "no worker attached" }
    }

    /// Sends the current buffers as a `compile` request. Never blocks the UI.
    ///
    /// Edits coalesce behind the request in flight (the newest buffer goes out
    /// when it returns). A layout-capability switch does not wait: it is sent
    /// at once under a new id — at the same revision when the buffer has not
    /// changed — and the older request's reply is then classified stale.
    func compile() {
        if controllerAttached { controllerCompile(); return }
        guard let worker, worker.isRunning else {
            workerStatus = "no worker attached"
            return
        }
        debounce?.cancel()
        let capabilities = requestedLayoutCapabilities
        if let latestID = latestRequestID, let latest = inFlightRequests[latestID] {
            guard latest.layoutCapabilities != capabilities else {
                compileQueued = true
                return
            }
            log("layout capability switch while \(latestID) is in flight: re-requesting revision \(editorRevision) under \(LayoutNegotiation.describe(capabilities))")
        } else if let current = result, previewSource != .fixture, current.revision == editorRevision,
                  negotiation.requested == capabilities {
            return // buffers and capability set unchanged since the applied result
        }
        let id = "mac-\(nextRequestID)"
        nextRequestID += 1
        let request = RuntimeV1.CompileRequest(
            projectId: result?.projectId ?? "demo",
            revision: editorRevision,
            entryPath: project.entryPath, // the entry stays first whichever document is being edited
            documents: documents,
            layoutCapabilities: capabilities.isEmpty ? nil : capabilities)
        do {
            if TypingBench.isBenchActive { FlashTeXLog.write("compile: sending revision \(editorRevision) at \(MonotonicClock.nowNs())") }
            try worker.send(request, id: id)
            inFlightRequests[id] = InFlight(projectId: request.projectId, revision: request.revision,
                                            documents: documents, sentAt: Date(), layoutCapabilities: capabilities)
            latestRequestID = id
            inFlightRevision = editorRevision
            workerStatus = "compiling revision \(editorRevision) (\(id))…"
        } catch {
            workerStatus = "send failed: \(error.localizedDescription)"
        }
    }

    /// Delivers an event as if the worker had sent it; a `compile_result` or
    /// `error` is also written to the transcript (re-encoded) so hand-delivered
    /// asynchronous replies are visible to `check_runtime.py`.
    func handleForTesting(_ event: WorkerClient.Event) {
        if let transcript {
            switch event {
            case .result(let env):
                if let line = try? RuntimeV1.encodeLine(env) { transcript.record(line) }
            case .error(let id, let message):
                let env = RuntimeV1.Envelope(protocolVersion: RuntimeV1.protocolVersion, id: id, type: "error",
                                             payload: RuntimeV1.ErrorPayload(message: message))
                if let line = try? RuntimeV1.encodeLine(env) { transcript.record(line) }
            default: break
            }
        }
        handle(event)
    }

    /// `inFlightRevision` tracks the latest request while it is unanswered.
    private func refreshInFlightRevision() {
        inFlightRevision = latestRequestID.flatMap { inFlightRequests[$0]?.revision }
    }

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
            inFlightRequests.removeValue(forKey: env.id)
            refreshInFlightRevision()
            guard incoming.projectId == sent.projectId, incoming.revision == sent.revision else {
                let msg = "compile_result \(env.id) reports project \(incoming.projectId) revision \(incoming.revision); request was project \(sent.projectId) revision \(sent.revision)"
                log("rejected mismatched " + msg)
                workerStatus = "protocol violation: " + msg
                return
            }
            // Layout capabilities are per request: the reply may only claim what
            // this request asked for, and may only carry negotiated shapes.
            if let violation = LayoutNegotiation.violation(in: incoming, requested: sent.layoutCapabilities) {
                let msg = "compile_result \(env.id): \(violation)"
                log("rejected " + msg)
                workerStatus = "protocol violation: " + msg
                return
            }
            // Superseded: a newer request went out after this one (capability
            // switch, or an edit sent after an error). Valid, but stale for the
            // preview — check_runtime.py's `stale_ignore` — so it never changes
            // the result or the renderer mode.
            if let latest = latestRequestID, latest != env.id {
                log("ignored stale compile_result \(env.id) (revision \(incoming.revision), \(LayoutNegotiation.describe(sent.layoutCapabilities))): superseded by \(latest)")
                return
            }
            // Contract: never replace a newer preview with an older revision.
            if let current = result, previewSource != .fixture, incoming.revision < current.revision {
                log("ignored stale compile_result revision \(incoming.revision) < \(current.revision)")
                return
            }
            result = incoming
            resultID = env.id
            previewSource = .worker(worker?.executable.lastPathComponent ?? "worker")
            historicalPreview = nil
            bindLayout(of: incoming, requested: sent.layoutCapabilities)
            compiledDocuments = Dictionary(uniqueKeysWithValues: sent.documents.map { ($0.path, $0.text) })
            fetchExplanations(for: incoming, id: env.id, documents: sent.documents)
            let ms = Date().timeIntervalSince(sent.sentAt) * 1000
            TypingBench.shared.noteCompile(revision: incoming.revision, ms: ms)
            if TypingBench.isBenchActive { FlashTeXLog.write("compile: applied revision \(incoming.revision) at \(MonotonicClock.nowNs())") }
            lastLatencyMs = ms
            latenciesMs.append(ms)
            if latenciesMs.count > 100 { latenciesMs.removeFirst(latenciesMs.count - 100) }
            let latencyText = String(format: " in %.0f ms", ms)
            workerStatus = "revision \(incoming.revision): \(incoming.status.rawValue), \(incoming.diagnostics.count) diagnostics\(latencyText)"
            selection = nil
            if compileQueued {
                compileQueued = false
                compile() // no-op when buffers and capability set are unchanged
            }
        case .displayList(let id, let line):
            receiveDisplayListV2(id: id, line: line) // negotiated live v2 frame (PreviewV2View.swift)
        case .error(let id, let message):
            inFlightRequests.removeValue(forKey: id)
            refreshInFlightRevision()
            if inFlightRevision == nil { compileQueued = false }
            workerStatus = "worker error for \(id): \(message)"
            log("error \(id): \(message)")
        case .protocolViolation(let message):
            workerStatus = "protocol violation: \(message)"
            log("protocol violation: \(message)")
        case .stderr(let text):
            log(text.trimmingCharacters(in: .whitespacesAndNewlines))
        case .exited(let code):
            inFlightRequests.removeAll()
            latestRequestID = nil
            inFlightRevision = nil
            compileQueued = false
            workerStatus = "worker exited (\(code))"
            log("worker exited with status \(code)")
            worker = nil
            scheduleWorkerRelaunch(afterExit: code)
        }
    }

    /// An abnormal exit of a worker we launched relaunches the same executable
    /// after a short backoff, at most `maxWorkerRelaunches` times per minute;
    /// beyond that the exit is left visible for the user. A clean exit (0) or
    /// an explicit detach never relaunches. The preview keeps the last result.
    private func scheduleWorkerRelaunch(afterExit code: Int32) {
        guard code != 0, let launch = workerLaunch else { return }
        let now = Date()
        workerRelaunchTimes = workerRelaunchTimes.filter { now.timeIntervalSince($0) < 60 }
        guard workerRelaunchTimes.count < Self.maxWorkerRelaunches else {
            workerStatus = "worker exited (\(code)); not relaunched: \(Self.maxWorkerRelaunches) relaunches in the last minute — File > Attach Built Compiler to retry"
            log("worker relaunch limit reached")
            return
        }
        let delay = Self.workerRelaunchDelays[min(workerRelaunchTimes.count, Self.workerRelaunchDelays.count - 1)]
        workerRelaunchTimes.append(now)
        workerStatus = String(format: "worker exited (%d); relaunching in %.1f s", code, delay)
        log("relaunching \(launch.url.lastPathComponent) in \(delay) s (attempt \(workerRelaunchTimes.count))")
        let item = DispatchWorkItem { [weak self] in
            guard let self, self.worker == nil, let launch = self.workerLaunch else { return }
            self.launchWorker(at: launch.url, arguments: launch.arguments)
            self.workerRelaunchCount += 1
            if self.workerAttached {
                self.log("relaunched \(launch.url.lastPathComponent)")
                if self.autoCompile { self.compile() }
            }
        }
        workerRelaunchWork = item
        DispatchQueue.main.asyncAfter(deadline: .now() + delay, execute: item)
    }

    /// Documents (path → text) the current result was compiled from.
    func setCompiledDocuments(_ docs: [String: String]) { compiledDocuments = docs }

    /// Records one producer round trip for the status line's latency summary.
    func recordLatency(_ ms: Double) {
        lastLatencyMs = ms
        latenciesMs.append(ms)
        if latenciesMs.count > 100 { latenciesMs.removeFirst(latenciesMs.count - 100) }
    }

    func log(_ line: String) {
        FlashTeXLog.write(line)
        workerLog.append(line)
        if workerLog.count > 200 { workerLog.removeFirst(workerLog.count - 200) }
    }

    // MARK: capture review and insertion

    /// Pins the current caret as the insertion destination (`destination_id`).
    func pinAnchorAtCaret() {
        if let why = historicalRefusal(of: "pinning an insertion point") { captureNote = why; return }
        guard let anchor = Insertion.makeAnchor(id: "mac-anchor-\(nextAnchorNumber)", path: activePath,
                                                text: activeText, caretUTF16: caretUTF16, revision: editorRevision)
        else { captureNote = "Caret position is not valid."; return }
        nextAnchorNumber += 1
        self.anchor = anchor
        captureNote = "Pinned \(anchor.id) at \(anchor.path) byte \(anchor.byteOffset) (revision \(anchor.revision))."
        bridgePin(anchor)
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
        bridgeReject(captureId: proposal.captureId)
    }

    enum ApproveOutcome: Equatable { case inserted(byteOffset: Int), needsReselection(String), duplicate, noAnchor, refused(String) }

    /// Applies one undoable edit after explicit approval. `latex` may have been
    /// edited by the reviewer. Returns what happened so the UI can explain it.
    /// Bridge captures go through `approveBridgeProposal` (prepared edits are
    /// verified against the bridge), never through this local path.
    @discardableResult
    func approveProposal(_ proposal: RuntimeV1.CaptureProposal, latex: String) -> ApproveOutcome {
        guard !appliedCaptureIDs.contains(proposal.captureId) else {
            rejectProposal(proposal); captureNote = "Capture \(proposal.captureId) already inserted."; return .duplicate
        }
        guard !isBridgeCapture(proposal.captureId) else {
            captureNote = "Capture \(proposal.captureId) belongs to the bridge; approve it through the bridge review."
            return .refused("bridge capture")
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
