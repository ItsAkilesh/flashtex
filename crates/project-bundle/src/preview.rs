//! Import preview: what would happen if a [`Bundle`] were imported into a
//! target [`ProjectRoot`], computed without writing anything.

use flashtex_project_files::Digest;

use crate::bundle::Bundle;
use crate::error::BundleError;
use crate::root::ProjectRoot;

/// What importing one bundle file into the target would do.
///
/// Every variant carries the full SHA-256 on whichever side(s) are
/// relevant, so a caller can distinguish "identical, nothing to do" from
/// "differs, needs a decision" without re-hashing anything itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOutcome {
    /// Not present in the target: importing would write a new file.
    New,
    /// Present in the target with byte-identical content (equal SHA-256):
    /// importing would be a no-op.
    Unchanged { sha256: Digest },
    /// Present in the target with *different* content: importing this path
    /// requires an explicit overwrite decision (see [`crate::ImportDecision`]).
    /// `ours` is the bundle file's hash, `theirs` the target's.
    Conflict {
        ours: Digest,
        theirs: Digest,
        theirs_size: u64,
    },
}

impl FileOutcome {
    pub fn is_conflict(&self) -> bool {
        matches!(self, FileOutcome::Conflict { .. })
    }
}

/// One bundle file's outcome against the target, at the path it would land
/// at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePreview {
    pub path: String,
    pub outcome: FileOutcome,
}

/// The full report of what importing a [`Bundle`] into a target
/// [`ProjectRoot`] would do, sorted by path (same ordering rule as
/// [`Bundle::files`](crate::Bundle::files)).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportPreview {
    pub files: Vec<FilePreview>,
}

impl ImportPreview {
    pub fn new_paths(&self) -> impl Iterator<Item = &str> {
        self.files
            .iter()
            .filter(|f| matches!(f.outcome, FileOutcome::New))
            .map(|f| f.path.as_str())
    }

    pub fn unchanged_paths(&self) -> impl Iterator<Item = &str> {
        self.files
            .iter()
            .filter(|f| matches!(f.outcome, FileOutcome::Unchanged { .. }))
            .map(|f| f.path.as_str())
    }

    pub fn conflicts(&self) -> impl Iterator<Item = &FilePreview> {
        self.files.iter().filter(|f| f.outcome.is_conflict())
    }

    pub fn has_conflicts(&self) -> bool {
        self.files.iter().any(|f| f.outcome.is_conflict())
    }
}

/// Compute what importing `bundle` into `target` would do, performing no
/// writes.
///
/// Each bundle file is read from `target` (rooted, bounded, exactly as
/// `flashtex_project_files::ProjectRoot::read` behaves — no lock is ever
/// taken and `save`/`remove` are never called from this function), and
/// classified: missing → [`FileOutcome::New`], present with an equal
/// SHA-256 → [`FileOutcome::Unchanged`], present with a different SHA-256 →
/// [`FileOutcome::Conflict`] carrying both full hashes.
pub fn preview_import(bundle: &Bundle, target: &ProjectRoot) -> Result<ImportPreview, BundleError> {
    let mut files = Vec::with_capacity(bundle.files.len());
    for file in &bundle.files {
        let existing = target.read_rooted_optional(&file.path)?;
        let outcome = match existing {
            None => FileOutcome::New,
            Some(existing) if existing.sha256 == file.sha256 => FileOutcome::Unchanged {
                sha256: file.sha256,
            },
            Some(existing) => FileOutcome::Conflict {
                ours: file.sha256,
                theirs: existing.sha256,
                theirs_size: existing.size,
            },
        };
        files.push(FilePreview {
            path: file.path.clone(),
            outcome,
        });
    }
    files.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
    Ok(ImportPreview { files })
}
