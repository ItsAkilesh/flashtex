use flashtex_rendering_core::{digest, pdf_compare::*, pdf_export, pdf_stream::*};
use serde_json::{json, Value};
const STIX: &[u8] = include_bytes!("fixtures/stix-exact-export.pdf");
const STIX_EVIDENCE: &[u8] = include_bytes!("fixtures/stix-exact-export.evidence.json");
const SYNTHETIC: &[u8] = include_bytes!("fixtures/synthetic-exact-export.pdf");
#[test]
fn identical_actual_pdf_is_not_a_visual_oracle() {
    let report = compare(STIX, STIX, Some(STIX_EVIDENCE), CompareLimits::default()).unwrap();
    assert_eq!(report.value()["raw_bytes_equal"], true);
    assert_eq!(report.value()["parsed_operators_equal"], true);
    assert_eq!(report.value()["visual_equal"], Value::Null);
    assert_eq!(report.value()["reference_reemitted"], false);
    assert_eq!(report.value()["candidate_sha256"], digest(STIX));
    assert_eq!(report.value()["reference_sha256"], digest(STIX));
}
#[test]
fn changed_geometry_paint_attaches_only_validated_candidate_span() {
    let mut fixture: Value =
        serde_json::from_slice(include_bytes!("fixtures/synthetic-pdf-stream.json")).unwrap();
    fixture["primitives"][0]["geometry"]["commands"][0][1][0] = json!(["3", "1"]);
    fixture["primitives"][0]["paint"]["r"] = 1.into();
    let source = serde_json::to_vec(&fixture).unwrap();
    let stream = PdfCommandStream::from_mixed_replay(&source, StreamLimits::default()).unwrap();
    let candidate = pdf_export::export(&[stream], Default::default()).unwrap();
    let result = compare(
        candidate.bytes(),
        SYNTHETIC,
        Some(candidate.evidence_bytes()),
        CompareLimits::default(),
    )
    .unwrap();
    assert_eq!(result.value()["parsed_operators_equal"], false);
    assert_eq!(result.value()["raw_bytes_equal"], false);
    let diffs = result.value()["differences"].as_array().unwrap();
    assert!(diffs.iter().any(|d| d["kind"] == "geometry"));
    assert!(diffs.iter().any(|d| d["kind"] == "paint"));
    for d in diffs {
        let p = &d["candidate_provenance"];
        assert_eq!(p["original_gid"], 1);
        assert_eq!(p["reference_correspondence"], "unknown");
        assert_eq!(p["font_and_source_claims_independently_verified"], false);
        assert_eq!(p["mapping"], "candidate_operator_span_verified");
    }
    let bad = compare(
        candidate.bytes(),
        SYNTHETIC,
        Some(STIX_EVIDENCE),
        CompareLimits::default(),
    );
    assert!(matches!(bad, Err(CompareError::Evidence(_))));
    let limited = compare(
        candidate.bytes(),
        SYNTHETIC,
        Some(candidate.evidence_bytes()),
        CompareLimits {
            max_differences: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(limited.value()["truncated"], true);
    assert_eq!(limited.value()["parsed_operators_equal"], Value::Null);
}
#[test]
fn identical_unsupported_content_is_unknown_despite_owner_fast_path() {
    let mut pdf = SYNTHETIC.to_vec();
    let at = pdf.windows(3).position(|v| v == b" rg").unwrap();
    pdf[at + 2] = b's';
    let file = flashtex_pdf::reader::PdfFile::parse(&pdf).unwrap();
    assert!(
        flashtex_pdf::exact::parse(&file.page_content(file.pages().unwrap()[0]).unwrap()).is_err()
    );
    let report = compare(&pdf, &pdf, None, CompareLimits::default()).unwrap();
    assert_eq!(report.value()["raw_bytes_equal"], true);
    assert_eq!(report.value()["parsed_operators_equal"], Value::Null);
    assert!(!report.value()["unsupported_pages"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(report.value()["categories"]
        .as_array()
        .unwrap()
        .contains(&json!("ContentUnsupported")));
}
#[test]
fn byte_object_layout_changes_are_separate_from_operators() {
    let mut pdf = SYNTHETIC.to_vec();
    pdf.extend_from_slice(b"\n% caller comparison fixture\n");
    let report = compare(&pdf, SYNTHETIC, None, CompareLimits::default()).unwrap();
    assert_eq!(report.value()["raw_bytes_equal"], false);
    assert_eq!(report.value()["parsed_operators_equal"], true);
    assert_eq!(report.value()["visual_equal"], Value::Null);
    for limits in [
        CompareLimits {
            max_pdf_bytes: 1,
            ..Default::default()
        },
        CompareLimits {
            max_objects: 1,
            ..Default::default()
        },
        CompareLimits {
            max_decoded_bytes: 1,
            ..Default::default()
        },
        CompareLimits {
            max_operators: 1,
            ..Default::default()
        },
        CompareLimits {
            max_report_bytes: 1,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            compare(STIX, STIX, None, limits),
            Err(CompareError::Budget)
        ));
    }
}
#[test]
fn legacy_flashtex_pdf_is_explicit_reference_not_tex_oracle() {
    let reference =
        include_bytes!("../../../tests/tex-corpus/evidence/pdf-a0855dd/plain-paragraphs.pdf");
    assert_eq!(
        digest(reference),
        "ca65e836026239ece90053a638c48055bffd69497b491f10028414fbd62fd9b6"
    );
    let report = compare(
        STIX,
        reference,
        Some(STIX_EVIDENCE),
        CompareLimits::default(),
    )
    .unwrap();
    assert_eq!(report.value()["parsed_operators_equal"], false);
    assert_eq!(report.value()["reference_sha256"], digest(reference));
    assert_eq!(report.value()["reference_source_correspondence"], "unknown");
    assert!(!report.value()["categories"].as_array().unwrap().is_empty());
}

#[test]
fn repeated_page_graph_is_rejected_before_owner_page_expansion() {
    let mut pdf = SYNTHETIC.to_vec();
    let start = pdf.windows(6).position(|v| v == b"/Kids ").unwrap() + 6;
    let open = start + pdf[start..].iter().position(|b| *b == b'[').unwrap();
    let close = open + pdf[open..].iter().position(|b| *b == b']').unwrap();
    let repeated = pdf[open + 1..close].to_vec();
    pdf.splice(close..close, repeated);
    assert!(matches!(
        compare(&pdf, SYNTHETIC, None, CompareLimits::default()),
        Err(CompareError::Unsupported(_))
    ));
}

#[test]
fn expanded_repeated_content_budget_precedes_concatenation() {
    let mut pdf = SYNTHETIC.to_vec();
    let start = pdf.windows(10).position(|v| v == b"/Contents ").unwrap() + 10;
    let end = start + pdf[start..].iter().position(|b| *b == b'R').unwrap() + 1;
    let part = pdf[start..end].to_vec();
    let mut array = b"[".to_vec();
    for _ in 0..10 {
        array.extend(&part);
        array.push(b' ')
    }
    array.push(b']');
    pdf.splice(start..end, array);
    assert!(matches!(
        compare(
            &pdf,
            SYNTHETIC,
            None,
            CompareLimits {
                max_decoded_bytes: 1000,
                ..Default::default()
            }
        ),
        Err(CompareError::Budget)
    ));
}
