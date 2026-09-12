//! Worker-thread editor controller. Durable source precedes disposable caches.
pub mod file_project;
use flashtex_document_runtime::{Document as InputDocument, Event, Limits, Request, Session};
use flashtex_edit_ledger::history::{GroupedEdit, HistoryMove, HistoryResult, HistoryStatus};
use flashtex_edit_ledger::{AppliedReceipt, AppliedTransaction, Document, PreparedEdit, Store};
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
/// An explicit application boundary: construct only from the user's approval
/// action after displaying the exact prepared edit. This type is not a verifier
/// of human intent and must never be constructed automatically by conversion.
pub struct ApprovedEdit(PreparedEdit);
impl ApprovedEdit {
    pub fn from_explicit_user_approval(edit: PreparedEdit) -> Self {
        Self(edit)
    }
}
pub enum HistoryAction {
    Group(GroupedEdit),
    Undo(HistoryMove),
    Redo(HistoryMove),
}
#[derive(Debug)]
pub struct HistoryOutcome {
    pub history: HistoryResult,
    pub source: EditOutcome,
}
#[derive(Debug)]
pub struct AppliedOutcome {
    pub receipt: AppliedReceipt,
    pub source: EditOutcome,
}
#[derive(Debug)]
pub struct Preview {
    pub missing_layout_capabilities: Vec<String>,
    pub request_id: String,
    pub compile_revision: u64,
    pub source_versions: VersionSnapshot,
    pub result: Value,
    pub runtime_total_ms: f64,
    /// Controller invocation through observed result, including successful save/index work.
    pub controller_total_ms: f64,
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
    runtime: Option<Session>,
    generation: u64,
    layout_capabilities: Vec<String>,
    submitted: Option<(String, VersionSnapshot, Instant)>,
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
        let mut controller = Self::open_without_compiler(project_id, entry_path, stores)?;
        controller.runtime = Some(Session::spawn_command(command, limits)?);
        Ok(controller)
    }
    /// Open authoritative source and navigation even when no compiler is available.
    pub fn open_without_compiler(
        project_id: String,
        entry_path: String,
        stores: Vec<Store>,
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
            runtime: None,
            generation: 0,
            layout_capabilities: Vec::new(),
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
        Ok(self.after_save(document, started))
    }
    fn after_save(&mut self, document: Document, started: Instant) -> EditOutcome {
        self.submitted = None;
        let indexed =
            if self.index.snapshot().documents.get(&document.path) == Some(&document.revision) {
                Ok(())
            } else {
                self.index
                    .replace_document(&document.path, document.revision, &document.text)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            };
        let preview_error = match indexed {
            Ok(()) => self.compile_current().err(),
            Err(error) => Some(format!("source saved; index recovery required: {error}")),
        };
        if preview_error.is_none() {
            if let Some((_, _, submitted_at)) = self.submitted.as_mut() {
                *submitted_at = started;
            }
        }
        EditOutcome {
            document,
            preview_error,
            save_and_submit_ms: started.elapsed().as_secs_f64() * 1000.0,
        }
    }
    /// The returned receipt is durable before any compile attempt. A matching
    /// retry returns the original receipt and cannot apply the source edit twice.
    pub fn apply_reviewed(&mut self, approved: ApprovedEdit) -> Result<AppliedOutcome, String> {
        if self.closed {
            return Err("project closed".into());
        }
        let started = Instant::now();
        self.submitted = None;
        let path = approved.0.path.clone();
        let store = self.stores.get_mut(&path).ok_or("unknown document")?;
        let receipt = store.apply(approved.0).map_err(|e| e.to_string())?;
        let document = store
            .document()
            .map_err(|e| e.to_string())?
            .ok_or("uninitialized document")?
            .clone();
        Ok(AppliedOutcome {
            receipt,
            source: self.after_save(document, started),
        })
    }
    /// Pending durable receipts for bridge reconciliation, including after reopen.
    pub fn recovery(&self, path: &str) -> Result<Vec<AppliedTransaction>, String> {
        self.stores
            .get(path)
            .ok_or("unknown document")?
            .recovery()
            .map_err(|e| e.to_string())
    }
    /// Invoke only after the bridge acknowledges this exact applied receipt.
    pub fn confirm_receipt(&mut self, path: &str, receipt: &AppliedReceipt) -> Result<(), String> {
        self.stores
            .get_mut(path)
            .ok_or("unknown document")?
            .confirm(receipt)
            .map_err(|e| e.to_string())
    }
    pub fn history_status(&self, path: &str) -> Result<HistoryStatus, String> {
        self.stores
            .get(path)
            .ok_or("unknown document")?
            .history_status()
            .map_err(|e| e.to_string())
    }
    /// Ordinary explicitly requested editor history operations. Permanent command
    /// IDs and source changes are committed by the authoritative ledger together.
    pub fn apply_history(
        &mut self,
        path: &str,
        action: HistoryAction,
    ) -> Result<HistoryOutcome, String> {
        if self.closed {
            return Err("project closed".into());
        }
        let started = Instant::now();
        self.submitted = None;
        let store = self.stores.get_mut(path).ok_or("unknown document")?;
        let history = match action {
            HistoryAction::Group(group) => store.apply_group(group),
            HistoryAction::Undo(command) => store.undo(command),
            HistoryAction::Redo(command) => store.redo(command),
        }
        .map_err(|e| e.to_string())?;
        let source = self.after_save(history.document.clone(), started);
        Ok(HistoryOutcome { history, source })
    }
    /// Explicit native opt-in after implementing the requested draw capabilities.
    /// Empty restores legacy output. Every subsequent compile binds this request.
    pub fn configure_layout(&mut self, capabilities: Vec<String>) -> Result<(), String> {
        if self.closed {
            return Err("project closed".into());
        }
        flashtex_document_runtime::validate_layout_capabilities(&capabilities)?;
        self.layout_capabilities = capabilities;
        self.submitted = None;
        self.compile_current()
    }
    pub fn compile_current(&mut self) -> Result<(), String> {
        let started = Instant::now();
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
        self.runtime
            .as_mut()
            .ok_or("compiler unavailable; source remains saved")?
            .submit_with_capabilities(
                Request {
                    id: id.clone(),
                    project_id: self.project_id.clone(),
                    revision: generation,
                    entry_path: self.entry_path.clone(),
                    documents,
                },
                self.layout_capabilities.clone(),
            )?;
        self.generation = generation;
        self.submitted = Some((id, self.index.snapshot(), started));
        Ok(())
    }
    /// Recheck immediately before applying a retained result on the UI thread.
    /// Dispatching a preview event is not permission to paint it after a newer edit.
    pub fn is_current_preview(&self, preview: &Preview) -> bool {
        !self.closed
            && self.submitted.as_ref().is_some_and(|(id, snapshot, _)| {
                id == &preview.request_id
                    && snapshot == &preview.source_versions
                    && snapshot == &self.index.snapshot()
            })
    }
    pub fn poll(&mut self) -> Vec<Update> {
        let current = self.index.snapshot();
        let Some(runtime) = self.runtime.as_mut() else {
            return Vec::new();
        };
        runtime
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
                        && self
                            .submitted
                            .as_ref()
                            .is_some_and(|(expected, snapshot, _)| {
                                expected == &id && snapshot == &current
                            })
                    {
                        let accepted = result["payload"]["layout_capabilities"].as_array();
                        let missing_layout_capabilities = self
                            .layout_capabilities
                            .iter()
                            .filter(|cap| {
                                !accepted.is_some_and(|items| {
                                    items.iter().any(|item| item.as_str() == Some(cap.as_str()))
                                })
                            })
                            .cloned()
                            .collect();
                        Update::Preview(Preview {
                            missing_layout_capabilities,
                            request_id: id,
                            compile_revision: revision,
                            source_versions: current.clone(),
                            result,
                            runtime_total_ms: total_ms,
                            controller_total_ms: self
                                .submitted
                                .as_ref()
                                .unwrap()
                                .2
                                .elapsed()
                                .as_secs_f64()
                                * 1000.0,
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
        self.runtime = Some(runtime);
        self.index = index;
        self.submitted = None;
        self.compile_current()
    }
    pub fn close(&mut self) -> Result<(), String> {
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.close_project(&self.project_id)?;
        }
        self.closed = true;
        self.submitted = None;
        Ok(())
    }
}
