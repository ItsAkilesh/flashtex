//! Bounded adversarial acceptance tests (FT-037 rev 3).
//!
//! Every case here attacks either URI scheme validation (`validate_uri`) or
//! source/range binding (`SourceSpan::new`, `SourceIdentity::bind`) with a
//! malformed, hostile, or boundary-exact input, and asserts the *exact*
//! typed error variant produced. None of these panic or hang: every
//! function under test does a bounded number of linear scans over the input
//! (no regex, no recursion, no backtracking), so cost is bounded by input
//! length alone, independent of content.
//!
//! ## Percent-encoding and the allowlist: is it a bypass?
//!
//! **No.** `validate_uri` never percent-decodes anything, at any stage,
//! before or during the scheme check:
//!
//! - If percent-encoded bytes appear **before** the first `:`, they become
//!   part of the literal scheme token, which is checked as raw text against
//!   `ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )` (RFC 3986). A `%`
//!   character always fails that check, so an encoded scheme cannot decode
//!   its way past the allowlist — it fails as `InvalidSchemeSyntax`, never
//!   as a match against the decoded (blocked) scheme name.
//! - If percent-encoded bytes appear **after** the first `:` (i.e. in the
//!   path/query of an already-determined, allowed scheme), they are never
//!   inspected for scheme purposes at all: the scheme was already fixed
//!   from the raw prefix before that colon. `https://x/%6A%61vascript:evil`
//!   is simply a normal, accepted `https` URI whose path happens to contain
//!   percent-escapes; nothing decodes it.
//!
//! In both directions, the allowlist only ever sees the raw, undecoded
//! bytes before the first colon.

use flashtex_link_annotations::{
    LabelError, LabelId, MAX_LABEL_LEN, MAX_URI_LEN, Point, Rect, RectError, RevisionId,
    SourceIdentity, SourceIdentityError, SourcePos, SourceSpan, SpanError, UriError, UriScheme,
    validate_uri,
};

// ---------------------------------------------------------------------
// Scheme casing, embedded whitespace, and control characters
// ---------------------------------------------------------------------

#[test]
fn scheme_with_embedded_space_fails_syntax_not_control_character() {
    // A space is not a control character, so it survives the
    // control-character scan and is instead caught by scheme syntax
    // validation (space is not ALPHA/DIGIT/"+"/"-"/".").
    let err = validate_uri("Ht Tp://example.com").unwrap_err();
    assert_eq!(
        err,
        UriError::InvalidSchemeSyntax {
            scheme: "Ht Tp".to_string()
        }
    );
}

#[test]
fn scheme_with_embedded_tab_is_caught_as_control_character() {
    // A tab (0x09) IS a control character: the whole-string scan that runs
    // before scheme parsing catches it first, at its exact byte offset.
    let err = validate_uri("Ht\tTp://example.com").unwrap_err();
    assert_eq!(err, UriError::ControlCharacter { at: 2 });
}

#[test]
fn scheme_with_unusual_casing_and_embedded_control_character() {
    let err = validate_uri("HTT\u{0001}PS://example.com").unwrap_err();
    assert_eq!(err, UriError::ControlCharacter { at: 3 });
}

// ---------------------------------------------------------------------
// Scheme-like prefix with no colon at all
// ---------------------------------------------------------------------

#[test]
fn bare_scheme_like_words_with_no_colon_are_missing_scheme() {
    for text in ["javascript", "https", "mailto", "data", "file"] {
        let err = validate_uri(text).unwrap_err();
        assert_eq!(err, UriError::MissingScheme, "input: {text:?}");
    }
}

// ---------------------------------------------------------------------
// Nested / repeated schemes
// ---------------------------------------------------------------------

#[test]
fn allowed_outer_scheme_with_denied_scheme_word_in_the_path_is_accepted_as_the_outer_scheme() {
    // The FIRST colon in the string determines the scheme. Everything after
    // it — including a scheme-like word followed by its own colon — is
    // opaque path/query content and is never re-parsed as a nested scheme.
    let uri = validate_uri("http://example.com/redirect?to=javascript:alert(1)").unwrap();
    assert_eq!(uri.scheme(), UriScheme::Http);
}

#[test]
fn denied_outer_scheme_with_allowed_scheme_word_nested_inside_is_still_rejected() {
    // Reversing the nesting must not create a bypass either: the outer
    // (first) scheme governs, regardless of what looks like an allowed
    // scheme further inside the string.
    let err = validate_uri("javascript://http:evil.example/payload").unwrap_err();
    assert_eq!(
        err,
        UriError::SchemeNotAllowed {
            scheme: "javascript".to_string()
        }
    );
}

// ---------------------------------------------------------------------
// Percent-encoded content that decodes to a blocked scheme
// ---------------------------------------------------------------------

#[test]
fn partially_percent_encoded_scheme_token_fails_syntax_rather_than_decoding() {
    // "%6A%61vascript" would decode to "javascript", but this function
    // never decodes: the raw '%' characters in the scheme position fail
    // syntax validation outright, before any allowlist lookup happens.
    let err = validate_uri("%6A%61vascript://evil.example").unwrap_err();
    assert_eq!(
        err,
        UriError::InvalidSchemeSyntax {
            scheme: "%6A%61vascript".to_string()
        }
    );
}

#[test]
fn fully_percent_encoded_scheme_also_fails_syntax() {
    // Fully percent-encoded "javascript".
    let err = validate_uri("%6A%61%76%61%73%63%72%69%70%74:alert(1)").unwrap_err();
    assert!(matches!(err, UriError::InvalidSchemeSyntax { .. }));
}

#[test]
fn percent_encoding_after_an_allowed_scheme_is_inert_and_accepted_verbatim() {
    // Percent-encoded bytes appearing after the already-determined scheme
    // are never decoded or specially inspected.
    let uri = validate_uri("https://example.com/%6A%61vascript:alert(1)").unwrap();
    assert_eq!(uri.scheme(), UriScheme::Https);
    assert_eq!(uri.as_str(), "https://example.com/%6A%61vascript:alert(1)");
}

// ---------------------------------------------------------------------
// Length cap: exactly at it, and one byte past it
// ---------------------------------------------------------------------

#[test]
fn valid_uri_of_exactly_max_length_is_accepted() {
    let prefix = "https://example.com/";
    let exact = format!("{prefix}{}", "a".repeat(MAX_URI_LEN - prefix.len()));
    assert_eq!(exact.len(), MAX_URI_LEN);
    let uri = validate_uri(&exact).unwrap();
    assert_eq!(uri.scheme(), UriScheme::Https);
}

#[test]
fn valid_uri_one_byte_past_max_length_is_rejected() {
    let prefix = "https://example.com/";
    let over = format!("{prefix}{}", "a".repeat(MAX_URI_LEN - prefix.len() + 1));
    assert_eq!(over.len(), MAX_URI_LEN + 1);
    let err = validate_uri(&over).unwrap_err();
    assert_eq!(
        err,
        UriError::TooLong {
            len: over.len(),
            max: MAX_URI_LEN
        }
    );
}

// ---------------------------------------------------------------------
// Labels: empty, and absurdly long
// ---------------------------------------------------------------------

#[test]
fn empty_label_is_a_typed_error() {
    assert_eq!(LabelId::parse("").unwrap_err(), LabelError::Empty);
}

#[test]
fn enormous_label_fails_the_length_bound_before_any_scan() {
    let hostile = "x".repeat(50_000_000);
    let err = LabelId::parse(&hostile).unwrap_err();
    assert_eq!(
        err,
        LabelError::TooLong {
            len: hostile.len(),
            max: MAX_LABEL_LEN
        }
    );
}

// ---------------------------------------------------------------------
// Byte ranges: inverted, past end, mid-character (start and end)
// ---------------------------------------------------------------------

#[test]
fn inverted_byte_range_is_rejected_at_span_construction() {
    let start = SourcePos::new(40, 1, 41);
    let end = SourcePos::new(5, 1, 6);
    let err = SourceSpan::new(start, end).unwrap_err();
    assert_eq!(err, SpanError::Inverted { start, end });
}

#[test]
fn byte_range_past_end_of_source_is_rejected_at_bind() {
    let source = "short source text";
    let span = SourceSpan::new(SourcePos::new(0, 1, 1), SourcePos::new(9_999, 1, 1)).unwrap();
    let err = SourceIdentity::bind(RevisionId::parse("r1").unwrap(), source, span).unwrap_err();
    assert_eq!(
        err,
        SourceIdentityError::SpanOutOfBounds {
            end: 9_999,
            source_len: source.len()
        }
    );
}

#[test]
fn byte_range_end_landing_mid_character_is_rejected_at_bind() {
    // "日" is 3 bytes (boundaries only at 0, 3, 6, 9); end offset 2 lands
    // inside the first character.
    let source = "日本語";
    let span = SourceSpan::new(SourcePos::new(0, 1, 1), SourcePos::new(2, 1, 1)).unwrap();
    let err = SourceIdentity::bind(RevisionId::parse("r1").unwrap(), source, span).unwrap_err();
    assert_eq!(err, SourceIdentityError::NotCharBoundary { offset: 2 });
}

#[test]
fn byte_range_start_landing_mid_character_is_rejected_at_bind() {
    let source = "日本語";
    // Start offset 1 lands inside the first character; end (3) is a valid
    // boundary, but start is checked and must fail on its own.
    let span = SourceSpan::new(SourcePos::new(1, 1, 1), SourcePos::new(3, 1, 1)).unwrap();
    let err = SourceIdentity::bind(RevisionId::parse("r1").unwrap(), source, span).unwrap_err();
    assert_eq!(err, SourceIdentityError::NotCharBoundary { offset: 1 });
}

// ---------------------------------------------------------------------
// Rectangles: non-finite and negative dimensions
// ---------------------------------------------------------------------

#[test]
fn rect_rejects_nan_width() {
    let err = Rect::new(Point::new(0.0, 0.0), f64::NAN, 10.0).unwrap_err();
    assert!(matches!(err, RectError::NonFiniteExtent { .. }));
}

#[test]
fn rect_rejects_negative_infinity_height() {
    let err = Rect::new(Point::new(0.0, 0.0), 10.0, f64::NEG_INFINITY).unwrap_err();
    assert!(matches!(err, RectError::NonFiniteExtent { .. }));
}

#[test]
fn rect_rejects_negative_width_and_height_together() {
    let err = Rect::new(Point::new(0.0, 0.0), -1.0, -2.0).unwrap_err();
    assert_eq!(
        err,
        RectError::NegativeExtent {
            width: -1.0,
            height: -2.0
        }
    );
}

#[test]
fn rect_rejects_non_finite_origin() {
    let err = Rect::new(Point::new(f64::INFINITY, f64::NAN), 1.0, 1.0).unwrap_err();
    assert!(matches!(err, RectError::NonFiniteOrigin { .. }));
}
