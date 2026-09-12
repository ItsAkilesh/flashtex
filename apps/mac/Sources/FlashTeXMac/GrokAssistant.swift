import AppKit
import Combine
import Foundation
import FlashTeXProtocol

/// "Ask Grok": the editor's live-document assistant (Edit › Ask Grok…, ⌘⌥G,
/// the toolbar's Ask Grok button, the command palette, and "Fix with Grok" on a
/// Problems row). Unlike the capture-review sheet's Explain, which works on a
/// capture's shadow compile, this binds the assistant-context helper to the
/// LAST REAL COMPILE of the open document (request id / project / revision /
/// compiled sources exactly as `ShellModel` recorded them) and runs the same
/// stages `ProposalPreview.explain()` runs, reusing its decoders:
///
/// 1. `prepare` without destinations (probe) — learns which byte ranges the
///    helper supplied as snippets (the document head as `related`, 2 KiB
///    around each selected diagnostic);
/// 2. `prepare` with `destinations` — the selection clipped to those snippets
///    (or every snippet when there is no selection) — giving the bound,
///    hashed context;
/// 3. the provider: `flashtex-assistant-context --provider-session` with the
///    key only in that child's environment (`GrokProviderSession`), or a local
///    provider command, or nothing (the context is shown and sent nowhere);
/// 4. `review` (edits) / `validate` (explanation only) of the reply;
/// 5. on **Apply** only: `approve` with the exact review id → `approved_group`,
///    mapped onto the live buffer through `EditorDiagnostics.QuickFix` (byte
///    exact, rebased across edits made since the compile, refused when they
///    overlap) → ONE `ShellModel.PendingEdit`, applied by the editor as one
///    undoable step. Nothing is written before Apply.
///
/// The helper only allows edits inside the snippets it supplied, and a related
/// document's snippet is its first 2 KiB (`snippet(doc, 0)` in lib.rs): when
/// the selection lies outside every snippet the panel says so
/// (`coverageNote`) instead of silently truncating.
@MainActor
final class GrokAssistant: ObservableObject {
    typealias Configuration = ProposalPreview.ExplanationConfiguration
    typealias ByteRange = ProposalPreview.ByteRange
    typealias Stage = ProposalPreview.ExplanationStage
    typealias Failure = OneShotProcess.Failure

    /// Read-only copy of the model state one request binds to.
    struct Snapshot: Equatable {
        /// The last compile result and its request id (the binding).
        var resultID: String
        var result: RuntimeV1.CompileResult
        /// Every document as the compile saw it (path → text).
        var compiledDocuments: [String: String]
        var activePath: String
        /// The live buffer of the active document.
        var activeText: String
        var editorRevision: Int
        /// The editor selection in UTF-8 bytes of `activeText`; nil for a
        /// caret only (then the whole document is the subject).
        var selection: Range<Int>?
        var caretByte: Int?
        /// A diagnostic index the request must include (Fix with Grok).
        var pinnedDiagnostic: Int? = nil
    }

    /// The request shape (pure; `GrokAssistantTests` checks it).
    struct Request: Equatable {
        var requestId: String
        var projectId: String
        var compileRevision: Int
        var relatedPaths: [String]
        var selectedDiagnostics: [Int]
        var userInstruction: String
        /// "Selection: lines 12–18 of main.tex" / "Whole document: main.tex".
        var contextLine: String
        var diagnosticsLine: String
    }

    struct Edit: Equatable, Identifiable {
        var id: Int
        var range: ByteRange
        var removedText: String
        var replacement: String
        var relocated: Bool
    }

    struct Reply: Equatable {
        var contextId: String
        var text: String
        var edits: [Edit]
        /// Helper notes for edits it dropped (could not be located).
        var notes: [String]
        var reviewId: String?
        var model: String?
        var elapsed: TimeInterval
    }

    enum State: Equatable {
        case idle
        case unavailable(String)
        case preparing
        /// Context bound; no provider is enabled, so nothing was sent.
        case prepared(contextId: String, bytes: Int)
        case awaitingProvider
        case validating
        case ready(Reply)
        case approving(Reply)
        case applied(String)
        case failed(String)
        case cancelled

        var isInFlight: Bool {
            switch self {
            case .preparing, .awaitingProvider, .validating, .approving: return true
            default: return false
            }
        }
    }

    struct Job {
        var id: String
        var stage: Stage
        var snapshot: Snapshot
        var request: Request
        var projectId: String
        var revision: Int
        var sources: [[String: Any]]
        var helperRequest: [String: Any]
        var snippets: [ByteRange] = []
        var contextId: String?
        var allowedEdits: [ByteRange] = []
        var reply: Reply?
        var startedAt = Date()
    }

    @Published var shown = false
    @Published var instruction = ""
    @Published private(set) var state: State = .idle
    @Published private(set) var contextLine = ""
    @Published private(set) var diagnosticsLine = ""
    /// "The selection lies outside …" when edits cannot reach the selection.
    @Published private(set) var coverageNote: String?
    @Published private(set) var providerStartedAt: Date?
    @Published private(set) var lastRequest: Request?
    /// Set once the reply's edits were mapped onto the live buffer (before/after).
    @Published private(set) var applicationPreview: EditorDiagnostics.QuickFix.Preview?
    @Published private(set) var applicationRefusal: String?
    /// Diagnostic index pinned by "Fix with Grok" for the next request.
    var pinnedDiagnostic: Int?

    /// The configuration in force: the injected one (tests), else the
    /// environment/Preferences/Keychain as last refreshed (`refreshConfiguration`,
    /// called when the panel opens and when a request starts, so a key or model
    /// saved in Preferences is picked up without relaunching).
    var configuration: Configuration { injected ?? cached ?? refreshConfiguration() }
    /// Injected configuration (tests, or a caller that resolved one itself);
    /// nil resolves the environment.
    var injected: Configuration? { didSet { cached = nil } }
    private var cached: Configuration?
    private(set) var job: Job?
    private var process: OneShotProcess?
    private var grokSession: GrokProviderSession?
    private var nextId = 1
    private(set) var staleReplies = 0
    private(set) var childLaunches: [ProposalPreview.ChildLaunch] = []
    private(set) var lastGrokLaunch: GrokProviderSession.Launch?

    static let instructionPlaceholder = "e.g. convert this to an align environment / fix the error on this line / write the TikZ for a right triangle"
    /// The helper's own limit on `user_instruction`.
    static let maxInstructionBytes = 8192

    /// nil reads the environment lazily (`GrokAssistant.configuration()`);
    /// nothing is resolved at init, so creating a `ShellModel` touches no Keychain.
    init(configuration: Configuration? = nil) {
        injected = configuration
    }

    @discardableResult
    func refreshConfiguration() -> Configuration {
        let c = injected ?? Self.configuration()
        cached = c
        objectWillChange.send()
        return c
    }

    /// The review sheet's configuration with the Ask model substituted
    /// (`GrokCredential.askModel`): the fast non-reasoning model by default
    /// now that the helper relocates its misplaced edits (relocate-edits).
    @MainActor static func configuration(_ env: [String: String] = ProcessInfo.processInfo.environment,
                                         preferences: GrokPreferences = .shared,
                                         keychain: any GrokKeychainStore = SecItemKeychain.shared) -> Configuration {
        var c = Configuration.fromEnvironment(env, preferences: preferences, keychain: keychain)
        if c.grok != nil {
            let model = GrokCredential.askModel(environment: env, preferences: preferences)
            c.grok?.model = model
            if let s = env["FLASHTEX_ASSISTANT_TIMEOUT_S"], let t = TimeInterval(s), t > 0 {
                c.providerTimeout = min(t, 120)
            } else {
                c.providerTimeout = GrokCredential.providerTimeout(for: model)
            }
        }
        return c
    }

    var available: Bool { configuration.helper != nil }
    var inFlight: Bool { state.isInFlight }
    var grokLive: Bool { configuration.grok?.credential != nil }
    var providerEnabled: Bool { configuration.grok != nil || configuration.provider != nil }
    var model: String? { configuration.grok?.model }
    /// The status-bar wording: "Grok: on (model)" / "Grok: off".
    var providerStatusText: String { configuration.grokStatusText }
    var providerStatusHelp: String { configuration.grokStatusHelp }
    var runningChildProcessIdentifier: Int32? {
        if let s = grokSession, s.isRunning { return s.processIdentifier }
        guard let p = process, p.isRunning else { return nil }
        return p.processIdentifier
    }
    var canApply: Bool {
        if case .ready(let r) = state, r.reviewId != nil, applicationPreview != nil, applicationRefusal == nil, job != nil { return true }
        return false
    }
    var canAsk: Bool { available && !state.isInFlight && !instruction.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }

    var statusText: String {
        switch state {
        case .idle: return available ? "Not asked yet." : "unavailable: no flashtex-assistant-context helper found (build crates/assistant-context or set FLASHTEX_ASSISTANT_CONTEXT)"
        case .unavailable(let why): return "unavailable: \(why)"
        case .preparing: return "Preparing the bound context…"
        case .prepared(let id, let bytes): return "Context \(id.prefix(8)) prepared (\(bytes) bytes). No provider is enabled — nothing was sent. \(providerStatusHelp)"
        case .awaitingProvider:
            if let grok = configuration.grok { return "Asking Grok (\(grok.model))… up to \(Int(configuration.providerTimeout)) s; Cancel stops the helper." }
            return "Context handed to \(configuration.provider?.lastPathComponent ?? "the provider command"); waiting…"
        case .validating: return "Validating the reply against the bound context…"
        case .ready(let r):
            let edits = r.edits.isEmpty ? "no edits proposed" : "\(r.edits.count) proposed edit\(r.edits.count == 1 ? "" : "s")"
            let elapsed = String(format: "%.1f s", r.elapsed)
            return "\(r.model.map { "Grok (\($0))" } ?? "The provider") answered in \(elapsed): \(edits); nothing applied" + (r.reviewId != nil ? " — Apply inserts them as one undoable edit" : ".")
        case .approving: return "Asking the helper to approve the reviewed edit…"
        case .applied(let s): return s
        case .failed(let why): return "Failed: \(why)"
        case .cancelled: return "Cancelled."
        }
    }

    // MARK: request shaping (pure)

    /// The diagnostics whose span intersects the selection (≤16, plus the
    /// pinned one), or none when there is no selection.
    static func selectedDiagnostics(_ snapshot: Snapshot) -> [Int] {
        var picked: [Int] = []
        if let pinned = snapshot.pinnedDiagnostic, snapshot.result.diagnostics.indices.contains(pinned) { picked.append(pinned) }
        if let sel = snapshot.selection, !sel.isEmpty {
            for (i, d) in snapshot.result.diagnostics.enumerated() where !picked.contains(i) {
                guard let s = d.source, s.path == snapshot.activePath else { continue }
                let lo = s.startByte, hi = max(s.startByte, s.endByte)
                let intersects = lo == hi ? sel.lowerBound <= lo && lo <= sel.upperBound : lo < sel.upperBound && sel.lowerBound < hi
                if intersects { picked.append(i) }
            }
        }
        return Array(picked.prefix(16))
    }

    static func lines(of range: Range<Int>, in text: String) -> (Int, Int) {
        let a = EditorDiagnostics.lineNumber(ofByte: range.lowerBound, in: text) ?? 1
        let b = EditorDiagnostics.lineNumber(ofByte: max(range.lowerBound, range.upperBound - 1), in: text) ?? a
        return (a, max(a, b))
    }

    struct Refusal: Error, Equatable { var message: String; init(_ m: String) { message = m } }

    /// The request for `snapshot` and the user's text, or the reason none can be made.
    static func makeRequest(snapshot: Snapshot, instruction: String) -> Result<Request, Refusal> {
        let text = instruction.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return .failure(Refusal("type what Grok should do first")) }
        guard let compiled = snapshot.compiledDocuments[snapshot.activePath] else {
            return .failure(Refusal("\(snapshot.activePath) was not part of the last compile; compile first (⌘B)"))
        }
        guard compiled.sameBytes(as: snapshot.activeText) else {
            return .failure(Refusal("the buffer changed since the last compile (revision \(snapshot.result.revision)); wait for the next compile or press ⌘B"))
        }
        var snapshot = snapshot
        if let sel = snapshot.selection, sel.isEmpty { snapshot.selection = nil }
        let selected = selectedDiagnostics(snapshot)
        let frame: String
        let contextLine: String
        if let sel = snapshot.selection {
            let (a, b) = lines(of: sel, in: snapshot.activeText)
            contextLine = "Selection: line\(a == b ? " \(a)" : "s \(a)–\(b)") of \(snapshot.activePath)"
            frame = "The user selected bytes \(sel.lowerBound)..<\(sel.upperBound) (lines \(a)–\(b)) of \(snapshot.activePath) (editor revision \(snapshot.editorRevision)). "
                + "Propose edits only inside the selection. Reply with the exact removed_text as it appears in the source."
        } else {
            contextLine = "Whole document: \(snapshot.activePath)"
            let caret = snapshot.caretByte.map { "The caret is at byte \($0) (line \(EditorDiagnostics.lineNumber(ofByte: $0, in: snapshot.activeText) ?? 1)). " } ?? ""
            frame = "The user is editing \(snapshot.activePath) (editor revision \(snapshot.editorRevision)) with no selection. \(caret)"
                + "Propose edits only where the request needs them, inside the supplied source, quoting the exact removed_text."
        }
        var instruction = frame + "\nRequest: " + text
        if instruction.utf8.count > maxInstructionBytes {
            var bytes = Array(instruction.utf8.prefix(maxInstructionBytes))
            while !bytes.isEmpty, String(bytes: bytes, encoding: .utf8) == nil { bytes.removeLast() }
            instruction = String(decoding: bytes, as: UTF8.self)
        }
        let inDocument = snapshot.result.diagnostics.filter { $0.source?.path == snapshot.activePath }.count
        let diagnosticsLine = snapshot.selection.map { _ in "\(selected.count) diagnostic\(selected.count == 1 ? "" : "s") in range" }
            ?? (selected.isEmpty ? "\(inDocument) diagnostic\(inDocument == 1 ? "" : "s") in the document (select a range to include them)" : "1 diagnostic pinned")
        return .success(Request(requestId: snapshot.resultID, projectId: snapshot.result.projectId, compileRevision: snapshot.result.revision,
                                relatedPaths: [snapshot.activePath], selectedDiagnostics: selected, userInstruction: instruction,
                                contextLine: contextLine, diagnosticsLine: diagnosticsLine))
    }

    /// Snippet ranges of the active document the probe reply supplied.
    static func snippets(in payload: [String: Any], path: String) -> [ByteRange] {
        var out: [ByteRange] = []
        for r in payload["related"] as? [[String: Any]] ?? [] { out += ProposalPreview.ranges(in: [r["location"] ?? [:]]) }
        for d in payload["diagnostics"] as? [[String: Any]] ?? [] {
            if let s = d["snippet"] as? [String: Any] { out += ProposalPreview.ranges(in: [s["location"] ?? [:]]) }
        }
        var seen: [ByteRange] = []
        for s in out where s.path == path && !seen.contains(s) { seen.append(s) }
        return seen
    }

    /// Destinations for the second `prepare`: the selection clipped to each
    /// snippet that overlaps it (each destination must lie inside ONE snippet),
    /// or every snippet when there is no selection; at most 8. Empty with a
    /// selection means "explanation only" and `coverageNote` says why.
    static func destinations(snippets: [ByteRange], selection: Range<Int>?) -> [ByteRange] {
        guard let sel = selection, !sel.isEmpty else { return Array(snippets.prefix(8)) }
        var out: [ByteRange] = []
        for s in snippets {
            let lo = max(s.startByte, sel.lowerBound), hi = min(s.endByte, sel.upperBound)
            guard hi > lo else { continue }
            let r = ByteRange(path: s.path, startByte: lo, endByte: hi)
            if !out.contains(r) { out.append(r) }
        }
        return Array(out.prefix(8))
    }

    static func coverageNote(snippets: [ByteRange], selection: Range<Int>?, destinations: [ByteRange], path: String) -> String? {
        guard let sel = selection, !sel.isEmpty else { return nil }
        let covered = destinations.contains { $0.startByte <= sel.lowerBound && $0.endByte >= sel.upperBound }
        if covered { return nil }
        let windows = snippets.map { "bytes \($0.startByte)..<\($0.endByte)" }.joined(separator: ", ")
        let reach = destinations.isEmpty ? "Grok can explain but cannot propose edits there"
            : "edits are limited to bytes " + destinations.map { "\($0.startByte)..<\($0.endByte)" }.joined(separator: ", ")
        return "The selection (bytes \(sel.lowerBound)..<\(sel.upperBound)) is not fully inside the helper's context window of \(path) (\(windows.isEmpty ? "none" : windows): the first 2 KiB of the document plus 2 KiB around each selected diagnostic); \(reach)."
    }

    // MARK: driving

    /// Starts one request for `snapshot` with the panel's instruction.
    func ask(snapshot: Snapshot) {
        refreshConfiguration()
        guard let helper = configuration.helper else {
            state = .unavailable("no flashtex-assistant-context helper found (build crates/assistant-context or set FLASHTEX_ASSISTANT_CONTEXT)")
            return
        }
        var snapshot = snapshot
        if snapshot.pinnedDiagnostic == nil { snapshot.pinnedDiagnostic = pinnedDiagnostic }
        let request: Request
        switch Self.makeRequest(snapshot: snapshot, instruction: instruction) {
        case .failure(let why): state = .failed(why.message); return
        case .success(let r): request = r
        }
        cancelProcesses()
        applicationPreview = nil
        applicationRefusal = nil
        coverageNote = nil
        lastRequest = request
        contextLine = request.contextLine
        diagnosticsLine = request.diagnosticsLine
        let id = "ask-\(nextId)"
        nextId += 1
        let projectId = request.projectId
        let revision = max(1, request.compileRevision)
        let documents = snapshot.compiledDocuments.keys.sorted().map { RuntimeV1.Document(path: $0, text: snapshot.compiledDocuments[$0]!) }
        let sources = ProposalPreview.ledgerDocuments(documents, projectId: projectId, revision: revision)
        var identities: [String: Any] = [:]
        for doc in documents { identities[doc.path] = ["revision": revision, "sha256": SourceDigest.sha256Hex(doc.text)] }
        let compilerResult: Any
        do { compilerResult = try ProposalPreview.compilerResultJSON(snapshot.result, id: snapshot.resultID) } catch {
            state = .failed("could not encode the compile result: \(error.localizedDescription)"); return
        }
        let helperRequest: [String: Any] = [
            "operation": "prepare",
            "binding": ["request_id": snapshot.resultID, "project_id": projectId, "compile_revision": snapshot.result.revision, "sources": identities],
            "sources": sources,
            "compiler_result": compilerResult,
            "user_instruction": request.userInstruction,
            "related_paths": request.relatedPaths,
            "selected_diagnostics": request.selectedDiagnostics,
        ]
        job = Job(id: id, stage: .probe, snapshot: snapshot, request: request, projectId: projectId, revision: revision,
                  sources: sources, helperRequest: helperRequest)
        state = .preparing
        runHelper(helper, id: id, stage: .probe, request: helperRequest)
    }

    /// Apply: the helper's `approve` for the exact review id; on success the
    /// approved group is mapped onto the live buffer and handed to `deliver`
    /// as ONE grouped edit (the caller sets `ShellModel.pendingEdit`).
    func apply(activeText: String, deliver: @escaping (EditorDiagnostics.QuickFix.Grouped) -> Void) {
        guard let helper = configuration.helper, case .ready(let reply) = state, let reviewId = reply.reviewId, var job, job.stage == .done else {
            state = .failed("no reviewed edit is current; ask again first"); return
        }
        guard let preview = Self.applicationPreview(reply: reply, snapshot: job.snapshot, activeText: activeText) else {
            applicationRefusal = Self.applicationRefusalText(reply: reply, snapshot: job.snapshot, activeText: activeText)
            state = .failed("the edit no longer fits the buffer: \(applicationRefusal ?? "?")"); return
        }
        applicationPreview = preview
        job.stage = .approve
        job.helperRequest["operation"] = "approve"
        job.helperRequest["user_approved"] = true
        job.helperRequest["approved_review_id"] = reviewId
        job.helperRequest["current_sources"] = job.sources
        job.reply = reply
        self.job = job
        pendingDelivery = deliver
        state = .approving(reply)
        runHelper(helper, id: job.id, stage: .approve, request: job.helperRequest)
    }
    private var pendingDelivery: ((EditorDiagnostics.QuickFix.Grouped) -> Void)?

    /// A failure decided by the shell before any request (no compile result).
    func setFailed(_ why: String) {
        cancelProcesses()
        job = nil
        pendingDelivery = nil
        state = .failed(why)
    }

    func cancel() {
        cancelProcesses()
        job = nil
        pendingDelivery = nil
        switch state {
        case .idle, .unavailable: break
        default: state = .cancelled
        }
    }

    /// Clears the reply (the sheet's Close); a running request is cancelled.
    func reset() {
        cancel()
        state = .idle
        applicationPreview = nil
        applicationRefusal = nil
        coverageNote = nil
    }

    private func cancelProcesses() {
        process?.cancel(); process = nil
        grokSession?.cancel(); grokSession = nil
        providerStartedAt = nil
    }

    private func runHelper(_ helper: URL, id: String, stage: Stage, request: [String: Any]) {
        let data: Data
        do { data = try JSONSerialization.data(withJSONObject: request) } catch {
            fail(id, "could not encode the helper request: \(error.localizedDescription)"); return
        }
        guard data.count <= 16 * 1024 * 1024 else { fail(id, "helper request of \(data.count) bytes exceeds 16 MiB"); return }
        launch(helper, arguments: configuration.helperArguments, input: data, timeout: configuration.helperTimeout,
               limit: Configuration.helperOutputLimit, id: id, stage: stage)
    }

    private func launch(_ executable: URL, arguments: [String], input: Data, timeout: TimeInterval, limit: Int, id: String, stage: Stage) {
        let role: Configuration.ChildRole = stage == .provider ? .provider : .helper
        let environment = Configuration.childEnvironment(for: role)
        childLaunches.append(.init(role: role, executable: executable, arguments: arguments, environmentKeys: environment.keys.sorted(), stage: stage))
        if childLaunches.count > 64 { childLaunches.removeFirst(childLaunches.count - 64) }
        do {
            process = try OneShotProcess(executable: executable, arguments: arguments, input: input, timeout: timeout,
                                         maxOutputBytes: limit, environment: environment) { [weak self] result in
                self?.handleReply(id: id, stage: stage, result: result)
            }
        } catch {
            fail(id, "could not launch \(executable.lastPathComponent): \(error.localizedDescription)")
        }
    }

    private func launchGrok(_ grok: ProposalPreview.GrokProviderConfiguration, job: Job) {
        guard let credential = grok.credential else {
            fail(job.id, "Grok (xAI) is selected but no API key is present — add one in Preferences (⌘,) → Grok (xAI), or set XAI_API_KEY; the context was prepared and sent nowhere")
            return
        }
        guard let helper = grok.helper ?? configuration.helper else { fail(job.id, "helper vanished"); return }
        let sessionId = "mac-\(job.id)-\(UUID().uuidString.lowercased().prefix(8))".filter { $0.isLetter || $0.isNumber || $0 == "-" || $0 == "_" }
        var request = job.helperRequest
        request["operation"] = "prepare"
        request.removeValue(forKey: "response")
        request.removeValue(forKey: "current_sources")
        do {
            let environment = Configuration.childEnvironment(for: .grok)
            let session = try GrokProviderSession(helper: helper, model: grok.model, sessionId: String(sessionId),
                                                  credential: credential, environment: environment) { [weak self] result in
                self?.grokSession = nil
                self?.providerStartedAt = nil
                self?.handleReply(id: job.id, stage: .provider, result: result)
            }
            grokSession = session
            lastGrokLaunch = session.launch
            providerStartedAt = Date()
            childLaunches.append(.init(role: .grok, executable: helper, arguments: session.launch.arguments,
                                       environmentKeys: session.launch.environmentKeys, stage: .provider))
            session.run(prepareRequest: request, currentSources: job.sources, timeout: configuration.providerTimeout,
                        allocation: "flashtex-mac ask \(job.id) \(job.projectId)")
        } catch {
            fail(job.id, "could not launch \(helper.lastPathComponent) --provider-session: \(error.localizedDescription)")
        }
    }

    private func fail(_ id: String, _ why: String) {
        guard job?.id == id else { staleReplies += 1; return }
        job = nil
        process = nil
        pendingDelivery = nil
        state = .failed(why)
    }

    /// Every helper/provider completion lands here; a reply for another
    /// request or stage is counted and dropped.
    func handleReply(id: String, stage: Stage, result: Result<OneShotProcess.Output, Failure>) {
        guard var job, job.id == id, job.stage == stage, stage != .done else { staleReplies += 1; return }
        process = nil
        let output: OneShotProcess.Output
        switch result {
        case .success(let o): output = o
        case .failure(let f): fail(id, ProposalPreview.describe(f, stage: stage)); return
        }
        do {
            switch stage {
            case .probe:
                let payload = try ProposalPreview.preparedPayload(output.stdout)
                let snippets = Self.snippets(in: payload, path: job.snapshot.activePath)
                let destinations = Self.destinations(snippets: snippets, selection: job.snapshot.selection)
                coverageNote = Self.coverageNote(snippets: snippets, selection: job.snapshot.selection, destinations: destinations, path: job.snapshot.activePath)
                job.snippets = snippets
                job.helperRequest["destinations"] = destinations.map { ["path": $0.path, "start_byte": $0.startByte, "end_byte": $0.endByte] }
                job.stage = .prepare
                self.job = job
                guard let helper = configuration.helper else { throw ProposalPreview.ExplanationError("helper vanished") }
                runHelper(helper, id: job.id, stage: .prepare, request: job.helperRequest)
            case .prepare:
                let payload = try ProposalPreview.preparedPayload(output.stdout)
                guard let contextId = payload["context_id"] as? String, contextId.count == 64,
                      payload["compile_revision"] as? Int == job.request.compileRevision, payload["project_id"] as? String == job.projectId else {
                    throw ProposalPreview.ExplanationError("prepared context is not bound to \(job.request.requestId) revision \(job.request.compileRevision)")
                }
                guard let allowed = payload["allowed_edits"] as? [[String: Any]] else {
                    throw ProposalPreview.ExplanationError("prepared context has no explicit edit boundary")
                }
                job.contextId = contextId
                job.allowedEdits = ProposalPreview.ranges(in: allowed)
                let bytes = (try? JSONSerialization.data(withJSONObject: payload).count) ?? 0
                if let grok = configuration.grok {
                    job.stage = .provider; self.job = job
                    state = .awaitingProvider
                    launchGrok(grok, job: job)
                } else if let provider = configuration.provider {
                    job.stage = .provider; self.job = job
                    state = .awaitingProvider
                    providerStartedAt = Date()
                    launch(provider, arguments: configuration.providerArguments, input: (try? JSONSerialization.data(withJSONObject: payload)) ?? Data(),
                           timeout: configuration.providerTimeout, limit: Configuration.providerOutputLimit, id: job.id, stage: .provider)
                } else {
                    job.stage = .done; self.job = job
                    state = .prepared(contextId: contextId, bytes: bytes)
                }
            case .provider:
                providerStartedAt = nil
                guard let response = try? JSONSerialization.jsonObject(with: output.stdout) as? [String: Any] else {
                    throw ProposalPreview.ExplanationError("the provider's reply is not a JSON object (\(output.stdout.count) bytes)")
                }
                let hasEdits = !((response["edits"] as? [Any]) ?? []).isEmpty
                job.helperRequest["operation"] = hasEdits ? "review" : "validate"
                job.helperRequest["response"] = response
                job.helperRequest["current_sources"] = job.sources
                if hasEdits { job.helperRequest["explanation_request_id"] = job.id }
                job.stage = hasEdits ? .review : .validate
                self.job = job
                state = .validating
                guard let helper = configuration.helper else { throw ProposalPreview.ExplanationError("helper vanished") }
                runHelper(helper, id: job.id, stage: job.stage, request: job.helperRequest)
            case .validate, .review:
                guard let contextId = job.contextId else { throw ProposalPreview.ExplanationError("no prepared context") }
                let reply = try Self.reply(from: output.stdout, contextId: contextId, allowed: job.allowedEdits,
                                           expecting: stage == .review ? "proposal_review" : "validated_proposal",
                                           model: configuration.grok?.model, elapsed: Date().timeIntervalSince(job.startedAt))
                job.stage = .done
                job.reply = reply
                self.job = job
                applicationPreview = Self.applicationPreview(reply: reply, snapshot: job.snapshot, activeText: job.snapshot.activeText)
                applicationRefusal = reply.edits.isEmpty || applicationPreview != nil ? nil
                    : Self.applicationRefusalText(reply: reply, snapshot: job.snapshot, activeText: job.snapshot.activeText)
                state = .ready(reply)
            case .approve:
                guard let reply = job.reply, let reviewId = reply.reviewId, let preview = applicationPreview else {
                    throw ProposalPreview.ExplanationError("no reviewed edit")
                }
                let commandId = try Self.checkApprovedGroup(output.stdout, reply: reply, reviewId: reviewId, job: job)
                job.stage = .done
                self.job = job
                let grouped = preview.apply()
                pendingDelivery?(grouped)
                pendingDelivery = nil
                state = .applied("Applied review \(reviewId.prefix(8)) (\(commandId.prefix(18))…) as one undoable edit (⌘Z undoes it).")
            case .done:
                staleReplies += 1
            }
        } catch {
            fail(id, (error as? ProposalPreview.ExplanationError)?.message ?? error.localizedDescription)
        }
    }

    // MARK: decoding (pure)

    /// Decodes a `validated_proposal` / `proposal_review` reply bound to
    /// `contextId`; refuses `applied:true`, edits outside `allowed`, and a
    /// review without its id.
    static func reply(from data: Data, contextId: String, allowed: [ByteRange], expecting type: String,
                      model: String?, elapsed: TimeInterval) throws -> Reply {
        let obj: [String: Any]
        do { obj = try ProposalPreview.helperReply(data, type: type) } catch let e as ProposalPreview.ExplanationError {
            throw ProposalPreview.ExplanationError(e.message.replacingOccurrences(of: "helper refused:", with: "helper refused the provider's reply:"))
        }
        guard let payload = obj["payload"] as? [String: Any] else { throw ProposalPreview.ExplanationError("\(type) has no payload") }
        guard obj["applied"] as? Bool == false else { throw ProposalPreview.ExplanationError("helper claims the edit was applied; refusing (nothing is applied from here)") }
        guard payload["context_id"] as? String == contextId else { throw ProposalPreview.ExplanationError("validated proposal answers another context") }
        guard let text = payload["explanation"] as? String, !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            throw ProposalPreview.ExplanationError("validated proposal has no explanation")
        }
        var reviewId: String?
        if type == "proposal_review" {
            guard obj["requires_user_approval"] as? Bool == true, let id = obj["review_id"] as? String, id.count == 64 else {
                throw ProposalPreview.ExplanationError("proposal_review lacks review_id or requires_user_approval")
            }
            reviewId = id
        }
        var edits: [Edit] = []
        for (i, e) in (payload["edits"] as? [[String: Any]] ?? []).enumerated() {
            guard let range = ProposalPreview.ranges(in: [e["location"] ?? [:]]).first,
                  let removed = e["removed_text"] as? String, let replacement = e["replacement"] as? String else {
                throw ProposalPreview.ExplanationError("proposed edit \(i) is malformed")
            }
            guard allowed.contains(where: { $0.path == range.path && range.startByte >= $0.startByte && range.endByte <= $0.endByte }) else {
                throw ProposalPreview.ExplanationError("proposed edit \(i) is outside the allowed range")
            }
            edits.append(Edit(id: i, range: range, removedText: removed, replacement: replacement, relocated: e["relocated"] as? Bool ?? false))
        }
        if reviewId != nil, edits.isEmpty { throw ProposalPreview.ExplanationError("proposal_review carries no edits") }
        let notes = (payload["notes"] as? [String] ?? []).prefix(8).map { String($0.prefix(512)) }
        return Reply(contextId: contextId, text: text, edits: edits, notes: Array(notes), reviewId: reviewId, model: model, elapsed: elapsed)
    }

    /// The reply's edits as a catalogue-style suggestion so the existing
    /// quick-fix machinery (byte-exact, rebased across later edits) yields the
    /// before/after snippet and the one grouped edit.
    static func suggestion(reply: Reply, path: String) -> EditorDiagnostics.Explanation {
        let edits = reply.edits.map { EditorDiagnostics.Explanation.Edit(path: $0.range.path, startByte: $0.range.startByte, endByte: $0.range.endByte, replacement: $0.replacement) }
        let suggestion = EditorDiagnostics.Explanation.Suggestion(text: "Grok's proposed edit", confidence: "reviewed", edits: edits)
        return EditorDiagnostics.Explanation(catalogID: "grok-assistant", title: "Ask Grok", category: "assistant", severity: "info",
                                             message: reply.text, why: "", whatHappened: "", suggestions: [suggestion], context: nil)
    }

    static func applicationPreview(reply: Reply, snapshot: Snapshot, activeText: String) -> EditorDiagnostics.QuickFix.Preview? {
        guard !reply.edits.isEmpty else { return nil }
        if case .success(let p) = EditorDiagnostics.QuickFix.prepare(suggestion(reply: reply, path: snapshot.activePath), path: snapshot.activePath,
                                                                    in: activeText, compiledText: snapshot.compiledDocuments[snapshot.activePath]) {
            return p
        }
        return nil
    }

    static func applicationRefusalText(reply: Reply, snapshot: Snapshot, activeText: String) -> String? {
        guard !reply.edits.isEmpty else { return nil }
        if case .failure(let why) = EditorDiagnostics.QuickFix.prepare(suggestion(reply: reply, path: snapshot.activePath), path: snapshot.activePath,
                                                                      in: activeText, compiledText: snapshot.compiledDocuments[snapshot.activePath]) {
            return why.text
        }
        return nil
    }

    /// Checks the `approved_group` names this review/request/project/document,
    /// the compiled text's revision and SHA-256, and exactly the reviewed
    /// edits. Returns the command id.
    static func checkApprovedGroup(_ data: Data, reply: Reply, reviewId: String, job: Job) throws -> String {
        let obj = try ProposalPreview.helperReply(data, type: "approved_group")
        guard obj["applied"] as? Bool == false else { throw ProposalPreview.ExplanationError("helper claims the group was applied; refusing") }
        guard let payload = obj["payload"] as? [String: Any], let group = payload["group"] as? [String: Any] else {
            throw ProposalPreview.ExplanationError("approved_group has no group")
        }
        let path = job.snapshot.activePath
        guard payload["review_id"] as? String == reviewId, payload["request_id"] as? String == job.id,
              payload["project_id"] as? String == job.projectId, payload["path"] as? String == path else {
            throw ProposalPreview.ExplanationError("approved group names another review, request, project or document")
        }
        guard let commandId = group["command_id"] as? String, commandId == "assistant-\(reviewId)",
              let expectedRevision = group["expected_revision"] as? Int, let expectedSha = group["expected_sha256"] as? String,
              let rawEdits = group["edits"] as? [[String: Any]] else {
            throw ProposalPreview.ExplanationError("approved group is malformed")
        }
        let compiled = job.snapshot.compiledDocuments[path] ?? ""
        guard expectedRevision == job.revision, expectedSha == SourceDigest.sha256Hex(compiled) else {
            throw ProposalPreview.ExplanationError("approved group expects revision \(expectedRevision)/\(expectedSha.prefix(8)), not the compiled text")
        }
        var ranges: [ByteRange] = [], replacements: [String] = []
        for (i, e) in rawEdits.enumerated() {
            guard let s = e["start_byte"] as? Int, let en = e["end_byte"] as? Int, s <= en,
                  let removed = e["removed_text"] as? String, let replacement = e["replacement"] as? String else {
                throw ProposalPreview.ExplanationError("approved edit \(i) is malformed")
            }
            guard let r = compiled.rangeOfUTF8(start: s, end: en), String(compiled[r]).sameBytes(as: removed) else {
                throw ProposalPreview.ExplanationError("approved edit \(i) does not match the compiled text")
            }
            ranges.append(ByteRange(path: path, startByte: s, endByte: en)); replacements.append(replacement)
        }
        guard ranges == reply.edits.map(\.range), replacements == reply.edits.map(\.replacement) else {
            throw ProposalPreview.ExplanationError("approved group differs from the reviewed edits")
        }
        return commandId
    }
}

extension GrokCredential {
    /// Model for Ask Grok: `FLASHTEX_GROK_ASK_MODEL`, else `FLASHTEX_GROK_MODEL`,
    /// else the Preferences pick, else `defaultAskModel`.
    static let askModelVariable = "FLASHTEX_GROK_ASK_MODEL"
    /// The fast non-reasoning model: the helper now relocates its misplaced
    /// edits by `removed_text` (agent/mac-assistant-context/relocate-edits),
    /// which is what made it unusable for edits before. The review sheet keeps
    /// `defaultModel`.
    static let defaultAskModel = fastModel
    static func askModel(environment: [String: String] = ProcessInfo.processInfo.environment,
                         preferences: GrokPreferences = .shared) -> String {
        for candidate in [environment[askModelVariable], environment[modelVariable], preferences.model] {
            if let candidate, isValidModel(candidate) { return candidate }
        }
        return defaultAskModel
    }
}
