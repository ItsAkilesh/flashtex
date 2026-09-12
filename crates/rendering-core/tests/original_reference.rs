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

#[test]
fn opt_in_cff_binding_requires_actual_resources_and_current_sources() {
    use flashtex_rendering_core::{pipeline_cff::PipelineCff, Message, SourceSnapshot};
    use std::collections::BTreeMap;
    let Message::Offer(caps) = parse(include_bytes!("fixtures/capabilities.json"))
        .unwrap()
        .message
    else {
        unreachable!()
    };
    let request: Value =
        serde_json::from_slice(include_bytes!("fixtures/original-reference/request.jsonl"))
            .unwrap();
    let mut docs = BTreeMap::from([(
        "main.tex".into(),
        SourceSnapshot {
            revision: 1,
            text: request["payload"]["documents"][0]["text"]
                .as_str()
                .unwrap()
                .into(),
        },
    )]);
    let error = PipelineCff::bind(V2, &caps, &docs, &BTreeMap::new())
        .err()
        .unwrap();
    assert_eq!(error.0, "missing immutable CFF resource");
    docs.get_mut("main.tex").unwrap().revision = 2;
    assert_eq!(
        PipelineCff::bind(V2, &caps, &docs, &BTreeMap::new())
            .err()
            .unwrap()
            .0,
        "source identity mismatch"
    );
}

#[test]
fn actual_font_digest_is_engine_identity_not_raw_resource() {
    let fixtures =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/original-reference");
    let font = std::fs::read(fixtures.join("lmroman12-regular.otf")).unwrap();
    let v: Value = serde_json::from_slice(V2).unwrap();
    assert_ne!(v["payload"]["fonts"][0]["sha256"], digest(&font));
    let mut engine_input = font.clone();
    engine_input.extend_from_slice(&[0; 4]);
    assert_eq!(v["payload"]["fonts"][0]["sha256"], digest(&engine_input));
    // Binary invocation is intentionally left to the explicit documented probe;
    // this test pins the actual producer refusal condition without fabricating
    // a corrected original-output artifact.
}

#[test]
fn explicit_cff_contract_binds_bytes_and_budgets_atomically() {
    use flashtex_font_resources::{cff::HintPolicy, registry::*};
    use flashtex_rendering_core::{
        mixed::MixedLimits, pipeline_cff::PipelineCff, Message, SourceSnapshot,
    };
    use std::collections::BTreeMap;
    let dir = tempfile::tempdir().unwrap();
    let font = include_bytes!("fixtures/original-reference/lmroman12-regular.otf");
    let license = include_bytes!("fixtures/original-reference/GUST-FONT-LICENSE.txt");
    std::fs::write(dir.path().join("font.otf"), font).unwrap();
    std::fs::write(dir.path().join("LICENSE"), license).unwrap();
    let mut hypothetical: Value = serde_json::from_slice(include_bytes!(
        "fixtures/original-reference/matched-v2.json"
    ))
    .unwrap();
    let mut descriptor = hypothetical["payload"]["fonts"][0].clone();
    descriptor["format"] = "static-cff".into();
    descriptor["sha256"] = digest(font).into();
    let binding = serde_json::json!({"family":"LM","weight":400,"style":"upright"});
    let manifest = serde_json::json!({"schema_version":1,"entries":[{"binding":binding,"resource":{"font":descriptor,"path":"font.otf","license":{"identifier":"GUST","copyright":"See supplied license","source":"official LM2.004","text_path":"LICENSE","text_sha256":digest(license),"embedding_permission":"unknown"}}}]});
    std::fs::write(
        dir.path().join("fonts.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let root = flashtex_project_files::ProjectRoot::open(dir.path()).unwrap();
    let registry =
        ProjectFontRegistry::load(&root, "fonts.json", RegistryLimits::default()).unwrap();
    let RegistryResource::Cff(resource) = registry
        .resource(&serde_json::from_value(binding).unwrap())
        .unwrap()
    else {
        unreachable!()
    };
    let resources = BTreeMap::from([(descriptor["font_id"].as_str().unwrap().into(), resource)]);
    let req: Value =
        serde_json::from_slice(include_bytes!("fixtures/original-reference/request.jsonl"))
            .unwrap();
    let docs = BTreeMap::from([(
        "main.tex".into(),
        SourceSnapshot {
            revision: 1,
            text: req["payload"]["documents"][0]["text"]
                .as_str()
                .unwrap()
                .into(),
        },
    )]);
    let Message::Offer(caps) = parse(include_bytes!("fixtures/capabilities.json"))
        .unwrap()
        .message
    else {
        unreachable!()
    };
    assert_eq!(
        PipelineCff::bind(
            &serde_json::to_vec(&hypothetical).unwrap(),
            &caps,
            &docs,
            &resources
        )
        .err()
        .unwrap()
        .0,
        "CFF resource metadata mismatch"
    );
    // Explicit hypothetical contract test, never saved as actual producer output.
    hypothetical["payload"]["fonts"][0]["sha256"] = digest(font).into();
    let bound = PipelineCff::bind(
        &serde_json::to_vec(&hypothetical).unwrap(),
        &caps,
        &docs,
        &resources,
    )
    .unwrap();
    let batch = bound
        .page(0, HintPolicy::Unhinted, MixedLimits::default())
        .unwrap();
    assert!(!batch.primitives().is_empty());
    let searchable = bound.export_searchable(8 * 1024 * 1024).unwrap();
    assert!(bound.export_searchable(1).is_err());
    flashtex_pdf::verify::check_structure(&searchable.bytes).unwrap();
    let pdf = flashtex_pdf::reader::PdfFile::parse(&searchable.bytes).unwrap();
    let pages = pdf.pages().unwrap();
    let fonts = pdf.page_fonts(pages[0]);
    let exported_font = flashtex_pdf::compare::font_from_dict(&pdf, fonts["F1"]).unwrap();
    let flashtex_pdf::exact::ExactFont::CidCff(cid) = exported_font else {
        panic!("expected original-GID CFF subset")
    };
    let unicode =
        flashtex_pdf::exact::parse_to_unicode(cid.to_unicode_verbatim.as_deref().unwrap()).unwrap();
    assert_eq!(unicode.get(&62).map(String::as_str), Some("H"));
    assert!(unicode.values().any(|s| s == "fi"));
    // Published producer65dbe7d, without JSON repair, is now accepted end to end.
    let published = include_bytes!("fixtures/original-reference/65dbe7d-clean-v2.json");
    let published_resources =
        BTreeMap::from([(digest(font), resources.values().next().unwrap().clone())]);
    let published = PipelineCff::bind(published, &caps, &docs, &published_resources).unwrap();
    let original_pdf = published.export_searchable(8 * 1024 * 1024).unwrap();
    let escaped_request: Value = serde_json::from_slice(include_bytes!(
        "fixtures/original-reference/escaped-request.jsonl"
    ))
    .unwrap();
    let escaped_docs = BTreeMap::from([(
        "main.tex".into(),
        SourceSnapshot {
            revision: 1,
            text: escaped_request["payload"]["documents"][0]["text"]
                .as_str()
                .unwrap()
                .into(),
        },
    )]);
    let escaped = PipelineCff::bind(
        include_bytes!("fixtures/original-reference/escaped-display.json"),
        &caps,
        &escaped_docs,
        &published_resources,
    )
    .unwrap();
    assert_eq!(
        escaped.export_searchable(8 * 1024 * 1024).unwrap().bytes,
        include_bytes!("fixtures/original-reference/escaped-searchable.pdf")
    );

    let unavailable = PipelineCff::bind(
        include_bytes!("fixtures/original-reference/65dbe7d-required-unavailable.json"),
        &caps,
        &docs,
        &published_resources,
    )
    .unwrap();
    assert_eq!(
        unavailable
            .export_searchable(8 * 1024 * 1024)
            .err()
            .unwrap()
            .0,
        "error diagnostics prevent searchable export"
    );
    assert_eq!(
        original_pdf.bytes,
        include_bytes!("fixtures/original-reference/65dbe7d-clean-searchable.pdf")
    );

    // Explicit consumer extraction fixture with a real empty-outline space,
    // repeated original GIDs, and a single-glyph multi-character ligature.
    use flashtex_font_engine::Face;
    let face = flashtex_font_engine::TrueTypeFace::parse(font.to_vec()).unwrap();
    let space = face.glyph_id(' ').unwrap().0;
    let fi = *unicode.iter().find(|(_, s)| s.as_str() == "fi").unwrap().0;
    let text = "H H fi";
    let mut extraction = hypothetical.clone();
    let mut run = extraction["payload"]["pages"][0]["items"][0].clone();
    run["text"] = text.into();
    let spans = [
        (0, 1, 62u16),
        (1, 2, space),
        (2, 3, 62u16),
        (3, 4, space),
        (4, 6, fi),
    ];
    run["glyphs"]=serde_json::json!(spans.iter().enumerate().map(|(i,(_,_,gid))|serde_json::json!({"gid":gid,"origin_x":75497472+i as i64*10000000,"baseline_y":88033374,"advance_x":10000000,"advance_y":0,"cluster":i})).collect::<Vec<_>>());
    run["clusters"]=serde_json::json!(spans.iter().enumerate().map(|(i,(a,b,_))|serde_json::json!({"text_start_byte":a,"text_end_byte":b,"hit_rects":[{"x":75497472+i as i64*10000000,"top":78033374,"width":10000000,"height":10000000}],"carets":[{"text_byte":a,"x":75497472+i as i64*10000000,"top":78033374,"height":10000000}],"sources":[{"path":"main.tex","start_byte":a,"end_byte":b}]})).collect::<Vec<_>>());
    extraction["payload"]["pages"][0]["items"] = serde_json::json!([run]);
    extraction["payload"]["documents"][0]["byte_length"] = text.len().into();
    extraction["payload"]["documents"][0]["sha256"] = digest(text.as_bytes()).into();
    let extraction_docs = BTreeMap::from([(
        "main.tex".into(),
        SourceSnapshot {
            revision: 1,
            text: text.into(),
        },
    )]);
    let extraction = PipelineCff::bind(
        &serde_json::to_vec(&extraction).unwrap(),
        &caps,
        &extraction_docs,
        &resources,
    )
    .unwrap();
    let exported = extraction.export_searchable(8 * 1024 * 1024).unwrap();
    let file = flashtex_pdf::reader::PdfFile::parse(&exported.bytes).unwrap();
    let page = file.pages().unwrap()[0];
    let fonts = file.page_fonts(page);
    let flashtex_pdf::exact::ExactFont::CidCff(cid) =
        flashtex_pdf::compare::font_from_dict(&file, fonts["F1"]).unwrap()
    else {
        unreachable!()
    };
    let mapping =
        flashtex_pdf::exact::parse_to_unicode(cid.to_unicode_verbatim.as_deref().unwrap()).unwrap();
    let mut extracted = String::new();
    for op in flashtex_pdf::exact::parse(&file.page_content(page).unwrap()).unwrap() {
        if let flashtex_pdf::exact::Op::ShowText(bytes) = op {
            let (gids, remainder) = bytes.as_chunks::<2>();
            assert!(remainder.is_empty());
            for g in gids {
                extracted.push_str(&mapping[&u16::from_be_bytes([g[0], g[1]])]);
            }
        }
    }
    assert_eq!(extracted, text);

    let mut ambiguous = hypothetical.clone();
    let repeated_gid = ambiguous["payload"]["pages"][0]["items"][0]["glyphs"][1]["gid"].clone();
    ambiguous["payload"]["pages"][0]["items"][0]["glyphs"][0]["gid"] = repeated_gid;
    let ambiguous = PipelineCff::bind(
        &serde_json::to_vec(&ambiguous).unwrap(),
        &caps,
        &docs,
        &resources,
    )
    .unwrap();
    assert_eq!(
        ambiguous
            .export_searchable(8 * 1024 * 1024)
            .err()
            .unwrap()
            .0,
        "ambiguous GID text requires ActualText support"
    );
    let mut multiple = hypothetical.clone();
    let g = multiple["payload"]["pages"][0]["items"][0]["glyphs"][0].clone();
    multiple["payload"]["pages"][0]["items"][0]["glyphs"]
        .as_array_mut()
        .unwrap()
        .push(g);
    let multiple = PipelineCff::bind(
        &serde_json::to_vec(&multiple).unwrap(),
        &caps,
        &docs,
        &resources,
    )
    .unwrap();
    assert_eq!(
        multiple.export_searchable(8 * 1024 * 1024).err().unwrap().0,
        "multi-glyph cluster requires ActualText support"
    );

    assert!(batch
        .primitives()
        .iter()
        .all(|p| p.font_sha256.as_deref() == Some(digest(font).as_str()) && !p.sources.is_empty()));
    assert!(bound
        .page(
            0,
            HintPolicy::Unhinted,
            MixedLimits {
                max_commands: 1,
                ..MixedLimits::default()
            }
        )
        .is_err());
    assert!(bound
        .page(
            0,
            HintPolicy::Unhinted,
            MixedLimits {
                max_primitives: 1,
                ..MixedLimits::default()
            }
        )
        .is_err());
}

#[test]
fn reviewable_producer_candidate_changes_only_raw_digest() {
    let base: Value = serde_json::from_slice(
        include_bytes!("fixtures/original-reference/4888-matched.jsonl")
            .split(|b| *b == b'\n')
            .nth(1)
            .unwrap(),
    )
    .unwrap();
    let candidate: Value = serde_json::from_slice(include_bytes!(
        "../docs/handoffs/pipeline-4888a67-candidate.json"
    ))
    .unwrap();
    let raw = digest(include_bytes!(
        "fixtures/original-reference/lmroman12-regular.otf"
    ));
    let mut expected = base.clone();
    expected["payload"]["fonts"][0]["sha256"] = raw.into();
    assert_eq!(candidate, expected);
    assert_eq!(
        candidate["payload"]["fonts"][0]["font_id"],
        base["payload"]["fonts"][0]["font_id"]
    );
}

#[test]
fn actual_escape_text_is_distinct_from_tex_source_spelling() {
    let request: Value = serde_json::from_slice(include_bytes!(
        "fixtures/original-reference/escaped-request.jsonl"
    ))
    .unwrap();
    let source = request["payload"]["documents"][0]["text"].as_str().unwrap();
    let display: Value = serde_json::from_slice(include_bytes!(
        "fixtures/original-reference/escaped-display.json"
    ))
    .unwrap();
    assert!(display["payload"]["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
    for symbol in ["%", "_", "&", "#", "{", "}"] {
        let run = display["payload"]["pages"][0]["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["text"] == symbol)
            .unwrap();
        let span = &run["clusters"][0]["sources"][0];
        assert_eq!(
            &source[span["start_byte"].as_u64().unwrap() as usize
                ..span["end_byte"].as_u64().unwrap() as usize],
            format!("\\{symbol}")
        );
    }
    assert_eq!(
        include_str!("fixtures/original-reference/escaped-extracted.txt"),
        "Escaped % _ & # { } and office fi.\n\n\u{c}"
    );
}
