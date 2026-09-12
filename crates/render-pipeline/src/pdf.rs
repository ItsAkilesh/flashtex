//! `--pdf` output through the `pdf` sibling (4bd8c2e).
//!
//! That crate consumes runtime-v1 pages on the negotiated route
//! (`rules-v1`, `font-hints-v1`): typed rules are drawn as filled
//! rectangles from their real geometry, and font hints pick the Latin
//! Modern regular/bold/italic faces (each embedded whole from the local
//! Latin Modern directory). It has no glyph-run API yet, so this is still a
//! SHIM over the v1 fallback: positions are the pipeline's, but the writer
//! re-encodes text by character and advances with its own widths, and math
//! glyphs are drawn from Latin Modern Roman/Symbol rather than Latin Modern
//! Math. Requested pdf API: accept v2 glyph runs (font id + original GIDs +
//! positions) — see docs/proposals/rendering-abi.md.

use std::path::Path;

use flashtex_pdf::embed::EmbedFont;
use flashtex_pdf::{CompileResult, FontHint, Item, Page, RenderOptions as PdfOptions, RuleItem, Style, TextItem, Weight};

use crate::display::DisplayList;
use crate::v1::{self, Capabilities, V1Item, CAP_FONT_HINTS, CAP_RULES};

pub struct PdfOut {
    pub bytes: Vec<u8>,
    pub warnings: Vec<String>,
    /// PostScript name of the embedded document face, if any.
    pub embedded: Option<String>,
}

fn hint(h: &v1::FontHint) -> FontHint {
    FontHint {
        family: h.family.to_string(),
        weight: if h.weight == "bold" { Weight::Bold } else { Weight::Normal },
        style: if h.style == "italic" { Style::Italic } else { Style::Normal },
    }
}

pub fn write_pdf(v2: &DisplayList) -> Result<PdfOut, String> {
    let caps = Capabilities {
        rules: true,
        font_hints: true,
        display_list: false,
    };
    let accepted = vec![CAP_RULES.to_string(), CAP_FONT_HINTS.to_string()];
    let v1 = v1::fallback(v2, caps, Some(accepted.clone()));
    let pages = v1
        .pages
        .iter()
        .map(|p| Page {
            number: p.number,
            width_pt: p.width_pt,
            height_pt: p.height_pt,
            items: p
                .items
                .iter()
                .map(|it| match it {
                    V1Item::Text {
                        text,
                        x_pt,
                        baseline_y_pt,
                        font_size_pt,
                        font,
                        ..
                    } => Item::Text(TextItem {
                        text: text.clone(),
                        x_pt: *x_pt,
                        baseline_y_pt: *baseline_y_pt,
                        font_size_pt: *font_size_pt,
                        font: font.as_ref().map(hint),
                    }),
                    V1Item::Rule {
                        x_pt,
                        y_pt,
                        width_pt,
                        height_pt,
                        ..
                    } => Item::Rule(RuleItem {
                        x_pt: *x_pt,
                        y_pt: *y_pt,
                        width_pt: *width_pt,
                        height_pt: *height_pt,
                    }),
                })
                .collect(),
        })
        .collect();
    let result = CompileResult {
        pages,
        capabilities: Some(accepted),
    };
    // Embed the regular Latin Modern text face the document used as the
    // document face; the hints select bold/italic siblings from its directory.
    let face = v2
        .fonts
        .iter()
        .filter(|f| f.format == "opentype-cff" && f.postscript_name.starts_with("LMRoman"))
        .find(|f| f.postscript_name.ends_with("-Regular"))
        .or_else(|| v2.fonts.iter().find(|f| f.format == "opentype-cff"));
    let mut warnings = Vec::new();
    let mut embedded = None;
    let options = match face.and_then(|f| f.path.as_deref()) {
        Some(path) => match EmbedFont::load(Path::new(path)) {
            Ok(font) => {
                embedded = Some(font.font.postscript_name.clone());
                PdfOptions::with_document_face(font)
            }
            Err(e) => {
                warnings.push(format!("could not embed {path}: {e}; falling back to Times"));
                PdfOptions::default()
            }
        },
        None => {
            warnings.push("no OpenType face used by the document; Times (unembedded) output".into());
            PdfOptions::default()
        }
    };
    let out = flashtex_pdf::render_pdf_with(&result, &options).map_err(|e| e.to_string())?;
    warnings.extend(out.warnings);
    Ok(PdfOut {
        bytes: out.bytes,
        warnings,
        embedded,
    })
}
