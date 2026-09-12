use std::fmt;

/// Everything that can go wrong building a bundle.
///
/// Every variant is typed and carries the offending caller-declared path (as
/// given, not normalized) so callers can report precisely what was rejected
/// and why, without parsing a string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleError {
    /// The bundle root does not exist, is not a directory, or could not be
    /// canonicalized.
    InvalidRoot(String),
    /// A caller-supplied path was the empty string.
    EmptyPath,
    /// A caller-supplied path started with `/` (an absolute path). Rejected
    /// before any filesystem access.
    AbsolutePath(String),
    /// A caller-supplied path contained a `..` component. Rejected before
    /// any filesystem access, regardless of whether the target exists.
    PathTraversal(String),
    /// A caller-supplied path was malformed in some other way: a `.`
    /// component, an empty component (`//` or a trailing `/`), or an
    /// embedded NUL byte.
    MalformedPath(String),
    /// The same bundle path was declared more than once in one spec.
    DuplicatePath(String),
    /// The path resolved, after following symlinks, to a location outside
    /// the bundle root.
    SymlinkEscapesRoot(String),
    /// The declared path does not exist under the root.
    NotFound(String),
    /// The declared path exists but is not a regular file (e.g. a
    /// directory). The crate never walks directories, so this is always a
    /// caller error, not a discovery decision.
    NotAFile(String),
    /// Any other I/O failure reading the file, with context.
    Io(String),
}

impl fmt::Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BundleError::InvalidRoot(msg) => write!(f, "invalid bundle root: {msg}"),
            BundleError::EmptyPath => write!(f, "empty bundle path"),
            BundleError::AbsolutePath(p) => write!(f, "absolute path not allowed: {p:?}"),
            BundleError::PathTraversal(p) => write!(f, "path traversal not allowed: {p:?}"),
            BundleError::MalformedPath(msg) => write!(f, "malformed path: {msg}"),
            BundleError::DuplicatePath(p) => write!(f, "duplicate bundle path: {p:?}"),
            BundleError::SymlinkEscapesRoot(p) => {
                write!(f, "symlink escapes bundle root: {p:?}")
            }
            BundleError::NotFound(p) => write!(f, "not found under root: {p:?}"),
            BundleError::NotAFile(p) => write!(f, "not a regular file: {p:?}"),
            BundleError::Io(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for BundleError {}
