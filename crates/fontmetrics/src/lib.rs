//! Glyph advance widths and vertical metrics for the PDF base-14 text faces.
//!
//! The compiler lays text out, the Mac preview draws it with the system Times,
//! and `crates/pdf` writes it with the base-14 `Times-Roman`. All three agree
//! only if they measure words with the same advances. This crate holds those
//! advances as plain data (`src/tables.rs`), generated once on macOS by
//! `tools/dump-metrics.swift` and committed, so the crate builds anywhere with
//! zero dependencies.
//!
//! Units: tables are in 1/1000 em (the glyph-space unit of PDF base-14 AFM
//! metrics and `/Widths` arrays). `*_pt` functions scale by `size_pt / 1000`.
//!
//! Scope: per-glyph advances only, over the WinAnsi (Windows-1252) repertoire.
//! No kerning, no ligatures, no shaping, no font embedding. Characters outside
//! the repertoire are charged the face's `.notdef` width (the width of `?`,
//! which is what `crates/pdf` writes for them) and counted in
//! [`Measured::unknown_chars`].

pub mod tables;

/// One base-14 text face for which this crate carries metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    TimesRoman,
    TimesBold,
    TimesItalic,
    TimesBoldItalic,
    Helvetica,
    HelveticaBold,
    Courier,
}

impl Face {
    /// Every face, in table order.
    pub const ALL: [Face; 7] = [
        Face::TimesRoman,
        Face::TimesBold,
        Face::TimesItalic,
        Face::TimesBoldItalic,
        Face::Helvetica,
        Face::HelveticaBold,
        Face::Courier,
    ];

    /// The static metrics record for this face.
    pub fn metrics(self) -> &'static FaceMetrics {
        match self {
            Face::TimesRoman => &tables::TIMES_ROMAN_FACE,
            Face::TimesBold => &tables::TIMES_BOLD_FACE,
            Face::TimesItalic => &tables::TIMES_ITALIC_FACE,
            Face::TimesBoldItalic => &tables::TIMES_BOLD_ITALIC_FACE,
            Face::Helvetica => &tables::HELVETICA_FACE,
            Face::HelveticaBold => &tables::HELVETICA_BOLD_FACE,
            Face::Courier => &tables::COURIER_FACE,
        }
    }

    /// The PDF base-14 PostScript name, e.g. `"Times-Roman"`. Suitable for a
    /// `/BaseFont` entry.
    pub fn postscript_name(self) -> &'static str {
        self.metrics().postscript_name
    }
}

/// Raw metrics for one face, in 1/1000 em.
///
/// `descender` is negative (PDF `/Descent` convention). `advances` is sorted
/// by code point and contains no duplicates, so lookups binary-search it.
#[derive(Debug)]
pub struct FaceMetrics {
    pub postscript_name: &'static str,
    pub ascender: i16,
    pub descender: i16,
    pub cap_height: i16,
    pub x_height: i16,
    /// Advance of U+0020.
    pub space: u16,
    /// Advance charged for characters not in `advances` (the width of `?`).
    pub notdef: u16,
    pub advances: &'static [(u32, u16)],
}

impl FaceMetrics {
    /// Advance of `ch` in 1/1000 em, or `None` if the character is outside the
    /// WinAnsi repertoire this crate carries.
    pub fn advance(&self, ch: char) -> Option<u16> {
        let cp = ch as u32;
        self.advances
            .binary_search_by_key(&cp, |&(c, _)| c)
            .ok()
            .map(|i| self.advances[i].1)
    }
}

/// Result of measuring a string with unknown characters accounted for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measured {
    /// Total advance in points at the requested size.
    pub width_pt: f64,
    /// Number of characters that were outside the face's table and were charged
    /// the `.notdef` width instead of a real advance.
    pub unknown_chars: usize,
}

/// Vertical metrics scaled to a point size. `descender` is negative.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VerticalMetrics {
    pub ascender: f64,
    pub descender: f64,
    pub cap_height: f64,
    pub x_height: f64,
}

impl VerticalMetrics {
    /// Vertical metrics of `face` at `size_pt`.
    pub fn at(face: Face, size_pt: f64) -> VerticalMetrics {
        let m = face.metrics();
        let s = size_pt / 1000.0;
        VerticalMetrics {
            ascender: f64::from(m.ascender) * s,
            descender: f64::from(m.descender) * s,
            cap_height: f64::from(m.cap_height) * s,
            x_height: f64::from(m.x_height) * s,
        }
    }

    /// `ascender - descender`: the natural line height without leading.
    pub fn height(&self) -> f64 {
        self.ascender - self.descender
    }
}

/// Advance of `ch` in `face`, in 1/1000 em. `None` outside the WinAnsi
/// repertoire; use [`measure`] if you want the `.notdef` fallback applied.
pub fn advance(face: Face, ch: char) -> Option<u16> {
    face.metrics().advance(ch)
}

/// Width of `text` in points at `size_pt`: the sum of per-character advances.
/// Unknown characters are charged the `.notdef` width silently; use
/// [`measure`] to learn how many there were.
pub fn width_pt(face: Face, text: &str, size_pt: f64) -> f64 {
    measure(face, text, size_pt).width_pt
}

/// Like [`width_pt`], but also reports how many characters fell back to the
/// `.notdef` width because they are outside the table.
pub fn measure(face: Face, text: &str, size_pt: f64) -> Measured {
    let m = face.metrics();
    let mut units: u64 = 0;
    let mut unknown_chars = 0usize;
    for ch in text.chars() {
        match m.advance(ch) {
            Some(a) => units += u64::from(a),
            None => {
                units += u64::from(m.notdef);
                unknown_chars += 1;
            }
        }
    }
    Measured {
        width_pt: units as f64 * size_pt / 1000.0,
        unknown_chars,
    }
}

/// Advance of U+0020 in `face` at `size_pt`, in points.
pub fn space_pt(face: Face, size_pt: f64) -> f64 {
    f64::from(face.metrics().space) * size_pt / 1000.0
}

/// Vertical metrics of `face` at `size_pt`. Convenience for
/// [`VerticalMetrics::at`].
pub fn vertical_metrics(face: Face, size_pt: f64) -> VerticalMetrics {
    VerticalMetrics::at(face, size_pt)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn hello_times_roman_12pt() {
        // CoreText at 1000 pt: H 722.17, e 443.85, l 277.83, l 277.83, o 500.00
        // = 2221.68 units → 26.660 pt at 12 pt. The rounded table gives
        // 722+444+278+278+500 = 2222 units → 26.664 pt.
        let w = width_pt(Face::TimesRoman, "Hello", 12.0);
        assert!(w > 26.0 && w < 28.0, "width {w}");
        assert!(close(w, 26.66, 0.5), "width {w}");
        assert!(close(w, 26.664, 1e-9), "width {w}");
    }

    #[test]
    fn accented_forms_share_advances_in_times_roman() {
        let m = Face::TimesRoman.metrics();
        for (base, accented) in [
            ('e', "éèêë"),
            ('a', "àáâãäå"),
            ('o', "òóôõö"),
            ('u', "ùúûü"),
            ('i', "ìíîï"),
            ('n', "ñ"),
            ('c', "ç"),
            ('E', "ÈÉÊË"),
            ('A', "ÀÁÂÃÄÅ"),
            ('O', "ÒÓÔÕÖ"),
            ('U', "ÙÚÛÜ"),
            ('N', "Ñ"),
            ('C', "Ç"),
        ] {
            let b = m.advance(base).unwrap();
            for ch in accented.chars() {
                assert_eq!(m.advance(ch), Some(b), "{ch} vs {base}");
            }
        }
    }

    #[test]
    fn courier_ascii_printables_are_600() {
        let m = Face::Courier.metrics();
        for cp in 0x20u32..=0x7E {
            let ch = char::from_u32(cp).unwrap();
            assert_eq!(m.advance(ch), Some(600), "U+{cp:04X}");
        }
        assert_eq!(m.space, 600);
        assert_eq!(m.notdef, 600);
    }

    #[test]
    fn tables_are_sorted_unique_and_cover_winansi() {
        for face in Face::ALL {
            let m = face.metrics();
            assert_eq!(m.advances.len(), 218, "{face:?}");
            for w in m.advances.windows(2) {
                assert!(w[0].0 < w[1].0, "{face:?}: {:#x} !< {:#x}", w[0].0, w[1].0);
            }
            for cp in (0x20u32..=0x7E).chain(0xA0..=0xFF) {
                let ch = char::from_u32(cp).unwrap();
                assert!(m.advance(ch).is_some(), "{face:?} U+{cp:04X}");
            }
            for ch in "€‚ƒ„…†‡ˆ‰Š‹ŒŽ‘’“”•–—˜™š›œžŸ".chars()
            {
                assert!(m.advance(ch).is_some(), "{face:?} {ch}");
            }
            for &(_, a) in m.advances {
                assert!(a > 0, "{face:?}: zero advance");
            }
            assert_eq!(m.advance(' '), Some(m.space), "{face:?}");
            assert_eq!(m.advance('?'), Some(m.notdef), "{face:?}");
            assert!(m.ascender > 0 && m.descender < 0, "{face:?}");
            assert!(m.cap_height > m.x_height && m.x_height > 0, "{face:?}");
        }
    }

    #[test]
    fn unknown_chars_use_notdef_and_are_counted() {
        assert_eq!(advance(Face::TimesRoman, 'λ'), None);
        assert_eq!(advance(Face::TimesRoman, '∑'), None);
        let m = measure(Face::TimesRoman, "aλb∑", 10.0);
        assert_eq!(m.unknown_chars, 2);
        let expected = (444 + 444 + 500 + 444) as f64 * 10.0 / 1000.0;
        assert!(close(m.width_pt, expected, 1e-9), "{m:?}");
        assert!(close(
            width_pt(Face::TimesRoman, "aλb∑", 10.0),
            expected,
            1e-9
        ));
        assert_eq!(measure(Face::Helvetica, "ok", 10.0).unknown_chars, 0);
    }

    #[test]
    fn space_and_vertical_metrics_scale() {
        assert!(close(space_pt(Face::TimesRoman, 10.0), 2.5, 1e-9));
        assert!(close(space_pt(Face::Helvetica, 10.0), 2.78, 1e-9));
        assert!(close(space_pt(Face::Courier, 10.0), 6.0, 1e-9));
        let v = VerticalMetrics::at(Face::TimesRoman, 10.0);
        assert!(close(v.ascender, 7.5, 1e-9));
        assert!(close(v.descender, -2.5, 1e-9));
        assert!(close(v.height(), 10.0, 1e-9));
        assert!(v.cap_height > v.x_height && v.x_height > 0.0);
        assert_eq!(vertical_metrics(Face::TimesRoman, 10.0), v);
    }

    #[test]
    fn empty_text_is_zero_wide() {
        let m = measure(Face::TimesBold, "", 12.0);
        assert_eq!(
            m,
            Measured {
                width_pt: 0.0,
                unknown_chars: 0
            }
        );
    }

    #[test]
    fn postscript_names_match_base_14() {
        assert_eq!(Face::TimesRoman.postscript_name(), "Times-Roman");
        assert_eq!(Face::TimesBold.postscript_name(), "Times-Bold");
        assert_eq!(Face::TimesItalic.postscript_name(), "Times-Italic");
        assert_eq!(Face::TimesBoldItalic.postscript_name(), "Times-BoldItalic");
        assert_eq!(Face::Helvetica.postscript_name(), "Helvetica");
        assert_eq!(Face::HelveticaBold.postscript_name(), "Helvetica-Bold");
        assert_eq!(Face::Courier.postscript_name(), "Courier");
    }

    #[test]
    fn placeholder_ratio_overestimates_real_times() {
        // The compiler's 0.5 × size placeholder for "Hello" at 12 pt is 30 pt;
        // real Times is 26.664 pt. This is the overlap FT-005 observed.
        let placeholder = 5.0 * 0.5 * 12.0;
        assert!(width_pt(Face::TimesRoman, "Hello", 12.0) < placeholder);
    }
}
