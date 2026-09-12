//! FlashTeX render pipeline.
//!
//! Original Rust implementation: compiler parse tree -> styled blocks ->
//! font-engine shaping (kerning + ligatures) -> Knuth–Plass line breaking ->
//! TeX Appendix G math boxes -> page builder -> display list v2 (glyph runs +
//! explicit rules) -> runtime-v1 fallback items. No TeX engine is invoked.
//! See README.md for scope, sibling pins, shims and limitations.

pub mod adapter;
pub mod cff;
pub mod display;
pub mod fonts;
pub mod linebreak;
pub mod mathlayout;
pub mod otf;
pub mod otl;
pub mod pages;
pub mod params;
pub mod pdf;
pub mod protocol;
pub mod shape;
pub mod style;
pub mod v1;

pub use display::DisplayList;
pub use fonts::FontSet;
pub use style::Stylesheet;

/// Everything `render` produces.
pub struct Rendered {
    /// Display list v2 (the authoritative geometry).
    pub v2: DisplayList,
    /// runtime-v1 `compile_result` payload derived from `v2`.
    pub v1: v1::V1Payload,
}

/// Options that runtime-v1 cannot carry and the compiler does not expose.
#[derive(Debug, Clone, Default)]
pub struct RenderOptions {
    /// Defaults applied when the source has no `\documentclass` (e.g. the
    /// visual-oracle harness sends body-only documents): class options such
    /// as `12pt`.
    pub default_class_options: String,
    /// `\parindent` when the source does not set it (points).
    pub default_parindent_pt: Option<f64>,
    /// Emit the proposed optional `font` field on v1 text items.
    pub v1_font_hints: bool,
}

/// Renders one document. `path` is the project-relative source path used in
/// every source range; `revision` is echoed into the display list.
pub fn render(
    path: &str,
    text: &str,
    revision: u64,
    project_id: &str,
    fonts: &FontSet,
    options: &RenderOptions,
) -> Rendered {
    let parsed = flashtex_compiler::parser::parse(text);
    let doc = adapter::adapt(text, &parsed, options);
    let mut diagnostics: Vec<display::Diagnostic> = parsed
        .diagnostics
        .iter()
        .map(|d| display::Diagnostic::from_compiler(d, path))
        .collect();
    diagnostics.extend(doc.diagnostics.iter().cloned());
    let mut ctx = pages::Context::new(fonts, &doc.style, path);
    let laid = pages::build(&mut ctx, &doc);
    diagnostics.extend(ctx.diagnostics.drain(..));
    let v2 = display::assemble(project_id, revision, path, text, &doc.style, fonts, laid, diagnostics);
    let v1 = v1::fallback(&v2, options.v1_font_hints);
    Rendered { v2, v1 }
}
