//! Opt-in exact-peer gate over existing fixtures; no old-version geometry oracle.
use flashtex_document_runtime::{Document, Event, Limits, Request, Session};
use serde_json::{json, Value};
use std::{
    process::Command,
    thread,
    time::{Duration, Instant},
};

fn spawn(binary: &str, limit: Option<&str>) -> Session {
    let mut command = Command::new(binary);
    command.env_remove("FLASHTEX_MAX_REPLY_BYTES");
    if let Some(limit) = limit {
        command.env("FLASHTEX_MAX_REPLY_BYTES", limit);
    }
    let mut session =
        Session::spawn_command_raw_display_prototype(command, Limits::default()).unwrap();
    session.set_display_candidates_enabled(true).unwrap();
    session
}

fn collect(session: &mut Session, request: &Value) -> Value {
    let p = &request["payload"];
    let request = Request {
        id: request["id"].as_str().unwrap().into(),
        project_id: p["project_id"].as_str().unwrap().into(),
        revision: p["revision"].as_u64().unwrap(),
        entry_path: p["entry_path"].as_str().unwrap().into(),
        documents: p["documents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| Document {
                path: d["path"].as_str().unwrap().into(),
                text: d["text"].as_str().unwrap().into(),
            })
            .collect(),
    };
    let caps = p["layout_capabilities"]
        .as_array()
        .map(|v| v.iter().map(|s| s.as_str().unwrap().into()).collect())
        .unwrap_or_default();
    session
        .submit_with_capabilities(request.clone(), caps)
        .unwrap();
    let start = Instant::now();
    let mut result = None;
    let mut candidate = None;
    loop {
        for event in session.poll() {
            match event {
                Event::Preview {
                    revision,
                    result: value,
                    ..
                } => {
                    assert_eq!(revision, request.revision);
                    assert!(result.replace(value).is_none());
                }
                Event::Failed { reason, .. } => panic!("runtime refused {}: {reason}", request.id),
                _ => {}
            }
        }
        if let Some(value) = session.take_current_raw_display_candidate() {
            assert_eq!(value.request_id(), request.id);
            assert_eq!(value.project_id(), request.project_id);
            assert_eq!(value.revision(), request.revision);
            let envelope: Value = serde_json::from_str(value.raw().get()).unwrap();
            candidate = Some(json!({"raw":value.raw().get(),"envelope":envelope}));
        }
        if let Some(result) = &result {
            let accepted = result["payload"]["status"] != "failed"
                && result["payload"]["layout_capabilities"]
                    .as_array()
                    .is_some_and(|v| v.iter().any(|s| s == "display-list-v2"));
            if !accepted || candidate.is_some() {
                assert!(accepted || candidate.is_none());
                return json!({"result":result,"candidate":candidate});
            }
        }
        assert!(
            session.is_alive() && start.elapsed() < Duration::from_secs(15),
            "{} did not complete",
            request.id
        );
        thread::sleep(Duration::from_millis(2));
    }
}

#[test]
#[ignore = "requires caller-verified exact producer and pinned font/TFM assets"]
fn same_peer_full_outputs_match_fresh_and_persistent_existing_fixtures() {
    let binary = std::env::var("FLASHTEX_FULL_PEER_BINARY").unwrap();
    let cases: Value = serde_json::from_slice(
        &std::fs::read(std::env::var("FLASHTEX_FULL_PEER_CASES").unwrap()).unwrap(),
    )
    .unwrap();
    let output = std::path::PathBuf::from(std::env::var("FLASHTEX_FULL_PEER_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut report = vec![];
    for group in cases.as_array().unwrap() {
        for limit in [None, Some(group["reply_limit"].as_str().unwrap())] {
            let mut persistent = spawn(&binary, limit);
            for case in group["cases"].as_array().unwrap() {
                let warm = collect(&mut persistent, &case["request"]);
                let cold = collect(&mut spawn(&binary, limit), &case["request"]);
                let stem = format!(
                    "{}-{}-{}",
                    group["name"].as_str().unwrap(),
                    case["case_id"].as_str().unwrap(),
                    limit.unwrap_or("default")
                );
                // Preserve both complete replies even when the equality gate fails.
                std::fs::write(
                    output.join(format!("{stem}.persistent.json")),
                    serde_json::to_vec(&warm).unwrap(),
                )
                .unwrap();
                std::fs::write(
                    output.join(format!("{stem}.fresh.json")),
                    serde_json::to_vec(&cold).unwrap(),
                )
                .unwrap();
                assert_eq!(warm, cold, "same-peer mismatch {stem}");
                let result = &warm["result"]["payload"];
                let pages = result["pages"].as_array();
                if group["name"] == "multipage" && result["status"] != "failed" {
                    let pages = pages.expect("nonfailed result pages");
                    assert!(!pages.is_empty());
                    let joined = pages
                        .iter()
                        .flat_map(|p| p["items"].as_array().unwrap())
                        .filter_map(|i| i["text"].as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    assert!(joined.contains("FINAL DOCUMENT SENTINEL."));
                }
                report.push(json!({"case":stem,"full_equal":true,"status":result["status"],"diagnostics":result["diagnostics"],"page_item_counts":pages.map(|v| v.iter().map(|p| p["items"].as_array().unwrap().len()).collect::<Vec<_>>()),"candidate_present":!warm["candidate"].is_null()}));
                std::fs::write(
                    output.join("report.json"),
                    serde_json::to_vec_pretty(&report).unwrap(),
                )
                .unwrap();
            }
        }
    }
}
