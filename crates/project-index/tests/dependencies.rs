use flashtex_project_index::{Category, IndexError, ProjectIndex};

#[test]
fn definition_changes_only_recheck_affected_reader_documents() {
    let mut index = ProjectIndex::new("dependencies").unwrap();
    index.replace_document("defs.tex", 0, r"\label{x}").unwrap();
    index.replace_document("a.tex", 0, r"\ref{x}").unwrap();
    index.replace_document("b.tex", 0, r"\ref{y}").unwrap();
    index
        .replace_document("unrelated.tex", 0, r"\ref{z}")
        .unwrap();
    let original = index.unresolved_references(&index.snapshot()).unwrap();
    assert_eq!(
        original.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
        ["y", "z"]
    );
    let update = index.replace_document("defs.tex", 1, r"\label{y}").unwrap();
    assert_eq!(update.metrics.documents_reindexed, 1);
    assert_eq!(update.metrics.document_indexes_reused, 3);
    assert_eq!(update.metrics.definition_availability_changes, 2);
    assert_eq!(update.metrics.reference_documents_rechecked, 3);
    let after = index.unresolved_references(&index.snapshot()).unwrap();
    assert_eq!(
        after.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
        ["x", "z"]
    );
    assert_eq!(after[1], original[1]);
}

#[test]
fn unrelated_and_same_definition_edits_recheck_only_the_changed_document() {
    let mut index = ProjectIndex::new("dependencies").unwrap();
    index.replace_document("defs.tex", 0, r"\label{x}").unwrap();
    index
        .replace_document("a.tex", 0, r"\ref{x}\ref{missing}")
        .unwrap();
    let before = index.unresolved_references(&index.snapshot()).unwrap();
    let result = index
        .replace_document("defs.tex", 1, "UTF8 東京 \\label{x}")
        .unwrap();
    assert_eq!(result.metrics.reference_documents_rechecked, 1);
    assert_eq!(result.metrics.definition_availability_changes, 0);
    assert_eq!(
        result.metrics.input_bytes_reindexed,
        "UTF8 東京 \\label{x}".len()
    );
    assert_eq!(result.metrics.document_indexes_reused, 1);
    assert_eq!(
        index.unresolved_references(&index.snapshot()).unwrap(),
        before
    );
    let result = index
        .replace_document("a.tex", 1, "前 \\ref{x}\\ref{missing}")
        .unwrap();
    assert_eq!(result.metrics.reference_documents_rechecked, 1);
    let after = index.unresolved_references(&index.snapshot()).unwrap();
    assert_eq!(after[0].source.revision, 1);
    assert!(after[0].source.start_byte > before[0].source.start_byte);
    assert_eq!(
        index.source_text(&index.snapshot(), &after[0].source),
        Ok("missing")
    );
}

#[test]
fn duplicate_definition_removal_does_not_unresolve_until_last_definition_is_gone() {
    let mut index = ProjectIndex::new("dependencies").unwrap();
    index
        .replace_document("one.tex", 0, r"\label{x}\label{x}")
        .unwrap();
    index.replace_document("two.tex", 0, r"\label{x}").unwrap();
    index.replace_document("reader.tex", 0, r"\ref{x}").unwrap();
    index.remove_document("one.tex", 1).unwrap();
    let metrics = index
        .last_reindex_metrics(&index.snapshot())
        .unwrap()
        .unwrap();
    assert_eq!(metrics.documents_reindexed, 0);
    assert_eq!(metrics.reference_documents_rechecked, 0);
    assert!(index
        .unresolved_references(&index.snapshot())
        .unwrap()
        .is_empty());
    index.remove_document("two.tex", 1).unwrap();
    let metrics = index
        .last_reindex_metrics(&index.snapshot())
        .unwrap()
        .unwrap();
    assert_eq!(metrics.reference_documents_rechecked, 1);
    assert_eq!(metrics.definition_availability_changes, 1);
    assert_eq!(
        index.unresolved_references(&index.snapshot()).unwrap()[0].name,
        "x"
    );
}

#[test]
fn citation_and_label_names_have_independent_dependencies_and_commands_are_not_errors() {
    let mut index = ProjectIndex::new("dependencies").unwrap();
    index
        .replace_document("reader.tex", 0, r"\cite{same}\ref{same}\unknownCommand")
        .unwrap();
    assert_eq!(
        index
            .unresolved_references(&index.snapshot())
            .unwrap()
            .len(),
        2
    );
    index
        .replace_document("defs.tex", 0, r"\bibitem{same}")
        .unwrap();
    let unresolved = index.unresolved_references(&index.snapshot()).unwrap();
    assert_eq!(unresolved.len(), 1);
    assert_eq!(unresolved[0].category, Category::Label);
    index
        .replace_document("defs.tex", 1, r"\bibitem{same}\label{same}")
        .unwrap();
    assert!(index
        .unresolved_references(&index.snapshot())
        .unwrap()
        .is_empty());
}

#[test]
fn removing_a_reader_drops_its_cached_diagnostics_and_rejected_updates_keep_metrics() {
    let mut index = ProjectIndex::new("dependencies").unwrap();
    assert_eq!(index.last_reindex_metrics(&index.snapshot()).unwrap(), None);
    index
        .replace_document("reader.tex", 0, r"\ref{missing}")
        .unwrap();
    let snapshot = index.snapshot();
    let metrics = index.last_reindex_metrics(&snapshot).unwrap();
    assert!(index.replace_document("reader.tex", 0, "").is_err());
    assert_eq!(index.last_reindex_metrics(&snapshot).unwrap(), metrics);
    index.remove_document("reader.tex", 1).unwrap();
    assert_eq!(
        index.unresolved_references(&snapshot),
        Err(IndexError::StaleSnapshot)
    );
    assert_eq!(
        index.last_reindex_metrics(&snapshot),
        Err(IndexError::StaleSnapshot)
    );
    assert!(index
        .unresolved_references(&index.snapshot())
        .unwrap()
        .is_empty());
}

#[test]
fn retained_indexes_match_clean_rebuild_after_each_definition_and_reference_edit() {
    let mut index = ProjectIndex::new("incremental").unwrap();
    let mut sources = [
        ("defs.tex", 0, r"\label{a}\bibitem{paper}".to_owned()),
        ("reader.tex", 0, r"\ref{a}\cite{paper}".to_owned()),
        ("other.tex", 0, r"\ref{b}".to_owned()),
    ];
    for (file, revision, text) in &sources {
        index.replace_document(file, *revision, text).unwrap();
    }
    for revision in 1..=16 {
        let slot = revision as usize % sources.len();
        sources[slot].1 = revision;
        sources[slot].2 = if revision % 2 == 0 {
            format!("東京 \\label{{a}}\\ref{{b}}\\cite{{paper}} {revision}")
        } else {
            format!("\\label{{b}}\\bibitem{{paper}}\\ref{{a}} {revision}")
        };
        let update = index
            .replace_document(sources[slot].0, revision, &sources[slot].2)
            .unwrap();
        assert_eq!(update.metrics.documents_reindexed, 1);
        assert_eq!(update.metrics.document_indexes_reused, 2);
        assert!(update.metrics.total_elapsed_nanos >= update.metrics.lexical_elapsed_nanos);
        let mut clean = ProjectIndex::new("clean").unwrap();
        for (file, rev, text) in &sources {
            clean.replace_document(file, *rev, text).unwrap();
        }
        assert_eq!(
            index.symbols(&index.snapshot()).unwrap(),
            clean.symbols(&clean.snapshot()).unwrap()
        );
        assert_eq!(
            index.unresolved_references(&index.snapshot()).unwrap(),
            clean.unresolved_references(&clean.snapshot()).unwrap()
        );
        assert_eq!(
            index.diagnostics(&index.snapshot()).unwrap(),
            clean.diagnostics(&clean.snapshot()).unwrap()
        );
    }
}
