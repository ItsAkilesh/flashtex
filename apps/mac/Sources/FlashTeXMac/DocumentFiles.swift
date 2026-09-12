import AppKit
import CryptoKit
import ObjectiveC
import Observation
import UniformTypeIdentifiers
import FlashTeXProtocol

/// The on-disk file no longer matches what the editor last opened or saved.
/// Nothing was written; the buffer is kept. The UI resolves it explicitly
/// (`overwriteOnDisk`, `reloadFromDisk`, or keep editing).
struct DocumentConflict: Equatable {
    var url: URL
    var kind: ProjectFilesV1.ConflictKind
    /// Hash the editor last saw on disk (nil when it expected no file).
    var ours: String?
    /// Hash actually on disk now (nil when the file is gone).
    var theirs: String?
    var size: Int?
    var mtimeUnixMs: Int?
    /// Detected by the rooted helper (locked compare-and-replace) or by the
    /// direct Foundation path (best-effort read-compare-write, not locked).
    var viaHelper: Bool

    var summary: String {
        let what: String = switch kind {
        case .modifiedExternally: "was modified on disk since it was opened"
        case .deletedExternally: "was deleted on disk since it was opened"
        case .alreadyExists: "appeared on disk although this buffer was never saved there"
        case .modifiedDuringSave: "changed while it was being saved"
        }
        let by = viaHelper ? "" : " (direct file access: check is best-effort, not locked)"
        return "\(url.lastPathComponent) \(what)\(by); your buffer is kept unsaved. Overwrite, reload, or keep editing."
    }
}

/// File-layer state of the shell: which backend performs reads/saves, its
/// human-readable status, and the explicit conflict the UI must resolve.
/// Stored on the model as an associated object so `DocumentFiles.swift` stays a
/// self-contained extension file (parent may fold it into `ShellModel` as a
/// stored `var files = DocumentFilesState()`).
///
/// Backends: the rooted `flashtex-project-files` helper (one child process per
/// project directory; locked, symlink-refusing, hash-checked compare-and-replace)
/// or, when no helper binary exists, direct Foundation I/O with a best-effort
/// hash check. The helper is never bypassed once it exists: a helper failure
/// (timeout, crash, refusal) is reported and the buffer stays unsaved.
///
/// Helper calls are synchronous with a bounded wait (`helperTimeout`) because
/// `saveTex()` must answer the quit/open flows. A reply arriving after the wait
/// is *not* dropped: it is reconciled on the main actor (a late save receipt for
/// the same file marks that text as on disk; a late conflict is surfaced) so a
/// lost reply never desynchronizes the dirty state, and the helper is restarted
/// before the next request because its replies are strictly in order.
@MainActor
@Observable
final class DocumentFilesState {
    enum Backend: Equatable {
        case helper(URL)
        case direct(reason: String)
    }
    /// How the helper binary is chosen; tests pin a fake or disable it.
    enum HelperPolicy: Equatable {
        case discover
        case disabled(reason: String)
        case executable(URL, arguments: [String])
    }

    var policy: HelperPolicy = .discover
    /// Backend of the most recent file operation (nil before any).
    private(set) var backend: Backend?
    private(set) var status = "no file operation yet"
    var conflict: DocumentConflict?
    /// Result of the last `status` query for the open document.
    private(set) var lastDiskState: ProjectFilesV1.DiskState?
    /// Bounded wait for one helper reply.
    var helperTimeout: TimeInterval = 10
    /// Notes about replies that arrived after their wait expired (tests read these).
    private(set) var lateReplies: [String] = []
    private(set) var helperExits = 0
    private(set) var helperRestarts = 0

    @ObservationIgnored fileprivate var client: ProjectFilesClient?
    @ObservationIgnored private let replyQueue = DispatchQueue(label: "flashtex.project-files.replies")

    enum Outcome<T> {
        case reply(T)
        case failed(LineProcessFailure)
        case timedOut(TimeInterval)
    }

    enum ReadResult: Equatable {
        case text(String)
        case missing
        case failed(String)
    }

    struct StatusFailure: Error, Equatable {
        var reason: String
        init(_ reason: String) { self.reason = reason }
    }

    enum SaveResult: Equatable {
        case saved(sha256: String)
        case conflict(DocumentConflict)
        case failed(String)
    }

    private enum Acquired {
        case client(ProjectFilesClient)
        case direct(String)
        case unavailable(String)
    }

    var helperRunning: Bool { client?.isRunning == true }
    var usesHelper: Bool { if case .helper = backend { true } else { false } }
    var helperRoot: URL? { client?.root }

    /// Stops the helper process (if any); the next operation respawns it.
    func detachHelper() {
        client?.terminate()
        client = nil
    }

    private func note(_ s: String) {
        status = s
        FlashTeXLog.write("files: " + s)
    }

    private func executable() -> (URL, [String])? {
        switch policy {
        case .disabled: return nil
        case .executable(let url, let args): return (url, args)
        case .discover: return ProjectFilesClient.locate().map { ($0, []) }
        }
    }

    /// The project root the helper is bound to for `url`: its directory with
    /// symlinks resolved (the helper refuses a symlinked root; components above
    /// the root are the caller's choice).
    static func root(for url: URL) -> URL {
        url.standardizedFileURL.deletingLastPathComponent().resolvingSymlinksInPath()
    }

    private func acquire(for url: URL) -> Acquired {
        if case .disabled(let reason) = policy {
            backend = .direct(reason: reason)
            return .direct(reason)
        }
        guard let (exe, args) = executable() else {
            let reason = "no flashtex-project-files helper (set FLASHTEX_PROJECT_FILES or build crates/project-files); direct file access with best-effort conflict checks"
            backend = .direct(reason: reason)
            return .direct(reason)
        }
        let root = Self.root(for: url)
        if let client {
            let sameBinary = client.executable == exe && client.arguments == args
            if client.isRunning, sameBinary, client.root == root, client.outstanding == 0 { return .client(client) }
            let why = !client.isRunning ? "exited" : !sameBinary ? "helper binary changed" : client.root != root ? "bound to \(client.root.path)" : "\(client.outstanding) unanswered request(s)"
            FlashTeXLog.write("files: restarting helper (\(why))")
            client.terminate()
            self.client = nil
            helperRestarts += 1
        }
        do {
            let client = try ProjectFilesClient(executable: exe, arguments: args, root: root, queue: replyQueue) { [weak self] event in
                DispatchQueue.main.async { [weak self] in
                    MainActor.assumeIsolated { self?.handle(event) }
                }
            }
            self.client = client
            backend = .helper(exe)
            return .client(client)
        } catch {
            let reason = "helper \(exe.lastPathComponent) failed to launch: \(error.localizedDescription)"
            backend = .helper(exe)
            note(reason)
            return .unavailable(reason)
        }
    }

    private func handle(_ event: ProjectFilesClient.Event) {
        switch event {
        case .stderr(let s): FlashTeXLog.write("files helper stderr: " + s.trimmingCharacters(in: .newlines))
        case .protocolViolation(let s): note("helper protocol violation: \(s)")
        case .unsolicited(let id, let type, _): FlashTeXLog.write("files helper: unsolicited \(type) reply \(id ?? "-")")
        case .exited(let code):
            helperExits += 1
            note("project-files helper exited (\(code)); it restarts on the next file operation")
            if client?.isRunning != true { client = nil }
        }
    }

    /// Sends one request and waits at most `helperTimeout` for its reply on the
    /// calling (main) thread. A reply after the deadline goes to `late` on the
    /// main actor instead of being dropped.
    private func roundTrip<T>(_ send: (@escaping (Result<T, LineProcessFailure>) -> Void) -> Void,
                              late: @escaping @MainActor (Result<T, LineProcessFailure>) -> Void) -> Outcome<T> {
        let box = ReplyBox<T>()
        let sem = DispatchSemaphore(value: 0)
        let timeout = helperTimeout
        send { result in
            let deliverLate: Bool = box.lock.withLock {
                if box.settled { return true }
                box.result = result
                box.settled = true
                return false
            }
            if deliverLate {
                DispatchQueue.main.async { MainActor.assumeIsolated { late(result) } }
            } else {
                sem.signal()
            }
        }
        _ = sem.wait(timeout: .now() + timeout)
        let result: Result<T, LineProcessFailure>? = box.lock.withLock {
            defer { box.settled = true }
            return box.result
        }
        switch result {
        case .success(let v): return .reply(v)
        case .failure(let f): return .failed(f)
        case nil: return .timedOut(timeout)
        }
    }

    private final class ReplyBox<T> {
        let lock = NSLock()
        var settled = false
        var result: Result<T, LineProcessFailure>?
    }

    // MARK: operations

    func read(_ url: URL) -> ReadResult {
        switch acquire(for: url) {
        case .direct(let reason):
            note(reason)
            return readDirect(url)
        case .unavailable(let reason):
            return .failed(reason)
        case .client(let client):
            let name = url.lastPathComponent
            let outcome: Outcome<ProjectFilesV1.Read> = roundTrip({ done in
                client.send({ ProjectFilesV1.ReadRequest(id: $0, path: name) }, as: ProjectFilesV1.Read.self, completion: done)
            }, late: { [weak self] result in
                self?.lateReplies.append("read \(name): \(Self.describe(result))")
                self?.note("late reply to read \(name) arrived after \(Int(self?.helperTimeout ?? 0)) s; ignored (buffer untouched)")
            })
            switch outcome {
            case .reply(let r):
                note("rooted helper \(client.executable.lastPathComponent) at \(client.root.path)")
                guard r.exists, let text = r.text else { return .missing }
                return .text(text)
            case .failed(let f):
                note("helper read of \(name) failed: \(f.text)")
                return .failed(f.text)
            case .timedOut(let t):
                note("helper did not answer read of \(name) within \(Int(t)) s; buffer untouched")
                return .failed("no reply from the project-files helper within \(Int(t)) s")
            }
        }
    }

    /// Compare-and-replace save. `expected` is what the editor last saw on disk;
    /// `force` overwrites regardless (only after an explicit user decision).
    func save(_ url: URL, text: String, expected: ProjectFilesV1.Expected, force: Bool,
              lateReceipt: @escaping @MainActor (String) -> Void = { _ in }) -> SaveResult {
        switch acquire(for: url) {
        case .direct(let reason):
            note(reason)
            return saveDirect(url, text: text, expected: expected, force: force)
        case .unavailable(let reason):
            return .failed(reason)
        case .client(let client):
            let name = url.lastPathComponent
            let sent = SourceDigest.sha256Hex(text)
            let outcome: Outcome<ProjectFilesV1.SaveOutcome> = roundTrip({ done in
                client.send({ ProjectFilesV1.SaveRequest(id: $0, path: name, text: text, expected: expected.wire, force: force) },
                            as: ProjectFilesV1.SaveOutcome.self, completion: done)
            }, late: { [weak self] result in
                guard let self else { return }
                lateReplies.append("save \(name): \(Self.describe(result))")
                switch result {
                case .success(.saved(let receipt)) where receipt.sha256 == sent:
                    note("late save receipt for \(name): the text sent \(Int(helperTimeout)) s ago is on disk (verified hash)")
                    lateReceipt(sent)
                case .success(.saved(let receipt)):
                    note("late save receipt for \(name) reports hash \(receipt.sha256.prefix(12)), not the text sent; buffer kept unsaved")
                case .success(.conflict(let c)):
                    conflict = Self.conflict(url: url, c, viaHelper: true)
                    note("late reply for \(name): conflict; buffer kept unsaved")
                case .failure(let f):
                    note("late reply for \(name): \(f.text); buffer kept unsaved")
                }
            })
            switch outcome {
            case .reply(.saved(let receipt)):
                guard receipt.sha256 == sent else {
                    note("helper receipt hash for \(name) does not match the text sent; treated as not saved")
                    return .failed("save receipt hash mismatch for \(name)")
                }
                conflict = nil
                lastDiskState = .unchanged
                note("saved \(name) via rooted helper (\(receipt.bytes) bytes, sha256 \(receipt.sha256.prefix(12)))")
                return .saved(sha256: receipt.sha256)
            case .reply(.conflict(let c)):
                let conflict = Self.conflict(url: url, c, viaHelper: true)
                self.conflict = conflict
                lastDiskState = c.kind == .deletedExternally ? .deleted : .modified
                note(conflict.summary)
                return .conflict(conflict)
            case .failed(let f):
                note("helper save of \(name) failed: \(f.text); buffer kept unsaved")
                return .failed(f.text)
            case .timedOut(let t):
                note("helper did not confirm the save of \(name) within \(Int(t)) s; buffer kept unsaved (a late receipt will be reconciled)")
                return .failed("no save receipt from the project-files helper within \(Int(t)) s")
            }
        }
    }

    /// Asks whether `url` still matches `expectedSha256` (nil: the editor expects no file).
    func diskStatus(_ url: URL, expectedSha256: String?) -> Result<ProjectFilesV1.Status, StatusFailure> {
        let name = url.lastPathComponent
        let result: Result<ProjectFilesV1.Status, StatusFailure>
        switch acquire(for: url) {
        case .direct(let reason):
            note(reason)
            result = .success(statusDirect(url, expectedSha256: expectedSha256))
        case .unavailable(let reason):
            result = .failure(.init(reason))
        case .client(let client):
            let outcome: Outcome<ProjectFilesV1.Status> = roundTrip({ done in
                client.send({ ProjectFilesV1.StatusRequest(id: $0, path: name, expectedSha256: expectedSha256) },
                            as: ProjectFilesV1.Status.self, completion: done)
            }, late: { [weak self] r in
                self?.lateReplies.append("status \(name): \(Self.describe(r))")
                self?.note("late status reply for \(name) ignored")
            })
            switch outcome {
            case .reply(let s): result = .success(s)
            case .failed(let f): result = .failure(.init(f.text))
            case .timedOut(let t): result = .failure(.init("no status reply from the project-files helper within \(Int(t)) s"))
            }
        }
        if case .success(let s) = result { lastDiskState = s.state }
        return result
    }

    // MARK: direct (no helper) path

    private func readDirect(_ url: URL) -> ReadResult {
        do {
            return .text(try String(contentsOf: url, encoding: .utf8))
        } catch let e as NSError where e.domain == NSCocoaErrorDomain && e.code == NSFileReadNoSuchFileError {
            return .missing
        } catch {
            return .failed(error.localizedDescription)
        }
    }

    private func statusDirect(_ url: URL, expectedSha256: String?) -> ProjectFilesV1.Status {
        let name = url.lastPathComponent
        guard let data = try? Data(contentsOf: url) else {
            return .init(path: name, exists: false, state: expectedSha256 == nil ? .unchanged : .deleted)
        }
        let sha = SourceDigest.sha256Hex(data)
        let state: ProjectFilesV1.DiskState = expectedSha256 == nil ? .created : (expectedSha256 == sha ? .unchanged : .modified)
        let mtime = (try? FileManager.default.attributesOfItem(atPath: url.path)[.modificationDate] as? Date)
            .map { Int($0.timeIntervalSince1970 * 1000) }
        return .init(path: name, exists: true, state: state, sha256: sha, bytes: data.count, mtimeUnixMs: mtime)
    }

    private func saveDirect(_ url: URL, text: String, expected: ProjectFilesV1.Expected, force: Bool) -> SaveResult {
        if !force {
            let current = statusDirect(url, expectedSha256: nil)
            var kind: ProjectFilesV1.ConflictKind?
            var ours: String?
            switch expected {
            case .any: break
            case .newFile: if current.exists { kind = .alreadyExists }
            case .hash(let h):
                ours = h
                if !current.exists { kind = .deletedExternally } else if current.sha256 != h { kind = .modifiedExternally }
            }
            if let kind {
                let conflict = DocumentConflict(url: url, kind: kind, ours: ours, theirs: current.sha256,
                                                size: current.bytes, mtimeUnixMs: current.mtimeUnixMs, viaHelper: false)
                self.conflict = conflict
                lastDiskState = kind == .deletedExternally ? .deleted : .modified
                note(conflict.summary)
                return .conflict(conflict)
            }
        }
        do {
            try text.write(to: url, atomically: true, encoding: .utf8)
            conflict = nil
            lastDiskState = .unchanged
            note("saved \(url.lastPathComponent) directly (no helper: best-effort conflict check, not locked)")
            return .saved(sha256: SourceDigest.sha256Hex(text))
        } catch {
            note("direct save of \(url.lastPathComponent) failed: \(error.localizedDescription)")
            return .failed(error.localizedDescription)
        }
    }

    private static func conflict(url: URL, _ c: ProjectFilesV1.Conflict, viaHelper: Bool) -> DocumentConflict {
        .init(url: url, kind: c.kind, ours: c.ours, theirs: c.theirs, size: c.size, mtimeUnixMs: c.mtimeUnixMs, viaHelper: viaHelper)
    }

    private static func describe<T>(_ r: Result<T, LineProcessFailure>) -> String {
        switch r {
        case .success(let v): "\(v)"
        case .failure(let f): f.text
        }
    }
}

extension SourceDigest {
    static func sha256Hex(_ data: Data) -> String {
        SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined()
    }
}

/// Open/save of the active `.tex` document. The project is single-entry for
/// now: the opened file becomes `main.tex` in the compile request (the compiler
/// only compiles the entry document), and its URL is remembered for saving.
/// Reads and saves go through `files` (rooted helper or direct fallback) with an
/// explicit conflict state: a save never silently overwrites a file that changed
/// on disk since it was opened, and the buffer is never lost to a lost reply.
extension ShellModel {
    static let texType = UTType(filenameExtension: "tex") ?? .plainText

    private static var filesKey = 0
    /// File-layer state (see `DocumentFilesState`).
    var files: DocumentFilesState {
        if let existing = objc_getAssociatedObject(self, &Self.filesKey) as? DocumentFilesState { return existing }
        let state = DocumentFilesState()
        objc_setAssociatedObject(self, &Self.filesKey, state, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
        return state
    }

    func openTexPanel() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [Self.texType, .plainText]
        panel.message = "Open a LaTeX source file as the entry document"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        guard isDirty else { openTex(at: url); return }
        let alert = NSAlert()
        alert.messageText = "Save changes to \(documentURL?.lastPathComponent ?? "the unsaved buffer") before opening \(url.lastPathComponent)?"
        alert.informativeText = "Discarded text stays recoverable this session via Edit > Restore Discarded Buffer."
        alert.addButton(withTitle: "Save")
        alert.addButton(withTitle: "Discard")
        alert.addButton(withTitle: "Cancel")
        switch alert.runModal() {
        case .alertFirstButtonReturn:
            if documentURL == nil, !saveTexAs() { return }
            if openTex(at: url, dirty: .saveFirst) == .saveFailed, files.conflict != nil { resolveConflictPanel() }
        case .alertSecondButtonReturn: openTex(at: url, dirty: .discard)
        default: break
        }
    }

    /// What a caller authorized for the unsaved buffer before opening another file.
    enum DirtyDisposition: Equatable { case none, saveFirst, discard }

    enum OpenOutcome: Equatable { case opened, blockedByUnsavedEdits, saveFailed, readFailed }

    /// The last buffer replaced by an authorized open, kept so an accidental
    /// discard remains recoverable within the session.
    struct RecoverableBuffer: Equatable { var url: URL?; var text: String }

    /// Opens a `.tex` file as the entry document. When the current buffer is
    /// dirty, nothing is replaced unless the caller passes an explicit
    /// disposition: `.saveFirst` writes the current file (or refuses if it has
    /// no URL), `.discard` replaces it but keeps the text in `recoverableBuffer`.
    @discardableResult
    func openTex(at url: URL, dirty: DirtyDisposition = .none) -> OpenOutcome {
        var discarding: RecoverableBuffer?
        if isDirty {
            switch dirty {
            case .none:
                captureNote = "\(documentURL?.lastPathComponent ?? "The unsaved buffer") has unsaved edits; save or discard before opening \(url.lastPathComponent)."
                return .blockedByUnsavedEdits
            case .saveFirst:
                guard documentURL != nil, saveTex() else {
                    captureNote = "Could not save the current buffer (\(files.status)); \(url.lastPathComponent) was not opened."
                    return .saveFailed
                }
            case .discard:
                discarding = RecoverableBuffer(url: documentURL, text: activeText)
            }
        }
        switch files.read(url) {
        case .text(let text):
            // Only a successful read consumes the discard decision: a failed
            // open leaves the dirty buffer in place, not "discarded".
            if let discarding { recoverableBuffer = discarding }
            replaceProject(entryText: text)
            documentURL = url
            savedText = text
            files.conflict = nil
            captureNote = "Opened \(url.lastPathComponent) (\(text.utf8.count) bytes)"
                + (recoverableBuffer == nil ? "" : "; previous unsaved buffer kept (Edit > Restore Discarded Buffer)")
            if workerAttached { compile() }
            return .opened
        case .missing:
            captureNote = "Could not open \(url.lastPathComponent): no such file"
            return .readFailed
        case .failed(let reason):
            captureNote = "Could not open \(url.lastPathComponent): \(reason)"
            return .readFailed
        }
    }

    /// Restores the buffer discarded by the last authorized open (undo of the
    /// discard decision); the currently open file is left untouched on disk.
    @discardableResult
    func restoreDiscardedBuffer() -> Bool {
        guard let kept = recoverableBuffer else { return false }
        if isDirty {
            captureNote = "Current buffer has unsaved edits; save it before restoring the discarded buffer."
            return false
        }
        replaceProject(entryText: kept.text)
        documentURL = kept.url
        // "Saved" is whatever is on disk now, so the restored text stays dirty
        // (it differs from disk) and cannot be lost again silently. No file on
        // disk means the next save expects a new file.
        savedText = kept.url.flatMap { url -> String? in
            if case .text(let t) = files.read(url) { return t }
            return nil
        }
        files.conflict = nil
        recoverableBuffer = nil
        captureNote = "Restored the discarded buffer (\(kept.text.utf8.count) bytes, unsaved)."
        if workerAttached { compile() }
        return true
    }

    /// Dirty means the buffer differs from what was last opened/saved. A fresh
    /// fixture-seeded buffer (no file, never saved) counts as dirty only once edited.
    var isDirty: Bool {
        // A non-entry document compares with the text it was opened with
        // (ProjectDocuments); the entry with its saved text.
        if activePath != project.entryPath { return project.isDirty(activePath) }
        if let savedText { return !savedText.sameBytes(as: activeText) }
        return documentURL == nil && editorRevision > 1 && !activeText.isEmpty
    }

    /// Hash of the text the editor last saw on disk for the open document.
    var baselineSha256: String? { savedText.map { SourceDigest.sha256Hex($0) } }

    /// What a save to `url` must find on disk: the baseline for the open
    /// document (or no file, if it was never on disk), anything for Save As
    /// (the panel already confirmed a replacement).
    private func expectedOnDisk(for url: URL) -> ProjectFilesV1.Expected {
        guard url == documentURL else { return .any }
        return baselineSha256.map { .hash($0) } ?? .newFile
    }

    /// Saves the open document. On a conflict returns false, sets
    /// `files.conflict`, keeps the buffer, and writes nothing.
    @discardableResult
    func saveTex() -> Bool {
        guard let url = documentURL else { return saveTexAs() }
        return write(to: url, expected: expectedOnDisk(for: url), force: false)
    }

    /// Menu-driven save: on a conflict, asks the user how to resolve it.
    func saveTexInteractive() {
        // A non-entry document saves to its own rooted file (never to the
        // entry URL): ProjectDocuments.saveDocument, helper export or rooted
        // compare-and-replace, conflicts reported the same way.
        if activePath != project.entryPath {
            let path = activePath
            Task { @MainActor [weak self] in
                guard let self else { return }
                switch await project.saveDocument(path) {
                case .saved(let p, _): captureNote = "Saved \(p)"
                case .conflict(let c): captureNote = c.summary
                case .failed(let why): captureNote = "Save of \(path) failed: \(why)"
                }
            }
            return
        }
        // With the durable helper attached the export goes through its rooted,
        // locked save so the ledger text and the .tex never diverge; the
        // result is reported asynchronously (never blocks the UI).
        if controllerAttached, documentURL != nil {
            Task { @MainActor [weak self] in
                guard let self else { return }
                switch await controllerSave() {
                case .saved: break
                case .conflict: resolveConflictPanel()
                case .failed(let why): captureNote = "Save through the preview controller failed: \(why)"
                }
            }
            return
        }
        if !saveTex(), files.conflict != nil { resolveConflictPanel() }
    }

    @discardableResult
    func saveTexAs() -> Bool {
        let panel = NSSavePanel()
        panel.allowedContentTypes = [Self.texType]
        panel.nameFieldStringValue = documentURL?.lastPathComponent ?? "main.tex"
        guard panel.runModal() == .OK, let url = panel.url else { return false }
        return write(to: url, expected: expectedOnDisk(for: url), force: false)
    }

    /// Resolves a conflict by writing the buffer over whatever is on disk.
    /// Only after an explicit user decision; never called automatically.
    @discardableResult
    func overwriteOnDisk() -> Bool {
        guard let url = files.conflict?.url ?? documentURL else { captureNote = "No file to overwrite."; return false }
        return write(to: url, expected: .any, force: true)
    }

    /// Resolves a conflict by replacing the buffer with the file on disk. A
    /// dirty buffer is only replaced with `.discard` (kept recoverable) or
    /// `.saveFirst` (which cannot succeed while the conflict stands).
    @discardableResult
    func reloadFromDisk(dirty: DirtyDisposition = .none) -> OpenOutcome {
        guard let url = files.conflict?.url ?? documentURL else { captureNote = "No file to reload."; return .readFailed }
        let outcome = openTex(at: url, dirty: dirty)
        if outcome == .opened { captureNote = "Reloaded \(url.lastPathComponent) from disk" + (recoverableBuffer == nil ? "." : "; previous buffer kept (Edit > Restore Discarded Buffer).") }
        return outcome
    }

    /// Asks the file layer whether the open document still matches its baseline
    /// and records an explicit conflict if not (before any save is attempted).
    @discardableResult
    func checkDiskStatus() -> ProjectFilesV1.DiskState? {
        guard let url = documentURL else { return nil }
        switch files.diskStatus(url, expectedSha256: baselineSha256) {
        case .failure(let failure):
            captureNote = "Could not check \(url.lastPathComponent) on disk: \(failure.reason)"
            return nil
        case .success(let s):
            switch s.state {
            case .unchanged:
                if files.conflict?.url == url { files.conflict = nil }
            case .created:
                // The editor expected no file; one appeared. Saving would be
                // refused (`alreadyExists`), so say so now.
                files.conflict = DocumentConflict(url: url, kind: .alreadyExists, ours: nil, theirs: s.sha256, size: s.bytes,
                                                  mtimeUnixMs: s.mtimeUnixMs, viaHelper: files.usesHelper)
                captureNote = files.conflict?.summary
            case .deleted:
                // Nothing to overwrite: not a blocking conflict; Save recreates it.
                if files.conflict?.url == url { files.conflict = nil }
                captureNote = "\(url.lastPathComponent) was deleted on disk; Save will recreate it from the buffer."
            case .modified:
                files.conflict = DocumentConflict(url: url, kind: .modifiedExternally, ours: baselineSha256, theirs: s.sha256,
                                                  size: s.bytes, mtimeUnixMs: s.mtimeUnixMs, viaHelper: files.usesHelper)
                captureNote = files.conflict?.summary
            }
            return s.state
        }
    }

    /// Modal resolution of `files.conflict`: Overwrite / Reload / Keep Editing.
    func resolveConflictPanel() {
        guard let conflict = files.conflict else { return }
        let alert = NSAlert()
        alert.messageText = "\(conflict.url.lastPathComponent) changed on disk"
        alert.informativeText = conflict.summary
        alert.addButton(withTitle: "Overwrite")
        alert.addButton(withTitle: "Reload")
        alert.addButton(withTitle: "Keep Editing")
        switch alert.runModal() {
        case .alertFirstButtonReturn: overwriteOnDisk()
        case .alertSecondButtonReturn: reloadFromDisk(dirty: isDirty ? .discard : .none)
        default: break
        }
    }

    private func write(to url: URL, expected: ProjectFilesV1.Expected, force: Bool) -> Bool {
        let text = activeText
        let lateReceipt: @MainActor (String) -> Void = { [weak self] sha in
            // The helper confirmed, after our wait expired, that exactly `text`
            // is on disk at `url`: that text is the new baseline. Edits made
            // meanwhile keep the buffer dirty; a different open file is untouched.
            guard let self, self.documentURL == url, SourceDigest.sha256Hex(text) == sha else { return }
            self.savedText = text
            self.captureNote = "Late confirmation: \(url.lastPathComponent) was saved" + (self.isDirty ? " (buffer edited since; still unsaved)." : ".")
            self.bridgeSourceSaved(url: url, text: text)
        }
        var result = files.save(url, text: text, expected: expected, force: force, lateReceipt: lateReceipt)
        var recreated = false
        if case .conflict(let c) = result, c.kind == .deletedExternally, !force {
            // Nothing on disk can be overwritten: recreate the file, but only if
            // it is still absent (a file appearing meanwhile is `alreadyExists`).
            files.conflict = nil
            recreated = true
            result = files.save(url, text: text, expected: .newFile, force: false, lateReceipt: lateReceipt)
        }
        switch result {
        case .saved:
            documentURL = url
            savedText = text
            captureNote = "Saved \(url.lastPathComponent)" + (recreated ? " (recreated; it had been deleted on disk)" : "")
            bridgeSourceSaved(url: url, text: text)
            return true
        case .conflict(let conflict):
            captureNote = conflict.summary
            return false
        case .failed(let reason):
            captureNote = "Save failed: \(reason)"
            return false
        }
    }
}
