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
use unicode_normalization::UnicodeNormalization;

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

// --- Case-sensitivity / normalization bypass of the reservation itself ---
//
// `is_reserved` originally compared the declared path against `.flashtex`
// with a plain byte-exact `==`/`starts_with`. macOS's default APFS is
// case-insensitive (and, separately, normalization-insensitive), so a
// spelling such as `.FlashTeX/Project.Lock` sailed straight through that
// check while the underlying rooted writer's write-then-`rename` still
// landed on the exact same directory entry as the real
// `.flashtex/project.lock` once written — confirmed by actually
// overwriting real bytes on disk. Each test below drives the full
// `apply_import` path against a *real* lock file holding known bytes and
// asserts both that the import is refused as `ReservedPath` and that
// those bytes are byte-for-byte unchanged afterward.

const REAL_LOCK_BYTES: &[u8] = b"REAL-LOCK-HOLDER-DO-NOT-OVERWRITE";
const ATTACKER_BYTES: &[u8] = b"attacker-controlled-bytes";

/// Writes `REAL_LOCK_BYTES` to the real, canonically-spelled
/// `.flashtex/project.lock` under a fresh target root, builds a one-entry
/// bundle named `entry_path` (holding `ATTACKER_BYTES`) from a fresh
/// source root, previews it, decides to write it, and asserts both that
/// `apply_import` refuses it as `ReservedPath(entry_path)` and that the
/// real lock file's bytes are unchanged on disk afterward.
fn assert_variant_is_refused_and_lock_is_untouched(label: &str, entry_path: &str) {
    let src = TempDir::new(&format!("reserved-variant-src-{label}"));
    src.write(entry_path, ATTACKER_BYTES);
    let src_root = ProjectRoot::new(src.path()).unwrap();

    let dst = TempDir::new(&format!("reserved-variant-dst-{label}"));
    let real_lock_path = dst.write(".flashtex/project.lock", REAL_LOCK_BYTES);
    let dst_root = ProjectRoot::new(dst.path()).unwrap();

    let bundle = build_bundle(&src_root, &[BundleEntry::new(entry_path)]).unwrap();
    let preview = preview_import(&bundle, &dst_root).unwrap();

    let mut decisions = HashMap::new();
    decisions.insert(entry_path.to_string(), ImportDecision::Write);
    let err = apply_import(&bundle, &preview, &dst_root, &decisions).unwrap_err();
    assert_eq!(
        err,
        BundleError::ReservedPath(entry_path.to_string()),
        "{label}: a variant spelling of the control directory must be refused just like the exact spelling"
    );

    assert_eq!(
        std::fs::read(&real_lock_path).unwrap(),
        REAL_LOCK_BYTES,
        "{label}: the real .flashtex/project.lock bytes must be unchanged after the refused import"
    );
}

#[test]
fn reserved_path_exact_spelling_is_refused_and_lock_is_untouched() {
    assert_variant_is_refused_and_lock_is_untouched("exact", ".flashtex/project.lock");
}

#[test]
fn reserved_path_ascii_case_variant_is_refused_and_lock_is_untouched() {
    // Every letter of the control directory and the lock filename recased.
    // This is the confirmed bypass of the pre-fix check.
    assert_variant_is_refused_and_lock_is_untouched("ascii-case", ".FlashTeX/Project.Lock");
}

#[test]
fn reserved_path_unicode_nfd_variant_is_refused_and_lock_is_untouched() {
    // `.flashtex/project.lock` is pure ASCII, and Unicode canonical
    // decomposition (NFD) has no entries for plain ASCII code points, so
    // running it through NFD is provably a byte-for-byte no-op for this
    // exact literal — asserted below rather than assumed. There is
    // therefore no *separate* normalization-only bypass of this literal,
    // distinct from the case-sensitivity one above; this test documents
    // that fact and exercises the fixed check's NFC-normalization step on
    // normalization-neutral input, in the same spirit as the
    // `AmbiguousPath` NFC/NFD pair in `tests/malformed_and_unicode.rs`
    // (which uses an accented filename, where NFC/NFD genuinely differ).
    let nfd_spelling: String = ".flashtex/project.lock".nfd().collect();
    assert_eq!(
        nfd_spelling, ".flashtex/project.lock",
        "sanity check: NFD of a pure-ASCII path must be byte-identical to the path itself"
    );
    assert_variant_is_refused_and_lock_is_untouched("unicode-nfd", &nfd_spelling);
}

#[test]
fn reserved_path_case_and_normalization_combination_is_refused_and_lock_is_untouched() {
    // The ASCII case variant, additionally round-tripped through NFD then
    // NFC. For this pure-ASCII literal the round-trip changes nothing
    // (asserted below, for the same reason as the previous test), so this
    // exercises the fixed check's case-folding and NFC-normalization
    // steps together on the same input, on top of the confirmed
    // case-sensitivity bypass.
    let case_variant = ".FlashTeX/Project.Lock";
    let combined: String = case_variant.nfd().collect::<String>().nfc().collect();
    assert_eq!(
        combined, case_variant,
        "sanity check: NFD/NFC round-trip of a pure-ASCII path must be byte-identical to the path itself"
    );
    assert_variant_is_refused_and_lock_is_untouched("case-and-normalization", &combined);
}
