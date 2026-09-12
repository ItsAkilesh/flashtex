import CryptoKit
import Foundation

/// Client-side stub for the proposed transfer-v1 `capture_list` request
/// (`docs/handoffs/transfer-v1-capture-list.md` §3–§6): the wire shapes, the
/// per-row outcome, the duplicate-free merge into the session's capture rows,
/// the bounded pagination and the "Restore destination" gate — all pure, and
/// all behind `CaptureListFeature`, which is off by default. No bridge call is
/// made anywhere here: the bridge owner has not accepted the request yet, so
/// nothing in this file talks to `BridgeSession`/`BridgeClient`; the single
/// call-site hook (after `session.open` in `attachBridgeAndWait`) is the
/// parent's, and stays out until the bridge answers `capture_list`.
enum CaptureListFeature {
    static let defaultsKey = "flashtex.captureList.enabled"
    /// `FLASHTEX_CAPTURE_LIST=1` or the defaults key; anything else is off.
    static func isEnabled(environment: [String: String] = ProcessInfo.processInfo.environment,
                          defaults: UserDefaults = .standard) -> Bool {
        if let v = environment["FLASHTEX_CAPTURE_LIST"] { return v == "1" }
        return defaults.bool(forKey: defaultsKey)
    }
    /// Pages fetched per attach before the UI is told to load more (§6).
    static let maxPagesPerAttach = 8
    static let defaultLimit = 64
}

enum CaptureList {
    // MARK: wire shapes (§3; snake_case, additive to transfer-v1)

    struct DestinationFilter: Codable, Equatable {
        var path: String?
        var destinationId: String?
        enum CodingKeys: String, CodingKey { case path, destinationId = "destination_id" }
        static func path(_ p: String) -> DestinationFilter { .init(path: p, destinationId: nil) }
        static func destinationId(_ id: String) -> DestinationFilter { .init(path: nil, destinationId: id) }
    }

    struct Request: Codable, Equatable {
        var projectId: String
        var storeId: String?
        var destination: DestinationFilter?
        var sinceReceipt: String?
        var limit: Int = CaptureListFeature.defaultLimit
        var includeTerminal: Bool = false
        enum CodingKeys: String, CodingKey {
            case projectId = "project_id", storeId = "store_id", destination, sinceReceipt = "since_receipt", limit
            case includeTerminal = "include_terminal"
        }
        static let type = "capture_list"
    }

    enum Status: String, Codable { case journaled, staged, inserted, rejected, failed }

    struct Binding: Codable, Equatable {
        var projectId: String
        var path: String
        var revision: Int
        var startByte: Int
        var endByte: Int
        var sourceSha256: String
        enum CodingKeys: String, CodingKey {
            case projectId = "project_id", path, revision, startByte = "start_byte", endByte = "end_byte", sourceSha256 = "source_sha256"
        }
    }
    struct Prepared: Codable, Equatable {
        var editId: String
        var expectedRevision: Int
        enum CodingKeys: String, CodingKey { case editId = "edit_id", expectedRevision = "expected_revision" }
    }
    struct Applied: Codable, Equatable {
        var editId: String
        var newRevision: Int
        enum CodingKeys: String, CodingKey { case editId = "edit_id", newRevision = "new_revision" }
    }

    struct Row: Codable, Equatable {
        var receipt: String
        var captureId: String
        var status: Status
        var requestSha256: String?
        var imageMimeType: String?
        var imageBytes: Int?
        var instructionsBytes: Int?
        var destinationId: String?
        var baseRevision: Int?
        var destinationBinding: Binding?
        var bindingMissing: Bool = false
        var hasProposal: Bool = false
        var proposalSha256: String?
        var prepared: Prepared?
        var applied: Applied?
        var rejected: Bool = false
        var reason: String?
        enum CodingKeys: String, CodingKey {
            case receipt, captureId = "capture_id", status, requestSha256 = "request_sha256", imageMimeType = "image_mime_type"
            case imageBytes = "image_bytes", instructionsBytes = "instructions_bytes", destinationId = "destination_id"
            case baseRevision = "base_revision", destinationBinding = "destination_binding", bindingMissing = "binding_missing"
            case hasProposal = "has_proposal", proposalSha256 = "proposal_sha256", prepared, applied, rejected, reason
        }
    }

    struct Reply: Codable, Equatable {
        var projectId: String
        var storeId: String?
        var captures: [Row]
        var nextReceipt: String?
        var truncated: Bool
        enum CodingKeys: String, CodingKey {
            case projectId = "project_id", storeId = "store_id", captures, nextReceipt = "next_receipt", truncated
        }
    }

    /// Refusal codes the handoff defines (§4); `storeUnavailable` is the only transient one.
    enum Refusal: String {
        case storeMismatch = "store_mismatch", destinationUnbound = "destination_unbound", badRequest = "bad_request"
        case invalidLimit = "invalid_limit", invalidCursor = "invalid_cursor", storeUnavailable = "store_unavailable"
        var isTransient: Bool { self == .storeUnavailable }
    }

    // MARK: per-row outcome (§6)

    /// What the edit ledger already knows; the listing only cross-checks it.
    struct LedgerKnowledge: Equatable {
        var knownEditIds: Set<String> = []
        var confirmedEditIds: Set<String> = []
    }

    enum Outcome: Equatable {
        /// Journaled before this session with an intact binding: review to continue.
        case pending
        case needsReselection(note: String)
        /// A staged edit the ledger knows: `reconcile` already decided its state.
        case leftToReconcile
        case confirmed
        /// Inserted on the bridge but not confirmed in this ledger: shown for audit, never re-applied.
        case insertedUnconfirmed(note: String)
        case rejected
        case failed(reason: String)

        var note: String {
            switch self {
            case .pending: return "received before relaunch; review to continue"
            case .needsReselection(let n), .insertedUnconfirmed(let n): return n
            case .leftToReconcile: return "staged edit reconciled through the ledger"
            case .confirmed: return "inserted and confirmed"
            case .rejected: return "rejected"
            case .failed(let r): return r
            }
        }
    }

    static func outcome(for row: Row, ledger: LedgerKnowledge) -> Outcome {
        switch row.status {
        case .journaled:
            return row.bindingMissing || row.destinationBinding == nil
                ? .needsReselection(note: "legacy record: capture again") : .pending
        case .staged:
            guard let edit = row.prepared?.editId, ledger.knownEditIds.contains(edit) else {
                return .needsReselection(note: "issued edit never recorded here — resolve before a new attempt")
            }
            return .leftToReconcile
        case .inserted:
            if let edit = row.applied?.editId, ledger.confirmedEditIds.contains(edit) { return .confirmed }
            return .insertedUnconfirmed(note: "inserted on the bridge; no confirmed receipt in this ledger (audit)")
        case .rejected: return .rejected
        case .failed: return .failed(reason: row.reason ?? "record could not be loaded")
        }
    }

    // MARK: merge (§6 "merged into `captures` by capture_id")

    struct MergeDecision: Equatable {
        enum Kind: Equatable { case add(Outcome), keepExisting(BridgeSession.CaptureState) }
        var captureId: String
        var kind: Kind
    }

    /// Rows this session already holds (its own `capture_submit`s, or an
    /// earlier page) keep their state; every other row is added once with its
    /// outcome. Order: existing rows first, then listed rows in receipt order.
    static func merge(rows: [Row], into existing: [BridgeSession.Capture], ledger: LedgerKnowledge) -> [MergeDecision] {
        var seen = Set(existing.map(\.captureId))
        var decisions = existing.map { MergeDecision(captureId: $0.captureId, kind: .keepExisting($0.state)) }
        for row in rows.sorted(by: { $0.receipt < $1.receipt }) where !seen.contains(row.captureId) {
            seen.insert(row.captureId)
            decisions.append(.init(captureId: row.captureId, kind: .add(outcome(for: row, ledger: ledger))))
        }
        return decisions
    }

    // MARK: pagination (§6 "truncated:true → repeat … at most 8 pages per attach")

    struct Pager: Equatable {
        enum Phase: Equatable {
            case idle
            /// The request to send next (page `number`, 1-based).
            case listing(number: Int, request: Request)
            case done(pages: Int)
            /// The page cap was reached with more rows on the bridge.
            case moreAvailable(pages: Int, note: String)
            case failed(note: String)
        }
        private(set) var phase: Phase = .idle
        private(set) var rows: [Row] = []
        let base: Request
        let maxPages: Int

        init(base: Request, maxPages: Int = CaptureListFeature.maxPagesPerAttach) {
            self.base = base; self.maxPages = max(1, maxPages)
        }

        /// Starts the first page (a no-op unless the feature is on).
        mutating func start(enabled: Bool) {
            guard enabled, case .idle = phase else { return }
            phase = .listing(number: 1, request: base)
        }

        mutating func received(_ reply: Reply) {
            guard case .listing(let n, _) = phase else { return }
            rows += reply.captures
            guard reply.truncated, let next = reply.nextReceipt else { phase = .done(pages: n); return }
            guard n < maxPages else {
                phase = .moreAvailable(pages: n, note: "more journaled captures; open the capture list to load more")
                return
            }
            var r = base; r.sinceReceipt = next
            phase = .listing(number: n + 1, request: r)
        }

        /// Any refusal or transport failure ends the listing; rows already
        /// received stay (they are still true), the note goes to the capture note.
        mutating func failed(code: String?, message: String) {
            guard case .listing = phase else { return }
            let refusal = code.flatMap(Refusal.init(rawValue:))
            phase = .failed(note: "capture listing \(refusal?.isTransient == true ? "unavailable, retry" : "refused"): \(code ?? "transport") — \(message)")
        }

        /// Edit > Retry Bridge Reconciliation re-runs the listing from the start.
        mutating func reset() { phase = .idle; rows = [] }
    }

    // MARK: "Restore destination" gate (§6)

    /// The exact re-pin is offered only when the active buffer is byte-for-byte
    /// what the capture was bound to and the ledger-aligned revision matches;
    /// otherwise the row says "capture again or reject" and no pin is sent.
    static func canRestoreDestination(_ row: Row, activeText: String, editorRevision: Int) -> Bool {
        guard row.status == .journaled, let b = row.destinationBinding, !row.bindingMissing else { return false }
        return b.revision == editorRevision && sha256Hex(activeText) == b.sourceSha256.lowercased()
    }

    static func sha256Hex(_ text: String) -> String {
        SHA256.hash(data: Data(text.utf8)).map { String(format: "%02x", $0) }.joined()
    }
}
