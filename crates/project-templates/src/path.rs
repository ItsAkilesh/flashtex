//! Validation for template-relative file paths.
//!
//! A template's manifest lists each file it writes by a path such as
//! `"chapters/intro.tex"`. Before anything is written to disk, every path is
//! run through [`validate`], which rejects anything that could let
//! instantiation escape the caller's target directory or write to a location
//! the caller did not ask for. Validation is intentionally strict and not
//! merely path *normalization*: `..` is never resolved and cancelled out, it
//! is refused outright, so a manifest can never depend on subtle traversal
//! arithmetic.

use std::fmt;

/// Why a template file path was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// The path was empty, or normalized to nothing (e.g. `""`, `"."`).
    Empty,
    /// The path is absolute (leads with `/`, `~`, or a Windows drive prefix
    /// such as `C:`).
    Absolute,
    /// The path contains a `..` segment. Refused outright rather than
    /// resolved, so a manifest can never rely on `..` cancelling out.
    ParentTraversal,
    /// A path segment is empty (a doubled `/`) or is exactly `.`.
    EmptySegment,
    /// The path contains a backslash, NUL, or other control character.
    ForbiddenCharacter(char),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::Empty => write!(f, "template file path is empty"),
            PathError::Absolute => write!(f, "template file path must be relative, not absolute"),
            PathError::ParentTraversal => {
                write!(f, "template file path contains a '..' segment")
            }
            PathError::EmptySegment => {
                write!(f, "template file path contains an empty or '.' segment")
            }
            PathError::ForbiddenCharacter(c) => {
                write!(f, "template file path contains forbidden character {c:?}")
            }
        }
    }
}

impl std::error::Error for PathError {}

/// Validates `raw` as a template-relative file path and returns its
/// slash-separated segments on success.
///
/// A valid path:
/// - is non-empty;
/// - does not start with `/` or `~`, and does not contain a Windows drive
///   prefix such as `C:`;
/// - contains no `..` segment, anywhere;
/// - contains no empty (`//`) or `.` segment;
/// - contains no backslash, NUL, or other control character.
pub fn validate(raw: &str) -> Result<Vec<&str>, PathError> {
    if raw.is_empty() {
        return Err(PathError::Empty);
    }
    if let Some(c) = raw
        .chars()
        .find(|c| matches!(c, '\\' | ':' | '\0') || c.is_control())
    {
        return Err(PathError::ForbiddenCharacter(c));
    }
    if raw.starts_with('/') || raw.starts_with('~') {
        return Err(PathError::Absolute);
    }
    let mut segments = Vec::new();
    for seg in raw.split('/') {
        match seg {
            "" | "." => return Err(PathError::EmptySegment),
            ".." => return Err(PathError::ParentTraversal),
            s => segments.push(s),
        }
    }
    if segments.is_empty() {
        return Err(PathError::Empty);
    }
    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_relative_paths() {
        assert_eq!(validate("report.tex").unwrap(), vec!["report.tex"]);
        assert_eq!(
            validate("chapters/intro.tex").unwrap(),
            vec!["chapters", "intro.tex"]
        );
    }

    #[test]
    fn rejects_parent_traversal() {
        assert_eq!(validate("../etc/passwd"), Err(PathError::ParentTraversal));
        assert_eq!(
            validate("chapters/../../etc/passwd"),
            Err(PathError::ParentTraversal)
        );
        // Even a traversal that would mathematically cancel out is refused
        // outright: `..` is never resolved, only rejected.
        assert_eq!(validate("a/../a.tex"), Err(PathError::ParentTraversal));
    }

    #[test]
    fn rejects_absolute_paths() {
        assert_eq!(validate("/etc/passwd"), Err(PathError::Absolute));
        assert_eq!(validate("~/x.tex"), Err(PathError::Absolute));
        assert_eq!(
            validate("C:/Users/x.tex"),
            Err(PathError::ForbiddenCharacter(':'))
        );
    }

    #[test]
    fn rejects_forbidden_characters_and_segments() {
        assert_eq!(
            validate("a\\b.tex"),
            Err(PathError::ForbiddenCharacter('\\'))
        );
        assert_eq!(
            validate("a\0b.tex"),
            Err(PathError::ForbiddenCharacter('\0'))
        );
        assert_eq!(validate(""), Err(PathError::Empty));
        assert_eq!(validate("a//b.tex"), Err(PathError::EmptySegment));
        assert_eq!(validate("./a.tex"), Err(PathError::EmptySegment));
    }
}
