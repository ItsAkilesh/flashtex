//! Bounded, validated experimental wire boundary. Unknown required primitives
//! have typed errors and cannot disappear into a partially rendered document.
use crate::*;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    TooLarge {
        limit: usize,
    },
    Malformed {
        message: String,
    },
    UnsupportedVersion {
        version: u8,
    },
    UnsupportedMessage {
        kind: String,
    },
    UnsupportedPrimitive {
        page_index: usize,
        item_index: usize,
        kind: String,
    },
    Semantic {
        message: String,
    },
}
impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for WireError {}
pub type WireResult<T> = std::result::Result<T, WireError>;
fn semantic(error: ValidationError) -> WireError {
    WireError::Semantic { message: error.0 }
}
fn malformed(error: impl std::fmt::Display) -> WireError {
    WireError::Malformed {
        message: error.to_string(),
    }
}

/// The returned envelope has passed structural and semantic checks against the
/// explicit offer. Resource bytes still require the separate loader/source gate.
pub fn parse_validated(bytes: &[u8], offer: Option<&Envelope>) -> WireResult<Envelope> {
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(WireError::TooLarge {
            limit: MAX_MESSAGE_BYTES,
        });
    }
    let raw: RawEnvelope = serde_json::from_slice(bytes).map_err(malformed)?;
    if raw.protocol_version != 2 {
        return Err(WireError::UnsupportedVersion {
            version: raw.protocol_version,
        });
    }
    if raw.kind == "display_list" {
        // Diagnostic-only inspection; typed decoding below still reads ORIGINAL
        // raw payload bytes, so this Value never erases validation evidence.
        let diagnostic: serde_json::Value =
            serde_json::from_str(raw.payload.get()).map_err(malformed)?;
        if let Some(pages) = diagnostic.get("pages").and_then(|p| p.as_array()) {
            for (page_index, page) in pages.iter().enumerate() {
                if let Some(items) = page.get("items").and_then(|i| i.as_array()) {
                    for (item_index, item) in items.iter().enumerate() {
                        if let Some(kind) = item.get("kind").and_then(|k| k.as_str()) {
                            if kind != "glyph_run" && kind != "rule" {
                                return Err(WireError::UnsupportedPrimitive {
                                    page_index,
                                    item_index,
                                    kind: kind.into(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    let message = match raw.kind.as_str() {
        "render_capabilities" => Message::Offer(decode(&raw.payload).map_err(malformed)?),
        "render_format_selected" => Message::Selected(decode(&raw.payload).map_err(malformed)?),
        "render_format_rejected" => Message::Rejected(decode(&raw.payload).map_err(malformed)?),
        "display_list" => Message::DisplayList(decode(&raw.payload).map_err(malformed)?),
        _ => return Err(WireError::UnsupportedMessage { kind: raw.kind }),
    };
    let envelope = Envelope {
        id: raw.id,
        message,
    };
    envelope.validate(offer).map_err(semantic)?;
    Ok(envelope)
}
#[derive(Serialize)]
struct Outgoing<'a, T: Serialize> {
    protocol_version: u8,
    id: &'a str,
    #[serde(rename = "type")]
    kind: &'a str,
    payload: &'a T,
}
struct Output {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("wire output capacity reached"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn write_payload<T: Serialize>(
    id: &str,
    kind: &str,
    payload: &T,
    limit: usize,
) -> WireResult<Vec<u8>> {
    let mut output = Output {
        bytes: vec![],
        limit,
    };
    match serde_json::to_writer(
        &mut output,
        &Outgoing {
            protocol_version: 2,
            id,
            kind,
            payload,
        },
    ) {
        Ok(()) => Ok(output.bytes),
        Err(error) if error.is_io() => Err(WireError::TooLarge { limit }),
        Err(error) => Err(malformed(error)),
    }
}
/// Return a whole bounded JSON object only after validation. A capacity failure
/// never gives callers a partially serialized frame to publish.
pub fn serialize_validated(
    envelope: &Envelope,
    offer: Option<&Envelope>,
    maximum_bytes: usize,
) -> WireResult<Vec<u8>> {
    let limit = maximum_bytes.min(MAX_MESSAGE_BYTES);
    envelope.validate(offer).map_err(semantic)?;
    match &envelope.message {
        Message::Offer(payload) => {
            write_payload(&envelope.id, "render_capabilities", payload, limit)
        }
        Message::Selected(payload) => {
            write_payload(&envelope.id, "render_format_selected", payload, limit)
        }
        Message::Rejected(payload) => {
            write_payload(&envelope.id, "render_format_rejected", payload, limit)
        }
        Message::DisplayList(payload) => {
            write_payload(&envelope.id, "display_list", payload, limit)
        }
    }
}
