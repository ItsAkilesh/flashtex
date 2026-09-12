//! FlashTeX math layout: an original Rust implementation of TeX's math
//! typesetting rules (TeXbook Appendix G) that turns a math list into
//! explicit boxes carrying glyph identity and rule geometry.
//!
//! Pipeline: build a [`MathList`] of [`Atom`]s → [`layout()`] it in a
//! [`Style`] against a [`MathFontMetrics`] provider → get a [`MathBox`] tree →
//! [`positioned_runs`] flattens it into glyphs and rules in points.
//!
//! No TeX engine is involved at runtime. The Computer Modern adapter embeds
//! metrics extracted from TFM files at development time; see `README.md`.

use std::fmt;

pub mod boxes;
pub mod cm;
pub mod cm_tfm;
#[doc(hidden)]
pub mod fixtures;
pub mod layout;
pub mod mathlist;
pub mod metrics;
pub mod spacing;
pub mod style;
pub mod tfm;
pub mod times;

pub use boxes::{
    BoxKind, Child, MathBox, PositionedGlyph, PositionedRule, PositionedRuns, positioned_runs,
};
pub use cm::CmMathMetrics;
pub use layout::{Layout, Limitation, layout, layout_with_report};
pub use mathlist::{Atom, AtomClass, Limits, MathList, Nucleus};
pub use metrics::{FontId, Glyph, MathFontMetrics, MathParams, OpenTypeMathConstants, SizeClass};
pub use spacing::{Space, between};
pub use style::{Style, StyleLevel};
pub use times::TimesApproxMetrics;

/// Errors from computing TFM-derived font metrics ([`tfm`], [`cm`]):
/// conditions those modules cannot recover from, so they report a typed
/// error rather than panicking or silently substituting a plausible-looking
/// but wrong number.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// [`tfm::scale`] was asked to scale a TFM dimension to `at_pt`, but at
    /// that size TeX's own fixed-point scaling algorithm (`store_scaled`,
    /// tex.web §571-572) makes the scaling factor `beta` truncate to zero,
    /// which would divide by zero. TeX itself never reaches a design size
    /// this large (`\maxdimen` bounds it well below this threshold), so this
    /// is out-of-range input to reject, not a value to clamp and continue
    /// with.
    PointSizeTooLarge { at_pt: f64 },
    /// A `&'static TfmFont` was looked up for a [`metrics::FontId`], but it
    /// is not one of [`cm::ALL_FONTS`], so no id can be assigned to it.
    UnknownFont { name: &'static str },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::PointSizeTooLarge { at_pt } => write!(
                f,
                "cannot scale a font dimension to {at_pt}pt: the point size is too large for TeX's fixed-point scaling"
            ),
            Error::UnknownFont { name } => write!(
                f,
                "font {name:?} is not one of this metrics provider's own embedded fonts"
            ),
        }
    }
}

impl std::error::Error for Error {}
