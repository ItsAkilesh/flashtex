//! A declared path whose *parent directory* does not exist under the root
//! is simply absent, exactly like a missing leaf.
//!
//! The rooted reader walks the path one component at a time and turns a
//! missing leaf into `Ok(None)`, but a missing intermediate directory into
//! a raw `ENOENT` from the walk. This crate used to surface that verbatim
//! as an untyped `BundleError::Io`, which meant `preview_import` could not
//! classify a bundle entry such as `chapters/intro.tex` against a target
//! that did not have a `chapters/` directory yet — it failed the whole
//! preview with an I/O error, even though `apply_import`'s writer creates
//! missing parent directories. Importing a bundle with any nested file
//! into a fresh project was therefore impossible.

mod common;

use std::collections::HashMap;

use common::TempDir;
use flashtex_project_bundle::{
    BundleEntry, BundleError, FileOutcome, ProjectRoot, apply_import, build_bundle, preview_import,
};

#[test]
fn missing_parent_directory_reads_as_absent_not_an_io_error() {
    let dir = TempDir::new("missing-parent-read");
    let root = ProjectRoot::new(dir.path()).unwrap();

    assert_eq!(root.read_rooted_optional("a.tex").unwrap(), None);
    assert_eq!(root.read_rooted_optional("sub/a.tex").unwrap(), None);
    assert_eq!(root.read_rooted_optional("x/y/z/a.tex").unwrap(), None);
}

#[test]
fn missing_parent_directory_is_not_found_not_an_io_error() {
    let dir = TempDir::new("missing-parent-notfound");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let err = root.read_rooted("chapters/intro.tex").unwrap_err();
    assert_eq!(err, BundleError::NotFound("chapters/intro.tex".to_string()));
}

#[test]
fn a_real_traversal_or_symlink_is_still_a_typed_refusal_not_absence() {
    // Guard the fix's blast radius: mapping the walk's ENOENT to "absent"
    // must not have swallowed any of the security refusals, which are
    // different errno values entirely.
    let dir = TempDir::new("missing-parent-guard");
    let root = ProjectRoot::new(dir.path()).unwrap();
    assert!(matches!(
        root.read_rooted_optional("../outside.tex").unwrap_err(),
        BundleError::PathTraversal(_)
    ));

    #[cfg(unix)]
    {
        let outside = TempDir::new("missing-parent-guard-out");
        outside.write("secret.txt", b"outside");
        std::os::unix::fs::symlink(outside.path(), dir.path().join("link")).unwrap();
        assert!(matches!(
            root.read_rooted_optional("link/secret.txt").unwrap_err(),
            BundleError::SymlinkRefused(_)
        ));
    }
}

#[test]
fn nested_new_file_previews_as_new_and_imports_into_a_fresh_target() {
    let src = TempDir::new("missing-parent-src");
    src.write("chapters/intro.tex", b"intro");
    src.write("figures/deep/plot.svg", b"<svg/>");
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new("missing-parent-dst");
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let bundle = build_bundle(
        &src_root,
        &[
            BundleEntry::new("chapters/intro.tex"),
            BundleEntry::new("figures/deep/plot.svg"),
        ],
    )
    .unwrap();

    let preview = preview_import(&bundle, &dst_root).unwrap();
    assert_eq!(preview.files.len(), 2);
    for file in &preview.files {
        assert_eq!(file.outcome, FileOutcome::New, "{}", file.path);
    }

    apply_import(&bundle, &preview, &dst_root, &HashMap::new()).unwrap();
    assert_eq!(
        std::fs::read(dst.path().join("chapters/intro.tex")).unwrap(),
        b"intro"
    );
    assert_eq!(
        std::fs::read(dst.path().join("figures/deep/plot.svg")).unwrap(),
        b"<svg/>"
    );
}
