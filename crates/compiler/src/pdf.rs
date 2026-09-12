//! Minimal PDF 1.7 serialization for laid-out FlashTeX pages.
//!
//! The serializer uses the PDF standard Times-Roman and Times-Bold fonts and
//! emits only ASCII text. A non-ASCII Unicode scalar is replaced by `?` because
//! the base-14 fonts plus a simple single-byte encoding cannot represent general
//! Unicode reliably; silently writing UTF-8 bytes into a PDF string would create
//! corrupt or misleading output.

use crate::layout::{Page, BODY_SIZE_PT};

const CATALOG_ID: usize = 1;
const PAGES_ID: usize = 2;
const TIMES_ROMAN_ID: usize = 3;
const TIMES_BOLD_ID: usize = 4;
const FIRST_PAGE_ID: usize = 5;

/// Render positioned pages as a complete PDF 1.7 byte stream.
pub fn render(pages: &[Page]) -> Vec<u8> {
    let object_count = TIMES_BOLD_ID + pages.len() * 2;
    let mut objects = vec![Vec::new(); object_count + 1];

    objects[CATALOG_ID] = b"<< /Type /Catalog /Pages 2 0 R >>".to_vec();

    let kids = pages
        .iter()
        .enumerate()
        .map(|(index, _)| format!("{} 0 R", page_id(index)))
        .collect::<Vec<_>>()
        .join(" ");
    objects[PAGES_ID] =
        format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids, pages.len()).into_bytes();

    objects[TIMES_ROMAN_ID] = font_object("Times-Roman");
    objects[TIMES_BOLD_ID] = font_object("Times-Bold");

    for (index, page) in pages.iter().enumerate() {
        let page_object = format!(
            "<< /Type /Page /Parent {} 0 R /MediaBox [0 0 {} {}] \
             /Resources << /Font << /F1 {} 0 R /F2 {} 0 R >> >> \
             /Contents {} 0 R >>",
            PAGES_ID,
            pdf_number(page.width_pt),
            pdf_number(page.height_pt),
            TIMES_ROMAN_ID,
            TIMES_BOLD_ID,
            content_id(index)
        );
        objects[page_id(index)] = page_object.into_bytes();

        let content = page_content(page);
        let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
        stream.extend_from_slice(&content);
        stream.extend_from_slice(b"endstream");
        objects[content_id(index)] = stream;
    }

    serialize_objects(&objects)
}

fn page_id(index: usize) -> usize {
    FIRST_PAGE_ID + index * 2
}

fn content_id(index: usize) -> usize {
    page_id(index) + 1
}

fn font_object(base_font: &str) -> Vec<u8> {
    format!(
        "<< /Type /Font /Subtype /Type1 /BaseFont /{} /Encoding /WinAnsiEncoding >>",
        base_font
    )
    .into_bytes()
}

fn page_content(page: &Page) -> Vec<u8> {
    let mut content = String::new();
    for item in &page.items {
        let font = if item.font_size_pt == BODY_SIZE_PT {
            "F1"
        } else {
            "F2"
        };
        // Layout coordinates are top-left; PDF user-space coordinates are
        // bottom-left. Both values describe the text baseline.
        let pdf_y = page.height_pt - item.baseline_y_pt;
        content.push_str(&format!(
            "BT /{} {} Tf {} {} Td ({}) Tj ET\n",
            font,
            pdf_number(item.font_size_pt),
            pdf_number(item.x_pt),
            pdf_number(pdf_y),
            pdf_string(&item.text)
        ));
    }
    content.into_bytes()
}

fn pdf_string(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '(' | ')' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{0008}' => escaped.push_str("\\b"),
            '\u{000c}' => escaped.push_str("\\f"),
            ' '..='~' => escaped.push(ch),
            _ => escaped.push('?'),
        }
    }
    escaped
}

fn pdf_number(value: f64) -> String {
    let mut number = format!("{value:.4}");
    while number.contains('.') && number.ends_with('0') {
        number.pop();
    }
    if number.ends_with('.') {
        number.pop();
    }
    if number == "-0" {
        number = "0".to_string();
    }
    number
}

fn serialize_objects(objects: &[Vec<u8>]) -> Vec<u8> {
    let mut output = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = vec![0usize; objects.len()];

    for (id, object) in objects.iter().enumerate().skip(1) {
        offsets[id] = output.len();
        output.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        output.extend_from_slice(object);
        output.extend_from_slice(b"\nendobj\n");
    }

    let xref_offset = output.len();
    output.extend_from_slice(format!("xref\n0 {}\n", objects.len()).as_bytes());
    output.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        output.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    output.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root {} 0 R >>\nstartxref\n{}\n%%EOF",
            objects.len(),
            CATALOG_ID,
            xref_offset
        )
        .as_bytes(),
    );
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::TextItem;
    use crate::Span;

    fn page(number: u32, text: &str) -> Page {
        Page {
            number,
            width_pt: 612.0,
            height_pt: 792.0,
            items: vec![TextItem {
                text: text.to_string(),
                x_pt: 72.0,
                baseline_y_pt: 84.0,
                font_size_pt: 12.0,
                span: Span::new(0, text.len()),
            }],
        }
    }

    #[test]
    fn output_has_pdf_markers_and_one_page_object_per_page() {
        let pdf = render(&[page(1, "one"), page(2, "two")]);
        assert!(pdf.starts_with(b"%PDF-1.7"));
        assert!(pdf.ends_with(b"%%EOF"));

        let text = String::from_utf8_lossy(&pdf);
        assert_eq!(text.matches("/Type /Page ").count(), 2);
        assert!(text.contains("/Count 2"));
    }

    #[test]
    fn top_left_baseline_is_converted_to_pdf_bottom_left_y() {
        let mut input = page(1, "origin");
        input.items[0].baseline_y_pt = 72.0;
        let pdf = String::from_utf8_lossy(&render(&[input])).into_owned();
        assert!(pdf.contains("72 720 Td (origin) Tj"));
    }

    #[test]
    fn literal_strings_are_escaped_and_non_ascii_is_replaced() {
        let pdf = String::from_utf8_lossy(&render(&[page(1, "a(b)\\cé")])).into_owned();
        assert!(pdf.contains(r"(a\(b\)\\c?) Tj"));
    }

    #[test]
    fn xref_offsets_point_to_their_object_headers() {
        let pdf = render(&[page(1, "one"), page(2, "two")]);
        let text = String::from_utf8_lossy(&pdf);
        let xref_start = text.find("xref\n").expect("xref table");
        let mut lines = text[xref_start..].lines();
        assert_eq!(lines.next(), Some("xref"));
        let range = lines.next().expect("xref range");
        let object_count: usize = range
            .split_whitespace()
            .nth(1)
            .expect("xref size")
            .parse()
            .expect("numeric xref size");
        assert_eq!(lines.next(), Some("0000000000 65535 f "));

        for id in 1..object_count {
            let entry = lines.next().expect("one xref entry per object");
            let offset: usize = entry[..10].parse().expect("numeric xref offset");
            let header = format!("{id} 0 obj\n");
            assert_eq!(
                pdf.get(offset..offset + header.len()),
                Some(header.as_bytes())
            );
        }
    }
}
