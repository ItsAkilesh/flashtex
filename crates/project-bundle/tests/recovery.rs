//! Batch recovery: `apply_import` is not atomic at the level of the reused
//! single-file writer (`flashtex_project_files::ProjectLock::save` only
//! promises one file's write is atomic), so this crate builds batch
//! recoverability on top of it — rolling back every write already
//! committed earlier in the same `apply_import` call when a later file in
//! that call fails. These tests inject a real failure (an out-of-contract
//! writer racing the target between preview and apply, exactly like
//! `tests/apply.rs`'s existing single-file race tests) at the first, the
//! middle, and the last of several writes, and assert precisely what
//! survives: everything this call had already written is undone, the
//! failing path is left exactly as the race left it (never touched by this
//! call), and every path after it was never attempted at all.

mod common;

use std::collections::HashMap;

use common::TempDir;
use flashtex_project_bundle::{
    apply_import, build_bundle, preview_import, BundleEntry, BundleError, ImportDecision,
    ProjectRoot,
};

/// Three new files, sorted by path (the order `apply_import` processes
/// them in), none present in the target yet.
fn setup_three_new_files() -> (TempDir, ProjectRoot, TempDir, ProjectRoot) {
    let src = TempDir::new("recovery-src");
    src.write("a.tex", b"content a");
    src.write("m.tex", b"content m");
    src.write("z.tex", b"content z");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("recovery-dst");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    (src, src_root, dst, dst_root)
}

fn bundle_and_preview(
    src_root: &ProjectRoot,
    dst_root: &ProjectRoot,
) -> (
    flashtex_project_bundle::Bundle,
    flashtex_project_bundle::ImportPreview,
) {
    let bundle = build_bundle(
        src_root,
        &[
            BundleEntry::new("a.tex"),
            BundleEntry::new("m.tex"),
            BundleEntry::new("z.tex"),
        ],
    )
    .unwrap();
    let preview = preview_import(&bundle, dst_root).unwrap();
    (bundle, preview)
}

#[test]
fn failure_on_the_first_write_leaves_nothing_written() {
    let (_src, src_root, dst, dst_root) = setup_three_new_files();
    let (bundle, preview) = bundle_and_preview(&src_root, &dst_root);

    // Race: something outside this call's contract creates "a.tex" (the
    // first file apply_import will attempt) between preview and apply.
    dst.write("a.tex", b"raced in before any write");

    let err = apply_import(&bundle, &preview, &dst_root, &HashMap::new()).unwrap_err();
    assert!(
        matches!(err, BundleError::ConcurrentModification { ref path, .. } if path == "a.tex"),
        "{err:?}"
    );

    // Nothing had been written yet, so there is nothing to roll back: the
    // error is the plain typed cause, not `RollbackIncomplete`.
    assert_eq!(
        std::fs::read(dst.path().join("a.tex")).unwrap(),
        b"raced in before any write",
        "the racing writer's content must be left alone"
    );
    assert!(!dst.path().join("m.tex").exists(), "never attempted");
    assert!(!dst.path().join("z.tex").exists(), "never attempted");
}

#[test]
fn failure_on_the_middle_write_rolls_back_the_first() {
    let (_src, src_root, dst, dst_root) = setup_three_new_files();
    let (bundle, preview) = bundle_and_preview(&src_root, &dst_root);

    // Race lands on "m.tex", the second (middle) of three files.
    dst.write("m.tex", b"raced in at the middle");

    let err = apply_import(&bundle, &preview, &dst_root, &HashMap::new()).unwrap_err();
    assert!(
        matches!(err, BundleError::ConcurrentModification { ref path, .. } if path == "m.tex"),
        "{err:?}"
    );

    // "a.tex" was written by this call, then rolled back: it must not
    // exist any more (it did not exist before the call either).
    assert!(
        !dst.path().join("a.tex").exists(),
        "a.tex was written then must have been rolled back (removed)"
    );
    // "m.tex" — the failing path — was never touched by this call; the
    // racing writer's content stands.
    assert_eq!(
        std::fs::read(dst.path().join("m.tex")).unwrap(),
        b"raced in at the middle"
    );
    // "z.tex" comes after the failure point in iteration order: never
    // attempted at all.
    assert!(!dst.path().join("z.tex").exists(), "never attempted");
}

#[test]
fn failure_on_the_last_write_rolls_back_everything_before_it() {
    let (_src, src_root, dst, dst_root) = setup_three_new_files();
    let (bundle, preview) = bundle_and_preview(&src_root, &dst_root);

    // Race lands on "z.tex", the last of three files.
    dst.write("z.tex", b"raced in at the end");

    let err = apply_import(&bundle, &preview, &dst_root, &HashMap::new()).unwrap_err();
    assert!(
        matches!(err, BundleError::ConcurrentModification { ref path, .. } if path == "z.tex"),
        "{err:?}"
    );

    // Both earlier writes were committed by this call, then rolled back.
    assert!(!dst.path().join("a.tex").exists(), "rolled back");
    assert!(!dst.path().join("m.tex").exists(), "rolled back");
    // The failing path itself is left exactly as the race left it.
    assert_eq!(
        std::fs::read(dst.path().join("z.tex")).unwrap(),
        b"raced in at the end"
    );
}

#[test]
fn rollback_restores_overwritten_conflicts_not_just_removes_new_files() {
    // Same three-position shape, but every file is a *conflict* the caller
    // decided to overwrite, so rollback must restore original bytes, not
    // just remove a created file.
    let src = TempDir::new("recovery-restore-src");
    src.write("a.tex", b"new a");
    src.write("m.tex", b"new m");
    src.write("z.tex", b"new z");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("recovery-restore-dst");
    dst.write("a.tex", b"old a");
    dst.write("m.tex", b"old m");
    dst.write("z.tex", b"old z");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let bundle = build_bundle(
        &src_root,
        &[
            BundleEntry::new("a.tex"),
            BundleEntry::new("m.tex"),
            BundleEntry::new("z.tex"),
        ],
    )
    .unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    let mut decisions = HashMap::new();
    decisions.insert("a.tex".to_string(), ImportDecision::Write);
    decisions.insert("m.tex".to_string(), ImportDecision::Write);
    decisions.insert("z.tex".to_string(), ImportDecision::Write);

    // Race lands on the middle file, *after* the preview computed its
    // expected hash: someone else changes "m.tex" again in between.
    dst.write("m.tex", b"raced-in m");

    let err = apply_import(&bundle, &preview, &dst_root, &decisions).unwrap_err();
    assert!(
        matches!(err, BundleError::ConcurrentModification { ref path, .. } if path == "m.tex"),
        "{err:?}"
    );

    // "a.tex" was overwritten with the bundle's content, then rolled back
    // to its exact original bytes.
    assert_eq!(std::fs::read(dst.path().join("a.tex")).unwrap(), b"old a");
    // "m.tex" — the failing path — keeps the racing writer's content; this
    // call never wrote it.
    assert_eq!(
        std::fs::read(dst.path().join("m.tex")).unwrap(),
        b"raced-in m"
    );
    // "z.tex" was never attempted.
    assert_eq!(std::fs::read(dst.path().join("z.tex")).unwrap(), b"old z");
}

#[test]
fn rollback_removes_the_file_but_leaves_the_directory_it_created() {
    // Characterization, not a guarantee being added: the rooted writer
    // creates missing parent directories for a new file, and rollback has
    // no record of that — it removes the file it wrote and leaves the
    // now-empty directory behind, without reporting a rollback failure.
    //
    // This pins the real boundary of the batch-recovery promise: file
    // *content* is restored exactly, the directory tree is not. It is
    // pinned here so the crate docs (see `apply_import`'s "Batch recovery"
    // section) cannot drift back into claiming the target is left exactly
    // as it was in every respect.
    let src = TempDir::new("recovery-dirs-src");
    src.write("chapters/a.tex", b"content a");
    src.write("z.tex", b"content z");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("recovery-dirs-dst");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let bundle = build_bundle(
        &src_root,
        &[BundleEntry::new("chapters/a.tex"), BundleEntry::new("z.tex")],
    )
    .unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    // Race "z.tex" in so the second write fails and the first is rolled back.
    dst.write("z.tex", b"raced in");

    let err = apply_import(&bundle, &preview, &dst_root, &HashMap::new()).unwrap_err();
    assert!(
        matches!(err, BundleError::ConcurrentModification { ref path, .. } if path == "z.tex"),
        "{err:?}"
    );

    // The written file is gone...
    assert!(
        !dst.path().join("chapters/a.tex").exists(),
        "the file this call wrote must be rolled back"
    );
    // ...and the directory created to hold it remains, empty. Not reported
    // as a failure, because no file content is wrong.
    assert!(
        dst.path().join("chapters").is_dir(),
        "documented residue: the created directory is not removed"
    );
    assert_eq!(
        std::fs::read_dir(dst.path().join("chapters")).unwrap().count(),
        0,
        "and it is empty"
    );
}
