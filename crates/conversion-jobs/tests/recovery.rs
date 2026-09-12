use flashtex_conversion_jobs::{snapshot::*, *};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
fn context() -> ContextFingerprint {
    ContextFingerprint {
        revision: 7,
        sha256: "a".repeat(64),
    }
}
fn terminal(s: &Scheduler<String>, id: &str) -> State<String> {
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let state = s.state(id).unwrap();
        if !matches!(state, State::Queued | State::Running) {
            return state;
        }
        assert!(Instant::now() < until);
        thread::yield_now();
    }
}
fn decode(bytes: &[u8]) -> Result<String, String> {
    String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())
}
#[test]
fn queued_and_running_restore_without_calls_until_explicit_authorization() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("jobs.json");
    let s = Scheduler::<String>::new(1, 3, 3).unwrap();
    let (entered, started) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    let (done, finished) = mpsc::channel();
    s.submit("running", context(), move |_| {
        entered.send(()).unwrap();
        gate.recv().unwrap();
        done.send(()).unwrap();
        Ok("late".into())
    })
    .unwrap();
    started.recv_timeout(Duration::from_secs(10)).unwrap();
    s.submit("queued", context(), |_| {
        panic!("old queued closure must never run")
    })
    .unwrap();
    s.save_snapshot(&path, Limits::default(), |v| Ok(v.as_bytes().to_vec()))
        .unwrap();
    drop(s);
    release.send(()).unwrap();
    finished.recv_timeout(Duration::from_secs(10)).unwrap();
    let restored =
        Scheduler::<String>::restore_snapshot(&path, 1, 3, 3, Limits::default(), decode).unwrap();
    assert!(matches!(
        restored.state("running").unwrap(),
        State::RecoveryRequired(RecoveryReason::ProviderMayHaveCompleted)
    ));
    assert!(matches!(
        restored.state("queued").unwrap(),
        State::RecoveryRequired(RecoveryReason::QueuedAwaitingAuthorization)
    ));
    assert_eq!(
        restored.authorize_resume(
            "running",
            ResumeAuthorization::QueuedOnly,
            context(),
            |_| Ok("bad".into())
        ),
        Err(Error::RecoveryAuthorizationRequired)
    );
    restored
        .authorize_resume("queued", ResumeAuthorization::QueuedOnly, context(), |_| {
            Ok("approved queued".into())
        })
        .unwrap();
    assert!(matches!(terminal(&restored,"queued"),State::Completed(v) if *v=="approved queued"));
    restored
        .authorize_resume(
            "running",
            ResumeAuthorization::ReconciledPossibleProviderCompletion,
            context(),
            |_| Ok("explicit reconciled retry".into()),
        )
        .unwrap();
    assert!(matches!(
        terminal(&restored, "running"),
        State::Completed(_)
    ));
}
#[test]
fn result_codec_is_bounded_and_failed_checkpoint_preserves_old_atomic_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("jobs.json");
    let s = Scheduler::<String>::new(1, 2, 2).unwrap();
    s.submit("done", context(), |_| Ok("result".into()))
        .unwrap();
    terminal(&s, "done");
    s.save_snapshot(&path, Limits::default(), |value| {
        assert!(matches!(s.state("done").unwrap(), State::Completed(_)));
        Ok(value.as_bytes().to_vec())
    })
    .unwrap();
    let before = std::fs::read(&path).unwrap();
    assert!(s
        .save_snapshot(
            &path,
            Limits {
                snapshot_bytes: 1024,
                result_bytes: 2
            },
            |v| Ok(v.as_bytes().to_vec())
        )
        .is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let restored =
        Scheduler::<String>::restore_snapshot(&path, 1, 2, 2, Limits::default(), decode).unwrap();
    assert!(matches!(restored.state("done").unwrap(),State::Completed(v) if *v=="result"));
    assert!(
        Scheduler::<String>::restore_snapshot(&path, 1, 2, 2, Limits::default(), |_| Err(
            "codec denied".into()
        ))
        .is_err()
    );
}
#[test]
fn ambiguous_failures_restore_recovery_required_not_automatic_retry() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("jobs.json");
    let s = Scheduler::<String>::new(1, 2, 2).unwrap();
    s.submit("ambiguous", context(), |_| {
        Err(Failure::new(FailureKind::AmbiguousProvider, "timeout"))
    })
    .unwrap();
    terminal(&s, "ambiguous");
    s.save_snapshot(&path, Limits::default(), |v| Ok(v.as_bytes().to_vec()))
        .unwrap();
    drop(s);
    let restored =
        Scheduler::<String>::restore_snapshot(&path, 1, 2, 2, Limits::default(), decode).unwrap();
    assert!(matches!(
        restored.state("ambiguous").unwrap(),
        State::RecoveryRequired(RecoveryReason::ProviderMayHaveCompleted)
    ));
    assert_eq!(
        restored.submit("ambiguous", context(), |_| Ok("retry".into())),
        Err(Error::DuplicateIdentity)
    );
}
#[test]
fn malformed_oversized_and_duplicate_snapshots_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("jobs.json");
    let s = Scheduler::<String>::new(1, 2, 2).unwrap();
    s.submit("done", context(), |_| Ok("result".into()))
        .unwrap();
    terminal(&s, "done");
    s.save_snapshot(&path, Limits::default(), |v| Ok(v.as_bytes().to_vec()))
        .unwrap();
    let mut duplicate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let job = duplicate["jobs"][0].clone();
    duplicate["jobs"].as_array_mut().unwrap().push(job);
    std::fs::write(&path, serde_json::to_vec(&duplicate).unwrap()).unwrap();
    assert!(
        Scheduler::<String>::restore_snapshot(&path, 1, 2, 2, Limits::default(), decode).is_err()
    );
    std::fs::write(&path, b"{broken").unwrap();
    assert!(
        Scheduler::<String>::restore_snapshot(&path, 1, 2, 2, Limits::default(), decode).is_err()
    );
    std::fs::write(&path, vec![0; 1025]).unwrap();
    assert!(Scheduler::<String>::restore_snapshot(
        &path,
        1,
        2,
        2,
        Limits {
            snapshot_bytes: 1024,
            result_bytes: 64
        },
        decode
    )
    .is_err());
}
