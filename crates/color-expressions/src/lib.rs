//! FlashTeX colour expressions: an original, bounded, xcolor-*style*
//! expression parser that resolves over explicitly named palettes into
//! `flashtex-vector-graphics`'s existing colour types.
//!
//! This is **not** a reimplementation of LaTeX's `xcolor` package and makes
//! no claim of matching its output. It borrows the familiar surface syntax
//! (`red!50!blue`, `-red`) and defines its own, simpler, fully-specified
//! semantics documented on [`expr::mix`] and [`expr::negate`] (kept private;
//! see the module docs and the tests in `src/expr.rs` and `src/parser.rs`
//! for the exact, hand-checkable arithmetic).
//!
//! # What this crate reuses vs. owns
//!
//! - **Reused, unmodified**: [`flashtex_vector_graphics::Color`] (the
//!   `Gray`/`Rgb`/`Cmyk` device-colour enum) and
//!   [`flashtex_vector_graphics::Paint`] (colour + straight alpha). This
//!   crate resolves expressions *into* those types; it does not define a
//!   parallel colour representation, and it does not touch
//!   `crates/vector-graphics` at all.
//! - **Reused, as-is**: `Color::to_rgb`'s already-documented naive
//!   conversion, used only when [`expr::mix`] must blend across two
//!   different `Color` variants. No new colour-space approximation is
//!   introduced here.
//! - **Owned by this crate**: the expression grammar ([`parser`]), the AST
//!   and evaluation ([`expr`]), the palette type ([`palette::Palette`]),
//!   and the error type ([`error::ColorExprError`]).
//!
//! # Grammar and bounds
//!
//! See the [`parser`] module for the full grammar. Two independent, typed
//! bounds apply to every call:
//!
//! - [`MAX_INPUT_LEN`] bytes, checked before any parsing begins.
//! - [`MAX_DEPTH`] "atoms" (each `-` prefix, each `(...)` nesting level, and
//!   each term of a `!`-chain spends one unit), checked before each
//!   recursive descent step. Exceeding either bound is
//!   [`ColorExprError::TooLong`] / [`ColorExprError::TooDeep`], never a
//!   stack overflow or an unbounded loop.
//!
//! Unknown palette names are always [`ColorExprError::UnknownColor`] —
//! never silently resolved to black or any other default.
//!
//! # Literal colours
//!
//! Alongside palette names, an atom may also be a `model:components`
//! literal — `gray:0.5`, `rgb:1,0,0`, `cmyk:0,0,0,1` — resolved directly to
//! a [`flashtex_vector_graphics::Color`] with no palette lookup. Only
//! `gray`, `rgb`, and `cmyk` are recognised: exactly the three variants
//! `Color` already has. An unrecognised model name (e.g. `hsb:...`, a real
//! xcolor model this crate's dependency has no representation for), the
//! wrong number of components for a model, or a component outside
//! `0.0..=1.0` is a typed [`ColorExprError`], never an invented colour
//! space and never a silent fallback.
//!
//! # Example
//!
//! ```
//! use flashtex_color_expressions::{base_palette, resolve};
//! use flashtex_vector_graphics::Color;
//!
//! let palette = base_palette();
//! // 50% red mixed with 50% blue, in RGB: (0.5, 0.0, 0.5).
//! assert_eq!(resolve("red!50!blue", &palette), Ok(Color::Rgb(0.5, 0.0, 0.5)));
//! // Unknown names are a typed error, never black.
//! assert!(resolve("chartreuse", &palette).is_err());
//! // Literal colours need no palette entry at all.
//! assert_eq!(resolve("cmyk:0,0,0,1", &palette), Ok(Color::Cmyk(0.0, 0.0, 0.0, 1.0)));
//! // Unsupported colour models are a typed error, never approximated.
//! assert!(resolve("hsb:0.5,1,1", &palette).is_err());
//! ```

mod error;
mod expr;
mod palette;
mod parser;

pub use error::ColorExprError;
pub use palette::{Palette, base_palette};

use flashtex_vector_graphics::{Color, Paint};

/// Maximum accepted input length, in bytes. Checked before any parsing
/// work, so a huge input is rejected in O(1).
pub const MAX_INPUT_LEN: usize = 512;

/// Maximum number of atoms (palette names, counting each `-` prefix and
/// each level of `(...)` nesting, and each term of a `!`-mix chain) a
/// single expression may contain. This is the bound that turns a hostile,
/// deeply nested or very long expression into a typed error instead of
/// unbounded recursion.
pub const MAX_DEPTH: usize = 32;

/// Parses `expression` and resolves it against `palette` into a
/// [`flashtex_vector_graphics::Color`].
///
/// Returns a typed [`ColorExprError`] for malformed input, input over
/// [`MAX_INPUT_LEN`] bytes, nesting/chaining over [`MAX_DEPTH`], or any
/// palette name not present in `palette` — the last never falls back to a
/// default colour.
pub fn resolve(expression: &str, palette: &Palette) -> Result<Color, ColorExprError> {
    let ast = parser::parse(expression)?;
    ast.eval(palette)
}

/// Convenience wrapper around [`resolve`] that packages the resolved
/// colour with an explicit straight alpha into a
/// [`flashtex_vector_graphics::Paint`].
pub fn resolve_paint(
    expression: &str,
    palette: &Palette,
    alpha: f64,
) -> Result<Paint, ColorExprError> {
    let color = resolve(expression, palette)?;
    Ok(Paint::new(color, alpha))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette() -> Palette {
        base_palette()
    }

    #[test]
    fn resolves_plain_name() {
        assert_eq!(resolve("red", &palette()), Ok(Color::Rgb(1.0, 0.0, 0.0)));
    }

    #[test]
    fn resolves_mix_by_hand() {
        // 50% red + 50% blue = (0.5, 0.0, 0.5).
        assert_eq!(
            resolve("red!50!blue", &palette()),
            Ok(Color::Rgb(0.5, 0.0, 0.5))
        );
    }

    #[test]
    fn resolves_mix_against_implicit_white() {
        // "red!50" mixes 50% red with 50% white: (1,0,0)*0.5 + (1,1,1)*0.5
        // = (1.0, 0.5, 0.5).
        assert_eq!(resolve("red!50", &palette()), Ok(Color::Rgb(1.0, 0.5, 0.5)));
    }

    #[test]
    fn resolves_negation_by_hand() {
        // -red complements each RGB channel: (0.0, 1.0, 1.0) = cyan-ish.
        assert_eq!(resolve("-red", &palette()), Ok(Color::Rgb(0.0, 1.0, 1.0)));
    }

    #[test]
    fn resolves_parenthesised_sub_expression() {
        // (red!50!blue)!50!green: first (0.5,0,0.5), then 50% of that with
        // 50% green: (0.25, 0.5, 0.25).
        assert_eq!(
            resolve("(red!50!blue)!50!green", &palette()),
            Ok(Color::Rgb(0.25, 0.5, 0.25))
        );
    }

    #[test]
    fn double_negation_round_trips() {
        assert_eq!(resolve("--red", &palette()), resolve("red", &palette()));
    }

    #[test]
    fn unknown_name_is_a_typed_error_never_black() {
        let err = resolve("nonexistent", &palette()).unwrap_err();
        assert_eq!(
            err,
            ColorExprError::UnknownColor {
                name: "nonexistent".to_string()
            }
        );
        assert_ne!(resolve("nonexistent", &palette()), Ok(Color::BLACK));
    }

    #[test]
    fn unknown_name_inside_mix_is_still_an_error() {
        assert!(resolve("red!50!nonexistent", &palette()).is_err());
    }

    #[test]
    fn malformed_expression_is_typed_error() {
        assert!(matches!(
            resolve("red!!blue", &palette()),
            Err(ColorExprError::UnexpectedChar { .. })
        ));
        assert!(matches!(
            resolve("", &palette()),
            Err(ColorExprError::Empty)
        ));
        assert!(matches!(
            resolve("red!101!blue", &palette()),
            Err(ColorExprError::InvalidPercentage { .. })
        ));
    }

    #[test]
    fn hostile_deeply_nested_expression_is_bounded_not_a_stack_overflow() {
        let hostile = format!("{}red{}", "(".repeat(10_000), ")".repeat(10_000));
        // Over MAX_INPUT_LEN, so this is rejected on length alone; the
        // dedicated depth tests in `parser` cover a within-length-limit
        // depth attack precisely.
        assert!(matches!(
            resolve(&hostile, &palette()),
            Err(ColorExprError::TooLong { .. })
        ));

        let hostile_within_len = format!(
            "{}red{}",
            "(".repeat(MAX_DEPTH + 10),
            ")".repeat(MAX_DEPTH + 10)
        );
        assert!(hostile_within_len.len() <= MAX_INPUT_LEN);
        assert!(matches!(
            resolve(&hostile_within_len, &palette()),
            Err(ColorExprError::TooDeep { .. })
        ));
    }

    #[test]
    fn unicode_identifier_round_trips_through_a_custom_palette() {
        // Dyadic fractions (eighths) so the 50/50 lerp is bit-exact, not
        // just approximately right.
        let mut p = Palette::new();
        p.insert("rouge", Color::Rgb(0.875, 0.25, 0.0));
        p.insert("café", Color::Rgb(0.125, 0.5, 0.25));
        // r: 0.5*0.875 + 0.5*0.125 = 0.5
        // g: 0.5*0.25  + 0.5*0.5   = 0.375
        // b: 0.5*0.0   + 0.5*0.25  = 0.125
        assert_eq!(
            resolve("rouge!50!café", &p),
            Ok(Color::Rgb(0.5, 0.375, 0.125))
        );
    }

    #[test]
    fn unicode_garbage_input_is_typed_error_not_panic() {
        assert!(resolve("🎨!50!red", &palette()).is_err());
        assert!(resolve("re🎨d", &palette()).is_err());
    }

    #[test]
    fn resolve_paint_carries_alpha() {
        let paint = resolve_paint("red", &palette(), 0.4).unwrap();
        assert_eq!(paint.color, Color::Rgb(1.0, 0.0, 0.0));
        assert_eq!(paint.alpha, 0.4);
    }

    // --- Literal `model:components` colours: expanded model coverage ---
    //
    // Expected values below are literal constants read straight off the
    // input text (a `gray:g`/`rgb:r,g,b`/`cmyk:c,m,y,k` literal *is* its
    // components, by definition of the grammar in `parser`) or worked out
    // by hand per the module-doc'd linear-interpolation mixing rule; none
    // are produced by calling this crate's own resolver.

    #[test]
    fn resolves_gray_literal() {
        assert_eq!(resolve("gray:0.5", &palette()), Ok(Color::Gray(0.5)));
    }

    #[test]
    fn resolves_rgb_literal() {
        assert_eq!(
            resolve("rgb:0.25,0.5,1", &palette()),
            Ok(Color::Rgb(0.25, 0.5, 1.0))
        );
    }

    #[test]
    fn resolves_cmyk_literal() {
        assert_eq!(
            resolve("cmyk:0,0.5,1,0.25", &palette()),
            Ok(Color::Cmyk(0.0, 0.5, 1.0, 0.25))
        );
    }

    #[test]
    fn literal_matching_a_named_palette_entry_mixes_identically() {
        // "blue" in the base palette is exactly Rgb(0,0,1), so mixing a
        // literal spelling of the same colour must give the same answer as
        // mixing the name.
        assert_eq!(
            resolve("red!50!rgb:0,0,1", &palette()),
            resolve("red!50!blue", &palette())
        );
    }

    #[test]
    fn mix_of_two_cmyk_literals_is_exact_channelwise_lerp() {
        // 60% of cmyk:1,0,0,0 + 40% of cmyk:0,0,1,0:
        //   c: 0.6*1 + 0.4*0 = 0.6
        //   m: 0
        //   y: 0.6*0 + 0.4*1 = 0.4
        //   k: 0
        assert_eq!(
            resolve("cmyk:1,0,0,0!60!cmyk:0,0,1,0", &palette()),
            Ok(Color::Cmyk(0.6, 0.0, 0.4, 0.0))
        );
    }

    #[test]
    fn unsupported_color_model_end_to_end_is_typed_error_never_black() {
        // hsb is a real xcolor model; flashtex-vector-graphics::Color has
        // no HSB representation, so this is a typed error end to end, not
        // an approximation into RGB and not black.
        let err = resolve("hsb:0.5,1,1", &palette()).unwrap_err();
        assert_eq!(
            err,
            ColorExprError::UnsupportedColorModel {
                pos: 0,
                name: "hsb".to_string()
            }
        );
        assert_ne!(resolve("hsb:0.5,1,1", &palette()), Ok(Color::BLACK));
    }

    #[test]
    fn component_count_and_range_errors_are_typed_end_to_end() {
        assert!(matches!(
            resolve("rgb:1,0", &palette()),
            Err(ColorExprError::InvalidComponentCount { .. })
        ));
        assert!(matches!(
            resolve("gray:1.5", &palette()),
            Err(ColorExprError::ComponentOutOfRange { .. })
        ));
    }
}
