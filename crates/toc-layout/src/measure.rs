//! The typed adapter contract between this crate and a real text measurer
//! (font-engine, the compiler, or native layout).
//!
//! `toc-layout` does no font parsing or shaping itself. Anything that wants
//! real dot-leader math implements [`TextMeasure`] against its own glyph
//! metrics and hands it to [`crate::leader::layout_entry`]. This crate ships
//! one reference implementation, [`CharWidthMeasure`], for tests and for
//! callers with no font yet — it is a per-character width table, not a
//! shaping engine, and is not a parity claim against any real typesetting
//! output.

use std::collections::HashMap;

/// Measures the width of laid-out text and of one dot-leader unit, in
/// whatever length unit the caller uses consistently (TeX points, PDF
/// points, device pixels, ...).
///
/// Implementations MUST measure `text` as a single run with no line breaks,
/// and MUST measure by extended grapheme cluster (a user-perceived
/// character — e.g. "e" + combining acute, or a multi-codepoint emoji,
/// measures once), not by UTF-8 byte or by `char`. [`CharWidthMeasure`]
/// measures by `char` instead, as a documented simplification suitable for
/// tests and non-combining scripts; a caller layering real shaping on top
/// (e.g. font-engine) is expected to do grapheme-cluster measurement.
pub trait TextMeasure {
    /// Width of `text` laid out as one run.
    fn width(&self, text: &str) -> f64;

    /// Width of one leader unit (typically a dot plus its following
    /// inter-dot space, e.g. `.\,` in `article.cls`).
    fn leader_unit_width(&self) -> f64;
}

/// A measurer rejected an input by returning a non-finite or negative
/// width. Callers that see this from their own [`TextMeasure`] have a bug
/// in their metrics table, not in `toc-layout`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidWidth(pub f64);

impl std::fmt::Display for InvalidWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "measurer returned an invalid width: {}", self.0)
    }
}

impl std::error::Error for InvalidWidth {}

/// Checks that a width returned by a [`TextMeasure`] is finite and
/// non-negative.
pub(crate) fn checked_width(w: f64) -> Result<f64, InvalidWidth> {
    if w.is_finite() && w >= 0.0 {
        Ok(w)
    } else {
        Err(InvalidWidth(w))
    }
}

/// A reference [`TextMeasure`]: a default width plus per-`char` overrides.
/// With no overrides it is a uniform (monospace-like) measurer, which is
/// enough to test leader-fill arithmetic without a real font.
#[derive(Debug, Clone)]
pub struct CharWidthMeasure {
    default_width: f64,
    overrides: HashMap<char, f64>,
    leader_unit: f64,
}

impl CharWidthMeasure {
    /// `default_width` is used for any `char` with no override.
    /// `leader_unit` is the width of one dot-leader unit.
    pub fn new(default_width: f64, leader_unit: f64) -> Self {
        Self {
            default_width,
            overrides: HashMap::new(),
            leader_unit,
        }
    }

    /// Sets an explicit width for one `char` (e.g. a wide CJK character or
    /// a narrow digit), overriding the default width.
    #[must_use]
    pub fn with_override(mut self, ch: char, width: f64) -> Self {
        self.overrides.insert(ch, width);
        self
    }
}

impl TextMeasure for CharWidthMeasure {
    fn width(&self, text: &str) -> f64 {
        text.chars()
            .map(|c| *self.overrides.get(&c).unwrap_or(&self.default_width))
            .sum()
    }

    fn leader_unit_width(&self) -> f64 {
        self.leader_unit
    }
}
