//! Explanations and previewable fix suggestions for FlashTeX compiler diagnostics.
//!
//! The compiler reports a diagnostic as `{severity, message, source, recovery}`
//! (`docs/contracts/runtime-v1.md`). This crate turns one into an
//! [`Explanation`]: a short title, a category, why it happened, what the
//! compiler did instead (from `recovery`), a bounded source [`ContextWindow`],
//! and [`Suggestion`]s whose [`Edit`]s the UI can preview. Nothing here applies
//! an edit; nothing here calls a model or any external service.
//!
//! Matching is keyed on the compiler's message text ([`catalog`]), so this crate
//! never has to be linked into the compiler and never edits it. Every catalog
//! pattern is pinned to the compiler revision it was read from; see `README.md`.
//!
//! All offsets are zero-based, end-exclusive UTF-8 byte offsets into the exact
//! document text the diagnostic was produced for, the contract's currency.

pub mod catalog;
pub mod context;
pub mod json;
pub mod pattern;
pub mod preview;
pub mod suggest;
pub mod text;

pub use context::ContextWindow;
pub use preview::{ChangedRegion, Preview, PreviewError, Rebase};

/// Diagnostic severity, as the contract spells it.
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

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "error" => Some(Severity::Error),
            "warning" => Some(Severity::Warning),
            _ => None,
        }
    }
}

/// `source` of a runtime-v1 diagnostic: a byte range into one document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub path: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// A runtime-v1 diagnostic as the Mac app receives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub source: Option<Source>,
    pub recovery: Option<String>,
}

impl Diagnostic {
    pub fn new(
        severity: Severity,
        message: impl Into<String>,
        source: Option<Source>,
        recovery: Option<&str>,
    ) -> Self {
        Diagnostic {
            severity,
            message: message.into(),
            source,
            recovery: recovery.map(str::to_string),
        }
    }

    /// Convenience for tests and callers holding a single document.
    pub fn at(
        severity: Severity,
        message: &str,
        path: &str,
        start: usize,
        end: usize,
        recovery: &str,
    ) -> Self {
        Diagnostic::new(
            severity,
            message,
            Some(Source {
                path: path.to_string(),
                start_byte: start,
                end_byte: end,
            }),
            Some(recovery),
        )
    }
}

/// Coarse class of problem, stable for UI grouping and icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    UnsupportedCommand,
    UnmatchedBrace,
    EnvironmentMismatch,
    UnsupportedEnvironment,
    MathUnsupported,
    MathSyntax,
    MissingArgument,
    EmptyArgument,
    PreambleUnsupported,
    MacroDefinition,
    MultiFile,
    PathRejected,
    Protocol,
    Unknown,
}

impl Category {
    pub const ALL: &'static [Category] = &[
        Category::UnsupportedCommand,
        Category::UnmatchedBrace,
        Category::EnvironmentMismatch,
        Category::UnsupportedEnvironment,
        Category::MathUnsupported,
        Category::MathSyntax,
        Category::MissingArgument,
        Category::EmptyArgument,
        Category::PreambleUnsupported,
        Category::MacroDefinition,
        Category::MultiFile,
        Category::PathRejected,
        Category::Protocol,
        Category::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Category::UnsupportedCommand => "unsupported-command",
            Category::UnmatchedBrace => "unmatched-brace",
            Category::EnvironmentMismatch => "environment-mismatch",
            Category::UnsupportedEnvironment => "unsupported-environment",
            Category::MathUnsupported => "math-unsupported",
            Category::MathSyntax => "math-syntax",
            Category::MissingArgument => "missing-argument",
            Category::EmptyArgument => "empty-argument",
            Category::PreambleUnsupported => "preamble-unsupported",
            Category::MacroDefinition => "macro-definition",
            Category::MultiFile => "multi-file",
            Category::PathRejected => "path-rejected",
            Category::Protocol => "protocol",
            Category::Unknown => "unknown",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Category::ALL.iter().copied().find(|c| c.as_str() == s)
    }
}

/// How sure the heuristic is that the edit is what the author wants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Confidence::Low => "low",
            Confidence::Medium => "medium",
            Confidence::High => "high",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "low" => Some(Confidence::Low),
            "medium" => Some(Confidence::Medium),
            "high" => Some(Confidence::High),
            _ => None,
        }
    }
}

/// One replacement the UI may preview. `start_byte == end_byte` is an
/// insertion; an empty `replacement` is a deletion. Never applied here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub path: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub replacement: String,
}

impl Edit {
    pub fn new(
        path: &str,
        start_byte: usize,
        end_byte: usize,
        replacement: impl Into<String>,
    ) -> Self {
        Edit {
            path: path.to_string(),
            start_byte,
            end_byte,
            replacement: replacement.into(),
        }
    }

    pub fn insert(path: &str, at: usize, replacement: impl Into<String>) -> Self {
        Edit::new(path, at, at, replacement)
    }

    pub fn delete(path: &str, start_byte: usize, end_byte: usize) -> Self {
        Edit::new(path, start_byte, end_byte, "")
    }
}

/// A concrete thing the author can do. `edits` may be empty for advice that
/// has no mechanical form (for example, moving a file into the project).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub text: String,
    pub edits: Vec<Edit>,
    pub confidence: Confidence,
}

impl Suggestion {
    pub fn new(text: impl Into<String>, edits: Vec<Edit>, confidence: Confidence) -> Self {
        Suggestion {
            text: text.into(),
            edits,
            confidence,
        }
    }

    pub fn advice(text: impl Into<String>, confidence: Confidence) -> Self {
        Suggestion::new(text, Vec::new(), confidence)
    }
}

/// The explanation of one diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    /// Stable catalog identifier, or `None` when no pattern matched.
    pub catalog_id: Option<String>,
    pub title: String,
    pub category: Category,
    pub severity: Severity,
    /// Why the compiler complained.
    pub why: String,
    /// What the compiler did instead, derived from the diagnostic's `recovery`.
    pub what_happened: String,
    pub suggestions: Vec<Suggestion>,
    /// Bounded source excerpt, or `None` when the diagnostic has no source.
    pub context: Option<ContextWindow>,
    /// The original message, echoed so the UI can show both.
    pub message: String,
}

/// One document of a compile request, for the batch API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Document<'a> {
    pub path: &'a str,
    pub text: &'a str,
}

/// Commands the app already knows the compiler implements (names without the
/// leading backslash). This mirrors `Completion.defaultSupported` in the Mac
/// app: the core list from `crates/compiler/src/parser.rs` `SUPPORTED` on
/// `origin/main` 1dd26c5 plus the math names in `crates/compiler/src/math.rs`
/// on `origin/agent/claude/compiler-foundation` de1020c. The app passes its own
/// list; this constant exists so callers without one still get suggestions.
pub const DEFAULT_SUPPORTED_COMMANDS: &[&str] = &[
    "section",
    "subsection",
    "textbf",
    "emph",
    "textit",
    "begin",
    "end",
    "par",
    "\\",
    "frac",
    "sqrt",
    "alpha",
    "beta",
    "gamma",
    "delta",
    "theta",
    "lambda",
    "mu",
    "pi",
    "sigma",
    "phi",
    "omega",
    "times",
    "div",
    "pm",
    "leq",
    "geq",
    "neq",
    "approx",
    "cdot",
    "infty",
    "sum",
    "int",
];

/// Explain one diagnostic against the text of the document its `source` names.
///
/// `document_text` must be the exact text the compile ran on (same revision);
/// pass `""` when it is unknown and the explanation degrades to advice with
/// an empty context and no edits. `supported_commands` are names without a
/// backslash. The result is deterministic for identical inputs.
pub fn explain(
    diagnostic: &Diagnostic,
    document_text: &str,
    supported_commands: &[&str],
) -> Explanation {
    let matched = catalog::lookup(&diagnostic.message);
    let context = diagnostic
        .source
        .as_ref()
        .map(|s| context::window(&s.path, document_text, s.start_byte, s.end_byte));

    let what_happened = match &diagnostic.recovery {
        Some(r) if !r.trim().is_empty() => {
            let mut s = r.trim().to_string();
            if !s.ends_with('.') {
                s.push('.');
            }
            format!("The compiler {}", lowercase_first(&s))
        }
        _ => match diagnostic.severity {
            Severity::Error => {
                "The compiler recorded the error and did not recover any output for it.".into()
            }
            Severity::Warning => {
                "The compiler continued; nothing was recovered or dropped for this warning.".into()
            }
        },
    };

    match matched {
        Some((entry, captures)) => {
            let mut suggestions = suggest::suggestions(
                entry.kind,
                &captures,
                diagnostic,
                document_text,
                supported_commands,
            );
            if suggestions.is_empty() {
                suggestions.push(generic_suggestion(diagnostic));
            }
            Explanation {
                catalog_id: Some(entry.id.to_string()),
                title: (entry.title)(&captures),
                category: entry.category,
                severity: diagnostic.severity,
                why: (entry.why)(&captures),
                what_happened,
                suggestions,
                context,
                message: diagnostic.message.clone(),
            }
        }
        None => Explanation {
            catalog_id: None,
            title: fallback_title(diagnostic),
            category: Category::Unknown,
            severity: diagnostic.severity,
            why: format!(
                "The compiler reported \"{}\". This message is not in the explanation catalog yet, so only the compiler's own wording is available.",
                diagnostic.message
            ),
            what_happened,
            suggestions: vec![generic_suggestion(diagnostic)],
            context,
            message: diagnostic.message.clone(),
        },
    }
}

/// Explain every diagnostic of a `compile_result`, looking each one's document
/// up by `source.path`. Diagnostics without a source, or whose path is not in
/// `documents`, are explained without document text. Order is preserved.
pub fn explain_all(
    diagnostics: &[Diagnostic],
    documents: &[Document<'_>],
    supported_commands: &[&str],
) -> Vec<Explanation> {
    diagnostics
        .iter()
        .map(|d| {
            let text = d
                .source
                .as_ref()
                .and_then(|s| documents.iter().find(|doc| doc.path == s.path))
                .map(|doc| doc.text)
                .unwrap_or("");
            explain(d, text, supported_commands)
        })
        .collect()
}

/// Parse a runtime-v1 `compile_result` (either the full envelope or just its
/// payload) and explain its diagnostics.
pub fn explain_compile_result_json(
    compile_result_json: &str,
    documents: &[Document<'_>],
    supported_commands: &[&str],
) -> Result<Vec<Explanation>, json::JsonError> {
    let diagnostics = json::diagnostics_from_compile_result(compile_result_json)?;
    Ok(explain_all(&diagnostics, documents, supported_commands))
}

fn generic_suggestion(diagnostic: &Diagnostic) -> Suggestion {
    match &diagnostic.source {
        Some(_) => Suggestion::advice(
            "Review the highlighted source; the compiler's recovery text says what was typeset in its place.",
            Confidence::Low,
        ),
        None => Suggestion::advice(
            "This problem is not tied to a source position; check the compile request (paths, documents, protocol fields).",
            Confidence::Low,
        ),
    }
}

fn fallback_title(diagnostic: &Diagnostic) -> String {
    let first = diagnostic
        .message
        .split([';', '—', ':'])
        .next()
        .unwrap_or("")
        .trim();
    let mut title = capitalize(first);
    // Bound the title so the UI never has to wrap a whole paragraph.
    if title.chars().count() > 60 {
        title = title.chars().take(57).collect::<String>() + "...";
    }
    if title.is_empty() {
        match diagnostic.severity {
            Severity::Error => "Compiler error".to_string(),
            Severity::Warning => "Compiler warning".to_string(),
        }
    } else {
        title
    }
}

pub(crate) fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) if first.is_alphabetic() => {
            first.to_uppercase().collect::<String>() + chars.as_str()
        }
        _ => s.to_string(),
    }
}

pub(crate) fn lowercase_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) if first.is_alphabetic() => {
            first.to_lowercase().collect::<String>() + chars.as_str()
        }
        _ => s.to_string(),
    }
}
