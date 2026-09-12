//! Diagnostics in the shape `docs/contracts/runtime-v1.md` specifies.

use crate::json::{str_, Value};
use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    /// `None` where the compiler genuinely cannot map the problem to source.
    pub span: Option<Span>,
    /// What was rendered provisionally instead, or `None` if nothing was recovered.
    pub recovery: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Option<Span>, recovery: Option<String>) -> Self {
        Diagnostic { severity: Severity::Error, message: message.into(), span, recovery }
    }

    pub fn warning(message: impl Into<String>, span: Option<Span>, recovery: Option<String>) -> Self {
        Diagnostic { severity: Severity::Warning, message: message.into(), span, recovery }
    }

    pub fn to_json(&self, path: &str) -> Value {
        let mut v = Value::obj();
        v.set("severity", str_(self.severity.as_str()));
        v.set("message", str_(self.message.clone()));
        v.set(
            "source",
            match self.span {
                Some(s) => {
                    let mut src = Value::obj();
                    src.set("path", str_(path));
                    src.set("start_byte", Value::Num(s.start as f64));
                    src.set("end_byte", Value::Num(s.end as f64));
                    src
                }
                None => Value::Null,
            },
        );
        v.set(
            "recovery",
            match &self.recovery {
                Some(r) => str_(r.clone()),
                None => Value::Null,
            },
        );
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::write;

    fn error_with_span() -> Diagnostic {
        Diagnostic::error(
            "test error message",
            Some(Span::new(3, 10)),
            Some("recovered fine".into()),
        )
    }

    fn warning_no_span() -> Diagnostic {
        Diagnostic::warning("missing closing brace", None, None)
    }

    // ── Severity ──────────────────────────────────────────────────────────────

    #[test]
    fn severity_error_as_str() {
        assert_eq!(Severity::Error.as_str(), "error");
    }

    #[test]
    fn severity_warning_as_str() {
        assert_eq!(Severity::Warning.as_str(), "warning");
    }

    // ── to_json: shape ────────────────────────────────────────────────────────

    #[test]
    fn to_json_has_required_top_level_keys() {
        let d = error_with_span();
        let v = d.to_json("doc.tex");
        assert!(v.get("severity").is_some(), "must have 'severity'");
        assert!(v.get("message").is_some(), "must have 'message'");
        assert!(v.get("source").is_some(), "must have 'source'");
        assert!(v.get("recovery").is_some(), "must have 'recovery'");
    }

    #[test]
    fn to_json_severity_string_error() {
        let v = error_with_span().to_json("x.tex");
        assert_eq!(v.get("severity").and_then(|s| s.as_str()), Some("error"));
    }

    #[test]
    fn to_json_severity_string_warning() {
        let v = warning_no_span().to_json("x.tex");
        assert_eq!(v.get("severity").and_then(|s| s.as_str()), Some("warning"));
    }

    #[test]
    fn to_json_message_round_trips() {
        let d = Diagnostic::error("unmatched '{'", None, None);
        let v = d.to_json("f.tex");
        assert_eq!(v.get("message").and_then(|s| s.as_str()), Some("unmatched '{'"));
    }

    // ── to_json: source span ──────────────────────────────────────────────────

    #[test]
    fn to_json_source_is_null_when_no_span() {
        let v = warning_no_span().to_json("f.tex");
        assert_eq!(v.get("source"), Some(&crate::json::Value::Null),
            "source must be null when no span is provided");
    }

    #[test]
    fn to_json_source_contains_path_when_span_present() {
        let v = error_with_span().to_json("input.tex");
        let src = v.get("source").unwrap();
        assert_eq!(src.get("path").and_then(|s| s.as_str()), Some("input.tex"));
    }

    #[test]
    fn to_json_source_byte_offsets_match_span() {
        let v = error_with_span().to_json("x.tex"); // span 3..10
        let src = v.get("source").unwrap();
        assert_eq!(src.get("start_byte").and_then(|n| n.as_i64()), Some(3));
        assert_eq!(src.get("end_byte").and_then(|n| n.as_i64()), Some(10));
    }

    // ── to_json: recovery ─────────────────────────────────────────────────────

    #[test]
    fn to_json_recovery_is_null_when_none() {
        let v = warning_no_span().to_json("f.tex");
        assert_eq!(v.get("recovery"), Some(&crate::json::Value::Null));
    }

    #[test]
    fn to_json_recovery_string_when_some() {
        let v = error_with_span().to_json("f.tex");
        assert_eq!(v.get("recovery").and_then(|s| s.as_str()), Some("recovered fine"));
    }

    // ── serialised JSON is one-line and valid ─────────────────────────────────

    #[test]
    fn to_json_serialises_to_valid_json() {
        let d = error_with_span();
        let v = d.to_json("doc.tex");
        let s = write(&v);
        // Must not be empty and must parse back correctly.
        let reparsed = crate::json::parse(&s).unwrap();
        assert_eq!(
            reparsed.get("severity").and_then(|x| x.as_str()),
            Some("error")
        );
    }

    #[test]
    fn to_json_output_has_no_newlines() {
        let d = Diagnostic::error("line\nbreak in message", Some(Span::new(0, 5)), None);
        let v = d.to_json("f.tex");
        let s = write(&v);
        // The JSON writer must escape the newline; the output must be a single line.
        assert!(!s.contains('\n'), "serialised diagnostic must not contain a raw newline");
    }
}
