//! Integration fixture: `SourceItem`s built by adapting the REAL output of
//! `flashtex-compiler` (`crates/compiler`) over the repository's real tex
//! corpus (`tests/tex-corpus/cases`), not hand-written items shaped for
//! convenience.
//!
//! Repository-wide search performed for this fixture (rev 4): `flashtex-compiler`
//! is the only crate under `crates/` that produces a document/item structure at
//! all (`parser::Parsed` / `Block` / `Inline`, and `layout::Page` /
//! `TextItem`). `document-runtime` is a transport layer around an external
//! compiler process and defines no item model of its own; no other crate
//! (`rendering-core`, `pdf`, etc.) parses `.tex` source. No crate in this
//! repository currently constructs `flashtex_document_statistics::SourceItem`
//! in production -- see `coordination/daniel-statistics.md` for that status,
//! unchanged by this revision. `crates/compiler` is used here strictly as a
//! read-only, dev-dependency-only producer (see `tests/support/mod.rs`); it is
//! not edited, and `flashtex-document-statistics`'s own `src/` gains no new
//! dependency.

mod support;

use flashtex_document_statistics::{RevisionId, ScanLimit, SourceItem, Statistics};

fn item_len(item: &SourceItem) -> usize {
    match item {
        SourceItem::Text(t) => t.len(),
        SourceItem::Math(m) => m.source.len(),
        SourceItem::PageMark => 0,
    }
}

#[test]
fn every_real_corpus_case_computes_bounded_statistics_without_panicking() {
    let cases = support::all_cases();
    assert!(
        cases.len() >= 10,
        "expected the real tests/tex-corpus/cases fixtures to be present, found {}",
        cases.len()
    );
    for case in &cases {
        let (_parsed, items) = support::adapt(case);
        let revision = RevisionId::new(case.name.clone(), 1);
        let stats = Statistics::compute_bounded(revision, &items, ScanLimit::default())
            .unwrap_or_else(|e| {
                panic!(
                    "real corpus case '{}' exceeded the default scan limit: {e}",
                    case.name
                )
            });
        let real_content_bytes: usize = items.iter().map(item_len).sum();
        assert!(
            stats.scanned_bytes <= real_content_bytes,
            "case '{}': scanned {} bytes but real content is only {} bytes",
            case.name,
            stats.scanned_bytes,
            real_content_bytes
        );
    }
}

#[test]
fn plain_paragraphs_case_word_count_matches_the_real_compiler_output() {
    let case = support::load_case(&support::corpus_root().join("plain-paragraphs"));
    let (parsed, items) = support::adapt(&case);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let stats = Statistics::compute(RevisionId::new("plain-paragraphs", 1), &items);
    // "First paragraph." "Second paragraph." -- 4 real compiler word-tokens.
    assert_eq!(stats.words.words, 4);
    assert_eq!(stats.pages, 1);
    assert_eq!(stats.math.total, 0);
}

#[test]
fn math_inline_display_case_math_is_counted_and_never_word_counted() {
    let case = support::load_case(&support::corpus_root().join("math-inline-display"));
    let (parsed, items) = support::adapt(&case);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let stats = Statistics::compute(RevisionId::new("math-inline-display", 1), &items);
    assert_eq!(stats.math.total, 2);
    assert_eq!(stats.math.inline, 1);
    assert_eq!(stats.math.display, 1);
    // "Inline" ... "ends." around the real math span; math source itself
    // never contributes to the word count.
    assert_eq!(stats.words.words, 2);
    // The real math source was sliced verbatim from the actual document
    // text via the compiler's own span, not reconstructed -- delimiters
    // included, exactly as the compiler's `Inline::Math` span covers them.
    let math_sources: Vec<&str> = items
        .iter()
        .filter_map(|item| match item {
            SourceItem::Math(m) => Some(m.source.as_str()),
            _ => None,
        })
        .collect();
    assert!(math_sources.contains(&"$x_1^2 + y$"));
    assert!(math_sources.contains(&"\\[\\frac{a+b}{c}=d\\]"));
}

#[test]
fn included_file_case_pulls_prose_from_the_actually_included_document() {
    let case = support::load_case(&support::corpus_root().join("included-file"));
    let (parsed, items) = support::adapt(&case);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let texts: Vec<&str> = items
        .iter()
        .filter_map(|item| match item {
            SourceItem::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    // "Before." from main.tex, the included section's real content, then
    // "After." from main.tex again -- the compiler's own \input resolution,
    // not a fixture that inlines the included text by hand.
    assert!(texts.contains(&"Before."));
    assert!(texts.contains(&"After."));
    let included_source = &case
        .documents
        .iter()
        .find(|(path, _)| path == "parts/section.tex")
        .expect("included-file corpus case ships parts/section.tex")
        .1;
    let included_word = included_source.split_whitespace().next().unwrap();
    assert!(
        texts
            .iter()
            .any(|t| t.contains(included_word.trim_end_matches('.'))),
        "expected a word from the actually-included file's real text ({included_word:?}) among {texts:?}"
    );
}

#[test]
fn editing_the_real_source_file_is_reflected_through_is_current_for() {
    // Read the real fixture, edit its actual text, and re-run the edited
    // text back through the real compiler -- not a hand-written "changed"
    // string standing in for an edit.
    let mut case = support::load_case(&support::corpus_root().join("plain-paragraphs"));
    let (_, items) = support::adapt(&case);
    let revision = RevisionId::new("plain-paragraphs", 1);
    let stats = Statistics::compute(revision.clone(), &items);
    assert!(stats.is_current_for(&revision, &items));

    let main = case
        .documents
        .iter_mut()
        .find(|(path, _)| path == "main.tex")
        .expect("plain-paragraphs corpus case ships main.tex");
    assert!(main.1.contains("First paragraph."));
    main.1 = main
        .1
        .replace("First paragraph.", "First paragraph, edited.");

    let (_, edited_items) = support::adapt(&case);
    assert!(
        !stats.is_current_for(&revision, &edited_items),
        "editing the real source file must be visible as staleness under the same revision id"
    );
}
