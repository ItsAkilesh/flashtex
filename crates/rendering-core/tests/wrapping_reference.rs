//! Actual fixture: reference character codes remain distinct from original GIDs.
use flashtex_pdf::{compare::font_from_dict, exact::ExactFont, reader::PdfFile};
use flashtex_rendering_core::digest;
use serde_json::Value;
#[test]
fn wrapping_reference_widths_and_producer_identity_are_explicit() {
    let reference = include_bytes!("fixtures/wrapping-reference/reference.pdf");
    let engine: Value = serde_json::from_slice(include_bytes!(
        "fixtures/wrapping-reference/reference-engine.json"
    ))
    .unwrap();
    assert_eq!(engine["pdf_sha256"], digest(reference));
    let file = PdfFile::parse(reference).unwrap();
    let pages = file.pages().unwrap();
    let fonts = file.page_fonts(pages[0]);
    let ExactFont::Simple(font) = font_from_dict(&file, fonts["F43"]).unwrap() else {
        panic!("pinned reference font profile changed")
    };
    for (code, expected) in [(b'T', "707.2"), (b'h', "544"), (b'e', "435.2")] {
        assert_eq!(
            font.widths[usize::from(code - font.first_char)].as_str(),
            expected
        );
    }
    let display: Value =
        serde_json::from_slice(include_bytes!("fixtures/wrapping-reference/display.json")).unwrap();
    assert_eq!(display["payload"]["diagnostics"], serde_json::json!([]));
    let first = &display["payload"]["pages"][0]["items"][0];
    assert_eq!(first["text"], "The");
    assert_eq!(first["glyphs"][1]["gid"], 63);
    assert_eq!(first["glyphs"][1]["origin_x"], 84362432);
    // The original PDF uses this exact tick position, not reference character code104.
    let original =
        PdfFile::parse(include_bytes!("fixtures/wrapping-reference/original.pdf")).unwrap();
    let pages = original.pages().unwrap();
    let content = original.page_content(pages[0]).unwrap();
    let ops = flashtex_pdf::exact::parse(&content).unwrap();
    assert!(ops.iter().any(|op| matches!(op, flashtex_pdf::exact::Op::TextMatrix(values) if values[4].as_str()=="80.45428466796875")));
    assert_eq!(
        include_bytes!("fixtures/wrapping-reference/original.txt"),
        include_bytes!("fixtures/wrapping-reference/reference.txt")
    );
}
