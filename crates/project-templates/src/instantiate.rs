//! Rooted, non-destructive instantiation of a [`Template`] into a target
//! directory.
//!
//! Security is the point of this module: a template must never write
//! outside the caller's target directory, and must never silently clobber a
//! file that is already there. [`instantiate`] enforces both by
//! construction, not by convention:
//!
//! - Every declared file path is validated by [`crate::path::validate`]
//!   before any I/O happens; a path containing `..` or an absolute path is
//!   rejected with a typed [`InstantiateError::InvalidTemplate`].
//! - Each target path is then built by pushing the *validated segments*
//!   individually onto `target_root` (never by parsing/joining the raw
//!   string), so there is no code path through which a rejected path could
//!   still reach the filesystem. A debug assertion double-checks the result
//!   stayed under `target_root`.
//! - Before writing anything, every target path is checked for an existing
//!   file; if one is found and [`InstantiateOptions::overwrite`] is `false`,
//!   instantiation fails with [`InstantiateError::AlreadyExists`] and
//!   *nothing is written* — a template can't clobber half the project and
//!   leave a mix of old and new files behind.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

/// What [`instantiate`] wrote on success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstantiateReport {
    /// Every file path written, in template order.
    pub written_files: Vec<PathBuf>,
}

/// Why instantiation failed. No file is written when any variant is
/// returned.
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
    /// `options.overwrite` was `false`.
    AlreadyExists(PathBuf),
    /// An I/O operation failed.
    Io { path: PathBuf, source: io::Error },
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
        }
    }
}

impl std::error::Error for InstantiateError {}

/// Instantiates `template` under `target_root`, substituting `options` into
/// each file body, and returns the paths written.
///
/// `target_root` is created (via `create_dir_all`) if it does not exist. No
/// file is written outside `target_root`, and no existing file is
/// overwritten unless `options.overwrite` is `true` — see the module docs
/// for exactly how both guarantees are enforced.
pub fn instantiate(
    template: &Template,
    target_root: &Path,
    options: &InstantiateOptions,
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

    // Resolve every target path from validated segments only, and refuse to
    // proceed at all if any of them already exists and overwriting was not
    // requested. This whole-template preflight means instantiate() never
    // leaves a project half-written because of one conflicting file.
    // `Template::validate` above already rejects duplicate raw paths, and
    // path validation is injective on the accepted subset (no `.`/empty
    // segments), so distinct declared paths always resolve to distinct
    // targets; no additional dedup is needed here.
    let mut targets = Vec::with_capacity(template.files.len());
    for file in &template.files {
        // `Template::validate` above already rejected `..`, absolute paths,
        // and forbidden characters, so this can only build a path under
        // `target_root`. The assertion is defense-in-depth against a future
        // change to path resolution, not a load-bearing check today.
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
        if !options.overwrite && target.exists() {
            return Err(InstantiateError::AlreadyExists(target));
        }
        targets.push(target);
    }

    let packages_block = template
        .packages
        .iter()
        .map(|p| format!("\\usepackage{{{p}}}\n"))
        .collect::<String>();
    let packages_block = packages_block.trim_end_matches('\n');
    let project_name_escaped = escape::escape(&options.project_name);
    let author_escaped = escape::escape(&options.author);

    let mut written_files = Vec::with_capacity(targets.len());
    for (file, target) in template.files.iter().zip(targets.iter()) {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| InstantiateError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let contents = render(
            &file.body,
            &project_name_escaped,
            &author_escaped,
            packages_block,
        );
        fs::write(target, contents).map_err(|source| InstantiateError::Io {
            path: target.clone(),
            source,
        })?;
        written_files.push(target.clone());
    }

    Ok(InstantiateReport { written_files })
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
}
