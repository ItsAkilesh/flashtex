#![cfg(feature = "bridge-integration")]
use base64::{engine::general_purpose::STANDARD, Engine};
use flashtex_bridge::{store::Store, Bridge, CaptureImage, CaptureSubmit, Document, Proposal};
use flashtex_conversion_jobs::{bridge_adapter::*, *};
use std::{
    io::Cursor,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
fn bridge(path: &Path) -> Bridge {
    let mut b = Bridge::new(Store::open(path.join("captures")).unwrap());
    b.open_document(Document {
        project_id: "p".into(),
        path: "main.tex".into(),
        revision: 1,
        text: "before after".into(),
    })
    .unwrap();
    b.pin("anchor", "p", "main.tex", 1, 7, 7).unwrap();
    b
}
fn receive(b: &mut Bridge, id: &str) {
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(1, 1)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    b.receive(CaptureSubmit {
        capture_id: id.into(),
        destination_id: "anchor".into(),
        base_revision: 1,
        image: CaptureImage {
            mime_type: "image/png".into(),
            data_base64: STANDARD.encode(png.into_inner()),
        },
        instructions: "fixture".into(),
    })
    .unwrap();
}
fn proposal() -> Proposal {
    Proposal {
        latex: "$x$".into(),
        ambiguities: vec![],
        required_dependencies: vec![],
    }
}
fn wait(handle: &StatusHandle, id: &str) {
    let end = Instant::now() + Duration::from_secs(10);
    while matches!(
        handle.status(id).unwrap(),
        AdapterState::Queued | AdapterState::Running
    ) {
        assert!(Instant::now() < end);
        thread::yield_now();
    }
}
#[test]
fn nonblocking_status_and_durable_proposal_survive_restart_without_second_call() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(dir.path());
    receive(&mut b, "capture");
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let (entered, started) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    let gate = Mutex::new(gate);
    let mut a = BridgeAdapter::new(
        dir.path().join("intents"),
        1,
        2,
        4,
        Arc::new(move |_, _, _| {
            count.fetch_add(1, Ordering::SeqCst);
            entered.send(()).unwrap();
            gate.lock().unwrap().recv().unwrap();
            Ok(proposal())
        }),
    )
    .unwrap();
    a.start(&b, "capture", vec![]).unwrap();
    started.recv_timeout(Duration::from_secs(10)).unwrap();
    let handle = a.status_handle();
    assert!(matches!(
        handle.status("capture").unwrap(),
        AdapterState::Running
    ));
    a.start(&b, "capture", vec![]).unwrap();
    release.send(()).unwrap();
    wait(&handle, "capture");
    assert!(matches!(
        handle.status("capture").unwrap(),
        AdapterState::AwaitingJournal
    ));
    assert!(b.store.require("capture").unwrap().proposal.is_none());
    assert!(matches!(
        a.reconcile(&mut b, "capture").unwrap(),
        AdapterState::Proposal(_)
    ));
    drop(a);
    drop(b);
    let b = bridge(dir.path());
    let mut a = BridgeAdapter::new(
        dir.path().join("intents"),
        1,
        2,
        4,
        Arc::new(|_, _, _| panic!("journaled proposal must avoid another provider call")),
    )
    .unwrap();
    assert!(matches!(
        a.start(&b, "capture", vec![]).unwrap(),
        AdapterState::Proposal(_)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
#[test]
fn ambiguous_attempt_recovers_without_retry_and_intent_is_exclusive() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(dir.path());
    receive(&mut b, "capture");
    let mut a = BridgeAdapter::new(
        dir.path().join("intents"),
        1,
        2,
        4,
        Arc::new(|_, _, _| Err(Failure::new(FailureKind::AmbiguousProvider, "timeout"))),
    )
    .unwrap();
    a.start(&b, "capture", vec![]).unwrap();
    wait(&a.status_handle(), "capture");
    assert!(matches!(
        a.status_handle().status("capture").unwrap(),
        AdapterState::Failed(_)
    ));
    drop(a);
    let mut a = BridgeAdapter::new(
        dir.path().join("intents"),
        1,
        2,
        4,
        Arc::new(|_, _, _| panic!("ambiguous attempt cannot retry")),
    )
    .unwrap();
    assert!(matches!(
        a.start(&b, "capture", vec![]).unwrap(),
        AdapterState::RecoveryRequired
    ));
    a.retire("capture").unwrap();
    assert!(matches!(
        a.start(&b, "capture", vec![]).unwrap(),
        AdapterState::RecoveryRequired
    ));
}
#[test]
fn root_context_change_rejects_late_result_and_cancel_before_journal_blocks_promotion() {
    for cancel in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let mut b = bridge(dir.path());
        b.open_document(Document {
            project_id: "p".into(),
            path: "root.tex".into(),
            revision: 1,
            text: "\\def\\x{old}\\input{main}".into(),
        })
        .unwrap();
        receive(&mut b, "capture");
        let mut a = BridgeAdapter::new(
            dir.path().join("intents"),
            1,
            2,
            4,
            Arc::new(|_, _, _| Ok(proposal())),
        )
        .unwrap();
        a.start(&b, "capture", vec![]).unwrap();
        wait(&a.status_handle(), "capture");
        if cancel {
            a.cancel("capture").unwrap();
            assert!(matches!(
                a.reconcile(&mut b, "capture").unwrap(),
                AdapterState::Cancelled
            ));
        } else {
            b.open_document(Document {
                project_id: "p".into(),
                path: "root.tex".into(),
                revision: 2,
                text: "\\def\\x{new}\\input{main}".into(),
            })
            .unwrap();
            assert!(matches!(
                a.reconcile(&mut b, "capture").unwrap(),
                AdapterState::StaleContext
            ));
        }
        assert!(b.store.require("capture").unwrap().proposal.is_none());
    }
}
#[test]
fn bounded_queue_is_fifo_and_backpressure_does_not_burn_capture_identity() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(dir.path());
    for id in ["one", "two", "three"] {
        receive(&mut b, id);
    }
    let (order, started) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    let gate = Mutex::new(gate);
    let mut a = BridgeAdapter::new(
        dir.path().join("intents"),
        1,
        1,
        4,
        Arc::new(move |capture, _, _| {
            order.send(capture.capture_id.clone()).unwrap();
            if capture.capture_id == "one" {
                gate.lock().unwrap().recv().unwrap();
            }
            Ok(proposal())
        }),
    )
    .unwrap();
    a.start(&b, "one", vec![]).unwrap();
    assert_eq!(
        started.recv_timeout(Duration::from_secs(10)).unwrap(),
        "one"
    );
    a.start(&b, "two", vec![]).unwrap();
    assert!(matches!(
        a.start(&b, "three", vec![]),
        Err(AdapterError::Scheduler(Error::QueueFull))
    ));
    assert!(!dir.path().join("intents/three.intent.json").exists());
    release.send(()).unwrap();
    assert_eq!(
        started.recv_timeout(Duration::from_secs(10)).unwrap(),
        "two"
    );
    wait(&a.status_handle(), "two");
    a.start(&b, "three", vec![]).unwrap();
    assert_eq!(
        started.recv_timeout(Duration::from_secs(10)).unwrap(),
        "three"
    );
    wait(&a.status_handle(), "three");
    for id in ["one", "two", "three"] {
        assert!(matches!(
            a.reconcile(&mut b, id).unwrap(),
            AdapterState::Proposal(_)
        ));
    }
    assert_eq!(a.usage().calls_started, 3);
}
#[test]
fn adapter_restart_during_live_old_call_never_dispatches_duplicate() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(dir.path());
    receive(&mut b, "capture");
    let (entered, started) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    let (done, finished) = mpsc::channel();
    let gate = Mutex::new(gate);
    let mut a = BridgeAdapter::new(
        dir.path().join("intents"),
        1,
        2,
        4,
        Arc::new(move |_, _, token| {
            entered.send(()).unwrap();
            gate.lock().unwrap().recv().unwrap();
            assert!(token.is_cancelled());
            done.send(()).unwrap();
            Ok(proposal())
        }),
    )
    .unwrap();
    a.start(&b, "capture", vec![]).unwrap();
    started.recv_timeout(Duration::from_secs(10)).unwrap();
    drop(a);
    let mut restored = BridgeAdapter::new(
        dir.path().join("intents"),
        1,
        2,
        4,
        Arc::new(|_, _, _| panic!("in-flight intent must never permit another attempt")),
    )
    .unwrap();
    assert!(matches!(
        restored.start(&b, "capture", vec![]).unwrap(),
        AdapterState::RecoveryRequired
    ));
    release.send(()).unwrap();
    finished.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(b.store.require("capture").unwrap().proposal.is_none());
}
