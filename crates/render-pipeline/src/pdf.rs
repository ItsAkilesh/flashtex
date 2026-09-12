//! `--pdf` output through the `pdf` sibling (52b3711).
//!
//! That crate accepts runtime-v1 text items only (one `Tj` per item, one
//! embedded document face, U+2500 runs drawn as rules); it has no glyph-run
//! or multi-font API yet. So this is a SHIM: the v2 display list is reduced
//! to the legacy v1 items (no typed rules, no font hints) and the body
//! Latin Modern face the pipeline actually used is embedded as the document
//! face. Consequences, stated rather than hidden: bold/italic/math glyphs
//! are drawn in the regular face by the PDF writer, and positions are the
//! pipeline's but advances are the writer's. Requested pdf API: accept v2
//! glyph runs (font id + original GIDs + positions) — see
//! docs/proposals/rendering-abi.md.

use std::path::Path;

use flashtex_pdf::embed::EmbedFont;
use flashtex_pdf::{CompileResult, Item, Page, RenderOptions as PdfOptions};

use crate::display::DisplayList;
use crate::v1::{self, Capabilities, V1Item};

pub struct PdfOut {
    pub bytes: Vec<u8>,
    pub warnings: Vec<String>,
    /// PostScript name of the embedded face, if any.
    pub embedded: Option<String>,
}

pub fn write_pdf(v2: &DisplayList) -> Result<PdfOut, String> {
    let v1 = v1::fallback(v2, Capabilities::default(), None);
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
                .filter_map(|it| match it {
                    V1Item::Text {
                        text,
                        x_pt,
                        baseline_y_pt,
                        font_size_pt,
                        ..
                    } => Some(Item {
                        text: text.clone(),
                        x_pt: *x_pt,
                        baseline_y_pt: *baseline_y_pt,
                        font_size_pt: *font_size_pt,
                    }),
                    V1Item::Rule { .. } => None,
                })
                .collect(),
        })
        .collect();
    let result = CompileResult { pages };
    // Embed the regular Latin Modern text face the document used, if any.
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
