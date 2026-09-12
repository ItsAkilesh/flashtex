//! Stale-identity acceptance suite for `SnippetPlan`'s contract.
//!
//! Whether a previously computed plan is still safe to apply to a document
//! depends on matching BOTH the caller's own revision id and a hash of the
//! document's exact bytes - never the revision id alone. This file is the
//! specification for `SnippetPlan::staleness` (equivalently,
//! `DocumentId::compare`), stated as the full two-by-two matrix of
//! "revision same/changed" x "content same/changed":
//!
//! | revision  | content   | expected staleness |
//! |-----------|-----------|---------------------|
//! | unchanged | unchanged | `Fresh`             |
//! | unchanged | changed   | `ContentMismatch`   |
//! | changed   | unchanged | `RevisionMismatch`  |
//! | changed   | changed   | `RevisionMismatch`  |
//!
//! The second row is the case a naive staleness check - one that trusts
//! the caller's revision counter alone - gets wrong: a caller can fail to
//! bump its revision id even though the underlying bytes changed, and a
//! plan computed against the old bytes must still be refused. The fourth
//! row is not a distinct "everything changed" outcome: `DocumentId::compare`
//! checks revision first, so any revision mismatch reports
//! `RevisionMismatch` regardless of what the content hash shows - a
//! revision mismatch is already sufficient reason to refuse the plan.

use std::collections::HashMap;

use flashtex_editor_snippets::{Anchor, DocumentId, Snippet, SnippetPlan, Staleness};

const REVISION: u64 = 7;
const ORIGINAL_DOC: &str = "func ()";
const EDITED_DOC: &str = "func (edited)";

/// A plan computed against `ORIGINAL_DOC` at `REVISION`, caret at offset 5
/// (inside the parentheses). Every test below checks this same plan's
/// staleness against a different "current" document identity.
fn plan_against_original() -> SnippetPlan {
    let snippet = Snippet::parse("${1:name}").unwrap();
    SnippetPlan::compute(
        &snippet,
        REVISION,
        ORIGINAL_DOC,
        Anchor::Caret(5),
        &HashMap::new(),
    )
    .unwrap()
}

// ---------------------------------------------------------------------
// Row 1: revision unchanged, content unchanged -> Fresh.
// ---------------------------------------------------------------------

/// The only outcome under which a caller may apply a plan's anchor and
/// expansion to the document exactly as computed.
#[test]
fn same_revision_and_same_content_is_fresh() {
    let plan = plan_against_original();
    let current = DocumentId::new(REVISION, ORIGINAL_DOC);

    assert_eq!(plan.staleness(&current), Staleness::Fresh);
    assert!(plan.staleness(&current).is_fresh());
}

// ---------------------------------------------------------------------
// Row 2: revision changed, content unchanged -> RevisionMismatch.
// ---------------------------------------------------------------------

/// Even when the document's bytes happen to be unchanged, a revision id
/// that no longer matches is on its own enough to refuse the plan: this
/// crate treats the caller's revision counter as authoritative evidence
/// that *something* in the caller's world moved on, whether or not the
/// bytes visible to this crate reflect it yet.
#[test]
fn changed_revision_with_same_content_is_revision_mismatch() {
    let plan = plan_against_original();
    let current = DocumentId::new(REVISION + 1, ORIGINAL_DOC);

    let staleness = plan.staleness(&current);
    assert!(!staleness.is_fresh());
    match staleness {
        Staleness::RevisionMismatch { planned, current } => {
            assert_eq!(planned, REVISION);
            assert_eq!(current, REVISION + 1);
        }
        other => panic!("expected RevisionMismatch, got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// Row 3 (the critical case): revision unchanged, content changed ->
// ContentMismatch, never Fresh.
// ---------------------------------------------------------------------

/// The case a naive revision-only check misses. The caller's revision
/// counter did not change, but the document's bytes did underneath the
/// plan - the content hash must catch it anyway, because applying the
/// plan's byte offsets to the new bytes would be wrong.
#[test]
fn same_revision_with_changed_content_is_content_mismatch_not_fresh() {
    let plan = plan_against_original();
    // Same revision id (REVISION) as the plan was computed against...
    let current = DocumentId::new(REVISION, EDITED_DOC);

    let staleness = plan.staleness(&current);
    // ...but the bytes differ, so this must never be reported as Fresh.
    assert!(!staleness.is_fresh());
    match staleness {
        Staleness::ContentMismatch { planned, current } => {
            assert_ne!(planned, current);
        }
        other => panic!("expected ContentMismatch, got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// Row 4: revision changed AND content changed -> RevisionMismatch (the
// revision check runs first, so it wins; there is no combined variant).
// ---------------------------------------------------------------------

#[test]
fn changed_revision_and_changed_content_is_revision_mismatch() {
    let plan = plan_against_original();
    let current = DocumentId::new(REVISION + 1, EDITED_DOC);

    let staleness = plan.staleness(&current);
    assert!(!staleness.is_fresh());
    match staleness {
        Staleness::RevisionMismatch { planned, current } => {
            assert_eq!(planned, REVISION);
            assert_eq!(current, REVISION + 1);
        }
        other => panic!("expected RevisionMismatch, got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// The same four-row matrix restated directly against `DocumentId::compare`,
// bypassing `SnippetPlan` entirely, so the specification is pinned at both
// the identity primitive and the plan contract built on top of it.
// ---------------------------------------------------------------------

#[test]
fn document_id_compare_matches_the_same_four_row_matrix() {
    let planned = DocumentId::new(REVISION, ORIGINAL_DOC);

    let same_revision_same_content = DocumentId::new(REVISION, ORIGINAL_DOC);
    assert_eq!(
        planned.compare(&same_revision_same_content),
        Staleness::Fresh
    );

    let changed_revision_same_content = DocumentId::new(REVISION + 1, ORIGINAL_DOC);
    assert!(matches!(
        planned.compare(&changed_revision_same_content),
        Staleness::RevisionMismatch { .. }
    ));

    let same_revision_changed_content = DocumentId::new(REVISION, EDITED_DOC);
    assert!(matches!(
        planned.compare(&same_revision_changed_content),
        Staleness::ContentMismatch { .. }
    ));

    let changed_revision_changed_content = DocumentId::new(REVISION + 1, EDITED_DOC);
    assert!(matches!(
        planned.compare(&changed_revision_changed_content),
        Staleness::RevisionMismatch { .. }
    ));
}
