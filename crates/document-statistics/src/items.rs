//! Explicit rendered-content items supplied by the caller.
//!
//! This crate never reads a filesystem, opens a socket, or walks a TeX/PDF
//! object model on its own. Every word, math span, and page boundary it
//! counts is one the caller handed it directly as a [`SourceItem`]; turning
//! an actual document into this list is entirely the caller's job.

/// One piece of already-rendered document content, in reading order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceItem {
    /// Rendered prose text (paragraph text, captions, headings, footnote
    /// bodies, ...). See the [`crate::words`] module docs for exactly how
    /// this is turned into a word count, and for that rule's limitations.
    Text(String),
    /// A math formula. Its `source` is never scanned for words — math is
    /// not prose — and it always contributes exactly one to the math count,
    /// regardless of how long or short the formula is.
    Math(MathItem),
    /// Marks the start of one rendered page. Emit exactly one `PageMark`
    /// per page, including the first one: page count is simply the number
    /// of `PageMark` items present. A document with text but no `PageMark`
    /// items has a page count of zero — this crate does not guess a page
    /// count from text volume.
    PageMark,
}

/// A single math formula as supplied by the caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MathItem {
    /// The formula's source text (e.g. TeX math-mode content). Kept for
    /// identity and debugging only; it is never word-counted.
    pub source: String,
    /// `true` for display math (`\[ ... \]`, `equation`, ...), `false` for
    /// inline math (`$ ... $`).
    pub display: bool,
}

impl SourceItem {
    /// Shorthand for `SourceItem::Text(s.into())`.
    pub fn text(s: impl Into<String>) -> Self {
        SourceItem::Text(s.into())
    }

    /// Shorthand for an inline-math `SourceItem::Math`.
    pub fn inline_math(source: impl Into<String>) -> Self {
        SourceItem::Math(MathItem {
            source: source.into(),
            display: false,
        })
    }

    /// Shorthand for a display-math `SourceItem::Math`.
    pub fn display_math(source: impl Into<String>) -> Self {
        SourceItem::Math(MathItem {
            source: source.into(),
            display: true,
        })
    }
}
