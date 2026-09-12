//! Transport correlation only. No font, geometry, feature or rendering validation.
use crate::Request;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBinding {
    pub path: String,
    pub revision: u64,
    pub sha256: String,
    pub byte_length: usize,
}
#[derive(Debug)]
pub struct UntrustedDisplayCandidate {
    request_id: String,
    project_id: String,
    revision: u64,
    sources: Vec<SourceBinding>,
    envelope: Value,
}
impl UntrustedDisplayCandidate {
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
    pub fn project_id(&self) -> &str {
        &self.project_id
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn sources(&self) -> &[SourceBinding] {
        &self.sources
    }
    /// Read-only untrusted transport data, not validated rendering data.
    pub fn envelope(&self) -> &Value {
        &self.envelope
    }
    /// Move without cloning. Downstream rendering/source validation is mandatory.
    pub fn into_envelope(self) -> Value {
        self.envelope
    }
}
pub(crate) fn validate(
    envelope: Value,
    request: &Request,
) -> Result<UntrustedDisplayCandidate, String> {
    let p = &envelope["payload"];
    if envelope["protocol_version"] != 2
        || envelope["type"] != "display_list"
        || envelope["id"] != request.id
        || p["project_id"] != request.project_id
        || p["revision"].as_u64() != Some(request.revision)
        || p["render_format"] != "display-list-v2"
    {
        return Err("display sibling correlation mismatch".into());
    }
    let documents = p["documents"]
        .as_array()
        .ok_or("display documents missing")?;
    if documents.len() != request.documents.len() {
        return Err("display document set mismatch".into());
    }
    let source_map: std::collections::BTreeMap<_, _> = request
        .documents
        .iter()
        .map(|d| (d.path.as_str(), d.text.as_str()))
        .collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut sources = Vec::with_capacity(documents.len());
    for doc in documents {
        let path = doc["path"]
            .as_str()
            .ok_or("display document path missing")?;
        if !seen.insert(path) {
            return Err("duplicate display document".into());
        }
        let source = source_map.get(path).ok_or("unknown display document")?;
        let hash = flashtex_project_files::sha256_hex(source.as_bytes());
        if doc["revision"].as_u64() != Some(request.revision)
            || doc["byte_length"].as_u64() != Some(source.len() as u64)
            || doc["sha256"].as_str() != Some(hash.as_str())
        {
            return Err("display source identity mismatch".into());
        }
        sources.push(SourceBinding {
            path: path.into(),
            revision: request.revision,
            sha256: hash,
            byte_length: source.len(),
        });
    }
    Ok(UntrustedDisplayCandidate {
        request_id: request.id.clone(),
        project_id: request.project_id.clone(),
        revision: request.revision,
        sources,
        envelope,
    })
}
