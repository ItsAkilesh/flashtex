//! Tests for `SnippetPlan`: document identity, staleness detection,
//! Unicode-safe caret/selection anchors, placeholder preservation, and the
//! never-mutates-a-document contract.

use std::collections::HashMap;

use flashtex_editor_snippets::{Anchor, DocumentId, PlanError, Snippet, SnippetPlan, Staleness};

fn plan(src: &str, revision: u64, doc: &str, anchor: Anchor) -> Result<SnippetPlan, PlanError> {
    let snippet = Snippet::parse(src).unwrap();
    SnippetPlan::compute(&snippet, revision, doc, anchor, &HashMap::new())
}

// ---------------------------------------------------------------------
// 1. Document identity and staleness
// ---------------------------------------------------------------------

#[test]
fn fresh_plan_compares_fresh_against_the_same_document_identity() {
    let doc = "func ()";
    let p = plan("${1:name}", 1, doc, Anchor::Caret(5)).unwrap();
    let current = DocumentId::new(1, doc);
    assert_eq!(p.staleness(&current), Staleness::Fresh);
    assert!(p.staleness(&current).is_fresh());
}

#[test]
fn revision_bump_is_detected_as_stale() {
    let doc = "func ()";
    let p = plan("${1:name}", 1, doc, Anchor::Caret(5)).unwrap();
    let current = DocumentId::new(2, doc);
    let staleness = p.staleness(&current);
    assert!(!staleness.is_fresh());
    match staleness {
        Staleness::RevisionMismatch { planned, current } => {
            assert_eq!(planned, 1);
            assert_eq!(current, 2);
        }
        other => panic!("expected RevisionMismatch, got {other:?}"),
    }
}

/// The critical case: the caller's revision counter did NOT change, but
/// the document's bytes did underneath it. A staleness check that only
/// compared `revision` would wrongly call this fresh; the content hash
/// must catch it.
#[test]
fn bytes_changed_but_revision_id_did_not_is_still_detected_as_stale() {
    let original = "func ()";
    let edited = "func (edited)";
    let p = plan("${1:name}", 7, original, Anchor::Caret(5)).unwrap();

    // Same revision id (7) as the plan was computed against...
    let current = DocumentId::new(7, edited);
    let staleness = p.staleness(&current);

    // ...but the bytes differ, so this must not be reported as Fresh.
    assert!(!staleness.is_fresh());
    match staleness {
        Staleness::ContentMismatch { planned, current } => {
            assert_ne!(planned, current);
        }
        other => panic!("expected ContentMismatch, got {other:?}"),
    }
}

#[test]
fn identical_revision_and_identical_bytes_hash_equal() {
    let a = DocumentId::new(3, "hello world");
    let b = DocumentId::new(3, "hello world");
    assert_eq!(a, b);
    assert_eq!(a.compare(&b), Staleness::Fresh);
}

#[test]
fn content_hash_differs_for_different_bytes_with_the_same_length() {
    let a = DocumentId::new(1, "aaaa");
    let b = DocumentId::new(1, "aaab");
    assert_ne!(a.content_hash, b.content_hash);
}

// ---------------------------------------------------------------------
// 2. Unicode-safe caret and selection ranges
// ---------------------------------------------------------------------

#[test]
fn caret_at_a_valid_char_boundary_in_cjk_text_succeeds() {
    // "你好" is 3 bytes per character: valid boundaries are 0, 3, 6.
    let doc = "你好";
    assert!(plan("$1", 1, doc, Anchor::Caret(0)).is_ok());
    assert!(plan("$1", 1, doc, Anchor::Caret(3)).is_ok());
    assert!(plan("$1", 1, doc, Anchor::Caret(6)).is_ok());
}

#[test]
fn caret_mid_character_in_cjk_text_is_a_typed_error_not_a_panic() {
    let doc = "你好";
    let err = plan("$1", 1, doc, Anchor::Caret(1)).unwrap_err();
    assert_eq!(err, PlanError::InvalidOffset { offset: 1 });
}

#[test]
fn caret_mid_character_in_accented_latin_text_is_rejected() {
    // "café" - 'é' is a 2-byte char starting at offset 3; offset 4 is
    // mid-character.
    let doc = "café";
    assert!(plan("$1", 1, doc, Anchor::Caret(3)).is_ok());
    let err = plan("$1", 1, doc, Anchor::Caret(4)).unwrap_err();
    assert_eq!(err, PlanError::InvalidOffset { offset: 4 });
    // The end of the string (5 bytes: c-a-f-é(2 bytes)) is a valid caret.
    assert!(plan("$1", 1, doc, Anchor::Caret(doc.len())).is_ok());
}

#[test]
fn selection_around_a_multi_codepoint_emoji_sequence_is_unicode_safe() {
    // Family emoji built from a ZWJ sequence: multiple 4-byte and 3-byte
    // codepoints joined by U+200D. Only the boundaries between whole
    // codepoints are valid.
    let emoji = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}"; // man-ZWJ-woman-ZWJ-girl
    let doc = format!("before {emoji} after");
    let start = doc.find(emoji).unwrap();
    let end = start + emoji.len();

    // Selecting exactly the whole sequence succeeds.
    let ok = plan("$1", 1, &doc, Anchor::Selection(start..end));
    assert!(ok.is_ok(), "{ok:?}");

    // Splitting inside the first 4-byte codepoint is rejected.
    let mid = start + 1;
    let err = plan("$1", 1, &doc, Anchor::Selection(start..mid)).unwrap_err();
    assert_eq!(err, PlanError::InvalidOffset { offset: mid });
}

#[test]
fn caret_past_the_end_of_the_document_is_rejected() {
    let doc = "short";
    let err = plan("$1", 1, doc, Anchor::Caret(doc.len() + 1)).unwrap_err();
    assert_eq!(
        err,
        PlanError::InvalidOffset {
            offset: doc.len() + 1
        }
    );
}

#[test]
fn reversed_selection_is_a_typed_error() {
    let doc = "hello world";
    // Built from variables, not a range literal, so the reversal isn't a
    // `clippy::reversed_empty_ranges` lint trigger - it's exactly the
    // caller mistake (start/end swapped) this check exists to catch.
    let (start, end) = (5usize, 2usize);
    let err = plan("$1", 1, doc, Anchor::Selection(start..end)).unwrap_err();
    assert_eq!(err, PlanError::SelectionReversed { start: 5, end: 2 });
}

#[test]
fn anchor_range_reports_caret_and_selection_correctly() {
    assert_eq!(Anchor::Caret(4).range(), 4..4);
    assert_eq!(Anchor::Selection(2..9).range(), 2..9);
}

// ---------------------------------------------------------------------
// 3. Placeholders, linked edits, and tab order still work through a plan
// ---------------------------------------------------------------------

#[test]
fn plan_expansion_preserves_linked_placeholder_offsets() {
    let doc = "";
    let snippet = Snippet::parse("Hi, ${1:World}! $1 again, $1.").unwrap();
    let p = SnippetPlan::compute(&snippet, 1, doc, Anchor::Caret(0), &HashMap::new()).unwrap();
    assert_eq!(p.expansion().text, "Hi, World! World again, World.");
    assert_eq!(p.expansion().occurrences_of(1).len(), 3);
}

#[test]
fn plan_compute_applies_overrides_for_a_linked_edit() {
    let doc = "";
    let snippet = Snippet::parse("Hi, ${1:World}! $1 again.").unwrap();
    let mut overrides = HashMap::new();
    overrides.insert(1, "Rust".to_string());
    let p = SnippetPlan::compute(&snippet, 1, doc, Anchor::Caret(0), &overrides).unwrap();
    assert_eq!(p.expansion().text, "Hi, Rust! Rust again.");
}

#[test]
fn plan_expansion_tab_order_matches_direct_expand() {
    let doc = "";
    let snippet = Snippet::parse("$3 $1 $0 $2").unwrap();
    let direct = snippet.expand().unwrap();
    let p = SnippetPlan::compute(&snippet, 1, doc, Anchor::Caret(0), &HashMap::new()).unwrap();
    assert_eq!(p.expansion().tab_order(), direct.tab_order());
    assert_eq!(p.expansion().tab_order(), vec![1, 2, 3, 0]);
}

#[test]
fn plan_still_reports_self_referential_placeholders_as_a_typed_error() {
    let doc = "";
    let snippet = Snippet::parse("${1:$1}").unwrap();
    let err =
        SnippetPlan::compute(&snippet, 1, doc, Anchor::Caret(0), &HashMap::new()).unwrap_err();
    match err {
        PlanError::Snippet(flashtex_editor_snippets::SnippetError::SelfReferential { index }) => {
            assert_eq!(index, 1);
        }
        other => panic!("expected wrapped SelfReferential, got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// 4. A SnippetPlan carries the anchor and document id it was built with,
//    and exposes no way to apply itself to anything
// ---------------------------------------------------------------------

#[test]
fn plan_exposes_the_document_and_anchor_it_was_computed_against() {
    let doc = "func ()";
    let snippet = Snippet::parse("${1:name}").unwrap();
    let p = SnippetPlan::compute(&snippet, 42, doc, Anchor::Caret(5), &HashMap::new()).unwrap();
    assert_eq!(p.document(), &DocumentId::new(42, doc));
    assert_eq!(p.anchor(), &Anchor::Caret(5));
}
