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
    let pdf = export_math(DISPLAY, &request);
    assert_eq!(pdf, include_bytes!("fixtures/math-reference/original.pdf"));
    let reference = include_bytes!("fixtures/math-reference/reference.pdf");
    let engine: Value = serde_json::from_slice(include_bytes!(
        "fixtures/math-reference/reference-engine.json"
    ))
    .unwrap();
    assert_eq!(engine["pdf_sha256"], digest(reference));
    let comparison =
        flashtex_rendering_core::pdf_compare::compare(&pdf, reference, None, Default::default())
            .unwrap();
    assert_eq!(comparison.value()["raw_bytes_equal"], false);
    assert_eq!(comparison.value()["visual_equal"], Value::Null);
    assert_eq!(
        include_bytes!("fixtures/math-reference/original.txt"),
        include_bytes!("fixtures/math-reference/reference.txt")
    );
}

fn export_math(display_bytes: &[u8], request: &Value) -> Vec<u8> {
    let display: Value = serde_json::from_slice(display_bytes).unwrap();
    let source = request["payload"]["documents"][0]["text"].as_str().unwrap();
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
    let bound = PipelineCff::bind(display_bytes, &caps, &docs, &resources).unwrap();
    let pdf = bound.export_searchable(8 * 1024 * 1024).unwrap();
    pdf.bytes
}

#[test]
fn actual_display_math_preserves_sum_and_reference_mapping_difference() {
    let display = include_bytes!("fixtures/display-math-reference/display.json");
    let request: Value = serde_json::from_slice(include_bytes!(
        "fixtures/display-math-reference/request.jsonl"
    ))
    .unwrap();
    let raw: Value = serde_json::from_slice(display).unwrap();
    assert_eq!(raw["payload"]["diagnostics"], json!([]));
    let pdf = export_math(display, &request);
    assert_eq!(
        pdf,
        include_bytes!("fixtures/display-math-reference/original.pdf")
    );
    let original_text = include_str!("fixtures/display-math-reference/original.txt");
    let reference_text = include_str!("fixtures/display-math-reference/reference.txt");
    assert!(original_text.contains('∑'));
    assert!(!reference_text.contains('∑'));
    assert_eq!(original_text.replace('∑', "X"), reference_text);
    let reference = include_bytes!("fixtures/display-math-reference/reference.pdf");
    let file = flashtex_pdf::reader::PdfFile::parse(reference).unwrap();
    let pages = file.pages().unwrap();
    let fonts = file.page_fonts(pages[0]);
    let font = fonts["F53"];
    let cmap = file
        .decode_stream(file.resolve(&font["ToUnicode"]))
        .unwrap();
    let unicode = flashtex_pdf::exact::parse_to_unicode(&cmap).unwrap();
    assert_eq!(unicode.get(&88), None);
    assert_eq!(
        cmap,
        include_bytes!("fixtures/display-math-reference/reference-F53-cmap.txt")
    );
    let original = flashtex_pdf::reader::PdfFile::parse(&pdf).unwrap();
    let pages = original.pages().unwrap();
    let fonts = original.page_fonts(pages[0]);
    let cmap = original
        .decode_stream(original.resolve(&fonts["F1"]["ToUnicode"]))
        .unwrap();
    let unicode = flashtex_pdf::exact::parse_to_unicode(&cmap).unwrap();
    assert_eq!(unicode.get(&3060).map(String::as_str), Some("∑"));
    let engine: Value = serde_json::from_slice(include_bytes!(
        "fixtures/display-math-reference/reference-engine.json"
    ))
    .unwrap();
    assert_eq!(engine["pdf_sha256"], digest(reference));
}
