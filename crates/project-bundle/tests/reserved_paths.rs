//! Security: an import must never write into the project's own `.flashtex/`
//! control directory.
//!
//! `apply_import` holds `flashtex_project_files`' advisory lock —
//! `<root>/.flashtex/project.lock` — open for the whole batch, and the
//! rooted writer commits every file by writing a temp file and `rename`ing
//! it over the target. So importing a bundle entry named
//! `.flashtex/project.lock` replaces the *inode* the `flock` is held on:
//! the lock this call still believes it holds is stranded on an orphaned
//! inode, the new lock file is unlocked, and any other writer can take
//! "the lock" and interleave with the rest of this same batch. Before the
//! fix this succeeded silently and `apply_import` returned `Ok`.
//!
//! The same directory holds the crash-recovery journal
//! (`.flashtex/recovery/...`), so the whole prefix is reserved.

mod common;

use std::collections::HashMap;

use common::TempDir;
use flashtex_project_bundle::{
    BundleEntry, BundleError, ImportDecision, ProjectRoot, apply_import, build_bundle,
    preview_import,
};

/// The exact mechanism the reservation exists to prevent, characterized
/// against the underlying writer: replacing the lock file while holding it
/// voids mutual exclusion. This pins *why* `ReservedPath` must exist; it
/// exercises `flashtex_project_files`, not this crate's own logic.
#[test]
fn replacing_a_held_lock_file_voids_mutual_exclusion() {
    use flashtex_project_files::{Expected, ProjectPath, ProjectRoot as FilesRoot};

    let dir = TempDir::new("reserved-lock-mechanism");
    let a = FilesRoot::open(dir.path()).unwrap();
    let held = a.lock().unwrap();

    // Baseline: while the lock is held, nobody else can take it.
    let b = FilesRoot::open(dir.path()).unwrap();
    assert!(
        b.lock().is_err(),
        "exclusion must hold while the lock is held"
    );

    // Exactly what an import of a `.flashtex/project.lock` entry does.
    let lock_path = ProjectPath::normalize(".flashtex/project.lock").unwrap();
    held.save(&lock_path, b"imported", Expected::Any, true)
        .expect("the underlying writer does not itself refuse the lock path");

    // ...and now exclusion is gone, while the original holder still thinks
    // it owns the project.
    let c = FilesRoot::open(dir.path()).unwrap();
    assert!(
        c.lock().is_ok(),
        "replacing the lock file strands the flock on an orphaned inode"
    );
    drop(held);
}

#[test]
fn importing_over_the_project_lock_file_is_refused_before_any_write() {
    let src = TempDir::new("reserved-lock-src");
    src.write(".flashtex/project.lock", b"imported lock");
    src.write("a.tex", b"ordinary content");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("reserved-lock-dst");
    // A project that has been locked at least once already has this file.
    dst.write(".flashtex/project.lock", b"");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let bundle = build_bundle(
        &src_root,
        &[
            BundleEntry::new(".flashtex/project.lock"),
            BundleEntry::new("a.tex"),
        ],
    )
    .unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    let mut decisions = HashMap::new();
    decisions.insert(".flashtex/project.lock".to_string(), ImportDecision::Write);
    let err = apply_import(&bundle, &preview, &dst_root, &decisions).unwrap_err();
    assert_eq!(
        err,
        BundleError::ReservedPath(".flashtex/project.lock".to_string())
    );

    // Refused *before* any write in the batch: the lock file still holds
    // its own content and the ordinary file was never created.
    assert_eq!(
        std::fs::read(dst.path().join(".flashtex/project.lock")).unwrap(),
        b"",
        "the lock file must not have been overwritten"
    );
    assert!(
        !dst.path().join("a.tex").exists(),
        "nothing in the batch may be written once the batch is refused"
    );
}

#[test]
fn importing_over_the_recovery_journal_is_refused() {
    let src = TempDir::new("reserved-journal-src");
    src.write(".flashtex/recovery/deadbeef.json", b"{}");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("reserved-journal-dst");
    dst.write(".flashtex/recovery/placeholder", b"x");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let bundle = build_bundle(
        &src_root,
        &[BundleEntry::new(".flashtex/recovery/deadbeef.json")],
    )
    .unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();
    let err = apply_import(&bundle, &preview, &dst_root, &HashMap::new()).unwrap_err();
    assert_eq!(
        err,
        BundleError::ReservedPath(".flashtex/recovery/deadbeef.json".to_string())
    );
    assert!(!dst.path().join(".flashtex/recovery/deadbeef.json").exists());
}

#[test]
fn a_path_merely_resembling_the_control_directory_is_still_importable() {
    // The reservation is the `.flashtex/` prefix, not "anything containing
    // the word": a real project file must not be caught by it.
    let src = TempDir::new("reserved-lookalike-src");
    src.write(".flashtexrc", b"config");
    src.write("notes/.flashtex-notes.tex", b"notes");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("reserved-lookalike-dst");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let bundle = build_bundle(
        &src_root,
        &[
            BundleEntry::new(".flashtexrc"),
            BundleEntry::new("notes/.flashtex-notes.tex"),
        ],
    )
    .unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();
    apply_import(&bundle, &preview, &dst_root, &HashMap::new()).unwrap();
    assert_eq!(
        std::fs::read(dst.path().join(".flashtexrc")).unwrap(),
        b"config"
    );
}
