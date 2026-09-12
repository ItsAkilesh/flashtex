use flashtex_edit_ledger::{Document, Store};
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};
struct Client {
    child: Child,
    input: Option<ChildStdin>,
    output: Receiver<Value>,
}
impl Client {
    fn start(root: &std::path::Path) -> Self {
        Self::with_compiler(root, None)
    }
    fn with_compiler(root: &std::path::Path, compiler: Option<&std::path::Path>) -> Self {
        let path = root.join("store");
        {
            let mut store = Store::open(&path).unwrap();
            if store.document().unwrap().is_none() {
                store
                    .initialize(
                        Document::new("p".into(), "main.tex".into(), 1, "α original".into())
                            .unwrap(),
                    )
                    .unwrap();
            }
        }
        let config = root.join("config.json");
        std::fs::write(&config,serde_json::to_vec(&json!({"session_id":"session1","project_id":"p","entry_path":"main.tex","store_paths":[path],"compiler_path":compiler})).unwrap()).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_flashtex-preview-controller"))
            .arg(config)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let input = child.stdin.take();
        let (tx, output) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else {
                    break;
                };
                if tx.send(serde_json::from_str(&line).unwrap()).is_err() {
                    break;
                }
            }
        });
        let client = Self {
            child,
            input,
            output,
        };
        assert_eq!(
            client.output.recv_timeout(Duration::from_secs(3)).unwrap()["type"],
            "ready"
        );
        client
    }
    fn send(&mut self, id: &str, kind: &str, payload: Value) {
        let input = self.input.as_mut().unwrap();
        writeln!(input,"{}",json!({"protocol_version":1,"session_id":"session1","id":id,"type":kind,"payload":payload})).unwrap();
        input.flush().unwrap();
    }
    fn reply(&self, id: &str) -> Value {
        loop {
            let event = self.output.recv_timeout(Duration::from_secs(3)).unwrap();
            if event["id"] == id {
                return event;
            }
        }
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[test]
fn eof_drains_durable_edit_response_and_reopen_reads_source() {
    let dir = tempfile::tempdir().unwrap();
    let mut client = Client::start(dir.path());
    client.send("get", "document", json!({"path":"main.tex"}));
    let document = client.reply("get")["payload"]["document"].clone();
    client.send("edit","edit",json!({"path":"main.tex","expected_revision":1,"expected_sha256":document["source_sha256"],"text":"β durable"}));
    client.input.take();
    let response = client.reply("edit");
    assert_eq!(response["payload"]["document"]["text"], "β durable");
    assert!(client.child.wait().unwrap().success());
    drop(client);
    let mut reopened = Client::start(dir.path());
    reopened.send("get", "document", json!({"path":"main.tex"}));
    assert_eq!(reopened.reply("get")["payload"]["document"]["revision"], 2);
}
#[test]
fn review_token_requires_explicit_approval_and_kill_preserves_receipt() {
    let dir = tempfile::tempdir().unwrap();
    let mut client = Client::start(dir.path());
    client.send("get", "document", json!({"path":"main.tex"}));
    let doc = client.reply("get")["payload"]["document"].clone();
    let edit = json!({"capture_id":"capture1","edit_id":"edit1","project_id":"p","path":"main.tex","expected_revision":1,"start_byte":0,"end_byte":2,"removed_text":"α","replacement":"β","document_before_sha256":doc["source_sha256"]});
    client.send("review", "review", json!({"edit":edit}));
    let token = client.reply("review")["payload"]["approval_token"].clone();
    assert_eq!(token.as_str().unwrap().len(), 64);
    client.send(
        "no",
        "apply_reviewed",
        json!({"approval_token":token,"user_approved":false}),
    );
    assert_eq!(client.reply("no")["type"], "error");
    client.send(
        "yes",
        "apply_reviewed",
        json!({"approval_token":token,"user_approved":true}),
    );
    let applied = client.reply("yes");
    assert_eq!(applied["payload"]["receipt"]["new_revision"], 2);
    client.send(
        "again",
        "apply_reviewed",
        json!({"approval_token":token,"user_approved":true}),
    );
    assert_eq!(
        client.reply("again")["payload"]["receipt"],
        applied["payload"]["receipt"]
    );
    drop(client); // kill helper without graceful protocol close
    let mut reopened = Client::start(dir.path());
    reopened.send("recover", "recovery", json!({"path":"main.tex"}));
    let recovery = reopened.reply("recover");
    assert_eq!(
        recovery["payload"]["transactions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    reopened.send("get", "document", json!({"path":"main.tex"}));
    assert_eq!(
        reopened.reply("get")["payload"]["document"]["text"],
        "β original"
    );
}

#[test]
#[ignore = "requires explicitly configured original compiler"]
fn helper_streams_original_compiler_result_for_latest_durable_edit() {
    let binary = std::env::var_os("FLASHTEX_TEST_COMPILER").expect("original compiler required");
    let dir = tempfile::tempdir().unwrap();
    let mut client = Client::with_compiler(dir.path(), Some(std::path::Path::new(&binary)));
    client.send("get", "document", json!({"path":"main.tex"}));
    let doc = client.reply("get")["payload"]["document"].clone();
    client.send("edit","edit",json!({"path":"main.tex","expected_revision":1,"expected_sha256":doc["source_sha256"],"text":"Actual streamed compiler preview"}));
    assert!(client.reply("edit")["payload"]["preview_error"].is_null());
    loop {
        let event = client.output.recv_timeout(Duration::from_secs(3)).unwrap();
        if event["type"] == "update"
            && event["payload"]["kind"] == "preview"
            && event["payload"]["source_versions"]["main.tex"] == 2
        {
            assert_eq!(event["payload"]["result"]["payload"]["status"], "ok");
            break;
        }
    }
}

#[test]
fn native_queries_check_versions_and_return_utf8_source_navigation() {
    let dir = tempfile::tempdir().unwrap();
    let mut client = Client::start(dir.path());
    client.send("get", "document", json!({"path":"main.tex"}));
    let doc = client.reply("get")["payload"]["document"].clone();
    let text = "α \\label{chapter} \\ref{chapter}";
    client.send("edit","edit",json!({"path":"main.tex","expected_revision":1,"expected_sha256":doc["source_sha256"],"text":text}));
    assert_eq!(client.reply("edit")["type"], "result");
    client.send("snapshot", "snapshot", json!({}));
    let versions = client.reply("snapshot")["payload"]["source_versions"].clone();
    client.send(
        "complete",
        "complete",
        json!({"source_versions":versions,"category":"label","prefix":"cha","limit":10}),
    );
    assert_eq!(
        client.reply("complete")["payload"]["completions"][0]["name"],
        "chapter"
    );
    client.send("nav","navigate",json!({"source_versions":versions,"path":"main.tex","byte_offset":text.rfind("chapter").unwrap()}));
    let navigation = client.reply("nav")["payload"]["navigation"].clone();
    assert_eq!(navigation["definitions"].as_array().unwrap().len(), 1);
    assert_eq!(
        navigation["definitions"][0]["start_byte"],
        text.find("chapter").unwrap()
    );
    client.send(
        "stale",
        "complete",
        json!({"source_versions":{"main.tex":1},"category":"label","prefix":"cha","limit":10}),
    );
    assert_eq!(client.reply("stale")["type"], "error");
    client.send(
        "badutf8",
        "navigate",
        json!({"source_versions":versions,"path":"main.tex","byte_offset":1}),
    );
    assert_eq!(client.reply("badutf8")["type"], "error");
}

#[test]
fn undo_retry_after_helper_kill_is_idempotent_and_redo_remains_available() {
    let dir = tempfile::tempdir().unwrap();
    let mut client = Client::start(dir.path());
    client.send("get", "document", json!({"path":"main.tex"}));
    let doc = client.reply("get")["payload"]["document"].clone();
    client.send("edit","edit",json!({"path":"main.tex","expected_revision":1,"expected_sha256":doc["source_sha256"],"text":"typed"}));
    let edited = client.reply("edit")["payload"]["document"].clone();
    let undo = json!({"command_id":"undo1","expected_revision":2,"expected_sha256":edited["source_sha256"]});
    client.send("undo", "undo", json!({"path":"main.tex","command":undo}));
    let undone = client.reply("undo")["payload"]["history"].clone();
    assert_eq!(undone["document"]["text"], "α original");
    assert_eq!(undone["document"]["revision"], 3);
    drop(client);
    let mut reopened = Client::start(dir.path());
    reopened.send("retry", "undo", json!({"path":"main.tex","command":undo}));
    let replayed = reopened.reply("retry")["payload"]["history"].clone();
    assert_eq!(replayed["replayed_command"], true);
    assert_eq!(replayed["document"]["revision"], 3);
    reopened.send("redo","redo",json!({"path":"main.tex","command":{"command_id":"redo1","expected_revision":3,"expected_sha256":undone["document"]["source_sha256"]}}));
    assert_eq!(
        reopened.reply("redo")["payload"]["history"]["document"]["text"],
        "typed"
    );
}
