//! A snippet expansion bound to the exact document state and location it
//! was computed against, and never anything more than that.

use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

use crate::anchor::Anchor;
use crate::error::SnippetError;
use crate::expand::Expansion;
use crate::identity::{DocumentId, Staleness};
use crate::model::Snippet;

/// Everything that can go wrong computing a [`SnippetPlan`], beyond what
/// [`SnippetError`] already covers for parsing and expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// Expanding the snippet itself failed; see [`SnippetError`].
    Snippet(SnippetError),
    /// An anchor offset is not a valid `char` boundary in the document
    /// text passed to [`SnippetPlan::compute`] — including an offset past
    /// the end of that text.
    InvalidOffset {
        /// The rejected offset.
        offset: usize,
    },
    /// A selection anchor's start is after its end.
    SelectionReversed {
        /// The selection's start offset.
        start: usize,
        /// The selection's end offset.
        end: usize,
    },
}

impl From<SnippetError> for PlanError {
    fn from(err: SnippetError) -> Self {
        PlanError::Snippet(err)
    }
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanError::Snippet(err) => write!(f, "{err}"),
            PlanError::InvalidOffset { offset } => {
                write!(f, "anchor offset {offset} is not a valid char boundary")
            }
            PlanError::SelectionReversed { start, end } => {
                write!(f, "selection start {start} is after end {end}")
            }
        }
    }
}

impl std::error::Error for PlanError {}

/// A snippet expansion bound to the exact document state and location it
/// was computed against: a [`DocumentId`] (revision id plus content hash)
/// and an [`Anchor`] (the caret or selection the expansion targets).
///
/// A `SnippetPlan` is inert data. Computing one never touches, borrows, or
/// requires mutable access to any document buffer — [`SnippetPlan::compute`]
/// takes the document's text by shared reference only — and this crate
/// exposes no method on `SnippetPlan` that applies, writes, or mutates
/// anything; every method here is a getter or a pure comparison. Splicing
/// [`Self::expansion`]'s text into the real document at [`Self::anchor`],
/// and re-computing a fresh plan after any edit, is entirely the caller's
/// job, and the API gives it no other way to happen.
///
/// Before applying a plan, call [`Self::staleness`] against the document's
/// *current* [`DocumentId`] and refuse to apply anything but
/// [`Staleness::Fresh`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnippetPlan {
    document: DocumentId,
    anchor: Anchor,
    expansion: Expansion,
}

impl SnippetPlan {
    /// Computes a plan for `snippet`, substituting `overrides[&index]` for
    /// that placeholder's resolved text exactly as [`Snippet::expand_with`]
    /// does, anchored at `anchor` in `document_text` at `revision`.
    ///
    /// # Errors
    /// [`PlanError::InvalidOffset`] or [`PlanError::SelectionReversed`] if
    /// `anchor`'s offsets are not valid for `document_text`; otherwise any
    /// [`SnippetError`] from expansion, wrapped in [`PlanError::Snippet`].
    pub fn compute(
        snippet: &Snippet,
        revision: u64,
        document_text: &str,
        anchor: Anchor,
        overrides: &HashMap<u32, String>,
    ) -> Result<SnippetPlan, PlanError> {
        validate_anchor(&anchor, document_text)?;
        let expansion = snippet.expand_with(overrides)?;
        let document = DocumentId::new(revision, document_text);
        Ok(SnippetPlan {
            document,
            anchor,
            expansion,
        })
    }

    /// The document identity this plan was computed against.
    pub fn document(&self) -> &DocumentId {
        &self.document
    }

    /// The caret or selection this plan targets in that document.
    pub fn anchor(&self) -> &Anchor {
        &self.anchor
    }

    /// The computed expansion: text plus placeholder spans.
    pub fn expansion(&self) -> &Expansion {
        &self.expansion
    }

    /// Compares the [`DocumentId`] this plan was computed against with
    /// `current`. Only [`Staleness::Fresh`] means [`Self::anchor`] and
    /// [`Self::expansion`] are still safe to apply to the document as-is.
    pub fn staleness(&self, current: &DocumentId) -> Staleness {
        self.document.compare(current)
    }
}

fn validate_anchor(anchor: &Anchor, text: &str) -> Result<(), PlanError> {
    let Range { start, end } = anchor.range();
    if start > end {
        return Err(PlanError::SelectionReversed { start, end });
    }
    if !text.is_char_boundary(start) {
        return Err(PlanError::InvalidOffset { offset: start });
    }
    if !text.is_char_boundary(end) {
        return Err(PlanError::InvalidOffset { offset: end });
    }
    Ok(())
}
