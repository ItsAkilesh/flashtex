#![allow(dead_code)]
use flashtex_compiler::parser::SourceDocument;
use flashtex_render_pipeline::v1::{self, Capabilities, V1Payload};
use flashtex_render_pipeline::{render, FontSet, RenderOptions, Rendered};

pub fn lm_available() -> bool {
    flashtex_render_pipeline::fonts::DEFAULT_FONT_DIRS
        .iter()
        .any(|d| std::path::Path::new(d).join("lmroman12-regular.otf").is_file())
        && flashtex_render_pipeline::fonts::DEFAULT_FONT_DIRS
            .iter()
            .any(|d| std::path::Path::new(d).join("latinmodern-math.otf").is_file())
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
