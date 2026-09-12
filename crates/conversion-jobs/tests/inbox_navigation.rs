#![cfg(feature = "bridge-integration")]
use flashtex_bridge::Proposal;
use flashtex_conversion_jobs::bridge_adapter::{
    native::ContextIdentity,
    review::{navigation::*, *},
};
fn context(project: &str) -> ContextIdentity {
    ContextIdentity {
        project_id: project.into(),
        path: "main.tex".into(),
        revision: 1,
        sha256: "a".repeat(64),
    }
}
fn proposal() -> Proposal {
    Proposal {
        latex: "$x$".into(),
        ambiguities: vec![],
        required_dependencies: vec![],
    }
}
#[test]
fn per_project_arrival_navigation_is_explicit_bounded_and_survives_reopen() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut inbox = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
        inbox.admit("z-first", context("p1"), proposal()).unwrap();
        inbox.admit("other", context("p2"), proposal()).unwrap();
        inbox.admit("a-last", context("p1"), proposal()).unwrap();
        let filter = InboxFilter {
            project_id: Some("p1".into()),
            undecided_only: true,
        };
        assert_eq!(
            inbox.view().visible_ids(&filter).unwrap(),
            vec!["z-first", "a-last"]
        );
        assert_eq!(
            inbox.navigate(&filter, Direction::Next).unwrap().as_deref(),
            Some("z-first")
        );
        assert_eq!(
            inbox.navigate(&filter, Direction::Next).unwrap().as_deref(),
            Some("a-last")
        );
        assert!(inbox.navigate(&filter, Direction::Next).unwrap().is_none());
        assert_eq!(
            inbox.view().snapshot().unwrap().selected_capture.as_deref(),
            Some("a-last")
        );
        assert!(inbox.view().snapshot().unwrap().decisions.is_empty());
        inbox.cancel("z-first").unwrap();
        assert_eq!(inbox.view().visible_ids(&filter).unwrap(), vec!["a-last"]);
    }
    let mut inbox = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
    assert_eq!(
        inbox.view().snapshot().unwrap().selected_capture.as_deref(),
        Some("a-last")
    );
    let empty = InboxFilter {
        project_id: Some("missing".into()),
        undecided_only: false,
    };
    assert!(inbox.navigate(&empty, Direction::Next).unwrap().is_none());
    assert!(inbox.view().snapshot().unwrap().selected_capture.is_none());
}
#[test]
fn slow_and_disconnected_event_consumers_do_not_block_durable_decisions() {
    let dir = tempfile::tempdir().unwrap();
    let mut inbox = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
    let slow = inbox.subscribe(1).unwrap();
    let events = inbox.subscribe(16).unwrap();
    let disconnected = inbox.subscribe(1).unwrap();
    drop(disconnected);
    inbox.admit("capture", context("p"), proposal()).unwrap();
    inbox.select(Some("capture")).unwrap();
    inbox
        .decide(DecisionRequest {
            decision_id: "d".into(),
            capture_id: "capture".into(),
            expected_context: context("p"),
            expected_proposal_sha256: proposal_hash(&proposal()).unwrap(),
            intent: ReviewIntent::AcceptForPreparation,
        })
        .unwrap();
    assert_eq!(
        inbox.state("capture").unwrap(),
        ReviewState::AcceptedForPreparation
    );
    assert_eq!(slow.try_iter().count(), 1);
    assert_eq!(inbox.event_usage().dropped_deliveries, 2);
    assert_eq!(inbox.event_usage().subscribers, 2);
    let events = events.try_iter().collect::<Vec<_>>();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].kind, InboxEventKind::Added);
    assert_eq!(events[1].kind, InboxEventKind::SelectionChanged);
    assert_eq!(events[2].kind, InboxEventKind::DecisionRecorded);
    assert!(events.windows(2).all(|p| p[0].generation < p[1].generation));
    assert_eq!(
        events[2].generation,
        inbox.view().snapshot().unwrap().generation
    );
    assert!(events.iter().all(|e| e.project_id.as_deref() == Some("p")));
}
#[test]
fn failed_storage_notifies_uncertainty_and_view_requires_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let mut inbox = ReviewInbox::open(dir.path(), InboxLimits::default()).unwrap();
    let events = inbox.subscribe(1).unwrap();
    std::fs::create_dir(dir.path().join("inbox.json")).unwrap();
    assert!(inbox.admit("capture", context("p"), proposal()).is_err());
    assert_eq!(
        events.try_recv().unwrap().kind,
        InboxEventKind::PersistenceUncertain
    );
    assert!(matches!(
        inbox.view().snapshot(),
        Err(InboxError::RecoveryRequired)
    ));
}
