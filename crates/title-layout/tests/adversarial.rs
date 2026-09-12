//! Revision 3: adversarial bounds. Every hostile input here must come back
//! as a typed error or a bounded `Ok` — never a panic, never a hang. The
//! sharp case is non-finite metric values (zero/negative/infinite/NaN):
//! this crate must reject them before they can silently produce a nonsense
//! (negative, infinite, or NaN) baseline or centering offset, or make a
//! page-fit `>` comparison come out wrong (a NaN width compares `false`
//! against everything, so an unchecked check would never trip).

use std::collections::HashMap;

use flashtex_document_style::{BaseSize, ClassOptions, Geometry, Paper, Pt, Stylesheet};
use flashtex_title_layout::{
    DateField, DocumentClass, GlyphMetrics, TitleBlockInput, TitleLayoutError, layout_title_block,
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

/// A metrics provider that returns a fixed width for every character and a
/// fixed em, unless a character is listed in `missing` (then `None`) or in
/// `overrides` (then that exact value, hostile values included).
struct HostileMetrics {
    default_width: f64,
    em: Option<f64>,
    missing: Vec<char>,
    overrides: HashMap<char, f64>,
}

impl HostileMetrics {
    fn uniform(width: f64) -> Self {
        HostileMetrics {
            default_width: width,
            em: Some(width),
            missing: Vec::new(),
            overrides: HashMap::new(),
        }
    }

    fn with_override(mut self, ch: char, width: f64) -> Self {
        self.overrides.insert(ch, width);
        self
    }

    fn missing_char(mut self, ch: char) -> Self {
        self.missing.push(ch);
        self
    }

    fn with_em(mut self, em: f64) -> Self {
        self.em = Some(em);
        self
    }
}

impl GlyphMetrics for HostileMetrics {
    fn advance_width(&self, ch: char, _size: Pt) -> Option<Pt> {
        if self.missing.contains(&ch) {
            return None;
        }
        Some(Pt(*self.overrides.get(&ch).unwrap_or(&self.default_width)))
    }

    fn em(&self, _size: Pt) -> Option<Pt> {
        self.em.map(Pt)
    }
}

// ---- enormous length -------------------------------------------------------

#[test]
fn an_enormous_title_is_a_typed_error_not_a_hang_or_panic() {
    let sheet = sheet(BaseSize::Pt10);
    let huge_title: String = "x".repeat(500_000);
    let input = TitleBlockInput {
        title_lines: vec![huge_title],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    // Vertical-only layout doesn't measure width at all, so it succeeds
    // regardless of title length.
    assert!(layout_title_block(DocumentClass::Article, &sheet, &input).is_ok());

    // With metrics, half a million 1pt-wide characters is far wider than
    // any usable page: a typed error, not a hang or an overflowed width.
    let metrics = HostileMetrics::uniform(1.0);
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    assert!(matches!(err, TitleLayoutError::RowTooWide { .. }));
}

// ---- thousands of authors ---------------------------------------------------

#[test]
fn thousands_of_authors_is_a_bounded_result_not_a_hang() {
    let sheet = sheet(BaseSize::Pt10);
    let author_lines: Vec<Vec<String>> = (0..5000).map(|_| vec!["A".to_string()]).collect();
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines,
        date: DateField::Suppressed,
    };
    let layout = layout_title_block(DocumentClass::Article, &sheet, &input).unwrap();
    assert_eq!(layout.rows.len(), 1 + 5000);

    // With metrics, 5000 authors side by side are certainly wider than the
    // page: a typed error, never a panic from the running-width arithmetic.
    let metrics = HostileMetrics::uniform(1.0);
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    assert!(matches!(err, TitleLayoutError::AuthorGroupTooWide { .. }));
}

// ---- whitespace-only author line -------------------------------------------

#[test]
fn a_whitespace_only_author_line_is_rejected_not_measured() {
    let sheet = sheet(BaseSize::Pt10);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: vec![vec!["   \t  \u{a0}".to_string()]],
        date: DateField::Suppressed,
    };
    // `\u{a0}` (NBSP) is not ASCII whitespace but is still blank content in
    // spirit; the crate only recognizes `str::trim`'s notion of whitespace,
    // so confirm the plain-whitespace case is caught cleanly first.
    let plain = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: vec![vec!["   \t  ".to_string()]],
        date: DateField::Suppressed,
    };
    assert_eq!(
        layout_title_block(DocumentClass::Article, &sheet, &plain).unwrap_err(),
        TitleLayoutError::EmptyAuthorLine(0)
    );
    // The NBSP case must not panic or hang either way, whichever this
    // crate's blank-detection decides.
    let _ = layout_title_block(DocumentClass::Article, &sheet, &input);
}

// ---- metrics provider returns None mid-string ------------------------------

#[test]
fn a_metrics_provider_returning_none_mid_string_is_a_typed_error() {
    let sheet = sheet(BaseSize::Pt10);
    let metrics = HostileMetrics::uniform(5.0).missing_char('m');
    let input = TitleBlockInput {
        title_lines: vec!["so-me-title".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    match err {
        TitleLayoutError::MissingGlyphMetric { ch, .. } => assert_eq!(ch, 'm'),
        other => panic!("expected MissingGlyphMetric, got {other:?}"),
    }
}

// ---- non-finite / negative glyph widths: the sharp case --------------------

#[test]
fn zero_glyph_width_is_accepted_as_a_bounded_result() {
    // Zero is a legitimate (if unusual) width -- e.g. a combining mark a
    // real font renders with no advance -- and must not be confused with
    // the invalid (negative/non-finite) case.
    let sheet = sheet(BaseSize::Pt10);
    let metrics = HostileMetrics::uniform(0.0);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let measured =
        layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics).unwrap();
    assert_eq!(measured.extents[0].width, Pt(0.0));
}

#[test]
fn negative_glyph_width_is_a_typed_error_never_a_negative_layout() {
    let sheet = sheet(BaseSize::Pt10);
    let metrics = HostileMetrics::uniform(5.0).with_override('B', -3.0);
    let input = TitleBlockInput {
        title_lines: vec!["AB".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    match err {
        TitleLayoutError::InvalidGlyphMetric { ch, value, .. } => {
            assert_eq!(ch, 'B');
            assert_eq!(value, Pt(-3.0));
        }
        other => panic!("expected InvalidGlyphMetric, got {other:?}"),
    }
}

#[test]
fn infinite_glyph_width_is_a_typed_error_never_an_infinite_layout() {
    let sheet = sheet(BaseSize::Pt10);
    let metrics = HostileMetrics::uniform(5.0).with_override('B', f64::INFINITY);
    let input = TitleBlockInput {
        title_lines: vec!["AB".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    match err {
        TitleLayoutError::InvalidGlyphMetric { ch, value, .. } => {
            assert_eq!(ch, 'B');
            assert!(value.0.is_infinite());
        }
        other => panic!("expected InvalidGlyphMetric, got {other:?}"),
    }
}

#[test]
fn nan_glyph_width_is_a_typed_error_not_a_silently_accepted_layout() {
    // The sharpest case: a NaN width compares `false` against every `>`
    // check, so a naive "reject if width > text_width" bounds check would
    // never fire on it. This crate must catch it up front instead.
    let sheet = sheet(BaseSize::Pt10);
    let metrics = HostileMetrics::uniform(5.0).with_override('B', f64::NAN);
    let input = TitleBlockInput {
        title_lines: vec!["AB".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    match err {
        TitleLayoutError::InvalidGlyphMetric { ch, value, .. } => {
            assert_eq!(ch, 'B');
            assert!(value.0.is_nan());
        }
        other => panic!("expected InvalidGlyphMetric, got {other:?}"),
    }
}

#[test]
fn nan_em_metric_is_a_typed_error() {
    let sheet = sheet(BaseSize::Pt10);
    let metrics = HostileMetrics::uniform(5.0).with_em(f64::NAN);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: vec![vec!["A".to_string()], vec!["B".to_string()]],
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    match err {
        TitleLayoutError::InvalidEmMetric { value, .. } => assert!(value.0.is_nan()),
        other => panic!("expected InvalidEmMetric, got {other:?}"),
    }
}

#[test]
fn negative_infinite_em_metric_is_a_typed_error() {
    let sheet = sheet(BaseSize::Pt10);
    let metrics = HostileMetrics::uniform(5.0).with_em(f64::NEG_INFINITY);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: vec![vec!["A".to_string()], vec!["B".to_string()]],
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    assert!(matches!(err, TitleLayoutError::InvalidEmMetric { .. }));
}

// ---- degenerate page area ---------------------------------------------------

#[test]
fn zero_usable_text_width_is_a_typed_error_not_a_division_or_clip() {
    let sheet = sheet(BaseSize::Pt10).with_geometry(Geometry {
        textwidth: Some(Pt(0.0)),
        ..Geometry::default()
    });
    let metrics = HostileMetrics::uniform(5.0);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    assert!(matches!(err, TitleLayoutError::InvalidPageArea { .. }));
}

#[test]
fn negative_usable_text_width_is_a_typed_error() {
    // A `geometry`-style margin larger than half the paper drives textwidth
    // negative -- a hostile but reachable Stylesheet, not a hand-crafted one.
    let sheet = sheet(BaseSize::Pt10).with_geometry(Geometry::margin(Pt::inches(10.0)));
    assert!(sheet.page_layout().text_area.width.0 < 0.0);
    let metrics = HostileMetrics::uniform(5.0);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    assert!(matches!(err, TitleLayoutError::InvalidPageArea { .. }));
}

#[test]
fn negative_usable_text_height_is_a_typed_error() {
    let sheet = sheet(BaseSize::Pt10).with_geometry(Geometry {
        textheight: Some(Pt(-5.0)),
        ..Geometry::default()
    });
    let metrics = HostileMetrics::uniform(5.0);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    assert!(matches!(err, TitleLayoutError::InvalidPageArea { .. }));
}

#[test]
fn nonfinite_usable_text_height_is_a_typed_error() {
    let sheet = sheet(BaseSize::Pt10).with_geometry(Geometry {
        textheight: Some(Pt(f64::INFINITY)),
        ..Geometry::default()
    });
    let metrics = HostileMetrics::uniform(5.0);
    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    assert!(matches!(err, TitleLayoutError::InvalidPageArea { .. }));
}

// ---- combining marks and zero-width characters -----------------------------

#[test]
fn text_of_only_combining_marks_is_bounded_never_a_panic_or_hang() {
    let sheet = sheet(BaseSize::Pt10);
    // U+0301 COMBINING ACUTE ACCENT, repeated; no base character at all.
    let combining_title: String = "\u{0301}".repeat(50);
    let metrics = HostileMetrics::uniform(0.0);
    let input = TitleBlockInput {
        title_lines: vec![combining_title],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let measured =
        layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics).unwrap();
    assert_eq!(measured.extents[0].width, Pt(0.0));
}

#[test]
fn text_of_only_zero_width_characters_is_bounded_never_a_panic_or_hang() {
    let sheet = sheet(BaseSize::Pt10);
    // U+200B ZERO WIDTH SPACE and U+FEFF ZERO WIDTH NO-BREAK SPACE.
    let zw_title: String = "\u{200b}\u{feff}\u{200b}".repeat(20);
    let metrics = HostileMetrics::uniform(0.0);
    let input = TitleBlockInput {
        title_lines: vec![zw_title],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let measured =
        layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics).unwrap();
    assert_eq!(measured.extents[0].width, Pt(0.0));
    // Centered at width 0: x sits exactly at mid-page.
    let text_width = sheet.page_layout().text_area.width;
    assert_eq!(measured.extents[0].x, Pt(text_width.0 * 0.5));
}

#[test]
fn combining_marks_with_a_metrics_provider_returning_none_is_a_typed_error() {
    // A real font engine plausibly has no independent advance for a bare
    // combining mark and returns None; that must still be the ordinary
    // typed rejection, not a panic.
    let sheet = sheet(BaseSize::Pt10);
    let metrics = HostileMetrics::uniform(5.0).missing_char('\u{0301}');
    let input = TitleBlockInput {
        title_lines: vec!["\u{0301}\u{0301}".to_string()],
        author_lines: one_author("A"),
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet, &input, &metrics)
        .unwrap_err();
    match err {
        TitleLayoutError::MissingGlyphMetric { ch, .. } => assert_eq!(ch, '\u{0301}'),
        other => panic!("expected MissingGlyphMetric, got {other:?}"),
    }
}
