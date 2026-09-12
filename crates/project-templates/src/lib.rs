//! FlashTeX original undergraduate project templates (original Rust, no
//! external crates).
//!
//! - [`registry`]: the built-in template list (`course-report`,
//!   `senior-thesis`, `problem-set`, `lab-notebook`) and lookup by id.
//! - [`manifest`]: [`Template`] / [`TemplateFile`], the bounded manifest
//!   shape, and its validation.
//! - [`instantiate`]: the rooted, non-destructive instantiation API —
//!   writing a template to a target directory can never escape that
//!   directory and never silently overwrites an existing file.
//! - [`path`], [`field`], [`package`]: the typed validation each of the
//!   above builds on (file paths, free-text fields, LaTeX package names).
//! - [`escape`]: LaTeX-safe escaping applied to substituted field values.
//!
//! ```
//! use flashtex_project_templates::{find_template, instantiate, InstantiateOptions};
//!
//! let template = find_template("course-report").unwrap();
//! let dir = std::env::temp_dir().join("flashtex-project-templates-doctest");
//! let options = InstantiateOptions {
//!     project_name: "Optics Lab Report".to_string(),
//!     author: "A. Student".to_string(),
//!     overwrite: false,
//! };
//! let report = instantiate(&template, &dir, &options).unwrap();
//! assert_eq!(report.written_files.len(), 1);
//! # std::fs::remove_dir_all(&dir).ok();
//! ```

pub mod escape;
pub mod field;
pub mod instantiate;
pub mod manifest;
pub mod package;
pub mod path;
pub mod registry;

pub use instantiate::{
    CreationRecord, InstantiateError, InstantiateOptions, InstantiateReport, instantiate,
};
pub use manifest::{ManifestError, Template, TemplateFile};
pub use registry::{all_templates, find_template};
