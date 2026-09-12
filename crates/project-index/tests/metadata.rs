use flashtex_project_index::*;

fn index(source: &str) -> ProjectIndex {
    let mut index = ProjectIndex::new("metadata").unwrap();
    index
        .replace_bibliography_document("refs.bib", 1, source)
        .unwrap();
    index
}

#[test]
fn nested_literals_concatenation_macros_and_exact_unicode_spans() {
    let source = r#"@string{prefix="Hello "}
@book{κ, author={Å {B}}, title=PREFIX # "{世界}" # {\TeX}, year=2026}"#;
    let index = index(source);
    let snapshot = index.snapshot();
    let metadata = index.citation_metadata(&snapshot, "κ").unwrap();
    assert_eq!(metadata.status, MetadataStatus::Resolved);
    assert_eq!(
        metadata.fields["title"].value.as_deref(),
        Some(r"Hello {世界}\TeX")
    );
    assert_eq!(metadata.fields["author"].value.as_deref(), Some("Å {B}"));
    assert_eq!(metadata.fields["year"].value.as_deref(), Some("2026"));
    assert_eq!(
        index
            .source_text(&snapshot, metadata.records[0].key.as_ref().unwrap())
            .unwrap(),
        "κ"
    );
    for field in &metadata.records[0].fields {
        assert_eq!(
            index.source_text(&snapshot, &field.name_source).unwrap(),
            field.name
        );
        for part in &field.parts {
            let raw = index.source_text(&snapshot, &part.source).unwrap();
            assert!(raw.contains(&part.text));
        }
    }
}

#[test]
fn repeated_keys_and_fields_never_resolve() {
    for (source, status) in [
        (
            "@book{k,title={A}} @book{k,title={B}}",
            MetadataStatus::Ambiguous,
        ),
        ("@book{k,title={A},TITLE={B}}", MetadataStatus::Malformed),
        (
            "@book{k,title={A}\n@book{good,title={B}}",
            MetadataStatus::Malformed,
        ),
        ("@book{k,title=}", MetadataStatus::Malformed),
    ] {
        let index = index(source);
        let metadata = index.citation_metadata(&index.snapshot(), "k").unwrap();
        assert_eq!(metadata.status, status, "{source}");
        assert!(metadata.fields.is_empty());
    }
}

#[test]
fn missing_ambiguous_malformed_and_cyclic_macros_remain_explicit() {
    for declarations in [
        "",
        "@string{x={a}} @string{x={b}}",
        "@string{x=}",
        "@string{x=y} @string{y=x}",
        "@string{x={a}} @string{x=}",
    ] {
        let index = index(&format!("{declarations} @book{{k,title=x # {{suffix}}}}"));
        let metadata = index.citation_metadata(&index.snapshot(), "k").unwrap();
        assert_eq!(
            metadata.status,
            MetadataStatus::Incomplete,
            "{declarations}"
        );
        assert!(metadata.fields["title"].value.is_none());
        assert!(!metadata.fields["title"].issues.is_empty());
    }
}

#[test]
fn recovery_does_not_expose_entry_text_in_literals() {
    let index = index(
        r#"@book{bad,title=!,year=2020}
@book{good,title="@book{fake,title={x}}"}"#,
    );
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "bad")
            .unwrap()
            .status,
        MetadataStatus::Malformed
    );
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "good")
            .unwrap()
            .status,
        MetadataStatus::Resolved
    );
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "fake")
            .unwrap()
            .status,
        MetadataStatus::Missing
    );
}

#[test]
fn expansion_and_concatenation_bounds_are_observable() {
    let atoms = vec!["{a}"; 257].join(" # ");
    let index = index(&format!("@book{{k,title={atoms}}}"));
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "k")
            .unwrap()
            .status,
        MetadataStatus::Malformed
    );
    let mut source = String::from("@string{x0={aa}}\n");
    for n in 1..22 {
        source.push_str(&format!("@string{{x{n}=x{} # x{}}}\n", n - 1, n - 1));
    }
    source.push_str("@book{k,title=x21}");
    let index = self::index(&source);
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "k")
            .unwrap()
            .status,
        MetadataStatus::Incomplete
    );
}

#[test]
fn metadata_queries_reject_stale_snapshots_and_bibitem_is_not_a_record() {
    let mut index = index("@book{k,title={A}}");
    let old = index.snapshot();
    index.replace_document("a.tex", 1, r"\bibitem{k}").unwrap();
    assert_eq!(
        index.citation_metadata(&old, "k"),
        Err(IndexError::StaleSnapshot)
    );
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "k")
            .unwrap()
            .status,
        MetadataStatus::Ambiguous
    );
    index.remove_document("refs.bib", 2).unwrap();
    assert_eq!(
        index
            .citation_metadata(&index.snapshot(), "k")
            .unwrap()
            .status,
        MetadataStatus::NoBibliographyRecord
    );
}
