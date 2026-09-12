#![cfg(feature = "bridge-integration")]
use base64::{engine::general_purpose::STANDARD, Engine};
use flashtex_bridge::{store::Store, Bridge, CaptureImage, CaptureSubmit, Document, Proposal};
use flashtex_conversion_jobs::bridge_adapter::*;
use std::{
    io::Cursor,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
fn document(b: &mut Bridge, project: &str) {
    b.open_document(Document {
        project_id: project.into(),
        path: "main.tex".into(),
        revision: 1,
        text: "before after".into(),
    })
    .unwrap();
    b.pin(&format!("{project}-anchor"), project, "main.tex", 1, 7, 7)
        .unwrap();
}
fn capture(b: &mut Bridge, project: &str, id: &str) {
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(1, 1)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    b.receive(CaptureSubmit {
        capture_id: id.into(),
        destination_id: format!("{project}-anchor"),
        base_revision: 1,
        image: CaptureImage {
            mime_type: "image/png".into(),
            data_base64: STANDARD.encode(png.into_inner()),
        },
        instructions: "offline fixture".into(),
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
fn wait(a: &BridgeAdapter, id: &str) {
    let end = Instant::now() + Duration::from_secs(10);
    while matches!(
        a.status_handle().status(id).unwrap(),
        AdapterState::Queued | AdapterState::Running
    ) {
        assert!(Instant::now() < end);
        thread::yield_now();
    }
}
#[test]
fn conversion_never_writes_source_then_explicit_ledger_receipt_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = Bridge::new(Store::open(dir.path().join("captures")).unwrap());
    document(&mut b, "p");
    capture(&mut b, "p", "capture");
    let ledger_path = dir.path().join("ledger");
    let mut ledger = flashtex_edit_ledger::Store::open(&ledger_path).unwrap();
    ledger
        .initialize(
            flashtex_edit_ledger::Document::new(
                "p".into(),
                "main.tex".into(),
                1,
                "before after".into(),
            )
            .unwrap(),
        )
        .unwrap();
    let before = std::fs::read(ledger_path.join("document.json")).unwrap();
    let mut a = BridgeAdapter::new(
        dir.path().join("intents"),
        1,
        2,
        4,
        Arc::new(|_, _, _| Ok(proposal())),
    )
    .unwrap();
    a.start(&b, "capture", vec![]).unwrap();
    wait(&a, "capture");
    assert!(matches!(
        a.reconcile(&mut b, "capture").unwrap(),
        AdapterState::Proposal(_)
    ));
    assert_eq!(
        std::fs::read(ledger_path.join("document.json")).unwrap(),
        before
    );
    assert_eq!(b.document("p", "main.tex").unwrap().text, "before after");
    // This explicit fixture review/application is owned by the caller, not jobs.
    // Compiler and native user-review UI acceptance are separate integration gates.
    let edit = b.prepare_insert("capture", 1, true).unwrap();
    assert_eq!(
        std::fs::read(ledger_path.join("document.json")).unwrap(),
        before
    );
    let ledger_edit: flashtex_edit_ledger::PreparedEdit =
        serde_json::from_value(serde_json::to_value(&edit).unwrap()).unwrap();
    let receipt = ledger.apply(ledger_edit.clone()).unwrap();
    assert_eq!(ledger.document().unwrap().unwrap().text, "before $x$after");
    assert_eq!(b.document("p", "main.tex").unwrap().text, "before after");
    let applied = b
        .confirm_insert(&receipt.capture_id, &receipt.edit_id, receipt.new_revision)
        .unwrap();
    ledger.confirm(&receipt).unwrap();
    assert_eq!(applied.new_revision, 2);
    assert_eq!(b.document("p", "main.tex").unwrap().text, "before $x$after");
    assert!(matches!(
        a.reconcile(&mut b, "capture").unwrap(),
        AdapterState::Applied
    ));
    drop(a);
    drop(b);
    drop(ledger);
    let mut ledger = flashtex_edit_ledger::Store::open(&ledger_path).unwrap();
    assert_eq!(ledger.apply(ledger_edit).unwrap(), receipt);
    assert_eq!(ledger.document().unwrap().unwrap().text, "before $x$after");
    assert!(ledger.recovery().unwrap().is_empty());
    let mut b = Bridge::new(Store::open(dir.path().join("captures")).unwrap());
    assert_eq!(
        b.confirm_insert(&receipt.capture_id, &receipt.edit_id, receipt.new_revision)
            .unwrap(),
        applied
    );
}
#[test]
fn blocked_project_does_not_starve_other_project_and_cancellation_is_isolated() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = Bridge::new(Store::open(dir.path().join("captures")).unwrap());
    for project in ["p1", "p2"] {
        document(&mut b, project);
        capture(&mut b, project, &format!("{project}-capture"));
    }
    let (entered, started) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    let (done, finished) = mpsc::channel();
    let gate = Mutex::new(gate);
    let mut a = BridgeAdapter::new(
        dir.path().join("intents"),
        2,
        2,
        4,
        Arc::new(move |_, context, token| {
            if context.project_id == "p1" {
                entered.send(()).unwrap();
                gate.lock().unwrap().recv().unwrap();
                assert!(token.is_cancelled());
                done.send(()).unwrap();
            }
            Ok(proposal())
        }),
    )
    .unwrap();
    a.start(&b, "p1-capture", vec![]).unwrap();
    started.recv_timeout(Duration::from_secs(10)).unwrap();
    a.start(&b, "p2-capture", vec![]).unwrap();
    wait(&a, "p2-capture");
    assert!(matches!(
        a.status_handle().status("p1-capture").unwrap(),
        AdapterState::Running
    ));
    assert!(matches!(
        a.reconcile(&mut b, "p2-capture").unwrap(),
        AdapterState::Proposal(_)
    ));
    a.cancel("p1-capture").unwrap();
    release.send(()).unwrap();
    finished.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(matches!(
        a.reconcile(&mut b, "p1-capture").unwrap(),
        AdapterState::Cancelled
    ));
    assert!(b.store.require("p1-capture").unwrap().proposal.is_none());
    assert!(b.store.require("p2-capture").unwrap().proposal.is_some());
    for project in ["p1", "p2"] {
        assert_eq!(
            b.document(project, "main.tex").unwrap().text,
            "before after"
        );
    }
}
