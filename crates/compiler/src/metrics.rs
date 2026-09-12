//! Advance-width metrics for the five Adobe Core 14 faces used by FlashTeX.
//!
//! Values are the standard AFM character widths in 1/1000 em. ASCII and the
//! printable ISO-8859-1 repertoire are covered. C0/C1 controls and characters
//! outside those ranges use [`DEFAULT_ADVANCE_UNITS`]; the fallback is applied
//! once per Unicode scalar value, so unsupported characters are never dropped.
//! These are unkerned advances: kerning and shaping belong to a later stage.

/// Fonts for which this module embeds Adobe Core 14 advance widths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Font {
    TimesRoman,
    TimesBold,
    TimesItalic,
    Helvetica,
    Courier,
}

/// Width used for an unsupported character, in 1/1000 em.
pub const DEFAULT_ADVANCE_UNITS: u16 = 500;

// Entries correspond to Unicode U+0020 through U+007E.
const TIMES_ROMAN_ASCII: [u16; 95] = [
    250, 333, 408, 500, 500, 833, 778, 180, 333, 333, 500, 564, 250, 333, 250, 278, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 278, 278, 564, 564, 564, 444, 921, 722, 667, 667, 722, 611,
    556, 722, 722, 333, 389, 722, 611, 889, 722, 722, 556, 722, 667, 556, 611, 722, 722, 944, 722,
    722, 611, 333, 278, 333, 469, 500, 333, 444, 500, 444, 500, 444, 333, 500, 500, 278, 278, 500,
    278, 778, 500, 500, 500, 500, 333, 389, 278, 500, 500, 722, 500, 500, 444, 480, 200, 480, 541,
];

const TIMES_BOLD_ASCII: [u16; 95] = [
    250, 333, 555, 500, 500, 1000, 833, 278, 333, 333, 500, 570, 250, 333, 250, 278, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 333, 333, 570, 570, 570, 500, 930, 722, 667, 722, 722, 667,
    611, 778, 778, 389, 500, 778, 667, 944, 722, 778, 611, 778, 722, 556, 667, 722, 722, 1000, 722,
    722, 667, 333, 278, 333, 581, 500, 333, 500, 556, 444, 556, 444, 333, 500, 556, 278, 333, 556,
    278, 833, 556, 500, 556, 556, 444, 389, 333, 556, 500, 722, 500, 500, 444, 394, 220, 394, 520,
];

const TIMES_ITALIC_ASCII: [u16; 95] = [
    250, 333, 420, 500, 500, 833, 778, 214, 333, 333, 500, 675, 250, 333, 250, 278, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 333, 333, 675, 675, 675, 500, 920, 611, 611, 667, 722, 611,
    611, 722, 722, 333, 444, 667, 556, 833, 667, 722, 611, 722, 611, 500, 556, 722, 611, 833, 611,
    556, 556, 389, 278, 389, 422, 500, 333, 500, 500, 444, 500, 444, 278, 500, 500, 278, 278, 444,
    278, 722, 500, 500, 500, 500, 389, 389, 278, 500, 444, 667, 444, 444, 389, 400, 275, 400, 541,
];

const HELVETICA_ASCII: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556,
    556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722, 722, 667,
    611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667,
    667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500,
    222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];

const COURIER_ASCII: [u16; 95] = [600; 95];

// Printable ISO-8859-1, U+00A0 through U+00FF. The Core 14 AFMs name these
// glyphs rather than assigning them Unicode values; this table records the
// corresponding standard widths explicitly.
const TIMES_ROMAN_LATIN1: [u16; 96] = [
    250, 333, 500, 500, 500, 500, 200, 500, 333, 760, 276, 500, 564, 333, 760, 500, 400, 564, 300,
    300, 333, 500, 453, 250, 333, 300, 310, 500, 750, 750, 750, 444, 722, 722, 722, 722, 722, 722,
    889, 667, 611, 611, 611, 611, 333, 333, 333, 333, 722, 722, 722, 722, 722, 722, 722, 564, 722,
    722, 722, 722, 722, 722, 556, 500, 444, 444, 444, 444, 444, 444, 667, 444, 444, 444, 444, 444,
    278, 278, 278, 278, 500, 500, 500, 500, 500, 500, 500, 564, 500, 500, 500, 500, 500, 500, 500,
    500,
];

const TIMES_BOLD_LATIN1: [u16; 96] = [
    250, 333, 500, 500, 500, 500, 220, 500, 333, 747, 300, 500, 570, 333, 747, 500, 400, 570, 300,
    300, 333, 556, 540, 250, 333, 300, 330, 500, 750, 750, 750, 500, 722, 722, 722, 722, 722, 722,
    1000, 722, 667, 667, 667, 667, 389, 389, 389, 389, 722, 722, 778, 778, 778, 778, 778, 570, 778,
    722, 722, 722, 722, 722, 611, 556, 500, 500, 500, 500, 500, 500, 722, 444, 444, 444, 444, 444,
    278, 278, 278, 278, 500, 556, 500, 500, 500, 500, 500, 570, 500, 556, 556, 556, 556, 500, 556,
    500,
];

const TIMES_ITALIC_LATIN1: [u16; 96] = [
    250, 389, 500, 500, 500, 500, 275, 500, 333, 760, 276, 500, 675, 333, 760, 500, 400, 675, 300,
    300, 333, 500, 523, 250, 333, 300, 310, 500, 750, 750, 750, 500, 611, 611, 611, 611, 611, 611,
    889, 667, 611, 611, 611, 611, 333, 333, 333, 333, 722, 667, 722, 722, 722, 722, 722, 675, 722,
    722, 722, 722, 722, 556, 611, 500, 500, 500, 500, 500, 500, 500, 667, 444, 444, 444, 444, 444,
    278, 278, 278, 278, 500, 500, 500, 500, 500, 500, 500, 675, 500, 500, 500, 500, 500, 444, 500,
    444,
];

const HELVETICA_LATIN1: [u16; 96] = [
    278, 333, 556, 556, 556, 556, 260, 556, 333, 737, 370, 556, 584, 333, 737, 333, 400, 584, 333,
    333, 333, 556, 537, 278, 333, 333, 365, 556, 834, 834, 834, 611, 667, 667, 667, 667, 667, 667,
    1000, 722, 667, 667, 667, 667, 278, 278, 278, 278, 722, 722, 778, 778, 778, 778, 778, 584, 778,
    722, 722, 722, 722, 667, 667, 611, 556, 556, 556, 556, 556, 556, 889, 500, 556, 556, 556, 556,
    278, 278, 278, 278, 556, 556, 556, 556, 556, 556, 556, 584, 556, 556, 556, 556, 556, 500, 556,
    500,
];

const COURIER_LATIN1: [u16; 96] = [600; 96];

fn width_units(font: Font, ch: char) -> u16 {
    let code = ch as u32;
    let table: &[u16] = if (0x20..=0x7e).contains(&code) {
        match font {
            Font::TimesRoman => &TIMES_ROMAN_ASCII,
            Font::TimesBold => &TIMES_BOLD_ASCII,
            Font::TimesItalic => &TIMES_ITALIC_ASCII,
            Font::Helvetica => &HELVETICA_ASCII,
            Font::Courier => &COURIER_ASCII,
        }
    } else if (0xa0..=0xff).contains(&code) {
        match font {
            Font::TimesRoman => &TIMES_ROMAN_LATIN1,
            Font::TimesBold => &TIMES_BOLD_LATIN1,
            Font::TimesItalic => &TIMES_ITALIC_LATIN1,
            Font::Helvetica => &HELVETICA_LATIN1,
            Font::Courier => &COURIER_LATIN1,
        }
    } else {
        return DEFAULT_ADVANCE_UNITS;
    };

    let first = if code <= 0x7e { 0x20 } else { 0xa0 };
    table[(code - first) as usize]
}

/// Returns the unkerned advance of one character at `size_pt`, in points.
pub fn advance_width(font: Font, ch: char, size_pt: f64) -> f64 {
    f64::from(width_units(font, ch)) * size_pt / 1000.0
}

/// Returns the sum of unkerned character advances at `size_pt`, in points.
pub fn string_width(font: Font, text: &str, size_pt: f64) -> f64 {
    text.chars()
        .map(|ch| advance_width(font, ch, size_pt))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_times_roman_widths_match_afm_units() {
        assert_eq!(advance_width(Font::TimesRoman, 'M', 1000.0), 889.0);
        assert_eq!(advance_width(Font::TimesRoman, ' ', 1000.0), 250.0);
    }

    #[test]
    fn known_string_is_hand_summed_at_twelve_points() {
        // "Mac": M=889, a=444, c=444 => 1777 units.
        assert!((string_width(Font::TimesRoman, "Mac", 12.0) - 21.324).abs() < 1e-12);
    }

    #[test]
    fn unsupported_characters_are_counted_with_the_documented_fallback() {
        assert_eq!(advance_width(Font::Helvetica, '🦀', 12.0), 6.0);
        assert_eq!(string_width(Font::Helvetica, "🦀🦀", 12.0), 12.0);
    }

    #[test]
    fn latin1_and_monospace_widths_are_available() {
        assert_eq!(advance_width(Font::TimesRoman, 'é', 1000.0), 444.0);
        assert_eq!(string_width(Font::Courier, "café", 10.0), 24.0);
    }
}
