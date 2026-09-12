//! Disk discovery imports into authoritative private ledgers. Export remains
//! explicitly unavailable until the shared rooted-save primitive is ready.
use crate::Controller;
use flashtex_edit_ledger::{Document, Store};
use flashtex_project_files::{sha256_hex, DiscoverError, Overlay, ProjectGraph, ProjectPath};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiskState {
    MatchesSource {
        sha256: String,
    },
    DiffersFromSource {
        disk_sha256: String,
        source_sha256: String,
    },
    Missing,
    Unavailable {
        reason: String,
    },
}
pub struct FileProject {
    root: PathBuf,
    project_id: String,
    diagnostics: Vec<String>,
}
impl FileProject {
    /// Existing ledgers take precedence over disk. Discovery uses their source as
    /// overlays so an externally deleted entry can still reopen without data loss.
    /// `private_root` must already be an application-owned directory.
    pub fn open(
        root: &Path,
        private_root: &Path,
        project_id: &str,
        entry: &str,
    ) -> Result<(Self, Controller), String> {
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        if !root.is_dir() {
            return Err("project root is not a directory".into());
        }
        let private_root = private_root.canonicalize().map_err(|e| e.to_string())?;
        if !private_root.is_dir() {
            return Err("private ledger root is not a directory".into());
        }
        let entry = ProjectPath::normalize(entry).map_err(|e| e.to_string())?;
        let root_text = root
            .to_str()
            .ok_or("project root must be UTF-8 for stable binding")?;
        let binding =
            sha256_hex(format!("{}:{root_text}:{project_id}", root_text.len()).as_bytes());
        let ledger_root = private_root.join(format!("project-{binding}"));
        if fs::symlink_metadata(&ledger_root).is_ok_and(|meta| meta.file_type().is_symlink()) {
            return Err("private ledger binding cannot be a symlink".into());
        }
        fs::create_dir_all(&ledger_root).map_err(|e| e.to_string())?;
        let mut stores = BTreeMap::new();
        let mut overlay = Overlay::new();
        for item in fs::read_dir(&ledger_root).map_err(|e| e.to_string())? {
            let item = item.map_err(|e| e.to_string())?;
            let name = item
                .file_name()
                .into_string()
                .map_err(|_| "invalid ledger name")?;
            if name.len() != 64 || !name.bytes().all(|b| b.is_ascii_hexdigit()) {
                continue;
            }
            if !item.file_type().map_err(|e| e.to_string())?.is_dir() {
                return Err("ledger slot is not a private directory".into());
            }
            let store = Store::open(item.path()).map_err(|e| e.to_string())?;
            if let Some(document) = store.document().map_err(|e| e.to_string())? {
                let path = ProjectPath::normalize(&document.path).map_err(|e| e.to_string())?;
                if document.project_id != project_id
                    || path.as_str() != document.path
                    || sha256_hex(document.path.as_bytes()) != name
                {
                    return Err("ledger binding disagrees with document identity".into());
                }
                overlay.insert(path, document.text.clone());
                stores.insert(document.path.clone(), store);
                if stores.len() > 256 {
                    return Err("project exceeds 256 source stores".into());
                }
            }
        }
        if overlay.get(&entry).is_none() {
            check_entry(&root, &entry)?;
        }
        let graph =
            ProjectGraph::discover_with(&root, &entry, &overlay).map_err(|e| e.to_string())?;
        for file in graph.files() {
            let Some(text) = &file.text else {
                continue;
            };
            if stores.contains_key(file.path.as_str()) {
                continue;
            }
            if stores.len() >= 256 {
                return Err("project exceeds 256 source stores".into());
            }
            let mut store =
                Store::open(ledger_root.join(sha256_hex(file.path.as_str().as_bytes())))
                    .map_err(|e| e.to_string())?;
            store
                .initialize(
                    Document::new(
                        project_id.into(),
                        file.path.as_str().into(),
                        1,
                        text.clone(),
                    )
                    .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
            stores.insert(file.path.as_str().to_owned(), store);
        }
        let diagnostics = graph
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.message.clone())
            .collect();
        let controller = Controller::open_without_compiler(
            project_id.into(),
            entry.as_str().into(),
            stores.into_values().collect(),
        )?;
        Ok((
            Self {
                root,
                project_id: project_id.into(),
                diagnostics,
            },
            controller,
        ))
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }
    /// Always rereads the current disk graph. No mtime-only shortcut; this check
    /// never changes source, advances a disk baseline or implies save approval.
    pub fn inspect(&self, controller: &Controller, path: &str) -> Result<DiskState, String> {
        let source = controller.document(path)?;
        if source.project_id != self.project_id {
            return Err("controller belongs to another project".into());
        }
        let path = ProjectPath::normalize(path).map_err(|e| e.to_string())?;
        if let Err(reason) = check_entry(&self.root, &path) {
            return Ok(DiskState::Unavailable { reason });
        }
        match ProjectGraph::discover(&self.root, &path) {
            Ok(graph) => {
                let file = graph
                    .files()
                    .iter()
                    .find(|file| file.path == path)
                    .ok_or("entry absent from disk graph")?;
                let hash = file
                    .sha256
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>();
                Ok(if hash == source.source_sha256 {
                    DiskState::MatchesSource { sha256: hash }
                } else {
                    DiskState::DiffersFromSource {
                        disk_sha256: hash,
                        source_sha256: source.source_sha256.clone(),
                    }
                })
            }
            Err(DiscoverError::EntryMissing(_)) => Ok(DiskState::Missing),
            Err(error) => Ok(DiskState::Unavailable {
                reason: error.to_string(),
            }),
        }
    }
    /// This API intentionally refuses export while GH18's rooted-save guarantees
    /// are unresolved; it never calls the shared unconfined save implementation.
    pub fn export(
        &self,
        _controller: &Controller,
        _path: &str,
        _expected_disk_sha256: Option<&str>,
    ) -> Result<(), String> {
        Err(
            "export unavailable: shared rooted-save confinement/conflict gate GH18 unresolved"
                .into(),
        )
    }
}

// Consumer preflight for the shared graph's unchecked entry path. This is not a
// race-proof directory capability; shared rooted IO is tracked in GH18.
fn check_entry(root: &Path, path: &ProjectPath) -> Result<(), String> {
    match path.to_os_path(root).canonicalize() {
        Ok(actual) if actual.starts_with(root) => Ok(()),
        Ok(_) => Err("entry resolves outside bound project root".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}
