//! HW2 gate (#71): the reference page count and the known-gap diagnostics.
use flashtex_compiler::incremental::compile_full_project;
use flashtex_compiler::layout::LayoutConstraints;
use flashtex_compiler::parser::SourceDocument;

const HW2: &str = include_str!("../../../fixtures/real-world/hw2/HW2.tex");

#[test]
fn hw2_matches_the_reference_page_count() {
    let out = compile_full_project(
        &[SourceDocument {
            path: "main.tex",
            text: HW2,
        }],
        "main.tex",
        LayoutConstraints::default(),
    );
    assert_eq!(out.pages.len(), 3, "HW2-reference.pdf has 3 pages");
    // Remaining known gaps, each owned by another lane: math symbols (#62),
    // amsthm (#55), microtype, enumitem leftmargin=*, and PDF-export notes.
    let unsupported: Vec<String> = out
        .diagnostics
        .iter()
        .map(|d| d.message.clone())
        .filter(|m| m.contains("not supported"))
        .collect();
    let known = [
        "\\subsetneq",
        "\\Longleftrightarrow",
        "\\mathbin",
        "\\triangle",
        "\\mathcal",
        "\\longrightarrow",
    ];
    assert!(
        unsupported
            .iter()
            .all(|m| known.iter().any(|k| m.starts_with(k))),
        "{unsupported:?}"
    );
}
