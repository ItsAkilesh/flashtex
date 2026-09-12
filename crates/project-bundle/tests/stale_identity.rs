//! Stale-identity acceptance: the specification, proven directly.
//!
//! An [`ImportPreview`](flashtex_project_bundle::ImportPreview) is a
//! snapshot of the SHA-256 hashes the target held at preview time. It is
//! valid only for as long as those hashes still hold: any change to a
//! previewed file between preview and apply — modified, deleted, or (for a
//! file the preview said was new) created — invalidates that file's part
//! of the preview. `apply_import` enforces this with a compare-and-swap on
//! every write (`Expected::Hash`/`Expected::NewFile` against the exact
//! hash the preview observed): a stale preview is refused with a typed
//! [`BundleError::ConcurrentModification`], never silently reapplied
//! against whatever is actually there by the time apply runs.
//!
//! "Modified" and "created" are proven in `tests/apply.rs`
//! (`concurrent_modification_between_preview_and_apply_is_refused_not_clobbered`,
//! `concurrently_created_new_file_is_refused_not_clobbered`). This file
//! proves the third way a previewed file can go stale — deletion — and
//! proves the property holds at the whole-batch level together with
//! `tests/recovery.rs`'s rollback: a single previewed file going stale by
//! deletion, mid-batch, still leaves every other write already committed
//! in that same call rolled back, not left standing.

mod common;

use std::collections::HashMap;

use common::TempDir;
use flashtex_project_bundle::{
    apply_import, build_bundle, preview_import, BundleEntry, BundleError, ImportDecision,
    ProjectRoot,
};

#[test]
fn conflict_target_deleted_between_preview_and_apply_is_refused_not_clobbered() {
    let src = TempDir::new("stale-deleted-src");
    src.write("doc.tex", b"new content");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("stale-deleted-dst");
    let target_path = dst.write("doc.tex", b"old content");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let bundle = build_bundle(&src_root, &[BundleEntry::new("doc.tex")]).unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    // The previewed file is deleted entirely before apply runs — a change
    // the preview's remembered hash cannot have observed, and one that
    // "was the target modified since preview" checks alone would not
    // exercise (there is no longer any content to compare a hash to).
    std::fs::remove_file(&target_path).unwrap();

    let mut decisions = HashMap::new();
    decisions.insert("doc.tex".to_string(), ImportDecision::Write);
    let err = apply_import(&bundle, &preview, &dst_root, &decisions).unwrap_err();
    assert!(
        matches!(err, BundleError::ConcurrentModification { ref path, .. } if path == "doc.tex"),
        "{err:?}"
    );
    assert!(
        !target_path.exists(),
        "refused, not recreated: the deletion stands"
    );
}

#[test]
fn one_previewed_file_deleted_mid_batch_still_rolls_back_every_earlier_write() {
    let src = TempDir::new("stale-deleted-batch-src");
    src.write("a.tex", b"new a");
    src.write("m.tex", b"new m");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("stale-deleted-batch-dst");
    dst.write("a.tex", b"old a");
    let m_path = dst.write("m.tex", b"old m");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let bundle = build_bundle(
        &src_root,
        &[BundleEntry::new("a.tex"), BundleEntry::new("m.tex")],
    )
    .unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    // "m.tex" (the second file apply_import will attempt) is deleted after
    // the preview observed it but before apply reaches it.
    std::fs::remove_file(&m_path).unwrap();

    let mut decisions = HashMap::new();
    decisions.insert("a.tex".to_string(), ImportDecision::Write);
    decisions.insert("m.tex".to_string(), ImportDecision::Write);
    let err = apply_import(&bundle, &preview, &dst_root, &decisions).unwrap_err();
    assert!(
        matches!(err, BundleError::ConcurrentModification { ref path, .. } if path == "m.tex"),
        "{err:?}"
    );

    // "a.tex" was written by this call, then rolled back to its original
    // bytes: the batch-recovery guarantee holds even though the failure
    // that triggered it was a deletion, not a content modification.
    assert_eq!(std::fs::read(dst.path().join("a.tex")).unwrap(), b"old a");
    // "m.tex" stays deleted: apply must not recreate what raced it away.
    assert!(!m_path.exists());
}
