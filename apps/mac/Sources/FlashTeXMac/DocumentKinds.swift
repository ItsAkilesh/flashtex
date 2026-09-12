import AppKit
import ObjectiveC
import Observation
import SwiftUI

/// Explicit document kinds (lane `mac-bibliography-kinds`, gap 3).
///
/// The durable helper indexes every attached source as `latex` or
/// `bibliography` (`crates/preview-controller/docs/source-plans.md`). A kind is
/// only ever what the application *declared*: `open_document` with
/// `document_kind: "bibliography"`, or `bibliography_paths` in the startup
/// config. The helper never infers a kind from a file extension or from the
/// text, and neither does this file — `refs.bib` opened through the generic
/// `ProjectDocuments.openDocument` is a LaTeX source until it is declared.
///
/// The helper does not persist declarations across a process restart: "the
/// application owns persistence of its startup configuration" and the
/// declared paths must be supplied again on reopen. This file therefore
///
/// - declares a bibliography source through the helper (`DocumentKinds
///   .declareBibliography`), with the exact `snapshot` versions/generation
///   the helper requires, then lets `ProjectDocuments` adopt the durable
///   document as an ordinary project member (no second `open_document`);
/// - reads `document_kinds` from `snapshot` (`refresh`), the only source of
///   truth for what is shown;
/// - persists the declared bibliography paths per project entry after a
///   successful typed open or a detach (`DocumentKindsStore`, a small JSON
///   file next to the helper's private ledger root), so `attachController`
///   can hand them back to the helper as `bibliography_paths` — where the
///   helper restores the retained ledger even when the original `.bib` file
///   has been deleted from disk;
/// - shows a read-only indicator and a small Project menu section.
///
/// Attached to the model as an associated object (`model.documentKinds`),
/// the same way `ProjectDocuments` is, so ShellModel.swift stays untouched.

/// A kind exactly as the helper reports it in `document_kinds`.
enum DocumentKind: String, Codable, Equatable, Sendable {
    case latex, bibliography
}

// MARK: - persistence

/// Application-owned record of explicit declarations, keyed by entry path so
/// two projects sharing a directory (different entries) do not share
/// declarations. Version 1.
struct DocumentKindsRecord: Codable, Equatable {
    var version: Int = 1
    /// Entry path → sorted, de-duplicated rooted bibliography paths.
    var bibliographyByEntry: [String: [String]] = [:]

    enum CodingKeys: String, CodingKey { case version, bibliographyByEntry = "bibliography_by_entry" }

    func bibliographyPaths(entry: String) -> [String] { bibliographyByEntry[entry] ?? [] }

    mutating func set(bibliographyPaths: [String], entry: String) {
        let clean = Array(Set(bibliographyPaths.filter { $0 != entry })).sorted()
        if clean.isEmpty { bibliographyByEntry.removeValue(forKey: entry) } else { bibliographyByEntry[entry] = clean }
    }
}

enum DocumentKindsStore {
    static let fileName = "document-kinds.json"
    /// Only this record version is understood; another version is left alone
    /// and reported (never silently rewritten).
    static let supportedVersion = 1

    enum LoadError: Error, Equatable { case unreadable(String), unsupportedVersion(Int) }

    /// `<ledger root>/document-kinds.json`. The helper's own state lives in
    /// `<ledger root>/project-<binding>/…` and it never reads siblings.
    static func url(ledgerRoot: URL) -> URL { ledgerRoot.appendingPathComponent(fileName) }

    @MainActor static func url(projectRoot: URL) -> URL { url(ledgerRoot: ShellModel.controllerLedgerRoot(for: projectRoot)) }

    /// Missing file → empty record. Unreadable or a different version → error
    /// (the caller decides whether to start empty; `save` overwrites).
    static func load(ledgerRoot: URL) -> Result<DocumentKindsRecord, LoadError> {
        let file = url(ledgerRoot: ledgerRoot)
        guard FileManager.default.fileExists(atPath: file.path) else { return .success(DocumentKindsRecord()) }
        do {
            let data = try Data(contentsOf: file)
            let record = try JSONDecoder().decode(DocumentKindsRecord.self, from: data)
            guard record.version == supportedVersion else { return .failure(.unsupportedVersion(record.version)) }
            return .success(record)
        } catch {
            return .failure(.unreadable(error.localizedDescription))
        }
    }

    static func save(_ record: DocumentKindsRecord, ledgerRoot: URL) throws {
        try FileManager.default.createDirectory(at: ledgerRoot, withIntermediateDirectories: true)
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys, .prettyPrinted]
        try encoder.encode(record).write(to: url(ledgerRoot: ledgerRoot), options: .atomic)
    }

    /// The declared bibliography paths a helper launch for `entry` under
    /// `projectRoot` must supply as `bibliography_paths` (empty when nothing
    /// was declared or the record cannot be read — the launch still works,
    /// the kinds are just not restored, and `DocumentKinds.status` says so).
    @MainActor static func bibliographyPaths(projectRoot: URL, entry: String) -> [String] {
        switch load(ledgerRoot: ShellModel.controllerLedgerRoot(for: projectRoot)) {
        case .success(let record): return record.bibliographyPaths(entry: entry)
        case .failure: return []
        }
    }

    /// Replaces the declarations for `entry` (other entries are kept).
    @MainActor static func persist(bibliographyPaths: [String], projectRoot: URL, entry: String) throws {
        let ledgerRoot = ShellModel.controllerLedgerRoot(for: projectRoot)
        var record: DocumentKindsRecord
        switch load(ledgerRoot: ledgerRoot) {
        case .success(let r): record = r
        case .failure(.unsupportedVersion(let v)): throw LoadError.unsupportedVersion(v)
        case .failure(.unreadable): record = DocumentKindsRecord() // corrupt file: rewrite
        }
        record.set(bibliographyPaths: bibliographyPaths, entry: entry)
        try save(record, ledgerRoot: ledgerRoot)
    }
}

// MARK: - model layer

@MainActor
@Observable
final class DocumentKinds {
    enum DeclareOutcome: Equatable {
        case declared(path: String)
        case alreadyDeclared(path: String)
        case refused(String)
    }

    /// `document_kinds` from the last helper `snapshot`/refresh (empty until
    /// the helper reported one). Never derived from paths.
    private(set) var kinds: [String: DocumentKind] = [:]
    /// Membership generation the `kinds` map belongs to.
    private(set) var generation: Int?
    /// Human-readable state of the last kind operation.
    private(set) var status = "no document kinds reported yet"
    /// The declared paths last written to (or read from) the persisted record
    /// for this project, and the last persistence failure, if any.
    private(set) var persistedBibliographyPaths: [String] = []
    private(set) var persistError: String?
    /// Number of completed refreshes (tests).
    private(set) var refreshes = 0

    @ObservationIgnored private unowned let model: ShellModel
    @ObservationIgnored private var armed = false
    @ObservationIgnored private var lastSeenGeneration: Int?

    init(model: ShellModel) {
        self.model = model
        armTracking()
    }

    /// Paths the helper currently reports as `bibliography`, sorted.
    var bibliographyPaths: [String] { kinds.filter { $0.value == .bibliography }.keys.sorted() }

    func kind(of path: String) -> DocumentKind? { kinds[path] }

    /// Project root/entry the persisted record is keyed by (nil for an
    /// unsaved buffer: the helper runs in a session temporary project then,
    /// and nothing durable should be written for it).
    private var persistenceKey: (root: URL, entry: String)? {
        guard let url = model.documentURL else { return nil }
        return (url.deletingLastPathComponent(), model.project.entryPath)
    }

    /// What `attachController` should pass as `bibliography_paths` for the
    /// current saved project (empty for an unsaved buffer).
    var startupBibliographyPaths: [String] {
        guard let key = persistenceKey else { return [] }
        return DocumentKindsStore.bibliographyPaths(projectRoot: key.root, entry: key.entry)
    }

    // MARK: helper

    /// `snapshot` → `document_kinds`. Does not persist: a reopen without the
    /// declarations reports the retained `.bib` ledger as `latex`, and that
    /// must not overwrite the record the next reopen needs.
    @discardableResult
    func refresh() async -> Bool {
        guard model.controllerAttached, model.controllerState.ready else {
            status = "no preview controller: kinds are reported by the helper only"
            return false
        }
        switch await model.project.helperRequest("snapshot", [:]) {
        case .success(let payload):
            guard let raw = payload["document_kinds"] as? [String: String] else {
                status = "helper snapshot has no document_kinds (older helper?)"
                return false
            }
            var parsed: [String: DocumentKind] = [:]
            for (path, value) in raw {
                guard let kind = DocumentKind(rawValue: value) else {
                    status = "helper reported unknown kind \(value) for \(path)"
                    return false
                }
                parsed[path] = kind
            }
            kinds = parsed
            generation = payload["membership_generation"] as? Int
            lastSeenGeneration = generation
            refreshes += 1
            let bib = bibliographyPaths
            status = bib.isEmpty ? "no bibliography source declared (\(parsed.count) latex)"
                : "bibliography: \(bib.joined(separator: ", ")) (membership g\(generation ?? -1))"
            return true
        case .failure(let e):
            status = "helper snapshot failed: \(e.message)"
            return false
        }
    }

    /// Declares `path` (a rooted project path, never the entry) as a
    /// bibliography source through the helper: `open_document` with the exact
    /// snapshot and `document_kind: "bibliography"`. On success the durable
    /// document is adopted into the shell membership by `ProjectDocuments`
    /// (which sees it in the next snapshot and reads it without another
    /// open), `document_kinds` is re-read, and the declarations are persisted.
    /// A path the helper already indexes as `latex` is refused: the helper
    /// changes a kind only through detach + typed reattach.
    func declareBibliography(_ path: String) async -> DeclareOutcome {
        guard model.controllerAttached, model.controllerState.ready else {
            return note(.refused("cannot declare \(path): no preview controller attached"))
        }
        guard path != model.project.entryPath else { return note(.refused("cannot declare the entry document \(path) as a bibliography")) }
        guard let snapshot = await model.project.refreshSnapshot() else {
            return note(.refused("cannot declare \(path): \(model.project.status)"))
        }
        if snapshot.versions[path] != nil {
            _ = await refresh()
            switch kinds[path] {
            case .bibliography: return note(.alreadyDeclared(path: path))
            case .latex: return note(.refused("\(path) is already open as latex; detach it first, then declare it as a bibliography"))
            case nil: return note(.refused("\(path) is indexed but the helper reported no kind for it"))
            }
        }
        let payload: [String: Any] = ["path": path, "source_versions": snapshot.versions,
                                      "membership_generation": snapshot.generation, "document_kind": DocumentKind.bibliography.rawValue]
        switch await model.project.helperRequest("open_document", payload) {
        case .success(let reply):
            if let e = reply["preview_error"] as? String { model.log("controller preview_error after declaring \(path): \(e)") }
        case .failure(let e):
            if e.message.contains("stale") {
                return note(.refused("helper refused declaring \(path): \(e.message) (snapshot g\(snapshot.generation) was superseded; try again)"))
            }
            return note(.refused("helper refused declaring \(path): \(e.message)"))
        }
        // The helper now lists the path in its snapshot, so the generic open
        // reads the durable document without a second membership change.
        let adopted = await model.project.openDocument(path)
        if case .refused(let why) = adopted { model.log("document kinds: declared \(path) but could not adopt it into the editor: \(why)") }
        guard await refresh(), kinds[path] == .bibliography else {
            return note(.refused("declared \(path) but the helper did not report it as bibliography (\(status))"))
        }
        persist()
        return note(.declared(path: path))
    }

    /// Detaches a declared bibliography source through `ProjectDocuments`
    /// (exact snapshot, session only) and persists the remaining declarations
    /// so the next reopen no longer supplies it.
    func undeclare(_ path: String, discardingEdits: Bool = false) async -> ProjectDocuments.DetachOutcome {
        guard kinds[path] == .bibliography else {
            let outcome = ProjectDocuments.DetachOutcome.refused("\(path) is not a declared bibliography source")
            status = "\(path) is not a declared bibliography source"
            return outcome
        }
        let outcome = await model.project.detachDocument(path, discardingEdits: discardingEdits)
        if case .detached = outcome {
            _ = await refresh()
            persist()
        } else if case .refused(let why) = outcome {
            status = why
        }
        return outcome
    }

    /// Writes the helper-reported bibliography paths for this saved project.
    private func persist() {
        guard let key = persistenceKey else {
            persistError = nil
            status += "; not persisted (unsaved buffer has no project)"
            return
        }
        let paths = bibliographyPaths
        do {
            try DocumentKindsStore.persist(bibliographyPaths: paths, projectRoot: key.root, entry: key.entry)
            persistedBibliographyPaths = paths
            persistError = nil
            model.log("document kinds: persisted bibliography \(paths) for \(key.entry) in \(DocumentKindsStore.url(projectRoot: key.root).path)")
        } catch {
            persistError = "\(error)"
            status += "; NOT persisted: \(error)"
            model.log("document kinds: persist failed: \(error)")
        }
    }

    /// Re-reads kinds whenever the controller status changes (ready, restart,
    /// close) or the project membership generation moves (open/detach).
    private func armTracking() {
        guard !armed else { return }
        armed = true
        withObservationTracking { [weak self] in
            guard let self else { return }
            _ = self.model.controllerStatus
            _ = self.model.project.membershipGeneration
        } onChange: { [weak self] in
            DispatchQueue.main.async { [weak self] in
                MainActor.assumeIsolated {
                    guard let self else { return }
                    self.armed = false
                    self.armTracking()
                    if !self.model.controllerAttached {
                        if !self.kinds.isEmpty { self.kinds = [:]; self.generation = nil; self.status = "no preview controller attached" }
                        return
                    }
                    guard self.model.controllerState.ready else { return }
                    let g = self.model.project.membershipGeneration
                    if self.kinds.isEmpty || g != self.lastSeenGeneration { Task { await self.refresh() } }
                }
            }
        }
    }

    @discardableResult
    private func note(_ outcome: DeclareOutcome) -> DeclareOutcome {
        let line = switch outcome {
        case .declared(let p): "declared \(p) as bibliography"
        case .alreadyDeclared(let p): "\(p) is already declared as bibliography"
        case .refused(let why): why
        }
        status = line
        FlashTeXLog.write("document kinds: " + line)
        return outcome
    }
}

// MARK: - model attachment

extension ShellModel {
    private static var documentKindsKey = 0
    /// Explicit document kinds (see `DocumentKinds`).
    var documentKinds: DocumentKinds {
        if let existing = objc_getAssociatedObject(self, &Self.documentKindsKey) as? DocumentKinds { return existing }
        let state = DocumentKinds(model: self)
        objc_setAssociatedObject(self, &Self.documentKindsKey, state, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
        return state
    }
}

// MARK: - UI

/// Read-only caption for the header row: the active document's helper-reported
/// kind when it is a declared bibliography, or the count of declared sources.
/// Shows nothing when the helper reported no bibliography kind at all.
struct DocumentKindIndicator: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        let kinds = model.documentKinds
        let declared = kinds.bibliographyPaths
        if !declared.isEmpty {
            let active = kinds.kind(of: model.activePath)
            Text(active == .bibliography ? "bibliography (declared)" : "\(declared.count) bibliography source\(declared.count == 1 ? "" : "s")")
                .font(.caption).foregroundStyle(.secondary)
                .help(kinds.status + (kinds.persistError.map { "\nnot persisted: \($0)" } ?? ""))
                .accessibilityIdentifier("document-kind-indicator")
        }
    }
}

/// Project menu section: declared bibliography sources (helper-reported), a
/// rooted open panel to declare another one, and per-source undeclare.
struct DocumentKindsMenuSection: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        let kinds = model.documentKinds
        Divider()
        if kinds.bibliographyPaths.isEmpty {
            Text("No bibliography source declared")
        }
        ForEach(kinds.bibliographyPaths, id: \.self) { path in
            Button("Undeclare \(path) (detach, this session)") {
                Task {
                    switch await model.documentKinds.undeclare(path) {
                    case .refused(let why): model.captureNote = why
                    case .detached(let p): model.captureNote = "Undeclared \(p) — " + ProjectDocuments.detachScopeNote
                    }
                }
            }
        }
        Button("Declare Bibliography Source…") { declareThroughPanel() }
            .disabled(model.documentURL == nil || !model.controllerAttached)
            .help("Declares a rooted project file as a bibliography source through the helper (never inferred from its extension); the declaration is persisted for reopen")
    }

    private func declareThroughPanel() {
        guard let root = model.project.projectRoot else { return }
        let panel = NSOpenPanel()
        panel.directoryURL = root
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = false
        panel.message = "Choose a file under \(root.path) to declare as a bibliography source"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        let rootPath = root.standardizedFileURL.path + "/"
        let chosen = url.standardizedFileURL.path
        guard chosen.hasPrefix(rootPath) else {
            model.captureNote = "\(url.lastPathComponent) is outside the project root \(root.path)"
            return
        }
        let path = String(chosen.dropFirst(rootPath.count))
        Task {
            switch await model.documentKinds.declareBibliography(path) {
            case .declared(let p): model.captureNote = "Declared \(p) as a bibliography source"
            case .alreadyDeclared(let p): model.captureNote = "\(p) is already declared as a bibliography source"
            case .refused(let why): model.captureNote = why
            }
        }
    }
}
