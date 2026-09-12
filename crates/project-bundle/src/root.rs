use std::path::Path;

use flashtex_project_files::ProjectRoot as FilesRoot;
use flashtex_project_files::{Digest, PathError, ProjectPath, Refused, SaveError};
use unicode_normalization::UnicodeNormalization;

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
    ///
    /// "Does not exist" covers both a missing leaf *and* a missing parent
    /// directory. The underlying rooted reader reports the first as
    /// `Ok(None)` but the second as a raw `ENOENT` from the component walk
    /// (it opens each directory in turn and never has to distinguish "no
    /// such directory" from "no such file"); both mean the declared path is
    /// not present under the root, so both are `Ok(None)` here. Without
    /// this, previewing a bundle entry such as `chapters/intro.tex` into a
    /// target that does not have a `chapters/` directory yet failed with an
    /// untyped [`BundleError::Io`] instead of classifying it as
    /// [`crate::FileOutcome::New`] — even though `apply_import`'s writer
    /// creates missing parent directories.
    pub fn read_rooted_optional(&self, relative: &str) -> Result<Option<RootedFile>, BundleError> {
        let path = Self::normalize(relative)?;
        match self.inner.read(&path, self.file_limit) {
            Ok(Some(read)) => Ok(Some(RootedFile {
                size: read.bytes.len() as u64,
                bytes: read.bytes,
                sha256: read.sha256,
            })),
            Ok(None) => Ok(None),
            Err(e) if is_missing_component(&e) => Ok(None),
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

    /// Unicode-canonical identity for `relative`, used to detect two
    /// *different* caller-declared paths that are the same visual name in
    /// different normalization forms — the APFS hazard documented at the
    /// crate level: precomposed (`é`, U+00E9) vs. combining-mark-decomposed
    /// (`e` + U+0301) spellings are distinct byte strings that a
    /// normalization-insensitive volume (default macOS APFS) folds into one
    /// directory entry.
    ///
    /// Purely syntactic: normalizes `relative` (after the same syntactic
    /// validation [`ProjectRoot::normalize`] applies) to Unicode NFC and
    /// returns that string. This takes no `&self`, opens nothing, and reads
    /// nothing — unlike an `fs::canonicalize`-based identity check, it does
    /// not require `relative` to exist on disk, so it can classify a
    /// collision between two declared paths before either has been read or
    /// written (the import-preview case: nothing on the target exists yet).
    /// It is also filesystem-independent: two paths this function
    /// identifies as colliding do so on every platform, not only on a host
    /// whose filesystem happens to fold Unicode normalization forms
    /// together. `None` only when `relative` itself fails syntactic
    /// validation; the caller's own [`ProjectRoot::normalize`] call reports
    /// the specific reason.
    pub(crate) fn normalized_identity(relative: &str) -> Option<String> {
        let normalized = Self::normalize(relative).ok()?;
        Some(nfc_form(normalized.as_str()))
    }
}

/// NFC-normalize `s`. The pure string transform behind
/// [`ProjectRoot::normalized_identity`], factored out so [`is_reserved`]'s
/// case-folded identity can build on the exact same normalization step
/// rather than a second, independently-written one.
fn nfc_form(s: &str) -> String {
    s.nfc().collect()
}

/// Case-folded, Unicode-canonical identity of an already-[`ProjectRoot::normalize`]d
/// path, used only by [`is_reserved`].
///
/// NFC-normalizes with the same [`nfc_form`] step [`ProjectRoot::normalized_identity`]
/// uses for [`crate::error::BundleError::AmbiguousPath`], then additionally
/// case-folds with `char::to_lowercase` — Rust's Unicode-aware case
/// conversion (the Unicode Derived Core Property `Lowercase`, plus the
/// multi-character mappings in `SpecialCasing.txt`), never the ASCII-only
/// `to_ascii_lowercase`, which touches only `A`-`Z` and would leave every
/// non-ASCII upper/lower pair unfolded.
///
/// This is deliberately *not* the same key [`ProjectRoot::normalized_identity`]
/// produces: two different bundle-declared paths differing only by case are
/// not [`crate::error::BundleError::AmbiguousPath`] (a case-sensitive
/// filesystem keeps them genuinely distinct files, so folding case there
/// would flag harmless imports as colliding). But a path that reaches
/// `.flashtex` is reserved regardless of the *target* filesystem's own
/// case sensitivity — the risk (stranding the project lock, see
/// [`crate::error::BundleError::ReservedPath`]) exists the moment the
/// filesystem *might* fold two spellings together, and this crate has no
/// reliable way to ask a given target root whether it does.
fn caseless_identity(s: &str) -> String {
    let folded: String = s.chars().flat_map(char::to_lowercase).collect();
    nfc_form(&folded)
}

/// Syntactic validation of a caller-declared bundle path, independent of
/// the filesystem — delegates entirely to
/// `flashtex_project_files::ProjectPath::normalize`. Kept as a standalone
/// function (rather than folded into `ProjectRoot`) since rev 1 exposed it
/// this way and it needs no root to run.
pub fn validate_relative_path(path: &str) -> Result<(), BundleError> {
    ProjectRoot::normalize(path).map(|_| ())
}

/// The project's own control directory, relative to the root: the advisory
/// lock (`.flashtex/project.lock`) and the crash-recovery journal
/// (`.flashtex/recovery/...`) both live here. It is `flashtex-project-files`'
/// private bookkeeping, never project content, so this crate refuses to
/// import over it — see [`BundleError::ReservedPath`].
pub(crate) const CONTROL_DIR: &str = ".flashtex";

/// Whether a normalized project path lands inside [`CONTROL_DIR`].
///
/// Compared on [`caseless_identity`] rather than the raw string: `.flashtex`
/// must be unreachable by any bundle-declared path on every filesystem this
/// crate might run against, including one that is case-insensitive and/or
/// normalization-insensitive (default macOS APFS is both). A byte-exact
/// comparison alone lets a spelling such as `.FlashTeX/Project.Lock` sail
/// past this check while the underlying rooted writer's write-then-`rename`
/// still lands on the very same directory entry as the real
/// `.flashtex/project.lock` once such a filesystem folds the two spellings
/// together.
pub(crate) fn is_reserved(path: &ProjectPath) -> bool {
    let folded = caseless_identity(path.as_str());
    folded == CONTROL_DIR || folded.starts_with(concat!(".flashtex", "/"))
}

/// Whether a rooted-read failure is really "nothing is there": the walk to
/// the parent directory hit a component that does not exist. The rooted
/// reader already turns a missing *leaf* into `Ok(None)`, so an `ENOENT`
/// surfacing as an I/O error can only have come from the directory walk.
fn is_missing_component(err: &SaveError) -> bool {
    matches!(err, SaveError::Io(e) if e.kind() == std::io::ErrorKind::NotFound)
}

fn map_path_error(raw: &str, err: PathError) -> BundleError {
    match err {
        PathError::Empty => BundleError::EmptyPath,
        PathError::Absolute => BundleError::AbsolutePath(raw.to_string()),
        PathError::EscapesRoot => BundleError::PathTraversal(raw.to_string()),
        PathError::ForbiddenCharacter(c) => {
            BundleError::MalformedPath(format!("{raw:?}: contains forbidden character {c:?}"))
        }
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
        SaveError::Refused(Refused::NotADirectory { component }) => {
            BundleError::MalformedPath(format!("{raw:?}: {component:?} is not a directory"))
        }
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
        SaveError::Refused(Refused::Unsupported) => BundleError::Io(format!(
            "{raw}: rooted file operations unsupported on this platform"
        )),
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
