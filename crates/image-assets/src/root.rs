//! A bounded document root that every asset path is resolved against.
//!
//! [`AssetRoot`] is a thin adapter over the workspace's existing rooted,
//! symlink-refusing reader — [`flashtex_project_files::ProjectRoot`] — rather
//! than a second hand-rolled implementation. Every caller-supplied relative
//! path is:
//!
//! 1. lexically normalized by [`flashtex_project_files::ProjectPath`]
//!    (rejecting an absolute path, `..` traversal, and forbidden characters
//!    before anything touches the filesystem), then
//! 2. walked component-by-component with `openat(O_NOFOLLOW)` by
//!    `ProjectRoot`, so **no** component of the path — not just the final
//!    one — may be a symlink, and each walked directory's `..` is checked by
//!    device/inode against the handle it was opened from.
//!
//! ## Reuse and the one real gap
//!
//! `ProjectRoot` fully subsumes this crate's old symlink-escape check and is
//! strictly *stronger*: the old check canonicalized the candidate path and
//! compared it against the canonical root as two separate filesystem calls
//! (a resolve-then-compare with a TOCTOU seam between them), and it
//! permitted a symlink whose target happened to resolve back inside the
//! root. `ProjectRoot` refuses every symlink component outright, verified
//! one `openat` at a time with no separate resolve step. So reusing it does
//! not just avoid duplicating a filesystem primitive — it removes a real,
//! if narrow, TOCTOU window this crate used to have. The one behavior this
//! crate loses as a result is permissiveness, not safety: a symlink that
//! stays inside the root, previously allowed, is now rejected too (see
//! `rejects_a_symlink_even_when_it_stays_inside_the_root` below). That is a
//! stricter default, not a weaker one, so it is not treated as a regression.
//!
//! The one place `ProjectRoot` does not offer an equivalent to what this
//! crate used to do: it has no "resolve to a path" primitive — resolution
//! and a bounded read happen together in [`ProjectRoot::read`]. That is a
//! deliberately safer shape (a separate resolve-then-open API would
//! reintroduce the TOCTOU window this reader exists to close), and it is
//! exactly what an image loader needs anyway, so this crate reads bytes
//! straight through it rather than asking for a path back.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use flashtex_project_files::{
    PathError as PfPathError, ProjectPath, ProjectRoot as PfProjectRoot, Refused as PfRefused,
    SaveError as PfSaveError,
};

/// Why a path was refused, or why the root itself is unusable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootError {
    /// The configured root does not exist.
    RootNotFound(PathBuf),
    /// The configured root exists but is not a directory (or is a symlink to
    /// one — `ProjectRoot::open` refuses a symlinked root outright).
    RootNotADirectory(PathBuf),
    /// The requested path was absolute (or carried a Windows drive prefix).
    AbsolutePath(PathBuf),
    /// The requested path contained a `..` component that would leave the
    /// root, whether caught lexically or by a directory-identity mismatch
    /// while walking.
    PathTraversal(PathBuf),
    /// A component of the path — parent directory or the file itself — is a
    /// symbolic link. Unlike the old lexical-canonicalize check, this is
    /// refused even when the link's target would have stayed in-root.
    SymlinkEscape(PathBuf),
    /// The path is empty or every component was `.`.
    EmptyPath,
    /// The resolved path does not exist on disk (a missing final file, or a
    /// missing intermediate directory).
    NotFound(PathBuf),
    /// A component that must be a directory is not one.
    NotADirectory(PathBuf),
    /// The target exists but is not a regular file (e.g. a directory).
    NotARegularFile(PathBuf),
    /// The path contains a character `ProjectPath` forbids: a backslash,
    /// colon, NUL, or other control character.
    ForbiddenCharacter(PathBuf, char),
    /// The raw path is not valid UTF-8, so it cannot be checked against the
    /// project-relative path rules at all. Rejected rather than approximated.
    NotUtf8(PathBuf),
    /// The file is larger than the configured read bound.
    TooLarge { limit: u64, size: u64 },
    /// Some other I/O failure while resolving or reading the path.
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
                write!(
                    f,
                    "asset path must be root-relative, not absolute: {}",
                    p.display()
                )
            }
            RootError::PathTraversal(p) => {
                write!(
                    f,
                    "asset path contains '..' and was rejected: {}",
                    p.display()
                )
            }
            RootError::SymlinkEscape(p) => write!(
                f,
                "asset path resolves outside the document root via a symlink: {}",
                p.display()
            ),
            RootError::EmptyPath => write!(f, "asset path is empty"),
            RootError::NotFound(p) => write!(f, "asset path does not exist: {}", p.display()),
            RootError::NotADirectory(p) => {
                write!(f, "asset path component is not a directory: {}", p.display())
            }
            RootError::NotARegularFile(p) => {
                write!(f, "asset path is not a regular file: {}", p.display())
            }
            RootError::ForbiddenCharacter(p, c) => write!(
                f,
                "asset path {} contains forbidden character {c:?}",
                p.display()
            ),
            RootError::NotUtf8(p) => {
                write!(f, "asset path is not valid UTF-8: {}", p.display())
            }
            RootError::TooLarge { limit, size } => write!(
                f,
                "asset is {size} bytes, exceeding the {limit}-byte bound"
            ),
            RootError::Io(msg) => write!(f, "asset root I/O error: {msg}"),
        }
    }
}

impl std::error::Error for RootError {}

/// A document root that bounds every asset resolution beneath it.
///
/// Constructed once from a directory that must already exist; every
/// subsequent read is checked against it by
/// [`flashtex_project_files::ProjectRoot`].
#[derive(Debug, Clone)]
pub struct AssetRoot {
    inner: Arc<PfProjectRoot>,
}

impl AssetRoot {
    /// Opens `root` as a document root. `root` must exist, be a directory,
    /// and not itself be a symlink.
    pub fn new(root: impl AsRef<Path>) -> Result<Self, RootError> {
        let root = root.as_ref();
        let inner = PfProjectRoot::open(root).map_err(|e| map_open_error(e, root))?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// The root directory this instance bounds asset loads to (as given to
    /// [`AssetRoot::new`]; not canonicalized).
    pub fn root(&self) -> &Path {
        self.inner.path()
    }

    /// Resolves `relative` against the root and reads it, bounded to
    /// `limit` bytes, in one step. There is deliberately no separate
    /// "resolve to a path" call: see the module docs for why.
    pub(crate) fn read_bounded(&self, relative: &Path, limit: u64) -> Result<Vec<u8>, RootError> {
        let project_path = to_project_path(relative)?;
        match self.inner.read(&project_path, limit) {
            Ok(Some(read)) => Ok(read.bytes),
            Ok(None) => Err(RootError::NotFound(relative.to_path_buf())),
            Err(e) => Err(map_read_error(e, relative)),
        }
    }
}

fn to_project_path(relative: &Path) -> Result<ProjectPath, RootError> {
    let raw = relative
        .to_str()
        .ok_or_else(|| RootError::NotUtf8(relative.to_path_buf()))?;
    ProjectPath::normalize(raw).map_err(|e| match e {
        PfPathError::Empty => RootError::EmptyPath,
        PfPathError::Absolute => RootError::AbsolutePath(relative.to_path_buf()),
        PfPathError::EscapesRoot => RootError::PathTraversal(relative.to_path_buf()),
        PfPathError::ForbiddenCharacter(c) => {
            RootError::ForbiddenCharacter(relative.to_path_buf(), c)
        }
    })
}

/// Maps a failure from opening the root directory itself.
fn map_open_error(e: PfSaveError, root: &Path) -> RootError {
    match e {
        // Only the root's own final path component is inspected when
        // opening it. `ProjectRoot::open` refuses a symlinked root outright
        // (see `flashtex_project_files::save::tests::symlinked_root_is_refused`),
        // where this crate's old check silently canonicalized through it.
        PfSaveError::Refused(PfRefused::SymlinkComponent { .. }) => {
            RootError::RootNotADirectory(root.to_path_buf())
        }
        PfSaveError::Refused(PfRefused::NotADirectory { .. }) => {
            RootError::RootNotADirectory(root.to_path_buf())
        }
        PfSaveError::Refused(PfRefused::Unsupported) => {
            RootError::Io("rooted file operations are not supported on this platform".to_string())
        }
        PfSaveError::Io(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
            RootError::RootNotFound(root.to_path_buf())
        }
        other => RootError::Io(other.to_string()),
    }
}

/// Maps a failure from resolving/reading a relative path inside an open root.
fn map_read_error(e: PfSaveError, relative: &Path) -> RootError {
    match e {
        PfSaveError::Refused(PfRefused::SymlinkComponent { .. }) => {
            RootError::SymlinkEscape(relative.to_path_buf())
        }
        PfSaveError::Refused(PfRefused::EscapesRoot { component }) => {
            RootError::PathTraversal(PathBuf::from(component))
        }
        PfSaveError::Refused(PfRefused::NotADirectory { component }) => {
            RootError::NotADirectory(PathBuf::from(component))
        }
        PfSaveError::Refused(PfRefused::NotARegularFile { component }) => {
            RootError::NotARegularFile(PathBuf::from(component))
        }
        PfSaveError::Refused(PfRefused::TooLarge { limit, size }) => {
            RootError::TooLarge { limit, size }
        }
        PfSaveError::Refused(PfRefused::LockUnavailable { .. }) => {
            RootError::Io("unexpected lock contention on a read-only path".to_string())
        }
        PfSaveError::Refused(PfRefused::Unsupported) => {
            RootError::Io("rooted file operations are not supported on this platform".to_string())
        }
        PfSaveError::Io(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
            RootError::NotFound(relative.to_path_buf())
        }
        other => RootError::Io(other.to_string()),
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

    const LIMIT: u64 = 1 << 20;

    #[test]
    fn resolves_a_plain_file_inside_the_root() {
        let dir = tempfile::tempdir().unwrap();
        write_file(&dir.path().join("a.png"), b"stub");
        let root = AssetRoot::new(dir.path()).unwrap();
        assert_eq!(
            root.read_bounded(Path::new("a.png"), LIMIT).unwrap(),
            b"stub"
        );
    }

    #[test]
    fn resolves_a_nested_file_inside_the_root() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("figs/ch1")).unwrap();
        write_file(&dir.path().join("figs/ch1/a.png"), b"stub");
        let root = AssetRoot::new(dir.path()).unwrap();
        assert_eq!(
            root.read_bounded(Path::new("figs/ch1/a.png"), LIMIT)
                .unwrap(),
            b"stub"
        );
    }

    #[test]
    fn rejects_parent_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root
            .read_bounded(Path::new("../../etc/passwd"), LIMIT)
            .unwrap_err();
        assert!(matches!(err, RootError::PathTraversal(_)), "{err:?}");
    }

    #[test]
    fn rejects_traversal_buried_in_the_middle_of_a_path() {
        let dir = tempfile::tempdir().unwrap();
        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root
            .read_bounded(Path::new("figs/../../etc/passwd"), LIMIT)
            .unwrap_err();
        assert!(matches!(err, RootError::PathTraversal(_)), "{err:?}");
    }

    #[test]
    fn rejects_absolute_paths() {
        let dir = tempfile::tempdir().unwrap();
        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root
            .read_bounded(Path::new("/etc/passwd"), LIMIT)
            .unwrap_err();
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
        let err = root
            .read_bounded(Path::new("looks-local.png"), LIMIT)
            .unwrap_err();
        assert!(matches!(err, RootError::SymlinkEscape(_)), "{err:?}");
    }

    #[test]
    #[cfg(unix)]
    fn rejects_escape_via_a_symlinked_directory_component() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(outside.path().join("real")).unwrap();
        write_file(&outside.path().join("real/img.png"), b"not yours");

        std::os::unix::fs::symlink(outside.path().join("real"), dir.path().join("figs")).unwrap();

        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root
            .read_bounded(Path::new("figs/img.png"), LIMIT)
            .unwrap_err();
        assert!(matches!(err, RootError::SymlinkEscape(_)), "{err:?}");
    }

    /// Documents the deliberate behavior change from reusing
    /// `flashtex_project_files::ProjectRoot`: it refuses *every* symlink
    /// component, even one whose target stays inside the root. The old
    /// hand-rolled `AssetRoot` allowed this case; see the module docs for
    /// why this is a tightening, not a regression.
    #[test]
    #[cfg(unix)]
    fn rejects_a_symlink_even_when_it_stays_inside_the_root() {
        let dir = tempfile::tempdir().unwrap();
        write_file(&dir.path().join("real.png"), b"stub");
        std::os::unix::fs::symlink(dir.path().join("real.png"), dir.path().join("alias.png"))
            .unwrap();
        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root
            .read_bounded(Path::new("alias.png"), LIMIT)
            .unwrap_err();
        assert!(matches!(err, RootError::SymlinkEscape(_)), "{err:?}");
    }

    #[test]
    fn rejects_missing_root() {
        let err = AssetRoot::new("/does/not/exist/hopefully").unwrap_err();
        assert!(matches!(err, RootError::RootNotFound(_)), "{err:?}");
    }

    #[test]
    fn rejects_root_that_is_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("not-a-dir");
        write_file(&file, b"x");
        let err = AssetRoot::new(&file).unwrap_err();
        assert!(matches!(err, RootError::RootNotADirectory(_)), "{err:?}");
    }

    #[test]
    fn rejects_empty_path() {
        let dir = tempfile::tempdir().unwrap();
        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root.read_bounded(Path::new(""), LIMIT).unwrap_err();
        assert_eq!(err, RootError::EmptyPath);
    }

    #[test]
    fn rejects_unicode_path_that_does_not_exist() {
        let dir = tempfile::tempdir().unwrap();
        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root
            .read_bounded(Path::new("figs/日本語/😀.png"), LIMIT)
            .unwrap_err();
        assert!(matches!(err, RootError::NotFound(_)), "{err:?}");
    }

    #[test]
    fn read_bounded_rejects_files_over_the_limit() {
        let dir = tempfile::tempdir().unwrap();
        write_file(&dir.path().join("big.png"), &[0u8; 32]);
        let root = AssetRoot::new(dir.path()).unwrap();
        let err = root.read_bounded(Path::new("big.png"), 16).unwrap_err();
        assert!(
            matches!(err, RootError::TooLarge { limit: 16, .. }),
            "{err:?}"
        );
    }
}
