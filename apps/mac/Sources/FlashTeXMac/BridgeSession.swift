import CryptoKit
import Foundation
import FlashTeXProtocol

/// SHA-256 hex digest of a string's UTF-8 bytes (contract: `document_before_sha256`).
enum SourceDigest {
    static func sha256Hex(_ text: String) -> String {
        SHA256.hash(data: Data(text.utf8)).map { String(format: "%02x", $0) }.joined()
    }
}

/// Application ledger for bridge-prepared edits, persisted as JSON under the
/// bridge store (`<store>/mac/edit-ledger.json`). An edit ID is applied at most
/// once; `prepared` entries survive a crash between preparation and application,
/// `applied` entries survive a crash before the bridge confirmed the receipt.
/// The pre-edit text is kept only until the receipt is confirmed, because the
/// contract's crash recovery reopens the pre-edit snapshot before replaying it.
struct EditLedgerEntry: Codable, Equatable {
    enum State: String, Codable { case prepared, applied, confirmed, abandoned }
    var editId: String
    var captureId: String
    var projectId: String
    var path: String
    var expectedRevision: Int
    var startByte: Int
    var endByte: Int
    var removedText: String
    var replacement: String
    var documentBeforeSha256: String
    var documentBeforeText: String?
    var newRevision: Int?
    var documentAfterSha256: String?
    var state: State
    var note: String?
    var updatedAt: Date

    init(edit: TransferV1.CaptureEdit, beforeText: String) {
        editId = edit.editId; captureId = edit.captureId; projectId = edit.projectId; path = edit.path
        expectedRevision = edit.expectedRevision; startByte = edit.startByte; endByte = edit.endByte
        removedText = edit.removedText; replacement = edit.replacement
        documentBeforeSha256 = edit.documentBeforeSha256; documentBeforeText = beforeText
        state = .prepared; updatedAt = Date()
    }
}

final class EditLedger {
    let url: URL
    private(set) var entries: [EditLedgerEntry] = []
    private(set) var loadError: String?

    init(storeDirectory: URL) {
        url = storeDirectory.appendingPathComponent("mac/edit-ledger.json")
        if let data = try? Data(contentsOf: url) {
            do { entries = try Self.decoder.decode([EditLedgerEntry].self, from: data) }
            catch { loadError = "ledger unreadable: \(error)" }
        }
    }

    private static let decoder: JSONDecoder = { let d = JSONDecoder(); d.dateDecodingStrategy = .iso8601; return d }()
    private static let encoder: JSONEncoder = {
        let e = JSONEncoder(); e.dateEncodingStrategy = .iso8601; e.outputFormatting = [.prettyPrinted, .sortedKeys]; return e
    }()

    func entry(editId: String) -> EditLedgerEntry? { entries.first { $0.editId == editId } }
    func entry(captureId: String) -> EditLedgerEntry? { entries.first { $0.captureId == captureId } }
    var unconfirmed: [EditLedgerEntry] { entries.filter { $0.state == .prepared || $0.state == .applied } }

    /// Inserts or replaces by edit ID and writes the file atomically (temp + rename).
    func upsert(_ entry: EditLedgerEntry) throws {
        var e = entry
        e.updatedAt = Date()
        if let i = entries.firstIndex(where: { $0.editId == e.editId }) { entries[i] = e } else { entries.append(e) }
        try save()
    }

    func update(editId: String, _ change: (inout EditLedgerEntry) -> Void) throws {
        guard var e = entry(editId: editId) else { return }
        change(&e)
        try upsert(e)
    }

    private func save() throws {
        let dir = url.deletingLastPathComponent()
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true,
                                                attributes: [.posixPermissions: 0o700])
        let data = try Self.encoder.encode(entries)
        try data.write(to: url, options: .atomic)
    }
}

/// One attached bridge process plus everything the Mac must remember about it:
/// the bridge's view of the document (what was sent), the pinned destination,
/// the captures in flight, and the edit ledger. Owned by `ShellModel`; all
/// state changes happen on the main actor and `onChange` lets the model publish.
@MainActor
final class BridgeSession {
    enum CaptureState: String { case received, converting, proposed, prepared, applied, confirmed, rejected, needsReselection, failed }
    struct Capture: Equatable {
        var captureId: String
        var destinationId: String
        var state: CaptureState
        var note: String
    }

    let client: BridgeClient
    let projectId: String
    let ledger: EditLedger
    let storeDirectory: URL
    private(set) var status: String
    private(set) var running = true
    private(set) var destination: TransferV1.Anchor?
    private(set) var captures: [Capture] = []
    private(set) var log: [String] = []
    /// Snapshot the bridge is believed to hold per path (mirrors sent edits).
    private(set) var shadow: [String: (revision: Int, text: String)] = [:]
    /// Edit the editor is about to apply; the next text change equal to
    /// `afterText` is its application and must not be sent as `document_edit`.
    private(set) var expectedApplication: (edit: TransferV1.CaptureEdit, afterText: String)?
    var onChange: () -> Void = {}

    init(executable: URL, arguments: [String] = [], storeDirectory: URL, enableGrok: Bool = false,
         projectId: String) throws {
        self.projectId = projectId
        self.storeDirectory = storeDirectory
        self.ledger = EditLedger(storeDirectory: storeDirectory)
        status = "launching \(executable.lastPathComponent)"
        var events: ((BridgeClient.Event) -> Void)?
        client = try BridgeClient(executable: executable, arguments: arguments, storeDirectory: storeDirectory,
                                  enableGrok: enableGrok) { events?($0) }
        events = { [weak self] event in
            guard let self else { return }
            Task { @MainActor in self.handle(event) }
        }
        status = "attached: \(executable.lastPathComponent)"
        if let err = ledger.loadError { note(err) }
    }

    func terminate() {
        client.terminate()
        running = false
        status = "bridge detached"
        onChange()
    }

    private func handle(_ event: BridgeClient.Event) {
        switch event {
        case .stderr(let s): note("bridge: " + s.trimmingCharacters(in: .whitespacesAndNewlines))
        case .protocolViolation(let m): status = "protocol violation: \(m)"; note(status)
        case .unsolicited(let id, let type, let code): note("unsolicited \(type) id=\(id ?? "null") code=\(code ?? "-")")
        case .exited(let code):
            running = false
            status = "bridge exited (\(code))"
            expectedApplication = nil
            note(status)
        }
        onChange()
    }

    private func note(_ line: String) {
        log.append(line)
        if log.count > 200 { log.removeFirst(log.count - 200) }
    }

    private func fail(_ what: String, _ error: Error) -> BridgeClient.Failure {
        let f = (error as? BridgeClient.Failure) ?? .undecodable("\(error)")
        status = "\(what) failed — \(f.text)"
        note(status)
        onChange()
        return f
    }

    func capture(_ id: String) -> Capture? { captures.first { $0.captureId == id } }

    private func setCapture(_ id: String, destination: String? = nil, _ state: CaptureState, _ text: String) {
        if let i = captures.firstIndex(where: { $0.captureId == id }) {
            captures[i].state = state; captures[i].note = text
        } else {
            captures.append(.init(captureId: id, destinationId: destination ?? "", state: state, note: text))
        }
        onChange()
    }

    // MARK: documents

    func open(path: String, revision: Int, text: String) async throws {
        do {
            _ = try await client.request(.documentOpen, TransferV1.DocumentOpen(projectId: projectId, path: path, revision: revision, text: text),
                                         as: TransferV1.Empty.self)
            shadow[path] = (revision, text)
            status = "attached: \(client.executable.lastPathComponent) · \(path) open at revision \(revision)"
            onChange()
        } catch { throw fail("document_open", error) }
    }

    /// Sends the byte-range difference between `oldText` and `newText` as
    /// `document_edit`. Fire-and-forget: a rejected edit triggers a full
    /// `document_open` resynchronization from the newest shadow snapshot.
    func edited(path: String, oldText: String, newText: String, base: Int, revision: Int) {
        guard running else { return }
        let region = SourceMapping.changedRegion(from: oldText, to: newText)
        let edit = TransferV1.DocumentEdit(projectId: projectId, path: path, baseRevision: base, revision: revision,
                                           startByte: region.startByte, endByte: region.oldEndByte, replacement: region.replacement)
        shadow[path] = (revision, newText)
        client.send(.documentEdit, edit, as: TransferV1.DocumentUpdated.self) { [weak self] result in
            guard let self else { return }
            switch result {
            case .success(let updated):
                if updated.revision != revision { self.note("document_updated reports revision \(updated.revision), sent \(revision)") }
            case .failure(let f):
                self.note("document_edit \(base)->\(revision) refused (\(f.text)); resynchronizing snapshot")
                self.resync(path: path)
            }
        }
    }

    private func resync(path: String) {
        guard running, let snap = shadow[path] else { return }
        client.send(.documentOpen, TransferV1.DocumentOpen(projectId: projectId, path: path, revision: snap.revision, text: snap.text),
                    as: TransferV1.Empty.self) { [weak self] result in
            guard let self else { return }
            if case .failure(let f) = result { self.status = "snapshot resync failed — \(f.text)"; self.note(self.status); self.onChange() }
            else { self.note("snapshot resynchronized at revision \(snap.revision)") }
        }
    }

    // MARK: destinations and captures

    func pin(destinationId: String, path: String, revision: Int, startByte: Int, endByte: Int) async throws -> TransferV1.Anchor {
        do {
            let anchor = try await client.request(.destinationPin,
                TransferV1.DestinationPin(destinationId: destinationId, projectId: projectId, path: path, revision: revision,
                                          startByte: startByte, endByte: endByte), as: TransferV1.Anchor.self)
            destination = anchor
            status = "pinned \(destinationId) at \(path) bytes \(startByte)..<\(endByte) (revision \(revision))"
            onChange()
            return anchor
        } catch { throw fail("destination_pin", error) }
    }

    func submit(_ capture: RuntimeV1.CaptureSubmit) async throws -> TransferV1.CaptureReceived {
        do {
            let received = try await client.request(.captureSubmit, capture, as: TransferV1.CaptureReceived.self)
            setCapture(capture.captureId, destination: capture.destinationId, .received,
                       "received (durable: \(received.durable), proposal: \(received.hasProposal), applied: \(received.applied))")
            status = "capture \(capture.captureId) received durably"
            onChange()
            return received
        } catch {
            let f = fail("capture_submit", error)
            // A conflicting retry does not change the state of the capture already journaled.
            setCapture(capture.captureId, destination: capture.destinationId, self.capture(capture.captureId)?.state ?? .failed, f.text)
            throw f
        }
    }

    /// Forgets the pinned destination (after an insertion at it, or when the
    /// bridge reported that reselection is required). A new pin gets a new ID.
    func invalidateDestination() {
        destination = nil
        onChange()
    }

    func convert(captureId: String, supportedFeatures: [String] = []) async throws -> RuntimeV1.CaptureProposal {
        let previous = capture(captureId)?.state ?? .received
        setCapture(captureId, .converting, "converting…")
        do {
            let proposal = try await client.request(.captureConvert,
                TransferV1.CaptureConvert(captureId: captureId, supportedFeatures: supportedFeatures),
                as: RuntimeV1.CaptureProposal.self)
            setCapture(captureId, .proposed, "proposal ready (context revision \(proposal.contextRevision.map(String.init) ?? "?"))")
            status = "proposal for \(captureId) awaiting review"
            onChange()
            return proposal
        } catch {
            let f = fail("capture_convert", error)
            // Provider errors are plain text; nothing here prompts for a key. The
            // capture stays convertible unless the bridge says it is rejected.
            setCapture(captureId, f.code == "capture_rejected" ? .rejected : previous, f.text)
            throw f
        }
    }

    func prepare(captureId: String, expectedRevision: Int) async throws -> TransferV1.CaptureEdit {
        do {
            let edit = try await client.request(.capturePrepareInsert,
                TransferV1.CapturePrepareInsert(captureId: captureId, expectedRevision: expectedRevision, approved: true),
                as: TransferV1.CaptureEdit.self)
            setCapture(captureId, .prepared, "edit \(edit.editId) prepared at revision \(edit.expectedRevision)")
            return edit
        } catch {
            let f = fail("capture_prepare_insert", error)
            setCapture(captureId, f.code == "destination_reselection_required" ? .needsReselection : .proposed, f.text)
            throw f
        }
    }

    func reject(captureId: String) async throws {
        do {
            _ = try await client.request(.captureReject, TransferV1.CaptureID(captureId: captureId), as: TransferV1.CaptureID.self)
            setCapture(captureId, .rejected, "rejected (durable, terminal)")
            status = "capture \(captureId) rejected"
            onChange()
        } catch {
            let f = fail("capture_reject", error)
            setCapture(captureId, capture(captureId)?.state ?? .failed, f.text)
            throw f
        }
    }

    func status(captureId: String) async throws -> TransferV1.CaptureStatus {
        do { return try await client.request(.captureStatus, TransferV1.CaptureID(captureId: captureId), as: TransferV1.CaptureStatus.self) }
        catch { throw fail("capture_status", error) }
    }

    // MARK: verified application and ledger

    enum Verification: Equatable { case ok(afterText: String); case refused(String) }

    /// Contract step 2: revision, SHA-256 of the UTF-8 source, scalar boundaries
    /// and removed text must all match the current buffer before anything is applied.
    static func verify(_ edit: TransferV1.CaptureEdit, projectId: String, path: String, revision: Int, text: String) -> Verification {
        guard edit.projectId == projectId, edit.path == path else {
            return .refused("edit targets \(edit.projectId)/\(edit.path), not the open document")
        }
        guard edit.expectedRevision == revision else {
            return .refused("edit expects revision \(edit.expectedRevision) but the editor is at \(revision)")
        }
        guard SourceDigest.sha256Hex(text) == edit.documentBeforeSha256 else {
            return .refused("document SHA-256 does not match the prepared edit (buffer changed)")
        }
        guard let range = text.rangeOfUTF8(start: edit.startByte, end: edit.endByte) else {
            return .refused("bytes \(edit.startByte)..<\(edit.endByte) are not a scalar-aligned range of the buffer")
        }
        guard String(text[range]) == edit.removedText else {
            return .refused("removed_text does not match the bytes at \(edit.startByte)..<\(edit.endByte)")
        }
        return .ok(afterText: text.replacingCharacters(in: range, with: edit.replacement))
    }

    /// Records a verified edit as `prepared` and arms `expectedApplication`.
    /// Returns false when the edit ID was already applied (never twice).
    func recordPrepared(_ edit: TransferV1.CaptureEdit, beforeText: String, afterText: String) throws -> Bool {
        if let existing = ledger.entry(editId: edit.editId), existing.state == .applied || existing.state == .confirmed {
            setCapture(edit.captureId, .applied, "edit \(edit.editId) already applied; ignoring")
            return false
        }
        try ledger.upsert(EditLedgerEntry(edit: edit, beforeText: beforeText))
        expectedApplication = (edit, afterText)
        onChange()
        return true
    }

    func abandonExpectedApplication(_ why: String) {
        guard let expected = expectedApplication else { return }
        expectedApplication = nil
        try? ledger.update(editId: expected.edit.editId) { $0.state = .abandoned; $0.note = why; $0.documentBeforeText = nil }
        setCapture(expected.edit.captureId, .needsReselection, "not applied: \(why)")
    }

    /// Contract steps 3–4: the editor applied the edit; persist `applied`, then
    /// send `capture_applied` (never `document_edit` for this change).
    func applicationApplied(newRevision: Int, afterText: String) {
        guard let expected = expectedApplication else { return }
        expectedApplication = nil
        let edit = expected.edit
        do {
            try ledger.update(editId: edit.editId) {
                $0.state = .applied; $0.newRevision = newRevision; $0.documentAfterSha256 = SourceDigest.sha256Hex(afterText)
            }
        } catch {
            status = "ledger write failed — \(error.localizedDescription)"; note(status)
        }
        shadow[edit.path] = (newRevision, afterText)
        destination = nil // the insertion intersected (or sat exactly at) the pinned target; the bridge invalidated it
        setCapture(edit.captureId, .applied, "applied as revision \(newRevision); confirming…")
        sendApplied(captureId: edit.captureId, editId: edit.editId, newRevision: newRevision)
    }

    private func sendApplied(captureId: String, editId: String, newRevision: Int) {
        client.send(.captureApplied, TransferV1.CaptureApplied(captureId: captureId, editId: editId, newRevision: newRevision),
                    as: TransferV1.CaptureApplied.self) { [weak self] result in
            guard let self else { return }
            switch result {
            case .success(let receipt):
                try? self.ledger.update(editId: editId) { $0.state = .confirmed; $0.documentBeforeText = nil }
                self.setCapture(captureId, .confirmed, "insertion confirmed (edit \(receipt.editId), revision \(receipt.newRevision))")
                self.status = "capture \(captureId) inserted and confirmed"
            case .failure(let f):
                // Ledger keeps `applied`; reconciliation retries the receipt on the next attach.
                self.setCapture(captureId, .applied, "applied locally; receipt not confirmed (\(f.text))")
                self.status = "capture_applied failed — \(f.text)"
            }
            self.onChange()
        }
    }

    // MARK: restart reconciliation (contract step 5)

    enum ReconcileAction: Equatable {
        case confirmed(editId: String)
        case replayedReceipt(editId: String, newRevision: Int)
        case reoffered(captureId: String, proposal: RuntimeV1.CaptureProposal)
        case reselectionRequired(editId: String, why: String)
        case abandoned(editId: String, why: String)
    }

    /// Consults `capture_status` for every unconfirmed ledger entry before the
    /// current document is opened. Missing receipts are replayed against the
    /// pre-edit snapshot; prepared-but-unapplied edits are re-offered only when
    /// the buffer still matches; anything else requires a new capture/destination.
    /// Returns the actions plus the minimum revision the editor must advance to.
    func reconcile(path: String, currentText: String, currentRevision: Int) async -> (actions: [ReconcileAction], minimumRevision: Int) {
        var actions: [ReconcileAction] = []
        var minimum = currentRevision
        let currentSha = SourceDigest.sha256Hex(currentText)
        for entry in ledger.unconfirmed where entry.projectId == projectId && entry.path == path {
            let st: TransferV1.CaptureStatus
            do { st = try await status(captureId: entry.captureId) } catch {
                let why = "bridge has no usable record (\((error as? BridgeClient.Failure)?.text ?? "\(error)"))"
                try? ledger.update(editId: entry.editId) { $0.state = .abandoned; $0.note = why; $0.documentBeforeText = nil }
                actions.append(.abandoned(editId: entry.editId, why: why))
                continue
            }
            if st.applied?.editId == entry.editId {
                try? ledger.update(editId: entry.editId) { $0.state = .confirmed; $0.documentBeforeText = nil }
                minimum = max(minimum, (st.applied?.newRevision ?? 0) + 1)
                actions.append(.confirmed(editId: entry.editId))
                continue
            }
            switch entry.state {
            case .applied:
                guard st.prepared?.editId == entry.editId, let before = entry.documentBeforeText, let newRevision = entry.newRevision else {
                    let why = "applied locally but the bridge holds no prepared edit \(entry.editId); receipt cannot be replayed"
                    try? ledger.update(editId: entry.editId) { $0.state = .abandoned; $0.note = why; $0.documentBeforeText = nil }
                    actions.append(.abandoned(editId: entry.editId, why: why))
                    continue
                }
                // Reopen the pre-edit snapshot, replay the receipt, then the caller resynchronizes the live source.
                do {
                    try await open(path: path, revision: entry.expectedRevision, text: before)
                    _ = try await client.request(.captureApplied,
                        TransferV1.CaptureApplied(captureId: entry.captureId, editId: entry.editId, newRevision: newRevision),
                        as: TransferV1.CaptureApplied.self)
                    try? ledger.update(editId: entry.editId) { $0.state = .confirmed; $0.documentBeforeText = nil }
                    shadow[path] = (newRevision, entry.documentAfterSha256 == SourceDigest.sha256Hex(currentText) ? currentText : before)
                    minimum = max(minimum, newRevision + 1)
                    setCapture(entry.captureId, .confirmed, "receipt replayed after restart")
                    actions.append(.replayedReceipt(editId: entry.editId, newRevision: newRevision))
                } catch {
                    let f = fail("receipt replay", error)
                    actions.append(.abandoned(editId: entry.editId, why: "receipt replay refused (\(f.text)); ledger keeps the applied entry"))
                }
            case .prepared:
                if st.prepared?.editId == entry.editId, currentSha == entry.documentBeforeSha256, currentRevision <= entry.expectedRevision,
                   let p = st.proposal {
                    minimum = max(minimum, entry.expectedRevision)
                    let proposal = RuntimeV1.CaptureProposal(captureId: entry.captureId, latex: p.latex, ambiguities: p.ambiguities,
                                                             requiredDependencies: p.requiredDependencies)
                    setCapture(entry.captureId, .proposed, "prepared edit re-offered after restart")
                    actions.append(.reoffered(captureId: entry.captureId, proposal: proposal))
                } else {
                    let why = currentSha == entry.documentBeforeSha256
                        ? "editor revision \(currentRevision) passed the prepared revision \(entry.expectedRevision); use a new capture"
                        : "buffer changed since edit \(entry.editId) was prepared; pin a new destination and submit a new capture"
                    try? ledger.update(editId: entry.editId) { $0.state = .abandoned; $0.note = why; $0.documentBeforeText = nil }
                    setCapture(entry.captureId, .needsReselection, why)
                    actions.append(.reselectionRequired(editId: entry.editId, why: why))
                }
            case .confirmed, .abandoned:
                break
            }
        }
        onChange()
        return (actions, minimum)
    }
}
