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
pub mod json;
pub mod layout;
pub mod lexer;
pub mod parser;
pub mod cache;
pub mod protocol;

/// A zero-based, end-exclusive UTF-8 byte range into a source document.
///
/// Invariant: `start <= end`, both land on UTF-8 character boundaries of the
/// document they refer to, so `&text[start..end]` never panics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= end, "span start must not exceed end");
        Span { start, end }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Smallest span covering both inputs.
    pub fn merge(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }
}
pub mod pdf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_new_stores_bounds() {
        let s = Span::new(3, 10);
        assert_eq!(s.start, 3);
        assert_eq!(s.end, 10);
    }

    #[test]
    fn span_is_empty_when_start_equals_end() {
        assert!(Span::new(5, 5).is_empty());
    }

    #[test]
    fn span_is_not_empty_when_start_less_than_end() {
        assert!(!Span::new(0, 1).is_empty());
    }

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
    fn span_merge_with_adjacent_spans() {
        // Adjacent (non-overlapping) spans merge to cover both exactly.
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
    fn span_equality() {
        assert_eq!(Span::new(1, 4), Span::new(1, 4));
        assert_ne!(Span::new(1, 4), Span::new(1, 5));
    }
}
