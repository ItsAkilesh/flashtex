//! Latin Modern Math parameters per style, derived from the embedded OpenType
//! `MATH` constants ([`crate::lm_math`]) through [`MathParams::from_opentype`],
//! and the comparison matrix against the Computer Modern TFM parameters.
//!
//! LuaTeX/XeTeX set the script and scriptscript sizes from
//! `ScriptPercentScaleDown` / `ScriptScriptPercentScaleDown` (70% and 50%
//! for Latin Modern Math: 10/7/5pt), the same sizes plain TeX uses, so the
//! two derivations can be compared style by style.

use crate::cm::CmMathMetrics;
use crate::lm_math::{
    CONSTANTS, DISPLAY_OPERATOR_MIN_HEIGHT, QUAD, SCRIPT_PERCENT_SCALE_DOWN,
    SCRIPT_SCRIPT_PERCENT_SCALE_DOWN, UNITS_PER_EM, VERTICAL_VARIANTS, X_HEIGHT,
};
use crate::metrics::{MathFontMetrics, MathParams, SizeClass};

/// The point size Latin Modern Math uses for a size class at `text_size`.
pub fn size_pt(size: SizeClass, text_size: f64) -> f64 {
    let pct = match size {
        SizeClass::Text => 100,
        SizeClass::Script => SCRIPT_PERCENT_SCALE_DOWN,
        SizeClass::ScriptScript => SCRIPT_SCRIPT_PERCENT_SCALE_DOWN,
    };
    text_size * f64::from(pct) / 100.0
}

/// TeX parameters from Latin Modern Math's `MathConstants` at a size class.
pub fn params(size: SizeClass, text_size: f64) -> MathParams {
    MathParams::from_opentype(&CONSTANTS, X_HEIGHT, QUAD, size_pt(size, text_size))
}

/// One row of the CM-vs-LM matrix: a parameter at a style.
#[derive(Debug, Clone, PartialEq)]
pub struct MatrixRow {
    pub size: SizeClass,
    pub name: &'static str,
    pub cm: f64,
    pub lm: f64,
}

impl MatrixRow {
    pub fn delta(&self) -> f64 {
        self.lm - self.cm
    }
}

/// The parameter names in TeXbook order with their accessors.
pub const PARAMETERS: &[(&str, fn(&MathParams) -> f64)] = &[
    ("x_height", |p| p.x_height),
    ("quad", |p| p.quad),
    ("num1", |p| p.num1),
    ("num2", |p| p.num2),
    ("num3", |p| p.num3),
    ("denom1", |p| p.denom1),
    ("denom2", |p| p.denom2),
    ("sup1", |p| p.sup1),
    ("sup2", |p| p.sup2),
    ("sup3", |p| p.sup3),
    ("sub1", |p| p.sub1),
    ("sub2", |p| p.sub2),
    ("sup_drop", |p| p.sup_drop),
    ("sub_drop", |p| p.sub_drop),
    ("delim1", |p| p.delim1),
    ("delim2", |p| p.delim2),
    ("axis_height", |p| p.axis_height),
    ("default_rule_thickness", |p| p.default_rule_thickness),
    ("big_op_spacing1", |p| p.big_op_spacing1),
    ("big_op_spacing2", |p| p.big_op_spacing2),
    ("big_op_spacing3", |p| p.big_op_spacing3),
    ("big_op_spacing4", |p| p.big_op_spacing4),
    ("big_op_spacing5", |p| p.big_op_spacing5),
];

/// CM (`cmsy10/7/5` + `cmex10`, plain/LaTeX 10pt) against Latin Modern Math
/// at 10pt, every parameter at every size class.
pub fn matrix() -> Vec<MatrixRow> {
    let cm = CmMathMetrics::latex_10pt();
    let mut rows = Vec::new();
    for size in [SizeClass::Text, SizeClass::Script, SizeClass::ScriptScript] {
        let c = cm.params(size);
        let l = params(size, 10.0);
        for (name, get) in PARAMETERS {
            rows.push(MatrixRow {
                size,
                name,
                cm: get(&c),
                lm: get(&l),
            });
        }
    }
    rows
}

/// Parameters whose OpenType counterpart is not the same quantity, so a
/// difference is expected rather than a fault (see `docs/params-matrix.md`).
pub fn known_divergence(name: &str) -> Option<&'static str> {
    match name {
        "num3" => Some("StackTopShiftUp is the \\atop numerator shift; CM num3 = 0.4431 quad"),
        "sup3" => Some("SuperscriptShiftUpCramped; CM sup3 is cmsy fontdimen 15"),
        "sup_drop" => Some("SuperscriptBaselineDropMax (250) vs cmsy σ18 (386): LM chose the OpenType default semantics"),
        "sub_drop" => Some("SubscriptBaselineDropMin (200) vs cmsy σ19 (50)"),
        "delim1" | "delim2" => Some("DelimitedSubFormulaMinHeight replaces both σ20/σ21 (TeX uses them only for \\over with delimiters)"),
        "big_op_spacing1" | "big_op_spacing2" => Some("Upper/LowerLimitGapMin are swapped relative to ξ9/ξ10 in LM's table (200/167 vs 111/167)"),
        "big_op_spacing3" | "big_op_spacing4" => Some("Upper/LowerLimitBaselineRiseMin/DropMin vs ξ11/ξ12"),
        "big_op_spacing5" => Some("no OpenType counterpart; 0 by construction vs ξ13 = 0.1 em"),
        _ => None,
    }
}

/// The display-size variant Latin Modern Math offers for a large operator:
/// LuaTeX's `make_op` walks the `MathVariants` vertical list and takes the
/// first variant whose advance height reaches `DisplayOperatorMinHeight`
/// (1300 units = 13pt at 10pt). Returns `(glyph id, advance height in pt)`
/// or `None` when the symbol has no variant list in the embedded table.
pub fn display_operator_variant(ch: char, text_size: f64) -> Option<(u16, f64)> {
    let v = VERTICAL_VARIANTS.iter().find(|v| v.ch == ch)?;
    let upem = f64::from(UNITS_PER_EM);
    let min = f64::from(DISPLAY_OPERATOR_MIN_HEIGHT);
    let pick = v
        .variants
        .iter()
        .find(|(_, adv)| f64::from(*adv) >= min)
        .or_else(|| v.variants.last())?;
    Some((pick.0, f64::from(pick.1) * text_size / upem))
}

/// Every vertical variant of a delimiter or radical, as `(glyph id, advance
/// height in pt)`, smallest first, plus whether an assembly exists.
pub fn vertical_variants(ch: char, text_size: f64) -> Option<(Vec<(u16, f64)>, bool)> {
    let v = VERTICAL_VARIANTS.iter().find(|v| v.ch == ch)?;
    let upem = f64::from(UNITS_PER_EM);
    Some((
        v.variants
            .iter()
            .map(|(g, adv)| (*g, f64::from(*adv) * text_size / upem))
            .collect(),
        !v.assembly.is_empty(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_follow_the_percent_scale_downs() {
        assert_eq!(size_pt(SizeClass::Text, 10.0), 10.0);
        assert_eq!(size_pt(SizeClass::Script, 10.0), 7.0);
        assert_eq!(size_pt(SizeClass::ScriptScript, 10.0), 5.0);
    }

    /// The parameters that map to the same quantity agree with Computer
    /// Modern to within pdfTeX's TFM rounding at every style (Latin Modern
    /// Math was built to reproduce CM's fontdimens); the rest are listed in
    /// `known_divergence` and their deltas are tabulated, not hidden.
    #[test]
    fn lm_math_reproduces_cm_where_the_quantities_coincide() {
        let mut unexplained = Vec::new();
        for row in matrix() {
            let d = row.delta().abs();
            match known_divergence(row.name) {
                Some(_) => continue,
                None => {
                    // 0.02pt covers TFM fixword rounding (≤ 1sp) plus the
                    // font's own rounding of CM's design values to whole
                    // units (0.001em = 0.01pt at 10pt, 0.005pt at 5pt).
                    if d > 0.02 {
                        unexplained.push(format!(
                            "{:?} {}: cm {:.5} lm {:.5}",
                            row.size, row.name, row.cm, row.lm
                        ));
                    }
                }
            }
        }
        assert!(unexplained.is_empty(), "{unexplained:#?}");
    }

    #[test]
    fn known_divergences_are_real_differences() {
        // Guard against the table going stale: each listed name differs at
        // text size by more than the tolerance the other rows meet.
        for row in matrix().into_iter().filter(|r| r.size == SizeClass::Text) {
            if known_divergence(row.name).is_some() {
                assert!(
                    row.delta().abs() > 0.02,
                    "{} no longer diverges ({:.5} vs {:.5})",
                    row.name,
                    row.cm,
                    row.lm
                );
            }
        }
    }

    #[test]
    fn display_operator_variants_match_cmex_next_larger() {
        let cm = CmMathMetrics::latex_10pt();
        // ∑ ∏ ∫ ∮ ⋃ ⋂ ⨁ ⨂ ⨀ ⋁ ⋀ ∐: cmex's display variant is the next larger
        // character; LM's is the first MathVariants entry ≥ 13pt. Both are
        // 14pt-class glyphs (cmex 0x58: 1.4 em); the advance heights agree
        // to within the font's unit rounding.
        for ch in [
            '\u{2211}', '\u{220F}', '\u{222B}', '\u{222E}', '\u{22C3}', '\u{22C2}', '\u{2A01}',
            '\u{2A02}', '\u{2A00}', '\u{22C1}', '\u{22C0}', '\u{2210}',
        ] {
            let big = cm.large_operator(ch, SizeClass::Text).expect("cmex variant");
            let (gid, adv) = display_operator_variant(ch, 10.0).expect("LM variant");
            let base = VERTICAL_VARIANTS.iter().find(|v| v.ch == ch).unwrap().base_gid;
            assert_ne!(gid, base, "{ch}: LM offers a larger variant");
            assert!(
                (adv - big.total_height()).abs() < 0.1,
                "{ch}: cmex {:.3} vs LM {:.3}",
                big.total_height(),
                adv
            );
        }
    }
}
