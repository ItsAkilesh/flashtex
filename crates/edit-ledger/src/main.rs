//! Private-pipe host for the serialized background service.
use flashtex_edit_ledger::service::{BackgroundService, ServiceOptions, MAX_FRAME_BYTES};
use serde_json::json;
use std::io::{self, BufRead, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--store") {
        return Err("usage: flashtex-edit-ledger --store PRIVATE_DIRECTORY".into());
    }
    let path = args.next().ok_or("missing store directory")?;
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let service = BackgroundService::start(path.into(), ServiceOptions::default())?;
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut line = Vec::new();
        let n = input
            .by_ref()
            .take(MAX_FRAME_BYTES as u64 + 1)
            .read_until(b'\n', &mut line)?;
        if n == 0 {
            break;
        }
        let pending = match service.try_submit(line) {
            Ok(pending) => pending,
            Err(error) => {
                writeln!(output, "{}", json!({"id":null,"error":error}))?;
                output.flush()?;
                return Err("invalid input frame".into());
            }
        };
        // Blocking is confined to this CLI host. Native library adapters use
        // try_submit + try_recv and never wait on MainActor.
        let reply = pending.wait()?;
        writeln!(output, "{}", serde_json::to_string(&reply)?)?;
        output.flush()?;
    }
    Ok(())
}
