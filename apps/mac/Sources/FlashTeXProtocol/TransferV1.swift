import Foundation

/// Models for the capture bridge contract (docs/contracts/transfer-v1.md,
/// consumed at bridge commit b5ca96b). Additive to runtime v1: the same
/// `{protocol_version, id, type, payload}` envelope, snake_case keys, zero-based
/// end-exclusive UTF-8 byte offsets, and `error {code, message}` replies.
/// `capture_submit` reuses `RuntimeV1.CaptureSubmit`.
public enum TransferV1 {
    /// Each line, including its newline, is at most 12 MiB in either direction.
    public static let maxLineBytes = 12 * 1024 * 1024
    public static let maxRequestIDBytes = 128
    public static let maxDocumentBytes = 8 * 1024 * 1024
    public static let maxImageBytes = 8 * 1024 * 1024
    public static let maxInstructionBytes = 4096

    /// Request type → reply type, exactly as `crates/bridge/src/main.rs` dispatches.
    public enum Request: String, CaseIterable {
        case documentOpen = "document_open"
        case documentEdit = "document_edit"
        case destinationPin = "destination_pin"
        case captureSubmit = "capture_submit"
        case captureConvert = "capture_convert"
        case capturePrepareInsert = "capture_prepare_insert"
        case captureApplied = "capture_applied"
        case captureStatus = "capture_status"
        case captureReject = "capture_reject"

        public var replyType: String {
            switch self {
            case .documentOpen: "document_opened"
            case .documentEdit: "document_updated"
            case .destinationPin: "destination_pinned"
            case .captureSubmit: "capture_received"
            case .captureConvert: "capture_proposal"
            case .capturePrepareInsert: "capture_edit"
            case .captureApplied: "capture_application_received"
            case .captureStatus: "capture_status"
            case .captureReject: "capture_rejected"
            }
        }
    }

    /// `error` reply payload. Codes are stable strings such as `provider_disabled`.
    public struct ErrorPayload: Codable, Equatable, Error {
        public var code: String
        public var message: String
        public init(code: String, message: String) { self.code = code; self.message = message }
    }

    public struct Empty: Codable, Equatable { public init() {} }

    // MARK: documents and destinations

    public struct DocumentOpen: Codable, Equatable {
        public var projectId: String
        public var path: String
        public var revision: Int
        public var text: String
        enum CodingKeys: String, CodingKey { case projectId = "project_id", path, revision, text }
        public init(projectId: String, path: String, revision: Int, text: String) {
            self.projectId = projectId; self.path = path; self.revision = revision; self.text = text
        }
    }

    public struct DocumentEdit: Codable, Equatable {
        public var projectId: String
        public var path: String
        public var baseRevision: Int
        public var revision: Int
        public var startByte: Int
        public var endByte: Int
        public var replacement: String
        enum CodingKeys: String, CodingKey {
            case projectId = "project_id", path, baseRevision = "base_revision", revision
            case startByte = "start_byte", endByte = "end_byte", replacement
        }
        public init(projectId: String, path: String, baseRevision: Int, revision: Int,
                    startByte: Int, endByte: Int, replacement: String) {
            self.projectId = projectId; self.path = path; self.baseRevision = baseRevision
            self.revision = revision; self.startByte = startByte; self.endByte = endByte
            self.replacement = replacement
        }
    }

    public struct DocumentUpdated: Codable, Equatable {
        public var revision: Int
        public init(revision: Int) { self.revision = revision }
    }

    public struct DestinationPin: Codable, Equatable {
        public var destinationId: String
        public var projectId: String
        public var path: String
        public var revision: Int
        public var startByte: Int
        public var endByte: Int
        enum CodingKeys: String, CodingKey {
            case destinationId = "destination_id", projectId = "project_id", path, revision
            case startByte = "start_byte", endByte = "end_byte"
        }
        public init(destinationId: String, projectId: String, path: String, revision: Int, startByte: Int, endByte: Int) {
            self.destinationId = destinationId; self.projectId = projectId; self.path = path
            self.revision = revision; self.startByte = startByte; self.endByte = endByte
        }
    }

    /// Immutable binding of a destination to the exact source it was pinned in.
    public struct AnchorBinding: Codable, Equatable {
        public var projectId: String
        public var path: String
        public var revision: Int
        public var startByte: Int
        public var endByte: Int
        public var sourceSha256: String
        enum CodingKeys: String, CodingKey {
            case projectId = "project_id", path, revision, startByte = "start_byte", endByte = "end_byte"
            case sourceSha256 = "source_sha256"
        }
    }

    public struct Anchor: Codable, Equatable {
        public var destinationId: String
        public var projectId: String
        public var path: String
        public var pinnedRevision: Int
        public var currentRevision: Int
        public var startByte: Int
        public var endByte: Int
        public var valid: Bool
        public var binding: AnchorBinding
        enum CodingKeys: String, CodingKey {
            case destinationId = "destination_id", projectId = "project_id", path
            case pinnedRevision = "pinned_revision", currentRevision = "current_revision"
            case startByte = "start_byte", endByte = "end_byte", valid, binding
        }
    }

    // MARK: capture lifecycle

    public struct CaptureReceived: Codable, Equatable {
        public var captureId: String
        public var durable: Bool
        public var hasProposal: Bool
        public var applied: Bool
        enum CodingKeys: String, CodingKey {
            case captureId = "capture_id", durable, hasProposal = "has_proposal", applied
        }
        public init(captureId: String, durable: Bool, hasProposal: Bool, applied: Bool) {
            self.captureId = captureId; self.durable = durable; self.hasProposal = hasProposal; self.applied = applied
        }
    }

    public struct CaptureConvert: Codable, Equatable {
        public var captureId: String
        public var supportedFeatures: [String]
        enum CodingKeys: String, CodingKey { case captureId = "capture_id", supportedFeatures = "supported_features" }
        public init(captureId: String, supportedFeatures: [String]) {
            self.captureId = captureId; self.supportedFeatures = supportedFeatures
        }
    }

    public struct CaptureID: Codable, Equatable {
        public var captureId: String
        enum CodingKeys: String, CodingKey { case captureId = "capture_id" }
        public init(captureId: String) { self.captureId = captureId }
    }

    public struct CapturePrepareInsert: Codable, Equatable {
        public var captureId: String
        public var expectedRevision: Int
        public var approved: Bool
        enum CodingKeys: String, CodingKey {
            case captureId = "capture_id", expectedRevision = "expected_revision", approved
        }
        public init(captureId: String, expectedRevision: Int, approved: Bool) {
            self.captureId = captureId; self.expectedRevision = expectedRevision; self.approved = approved
        }
    }

    /// A prepared, persisted edit. Preparation never edits source; the Mac
    /// verifies every field against its buffer before applying it once.
    public struct CaptureEdit: Codable, Equatable {
        public var captureId: String
        public var editId: String
        public var projectId: String
        public var path: String
        public var expectedRevision: Int
        public var startByte: Int
        public var endByte: Int
        public var removedText: String
        public var replacement: String
        public var documentBeforeSha256: String
        enum CodingKeys: String, CodingKey {
            case captureId = "capture_id", editId = "edit_id", projectId = "project_id", path
            case expectedRevision = "expected_revision", startByte = "start_byte", endByte = "end_byte"
            case removedText = "removed_text", replacement, documentBeforeSha256 = "document_before_sha256"
        }
        public init(captureId: String, editId: String, projectId: String, path: String, expectedRevision: Int,
                    startByte: Int, endByte: Int, removedText: String, replacement: String, documentBeforeSha256: String) {
            self.captureId = captureId; self.editId = editId; self.projectId = projectId; self.path = path
            self.expectedRevision = expectedRevision; self.startByte = startByte; self.endByte = endByte
            self.removedText = removedText; self.replacement = replacement; self.documentBeforeSha256 = documentBeforeSha256
        }
    }

    public struct CaptureApplied: Codable, Equatable {
        public var captureId: String
        public var editId: String
        public var newRevision: Int
        enum CodingKeys: String, CodingKey { case captureId = "capture_id", editId = "edit_id", newRevision = "new_revision" }
        public init(captureId: String, editId: String, newRevision: Int) {
            self.captureId = captureId; self.editId = editId; self.newRevision = newRevision
        }
    }

    /// Journaled proposal as it appears inside `capture_status` (no capture_id).
    public struct Proposal: Codable, Equatable {
        public var latex: String
        public var ambiguities: [String]
        public var requiredDependencies: [String]
        enum CodingKeys: String, CodingKey { case latex, ambiguities, requiredDependencies = "required_dependencies" }
    }

    public struct AppliedEdit: Codable, Equatable {
        public var editId: String
        public var newRevision: Int
        enum CodingKeys: String, CodingKey { case editId = "edit_id", newRevision = "new_revision" }
    }

    public struct CaptureStatus: Codable, Equatable {
        public var captureId: String
        public var proposal: Proposal?
        public var prepared: CaptureEdit?
        public var applied: AppliedEdit?
        public var rejected: Bool
        enum CodingKeys: String, CodingKey { case captureId = "capture_id", proposal, prepared, applied, rejected }
    }

    // MARK: validation helpers (mirror the bridge's limits so bad requests fail locally)

    /// 1–128 ASCII alphanumerics, `-` or `_`.
    public static func isValidIdentifier(_ id: String) -> Bool {
        let u = id.utf8
        guard !u.isEmpty, u.count <= 128 else { return false }
        return u.allSatisfy { ($0 >= 0x30 && $0 <= 0x39) || ($0 >= 0x41 && $0 <= 0x5A) || ($0 >= 0x61 && $0 <= 0x7A) || $0 == 0x2D || $0 == 0x5F }
    }

    /// Normalized relative path: no leading `/`, no `\`, `:`, NUL, empty, `.` or `..` segments.
    public static func isValidRelativePath(_ path: String) -> Bool {
        guard !path.isEmpty, !path.hasPrefix("/"), !path.contains("\\"), !path.contains(":"), !path.contains("\0") else { return false }
        return !path.split(separator: "/", omittingEmptySubsequences: false).contains { $0.isEmpty || $0 == "." || $0 == ".." }
    }
}
