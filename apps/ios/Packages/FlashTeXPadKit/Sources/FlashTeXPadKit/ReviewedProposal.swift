import Foundation
import FlashTeXProtocol

/// Reviewed-proposal shapes of the assistant-context helper
/// (crates/assistant-context/README.md, "review" / "approve" operations;
/// recorded in crates/assistant-context/examples/review-workflow.json).
///
/// KEEP IN SYNC with the helper's JSON: these are decode-only mirrors so the
/// iPad can display and apply a proposal the helper produced. Nothing here is
/// a new wire message. The Mac's own mirror lives in
/// apps/mac/Sources/FlashTeXMac/ProposalPreview.swift (`Explanation`,
/// `ApprovedAmendment`).
public enum ReviewedProposal {
    /// `edits[].location` — the same zero-based end-exclusive UTF-8 byte range
    /// as runtime-v1 `SourceRange`.
    public typealias Location = RuntimeV1.SourceRange

    public struct Edit: Codable, Equatable {
        public var location: Location
        public var removedText: String
        public var replacement: String
        enum CodingKeys: String, CodingKey { case location, removedText = "removed_text", replacement }
        public init(location: Location, removedText: String, replacement: String) {
            self.location = location; self.removedText = removedText; self.replacement = replacement
        }
    }

    /// Payload of the helper's `proposal_review` reply.
    public struct ReviewPayload: Codable, Equatable {
        public var contextId: String
        public var explanation: String
        public var edits: [Edit]
        enum CodingKeys: String, CodingKey { case contextId = "context_id", explanation, edits }
    }

    /// The helper's `proposal_review` reply: immutable proposal + review digest.
    /// `applied` is always false from the helper; a reply claiming otherwise is refused.
    public struct Review: Codable, Equatable {
        public var type: String
        public var applied: Bool
        public var requiresUserApproval: Bool
        public var reviewId: String
        public var payload: ReviewPayload
        enum CodingKeys: String, CodingKey {
            case type, applied, requiresUserApproval = "requires_user_approval", reviewId = "review_id", payload
        }
    }

    /// One edit inside `approved_group.payload.group.edits` (flat offsets).
    public struct GroupEdit: Codable, Equatable {
        public var startByte: Int
        public var endByte: Int
        public var removedText: String
        public var replacement: String
        enum CodingKeys: String, CodingKey {
            case startByte = "start_byte", endByte = "end_byte", removedText = "removed_text", replacement
        }
    }

    public struct Group: Codable, Equatable {
        public var commandId: String
        public var edits: [GroupEdit]
        public var expectedRevision: Int
        public var expectedSha256: String
        public var label: String
        enum CodingKeys: String, CodingKey {
            case commandId = "command_id", edits, expectedRevision = "expected_revision"
            case expectedSha256 = "expected_sha256", label
        }
    }

    public struct ApprovedPayload: Codable, Equatable {
        public var group: Group
        public var path: String
        public var projectId: String
        public var requestId: String
        public var reviewId: String
        enum CodingKeys: String, CodingKey {
            case group, path, projectId = "project_id", requestId = "request_id", reviewId = "review_id"
        }
    }

    /// The helper's `approved_group` reply: input to the ledger, `applied:false`.
    public struct Approved: Codable, Equatable {
        public var type: String
        public var applied: Bool
        public var payload: ApprovedPayload
    }

    public enum DecodeError: Error, Equatable, CustomStringConvertible {
        case unexpectedType(expected: String, actual: String)
        case claimsApplied
        case missingStep(String)
        public var description: String {
            switch self {
            case .unexpectedType(let e, let a): return "expected \(e), got \(a)"
            case .claimsApplied: return "helper reply claims applied:true; refused"
            case .missingStep(let s): return "fixture has no \(s) step"
            }
        }
    }

    public static func decodeReview(_ data: Data) throws -> Review {
        let r = try JSONDecoder().decode(Review.self, from: data)
        guard r.type == "proposal_review" else { throw DecodeError.unexpectedType(expected: "proposal_review", actual: r.type) }
        guard !r.applied else { throw DecodeError.claimsApplied }
        return r
    }

    public static func decodeApproved(_ data: Data) throws -> Approved {
        let a = try JSONDecoder().decode(Approved.self, from: data)
        guard a.type == "approved_group" else { throw DecodeError.unexpectedType(expected: "approved_group", actual: a.type) }
        guard !a.applied else { throw DecodeError.claimsApplied }
        return a
    }

    /// The recorded four-step compiler→prepare→review→approve fixture
    /// (`crates/assistant-context/examples/review-workflow.json`), as the iPad
    /// consumes it: the compiled `main.tex` text, the compiler's diagnostics,
    /// the `proposal_review` and the `approved_group`. The fixture is
    /// explicitly synthetic (`provider_called:false`); no model was called.
    public struct WorkflowFixture: Equatable {
        public var sourceText: String
        public var sourcePath: String
        public var compileResult: RuntimeV1.CompileResult
        public var review: Review
        public var approved: Approved
    }

    public static func decodeWorkflowFixture(_ data: Data) throws -> WorkflowFixture {
        struct Step: Decodable { var request: JSONValue; var response: JSONValue }
        struct File: Decodable { var kind: String; var providerCalled: Bool; var sourceMutated: Bool; var steps: [Step]
            enum CodingKeys: String, CodingKey { case kind, providerCalled = "provider_called", sourceMutated = "source_mutated", steps }
        }
        let f = try JSONDecoder().decode(File.self, from: data)
        var source: (String, String)?
        var compile: RuntimeV1.CompileResult?
        var review: Review?
        var approved: Approved?
        let enc = JSONEncoder()
        for step in f.steps {
            let reqType = step.request["type"]?.string ?? step.request["operation"]?.string ?? ""
            switch reqType {
            case "compile":
                let bytes = try enc.encode(step.response)
                let env: RuntimeV1.Envelope<RuntimeV1.CompileResult> = try RuntimeV1.decodeCompileResultReference(bytes)
                compile = env.payload
                if let docs = step.request["payload"]?["documents"]?.array, let first = docs.first,
                   let text = first["text"]?.string, let path = first["path"]?.string {
                    source = (path, text)
                }
            case "review": review = try decodeReview(try enc.encode(step.response))
            case "approve": approved = try decodeApproved(try enc.encode(step.response))
            default: break
            }
        }
        guard let source else { throw DecodeError.missingStep("compile (documents)") }
        guard let compile else { throw DecodeError.missingStep("compile") }
        guard let review else { throw DecodeError.missingStep("review") }
        guard let approved else { throw DecodeError.missingStep("approve") }
        return WorkflowFixture(sourceText: source.1, sourcePath: source.0, compileResult: compile, review: review, approved: approved)
    }
}

/// Minimal JSON tree for re-encoding fixture steps into typed decoders.
public enum JSONValue: Codable, Equatable {
    case null, bool(Bool), number(Double), string(String), array([JSONValue]), object([String: JSONValue])

    public init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if c.decodeNil() { self = .null }
        else if let b = try? c.decode(Bool.self) { self = .bool(b) }
        else if let n = try? c.decode(Double.self) { self = .number(n) }
        else if let s = try? c.decode(String.self) { self = .string(s) }
        else if let a = try? c.decode([JSONValue].self) { self = .array(a) }
        else { self = .object(try c.decode([String: JSONValue].self)) }
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.singleValueContainer()
        switch self {
        case .null: try c.encodeNil()
        case .bool(let b): try c.encode(b)
        case .number(let n): if n == n.rounded(), abs(n) < 1e15 { try c.encode(Int64(n)) } else { try c.encode(n) }
        case .string(let s): try c.encode(s)
        case .array(let a): try c.encode(a)
        case .object(let o): try c.encode(o)
        }
    }

    public subscript(key: String) -> JSONValue? { if case .object(let o) = self { return o[key] } else { return nil } }
    public var string: String? { if case .string(let s) = self { return s } else { return nil } }
    public var array: [JSONValue]? { if case .array(let a) = self { return a } else { return nil } }
}
