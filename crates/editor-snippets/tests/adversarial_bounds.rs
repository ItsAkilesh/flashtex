//! Adversarial acceptance tests for the full snippet-to-plan pipeline.
//!
//! Every case below is driven through [`attempt_plan`], which chains
//! `Snippet::parse` -> `Snippet::expand_with` -> `SnippetPlan::compute`
//! exactly as a real caller would: parse a template, then bind it to a
//! document, caret/selection, and override set. This is "attacking plan
//! computation" - the full pipeline, not just one internal stage - and
//! every case here must resolve to a typed `PlanError` (never panic, never
//! hang), matching a specific documented variant.
//!
//! Every *numeric* bound is tested as a pair: a template exactly at the
//! documented limit succeeds, and one unit past it fails with the
//! documented error. A bound that is only checked as "exactly right" would
//! hide an off-by-one; testing both sides of the line is what actually
//! proves the bound binds where it claims to.

use std::collections::HashMap;

use flashtex_editor_snippets::{
    Anchor, MAX_INPUT_BYTES, MAX_NESTING_DEPTH, MAX_OCCURRENCES, MAX_OUTPUT_BYTES,
    MAX_PLACEHOLDER_INDEX, MAX_PLACEHOLDERS, PlanError, Snippet, SnippetError, SnippetPlan,
};

/// Drives `src` through the entire public pipeline: parse, expand with
/// `overrides`, and bind into a plan against `doc` at `revision`/`anchor`.
/// A failure during parsing or expansion surfaces here as
/// `PlanError::Snippet(SnippetError::...)` via the crate's `From` impl.
fn attempt_plan(
    src: &str,
    revision: u64,
    doc: &str,
    anchor: Anchor,
    overrides: &HashMap<u32, String>,
) -> Result<SnippetPlan, PlanError> {
    let snippet = Snippet::parse(src)?;
    SnippetPlan::compute(&snippet, revision, doc, anchor, overrides)
}

fn ok(src: &str) -> bool {
    attempt_plan(src, 1, "doc", Anchor::Caret(0), &HashMap::new()).is_ok()
}

fn err(src: &str) -> PlanError {
    attempt_plan(src, 1, "doc", Anchor::Caret(0), &HashMap::new()).unwrap_err()
}

// ---------------------------------------------------------------------
// 1. Input size cap
// ---------------------------------------------------------------------

#[test]
fn input_exactly_at_the_byte_cap_succeeds_one_byte_past_is_rejected() {
    assert!(ok(&"a".repeat(MAX_INPUT_BYTES)));

    let past_cap = "a".repeat(MAX_INPUT_BYTES + 1);
    assert_eq!(
        err(&past_cap),
        PlanError::Snippet(SnippetError::InputTooLarge {
            len: MAX_INPUT_BYTES + 1
        })
    );
}

// ---------------------------------------------------------------------
// 2. Nesting depth cap
// ---------------------------------------------------------------------

fn nested_source(depth: usize) -> String {
    let mut src = String::new();
    for i in 0..depth {
        src.push_str(&format!("${{{}:", i + 1));
    }
    src.push('x');
    for _ in 0..depth {
        src.push('}');
    }
    src
}

#[test]
fn nesting_exactly_at_the_depth_cap_succeeds_one_level_past_is_rejected() {
    assert!(ok(&nested_source(MAX_NESTING_DEPTH)));

    let too_deep = nested_source(MAX_NESTING_DEPTH + 1);
    assert!(matches!(
        err(&too_deep),
        PlanError::Snippet(SnippetError::NestingTooDeep { .. })
    ));
}

// ---------------------------------------------------------------------
// 3. Placeholder index cap
// ---------------------------------------------------------------------

#[test]
fn placeholder_index_exactly_at_the_bound_succeeds_one_past_is_rejected() {
    let at_bound = format!("${{{MAX_PLACEHOLDER_INDEX}}}");
    assert!(ok(&at_bound));

    let past_bound = format!("${{{}}}", MAX_PLACEHOLDER_INDEX + 1);
    assert_eq!(
        err(&past_bound),
        PlanError::Snippet(SnippetError::PlaceholderIndexTooLarge { offset: 0 })
    );
}

// ---------------------------------------------------------------------
// 4. Distinct placeholder count cap
// ---------------------------------------------------------------------

fn distinct_placeholders_source(count: usize) -> String {
    let mut src = String::new();
    for i in 1..=count {
        src.push_str(&format!("${i} "));
    }
    src
}

#[test]
fn distinct_placeholders_exactly_at_the_bound_succeeds_one_past_is_rejected() {
    assert!(ok(&distinct_placeholders_source(MAX_PLACEHOLDERS)));

    let one_past = distinct_placeholders_source(MAX_PLACEHOLDERS + 1);
    assert!(matches!(
        err(&one_past),
        PlanError::Snippet(SnippetError::TooManyPlaceholders { .. })
    ));
}

// ---------------------------------------------------------------------
// 5. Total occurrence count cap
// ---------------------------------------------------------------------

#[test]
fn occurrences_exactly_at_the_bound_succeeds_one_past_is_rejected() {
    assert!(ok(&"$1 ".repeat(MAX_OCCURRENCES)));

    let one_past = "$1 ".repeat(MAX_OCCURRENCES + 1);
    assert!(matches!(
        err(&one_past),
        PlanError::Snippet(SnippetError::TooManyOccurrences { .. })
    ));
}

// ---------------------------------------------------------------------
// 6. Output size cap
// ---------------------------------------------------------------------

#[test]
fn output_exactly_at_the_byte_cap_succeeds_one_byte_past_is_rejected() {
    let mut at_cap = HashMap::new();
    at_cap.insert(1, "a".repeat(MAX_OUTPUT_BYTES));
    assert!(attempt_plan("$1", 1, "doc", Anchor::Caret(0), &at_cap).is_ok());

    let mut past_cap = HashMap::new();
    past_cap.insert(1, "a".repeat(MAX_OUTPUT_BYTES + 1));
    let result = attempt_plan("$1", 1, "doc", Anchor::Caret(0), &past_cap);
    assert_eq!(
        result.unwrap_err(),
        PlanError::Snippet(SnippetError::OutputTooLarge {
            len: MAX_OUTPUT_BYTES + 1
        })
    );
}

// ---------------------------------------------------------------------
// 7. Unbalanced and escaped braces
// ---------------------------------------------------------------------

#[test]
fn unterminated_placeholder_brace_is_a_typed_error() {
    assert_eq!(
        err("${1:abc"),
        PlanError::Snippet(SnippetError::UnterminatedPlaceholder { offset: 0 })
    );
}

#[test]
fn unterminated_brace_with_no_colon_is_a_typed_error() {
    assert_eq!(
        err("${1"),
        PlanError::Snippet(SnippetError::UnterminatedPlaceholder { offset: 0 })
    );
}

#[test]
fn escaped_closing_brace_is_literal_text_not_a_structural_close() {
    let plan = attempt_plan(r"${1:a\}b}", 1, "doc", Anchor::Caret(0), &HashMap::new()).unwrap();
    assert_eq!(plan.expansion().text, "a}b");
}

#[test]
fn stray_unmatched_closing_brace_at_top_level_is_literal_not_an_error() {
    let plan = attempt_plan("a}b", 1, "doc", Anchor::Caret(0), &HashMap::new()).unwrap();
    assert_eq!(plan.expansion().text, "a}b");
}

// ---------------------------------------------------------------------
// 8. Dollar sign at end of input
// ---------------------------------------------------------------------

#[test]
fn dollar_sign_at_end_of_input_is_literal_not_a_hang_or_panic() {
    let plan = attempt_plan("abc$", 1, "doc", Anchor::Caret(0), &HashMap::new()).unwrap();
    assert_eq!(plan.expansion().text, "abc$");
}

// ---------------------------------------------------------------------
// 9. Mutually self-referential placeholders
// ---------------------------------------------------------------------

#[test]
fn mutually_self_referential_placeholders_is_a_typed_error() {
    assert!(matches!(
        err("${1:$2}${2:$1}"),
        PlanError::Snippet(SnippetError::SelfReferential { .. })
    ));
}

// ---------------------------------------------------------------------
// 10. Caret past end of document
// ---------------------------------------------------------------------

#[test]
fn caret_past_end_of_document_is_a_typed_error() {
    let doc = "short";
    let result = attempt_plan("$1", 1, doc, Anchor::Caret(doc.len() + 1), &HashMap::new());
    assert_eq!(
        result.unwrap_err(),
        PlanError::InvalidOffset {
            offset: doc.len() + 1
        }
    );
}

// ---------------------------------------------------------------------
// 11. Inverted selection
// ---------------------------------------------------------------------

#[test]
fn inverted_selection_is_a_typed_error() {
    let doc = "hello world";
    let (start, end) = (5usize, 2usize);
    let result = attempt_plan("$1", 1, doc, Anchor::Selection(start..end), &HashMap::new());
    assert_eq!(
        result.unwrap_err(),
        PlanError::SelectionReversed { start: 5, end: 2 }
    );
}

// ---------------------------------------------------------------------
// 12. Caret mid-character in multi-byte text
// ---------------------------------------------------------------------

#[test]
fn caret_mid_character_in_multibyte_text_is_a_typed_error() {
    // "café": 'é' is a 2-byte char starting at offset 3, so offset 4 is
    // mid-character.
    let doc = "café";
    let result = attempt_plan("$1", 1, doc, Anchor::Caret(4), &HashMap::new());
    assert_eq!(result.unwrap_err(), PlanError::InvalidOffset { offset: 4 });

    // The valid boundary immediately before it still succeeds, proving
    // this is a real char-boundary check and not a blanket rejection.
    assert!(attempt_plan("$1", 1, doc, Anchor::Caret(3), &HashMap::new()).is_ok());
}
