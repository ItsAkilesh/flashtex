use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};
fn run(input: &[u8]) -> Vec<Value> {
    let dir = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_flashtex-bridge"))
        .arg("--store")
        .arg(dir.path())
        .env_remove("XAI_API_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}
fn request(id: &str, kind: &str, payload: Value) -> String {
    format!(
        "{}\n",
        json!({"protocol_version":1,"id":id,"type":kind,"payload":payload})
    )
}
#[test]
fn malformed_frame_does_not_poison_next_request_and_provider_is_disabled() {
    let input = format!(
        "{{bad\n{}",
        request("convert", "capture_convert", json!({"capture_id":"test"}))
    );
    let replies = run(input.as_bytes());
    assert_eq!(replies.len(), 2);
    assert_eq!(replies[0]["id"], Value::Null);
    assert_eq!(replies[0]["payload"]["code"], "invalid_json");
    assert_eq!(replies[1]["id"], "convert");
    assert_eq!(replies[1]["payload"]["code"], "provider_disabled");
}
#[test]
fn real_wire_fixture_receipt_and_rejection() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../protocol/fixtures/capture-submission.json"
    ))
    .unwrap();
    let input=[
        request("open","document_open",json!({"project_id":"p","path":"main.tex","revision":1,"text":"α"})),
        request("pin","destination_pin",json!({"destination_id":"fixture-anchor-1","project_id":"p","path":"main.tex","revision":1,"start_byte":2,"end_byte":2})),
        format!("{fixture}\n"),
        request("reject","capture_reject",json!({"capture_id":"fixture-capture-1"})),
        request("status","capture_status",json!({"capture_id":"fixture-capture-1"})),
    ].concat();
    let replies = run(input.as_bytes());
    assert_eq!(replies.len(), 5);
    assert_eq!(replies[2]["type"], "capture_received");
    assert_eq!(replies[2]["payload"]["durable"], true);
    assert_eq!(replies[3]["type"], "capture_rejected");
    assert_eq!(replies[4]["payload"]["rejected"], true);
}
#[test]
fn oversized_frame_is_drained_and_next_request_is_processed() {
    let mut input = vec![b'x'; 12 * 1024 * 1024 + 9];
    input.push(b'\n');
    input.extend(request("next", "unknown", json!({})).as_bytes());
    let replies = run(&input);
    assert_eq!(replies.len(), 2);
    assert_eq!(replies[0]["payload"]["code"], "message_too_large");
    assert_eq!(replies[1]["id"], "next");
    assert_eq!(replies[1]["payload"]["code"], "unsupported_type");
}
