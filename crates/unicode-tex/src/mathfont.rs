//! OpenType MATH constants for a `\setmathfont` face, in points, and the
//! LuaTeX `\Umath...` parameter each one initialises (the names math layout
//! code will look for when it emulates LuaLaTeX; XeTeX reads the same
//! constants).
//!
//! The MATH table itself is parsed by `crates/font-engine` (`math::MathTable`);
//! this module only selects, scales and names. Every mapping listed in
//! [`UMATH_PARAMETERS`] was compared with `\the\Umath..` from LuaLaTeX for
//! Latin Modern Math, STIX Two Math and New Computer Modern Math
//! (`fixtures/expected/math_*.lualatex.json`).

use flashtex_font_engine::math::MathConstants;
use flashtex_font_engine::{Face, TrueTypeFace};

/// Math style a `\Umath` parameter is read in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathStyle {
    Text,
    Display,
}

/// `(oracle key, \Umath name, style, constant accessor)`.
pub type UmathMapping = (
    &'static str,
    &'static str,
    MathStyle,
    fn(&MathConstants) -> i32,
);

pub const UMATH_PARAMETERS: &[UmathMapping] = &[
    ("axis", "Umathaxis", MathStyle::Text, |c| {
        c.axis_height.into()
    }),
    ("fractionrule", "Umathfractionrule", MathStyle::Text, |c| {
        c.fraction_rule_thickness.into()
    }),
    ("supshiftup", "Umathsupshiftup", MathStyle::Text, |c| {
        c.superscript_shift_up.into()
    }),
    ("subshiftdown", "Umathsubshiftdown", MathStyle::Text, |c| {
        c.subscript_shift_down.into()
    }),
    ("radicalrule", "Umathradicalrule", MathStyle::Text, |c| {
        c.radical_rule_thickness.into()
    }),
    ("radicalvgap", "Umathradicalvgap", MathStyle::Text, |c| {
        c.radical_vertical_gap.into()
    }),
    (
        "radicalvgapdisplay",
        "Umathradicalvgap",
        MathStyle::Display,
        |c| c.radical_display_style_vertical_gap.into(),
    ),
    ("overbarrule", "Umathoverbarrule", MathStyle::Text, |c| {
        c.overbar_rule_thickness.into()
    }),
    ("stackvgap", "Umathstackvgap", MathStyle::Text, |c| {
        c.stack_gap_min.into()
    }),
    (
        "stackvgapdisplay",
        "Umathstackvgap",
        MathStyle::Display,
        |c| c.stack_display_style_gap_min.into(),
    ),
    ("subsupvgap", "Umathsubsupvgap", MathStyle::Text, |c| {
        c.sub_superscript_gap_min.into()
    }),
    ("supbottommin", "Umathsupbottommin", MathStyle::Text, |c| {
        c.superscript_bottom_min.into()
    }),
    ("subtopmax", "Umathsubtopmax", MathStyle::Text, |c| {
        c.subscript_top_max.into()
    }),
    (
        "spaceafterscript",
        "Umathspaceafterscript",
        MathStyle::Text,
        |c| c.space_after_script.into(),
    ),
    (
        "limitabovevgap",
        "Umathlimitabovevgap",
        MathStyle::Text,
        |c| c.upper_limit_gap_min.into(),
    ),
    (
        "limitbelowvgap",
        "Umathlimitbelowvgap",
        MathStyle::Text,
        |c| c.lower_limit_gap_min.into(),
    ),
    (
        "fractionnumup",
        "Umathfractionnumup",
        MathStyle::Text,
        |c| c.fraction_numerator_shift_up.into(),
    ),
    (
        "fractionnumupdisplay",
        "Umathfractionnumup",
        MathStyle::Display,
        |c| c.fraction_numerator_display_style_shift_up.into(),
    ),
    (
        "fractiondenomdown",
        "Umathfractiondenomdown",
        MathStyle::Text,
        |c| c.fraction_denominator_shift_down.into(),
    ),
    (
        "fractiondenomdowndisplay",
        "Umathfractiondenomdown",
        MathStyle::Display,
        |c| c.fraction_denominator_display_style_shift_down.into(),
    ),
];

/// MATH constants of a face with its em size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MathFontParams {
    pub units_per_em: u16,
    pub constants: MathConstants,
}

impl MathFontParams {
    pub fn from_face(face: &TrueTypeFace) -> Option<MathFontParams> {
        Some(MathFontParams {
            units_per_em: face.units_per_em(),
            constants: face.math()?.constants,
        })
    }

    pub fn pt(&self, units: i32, size_pt: f64) -> f64 {
        f64::from(units) * size_pt / f64::from(self.units_per_em)
    }

    /// `(oracle key, value in pt)` for every mapped `\Umath` parameter.
    pub fn umath_pt(&self, size_pt: f64) -> Vec<(&'static str, f64)> {
        UMATH_PARAMETERS
            .iter()
            .map(|(k, _, _, f)| (*k, self.pt(f(&self.constants), size_pt)))
            .collect()
    }

    /// Script and scriptscript sizes (`ScriptPercentScaleDown`, ...).
    pub fn script_sizes(&self, size_pt: f64) -> (f64, f64) {
        (
            size_pt * f64::from(self.constants.script_percent_scale_down) / 100.0,
            size_pt * f64::from(self.constants.script_script_percent_scale_down) / 100.0,
        )
    }
}
