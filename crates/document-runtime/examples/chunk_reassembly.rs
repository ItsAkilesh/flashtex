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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("request JSON path required")?;
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
    Ok(())
}
