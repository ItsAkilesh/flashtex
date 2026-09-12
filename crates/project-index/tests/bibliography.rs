use flashtex_project_index::{Category, DocumentKind, IndexError, ProjectIndex};

#[test]
fn unresolved_citation_navigates_to_declared_unicode_bibliography_key() {
    let text = "東京 \\cite{paper:α,missing}";
    let bib = "% @article{comment,}\n@Article{paper:α, title={A title}}";
    let mut index = ProjectIndex::new("bibliography").unwrap();
    index.replace_document("main.tex", 1, text).unwrap();
    let before = index.snapshot();
    assert_eq!(index.unresolved_references(&before).unwrap().len(), 2);
    assert!(index
        .navigate(&before, "main.tex", text.find("paper:α").unwrap())
        .unwrap()
        .unwrap()
        .definitions
        .is_empty());
    let update = index
        .replace_bibliography_document("refs/東京.bib", 7, bib)
        .unwrap();
    assert_eq!(update.metrics.reference_documents_rechecked, 2);
    assert_eq!(
        index.unresolved_references(&before),
        Err(IndexError::StaleSnapshot)
    );
    let view = index.snapshot();
    let hit = index
        .navigate(&view, "main.tex", text.find("paper:α").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(hit.definitions.len(), 1);
    assert_eq!(hit.definitions[0].source.file, "refs/東京.bib");
    assert_eq!(hit.definitions[0].source.revision, 7);
    assert_eq!(
        index.source_text(&view, &hit.definitions[0].source),
        Ok("paper:α")
    );
    assert_eq!(
        index.unresolved_references(&view).unwrap()[0].name,
        "missing"
    );
    assert!(index
        .definitions(&view, Category::Citation, "comment")
        .unwrap()
        .is_empty());
}

#[test]
fn document_kind_is_explicit_and_kind_replacement_drops_old_symbols() {
    let mut index = ProjectIndex::new("bibliography").unwrap();
    let text = r"@book{key, title={\label{fake}}}";
    index.replace_document("not-inferred.bib", 0, text).unwrap();
    let old = index.snapshot();
    assert_eq!(
        index.document_kind(&old, "not-inferred.bib"),
        Ok(DocumentKind::Latex)
    );
    assert!(index
        .definitions(&old, Category::Citation, "key")
        .unwrap()
        .is_empty());
    index
        .replace_bibliography_document("not-inferred.bib", 1, text)
        .unwrap();
    let view = index.snapshot();
    assert_eq!(
        index.document_kind(&old, "not-inferred.bib"),
        Err(IndexError::StaleSnapshot)
    );
    assert_eq!(
        index.document_kind(&view, "not-inferred.bib"),
        Ok(DocumentKind::Bibliography)
    );
    assert!(index
        .definitions(&view, Category::Label, "fake")
        .unwrap()
        .is_empty());
    assert_eq!(
        index.plan_label_rename(&view, "fake", "new"),
        Err(IndexError::MissingLabelDefinition)
    );
    index
        .replace_bibliography_document("also-valid.data", 0, "@misc{second,}")
        .unwrap();
    assert_eq!(
        index
            .definitions(&index.snapshot(), Category::Citation, "second")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn metadata_nested_fields_and_quoted_entry_text_never_make_fake_keys() {
    let text = r#"@comment{arbitrary " text @article{fake-comment,}}
@string{publisher = "@book{fake-string,}"}
@preamble{"@book{fake-preamble,}"}
@article(real,
  title = "Quoted \" @book{fake-quoted,} text",
  note = {Nested {braces} @book{fake-braced,} \label{fake-label}}
)
@misc{next, title = {Done}}
"#;
    let mut index = ProjectIndex::new("bibliography").unwrap();
    index
        .replace_bibliography_document("refs.bib", 0, text)
        .unwrap();
    let view = index.snapshot();
    let keys = index.complete(&view, Category::Citation, "", 100).unwrap();
    assert_eq!(
        keys.iter().map(|key| key.name.as_str()).collect::<Vec<_>>(),
        ["next", "real"]
    );
    assert!(index.diagnostics(&view).unwrap().is_empty());
    assert!(index
        .definitions(&view, Category::Label, "fake-label")
        .unwrap()
        .is_empty());
}

#[test]
fn duplicate_keys_and_bibitems_are_all_navigation_targets() {
    let mut index = ProjectIndex::new("bibliography").unwrap();
    index
        .replace_document("main.tex", 0, r"\cite{paper}\bibitem{paper}")
        .unwrap();
    index
        .replace_bibliography_document("one.bib", 0, "@book{paper,}")
        .unwrap();
    index
        .replace_bibliography_document("two.bib", 0, "@article{paper,}")
        .unwrap();
    assert_eq!(
        index
            .definitions(&index.snapshot(), Category::Citation, "paper")
            .unwrap()
            .len(),
        3
    );
    index.remove_document("one.bib", 1).unwrap();
    assert!(index
        .unresolved_references(&index.snapshot())
        .unwrap()
        .is_empty());
    index
        .replace_document("main.tex", 1, r"\cite{paper}")
        .unwrap();
    index.remove_document("two.bib", 1).unwrap();
    assert_eq!(
        index.unresolved_references(&index.snapshot()).unwrap()[0].name,
        "paper"
    );
}

#[test]
fn bibliography_replacement_invalidates_affected_citations_and_retained_key_ranges() {
    let mut index = ProjectIndex::new("bibliography").unwrap();
    index
        .replace_document("main.tex", 0, r"\cite{old,new}")
        .unwrap();
    index
        .replace_document("unrelated.tex", 0, r"\cite{unrelated}")
        .unwrap();
    index
        .replace_bibliography_document("refs.bib", 0, "@book{old,}")
        .unwrap();
    let old_key = index
        .definitions(&index.snapshot(), Category::Citation, "old")
        .unwrap()
        .remove(0)
        .source;
    let update = index
        .replace_bibliography_document("refs.bib", 1, "前 @book{new,}")
        .unwrap();
    assert_eq!(update.metrics.documents_reindexed, 1);
    assert_eq!(update.metrics.document_indexes_reused, 2);
    assert_eq!(update.metrics.reference_documents_rechecked, 2);
    assert!(matches!(
        index.source_text(&index.snapshot(), &old_key),
        Err(IndexError::StaleSourceRange { .. })
    ));
    assert_eq!(
        index
            .unresolved_references(&index.snapshot())
            .unwrap()
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>(),
        ["old", "unrelated"]
    );
}

#[test]
fn malformed_outer_entry_recovers_at_next_header_but_unclosed_value_excludes_tail() {
    let mut index = ProjectIndex::new("bibliography").unwrap();
    index
        .replace_bibliography_document(
            "refs.bib",
            0,
            "@book{partial, title={closed value}\n@misc{good,}\n",
        )
        .unwrap();
    let view = index.snapshot();
    assert_eq!(
        index
            .definitions(&view, Category::Citation, "partial")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        index
            .definitions(&view, Category::Citation, "good")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(index.diagnostics(&view).unwrap().len(), 1);
    index
        .replace_bibliography_document(
            "refs.bib",
            1,
            "@book{partial, title={unclosed value\n@misc{hidden,}\n",
        )
        .unwrap();
    let view = index.snapshot();
    assert!(index
        .definitions(&view, Category::Citation, "hidden")
        .unwrap()
        .is_empty());
    assert!(!index.diagnostics(&view).unwrap().is_empty());
}

#[test]
fn invalid_keys_are_diagnosed_and_escaped_at_and_comments_are_ignored() {
    let mut index = ProjectIndex::new("bibliography").unwrap();
    let text = format!("% @misc{{comment,}}\n\\@book{{escaped,}}\n@book{{white space, title={{x}}}}\n@book{{{},}}\n@book{{valid,}}", "x".repeat(4097));
    index
        .replace_bibliography_document("refs.bib", 0, &text)
        .unwrap();
    let view = index.snapshot();
    assert_eq!(
        index
            .symbols(&view)
            .unwrap()
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
        ["valid"]
    );
    assert_eq!(index.diagnostics(&view).unwrap().len(), 2);
}

#[test]
fn bibliography_nesting_bound_excludes_untrusted_tail_with_a_diagnostic() {
    let mut index = ProjectIndex::new("bibliography").unwrap();
    let text = format!(
        "@book{{bounded, title={}x{}}}\n@book{{excluded,}}",
        "{".repeat(150),
        "}".repeat(150)
    );
    index
        .replace_bibliography_document("refs.bib", 0, &text)
        .unwrap();
    let view = index.snapshot();
    assert!(index
        .definitions(&view, Category::Citation, "excluded")
        .unwrap()
        .is_empty());
    assert!(index
        .diagnostics(&view)
        .unwrap()
        .iter()
        .any(|d| d.message.contains("bounds")));
}

#[test]
fn malformed_unicode_bibliography_spans_never_escape_actual_source() {
    let pieces = [
        "東京",
        "@article{α,",
        "@comment{",
        "@book(β,",
        "title=\"",
        "{",
        "}",
        ")",
        "%x\n",
        "\\\"",
        "\n",
        "@misc{fine,}",
    ];
    for seed in 0..200_u32 {
        let mut state = seed + 1;
        let mut text = String::new();
        for _ in 0..40 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            text.push_str(pieces[state as usize % pieces.len()]);
        }
        let mut index = ProjectIndex::new("malformed").unwrap();
        index
            .replace_bibliography_document("refs.bib", 0, &text)
            .unwrap();
        let view = index.snapshot();
        for symbol in index.symbols(&view).unwrap() {
            assert_eq!(
                index.source_text(&view, &symbol.source).unwrap(),
                symbol.name
            );
        }
        for diagnostic in index.diagnostics(&view).unwrap() {
            index.source_text(&view, &diagnostic.source).unwrap();
        }
    }
}
