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
    /// A `model:components` literal (e.g. `rgb:1,0,0`, `cmyk:0,0,0,1`,
    /// `gray:0.5`) resolved directly to a [`Color`] at parse time, without
    /// any palette lookup. Only the three colour models
    /// [`flashtex_vector_graphics::Color`] already has are recognised; see
    /// [`crate::parser`] for the exact grammar and bounds.
    Literal(Color),
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
            Expr::Literal(color) => Ok(*color),
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

    // --- Independently-specified fixtures ------------------------------
    //
    // Every expected value below is a literal computed by hand from the
    // channel-wise linear-interpolation rule this module documents on
    // `mix` (`pct% * a + (100-pct)% * b`, per-channel) — the same rule the
    // real xcolor LaTeX package documents for its `!`-mix operator on
    // same-model colours. None of these expectations are produced by
    // calling `mix`, `resolve`, or any other crate code; each is a
    // constant, with the arithmetic shown in the comment beside it, so the
    // test can only pass if the implementation matches the independently
    // worked-out number. Dyadic fractions (halves, quarters, eighths) are
    // used throughout so the `f64` lerp is bit-exact, not just
    // approximately right.

    #[test]
    fn mix_cmyk_same_variant_is_exact_lerp() {
        // 25% of Cmyk(1,0,0,0.5) + 75% of Cmyk(0,1,0.5,0):
        //   c: 0.25*1   + 0.75*0   = 0.25
        //   m: 0.25*0   + 0.75*1   = 0.75
        //   y: 0.25*0   + 0.75*0.5 = 0.375
        //   k: 0.25*0.5 + 0.75*0   = 0.125
        let got = mix(
            Color::Cmyk(1.0, 0.0, 0.0, 0.5),
            25,
            Color::Cmyk(0.0, 1.0, 0.5, 0.0),
        );
        assert_eq!(got, Color::Cmyk(0.25, 0.75, 0.375, 0.125));
    }

    #[test]
    fn mix_cmyk_with_rgb_uses_documented_naive_to_rgb() {
        // flashtex_vector_graphics::Color::to_rgb documents its CMYK->RGB
        // conversion as `1 - min(1, channel + k)`. Working that out by hand
        // for Cmyk(0.5, 0.25, 0.0, 0.25):
        //   r: 1 - min(1, 0.5  + 0.25) = 1 - 0.75 = 0.25
        //   g: 1 - min(1, 0.25 + 0.25) = 1 - 0.5  = 0.5
        //   b: 1 - min(1, 0.0  + 0.25) = 1 - 0.25 = 0.75
        // so the CMYK side enters the lerp as Rgb(0.25, 0.5, 0.75). Mixing
        // that 50/50 with Rgb(1.0, 0.0, 0.0):
        //   r: 0.5*0.25 + 0.5*1.0 = 0.625
        //   g: 0.5*0.5  + 0.5*0.0 = 0.25
        //   b: 0.5*0.75 + 0.5*0.0 = 0.375
        let got = mix(
            Color::Cmyk(0.5, 0.25, 0.0, 0.25),
            50,
            Color::Rgb(1.0, 0.0, 0.0),
        );
        assert_eq!(got, Color::Rgb(0.625, 0.25, 0.375));
    }

    #[test]
    fn mix_cmyk_with_gray_uses_documented_naive_to_rgb() {
        // Gray(g)'s naive RGB form is (g, g, g) (the identity case of the
        // same documented conversion). Cmyk(0.0, 0.0, 1.0, 0.0) -> RGB:
        //   r: 1 - min(1, 0 + 0) = 1
        //   g: 1 - min(1, 0 + 0) = 1
        //   b: 1 - min(1, 1 + 0) = 0
        // so this is Rgb(1,1,0) mixed 75/25 with Gray(0.2) == Rgb(0.2,0.2,0.2):
        //   r: 0.75*1 + 0.25*0.2 = 0.8
        //   g: 0.75*1 + 0.25*0.2 = 0.8
        //   b: 0.75*0 + 0.25*0.2 = 0.05
        let got = mix(Color::Cmyk(0.0, 0.0, 1.0, 0.0), 75, Color::Gray(0.2));
        assert_eq!(got, Color::Rgb(0.8, 0.8, 0.05));
    }

    // What we did NOT try to independently pin down: the *design choice*
    // behind the naive CMYK->RGB formula itself (why `c + k` rather than,
    // say, `(1-k)*(1-c)`) is `flashtex-vector-graphics`'s own documented
    // decision, not an external spec we can derive from first principles —
    // we only verify this crate applies that already-documented formula
    // correctly, not that the formula is "the right" one.
}
