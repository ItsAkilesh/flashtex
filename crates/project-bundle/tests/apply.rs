//! Explicit, typed no-clobber import: a conflict is never overwritten
//! without an explicit caller decision, and an undecided path is a typed
//! error rather than a default.

mod common;

use std::collections::HashMap;

use common::TempDir;
use flashtex_project_bundle::{
    BundleEntry, BundleError, ImportAction, ImportDecision, ProjectRoot, apply_import,
    build_bundle, preview_import,
};

fn setup() -> (TempDir, ProjectRoot, TempDir, ProjectRoot) {
    let src = TempDir::new("apply-src");
    src.write("new.tex", b"brand new");
    src.write("same.tex", b"identical content");
    src.write("changed.tex", b"new version");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("apply-dst");
    dst.write("same.tex", b"identical content");
    dst.write("changed.tex", b"old version");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    (src, src_root, dst, dst_root)
}

#[test]
fn conflict_with_no_decision_is_a_typed_error_and_nothing_is_written() {
    let (_src, src_root, dst, dst_root) = setup();
    let bundle = build_bundle(
        &src_root,
        &[
            BundleEntry::new("new.tex"),
            BundleEntry::new("same.tex"),
            BundleEntry::new("changed.tex"),
        ],
    )
    .unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    // No decision at all for "changed.tex", the one conflict.
    let decisions = HashMap::new();
    let err = apply_import(&bundle, &preview, &dst_root, &decisions).unwrap_err();
    assert_eq!(
        err,
        BundleError::OverwriteNotDecided("changed.tex".to_string())
    );

    // Nothing was written: the omission must not act as partial progress.
    assert_eq!(
        std::fs::read(dst.path().join("changed.tex")).unwrap(),
        b"old version"
    );
}

#[test]
fn explicit_skip_on_a_conflict_leaves_the_target_untouched() {
    let (_src, src_root, dst, dst_root) = setup();
    let bundle = build_bundle(
        &src_root,
        &[
            BundleEntry::new("same.tex"),
            BundleEntry::new("changed.tex"),
        ],
    )
    .unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    let mut decisions = HashMap::new();
    decisions.insert("changed.tex".to_string(), ImportDecision::Skip);
    let outcomes = apply_import(&bundle, &preview, &dst_root, &decisions).unwrap();

    let changed = outcomes.iter().find(|o| o.path == "changed.tex").unwrap();
    assert_eq!(changed.action, ImportAction::Skipped);
    assert_eq!(
        std::fs::read(dst.path().join("changed.tex")).unwrap(),
        b"old version"
    );
}

#[test]
fn explicit_write_on_a_conflict_overwrites_with_the_bundle_content() {
    let (_src, src_root, dst, dst_root) = setup();
    let bundle = build_bundle(&src_root, &[BundleEntry::new("changed.tex")]).unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    let mut decisions = HashMap::new();
    decisions.insert("changed.tex".to_string(), ImportDecision::Write);
    let outcomes = apply_import(&bundle, &preview, &dst_root, &decisions).unwrap();

    match &outcomes[0].action {
        ImportAction::Written { sha256, bytes } => {
            assert_eq!(*bytes, "new version".len() as u64);
            assert_eq!(*sha256, bundle.files[0].sha256);
        }
        other => panic!("expected Written, got {other:?}"),
    }
    assert_eq!(
        std::fs::read(dst.path().join("changed.tex")).unwrap(),
        b"new version"
    );
}

#[test]
fn new_file_is_written_by_default_without_a_decision() {
    let (_src, src_root, dst, dst_root) = setup();
    let bundle = build_bundle(&src_root, &[BundleEntry::new("new.tex")]).unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    let outcomes = apply_import(&bundle, &preview, &dst_root, &HashMap::new()).unwrap();
    assert!(matches!(outcomes[0].action, ImportAction::Written { .. }));
    assert_eq!(
        std::fs::read(dst.path().join("new.tex")).unwrap(),
        b"brand new"
    );
}

#[test]
fn new_file_can_be_explicitly_skipped() {
    let (_src, src_root, dst, dst_root) = setup();
    let bundle = build_bundle(&src_root, &[BundleEntry::new("new.tex")]).unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    let mut decisions = HashMap::new();
    decisions.insert("new.tex".to_string(), ImportDecision::Skip);
    let outcomes = apply_import(&bundle, &preview, &dst_root, &decisions).unwrap();
    assert_eq!(outcomes[0].action, ImportAction::Skipped);
    assert!(!dst.path().join("new.tex").exists());
}

#[test]
fn unchanged_file_is_never_rewritten_even_with_an_explicit_write_decision() {
    let (_src, src_root, _dst, dst_root) = setup();
    let bundle = build_bundle(&src_root, &[BundleEntry::new("same.tex")]).unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    let mut decisions = HashMap::new();
    decisions.insert("same.tex".to_string(), ImportDecision::Write);
    let outcomes = apply_import(&bundle, &preview, &dst_root, &decisions).unwrap();
    assert_eq!(outcomes[0].action, ImportAction::Skipped);
}

#[test]
fn concurrent_modification_between_preview_and_apply_is_refused_not_clobbered() {
    let (_src, src_root, dst, dst_root) = setup();
    let bundle = build_bundle(&src_root, &[BundleEntry::new("changed.tex")]).unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    // Someone else writes to the target after the preview was computed but
    // before apply_import runs.
    dst.write("changed.tex", b"raced in");

    let mut decisions = HashMap::new();
    decisions.insert("changed.tex".to_string(), ImportDecision::Write);
    let err = apply_import(&bundle, &preview, &dst_root, &decisions).unwrap_err();
    assert!(
        matches!(err, BundleError::ConcurrentModification { .. }),
        "{err:?}"
    );
    assert_eq!(
        std::fs::read(dst.path().join("changed.tex")).unwrap(),
        b"raced in",
        "the racing writer's content must be left alone, not overwritten"
    );
}

#[test]
fn concurrently_created_new_file_is_refused_not_clobbered() {
    let (_src, src_root, dst, dst_root) = setup();
    let bundle = build_bundle(&src_root, &[BundleEntry::new("new.tex")]).unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    // Someone else creates the file after the preview said it was new.
    dst.write("new.tex", b"created meanwhile");

    let outcome = apply_import(&bundle, &preview, &dst_root, &HashMap::new()).unwrap_err();
    assert!(matches!(
        outcome,
        BundleError::ConcurrentModification { .. }
    ));
    assert_eq!(
        std::fs::read(dst.path().join("new.tex")).unwrap(),
        b"created meanwhile"
    );
}
