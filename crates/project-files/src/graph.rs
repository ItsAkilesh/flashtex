//! Normalized project graph: root directory, entry file, discovered
//! references, resolved files, cycle/missing/escape diagnostics, and the
//! runtime-v1 `documents` export.
//!
//! Resolution follows TeX's working-directory rule: every reference is
//! resolved against the project root, not the referencing file's directory
//! (`\input{chapters/a}` inside `chapters/main.tex` still means
//! `<root>/chapters/a.tex`). Files are visited depth-first in reference
//! order, so `files()` and `documents()` are deterministic for a given tree.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::json::Json;
use crate::path::{PathError, ProjectPath};
use crate::scan::{ByteSpan, Reference, ReferenceKind, scan_references};
use crate::sha256::{Digest, sha256};

/// Maximum nesting of `\input`/`\include` before discovery stops descending.
pub const MAX_DEPTH: usize = 64;

/// Extensions tried, in order, for `\includegraphics{name}` without extension.
pub const GRAPHIC_EXTENSIONS: &[&str] = &["pdf", "png", "jpg", "jpeg", "eps", "svg"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileKind {
    /// LaTeX source; scanned for references and exported as a document.
    Tex,
    /// BibTeX database; loaded as text, not scanned.
    Bibliography,
    /// Graphic asset; hashed but not loaded as text.
    Graphic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSource {
    /// Bytes read from `<root>/<path>`.
    Disk,
    /// Text supplied by the caller (unsaved editor buffer); disk was not read.
    Overlay,
}

/// One file in the graph.
#[derive(Debug, Clone)]
pub struct ProjectFile {
    pub path: ProjectPath,
    pub kind: FileKind,
    pub source: FileSource,
    /// UTF-8 text for `Tex` and `Bibliography`; `None` for graphics or when
    /// the bytes were not valid UTF-8 (a diagnostic is emitted).
    pub text: Option<String>,
    /// SHA-256 of the exact bytes (overlay text or disk contents).
    pub sha256: Digest,
    pub bytes: u64,
    /// References found in this file (`Tex` only).
    pub references: Vec<Reference>,
}

/// A resolved reference from one file to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: ProjectPath,
    pub to: ProjectPath,
    pub reference: Reference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// No candidate for the referenced name exists.
    MissingFile {
        target: String,
        tried: Vec<ProjectPath>,
    },
    /// The referenced name is not a valid project-relative path.
    InvalidPath { target: String, error: PathError },
    /// The referenced name is a symlink resolving outside the root.
    EscapesRootViaSymlink { target: ProjectPath },
    /// The reference closes a cycle; `chain` runs from the first repeated
    /// file to the referencing file, and the target is `chain[0]`.
    Cycle { chain: Vec<ProjectPath> },
    /// The argument contains `\` or `#` and cannot be resolved without expansion.
    UnresolvableReference { target: String },
    /// The file exists but is not valid UTF-8.
    InvalidUtf8 { path: ProjectPath },
    /// The file exists but could not be read.
    ReadError { path: ProjectPath, message: String },
    /// Nesting exceeded [`MAX_DEPTH`]; the target was not descended into.
    DepthExceeded { target: ProjectPath },
}

/// A discovery diagnostic attributed to the referencing file and span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    /// The file containing the reference (or the file itself for read errors).
    pub path: ProjectPath,
    /// Span of the whole referencing command, if any.
    pub span: Option<ByteSpan>,
    /// Span of the referenced name inside the command, if any.
    pub argument_span: Option<ByteSpan>,
    pub kind: DiagnosticKind,
}

/// Unsaved buffers that take precedence over disk contents during discovery.
#[derive(Debug, Clone, Default)]
pub struct Overlay {
    texts: BTreeMap<ProjectPath, String>,
}

impl Overlay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, path: ProjectPath, text: impl Into<String>) -> &mut Self {
        self.texts.insert(path, text.into());
        self
    }

    pub fn remove(&mut self, path: &ProjectPath) -> Option<String> {
        self.texts.remove(path)
    }

    pub fn get(&self, path: &ProjectPath) -> Option<&str> {
        self.texts.get(path).map(String::as_str)
    }

    pub fn paths(&self) -> impl Iterator<Item = &ProjectPath> {
        self.texts.keys()
    }
}

/// Why discovery could not even start.
#[derive(Debug)]
pub enum DiscoverError {
    RootNotDirectory(PathBuf),
    EntryMissing(ProjectPath),
    EntryUnreadable(ProjectPath, io::Error),
    EntryNotUtf8(ProjectPath),
}

impl fmt::Display for DiscoverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoverError::RootNotDirectory(p) => {
                write!(f, "project root {} is not a directory", p.display())
            }
            DiscoverError::EntryMissing(p) => write!(f, "entry file {p} does not exist"),
            DiscoverError::EntryUnreadable(p, e) => {
                write!(f, "entry file {p} could not be read: {e}")
            }
            DiscoverError::EntryNotUtf8(p) => write!(f, "entry file {p} is not valid UTF-8"),
        }
    }
}

impl std::error::Error for DiscoverError {}

/// A runtime-v1 document: `{path, text}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub path: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ProjectGraph {
    root: PathBuf,
    entry: ProjectPath,
    files: Vec<ProjectFile>,
    edges: Vec<Edge>,
    diagnostics: Vec<Diagnostic>,
}

impl ProjectGraph {
    /// Discovers the graph from disk only.
    pub fn discover(root: &Path, entry: &ProjectPath) -> Result<ProjectGraph, DiscoverError> {
        Self::discover_with(root, entry, &Overlay::default())
    }

    /// Discovers the graph, preferring `overlay` buffers over disk contents.
    pub fn discover_with(
        root: &Path,
        entry: &ProjectPath,
        overlay: &Overlay,
    ) -> Result<ProjectGraph, DiscoverError> {
        if !root.is_dir() {
            return Err(DiscoverError::RootNotDirectory(root.to_path_buf()));
        }
        let canonical_root = fs::canonicalize(root)
            .map_err(|_| DiscoverError::RootNotDirectory(root.to_path_buf()))?;
        let mut d = Discovery {
            root: root.to_path_buf(),
            canonical_root,
            overlay,
            graph: ProjectGraph {
                root: root.to_path_buf(),
                entry: entry.clone(),
                files: Vec::new(),
                edges: Vec::new(),
                diagnostics: Vec::new(),
            },
            index: BTreeMap::new(),
            stack: Vec::new(),
        };
        // The entry must load; anything else is a diagnostic.
        match d.load(entry, FileKind::Tex) {
            Loaded::Ok(file) => {
                if file.text.is_none() {
                    return Err(DiscoverError::EntryNotUtf8(entry.clone()));
                }
                d.visit_loaded(file);
            }
            Loaded::Missing => return Err(DiscoverError::EntryMissing(entry.clone())),
            Loaded::Error(e) => return Err(DiscoverError::EntryUnreadable(entry.clone(), e)),
        }
        Ok(d.graph)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn entry(&self) -> &ProjectPath {
        &self.entry
    }

    /// Files in deterministic depth-first discovery order; the entry is first.
    pub fn files(&self) -> &[ProjectFile] {
        &self.files
    }

    pub fn file(&self, path: &ProjectPath) -> Option<&ProjectFile> {
        self.files.iter().find(|f| &f.path == path)
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Paths of all text files (tex and bib) in discovery order.
    pub fn text_paths(&self) -> impl Iterator<Item = &ProjectPath> {
        self.files
            .iter()
            .filter(|f| f.kind != FileKind::Graphic)
            .map(|f| &f.path)
    }

    /// Paths of every discovered file, including graphics.
    pub fn all_paths(&self) -> impl Iterator<Item = &ProjectPath> {
        self.files.iter().map(|f| &f.path)
    }

    /// The runtime-v1 `documents` list: every `Tex` file with valid UTF-8
    /// text, entry first, then depth-first reference order.
    pub fn documents(&self) -> Vec<Document> {
        self.documents_where(|f| f.kind == FileKind::Tex)
    }

    /// Like [`documents`](Self::documents) but also carries `.bib` sources
    /// (for a bibliography-aware compiler; runtime-v1 does not forbid them).
    pub fn documents_including_bibliography(&self) -> Vec<Document> {
        self.documents_where(|f| f.kind != FileKind::Graphic)
    }

    fn documents_where(&self, keep: impl Fn(&ProjectFile) -> bool) -> Vec<Document> {
        self.files
            .iter()
            .filter(|f| keep(f))
            .filter_map(|f| {
                f.text.as_ref().map(|t| Document {
                    path: f.path.as_str().to_string(),
                    text: t.clone(),
                })
            })
            .collect()
    }

    /// The runtime-v1 `compile` payload object for `project_id` at `revision`.
    pub fn compile_payload(&self, project_id: &str, revision: u64) -> Json {
        let docs = self
            .documents()
            .into_iter()
            .map(|d| {
                let mut o = Json::object();
                o.insert("path", d.path).insert("text", d.text);
                o
            })
            .collect::<Vec<_>>();
        let mut payload = Json::object();
        payload
            .insert("project_id", project_id)
            .insert("revision", revision)
            .insert("entry_path", self.entry.as_str())
            .insert("documents", docs);
        payload
    }

    /// A complete runtime-v1 `compile` envelope line (no trailing newline).
    pub fn compile_envelope(&self, id: &str, project_id: &str, revision: u64) -> String {
        let mut env = Json::object();
        env.insert("protocol_version", 1u64)
            .insert("id", id)
            .insert("type", "compile")
            .insert("payload", self.compile_payload(project_id, revision));
        env.to_string_compact()
    }
}

enum Loaded {
    Ok(ProjectFile),
    Missing,
    Error(io::Error),
}

struct Discovery<'a> {
    root: PathBuf,
    canonical_root: PathBuf,
    overlay: &'a Overlay,
    graph: ProjectGraph,
    index: BTreeMap<ProjectPath, usize>,
    stack: Vec<ProjectPath>,
}

impl Discovery<'_> {
    fn exists(&self, path: &ProjectPath) -> bool {
        self.overlay.get(path).is_some() || path.to_os_path(&self.root).is_file()
    }

    fn load(&self, path: &ProjectPath, kind: FileKind) -> Loaded {
        if kind != FileKind::Graphic
            && let Some(text) = self.overlay.get(path)
        {
            return Loaded::Ok(ProjectFile {
                path: path.clone(),
                kind,
                source: FileSource::Overlay,
                text: Some(text.to_string()),
                sha256: sha256(text.as_bytes()),
                bytes: text.len() as u64,
                references: Vec::new(),
            });
        }
        let os = path.to_os_path(&self.root);
        let bytes = match fs::read(&os) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Loaded::Missing,
            Err(e) => return Loaded::Error(e),
        };
        let digest = sha256(&bytes);
        let len = bytes.len() as u64;
        let text = if kind == FileKind::Graphic {
            None
        } else {
            String::from_utf8(bytes).ok()
        };
        Loaded::Ok(ProjectFile {
            path: path.clone(),
            kind,
            source: FileSource::Disk,
            text,
            sha256: digest,
            bytes: len,
            references: Vec::new(),
        })
    }

    /// True if `path` exists on disk as a symlink that resolves outside the root.
    fn escapes_via_symlink(&self, path: &ProjectPath) -> bool {
        if self.overlay.get(path).is_some() {
            return false;
        }
        let os = path.to_os_path(&self.root);
        match fs::canonicalize(&os) {
            Ok(canon) => !canon.starts_with(&self.canonical_root),
            Err(_) => false,
        }
    }

    fn add_file(&mut self, mut file: ProjectFile) -> usize {
        if file.kind == FileKind::Tex
            && let Some(text) = &file.text
        {
            file.references = scan_references(text);
        }
        let idx = self.graph.files.len();
        self.index.insert(file.path.clone(), idx);
        self.graph.files.push(file);
        idx
    }

    fn visit_loaded(&mut self, file: ProjectFile) {
        let path = file.path.clone();
        if file.kind == FileKind::Tex && file.text.is_none() {
            self.graph.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: format!("{path} is not valid UTF-8"),
                path: path.clone(),
                span: None,
                argument_span: None,
                kind: DiagnosticKind::InvalidUtf8 { path: path.clone() },
            });
        }
        let idx = self.add_file(file);
        self.stack.push(path.clone());
        let refs = self.graph.files[idx].references.clone();
        for r in refs {
            self.follow(&path, &r);
        }
        self.stack.pop();
    }

    fn diag(
        &mut self,
        from: &ProjectPath,
        r: &Reference,
        severity: Severity,
        message: String,
        kind: DiagnosticKind,
    ) {
        self.graph.diagnostics.push(Diagnostic {
            severity,
            message,
            path: from.clone(),
            span: Some(r.span),
            argument_span: Some(r.argument_span),
            kind,
        });
    }

    fn follow(&mut self, from: &ProjectPath, r: &Reference) {
        if !r.literal {
            self.diag(
                from,
                r,
                Severity::Warning,
                format!(
                    "\\{}{{{}}} cannot be resolved without macro expansion",
                    r.kind.command(),
                    r.argument
                ),
                DiagnosticKind::UnresolvableReference {
                    target: r.argument.clone(),
                },
            );
            return;
        }
        let base = match ProjectPath::normalize(&r.argument) {
            Ok(p) => p,
            Err(e) => {
                self.diag(
                    from,
                    r,
                    Severity::Error,
                    format!("\\{}{{{}}}: {e}", r.kind.command(), r.argument),
                    DiagnosticKind::InvalidPath {
                        target: r.argument.clone(),
                        error: e,
                    },
                );
                return;
            }
        };
        let (kind, candidates) = candidates(r.kind, &base);
        let Some(target) = candidates.iter().find(|c| self.exists(c)).cloned() else {
            let severity = if kind == FileKind::Graphic {
                Severity::Warning
            } else {
                Severity::Error
            };
            self.diag(
                from,
                r,
                severity,
                format!(
                    "\\{}{{{}}}: no file found (tried {})",
                    r.kind.command(),
                    r.argument,
                    candidates
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                DiagnosticKind::MissingFile {
                    target: r.argument.clone(),
                    tried: candidates,
                },
            );
            return;
        };
        if self.escapes_via_symlink(&target) {
            self.diag(
                from,
                r,
                Severity::Error,
                format!(
                    "\\{}{{{}}}: {target} is a symlink outside the project root",
                    r.kind.command(),
                    r.argument
                ),
                DiagnosticKind::EscapesRootViaSymlink { target },
            );
            return;
        }
        self.graph.edges.push(Edge {
            from: from.clone(),
            to: target.clone(),
            reference: r.clone(),
        });

        if let Some(pos) = self.stack.iter().position(|p| p == &target) {
            let chain = self.stack[pos..].to_vec();
            self.diag(
                from,
                r,
                Severity::Error,
                format!(
                    "\\{}{{{}}} closes an include cycle: {} -> {target}",
                    r.kind.command(),
                    r.argument,
                    chain
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" -> ")
                ),
                DiagnosticKind::Cycle { chain },
            );
            return;
        }
        if self.index.contains_key(&target) {
            return; // already discovered (diamond); the edge is recorded above
        }
        if kind == FileKind::Tex && self.stack.len() >= MAX_DEPTH {
            self.diag(
                from,
                r,
                Severity::Error,
                format!(
                    "\\{}{{{}}}: nesting deeper than {MAX_DEPTH} files",
                    r.kind.command(),
                    r.argument
                ),
                DiagnosticKind::DepthExceeded { target },
            );
            return;
        }
        match self.load(&target, kind) {
            Loaded::Ok(file) => {
                if kind == FileKind::Tex {
                    self.visit_loaded(file);
                } else {
                    if kind == FileKind::Bibliography && file.text.is_none() {
                        self.diag(
                            from,
                            r,
                            Severity::Warning,
                            format!("{target} is not valid UTF-8"),
                            DiagnosticKind::InvalidUtf8 {
                                path: target.clone(),
                            },
                        );
                    }
                    self.add_file(file);
                }
            }
            Loaded::Missing => {
                // Raced with an external deletion between exists() and read().
                self.diag(
                    from,
                    r,
                    Severity::Error,
                    format!(
                        "\\{}{{{}}}: {target} disappeared during discovery",
                        r.kind.command(),
                        r.argument
                    ),
                    DiagnosticKind::MissingFile {
                        target: r.argument.clone(),
                        tried: vec![target],
                    },
                );
            }
            Loaded::Error(e) => {
                self.diag(
                    from,
                    r,
                    Severity::Error,
                    format!("{target} could not be read: {e}"),
                    DiagnosticKind::ReadError {
                        path: target,
                        message: e.to_string(),
                    },
                );
            }
        }
    }
}

/// Candidate paths for a reference, in the order TeX-like tools try them.
pub fn candidates(kind: ReferenceKind, base: &ProjectPath) -> (FileKind, Vec<ProjectPath>) {
    match kind {
        ReferenceKind::Input | ReferenceKind::Include => {
            if base.extension() == Some("tex") {
                (FileKind::Tex, vec![base.clone()])
            } else {
                (
                    FileKind::Tex,
                    vec![base.with_appended_extension("tex"), base.clone()],
                )
            }
        }
        ReferenceKind::Bibliography => {
            if base.extension() == Some("bib") {
                (FileKind::Bibliography, vec![base.clone()])
            } else {
                (
                    FileKind::Bibliography,
                    vec![base.with_appended_extension("bib")],
                )
            }
        }
        ReferenceKind::AddBibResource => {
            if base.extension().is_some() {
                (FileKind::Bibliography, vec![base.clone()])
            } else {
                (
                    FileKind::Bibliography,
                    vec![base.with_appended_extension("bib")],
                )
            }
        }
        ReferenceKind::IncludeGraphics => {
            let has_known = base
                .extension()
                .is_some_and(|e| GRAPHIC_EXTENSIONS.iter().any(|g| g.eq_ignore_ascii_case(e)));
            if has_known {
                (FileKind::Graphic, vec![base.clone()])
            } else {
                (
                    FileKind::Graphic,
                    GRAPHIC_EXTENSIONS
                        .iter()
                        .map(|e| base.with_appended_extension(e))
                        .collect(),
                )
            }
        }
    }
}
