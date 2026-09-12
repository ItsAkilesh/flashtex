#![cfg(feature = "bridge-integration")]
use base64::{engine::general_purpose::STANDARD, Engine};
use flashtex_bridge::{store::Store, Bridge, CaptureImage, CaptureSubmit, Document, Proposal};
use flashtex_conversion_jobs::bridge_adapter::{native::*, BridgeAdapter};
use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
fn setup() -> (tempfile::TempDir, Bridge, NativeService) {
    let dir = tempfile::tempdir().unwrap();
    let mut bridge = Bridge::new(Store::open(dir.path().join("captures")).unwrap());
    bridge
        .open_document(Document {
            project_id: "project".into(),
            path: "main.tex".into(),
            revision: 1,
            text: "hello".into(),
        })
        .unwrap();
    bridge
        .pin("anchor", "project", "main.tex", 1, 5, 5)
        .unwrap();
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(1, 1)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    bridge
        .receive(CaptureSubmit {
            capture_id: "capture".into(),
            destination_id: "anchor".into(),
            base_revision: 1,
            image: CaptureImage {
                mime_type: "image/png".into(),
                data_base64: STANDARD.encode(png.into_inner()),
            },
            instructions: "fixture".into(),
        })
        .unwrap();
    let adapter = BridgeAdapter::new(
        dir.path().join("intents"),
        1,
        2,
        4,
        Arc::new(|_, _, token| {
            token.progress(50, "offline").unwrap();
            Ok(Proposal {
                latex: "$x$".into(),
                ambiguities: vec![],
                required_dependencies: vec![],
            })
        }),
    )
    .unwrap();
    (dir, bridge, NativeService::new(adapter))
}
fn req(command: Command) -> Request {
    Request {
        protocol_version: 1,
        id: "request-1".into(),
        command,
    }
}
fn wait(handle: &NativeStatusHandle, identity: &ContextIdentity) {
    let until = Instant::now() + Duration::from_secs(10);
    while matches!(
        handle.snapshot("capture", identity).unwrap().conversion,
        ConversionState::Queued | ConversionState::Running
    ) {
        assert!(Instant::now() < until);
        thread::yield_now();
    }
}
#[test]
fn admission_intent_journal_states_have_exact_id_and_bounded_snapshots() {
    let (_dir, mut bridge, mut service) = setup();
    let context = NativeService::context_identity(&bridge, "capture", vec![]).unwrap();
    let events = service.subscribe(16).unwrap();
    let admitted = service
        .execute(
            &mut bridge,
            req(Command::Admit {
                capture_id: "capture".into(),
                context: context.clone(),
                supported_features: vec![],
            }),
        )
        .unwrap();
    assert_eq!(admitted.id, "request-1");
    assert!(matches!(
        admitted.status.conversion,
        ConversionState::Admitted
    ));
    assert!(matches!(admitted.status.intent, IntentState::NotRecorded));
    let started = service
        .execute(
            &mut bridge,
            req(Command::Start {
                capture_id: "capture".into(),
                context: context.clone(),
            }),
        )
        .unwrap();
    assert!(matches!(
        started.status.intent,
        IntentState::DurableMayHaveStarted
    ));
    let handle = service.status_handle();
    wait(&handle, &context);
    assert!(matches!(
        handle.snapshot("capture", &context).unwrap().conversion,
        ConversionState::AwaitingJournal
    ));
    assert!(bridge.store.require("capture").unwrap().proposal.is_none());
    let response = service
        .execute(
            &mut bridge,
            req(Command::Reconcile {
                capture_id: "capture".into(),
                context: context.clone(),
            }),
        )
        .unwrap();
    assert!(matches!(
        response.status.conversion,
        ConversionState::ProposalReady(_)
    ));
    assert!(matches!(
        response.status.intent,
        IntentState::ProposalJournaled
    ));
    assert_eq!(
        bridge.document("project", "main.tex").unwrap().text,
        "hello"
    );
    assert!(handle.encode("capture", &context, 16).is_err());
    let encoded = handle
        .encode("capture", &context, MAX_STATUS_BYTES)
        .unwrap();
    let decoded: StatusSnapshot = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded.context, context);
    let mut count = 0;
    while let Some(event) = events.try_next().unwrap() {
        assert_eq!(event.context, context);
        assert_eq!(event.capture_id, "capture");
        count += 1;
    }
    assert!(count >= 3);
}
#[test]
fn malformed_oversized_unknown_and_wrong_context_commands_fail_closed() {
    let (_dir, mut bridge, mut service) = setup();
    let context = NativeService::context_identity(&bridge, "capture", vec![]).unwrap();
    assert_eq!(
        service
            .execute_bytes(&mut bridge, &vec![b' '; MAX_COMMAND_BYTES + 1])
            .unwrap_err()
            .code,
        "command_too_large"
    );
    assert_eq!(
        service
            .execute_bytes(&mut bridge, b"{broken")
            .unwrap_err()
            .code,
        "invalid_command"
    );
    let mut unknown = serde_json::to_value(req(Command::Admit {
        capture_id: "capture".into(),
        context: context.clone(),
        supported_features: vec![],
    }))
    .unwrap();
    unknown["unexpected"] = true.into();
    assert_eq!(
        service
            .execute_bytes(&mut bridge, &serde_json::to_vec(&unknown).unwrap())
            .unwrap_err()
            .code,
        "invalid_command"
    );
    service
        .execute(
            &mut bridge,
            req(Command::Admit {
                capture_id: "capture".into(),
                context: context.clone(),
                supported_features: vec![],
            }),
        )
        .unwrap();
    let mut wrong = context.clone();
    wrong.project_id = "another".into();
    assert_eq!(
        service
            .execute(
                &mut bridge,
                req(Command::Start {
                    capture_id: "capture".into(),
                    context: wrong
                })
            )
            .unwrap_err()
            .code,
        "context_identity_mismatch"
    );
    assert!(bridge.store.require("capture").unwrap().proposal.is_none());
}
#[test]
fn recovered_intent_is_visible_during_admission_without_retry() {
    let (dir, mut bridge, _old) = setup();
    std::fs::write(
        dir.path().join("intents/capture.intent.json"),
        b"interrupted",
    )
    .unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let adapter = BridgeAdapter::new(
        dir.path().join("intents"),
        1,
        2,
        4,
        Arc::new(move |_, _, _| {
            count.fetch_add(1, Ordering::SeqCst);
            panic!("must not retry")
        }),
    )
    .unwrap();
    let mut service = NativeService::new(adapter);
    let context = NativeService::context_identity(&bridge, "capture", vec![]).unwrap();
    let response = service
        .execute(
            &mut bridge,
            req(Command::Admit {
                capture_id: "capture".into(),
                context: context.clone(),
                supported_features: vec![],
            }),
        )
        .unwrap();
    assert!(matches!(
        response.status.conversion,
        ConversionState::RecoveryRequired
    ));
    service
        .execute(
            &mut bridge,
            req(Command::Start {
                capture_id: "capture".into(),
                context,
            }),
        )
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
#[test]
fn missing_current_context_revokes_pending_status_without_journaling() {
    let (_dir, mut bridge, mut service) = setup();
    let context = NativeService::context_identity(&bridge, "capture", vec![]).unwrap();
    service
        .execute(
            &mut bridge,
            req(Command::Admit {
                capture_id: "capture".into(),
                context: context.clone(),
                supported_features: vec![],
            }),
        )
        .unwrap();
    service
        .execute(
            &mut bridge,
            req(Command::Start {
                capture_id: "capture".into(),
                context: context.clone(),
            }),
        )
        .unwrap();
    wait(&service.status_handle(), &context);
    bridge
        .open_document(Document {
            project_id: "project".into(),
            path: "main.tex".into(),
            revision: 2,
            text: "replaced whole snapshot".into(),
        })
        .unwrap();
    assert_eq!(
        service
            .execute(
                &mut bridge,
                req(Command::Reconcile {
                    capture_id: "capture".into(),
                    context: context.clone()
                })
            )
            .unwrap_err()
            .code,
        "context_unavailable"
    );
    assert!(matches!(
        service
            .status_handle()
            .snapshot("capture", &context)
            .unwrap()
            .conversion,
        ConversionState::StaleContext
    ));
    assert!(bridge.store.require("capture").unwrap().proposal.is_none());
}
