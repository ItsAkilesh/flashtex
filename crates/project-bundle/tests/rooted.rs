//! Security: every read is rooted, via `flashtex_project_files`'s
//! `openat(O_NOFOLLOW)`-based reader. A `..` traversal, an absolute path,
//! and any symlink (whether or not it would resolve outside the root) must
//! each be rejected with a typed error, not silently clamped or followed.

mod common;

use common::TempDir;
use flashtex_project_bundle::{BundleError, ProjectRoot};

#[test]
fn valid_relative_path_reads_the_file() {
    let dir = TempDir::new("rooted-ok");
    dir.write("main.tex", b"\\documentclass{article}");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let bytes = root.read_rooted("main.tex").unwrap();
    assert_eq!(bytes, b"\\documentclass{article}");
}

#[test]
fn valid_nested_relative_path_reads_the_file() {
    let dir = TempDir::new("rooted-nested");
    dir.write("chapters/intro.tex", b"intro");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let bytes = root.read_rooted("chapters/intro.tex").unwrap();
    assert_eq!(bytes, b"intro");
}

#[test]
fn dot_dot_traversal_is_rejected() {
    let dir = TempDir::new("rooted-dotdot");
    // A real file that exists outside the root, so a naive implementation
    // relying only on "does canonicalize succeed" would happily read it.
    dir.write("inner/placeholder", b"noop");
    let outside = TempDir::new("rooted-dotdot-outside");
    let secret = outside.write("secret.txt", b"outside contents");
    assert!(secret.exists());

    let root = ProjectRoot::new(dir.path()).unwrap();
    let escape = format!(
        "../{}/secret.txt",
        outside.path().file_name().unwrap().to_str().unwrap()
    );
    let err = root.read_rooted(&escape).unwrap_err();
    assert_eq!(err, BundleError::PathTraversal(escape));
}

#[test]
fn dot_dot_traversal_is_rejected_even_when_target_does_not_exist() {
    // The rejection must be syntactic, not "canonicalize failed to find
    // it" — it must fire before any filesystem lookup.
    let dir = TempDir::new("rooted-dotdot-missing");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let err = root.read_rooted("../nonexistent-should-never-be-touched").unwrap_err();
    assert!(matches!(err, BundleError::PathTraversal(_)));
}

#[test]
fn absolute_path_is_rejected() {
    let dir = TempDir::new("rooted-abs");
    dir.write("main.tex", b"content");
    let root = ProjectRoot::new(dir.path()).unwrap();

    // An absolute path that happens to point at the very file that *is*
    // inside the root must still be rejected: absoluteness alone is
    // disqualifying.
    let absolute = dir.path().join("main.tex");
    let absolute_str = absolute.to_str().unwrap().to_string();
    let err = root.read_rooted(&absolute_str).unwrap_err();
    assert_eq!(err, BundleError::AbsolutePath(absolute_str));
}

#[cfg(unix)]
#[test]
fn symlink_escaping_root_is_rejected() {
    use std::os::unix::fs::symlink;

    let dir = TempDir::new("rooted-symlink-in");
    let outside = TempDir::new("rooted-symlink-out");
    let secret = outside.write("secret.txt", b"outside contents");

    let link = dir.path().join("link.txt");
    symlink(&secret, &link).unwrap();

    let root = ProjectRoot::new(dir.path()).unwrap();
    let err = root.read_rooted("link.txt").unwrap_err();
    assert_eq!(err, BundleError::SymlinkRefused("link.txt".to_string()));
}

// Rev 1's own canonicalize-based rooting resolved a symlink and only
// rejected it if the resolved target landed outside the root, so a link
// staying inside the root was followed. The reused `flashtex_project_files`
// reader is stricter: it opens every path component with
// `openat(O_NOFOLLOW)` and refuses *any* symlink outright, whether or not
// it would resolve inside the root. That is a tightening, not a gap — it
// still rejects every escape rev 1 rejected, plus more — so this case is
// now also refused rather than "allowed".
#[cfg(unix)]
#[test]
fn symlink_staying_inside_root_is_also_refused() {
    use std::os::unix::fs::symlink;

    let dir = TempDir::new("rooted-symlink-inside-refused");
    dir.write("real.txt", b"real contents");
    let link = dir.path().join("link.txt");
    symlink(dir.path().join("real.txt"), &link).unwrap();

    let root = ProjectRoot::new(dir.path()).unwrap();
    let err = root.read_rooted("link.txt").unwrap_err();
    assert_eq!(err, BundleError::SymlinkRefused("link.txt".to_string()));
}

#[test]
fn directory_target_is_rejected_not_a_file() {
    let dir = TempDir::new("rooted-dir-target");
    dir.write("subdir/placeholder.txt", b"x");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let err = root.read_rooted("subdir").unwrap_err();
    assert_eq!(err, BundleError::NotAFile("subdir".to_string()));
}

#[test]
fn missing_file_is_not_found() {
    let dir = TempDir::new("rooted-missing");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let err = root.read_rooted("nope.tex").unwrap_err();
    assert_eq!(err, BundleError::NotFound("nope.tex".to_string()));
}

#[test]
fn root_must_be_an_existing_directory() {
    let dir = TempDir::new("rooted-root-check");
    let file = dir.write("not-a-dir.txt", b"x");
    assert!(ProjectRoot::new(&file).is_err());
    assert!(ProjectRoot::new(dir.path().join("missing")).is_err());
}
