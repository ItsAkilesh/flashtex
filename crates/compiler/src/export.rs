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
    ('\u{393}', 0x47),  // Gamma
    ('\u{394}', 0x44),  // Delta
    ('\u{398}', 0x51),  // Theta
    ('\u{39B}', 0x4C),  // Lambda
    ('\u{39E}', 0x58),  // Xi
    ('\u{3A0}', 0x50),  // Pi
    ('\u{3A3}', 0x53),  // Sigma
    ('\u{3A5}', 0x55),  // Upsilon
    ('\u{3A6}', 0x46),  // Phi
    ('\u{3A8}', 0x59),  // Psi
    ('\u{3A9}', 0x57),  // Omega
    ('\u{2202}', 0xB6), // partial
    ('\u{2207}', 0xD1), // nabla
    ('\u{2208}', 0xCE), // in
    ('\u{220F}', 0xD5), // prod
    ('\u{2192}', 0xAE), // to
    ('\u{2190}', 0xAC), // gets
    ('\u{21D2}', 0xDE), // Rightarrow
    ('\u{21D4}', 0xDB), // Leftrightarrow
    ('\u{2227}', 0xD9), // wedge
    ('\u{2228}', 0xDA), // vee
    ('\u{AC}', 0xD8),   // neg
    ('\u{2200}', 0x22), // forall
    ('\u{2203}', 0x24), // exists
    ('\u{2205}', 0xC6), // emptyset
    ('\u{2261}', 0xBA), // equiv
    ('\u{223C}', 0x7E), // sim
    ('\u{2282}', 0xCC), // subset
    ('\u{2286}', 0xCD), // subseteq
    ('\u{22A5}', 0x5E), // perp
    ('\u{2220}', 0xD0), // angle
];

/// WinAnsiEncoding's 0x80..0x9F block, which is NOT Latin-1.
///
/// This block is where WinAnsi puts typographic punctuation: em and en dashes,
/// curly quotes, the ellipsis and the bullet. Treating WinAnsi as plain Latin-1
/// wrongly reports all of them as unexportable, and they are among the most
/// common non-ASCII characters in ordinary prose — an em dash in a sentence
/// would have warned the author that their PDF was lossy when it was not.
const WINANSI_HIGH: &[(char, u8)] = &[
    ('\u{20AC}', 0x80), // Euro
    ('\u{201A}', 0x82), // single low quote
    ('\u{192}', 0x83),  // florin
    ('\u{201E}', 0x84), // double low quote
    ('\u{2026}', 0x85), // ellipsis
    ('\u{2020}', 0x86), // dagger
    ('\u{2021}', 0x87), // double dagger
    ('\u{2C6}', 0x88),  // circumflex
    ('\u{2030}', 0x89), // per mille
    ('\u{160}', 0x8A),  // S caron
    ('\u{2039}', 0x8B), // single left guillemet
    ('\u{152}', 0x8C),  // OE
    ('\u{17D}', 0x8E),  // Z caron
    ('\u{2018}', 0x91), // left single quote
    ('\u{2019}', 0x92), // right single quote
    ('\u{201C}', 0x93), // left double quote
    ('\u{201D}', 0x94), // right double quote
    ('\u{2022}', 0x95), // bullet
    ('\u{2013}', 0x96), // en dash
    ('\u{2014}', 0x97), // em dash
    ('\u{2DC}', 0x98),  // small tilde
    ('\u{2122}', 0x99), // trademark
    ('\u{161}', 0x9A),  // s caron
    ('\u{203A}', 0x9B), // single right guillemet
    ('\u{153}', 0x9C),  // oe
    ('\u{17E}', 0x9E),  // z caron
    ('\u{178}', 0x9F),  // Y dieresis
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
    if let Some((_, code)) = WINANSI_HIGH.iter().find(|(ch, _)| *ch == c) {
        return Glyph::Encodable {
            font: ExportFont::Text,
            code: *code,
        };
    }
    // Below U+0100 WinAnsi agrees with Latin-1, EXCEPT the 0x80..0x9F block
    // handled above, which Latin-1 leaves as control codes.
    if (c as u32) < 0x100 && c != '\u{7F}' && (c as u32) >= 0xA0
        || (0x20..0x7F).contains(&(c as u32))
    {
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

#[cfg(test)]
mod winansi_tests {
    use super::*;

    #[test]
    fn typographic_punctuation_is_exportable() {
        // These live in WinAnsi's 0x80..0x9F block, not in Latin-1. Reporting an
        // em dash as unexportable would tell an author their PDF is lossy when
        // it is not.
        for (c, expected) in [
            ('\u{2014}', 0x97u8), // em dash
            ('\u{2013}', 0x96),   // en dash
            ('\u{2018}', 0x91),   // left single quote
            ('\u{2019}', 0x92),   // right single quote
            ('\u{201C}', 0x93),   // left double quote
            ('\u{201D}', 0x94),   // right double quote
            ('\u{2026}', 0x85),   // ellipsis
            ('\u{2022}', 0x95),   // bullet
        ] {
            assert_eq!(
                map_char(c),
                Glyph::Encodable {
                    font: ExportFont::Text,
                    code: expected
                },
                "{c:?} should encode as WinAnsi 0x{expected:02X}"
            );
        }
    }

    #[test]
    fn characters_with_genuinely_no_base14_glyph_still_report() {
        // CJK has no glyph in any base-14 face. This must stay honest.
        for c in ['東', '京', '\u{1F600}'] {
            assert!(
                matches!(map_char(c), Glyph::Unrepresentable { .. }),
                "{c:?} has no base-14 glyph and must be reported"
            );
        }
    }
}
