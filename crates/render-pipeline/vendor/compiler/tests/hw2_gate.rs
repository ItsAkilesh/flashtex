//! HW2 gate (FT-060, #71): the user's real-world HW2 compiles with no
//! "not supported" diagnostics.
use flashtex_compiler::incremental::compile_full_project;
use flashtex_compiler::layout::LayoutConstraints;
use flashtex_compiler::parser::SourceDocument;

const HW2: &str = include_str!("../../../fixtures/real-world/hw2/HW2.tex");

#[test]
fn hw2_has_no_unsupported_diagnostics() {
    let out = compile_full_project(
        &[SourceDocument {
            path: "main.tex",
            text: HW2,
        }],
        "main.tex",
        LayoutConstraints::default(),
    );
    for d in &out.diagnostics {
        eprintln!("{:?}", d.message);
    }
    let unsupported: Vec<&str> = out
        .diagnostics
        .iter()
        .map(|d| d.message.as_str())
        .filter(|m| m.contains("not supported"))
        .collect();
    assert!(unsupported.is_empty(), "{unsupported:?}");
}
