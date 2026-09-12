//! Exact-number tests of `layout_abstract` against hand-computed values from
//! `article.cls` v1.4n (see `src/abstract_block.rs`).

use flashtex_document_style::{BaseSize, ClassOptions, Paper, Skip, Stylesheet};
use flashtex_title_layout::{DocumentClass, TitleLayoutError, layout_abstract};

fn sheet(size: BaseSize) -> Stylesheet {
    Stylesheet::article(ClassOptions {
        paper: Paper::Letter,
        size,
    })
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6, "{what}: got {a}, expected {b}");
}

#[test]
fn pt10_abstract_matches_hand_computed_small_size_and_list_level_one() {
    // 10pt: \small = 9.0pt/11.0, \@listi leftmargin = 2.5em(normalsize) =
    // 25.00005pt, topsep = 8pt plus 2pt minus 4pt, \parskip = 0pt plus 1pt.
    let sheet = sheet(BaseSize::Pt10);
    let layout = layout_abstract(DocumentClass::Article, &sheet).unwrap();

    close(layout.heading_font_size.0, 9.0, "heading font size");
    close(layout.heading_baselineskip.0, 11.0, "heading baselineskip");
    close(layout.body_font_size.0, 9.0, "body font size");
    close(layout.body_baselineskip.0, 11.0, "body baselineskip");
    close(layout.left_margin.0, 25.00005, "left margin");
    close(layout.right_margin.0, 25.00005, "right margin");
    assert_eq!(layout.left_margin, layout.right_margin);

    // topsep(8,+2,-4) + parskip(0,+1,0) = (8, 3, 4)
    close(layout.gap_before_heading.pt, 8.0, "gap before heading pt");
    close(
        layout.gap_before_heading.plus,
        3.0,
        "gap before heading plus",
    );
    close(
        layout.gap_before_heading.minus,
        4.0,
        "gap before heading minus",
    );

    // heading-to-body: addvspace of two equal (8,3,4) skips is itself (8,3,4).
    assert_eq!(layout.gap_heading_to_body, layout.gap_before_heading);

    // quotation overrides \parsep to 0pt plus 1pt, unconditionally.
    assert_eq!(layout.paragraph_gap, Skip::new(0.0, 1.0, 0.0));
}

#[test]
fn base_size_changes_shift_small_font_and_left_margin() {
    let ten = layout_abstract(DocumentClass::Article, &sheet(BaseSize::Pt10)).unwrap();
    let twelve = layout_abstract(DocumentClass::Article, &sheet(BaseSize::Pt12)).unwrap();
    assert_ne!(ten.heading_font_size.0, twelve.heading_font_size.0);
    assert_ne!(ten.left_margin.0, twelve.left_margin.0);
    // 12pt \small (10.95) is larger than 10pt \small (9.0).
    assert!(twelve.heading_font_size.0 > ten.heading_font_size.0);
    // quotation's \parsep override is a fixed literal, independent of size.
    assert_eq!(ten.paragraph_gap, twelve.paragraph_gap);
}

#[test]
fn user_parskip_delta_propagates_into_the_list_gaps() {
    use flashtex_document_style::StyleDelta;
    let base = sheet(BaseSize::Pt10);
    let custom = base.clone().with_delta(StyleDelta {
        rules: vec![],
        parskip: Some(Skip::new(5.0, 0.0, 0.0)),
    });
    let default_layout = layout_abstract(DocumentClass::Article, &base).unwrap();
    let custom_layout = layout_abstract(DocumentClass::Article, &custom).unwrap();
    assert_ne!(
        default_layout.gap_before_heading,
        custom_layout.gap_before_heading
    );
    // topsep(8,+2,-4) + parskip(5,0,0) = (13, 2, 4)
    close(custom_layout.gap_before_heading.pt, 13.0, "custom gap pt");
}

#[test]
fn unsupported_document_classes_fail_explicitly_never_approximate() {
    let sheet = sheet(BaseSize::Pt10);
    for class in [
        DocumentClass::Report,
        DocumentClass::Book,
        DocumentClass::Letter,
    ] {
        assert_eq!(
            layout_abstract(class, &sheet).unwrap_err(),
            TitleLayoutError::UnsupportedDocumentClass(class)
        );
    }
    assert!(layout_abstract(DocumentClass::Article, &sheet).is_ok());
}
