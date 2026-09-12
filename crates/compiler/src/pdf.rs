//! PDF output for the positioned display list produced by the layout engine.
//!
//! Uses `pdf-writer` (pure Rust, no TeX engine involvement) to serialise
//! layout::Page/TextItem structs into a valid PDF byte stream.
//!
//! HONEST LIMITATIONS:
//! - Font: Helvetica (PDF base-14). Characters outside Windows-1252 are
//!   replaced with '?'. Proper Unicode typesetting requires font embedding.
//! - Glyph metrics: inherited from layout.rs placeholder ratios, so word
//!   positions may not match a reference engine.
//! - Images, math glyphs, and colour are outstanding requirements.

use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};

use crate::layout::Page;

/// Render a sequence of pages to PDF bytes.
///
/// Returns an empty Vec if `pages` is empty; callers should not write
/// that as a pdf_path.
pub fn render(pages: &[Page]) -> Vec<u8> {
    if pages.is_empty() {
        return Vec::new();
    }

    // ---- Reference allocation ----
    // IDs: 1=catalog, 2=pages, 3..2+n=page dicts, 3+n..2+2n=content streams,
    //      3+2n=Helvetica font dict.
    let n = pages.len() as i32;
    let catalog_id = Ref::new(1);
    let pages_id   = Ref::new(2);
    let page_ids: Vec<Ref>    = (0..n).map(|i| Ref::new(3 + i)).collect();
    let content_ids: Vec<Ref> = (0..n).map(|i| Ref::new(3 + n + i)).collect();
    let font_id = Ref::new(3 + 2 * n);

    let mut pdf = Pdf::new();

    // ---- Catalog ----
    pdf.catalog(catalog_id).pages(pages_id);

    // ---- Pages tree ----
    {
        let mut pd = pdf.pages(pages_id);
        pd.kids(page_ids.iter().copied());
        pd.count(n);
    }

    // ---- Helvetica (base-14, no embedding) ----
    pdf.type1_font(font_id).base_font(Name(b"Helvetica"));

    // ---- Per-page output ----
    for (i, page) in pages.iter().enumerate() {
        let w = page.width_pt as f32;
        let h = page.height_pt as f32;

        // Content stream
        let stream_bytes = page_content(page);
        pdf.stream(content_ids[i], &stream_bytes);

        // Page dictionary
        let mut pg = pdf.page(page_ids[i]);
        pg.parent(pages_id);
        pg.media_box(Rect::new(0.0, 0.0, w, h));
        pg.contents(content_ids[i]);
        {
            let mut res = pg.resources();
            res.fonts().pair(Name(b"F1"), font_id);
        }
    }

    pdf.finish()
}

/// Build the content stream for one page.
fn page_content(page: &Page) -> Vec<u8> {
    let h = page.height_pt as f32;
    let mut c = Content::new();

    if page.items.is_empty() {
        return c.finish().to_vec();
    }

    c.begin_text();

    let mut cur_size: f32 = -1.0;
    for item in &page.items {
        let size = item.font_size_pt as f32;
        if (size - cur_size).abs() > 0.01 {
            c.set_font(Name(b"F1"), size);
            cur_size = size;
        }
        // PDF origin is bottom-left; layout origin is top-left.
        let pdf_y = h - item.baseline_y_pt as f32;
        // Use absolute text matrix (Tm) so each word is placed exactly.
        c.set_text_matrix([1.0, 0.0, 0.0, 1.0, item.x_pt as f32, pdf_y]);
        // Encode text: Latin-1 pass-through; non-encodable chars become '?'.
        let encoded = latin1_encode(&item.text);
        c.show(Str(&encoded));
    }

    c.end_text();
    c.finish().to_vec()
}

/// Encode a &str into Windows-1252 bytes as best we can.
/// Characters outside the range are replaced with b'?'.
fn latin1_encode(text: &str) -> Vec<u8> {
    text.chars()
        .map(|ch| {
            let u = ch as u32;
            if u < 0x80 || (0xa0..=0xff).contains(&u) {
                u as u8
            } else {
                // Windows-1252 supplementary range (0x80–0x9F).
                // Map the commonly used ones; fall back to '?'.
                match ch {
                    '\u{20ac}' => 0x80, // €
                    '\u{201a}' => 0x82, // ‚
                    '\u{0192}' => 0x83, // ƒ
                    '\u{201e}' => 0x84, // „
                    '\u{2026}' => 0x85, // …
                    '\u{2020}' => 0x86, // †
                    '\u{2021}' => 0x87, // ‡
                    '\u{2022}' => 0x95, // •
                    '\u{2013}' => 0x96, // –
                    '\u{2014}' => 0x97, // —
                    '\u{2018}' => 0x91, // '
                    '\u{2019}' => 0x92, // '
                    '\u{201c}' => 0x93, // "
                    '\u{201d}' => 0x94, // "
                    _ => b'?',
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Page, TextItem};
    use crate::Span;

    fn dummy_span() -> Span { Span { start: 0, end: 1 } }

    fn one_word_page() -> Page {
        Page {
            number: 1,
            width_pt: 612.0,
            height_pt: 792.0,
            items: vec![TextItem {
                text: "Hello".into(),
                x_pt: 72.0,
                baseline_y_pt: 84.0,
                font_size_pt: 12.0,
                span: dummy_span(),
            }],
        }
    }

    #[test]
    fn render_empty_returns_empty() {
        assert!(render(&[]).is_empty());
    }

    #[test]
    fn render_produces_pdf_header() {
        let bytes = render(&[one_word_page()]);
        assert!(bytes.starts_with(b"%PDF-"), "should start with %PDF-");
    }

    #[test]
    fn render_contains_word() {
        let bytes = render(&[one_word_page()]);
        // "Hello" is in the content stream; search raw bytes.
        assert!(bytes.windows(5).any(|w| w == b"Hello"));
    }

    #[test]
    fn render_two_pages() {
        let p1 = one_word_page();
        let mut p2 = one_word_page();
        p2.number = 2;
        let bytes = render(&[p1, p2]);
        assert!(bytes.starts_with(b"%PDF-"));
        // Should have two Page objects in the output.
        let count = bytes.windows(5).filter(|w| *w == b"/Type").count();
        assert!(count >= 2, "expected multiple /Type entries in PDF");
    }

    #[test]
    fn latin1_encode_ascii() {
        assert_eq!(latin1_encode("abc"), b"abc");
    }

    #[test]
    fn latin1_encode_fallback() {
        // CJK char → '?'
        assert_eq!(latin1_encode("日"), vec![b'?']);
    }
}
