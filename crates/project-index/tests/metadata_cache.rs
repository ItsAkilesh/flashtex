use flashtex_project_index::*;

#[test]
fn transitive_macro_edits_recompute_only_dependent_keys() {
    let mut index = ProjectIndex::new("cache").unwrap();
    index
        .replace_bibliography_document("macros.bib", 1, "@string{base={A}}")
        .unwrap();
    index
        .replace_bibliography_document(
            "refs.bib",
            1,
            "@string{indirect=base} @book{a,title=indirect} @book{b,title={B}}",
        )
        .unwrap();
    let old = index.snapshot();
    let old_span = index
        .citation_metadata(&old, "a")
        .unwrap()
        .field("title")
        .unwrap()
        .source
        .clone();
    index
        .replace_bibliography_document("macros.bib", 2, "@string{base={New}}")
        .unwrap();
    let snapshot = index.snapshot();
    assert_eq!(
        index.metadata_cache_metrics(&snapshot).unwrap(),
        MetadataCacheMetrics {
            keys_recomputed: 1,
            keys_reused: 1,
            keys_removed: 0
        }
    );
    assert_eq!(
        index
            .citation_metadata(&snapshot, "a")
            .unwrap()
            .field("title")
            .unwrap()
            .value
            .as_deref(),
        Some("New")
    );
    assert_eq!(index.source_text(&snapshot, &old_span).unwrap(), "indirect");
    assert_eq!(
        index.complete_citations(&old, "", 10),
        Err(IndexError::StaleSnapshot)
    );
    index
        .replace_document("unrelated.tex", 1, "plain text")
        .unwrap();
    assert_eq!(
        index
            .metadata_cache_metrics(&index.snapshot())
            .unwrap()
            .keys_reused,
        2
    );
    assert_eq!(
        index
            .metadata_cache_metrics(&index.snapshot())
            .unwrap()
            .keys_recomputed,
        0
    );
}

#[test]
fn missing_macro_creation_duplicate_removal_and_metadata_revisions_invalidate() {
    let mut index = ProjectIndex::new("cache").unwrap();
    index
        .replace_bibliography_document("refs.bib", 1, "@book{k,author=who,title={Old},year=2025}")
        .unwrap();
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "k")
            .unwrap()
            .status,
        MetadataStatus::Incomplete
    );
    index
        .replace_bibliography_document("macros.bib", 1, "@string{who={Å}} @string{who={B}}")
        .unwrap();
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "k")
            .unwrap()
            .status,
        MetadataStatus::Incomplete
    );
    index
        .replace_bibliography_document("macros.bib", 2, "@string{who={Å}}")
        .unwrap();
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "k")
            .unwrap()
            .status,
        MetadataStatus::Resolved
    );
    let old_span = index
        .citation_metadata(&index.snapshot(), "k")
        .unwrap()
        .field("title")
        .unwrap()
        .source
        .clone();
    index
        .replace_bibliography_document("refs.bib", 2, "@book{k,author=who,title={New},year=2026}")
        .unwrap();
    let snapshot = index.snapshot();
    let completion = index.complete_citations(&snapshot, "k", 1).unwrap();
    assert_eq!(
        completion[0].field("AUTHOR").unwrap().value.as_deref(),
        Some("Å")
    );
    assert_eq!(
        completion[0].field("title").unwrap().value.as_deref(),
        Some("New")
    );
    assert_eq!(
        index
            .source_text(&snapshot, &completion[0].field("year").unwrap().source)
            .unwrap(),
        "2026"
    );
    assert!(matches!(
        index.source_text(&snapshot, &old_span),
        Err(IndexError::StaleSourceRange { .. })
    ));
    index.remove_document("macros.bib", 3).unwrap();
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "k")
            .unwrap()
            .status,
        MetadataStatus::Incomplete
    );
}

#[test]
fn duplicate_keys_and_removal_keep_cache_exact() {
    let mut index = ProjectIndex::new("cache").unwrap();
    index
        .replace_bibliography_document("a.bib", 1, "@book{k,title={A}}")
        .unwrap();
    index
        .replace_bibliography_document("b.bib", 1, "@book{k,title={B}}")
        .unwrap();
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "k")
            .unwrap()
            .status,
        MetadataStatus::Ambiguous
    );
    index.remove_document("b.bib", 2).unwrap();
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "k")
            .unwrap()
            .status,
        MetadataStatus::Resolved
    );
    index.replace_document("a.tex", 1, r"\cite{k,z}").unwrap();
    index.remove_document("a.bib", 2).unwrap();
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "k")
            .unwrap()
            .status,
        MetadataStatus::Missing
    );
    assert_eq!(
        index
            .complete_citations(&index.snapshot(), "", 10)
            .unwrap()
            .iter()
            .map(|m| m.key.as_str())
            .collect::<Vec<_>>(),
        ["k", "z"]
    );
    index.remove_document("a.tex", 2).unwrap();
    assert_eq!(
        index
            .metadata_cache_metrics(&index.snapshot())
            .unwrap()
            .keys_removed,
        2
    );
    assert!(index
        .complete_citations(&index.snapshot(), "", 10)
        .unwrap()
        .is_empty());
}

#[test]
fn incremental_results_equal_fresh_rebuild_through_macro_failure_recovery() {
    let mut index = ProjectIndex::new("cache").unwrap();
    let refs = "@book{a,title=x # y,year=2026} @book{b,title={stable}}";
    index
        .replace_bibliography_document("refs.bib", 1, refs)
        .unwrap();
    for (n, macros) in [
        "",
        "@string{y={Y}}",
        "@string{x=z} @string{y={Y}}",
        "@string{x=z} @string{z={Z}} @string{y={Y}}",
        "@string{x=z} @string{z=x}",
        "@string{x={X}} @string{x=}",
        "@string{x={X}} @string{y={Y}}",
    ]
    .iter()
    .enumerate()
    {
        let revision = n as u64 + 1;
        index
            .replace_bibliography_document("macros.bib", revision, macros)
            .unwrap();
        let mut rebuilt = ProjectIndex::new("cache").unwrap();
        rebuilt
            .replace_bibliography_document("refs.bib", 1, refs)
            .unwrap();
        rebuilt
            .replace_bibliography_document("macros.bib", revision, macros)
            .unwrap();
        assert_eq!(
            index
                .complete_citations(&index.snapshot(), "", 100)
                .unwrap(),
            rebuilt
                .complete_citations(&rebuilt.snapshot(), "", 100)
                .unwrap(),
            "{macros}"
        );
    }
}
