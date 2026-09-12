//! The bounded template manifest: [`Template`] and [`TemplateFile`].
//!
//! A `Template` is a small, fully-declared bundle: a fixed list of files
//! (each a relative path plus a body that may reference the placeholders
//! `{{project_name}}`, `{{author}}`, and `{{packages}}`) and a fixed list of
//! required LaTeX packages. The packages list is the *only* source of
//! `\usepackage{...}` lines: a file body never hardcodes one, so a template
//! can never silently require a package it did not declare (acceptance
//! criterion: "required packages must be declared explicitly in the
//! manifest, not implied by the body text" — enforced structurally, see
//! [`crate::instantiate`]).

use std::fmt;

use crate::package::{self, PackageNameError};
use crate::path::{self, PathError};

/// One file a template will write, relative to the instantiation target
/// directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateFile {
    /// Relative path, e.g. `"chapters/intro.tex"`. Validated by
    /// [`Template::validate`] before any instantiation is attempted.
    pub path: String,
    /// File body. May contain the placeholders `{{project_name}}`,
    /// `{{author}}`, and `{{packages}}`, substituted at instantiation time.
    pub body: String,
}

impl TemplateFile {
    /// Convenience constructor.
    pub fn new(path: impl Into<String>, body: impl Into<String>) -> Self {
        TemplateFile {
            path: path.into(),
            body: body.into(),
        }
    }
}

/// A bounded, self-describing project template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    /// Short, stable identifier, e.g. `"course-report"`.
    pub id: String,
    /// Human-readable name.
    pub title: String,
    /// One-line description.
    pub description: String,
    /// LaTeX packages this template requires. The single source of truth
    /// for the `\usepackage{...}` lines emitted into generated files.
    pub packages: Vec<String>,
    /// Files this template writes.
    pub files: Vec<TemplateFile>,
}

/// Why a [`Template`] failed manifest validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// A file's declared path was rejected. Carries the offending path and
    /// why.
    InvalidPath {
        file_path: String,
        source: PathError,
    },
    /// A declared package name was rejected. Carries the offending name and
    /// why.
    InvalidPackage {
        package: String,
        source: PackageNameError,
    },
    /// Two files declared the same path.
    DuplicatePath(String),
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestError::InvalidPath { file_path, source } => {
                write!(f, "invalid file path {file_path:?}: {source}")
            }
            ManifestError::InvalidPackage { package, source } => {
                write!(f, "invalid package {package:?}: {source}")
            }
            ManifestError::DuplicatePath(p) => {
                write!(f, "duplicate file path {p:?}")
            }
        }
    }
}

impl std::error::Error for ManifestError {}

impl Template {
    /// Validates every declared file path and package name, and checks for
    /// duplicate paths. Called automatically by
    /// [`crate::instantiate::instantiate`] before anything is written.
    pub fn validate(&self) -> Result<(), ManifestError> {
        let mut seen = std::collections::HashSet::new();
        for file in &self.files {
            path::validate(&file.path).map_err(|source| ManifestError::InvalidPath {
                file_path: file.path.clone(),
                source,
            })?;
            if !seen.insert(file.path.as_str()) {
                return Err(ManifestError::DuplicatePath(file.path.clone()));
            }
        }
        for pkg in &self.packages {
            package::validate(pkg).map_err(|source| ManifestError::InvalidPackage {
                package: pkg.clone(),
                source,
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_template() -> Template {
        Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec!["amsmath".into()],
            files: vec![TemplateFile::new("main.tex", "hello")],
        }
    }

    #[test]
    fn valid_template_passes() {
        assert_eq!(base_template().validate(), Ok(()));
    }

    #[test]
    fn rejects_traversal_in_a_file_path() {
        let mut t = base_template();
        t.files.push(TemplateFile::new("../escape.tex", "x"));
        assert_eq!(
            t.validate(),
            Err(ManifestError::InvalidPath {
                file_path: "../escape.tex".into(),
                source: PathError::ParentTraversal,
            })
        );
    }

    #[test]
    fn rejects_absolute_file_path() {
        let mut t = base_template();
        t.files.push(TemplateFile::new("/etc/passwd", "x"));
        assert_eq!(
            t.validate(),
            Err(ManifestError::InvalidPath {
                file_path: "/etc/passwd".into(),
                source: PathError::Absolute,
            })
        );
    }

    #[test]
    fn rejects_duplicate_paths() {
        let mut t = base_template();
        t.files.push(TemplateFile::new("main.tex", "other"));
        assert_eq!(
            t.validate(),
            Err(ManifestError::DuplicatePath("main.tex".into()))
        );
    }

    #[test]
    fn rejects_bad_package_name() {
        let mut t = base_template();
        t.packages.push("bad name".into());
        assert_eq!(
            t.validate(),
            Err(ManifestError::InvalidPackage {
                package: "bad name".into(),
                source: PackageNameError::ForbiddenCharacter(' '),
            })
        );
    }
}
