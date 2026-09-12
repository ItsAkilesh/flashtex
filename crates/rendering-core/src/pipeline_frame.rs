//! Bounded pairing of the existing runtime-v1 result and optional v2 sibling.
//! Transport framing only: successful pairing never verifies resource bytes or
//! grants paint readiness. CFF/resource validation remains a separate opt-in step.
use crate::*;
use serde_json::Value;
pub const MAX_REPLY_LINE: usize = 16 * 1024 * 1024;

pub fn pair(
    result: &[u8],
    sibling: Option<&[u8]>,
    require_tex_metrics: bool,
) -> Result<Option<Envelope>> {
    require(
        result.len() <= MAX_REPLY_LINE && sibling.is_none_or(|s| s.len() <= MAX_REPLY_LINE),
        "reply line exceeds transport limit",
    )?;
    let value: Value =
        serde_json::from_slice(result).map_err(|e| ValidationError(format!("result JSON: {e}")))?;
    require(
        value["protocol_version"] == 1 && value["type"] == "compile_result",
        "runtime-v1 compile_result required",
    )?;
    let request_id = value["id"]
        .as_str()
        .ok_or_else(|| ValidationError("missing result ID".into()))?;
    id(request_id)?;
    let p = &value["payload"];
    let status = p["status"]
        .as_str()
        .ok_or_else(|| ValidationError("missing result status".into()))?;
    require(
        matches!(status, "ok" | "recovered" | "failed"),
        "unknown result status",
    )?;
    let caps = match p.get("layout_capabilities") {
        None => Vec::new(),
        Some(Value::Array(v)) if v.len() <= 16 => {
            let mut seen = BTreeSet::new();
            for c in v {
                let c = c
                    .as_str()
                    .ok_or_else(|| ValidationError("invalid accepted capability".into()))?;
                require(
                    !c.is_empty() && c.len() <= 64 && seen.insert(c),
                    "invalid or duplicate accepted capability",
                )?;
            }
            seen.into_iter().collect()
        }
        _ => return Err(ValidationError("invalid accepted capabilities".into())),
    };
    if require_tex_metrics {
        let diagnostics = p["diagnostics"]
            .as_array()
            .ok_or_else(|| ValidationError("missing diagnostics".into()))?;
        require(
            !diagnostics.iter().any(|d| {
                d["code"] == "tfm_missing"
                    || d["code"] == "required_metrics_unavailable"
                    || d["severity"] == "error"
            }),
            "reference metrics unavailable",
        )?;
    }
    let accepted = caps.contains(&"display-list-v2");
    require(
        !(status == "failed" && accepted),
        "failed result cannot accept display sibling",
    )?;
    require(
        accepted == sibling.is_some(),
        "display sibling acceptance mismatch",
    )?;
    let Some(bytes) = sibling else {
        return Ok(None);
    };
    let envelope = parse(bytes)?;
    require(
        envelope.id == request_id,
        "display sibling request mismatch",
    )?;
    let Message::DisplayList(list) = &envelope.message else {
        return Err(ValidationError("display_list sibling required".into()));
    };
    require(
        p["project_id"].as_str() == Some(list.project_id.as_str())
            && p["revision"].as_u64() == Some(list.revision),
        "display sibling revision/project mismatch",
    )?;
    Ok(Some(envelope))
}
