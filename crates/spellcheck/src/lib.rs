//! `flashtex-spellcheck`: offline, bounded, source-aware spelling suggestions.
//!
//! This crate performs **zero network access** and ships **zero bundled
//! dictionary**. Callers supply their own word list via the [`Dictionary`]
//! trait (a `HashSet<String>` works out of the box). The checker:
//!
//! - Skips text inside math (`$...$`, `$$...$$`, `\(...\)`, `\[...\]`) and
//!   inside LaTeX command names (`\foo`), so those are never flagged or
//!   "corrected".
//! - Never rewrites the input: it only reports byte ranges and suggestions.
//!   The caller's source text and revisions are left exactly as given.
//! - Bounds suggestion generation so a long input word, checked against an
//!   arbitrarily large caller dictionary, cannot blow up. See
//!   [`SpellCheckerConfig`] for the exact bounds and why dictionary size does
//!   not matter to the cost.
//! - Reports every range as a byte range that lands on a UTF-8 character
//!   boundary, so multi-byte characters are never split.
//! - Lets callers layer a bounded, in-memory [`UserDictionary`] (additions
//!   and ignores) on top of their own dictionary via [`LayeredDictionary`],
//!   without this crate ever loading or persisting one itself.
//! - Lets callers tag a check with a caller-defined [`Revision`] via
//!   [`SpellChecker::check_revision`], so a previously computed
//!   [`CheckResult`] can be detected as stale ([`CheckResult::is_stale`],
//!   or [`CheckResult::is_stale_for`] when the current text is on hand too,
//!   which also catches a missed revision bump) instead of being silently
//!   reused after the source moved on.
//! - Lets callers interrupt a long check via [`SpellChecker::check_cancellable`],
//!   which polls a caller-supplied callback and returns
//!   [`CheckOutcome::Cancelled`] promptly instead of running to completion.
//!   No threads are spawned; the callback runs synchronously on the calling
//!   thread.
//!
//! None of this crate's checking APIs ever modify the input text: they only
//! read `text` and report byte ranges and suggestion strings into/about it.
//! There is no "apply correction" API and there never will be one.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Range;

/// A caller-supplied source of known-correct words.
///
/// This crate ships no dictionary and performs no I/O or network access;
/// callers construct a `Dictionary` implementation from whatever word list
/// they already have loaded in memory (a file they read themselves, an
/// embedded resource, etc).
///
/// Implementations should make `contains` an effectively O(1) lookup (e.g. a
/// hash set). The bounds documented on [`SpellCheckerConfig`] assume this;
/// they do not protect against a `Dictionary` whose own `contains` is slow.
pub trait Dictionary {
    /// Returns true if `word` is a known-correct word.
    fn contains(&self, word: &str) -> bool;
}

impl Dictionary for HashSet<String> {
    fn contains(&self, word: &str) -> bool {
        HashSet::contains(self, word)
    }
}

impl Dictionary for HashSet<&str> {
    fn contains(&self, word: &str) -> bool {
        HashSet::contains(self, word)
    }
}

impl Dictionary for BTreeSet<String> {
    fn contains(&self, word: &str) -> bool {
        BTreeSet::contains(self, word)
    }
}

impl Dictionary for BTreeSet<&str> {
    fn contains(&self, word: &str) -> bool {
        BTreeSet::contains(self, word)
    }
}

/// Bounds on suggestion generation.
///
/// Suggestion cost is a function of the *misspelled word's length* and these
/// bounds, not of the dictionary's size: candidates are generated from the
/// word itself (insertions/deletions/substitutions/transpositions) and each
/// candidate is checked against the dictionary with a single `contains`
/// call, rather than scanning the dictionary and diffing every entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellCheckerConfig {
    /// Maximum edit distance considered when generating candidate
    /// corrections. Clamped internally to
    /// [`SpellChecker::MAX_ALLOWED_EDIT_DISTANCE`] because the number of raw
    /// candidates grows combinatorially with this value.
    pub max_edit_distance: usize,
    /// Maximum number of suggestions returned per misspelled word.
    pub max_suggestions: usize,
    /// Words with more than this many characters are still reported as
    /// misspelled, but no suggestions are generated for them. Candidate
    /// generation cost grows with word length, so this bounds worst-case
    /// cost for pathological (e.g. malformed or adversarial) input
    /// regardless of `max_edit_distance` or dictionary size.
    pub max_word_length_for_suggestions: usize,
}

impl Default for SpellCheckerConfig {
    fn default() -> Self {
        Self {
            max_edit_distance: 2,
            max_suggestions: 5,
            max_word_length_for_suggestions: 20,
        }
    }
}

/// One misspelled word found in a `check()` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Misspelling {
    /// The exact word as it appeared in the source text (original case,
    /// original bytes).
    pub word: String,
    /// The byte range of `word` in the text passed to `check`. Always lands
    /// on UTF-8 character boundaries.
    pub range: Range<usize>,
    /// Bounded, ranked list of suggested corrections (closest edit distance
    /// first, then lexicographic). May be empty if none were found within
    /// the configured bounds, including when the word exceeded
    /// `max_word_length_for_suggestions`.
    pub suggestions: Vec<String>,
}

/// Offline, bounded spell checker over caller-supplied text and dictionary.
pub struct SpellChecker {
    config: SpellCheckerConfig,
}

/// Hard ceiling on the number of raw (pre-dictionary-filter) candidate
/// strings a single suggestion request may generate. This is defense in
/// depth: `max_word_length_for_suggestions` already keeps generation cheap,
/// but this guarantees a fixed ceiling on work performed no matter what a
/// caller configures.
const MAX_RAW_CANDIDATES: usize = 50_000;

/// Hard ceiling on the total `edits1`-generation work (measured in the same
/// raw-candidate-count currency as `MAX_RAW_CANDIDATES`) a single
/// `check`/`check_revision`/`check_cancellable` call may spend across ALL
/// distinct misspelled words combined, not just one. `MAX_RAW_CANDIDATES`
/// and `max_word_length_for_suggestions` already bound the cost of any one
/// word's suggestion search, but neither bounds the *aggregate* cost of a
/// document containing many distinct words that never match the dictionary
/// (e.g. prose checked against an empty or mismatched-language dictionary,
/// or plain identifiers/jargon) -- exactly the fully-untrusted-input case
/// this crate must stay bounded against, since suggestion cost is paid once
/// per *distinct* word (`suggestion_cache` in `check_impl`) and distinct
/// word count is otherwise unbounded. Once this budget is spent, later
/// distinct words in the same call are still correctly flagged as
/// misspelled; they simply stop receiving generated suggestions, which is
/// already documented, caller-visible behavior (see
/// [`Misspelling::suggestions`]).
const MAX_TOTAL_SUGGESTION_WORK_PER_CHECK: usize = 300_000;

/// Upper bound on the number of raw candidates `edits1` produces for a word
/// of `n` chars: deletions(n) + transpositions(n-1) + substitutions(n *
/// alphabet) + insertions((n+1) * alphabet). Used to charge
/// `MAX_TOTAL_SUGGESTION_WORK_PER_CHECK` *before* paying the cost of
/// actually generating those candidates.
fn edits1_cost_upper_bound(n: usize) -> usize {
    let alphabet = EDIT_ALPHABET.len();
    n + n.saturating_sub(1) + n * alphabet + (n + 1) * alphabet
}

/// Alphabet used for insertion/substitution edits. Bounded to ASCII
/// lowercase letters: this is a deliberate, documented limitation, not an
/// oversight. Growing it to full Unicode would make candidate generation
/// unbounded in the size of the target alphabet, defeating the point of a
/// bounded checker. Callers whose dictionaries are not Latin-alphabet based
/// will still get correct misspelling *detection* and exclusion behavior;
/// they just will not get generated suggestions from this alphabet.
const EDIT_ALPHABET: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

impl SpellChecker {
    /// Hard ceiling on edit distance regardless of configuration. Raw
    /// candidate count for distance `d` grows roughly as
    /// `(alphabet_len * word_len)^d`, so distances beyond 2 are rejected as
    /// unbounded for realistic word lengths.
    pub const MAX_ALLOWED_EDIT_DISTANCE: usize = 2;

    /// Hard ceiling on `max_word_length_for_suggestions` regardless of
    /// configuration. `edits1` generates `O(alphabet_len * word_len)` raw
    /// candidates and each is itself `O(word_len)` to build, so a single
    /// suggestion search costs `O(alphabet_len * word_len^2)`: quadratic in
    /// word length. Without this ceiling, a caller configuring
    /// `max_word_length_for_suggestions` to a large value (or `usize::MAX`)
    /// combined with one long misspelled word in the input text turns that
    /// quadratic cost loose -- symmetric with `MAX_ALLOWED_EDIT_DISTANCE`
    /// above, which bounds the other input to the same cost model.
    pub const MAX_ALLOWED_WORD_LENGTH_FOR_SUGGESTIONS: usize = 64;

    /// Creates a checker with the given bounds. `config.max_edit_distance`
    /// is silently clamped to [`Self::MAX_ALLOWED_EDIT_DISTANCE`] and
    /// `config.max_word_length_for_suggestions` is silently clamped to
    /// [`Self::MAX_ALLOWED_WORD_LENGTH_FOR_SUGGESTIONS`].
    pub fn new(mut config: SpellCheckerConfig) -> Self {
        if config.max_edit_distance > Self::MAX_ALLOWED_EDIT_DISTANCE {
            config.max_edit_distance = Self::MAX_ALLOWED_EDIT_DISTANCE;
        }
        if config.max_word_length_for_suggestions > Self::MAX_ALLOWED_WORD_LENGTH_FOR_SUGGESTIONS {
            config.max_word_length_for_suggestions = Self::MAX_ALLOWED_WORD_LENGTH_FOR_SUGGESTIONS;
        }
        Self { config }
    }

    /// The effective (post-clamp) configuration in use.
    pub fn config(&self) -> &SpellCheckerConfig {
        &self.config
    }

    /// Checks `text` against `dictionary`, returning misspellings in
    /// left-to-right order. Text inside math (`$...$`, `$$...$$`,
    /// `\(...\)`, `\[...\]`) and inside LaTeX command names (`\foo`) is
    /// excluded and never reported. `text` is never modified; exact source
    /// bytes and revisions are preserved by construction, since this method
    /// only ever reads `text` and returns byte ranges into it.
    pub fn check(&self, text: &str, dictionary: &dyn Dictionary) -> Vec<Misspelling> {
        // `&|| false` never cancels, so this always returns `Some`.
        self.check_impl(text, dictionary, &|| false)
            .unwrap_or_default()
    }

    /// Same as [`Self::check`], but tags the result with `revision` (a
    /// caller-defined identifier for the exact source text this call
    /// checked). Use [`CheckResult::is_stale`] later to detect that the
    /// source has since moved on, instead of assuming a previously
    /// computed result still applies.
    pub fn check_revision(
        &self,
        text: &str,
        revision: Revision,
        dictionary: &dyn Dictionary,
    ) -> CheckResult {
        CheckResult {
            revision,
            text_fingerprint: fingerprint(text),
            misspellings: self.check(text, dictionary),
        }
    }

    /// Same as [`Self::check_revision`], but polls `is_cancelled` before
    /// starting and again before processing each candidate word (the point
    /// at which the potentially expensive suggestion search would run),
    /// returning [`CheckOutcome::Cancelled`] the first time it reports
    /// `true` instead of running to completion.
    ///
    /// This requires no threads or async runtime: `is_cancelled` is called
    /// synchronously on the calling thread, so it can be backed by whatever
    /// cancellation primitive the caller already has (an `AtomicBool` flag,
    /// a deadline check, a channel poll, ...).
    pub fn check_cancellable(
        &self,
        text: &str,
        revision: Revision,
        dictionary: &dyn Dictionary,
        is_cancelled: &dyn Fn() -> bool,
    ) -> CheckOutcome {
        match self.check_impl(text, dictionary, is_cancelled) {
            Some(misspellings) => CheckOutcome::Completed(CheckResult {
                revision,
                text_fingerprint: fingerprint(text),
                misspellings,
            }),
            None => CheckOutcome::Cancelled { revision },
        }
    }

    /// Shared implementation behind [`Self::check`] and
    /// [`Self::check_cancellable`]. Returns `None` the moment `is_cancelled`
    /// reports `true`, otherwise `Some` with the complete misspelling list.
    fn check_impl(
        &self,
        text: &str,
        dictionary: &dyn Dictionary,
        is_cancelled: &dyn Fn() -> bool,
    ) -> Option<Vec<Misspelling>> {
        if is_cancelled() {
            return None;
        }
        let excluded = excluded_ranges(text);
        let mut out = Vec::new();
        // Both `excluded` (built by one forward scan in `excluded_ranges`)
        // and the tokens from `tokenize_words` (also one forward scan) are
        // produced in strictly increasing, non-overlapping order. That lets
        // `excl_idx` advance monotonically across the whole loop instead of
        // rescanning `excluded` from the start for every token: without
        // this, adversarial input with many thousands of math/command
        // toggles (e.g. deeply nested or repeated unbalanced delimiters)
        // made this loop O(tokens * excluded) -- a genuine quadratic
        // blowup on exactly that kind of input -- instead of the
        // O(tokens + excluded) this two-pointer merge gives.
        let mut excl_idx = 0usize;
        // Suggestions are a pure function of (word, dictionary), and a
        // large document can repeat the same misspelling many times (e.g.
        // one typo throughout a megabyte-scale file). Caching by word
        // avoids redoing the bounded-but-nontrivial edit-distance search
        // once per occurrence -- once per *distinct* misspelled word
        // instead.
        let mut suggestion_cache: HashMap<&str, Vec<String>> = HashMap::new();
        // Shared across every distinct word processed by this call (see
        // `MAX_TOTAL_SUGGESTION_WORK_PER_CHECK`): bounds aggregate
        // suggestion-generation cost for the whole document, not just per
        // word.
        let mut suggestion_budget = MAX_TOTAL_SUGGESTION_WORK_PER_CHECK;
        for (range, word) in tokenize_words(text) {
            if is_cancelled() {
                return None;
            }
            while excl_idx < excluded.len() && excluded[excl_idx].end <= range.start {
                excl_idx += 1;
            }
            let excluded_here = excl_idx < excluded.len() && excluded[excl_idx].start < range.end;
            if excluded_here {
                continue;
            }
            if dictionary.contains(word) || dictionary.contains(&word.to_lowercase()) {
                continue;
            }
            let suggestions = if word.chars().count() > self.config.max_word_length_for_suggestions
            {
                Vec::new()
            } else if let Some(cached) = suggestion_cache.get(word) {
                cached.clone()
            } else {
                let computed = self.suggest(word, dictionary, &mut suggestion_budget);
                suggestion_cache.insert(word, computed.clone());
                computed
            };
            out.push(Misspelling {
                word: word.to_string(),
                range,
                suggestions,
            });
        }
        Some(out)
    }

    fn suggest(&self, word: &str, dictionary: &dyn Dictionary, budget: &mut usize) -> Vec<String> {
        let lower = word.to_lowercase();
        let max_distance = self.config.max_edit_distance;

        let mut ranked: Vec<(u8, String)> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        // Charge the whole-document budget for the first-pass `edits1`
        // call *before* paying for it: once the aggregate budget for this
        // call is spent, later distinct words simply stop getting
        // suggestions (still correctly flagged as misspelled) rather than
        // letting cost grow without bound in the number of distinct words.
        let first_pass_cost = edits1_cost_upper_bound(lower.chars().count());
        if first_pass_cost > *budget {
            *budget = 0;
            return Vec::new();
        }
        *budget -= first_pass_cost;

        let edit1 = edits1(&lower);
        for cand in &edit1 {
            if dictionary.contains(cand) && seen.insert(cand.clone()) {
                ranked.push((1, cand.clone()));
            }
        }

        // Only fall back to distance 2 when distance 1 found nothing: this
        // keeps the common case cheap and matches the classic bounded
        // spelling-corrector approach (Norvig-style), where the expensive
        // second pass only runs when it can actually add value.
        if max_distance >= 2 && ranked.is_empty() {
            let mut generated: usize = 0;
            'outer: for e1 in &edit1 {
                let per_candidate_cost = edits1_cost_upper_bound(e1.chars().count());
                if per_candidate_cost > *budget {
                    break;
                }
                *budget -= per_candidate_cost;
                for cand in edits1(e1) {
                    generated += 1;
                    if generated > MAX_RAW_CANDIDATES {
                        break 'outer;
                    }
                    if seen.contains(&cand) {
                        continue;
                    }
                    if dictionary.contains(&cand) {
                        seen.insert(cand.clone());
                        ranked.push((2, cand));
                    }
                }
            }
        }

        ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        ranked.truncate(self.config.max_suggestions);
        ranked.into_iter().map(|(_, w)| w).collect()
    }
}

impl Default for SpellChecker {
    fn default() -> Self {
        Self::new(SpellCheckerConfig::default())
    }
}

/// Generates every string within edit distance 1 of `word` under a bounded
/// ASCII-lowercase alphabet: deletions, transpositions, substitutions, and
/// insertions. Cost is `O(alphabet_len * word_len)`, independent of any
/// dictionary.
fn edits1(word: &str) -> HashSet<String> {
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    let mut out = HashSet::new();

    // Deletions.
    for i in 0..n {
        let s: String = chars
            .iter()
            .enumerate()
            .filter(|&(j, _)| j != i)
            .map(|(_, c)| *c)
            .collect();
        out.insert(s);
    }

    // Adjacent transpositions.
    for i in 0..n.saturating_sub(1) {
        let mut v = chars.clone();
        v.swap(i, i + 1);
        out.insert(v.into_iter().collect());
    }

    // Substitutions.
    for (i, &orig) in chars.iter().enumerate() {
        for &c in EDIT_ALPHABET {
            if orig == c {
                continue;
            }
            let mut v = chars.clone();
            v[i] = c;
            out.insert(v.into_iter().collect());
        }
    }

    // Insertions (including at the end, hence `0..=n`).
    for i in 0..=n {
        for &c in EDIT_ALPHABET {
            let mut v = chars.clone();
            v.insert(i, c);
            out.insert(v.into_iter().collect());
        }
    }

    out
}

/// Splits `text` into word tokens: maximal runs of Unicode alphabetic
/// characters, allowing an internal apostrophe or hyphen when immediately
/// followed by another alphabetic character (so `don't` and `well-known`
/// are single tokens). Byte ranges always fall on UTF-8 character
/// boundaries because they are derived exclusively from `char_indices`.
fn tokenize_words(text: &str) -> Vec<(Range<usize>, &str)> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    let text_len = text.len();
    let mut it = text.char_indices().peekable();

    while let Some(&(idx, ch)) = it.peek() {
        if ch.is_alphabetic() {
            if start.is_none() {
                start = Some(idx);
            }
            it.next();
            continue;
        }
        if (ch == '\'' || ch == '-') && start.is_some() {
            let mut lookahead = it.clone();
            lookahead.next();
            let continues = matches!(lookahead.peek(), Some((_, c)) if c.is_alphabetic());
            if continues {
                it.next();
                continue;
            }
        }
        if let Some(s) = start.take() {
            out.push((s..idx, &text[s..idx]));
        }
        it.next();
    }
    if let Some(s) = start.take() {
        out.push((s..text_len, &text[s..text_len]));
    }
    out
}

/// Which delimiter kind currently opens a math span.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MathKind {
    Dollar,
    DoubleDollar,
    Paren,
    Bracket,
}

/// Computes the byte ranges of `text` that must be excluded from spell
/// checking: math spans (`$...$`, `$$...$$`, `\(...\)`, `\[...\]`) and
/// LaTeX command names (`\foo`, `\foo*`). Control symbols (`\$`, `\%`,
/// `\\`, ...) are also excluded, incidentally, since they carry no
/// checkable word.
///
/// Malformed input (an opening delimiter with no matching close, a
/// trailing lone backslash) degrades gracefully: the exclusion simply runs
/// to the end of the text instead of panicking or looping.
///
/// All returned ranges are computed from `char_indices` byte offsets only,
/// so they always land on UTF-8 character boundaries.
fn excluded_ranges(text: &str) -> Vec<Range<usize>> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let n = chars.len();
    let text_len = text.len();
    let mut ranges: Vec<Range<usize>> = Vec::new();
    let mut math: Option<MathKind> = None;
    let mut excl_start: Option<usize> = None;
    let mut i = 0usize;

    while i < n {
        let (byte, ch) = chars[i];
        let next_ch = chars.get(i + 1).map(|&(_, c)| c);

        if let Some(kind) = math {
            let closes = match kind {
                MathKind::Dollar => ch == '$' && next_ch != Some('$'),
                MathKind::DoubleDollar => ch == '$' && next_ch == Some('$'),
                MathKind::Paren => ch == '\\' && next_ch == Some(')'),
                MathKind::Bracket => ch == '\\' && next_ch == Some(']'),
            };
            if excl_start.is_none() {
                excl_start = Some(byte);
            }
            if closes {
                let consumed = if kind == MathKind::DoubleDollar || ch == '\\' {
                    2
                } else {
                    1
                };
                let end_byte = chars.get(i + consumed).map(|&(b, _)| b).unwrap_or(text_len);
                if let Some(s) = excl_start.take() {
                    ranges.push(s..end_byte);
                }
                math = None;
                i += consumed;
            } else {
                i += 1;
            }
            continue;
        }

        if ch == '\\' {
            match next_ch {
                Some('(') => {
                    excl_start = Some(byte);
                    math = Some(MathKind::Paren);
                    i += 2;
                }
                Some('[') => {
                    excl_start = Some(byte);
                    math = Some(MathKind::Bracket);
                    i += 2;
                }
                Some(c) if c.is_ascii_alphabetic() => {
                    let mut j = i + 1;
                    while j < n && chars[j].1.is_ascii_alphabetic() {
                        j += 1;
                    }
                    if j < n && chars[j].1 == '*' {
                        j += 1;
                    }
                    let end = chars.get(j).map(|&(b, _)| b).unwrap_or(text_len);
                    ranges.push(byte..end);
                    i = j;
                }
                Some(_) => {
                    // Control symbol / escaped special character, e.g. `\$`, `\%`, `\\`.
                    let end = chars.get(i + 2).map(|&(b, _)| b).unwrap_or(text_len);
                    ranges.push(byte..end);
                    i += 2;
                }
                None => {
                    // Trailing lone backslash at end of input.
                    ranges.push(byte..text_len);
                    i += 1;
                }
            }
            continue;
        }

        if ch == '$' {
            excl_start = Some(byte);
            if next_ch == Some('$') {
                math = Some(MathKind::DoubleDollar);
                i += 2;
            } else {
                math = Some(MathKind::Dollar);
                i += 1;
            }
            continue;
        }

        i += 1;
    }

    if let Some(s) = excl_start.take() {
        ranges.push(s..text_len);
    }

    ranges
}

// ---- Source-revision awareness ------------------------------------------

/// Opaque, caller-defined identifier for a specific version of the source
/// text (e.g. an incrementing edit counter, or a content hash truncated to
/// 64 bits). This crate never interprets it beyond equality comparison; it
/// exists purely so a computed result can later be compared against the
/// source's *current* revision to detect staleness.
pub type Revision = u64;

/// The result of a revision-aware check: the misspellings found, tagged
/// with the exact revision they were computed against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    /// The revision passed to [`SpellChecker::check_revision`] or
    /// [`SpellChecker::check_cancellable`] that produced this result.
    pub revision: Revision,
    /// The misspellings found in the text at that revision.
    pub misspellings: Vec<Misspelling>,
    /// A fast, non-cryptographic fingerprint of the exact text this result
    /// was computed against. This exists solely to catch a caller bug:
    /// reusing the same [`Revision`] value after the text actually changed
    /// (i.e. forgetting to bump the revision counter). It is never exposed
    /// or compared on its own; see [`Self::is_stale_for`].
    text_fingerprint: u64,
}

impl CheckResult {
    /// True if `current_revision` differs from the revision this result was
    /// computed against — i.e. the source has since changed and this result
    /// must not be reused (displayed, acted on, ...) without recomputing.
    ///
    /// This compares only the revision identifier. It trusts the caller's
    /// revision bookkeeping: if the text changed but the caller passes back
    /// the *same* revision value by mistake, this method alone cannot see
    /// that. Use [`Self::is_stale_for`] when the current text is available
    /// and that class of bug matters to detect.
    pub fn is_stale(&self, current_revision: Revision) -> bool {
        self.revision != current_revision
    }

    /// True if this result must not be reused against `current_text` at
    /// `current_revision`: either the revision moved on (same check as
    /// [`Self::is_stale`]), or the revision is unchanged but the text is not
    /// byte-for-byte what this result was computed against — a caller bug
    /// (a missed revision bump) that plain revision comparison cannot catch
    /// on its own. Identical revision *and* identical text is fresh.
    pub fn is_stale_for(&self, current_revision: Revision, current_text: &str) -> bool {
        self.is_stale(current_revision) || self.text_fingerprint != fingerprint(current_text)
    }
}

/// Fast, non-cryptographic fingerprint of `text`, used only by
/// [`CheckResult::is_stale_for`] to detect same-revision text drift. Not a
/// security hash and not exposed to callers; collisions are irrelevant to
/// its purpose (a diagnostic for a caller bookkeeping bug), not a source of
/// unsoundness anywhere else in this crate.
fn fingerprint(text: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

/// Outcome of a cancellable check: either it ran to completion, or a
/// caller-supplied cancellation signal fired first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckOutcome {
    /// The check ran to completion.
    Completed(CheckResult),
    /// Cancelled before completion. The revision is still reported (it was
    /// known before any work started) so the caller can correlate the
    /// cancellation with the request that issued it, but no misspellings
    /// are returned: a cancelled check makes no claim about the text.
    Cancelled {
        /// The revision that was passed in when cancellation was observed.
        revision: Revision,
    },
}

// ---- Bounded user dictionary ---------------------------------------------

/// Default bound on the number of entries a [`UserDictionary`] holds; see
/// [`UserDictionary::new`] to configure a different bound.
pub const USER_DICTIONARY_DEFAULT_MAX_ENTRIES: usize = 10_000;

/// Error returned when a [`UserDictionary`] mutation would exceed its
/// configured capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserDictionaryError {
    /// The dictionary already holds `max_entries` distinct entries; the
    /// requested addition/ignore was rejected rather than silently applied
    /// or an existing entry silently evicted.
    CapacityExceeded {
        /// The configured bound that was hit.
        max_entries: usize,
    },
}

impl std::fmt::Display for UserDictionaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserDictionaryError::CapacityExceeded { max_entries } => {
                write!(
                    f,
                    "user dictionary capacity exceeded (max {max_entries} entries)"
                )
            }
        }
    }
}

impl std::error::Error for UserDictionaryError {}

/// A bounded, caller-managed, in-memory layer of words on top of a base
/// [`Dictionary`]. This crate ships no dictionary and does no I/O: the
/// caller decides how (or whether) to persist this across sessions.
///
/// Two kinds of entries, both counted against the same bound:
/// - [`Self::add_word`]: a word the caller has confirmed is genuinely
///   correct (e.g. "FlashTeX"). It stops being flagged, project-wide.
/// - [`Self::ignore_word`]: a one-off suppression (e.g. a placeholder
///   token) the caller does not want flagged but is not vouching for as a
///   real word.
///
/// Exceeding the configured capacity returns a typed
/// [`UserDictionaryError`] rather than growing unbounded or silently
/// evicting an existing entry.
#[derive(Debug, Clone)]
pub struct UserDictionary {
    max_entries: usize,
    additions: HashSet<String>,
    ignored: HashSet<String>,
}

impl UserDictionary {
    /// Creates an empty user dictionary bounded to `max_entries` combined
    /// additions + ignores.
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            additions: HashSet::new(),
            ignored: HashSet::new(),
        }
    }

    /// Total number of distinct entries currently stored (additions +
    /// ignores).
    pub fn len(&self) -> usize {
        self.additions.len() + self.ignored.len()
    }

    /// True if no entries have been added or ignored yet.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Marks `word` as a known-correct word: it stops being flagged, and it
    /// becomes eligible as a suggestion for *other* misspelled words.
    /// Idempotent (adding an already-added word always succeeds, even at
    /// capacity); promotes an existing ignore to an addition in place.
    /// Returns [`UserDictionaryError::CapacityExceeded`] only when the
    /// dictionary is already full and `word` is a genuinely new entry.
    pub fn add_word(&mut self, word: &str) -> Result<(), UserDictionaryError> {
        if self.additions.contains(word) {
            return Ok(());
        }
        if self.ignored.remove(word) {
            self.additions.insert(word.to_string());
            return Ok(());
        }
        if self.len() >= self.max_entries {
            return Err(UserDictionaryError::CapacityExceeded {
                max_entries: self.max_entries,
            });
        }
        self.additions.insert(word.to_string());
        Ok(())
    }

    /// Marks `word` as ignored: it stops being flagged, but (unlike
    /// [`Self::add_word`]) it is never offered as a suggestion for other
    /// misspelled words. A no-op if `word` is already an addition
    /// (additions are the stronger claim and are not downgraded).
    pub fn ignore_word(&mut self, word: &str) -> Result<(), UserDictionaryError> {
        if self.additions.contains(word) || self.ignored.contains(word) {
            return Ok(());
        }
        if self.len() >= self.max_entries {
            return Err(UserDictionaryError::CapacityExceeded {
                max_entries: self.max_entries,
            });
        }
        self.ignored.insert(word.to_string());
        Ok(())
    }

    /// True if `word` was added via [`Self::add_word`].
    pub fn is_addition(&self, word: &str) -> bool {
        self.additions.contains(word)
    }

    /// True if `word` was ignored via [`Self::ignore_word`] (and not since
    /// promoted to an addition).
    pub fn is_ignored(&self, word: &str) -> bool {
        self.ignored.contains(word)
    }

    /// True if `word` is suppressed from flagging by either layer.
    fn contains_entry(&self, word: &str) -> bool {
        self.is_addition(word) || self.is_ignored(word)
    }
}

impl Default for UserDictionary {
    fn default() -> Self {
        Self::new(USER_DICTIONARY_DEFAULT_MAX_ENTRIES)
    }
}

/// Layers a caller-managed [`UserDictionary`] over a base [`Dictionary`]
/// without mutating or copying either. Pass a value of this type anywhere a
/// `&dyn Dictionary` is expected (e.g. to [`SpellChecker::check`]) to
/// combine both.
pub struct LayeredDictionary<'a> {
    base: &'a dyn Dictionary,
    user: &'a UserDictionary,
}

impl<'a> LayeredDictionary<'a> {
    /// Creates a view that checks `user` first, falling back to `base`.
    pub fn new(base: &'a dyn Dictionary, user: &'a UserDictionary) -> Self {
        Self { base, user }
    }
}

impl Dictionary for LayeredDictionary<'_> {
    fn contains(&self, word: &str) -> bool {
        self.user.contains_entry(word) || self.base.contains(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn dict(words: &[&str]) -> HashSet<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    // ---- Hand-worked suggestion tests (exact lists) ----------------------

    #[test]
    fn suggests_transposition_exactly() {
        let d = dict(&["world", "word", "worlds"]);
        let checker = SpellChecker::default();
        let out = checker.check("wrold", &d);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].word, "wrold");
        assert_eq!(out[0].range, 0..5);
        assert_eq!(out[0].suggestions, vec!["world".to_string()]);
    }

    #[test]
    fn suggests_insertion_exactly() {
        let d = dict(&["hello"]);
        let checker = SpellChecker::default();
        let out = checker.check("helo", &d);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].suggestions, vec!["hello".to_string()]);
    }

    #[test]
    fn suggests_deletion_exactly() {
        let d = dict(&["function", "fiction"]);
        let checker = SpellChecker::default();
        // "functtion" -> delete one 't' -> "function".
        let out = checker.check("functtion", &d);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].suggestions, vec!["function".to_string()]);
    }

    #[test]
    fn suggests_substitution_exactly() {
        let d = dict(&["dictionary"]);
        let checker = SpellChecker::default();
        // "dixtionary" -> substitute 'x' with 'c' -> "dictionary".
        let out = checker.check("dixtionary", &d);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].suggestions, vec!["dictionary".to_string()]);
    }

    #[test]
    fn ranks_multiple_suggestions_deterministically() {
        // "cxt" is a single substitution away from "cat", "cot", and "cut"
        // (position 1: x -> a/o/u), and two substitutions away from "bat"
        // (positions 0 and 1), so "bat" must not appear at all. Result must
        // be sorted lexicographically at equal distance and capped.
        let d = dict(&["cat", "cot", "cut", "bat"]);
        let cfg = SpellCheckerConfig {
            max_suggestions: 2,
            ..SpellCheckerConfig::default()
        };
        let checker = SpellChecker::new(cfg);
        let out = checker.check("cxt", &d);
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].suggestions,
            vec!["cat".to_string(), "cot".to_string()]
        );
    }

    #[test]
    fn correct_word_has_no_suggestions_reported() {
        let d = dict(&["hello", "world"]);
        let checker = SpellChecker::default();
        let out = checker.check("hello world", &d);
        assert!(out.is_empty());
    }

    #[test]
    fn case_insensitive_dictionary_match_does_not_flag() {
        let d = dict(&["the", "quick", "brown", "fox"]);
        let checker = SpellChecker::default();
        // Capitalized sentence-start word should match lowercase dict entry.
        let out = checker.check("The quick brown fox", &d);
        assert!(out.is_empty());
    }

    // ---- Math exclusion ----------------------------------------------------

    #[test]
    fn inline_math_is_excluded_but_body_text_is_checked() {
        let d = dict(&["the", "is", "big", "world"]);
        let text = "The wrold is $wrold$ big.";
        let checker = SpellChecker::default();
        let out = checker.check(text, &d);
        assert_eq!(out.len(), 1, "expected exactly one flagged word: {out:?}");
        assert_eq!(out[0].word, "wrold");
        // The body-text occurrence, not the one inside $...$.
        let expected_start = text.find("wrold").unwrap();
        assert_eq!(out[0].range, expected_start..expected_start + 5);
    }

    #[test]
    fn display_math_double_dollar_is_excluded() {
        let d = dict(&["the", "answer", "is"]);
        let text = "The answer is $$ wrold = qwerty $$ done.";
        let checker = SpellChecker::default();
        let out = checker.check(text, &d);
        // "wrold", "qwerty", and "done" are all inside/adjacent; only "done"
        // is outside math and unknown -> flagged. "wrold"/"qwerty" excluded.
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].word, "done");
    }

    #[test]
    fn paren_and_bracket_math_are_excluded() {
        let d = dict(&["the", "value", "is", "and"]);
        let text = r"The value is \(wrold\) and \[qwerty\] end.";
        let checker = SpellChecker::default();
        let out = checker.check(text, &d);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].word, "end");
    }

    // ---- Command-name exclusion --------------------------------------------

    #[test]
    fn command_name_is_excluded_but_same_word_in_body_is_checked() {
        let d = dict(&["and", "body"]);
        let text = r"\wrold and wrold body";
        let checker = SpellChecker::default();
        let out = checker.check(text, &d);
        assert_eq!(out.len(), 1, "expected exactly one flagged word: {out:?}");
        assert_eq!(out[0].word, "wrold");
        let expected_start = text.rfind("wrold").unwrap();
        assert_eq!(out[0].range, expected_start..expected_start + 5);
    }

    #[test]
    fn starred_command_name_is_excluded() {
        let d = dict(&["heading"]);
        let text = r"\wrold*{heading}";
        let checker = SpellChecker::default();
        let out = checker.check(text, &d);
        assert!(
            out.is_empty(),
            "starred command name must not be flagged: {out:?}"
        );
    }

    #[test]
    fn escaped_special_characters_do_not_open_math() {
        let d = dict(&["price", "is", "dollars"]);
        // `\$` is a literal dollar sign, not a math delimiter.
        let text = r"The price is \$5 dollars";
        let checker = SpellChecker::default();
        let out = checker.check(text, &d);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].word, "The");
    }

    // ---- UTF-8 safety with multi-byte characters ---------------------------

    #[test]
    fn multibyte_word_range_is_char_boundary_safe_and_correct() {
        let d = dict(&["hello"]);
        let text = "café hello"; // "café" = c,a,f,é -> 5 bytes, 4 chars
        let checker = SpellChecker::default();
        let out = checker.check(text, &d);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].word, "café");
        assert_eq!(out[0].range, 0..5);
        assert!(text.is_char_boundary(out[0].range.start));
        assert!(text.is_char_boundary(out[0].range.end));
        assert_eq!(&text[out[0].range.clone()], "café");
    }

    #[test]
    fn multibyte_math_exclusion_boundaries_are_char_safe() {
        let text = "münchen $föö wrold$ bar";
        let excluded = excluded_ranges(text);
        assert_eq!(excluded.len(), 1);
        let r = excluded[0].clone();
        assert!(text.is_char_boundary(r.start));
        assert!(text.is_char_boundary(r.end));
        assert_eq!(&text[r], "$föö wrold$");
    }

    #[test]
    fn emoji_and_zero_width_characters_do_not_panic_or_split() {
        let d = dict(&["hello"]);
        let text = "hello \u{1F389} w\u{200B}orld"; // party emoji + zero-width space
        let checker = SpellChecker::default();
        // Must not panic; every reported range must be a valid char-boundary slice.
        let out = checker.check(text, &d);
        for m in &out {
            assert!(text.is_char_boundary(m.range.start));
            assert!(text.is_char_boundary(m.range.end));
            assert_eq!(&text[m.range.clone()], m.word);
        }
    }

    // ---- Malformed input: bounded, no panics -------------------------------

    #[test]
    fn unterminated_math_delimiter_excludes_to_end_without_panicking() {
        let d = dict(&["hello"]);
        let text = "hello $unterminated forever";
        let checker = SpellChecker::default();
        let out = checker.check(text, &d);
        // Everything from the stray '$' onward is excluded; only "hello" is
        // in normal prose and it's a known word, so nothing is flagged.
        assert!(out.is_empty(), "unexpected flags: {out:?}");
    }

    #[test]
    fn trailing_lone_backslash_does_not_panic() {
        let d = dict(&["trailing"]);
        let text = "trailing backslash \\";
        let checker = SpellChecker::default();
        let out = checker.check(text, &d);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].word, "backslash");
    }

    #[test]
    fn degenerate_double_backslash_command_does_not_panic() {
        let d = dict(&["word"]);
        let text = r"\\word";
        let checker = SpellChecker::default();
        let out = checker.check(text, &d);
        assert!(
            out.is_empty(),
            "\\\\ is a line-break control symbol, 'word' is known: {out:?}"
        );
    }

    #[test]
    fn dollar_at_end_of_input_does_not_panic() {
        let d = dict(&["price", "is"]);
        let text = "price is $";
        let checker = SpellChecker::default();
        let out = checker.check(text, &d);
        assert!(out.is_empty(), "unexpected flags: {out:?}");
    }

    #[test]
    fn empty_input_produces_no_misspellings() {
        let d = dict(&["anything"]);
        let checker = SpellChecker::default();
        assert!(checker.check("", &d).is_empty());
    }

    // ---- Bounded suggestion generation --------------------------------------

    #[test]
    fn long_word_gets_no_suggestions_but_is_still_flagged_and_is_fast() {
        let d = dict(&["hello"]);
        let checker = SpellChecker::default();
        assert!(
            "x".repeat(200).len() > checker.config().max_word_length_for_suggestions,
            "test word must exceed the bound"
        );
        let long_word = "x".repeat(200);
        let text = format!("hello {long_word}");
        let start = Instant::now();
        let out = checker.check(&text, &d);
        let elapsed = start.elapsed();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].word, long_word);
        assert!(
            out[0].suggestions.is_empty(),
            "words beyond max_word_length_for_suggestions must get no suggestions"
        );
        assert!(
            elapsed.as_secs() < 2,
            "long-word handling must stay fast, took {elapsed:?}"
        );
    }

    #[test]
    fn suggestion_count_never_exceeds_configured_cap_even_with_large_dictionary() {
        // A large dictionary containing all 26 "c?t" words plus many
        // unrelated entries. "cαt" (Greek alpha in the middle) is not
        // itself in the dictionary and is a single substitution away from
        // all 26 "c?t" entries, so this genuinely exercises the cap.
        // Suggestion count must still respect it; cost is bounded by word
        // length and the cap, not by dictionary size.
        let mut words: Vec<String> = ('a'..='z').map(|c| format!("c{c}t")).collect();
        for i in 0..5000 {
            words.push(format!("unrelated{i}"));
        }
        let d: HashSet<String> = words.into_iter().collect();

        let cfg = SpellCheckerConfig {
            max_suggestions: 4,
            ..SpellCheckerConfig::default()
        };
        let checker = SpellChecker::new(cfg);
        let out = checker.check("cαt", &d);
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].suggestions.len(),
            4,
            "must be capped at max_suggestions even though 26 candidates matched"
        );
        assert_eq!(
            out[0].suggestions,
            vec![
                "cat".to_string(),
                "cbt".to_string(),
                "cct".to_string(),
                "cdt".to_string()
            ]
        );
    }

    #[test]
    fn edit_distance_is_clamped_to_hard_ceiling() {
        let cfg = SpellCheckerConfig {
            max_edit_distance: 50,
            ..SpellCheckerConfig::default()
        };
        let checker = SpellChecker::new(cfg);
        assert_eq!(
            checker.config().max_edit_distance,
            SpellChecker::MAX_ALLOWED_EDIT_DISTANCE
        );
    }

    #[test]
    fn word_length_for_suggestions_is_clamped_to_hard_ceiling() {
        let cfg = SpellCheckerConfig {
            max_word_length_for_suggestions: usize::MAX,
            ..SpellCheckerConfig::default()
        };
        let checker = SpellChecker::new(cfg);
        assert_eq!(
            checker.config().max_word_length_for_suggestions,
            SpellChecker::MAX_ALLOWED_WORD_LENGTH_FOR_SUGGESTIONS
        );
    }

    #[test]
    fn edits1_length_is_bounded_by_alphabet_and_word_length() {
        let word = "hello";
        let n = word.chars().count();
        let alphabet = EDIT_ALPHABET.len();
        let candidates = edits1(word);
        // deletions(n) + transpositions(n-1) + substitutions(n*alphabet) +
        // insertions((n+1)*alphabet), before dedup.
        let upper_bound = n + n.saturating_sub(1) + n * alphabet + (n + 1) * alphabet;
        assert!(candidates.len() <= upper_bound);
    }

    // ---- No bundled dictionary / dictionary trait shape --------------------

    #[test]
    fn btreeset_dictionary_works_too() {
        let d: BTreeSet<String> = ["hello", "world"].iter().map(|s| s.to_string()).collect();
        let checker = SpellChecker::default();
        assert!(checker.check("hello world", &d).is_empty());
        assert_eq!(
            checker.check("wrold", &d)[0].suggestions,
            vec!["world".to_string()]
        );
    }

    // ---- User dictionary: bounded, additions vs. ignores -------------------

    #[test]
    fn user_dictionary_add_word_respects_capacity_bound() {
        let mut ud = UserDictionary::new(2);
        ud.add_word("alpha").unwrap();
        ud.add_word("beta").unwrap();
        let err = ud.add_word("gamma").unwrap_err();
        assert_eq!(
            err,
            UserDictionaryError::CapacityExceeded { max_entries: 2 }
        );
        // Re-adding an already-present word is always fine, even at capacity.
        ud.add_word("alpha").unwrap();
        assert_eq!(ud.len(), 2);
    }

    #[test]
    fn user_dictionary_ignore_word_shares_the_same_capacity_bound_as_additions() {
        let mut ud = UserDictionary::new(1);
        ud.ignore_word("todo").unwrap();
        let err = ud.add_word("other").unwrap_err();
        assert_eq!(
            err,
            UserDictionaryError::CapacityExceeded { max_entries: 1 }
        );
    }

    #[test]
    fn user_dictionary_error_display_mentions_the_bound() {
        let err = UserDictionaryError::CapacityExceeded { max_entries: 3 };
        assert!(err.to_string().contains('3'));
    }

    #[test]
    fn user_dictionary_ignore_does_not_promote_to_addition_and_add_promotes_existing_ignore() {
        let mut ud = UserDictionary::new(10);
        ud.ignore_word("scratch").unwrap();
        assert!(ud.is_ignored("scratch"));
        assert!(!ud.is_addition("scratch"));

        ud.add_word("scratch").unwrap();
        assert!(ud.is_addition("scratch"));
        assert!(!ud.is_ignored("scratch"));
        // Promotion must not double-count against capacity.
        assert_eq!(ud.len(), 1);
    }

    #[test]
    fn layered_dictionary_suppresses_additions_and_ignores_without_mutating_the_base() {
        let base = dict(&["hello"]);
        let mut ud = UserDictionary::new(10);
        ud.add_word("flashtex").unwrap();
        ud.ignore_word("wrold").unwrap();
        let layered = LayeredDictionary::new(&base, &ud);

        let checker = SpellChecker::default();
        let out = checker.check("hello flashtex wrold unknownword", &layered);
        assert_eq!(
            out.len(),
            1,
            "only the truly unknown word should be flagged: {out:?}"
        );
        assert_eq!(out[0].word, "unknownword");

        // The caller-supplied base dictionary itself is never mutated.
        assert!(!Dictionary::contains(&base, "flashtex"));
        assert!(!Dictionary::contains(&base, "wrold"));
    }

    // ---- Source-revision awareness ------------------------------------------

    #[test]
    fn check_revision_tags_result_and_detects_staleness() {
        let d = dict(&["hello"]);
        let checker = SpellChecker::default();
        let result = checker.check_revision("helo", 5, &d);
        assert_eq!(result.revision, 5);
        assert_eq!(result.misspellings.len(), 1);
        assert!(
            !result.is_stale(5),
            "a result checked at the current revision is not stale"
        );
        assert!(
            result.is_stale(6),
            "a result checked at an older revision must be detectably stale"
        );
    }

    // ---- Cancellation --------------------------------------------------------

    #[test]
    fn check_cancellable_without_cancellation_matches_plain_check() {
        let d = dict(&["world"]);
        let checker = SpellChecker::default();
        let text = "wrold and wrold again";
        let plain = checker.check(text, &d);
        let outcome = checker.check_cancellable(text, 42, &d, &|| false);
        match outcome {
            CheckOutcome::Completed(result) => {
                assert_eq!(result.revision, 42);
                assert_eq!(result.misspellings, plain);
            }
            CheckOutcome::Cancelled { .. } => panic!("must not cancel when the signal never fires"),
        }
    }

    #[test]
    fn cancellation_short_circuits_a_long_check_instead_of_running_to_completion() {
        let d = dict(&["hello"]);
        let checker = SpellChecker::default();
        // Many separately-tokenized misspelled words, each of which would
        // trigger a bounded but non-trivial suggestion search if processed.
        let words: Vec<String> = (0..500).map(|i| format!("wrold{i}")).collect();
        let text = words.join(" ");

        let calls = std::cell::Cell::new(0usize);
        let is_cancelled = || {
            calls.set(calls.get() + 1);
            calls.get() > 3
        };
        let outcome = checker.check_cancellable(&text, 7, &d, &is_cancelled);

        match outcome {
            CheckOutcome::Cancelled { revision } => assert_eq!(revision, 7),
            CheckOutcome::Completed(_) => {
                panic!("expected cancellation to short-circuit the check")
            }
        }
        assert!(
            calls.get() <= 5,
            "cancellation must be observed promptly, not only after scanning all {} words (polled {} times)",
            words.len(),
            calls.get()
        );
    }

    #[test]
    fn check_cancellable_reports_the_given_revision_even_when_cancelled_immediately() {
        let d = dict(&["hello"]);
        let checker = SpellChecker::default();
        let outcome = checker.check_cancellable("hello world", 99, &d, &|| true);
        assert_eq!(outcome, CheckOutcome::Cancelled { revision: 99 });
    }
}
