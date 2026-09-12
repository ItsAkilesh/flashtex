//! Acceptance specification for [`CreationRecord`]: the receipt `instantiate`
//! returns for each file it writes.
//!
//! A `CreationRecord` is a point-in-time proof, not a live view: it carries
//! the sha-256 hash and byte length a file held *at the moment it was
//! written*, taken from the rooted writer's own post-write verification
//! (never recomputed by re-reading afterward — see `crate::instantiate`).
//! This suite specifies exactly what that guarantees, and — just as
//! precisely — what it stops guaranteeing the instant anything else touches
//! the file. Read each block below as Given/When/Then; the test underneath it
//! is the executable form of the same sentence.
//!
//! Given a template is instantiated,
//!     the returned `CreationRecord` for every written file
//!     must report the exact sha-256 hash and byte length the file holds on
//!     disk at that moment.
//!     -> `receipt_hash_and_size_match_what_is_actually_on_disk`
//!     -> `receipt_hash_and_size_match_disk_for_every_builtin_template`
//!
//! Given a file is modified after its `CreationRecord` was issued,
//!     that record's hash and size
//!     must no longer match the file's current on-disk content: a stale
//!     receipt is detectable, never silently treated as still current.
//!     -> `a_file_modified_after_creation_no_longer_matches_its_receipt`
//!
//! Given the same path is instantiated twice (the second time with
//! `overwrite: true` and materially different content),
//!     the first instantiation's `CreationRecord`
//!     must be detectably stale against the second: identical path, but a
//!     different hash — and only the second, later record still matches what
//!     is on disk once both calls have completed.
//!     -> `a_receipt_from_one_instantiation_is_detectably_stale_against_a_later_one`

use flashtex_project_templates::{InstantiateOptions, all_templates, find_template, instantiate};

fn temp_dir(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "flashtex-project-templates-receipt-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn options(name: &str) -> InstantiateOptions {
    InstantiateOptions {
        project_name: name.to_string(),
        author: "Receipt Spec".to_string(),
        overwrite: false,
    }
}

/// Recomputes a file's actual on-disk hash and size independently of
/// anything `instantiate` reported, so a receipt can be checked against
/// ground truth rather than against itself.
fn actual_on_disk(path: &std::path::Path) -> (flashtex_project_files::Digest, u64) {
    let bytes = std::fs::read(path).unwrap();
    (flashtex_project_files::sha256(&bytes), bytes.len() as u64)
}

/// Given a template is instantiated, every returned `CreationRecord` must
/// report the exact hash and size actually on disk — not an approximation,
/// not a value computed before the write, a value taken from verification
/// that happened *after* the bytes landed.
#[test]
fn receipt_hash_and_size_match_what_is_actually_on_disk() {
    let root = temp_dir("matches-disk");
    let template = find_template("course-report").unwrap();
    let report = instantiate(&template, &root, &options("Spec Project")).unwrap();

    assert_eq!(report.created.len(), report.written_files.len());
    assert!(!report.created.is_empty());
    for record in &report.created {
        let (actual_hash, actual_size) = actual_on_disk(&record.path);
        assert_eq!(
            record.sha256, actual_hash,
            "recorded hash must match disk content exactly for {:?}",
            record.path
        );
        assert_eq!(
            record.bytes, actual_size,
            "recorded size must match disk content exactly for {:?}",
            record.path
        );
        assert_eq!(record.sha256_hex().len(), 64);
    }
    std::fs::remove_dir_all(&root).ok();
}

/// The same guarantee, generalized across every built-in template (including
/// the multi-file senior-thesis skeleton) rather than spot-checked on one.
#[test]
fn receipt_hash_and_size_match_disk_for_every_builtin_template() {
    for template in all_templates() {
        let root = temp_dir(&format!("matches-disk-{}", template.id));
        let report = instantiate(&template, &root, &options("Spec Project")).unwrap();
        assert_eq!(report.created.len(), template.files.len());
        for record in &report.created {
            let (actual_hash, actual_size) = actual_on_disk(&record.path);
            assert_eq!(
                record.sha256, actual_hash,
                "template {:?}: hash mismatch for {:?}",
                template.id, record.path
            );
            assert_eq!(
                record.bytes, actual_size,
                "template {:?}: size mismatch for {:?}",
                template.id, record.path
            );
        }
        std::fs::remove_dir_all(&root).ok();
    }
}

/// Given a file is modified after its receipt was issued, the receipt must
/// stop matching. A `CreationRecord` is proof of a past state, and nothing
/// in this crate re-verifies it lazily, so a caller who trusts it as a live
/// view must be able to tell the moment it stopped being true.
#[test]
fn a_file_modified_after_creation_no_longer_matches_its_receipt() {
    let root = temp_dir("modified-after");
    let template = find_template("course-report").unwrap();
    let report = instantiate(&template, &root, &options("Original")).unwrap();
    let record = report
        .created
        .iter()
        .find(|r| r.path.ends_with("main.tex"))
        .unwrap()
        .clone();
    let (hash_at_creation, size_at_creation) = (record.sha256, record.bytes);

    // Confirm the receipt was accurate at the moment of creation, so the
    // mismatch asserted below is caused by the tampering, not a pre-existing
    // bug in the receipt itself.
    let (hash_before_tamper, size_before_tamper) = actual_on_disk(&record.path);
    assert_eq!(hash_at_creation, hash_before_tamper);
    assert_eq!(size_at_creation, size_before_tamper);

    // Something else touches the file after `instantiate` returned.
    let mut contents = std::fs::read(&record.path).unwrap();
    contents.extend_from_slice(b"\n% tampered after creation\n");
    std::fs::write(&record.path, &contents).unwrap();

    let (actual_hash, actual_size) = actual_on_disk(&record.path);
    assert_ne!(
        hash_at_creation, actual_hash,
        "a stale receipt's hash must not match content modified after creation"
    );
    assert_ne!(
        size_at_creation, actual_size,
        "a stale receipt's size must not match content modified after creation"
    );
    std::fs::remove_dir_all(&root).ok();
}

/// Given the same path is instantiated twice with different content
/// (`overwrite: true` the second time), the earlier receipt must be
/// detectably stale against the later one: same path, but the hash the first
/// call recorded must not describe what the second call put on disk.
#[test]
fn a_receipt_from_one_instantiation_is_detectably_stale_against_a_later_one() {
    let root = temp_dir("stale-across-instantiations");
    let template = find_template("course-report").unwrap();

    let first = instantiate(&template, &root, &options("First Version")).unwrap();
    let first_record = first
        .created
        .iter()
        .find(|r| r.path.ends_with("main.tex"))
        .unwrap()
        .clone();

    let mut overwrite_opts = options("Second Version, Materially Different Content");
    overwrite_opts.overwrite = true;
    let second = instantiate(&template, &root, &overwrite_opts).unwrap();
    let second_record = second
        .created
        .iter()
        .find(|r| r.path.ends_with("main.tex"))
        .unwrap()
        .clone();

    assert_eq!(
        first_record.path, second_record.path,
        "both instantiations wrote the same declared path"
    );
    assert_ne!(
        first_record.sha256, second_record.sha256,
        "distinct instantiated content must yield distinct receipts"
    );

    // Only the later receipt still matches what is on disk now; the earlier
    // one is stale evidence of a state the file no longer holds.
    let (actual_hash, actual_size) = actual_on_disk(&second_record.path);
    assert_eq!(second_record.sha256, actual_hash, "the later receipt must match current disk state");
    assert_eq!(second_record.bytes, actual_size, "the later receipt must match current disk state");
    assert_ne!(
        first_record.sha256, actual_hash,
        "the earlier receipt must be detectably stale against current disk state"
    );
    std::fs::remove_dir_all(&root).ok();
}
