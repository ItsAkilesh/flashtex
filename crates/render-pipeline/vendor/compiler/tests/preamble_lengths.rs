//! `\documentclass[..pt]` body size and preamble `\setlength{\parskip|\parindent}`.
use flashtex_compiler::incremental::compile_full_project;
use flashtex_compiler::layout::{LayoutConstraints, Page, PARAGRAPH_GAP_PT};
use flashtex_compiler::parser::{parse, SourceDocument};

fn compile(text: &str) -> (Vec<Page>, Vec<String>) {
    let out = compile_full_project(
        &[SourceDocument {
            path: "main.tex",
            text,
        }],
        "main.tex",
        LayoutConstraints::default(),
    );
    let messages = out.diagnostics.into_iter().map(|d| d.message).collect();
    (out.pages, messages)
}

fn baseline_gap(text: &str) -> f64 {
    let (pages, _) = compile(text);
    let y = |word: &str| {
        pages[0]
            .items
            .iter()
            .find(|item| item.text == word)
            .map(|item| item.baseline_y_pt)
            .unwrap()
    };
    y("Two") - y("One")
}

#[test]
fn class_option_sets_the_body_size() {
    let parsed = parse("\\documentclass[11pt]{article}\\begin{document}x\\end{document}");
    assert_eq!(parsed.class_size_pt, Some(11.0));
    let (pages, _) =
        compile("\\documentclass[a4paper,10pt]{article}\\begin{document}Body\\end{document}");
    assert_eq!(pages[0].items[0].font_size_pt, 10.0);
    let (pages, _) = compile("\\documentclass{article}\\begin{document}Body\\end{document}");
    assert_eq!(
        pages[0].items[0].font_size_pt,
        LayoutConstraints::default().font_size_pt,
        "no size option keeps the existing default"
    );
}

#[test]
fn parskip_replaces_the_paragraph_gap_with_em_relative_to_the_class_size() {
    let doc = |preamble: &str| {
        format!("\\documentclass[11pt]{{article}}{preamble}\\begin{{document}}One\n\nTwo\\end{{document}}")
    };
    let default_gap = baseline_gap(&doc(""));
    let parskip_gap = baseline_gap(&doc("\\setlength{\\parskip}{0.65em}"));
    let expected = 0.65 * 11.0 - PARAGRAPH_GAP_PT;
    assert!(
        (parskip_gap - default_gap - expected).abs() < 0.02,
        "gap grew by {} not {expected}",
        parskip_gap - default_gap
    );
    let (_, messages) = compile(&doc(
        "\\setlength{\\parskip}{0.65em}\\setlength{\\parindent}{0pt}",
    ));
    assert!(
        !messages
            .iter()
            .any(|m| m.contains("setlength") || m.contains("parindent")),
        "{messages:?}"
    );
}

#[test]
fn unimplemented_lengths_are_reported_not_silently_ignored() {
    let (_, messages) = compile(
        "\\documentclass{article}\\setlength{\\parindent}{15pt}\\setlength{\\textwidth}{5in}\
         \\begin{document}x\\setlength{\\parskip}{1em}\\setlength{\\parskip}{banana}\\end{document}",
    );
    for expected in [
        "\\parindent is recognised but paragraph indentation is not implemented",
        "\\setlength{\\textwidth} is recognised but not implemented here",
        "\\setlength{\\parskip} is recognised but not implemented here",
        "\\setlength requires a recognised dimension, got 'banana'",
    ] {
        assert!(
            messages.iter().any(|m| m == expected),
            "missing {expected:?} in {messages:?}"
        );
    }
}

#[test]
fn hw1_keeps_its_three_reference_pages_with_parskip_applied() {
    let hw1 = include_str!("../../../fixtures/real-world/hw1/HW1.tex");
    let parsed = parse(hw1);
    assert_eq!(parsed.class_size_pt, Some(11.0));
    assert_eq!(parsed.parskip_pt, Some(0.65 * 11.0));
    let (pages, messages) = compile(hw1);
    assert_eq!(pages.len(), 3, "HW1-reference.pdf has 3 pages");
    assert!(
        !messages.iter().any(|m| m.contains("setlength")),
        "{messages:?}"
    );
}
