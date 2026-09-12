//! The expression AST and its evaluation into
//! [`flashtex_vector_graphics::Color`].
//!
//! The AST is only ever built by [`crate::parser`], which enforces
//! [`crate::MAX_DEPTH`] while constructing it — so `eval`'s own recursion
//! (which walks the same shape) is bounded for free and needs no separate
//! depth check.

use crate::error::ColorExprError;
use crate::palette::Palette;
use flashtex_vector_graphics::Color;

/// A parsed colour expression.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expr {
    /// A bare palette name, e.g. `red`.
    Name(String),
    /// `-e`: the component-wise complement of `e` (see [`negate`]).
    Negate(Box<Expr>),
    /// `left!pct!right`: `pct`% of `left` mixed with `(100 - pct)`% of
    /// `right` (see [`mix`]). `right` is `None` for the one-argument form
    /// `left!pct`, which xcolor-style expressions mix against white.
    Mix {
        left: Box<Expr>,
        pct: u8,
        right: Option<Box<Expr>>,
    },
}

/// Component-wise complement in whatever colour model `c` is already in.
///
/// Worked example: `Rgb(1.0, 0.0, 0.25)` negates to `Rgb(0.0, 1.0, 0.75)`
/// (each channel replaced by `1 - channel`). This is our own definition,
/// not a claim of matching xcolor's `-color`.
pub(crate) fn negate(c: Color) -> Color {
    match c {
        Color::Gray(g) => Color::Gray(1.0 - g),
        Color::Rgb(r, g, b) => Color::Rgb(1.0 - r, 1.0 - g, 1.0 - b),
        Color::Cmyk(c, m, y, k) => Color::Cmyk(1.0 - c, 1.0 - m, 1.0 - y, 1.0 - k),
    }
}

/// `pct`% of `a` plus `(100 - pct)`% of `b`.
///
/// When `a` and `b` are the same variant, the mix is exact channel-wise
/// linear interpolation in that colour model. Worked example:
/// `mix(Gray(1.0), 25, Gray(0.0))` = `0.25 * 1.0 + 0.75 * 0.0` = `Gray(0.25)`.
///
/// When the variants differ, both sides are converted with the existing,
/// already-documented-as-naive [`Color::to_rgb`] before interpolating, and
/// the result is `Rgb`. No new approximation is introduced here beyond the
/// one `flashtex-vector-graphics` already documents and owns.
pub(crate) fn mix(a: Color, pct: u8, b: Color) -> Color {
    let t = f64::from(pct) / 100.0;
    let u = 1.0 - t;
    fn lerp(t: f64, u: f64, x: f64, y: f64) -> f64 {
        t * x + u * y
    }
    match (a, b) {
        (Color::Gray(ga), Color::Gray(gb)) => Color::Gray(lerp(t, u, ga, gb)),
        (Color::Rgb(ra, ga, ba), Color::Rgb(rb, gb, bb)) => {
            Color::Rgb(lerp(t, u, ra, rb), lerp(t, u, ga, gb), lerp(t, u, ba, bb))
        }
        (Color::Cmyk(ca, ma, ya, ka), Color::Cmyk(cb, mb, yb, kb)) => Color::Cmyk(
            lerp(t, u, ca, cb),
            lerp(t, u, ma, mb),
            lerp(t, u, ya, yb),
            lerp(t, u, ka, kb),
        ),
        (a, b) => {
            let (ra, ga, ba) = a.to_rgb();
            let (rb, gb, bb) = b.to_rgb();
            Color::Rgb(lerp(t, u, ra, rb), lerp(t, u, ga, gb), lerp(t, u, ba, bb))
        }
    }
}

impl Expr {
    /// Resolves the expression against a palette. Unknown names are always
    /// an error — never a default colour.
    pub(crate) fn eval(&self, palette: &Palette) -> Result<Color, ColorExprError> {
        match self {
            Expr::Name(name) => palette
                .get(name)
                .ok_or_else(|| ColorExprError::UnknownColor { name: name.clone() }),
            Expr::Negate(inner) => Ok(negate(inner.eval(palette)?)),
            Expr::Mix { left, pct, right } => {
                let l = left.eval(palette)?;
                let r = match right {
                    Some(r) => r.eval(palette)?,
                    None => Color::WHITE,
                };
                Ok(mix(l, *pct, r))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negate_gray() {
        assert_eq!(negate(Color::Gray(0.3)), Color::Gray(0.7));
    }

    #[test]
    fn negate_rgb() {
        assert_eq!(
            negate(Color::Rgb(1.0, 0.0, 0.25)),
            Color::Rgb(0.0, 1.0, 0.75)
        );
    }

    #[test]
    fn mix_same_variant_is_exact_lerp() {
        let got = mix(Color::Gray(1.0), 25, Color::Gray(0.0));
        assert_eq!(got, Color::Gray(0.25));
    }

    #[test]
    fn mix_rgb_by_hand() {
        // 40% red + 60% blue: (0.4, 0.0, 0.6).
        let got = mix(Color::Rgb(1.0, 0.0, 0.0), 40, Color::Rgb(0.0, 0.0, 1.0));
        assert_eq!(got, Color::Rgb(0.4, 0.0, 0.6));
    }

    #[test]
    fn mix_cross_variant_uses_to_rgb() {
        // Gray(0.2) as rgb is (0.2, 0.2, 0.2); mixed 50/50 with Rgb(1,0,0)
        // gives (0.6, 0.1, 0.1).
        let got = mix(Color::Gray(0.2), 50, Color::Rgb(1.0, 0.0, 0.0));
        assert_eq!(got, Color::Rgb(0.6, 0.1, 0.1));
    }

    #[test]
    fn mix_pct_100_is_left_pct_0_is_right() {
        let a = Color::Rgb(1.0, 0.2, 0.3);
        let b = Color::Rgb(0.0, 0.9, 0.1);
        assert_eq!(mix(a, 100, b), a);
        assert_eq!(mix(a, 0, b), b);
    }
}
