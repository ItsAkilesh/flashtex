//! Revision 2: horizontal measurement bound to caller-supplied exact glyph
//! metrics. Covers the missing-metric rejection path, page-bounds checks,
//! and Unicode measurement — the gap revision 1 reported ("no horizontal
//! text measurement ... since that needs glyph metrics this crate doesn't
//! consume").

use std::collections::HashMap;

use flashtex_document_style::{BaseSize, ClassOptions, Geometry, Paper, Pt, Stylesheet};
use flashtex_title_layout::{
    DateField, DocumentClass, GlyphMetrics, RowKind, TitleBlockInput, TitleLayoutError,
    layout_title_block_with_metrics,
};

fn sheet(size: BaseSize) -> Stylesheet {
    Stylesheet::article(ClassOptions {
        paper: Paper::Letter,
        size,
    })
}

fn one_author(name: &str) -> Vec<Vec<String>> {
    vec![vec![name.to_string()]]
}

/// A caller-supplied metrics table for tests: exact per-character widths and
/// an optional em, both independent of the queried size (this crate always
/// queries the size it computed itself from `article.cls`'s tables, which
/// are already covered by `tests/title.rs`; these tests exercise binding,
/// rejection, and Unicode handling, not size-dependent scaling).
struct TestMetrics {
    widths: HashMap<char, f64>,
    em: Option<f64>,
}

impl TestMetrics {
    fn new(pairs: &[(char, f64)]) -> Self {
        TestMetrics {
            widths: pairs.iter().copied().collect(),
            em: Some(6.0),
        }
    }

    fn without_em(mut self) -> Self {
        self.em = None;
        self
    }
}

impl GlyphMetrics for TestMetrics {
    fn advance_width(&self, ch: char, _size: Pt) -> Option<Pt> {
        self.widths.get(&ch).copied().map(Pt)
    }

    fn em(&self, _size: Pt) -> Option<Pt> {
        self.em.map(Pt)
    }
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-9, "{what}: got {a}, expected {b}");
}

// ---- missing metrics are rejected, never fabricated ----------------------

#[test]
fn missing_glyph_metric_on_the_title_is_rejected_with_the_exact_character() {
    let sheet = sheet(BaseSize::Pt10);
    let metrics = TestMetrics::new(&[('A', 8.0)]); // 'B' has no entry
    let input = TitleBlockInput {
        title_lines: vec!["AB".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    match err {
        TitleLayoutError::MissingGlyphMetric { ch, .. } => assert_eq!(ch, 'B'),
        other => panic!("expected MissingGlyphMetric, got {other:?}"),
    }
}

#[test]
fn missing_glyph_metric_on_an_author_line_is_rejected() {
    let sheet = sheet(BaseSize::Pt10);
    let metrics = TestMetrics::new(&[('A', 8.0)]);
    let input = TitleBlockInput {
        title_lines: vec!["A".to_string()],
        author_lines: one_author("Zed"), // 'Z', 'e', 'd' all missing
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    assert!(matches!(err, TitleLayoutError::MissingGlyphMetric { .. }));
}

#[test]
fn missing_glyph_metric_on_the_date_is_rejected() {
    let sheet = sheet(BaseSize::Pt10);
    let metrics = TestMetrics::new(&[('A', 8.0)]);
    let input = TitleBlockInput {
        title_lines: vec!["A".to_string()],
        author_lines: one_author("A"),
        date: DateField::Text("today".to_string()), // none of these chars measured
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    match err {
        TitleLayoutError::MissingGlyphMetric { ch, .. } => assert_eq!(ch, 't'),
        other => panic!("expected MissingGlyphMetric, got {other:?}"),
    }
}

#[test]
fn missing_em_metric_is_rejected_only_when_more_than_one_author_needs_the_gap() {
    let sheet = sheet(BaseSize::Pt10);
    let input_two_authors = TitleBlockInput {
        title_lines: vec!["A".to_string()],
        author_lines: vec![vec!["A".to_string()], vec!["A".to_string()]],
        date: DateField::Suppressed,
    };
    let metrics_no_em = TestMetrics::new(&[('A', 5.0)]).without_em();
    let err = layout_title_block_with_metrics(
        DocumentClass::Article,
        &sheet,
        &input_two_authors,
        &metrics_no_em,
    )
    .unwrap_err();
    assert!(matches!(err, TitleLayoutError::MissingEmMetric { .. }));

    // A single author needs no inter-author gap, so this crate never asks
    // for an em it does not need.
    let input_one_author = TitleBlockInput {
        title_lines: vec!["A".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    assert!(
        layout_title_block_with_metrics(
            DocumentClass::Article,
            &sheet,
            &input_one_author,
            &metrics_no_em,
        )
        .is_ok()
    );
}

// ---- page bounds: reported explicitly, never clipped ----------------------

#[test]
fn a_title_line_wider_than_the_page_is_reported_not_clipped() {
    let sheet = sheet(BaseSize::Pt10);
    let text_width = sheet.page_layout().text_area.width;
    let input = TitleBlockInput {
        title_lines: vec!["W".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    // One enormous glyph, comfortably wider than the whole usable text width.
    let metrics = TestMetrics::new(&[('W', text_width.0 * 2.0), ('A', 5.0)]);
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    match err {
        TitleLayoutError::RowTooWide {
            row,
            natural_width,
            available_width,
        } => {
            assert_eq!(row, RowKind::TitleLine(0));
            close(available_width.0, text_width.0, "available width");
            assert!(natural_width.0 > available_width.0);
        }
        other => panic!("expected RowTooWide, got {other:?}"),
    }
}

#[test]
fn an_author_group_wider_than_the_page_is_reported_not_clipped() {
    let sheet = sheet(BaseSize::Pt10);
    let text_width = sheet.page_layout().text_area.width;
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: vec![vec!["A".to_string()], vec!["B".to_string()]],
        date: DateField::Suppressed,
    };
    let metrics = TestMetrics::new(&[('A', text_width.0), ('B', text_width.0), ('T', 5.0)]);
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    match err {
        TitleLayoutError::AuthorGroupTooWide {
            natural_width,
            available_width,
        } => {
            close(available_width.0, text_width.0, "available width");
            assert!(natural_width.0 > available_width.0);
        }
        other => panic!("expected AuthorGroupTooWide, got {other:?}"),
    }
}

#[test]
fn a_block_taller_than_a_squeezed_page_is_reported_not_clipped() {
    // Force a tiny usable text height via a `geometry`-style override so the
    // ordinary title/author/date block (whose own vertical math is already
    // covered by tests/title.rs) cannot possibly fit.
    let sheet = sheet(BaseSize::Pt10).with_geometry(Geometry {
        textheight: Some(Pt(5.0)),
        ..Geometry::default()
    });
    let metrics = TestMetrics::new(&[('T', 5.0), ('A', 5.0)]);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    match err {
        TitleLayoutError::BlockTallerThanPage {
            total_height,
            available_height,
        } => {
            close(available_height.0, 5.0, "available height");
            assert!(total_height.0 > available_height.0);
        }
        other => panic!("expected BlockTallerThanPage, got {other:?}"),
    }
}

// ---- Unicode: measured per scalar value, never per byte -------------------

#[test]
fn a_multi_byte_title_and_author_are_measured_per_character_not_per_byte() {
    let sheet = sheet(BaseSize::Pt10);
    // 'Ω' (U+03A9, 2 UTF-8 bytes) and '数' (U+6570, 3 UTF-8 bytes): if this
    // crate ever iterated bytes instead of chars, either it would panic on
    // a non-UTF-8-boundary split, or it would query far more than one
    // metric per character. Assign each its own exact width so any
    // byte-based bug shows up as a wrong sum or a spurious missing-metric
    // error.
    let metrics = TestMetrics::new(&[('Ω', 9.5), ('数', 12.25), ('=', 3.0)]);
    let input = TitleBlockInput {
        title_lines: vec!["Ω=数".to_string()],
        author_lines: vec![vec!["数Ω".to_string()]],
        date: DateField::Suppressed,
    };
    let measured =
        layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics).unwrap();

    // Exactly 3 chars were queried for the title, not `"Ω=数".len()` (6) bytes.
    let title_extent = measured.extents[0];
    close(title_extent.width.0, 9.5 + 3.0 + 12.25, "title width");

    let author_extent = measured.extents[1];
    close(author_extent.width.0, 12.25 + 9.5, "author width");

    // Both lines are still centered within the usable text width.
    let text_width = sheet.page_layout().text_area.width;
    close(
        title_extent.x.0,
        (text_width.0 - title_extent.width.0) / 2.0,
        "title centering",
    );
}

#[test]
fn unicode_title_is_rejected_the_same_way_ascii_is_when_a_glyph_is_missing() {
    let sheet = sheet(BaseSize::Pt10);
    let metrics = TestMetrics::new(&[('数', 12.0), ('A', 5.0)]); // 'Ω' missing
    let input = TitleBlockInput {
        title_lines: vec!["数Ω".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    match err {
        TitleLayoutError::MissingGlyphMetric { ch, .. } => assert_eq!(ch, 'Ω'),
        other => panic!("expected MissingGlyphMetric, got {other:?}"),
    }
}

// ---- centering and side-by-side placement arithmetic -----------------------

#[test]
fn a_single_line_title_is_centered_within_the_usable_text_width() {
    let sheet = sheet(BaseSize::Pt10);
    let metrics = TestMetrics::new(&[('A', 10.0), ('B', 20.0)]);
    let input = TitleBlockInput {
        title_lines: vec!["AB".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let measured =
        layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics).unwrap();
    let text_width = sheet.page_layout().text_area.width;
    let title_extent = measured.extents[0];
    close(title_extent.width.0, 30.0, "title width");
    close(
        title_extent.x.0,
        (text_width.0 - 30.0) / 2.0,
        "title x centered",
    );
}

#[test]
fn two_authors_are_placed_side_by_side_separated_by_one_em_and_centered_as_a_group() {
    let sheet = sheet(BaseSize::Pt10);
    let metrics = TestMetrics::new(&[('A', 10.0), ('B', 40.0)]); // em defaults to 6.0
    let input = TitleBlockInput {
        title_lines: vec!["A".to_string()],
        author_lines: vec![vec!["A".to_string()], vec!["B".to_string()]],
        date: DateField::Suppressed,
    };
    let measured =
        layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics).unwrap();
    let text_width = sheet.page_layout().text_area.width;

    let author_rows: Vec<_> = measured
        .layout
        .rows
        .iter()
        .zip(measured.extents.iter())
        .filter(|(r, _)| matches!(r.kind, RowKind::AuthorLine(_, _)))
        .collect();
    assert_eq!(author_rows.len(), 2);

    let (row0, extent0) = author_rows[0];
    let (row1, extent1) = author_rows[1];
    assert_eq!(row0.kind, RowKind::AuthorLine(0, 0));
    assert_eq!(row1.kind, RowKind::AuthorLine(1, 0));

    // total width = 10 + 40 + 1em(6.0) = 56.0, centered as a group.
    let total_width = 56.0;
    let group_x = (text_width.0 - total_width) / 2.0;
    close(extent0.x.0, group_x, "first author x");
    close(extent0.width.0, 10.0, "first author width");
    // second author starts after the first author's width plus the 1em gap.
    close(extent1.x.0, group_x + 10.0 + 6.0, "second author x");
    close(extent1.width.0, 40.0, "second author width");
}

// ---- revision 1 behaviour preserved: unsupported classes never approximate ----

#[test]
fn unsupported_document_classes_still_fail_explicitly_with_metrics() {
    let sheet = sheet(BaseSize::Pt10);
    let metrics = TestMetrics::new(&[('A', 5.0)]);
    let input = TitleBlockInput {
        title_lines: vec!["A".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    for class in [
        DocumentClass::Report,
        DocumentClass::Book,
        DocumentClass::Letter,
    ] {
        let err = layout_title_block_with_metrics(class, &sheet, &input, &metrics).unwrap_err();
        assert_eq!(err, TitleLayoutError::UnsupportedDocumentClass(class));
    }
}
