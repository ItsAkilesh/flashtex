//! Minimal static representation of TeX font metric (TFM) data.
//!
//! Only what the layout rules consume is kept: per-character box dimensions,
//! italic correction, the kern against the font's skew character (used for
//! accent placement), the `next_larger` chain used to pick delimiter and big
//! operator sizes, and the extensible recipe (recorded but not built yet).
//!
//! Dimensions are stored as raw TFM fixwords and scaled with [`scale`], a
//! transcription of TeX's integer algorithm (tex.web §571–572), so the
//! resulting dimensions agree with TeX's own boxes to the scaled point.

use crate::Error;

/// One scaled point: TeX's internal unit, 1/65536 pt.
pub const SP_PER_PT: f64 = 65536.0;

/// One character of a TFM font; dimensions are fixwords (value / 2^20 em).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TfmChar {
    pub code: u8,
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub italic: i32,
    /// Kern between this character and the font's skew character, or 0.
    pub skew_kern: i32,
    /// Next larger character in the same font, or `u8::MAX`.
    pub next_larger: u8,
    /// Extensible recipe `[top, mid, bot, rep]`, each `u8::MAX` when absent.
    pub extensible: [u8; 4],
}

/// Compact constructor used by the generated table.
#[allow(clippy::too_many_arguments)]
pub const fn c(
    code: u8,
    width: i32,
    height: i32,
    depth: i32,
    italic: i32,
    skew_kern: i32,
    next_larger: u8,
    extensible: [u8; 4],
) -> TfmChar {
    TfmChar {
        code,
        width,
        height,
        depth,
        italic,
        skew_kern,
        next_larger,
        extensible,
    }
}

/// A TFM font: fontdimen parameters and its character table.
#[derive(Debug)]
pub struct TfmFont {
    pub name: &'static str,
    pub design_size: f64,
    pub skew_char: u8,
    /// `params[i]` is fontdimen `i + 1` as a fixword (fontdimen 1, the slant,
    /// is a pure number that TeX never scales).
    pub params: &'static [i32],
    pub chars: &'static [TfmChar],
}

/// Scales a fixword to points at font size `at_pt`, exactly as TeX does
/// (`store_scaled`, tex.web §571–572): `z` is the size in scaled points and
/// the fixword bytes are combined with truncating integer arithmetic.
///
/// # Errors
///
/// Returns [`Error::PointSizeTooLarge`] when `at_pt` is large enough that
/// the loop below drives `alpha` past 256: `beta = 256 / alpha` would then
/// truncate to 0 and the following division would divide by zero. TeX
/// itself never calls `store_scaled` with a `z` this large (`\maxdimen`
/// bounds it well below this threshold), so this is rejected outright
/// rather than clamped to some in-range value, which would silently produce
/// wrong glyph geometry.
pub fn scale(fixword: i32, at_pt: f64) -> Result<f64, Error> {
    let mut z = (at_pt * SP_PER_PT).round() as i64;
    let mut alpha: i64 = 16;
    while z >= 0o40000000 {
        z /= 2;
        alpha *= 2;
    }
    if alpha > 256 {
        return Err(Error::PointSizeTooLarge { at_pt });
    }
    let beta = 256 / alpha;
    let alpha = alpha * z;
    let u = fixword as u32;
    let a = (u >> 24) as i64;
    let b = ((u >> 16) & 255) as i64;
    let c = ((u >> 8) & 255) as i64;
    let d = (u & 255) as i64;
    let sw = (((d * z) / 256 + c * z) / 256 + b * z) / beta;
    let sp = if a == 0 {
        sw
    } else if a == 255 {
        sw - alpha
    } else {
        // Out-of-range fixword; TeX would reject the font. Fall back to the
        // real-valued scaling so a bad table cannot panic the layout.
        return Ok(fixword as f64 / (1 << 20) as f64 * at_pt);
    };
    Ok(sp as f64 / SP_PER_PT)
}

impl TfmFont {
    pub fn char(&self, code: u8) -> Option<&TfmChar> {
        self.chars.iter().find(|c| c.code == code)
    }

    /// fontdimen `n` (1-based) scaled to `at_pt`, 0 when absent.
    ///
    /// # Errors
    ///
    /// Propagates [`scale`]'s error when `at_pt` is too large to scale.
    pub fn fontdimen(&self, n: usize, at_pt: f64) -> Result<f64, Error> {
        match self.params.get(n.wrapping_sub(1)) {
            Some(&p) => scale(p, at_pt),
            None => Ok(0.0),
        }
    }

    pub fn next_larger(&self, ch: &TfmChar) -> Option<&TfmChar> {
        if ch.next_larger == u8::MAX {
            None
        } else {
            self.char(ch.next_larger)
        }
    }

    pub fn is_extensible(&self, ch: &TfmChar) -> bool {
        ch.extensible[3] != u8::MAX
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaling_matches_tex_truncation() {
        // cmex10 fontdimen 8 (0.039999 em) at 10pt is 0.39998pt in TeX's
        // \showbox output, not 0.39999: the low bits truncate.
        let theta = scale(41942, 10.0).unwrap();
        assert!((theta - 26213.0 / SP_PER_PT).abs() < 1e-12);
        assert_eq!(format!("{:.5}", theta), "0.39998");
        // cmsy10 quad (1.000003 em) at 10pt is 655361sp.
        assert_eq!(scale(1048579, 10.0).unwrap() * SP_PER_PT, 655361.0);
        // Negative fixwords (the depth of cmr10 '=') keep their sign.
        assert!(scale(-139594, 10.0).unwrap() < 0.0);
    }

    /// Minimized repro: at a large enough `at_pt`, the `while z >=
    /// 0o40000000 { .. alpha *= 2 }` loop above runs enough times that
    /// `alpha` exceeds 256, so `beta = 256 / alpha` would truncate to 0 and
    /// the following `.. / beta` would divide by zero. Before the fix this
    /// panicked with "attempt to divide by zero" at src/tfm.rs:84 (in both
    /// debug and release: integer division by zero always panics,
    /// regardless of profile); it must now be a typed error instead.
    #[test]
    fn scale_rejects_a_point_size_too_large_to_fix_point_scale() {
        let err = scale(1 << 20, 100_000.0).unwrap_err();
        assert_eq!(err, Error::PointSizeTooLarge { at_pt: 100_000.0 });
    }
}
