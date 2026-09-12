//! Three distinct identities that must never be cast into one another:
//!
//! * [`EncodingCode`]: an 8-bit slot in a TeX font encoding (T1 here). TFM
//!   metrics and TeX input conventions (`--`, ``` `` ```) are expressed in
//!   these; they are neither Unicode nor glyph indices.
//! * `char`: a Unicode scalar value, the currency of the compiler's parse tree
//!   and of the shaper's input.
//! * [`GlyphId`]: an ORIGINAL glyph index inside one specific font program
//!   (font-engine's type, re-exported). Only meaningful together with the
//!   content hash of that program.
//!
//! The only ways across are the explicit functions below: an encoding slot
//! resolves to a `char` through a declared encoding table, and a `char`
//! resolves to a `GlyphId` through one face's `cmap`. Nothing here is a
//! numeric conversion.

pub use flashtex_font_engine::GlyphId;

use flashtex_font_engine::Face;

/// An 8-bit character code in a TeX font encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EncodingCode(pub u8);

/// The TeX font encodings this pipeline can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    /// Cork (`T1`, `\usepackage[T1]{fontenc}`), the encoding of `ec-lm*` and
    /// `ptm*8t` and the one every fixture selects.
    T1,
}

/// The T1 slots this pipeline synthesises from TeX input conventions and
/// accent commands, declared explicitly. Slots outside this table are
/// reported as unmapped rather than guessed.
const T1_TO_UNICODE: &[(u8, char)] = &[
    (0x10, '\u{2018}'), // quoteleft   (`)
    (0x11, '\u{2019}'), // quoteright  (')
    (0x12, '\u{201C}'), // quotedblleft  (``)
    (0x13, '\u{201D}'), // quotedblright ('')
    (0x15, '\u{2013}'), // endash  (--)
    (0x16, '\u{2014}'), // emdash  (---)
    (0x1B, '\u{FB00}'), // ff
    (0x1C, '\u{FB01}'), // fi
    (0x1D, '\u{FB02}'), // fl
    (0x1E, '\u{FB03}'), // ffi
    (0x1F, '\u{FB04}'), // ffl
    (0x2D, '-'),        // hyphen
];

impl EncodingCode {
    /// T1 slot of the endash produced by `--`.
    pub const T1_ENDASH: EncodingCode = EncodingCode(0x15);
    /// T1 slot of the emdash produced by `---`.
    pub const T1_EMDASH: EncodingCode = EncodingCode(0x16);
    pub const T1_QUOTELEFT: EncodingCode = EncodingCode(0x10);
    pub const T1_QUOTERIGHT: EncodingCode = EncodingCode(0x11);
    pub const T1_QUOTEDBLLEFT: EncodingCode = EncodingCode(0x12);
    pub const T1_QUOTEDBLRIGHT: EncodingCode = EncodingCode(0x13);

    /// The Unicode character an encoding slot denotes, if this pipeline has
    /// declared it. `None` means "unknown slot", never "code point 0xNN".
    pub fn to_char(self, encoding: Encoding) -> Option<char> {
        match encoding {
            Encoding::T1 => {
                if let Some((_, c)) = T1_TO_UNICODE.iter().find(|(code, _)| *code == self.0) {
                    return Some(*c);
                }
                // Printable ASCII occupies its own code points in T1.
                if (0x20..0x7F).contains(&self.0) && self.0 != 0x2D {
                    return Some(char::from(self.0));
                }
                None
            }
        }
    }
}

/// Resolves a Unicode character to an original glyph id of `face` through
/// its character map. `None` is a missing glyph, to be reported; it is never
/// silently `.notdef`.
pub fn glyph_for_char(face: &dyn Face, ch: char) -> Option<GlyphId> {
    face.glyph_id(ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t1_slots_are_declared_not_cast() {
        assert_eq!(EncodingCode::T1_ENDASH.to_char(Encoding::T1), Some('\u{2013}'));
        assert_eq!(EncodingCode(0x41).to_char(Encoding::T1), Some('A'));
        // 0x15 is NOT U+0015 and an undeclared slot is unknown.
        assert_ne!(EncodingCode(0x15).to_char(Encoding::T1), Some('\u{15}'));
        assert_eq!(EncodingCode(0x00).to_char(Encoding::T1), None);
    }
}
