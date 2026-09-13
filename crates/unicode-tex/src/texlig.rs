//! TeX input ligatures for OpenType fonts (`Ligatures=TeX`).
//!
//! pdfLaTeX gets ``` `` ''  -- --- !` ?` ``` from the ligature program inside
//! every text TFM (always on, for every family). Under the Unicode engines
//! they are an opt-in font feature, and fontspec's `fontspec.cfg` turns it on
//! only for `\rmfamily` and `\sffamily` (`\setmainfont`, `\setsansfont`);
//! `\setmonofont`, `\newfontfamily` and `\fontspec` do not get it.
//!
//! * XeTeX: the TECkit mapping `tex-text.map` (TeX Live
//!   `fonts/misc/xetex/fontmapping/base/tex-text.map`) rewrites *characters*
//!   before shaping, independent of which glyphs the font has.
//! * LuaTeX: luaotfload's `tlig` feature (`luaotfload-features.lua`,
//!   `tlig_specification`): a glyph substitution (`"`→”, `'`→’, `` ` ``→‘) and a
//!   ligature lookup with the same sequences as tex-text. Being a font
//!   feature, a replacement only happens when the font has the target glyph.
//!
//! Both tables contain the same pairs; the only modelled difference is the
//! glyph-presence condition.

use crate::engine::Engine;

/// Multi-character sequences, longest first (matching order matters for `---`).
pub const TEX_TEXT_LIGATURES: &[(&str, char)] = &[
    ("---", '\u{2014}'),
    ("--", '\u{2013}'),
    ("``", '\u{201C}'),
    ("''", '\u{201D}'),
    ("!`", '\u{00A1}'),
    ("?`", '\u{00BF}'),
    (",,", '\u{201E}'),
    ("<<", '\u{00AB}'),
    (">>", '\u{00BB}'),
];

/// Single-character replacements.
pub const TEX_TEXT_SINGLES: &[(char, char)] =
    &[('\'', '\u{2019}'), ('`', '\u{2018}'), ('"', '\u{201D}')];

/// Text after the TeX-ligature pass plus, for each output char, the byte
/// range in the input it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapped {
    pub text: String,
    pub source: Vec<std::ops::Range<usize>>,
}

/// Applies the TeX input ligatures to `text` as `engine` would. `has_glyph`
/// answers whether the font maps a character (only consulted for LuaTeX).
pub fn apply(text: &str, engine: Engine, has_glyph: &dyn Fn(char) -> bool) -> Mapped {
    let mut out = String::with_capacity(text.len());
    let mut source = Vec::with_capacity(text.len());
    let needs_glyph = engine == Engine::LuaTeX;
    let mut i = 0;
    'outer: while i < text.len() {
        let rest = &text[i..];
        for (seq, rep) in TEX_TEXT_LIGATURES {
            if rest.starts_with(seq) && (!needs_glyph || has_glyph(*rep)) {
                out.push(*rep);
                source.push(i..i + seq.len());
                i += seq.len();
                continue 'outer;
            }
        }
        let c = rest.chars().next().unwrap();
        let mut emitted = c;
        for (from, to) in TEX_TEXT_SINGLES {
            if c == *from && (!needs_glyph || has_glyph(*to)) {
                emitted = *to;
            }
        }
        out.push(emitted);
        source.push(i..i + c.len_utf8());
        i += c.len_utf8();
    }
    Mapped { text: out, source }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xetex_maps_all() {
        let m = apply(
            "``a'' --- b -- c !`d <<e>> \"f\" it's",
            Engine::XeTeX,
            &|_| false,
        );
        assert_eq!(m.text, "“a” — b – c ¡d «e» ”f” it’s");
        assert_eq!(m.source[0], 0..2);
        assert_eq!(m.source.len(), m.text.chars().count());
    }

    #[test]
    fn luatex_needs_target_glyph() {
        let m = apply("a--b 'c'", Engine::LuaTeX, &|c| c != '\u{2013}');
        assert_eq!(m.text, "a--b ’c’");
    }
}
