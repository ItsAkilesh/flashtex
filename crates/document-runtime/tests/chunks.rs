use flashtex_document_runtime::{
    experimental_chunks::{Assembly, Chunk},
    Request,
};
use serde_json::{json, Value};
use std::time::Duration;
fn request() -> Request {
    Request {
        id: "r".into(),
        project_id: "p".into(),
        revision: 1,
        entry_path: "main.tex".into(),
        documents: vec![flashtex_document_runtime::Document {
            path: "main.tex".into(),
            text: "α".into(),
        }],
    }
}
fn result() -> Value {
    json!({"protocol_version":1,"type":"compile_result","id":"r","payload":{"project_id":"p","revision":1,"status":"ok","pages":[],"diagnostics":[]}})
}
fn frame(sequence: usize, data: &str) -> Vec<u8> {
    serde_json::to_vec(&Chunk {
        id: "r".into(),
        project_id: "p".into(),
        revision: 1,
        sequence,
        data: data.into(),
    })
    .unwrap()
}
fn assembly(bytes: usize, pages: usize) -> Assembly {
    Assembly::new(
        request(),
        vec![],
        bytes,
        pages,
        1024,
        Duration::from_secs(5),
    )
    .unwrap()
}
#[test]
fn exact_reassembly_and_atomic_failure() {
    let value = result();
    let text = value.to_string();
    let mut a = assembly(text.len(), 0);
    a.push(&frame(0, &text[..30]), 1).unwrap();
    a.push(&frame(1, &text[30..]), 1).unwrap();
    assert_eq!(a.finish(1).unwrap(), value);
    let mut incomplete = assembly(text.len(), 0);
    incomplete.push(&frame(0, &text[..30]), 1).unwrap();
    assert!(incomplete.finish(1).is_err());
    let mut wrong_count = assembly(text.len(), 1);
    wrong_count.push(&frame(0, &text), 1).unwrap();
    assert!(wrong_count.finish(1).is_err());
}
#[test]
fn stale_duplicate_oversize_and_cancel_are_terminal() {
    let text = result().to_string();
    let mut stale = assembly(text.len(), 0);
    assert!(stale.push(&frame(0, &text), 2).is_err());
    assert!(stale.push(&frame(0, &text), 1).is_err());
    let mut duplicate = assembly(text.len(), 0);
    duplicate.push(&frame(0, &text[..30]), 1).unwrap();
    assert!(duplicate.push(&frame(0, &text[30..]), 1).is_err());
    assert!(duplicate.finish(1).is_err());
    let mut oversized = assembly(text.len(), 0);
    assert!(oversized.push(&vec![b'x'; 1025], 1).is_err());
    let mut cancelled = assembly(text.len(), 0);
    cancelled.cancel();
    assert!(cancelled.push(&frame(0, &text), 1).is_err());
}

#[test]
fn typed_pages_preserve_all_fields_and_reject_incomplete_or_invalid_geometry() {
    use flashtex_document_runtime::experimental_chunks::{PageAssembly, PageChunk};
    let header = result();
    let bytes = serde_json::to_vec(&header).unwrap();
    let page = json!({"number":1,"width_pt":612,"height_pt":792,"items":[],"extension":{"α":true}});
    let chunk = serde_json::to_vec(&PageChunk {
        id: "r".into(),
        project_id: "p".into(),
        revision: 1,
        page: page.clone(),
    })
    .unwrap();
    let mut a =
        PageAssembly::new(request(), vec![], &bytes, 1, 1024, Duration::from_secs(5)).unwrap();
    a.push(&chunk, 1).unwrap();
    let mut expected = header.clone();
    expected["payload"]["pages"] = json!([page]);
    assert_eq!(a.finish(1).unwrap(), expected);
    let incomplete =
        PageAssembly::new(request(), vec![], &bytes, 1, 1024, Duration::from_secs(5)).unwrap();
    assert!(incomplete.finish(1).is_err());
    let mut invalid =
        PageAssembly::new(request(), vec![], &bytes, 1, 1024, Duration::from_secs(5)).unwrap();
    let frame = serde_json::to_vec(&PageChunk {
        id: "r".into(),
        project_id: "p".into(),
        revision: 1,
        page: json!({"number":1,"width_pt":-1,"height_pt":792,"items":[]}),
    })
    .unwrap();
    invalid.push(&frame, 1).unwrap();
    assert!(invalid.finish(1).is_err());
    let mut stale =
        PageAssembly::new(request(), vec![], &bytes, 1, 1024, Duration::from_secs(5)).unwrap();
    assert!(stale.push(&chunk, 2).is_err());
    assert!(stale.push(&chunk, 1).is_err());
}
