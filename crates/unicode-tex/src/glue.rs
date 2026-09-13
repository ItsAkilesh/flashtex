//! Interword glue (`\fontdimen2/3/4/7`) for OpenType fonts under XeTeX and
//! LuaTeX, after fontspec's `WordSpace`/`PunctuationSpace`/`LetterSpace`.
//!
//! Measured (TeX Live 2026, `fixtures/expected/*` `fontdimens_pt`):
//!
//! | | XeTeX | LuaTeX (luaotfload) |
//! |---|---|---|
//! | space | advance of U+0020 (+ LetterSpace) | advance of U+0020 |
//! | stretch | space / 2 | space / 2; 0 when `post.isFixedPitch` |
//! | shrink | space / 3 | space / 3; 0 when fixed pitch |
//! | extra (`\fontdimen7`) | space / 3 | = shrink |
//!
//! luaotfload source: `fontloader-font-otl.lua` (space from U+0020, else
//! em-dash/2, else units/2; `monospaced` zeroes stretch and shrink;
//! `extra_space = space_shrink`). XeTeX: Menlo via `\newfontfamily` keeps
//! stretch 3.01025pt = space/2 although the font is fixed pitch; only
//! fontspec.cfg's `\ttfamily` `WordSpace={1,0,0}` zeroes it.
//!
//! fontspec then multiplies space/stretch/shrink by `WordSpace` (extra
//! untouched) and sets extra from `PunctuationSpace`.

use flashtex_font_engine::{Face, TrueTypeFace};

use crate::engine::{Engine, EngineProfile};
use crate::fontspec::{FeaturePlan, PunctuationSpace};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterwordGlue {
    pub space: f64,
    pub stretch: f64,
    pub shrink: f64,
    /// `\fontdimen7`, added to the natural space when the space factor is ≥ 2000.
    pub extra: f64,
}

/// Space advance in font units following luaotfload's fallbacks.
pub fn space_units(face: &TrueTypeFace) -> f64 {
    let adv = |c: char| {
        face.glyph_id(c)
            .and_then(|g| face.advance(g).ok())
            .map(f64::from)
    };
    adv(' ')
        .or_else(|| adv('\u{2014}').map(|w| w / 2.0))
        .unwrap_or(f64::from(face.units_per_em()) / 2.0)
}

/// Glue at `size_pt` (the size after `Scale=`).
pub fn interword_glue(
    face: &TrueTypeFace,
    size_pt: f64,
    profile: EngineProfile,
    plan: &FeaturePlan,
) -> InterwordGlue {
    let unit = size_pt / f64::from(face.units_per_em());
    let space = space_units(face) * unit;
    let ls = plan.letter_space_percent * size_pt / 100.0;
    let mut g = match profile.engine {
        Engine::XeTeX | Engine::PdfTeX => {
            let s = space + ls;
            InterwordGlue {
                space: s,
                stretch: s / 2.0,
                shrink: s / 3.0,
                extra: s / 3.0,
            }
        }
        Engine::LuaTeX => {
            if face.is_fixed_pitch() {
                InterwordGlue {
                    space,
                    stretch: 0.0,
                    shrink: 0.0,
                    extra: 0.0,
                }
            } else {
                InterwordGlue {
                    space,
                    stretch: space / 2.0,
                    shrink: space / 3.0,
                    extra: space / 3.0,
                }
            }
        }
    };
    if let Some(ws) = plan.word_space {
        g.space *= ws.space;
        g.stretch *= ws.stretch;
        g.shrink *= ws.shrink;
    }
    match plan.punctuation_space {
        Some(PunctuationSpace::WordSpace) => g.extra = 0.0,
        Some(PunctuationSpace::TwiceWordSpace) => g.extra = g.space,
        Some(PunctuationSpace::Scale(x)) => g.extra *= x,
        None => {}
    }
    g
}

/// TeX's space factor after a run of characters (LaTeX `\nonfrenchspacing`
/// sfcodes: `.?!` 3000, `:` 2000, `;` 1500, `,` 1250, upper-case letters 999,
/// `)` `]` `'` `’` `”` 0 = transparent; `\frenchspacing` makes all 1000
/// except the transparent ones and 999 for capitals).
pub fn space_factor(text: &str, french: bool, start: u32) -> u32 {
    let mut sf = start;
    for c in text.chars() {
        let code = match c {
            ')' | ']' | '\'' | '\u{2019}' | '\u{201D}' | '"' => 0,
            '.' | '?' | '!' if !french => 3000,
            ':' if !french => 2000,
            ';' if !french => 1500,
            ',' if !french => 1250,
            c if c.is_uppercase() => 999,
            _ => 1000,
        };
        if code == 0 {
            continue;
        }
        sf = if code > 1000 && sf < 1000 { 1000 } else { code };
    }
    sf
}

impl InterwordGlue {
    /// Natural width of an interword space after space factor `sf`
    /// (TeX: `\xspaceskip` is zero, so extra is added when sf ≥ 2000).
    pub fn natural(&self, sf: u32) -> f64 {
        if sf >= 2000 {
            self.space + self.extra
        } else {
            self.space
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_factor_rules() {
        assert_eq!(space_factor("home.", false, 1000), 3000);
        assert_eq!(space_factor("U.S.", false, 1000), 1000);
        assert_eq!(space_factor("Mr.", false, 1000), 3000);
        assert_eq!(space_factor("(Really.)", false, 1000), 3000);
        assert_eq!(space_factor("maybe;", false, 1000), 1500);
        assert_eq!(space_factor("home.", true, 1000), 1000);
        assert_eq!(space_factor("I", false, 1000), 999);
    }
}
