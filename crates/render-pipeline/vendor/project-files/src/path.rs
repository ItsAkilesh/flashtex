//! Project-relative path normalization.
//!
//! A [`ProjectPath`] is always relative, forward-slash separated, free of `.`
//! and empty segments, and never escapes the project root. This mirrors and
//! tightens the runtime-v1 rule (relative, no parent traversal) and the
//! transfer-v1 rule (no backslashes, colons, NUL).

use std::fmt;
use std::path::{Path, PathBuf};

/// A normalized project-relative path such as `chapters/intro.tex`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectPath(String);

/// Why a raw path was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// Empty after normalization (`""`, `"."`, `"./"`).
    Empty,
    /// Starts with `/`, `~`, or a drive prefix such as `C:`.
    Absolute,
    /// `..` segments would leave the project root.
    EscapesRoot,
    /// Contains a backslash, colon, NUL, or control character.
    ForbiddenCharacter(char),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::Empty => write!(f, "path is empty"),
            PathError::Absolute => write!(f, "path must be project-relative, not absolute"),
            PathError::EscapesRoot => write!(f, "path escapes the project root via '..'"),
            PathError::ForbiddenCharacter(c) => {
                write!(f, "path contains forbidden character {c:?}")
            }
        }
    }
}

impl std::error::Error for PathError {}

impl ProjectPath {
    /// Normalizes `raw` against the project root. `.` and empty segments are
    /// dropped; `..` pops the previous segment and is an error if there is
    /// nothing left to pop.
    pub fn normalize(raw: &str) -> Result<Self, PathError> {
        Self::resolve_in("", raw)
    }

    /// Normalizes `raw` relative to `base_dir` (itself a project-relative
    /// directory, `""` for the root). Used when a reference must be resolved
    /// relative to the referencing file's directory rather than the root.
    pub fn resolve_in(base_dir: &str, raw: &str) -> Result<Self, PathError> {
        if let Some(c) = raw
            .chars()
            .find(|c| matches!(c, '\\' | ':' | '\0') || c.is_control())
        {
            return Err(PathError::ForbiddenCharacter(c));
        }
        if raw.starts_with('/') || raw.starts_with('~') {
            return Err(PathError::Absolute);
        }
        let mut segments: Vec<&str> = Vec::new();
        for seg in base_dir.split('/').chain(raw.split('/')) {
            match seg {
                "" | "." => {}
                ".." => {
                    if segments.pop().is_none() {
                        return Err(PathError::EscapesRoot);
                    }
                }
                s => segments.push(s),
            }
        }
        if segments.is_empty() {
            return Err(PathError::Empty);
        }
        Ok(ProjectPath(segments.join("/")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The directory part (`""` for a root-level file).
    pub fn parent_dir(&self) -> &str {
        match self.0.rfind('/') {
            Some(i) => &self.0[..i],
            None => "",
        }
    }

    /// The last segment.
    pub fn file_name(&self) -> &str {
        match self.0.rfind('/') {
            Some(i) => &self.0[i + 1..],
            None => &self.0,
        }
    }

    /// The extension of the last segment without the dot, if any.
    pub fn extension(&self) -> Option<&str> {
        let name = self.file_name();
        let dot = name.rfind('.')?;
        if dot == 0 {
            None
        } else {
            Some(&name[dot + 1..])
        }
    }

    /// Returns a path with `ext` appended (`foo` -> `foo.tex`).
    pub fn with_appended_extension(&self, ext: &str) -> ProjectPath {
        ProjectPath(format!("{}.{}", self.0, ext))
    }

    /// Joins onto an OS root directory.
    pub fn to_os_path(&self, root: &Path) -> PathBuf {
        let mut p = root.to_path_buf();
        for seg in self.0.split('/') {
            p.push(seg);
        }
        p
    }
}

impl fmt::Display for ProjectPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for ProjectPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProjectPath({:?})", self.0)
    }
}

impl AsRef<str> for ProjectPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_dots_and_slashes() {
        assert_eq!(
            ProjectPath::normalize("./a/./b//c.tex").unwrap().as_str(),
            "a/b/c.tex"
        );
        assert_eq!(
            ProjectPath::normalize("a/b/../c.tex").unwrap().as_str(),
            "a/c.tex"
        );
        assert_eq!(
            ProjectPath::resolve_in("ch", "../x.tex").unwrap().as_str(),
            "x.tex"
        );
    }

    #[test]
    fn rejects_bad_paths() {
        assert_eq!(
            ProjectPath::normalize("../x.tex"),
            Err(PathError::EscapesRoot)
        );
        assert_eq!(
            ProjectPath::normalize("a/../../x.tex"),
            Err(PathError::EscapesRoot)
        );
        assert_eq!(
            ProjectPath::normalize("/etc/passwd"),
            Err(PathError::Absolute)
        );
        assert_eq!(ProjectPath::normalize("~/x"), Err(PathError::Absolute));
        assert_eq!(
            ProjectPath::normalize("C:/x"),
            Err(PathError::ForbiddenCharacter(':'))
        );
        assert_eq!(
            ProjectPath::normalize("a\\b"),
            Err(PathError::ForbiddenCharacter('\\'))
        );
        assert_eq!(ProjectPath::normalize(""), Err(PathError::Empty));
        assert_eq!(ProjectPath::normalize("."), Err(PathError::Empty));
    }

    #[test]
    fn parts() {
        let p = ProjectPath::normalize("a/b/c.tex").unwrap();
        assert_eq!(p.parent_dir(), "a/b");
        assert_eq!(p.file_name(), "c.tex");
        assert_eq!(p.extension(), Some("tex"));
        assert_eq!(ProjectPath::normalize(".hidden").unwrap().extension(), None);
        assert_eq!(p.to_os_path(Path::new("/r")), PathBuf::from("/r/a/b/c.tex"));
    }
}
