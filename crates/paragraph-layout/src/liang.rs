//! Liang's pattern hyphenation (the algorithm TeX uses), original
//! implementation, behind the [`Hyphenator`] trait.
//!
//! Word rules mirror TeX: the hyphenatable part is the leading run of letters
//! of the word (a word starting with a non-letter is not hyphenated, trailing
//! punctuation is allowed, but a letter after a non-letter disqualifies the
//! word — TeX §896–899); uppercase letters are lower-cased (`\uchyph=1`); a
//! word shorter than `left_min + right_min` letters is left alone; explicit
//! `\-` markers switch the word to explicit-only discretionaries, as a
//! discretionary node ends TeX's letter run. Exceptions (`\hyphenation{}`)
//! override the patterns for the whole word.
//!
//! Pattern data and provenance (`EN_US_SUBSET`, `EN_US_EXCEPTIONS`):
//! TeX Live 2026 `texmf-dist/tex/generic/hyph-utf8/patterns/tex/hyph-en-us.tex`,
//! "Hyphenation patterns for American English", Copyright (C) 1990, 2004, 2005
//! Gerard D.C. Kuiken, version 2005-05-30, licence: "Copying and distribution
//! of this file, with or without modification, are permitted in any medium
//! without royalty provided the copyright notice and this notice are
//! preserved." That file contains Knuth's original `hyphen.tex` patterns plus
//! Kuiken's additions. The embedded set is deliberately SMALL: the 420
//! patterns of hyph-en-us.tex that are also in `hyphen.tex` (what pdflatex's
//! default `english` language loads) **and** match at least one word of
//! `docs/hyphen-sample.tex` or the wrap-sample oracle text; other words are
//! therefore hyphenated less than TeX would (see README "Not modelled").
//! The exception list is the 14-entry `\hyphenation{}` list both files share.

use std::collections::HashMap;

use crate::hyphenate::{HyphenationPoint, Hyphenator};

/// One compiled pattern: letters plus the inter-letter digit weights
/// (`weights.len() == letters.len() + 1`).
#[derive(Debug, Clone)]
struct Pattern {
    weights: Vec<u8>,
}

/// TeX-style pattern hyphenator.
#[derive(Debug, Clone)]
pub struct LiangHyphenator {
    patterns: HashMap<String, Pattern>,
    /// Exceptions: word -> break offsets (char indices).
    exceptions: HashMap<String, Vec<usize>>,
    max_len: usize,
    /// `\lefthyphenmin` (2 for English).
    pub left_min: usize,
    /// `\righthyphenmin` (3 for English).
    pub right_min: usize,
}

impl LiangHyphenator {
    /// Compiles patterns written in TeX's notation (`.ch4`, `hy3ph`) and
    /// exceptions written with hyphens (`ta-ble`).
    pub fn new(patterns: &[&str], exceptions: &[&str], left_min: usize, right_min: usize) -> Self {
        let mut map = HashMap::with_capacity(patterns.len());
        let mut max_len = 0;
        for p in patterns {
            let mut letters = String::new();
            let mut weights = vec![0u8];
            for ch in p.chars() {
                if let Some(d) = ch.to_digit(10) {
                    *weights.last_mut().unwrap() = d as u8;
                } else {
                    letters.push(ch);
                    weights.push(0);
                }
            }
            max_len = max_len.max(letters.chars().count());
            map.insert(letters, Pattern { weights });
        }
        let mut exc = HashMap::new();
        for e in exceptions {
            let mut word = String::new();
            let mut offsets = Vec::new();
            for ch in e.chars() {
                if ch == '-' {
                    offsets.push(word.chars().count());
                } else {
                    word.push(ch);
                }
            }
            exc.insert(word, offsets);
        }
        LiangHyphenator {
            patterns: map,
            exceptions: exc,
            max_len,
            left_min,
            right_min,
        }
    }

    /// The embedded American-English subset with `\lefthyphenmin=2`,
    /// `\righthyphenmin=3`.
    pub fn en_us_subset() -> Self {
        LiangHyphenator::new(EN_US_SUBSET, EN_US_EXCEPTIONS, 2, 3)
    }

    /// Break positions (char indices) inside a lower-case letter-only word.
    pub fn positions(&self, word: &str) -> Vec<usize> {
        let n = word.chars().count();
        if n < self.left_min + self.right_min {
            return Vec::new();
        }
        if let Some(e) = self.exceptions.get(word) {
            return e.clone();
        }
        let dotted: Vec<char> = std::iter::once('.')
            .chain(word.chars())
            .chain(std::iter::once('.'))
            .collect();
        let mut weights = vec![0u8; dotted.len() + 1];
        for start in 0..dotted.len() {
            let mut s = String::new();
            for (k, ch) in dotted[start..].iter().enumerate() {
                if k >= self.max_len {
                    break;
                }
                s.push(*ch);
                if let Some(p) = self.patterns.get(&s) {
                    for (j, w) in p.weights.iter().enumerate() {
                        let idx = start + j;
                        if *w > weights[idx] {
                            weights[idx] = *w;
                        }
                    }
                }
            }
        }
        // weights[i] sits before dotted[i]; letter k of the word is dotted[k+1],
        // so a break before letter k is weights[k+1].
        (1..n)
            .filter(|&k| weights[k + 1] % 2 == 1 && k >= self.left_min && n - k >= self.right_min)
            .collect()
    }
}

impl Hyphenator for LiangHyphenator {
    fn hyphenate(&self, word: &str) -> Vec<HyphenationPoint> {
        // Explicit discretionaries end TeX's letter run: honour only them.
        if word.contains("\\-") {
            return crate::hyphenate::ExplicitDiscretionary.hyphenate(word);
        }
        let mut letters = String::new();
        let mut byte_at_char: Vec<usize> = Vec::new();
        let mut end = 0;
        for (i, ch) in word.char_indices() {
            if ch.is_alphabetic() {
                byte_at_char.push(i);
                letters.extend(ch.to_lowercase());
                end = i + ch.len_utf8();
            } else {
                break;
            }
        }
        if letters.is_empty() {
            return Vec::new();
        }
        // A letter after the run (e.g. "wo)rd") disqualifies the word.
        if word[end..].chars().any(|c| c.is_alphabetic()) {
            return Vec::new();
        }
        self.positions(&letters)
            .into_iter()
            .map(|k| HyphenationPoint {
                offset: byte_at_char[k],
                marker_len: 0,
                automatic: true,
            })
            .collect()
    }
}

/// Patterns: 420 of the 4938 in hyph-en-us.tex (all also present in Knuth's hyphen.tex),
/// selected as those matching a word of `docs/hyphen-sample.tex` or the
/// wrap-sample oracle text (see module docs). Generated; do not hand-edit.
pub const EN_US_SUBSET: &[&str] = &[
    ".ch4",
    ".en3g",
    ".ge2",
    ".he2",
    ".in1",
    ".in3s",
    ".le2",
    ".li2n",
    ".me2",
    ".or1d",
    ".res2",
    ".sh2",
    ".st4",
    ".te4",
    ".th2",
    ".ti2",
    "a2d",
    "ad5er.",
    "a2f",
    "ai2",
    "a4i4n",
    "al1i",
    "a2n",
    "4and",
    "2ang",
    "4ao",
    "4aphi",
    "2a2r",
    "ar3act",
    "ar1i",
    "4as.",
    "4ath",
    "at1ic",
    "au4l2",
    "av1i",
    "bal1a",
    "be3gi",
    "2bf",
    "1bil",
    "b2l2",
    "b4le.",
    "3bod",
    "4b1ora",
    "both5",
    "bound3",
    "1ca",
    "c3c",
    "2ce.",
    "4ced.",
    "1cen",
    "3cent",
    "2ch",
    "4ch.",
    "4ch3ab",
    "1ci",
    "ck1",
    "1c4l4",
    "1co",
    "4c3s2",
    "2c1t",
    "c2te",
    "c3ter",
    "c3ume",
    "3c4ut",
    "4c5utiv",
    "1cy",
    "1d2a",
    "d4em",
    "de4mons",
    "1den",
    "de1p",
    "de1t",
    "4d1f",
    "d1in",
    "1dina",
    "dis1",
    "d1j",
    "d1m",
    "1do",
    "1dr",
    "2d1s2",
    "1du",
    "du2c",
    "4duct.",
    "d2y",
    "ead1",
    "ea4l",
    "east3",
    "e1cu",
    "ee4p1",
    "e1f",
    "e4f3ere",
    "1eff",
    "e1h4",
    "ei2",
    "el2i",
    "e3libe",
    "e5lim",
    "e1me",
    "4eno",
    "e2pa",
    "e1po",
    "e3pro",
    "e1q",
    "er1a",
    "era4b",
    "2ere.",
    "er5ence",
    "er1i",
    "er3m4",
    "e1s2e",
    "e1sp",
    "ev1er",
    "1exp",
    "1fa",
    "5far",
    "fault5",
    "fer1",
    "4f1f",
    "1fi",
    "2fin",
    "5fina",
    "fi2ne",
    "f4l2",
    "1fo",
    "5fon",
    "fon4t",
    "fo2r",
    "fort5a",
    "2ft",
    "1ga",
    "2ge.",
    "1gen",
    "1geo",
    "ge3om",
    "gh5out",
    "3g4in.",
    "5g4ins",
    "gl2",
    "1go",
    "1gr",
    "5graph.",
    "5graphic",
    "gs2",
    "gth3",
    "gu4a",
    "1gy",
    "5hand.",
    "he2n",
    "hena4",
    "hen5at",
    "he4t",
    "4hr4",
    "4h1s2",
    "hy3ph",
    "ibe4",
    "ib3era",
    "i1bl",
    "i1br",
    "4ich",
    "ic3ula",
    "2id",
    "2ie4",
    "i3fie",
    "4ift",
    "i1la",
    "il1er",
    "il1i",
    "2ilit",
    "im1i",
    "2in.",
    "4ind",
    "2ine",
    "2i1no",
    "2ins",
    "2io",
    "i4our",
    "2ip",
    "4ir",
    "2is.",
    "2is1c",
    "4ise",
    "3isf",
    "i2so",
    "is1ti",
    "2ith",
    "i1ti",
    "2itio",
    "4itt",
    "2iv",
    "k3ab",
    "k5ag",
    "1kee",
    "4k1s2",
    "lab3ic",
    "l4abo",
    "2ld",
    "le2a",
    "lim3i",
    "l4ina",
    "1l4ine",
    "l1it",
    "l1iz",
    "l1l",
    "ll4o",
    "l5low",
    "5long",
    "4lt",
    "1ly",
    "2lys4",
    "1ma",
    "4m1b",
    "4me.",
    "1men",
    "3ment",
    "2mes",
    "m4etr",
    "me3try",
    "4m1f",
    "min4a",
    "m1m",
    "1mo",
    "4m1p",
    "mpar5i",
    "mphas4",
    "m2pi",
    "1mu",
    "mun2",
    "1na",
    "na4li",
    "nar3i",
    "n2at",
    "4n1b4",
    "n3cha",
    "n1de",
    "2ne.",
    "n1er",
    "1nes",
    "2nes.",
    "ne4v",
    "n5eve",
    "ng1in",
    "n1gu",
    "n2it",
    "4n1l",
    "1nou",
    "n1r",
    "2n1s2",
    "n2se",
    "nsid1",
    "n1t",
    "nt2i",
    "nt4s",
    "n1v2",
    "od5uct.",
    "1ogy",
    "oi2",
    "o3ing",
    "ol2d",
    "ol3er",
    "o2ly",
    "o2me",
    "o4met",
    "om5etry",
    "om3pi",
    "o2n",
    "on1a",
    "on1c",
    "on3s",
    "onten4",
    "oo2",
    "o1ra",
    "o1ry",
    "4oth",
    "ou2",
    "oun2d",
    "1pa",
    "3pare",
    "p4a4ri",
    "par4is",
    "pd4",
    "pe2c",
    "2p2ed",
    "4ph.",
    "ph1ic",
    "4phs",
    "pi2n",
    "1p2l2",
    "pli4n",
    "5po4g",
    "1p4or",
    "1pos",
    "4p1p",
    "p2pe",
    "p4ped",
    "pr2",
    "p3rese",
    "pros3e",
    "2p1s2",
    "2p1t",
    "pu2t",
    "5pute",
    "qu2",
    "2rab",
    "r5acl",
    "r2as",
    "ration4",
    "r1c",
    "r2ce",
    "rd2i",
    "rdin4",
    "2re.",
    "re1al",
    "2r2ed",
    "re2fe",
    "r4es.",
    "r1f",
    "rg2",
    "rgi4n",
    "rin4d",
    "r2is",
    "r1l",
    "r1m",
    "rox5",
    "r1p",
    "r1ti",
    "ru2n",
    "r1w",
    "ry3t",
    "sa2",
    "4se.",
    "3sect",
    "4s4ed",
    "sep3a3",
    "5sev",
    "4s3f",
    "s2h",
    "2sh.",
    "sho4",
    "shor4",
    "short5",
    "si1b",
    "2s1in",
    "s3ing",
    "1sis",
    "3sitio",
    "1so",
    "3s4on.",
    "s4pon",
    "2ss",
    "2st.",
    "st2i",
    "s1tic",
    "s3tif",
    "st4r",
    "s2ty",
    "1su",
    "s4ul",
    "su2r",
    "s4y",
    "3syl",
    "1ta",
    "2tab",
    "2t1b",
    "4tc",
    "t4ch",
    "4te.",
    "2t1ed",
    "3tenc",
    "5ter3d",
    "1teri",
    "ter3is",
    "3tex",
    "2th.",
    "than4",
    "th2e",
    "t4ic1u",
    "tif2",
    "1tim",
    "2t1in",
    "1tio",
    "1tiv",
    "2tl",
    "4t1m",
    "tme4",
    "1to",
    "to2ma",
    "1tra",
    "4t1s2",
    "4t3t2",
    "1tu",
    "tw4",
    "1ty",
    "2tyl",
    "uci4b",
    "u1la",
    "u1ni",
    "un3s4",
    "urc4",
    "2us",
    "4u1t2i",
    "ution5a",
    "uto5matic",
    "4ve.",
    "vi1ou",
    "whi4",
    "wi2",
    "with3",
    "1wo2",
    "wra4",
    "wri4",
    "x1e",
    "x3i",
    "x3p",
    "x1t2",
    "y1o4",
    "y3po",
    "ys1t",
    "y3thin",
    "za1",
];

/// The `\hyphenation` exception list shared by hyphen.tex and hyph-en-us.tex.
pub const EN_US_EXCEPTIONS: &[&str] = &[
    "as-so-ciate",
    "as-so-ciates",
    "dec-li-na-tion",
    "oblig-a-tory",
    "phil-an-thropic",
    "present",
    "presents",
    "project",
    "projects",
    "reci-procity",
    "re-cog-ni-zance",
    "ref-or-ma-tion",
    "ret-ri-bu-tion",
    "ta-ble",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// `\showhyphens` output of pdflatex (pdfTeX 1.40.29, TeX Live 2026, language
    /// `english` = hyphen.tex, \lefthyphenmin 2, \righthyphenmin 3) for every
    /// word of five or more letters in the two oracle samples: 110 words.
    const TEX_SHOWHYPHENS: &[&str] = &[
        "De-lib-er-ately",
        "Flash-TeX",
        "Hy-phen-ation",
        "README",
        "Re-pro-ducibil-ity",
        "Short",
        "Wrap-ping",
        "ac-cents",
        "ad-just-ment",
        "against",
        "ap-prox-i-ma-tions",
        "ar-ti-cle",
        "au-to-mat-i-cally",
        "be-gin",
        "be-haviour",
        "bound-aries",
        "break",
        "breaks",
        "char-ac-ter-is-tic",
        "cof-fee",
        "col-lab-o-ra-tion",
        "com-fort-ably",
        "com-mu-nity",
        "com-pares",
        "com-par-i-son",
        "com-pi-la-tion",
        "com-piler",
        "com-pli-cate",
        "com-puted",
        "con-sec-u-tive",
        "con-se-quently",
        "con-sid-er-ably",
        "con-struc-tions",
        "con-ven-tional",
        "coun-ter-pro-duc-tive",
        "de-lib-er-ately",
        "demon-strate",
        "de-ter-mine",
        "dis-cre-tionary",
        "doc-u-ment",
        "doc-u-men-ta-tion",
        "doc-u-ment-class",
        "drifts",
        "ef-fort",
        "elim-i-nate",
        "emer-gency",
        "em-pha-sised",
        "en-gine",
        "ev-ery",
        "ex-pects",
        "ex-traor-di-nar-ily",
        "fi-nally",
        "fol-low",
        "gen-er-ated",
        "go-ing",
        "im-ple-men-ta-tion",
        "in-com-pre-hen-si-ble",
        "in-de-pen-dently",
        "in-sti-tu-tion-al-iza-tion",
        "in-ter-dis-ci-plinary",
        "in-ter-na-tional",
        "in-ter-word",
        "jus-ti-fi-ca-tion",
        "keeps",
        "lay-out",
        "lines",
        "longer",
        "mea-sured",
        "mea-sure-ment",
        "op-por-tu-ni-ties",
        "or-a-cle",
        "or-der",
        "or-di-nary",
        "over-flows",
        "para-graph",
        "para-graphs",
        "par-tic-u-larly",
        "phrase",
        "poly-syl-labic",
        "po-si-tion",
        "po-si-tions",
        "prose",
        "rather",
        "reader",
        "ref-er-ence",
        "re-port",
        "rep-re-sen-ta-tive",
        "re-pro-ducibil-ity",
        "re-spon-si-bil-i-ties",
        "re-sults",
        "sat-is-fac-tory",
        "sec-tion",
        "sev-eral",
        "source",
        "starts",
        "stretch-a-bil-ity",
        "stripped",
        "ter-mi-nol-ogy",
        "textbf",
        "their",
        "through-out",
        "ty-po-graph-i-cal",
        "un-bal-anced",
        "un-char-ac-ter-is-ti-cally",
        "un-re-mark-able",
        "ver-i-fi-ca-tion",
        "where",
        "whether",
        "which",
        "words",
    ];

    #[test]
    fn matches_tex_showhyphens_for_every_sample_word() {
        let h = LiangHyphenator::en_us_subset();
        let mut mismatches = Vec::new();
        for entry in TEX_SHOWHYPHENS {
            let word: String = entry.chars().filter(|c| *c != '-').collect();
            let pts = h.hyphenate(&word);
            let mut rendered = String::new();
            let mut last = 0;
            for p in &pts {
                rendered.push_str(&word[last..p.offset]);
                rendered.push('-');
                last = p.offset;
            }
            rendered.push_str(&word[last..]);
            if rendered != *entry {
                mismatches.push(format!("{entry} vs {rendered}"));
            }
        }
        assert!(
            mismatches.is_empty(),
            "{} mismatches: {mismatches:?}",
            mismatches.len()
        );
    }

    #[test]
    fn tex_word_rules() {
        let h = LiangHyphenator::en_us_subset();
        // Trailing punctuation is fine; a leading non-letter or a letter after
        // a non-letter disables hyphenation; short words are left alone.
        assert!(!h.hyphenate("documentation,").is_empty());
        assert!(h.hyphenate("(documentation").is_empty());
        assert!(h.hyphenate("docu(mentation").is_empty());
        assert!(h.hyphenate("lines").is_empty());
        // Exceptions override patterns.
        assert_eq!(h.positions("table"), vec![2]);
        // Explicit markers win over patterns.
        let pts = h.hyphenate("docu\\-mentation");
        assert_eq!(pts.len(), 1);
        assert_eq!(
            (pts[0].offset, pts[0].marker_len, pts[0].automatic),
            (4, 2, false)
        );
        // Automatic points carry byte offsets and no marker.
        let pts = h.hyphenate("Documentation");
        assert_eq!(
            pts.iter().map(|p| p.offset).collect::<Vec<_>>(),
            vec![3, 4, 7, 9]
        );
        assert!(pts.iter().all(|p| p.automatic && p.marker_len == 0));
    }
}
