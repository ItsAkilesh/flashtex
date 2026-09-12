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
    /// [`crate::metrics::GlyphMetrics::advance_width`] returned `Some`, but
    /// the value is not a usable width: negative, infinite, or `NaN`. This
    /// crate never lets such a value flow into layout arithmetic, where it
    /// could silently produce a negative or infinite position, or make a
    /// page-fit comparison vacuously true or false.
    InvalidGlyphMetric { ch: char, size: Pt, value: Pt },
    /// [`crate::metrics::GlyphMetrics::em`] returned `Some`, but the value
    /// is not usable: negative, infinite, or `NaN`.
    InvalidEmMetric { size: Pt, value: Pt },
    /// The page's usable text width or height
    /// ([`flashtex_document_style::PageLayout::text_area`]) is zero,
    /// negative, or non-finite. This crate refuses to lay out against a
    /// degenerate page rather than computing a nonsense centering offset or
    /// letting every width comparison against it come out wrong.
    InvalidPageArea { width: Pt, height: Pt },
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
            TitleLayoutError::InvalidGlyphMetric { ch, size, value } => write!(
                f,
                "caller-supplied metric for {ch:?} at {size} is {value}, not a usable width (must be finite and non-negative)"
            ),
            TitleLayoutError::InvalidEmMetric { size, value } => write!(
                f,
                "caller-supplied em (quad) metric at {size} is {value}, not usable (must be finite and non-negative)"
            ),
            TitleLayoutError::InvalidPageArea { width, height } => write!(
                f,
                "the page's usable text area is {width} x {height}, not usable (width and height must both be finite and positive)"
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
