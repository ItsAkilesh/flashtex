//! FlashTeX document statistics: bounded counts computed from explicit
//! rendered content.
//!
//! This crate computes word, math, and page counts for a document from
//! items the caller supplies directly: [`SourceItem::Text`],
//! [`SourceItem::Math`], and [`SourceItem::PageMark`]. It does **no**
//! filesystem or network access, and it does not itself render, parse, or
//! tokenize TeX — turning an actual document into this explicit item list
//! is entirely the caller's job.
//!
//! Every [`Statistics`] value is bound to the exact [`RevisionId`] and
//! content it was computed from — see [`Statistics::is_current_for`] — so a
//! result computed for an old revision can never be silently mistaken for a
//! current one.
//!
//! Word counting is where this crate is least able to hide behind "just
//! count them": see the [`words`] module for the exact rule used and its
//! documented limitations (CJK text, hyphenation, apostrophes, math is not
//! prose).
//!
//! [`ScanLimit`] bounds how many source bytes one computation may scan,
//! raising [`ScanTooLarge`] instead of scanning without limit. [`ProjectCache`]
//! caches [`Statistics`] by exact source identity (revision id AND content,
//! never revision id alone) across several documents in one project, so
//! that editing one document does not force rescanning the others — see the
//! [`cache`] module docs for the exact hit/miss rule.
//!
//! ```
//! use flashtex_document_statistics::{RevisionId, SourceItem, Statistics};
//!
//! let items = vec![
//!     SourceItem::PageMark,
//!     SourceItem::text("Well-known results don't need proof."),
//!     SourceItem::inline_math("x^2"),
//! ];
//! let stats = Statistics::compute(RevisionId::new("draft.tex", 1), &items);
//! assert_eq!(stats.words.words, 5); // "Well-known" "results" "don't" "need" "proof."
//! assert_eq!(stats.math.total, 1);
//! assert_eq!(stats.pages, 1);
//! assert!(stats.is_current_for(&RevisionId::new("draft.tex", 1), &items));
//!
//! // Edit the text without bumping the revision: the cached stats are stale.
//! let edited = vec![SourceItem::text("Well-known results don't need proof, actually.")];
//! assert!(!stats.is_current_for(&RevisionId::new("draft.tex", 1), &edited));
//! ```

pub mod cache;
pub mod items;
pub mod revision;
pub mod scan_limit;
pub mod statistics;
pub mod words;

pub use cache::{Lookup, ProjectCache, ProjectTotals};
pub use items::{MathItem, SourceItem};
pub use revision::RevisionId;
pub use scan_limit::{ScanLimit, ScanTooLarge};
pub use statistics::{MathStats, Statistics};
pub use words::WordStats;
