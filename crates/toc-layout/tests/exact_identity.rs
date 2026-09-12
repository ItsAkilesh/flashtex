//! Rev 3: exact identity regressions. Pins stabilized `stabilize_toc` +
//! `layout_entry` output for a representative multi-entry document as
//! literal expected values — leader counts, widths, gaps, and resolved
//! pages — checked by exact struct equality (`PartialEq`, no epsilon), so
//! any change to the arithmetic fails loudly rather than drifting.
//!
//! All measurements below (`2.5`, `1.5`, `5.0`, whole-number widths) are
//! exactly representable in `f64`, so `==` is safe and never a source of
//! flakiness from floating-point rounding.

use flashtex_toc_layout::*;

struct FixedNeedModel {
    needed: u32,
}
impl FrontMatterModel for FixedNeedModel {
    fn pages_for(&self, _candidate: u32) -> u32 {
        self.needed
    }
}

#[test]
fn stabilized_toc_pins_exact_front_matter_and_resolved_pages() {
    let model = FixedNeedModel { needed: 4 };
    let entries = vec![
        SourcedEntry {
            source: SourceId::new("thesis.tex", "rev-9"),
            sequence: 0,
            entry: RelativeEntry::new("Overview", 0, 1).unwrap(),
        },
        SourcedEntry {
            source: SourceId::new("thesis.tex", "rev-9"),
            sequence: 1,
            entry: RelativeEntry::new("Related Work", 1, 6).unwrap(),
        },
        SourcedEntry {
            source: SourceId::new("thesis.tex", "rev-9"),
            sequence: 2,
            entry: RelativeEntry::new("Evaluation", 2, 42).unwrap(),
        },
        SourcedEntry {
            source: SourceId::new("thesis.tex", "rev-9"),
            sequence: 3,
            entry: RelativeEntry::new("Appendix A", 3, 999).unwrap(),
        },
    ];

    let toc = stabilize_toc(&entries, &model, 0).unwrap();

    let expected = StabilizedToc {
        front_matter_pages: 4,
        entries: vec![
            StabilizedEntry {
                source: SourceId::new("thesis.tex", "rev-9"),
                sequence: 0,
                record: EntryRecord::new("Overview", 5, 0).unwrap(),
            },
            StabilizedEntry {
                source: SourceId::new("thesis.tex", "rev-9"),
                sequence: 1,
                record: EntryRecord::new("Related Work", 10, 1).unwrap(),
            },
            StabilizedEntry {
                source: SourceId::new("thesis.tex", "rev-9"),
                sequence: 2,
                record: EntryRecord::new("Evaluation", 46, 2).unwrap(),
            },
            StabilizedEntry {
                source: SourceId::new("thesis.tex", "rev-9"),
                sequence: 3,
                record: EntryRecord::new("Appendix A", 1003, 3).unwrap(),
            },
        ],
    };

    assert_eq!(toc, expected);
}

#[test]
fn laid_out_entries_pin_exact_leader_counts_widths_and_gaps() {
    // Same document as above, laid out against a fixed, exactly-representable
    // geometry: 2.5 units/char, 1.5 units/leader-dot, 5.0 units/indent
    // level, 100.0-unit-wide line.
    let model = FixedNeedModel { needed: 4 };
    let entries = vec![
        SourcedEntry {
            source: SourceId::new("thesis.tex", "rev-9"),
            sequence: 0,
            entry: RelativeEntry::new("Overview", 0, 1).unwrap(),
        },
        SourcedEntry {
            source: SourceId::new("thesis.tex", "rev-9"),
            sequence: 1,
            entry: RelativeEntry::new("Related Work", 1, 6).unwrap(),
        },
        SourcedEntry {
            source: SourceId::new("thesis.tex", "rev-9"),
            sequence: 2,
            entry: RelativeEntry::new("Evaluation", 2, 42).unwrap(),
        },
        SourcedEntry {
            source: SourceId::new("thesis.tex", "rev-9"),
            sequence: 3,
            entry: RelativeEntry::new("Appendix A", 3, 999).unwrap(),
        },
    ];
    let toc = stabilize_toc(&entries, &model, 0).unwrap();

    let measure = CharWidthMeasure::new(2.5, 1.5);
    let line = LineBox::new(100.0, 5.0).unwrap();
    let records: Vec<EntryRecord> = toc.entries.iter().map(|e| e.record.clone()).collect();
    let laid_out = layout_entries(&records, line, &measure).unwrap();

    let expected = vec![
        LaidOutEntry {
            title: "Overview".to_string(),
            level: 0,
            indent: 0.0,
            title_width: 20.0,
            page_label: "5".to_string(),
            page_label_width: 2.5,
            leader_count: 51,
            leader_width: 76.5,
            gap_before_page: 1.0,
        },
        LaidOutEntry {
            title: "Related Work".to_string(),
            level: 1,
            indent: 5.0,
            title_width: 30.0,
            page_label: "10".to_string(),
            page_label_width: 5.0,
            leader_count: 40,
            leader_width: 60.0,
            gap_before_page: 0.0,
        },
        LaidOutEntry {
            title: "Evaluation".to_string(),
            level: 2,
            indent: 10.0,
            title_width: 25.0,
            page_label: "46".to_string(),
            page_label_width: 5.0,
            leader_count: 40,
            leader_width: 60.0,
            gap_before_page: 0.0,
        },
        LaidOutEntry {
            title: "Appendix A".to_string(),
            level: 3,
            indent: 15.0,
            title_width: 25.0,
            page_label: "1003".to_string(),
            page_label_width: 10.0,
            leader_count: 33,
            leader_width: 49.5,
            gap_before_page: 0.5,
        },
    ];

    assert_eq!(laid_out, expected);

    // The exact-sum invariant must hold for every pinned entry too.
    for entry in &laid_out {
        let total = entry.indent
            + entry.title_width
            + entry.leader_width
            + entry.gap_before_page
            + entry.page_label_width;
        assert_eq!(total, 100.0);
    }
}
