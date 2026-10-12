//! The bidirectional analysis `measure_native_node` asks ICU's `ubidi` for
//! (XeTeX_ext.c): `ubidi_setPara(text, getDefaultDirection(engine))`, then
//! `ubidi_getDirection`; a single-direction node is laid out in one call,
//! a `UBIDI_MIXED` one run by run in visual order (`ubidi_getVisualRun`).
//!
//! The Unicode Bidirectional Algorithm itself (UAX #9: P2–P3, X1–X10,
//! W1–W7, N0–N2, I1–I2, L1–L2) is the `unicode-bidi` crate's (Unicode 16.0,
//! the version of TeX Live 2026's ICU), which conforms to the same
//! algorithm as ICU's. What is ICU's own and decides how XeTeX shapes is
//! ported here: the paragraph level a `UBIDI_DEFAULT_LTR`/`_RTL` call
//! takes, and `directionFromFlags`, which calls a node `UBIDI_LTR` or
//! `UBIDI_RTL` from the bidi classes it holds (plus the paragraph level's
//! own), before any level is resolved: such a node is shaped whole, in that
//! direction, even where the resolved levels would split it (an Arabic
//! number next to Latin letters).

use unicode_bidi::utf16::ParagraphBidiInfo;
use unicode_bidi::{bidi_class, BidiClass, Level};

/// How `measure_native_node` lays out a node's text.
#[derive(Debug, PartialEq, Eq)]
pub enum Layout {
    /// `UBIDI_LTR` / `UBIDI_RTL`: the whole text in one direction.
    Whole { rtl: bool },
    /// `UBIDI_MIXED`: (start, length, rtl) in UTF-16 units, in visual order.
    Runs(Vec<(usize, usize, bool)>),
}

/// ICU's `DIRPROP_FLAG` masks (ubidiimp.h) over `unicode-bidi`'s classes.
fn flag(c: BidiClass) -> u32 {
    use BidiClass::*;
    1 << match c {
        L => 0,
        R => 1,
        EN => 2,
        ES => 3,
        ET => 4,
        AN => 5,
        CS => 6,
        B => 7,
        S => 8,
        WS => 9,
        ON => 10,
        LRE => 11,
        LRO => 12,
        AL => 13,
        RLE => 14,
        RLO => 15,
        PDF => 16,
        NSM => 17,
        BN => 18,
        FSI => 19,
        LRI => 20,
        RLI => 21,
        PDI => 22,
    }
}

fn mask(cs: &[BidiClass]) -> u32 {
    cs.iter().fold(0, |m, &c| m | flag(c))
}

/// The code points of UTF-16 `text` (an unpaired surrogate is U+FFFD, as
/// ICU's `U16_NEXT` gives it no class of its own: ON either way).
fn chars(text: &[u16]) -> impl Iterator<Item = char> + '_ {
    char::decode_utf16(text.iter().copied()).map(|r| r.unwrap_or('\u{FFFD}'))
}

/// The paragraph level of `ubidi_setPara` with `UBIDI_DEFAULT_LTR` or
/// `UBIDI_DEFAULT_RTL` (P2–P3): the first strong character's, not counting
/// those inside an isolate, else the default.
fn para_level(text: &[u16], default_rtl: bool) -> u8 {
    use BidiClass::*;
    let mut isolates = 0u32;
    for c in chars(text) {
        match bidi_class(c) {
            LRI | RLI | FSI => isolates += 1,
            PDI if isolates > 0 => isolates -= 1,
            L if isolates == 0 => return 0,
            R | AL if isolates == 0 => return 1,
            B => break,
            _ => {}
        }
    }
    u8::from(default_rtl)
}

/// `ubidi_setPara(text, default_rtl ? UBIDI_DEFAULT_RTL : UBIDI_DEFAULT_LTR)`
/// and how XeTeX then lays the text out.
pub fn analyse(text: &[u16], default_rtl: bool) -> Layout {
    use BidiClass::*;
    let level = para_level(text, default_rtl);
    // getDirProps: the classes present, plus the paragraph level's own
    // (for a default-level paragraph without a strong character)
    let mut flags = flag(if level & 1 == 1 { R } else { L });
    for c in chars(text) {
        flags |= flag(bidi_class(c));
    }
    // directionFromFlags
    let mask_rtl = mask(&[R, AL, RLE, RLO, RLI]);
    let mask_ltr = mask(&[L, EN, AN, LRE, LRO, LRI]);
    let mask_possible_n = mask(&[
        ON, CS, ES, ET, B, S, WS, BN, LRE, RLE, LRO, RLO, PDF, LRI, RLI, FSI, PDI,
    ]);
    if flags & mask_rtl == 0 && !(flags & flag(AN) != 0 && flags & mask_possible_n != 0) {
        return Layout::Whole { rtl: false };
    }
    if flags & mask_ltr == 0 {
        return Layout::Whole { rtl: true };
    }
    let info = ParagraphBidiInfo::new(text, Some(Level::new(level).unwrap_or(Level::ltr())));
    let mut levels: Vec<u8> = info.levels.iter().map(|l| l.number()).collect();
    adjust_ws_levels(&mut levels, &info.original_classes, level);
    Layout::Runs(visual_runs(&levels))
}

/// ICU's `adjustWSLevels` (L1, and the levels of the characters X9
/// removes): a sequence of whitespace, isolate controls, BN and explicit
/// codes before a segment or paragraph separator or the end goes to the
/// paragraph level, the separator too; any other BN or explicit code takes
/// the level of the character after it (`unicode-bidi` gives it the one
/// before: an RLO between X and abc then drew left of `cba`, not right).
fn adjust_ws_levels(levels: &mut [u8], classes: &[BidiClass], para: u8) {
    use BidiClass::*;
    let ws = |c: BidiClass| {
        matches!(
            c,
            B | S | WS | BN | LRE | RLE | LRO | RLO | PDF | LRI | RLI | FSI | PDI
        )
    };
    let bn_explicit = |c: BidiClass| matches!(c, BN | LRE | RLE | LRO | RLO | PDF);
    let mut i = levels.len();
    while i > 0 {
        // a sequence of WS/BN before the end (or a B/S) to the paragraph level
        while i > 0 {
            i -= 1;
            if !ws(classes[i]) {
                break;
            }
            levels[i] = para;
        }
        // BN to the next character's level, until a B/S
        while i > 0 {
            i -= 1;
            let c = classes[i];
            if bn_explicit(c) {
                levels[i] = levels[i + 1];
            } else if matches!(c, B | S) {
                levels[i] = para;
                break;
            }
        }
    }
}

/// ICU's runs (`getRuns`: maximal same-level runs) in visual order (L2,
/// `reorderLine`: from the highest level down to the lowest odd one, each
/// maximal sequence of runs at that level or above reversed), as (start,
/// length, rtl).
fn visual_runs(levels: &[u8]) -> Vec<(usize, usize, bool)> {
    let mut runs: Vec<(usize, usize, u8)> = vec![];
    for (i, &l) in levels.iter().enumerate() {
        match runs.last_mut() {
            Some(r) if r.2 == l => r.1 += 1,
            _ => runs.push((i, 1, l)),
        }
    }
    let max = runs.iter().map(|r| r.2).max().unwrap_or(0);
    let min_odd = runs.iter().map(|r| r.2).filter(|l| l & 1 == 1).min();
    if let Some(min_odd) = min_odd {
        for lev in (min_odd..=max).rev() {
            let mut k = 0;
            while k < runs.len() {
                if runs[k].2 < lev {
                    k += 1;
                    continue;
                }
                let start = k;
                while k < runs.len() && runs[k].2 >= lev {
                    k += 1;
                }
                runs[start..k].reverse();
            }
        }
    }
    runs.into_iter()
        .map(|(s, n, l)| (s, n, l & 1 == 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn single_direction_nodes() {
        assert_eq!(analyse(&u("office"), false), Layout::Whole { rtl: false });
        assert_eq!(analyse(&u("office"), true), Layout::Whole { rtl: false });
        // a Hebrew word, its period too
        assert_eq!(analyse(&u("עולם."), false), Layout::Whole { rtl: true });
        assert_eq!(analyse(&u("سلام"), false), Layout::Whole { rtl: true });
        // neutral only: the default decides
        assert_eq!(analyse(&u("."), true), Layout::Whole { rtl: true });
        assert_eq!(analyse(&u("."), false), Layout::Whole { rtl: false });
        // an Arabic number beside Latin letters, no neutral: ICU's LTR, whole
        assert_eq!(analyse(&u("١٢٣abc"), false), Layout::Whole { rtl: false });
    }

    #[test]
    fn mixed_nodes_are_visual_runs() {
        // Arabic-Indic digits in an Arabic paragraph: one LTR run (level 2)
        assert_eq!(
            analyse(&u("١٢٣٤٥"), true),
            Layout::Runs(vec![(0, 5, false)])
        );
        // European digits after an RTL default
        assert_eq!(analyse(&u("123"), true), Layout::Runs(vec![(0, 3, false)]));
        // Hebrew then digits: the number (level 2) stays left of nothing; the
        // RTL paragraph puts it visually first
        assert_eq!(
            analyse(&u("שלום123"), false),
            Layout::Runs(vec![(4, 3, false), (0, 4, true)])
        );
        // Latin then Hebrew in an LTR paragraph
        assert_eq!(
            analyse(&u("ab\u{5D0}\u{5D1}"), false),
            Layout::Runs(vec![(0, 2, false), (2, 2, true)])
        );
    }
}
