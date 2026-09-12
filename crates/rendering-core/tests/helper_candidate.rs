use flashtex_font_resources::registry::*;
use flashtex_rendering_core::{helper_candidate::*, pipeline_cff::PipelineCff, *};
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::Arc};
fn resources(display: &Value) -> BTreeMap<String, Arc<CffFontResource>> {
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
    resources
}
fn fixture(bytes: &[u8]) -> (Value, Vec<u8>, CurrentHelper, Value) {
    let fixture: Value = serde_json::from_slice(bytes).unwrap();
    let events = fixture["events"].as_array().unwrap();
    let event = events
        .iter()
        .find(|e| e["payload"]["kind"] == "display_candidate")
        .unwrap()
        .clone();
    let result = events
        .iter()
        .find(|e| e["payload"]["kind"] == "preview")
        .unwrap()["payload"]["result"]
        .clone();
    let p = &event["payload"];
    let sources = fixture["request"]["payload"]["documents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            let path = d["path"].as_str().unwrap();
            (
                path.into(),
                CurrentSource {
                    editor_revision: p["source_versions"][path].as_u64().unwrap(),
                    text: d["text"].as_str().unwrap().into(),
                },
            )
        })
        .collect();
    let current = CurrentHelper {
        session_id: event["session_id"].as_str().unwrap().into(),
        project_id: p["project_id"].as_str().unwrap().into(),
        request_id: p["request_id"].as_str().unwrap().into(),
        compile_revision: p["compile_revision"].as_u64().unwrap(),
        membership_generation: p["membership_generation"].as_u64().unwrap(),
        sources,
    };
    (
        event,
        serde_json::to_vec(&result).unwrap(),
        current,
        fixture,
    )
}
fn caps() -> Capabilities {
    let Message::Offer(caps) = parse(include_bytes!("fixtures/capabilities.json"))
        .unwrap()
        .message
    else {
        unreachable!()
    };
    caps
}
#[test]
fn actual_helper_three_source_states_export_without_metadata_repair() {
    for bytes in [
        include_bytes!("fixtures/runtime-candidates/step-0.json").as_slice(),
        include_bytes!("fixtures/runtime-candidates/step-1.json").as_slice(),
        include_bytes!("fixtures/runtime-candidates/step-2.json").as_slice(),
    ] {
        let (event, result, current, fixture) = fixture(bytes);
        let sibling = &event["payload"]["display_list"];
        assert_eq!(sibling, &fixture["direct"][1]);
        assert_ne!(
            current.compile_revision,
            current.sources["main.tex"].editor_revision
        );
        let resources = resources(sibling);
        let bound = bind(
            &serde_json::to_vec(&event).unwrap(),
            &result,
            &current,
            &caps(),
            &resources,
        )
        .unwrap();
        let pdf = bound.export_searchable(&current, 8 * 1024 * 1024).unwrap();
        let docs = current
            .sources
            .iter()
            .map(|(p, s)| {
                (
                    p.clone(),
                    SourceSnapshot {
                        revision: current.compile_revision,
                        text: s.text.clone(),
                    },
                )
            })
            .collect();
        let direct = PipelineCff::bind(
            &serde_json::to_vec(&fixture["direct"][1]).unwrap(),
            &caps(),
            &docs,
            &resources,
        )
        .unwrap()
        .export_searchable(8 * 1024 * 1024)
        .unwrap();
        assert_eq!(pdf.bytes, direct.bytes);
        assert!(pdf.bytes.starts_with(b"%PDF-"));
        let mut stale = current.clone();
        stale.membership_generation += 1;
        assert!(bound.export_searchable(&stale, 8 * 1024 * 1024).is_err());
    }
}
#[test]
fn stale_editor_session_membership_and_corrupt_resources_are_refused() {
    let (event, result, current, _) =
        fixture(include_bytes!("fixtures/runtime-candidates/step-1.json"));
    let resources = resources(&event["payload"]["display_list"]);
    let caps = caps();
    let bytes = serde_json::to_vec(&event).unwrap();
    for axis in 0..5 {
        let mut stale = current.clone();
        match axis {
            0 => stale.session_id.push('x'),
            1 => stale.membership_generation += 1,
            2 => stale.compile_revision += 1,
            3 => stale.sources.get_mut("main.tex").unwrap().editor_revision += 1,
            _ => stale.sources.get_mut("main.tex").unwrap().text.push('x'),
        };
        assert!(bind(&bytes, &result, &stale, &caps, &resources).is_err());
    }
    for pointer in [
        "/payload/display_list/payload/fonts/0/sha256",
        "/payload/display_list/payload/fonts/0/glyph_count",
        "/payload/display_list/payload/pages/0/items/0/glyphs/0/gid",
    ] {
        let mut corrupt = event.clone();
        *corrupt.pointer_mut(pointer).unwrap() = if pointer.ends_with("sha256") {
            Value::String("0".repeat(64))
        } else {
            json!(999999)
        };
        assert!(bind(
            &serde_json::to_vec(&corrupt).unwrap(),
            &result,
            &current,
            &caps,
            &resources
        )
        .is_err());
    }
    assert!(bind(&bytes, &result, &current, &caps, &BTreeMap::new()).is_err());
    assert!(bind(b"{", &result, &current, &caps, &resources).is_err());
}
#[test]
fn actual_runtime_raw_sibling_pairs_and_exports_without_repair() {
    let lines = include_bytes!("fixtures/runtime-candidates/requested.stdout.jsonl")
        .split(|b| *b == b'\n')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    assert!(pipeline_frame::pair(lines[0], Some(lines[1]), true)
        .unwrap()
        .is_some());
    let display: Value = serde_json::from_slice(lines[1]).unwrap();
    let resources = resources(&display);
    let request: Value = serde_json::from_slice(include_bytes!(
        "fixtures/runtime-candidates/requested.request.jsonl"
    ))
    .unwrap();
    let docs = BTreeMap::from([(
        "main.tex".into(),
        SourceSnapshot {
            revision: 1,
            text: request["payload"]["documents"][0]["text"]
                .as_str()
                .unwrap()
                .into(),
        },
    )]);
    assert!(PipelineCff::bind(lines[1], &caps(), &docs, &resources)
        .unwrap()
        .export_searchable(8 * 1024 * 1024)
        .is_ok());
    for raw in [
        include_bytes!("fixtures/runtime-candidates/declined.stdout.jsonl").as_slice(),
        include_bytes!("fixtures/runtime-candidates/legacy.stdout.jsonl").as_slice(),
        include_bytes!("fixtures/runtime-candidates/failed.stdout.jsonl").as_slice(),
    ] {
        assert!(pipeline_frame::pair(raw, None, false).unwrap().is_none());
        assert!(pipeline_frame::pair(raw, Some(lines[1]), false).is_err());
    }
}

#[test]
fn native_migration_cases_reuse_actual_helper_envelope_and_existing_binder() {
    let suite: Value = serde_json::from_slice(include_bytes!(
        "../docs/handoffs/native-helper-interop/cases.json"
    ))
    .unwrap();
    let raw = include_bytes!("fixtures/runtime-candidates/step-1.json");
    assert_eq!(suite["base_sha256"], digest(raw));
    let (base, result, current, _) = fixture(raw);
    let resources = resources(&base["payload"]["display_list"]);
    for case in suite["cases"].as_array().unwrap() {
        let mut event = base.clone();
        let mut state = current.clone();
        for (key, value) in case["current"].as_object().unwrap() {
            match key.as_str() {
                "editor_revision" => {
                    state.sources.get_mut("main.tex").unwrap().editor_revision =
                        value.as_u64().unwrap()
                }
                "membership_generation" => state.membership_generation = value.as_u64().unwrap(),
                "compile_revision" => state.compile_revision = value.as_u64().unwrap(),
                "session_id" => state.session_id = value.as_str().unwrap().into(),
                "append_source" => state
                    .sources
                    .get_mut("main.tex")
                    .unwrap()
                    .text
                    .push_str(value.as_str().unwrap()),
                _ => panic!("unrecognized fixture mutation"),
            }
        }
        for (pointer, value) in case["candidate"].as_object().unwrap() {
            *event.pointer_mut(pointer).unwrap() = value.clone();
        }
        let actual = bind(
            &serde_json::to_vec(&event).unwrap(),
            &result,
            &state,
            &caps(),
            &resources,
        );
        assert_eq!(
            actual.is_ok(),
            case["accept"].as_bool().unwrap(),
            "{}",
            case["name"]
        );
    }
}

#[test]
fn actual_raw_normalized_and_escaped_json_preserve_bound_pdf_identity() {
    let lines = include_bytes!("fixtures/runtime-candidates/requested.stdout.jsonl")
        .split(|b| *b == b'\n')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let value: Value = serde_json::from_slice(lines[1]).unwrap();
    let request: Value = serde_json::from_slice(include_bytes!(
        "fixtures/runtime-candidates/requested.request.jsonl"
    ))
    .unwrap();
    let resources = resources(&value);
    let docs = BTreeMap::from([(
        "main.tex".into(),
        SourceSnapshot {
            revision: 1,
            text: request["payload"]["documents"][0]["text"]
                .as_str()
                .unwrap()
                .into(),
        },
    )]);
    let normalized = serde_json::to_vec(&value).unwrap();
    let escaped = String::from_utf8(normalized.clone())
        .unwrap()
        .replace("main.tex", "main\\u002etex")
        .replace("Office", "Off\\u0069ce");
    let original = PipelineCff::bind(lines[1], &caps(), &docs, &resources)
        .unwrap()
        .export_searchable(8 * 1024 * 1024)
        .unwrap();
    for bytes in [normalized.as_slice(), escaped.as_bytes()] {
        let pdf = PipelineCff::bind(bytes, &caps(), &docs, &resources)
            .unwrap()
            .export_searchable(8 * 1024 * 1024)
            .unwrap();
        assert_eq!(pdf.bytes, original.bytes);
    }
    let mut exact = value.clone();
    exact["payload"]["revision"] = json!(MAX_EXACT_INTEGER);
    exact["payload"]["documents"][0]["revision"] = json!(MAX_EXACT_INTEGER);
    let mut current = docs.clone();
    current.get_mut("main.tex").unwrap().revision = MAX_EXACT_INTEGER as u64;
    assert!(PipelineCff::bind(
        &serde_json::to_vec(&exact).unwrap(),
        &caps(),
        &current,
        &resources
    )
    .is_ok());
    exact["payload"]["revision"] = json!(MAX_EXACT_INTEGER as u64 + 1);
    assert!(PipelineCff::bind(
        &serde_json::to_vec(&exact).unwrap(),
        &caps(),
        &current,
        &resources
    )
    .is_err());
    for token in ["1.0", "1e0", "9007199254740993", "18446744073709551616"] {
        let altered = String::from_utf8(normalized.clone()).unwrap().replacen(
            "\"revision\":1",
            &format!("\"revision\":{token}"),
            1,
        );
        assert!(
            PipelineCff::bind(altered.as_bytes(), &caps(), &docs, &resources).is_err(),
            "{token}"
        );
    }
}

#[test]
fn duplicate_field_representation_gap_is_explicit_before_raw_transport_activation() {
    let lines = include_bytes!("fixtures/runtime-candidates/requested.stdout.jsonl")
        .split(|b| *b == b'\n')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let raw = std::str::from_utf8(lines[1]).unwrap();
    let duplicate = raw.replacen("\"id\":", "\"id\":\"conflicting\",\"id\":", 1);
    assert!(parse(duplicate.as_bytes()).is_err());
    let normalized =
        serde_json::to_vec(&serde_json::from_str::<Value>(&duplicate).unwrap()).unwrap();
    assert!(
        parse(&normalized).is_ok(),
        "known normalization erases duplicate envelope field"
    );
    let nested = raw.replacen("\"glyph_count\":", "\"glyph_count\":0,\"glyph_count\":", 1);
    assert!(
        parse(nested.as_bytes()).is_err(),
        "nested duplicate must be refused before normalization"
    );
}

#[test]
fn raw_helper_preserves_nested_duplicate_evidence_and_strict_source_map() {
    let (event, result, current, _) =
        fixture(include_bytes!("fixtures/runtime-candidates/step-1.json"));
    let resources = resources(&event["payload"]["display_list"]);
    let raw = serde_json::to_string(&event).unwrap();
    for (field, prefix) in [
        ("\"glyph_count\":", "\"glyph_count\":0,\"glyph_count\":"),
        (
            "\"glyph_count\":",
            "\"glyph_\\u0063ount\":0,\"glyph_count\":",
        ),
        ("\"origin_x\":", "\"origin_x\":0,\"origin_x\":"),
        (
            "\"source_actions_enabled\":",
            "\"source_actions_enabled\":true,\"source_actions_enabled\":",
        ),
        ("\"main.tex\":2", "\"main\\u002etex\":999,\"main.tex\":2"),
    ] {
        let invalid = raw.replacen(field, prefix, 1);
        assert!(invalid != raw, "mutation must change the actual fixture");
        assert!(
            bind(invalid.as_bytes(), &result, &current, &caps(), &resources).is_err(),
            "{field}"
        );
    }
    for invalid in [
        raw.replacen('{', "{\"extension\":1e400,", 1),
        raw.replacen(
            '{',
            &format!("{{\"extension\":{}0{},", "[".repeat(130), "]".repeat(130)),
            1,
        ),
        raw.replace("main.tex", "main\\ud800.tex"),
        raw.replacen("\"origin_x\":134651073", "\"origin_x\":1e400", 1),
    ] {
        assert!(invalid != raw, "mutation must change the actual fixture");
        assert!(bind(invalid.as_bytes(), &result, &current, &caps(), &resources).is_err());
    }
}
