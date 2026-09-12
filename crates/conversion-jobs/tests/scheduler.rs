use flashtex_conversion_jobs::*;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};
fn context(revision: u64) -> ContextFingerprint {
    ContextFingerprint {
        revision,
        sha256: "a".repeat(64),
    }
}
fn terminal<T: Send + Sync + 'static>(scheduler: &Scheduler<T>, id: &str) -> State<T> {
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let state = scheduler.state(id).unwrap();
        if !matches!(state, State::Queued | State::Running) {
            return state;
        }
        assert!(Instant::now() < until, "job did not finish");
        thread::yield_now();
    }
}
#[test]
fn duplicate_identity_is_reserved_during_and_after_ambiguous_failure() {
    let s = Scheduler::<String>::new(1, 4, 4).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let (entered, started) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    s.submit("one", context(1), move |_| {
        count.fetch_add(1, Ordering::SeqCst);
        entered.send(()).unwrap();
        gate.recv().unwrap();
        Err(Failure::new(
            FailureKind::AmbiguousProvider,
            "timeout after sending",
        ))
    })
    .unwrap();
    started.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(
        s.submit("one", context(1), |_| Ok("duplicate".into())),
        Err(Error::DuplicateIdentity)
    );
    release.send(()).unwrap();
    assert!(matches!(
        terminal(&s, "one"),
        State::Failed(Failure {
            kind: FailureKind::AmbiguousProvider,
            ..
        })
    ));
    assert_eq!(
        s.submit("one", context(1), |_| Ok("duplicate".into())),
        Err(Error::DuplicateIdentity)
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
#[test]
fn bounded_queue_and_cancelled_pending_task_never_call_converter() {
    let s = Scheduler::<u32>::new(1, 1, 3).unwrap();
    let (entered, started) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    s.submit("running", context(1), move |_| {
        entered.send(()).unwrap();
        gate.recv().unwrap();
        Ok(1)
    })
    .unwrap();
    started.recv_timeout(Duration::from_secs(10)).unwrap();
    s.submit("queued", context(1), |_| {
        panic!("cancelled queue must not run")
    })
    .unwrap();
    assert!(matches!(s.state("queued").unwrap(), State::Queued));
    assert_eq!(
        s.submit("extra", context(1), |_| Ok(3)),
        Err(Error::QueueFull)
    );
    s.cancel("queued").unwrap();
    assert!(matches!(s.forget("queued").unwrap(), State::Cancelled));
    s.submit("extra", context(1), |_| Ok(3)).unwrap();
    release.send(()).unwrap();
    assert!(matches!(terminal(&s, "running"), State::Completed(_)));
    assert!(matches!(terminal(&s, "extra"), State::Completed(_)));
}
#[test]
fn running_cancellation_preserves_physical_slot_and_discards_late_result() {
    let s = Scheduler::<u32>::new(1, 2, 3).unwrap();
    let (entered, started) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    let (observed, cancelled) = mpsc::channel();
    s.submit("running", context(1), move |token| {
        entered.send(()).unwrap();
        gate.recv().unwrap();
        observed.send(token.is_cancelled()).unwrap();
        Ok(1)
    })
    .unwrap();
    started.recv_timeout(Duration::from_secs(10)).unwrap();
    s.cancel("running").unwrap();
    assert_eq!(s.forget("running").unwrap_err(), Error::StillExecuting);
    s.submit("next", context(1), |_| Ok(2)).unwrap();
    assert!(matches!(s.state("next").unwrap(), State::Queued));
    release.send(()).unwrap();
    assert!(cancelled.recv_timeout(Duration::from_secs(10)).unwrap());
    assert!(matches!(terminal(&s, "next"), State::Completed(_)));
    assert!(matches!(s.state("running").unwrap(), State::Cancelled));
}
#[test]
fn changed_revision_or_hash_rejects_completion_and_queued_stale_skips_provider() {
    for hash_only in [false, true] {
        let s = Scheduler::<u32>::new(1, 2, 3).unwrap();
        let (entered, started) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        s.submit("running", context(1), move |_| {
            entered.send(()).unwrap();
            gate.recv().unwrap();
            Ok(1)
        })
        .unwrap();
        started.recv_timeout(Duration::from_secs(10)).unwrap();
        s.submit("queued", context(1), |_| {
            panic!("stale queued conversion must not run")
        })
        .unwrap();
        let changed = if hash_only {
            ContextFingerprint {
                revision: 1,
                sha256: "b".repeat(64),
            }
        } else {
            context(2)
        };
        s.update_context("running", changed.clone()).unwrap();
        s.update_context("queued", changed).unwrap();
        release.send(()).unwrap();
        for id in ["running", "queued"] {
            assert!(matches!(
                terminal(&s, id),
                State::Failed(Failure {
                    kind: FailureKind::StaleContext,
                    ..
                })
            ));
        }
    }
}
#[test]
fn completed_result_is_revoked_when_context_changes_and_retention_is_bounded() {
    let s = Scheduler::<u32>::new(1, 2, 1).unwrap();
    s.submit("one", context(1), |_| Ok(7)).unwrap();
    assert!(matches!(terminal(&s,"one"),State::Completed(value) if *value==7));
    assert_eq!(
        s.submit("two", context(1), |_| Ok(2)),
        Err(Error::RetentionFull)
    );
    s.update_context("one", context(2)).unwrap();
    assert!(matches!(
        s.state("one").unwrap(),
        State::Failed(Failure {
            kind: FailureKind::StaleContext,
            ..
        })
    ));
    s.forget("one").unwrap();
    s.submit("two", context(1), |_| Ok(2)).unwrap();
    assert!(matches!(terminal(&s, "two"), State::Completed(_)));
}
#[test]
fn two_workers_run_independently_and_panics_do_not_kill_pool() {
    let s = Scheduler::<u32>::new(2, 4, 4).unwrap();
    let (entered, started) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    s.submit("blocked", context(1), move |_| {
        entered.send(()).unwrap();
        gate.recv().unwrap();
        Ok(1)
    })
    .unwrap();
    started.recv_timeout(Duration::from_secs(10)).unwrap();
    s.submit("panic", context(1), |_| panic!("injected converter panic"))
        .unwrap();
    assert!(matches!(
        terminal(&s, "panic"),
        State::Failed(Failure {
            kind: FailureKind::Panicked,
            ..
        })
    ));
    s.submit("next", context(1), |_| Ok(2)).unwrap();
    assert!(matches!(terminal(&s, "next"), State::Completed(_)));
    release.send(()).unwrap();
}
#[test]
fn shutdown_requests_cancellation_without_waiting_for_provider() {
    let s = Scheduler::<u32>::new(1, 2, 2).unwrap();
    let (entered, started) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    let (observed, cancelled) = mpsc::channel();
    s.submit("one", context(1), move |token| {
        entered.send(()).unwrap();
        gate.recv().unwrap();
        observed.send(token.is_cancelled()).unwrap();
        Ok(1)
    })
    .unwrap();
    started.recv_timeout(Duration::from_secs(10)).unwrap();
    drop(s);
    release.send(()).unwrap();
    assert!(cancelled.recv_timeout(Duration::from_secs(10)).unwrap());
}
#[test]
fn rejects_invalid_limits_identity_and_digest_without_starting_work() {
    assert!(matches!(
        Scheduler::<u32>::new(0, 1, 1),
        Err(Error::InvalidLimits)
    ));
    let s = Scheduler::<u32>::new(1, 1, 1).unwrap();
    assert_eq!(
        s.submit("../id", context(1), |_| Ok(1)),
        Err(Error::InvalidIdentity)
    );
    assert_eq!(
        s.submit(
            "id",
            ContextFingerprint {
                revision: 1,
                sha256: "bad".into()
            },
            |_| Ok(1)
        ),
        Err(Error::InvalidFingerprint)
    );
}
