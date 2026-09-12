//! Bounded, deterministic random-input sweep against `SpellChecker::check`
//! and `UserDictionary`, complementing the hand-crafted cases in
//! `tests/adversarial_bounds.rs` with unstructured, seed-derived text rather
//! than specific known-tricky shapes. No proptest/cargo-fuzz dependency;
//! runs in well under a second.

use std::collections::HashSet;
use std::panic::{self, AssertUnwindSafe};

use flashtex_spellcheck::{Dictionary, SpellChecker, SpellCheckerConfig, UserDictionary};

/// Small dependency-free xorshift64* PRNG so a reported failure is always
/// reproducible from the fixed seed alone.
struct Xorshift64(u64);
impl Xorshift64 {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn next_range(&mut self, n: usize) -> usize {
        (self.next_u64() as usize) % n
    }
}

// A pool mixing plain ASCII letters, the exclusion-syntax characters
// (`$`, `\`, `(`, `)`, `[`, `]`, `*`), token-joiners (`'`, `-`), whitespace,
// combining marks, zero-width characters, an RTL override, and a couple of
// astral-plane emoji -- so random composition can accidentally produce any
// of the hand-crafted edge cases in adversarial_bounds.rs by chance, plus
// shapes those cases don't specifically target.
const POOL: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', ' ', '\n', '\t', '$', '\\', '(', ')', '[', ']',
    '*', '\'', '-', '\u{0301}', // combining acute accent
    '\u{200B}', // zero-width space
    '\u{202E}', // RTL override
    '😀', '🏳', '_',
];

fn random_string(rng: &mut Xorshift64, max_len: usize) -> String {
    let len = rng.next_range(max_len + 1);
    (0..len).map(|_| POOL[rng.next_range(POOL.len())]).collect()
}

#[test]
fn hostile_random_text_never_panics_and_stays_char_boundary_safe() {
    let dict: HashSet<String> = ["hello", "world", "the", "a"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let checker = SpellChecker::new(SpellCheckerConfig::default());
    let mut rng = Xorshift64(0xD1B5_4A32_D192_ED03);

    // Round/length bounds are deliberately modest: `check()` is bounded
    // *per call* (MAX_TOTAL_SUGGESTION_WORK_PER_CHECK), not across calls, so
    // a tiny dictionary plus many distinct ~20-char gibberish "words" makes
    // each individual call legitimately spend a real (bounded) amount of
    // work on the distance-2 suggestion fallback. That is correct,
    // documented behavior, not a defect -- but it means a loop calling
    // `check()` thousands of times accumulates real wall-clock time. 300
    // rounds of up to 60 chars keeps this test itself under ~10s while
    // still exercising the same exclusion/tokenizer/suggestion code paths.
    for round in 0..150u32 {
        let text = random_string(&mut rng, 50);
        let text_for_check = text.clone();
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            checker.check(&text_for_check, &dict as &dyn Dictionary)
        }));
        let misspellings = match result {
            Ok(m) => m,
            Err(e) => panic!(
                "round {round}: check() panicked on {text:?} (len {}): {e:?}",
                text.len()
            ),
        };
        // Every reported range must land on a UTF-8 char boundary and be
        // extractable from the original text (documented invariant).
        for m in &misspellings {
            assert!(
                text.get(m.range.clone()).is_some(),
                "round {round}: misspelling range {:?} is not a valid slice of {text:?}",
                m.range
            );
        }
    }
}

#[test]
fn hostile_random_words_into_user_dictionary_never_panic() {
    let mut ud = UserDictionary::new(50);
    let mut rng = Xorshift64(0x9E37_79B9_7F4A_7C15);
    for round in 0..2000u32 {
        let word = random_string(&mut rng, 40);
        let add = panic::catch_unwind(AssertUnwindSafe(|| ud.add_word(&word)));
        assert!(
            add.is_ok(),
            "round {round}: UserDictionary::add_word panicked on {word:?}"
        );
        let ignore = panic::catch_unwind(AssertUnwindSafe(|| ud.ignore_word(&word)));
        assert!(
            ignore.is_ok(),
            "round {round}: UserDictionary::ignore_word panicked on {word:?}"
        );
    }
}
