//! Measured layout of the one-column `abstract` environment for `article`.
//!
//! Provenance: `article.cls` v1.4n (BasicTeX, TeX Live 2026):
//!
//! ```tex
//! \newenvironment{abstract}{%
//!       \if@twocolumn \section*{\abstractname}%
//!       \else \small
//!         \begin{center}{\bfseries \abstractname}\end{center}%
//!         \quotation
//!       \fi}
//!       {\if@twocolumn\else \endquotation \fi}
//! \newenvironment{quotation}
//!     {\list{}{\listparindent 1.5em
//!              \itemindent    \listparindent
//!              \rightmargin   \leftmargin
//!              \parsep        \z@ \@plus\p@}%
//!      \item\relax}
//!     {\endlist}
//! ```
//!
//! (This module only measures the one-column form; article's `\if@twocolumn`
//! branch uses `\section*`, an entirely different — and unsupported — shape.)
//!
//! Both the centered heading and the `quotation` list are `\list`-based
//! environments (`center` is a `\trivlist`; `quotation` is `\list{}{...}` at
//! the same nesting depth 1), so the vertical gaps around them come from
//! `\@listi` (via [`flashtex_document_style::list_level`]) combined with
//! `\addvspace`, not plain `\vskip`: consecutive list boundaries collapse to
//! the larger skip rather than summing (see [`flashtex_document_style::Skip::addvspace`]).
//!
//! Not measured, because `flashtex-document-style` has no font metrics for
//! sizes other than `\normalsize`: `\listparindent`/`\itemindent` (`1.5em`
//! evaluated in the abstract's own `\small`, not the body font). A caller
//! that needs the abstract body's first-line indent must supply `\small`'s
//! `\fontdimen6` itself; this crate does not approximate it with the
//! `\normalsize` quad.

use flashtex_document_style::{Pt, SizeName, Skip, Stylesheet, font_size, list_level};

use crate::class::DocumentClass;
use crate::error::TitleLayoutError;

/// `quotation`'s `\parsep \z@ \@plus\p@` override (`article.cls` v1.4n),
/// independent of size: it replaces `\@listi`'s own `\parsep` outright.
const QUOTATION_PARSEP: Skip = Skip::new(0.0, 1.0, 0.0);

/// The measured `abstract` block (one-column form only).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AbstractLayout {
    /// `\small\bfseries` heading font size (`\abstractname`).
    pub heading_font_size: Pt,
    pub heading_baselineskip: Pt,
    /// Vertical glue before the heading, from whatever precedes the
    /// abstract (`\@listi` `\topsep` plus `\parskip`, `\addvspace`d against
    /// the preceding material's own trailing skip).
    pub gap_before_heading: Skip,
    /// Vertical glue between the heading and the abstract body's first
    /// paragraph (the `center`'s closing skip `\addvspace`d against
    /// `quotation`'s opening skip).
    pub gap_heading_to_body: Skip,
    /// `\small` body font size.
    pub body_font_size: Pt,
    pub body_baselineskip: Pt,
    /// `\leftmargin` and `\rightmargin` of the `quotation` list (equal, per
    /// `\rightmargin \leftmargin`).
    pub left_margin: Pt,
    pub right_margin: Pt,
    /// Vertical glue between body paragraphs inside the abstract
    /// (`quotation`'s `\parsep` override, not `\@listi`'s own `\parsep`).
    pub paragraph_gap: Skip,
}

fn require_article(class: DocumentClass) -> Result<(), TitleLayoutError> {
    if class.is_supported() {
        Ok(())
    } else {
        Err(TitleLayoutError::UnsupportedDocumentClass(class))
    }
}

/// Measures the one-column `abstract` environment for `class` under `sheet`.
///
/// Fails with [`TitleLayoutError::UnsupportedDocumentClass`] for anything
/// but `article` — including for `article` in two-column mode, which takes
/// the unmeasured `\section*` branch (`sheet.page_layout()` does not expose
/// column count, so callers must not call this for two-column documents).
pub fn layout_abstract(
    class: DocumentClass,
    sheet: &Stylesheet,
) -> Result<AbstractLayout, TitleLayoutError> {
    require_article(class)?;

    let base = sheet.base_size();
    let small = font_size(base, SizeName::Small);
    let parskip = sheet.parskip();
    let lp = list_level(base, 1);

    let heading_skip = lp.topsep.plus(parskip);
    let body_open_skip = lp.topsep.plus(parskip);

    Ok(AbstractLayout {
        heading_font_size: small.size,
        heading_baselineskip: small.baselineskip,
        gap_before_heading: heading_skip,
        gap_heading_to_body: heading_skip.addvspace(body_open_skip),
        body_font_size: small.size,
        body_baselineskip: small.baselineskip,
        left_margin: lp.leftmargin,
        right_margin: lp.leftmargin,
        paragraph_gap: QUOTATION_PARSEP,
    })
}
