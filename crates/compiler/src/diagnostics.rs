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

#[derive(Debug, Clone, PartialEq, Eq)]
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
        Diagnostic {
            severity: Severity::Error,
            message: message.into(),
            span,
            recovery,
        }
    }

    pub fn warning(
        message: impl Into<String>,
        span: Option<Span>,
        recovery: Option<String>,
    ) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            message: message.into(),
            span,
            recovery,
        }
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
