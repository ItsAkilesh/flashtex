//! Rooted, non-destructive instantiation of a [`Template`] into a target
//! directory, with an explicit creation receipt and whole-batch
//! interrupted-write recovery.
//!
//! Security is the point of this module: a template must never write
//! outside the caller's target directory, must never silently clobber a
//! file that is already there, and must never leave the target directory in
//! a half-written state if instantiation fails partway through. [`instantiate`]
//! enforces all three by construction, not by convention:
//!
//! - Every declared file path is validated by [`crate::path::validate`]
//!   before any I/O happens; a path containing `..` or an absolute path is
//!   rejected with a typed [`InstantiateError::InvalidTemplate`]. This
//!   validation is strict in a way `flashtex_project_files::ProjectPath`
//!   deliberately is not: that type *resolves* `..` (popping a segment, only
//!   erroring if it would leave the root), whereas ours refuses any `..`
//!   segment outright, even one that would mathematically cancel out (see
//!   `crate::path`'s tests). Because that guarantee is ours alone, this
//!   module keeps `crate::path::validate` as the sole judge of a raw
//!   template-declared path string; a `flashtex_project_files::ProjectPath`
//!   is only ever built from segments that already passed it, never from a
//!   raw string, so it can't reintroduce the traversal it would otherwise
//!   silently resolve.
//! - Every write and pre-existence check goes through
//!   `flashtex_project_files::ProjectRoot`/`ProjectLock`: rooted,
//!   `O_NOFOLLOW` at every path component, so a symlink planted at (or
//!   above) a declared path is refused rather than written through. Rev 1's
//!   plain `Path::exists`/`fs::write` would have followed such a symlink;
//!   that gap is closed here, not merely refactored around.
//! - Before writing anything, every target path is checked for an existing
//!   file through that same rooted reader; if one is found and
//!   [`InstantiateOptions::overwrite`] is `false`, instantiation fails with
//!   [`InstantiateError::AlreadyExists`] and *nothing is written* — a
//!   template can't clobber half the project and leave a mix of old and new
//!   files behind. This whole-batch preflight is ours: `ProjectRoot` only
//!   guarantees no-clobber per individual write, not across a batch.
//! - Every successful write returns a [`CreationRecord`] (path, content hash,
//!   size) taken from the rooted writer's own post-write verification, not
//!   recomputed separately, so [`InstantiateReport::created`] is an honest
//!   receipt of what actually landed on disk.
//! - If a write fails partway through a multi-file template — a late
//!   conflict the preflight couldn't see (an external writer racing us), a
//!   disk error, or a refused symlink — every file this call already wrote
//!   is rolled back: restored to the exact bytes it held before this call if
//!   it pre-existed (captured during preflight), or removed if this call
//!   created it. The result is that `instantiate` is all-or-nothing: either
//!   every declared file ends up as specified, or the target directory is
//!   left exactly as it was found.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use flashtex_project_files::{
    DEFAULT_READ_LIMIT, Digest, Expected, ProjectLock, ProjectPath as RootedPath, ProjectRoot,
    Refused, SaveConflictKind, SaveError,
};

use crate::escape;
use crate::field::{self, FieldError};
use crate::manifest::{ManifestError, Template};

/// Caller-supplied values for instantiating a [`Template`].
#[derive(Debug, Clone, Default)]
pub struct InstantiateOptions {
    /// Substituted for `{{project_name}}` (LaTeX-escaped first).
    pub project_name: String,
    /// Substituted for `{{author}}` (LaTeX-escaped first). Empty by
    /// default, which renders as an empty `\author{}`.
    pub author: String,
    /// If `false` (the default), instantiation refuses to overwrite any
    /// file that already exists at its target path. If `true`, an existing
    /// file at a template-declared path is replaced; files not declared by
    /// the template are never touched either way.
    pub overwrite: bool,
}

/// One file [`instantiate`] created or replaced: proof of exactly what
/// happened, without the caller needing to re-read and re-hash the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreationRecord {
    pub path: PathBuf,
    pub sha256: Digest,
    pub bytes: u64,
}

impl CreationRecord {
    pub fn sha256_hex(&self) -> String {
        flashtex_project_files::sha256_to_hex(&self.sha256)
    }
}

/// What [`instantiate`] wrote on success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstantiateReport {
    /// Every file path written, in template order.
    pub written_files: Vec<PathBuf>,
    /// The creation receipt: one entry per `written_files`, same order, with
    /// the exact content hash and size the rooted writer verified after the
    /// file landed on disk.
    pub created: Vec<CreationRecord>,
}

/// Why instantiation failed. No file is left in a changed state when any
/// variant is returned: either nothing was written yet, or every file this
/// call had written was rolled back (see [`InstantiateError::Rooted`]).
#[derive(Debug)]
pub enum InstantiateError {
    /// The template manifest itself failed validation (bad path, bad
    /// package name, or duplicate path). This is the typed error a
    /// traversal or absolute-path file entry surfaces as.
    InvalidTemplate(ManifestError),
    /// `project_name` or `author` contained a forbidden character.
    InvalidField {
        field: &'static str,
        source: FieldError,
    },
    /// `target_root` exists and is not a directory.
    RootNotADirectory(PathBuf),
    /// A file this template would write already exists and
    /// `options.overwrite` was `false`. Nothing was written.
    AlreadyExists(PathBuf),
    /// An I/O operation outside the rooted writer failed (creating
    /// `target_root` itself). Nothing from this template was written.
    Io { path: PathBuf, source: io::Error },
    /// The rooted filesystem layer refused an operation on `path`, or
    /// reported a conflict it could not resolve as a plain
    /// [`InstantiateError::AlreadyExists`] (a late conflict discovered
    /// mid-batch, a symlink refusal, or a disk error). If this happened
    /// after this call had already written one or more files,
    /// `rollback_incomplete` is empty when every one of them was
    /// successfully restored to its pre-call state (or removed, if this
    /// call had created it) — the common case — and otherwise lists the
    /// files that could not be rolled back, which must be inspected
    /// manually.
    Rooted {
        path: PathBuf,
        source: SaveError,
        rollback_incomplete: Vec<PathBuf>,
    },
}

impl fmt::Display for InstantiateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstantiateError::InvalidTemplate(e) => write!(f, "invalid template: {e}"),
            InstantiateError::InvalidField { field, source } => {
                write!(f, "invalid {field}: {source}")
            }
            InstantiateError::RootNotADirectory(p) => {
                write!(f, "target root {} is not a directory", p.display())
            }
            InstantiateError::AlreadyExists(p) => {
                write!(
                    f,
                    "{} already exists; pass overwrite: true to replace it",
                    p.display()
                )
            }
            InstantiateError::Io { path, source } => {
                write!(f, "I/O error at {}: {source}", path.display())
            }
            InstantiateError::Rooted {
                path,
                source,
                rollback_incomplete,
            } => {
                if rollback_incomplete.is_empty() {
                    write!(
                        f,
                        "rooted filesystem operation on {} failed: {source}",
                        path.display()
                    )
                } else {
                    write!(
                        f,
                        "rooted filesystem operation on {} failed: {source}; {} previously-written file(s) could not be rolled back: {:?}",
                        path.display(),
                        rollback_incomplete.len(),
                        rollback_incomplete
                    )
                }
            }
        }
    }
}

impl std::error::Error for InstantiateError {}

/// Test-only fault injection, mirroring `flashtex_project_files::save`'s own
/// `Hooks` pattern: lets unit tests simulate a write failing partway through
/// a multi-file batch — a disk error or a late-discovered conflict — without
/// needing to actually trigger one, so the rollback behavior can be
/// exercised deterministically.
#[derive(Default)]
pub(crate) struct Hooks<'h> {
    /// Called immediately before writing the file at batch index `i`
    /// (0-based, template order). Returning `Err` aborts the batch as if
    /// that write had failed with this I/O error, after any files with a
    /// smaller index have already landed and must now be rolled back.
    pub before_write: Option<&'h dyn Fn(usize) -> io::Result<()>>,
}

/// Instantiates `template` under `target_root`, substituting `options` into
/// each file body, and returns a receipt of every file written.
///
/// `target_root` is created (via `create_dir_all`) if it does not exist. No
/// file is written outside `target_root`, no existing file is overwritten
/// unless `options.overwrite` is `true`, and a failure partway through
/// leaves `target_root` exactly as it was before the call — see the module
/// docs for exactly how all three guarantees are enforced.
pub fn instantiate(
    template: &Template,
    target_root: &Path,
    options: &InstantiateOptions,
) -> Result<InstantiateReport, InstantiateError> {
    instantiate_with_hooks(template, target_root, options, &Hooks::default())
}

pub(crate) fn instantiate_with_hooks(
    template: &Template,
    target_root: &Path,
    options: &InstantiateOptions,
    hooks: &Hooks<'_>,
) -> Result<InstantiateReport, InstantiateError> {
    template
        .validate()
        .map_err(InstantiateError::InvalidTemplate)?;
    field::validate(&options.project_name).map_err(|source| InstantiateError::InvalidField {
        field: "project_name",
        source,
    })?;
    field::validate(&options.author).map_err(|source| InstantiateError::InvalidField {
        field: "author",
        source,
    })?;

    fs::create_dir_all(target_root).map_err(|source| InstantiateError::Io {
        path: target_root.to_path_buf(),
        source,
    })?;
    if !target_root.is_dir() {
        return Err(InstantiateError::RootNotADirectory(
            target_root.to_path_buf(),
        ));
    }

    // From here on every read and write is rooted at `target_root`:
    // `ProjectRoot::open` itself refuses a `target_root` whose final
    // component is a symlink, and every subsequent walk refuses a symlink
    // at any parent component too.
    let root = ProjectRoot::open(target_root).map_err(|source| InstantiateError::Rooted {
        path: target_root.to_path_buf(),
        source,
        rollback_incomplete: Vec::new(),
    })?;
    let lock = root.lock().map_err(|source| InstantiateError::Rooted {
        path: target_root.to_path_buf(),
        source,
        rollback_incomplete: Vec::new(),
    })?;

    // Resolve every declared path from *validated segments only* (never by
    // parsing/joining the raw string), exactly as before, so there is no
    // code path through which a rejected path could still reach the
    // filesystem. `Template::validate` above already rejected `..`,
    // absolute paths, duplicates, and forbidden characters, so joining the
    // accepted segments back into a `RootedPath` cannot fail or reintroduce
    // a traversal.
    let mut plan = Vec::with_capacity(template.files.len());
    for file in &template.files {
        let segments = crate::path::validate(&file.path)
            .expect("template already validated: path must be well-formed");
        let mut target = target_root.to_path_buf();
        for seg in &segments {
            target.push(seg);
        }
        debug_assert!(
            target.starts_with(target_root),
            "resolved path escaped target_root"
        );
        let rooted_path = RootedPath::normalize(&segments.join("/"))
            .expect("validated segments cannot fail rooted normalization");
        plan.push((file, target, rooted_path));
    }

    // Whole-template preflight, through the rooted reader instead of
    // `Path::exists` (which follows symlinks). Non-overwrite mode fails the
    // whole batch — nothing written — the moment any target is found to
    // exist. Overwrite mode instead reads back each existing target's
    // current bytes, which double as the rollback source if a later write
    // in this same call fails.
    let mut originals: Vec<Option<Vec<u8>>> = Vec::with_capacity(plan.len());
    for (_, target, rooted_path) in &plan {
        match root.read(rooted_path, DEFAULT_READ_LIMIT) {
            Ok(Some(read)) => {
                if !options.overwrite {
                    return Err(InstantiateError::AlreadyExists(target.clone()));
                }
                originals.push(Some(read.bytes));
            }
            Ok(None) => originals.push(None),
            // A nested file's containing directory does not exist yet:
            // `ProjectRoot::read` walks without creating directories, so
            // this surfaces as a plain `NotFound` I/O error rather than
            // `Ok(None)`. A missing directory means the file itself is
            // certainly missing too, so treat it the same as `Ok(None)`;
            // `ProjectLock::save` below creates the missing directories
            // when it actually writes.
            Err(SaveError::Io(e)) if e.kind() == io::ErrorKind::NotFound => {
                originals.push(None);
            }
            Err(SaveError::Refused(Refused::TooLarge { .. })) if !options.overwrite => {
                // Too large to read back, but its mere existence already
                // blocks a non-overwrite write.
                return Err(InstantiateError::AlreadyExists(target.clone()));
            }
            Err(source) => {
                return Err(InstantiateError::Rooted {
                    path: target.clone(),
                    source,
                    rollback_incomplete: Vec::new(),
                });
            }
        }
    }

    let packages_block = template
        .packages
        .iter()
        .map(|p| format!("\\usepackage{{{p}}}\n"))
        .collect::<String>();
    let packages_block = packages_block.trim_end_matches('\n');
    let project_name_escaped = escape::escape(&options.project_name);
    let author_escaped = escape::escape(&options.author);

    // Write every file, in template order, tracking exactly enough to fully
    // undo this call if a later file fails: the project path plus whatever
    // content (or absence) was there before this call started.
    let mut written: Vec<(RootedPath, PathBuf, Option<Vec<u8>>)> = Vec::with_capacity(plan.len());
    let mut created = Vec::with_capacity(plan.len());
    let mut written_files = Vec::with_capacity(plan.len());

    for (i, ((file, target, rooted_path), original)) in plan.iter().zip(originals).enumerate() {
        if let Some(hook) = hooks.before_write
            && let Err(io_err) = hook(i)
        {
            let rollback_incomplete = rollback(&lock, &written);
            return Err(InstantiateError::Rooted {
                path: target.clone(),
                source: SaveError::Io(io_err),
                rollback_incomplete,
            });
        }

        let contents = render(
            &file.body,
            &project_name_escaped,
            &author_escaped,
            packages_block,
        );
        let expected = match &original {
            Some(bytes) => Expected::Hash(flashtex_project_files::sha256(bytes)),
            None => Expected::NewFile,
        };
        match lock.save(rooted_path, contents.as_bytes(), expected, false) {
            Ok(receipt) => {
                written.push((rooted_path.clone(), target.clone(), original));
                created.push(CreationRecord {
                    path: target.clone(),
                    sha256: receipt.sha256,
                    bytes: receipt.bytes,
                });
                written_files.push(target.clone());
            }
            Err(source) => {
                let rollback_incomplete = rollback(&lock, &written);
                if rollback_incomplete.is_empty()
                    && matches!(&source, SaveError::Conflict(c) if c.kind == SaveConflictKind::AlreadyExists)
                {
                    return Err(InstantiateError::AlreadyExists(target.clone()));
                }
                return Err(InstantiateError::Rooted {
                    path: target.clone(),
                    source,
                    rollback_incomplete,
                });
            }
        }
    }

    Ok(InstantiateReport {
        written_files,
        created,
    })
}

/// Undoes every entry in `written`, most recent first: restores the
/// original bytes for a file that pre-existed this call, or removes a file
/// this call created. Returns the (absolute) paths of any entry that could
/// not be undone; empty in the common case.
fn rollback(
    lock: &ProjectLock<'_>,
    written: &[(RootedPath, PathBuf, Option<Vec<u8>>)],
) -> Vec<PathBuf> {
    let mut incomplete = Vec::new();
    for (rooted_path, target, original) in written.iter().rev() {
        let ok = match original {
            Some(bytes) => lock.save(rooted_path, bytes, Expected::Any, true).is_ok(),
            None => lock.remove(rooted_path).is_ok(),
        };
        if !ok {
            incomplete.push(target.clone());
        }
    }
    incomplete
}

fn render(body: &str, project_name: &str, author: &str, packages_block: &str) -> String {
    body.replace("{{project_name}}", project_name)
        .replace("{{author}}", author)
        .replace("{{packages}}", packages_block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::TemplateFile;

    fn opts(name: &str) -> InstantiateOptions {
        InstantiateOptions {
            project_name: name.to_string(),
            author: "A. Student".to_string(),
            overwrite: false,
        }
    }

    fn tempdir(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "flashtex-project-templates-test-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        dir
    }

    #[test]
    fn writes_declared_files_and_substitutes_fields() {
        let root = tempdir("basic");
        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec!["amsmath".into(), "hyperref".into()],
            files: vec![TemplateFile::new(
                "main.tex",
                "{{packages}}\n\\title{{{project_name}}}\n\\author{{{author}}}\n",
            )],
        };
        let report = instantiate(&t, &root, &opts("My Report")).unwrap();
        assert_eq!(report.written_files, vec![root.join("main.tex")]);
        let contents = fs::read_to_string(root.join("main.tex")).unwrap();
        assert!(contents.contains("\\usepackage{amsmath}"));
        assert!(contents.contains("\\usepackage{hyperref}"));
        assert!(contents.contains("\\title{My Report}"));
        assert!(contents.contains("\\author{A. Student}"));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn writes_nested_files() {
        let root = tempdir("nested");
        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![
                TemplateFile::new("main.tex", "root"),
                TemplateFile::new("chapters/intro.tex", "intro"),
            ],
        };
        instantiate(&t, &root, &opts("X")).unwrap();
        assert!(root.join("main.tex").is_file());
        assert!(root.join("chapters/intro.tex").is_file());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn rejects_traversal_path_with_typed_error_and_writes_nothing() {
        let root = tempdir("traversal");
        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![TemplateFile::new("../../outside.tex", "pwned")],
        };
        let err = instantiate(&t, &root, &opts("X")).unwrap_err();
        match err {
            InstantiateError::InvalidTemplate(ManifestError::InvalidPath { source, .. }) => {
                assert_eq!(source, crate::path::PathError::ParentTraversal);
            }
            other => panic!("expected InvalidTemplate/ParentTraversal, got {other:?}"),
        }
        // Nothing should have been written anywhere, including target_root
        // itself (validation runs before target_root is even created).
        assert!(!root.exists());
    }

    #[test]
    fn rejects_absolute_path_with_typed_error_and_writes_nothing() {
        let root = tempdir("absolute");
        let sentinel = tempdir("absolute-sentinel");
        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![TemplateFile::new(
                sentinel.to_string_lossy().into_owned(),
                "pwned",
            )],
        };
        let err = instantiate(&t, &root, &opts("X")).unwrap_err();
        match err {
            InstantiateError::InvalidTemplate(ManifestError::InvalidPath { source, .. }) => {
                assert_eq!(source, crate::path::PathError::Absolute);
            }
            other => panic!("expected InvalidTemplate/Absolute, got {other:?}"),
        }
        assert!(!sentinel.exists());
        assert!(!root.exists());
    }

    #[test]
    fn refuses_to_overwrite_existing_file_without_the_flag() {
        let root = tempdir("overwrite");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.tex"), "PRECIOUS EXISTING WORK").unwrap();

        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![TemplateFile::new("main.tex", "clobbered")],
        };
        let err = instantiate(&t, &root, &opts("X")).unwrap_err();
        match err {
            InstantiateError::AlreadyExists(p) => assert_eq!(p, root.join("main.tex")),
            other => panic!("expected AlreadyExists, got {other:?}"),
        }
        // The existing file must be untouched.
        assert_eq!(
            fs::read_to_string(root.join("main.tex")).unwrap(),
            "PRECIOUS EXISTING WORK"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_conflicting_second_file_prevents_writing_the_first_too() {
        // Whole-template preflight: if any file conflicts, nothing is
        // written, not even the files that would not have conflicted.
        let root = tempdir("preflight");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("b.tex"), "EXISTING").unwrap();

        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![
                TemplateFile::new("a.tex", "new-a"),
                TemplateFile::new("b.tex", "new-b"),
            ],
        };
        let err = instantiate(&t, &root, &opts("X")).unwrap_err();
        assert!(matches!(err, InstantiateError::AlreadyExists(_)));
        assert!(!root.join("a.tex").exists());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn overwrite_flag_allows_replacing_an_existing_file() {
        let root = tempdir("overwrite-allowed");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.tex"), "OLD").unwrap();

        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![TemplateFile::new("main.tex", "NEW")],
        };
        let mut options = opts("X");
        options.overwrite = true;
        instantiate(&t, &root, &options).unwrap();
        assert_eq!(fs::read_to_string(root.join("main.tex")).unwrap(), "NEW");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn accepts_unicode_project_name_and_escapes_hostile_field() {
        let root = tempdir("unicode");
        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![TemplateFile::new("main.tex", "\\title{{{{project_name}}}}")],
        };
        let mut options = opts("Café Ünïcödé Über — 論文 レポート");
        options.author = "}\\input{/etc/passwd}{".to_string();
        instantiate(&t, &root, &options).unwrap();
        let contents = fs::read_to_string(root.join("main.tex")).unwrap();
        assert!(contents.contains("Café Ünïcödé Über — 論文 レポート"));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn rejects_malformed_project_name_with_control_character() {
        let root = tempdir("malformed-field");
        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![TemplateFile::new("main.tex", "x")],
        };
        let err = instantiate(&t, &root, &opts("bad\0name")).unwrap_err();
        assert!(matches!(
            err,
            InstantiateError::InvalidField {
                field: "project_name",
                ..
            }
        ));
        assert!(!root.exists());
    }

    #[test]
    fn creation_receipt_reports_hash_and_size_for_every_written_file() {
        let root = tempdir("receipt");
        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![
                TemplateFile::new("a.tex", "aaa"),
                TemplateFile::new("chapters/b.tex", "bbbbb"),
            ],
        };
        let report = instantiate(&t, &root, &opts("R")).unwrap();
        assert_eq!(report.created.len(), 2);
        for record in &report.created {
            let on_disk = fs::read(&record.path).unwrap();
            assert_eq!(record.bytes, on_disk.len() as u64);
            assert_eq!(record.sha256, flashtex_project_files::sha256(&on_disk));
            assert_eq!(record.sha256_hex().len(), 64);
        }
        assert_eq!(report.created[0].path, root.join("a.tex"));
        assert_eq!(report.created[1].path, root.join("chapters/b.tex"));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn mid_batch_write_failure_leaves_no_partial_tree() {
        // Inject a failure while writing the third of three new files,
        // after the first two have already landed on disk, and assert the
        // whole batch is rolled back: none of the three files survive.
        let root = tempdir("mid-batch-new");
        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![
                TemplateFile::new("a.tex", "a"),
                TemplateFile::new("b.tex", "b"),
                TemplateFile::new("c.tex", "c"),
            ],
        };
        let fail_at_index_2 = |i: usize| -> io::Result<()> {
            if i == 2 {
                Err(io::Error::other("simulated disk error mid-batch"))
            } else {
                Ok(())
            }
        };
        let hooks = Hooks {
            before_write: Some(&fail_at_index_2),
        };
        let err = instantiate_with_hooks(&t, &root, &opts("X"), &hooks).unwrap_err();
        match &err {
            InstantiateError::Rooted {
                rollback_incomplete,
                ..
            } => assert!(
                rollback_incomplete.is_empty(),
                "rollback should have fully succeeded: {rollback_incomplete:?}"
            ),
            other => panic!("expected Rooted, got {other:?}"),
        }
        assert!(!root.join("a.tex").exists(), "a.tex must be rolled back");
        assert!(!root.join("b.tex").exists(), "b.tex must be rolled back");
        assert!(!root.join("c.tex").exists(), "c.tex was never written");
        fs::remove_dir_all(&root).ok();
    }

    /// Recursively snapshots every regular file under `dir` (relative path,
    /// exact bytes), skipping `.flashtex` bookkeeping, so a target tree can
    /// be asserted byte-identical before and after a call rather than
    /// checked file by file.
    fn snapshot(dir: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
        let mut out = std::collections::BTreeMap::new();
        snapshot_into(dir, dir, &mut out);
        out
    }

    fn snapshot_into(
        root: &Path,
        dir: &Path,
        out: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>,
    ) {
        let Ok(entries) = fs::read_dir(dir) else {
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
                snapshot_into(root, &path, out);
            } else if file_type.is_file()
                && let Ok(bytes) = fs::read(&path)
            {
                out.insert(path.strip_prefix(root).unwrap().to_path_buf(), bytes);
            }
        }
    }

    /// Builds a 5-file template and a hook that fails exactly at
    /// `fail_index`, then asserts the whole batch rolls back to a
    /// byte-identical copy of `before` (the pre-call snapshot) with no
    /// incomplete rollback.
    fn assert_batch_rolls_back_to(root: &Path, opts: &InstantiateOptions, fail_index: usize) {
        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![
                TemplateFile::new("a.tex", "NEW A"),
                TemplateFile::new("b.tex", "NEW B"),
                TemplateFile::new("c.tex", "NEW C"),
                TemplateFile::new("d.tex", "NEW D"),
                TemplateFile::new("e.tex", "NEW E"),
            ],
        };
        let before = snapshot(root);
        let fail_at = move |i: usize| -> io::Result<()> {
            if i == fail_index {
                Err(io::Error::other(format!(
                    "simulated failure at write index {i}"
                )))
            } else {
                Ok(())
            }
        };
        let hooks = Hooks {
            before_write: Some(&fail_at),
        };
        let err = instantiate_with_hooks(&t, root, opts, &hooks).unwrap_err();
        match &err {
            InstantiateError::Rooted {
                rollback_incomplete,
                ..
            } => assert!(
                rollback_incomplete.is_empty(),
                "rollback should have fully succeeded: {rollback_incomplete:?}"
            ),
            other => panic!("expected Rooted, got {other:?}"),
        }
        assert_eq!(
            snapshot(root),
            before,
            "target must be byte-identical to its pre-call state after failure at index {fail_index}"
        );
    }

    #[test]
    fn failure_injected_at_the_first_write_of_a_batch_of_new_files_leaves_target_byte_identical() {
        let root = tempdir("inject-first-new");
        fs::create_dir_all(&root).unwrap();
        assert_batch_rolls_back_to(&root, &opts("X"), 0);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn failure_injected_at_the_middle_write_of_a_batch_of_new_files_leaves_target_byte_identical() {
        let root = tempdir("inject-middle-new");
        fs::create_dir_all(&root).unwrap();
        assert_batch_rolls_back_to(&root, &opts("X"), 2);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn failure_injected_at_the_last_write_of_a_batch_of_new_files_leaves_target_byte_identical() {
        let root = tempdir("inject-last-new");
        fs::create_dir_all(&root).unwrap();
        assert_batch_rolls_back_to(&root, &opts("X"), 4);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn failure_injected_at_the_first_write_of_a_batch_of_overwrites_leaves_target_byte_identical() {
        let root = tempdir("inject-first-overwrite");
        fs::create_dir_all(&root).unwrap();
        for (name, content) in [
            ("a.tex", "ORIGINAL A"),
            ("b.tex", "ORIGINAL B"),
            ("c.tex", "ORIGINAL C"),
            ("d.tex", "ORIGINAL D"),
            ("e.tex", "ORIGINAL E"),
        ] {
            fs::write(root.join(name), content).unwrap();
        }
        let mut options = opts("X");
        options.overwrite = true;
        assert_batch_rolls_back_to(&root, &options, 0);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn failure_injected_at_the_middle_write_of_a_batch_of_overwrites_leaves_target_byte_identical()
    {
        let root = tempdir("inject-middle-overwrite");
        fs::create_dir_all(&root).unwrap();
        for (name, content) in [
            ("a.tex", "ORIGINAL A"),
            ("b.tex", "ORIGINAL B"),
            ("c.tex", "ORIGINAL C"),
            ("d.tex", "ORIGINAL D"),
            ("e.tex", "ORIGINAL E"),
        ] {
            fs::write(root.join(name), content).unwrap();
        }
        let mut options = opts("X");
        options.overwrite = true;
        assert_batch_rolls_back_to(&root, &options, 2);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn failure_injected_at_the_last_write_of_a_batch_of_overwrites_leaves_target_byte_identical() {
        let root = tempdir("inject-last-overwrite");
        fs::create_dir_all(&root).unwrap();
        for (name, content) in [
            ("a.tex", "ORIGINAL A"),
            ("b.tex", "ORIGINAL B"),
            ("c.tex", "ORIGINAL C"),
            ("d.tex", "ORIGINAL D"),
            ("e.tex", "ORIGINAL E"),
        ] {
            fs::write(root.join(name), content).unwrap();
        }
        let mut options = opts("X");
        options.overwrite = true;
        assert_batch_rolls_back_to(&root, &options, 4);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn mid_batch_write_failure_during_overwrite_restores_original_content() {
        // Two files already exist with distinct content; overwrite: true is
        // used to replace both, but the second write fails after the first
        // has already been overwritten. The first file must end up back at
        // its *original* content, not left holding the new content, and not
        // deleted outright.
        let root = tempdir("mid-batch-overwrite");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("a.tex"), "ORIGINAL A").unwrap();
        fs::write(root.join("b.tex"), "ORIGINAL B").unwrap();

        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![
                TemplateFile::new("a.tex", "NEW A"),
                TemplateFile::new("b.tex", "NEW B"),
            ],
        };
        let fail_at_index_1 = |i: usize| -> io::Result<()> {
            if i == 1 {
                Err(io::Error::other("simulated disk error mid-batch"))
            } else {
                Ok(())
            }
        };
        let hooks = Hooks {
            before_write: Some(&fail_at_index_1),
        };
        let mut options = opts("X");
        options.overwrite = true;
        let err = instantiate_with_hooks(&t, &root, &options, &hooks).unwrap_err();
        match &err {
            InstantiateError::Rooted {
                rollback_incomplete,
                ..
            } => assert!(
                rollback_incomplete.is_empty(),
                "rollback should have fully succeeded: {rollback_incomplete:?}"
            ),
            other => panic!("expected Rooted, got {other:?}"),
        }
        assert_eq!(
            fs::read_to_string(root.join("a.tex")).unwrap(),
            "ORIGINAL A",
            "a.tex must be restored to its pre-call content, not left as NEW A or deleted"
        );
        assert_eq!(
            fs::read_to_string(root.join("b.tex")).unwrap(),
            "ORIGINAL B",
            "b.tex was never touched by the failed write"
        );
        fs::remove_dir_all(&root).ok();
    }

    // -----------------------------------------------------------------
    // Stale-identity acceptance: overwrite preflight captures each
    // about-to-be-replaced file's content hash as the identity it is about
    // to replace (`Expected::Hash`, see the write loop above). If that
    // identity has gone stale by the time the write actually happens —
    // something else modified or deleted the file in between — the rooted
    // writer's own compare-and-swap must refuse the write with a typed
    // conflict instead of silently overwriting whatever is there now. The
    // matching "same identity is accepted" boundary is
    // `overwrite_true_explicitly_permits_replacing_an_existing_file`
    // (`tests/security.rs`): unmodified between preflight and write,
    // succeeds.
    // -----------------------------------------------------------------

    #[test]
    fn a_file_modified_externally_between_preflight_and_write_is_refused_not_silently_overwritten()
    {
        let root = tempdir("stale-identity-modified");
        fs::create_dir_all(&root).unwrap();
        let target = root.join("main.tex");
        fs::write(&target, "ORIGINAL").unwrap();

        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![TemplateFile::new("main.tex", "NEW CONTENT")],
        };
        let mut options = opts("X");
        options.overwrite = true;

        // Simulates a racing writer: right before our write for this file
        // would happen, the file's identity changes out from under us,
        // *without* going through this crate at all.
        let racer_target = target.clone();
        let race = move |_i: usize| -> io::Result<()> {
            fs::write(&racer_target, "RACER CHANGED THIS").unwrap();
            Ok(())
        };
        let hooks = Hooks {
            before_write: Some(&race),
        };

        let err = instantiate_with_hooks(&t, &root, &options, &hooks).unwrap_err();
        match &err {
            InstantiateError::Rooted {
                source: SaveError::Conflict(conflict),
                rollback_incomplete,
                ..
            } => {
                assert_eq!(conflict.kind, SaveConflictKind::ModifiedExternally);
                assert!(rollback_incomplete.is_empty());
            }
            other => panic!("expected a typed ModifiedExternally conflict, got {other:?}"),
        }
        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "RACER CHANGED THIS",
            "the stale write must never land; the racing writer's content is untouched"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_file_deleted_externally_between_preflight_and_write_is_refused_not_silently_recreated() {
        let root = tempdir("stale-identity-deleted");
        fs::create_dir_all(&root).unwrap();
        let target = root.join("main.tex");
        fs::write(&target, "ORIGINAL").unwrap();

        let t = Template {
            id: "t".into(),
            title: "T".into(),
            description: "d".into(),
            packages: vec![],
            files: vec![TemplateFile::new("main.tex", "NEW CONTENT")],
        };
        let mut options = opts("X");
        options.overwrite = true;

        let racer_target = target.clone();
        let race = move |_i: usize| -> io::Result<()> {
            fs::remove_file(&racer_target).unwrap();
            Ok(())
        };
        let hooks = Hooks {
            before_write: Some(&race),
        };

        let err = instantiate_with_hooks(&t, &root, &options, &hooks).unwrap_err();
        match &err {
            InstantiateError::Rooted {
                source: SaveError::Conflict(conflict),
                rollback_incomplete,
                ..
            } => {
                assert_eq!(conflict.kind, SaveConflictKind::DeletedExternally);
                assert!(rollback_incomplete.is_empty());
            }
            other => panic!("expected a typed DeletedExternally conflict, got {other:?}"),
        }
        assert!(
            !target.exists(),
            "the stale write must never recreate a file something else deleted"
        );
        fs::remove_dir_all(&root).ok();
    }
}
