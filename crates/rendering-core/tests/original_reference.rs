//! A real original compiler route versus established pdfTeX. Non-equivalence is
//! the observed result; a passing test means the gap remains honestly reported.
use flashtex_rendering_core::{
    digest, parse,
    pdf_compare::{compare, CompareLimits},
};
use serde_json::Value;
const ORIGINAL: &[u8] = include_bytes!("fixtures/original-reference/original.pdf");
const REFERENCE: &[u8] = include_bytes!("fixtures/original-reference/reference.pdf");
const V2: &[u8] = include_bytes!("fixtures/original-reference/original-v2.json");
#[test]
fn established_reference_is_pinned_and_not_reemitted() {
    let engine: Value = serde_json::from_slice(include_bytes!(
        "fixtures/original-reference/reference-engine.json"
    ))
    .unwrap();
    assert_eq!(engine["pdf_sha256"], digest(REFERENCE));
    assert_eq!(
        engine["fixture_sha256"],
        digest(include_bytes!("fixtures/original-reference/source.tex"))
    );
    let r = compare(ORIGINAL, REFERENCE, None, CompareLimits::default()).unwrap();
    assert_eq!(r.value()["raw_bytes_equal"], false);
    assert_eq!(r.value()["parsed_operators_equal"], false);
    assert_eq!(r.value()["reference_reemitted"], false);
    assert_eq!(r.value()["visual_equal"], Value::Null);
    assert_eq!(r.value()["reference_source_correspondence"], "unknown");
}
#[test]
fn compiler_source_identity_and_original_gids_exist_but_wire_is_not_activated() {
    let request: Value =
        serde_json::from_slice(include_bytes!("fixtures/original-reference/request.jsonl"))
            .unwrap();
    let output: Value = serde_json::from_slice(V2).unwrap();
    let text = request["payload"]["documents"][0]["text"].as_str().unwrap();
    assert_eq!(
        output["payload"]["documents"][0]["sha256"],
        digest(text.as_bytes())
    );
    assert_eq!(output["payload"]["fonts"][0]["format"], "opentype-cff");
    assert!(output["payload"]["pages"][0]["items"][0]["glyphs"]
        .as_array()
        .unwrap()
        .iter()
        .all(|g| g["gid"].as_u64().unwrap() > 0));
    // This is a real producer contract disagreement, not a missing glyph map.
    let offer = parse(include_bytes!("fixtures/capabilities.json")).unwrap();
    let error = parse(V2).unwrap().validate(Some(&offer)).unwrap_err();
    assert!(error.0.contains("unsupported font profile"), "{error}");
}
