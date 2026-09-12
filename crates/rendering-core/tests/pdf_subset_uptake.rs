//! Old artifacts are retained; compare semantic identities before new snapshots.
#[allow(dead_code)]
#[path = "../examples/pdf_subset_compare.rs"]
mod probe;
#[test]
fn exact_subset_geometry_text_and_programs_match_all_existing_candidates() {
    let cases: &[(&[u8], &[u8])] = &[
        (
            include_bytes!("fixtures/original-reference/65dbe7d-clean-searchable.pdf"),
            include_bytes!("fixtures/pdf-subset-20e5277/plain.pdf"),
        ),
        (
            include_bytes!("fixtures/math-reference/original.pdf"),
            include_bytes!("fixtures/pdf-subset-20e5277/inline-math.pdf"),
        ),
        (
            include_bytes!("fixtures/display-math-reference/original.pdf"),
            include_bytes!("fixtures/pdf-subset-20e5277/display-math.pdf"),
        ),
        (
            include_bytes!("fixtures/wrapping-reference/original.pdf"),
            include_bytes!("fixtures/pdf-subset-20e5277/wrapping.pdf"),
        ),
        (
            include_bytes!("fixtures/ligatures-reference/original.pdf"),
            include_bytes!("fixtures/pdf-subset-20e5277/ligatures.pdf"),
        ),
        (
            include_bytes!("fixtures/original-reference/escaped-searchable.pdf"),
            include_bytes!("fixtures/pdf-subset-20e5277/escaped.pdf"),
        ),
    ];
    for (old, new) in cases {
        let report = probe::compare(old, new).unwrap();
        assert_eq!(report["exact_original_gid_and_positions_equal"], true);
        assert!(new.len() < old.len());
    }
    assert!(probe::compare(b"not a PDF", cases[0].1).is_err());
}
