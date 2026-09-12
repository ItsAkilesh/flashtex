//! Durable edit acknowledgement without copying source into the response.
use crate::Controller;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Serialize)]
pub struct DocumentMetadata {
    pub project_id: String,
    pub path: String,
    pub revision: u64,
    pub source_sha256: String,
    pub byte_length: usize,
}
#[derive(Debug, Serialize)]
pub struct MetadataEditOutcome {
    pub document: DocumentMetadata,
    pub preview_error: Option<String>,
    pub save_and_submit_ms: f64,
}
impl Controller {
    pub fn replace_document_metadata(
        &mut self,
        path: &str,
        expected_revision: u64,
        expected_sha256: &str,
        text: String,
    ) -> Result<MetadataEditOutcome, String> {
        if self.closed {
            return Err("project closed".into());
        }
        let started = Instant::now();
        self.submitted = None;
        let store = self.stores.get_mut(path).ok_or("unknown document")?;
        store
            .replace_document_in_place(expected_revision, expected_sha256, text)
            .map_err(|e| e.to_string())?;
        let saved = store
            .document()
            .map_err(|e| e.to_string())?
            .ok_or("saved source missing")?;
        let document = DocumentMetadata {
            project_id: saved.project_id.clone(),
            path: saved.path.clone(),
            revision: saved.revision,
            source_sha256: saved.source_sha256.clone(),
            byte_length: saved.text.len(),
        };
        let indexed = Self::index_saved_document(&mut self.index, saved);
        let (preview_error, save_and_submit_ms) = self.finish_saved_index(indexed, started);
        Ok(MetadataEditOutcome {
            document,
            preview_error,
            save_and_submit_ms,
        })
    }
}
