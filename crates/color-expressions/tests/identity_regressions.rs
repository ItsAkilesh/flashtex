//! Exact identity regressions (revision 3).
//!
//! Every expected [`Color`] value below is a literal constant, worked out
//! by hand from the two rules this crate documents and depends on:
//!
//! - its own channel-wise-lerp mixing rule (`pct% * left + (100-pct)% *
//!   right`, same-variant exact, cross-variant via `Color::to_rgb()` first
//!   — see the module docs on `expr::mix` in `src/expr.rs`), and
//! - `flashtex_vector_graphics::Color::to_rgb`'s already-documented naive
//!   conversion (`Gray(g) -> (g,g,g)`, `Cmyk(c,m,y,k) -> 1 - min(1, c+k)`
//!   per channel), read from `crates/vector-graphics/src/color.rs`, not
//!   from running any code.
//!
//! None of these expectations were produced by calling `resolve`, `mix`,
//! or any other function in this crate — each arithmetic step is shown in
//! the comment beside its `assert_eq!`, matching the rev-2/rev-3 fixture
//! discipline recorded in `coordination/daniel-color.md`. Dyadic weights
//! (25/50/75/100/0) are used throughout so every `f64` lerp is bit-exact,
//! not just approximately right. This file exists specifically so that any
//! future change to the mixing arithmetic or literal/palette resolution
//! path fails a test here, loudly and by exact value, rather than only
//! failing a looser `is_ok()`/shape assertion elsewhere.
//!
//! Named palette entries used throughout, from `base_palette()`
//! (`src/palette.rs`, read directly, not resolved): `red = Rgb(1,0,0)`,
//! `green = Rgb(0,1,0)`, `blue = Rgb(0,0,1)`, `yellow = Rgb(1,1,0)`,
//! `gray = Gray(0.5)`, `white = Gray(1.0)` (also
//! `flashtex_vector_graphics::Color::WHITE`, used implicitly by the
//! one-argument `!pct` mix form).

use flashtex_color_expressions::{base_palette, resolve};
use flashtex_vector_graphics::Color;

#[test]
fn identity_plain_name_is_the_palette_entry_unchanged() {
    assert_eq!(resolve("red", &base_palette()), Ok(Color::Rgb(1.0, 0.0, 0.0)));
}

#[test]
fn identity_negation_of_blue_is_yellow_numerically() {
    // -blue: (1-0, 1-0, 1-1) = (1, 1, 0) -- numerically identical to
    // `yellow`, by the component-wise complement definition, not a lookup.
    assert_eq!(resolve("-blue", &base_palette()), Ok(Color::Rgb(1.0, 1.0, 0.0)));
}

#[test]
fn identity_double_negation_is_the_original_exact_value() {
    // -(-red): negate(Rgb(1,0,0)) = Rgb(0,1,1); negate that again =
    // Rgb(1,0,0). Pinned as a literal constant, not compared against a
    // second `resolve("red", ...)` call.
    assert_eq!(resolve("--red", &base_palette()), Ok(Color::Rgb(1.0, 0.0, 0.0)));
}

#[test]
fn identity_mix_25_75_same_variant() {
    // 0.25*red + 0.75*blue = (0.25*1+0.75*0, 0, 0.25*0+0.75*1) = (0.25, 0, 0.75)
    assert_eq!(
        resolve("red!25!blue", &base_palette()),
        Ok(Color::Rgb(0.25, 0.0, 0.75))
    );
}

#[test]
fn identity_mix_75_25_same_variant() {
    // 0.75*red + 0.25*blue = (0.75, 0, 0.25)
    assert_eq!(
        resolve("red!75!blue", &base_palette()),
        Ok(Color::Rgb(0.75, 0.0, 0.25))
    );
}

#[test]
fn identity_mix_pct_100_is_exactly_left_unchanged() {
    // t=1.0, u=0.0: 1.0*x + 0.0*y == x exactly in IEEE-754 for these
    // operands (no rounding), so this must be bit-exact, not just close.
    assert_eq!(
        resolve("red!100!blue", &base_palette()),
        Ok(Color::Rgb(1.0, 0.0, 0.0))
    );
}

#[test]
fn identity_mix_pct_0_is_exactly_right_unchanged() {
    assert_eq!(
        resolve("red!0!blue", &base_palette()),
        Ok(Color::Rgb(0.0, 0.0, 1.0))
    );
}

#[test]
fn identity_mix_against_implicit_white() {
    // "yellow!50" mixes 50% yellow with 50% white. Rgb vs Gray is
    // cross-variant, so both convert with to_rgb() first:
    // yellow.to_rgb() = (1,1,0), white.to_rgb() (Gray(1.0)) = (1,1,1).
    //   r: 0.5*1 + 0.5*1 = 1
    //   g: 0.5*1 + 0.5*1 = 1
    //   b: 0.5*0 + 0.5*1 = 0.5
    assert_eq!(
        resolve("yellow!50", &base_palette()),
        Ok(Color::Rgb(1.0, 1.0, 0.5))
    );
}

#[test]
fn identity_mix_same_variant_gray() {
    // gray = Gray(0.5), white = Gray(1.0), same-variant exact lerp:
    // 0.5*0.5 + 0.5*1.0 = 0.75
    assert_eq!(
        resolve("gray!50!white", &base_palette()),
        Ok(Color::Gray(0.75))
    );
}

#[test]
fn identity_nested_parenthesised_mix() {
    // Inner: red!50!blue = (0.5, 0, 0.5).
    // Outer: 0.5*(0.5,0,0.5) + 0.5*green(0,1,0) = (0.25, 0.5, 0.25)
    assert_eq!(
        resolve("(red!50!blue)!50!green", &base_palette()),
        Ok(Color::Rgb(0.25, 0.5, 0.25))
    );
}

#[test]
fn identity_negation_of_a_parenthesised_mix() {
    // Inner: red!50!blue = (0.5, 0, 0.5). Negate each channel:
    // (1-0.5, 1-0, 1-0.5) = (0.5, 1, 0.5)
    assert_eq!(
        resolve("-(red!50!blue)", &base_palette()),
        Ok(Color::Rgb(0.5, 1.0, 0.5))
    );
}

#[test]
fn identity_three_term_left_associative_mix_chain() {
    // "red!50!blue!50!green!50!yellow" is left-associative:
    //   ((red!50!blue)!50!green)!50!yellow
    // step 1: red!50!blue           = (0.5, 0, 0.5)
    // step 2: step1!50!green(0,1,0) = 0.5*(0.5,0,0.5) + 0.5*(0,1,0)
    //                               = (0.25, 0.5, 0.25)
    // step 3: step2!50!yellow(1,1,0)= 0.5*(0.25,0.5,0.25) + 0.5*(1,1,0)
    //                               = (0.625, 0.75, 0.125)
    assert_eq!(
        resolve("red!50!blue!50!green!50!yellow", &base_palette()),
        Ok(Color::Rgb(0.625, 0.75, 0.125))
    );
}

#[test]
fn identity_negated_gray_literal() {
    assert_eq!(
        resolve("-gray:0.3", &base_palette()),
        Ok(Color::Gray(0.7))
    );
}

#[test]
fn identity_cmyk_literal_negated_by_channel() {
    // -cmyk:0.25,0.5,0.75,0.125 = (1-0.25, 1-0.5, 1-0.75, 1-0.125)
    //                           = (0.75, 0.5, 0.25, 0.875)
    // Dyadic fractions throughout so every subtraction is bit-exact in
    // f64 (unlike e.g. 1.0 - 0.8, which is not exactly 0.2).
    assert_eq!(
        resolve("-cmyk:0.25,0.5,0.75,0.125", &base_palette()),
        Ok(Color::Cmyk(0.75, 0.5, 0.25, 0.875))
    );
}

#[test]
fn identity_two_rgb_literals_mixed_match_the_equivalent_named_mix() {
    // rgb:1,0,0 and rgb:0,0,1 are exactly red and blue spelled out in
    // full; the same channel-wise-lerp arithmetic as the named-palette
    // case, pinned separately to prove the literal leaf path produces the
    // identical exact value, not merely "close" or "consistent" (that
    // weaker check is `literal_matching_a_named_palette_entry_mixes_identically`
    // in src/lib.rs; this one asserts the literal constant directly).
    assert_eq!(
        resolve("rgb:1,0,0!50!rgb:0,0,1", &base_palette()),
        Ok(Color::Rgb(0.5, 0.0, 0.5))
    );
}

#[test]
fn identity_cmyk_literal_mixed_with_rgb_literal_uses_documented_naive_to_rgb() {
    // cmyk:0,0,0,0.5 -> to_rgb(): 1 - min(1, channel + k) per channel, all
    // channels 0 here, k=0.5: r=g=b = 1 - min(1, 0 + 0.5) = 0.5, so
    // (0.5, 0.5, 0.5). Mixed 50/50 with rgb:1,0,0:
    //   r: 0.5*0.5 + 0.5*1 = 0.75
    //   g: 0.5*0.5 + 0.5*0 = 0.25
    //   b: 0.5*0.5 + 0.5*0 = 0.25
    assert_eq!(
        resolve("cmyk:0,0,0,0.5!50!rgb:1,0,0", &base_palette()),
        Ok(Color::Rgb(0.75, 0.25, 0.25))
    );
}
