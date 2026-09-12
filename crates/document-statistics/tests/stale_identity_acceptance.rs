//! Stale-identity acceptance: when may a cached `Statistics` be trusted?
//!
//! This file states, as acceptance criteria rather than white-box unit
//! tests, the exact contract under which a cached answer may be trusted:
//!
//! 1. Same revision, same content -> HIT. Trust the cached answer.
//! 2. Same revision, changed content -> MISS. This is the dangerous case: a
//!    caller that forgot to bump its revision counter must NOT be served a
//!    stale answer under the guise of a fresh one.
//! 3. Changed revision, same content -> MISS. A `RevisionId` match is
//!    necessary but a content match alone is not sufficient either way: the
//!    claimed identity must match exactly, not just the bytes.
//! 4. A sibling document's cache entry is untouched by either a hit or a
//!    miss on another document.
//!
//! `rev-2`'s `tests/incremental_equals_fresh.rs` already proves this
//! property statistically, across thousands of randomized steps. This file
//! is the readable specification version: one example per rule, named after
//! the rule it states, so the contract can be read straight off the test
//! names without reconstructing it from a property test's PRNG.

use flashtex_document_statistics::{ProjectCache, RevisionId, ScanLimit, SourceItem};

fn rev(source: &str, n: u64) -> RevisionId {
    RevisionId::new(source, n)
}

/// Rule 1: same revision, same content -> a cached answer MAY be trusted.
#[test]
fn same_revision_same_content_is_a_hit_the_cached_answer_is_trustworthy() {
    let mut cache = ProjectCache::new();
    let content = vec![SourceItem::text("the results are unchanged")];

    let first = cache
        .update(rev("paper.tex", 5), &content, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(
        !first.hit,
        "the very first sighting of a document is always a miss"
    );

    // Re-asserting the identical revision id over byte-for-byte identical
    // content: this is the ONLY situation in which trusting a cached answer
    // without rescanning is safe, and the cache must recognize it as such.
    let second = cache
        .update(rev("paper.tex", 5), &content, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(
        second.hit,
        "identical revision + identical content must be a hit"
    );
    assert_eq!(second.bytes_scanned, 0, "a trusted hit rescans nothing");
    assert_eq!(
        second.stats, first.stats,
        "a hit must return the exact same statistics"
    );
}

/// Rule 2, THE DANGEROUS CASE: same revision, changed content -> MISS.
///
/// If this were ever a hit, a caller that forgets to bump its revision
/// counter after a real edit would silently be served the OLD document's
/// word/math/page counts under the NEW document's identity claim. That is
/// exactly the failure this crate exists to prevent, so this case must
/// never be trusted from cache, however tempting it is to trust a matching
/// revision id alone.
#[test]
fn same_revision_changed_content_is_a_miss_the_cached_answer_would_be_a_lie() {
    let mut cache = ProjectCache::new();
    let original = vec![SourceItem::text("the proof has three steps")];
    let changed = vec![SourceItem::text(
        "the proof has three steps, plus a fourth step added after the fact",
    )];

    cache
        .update(rev("proof.tex", 1), &original, ScanLimit::UNBOUNDED)
        .unwrap();

    // Same claimed revision (1), but the actual content moved on underneath
    // it. Trusting the cache here would silently under-report a document
    // that has, in reality, grown a whole new step.
    let lookup = cache
        .update(rev("proof.tex", 1), &changed, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(
        !lookup.hit,
        "a reused revision id over changed content must never be trusted as a hit"
    );
    // The returned (and now cached) answer must reflect the NEW content,
    // never the stale one served under a false hit.
    // "the" "proof" "has" "three" "steps," "plus" "a" "fourth" "step"
    // "added" "after" "the" "fact" = 13, not the original's 5.
    assert_eq!(lookup.stats.words().words, 13);
    assert_eq!(
        cache.get("proof.tex").unwrap().words().words,
        lookup.stats.words().words
    );
}

/// Rule 3: changed revision, same content -> MISS. A content match alone
/// does not entitle a caller to a cached answer either: the claimed
/// identity (source + revision number) must match exactly.
#[test]
fn changed_revision_same_content_is_a_miss_identity_must_match_exactly() {
    let mut cache = ProjectCache::new();
    let content = vec![SourceItem::text("this text never changes at all")];

    let first = cache
        .update(rev("stable.tex", 1), &content, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(!first.hit);

    // Byte-for-byte identical content, but the caller now claims revision 2
    // (e.g. a revision counter bumped defensively even though nothing
    // rendered differently). This must still miss: a content match with no
    // matching revision claim is not the trust condition this cache uses.
    let lookup = cache
        .update(rev("stable.tex", 2), &content, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(
        !lookup.hit,
        "identical content under a different claimed revision must still miss"
    );
    assert_eq!(
        lookup.stats.words(),
        first.stats.words(),
        "the recomputed answer still agrees on content"
    );
    assert_eq!(cache.get("stable.tex").unwrap().revision().revision, 2);
}

/// Rule 4: a sibling document's cache entry is untouched by either a hit or
/// a miss on another document. Trusting `a.tex`'s cached answer must never
/// depend on, or be perturbed by, whatever just happened to `b.tex`.
#[test]
fn a_sibling_documents_entry_is_untouched_by_either_a_hit_or_a_miss_elsewhere() {
    let mut cache = ProjectCache::new();
    cache
        .update(
            rev("a.tex", 1),
            &[SourceItem::text("alpha content")],
            ScanLimit::UNBOUNDED,
        )
        .unwrap();
    cache
        .update(
            rev("b.tex", 1),
            &[SourceItem::text("bravo content")],
            ScanLimit::UNBOUNDED,
        )
        .unwrap();
    let b_snapshot = cache.get("b.tex").cloned().unwrap();

    // (a) A HIT on a.tex: re-assert identical identity.
    cache
        .update(
            rev("a.tex", 1),
            &[SourceItem::text("alpha content")],
            ScanLimit::UNBOUNDED,
        )
        .unwrap();
    assert_eq!(
        cache.get("b.tex").cloned().unwrap(),
        b_snapshot,
        "a hit on a sibling document must not touch b.tex's entry"
    );

    // (b) A MISS on a.tex: same revision, changed content (the dangerous
    // case from Rule 2) -- must still leave b.tex completely alone.
    cache
        .update(
            rev("a.tex", 1),
            &[SourceItem::text(
                "alpha content, mutated under the same revision",
            )],
            ScanLimit::UNBOUNDED,
        )
        .unwrap();
    assert_eq!(
        cache.get("b.tex").cloned().unwrap(),
        b_snapshot,
        "a miss on a sibling document must not touch b.tex's entry"
    );

    // (c) Another MISS on a.tex: revision bumped, content changed again --
    // still no effect on b.tex.
    cache
        .update(
            rev("a.tex", 2),
            &[SourceItem::text("alpha content, revised again")],
            ScanLimit::UNBOUNDED,
        )
        .unwrap();
    assert_eq!(
        cache.get("b.tex").cloned().unwrap(),
        b_snapshot,
        "b.tex's entry must survive every kind of update to a.tex, byte for byte"
    );
}

/// The four rules stated together, walked through in one narrative sequence,
/// as a single readable specification of the whole trust contract.
#[test]
fn the_full_trust_contract_in_one_sequence() {
    let mut cache = ProjectCache::new();
    let v1 = vec![SourceItem::text("draft text, version one")];
    let v2 = vec![SourceItem::text(
        "draft text, version two, materially different and longer",
    )];

    // Seed a sibling that must survive everything below.
    cache
        .update(
            rev("sibling.tex", 1),
            &[SourceItem::text("sibling content")],
            ScanLimit::UNBOUNDED,
        )
        .unwrap();
    let sibling_snapshot = cache.get("sibling.tex").cloned().unwrap();

    // Step 1: first sighting, always a miss.
    let step1 = cache
        .update(rev("main.tex", 1), &v1, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(!step1.hit);

    // Step 2 (Rule 1): identical revision + identical content -> trusted hit.
    let step2 = cache
        .update(rev("main.tex", 1), &v1, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(step2.hit);
    assert_eq!(step2.bytes_scanned, 0);

    // Step 3 (Rule 2, the dangerous case): same revision, changed content ->
    // miss, never a stale hit.
    let step3 = cache
        .update(rev("main.tex", 1), &v2, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(!step3.hit);
    assert_eq!(
        step3.stats.words(),
        flashtex_document_statistics::Statistics::compute(rev("main.tex", 1), &v2).words()
    );

    // Step 4 (Rule 3): changed revision, same (now-current) content -> miss.
    let step4 = cache
        .update(rev("main.tex", 2), &v2, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(!step4.hit);

    // Step 5: revision 2 + v2 again -> now THIS identity is a trusted hit.
    let step5 = cache
        .update(rev("main.tex", 2), &v2, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(step5.hit);
    assert_eq!(step5.bytes_scanned, 0);

    // Rule 4 throughout: the sibling never moved.
    assert_eq!(cache.get("sibling.tex").cloned().unwrap(), sibling_snapshot);
}
