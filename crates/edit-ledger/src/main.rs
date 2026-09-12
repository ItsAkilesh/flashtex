//! Local JSON Lines adapter. Keep it on private pipes, never an anonymous port.
use flashtex_edit_ledger::recovery::RecoveryImport;
use flashtex_edit_ledger::retention::RetentionPolicy;
use flashtex_edit_ledger::{AppliedReceipt, Document, PreparedEdit, Store};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, BufRead, Read, Write};

#[derive(Deserialize)]
struct Request {
    id: String,
    #[serde(flatten)]
    operation: Operation,
}
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
enum Operation {
    Initialize {
        document: Document,
    },
    Apply {
        edit: PreparedEdit,
    },
    ReplaceDocument {
        expected_revision: u64,
        expected_sha256: String,
        text: String,
    },
    Confirm {
        receipt: AppliedReceipt,
    },
    Status,
    RecoveryExport,
    RecoveryImport {
        recovery: RecoveryImport,
    },
    Compact {
        policy: RetentionPolicy,
    },
}
fn execute(store: &mut Store, operation: Operation) -> flashtex_edit_ledger::Result<Value> {
    match operation {
        Operation::Initialize { document } => {
            store.initialize(document)?;
            Ok(json!({"document": store.document()?}))
        }
        Operation::Apply { edit } => {
            let receipt = store.apply(edit)?;
            Ok(json!({"receipt": receipt, "document": store.document()?}))
        }
        Operation::ReplaceDocument {
            expected_revision,
            expected_sha256,
            text,
        } => Ok(
            json!({"document": store.replace_document(expected_revision, &expected_sha256, text)?}),
        ),
        Operation::Confirm { receipt } => {
            store.confirm(&receipt)?;
            Ok(json!({"confirmed": receipt}))
        }
        Operation::Status => {
            Ok(json!({"document": store.document()?, "pending_receipts": store.recovery()?}))
        }
        Operation::RecoveryExport => Ok(json!(store.export_recovery()?)),
        Operation::RecoveryImport { recovery } => Ok(json!(store.import_recovery(recovery)?)),
        Operation::Compact { policy } => Ok(json!(store.compact(policy)?)),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--store") {
        return Err("usage: flashtex-edit-ledger --store PRIVATE_DIRECTORY".into());
    }
    let path = args.next().ok_or("missing store directory")?;
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let mut store = Store::open(path)?;
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    const MAX_LINE: u64 = 12 * 1024 * 1024;
    loop {
        let mut line = Vec::new();
        let n = input
            .by_ref()
            .take(MAX_LINE + 1)
            .read_until(b'\n', &mut line)?;
        if n == 0 {
            break;
        }
        if n as u64 > MAX_LINE || line.last() != Some(&b'\n') {
            writeln!(
                output,
                "{}",
                json!({"id": null, "error": {"code": "invalid_frame", "message": "expected newline-terminated line of at most 12 MiB"}})
            )?;
            output.flush()?;
            return Err("invalid input frame".into());
        }
        let response = match serde_json::from_slice::<Request>(&line) {
            Ok(request) if !request.id.is_empty() && request.id.len() <= 128 => {
                match execute(&mut store, request.operation) {
                    Ok(payload) => json!({"id": request.id, "payload": payload}),
                    Err(error) => json!({"id": request.id, "error": error}),
                }
            }
            Ok(_) => {
                json!({"id": null, "error": {"code": "invalid_id", "message": "request ID must be 1–128 bytes"}})
            }
            Err(error) => {
                json!({"id": null, "error": {"code": "invalid_request", "message": error.to_string()}})
            }
        };
        writeln!(output, "{response}")?;
        output.flush()?;
    }
    Ok(())
}
