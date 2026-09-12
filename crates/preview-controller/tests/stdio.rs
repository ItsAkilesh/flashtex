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
        Self::configured(
            root,
            json!({"session_id":"session1","project_id":"p","entry_path":"main.tex","store_paths":[path],"compiler_path":compiler}),
        )
    }
    fn configured(root: &std::path::Path, value: Value) -> Self {
        let config = root.join("config.json");
        std::fs::write(&config, serde_json::to_vec(&value).unwrap()).unwrap();
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

#[test]
fn stalled_output_reader_causes_bounded_failure_instead_of_unlimited_queueing() {
    stalled_reader(40);
}

#[test]
fn single_stalled_reply_times_out_without_filling_output_queue() {
    stalled_reader(1);
}

fn stalled_reader(requests: usize) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store");
    {
        let mut store = Store::open(&path).unwrap();
        store
            .initialize(
                Document::new("p".into(), "main.tex".into(), 1, "x".repeat(1024 * 1024)).unwrap(),
            )
            .unwrap();
    }
    let config = dir.path().join("config.json");
    std::fs::write(&config,serde_json::to_vec(&json!({"session_id":"session1","project_id":"p","entry_path":"main.tex","store_paths":[path]})).unwrap()).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_flashtex-preview-controller"))
        .arg(config)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut unread_output = BufReader::new(child.stdout.take().unwrap());
    let input = child.stdin.take();
    let (_sender, output) = mpsc::channel();
    let mut client = Client {
        child,
        input,
        output,
    };
    let mut ready = String::new();
    unread_output.read_line(&mut ready).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&ready).unwrap()["type"],
        "ready"
    );
    // Keep stdout open but deliberately stop draining it. Each document reply is
    // larger than the OS pipe; bounded helper output admission must stop work.
    for id in 0..requests {
        let line = json!({"protocol_version":1,"session_id":"session1","id":format!("r{id}"),"type":"document","payload":{"path":"main.tex"}});
        if writeln!(client.input.as_mut().unwrap(), "{line}").is_err() {
            break;
        }
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    loop {
        if let Some(status) = client.child.try_wait().unwrap() {
            assert!(!status.success());
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "helper did not stop on output backpressure"
        );
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn wrong_session_cannot_edit_authoritative_source() {
    let dir = tempfile::tempdir().unwrap();
    let mut client = Client::start(dir.path());
    writeln!(client.input.as_mut().unwrap(),"{}",json!({"protocol_version":1,"session_id":"another-session","id":"wrong","type":"edit","payload":{"path":"main.tex","expected_revision":1,"expected_sha256":"unused","text":"must not save"}})).unwrap();
    assert_eq!(client.reply("wrong")["type"], "error");
    client.send("get", "document", json!({"path":"main.tex"}));
    assert_eq!(
        client.reply("get")["payload"]["document"]["text"],
        "α original"
    );
}

#[test]
fn file_project_helper_reports_external_change_without_overwrite() {
    let root = tempfile::tempdir().unwrap();
    let private = tempfile::tempdir().unwrap();
    let config_dir = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("main.tex"), "initial source").unwrap();
    let config = json!({"session_id":"session1","project_id":"p","entry_path":"main.tex","project_root":root.path(),"private_ledger_root":private.path()});
    let mut client = Client::configured(config_dir.path(), config.clone());
    client.send("status", "file_status", json!({"path":"main.tex"}));
    assert_eq!(
        client.reply("status")["payload"]["disk"]["state"],
        "matches_source"
    );
    std::fs::write(root.path().join("main.tex"), "external source").unwrap();
    client.send("status2", "file_status", json!({"path":"main.tex"}));
    assert_eq!(
        client.reply("status2")["payload"]["disk"]["state"],
        "differs_from_source"
    );
    client.send("export", "export", json!({"path":"main.tex"}));
    assert_eq!(client.reply("export")["type"], "error");
    drop(client);
    let mut reopened = Client::configured(config_dir.path(), config);
    reopened.send("get", "document", json!({"path":"main.tex"}));
    assert_eq!(
        reopened.reply("get")["payload"]["document"]["text"],
        "initial source"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.tex")).unwrap(),
        "external source"
    );
}

#[test]
fn literal_search_exposes_utf8_matches_work_limits_and_stale_version_errors() {
    let dir = tempfile::tempdir().unwrap();
    let mut client = Client::start(dir.path());
    client.send("get", "document", json!({"path":"main.tex"}));
    let doc = client.reply("get")["payload"]["document"].clone();
    client.send("edit","edit",json!({"path":"main.tex","expected_revision":1,"expected_sha256":doc["source_sha256"],"text":"α x α"}));
    assert_eq!(client.reply("edit")["type"], "result");
    let query =
        json!({"source_versions":{"main.tex":2},"literal":"α","max_matches":10,"max_work":100});
    client.send("search", "search_literal", query.clone());
    let result = client.reply("search")["payload"].clone();
    assert_eq!(result["termination"], "complete");
    assert_eq!(result["matches"].as_array().unwrap().len(), 2);
    assert_eq!(result["matches"][1]["start_byte"], 5);
    let mut bounded = query.clone();
    bounded["max_work"] = json!(1);
    client.send("bounded", "search_literal", bounded);
    assert_eq!(
        client.reply("bounded")["payload"]["termination"],
        "work_limit"
    );
    let mut stale = query;
    stale["source_versions"] = json!({"main.tex":1});
    client.send("stale", "search_literal", stale);
    assert_eq!(client.reply("stale")["type"], "error");
}

#[test]
fn helper_exports_exact_source_and_refuses_conflicting_or_ambiguous_expectations() {
    let root = tempfile::tempdir().unwrap();
    let private = tempfile::tempdir().unwrap();
    let config_dir = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("main.tex"), "initial").unwrap();
    let config = json!({"session_id":"session1","project_id":"p","entry_path":"main.tex","project_root":root.path(),"private_ledger_root":private.path()});
    let mut client = Client::configured(config_dir.path(), config.clone());
    client.send("get", "document", json!({"path":"main.tex"}));
    let old = client.reply("get")["payload"]["document"].clone();
    client.send("edit", "edit", json!({"path":"main.tex","expected_revision":1,"expected_sha256":old["source_sha256"],"text":"saved β"}));
    let new = client.reply("edit")["payload"]["document"].clone();
    let request = json!({"path":"main.tex","expected_revision":2,"expected_sha256":new["source_sha256"],"expected_disk_sha256":old["source_sha256"]});
    let mut missing = request.clone();
    missing
        .as_object_mut()
        .unwrap()
        .remove("expected_disk_sha256");
    client.send("missing", "export", missing);
    assert_eq!(client.reply("missing")["type"], "error");
    client.send("save", "export", request.clone());
    let saved = client.reply("save");
    assert_eq!(saved["type"], "result");
    assert_eq!(saved["payload"]["sha256"], new["source_sha256"]);
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.tex")).unwrap(),
        "saved β"
    );
    std::fs::write(root.path().join("main.tex"), "external").unwrap();
    client.send("conflict", "export", request);
    assert_eq!(client.reply("conflict")["type"], "error");
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.tex")).unwrap(),
        "external"
    );
    drop(client);
    let mut reopened = Client::configured(config_dir.path(), config);
    reopened.send("get", "document", json!({"path":"main.tex"}));
    assert_eq!(
        reopened.reply("get")["payload"]["document"]["text"],
        "saved β"
    );
}

#[test]
fn helper_reload_requires_approval_and_preserves_disk() {
    let root = tempfile::tempdir().unwrap();
    let private = tempfile::tempdir().unwrap();
    let config_dir = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("main.tex"), "initial").unwrap();
    let config = json!({"session_id":"session1","project_id":"p","entry_path":"main.tex","project_root":root.path(),"private_ledger_root":private.path()});
    let mut client = Client::configured(config_dir.path(), config);
    client.send("get", "document", json!({"path":"main.tex"}));
    let old = client.reply("get")["payload"]["document"].clone();
    std::fs::write(root.path().join("main.tex"), "external").unwrap();
    let hash = flashtex_project_files::sha256_hex(b"external");
    let mut request = json!({"path":"main.tex","expected_revision":1,"expected_sha256":old["source_sha256"],"expected_disk_sha256":hash});
    client.send("refused", "reload", request.clone());
    assert_eq!(client.reply("refused")["type"], "error");
    client.send("unchanged", "document", json!({"path":"main.tex"}));
    assert_eq!(client.reply("unchanged")["payload"]["document"], old);
    request["user_approved"] = json!(true);
    client.send("reload", "reload", request.clone());
    assert_eq!(
        client.reply("reload")["payload"]["document"]["text"],
        "external"
    );
    client.send("retry", "reload", request);
    assert_eq!(client.reply("retry")["type"], "error");
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.tex")).unwrap(),
        "external"
    );
}
