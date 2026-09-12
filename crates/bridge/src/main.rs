use flashtex_bridge::{
    grok::{GrokClient, DEFAULT_MODEL},
    store::Store,
    *,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, BufRead, Read, Write};

const MAX_FRAME: usize = 12 * 1024 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    protocol_version: u8,
    id: String,
    #[serde(rename = "type")]
    kind: String,
    payload: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pin {
    destination_id: String,
    project_id: String,
    path: String,
    revision: u64,
    start_byte: usize,
    end_byte: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Edit {
    project_id: String,
    path: String,
    base_revision: u64,
    revision: u64,
    start_byte: usize,
    end_byte: usize,
    replacement: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Convert {
    capture_id: String,
    #[serde(default)]
    supported_features: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Prepare {
    capture_id: String,
    expected_revision: u64,
    approved: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Confirm {
    capture_id: String,
    edit_id: String,
    new_revision: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureId {
    capture_id: String,
}
fn decode<T: serde::de::DeserializeOwned>(payload: Value) -> Result<T> {
    Ok(serde_json::from_value(payload)?)
}
fn dispatch(
    bridge: &mut Bridge,
    message: Envelope,
    enable_grok: bool,
) -> Result<(&'static str, Value)> {
    if message.protocol_version != 1 {
        return Err(BridgeError::new(
            "unsupported_version",
            "Only protocol_version 1 is supported",
        ));
    }
    match message.kind.as_str() {
        "document_open" => {
            let doc: Document = decode(message.payload)?;
            bridge.open_document(doc)?;
            Ok(("document_opened", json!({})))
        }
        "document_edit" => {
            let edit: Edit = decode(message.payload)?;
            bridge.edit(
                &edit.project_id,
                &edit.path,
                edit.base_revision,
                edit.revision,
                edit.start_byte,
                edit.end_byte,
                &edit.replacement,
            )?;
            Ok(("document_updated", json!({"revision":edit.revision})))
        }
        "destination_pin" => {
            let pin: Pin = decode(message.payload)?;
            let a = bridge.pin(
                &pin.destination_id,
                &pin.project_id,
                &pin.path,
                pin.revision,
                pin.start_byte,
                pin.end_byte,
            )?;
            Ok(("destination_pinned", serde_json::to_value(a)?))
        }
        "capture_submit" => {
            let capture: CaptureSubmit = decode(message.payload)?;
            let record = bridge.receive(capture)?;
            Ok((
                "capture_received",
                json!({"capture_id":record.capture.capture_id,"durable":true,"has_proposal":record.proposal.is_some(),"applied":record.applied.is_some()}),
            ))
        }
        "capture_convert" => {
            let request: Convert = decode(message.payload)?;
            if !enable_grok {
                return Err(BridgeError::new(
                    "provider_disabled",
                    "Grok conversion requires explicitly enabled provider configuration",
                ));
            }
            let key = std::env::var("XAI_API_KEY").map_err(|_| {
                BridgeError::new(
                    "provider_auth_missing",
                    "Supply the Mac's authorized Grok key through its credential adapter",
                )
            })?;
            let model =
                std::env::var("FLASHTEX_GROK_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.into());
            let record = bridge.convert(
                &request.capture_id,
                request.supported_features,
                &GrokClient::new(key, model)?,
            )?;
            let proposal = record.proposal.unwrap();
            Ok((
                "capture_proposal",
                json!({"capture_id":request.capture_id,"latex":proposal.latex,"ambiguities":proposal.ambiguities,"required_dependencies":proposal.required_dependencies,"context_revision":record.context.map(|c|c.revision)}),
            ))
        }
        "capture_prepare_insert" => {
            let request: Prepare = decode(message.payload)?;
            let edit = bridge.prepare_insert(
                &request.capture_id,
                request.expected_revision,
                request.approved,
            )?;
            Ok(("capture_edit", serde_json::to_value(edit)?))
        }
        "capture_applied" => {
            let request: Confirm = decode(message.payload)?;
            let receipt = bridge.confirm_insert(
                &request.capture_id,
                &request.edit_id,
                request.new_revision,
            )?;
            Ok((
                "capture_application_received",
                json!({"capture_id":request.capture_id,"edit_id":receipt.edit_id,"new_revision":receipt.new_revision}),
            ))
        }
        "capture_status" => {
            let request: CaptureId = decode(message.payload)?;
            let record = bridge.store.require(&request.capture_id)?;
            Ok((
                "capture_status",
                json!({"capture_id":request.capture_id,"proposal":record.proposal,"prepared":record.prepared,"applied":record.applied,"rejected":record.rejected}),
            ))
        }
        "capture_reject" => {
            let request: CaptureId = decode(message.payload)?;
            bridge.reject(&request.capture_id)?;
            Ok(("capture_rejected", json!({"capture_id":request.capture_id})))
        }
        _ => Err(BridgeError::new(
            "unsupported_type",
            "Unknown bridge request type",
        )),
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut store = None;
    let mut enable_grok = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--store" => store = args.next(),
            "--enable-grok" => enable_grok = true,
            "--help" => {
                println!("flashtex-bridge --store PRIVATE_APP_DATA_DIRECTORY [--enable-grok]\nReads runtime-v1 JSONLines from stdin; logs to stderr. No network calls without capture_convert and --enable-grok.");
                return Ok(());
            }
            _ => {
                return Err(BridgeError::new(
                    "invalid_arguments",
                    "Use --store DIRECTORY and optional --enable-grok",
                ))
            }
        }
    }
    let mut bridge = Bridge::new(Store::open(store.ok_or_else(|| {
        BridgeError::new(
            "invalid_arguments",
            "A private capture journal directory is required",
        )
    })?)?);
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    loop {
        let mut frame = Vec::new();
        let count = reader
            .by_ref()
            .take((MAX_FRAME + 1) as u64)
            .read_until(b'\n', &mut frame)?;
        if count == 0 {
            break;
        }
        let mut id = Value::Null;
        let result = if frame.len() > MAX_FRAME {
            if frame.last() != Some(&b'\n') {
                loop {
                    let buf = reader.fill_buf()?;
                    if buf.is_empty() {
                        break;
                    }
                    let n = buf
                        .iter()
                        .position(|b| *b == b'\n')
                        .map(|p| p + 1)
                        .unwrap_or(buf.len());
                    let ended = buf[n - 1] == b'\n';
                    reader.consume(n);
                    if ended {
                        break;
                    }
                }
            }
            Err(BridgeError::new(
                "message_too_large",
                "Bridge message exceeded 12 MiB; record discarded",
            ))
        } else {
            let value: std::result::Result<Value, _> = serde_json::from_slice(&frame);
            match value {
                Ok(value) => {
                    id = value
                        .get("id")
                        .filter(|v| v.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 128))
                        .cloned()
                        .unwrap_or(Value::Null);
                    if id.is_null() {
                        Err(BridgeError::new(
                            "invalid_id",
                            "Every request needs a bounded nonempty string ID",
                        ))
                    } else {
                        decode(value).and_then(|message: Envelope| {
                            let _ = &message.id;
                            dispatch(&mut bridge, message, enable_grok)
                        })
                    }
                }
                Err(_) => Err(BridgeError::new(
                    "invalid_json",
                    "Malformed UTF-8 JSON request",
                )),
            }
        };
        let reply = match result {
            Ok((kind, payload)) => {
                json!({"protocol_version":1,"id":id,"type":kind,"payload":payload})
            }
            Err(error) => json!({"protocol_version":1,"id":id,"type":"error","payload":error}),
        };
        serde_json::to_writer(&mut writer, &reply)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }
    Ok(())
}
