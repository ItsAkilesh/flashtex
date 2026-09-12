use std::collections::HashSet;

use sha2::{Digest, Sha256};

use crate::error::BundleError;
use crate::root::ProjectRoot;

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
    pub sha256: [u8; 32],
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
    pub fn manifest_sha256(&self) -> [u8; 32] {
        sha256(&self.manifest_bytes())
    }

    /// [`Bundle::manifest_sha256`] as lowercase hex.
    pub fn manifest_hex(&self) -> String {
        hex(&self.manifest_sha256())
    }
}

/// Build a bundle from exactly `entries`, resolved through `root`.
///
/// No directory is ever listed or scanned: each entry is resolved one at a
/// time through [`ProjectRoot::read_rooted`], which rejects traversal,
/// absolute paths and symlinks escaping the root. The result is sorted by
/// path before being returned, so the same `entries` (in any order) always
/// produce the same [`Bundle::manifest_bytes`].
pub fn build_bundle(root: &ProjectRoot, entries: &[BundleEntry]) -> Result<Bundle, BundleError> {
    let mut seen = HashSet::with_capacity(entries.len());
    let mut files = Vec::with_capacity(entries.len());
    for entry in entries {
        if !seen.insert(entry.path.clone()) {
            return Err(BundleError::DuplicatePath(entry.path.clone()));
        }
        let contents = root.read_rooted(&entry.path)?;
        let digest = sha256(&contents);
        files.push(BundleFile {
            path: entry.path.clone(),
            sha256: digest,
            size: contents.len() as u64,
            contents,
        });
    }
    files.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
    Ok(Bundle { files })
}

fn sha256(data: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(data);
    digest.into()
}

fn hex(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
