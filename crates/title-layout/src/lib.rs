//! FlashTeX title-block layout: an original measured-layout adapter for
//! LaTeX `article`'s `\maketitle` (title/author/date) and one-column
//! `abstract` environment.
//!
//! This crate does no text shaping, line breaking, or width measurement of
//! its own. It consumes [`flashtex_document_style::Stylesheet`] — font
//! sizes, `\baselineskip`, `\parskip`, and list margins transcribed from
//! `article.cls` — and turns already-line-broken title/author/date text
//! into a vertical measured layout: baseline positions, font sizes, and the
//! vertical glue LaTeX inserts around each piece.
//!
//! `flashtex-document-style` only models the `article` class, so this crate
//! only measures `article`. Calling either entry point with any other
//! [`DocumentClass`] fails with [`TitleLayoutError::UnsupportedDocumentClass`]
//! — never a best-guess approximation of a class this crate has no
//! measurements for.
//!
//! ```
//! use flashtex_document_style::{BaseSize, ClassOptions, Paper, Stylesheet};
//! use flashtex_title_layout::{DateField, DocumentClass, TitleBlockInput, layout_title_block};
//!
//! let sheet = Stylesheet::article(ClassOptions { paper: Paper::Letter, size: BaseSize::Pt10 });
//! let input = TitleBlockInput {
//!     title_lines: vec!["A Study of Measured Layout".to_string()],
//!     author_lines: vec![vec!["A. Author".to_string()]],
//!     date: DateField::Text("\\today".to_string()),
//! };
//! let layout = layout_title_block(DocumentClass::Article, &sheet, &input).unwrap();
//! assert_eq!(layout.rows.len(), 3); // title, author, date
//!
//! // report has no measurements in flashtex-document-style, so it is
//! // rejected explicitly rather than approximated with article's numbers.
//! assert!(layout_title_block(DocumentClass::Report, &sheet, &input).is_err());
//! ```

pub mod abstract_block;
pub mod class;
pub mod error;
pub mod metrics;
pub mod title;

pub use abstract_block::{AbstractLayout, layout_abstract};
pub use class::DocumentClass;
pub use error::TitleLayoutError;
pub use metrics::GlyphMetrics;
pub use title::{
    DateField, HorizontalExtent, MeasuredRow, MeasuredTitleBlock, RowKind, TitleBlockInput,
    TitleBlockLayout, layout_title_block, layout_title_block_with_metrics,
};
