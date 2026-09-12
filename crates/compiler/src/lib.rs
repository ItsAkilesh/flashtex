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
