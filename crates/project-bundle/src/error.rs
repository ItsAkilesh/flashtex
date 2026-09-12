use std::fmt;

use flashtex_project_files::Digest;

/// Everything that can go wrong building, previewing or applying a bundle.
///
/// Every variant is typed and carries the offending caller-declared path (as
/// given, not normalized) so callers can report precisely what was rejected
/// and why, without parsing a string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleError {
    /// The bundle/target root does not exist, is not a directory, is a
    /// symlink, or could not be opened for another reason reported by the
    /// underlying rooted reader.
    InvalidRoot(String),
    /// A caller-supplied path was the empty string.
    EmptyPath,
    /// A caller-supplied path started with `/` or `~` (an absolute path).
    /// Rejected before any filesystem access.
    AbsolutePath(String),
    /// A caller-supplied path had a `..` component that would leave the
    /// root, whether caught syntactically before any filesystem access or
    /// by the walk-time device/inode check in the rooted reader. Rejected
    /// regardless of whether anything exists at the escaped-to location.
    PathTraversal(String),
    /// A caller-supplied path was malformed in some other way: a forbidden
    /// character (backslash, colon, NUL, other control character), or a
    /// parent path component that is not a directory.
    MalformedPath(String),
    /// The same bundle path was declared more than once in one spec.
    DuplicatePath(String),
    /// Two *different* caller-declared paths resolve to the same
    /// underlying file on disk — most notably, two Unicode normalization
    /// forms of one visual filename (precomposed vs. combining-mark
    /// decomposed) that a normalization-insensitive filesystem (default
    /// macOS APFS) folds into one directory entry. Rejected as a typed
    /// error rather than silently building a bundle with two entries that
    /// would in fact overwrite each other, or that both happen to read the
    /// same bytes without the caller ever being told why.
    AmbiguousPath { first: String, second: String },
    /// A parent directory or the file itself is a symbolic link. The
    /// underlying rooted reader (`flashtex-project-files`) refuses *every*
    /// symlink component outright — whether or not it would resolve inside
    /// the root — so this fires strictly more often than rev 1's
    /// `SymlinkEscapesRoot` did; a symlink that stays inside the root is no
    /// longer silently followed.
    SymlinkRefused(String),
    /// The declared path does not exist under the root.
    NotFound(String),
    /// The declared path exists but is not a regular file (e.g. a
    /// directory). The crate never walks directories, so this is always a
    /// caller error, not a discovery decision.
    NotAFile(String),
    /// The declared path exists and is a regular file, but is larger than
    /// the configured per-file read limit.
    FileTooLarge { path: String, limit: u64, size: u64 },
    /// More entries were supplied than `BundleLimits::max_entries` allows.
    TooManyEntries { limit: usize, actual: usize },
    /// The running total size of read files exceeded
    /// `BundleLimits::max_total_bytes`.
    TotalBytesExceeded { limit: u64, actual: u64 },
    /// `apply_import` was asked to act on a file the preview marked as a
    /// conflict, but `decisions` said nothing about that path at all. A
    /// conflict must be given an explicit decision (`Write` or `Skip`); it
    /// is never resolved by omission.
    OverwriteNotDecided(String),
    /// A write in `apply_import` was refused because the target changed
    /// between the preview and the write: the file the caller decided about
    /// is not the file that is actually there any more. This is the
    /// underlying rooted writer's compare-and-swap firing, surfaced typed
    /// rather than folded into a generic I/O error — it is exactly the "no
    /// silent overwrite" guarantee catching a race, not a caller mistake.
    ConcurrentModification {
        path: String,
        expected: Option<Digest>,
        found: Option<Digest>,
    },
    /// A write in [`crate::apply_import`] failed partway through a
    /// multi-file batch, and the rollback that undoes every write already
    /// committed earlier in that same call could not fully complete — a
    /// double fault (an out-of-contract writer bypassing the project lock
    /// to touch a path this call had just written, or the disk filling
    /// during the restore). This is distinct from every other error
    /// `apply_import` returns: those mean the whole call left the target
    /// exactly as it was before (the earlier writes were cleanly rolled
    /// back and the original typed cause is returned directly); this one
    /// means it did not, and names precisely which paths are left in the
    /// state this call wrote them to.
    RollbackIncomplete {
        /// The error that triggered the rollback — the same error
        /// `apply_import` would have returned had rollback fully
        /// succeeded.
        original_cause: Box<BundleError>,
        /// Paths this call had already written that rollback could not
        /// restore to their pre-import state, each paired with why.
        left_in_written_state: Vec<(String, String)>,
    },
    /// Any other I/O failure reading or writing the file, with context.
    Io(String),
}

impl fmt::Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BundleError::InvalidRoot(msg) => write!(f, "invalid root: {msg}"),
            BundleError::EmptyPath => write!(f, "empty bundle path"),
            BundleError::AbsolutePath(p) => write!(f, "absolute path not allowed: {p:?}"),
            BundleError::PathTraversal(p) => write!(f, "path traversal not allowed: {p:?}"),
            BundleError::MalformedPath(msg) => write!(f, "malformed path: {msg}"),
            BundleError::DuplicatePath(p) => write!(f, "duplicate bundle path: {p:?}"),
            BundleError::AmbiguousPath { first, second } => write!(
                f,
                "{first:?} and {second:?} are different declared paths but resolve to the same file on disk"
            ),
            BundleError::SymlinkRefused(p) => {
                write!(f, "symlink component refused: {p:?}")
            }
            BundleError::NotFound(p) => write!(f, "not found under root: {p:?}"),
            BundleError::NotAFile(p) => write!(f, "not a regular file: {p:?}"),
            BundleError::FileTooLarge { path, limit, size } => write!(
                f,
                "{path:?} is {size} bytes, over the {limit}-byte per-file limit"
            ),
            BundleError::TooManyEntries { limit, actual } => write!(
                f,
                "{actual} entries exceeds the {limit}-entry bundle limit"
            ),
            BundleError::TotalBytesExceeded { limit, actual } => write!(
                f,
                "bundle total {actual} bytes exceeds the {limit}-byte limit"
            ),
            BundleError::OverwriteNotDecided(p) => write!(
                f,
                "{p:?} conflicts with an existing file and no import decision was given for it"
            ),
            BundleError::ConcurrentModification {
                path,
                expected,
                found,
            } => write!(
                f,
                "{path:?} changed since the import preview was computed (expected {expected:?}, found {found:?}); refusing to overwrite"
            ),
            BundleError::RollbackIncomplete {
                original_cause,
                left_in_written_state,
            } => write!(
                f,
                "apply_import batch failed ({original_cause}) and rollback could not fully undo it; left in written state: {left_in_written_state:?}"
            ),
            BundleError::Io(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for BundleError {}
