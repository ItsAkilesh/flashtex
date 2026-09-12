//! Black-box security tests against the public API only.
//!
//! These mirror the crate's acceptance criteria directly: a template file
//! path containing `..` is refused with a typed error, an absolute path is
//! refused with a typed error, and instantiating over an existing file
//! fails rather than clobbering it unless `overwrite: true` is passed.

use flashtex_project_templates::manifest::ManifestError;
use flashtex_project_templates::path::PathError;
use flashtex_project_templates::{
    InstantiateError, InstantiateOptions, Template, TemplateFile, find_template, instantiate,
};

fn temp_dir(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "flashtex-project-templates-it-{label}-{}-{}",
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
        author: "Test Author".to_string(),
        overwrite: false,
    }
}

#[test]
fn a_path_containing_dotdot_is_rejected_with_a_typed_error() {
    let root = temp_dir("dotdot");
    let template = Template {
        id: "hostile".into(),
        title: "Hostile".into(),
        description: "d".into(),
        packages: vec![],
        files: vec![TemplateFile::new(
            "../../../../tmp/flashtex-escaped.tex",
            "escaped!",
        )],
    };

    let err = instantiate(&template, &root, &options("X")).unwrap_err();
    assert!(
        matches!(
            err,
            InstantiateError::InvalidTemplate(ManifestError::InvalidPath {
                source: PathError::ParentTraversal,
                ..
            })
        ),
        "expected a typed ParentTraversal error, got {err:?}"
    );
    assert!(!root.exists(), "nothing should have been written at all");
}

#[test]
fn an_absolute_path_is_rejected_with_a_typed_error() {
    let root = temp_dir("absolute");
    let outside_target = temp_dir("absolute-outside-target").join("pwned.tex");
    let template = Template {
        id: "hostile".into(),
        title: "Hostile".into(),
        description: "d".into(),
        packages: vec![],
        files: vec![TemplateFile::new(
            outside_target.to_string_lossy().into_owned(),
            "escaped!",
        )],
    };

    let err = instantiate(&template, &root, &options("X")).unwrap_err();
    assert!(
        matches!(
            err,
            InstantiateError::InvalidTemplate(ManifestError::InvalidPath {
                source: PathError::Absolute,
                ..
            })
        ),
        "expected a typed Absolute error, got {err:?}"
    );
    assert!(
        !outside_target.exists(),
        "an absolute path must never be written to"
    );
    assert!(!root.exists());
}

#[test]
fn instantiating_over_an_existing_file_fails_without_clobbering_it() {
    let root = temp_dir("no-clobber");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("main.tex"), "ORIGINAL CONTENT, DO NOT LOSE").unwrap();

    let template = find_template("course-report").unwrap();
    let err = instantiate(&template, &root, &options("New Project")).unwrap_err();
    assert!(
        matches!(err, InstantiateError::AlreadyExists(_)),
        "expected AlreadyExists, got {err:?}"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("main.tex")).unwrap(),
        "ORIGINAL CONTENT, DO NOT LOSE",
        "the pre-existing file must not have been touched"
    );

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn overwrite_true_explicitly_permits_replacing_an_existing_file() {
    let root = temp_dir("clobber-allowed");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("main.tex"), "OLD CONTENT").unwrap();

    let template = find_template("course-report").unwrap();
    let mut opts = options("New Project");
    opts.overwrite = true;
    instantiate(&template, &root, &opts).unwrap();
    let contents = std::fs::read_to_string(root.join("main.tex")).unwrap();
    assert!(contents.contains("New Project"));
    assert!(!contents.contains("OLD CONTENT"));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn a_project_name_with_non_ascii_characters_instantiates_every_builtin_template() {
    for template in flashtex_project_templates::all_templates() {
        let root = temp_dir(&format!("unicode-{}", template.id));
        let opts = InstantiateOptions {
            project_name: "Análisis Estadístico — 統計分析 — Статистический анализ".to_string(),
            author: "José Núñez".to_string(),
            overwrite: false,
        };
        let report = instantiate(&template, &root, &opts).unwrap();
        assert_eq!(report.written_files.len(), template.files.len());
        let main = std::fs::read_to_string(root.join("main.tex")).unwrap();
        assert!(main.contains("Análisis Estadístico"));
        assert!(main.contains("統計分析"));
        std::fs::remove_dir_all(&root).ok();
    }
}
