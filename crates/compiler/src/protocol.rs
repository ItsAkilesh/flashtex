//! runtime-v1 JSON Lines transport.
//!
//! One complete JSON object per line. Replies preserve the request `id`. Unknown
//! protocol versions and unknown types produce an `error` envelope — never a
//! silent success, as the contract requires.

use crate::diagnostics::{Diagnostic, Severity};
use crate::incremental::Session;
use crate::json::{self, str_, Value};
use crate::layout::{LayoutConstraints, Page};
use crate::parser::SourceDocument;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

pub const PROTOCOL_VERSION: i64 = 1;
/// Documented maximum accepted line size. Oversized payloads are rejected.
pub const MAX_LINE_BYTES: usize = 8 * 1024 * 1024;

/// Most documents kept warm at once. A worker can be asked to compile any number
/// of projects over its lifetime, so the cache is bounded and evicts the
/// least-recently-used document rather than retaining every text it has ever seen.
const MAX_WARM_SESSIONS: usize = 8;

type WarmSessions = HashMap<(String, String), (u64, Session)>;
static SESSIONS: OnceLock<Mutex<WarmSessions>> = OnceLock::new();
static SESSION_TICK: AtomicU64 = AtomicU64::new(0);

/// One request read without allowing an untrusted line to grow memory without
/// bound. `TooLarge` is returned only after the complete offending line has
/// been consumed, so the caller can safely continue with the next request.
#[derive(Debug, PartialEq, Eq)]
pub enum RequestLine {
    Data(Vec<u8>),
    TooLarge,
}

pub fn read_request_line<R: BufRead>(reader: &mut R) -> io::Result<Option<RequestLine>> {
    let mut line = Vec::new();
    let mut oversized = false;
    let mut saw_input = false;

    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            if !saw_input {
                return Ok(None);
            }
            return Ok(Some(if oversized {
                RequestLine::TooLarge
            } else {
                RequestLine::Data(line)
            }));
        }

        saw_input = true;
        let newline = available.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map_or(available.len(), |index| index + 1);
        let content = newline.map_or(available, |index| &available[..index]);

        if !oversized {
            let remaining = MAX_LINE_BYTES.saturating_sub(line.len());
            if content.len() > remaining {
                oversized = true;
                line.clear();
            } else {
                line.extend_from_slice(content);
            }
        }

        reader.consume(consumed);
        if newline.is_some() {
            return Ok(Some(if oversized {
                RequestLine::TooLarge
            } else {
                RequestLine::Data(line)
            }));
        }
    }
}

pub fn error_envelope(id: &str, code: &str, message: &str) -> Value {
    let mut payload = Value::obj();
    payload.set("code", str_(code));
    payload.set("message", str_(message));
    payload.set(
        "diagnostics",
        Value::Arr(vec![Diagnostic::error(message, None, None).to_json("")]),
    );
    let mut v = Value::obj();
    v.set("protocol_version", Value::Num(PROTOCOL_VERSION as f64));
    v.set("id", str_(id));
    v.set("type", str_("error"));
    v.set("payload", payload);
    v
}

/// Handles one already-delimited request line, including transport bytes that
/// cannot be represented as UTF-8. The worker and tests share this reply path.
pub fn handle_request_bytes(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(line) => handle_line(line),
        Err(_) => json::write(&error_envelope(
            "",
            "invalid_utf8",
            "request line is not valid UTF-8",
        )),
    }
}

/// Project-relative paths only: no absolute paths, no parent traversal.
fn path_is_safe(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || path.starts_with('\\') {
        return false;
    }
    // Reject Windows-style drive prefixes too.
    if path.len() >= 2 && path.as_bytes()[1] == b':' {
        return false;
    }
    !path.split(['/', '\\']).any(|c| c == "..")
}

/// Handles one input line and returns the reply line to write.
pub fn handle_line(line: &str) -> String {
    if line.len() > MAX_LINE_BYTES {
        return json::write(&error_envelope(
            "",
            "payload_too_large",
            &format!("line exceeds the {}-byte limit", MAX_LINE_BYTES),
        ));
    }

    let value = match json::parse(line) {
        Ok(v) => v,
        Err(e) => {
            return json::write(&error_envelope("", "malformed_json", &e.0));
        }
    };

    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    match value.get("protocol_version").and_then(|v| v.as_i64()) {
        Some(PROTOCOL_VERSION) => {}
        Some(other) => {
            return json::write(&error_envelope(
                &id,
                "unsupported_protocol_version",
                &format!(
                    "protocol version {} is not supported; this build speaks version {}",
                    other, PROTOCOL_VERSION
                ),
            ));
        }
        None => {
            return json::write(&error_envelope(
                &id,
                "missing_protocol_version",
                "protocol_version is required",
            ));
        }
    }

    match value.get("type").and_then(|v| v.as_str()) {
        Some("compile") => match value.get("payload") {
            Some(p) => json::write(&compile(&id, p)),
            None => json::write(&error_envelope(
                &id,
                "missing_payload",
                "compile requires a payload",
            )),
        },
        Some(other) => json::write(&error_envelope(
            &id,
            "unsupported_type",
            &format!("message type '{}' is not supported", other),
        )),
        None => json::write(&error_envelope(&id, "missing_type", "type is required")),
    }
}

fn result_envelope(id: &str, payload: Value) -> Value {
    let mut v = Value::obj();
    v.set("protocol_version", Value::Num(PROTOCOL_VERSION as f64));
    v.set("id", str_(id));
    v.set("type", str_("compile_result"));
    v.set("payload", payload);
    v
}

/// `paths` is indexed by `DocumentId`. Each item reports the file its bytes
/// actually live in, so click-to-source navigation opens the right file in a
/// multi-file project instead of always pointing at the entry document.
fn pages_json(pages: &[Page], paths: &[&str]) -> Value {
    Value::Arr(
        pages
            .iter()
            .map(|pg| {
                let mut p = Value::obj();
                p.set("number", Value::Num(pg.number as f64));
                p.set("width_pt", Value::Num(pg.width_pt));
                p.set("height_pt", Value::Num(pg.height_pt));
                p.set(
                    "items",
                    Value::Arr(
                        pg.items
                            .iter()
                            .map(|it| {
                                let mut src = Value::obj();
                                src.set(
                                    "path",
                                    str_(paths.get(it.span.document.0).copied().unwrap_or("")),
                                );
                                src.set("start_byte", Value::Num(it.span.start as f64));
                                src.set("end_byte", Value::Num(it.span.end as f64));
                                let mut i = Value::obj();
                                i.set("kind", str_("text"));
                                i.set("text", str_(it.text.clone()));
                                i.set("x_pt", Value::Num(it.x_pt));
                                i.set("baseline_y_pt", Value::Num(it.baseline_y_pt));
                                i.set("font_size_pt", Value::Num(it.font_size_pt));
                                i.set("source", src);
                                i
                            })
                            .collect(),
                    ),
                );
                p
            })
            .collect(),
    )
}

fn failed(id: &str, project_id: &str, revision: i64, diags: Vec<Diagnostic>, path: &str) -> Value {
    let mut payload = Value::obj();
    payload.set("project_id", str_(project_id));
    payload.set("revision", Value::Num(revision as f64));
    payload.set("status", str_("failed"));
    payload.set("pages", Value::Arr(Vec::new()));
    payload.set(
        "diagnostics",
        Value::Arr(diags.iter().map(|d| d.to_json(path)).collect()),
    );
    payload.set("pdf_path", Value::Null);
    result_envelope(id, payload)
}

fn compile(id: &str, payload: &Value) -> Value {
    let project_id = payload
        .get("project_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let revision = payload
        .get("revision")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let entry = payload
        .get("entry_path")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let empty = Vec::new();
    let docs = payload
        .get("documents")
        .and_then(|v| v.as_arr())
        .unwrap_or(&empty);

    // Validate every supplied path before compiling anything.
    for d in docs {
        let p = d.get("path").and_then(|v| v.as_str()).unwrap_or("");
        if !path_is_safe(p) {
            let diag = Diagnostic {
                severity: Severity::Error,
                message: format!("rejected document path '{}': paths must be project-relative with no parent traversal", p),
                span: None,
                recovery: None,
            };
            return failed(id, &project_id, revision, vec![diag], p);
        }
    }
    if !entry.is_empty() && !path_is_safe(&entry) {
        let diag = Diagnostic {
            severity: Severity::Error,
            message: format!(
                "rejected entry_path '{}': paths must be project-relative with no parent traversal",
                entry
            ),
            span: None,
            recovery: None,
        };
        return failed(id, &project_id, revision, vec![diag], &entry);
    }

    // Every supplied document participates: \input resolves against this set, and
    // DocumentId indexes it, so span order here defines the identity of a span.
    let project: Vec<(String, String)> = docs
        .iter()
        .map(|d| {
            (
                d.get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                d.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            )
        })
        .collect();

    let entry_doc = docs
        .iter()
        .find(|d| d.get("path").and_then(|v| v.as_str()) == Some(entry.as_str()))
        .or_else(|| docs.first());

    let path = match entry_doc {
        Some(d) => d
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        None => {
            let diag = Diagnostic {
                severity: Severity::Error,
                message: "no documents supplied to compile".into(),
                span: None,
                recovery: None,
            };
            return failed(id, &project_id, revision, vec![diag], &entry);
        }
    };

    // Session state is internal to runtime-v1: request and response shapes stay
    // unchanged. Project plus entry path identifies a document across revisions.
    let sessions = SESSIONS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut sessions = sessions
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let tick = SESSION_TICK.fetch_add(1, Ordering::Relaxed);
    let key = (project_id.clone(), path.clone());
    if !sessions.contains_key(&key) && sessions.len() >= MAX_WARM_SESSIONS {
        // Evict the least recently used document. Dropping a session only costs
        // the next compile of that document its reuse; it never changes output,
        // because every result is equivalent to a clean build by construction.
        if let Some(oldest) = sessions
            .iter()
            .min_by_key(|(_, (used, _))| *used)
            .map(|(k, _)| k.clone())
        {
            sessions.remove(&oldest);
        }
    }
    let slot = sessions
        .entry(key)
        .or_insert_with(|| (tick, Session::new()));
    slot.0 = tick;
    let sources: Vec<SourceDocument<'_>> = project
        .iter()
        .map(|(p, t)| SourceDocument {
            path: p.as_str(),
            text: t.as_str(),
        })
        .collect();
    let incremental = slot
        .1
        .compile_project(&sources, &path, LayoutConstraints::default());
    let pages = incremental.output.pages;
    let diags = incremental.output.diagnostics;
    let paths: Vec<&str> = project.iter().map(|(p, _)| p.as_str()).collect();

    let has_content = pages.iter().any(|p| !p.items.is_empty());
    let status = if diags.is_empty() {
        "ok"
    } else if has_content {
        "recovered"
    } else {
        "failed"
    };

    let mut p = Value::obj();
    p.set("project_id", str_(project_id));
    p.set("revision", Value::Num(revision as f64));
    p.set("status", str_(status));
    p.set("pages", pages_json(&pages, &paths));
    p.set(
        "diagnostics",
        Value::Arr(diags.iter().map(|d| d.to_json_with_paths(&paths)).collect()),
    );
    p.set("pdf_path", Value::Null);
    result_envelope(id, p)
}
