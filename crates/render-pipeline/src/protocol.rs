//! runtime-v1 JSON Lines transport for `flashtex-render`: reads `compile`
//! envelopes, replies with `compile_result` (the v1 fallback), follows it
//! with the rendering-v2 `display_list` line when `display-list-v2` was
//! negotiated (`docs/contracts/runtime-v1-display-list-v2.md`), and can
//! keep the v2 display list of the last request for `--v2`/`--pdf`.
//! Unknown protocol versions, message types and unsafe paths are rejected
//! with the compiler's error envelopes — never a silent partial success.

use flashtex_compiler::json::{self, Value};
use flashtex_compiler::parser::SourceDocument;
use flashtex_compiler::protocol::{error_envelope, PROTOCOL_VERSION};

use crate::v1::Capabilities;
use crate::{render, FontSet, RenderOptions, Rendered};

pub const MAX_LINE_BYTES: usize = flashtex_compiler::protocol::MAX_LINE_BYTES;
/// Largest reply line the Mac reader accepts (`JSONLines.maxLineBytes`);
/// a larger result is failed explicitly instead of being cut off.
/// `FLASHTEX_MAX_REPLY_BYTES` lowers it (tests exercise the limit paths).
pub const MAX_REPLY_BYTES: usize = 16 * 1024 * 1024;

pub fn max_reply_bytes() -> usize {
    std::env::var("FLASHTEX_MAX_REPLY_BYTES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .map_or(MAX_REPLY_BYTES, |v| v.min(MAX_REPLY_BYTES))
}
pub const MAX_CAPABILITIES: usize = 16;
pub const MAX_CAPABILITY_BYTES: usize = 64;

/// Project-relative paths only: no absolute paths, no parent traversal.
pub fn path_is_safe(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || path.starts_with('\\') {
        return false;
    }
    if path.len() >= 2 && path.as_bytes()[1] == b':' {
        return false;
    }
    !path.split(['/', '\\']).any(|c| c == "..")
}

/// The outcome of one request line.
pub struct Reply {
    /// The JSON Lines reply to write.
    pub line: String,
    /// Lines to write right after `line`, before any later reply: the
    /// `display_list` envelope when `display-list-v2` was accepted.
    pub extra_lines: Vec<String>,
    /// The render, when the request compiled (for `--v2`/`--pdf`).
    pub rendered: Option<Rendered>,
    pub id: String,
}

fn failed(id: &str, project_id: &str, revision: i64, message: &str, accepted: Option<Vec<String>>) -> Value {
    let mut payload = Value::obj();
    payload.set("project_id", json::str_(project_id));
    payload.set("revision", json::num(revision as f64));
    payload.set("status", json::str_("failed"));
    payload.set("pages", Value::Arr(Vec::new()));
    let mut d = Value::obj();
    d.set("severity", json::str_("error"));
    d.set("message", json::str_(message));
    d.set("source", Value::Null);
    d.set("recovery", Value::Null);
    payload.set("diagnostics", Value::Arr(vec![d]));
    payload.set("pdf_path", Value::Null);
    if let Some(acc) = accepted {
        payload.set("layout_capabilities", Value::Arr(acc.into_iter().map(json::str_).collect()));
    }
    result_envelope(id, payload)
}

fn result_envelope(id: &str, payload: Value) -> Value {
    let mut v = Value::obj();
    v.set("protocol_version", json::num(PROTOCOL_VERSION as f64));
    v.set("id", json::str_(id));
    v.set("type", json::str_("compile_result"));
    v.set("payload", payload);
    v
}

/// Handles one request line.
pub fn handle_line(line: &str, fonts: &FontSet, options: &RenderOptions) -> Reply {
    let err = |id: &str, code: &str, msg: &str| Reply {
        line: json::write(&error_envelope(id, code, msg)),
        extra_lines: Vec::new(),
        rendered: None,
        id: id.to_string(),
    };
    if line.len() > MAX_LINE_BYTES {
        return err("", "payload_too_large", &format!("line exceeds the {}-byte limit", MAX_LINE_BYTES));
    }
    let value = match json::parse(line) {
        Ok(v) => v,
        Err(e) => return err("", "malformed_json", &e.0),
    };
    let id = value.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    match value.get("protocol_version").and_then(|v| v.as_i64()) {
        Some(PROTOCOL_VERSION) => {}
        Some(other) => {
            return err(
                &id,
                "unsupported_protocol_version",
                &format!("protocol version {other} is not supported; this build speaks version {PROTOCOL_VERSION}"),
            )
        }
        None => return err(&id, "missing_protocol_version", "protocol_version is required"),
    }
    match value.get("type").and_then(|v| v.as_str()) {
        Some("compile") => {}
        Some(other) => return err(&id, "unsupported_type", &format!("message type '{other}' is not supported")),
        None => return err(&id, "missing_type", "type is required"),
    }
    let Some(payload) = value.get("payload") else {
        return err(&id, "missing_payload", "compile requires a payload");
    };
    let project_id = payload.get("project_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let revision = payload.get("revision").and_then(|v| v.as_i64()).unwrap_or(0);
    let entry = payload.get("entry_path").and_then(|v| v.as_str()).unwrap_or("").to_string();

    // Capability negotiation (runtime-v1-layout-capabilities.md).
    let mut requested: Option<Vec<String>> = None;
    if let Some(caps) = payload.get("layout_capabilities") {
        let Some(arr) = caps.as_arr() else {
            return Reply {
                line: json::write(&failed(&id, &project_id, revision, "layout_capabilities must be an array of strings", None)),
                extra_lines: Vec::new(),
                rendered: None,
                id,
            };
        };
        let mut list = Vec::new();
        for c in arr {
            match c.as_str() {
                Some(s) if !s.is_empty() && s.len() <= MAX_CAPABILITY_BYTES && !list.contains(&s.to_string()) => list.push(s.to_string()),
                _ => {
                    return Reply {
                        line: json::write(&failed(
                            &id,
                            &project_id,
                            revision,
                            "layout_capabilities entries must be unique nonempty strings of at most 64 bytes",
                            None,
                        )),
                        extra_lines: Vec::new(),
                        rendered: None,
                        id,
                    };
                }
            }
        }
        if list.len() > MAX_CAPABILITIES {
            return Reply {
                line: json::write(&failed(&id, &project_id, revision, "layout_capabilities lists more than 16 entries", None)),
                extra_lines: Vec::new(),
                rendered: None,
                id,
            };
        }
        requested = Some(list);
    }
    let (caps, accepted) = match &requested {
        Some(list) => {
            let (c, a) = Capabilities::negotiate(list);
            (c, Some(a))
        }
        None => (Capabilities::default(), None),
    };

    let empty = Vec::new();
    let docs = payload.get("documents").and_then(|v| v.as_arr()).unwrap_or(&empty);
    let mut project: Vec<(String, String)> = Vec::with_capacity(docs.len());
    for d in docs {
        let p = d.get("path").and_then(|v| v.as_str()).unwrap_or("");
        if !path_is_safe(p) {
            return Reply {
                line: json::write(&failed(
                    &id,
                    &project_id,
                    revision,
                    &format!("rejected document path '{p}': paths must be project-relative with no parent traversal"),
                    accepted,
                )),
                extra_lines: Vec::new(),
                rendered: None,
                id,
            };
        }
        project.push((p.to_string(), d.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string()));
    }
    if !entry.is_empty() && !path_is_safe(&entry) {
        return Reply {
            line: json::write(&failed(
                &id,
                &project_id,
                revision,
                &format!("rejected entry_path '{entry}': paths must be project-relative with no parent traversal"),
                accepted,
            )),
            extra_lines: Vec::new(),
            rendered: None,
            id,
        };
    }
    if project.is_empty() {
        return Reply {
            line: json::write(&failed(&id, &project_id, revision, "no documents supplied to compile", accepted)),
            extra_lines: Vec::new(),
            rendered: None,
            id,
        };
    }
    let entry_path = if project.iter().any(|(p, _)| *p == entry) {
        entry
    } else {
        project[0].0.clone()
    };
    let sources: Vec<SourceDocument<'_>> = project
        .iter()
        .map(|(p, t)| SourceDocument {
            path: p.as_str(),
            text: t.as_str(),
        })
        .collect();
    let rendered = render(&sources, &entry_path, revision.max(0) as u64, &project_id, fonts, options);
    let limit = max_reply_bytes();
    let mut v1 = crate::v1::fallback(&rendered.v2, caps, accepted.clone());
    // display-list-v2: the envelope is serialised first because declining it
    // (over the line limit) changes the echoed capabilities and diagnostics
    // of the compile_result that precedes it.
    let mut extra_lines = Vec::new();
    if caps.display_list && v1.status != "failed" {
        let dl = json::write(&rendered.v2.to_json(&id));
        if dl.len() > limit {
            v1.accepted = v1.accepted.map(|a| a.into_iter().filter(|c| c != crate::v1::CAP_DISPLAY_LIST).collect());
            v1.diagnostics.push(crate::display::Diagnostic::warning(
                "display_list_declined",
                format!(
                    "display-list-v2 declined: the display_list line would be {} bytes for {} pages, over the {limit}-byte line limit",
                    dl.len(),
                    rendered.v2.pages.len()
                ),
                Vec::new(),
            ));
            if v1.status == "ok" {
                v1.status = "recovered";
            }
        } else {
            extra_lines.push(dl);
        }
    }
    let accepted = v1.accepted.clone();
    let line = v1.write_envelope(&id);
    if line.len() > limit {
        let pages = rendered.v2.pages.len();
        return Reply {
            line: json::write(&failed(
                &id,
                &project_id,
                revision,
                &format!(
                    "compile_result would be {} bytes for {pages} pages, over the {limit}-byte reply limit; split the project or compile fewer pages",
                    line.len(),
                ),
                accepted.map(|a| a.into_iter().filter(|c| c != crate::v1::CAP_DISPLAY_LIST).collect()),
            )),
            extra_lines: Vec::new(),
            rendered: Some(rendered),
            id,
        };
    }
    Reply {
        line,
        extra_lines,
        rendered: Some(rendered),
        id,
    }
}
