//! Deterministic, explicitly-scoped project export bundle.
//!
//! This crate builds an export bundle from a caller-supplied list of paths
//! (a [`BundleEntry`] per file) rooted at a [`ProjectRoot`], and produces a
//! [`Bundle`] whose SHA-256 manifest ([`Bundle::manifest_bytes`],
//! [`Bundle::manifest_sha256`]) is byte-identical across runs regardless of
//! filesystem iteration order, hash-map order, or the order entries were
//! supplied in.
//!
//! Two properties are load-bearing and deliberately narrow:
//!
//! - **Rooted reads.** Every read goes through
//!   [`ProjectRoot::resolve`]/[`ProjectRoot::read_rooted`]. A `..`
//!   component, an absolute path, or a symlink that resolves outside the
//!   root is rejected with a typed [`BundleError`] variant, never silently
//!   clamped or ignored.
//! - **No implicit discovery.** The crate never calls `read_dir`, globs, or
//!   otherwise walks a directory to decide what belongs in a bundle. If the
//!   caller did not name a path in the [`BundleEntry`] list, it is not in
//!   the bundle — including files sitting right next to ones that are.
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

mod bundle;
mod error;
mod root;

pub use bundle::{build_bundle, Bundle, BundleEntry, BundleFile};
pub use error::BundleError;
pub use root::{validate_relative_path, ProjectRoot};
