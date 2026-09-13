//! The project closure: the entry file and every `\input`/`\include` it
//! reaches, resolved from the project root by project-files' graph
//! discovery (rooted reads through a directory handle, `O_NOFOLLOW` at
//! every component, so no include escapes the root through `..` or a
//! symlink). The compiler resolves the same references again by
//! project-relative path when it parses the documents.

use std::path::{Path, PathBuf};

use flashtex_project_files::{DiagnosticKind, ProjectGraph, ProjectPath, Severity};

/// One source document, as runtime-v1 carries it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub path: String,
    pub text: String,
}

/// A diagnostic from discovery (a missing or unreadable include, a path
/// escaping the root, a cycle), attributed like the engine's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDiagnostic {
    pub path: String,
    pub start_byte: Option<usize>,
    pub end_byte: Option<usize>,
    pub error: bool,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Project {
    /// Absolute project root (every path below is relative to it).
    pub root: PathBuf,
    /// The entry file, project-relative (`main.tex`, `paper/main.tex`).
    pub entry: String,
    pub documents: Vec<Document>,
    pub diagnostics: Vec<ProjectDiagnostic>,
    /// Every file in the closure (documents, bibliographies, graphics),
    /// project-relative, for `watch`.
    pub files: Vec<String>,
}

/// Resolves `main` against `project_root` (default: the directory holding
/// `main`) and discovers the closure. Errors are usage errors (exit 2): a
/// missing file, an entry outside the root, an unreadable root.
pub fn load(main: &Path, project_root: Option<&Path>) -> Result<Project, String> {
    let main_abs = std::fs::canonicalize(main).map_err(|e| format!("cannot read {}: {e}", main.display()))?;
    if !main_abs.is_file() {
        return Err(format!("{} is not a file", main.display()));
    }
    let root = match project_root {
        Some(r) => std::fs::canonicalize(r).map_err(|e| format!("cannot open project root {}: {e}", r.display()))?,
        None => main_abs.parent().map(Path::to_path_buf).ok_or_else(|| format!("{} has no parent directory", main.display()))?,
    };
    let rel = main_abs
        .strip_prefix(&root)
        .map_err(|_| format!("{} is not inside the project root {}", main_abs.display(), root.display()))?;
    let rel = rel.to_str().ok_or_else(|| format!("{} is not valid UTF-8", rel.display()))?;
    let entry = ProjectPath::normalize(rel).map_err(|e| format!("{rel}: {e}"))?;
    let graph = ProjectGraph::discover(&root, &entry).map_err(|e| e.to_string())?;
    let documents = graph
        .documents()
        .into_iter()
        .map(|d| Document { path: d.path, text: d.text })
        .collect();
    let diagnostics = graph
        .diagnostics()
        .iter()
        .map(|d| ProjectDiagnostic {
            path: d.path.as_str().to_string(),
            start_byte: d.span.map(|s| s.start),
            end_byte: d.span.map(|s| s.end),
            error: d.severity == Severity::Error,
            code: match d.kind {
                DiagnosticKind::MissingFile { .. } => "missing_file",
                DiagnosticKind::InvalidPath { .. } => "invalid_path",
                DiagnosticKind::EscapesRootViaSymlink { .. } => "path_escapes_root",
                DiagnosticKind::Cycle { .. } => "include_cycle",
                DiagnosticKind::UnresolvableReference { .. } => "unresolved_reference",
                DiagnosticKind::InvalidUtf8 { .. } => "not_utf8",
                DiagnosticKind::ReadError { .. } => "read_error",
                DiagnosticKind::DepthExceeded { .. } => "include_depth",
            },
            message: d.message.clone(),
        })
        .collect();
    let files = graph.all_paths().map(|p| p.as_str().to_string()).collect();
    Ok(Project {
        root,
        entry: entry.as_str().to_string(),
        documents,
        diagnostics,
        files,
    })
}
