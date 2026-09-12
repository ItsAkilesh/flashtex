//! Exact proposal review to a single-document grouped ledger request. No writes.
use crate::{CompileBinding, Context, ExplanationProposal};
use flashtex_edit_ledger::{
    history::{GroupedEdit, SourceEdit},
    Document,
};
use flashtex_project_files::sha256_hex;
use serde::Serialize;

pub struct ProposalReview {
    request_id: String,
    review_id: String,
    binding: CompileBinding,
    proposal: ExplanationProposal,
}
#[derive(Serialize)]
pub struct ApprovedGroup {
    pub project_id: String,
    pub path: String,
    pub request_id: String,
    pub review_id: String,
    pub group: GroupedEdit,
}
impl ProposalReview {
    /// Revalidate exact provider JSON against the supplied immutable context and
    /// current snapshot. Expose this review before asking the user to approve it.
    pub fn prepare(
        request_id: &str,
        context: &Context,
        response: &[u8],
        current: &[Document],
    ) -> Result<Self, String> {
        if request_id.is_empty()
            || request_id.len() > 256
            || request_id.chars().any(char::is_control)
        {
            return Err("invalid explanation request identity".into());
        }
        let proposal = context.validate_response(response, current)?;
        if proposal.edits.is_empty() {
            return Err("explanation has no proposed edits".into());
        }
        let first = &proposal.edits[0].location.path;
        if proposal
            .edits
            .iter()
            .any(|edit| &edit.location.path != first)
        {
            return Err(
                "multi-document atomic approval is unsupported; no partial edit prepared".into(),
            );
        }
        let review_id = sha256_hex(
            &serde_json::to_vec(&(request_id, &context.binding, &proposal))
                .map_err(|e| e.to_string())?,
        );
        Ok(Self {
            request_id: request_id.to_owned(),
            review_id,
            binding: context.binding.clone(),
            proposal,
        })
    }
    pub fn review_id(&self) -> &str {
        &self.review_id
    }
    pub fn proposal(&self) -> &ExplanationProposal {
        &self.proposal
    }
    /// Approval must identify this exact review. The host owns the actual user
    /// interaction; a boolean/token is not independent proof of user consent.
    pub fn approve(
        &self,
        user_approved: bool,
        approved_review_id: &str,
        current: &[Document],
    ) -> Result<ApprovedGroup, String> {
        if !user_approved || approved_review_id != self.review_id {
            return Err("explicit approval of exact review required".into());
        }
        self.binding.check(current)?;
        let path = self.proposal.edits[0].location.path.clone();
        let source = &self.binding.sources[&path];
        Ok(ApprovedGroup {
            project_id: self.binding.project_id.clone(),
            path,
            request_id: self.request_id.clone(),
            review_id: self.review_id.clone(),
            group: GroupedEdit {
                command_id: format!("assistant-{}", self.review_id),
                expected_revision: source.revision,
                expected_sha256: source.sha256.clone(),
                label: "Apply reviewed AI proposal".into(),
                edits: self
                    .proposal
                    .edits
                    .iter()
                    .map(|edit| SourceEdit {
                        start_byte: edit.location.start_byte,
                        end_byte: edit.location.end_byte,
                        removed_text: edit.removed_text.clone(),
                        replacement: edit.replacement.clone(),
                    })
                    .collect(),
            },
        })
    }
}
impl ApprovedGroup {
    /// Check the selected ledger's document identity before handing it the group.
    /// Retry the unchanged group directly through the ledger's command receipt;
    /// rebuilding a review against the post-edit source is deliberately refused.
    pub fn check_target(&self, document: &Document) -> Result<(), String> {
        if document.project_id != self.project_id || document.path != self.path {
            return Err("approved group targets a different document".into());
        }
        Ok(())
    }
}
