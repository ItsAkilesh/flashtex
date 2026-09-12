use super::{build_context, Input};
use flashtex_assistant_context::ExplanationRegistry;
use flashtex_edit_ledger::Document;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    io::{self, BufRead, Write},
    time::Duration,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    id: String,
    action: Action,
}
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Action {
    Submit {
        input: Box<Input>,
        timeout_ms: u64,
    },
    Receive {
        request_id: String,
        context_id: String,
        response: Value,
        current_sources: Vec<Document>,
    },
    Cancel {
        request_id: String,
    },
    RevokeStale {
        project_id: String,
        current_sources: Vec<Document>,
    },
    Status {
        request_id: String,
    },
    Sweep {},
}
fn execute(registry: &mut ExplanationRegistry, action: Action) -> Result<Value, String> {
    match action {
        Action::Submit { input, timeout_ms } => {
            if input.operation != "prepare"
                || input.response.is_some()
                || input.current_sources.is_some()
            {
                return Err(
                    "submit input must be a prepare request without response/current_sources"
                        .into(),
                );
            }
            let context = build_context(&input)?;
            let payload = serde_json::to_value(context.payload()).map_err(|e| e.to_string())?;
            let request_id =
                registry.submit(context, &input.sources, Duration::from_millis(timeout_ms))?;
            Ok(json!({"type":"submitted","request_id":request_id,"payload":payload}))
        }
        Action::Receive {
            request_id,
            context_id,
            response,
            current_sources,
        } => {
            let bytes = serde_json::to_vec(&response).map_err(|e| e.to_string())?;
            let proposal = registry.receive(&request_id, &context_id, &bytes, &current_sources)?;
            Ok(
                json!({"type":"validated_proposal","request_id":request_id,"payload":proposal,"applied":false}),
            )
        }
        Action::Cancel { request_id } => Ok(
            json!({"type":"cancelled","request_id":request_id,"changed":registry.cancel(&request_id)}),
        ),
        Action::RevokeStale {
            project_id,
            current_sources,
        } => Ok(
            json!({"type":"revoked","request_ids":registry.revoke_stale(&project_id, &current_sources)}),
        ),
        Action::Status { request_id } => Ok(
            json!({"type":"status","request_id":request_id,"state":registry.state(&request_id).map(|s| format!("{s:?}"))}),
        ),
        Action::Sweep {} => Ok(json!({"type":"expired","request_ids":registry.sweep()})),
    }
}
pub fn run(session: &str) -> Result<(), String> {
    let mut registry = ExplanationRegistry::new(session.to_owned(), 8, 64)?;
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut frame = Vec::new();
        let read = std::io::Read::take(&mut input, 16 * 1024 * 1024 + 1)
            .read_until(b'\n', &mut frame)
            .map_err(|e| e.to_string())?;
        if read == 0 {
            return Ok(());
        }
        if read > 16 * 1024 * 1024 {
            return Err("session frame exceeds16MiB".into());
        }
        let parsed = serde_json::from_slice::<Command>(&frame).map_err(|e| e.to_string());
        let (id, result) = match parsed {
            Ok(command) if !command.id.is_empty() && command.id.len() <= 128 => {
                (Some(command.id), execute(&mut registry, command.action))
            }
            Ok(_) => (None, Err("command id must contain1..128bytes".into())),
            Err(error) => (None, Err(error)),
        };
        let value = match result {
            Ok(value) => json!({"id":id,"result":value}),
            Err(error) => json!({"id":id,"error":error}),
        };
        let bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
        if bytes.len() > 128 * 1024 {
            return Err("session output exceeds128KiB".into());
        }
        output
            .write_all(&bytes)
            .and_then(|_| output.write_all(b"\n"))
            .and_then(|_| output.flush())
            .map_err(|e| e.to_string())?;
    }
}
