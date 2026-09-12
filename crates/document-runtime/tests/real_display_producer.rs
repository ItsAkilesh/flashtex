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
