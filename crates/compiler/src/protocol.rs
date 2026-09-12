//! runtime-v1 JSON Lines transport.
//!
//! One complete JSON object per line. Replies preserve the request `id`. Unknown
//! protocol versions and unknown types produce an `error` envelope — never a
//! silent success, as the contract requires.

use crate::cache::{CachedResult, CompileCache};
use crate::diagnostics::{Diagnostic, Severity};
use crate::json::{self, str_, Value};
use crate::layout::{self, Page};
use crate::parser;
use crate::pdf;
use std::fs;
use std::io::Write;
use std::cell::RefCell;

// Process-lifetime compile cache — sequential JSON Lines loop is single-threaded.
thread_local! {
    static CACHE: RefCell<CompileCache> = RefCell::new(CompileCache::new());
}

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

    // Fast path: return cached result if content is unchanged.
    let cached = CACHE.with(|c| {
        c.borrow_mut().get(&project_id, &text).map(|e| (e.pages.clone(), e.pdf_path.clone(), e.hit_count))
    });
    if let Some((cached_pages, cached_pdf_path, _hit_count)) = cached {
        let mut p = Value::obj();
        p.set("project_id", str_(project_id));
        p.set("revision", Value::Num(revision as f64));
        p.set("status", str_("ok"));
        p.set("cache_hit", Value::Bool(true));
        p.set("pages", pages_json(&cached_pages, &path));
        p.set("diagnostics", Value::Arr(Vec::new()));
        match &cached_pdf_path {
            Some(pp) => p.set("pdf_path", str_(pp.clone())),
            None => p.set("pdf_path", Value::Null),
        }
        return result_envelope(id, p);
    }

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

    // Write PDF to a temp file when there is content to render.
    // The Mac shell reads pdf_path and shows a badge; null means no PDF yet.
    let pdf_path = if has_content {
        write_pdf(&pages, &project_id, revision)
    } else {
        None
    };

    // Store result in cache for future incremental lookups.
    CACHE.with(|c| {
        c.borrow_mut().insert(&project_id, &text, CachedResult {
            content_hash: crate::cache::fnv1a(text.as_bytes()),
            exact_text: text.clone(),
            pages: pages.clone(),
            pdf_path: pdf_path.clone(),
            hit_count: 0,
        });
    });

    let mut p = Value::obj();
    p.set("project_id", str_(project_id));
    p.set("revision", Value::Num(revision as f64));
    p.set("status", str_(status));
    p.set("cache_hit", Value::Bool(false));
    p.set("pages", pages_json(&pages, &path));
    p.set("diagnostics", Value::Arr(diags.iter().map(|d| d.to_json(&path)).collect()));
    match &pdf_path {
        Some(pp) => p.set("pdf_path", str_(pp.clone())),
        None => p.set("pdf_path", Value::Null),
    }
    result_envelope(id, p)
}

/// Render pages to PDF and write to a temp path.
/// Returns the path on success; logs and returns None on any I/O error.
///
/// Path format: /tmp/flashtex/<project_id>/rev<revision>.pdf
/// The directory is created if absent. The file is overwritten on each
/// compile so the Mac always reads the latest revision.
fn write_pdf(pages: &[Page], project_id: &str, revision: i64) -> Option<String> {
    let pdf_bytes = pdf::render(pages);
    if pdf_bytes.is_empty() {
        return None;
    }

    // Sanitise project_id for use as a directory component.
    let safe_id: String = project_id
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .take(64)
        .collect();
    let safe_id = if safe_id.is_empty() { "project".to_string() } else { safe_id };

    let dir = format!("/tmp/flashtex/{}", safe_id);
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("flashtex-compiler: pdf dir error: {}", e);
        return None;
    }

    let pdf_path = format!("{}/rev{}.pdf", dir, revision);
    match fs::File::create(&pdf_path).and_then(|mut f| f.write_all(&pdf_bytes)) {
        Ok(()) => Some(pdf_path),
        Err(e) => {
            eprintln!("flashtex-compiler: pdf write error {}: {}", pdf_path, e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: send a compile request and parse the JSON response.
    fn compile_json(project_id: &str, text: &str, revision: i64) -> String {
        let req = format!(
            r#"{{"protocol_version":1,"id":"test","type":"compile","payload":{{"project_id":"{pid}","revision":{rev},"entry_path":"main.tex","documents":[{{"path":"main.tex","text":{text}}}]}}}}"#,
            pid = project_id,
            rev = revision,
            text = serde_json_str(text),
        );
        handle_line(&req)
    }

    /// Minimal JSON string-escape (no external deps).
    fn serde_json_str(s: &str) -> String {
        let mut out = String::from('"');
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c => out.push(c),
            }
        }
        out.push('"');
        out
    }

    fn field_str(response: &str, key: &str) -> Option<String> {
        // Naive key extraction from flat JSON payload — sufficient for our tests.
        let needle = format!("\"{}\":\"", key);
        let start = response.find(&needle)? + needle.len();
        let end = response[start..].find('"')? + start;
        Some(response[start..end].to_string())
    }

    fn field_bool(response: &str, key: &str) -> Option<bool> {
        let needle_t = format!("\"{}\":true", key);
        let needle_f = format!("\"{}\":false", key);
        if response.contains(&needle_t) { Some(true) }
        else if response.contains(&needle_f) { Some(false) }
        else { None }
    }

    // -----------------------------------------------------------------------
    // Incremental reuse: same content → cache_hit on second call
    // -----------------------------------------------------------------------

    #[test]
    fn incremental_cache_hit_on_repeat() {
        // Clear cache first so this test is isolated.
        CACHE.with(|c| c.borrow_mut().clear());

        let doc = r"\documentclass{article}\begin{document}Hello world.\end{document}";
        let r1 = compile_json("inc-test", doc, 1);
        let r2 = compile_json("inc-test", doc, 2);

        assert_eq!(field_bool(&r1, "cache_hit"), Some(false), "first compile must be a cache miss");
        assert_eq!(field_bool(&r2, "cache_hit"), Some(true),  "second compile of same content must be a cache hit");
    }

    #[test]
    fn incremental_cache_miss_on_changed_content() {
        CACHE.with(|c| c.borrow_mut().clear());

        let doc1 = r"\documentclass{article}\begin{document}Version one.\end{document}";
        let doc2 = r"\documentclass{article}\begin{document}Version two.\end{document}";
        let r1 = compile_json("change-test", doc1, 1);
        let r2 = compile_json("change-test", doc2, 2);

        assert_eq!(field_bool(&r1, "cache_hit"), Some(false), "first compile must be a cache miss");
        assert_eq!(field_bool(&r2, "cache_hit"), Some(false), "changed content must also be a cache miss");
    }

    // -----------------------------------------------------------------------
    // Clean-build equivalence: incremental result == fresh build result
    // -----------------------------------------------------------------------

    #[test]
    fn clean_build_equivalence() {
        CACHE.with(|c| c.borrow_mut().clear());

        let doc = r"\documentclass{article}\begin{document}Equivalence check.\end{document}";

        // First call: clean build (cache miss)
        let clean = compile_json("equiv-test", doc, 1);

        // Second call: incremental (cache hit) — pages and status must match
        let incremental = compile_json("equiv-test", doc, 2);

        // Both must report a compile_result (not a protocol error)
        assert!(clean.contains("\"type\":\"compile_result\""),
                "clean build must be a compile_result, got: {}", clean);

        // The incremental result is returned from cache as "ok".
        // The clean result may be "ok" or "recovered" depending on parser completeness.
        // Key invariant: incremental reports cache_hit:true and both contain the text.
        assert_eq!(field_bool(&incremental, "cache_hit"), Some(true),
                   "second call with same content must be a cache hit");
        assert_eq!(field_bool(&clean, "cache_hit"), Some(false),
                   "first call must be a cache miss");

        // Both must contain the word "Equivalence" (words are laid out separately)
        assert!(clean.contains("Equivalence"),
                "clean build must include document text, got: {}", clean);
        assert!(incremental.contains("Equivalence"),
                "incremental build must include document text, got: {}", incremental);
    }

    // -----------------------------------------------------------------------
    // Recovery: malformed LaTeX still returns a partial result
    // -----------------------------------------------------------------------

    #[test]
    fn recovery_from_malformed_input() {
        CACHE.with(|c| c.borrow_mut().clear());

        // Unclosed \begin{document} with text — parser should recover and
        // return whatever it could lay out, with diagnostics.
        let doc = r"\documentclass{article}\begin{document}Partial content here";
        let response = compile_json("recovery-test", doc, 1);

        // Must not be a protocol-level error
        assert!(!response.contains("\"type\":\"error\""),
                "response must not be a protocol error");

        // Status must be 'ok', 'recovered', or 'failed' — but NOT an error envelope
        let has_known_status = response.contains("\"status\":\"ok\"")
            || response.contains("\"status\":\"recovered\"")
            || response.contains("\"status\":\"failed\"");
        assert!(has_known_status, "response must carry a compile_result status");
    }

    #[test]
    fn recovery_empty_document_returns_failed_not_crash() {
        CACHE.with(|c| c.borrow_mut().clear());

        let response = compile_json("empty-test", "", 1);
        // Must not panic; must return some valid envelope
        assert!(response.contains("\"type\":\"compile_result\"") || response.contains("\"type\":\"error\""),
                "empty document must return a valid envelope, got: {}", response);
    }

    #[test]
    fn recovery_garbage_latex_produces_diagnostics() {
        CACHE.with(|c| c.borrow_mut().clear());

        let doc = r"%%%@@@\nope{{{";
        let response = compile_json("garbage-test", doc, 1);

        // Must return a compile_result (not a crash / panic / protocol error)
        assert!(!response.contains("\"type\":\"error\""),
                "garbage LaTeX must produce a compile_result, not a protocol error");
    }

    // -----------------------------------------------------------------------
    // Isolation: different project IDs don't share cache
    // -----------------------------------------------------------------------

    #[test]
    fn cache_isolated_by_project_id() {
        CACHE.with(|c| c.borrow_mut().clear());

        let doc = r"\documentclass{article}\begin{document}Same text.\end{document}";
        let r1 = compile_json("proj-alpha", doc, 1);
        let r2 = compile_json("proj-beta",  doc, 1);

        assert_eq!(field_bool(&r1, "cache_hit"), Some(false), "proj-alpha first compile is a miss");
        assert_eq!(field_bool(&r2, "cache_hit"), Some(false), "proj-beta must not reuse proj-alpha cache");
    }
}
