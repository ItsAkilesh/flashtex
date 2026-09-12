//! Actual untouched producer math, existing immutable loader/exporter/reference.
use flashtex_font_resources::registry::*;
use flashtex_rendering_core::{digest, parse, pipeline_cff::PipelineCff, Message, SourceSnapshot};
use serde_json::{json, Value};
use std::collections::BTreeMap;
const DISPLAY: &[u8] = include_bytes!("fixtures/math-reference/display.json");
#[test]
fn published_math_source_clusters_and_real_pdf_remain_exact() {
    let display: Value = serde_json::from_slice(DISPLAY).unwrap();
    assert!(display["payload"]["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
    let request: Value =
        serde_json::from_slice(include_bytes!("fixtures/math-reference/request.jsonl")).unwrap();
    let source = request["payload"]["documents"][0]["text"].as_str().unwrap();
    let greek = display["payload"]["pages"][0]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["text"] == "α+β")
        .unwrap();
    let span = &greek["clusters"][0]["sources"][0];
    let tex = &source[span["start_byte"].as_u64().unwrap() as usize
        ..span["end_byte"].as_u64().unwrap() as usize];
    assert!(tex.contains("\\alpha") && tex.contains("\\beta"));
    let dir = tempfile::tempdir().unwrap();
    let math = include_bytes!("fixtures/math-reference/latinmodern-math.otf");
    assert_eq!(
        digest(math),
        "6075562b771f8b82f0c179e363389684f2dd09de30038269e2628e504bd7be0f"
    );
    let text = include_bytes!("fixtures/original-reference/lmroman12-regular.otf");
    let license = include_bytes!("fixtures/math-reference/GUST-FONT-LICENSE.TXT");
    std::fs::write(dir.path().join("LICENSE"), license).unwrap();
    let bytes = BTreeMap::from([
        (digest(math), math.as_slice()),
        (digest(text), text.as_slice()),
    ]);
    let mut entries = Vec::new();
    for (index, f) in display["payload"]["fonts"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let mut descriptor = f.clone();
        descriptor["format"] = "static-cff".into();
        let path = format!("font-{index}.otf");
        std::fs::write(dir.path().join(&path), bytes[f["sha256"].as_str().unwrap()]).unwrap();
        entries.push(json!({"binding":{"family":f["font_id"],"weight":400,"style":"upright"},"resource":{"font":descriptor,"path":path,"license":{"identifier":"GUST","copyright":"See supplied license","source":"unchanged Mac b898cfc asset","text_path":"LICENSE","text_sha256":digest(license),"embedding_permission":"unknown"}}}));
    }
    let manifest = json!({"schema_version":1,"entries":entries});
    std::fs::write(
        dir.path().join("fonts.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let root = flashtex_project_files::ProjectRoot::open(dir.path()).unwrap();
    let registry =
        ProjectFontRegistry::load(&root, "fonts.json", RegistryLimits::default()).unwrap();
    let mut resources = BTreeMap::new();
    for entry in &entries {
        let RegistryResource::Cff(resource) = registry
            .resource(&serde_json::from_value(entry["binding"].clone()).unwrap())
            .unwrap()
        else {
            unreachable!()
        };
        resources.insert(
            entry["binding"]["family"].as_str().unwrap().into(),
            resource,
        );
    }
    let docs = BTreeMap::from([(
        "main.tex".into(),
        SourceSnapshot {
            revision: 1,
            text: source.into(),
        },
    )]);
    let Message::Offer(caps) = parse(include_bytes!("fixtures/capabilities.json"))
        .unwrap()
        .message
    else {
        unreachable!()
    };
    let bound = PipelineCff::bind(DISPLAY, &caps, &docs, &resources).unwrap();
    let pdf = bound.export_searchable(8 * 1024 * 1024).unwrap();
    assert_eq!(
        pdf.bytes,
        include_bytes!("fixtures/math-reference/original.pdf")
    );
    let reference = include_bytes!("fixtures/math-reference/reference.pdf");
    let engine: Value = serde_json::from_slice(include_bytes!(
        "fixtures/math-reference/reference-engine.json"
    ))
    .unwrap();
    assert_eq!(engine["pdf_sha256"], digest(reference));
    let comparison = flashtex_rendering_core::pdf_compare::compare(
        &pdf.bytes,
        reference,
        None,
        Default::default(),
    )
    .unwrap();
    assert_eq!(comparison.value()["raw_bytes_equal"], false);
    assert_eq!(comparison.value()["visual_equal"], Value::Null);
    assert_eq!(
        include_bytes!("fixtures/math-reference/original.txt"),
        include_bytes!("fixtures/math-reference/reference.txt")
    );
}
