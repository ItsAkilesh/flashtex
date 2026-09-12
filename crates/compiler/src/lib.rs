//! FlashTeX compiler foundation.
//!
//! This is an original implementation. No existing TeX engine is invoked, linked,
//! or shelled out to. It implements a deliberately finite, documented subset of
//! LaTeX (see `README.md`); anything outside that subset produces an explicit
//! diagnostic rather than silently rendering or silently vanishing.
//!
//! Byte offsets are the contract's currency: every span is a zero-based,
//! end-exclusive UTF-8 byte range into the exact input text of the stated
//! revision, per `docs/contracts/runtime-v1.md`.

pub mod diagnostics;
pub mod export;
pub mod incremental;
pub mod json;
pub mod layout;
pub mod lexer;
pub mod math;
pub mod parser;
pub mod protocol;

/// Stable identity of one document in a compile request.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentId(pub usize);

/// A zero-based, end-exclusive UTF-8 byte range into a source document.
///
/// Invariant: `start <= end`, both land on UTF-8 character boundaries of the
/// document they refer to, so `&text[start..end]` never panics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub document: DocumentId,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self::in_document(DocumentId::default(), start, end)
    }

    pub fn in_document(document: DocumentId, start: usize, end: usize) -> Self {
        debug_assert!(start <= end, "span start must not exceed end");
        Span {
            document,
            start,
            end,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Smallest span covering both inputs.
    pub fn merge(self, other: Span) -> Span {
        debug_assert_eq!(
            self.document, other.document,
            "cannot merge spans from different documents"
        );
        Span::in_document(
            self.document,
            self.start.min(other.start),
            self.end.max(other.end),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── DocumentId ────────────────────────────────────────────────────────────

    #[test]
    fn document_id_default_is_zero() {
        assert_eq!(DocumentId::default(), DocumentId(0));
    }

    #[test]
    fn document_id_ordering() {
        assert!(DocumentId(0) < DocumentId(1));
        assert!(DocumentId(1) < DocumentId(2));
    }

    #[test]
    fn document_id_equality() {
        assert_eq!(DocumentId(3), DocumentId(3));
        assert_ne!(DocumentId(3), DocumentId(4));
    }

    // ── Span::new ─────────────────────────────────────────────────────────────

    #[test]
    fn span_new_stores_bounds_in_default_document() {
        let s = Span::new(3, 10);
        assert_eq!(s.start, 3);
        assert_eq!(s.end, 10);
        assert_eq!(s.document, DocumentId::default());
    }

    // ── Span::in_document ─────────────────────────────────────────────────────

    #[test]
    fn span_in_document_stores_document_and_bounds() {
        let doc = DocumentId(2);
        let s = Span::in_document(doc, 5, 15);
        assert_eq!(s.document, doc);
        assert_eq!(s.start, 5);
        assert_eq!(s.end, 15);
    }

    #[test]
    fn span_in_document_zero_is_same_as_new() {
        let a = Span::new(4, 8);
        let b = Span::in_document(DocumentId(0), 4, 8);
        assert_eq!(a, b);
    }

    // ── Span::is_empty ────────────────────────────────────────────────────────

    #[test]
    fn span_is_empty_when_start_equals_end() {
        assert!(Span::new(5, 5).is_empty());
    }

    #[test]
    fn span_is_not_empty_when_start_less_than_end() {
        assert!(!Span::new(0, 1).is_empty());
    }

    #[test]
    fn zero_length_span_at_origin_is_empty() {
        assert!(Span::new(0, 0).is_empty());
    }

    // ── Span::merge ───────────────────────────────────────────────────────────

    #[test]
    fn span_merge_covers_both() {
        let a = Span::new(2, 5);
        let b = Span::new(4, 9);
        let m = a.merge(b);
        assert_eq!(m.start, 2);
        assert_eq!(m.end, 9);
    }

    #[test]
    fn span_merge_is_commutative() {
        let a = Span::new(0, 4);
        let b = Span::new(7, 12);
        assert_eq!(a.merge(b), b.merge(a));
    }

    #[test]
    fn span_merge_adjacent_spans_cover_both() {
        let a = Span::new(0, 3);
        let b = Span::new(3, 6);
        assert_eq!(a.merge(b), Span::new(0, 6));
    }

    #[test]
    fn span_merge_with_self_is_identity() {
        let s = Span::new(2, 8);
        assert_eq!(s.merge(s), s);
    }

    #[test]
    fn span_merge_same_document_preserved() {
        let doc = DocumentId(1);
        let a = Span::in_document(doc, 0, 5);
        let b = Span::in_document(doc, 3, 10);
        let m = a.merge(b);
        assert_eq!(m.document, doc);
        assert_eq!(m.start, 0);
        assert_eq!(m.end, 10);
    }

    // ── Span equality ─────────────────────────────────────────────────────────

    #[test]
    fn span_equality_includes_document() {
        let a = Span::in_document(DocumentId(0), 1, 4);
        let b = Span::in_document(DocumentId(1), 1, 4);
        // Same byte range, different documents → not equal.
        assert_ne!(a, b);
    }

    #[test]
    fn span_equality_same_document_same_range() {
        assert_eq!(Span::new(1, 4), Span::new(1, 4));
        assert_ne!(Span::new(1, 4), Span::new(1, 5));
    }
}
