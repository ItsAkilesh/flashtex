//! runtime-v1 JSON Lines transport.
//!
//! One complete JSON object per line. Replies preserve the request `id`. Unknown
//! protocol versions and unknown types produce an `error` envelope — never a
//! silent success, as the contract requires.

use crate::diagnostics::{Diagnostic, Severity};
use crate::json::{self, str_, Value};
use crate::layout::{self, Page};
use crate::parser;

pub const PROTOCOL_VERSION: i64 = 1;
/// Documented maximum accepted line size. Oversized payloads are rejected.
pub const MAX_LINE_BYTES: usize = 8 * 1024 * 1024;

pub fn error_envelope(id: &str, code: &str, message: &str) -> Value {
    let mut payload = Value::obj();
    payload.set("code", str_(code));
    payload.set("message", str_(message));
    let mut v = Value::obj();
    v.set("protocol_version", Value::Num(PROTOCOL_VERSION as f64));
    v.set("id", str_(id));
    v.set("type", str_("error"));
    v.set("payload", payload);
    v
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

    let id = value.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    match value.get("protocol_version").and_then(|v| v.as_i64()) {
        Some(PROTOCOL_VERSION) => {}
        Some(other) => {
            return json::write(&error_envelope(
                &id,
                "unsupported_protocol_version",
                &format!("protocol version {} is not supported; this build speaks version {}", other, PROTOCOL_VERSION),
            ));
        }
        None => {
            return json::write(&error_envelope(&id, "missing_protocol_version", "protocol_version is required"));
        }
    }

    match value.get("type").and_then(|v| v.as_str()) {
        Some("compile") => match value.get("payload") {
            Some(p) => json::write(&compile(&id, p)),
            None => json::write(&error_envelope(&id, "missing_payload", "compile requires a payload")),
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

fn pages_json(pages: &[Page], path: &str) -> Value {
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
                                src.set("path", str_(path));
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
    payload.set("diagnostics", Value::Arr(diags.iter().map(|d| d.to_json(path)).collect()));
    payload.set("pdf_path", Value::Null);
    result_envelope(id, payload)
}

fn compile(id: &str, payload: &Value) -> Value {
    let project_id = payload.get("project_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let revision = payload.get("revision").and_then(|v| v.as_i64()).unwrap_or(0);
    let entry = payload.get("entry_path").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let empty = Vec::new();
    let docs = payload.get("documents").and_then(|v| v.as_arr()).unwrap_or(&empty);

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
            message: format!("rejected entry_path '{}': paths must be project-relative with no parent traversal", entry),
            span: None,
            recovery: None,
        };
        return failed(id, &project_id, revision, vec![diag], &entry);
    }

    // This version compiles the entry document only. Multi-file projects are an
    // outstanding requirement, reported rather than silently ignored.
    let entry_doc = docs
        .iter()
        .find(|d| d.get("path").and_then(|v| v.as_str()) == Some(entry.as_str()))
        .or_else(|| docs.first());

    let (path, text) = match entry_doc {
        Some(d) => (
            d.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            d.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        ),
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

    let parsed = parser::parse(&text);
    let pages = layout::layout(&parsed.blocks);

    let mut diags = parsed.diagnostics;
    if docs.len() > 1 {
        diags.push(Diagnostic::warning(
            format!("{} documents were supplied; this version compiles only the entry document", docs.len()),
            None,
            Some("compiled the entry document alone".into()),
        ));
    }

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
    p.set("pages", pages_json(&pages, &path));
    p.set("diagnostics", Value::Arr(diags.iter().map(|d| d.to_json(&path)).collect()));
    p.set("pdf_path", Value::Null);
    result_envelope(id, p)
}
