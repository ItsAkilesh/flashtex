//! Bounded single-request or JSONL session helper. Run IO off the UI thread.
use flashtex_assistant_context::{CompileBinding, Context, Location};
use flashtex_edit_ledger::Document;
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, Read, Write};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    operation: String,
    binding: CompileBinding,
    sources: Vec<Document>,
    compiler_result: Value,
    user_instruction: String,
    #[serde(default)]
    related_paths: Vec<String>,
    selected_diagnostics: Option<Vec<usize>>,
    destinations: Option<Vec<Location>>,
    response: Option<Value>,
    current_sources: Option<Vec<Document>>,
}
fn build_context(request: &Input) -> Result<Context, String> {
    let mut context = match &request.selected_diagnostics {
        Some(indices) => Context::build_selected(
            request.binding.clone(),
            &request.sources,
            &request.compiler_result,
            &request.user_instruction,
            &request.related_paths,
            indices,
        )?,
        None => Context::build(
            request.binding.clone(),
            &request.sources,
            &request.compiler_result,
            &request.user_instruction,
            &request.related_paths,
        )?,
    };
    if let Some(destinations) = &request.destinations {
        context = context.restrict_edits(destinations.clone(), &request.sources)?;
    }
    Ok(context)
}
fn run() -> Result<Value, String> {
    let mut bytes = Vec::new();
    io::stdin()
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("input exceeds16MiB".into());
    }
    let request: Input = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let context = build_context(&request)?;
    match request.operation.as_str() {
        "prepare" => Ok(json!({"type":"prepared_context","payload":context.payload()})),
        "validate" => {
            let response = serde_json::to_vec(&request.response.ok_or("response required")?)
                .map_err(|e| e.to_string())?;
            let proposal = context.validate_response(
                &response,
                &request.current_sources.ok_or("current_sources required")?,
            )?;
            Ok(json!({"type":"validated_proposal","payload":proposal,"applied":false}))
        }
        _ => Err("operation must be prepare or validate".into()),
    }
}
#[cfg(feature = "grok")]
mod provider_session;
mod session;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    #[cfg(feature = "grok")]
    if args.first().map(String::as_str) == Some("--provider-session") && args.len() == 3 {
        let result = std::env::var("FLASHTEX_GROK_API_KEY")
            .map_err(|_| "explicit provider credential missing".to_owned())
            .and_then(|key| session::run_provider(&args[1], &args[2], key));
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(2);
        }
        return;
    }
    if args.first().map(String::as_str) == Some("--session") && args.len() == 2 {
        if let Err(error) = session::run(&args[1]) {
            eprintln!("{error}");
            std::process::exit(2);
        }
        return;
    }
    if !args.is_empty() {
        eprintln!("usage: flashtex-assistant-context [--session FRESH_SESSION_ID]");
        std::process::exit(2);
    }
    let (value, failed) = match run() {
        Ok(value) => (value, false),
        Err(error) => (json!({"type":"error","message":error}), true),
    };
    let bytes = serde_json::to_vec(&value).unwrap();
    if bytes.len() > 128 * 1024 {
        eprintln!("output exceeds128KiB");
        std::process::exit(2);
    }
    let mut out = io::stdout().lock();
    if out
        .write_all(&bytes)
        .and_then(|_| out.write_all(b"\n"))
        .is_err()
    {
        std::process::exit(2);
    }
    drop(out);
    if failed {
        std::process::exit(1);
    }
}
