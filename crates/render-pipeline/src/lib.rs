//! FlashTeX render pipeline.
//!
//! Original Rust implementation: compiler parse tree -> styled blocks ->
//! font-engine shaping (kerning + ligatures, source-byte clusters) ->
//! paragraph-layout Knuth–Plass line breaking and page breaking ->
//! math-layout Appendix G boxes with explicit rules -> TeX page builder
//! (`pagebuild`) -> display list v2 (glyph runs + rules, content-addressed
//! fonts, original glyph ids) -> runtime-v1 `compile_result` fallback. No
//! TeX engine is invoked at any point. See README.md for scope, sibling
//! pins and limitations.

pub mod adapter;
pub mod cff;
pub mod display;
pub mod fonts;
pub mod ids;
pub mod incremental;
pub mod mathfont;
pub mod mathtex;
pub mod pagebuild;
pub mod params;
pub mod pdf;
pub mod protocol;
pub mod shape;
pub mod style;
pub mod tfm;
pub mod typeset;
pub mod v1;

pub use display::DisplayList;
pub use fonts::FontSet;
pub use incremental::RenderCache;
pub use style::Stylesheet;

use flashtex_compiler::parser::SourceDocument;

/// Everything `render` produces. The runtime-v1 payload is derived per
/// request by `v1::fallback` because it depends on negotiated capabilities.
pub struct Rendered {
    /// Display list v2 (the authoritative geometry).
    pub v2: DisplayList,
    /// Wall-clock milliseconds spent in `render` (parse + layout + output).
    pub elapsed_ms: f64,
    /// Layout passes run (1 unless `\pageref` needed page numbers).
    pub passes: u32,
}

/// `\pageref` values converge in two passes in practice; the cap bounds a
/// document whose page numbers oscillate (reported, not looped forever).
pub const MAX_LABEL_PASSES: u32 = 3;

/// Options that runtime-v1 cannot carry and the compiler does not expose.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Class options assumed when the source has no `\documentclass` (the
    /// visual-oracle harness and the Mac app send body-only documents). The
    /// FlashTeX compiler's implicit preamble is `12pt`, US Letter, 1in
    /// margins, `\parindent 0pt`; that is the default here too.
    pub default_class_options: String,
    /// `\parindent` when the source sets neither a class nor the length.
    pub default_parindent_pt: f64,
    /// `secnumdepth` when the source does not set the counter: 2 numbers
    /// `\section` and `\subsection` (article); 0 numbers nothing (the
    /// visual-oracle preamble, which the harness strips before sending the
    /// body).
    pub default_secnumdepth: u8,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            default_class_options: "12pt".into(),
            default_parindent_pt: 0.0,
            default_secnumdepth: 2,
        }
    }
}

/// Renders one project. `documents` are indexed by the compiler's
/// `DocumentId`; `entry_path` selects the root (the first document when
/// absent). `revision` is echoed into both outputs.
pub fn render(
    documents: &[SourceDocument<'_>],
    entry_path: &str,
    revision: u64,
    project_id: &str,
    fonts: &FontSet,
    options: &RenderOptions,
) -> Rendered {
    render_cached(documents, entry_path, revision, project_id, fonts, options, None)
}

/// [`render`] with a block cache that outlives requests (`RenderCache`):
/// unchanged paragraphs are reused, the output is byte-identical.
#[allow(clippy::too_many_arguments)]
pub fn render_cached(
    documents: &[SourceDocument<'_>],
    entry_path: &str,
    revision: u64,
    project_id: &str,
    fonts: &FontSet,
    options: &RenderOptions,
    cache: Option<&RenderCache>,
) -> Rendered {
    let started = std::time::Instant::now();
    let parsed = flashtex_compiler::parser::parse_project(documents, entry_path);
    let texts: Vec<&str> = documents.iter().map(|d| d.text).collect();
    let paths: Vec<&str> = documents.iter().map(|d| d.path).collect();
    let entry_index = documents.iter().position(|d| d.path == entry_path).unwrap_or(0);
    let mut labels = adapter::Labels::from_parsed(&parsed);
    let max_passes = if adapter::Labels::needs_pages(&parsed) { MAX_LABEL_PASSES } else { 1 };
    let mut passes = 0;
    loop {
        passes += 1;
        let doc = adapter::adapt(&texts, entry_index, &parsed, options, &labels);
        let mut diagnostics: Vec<display::Diagnostic> = parsed
            .diagnostics
            .iter()
            .map(|d| display::Diagnostic::from_compiler(d, &paths))
            .collect();
        diagnostics.extend(doc.diagnostics.iter().cloned());
        let mut ctx = typeset::Context::new(fonts, &doc.style, &paths);
        let laid = typeset::build(&mut ctx, &doc, cache);
        diagnostics.extend(ctx.take_diagnostics());
        if max_passes > 1 {
            let pages = typeset::label_pages(&laid);
            if pages == labels.pages {
                // Converged: the numbers shown are the pages they sit on.
            } else if passes < max_passes {
                labels.pages = pages;
                continue;
            } else {
                labels.pages = pages;
                diagnostics.push(display::Diagnostic::warning(
                    "labels_unstable",
                    format!("\\pageref values did not converge after {max_passes} layout passes; the last pass is shown"),
                    Vec::new(),
                ));
            }
        }
        let v2 = typeset::assemble(project_id, revision, documents, &doc.style, fonts, laid, diagnostics);
        return Rendered {
            v2,
            elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
            passes,
        };
    }
}
