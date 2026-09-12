//! A paragraph ending in `\\` used to panic inside the pinned
//! paragraph-layout (`vendor/paragraph-layout/src/linebreak.rs:988`, slice
//! index on the empty final line TeX would set) and kill the worker. The
//! pipeline refuses such a paragraph with a typed diagnostic instead; the
//! vendored crate is not patched here (owner report in the handoff).

mod common;

use common::*;
use flashtex_render_pipeline::display::Severity;

fn body(text: &str) -> String {
    format!("\\documentclass[12pt]{{article}}\n\\begin{{document}}\n{text}\n\\end{{document}}\n")
}

#[test]
fn trailing_linebreak_is_a_typed_diagnostic_not_a_panic() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    for text in ["Alpha beta\\\\\n\nNext paragraph.", "Alpha beta\\\\\n\n", "Alpha beta\\\\ \n\nNext.", "Alpha $x$\\\\\n", "Alpha\\\\\\\\\n\nNext."] {
        let r = render_one(&body(text));
        let d = r
            .v2
            .diagnostics
            .iter()
            .find(|d| d.code == "paragraph_final_linebreak")
            .unwrap_or_else(|| panic!("{text:?}: {:?}", r.v2.diagnostics));
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.sources.len(), 1, "{text:?}: the diagnostic points at the text before the break");
        let src = &d.sources[0];
        let doc = body(text);
        let pointed = &doc[src.start_byte..src.end_byte];
        assert!(text.contains(pointed) && !pointed.is_empty(), "{text:?} -> {pointed:?}");
        // The refused paragraph is absent; a following paragraph is still set.
        let texts: Vec<String> = r.v2.pages.iter().flat_map(|p| p.items.iter()).filter_map(|i| match i {
            flashtex_render_pipeline::display::Item::GlyphRun(g) => Some(g.text.clone()),
            _ => None,
        }).collect();
        assert!(!texts.iter().any(|t| t == "Alpha"), "{text:?}: {texts:?}");
        if text.contains("Next") {
            assert!(texts.iter().any(|t| t.starts_with("Next")), "{text:?}: {texts:?}");
        }
    }
}

#[test]
fn a_linebreak_inside_a_paragraph_is_still_typeset() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    let r = render_one(&body("Alpha\\\\beta\n\nNext."));
    assert!(!r.v2.diagnostics.iter().any(|d| d.code == "paragraph_final_linebreak"), "{:?}", r.v2.diagnostics);
    let texts: Vec<String> = r.v2.pages.iter().flat_map(|p| p.items.iter()).filter_map(|i| match i {
        flashtex_render_pipeline::display::Item::GlyphRun(g) => Some(g.text.clone()),
        _ => None,
    }).collect();
    assert!(texts.iter().any(|t| t == "Alpha") && texts.iter().any(|t| t == "beta"), "{texts:?}");
}
