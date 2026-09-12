//! Integration tests against the public API only.

use flashtex_document_statistics::{RevisionId, SourceItem, Statistics};

#[test]
fn hand_worked_document_with_two_pages() {
    let items = vec![
        SourceItem::PageMark,
        SourceItem::text("The quick brown fox jumps over the lazy dog."),
        SourceItem::display_math("E = mc^2"),
        SourceItem::PageMark,
        SourceItem::text("It don't matter, said the well-known author O'Brien."),
        SourceItem::inline_math("\\alpha + \\beta"),
    ];
    let stats = Statistics::compute(RevisionId::new("paper.tex", 7), &items);

    // Page 1: "The" "quick" "brown" "fox" "jumps" "over" "the" "lazy" "dog." = 9
    // Page 2: "It" "don't" "matter," "said" "the" "well-known" "author" "O'Brien." = 8
    assert_eq!(stats.words.words, 17);
    assert_eq!(stats.math.total, 2);
    assert_eq!(stats.math.display, 1);
    assert_eq!(stats.math.inline, 1);
    assert_eq!(stats.pages, 2);
    assert_eq!(stats.revision, RevisionId::new("paper.tex", 7));
}

#[test]
fn unicode_document_documents_its_own_limitation() {
    // A CJK sentence has no whitespace, so it is one "word" under this
    // crate's stated rule -- badly undercounting what a reader would call
    // word count. `chars` gives an honest, script-agnostic size instead.
    let items = vec![SourceItem::text("你好，世界！这是一个测试。")];
    let stats = Statistics::compute(RevisionId::new("cjk.tex", 1), &items);
    assert_eq!(stats.words.words, 1);
    assert_eq!(
        stats.words.chars,
        "你好，世界！这是一个测试。".chars().count()
    );
}

#[test]
fn malformed_input_no_page_marks_and_blank_text_items() {
    let items = vec![
        SourceItem::text(""),
        SourceItem::text("   \n\t  "),
        SourceItem::text("real words here"),
    ];
    let stats = Statistics::compute(RevisionId::new("draft.tex", 1), &items);
    assert_eq!(stats.words.words, 3);
    assert_eq!(stats.pages, 0);
}

#[test]
fn revision_binding_survives_across_the_public_api() {
    let items = vec![SourceItem::text("stable content")];
    let stats = Statistics::compute(RevisionId::new("locked.tex", 42), &items);

    // Correct revision, correct content: current.
    assert!(stats.is_current_for(&RevisionId::new("locked.tex", 42), &items));

    // Content silently changed under the same claimed revision: stale.
    let mutated = vec![SourceItem::text("stable content, mutated")];
    assert!(!stats.is_current_for(&RevisionId::new("locked.tex", 42), &mutated));

    // Revision bumped, content unchanged: the old stats are still stale
    // for the new revision (they were computed for revision 42, not 43).
    assert!(!stats.is_current_for(&RevisionId::new("locked.tex", 43), &items));
}
