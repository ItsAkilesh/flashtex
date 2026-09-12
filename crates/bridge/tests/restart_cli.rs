//! Reproduces the measured restart asymmetry through the real compiled
//! `flashtex-bridge` binary and genuinely separate OS processes -- not just
//! library calls -- matching how the Mac app actually drives this bridge as a
//! subprocess over JSON Lines on stdin/stdout.
//!
//! `capture_convert` is not exercised here: the real dispatch for it always
//! builds a live `GrokClient` and makes a network call (see `grok.rs` /
//! `main.rs`), which this offline suite must not do. Instead, `seed_proposal`
//! attaches a proposal the same way `capture_convert` would internally
//! (`Bridge::convert` with a fake `Converter`), as a library call against the
//! same on-disk store in between two real subprocess spawns. Every other step
//! -- including the one that reproduces the reported bug and the one that
//! recovers from it -- goes through the actual compiled binary.
use base64::{engine::general_purpose::STANDARD, Engine};
use flashtex_bridge::{store::Store, *};
use serde_json::{json, Value};
use std::io::{Cursor, Write};
use std::path::Path;
use std::process::{Command, Stdio};

fn request(id: &str, kind: &str, payload: Value) -> String {
    format!(
        "{}\n",
        json!({"protocol_version":1,"id":id,"type":kind,"payload":payload})
    )
}
/// Spawns a genuinely separate `flashtex-bridge` OS process against `store`,
/// writes `input` to its stdin, closes stdin, and returns its parsed replies.
/// Each call is a fresh process image with an empty `documents`/`anchors`
/// map; only what `Store` persisted on disk is available to it.
fn spawn(store: &Path, input: &str) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_flashtex-bridge"))
        .arg("--store")
        .arg(store)
        .env_remove("XAI_API_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
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
fn capture_payload() -> Value {
    let mut image = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(1, 1)
        .write_to(&mut image, image::ImageFormat::Png)
        .unwrap();
    json!({
        "capture_id": "restart-cli-capture",
        "destination_id": "restart-cli-anchor",
        "base_revision": 1,
        "image": {"mime_type":"image/png","data_base64": STANDARD.encode(image.into_inner())},
        "instructions": "preserve notation",
    })
}
struct OfflineConverter;
impl Converter for OfflineConverter {
    fn convert(&self, _: &CaptureSubmit, _: &Context) -> Result<Proposal> {
        Ok(Proposal {
            latex: "$x$".into(),
            ambiguities: vec![],
            required_dependencies: vec![],
        })
    }
}
/// Stands in for a real (paid) `capture_convert`, offline. Opens and releases
/// the store's exclusive lock like any other bridge process would.
fn seed_proposal(store: &Path) {
    let mut b = Bridge::new(Store::open(store).unwrap());
    b.open_document(Document {
        project_id: "proj".into(),
        path: "main.tex".into(),
        revision: 1,
        text: "α target".into(),
    })
    .unwrap();
    b.pin("restart-cli-anchor", "proj", "main.tex", 1, 3, 9)
        .unwrap();
    b.convert("restart-cli-capture", vec![], &OfflineConverter)
        .unwrap();
}
fn open_and_pin() -> String {
    [
        request(
            "open",
            "document_open",
            json!({"project_id":"proj","path":"main.tex","revision":1,"text":"α target"}),
        ),
        request(
            "pin",
            "destination_pin",
            json!({"destination_id":"restart-cli-anchor","project_id":"proj","path":"main.tex","revision":1,"start_byte":3,"end_byte":9}),
        ),
    ]
    .concat()
}

/// The full measured sequence, driven through real, independently spawned OS
/// processes against one `--store` directory:
///
/// 1. Process A: document_open, destination_pin, capture_submit.
/// 2. (offline conversion attached to the store, standing in for a paid
///    capture_convert)
/// 3. Process B: document_open, destination_pin, capture_prepare_insert --
///    then exits without confirming.
/// 4. Process C: capture_applied ALONE, with no document_open. This is the
///    exact reported failure: "document_missing" / "Open the source snapshot
///    on the Mac first".
/// 5. Process D: the documented recovery -- document_open, then
///    capture_applied -- succeeds, with no second paid conversion anywhere
///    in this test.
#[test]
fn capture_applied_in_a_freshly_spawned_process_fails_then_recovers_after_reopening_document() {
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path();

    let replies = spawn(
        store,
        &[
            open_and_pin(),
            request("submit", "capture_submit", capture_payload()),
        ]
        .concat(),
    );
    assert_eq!(replies[2]["type"], "capture_received");
    assert_eq!(replies[2]["payload"]["has_proposal"], false);

    seed_proposal(store);

    let replies = spawn(
        store,
        &[
            open_and_pin(),
            request(
                "prepare",
                "capture_prepare_insert",
                json!({"capture_id":"restart-cli-capture","expected_revision":1,"approved":true}),
            ),
        ]
        .concat(),
    );
    assert_eq!(replies[2]["type"], "capture_edit", "{replies:?}");
    let edit_id = replies[2]["payload"]["edit_id"]
        .as_str()
        .unwrap()
        .to_owned();

    // The measured bug: a second, freshly spawned process sends only
    // capture_applied, never reopening the document.
    let replies = spawn(
        store,
        &request(
            "apply",
            "capture_applied",
            json!({"capture_id":"restart-cli-capture","edit_id":edit_id,"new_revision":2}),
        ),
    );
    assert_eq!(replies[0]["type"], "error");
    assert_eq!(replies[0]["payload"]["code"], "document_missing");
    assert_eq!(
        replies[0]["payload"]["message"],
        "Open the source snapshot on the Mac first"
    );

    // The capture is durable across that failed attempt: the paid proposal
    // and the prepared edit are exactly as they were before process C ran.
    let record = Store::open(store)
        .unwrap()
        .require("restart-cli-capture")
        .unwrap();
    assert!(record.proposal.is_some());
    assert!(record.prepared.is_some());
    assert!(record.applied.is_none());

    // The documented recovery: resend document_open, then capture_applied.
    let replies = spawn(
        store,
        &[
            request(
                "open",
                "document_open",
                json!({"project_id":"proj","path":"main.tex","revision":1,"text":"α target"}),
            ),
            request(
                "apply",
                "capture_applied",
                json!({"capture_id":"restart-cli-capture","edit_id":edit_id,"new_revision":2}),
            ),
        ]
        .concat(),
    );
    assert_eq!(
        replies[1]["type"], "capture_application_received",
        "{replies:?}"
    );
    assert_eq!(replies[1]["payload"]["new_revision"], 2);
    assert_eq!(replies[1]["payload"]["edit_id"], edit_id);

    let record = Store::open(store)
        .unwrap()
        .require("restart-cli-capture")
        .unwrap();
    assert_eq!(record.applied.unwrap().new_revision, 2);
}
