//! Import preview: new / unchanged / conflict classification, full hashes
//! on both sides, and — the load-bearing property — zero writes.

mod common;

use std::time::{Duration, SystemTime};

use common::TempDir;
use flashtex_project_bundle::{
    build_bundle, preview_import, BundleEntry, FileOutcome, ProjectRoot,
};

fn snapshot(dir: &std::path::Path) -> Vec<(String, u64, SystemTime)> {
    fn walk(dir: &std::path::Path, prefix: &str, out: &mut Vec<(String, u64, SystemTime)>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap()).collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let meta = entry.metadata().unwrap();
            let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
            if meta.is_dir() {
                walk(&entry.path(), &format!("{name}/"), out);
            } else {
                out.push((name, meta.len(), meta.modified().unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, "", &mut out);
    out.sort();
    out
}

#[test]
fn preview_classifies_new_unchanged_and_conflicting_files() {
    let src = TempDir::new("preview-src");
    src.write("new.tex", b"brand new");
    src.write("same.tex", b"identical content");
    src.write("changed.tex", b"new version");
    let src_root = ProjectRoot::new(src.path()).unwrap();
    let bundle = build_bundle(
        &src_root,
        &[
            BundleEntry::new("new.tex"),
            BundleEntry::new("same.tex"),
            BundleEntry::new("changed.tex"),
        ],
    )
    .unwrap();

    let dst = TempDir::new("preview-dst");
    dst.write("same.tex", b"identical content");
    dst.write("changed.tex", b"old version");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let preview = preview_import(&bundle, &dst_root).unwrap();
    assert_eq!(preview.files.len(), 3);

    let outcome = |path: &str| preview.files.iter().find(|f| f.path == path).unwrap().outcome;
    assert!(matches!(outcome("new.tex"), FileOutcome::New));
    assert!(matches!(outcome("same.tex"), FileOutcome::Unchanged { .. }));
    match outcome("changed.tex") {
        FileOutcome::Conflict {
            ours,
            theirs,
            theirs_size,
        } => {
            assert_ne!(ours, theirs);
            assert_eq!(theirs_size, "old version".len() as u64);
        }
        other => panic!("expected Conflict, got {other:?}"),
    }

    assert!(preview.has_conflicts());
    assert_eq!(preview.conflicts().count(), 1);
    assert_eq!(preview.new_paths().collect::<Vec<_>>(), vec!["new.tex"]);
    assert_eq!(
        preview.unchanged_paths().collect::<Vec<_>>(),
        vec!["same.tex"]
    );
}

#[test]
fn preview_hashes_distinguish_identical_from_differing_content() {
    // Same size, different bytes: a size/mtime-only comparison would miss
    // this; the full SHA-256 must not.
    let src = TempDir::new("preview-hash-src");
    src.write("f.tex", b"AAAAAAAAAA");
    let src_root = ProjectRoot::new(src.path()).unwrap();
    let bundle = build_bundle(&src_root, &[BundleEntry::new("f.tex")]).unwrap();

    let dst = TempDir::new("preview-hash-dst");
    dst.write("f.tex", b"BBBBBBBBBB");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let preview = preview_import(&bundle, &dst_root).unwrap();
    assert!(preview.has_conflicts());
}

#[test]
fn preview_performs_no_writes_at_all() {
    let src = TempDir::new("preview-nowrite-src");
    src.write("new.tex", b"brand new");
    src.write("changed.tex", b"new version");
    let src_root = ProjectRoot::new(src.path()).unwrap();
    let bundle = build_bundle(
        &src_root,
        &[BundleEntry::new("new.tex"), BundleEntry::new("changed.tex")],
    )
    .unwrap();

    let dst = TempDir::new("preview-nowrite-dst");
    dst.write("changed.tex", b"old version");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    // Force filesystem timestamp resolution apart from "now" so a
    // millisecond-level accidental touch would show up.
    std::thread::sleep(Duration::from_millis(20));

    let before = snapshot(dst.path());
    let preview = preview_import(&bundle, &dst_root).unwrap();
    let after = snapshot(dst.path());

    assert!(preview.has_conflicts());
    assert_eq!(
        before, after,
        "preview_import must not create, modify or touch any file"
    );
    assert!(
        !dst.path().join(".flashtex").exists(),
        "preview must never take the project lock (which lazily creates .flashtex/)"
    );
    assert_eq!(
        std::fs::read(dst.path().join("changed.tex")).unwrap(),
        b"old version",
        "the conflicting file's bytes must be exactly what they were before the preview"
    );
    assert!(
        !dst.path().join("new.tex").exists(),
        "a file only present in the bundle must not be created by preview"
    );
}
