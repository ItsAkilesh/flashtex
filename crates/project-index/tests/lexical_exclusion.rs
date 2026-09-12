use flashtex_project_index::{Category, IndexError, ProjectIndex, MAX_DOCUMENT_BYTES};

#[test]
fn known_literal_environments_never_add_label_citation_or_rename_targets() {
    for environment in [
        "verbatim",
        "verbatim*",
        "Verbatim",
        "Verbatim*",
        "BVerbatim",
        "LVerbatim",
        "lstlisting",
        "minted",
        "comment",
        "filecontents",
        "filecontents*",
    ] {
        let text = format!("\\label{{real}}\\ref{{real}}\n\\begin{{{environment}}}[language=TeX]{{tex}}\n\\label{{real}}\\ref{{real}}\\cite{{paper}}\\newcommand{{\\fake}}{{x}}\n\\end{{{environment}}}\n\\ref{{real}}");
        let mut index = ProjectIndex::new("literal").unwrap();
        index.replace_document("main.tex", 0, &text).unwrap();
        let view = index.snapshot();
        assert_eq!(
            index
                .occurrences(&view, Category::Label, "real")
                .unwrap()
                .len(),
            3,
            "{environment}"
        );
        assert!(
            index
                .occurrences(&view, Category::Citation, "paper")
                .unwrap()
                .is_empty(),
            "{environment}"
        );
        assert!(index
            .occurrences(&view, Category::Command, "fake")
            .unwrap()
            .is_empty());
        assert_eq!(
            index
                .plan_label_rename(&view, "real", "new")
                .unwrap()
                .edits
                .len(),
            3,
            "{environment}"
        );
    }
}

#[test]
fn escaped_literal_end_tokens_do_not_expose_body_names() {
    let text = r"\label{real}\begin{lstlisting}
\\end{lstlisting} \label{fake}\cite{fake}
\end{lstlisting}\ref{real}";
    let mut index = ProjectIndex::new("literal").unwrap();
    index.replace_document("main.tex", 0, text).unwrap();
    let view = index.snapshot();
    assert!(index
        .occurrences(&view, Category::Label, "fake")
        .unwrap()
        .is_empty());
    assert!(index
        .occurrences(&view, Category::Citation, "fake")
        .unwrap()
        .is_empty());
    assert_eq!(
        index
            .plan_label_rename(&view, "real", "new")
            .unwrap()
            .edits
            .len(),
        2
    );
}

#[test]
fn inline_code_delimiters_and_braced_minted_body_are_excluded() {
    for code in [
        r"\verb|\label{fake}\cite{fake}|",
        r"\verb*+\label{fake}+",
        r"\lstinline[language=TeX]!\label{fake}!",
        r"\lstinline*|\cite{fake}|",
        r"\mintinline[breaklines]{tex}|\label{fake}|",
        r"\mintinline{tex}{\label{fake} % literal percent }",
    ] {
        let mut index = ProjectIndex::new("literal").unwrap();
        index
            .replace_document("main.tex", 0, &format!("{code} \\label{{real}}"))
            .unwrap();
        let view = index.snapshot();
        assert!(
            index
                .occurrences(&view, Category::Label, "fake")
                .unwrap()
                .is_empty(),
            "{code}"
        );
        assert!(
            index
                .occurrences(&view, Category::Citation, "fake")
                .unwrap()
                .is_empty(),
            "{code}"
        );
        assert_eq!(
            index
                .definitions(&view, Category::Label, "real")
                .unwrap()
                .len(),
            1,
            "{code}"
        );
    }
}

#[test]
fn odd_even_backslashes_control_command_and_comment_recognition() {
    for count in 0..9 {
        let mut index = ProjectIndex::new("escapes").unwrap();
        let text = format!("{}label{{key}}", "\\".repeat(count));
        index.replace_document("main.tex", 0, &text).unwrap();
        assert_eq!(
            index
                .definitions(&index.snapshot(), Category::Label, "key")
                .unwrap()
                .len(),
            count % 2
        );
        let text = format!("{}% \\label{{key}}\n", "\\".repeat(count));
        index.replace_document("main.tex", 1, &text).unwrap();
        assert_eq!(
            index
                .definitions(&index.snapshot(), Category::Label, "key")
                .unwrap()
                .len(),
            count % 2
        );
    }
}

#[test]
fn escaped_begin_is_not_misinterpreted_as_an_environment() {
    let mut index = ProjectIndex::new("escapes").unwrap();
    index
        .replace_document("main.tex", 0, r"\\begin{verbatim}\label{real}")
        .unwrap();
    assert_eq!(
        index
            .definitions(&index.snapshot(), Category::Label, "real")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn malformed_inline_recovers_next_line_but_unclosed_environment_excludes_tail() {
    let mut index = ProjectIndex::new("recovery").unwrap();
    index
        .replace_document(
            "main.tex",
            0,
            "\\lstinline|bad \\label{fake}\n\\label{good}\n\\begin{minted}{tex}\n\\label{tail}",
        )
        .unwrap();
    let view = index.snapshot();
    assert!(index
        .definitions(&view, Category::Label, "fake")
        .unwrap()
        .is_empty());
    assert!(index
        .definitions(&view, Category::Label, "tail")
        .unwrap()
        .is_empty());
    assert_eq!(
        index
            .definitions(&view, Category::Label, "good")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(index.diagnostics(&view).unwrap().len(), 2);
}

#[test]
fn document_and_group_bounds_fail_without_silent_mutation_or_panics() {
    let mut index = ProjectIndex::new("bounds").unwrap();
    index
        .replace_document("main.tex", 0, r"\label{keep}")
        .unwrap();
    let view = index.snapshot();
    assert!(matches!(
        index.replace_document("main.tex", 1, &"x".repeat(MAX_DOCUMENT_BYTES + 1)),
        Err(IndexError::DocumentTooLarge { .. })
    ));
    assert_eq!(view, index.snapshot());
    let text = format!(
        "\\label{{{}x{}}}\n\\label{{good}}",
        "{".repeat(150),
        "}".repeat(150)
    );
    index.replace_document("main.tex", 1, &text).unwrap();
    assert_eq!(
        index
            .definitions(&index.snapshot(), Category::Label, "good")
            .unwrap()
            .len(),
        1
    );
    assert!(!index.diagnostics(&index.snapshot()).unwrap().is_empty());
}
