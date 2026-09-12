use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::BundleError;

/// A canonicalized project root that every read is rooted against.
///
/// `ProjectRoot` never lists a directory. It only ever resolves a single
/// caller-declared relative path at a time, and refuses to return anything
/// that would end up outside the root once symlinks are followed.
#[derive(Debug, Clone)]
pub struct ProjectRoot {
    canonical: PathBuf,
}

impl ProjectRoot {
    /// Canonicalize `path` and use it as the bundle root.
    ///
    /// Fails if `path` does not exist or is not a directory.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, BundleError> {
        let path = path.as_ref();
        let canonical = fs::canonicalize(path)
            .map_err(|e| BundleError::InvalidRoot(format!("{}: {e}", path.display())))?;
        if !canonical.is_dir() {
            return Err(BundleError::InvalidRoot(format!(
                "{} is not a directory",
                canonical.display()
            )));
        }
        Ok(Self { canonical })
    }

    /// The canonicalized root path.
    pub fn as_path(&self) -> &Path {
        &self.canonical
    }

    /// Resolve `relative` against the root and return its canonical path,
    /// refusing anything that is not a plain relative path confined to the
    /// root after symlink resolution.
    ///
    /// This is the single chokepoint every read goes through. It performs,
    /// in order:
    /// 1. Syntactic validation of `relative` (no absolute path, no `..`,
    ///    no malformed components) — rejected before touching the
    ///    filesystem at all, so the result does not depend on whether a
    ///    matching file happens to exist outside the root.
    /// 2. Filesystem resolution: the joined path is canonicalized (which
    ///    follows symlinks), and the result must still start with the
    ///    canonical root.
    /// 3. A regular-file check: directories are refused, since this crate
    ///    never walks them.
    pub fn resolve(&self, relative: &str) -> Result<PathBuf, BundleError> {
        validate_relative_path(relative)?;
        let joined = self.canonical.join(relative);
        let resolved = fs::canonicalize(&joined).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => BundleError::NotFound(relative.to_string()),
            _ => BundleError::Io(format!("{relative}: {e}")),
        })?;
        if !resolved.starts_with(&self.canonical) {
            return Err(BundleError::SymlinkEscapesRoot(relative.to_string()));
        }
        if !resolved.is_file() {
            return Err(BundleError::NotAFile(relative.to_string()));
        }
        Ok(resolved)
    }

    /// Read the bytes of `relative`, rooted per [`ProjectRoot::resolve`].
    pub fn read_rooted(&self, relative: &str) -> Result<Vec<u8>, BundleError> {
        let resolved = self.resolve(relative)?;
        fs::read(&resolved).map_err(|e| BundleError::Io(format!("{relative}: {e}")))
    }
}

/// Syntactic validation of a caller-declared bundle path, independent of
/// the filesystem. Every rejection here happens without a single I/O call,
/// so it is total: it rejects `..` and absolute paths whether or not
/// anything exists at that location.
pub fn validate_relative_path(path: &str) -> Result<(), BundleError> {
    if path.is_empty() {
        return Err(BundleError::EmptyPath);
    }
    if path.contains('\0') {
        return Err(BundleError::MalformedPath(format!(
            "{path:?}: contains a NUL byte"
        )));
    }
    if path.starts_with('/') {
        return Err(BundleError::AbsolutePath(path.to_string()));
    }
    for component in path.split('/') {
        if component.is_empty() {
            return Err(BundleError::MalformedPath(format!(
                "{path:?}: empty path component"
            )));
        }
        if component == ".." {
            return Err(BundleError::PathTraversal(path.to_string()));
        }
        if component == "." {
            return Err(BundleError::MalformedPath(format!(
                "{path:?}: '.' component not allowed"
            )));
        }
    }
    Ok(())
}
