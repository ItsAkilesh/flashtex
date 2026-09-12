//! Bounded adversarial acceptance suite for [`instantiate`].
//!
//! Every case here is a hostile or edge-of-bounds input aimed at
//! instantiation itself: a manifest shaped to be expensive, a path shaped to
//! break assumptions about length or structure, or a target root that is not
//! a cooperative empty directory. The contract under test is narrow and
//! absolute: **every case must end as a typed error with the target
//! byte-identical to its pre-call state, or as a clean success — never a
//! panic, a hang, or a half-written tree.**
//!
//! Fault injection at a specific batch index (first/middle/last write of a
//! multi-file template) needs the crate-internal `Hooks` type and so lives in
//! `src/instantiate.rs`'s own `#[cfg(test)]` module instead of here; this
//! file is black-box, public-API only.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use flashtex_project_templates::manifest::ManifestError;
use flashtex_project_templates::path::PathError;
use flashtex_project_templates::{InstantiateError, InstantiateOptions, Template, TemplateFile};

fn temp_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "flashtex-project-templates-adversarial-{label}-{}-{}",
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
        author: "Adversarial Suite".to_string(),
        overwrite: false,
    }
}

fn template(files: Vec<TemplateFile>) -> Template {
    Template {
        id: "hostile".into(),
        title: "Hostile".into(),
        description: "d".into(),
        packages: vec![],
        files,
    }
}

/// Recursively snapshots every regular file under `dir` (path relative to
/// `dir`, plus its exact bytes), skipping FlashTeX's own `.flashtex`
/// bookkeeping directory (the per-project lock file), which is internal and
/// not part of the caller-visible tree. `None` if `dir` does not exist.
fn snapshot(dir: &Path) -> Option<BTreeMap<PathBuf, Vec<u8>>> {
    if !dir.exists() {
        return None;
    }
    let mut out = BTreeMap::new();
    collect(dir, dir, &mut out);
    Some(out)
}

fn collect(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name().and_then(|n| n.to_str()) == Some(".flashtex") {
            continue;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect(root, &path, out);
        } else if file_type.is_file()
            && let Ok(bytes) = std::fs::read(&path)
        {
            out.insert(path.strip_prefix(root).unwrap().to_path_buf(), bytes);
        }
    }
}

#[cfg(unix)]
unsafe extern "C" {
    fn geteuid() -> u32;
}

#[cfg(unix)]
fn running_as_root() -> bool {
    unsafe { geteuid() == 0 }
}

// ---------------------------------------------------------------------
// A manifest with thousands of files.
// ---------------------------------------------------------------------

#[test]
fn a_manifest_with_thousands_of_files_instantiates_completely_without_hanging() {
    let root = temp_dir("thousands-of-files");
    const N: usize = 2000;
    let files = (0..N)
        .map(|i| TemplateFile::new(format!("dir{}/file{i}.tex", i % 25), format!("content-{i}")))
        .collect();
    let report =
        flashtex_project_templates::instantiate(&template(files), &root, &options("Bulk")).unwrap();
    assert_eq!(report.written_files.len(), N);
    assert_eq!(report.created.len(), N);
    for i in 0..N {
        let path = root.join(format!("dir{}/file{i}.tex", i % 25));
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            format!("content-{i}")
        );
    }
    std::fs::remove_dir_all(&root).ok();
}

// ---------------------------------------------------------------------
// A single file of enormous size.
// ---------------------------------------------------------------------

#[test]
fn a_single_enormous_file_is_written_completely_and_correctly() {
    let root = temp_dir("enormous-file");
    let big = "A".repeat(20 * 1024 * 1024); // 20 MiB
    let t = template(vec![TemplateFile::new("big.tex", big.clone())]);
    let report = flashtex_project_templates::instantiate(&t, &root, &options("Big")).unwrap();
    assert_eq!(report.created.len(), 1);
    let on_disk = std::fs::read(root.join("big.tex")).unwrap();
    assert_eq!(on_disk.len(), big.len());
    assert_eq!(report.created[0].bytes, big.len() as u64);
    assert_eq!(
        report.created[0].sha256,
        flashtex_project_files::sha256(&on_disk)
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn overwriting_a_pre_existing_file_larger_than_the_read_limit_is_a_typed_error_and_leaves_it_untouched()
 {
    let root = temp_dir("too-large-existing");
    std::fs::create_dir_all(&root).unwrap();
    let huge = vec![b'x'; (flashtex_project_files::DEFAULT_READ_LIMIT + 1) as usize];
    std::fs::write(root.join("main.tex"), &huge).unwrap();

    let t = template(vec![TemplateFile::new("main.tex", "new content")]);
    let mut opts = options("X");
    opts.overwrite = true;
    let err = flashtex_project_templates::instantiate(&t, &root, &opts).unwrap_err();
    assert!(
        matches!(err, InstantiateError::Rooted { .. }),
        "expected Rooted, got {err:?}"
    );
    let after = std::fs::read(root.join("main.tex")).unwrap();
    assert_eq!(
        after, huge,
        "an oversized pre-existing file must be left byte-identical"
    );
    std::fs::remove_dir_all(&root).ok();
}

// ---------------------------------------------------------------------
// A path at, and past, a filesystem length limit.
// ---------------------------------------------------------------------

#[test]
fn a_path_component_well_within_the_filesystem_name_limit_is_accepted() {
    let root = temp_dir("name-limit-ok");
    let name = format!("{}.tex", "a".repeat(200));
    let t = template(vec![TemplateFile::new(name.clone(), "ok")]);
    let report = flashtex_project_templates::instantiate(&t, &root, &options("X")).unwrap();
    assert_eq!(report.written_files, vec![root.join(&name)]);
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn a_path_component_past_the_filesystem_name_limit_is_a_typed_error_and_writes_no_file() {
    let root = temp_dir("name-limit-exceeded");
    let name = format!("{}.tex", "a".repeat(5000));
    let t = template(vec![TemplateFile::new(name.clone(), "pwned")]);
    let err = flashtex_project_templates::instantiate(&t, &root, &options("X")).unwrap_err();
    assert!(
        matches!(err, InstantiateError::Rooted { ref rollback_incomplete, .. } if rollback_incomplete.is_empty()),
        "expected a clean Rooted refusal, got {err:?}"
    );
    assert!(
        !root.join(&name).exists(),
        "the oversized-name file must never land on disk"
    );
    std::fs::remove_dir_all(&root).ok();
}

// ---------------------------------------------------------------------
// Deeply nested directories.
// ---------------------------------------------------------------------

#[test]
fn deeply_nested_directories_are_created_without_panicking_or_hanging() {
    let root = temp_dir("deep-nest");
    let depth = 100;
    let mut segments: Vec<String> = (0..depth).map(|i| format!("d{i}")).collect();
    segments.push("leaf.tex".to_string());
    let path = segments.join("/");
    let t = template(vec![TemplateFile::new(path, "leaf content")]);
    let report = flashtex_project_templates::instantiate(&t, &root, &options("X")).unwrap();
    assert_eq!(report.written_files.len(), 1);
    let mut p = root.clone();
    for seg in &segments {
        p.push(seg);
    }
    assert_eq!(std::fs::read_to_string(&p).unwrap(), "leaf content");
    std::fs::remove_dir_all(&root).ok();
}

// ---------------------------------------------------------------------
// A path whose parent is a regular file.
// ---------------------------------------------------------------------

#[test]
fn a_path_whose_parent_is_a_regular_file_is_a_typed_error_and_leaves_everything_untouched() {
    let root = temp_dir("parent-is-file");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("notadir"), "I AM A FILE, NOT A DIRECTORY").unwrap();

    let t = template(vec![
        TemplateFile::new("notadir/inner.tex", "pwned"),
        TemplateFile::new("other.tex", "should never appear"),
    ]);
    let before = snapshot(&root).unwrap();
    let err = flashtex_project_templates::instantiate(&t, &root, &options("X")).unwrap_err();
    assert!(
        matches!(err, InstantiateError::Rooted { .. }),
        "expected Rooted, got {err:?}"
    );
    let after = snapshot(&root).unwrap();
    assert_eq!(
        before, after,
        "target tree must be byte-identical to its pre-call state"
    );
    std::fs::remove_dir_all(&root).ok();
}

// ---------------------------------------------------------------------
// Target root that does not exist, is a file, or is read-only.
// ---------------------------------------------------------------------

#[test]
fn a_target_root_that_does_not_exist_is_created_and_populated() {
    let root = temp_dir("root-missing");
    assert!(!root.exists());
    let t = template(vec![TemplateFile::new("main.tex", "x")]);
    flashtex_project_templates::instantiate(&t, &root, &options("X")).unwrap();
    assert!(root.join("main.tex").is_file());
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn a_target_root_that_is_a_file_is_a_typed_error_and_the_file_is_untouched() {
    let root = temp_dir("root-is-file");
    std::fs::write(&root, "I AM A FILE").unwrap();
    let t = template(vec![TemplateFile::new("main.tex", "x")]);
    let err = flashtex_project_templates::instantiate(&t, &root, &options("X")).unwrap_err();
    assert!(
        matches!(
            err,
            InstantiateError::Io { .. } | InstantiateError::RootNotADirectory(_)
        ),
        "expected a typed I/O-shaped error, got {err:?}"
    );
    assert_eq!(std::fs::read_to_string(&root).unwrap(), "I AM A FILE");
    std::fs::remove_file(&root).ok();
}

#[cfg(unix)]
#[test]
fn a_target_root_that_is_read_only_is_a_typed_error_and_nothing_is_written() {
    if running_as_root() {
        eprintln!("skipping: running as root, permission bits are not enforced");
        return;
    }
    use std::os::unix::fs::PermissionsExt;

    let root = temp_dir("root-readonly");
    std::fs::create_dir_all(&root).unwrap();
    let mut perms = std::fs::metadata(&root).unwrap().permissions();
    perms.set_mode(0o555);
    std::fs::set_permissions(&root, perms).unwrap();

    let t = template(vec![TemplateFile::new("main.tex", "x")]);
    let err = flashtex_project_templates::instantiate(&t, &root, &options("X")).unwrap_err();

    // Restore write permission before any cleanup, regardless of assertion
    // outcome below.
    let mut perms = std::fs::metadata(&root).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&root, perms).unwrap();

    assert!(
        matches!(err, InstantiateError::Rooted { .. }),
        "expected Rooted, got {err:?}"
    );
    assert!(
        std::fs::read_dir(&root).unwrap().next().is_none(),
        "a read-only target root must end up with nothing written"
    );
    std::fs::remove_dir_all(&root).ok();
}

// ---------------------------------------------------------------------
// Duplicate declared paths; an empty path.
// ---------------------------------------------------------------------

#[test]
fn duplicate_declared_paths_is_a_typed_error_and_writes_nothing() {
    let root = temp_dir("dup-paths");
    let t = template(vec![
        TemplateFile::new("main.tex", "first"),
        TemplateFile::new("main.tex", "second"),
    ]);
    let err = flashtex_project_templates::instantiate(&t, &root, &options("X")).unwrap_err();
    assert!(
        matches!(
            err,
            InstantiateError::InvalidTemplate(ManifestError::DuplicatePath(_))
        ),
        "expected InvalidTemplate/DuplicatePath, got {err:?}"
    );
    assert!(
        !root.exists(),
        "validation runs before target_root is even created"
    );
}

#[test]
fn an_empty_declared_path_is_a_typed_error_and_writes_nothing() {
    let root = temp_dir("empty-path");
    let t = template(vec![TemplateFile::new("", "x")]);
    let err = flashtex_project_templates::instantiate(&t, &root, &options("X")).unwrap_err();
    assert!(
        matches!(
            err,
            InstantiateError::InvalidTemplate(ManifestError::InvalidPath {
                source: PathError::Empty,
                ..
            })
        ),
        "expected InvalidTemplate/Empty, got {err:?}"
    );
    assert!(!root.exists());
}

// ---------------------------------------------------------------------
// Unicode: right-to-left override, non-NFC, replacement/noncharacters.
// (Byte-level rejection is proven directly against path::validate in
// src/path.rs's own unit tests; these prove the same guarantee holds
// end-to-end through the public instantiate() entry point.)
// ---------------------------------------------------------------------

#[test]
fn a_declared_path_containing_a_right_to_left_override_is_a_typed_error_and_writes_nothing() {
    let root = temp_dir("rtl-override");
    // Renders (in a bidi-aware viewer) as if the extension were reversed --
    // the classic filename-spoofing trick.
    let t = template(vec![TemplateFile::new("invoice\u{202E}fdp.exe", "pwned")]);
    let err = flashtex_project_templates::instantiate(&t, &root, &options("X")).unwrap_err();
    assert!(
        matches!(
            err,
            InstantiateError::InvalidTemplate(ManifestError::InvalidPath {
                source: PathError::ForbiddenCharacter('\u{202E}'),
                ..
            })
        ),
        "expected InvalidTemplate/ForbiddenCharacter('\\u202E'), got {err:?}"
    );
    assert!(
        !root.exists(),
        "validation runs before target_root is even created"
    );
}

#[test]
fn a_declared_path_with_non_nfc_unicode_instantiates_with_the_exact_decomposed_bytes_preserved() {
    let root = temp_dir("non-nfc-path");
    // "é" spelled as `e` + COMBINING ACUTE ACCENT (NFD), not the precomposed
    // U+00E9 (NFC). If anything silently normalized this, the resulting path
    // would differ from what the template declared -- an implicit
    // approximation this crate must never perform.
    let name = "e\u{0301}tude.tex";
    let t = template(vec![TemplateFile::new(name, "contenu")]);
    let report = flashtex_project_templates::instantiate(&t, &root, &options("X")).unwrap();
    assert_eq!(report.written_files, vec![root.join(name)]);
    let entries: Vec<_> = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name != ".flashtex")
        .collect();
    assert_eq!(
        entries,
        vec![name.to_string()],
        "on-disk file name must be byte-identical to the declared (decomposed) form"
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn a_declared_path_with_a_replacement_character_instantiates_without_panicking() {
    let root = temp_dir("replacement-char-path");
    // Nearest real proxy for "a surrogate-derived sequence": Rust's `&str`
    // cannot hold a lone UTF-16 surrogate half (no `char` value exists for
    // one), but U+FFFD is what a lossy conversion of ill-formed UTF-16
    // containing one turns it into.
    let name = "bad\u{FFFD}encoding.tex";
    let t = template(vec![TemplateFile::new(name, "x")]);
    let report = flashtex_project_templates::instantiate(&t, &root, &options("X")).unwrap();
    assert_eq!(report.written_files, vec![root.join(name)]);
    std::fs::remove_dir_all(&root).ok();
}

// ---------------------------------------------------------------------
// project_name / author: free text, not a path -- separators and ".."
// have no filesystem meaning here and must never be treated as one.
// ---------------------------------------------------------------------

#[test]
fn a_project_name_containing_path_separators_and_dotdot_is_ordinary_text_not_a_path() {
    let root = temp_dir("name-looks-like-path");
    let t = template(vec![TemplateFile::new(
        "main.tex",
        "\\title{{{{project_name}}}}",
    )]);
    let hostile_name = "../../etc/passwd";
    let report =
        flashtex_project_templates::instantiate(&t, &root, &options(hostile_name)).unwrap();
    assert_eq!(report.written_files, vec![root.join("main.tex")]);
    let contents = std::fs::read_to_string(root.join("main.tex")).unwrap();
    assert!(
        contents.contains(hostile_name),
        "the text must land verbatim in the title, not be interpreted as a path"
    );
    // Nothing was created anywhere except the one declared file (plus the
    // crate's own `.flashtex` lock bookkeeping).
    let entries: Vec<_> = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name != ".flashtex")
        .collect();
    assert_eq!(entries, vec!["main.tex".to_string()]);
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn an_absurdly_long_project_name_instantiates_completely_without_hanging() {
    let root = temp_dir("absurd-name-length");
    let t = template(vec![TemplateFile::new(
        "main.tex",
        "\\title{{{{project_name}}}}",
    )]);
    let long_name = "A".repeat(1_000_000);
    let report = flashtex_project_templates::instantiate(&t, &root, &options(&long_name)).unwrap();
    assert_eq!(report.written_files, vec![root.join("main.tex")]);
    let contents = std::fs::read_to_string(root.join("main.tex")).unwrap();
    assert!(contents.contains(&long_name));
    std::fs::remove_dir_all(&root).ok();
}

// ---------------------------------------------------------------------
// DEFAULT_READ_LIMIT: exactly at the bound (succeeds), one byte past it
// (a typed error, tested above in
// `overwriting_a_pre_existing_file_larger_than_the_read_limit_is_a_typed_error_and_leaves_it_untouched`).
// ---------------------------------------------------------------------

#[test]
fn overwriting_a_pre_existing_file_exactly_at_the_read_limit_succeeds() {
    let root = temp_dir("at-read-limit");
    std::fs::create_dir_all(&root).unwrap();
    let at_limit = vec![b'x'; flashtex_project_files::DEFAULT_READ_LIMIT as usize];
    std::fs::write(root.join("main.tex"), &at_limit).unwrap();

    let t = template(vec![TemplateFile::new("main.tex", "new content")]);
    let mut opts = options("X");
    opts.overwrite = true;
    let report = flashtex_project_templates::instantiate(&t, &root, &opts).unwrap();
    assert_eq!(report.created.len(), 1);
    assert_eq!(
        std::fs::read_to_string(root.join("main.tex")).unwrap(),
        "new content"
    );
    std::fs::remove_dir_all(&root).ok();
}
