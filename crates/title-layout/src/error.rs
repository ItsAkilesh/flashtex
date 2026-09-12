use std::fmt;

use crate::class::DocumentClass;

/// Errors from measuring a title block or abstract.
///
/// There is no fallback variant: an unsupported document class or a
/// malformed input always returns `Err`, never a best-guess `Ok`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TitleLayoutError {
    /// This crate has no `\maketitle`/`abstract` measurements for `class`.
    /// See [`crate::DocumentClass::is_supported`].
    UnsupportedDocumentClass(DocumentClass),
    /// `\title{...}` had no non-blank line.
    EmptyTitle,
    /// `\author{...}` had no author at all.
    NoAuthors,
    /// One author (0-based index) had no non-blank line.
    EmptyAuthorLine(usize),
    /// `\date{...}` was given but every line was blank; use
    /// [`crate::DateField::Suppressed`] to omit the date deliberately.
    EmptyDate,
}

impl fmt::Display for TitleLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TitleLayoutError::UnsupportedDocumentClass(c) => {
                write!(
                    f,
                    "flashtex-title-layout has no measurements for document class `{c}`; only `article` is supported"
                )
            }
            TitleLayoutError::EmptyTitle => write!(f, "title has no non-blank line"),
            TitleLayoutError::NoAuthors => write!(f, "no authors given"),
            TitleLayoutError::EmptyAuthorLine(i) => write!(f, "author {i} has no non-blank line"),
            TitleLayoutError::EmptyDate => {
                write!(
                    f,
                    "date text is blank; use DateField::Suppressed to omit it"
                )
            }
        }
    }
}

impl std::error::Error for TitleLayoutError {}
