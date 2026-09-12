//! WinAnsiEncoding (PDF 32000-1:2008, Annex D) for the base-14 fonts.
//!
//! This is a placeholder until FlashTeX embeds its own fonts. WinAnsi covers
//! ASCII, Latin-1, and the Windows-1252 punctuation block (curly quotes, dashes,
//! ellipsis, euro, ligatures Œ/œ, and so on). Everything else, including all
//! mathematics, Greek, CJK, and emoji, is not representable and is substituted.

/// The byte written for a character WinAnsi cannot represent.
pub const SUBSTITUTE: u8 = b'?';

/// Maps a character to its WinAnsi byte, or `None` if it has no code.
pub fn winansi_byte(c: char) -> Option<u8> {
    let cp = c as u32;
    match cp {
        0x20..=0x7E => Some(cp as u8),
        // 0xA0..=0xFF coincide with Latin-1. 0xA0 renders as a space and 0xAD as
        // a hyphen in WinAnsi, which is what those code points mean anyway.
        0xA0..=0xFF => Some(cp as u8),
        _ => match c {
            '€' => Some(0x80),
            '‚' => Some(0x82),
            'ƒ' => Some(0x83),
            '„' => Some(0x84),
            '…' => Some(0x85),
            '†' => Some(0x86),
            '‡' => Some(0x87),
            'ˆ' => Some(0x88),
            '‰' => Some(0x89),
            'Š' => Some(0x8A),
            '‹' => Some(0x8B),
            'Œ' => Some(0x8C),
            'Ž' => Some(0x8E),
            '\u{2018}' => Some(0x91),
            '\u{2019}' => Some(0x92),
            '\u{201C}' => Some(0x93),
            '\u{201D}' => Some(0x94),
            '•' => Some(0x95),
            '–' => Some(0x96),
            '—' => Some(0x97),
            '˜' => Some(0x98),
            '™' => Some(0x99),
            'š' => Some(0x9A),
            '›' => Some(0x9B),
            'œ' => Some(0x9C),
            'ž' => Some(0x9E),
            'Ÿ' => Some(0x9F),
            _ => None,
        },
    }
}

/// Result of encoding one text run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Encoded {
    /// WinAnsi bytes, one per input character, substitutions applied.
    pub bytes: Vec<u8>,
    /// Characters that were replaced by [`SUBSTITUTE`], in order of first
    /// appearance, without duplicates.
    pub unrepresentable: Vec<char>,
}

/// Encodes a string to WinAnsi, substituting characters that have no code.
pub fn encode(text: &str) -> Encoded {
    let mut bytes = Vec::with_capacity(text.len());
    let mut unrepresentable = Vec::new();
    for c in text.chars() {
        match winansi_byte(c) {
            Some(b) => bytes.push(b),
            None => {
                bytes.push(SUBSTITUTE);
                if !unrepresentable.contains(&c) {
                    unrepresentable.push(c);
                }
            }
        }
    }
    Encoded {
        bytes,
        unrepresentable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_and_latin1_are_identity() {
        assert_eq!(winansi_byte('A'), Some(0x41));
        assert_eq!(winansi_byte('é'), Some(0xE9));
        assert_eq!(winansi_byte('ï'), Some(0xEF));
        assert_eq!(winansi_byte('ÿ'), Some(0xFF));
    }

    #[test]
    fn windows_1252_punctuation_maps() {
        assert_eq!(winansi_byte('—'), Some(0x97));
        assert_eq!(winansi_byte('€'), Some(0x80));
        assert_eq!(winansi_byte('\u{201C}'), Some(0x93));
    }

    #[test]
    fn controls_and_non_latin_are_unrepresentable() {
        assert_eq!(winansi_byte('\n'), None);
        assert_eq!(winansi_byte('∫'), None);
        assert_eq!(winansi_byte('😀'), None);
        assert_eq!(winansi_byte('\u{81}'), None);
    }

    #[test]
    fn encode_substitutes_and_reports_once() {
        let e = encode("a∫b∫😀");
        assert_eq!(e.bytes, b"a?b??");
        assert_eq!(e.unrepresentable, vec!['∫', '😀']);
    }
}
