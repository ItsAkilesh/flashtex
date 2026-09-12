//! Revision 4 consumer-integration fixture.
//!
//! Rev 3 reported that no consumer of `flashtex_title_layout` exists. Rev 4
//! asked to look harder and wider before accepting that. Repeating the
//! search across the whole repo, not just this crate's own dependents:
//!
//! ```sh
//! grep -rn 'title-layout\|title_layout' --include='*.toml' --include='*.rs' . \
//!   | grep -v '/target/' | grep -v '^crates/title-layout/'
//! # (no output — nothing references this crate's package or module name)
//! ```
//!
//! and by hand against the three places rev 4 named specifically:
//!
//! - `crates/document-runtime`: a JSON-Lines transport that shells out to
//!   the compiler binary and decodes its protocol frames (see its
//!   `lib.rs`: "Persistent original-compiler transport"). No document
//!   layout, title, or metrics concern of any kind lives there.
//! - `crates/compiler`: its parser records `Parsed::document_class:
//!   Option<String>` from `\documentclass{...}` (`src/parser.rs`), but its
//!   built-in command table (`BUILT_INS` in the same file) has no
//!   `title`, `author`, `date`, or `maketitle` entry — grepping
//!   `"title"|"author"|"date"|"maketitle"` across `crates/compiler/src/`
//!   returns nothing. It parses *which class* a document declares; it does
//!   not parse a title block at all.
//! - `apps/mac`: a Swift client of the compiler's JSON protocol
//!   (`Sources/FlashTeXProtocol`, `FlashTeXMac`) — no Rust code to
//!   integrate against, and no title/author/date handling visible in its
//!   sources either.
//!
//! So there is genuinely no existing consumer of this crate's title/author/
//! date or abstract layout. This fixture instead drives
//! `layout_title_block_with_metrics` from the two real, in-repo types
//! closest to what an actual caller would supply — real inputs, not the
//! hand-made `TestMetrics`/one-off `Stylesheet` pairing `tests/metrics.rs`
//! already covers for the binding/rejection/Unicode-handling contract:
//!
//! - **Document class**: `flashtex_compiler::parser::parse` is a real,
//!   in-repo LaTeX preamble parser. This fixture runs it over one of the
//!   project's own corpus documents
//!   (`tests/tex-corpus/cases/plain-paragraphs/main.tex`) and feeds its
//!   `Parsed::document_class` straight into `DocumentClass::parse` — the
//!   exact string shape a real caller in this repo produces today, not a
//!   literal written for this test.
//! - **Horizontal metrics**: [`RealFaceMetrics`] implements this crate's
//!   [`GlyphMetrics`] over `flashtex_font_engine::Face`, the same trait
//!   `flashtex-font-engine`'s own `adapters::paragraph::FaceMetrics`
//!   reads to hand `flashtex-paragraph-layout` real advances (see that
//!   module for the pattern this mirrors). Font-engine has no
//!   title-layout adapter of its own to reuse — nothing calls this crate,
//!   so nothing needed one — so this fixture builds the equivalent from
//!   title-layout's side, without editing `flashtex-font-engine`. It loads
//!   the real Latin Modern Roman OTF from the local BasicTeX install, the
//!   same file and path `flashtex-font-engine/tests/latin_modern.rs`
//!   already uses, and skips (with a message, not a failure) if that
//!   install isn't present — matching that test's own convention.
//!
//! `Stylesheet::article` is already this crate's one real, direct
//! dependency, used here exactly as `tests/title.rs` and
//! `tests/metrics.rs` already use it — not a new integration, but not a
//! stand-in either.
//!
//! One real caller gap surfaces along the way: `Parsed::document_class`
//! only ever carries the bare class name (`document_class` in
//! `crates/compiler/src/parser.rs` discards `\documentclass[options]`'s
//! bracket argument), so no real caller in this repo can yet supply
//! anything but `article.cls`'s own default paper/size to
//! [`Stylesheet::article`]. [`sheet`] below uses that default rather than
//! inventing a caller-chosen one.

use std::path::Path;

use flashtex_document_style::{BaseSize, ClassOptions, Paper, Pt, SizeName, Stylesheet, font_size};
use flashtex_font_engine::{Face, TrueTypeFace, load_from_path};
use flashtex_title_layout::{
    DateField, DocumentClass, GlyphMetrics, TitleBlockInput, TitleLayoutError, layout_title_block,
    layout_title_block_with_metrics,
};

/// Same file, same path, as `flashtex-font-engine/tests/latin_modern.rs`.
const LM_ROMAN10: &str =
    "/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm/lmroman10-regular.otf";

fn corpus_source(case: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tex-corpus/cases")
        .join(case)
        .join("main.tex");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading corpus case {case} at {}: {e}", path.display()))
}

/// Loads the real face, or skips (matching
/// `flashtex-font-engine/tests/latin_modern.rs`'s own convention) when this
/// machine has no BasicTeX install at that path.
fn lm_roman_10() -> Option<TrueTypeFace> {
    if !Path::new(LM_ROMAN10).is_file() {
        eprintln!("SKIP: {LM_ROMAN10} not present on this machine");
        return None;
    }
    Some(load_from_path(Path::new(LM_ROMAN10)).unwrap_or_else(|e| panic!("{LM_ROMAN10}: {e}")))
}

/// [`GlyphMetrics`] over a real `flashtex_font_engine::Face`. See the module
/// docs above for how this mirrors `flashtex-font-engine`'s own
/// `adapters::paragraph::FaceMetrics`.
struct RealFaceMetrics<'a> {
    face: &'a dyn Face,
}

impl GlyphMetrics for RealFaceMetrics<'_> {
    fn advance_width(&self, ch: char, size: Pt) -> Option<Pt> {
        // No cmap entry: a real gap in this real font, reported as `None`
        // exactly like every other caller-side "I don't know" — never
        // fabricated (see `missing_real_glyph_is_rejected_not_fabricated`).
        let gid = self.face.glyph_id(ch)?;
        let units = self.face.advance(gid).ok()?;
        Some(Pt(self.face.to_points(i64::from(units), size.0)))
    }

    fn em(&self, size: Pt) -> Option<Pt> {
        // OpenType carries no TFM `\fontdimen6`; the face's own em square
        // scaled to `size` stands in for TeX's quad. This is a labeled
        // approximation, not a fabricated constant: it comes from this
        // face's real `units_per_em`, not a hand-picked number, and this
        // fixture never needs it (all its author groups have one author,
        // so `layout_title_block_with_metrics` never calls `em`).
        Some(Pt(self
            .face
            .to_points(i64::from(self.face.units_per_em()), size.0)))
    }
}

/// `article.cls`'s own default paper/size — see the module docs' note on
/// why no real caller in this repo can supply anything else yet.
fn sheet() -> Stylesheet {
    Stylesheet::article(ClassOptions {
        paper: Paper::Letter,
        size: BaseSize::Pt10,
    })
}

#[test]
fn document_class_from_real_compiler_parser_selects_article() {
    let source = corpus_source("plain-paragraphs");
    let parsed = flashtex_compiler::parser::parse(&source);
    assert_eq!(parsed.document_class.as_deref(), Some("article"));

    let class = DocumentClass::parse(parsed.document_class.as_deref().unwrap())
        .expect("the compiler's own class name must parse");
    assert_eq!(class, DocumentClass::Article);
    assert!(class.is_supported());
}

#[test]
fn document_class_from_real_compiler_parser_rejects_report() {
    // No corpus case declares anything but `article` (14/14 — see the rev-4
    // gap measurement in coordination/daniel-title.md), so this exercises
    // the real parser on a minimal document shaped like one instead of a
    // corpus fixture.
    let parsed = flashtex_compiler::parser::parse(
        "\\documentclass{report}\n\\begin{document}Body.\\end{document}",
    );
    assert_eq!(parsed.document_class.as_deref(), Some("report"));

    let class = DocumentClass::parse(parsed.document_class.as_deref().unwrap())
        .expect("\"report\" is a recognized, if unsupported, class name");
    assert!(!class.is_supported());

    let input = TitleBlockInput {
        title_lines: vec!["T".to_string()],
        author_lines: vec![vec!["A".to_string()]],
        date: DateField::Suppressed,
    };
    assert_eq!(
        layout_title_block(class, &sheet(), &input),
        Err(TitleLayoutError::UnsupportedDocumentClass(class))
    );
}

#[test]
fn layout_with_real_font_engine_metrics_matches_hand_computed_width() {
    let Some(face) = lm_roman_10() else { return };
    let metrics = RealFaceMetrics { face: &face };

    let input = TitleBlockInput {
        title_lines: vec!["Hello".to_string()],
        author_lines: vec![vec!["A. Author".to_string()], vec!["Café".to_string()]],
        date: DateField::Text("2026".to_string()),
    };

    let measured =
        layout_title_block_with_metrics(DocumentClass::Article, &sheet(), &input, &metrics)
            .expect("real Latin Modern Roman metrics fit a Letter page at 10pt");

    assert_eq!(measured.layout.rows.len(), 4); // title, 2 authors, date
    assert!(measured.layout.total_height.0.is_finite());
    assert!(measured.layout.total_height.0 > 0.0);

    // Cross-check against an independent measurement of the same real face:
    // sum "Hello"'s own advances directly through `Face`, at the exact size
    // `\LARGE` resolves to for this stylesheet (17.28pt at 10pt base — see
    // `flashtex_document_style::fonts`'s size table), not a width this
    // fixture invented, and not by calling back into `RealFaceMetrics` or
    // this crate's own arithmetic.
    let title_size = font_size(sheet().base_size(), SizeName::LARGE3).size;
    let expected_title_width: f64 = "Hello"
        .chars()
        .map(|ch| {
            let gid = face.glyph_id(ch).expect("Hello is plain ASCII");
            let units = face.advance(gid).expect("advance for a resolved glyph id");
            face.to_points(i64::from(units), title_size.0)
        })
        .sum();
    let title_extent = &measured.extents[0];
    assert!(
        (title_extent.width.0 - expected_title_width).abs() < 1e-9,
        "title width {} from the crate, expected {expected_title_width} from an independent \
         measurement of the same real face",
        title_extent.width.0
    );
    // Centered on the page: x = (usable text width - line width) / 2.
    let text_width = sheet().page_layout().text_area.width;
    assert!((title_extent.x.0 - (text_width.0 - expected_title_width) / 2.0).abs() < 1e-9);
}

#[test]
fn missing_real_glyph_is_rejected_not_fabricated() {
    let Some(face) = lm_roman_10() else { return };
    let metrics = RealFaceMetrics { face: &face };

    // U+5C3E, from the project's own
    // tests/tex-corpus/cases/literal-source-map/main.tex fixture: a real
    // character this real font's cmap has no glyph for (Latin Modern is a
    // Latin/Cyrillic/Greek text family; it does not cover CJK).
    assert!(
        face.glyph_id('尾').is_none(),
        "fixture assumption: Latin Modern Roman has no CJK glyphs"
    );

    let input = TitleBlockInput {
        title_lines: vec!["尾".to_string()],
        author_lines: vec![vec!["A".to_string()]],
        date: DateField::Suppressed,
    };
    let err = layout_title_block_with_metrics(DocumentClass::Article, &sheet(), &input, &metrics)
        .expect_err("a real font's real missing glyph must reject, not substitute a width");
    match err {
        TitleLayoutError::MissingGlyphMetric { ch, .. } => assert_eq!(ch, '尾'),
        other => panic!("expected MissingGlyphMetric, got {other:?}"),
    }
}
