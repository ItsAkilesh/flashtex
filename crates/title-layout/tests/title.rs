//! Exact-number tests of `layout_title_block` against hand-computed
//! `\@maketitle` values from `article.cls` v1.4n (see `src/title.rs`).

use flashtex_document_style::{BaseSize, ClassOptions, Paper, Stylesheet};
use flashtex_title_layout::{
    DateField, DocumentClass, RowKind, TitleBlockInput, TitleLayoutError, layout_title_block,
};

fn sheet(size: BaseSize) -> Stylesheet {
    Stylesheet::article(ClassOptions {
        paper: Paper::Letter,
        size,
    })
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6, "{what}: got {a}, expected {b}");
}

fn one_author(name: &str) -> Vec<Vec<String>> {
    vec![vec![name.to_string()]]
}

#[test]
fn pt10_single_line_title_author_date_matches_hand_computed_baselines() {
    // 10pt: normalsize em = 10.00002pt, LARGE (17.28pt/22.0), large (12pt/14.0).
    let sheet = sheet(BaseSize::Pt10);
    let input = TitleBlockInput {
        title_lines: vec!["A Measured Title".to_string()],
        author_lines: one_author("A. Author"),
        date: DateField::Text("\\today".to_string()),
    };
    let layout = layout_title_block(DocumentClass::Article, &sheet, &input).unwrap();
    assert_eq!(layout.rows.len(), 3);

    let title = layout.rows[0];
    assert_eq!(title.kind, RowKind::TitleLine(0));
    close(title.font_size.0, 17.28, "title font size");
    // y = 2em (20.00004) + LARGE baselineskip (22.0)
    close(title.baseline_y.0, 20.00004 + 22.0, "title baseline");

    let author = layout.rows[1];
    assert_eq!(author.kind, RowKind::AuthorLine(0, 0));
    close(author.font_size.0, 12.0, "author font size");
    // + 1.5em (15.00003) + large baselineskip (14.0)
    close(
        author.baseline_y.0,
        20.00004 + 22.0 + 15.00003 + 14.0,
        "author baseline",
    );

    let date = layout.rows[2];
    assert_eq!(date.kind, RowKind::DateLine);
    close(date.font_size.0, 12.0, "date font size");
    // + 1em (10.00002) + large baselineskip (14.0)
    close(
        date.baseline_y.0,
        20.00004 + 22.0 + 15.00003 + 14.0 + 10.00002 + 14.0,
        "date baseline",
    );

    // total = date baseline + trailing 1.5em
    close(
        layout.total_height.0,
        date.baseline_y.0 + 15.00003,
        "total height",
    );
}

#[test]
fn suppressed_date_omits_the_row_but_still_advances_the_gap() {
    let sheet = sheet(BaseSize::Pt10);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let layout = layout_title_block(DocumentClass::Article, &sheet, &input).unwrap();
    // title + author only, no date row.
    assert_eq!(layout.rows.len(), 2);
    assert!(!layout.rows.iter().any(|r| r.kind == RowKind::DateLine));

    let author_baseline = layout.rows[1].baseline_y;
    // Suppressing the date still leaves the unconditional `\vskip 1em` plus
    // the trailing `\vskip 1.5em`, since \@maketitle inserts them regardless
    // of whether \@date has content.
    let body_em = sheet.body_font();
    close(
        layout.total_height.0,
        author_baseline.0 + body_em.em(1.0).0 + body_em.em(1.5).0,
        "total height with suppressed date",
    );
}

#[test]
fn multi_line_authors_share_row_indices_without_side_by_side_placement() {
    let sheet = sheet(BaseSize::Pt10);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: vec![
            vec!["Alice".to_string(), "Dept. A".to_string()],
            vec!["Bob".to_string()],
        ],
        date: DateField::Suppressed,
    };
    let layout = layout_title_block(DocumentClass::Article, &sheet, &input).unwrap();
    // title(1) + author rows: max(2,1) = 2 tabular rows, but only 3 lines of
    // author text total (Alice/Dept. A on row 0/1, Bob only on row 0).
    let author_rows: Vec<_> = layout
        .rows
        .iter()
        .filter(|r| matches!(r.kind, RowKind::AuthorLine(_, _)))
        .collect();
    assert_eq!(author_rows.len(), 3);
    let bob_row = author_rows
        .iter()
        .find(|r| r.kind == RowKind::AuthorLine(1, 0))
        .expect("bob's single line is present");
    let alice_line1 = author_rows
        .iter()
        .find(|r| r.kind == RowKind::AuthorLine(0, 0))
        .unwrap();
    // Bob (one line) shares row 0's baseline with Alice's first line.
    close(
        bob_row.baseline_y.0,
        alice_line1.baseline_y.0,
        "shared baseline",
    );
    // Alice has no second-row author entry beyond her own second line.
    assert!(
        !author_rows
            .iter()
            .any(|r| r.kind == RowKind::AuthorLine(1, 1))
    );
}

#[test]
fn base_size_changes_shift_every_font_and_skip() {
    let ten = layout_title_block(
        DocumentClass::Article,
        &sheet(BaseSize::Pt10),
        &TitleBlockInput {
            title_lines: vec!["T".to_string()],
            author_lines: one_author("A"),
            date: DateField::Suppressed,
        },
    )
    .unwrap();
    let twelve = layout_title_block(
        DocumentClass::Article,
        &sheet(BaseSize::Pt12),
        &TitleBlockInput {
            title_lines: vec!["T".to_string()],
            author_lines: one_author("A"),
            date: DateField::Suppressed,
        },
    )
    .unwrap();
    assert_ne!(ten.rows[0].font_size.0, twelve.rows[0].font_size.0);
    assert_ne!(ten.total_height.0, twelve.total_height.0);
    // 12pt LARGE (20.74) is larger than 10pt LARGE (17.28).
    assert!(twelve.rows[0].font_size.0 > ten.rows[0].font_size.0);
}

#[test]
fn unicode_title_author_and_date_measure_identically_to_ascii() {
    let sheet = sheet(BaseSize::Pt11);
    let ascii = TitleBlockInput {
        title_lines: vec!["Title".to_string()],
        author_lines: one_author("Name"),
        date: DateField::Text("Date".to_string()),
    };
    let unicode = TitleBlockInput {
        title_lines: vec!["Über Kryptographie und Größenordnung: 数理論理学".to_string()],
        author_lines: vec![vec![
            "Renée Müller-Åström".to_string(),
            "北京大学".to_string(),
        ]],
        date: DateField::Text("Ⅷ日 二〇二六年".to_string()),
    };
    let a = layout_title_block(DocumentClass::Article, &sheet, &ascii).unwrap();
    let u = layout_title_block(DocumentClass::Article, &sheet, &unicode).unwrap();
    // Row count differs (unicode author has two lines), but the baseline
    // math for the rows that do exist is identical: this crate measures
    // vertical layout only, never a function of glyph content.
    assert_eq!(a.rows[0].baseline_y, u.rows[0].baseline_y); // title
    assert_eq!(a.rows[0].font_size, u.rows[0].font_size);
    assert_eq!(u.rows.len(), 1 + 2 + 1); // title + 2 author lines + date
    let unicode_date = u.rows.last().unwrap();
    assert_eq!(unicode_date.kind, RowKind::DateLine);
}

#[test]
fn unsupported_document_classes_fail_explicitly_never_approximate() {
    let sheet = sheet(BaseSize::Pt10);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: one_author("A"),
        date: DateField::Text("D".to_string()),
    };
    for class in [
        DocumentClass::Report,
        DocumentClass::Book,
        DocumentClass::Letter,
    ] {
        let err = layout_title_block(class, &sheet, &input).unwrap_err();
        assert_eq!(err, TitleLayoutError::UnsupportedDocumentClass(class));
    }
    assert!(layout_title_block(DocumentClass::Article, &sheet, &input).is_ok());
}

#[test]
fn malformed_blank_title_is_rejected() {
    let sheet = sheet(BaseSize::Pt10);
    let input = TitleBlockInput {
        title_lines: vec!["   ".to_string(), "\t".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    assert_eq!(
        layout_title_block(DocumentClass::Article, &sheet, &input).unwrap_err(),
        TitleLayoutError::EmptyTitle
    );
}

#[test]
fn malformed_no_authors_is_rejected() {
    let sheet = sheet(BaseSize::Pt10);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: vec![],
        date: DateField::Suppressed,
    };
    assert_eq!(
        layout_title_block(DocumentClass::Article, &sheet, &input).unwrap_err(),
        TitleLayoutError::NoAuthors
    );
}

#[test]
fn malformed_blank_author_line_is_rejected_with_its_index() {
    let sheet = sheet(BaseSize::Pt10);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: vec![vec!["Real Author".to_string()], vec!["  ".to_string()]],
        date: DateField::Suppressed,
    };
    assert_eq!(
        layout_title_block(DocumentClass::Article, &sheet, &input).unwrap_err(),
        TitleLayoutError::EmptyAuthorLine(1)
    );
}

#[test]
fn malformed_blank_date_text_is_rejected_distinctly_from_suppressed() {
    let sheet = sheet(BaseSize::Pt10);
    let mut input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: one_author("A"),
        date: DateField::Text("   ".to_string()),
    };
    assert_eq!(
        layout_title_block(DocumentClass::Article, &sheet, &input).unwrap_err(),
        TitleLayoutError::EmptyDate
    );
    input.date = DateField::Suppressed;
    assert!(layout_title_block(DocumentClass::Article, &sheet, &input).is_ok());
}
