//! A bounded document root that every asset path is resolved against.
//!
//! [`AssetRoot`] is the single gate between a caller-supplied relative path
//! and the filesystem. It rejects, with a distinct typed error, every way a
//! path could name something outside the configured root:
//!
//! - an absolute path (`/etc/passwd`, `C:\evil`),
//! - lexical traversal (`../../etc/passwd`, `a/../../b`),
//! - a symlink — at any component, not just the last — whose resolved
//!   target lands outside the root.
//!
//! The first two are rejected lexically, before touching the filesystem. The
//! third can only be caught by canonicalizing the resolved candidate and
//! checking the real, symlink-free path still starts with the real root.

use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Why a path was refused, or why the root itself is unusable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootError {
    /// The configured root does not exist.
    RootNotFound(PathBuf),
    /// The configured root exists but is not a directory.
    RootNotADirectory(PathBuf),
    /// The requested path was absolute (or carried a Windows drive prefix).
    AbsolutePath(PathBuf),
    /// The requested path contained a `..` component.
    PathTraversal(PathBuf),
    /// The path resolved, once symlinks are followed, to somewhere outside
    /// the root. This is the case a purely lexical check cannot catch.
    SymlinkEscape(PathBuf),
    /// The path is empty or every component was `.`.
    EmptyPath,
    /// The resolved path does not exist on disk.
    NotFound(PathBuf),
    /// Some other I/O failure while resolving the path.
    Io(String),
}

impl fmt::Display for RootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RootError::RootNotFound(p) => write!(f, "asset root does not exist: {}", p.display()),
            RootError::RootNotADirectory(p) => {
                write!(f, "asset root is not a directory: {}", p.display())
            }
            RootError::AbsolutePath(p) => {
                write!(f, "asset path must be root-relative, not absolute: {}", p.display())
            }
            RootError::PathTraversal(p) => {
                write!(f, "asset path contains '..' and was rejected: {}", p.display())
            }
            RootError::SymlinkEscape(p) => write!(
                f,
                "asset path resolves outside the document root via a symlink: {}",
                p.display()
            ),
            RootError::EmptyPath => write!(f, "asset path is empty"),
            RootError::NotFound(p) => write!(f, "asset path does not exist: {}", p.display()),
            RootError::Io(msg) => write!(f, "asset root I/O error: {msg}"),
        }
    }
}

impl std::error::Error for RootError {}

/// A document root that bounds every asset resolution beneath it.
///
/// Constructed once from a directory that must already exist; every
/// subsequent [`AssetRoot::resolve`] call is checked against it.
#[derive(Debug, Clone)]
pub struct AssetRoot {
    /// The canonical (symlink-free, absolute) form of the configured root.
    canonical_root: PathBuf,
}

impl AssetRoot {
    /// Opens `root` as a document root. `root` must exist and be a
    /// directory; it is canonicalized once up front so every later
    /// resolution compares against a stable, symlink-free base.
    pub fn new(root: impl AsRef<Path>) -> Result<Self, RootError> {
        let root = root.as_ref();
        let metadata = fs::metadata(root).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => RootError::RootNotFound(root.to_path_buf()),
            _ => RootError::Io(e.to_string()),
        })?;
        if !metadata.is_dir() {
            return Err(RootError::RootNotADirectory(root.to_path_buf()));
        }
        let canonical_root = fs::canonicalize(root).map_err(|e| RootError::Io(e.to_string()))?;
        Ok(Self { canonical_root })
    }

    /// The canonical root directory this instance bounds asset loads to.
    pub fn root(&self) -> &Path {
        &self.canonical_root
    }

    /// Resolves `relative` against the root, returning the real
    /// (symlink-free) path on disk if, and only if, it names something
    /// inside the root. `relative` must be a relative path with no `..`
    /// component.
    pub fn resolve(&self, relative: impl AsRef<Path>) -> Result<PathBuf, RootError> {
        let relative = relative.as_ref();
        let mut had_component = false;
        for component in relative.components() {
            had_component = true;
            match component {
                Component::Normal(_) => {}
                Component::CurDir => {}
                Component::ParentDir => {
                    return Err(RootError::PathTraversal(relative.to_path_buf()));
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(RootError::AbsolutePath(relative.to_path_buf()));
                }
            }
        }
        if !had_component || relative.as_os_str().is_empty() {
            return Err(RootError::EmptyPath);
        }

        // Lexically joining is safe here: we have already rejected any
        // `..` and any absolute/prefix component above, so `candidate` is
        // guaranteed to be a lexical descendant of `canonical_root`.
        let candidate = self.canonical_root.join(relative);

        let real = fs::canonicalize(&candidate).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => RootError::NotFound(candidate.clone()),
            _ => RootError::Io(e.to_string()),
        })?;

        if real.starts_with(&self.canonical_root) {
            Ok(real)
        } else {
            Err(RootError::SymlinkEscape(relative.to_path_buf()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn write_file(path: &Path, bytes: &[u8]) {
        let mut f = File::create(path).unwrap();
        f.write_all(bytes).unwrap();
    }

    #[test]
    fn resolves_a_plain_file_inside_the_root() {
        let dir = tempfile::tempdir().unwrap();
        write_file(&dir.path().join("a.png"), b"stub");
        let root = AssetRoot::new(dir.path()).unwrap();
        let resolved = root.resolve("a.png").unwrap();
        assert_eq!(resolved, fs::canonicalize(dir.path().join("a.png")).unwrap());
    }

    #[test]
    fn resolves_a_nested_file_inside_the_root() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("figs/ch1")).unwrap();
        write_file(&dir.path().join("figs/ch1/a.png"), b"stub");
        let root = AssetRoot::new(dir.path()).unwrap();
        assert!(root.resolve("figs/ch1/a.png").is_ok());
    }

    #[test]
    fn rejects_parent_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root.resolve("../../etc/passwd").unwrap_err();
        assert!(matches!(err, RootError::PathTraversal(_)), "{err:?}");
    }

    #[test]
    fn rejects_traversal_buried_in_the_middle_of_a_path() {
        let dir = tempfile::tempdir().unwrap();
        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root.resolve("figs/../../etc/passwd").unwrap_err();
        assert!(matches!(err, RootError::PathTraversal(_)), "{err:?}");
    }

    #[test]
    fn rejects_absolute_paths() {
        let dir = tempfile::tempdir().unwrap();
        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root.resolve("/etc/passwd").unwrap_err();
        assert!(matches!(err, RootError::AbsolutePath(_)), "{err:?}");
    }

    #[test]
    #[cfg(unix)]
    fn rejects_symlinks_that_escape_the_root() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        write_file(&outside.path().join("secret.png"), b"not yours");

        std::os::unix::fs::symlink(
            outside.path().join("secret.png"),
            dir.path().join("looks-local.png"),
        )
        .unwrap();

        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root.resolve("looks-local.png").unwrap_err();
        assert!(matches!(err, RootError::SymlinkEscape(_)), "{err:?}");
    }

    #[test]
    #[cfg(unix)]
    fn rejects_escape_via_a_symlinked_directory_component() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::create_dir_all(outside.path().join("real")).unwrap();
        write_file(&outside.path().join("real/img.png"), b"not yours");

        std::os::unix::fs::symlink(outside.path().join("real"), dir.path().join("figs")).unwrap();

        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root.resolve("figs/img.png").unwrap_err();
        assert!(matches!(err, RootError::SymlinkEscape(_)), "{err:?}");
    }

    #[test]
    fn allows_a_symlink_that_stays_inside_the_root() {
        let dir = tempfile::tempdir().unwrap();
        write_file(&dir.path().join("real.png"), b"stub");
        #[cfg(unix)]
        std::os::unix::fs::symlink(dir.path().join("real.png"), dir.path().join("alias.png"))
            .unwrap();
        #[cfg(unix)]
        {
            let root = AssetRoot::new(dir.path()).unwrap();
            assert!(root.resolve("alias.png").is_ok());
        }
    }

    #[test]
    fn rejects_missing_root() {
        let err = AssetRoot::new("/does/not/exist/hopefully").unwrap_err();
        assert!(matches!(err, RootError::RootNotFound(_)));
    }

    #[test]
    fn rejects_root_that_is_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("not-a-dir");
        write_file(&file, b"x");
        let err = AssetRoot::new(&file).unwrap_err();
        assert!(matches!(err, RootError::RootNotADirectory(_)));
    }

    #[test]
    fn rejects_empty_path() {
        let dir = tempfile::tempdir().unwrap();
        let root = AssetRoot::new(dir.path()).unwrap();
        assert_eq!(root.resolve("").unwrap_err(), RootError::EmptyPath);
    }

    #[test]
    fn rejects_unicode_path_that_does_not_exist() {
        let dir = tempfile::tempdir().unwrap();
        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root.resolve("figs/日本語/😀.png").unwrap_err();
        assert!(matches!(err, RootError::NotFound(_)), "{err:?}");
    }
}
