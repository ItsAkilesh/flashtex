//! Opt-in helper candidate binding. The caller supplies authoritative current
//! controller state; untrusted event fields never create freshness authority.
//! Editor document versions are distinct from the compile generation in v2.
//! Successful export grants no native paint or source-navigation authority.
use crate::{
    pipeline_cff::{PipelineCff, SearchablePdf},
    *,
};
use flashtex_font_resources::registry::CffFontResource;
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentSource {
    pub editor_revision: u64,
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentHelper {
    pub session_id: String,
    pub project_id: String,
    pub request_id: String,
    pub compile_revision: u64,
    pub membership_generation: u64,
    pub sources: BTreeMap<String, CurrentSource>,
}
pub struct BoundHelperCandidate {
    current: CurrentHelper,
    display: PipelineCff,
}
impl BoundHelperCandidate {
    /// A fresh authoritative snapshot is required at every export. An A→B→A
    /// transition must change membership/compile generation in the controller.
    pub fn export_searchable(
        &self,
        current: &CurrentHelper,
        max_bytes: usize,
    ) -> Result<SearchablePdf> {
        require(current == &self.current, "stale helper candidate")?;
        self.display.export_searchable(max_bytes)
    }
}
pub fn bind(
    event: &[u8],
    result: &[u8],
    current: &CurrentHelper,
    capabilities: &Capabilities,
    resources: &BTreeMap<String, Arc<CffFontResource>>,
) -> Result<BoundHelperCandidate> {
    require(
        event.len() <= pipeline_frame::MAX_REPLY_LINE,
        "helper event byte limit",
    )?;
    require(
        !current.sources.is_empty() && current.sources.len() <= 256,
        "helper source count",
    )?;
    let value: Value =
        serde_json::from_slice(event).map_err(|e| ValidationError(format!("helper JSON: {e}")))?;
    let p = &value["payload"];
    require(
        value["protocol_version"] == 1
            && value["type"] == "update"
            && value["session_id"].as_str() == Some(current.session_id.as_str())
            && p["kind"] == "display_candidate"
            && p["untrusted"] == true
            && p["source_actions_enabled"] == false,
        "helper envelope/policy mismatch",
    )?;
    require(
        p["project_id"].as_str() == Some(current.project_id.as_str())
            && p["request_id"].as_str() == Some(current.request_id.as_str())
            && p["compile_revision"].as_u64() == Some(current.compile_revision)
            && p["membership_generation"].as_u64() == Some(current.membership_generation),
        "stale helper identity",
    )?;
    let versions = p["source_versions"]
        .as_object()
        .ok_or_else(|| ValidationError("missing helper source versions".into()))?;
    require(
        versions.len() == current.sources.len(),
        "helper source membership mismatch",
    )?;
    let mut documents = BTreeMap::new();
    for (path, source) in &current.sources {
        require(
            versions.get(path).and_then(Value::as_u64) == Some(source.editor_revision),
            "stale editor source version",
        )?;
        documents.insert(
            path.clone(),
            SourceSnapshot {
                revision: current.compile_revision,
                text: source.text.clone(),
            },
        );
    }
    let bytes =
        serde_json::to_vec(&p["display_list"]).map_err(|e| ValidationError(e.to_string()))?;
    let paired = pipeline_frame::pair(result, Some(&bytes), true)?
        .ok_or_else(|| ValidationError("missing paired display".into()))?;
    require(
        paired.id == current.request_id,
        "helper display request mismatch",
    )?;
    let Message::DisplayList(ref list) = paired.message else {
        return Err(ValidationError("helper display type".into()));
    };
    require(
        list.project_id == current.project_id && list.revision == current.compile_revision,
        "helper display identity mismatch",
    )?;
    let display = PipelineCff::bind(&bytes, capabilities, &documents, resources)?;
    Ok(BoundHelperCandidate {
        current: current.clone(),
        display,
    })
}
