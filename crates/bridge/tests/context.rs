use flashtex_bridge::{context, Document, MAX_CONTEXT_BYTES};
fn doc(path: &str, text: &str) -> Document {
    Document {
        project_id: "p".into(),
        path: path.into(),
        revision: 7,
        text: text.into(),
    }
}
fn build(files: &[Document], target: usize) -> flashtex_bridge::Context {
    let d = &files[target];
    context::build(d, d.text.len(), d.text.len(), files.iter(), vec![]).unwrap()
}
#[test]
fn multiline_complete_declarations_and_provenance() {
    let files=[doc("main.tex", "\\usepackage[foo]{amsmath}\n\\newcommand{\\vect}[1]{\n  \\mathbf{#1}% comment }\n}\n\\DeclareMathOperator{\\argmin}{arg min}\n")];
    let c = build(&files, 0);
    let s = c.definitions.join("\n");
    assert!(s.contains("\\mathbf{#1}% comment }\n}"));
    assert!(s.contains("snapshot main.tex revision 7 line 2"));
    assert!(s.contains("{\\argmin}{arg min}"));
}
#[test]
fn resolves_uploaded_parent_preamble_and_cycles_but_excludes_unrelated() {
    let mut alien = doc("alien.tex", "\\newcommand{\\secret}{private}");
    alien.project_id = "other".into();
    let files = [
        doc("main.tex", "\\input{preamble}\n\\include{chapters/one}"),
        doc(
            "preamble.tex",
            "\\newcommand{\\R}{\\mathbb{R}}\n\\input{main}",
        ),
        doc("chapters/one.tex", "Hello"),
        doc("unrelated.tex", "\\def\\nope{unrelated}"),
        alien,
    ];
    let c = build(&files, 2);
    let s = c.definitions.join("\n");
    assert!(s.contains("\\mathbb{R}"));
    assert!(!s.contains("private"));
    assert!(!s.contains("unrelated}"));
}
#[test]
fn comments_escaped_commands_and_verbatim_are_not_declarations_or_edges() {
    let files=[doc("main.tex", "% \\input{secret}\n\\\\newcommand{\\fake}{bad}\n\\verb|\\input{secret}|\n\\begin{verbatim}\n\\def\\fake{bad}\n\\input{secret}\n\\end{verbatim}\n\\newcommand{\\ok}{good}"),doc("secret.tex","\\def\\secret{bad}")];
    let s = build(&files, 0).definitions.join("\n");
    assert!(s.contains("good"));
    assert!(!s.contains("bad"));
}
#[test]
fn malformed_body_not_silently_truncated() {
    let files = [doc("main.tex", "\\newcommand{\\broken}{never closes\n")];
    assert_eq!(build(&files, 0).definitions.len(), 1);
}
#[test]
fn unicode_control_symbol_does_not_panic_and_windows_are_bounded() {
    let text = format!("\\α \\newcommand{{\\x}}{{😀}}{}", "α😀".repeat(6000));
    let files = [doc("main.tex", &text)];
    let start = text.char_indices().nth(4000).unwrap().0;
    let end = text.char_indices().nth(5000).unwrap().0;
    let c = context::build(&files[0], start, end, files.iter(), vec![]).unwrap();
    let total = c.source_before.len()
        + c.source_after.len()
        + c.selected_source.len()
        + c.definitions.iter().map(String::len).sum::<usize>();
    assert!(total <= MAX_CONTEXT_BYTES);
    assert!(c.definitions.iter().any(|s| s.contains("{😀}")));
}
#[test]
fn relative_include_fallback_and_traversal_rejection() {
    let files = [
        doc("chapters/main.tex", "\\input{defs}\n\\input{../secret}"),
        doc("chapters/defs.tex", "\\def\\ok{good}"),
        doc("secret.tex", "\\def\\secret{bad}"),
    ];
    let s = build(&files, 0).definitions.join("\n");
    assert!(s.contains("good"));
    assert!(!s.contains("bad"));
}
#[test]
fn selection_limit_and_feature_limit_are_preserved() {
    let d = doc("main.tex", &"a".repeat(9000));
    assert!(context::build(&d, 0, 9000, std::iter::once(&d), vec![]).is_err());
    assert!(context::build(&d, 0, 0, std::iter::once(&d), vec!["x".into(); 65]).is_err());
}
#[test]
fn omission_is_explicit_and_never_clips_declaration() {
    let text = format!(
        "\\newcommand{{\\huge}}{{{}}}\\newcommand{{\\small}}{{ok}}",
        "x".repeat(20000)
    );
    let files = [doc("main.tex", &text)];
    let c = build(&files, 0);
    assert!(c.definitions[0].contains("Context incomplete"));
    assert!(!c.definitions.iter().any(|s| s.contains("\\huge")));
    assert!(c.definitions.iter().any(|s| s.contains("\\small")));
}
#[test]
fn parent_revision_change_changes_context_even_when_child_unchanged() {
    let mut files = [
        doc("main.tex", "\\input{child}\\def\\x{old}"),
        doc("child.tex", "text"),
    ];
    let old = build(&files, 1);
    files[0].revision += 1;
    files[0].text = "\\input{child}\\def\\x{new}".into();
    let new = build(&files, 1);
    assert_ne!(old.definitions, new.definitions);
    assert_eq!(old.revision, new.revision);
}
