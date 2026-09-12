//! Isolated response encoding; ledger/index/compile time is deliberately excluded.
#[path = "../src/output_buffer.rs"]
mod output_buffer;
use flashtex_edit_ledger::Document;
use serde_json::json;
use std::time::Instant;

fn main() {
    let document = Document::new("p".into(), "main.tex".into(), 2, "α".repeat(250_000)).unwrap();
    let full = json!({"protocol_version":1,"session_id":"benchmark","id":"save","type":"result",
        "payload":{"document":document,"preview_error":null,"save_and_submit_ms":0.0}});
    let mut metadata = full.clone();
    metadata["payload"]["document"]
        .as_object_mut()
        .unwrap()
        .remove("text");
    metadata["payload"]["document"]["byte_length"] = json!(500_000);
    metadata["payload"]["response_mode"] = json!("metadata");
    let mut observations = Vec::new();
    let mut sizes = [0, 0];
    for pair in 0..10 {
        for index in if pair % 2 == 0 { [0, 1] } else { [1, 0] } {
            let value = [&full, &metadata][index];
            let start = Instant::now();
            let bytes = output_buffer::serialize(value, 16 * 1024 * 1024).unwrap();
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
                *value
            );
            sizes[index] = bytes.len();
            observations.push(json!({"pair":pair,"metadata_only":index==1,"encoding_ms":ms}));
        }
    }
    println!(
        "{}",
        json!({"source_bytes":500_000,"source_sha256":document.source_sha256,
        "full_frame_bytes":sizes[0],"metadata_frame_bytes":sizes[1],"observations":observations,
        "scope":"synthetic wire encoding with same500KB Unicode source; no ledger/native timing"})
    );
}
