//! Applying an import: turning an [`ImportPreview`] plus explicit
//! per-file caller decisions into actual writes, with no silent overwrite.

use std::collections::HashMap;

use flashtex_project_files::{Digest, Expected};

use crate::bundle::Bundle;
use crate::error::BundleError;
use crate::preview::{FileOutcome, ImportPreview};
use crate::root::{map_write_error, ProjectRoot};

/// What the caller wants done with one previewed file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportDecision {
    /// Do not write this file.
    Skip,
    /// Write it: for a [`FileOutcome::New`] or [`FileOutcome::Conflict`],
    /// this is the caller's explicit consent to create or overwrite.
    Write,
}

/// What actually happened to one file during [`apply_import`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportAction {
    /// The bundle's bytes were written, verified by the rooted writer's own
    /// post-write hash check.
    Written { sha256: Digest, bytes: u64 },
    /// Nothing was written — either the caller chose `Skip`, or the file was
    /// already byte-identical ([`FileOutcome::Unchanged`]) so a write would
    /// have been a no-op.
    Skipped,
}

/// One file's actual outcome from [`apply_import`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOutcome {
    pub path: String,
    pub action: ImportAction,
}

/// Apply an import computed by [`crate::preview_import`], writing only what
/// `decisions` explicitly authorizes.
///
/// Rules, enforced here rather than merely documented:
///
/// - [`FileOutcome::Conflict`] — a file that already exists with different
///   content. Writing it requires `decisions[path] == Some(Write)`. If the
///   caller never mentioned that path at all, this is
///   [`BundleError::OverwriteNotDecided`] — an omission is never treated as
///   consent, unlike an explicit `Skip`. When `Write` is given, the actual
///   write uses `Expected::Hash(theirs)` (the exact hash observed at
///   preview time) as a compare-and-swap: if the target changed again
///   between preview and this call, the write is refused as
///   [`BundleError::ConcurrentModification`], never silently overwritten
///   with a value the caller never actually saw.
/// - [`FileOutcome::New`] — writes with `Expected::NewFile`, so a file that
///   appeared at that path between preview and apply is likewise refused as
///   [`BundleError::ConcurrentModification`] rather than clobbered. A
///   caller may still opt out with an explicit `Skip`; omitting a decision
///   for a `New` file defaults to writing it (there is nothing to
///   silently overwrite — the path was empty at preview time).
/// - [`FileOutcome::Unchanged`] — never written; the target already holds
///   these exact bytes, decision or not.
///
/// All writes for one call share a single project lock (from the target's
/// underlying `flashtex_project_files::ProjectRoot`), so no other in-contract
/// writer can interleave partway through this batch.
pub fn apply_import(
    bundle: &Bundle,
    preview: &ImportPreview,
    target: &ProjectRoot,
    decisions: &HashMap<String, ImportDecision>,
) -> Result<Vec<ImportOutcome>, BundleError> {
    let lock = target
        .files_root()
        .lock()
        .map_err(|e| map_write_error(target.as_path().to_string_lossy().as_ref(), e))?;

    let mut outcomes = Vec::with_capacity(preview.files.len());
    for fp in &preview.files {
        let decision = decisions.get(&fp.path).copied();
        let action = match fp.outcome {
            FileOutcome::Unchanged { .. } => ImportAction::Skipped,
            FileOutcome::Conflict { theirs, .. } => match decision {
                Some(ImportDecision::Write) => {
                    let contents = &bundle
                        .file(&fp.path)
                        .expect("preview built from this bundle")
                        .contents;
                    let path = ProjectRoot::normalize(&fp.path)?;
                    let receipt = lock
                        .save(&path, contents, Expected::Hash(theirs), false)
                        .map_err(|e| map_write_error(&fp.path, e))?;
                    ImportAction::Written {
                        sha256: receipt.sha256,
                        bytes: receipt.bytes,
                    }
                }
                Some(ImportDecision::Skip) => ImportAction::Skipped,
                None => return Err(BundleError::OverwriteNotDecided(fp.path.clone())),
            },
            FileOutcome::New => {
                if decision == Some(ImportDecision::Skip) {
                    ImportAction::Skipped
                } else {
                    let contents = &bundle
                        .file(&fp.path)
                        .expect("preview built from this bundle")
                        .contents;
                    let path = ProjectRoot::normalize(&fp.path)?;
                    let receipt = lock
                        .save(&path, contents, Expected::NewFile, false)
                        .map_err(|e| map_write_error(&fp.path, e))?;
                    ImportAction::Written {
                        sha256: receipt.sha256,
                        bytes: receipt.bytes,
                    }
                }
            }
        };
        outcomes.push(ImportOutcome {
            path: fp.path.clone(),
            action,
        });
    }
    Ok(outcomes)
}
