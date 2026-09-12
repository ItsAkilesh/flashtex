import Foundation

/// Recovery-oriented record of what happened to the review sheet's assistant
/// requests (ProposalPreview.swift `explain()` and friends). `ProposalPreview`
/// keeps the request/stage guard that decides whether a helper/provider reply
/// is current; this file only *names* the recovery outcomes so the sheet can
/// show an honest note ("explanation expired: the document changed", "provider
/// gave no reply within 1 s; a retry starts a new request") and tests can
/// assert the exact sequence. Nothing here launches a process or writes text.
enum AssistantRecoveryEvent: Equatable {
    enum Party: Equatable {
        case helper, provider
        var label: String { self == .helper ? "helper" : "provider" }
    }

    /// The reviewer pressed Cancel while `requestId` was running at `stage`.
    case cancelledByReviewer(requestId: String, stage: ProposalPreview.ExplanationStage)
    /// The review sheet closed (the proposal was approved for insertion,
    /// rejected, or dismissed) while `requestId` was running at `stage`: the
    /// child is terminated and its reply, if any, is discarded unseen.
    case reviewClosed(requestId: String, stage: ProposalPreview.ExplanationStage)
    /// The bound document moved under the request (editor revision or document
    /// text changed): the helper's binding can no longer be satisfied, so the
    /// request — running or displayed — is discarded before any reply is trusted.
    case expired(requestId: String, fromRevision: Int, toRevision: Int)
    /// The proposal text or anchor changed: the request describes an older draft.
    case superseded(requestId: String, what: String)
    /// No reply within the configured deadline; the child was terminated. A
    /// reply that lands after the deadline is never accepted, even if complete.
    case timedOut(requestId: String, party: Party, stage: ProposalPreview.ExplanationStage, seconds: TimeInterval)
    /// The child exited without a usable reply (crash, signal, nonzero exit).
    case exited(requestId: String, party: Party, stage: ProposalPreview.ExplanationStage, code: Int32)
    /// The helper answered an `error` reply (its own validation refused the
    /// request, e.g. "source snapshot is stale").
    case refused(requestId: String, stage: ProposalPreview.ExplanationStage, message: String)
    /// A reply for a request or stage that is no longer current arrived and was
    /// dropped without inspection.
    case lateReplyDiscarded(requestId: String, stage: ProposalPreview.ExplanationStage)

    var requestId: String {
        switch self {
        case .cancelledByReviewer(let id, _), .reviewClosed(let id, _), .expired(let id, _, _), .superseded(let id, _),
             .timedOut(let id, _, _, _), .exited(let id, _, _, _), .refused(let id, _, _), .lateReplyDiscarded(let id, _):
            return id
        }
    }

    /// Reviewer-facing note. Always says what happens next; never claims an edit.
    var note: String {
        switch self {
        case .cancelledByReviewer(_, let stage):
            return "cancelled during \(stage); any late reply is discarded. Explain starts a new request."
        case .reviewClosed(_, let stage):
            return "review closed during \(stage); the request was stopped and any late reply is discarded. Nothing was applied."
        case .expired(_, let from, let to):
            return "explanation expired: the document changed (revision \(from) → \(to)); its reply, if any, is discarded and nothing was applied. Explain again for the current text."
        case .superseded(_, let what):
            return "explanation discarded: the \(what) changed; nothing was applied. Explain again for the current draft."
        case .timedOut(_, let party, let stage, let seconds):
            return "\(party.label) gave no reply within \(Int(seconds.rounded(.up))) s (\(stage)) and was stopped; a late reply is ignored. Explain again starts a new request."
        case .exited(_, let party, let stage, let code):
            return "\(party.label) exited (\(code)) during \(stage); it is relaunched on the next request. Nothing was applied."
        case .refused(_, let stage, let message):
            return "helper refused (\(stage)): \(message). Nothing was applied."
        case .lateReplyDiscarded(let id, let stage):
            return "a late \(stage) reply for \(id) was discarded (not the current request)."
        }
    }

    /// Names a child-process failure for `requestId`/`stage`. `.cancelled` is
    /// recorded by whoever cancelled (reviewer or expiry), so it maps to nil;
    /// launch failures and oversized replies are plain failures, not recovery
    /// events, and map to nil too.
    static func classify(_ failure: OneShotProcess.Failure, requestId: String,
                         stage: ProposalPreview.ExplanationStage) -> AssistantRecoveryEvent? {
        let party: Party = stage == .provider ? .provider : .helper
        switch failure {
        case .timeout(let t): return .timedOut(requestId: requestId, party: party, stage: stage, seconds: t)
        case .exited(let code, let output, _):
            if party == .helper, let obj = try? JSONSerialization.jsonObject(with: output) as? [String: Any],
               obj["type"] as? String == "error", let m = obj["message"] as? String {
                return .refused(requestId: requestId, stage: stage, message: m)
            }
            return .exited(requestId: requestId, party: party, stage: stage, code: code)
        case .cancelled, .launch, .outputTooLarge: return nil
        }
    }

    /// Names why a running or displayed explanation bound to `old` no longer
    /// applies to `new`: a document/revision move is an expiry; a proposal or
    /// anchor move supersedes it.
    static func forChange(requestId: String, old: (input: ProposalPreview.Input, latex: String),
                          new: (input: ProposalPreview.Input, latex: String)) -> AssistantRecoveryEvent {
        if old.input.editorRevision != new.input.editorRevision || old.input.documents != new.input.documents
            || old.input.entryPath != new.input.entryPath || old.input.projectId != new.input.projectId {
            return .expired(requestId: requestId, fromRevision: old.input.editorRevision, toRevision: new.input.editorRevision)
        }
        if old.latex != new.latex { return .superseded(requestId: requestId, what: "proposal text") }
        return .superseded(requestId: requestId, what: "insertion anchor")
    }
}

/// Bounded, append-only log of recovery events for one review sheet.
struct AssistantRecoveryLog: Equatable {
    static let capacity = 32
    private(set) var events: [AssistantRecoveryEvent] = []

    var last: AssistantRecoveryEvent? { events.last }
    var lateReplies: Int { events.filter { if case .lateReplyDiscarded = $0 { return true }; return false }.count }

    mutating func record(_ event: AssistantRecoveryEvent) {
        events.append(event)
        if events.count > Self.capacity { events.removeFirst(events.count - Self.capacity) }
    }

    func events(for requestId: String) -> [AssistantRecoveryEvent] { events.filter { $0.requestId == requestId } }
}

extension ProposalPreview.ExplanationState {
    /// True when nothing is running and no model output is displayed: the
    /// sheet is back where it was before Explain was pressed (possibly with a
    /// note saying why).
    var isIdleForReviewer: Bool {
        switch self {
        case .idle, .unavailable, .failed, .cancelled: return true
        case .preparing, .prepared, .awaitingProvider, .validating, .ready, .approving, .approved: return false
        }
    }
}
