//! Source-mapped positions: where in the original document source a link
//! annotation was written, for diagnostics. Nothing here resolves a span
//! back to a file or reads anything — it is plain data with a shape check.

use std::fmt;

/// A single position in document source: a 0-based byte offset plus the
/// 1-based line/column a human would see in an editor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourcePos {
    pub offset: u32,
    pub line: u32,
    pub column: u32,
}

impl SourcePos {
    pub fn new(offset: u32, line: u32, column: u32) -> SourcePos {
        SourcePos {
            offset,
            line,
            column,
        }
    }
}

/// A half-open span `[start, end)` in document source.
///
/// `start` and `end` are deliberately private: [`SourceSpan::new`] is the
/// only way to build one, so `end.offset >= start.offset` holds for every
/// live `SourceSpan`. If the fields were `pub`, a caller could assemble a
/// `SourceSpan { start, end }` struct literal directly, skip that check, and
/// hand [`SourceSpan::len`] an inverted span — `end.offset - start.offset`
/// would then panic on overflow in a debug build but silently wrap to a huge
/// `u32` in release.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceSpan {
    start: SourcePos,
    end: SourcePos,
}

impl SourceSpan {
    /// Builds a span, rejecting one whose end precedes its start.
    pub fn new(start: SourcePos, end: SourcePos) -> Result<SourceSpan, SpanError> {
        if end.offset < start.offset {
            return Err(SpanError::Inverted { start, end });
        }
        Ok(SourceSpan { start, end })
    }

    pub fn start(&self) -> SourcePos {
        self.start
    }

    pub fn end(&self) -> SourcePos {
        self.end
    }

    /// Length in bytes. Safe by construction: every `SourceSpan` in
    /// existence satisfies `end.offset >= start.offset` (see the struct
    /// doc comment), so this subtraction can never underflow.
    pub fn len(&self) -> u32 {
        self.end.offset - self.start.offset
    }

    pub fn is_empty(&self) -> bool {
        self.start.offset == self.end.offset
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanError {
    Inverted { start: SourcePos, end: SourcePos },
}

impl fmt::Display for SpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpanError::Inverted { start, end } => write!(
                f,
                "span end offset {} precedes start offset {}",
                end.offset, start.offset
            ),
        }
    }
}

impl std::error::Error for SpanError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_inverted_span() {
        let start = SourcePos::new(50, 3, 1);
        let end = SourcePos::new(10, 1, 1);
        let err = SourceSpan::new(start, end).unwrap_err();
        assert_eq!(err, SpanError::Inverted { start, end });
    }

    #[test]
    fn accepts_and_measures_span() {
        let start = SourcePos::new(10, 2, 5);
        let end = SourcePos::new(25, 2, 20);
        let span = SourceSpan::new(start, end).unwrap();
        assert_eq!(span.len(), 15);
        assert!(!span.is_empty());
    }

    #[test]
    fn empty_span_at_same_offset() {
        let p = SourcePos::new(7, 1, 8);
        let span = SourceSpan::new(p, p).unwrap();
        assert!(span.is_empty());
        assert_eq!(span.len(), 0);
    }

    /// Regression test for a debug-panics/release-wraps defect: `start` and
    /// `end` used to be `pub`, so a caller could skip `SourceSpan::new`'s
    /// ordering check entirely via a `SourceSpan { start, end }` struct
    /// literal. With an inverted, u32-extreme span like this one,
    /// `len()`'s `end.offset - start.offset` panicked with "attempt to
    /// subtract with overflow" under `cargo test`, but silently returned
    /// 4294967201 (a nonsense length wrapped from -95) under
    /// `cargo test --release` — a wrong value with no error anywhere.
    ///
    /// Now that the fields are private, `SourceSpan::new` is the only way
    /// to build one (the struct-literal bypass is a compile error from
    /// outside this module), so this same input must come back as a typed
    /// `SpanError::Inverted` in every build profile — never a constructed
    /// span, and never a value from `len()` at all.
    #[test]
    fn inverted_span_at_u32_extremes_is_a_typed_error_not_a_wrapped_length() {
        let start = SourcePos::new(100, 1, 1);
        let end = SourcePos::new(5, 1, 1);
        let err = SourceSpan::new(start, end).unwrap_err();
        assert_eq!(err, SpanError::Inverted { start, end });
    }
}
