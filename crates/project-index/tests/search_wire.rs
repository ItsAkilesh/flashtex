use flashtex_project_index::*;

fn setup() -> (ProjectIndex, LiteralReplacementPlan) {
    let mut index = ProjectIndex::new("project\"\\\n文").unwrap();
    index.replace_document("α.tex", u64::MAX, "é").unwrap();
    let search = index
        .search_literal(&index.snapshot(), &SearchRequest::literal("é"), || false)
        .unwrap();
    let plan = index
        .plan_literal_replacement(&search, "\"\\\n\r\t\0文")
        .unwrap();
    (index, plan)
}

#[test]
fn wire_escapes_controls_preserves_unicode_and_exact_u64_revisions() {
    let (index, plan) = setup();
    let wire = index
        .serialize_literal_replacement_plan(&plan, MAX_REPLACEMENT_WIRE_BYTES)
        .unwrap();
    assert!(wire.contains(r#""schema":"flashtex.literal-replacement-plan.v1""#));
    assert!(wire.contains(r#""proposal_only":true,"requires_user_approval":true"#));
    assert!(wire.contains(r#""revision":"18446744073709551615""#));
    assert!(wire.contains(r#""start_byte":"0","end_byte":"2","expected_text":"é""#));
    assert!(wire.contains(r#""replacement":"\"\\\u000a\u000d\u0009\u0000文""#));
    assert!(!wire.chars().any(|ch| ch < ' '));
    assert_eq!(
        wire,
        index
            .serialize_literal_replacement_plan(&plan, wire.len())
            .unwrap()
    );
    assert_eq!(
        index.serialize_literal_replacement_plan(&plan, wire.len() - 1),
        Err(IndexError::SerializationLimit)
    );
    assert_eq!(
        index.serialize_literal_replacement_plan(&plan, 0),
        Err(IndexError::SerializationLimit)
    );
}

#[test]
fn wire_rejects_malformed_ranges_and_overlaps_before_export() {
    let (index, plan) = setup();
    for change in 0..4 {
        let mut changed = plan.clone();
        match change {
            0 => changed.edits[0].source.start_byte = 1,
            1 => changed.edits[0].source.end_byte = usize::MAX,
            2 => changed.edits[0].source.start_byte = 5,
            _ => changed.edits.push(changed.edits[0].clone()),
        }
        assert_eq!(
            index.serialize_literal_replacement_plan(&changed, MAX_REPLACEMENT_WIRE_BYTES),
            Err(IndexError::InvalidSearchPlan)
        );
    }
}

#[test]
fn wire_guards_full_project_snapshot_even_for_selected_documents() {
    let (mut index, plan) = setup();
    index.replace_document("other.tex", 1, "unrelated").unwrap();
    assert_eq!(
        index.serialize_literal_replacement_plan(&plan, MAX_REPLACEMENT_WIRE_BYTES),
        Err(IndexError::StaleSnapshot)
    );
}
