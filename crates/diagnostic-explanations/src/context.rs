//! Bounded source context around a diagnostic span.

use crate::text;

/// Byte budget on each side of the span, in addition to the line bound.
pub const MAX_SIDE_BYTES: usize = 200;

/// A bounded excerpt of the document around the diagnostic's span.
///
/// The window covers the line(s) containing the span plus one line before and
/// one after, but never more than [`MAX_SIDE_BYTES`] bytes beyond either end of
/// the span. Both window ends and both span ends are character boundaries, so
/// the excerpt never splits a Unicode scalar and `text[..]` slicing is safe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextWindow {
    pub path: String,
    /// Absolute byte range of `text` within the document.
    pub start_byte: usize,
    pub end_byte: usize,
    pub text: String,
    /// The diagnostic span, clamped to the document and snapped to boundaries.
    pub span_start: usize,
    pub span_end: usize,
    /// 1-based line and character column of `span_start`.
    pub line: usize,
    pub column: usize,
    /// `false` when the span fell outside the supplied text (stale revision or
    /// no text); the window is then empty and should not be shown as source.
    pub span_in_bounds: bool,
}

pub fn window(path: &str, document: &str, start: usize, end: usize) -> ContextWindow {
    let in_bounds = start <= end && end <= document.len();
    let (s, e) = text::clamp_span(document, start, end);
    if document.is_empty() {
        return ContextWindow {
            path: path.to_string(),
            start_byte: 0,
            end_byte: 0,
            text: String::new(),
            span_start: 0,
            span_end: 0,
            line: 1,
            column: 1,
            span_in_bounds: false,
        };
    }

    let this_line_start = text::line_start(document, s);
    let prev_line_start = if this_line_start == 0 {
        0
    } else {
        text::line_start(document, this_line_start - 1)
    };
    let lo = text::snap_up(
        document,
        prev_line_start.max(s.saturating_sub(MAX_SIDE_BYTES)),
    );

    let this_line_end = text::line_end(document, e);
    let next_line_end = if this_line_end >= document.len() {
        document.len()
    } else {
        text::line_end(document, this_line_end + 1)
    };
    let hi = text::snap_down(
        document,
        next_line_end.min(e.saturating_add(MAX_SIDE_BYTES)),
    );

    let (line, column) = text::line_and_column(document, s);
    ContextWindow {
        path: path.to_string(),
        start_byte: lo,
        end_byte: hi,
        text: document[lo..hi].to_string(),
        span_start: s,
        span_end: e,
        line,
        column,
        span_in_bounds: in_bounds,
    }
}

impl ContextWindow {
    /// The span's offsets relative to `text`, for highlighting the excerpt.
    pub fn span_in_window(&self) -> (usize, usize) {
        (
            self.span_start.saturating_sub(self.start_byte),
            self.span_end
                .saturating_sub(self.start_byte)
                .min(self.text.len()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_is_line_bounded_and_scalar_safe() {
        let doc = "line one\nsecond é line\nthird 😀 line\nfourth";
        let start = doc.find("é").unwrap();
        let w = window("main.tex", doc, start, start + 2);
        assert_eq!(w.text, "line one\nsecond é line\nthird 😀 line");
        assert_eq!((w.line, w.column), (2, 8));
        assert!(doc.is_char_boundary(w.start_byte) && doc.is_char_boundary(w.end_byte));
        assert!(w.span_in_bounds);
        let (a, b) = w.span_in_window();
        assert_eq!(&w.text[a..b], "é");
    }

    #[test]
    fn window_caps_at_200_bytes_each_side_without_splitting() {
        let long = "😀".repeat(200); // 800 bytes, no newlines
        let doc = format!("{long}X{long}");
        let x = doc.find('X').unwrap();
        let w = window("m", &doc, x, x + 1);
        assert!(x - w.start_byte <= MAX_SIDE_BYTES);
        assert!(w.end_byte - (x + 1) <= MAX_SIDE_BYTES);
        assert!(doc.is_char_boundary(w.start_byte) && doc.is_char_boundary(w.end_byte));
        assert_eq!(w.text.chars().filter(|c| *c == '😀').count(), 100);
    }

    #[test]
    fn out_of_range_span_is_flagged() {
        let w = window("m", "abc", 10, 12);
        assert!(!w.span_in_bounds);
        assert_eq!((w.span_start, w.span_end), (3, 3));
        let empty = window("m", "", 0, 1);
        assert!(!empty.span_in_bounds);
        assert_eq!(empty.text, "");
    }
}
