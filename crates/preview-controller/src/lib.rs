//! Worker-thread editor controller. Durable source precedes disposable caches.
use flashtex_document_runtime::{Document as InputDocument, Event, Limits, Request, Session};
use flashtex_edit_ledger::{Document, Store};
use flashtex_project_index::{ProjectIndex, VersionSnapshot};
use serde_json::Value;
use std::{collections::BTreeMap, process::Command, time::Instant};

#[derive(Debug)]
pub struct EditOutcome {
    /// This exact source is durable even when preview submission fails.
    pub document: Document,
    pub preview_error: Option<String>,
    /// Includes fsync/index/submission; excludes compiler completion and paint.
    pub save_and_submit_ms: f64,
}
#[derive(Debug)]
pub struct Preview {
    pub request_id: String,
    pub compile_revision: u64,
    pub source_versions: VersionSnapshot,
    pub result: Value,
    pub runtime_total_ms: f64,
}
#[derive(Debug)]
pub enum Update {
    Preview(Preview),
    Discarded { request_id: String },
    Runtime(Event),
}

pub struct Controller {
    project_id: String,
    entry_path: String,
    stores: BTreeMap<String, Store>,
    index: ProjectIndex,
    runtime: Session,
    generation: u64,
    submitted: Option<(String, VersionSnapshot)>,
    closed: bool,
}
impl Controller {
    /// All stores must already contain initialized durable documents. Ownership of
    /// their exclusive locks transfers here. No source is imported or overwritten.
    pub fn new(
        project_id: String,
        entry_path: String,
        stores: Vec<Store>,
        command: Command,
        limits: Limits,
    ) -> Result<Self, String> {
        let mut by_path = BTreeMap::new();
        let mut index = ProjectIndex::new(&project_id).map_err(|e| e.to_string())?;
        for store in stores {
            let document = store
                .document()
                .map_err(|e| e.to_string())?
                .ok_or("uninitialized document store")?;
            if document.project_id != project_id || by_path.contains_key(&document.path) {
                return Err("wrong project or duplicate document store".into());
            }
            index
                .replace_document(&document.path, document.revision, &document.text)
                .map_err(|e| e.to_string())?;
            by_path.insert(document.path.clone(), store);
        }
        if !by_path.contains_key(&entry_path) {
            return Err("entry store missing".into());
        }
        Ok(Self {
            project_id,
            entry_path,
            stores: by_path,
            index,
            runtime: Session::spawn_command(command, limits)?,
            generation: 0,
            submitted: None,
            closed: false,
        })
    }
    pub fn index(&self) -> &ProjectIndex {
        &self.index
    }
    pub fn document(&self, path: &str) -> Result<&Document, String> {
        self.stores
            .get(path)
            .ok_or("unknown document")?
            .document()
            .map_err(|e| e.to_string())?
            .ok_or("uninitialized document".into())
    }
    /// An Err means the save did not report success; on storage uncertainty reopen
    /// the authoritative ledger before retry. A compile error is an Ok outcome.
    pub fn replace_document(
        &mut self,
        path: &str,
        expected_revision: u64,
        expected_sha256: &str,
        text: String,
    ) -> Result<EditOutcome, String> {
        if self.closed {
            return Err("project closed".into());
        }
        let started = Instant::now();
        self.submitted = None;
        let document = self
            .stores
            .get_mut(path)
            .ok_or("unknown document")?
            .replace_document(expected_revision, expected_sha256, text)
            .map_err(|e| e.to_string())?;
        self.submitted = None;
        let indexed = self
            .index
            .replace_document(path, document.revision, &document.text)
            .map_err(|e| e.to_string());
        let preview_error = match indexed {
            Ok(_) => self.compile_current().err(),
            Err(error) => Some(format!("source saved; index recovery required: {error}")),
        };
        Ok(EditOutcome {
            document,
            preview_error,
            save_and_submit_ms: started.elapsed().as_secs_f64() * 1000.0,
        })
    }
    pub fn compile_current(&mut self) -> Result<(), String> {
        if self.closed {
            return Err("project closed".into());
        }
        let generation = self
            .generation
            .checked_add(1)
            .filter(|g| *g < (1u64 << 53))
            .ok_or("compile revision exhausted")?;
        let mut documents = Vec::new();
        let indexed = self.index.snapshot();
        for store in self.stores.values() {
            let document = store
                .document()
                .map_err(|e| e.to_string())?
                .ok_or("uninitialized document")?;
            if indexed.documents.get(&document.path) != Some(&document.revision) {
                return Err("index differs from durable source; restart required".into());
            }
            documents.push(InputDocument {
                path: document.path.clone(),
                text: document.text.clone(),
            });
        }
        let id = format!("preview-{generation}");
        self.runtime.submit(Request {
            id: id.clone(),
            project_id: self.project_id.clone(),
            revision: generation,
            entry_path: self.entry_path.clone(),
            documents,
        })?;
        self.generation = generation;
        self.submitted = Some((id, self.index.snapshot()));
        Ok(())
    }
    /// Recheck immediately before applying a retained result on the UI thread.
    /// Dispatching a preview event is not permission to paint it after a newer edit.
    pub fn is_current_preview(&self, preview: &Preview) -> bool {
        !self.closed
            && self.submitted.as_ref().is_some_and(|(id, snapshot)| {
                id == &preview.request_id
                    && snapshot == &preview.source_versions
                    && snapshot == &self.index.snapshot()
            })
    }
    pub fn poll(&mut self) -> Vec<Update> {
        let current = self.index.snapshot();
        self.runtime
            .poll()
            .into_iter()
            .map(|event| match event {
                Event::Preview {
                    id,
                    revision,
                    result,
                    total_ms,
                    ..
                } => {
                    if !self.closed
                        && self.submitted.as_ref().is_some_and(|(expected, snapshot)| {
                            expected == &id && snapshot == &current
                        })
                    {
                        Update::Preview(Preview {
                            request_id: id,
                            compile_revision: revision,
                            source_versions: current.clone(),
                            result,
                            runtime_total_ms: total_ms,
                        })
                    } else {
                        Update::Discarded { request_id: id }
                    }
                }
                other => Update::Runtime(other),
            })
            .collect()
    }
    /// Rebuild disposable index and compiler session from locked durable sources.
    pub fn restart(&mut self, command: Command, limits: Limits) -> Result<(), String> {
        if self.closed {
            return Err("project closed".into());
        }
        let mut index = ProjectIndex::new(&self.project_id).map_err(|e| e.to_string())?;
        for store in self.stores.values() {
            let document = store
                .document()
                .map_err(|e| e.to_string())?
                .ok_or("uninitialized document")?;
            index
                .replace_document(&document.path, document.revision, &document.text)
                .map_err(|e| e.to_string())?;
        }
        let runtime = Session::spawn_command(command, limits)?;
        self.runtime = runtime;
        self.index = index;
        self.submitted = None;
        self.compile_current()
    }
    pub fn close(&mut self) -> Result<(), String> {
        self.runtime.close_project(&self.project_id)?;
        self.closed = true;
        self.submitted = None;
        Ok(())
    }
}
