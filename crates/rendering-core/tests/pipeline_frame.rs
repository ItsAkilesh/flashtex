use flashtex_rendering_core::pipeline_frame::pair;
use serde_json::Value;
fn lines(bytes: &[u8]) -> Vec<&[u8]> {
    bytes
        .split(|b| *b == b'\n')
        .filter(|s| !s.is_empty())
        .collect()
}
const MATCHED: &[u8] = include_bytes!("fixtures/original-reference/4888-matched.jsonl");
#[test]
fn actual_sibling_and_declined_frames_preserve_acceptance() {
    let m = lines(MATCHED);
    assert_eq!(m.len(), 2);
    assert!(pair(m[0], Some(m[1]), true).unwrap().is_some());
    let limited = lines(include_bytes!(
        "fixtures/original-reference/4888-limited.jsonl"
    ));
    assert_eq!(limited.len(), 1);
    assert!(pair(limited[0], None, true).unwrap().is_none());
    assert!(pair(limited[0], Some(m[1]), true).is_err());
    assert!(pair(m[0], None, true).is_err());
}
#[test]
fn actual_metrics_warning_is_explicit_acceptance_refusal() {
    let m = lines(include_bytes!(
        "fixtures/original-reference/4888-missing.jsonl"
    ));
    assert!(pair(m[0], Some(m[1]), false).is_ok());
    assert_eq!(
        pair(m[0], Some(m[1]), true).unwrap_err().0,
        "reference metrics unavailable"
    );
}
#[test]
fn malformed_stale_duplicate_and_unrequested_siblings_are_rejected() {
    let m = lines(MATCHED);
    for field in ["id", "payload"] {
        let mut v: Value = serde_json::from_slice(m[1]).unwrap();
        if field == "id" {
            v[field] = "different".into();
        } else {
            v[field]["revision"] = 2.into();
        }
        assert!(pair(m[0], Some(&serde_json::to_vec(&v).unwrap()), true).is_err());
    }
    let mut v: Value = serde_json::from_slice(m[0]).unwrap();
    v["payload"]["layout_capabilities"] = serde_json::json!(["display-list-v2", "display-list-v2"]);
    assert!(pair(&serde_json::to_vec(&v).unwrap(), Some(m[1]), true).is_err());
    assert!(pair(m[0], Some(b"{}"), true).is_err());
    assert!(pair(&vec![b' '; 16 * 1024 * 1024 + 1], None, true).is_err());
}
