//! Applying an import: turning an [`ImportPreview`] plus explicit
//! per-file caller decisions into actual writes, with no silent overwrite
//! **and** batch recoverability: if any file's write fails partway through
//! a multi-file call, every write already committed earlier in that same
//! call is undone before the error is returned (see "Batch recovery"
//! below).

use std::collections::HashMap;

use flashtex_project_files::{Digest, Expected, ProjectLock};

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
///
/// # Batch recovery
///
/// The reused writer (`flashtex_project_files::ProjectLock::save`) makes
/// exactly one file's write atomic — it does not offer a multi-file
/// transaction, so this function builds recoverability on top of it rather
/// than assuming it: as each file is written, this function first records
/// enough to undo *that one write* (the exact bytes it is about to
/// overwrite, or the fact that the path did not exist before). If a later
/// file in the same call then fails — a decision was missing, the target
/// raced out from under a compare-and-swap, an I/O error, anything — every
/// write already committed earlier in *this same call* is rolled back, in
/// reverse order, before the error is returned: a file this call created is
/// removed again; a file this call overwrote is restored to the exact bytes
/// it held before this call touched it. The failing path itself, and every
/// path after it in iteration order, were never written by this call, so
/// there is nothing to undo for them.
///
/// This rollback runs under the same exclusive project lock already held
/// for the forward writes, so in the ordinary case — no writer bypassing the
/// lock, no disk exhaustion — it always fully succeeds, and the error
/// returned is the same typed [`BundleError`] the failing file itself
/// produced (e.g. [`BundleError::OverwriteNotDecided`] or
/// [`BundleError::ConcurrentModification`]): callers that already match on
/// those variants keep working unchanged. Only in the rare case that
/// rollback *itself* cannot fully complete — a genuine double fault, such as
/// an out-of-contract writer bypassing the lock to touch a path this call
/// had just written, or the disk filling during the restore — does this
/// return the distinct [`BundleError::RollbackIncomplete`], which names
/// exactly which paths are left in the state this call wrote them to (never
/// silently left inconsistent without saying so).
///
/// Either way, this function never partially applies a batch without
/// reporting it: the only two outcomes are "every file this call wrote is
/// back to its pre-call content or absence" (the typed cause is returned
/// directly) or "here is exactly what could not be undone"
/// ([`BundleError::RollbackIncomplete`]).
///
/// One documented exception, which rollback does **not** undo: writing a
/// new file under a directory the target did not have yet creates that
/// directory (the rooted writer creates missing parents). Rollback removes
/// the file but leaves the now-empty directory in place, and does not
/// report it as a rollback failure. No file content is affected — an empty
/// directory is not a write this call can distinguish from one a
/// concurrent in-contract writer made — so "target exactly as before" is
/// true of file contents, not of the directory tree. See
/// `tests/recovery.rs::rollback_removes_the_file_but_leaves_the_directory_it_created`.
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

    // What it takes to undo one already-committed write in this call, in
    // the order the writes happened (undone in reverse on failure).
    let mut undo_log: Vec<(String, Undo)> = Vec::new();
    let mut outcomes = Vec::with_capacity(preview.files.len());

    for fp in &preview.files {
        let decision = decisions.get(&fp.path).copied();
        match apply_one(bundle, target, &lock, fp, decision) {
            Ok((action, undo)) => {
                if let Some(undo) = undo {
                    undo_log.push((fp.path.clone(), undo));
                }
                outcomes.push(ImportOutcome {
                    path: fp.path.clone(),
                    action,
                });
            }
            Err(cause) => {
                let failures = roll_back(target, &lock, undo_log);
                return Err(if failures.is_empty() {
                    cause
                } else {
                    BundleError::RollbackIncomplete {
                        original_cause: Box::new(cause),
                        left_in_written_state: failures,
                    }
                });
            }
        }
    }
    Ok(outcomes)
}

/// How to undo one write this call already committed.
enum Undo {
    /// This call created the path (it did not exist before); undo by
    /// removing it, but only if it still holds exactly what this call
    /// wrote — never remove content nobody here wrote.
    Remove { written_sha256: Digest },
    /// This call overwrote the path; undo by restoring these exact
    /// pre-write bytes, compare-and-swapped against exactly what this call
    /// wrote (so a further double-fault is reported, never silently
    /// clobbered a second time).
    Restore {
        original_bytes: Vec<u8>,
        written_sha256: Digest,
    },
}

/// Apply the decision for one previewed file, returning what happened plus
/// (if this call wrote anything) how to undo it.
fn apply_one(
    bundle: &Bundle,
    target: &ProjectRoot,
    lock: &ProjectLock<'_>,
    fp: &crate::preview::FilePreview,
    decision: Option<ImportDecision>,
) -> Result<(ImportAction, Option<Undo>), BundleError> {
    match fp.outcome {
        FileOutcome::Unchanged { .. } => Ok((ImportAction::Skipped, None)),
        FileOutcome::Conflict { theirs, .. } => match decision {
            Some(ImportDecision::Write) => {
                let contents = &bundle
                    .file(&fp.path)
                    .expect("preview built from this bundle")
                    .contents;
                let path = ProjectRoot::normalize(&fp.path)?;
                // Snapshot exactly what is about to be overwritten so a
                // later failure elsewhere in this batch can restore it.
                let before = target.read_rooted_optional(&fp.path)?;
                let receipt = lock
                    .save(&path, contents, Expected::Hash(theirs), false)
                    .map_err(|e| map_write_error(&fp.path, e))?;
                let undo = Some(match before {
                    Some(existing) => Undo::Restore {
                        original_bytes: existing.bytes,
                        written_sha256: receipt.sha256,
                    },
                    // Raced away between preview and this read, yet the
                    // compare-and-swap above still matched `theirs`: treat
                    // it like a New write for rollback purposes.
                    None => Undo::Remove {
                        written_sha256: receipt.sha256,
                    },
                });
                Ok((
                    ImportAction::Written {
                        sha256: receipt.sha256,
                        bytes: receipt.bytes,
                    },
                    undo,
                ))
            }
            Some(ImportDecision::Skip) => Ok((ImportAction::Skipped, None)),
            None => Err(BundleError::OverwriteNotDecided(fp.path.clone())),
        },
        FileOutcome::New => {
            if decision == Some(ImportDecision::Skip) {
                Ok((ImportAction::Skipped, None))
            } else {
                let contents = &bundle
                    .file(&fp.path)
                    .expect("preview built from this bundle")
                    .contents;
                let path = ProjectRoot::normalize(&fp.path)?;
                let receipt = lock
                    .save(&path, contents, Expected::NewFile, false)
                    .map_err(|e| map_write_error(&fp.path, e))?;
                Ok((
                    ImportAction::Written {
                        sha256: receipt.sha256,
                        bytes: receipt.bytes,
                    },
                    Some(Undo::Remove {
                        written_sha256: receipt.sha256,
                    }),
                ))
            }
        }
    }
}

/// Undo every recorded write in `undo_log`, most recent first. Returns the
/// (path, reason) pairs that could **not** be undone — empty means every
/// write was cleanly rolled back (the common case, since this runs under
/// the same exclusive lock the forward writes used).
fn roll_back(
    target: &ProjectRoot,
    lock: &ProjectLock<'_>,
    undo_log: Vec<(String, Undo)>,
) -> Vec<(String, String)> {
    let mut failures = Vec::new();
    for (path, undo) in undo_log.into_iter().rev() {
        if let Err(reason) = undo_one(target, lock, &path, undo) {
            failures.push((path, reason));
        }
    }
    failures
}

fn undo_one(target: &ProjectRoot, lock: &ProjectLock<'_>, path: &str, undo: Undo) -> Result<(), String> {
    let normalized = ProjectRoot::normalize(path).map_err(|e| e.to_string())?;
    match undo {
        Undo::Remove { written_sha256 } => {
            match target.read_rooted_optional(path).map_err(|e| e.to_string())? {
                None => Ok(()), // already gone; the rollback goal is met
                Some(current) if current.sha256 == written_sha256 => lock
                    .remove(&normalized)
                    .map(|_| ())
                    .map_err(|e| map_write_error(path, e).to_string()),
                Some(_) => Err(format!(
                    "{path}: content changed since this call wrote it; left in place rather than removing the wrong bytes"
                )),
            }
        }
        Undo::Restore {
            original_bytes,
            written_sha256,
        } => lock
            .save(
                &normalized,
                &original_bytes,
                Expected::Hash(written_sha256),
                false,
            )
            .map(|_| ())
            .map_err(|e| map_write_error(path, e).to_string()),
    }
}
