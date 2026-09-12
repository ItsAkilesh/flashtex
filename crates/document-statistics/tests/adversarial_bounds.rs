//! Adversarial bounds acceptance: attack the scanner and the cache directly.
//!
//! Every case here must resolve to either a typed error (`ScanTooLarge`) or a
//! bounded, correct `Result::Ok` value. None of them may panic, hang, or let
//! a cache grow without bound. Where the crate's honesty guarantee is at
//! stake (exact byte-limit boundary, exact error variants), assertions check
//! exact fields, not just "it didn't crash".

use flashtex_document_statistics::{
    ProjectCache, RevisionId, ScanLimit, ScanTooLarge, SourceItem, Statistics,
};

fn rev(source: &str, n: u64) -> RevisionId {
    RevisionId::new(source, n)
}

// --- A document at and past the scan-byte limit: the boundary binds exactly ---

#[test]
fn scan_exactly_at_the_limit_is_accepted() {
    // Exactly `limit` bytes scanned must succeed: the bound is "may not
    // exceed", not "must stay strictly under".
    let text = "a".repeat(100);
    assert_eq!(text.len(), 100);
    let items = vec![SourceItem::text(text)];
    let stats = Statistics::compute_bounded(rev("edge.tex", 1), &items, ScanLimit::new(100))
        .expect("scanning exactly up to the limit must be accepted, not rejected");
    assert_eq!(stats.scanned_bytes, 100);
}

#[test]
fn scan_one_byte_past_the_limit_is_a_typed_error_with_exact_fields() {
    let text = "a".repeat(101);
    let items = vec![SourceItem::text(text)];
    let err = Statistics::compute_bounded(rev("edge.tex", 1), &items, ScanLimit::new(100))
        .expect_err("one byte past the limit must be rejected");
    assert_eq!(
        err,
        ScanTooLarge {
            scanned: 101,
            limit: 100
        }
    );
}

#[test]
fn scan_limit_of_zero_rejects_any_nonempty_content_but_accepts_empty() {
    // Zero is a legal (if extreme) limit: it must behave exactly like any
    // other limit at its own boundary, not panic on an edge value.
    let empty: Vec<SourceItem> = vec![SourceItem::text(""), SourceItem::PageMark];
    let stats = Statistics::compute_bounded(rev("zero.tex", 1), &empty, ScanLimit::new(0))
        .expect("zero scanned bytes is within a zero limit");
    assert_eq!(stats.scanned_bytes, 0);
    assert_eq!(stats.pages, 1);

    let nonempty = vec![SourceItem::text("x")];
    let err = Statistics::compute_bounded(rev("zero.tex", 2), &nonempty, ScanLimit::new(0))
        .expect_err("any scanned byte must exceed a zero limit");
    assert_eq!(err.scanned, 1);
    assert_eq!(err.limit, 0);
}

// --- A project with thousands of documents ---

#[test]
fn a_project_with_thousands_of_documents_stays_correct_and_bounded() {
    const N: usize = 5_000;
    let mut cache = ProjectCache::new();
    for i in 0..N {
        let source = format!("doc-{i}.tex");
        let items = vec![SourceItem::text("one two three")];
        let lookup = cache
            .update(rev(&source, 1), &items, ScanLimit::UNBOUNDED)
            .unwrap_or_else(|e| panic!("document {i} unexpectedly rejected: {e}"));
        assert!(!lookup.hit, "first sighting of {source} must be a miss");
    }
    assert_eq!(cache.len(), N);
    let totals = cache.project_totals();
    assert_eq!(totals.words.words, 3 * N);

    // Re-running every update is now all hits, and the cache does not grow:
    // one entry per distinct source, not one per call.
    for i in 0..N {
        let source = format!("doc-{i}.tex");
        let items = vec![SourceItem::text("one two three")];
        let lookup = cache
            .update(rev(&source, 1), &items, ScanLimit::UNBOUNDED)
            .unwrap();
        assert!(lookup.hit);
    }
    assert_eq!(cache.len(), N);
}

// --- A document with a single enormous item ---

#[test]
fn a_single_enormous_text_item_is_bounded_by_scan_limit_not_by_luck() {
    // One item, ~2 MB, far larger than any of the small-fixture tests
    // elsewhere in this crate. A limit smaller than the item alone must
    // reject it with an exact scanned count -- the limit must bind even
    // when a single item, not an accumulation of many, is what crosses it.
    let huge = "word ".repeat(400_000); // 2,000,000 bytes
    let len = huge.len();
    let items = vec![SourceItem::text(huge)];

    let err = Statistics::compute_bounded(rev("huge.tex", 1), &items, ScanLimit::new(1_000))
        .expect_err("a single oversized item must still be rejected");
    assert_eq!(err.scanned, len);
    assert_eq!(err.limit, 1_000);

    // The same enormous item, unbounded, must still terminate, produce a
    // correct count, and not panic -- "enormous" is not "unbounded".
    let items = vec![SourceItem::text("word ".repeat(400_000))];
    let stats = Statistics::compute(rev("huge.tex", 1), &items);
    assert_eq!(stats.words.words, 400_000);
    assert_eq!(stats.scanned_bytes, len);
}

// --- Text of only combining marks or zero-width characters ---

#[test]
fn text_of_only_combining_marks_is_bounded_and_not_a_word() {
    // A base-less run of combining acute accents: no alphanumeric scalar
    // value anywhere in the token, so it must not count as a word, but it
    // must also not panic or be silently dropped from `chars`.
    let text: String = std::iter::repeat_n('\u{0301}', 50).collect();
    let items = vec![SourceItem::text(text.clone())];
    let stats = Statistics::compute(rev("marks.tex", 1), &items);
    assert_eq!(stats.words.words, 0);
    assert_eq!(stats.words.chars, 50);
    assert_eq!(stats.scanned_bytes, text.len());
}

#[test]
fn text_of_only_zero_width_characters_is_bounded_and_not_a_word() {
    // ZERO WIDTH SPACE, ZERO WIDTH NON-JOINER, ZERO WIDTH JOINER: none are
    // Unicode `White_Space`, so this is a SINGLE non-empty token under this
    // crate's whitespace-splitting rule, and none is alphanumeric, so it is
    // not a word.
    let text = "\u{200B}\u{200C}\u{200D}\u{200B}";
    let items = vec![SourceItem::text(text)];
    let stats = Statistics::compute(rev("zwsp.tex", 1), &items);
    assert_eq!(stats.words.words, 0);
    assert_eq!(stats.words.chars, 4);
    assert_eq!(stats.scanned_bytes, text.len());
}

#[test]
fn mix_of_combining_marks_and_zero_width_characters_across_many_items_is_bounded() {
    // Many small adversarial items back to back: must sum correctly, scan
    // exactly the claimed number of bytes, and never panic.
    let pathological = "\u{0301}\u{200D}\u{0300}\u{200B}";
    let items: Vec<SourceItem> = (0..1_000).map(|_| SourceItem::text(pathological)).collect();
    let expected_bytes = pathological.len() * 1_000;
    let stats = Statistics::compute(rev("pathological.tex", 1), &items);
    assert_eq!(stats.words.words, 0);
    assert_eq!(stats.scanned_bytes, expected_bytes);
}

// --- A revision id at integer maximum ---

#[test]
fn a_revision_id_at_u64_max_behaves_exactly_like_any_other_revision() {
    let items = vec![SourceItem::text("hello world")];
    let revision = rev("max.tex", u64::MAX);
    let stats = Statistics::compute(revision.clone(), &items);
    assert_eq!(stats.revision.revision, u64::MAX);
    assert!(stats.is_current_for(&revision, &items));
    // Not equal to a "wrapped around" revision 0 -- there must be no
    // silent wraparound anywhere near this boundary.
    assert!(!stats.is_current_for(&rev("max.tex", 0), &items));

    let mut cache = ProjectCache::new();
    let lookup = cache
        .update(revision.clone(), &items, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(!lookup.hit);
    let repeat = cache
        .update(revision, &items, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(
        repeat.hit,
        "u64::MAX revision must still cache-hit normally"
    );
}

// --- The same document id registered twice with different content ---

#[test]
fn same_document_id_registered_twice_with_different_content_never_duplicates_the_entry() {
    let mut cache = ProjectCache::new();
    let source = "dup.tex";

    // Register the SAME source id many times in a row, each time with a
    // different revision AND different content -- an attempt to make the
    // cache accumulate stale duplicate entries under one key.
    for n in 1..=200u64 {
        let content = format!("revision number {n} has unique content {n}{n}{n}");
        cache
            .update(
                rev(source, n),
                &[SourceItem::text(content)],
                ScanLimit::UNBOUNDED,
            )
            .unwrap();
        // Exactly one entry ever exists for this source, no matter how many
        // times it is re-registered with different content.
        assert_eq!(cache.len(), 1);
    }
    assert!(cache.contains(source));
    assert_eq!(cache.get(source).unwrap().revision.revision, 200);
}

#[test]
fn same_document_id_same_revision_different_content_is_a_miss_not_corruption() {
    // The specific dangerous sub-case: revision number NOT bumped, content
    // silently changed underneath it. Must miss (see stale identity
    // acceptance suite) and must leave exactly one, correct, entry behind --
    // never two entries, never a merged/corrupted stats value.
    let mut cache = ProjectCache::new();
    cache
        .update(
            rev("dup2.tex", 1),
            &[SourceItem::text("first content")],
            ScanLimit::UNBOUNDED,
        )
        .unwrap();
    let lookup = cache
        .update(
            rev("dup2.tex", 1),
            &[SourceItem::text("totally different second content, longer")],
            ScanLimit::UNBOUNDED,
        )
        .unwrap();
    assert!(!lookup.hit);
    assert_eq!(cache.len(), 1);
    assert_eq!(
        cache.get("dup2.tex").unwrap().words.words,
        lookup.stats.words.words
    );
}

// --- An empty project ---

#[test]
fn an_empty_project_answers_every_query_with_a_bounded_default_never_a_panic() {
    let cache = ProjectCache::new();
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
    assert_eq!(cache.get("anything.tex"), None);
    assert!(!cache.contains("anything.tex"));

    let totals = cache.project_totals();
    assert_eq!(totals.words.words, 0);
    assert_eq!(totals.math.total, 0);
    assert_eq!(totals.pages, 0);
}

// --- A cache repeatedly updated many times: entries do not grow without bound ---

#[test]
fn repeatedly_updating_one_document_never_grows_the_cache_past_one_entry() {
    let mut cache = ProjectCache::new();
    for n in 1..=20_000u64 {
        cache
            .update(
                rev("churn.tex", n),
                &[SourceItem::text(format!("content at revision {n}"))],
                ScanLimit::UNBOUNDED,
            )
            .unwrap();
        assert_eq!(
            cache.len(),
            1,
            "cache must hold exactly one entry per distinct source, not one per update"
        );
    }
}

#[test]
fn repeatedly_updating_a_fixed_set_of_documents_never_grows_the_cache_past_that_set() {
    const N_DOCS: usize = 10;
    const N_UPDATES: usize = 5_000;
    let mut cache = ProjectCache::new();
    let mut revisions = [0u64; N_DOCS];
    for update_n in 0..N_UPDATES {
        let doc = update_n % N_DOCS;
        revisions[doc] += 1;
        let source = format!("fixed-{doc}.tex");
        cache
            .update(
                rev(&source, revisions[doc]),
                &[SourceItem::text(format!("update {update_n}"))],
                ScanLimit::UNBOUNDED,
            )
            .unwrap();
        assert!(
            cache.len() <= N_DOCS,
            "cache grew past the fixed document set: {} entries after {} updates",
            cache.len(),
            update_n + 1
        );
    }
    assert_eq!(cache.len(), N_DOCS);
}
