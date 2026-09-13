//! Character classes and the cmr10 width table of BibTeX 0.99.
//!
//! `bibtex.web` §§31–35 define `lex_class`, `id_class` and `char_width` for
//! seven-bit codes; the TeX Live change file (`bibtex.ch` [32], [33]) makes
//! codes 128–255 `alpha` and legal in identifiers and makes CR (13)
//! `white_space`. Codes 128–255 have width 0 (the width loop only covers
//! 0..127).

pub(crate) const ILLEGAL: u8 = 0;
pub(crate) const WHITE: u8 = 1;
pub(crate) const ALPHA: u8 = 2;
pub(crate) const NUMERIC: u8 = 3;
pub(crate) const SEP: u8 = 4;
pub(crate) const OTHER: u8 = 5;

#[inline]
pub(crate) fn lex(c: u8) -> u8 {
    match c {
        9 | 13 | 32 => WHITE,
        b'~' | b'-' => SEP,
        b'0'..=b'9' => NUMERIC,
        b'A'..=b'Z' | b'a'..=b'z' => ALPHA,
        128..=255 => ALPHA,
        0..=31 | 127 => ILLEGAL,
        _ => OTHER,
    }
}

#[inline]
pub(crate) fn white(c: u8) -> bool {
    lex(c) == WHITE
}

/// `id_class[c] = legal_id_char` (§33 plus `bibtex.ch` [33]).
#[inline]
pub(crate) fn legal_id(c: u8) -> bool {
    !matches!(
        c,
        0..=31 | b' ' | b'"' | b'#' | b'%' | b'\'' | b'(' | b')' | b',' | b'=' | b'{' | b'}'
    )
}

pub(crate) const SS_WIDTH: i32 = 500;
pub(crate) const AE_WIDTH: i32 = 722;
pub(crate) const OE_WIDTH: i32 = 778;
pub(crate) const UPPER_AE_WIDTH: i32 = 903;
pub(crate) const UPPER_OE_WIDTH: i32 = 1014;

/// `char_width` (§35), cmr10 widths in hundredths of a point.
const CHAR_WIDTH: [i32; 128] = {
    let mut w = [0i32; 128];
    let pairs: [(u8, i32); 95] = [
        (b' ', 278), (b'!', 278), (b'"', 500), (b'#', 833), (b'$', 500), (b'%', 833),
        (b'&', 778), (b'\'', 278), (b'(', 389), (b')', 389), (b'*', 500), (b'+', 778),
        (b',', 278), (b'-', 333), (b'.', 278), (b'/', 500), (b'0', 500), (b'1', 500),
        (b'2', 500), (b'3', 500), (b'4', 500), (b'5', 500), (b'6', 500), (b'7', 500),
        (b'8', 500), (b'9', 500), (b':', 278), (b';', 278), (b'<', 278), (b'=', 778),
        (b'>', 472), (b'?', 472), (b'@', 778), (b'A', 750), (b'B', 708), (b'C', 722),
        (b'D', 764), (b'E', 681), (b'F', 653), (b'G', 785), (b'H', 750), (b'I', 361),
        (b'J', 514), (b'K', 778), (b'L', 625), (b'M', 917), (b'N', 750), (b'O', 778),
        (b'P', 681), (b'Q', 778), (b'R', 736), (b'S', 556), (b'T', 722), (b'U', 750),
        (b'V', 750), (b'W', 1028), (b'X', 750), (b'Y', 750), (b'Z', 611), (b'[', 278),
        (b'\\', 500), (b']', 278), (b'^', 500), (b'_', 278), (b'`', 278), (b'a', 500),
        (b'b', 556), (b'c', 444), (b'd', 556), (b'e', 444), (b'f', 306), (b'g', 500),
        (b'h', 556), (b'i', 278), (b'j', 306), (b'k', 528), (b'l', 278), (b'm', 833),
        (b'n', 556), (b'o', 500), (b'p', 556), (b'q', 528), (b'r', 392), (b's', 394),
        (b't', 389), (b'u', 556), (b'v', 528), (b'w', 722), (b'x', 528), (b'y', 528),
        (b'z', 444), (b'{', 500), (b'|', 1000), (b'}', 500), (b'~', 500),
    ];
    let mut i = 0;
    while i < pairs.len() {
        w[pairs[i].0 as usize] = pairs[i].1;
        i += 1;
    }
    w
};

#[inline]
pub(crate) fn char_width(c: u8) -> i32 {
    if c < 128 { CHAR_WIDTH[c as usize] } else { 0 }
}

/// `lower_case` / `upper_case` (§§62–63): ASCII letters only.
pub(crate) fn lower(buf: &mut [u8]) {
    for c in buf {
        if c.is_ascii_uppercase() {
            *c += 32;
        }
    }
}

pub(crate) fn upper(buf: &mut [u8]) {
    for c in buf {
        if c.is_ascii_lowercase() {
            *c -= 32;
        }
    }
}

pub(crate) fn lowered(b: &[u8]) -> Vec<u8> {
    let mut v = b.to_vec();
    lower(&mut v);
    v
}

/// The thirteen control sequences of `control_seq_ilk` (§336).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Cs {
    I,
    J,
    Oe,
    OeU,
    Ae,
    AeU,
    Aa,
    AaU,
    O,
    OU,
    L,
    LU,
    Ss,
}

pub(crate) fn control_seq(name: &[u8]) -> Option<Cs> {
    Some(match name {
        b"i" => Cs::I,
        b"j" => Cs::J,
        b"oe" => Cs::Oe,
        b"OE" => Cs::OeU,
        b"ae" => Cs::Ae,
        b"AE" => Cs::AeU,
        b"aa" => Cs::Aa,
        b"AA" => Cs::AaU,
        b"o" => Cs::O,
        b"O" => Cs::OU,
        b"l" => Cs::L,
        b"L" => Cs::LU,
        b"ss" => Cs::Ss,
        _ => return None,
    })
}
