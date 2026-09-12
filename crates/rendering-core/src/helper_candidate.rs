//! Opt-in helper candidate binding. The caller supplies authoritative current
//! controller state; untrusted event fields never create freshness authority.
//! Editor document versions are distinct from the compile generation in v2.
//! Successful export grants no native paint or source-navigation authority.
use crate::{
    pipeline_cff::{PipelineCff, SearchablePdf},
    *,
};
use flashtex_font_resources::registry::CffFontResource;
use serde::{Deserialize, Deserializer};
use std::sync::Arc;
mod syntax;

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
    // Preserve whole-event finite/depth/Unicode validation without a Value tree,
    // including ignored extensions. Typed decoding still reads ORIGINAL bytes.
    let _: syntax::Syntax =
        serde_json::from_slice(event).map_err(|e| ValidationError(format!("helper JSON: {e}")))?;
    let value: RawHelperEvent =
        serde_json::from_slice(event).map_err(|e| ValidationError(format!("helper JSON: {e}")))?;
    let p = &value.payload;
    require(
        value.protocol_version == 1
            && value.kind == "update"
            && value.session_id == current.session_id
            && p.kind == "display_candidate"
            && p.untrusted
            && !p.source_actions_enabled,
        "helper envelope/policy mismatch",
    )?;
    require(
        p.project_id == current.project_id
            && p.request_id == current.request_id
            && p.compile_revision == current.compile_revision
            && p.membership_generation == current.membership_generation,
        "stale helper identity",
    )?;
    let versions = &p.source_versions;
    require(
        versions.len() == current.sources.len(),
        "helper source membership mismatch",
    )?;
    let mut documents = BTreeMap::new();
    for (path, source) in &current.sources {
        require(
            versions.get(path).copied() == Some(source.editor_revision),
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
    let bytes = p.display_list.get().as_bytes();
    let paired = pipeline_frame::pair(result, Some(bytes), true)?
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
    let display = PipelineCff::bind(bytes, capabilities, &documents, resources)?;
    Ok(BoundHelperCandidate {
        current: current.clone(),
        display,
    })
}

// Existing serde decoding keeps opaque display bytes until typed rendering
// validation; this adds no JSON syntax parser.
#[derive(Deserialize)]
struct RawHelperEvent {
    protocol_version: u8,
    #[serde(rename = "type")]
    kind: String,
    session_id: String,
    payload: RawHelperPayload,
}
#[derive(Deserialize)]
struct RawHelperPayload {
    kind: String,
    untrusted: bool,
    source_actions_enabled: bool,
    project_id: String,
    request_id: String,
    compile_revision: u64,
    membership_generation: u64,
    #[serde(deserialize_with = "unique_versions")]
    source_versions: BTreeMap<String, u64>,
    display_list: Box<serde_json::value::RawValue>,
}
fn unique_versions<'de, D: Deserializer<'de>>(
    decoder: D,
) -> std::result::Result<BTreeMap<String, u64>, D::Error> {
    struct Unique;
    impl<'de> serde::de::Visitor<'de> for Unique {
        type Value = BTreeMap<String, u64>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("unique source-version map")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, u64>()? {
                if out.len() >= 256 || out.insert(key, value).is_some() {
                    return Err(serde::de::Error::custom("duplicate/excess source version"));
                }
            }
            Ok(out)
        }
    }
    decoder.deserialize_map(Unique)
}
