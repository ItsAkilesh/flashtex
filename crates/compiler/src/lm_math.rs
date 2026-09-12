//! Glyphs drawn from the pinned Latin Modern Math resource, not the base-14 fonts.
//!
//! Blackboard bold, `\setminus` and the long `\Longrightarrow` arrow have no
//! glyph in the base-14 Symbol face. Rather than substitute a look-alike, the
//! compiler emits their real Unicode code points and binds them to the
//! `lm.math` resource the font-engine manifest already pins (the same file the
//! Mac app bundles as `apps/mac/Fonts/latinmodern-math.otf`). Advance widths
//! come from that exact font program; `tests/lm_math_binding.rs` re-reads the
//! file and fails if the digest or any advance drifts.
//!
//! Fidelity limitation: this is the `unicode-math` design of Latin Modern
//! Math. pdfLaTeX draws `\mathbb` from `msbm10` and `\setminus` from `cmsy10`,
//! whose widths differ (for example R is 0.83pt narrower here at 10pt), so
//! output using these glyphs does not have pixel parity with pdfLaTeX.

/// Font-engine manifest id of the resource.
pub const FONT_ID: &str = "lm.math";
/// SHA-256 of the exact font program the advances below were read from.
pub const SHA256: &str = "6075562b771f8b82f0c179e363389684f2dd09de30038269e2628e504bd7be0f";
/// `font-hints-v1` family for items drawn from this resource.
pub const FAMILY: &str = "Latin Modern Math";
pub const UNITS_PER_EM: f64 = 1000.0;

/// Every glyph the compiler emits from this resource, with its advance width
/// in font units.
pub const ADVANCES: &[(char, u16)] = &[
    ('\u{1D538}', 611), // \mathbb{A}
    ('\u{1D539}', 639), // \mathbb{B}
    ('\u{2102}', 667),  // \mathbb{C}
    ('\u{1D53B}', 694), // \mathbb{D}
    ('\u{1D53C}', 611), // \mathbb{E}
    ('\u{1D53D}', 611), // \mathbb{F}
    ('\u{1D53E}', 667), // \mathbb{G}
    ('\u{210D}', 722),  // \mathbb{H}
    ('\u{1D540}', 334), // \mathbb{I}
    ('\u{1D541}', 639), // \mathbb{J}
    ('\u{1D542}', 639), // \mathbb{K}
    ('\u{1D543}', 611), // \mathbb{L}
    ('\u{1D544}', 722), // \mathbb{M}
    ('\u{2115}', 722),  // \mathbb{N}
    ('\u{1D546}', 667), // \mathbb{O}
    ('\u{2119}', 639),  // \mathbb{P}
    ('\u{211A}', 667),  // \mathbb{Q}
    ('\u{211D}', 639),  // \mathbb{R}
    ('\u{1D54A}', 611), // \mathbb{S}
    ('\u{1D54B}', 611), // \mathbb{T}
    ('\u{1D54C}', 722), // \mathbb{U}
    ('\u{1D54D}', 611), // \mathbb{V}
    ('\u{1D54E}', 833), // \mathbb{W}
    ('\u{1D54F}', 667), // \mathbb{X}
    ('\u{1D550}', 611), // \mathbb{Y}
    ('\u{2124}', 667),  // \mathbb{Z}
    ('\u{2216}', 568),  // \setminus
    ('\u{27F9}', 1457), // \Longrightarrow
];

/// The double-struck code point for `\mathbb{letter}`: the Mathematical
/// Alphanumeric block, except the seven letters Unicode encodes in
/// Letterlike Symbols (whose Alphanumeric slots are reserved).
pub fn double_struck(letter: char) -> Option<char> {
    let exception = match letter {
        'C' => Some('\u{2102}'),
        'H' => Some('\u{210D}'),
        'N' => Some('\u{2115}'),
        'P' => Some('\u{2119}'),
        'Q' => Some('\u{211A}'),
        'R' => Some('\u{211D}'),
        'Z' => Some('\u{2124}'),
        _ => None,
    };
    match letter {
        'A'..='Z' => exception.or_else(|| char::from_u32(0x1D538 + (letter as u32 - 'A' as u32))),
        _ => None,
    }
}

pub fn advance(c: char) -> Option<u16> {
    ADVANCES
        .iter()
        .find(|(glyph, _)| *glyph == c)
        .map(|(_, advance)| *advance)
}

/// True when `text` is non-empty and every character comes from this resource.
pub fn covers(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| advance(c).is_some())
}

/// Width of `text` at `size` points, when every character is covered.
pub fn width_pt(text: &str, size: f64) -> Option<f64> {
    if !covers(text) {
        return None;
    }
    Some(
        text.chars()
            .filter_map(advance)
            .map(|units| f64::from(units) * size / UNITS_PER_EM)
            .sum(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_capital_has_exactly_one_double_struck_glyph_in_the_table() {
        let mut seen = Vec::new();
        for letter in 'A'..='Z' {
            let glyph = double_struck(letter).expect("capital letter");
            assert!(advance(glyph).is_some(), "{letter} -> {glyph:?} missing");
            assert!(!seen.contains(&glyph));
            seen.push(glyph);
        }
        assert_eq!(double_struck('R'), Some('ℝ'));
        assert_eq!(double_struck('A'), Some('\u{1D538}'));
        assert_eq!(double_struck('a'), None);
        assert_eq!(double_struck('1'), None);
        assert_eq!(ADVANCES.len(), 28);
    }

    #[test]
    fn widths_scale_from_font_units() {
        assert_eq!(width_pt("ℝ", 10.0), Some(6.39));
        assert_eq!(width_pt("ℝx", 10.0), None);
        assert_eq!(width_pt("", 10.0), None);
    }
}
