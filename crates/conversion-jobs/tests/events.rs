use flashtex_conversion_jobs::{events::*, *};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
fn context() -> ContextFingerprint {
    ContextFingerprint {
        revision: 1,
        sha256: "a".repeat(64),
    }
}
fn terminal(s: &Scheduler<u32>) {
    let end = Instant::now() + Duration::from_secs(10);
    while matches!(s.state("job").unwrap(), State::Queued | State::Running) {
        assert!(Instant::now() < end);
        thread::yield_now();
    }
}
#[test]
fn ordered_transitions_progress_and_actual_call_evidence() {
    let s = Scheduler::<u32>::new(1, 2, 2).unwrap();
    let events = s.subscribe(16).unwrap();
    s.submit("job", context(), |token| {
        token.progress(50, "transcribing").unwrap();
        Ok(5)
    })
    .unwrap();
    terminal(&s);
    let events = events.try_iter().collect::<Vec<_>>();
    assert!(matches!(events[0].kind, EventKind::Queued));
    assert!(matches!(events[1].kind, EventKind::Running));
    assert!(
        matches!(&events[2].kind,EventKind::Progress{percent:50,message} if message=="transcribing")
    );
    assert!(matches!(events[3].kind, EventKind::Completed));
    assert!(events
        .windows(2)
        .all(|pair| pair[0].sequence < pair[1].sequence));
    let usage = s.usage();
    assert_eq!(usage.calls_started, 1);
    assert_eq!(usage.executing, 0);
    assert_eq!(usage.retained, 1);
    assert!(!usage.provider_billing_known);
}
#[test]
fn full_and_disconnected_subscribers_cannot_block_conversion() {
    let s = Scheduler::<u32>::new(1, 2, 2).unwrap();
    let slow = s.subscribe(1).unwrap();
    let disconnected = s.subscribe(1).unwrap();
    drop(disconnected);
    s.submit("job", context(), |token| {
        for i in 0..100 {
            token.progress(i, "progress").unwrap();
        }
        Ok(9)
    })
    .unwrap();
    terminal(&s);
    assert!(matches!(s.state("job").unwrap(), State::Completed(_)));
    assert!(s.usage().events_dropped >= 100);
    assert_eq!(slow.try_iter().count(), 1);
}
#[test]
fn cancelled_job_cannot_emit_unbounded_or_late_progress() {
    let s = Scheduler::<u32>::new(1, 2, 2).unwrap();
    let (entered, started) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    let (done, finished) = mpsc::channel();
    s.submit("job", context(), move |token| {
        assert_eq!(token.progress(101, "bad"), Err(Error::InvalidProgress));
        assert_eq!(
            token.progress(1, "x".repeat(1025)),
            Err(Error::InvalidProgress)
        );
        entered.send(()).unwrap();
        gate.recv().unwrap();
        assert_eq!(token.progress(70, "late"), Err(Error::Stopped));
        done.send(()).unwrap();
        Ok(1)
    })
    .unwrap();
    started.recv_timeout(Duration::from_secs(10)).unwrap();
    s.cancel("job").unwrap();
    release.send(()).unwrap();
    finished.recv_timeout(Duration::from_secs(10)).unwrap();
}
