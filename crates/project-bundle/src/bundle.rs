use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use flashtex_project_files::{Digest, sha256, sha256_to_hex as hex};

use crate::error::BundleError;
use crate::root::ProjectRoot;

/// Default cap on the number of entries in one bundle spec.
pub const DEFAULT_MAX_ENTRIES: usize = 100_000;

/// Default cap on the running total of read file bytes in one bundle.
pub const DEFAULT_MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;

/// Bounds enforced while building a [`Bundle`]. Every field has a typed
/// error (see [`BundleError::TooManyEntries`], [`BundleError::TotalBytesExceeded`],
/// [`BundleError::FileTooLarge`]) — nothing is silently truncated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundleLimits {
    pub max_entries: usize,
    pub max_total_bytes: u64,
}

impl Default for BundleLimits {
    fn default() -> Self {
        Self {
            max_entries: DEFAULT_MAX_ENTRIES,
            max_total_bytes: DEFAULT_MAX_TOTAL_BYTES,
        }
    }
}

/// One file the caller wants in the bundle, named by its path relative to
/// the [`ProjectRoot`].
///
/// This is the entire input contract: the crate includes exactly the
/// entries it is given and nothing else. It never walks or globs a
/// directory to decide what belongs in a bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleEntry {
    pub path: String,
}

impl BundleEntry {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

/// One resolved, hashed file in a built [`Bundle`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleFile {
    pub path: String,
    pub sha256: Digest,
    pub size: u64,
    pub contents: Vec<u8>,
}

/// A built bundle: the caller's declared entries, resolved, read and
/// hashed, held in one fixed total order.
///
/// The order is a plain byte-wise sort of `path` — never filesystem
/// iteration order, never hash-map order, and independent of the order the
/// caller listed entries in. [`Bundle::manifest_bytes`] is therefore
/// byte-identical for the same (root, entry-set) pair no matter how the
/// entries were supplied or in what order the files were read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bundle {
    pub files: Vec<BundleFile>,
}

impl Bundle {
    /// Serialize the manifest as sorted `<sha256-hex>  <size>  <path>\n`
    /// lines, one per file. This is the exact byte sequence hashed by
    /// [`Bundle::manifest_sha256`] and the exact byte sequence that must be
    /// reproducible across runs and input orderings.
    pub fn manifest_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for file in &self.files {
            out.extend_from_slice(hex(&file.sha256).as_bytes());
            out.extend_from_slice(b"  ");
            out.extend_from_slice(file.size.to_string().as_bytes());
            out.extend_from_slice(b"  ");
            out.extend_from_slice(file.path.as_bytes());
            out.push(b'\n');
        }
        out
    }

    /// SHA-256 over [`Bundle::manifest_bytes`]: one fixed-size fingerprint
    /// for the whole bundle.
    pub fn manifest_sha256(&self) -> Digest {
        sha256(&self.manifest_bytes())
    }

    /// [`Bundle::manifest_sha256`] as lowercase hex.
    pub fn manifest_hex(&self) -> String {
        hex(&self.manifest_sha256())
    }

    /// Look up one file's built record by its declared path.
    pub fn file(&self, path: &str) -> Option<&BundleFile> {
        self.files.iter().find(|f| f.path == path)
    }
}

/// Build a bundle from exactly `entries`, resolved through `root`, with
/// [`BundleLimits::default`] applied.
pub fn build_bundle(root: &ProjectRoot, entries: &[BundleEntry]) -> Result<Bundle, BundleError> {
    build_bundle_with_limits(root, entries, &BundleLimits::default())
}

/// Build a bundle from exactly `entries`, resolved through `root`.
///
/// No directory is ever listed or scanned: each entry is resolved one at a
/// time through [`ProjectRoot::read_rooted`], which rejects traversal,
/// absolute paths and symlinks via the rooted reader in
/// `flashtex_project_files`. The result is sorted by path before being
/// returned, so the same `entries` (in any order) always produce the same
/// [`Bundle::manifest_bytes`].
///
/// Bounded: more than `limits.max_entries` entries is
/// [`BundleError::TooManyEntries`] before any file is read; the running
/// total of read bytes exceeding `limits.max_total_bytes` is
/// [`BundleError::TotalBytesExceeded`], checked after every file so a
/// caller never waits for a bundle that was always going to be rejected.
/// Each individual file is additionally bounded by the `ProjectRoot`'s own
/// per-file limit ([`BundleError::FileTooLarge`]).
///
/// Two declared paths that are byte-identical are [`BundleError::DuplicatePath`].
/// Two declared paths that are byte-*different* but resolve to the same
/// underlying file (the APFS Unicode-normalization hazard — see
/// [`BundleError::AmbiguousPath`]) are also rejected rather than silently
/// admitted as two bundle entries that would, in fact, collide.
pub fn build_bundle_with_limits(
    root: &ProjectRoot,
    entries: &[BundleEntry],
    limits: &BundleLimits,
) -> Result<Bundle, BundleError> {
    if entries.len() > limits.max_entries {
        return Err(BundleError::TooManyEntries {
            limit: limits.max_entries,
            actual: entries.len(),
        });
    }
    let mut seen = HashSet::with_capacity(entries.len());
    let mut seen_identity: HashMap<PathBuf, String> = HashMap::with_capacity(entries.len());
    let mut files = Vec::with_capacity(entries.len());
    let mut total_bytes: u64 = 0;
    for entry in entries {
        if !seen.insert(entry.path.clone()) {
            return Err(BundleError::DuplicatePath(entry.path.clone()));
        }
        let read = root
            .read_rooted_optional(&entry.path)?
            .ok_or_else(|| BundleError::NotFound(entry.path.clone()))?;
        if let Some(identity) = root.canonical_identity(&entry.path)
            && let Some(first) = seen_identity.insert(identity, entry.path.clone())
        {
            return Err(BundleError::AmbiguousPath {
                first,
                second: entry.path.clone(),
            });
        }
        total_bytes = total_bytes.saturating_add(read.size);
        if total_bytes > limits.max_total_bytes {
            return Err(BundleError::TotalBytesExceeded {
                limit: limits.max_total_bytes,
                actual: total_bytes,
            });
        }
        files.push(BundleFile {
            path: entry.path.clone(),
            sha256: read.sha256,
            size: read.size,
            contents: read.bytes,
        });
    }
    files.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
    Ok(Bundle { files })
}
