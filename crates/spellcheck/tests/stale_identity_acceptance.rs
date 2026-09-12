//! Acceptance suite for the revision contract on [`CheckResult`].
//!
//! This file is meant to read as a specification of exactly three
//! observable behaviors, independent of any implementation detail:
//!
//! 1. A result computed at one revision is STALE against a later revision.
//! 2. A result computed at a revision is FRESH when re-checked against that
//!    same revision *and* the same text.
//! 3. A result computed at a revision is still detected as unusable when the
//!    caller reuses that same revision value after the text actually
//!    changed (a missed revision-bump bug) — "identical revision" alone
//!    does not certify freshness.
//!
//! [`CheckResult::is_stale`] covers only revision comparison (behavior 1 and
//! half of behavior 2/3: same-revision-same-text is fresh, but a
//! same-revision-changed-text caller bug is invisible to it by design,
//! since it never sees text). [`CheckResult::is_stale_for`] covers all
//! three by additionally comparing the text itself.

use flashtex_spellcheck::{Revision, SpellChecker};
use std::collections::HashSet;

fn dict() -> HashSet<String> {
    ["hello", "world"].iter().map(|s| s.to_string()).collect()
}

const REV_1: Revision = 1;
const REV_2: Revision = 2;

// ---------------------------------------------------------------------
// Specification 1: later revision => stale, for both is_stale and
// is_stale_for, regardless of whether the text also changed.
// ---------------------------------------------------------------------

#[test]
fn spec_result_computed_at_one_revision_is_stale_against_a_later_revision() {
    let checker = SpellChecker::default();
    let text = "hello wrold";
    let result = checker.check_revision(text, REV_1, &dict());

    assert!(
        result.is_stale(REV_2),
        "a result computed at revision {REV_1} must be stale when compared against a later revision {REV_2}"
    );
    assert!(
        result.is_stale_for(REV_2, text),
        "is_stale_for must also report stale on a later revision, even with unchanged text"
    );
}

#[test]
fn spec_later_revision_with_changed_text_is_also_stale() {
    let checker = SpellChecker::default();
    let result = checker.check_revision("hello wrold", REV_1, &dict());

    assert!(result.is_stale(REV_2));
    assert!(result.is_stale_for(REV_2, "hello wrold, more text now"));
}

// ---------------------------------------------------------------------
// Specification 2: identical revision AND identical text => fresh.
// ---------------------------------------------------------------------

#[test]
fn spec_identical_revision_and_identical_text_is_fresh() {
    let checker = SpellChecker::default();
    let text = "hello wrold";
    let result = checker.check_revision(text, REV_1, &dict());

    assert!(
        !result.is_stale(REV_1),
        "same revision, no text supplied to compare, must read as fresh"
    );
    assert!(
        !result.is_stale_for(REV_1, text),
        "same revision AND byte-identical text must read as fresh"
    );
}

// ---------------------------------------------------------------------
// Specification 3: identical revision but the text actually changed must
// still be detected — this is the case plain is_stale(revision) cannot see,
// and is_stale_for(revision, text) exists specifically to catch.
// ---------------------------------------------------------------------

#[test]
fn spec_identical_revision_with_changed_text_is_still_detected_as_unusable() {
    let checker = SpellChecker::default();
    let original_text = "hello wrold";
    let result = checker.check_revision(original_text, REV_1, &dict());

    // The caller made a bookkeeping mistake: the text moved on but the
    // revision counter was not bumped. Plain revision comparison is blind
    // to this by construction (it never looks at text) -- documented, not
    // a bug in is_stale itself.
    let changed_text = "hello wrold, but now there is more prose";
    assert!(
        !result.is_stale(REV_1),
        "is_stale only ever compares revisions, so an unchanged revision reads as fresh here"
    );

    // is_stale_for is the API built to catch exactly this: same revision,
    // different text, must still come back as stale/unusable.
    assert!(
        result.is_stale_for(REV_1, changed_text),
        "identical revision with changed text must still be detected as stale via is_stale_for"
    );
}

#[test]
fn spec_is_stale_for_is_insensitive_to_which_misspellings_the_change_introduces() {
    // The mechanism must not depend on the change producing a different
    // misspelling count/content -- even a single trailing byte difference
    // with identical misspellings must be caught.
    let checker = SpellChecker::default();
    let text_a = "hello wrold today";
    let text_b = "hello wrold today.";
    assert_eq!(
        checker.check(text_a, &dict()).len(),
        checker.check(text_b, &dict()).len(),
        "test setup: both texts must flag the same single misspelling"
    );

    let result = checker.check_revision(text_a, REV_1, &dict());
    assert!(result.is_stale_for(REV_1, text_b));
}

// ---------------------------------------------------------------------
// The same three specifications also hold through check_cancellable's
// CheckOutcome::Completed path, not just check_revision.
// ---------------------------------------------------------------------

#[test]
fn spec_holds_through_check_cancellable_completed_outcome() {
    use flashtex_spellcheck::CheckOutcome;

    let checker = SpellChecker::default();
    let text = "hello wrold";
    let outcome = checker.check_cancellable(text, REV_1, &dict(), &|| false);
    let result = match outcome {
        CheckOutcome::Completed(result) => result,
        CheckOutcome::Cancelled { .. } => panic!("must not cancel: signal never fires"),
    };

    // Fresh: identical revision, identical text.
    assert!(!result.is_stale_for(REV_1, text));
    // Stale: later revision.
    assert!(result.is_stale_for(REV_2, text));
    // Stale: identical revision, changed text.
    assert!(result.is_stale_for(REV_1, "hello wrold, changed"));
}
