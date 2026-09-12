use flashtex_edit_ledger::{
    history::{GroupedEdit, HistoryCommandStatus, HistoryMove, HistoryResult, SourceEdit},
    Document, Store,
};

fn same(full: &HistoryResult, compact: HistoryCommandStatus, store: &Store) {
    assert_eq!(full.command_revision, compact.command_revision);
    assert_eq!(full.replayed_command, compact.replayed_command);
    assert_eq!(full.can_undo, compact.can_undo);
    assert_eq!(full.can_redo, compact.can_redo);
    assert_eq!(
        serde_json::to_value(&full.document).unwrap(),
        serde_json::to_value(store.document().unwrap().unwrap()).unwrap()
    );
}
#[test]
fn compact_history_matches_full_state_and_permanent_retries() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let mut full = Store::open(a.path()).unwrap();
    let doc = Document::new("p".into(), "main.tex".into(), 1, "α".repeat(250_000)).unwrap();
    full.initialize(doc.clone()).unwrap();
    std::fs::copy(
        a.path().join("document.json"),
        b.path().join("document.json"),
    )
    .unwrap();
    let mut compact = Store::open(b.path()).unwrap();
    let group = GroupedEdit {
        command_id: "group".into(),
        expected_revision: 1,
        expected_sha256: doc.source_sha256.clone(),
        label: "Unicode".into(),
        edits: vec![SourceEdit {
            start_byte: 0,
            end_byte: 2,
            removed_text: "α".into(),
            replacement: "β".into(),
        }],
    };
    let f = full.apply_group(group.clone()).unwrap();
    same(
        &f,
        compact.apply_group_status(group.clone()).unwrap(),
        &compact,
    );
    let undo = HistoryMove {
        command_id: "undo".into(),
        expected_revision: 2,
        expected_sha256: f.document.source_sha256,
    };
    let f = full.undo(undo.clone()).unwrap();
    same(&f, compact.undo_status(undo.clone()).unwrap(), &compact);
    let redo = HistoryMove {
        command_id: "redo".into(),
        expected_revision: 3,
        expected_sha256: f.document.source_sha256,
    };
    let f = full.redo(redo.clone()).unwrap();
    same(&f, compact.redo_status(redo.clone()).unwrap(), &compact);
    for store in [&mut full, &mut compact] {
        store
            .replace_document(4, &f.document.source_sha256, "later".into())
            .unwrap();
    }
    drop(full);
    drop(compact);
    let mut full = Store::open(a.path()).unwrap();
    let mut compact = Store::open(b.path()).unwrap();
    same(
        &full.apply_group(group.clone()).unwrap(),
        compact.apply_group_status(group.clone()).unwrap(),
        &compact,
    );
    same(
        &full.undo(undo.clone()).unwrap(),
        compact.undo_status(undo.clone()).unwrap(),
        &compact,
    );
    same(
        &full.redo(redo.clone()).unwrap(),
        compact.redo_status(redo.clone()).unwrap(),
        &compact,
    );
    let mut conflict = group;
    conflict.label = "changed".into();
    assert_eq!(
        full.apply_group(conflict.clone()).unwrap_err().code,
        compact.apply_group_status(conflict).unwrap_err().code
    );
    let mut stale = undo;
    stale.command_id = "new-id".into();
    assert_eq!(
        full.undo(stale.clone()).unwrap_err().code,
        compact.undo_status(stale).unwrap_err().code
    );
    assert!(
        std::fs::read(a.path().join("document.json")).unwrap()
            == std::fs::read(b.path().join("document.json")).unwrap(),
        "persisted bytes differ"
    );
    assert_eq!(compact.document().unwrap().unwrap().revision, 5);
    assert_eq!(compact.document().unwrap().unwrap().text, "later");
}
