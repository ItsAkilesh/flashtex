//! Current-source correlation only; forwarded display data still needs renderer validation.
use crate::Controller;
use serde_json::{json, Value};

impl Controller {
    /// Returns a separate compile error after successful policy negotiation.
    /// No already-submitted work is retroactively enrolled.
    pub fn configure_display_candidates(
        &mut self,
        enabled: bool,
    ) -> Result<Option<String>, String> {
        if self.closed {
            return Err("project closed".into());
        }
        if enabled && self.historical.enabled {
            return Err(
                "display candidates and historical snapshots are mutually exclusive".into(),
            );
        }
        if enabled
            && !self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.is_alive())
        {
            return Err("compiler unavailable".into());
        }
        let mut capabilities = self.layout_capabilities.clone();
        capabilities.retain(|cap| cap != "display-list-v2");
        if enabled {
            capabilities.push("display-list-v2".into());
        }
        flashtex_document_runtime::validate_layout_capabilities(&capabilities)?;
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.set_display_candidates_enabled(enabled)?;
        }
        self.display_enabled = enabled;
        self.layout_capabilities = capabilities;
        self.submitted = None;
        Ok(self.compile_current().err())
    }

    /// Call only when optional output can be admitted. Moves one candidate, then
    /// checks controller identity on the serialized owner before constructing output.
    pub fn take_current_display_payload(&mut self) -> Option<Value> {
        if self.closed || !self.display_enabled {
            return None;
        }
        let candidate = self.runtime.as_mut()?.take_current_display_candidate()?;
        let (id, submitted, _) = self.submitted.as_ref()?;
        let current = self.index.snapshot();
        if candidate.request_id() != id
            || candidate.project_id() != self.project_id
            || candidate.revision() != self.generation
            || submitted != &current
            || candidate.sources().len() != current.documents.len()
        {
            return None;
        }
        for source in candidate.sources() {
            let document = self.document(&source.path).ok()?;
            if current.documents.get(&source.path) != Some(&document.revision)
                || source.sha256 != document.source_sha256
                || source.byte_length != document.text.len()
            {
                return None;
            }
        }
        let mut payload = json!({"kind":"display_candidate","untrusted":true,
            "source_actions_enabled":false,"request_id":candidate.request_id(),
            "project_id":candidate.project_id(),"compile_revision":candidate.revision(),
            "source_versions":current.documents,"membership_generation":current.generation});
        payload["display_list"] = candidate.into_envelope();
        Some(payload)
    }
}
