#![cfg(feature = "bridge-integration")]
use flashtex_bridge::Proposal;
use flashtex_conversion_jobs::bridge_adapter::{native::ContextIdentity, review::*};
use serde_json::{json, Value};
use std::{
    io::Write,
    process::{Command, Stdio},
};
fn run(dir: &std::path::Path, bytes: &[u8]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_flashtex-review-inbox"))
        .arg(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
fn request(kind: &str, payload: Value, now: u64) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(
        &json!({"protocol_version":1,"id":"request","now":now,"type":kind,"payload":payload}),
    )
    .unwrap();
    bytes.push(b'\n');
    bytes
}
#[test]
fn subprocess_restart_expires_selection_and_refuses_handoff() {
    let dir = tempfile::tempdir().unwrap();
    let context = ContextIdentity {
        project_id: "project".into(),
        path: "main.tex".into(),
        revision: 1,
        sha256: "a".repeat(64),
    };
    let proposal = Proposal {
        latex: "$x$".into(),
        ambiguities: vec![],
        required_dependencies: vec![],
    };
    let decision;
    {
        let mut inbox = ReviewInbox::open(dir.path(), Default::default()).unwrap();
        inbox
            .admit("capture", context.clone(), proposal.clone())
            .unwrap();
        inbox.set_expiry("capture", 10).unwrap();
        decision = DecisionRequest {
            decision_id: "decision".into(),
            capture_id: "capture".into(),
            expected_context: context.clone(),
            expected_proposal_sha256: proposal_hash(&proposal).unwrap(),
            intent: ReviewIntent::AcceptForPreparation,
        };
    }
    let mut commands = request("select", json!({"capture_id":"capture"}), 1);
    commands.extend(request("decide", json!({"decision":decision}), 1));
    let result = run(dir.path(), &commands);
    assert_eq!(result[0]["type"], "review_result");
    assert_eq!(result[1]["type"], "review_result");
    let token = result[1]["payload"].clone();
    let mut commands = request("snapshot", Value::Null, 10);
    commands.extend(request(
        "validate_handoff",
        json!({"handoff":token,"context":context}),
        10,
    ));
    let result = run(dir.path(), &commands);
    assert_eq!(result[0]["payload"]["entries"]["capture"]["expired"], true);
    assert!(result[0]["payload"]["selected_capture"].is_null());
    assert_eq!(result[1]["payload"]["code"], "expired");
}
#[test]
fn malformed_oversized_and_partial_frames_do_not_prevent_valid_next_request() {
    let dir = tempfile::tempdir().unwrap();
    let mut input = b"{bad}\n".to_vec();
    input.extend(vec![b'x'; 512 * 1024 + 1]);
    input.push(b'\n');
    input.extend(request("snapshot", Value::Null, 0));
    input.extend(b"{\"protocol_version\":1");
    let result = run(dir.path(), &input);
    assert_eq!(result.len(), 4);
    assert_eq!(result[0]["payload"]["code"], "invalid_request");
    assert_eq!(result[1]["payload"]["code"], "frame_too_large");
    assert_eq!(result[2]["type"], "review_result");
    assert_eq!(result[3]["payload"]["code"], "incomplete_frame");
    assert!(run(dir.path(), b"").is_empty());
}

#[test]
fn strict_envelopes_unknown_commands_and_fields_never_mutate_inbox() {
    let dir = tempfile::tempdir().unwrap();
    let mut bytes = Vec::new();
    for value in [
        json!({"protocol_version":2,"id":"r","now":0,"type":"snapshot","payload":null}),
        json!({"protocol_version":1,"id":"r","now":0,"type":"snapshot","payload":null,"extra":true}),
        json!({"protocol_version":1,"id":"r","now":0,"type":"apply_source","payload":{}}),
        json!({"protocol_version":1,"id":"r","now":0,"type":"select","payload":{"capture_id":null,"extra":true}}),
        json!({"protocol_version":1,"id":"r","type":"snapshot","payload":null}),
    ] {
        bytes.extend(serde_json::to_vec(&value).unwrap());
        bytes.push(b'\n');
    }
    let result = run(dir.path(), &bytes);
    assert_eq!(result.len(), 5);
    assert!(result
        .iter()
        .all(|r| r["payload"]["code"] == "invalid_request"));
    assert!(!dir.path().join("inbox.json").exists());
}

#[test]
fn corrupt_recovery_fails_closed_and_preserves_exact_bytes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("inbox.json"), b"{truncated").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_flashtex-review-inbox"))
        .arg(dir.path())
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        std::fs::read(dir.path().join("inbox.json")).unwrap(),
        b"{truncated"
    );
}
