//! No implicit discovery: the crate never globs or walks a directory to
//! decide what belongs in a bundle. An unlisted sibling file must never
//! appear in the output, no matter how "obviously" related it looks.

mod common;

use common::TempDir;
use flashtex_project_bundle::{BundleEntry, ProjectRoot, build_bundle};

#[test]
fn unlisted_sibling_file_never_appears_in_bundle() {
    let dir = TempDir::new("no-discovery");
    dir.write(
        "main.tex",
        b"\\documentclass{article}\\begin{document}\\end{document}",
    );
    // A sibling that a directory-walking implementation would happily pick
    // up (same directory, same extension, alphabetically adjacent).
    dir.write("main2.tex", b"should never be bundled");
    dir.write("chapters/intro.tex", b"chapter one");
    // Another unlisted file nested next to a listed one.
    dir.write("chapters/notes.tex", b"private notes, not requested");

    let root = ProjectRoot::new(dir.path()).unwrap();
    let entries = [
        BundleEntry::new("main.tex"),
        BundleEntry::new("chapters/intro.tex"),
    ];
    let bundle = build_bundle(&root, &entries).unwrap();

    let paths: Vec<&str> = bundle.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, vec!["chapters/intro.tex", "main.tex"]);
    assert!(!paths.contains(&"main2.tex"));
    assert!(!paths.contains(&"chapters/notes.tex"));

    let manifest = String::from_utf8(bundle.manifest_bytes()).unwrap();
    assert!(!manifest.contains("main2.tex"));
    assert!(!manifest.contains("notes.tex"));
}

#[test]
fn empty_entry_list_produces_empty_bundle_even_with_files_present() {
    let dir = TempDir::new("no-discovery-empty");
    dir.write("main.tex", b"content");
    dir.write("other.tex", b"other content");

    let root = ProjectRoot::new(dir.path()).unwrap();
    let bundle = build_bundle(&root, &[]).unwrap();
    assert!(bundle.files.is_empty());
    assert!(bundle.manifest_bytes().is_empty());
}
