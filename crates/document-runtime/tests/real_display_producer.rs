//! Explicit opt-in replay; caller must verify executable and resource provenance.
use flashtex_document_runtime::{Document, Event, Limits, Request, Session};
use std::{
    process::Command,
    thread,
    time::{Duration, Instant},
};
#[test]
#[ignore = "requires exact published producer/assets; run tools/replay_display_producer.py"]
fn published_producer_negotiates_source_bound_candidates() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/display-producer-request.json")).unwrap();
    let source = fixture["payload"]["documents"][0]["text"].as_str().unwrap();
    let binary = std::env::var("FLASHTEX_REPLAY_PRODUCER").unwrap();
    let mut reports = vec![];
    for (mode, limit, requested, expected) in [
        ("requested", None, true, "ok"),
        ("legacy", None, false, "ok"),
        ("declined", Some("1500"), true, "recovered"),
        ("failed", Some("1"), true, "failed"),
    ] {
        let mut command = Command::new(&binary);
        command.env_remove("FLASHTEX_MAX_REPLY_BYTES");
        if let Some(limit) = limit {
            command.env("FLASHTEX_MAX_REPLY_BYTES", limit);
        }
        let mut session = Session::spawn_command(command, Limits::default()).unwrap();
        session.set_display_candidates_enabled(requested).unwrap();
        session
            .submit_with_capabilities(
                Request {
                    id: "real-1".into(),
                    project_id: "p".into(),
                    revision: 1,
                    entry_path: "main.tex".into(),
                    documents: vec![Document {
                        path: "main.tex".into(),
                        text: source.into(),
                    }],
                },
                if requested {
                    vec!["display-list-v2".into()]
                } else {
                    vec![]
                },
            )
            .unwrap();
        let start = Instant::now();
        let mut result = None;
        let mut candidate = None;
        while start.elapsed() < Duration::from_secs(10) {
            for event in session.poll() {
                match event {
                    Event::Preview { result: r, .. } => result = Some(r),
                    Event::Failed { reason, .. } => panic!("{mode}: {reason}"),
                    _ => {}
                }
            }
            if let Some(c) = session.take_current_display_candidate() {
                candidate = Some(c.into_envelope());
            }
            if result.is_some() && (mode != "requested" || candidate.is_some()) {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        let result = result.expect(mode);
        assert_eq!(result["payload"]["status"], expected, "{mode}");
        assert_eq!(candidate.is_some(), mode == "requested");
        assert!(session.is_alive());
        reports.push(serde_json::json!({"mode":mode,"result":result,"candidate":candidate,"transport_acceptance":true,"rendering_validation":"not performed"}));
    }
    if let Ok(path) = std::env::var("FLASHTEX_REPLAY_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&reports).unwrap()).unwrap();
    }
}

#[test]
#[ignore = "requires exact published producer/assets; run tools/replay_display_producer.py"]
fn actual_producer_slow_consumer_cancellation_and_epoch_replay() {
    let binary = std::env::var("FLASHTEX_REPLAY_PRODUCER").unwrap();
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/display-producer-request.json")).unwrap();
    let text = fixture["payload"]["documents"][0]["text"].as_str().unwrap();
    let make = |revision| Request {
        id: format!("slow-{revision}"),
        project_id: "p".into(),
        revision,
        entry_path: "main.tex".into(),
        documents: vec![Document {
            path: "main.tex".into(),
            text: text.replace(
                "Office AV fi.",
                &format!("Office AV fi. Revision {revision}."),
            ),
        }],
    };
    let mut command = Command::new(binary);
    command.env_remove("FLASHTEX_MAX_REPLY_BYTES");
    let mut session = Session::spawn_command(command, Limits::default()).unwrap();
    session.set_display_candidates_enabled(true).unwrap();
    let caps = || vec!["display-list-v2".into()];
    // No polling between submission and cancellation: any queued real reply is obsolete.
    session.submit_with_capabilities(make(1), caps()).unwrap();
    session.close_project("p").unwrap();
    let cancelled = drain_for(&mut session, Duration::from_millis(100));
    assert!(cancelled
        .iter()
        .any(|e| matches!(e, Event::Cancelled {id} if id=="slow-1")));
    assert!(!cancelled.iter().any(|e| matches!(e, Event::Preview { .. })));
    assert!(session.take_current_display_candidate().is_none());

    session.submit_with_capabilities(make(2), caps()).unwrap();
    let first = wait_preview(&mut session, 2);
    assert!(first
        .iter()
        .any(|e| matches!(e, Event::Preview { revision: 2, .. })));
    // Emulate busy downstream admission, not a native validation benchmark.
    let delayed = Instant::now();
    thread::sleep(Duration::from_millis(60));
    assert!(drain_for(&mut session, Duration::from_millis(20)).is_empty());
    let held = take_real_candidate(&mut session);
    let observed_delay = delayed.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(held.revision(), 2);
    assert!(session.take_current_display_candidate().is_none());
    let held_hash = held.sources()[0].sha256.clone();

    session.submit_with_capabilities(make(3), caps()).unwrap();
    session.set_display_candidates_enabled(false).unwrap();
    session.set_display_candidates_enabled(true).unwrap();
    let toggled = wait_preview(&mut session, 3);
    assert!(toggled
        .iter()
        .any(|e| matches!(e, Event::Preview { revision: 3, .. })));
    assert!(session.take_current_display_candidate().is_none());

    session.submit_with_capabilities(make(4), caps()).unwrap();
    session.submit_with_capabilities(make(5), caps()).unwrap();
    let latest = wait_preview(&mut session, 5);
    let discarded = latest
        .iter()
        .find_map(|e| match e {
            Event::Stale { revision: 4, .. } => Some("stale_inflight"),
            Event::Superseded { id, .. } if id == "slow-4" => Some("coalesced_before_dispatch"),
            _ => None,
        })
        .expect("revision4 suppressed");
    assert!(!latest
        .iter()
        .any(|e| matches!(e, Event::Preview { revision: 4, .. })));
    assert!(latest
        .iter()
        .any(|e| matches!(e, Event::Preview { revision: 5, .. })));
    let current = take_real_candidate(&mut session);
    assert_eq!(current.revision(), 5);
    assert_eq!(
        current.sources()[0].sha256,
        flashtex_project_files::sha256_hex(make(5).documents[0].text.as_bytes())
    );
    assert_ne!(held_hash, current.sources()[0].sha256);
    assert!(session.take_current_display_candidate().is_none());
    assert!(session.is_alive());
    if let Ok(path) = std::env::var("FLASHTEX_REPLAY_OUTPUT") {
        let evidence = serde_json::json!({"format":"actual-producer-cancellation-v1",
            "injected_consumer_delay_ms":60,"observed_delay_and_followup_ms":observed_delay,
            "cancelled_preview_suppressed":true,"toggled_candidate_suppressed":true,
            "legacy_preview_revisions":[2,3,5],"suppressed_revision":4,"suppression_mode":discarded,
            "candidate_slot_second_take_empty":true,
            "held_old_candidate":held.into_envelope(),"current_candidate":current.into_envelope(),
            "downstream_boundary":"moved candidates cannot be revoked; consumer must recheck source/session epochs before paint",
            "native_validation_or_latency":"not measured"});
        std::fs::write(
            std::path::Path::new(&path).with_file_name("lifecycle.json"),
            serde_json::to_vec_pretty(&evidence).unwrap(),
        )
        .unwrap();
    }
}
fn drain_for(session: &mut Session, duration: Duration) -> Vec<Event> {
    let started = Instant::now();
    let mut events = vec![];
    while started.elapsed() < duration {
        events.extend(session.poll());
        thread::sleep(Duration::from_millis(2));
    }
    assert!(session.is_alive());
    events
}

fn wait_preview(s: &mut Session, revision: u64) -> Vec<Event> {
    let start = Instant::now();
    let mut events = vec![];
    loop {
        events.extend(s.poll());
        if events
            .iter()
            .any(|e| matches!(e,Event::Preview{revision:r,..} if *r==revision))
        {
            return events;
        }
        assert!(
            s.is_alive() && start.elapsed() < Duration::from_secs(10),
            "{events:?}"
        );
        thread::sleep(Duration::from_millis(2));
    }
}
fn take_real_candidate(s: &mut Session) -> flashtex_document_runtime::UntrustedDisplayCandidate {
    let start = Instant::now();
    loop {
        s.poll();
        if let Some(c) = s.take_current_display_candidate() {
            return c;
        }
        assert!(s.is_alive() && start.elapsed() < Duration::from_secs(10));
        thread::sleep(Duration::from_millis(2));
    }
}
