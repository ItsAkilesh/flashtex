//! Exact TeX scaled points.
//!
//! TeX represents every dimension internally as an integer number of scaled
//! points (`sp`), where `1pt = 65536sp` (a 16-bit binary fraction). This
//! module is deliberately `f64`-free: a decimal literal such as `10.5cm` is
//! parsed as an exact `(numerator, denominator)` pair and converted with
//! `i128` rational arithmetic, so the result is bit-for-bit reproducible and
//! independent of any particular float rounding mode.
//!
//! Rounding matches real TeX's well-known behavior of truncating the exact
//! conversion toward zero rather than rounding to nearest — this is why
//! `\dimen0=1in \showthe\dimen0` famously prints `72.26999pt`, not
//! `72.27pt`. See the module tests for that and other hand-checked values.

use std::fmt;

/// Scaled points per TeX point (`1pt = 65536sp`), i.e. `2^16`.
pub const SP_PER_PT: i64 = 65536;

/// TeX's maximum internal dimension magnitude: `2^30 - 1` scaled points,
/// i.e. `16383.99998pt` (tex.web `max_dimen`). Dimensions are bounded to
/// `[-MAX_DIMEN_SP, MAX_DIMEN_SP]`.
pub const MAX_DIMEN_SP: i64 = (1i64 << 30) - 1;

/// A unit that a dimension literal may be written in.
///
/// Only units TeX itself resolves to an exact multiple of a point are
/// supported. Font-relative units (`em`, `ex`) and anything else are
/// rejected explicitly by the parser rather than approximated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Unit {
    Pt,
    In,
    Pc,
    Cm,
    Mm,
    Bp,
    Sp,
}

impl Unit {
    /// Points-per-unit as an exact `(numerator, denominator)` pair, matching
    /// TeX's own internal conversion table (tex.web `scan_dimen`). `Sp` is
    /// handled separately since it is the base unit, not pt-relative.
    fn pt_ratio(self) -> (i128, i128) {
        match self {
            Unit::Pt => (1, 1),
            Unit::In => (7227, 100),
            Unit::Pc => (12, 1),
            Unit::Cm => (7227, 254),
            Unit::Mm => (7227, 2540),
            Unit::Bp => (7227, 7200),
            Unit::Sp => (0, 0), // unused; see `Unit::to_sp`
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Unit::Pt => "pt",
            Unit::In => "in",
            Unit::Pc => "pc",
            Unit::Cm => "cm",
            Unit::Mm => "mm",
            Unit::Bp => "bp",
            Unit::Sp => "sp",
        }
    }

    pub fn parse(s: &str) -> Option<Unit> {
        Some(match s {
            "pt" => Unit::Pt,
            "in" => Unit::In,
            "pc" => Unit::Pc,
            "cm" => Unit::Cm,
            "mm" => Unit::Mm,
            "bp" => Unit::Bp,
            "sp" => Unit::Sp,
            _ => return None,
        })
    }

    /// Convert an exact decimal value `numerator / denominator` (`denominator
    /// > 0`) in this unit to scaled points, truncating toward zero exactly as
    /// TeX does. Returns a typed overflow error if the magnitude exceeds
    /// [`MAX_DIMEN_SP`].
    pub fn to_sp(self, numerator: i128, denominator: i128) -> Result<Sp, CalcError> {
        debug_assert!(denominator > 0);
        let (mag_num, sign): (i128, i128) = if numerator < 0 {
            (-numerator, -1)
        } else {
            (numerator, 1)
        };
        let magnitude: i128 = match self {
            Unit::Sp => mag_num / denominator,
            other => {
                let (un, ud) = other.pt_ratio();
                // exact rational: mag_num/denominator * un/ud * SP_PER_PT,
                // truncated toward zero (floor of a non-negative value).
                (mag_num * un * SP_PER_PT as i128) / (denominator * ud)
            }
        };
        let signed = magnitude * sign;
        Sp::from_i128(signed)
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// An exact TeX dimension: an integer number of scaled points, bounded to
/// `[-MAX_DIMEN_SP, MAX_DIMEN_SP]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sp(pub i64);

impl Sp {
    pub const ZERO: Sp = Sp(0);
    pub const MAX: Sp = Sp(MAX_DIMEN_SP);
    pub const MIN: Sp = Sp(-MAX_DIMEN_SP);

    fn from_i128(v: i128) -> Result<Sp, CalcError> {
        if v.abs() > MAX_DIMEN_SP as i128 {
            Err(CalcError::Overflow)
        } else {
            Ok(Sp(v as i64))
        }
    }

    pub fn checked_add(self, other: Sp) -> Result<Sp, CalcError> {
        Sp::from_i128(self.0 as i128 + other.0 as i128)
    }

    pub fn checked_sub(self, other: Sp) -> Result<Sp, CalcError> {
        Sp::from_i128(self.0 as i128 - other.0 as i128)
    }

    pub fn checked_neg(self) -> Result<Sp, CalcError> {
        Sp::from_i128(-(self.0 as i128))
    }

    /// Multiply by an exact dimensionless scalar `numerator/denominator`.
    pub fn checked_mul_scalar(self, numerator: i128, denominator: i128) -> Result<Sp, CalcError> {
        debug_assert!(denominator > 0);
        let scaled = (self.0 as i128 * numerator) / denominator;
        Sp::from_i128(scaled)
    }

    /// Divide by an exact dimensionless scalar `numerator/denominator`,
    /// truncating toward zero. A zero scalar is a typed error, never a panic.
    pub fn checked_div_scalar(self, numerator: i128, denominator: i128) -> Result<Sp, CalcError> {
        debug_assert!(denominator > 0);
        if numerator == 0 {
            return Err(CalcError::DivisionByZero);
        }
        let scaled = (self.0 as i128 * denominator) / numerator;
        Sp::from_i128(scaled)
    }

    /// Convert to the interchange type consumed from `document-style`:
    /// [`flashtex_document_style::length::Pt`] (an `f64` TeX point, 72.27 per
    /// inch — the same base unit as `Sp`, without the exactness guarantee).
    /// This conversion is inherently lossy in the direction `Sp -> Pt`.
    pub fn to_style_pt(self) -> flashtex_document_style::length::Pt {
        flashtex_document_style::length::Pt(self.0 as f64 / SP_PER_PT as f64)
    }

    /// Convert from `document-style`'s `f64` TeX point into an exact `Sp`,
    /// truncating toward zero. Returns a typed overflow error if the value
    /// (or a non-finite `f64`) is out of range.
    pub fn try_from_style_pt(pt: flashtex_document_style::length::Pt) -> Result<Sp, CalcError> {
        let v = pt.0;
        if !v.is_finite() {
            return Err(CalcError::Overflow);
        }
        let sp = (v * SP_PER_PT as f64).trunc();
        if sp.abs() > MAX_DIMEN_SP as f64 {
            return Err(CalcError::Overflow);
        }
        Ok(Sp(sp as i64))
    }
}

impl fmt::Display for Sp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}sp", self.0)
    }
}

/// Typed evaluator errors. Nothing in this crate panics on malformed,
/// overflowing, or unsupported input — every failure mode is one of these.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CalcError {
    /// A dimension arithmetic result (or a literal's own magnitude) exceeds
    /// `MAX_DIMEN_SP` in absolute value.
    Overflow,
    /// A `/` operation's divisor evaluated to exactly zero.
    DivisionByZero,
    /// A unit token was not one of the exact units this crate supports.
    UnsupportedUnit(String),
    /// A named length was referenced but is not bound in any enclosing scope.
    UndefinedLength(String),
    /// A named length's definition refers back to itself, directly or
    /// through a chain of other named lengths.
    CyclicLength(String),
    /// Resolving a named length required more nesting than
    /// [`crate::eval::MAX_RESOLUTION_DEPTH`] allows.
    ResolutionDepthExceeded,
    /// A syntax error while lexing or parsing, with a human-readable
    /// description and the byte offset it was found at.
    Parse { message: String, at: usize },
    /// A construct that parsed but combines dimensions and dimensionless
    /// scalars (or two dimensions) in a way this evaluator does not support,
    /// e.g. adding a bare number to a dimension, multiplying two dimensions,
    /// or a top-level result that is a scalar rather than a dimension.
    TypeMismatch(String),
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalcError::Overflow => write!(f, "dimension exceeds TeX's maximum (16383.99998pt)"),
            CalcError::DivisionByZero => write!(f, "division by zero"),
            CalcError::UnsupportedUnit(u) => write!(f, "unsupported unit `{u}`"),
            CalcError::UndefinedLength(n) => write!(f, "undefined length `{n}`"),
            CalcError::CyclicLength(n) => write!(f, "cyclic definition of length `{n}`"),
            CalcError::ResolutionDepthExceeded => write!(f, "length resolution nested too deeply"),
            CalcError::Parse { message, at } => write!(f, "parse error at byte {at}: {message}"),
            CalcError::TypeMismatch(m) => write!(f, "type mismatch: {m}"),
        }
    }
}

impl std::error::Error for CalcError {}

#[cfg(test)]
mod tests {
    use super::*;

    // Hand-computed (see fractions.Fraction cross-check in the assignment
    // notes): 1pt/1in/1pc/1cm/1mm/1bp exact -> sp, matching TeX's own
    // truncating conversion table.

    #[test]
    fn pt_is_exact() {
        assert_eq!(Unit::Pt.to_sp(1, 1).unwrap(), Sp(65536));
    }

    #[test]
    fn inch_matches_tex_72_26999pt_quirk() {
        // 1in = 7227/100 pt exactly = 4736286.72sp -> truncates to 4736286,
        // which is the real, famous pdfTeX `\showthe` output of 72.26999pt
        // for `\dimen0=1in`, not 72.27pt.
        let sp = Unit::In.to_sp(1, 1).unwrap();
        assert_eq!(sp, Sp(4_736_286));
    }

    #[test]
    fn pica_is_twelve_points_exact() {
        assert_eq!(Unit::Pc.to_sp(1, 1).unwrap(), Sp(12 * 65536));
    }

    #[test]
    fn cm_matches_hand_computed_sp() {
        // 1cm = 7227/254 pt exactly = 1864679.811...sp -> truncates to 1864679.
        assert_eq!(Unit::Cm.to_sp(1, 1).unwrap(), Sp(1_864_679));
    }

    #[test]
    fn mm_matches_hand_computed_sp() {
        // 1mm = 7227/2540 pt exactly = 186467.981...sp -> truncates to 186467.
        assert_eq!(Unit::Mm.to_sp(1, 1).unwrap(), Sp(186_467));
    }

    #[test]
    fn bp_matches_hand_computed_sp() {
        // 1bp = 7227/7200 pt exactly = 65781.76sp -> truncates to 65781.
        assert_eq!(Unit::Bp.to_sp(1, 1).unwrap(), Sp(65_781));
    }

    #[test]
    fn sp_unit_is_identity() {
        assert_eq!(Unit::Sp.to_sp(1_073_741_823, 1).unwrap(), Sp::MAX);
    }

    #[test]
    fn fractional_cm_hand_computed() {
        // 10.5cm = 105/10 * 7227/254 pt = 2486550528/127 sp = 19579138.0157...
        // -> truncates to 19579138.
        assert_eq!(Unit::Cm.to_sp(105, 10).unwrap(), Sp(19_579_138));
    }

    #[test]
    fn negative_dimension_truncates_toward_zero() {
        // -1in truncates the same magnitude as +1in, sign applied after.
        assert_eq!(Unit::In.to_sp(-1, 1).unwrap(), Sp(-4_736_286));
    }

    #[test]
    fn boundary_exactly_at_max_dimen_via_pt() {
        // Hand-computed: 16383.99999pt = 1638399999/100000 pt exactly ->
        // 1073741823.34464sp -> truncates to exactly MAX_DIMEN_SP.
        assert_eq!(Unit::Pt.to_sp(1_638_399_999, 100_000).unwrap(), Sp::MAX);
    }

    #[test]
    fn boundary_one_sp_over_via_sp_unit_overflows() {
        assert_eq!(
            Unit::Sp.to_sp(MAX_DIMEN_SP as i128 + 1, 1),
            Err(CalcError::Overflow)
        );
    }

    #[test]
    fn boundary_16384pt_overflows() {
        assert_eq!(Unit::Pt.to_sp(16_384, 1), Err(CalcError::Overflow));
    }

    #[test]
    fn boundary_negative_max_dimen_is_ok() {
        assert_eq!(Unit::Sp.to_sp(-(MAX_DIMEN_SP as i128), 1).unwrap(), Sp::MIN);
    }

    #[test]
    fn boundary_negative_one_over_overflows() {
        assert_eq!(
            Unit::Sp.to_sp(-(MAX_DIMEN_SP as i128) - 1, 1),
            Err(CalcError::Overflow)
        );
    }

    #[test]
    fn add_overflow_is_typed_not_panic() {
        let a = Sp::MAX;
        let b = Sp(1);
        assert_eq!(a.checked_add(b), Err(CalcError::Overflow));
    }

    #[test]
    fn sub_underflow_is_typed_not_panic() {
        let a = Sp::MIN;
        let b = Sp(1);
        assert_eq!(a.checked_sub(b), Err(CalcError::Overflow));
    }

    #[test]
    fn mul_overflow_is_typed_not_panic() {
        assert_eq!(Sp::MAX.checked_mul_scalar(2, 1), Err(CalcError::Overflow));
    }

    #[test]
    fn div_by_zero_is_typed_not_panic() {
        assert_eq!(
            Sp(65536).checked_div_scalar(0, 1),
            Err(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn div_truncates_toward_zero() {
        // 65536sp (1pt) / 3 = 21845.33... -> truncates to 21845.
        assert_eq!(Sp(65536).checked_div_scalar(3, 1).unwrap(), Sp(21845));
    }

    #[test]
    fn add_within_bounds_is_exact() {
        assert_eq!(Sp(65536).checked_add(Sp(65536)).unwrap(), Sp(131072));
    }

    #[test]
    fn round_trip_to_style_pt_and_back() {
        let sp = Sp(65536 * 3);
        let pt = sp.to_style_pt();
        assert_eq!(pt.0, 3.0);
        assert_eq!(Sp::try_from_style_pt(pt).unwrap(), sp);
    }

    #[test]
    fn from_style_pt_overflow_is_typed() {
        let huge = flashtex_document_style::length::Pt(100_000.0);
        assert_eq!(Sp::try_from_style_pt(huge), Err(CalcError::Overflow));
    }

    #[test]
    fn from_style_pt_non_finite_is_typed_overflow() {
        let nan = flashtex_document_style::length::Pt(f64::NAN);
        assert_eq!(Sp::try_from_style_pt(nan), Err(CalcError::Overflow));
    }
}
