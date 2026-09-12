//! Word counting, and its limits.
//!
//! Counting "words" is inherently lexical and language-dependent: there is
//! no script-neutral, universally correct answer. Rather than pretend
//! otherwise, this crate commits to ONE explicit rule:
//!
//! 1. Text is split into tokens on Unicode whitespace ([`char::is_whitespace`]).
//! 2. A token counts as one word if it contains at least one Unicode
//!    alphanumeric character ([`char::is_alphanumeric`]). Tokens made
//!    entirely of punctuation, symbols, combining marks, or emoji (e.g.
//!    `"--"`, `"..."`, a lone `"🎉"`) do not count.
//! 3. A token is never split further, no matter what it contains. Internal
//!    hyphens (`"well-known"`, `"state-of-the-art"`) and apostrophes
//!    (`"don't"`, `"O'Brien"`) keep the whole token as ONE word. This crate
//!    does not attempt hyphenation-point detection or contraction-aware
//!    splitting; both are language-specific problems this crate is
//!    deliberately not solving.
//!
//! Numerals count as words under rule 2 (`"42"` is one word), matching most
//! everyday word counters.
//!
//! ## Known limitation: scripts without whitespace
//!
//! Rule 1 makes `words` a poor measure for Chinese, Japanese, and Korean
//! running text, which is not space-delimited: a whole CJK sentence with no
//! spaces is a single token, and therefore counts as exactly ONE word,
//! however many words a native reader would count. This crate does **not**
//! implement CJK word segmentation — that needs a dictionary or a trained
//! segmenter, well outside "bounded statistics from explicit items" — so
//! `words` should not be trusted for CJK-heavy text. [`WordStats::chars`]
//! (a count of non-whitespace Unicode scalar values) is offered as a
//! script-agnostic size proxy for callers who need *something* comparable
//! across scripts; it is not a word count either, just a different, more
//! honest measurement.

/// Word and character counts for one run of rendered prose text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WordStats {
    /// Words, per the rule documented on this module.
    pub words: usize,
    /// Non-whitespace Unicode scalar values. A script-agnostic size proxy —
    /// see the module docs for why `words` alone is not enough for CJK text.
    pub chars: usize,
}

impl WordStats {
    /// Count words and non-whitespace characters in `text`, per the rule
    /// documented on this module.
    pub fn of(text: &str) -> WordStats {
        let mut words = 0usize;
        let mut chars = 0usize;
        for token in text.split_whitespace() {
            if token.chars().any(char::is_alphanumeric) {
                words += 1;
            }
            chars += token.chars().count();
        }
        WordStats { words, chars }
    }
}

impl std::ops::Add for WordStats {
    type Output = WordStats;
    fn add(self, rhs: WordStats) -> WordStats {
        WordStats {
            words: self.words + rhs.words,
            chars: self.chars + rhs.chars,
        }
    }
}

impl std::iter::Sum for WordStats {
    fn sum<I: Iterator<Item = WordStats>>(iter: I) -> WordStats {
        iter.fold(WordStats::default(), std::ops::Add::add)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_worked_sentence() {
        let s = WordStats::of("The quick brown fox jumps over the lazy dog.");
        assert_eq!(s.words, 9);
    }

    #[test]
    fn empty_and_whitespace_only() {
        assert_eq!(WordStats::of(""), WordStats { words: 0, chars: 0 });
        assert_eq!(
            WordStats::of("   \n\t\u{00A0} "),
            WordStats { words: 0, chars: 0 }
        );
    }

    #[test]
    fn hyphen_and_apostrophe_stay_one_word() {
        assert_eq!(WordStats::of("well-known").words, 1);
        assert_eq!(WordStats::of("don't").words, 1);
        assert_eq!(WordStats::of("O'Brien").words, 1);
        assert_eq!(WordStats::of("state-of-the-art").words, 1);
    }

    #[test]
    fn punctuation_only_tokens_are_not_words() {
        let s = WordStats::of("-- ... !!! ---");
        assert_eq!(s.words, 0);
        assert_eq!(s.chars, "--...!!!---".chars().count());
    }

    #[test]
    fn digits_count_as_words() {
        assert_eq!(WordStats::of("42 apples and 7 oranges").words, 5);
    }

    #[test]
    fn cjk_text_undercounts_by_design() {
        // "Hello, world" in Chinese, four characters, no whitespace at all.
        let s = WordStats::of("你好世界");
        // Documented limitation: one whitespace-delimited token -> one word.
        assert_eq!(s.words, 1);
        // The character-count proxy is honest about the actual size.
        assert_eq!(s.chars, 4);
    }

    #[test]
    fn mixed_script_token_without_space_is_one_word() {
        let s = WordStats::of("hello你好 world");
        assert_eq!(s.words, 2);
        assert_eq!(s.chars, 7 + 5); // "hello你好" (7 scalars) + "world" (5)
    }

    #[test]
    fn combining_mark_does_not_split_or_hide_a_word() {
        // 'e' + COMBINING ACUTE ACCENT (U+0301): two scalar values, one token.
        let s = WordStats::of("e\u{0301}");
        assert_eq!(s.words, 1);
        assert_eq!(s.chars, 2);
    }

    #[test]
    fn lone_emoji_token_is_not_a_word() {
        let s = WordStats::of("🎉 party");
        assert_eq!(s.words, 1);
        assert_eq!(s.chars, 1 + 5);
    }

    #[test]
    fn zero_width_joiner_does_not_split_the_token() {
        // U+200D ZERO WIDTH JOINER is not Unicode whitespace, so "a\u{200D}b"
        // is one token and counts as one word, not a separator.
        let s = WordStats::of("a\u{200D}b");
        assert_eq!(s.words, 1);
        assert_eq!(s.chars, 3);
    }

    #[test]
    fn sum_over_multiple_texts() {
        let total: WordStats = ["one two", "three", ""]
            .iter()
            .map(|t| WordStats::of(t))
            .sum();
        assert_eq!(total.words, 3);
        assert_eq!(total.chars, "onetwothree".chars().count());
    }
}
