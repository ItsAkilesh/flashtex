use flashtex_project_index::{Category, IndexError, ProjectIndex, SymbolKind};

fn index(source: &str) -> ProjectIndex {
    let mut index = ProjectIndex::new("project-a").unwrap();
    index.replace_document("main.tex", 1, source).unwrap();
    index
}

#[test]
fn unicode_keys_have_exact_utf8_ranges_and_cross_file_navigation() {
    let root = "東京 \\label{sec:αβ}\n";
    let child = "π \\ref{sec:αβ}";
    let mut project = index(root);
    project
        .replace_document("chapters/東京.tex", 7, child)
        .unwrap();
    let snapshot = project.snapshot();
    let definitions = project
        .definitions(&snapshot, Category::Label, "sec:αβ")
        .unwrap();
    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0].source.file, "main.tex");
    assert_eq!(
        &root[definitions[0].source.start_byte..definitions[0].source.end_byte],
        "sec:αβ"
    );
    let hit = project
        .navigate(&snapshot, "chapters/東京.tex", child.find("sec:").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(hit.origin.kind, SymbolKind::LabelReference);
    assert_eq!(hit.origin.source.revision, 7);
    assert_eq!(hit.definitions, definitions);
    for symbol in project.symbols(&snapshot).unwrap() {
        let text = if symbol.source.file == "main.tex" {
            root
        } else {
            child
        };
        assert!(text.is_char_boundary(symbol.source.start_byte));
        assert!(text.is_char_boundary(symbol.source.end_byte));
        assert_eq!(
            &text[symbol.source.start_byte..symbol.source.end_byte],
            symbol.name
        );
    }
}

#[test]
fn comments_escaped_percent_and_double_backslash_do_not_make_phantom_labels() {
    let project = index("% \\label{comment}\n\\% \\label{real} \\\\label{not-a-command}\n");
    let snapshot = project.snapshot();
    assert!(project
        .definitions(&snapshot, Category::Label, "comment")
        .unwrap()
        .is_empty());
    assert!(project
        .definitions(&snapshot, Category::Label, "not-a-command")
        .unwrap()
        .is_empty());
    assert_eq!(
        project
            .definitions(&snapshot, Category::Label, "real")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn verb_and_verbatim_bodies_are_not_indexed() {
    let project = index(
        r"\verb|\label{hidden}| \verb*+\cite{hidden}+ \begin{verbatim}
\label{also-hidden} \newcommand{\phantom}{x}
\end{verbatim} \label{visible}",
    );
    let snapshot = project.snapshot();
    assert!(project
        .occurrences(&snapshot, Category::Label, "hidden")
        .unwrap()
        .is_empty());
    assert!(project
        .occurrences(&snapshot, Category::Citation, "hidden")
        .unwrap()
        .is_empty());
    assert!(project
        .occurrences(&snapshot, Category::Command, "phantom")
        .unwrap()
        .is_empty());
    assert_eq!(
        project
            .definitions(&snapshot, Category::Label, "visible")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn malformed_verb_recovers_on_the_next_line() {
    let project = index("\\verb|unclosed\n\\label{next}\n");
    let snapshot = project.snapshot();
    assert_eq!(
        project
            .definitions(&snapshot, Category::Label, "next")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(project.diagnostics(&snapshot).unwrap().len(), 1);
}

#[test]
fn citations_optional_arguments_lists_and_bibitem_targets() {
    let source = r"\bibitem[Author {[}]{ paper:α } Entry.
\citep*[see {section]}][p. 4]{paper:α, paper:β} \nocite{*}";
    let project = index(source);
    let snapshot = project.snapshot();
    let references = project
        .occurrences(&snapshot, Category::Citation, "paper:α")
        .unwrap();
    assert_eq!(references.len(), 2);
    assert_eq!(references[0].kind, SymbolKind::CitationDefinition);
    assert_eq!(references[1].kind, SymbolKind::CitationReference);
    assert_eq!(
        project
            .occurrences(&snapshot, Category::Citation, "paper:β")
            .unwrap()
            .len(),
        1
    );
    assert!(project
        .occurrences(&snapshot, Category::Citation, "*")
        .unwrap()
        .is_empty());
    for symbol in references {
        assert_eq!(
            &source[symbol.source.start_byte..symbol.source.end_byte],
            "paper:α"
        );
    }
}

#[test]
fn reference_lists_and_literal_whitespace_keep_each_name_range() {
    let project = index(r"\label{a}\label{b}\cref{ a, b } \eqref{a} \pageref{b}");
    let snapshot = project.snapshot();
    assert_eq!(
        project
            .occurrences(&snapshot, Category::Label, "a")
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        project
            .occurrences(&snapshot, Category::Label, "b")
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn command_declarations_and_uses_are_separate_without_duplicate_target_use() {
    let project = index(
        r"\newcommand*{\hello}[1]{Hi #1}\hello{x}\renewcommand\hello{y}\def\other#1{\hello{#1}}\other{z}",
    );
    let snapshot = project.snapshot();
    assert_eq!(
        project
            .definitions(&snapshot, Category::Command, "hello")
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        project
            .occurrences(&snapshot, Category::Command, "hello")
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        project
            .definitions(&snapshot, Category::Command, "other")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        project
            .occurrences(&snapshot, Category::Command, "other")
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn command_definition_comment_and_space_trivia_preserve_ranges() {
    let source = "\\providecommand % target follows\n { \\foo % comment\n }{x} \\foo";
    let project = index(source);
    let definitions = project
        .definitions(&project.snapshot(), Category::Command, "foo")
        .unwrap();
    assert_eq!(definitions.len(), 1);
    assert_eq!(
        &source[definitions[0].source.start_byte..definitions[0].source.end_byte],
        "foo"
    );
}

#[test]
fn replacement_removes_old_symbols_and_invalidates_all_old_queries() {
    let mut project = index(r"\label{old}");
    let old = project.snapshot();
    project
        .replace_document("main.tex", 2, r"\label{new}")
        .unwrap();
    assert_eq!(project.symbols(&old), Err(IndexError::StaleSnapshot));
    assert_eq!(
        project.complete(&old, Category::Label, "", 0),
        Err(IndexError::StaleSnapshot)
    );
    assert_eq!(
        project.navigate(&old, "main.tex", 0),
        Err(IndexError::StaleSnapshot)
    );
    assert_eq!(project.diagnostics(&old), Err(IndexError::StaleSnapshot));
    let current = project.snapshot();
    assert!(project
        .definitions(&current, Category::Label, "old")
        .unwrap()
        .is_empty());
    assert_eq!(
        project
            .definitions(&current, Category::Label, "new")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn per_document_revisions_and_rejected_update_are_atomic() {
    let mut project = index(r"\label{keep}");
    project
        .replace_document("other.tex", 100, r"\label{other}")
        .unwrap();
    let snapshot = project.snapshot();
    for revision in [0, 1] {
        assert!(matches!(
            project.replace_document("main.tex", revision, r"\label{bad}"),
            Err(IndexError::StaleDocument { .. })
        ));
        assert_eq!(project.snapshot(), snapshot);
    }
    project
        .replace_document("main.tex", 2, r"\label{updated}")
        .unwrap();
    assert_eq!(project.snapshot().documents["other.tex"], 100);
    assert_eq!(project.snapshot().documents["main.tex"], 2);
}

#[test]
fn deletion_retains_revision_tombstone_and_stale_snapshot_rejection() {
    let mut project = index(r"\label{gone}");
    let old = project.snapshot();
    project.remove_document("main.tex", 2).unwrap();
    assert_eq!(project.symbols(&old), Err(IndexError::StaleSnapshot));
    assert!(project.symbols(&project.snapshot()).unwrap().is_empty());
    assert!(matches!(
        project.replace_document("main.tex", 2, r"\label{stale}"),
        Err(IndexError::StaleDocument { .. })
    ));
    project
        .replace_document("main.tex", 3, r"\label{fresh}")
        .unwrap();
    assert_eq!(
        project
            .definitions(&project.snapshot(), Category::Label, "fresh")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn forged_snapshot_vector_and_other_project_are_rejected() {
    let project = index("hello");
    let mut wrong = project.snapshot();
    wrong.documents.insert("missing.tex".into(), 0);
    assert_eq!(project.symbols(&wrong), Err(IndexError::StaleSnapshot));
    wrong = project.snapshot();
    wrong.project_id = "another-project".into();
    assert_eq!(project.symbols(&wrong), Err(IndexError::WrongProject));
}

#[test]
fn navigation_rejects_non_utf8_boundary_and_out_of_bounds() {
    let project = index("東京 \\label{東京}");
    let snapshot = project.snapshot();
    assert_eq!(
        project.navigate(&snapshot, "main.tex", 1),
        Err(IndexError::InvalidOffset)
    );
    assert_eq!(
        project.navigate(&snapshot, "main.tex", usize::MAX),
        Err(IndexError::InvalidOffset)
    );
    assert_eq!(
        project.navigate(&snapshot, "absent.tex", 0),
        Err(IndexError::MissingDocument)
    );
    assert_eq!(project.navigate(&snapshot, "main.tex", 0), Ok(None));
}

#[test]
fn traversal_absolute_windows_and_alias_paths_are_rejected_without_changes() {
    let mut project = index("keep");
    let snapshot = project.snapshot();
    for file in [
        "",
        "/main.tex",
        "../main.tex",
        "a/../main.tex",
        "./main.tex",
        "a//b.tex",
        "a/",
        "C:main.tex",
        "a\\b.tex",
        "bad\0.tex",
    ] {
        assert_eq!(
            project.replace_document(file, 2, "bad"),
            Err(IndexError::InvalidPath),
            "{file:?}"
        );
        assert_eq!(project.snapshot(), snapshot);
    }
}

#[test]
fn prefix_completion_is_sorted_deduplicated_limited_and_retains_definitions() {
    let mut project =
        index(r"\label{sec:z}\label{sec:a}\ref{sec:a}\cite{sec:c}\newcommand{\Section}{}");
    project
        .replace_document("part.tex", 0, r"\label{sec:a}\ref{sec:missing}")
        .unwrap();
    let snapshot = project.snapshot();
    let results = project
        .complete(&snapshot, Category::Label, "sec:", 2)
        .unwrap();
    assert_eq!(
        results.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        ["sec:a", "sec:missing"]
    );
    assert_eq!(results[0].definitions.len(), 2);
    assert_eq!(results[0].occurrences.len(), 3);
    assert!(results[1].definitions.is_empty());
    assert!(project
        .complete(&snapshot, Category::Command, "section", 10)
        .unwrap()
        .is_empty());
    assert_eq!(
        project
            .complete(&snapshot, Category::Command, "Sec", 10)
            .unwrap()[0]
            .name,
        "Section"
    );
    assert!(project
        .complete(&snapshot, Category::Label, "", 0)
        .unwrap()
        .is_empty());
}

#[test]
fn malformed_and_dynamic_arguments_produce_diagnostics_without_invented_targets() {
    let project = index("\\label{\\dynamic}\n\\label{}\n\\label{unclosed\n\\label{good}");
    let snapshot = project.snapshot();
    assert_eq!(
        project
            .definitions(&snapshot, Category::Label, "good")
            .unwrap()
            .len(),
        1
    );
    assert!(project
        .definitions(&snapshot, Category::Label, "\\dynamic")
        .unwrap()
        .is_empty());
    assert_eq!(project.diagnostics(&snapshot).unwrap().len(), 3);
}

#[test]
fn query_order_is_file_then_byte_and_independent_of_insertion_order() {
    let mut first = ProjectIndex::new("p").unwrap();
    let mut second = ProjectIndex::new("p").unwrap();
    first.replace_document("z.tex", 0, r"\label{z}").unwrap();
    first.replace_document("a.tex", 0, r"\label{a}").unwrap();
    second.replace_document("a.tex", 0, r"\label{a}").unwrap();
    second.replace_document("z.tex", 0, r"\label{z}").unwrap();
    assert_eq!(
        first.symbols(&first.snapshot()).unwrap(),
        second.symbols(&second.snapshot()).unwrap()
    );
}

#[test]
fn retained_source_range_is_revalidated_even_with_a_fresh_snapshot() {
    let mut project = index(r"\label{東京}");
    let symbol = project
        .definitions(&project.snapshot(), Category::Label, "東京")
        .unwrap()
        .remove(0);
    assert_eq!(
        project.source_text(&project.snapshot(), &symbol.source),
        Ok("東京")
    );
    let mut broken = symbol.source.clone();
    broken.start_byte += 1;
    assert_eq!(
        project.source_text(&project.snapshot(), &broken),
        Err(IndexError::InvalidOffset)
    );
    broken.start_byte = symbol.source.end_byte + 1;
    assert_eq!(
        project.source_text(&project.snapshot(), &broken),
        Err(IndexError::InvalidOffset)
    );
    project
        .replace_document("main.tex", 2, "shifted \\label{東京}")
        .unwrap();
    assert_eq!(
        project.source_text(&project.snapshot(), &symbol.source),
        Err(IndexError::StaleSourceRange {
            current_revision: 2,
            source_revision: 1
        })
    );
}

#[test]
fn all_generated_lexical_spans_remain_valid_for_malformed_unicode_sources() {
    let pieces = [
        "東京",
        "α",
        "\\label{a}",
        "\\cite[x]{a,β}",
        "{",
        "}",
        "%x\n",
        "\\",
        "\\verb|",
        "\\newcommand{\\x}",
        "[",
        "]",
    ];
    for seed in 0..200 {
        let mut state = seed as u32 + 1;
        let mut text = String::new();
        for _ in 0..40 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            text.push_str(pieces[state as usize % pieces.len()]);
        }
        let project = index(&text);
        let snapshot = project.snapshot();
        for symbol in project.symbols(&snapshot).unwrap() {
            assert_eq!(
                project.source_text(&snapshot, &symbol.source).unwrap(),
                symbol.name
            );
        }
        for diagnostic in project.diagnostics(&snapshot).unwrap() {
            project.source_text(&snapshot, &diagnostic.source).unwrap();
        }
    }
}
