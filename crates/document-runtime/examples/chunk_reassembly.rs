//! Offline prototype only: full result stdin, matching single request JSON path argv1.
use flashtex_document_runtime::{
    experimental_chunks::{Assembly, Chunk},
    Request,
};
use serde_json::{json, Value};
use std::{
    io::{self, Read},
    time::{Duration, Instant},
};
#[derive(serde::Serialize)]
struct BorrowedPage<'a> {
    id: &'a str,
    project_id: &'a str,
    revision: u64,
    page: &'a Value,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("request JSON path required")?;
    let mode = std::env::args().nth(2).unwrap_or_else(|| "both".into());
    if !matches!(mode.as_str(), "both" | "strings" | "pages") {
        return Err("mode must be both, strings or pages".into());
    }
    let input: Value = serde_json::from_reader(std::fs::File::open(path)?)?;
    let p = &input["payload"];
    let request = Request {
        id: input["id"].as_str().ok_or("id")?.into(),
        project_id: p["project_id"].as_str().ok_or("project")?.into(),
        revision: p["revision"].as_u64().ok_or("revision")?,
        entry_path: p["entry_path"].as_str().ok_or("entry")?.into(),
        documents: serde_json::from_value(p["documents"].clone())?,
    };
    let mut bytes = Vec::new();
    io::stdin()
        .take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err("result too large".into());
    }
    let text = std::str::from_utf8(&bytes)?;
    let full: Value = serde_json::from_slice(&bytes)?;
    let pages = full["payload"]["pages"].as_array().ok_or("pages")?.len();
    let revision = request.revision;
    if mode != "pages" {
        let mut assembly = Assembly::new(
            request.clone(),
            vec![],
            bytes.len(),
            pages,
            1024 * 1024,
            Duration::from_secs(60),
        )?;
        let started = Instant::now();
        let mut start = 0;
        let mut sequence = 0;
        let mut wire_bytes = 0;
        let mut largest = 0;
        while start < text.len() {
            let mut end = (start + 64 * 1024).min(text.len());
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            let chunk = Chunk {
                id: request.id.clone(),
                project_id: request.project_id.clone(),
                revision,
                sequence,
                data: text[start..end].into(),
            };
            let frame = serde_json::to_vec(&chunk)?;
            wire_bytes += frame.len();
            largest = largest.max(frame.len());
            assembly.push(&frame, revision)?;
            start = end;
            sequence += 1;
        }
        let restored = assembly.finish(revision)?;
        let reassembly_ms = started.elapsed().as_secs_f64() * 1000.0;
        let equal = restored == full;
        println!(
            "{}",
            json!({"exact_json_equal":equal,"source_result_bytes":bytes.len(),"wire_bytes":wire_bytes,
        "chunks":sequence,"largest_chunk_bytes":largest,"pages":pages,"pack_reassemble_validate_ms":reassembly_ms,
        "production_transport":false,"native_paint_measured":false,"peak_memory_measured":false})
        );
        if !equal {
            return Err("reassembly mismatch".into());
        }
    }
    if mode != "strings" {
        // Build only the header metadata; avoid cloning all page items for it.
        let mut header = serde_json::Map::new();
        for (key, value) in full.as_object().ok_or("envelope")? {
            if key == "payload" {
                let mut payload = serde_json::Map::new();
                for (name, value) in value.as_object().ok_or("payload")? {
                    payload.insert(
                        name.clone(),
                        if name == "pages" {
                            json!([])
                        } else {
                            value.clone()
                        },
                    );
                }
                header.insert(key.clone(), Value::Object(payload));
            } else {
                header.insert(key.clone(), value.clone());
            }
        }
        let header = serde_json::to_vec(&Value::Object(header))?;
        let mut typed = flashtex_document_runtime::experimental_chunks::PageAssembly::new(
            request.clone(),
            vec![],
            &header,
            pages,
            1024 * 1024,
            Duration::from_secs(60),
        )?;
        let typed_start = Instant::now();
        let mut typed_wire = header.len();
        let mut typed_largest = header.len();
        for page in full["payload"]["pages"].as_array().ok_or("pages")? {
            let frame = serde_json::to_vec(&BorrowedPage {
                id: &request.id,
                project_id: &request.project_id,
                revision,
                page,
            })?;
            typed_wire += frame.len();
            typed_largest = typed_largest.max(frame.len());
            typed.push(&frame, revision)?;
        }
        let typed_result = typed.finish(revision)?;
        let typed_ms = typed_start.elapsed().as_secs_f64() * 1000.0;
        let typed_equal = typed_result == full;
        println!(
            "{}",
            json!({"mode":"typed_pages","pages":pages,"wire_bytes":typed_wire,
        "largest_chunk_bytes":typed_largest,"pack_reassemble_validate_ms":typed_ms,"exact_json_equal":typed_equal,
        "production_transport":false,"peak_memory_measured":false,"native_paint_measured":false})
        );
        if !typed_equal {
            return Err("typed page mismatch".into());
        }
    }
    Ok(())
}
