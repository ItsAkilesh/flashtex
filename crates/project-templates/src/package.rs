//! Validation for LaTeX package names declared in a manifest.
//!
//! A [`crate::manifest::Template`]'s `packages` list is spliced directly
//! into `\usepackage{...}` lines with no further escaping (package names are
//! not free text), so it is validated at manifest-build time instead:
//! non-empty, and free of characters that could break out of the macro
//! argument or smuggle in a second package/option.

use std::fmt;

/// Why a declared package name was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageNameError {
    /// The package name was empty.
    Empty,
    /// The package name contains a character that has no business in a
    /// LaTeX package name (whitespace, `{`, `}`, `\`, `%`, `,`, `[`, `]`, or
    /// a control character).
    ForbiddenCharacter(char),
}

impl fmt::Display for PackageNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageNameError::Empty => write!(f, "package name is empty"),
            PackageNameError::ForbiddenCharacter(c) => {
                write!(f, "package name contains forbidden character {c:?}")
            }
        }
    }
}

impl std::error::Error for PackageNameError {}

/// Validates a single package name such as `"amsmath"`.
pub fn validate(name: &str) -> Result<(), PackageNameError> {
    if name.is_empty() {
        return Err(PackageNameError::Empty);
    }
    match name.chars().find(|c| {
        c.is_whitespace() || matches!(c, '{' | '}' | '\\' | '%' | ',' | '[' | ']') || c.is_control()
    }) {
        Some(c) => Err(PackageNameError::ForbiddenCharacter(c)),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_typical_package_names() {
        assert_eq!(validate("amsmath"), Ok(()));
        assert_eq!(validate("hyperref"), Ok(()));
        assert_eq!(validate("geometry"), Ok(()));
    }

    #[test]
    fn rejects_empty_and_hostile_names() {
        assert_eq!(validate(""), Err(PackageNameError::Empty));
        assert_eq!(
            validate("amsmath}\\input{x"),
            Err(PackageNameError::ForbiddenCharacter('}'))
        );
        assert_eq!(
            validate("amsmath,evil"),
            Err(PackageNameError::ForbiddenCharacter(','))
        );
        assert_eq!(
            validate("a b"),
            Err(PackageNameError::ForbiddenCharacter(' '))
        );
    }
}
