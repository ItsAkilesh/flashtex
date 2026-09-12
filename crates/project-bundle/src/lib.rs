//! Deterministic, explicitly-scoped project export bundle, plus a bounded
//! import preview and no-clobber apply against another project root.
//!
//! This crate builds an export bundle from a caller-supplied list of paths
//! (a [`BundleEntry`] per file) rooted at a [`ProjectRoot`], and produces a
//! [`Bundle`] whose SHA-256 manifest ([`Bundle::manifest_bytes`],
//! [`Bundle::manifest_sha256`]) is byte-identical across runs regardless of
//! filesystem iteration order, hash-map order, or the order entries were
//! supplied in. It can then [`preview_import`] that bundle against a target
//! [`ProjectRoot`] (read-only — see [`preview`]) and, given the caller's
//! explicit per-file [`ImportDecision`]s, [`apply_import`] it with no silent
//! overwrite (see [`apply`]).
//!
//! Properties that are load-bearing and deliberately narrow:
//!
//! - **Rooted reads and writes, via reuse, not reimplementation.** Every
//!   read and write goes through `flashtex_project_files::ProjectRoot`
//!   (issue #18's `openat(O_NOFOLLOW)`-based rooted reader/writer) —
//!   [`ProjectRoot`] here is a thin typed adapter over it, not a second
//!   implementation. An absolute path, a `..` that would leave the root, or
//!   any symlink component is rejected with a typed [`BundleError`]
//!   variant, never silently clamped or ignored. See [`root`] for exactly
//!   what is reused and the one behavior that is now *stricter* than rev 1
//!   (every symlink is refused, not just ones that escape).
//! - **No implicit discovery.** The crate never calls `read_dir`, globs, or
//!   otherwise walks a directory to decide what belongs in a bundle. If the
//!   caller did not name a path in the [`BundleEntry`] list, it is not in
//!   the bundle — including files sitting right next to ones that are.
//! - **Preview never writes.** [`preview_import`] only calls the target
//!   root's `read` — it never takes the project lock and never calls
//!   `save`/`remove`. Every file it inspects gets a full SHA-256 on both
//!   sides, so "identical" and "differs" are distinguished by content, not
//!   by size or mtime.
//! - **No silent overwrite.** [`apply_import`] writes a [`FileOutcome::Conflict`]
//!   file only when the caller's `decisions` map explicitly says `Write`
//!   for that path — an *absent* decision is [`BundleError::OverwriteNotDecided`],
//!   never treated as consent — and even then the write is a
//!   compare-and-swap against the exact hash seen at preview time, so a
//!   target that changed again in between is refused as
//!   [`BundleError::ConcurrentModification`] rather than clobbered.
//! - **Bounded.** [`build_bundle_with_limits`] caps entry count
//!   ([`BundleError::TooManyEntries`]) and running total bytes
//!   ([`BundleError::TotalBytesExceeded`]); each individual file is bounded
//!   by the `ProjectRoot`'s own read limit ([`BundleError::FileTooLarge`]).
//!
//! # Example
//!
//! ```
//! use flashtex_project_bundle::{build_bundle, BundleEntry, ProjectRoot};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let dir = tempfile_root()?;
//! let root = ProjectRoot::new(&dir)?;
//! let entries = [BundleEntry::new("main.tex")];
//! let bundle = build_bundle(&root, &entries)?;
//! let manifest = bundle.manifest_bytes();
//! assert!(!manifest.is_empty());
//! # std::fs::remove_dir_all(&dir).ok();
//! # Ok(())
//! # }
//! # fn tempfile_root() -> std::io::Result<std::path::PathBuf> {
//! #     let dir = std::env::temp_dir().join(format!("pb-doctest-{}", std::process::id()));
//! #     std::fs::create_dir_all(&dir)?;
//! #     std::fs::write(dir.join("main.tex"), b"\\documentclass{article}")?;
//! #     Ok(dir)
//! # }
//! ```

mod apply;
mod bundle;
mod error;
mod preview;
mod root;

pub use apply::{apply_import, ImportAction, ImportDecision, ImportOutcome};
pub use bundle::{
    build_bundle, build_bundle_with_limits, Bundle, BundleEntry, BundleFile, BundleLimits,
    DEFAULT_MAX_ENTRIES, DEFAULT_MAX_TOTAL_BYTES,
};
pub use error::BundleError;
pub use preview::{preview_import, FileOutcome, FilePreview, ImportPreview};
pub use root::{validate_relative_path, ProjectRoot, RootedFile, DEFAULT_FILE_LIMIT};
