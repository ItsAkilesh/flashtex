//! Local review metadata process. No providers, source writes or edit approvals.
use flashtex_conversion_jobs::bridge_adapter::{
    native::{ContextIdentity, StatusSnapshot},
    review::*,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
const MAX_INPUT: usize = 512 * 1024;
const MAX_OUTPUT: usize = 1024 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    protocol_version: u8,
    id: String,
    now: u64,
    #[serde(flatten)]
    command: Command,
}
#[derive(Deserialize)]
#[serde(
    tag = "type",
    content = "payload",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum Command {
    Snapshot,
    Select {
        capture_id: Option<String>,
    },
    AdmitReady {
        status: StatusSnapshot,
    },
    UpdateContext {
        capture_id: String,
        context: ContextIdentity,
    },
    Decide {
        decision: DecisionRequest,
    },
    ValidateHandoff {
        handoff: ReviewIntentHandoff,
        context: ContextIdentity,
    },
    SetExpiry {
        capture_id: String,
        expires_at: u64,
    },
    Expire,
    Cancel {
        capture_id: String,
    },
    Retire {
        capture_id: String,
    },
}
fn error_code(error: InboxError) -> &'static str {
    match error {
        InboxError::Io(_) | InboxError::RecoveryRequired => "recovery_required",
        InboxError::Invalid => "invalid",
        InboxError::Busy => "busy",
        InboxError::Capacity => "capacity",
        InboxError::Missing => "missing",
        InboxError::Conflict => "conflict",
        InboxError::SelectionRequired => "selection_required",
        InboxError::StaleContext => "stale_context",
        InboxError::Cancelled => "cancelled",
        InboxError::Retired => "retired",
        InboxError::Expired => "expired",
    }
}
fn execute(inbox: &mut ReviewInbox, request: Request) -> Result<Value, InboxError> {
    inbox.expire_due(request.now)?;
    match request.command {
        Command::Snapshot => {
            serde_json::to_value(&*inbox.view().snapshot()?).map_err(|_| InboxError::Invalid)
        }
        Command::Select { capture_id } => {
            inbox.select(capture_id.as_deref())?;
            Ok(json!({"selected_capture":capture_id}))
        }
        Command::AdmitReady { status } => {
            inbox.admit_ready(&status)?;
            Ok(json!({"capture_id":status.capture_id}))
        }
        Command::UpdateContext {
            capture_id,
            context,
        } => {
            inbox.update_context(&capture_id, context)?;
            Ok(json!({"capture_id":capture_id}))
        }
        Command::Decide { decision } => {
            serde_json::to_value(inbox.decide_at(decision, request.now)?)
                .map_err(|_| InboxError::Invalid)
        }
        Command::ValidateHandoff { handoff, context } => {
            inbox.validate_handoff_at(&handoff, &context, request.now)?;
            Ok(json!({"valid_for_preparation_only":true}))
        }
        Command::SetExpiry {
            capture_id,
            expires_at,
        } => {
            inbox.set_expiry(&capture_id, expires_at)?;
            inbox.expire_due(request.now)?;
            Ok(json!({"capture_id":capture_id}))
        }
        Command::Expire => Ok(json!({"generation":inbox.view().snapshot()?.generation})),
        Command::Cancel { capture_id } => {
            inbox.cancel(&capture_id)?;
            Ok(json!({"capture_id":capture_id}))
        }
        Command::Retire { capture_id } => {
            inbox.retire(&capture_id)?;
            Ok(json!({"capture_id":capture_id}))
        }
    }
}
// Drain oversized lines without retaining them. EOF cannot execute a partial frame.
fn frame(reader: &mut impl BufRead) -> io::Result<Option<Result<Vec<u8>, &'static str>>> {
    let mut bytes = Vec::new();
    let mut oversized = false;
    loop {
        let chunk = reader.fill_buf()?;
        if chunk.is_empty() {
            return Ok(if bytes.is_empty() && !oversized {
                None
            } else {
                Some(Err("incomplete_frame"))
            });
        }
        let newline = chunk.iter().position(|b| *b == b'\n');
        let count = newline.map_or(chunk.len(), |n| n + 1);
        if !oversized && bytes.len().saturating_add(count) <= MAX_INPUT {
            bytes.extend_from_slice(&chunk[..count]);
        } else {
            oversized = true;
            bytes.clear();
        }
        reader.consume(count);
        if newline.is_some() {
            return Ok(Some(if oversized {
                Err("frame_too_large")
            } else {
                Ok(bytes)
            }));
        }
    }
}
fn response(id: Value, result: Result<Value, &str>) -> Value {
    match result {
        Ok(payload) => {
            json!({"protocol_version":1,"id":id,"type":"review_result","payload":payload})
        }
        Err(code) => {
            json!({"protocol_version":1,"id":id,"type":"review_error","payload":{"code":code}})
        }
    }
}
struct Bounded(Vec<u8>);
impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) > MAX_OUTPUT {
            return Err(io::Error::other("output limit"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let root = args
        .next()
        .ok_or("usage: flashtex-review-inbox PRIVATE_INBOX_DIRECTORY")?;
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let mut inbox = ReviewInbox::open(
        root,
        InboxLimits {
            bytes: MAX_OUTPUT / 2,
            ..InboxLimits::default()
        },
    )
    .map_err(|e| error_code(e).to_owned())?;
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    while let Some(line) = frame(&mut input).map_err(|e| e.to_string())? {
        let reply = match line {
            Err(code) => response(Value::Null, Err(code)),
            Ok(bytes) => match serde_json::from_slice::<Request>(&bytes) {
                Ok(request)
                    if request.protocol_version == 1
                        && flashtex_bridge::identifier(&request.id).is_ok() =>
                {
                    let id = Value::String(request.id.clone());
                    response(id, execute(&mut inbox, request).map_err(error_code))
                }
                _ => response(Value::Null, Err("invalid_request")),
            },
        };
        let mut bounded = Bounded(Vec::new());
        if serde_json::to_writer(&mut bounded, &reply).is_err() {
            bounded.0 =
                serde_json::to_vec(&response(reply["id"].clone(), Err("response_too_large")))
                    .unwrap();
        }
        output
            .write_all(&bounded.0)
            .and_then(|_| output.write_all(b"\n"))
            .and_then(|_| output.flush())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("review helper: {error}");
        std::process::exit(1);
    }
}
