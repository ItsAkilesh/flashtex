//! Statistics computed from explicit items, bound exactly to a revision.

use crate::items::SourceItem;
use crate::revision::RevisionId;
use crate::scan_limit::{ScanLimit, ScanTooLarge};
use crate::words::WordStats;

/// Math item counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MathStats {
    /// All math items, inline and display combined.
    pub total: usize,
    /// Inline math items (`$ ... $`).
    pub inline: usize,
    /// Display math items (`\[ ... \]`, `equation`, ...).
    pub display: usize,
}

/// Bounded statistics computed from an explicit, caller-supplied list of
/// [`SourceItem`]s, bound exactly to the [`RevisionId`] and content they
/// were computed from.
///
/// A `Statistics` value never claims to describe anything the caller didn't
/// hand it directly: there is no filesystem read, no network call, and no
/// TeX/PDF parsing anywhere in [`Statistics::compute`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statistics {
    /// The revision these statistics were computed from.
    pub revision: RevisionId,
    /// A deterministic fingerprint of the exact items counted, used by
    /// [`Statistics::is_current_for`] to detect a `RevisionId` that was
    /// reused for different content. Not cryptographic, and not meant to be
    /// stable across crate versions — only to catch drift within one
    /// pipeline run. See that method before relying on `revision` alone.
    pub content_hash: u64,
    /// Word statistics, summed over every [`SourceItem::Text`] item.
    pub words: WordStats,
    /// Math statistics, summed over every [`SourceItem::Math`] item.
    pub math: MathStats,
    /// Number of pages, i.e. the number of [`SourceItem::PageMark`] items.
    pub pages: usize,
    /// Source bytes actually scanned to compute `words` and `math`: the
    /// summed byte length of every [`SourceItem::Text`] string and every
    /// [`crate::items::MathItem::source`] string. `PageMark` items and the
    /// cheap `content_hash` fingerprint pass are not counted here — this is
    /// specifically the cost [`ScanLimit`] bounds, and what a cache is
    /// expected to let a caller avoid paying twice for unchanged content.
    pub scanned_bytes: usize,
}

impl Statistics {
    /// Compute statistics for `items`, bound to `revision`, with no cap on
    /// how many bytes may be scanned.
    ///
    /// This is pure counting over `items`: no filesystem or network access,
    /// no rendering, and no interpretation of math or page-break syntax
    /// beyond the [`SourceItem`] variant the caller chose. Equivalent to
    /// [`Statistics::compute_bounded`] with [`ScanLimit::UNBOUNDED`], which
    /// can never fail.
    pub fn compute(revision: RevisionId, items: &[SourceItem]) -> Statistics {
        Statistics::compute_bounded(revision, items, ScanLimit::UNBOUNDED)
            .expect("ScanLimit::UNBOUNDED never rejects a scan")
    }

    /// Compute statistics for `items`, bound to `revision`, refusing to scan
    /// past `limit`.
    ///
    /// `limit` bounds [`Statistics::scanned_bytes`] — the summed byte length
    /// of [`SourceItem::Text`] and math-source content actually counted.
    /// Items are scanned in order; as soon as the running total would exceed
    /// `limit`, this returns [`ScanTooLarge`] instead of silently truncating
    /// or producing a partial count. No partial `Statistics` is ever
    /// returned on error.
    pub fn compute_bounded(
        revision: RevisionId,
        items: &[SourceItem],
        limit: ScanLimit,
    ) -> Result<Statistics, ScanTooLarge> {
        let mut words = WordStats::default();
        let mut math = MathStats::default();
        let mut pages = 0usize;
        let mut scanned_bytes = 0usize;
        for item in items {
            match item {
                SourceItem::Text(text) => {
                    scanned_bytes += text.len();
                    if scanned_bytes > limit.max_bytes() {
                        return Err(ScanTooLarge::new(scanned_bytes, limit));
                    }
                    words = words + WordStats::of(text);
                }
                SourceItem::Math(m) => {
                    scanned_bytes += m.source.len();
                    if scanned_bytes > limit.max_bytes() {
                        return Err(ScanTooLarge::new(scanned_bytes, limit));
                    }
                    math.total += 1;
                    if m.display {
                        math.display += 1;
                    } else {
                        math.inline += 1;
                    }
                }
                SourceItem::PageMark => pages += 1,
            }
        }
        let content_hash = fingerprint(items);
        Ok(Statistics {
            revision,
            content_hash,
            words,
            math,
            pages,
            scanned_bytes,
        })
    }

    /// `true` if `self` is exactly what [`Statistics::compute`] would
    /// return for `revision` and `items` right now: the same revision
    /// identity AND the same exact content.
    ///
    /// This is the guard against mistaking a stale result for a current
    /// one: a `RevisionId` match alone is not trusted, because a caller
    /// could reuse a revision id for changed content (e.g. forgetting to
    /// bump a revision counter). Only a match on both fields counts as
    /// current.
    pub fn is_current_for(&self, revision: &RevisionId, items: &[SourceItem]) -> bool {
        &self.revision == revision && self.content_hash == fingerprint(items)
    }
}

/// FNV-1a, fed a length-prefixed encoding of each item so that, e.g.,
/// `[Text("ab"), Text("c")]` and `[Text("a"), Text("bc")]` never collide
/// just because their bytes concatenate the same way.
///
/// `pub(crate)` so [`crate::cache`] can use the exact same fingerprint as a
/// cheap cache-key identity check, without re-running the (bounded, and
/// potentially rejected) full statistics scan just to know whether cached
/// content is still current.
pub(crate) fn fingerprint(items: &[SourceItem]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    let mut h = FNV_OFFSET;
    for item in items {
        match item {
            SourceItem::Text(s) => {
                fnv_feed(&mut h, &[0u8]);
                fnv_feed_str(&mut h, s);
            }
            SourceItem::Math(m) => {
                fnv_feed(&mut h, &[if m.display { 2u8 } else { 1u8 }]);
                fnv_feed_str(&mut h, &m.source);
            }
            SourceItem::PageMark => fnv_feed(&mut h, &[3u8]),
        }
    }
    h
}

fn fnv_feed(h: &mut u64, bytes: &[u8]) {
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    for &b in bytes {
        *h ^= u64::from(b);
        *h = h.wrapping_mul(FNV_PRIME);
    }
}

fn fnv_feed_str(h: &mut u64, s: &str) {
    fnv_feed(h, &(s.len() as u64).to_le_bytes());
    fnv_feed(h, s.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::SourceItem;

    fn rev(n: u64) -> RevisionId {
        RevisionId::new("draft.tex", n)
    }

    #[test]
    fn empty_document_is_all_zero() {
        let s = Statistics::compute(rev(1), &[]);
        assert_eq!(s.words, WordStats::default());
        assert_eq!(s.math, MathStats::default());
        assert_eq!(s.pages, 0);
    }

    #[test]
    fn hand_worked_mixed_document() {
        let items = vec![
            SourceItem::PageMark,
            SourceItem::text("Well-known results don't need proof."),
            SourceItem::inline_math("x^2"),
            SourceItem::PageMark,
            SourceItem::text("See equation above."),
            SourceItem::display_math("\\int_0^1 x\\,dx"),
        ];
        let s = Statistics::compute(rev(1), &items);
        // "Well-known" "results" "don't" "need" "proof." = 5
        // "See" "equation" "above." = 3
        assert_eq!(s.words.words, 8);
        assert_eq!(s.math.total, 2);
        assert_eq!(s.math.inline, 1);
        assert_eq!(s.math.display, 1);
        assert_eq!(s.pages, 2);
    }

    #[test]
    fn no_page_marks_means_zero_pages_even_with_text() {
        let items = vec![SourceItem::text("some text with no page markers")];
        let s = Statistics::compute(rev(1), &items);
        assert_eq!(s.pages, 0);
        assert_eq!(s.words.words, 6);
    }

    #[test]
    fn consecutive_empty_page_marks_still_count_pages() {
        let items = vec![
            SourceItem::PageMark,
            SourceItem::PageMark,
            SourceItem::PageMark,
        ];
        let s = Statistics::compute(rev(1), &items);
        assert_eq!(s.pages, 3);
        assert_eq!(s.words.words, 0);
    }

    #[test]
    fn math_with_empty_source_still_counts() {
        let items = vec![SourceItem::inline_math(""), SourceItem::display_math("")];
        let s = Statistics::compute(rev(1), &items);
        assert_eq!(s.math.total, 2);
        assert_eq!(s.words.words, 0);
    }

    #[test]
    fn math_source_is_never_word_counted() {
        let items = vec![SourceItem::inline_math(
            "this looks like prose but it is math source",
        )];
        let s = Statistics::compute(rev(1), &items);
        assert_eq!(s.words.words, 0);
        assert_eq!(s.math.total, 1);
    }

    #[test]
    fn bounded_large_input_sums_correctly() {
        let items: Vec<SourceItem> = (0..10_000).map(|_| SourceItem::text("word ")).collect();
        let s = Statistics::compute(rev(1), &items);
        assert_eq!(s.words.words, 10_000);
    }

    // --- Revision binding: the core of this crate's honesty guarantee ---

    #[test]
    fn is_current_for_same_revision_and_content() {
        let items = vec![SourceItem::text("hello world")];
        let s = Statistics::compute(rev(1), &items);
        assert!(s.is_current_for(&rev(1), &items));
    }

    #[test]
    fn stale_when_content_changed_but_revision_id_reused() {
        let original = vec![SourceItem::text("hello world")];
        let changed = vec![SourceItem::text("hello world, edited")];
        let s = Statistics::compute(rev(1), &original);
        // Same claimed revision id, but the content moved on: NOT current.
        assert!(!s.is_current_for(&rev(1), &changed));
    }

    #[test]
    fn stale_when_revision_id_differs_even_with_identical_content() {
        let items = vec![SourceItem::text("hello world")];
        let s = Statistics::compute(rev(1), &items);
        // Same content, but a different claimed revision: NOT current.
        assert!(!s.is_current_for(&rev(2), &items));
        assert!(!s.is_current_for(&RevisionId::new("other.tex", 1), &items));
    }

    #[test]
    fn fingerprint_is_deterministic_for_identical_items() {
        let items = vec![SourceItem::text("a"), SourceItem::inline_math("b")];
        let s1 = Statistics::compute(rev(1), &items);
        let s2 = Statistics::compute(rev(1), &items);
        assert_eq!(s1.content_hash, s2.content_hash);
    }

    #[test]
    fn fingerprint_distinguishes_concatenation_boundaries() {
        let a = vec![SourceItem::text("ab"), SourceItem::text("c")];
        let b = vec![SourceItem::text("a"), SourceItem::text("bc")];
        let sa = Statistics::compute(rev(1), &a);
        let sb = Statistics::compute(rev(1), &b);
        assert_ne!(sa.content_hash, sb.content_hash);
    }

    #[test]
    fn fingerprint_distinguishes_inline_from_display_math() {
        let a = vec![SourceItem::inline_math("x")];
        let b = vec![SourceItem::display_math("x")];
        let sa = Statistics::compute(rev(1), &a);
        let sb = Statistics::compute(rev(1), &b);
        assert_ne!(sa.content_hash, sb.content_hash);
    }

    // --- Bounded scanning: the cap this crate enforces on scanned bytes ---

    #[test]
    fn scanned_bytes_sums_text_and_math_source_but_not_page_marks() {
        let items = vec![
            SourceItem::PageMark,
            SourceItem::text("abcde"),     // 5 bytes
            SourceItem::inline_math("xy"), // 2 bytes
            SourceItem::display_math("z"), // 1 byte
        ];
        let s = Statistics::compute(rev(1), &items);
        assert_eq!(s.scanned_bytes, 8);
    }

    #[test]
    fn compute_bounded_within_limit_matches_compute() {
        let items = vec![SourceItem::text("hello world")];
        let unbounded = Statistics::compute(rev(1), &items);
        let bounded =
            Statistics::compute_bounded(rev(1), &items, crate::scan_limit::ScanLimit::new(64))
                .unwrap();
        assert_eq!(unbounded, bounded);
    }

    #[test]
    fn compute_bounded_rejects_past_the_limit_with_no_partial_result() {
        let items = vec![SourceItem::text("this text is much too long")];
        let err = Statistics::compute_bounded(rev(1), &items, crate::scan_limit::ScanLimit::new(5))
            .unwrap_err();
        assert_eq!(err.limit, 5);
        assert!(err.scanned > 5);
    }

    #[test]
    fn compute_bounded_stops_at_the_item_that_crosses_the_limit() {
        // First item alone fits; the second pushes the running total past
        // the limit, so counting must stop there, not silently include it
        // partially or skip ahead to later items.
        let items = vec![
            SourceItem::text("fits"),      // 4 bytes, running total 4
            SourceItem::text("overflow!"), // 9 bytes, running total 13 > 10
            SourceItem::text("never scanned"),
        ];
        let err =
            Statistics::compute_bounded(rev(1), &items, crate::scan_limit::ScanLimit::new(10))
                .unwrap_err();
        assert_eq!(err.scanned, 13);
        assert_eq!(err.limit, 10);
    }
}
