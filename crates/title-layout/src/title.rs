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
use crate::metrics::GlyphMetrics;

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

/// One row's horizontal placement, added by [`layout_title_block_with_metrics`].
/// `x` is measured from the left edge of the page's usable text width (see
/// [`flashtex_document_style::PageLayout::text_area`]), matching how
/// `\begin{center}` centers each title/date line, and how the `tabular[t]{c}`
/// author group is centered as a whole with each author's own lines then
/// centered within that author's column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HorizontalExtent {
    /// Natural width of this row's text (or, for an author line, that
    /// author's own widest line — the tabular column's natural width).
    pub width: Pt,
    /// Offset from the left edge of the usable text width to this row's
    /// left edge.
    pub x: Pt,
}

/// [`TitleBlockLayout`] plus a [`HorizontalExtent`] for every row, in the
/// same order as `layout.rows`.
#[derive(Clone, Debug, PartialEq)]
pub struct MeasuredTitleBlock {
    pub layout: TitleBlockLayout,
    /// Same length and order as `layout.rows`.
    pub extents: Vec<HorizontalExtent>,
}

fn pt_max(a: Pt, b: Pt) -> Pt {
    if b.0 > a.0 { b } else { a }
}

/// A width is only usable when it is finite and non-negative. A caller's
/// metrics provider that returns zero, negative, infinite, or `NaN` gets
/// rejected here rather than being allowed to flow into baseline/centering
/// arithmetic, where it could silently produce a negative or infinite
/// position, or flip a page-fit comparison the wrong way (a `NaN` width
/// compares `false` against everything, so an unchecked `>` check would
/// never trip).
fn is_usable_width(w: Pt) -> bool {
    w.0.is_finite() && w.0 >= 0.0
}

/// Measures one line's natural width by summing caller-supplied glyph
/// advances one Unicode scalar value at a time (never per byte, so a
/// multi-byte character costs exactly one query). Fails with
/// [`TitleLayoutError::MissingGlyphMetric`] at the first character the
/// caller cannot measure, naming exactly that character, and with
/// [`TitleLayoutError::InvalidGlyphMetric`] at the first character whose
/// measured width is not finite and non-negative — never a substituted or
/// silently accepted nonsense width.
fn measure_line(metrics: &dyn GlyphMetrics, line: &str, size: Pt) -> Result<Pt, TitleLayoutError> {
    let mut width = Pt::ZERO;
    for ch in line.chars() {
        let w = metrics
            .advance_width(ch, size)
            .ok_or(TitleLayoutError::MissingGlyphMetric { ch, size })?;
        if !is_usable_width(w) {
            return Err(TitleLayoutError::InvalidGlyphMetric { ch, size, value: w });
        }
        width += w;
    }
    Ok(width)
}

/// Measures `\maketitle` for `class` under `sheet`, additionally computing
/// horizontal placement (line widths, centering, and multi-author
/// side-by-side placement) from caller-supplied `metrics`, and checking the
/// result against the page's usable text width and height.
///
/// This is [`layout_title_block`] plus everything that function's module
/// docs say it deliberately does not measure. It fails with every error
/// [`layout_title_block`] can return, plus:
/// - [`TitleLayoutError::MissingGlyphMetric`] / [`TitleLayoutError::MissingEmMetric`]
///   when `metrics` has no measurement for a character or em the layout needs —
///   never a fabricated substitute.
/// - [`TitleLayoutError::RowTooWide`] / [`TitleLayoutError::AuthorGroupTooWide`]
///   when a line or the author group is wider than the page's usable text
///   width — never silently clipped.
/// - [`TitleLayoutError::BlockTallerThanPage`] when the block is taller than
///   the page's usable text height — never silently overflowed.
pub fn layout_title_block_with_metrics(
    class: DocumentClass,
    sheet: &Stylesheet,
    input: &TitleBlockInput,
    metrics: &dyn GlyphMetrics,
) -> Result<MeasuredTitleBlock, TitleLayoutError> {
    let layout = layout_title_block(class, sheet, input)?;

    let text_area = sheet.page_layout().text_area;
    let text_width = text_area.width;

    let page_area_usable = text_width.0.is_finite()
        && text_width.0 > 0.0
        && text_area.height.0.is_finite()
        && text_area.height.0 > 0.0;
    if !page_area_usable {
        return Err(TitleLayoutError::InvalidPageArea {
            width: text_width,
            height: text_area.height,
        });
    }

    if layout.total_height > text_area.height {
        return Err(TitleLayoutError::BlockTallerThanPage {
            total_height: layout.total_height,
            available_height: text_area.height,
        });
    }

    let base = sheet.base_size();
    let title_font = font_size(base, SizeName::LARGE3);
    let author_font = font_size(base, SizeName::Large);
    let date_font = font_size(base, SizeName::Large);

    // Measure every author's lines up front: each author is one
    // `tabular[t]{c}` column, so its natural width is the widest of its own
    // lines, and each of its lines is centered within that width.
    let mut author_line_widths: Vec<Vec<Pt>> = Vec::with_capacity(input.author_lines.len());
    for lines in &input.author_lines {
        let mut widths = Vec::with_capacity(lines.len());
        for line in lines {
            widths.push(measure_line(metrics, line, author_font.size)?);
        }
        author_line_widths.push(widths);
    }
    let author_block_widths: Vec<Pt> = author_line_widths
        .iter()
        .map(|widths| widths.iter().copied().fold(Pt::ZERO, pt_max))
        .collect();

    // `\and` inserts `\hskip 1em \@plus.17fil` between authors' tabulars. At
    // natural width (nothing here stretches the block to a forced width)
    // only the fixed `1em` survives; the `.17fil` stretch never applies. A
    // single author needs no gap, and this crate never asks the caller for
    // an em it does not need.
    let author_count = author_block_widths.len();
    let mut total_author_width = author_block_widths
        .iter()
        .copied()
        .fold(Pt::ZERO, |a, b| a + b);
    let em_gap = if author_count > 1 {
        let gap = metrics
            .em(author_font.size)
            .ok_or(TitleLayoutError::MissingEmMetric {
                size: author_font.size,
            })?;
        if !is_usable_width(gap) {
            return Err(TitleLayoutError::InvalidEmMetric {
                size: author_font.size,
                value: gap,
            });
        }
        total_author_width += gap * (author_count as f64 - 1.0);
        gap
    } else {
        Pt::ZERO
    };
    if total_author_width > text_width {
        return Err(TitleLayoutError::AuthorGroupTooWide {
            natural_width: total_author_width,
            available_width: text_width,
        });
    }
    let mut author_x = Vec::with_capacity(author_count);
    let mut cursor = (text_width - total_author_width) * 0.5;
    for &w in &author_block_widths {
        author_x.push(cursor);
        cursor += w + em_gap;
    }

    finish_measured_block(
        layout,
        input,
        &author_line_widths,
        &author_block_widths,
        &author_x,
        metrics,
        title_font.size,
        date_font.size,
        text_width,
    )
}

#[allow(clippy::too_many_arguments)]
fn finish_measured_block(
    layout: TitleBlockLayout,
    input: &TitleBlockInput,
    author_line_widths: &[Vec<Pt>],
    author_block_widths: &[Pt],
    author_x: &[Pt],
    metrics: &dyn GlyphMetrics,
    title_size: Pt,
    date_size: Pt,
    text_width: Pt,
) -> Result<MeasuredTitleBlock, TitleLayoutError> {
    let mut extents = Vec::with_capacity(layout.rows.len());
    for row in &layout.rows {
        let extent = match row.kind {
            RowKind::TitleLine(i) => {
                let w = measure_line(metrics, &input.title_lines[i], title_size)?;
                if w > text_width {
                    return Err(TitleLayoutError::RowTooWide {
                        row: row.kind,
                        natural_width: w,
                        available_width: text_width,
                    });
                }
                HorizontalExtent {
                    width: w,
                    x: (text_width - w) * 0.5,
                }
            }
            RowKind::AuthorLine(a, l) => {
                let w = author_line_widths[a][l];
                let block_w = author_block_widths[a];
                HorizontalExtent {
                    width: w,
                    x: author_x[a] + (block_w - w) * 0.5,
                }
            }
            RowKind::DateLine => {
                let text = match &input.date {
                    DateField::Text(t) => t,
                    DateField::Suppressed => {
                        unreachable!("layout_title_block only emits DateLine for DateField::Text")
                    }
                };
                let w = measure_line(metrics, text, date_size)?;
                if w > text_width {
                    return Err(TitleLayoutError::RowTooWide {
                        row: row.kind,
                        natural_width: w,
                        available_width: text_width,
                    });
                }
                HorizontalExtent {
                    width: w,
                    x: (text_width - w) * 0.5,
                }
            }
        };
        extents.push(extent);
    }
    Ok(MeasuredTitleBlock { layout, extents })
}
