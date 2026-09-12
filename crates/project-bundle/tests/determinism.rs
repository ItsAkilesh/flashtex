//! Determinism: the same inputs must produce a byte-identical manifest
//! every time, under a total ordering that does not depend on filesystem
//! iteration order or hash-map order.

mod common;

use common::TempDir;
use flashtex_project_bundle::{BundleEntry, ProjectRoot, build_bundle};

fn sample_root() -> TempDir {
    let dir = TempDir::new("determinism");
    dir.write("main.tex", b"\\documentclass{article}");
    dir.write("chapters/a.tex", b"chapter a");
    dir.write("chapters/b.tex", b"chapter b");
    dir.write("refs.bib", b"@article{x, title={y}}");
    dir
}

#[test]
fn building_twice_yields_identical_manifest_bytes() {
    let dir = sample_root();
    let root = ProjectRoot::new(dir.path()).unwrap();
    let entries = [
        BundleEntry::new("main.tex"),
        BundleEntry::new("chapters/a.tex"),
        BundleEntry::new("chapters/b.tex"),
        BundleEntry::new("refs.bib"),
    ];

    let first = build_bundle(&root, &entries).unwrap();
    let second = build_bundle(&root, &entries).unwrap();

    assert_eq!(first.manifest_bytes(), second.manifest_bytes());
    assert_eq!(first.manifest_sha256(), second.manifest_sha256());
}

#[test]
fn input_order_does_not_affect_manifest_bytes() {
    let dir = sample_root();
    let root = ProjectRoot::new(dir.path()).unwrap();

    let forward = [
        BundleEntry::new("main.tex"),
        BundleEntry::new("chapters/a.tex"),
        BundleEntry::new("chapters/b.tex"),
        BundleEntry::new("refs.bib"),
    ];
    let shuffled = [
        BundleEntry::new("refs.bib"),
        BundleEntry::new("chapters/b.tex"),
        BundleEntry::new("main.tex"),
        BundleEntry::new("chapters/a.tex"),
    ];

    let a = build_bundle(&root, &forward).unwrap();
    let b = build_bundle(&root, &shuffled).unwrap();

    assert_eq!(a.manifest_bytes(), b.manifest_bytes());
    assert_eq!(a.manifest_hex(), b.manifest_hex());
}

#[test]
fn manifest_ordering_is_a_plain_byte_sort_of_path_not_locale_aware() {
    // Byte-wise ordering, not any locale/collation-aware ordering: an
    // uppercase "Z" (0x5A) sorts before a lowercase "a" (0x61), which a
    // locale-aware sort would typically invert.
    let dir = TempDir::new("determinism-byte-order");
    dir.write("Zebra.tex", b"z");
    dir.write("apple.tex", b"a");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let entries = [BundleEntry::new("apple.tex"), BundleEntry::new("Zebra.tex")];

    let bundle = build_bundle(&root, &entries).unwrap();
    let paths: Vec<&str> = bundle.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, vec!["Zebra.tex", "apple.tex"]);
}

#[test]
fn manifest_format_is_stable_and_readable() {
    let dir = TempDir::new("determinism-format");
    dir.write("a.tex", b"hello");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let bundle = build_bundle(&root, &[BundleEntry::new("a.tex")]).unwrap();

    let manifest = String::from_utf8(bundle.manifest_bytes()).unwrap();
    // sha256("hello"), verified independently via `shasum -a 256`.
    let expected_sha = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
    assert_eq!(manifest, format!("{expected_sha}  5  a.tex\n"));
}
