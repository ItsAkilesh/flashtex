//! Exact, measured numbers for two of this crate's documented limitations,
//! computed over the repository's real tex corpus
//! (`tests/tex-corpus/cases`) via the real `flashtex-compiler` adapter in
//! `tests/support/mod.rs` -- not described, not estimated.
//!
//! Run `cargo test --test corpus_measurements -- --nocapture` to see the
//! numbers printed as they are computed; the same numbers are asserted
//! below so they cannot silently drift, and are recorded in
//! `coordination/daniel-statistics.md`.

mod support;

use std::collections::HashMap;

use flashtex_document_statistics::{ProjectCache, RevisionId, ScanLimit, SourceItem};

/// Unicode ranges treated as "CJK ideographic/syllabic" for this measurement:
/// Han, Hiragana, Katakana, Hangul syllables. This is a measurement
/// convention for THIS test only, not a claim this crate implements CJK
/// segmentation anywhere.
fn is_cjk(ch: char) -> bool {
    matches!(ch as u32,
        0x4E00..=0x9FFF   // CJK Unified Ideographs (Han)
        | 0x3400..=0x4DBF // CJK Extension A
        | 0x3040..=0x309F // Hiragana
        | 0x30A0..=0x30FF // Katakana
        | 0xAC00..=0xD7A3 // Hangul syllables
    )
}

struct CjkFinding {
    case_name: String,
    token: String,
    cjk_chars: usize,
    words_counted_for_token: usize,
}

/// For one real corpus case, find every `SourceItem::Text` token (i.e. one
/// real compiler word-token) that contains at least one CJK character, and
/// record how many CJK characters it packs versus the exactly one word this
/// crate's rule (documented in `src/words.rs`) counts it as.
fn cjk_findings_for(case: &support::CorpusCase) -> Vec<CjkFinding> {
    let (_parsed, items) = support::adapt(case);
    items
        .iter()
        .filter_map(|item| match item {
            SourceItem::Text(text) => {
                let cjk_chars = text.chars().filter(|c| is_cjk(*c)).count();
                if cjk_chars == 0 {
                    return None;
                }
                let words_counted_for_token =
                    flashtex_document_statistics::WordStats::of(text).words;
                Some(CjkFinding {
                    case_name: case.name.clone(),
                    token: text.clone(),
                    cjk_chars,
                    words_counted_for_token,
                })
            }
            _ => None,
        })
        .collect()
}

#[test]
fn total_documents_and_total_words_across_the_real_corpus() {
    let cases = support::all_cases();
    let total_documents: usize = cases.iter().map(|c| c.documents.len()).sum();

    let mut total_words = 0usize;
    let mut total_math = 0usize;
    let mut total_pages = 0usize;
    for case in &cases {
        let (_parsed, items) = support::adapt(case);
        let stats = flashtex_document_statistics::Statistics::compute(
            RevisionId::new(case.name.clone(), 1),
            &items,
        );
        total_words += stats.words().words;
        total_math += stats.math().total;
        total_pages += stats.pages();
    }

    eprintln!(
        "corpus cases: {}, total .tex documents: {total_documents}, total words: {total_words}, total math items: {total_math}, total pages: {total_pages}",
        cases.len()
    );

    // Exact measured baseline (rev 4). If the corpus changes, re-measure and
    // update both this assertion and coordination/daniel-statistics.md.
    assert_eq!(cases.len(), 14, "number of real corpus cases changed");
    assert_eq!(total_documents, 16, "number of real .tex files changed");
    assert_eq!(
        total_words, 49,
        "total real word count across the corpus changed"
    );
    assert_eq!(
        total_math, 2,
        "total real math item count across the corpus changed"
    );
    assert_eq!(
        total_pages, 14,
        "total real page count across the corpus changed"
    );
}

#[test]
fn exact_cjk_undercount_measured_across_the_real_corpus() {
    let cases = support::all_cases();
    let mut findings_by_case: HashMap<String, Vec<CjkFinding>> = HashMap::new();
    for case in &cases {
        let findings = cjk_findings_for(case);
        if !findings.is_empty() {
            findings_by_case.insert(case.name.clone(), findings);
        }
    }

    let mut affected_case_names: Vec<&String> = findings_by_case.keys().collect();
    affected_case_names.sort();

    let mut total_undercount = 0usize;
    for name in &affected_case_names {
        for finding in &findings_by_case[*name] {
            // "Undercount" = real CJK characters in this one token minus the
            // exactly-one word this crate's rule counts it as. A lone CJK
            // character in its own whitespace-delimited token (cjk_chars ==
            // 1) is NOT undercounted by this measure -- one character
            // counted as one word is not wrong on its own; the limitation
            // is specifically multi-character CJK runs collapsing to one
            // token.
            let undercount = finding.cjk_chars.saturating_sub(1);
            total_undercount += undercount;
            eprintln!(
                "CJK finding: case={:?} token={:?} cjk_chars={} words_counted={} undercount={}",
                finding.case_name,
                finding.token,
                finding.cjk_chars,
                finding.words_counted_for_token,
                undercount
            );
        }
    }

    eprintln!(
        "documents (cases) containing CJK script: {} of {}; total undercount across them: {} words",
        affected_case_names.len(),
        cases.len(),
        total_undercount
    );

    // Exact measured baseline (rev 4): the real corpus is small, so this is
    // a small, real number -- not scaled up or estimated.
    assert_eq!(
        affected_case_names,
        vec!["literal-source-map", "unicode-literals"],
        "set of real corpus cases containing CJK script changed"
    );

    // literal-source-map/main.tex contains "尾" alone on its own line: one
    // CJK character in its own token. cjk_chars == 1, so undercount == 0 --
    // this case demonstrates a CJK token that is NOT undercounted by this
    // measure, and is reported as such rather than omitted.
    let literal_source_map = &findings_by_case["literal-source-map"];
    assert_eq!(literal_source_map.len(), 1);
    assert_eq!(literal_source_map[0].token, "尾");
    assert_eq!(literal_source_map[0].cjk_chars, 1);
    assert_eq!(literal_source_map[0].words_counted_for_token, 1);

    // unicode-literals/main.tex contains "東京." as one real compiler
    // word-token (the lexer stops a word only at whitespace or a special
    // TeX character -- '.' is neither): 2 real CJK characters, counted as
    // exactly 1 word by this crate's rule. Measured undercount: 1 word (the
    // token is undercounted by half relative to a per-character CJK count).
    let unicode_literals = &findings_by_case["unicode-literals"];
    assert_eq!(unicode_literals.len(), 1);
    assert_eq!(unicode_literals[0].token, "東京.");
    assert_eq!(unicode_literals[0].cjk_chars, 2);
    assert_eq!(unicode_literals[0].words_counted_for_token, 1);

    assert_eq!(
        total_undercount, 1,
        "total measured CJK undercount across the real corpus changed"
    );
}

/// A realistic edit sequence over the whole real corpus: every "document" is
/// one real corpus case's adapted `SourceItem`s. Across 30 rounds, exactly
/// one document is edited per round (round-robin over all 14 real cases,
/// toggling between its real unedited content and a real "edited" variant --
/// see `edited_variant` below); every other document is resubmitted
/// unchanged. This is exactly the realistic shape rev 2's incremental-equals-
/// fresh property test already established (recompute the whole project on
/// every edit, but only the touched document should actually be rescanned);
/// this test measures the ACTUAL hit rate and bytes saved that shape
/// produces over real content, rather than asserting only that incremental
/// equals fresh.
///
/// The "edited" variant of a document is still real content: it is that
/// document's own real items with one more real `SourceItem` appended, taken
/// verbatim from a different real corpus case (the first math item found
/// anywhere in the corpus). No text in this test is hand-invented.
#[test]
fn measured_cache_hit_rate_and_bytes_saved_over_a_realistic_edit_sequence() {
    let cases = support::all_cases();
    let real_items: Vec<(String, Vec<SourceItem>)> = cases
        .iter()
        .map(|case| (case.name.clone(), support::adapt(case).1))
        .collect();

    // A real math SourceItem borrowed from elsewhere in the real corpus, to
    // build a real (not invented) "edited" variant of each document.
    let borrowed_real_math = real_items
        .iter()
        .flat_map(|(_, items)| items.iter())
        .find(|item| matches!(item, SourceItem::Math(_)))
        .cloned()
        .expect("the real corpus contains at least one math item to borrow");

    let edited_variant = |base: &[SourceItem]| -> Vec<SourceItem> {
        let mut edited = base.to_vec();
        edited.push(borrowed_real_math.clone());
        edited
    };

    const ROUNDS: usize = 30;
    let mut cache = ProjectCache::new();
    let mut revision_of: HashMap<&str, u64> = HashMap::new();
    let mut edited_of: HashMap<&str, bool> = HashMap::new();
    for (name, _) in &real_items {
        revision_of.insert(name.as_str(), 1);
        edited_of.insert(name.as_str(), false);
    }

    let mut total_updates = 0usize;
    let mut hits = 0usize;
    let mut incremental_bytes_scanned = 0usize;
    let mut fresh_equivalent_bytes_scanned = 0usize;

    for round in 0..ROUNDS {
        let edit_target = if round == 0 {
            None // round 0 is the initial population: every document is a miss.
        } else {
            Some((round - 1) % real_items.len())
        };
        for (index, (name, base_items)) in real_items.iter().enumerate() {
            if Some(index) == edit_target {
                *edited_of.get_mut(name.as_str()).unwrap() ^= true;
                *revision_of.get_mut(name.as_str()).unwrap() += 1;
            }
            let items = if edited_of[name.as_str()] {
                edited_variant(base_items)
            } else {
                base_items.clone()
            };
            let revision = RevisionId::new(name.clone(), revision_of[name.as_str()]);
            let lookup = cache
                .update(revision, &items, ScanLimit::default())
                .expect("real corpus content stays within the default scan limit");

            total_updates += 1;
            if lookup.hit {
                hits += 1;
            }
            incremental_bytes_scanned += lookup.bytes_scanned;
            fresh_equivalent_bytes_scanned += lookup.stats.scanned_bytes();
        }
    }

    let hit_rate = hits as f64 / total_updates as f64;
    let bytes_saved = fresh_equivalent_bytes_scanned - incremental_bytes_scanned;
    let misses = total_updates - hits;
    eprintln!(
        "cache measurement over {ROUNDS} rounds x {} documents = {total_updates} updates: \
         hits={hits} misses={misses} hit_rate={hit_rate:.4} incremental_bytes={incremental_bytes_scanned} \
         fresh_equivalent_bytes={fresh_equivalent_bytes_scanned} bytes_saved={bytes_saved}",
        real_items.len(),
    );

    // Exact measured baseline (rev 4): 14 documents, 1 edit/round, 30 rounds.
    assert_eq!(total_updates, ROUNDS * real_items.len());
    assert_eq!(hits, 377);
    assert_eq!(total_updates - hits, 43);
    assert!(
        (hit_rate - 0.897_619_047_6).abs() < 1e-9,
        "hit_rate={hit_rate}"
    );
    assert_eq!(fresh_equivalent_bytes_scanned, 11137);
    assert_eq!(incremental_bytes_scanned, 1087);
    assert_eq!(bytes_saved, 10050);
}
