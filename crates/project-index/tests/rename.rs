use flashtex_project_index::{Category, IndexError, ProjectIndex};
use std::collections::BTreeMap;

fn project() -> ProjectIndex {
    let mut index = ProjectIndex::new("rename").unwrap();
    index
        .replace_document("z.tex", 4, "東京 \\label{sec:α} \\cref{sec:α,sec:α}")
        .unwrap();
    index
        .replace_document(
            "a.tex",
            9,
            r"See \ref{sec:α}, not \cite{sec:α}. % \ref{sec:α}",
        )
        .unwrap();
    index
}

#[test]
fn rename_plan_is_project_wide_exact_sorted_and_does_not_mutate() {
    let index = project();
    let snapshot = index.snapshot();
    let before = index.symbols(&snapshot).unwrap();
    let plan = index
        .plan_label_rename(&snapshot, "sec:α", "chapter:東京")
        .unwrap();
    assert_eq!(plan.edits.len(), 4);
    assert_eq!(plan.edits[0].source.file, "a.tex");
    assert_eq!(plan.edits[1].source.file, "z.tex");
    for edit in &plan.edits {
        assert_eq!(index.source_text(&snapshot, &edit.source).unwrap(), "sec:α");
        assert_eq!(edit.expected_text, "sec:α");
        assert_eq!(edit.replacement, "chapter:東京");
    }
    for pair in plan.edits.windows(2) {
        if pair[0].source.file == pair[1].source.file {
            assert!(pair[0].source.end_byte <= pair[1].source.start_byte);
        }
    }
    assert_eq!(
        index
            .plan_label_rename(&snapshot, "sec:α", "chapter:東京")
            .unwrap(),
        plan
    );
    index.validate_rename_plan(&plan).unwrap();
    assert_eq!(index.snapshot(), snapshot);
    assert_eq!(index.symbols(&snapshot).unwrap(), before);
}

#[test]
fn caller_can_apply_reverse_edits_to_a_copy_without_offset_drift() {
    let index = project();
    let mut copies = BTreeMap::from([
        (
            "z.tex".to_owned(),
            "東京 \\label{sec:α} \\cref{sec:α,sec:α}".to_owned(),
        ),
        (
            "a.tex".to_owned(),
            r"See \ref{sec:α}, not \cite{sec:α}. % \ref{sec:α}".to_owned(),
        ),
    ]);
    let plan = index
        .plan_label_rename(&index.snapshot(), "sec:α", "長い:label")
        .unwrap();
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
        copies["z.tex"],
        "東京 \\label{長い:label} \\cref{長い:label,長い:label}"
    );
    assert_eq!(
        copies["a.tex"],
        r"See \ref{長い:label}, not \cite{sec:α}. % \ref{sec:α}"
    );
    assert_eq!(
        index
            .definitions(&index.snapshot(), Category::Label, "sec:α")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn stale_snapshot_or_retained_range_cannot_be_renamed() {
    let mut index = project();
    let snapshot = index.snapshot();
    let source = index
        .definitions(&snapshot, Category::Label, "sec:α")
        .unwrap()
        .remove(0)
        .source;
    let plan = index
        .plan_label_rename_at(&snapshot, &source, "new")
        .unwrap();
    index
        .replace_document("z.tex", 5, "shift \\label{sec:α}")
        .unwrap();
    assert_eq!(
        index.validate_rename_plan(&plan),
        Err(IndexError::StaleSnapshot)
    );
    assert!(matches!(
        index.plan_label_rename_at(&index.snapshot(), &source, "new"),
        Err(IndexError::StaleSourceRange { .. })
    ));
}

#[test]
fn all_plan_tampering_is_rejected() {
    let index = project();
    let plan = index
        .plan_label_rename(&index.snapshot(), "sec:α", "new")
        .unwrap();
    let mut variants = Vec::new();
    let mut changed = plan.clone();
    changed.edits.remove(0);
    variants.push(changed);
    let mut changed = plan.clone();
    changed.edits.push(changed.edits[0].clone());
    variants.push(changed);
    let mut changed = plan.clone();
    changed.edits.reverse();
    variants.push(changed);
    let mut changed = plan.clone();
    changed.edits[0].expected_text = "wrong".into();
    variants.push(changed);
    let mut changed = plan.clone();
    changed.edits[0].replacement = "wrong".into();
    variants.push(changed);
    let mut changed = plan.clone();
    changed.edits[0].source.start_byte += 1;
    variants.push(changed);
    let mut changed = plan.clone();
    changed.edits[0].source.revision += 1;
    variants.push(changed);
    for changed in variants {
        assert_eq!(
            index.validate_rename_plan(&changed),
            Err(IndexError::InvalidRenamePlan)
        );
    }
}

#[test]
fn collision_with_definition_or_unresolved_reference_is_rejected() {
    for source in [r"\label{taken}", r"\ref{taken}"] {
        let mut index = project();
        index.replace_document("other.tex", 0, source).unwrap();
        assert_eq!(
            index.plan_label_rename(&index.snapshot(), "sec:α", "taken"),
            Err(IndexError::RenameCollision {
                name: "taken".into()
            })
        );
    }
}

#[test]
fn invalid_names_missing_definitions_and_partial_selections_are_rejected() {
    let index = project();
    let snapshot = index.snapshot();
    for name in [
        "",
        "white space",
        "a,b",
        "a%comment",
        "\\macro",
        "{dynamic}",
        "a\n",
        "a\0",
    ] {
        assert_eq!(
            index.plan_label_rename(&snapshot, "sec:α", name),
            Err(IndexError::InvalidLabelName)
        );
    }
    assert_eq!(
        index.plan_label_rename(&snapshot, "unknown", "new"),
        Err(IndexError::MissingLabelDefinition)
    );
    let mut source = index
        .definitions(&snapshot, Category::Label, "sec:α")
        .unwrap()
        .remove(0)
        .source;
    source.end_byte -= 2;
    assert_eq!(
        index.plan_label_rename_at(&snapshot, &source, "new"),
        Err(IndexError::InvalidRenamePlan)
    );
}

#[test]
fn duplicate_lexical_definitions_are_all_included_and_same_name_is_noop() {
    let mut index = project();
    index
        .replace_document("duplicate.tex", 1, r"\label{sec:α}")
        .unwrap();
    let snapshot = index.snapshot();
    assert_eq!(
        index
            .plan_label_rename(&snapshot, "sec:α", "new")
            .unwrap()
            .edits
            .len(),
        5
    );
    let noop = index
        .plan_label_rename(&snapshot, "sec:α", "sec:α")
        .unwrap();
    assert!(noop.edits.is_empty());
    index.validate_rename_plan(&noop).unwrap();
}
