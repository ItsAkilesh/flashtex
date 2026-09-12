//! Exact-value tests for parsing, expansion, linked edits, guarded tab
//! navigation, bounded malformed input, and UTF-8 offset safety.

use std::collections::HashMap;
use std::ops::Range;

use flashtex_editor_snippets::{
    MAX_NESTING_DEPTH, MAX_OCCURRENCES, MAX_PLACEHOLDER_INDEX, Snippet, SnippetError, TabStops,
};

fn expand(src: &str) -> String {
    Snippet::parse(src).unwrap().expand().unwrap().text
}

// ---------------------------------------------------------------------
// Hand-worked exact text and offsets
// ---------------------------------------------------------------------

#[test]
fn plain_text_has_no_placeholders() {
    let exp = Snippet::parse("no placeholders here")
        .unwrap()
        .expand()
        .unwrap();
    assert_eq!(exp.text, "no placeholders here");
    assert!(exp.placeholders.is_empty());
}

#[test]
fn bare_placeholder_defaults_to_empty_text_at_exact_offset() {
    let exp = Snippet::parse("go: $1!").unwrap().expand().unwrap();
    assert_eq!(exp.text, "go: !");
    assert_eq!(exp.occurrences_of(1), &[Range { start: 4, end: 4 }]);
}

#[test]
fn placeholder_with_braces_no_default_is_same_as_bare() {
    let exp = Snippet::parse("x${1}y").unwrap().expand().unwrap();
    assert_eq!(exp.text, "xy");
    assert_eq!(exp.occurrences_of(1), &[Range { start: 1, end: 1 }]);
}

#[test]
fn default_text_is_used_and_offsets_are_exact() {
    // "func " (5 bytes) + "name" (4 bytes, offsets 5..9) + "()"
    let exp = Snippet::parse("func ${1:name}()")
        .unwrap()
        .expand()
        .unwrap();
    assert_eq!(exp.text, "func name()");
    assert_eq!(exp.occurrences_of(1), &[Range { start: 5, end: 9 }]);
}

#[test]
fn multiple_distinct_placeholders_get_independent_exact_offsets() {
    // "Hello, " = 7 bytes; "World" = 5 bytes -> 7..12
    // "! Bye, " next: "! Bye, " = 7 bytes starting at 12 -> ends at 19
    // "Mars" = 4 bytes -> 19..23
    let exp = Snippet::parse("Hello, ${1:World}! Bye, ${2:Mars}.")
        .unwrap()
        .expand()
        .unwrap();
    assert_eq!(exp.text, "Hello, World! Bye, Mars.");
    assert_eq!(exp.occurrences_of(1), &[Range { start: 7, end: 12 }]);
    assert_eq!(exp.occurrences_of(2), &[Range { start: 19, end: 23 }]);
}

#[test]
fn placeholders_sorted_by_index_with_zero_last_in_tab_order() {
    let exp = Snippet::parse("$2 $0 $1").unwrap().expand().unwrap();
    assert_eq!(exp.tab_order(), vec![1, 2, 0]);
}

// ---------------------------------------------------------------------
// Escapes
// ---------------------------------------------------------------------

#[test]
fn escaped_dollar_is_literal_and_not_a_placeholder() {
    let exp = Snippet::parse(r"price: \$5").unwrap().expand().unwrap();
    assert_eq!(exp.text, "price: $5");
    assert!(exp.placeholders.is_empty());
}

#[test]
fn escaped_brace_inside_default_is_literal() {
    let exp = Snippet::parse(r"${1:a\}b}").unwrap().expand().unwrap();
    assert_eq!(exp.text, "a}b");
}

#[test]
fn escaped_backslash_is_literal() {
    let exp = Snippet::parse(r"a\\b").unwrap().expand().unwrap();
    assert_eq!(exp.text, r"a\b");
}

#[test]
fn lone_dollar_at_end_of_input_is_literal() {
    assert_eq!(expand("cost: $"), "cost: $");
}

#[test]
fn dollar_followed_by_non_digit_non_brace_is_literal() {
    assert_eq!(expand("$ and $x"), "$ and $x");
}

// ---------------------------------------------------------------------
// Linked edits
// ---------------------------------------------------------------------

#[test]
fn linked_occurrences_share_default_value() {
    let snippet = Snippet::parse("Hello, ${1:World}! $1 says hi to $1.").unwrap();
    let exp = snippet.expand().unwrap();
    assert_eq!(exp.text, "Hello, World! World says hi to World.");
    assert_eq!(exp.occurrences_of(1).len(), 3);
    for range in exp.occurrences_of(1) {
        assert_eq!(&exp.text[range.clone()], "World");
    }
}

#[test]
fn editing_one_occurrence_updates_every_linked_occurrence() {
    let snippet = Snippet::parse("Hello, ${1:World}! $1 says hi to $1.").unwrap();
    let mut overrides = HashMap::new();
    overrides.insert(1, "Rust".to_string());
    let exp = snippet.expand_with(&overrides).unwrap();
    assert_eq!(exp.text, "Hello, Rust! Rust says hi to Rust.");
    assert_eq!(exp.occurrences_of(1).len(), 3);
    for range in exp.occurrences_of(1) {
        assert_eq!(&exp.text[range.clone()], "Rust");
    }
}

#[test]
fn override_of_one_index_does_not_affect_another() {
    let snippet = Snippet::parse("${1:a}-${2:b}").unwrap();
    let mut overrides = HashMap::new();
    overrides.insert(1, "AAA".to_string());
    let exp = snippet.expand_with(&overrides).unwrap();
    assert_eq!(exp.text, "AAA-b");
}

#[test]
fn later_bare_occurrence_before_the_defaulted_one_still_links() {
    // $1 appears (bare) before its default is declared later in the text;
    // the value still applies to every occurrence in the final expansion.
    let exp = Snippet::parse("$1-${1:value}").unwrap().expand().unwrap();
    assert_eq!(exp.text, "value-value");
}

// ---------------------------------------------------------------------
// Guarded tab navigation
// ---------------------------------------------------------------------

#[test]
fn tab_navigation_with_no_placeholders_is_well_defined() {
    let exp = Snippet::parse("nothing to tab through")
        .unwrap()
        .expand()
        .unwrap();
    let mut stops = TabStops::new(&exp);
    assert!(stops.is_empty());
    assert_eq!(stops.current(), None);
    assert_eq!(stops.advance(), None);
    assert_eq!(stops.current(), None);
    assert_eq!(stops.retreat(), None);
    assert_eq!(stops.current(), None);
}

#[test]
fn tab_navigation_past_the_last_stop_stays_put_and_does_not_panic() {
    let exp = Snippet::parse("$1 $2").unwrap().expand().unwrap();
    let mut stops = TabStops::new(&exp);
    assert_eq!(stops.advance(), Some(1));
    assert_eq!(stops.advance(), Some(2));
    // Now at the last stop: further advances stay put.
    assert_eq!(stops.advance(), None);
    assert_eq!(stops.current(), Some(2));
    assert_eq!(stops.advance(), None);
    assert_eq!(stops.current(), Some(2));
}

#[test]
fn tab_navigation_before_the_first_stop_stays_put_and_does_not_panic() {
    let exp = Snippet::parse("$1 $2").unwrap().expand().unwrap();
    let mut stops = TabStops::new(&exp);
    assert_eq!(stops.retreat(), None);
    assert_eq!(stops.current(), None);
    assert_eq!(stops.advance(), Some(1));
    assert_eq!(stops.retreat(), None); // already at the first stop
    assert_eq!(stops.current(), Some(1));
}

#[test]
fn tab_navigation_round_trip() {
    let exp = Snippet::parse("$1 $2 $0").unwrap().expand().unwrap();
    let mut stops = TabStops::new(&exp);
    assert_eq!(stops.advance(), Some(1));
    assert_eq!(stops.advance(), Some(2));
    assert_eq!(stops.advance(), Some(0));
    assert_eq!(stops.advance(), None); // guarded: stays at 0
    assert_eq!(stops.retreat(), Some(2));
    assert_eq!(stops.retreat(), Some(1));
    assert_eq!(stops.retreat(), None); // guarded: stays at 1
    assert_eq!(stops.current(), Some(1));
}

// ---------------------------------------------------------------------
// Bounded: malformed input
// ---------------------------------------------------------------------

#[test]
fn unterminated_placeholder_is_a_typed_error() {
    let err = Snippet::parse("${1:abc").unwrap_err();
    assert!(matches!(err, SnippetError::UnterminatedPlaceholder { .. }));
}

#[test]
fn unterminated_brace_with_no_colon_is_a_typed_error() {
    let err = Snippet::parse("${1").unwrap_err();
    assert!(matches!(err, SnippetError::UnterminatedPlaceholder { .. }));
}

#[test]
fn missing_index_is_a_typed_error() {
    let err = Snippet::parse("${}").unwrap_err();
    assert!(matches!(err, SnippetError::InvalidPlaceholderIndex { .. }));
}

#[test]
fn missing_index_before_colon_is_a_typed_error() {
    let err = Snippet::parse("${:default}").unwrap_err();
    assert!(matches!(err, SnippetError::InvalidPlaceholderIndex { .. }));
}

#[test]
fn dangling_backslash_is_a_typed_error() {
    let err = Snippet::parse("abc\\").unwrap_err();
    assert!(matches!(err, SnippetError::UnterminatedEscape { .. }));
}

#[test]
fn invalid_escape_character_is_a_typed_error() {
    let err = Snippet::parse(r"\n").unwrap_err();
    assert_eq!(
        err,
        SnippetError::InvalidEscape {
            offset: 0,
            found: 'n'
        }
    );
}

#[test]
fn huge_placeholder_index_is_a_typed_error_not_a_panic() {
    let err = Snippet::parse("$99999999999999999999").unwrap_err();
    assert!(matches!(err, SnippetError::PlaceholderIndexTooLarge { .. }));
}

#[test]
fn index_just_above_the_bound_is_rejected_and_just_below_is_accepted() {
    let too_big = format!("${{{}}}", MAX_PLACEHOLDER_INDEX as u64 + 1);
    assert!(matches!(
        Snippet::parse(&too_big).unwrap_err(),
        SnippetError::PlaceholderIndexTooLarge { .. }
    ));

    let ok = format!("${{{MAX_PLACEHOLDER_INDEX}}}");
    assert!(Snippet::parse(&ok).is_ok());
}

#[test]
fn absurd_nesting_fails_with_a_typed_error_instead_of_looping() {
    // Build `${1:${2:${3:...}}}` one level past the documented bound.
    let depth = MAX_NESTING_DEPTH + 1;
    let mut src = String::new();
    for i in 0..depth {
        src.push_str(&format!("${{{}:", i + 1));
    }
    src.push('x');
    for _ in 0..depth {
        src.push('}');
    }
    let err = Snippet::parse(&src).unwrap_err();
    assert!(matches!(err, SnippetError::NestingTooDeep { .. }));
}

#[test]
fn nesting_exactly_at_the_bound_succeeds() {
    let depth = MAX_NESTING_DEPTH;
    let mut src = String::new();
    for i in 0..depth {
        src.push_str(&format!("${{{}:", i + 1));
    }
    src.push('x');
    for _ in 0..depth {
        src.push('}');
    }
    assert!(Snippet::parse(&src).is_ok());
}

#[test]
fn self_referential_placeholder_fails_with_a_typed_error() {
    let err = Snippet::parse("${1:$1}").unwrap().expand().unwrap_err();
    assert_eq!(err, SnippetError::SelfReferential { index: 1 });
}

#[test]
fn mutually_self_referential_placeholders_fail_with_a_typed_error() {
    let err = Snippet::parse("${1:$2}${2:$1}")
        .unwrap()
        .expand()
        .unwrap_err();
    assert!(matches!(err, SnippetError::SelfReferential { .. }));
}

#[test]
fn unused_self_referential_default_does_not_error_since_it_is_never_resolved() {
    // $1's value comes from the FIRST `${1:...}` ("fine"); a second,
    // later `${1:$1}` is a distinct occurrence whose own default text is
    // discarded per the documented first-default-wins rule and therefore
    // never evaluated - so the cycle buried inside it never triggers.
    // This pins down first-default-wins explicitly, since it is easy to
    // mis-implement as "last wins" or "always merge every default."
    let exp = Snippet::parse("${1:fine}${1:$1}")
        .unwrap()
        .expand()
        .unwrap();
    assert_eq!(exp.text, "finefine");
}

#[test]
fn overriding_a_self_referential_index_bypasses_the_cycle() {
    // Overrides short-circuit resolution entirely, so supplying text for
    // the very index that would otherwise cycle is not itself an error.
    let snippet = Snippet::parse("${1:$1}").unwrap();
    let mut overrides = HashMap::new();
    overrides.insert(1, "resolved".to_string());
    let exp = snippet.expand_with(&overrides).unwrap();
    assert_eq!(exp.text, "resolved");
}

#[test]
fn too_many_distinct_placeholders_is_a_typed_error() {
    let mut src = String::new();
    for i in 1..=flashtex_editor_snippets::MAX_PLACEHOLDERS + 1 {
        src.push_str(&format!("${i} "));
    }
    let err = Snippet::parse(&src).unwrap().expand().unwrap_err();
    assert!(matches!(err, SnippetError::TooManyPlaceholders { .. }));
}

#[test]
fn too_many_occurrences_is_a_typed_error() {
    let src = "$1 ".repeat(MAX_OCCURRENCES + 1);
    let err = Snippet::parse(&src).unwrap().expand().unwrap_err();
    assert!(matches!(err, SnippetError::TooManyOccurrences { .. }));
}

#[test]
fn oversized_source_is_rejected_before_parsing() {
    let src = "a".repeat(flashtex_editor_snippets::MAX_INPUT_BYTES + 1);
    let err = Snippet::parse(&src).unwrap_err();
    assert!(matches!(err, SnippetError::InputTooLarge { .. }));
}

// ---------------------------------------------------------------------
// UTF-8 safety
// ---------------------------------------------------------------------

#[test]
fn accented_latin_text_around_a_placeholder_has_exact_valid_offsets() {
    // "Caf\u{e9} " = "Caf" (3) + "\u{e9}" (2-byte) + " " (1) = 6 bytes.
    let exp = Snippet::parse("Café ${1:crème}!")
        .unwrap()
        .expand()
        .unwrap();
    assert_eq!(exp.text, "Café crème!");
    let range = &exp.occurrences_of(1)[0];
    assert!(exp.text.is_char_boundary(range.start));
    assert!(exp.text.is_char_boundary(range.end));
    assert_eq!(&exp.text[range.clone()], "crème");
}

#[test]
fn cjk_text_placeholder_offsets_never_split_a_character() {
    // Each CJK character below is 3 bytes in UTF-8.
    let exp = Snippet::parse("你好${1:世界}再见")
        .unwrap()
        .expand()
        .unwrap();
    assert_eq!(exp.text, "你好世界再见");
    let range = &exp.occurrences_of(1)[0];
    assert!(exp.text.is_char_boundary(range.start));
    assert!(exp.text.is_char_boundary(range.end));
    assert_eq!(&exp.text[range.clone()], "世界");
    // "你好" is 2 * 3 = 6 bytes.
    assert_eq!(range.start, 6);
    // "世界" is 2 * 3 = 6 bytes.
    assert_eq!(range.end, 12);
}

#[test]
fn emoji_including_multi_codepoint_sequence_never_splits_and_never_panics() {
    // A family emoji built from a ZWJ sequence: several 4-byte codepoints
    // joined by 3-byte ZWJ (U+200D) codepoints.
    let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
    let src = format!("before ${{1:{family}}} after $1 end");
    let exp = Snippet::parse(&src).unwrap().expand().unwrap();
    for range in exp.occurrences_of(1) {
        assert!(exp.text.is_char_boundary(range.start));
        assert!(exp.text.is_char_boundary(range.end));
        assert_eq!(&exp.text[range.clone()], family);
    }
    // Slicing at every reported boundary must not panic.
    let _ = &exp.text[..];
}

#[test]
fn escaping_a_dollar_immediately_after_multibyte_text_keeps_boundaries_valid() {
    let src = "日本語\\$100 and $1"; // literal '$100', then a real placeholder
    let exp = Snippet::parse(src).unwrap().expand().unwrap();
    assert_eq!(exp.text, "日本語$100 and ");
    for range in exp.occurrences_of(1) {
        assert!(exp.text.is_char_boundary(range.start));
        assert!(exp.text.is_char_boundary(range.end));
    }
}

#[test]
fn every_offset_over_a_battery_of_unicode_snippets_is_a_valid_char_boundary() {
    // A small sweep of tricky inputs mixing ASCII, accented Latin, CJK,
    // and emoji around every placeholder form this crate supports. This
    // is the closest thing to a fuzz test that stays dependency-free: it
    // proves the boundary-safety invariant holds across many shapes, not
    // just one hand-picked case.
    let cases = [
        "$1",
        "${1}",
        "${1:}",
        "€${1:100}",
        "${1:€}💶",
        "naïve $1 café",
        "🎉${1:🎊}🎉$1🎉",
        "混合 ${1:文字} と $1 mix",
        "${1:${2:नमस्ते}}$2",
        "a\u{0301}${1:b\u{0301}}c\u{0301}", // combining accents
    ];
    for src in cases {
        let snippet = Snippet::parse(src).unwrap_or_else(|e| panic!("parse {src:?}: {e}"));
        let exp = snippet
            .expand()
            .unwrap_or_else(|e| panic!("expand {src:?}: {e}"));
        for span in &exp.placeholders {
            for range in &span.occurrences {
                assert!(
                    exp.text.is_char_boundary(range.start) && exp.text.is_char_boundary(range.end),
                    "invalid boundary for {src:?}: {range:?} in {:?}",
                    exp.text
                );
                // Must not panic:
                let _ = &exp.text[range.clone()];
            }
        }
    }
}
