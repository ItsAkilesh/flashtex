//! Validation for free-text fields substituted into a template (project
//! name, author).
//!
//! These strings are not file paths and cannot escape the target directory,
//! but a control character (a stray NUL, an embedded carriage return) is
//! never a legitimate project name or author and is rejected up front with a
//! typed error rather than silently passed through into generated LaTeX
//! source. Ordinary text is otherwise accepted unchanged, including
//! non-ASCII scripts: field values are LaTeX-escaped at substitution time by
//! [`crate::escape::escape`], not here.

use std::fmt;

/// Why a free-text field (project name, author) was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldError {
    /// The field contains a control character such as NUL, a raw newline,
    /// or a raw tab.
    ForbiddenCharacter(char),
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldError::ForbiddenCharacter(c) => {
                write!(f, "field contains forbidden control character {c:?}")
            }
        }
    }
}

impl std::error::Error for FieldError {}

/// Validates a free-text field value. Empty strings are allowed (an author
/// may legitimately be omitted); control characters are not.
pub fn validate(value: &str) -> Result<(), FieldError> {
    match value.chars().find(|c| c.is_control()) {
        Some(c) => Err(FieldError::ForbiddenCharacter(c)),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ordinary_and_unicode_text() {
        assert_eq!(validate("Optics Lab Report"), Ok(()));
        assert_eq!(validate("Café Ünïcödé — 論文"), Ok(()));
        assert_eq!(validate(""), Ok(()));
    }

    #[test]
    fn rejects_control_characters() {
        assert_eq!(
            validate("line1\nline2"),
            Err(FieldError::ForbiddenCharacter('\n'))
        );
        assert_eq!(
            validate("bad\0name"),
            Err(FieldError::ForbiddenCharacter('\0'))
        );
        assert_eq!(
            validate("tab\tstop"),
            Err(FieldError::ForbiddenCharacter('\t'))
        );
    }
}
