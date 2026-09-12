use std::path::{Path, PathBuf};

use flashtex_project_files::{Digest, PathError, ProjectPath, Refused, SaveError};
use flashtex_project_files::ProjectRoot as FilesRoot;

use crate::error::BundleError;

/// Largest single file this crate will read unless a caller picks a smaller
/// limit with [`ProjectRoot::with_file_limit`]. Matches
/// `flashtex_project_files::DEFAULT_READ_LIMIT`.
pub const DEFAULT_FILE_LIMIT: u64 = flashtex_project_files::DEFAULT_READ_LIMIT;

/// A rooted project directory that every read is confined to.
///
/// This is a thin, typed adapter over `flashtex_project_files::ProjectRoot`
/// (issue #18's rooted, symlink-refusing, `openat(O_NOFOLLOW)`-based reader):
/// this crate performs no filesystem traversal, symlink resolution or
/// escape-detection of its own. Every read goes through
/// [`ProjectRoot::read_rooted`] / [`ProjectRoot::read_rooted_optional`],
/// which resolve exactly one caller-declared path at a time and reject:
///
/// - an absolute path or a `..` that would leave the root — checked
///   syntactically by `ProjectPath::normalize`, before any filesystem
///   access, so the result never depends on whether something happens to
///   exist outside the root;
/// - a `..` that only the walk can detect (a directory replaced by a
///   symlink mid-path) — checked at walk time by comparing each directory's
///   `..` device/inode with the handle it was reached from;
/// - any symlink component, parent or leaf, whether or not it would resolve
///   inside the root (`openat(..., O_NOFOLLOW)` at every step) — strictly
///   more conservative than rev 1's canonicalize-then-`starts_with` check,
///   which allowed a symlink that happened to stay inside the root.
///
/// See the crate-level docs for the one behavior this reuse does **not**
/// carry over unchanged, and why that is a tightening rather than a gap.
#[derive(Debug)]
pub struct ProjectRoot {
    inner: FilesRoot,
    file_limit: u64,
}

/// One file read through a [`ProjectRoot`]: its bytes, size and full
/// content hash, from the same bounded, rooted read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootedFile {
    pub bytes: Vec<u8>,
    pub sha256: Digest,
    pub size: u64,
}

impl ProjectRoot {
    /// Open `path` as a bundle root, bounding individual reads at
    /// [`DEFAULT_FILE_LIMIT`].
    ///
    /// Fails if `path` does not exist, is not a directory, or is itself a
    /// symlink (refused by the underlying `openat(O_DIRECTORY|O_NOFOLLOW)`).
    pub fn new(path: impl AsRef<Path>) -> Result<Self, BundleError> {
        Self::with_file_limit(path, DEFAULT_FILE_LIMIT)
    }

    /// Like [`ProjectRoot::new`], bounding individual reads at `file_limit`
    /// bytes instead of the default.
    pub fn with_file_limit(path: impl AsRef<Path>, file_limit: u64) -> Result<Self, BundleError> {
        let path = path.as_ref();
        let inner = FilesRoot::open(path)
            .map_err(|e| BundleError::InvalidRoot(format!("{}: {e}", path.display())))?;
        Ok(Self { inner, file_limit })
    }

    /// The root path as opened.
    pub fn as_path(&self) -> &Path {
        self.inner.path()
    }

    /// Validate and normalize a caller-declared relative path with no
    /// filesystem access at all — the pure syntactic half of resolution,
    /// exposed standalone so callers can reject bad input before touching
    /// disk.
    pub fn normalize(relative: &str) -> Result<ProjectPath, BundleError> {
        ProjectPath::normalize(relative).map_err(|e| map_path_error(relative, e))
    }

    /// Read `relative`'s bytes, size and SHA-256, rooted and bounded at this
    /// root's file limit. `Ok(None)` when the file does not exist under the
    /// root; never performs a write.
    pub fn read_rooted_optional(&self, relative: &str) -> Result<Option<RootedFile>, BundleError> {
        let path = Self::normalize(relative)?;
        match self.inner.read(&path, self.file_limit) {
            Ok(Some(read)) => Ok(Some(RootedFile {
                size: read.bytes.len() as u64,
                bytes: read.bytes,
                sha256: read.sha256,
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(map_save_error(relative, e)),
        }
    }

    /// Read `relative`'s bytes, rooted per [`ProjectRoot::read_rooted_optional`].
    /// A missing file is [`BundleError::NotFound`] rather than `Ok(None)`.
    pub fn read_rooted(&self, relative: &str) -> Result<Vec<u8>, BundleError> {
        self.read_rooted_optional(relative)?
            .map(|f| f.bytes)
            .ok_or_else(|| BundleError::NotFound(relative.to_string()))
    }

    /// The underlying `flashtex_project_files::ProjectRoot`, for callers
    /// (within this crate) that need its lock/save/remove operations —
    /// e.g. [`crate::apply_import`] performing a caller-decided write.
    pub(crate) fn files_root(&self) -> &FilesRoot {
        &self.inner
    }

    /// Best-effort filesystem identity for `relative`, used only to detect
    /// two *different* caller-declared paths that the filesystem itself
    /// folds into the same underlying file — the APFS hazard documented at
    /// the crate level: two Unicode normalization forms of one visual name
    /// (precomposed vs. combining-mark decomposed) are distinct byte
    /// strings but the same directory entry on a normalization-insensitive
    /// volume (default macOS APFS). `None` when identity cannot be
    /// determined (nothing there any more, or the OS-level canonicalize
    /// call itself fails) — this is a collision *detector* layered on top
    /// of an already-successful read, never a precondition for it, so a
    /// failure here is silently treated as "no collision observed" rather
    /// than propagated as an error.
    pub(crate) fn canonical_identity(&self, relative: &str) -> Option<PathBuf> {
        let normalized = Self::normalize(relative).ok()?;
        let os_path = normalized.to_os_path(self.as_path());
        std::fs::canonicalize(&os_path).ok()
    }
}

/// Syntactic validation of a caller-declared bundle path, independent of
/// the filesystem — delegates entirely to
/// `flashtex_project_files::ProjectPath::normalize`. Kept as a standalone
/// function (rather than folded into `ProjectRoot`) since rev 1 exposed it
/// this way and it needs no root to run.
pub fn validate_relative_path(path: &str) -> Result<(), BundleError> {
    ProjectRoot::normalize(path).map(|_| ())
}

fn map_path_error(raw: &str, err: PathError) -> BundleError {
    match err {
        PathError::Empty => BundleError::EmptyPath,
        PathError::Absolute => BundleError::AbsolutePath(raw.to_string()),
        PathError::EscapesRoot => BundleError::PathTraversal(raw.to_string()),
        PathError::ForbiddenCharacter(c) => BundleError::MalformedPath(format!(
            "{raw:?}: contains forbidden character {c:?}"
        )),
    }
}

fn map_save_error(raw: &str, err: SaveError) -> BundleError {
    match err {
        SaveError::Refused(Refused::SymlinkComponent { .. }) => {
            BundleError::SymlinkRefused(raw.to_string())
        }
        SaveError::Refused(Refused::EscapesRoot { .. }) => {
            BundleError::PathTraversal(raw.to_string())
        }
        SaveError::Refused(Refused::NotADirectory { component }) => BundleError::MalformedPath(
            format!("{raw:?}: {component:?} is not a directory"),
        ),
        SaveError::Refused(Refused::NotARegularFile { .. }) => {
            BundleError::NotAFile(raw.to_string())
        }
        SaveError::Refused(Refused::TooLarge { limit, size }) => BundleError::FileTooLarge {
            path: raw.to_string(),
            limit,
            size,
        },
        SaveError::Refused(Refused::LockUnavailable { .. }) => {
            BundleError::Io(format!("{raw}: project lock unavailable"))
        }
        SaveError::Refused(Refused::Unsupported) => {
            BundleError::Io(format!("{raw}: rooted file operations unsupported on this platform"))
        }
        SaveError::Io(e) => BundleError::Io(format!("{raw}: {e}")),
        SaveError::DirectorySync(e) => {
            BundleError::Io(format!("{raw}: directory fsync failed after rename: {e}"))
        }
        SaveError::Conflict(c) => {
            BundleError::Io(format!("{raw}: unexpected conflict during read: {c:?}"))
        }
    }
}

/// Maps a `flashtex_project_files::save::ProjectLock::save` error for
/// `apply_import`'s write path. Differs from [`map_save_error`] only in how
/// it treats `SaveError::Conflict`: on a write, a conflict is the expected,
/// meaningful outcome of the compare-and-swap catching a race, not an
/// internal inconsistency — so it is surfaced as
/// [`BundleError::ConcurrentModification`] rather than a generic I/O string.
pub(crate) fn map_write_error(raw: &str, err: SaveError) -> BundleError {
    match err {
        SaveError::Conflict(c) => BundleError::ConcurrentModification {
            path: raw.to_string(),
            expected: c.ours,
            found: c.theirs,
        },
        other => map_save_error(raw, other),
    }
}
