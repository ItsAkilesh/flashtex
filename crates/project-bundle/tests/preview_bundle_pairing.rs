//! `apply_import` takes the bundle and the preview as two independent
//! arguments, and nothing structurally ties them together. A caller that
//! rebuilds (or swaps) the bundle after computing the preview must get a
//! typed error, not a panic — and must not get one *after* part of the
//! batch is already on disk.
//!
//! Before the fix, the mismatched path hit
//! `expect("preview built from this bundle")` inside the per-file write
//! loop: the panic unwound straight out of `apply_import`, past the
//! rollback of every write already committed in that same call, leaving
//! them standing on disk with no error value for the caller to inspect.

mod common;

use std::collections::HashMap;

use common::TempDir;
use flashtex_project_bundle::{
    apply_import, build_bundle, preview_import, BundleEntry, BundleError, ImportDecision,
    ProjectRoot,
};

#[test]
fn preview_naming_a_path_the_bundle_lacks_is_a_typed_error_not_a_panic() {
    let src = TempDir::new("pairing-src");
    src.write("a.tex", b"content a");
    src.write("m.tex", b"content m");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("pairing-dst");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    // Preview computed from the wide bundle...
    let wide = build_bundle(
        &src_root,
        &[BundleEntry::new("a.tex"), BundleEntry::new("m.tex")],
    )
    .unwrap();
    let preview = preview_import(&wide, &dst_root).unwrap();

    // ...then the caller narrows the bundle (user deselected "m.tex") and
    // reuses the preview it already computed.
    let narrow = build_bundle(&src_root, &[BundleEntry::new("a.tex")]).unwrap();

    let err = apply_import(&narrow, &preview, &dst_root, &HashMap::new()).unwrap_err();
    assert_eq!(
        err,
        BundleError::PreviewBundleMismatch("m.tex".to_string())
    );

    // "a.tex" sorts before "m.tex", so it is the file the pre-fix panic
    // left committed. The check now runs before the batch starts, so
    // nothing at all was written.
    assert!(
        !dst.path().join("a.tex").exists(),
        "the mismatch must be caught before any file in the batch is written"
    );
    assert!(!dst.path().join("m.tex").exists());
}

#[test]
fn mismatch_is_detected_even_when_it_is_the_very_first_previewed_path() {
    let src = TempDir::new("pairing-first-src");
    src.write("a.tex", b"content a");
    src.write("z.tex", b"content z");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("pairing-first-dst");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let wide = build_bundle(
        &src_root,
        &[BundleEntry::new("a.tex"), BundleEntry::new("z.tex")],
    )
    .unwrap();
    let preview = preview_import(&wide, &dst_root).unwrap();
    let narrow = build_bundle(&src_root, &[BundleEntry::new("z.tex")]).unwrap();

    let err = apply_import(&narrow, &preview, &dst_root, &HashMap::new()).unwrap_err();
    assert_eq!(err, BundleError::PreviewBundleMismatch("a.tex".to_string()));
    assert!(!dst.path().join("z.tex").exists());
}

#[test]
fn a_matched_bundle_and_preview_still_apply_normally() {
    // The new up-front check must not reject the ordinary paired case,
    // including files the caller explicitly skips.
    let src = TempDir::new("pairing-ok-src");
    src.write("a.tex", b"content a");
    src.write("m.tex", b"content m");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("pairing-ok-dst");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let bundle = build_bundle(
        &src_root,
        &[BundleEntry::new("a.tex"), BundleEntry::new("m.tex")],
    )
    .unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    let mut decisions = HashMap::new();
    decisions.insert("m.tex".to_string(), ImportDecision::Skip);
    let outcomes = apply_import(&bundle, &preview, &dst_root, &decisions).unwrap();
    assert_eq!(outcomes.len(), 2);
    assert_eq!(std::fs::read(dst.path().join("a.tex")).unwrap(), b"content a");
    assert!(!dst.path().join("m.tex").exists());
}
