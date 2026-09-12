use flashtex_preview_controller::file_project::{DiskState, FileProject};
#[test]
fn disk_import_reopen_and_external_changes_preserve_durable_source() {
    let root = tempfile::tempdir().unwrap();
    let private = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("main.tex"), "\\input{chapter}").unwrap();
    std::fs::write(root.path().join("chapter.tex"), "original").unwrap();
    let (files, mut controller) =
        FileProject::open(root.path(), private.path(), "p", "main.tex").unwrap();
    assert_eq!(controller.index().snapshot().documents.len(), 2);
    assert!(matches!(
        files.inspect(&controller, "chapter.tex").unwrap(),
        DiskState::MatchesSource { .. }
    ));
    let old = controller.document("chapter.tex").unwrap().clone();
    controller
        .replace_document(
            "chapter.tex",
            old.revision,
            &old.source_sha256,
            "durable unsaved".into(),
        )
        .unwrap();
    std::fs::write(root.path().join("chapter.tex"), "external edit").unwrap();
    assert!(matches!(
        files.inspect(&controller, "chapter.tex").unwrap(),
        DiskState::DiffersFromSource { .. }
    ));
    assert!(files
        .export(&controller, "chapter.tex", Some(&old.source_sha256))
        .is_err());
    drop(controller);
    std::fs::remove_file(root.path().join("main.tex")).unwrap();
    let (files, controller) =
        FileProject::open(root.path(), private.path(), "p", "main.tex").unwrap();
    assert_eq!(
        controller.document("chapter.tex").unwrap().text,
        "durable unsaved"
    );
    assert_eq!(
        files.inspect(&controller, "main.tex").unwrap(),
        DiskState::Missing
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("chapter.tex")).unwrap(),
        "external edit"
    );
}
#[test]
fn same_project_id_under_different_roots_has_separate_ledgers() {
    let private = tempfile::tempdir().unwrap();
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    std::fs::write(a.path().join("main.tex"), "first").unwrap();
    std::fs::write(b.path().join("main.tex"), "second").unwrap();
    let (_, first) = FileProject::open(a.path(), private.path(), "p", "main.tex").unwrap();
    let (_, second) = FileProject::open(b.path(), private.path(), "p", "main.tex").unwrap();
    assert_eq!(first.document("main.tex").unwrap().text, "first");
    assert_eq!(second.document("main.tex").unwrap().text, "second");
}
#[test]
#[cfg(unix)]
fn outside_symlink_entry_is_refused_before_shared_graph_discovery() {
    let root = tempfile::tempdir().unwrap();
    let private = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("external.tex"), "outside").unwrap();
    std::os::unix::fs::symlink(
        outside.path().join("external.tex"),
        root.path().join("main.tex"),
    )
    .unwrap();
    assert!(FileProject::open(root.path(), private.path(), "p", "main.tex").is_err());
}
