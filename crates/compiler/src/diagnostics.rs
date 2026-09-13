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

/// Machine-readable category, serialized as the optional runtime-v1 `code`.
///
/// Consumers classify by this rather than by message wording; messages stay
/// human prose and may change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    /// A command no LaTeX layer this compiler knows of defines (most often a
    /// typo); may carry a did-you-mean `suggestion`.
    UnknownCommand,
    /// Real LaTeX (a known command, environment, package or key) that this
    /// compiler does not implement yet.
    UnsupportedFeature,
    /// Malformed input: unbalanced braces, unterminated environments,
    /// misplaced `&`, missing arguments.
    SyntaxError,
    /// The preview is right but the exported PDF cannot reproduce it.
    ExportLimitation,
    /// Rendered, but visibly different from what pdfLaTeX would produce.
    FidelityNote,
    /// Input the author must fix (missing include, undefined reference,
    /// duplicate label) that the compiler worked around.
    RecoveredInput,
}

impl DiagnosticCode {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticCode::UnknownCommand => "unknown_command",
            DiagnosticCode::UnsupportedFeature => "unsupported_feature",
            DiagnosticCode::SyntaxError => "syntax_error",
            DiagnosticCode::ExportLimitation => "export_limitation",
            DiagnosticCode::FidelityNote => "fidelity_note",
            DiagnosticCode::RecoveredInput => "recovered_input",
        }
    }
}

/// Default code for a diagnostic from the compiler's own message conventions.
///
/// This lets every existing construction site carry a code without being
/// touched. Sites whose category needs more than wording (unknown versus
/// known-unsupported commands) set it explicitly via
/// [`Diagnostic::command_error`] / [`Diagnostic::with_code`]. New sites should
/// either follow these wordings or set a code explicitly; `None` (for example
/// request-validation errors) omits the field from JSON.
pub fn default_code(message: &str) -> Option<DiagnosticCode> {
    let has = |needles: &[&str]| needles.iter().any(|n| message.contains(n));
    if has(&["PDF export", "page-unit bounds"]) {
        Some(DiagnosticCode::ExportLimitation)
    } else if has(&[
        "has no glyph",
        "could not shape",
        "differ from pdfLaTeX",
        "accent glyph",
        "does not stretch",
        "did not converge",
    ]) {
        Some(DiagnosticCode::FidelityNote)
    } else if has(&[
        "undefined reference",
        "duplicate \\label",
        "was given an empty",
        "included file not found",
        "include path",
        "include cycle",
        "include depth",
        "project-relative path",
        "recursion limit",
        "received an empty required argument",
    ]) {
        Some(DiagnosticCode::RecoveredInput)
    } else if has(&[
        "not supported",
        "not implemented",
        "unsupported",
        "supports only",
        "does not support",
        "exceeds the supported",
        "recognised dimension",
    ]) {
        Some(DiagnosticCode::UnsupportedFeature)
    } else if has(&[
        "unmatched",
        "unterminated",
        "missing its",
        "misplaced",
        "stray",
        "duplicate script",
        "script marker",
        "unexpected math delimiter",
        "does not match",
        "no matching",
        "requires a",
        "requires math mode",
        "only supported inside",
        "argument count",
        "cannot redefine",
        "references #",
    ]) {
        Some(DiagnosticCode::SyntaxError)
    } else {
        None
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
    /// Serialized as `code`; `None` omits the field.
    pub code: Option<DiagnosticCode>,
    /// Replacement text for the source range (e.g. `\alpha` for `\alpah`),
    /// serialized as `suggestion`; `None` omits the field.
    pub suggestion: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Option<Span>, recovery: Option<String>) -> Self {
        let message = message.into();
        Diagnostic {
            severity: Severity::Error,
            code: default_code(&message),
            message,
            span,
            recovery,
            suggestion: None,
        }
    }

    pub fn warning(
        message: impl Into<String>,
        span: Option<Span>,
        recovery: Option<String>,
    ) -> Self {
        let message = message.into();
        Diagnostic {
            severity: Severity::Warning,
            code: default_code(&message),
            message,
            span,
            recovery,
            suggestion: None,
        }
    }

    pub fn with_code(mut self, code: DiagnosticCode) -> Self {
        self.code = Some(code);
        self
    }

    /// An error about command `name` (without the backslash):
    /// `unknown_command` with a did-you-mean suggestion when no known LaTeX
    /// layer defines it, otherwise `unsupported_feature`.
    pub fn command_error(
        name: &str,
        message: impl Into<String>,
        span: Option<Span>,
        recovery: Option<String>,
    ) -> Self {
        let mut diagnostic = Diagnostic::error(message, span, recovery);
        if crate::vocabulary::is_known_command(name) {
            diagnostic.code = Some(DiagnosticCode::UnsupportedFeature);
        } else {
            diagnostic.code = Some(DiagnosticCode::UnknownCommand);
            diagnostic.suggestion =
                crate::vocabulary::suggest_command(name).map(|known| format!("\\{known}"));
        }
        diagnostic
    }

    /// An error about environment `name`, classified like
    /// [`Diagnostic::command_error`]. No suggestion: the span covers `\begin`,
    /// not the name.
    pub fn environment_error(
        name: &str,
        message: impl Into<String>,
        span: Option<Span>,
        recovery: Option<String>,
    ) -> Self {
        Diagnostic::error(message, span, recovery).with_environment_code(name)
    }

    /// Warning counterpart of [`Diagnostic::environment_error`].
    pub fn environment_warning(
        name: &str,
        message: impl Into<String>,
        span: Option<Span>,
        recovery: Option<String>,
    ) -> Self {
        Diagnostic::warning(message, span, recovery).with_environment_code(name)
    }

    fn with_environment_code(self, name: &str) -> Self {
        self.with_code(if crate::vocabulary::is_known_environment(name) {
            DiagnosticCode::UnsupportedFeature
        } else {
            DiagnosticCode::UnknownCommand
        })
    }

    pub fn to_json(&self, path: &str) -> Value {
        self.to_json_with_paths(&[path])
    }

    /// Serialize with the path belonging to the document carried by the span.
    pub fn to_json_with_paths(&self, paths: &[&str]) -> Value {
        let mut v = Value::obj();
        v.set("severity", str_(self.severity.as_str()));
        v.set("message", str_(self.message.clone()));
        if let Some(code) = self.code {
            v.set("code", str_(code.as_str()));
        }
        if let Some(suggestion) = &self.suggestion {
            v.set("suggestion", str_(suggestion.clone()));
        }
        v.set(
            "source",
            match self.span {
                Some(s) => {
                    let mut src = Value::obj();
                    src.set("path", str_(paths.get(s.document.0).copied().unwrap_or("")));
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

    #[test]
    fn default_codes_follow_the_compiler_message_conventions() {
        use DiagnosticCode::*;
        for (message, code) in [
            (
                "'─' (U+2500) will not survive PDF export: stand-in",
                Some(ExportLimitation),
            ),
            (
                "fraction rule geometry exceeds the runtime-v1 page-unit bounds",
                Some(ExportLimitation),
            ),
            (
                "Times-Roman has no glyph for 'x' (U+0078)",
                Some(FidelityNote),
            ),
            (
                "widths differ from pdfLaTeX's msbm10/cmsy10",
                Some(FidelityNote),
            ),
            (
                "\\hat has no representable accent glyph in the compiler's base-14 fonts",
                Some(FidelityNote),
            ),
            ("undefined reference 'k'", Some(RecoveredInput)),
            (
                "duplicate \\label{k}; the second definition wins",
                Some(RecoveredInput),
            ),
            (
                "included file not found: looked for 'a' and 'a.tex'",
                Some(RecoveredInput),
            ),
            (
                "packages tikz are recognised but not implemented",
                Some(UnsupportedFeature),
            ),
            (
                "\\setlist keys x are recognised but not implemented",
                Some(UnsupportedFeature),
            ),
            (
                "\\includegraphics is unsupported; image loading is not implemented",
                Some(UnsupportedFeature),
            ),
            (
                "\\mathbb supports only capital letters A-Z, not \"ab\"",
                Some(UnsupportedFeature),
            ),
            (
                "\\hspace requires a recognised dimension, got 'x'",
                Some(UnsupportedFeature),
            ),
            ("unmatched '{' — group never closed", Some(SyntaxError)),
            (
                "unterminated environment 'x' — no matching \\end",
                Some(SyntaxError),
            ),
            ("misplaced alignment tab character &", Some(SyntaxError)),
            ("inline math is missing its closing '$'", Some(SyntaxError)),
            ("\\end{a} does not match \\begin{b}", Some(SyntaxError)),
            ("\\textbf requires a braced argument", Some(SyntaxError)),
            (
                "\\item is only supported inside itemize or enumerate",
                Some(SyntaxError),
            ),
            ("layout_capabilities must be a list", None),
        ] {
            assert_eq!(default_code(message), code, "{message}");
        }
    }

    #[test]
    fn code_and_suggestion_are_omitted_from_json_when_absent() {
        let plain = Diagnostic::error("layout_capabilities must be a list", None, None);
        let json = crate::json::write(&plain.to_json(""));
        assert!(
            !json.contains("\"code\"") && !json.contains("suggestion"),
            "{json}"
        );
        let typo = Diagnostic::command_error("alpah", "\\alpah is not supported", None, None);
        let json = crate::json::write(&typo.to_json(""));
        assert!(json.contains(r#""code":"unknown_command""#), "{json}");
        assert!(json.contains(r#""suggestion":"\\alpha""#), "{json}");
    }
}
