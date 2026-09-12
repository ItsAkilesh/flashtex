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
//! Rules: with `rules-v1` negotiated, typed `rule` items give the rectangle's
//! top-left corner and size in page space; they are drawn as `re f` after
//! converting to PDF's bottom-left, y-up space. On the legacy route (no
//! capabilities), the FT-002 compiler emits a fraction bar as a text item made
//! only of U+2500 BOX DRAWINGS LIGHT HORIZONTAL, N characters wide at an
//! assumed 0.5 em each, with the item's baseline at the bottom edge of the bar;
//! such items are drawn as rectangles too. That legacy convention is an
//! approximation (width and thickness are inferred from the font size, not
//! measured) and exists only on the legacy route.
//!
//! Font hints: with `font-hints-v1` negotiated, each text item may name a
//! family/weight/style. Latin Modern resolves to the matching
//! `lmroman10-*.otf` embedded as its own font object; Times resolves to the
//! base-14 Times variants; Courier and Helvetica resolve to their base-14
//! variants; anything else is substituted by the document face
//! at the requested weight/style and reported in the warnings. Additional
//! faces get resources `/F4`, `/F5`, … and their objects follow the document
//! font after the pages.

use crate::embed::{EmbedFont, EmbeddedSubset};
use crate::encoding;
use crate::{
    CAP_FONT_HINTS_V1, CAP_RULES_V1, CompileResult, FontHint, Item, PdfError, PdfOutput,
    RenderOptions, Style, Weight,
};
use std::collections::{BTreeMap, BTreeSet};
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

/// Base-14 Times variants selectable through `font-hints-v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimesVariant {
    Roman,
    Bold,
    Italic,
    BoldItalic,
}

impl TimesVariant {
    fn of(weight: Weight, style: Style) -> Self {
        match (weight, style) {
            (Weight::Normal, Style::Normal) => TimesVariant::Roman,
            (Weight::Bold, Style::Normal) => TimesVariant::Bold,
            (Weight::Normal, Style::Italic) => TimesVariant::Italic,
            (Weight::Bold, Style::Italic) => TimesVariant::BoldItalic,
        }
    }

    fn base_font(self) -> &'static str {
        match self {
            TimesVariant::Roman => "Times-Roman",
            TimesVariant::Bold => "Times-Bold",
            TimesVariant::Italic => "Times-Italic",
            TimesVariant::BoldItalic => "Times-BoldItalic",
        }
    }
}

/// Latin Modern Roman file for a weight/style.
fn latin_modern_file(weight: Weight, style: Style) -> &'static str {
    match (weight, style) {
        (Weight::Normal, Style::Normal) => "lmroman10-regular.otf",
        (Weight::Bold, Style::Normal) => "lmroman10-bold.otf",
        (Weight::Normal, Style::Italic) => "lmroman10-italic.otf",
        (Weight::Bold, Style::Italic) => "lmroman10-bolditalic.otf",
    }
}

fn describe(weight: Weight, style: Style) -> &'static str {
    match (weight, style) {
        (Weight::Normal, Style::Normal) => "regular",
        (Weight::Bold, Style::Normal) => "bold",
        (Weight::Normal, Style::Italic) => "italic",
        (Weight::Bold, Style::Italic) => "bold italic",
    }
}

fn is_latin_modern_family(family: &str) -> bool {
    let f = family.to_ascii_lowercase();
    f.starts_with("latin modern") || f.starts_with("lmroman") || f == "lm roman" || f == "lm"
}

fn is_times_family(family: &str) -> bool {
    let f = family.to_ascii_lowercase();
    f.starts_with("times")
}

/// Base-14 Courier or Helvetica at a weight/style, when `family` names one.
fn sans_or_mono_base_font(family: &str, weight: Weight, style: Style) -> Option<&'static str> {
    let f = family.to_ascii_lowercase();
    let [regular, bold, oblique, bold_oblique] = if f.starts_with("courier") {
        [
            "Courier",
            "Courier-Bold",
            "Courier-Oblique",
            "Courier-BoldOblique",
        ]
    } else if f.starts_with("helvetica") {
        [
            "Helvetica",
            "Helvetica-Bold",
            "Helvetica-Oblique",
            "Helvetica-BoldOblique",
        ]
    } else {
        return None;
    };
    Some(match (weight, style) {
        (Weight::Normal, Style::Normal) => regular,
        (Weight::Bold, Style::Normal) => bold,
        (Weight::Normal, Style::Italic) => oblique,
        (Weight::Bold, Style::Italic) => bold_oblique,
    })
}

/// One font written after the pages: the document's embedded font or a face
/// selected by a hint. The embedded variant is large (it owns a parsed font
/// and its subset); there are at most a handful per document.
#[allow(clippy::large_enum_variant)]
enum TrailingFont {
    Base14 {
        resource: encoding::Font,
        base_font: &'static str,
    },
    Embedded {
        resource: encoding::Font,
        font: EmbedFont,
        wanted: BTreeSet<char>,
        subset: Option<EmbeddedSubset>,
    },
}

impl TrailingFont {
    fn object_count(&self) -> usize {
        match self {
            TrailingFont::Base14 { .. } => 1,
            TrailingFont::Embedded { .. } => 5,
        }
    }

    fn resource(&self) -> encoding::Font {
        match self {
            TrailingFont::Base14 { resource, .. } | TrailingFont::Embedded { resource, .. } => {
                *resource
            }
        }
    }
}

/// How one text item's font hint (or absence of one) was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Resolved {
    face: encoding::Face,
    primary: encoding::Font,
    /// Index into the trailing-font list when `primary` is embedded.
    embedded: Option<usize>,
}

struct FontTable {
    fonts: Vec<TrailingFont>,
    /// Index of the document's embedded font (`/F3`), if any.
    document_embedded: Option<usize>,
    document_face: encoding::Face,
    /// Directory holding Latin Modern files, if known.
    latin_modern_dir: Option<std::path::PathBuf>,
    cache: BTreeMap<FontHint, Resolved>,
    next_extra: u8,
}

impl FontTable {
    fn new(options: &RenderOptions, warnings: &mut Vec<String>) -> Self {
        let document_face = match (options.face, &options.embed_font) {
            (encoding::Face::Embedded, None) => {
                warnings.push(
                    "document face 'embedded' requested but no font is embedded; using Times"
                        .into(),
                );
                encoding::Face::Times
            }
            (face, _) => face,
        };
        let mut fonts = Vec::new();
        let mut latin_modern_dir = None;
        if let Some(font) = &options.embed_font {
            if font.font.postscript_name.starts_with("LMRoman") {
                latin_modern_dir = font.source.parent().map(|p| p.to_path_buf());
            }
            fonts.push(TrailingFont::Embedded {
                resource: encoding::Font::Embedded,
                font: font.clone(),
                wanted: BTreeSet::new(),
                subset: None,
            });
        }
        if latin_modern_dir.is_none() {
            latin_modern_dir = crate::embed::candidate_paths()
                .into_iter()
                .find(|p| p.ends_with(crate::embed::LATIN_MODERN_FILE) && p.is_file())
                .and_then(|p| p.parent().map(|d| d.to_path_buf()));
        }
        FontTable {
            document_embedded: options.embed_font.as_ref().map(|_| 0),
            fonts,
            document_face,
            latin_modern_dir,
            cache: BTreeMap::new(),
            next_extra: 0,
        }
    }

    fn default_resolution(&self) -> Resolved {
        match (self.document_face, self.document_embedded) {
            (encoding::Face::Embedded, Some(i)) => Resolved {
                face: encoding::Face::Embedded,
                primary: encoding::Font::Embedded,
                embedded: Some(i),
            },
            _ => Resolved {
                face: encoding::Face::Times,
                primary: encoding::Font::Times,
                embedded: None,
            },
        }
    }

    fn allocate_extra(&mut self) -> encoding::Font {
        let f = encoding::Font::Extra(self.next_extra);
        self.next_extra += 1;
        f
    }

    fn times_variant(&mut self, variant: TimesVariant) -> Resolved {
        if variant == TimesVariant::Roman {
            return Resolved {
                face: encoding::Face::Times,
                primary: encoding::Font::Times,
                embedded: None,
            };
        }
        self.base14(variant.base_font())
    }

    /// A WinAnsi-encoded base-14 text face written as its own font object.
    fn base14(&mut self, name: &'static str) -> Resolved {
        let existing = self.fonts.iter().find_map(|f| match f {
            TrailingFont::Base14 {
                resource,
                base_font,
            } if *base_font == name => Some(*resource),
            _ => None,
        });
        let resource = existing.unwrap_or_else(|| {
            let resource = self.allocate_extra();
            self.fonts.push(TrailingFont::Base14 {
                resource,
                base_font: name,
            });
            resource
        });
        Resolved {
            face: encoding::Face::Times,
            primary: resource,
            embedded: None,
        }
    }

    /// Latin Modern at a weight/style, or `Err(reason)` when unavailable.
    fn latin_modern(&mut self, weight: Weight, style: Style) -> Result<Resolved, String> {
        let dir = self.latin_modern_dir.clone().ok_or_else(|| {
            "no Latin Modern installation found (set FLASHTEX_LM_DIR)".to_string()
        })?;
        let path = dir.join(latin_modern_file(weight, style));
        if let Some(i) = self.fonts.iter().position(|f| match f {
            TrailingFont::Embedded { font, .. } => font.source == path,
            _ => false,
        }) {
            return Ok(Resolved {
                face: encoding::Face::Embedded,
                primary: self.fonts[i].resource(),
                embedded: Some(i),
            });
        }
        if !path.is_file() {
            return Err(format!("{} is not installed", path.display()));
        }
        let font = EmbedFont::load(&path)?;
        let resource = self.allocate_extra();
        self.fonts.push(TrailingFont::Embedded {
            resource,
            font,
            wanted: BTreeSet::new(),
            subset: None,
        });
        Ok(Resolved {
            face: encoding::Face::Embedded,
            primary: resource,
            embedded: Some(self.fonts.len() - 1),
        })
    }

    /// Resolves a hint, reporting substitutions once per distinct hint.
    fn resolve(&mut self, hint: &FontHint, warnings: &mut Vec<String>) -> Resolved {
        if let Some(r) = self.cache.get(hint) {
            return *r;
        }
        let (weight, style) = (hint.weight, hint.style);
        let variant = TimesVariant::of(weight, style);
        let resolved = if is_latin_modern_family(&hint.family) {
            match self.latin_modern(weight, style) {
                Ok(r) => r,
                Err(reason) => {
                    warnings.push(format!(
                        "font {:?} ({}) substituted by '{}': {reason}",
                        hint.family,
                        describe(weight, style),
                        variant.base_font()
                    ));
                    self.times_variant(variant)
                }
            }
        } else if is_times_family(&hint.family) {
            self.times_variant(variant)
        } else if let Some(name) = sans_or_mono_base_font(&hint.family, weight, style) {
            self.base14(name)
        } else {
            // Unknown family: the document face at the requested weight/style,
            // reported as a substitution and never claimed preserved.
            let document_is_lm = self.document_face == encoding::Face::Embedded
                && self.latin_modern_dir.is_some()
                && self
                    .document_embedded
                    .is_some_and(|i| match &self.fonts[i] {
                        TrailingFont::Embedded { font, .. } => {
                            font.font.postscript_name.starts_with("LMRoman")
                        }
                        _ => false,
                    });
            let (resolved, by) = if document_is_lm {
                match self.latin_modern(weight, style) {
                    Ok(r) => (r, format!("Latin Modern Roman {}", describe(weight, style))),
                    Err(_) => (self.times_variant(variant), variant.base_font().to_string()),
                }
            } else {
                (self.times_variant(variant), variant.base_font().to_string())
            };
            warnings.push(format!(
                "font {:?} ({}) substituted by '{by}'; requested metrics were not preserved",
                hint.family,
                describe(weight, style)
            ));
            resolved
        };
        self.cache.insert(hint.clone(), resolved);
        resolved
    }
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
            match item {
                Item::Text(t) => {
                    for (name, v) in [
                        ("x_pt", t.x_pt),
                        ("baseline_y_pt", t.baseline_y_pt),
                        ("font_size_pt", t.font_size_pt),
                    ] {
                        if !v.is_finite() {
                            return Err(PdfError::Invalid(format!(
                                "page {}: item {i}: {name} must be finite, got {v}",
                                page.number
                            )));
                        }
                    }
                    if t.font_size_pt <= 0.0 {
                        return Err(PdfError::Invalid(format!(
                            "page {}: item {i}: font_size_pt must be positive, got {}",
                            page.number, t.font_size_pt
                        )));
                    }
                }
                Item::Rule(r) => {
                    if !result.accepts(CAP_RULES_V1) {
                        return Err(PdfError::Invalid(format!(
                            "page {}: item {i} is a rule but {CAP_RULES_V1:?} was not accepted",
                            page.number
                        )));
                    }
                    for (name, v) in [
                        ("x_pt", r.x_pt),
                        ("y_pt", r.y_pt),
                        ("width_pt", r.width_pt),
                        ("height_pt", r.height_pt),
                    ] {
                        if !v.is_finite() || v.abs() > crate::protocol::MAX_MAGNITUDE {
                            return Err(PdfError::Invalid(format!(
                                "page {}: item {i}: {name} must be finite with magnitude <= 1e6, got {v}",
                                page.number
                            )));
                        }
                    }
                    if r.width_pt <= 0.0 || r.height_pt <= 0.0 {
                        return Err(PdfError::Invalid(format!(
                            "page {}: item {i}: rule dimensions must be positive",
                            page.number
                        )));
                    }
                }
            }
        }
    }

    let mut warnings = Vec::new();
    let page_count = result.pages.len();
    let legacy = result.legacy();
    let hints_accepted = result.accepts(CAP_FONT_HINTS_V1);

    // Pass 1: resolve every item's face and collect the characters each
    // embedded face must carry.
    let mut table = FontTable::new(options, &mut warnings);
    let mut resolutions: Vec<Vec<Option<Resolved>>> = Vec::with_capacity(page_count);
    for page in &result.pages {
        let mut per_item = Vec::with_capacity(page.items.len());
        for item in &page.items {
            let Item::Text(t) = item else {
                per_item.push(None);
                continue;
            };
            if legacy && is_rule_item(&t.text) {
                per_item.push(None);
                continue;
            }
            let resolved = match &t.font {
                Some(hint) if hints_accepted => table.resolve(hint, &mut warnings),
                Some(_) => {
                    return Err(PdfError::Invalid(format!(
                        "page {}: a font hint is present but {CAP_FONT_HINTS_V1:?} was not accepted",
                        page.number
                    )));
                }
                None => table.default_resolution(),
            };
            match (resolved.face, resolved.embedded, table.document_embedded) {
                (encoding::Face::Embedded, Some(i), _) => {
                    if let TrailingFont::Embedded { wanted, .. } = &mut table.fonts[i] {
                        wanted.extend(t.text.chars());
                    }
                }
                (encoding::Face::Times, _, Some(doc)) => {
                    if let TrailingFont::Embedded { wanted, .. } = &mut table.fonts[doc] {
                        wanted.extend(t.text.chars().filter(|&c| encoding::needs_embedding(c)));
                    }
                }
                _ => {}
            }
            per_item.push(Some(resolved));
        }
        resolutions.push(per_item);
    }

    // Build each embedded face's program and report the characters that
    // will end up as '?' (those no other font covers either).
    for entry in &mut table.fonts {
        let TrailingFont::Embedded {
            font,
            wanted,
            subset,
            ..
        } = entry
        else {
            continue;
        };
        let built = font
            .subset_for(wanted.iter().copied())
            .map_err(|e| PdfError::Invalid(format!("embedding {}: {e}", font.source.display())))?;
        let missing: Vec<String> = wanted
            .iter()
            .filter(|c| !built.chars.contains_key(c) && encoding::needs_embedding(**c))
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
        *subset = Some(built);
    }

    // Object numbers: pages first, then the trailing fonts in table order.
    let mut font_objects = Vec::with_capacity(table.fonts.len());
    let mut next = embedded_font_object(page_count);
    for f in &table.fonts {
        font_objects.push(next);
        next += f.object_count();
    }
    let mut resources = format!(
        "/{} 3 0 R /{} 4 0 R",
        encoding::Font::Times.resource_name(),
        encoding::Font::Symbol.resource_name()
    );
    for (f, obj) in table.fonts.iter().zip(&font_objects) {
        resources.push_str(&format!(" /{} {obj} 0 R", f.resource().resource_name()));
    }

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

    for (i, page) in result.pages.iter().enumerate() {
        let page_obj = FIRST_PAGE_OBJECT + 2 * i;
        let content_obj = page_obj + 1;
        let content = page_content(page, &resolutions[i], &table, legacy, &mut warnings);
        doc.object(
            page_obj,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [ 0 0 {} {} ] /Resources << /Font << {resources} >> >> /Contents {content_obj} 0 R >>",
                num(page.width_pt),
                num(page.height_pt),
            )
            .as_bytes(),
        );
        doc.stream(content_obj, &content);
    }

    for (f, &obj) in table.fonts.iter().zip(&font_objects) {
        match f {
            TrailingFont::Base14 { base_font, .. } => doc.object(
                obj,
                format!(
                    "<< /Type /Font /Subtype /Type1 /BaseFont /{base_font} /Encoding /WinAnsiEncoding >>"
                )
                .as_bytes(),
            ),
            TrailingFont::Embedded { subset, .. } => {
                let e = subset.as_ref().expect("built above");
                write_embedded_font(&mut doc, obj, e);
            }
        }
    }

    Ok(PdfOutput {
        bytes: doc.finish(),
        warnings,
    })
}

/// Writes the five objects of one embedded font starting at `font_obj`.
fn write_embedded_font(doc: &mut Document, font_obj: usize, e: &EmbeddedSubset) {
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

/// True when the legacy compiler meant this text item as a fraction rule.
/// Only consulted on the legacy route; with negotiated capabilities a rule is
/// a typed `rule` item and U+2500 is ordinary text.
pub fn is_rule_item(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c == '\u{2500}')
}

fn subset_lookup(table: &FontTable, index: Option<usize>) -> Option<&EmbeddedSubset> {
    index.and_then(|i| match &table.fonts[i] {
        TrailingFont::Embedded { subset, .. } => subset.as_ref(),
        _ => None,
    })
}

/// Builds one page's content stream. The page is left untouched (white) apart
/// from black text and rules; there is deliberately no background fill and no
/// theme input.
fn page_content(
    page: &crate::Page,
    resolutions: &[Option<Resolved>],
    table: &FontTable,
    legacy: bool,
    warnings: &mut Vec<String>,
) -> Vec<u8> {
    let mut out = Vec::new();
    // Non-stroking colour: black in DeviceGray. Set explicitly so output never
    // depends on viewer defaults.
    out.extend_from_slice(b"0 g\n");
    for (i, item) in page.items.iter().enumerate() {
        let t = match item {
            Item::Rule(r) => {
                // rules-v1: top-left corner in page space; PDF wants the
                // bottom-left corner in y-up space.
                let y = page.height_pt - r.y_pt - r.height_pt;
                writeln!(
                    out,
                    "{} {} {} {} re f",
                    num(r.x_pt),
                    num(y),
                    num(r.width_pt),
                    num(r.height_pt)
                )
                .expect("writing to Vec cannot fail");
                continue;
            }
            Item::Text(t) => t,
        };
        if legacy && is_rule_item(&t.text) {
            // Legacy fraction-bar convention (approximation, see module docs):
            // the bar occupies [baseline - thickness, baseline] in top-left
            // space; in PDF space its bottom edge is at height - baseline.
            let dashes = t.text.chars().count() as f64;
            let width = dashes * RULE_DASH_EM * t.font_size_pt;
            let thickness = RULE_THICKNESS_EM * t.font_size_pt;
            let y = page.height_pt - t.baseline_y_pt;
            writeln!(
                out,
                "{} {} {} {} re f",
                num(t.x_pt),
                num(y),
                num(width),
                num(thickness)
            )
            .expect("writing to Vec cannot fail");
            continue;
        }
        let resolved = resolutions[i].expect("text items are resolved in pass 1");
        let primary_subset = subset_lookup(table, resolved.embedded);
        let document_subset = subset_lookup(table, table.document_embedded);
        let primary_lookup = |c: char| primary_subset.and_then(|e| e.chars.get(&c).copied());
        let fallback_lookup = |c: char| document_subset.and_then(|e| e.chars.get(&c).copied());
        let faces = encoding::Faces {
            face: resolved.face,
            primary: resolved.primary,
            primary_lookup: primary_subset.map(|_| &primary_lookup as &dyn Fn(char) -> Option<u16>),
            fallback_lookup: document_subset
                .map(|_| &fallback_lookup as &dyn Fn(char) -> Option<u16>),
        };
        let encoded = encoding::encode_runs(&t.text, &faces);
        if !encoded.unrepresentable.is_empty() {
            let listed: Vec<String> = encoded
                .unrepresentable
                .iter()
                .map(|c| format!("{c:?} (U+{:04X})", *c as u32))
                .collect();
            let fonts = match primary_subset.or(document_subset) {
                Some(e) => format!("WinAnsiEncoding, Symbol, or embedded {}", e.base_font),
                None => "WinAnsiEncoding or Symbol".to_string(),
            };
            warnings.push(format!(
                "page {}: item {i} {:?}: {} not representable in {fonts}; written as '{}'",
                page.number,
                t.text,
                listed.join(", "),
                encoding::SUBSTITUTE as char
            ));
        }
        let x = t.x_pt;
        let y = page.height_pt - t.baseline_y_pt;
        writeln!(out, "BT\n{} {} Td", num(x), num(y)).expect("writing to Vec cannot fail");
        for run in &encoded.runs {
            writeln!(
                out,
                "/{} {} Tf",
                run.font.resource_name(),
                num(t.font_size_pt)
            )
            .expect("writing to Vec cannot fail");
            let two_byte = match run.font {
                encoding::Font::Embedded => true,
                encoding::Font::Extra(_) => {
                    matches!(
                        table.fonts.iter().find(|f| f.resource() == run.font),
                        Some(TrailingFont::Embedded { .. })
                    )
                }
                _ => false,
            };
            if two_byte {
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

/// The one PDF container implementation: objects are appended in numeric
/// order, offsets recorded, and the classic cross-reference table and
/// trailer written by [`Document::finish`]. Both the runtime-v1 route and
/// the exact route (`crate::exact`) write through it.
pub(crate) struct Document {
    bytes: Vec<u8>,
    /// Byte offset of each object, indexed by object number (index 0 unused).
    offsets: Vec<usize>,
}

impl Document {
    pub(crate) fn new() -> Self {
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

    pub(crate) fn object(&mut self, number: usize, body: &[u8]) {
        self.begin(number);
        self.bytes.extend_from_slice(body);
        self.bytes.extend_from_slice(b"\nendobj\n");
    }

    pub(crate) fn stream(&mut self, number: usize, data: &[u8]) {
        self.stream_with(number, "", data);
    }

    /// A stream whose dictionary carries extra entries (e.g. `/Length1`).
    pub(crate) fn stream_with(&mut self, number: usize, extra: &str, data: &[u8]) {
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

    fn finish(self) -> Vec<u8> {
        self.finish_with_info(5)
    }

    /// Writes the xref table and trailer; `info_obj` is the object number of
    /// the document information dictionary.
    pub(crate) fn finish_with_info(mut self, info_obj: usize) -> Vec<u8> {
        let xref_offset = self.bytes.len();
        let size = self.offsets.len();
        write!(self.bytes, "xref\n0 {size}\n0000000000 65535 f \n").expect("Vec write");
        for &offset in &self.offsets[1..] {
            writeln!(self.bytes, "{offset:010} 00000 n ").expect("Vec write");
        }
        write!(
            self.bytes,
            "trailer\n<< /Size {size} /Root 1 0 R /Info {info_obj} 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n"
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
