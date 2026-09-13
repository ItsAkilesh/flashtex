#![allow(dead_code)]
use flashtex_compiler::parser::SourceDocument;
use flashtex_render_pipeline::v1::{self, Capabilities, V1Payload};
use flashtex_render_pipeline::{render, FontSet, RenderOptions, Rendered};

/// Whether Latin Modern resolves, for the ~110 `if !lm_available() { return }`
/// guards across this suite.
///
/// Those guards used to make a fontless run *silently green*: the tests did
/// not fail, they simply never executed, and `eprintln!` is captured by
/// libtest, so nothing was printed either. A whole-suite run with no
/// `FLASHTEX_*` set therefore reported success while measuring almost
/// nothing -- the same trap as an oracle harness scoring an OpenType
/// fallback as a pass.
///
/// So a missing Latin Modern is now a loud failure by default. A genuinely
/// fontless environment can still skip, but only by asking for it:
/// `FLASHTEX_ALLOW_FONTLESS_TESTS=1`, which restores the old `false`.
pub fn lm_available() -> bool {
    if FontSet::with_default_dirs(&[]).latin_modern_available() {
        return true;
    }
    if std::env::var_os("FLASHTEX_ALLOW_FONTLESS_TESTS").is_some() {
        return false;
    }
    panic!(
        "Latin Modern is not resolvable, so this test would have skipped silently \
         and the run would have been green without measuring anything. Point \
         FLASHTEX_FONT_DIRS at a directory of Latin Modern .otf files (the repo \
         bundles apps/mac/Fonts) and FLASHTEX_TFM_DIRS at its metrics, or set \
         FLASHTEX_ALLOW_FONTLESS_TESTS=1 to skip deliberately."
    );
}

pub fn render_docs(docs: &[(&str, &str)], entry: &str) -> Rendered {
    let fonts = FontSet::with_default_dirs(&[]);
    let sources: Vec<SourceDocument<'_>> = docs.iter().map(|(p, t)| SourceDocument { path: p, text: t }).collect();
    render(&sources, entry, 7, "test-project", &fonts, &RenderOptions::default())
}

pub fn render_one(text: &str) -> Rendered {
    render_docs(&[("main.tex", text)], "main.tex")
}

pub fn v1_of(r: &Rendered, caps: Capabilities) -> V1Payload {
    let accepted = {
        let mut a = Vec::new();
        if caps.rules {
            a.push(v1::CAP_RULES.to_string());
        }
        if caps.font_hints {
            a.push(v1::CAP_FONT_HINTS.to_string());
        }
        Some(a)
    };
    v1::fallback(&r.v2, caps, accepted)
}

/// One v2 glyph run as a word box: page, text, left edge, baseline and
/// width in bp (a math formula appears as one run per glyph).
#[derive(Debug, Clone)]
pub struct Word {
    pub page: u32,
    pub text: String,
    pub x: f64,
    pub baseline: f64,
    pub width: f64,
}

pub fn words_of(r: &Rendered) -> Vec<Word> {
    let mut words = Vec::new();
    for page in &r.v2.pages {
        for it in &page.items {
            if let flashtex_render_pipeline::display::Item::GlyphRun(run) = it {
                let Some(first) = run.glyphs.first() else { continue };
                let last = run.glyphs.last().expect("non-empty");
                words.push(Word {
                    page: page.number,
                    text: run.text.clone(),
                    x: first.origin_x.to_bp(),
                    baseline: first.baseline_y.to_bp(),
                    width: (last.origin_x.0 + last.advance_x.0 - first.origin_x.0) as f64 / flashtex_render_pipeline::display::TICKS_PER_BP,
                });
            }
        }
    }
    words
}
