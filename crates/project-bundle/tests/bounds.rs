//! Bounded bundle construction: entry-count and total-byte caps, each with
//! a typed error rather than silent truncation.

mod common;

use common::TempDir;
use flashtex_project_bundle::{
    BundleEntry, BundleError, BundleLimits, ProjectRoot, build_bundle_with_limits,
};

#[test]
fn too_many_entries_is_rejected_before_any_file_is_read() {
    let dir = TempDir::new("bounds-entries");
    // Only write 2 files but declare a limit of 1 with 2 entries named —
    // neither needs to exist for the entry-count check to fire first.
    dir.write("a.tex", b"a");
    dir.write("b.tex", b"b");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let entries = [BundleEntry::new("a.tex"), BundleEntry::new("b.tex")];
    let limits = BundleLimits {
        max_entries: 1,
        ..BundleLimits::default()
    };
    let err = build_bundle_with_limits(&root, &entries, &limits).unwrap_err();
    assert_eq!(
        err,
        BundleError::TooManyEntries {
            limit: 1,
            actual: 2
        }
    );
}

#[test]
fn entry_count_at_the_limit_is_accepted() {
    let dir = TempDir::new("bounds-entries-ok");
    dir.write("a.tex", b"a");
    let root = ProjectRoot::new(dir.path()).unwrap();
    let entries = [BundleEntry::new("a.tex")];
    let limits = BundleLimits {
        max_entries: 1,
        ..BundleLimits::default()
    };
    let bundle = build_bundle_with_limits(&root, &entries, &limits).unwrap();
    assert_eq!(bundle.files.len(), 1);
}

#[test]
fn total_bytes_exceeded_is_rejected_with_the_running_total() {
    let dir = TempDir::new("bounds-bytes");
    dir.write("a.tex", &[b'a'; 10]);
    dir.write("b.tex", &[b'b'; 10]);
    let root = ProjectRoot::new(dir.path()).unwrap();
    let entries = [BundleEntry::new("a.tex"), BundleEntry::new("b.tex")];
    let limits = BundleLimits {
        max_total_bytes: 15,
        ..BundleLimits::default()
    };
    let err = build_bundle_with_limits(&root, &entries, &limits).unwrap_err();
    assert_eq!(
        err,
        BundleError::TotalBytesExceeded {
            limit: 15,
            actual: 20
        }
    );
}

#[test]
fn total_bytes_at_the_limit_is_accepted() {
    let dir = TempDir::new("bounds-bytes-ok");
    dir.write("a.tex", &[b'a'; 10]);
    let root = ProjectRoot::new(dir.path()).unwrap();
    let entries = [BundleEntry::new("a.tex")];
    let limits = BundleLimits {
        max_total_bytes: 10,
        ..BundleLimits::default()
    };
    let bundle = build_bundle_with_limits(&root, &entries, &limits).unwrap();
    assert_eq!(bundle.files.len(), 1);
}

#[test]
fn single_file_over_the_root_file_limit_is_rejected() {
    let dir = TempDir::new("bounds-file-limit");
    dir.write("big.tex", &[b'x'; 100]);
    // A root whose per-file read limit is smaller than the file.
    let root = ProjectRoot::with_file_limit(dir.path(), 50).unwrap();
    let entries = [BundleEntry::new("big.tex")];
    let err = build_bundle_with_limits(&root, &entries, &BundleLimits::default()).unwrap_err();
    assert_eq!(
        err,
        BundleError::FileTooLarge {
            path: "big.tex".to_string(),
            limit: 50,
            size: 100
        }
    );
}
