use flashtex_project_index::*;
use std::collections::BTreeMap;

fn setup() -> ProjectIndex {
    let mut index = ProjectIndex::new("cite").unwrap();
    index
        .replace_bibliography_document("refs.bib", 3, "@book{κ,title={κ},author=unknown}")
        .unwrap();
    index.replace_document("a.tex", 1, "% \\cite{κ}\n\\cite[see][p.2]{κ,other,κ}\n\\verb|\\cite{κ}|\n\\begin{verbatim}\\cite{κ}\\end{verbatim}").unwrap();
    index.replace_document("b.tex", 9, r"\cite{κ}").unwrap();
    index
}

#[test]
fn multifile_rename_preserves_literal_text_comments_and_multikey_syntax() {
    let index = setup();
    let snapshot = index.snapshot();
    let plan = index
        .plan_citation_rename(&snapshot, "κ", "引用:2026")
        .unwrap();
    assert_eq!(plan.edits.len(), 4);
    index.validate_citation_rename_plan(&plan).unwrap();
    assert_eq!(index.snapshot(), snapshot);
    let mut copies = BTreeMap::new();
    for file in snapshot.documents.keys() {
        let end = match file.as_str() {
            "refs.bib" => "@book{κ,title={κ},author=unknown}".len(),
            "a.tex" => "% \\cite{κ}\n\\cite[see][p.2]{κ,other,κ}\n\\verb|\\cite{κ}|\n\\begin{verbatim}\\cite{κ}\\end{verbatim}".len(),
            _ => r"\cite{κ}".len(),
        };
        copies.insert(
            file.clone(),
            index
                .source_text(
                    &snapshot,
                    &SourceSpan {
                        file: file.clone(),
                        revision: snapshot.documents[file],
                        start_byte: 0,
                        end_byte: end,
                    },
                )
                .unwrap()
                .to_owned(),
        );
    }
    for edit in plan.edits.iter().rev() {
        let text = copies.get_mut(&edit.source.file).unwrap();
        assert_eq!(
            &text[edit.source.start_byte..edit.source.end_byte],
            edit.expected_text
        );
        text.replace_range(
            edit.source.start_byte..edit.source.end_byte,
            &edit.replacement,
        );
    }
    assert_eq!(
        copies["refs.bib"],
        "@book{引用:2026,title={κ},author=unknown}"
    );
    assert!(copies["a.tex"].contains(r"\cite[see][p.2]{引用:2026,other,引用:2026}"));
    assert!(copies["a.tex"].contains("% \\cite{κ}"));
    assert!(copies["a.tex"].contains(r"\verb|\cite{κ}|"));
    assert!(copies["a.tex"].contains(r"\begin{verbatim}\cite{κ}\end{verbatim}"));
}

#[test]
fn duplicate_missing_bibitem_only_and_malformed_definitions_are_refused() {
    for (source, expected) in [
        (
            "@book{k,title={A}} @book{k,title={B}}",
            IndexError::AmbiguousCitationDefinition,
        ),
        (
            "@book{k,title=}",
            IndexError::MalformedBibliographyDefinition,
        ),
        (
            "@book{k,title={A}",
            IndexError::MalformedBibliographyDefinition,
        ),
        (
            "@book{k,title={A},title={B}}",
            IndexError::MalformedBibliographyDefinition,
        ),
        ("", IndexError::MissingBibliographyDefinition),
    ] {
        let mut index = ProjectIndex::new("cite").unwrap();
        index
            .replace_bibliography_document("r.bib", 1, source)
            .unwrap();
        assert_eq!(
            index.plan_citation_rename(&index.snapshot(), "k", "new"),
            Err(expected)
        );
    }
    let mut index = ProjectIndex::new("cite").unwrap();
    index
        .replace_document("a.tex", 1, r"\bibitem{k}\cite{k}")
        .unwrap();
    assert_eq!(
        index.plan_citation_rename(&index.snapshot(), "k", "new"),
        Err(IndexError::MissingBibliographyDefinition)
    );
    index
        .replace_bibliography_document("r.bib", 1, "@book{k,title={A}}")
        .unwrap();
    assert_eq!(
        index.plan_citation_rename(&index.snapshot(), "k", "new"),
        Err(IndexError::AmbiguousCitationDefinition)
    );
}

#[test]
fn collisions_invalid_keys_and_noop_have_explicit_behavior() {
    let index = setup();
    assert_eq!(
        index.plan_citation_rename(&index.snapshot(), "κ", "other"),
        Err(IndexError::RenameCollision {
            name: "other".into()
        })
    );
    for name in [
        "", "a,b", "a b", "a%", "a=", "a@", "a#", "a(", "a)", "a\"", "a\\", "a\0",
    ] {
        assert_eq!(
            index.plan_citation_rename(&index.snapshot(), "κ", name),
            Err(IndexError::InvalidCitationKey),
            "{name}"
        );
    }
    assert!(index
        .plan_citation_rename(&index.snapshot(), "κ", "κ")
        .unwrap()
        .edits
        .is_empty());
}

#[test]
fn exact_selection_and_full_snapshot_guards_cover_unicode_and_stale_sources() {
    let mut index = setup();
    let snapshot = index.snapshot();
    let source = index
        .occurrences(&snapshot, Category::Citation, "κ")
        .unwrap()[0]
        .source
        .clone();
    let plan = index
        .plan_citation_rename_at(&snapshot, &source, "new")
        .unwrap();
    let mut partial = source.clone();
    partial.start_byte += 1;
    assert_eq!(
        index.plan_citation_rename_at(&snapshot, &partial, "new"),
        Err(IndexError::InvalidOffset)
    );
    index.replace_document("a.tex", 2, r"\cite{κ}").unwrap();
    assert_eq!(
        index.validate_citation_rename_plan(&plan),
        Err(IndexError::StaleSnapshot)
    );
    assert!(matches!(
        index.plan_citation_rename_at(&index.snapshot(), &source, "new"),
        Err(IndexError::StaleSourceRange { .. })
    ));
}

#[test]
fn all_edit_tampering_including_overlap_is_rejected() {
    let index = setup();
    let plan = index
        .plan_citation_rename(&index.snapshot(), "κ", "new")
        .unwrap();
    for change in 0..7 {
        let mut changed = plan.clone();
        match change {
            0 => {
                changed.edits.pop();
            }
            1 => changed.edits.reverse(),
            2 => changed.edits.push(changed.edits[0].clone()),
            3 => changed.edits[0].expected_text = "wrong".into(),
            4 => changed.edits[0].replacement = "wrong".into(),
            5 => changed.edits[0].source.revision += 1,
            _ => changed.edits[0].source.end_byte = usize::MAX,
        }
        assert_eq!(
            index.validate_citation_rename_plan(&changed),
            Err(IndexError::InvalidCitationRenamePlan)
        );
    }
}

#[test]
fn native_citation_wire_is_kind_tagged_bounded_and_validated() {
    let mut index = setup();
    let plan = index
        .plan_citation_rename(&index.snapshot(), "κ", "引用")
        .unwrap();
    let wire = index
        .serialize_citation_rename_plan(&plan, MAX_REPLACEMENT_WIRE_BYTES)
        .unwrap();
    assert!(wire
        .contains(r#""schema":"flashtex.citation-rename-plan.v1","kind":"citation_key_rename""#));
    assert!(wire.contains(r#""rename":{"old_name":"κ","new_name":"引用"}"#));
    assert!(wire.contains(r#""proposal_only":true,"requires_user_approval":true"#));
    assert!(!wire.contains("\"search\""));
    assert_eq!(
        index
            .serialize_citation_rename_plan(&plan, wire.len())
            .unwrap(),
        wire
    );
    assert_eq!(
        index.serialize_citation_rename_plan(&plan, wire.len() - 1),
        Err(IndexError::SerializationLimit)
    );
    let mut changed = plan.clone();
    changed.edits[0].expected_text = "wrong".into();
    assert_eq!(
        index.serialize_citation_rename_plan(&changed, MAX_REPLACEMENT_WIRE_BYTES),
        Err(IndexError::InvalidCitationRenamePlan)
    );
    index.replace_document("b.tex", 10, "changed").unwrap();
    assert_eq!(
        index.serialize_citation_rename_plan(&plan, MAX_REPLACEMENT_WIRE_BYTES),
        Err(IndexError::StaleSnapshot)
    );
}
