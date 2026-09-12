//! Bounded malformed-input and Unicode-filename coverage.

mod common;

use common::TempDir;
use flashtex_project_bundle::{build_bundle, BundleEntry, BundleError, ProjectRoot};

#[test]
fn empty_path_is_rejected() {
    let dir = TempDir::new("malformed-empty");
    let root = ProjectRoot::new(dir.path()).unwrap();
    assert_eq!(root.read_rooted("").unwrap_err(), BundleError::EmptyPath);
}

#[test]
fn null_byte_in_path_is_rejected() {
    let dir = TempDir::new("malformed-nul");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let err = root.read_rooted("foo\0bar").unwrap_err();
    assert!(matches!(err, BundleError::MalformedPath(_)));
}

#[test]
fn double_slash_empty_component_is_rejected() {
    let dir = TempDir::new("malformed-double-slash");
    dir.write("foo/bar.tex", b"x");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let err = root.read_rooted("foo//bar.tex").unwrap_err();
    assert!(matches!(err, BundleError::MalformedPath(_)));
}

#[test]
fn trailing_slash_is_rejected() {
    let dir = TempDir::new("malformed-trailing-slash");
    dir.write("foo.tex", b"x");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let err = root.read_rooted("foo.tex/").unwrap_err();
    assert!(matches!(err, BundleError::MalformedPath(_)));
}

#[test]
fn single_dot_component_is_rejected() {
    let dir = TempDir::new("malformed-dot");
    dir.write("foo.tex", b"x");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let err = root.read_rooted("./foo.tex").unwrap_err();
    assert!(matches!(err, BundleError::MalformedPath(_)));
}

#[test]
fn bare_dot_dot_is_traversal_not_malformed() {
    let dir = TempDir::new("malformed-bare-dotdot");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let err = root.read_rooted("..").unwrap_err();
    assert_eq!(err, BundleError::PathTraversal("..".to_string()));
}

#[test]
fn duplicate_entries_are_rejected() {
    let dir = TempDir::new("malformed-duplicate");
    dir.write("main.tex", b"x");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let entries = [BundleEntry::new("main.tex"), BundleEntry::new("main.tex")];
    let err = build_bundle(&root, &entries).unwrap_err();
    assert_eq!(err, BundleError::DuplicatePath("main.tex".to_string()));
}

#[test]
fn non_ascii_filenames_round_trip_through_the_bundle() {
    let dir = TempDir::new("unicode");
    dir.write("café.tex", "contenu français".as_bytes());
    dir.write("日本語のファイル.tex", "内容".as_bytes());
    dir.write("emoji-\u{1F4C4}.txt", b"page emoji");

    let root = ProjectRoot::new(dir.path()).unwrap();
    let entries = [
        BundleEntry::new("café.tex"),
        BundleEntry::new("日本語のファイル.tex"),
        BundleEntry::new("emoji-\u{1F4C4}.txt"),
    ];
    let bundle = build_bundle(&root, &entries).unwrap();
    assert_eq!(bundle.files.len(), 3);

    let cafe = bundle
        .files
        .iter()
        .find(|f| f.path == "café.tex")
        .expect("café.tex present");
    assert_eq!(cafe.contents, "contenu français".as_bytes());

    // Ordering must be a plain byte sort of the UTF-8 path, so building
    // again from a shuffled entry list gives the identical manifest.
    let shuffled = [
        BundleEntry::new("emoji-\u{1F4C4}.txt"),
        BundleEntry::new("café.tex"),
        BundleEntry::new("日本語のファイル.tex"),
    ];
    let bundle2 = build_bundle(&root, &shuffled).unwrap();
    assert_eq!(bundle.manifest_bytes(), bundle2.manifest_bytes());
}

#[test]
fn non_ascii_directory_and_file_names_are_rooted_normally() {
    let dir = TempDir::new("unicode-nested");
    dir.write("章/一.tex", "第一章".as_bytes());
    let root = ProjectRoot::new(dir.path()).unwrap();
    let bytes = root.read_rooted("章/一.tex").unwrap();
    assert_eq!(bytes, "第一章".as_bytes());
}
