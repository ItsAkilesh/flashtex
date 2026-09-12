//! Reference comparison: re-emit a reference PDF through the exact route
//! and classify every difference between two PDFs honestly.
//!
//! [`reemit`] reads a PDF (typically pdfTeX output for a corpus fixture)
//! with `crate::reader` and rebuilds it as an [`ExactDocument`]: the page
//! sizes, the decoded content streams as [`Content::Verbatim`], and every
//! font resource with its program bytes, widths, encoding, descriptor and
//! ToUnicode carried over unchanged. Rendering that document through
//! [`render_exact`] therefore exercises the exact API on real producer
//! streams, and [`classify`] then reports what still differs between the
//! two files and why.
//!
//! Categories are deliberately coarse and named after their cause, so a
//! report can say "the content operators are identical, the font program
//! bytes are identical, and the rest is object layout, compression and
//! document identity" without anyone reading a byte dump.

use crate::exact::{
    self, CidFont, Content, Decimal, Encoding, ExactDocument, ExactFont, ExactPage, FontDescriptor,
    FontProgram, Op, SimpleFont,
};
use crate::reader::{Obj, PdfFile, render};
use crate::sha256;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// Why two PDFs differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    /// Page count or MediaBox differs.
    PageGeometry,
    /// Same operators and operands, different whitespace/formatting.
    ContentFormatting,
    /// Same operator sequence, at least one operand differs.
    ContentOperands,
    /// Different operator sequences.
    ContentOperators,
    /// A content stream uses operators outside the bounded set.
    ContentUnsupported,
    /// Embedded font program bytes differ (or one side embeds none).
    FontProgram,
    /// Font dictionary metadata differs: widths, encoding, descriptor, ToUnicode, name.
    FontMetadata,
    /// A font resource exists on one side only.
    FontResources,
    /// Object numbering/order, object streams, xref form, PDF version.
    ObjectLayout,
    /// Stream filters.
    Compression,
    /// `/ID`, dates, producer/creator, other Info entries.
    DocumentIdentity,
}

#[derive(Debug, Clone, Default)]
pub struct Report {
    pub categories: BTreeSet<Category>,
    pub lines: Vec<String>,
    /// Per page: true when the decoded content bytes are identical.
    pub content_byte_identical: Vec<bool>,
    /// Per page: true when the parsed operator sequences are identical.
    pub content_ops_identical: Vec<bool>,
    /// Per page and font resource name: true when program bytes are identical.
    pub font_program_identical: BTreeMap<String, bool>,
}

impl Report {
    fn note(&mut self, c: Category, line: String) {
        self.categories.insert(c);
        self.lines.push(format!("[{c:?}] {line}"));
    }

    fn same(&mut self, line: String) {
        self.lines.push(format!("[same] {line}"));
    }

    pub fn text(&self) -> String {
        let mut s = String::new();
        for l in &self.lines {
            let _ = writeln!(s, "{l}");
        }
        let cats: Vec<String> = self.categories.iter().map(|c| format!("{c:?}")).collect();
        let _ = writeln!(
            s,
            "categories: {}",
            if cats.is_empty() {
                "none (byte-identical apart from nothing)".to_string()
            } else {
                cats.join(", ")
            }
        );
        s
    }
}

fn dec(o: &Obj, what: &str) -> Result<Decimal, String> {
    let n = o
        .as_number()
        .ok_or_else(|| format!("{what}: expected a number, found {}", render(o)))?;
    Decimal::new(n).map_err(|e| format!("{what}: {e}"))
}

fn dec_array<const N: usize>(f: &PdfFile, o: &Obj, what: &str) -> Result<[Decimal; N], String> {
    let a = f
        .resolve(o)
        .as_array()
        .ok_or_else(|| format!("{what}: expected an array"))?;
    if a.len() != N {
        return Err(format!("{what}: expected {N} numbers, found {}", a.len()));
    }
    let mut out = Vec::with_capacity(N);
    for x in a {
        out.push(dec(f.resolve(x), what)?);
    }
    Ok(out.try_into().expect("length checked"))
}

fn font_program(f: &PdfFile, desc: &BTreeMap<String, Obj>) -> Result<Option<FontProgram>, String> {
    if let Some(ff) = f.get(desc, "FontFile") {
        let d = ff.as_dict().ok_or("FontFile is not a stream")?;
        let bytes = f.decode_stream(ff)?;
        let l = |k: &str| {
            f.get(d, k)
                .and_then(Obj::as_usize)
                .ok_or_else(|| format!("FontFile without /{k}"))
        };
        return Ok(Some(FontProgram::Type1 {
            length1: l("Length1")?,
            length2: l("Length2")?,
            length3: l("Length3")?,
            bytes,
        }));
    }
    if let Some(ff) = f.get(desc, "FontFile2") {
        return Ok(Some(FontProgram::TrueType(f.decode_stream(ff)?)));
    }
    if let Some(ff) = f.get(desc, "FontFile3") {
        let d = ff.as_dict().ok_or("FontFile3 is not a stream")?;
        match f.get(d, "Subtype").and_then(Obj::as_name) {
            Some("Type1C" | "CIDFontType0C") => {
                return Ok(Some(FontProgram::Cff(f.decode_stream(ff)?)));
            }
            other => return Err(format!("FontFile3 subtype {other:?} is not supported")),
        }
    }
    Ok(None)
}

fn descriptor(f: &PdfFile, d: &BTreeMap<String, Obj>) -> Result<FontDescriptor, String> {
    let num = |k: &str| -> Result<Decimal, String> {
        match f.get(d, k) {
            Some(o) => dec(o, k),
            None => Err(format!("FontDescriptor without /{k}")),
        }
    };
    let opt = |k: &str| -> Result<Option<Decimal>, String> {
        match f.get(d, k) {
            Some(o) => dec(o, k).map(Some),
            None => Ok(None),
        }
    };
    Ok(FontDescriptor {
        flags: f
            .get(d, "Flags")
            .and_then(Obj::as_number)
            .and_then(|n| n.parse().ok())
            .ok_or("FontDescriptor without integer /Flags")?,
        bbox: dec_array::<4>(
            f,
            d.get("FontBBox")
                .ok_or("FontDescriptor without /FontBBox")?,
            "FontBBox",
        )?,
        italic_angle: num("ItalicAngle")?,
        ascent: num("Ascent")?,
        descent: num("Descent")?,
        cap_height: opt("CapHeight")?.unwrap_or_else(|| Decimal::from_i64(0)),
        stem_v: num("StemV")?,
        x_height: opt("XHeight")?,
        char_set: match f.get(d, "CharSet") {
            Some(Obj::String(s)) => Some(String::from_utf8_lossy(s).into_owned()),
            _ => None,
        },
    })
}

fn encoding(f: &PdfFile, o: &Obj) -> Result<Encoding, String> {
    match f.resolve(o) {
        Obj::Name(n) => Ok(Encoding::Named(n.clone())),
        Obj::Dict(d) => {
            let base = f
                .get(d, "BaseEncoding")
                .and_then(Obj::as_name)
                .map(String::from);
            let mut differences = Vec::new();
            if let Some(arr) = f.get(d, "Differences").and_then(Obj::as_array) {
                let mut code: Option<u16> = None;
                for item in arr {
                    match f.resolve(item) {
                        Obj::Number(n) => code = Some(n.parse().map_err(|_| "Differences code")?),
                        Obj::Name(name) => {
                            let c = code.ok_or("Differences name before any code")?;
                            if c > 255 {
                                return Err("Differences code above 255".into());
                            }
                            differences.push((c as u8, name.clone()));
                            code = Some(c + 1);
                        }
                        other => return Err(format!("Differences element {}", render(other))),
                    }
                }
            }
            Ok(Encoding::Differences { base, differences })
        }
        other => Err(format!("unsupported /Encoding {}", render(other))),
    }
}

/// Converts a font dictionary into an [`ExactFont`], carrying every value
/// verbatim. Type1/TrueType simple fonts and Type0 fonts over
/// CIDFontType0C/TrueType programs are handled; anything else is an error
/// that names what was found.
pub fn font_from_dict(f: &PdfFile, d: &BTreeMap<String, Obj>) -> Result<ExactFont, String> {
    let subtype = f
        .get(d, "Subtype")
        .and_then(Obj::as_name)
        .ok_or("font without /Subtype")?
        .to_string();
    let base_font = f
        .get(d, "BaseFont")
        .and_then(Obj::as_name)
        .ok_or("font without /BaseFont")?
        .to_string();
    match subtype.as_str() {
        "Type1" | "TrueType" | "MMType1" => {
            let desc_dict = f.get(d, "FontDescriptor").and_then(Obj::as_dict);
            let (descriptor, program) = match desc_dict {
                Some(dd) => (Some(descriptor(f, dd)?), font_program(f, dd)?),
                None => (None, None),
            };
            let first_char = f.get(d, "FirstChar").and_then(Obj::as_usize);
            let widths_obj = f.get(d, "Widths").and_then(Obj::as_array);
            let (first_char, widths) = match (first_char, widths_obj) {
                (Some(fc), Some(w)) => {
                    if fc > 255 {
                        return Err("FirstChar above 255".into());
                    }
                    let mut ws = Vec::with_capacity(w.len());
                    for x in w {
                        ws.push(dec(f.resolve(x), "Widths")?);
                    }
                    (fc as u8, ws)
                }
                // Standard 14 without /Widths: no width array is written and
                // any code is accepted.
                _ if program.is_none() => (0u8, Vec::new()),
                _ => return Err("embedded simple font without /FirstChar and /Widths".into()),
            };
            let enc = match d.get("Encoding") {
                Some(e) => Some(encoding(f, e)?),
                None => None,
            };
            let to_unicode = match f.get(d, "ToUnicode") {
                Some(tu) => Some(f.decode_stream(tu)?),
                None => None,
            };
            Ok(ExactFont::Simple(SimpleFont {
                subtype: if subtype == "MMType1" {
                    "Type1".into()
                } else {
                    subtype
                },
                base_font,
                program,
                first_char,
                widths,
                encoding: enc,
                descriptor,
                to_unicode,
            }))
        }
        "Type0" => {
            let enc = f.get(d, "Encoding").and_then(Obj::as_name).unwrap_or("");
            if enc != "Identity-H" {
                return Err(format!("Type0 encoding {enc:?} is not Identity-H"));
            }
            let desc_fonts = f
                .get(d, "DescendantFonts")
                .and_then(Obj::as_array)
                .ok_or("Type0 without DescendantFonts")?;
            let cid = desc_fonts
                .first()
                .and_then(|x| f.resolve(x).as_dict())
                .ok_or("empty DescendantFonts")?;
            let cid_subtype = f.get(cid, "Subtype").and_then(Obj::as_name).unwrap_or("");
            let dd = f
                .get(cid, "FontDescriptor")
                .and_then(Obj::as_dict)
                .ok_or("CID font without FontDescriptor")?;
            let program = font_program(f, dd)?.ok_or("CID font without an embedded program")?;
            let mut widths = BTreeMap::new();
            if let Some(w) = f.get(cid, "W").and_then(Obj::as_array) {
                let mut i = 0;
                while i < w.len() {
                    let first: u32 = f
                        .resolve(&w[i])
                        .as_number()
                        .and_then(|n| n.parse().ok())
                        .ok_or("W array")?;
                    match f.resolve(w.get(i + 1).ok_or("W array")?) {
                        Obj::Array(list) => {
                            for (k, x) in list.iter().enumerate() {
                                let cidn = first + k as u32;
                                if cidn > 0xFFFF {
                                    return Err("W CID above 65535".into());
                                }
                                widths.insert(cidn as u16, dec(f.resolve(x), "W")?);
                            }
                            i += 2;
                        }
                        Obj::Number(last) => {
                            let last: u32 = last.parse().map_err(|_| "W range end")?;
                            let wv = dec(f.resolve(w.get(i + 2).ok_or("W array")?), "W")?;
                            if last < first || last - first > 65535 {
                                return Err("W range".into());
                            }
                            for c in first..=last {
                                widths.insert(c as u16, wv.clone());
                            }
                            i += 3;
                        }
                        other => return Err(format!("W element {}", render(other))),
                    }
                }
            }
            let default_width = match f.get(cid, "DW") {
                Some(o) => dec(o, "DW")?,
                None => Decimal::from_i64(1000),
            };
            let mut to_unicode = BTreeMap::new();
            if let Some(tu) = f.get(d, "ToUnicode") {
                let cmap = f.decode_stream(tu)?;
                for (gid, c) in crate::embed::parse_to_unicode(&cmap).unwrap_or_default() {
                    to_unicode.insert(gid, c.to_string());
                }
            }
            let font = CidFont {
                base_font,
                program,
                widths,
                default_width,
                descriptor: descriptor(f, dd)?,
                to_unicode,
                glyphs: BTreeSet::new(),
            };
            match cid_subtype {
                "CIDFontType0" => Ok(ExactFont::CidCff(font)),
                "CIDFontType2" => {
                    let map = f
                        .get(cid, "CIDToGIDMap")
                        .and_then(Obj::as_name)
                        .unwrap_or("Identity");
                    if map != "Identity" {
                        return Err("CIDFontType2 with a non-Identity CIDToGIDMap".into());
                    }
                    Ok(ExactFont::CidTrueType(font))
                }
                other => Err(format!("descendant subtype {other:?}")),
            }
        }
        other => Err(format!("font subtype {other:?} is not supported")),
    }
}

/// Rebuilds a read PDF as an exact document (see module docs).
pub fn reemit(f: &PdfFile) -> Result<ExactDocument, String> {
    let mut doc = ExactDocument::default();
    for (i, page) in f.pages()?.iter().enumerate() {
        let mb = f
            .page_attr(page, "MediaBox")
            .ok_or_else(|| format!("page {}: no MediaBox", i + 1))?;
        let [x0, y0, x1, y1] = dec_array::<4>(f, mb, "MediaBox")?;
        if x0.approx() != 0.0 || y0.approx() != 0.0 {
            return Err(format!(
                "page {}: MediaBox origin [{x0} {y0}] is not [0 0]; the exact route writes [0 0 w h]",
                i + 1
            ));
        }
        let content = f.page_content(page)?;
        for (name, fd) in f.page_fonts(page) {
            if doc.fonts.contains_key(&name) {
                continue;
            }
            let font =
                font_from_dict(f, fd).map_err(|e| format!("page {} font /{name}: {e}", i + 1))?;
            doc.fonts.insert(name, font);
        }
        doc.pages.push(ExactPage {
            width: x1,
            height: y1,
            content: Content::Verbatim(content),
        });
    }
    Ok(doc)
}

/// Summary of a font resource for comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
struct FontFacts {
    subtype: String,
    base_font: String,
    /// (`FontFile` key, program length, sha256, Length1/2/3 or subtype)
    program: Option<(String, usize, String, String)>,
    widths: String,
    encoding: String,
    descriptor: String,
    to_unicode: Option<String>,
}

fn font_facts(f: &PdfFile, d: &BTreeMap<String, Obj>) -> FontFacts {
    let subtype = f
        .get(d, "Subtype")
        .and_then(Obj::as_name)
        .unwrap_or("?")
        .to_string();
    let base_font = f
        .get(d, "BaseFont")
        .and_then(Obj::as_name)
        .unwrap_or("?")
        .to_string();
    let target = if subtype == "Type0" {
        f.get(d, "DescendantFonts")
            .and_then(Obj::as_array)
            .and_then(|a| a.first())
            .and_then(|x| f.resolve(x).as_dict())
            .unwrap_or(d)
    } else {
        d
    };
    let desc = f.get(target, "FontDescriptor").and_then(Obj::as_dict);
    let program = desc.and_then(|dd| {
        for key in ["FontFile", "FontFile2", "FontFile3"] {
            if let Some(ff) = f.get(dd, key) {
                let data = f.decode_stream(ff).unwrap_or_default();
                let fd = ff.as_dict().cloned().unwrap_or_default();
                let extra = ["Length1", "Length2", "Length3", "Subtype"]
                    .iter()
                    .filter_map(|k| f.get(&fd, k).map(|v| format!("{k}={}", render(v))))
                    .collect::<Vec<_>>()
                    .join(" ");
                return Some((key.to_string(), data.len(), sha256::hex(&data), extra));
            }
        }
        None
    });
    let widths = match (f.get(target, "Widths"), f.get(target, "W")) {
        (Some(w), _) => format!(
            "FirstChar {} Widths {}",
            f.get(target, "FirstChar").map(render).unwrap_or_default(),
            render(w)
        ),
        (_, Some(w)) => format!(
            "DW {} W {}",
            f.get(target, "DW")
                .map(render)
                .unwrap_or_else(|| "1000".into()),
            render(w)
        ),
        _ => String::from("(none)"),
    };
    let encoding = f
        .get(d, "Encoding")
        .map(render)
        .unwrap_or_else(|| "(none)".into());
    let descriptor = desc
        .map(|dd| {
            let mut copy = dd.clone();
            for k in ["FontFile", "FontFile2", "FontFile3"] {
                copy.remove(k);
            }
            render(&Obj::Dict(copy))
        })
        .unwrap_or_else(|| "(none)".into());
    let to_unicode = f
        .get(d, "ToUnicode")
        .map(|tu| sha256::hex(&f.decode_stream(tu).unwrap_or_default()));
    FontFacts {
        subtype,
        base_font,
        program,
        widths,
        encoding,
        descriptor,
        to_unicode,
    }
}

fn op_summary(op: &Op) -> String {
    let mut v = Vec::new();
    op.write(&mut v);
    String::from_utf8_lossy(v.trim_ascii_end()).into_owned()
}

/// Compares two PDFs page by page and font by font.
pub fn classify(a: &PdfFile, b: &PdfFile, label_a: &str, label_b: &str) -> Report {
    let mut r = Report::default();
    let pages_a = a.pages().unwrap_or_default();
    let pages_b = b.pages().unwrap_or_default();
    if pages_a.len() != pages_b.len() {
        r.note(
            Category::PageGeometry,
            format!(
                "page count {} ({label_a}) vs {} ({label_b})",
                pages_a.len(),
                pages_b.len()
            ),
        );
    } else {
        r.same(format!("page count {}", pages_a.len()));
    }
    for (i, (pa, pb)) in pages_a.iter().zip(&pages_b).enumerate() {
        let n = i + 1;
        let mba = a.page_attr(pa, "MediaBox").map(render).unwrap_or_default();
        let mbb = b.page_attr(pb, "MediaBox").map(render).unwrap_or_default();
        let norm = |s: &str| s.replace(' ', "");
        if norm(&mba) != norm(&mbb) {
            r.note(
                Category::PageGeometry,
                format!("page {n} MediaBox {mba} vs {mbb}"),
            );
        } else {
            r.same(format!("page {n} MediaBox {mba}"));
        }
        let ca = a.page_content(pa);
        let cb = b.page_content(pb);
        match (&ca, &cb) {
            (Ok(ca), Ok(cb)) => {
                if ca == cb {
                    r.same(format!(
                        "page {n} content stream byte-identical ({} bytes)",
                        ca.len()
                    ));
                    r.content_byte_identical.push(true);
                    r.content_ops_identical.push(true);
                } else {
                    r.content_byte_identical.push(false);
                    match (exact::parse(ca), exact::parse(cb)) {
                        (Ok(oa), Ok(ob)) => {
                            if oa == ob {
                                r.content_ops_identical.push(true);
                                r.note(
                                    Category::ContentFormatting,
                                    format!(
                                        "page {n} content: {} operators identical, bytes differ only in formatting ({} vs {} bytes)",
                                        oa.len(),
                                        ca.len(),
                                        cb.len()
                                    ),
                                );
                            } else {
                                r.content_ops_identical.push(false);
                                let same_shape = oa.len() == ob.len()
                                    && oa.iter().zip(&ob).all(|(x, y)| {
                                        std::mem::discriminant(x) == std::mem::discriminant(y)
                                    });
                                let cat = if same_shape {
                                    Category::ContentOperands
                                } else {
                                    Category::ContentOperators
                                };
                                let mut shown = 0;
                                for (k, (x, y)) in oa.iter().zip(&ob).enumerate() {
                                    if x != y {
                                        r.note(
                                            cat,
                                            format!(
                                                "page {n} op {k}: {} | {}",
                                                op_summary(x),
                                                op_summary(y)
                                            ),
                                        );
                                        shown += 1;
                                        if shown == 5 {
                                            break;
                                        }
                                    }
                                }
                                if oa.len() != ob.len() {
                                    r.note(
                                        cat,
                                        format!(
                                            "page {n} operator count {} vs {}",
                                            oa.len(),
                                            ob.len()
                                        ),
                                    );
                                }
                            }
                        }
                        (Err(e), _) => {
                            r.content_ops_identical.push(false);
                            r.note(
                                Category::ContentUnsupported,
                                format!("page {n} {label_a}: {e}"),
                            );
                        }
                        (_, Err(e)) => {
                            r.content_ops_identical.push(false);
                            r.note(
                                Category::ContentUnsupported,
                                format!("page {n} {label_b}: {e}"),
                            );
                        }
                    }
                }
            }
            (Err(e), _) => {
                r.content_byte_identical.push(false);
                r.content_ops_identical.push(false);
                r.note(
                    Category::ContentUnsupported,
                    format!("page {n} {label_a}: {e}"),
                );
            }
            (_, Err(e)) => {
                r.content_byte_identical.push(false);
                r.content_ops_identical.push(false);
                r.note(
                    Category::ContentUnsupported,
                    format!("page {n} {label_b}: {e}"),
                );
            }
        }
        let fa = a.page_fonts(pa);
        let fb = b.page_fonts(pb);
        let names: BTreeSet<&String> = fa.keys().chain(fb.keys()).collect();
        for name in names {
            match (fa.get(name), fb.get(name)) {
                (Some(da), Some(db)) => {
                    let x = font_facts(a, da);
                    let y = font_facts(b, db);
                    let key = format!("page {n} /{name}");
                    if x.subtype != y.subtype {
                        r.note(
                            Category::FontMetadata,
                            format!("{key} subtype {} vs {}", x.subtype, y.subtype),
                        );
                    }
                    if x.base_font != y.base_font {
                        r.note(
                            Category::FontMetadata,
                            format!("{key} BaseFont {} vs {}", x.base_font, y.base_font),
                        );
                    }
                    match (&x.program, &y.program) {
                        (Some(pa), Some(pb)) => {
                            let identical = pa.2 == pb.2;
                            r.font_program_identical.insert(key.clone(), identical);
                            if identical {
                                r.same(format!(
                                    "{key} program {} {} bytes sha256 {} ({})",
                                    pa.0,
                                    pa.1,
                                    &pa.2[..16],
                                    pa.3
                                ));
                            } else {
                                r.note(
                                    Category::FontProgram,
                                    format!(
                                        "{key} program {} {} bytes sha256 {} ({}) vs {} {} bytes sha256 {} ({})",
                                        pa.0, pa.1, &pa.2[..16], pa.3, pb.0, pb.1, &pb.2[..16], pb.3
                                    ),
                                );
                            }
                        }
                        (None, None) => r.same(format!(
                            "{key} not embedded on either side ({})",
                            x.base_font
                        )),
                        (pa, pb) => {
                            r.font_program_identical.insert(key.clone(), false);
                            r.note(
                                Category::FontProgram,
                                format!(
                                    "{key} embedded {} vs {}",
                                    pa.as_ref().map_or("no".to_string(), |p| format!(
                                        "{} {} bytes",
                                        p.0, p.1
                                    )),
                                    pb.as_ref().map_or("no".to_string(), |p| format!(
                                        "{} {} bytes",
                                        p.0, p.1
                                    ))
                                ),
                            );
                        }
                    }
                    let norm = |s: &str| s.replace(' ', "");
                    if norm(&x.widths) != norm(&y.widths) {
                        r.note(Category::FontMetadata, format!("{key} widths differ"));
                    }
                    if norm(&x.encoding) != norm(&y.encoding) {
                        r.note(
                            Category::FontMetadata,
                            format!("{key} encoding {} vs {}", x.encoding, y.encoding),
                        );
                    }
                    if norm(&x.descriptor) != norm(&y.descriptor) {
                        r.note(
                            Category::FontMetadata,
                            format!(
                                "{key} descriptor differs: {} vs {}",
                                x.descriptor, y.descriptor
                            ),
                        );
                    }
                    if x.to_unicode != y.to_unicode {
                        r.note(
                            Category::FontMetadata,
                            format!(
                                "{key} ToUnicode {} vs {}",
                                x.to_unicode.as_deref().map_or("absent", |s| &s[..16]),
                                y.to_unicode.as_deref().map_or("absent", |s| &s[..16])
                            ),
                        );
                    }
                }
                (Some(_), None) => r.note(
                    Category::FontResources,
                    format!("page {n} /{name} only in {label_a}"),
                ),
                (None, Some(_)) => r.note(
                    Category::FontResources,
                    format!("page {n} /{name} only in {label_b}"),
                ),
                (None, None) => {}
            }
        }
    }
    // Layout, compression, identity.
    let (xa, xb) = (&a.features, &b.features);
    if xa.version != xb.version {
        r.note(
            Category::ObjectLayout,
            format!("PDF version {} vs {}", xa.version, xb.version),
        );
    }
    if xa.xref_stream != xb.xref_stream || xa.object_streams != xb.object_streams {
        r.note(
            Category::ObjectLayout,
            format!(
                "cross-reference {} with {} object stream(s) vs {} with {}",
                if xa.xref_stream { "stream" } else { "table" },
                xa.object_streams,
                if xb.xref_stream { "stream" } else { "table" },
                xb.object_streams
            ),
        );
    }
    if xa.object_count != xb.object_count || xa.object_order != xb.object_order {
        r.note(
            Category::ObjectLayout,
            format!(
                "{} objects in order {:?} vs {} objects in order {:?}",
                xa.object_count, xa.object_order, xb.object_count, xb.object_order
            ),
        );
    }
    if xa.filters != xb.filters {
        r.note(
            Category::Compression,
            format!("stream filters {:?} vs {:?}", xa.filters, xb.filters),
        );
    }
    if xa.has_id != xb.has_id {
        r.note(
            Category::DocumentIdentity,
            format!("trailer /ID {} vs {}", xa.has_id, xb.has_id),
        );
    }
    let ia = a.info().cloned().unwrap_or_default();
    let ib = b.info().cloned().unwrap_or_default();
    let keys: BTreeSet<&String> = ia.keys().chain(ib.keys()).collect();
    for k in keys {
        let va = ia.get(k).map(render);
        let vb = ib.get(k).map(render);
        if va != vb {
            r.note(
                Category::DocumentIdentity,
                format!(
                    "Info /{k} {} vs {}",
                    va.unwrap_or_else(|| "absent".into()),
                    vb.unwrap_or_else(|| "absent".into())
                ),
            );
        }
    }
    r
}
