//! Stale-identity acceptance suite (FT-037 rev 3).
//!
//! This is the acceptance test the rev 3 objective names: an explicit,
//! readable specification of the `SourceIdentity` staleness contract. It
//! covers every way the (revision, source) pair a link was bound against
//! can move by the time it is re-checked, and the exact `Staleness` variant
//! `check_fresh` reports for each.
//!
//! | how the source changed                                         | `check_fresh` result                     |
//! |------------------------------------------------------------------|-------------------------------------------|
//! | revision id only (bytes at the bound span unchanged)              | `Err(RevisionChanged { expected, found })` |
//! | content only (same revision, bytes under the span edited)         | `Err(ContentChanged)`                      |
//! | both revision and content changed at once                         | `Err(RevisionChanged { expected, found })` |
//! | the bound byte range no longer fits/aligns in the current source  | `Err(RangeInvalid)`                        |
//! | nothing changed (same revision, same bytes)                       | `Ok(())`                                   |
//!
//! The "both changed" row is a deliberate contract, not an oversight:
//! [`SourceIdentity::check_fresh`] checks the revision id before it ever
//! re-hashes the span, so a revision change is reported even when the
//! content also changed underneath it. A caller always learns "you're
//! looking at the wrong revision" first, rather than a possibly-misleading
//! "the text changed" that would still be true of the old revision too.
//!
//! `RangeInvalid` is one variant covering two distinct underlying causes —
//! the span now runs past the end of the source, or one of its offsets no
//! longer falls on a UTF-8 character boundary — because from the caller's
//! point of view both mean the same thing: this span cannot be trusted
//! against this source at all, so there is nothing to hash. Both causes are
//! exercised below.

use flashtex_link_annotations::{RevisionId, SourceIdentity, SourcePos, SourceSpan, Staleness};

/// The source text and span every scenario starts from: a `\href{...}{me}`
/// call, with the span bound over exactly the `\href{https://example.com}`
/// portion.
const ORIGINAL: &str = "click \\href{https://example.com}{me}";

fn rev(id: &str) -> RevisionId {
    RevisionId::parse(id).unwrap()
}

/// Binds a fresh identity against `ORIGINAL` at revision `r1`, returning it
/// alongside that revision so tests can assert on `expected`/`found`
/// payloads without repeating the literal.
fn bound_identity() -> (SourceIdentity, RevisionId) {
    let revision = rev("r1");
    let span = SourceSpan::new(SourcePos::new(6, 1, 7), SourcePos::new(33, 1, 34)).unwrap();
    let identity = SourceIdentity::bind(revision.clone(), ORIGINAL, span).unwrap();
    (identity, revision)
}

#[test]
fn revision_id_only_changes_reports_revision_changed() {
    let (identity, original_revision) = bound_identity();
    let new_revision = rev("r2");

    // Same exact bytes as at binding time; only the revision id moved.
    let result = identity.check_fresh(&new_revision, ORIGINAL);

    assert_eq!(
        result,
        Err(Staleness::RevisionChanged {
            expected: original_revision,
            found: new_revision,
        })
    );
}

#[test]
fn content_only_changes_reports_content_changed() {
    let (identity, revision) = bound_identity();
    // Same revision id; the bytes under the bound span were edited.
    let edited = "click \\href{https://evil.example}{me}";

    let result = identity.check_fresh(&revision, edited);

    assert_eq!(result, Err(Staleness::ContentChanged));
}

#[test]
fn both_revision_and_content_change_reports_revision_changed_not_content_changed() {
    let (identity, original_revision) = bound_identity();
    let new_revision = rev("r2");
    let edited = "click \\href{https://evil.example}{me}";

    // Both moved at once: the revision check runs first and short-circuits
    // before the span is ever re-hashed, so this is `RevisionChanged`,
    // never `ContentChanged` — the exact precedence documented above.
    let result = identity.check_fresh(&new_revision, edited);

    assert_eq!(
        result,
        Err(Staleness::RevisionChanged {
            expected: original_revision,
            found: new_revision,
        })
    );
}

#[test]
fn range_no_longer_fitting_the_shrunk_source_reports_range_invalid() {
    let (identity, revision) = bound_identity();
    // Same revision id, but the source shrank so the bound byte range no
    // longer fits inside it at all.
    let truncated = "click";

    let result = identity.check_fresh(&revision, truncated);

    assert_eq!(result, Err(Staleness::RangeInvalid));
}

#[test]
fn range_misaligned_by_a_multibyte_insertion_also_reports_range_invalid() {
    let (identity, revision) = bound_identity();
    // Same revision id and the span still fits inside the (now longer)
    // source, but a two-byte character ('é') was spliced in right at the
    // span's bound end offset (33), so that absolute offset now falls
    // inside the middle of a character instead of on a boundary.
    let respliced = format!("{}{}{}", &ORIGINAL[..32], "é", &ORIGINAL[32..]);

    let result = identity.check_fresh(&revision, &respliced);

    assert_eq!(result, Err(Staleness::RangeInvalid));
}

#[test]
fn nothing_changed_is_reported_as_fresh() {
    let (identity, revision) = bound_identity();

    let result = identity.check_fresh(&revision, ORIGINAL);

    assert_eq!(result, Ok(()));
}
