//! Where a [`crate::SnippetPlan`] targets the document.

use std::ops::Range;

/// Where a plan targets the document: either a single insertion caret, or
/// a selection that the expansion replaces — expressed as byte offsets
/// into the document text the plan was computed against.
///
/// This type does not itself validate its offsets; nothing you can build
/// directly from `Anchor::Caret`/`Anchor::Selection` is trusted as-is.
/// [`crate::SnippetPlan::compute`] is the only place an `Anchor` is turned
/// into a [`crate::SnippetPlan`], and it validates both offsets against
/// the exact document text passed to it — rejecting anything that is not
/// a valid `char` boundary (including past the end of the text, or a
/// reversed selection) with a typed [`crate::PlanError`] instead of
/// accepting it silently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Anchor {
    /// A single insertion point with no selected text.
    Caret(usize),
    /// A selected range, replaced by the plan's expansion.
    Selection(Range<usize>),
}

impl Anchor {
    /// The half-open byte range this anchor covers: `at..at` for a caret,
    /// or the selection itself.
    pub fn range(&self) -> Range<usize> {
        match self {
            Anchor::Caret(at) => *at..*at,
            Anchor::Selection(range) => range.clone(),
        }
    }
}
