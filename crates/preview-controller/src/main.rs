//! Local stdio adapter. Native callers must put pipe IO on a dedicated worker.
use flashtex_document_runtime::{Event, Limits};
use flashtex_edit_ledger::{AppliedReceipt, PreparedEdit, Store};
use flashtex_preview_controller::file_project::{DiskState, FileProject};
use flashtex_preview_controller::{ApprovedEdit, Controller, HistoryAction, Update};
use flashtex_project_index::{Category, SearchRequest, SearchTermination, SourceSpan};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
const MAX_FRAME: usize = 1024 * 1024;
// Leave four MiB below the helper frame bound for its wrapping metadata.
const MAX_COMPILER_FRAME: usize = 12 * 1024 * 1024;
fn compiler_limits(config: &Value) -> Result<Limits, String> {
    let mut limits = Limits::default();
    if let Some(value) = config.get("compiler_max_frame_bytes") {
        limits.max_frame = value
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .filter(|n| (128..=MAX_COMPILER_FRAME).contains(n))
            .ok_or("compiler_max_frame_bytes must be 128..12582912")?;
    }
    Ok(limits)
}
fn string<'a>(v: &'a Value, name: &str) -> Result<&'a str, String> {
    v[name].as_str().ok_or(format!("missing string {name}"))
}
fn number(v: &Value, name: &str) -> Result<u64, String> {
    v[name].as_u64().ok_or(format!("missing integer {name}"))
}
struct OutputBuffer(Vec<u8>);
impl Write for OutputBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) > 16 * 1024 * 1024 {
            return Err(io::Error::other("output frame exceeds 16 MiB"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn emit(tx: &SyncSender<Vec<u8>>, stopped: &AtomicBool, value: Value) {
    let mut buffer = OutputBuffer(Vec::new());
    if serde_json::to_writer(&mut buffer, &value).is_err() {
        buffer.0.clear();
        let error = failure(
            value["session_id"].as_str().unwrap_or(""),
            value["id"].clone(),
            "response exceeds output limit; source may already be durable",
        );
        if serde_json::to_writer(&mut buffer, &error).is_err() {
            stopped.store(true, Ordering::SeqCst);
            return;
        }
    }
    buffer.0.push(b'\n');
    if tx.try_send(buffer.0).is_err() {
        stopped.store(true, Ordering::SeqCst);
    }
}
fn failure(session: &str, id: Value, reason: impl AsRef<str>) -> Value {
    json!({"protocol_version":1,"session_id":session,"id":id,"type":"error","payload":{"message":reason.as_ref()}})
}
fn run(config: Value) -> Result<(), String> {
    let session = string(&config, "session_id")?.to_owned();
    if session.is_empty() || session.len() > 128 {
        return Err("invalid session identity".into());
    }
    let limits = compiler_limits(&config)?;
    let project = string(&config, "project_id")?.to_owned();
    let entry = string(&config, "entry_path")?.to_owned();
    let (mut controller, file_project) = if config.get("project_root").is_some() {
        if config.get("store_paths").is_some() {
            return Err("choose project_root or store_paths, not both".into());
        }
        let (files, controller) = FileProject::open(
            std::path::Path::new(string(&config, "project_root")?),
            std::path::Path::new(string(&config, "private_ledger_root")?),
            &project,
            &entry,
        )?;
        (controller, Some(files))
    } else {
        let paths = config["store_paths"]
            .as_array()
            .ok_or("store_paths array required")?;
        if paths.is_empty() || paths.len() > 256 {
            return Err("expected 1..256 stores".into());
        }
        let stores = paths
            .iter()
            .map(|p| {
                Store::open(p.as_str().ok_or("store path must be string")?)
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, String>>()?;
        (
            Controller::open_without_compiler(project, entry, stores)?,
            None,
        )
    };
    let compiler = config
        .get("compiler_path")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let compiler_error = compiler
        .as_ref()
        .and_then(|path| controller.restart(Command::new(path), limits.clone()).err());
    let (input_tx, input_rx) = mpsc::sync_channel::<Value>(16);
    let (output_tx, output_rx) = mpsc::sync_channel::<Vec<u8>>(8);
    let stopped = Arc::new(AtomicBool::new(false));
    let output_stopped = stopped.clone();
    let output_done = Arc::new(AtomicBool::new(false));
    let writer_done = output_done.clone();
    let writing_since = Arc::new(Mutex::new(None::<std::time::Instant>));
    let writer_clock = writing_since.clone();
    thread::spawn(move || {
        let mut stdout = io::stdout().lock();
        for bytes in output_rx {
            *writer_clock.lock().unwrap() = Some(std::time::Instant::now());
            if stdout
                .write_all(&bytes)
                .and_then(|_| stdout.flush())
                .is_err()
            {
                output_stopped.store(true, Ordering::SeqCst);
                break;
            }
            *writer_clock.lock().unwrap() = None;
        }
        *writer_clock.lock().unwrap() = None;
        writer_done.store(true, Ordering::SeqCst);
    });
    let reader_output = output_tx.clone();
    let reader_stopped = stopped.clone();
    let reader_session = session.clone();
    thread::spawn(move || {
        let mut stdin = io::stdin().lock();
        loop {
            if reader_stopped.load(Ordering::SeqCst) {
                break;
            }
            let mut frame = Vec::new();
            match stdin
                .by_ref()
                .take(MAX_FRAME as u64 + 1)
                .read_until(b'\n', &mut frame)
            {
                Ok(0) => break,
                Ok(_) if frame.len() <= MAX_FRAME && frame.last() == Some(&b'\n') => {}
                _ => {
                    emit(
                        &reader_output,
                        &reader_stopped,
                        failure(&reader_session, Value::Null, "truncated or oversized input"),
                    );
                    break;
                }
            }
            let request: Value = match serde_json::from_slice(&frame) {
                Ok(value) => value,
                Err(_) => {
                    emit(
                        &reader_output,
                        &reader_stopped,
                        failure(&reader_session, Value::Null, "malformed request JSON"),
                    );
                    continue;
                }
            };
            if let Err(error) = input_tx.try_send(request) {
                match error {
                    mpsc::TrySendError::Full(request) => emit(&reader_output,&reader_stopped,failure(&reader_session,request["id"].clone(),"busy: request not admitted; retry same operation after draining replies")),
                    mpsc::TrySendError::Disconnected(_) => break,
                }
            }
        }
    });
    emit(
        &output_tx,
        &stopped,
        json!({"protocol_version":1,"session_id":session,"id":null,"type":"ready","payload":{"compiler_error":compiler_error,"compiler_max_frame_bytes":limits.max_frame,"helper_max_output_bytes":16*1024*1024}}),
    );
    let mut reviews: BTreeMap<String, PreparedEdit> = BTreeMap::new();
    while !stopped.load(Ordering::SeqCst) {
        if writing_since
            .lock()
            .unwrap()
            .is_some_and(|start| start.elapsed() >= Duration::from_secs(2))
        {
            stopped.store(true, Ordering::SeqCst);
            break;
        }
        match input_rx.recv_timeout(Duration::from_millis(2)) {
            Ok(request) => {
                let id = request["id"].clone();
                let response = if request["protocol_version"] != 1
                    || request["session_id"] != session
                    || !id.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 128)
                {
                    Err("invalid version, session or request identity".into())
                } else {
                    handle(
                        &mut controller,
                        &mut reviews,
                        &request,
                        compiler.as_deref(),
                        &limits,
                        file_project.as_ref(),
                    )
                };
                let output = match response {
                    Ok(payload) => {
                        json!({"protocol_version":1,"session_id":session,"id":id,"type":"result","payload":payload})
                    }
                    Err(reason) => failure(&session, id, reason),
                };
                emit(&output_tx, &stopped, output);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        for update in controller.poll() {
            let payload = match update {
                Update::Preview(preview) => {
                    json!({"kind":"preview","request_id":preview.request_id,"compile_revision":preview.compile_revision,"source_versions":preview.source_versions.documents,"result":preview.result,"missing_layout_capabilities":preview.missing_layout_capabilities,"runtime_total_ms":preview.runtime_total_ms,"controller_total_ms":preview.controller_total_ms})
                }
                Update::Discarded { request_id } => {
                    json!({"kind":"discarded","request_id":request_id})
                }
                Update::Runtime(event) => match event {
                    Event::Superseded { id, by_id } => {
                        json!({"kind":"superseded","request_id":id,"by_id":by_id})
                    }
                    Event::Stale { id, revision } => {
                        json!({"kind":"stale","request_id":id,"compile_revision":revision})
                    }
                    Event::Cancelled { id } => json!({"kind":"cancelled","request_id":id}),
                    Event::Failed { id, reason } => {
                        json!({"kind":"failed","request_id":id,"reason":reason})
                    }
                    Event::Preview { .. } => unreachable!("controller unwraps previews"),
                },
            };
            emit(
                &output_tx,
                &stopped,
                json!({"protocol_version":1,"session_id":session,"id":null,"type":"update","payload":payload}),
            );
        }
    }
    // Drain normal EOF replies, bounded even if the native reader stopped.
    drop(output_tx);
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while !output_done.load(Ordering::SeqCst)
        && !stopped.load(Ordering::SeqCst)
        && std::time::Instant::now() < deadline
    {
        thread::sleep(Duration::from_millis(2));
    }
    if stopped.load(Ordering::SeqCst) || !output_done.load(Ordering::SeqCst) {
        return Err(
            "output stalled; delivery uncertain, recover durable source and receipts".into(),
        );
    }
    Ok(())
}
fn source_json(source: &SourceSpan) -> Value {
    json!({"path":source.file,"revision":source.revision,"start_byte":source.start_byte,"end_byte":source.end_byte})
}
fn handle(
    controller: &mut Controller,
    reviews: &mut BTreeMap<String, PreparedEdit>,
    request: &Value,
    compiler: Option<&str>,
    limits: &Limits,
    file_project: Option<&FileProject>,
) -> Result<Value, String> {
    let p = &request["payload"];
    match string(request, "type")? {
        "document" => Ok(json!({"document":controller.document(string(p,"path")?)?})),
        "snapshot" => {
            let snapshot = controller.index().snapshot();
            Ok(
                json!({"project_id":snapshot.project_id,"source_versions":snapshot.documents,"membership_generation":snapshot.generation}),
            )
        }
        "search_literal" => {
            let snapshot = controller.index().snapshot();
            if p["source_versions"] != json!(snapshot.documents) {
                return Err("source versions changed; refresh snapshot before searching".into());
            }
            let max_matches =
                usize::try_from(number(p, "max_matches")?).map_err(|_| "invalid match limit")?;
            let max_work =
                usize::try_from(number(p, "max_work")?).map_err(|_| "invalid work limit")?;
            if max_matches == 0 || max_matches > 1000 || max_work == 0 || max_work > 1_000_000 {
                return Err("search requires 1..1000 matches and 1..1000000 work budget".into());
            }
            let mut search = SearchRequest::literal(string(p, "literal")?);
            search.max_matches = max_matches;
            search.max_work = max_work;
            search.documents =
                serde_json::from_value(p.get("documents").cloned().unwrap_or(Value::Null))
                    .map_err(|e| e.to_string())?;
            let result = controller
                .index()
                .search_literal(&snapshot, &search, || false)
                .map_err(|e| e.to_string())?;
            let termination = match result.termination {
                SearchTermination::Complete => "complete",
                SearchTermination::MatchLimit => "match_limit",
                SearchTermination::WorkLimit => "work_limit",
                SearchTermination::Cancelled => "cancelled",
            };
            Ok(
                json!({"source_versions":snapshot.documents,"matches":result.matches.iter().map(source_json).collect::<Vec<_>>(),"termination":termination,"work_used":result.work_used}),
            )
        }
        "complete" | "navigate" => {
            let snapshot = controller.index().snapshot();
            if p["source_versions"] != json!(snapshot.documents) {
                return Err("source versions changed; refresh snapshot before querying".into());
            }
            if request["type"] == "complete" {
                let category = match string(p, "category")? {
                    "label" => Category::Label,
                    "citation" => Category::Citation,
                    "command" => Category::Command,
                    _ => return Err("unknown completion category".into()),
                };
                let limit =
                    usize::try_from(number(p, "limit")?).map_err(|_| "invalid completion limit")?;
                if limit == 0 || limit > 100 {
                    return Err("completion limit must be 1..100".into());
                }
                let results = controller
                    .index()
                    .complete(&snapshot, category, string(p, "prefix")?, limit)
                    .map_err(|e| e.to_string())?;
                let results: Vec<Value> = results.into_iter().map(|item|json!({"name":item.name,"definitions":item.definitions.iter().take(100).map(source_json).collect::<Vec<_>>(),"occurrences":item.occurrences.iter().take(100).map(source_json).collect::<Vec<_>>(),"locations_truncated":item.definitions.len()>100 || item.occurrences.len()>100})).collect();
                Ok(json!({"source_versions":snapshot.documents,"completions":results}))
            } else {
                let offset =
                    usize::try_from(number(p, "byte_offset")?).map_err(|_| "invalid offset")?;
                let navigation = controller
                    .index()
                    .navigate(&snapshot, string(p, "path")?, offset)
                    .map_err(|e| e.to_string())?;
                Ok(
                    json!({"source_versions":snapshot.documents,"navigation":navigation.map(|item|json!({"origin":source_json(&item.origin.source),"name":item.origin.name,"definitions":item.definitions.iter().take(100).map(|symbol|source_json(&symbol.source)).collect::<Vec<_>>(),"definitions_truncated":item.definitions.len()>100}))}),
                )
            }
        }

        "edit" => {
            let result = controller.replace_document(
                string(p, "path")?,
                number(p, "expected_revision")?,
                string(p, "expected_sha256")?,
                string(p, "text")?.to_owned(),
            )?;
            Ok(
                json!({"document":result.document,"preview_error":result.preview_error,"save_and_submit_ms":result.save_and_submit_ms}),
            )
        }
        "project_status" => {
            let snapshot = controller.index().snapshot();
            let max = match p.get("max_documents") {
                None => 256,
                Some(value) => value
                    .as_u64()
                    .filter(|n| (1..=256).contains(n))
                    .ok_or("max_documents must be 1..256")? as usize,
            };
            let documents = snapshot.documents.keys().take(max).map(|path| {
                let document = controller.document(path)?;
                Ok(json!({"path":path,"revision":document.revision,"sha256":document.source_sha256,"bytes":document.text.len()}))
            }).collect::<Result<Vec<Value>, String>>()?;
            Ok(
                json!({"project_id":snapshot.project_id,"source_versions":snapshot.documents,
                "membership_generation":snapshot.generation,"documents":documents,
                "total_documents":snapshot.documents.len(),"truncated":snapshot.documents.len()>max,
                "scope":"active_sources_only","disk_tree_enumerated":false}),
            )
        }
        "open_document" | "detach_document" => {
            let expected = controller.index().snapshot();
            if p["source_versions"] != json!(expected.documents)
                || p["membership_generation"].as_u64() != Some(expected.generation)
            {
                return Err("project membership snapshot is stale".into());
            }
            let path = string(p, "path")?;
            let (document, preview_error) = if request["type"] == "open_document" {
                let result = file_project
                    .ok_or("helper was not opened from a file project")?
                    .open_document(controller, &expected, path)?;
                (Some(result.document), result.preview_error)
            } else {
                (None, controller.detach_document(&expected, path)?)
            };
            let current = controller.index().snapshot();
            Ok(
                json!({"document":document,"preview_error":preview_error,"source_versions":current.documents,"membership_generation":current.generation}),
            )
        }
        "file_status" => {
            let files = file_project.ok_or("helper was not opened from a file project")?;
            let state = match files.inspect(controller, string(p, "path")?)? {
                DiskState::MatchesSource { sha256 } => {
                    json!({"state":"matches_source","sha256":sha256})
                }
                DiskState::DiffersFromSource {
                    disk_sha256,
                    source_sha256,
                } => {
                    json!({"state":"differs_from_source","disk_sha256":disk_sha256,"source_sha256":source_sha256})
                }
                DiskState::Missing => json!({"state":"missing"}),
                DiskState::Unavailable { reason } => json!({"state":"unavailable","reason":reason}),
            };
            Ok(
                json!({"path":string(p,"path")?,"disk":state,"discovery_diagnostics":files.diagnostics(),"export_available":true}),
            )
        }
        "reload" => {
            if p["user_approved"] != true {
                return Err("explicit reload approval required".into());
            }
            let result = file_project
                .ok_or("helper was not opened from a file project")?
                .reload_explicitly(
                    controller,
                    string(p, "path")?,
                    p["expected_revision"]
                        .as_u64()
                        .ok_or("expected_revision required")?,
                    string(p, "expected_sha256")?,
                    string(p, "expected_disk_sha256")?,
                )?;
            Ok(
                json!({"document":result.document,"preview_error":result.preview_error,"save_and_submit_ms":result.save_and_submit_ms}),
            )
        }
        "export" => {
            let receipt = file_project
                .ok_or("helper was not opened from a file project")?
                .export(
                    controller,
                    string(p, "path")?,
                    p["expected_revision"]
                        .as_u64()
                        .ok_or("expected_revision required")?,
                    string(p, "expected_sha256")?,
                    match p.get("expected_disk_sha256") {
                        Some(Value::Null) => None,
                        Some(Value::String(hash)) => Some(hash.as_str()),
                        _ => {
                            return Err("expected_disk_sha256 must be explicit null or hash".into())
                        }
                    },
                )?;
            Ok(
                json!({"exported":true,"path":receipt.path.as_str(),"sha256":receipt.sha256_hex(),"bytes":receipt.bytes}),
            )
        }
        "history_status" => Ok(json!({"history":controller.history_status(string(p,"path")?)?})),
        "apply_group" | "undo" | "redo" => {
            let command = p["command"].clone();
            let action = match request["type"].as_str().unwrap() {
                "apply_group" => HistoryAction::Group(
                    serde_json::from_value(command).map_err(|e| e.to_string())?,
                ),
                "undo" => {
                    HistoryAction::Undo(serde_json::from_value(command).map_err(|e| e.to_string())?)
                }
                _ => {
                    HistoryAction::Redo(serde_json::from_value(command).map_err(|e| e.to_string())?)
                }
            };
            let outcome = controller.apply_history(string(p, "path")?, action)?;
            Ok(
                json!({"history":outcome.history,"preview_error":outcome.source.preview_error,"save_and_submit_ms":outcome.source.save_and_submit_ms}),
            )
        }
        "configure_layout" => {
            if p["renderer_support_confirmed"] != true {
                return Err("explicit native renderer support confirmation required".into());
            }
            let capabilities: Vec<String> =
                serde_json::from_value(p["layout_capabilities"].clone())
                    .map_err(|e| e.to_string())?;
            controller.configure_layout(capabilities)?;
            Ok(json!({"submitted":true}))
        }
        "compile" => {
            controller.compile_current()?;
            Ok(json!({"submitted":true}))
        }
        "restart" => {
            controller.restart(
                Command::new(compiler.ok_or("compiler not configured")?),
                limits.clone(),
            )?;
            Ok(json!({"submitted":true}))
        }
        "close" => {
            controller.close()?;
            reviews.clear();
            Ok(json!({"closed":true}))
        }
        "review" => {
            if reviews.len() >= 128 {
                return Err("review capacity reached; retire a review first".into());
            }
            let edit: PreparedEdit =
                serde_json::from_value(p["edit"].clone()).map_err(|e| e.to_string())?;
            let current = controller.document(&edit.path)?;
            if current.project_id != edit.project_id
                || current.revision != edit.expected_revision
                || current.source_sha256 != edit.document_before_sha256
                || current.text.get(edit.start_byte..edit.end_byte)
                    != Some(edit.removed_text.as_str())
            {
                return Err("review differs from current source".into());
            }
            let mut random = [0u8; 32];
            getrandom::fill(&mut random).map_err(|e| e.to_string())?;
            let token = random
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            reviews.insert(token.clone(), edit.clone());
            Ok(json!({"approval_token":token,"edit":edit,"requires_explicit_user_approval":true}))
        }
        "apply_reviewed" => {
            if p["user_approved"] != true {
                return Err("explicit user approval required".into());
            }
            let edit = reviews
                .get(string(p, "approval_token")?)
                .ok_or("unknown review token")?
                .clone();
            let result =
                controller.apply_reviewed(ApprovedEdit::from_explicit_user_approval(edit))?;
            Ok(
                json!({"receipt":result.receipt,"document":result.source.document,"preview_error":result.source.preview_error}),
            )
        }
        "retire_review" => {
            reviews.remove(string(p, "approval_token")?);
            Ok(json!({"retired":true}))
        }
        "recovery" => Ok(json!({"transactions":controller.recovery(string(p,"path")?)?})),
        "confirm_receipt" => {
            let receipt: AppliedReceipt =
                serde_json::from_value(p["receipt"].clone()).map_err(|e| e.to_string())?;
            controller.confirm_receipt(string(p, "path")?, &receipt)?;
            Ok(json!({"confirmed":true}))
        }
        _ => Err("unknown operation".into()),
    }
}
fn main() {
    let result = std::env::args_os()
        .nth(1)
        .ok_or("usage: flashtex-preview-controller CONFIG.json".to_owned())
        .and_then(|path| {
            let file = File::open(path).map_err(|e| e.to_string())?;
            let mut bytes = Vec::new();
            BufReader::new(file)
                .take(MAX_FRAME as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() > MAX_FRAME {
                return Err("oversized configuration".into());
            }
            run(serde_json::from_slice(&bytes).map_err(|e| e.to_string())?)
        });
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod configuration_tests {
    use super::*;
    #[test]
    fn compiler_frame_configuration_preserves_helper_headroom() {
        assert_eq!(
            compiler_limits(&json!({})).unwrap().max_frame,
            8 * 1024 * 1024
        );
        assert_eq!(
            compiler_limits(&json!({"compiler_max_frame_bytes":MAX_COMPILER_FRAME}))
                .unwrap()
                .max_frame,
            MAX_COMPILER_FRAME
        );
        for value in [
            json!(0),
            json!(-1),
            json!(1.5),
            json!("large"),
            Value::Null,
            json!(MAX_COMPILER_FRAME + 1),
        ] {
            assert!(compiler_limits(&json!({"compiler_max_frame_bytes":value})).is_err());
        }
    }
}
