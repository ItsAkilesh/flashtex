use std::fmt;

use flashtex_document_style::Pt;

use crate::class::DocumentClass;
use crate::title::RowKind;

/// Errors from measuring a title block or abstract.
///
/// There is no fallback variant: an unsupported document class, a malformed
/// input, a missing caller-supplied metric, or a block that does not fit
/// the page always returns `Err`, never a best-guess `Ok`.
#[derive(Clone, Debug, PartialEq)]
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
    /// [`crate::metrics::GlyphMetrics::advance_width`] returned `None` for
    /// `ch` at `size`. This crate never substitutes a nominal em, a
    /// space-width guess, or any other fabricated width for a glyph the
    /// caller could not measure.
    MissingGlyphMetric { ch: char, size: Pt },
    /// [`crate::metrics::GlyphMetrics::em`] returned `None` for `size`,
    /// needed for the fixed part of `\and`'s inter-author glue.
    MissingEmMetric { size: Pt },
    /// A title or date line's measured natural width exceeds the page's
    /// usable text width. Reported explicitly; this line is never clipped
    /// or silently allowed to overflow the page.
    RowTooWide {
        row: RowKind,
        natural_width: Pt,
        available_width: Pt,
    },
    /// The `\author{...}` group, laid out side by side, is wider than the
    /// page's usable text width. Reported explicitly, not clipped.
    AuthorGroupTooWide {
        natural_width: Pt,
        available_width: Pt,
    },
    /// The full measured title block is taller than the page's usable text
    /// height. Reported explicitly, not clipped or silently overflowed.
    BlockTallerThanPage {
        total_height: Pt,
        available_height: Pt,
    },
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
            TitleLayoutError::MissingGlyphMetric { ch, size } => write!(
                f,
                "no caller-supplied metric for {ch:?} at {size}; refusing to fabricate a width"
            ),
            TitleLayoutError::MissingEmMetric { size } => write!(
                f,
                "no caller-supplied em (quad) metric at {size}; refusing to fabricate the inter-author glue"
            ),
            TitleLayoutError::RowTooWide {
                row,
                natural_width,
                available_width,
            } => write!(
                f,
                "{row:?} is {natural_width} wide, wider than the page's usable text width {available_width}"
            ),
            TitleLayoutError::AuthorGroupTooWide {
                natural_width,
                available_width,
            } => write!(
                f,
                "the author group is {natural_width} wide, wider than the page's usable text width {available_width}"
            ),
            TitleLayoutError::BlockTallerThanPage {
                total_height,
                available_height,
            } => write!(
                f,
                "the title block is {total_height} tall, taller than the page's usable text height {available_height}"
            ),
        }
    }
}

impl std::error::Error for TitleLayoutError {}
