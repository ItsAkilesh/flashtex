use flashtex_project_index::*;
use std::collections::BTreeMap;

fn setup() -> (ProjectIndex, SearchResult) {
    let mut index = ProjectIndex::new("replace").unwrap();
    index.replace_document("a.tex", 1, "é éé").unwrap();
    index.replace_document("b.tex", 4, "% é").unwrap();
    let search = index
        .search_literal(&index.snapshot(), &SearchRequest::literal("é"), || false)
        .unwrap();
    (index, search)
}

#[test]
fn guarded_multidocument_plan_can_be_explicitly_applied_to_caller_copies() {
    let (index, search) = setup();
    let before = index.snapshot();
    let plan = index.plan_literal_replacement(&search, "文").unwrap();
    index.validate_literal_replacement_plan(&plan).unwrap();
    assert_eq!(index.snapshot(), before);
    assert_eq!(
        index
            .search_literal(&before, &search.request, || false)
            .unwrap(),
        search
    );
    let mut copies = BTreeMap::from([
        ("a.tex".to_string(), "é éé".to_string()),
        ("b.tex".to_string(), "% é".to_string()),
    ]);
    // Test caller explicitly elects to apply this validated plan to private copies.
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
    assert_eq!(copies["a.tex"], "文 文文");
    assert_eq!(copies["b.tex"], "% 文");
    for edits in plan.edits.windows(2) {
        assert!(
            edits[0].source.file < edits[1].source.file
                || edits[0].source.end_byte <= edits[1].source.start_byte
        );
    }
}

#[test]
fn partial_or_forged_search_results_cannot_become_replace_all() {
    let (index, search) = setup();
    for termination in [
        SearchTermination::Cancelled,
        SearchTermination::MatchLimit,
        SearchTermination::WorkLimit,
    ] {
        let mut changed = search.clone();
        changed.termination = termination;
        assert_eq!(
            index.plan_literal_replacement(&changed, ""),
            Err(IndexError::IncompleteSearch)
        );
    }
    for change in 0..5 {
        let mut changed = search.clone();
        match change {
            0 => {
                changed.matches.pop();
            }
            1 => changed.matches.reverse(),
            2 => changed.matches[0].start_byte += 1,
            3 => changed.work_used += 1,
            _ => changed.matches[0].revision += 1,
        }
        assert_eq!(
            index.plan_literal_replacement(&changed, ""),
            Err(IndexError::InvalidSearchPlan)
        );
    }
}

#[test]
fn edit_tampering_omissions_duplicates_and_reordering_are_rejected() {
    let (index, search) = setup();
    let plan = index.plan_literal_replacement(&search, "x").unwrap();
    for change in 0..7 {
        let mut changed = plan.clone();
        match change {
            0 => {
                changed.edits.pop();
            }
            1 => changed.edits.reverse(),
            2 => changed.edits[0].expected_text = "wrong".into(),
            3 => changed.edits[0].replacement = "wrong".into(),
            4 => changed.edits[0].source.revision += 1,
            5 => changed.edits[0].source.end_byte += 1,
            _ => changed.edits.push(changed.edits[0].clone()),
        }
        assert_eq!(
            index.validate_literal_replacement_plan(&changed),
            Err(IndexError::InvalidSearchPlan)
        );
    }
}

#[test]
fn stale_snapshot_and_text_amplification_are_rejected_and_deletion_is_supported() {
    let (mut index, search) = setup();
    let deletion = index.plan_literal_replacement(&search, "").unwrap();
    assert!(deletion.edits.iter().all(|e| e.replacement.is_empty()));
    let huge = "x".repeat(MAX_REPLACEMENT_PLAN_TEXT_BYTES / 2);
    assert_eq!(
        index.plan_literal_replacement(&search, &huge),
        Err(IndexError::ReplacementPlanTooLarge)
    );
    index.replace_document("b.tex", 5, "% changed").unwrap();
    assert_eq!(
        index.validate_literal_replacement_plan(&deletion),
        Err(IndexError::StaleSnapshot)
    );
}

#[test]
fn selected_scope_and_exact_final_match_limit_can_make_complete_plans() {
    let (index, _) = setup();
    let mut request = SearchRequest::literal("é");
    request.documents = Some(["b.tex".to_owned()].into_iter().collect());
    request.max_matches = 1;
    let search = index
        .search_literal(&index.snapshot(), &request, || false)
        .unwrap();
    assert_eq!(search.termination, SearchTermination::Complete);
    let plan = index.plan_literal_replacement(&search, "x").unwrap();
    assert_eq!(plan.edits.len(), 1);
    assert_eq!(plan.edits[0].source.file, "b.tex");
    request.literal = "absent".into();
    let empty = index
        .search_literal(&index.snapshot(), &request, || false)
        .unwrap();
    assert!(index
        .plan_literal_replacement(&empty, "x")
        .unwrap()
        .edits
        .is_empty());
}
