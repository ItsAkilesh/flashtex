//! Integration tests: entry validation (including malformed input),
//! measured leader-fill arithmetic (including Unicode titles), and bounded
//! front-matter convergence (including a case that never converges).

use flashtex_toc_layout::*;

// ---------------------------------------------------------------------
// EntryRecord / RelativeEntry: malformed input is rejected, not fudged.
// ---------------------------------------------------------------------

#[test]
fn entry_rejects_empty_title() {
    assert_eq!(EntryRecord::new("", 1, 0), Err(EntryError::EmptyTitle));
    assert_eq!(
        EntryRecord::new("   \t  ", 1, 0),
        Err(EntryError::EmptyTitle)
    );
}

#[test]
fn entry_rejects_page_zero() {
    assert_eq!(
        EntryRecord::new("Introduction", 0, 0),
        Err(EntryError::PageZero)
    );
}

#[test]
fn entry_rejects_level_too_deep() {
    assert_eq!(
        EntryRecord::new("Deep", 1, MAX_LEVEL + 1),
        Err(EntryError::LevelTooDeep {
            level: MAX_LEVEL + 1,
            max: MAX_LEVEL
        })
    );
    // Exactly at the bound is fine.
    assert!(EntryRecord::new("Just deep enough", 1, MAX_LEVEL).is_ok());
}

#[test]
fn relative_entry_rejects_body_offset_zero() {
    assert_eq!(
        RelativeEntry::new("Chapter 1", 0, 0),
        Err(EntryError::BodyOffsetZero)
    );
}

#[test]
fn relative_entry_resolves_by_adding_front_matter_pages() {
    let rel = RelativeEntry::new("Chapter 1", 0, 5).unwrap();
    let resolved = rel.resolve(3).unwrap();
    assert_eq!(resolved.page, 8);
    assert_eq!(resolved.title, "Chapter 1");
}

#[test]
fn relative_entry_reports_page_overflow_typed_not_panic() {
    let rel = RelativeEntry::new("Overflowing", 0, u32::MAX).unwrap();
    assert_eq!(rel.resolve(1), Err(EntryError::PageOverflow));
}

// ---------------------------------------------------------------------
// LineBox: malformed geometry is rejected.
// ---------------------------------------------------------------------

#[test]
fn line_box_rejects_non_finite_and_non_positive_width() {
    assert!(LineBox::new(f64::NAN, 0.0).is_err());
    assert!(LineBox::new(f64::INFINITY, 0.0).is_err());
    assert!(LineBox::new(0.0, 0.0).is_err());
    assert!(LineBox::new(-10.0, 0.0).is_err());
}

#[test]
fn line_box_rejects_negative_indent_unit() {
    assert!(LineBox::new(100.0, -1.0).is_err());
}

// ---------------------------------------------------------------------
// Measured leaders: real arithmetic, not just Ok(_).
// ---------------------------------------------------------------------

#[test]
fn leader_count_matches_hand_computed_value() {
    // 1pt/char, 2pt/leader-unit, line 50pt wide, no indent.
    let measure = CharWidthMeasure::new(1.0, 2.0);
    let line = LineBox::new(50.0, 0.0).unwrap();
    // "Results" = 7 chars = 7pt; page "12" = 2 chars = 2pt.
    let entry = EntryRecord::new("Results", 12, 0).unwrap();
    let laid_out = layout_entry(&entry, line, &measure).unwrap();

    assert_eq!(laid_out.title_width, 7.0);
    assert_eq!(laid_out.page_label, "12");
    assert_eq!(laid_out.page_label_width, 2.0);
    // available = 50 - 0 - 7 - 2 = 41; 41 / 2 = 20.5 -> 20 whole leader units.
    assert_eq!(laid_out.leader_count, 20);
    assert_eq!(laid_out.leader_width, 40.0);
    assert!((laid_out.gap_before_page - 1.0).abs() < 1e-9);
}

#[test]
fn leader_sum_reaches_exact_line_width() {
    // Indentation, non-round widths: the exact-sum invariant must still
    // hold, not just "roughly fits".
    let measure = CharWidthMeasure::new(3.7, 1.3);
    let line = LineBox::new(123.45, 10.0).unwrap();
    let entry = EntryRecord::new("Appendix: Data Tables", 108, 2).unwrap();
    let laid_out = layout_entry(&entry, line, &measure).unwrap();

    let total = laid_out.indent
        + laid_out.title_width
        + laid_out.leader_width
        + laid_out.gap_before_page
        + laid_out.page_label_width;
    assert!(
        (total - 123.45).abs() < 1e-9,
        "expected exact line width, got {total}"
    );
    // The remainder before the page number must be less than one leader
    // unit, or a whole extra dot should have been placed instead.
    assert!(laid_out.gap_before_page < 1.3);
    assert!(laid_out.gap_before_page >= 0.0);
}

#[test]
fn leader_indent_scales_with_level() {
    let measure = CharWidthMeasure::new(1.0, 1.0);
    let line = LineBox::new(100.0, 15.0).unwrap();
    let top = layout_entry(&EntryRecord::new("Top", 1, 0).unwrap(), line, &measure).unwrap();
    let nested = layout_entry(&EntryRecord::new("Nested", 1, 3).unwrap(), line, &measure).unwrap();
    assert_eq!(top.indent, 0.0);
    assert_eq!(nested.indent, 45.0);
}

#[test]
fn leader_overflow_is_typed_not_clamped() {
    let measure = CharWidthMeasure::new(10.0, 1.0);
    let line = LineBox::new(20.0, 0.0).unwrap();
    // "Way Too Long A Title" alone is already far wider than 20pt at 10pt/char.
    let entry = EntryRecord::new("Way Too Long A Title", 1, 0).unwrap();
    match layout_entry(&entry, line, &measure) {
        Err(LayoutError::Overflow { title, deficit }) => {
            assert_eq!(title, "Way Too Long A Title");
            assert!(deficit > 0.0);
        }
        other => panic!("expected Overflow, got {other:?}"),
    }
}

#[test]
fn leader_zero_leader_unit_yields_zero_dots_not_infinite_loop() {
    let measure = CharWidthMeasure::new(1.0, 0.0);
    let line = LineBox::new(50.0, 0.0).unwrap();
    let entry = EntryRecord::new("No Dots Possible", 1, 0).unwrap();
    let laid_out = layout_entry(&entry, line, &measure).unwrap();
    assert_eq!(laid_out.leader_count, 0);
    assert_eq!(laid_out.leader_width, 0.0);
}

#[test]
fn leader_negative_measurement_is_a_typed_error() {
    struct Broken;
    impl TextMeasure for Broken {
        fn width(&self, _text: &str) -> f64 {
            -5.0
        }
        fn leader_unit_width(&self) -> f64 {
            1.0
        }
    }
    let line = LineBox::new(50.0, 0.0).unwrap();
    let entry = EntryRecord::new("Anything", 1, 0).unwrap();
    match layout_entry(&entry, line, &Broken) {
        Err(LayoutError::InvalidMeasurement { source, .. }) => {
            assert_eq!(source, InvalidWidth(-5.0));
        }
        other => panic!("expected InvalidMeasurement, got {other:?}"),
    }
}

#[test]
fn layout_entries_fails_fast_on_first_overflowing_entry() {
    let measure = CharWidthMeasure::new(1.0, 1.0);
    let line = LineBox::new(20.0, 0.0).unwrap();
    let records = vec![
        EntryRecord::new("Fits Fine", 1, 0).unwrap(),
        EntryRecord::new("This One Definitely Does Not Fit At All", 2, 0).unwrap(),
        EntryRecord::new("Never Reached", 3, 0).unwrap(),
    ];
    assert!(matches!(
        layout_entries(&records, line, &measure),
        Err(LayoutError::Overflow { .. })
    ));
}

// ---------------------------------------------------------------------
// Unicode titles: multi-byte scalars measure correctly, not by byte length.
// ---------------------------------------------------------------------

#[test]
fn unicode_title_measures_by_char_not_by_byte() {
    // "résumé" is 6 Unicode scalars but 8 UTF-8 bytes (é is 2 bytes each).
    let title = "résumé";
    assert_eq!(title.chars().count(), 6);
    assert_eq!(title.len(), 8);

    let measure = CharWidthMeasure::new(2.0, 1.0);
    let line = LineBox::new(50.0, 0.0).unwrap();
    let entry = EntryRecord::new(title, 3, 0).unwrap();
    let laid_out = layout_entry(&entry, line, &measure).unwrap();

    // 6 chars * 2pt/char = 12pt, not 8 bytes worth.
    assert_eq!(laid_out.title_width, 12.0);
}

#[test]
fn unicode_cjk_title_with_per_char_override() {
    // Give CJK characters a wider per-character width than Latin ones.
    let measure = CharWidthMeasure::new(1.0, 1.0)
        .with_override('第', 2.0)
        .with_override('一', 2.0)
        .with_override('章', 2.0);
    let line = LineBox::new(30.0, 0.0).unwrap();
    let entry = EntryRecord::new("第一章", 7, 0).unwrap();
    let laid_out = layout_entry(&entry, line, &measure).unwrap();

    assert_eq!(laid_out.title_width, 6.0); // 3 chars * 2pt
    let total = laid_out.indent
        + laid_out.title_width
        + laid_out.leader_width
        + laid_out.gap_before_page
        + laid_out.page_label_width;
    assert!((total - 30.0).abs() < 1e-9);
}

#[test]
fn unicode_emoji_and_combining_mark_titles_do_not_panic_and_measure_by_char() {
    // A multi-codepoint emoji (flag = 2 regional indicators) and a
    // combining mark: this crate's reference measurer counts Unicode
    // scalars, so these count as 2 and 2 chars respectively (documented
    // simplification, not a grapheme-cluster claim).
    let flag = "\u{1F1EE}\u{1F1F3}"; // 2 chars
    let combining = "e\u{0301}"; // "e" + combining acute = 2 chars

    let measure = CharWidthMeasure::new(1.0, 1.0);
    let line = LineBox::new(50.0, 0.0).unwrap();

    let flag_entry = EntryRecord::new(flag, 4, 0).unwrap();
    let laid_out_flag = layout_entry(&flag_entry, line, &measure).unwrap();
    assert_eq!(laid_out_flag.title_width, 2.0);

    let combining_entry = EntryRecord::new(combining, 5, 0).unwrap();
    let laid_out_combining = layout_entry(&combining_entry, line, &measure).unwrap();
    assert_eq!(laid_out_combining.title_width, 2.0);
}

#[test]
fn unicode_rtl_title_is_accepted_and_treated_as_opaque_text() {
    // Right-to-left Arabic text: this crate lays out width only and does
    // not reorder or bidi-shape; it must still measure and fit correctly.
    let title = "الفهرس"; // "the index"
    let measure = CharWidthMeasure::new(1.5, 1.0);
    let line = LineBox::new(40.0, 0.0).unwrap();
    let entry = EntryRecord::new(title, 9, 0).unwrap();
    let laid_out = layout_entry(&entry, line, &measure).unwrap();
    let expected_title_width = title.chars().count() as f64 * 1.5;
    assert_eq!(laid_out.title_width, expected_title_width);
}

// ---------------------------------------------------------------------
// Bounded, revision-safe convergence.
// ---------------------------------------------------------------------

/// A model whose real page requirement does not depend on the candidate:
/// mirrors real TOC pagination, where the entry count decides the page
/// count regardless of the guess used to seed the resolved page numbers.
struct FixedNeedModel {
    needed: u32,
}
impl FrontMatterModel for FixedNeedModel {
    fn pages_for(&self, _candidate: u32) -> u32 {
        self.needed
    }
}

#[test]
fn convergence_settles_when_guess_matches_real_need() {
    let model = FixedNeedModel { needed: 3 };
    // Perfect first guess: converges immediately.
    assert_eq!(converge_front_matter_pages(&model, 3), Ok(3));
}

#[test]
fn convergence_takes_more_than_one_pass_from_a_bad_guess() {
    let model = FixedNeedModel { needed: 4 };
    // Guessing 1 first requires: pages_for(1) = 4 (changed), pages_for(4) =
    // 4 (stable) -> converges, but only after more than one candidate.
    assert_eq!(converge_front_matter_pages(&model, 1), Ok(4));
}

/// A model that oscillates between two values forever and can never
/// reach a fixed point — the bounded-termination case.
struct OscillatingModel {
    a: u32,
    b: u32,
}
impl FrontMatterModel for OscillatingModel {
    fn pages_for(&self, candidate: u32) -> u32 {
        if candidate == self.a { self.b } else { self.a }
    }
}

#[test]
fn convergence_terminates_with_typed_error_when_it_never_settles() {
    let model = OscillatingModel { a: 2, b: 5 };
    let result = converge_front_matter_pages(&model, 2);
    match result {
        Err(ConvergenceError { bound, history }) => {
            assert_eq!(bound, MAX_CONVERGENCE_ITERATIONS);
            // Bounded: exactly bound + 1 candidates were ever tried, never more.
            assert_eq!(history.len(), MAX_CONVERGENCE_ITERATIONS + 1);
            assert_eq!(history[0], 2);
            assert!(history.iter().all(|&v| v == 2 || v == 5));
        }
        Ok(v) => panic!("expected non-convergence, got Ok({v})"),
    }
}

#[test]
fn full_pipeline_relative_entry_through_converged_front_matter_to_layout() {
    // Explicit section/page records as body offsets; the contents list
    // needs 2 front-matter pages once its own entry count is accounted for.
    let model = FixedNeedModel { needed: 2 };
    let front_matter_pages = converge_front_matter_pages(&model, 1).unwrap();
    assert_eq!(front_matter_pages, 2);

    let rel = RelativeEntry::new("Conclusion", 0, 10).unwrap();
    let resolved = rel.resolve(front_matter_pages).unwrap();
    assert_eq!(resolved.page, 12);

    let measure = CharWidthMeasure::new(1.0, 1.0);
    let line = LineBox::new(30.0, 0.0).unwrap();
    let laid_out = layout_entry(&resolved, line, &measure).unwrap();
    assert_eq!(laid_out.page_label, "12");
}
