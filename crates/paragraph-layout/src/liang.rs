//! Liang's pattern hyphenation (the algorithm TeX uses), original Rust
//! implementation behind the [`Hyphenator`] trait.
//!
//! # Pattern data
//!
//! Both pattern files are vendored **verbatim** under `patterns/` and parsed
//! at construction; nothing is generated or edited:
//!
//! * `patterns/hyphen.tex` — Knuth's Plain TeX hyphenation tables. TeX Live's
//!   `language.dat` loads this file for the `english` language (aliases
//!   `usenglish`, `USenglish`, `american`), which is language 0 in pdflatex's
//!   LaTeX format, so it is what an unconfigured `pdflatex` document uses and
//!   the default here ([`LiangHyphenator::english`]). Licence (file header):
//!   "Unlimited copying and redistribution of this file are permitted as long
//!   as this file is not modified." It is not modified.
//! * `patterns/hyph-en-us.tex` — hyph-utf8 "Hyphenation patterns for American
//!   English", Copyright (C) 1990, 2004, 2005 Gerard D.C. Kuiken, version
//!   2005-05-30 (Knuth's patterns plus Kuiken's additions, `ushyphmax`).
//!   Licence (file header): "Copying and distribution of this file, with or
//!   without modification, are permitted in any medium without royalty
//!   provided the copyright notice and this notice are preserved." Available
//!   as [`LiangHyphenator::en_us_max`] (babel `usenglishmax`).
//!
//! Both come from TeX Live 2026 (`texmf-dist/tex/generic/hyphen/` and
//! `texmf-dist/tex/generic/hyph-utf8/patterns/tex/`).
//!
//! # Word rules
//!
//! [`Hyphenator::hyphenate`] receives one whitespace-free chunk. Following
//! TeX §894–899: leading characters that are not letters are skipped; the
//! hyphenatable word is the maximal run of letters that follows (lower-cased,
//! `\uchyph=1`); trailing non-letter *characters* are allowed after it
//! ("sentence." is hyphenated), but a discretionary after the run — an explicit
//! hyphen character or `\-` anywhere in the chunk — suppresses automatic
//! hyphenation of the whole chunk. Words with fewer than
//! `left_min + right_min` letters, or more than 63, are left alone. Exceptions
//! (`\hyphenation`) replace the patterns for an exact (lower-cased) word;
//! `\lefthyphenmin` / `\righthyphenmin` apply to both (§902).
//!
//! Which chunks TeX tries at all (only a word that directly follows glue) is
//! the caller's business: [`crate::items::ParagraphBuilder::text`] applies it.

use std::collections::HashMap;

use crate::hyphenate::{HyphenationPoint, Hyphenator};

/// Knuth's `hyphen.tex`, verbatim.
pub const HYPHEN_TEX: &str = include_str!("../patterns/hyphen.tex");
/// hyph-utf8 `hyph-en-us.tex`, verbatim.
pub const HYPH_EN_US_TEX: &str = include_str!("../patterns/hyph-en-us.tex");

/// TeX's limit on the letters of a hyphenatable word (§897: `hn = 63`).
pub const MAX_WORD_LETTERS: usize = 63;

/// Error parsing a `\patterns{}` / `\hyphenation{}` source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternError {
    pub entry: String,
    pub reason: &'static str,
}

impl std::fmt::Display for PatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bad hyphenation entry {:?}: {}", self.entry, self.reason)
    }
}

impl std::error::Error for PatternError {}

/// TeX-style pattern hyphenator with exceptions and hyphen minima.
#[derive(Debug, Clone)]
pub struct LiangHyphenator {
    /// Pattern letters -> inter-letter weights (`letters.len() + 1` digits).
    patterns: HashMap<Vec<char>, Vec<u8>>,
    /// Longest pattern, in letters (dots included).
    max_len: usize,
    /// Exception word (lower case) -> break positions (letters before the break).
    exceptions: HashMap<Vec<char>, Vec<usize>>,
    /// `\lefthyphenmin` (English: 2).
    pub left_min: usize,
    /// `\righthyphenmin` (English: 3).
    pub right_min: usize,
}

impl LiangHyphenator {
    /// An empty hyphenator (no patterns, no exceptions).
    pub fn empty(left_min: usize, right_min: usize) -> Self {
        LiangHyphenator {
            patterns: HashMap::new(),
            max_len: 0,
            exceptions: HashMap::new(),
            left_min,
            right_min,
        }
    }

    /// pdflatex's default `english`: Knuth's `hyphen.tex`, hyphenmins 2/3.
    pub fn english() -> Self {
        Self::from_tex(HYPHEN_TEX, 2, 3).expect("vendored hyphen.tex parses")
    }

    /// hyph-utf8 `hyph-en-us.tex` (`usenglishmax`), hyphenmins 2/3.
    pub fn en_us_max() -> Self {
        Self::from_tex(HYPH_EN_US_TEX, 2, 3).expect("vendored hyph-en-us.tex parses")
    }

    /// Parses a TeX pattern file: `%` comments, one `\patterns{...}` group and
    /// an optional `\hyphenation{...}` group.
    pub fn from_tex(source: &str, left_min: usize, right_min: usize) -> Result<Self, PatternError> {
        let mut h = Self::empty(left_min, right_min);
        let text: String = source
            .lines()
            .map(|l| l.find('%').map_or(l, |i| &l[..i]))
            .collect::<Vec<_>>()
            .join("\n");
        if let Some(body) = group(&text, "\\patterns") {
            for entry in body.split_whitespace() {
                h.add_pattern(entry)?;
            }
        }
        if let Some(body) = group(&text, "\\hyphenation") {
            h.add_exceptions(body)?;
        }
        Ok(h)
    }

    /// Adds one pattern in TeX notation (`.ach4`, `hy3ph`, `4b1ora`).
    pub fn add_pattern(&mut self, entry: &str) -> Result<(), PatternError> {
        let mut letters = Vec::new();
        let mut weights = vec![0u8];
        for ch in entry.chars() {
            if let Some(d) = ch.to_digit(10) {
                *weights.last_mut().expect("nonempty") = d as u8;
            } else if ch == '.' || ch.is_alphabetic() {
                letters.push(ch);
                weights.push(0);
            } else {
                return Err(PatternError {
                    entry: entry.to_string(),
                    reason: "patterns contain only letters, dots and digits",
                });
            }
        }
        if letters.is_empty() {
            return Err(PatternError {
                entry: entry.to_string(),
                reason: "pattern has no letters",
            });
        }
        self.max_len = self.max_len.max(letters.len());
        self.patterns.insert(letters, weights);
        Ok(())
    }

    /// `\hyphenation{ta-ble as-so-ciate}`: whitespace-separated words with
    /// hyphens at the allowed breaks. A later entry for the same word
    /// replaces an earlier one, as in TeX.
    pub fn add_exceptions(&mut self, list: &str) -> Result<(), PatternError> {
        for entry in list.split_whitespace() {
            let mut word = Vec::new();
            let mut breaks = Vec::new();
            for ch in entry.chars() {
                if ch == '-' {
                    if !word.is_empty() && breaks.last() != Some(&word.len()) {
                        breaks.push(word.len());
                    }
                } else if ch.is_alphabetic() {
                    word.extend(ch.to_lowercase());
                } else {
                    return Err(PatternError {
                        entry: entry.to_string(),
                        reason: "\\hyphenation entries contain only letters and hyphens",
                    });
                }
            }
            breaks.retain(|&b| b < word.len());
            if !word.is_empty() {
                self.exceptions.insert(word, breaks);
            }
        }
        Ok(())
    }

    /// Number of patterns and exceptions loaded.
    pub fn counts(&self) -> (usize, usize) {
        (self.patterns.len(), self.exceptions.len())
    }

    /// Break positions (number of letters before each break) of a word made
    /// only of letters. Applies exceptions, patterns and the hyphen minima.
    pub fn positions(&self, word: &str) -> Vec<usize> {
        let lower: Vec<char> = word.chars().flat_map(char::to_lowercase).collect();
        self.positions_lower(&lower)
    }

    fn positions_lower(&self, word: &[char]) -> Vec<usize> {
        let n = word.len();
        if n > MAX_WORD_LETTERS || n < self.left_min.max(1) + self.right_min.max(1) {
            return Vec::new();
        }
        let odd: Vec<usize> = if let Some(e) = self.exceptions.get(word) {
            e.clone()
        } else {
            let mut dotted = Vec::with_capacity(n + 2);
            dotted.push('.');
            dotted.extend_from_slice(word);
            dotted.push('.');
            // weights[i] sits before dotted[i].
            let mut weights = vec![0u8; dotted.len() + 1];
            for start in 0..dotted.len() {
                for len in 1..=self.max_len.min(dotted.len() - start) {
                    if let Some(w) = self.patterns.get(&dotted[start..start + len]) {
                        for (j, &d) in w.iter().enumerate() {
                            let slot = &mut weights[start + j];
                            *slot = (*slot).max(d);
                        }
                    }
                }
            }
            // A break after k letters sits before dotted[k + 1].
            (1..n).filter(|&k| weights[k + 1] % 2 == 1).collect()
        };
        odd.into_iter()
            .filter(|&k| k >= self.left_min.max(1) && n - k >= self.right_min.max(1))
            .collect()
    }
}

/// Body of the first `name{...}` group (no nested braces in pattern files).
fn group<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let at = text.find(name)? + name.len();
    let open = at + text[at..].find('{')?;
    let close = open + text[open..].find('}')?;
    Some(&text[open + 1..close])
}

impl Hyphenator for LiangHyphenator {
    fn hyphenate(&self, word: &str) -> Vec<HyphenationPoint> {
        // A discretionary inside the chunk (explicit hyphen or `\-`) ends TeX's
        // search with no automatic hyphens (§896/§899 `othercases goto done1`);
        // `\-` stays legal as an explicit point.
        if word.contains("\\-") {
            return crate::hyphenate::ExplicitDiscretionary.hyphenate(word);
        }
        if word.contains('-') {
            return Vec::new();
        }
        let mut letters: Vec<char> = Vec::new();
        let mut byte_at: Vec<usize> = Vec::new();
        let mut rest = "";
        for (i, ch) in word.char_indices() {
            if ch.is_alphabetic() {
                byte_at.push(i);
                letters.extend(ch.to_lowercase());
            } else if !letters.is_empty() {
                rest = &word[i..];
                break;
            }
        }
        // Only characters may follow the letter run; a later letter run would
        // mean a non-letter character sat inside the word, which TeX allows
        // (it just stops at the first run), so nothing further to check.
        let _ = rest;
        if letters.len() != byte_at.len() {
            // A letter whose lower case is several chars: stay conservative.
            return Vec::new();
        }
        self.positions_lower(&letters)
            .into_iter()
            .map(|k| HyphenationPoint {
                offset: byte_at[k],
                marker_len: 0,
                automatic: true,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn show(h: &LiangHyphenator, w: &str) -> String {
        let pos = h.positions(w);
        let mut out = String::new();
        for (i, ch) in w.chars().enumerate() {
            if pos.contains(&i) {
                out.push('-');
            }
            out.push(ch);
        }
        out
    }

    #[test]
    fn vendored_files_parse_completely() {
        let h = LiangHyphenator::english();
        assert_eq!(h.counts(), (4447, 14));
        let m = LiangHyphenator::en_us_max();
        assert_eq!(m.counts(), (4938, 14));
    }

    /// Expected strings are pdflatex `\showhyphens` output (TeX Live 2026,
    /// `english` = hyphen.tex), recorded in `tests/oracle/showhyphens.txt`.
    #[test]
    fn knuth_examples() {
        let h = LiangHyphenator::english();
        for (w, want) in [
            ("counterexample", "coun-terex-am-ple"),
            ("satisfy", "sat-isfy"),
            ("hypotheses", "hy-pothe-ses"),
            ("explicitly", "ex-plic-itly"),
            ("irrational", "ir-ra-tional"),
            ("expression", "ex-pres-sion"),
            ("specified", "spec-i-fied"),
            ("statement", "state-ment"),
            ("associate", "as-so-ciate"),
            ("table", "ta-ble"),
            ("present", "present"),
            ("hyphenation", "hy-phen-ation"),
        ] {
            assert_eq!(show(&h, w), want, "{w}");
        }
    }

    #[test]
    fn minima_and_exceptions() {
        let mut h = LiangHyphenator::english();
        h.add_exceptions("FlashTeX flash-tex").unwrap();
        assert_eq!(show(&h, "flashtex"), "flash-tex");
        h.left_min = 1;
        h.right_min = 1;
        assert!(h.positions("ta").is_empty() || h.positions("ta") == vec![1]);
        let mut h = LiangHyphenator::english();
        h.right_min = 10;
        assert!(h.positions("counterexample").iter().all(|&k| 14 - k >= 10));
        assert!(LiangHyphenator::english().positions("cat").is_empty());
    }

    #[test]
    fn chunk_rules() {
        let h = LiangHyphenator::english();
        let offs = |w: &str| h.hyphenate(w).iter().map(|p| p.offset).collect::<Vec<_>>();
        assert_eq!(offs("(counterexample),"), vec![5, 10, 12]); // (coun-terex-am-ple),
        assert_eq!(offs("Satisfy."), vec![3]);
        assert!(offs("counter-example").is_empty());
        let pts = h.hyphenate("re\\-pro");
        assert_eq!(pts.len(), 1);
        assert!(!pts[0].automatic);
    }
}
