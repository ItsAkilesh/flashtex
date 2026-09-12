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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: SourcePos,
    pub end: SourcePos,
}

impl SourceSpan {
    /// Builds a span, rejecting one whose end precedes its start.
    pub fn new(start: SourcePos, end: SourcePos) -> Result<SourceSpan, SpanError> {
        if end.offset < start.offset {
            return Err(SpanError::Inverted { start, end });
        }
        Ok(SourceSpan { start, end })
    }

    /// Length in bytes.
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
}
