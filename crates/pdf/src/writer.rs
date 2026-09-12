//! Hand-written PDF 1.4 serialisation.
//!
//! Object numbering is fixed so the file is reproducible byte-for-byte:
//!
//! | object | content                                   |
//! |--------|-------------------------------------------|
//! | 1      | catalog                                   |
//! | 2      | page tree                                 |
//! | 3      | font resource /F1 (Times-Roman, WinAnsi)  |
//! | 4      | font resource /F2 (Symbol, built-in)      |
//! | 5      | document information                      |
//! | 6+2i   | page `i` (zero-based)                     |
//! | 7+2i   | content stream of page `i`                |
//!
//! When a font is embedded (opt-in, see `crate::embed`), five more objects
//! follow the last page, starting at `6 + 2 × pages`: the Type0 font `/F3`,
//! its CIDFontType2 (TrueType subset) or CIDFontType0 (whole CFF)
//! descendant, the font descriptor, the `/FontFile2` or `/FontFile3` stream,
//! and the ToUnicode CMap. Page numbering is unchanged either way.
//!
//! Every text item is emitted as its own `BT … ET` block with an absolute `Td`,
//! so the content stream is trivially checkable: each `Td` carries exactly the
//! item's PDF-space coordinates `(x_pt, height_pt - baseline_y_pt)`. Inside the
//! block the item's font runs are written as alternating `Tf`/`Tj` pairs; the
//! viewer advances between runs using the real base-14 widths, so this crate
//! carries no width tables of its own.
//!
//! Fraction rules: the FT-002 compiler (`crates/compiler/src/math.rs`) has no
//! rule primitive in runtime-v1, so it emits a fraction bar as a text item made
//! only of U+2500 BOX DRAWINGS LIGHT HORIZONTAL, N characters wide at 0.5 em
//! each, with the item's baseline at the bottom edge of the bar. Such items are
//! drawn here as filled rectangles (`re f`), never as glyphs.

use crate::embed::EmbeddedSubset;
use crate::encoding;
use crate::{CompileResult, PdfError, PdfOutput, RenderOptions};
use std::io::Write;

pub const PDF_HEADER: &[u8] = b"%PDF-1.4\n";
pub const FIRST_PAGE_OBJECT: usize = 6;
/// Width the compiler assumes for one U+2500 in a rule item, in em.
pub const RULE_DASH_EM: f64 = 0.5;
/// Fraction rule thickness in em of the *rule item's* font size: the compiler
/// uses 0.06 em of the parent size and sets the rule item at 0.7 of that size.
pub const RULE_THICKNESS_EM: f64 = 0.06 / 0.7;
pub const PRODUCER: &str = "FlashTeX flashtex-pdf 0.1.0";

/// Object number of the Type0 font when one is embedded.
pub fn embedded_font_object(page_count: usize) -> usize {
    FIRST_PAGE_OBJECT + 2 * page_count
}

pub fn render(result: &CompileResult, options: &RenderOptions) -> Result<PdfOutput, PdfError> {
    if result.pages.is_empty() {
        return Err(PdfError::Invalid("a PDF needs at least one page".into()));
    }
    for page in &result.pages {
        for (name, v) in [("width_pt", page.width_pt), ("height_pt", page.height_pt)] {
            if !(v.is_finite() && v > 0.0) {
                return Err(PdfError::Invalid(format!(
                    "page {}: {name} must be a positive finite number, got {v}",
                    page.number
                )));
            }
        }
        for (i, item) in page.items.iter().enumerate() {
            for (name, v) in [
                ("x_pt", item.x_pt),
                ("baseline_y_pt", item.baseline_y_pt),
                ("font_size_pt", item.font_size_pt),
            ] {
                if !v.is_finite() {
                    return Err(PdfError::Invalid(format!(
                        "page {}: item {i}: {name} must be finite, got {v}",
                        page.number
                    )));
                }
            }
            if item.font_size_pt <= 0.0 {
                return Err(PdfError::Invalid(format!(
                    "page {}: item {i}: font_size_pt must be positive, got {}",
                    page.number, item.font_size_pt
                )));
            }
        }
    }

    let mut warnings = Vec::new();
    let page_count = result.pages.len();

    let face = match (options.face, &options.embed_font) {
        (encoding::Face::Embedded, None) => {
            warnings.push(
                "document face 'embedded' requested but no font is embedded; using Times".into(),
            );
            encoding::Face::Times
        }
        (face, _) => face,
    };

    // Decide the embedded subset up front: it needs every character on every
    // page, and the pages need its glyph ids. As the document face it gets
    // every character; as a gap-filler only those outside WinAnsi and Symbol.
    let embedded: Option<EmbeddedSubset> = match &options.embed_font {
        None => None,
        Some(font) => {
            let wanted: std::collections::BTreeSet<char> = result
                .pages
                .iter()
                .flat_map(|p| p.items.iter())
                .filter(|item| !is_rule_item(&item.text))
                .flat_map(|item| item.text.chars())
                .filter(|&c| face == encoding::Face::Embedded || encoding::needs_embedding(c))
                .collect();
            let subset = font.subset_for(wanted.iter().copied()).map_err(|e| {
                PdfError::Invalid(format!("embedding {}: {e}", font.source.display()))
            })?;
            // Only characters that will end up as '?' are worth a font-level
            // warning; with the embedded face, Symbol and Times still cover
            // what it lacks silently.
            let missing: Vec<String> = wanted
                .iter()
                .filter(|c| !subset.chars.contains_key(c) && encoding::needs_embedding(**c))
                .map(|c| format!("{c:?} (U+{:04X})", *c as u32))
                .collect();
            if !missing.is_empty() {
                warnings.push(format!(
                    "embedded font {} ({}) has no glyph for {}; those characters fall back to '?'",
                    font.font.postscript_name,
                    font.source.display(),
                    missing.join(", ")
                ));
            }
            Some(subset)
        }
    };
    let lookup = embedded
        .as_ref()
        .map(|e| move |c: char| e.chars.get(&c).copied());
    let lookup_ref: Option<&dyn Fn(char) -> Option<u16>> = match &lookup {
        Some(f) => Some(f),
        None => None,
    };

    let mut doc = Document::new();

    doc.object(1, b"<< /Type /Catalog /Pages 2 0 R >>");

    let mut kids = String::new();
    for i in 0..page_count {
        kids.push_str(&format!("{} 0 R ", FIRST_PAGE_OBJECT + 2 * i));
    }
    doc.object(
        2,
        format!("<< /Type /Pages /Kids [ {kids}] /Count {page_count} >>").as_bytes(),
    );

    doc.object(
        3,
        format!(
            "<< /Type /Font /Subtype /Type1 /BaseFont /{} /Encoding /WinAnsiEncoding >>",
            encoding::Font::Times.base_font()
        )
        .as_bytes(),
    );
    // Symbol uses its built-in encoding; naming one would remap its glyphs.
    doc.object(
        4,
        format!(
            "<< /Type /Font /Subtype /Type1 /BaseFont /{} >>",
            encoding::Font::Symbol.base_font()
        )
        .as_bytes(),
    );

    doc.object(
        5,
        format!(
            "<< /Producer ({}) /Creator (FlashTeX) >>",
            escape_ascii(PRODUCER)
        )
        .as_bytes(),
    );

    let font_obj = embedded_font_object(page_count);
    let mut fonts = format!(
        "/{} 3 0 R /{} 4 0 R",
        encoding::Font::Times.resource_name(),
        encoding::Font::Symbol.resource_name()
    );
    if embedded.is_some() {
        fonts.push_str(&format!(
            " /{} {font_obj} 0 R",
            encoding::Font::Embedded.resource_name()
        ));
    }

    for (i, page) in result.pages.iter().enumerate() {
        let page_obj = FIRST_PAGE_OBJECT + 2 * i;
        let content_obj = page_obj + 1;
        let content = page_content(page, lookup_ref, embedded.as_ref(), face, &mut warnings);

        doc.object(
            page_obj,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [ 0 0 {} {} ] /Resources << /Font << {fonts} >> >> /Contents {content_obj} 0 R >>",
                num(page.width_pt),
                num(page.height_pt),
            )
            .as_bytes(),
        );
        doc.stream(content_obj, &content);
    }

    if let Some(e) = &embedded {
        let cid_obj = font_obj + 1;
        let desc_obj = font_obj + 2;
        let file_obj = font_obj + 3;
        let tounicode_obj = font_obj + 4;
        doc.object(
            font_obj,
            format!(
                "<< /Type /Font /Subtype /Type0 /BaseFont /{} /Encoding /Identity-H /DescendantFonts [ {cid_obj} 0 R ] /ToUnicode {tounicode_obj} 0 R >>",
                e.base_font
            )
            .as_bytes(),
        );
        // TrueType subsets go through CIDFontType2 with an identity GID map;
        // a whole CFF goes through CIDFontType0, where a non-CID-keyed CFF
        // program already selects glyphs by CID = GID.
        let (cid_subtype, gid_map, file_key, stream_extra) = match &e.program {
            crate::embed::Program::TrueType(subset) => (
                "CIDFontType2",
                " /CIDToGIDMap /Identity",
                "FontFile2",
                format!("/Length1 {}", subset.bytes.len()),
            ),
            crate::embed::Program::Cff { .. } => (
                "CIDFontType0",
                "",
                "FontFile3",
                "/Subtype /CIDFontType0C".to_string(),
            ),
        };
        doc.object(
            cid_obj,
            format!(
                "<< /Type /Font /Subtype /{cid_subtype} /BaseFont /{} /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor {desc_obj} 0 R /DW 1000 /W {}{gid_map} >>",
                e.base_font,
                e.widths_array()
            )
            .as_bytes(),
        );
        let d = &e.descriptor;
        // Flags 4 = Symbolic: the font is used through glyph ids, not a
        // standard Latin encoding. StemV is a nominal value; these fonts
        // carry no stem width and viewers do not rely on it for rendering.
        doc.object(
            desc_obj,
            format!(
                "<< /Type /FontDescriptor /FontName /{} /Flags 4 /FontBBox [ {} {} {} {} ] /ItalicAngle {} /Ascent {} /Descent {} /CapHeight {} /StemV 80 /{file_key} {file_obj} 0 R >>",
                e.base_font,
                d.bbox[0],
                d.bbox[1],
                d.bbox[2],
                d.bbox[3],
                num(d.italic_angle),
                d.ascent,
                d.descent,
                d.cap_height
            )
            .as_bytes(),
        );
        doc.stream_with(file_obj, &stream_extra, e.program_bytes());
        doc.stream(tounicode_obj, &e.to_unicode_cmap());
    }

    Ok(PdfOutput {
        bytes: doc.finish(),
        warnings,
    })
}

/// True when the compiler meant this item as a fraction rule, not text.
pub fn is_rule_item(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c == '\u{2500}')
}

/// Builds one page's content stream. The page is left untouched (white) apart
/// from black text and rules; there is deliberately no background fill and no
/// theme input.
fn page_content(
    page: &crate::Page,
    lookup: Option<&dyn Fn(char) -> Option<u16>>,
    embedded: Option<&EmbeddedSubset>,
    face: encoding::Face,
    warnings: &mut Vec<String>,
) -> Vec<u8> {
    let mut out = Vec::new();
    // Non-stroking colour: black in DeviceGray. Set explicitly so output never
    // depends on viewer defaults.
    out.extend_from_slice(b"0 g\n");
    for (i, item) in page.items.iter().enumerate() {
        if is_rule_item(&item.text) {
            // The bar occupies [baseline - thickness, baseline] in top-left
            // space; in PDF space its bottom edge is at height - baseline.
            let dashes = item.text.chars().count() as f64;
            let width = dashes * RULE_DASH_EM * item.font_size_pt;
            let thickness = RULE_THICKNESS_EM * item.font_size_pt;
            let x = item.x_pt;
            let y = page.height_pt - item.baseline_y_pt;
            writeln!(
                out,
                "{} {} {} {} re f",
                num(x),
                num(y),
                num(width),
                num(thickness)
            )
            .expect("writing to Vec cannot fail");
            continue;
        }
        let encoded = encoding::encode_face(&item.text, lookup, face);
        if !encoded.unrepresentable.is_empty() {
            let listed: Vec<String> = encoded
                .unrepresentable
                .iter()
                .map(|c| format!("{c:?} (U+{:04X})", *c as u32))
                .collect();
            let fonts = match embedded {
                Some(e) => format!("WinAnsiEncoding, Symbol, or embedded {}", e.base_font),
                None => "WinAnsiEncoding or Symbol".to_string(),
            };
            warnings.push(format!(
                "page {}: item {i} {:?}: {} not representable in {fonts}; written as '{}'",
                page.number,
                item.text,
                listed.join(", "),
                encoding::SUBSTITUTE as char
            ));
        }
        let x = item.x_pt;
        let y = page.height_pt - item.baseline_y_pt;
        writeln!(out, "BT\n{} {} Td", num(x), num(y)).expect("writing to Vec cannot fail");
        for run in &encoded.runs {
            writeln!(
                out,
                "/{} {} Tf",
                run.font.resource_name(),
                num(item.font_size_pt)
            )
            .expect("writing to Vec cannot fail");
            if run.font == encoding::Font::Embedded {
                // Identity-H: two bytes per glyph, written as a hex string.
                out.push(b'<');
                for b in &run.bytes {
                    write!(out, "{b:02X}").expect("writing to Vec cannot fail");
                }
                out.extend_from_slice(b"> Tj\n");
            } else {
                out.push(b'(');
                out.extend_from_slice(&escape_string(&run.bytes));
                out.extend_from_slice(b") Tj\n");
            }
        }
        out.extend_from_slice(b"ET\n");
    }
    out
}

/// Escapes the three bytes that would otherwise terminate or confuse a literal
/// string. Other bytes, including WinAnsi high bytes, are written verbatim;
/// PDF literal strings are byte strings and the stream is binary anyway.
pub fn escape_string(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len() + 4);
    for &b in bytes {
        match b {
            b'(' | b')' | b'\\' => {
                out.push(b'\\');
                out.push(b);
            }
            b'\r' => out.extend_from_slice(b"\\r"),
            b'\n' => out.extend_from_slice(b"\\n"),
            _ => out.push(b),
        }
    }
    out
}

fn escape_ascii(s: &str) -> String {
    String::from_utf8(escape_string(s.as_bytes())).expect("ASCII stays ASCII")
}

/// Formats a real number the way PDF requires: plain decimal, no exponent,
/// no trailing zeros, at most three decimals (a thousandth of a point).
pub fn num(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".to_string()
    } else {
        s.to_string()
    }
}

struct Document {
    bytes: Vec<u8>,
    /// Byte offset of each object, indexed by object number (index 0 unused).
    offsets: Vec<usize>,
}

impl Document {
    fn new() -> Self {
        Document {
            bytes: PDF_HEADER.to_vec(),
            offsets: vec![0],
        }
    }

    fn begin(&mut self, number: usize) {
        assert_eq!(
            number,
            self.offsets.len(),
            "objects must be written in order"
        );
        self.offsets.push(self.bytes.len());
        writeln!(self.bytes, "{number} 0 obj").expect("Vec write");
    }

    fn object(&mut self, number: usize, body: &[u8]) {
        self.begin(number);
        self.bytes.extend_from_slice(body);
        self.bytes.extend_from_slice(b"\nendobj\n");
    }

    fn stream(&mut self, number: usize, data: &[u8]) {
        self.stream_with(number, "", data);
    }

    /// A stream whose dictionary carries extra entries (e.g. `/Length1`).
    fn stream_with(&mut self, number: usize, extra: &str, data: &[u8]) {
        self.begin(number);
        write!(
            self.bytes,
            "<< /Length {}{}{} >>\nstream\n",
            data.len(),
            if extra.is_empty() { "" } else { " " },
            extra
        )
        .expect("Vec write");
        self.bytes.extend_from_slice(data);
        self.bytes.extend_from_slice(b"\nendstream\nendobj\n");
    }

    fn finish(mut self) -> Vec<u8> {
        let xref_offset = self.bytes.len();
        let size = self.offsets.len();
        write!(self.bytes, "xref\n0 {size}\n0000000000 65535 f \n").expect("Vec write");
        for &offset in &self.offsets[1..] {
            writeln!(self.bytes, "{offset:010} 00000 n ").expect("Vec write");
        }
        write!(
            self.bytes,
            "trailer\n<< /Size {size} /Root 1 0 R /Info 5 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n"
        )
        .expect("Vec write");
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_plain_decimals() {
        assert_eq!(num(612.0), "612");
        assert_eq!(num(708.5), "708.5");
        assert_eq!(num(0.1 + 0.2), "0.3");
        assert_eq!(num(-0.0001), "0");
        assert_eq!(num(1e-7), "0");
        assert_eq!(num(1234567.891), "1234567.891");
    }

    #[test]
    fn escapes_delimiters() {
        assert_eq!(escape_string(b"a(b)c\\d"), b"a\\(b\\)c\\\\d");
    }
}
