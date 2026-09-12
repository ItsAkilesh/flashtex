#![cfg(feature = "bridge-integration")]
use flashtex_bridge::Proposal;
use flashtex_conversion_jobs::bridge_adapter::{native::ContextIdentity, review::*};
fn context(revision: u64) -> ContextIdentity {
    ContextIdentity {
        project_id: "project".into(),
        path: "main.tex".into(),
        revision,
        sha256: if revision == 1 {
            "a".repeat(64)
        } else {
            "b".repeat(64)
        },
    }
}
fn proposal() -> Proposal {
    Proposal {
        latex: "$x$".into(),
        ambiguities: vec![],
        required_dependencies: vec![],
    }
}
fn request(id: &str, capture: &str, intent: ReviewIntent) -> DecisionRequest {
    DecisionRequest {
        decision_id: id.into(),
        capture_id: capture.into(),
        expected_context: context(1),
        expected_proposal_sha256: proposal_hash(&proposal()).unwrap(),
        intent,
    }
}
#[test]
fn explicit_selection_decision_and_duplicate_token_survive_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let token;
    let command = request("decision", "capture", ReviewIntent::AcceptForPreparation);
    {
        let mut inbox = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
        inbox.admit("capture", context(1), proposal()).unwrap();
        assert!(inbox.view().snapshot().unwrap().selected_capture.is_none());
        assert!(matches!(
            inbox.decide(command.clone()),
            Err(InboxError::SelectionRequired)
        ));
        inbox.select(Some("capture")).unwrap();
        token = inbox.decide(command.clone()).unwrap();
        assert_eq!(inbox.decide(command.clone()).unwrap(), token);
        assert_eq!(
            inbox.state("capture").unwrap(),
            ReviewState::AcceptedForPreparation
        );
    }
    let mut inbox = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
    assert_eq!(
        inbox.view().snapshot().unwrap().selected_capture.as_deref(),
        Some("capture")
    );
    assert_eq!(inbox.decide(command.clone()).unwrap(), token);
    inbox.validate_handoff(&token, &context(1)).unwrap();
    let mut changed = command;
    changed.intent = ReviewIntent::RejectCapture;
    assert!(matches!(inbox.decide(changed), Err(InboxError::Conflict)));
    let mut forged = token;
    forged.token_sha256 = "0".repeat(64);
    assert!(matches!(
        inbox.validate_handoff(&forged, &context(1)),
        Err(InboxError::Conflict)
    ));
}
#[test]
fn cancellation_revokes_accepted_handoff_durably() {
    let dir = tempfile::tempdir().unwrap();
    let token;
    {
        let mut inbox = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
        inbox.admit("capture", context(1), proposal()).unwrap();
        inbox.select(Some("capture")).unwrap();
        token = inbox
            .decide(request("d", "capture", ReviewIntent::AcceptForPreparation))
            .unwrap();
        inbox.cancel("capture").unwrap();
        assert!(matches!(
            inbox.validate_handoff(&token, &context(1)),
            Err(InboxError::Cancelled)
        ));
    }
    let inbox = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
    assert_eq!(inbox.state("capture").unwrap(), ReviewState::Cancelled);
    assert!(matches!(
        inbox.validate_handoff(&token, &context(1)),
        Err(InboxError::Cancelled)
    ));
}
#[test]
fn stale_context_cannot_accept_but_explicit_rejection_is_allowed() {
    let dir = tempfile::tempdir().unwrap();
    let mut inbox = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
    inbox.admit("capture", context(1), proposal()).unwrap();
    inbox.select(Some("capture")).unwrap();
    inbox.update_context("capture", context(2)).unwrap();
    let mut accept = request("accept", "capture", ReviewIntent::AcceptForPreparation);
    accept.expected_context = context(2);
    assert!(matches!(
        inbox.decide(accept),
        Err(InboxError::StaleContext)
    ));
    let mut reject = request("reject", "capture", ReviewIntent::RejectCapture);
    reject.expected_context = context(2);
    let token = inbox.decide(reject).unwrap();
    inbox.validate_handoff(&token, &context(2)).unwrap();
    assert_eq!(token.decision.intent, ReviewIntent::RejectCapture);
}
#[test]
fn caller_current_context_check_revokes_token_even_before_inbox_update() {
    let dir = tempfile::tempdir().unwrap();
    let mut inbox = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
    inbox.admit("capture", context(1), proposal()).unwrap();
    inbox.select(Some("capture")).unwrap();
    let token = inbox
        .decide(request("d", "capture", ReviewIntent::AcceptForPreparation))
        .unwrap();
    assert!(matches!(
        inbox.validate_handoff(&token, &context(2)),
        Err(InboxError::StaleContext)
    ));
    inbox.update_context("capture", context(2)).unwrap();
    assert!(matches!(
        inbox.validate_handoff(&token, &context(2)),
        Err(InboxError::StaleContext)
    ));
}
#[test]
fn bounded_records_retire_without_forgetting_decision_identity() {
    let dir = tempfile::tempdir().unwrap();
    let limits = InboxLimits {
        entries: 1,
        history: 4,
        ..InboxLimits::default()
    };
    let mut inbox = ReviewInbox::open(dir.path(), limits).unwrap();
    inbox.admit("one", context(1), proposal()).unwrap();
    assert!(matches!(
        inbox.admit("two", context(1), proposal()),
        Err(InboxError::Capacity)
    ));
    inbox.select(Some("one")).unwrap();
    let token = inbox
        .decide(request("d", "one", ReviewIntent::RejectCapture))
        .unwrap();
    inbox.retire("one").unwrap();
    assert!(matches!(
        inbox.validate_handoff(&token, &context(1)),
        Err(InboxError::Retired)
    ));
    assert!(matches!(
        inbox.admit("one", context(1), proposal()),
        Err(InboxError::Retired)
    ));
    inbox.admit("two", context(1), proposal()).unwrap();
    assert!(inbox.view().snapshot().unwrap().selected_capture.is_none());
}
#[test]
fn serialization_limit_preserves_old_checkpoint_and_corrupt_reopen_fails() {
    let dir = tempfile::tempdir().unwrap();
    let limits = InboxLimits {
        bytes: 256,
        ..InboxLimits::default()
    };
    let mut inbox = ReviewInbox::open(dir.path(), limits).unwrap();
    inbox.select(None).unwrap();
    let old = std::fs::read(dir.path().join("inbox.json")).unwrap();
    assert!(matches!(
        inbox.admit("capture", context(1), proposal()),
        Err(InboxError::Capacity)
    ));
    assert_eq!(std::fs::read(dir.path().join("inbox.json")).unwrap(), old);
    assert!(inbox.view().snapshot().unwrap().entries.is_empty());
    drop(inbox);
    std::fs::write(dir.path().join("inbox.json"), b"{broken").unwrap();
    assert!(matches!(
        ReviewInbox::open(dir.path(), limits),
        Err(InboxError::Invalid)
    ));
}
#[test]
fn storage_failure_poisoning_and_exclusive_owner_are_enforced() {
    let dir = tempfile::tempdir().unwrap();
    let mut inbox = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
    assert!(matches!(
        ReviewInbox::open(dir.path(), InboxLimits::default()),
        Err(InboxError::Busy)
    ));
    std::fs::create_dir(dir.path().join("inbox.json")).unwrap();
    assert!(matches!(
        inbox.admit("capture", context(1), proposal()),
        Err(InboxError::Io(_))
    ));
    assert!(matches!(
        inbox.view().snapshot(),
        Err(InboxError::RecoveryRequired)
    ));
    std::fs::remove_dir(dir.path().join("inbox.json")).unwrap();
    assert!(matches!(
        inbox.admit("capture", context(1), proposal()),
        Err(InboxError::RecoveryRequired)
    ));
    drop(inbox);
    let mut restored = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
    restored.admit("capture", context(1), proposal()).unwrap();
}
#[test]
fn corrupted_decision_project_cannot_restore_as_an_accepted_intent() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut inbox = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
        inbox.admit("capture", context(1), proposal()).unwrap();
        inbox.select(Some("capture")).unwrap();
        inbox
            .decide(request("d", "capture", ReviewIntent::AcceptForPreparation))
            .unwrap();
    }
    let path = dir.path().join("inbox.json");
    let mut state: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    state["decisions"]["d"]["expected_context"]["project_id"] = "other-project".into();
    std::fs::write(path, serde_json::to_vec(&state).unwrap()).unwrap();
    assert!(matches!(
        ReviewInbox::open(dir.path(), InboxLimits::default()),
        Err(InboxError::Invalid)
    ));
}
