//! Measured layout of `\maketitle` for `article`.
//!
//! Provenance: `article.cls` v1.4n (BasicTeX, TeX Live 2026) `\@maketitle`:
//!
//! ```tex
//! \def\@maketitle{%
//!   \newpage \null \vskip 2em%
//!   \begin{center}%
//!     {\LARGE \@title \par}%
//!     \vskip 1.5em%
//!     {\large \lineskip .5em
//!       \begin{tabular}[t]{c}\@author\end{tabular}\par}%
//!     \vskip 1em%
//!     {\large \@date}%
//!   \end{center}%
//!   \vskip 1.5em}
//! ```
//!
//! Two things this module deliberately does **not** measure, because they
//! need data `flashtex-document-style` does not expose:
//! - Horizontal extent: text widths, centering, and where `\@maketitle`
//!   line-breaks a long title or wraps multiple `\and`-separated authors
//!   across the page. That needs glyph metrics (a font-engine concern), not
//!   the vertical class metrics this crate consumes. Callers supply already
//!   broken lines.
//! - `\lineskip .5em`, the interline glue LaTeX substitutes when two lines'
//!   `\baselineskip` glue would be negative (rare for ordinary text sizes).
//!   Every line-to-line gap below uses plain `\baselineskip`.
//!
//! All four `\vskip` amounts are evaluated in `\normalsize` (the font active
//! when `\maketitle` is called; `\@maketitle` never changes the outer font),
//! and are plain TeX `\vskip`: unconditionally additive, unlike the
//! `\addvspace`-collapsing skips in [`crate::abstract_block`].

use flashtex_document_style::{Pt, SizeName, Stylesheet, font_size};

use crate::class::DocumentClass;
use crate::error::TitleLayoutError;

/// `\date{...}`, or its deliberate omission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateField {
    /// `\date{text}` with at least one non-blank line.
    Text(String),
    /// No `\date` command was given at all, or the user wrote `\date{}`
    /// intending to suppress the date line entirely.
    Suppressed,
}

/// One already-line-broken `\maketitle` input. Line breaking of the title or
/// of long author names is out of scope (see the module docs); every
/// `String` here is rendered as exactly one line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TitleBlockInput {
    /// `\title{...}`, pre-split on `\\`. Must have at least one non-blank line.
    pub title_lines: Vec<String>,
    /// `\author{...}`, one entry per `\and`-separated author, each
    /// pre-split on `\\` (name, affiliation, ...). Must be non-empty, and
    /// every author must have at least one non-blank line.
    pub author_lines: Vec<Vec<String>>,
    pub date: DateField,
}

/// What one measured row of the title block renders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowKind {
    /// `title_lines[.0]`, set `\LARGE`.
    TitleLine(usize),
    /// The `.1`-th line of `author_lines[.0]`, set `\large`. When authors
    /// have different line counts they still occupy the same row indices,
    /// matching the shared tabular row baselines `\@maketitle` produces;
    /// this crate does not place them left-to-right (see module docs).
    AuthorLine(usize, usize),
    /// `\date{...}`, set `\large`. Absent when the date is suppressed.
    DateLine,
}

/// One measured line: its kind, font size, and baseline measured downward
/// from the top of the block (i.e. from the `\vskip 2em` after `\null`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasuredRow {
    pub kind: RowKind,
    pub font_size: Pt,
    pub baseline_y: Pt,
}

/// The full measured `\maketitle` block.
#[derive(Clone, Debug, PartialEq)]
pub struct TitleBlockLayout {
    pub rows: Vec<MeasuredRow>,
    /// Distance from the top of the block (after `\null\vskip 2em`, i.e. the
    /// point `\@maketitle` starts drawing) to the block's end, including the
    /// trailing `\vskip 1.5em` after `\end{center}`.
    pub total_height: Pt,
}

fn require_article(class: DocumentClass) -> Result<(), TitleLayoutError> {
    if class.is_supported() {
        Ok(())
    } else {
        Err(TitleLayoutError::UnsupportedDocumentClass(class))
    }
}

fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

fn validate(input: &TitleBlockInput) -> Result<(), TitleLayoutError> {
    if input.title_lines.is_empty() || input.title_lines.iter().all(|l| is_blank(l)) {
        return Err(TitleLayoutError::EmptyTitle);
    }
    if input.author_lines.is_empty() {
        return Err(TitleLayoutError::NoAuthors);
    }
    for (i, lines) in input.author_lines.iter().enumerate() {
        if lines.is_empty() || lines.iter().all(|l| is_blank(l)) {
            return Err(TitleLayoutError::EmptyAuthorLine(i));
        }
    }
    if let DateField::Text(t) = &input.date
        && is_blank(t)
    {
        return Err(TitleLayoutError::EmptyDate);
    }
    Ok(())
}

/// Measures `\maketitle` for `class` under `sheet`.
///
/// Fails with [`TitleLayoutError::UnsupportedDocumentClass`] for anything
/// but `article`, and with the other [`TitleLayoutError`] variants for
/// blank titles, missing authors, or a blank (but present) date — never by
/// silently substituting a placeholder.
pub fn layout_title_block(
    class: DocumentClass,
    sheet: &Stylesheet,
    input: &TitleBlockInput,
) -> Result<TitleBlockLayout, TitleLayoutError> {
    require_article(class)?;
    validate(input)?;

    let base = sheet.base_size();
    let body_em = sheet.body_font();
    let title_font = font_size(base, SizeName::LARGE3);
    let author_font = font_size(base, SizeName::Large);
    let date_font = font_size(base, SizeName::Large);

    let mut rows = Vec::new();
    let mut y = body_em.em(2.0); // \null \vskip 2em

    for (i, _) in input.title_lines.iter().enumerate() {
        y += title_font.baselineskip;
        rows.push(MeasuredRow {
            kind: RowKind::TitleLine(i),
            font_size: title_font.size,
            baseline_y: y,
        });
    }

    y += body_em.em(1.5); // \vskip 1.5em

    let author_row_count = input
        .author_lines
        .iter()
        .map(|a| a.len())
        .max()
        .unwrap_or(0);
    for line_idx in 0..author_row_count {
        y += author_font.baselineskip;
        for (author_idx, lines) in input.author_lines.iter().enumerate() {
            if line_idx < lines.len() {
                rows.push(MeasuredRow {
                    kind: RowKind::AuthorLine(author_idx, line_idx),
                    font_size: author_font.size,
                    baseline_y: y,
                });
            }
        }
    }

    y += body_em.em(1.0); // \vskip 1em

    if let DateField::Text(_) = &input.date {
        y += date_font.baselineskip;
        rows.push(MeasuredRow {
            kind: RowKind::DateLine,
            font_size: date_font.size,
            baseline_y: y,
        });
    }

    y += body_em.em(1.5); // trailing \vskip 1.5em after \end{center}

    Ok(TitleBlockLayout {
        rows,
        total_height: y,
    })
}
