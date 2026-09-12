//! FlashTeX PDF writer (task FT-009).
//!
//! Turns runtime-v1 `compile_result` pages (`docs/contracts/runtime-v1.md`) into
//! a PDF 1.4 document. Everything here is original: the file structure, content
//! streams, cross-reference table, and the JSON reader are written by hand. No
//! TeX engine and no external crate is involved.
//!
//! Honest scope of this milestone:
//!
//! - Text only. Every `kind: text` item becomes one `Tj` at its baseline. Any
//!   other item kind is skipped and reported in the warnings list.
//! - By default the only fonts are the base-14 `Times-Roman` (WinAnsiEncoding)
//!   and `Symbol`; nothing is embedded, so glyph shapes and advance widths come
//!   from the viewer's substitutes. Characters outside both are written as `?`
//!   and reported, never silently dropped. Callers can opt in to embedding a
//!   subset of a Unicode TrueType font for those characters ([`RenderOptions`]).
//! - Pages are always white. The writer takes no theme input, so a dark preview
//!   in the Mac app cannot leak into the export.

pub mod embed;
pub mod encoding;
pub mod json;
pub mod protocol;
pub mod truetype;
pub mod verify;
pub mod writer;

/// One positioned text run. Coordinates follow runtime-v1: points, origin at the
/// top-left of the page, `baseline_y_pt` measured downwards.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub text: String,
    pub x_pt: f64,
    pub baseline_y_pt: f64,
    pub font_size_pt: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub number: u32,
    pub width_pt: f64,
    pub height_pt: f64,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompileResult {
    pub pages: Vec<Page>,
}

/// The rendered document plus everything the writer had to approximate.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfOutput {
    pub bytes: Vec<u8>,
    /// Human-readable notes about substitutions and skipped input. Empty means
    /// the document was reproduced exactly within this writer's stated scope.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PdfError {
    /// The input was not valid JSON.
    Json(String),
    /// The envelope was well-formed JSON but not a runtime-v1 `compile_result`.
    Protocol(String),
    /// The pages themselves cannot be represented (e.g. non-positive size).
    Invalid(String),
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdfError::Json(m) => write!(f, "invalid JSON: {m}"),
            PdfError::Protocol(m) => write!(f, "unsupported protocol input: {m}"),
            PdfError::Invalid(m) => write!(f, "cannot render: {m}"),
        }
    }
}

impl std::error::Error for PdfError {}

/// Rendering choices. The default embeds nothing.
#[derive(Debug, Clone, Default)]
pub struct RenderOptions {
    /// A Unicode TrueType font to subset and embed for characters outside
    /// WinAnsi and Symbol. `None` keeps the `?` + warning behaviour.
    pub embed_font: Option<embed::EmbedFont>,
}

/// Renders positioned pages to PDF bytes with default options (no embedding).
/// See the crate docs for scope.
pub fn render_pdf(result: &CompileResult) -> Result<PdfOutput, PdfError> {
    writer::render(result, &RenderOptions::default())
}

/// Renders positioned pages to PDF bytes with explicit options.
pub fn render_pdf_with(
    result: &CompileResult,
    options: &RenderOptions,
) -> Result<PdfOutput, PdfError> {
    writer::render(result, options)
}

/// Convenience: parse a runtime-v1 `compile_result` envelope and render it
/// with default options. Warnings from both stages (skipped item kinds,
/// substituted characters) are concatenated in order.
pub fn render_envelope(envelope_json: &str) -> Result<PdfOutput, PdfError> {
    render_envelope_with(envelope_json, &RenderOptions::default())
}

/// [`render_envelope`] with explicit options.
pub fn render_envelope_with(
    envelope_json: &str,
    options: &RenderOptions,
) -> Result<PdfOutput, PdfError> {
    let parsed = protocol::parse_compile_result(envelope_json)?;
    let mut out = render_pdf_with(&parsed.result, options)?;
    let mut warnings = parsed.warnings;
    warnings.append(&mut out.warnings);
    out.warnings = warnings;
    Ok(out)
}
