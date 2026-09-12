//! Revision 3: exact identity regressions.
//!
//! Every value asserted here is a literal, pinned expected number (not an
//! epsilon-bounded `close()` comparison, and not a value recomputed by
//! calling back into `flashtex-document-style`'s own tables from the test).
//! `assert_eq!` on `f64`/`Pt` is used deliberately: a real change to a
//! `\baselineskip`, a size table entry, an em conversion, or the vertical
//! arithmetic in `src/title.rs` must flip one of these literals and fail
//! loudly, rather than sliding through under a tolerance. The literals below
//! were captured from one known-good run of this crate's own code (they are
//! not hand-derived from `article.cls` a second time -- `tests/title.rs`
//! already covers that cross-check with an epsilon) and are pinned here as
//! the committed baseline.

use std::collections::HashMap;

use flashtex_document_style::{BaseSize, ClassOptions, Paper, Pt, Stylesheet};
use flashtex_title_layout::{
    DateField, DocumentClass, GlyphMetrics, MeasuredRow, RowKind, TitleBlockInput,
    layout_title_block, layout_title_block_with_metrics,
};

fn sheet(size: BaseSize) -> Stylesheet {
    Stylesheet::article(ClassOptions {
        paper: Paper::Letter,
        size,
    })
}

#[test]
fn pt10_single_title_author_date_pins_every_row_exactly() {
    let sheet = sheet(BaseSize::Pt10);
    let input = TitleBlockInput {
        title_lines: vec!["A Measured Title".to_string()],
        author_lines: vec![vec!["A. Author".to_string()]],
        date: DateField::Text("\\today".to_string()),
    };
    let layout = layout_title_block(DocumentClass::Article, &sheet, &input).unwrap();

    let expected = vec![
        MeasuredRow {
            kind: RowKind::TitleLine(0),
            font_size: Pt(17.28),
            baseline_y: Pt(42.00004),
        },
        MeasuredRow {
            kind: RowKind::AuthorLine(0, 0),
            font_size: Pt(12.0),
            baseline_y: Pt(71.00007),
        },
        MeasuredRow {
            kind: RowKind::DateLine,
            font_size: Pt(12.0),
            baseline_y: Pt(95.00009),
        },
    ];
    assert_eq!(layout.rows, expected);
    assert_eq!(layout.total_height, Pt(110.00012));
}

#[test]
fn pt12_two_title_lines_two_authors_suppressed_date_pins_every_row_exactly() {
    let sheet = sheet(BaseSize::Pt12);
    let input = TitleBlockInput {
        title_lines: vec!["T1".to_string(), "T2".to_string()],
        author_lines: vec![
            vec!["Alice".to_string(), "Dept A".to_string()],
            vec!["Bob".to_string()],
        ],
        date: DateField::Suppressed,
    };
    let layout = layout_title_block(DocumentClass::Article, &sheet, &input).unwrap();

    let expected = vec![
        MeasuredRow {
            kind: RowKind::TitleLine(0),
            font_size: Pt(20.74),
            baseline_y: Pt(48.499759999999995),
        },
        MeasuredRow {
            kind: RowKind::TitleLine(1),
            font_size: Pt(20.74),
            baseline_y: Pt(73.49976),
        },
        MeasuredRow {
            kind: RowKind::AuthorLine(0, 0),
            font_size: Pt(14.4),
            baseline_y: Pt(109.12458),
        },
        MeasuredRow {
            kind: RowKind::AuthorLine(1, 0),
            font_size: Pt(14.4),
            baseline_y: Pt(109.12458),
        },
        MeasuredRow {
            kind: RowKind::AuthorLine(0, 1),
            font_size: Pt(14.4),
            baseline_y: Pt(127.12458),
        },
    ];
    assert_eq!(layout.rows, expected);
    assert_eq!(layout.total_height, Pt(156.49928));
}

/// A fixed reference metrics table, used only to pin exact horizontal
/// geometry (widths, centering, and the two-author side-by-side layout).
struct FixedMetrics(HashMap<char, f64>, f64);

impl GlyphMetrics for FixedMetrics {
    fn advance_width(&self, ch: char, _size: Pt) -> Option<Pt> {
        self.0.get(&ch).copied().map(Pt)
    }
    fn em(&self, _size: Pt) -> Option<Pt> {
        Some(Pt(self.1))
    }
}

#[test]
fn pt10_letter_page_horizontal_geometry_pins_widths_and_x_exactly() {
    let sheet = sheet(BaseSize::Pt10);
    // Pinning the page's own usable text width too: if `flashtex-document-style`
    // ever changes the `article`/letter default text width, every centering
    // offset below silently shifts, so pin the width this test depends on.
    assert_eq!(sheet.page_layout().text_area.width, Pt(345.0));

    let widths: HashMap<char, f64> = [('A', 8.0), ('B', 12.0), ('C', 6.5), ('T', 5.0)]
        .into_iter()
        .collect();
    let metrics = FixedMetrics(widths, 6.0);
    let input = TitleBlockInput {
        title_lines: vec!["AB".to_string()],
        author_lines: vec![
            vec!["A".to_string()],
            vec!["B".to_string(), "C".to_string()],
        ],
        date: DateField::Text("T".to_string()),
    };
    let measured =
        layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics).unwrap();

    let extents: Vec<(f64, f64)> = measured
        .extents
        .iter()
        .map(|e| (e.width.0, e.x.0))
        .collect();
    assert_eq!(
        extents,
        vec![
            (20.0, 162.5), // TitleLine(0): "AB" = 8+12, centered in 345
            (8.0, 159.5),  // AuthorLine(0,0): "A", its own column centered
            (12.0, 173.5), // AuthorLine(1,0): "B", first line of author 1's column
            (6.5, 176.25), // AuthorLine(1,1): "C", second line, narrower, re-centered
            (5.0, 170.0),  // DateLine: "T", centered in 345
        ]
    );
    assert_eq!(measured.layout.total_height, Pt(124.00012));
}
