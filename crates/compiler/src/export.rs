//! Export glyph adapter: what the PDF path can and cannot represent.
//!
//! The compiler lays text out in Unicode. The PDF writer (`crates/pdf`, FT-009)
//! encodes with the base-14 fonts, whose repertoire is much smaller. Before this
//! module existed, the mismatch was discovered only at export time and resolved
//! by writing `?` — a student's exported equation silently lost its symbols.
//! Issue #9 records the reproduction.
//!
//! This table is the shared representation the two crates agree on. It is
//! exported from here so `crates/pdf` can consume it rather than re-deriving the
//! mapping and drifting apart again.
//!
//! What it deliberately does NOT do is invent a substitute. A character with no
//! base-14 glyph is reported as [`Glyph::Unrepresentable`] with a reason, and
//! the compiler raises a diagnostic so the author learns before exporting.

use crate::math::FRACTION_RULE_CHAR;

/// One of the 14 standard PDF fonts, or none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFont {
    /// Text faces encoded with WinAnsiEncoding.
    Text,
    /// The `Symbol` face, which carries Greek letters and mathematical operators.
    Symbol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    /// Encodable: use this font and this byte in that font's encoding.
    Encodable { font: ExportFont, code: u8 },
    /// No base-14 glyph exists. The reason is shown to the author.
    Unrepresentable { reason: &'static str },
}

/// Adobe Symbol encoding, for the symbols this compiler emits.
///
/// Values are the code points in the Symbol font's own encoding, not Unicode.
const SYMBOL_ENCODING: &[(char, u8)] = &[
    ('\u{3B1}', 0x61),  // alpha
    ('\u{3B2}', 0x62),  // beta
    ('\u{3B3}', 0x67),  // gamma
    ('\u{3B4}', 0x64),  // delta
    ('\u{3B8}', 0x71),  // theta
    ('\u{3BB}', 0x6C),  // lambda
    ('\u{3BC}', 0x6D),  // mu
    ('\u{3C0}', 0x70),  // pi
    ('\u{3C3}', 0x73),  // sigma
    ('\u{3C6}', 0x66),  // phi
    ('\u{3C9}', 0x77),  // omega
    ('\u{D7}', 0xB4),   // multiply
    ('\u{F7}', 0xB8),   // divide
    ('\u{B1}', 0xB1),   // plusminus
    ('\u{2264}', 0xA3), // lessequal
    ('\u{2265}', 0xB3), // greaterequal
    ('\u{2260}', 0xB9), // notequal
    ('\u{2248}', 0xBB), // approxequal
    ('\u{B7}', 0xD7),   // dotmath
    ('\u{221E}', 0xA5), // infinity
    ('\u{2211}', 0xE5), // summation
    ('\u{222B}', 0xF2), // integral
    ('\u{221A}', 0xD6), // radical
];

/// How a single character would be exported.
pub fn map_char(c: char) -> Glyph {
    if c == FRACTION_RULE_CHAR {
        return Glyph::Unrepresentable {
            reason: "fraction rules are drawn with a box-drawing character as a stand-in; \
                     runtime-v1 has no rule item type yet, so they cannot be exported faithfully",
        };
    }
    if let Some((_, code)) = SYMBOL_ENCODING.iter().find(|(ch, _)| *ch == c) {
        return Glyph::Encodable {
            font: ExportFont::Symbol,
            code: *code,
        };
    }
    // WinAnsiEncoding agrees with Latin-1 over the range the text path uses.
    if (c as u32) < 0x100 && c != '\u{7F}' && (c as u32) >= 0x20 {
        return Glyph::Encodable {
            font: ExportFont::Text,
            code: c as u32 as u8,
        };
    }
    Glyph::Unrepresentable {
        reason: "no glyph for this character exists in the base-14 PDF fonts",
    }
}

/// Characters in `text` that cannot be exported, in order, without duplicates.
pub fn unrepresentable(text: &str) -> Vec<char> {
    let mut out: Vec<char> = Vec::new();
    for c in text.chars() {
        if matches!(map_char(c), Glyph::Unrepresentable { .. }) && !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

/// The reason a character cannot be exported, for use in a diagnostic.
pub fn reason(c: char) -> Option<&'static str> {
    match map_char(c) {
        Glyph::Unrepresentable { reason } => Some(reason),
        Glyph::Encodable { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::COMMAND_GLYPHS;

    /// The guard that keeps this table honest: every symbol the math layer can
    /// emit must have a decided export outcome. Adding a symbol to
    /// `COMMAND_GLYPHS` without considering export fails here.
    #[test]
    fn every_math_symbol_has_a_decided_export_outcome() {
        for (command, glyph) in COMMAND_GLYPHS {
            for c in glyph.chars() {
                match map_char(c) {
                    Glyph::Encodable { font, .. } => {
                        assert_eq!(
                            font,
                            ExportFont::Symbol,
                            "\\{command} renders {c:?}, which should come from the Symbol font"
                        );
                    }
                    Glyph::Unrepresentable { reason } => {
                        panic!("\\{command} renders {c:?} which cannot be exported: {reason}");
                    }
                }
            }
        }
    }

    #[test]
    fn the_fraction_rule_is_reported_not_substituted() {
        let g = map_char(FRACTION_RULE_CHAR);
        let Glyph::Unrepresentable { reason } = g else {
            panic!("the fraction rule stand-in must not claim to be encodable");
        };
        assert!(
            reason.contains("rule item type"),
            "reason should name the contract gap: {reason}"
        );
    }

    #[test]
    fn ordinary_text_and_latin1_encode_as_text() {
        for c in ['A', 'z', '0', ' ', '.', 'é', 'ü', '±'] {
            assert!(
                matches!(map_char(c), Glyph::Encodable { .. }),
                "{c:?} should be encodable"
            );
        }
        assert_eq!(
            map_char('A'),
            Glyph::Encodable {
                font: ExportFont::Text,
                code: 0x41
            }
        );
        // A character that genuinely has no base-14 glyph.
        assert!(matches!(map_char('日'), Glyph::Unrepresentable { .. }));
    }

    #[test]
    fn unrepresentable_lists_each_offender_once() {
        let text = format!("a{0}b{0}c日日", FRACTION_RULE_CHAR);
        assert_eq!(unrepresentable(&text), vec![FRACTION_RULE_CHAR, '日']);
    }
}
